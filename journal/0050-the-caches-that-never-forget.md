# 0050 — The caches that never forget

The DeviceLost crashes (ROADMAP Observed: horizon-6 dies in minutes, texture
*smearing* at horizon-3 after heavy teleporting) had been filed as a renderer
leak — "the pooling doctrine's failure class," far-field tiles accumulated and
never released. That framing named `farmesh.rs` as the suspect. The framing was
wrong, and the instrument is what proved it wrong.

## The wrong turn we were told to take

The prior evidence pointed at the far field: crashes only at wide horizons,
lifetime scaling with far-field size (4.5 min at horizon 6 vs 44 min at
horizon 3), smearing as a VRAM-pressure symptom. Every arrow pointed below the
waterline at wgpu/GPU memory or at our own tile pooling. So the first move was
to *count our own allocations* rather than guess: a `DC_MEM_PROBE` plugin
(env-gated, zero-cost off, modelled on the existing `GpuProbePlugin`) logging
every ~10 s the sizes of the things we own — `ChunkMap`, `FarSurfaceMap`,
`FarChunkMap`, their live entities, and the `Assets<Mesh>` / `Assets<Image>`
stores those meshes and textures live in. Alongside it, an external sampler
(`nvidia-smi` + `Get-Process` working set) for the two numbers our own counts
can't see: GPU VRAM and process RSS.

## What the instrument said

**Idle horizon 6 does not leak.** Booted, let the field stream in (~10 s), then
sat untouched for 6.5 minutes. Every counter flat after fill: `meshes=790`,
`images=14`, `far_tiles=704`, `entities=1191` — unchanged to the byte across
370 s. RSS flat at ~978 MB, VRAM flat at ~3158 MB. No crash. This *falsifies*
the "idle leak" reading as reproduced on this build/driver: with the viewer
still, nothing grows. The leak is motion-driven, not time-driven.

**A teleport storm leaks, linearly, in host RAM.** Driving the live game over
its own MCP (`client_player_pose_set`, `surface:true`) to jump every 1.8 s to a
*fresh distant* coordinate (±15 km), horizon 3:

| t (s) | RSS (MB) | VRAM (MB) | jumps |
|------:|---------:|----------:|------:|
| 11 | 1048 | 3163 | 6 |
| 120 | 2950 | 3174 | 65 |
| 250 | 4662 | 3162 | 135 |
| 358 | 5871 | 3158 | 194 |

RSS climbs dead-straight, **~25 MB per jump / ~13 MB/s**, unbounded. VRAM never
moves. And crucially our own counts *oscillate but never grow* — `meshes` rode
26–133, `entities` 427–534, `far_tiles` 4–68, rising as each jump refills the
near/far field and falling as the old field is torn down. The mesh assets and
render entities are being freed correctly. The pooling doctrine is not failing.
Something below our counters, on the CPU, is not.

**The decisive cut: revisit vs. explore.** If the growth is per-teleport churn
(mesh create/destroy, render-world retention) it should climb no matter *where*
you jump. If it is per-*region* it should stop the moment you stop visiting new
regions. So: the same storm, but ping-ponging among **four fixed** locations.
RSS warmed to ~930 MB on the first cycle and then stayed **flat within 4 MB for
142 jumps / 250 s**. Revisiting is free. Only new ground costs memory. The leak
is keyed by world position, not by render work.

