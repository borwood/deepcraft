# Corrections — claims made and falsified

Check here before re-deriving. Each entry: the claim, why it was wrong, the
mechanism, where the fix lives.

## 1. "Bevy `depth_bias` resolves coplanar z-fighting" (2026-07-18)

**Claim**: giving each LOD level a `StandardMaterial` with progressively
negative `depth_bias` would make finer geometry win depth ties.
**Falsified**: user still saw z-fighting; `depth_bias` only adjusts draw
*sort order* — it never changes the depth fragments write. Coplanar-face
shimmer comes from two tessellations of one plane interpolating depth with
different per-pixel float error, which no draw order fixes.
**Fix**: radial push of each far chunk away from the viewer by 0.15 of its
coarse voxel (`64729c5`, farmesh.rs module docs).

## 2. "Spawn transform doesn't matter; the positioning system runs before rendering" (2026-07-18)

**Claim** (in-code comment): chunks could spawn with `Transform::default()`
because `position_chunks` sets the real transform "before rendering".
**Falsified**: the positioning systems run *earlier in the same frame's
chain* than the streamers, so a newly spawned chunk rendered one frame at
render-space (0,0,0) — the floating origin, up to 256 m behind the player —
producing movement-correlated geometry flashes.
**Fix**: compute the origin-relative transform at spawn (`64729c5`).

## 3. Walk-3 misdiagnosis: "floating LOD shards and a horizon seam line" (2026-07-18)