Horizon 6 under the same storm gave ~27 MB/jump — essentially the same slope as
horizon 3, because the streaming budget (2 tiles/frame) caps how much new world
a 1.8 s window can touch regardless of horizon. So the horizon-quadratic
lifetime the field reports saw is a *steady-state footprint* + *continuous
far-field sweep* effect (a moving viewer at horizon 6 drags a 6 km sampling
annulus across far more distinct columns per second than horizon 3's 3 km one),
not a per-teleport-rate effect. The teleport storm is the cleanest repro of the
underlying leak; it is not the sharpest repro of the horizon scaling.

## The mechanism

Position-keyed retention reached through the `Authority`'s
`Arc<Mutex<WorldGenerator>>` and its `HostWorld` — but the two suspects the
draft diagnosis named turned out to have **different verdicts at integration
review**:

1. **`HostWorld.chunks`** (`dc-api/src/host.rs:178`, materialized in `chunk_at`
   at `host.rs:367–373`): every chunk any voxel query touches — player collision
   (`is_solid_voxel`), grounding, surface scans, edit targeting, raycasts, near
   streaming — is lazily generated and `insert`ed into a
   `HashMap<ChunkPos, Chunk>` that is never pruned. A full 32³ chunk retained
   forever per distinct chunk ever looked at. **This is the only genuinely
   unbounded store on the measured path**, and it owns the linear march.
   Back-of-envelope (estimate, split unmeasured): ~25 MB/jump over ~33 KB
   per chunk ≈ ~750 chunks materialized per jump — the right order for a
   fresh near-field stream plus query warm-up around a teleport destination.

2. **`WorldGenerator`'s collapse caches** (`dc-worldgen/src/collapse.rs:245–249`:
   `lattice_memo`, `region_cache`, `locale_cache`, `column_cache`) — a large
   but **bounded** footprint, not the march. The draft claimed these are
   "pure memoization with no bound and no eviction"; integration falsified
   that against the code: `evict()` exists (`collapse.rs:609–624`) with caps
   (column 8 192, locale/region 65 536 each, lattice 2 M entries retaining
   coarse levels) and fires at the end of **every `generate_chunk`**
   (`collapse.rs:417`) — and a teleport storm generates chunks constantly, so
   these caches were being clamped *during* the measured runs. Their cap sum
   is a couple hundred MB of steady-state footprint, consistent with the
   ~1 GB idle baseline. The one real gap the review confirmed: `evict()`
   fires **only** on `generate_chunk`, so a consumer that only samples
   `coarse_surface`/`column_record` (a far-field-only path) can grow the
   lattice/locale/region caches between chunk generations — a sharpening for
   the fix slice, not the storm's mechanism.

Both stores are position-keyed, so RSS goes flat the instant you stop
exploring — exactly the revisit result, and exactly why an idle session
(which touches no new positions once its field is streamed) never grows. The
experiment alone could not split the per-jump cost; code inspection assigns
the unbounded term to `HostWorld.chunks` alone. (Integration note, kept
deliberately: the draft's "both unbounded" claim died at review against code
the integrator had read the same session — corrections #19's corollary,
"the fastest falsifier is a fact you handled an hour ago," working as
designed.)

The smearing at horizon 3, then, is most likely the *terminal* symptom of RAM
pressure (allocation stalls, driver paging under a working set marching toward
exhaustion), not a distinct texture bug — VRAM itself never grew in any run.
A DeviceLost is the crash that RAM/allocator exhaustion provokes in wgpu; the
secondary defect stands (a DeviceLost should degrade loudly, not cascade into
`unwrap` panics), but the primary cause is upstream of the GPU entirely.

> blogworthy: "the leak was below the counters we trusted" — the whole value
> was in instrumenting our *own* allocations first. Every prior arrow pointed at
> the GPU and the far-field pooling; a flat VRAM trace and a flat mesh count next
> to a linearly-climbing RSS is what turned the investigation 180°. The blind
> control (VRAM) reported its null, and this time we believed it *because* a
> different register (RSS) was lit.

## The fix is a policy decision, so this entry stops at the diagnosis

The cure is eviction, and eviction on authoritative state is not a
one-liner — it is a design call with correctness edges:

- `HostWorld.chunks` is the load-bearing half and the actual leak: it is the
  authoritative world and holds **edits**. Evicting a chunk that carries a
  player edit would lose it; eviction must distinguish generated-and-untouched
  chunks (re-derivable, safe to drop) from edited ones (must persist or spill
  to the decided save layer). This is the same doctrine S11 proved for water
  — *store only what the derivation cannot predict* — applied to the chunk
  store, and it interacts with the persistence follow-on.
- `WorldGenerator`'s caches already self-evict at caps; the residual work is
  small and lower priority: close the `coarse_surface`-only gap (evict is
  currently reachable only via `generate_chunk`) and, if measurement asks for
  it, make the caps distance-aware. Pure memoization by the module's own
  contract — determinism-neutral by construction.

The fix touches `dc-api` (and residually `dc-worldgen`) — outside a diagnosis
slice's remit. Filed to Sequenced with the numbers above; the instrument
(`DC_MEM_PROBE`) is merged so the fix can be measured against the same slope.

## Repro (all via the merged `DC_MEM_PROBE` plugin)

```
# idle control (flat):
DC_MEM_PROBE=1 dc-client --horizon 6      # sit; RSS + counts flat, no crash

# leak (linear ~25 MB/jump):  teleport to fresh coords every ~1.8 s via MCP
DC_MEM_PROBE=1 dc-client --horizon 3      # then pose_set{surface:true} storm

# decisive control (flat): same storm, cycling 4 FIXED coords → RSS plateaus
```