**Claim** (journal/0003 findings #3): far-mesh defects — detached geometry
hanging in the sky and a hard seam line across the horizon.
**Falsified by user review of the screenshots**: the camera was **inside a
block**. The "seam line" was the outline of the face being viewed from its
reverse side; the "floating" faces were outcroppings whose toward-facing
faces were visible from within the terrain. Observer error, not renderer
error.
**Walk 4 follow-through (journal/0004) falsified the rest of the report:**
- Finding #1 "inverted haze": FALSE — distance haze is normal; the near
  white is flat shading under a near-vertical sun blowing out pale top faces
  (user diagnosis; art calibration, not a shader bug).
- Finding #2 "MCP edits invisible / receipts-vs-events seam": FALSE twice —
  `tick_authority` applies changes from ALL receipts regardless of source
  (code reading), and a pillar placed over MCP photographs correctly from
  open air (`0004-pillar-verified.png`). The walk-3 pillar was invisible
  because the photographer was buried.
**Lesson**: a walker must know whether its own camera is inside solid —
now shipped as `eye_in_solid` + `surface:true` teleport (`0ba292f`). Do not
diagnose renderer defects from a viewpoint you haven't verified is in air.

## 4. "serde `skip_serializing_if` is safe on wire types" (2026-07-18)

**Falsified during S5**: postcard is positional; omitting fields silently
corrupts the stream. Standing rule in API.md § v0 implementation notes;
regression-tested in dc-api.

## 5. "`surface_height_m` under-reports ~7 m — likely a missing detail octave" (2026-07-19)

**Claim** (journal/0004, carried into ROADMAP Observed): the analytic helper
under-reports the voxel surface by ~7 m, "likely a detail octave present in
voxel generation but missing from the heightmap helper."
**Falsified**: there is one field — `block_in_column` calls `surface_height_m`
directly, so a same-column disagreement of meters is impossible *by
construction* (in-column error is bounded to +½ voxel by the center-solidity
rule). The real mechanism is two compounding effects: (1) the solid **top
face** sits up to half a voxel above the analytic height everywhere, and
(2) a body's **footprint spans columns** and rests on the highest of them —
on the chasm walls neighbouring columns differ by many meters. Measured worst
gap 14.49 m; 61% of near-surface columns would embed a body placed at
`analytic + 0.05` (journal/0006).
**Fix**: `true_surface_m` — footprint-max over per-column voxel scans, edits
included; everything that seats a body routes through it (merge `8aafc3a`).
**Lesson**: an analytic generator field and its own voxelization are
different surfaces; never seat a body on the former. Also: the plausible
single-cause story ("missing octave") survived three walks because nobody
priced the footprint; quantify before naming mechanisms.

## 6. Chunk-line family cutover: "cell-stepped climate context flips selection on chunk borders" (2026-07-19)

**Claim** (ROADMAP Observed, walk 8): material families cut hard on chunk
lines because "at N=2 the chunk (28.8 m) equals the S7 column-quantization
cell, so cell-stepped climate context flips selection exactly on chunk
borders."
**Half falsified (mechanism refined).** The pregen climate/provenance **cell**
is `CELL_VOXELS` = 16 384 voxels (512 chunks, ~14.7 km) — *not* the chunk — and
`climate_at` is already **bilinear** between cell centres, so climate is a
smooth field that does not step at chunk borders. Climate is not the culprit.
The half that is right: the quantization cell that *does* equal the chunk at
N=2 is the **chunk-column collapse unit** (`L_COLUMN`, 32 voxels), and the
strata passes run **once per chunk-column**. So every field the passes consume
— the single centre climate sample, the single centre `flow_energy`, the
footprint-mean elevation, the per-chunk selection hash, and the integer-rounded
thicknesses — is piecewise-constant across the whole 32×32 footprint and
uncorrelated with its neighbours. Member identity (and, for the widened roster,
which member of a class) is therefore uniform per chunk and flips on the grid.
It is **selection quantization**, not "cell-stepped climate."
**Fix** (3d mechanic 2, journal/0011): a material-tier **boundary dither** —
per voxel-column the host member is re-selected from a bilinear field whose
corners are the chunk-column selection hashes (C0-continuous across borders),
so a class's member contact wanders like a facies boundary and can fall inside
a chunk. Proven by a transect test (`family_contacts_wander_off_the_chunk_grid`).
The thickness/class-presence quantization (e.g. a sandstone cap appearing via
`flow_energy` rounding) is a *separate* per-chunk artifact, left as a loose end
(ROADMAP Observed) — it needs per-voxel-column context, not member dither.
**Lesson**: "the cell equals the chunk" was true of the *wrong* cell. When a
quantization has several candidate cells (pregen cell vs collapse unit),
measure which one the artifact actually rides before naming the field that
steps.

## 7. "env!(CARGO_MANIFEST_DIR) is a fine way for tests to find the workspace" (2026-07-19)

**Claim** (implicit in dc-host test plumbing since S5): compile-time
`env!("CARGO_MANIFEST_DIR")` locates the workspace for spawning the nested
demo-plugin build.
**Falsified during S9 integration**: agent worktrees share one
`CARGO_TARGET_DIR`; the workspace-feature flavor of the parity test binary
was last compiled inside a since-deleted worktree, so the baked path pointed
at nothing (OS 267 NotADirectory at spawn). Solo reruns used a different
cached feature flavor compiled from main and passed — deterministic per
flavor, masquerading as flaky.
**Fix**: runtime `std::env::var("CARGO_MANIFEST_DIR")` with compile-time
fallback (dc-host tests/common). Standing rule: no `env!`-baked absolute
paths in anything that outlives its compilation directory.

## 8. "B is minutes, paid once" and "erosion is bounded" (2026-07-19)

**Claims** (earth-processes.md § S9 framing, Claude): full-resolution
global deep-time (B) costs "minutes, paid once"; and "bounded processes
(erosion, deposition — not drainage)" can refine regionally.
**Falsified by S9 measurement**: B at Medium on the scalar engine is
**~63–80 minutes + ~3 GiB** (minutes only at Small extent or with ~30×
parallelism — the named flip condition). And only **hillslope** erosion is
bounded (clean 21-cell decay); **fluvial** erosion inherits drainage's
global reach (isolated spikes to 26 cells as reroutes teleport along
receiver chains). C is licensed solely because drainage is decided
coarse-globally.
**Lesson**: cost claims and boundedness claims are measurements, not
adjectives; the halo theorem applies per-process, never to "erosion" as a
lump.

## 9. "A parallel priority-flood could pull B to ~2 minutes" (2026-07-19)

**Claim** (S9-results.md recommendation, carried into earth-processes.md):
the deep-time verdict's flip condition — parallelizing the flood could make
brute-force B (~27 M cells) cost ~2 minutes, where its simplicity wins.
**Falsified by S9b measurement (on this 6-core/12-thread machine)**: the
flood (65–72% of the step, 98.5% at B scale) has **no byte-identical
parallel form** — level-gather reordering breaks fp-sum determinism, and
the open-seam tiled variant diverges up to 110 m at ~0.4% of cells. The
phases that DO parallelize deterministically (route 9×, diffusion 5.7×)
are a bandwidth-bound minority: whole-step speedup 1.18–1.26×, saturating
by 8 threads. Realistic parallel B ≈ 15–70 min vs the 2-min target.
(Also: S9's recorder empty-header estimate was 648 MiB; measured 833 MiB.)
**Standing**: A+C stands. The question reopens only on ~32-core hardware
with a deterministic parallel flood (Barnes spill-graph unbuilt) AND
parallel transport — `examples/deeptime_par.rs --full-b` re-measures it
anywhere.
**Lesson**: "could be parallelized" is a claim about an algorithm's
existence, not its cost under a byte-identity mandate — the determinism
tax must be priced per phase before it prices your architecture.

## 10. Walk-12 misdiagnosis: "surface:true seated the player inside rock" (2026-07-19)

**Claim** (journal/0015 walk section, ROADMAP Observed as BLOCKING, and the
dispatched fix brief — all mine): 3e-1's deep-time elevation left
`true_surface_m`'s per-column scan ceiling stale, so `surface:true` seated a
body inside solid rock at (40, 6) while `eye_in_solid` falsely reported
`false`.
**Falsified on reproduction.** At that column `true_surface_m` returns
1004.40 and the voxel there is **air with dirt beneath** — a correct
placement. My "proof" compared a pose in **meters** (feet y = 1004.45)
against `world_get_block` queries in **voxels** (granite at y = 1050): at
N=2 the surface voxel is 1115, so "granite at 1050" is basement 65 voxels
*below* the walker, not rock around their chest. Read in the wrong unit,
an ordinary column looks like a burial.
**The stated mechanism was also wrong**: the worldgen ceiling had already
stopped being an independent estimate — it reads `ColumnRec.heights`, which
derive from the same lattice 3e-1 taught to inject the deep-time surface,
so it inherited deep time for free the day the terrain did. Reading the
generator's own answer instead of re-deriving one is what saved it.
**What was really broken** (found by the investigation, so the false alarm
still paid): `eye_in_solid` consulted the client `ChunkMap`, whose miss path
falls back to the legacy S1 `TerrainGen` (surface ≈ 8 m). Under the worldgen
authority (~1000 m) it answered from a different planet — structurally
`false` right after any teleport, exactly when a walker needs it. And a scan
that found nothing silently returned `analytic − 220 m`: **a miss and a
surface shared a type.** Fault injection (ceiling forced 200 m low) buried a
body 195 m deep with no complaint.
**Fixes** (merge `81a87b8`): `true_surface_m → Option<f64>` so misses reject
loudly (teleport reports `surface_snapped:false`, attach refuses with
`no_surface`); `eye_in_solid` reads the authority via `Authority::is_solid_m`;
headroom re-scoped as an edit allowance (8→4 m), depth 220→96 m;
`debug_assert` when a column's top solid reaches the ceiling.
**Lessons**: (1) this is corrections #3 again — *do not diagnose from an
instrument you have not verified*, and units are part of the instrument;
pose speaks meters, block queries speak voxels, and nothing in either reply
says so. Echoing voxel coordinates in pose replies is filed as a fix. (2) An
`Option` is not pedantry: when "no answer" and "an answer" share a type, the
silent path is the one that ships.

## 11. "Adjacent same-level far chunks shift by near-identical vectors, so no visible gaps open" (2026-07-20)

**Claim** (farmesh.rs module docs since the radial push shipped, `64729c5`;
re-affirmed by FF2a/journal 0023's "crack class cured inherently — same-level
seams watertight" and its "no pixel cracks" grazing-angle photo): the
per-tile anti-z-fight push (0.15 of a coarse voxel, each tile along its OWN
center-to-viewer direction) never opens visible seams between same-level
neighbours.
**Falsified**: user field report, twice — thin bright seams between far
patches, seen from altitude looking down ~-45°, discernible even in
`0023-lit-high-vantage-rings.png` (which the integrator misread as the
cosmetic one-sided-normal stitch *shading*; light through = geometry gap —
the user's eye was the instrument that worked).
**Mechanism**: the push is a rigid per-tile translation whose direction
differs between adjacent tiles by their angular separation as seen from the
viewer (≈ tile_size/distance). The shared edge therefore separates by
push × (tile_m/dist): ≈0.08 m (L1) up to ≈0.9 m (L4) — a sub-pixel-to-pixel
sliver of background light at range. The claim was *observationally* true
for the S1 far mesh only because those chunks are volumetric shells: a
sub-meter lateral offset between closed volumes exposes the neighbour's own
side geometry, never the sky — the user confirmed v0 gen never showed this.
The moment the far field became a hollow top-surface sheet (journal/0022),
the same offsets became see-through slots; the walk-17-era "pixel gaps"
report was THIS, not (only) T-junctions. FF2a cured the T-junction class in
mesh space and kept the push, so the slots survived: **watertight geometry,
reopened at the transform stage**. Float precision is innocent by three
orders of magnitude (f32 at 1.2 km ≈ 0.1 mm vs 0.08–0.9 m differential).
Why the FF2a walk missed it: grazing angles foreshorten the slots and back
them with terrain; the top-down-from-altitude view maximizes the open
cross-section against bright haze.
**Fix direction** (fix cycle dispatched same day): make the push uniform
per level — one shared translation vector per LOD level per frame (along
the camera forward axis), so same-level seams stay closed *by construction*
(identical rigid motion) while every face orientation still gains real
depth separation along the view axis. Corrections #1 still stands: it must
remain a true world-space offset, never a bias.
**Fixed** (journal/0024, 2026-07-20): both far paths route their push through
one `level_depth_push(base, level, forward)` — `forward` is `Player::view_dir()`
(the camera axis, always unit, so the old `normalize_or_zero` degeneracy is
gone), magnitude `DEPTH_PUSH_FRAC × coarse voxel` unchanged. Since the push
takes no tile coordinate, every tile of a level translates by the byte-identical
vector; along the view axis this adds exactly `magnitude` of depth to every face
orientation uniformly, and per-level magnitude keeps overlapping rings apart.
Proven past the transform in world space by
`uniform_push_keeps_same_level_seams_watertight_in_world_space` (exact-zero
shared-edge gap at a nasty off-axis high vantage; the retired radial scheme
opens a >0.05 m gap on the same check, asserted as rationale). Walk-verified
live, lit + fullbright, high vantage −45° yaw sweep: the seams are gone with no
z-fight regression.
**Lesson**: a per-entity transform computed from per-entity state is part
of the mesh's watertightness contract. "Watertight" proven in mesh space
means nothing if the transform stage is allowed to move neighbours
differently — prove seam closure PAST the transform, in world space.

## 12. "The world-creation ritual costs 25 s with biology on" (2026-07-20)

**Claimed** (docs/spikes/S10-results.md § Cost, and repeated in ROADMAP and at
the GO ratification): flipping `production_config`'s `biotic` grows the ritual
from **15.17 s to 25.19 s** (+66 %, 1.66×) at the A tier. This is the number the
user was asked to ratify, and did.

**Measured on the shipped path** (journal/0026, same box, same seed
`0x0D5E_ED57_2026`, same 297 025 cells @ 460 m, same 200 iterations, same
`DeepConfig` values): **10.82 s → 13.46 s** for the deep-time sim, and **13.79 s**
for the whole `Pregen::run` ritual with biology on. Biology's marginal cost is
**+2.6 s (1.24×)**, not +10 s (1.66×).

**Mechanism**: the S10 cost table came from `examples/biotic_spike.rs`, which
calls `deeptime::run` — and `deeptime::run` delegates to `run_with(.., false)`,
the **scalar** path. Production calls `deeptime::build_field`, which passes
`true` and takes the **byte-identical parallel** path S9b built and proved. The
spike measured the scalar worst case and the doc reported it as "the ritual".
Both halves were honest in isolation; the composition was not, because nothing in
the results doc said which driver produced the table.

Not a defect and not a regression — the shipped world is cheaper than promised,
and byte-identical either way (S9b's parity proof is what makes the two
interchangeable in the first place). It matters only because a user ratified a
cost that is roughly double what they will experience, and because the same trap
is waiting for the next spike.

**Lesson**: a measurement harness may exercise a different driver than
production. When a spike publishes a cost the ship decision rests on, name the
code path that produced it, and measure the production entry point at least once
before quoting the number as the user-visible cost. S10-results.md is left
unamended — a spike result is a dated record of what was measured; this entry is
the pointer.

## 13. Misattributed user quote: "truly i don't think any darkness" (2026-07-20)

**Claim** (docs/design/visuals.md § PBR-1 walk-14 ratifications, added
2026-07-19 in commit `a29f3f5` by an assistant session): that the user held
a "noted lean" toward **possibly no darkness at all, even underground**,
supported by a direct quotation, "truly i don't think any darkness",
glossed as "a legibility-first stance".

**Falsified by the user, 2026-07-20**: "that's a misattribution." The user's
actual position, stated the same day and now DECIDED in visuals.md, is the
opposite — **much darker underground, with shadows**, referenced to modded
Minecraft with shaders, and consistent with § Mood's original "real
darkness underground, no floaty ambient minimum".

**Mechanism — CONFIRMED by the user, 2026-07-20**: the same commit's next
bullet ratifies **fullbright mixture visibility**, the albedo-only
diagnostic register that exists so screenshots can be audited "without
lighting noise". The user was talking about **debug visuals, and visuals
for AIs controlling characters over MCP** — a statement about *the
diagnostic view* was recorded as a position on *the game's lighting
design*. Two adjacent sentences about opposite things.

The user's actual position on the embodied case, stated at the same time
and now recorded in visuals.md: an AI that is **in-game rather than
dev/debug probably does need darkness, in fairness to players** — but as
*simulated* light (Minecraft-blocklight-like, "what the voxels know"),
not as crisp dynamic shadows.

**Fix**: the bullet is struck at its source with a pointer here; the DECIDED
entry no longer cites the quote (it stood on a contradiction that never
existed).

**Lessons**:
1. **Do not put quotation marks around words the user did not verifiably
   say.** In this corpus "the user said X" is load-bearing — later sessions
   treat it as ratified position. A paraphrase dressed as a quote is
   indistinguishable from a real one once the conversation is gone, and this
   one survived a day and was cited in a live design decision before the user
   caught it.
2. **Attribute the register, not just the words.** A remark about a debug
   view, a placeholder, or a diagnostic is not a statement about the shipped
   game; record which one it was.
3. Same family as #3 and #10: confident recording of something never
   verified. Walks, units, and now quotations.
## 14. "The body graph needs rich inlet/outlet structure" (2026-07-20)

**Claim** (docs/design/water.md § the hypothesis, and the S11 brief that
followed it): free water is a graph of bodies carrying *volume, level,
inlet(s), outlet(s)*, with links between them — and the spike's risk was that
adversarial digging would explode the node and link counts.

**Half falsified (the shape was wrong, and the risk was in the other half).**
Measured across all seven S11 scenarios, **links were 0 or 1** — never more.
The mechanism: in a voxel world **air connectivity already carries the
relation**. Two bodies in the same air component are not *linked*, they are the
*same body* (they merge), so a link can only exist between components — a
waterfall lip, a gate, a pipe. A kilometre-long channel joined to a river is
not "a link on the river's body" as the notebook sketched; it is the river.
Node count likewise never grew with edits: through 1 624 edits and 508 084 dug
voxels the body count stayed at **1**, because digging creates space, not
water. The ceiling on bodies is the **component** count (217 when one body is
forced into every component), and components track the derived coarse index.

**What this relocates rather than removes**: the dense, changes-on-every-edit
structure is real, but it is the *derived* connectivity index, not the
persisted graph. That is a better place for it — derived state need not be
saved, versioned, or made consistent across a reload, and S11 proves the
derivation by byte-identical reload from 39 persisted bytes.

**Lesson**: before pricing a graph, ask what the substrate already encodes.
Voxel geometry is itself a connectivity structure; a graph laid over it should
carry only what the geometry cannot say — here, that water *is* present and how
much. Fix/measurements: docs/spikes/S11-results.md § Q2, journal/0028.

## 15. "A closed domain with recharge and a drain measures a water-table halo" (2026-07-20)

**Claim** (implicit in S11's first Q1 harness, mine): perturb a saturation
field with a drain, relax, difference against the unperturbed field, read the
halo by Chebyshev ring — the method S9 used for erosion's decay length.

**Falsified by its own output, which looked like a *result*.** Every
permeability contrast at every drainage spacing reported the same halo — the
radius of the domain — with a perfectly flat profile (`d0=5.16 d16=3.10
d32=3.05`). Read at face value that falsifies the whole "bound water is local"
hypothesis. It was an artifact: a **closed box with recharge and one drain has
no local equilibrium**. Water accumulates until the domain saturates, and the
single drain is then the only sink in the world, so it necessarily influences
every column. The flat profile was the shape of a box with nowhere for water to
go, not a property of groundwater.

**Mechanism of the fix**: a real water table is not bounded by the edge of the
world, it is **pinned by the drainage network** — streams, springs, coastlines
every few hundred metres. With a drainage lattice plus a leaky aquitard over a
regional sink, the field has an actual equilibrium and the halo is 4–11 cells
with clean geometric decay.

Three further errors compounded it, each independently sufficient to produce a
confident wrong number: **the integrator was oscillating, not relaxing**
(`k·lateral_c` must stay under `φ/(2·ndim)`; the first version was ~30× over,
so the "noise floor" was the scheme ringing); **control and treated were
differenced at different step counts** (reference at `steady`, perturbed at
`steady + budget`, so part of every halo was just elapsed time); and the deep
sink's per-step acceptance was set **exactly equal to the recharge**, balancing
the column on a critical point.

**Lesson**: a relaxation measurement is only meaningful against the boundary
condition the real system has — get that wrong and the method still produces a
clean, reproducible, entirely fictional decay curve. And when a measurement
agrees with the hypothesis you were dispatched to *break*, that is the moment
to audit the harness, not to write it up. Corollary for the ring-difference
method generally (S9, orogeny, S11): always report the residual drift of the
unperturbed control alongside the halo, so a reader can see whether the number
sits above the noise floor. S11-results.md § Q1, journal/0028.

## 16. "Tempering abrasion resistance by cohesion sharpens the rock contrast" (2026-07-20)

**Claim** (erodibility coupling, first cut, mine): a rock's resistance to
mechanical abrasion should be its smash resistance *tempered by cohesion* —
`smash × (0.5 + 0.5·cohesion)` — on the reasoning that a hard but poorly-cemented
rock sheds clasts and erodes faster than its hardness alone suggests.

**Falsified by the probe.** With the tempering in, coupling on vs off changed the
world by **relief +0.0 %, slope_sd +0.0 %** — a null result whose control drift
was exactly zero, so it was not noise. Part of the cause (the other part is #17):
in our property sheet **mudstone is more cohesive than sandstone** (0.95 vs 0.85)
because clay is sticky, not strong. So multiplying by cohesion pulled the two
clastics' resistances *together* (from ~10 % apart toward ~4 % apart) — it
actively cancelled the differentiation the coupling exists to create. Cohesion is
the correct modifier for the *wave* agent (sea cliffs fail along joints, a
cementation question) and the wrong one for fluvial abrasion.

**Fix**: abrasion resistance reads the smash extraction resistance *straight*
(`deeptime::lithology::resistance_of_material`); a field-doc there records why
nothing may temper it. Cohesion keys the wave axis only.

**Lesson**: same family as corrections #6 — reaching for a plausible modifier
without measuring which *direction* it moves the quantity you care about. A
modifier that feels physical can be anti-correlated with the contrast you want;
measure the contrast before and after adding it, on the actual property values,
not the idealized ones in your head.

## 17. "Coupling bedrock incision is how you get differential erosion" (2026-07-20)

**Claim** (the erodibility brief's literal wording — "modulate incision" — and my
first implementation): making bedrock incision (and cover entrainment)
lithology-dependent is what produces hard-bed-stands-proud landforms.

**Falsified by measurement.** Coupling the two fluvial terms alone left the world
statistically unchanged (see #16). The world responded only once the
**bedrock→regolith weathering** phase was coupled too.

**Mechanism**: on a hillslope — the majority of land — the rate-limiting step is
neither incision nor entrainment. Hillslope diffusion is *flux-limited by the
regolith actually available*: on any real slope it exports everything there is,
so the landscape's lowering rate collapses to the rate at which bedrock is
*converted into regolith*, which is the weathering rate. Incision matters in the
channel network; entrainment matters where there is cover to entrain; but the
pace of an eroding upland is set by weathering. Couple the terms that don't set
the pace and nothing moves.

This is not a workaround — it is the correct long-run home for the coupling.
In-place weathering is the *sum of every agent's attack on rock that has not yet
moved*; today that sum has one term (mechanical), and when the dissolution agent
lands it becomes a sum over agents, so a limestone weathers fast chemically while
resisting mechanically. Karst arrives by adding a term to the phase this fix
already couples.

**Fix**: `erosion.rs::weather_cell` takes the lithic multiplier (composed with
the S10 biotic `wmult`); fluvial terms are coupled too, but weathering is what
carries the hillslope signal. journal/0029.

**Lesson**: "modulate X" in a brief names the *intent*, not necessarily the
*rate-limiting term*. Before coupling a resistance to a process, find which phase
actually sets that process's pace under the model's own flux limits — it may not
be the one the phenomenon is named after.

## 18. "The lit before/after difference is the sun moving, and the fullbright pair is the honest geometry comparison" (2026-07-20)

**Claim** (mine, journal/0030 as first written and committed): re-shooting four
vantages before and after the erodibility flip, the `--fullbright` pairs differed
by 0.08–0.93 % of pixels while the *lit* pairs differed by up to 48.9 %. I
attributed the lit difference to the sun having moved between passes and declared
the fullbright pairs the trustworthy geometry comparison — concluding the terrain
was visually unchanged.

**Falsified by the user**: *"the sun does not move whatsoever today."* There is no
day/night cycle. The lit difference cannot be a lighting change.

**Measured, after**: the lit difference survives 16×16 block averaging (mean |Δ|
per block 21.2 flank, 32.5 upland, 11.7 lowland) so it is not per-voxel texture
noise, and its *signed* mean is ≈ 0 (−0.3, −0.9, +0.1 of 255) so it is not a
brightness shift either — broad regions brightened and others darkened, in
balance. That is the signature of **face orientation changing**: surfaces that
were top faces became side faces and vice versa. The terrain really did change.

**The mechanism, and the real error**: in `--fullbright` every face of a block is
the same flat colour, because the whole point is unlit pure vertex colour. So for
terrain built from *one material*, fullbright cannot distinguish a top face from a
side face — and therefore **cannot see geometry at all**. The proof is in the
committed assets: `0030-flank-before-fullbright.png` is a *featureless grey
field*, an entire terraced hillside rendered as one uniform mass, while the lit
frame of the identical geometry shows every step. The 0.27 % fullbright diff never
meant "the shape didn't change"; it meant "this control is blind to shape."

I inverted the reliability of my two controls and then reported the blind one.

**Fix**: journal/0030 rewritten — the finding is now that ground-level appearance
*did* change substantially (real, lit, broad) while macro landform did not
(silhouette unchanged, 5 km lattice ±2 m). No claim about the sun survives.

**Lesson**: `--fullbright` is the correct control for **material and data**
questions — it is exactly what proved the coal was fine in 0027 when the renderer
had crushed it to black. It is the *wrong* control for **geometry** questions,
because the directional shading it removes is the only cue that distinguishes one
face of a voxel from another. Pick the control by what it is sensitive to, not by
what it is sensitive to *elsewhere*; a control that is blind to your question will
happily report "no change" forever. (This is the direct argument for giving
fullbright dark face borders — user, 2026-07-20 — which would restore geometric
legibility to the pass that currently destroys it; filed in ROADMAP.)

## 19. The integrator wrote an unverified agent mechanism into CLAUDE.md as doctrine (2026-07-20)

**Claim** (mine, integrating journal/0030): that lit before/after
screenshots are invalid for appearance comparison "because the sun moves
between launches" — filed within minutes to ROADMAP Observed as the
"highest-value instrument fix outstanding", and written into **CLAUDE.md's
walk protocol**, the read-first doc every future session and every agent
loads.

**Falsified the same day** (agent re-measurement, corrections #18): there
is no day/night cycle. The sun is **fixed**. The lit difference was real
terrain change — it survives 16×16 block averaging (so it is not texture
noise) and its *signed* mean is ≈ 0 (so it is not a brightness shift):
bright-here-dark-there in balance is **face orientation changing**, which
is geometry.

**Mechanism of my error, and the part that stings**: I had the falsifying
fact **in my own context, from this same session**. Writing
`docs/design/light.md` hours earlier I quoted S4-results verbatim —
*"app-set constants (fixed 0.35 time-of-day matching the fixed sun)"* —
and the whole sim-light design rests on there being **no** day/night cycle
(that is why heavenly-body paths had to be invented). I then accepted
"the sun moved" without connecting it to either fact, because it arrived
as a confident agent finding attached to a dramatic number (48.9 %).

**Fixes**: the CLAUDE.md rule is replaced with the true one — *pick the
control that can SEE your question*: lit for shape/relief (face
orientation carries shape), fullbright for material/data (flat albedo, no
lighting noise). The ROADMAP entry is retracted in place and replaced by
the real defect, #18's fullbright blindness.

**Lessons**:
1. **An agent's MECHANISM claim is not a measurement.** Its numbers are
   usually trustworthy; its causal story is a hypothesis. The integrator's
   job is to test the story against the corpus before it becomes doctrine
   — this is the same class as #3 (walk misdiagnoses), #10 (units), and
   #13 (a fabricated quote): confident recording of something never
   verified.
2. **Doctrine has a higher bar than an Observed line.** CLAUDE.md and the
   workflow skill are loaded by every future session and every agent; a
   wrong rule there propagates silently and forever. Speed is not a virtue
   when writing to the read-first documents.
3. **Check new claims against what you wrote today.** The corpus sweep
   guard (added this morning) points outward at the repo; it needs a
   corollary pointing at your own recent work — the fastest falsifier of a
   fresh claim is often a fact you handled an hour ago.

## 20. "Rivers share the one-shot disease" — half-wrong; the stale authority is elsewhere (2026-07-20)

**Claim** (assistant, written into earth-processes § 1's DECIDED scope item 5
the same day): deep-time drainage is computed once against final topography,
so water gaps / terraces / captures are impossible for the same reason
one-shot uplift forecloses superimposed orogenies.

**Falsified by** the tectonics design agent reading the source
(docs/design/tectonics.md § 14), verified by the integrator: deep-time
erosion re-derives drainage (priority-flood + steepest descent) **every
iteration** — inside the sim, rivers already migrate. The genuinely stale
authority is downstream: the rivers *carved into the world* at collapse come
from **pregen** `Cell.river` / `flow_to` / `discharge` (collapse.rs
surface_sample river segs), computed on the PRE-erosion coarse surface — a
second, older hydrology painted over terrain the deep sim has since
reshaped. The remedy in the DECIDED entry stands (export deep-time drainage,
retire the pregen chords); the *mechanism* stated for it was wrong.

**Lesson**: same class as #19 — a confident causal story recorded into a
design doc without a source check, caught only because the design pass was
briefed to contradict its own scope where the code disagreed. Briefing
agents to say "the ratified scope is wrong where it is wrong" is cheap and
pays.

## 21. The impossible red gate: two wrong mechanisms before the real one (2026-07-20)

**Claim 1** (assistant, on the first post-console-v2 red gate): transient
cross-worktree contamination — "heals once siblings stop building." Written
into the workflow skill. **Falsified within the hour** by an identical
second failure: the poisoning is persistent.

**Claim 2** (assistant, same episode): concurrent workspace test runs
colliding on port 7777. Never fit the evidence (the errors were COMPILE
errors) and was dropped when the full capture showed E0432/E0560.

**The real mechanism, confirmed by remedy**: sibling agent worktrees at
other commits build the same packages into the shared `CARGO_TARGET_DIR`;
under workspace feature-unification the dc-worldgen lib unit differs from
the `-p` unit, and the console-v2 worktree (branched pre-eolian) left a
stale artifact for that unit with a falsely-fresh fingerprint. Result:
`cargo test --workspace` compiled main's own `full_agents` test against a
lib without the fields, while `-p dc-worldgen --test full_agents` passed
from the same tree — THE discriminating observation. `cargo clean -p
dc-worldgen -p dc-client --release` (1.6 GiB evicted) + re-run → all green.

**Lesson**: when the compiler reports missing symbols that are visibly
present in source, the code is the LAST suspect — verify source, split
targeted-vs-workspace, evict. And a mechanism written into a load-bearing
doc (the skill) within minutes of forming is exactly how #19 happened;
this one was caught because the second failure arrived before the session
ended. The skill entry now records the confirmed mechanism.

## 22. My circulation gameplay-impact trace was surface-blind (2026-07-20)

**Claim** (assistant, written into earth-processes § 4's ratified zonal-
circulation entry as the required gameplay-impact trace): "deserts land
where a player who knows Earth expects them." Recorded as a *prediction* of
what the player would SEE.

**Falsified by the record-walk** (journal/0038, verified by integrator eye on
`0038-circulation-desert.png`): the surface does NOT show the Hadley desert
belt. The circulation code is correct — the ~30° subsidence band IS arid in
the data (precip ~0.2–0.3, arid by the 0.32 *biome* threshold that drives
erodibility/biotic tags). But `collapse.rs` only bares the surface to Dirt
below precip **0.10**, so the belt sits above the bare-surface threshold and
renders as ordinary grass. The visible surface pattern is
elevation/temperature (grass mid-latitudes, bare stone at coasts/peaks/
poles); 30° is the *greenest* band, not a desert.

**Not a code bug — a threshold reconciliation gap**, and it is
**user-owned appearance** (how bare a 30° desert should read). Filed to
Observed.

**The lesson is the good kind**: this is the 2026-07-20 gameplay-impact-
trace rule *working*. We wrote the predicted player-visible outcome, the
walk checked it, and the prediction was false at the surface — caught before
it became folklore. A mechanism can be fidelity-correct and gameplay-
invisible at once; the trace is what surfaces the gap. Write traces as
falsifiable predictions and walk them.

## 23. "The model is not the bottleneck, the amplitude is" (2026-07-21)

**Claim** (journal/0029 § headroom, carried into journal/0030's appraisal and
then into ROADMAP as the standing sequence item *"the amplitude call — the
last live cause of dismal mountains"*): the erosion/tectonics model is
sound and the reason mountains read as dismal is that the forcing amplitude
is too low. The pending user decision U7 was posed as a straight choice
between `thickening_scale` **80 and 160**, on the expectation that the
larger value would make landforms legible.

**Falsified by the first flagged walk** (journal/0040, the deep-config
plumbing's first use). Same seed, `--tectonics` on in both worlds, only
`--amplitude` changed. The knob **works** — it moves absolute elevation by
+714 to +1079 m across the continent, and the abyssal plain correctly does
not move (−2492.1 → −2490.3 m, since orogenic thickening does not drive
ocean floor). But it does not buy relief where relief is read:

| scale | amp 80 | amp 160 |
|---|---|---|
| 250 m steps across the highest crest (1.75 km span) | **7.2 m** | **7.2 m** |
| 10 km transect (z=0, x=0→10 km) | 38.6 m | **22.4 m** (flatter) |
| ~25 km across the whole belt | 380 m | 626 m |
| absolute elevation | — | +714…+1079 m |

The walking-scale number is not merely similar, it is **identical** — and
the two ground screenshots from the same vantage at the same height above
the surface (`0040-tectonics-crest-lit-south.png` vs
`…-amp160.png`) are visually indistinguishable: a level green plain with
scattered one-voxel ledges, photographed at the summit of the world's
highest range (1288 m, then 2249 m).

**Mechanism — HYPOTHESIS, not yet measured.** `DeepField::surface_at_voxel`
is a *bilinear* sample of a **460 m** deep grid, so every wavelength finer
than that comes from the collapse elevation lattice's own jitter, which is
seeded from `provenance_roughness` and never sees `thickening_scale`. That
would explain an exactly-identical sub-cell number. What it does **not**
explain, and what is the real lead: `provenance_roughness` is 90 m (Craton)
to 420 m (Orogeny), yet measured walking-scale relief is **7 m** — so the
lattice's per-refinement amplitude decay (`collapse.rs`) is attenuating
roughness by one to two orders of magnitude. **Needs measurement before it
is believed** (this entry's own rule: a mechanism is a hypothesis, only the
numbers are evidence — and that applies to the integrator too, #19).

**The lesson.** `thickening_scale` is a *lift-the-continent* knob, not a
*make-mountains* knob: it bites at ~25 km and above and vanishes below it.
Posing U7 as "80 or 160" framed an appearance decision the knob cannot
deliver — neither value fixes dismal mountains, so the honest answer to the
amplitude call is **"neither, and here is why."** The legibility problem
lives at sub-km scale — in the collapse lattice's roughness decay and in
erosion supply (already Sequenced from S12's metre-scale exhumation
finding) — not in deep-time forcing amplitude. Re-sequenced accordingly.

## 24. "The 460 m bilinear sample is low-passing deep-time detail away" (2026-07-21)

**Claim** (journal/0040 § Why, carried into corrections #23 as half the
mechanism, and into the S13 dispatch): `DeepField::surface_at_voxel` is a
bilinear sample of a 460 m grid, read by the collapse lattice at 460.8 m
spacing, so the near-Nyquist resample must be filtering deep-time relief out of
the walked surface.

**Falsified by measurement** (S13, `docs/spikes/S13-results.md` § 5; probe
`crates/dc-worldgen/examples/roughness_probe.rs`). Over a 20.7 km box at four
sites, the level-5 lattice reproduces the raw deep grid's relief to within
**0.4 %** and its mean cell-to-cell step to within **0.3 %**:

| site | raw grid relief / mean step | level-5 lattice |
|---|---|---|
| crest | 258.79 m / 4.301 m | 258.38 m / 4.312 m |
| steepest | 1579.74 m / 18.051 m | 1581.34 m / 18.073 m |
| walk-0040 | 185.07 m / 3.852 m | 184.43 m / 3.859 m |

**Mechanism**: bilinear interpolation is *exact at cell centres*, and the two
spacings differ by 0.17 %, so the lattice sees essentially every value the grid
holds. Nothing is smoothed away below 460 m because the source holds nothing
below 460 m.

**What was true instead.** #23's other half — that the lattice's
per-refinement decay attenuates roughness by 1–2 orders — is **confirmed at
exactly `AMP_DECAY^L_DEEP = 0.55⁵ = 1/19.8`**: the jitter at levels 1–4 (66.9,
39.5, 22.7, 13.9 m RMS, at 7.4 km → 921 m wavelengths) is computed and then
*discarded* when level 5 (`L_DEEP`) replaces elevation with the deep-time
surface, so **5.0 % of the scheduled roughness budget reaches the ground**. And
a third mechanism neither hypothesis named: at the summit the simulated deep
surface is itself a plateau, adjacent 460 m cells differing by **0.29 m
(0.065 % grade)** across a 10 km box — so fixing the decay fixes the
100–500 m band but cannot make that plateau a range.

**Also rejected by number, before anyone builds it**: deriving jitter amplitude
from the deep field's own local gradient (self-scaling — plains smooth, flanks
rough). The measured summit gradient is 0.29 m per 460 m, so that rule drives
summit roughness to ~zero and makes journal/0040's photograph strictly worse.

## 25. I walked the flattest place in the world and called the world flat (2026-07-21)

**Claim** (journal/0040, its ROADMAP Shipped entry, and how I narrated the
walk to the user): the world has no landform scale — *"the whole belt ~380 m
over 40 km, ≈1 % grade"*, *"a level green prairie"*, and the world reads as
*"miles and miles of gentle slope"* with no mountain anywhere.

**Qualified — and in its general form falsified — by S13**
(`docs/spikes/S13-results.md` § 3, same seed, same build). Relief as a function
of sampling window, at the site I walked versus the field's median and steepest
sites:

| window | walk-0040 site | median site | steepest site |
|---:|---:|---:|---:|
| 1 km | 19.7 m | **55.8 m** | **80.3 m** |
| 5 km | 29.3 m | 258.7 m | 390.5 m |
| 10 km | 54.7 m | 515.1 m | **791.3 m** |
| 25 km | 295.8 m | 1264.2 m | **1918.5 m** |

A typical place in this world falls ~56 m per kilometre. The steepest ground
falls ~790 m over 10 km. **That is a landscape.** The world is not uniformly
flat; I sampled the two flattest kinds of place in it and generalized from
them.

**Mechanism — the error is in the sampling, not the arithmetic.** Every number
journal/0040 reports is correct and reproduced by S13 to the decimetre
(the probe matched all eight heights at both amplitudes, offset a constant
1.0 m because the client pose sits one voxel above the surface it reports).
The defect is *where* I stood. I ran a deliberate hierarchical search for the
**summit** and then measured relief there — but a summit is by construction the
place where the gradient goes to zero. Then my other transect ran along z = 0,
which is also low-gradient ground. Having chosen the two flattest sites
available, I concluded about the world. S13 measured the summit's deep cells
differing by **0.29 m across a 10 km box (0.065 % grade)** — the plateau is a
genuine simulated feature, faithfully rendered, and I read it as a global
verdict.

**What survives.** Corrections #23 is untouched: the amplitude A/B was a
*same-site* before/after, and the jitter is byte-identical at 80 and 160, so
"amplitude buys no sub-km relief" stands. What does not survive is the
characterization of the *world*.

**The lesson — #18/#19 have a sampling twin.** Those two are about choosing an
instrument that can see the question. This is about choosing a *sample* that
can: when the question is "does this world have relief," the extremum of the
field is the blind place to ask, because extrema are where relief vanishes by
definition. **Sample the distribution, not the extremum** — and when a walk
reports a global property from one vantage, that is a claim about a
distribution made from n=1. The user's own flight corroborated my read, which
is how a sampling error survives two observers: we were both standing in the
same wrong place.

## 26. "The sim holds `H ≈ 0` in the deflation basin, so carrying `H` bares it out" (2026-07-21)

**The claim** (journal/0049 station 1, and then repeated as the premise of the
carry-`H` slice): the user standing in the world's most wind-stripped country
could not judge the erosion because the collapse tier painted topsoil over it —
"always going to have topsoil" — and *the sim holds `H = 0` there*, so carrying
the regolith plane would make deflation basins genuinely bare.

**What the measurement says.** Station 1's deep cell holds **10.66 m of `H`**.
The 13.06 m in the station's name is `ΔH` — cover the wind *removed* (the
off→wind isolation) — not cover remaining. `tour_map`'s deflation station is a
max-`ΔH` search, and the cell that loses the most is by construction the cell
that had the most to lose: a thick, loose, arid basin fill. It is a
*deflating* basin, not a *deflated* one. Carrying `H` therefore makes station 1
**deeper**, not barer — generated loose cover 6 → 12 voxels (5.4 m → 10.8 m).

**Where bareness actually lives.** Only 91 of 44 265 subaerial deep cells
(0.2 %) round to zero loose cover on this world; the barest is world
(101 663, 5 073) with `H = 0.168 m`, and it now generates basalt at the surface
with no soil at all. Bareness is real, it is caused by the erosion history, and
it is rare — the honest answer, not the promised one.

**The lesson — an extremum of a *rate* is not an extremum of a *state*.**
Sibling to #25: there we sampled the flattest place and concluded about relief;
here we named a site by its erosion *rate* and read the name as a description
of its erosion *state*. When a station is chosen by "where did the most
happen", do not assume it is also "where the least remains".

## 27. "A green gate means the code you changed passed" — the impossible GREEN (2026-07-21)

**The claim** (implicit in every gate run this project has ever done, and
explicit in corrections #21, which taught us to distrust an impossible *red*):
`cargo test --workspace --release` exiting 0 with every suite reporting
`test result: ok` means the code in the working tree was built and tested.

**Falsified — reported by the journal/0054 agent, with direct evidence.** Its
first full workspace run reported exit 0 and every suite ok while running
**none of the three tests it had just written**. Cargo reused a
`dc_client-*.exe` timestamped minutes *before* `devicelost.rs` existed, out of
the `CARGO_TARGET_DIR` that agent worktrees deliberately share (CLAUDE.md
§ Build rules). `cargo clean -p dc-client --release` resolved it, and the
crate's test count moved **99 → 102** — the three new tests appearing for the
first time.

**Why it is worse than the impossible red.** #21's failure mode *announces
itself*: the build goes red with missing symbols and you are forced to
investigate. This one is silent and flatters you. A grep for `test result: ok`
— the exact filter this project's gate discipline recommends — cannot see it,
because every suite genuinely did pass; the suite that mattered simply was not
the one on disk.

**Epistemic status, stated deliberately.** The stale-artifact mechanism is the
*agent's* account, supported by the timestamp and the 99 → 102 count. **The
integrator did not reproduce the stale serve** and cannot distinguish it from
an ordinary sequencing slip (gates run before the file was saved). What the
integrator *did* verify is that the remedy works and is cheap: the session's
final gate ran after `cargo clean -p dc-client -p dc-worldgen -p dc-api
-p dc-core --release`, and every new test of this session was confirmed
**present by name** in the output — 48 suites, 489 tests, exit 0. That run is
the authoritative one for journal/0050–0054.

**The practice that replaces the assumption**, now in CLAUDE.md § Gates:
before a merge gate, clean the crates you changed; then verify the gate by
**test name or count**, never by `test result: ok` alone. A gate is only
evidence about the code it actually ran, and "did it run?" is a separate
question from "did it pass?" — one this project had been conflating.

## 28. "The shipped world does not express journal/0055's numbers" (2026-07-21)

**The claim** (the integrator's, from a live dig): journal/0055 reports the
loess margin expressing **89 voxels of section, 188 spans, 76 mixed** over
`H` = 80.49 m. The user walked the running client — seed 1337, `Extent::Medium`,
stock boot — dug at the coordinates that entry names, and found **~3 voxels of
mudstone over ~4 of basalt over granite**, at two columns 180 m apart. Eighty-
nine against three is not rounding. The suspicion raised, and it was the right
one to raise, was that **the world a player walks is not the world the probes
measure** — that two entries had been ratified on numbers from a place nobody
plays.

**Falsified, headlessly, against current main.** `Pregen::run` is *defined* as
`run_with(params, &DeepOverrides::default())`, and the client's
`Authority::new_worldgen` calls exactly that with `seed: 1337_i32 as u64` and
`extent: Extent::Medium`. The measurement is better than the code read: the
walker reported surface **y153**, **y148** and (at the periglacial summit)
**y1095** at the voxels he stood on; the headless probe, with no client in the
loop, independently answers **surface y 153**, **y 148** and **y 1095** at those
same three voxels. Three columns kilometres apart, three exact agreements. The
world a player walks is the world we measure.

**Blocks against blocks, which is what finally settled it.** The two sides were
arguing in different currencies — the probe's metres and contents against
`world_scan_region`'s blocks — so the probe learned to speak blocks
(`block_column`: generate the chunks, run-length encode the column downward). At
the four addresses actually scanned, the generator says y153 `Mudstone×3 /
Basalt×4 / Granite…`, y1095 `Mudstone×12 / Basalt×3 / Granite…`, y98
`Mudstone×4 / Basalt×4 / Granite…`, y256 `CarbonaceousMudstone×1 / Basalt×3 /
Granite…` — **block for block, run for run, y for y, matching every live scan.**

**The "systematic under-expression" is also falsified, and its residue is a
known, documented +1.** The sharpest form of the worry was that every site
under-expresses against `round(H / 0.9)`, worst where the record is thickest.
Measured at both readings of all four labels, the error is **never negative**:
0 or **+1**, everywhere — 89→90 at the loess margin's true address, 3→3, 17→18,
11→12, 10→11, 3→4, 0→0, 1→1. The +1 is journal/0055's top-of-column remainder:
`h = floor(elev/0.9)` guarantees ground in the surface voxel, so it is filled
from its floor up to the real ground with **at least one eighth** — and a partial
voxel is still a solid block, so a block scan counts it whole. The generator
never under-expresses; **the block tier over-reads by exactly the one partial
voxel it cannot represent.** The apparent shortfall was `H` read at the metres
address compared against blocks counted at the voxel address — every site
disagreed because every site was two places.

**The second suspect — the probe's `idx_to_voxel` mapping — is also clean.**
Over 37 597 deep cells, `regolith_at_voxel` at each cell's own computed voxel
address reads back that cell's own `H`: **0 disagreements, worst
|ΔH| = 0.000000 m**. journal/0053's *scanned* coordinates (sites 6 and 7) point
where they say they do.

**The real mechanism: a coordinate written without its unit.**
`soil_depth_probe::STATIONS` holds its stations in **world metres**
(`82346.0, 24391.0`) and divides by 0.9 to reach a voxel. journal/0055 wrote the
station down as `(82 346, 24 391)` — no unit — in an entry whose every other
coordinate-shaped number is a voxel. Read as a voxel address it is world
(74 111 m, 21 952 m): **8.6 km away, nineteen deep cells over**. The loess
margin's voxel address is **(91 496, 27 101)**, and there the generator answers
`H` = 80.492 m, 354 recorded units, 170 events, **188 voxel spans, 76 mixed** —
journal/0055 to the digit. At the address that was actually dug, `H` = 2.281 m
= 2.53 voxels, which *is* three voxels of mudstone on basement. **Both
observations were correct measurements of two different places.**

**Why it looked site-specific, and why that is the fingerprint rather than an
alibi.** A second live sample — the periglacial summit — *agreed* with
journal/0053, which read as evidence against a world divergence. It is stronger
than that. Misreading a metres label as a voxel address displaces you by exactly
`1/0.9 − 1 = 11.1 %` **of the coordinate's own magnitude**: 8.6 km at the loess
margin (82 km out, 19 deep cells, a different geological story) but only 0.6 km
at the summit (4.6 km out, barely one deep cell, terrain that looks the same and
carries a similar pile — `H` = 15.36 m true against 9.70 m misread). A genuine
world difference has no reason to scale with `|x|`. A units error can do nothing
else. **"It only disagrees far from the origin" is a units diagnosis, not a
site-specific one.**

**The practice.** This codebase carries three coordinate systems — metres,
voxels, chunk columns — and they differ by a factor of 0.9 and 28.8. A bare
number pair is not an address. Journal entries and probe output must carry the
unit on every coordinate (`soil_depth_probe` already prints
`world (X m, Y m) | voxel (vx, vz) | chunk (cx, cz)`; entries quoting it must
not drop the qualifier). `examples/surface_dither_probe.rs` prints both readings
of the disputed label side by side so this one cannot be re-derived from memory.
Full account: journal/0058.
