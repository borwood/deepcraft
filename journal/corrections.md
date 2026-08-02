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

> **⚠ THE LAST SENTENCE'S POLICY WAS HALF-SUPERSEDED, 2026-07-28 (user).** *"A spike
> result is a dated record"* is **upheld** and is now doctrine. *"This entry is the
> pointer"* is **not enough** — and this entry is the case that proved it.
>
> For eight days this correction knew about `S10-results.md` and **`S10-results.md` did
> not know about this correction**, while `CLAUDE.md` read-first item 5 sent every cold
> session to the spikes saying *"measured numbers, don't re-guess them."* The two rules
> were a live, unreconciled **policy collision** in two different files; the user ruled
> on it 2026-07-28.
>
> **The policy is now: IMMUTABLE BODY, MUTABLE HEADER.** The measurements are never
> rewritten; the results doc **must carry a top-of-file banner pointing at whatever
> refuted it**, and **writing that banner is the correction author's obligation, in the
> same commit.** `S10-results.md` now carries one. *A one-directional pointer is not a
> pointer — it is a note to whoever already found the answer, and the stale end is
> exactly where a cold reader enters.* Full rule: CLAUDE.md read-first item 5.
>
> *This paragraph is an addition, not an edit: the entry above stands as written on
> 2026-07-20, including the sentence now superseded.*

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

> **⚠ THIS ENTRY'S *MECHANISM* HALF WAS FALSIFIED BY #24; its CONCLUSION stands.** Reciprocal
> pointer added 2026-07-29 (baseline sweep S7/F9) — **#24 named this entry and this entry
> named nothing back.** The *"460 m bilinear sample low-passes detail"* hypothesis — which
> this entry correctly flagged **"HYPOTHESIS, not yet measured … needs measurement before it
> is believed"** — was measured by S13: the level-5 lattice reproduces the raw deep grid's
> relief to within **0.4 %**; *nothing is smoothed away below 460 m because the source holds
> nothing below 460 m.* The real cause is named in #24 (`AMP_DECAY^L_DEEP`).
> **The conclusion — `thickening_scale` is a lift-the-continent knob — is untouched and live.**
> *Listed low in the census because the entry did the right thing: it flagged its own
> hypothesis, and the falsification arrived.*

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

## 29. "Partial-height will light up for free when deposition lands" (2026-07-21)

**Claimed** in journal/0010, and restated as a standing note in `meshing.rs`'s
module docs: sub-8 loose voxels cannot occur yet, so partial-height rendering is
a tested-but-dormant capability that "will light up for free the day deposition
produces its first sub-full column."

**Falsified by a walk**, hours after journal/0055 made the deep-time record
decide the world's skin and put a sub-8 loose partial on top of nearly every
column. It did not light up for free; it lit up with holes straight through the
ground (`assets/0056-holes-after-settle.png`, persistent across a 20 s settle).

**Mechanism** (journal/0057): partial-height *geometry* was correct, but face
**culling** was block-tier boolean — `neighbor.is_solid()`. A 5/8 partial beside
a 3/8 partial had its whole side face culled because the neighbour was "solid",
so the exposed 2/8 band was emitted by nobody. Every 0010 test placed its
synthetic partial in **open air**, where boolean and occupancy-aware culling
agree; the arrangement the world would inevitably produce — partial beside
partial — was the one arrangement untested.

**The general shape**, worth more than the instance: the dormant capability's
precondition was recorded as *prose in a doc comment directly above the code* —
which is the right place — and still failed, because prose cannot fail a build.
When a feature is parked on "this will work when X arrives", the dependency
needs a test that goes red when X arrives, not a sentence that goes stale.

## 30. "The narrow flooded shaft is the coarse capacity mechanism's worst case" (2026-07-22)

**Claimed** in `docs/design/water.md` § SPIKE SPEC S15 and repeated in the S15
dispatch brief: level error behaves as `ΔV / surface area`, so the coarse
capacity estimate is "most accurate exactly where an exact scan is most
expensive (big lakes) and worst where exact is cheap (flooded shafts)."

**Half right, and the wrong half is the memorable one.** The area law holds —
S15 measured mean |err| 0.0521 m over the small-area half of 791 real bodies
against 0.0135 m over the large-area half. But the *named* worst case is the
mechanism's **best** case. A shaft is **dug**, and a dug void enters the coarse
summary through the audited write path as an **exact signed integer delta by y**
— not as a sub-sample of terrain. Measured: a 1×1×64 shaft filled with 32 voxels
of water gives coarse level `981.000000` against exact `981.000000`, **error
0.000000 m**.

**The real worst case is a small NATURAL depression** — sub-cell relief that
16 samples per 1 024 columns cannot resolve, with no edits to correct it. The
three rows in 791 that failed the half-voxel rule are a 3-column puddle (2 m²,
err 0.858 m) and a 28-column puddle (23 m², err 0.506 m). Nothing dug failed at
all.

**The general shape:** "smallest area ⇒ largest error" silently assumes every
container is *sampled*. The moment part of a container is *recorded exactly*
(because a player made it, and the write path audited it), the small end of the
area axis splits into two populations with opposite behaviour. Ask which term of
the error a case actually exercises before naming it the worst case. Full
account: `docs/spikes/S15-results.md` § group 1, journal/0062.

## 31. "Connectivity only changes where someone edits, and edits only happen where chunks are loaded" (2026-07-22)

**Claimed** in `docs/design/water.md` § SPIKE SPEC S15 measurement group 4 and
restated in the S15 brief as the hypothesis to hunt a falsifier for: *to breach
a lake you must dig, and to dig you must be there.* The unstated inference is
what the claim was being used for — that the connectivity consequence of an edit
is therefore inside the loaded set.

**The first clause is true. The inference is false, and S15 measured the gap.**

- **True, and for a strong reason:** terrain is a pure function of the seed, and
  an evicted chunk re-derives byte-identically (journal/0051, re-proved in S15
  group 4 against the real bounded LRU with an edit pinned inside it). Unedited
  geometry cannot change connectivity, and the store cannot perturb it.
- **False:** a 1×1 tunnel dug 640 voxels out of a lake with **one solid plug**
  at its midpoint is two bodies. Removing the plug is **one audited voxel edit
  touching one chunk** — and it joins a body reaching **288 m beyond the edit**.
  The edit is local; the consequence is graph reachability, and no halo bounds
  it. Separately, **972 of 1 215** far-apart basin floors in the production
  world are already ONE body through terrain nobody has ever loaded: a container
  spanning never-generated ground is not the adversarial case, it is the
  default.

**Two adversarial cases were NOT constructible, and why is the useful part.**
A *natural* sill overtopping (two bodies merged by a rising level, with no edit
at all) cannot be built today because `generate_chunk` is a pure heightfield —
no caves, no overhangs — so all sub-level air in a basin is one component by
construction. It becomes constructible the day the cave families land. And "an
edit at a chunk boundary whose neighbour is absent" cannot be built because
`HostWorld` has **no absent state**; `block_at` materializes on demand, always.
It becomes constructible when the store can *fail* to answer — async streaming,
disk-backed regions, a network authority. **When that lands, the connectivity
index must treat an absent chunk as UNKNOWN, never as solid**, or eviction will
manufacture false component boundaries. Full account:
`docs/spikes/S15-results.md` § group 3, journal/0062.

## 32. "Comparing `fn` addresses can only mis-report in the harmless direction" (2026-07-22)

**The claim**, written on `Providers::is_identity` in journal/0060 and carried
unexamined through journal/0061: a provider slot's identity is decided by
casting its `fn` pointer to `usize` and comparing, and the only way that can be
wrong is identical-code-folding giving two *different* functions the *same*
address — reporting a custom provider as the identity, "which is the harmless
direction".

**Falsified — the opposite direction fired, and on the default set.** Splitting
`providers.rs` into a module directory (journal/0063, pure code motion) made
`Providers::default().is_identity()` return **false**, naming `outcrop_at`. One
function, two addresses.

**The mechanism.** `identity_outcrop_at` was a `pub use` of
`lithology::exposed_litho`, which is `#[inline]`. An `#[inline]` function may be
instantiated in **several codegen units**, and each instance has its own
address. Two `Providers::default()` values built in two codegen units therefore
held two different addresses for the same function. Rust does not guarantee
`fn`-pointer address uniqueness in either direction; only the folding direction
had been considered.

**Why it was green before.** The bug was latent on `main` — same comparison,
same `#[inline]`, same re-export. In the flatter file the optimizer saw both
sides of the comparison in one view and folded the result to `true`. The test
was passing for a reason unrelated to the property it asserted, and a refactor
that changed no values changed the inlining and exposed it. Byte-identity
proves a refactor moved no *values*; it cannot prove it disturbed no
*coincidences*.

**The rule that replaces the assumption**, now on
`providers/outcrop_at.rs::identity_outcrop_at`: **a slot's identity must be a
plain, non-`#[inline]` function defined in the slot's own module** — never a
`pub use` of another module's function, whose inlining attributes are not ours
to control and can change with no diff in the providers file. It is free: a
provider is always invoked through a pointer and is never inlined at its call
site anyway, so a wrapper generates the same code. The other three identity
functions already had this shape, which is why only `outcrop_at` failed.

**Superseded 2026-07-22 (journal/0064) — the remedy, not the falsification.**
The falsification above stands unchanged: `fn`-pointer addresses are stable in
neither direction, and comparing them cannot decide slot identity. What is
retired is the rule that replaced it. A rule in a docstring cannot fail a
build, and by then the mechanism was load-bearing (the resolved provider table
is part of world identity, DECIDED 2026-07-22), so a mis-report is a spurious
load refusal. Slots are now `Option<fn(..)>` with **`None` meaning identity**;
`non_identity_slots()` reads which fields are `Some`, `Providers::address` is
deleted, and no address is taken anywhere in the crate. An identity function
may therefore be `#[inline]`, a `pub use`, or anything else — the constraint
did not get enforced, it stopped existing. **Do not reintroduce address
comparison** to answer "is this slot supplied"; that is the question the
`Option` exists to make unaskable.

## 33. "`Authority::chunk_budget_for` scales with horizon" (2026-07-22)

**Claimed by:** the integrator, in the horizon-6 measurement brief, which
instructed the agent to "report what it resolves to at 6 versus 3".

**False.** The budget derives from `streaming::UNLOAD_RADIUS_M` =
`FULL_DETAIL_RADIUS_M + 32` — a constant **160 m near-field radius**. It
resolves to **2 360 at every horizon**, probe-confirmed across all eight runs.

**Why it is correct behaviour, and why the question was malformed:** the far
field is served from the coarse column summary and **never enters the chunk
store**, so widening the horizon cannot widen the budget. The brief asked the
agent to report a quantity that does not vary — the kind of instruction that
invites an agent to invent a difference rather than report a null.

*(Also false in the same brief, less consequentially: "you have no MCP tools."
The agent did have them, honoured the instruction anyway, and drove the client
over raw HTTP JSON-RPC — better for scripted timing regardless. An integrator
should not assert an agent's capabilities to it; the agent can see them.)*

## 34. "The RAM march at horizon 3 is flat — 0.00 MB/jump" (2026-07-22)

**Claimed by:** journal/0051, measured over 147 jumps after warm-up, and
carried since as the eviction fix's headline number.

**The conclusion survives; the number was phase.** RSS under teleport storm
**sawtooths** — climbing ~250 jumps then dropping 120–180 MB as eviction
catches up. A 700-jump / 23-minute run measures **+0.113 MB/jump** overall and
**+0.074** over its last 300; but *every 200-jump window sits inside a single
tooth*, and depending on where a window starts it reads anywhere from 0.00 to
+0.54. journal/0051's `0.00` and journal/0065's `+0.54` are **the same
bounded oscillation sampled at different phases**.

**The lesson, which is not about memory:** a measurement window shorter than
the period of the thing you are measuring cannot distinguish *flat* from
*oscillating*, and it will produce a confident number either way. Before
quoting a slope, establish that the window spans at least one full cycle of
whatever the system does on its own. Sibling of #25 (walking the flattest place
in the world and concluding the world was flat).
## 35. "A charcoal band cannot exist in a voxel column, so a charcoal member would be dead content" (2026-07-22)

**Claimed** verbatim in `dc_worldgen::geology::deep_class`'s doc comment
(journal/0026): *a fire bed is a thin event bed (measured mean ~0.035 m over
158 310 beds, and **none** of them survives the 0.9 m voxel quantization), so a
charcoal band cannot exist in a voxel column and a charcoal member would be dead
content. The honest representation is an inclusion (pore/debris partial) — filed,
not built.*

**Every clause survives except the load-bearing one.** The beds really are thin
(re-measured on today's production world: 102 113 beds, mean **0.0289 m**, max
0.0400 m, and **zero** of them reach even one eighth of a voxel on their own).
What expired is *"cannot exist"*. It was true of a generator that asked
`round(tᵢ / 0.9)` per unit and dropped the losers. Since journal/0055 the
generator asks the question **once per voxel span**, by unbiased **addressed
stochastic rounding**: a 2.9 cm bed claims `8 × 0.0289 / 0.9 ≈ 0.257` of an
eighth and therefore wins a whole eighth about a quarter of the times it is
asked. The mechanism the comment described as the honest answer and filed as
unbuilt had been built ~~eleven days~~ **one day** earlier, by a slice aimed at something else
entirely, and nobody went back to re-read the filings.

> **⚠ INTERVAL CORRECTED 2026-07-28** (independent re-coding, git-verified). The comment shipped
> in `75445f2` (**2026-07-20**, journal/0026); the addressed-stochastic-rounding mechanism shipped
> in `f286893` (**2026-07-21**, journal/0055); this entry is dated **2026-07-22**. The gap was
> **one day**, not eleven. **This sharpens the entry rather than weakening it** — a justification
> that went stale within twenty-four hours, in a doc comment nobody re-read, is a worse result
> than one that took a fortnight, and *"nobody went back to re-read the filings"* is
> correspondingly stronger. *Recorded because this entry's whole argument rests on the interval,
> and because an unsourced duration is the same defect as an unsourced coordinate (#28) or an
> unsourced quotation (#67).*

**Measured after routing charcoal to its own class:** 0.39 % of recorded voxel
spans carry at least one charcoal eighth; 0.0495 % of all allocated eighths in
the world are charcoal. Small, real, and exactly the inclusion. Fix: journal/0063
(`CLASS_ORGANIC_CHARCOAL`, `MaterialId::CHARCOAL`).

**The general shape** — and it is the reason this entry is worth more than the
charcoal: a justification for *not building* something is a claim with a
shelf life, and it expires silently. A stub gets an inventory entry and an heir;
a **decision not to build** gets a paragraph in a doc comment and no watcher. The
comment was still perfectly argued the day it became false.

## 36. "`promote_coal` promotes on seam thickness where burial diagenesis is a function of depth — it is a stub on the wrong axis" (2026-07-22)

> **⚠ TWO STANDING CLAIMS IN THIS ENTRY ARE FALSE TODAY — stamped 2026-07-29 (baseline sweep
> S7/F7). The MEASUREMENTS below are a dated record and stand untouched** (12,892 → 2,216
> coal-bearing cells; 13 of 35,382 peat units under 50 m).
> - ***"There is no geotherm in the project — the only temperature anywhere in the sim is
>   surface air temperature"*** — **false since 2026-07-24.** `deeptime/geotherm.rs` is the
>   first §5 field pass and writes `dc:field/temperature`
>   (`material-behavior.md` § 14, DECIDED 2026-07-24).
> - ***"listed in `stubs.md` § 14 with a geotherm as its heir"*** — **the heir landed.**
>   § 14 is **RETIRED 2026-07-24 (journal/0093)**, and coalification is now temperature-gated
>   at `COAL_ONSET_C = 22.0` °C. See **#51**, which measures the gradients (coal cells
>   **41.9 °C/km** vs peat-only **31.3**).
>
> **So *"rank is inexpressible"* is no longer true for the stated reason.** It remains true
> for a *different* one — the record still tops out far short of the ~1–2 km burial that
> discriminates lignite→anthracite. *Same one-directional shape as #40: `stubs.md` § 14 named
> its own retirement; this entry never learned of it.*

**Claimed** in ROADMAP (journal/0060's carried findings) and restated as the
brief for journal/0063. The diagnosis is right; one *implied* consequence in the
brief was wrong, and it is worth pinning because it is the kind of thing an
agent brief gets wrong by optimism.

The brief said moving to the burial axis would *move* coal placement and asked
which columns "gain or lose" it. The measured answer is not a redistribution, it
is a **collapse**: 12 892 → 2 216 coal-bearing deep cells, 19 008 → 3 888 units,
22 459 → 3 773 recorded metres. Nothing gains. That is because thickness and
burial depth are **not** independent in this record — a thick peat is a peat that
sat at a quiet, aggrading surface, which is exactly the setting that does *not*
pile a hundred metres of section on top of it. The old rule was not sampling a
noisy version of the right answer; it was selecting close to the complement of
it.

The second, self-inflicted claim: the axis is right but the *number* is not
Earth's. There is **no geotherm** in the project — the only temperature anywhere
in the sim is surface air temperature — so this is burial depth, not a P/T path,
and it cannot express coal **rank**. Of 35 382 peat-derived units in the
production Medium world, **13** lie under 50 m of section and **one** under 100 m,
so an Earth-calibrated peat→lignite threshold (10²–10³ m) would produce a world
with no coal in it at all. `COAL_BURIAL_M = 8.0` is calibrated to this record's
own burial distribution (its ~90th percentile) and is listed in
`docs/design/stubs.md` § 14 with a geotherm as its heir.

## 37. "`cargo clean -p <crate>` plus a gate proves the gate saw my code" — the cross-worktree serve (2026-07-22)

**Claimed** implicitly by CLAUDE.md's own remedy for corrections #27: before a
merge gate, `cargo clean -p <each crate you changed> --release`, then gate. Two
unstated assumptions ride along — that `-p dc-core` names *this* worktree's
`dc-core`, and that a gate which does not stop to compile has nothing to compile.

**Falsified, observed directly, with a sibling agent building concurrently into
the shared `CARGO_TARGET_DIR`.** A `cargo test --workspace --release` from this
worktree failed with ``no `CLASS_ORGANIC_CHARCOAL` in `materials::geology` `` —
dc-worldgen from **this** worktree compiled against a `dc-core` that did not
contain a constant this worktree's `dc-core` had had for an hour. The log is the
whole story: `Blocking waiting for file lock on build directory`, then
`Compiling dc-worldgen` **with no `Compiling dc-core` above it**. Re-running the
identical command once the sibling's build had drained compiled all eight crates
from this worktree's paths and went green.

**The operational rule, stated without claiming to know cargo's internals:** with
N worktrees pointed at one `CARGO_TARGET_DIR`, build state is shared and package
*names* are ambiguous. A `-p` clean and a concurrent sibling build can interleave
such that one worktree's dependency resolves against another's artifact, and the
`--release` gate then reports on a chimera. This is corrections #27's family, but
it is **not** the same failure: #27 was a stale artifact producing a false
*green*; this produced a false *red* about code that was fine — which is the
#21 shape — and would have produced a false green just as easily had the two
sources merely differed rather than failed to link.

**Two remedies, both cheap:**

- **Serialize for real, and re-read the mutex.** `target/.agent-build.lock` was
  **clobbered** during this slice — this agent created it, and it later contained
  a different agent's name and timestamp. A create-file mutex you never read back
  is not a mutex. Check the owner before every cargo call, and wait for
  `Get-Process cargo,rustc` to be empty before a gate that matters.
- **Verify the gate by what it BUILT.** Grep the log for the crate you changed —
  but grep for the right word: `cargo build`/`test` print **`Compiling`**,
  `cargo clippy` prints **`Checking`**, and a filter that only knows the first
  will report a clean clippy run as having built nothing. (That mistake was made
  and corrected inside this very slice; the 9 s clippy finish that looked like a
  false green was clippy's own check cache, which `clean -p --release` does not
  remove, plus a grep looking for the wrong verb.) "Did it pass?" and "did it
  run?" (#27) want a third question in front: **"did it build the code I
  wrote?"** — asked with the verb that gate actually prints.

## 38. "The Small world has no deep-time record" (2026-07-22)

**The claim** (journal/0066 § "What this cost, in goldens", repeated verbatim
in the thickness-rule brief): the Small contents-contract control did not move
"for the third slice running: **no deep-time record**, so no peat to promote
and no fire bed to express."

**Falsified by** the thickness-rule agent (journal/0068), which stopped on the
brief's own tripwire when Small's block hash moved. `Pregen` runs an
**always-on** deep-time field at every extent (`pregen/mod.rs:284`); the Small
world holds **~20,700 recorded deep cells**, 92 % of which change outcrop
under the dominance rule.

**The mechanism of the error**: Small stayed byte-identical across three
slices not because the record was absent but because those slices changed
record *labels and expression* — which Small's contents never surface — while
leaving erosion *rates* alone. The first slice to change erosion inputs moved
Small's bedrock geometry immediately. An unmoved control had been read as "no
substrate" when it meant "no coupling from the axis those slices touched."
The brief-premise-as-hypothesis discipline caught it: the brief said "if it
moves, STOP and report," and the agent did exactly that instead of
re-baselining silently.

**Standing lesson**: a control that never moves is evidence about the *axes
exercised so far*, not about what the control contains. Say which coupling a
control is blind to when citing its stillness.

## 39. "The coherent bilinear source biases the class split toward 50/50" (2026-07-22)

> **⚠ STILL `NEEDS RATIFICATION` (7 days) — AND DELIBERATELY DECOUPLED FROM THE USER'S
> APPEARANCE COMPLAINT, 2026-07-29.** This entry is about the **bias sign** of the coherent
> source: does the draw reproduce the true share? The user's 2026-07-29 field report — *"the
> bilinear noise does not actually approximate what loaded chunks look like well, it sticks out
> poorly"* — is about **spatial structure**, and **this correction's two named heirs
> (far-`summarize`, a CDF-corrected source) do not address it.** A perfectly unbiased draw with
> the wrong structure still fails to predict the near field. **Statistical agreement is not
> visual agreement.** Ratify or reject this entry on its own merits; it must not carry the
> appearance observation in as a rider. That observation is its own ROADMAP § Observed entry.
> *Scope: far/cold tier only — journal/0074 removed the near ground's class consult.*
>
> **⚠ ONE OF THIS ENTRY'S TWO NAMED HEIRS LEFT THE TIER, 2026-07-29 (E5 member #0,
> journal/0125).** "The far-`summarize` register" is no longer a refinement item: the user
> ruled `CoarseField::summarize` to **the octree node contract** (LOD machinery stays
> engine, and the far/LOD synthesizer is a second executor `refinement.md` does not model —
> member-#0 MM-4). The same slice adopted `sample_dithered` at the far site, so **the
> coherent source and this bias are now shipped through the type rather than a hand-rolled
> draw — and at TWO salts, not one**: the cake-law membership draw reads the same coherent
> field, deliberately (a white-noise membership draw speckles *which cell* each far sample
> reads from, which is journal/0073's aliasing one level up). *The bias is unchanged in
> kind and its remaining heir is a CDF-corrected source.* The body below still says
> `draw_class`; that function is retired — read it as `ShareVec::draw`, which is the same
> arithmetic.

**The claim** (journal/0073 § "The wrong turn"; spines § 4 carve-out 1): the
coherent (interpolated-uniform) surface-class draw carries *"a small toward-50/50
bias (interpolated uniforms are middle-heavy)"* and *"flattens mixes slightly …
a 55/45 window renders a few points closer to 50/50 than its true share."* The
cost was booked as *balancing* the mix.

**Falsified by** the `CoarseField` extraction (journal/0075), which built the
inverse-CDF draw in dc-core and measured it directly
(`coarse::tests::white_noise_is_unbiased_but_coherent_amplifies_the_majority`).
A middle-heavy `u` (the bilinear average of four uniforms bunches toward 0.5) has
a CDF with `F(s) > s` for the class that straddles the cumulative-½ point.
`draw_class` returns class 0 when `u < share₀`, so in a two-class cell the class
covering the ½ crossing — the **majority** — is drawn *more* often than its
share, and the minority *less*. A 0.6 majority renders ≈ 0.67. The split is
pushed **away** from 50/50, not toward it: the source **sharpens** the mix, it
does not flatten it.

**The mechanism of the error**: "middle-heavy `u`" is true, and it *feels* like
it should centre the output — but the output is `F(share)`, and a distribution
concentrated at 0.5 makes `F` *steep* through the middle, which pushes any share
already off 0.5 further off. The phrase "flattens mixes" is consistent with the
real behaviour (more dominated = flatter/less varied); the parenthetical
"toward 50/50 / closer to 50/50 than its true share" is the falsified half.

**Consequences**: minorities still surface (better than the plurality's zero —
journal/0073's cream sandstone specks are genuine), but at *less* than their true
areal share, so the coherent source **compounds** the cake observation
(perimeter guillotine) with a within-cell under-representation. The honest fixes
are unchanged in kind but re-weighted: the far-`summarize` register (unbiased
statistical agreement) and/or a CDF-corrected source. NEEDS RATIFICATION — the
decision to ship the coherent source stands (the far-mesh cost argument is
untouched); only its cost's sign is corrected.

**Standing lesson**: when a cost is a *distribution* distortion, measure the
output fraction `F(share)`, don't reason from the shape of the noise. "The noise
clusters at 0.5" and "the output clusters at 0.5" are different claims, and the
inverse-CDF flips the intuition.

## 40. "#28's `exhum`/`t_crust` comment claims 'the collapse tier reads them'" — the audit misquoted an already-honest comment (2026-07-23)

> **🔴 THIS ENTRY'S VERDICT IS ITSELF FALSIFIED — by #67 (2026-07-28). Struck here 2026-07-29
> (baseline sweep S7/F1); the body below is untouched, per the #56 repair shape.**
>
> **#40 concluded *"the A-2 was never live."* It was live.** #67 is git-verified
> (`git show 11d4385 -- .../field.rs`, both revisions quoted): **the audit did not paraphrase
> — it quoted the comment exactly as it stood the day before**, and the commit that changed
> the text calls the prior wording *"lying"*, which is the same verdict the audit reached.
> The audit was right about the defect and **one day behind the tree**.
>
> **#40's standing lesson — *"re-read the comment before believing the sweep"* — is therefore
> only half a rule, and #67 supplies the other half: *a quotation without a revision is not a
> quotation.*** A reader who cannot date the quote cannot distinguish a paraphrase from a
> stale read, and #40's author inferred the wrong one.
>
> `docs/design/stubs.md` § 4 and `docs/spines.md` A-2 were already amended per #67's own
> consequences list. **Only the correction itself was left** — for six days, in the file whose
> entire purpose is recording falsified claims, **1,300 lines from its own withdrawal.** The
> project applied exactly this repair to #56 on 2026-07-28 and did not carry it here.

The 2026-07-22 seam inventory listed seam #28 (`exhum`/`t_crust`) as an A-2
instance: *"documented in-code as 'the metamorphic-grade axes the collapse tier
reads' — and nothing reads them"* — a false-constraint comment (prose cannot
fail a build). journal/0078 went to cure it and found nothing to cure.

**The falsification.** The comment in the tree reads *"the metamorphic-grade axes
the collapse tier **WILL** read (§ 6.4): exported and, as of U8, populated in
every production world, but **currently consumed by nothing**"* — rewritten at
commit `11d43859` (2026-07-21 04:33, "U8: flip tectonic_history ON"), *before*
the audit was written. The audit quoted it as "the collapse tier reads them",
dropping the "WILL" and the "currently consumed by nothing" clause — which
inverts an honest future-tense note into a false present-tense claim. A comment
that already states "consumed by nothing" is not a justification outliving its
premise; the A-2 was never live.

**The mechanism of the error.** An audit that paraphrases a comment can
manufacture the very defect it reports. The check A-2 prescribes — *"does the
cited constraint still hold?"* — has a prerequisite the audit skipped: *is the
constraint quoted correctly?* Verify the quotation against the source, not just
the claim about it.

**Consequences.** No code was wrong; stubs.md § 4's "the comment overstates" was
the same misreading and is corrected. journal/0078 tightened the comment anyway —
it now cites spines.md § 3 and names the `burial_temp_c` geotherm arrival address
— but that is polish, not a fix. The `exhum`/`t_crust` row stays in spines.md § 3
(built-but-unconsumed): the comment correction does not consume it.

**Standing lesson.** When a sweep reports "the comment says X and X is false",
re-read the comment before believing the sweep. A stale or uncharitable quotation
is indistinguishable, in a summary, from a live defect — and only one of them is
worth a slice.

## 41. "Raising the erosion budget will raise the world's relief" (2026-07-23)

> **🔴 THE ORIGINAL CLAIM WAS RIGHT AFTER ALL; THIS ENTRY'S FALSIFIER WAS BLIND —
> stamped 2026-07-29 (baseline sweep S9/S9-1). Nothing below is rewritten.**
>
> The journal/0079 probe was faithful to the launch path it swept and **blind to
> `diffusion`**, which carries **96 %** of this world's export (creep does **918×** what the
> rivers do). `erosion_budget` scales `weathering`/`k_transport`/`k_bedrock` and **does not
> scale `diffusion`** — *"a knob that cannot move the thing it is named after"*
> (`journal/0111`, corrections **#56**, `stubs.md` #24). Measured since: budget 100× **+ creep
> 10×** buys **132×**, and `journal/0114` measures relief **+4.6 % at 45×, +18 % at 100×,
> +52 % at 300×**. **Erosion does move relief.**
>
> **What survives, and it is the entry's real value:** the *equilibrium/graded-to-base-level*
> mechanism is still the correct account of what the swept knob does, and the discipline it
> teaches — *the falsifier here is a measurement, not an argument* — is untouched. **What is
> withdrawn is the conclusion "relief is bottlenecked on the generating side, not erosion."**
> ⚠ **Do not restate the world as "supply-limited today"**: that holds of the *calibrated*
> world, and `EROSION_CALIBRATION = 45` ships **OFF**.
>
> *This is CLAUDE.md's own "corollary for reading a null" — written for journal/0110 — landing
> on journal/0079 word for word, three days late.*

**The claim.** Standing since journal/0029 and framed as the path forward in
journal/0076: the erodibility contrast is modest because the world "barely erodes
against its uplift", and *"raise the global erosion rates … and the same coupling
cuts 20 m of relief with a hard bed standing 44.7 m proud"* — i.e. the erosion
budget is the amplitude lever, waiting only on a walk to pick the magnitude.

**The falsification (journal/0079).** A faithful probe (1× asserted byte-identical
to shipped `build_field`) swept `--erosion-budget` 1×/3×/10×/30× on the client
world (seed 1337, Medium). Relief was **1287 m at every budget**; mean |Δsurf| vs
1× reached only **0.35 m at 30×**; lithologic separation did not widen. A mechanism
probe measured lowering against the zero-erosion counterfactual: **mean 41 m, max
1.4 km, and flat across the whole sweep**. The landscape is at erosional
**equilibrium** — graded to base level — so it is neither supply-limited (it
erodes a lot) nor iteration-starved (flat, not growing). Incision is capped by
`inc_pot.min(room).min(max_inc)`, `max_inc = r − floor` (`erosion.rs:1149`): a
graded cell stops incising regardless of `k_bedrock`. Raising the rate reaches the
same equilibrium faster, not deeper.

**Why experiment B looked otherwise.** journal/0076's cited "44.7 m proud / +20 m
relief" was on a *different seed* and a *different measurement* — the ON-vs-OFF/
UNIFORM differential at a fixed high budget, not absolute change vs 1× on the
production world — so it was never a prediction that the *production* surface would
move with the budget. The two are not in contradiction; the leaked inference was
"therefore cranking the budget carves visibly here", and that is false.

**The mechanism of the error.** A headroom number from an isolated probe on a
hand-picked seed was read as a promise about the shipped world. An experiment that
isolates a *differential* says nothing about the *absolute* response until you run
the absolute measurement on the real world — which is what the faithful sweep did.

**Consequences.** No code was wrong; the `--erosion-budget` flag is correct and
stays as a dev lever (on regolith / approach-to-grade). journal/0029's cause 3
("conservative amplitude") is reframed as a **deep-field relief-generation**
problem (converging with journal/0040 and S13), not an erosion-rate decision.
ROADMAP item 2 and the evening-close "amplitude walk" are marked resolved.

**Standing lesson.** A probe that cranks one term in isolation measures that
term's *ceiling*, not the shipped system's *response*. Before believing "lever X
will move the world", measure X on the world, faithfully (prove the instrument
reproduces the launch), and against the counterfactual that separates
"rate-limited" from "already done".

## 42. "Render-first is a cleanly-separable byte-identical wedge" (assistant, 2026-07-23)

**The claim.** In shaping the block↔material collapse, the assistant recommended
**render-first** as the safe first slice: delete `meshing.rs::block_layer`'s
geology re-translation, route `classify → material → atlas`, byte-identical.

**The falsification (journal/0082).** The wedge was *already done* for the path it
named and *impossible* for the path it forgot. The near-field mesher already
routes contents-bearing voxels `contents → dominant_material → material_layer`
(journal/0010's splat); it never calls `block_layer`. `block_layer`'s geology arms
are live only on the **block-only** render path — the far field, benches, and the
absent-contents fallback — which is block-only *because the far field carries
blocks, not materials.* Deleting them in isolation would break the far field or
make `block_layer` non-total. **The same mechanism as the four "legacy" arms the
brief said to keep.** Crux 1 (`Block={Air,Material}`) subsumes them all; a
piecemeal deletion "only trades one carve-out for another."

**Mechanism of the error.** The assistant reasoned against a mental model of the
mesher the code had already outrun (the near field was collapsed a slice ago). The
brief-as-hypothesis discipline caught it at work-time (a loud plea, not a forced
mess) — which is the point of writing briefs as hypotheses. Deliverable: a guard
test pinning the accidental layer-equivalence, and a corrected migration map.

## 43. "Synchronous chunk generation is the vertical-drop killer" (project premise, 2026-07-23)

**The claim.** The ROADMAP's standing perf suspect: `streaming.rs`'s synchronous
`chunk.gen` on the main schedule is what makes a vertical drop choppy.

**The falsification (journal/0080, the perf baseline).** A faithful `--perf-drop`
capture: `chunk.gen` is **0.1 %, 14 µs/call — cheap.** The per-frame killer is
**CPU meshing** — `far_tile.derive` 10.3 %, `mesh_chunk` 8.2 %, `far_tile.mesh`
2.6 % — plus `neighbor_fill.gen` (5 ms/call). Generation is lazily cached and
genuinely fast. The async-offload slice was **retargeted** off gen onto meshing as
a direct result (journal/0083/0084 → +20 % frames).

**Mechanism / lesson.** The suspect was a plausible unmeasured premise that had
sat in the ROADMAP for days. This is precisely why the observability instrument
was built: *the instrument overturned the suspect its own slice was filed under.*
Do not offload against a hypothesis; measure first. (Carried: the perf improved
onset but a **throughput ceiling remains at terminal velocity** — see ROADMAP
Observed.)

## 44. "`column_summary`'s `fully_resolved` is live in the renderer today" (assistant, 2026-07-24)

**The claim.** Building the S-9 spine, the assistant wrote that the far-field LOD's
pre/post-visit discrepancy was the *same* live system as S3's `fully_resolved`/
`sky_exposed` — "the renderer does observation-collapse today, unnamed."

**The falsification (grep + the 2026-07-24 spine-audit).** `fully_resolved` is real
and tested, but it lives on `ColumnInfo` in `dc-core/src/column.rs` and is **dormant**
— its only non-test caller is the `--bench-storage` harness; no renderer or lighting
consumer exists (sim-light unbuilt). Moved to spines § 3. The far-field LOD path is
different code entirely (`far.rs`/`farpyramid.rs`); the assistant welded two unrelated
systems.

**Mechanism / lesson.** A claim about *live code*, asserted from a design-doc reading
instead of a grep; the user caught it directly ("does the renderer do `fully_resolved`
today, or is that just a plan?"). Verify a claim about live code against the code, not
the doc that states the intent. Nearly shipped a false "already implemented" into the
spine.

## 45. "The member dither is chunk-anchored and ignores its neighbours" (assistant + user, shared premise, 2026-07-24)

**The claim.** Diagnosing the near-field material squares, both parties held the member
dither is "anchored to a chunk with no awareness of its neighbours," snapping at chunk
lines.

**The falsification (reading `interp_select_draw`, `geology.rs:198`).** It hashes four
corner values at **absolute** chunk corners (`cx,cz`) and bilinearly interpolates — so
it is **world-anchored** and **C0-continuous across seams** (adjacent chunks share the
corner hash). It does not ignore neighbours. The real defect: it is **a single octave
of value noise at chunk wavelength** — one scale, so every patch is chunk-sized and
reads as a grid though continuous.

**Mechanism / lesson.** One symptom ("squares aligned to chunks"), two candidate causes
(no-neighbour-awareness vs single-wavelength) that both predict it; we assumed the first
until reading the function. The fix follows the true cause — **octaves (multi-scale),
not a finer grid or neighbour-awareness**; a finer single-octave grid just makes smaller
squares. Read the noise function before prescribing its replacement.

## 46. "S18 weathering expresses a saprolite band in the walkable world" (assistant, 2026-07-24)

**The claim.** The S18 merge (ROADMAP Shipped; journal/0089) stated the collapse folds
`base + facts` into a "basal saprolite band" — recorded as if the first behavior is
visible in the world.

**The falsification (headless tour probe `examples/s18_weathering_tour`, then in-client).**
At production scale the strongest band *anywhere* is **0.04 m** against **0.9 m** voxels
(mean 0.01 m over banded cells). `quantize_to_eighths(0.04, 0.9) = round(0.35) = 0`: the
band reaches **zero eighths everywhere**. Confirmed in-client — a column scan at the
strongest cell reads stone/dirt/air, no saprolite voxel. The collapsed world is effectively
byte-identical; there is nothing to walk to.

**Mechanism / lesson.** Acceptance tested the **mechanism**, not the **outcome**. The fold
test (`the_weathering_front_folds_into_a_basal_band`) is green because it **hand-feeds
1.3 m** — an **A-3** green: true of the fold, silent on the production claim. The agent's
report gave a *recipe* for the exemplar (argmax `weathering_product_m`) but never its
**value**; the integrator recorded "it expresses" without demanding the number (violating
"an agent's mechanism is a hypothesis; test before recording"). **Fix (process):** a slice
claiming a world-visible outcome ships a **production-scale outcome probe as acceptance**,
and review **demands the load-bearing number**; the tour-map probe that caught this runs
**inside the slice**, not post-merge.

## 47. "S18 is the first real weathering BEHAVIOR" (assistant / brief, 2026-07-24)

**The claim.** S18 was briefed and shipped as "the first real cellular behavior" — modelling
subaerial weathering on the deep-cell inventory.

**The falsification (user's question, reading `weather_inventory.rs` + `field.rs::build_ledgers`).**
Weathering is a **continuous** process — the height-tier loop runs it *every epoch*, which is
why `H` holds meters of regolith. S18 runs `weather_column` **once**, *after* the run, over
the *finished* record, with the *final-state* fields frozen. That is not a small weathering;
it is a single synthetic application — **a snapshot of a continuous process, a category error,
not a simplification.** What shipped is keystone-consumption **plumbing** with a one-shot
behavior stub, **not weathering-as-a-process.**

**Mechanism / lesson.** The brief scoped it decoupled from the deep-time loop ("one chapter,
don't touch the erosion loop") to dodge the R/H entanglement, then labelled a pipework demo
"the first behavior." Two durable rules: **(1) a seam is placed WHERE THE FULL THING WILL
LIVE** (user, 2026-07-24) — weathering lives in the loop, over history, not bolted onto the
end; a seam in the wrong place is a dangling extra step, not a proxy for later work. **(2)**
For a slice modelling a natural **process**, ask *at design time* whether it runs *where and
when the process runs* (in the loop, over the span) or as a decoupled snapshot; a one-shot of
a continuous process is incoherent. **Fix:** the real first behavior is weathering run as a
per-epoch pass in the deep-time compile / **riding the `H` process** (the R/H unification),
carrying material identity + per-agent `cause` on top of the process the height loop already
runs. Tracked as **stubs.md #17**.

## 48. "The palette-quant station is at the east coast, ~110 km east of spawn" (assistant / ROADMAP, 2026-07-24)

**The claim.** The palette-quantization diagnostic station (journal/0088, the chunk-seam
checkerboard) was recorded in ROADMAP **Observed** with a prose landmark only: *"At the
**east coast** (~110 km east of spawn) … a top-down view shows the near/mid field as a grid
of chunk-sized squares."* No camera pose was stored — not in the journal entry, not in the
commit that added the screenshot (`530dccf` touched only ROADMAP + the PNG), not in the
diagnosis audit (whose § 7 in fact listed *"record the camera pose at observation time"* as
an unfilled needs-a-live-probe item).

**The falsification (user, recovering the pose from an old transcript, then flying to it).**
The station is at **`x ≈ +71.3 km`**, not ~110 km — **wrong by ~39 km**. The error is not
cosmetic: at 108–110 km the east coast is **grey single-class stone and open water at ~sea
level**, where the multi-class checkerboard is *absent*. A reconstruction attempt driven by
the recorded landmark therefore searched the wrong region, shot four null frames, and
concluded "the region is right but the signature isn't here" — a null produced by a bad
landmark, not by the world. The true pose (user-recovered, re-shot and confirmed against the
original capture): **feet `x 71291.7, y 372.1, z -2420.9`, `yaw 21.9968`, `pitch -1.5475`**
(`journal/assets/0088-palette-quant-reference-station.png`).

**Mechanism / lesson.** A **prose landmark is not a pose.** The user had explicitly named this
spot a standing *reference point* for the palette-quant issue, yet what entered the corpus was
an approximate direction-and-distance — and an approximation that was itself wrong, with
nothing able to fail. This is **"defer = write it now" applied to camera poses**: the instant a
station is called a reference, its **exact pose** (feet in world metres, yaw, pitch) goes into
the Observed entry in the same session, beside the asset. Corollary of the A-2 family — prose
cannot fail a build, and a landmark cannot be re-derived from a screenshot. **Fix:** the exact
pose is now recorded in the ROADMAP station entry with this correction beside it; re-shoot that
pose to compare before/after any palette-quant fix.

## 49. "The contents record reads empty over solid ground" (walk observation, 2026-07-25 — a QUERY bug, not a world bug)

**The claim** (journal/0097, filed honestly as *reported, not diagnosed*):
`world_get_contents` returned `dc:air` for voxels 288–299 while
`client_player_pose_set` reported `eye_in_solid: true` — read as ~11 voxels where
**the record is empty and the world is solid**, a record hole adjacent to the
bare-cell fallback.

**Falsified.** The world was solid and correct there. Unrecorded basement is
`Block::Stone` **by construction** (`collapse.rs:457-459`), and **no voxel below a
column's height can be `Block::Air` at all** (`collapse.rs:434-436`) — a structural
proof the column was never empty. `eye_in_solid` and `get_contents`'s **own `block`
field** read the *same* `HostWorld::block_at` (`authority.rs:480-487`,
`host.rs:1269`) and both correctly said stone. **They never disagreed.** What said
`dc:air` was the *derived* `classified` field.

**Mechanism.** `HostWorld::contents_at` (`host.rs:320-326`) answers a **per-voxel**
question with a **per-chunk** presence test: `chunk_contents` returns `Some(grid)`
if *any* voxel in the 32³ chunk is recorded (`collapse.rs:717-724`,
`intern.rs:313-314`), and inside that grid an unrecorded voxel resolves to a
perfectly ordinary `VoxelContents::EMPTY` (`intern.rs:101`, `:409-418`). So
`has_contents` reports **`true`** (`host.rs:83`) — contradicting its own
documentation in three places (`payload.rs:426-429`, `schema.rs:754-756`,
CLAUDE.md) — and `classified` reports **`dc:air`**, which is `classify` applied to
an unrecorded voxel: precisely the operation `classify.rs:31-45` forbids, naming
*"the unrecorded basement below the deep-time record"* by name. **The band's extent
(288–299) was set by the chunk floor at `9 × 32 = 288`, not by anything in the
world** — reproduced to the voxel at journal/0097's own station.

**Scope:** 702 / 10,985 solid voxels (**6.4 %**) in the top 65 of a column; **39 of
169** sampled columns; global, worldgen authority only. The F3 HUD
(`inspector.rs:125-133`) and `character_sense_raycast` (`host.rs:1424-1426`) carry
the identical bug — and their correct *"no contents record here"* branch is
**unreachable** in this case. The mesher (`meshing.rs:295-306`) and far field
(`farfield.rs:134-152`) are **immune** (block gate + `!c.is_empty()`), which is
exactly why it survived this long: **the only high-volume consumer is structurally
immune, and the two misled surfaces are the diagnostic ones.**

**Lesson — and it is the inverse of the one we already knew.** `Option::None` was
the *only* channel that could say "no record here", and it had already been spent
on a **whole-chunk** condition inherited from the **mesher's** needs. *A summary is
not an authority* — **including when the summary is a `bool` named after the thing
it is not measuring.** Where "block is a summary" is a voxel-level summary standing
in for a mixture, this is a **chunk-level summary worn as a voxel-level authority**:
the same defect, one tier up and inverted.

**Two process notes.** (1) The walk was right to file this as an observation rather
than a diagnosis (**A-5** honored) — its *instinct* that this differed from the
bare-cell fallback was correct, while its *reasoning* for the distinction ("paint
over stone vs record empty over solid") rested on the false premise. (2) The
integrator's caveat — *"was it air, or `has_contents: false` read as air?"* — asked
the right question and offered **two wrong answers**; the truth was a third thing
(`has_contents: true` over an EMPTY composition). Asking the discriminating
question mattered more than the hypotheses attached to it.

**Fix:** ~~not applied (diagnosis-only)~~ — **APPLIED 2026-07-25, journal/0101.**
`HostWorld::identify(pos) -> Identity` (`dc-api/src/identify.rs`) is the honest,
**untiered** surface, and `Identity::Unrecorded` is a first-class value distinct
**in the type** from `Mixture(VoxelContents::EMPTY)`. `has_contents` is now a
per-voxel fact, `classified` echoes the stored block where no record backs the
voxel, `character_sense_raycast` answers `None` as its doc always promised, and
the F3 HUD's `(no contents record here)` branch is reachable. The enabler was
already in hand: the stored `Block` disambiguates (empty record + `Air` ⇒
genuinely empty; empty record + anything else ⇒ unrecorded), so **no dc-worldgen
change was needed**. Residual: the answer is still **edit-blind for composition**
(the arc's runtime edit-fact overlay).

## 50. "The weathering front's voxel tier expresses +3.7 % more product than the record owes" (journal/0099, 2026-07-25 — an instrument artifact, not a world bias)

**The claim.** journal/0099 measured the front's voxel-tier mass over **21
columns** at **+3.7 %** (56.14 m expressed vs 54.12 m owed), with a **+16 %
median**, and called it quantization noise under journal/0055's unbiased-estimator
doctrine. The integrator's review (ROADMAP Observed) correctly objected that an
unbiased estimator's population mean trends to **zero**, and that a positive
*median* is the signature of a **floor effect** — a band thinner than one eighth
cannot express as less than one eighth without vanishing. Both readings assumed
the measurement was sound.

**Falsified — the measurement was not sound.** The probe counted **materials in
the finished voxel**: every eighth of the product's material found in a front
voxel was credited to the front. But the weathering product is
`CLASS_CLASTIC_FINE` (mudstone in this world) and so is much of the sediment pile
lying directly **on top of** the front, and at the top contact the two share a
`Mixed` voxel. Over **247 production columns**, **222 of them** contain at least
one front voxel whose fill plan holds a *non-front* event made of the product's
own material. **Voxel contents carry no provenance** — a finished voxel is a
multiset of materials and does not record which event contributed which eighth —
so the census could not tell the front's mudstone from its neighbour's.

**Mechanism of the fake trend.** Restricting the comparison to *attributable*
voxels (both sides), and splitting the pipeline into its two quantizers, the
aggregate error falls from **+6.76 % to −0.60 %** and the per-column distribution
becomes symmetric about zero (mean **−0.10 %**, median **−0.00 %**, p5 −30.00 %,
p95 +28.72 %, N=247):

| stage | aggregate | mean | median |
|---|---|---|---|
| naive record → voxels | +6.76 % | +12.59 % | +6.01 % |
| **stage 1** record → fill geometry | **−0.02 %** | +0.03 % | +0.00 % |
| **stage 2** the draw, attributable | **−0.60 %** | −0.10 % | −0.00 % |

The naive figure's beautiful monotone decay with front thickness (+76 % at 2–4
eighths → +4 % at ≥24) was **not** a floor effect. It was the *contact voxel's
share of the front* shrinking as the front grows: a thin front is mostly contact,
a thick one mostly interior, so the one contaminated voxel dominated a thin
front's count and barely touched a thick one's. Stage 2 shows no such trend —
only a spread that widens as the front thins, which is exactly what an unbiased
estimator over a one-eighth quantum does (a 2-eighth front cannot be wrong by less
than ±50 %, and is wrong in **both** directions).

**journal/0055's unbiased-estimator doctrine is NOT falsified — it is confirmed**,
and confirmed from the code as well as the data: `fill::allocate_to` is systematic
sampling (a Cranley–Patterson rotation) with `P(extra) = remainder` exactly;
`allocate_partial`'s rescale floors the *cumulative*, so errors cancel along the
run; `pore_rider_share`'s mean is exactly `cnt·k8/8`. **Nothing rounds up at the
floor.** (The one real floor in the path, `contents_for_event`'s
`eighths.clamp(1, 7)`, cannot bite on a profile that runs `[7,5,4,3,2,1,1,1]`.)

**Lesson, and it outlives this front.** *A conservation audit at the voxel tier
cannot work by counting materials.* The voxel does not know who put the material
there; only the **fill plan** does. Any mass budget built on the voxel tier —
flow.md § 3's, next — must compare against the plan, not against a census of the
finished contents. Also: **21 columns is not a population.** The naive median was
+16 % at N=21 and +6.0 % at N=247; it was moving with `N` the entire time, which
was itself the small-sample tell.

**Residual, stated:** 222 of 2 077 front voxels (10.7 %) are excluded from the
attributable figure and they are systematically the *top contact* voxels, not a
random tenth. This instrument cannot measure them at all. The conclusion rests on
stage 1 covering **100 %** of voxels and coming out flat, plus the code-level
proof that the draw is unbiased by construction. See journal/0103.

## 51. "The geotherm's coal recalibration still leaves a diggable seam on Medium" (journal/0093 / ROADMAP, 2026-07-24 — falsified 2026-07-25)

**The claim.** journal/0093 shipped the geotherm's coal recalibration (`COAL_ONSET_C 8 → 22 °C`)
with: *"12 % → 60 % of peat candidates, relocated to warm crust, **still a diggable seam on
Medium**"*, guarded by `the_geotherm_coal_shift_is_plausible_not_degenerate` and
`MIN_DIGGABLE_COAL_VOX = 6`. The ROADMAP recorded it as **world-changing: coal moves**.

**Falsified — the shipped world has NO COAL AT ALL.** On the world `dc-client` actually boots
(`BENCH_SEED = 1337` at `dc-client/src/bench.rs:17`, `WORLDGEN_EXTENT = Extent::Medium` at
`authority.rs:58`, passed to `Pregen::run_with` at `:209`): **0 coal units across all 297,025
deep cells**, against **27,134 peat units in 14,596 cells**. Not a thin seam — **zero**. The
hottest coalification candidate on the entire world is **15.4 °C**, i.e. **6.6 °C short of the
22 °C onset**; candidate temperatures run `p05 3.4 / p50 6.2 / p95 13.2`. The trial-onset curve
on this world is a cliff: `4 °C → 87 %`, `8 °C → 31 %`, `12 °C → 9 %`, **`16 °C and up → 0 %`**.

**Why the guard did not catch it — the root defect, and it is a naming failure as much as a
testing one.** `tests/geotherm.rs::production_field()` builds
`seed 0x0B0A_57EE_0059, Extent::Small`. **That is neither the production seed nor the production
extent.** A helper *named* `production_field` builds a world nobody ships, and every assertion
resting on it — including the A-3 guard written specifically to stop a magnitude claim from being
believed unverified — was measured on that world. A third seed (`0x0D5EED572026`) appears
elsewhere in the coal corpus, so the coal evidence is spread across **three worlds, none of them
the one the player walks.**

**The instrument is proven, so the zero is real.** A zero from an unproven census is not evidence,
so the probe (`examples/coal_walk_tour.rs`) ran the identical code over
`0x0D5EED572026 / Medium` — the world journal/0093's numbers came from — and found **1182 coal
cells / 1834 coal runs**, thickness p50 0.47 m / max 7.05 m. **The census sees coal when coal
exists.** The production zero is a fact about the world, not about the tool.

**The geotherm's physical claim is NOT falsified.** On a world that has coal, coal genuinely
followed the warm crust: coal cells mean gradient **41.9 °C/km** vs peat-only **31.3**, mean
surface T **22.9 °C** vs **11.4**; rift/arc ≥ 40 °C/km → **13.4 % coal**, craton < 20 → **0 %**.
The mechanism works. **Seed 1337 simply has no warm crust with peat on it**, and a threshold
calibrated on warm worlds fell off a cliff on a cold one.

**Lessons.**
1. **"Accept by OUTCOME, at production scale" means the world the PLAYER boots** — not a world
   named "production". This is corrections #46 one turn deeper: there the A-3 guard was green on a
   hand-fed magnitude; here it is green on a hand-picked *world*. **A guard is only evidence about
   the world it actually ran.**
2. **A helper named for an environment must BE that environment**, or it is a summary wearing an
   authority's clothes at the harness level. `production_field()` is the single highest-leverage
   line to fix, because every future coal/geotherm claim will route through it.
3. **A threshold calibrated on one world needs its sensitivity reported, not just its value.** The
   trial-onset curve (87 % → 31 % → 9 % → 0 %) shows `COAL_ONSET_C` sits on a cliff edge; that
   curve was computable at calibration time and would have shown the risk immediately.
4. Integrator note: the ROADMAP already carried *"don't over-calibrate a placeholder — it will just
   be calibrated again"* (user, 2026-07-24). That was correct guidance about **effort**, and it is
   **not** a licence for the calibration to go unverified on the shipped world. Cheap-to-check and
   not-worth-tuning are different things.

## 52. "Reusing the fill draw's bits here would correlate the two decisions into a visible pattern" (`collapse.rs::pore_rider_share`, journal/0099 — falsified 2026-07-25, journal/0105)

**The claim.** `pore_rider_share` justified its offset with: *"The offset is a **low digit** of the
voxel's own fill draw, not its high bits: `allocate_partial` consumes the high end, and reusing it
here would correlate 'this band won an extra eighth' with 'the product won an extra eighth of it'
into a visible pattern."* journal/0103 falsified the **premise** (the "low digits" are inside the
twenty bits the allocation consumes — ROADMAP Observed). This entry falsifies the **consequence**.

**The coupling was total.** Over 464,521 real rider decisions on the production weathering world,
the pore offset was **100.00 % predictable** from bits 8–10 of the allocation's offset — not
correlated, a *deterministic function*, zero conditional entropy.

**And it produced nothing the comment predicted.** Residual-vs-residual between the two decisions:
`r = +0.0006`. Mutual information 0.0609 bits against a **measured** estimator floor of 0.0610.
The dither's entropy conditioned on `(band, what the allocation did)`: **2.999 of 3.000 bits**.
Spatially — the actual meaning of "a visible pattern" — a 32×32 contact plane sharing one record
and one fill plan has **every** autocorrelation at lags 1–4 inside ±0.07 of zero, before and after
the fix, and same-sign run lengths of 1.889 vs 1.947 against 2.000 for no structure.

**The mechanism.** The allocation's decision is a *contiguous interval* in its 20-bit offset (a
material wins an extra eighth when the offset lands inside a window as wide as its fractional
remainder). Bits 8–10 are a **fast sawtooth** across that space, cycling all eight values every
2,048 of 1,048,576 counts — so conditioning on the allocation's interval leaves the pore offset
uniform unless a band's remainder is under 0.2 % of an eighth, which no real contact is. **The
shared bits were the wrong bits to matter.** Additionally, two decisions at *one* voxel cannot make
structure *between* voxels: both offsets are functions of a position hash, so both fields are white
noise, and "banding" was never a shape this defect could take.

**The harm was real and somewhere else.** The same `u` served **every band in the voxel**, and a
weathering front puts several thin bands of one parent in one contact voxel. Sibling riders
therefore rounded in lockstep — `r = +0.4878` over 187,701 pairs — and their errors **added**
instead of cancelling: a multi-band voxel's total product carried **1.488×** the second moment
independent roundings give. Nobody had written a comment about that, so nothing expired; it was
simply never checked.

**Lessons.**
1. **A justification that names a consequence is a testable claim — test it, in both directions.**
   This one was wrong about its mechanism and right about its outcome, which is the combination
   most likely to survive review forever.
2. **Chase a wrong justification even when its stated harm turns out to be nil.** Looking for the
   harm the comment named is the only reason the harm it did not name was found.
3. **"Visible" is a spatial claim and needs a spatial instrument.** ROADMAP had filed *"cheap next
   step: a fullbright walk looking for banding"*; that walk would have returned a null from an eye
   on a signature structurally incapable of existing, and a null from the wrong instrument proves
   nothing (corrections #18/#19). An autocorrelation over a contact plane can distinguish "no
   banding" from "banding I did not notice"; a screenshot cannot.
4. The durable fix was not a corrected comment but a **mechanism that makes the claim unnecessary**
   — the `Domain`/`Draws` provider, where disjointness is a compile-time property rather than an
   assertion in prose (journal/0105).

## 53. "Nine test sites fail under f32 fact storage" (`docs/spikes/S20-*` § 4.2, 2026-07-25 — falsified the same day by journal/0108's implementation)

**The claim.** S20 § 4.2 sorted every stored-value assertion in the tree into Class A ("stored vs
stored — unaffected") and Class B ("stored vs a freshly-computed f64 — **fails**"), and published a
nine-row Class B table with a measured error beside each row: `weather_inventory.rs:417, :482,
:504`, `inventory.rs:1643, 1647, 1652, 1654`, `s17_deep_cell_inventory.rs:91, :100`. The § 7
summary carried the count forward — *"it will be rejected for the nine test tolerances"* — and the
brief that implemented 2c inherited "all nine Class B sites" as a deliverable.

**What shipped.** **One** site moved class. The other eight pass **untouched, at their original
`1e-12` / `1e-9` bounds**, in a green workspace gate.

**The mechanism, which the spike itself named two subsections later.** § 4.2's verdicts were
computed against a model where `Fact` stores `f32` *everywhere*. The design that shipped narrows
**only at persist**: `LedgerField::from_accumulators` is the single narrowing point and the
gen-time `FactLedger` accumulator stays `Fact<FracM>` at f64, precisely so the per-epoch `*q +=
share` never rounds. Every one of the eight surviving sites reads a `FactLedger`. They are `f64`
vs `f64` — not stored vs computed — and never changed representation at all.

The one that did move is `weather_inventory.rs:549`
(`bedrock_facts_key_stably_as_the_record_grows`), which compares a band summed from the accumulator
against the same band summed from the finalized record — **the only assertion in the tree that
straddles the persist boundary.** § 4.3's final paragraph predicted precisely that site, and
predicted the direction (Class A → Class B). Its replacement bound is derived from the storage's
own resolution and published beside the constant (`inventory::stored_fold_tolerance`).

**Why this is worth recording rather than shrugging off.** The spike was *not* wrong about the
physics, the error magnitudes, or the mechanism. It was wrong in a subtler and more contagious way:
**a table stated verdicts without their condition**, while the condition sat in prose two
subsections away. A reader — human or agent — who reads a table reads the table. "Nine sites fail"
travelled into § 7's summary and into an implementation brief as a fact, and it would have
travelled into a design decision ("is 2c worth nine broken tolerances?") that was being weighed on
a blast radius **9× too large**.

**Lessons.**
1. **A verdict whose truth depends on a design choice belongs in the same cell as the choice.**
   "Fails" should have read "fails *if the accumulator narrows too*". A conditional stated once, in
   prose, in a different subsection, is not attached to the claim it qualifies.
2. **Where an optimization is applied is part of the optimization.** "Store f32" is not a design;
   "store f32 *at persist, once, widening on read*" is — and the two have different blast radii,
   different error behaviour (single rounding vs compounding), and different test consequences.
3. **Over-prediction is still a false number.** It is the flattering direction — the implementation
   comes in "under budget" — which is exactly why nobody checks it.
## 54. "MFD needs the head field — `dc:field/head` is what lets flux partition across several receivers" (flow.md § 2.6, spines § 3, `head.rs` docs, 2026-07-25 — falsified the same day by journal/0109)

**The claim**, written in at least four places and never questioned because it sounded like a
sequencing fact rather than a physical one:

- flow.md § 2.6: *"MFD is a SOLVE change and **belongs with the potential/head field**
  (continuation (a)): **a head field partitions flux across several receivers naturally**, where
  steepest-descent cannot."*
- flow.md § 9 Q8: *"Simultaneous divergence needs an MFD solve… **Sequenced with the potential/head
  field (continuation (a)), never ahead of it.**"*
- `head.rs` module docs: *"A **multi-flow-direction** partition is what head *unlocks*."*
- spines § 3, the `DeepField::head` row: *"an **MFD solve** — head is what lets flux partition
  across several receivers, and therefore what makes *simultaneous* divergence representable."*

**Falsified: MFD needs a potential, and the free regime's potential was already there.** For
free-phase flow, head is `z_bed + depth`, and *depth is zero on dry ground* — so the driving
potential of overland and channel flow is the **free water surface**, which is exactly what the
priority-flood `filled` array is (bare ground where the land drains, a flat spill-level water
surface inside every depression). The MFD partition that shipped in journal/0109 descends `filled`,
reads no new plane, adds no declaration, and produced 7.5 M simultaneous divergences on the shipped
world. **`dc:field/head` is not in the call path at all.**

**And the stronger half: using `dc:field/head` here would have been WRONG, by § 2.4's own
qualification.** That plane is the **bound** regime's potential, and it is deliberately built to
cross surface drainage divides — the module docs pin it with
`bound_head_crosses_a_surface_drainage_divide`, because the Great Artesian Basin and karst piracy
are real and a field that could not express them would foreclose them. § 2.4 then says in the same
breath that *"3e-2 decision 1's 'never crosses a drainage divide' … **binds FREE/surface refinement
only**"* — i.e. surface water **must not** cross divides. Partitioning surface discharge on the
water-table potential would have made every river cross its own watershed. The document contained
both halves and the sequencing note read only one of them.

**What was actually true, and is worth keeping.** Continuation (a) was correctly sequenced *before*
(b) — but for a different reason than the one written down: the head field is what made the record's
**vertical** faces honest (journal/0098), which is what let (b) change the *lateral* solve without
the record's other half still being a stub. The dependency was on the record's completeness, not on
the numerics.

**Lessons.**
1. **"X unlocks Y" is a physical claim wearing a schedule's clothes.** It was recorded as a
   sequencing constraint, which is the kind of statement nobody re-derives, and it survived four
   rewrites of the surrounding text.
2. **One word — *potential* — named two different fields.** The corpus had `head` meaning "the
   groundwater plane `dc:field/head`" and `head` meaning "the potential any flow descends", and the
   collision hid a category error. flow.md § 0's own table has *fluvial → head* and *solute →
   head*, which is right in the second sense and reads as the first.
3. **The correction is now a mechanism, not a note:** flow.md § 2.6.1 states which field is
   partitioned and why, and names **bound MFD on `dc:field/head`** as the genuinely-unbuilt thing
   continuation (c) owes.

**Struck at source, 2026-07-25** — because a correction that lives only in this file is a
correction the next author does not meet: flow.md § 2.6 and § 9 item 8, `head.rs`'s "what is
deliberately NOT here" bullet, and the `DeepField::head` row of spines § 3 all now carry the
struck sentence with a pointer here and to § 2.6.1.

**Two sites deliberately NOT struck.** `journal/0098` § "what this unlocks" and
`journal/0096` both state it, and the journal is **append-only** — an entry records what was
believed on the day it was written and rewriting it would destroy the very thing the journal is
for. `docs/audits/2026-07-25-roadmap-staleness-sweep.md` is likewise a dated snapshot.
**ROADMAP.md line ~2740 still carries it** (*"the head field is also what unlocks a
multi-flow-direction solve"*) and is owned by the integrator, not by the slice — flagged in the
slice's return rather than edited.

## 55. "The fluvial transport pass is what moves this world's sediment" (implicit in `material-behavior.md` § 13, `flow.md` § 8, and the Movement 2b brief — falsified 2026-07-26 by journal/0110's own probe)

**Never written as a sentence, which is why it survived.** `material-behavior.md`
§ 13 opens with the cycle *"weather → entrain → carry → sort → deposit"* and builds a
whole arc on making the **carry** step material-aware. `flow.md` § 8's acceptance test
is a cross-section reading *channel gravel → floodplain silt*. The Movement 2b brief
calls the slice *"the big appearance-changer."* All three take for granted that the
pass which entrains and routes suspended load is the pass that puts the sediment where
it ends up. Nobody stated it, so nobody checked it.

**Measured — seed 1337, `Extent::Medium`, 297,025 cells, 200 epochs
(`examples/facies_probe.rs`):**

| where this world's sediment goes | metres |
|---|---|
| picked up by the flow (entrained + incised) | **659.5** |
| weathered to regolith **in place** — never enters a load | **256,886** |
| moved by **hillslope creep** (diffusion) | **605,117** |

**Fluvial transport is 0.109 % of the sediment routing. Hillslope creep moves 918×
more.** Against a 440,595 m archive, everything the rivers ever touched is 0.15 % of
what the record holds.

**Two corroborating reads, from independent instruments.** (a) The largest competence
ceiling anywhere on the final epoch is **0.283**, against coarse clastic's settling
threshold of **0.840** — no cell on the shipped world can carry sand. (b) The world's
maximum transport capacity is **6.74 × 10⁻⁴**, **three times below** `energy_band`'s
own Low/Medium boundary of `0.002`, which the competence ceiling is anchored on. *(That
second read is the **final epoch only**; the full-run capacity distribution is
unmeasured, so `energy_band`'s "calibrated so trunk rivers read High" is **flagged, not
declared false** — see the slice's needs-measurement list.)*

**Why this matters more than the null it explains.** Movement 2b is correct and had
almost no effect, and without this number the natural diagnosis would have been "the
sorting rule needs tuning" — producing a constant chosen to manufacture an outcome out
of a thousandth of the sediment. The real reading is a **sequencing** one: the next
slice of this arc with a visible payoff is the **gravity/mass-wasting** member of
§ 13.2's transport family, not a refinement of the fluvial one. Creep is what moves
this world, and creep carries no identity.

**Lesson, and it is the third instance in two days.** journal/0109 twice mistook a
claim about *order* for a claim about *substance* ("head unlocks MFD"; "routing is
upstream of erosion, so the world will move"). This is the same error in a third
disguise: a claim about **naming** — *this is the transport pass, so it must be what
does the transporting* — read as a claim about **magnitude**. The defence is identical
and cheap: **before believing a mechanism matters, measure how much authority it has
over the thing you are claiming it changes.** One probe run.

## 56. "The deep-time engine's erosion rates are calibrated to the Phanerozoic register" (implicit in `earth-processes.md` § 3e-2 decision 5 beside `DeepConfig`'s rate constants — falsified 2026-07-26 by journal/0111)

**Never written as a sentence, and this time it could not have been checked from
inside.** § 3e-2 decision 5 (RATIFIED 2026-07-19, user) stipulates the **Phanerozoic
register**: *"the recorded span calibrates to ~500 Myr."* `DeepConfig::chapters` says
the same from another direction (*"K=8 gives Earth-orogeny-length chapters
(62.5 Myr)"*, and 8 × 62.5 = 500). Beside those sits a set of physical constants in
**metres per iteration** — `weathering: 0.02`, `k_bedrock: 0.0011`, `k_transport:
0.0016`, `diffusion: 0.12`, `uplift_scale: 3.0` — chosen by S9 so that *"orogenic belts
build hundreds of metres of net relief … over a few hundred iterations."* Nobody ever
divided one by the other. § 3e's own owed list still reads "calibrate iteration↔Myr
against a real orogen."

**Measured — seed 1337, `Extent::Medium`, 297,025 cells, 44,264 land cells, 200 epochs
(`examples/denudation_probe.rs`):**

| definition | m/Myr |
|---|---|
| **catchment-averaged denudation** (export from the land system / land area / time) | **0.0110** |
| bedrock erosion (incision + weathering-front descent, `grid.exhum`) | 0.0112 |
| rock uplift | 0.4095 |
| mean surface lowering | **−0.4084** (the land is *building*) |

Over the full 500 Myr the land system exports **5.48 m** of average thickness.

**Against the published record.** The global `10Be` outcrop median is **5.4 m/Myr**
(Portenga & Bierman 2011 *GSA Today*, n = 1599); stable-craton bedrock runs 1–10
(Bierman & Caffee 2001/2002; Veselovskiy et al. 2019); the Phanerozoic global
continental mean is 16 (Wilkinson & McElroy 2007). **The slowest surfaces ever
measured on Earth** — McMurdo Dry Valley bedrock and the hyperarid Atacama — sit at
**0.1–1 m/Myr** (Morgan et al. 2010 *JGR-ES*; Ritter et al. 2023 *JGR-ES*). This world
is **9× slower than that floor**, 91× below the craton band, 493× below the global
median. Real cratons strip **5–10 km** over a Phanerozoic span (Kola 3–5 km; Pilbara
multi-km in Paleozoic pulses, Morón et al. 2020; South African plateau ≥4.5 km since
130 Ma). This world strips **5.48 m** — about a thousandth.

**And it is not a quiet interior with active margins.** Across 44,264 land cells the
median bedrock erosion is 0.0104 m/Myr and **the single most active cell on the whole
world is 0.1341** — still slower than bare Antarctic bedrock. The distribution has
real structure (max/median 12.9×, top decile does 25.8 %) but the *entire* distribution
lies under the global floor. Denudation is **2.7 %** of rock uplift, so the landscape
has never approached topographic steady state.

~~**Two independent instruments agree.** The boundary-flux accounting carries a real
uncertainty — the ±35 m sea-level cycle shuffles cells across the shoreline, and the
land budget closes only to ~90 % of the export term. The per-cell rock-removal plane
(`grid.exhum`) has no shoreline in it at all. They agree **to 2.4 %**.~~

> **🔴 WITHDRAWN — see #60 (2026-07-26).** The two instruments agreed *"by coincidence of
> smallness"*: D1 is a **gross** land→sea edge flux and the ±35 m sinusoid sweeps the shoreline
> four times, so cover that crosses, is stranded, and crosses again is **counted every time**.
> D1 is an **upper bound**, not a measurement, and it becomes a loose one exactly when creep is
> fast. **A cross-check is only evidence if it survives the regime it is being used to license.**
> **What survives untouched:** the direction and the ~10³ magnitude of this entry's finding, and
> every literature comparison in it. D3 (`grid.exhum`) is the sound instrument once fluxes are
> large. *Struck here 2026-07-28, not 2026-07-26: #60 named this entry and this entry never
> named #60, so for two days `corrections.md` — the file whose whole purpose is recording
> falsified claims — carried an unstruck falsified claim 220 lines from its own withdrawal, with
> **no `#60` token anywhere in the file**. Found by an independent re-coding, not by a sweep.
> The repair is deliberately reciprocal.*

**Why the model is NOT wrong — only the rate.** A weathering-limited landscape routed
by hillslope creep, with minor rivers, regolith that armours its own weathering front,
and erosion mildly concentrated on steep ground, is a *textbook* low-relief craton.
Every qualitative statement journal/0110 made about this world holds. **The shape is
right and the clock is wrong**, which is exactly the failure mode an internal audit
cannot see: mass closes, goldens hold, passes are pure, and the simulation is perfectly
self-consistent at the wrong scale. **A closed system cannot detect its own scale
error.** It took an anchor from outside the corpus to see it.

**Corollary, falsified in the same run: `DeepOverrides::erosion_budget` is not the
erosion amplitude it is documented to be.** Its doc calls it *"the TERRAIN (erosion)
amplitude"* and says it lets *"the total amount of material erosion"* move. It scales
`weathering`, `k_transport` and `k_bedrock` — and **not `diffusion`**, which carries
**96 %** of this world's export. Measured response, production untouched:

| scenario | denudation m/Myr | vs production |
|---|---|---|
| production | 0.0110 | 1.0× |
| **erosion_budget 100×** | 0.0149 | **1.4×** |
| creep 10× only | 0.0181 | 1.7× |
| **budget 100× + creep 10×** | **1.4474** | **132×** |

**Neither lever pays alone and together they pay 59× more than their product.** The
coupling is the cover taper `exp(−H/H*)`, `H* = 3 m`: raise supply alone and the
regolith you make shields the rock that made it (export/bedrock-erosion falls
0.98 → 0.36, i.e. supply-limited → transport-limited); raise transport alone and there
is nothing to carry. journal/0108's shape a second time — **two levers that only pay
together**. Filed as stubs #24.

**Lesson.** journal/0109 twice mistook a claim about **order** for one about
**substance**; corrections #55 mistook a claim about **naming** for one about
**magnitude**. This is a claim about **units**. All three share a root — *a quantity
believed because it was written down* — but only this one was invisible to every
internal instrument the engine has. **Whenever a simulated quantity has a real
published counterpart, measure it against the literature at least once.** Those are the
only errors a perfect internal audit is structurally blind to.

**STILL OPEN after journal/0114, and the reason is worth more than a fix would have
been.** The calibration was built (`EROSION_CALIBRATION = 45`, one
`scale_erosion_rates`, a launch flag, a pinned fixed point) and **left switched off**.
Turned on it does move the number — catchment-averaged denudation 0.0110 → **0.4142
m/Myr**, bedrock erosion 0.0107 → **0.1271**, erosion's authority over the topography
(`D1/D4`) 0.027 → **0.91**, and the world leaves *"below every published terrestrial
band"* for the 0.1–1 floor band. It does **not** reach the 1–10 craton band, and **no
multiplier does**.

Two things sit under it, and neither is a constant:

* **A ceiling** (stubs #27). This entry diagnosed a coupling between two levers; the
  calibration measured a cap underneath *both*. Creep's flux limiter binds on ~89 % of
  the cells that have regolith to move *at the shipped rates already*, so the pass is a
  one-cell-per-epoch conveyor and raising `diffusion` cannot speed it up. Export ends up
  proportional to mean regolith thickness, so the band costs order a hundred metres of
  cover. **"Two levers that only pay together" is true and incomplete: they pay together
  up to a cap set by neither.**
* **A defect** (stubs #29). The incision clamp leaves deep closed depressions at any
  multiplier above 1× — 0 pits at 1×, **44 at 5×**, 148 at 45×, deepest 112 m — because
  four phases run *after* incision and can lower a cell past the floor it was clamped to.
  The shipped world scores zero only because it barely erodes. **That is this entry's own
  lesson one level down: a system that has stopped cannot detect its own logic errors
  either, because nothing exercises them.**

---

## 57. "The one lithology a deposit cannot be is basement" (`lithology.rs::Litho::as_deposited`, journal/0110 — falsified 2026-07-26 by journal/0112, and it had been false since the function was written)

`Litho::as_deposited` was introduced by Movement 2b to answer *"what is this rock once
a flow has carried it and set it down"*, and its doc stated the rule in the singular:

> *"Every recorded unit is a deposit — loose material that arrived — so **the one
> lithology that cannot be one** is `Litho::Basement`."*

The reasoning was right and the enumeration was incomplete. **Three more cannot be
one**, for the identical reason:

| species | why it cannot be a deposit |
|---|---|
| `OrganicPeat` | peat is **made where it lies** — a bog is not a delivery |
| `OrganicCoal` | coal is peat cooked **in place** (`material-behavior.md` § 12, category 2) |
| `OrganicCharcoal` | a **fire event**, and this enum's own doc calls it *"a thin event bed (capped at 0.04 m)"* |

A mover that picks any of them up is carrying **detrital organic matter**, and what it
sets down is carbonaceous mud with plant fragments in it — `Litho::OrganicSoil`, which
is exactly what that slot is for. It is not a peat bog, and it is emphatically not a
three-metre seam of charcoal.

**Measured, on the shipped world:** with hillslope creep carrying identity for the
first time, `charcoal_reaches_the_voxel_as_an_inclusion_never_as_a_stratum` found a
voxel that was **8/8 charcoal** against a cap of 0.04 m — 0.356 of an eighth. Thin fire
beds crept downslope, won the argmax at a low-deposition cell, and then **merged across
epochs under one mineral tag** into a stratum the cap exists to forbid.

**This is the genesis four-way test caught in the wild, and it is its first live
instance.** § 12's discriminator asks *"what did it come from, in the ontology we
intend to have?"*, and transported material answers **category 3 — "that material,
*moved*"**. An in-place organic is a **category 1/2 formation-or-transformation
product**. Recording a moved peat as a peat asserts that the peat *formed at the
receiving cell*, which is § 12's named pathology one row over: a unit filed under the
wrong category loses the edge, and with it the identity, mass and provenance chains
that edge would have carried. The fix keeps the record in category 3 and says so.

**The lesson, and it is the one worth carrying: the defect was surfaced by a
MAGNITUDE, not by a test.** The rule was equally wrong the day 2b shipped. Nothing
caught it, because the fluvial pass moves **0.109 %** of this world's sediment
(corrections #55) and a transported organic could never win a cell's mixture argmax at
that scale. Creep moves **918× more**, and the false claim became reachable within one
run. That is the same shape as corrections #51 (a guard that could not see the case)
and #55 (a claim nobody stated, so nobody checked it), with a new twist worth naming:

> **A rule can be wrong and unreachable at the same time, and "unreachable" is a
> property of the CURRENT magnitudes, not of the rule.** When a slice multiplies the
> throughput of a path by three orders of magnitude, every latent rule on that path
> becomes live at once. Re-read the enumerations that path depends on *before* trusting
> the suite — the suite only ever tested the reachable half.

**Fixed 2026-07-26 (journal/0112):** `as_deposited` remaps the three in-place organics
to `OrganicSoil`, and it now applies **only to the carried winner** rather than to the
tag's own default (a default was never carried anywhere). Asserted by name in
`tests/material_creep.rs::no_deposited_unit_claims_to_be_an_in_place_organic`, over the
whole record rather than at the site that found it — a stratum of charcoal is wrong
wherever it appears.

---

## 58. "`p → ∞` is single-receiver D8 **exactly**" (journal/0109, flow.md § 2.6.1, `DeepConfig::mfd_exponent` docs, 2026-07-25 — falsified 2026-07-26 by the repo's own test, while building hybrid `p`)

**The claim**, in three places and load-bearing in all of them, because it is what makes
the exponent a *knob* rather than a second model:

- journal/0109: *"`p → ∞` is single-receiver D8, **exactly**. The old solve is a limit of
  the new one, not a deleted alternative."*
- flow.md § 2.6.1: *"`p → ∞` is single-receiver D8 exactly, so the old solve is a *limit*
  of the new one rather than a deleted alternative."*
- `DeepConfig::mfd_exponent`: *"`p → ∞` recovers single-receiver D8 exactly."*

**Falsified: the limit is the steepest *SLOPE*; `route_cell` — the D8 rule the sentence
names — takes the steepest *DROP*.** They are not the same receiver. `partition_cell`
weights `(Δh / dₖ)^p`, dividing by the true flow-path length; `route_cell` compares
`filled[j]` and picks the lowest neighbour, with no `dₖ` anywhere. On a **diagonal** the
two differ by exactly the `√2` that the same slice introduced — and journal/0109's *own*
unit test pins a case where they disagree:

> `the_partition_follows_slope_not_drop` — a 3 m cardinal drop (slope 3.0) against a 4 m
> diagonal one (slope 2.83). The partition picks the **cardinal** (direction 3) at any
> `p`; the assertion `assert_eq!(route_cell(...), 8)` two lines below says D8 picks the
> **diagonal**. Raise `p` to infinity and the partition still picks the cardinal.

So the entry that introduced the correct treatment of flow-path length also wrote down a
limit claim that its own correction of D8 had just invalidated. **The two statements are
in the same document, forty lines apart.**

**Nothing shipped is wrong.** The limit the partition converges to is the *physically
better* one — a steepest-drop rule over-weights diagonals by `√2` and gives the drainage
net a systematic X-bias, which is exactly why the `dₖ` was added. What is wrong is the
word **"exactly"**, and it matters because that sentence was doing real work: it is the
argument that `mfd: false` lies *inside* the new model's family rather than beside it. It
does not. `mfd: false` is a **separate, pinned, byte-identical path** — which is why it
needs `the_single_receiver_path_still_hashes_to_the_pre_mfd_goldens` — and reading it as
"just `p = ∞`" would have been a licence to delete it.

**Mechanism, and why it survived a slice looking straight at it:** a claim about a *limit*
was checked against intuition ("a large exponent picks the steepest") instead of against
the code, inside the one slice whose whole contribution was noticing that *steepest* is
ambiguous. The correction is a single word; the family is the one journal/0109 named
against itself twice already — **a claim about shape smuggled in as a claim about
identity.** Struck at all three sites, and the doc comment on the test that falsifies it
now names this entry.

---

## 59. "The competence ceiling is fixed by an anchor that already ships, and is not a tuning knob" (`erosion.rs::COMPETENCE_SCALE` and `energy_band`, journal/0110, 2026-07-26 — falsified the same week by journal/0114, while calibrating the rate it was anchored to)

**The claim**, from `COMPETENCE_SCALE`'s own doc comment, and it is the argument that kept
Movement 2b's one new constant out of the tuning-knob category:

> *"Where the number comes from, and why it is not a tuning knob. It is fixed by an
> anchor that already ships: `energy_band` calls a capacity of `0.002` the Low/Medium
> boundary, and `litho_of_tag` turns exactly that boundary into the coarse/fine clastic
> split — so `0.002` is **already the world's stated 'energy at which sand stops
> moving'**. `settle_energy` puts the coarse-clastic reference sheet at ≈0.84, and
> 0.84 / 0.002 = 420."*

**The ratio is right and the FORM is wrong, and the difference only became visible when
something moved.** Transport capacity is `cap = k_transport · A^m · S^n`. A bare capacity
is therefore not a geomorphic quantity at all — it is the rate constant `k_transport`
multiplied by a **position in the drainage network**. `0.002` is not "the energy at which
sand stops moving"; it is `1.25 × k_transport`, and it read as a physical statement only
because `k_transport` had been `0.0016` since the day it was written and had never moved.
The same is true of the Medium/High boundary at `0.02` and, through the quoted division,
of `COMPETENCE_SCALE` itself.

**What that would have cost.** journal/0114's calibration multiplies `k_transport` along
with the other three erosion rates. Left absolute, the boundaries would have stayed put
while every capacity on the world rose by the multiplier, so essentially **every
depositional site would have classified as High energy** — `litho_of_tag` would have
recorded coarse clastic everywhere, and the competence ceiling would have risen far
enough to carry basement to the sea. The facies gradient the Movement 2b probes exist to
measure would have been **erased by the same commit that was supposed to make the world
erode**, and the erasure would have looked like a result: *"sand moves now."*

**The fix is to say what was always meant.** The thresholds are stated relative to a named
`REFERENCE_KT`, and `energy_band` / `competence_ceiling` take the world's own
`k_transport`, so they classify positions in the network rather than absolute rates. Every
ratio is formed as `k / REFERENCE_KT`, because `x / x` is exactly `1.0` in IEEE-754 — so
at the historical coefficient every threshold is bit-identical to the constant it
replaced, and the pre-calibration world still reproduces its goldens to the bit.

**Mechanism, and it is the family this repo keeps catching.** A constant was defended by
deriving it from *another constant in the same system* and calling that an anchor. It is
the closed-system error of journal/0111 one level down: not "the world was never checked
against the literature", but **"the constant was checked against a number that was itself
unchecked"**. A derivation is only an anchor if the thing it is derived from cannot move —
and `k_transport` was, at that moment, already named on the ROADMAP as due for
recalibration. The tell available at the time: the doc comment states the anchor in
*absolute capacity units* while the quantity it constrains is *defined* as a coefficient
times a dimensionless index. **When a threshold and the quantity it thresholds carry
different things inside them, one of them is holding a constant it does not own.**

## 60. "Two independent instruments agree on the denudation total to 2.4 %, so the number is cross-checked" (journal/0111 § 6, and propagated by the integrator into the merge commit and ROADMAP, 2026-07-26 — falsified the same day by journal/0114, by pushing the fluxes up)

**The claim**, and it is the sentence that made journal/0111's headline feel *confirmed*
rather than merely measured:

> *"Two independent instruments agree on the total: D1 (boundary-flux) and D3 (per-cell
> `grid.exhum`, no shoreline in it at all) agree to 2.4 %."*

I repeated it in the merge commit and treated it as a cross-check. It is not one.

**The mechanism.** D1's `creep_to_sea_m` is a **gross land→sea edge flux**, summed per
epoch. The paleo-sea-level sinusoid swings **±35 m and cycles four times** over the run,
so the shoreline sweeps back and forth across the same low-relief coastal cells. Cover
that creeps across the shoreline, is stranded by a falling stand, and creeps across again
is **counted every time it crosses**. D1 is therefore an **upper bound on export**, not a
measurement of it — and it becomes a *badly* loose one exactly when creep is fast.

**The arithmetic does not close, and that is the falsifier.** Under journal/0114's 45×
calibration: 63.6 m of bedrock removed + 39.2 m stored as regolith against **207.1 m of
claimed export**. Mass is not being lost; the export term is being multiply-counted.

**Why it looked like agreement.** At production rates the whole export is 5.48 m over
500 Myr and the double-count is a small fraction of a small number, so two instruments
with completely different failure modes landed within 2.4 % **by coincidence of
smallness**. The agreement was a property of the world being nearly static — the very fact
the measurement existed to report.

> **A cross-check between two instruments is only evidence if it survives the regime it is
> being used to license.** Ours was validated in the one regime where it could not
> discriminate, and then used to license conclusions about a regime 100× away.

**What survives, stated precisely so nobody over-corrects.** **D3 (per-cell `exhum`) is
the sound instrument once fluxes are large**; D1 remains useful as a bound. The direction
and the ~10³ magnitude of journal/0111's finding are **unaffected** — the world is still
far slower than any landscape measured on Earth, and every literature comparison in that
entry stands. What is withdrawn is only the claim that *two* instruments confirmed it.

**Heir:** a net (not gross) shoreline-export term, or a D1 that debits re-crossings. Until
then the denudation figures are **upper bounds** and should be written as such.

---

## 61. "The calibration preserves the landscape's shape — relief within 5 %" (journal/0114 § *The number that was not chosen*, criterion 1, **the binding criterion**; propagated by the integrator into the ROADMAP 2026-07-26 close block — falsified 2026-07-26 by the walk, journal/0115)

**The claim.** `EROSION_CALIBRATION = 45` was chosen against four criteria, and journal/0114
named the first as load-bearing: *"journal/0111's conclusion was 'the shape is right and the
clock is wrong', so a multiplier that moves the shape has stopped being a calibration and
become a redesign. Relief within 5 %: **+4.6 %** at 45×… **This is the binding criterion**,
and 45 is the largest measured row that clears it."*

**It is true and it does not mean what it was used to mean.** Measured on the same seed and
extent, deep-cell concavity — `mean(8 neighbours) − self`, the discrete Laplacian of the
surface:

| | mean | p10 | p50 | p90 | p99 | >1 m concave | >20 m concave |
|---|---|---|---|---|---|---|---|
| shipped | −0.14 m | −0.3 | −0.1 | +0.1 | +0.3 | **0.0 %** | **0.0 %** |
| calibrated | +0.73 m | **−50.8** | −0.1 | **+52.8** | **+106.3** | **35.8 %** | **23.2 %** |

> **Relief grew 4.6 %. Cell-to-cell roughness grew about 170×.**

The shipped world's *entire* concavity distribution fits inside ±0.3 m. The calibrated
world's tenth and ninetieth percentiles are −50.8 m and +52.8 m, with the **median
unchanged**. Nearly a quarter of land cells sit more than 20 m off the mean of their own
neighbours, tails symmetric. A landscape that becomes genuinely more rugged does so by
growing its **relief**; this one grew its **grid noise** while its shape stood still.

**The mechanism of the blindness, which is the transferable part.** Relief is
`max(surf) − min(surf)`: a **global extremal** statistic over ~44,000 cells. It is
mathematically incapable of detecting anything about the *spatial arrangement* between
those extremes. You can shuffle every interior cell of a heightfield and leave relief
exactly unchanged. So criterion 1 could not have failed for this reason **no matter how bad
the grid got** — it was not a weak test of shape, it was not a test of shape at all.

> **A criterion over a global aggregate cannot license a claim about local structure.**
> "The shape is preserved" is a claim about arrangement; relief, mean, min and max are
> claims about magnitude. Pair every aggregate criterion with a **neighbour-relative** one —
> a Laplacian, a gradient distribution, a spatial autocorrelation — or the acceptance test
> is measuring the axis that did not break.

**What survives.** Everything journal/0114 concluded about the *rates*: the transport
ceiling (stubs #27), export ∝ mean regolith thickness, the 100×-buys-1.6× measurement, and
the decision to ship the flag **off** — which this strengthens rather than weakens. What is
withdrawn is the licence criterion 1 was granting: **we did not know the shape was
preserved, and it was not.**

**Filed by the integrator against the integrator's own recording.** The close block
reproduced criterion 1 as settled evidence without asking what the statistic could see.

---

## 62. "148 pits (530 on production-Medium) measures the severity of the incision-clamp defect" (`stubs.md` #29, `tests/mfd_routing.rs::no_interior_cell_is_cut_below_all_of_its_neighbours`, the ROADMAP blocker entry, and the walk's own first probe — falsified 2026-07-26 **by the user, in flight**, journal/0115)

**The claim.** The clamp defect was sized by counting interior cells more than a metre below
**every one** of their eight neighbours: 148 on `mfd_routing`'s small fixture, and 530 when
the walk's tour map re-ran the identical census on production-Medium.

**The falsifier was a live read.** Flying the region around the deepest pit, the user
reported it *"absolutely pockmarked with similar pits — roughly every cell has a deep
depression. Honeycombed landscape."* That is irreconcilable with 530 of 44,090 cells.

**The mechanism: the census SATURATES.** "Below all eight neighbours" is a **winner-take-all**
predicate. It scores a cell only if its neighbours are *higher* — so as the defect spreads,
neighbouring cells sink too and **stop qualifying each other**. The count is maximised by
*isolated* pits and falls back toward zero exactly as the damage becomes universal. It is
structurally blind to the failure mode it was written to guard.

**The non-saturating instrument was already in the tree** — the router's own depression
fill, `filled[i] − routed[i]`, which measures the hollow at a cell regardless of what its
neighbours do (it is the quantity `DeepField::lake` thresholds at zero):

| | hollows >1 m | >10 m | >50 m | deepest | fill volume |
|---|---|---|---|---|---|
| shipped | **0** (0.0 %) | 0 | 0 | 0.0 m | 0.0 km³ |
| calibrated | **1,377** (3.1 %) | 817 (1.9 %) | 97 | 112.8 m | **5.4 km³** |

2.6× the saturating count, with a clean zero on the control. And **it still undercounts the
observation**, because it too only sees *closed* hollows — a bowl with a spillway scores
zero on both tests and looks identical from the air. Concavity (correction #61) is what
finally sized it: 23.2 % of land cells more than 20 m off their neighbours' mean. Regional
clustering accounts for the rest: **7.8 % closed hollows within 10 km of the station against
3.1 % globally**.

> **A guard built from a "more extreme than all of its neighbours" predicate degrades as the
> defect generalises.** It is a *ranking* test wearing a *magnitude* test's clothes. When the
> question is "how much of the world is broken", the predicate must be **absolute per cell**,
> never **relative to the cells that are also broken.**

**And note which instrument found it.** Two probes and a gated assertion agreed with each
other and were all wrong for the same structural reason; a person flying over the terrain
was right in one sentence. *The walk is not a formality after the measurement — here it was
the only instrument in the room that could see the question.*

**Heir:** `mfd_routing`'s guard should assert on **fill depth and concavity**, not on the
below-all-neighbours count, which cannot fail informatively. Tracked in `stubs.md` #29.
---

## 63. "The erosional solve above 1× is an explicit scheme past its numerical stability limit, and halving `myr_per_epoch` is the discriminator — the same register stubs #27's heir turns" (journal/0115 § *What this does to the blocker*, `stubs.md` #29, and the ROADMAP blocker entry, 2026-07-26 — **flagged as a hypothesis by its own author and falsified the same day** by journal/0116)

> **⚠ FALSIFIED 2026-07-29 — corrections #72, by journal/0122, which fixed the operator.**
> **It IS a stability limit; the 4× refinement was ~25× short of reaching it.** The pass is an
> explicit four-neighbour Laplacian, whose grid-scale mode decays only below a per-edge
> coefficient of **1/8**. The world's peak effective coefficient is **0.261 shipped and 12.60
> calibrated** — 2.1× and **100.8×** past that bound — so a 4× refinement left the calibrated
> arm still 25.2× past it. The experiment below is sound and its measurements all stand; the
> distance to the bound was simply never computed. **And section (ii)'s "opposite diagnoses"
> framing is backwards:** the *coefficient* makes the mode flip sign, the *limiter* caps the
> flip at the cell's inventory — so saturation is not evidence against instability, it is the
> reason the instability was a survivable finite-amplitude flip-flop instead of a blow-up.
> §§ (i) and (iii) are untouched. **Read #72 before acting on anything below.**

**The claim**, and journal/0115 deserves credit for the way it wrote it: *"The standing
hypothesis is now numerical, and it is **flagged as a hypothesis in the entry that carries
it**, because this session has already been burned twice by mechanisms that sounded right: an
explicit scheme run past its stability limit… If it is stability, the roughness collapses and
the landscape does not move. That second discriminator is the `cell_m / myr_per_epoch`
register — **the same one stubs #27's heir (b) turns**."*

**Three things in that are wrong, and the first is small but it is the tell.**

**(i) `myr_per_epoch` does not exist.** There is no such knob anywhere in the tree. The
register is `DeepConfig::iterations` (200) against a set of rates each stated *per iteration*,
with the epoch length living only in doc comments as "2.5 Myr/iteration". A discriminator
specified against a knob nobody has ever grepped for is a discriminator nobody has run.

> **⚠ NARROWED 2026-07-28** (independent re-coding, verified). **The load-bearing half stands:
> there is no `myr_per_epoch` KNOB** — no `DeepConfig` field, nothing tunable, and the
> discriminator specified against one was indeed unrunnable as written. **What is wrong is the
> absolute phrasing.** The *identifier* exists: `crates/dc-worldgen/examples/denudation_probe.rs`
> declares `fn myr_per_epoch(cfg: &DeepConfig) -> f64` at `:117`, documents it at `:105`, and
> calls it at `:417` — **in the probe this entry's own author was working with.** So *"does not
> exist anywhere in the tree"* is false of the string, and *"a knob nobody has ever grepped for"*
> is false twice over: it had been written, documented and called. *Kept as a narrowing rather
> than a correction because the diagnosis and the conclusion are both unaffected — but the tell
> this entry called "small but it is the tell" was itself imprecise, which is the same class of
> error one level down: **a claim about absence is a claim about a search, and the search was
> never stated.*** (Sibling of #67: a quotation needs a revision; **an absence needs its
> pathspec.**)

**(ii) The stability-limit hypothesis is falsified.** Refined **4×** at fixed total simulated
time — `k×` epochs against `1/k×` every per-epoch rate, with the shipped arm carried as the
operator's own falsifier and reproducing itself to three digits — concavity rms goes
**40.46 → 45.29 → 38.76 m**. A 4 % change under a 4× refinement, non-monotone, while the
landscape holds (relief +3.9 %, mean surface −0.3 %). A scheme past a CFL limit collapses
roughly with the step; this does not move. And the oscillation gets **purer** as the step
shrinks: lag-1 autocorrelation −0.867 → −0.909 → **−0.947**, with the full lag sequence
converging on the textbook alternation of a Nyquist mode.

**(iii) The shared register with stubs #27 is withdrawn.** They do not discharge together
through the clock.

**The mechanism of the error, and it is a family this repo has now caught three times.** The
argument was: *the limiter binds on 89–96 % of cells, so the operator has saturated, and a
saturated explicit operator overshoots.* Every clause is true and the conclusion does not
follow. **A flux limiter that caps export at the cover the cell actually has makes the
transfer a function of inventory, not of `rate × dt`** — so the saturated operator is
**time-step-independent**, which is the precise opposite of a CFL condition. Measured, on the
same ladder: the limiter's binding fraction is **96.0 % → 94.9 % → 94.7 %** across the 4×
refinement. It barely notices.

> **"The operator has saturated" and "the operator is unstable in time" are opposite
> diagnoses, and saturation is the evidence *against* the second one.** Saturation is what
> makes a scheme stop depending on its step size. The hypothesis took the single strongest
> piece of evidence that the step is irrelevant and read it as evidence that the step is the
> problem.

**What is actually true, and it came out of the same solves.** Split `surf = r + h` and the
two summands separate cleanly: the **bedrock converges** under refinement (concavity rms
23.77 → 12.59 → 5.88 m, ~`1/k`, autocorrelation back to −0.11) — so there *is* a genuine
time-step artefact in this world and D2 converged it away — while the **regolith sharpens**
(ACF −0.819 → −0.878 → −0.929, and roughness normalised by the cover it moves *grows*, 1.50 →
1.51 → 1.74). The flat surface total was **two defects cancelling**. The register is the
**flux limiter / donor-cell partition in `erosion.rs::diffuse`**.

**What survives, stated so nobody over-corrects.** journal/0115's *symptom* is entirely
intact and was confirmed by a sharper instrument: it is a checkerboard, in both axes, and its
own inference from "symmetric ±50 m tails with an unmoved median" was right. Its two
corrections (#61, #62) stand. What is withdrawn is only the mechanism it explicitly declined
to assert — **which is why this correction cost two probe runs instead of a fix slice.**

## 63b. (recorded here rather than as a separate number, because it never entered the corpus) "Isostasy is a positive feedback whose gain is set by local regolith excess"

Proposed mid-flight by the coordinator, explicitly labelled *"TREAT THIS AS A HYPOTHESIS…
I would rather be wrong here than have you find a way to agree with me"*, and never written
into a doc. It read `erosion.rs::isostasy` correctly — the Airy target *is* computed from
flexurally smoothed loads and differenced against the cell's own unsmoothed surface — and got
the **sign of the loop closure** backwards. The target is smooth and the cell is pulled
*toward* it: it is a low-pass filter.

Ablated on the calibrated arm, `iso_rate` 0.50 → 0.25 → 0.00 takes concavity rms **40.46 →
62.03 → 90.34 m** and closed hollows **1,377 → 2,150 → 13,012**. On the **shipped** world —
which has no defect — `iso_rate = 0` takes concavity rms 0.22 → 19.71 m and closed hollows
**0 → 6,215**. `corr(concavity, h − h̄)` is **−0.831**, the opposite sign to the predicted
feedback.

> **Isostasy is the only grid-scale damper in this solve. A slice that "fixed" it would have
> put six thousand holes in a world that currently has none.**

Recorded because the *near-miss* is the useful artefact: two mechanisms arrived hours apart,
both argued from the code, both specific, and they pointed in **opposite directions** — shrink
the step, versus stop a pass from running. Neither was caught by argument. Both were caught by
running the experiment on the arm nobody expected to be interesting.
## 64. "Converting `engine.rs`'s **two** step draws re-rolls every world's history layer: polities, sites, ruins" (`dc-sim/src/statistical/engine.rs:324-330` and `:354`, journal/0105 § "The three holes", ROADMAP Sequenced part (a), 2026-07-25 — falsified 2026-07-26 by a read-only trace taken while converting part (b))

**The claim**, which is the entire justification for part (a) being a user-owned
appearance slice rather than housekeeping:

> *"Two `dc-sim/engine.rs` draws are not on the provider. Their address is
> `[seed, k, SALT, r, t]` … converting them changes the key and re-rolls every world's
> history layer: polities, sites, ruins. That is a real appearance change and wants its
> own slice with the goldens re-baselined."*

**The load-bearing half is TRUE and should not be over-corrected.** The **region-step**
draw (`engine.rs:331`) does reach voxels a player can see, by a chain with no flag on it
anywhere: the collapsed pressure value is read at `pregen/history.rs:221`, a
`Value::Pressure(2)` opens the sack roll, `history.rs:237` sets `abandoned`, that survives
into `SiteSummary` (`history.rs:280-293`) and `Pregen.sites` (`pregen/mod.rs:302`), and
`collapse.rs:1588` gates ruin posts on exactly that flag — which `collapse.rs:483-488`
writes as `Block::Wood` inside `generate_chunk`, meshed at `dc-client/src/meshing.rs:214`.
The history pass is an unconditional member of `vanilla_passes()`
(`pipeline.rs:359-366`); there is no CLI flag and no config knob. **Ruins are in the
shipped world, standing up out of the ground, and re-addressing that draw moves them.**

**What is false is the word "two", and it halves the slice.** The **agent-step** draw
(`engine.rs:355`) re-rolls *nothing at all* in any world this project ships. The pregen
overlay is constructed with an empty agent roster — `history.rs:82` and `:84-88` both pass
`vec![]` as `ToyWorld::with_graph`'s third parameter, which is `agent_home`
(`dc-sim/src/statistical/world.rs:138`, doc'd at `:132-133`: *"Agents are optional — an
empty `agent_home` gives a pressure-field-only world"*). So `num_agents()` is 0
(`world.rs:175`), `build_scope`'s agent vector is empty (`engine.rs:183-185`), and the
loop at `engine.rs:342-363` **never executes** outside `dc-sim`'s own `s2_torture` /
`s2_measurements` suites. Converting that draw is byte-identical for every world and moves
only two dc-sim tests.

**"Polities" is false too, in a way worth stating precisely** because it is the noun that
makes the claim sound largest. The polity *count* is fixed at epoch 0 from slot count
(`history.rs:134-149`) and no draw touches it afterwards — no polity is ever founded,
merged or destroyed. `PolityExtent` facts do move (`history.rs:244-255`), but they are
written into `Pregen.ledger`, and **nothing in production reads the ledger**:
`Pregen.ledger` / `.overlay` / `.n_polities` / `.observe_count` (`pregen/mod.rs:300-304`)
have exactly one non-test reader between them, `approx_resident_bytes` at `mod.rs:367`.
That is a spines § 3 "built, and nothing calls it" cluster wearing an appearance claim's
clothes. Sites move for real — a sack removes a site from `polity_sites`
(`history.rs:238`), which changes the expansion frontier (`history.rs:158-181`) and hence
which slots are ever founded — but a site is *only* visible through its ruin posts.

**The mechanism of the error**, and it is the ordinary one: the sentence was written from
the *shape of the code* — two draws side by side in one loop, both addressed
`[seed, k, SALT, …]`, both feeding a subsystem named "history" — and never from a trace of
what a shipped world actually executes. It is A-2's neighbour: not a justification that
outlived its constraint, but one **assembled by symmetry** and never checked, exactly like
journal/0105's own pore-rider comment. The file even says the right thing in the right
place (`world.rs:132`) and no one followed the parameter.

**What this changes for part (a).** It is one draw, not two; one visible artifact class
(ruin posts) rather than a "history layer"; and the ledger half of the blast radius is
unread. It is still a **user-owned appearance change** and still wants the goldens
re-baselined — `contents_contract.rs:70-86`, `s7_walk.rs:32-40` and `geology.rs:32` all
hash `generate_chunk` blocks and are structurally downstream of the posts — plus
`s7_handoff.rs:118`, which pins a seed-specific sack and is the test most likely to break.
**The re-scoping is the integrator's and the user's call, not this correction's.**

---

## 65. "The deep-time phase ORDER falls out of the declared reads/writes, and this replaces a hand-declared canonical order" (`material-behavior.md` § 5 *Cadence: order × rate × window*, RECONCILED 2026-07-24; `journal/0090`; `north-star.md` § Passes — falsified 2026-07-26 by journal/0090's own summary paragraph, on the user's challenge)

**The claim.** §5 records the scheduler's ORDER axis as *"derived from `{reads, writes}` by
**topo-sort**; rejects cycles, conflicting writers, missing deps. **This *replaces* a
hand-declared 'canonical order'**: the order falls out of the declared dependencies and an
illegal schedule is **caught**, not trusted."* It was written as the reconciliation of the
user's 2026-07-23 fractional-phase sketch, whose ORDER half was an **authored** canonical
start order (*"tectonics → hydro → weathering"*).

**The refutation was already in the corpus, two sentences into the entry that celebrated
the replacement.** journal/0090:

> *"a genuinely linear relaxation pipeline (transport → weather → diffuse → …) does **not**
> fall out of a dataflow graph for free: the terrain is read, transformed, read again — and
> **you have to name each revision as a distinct resource for the topo-sort to reproduce a
> fixed sequence.**"*

**So the order does not fall out of the declarations. It is fed into them.**
`DeepAxis::{Forced, Incised, Weathered, Diffused, Compensated, Windblown, Settled}` are
seven synthetic resources whose only purpose is to encode the sequence someone already
chose. The topo-sort then "derives" that sequence — from an input constructed to produce
it. **It is the hand-declared canonical order, re-encoded in a form that makes the graph
appear to compute it**, and the re-encoding is what put the default pack's pass roster
inside an engine enum.

**What the argument actually established, and it is narrower than what it was used for.**
journal/0090's case is about **checkability**: *"an illegal schedule becomes a build-time
rejection instead of a silent bug."* True, and worth having — but it argues against
**unchecked** order, which nobody proposed. *Author-and-validate* is equally checkable: the
declarations validate the authored order instead of generating it. The argument was
answering a position that was not on the table.

> **A derivation whose inputs were constructed to produce the desired output is not a
> derivation. Ask what the mechanism would produce if you had NOT known the answer in
> advance** — here, nothing: without the revision chain the kernel rejects the schedule
> outright (`AmbiguousWriters`).

**And the process failure is the more transferable half.** The sketch was **user-originated**
and was recorded as *"carried forward to compare against the actual deep-sim loop and
discuss next session."* The comparison then happened **inside an implementation slice**, and
its ORDER half was superseded in a single clause of a design doc the user does not read.
The ratification protocol forbids recording *unratified assistant proposals*; it had nothing
to say about **an assistant reconciliation quietly superseding a ratified user design**.
That gap is what cost three days and produced `DeepAxis`. New rule in CLAUDE.md: a
user-originated design element may not be superseded by an implementation slice.

**What survives.** The pass-graph kernel, `reads`/`reads_prev` and the WAR/RAW distinction
(journal/0104), and every rejection class are all **kept** — they become the validator
rather than the generator. What is withdrawn is only the claim that the order is *derived*.

**Superseded by:** `ARCHITECTURE.md` § *The engine is plugin-agnostic, and pass ORDER is
authored* — DECIDED 2026-07-26 (user).

**⚠ SITE LIST AMENDED 2026-07-29 — this entry named THREE sites and there were FIVE.** The
baseline `doc-topology` sweep found two more, both unstruck for three days, both phrased
differently enough that a grep for the struck sentence never reached them. Both are now
struck in place:
- **`docs/design/geology.md` § *Backbone: everything is a pack*** — *"Declared reads/writes
  let the pipeline **topo-sort** passes and detect cycles — the coupling-order problem
  becomes a graph problem **instead of a hand-maintained list**."* That last clause is
  precisely the falsified half, in a design-backbone doc, written 2026-07-18. (Baseline S2/F1.)
- **`docs/design/material-behavior.md` § 5, the RATE bullet** — *"the RATE axis, **composed
  with** topo-sort, never replaced by it"*, twenty-three lines *below* the same section's
  own `🔴 SUPERSEDED` banner. Reported by the 2026-07-26 `doc-topology` sweep (its finding
  15), unfixed for three days while its two sibling bullets were fixed. (Baseline S3/F3.)
- **`docs/design/flow.md` § 11.1** carried *"ORDER (topo-sort)"* in a second, un-struck
  place; also fixed 2026-07-29.

*The lesson this adds to the entry: an enumeration of sites is itself an unchecked
enumeration. Strike by MEANING, not by string — and say in the correction which pathspec you
searched, so the next reader knows what your grep could not have caught.*

---

## 66. "The goldens will move and that is correct" / "`contents_contract`, `s7_walk` and `geology` all hash `generate_chunk` blocks and are **structurally downstream** of the posts" (ROADMAP Sequenced, the bootstrap-history removal entry; corrections #64's closing paragraph, 2026-07-26 — falsified 2026-07-28 by running the removal, journal/0121)

**The claim.** Two sentences, written two days apart, both saying the same thing:
the ruin posts are `generate_chunk` output, the byte-identity goldens hash
`generate_chunk` output, therefore deleting the posts re-baselines the goldens.
#64 named the files and even ranked them by risk: *"plus `s7_handoff.rs:118`,
which pins a seed-specific sack and is the test most likely to break."*

**Measured: NOT ONE GOLDEN MOVED.** The removal went in, and
`contents_contract::generated_world_is_byte_identical_to_the_pre_contract_goldens`,
`geology::{geology_world_regenerates_byte_identically,
class_registration_order_cannot_change_world_bytes}` and
`providers_golden::the_golden_world_still_hashes_to_the_pre_slice_goldens` all
passed **by name, unchanged**, against a tree that had just deleted 102
`Block::Wood` voxels from the shipped world.

**The mechanism, and it is arithmetic rather than architecture.** On
production-Medium (seed `0x0D5EED572026`) the pass abandoned **4 of 13 sites**,
and their posts landed in exactly **12 chunks**, at
`cx ∈ {−2805, −2804, −1840, −1839, −785, 234, 235}` with
`cz ∈ {−2871, −2870, −2805, −2804, −1336, −1335, −728, −727}` — the **nearest**
post to the origin is 727 chunks (≈ 20.9 km) off the `cz = 0` line. The samplers
walk fixed,
origin-clustered sets:

| sampler | chunk set | reaches a post-bearing chunk? |
|---|---|---|
| `contents_contract` | `(k·13 − 200, (k%7)·17 − 60)`, k<40 → `cx ∈ [−200, 307]`, `cz ∈ [−60, 42]` | no — measured directly on the *pre-removal* tree: **0 wood in the sample set** |
| `geology` | `(k·7 − 160, (k%5)·11 − 20)`, k<48 → `cx ∈ [−160, 169]`, `cz ∈ [−20, 24]` | no — `cz` cannot reach −727 |
| `providers_golden` | fingerprints the `DeepField`, not chunk blocks | no — upstream of the collapse tier entirely |
| `s7_walk` | the `z = 0` row, `cx ∈ [0, 9999]` — **and it is not a stored golden at all**: `sample_hashes` is compared against a *re-generation of the same world in the same run* | structurally cannot move |

Four cell interiors' worth of scattered posts, against samplers that hug the
origin and one axis. The two sets simply do not intersect.

**Why this is worth a number rather than a shrug.** "The goldens will move and
that is correct" reads like caution, and it is the opposite: it *pre-authorises* a
hash change, which is the one thing a byte-identity golden exists to make
expensive. Had the goldens moved for some **unrelated** reason — a sibling's stale
artifact, a merge that folded two changes — the prediction would have been sitting
there ready to absorb the movement as expected. **A pre-authorised golden move
cannot be distinguished from an unexplained one.** The honest form is a
*prediction with a mechanism*: which sampler, which chunks, why. That form is
falsifiable in advance, and this one would have been falsified in advance by two
lines of arithmetic on the sampler's own `for` loop.

**The mechanism of the error is #64's own, recurring inside #64.** That entry
exists to name *"a justification assembled by symmetry"* — a sentence generalised
over call sites that looked alike, found only by tracing what a shipped world
executes. Six lines after naming it, its closing paragraph does it again: the
inference *"hashes `generate_chunk` blocks" ⇒ "downstream of the posts" ⇒ "will
move"* is reasoning from the shape of the call graph, where the question was
**which chunks the sampler actually visits**. Structural downstreamness is
necessary and nowhere near sufficient.

**What was true.** `s7_handoff.rs` *was* the most affected test — it was deleted,
along with `s7_pregen::history_facts_are_causally_ordered` and four columns of
`s7_measurements`'s table. #64's *ranking* was right; its *class* of consequence
was wrong. A test that names the removed subject dies; a fingerprint over
unrelated ground does not notice.

**Transferable rule.** *Byte-identity is a regression detector, not a
specification* — so when a slice expects to move one, it owes **which fingerprint,
by what path, over which samples**, and the honest answer is sometimes "none of
them, and here is the arithmetic". Never a blanket licence.

---

## 67. "The 2026-07-22 seam audit misquoted the `exhum`/`t_crust` comment, so its A-2 was never live" (`journal/corrections.md` #40, `docs/design/stubs.md` § 4, `journal/0078`, 2026-07-23 — falsified 2026-07-28 by an independent re-coding of this file, git-verified)

**The claim.** #40 records that the seam inventory flagged the `exhum`/`t_crust` doc comment
as an A-2 instance by quoting it as *"the metamorphic-grade axes the collapse tier **reads**"*
— dropping a `WILL` and a *"currently consumed by nothing"* clause — and concludes:
*"A comment that already states 'consumed by nothing' is not a justification outliving its
premise; **the A-2 was never live**."* Its standing lesson: *"When a sweep reports 'the comment
says X and X is false', **re-read the comment before believing the sweep**."*

**Falsified: the audit did not paraphrase. It quoted the comment exactly as it stood the day
before.** `git show 11d4385 -- crates/dc-worldgen/src/deeptime/field.rs` removes the line

> `/// grade axes the collapse tier reads (§ 6.4). Empty when tectonic history is`

and replaces it with

> `/// grade axes the collapse tier WILL read (§ 6.4): exported and, as of U8,`
> `/// populated in every production world, but currently consumed by nothing.`

**And the commit says so in its own message:** *"**Fixed the lying field.rs doc-comments that
claimed the collapse tier "reads" these axes** (stubs.md section 4)."* `11d4385` is
**2026-07-21**; the seam inventory is **2026-07-22**. So the A-2 **was live** — from whenever
that comment was written until the day before the audit — and the commit that ended it calls
the prior text *lying*, which is the same verdict the audit reached.

**The mechanism, and it is the one #40 itself warned about, inverted.** #40's author read the
**current** comment, found it honest, and inferred that the audit must have mangled it. The
audit had quoted a **prior revision**. Neither party was careless about the text; they were
looking at the same file **at two different times**, and nothing in either artifact carried a
timestamp for the quotation. *#40 told the next reader to re-read the comment before believing
the sweep. The missing half is:* ***re-read it AS OF WHEN THE SWEEP RAN.*** A quotation without
a revision is not a quotation — the corpus already knows this about coordinates (#28: *"a bare
number pair is not an address"*) and it is the same defect one layer up.

**This is a stale READ, not a stale claim** — CLAUDE.md's cross-worktree hazard (#21/#27/#37)
appearing in the **document** layer, where there is no compiler to notice.

**Consequences, all propagation rather than code.**
- `stubs.md` § 4's parenthetical said *"corrections #40 records that the comment was already
  correct."* It was correct **as of 2026-07-21**, one day before the audit. Amended in place.
- **The `exhum`/`t_crust` A-2 is restored to the record as a real, since-fixed instance.** No
  code changes: `11d4385` already fixed it, and journal/0078's tightening still stands as polish.
- `spines.md` A-2's *"not-an-instance"* row is the same reading and is amended with a pointer.

**Two lessons, and the second is worth more than this entry.**
1. **A quotation needs a revision, not just a source.** Every `file:line` citation in this corpus
   is implicitly *"as of some unstated commit"*, and 808 of them exist.
2. ***A correction can be wrong, and nothing in this project was checking.*** This is the first
   entry filed against another entry. It was found by an **independent re-coding dispatched
   specifically to disagree**, told which two files it was forbidden to read so it could not
   anchor — not by any sweep, and not by the author of either artifact. **`corrections.md` had
   no more immunity than the documents it audits.**

## 68. "journal/0111's recalibration invalidates a whole cohort of pre-0111 observations" (assistant, 2026-07-28 — written into `staleness-sweep/SKILL.md` and falsified the same afternoon by the first agent run under it)

**Claimed** (`.claude/skills/staleness-sweep/SKILL.md`, the section headed *"THE RECALIBRATION
TRAP — the highest-value thing this sweep can catch"*): that the ~1000× denudation
recalibration means *"every Observed entry recorded before that about a magnitude, a rate, or
'system X doesn't seem to matter here' may be **an artifact of the wrong scale rather than a
real defect**"*, and that a whole cohort can be invalidated by one commit. The claim was then
**written into a dispatch brief** for the baseline sweep's § Observed reader.

**Falsified by that reader, same day.** It classified all **129** Observed entries and found
**exactly one** 0111-sensitive. The mechanism it returned:

> **`calibrated_rates` was built, measured, and left OFF.** Production ships it false —
> `crates/dc-worldgen/examples/walk_tour_0115.rs:150` asserts *"production must still ship
> `calibrated_rates` OFF"*; `denudation_probe.rs:131` says `Some(true)` is *"**not** what
> production"* uses; `dc-client` enables it only behind an explicit `--calibrated-rates` flag.
> **The shipped world still runs at the old rate**, so observations *of the shipped world* were
> never artifacts of a calibration nobody enabled.

**The corrected rule:** a recalibration voids prior observations **only if it is ENABLED in the
config those observations were made under.** A constant behind an off-by-default flag changes
nothing about what anyone saw. Both the skill and `scripts/sweep_due_hook.py` now carry the
qualifier.

**And the distinction the episode forced, which is the part worth keeping.** The same sweep
produced one finding of each kind, needing **opposite** verdicts:
- a **METHODOLOGICAL** defect invalidates a null **regardless of calibration** —
  `journal/0079`'s erosion null came from a probe blind to `diffusion`, the term doing 96 % of
  export, so *"nothing further to ratify on the erosion axis"* is unsound today;
- a **MAGNITUDE** claim only moves when the magnitude actually moved — and it did not.

Collapsing those two turns "the erosion block is stale" into the *wrong* stronger claim *"the
landscape is supply-limited today"*, which holds only of the calibrated world.

**Why it is filed rather than quietly edited:** *an agent's MECHANISM is a hypothesis; only its
numbers are evidence* has always been aimed at reports coming **back**. This is the mirror —
**the hypothesis was in the brief going OUT**, asserted by the integrator, from a skill shipped
hours earlier. The agent measured and said no. **A brief is an artifact that can be wrong**, and
briefing a hypothesis *as* a hypothesis is what made the disagreement possible.

## 69. "The general registry is deliberately unbuilt — four conversions is not enough to design one from" (`stubs.md:22`, ~2026-07-22 — withdrawn by the user 2026-07-28; there was never a registry)

**Claimed**, and quoted as **ratified project doctrine** in at least six places
(`doc-topology/SKILL.md`, `corpus-knowledge-notebook.md` ×2, `corpus-knowledge-evidence.md`,
`ROADMAP.md` ×3): that a *general provider registry* exists as deferred future work, gated on a
conversion count. It was the standing argument for **not designing the corpus
knowledge/addressability layer** — a live, user-owned design thread.

**Falsified by the user**, reading the clause: *"I honestly don't understand what the registry is
supposed to be except for a list which we can extend."* **Correct — and nobody had ever proposed
one.** The clause was an *inference* that hardened into doctrine.

**Mechanism — one ambiguous sentence.** `north-star.md:189-191` described material behavior slots
as *"the **`Providers` pattern** (`Option<fn>` slots with an identity fallback) **generalized from
world-level to material-level**."* True of the **code shape**. Read as the world-level provider
**system** being promoted into the SDK — at which point a "general registry" is the obvious next
inference. **"Slot" was doing two unrelated jobs:**
- **provider seams** (world-level, `deeptime/providers/`) — holes for systems that do not exist
  yet. **Scaffolding, and their success condition is to DISAPPEAR.**
- **material behavior slots** (`north-star.md` § Materials) — what a content author writes. **The
  SDK surface. Permanent.**

**The evidence that seams are meant to vanish was in the code the whole time.** Of six slots ever
created, one has had its heir arrive: `burial_temp_c`'s heir turned out to be a *field*, so it
**retired as a field pass and left the file** (journal/0093). `providers/mod.rs` states the lesson
outright — *"the answer was 'this is not a provider at all — it is a field.'"* `depth_to_water` is
documented as heading the same way.

**Not a defect in anything shipped** — no code changed, extending the list was always cheap. It
cost a live design thread a phantom veto, and it is filed because **the corpus implied work nobody
had asked for**, which is the documents-side of *existence is not standing*.

**⚠ The assistant then got the withdrawal wrong TWICE**, in two files, hours apart, both times
justifying it as *"`stubs.md:22` is about plugin-authorable engine sockets"* — the same misreading
of the same north-star sentence, committed **while correcting it**. Left visible in
`doc-topology/SKILL.md`. **Being wrong twice from one sentence is the argument for disambiguating
the sentence**, which `north-star.md` now does in place.

## 70. "Ore must be EXPOSED by erosion to be legible, so exhumed-core ore is illegible until the erosion calibration lands" (`ores.md`, 2026-07-21 — rejected by the user 2026-07-28)

**Claimed**, threaded through `ores.md` as measurement caveats: because a spike measured
exhumation at **metre-scale**, exhumed-core signals are *"illegible at any amplitude"*, the
`exhum` gate *"may barely discriminate"*, and the **lode-gold A/B fork must wait** *"until the
erosion-supply calibration lands."*

**Rejected by the user:**

> *"We do not need to have ore 'exposed' — the default plugin pack will ship a voxel game **with
> digging**, which spiritually inherits from Minecraft. Absolutely no reason to treat it like
> everything needs to be discoverable on the surface. Weird and misconceived and likely very
> relatively old."*

**Mechanism: an unstated premise riding inside measurements.** The numbers were never wrong —
exhumation *is* metre-scale. What was wrong is the buried assumption that **surface discoverability
is a precondition for shipping an ore.** In a game whose core verb is *digging*, **depth is the
feature.** The premise was never written as a design decision, so nothing ever pointed at it.

**What it cost:** a **user-owned** `NEEDS RATIFICATION` fork sat closed for a week on an
assistant-side caveat, and an assistant later offered to run `probe 3` to resolve it — a
measurement of the wrong quantity. Withdrawn.

**What survives:** `exhum`/`t_crust` remain real and useful for **genesis honesty** — where an ore
forms, under what pressure–temperature history. Only *exposure-as-precondition* dies.

**The transferable lesson, and the reason this is filed:** ***a caveat is where an unexamined
premise hides.*** A caveat reads as **evidence** — it cites a measurement — so reviewers check the
number and never the claim wrapped around it. All three sweeps (`spine-audit`, `doc-topology`,
staleness) compare **stated claims**; none of them look inside a caveat for a premise that was
never stated. `ores.md` is now marked as conceptually behind `materials.md` /
`material-behavior.md`, which win on disagreement.

## 71. "The erosion axis is engine work" (`ROADMAP.md`, 2026-07-28 evening close block — corrected by the user 2026-07-29)

**Claimed**, in the close block's *First things next session* item 2: *"The erosion axis is
marked settled and is not … **This is engine work and it is the highest-value thing on the
board.**"*

**Falsified by read-first item 0, by name.** `CLAUDE.md`'s first paragraph — the north-star
summary every session and every agent loads — reads *"**Every pass is content, including
tectonics and erosion**; pass ORDER is authored per world (ARCHITECTURE.md, DECIDED
2026-07-26)."* Erosion is **default-plugin-pack work**, not engine work. Ruled by the user
2026-07-29. The *priority* half of the claim is untouched: the erosion axis is still the
highest-value thing on the board.

**Mechanism — and it is the second instance of #68's, one layer up.** #68 recorded that *the
newer the source, the more explicitly it must be briefed as a hypothesis*, **within a single
session**: a claim written that afternoon was inherited as settled by the next agent. **This is
the same mechanism SOURCE-vs-SOURCE.** A cold session read the close block — the newest thing
in the corpus, and the thing written to be read first — and repeated its side of a question
`CLAUDE.md` had already ruled, **while holding `CLAUDE.md` in context**. Recency beat
authority; the two documents were never put side by side. The close block is structurally the
**most likely artifact to be wrong** (written last, unreviewed, at the end of a long session)
and structurally the **most likely to be believed** (it is what the next session opens).

**The transferable rule: a close block is a HANDOFF, not an authority.** It may summarise,
sequence and prioritise. It may not silently rule on anything read-first already settles, and
where it does, read-first wins. This is filed as a correction rather than repaired quietly
because a close block that can overturn `CLAUDE.md` without anyone noticing makes `CLAUDE.md`
unciteable — and the cost is paid by whoever reads the close block next, which is *always* a
cold session with no way to know.

**Where the fix lives.** Struck in place at `ROADMAP.md` § *First things next session* item 2,
with the ruling and this number beside it; the priority claim is left standing. **No other site
carried the miscoding** — pathspec:
`grep -rniE "erosion[^.]{0,40}engine work|engine work[^.]{0,40}erosion"` over the repo returns
only that clause.


## 72. "The stability-limit story is falsified — this is not an explicit scheme past its numerical stability limit" (`journal/corrections.md` #63 (ii), journal/0116 § D2, `stubs.md` #29, the ROADMAP blocker entry, 2026-07-26 — falsified 2026-07-29 by journal/0122, which fixed the operator)

**The claim.** journal/0116 refined the time step **4×** at fixed total simulated time and
found the concavity rms moving 40.46 → 45.29 → 38.76 m — 4 % under a 4× refinement,
non-monotone — while the checkerboard got *purer*. It concluded, and #63 recorded, that *"a
scheme past a CFL limit collapses roughly with the step; this does not move"*, and that the
defect is therefore structural rather than numerical.

**It is numerical, and the refinement was ~25× too small to see it.**

The hillslope pass is an explicit four-neighbour Laplacian. Its von Neumann amplification
factor is `g(k) = 1 − 2a(2 − cos k_x − cos k_y)` for a per-edge coefficient `a`, so the
grid-scale (Nyquist) mode is **reflected with its amplitude intact at `a = 1/4`** and **decays
monotonically only below `a = 1/8`**. The world's actual coefficients, measured:

| | `diffusion` | peak `eff_diff` (with lithic susceptibility) | multiple of the 1/8 bound |
|---|---|---|---|
| shipped | 0.12 | **0.261** | 2.1× |
| calibrated (45×) | 5.4 | **12.60** | **100.8×** |

A 4× refinement takes the calibrated arm from 100.8× past the bound to **25.2× past it**. Of
course nothing collapsed. **The experiment was sound, well-controlled, and an order of
magnitude short of its own register** — and nothing in it could have said so, because the
distance to the bound was never computed. The bound is two lines of arithmetic on a constant
that was sitting in the config.

**And the two diagnoses #63 called opposites are the same mechanism, composed.** #63's
sharpest line was:

> *"'The operator has saturated' and 'the operator is unstable in time' are opposite
> diagnoses, and saturation is the evidence against the second one."*

They are not opposites; they are the two halves of one behaviour, and each explains what the
other could not:

- the **coefficient** decides that the grid-scale mode changes sign every step — that is the
  instability, and it is why the field is a checkerboard;
- the **limiter** decides how big the flip is: capping export at the cell's whole inventory
  turns what would be an exponential blow-up into a **finite-amplitude period-2 limit cycle**
  whose amplitude is set by the cover, not by the rate.

So the limiter is exactly why the defect is deaf to `dt` (#63's real finding, and it stands)
**and** exactly why it never blew up to infinity and got noticed years ago. Saturation was not
evidence against instability; it was the reason the instability was survivable enough to ship.

**Isolated, not argued** (`erosion.rs::hillslope_operator_tests`): flat bedrock, a
checkerboard of cover, the creep pass and nothing else, at the calibrated rate. The two
populations swap **to the bit, forever**, with the amplitude conserved to 1e-9 — the exact
period-2 mode journal/0116 hypothesised and explicitly declined to promote. At the shipped
rate on the same fixture the same initial condition decays monotonically.

**What survives, stated so nobody over-corrects.** journal/0116's measurements are all intact
and none is withdrawn — the checkerboard (D1), isostasy as the damper rather than the driver
(D3), the bedrock/regolith split, the limiter's binding fractions, and the qualification that
*saturation alone is not sufficient* (which is now derived rather than observed: the shipped
world saturates at 88.7 % and sits only 2.1× past the bound, so its flip is small and its
cover is thin). Its **register is also correct** — the fix is in the flux limiter / donor-cell
partition in `erosion.rs::diffuse`, exactly where it said. What is withdrawn is one inference:
that a null from a 4× refinement rules out a stability limit.

> **A refinement experiment measures nothing unless you know how far the thing you are
> refining has to travel.** "We refined by 4× and it did not collapse" is a statement about
> the *number 4*, not about the scheme, until it is put beside the distance to the bound. This
> is the same shape as [[measure-against-the-literature]] one level in: the sim was checked
> against itself — against its own value at a different step — when the answer was a closed-form
> property of the stencil that no amount of internal refinement could reveal.

**Where the fix lives.** `erosion.rs::CREEP_MAX_EDGE_COEFF` and the sub-cycled
`Erosion::diffuse`; journal/0122. Banners stamped in the same commit on #63 above,
`journal/0116`, `docs/design/stubs.md` § 29 and the ROADMAP blocker entry — the four sites
that carry the falsified inference.

## 73. "The 460 m-vs-28.8 m checkerboard question has never been probed" (the member-#0 design pass, 2026-07-29 — propagated into a banner and a ROADMAP entry by the main session the same hour; falsified the same evening by the user's memory)

**The claim.** The member-#0 design agent, reading the 2026-07-24 palette-quant audit,
reported its § 7 INFERRED question — are U3's visible squares the 460 m deep-cell tiles or
the 28.8 m member-fitness stepping? — as *"never probed"*, five days old, and load-bearing
for U3's acceptance test. The main session accepted it, stamped it into the audit's first
staleness banner ("sat unprobed for five days"), and filed a probe-first item in
ROADMAP § Observed.

**The falsification.** The user: *"i already believed palette quant was from member fitness
stepping... that conversation is what lead to the whole octaves and facies driver
planning."* Checked against the record: the 2026-07-24 close block says outright
*"the palette-quant 460-vs-28.8 was settled — member stepping"*, resting on primary
evidence that was in the corpus all along — corrections **#45** (the member dither is
world-anchored and C0-continuous; the defect is **single-octave**) and the same-day
haunting diagnosis (*"the member squares are one octave of value noise at chunk
wavelength — fix = octaves, not resolution"*). The question was answered **the same day it
was asked**.

**The mechanism, and it is the sharpest instance yet of the one-directional pointer.** The
resolution lived in a close block and nowhere else; the audit that asked the question never
received it. A close block is a handoff, not an authority (#71) — which cuts BOTH ways:
nobody re-reading the audit could find the answer, and nobody should have had to trust the
close block if they had. Read-first item 5's rule (the correction's author stamps the
target, same commit) was written for exactly this, eight months of subjective time before
it happened again: **an answer that is not stamped onto its question will be re-derived,
and the re-derivation will be believed** — this time by two readers in sequence, an agent
reading the audit and a main session reading the agent, hours after that same main session
had corrected an identical shape (the "unprobed" phrasing) in a different file.

**Repairs, same evening:** the audit's banner now carries the answer and the correction's
history; the ROADMAP entry converted from probe-first to settled-with-consequence (the
near-path fix alone will not clear the checkerboard; the octaves source is a co-requisite);
the octaves lineage is stitched across its 2026-07-24 origin and both 2026-07-29
re-derivations. What was NOT wrong: the design agent's *consequence* — a perfect near fix
leaves the checkerboard on screen — survives falsification of its premise, strengthened.

## 74. "Verified at those exact lines; the adoption did not move them" (`docs/spines.md` CoarseField row, 2026-07-29 — falsified by the very commit that carried it; caught by the same day's FULL spine-audit)

The claim rode the member-#0 far-slice merge (`0dcdadb`) — and that commit itself moved
`sample_dithered` 479→498 and `summarize` 515→534, via the doc paragraph the slice added to
`coarse.rs` *because the type's old prose had misled its first adopter*. So the row asserted
citation accuracy about a tree its own commit was changing, and was wrong the moment it
landed.

**Mechanism:** a citation-accuracy claim is a claim about the tree you COMMIT, not the tree
you read. Anything that verifies line numbers must re-verify after the same change-set's
edits are complete — or cite by symbol, which cannot rot this way. This is corrections #67's
citation-rot class compressed to zero days, and the enumerate-what-you-touched failure shape
(the writer enumerated the files *checked*, not the files *the commit touched*).

**Corrected in place** by the 2026-07-29 FULL spine-audit (refs refreshed, the row's claim
rewritten); recorded here because the shape — *a self-refuting same-commit claim* — will
recur wherever a doc asserts freshness inside the change that stales it.

## 75. "The face-pairing rule is under-specified, not ratified — a live blocker on the tier's first slice" (`refinement.md` § 8, the coupling-priors audit § 5 item 6, `dependency-graph.md` E5 row — all 2026-07-29; stale AT ASSERTION: the ratification had existed since 2026-07-25, 360 lines below the flag every one of them cited)

`flow.md` § 2.2's flag (written with S19, 2026-07-25) asked the slot-pairing question and
named a candidate rule "NOT RATIFIED." **The same file's § 11.5, written later the same
day, ratified the principle** (pair by what confines the flow), and `flux.rs` shipped
chapter pairing for lateral free flux with the bound residual explicitly deferred. The flag
was never stamped. Three 2026-07-29 documents then read the unstamped flag and propagated
"unratified" — the coupling-priors agent even listed the § 2.2-vs-§ 11.5 pair as a noticed
contradiction (§ 5 item 6) *without resolving which side was current*, and `refinement.md`
§ 8 turned it into a build-order blocker.

**The honest half:** the BOUND mode genuinely was — and remains — unforced (no entry
carries `FlowForm::Bound`), and the conduit mode is real and assigned to continuation (c).
The claim was wrong only about the mode the channel operator actually needs.

**Mechanism:** a one-directional pointer inside ONE FILE — the founding doc-topology
pattern (corrections #65's claim-and-refutation 400 lines apart) at 360 lines. The flag was
the natural landing point for every later reader; the ratification lived below where no
grep for "face-pairing" reaches (§ 11.5 says "pairing," not "face-pairing").

**Resolution:** re-argued fresh against the channel operator at the user's direction
(re-opened rather than inherited) and **CAUTIOUSLY RATIFIED 2026-07-29 as the three-mode
confinement rule** — record in `flow.md` § 11.5's banner, with three sharpenings
(eroding-cell attachment; window-vs-pairing; the 2× is gross-vs-net). All five sites
stamped in the same commit as this entry: `flow.md` § 2.2, `refinement.md` § 8, the
coupling-priors header, `dependency-graph.md` E5, `S19-flow-record-cost-results.md`.

## 76. "Per chunk enumerate touched deep cells (≤ 9, typically 1)" (the member-#0 design pass § 2b, 2026-07-29 — refuted by arithmetic during the build the same day, journal/0129)

The bilinear membership stencil is **always 2×2**, so a chunk interior to a deep cell
touches **4** cells, not 1; with 1024 columns drawing per chunk, even a 0.001 weight is
realised somewhere. "Typically 1" is true only within about half a metre of a cell-centre
line. Consequence: the near-path restructure costs **~4× `run_strata` per chunk**, not ~1×
— a 4× on a hot path (chunk load), mispriced in the plan that sequenced it.

**Mechanism — the third outfit of the same misreading in five days:** *"away from a
boundary one weight ≈ 1"* reads as *"the stencil usually collapses to one cell"*, and it
never does — the weight is large, the SUPPORT is still four cells. First outfit: the
`coarse.rs` doc comment that cost journal/0125 a failing gate. Second: the cell-wide-blend
surprise (E[w_home] = 9/16, ~44 % of far voxels reading a neighbour). Third: this. A
bilinear stencil's support is a *set*, not a winner.

**Same document, same lesson, second claim:** § 2b's blast radius ("six functions") was a
**lower bound read as a total** — measured at build time as **13 files / ~40 sites**,
because the coupling was a `pub` *field*, invisible to a trace of producing code. *Grep
the field, not the function.* Both targets stamped (the member-#0 audit header; the graph
E5 row already carried the corrected numbers at merge).
## 77. "The foot-float defect is sub-perceptual, so it is a design decision rather than a fire" (2026-07-29, the integrator's own, falsified within the hour by the user)

**The claim.** Having driven the two-body-plan walk (journal/0130) and read four
screenshots, the integrator reported the measured 20 mm foot float as **not visible
by eye**, and drew a priority conclusion from it: *"the defect is real, measured, and
currently sub-perceptual. That argues it's a design decision to make deliberately
rather than a fire to fight."*

**Falsified by the user, immediately, from having watched the body move:**

> *"the hover looks bad. i do not like the body hover, the body is oscillating gently
> up and down. it's not a bob, it's a hover, because as you said, the feet don't touch
> the ground… note the defects are sub-perceptual to claude who can only look at
> screenshots, but human can see the problems jumping out and just hasn't bothered with
> it because there's a lot else going on in this repo."*

**Mechanism — a still frame is structurally blind to a temporal artifact.** The float
is near-constant *within* any single frame, so a screenshot carries the offset and
**cannot carry the oscillation**. The visible defect is the oscillation. No number of
frames fixes this; it is the medium, not the sample size.

**The number was already in hand and was misread.** The probe reported `idle` sole
height as the range **`[+0.020, +0.035] m`**. That is a **15 mm amplitude** with the
feet never planted — the hover, quantified. The integrator read a range as a
*tolerance* (an error bar on a static offset) rather than as an *amplitude* (a time
series), and then let the blind instrument overrule the sighted one.

**Why this is not merely a repeat of #18/#19.** Those established *pick the control
that can SEE your question* for the **lighting mode** — lit for shape, fullbright for
material — and that discipline was followed correctly here (the walk used the lit pass
precisely because fullbright flattens body cuboids). The unasked question was whether
the **medium** could see it. **For a motion question the screenshot is the blind
control, and its null proves nothing.** The instrument that could see it existed in
the same report: a per-frame trace of the quantity.

**Corollary, and it is the durable half:** the user's live view is senior to the
integrator's screenshot read — already doctrine — but the *reason* matters, because it
tells you when to insist. It is senior **specifically for anything temporal**:
oscillation, skate, pop, jitter, easing, stutter. Where the question is motion, do not
report a visual verdict from frames at all; report the trace, or record the screen.

**Stamped in the same commit** (read-first item 5): journal/0130 § "The walk, and the
instrument that could not see the defect" carries the pointer back here.

## 78. "Foot-placement IK has never engaged — the path is dead in the common case, for any body plan" (journal/0130 + the ROADMAP Observed line it shipped, 2026-07-29 — falsified the same evening by the very next slice, journal/0131)

**The claim.** journal/0130 measured 176/176 sampled frames beyond the leg's reach and
concluded that foot-placement IK *"has never once engaged"*, that *"nobody has ever seen
this rig bend a joint under IK"*, and — hedged, but still wrong — that the path is
*"dead in the common case."* The integrator merged it.

**Falsified: the stock biped's IK has ALWAYS solved while CROUCHING** — 88/88 samples,
knee bending to 123.7°, on the unmodified `dc:body/biped`, since the day it shipped.
journal/0130's finding is true of **standing only**.

**Mechanism — verified independently by arithmetic on main's own constants, not taken
from the report.** `CROUCH_ROOT_DROP_M = 0.45` (`dc-client/src/body.rs:58`) drops the
rendered hip from **0.900 m** to **0.450 m**, against a leg reach of **0.880 m**
(0.45 + 0.43). Standing, the target at y=0 needs a 0.900 m span and is unreachable by
20 mm. Crouching, it needs 0.450 m and is reachable **with 0.430 m to spare** — deep
inside the annulus, which is why the knee folds so hard.

**The defect is a SWEEP, not a solver.** journal/0130's probe swept three axes
exhaustively — clip × plan × foot — and held a fourth **fixed at its default**.
`Posture` has exactly **two** variants, and the non-default one **moves the hip by half
the body**. A claim quantified over *"any body plan"* was silently also quantified over
*one posture*, and nothing in the itemisation could show it: the totals closed, every
assertion was a derivation, and the count was 100% of what was actually sampled.

**This is the same shape as #77, one level up, in the same session.** #77 was *the
instrument could not see the axis* (a still frame cannot see time). #78 is *the sweep did
not include the axis* (the probe never varied posture). **Both are an unswept axis
reported as a property of the system**, and neither is detectable from inside the
measurement — the itemisation is complete over the axes it has. It is the null-reading
corollary of CLAUDE.md § *"a closed system cannot detect its own scale error"* — *before
concluding "system X does nothing here", check that the conditions for it to do anything
were ever in the sample.*

**Two further things the wider sweep found, both of which sharpen the user call rather
than answer it:**
- **A FOURTH absolute-metres constant.** `CROUCH_ROOT_DROP_M` is **50.0 %** of the
  biped's hip height and **97.8 %** of the stout's — a crouching stout has a **0.010 m**
  hip and a degenerate **180°** knee. The banner journal/0130 wrote on `bodies.md` § IK
  named **three** such constants; it was short by one, and is corrected in place with the
  original left as dated testimony.
- **The half-voxel correction window can never admit a terrain step — 2:1 by
  construction at every scale N**, since the window is *defined* as half a voxel and the
  smallest real relief is one voxel. So foot placement structurally only ever sees
  *sub-voxel* offsets, which today are produced by nothing except a plan's own hip/reach
  mismatch. Now an assertion rather than an observation.

**Stamped in the same commit** (read-first item 5, the writer of the correction stamps
its target): journal/0130 carries a banner at its head pointing here, and the ROADMAP
Observed line it shipped is corrected in place.

## 79. "The block-tier collider staying binary is the accepted, documented visible mismatch" (`crates/dc-client/src/meshing.rs:57-58`, citing `visuals.md` — falsified 2026-08-01 by the user, against the very doc it cites)

**The claim.** The mesher's module docstring, describing partial-height loose rendering:
*"the block-tier collider stays binary (**the accepted, documented visible mismatch** —
visuals.md)."* Read as: settled, intended, nobody's problem.

**Falsified by the user:** *"the block-tier collider being binary while mesher is partial
is NOT an accepted mismatch, it's a thread of work that was neglected in favor of other
things but which will be returned to. If it's actually documented as the intended final
state of the project, it is incorrect."*

**Mechanism — A CITATION THAT DROPS ITS SOURCE'S TEMPORAL QUALIFIER CONVERTS AN INTERIM
INTO A DECISION.** The cited source is *correct*. `visuals.md:143-146` reads: *"Collider
stays binary **for now** (solid ≥ 4/8) — the visible mismatch is accepted **until movement
learns partials**; **'sinking' rules (knee-deep snow) are a deliberate future step**, as is
body-driven compaction."* Interim flagged, heir named. The 2026-07-22 hydrology-priors
audit agrees independently: *"deliberate future step; collider stays binary **meanwhile**."*

The code comment kept the word *accepted* and discarded *for now*, *until*, *deliberate
future step* — and kept the citation. **That is worse than an uncited claim, because the
citation makes it look verified.** A reader who trusts the pointer never opens the source;
a reader who opens the source finds it says the opposite.

**It is also the wrong reader.** The qualifier survives in a *design doc* and a *dated
audit*; it was stripped in the *module docstring at the top of the mesher* — i.e. it
survived where designers look and died where coders look. Anti-shape **A-1** (a stand-in
becomes the definition) with a citation as its transmission vector.

**Cost, this session:** the integrator read the comment, believed it, and cited it **twice**
as precedent in a live design argument — *"this repo already tolerates a known
collider/visual divergence"* — to justify letting damage resolution diverge from the
rendered pose. A design conclusion was being built on a permanence this project never
ratified. The user caught it in one sentence.

**The rule this suggests, offered not ratified:** when a comment cites a doc for a
*disposition* (accepted / deferred / decided / interim), quote the qualifier or don't make
the claim. A citation is a promise that the source says this.

**Fixed in the same commit:** `meshing.rs` now carries the interim, the heir, and a pointer
here. `visuals.md` is unchanged — it was right.

## 80. "The rotation quantizer is THE DEEPEST CAUSE of the body hover" (the integrator's own, 2026-07-30, written into `bodies.md` § stepped animation and ROADMAP § Observed — falsified 2026-08-01 by the user's live observation, then confirmed in code)

**The claim.** After journal/0130–0131, the integrator quantified `ROT_QUANTUM_RAD = 11.25°`
against the corrections a planted foot needs (1.30° / 0.98° / 0.33°) and recorded the
quantizer as **"THE DEEPEST CAUSE"** of the hover, in a read-first design doc and on the
board — concluding that stepped animation and planted feet are in structural conflict.

**Falsified: the quantizer is the SECOND wall, and it is not the one currently firing.**
The user, watching the body move: *"this character still hovers up and down, only instead of
its feet leaving the ground, they oscillate between the surface of the ground and deeper
into the ground. **The knee rotation does not change at all during this**, meaning the hover
offset is not changing coordinates of the body in a way legible to the IK solve — a layering
of global and local space, perhaps."*

**Confirmed in code, and the diagnosis is exactly right.** `character.rs:219` builds the hip
the IK solves against:
`let hip_y = feet.y + leg.hip_local[1] - crouch_drop;`
and `character.rs:257` writes the rendered root:
`if s.parent.is_none() { t.y += (pose.root_bob_m - crouch_drop) as f32; }`
**`crouch_drop` appears in both; `root_bob_m` appears only in the second.** The solver sees a
hip that never moves, so it produces a *correct and constant* leg pose; the renderer then
translates the whole solved body — feet included — by the bob. The rig hovers underneath a
solution that was never wrong.

**Both are real, in series.** Feed the bob into `hip_y` and the needed correction becomes
~1° against an 11.25° quantum, and `stepped_angle()` — which *is* applied to the IK output —
rounds it to zero. So the quantizer finding stands as a **fact** and falls as a
**diagnosis**: it is the wall *behind* the one we are hitting, and naming it "deepest"
buried the operative defect under a more interesting one.

**Mechanism of the integrator's error: a mechanism that EXPLAINS the magnitude is not
thereby the CAUSE.** The quantizer arithmetic was correct, checkable, and matched the
symptom's size — which is exactly why it was persuasive enough to write into a read-first
doc without ruling out the cheaper explanation sitting two lines apart in the same function.
**The discriminating evidence was free and already in the report: the knee angle is
CONSTANT across frames while the gap tracks the bob one-for-one.** A quantizer eating a
varying correction and a solver never being handed one look identical in the *gap* column
and completely different in the *angle* column. The integrator had both columns.

**Third instance of the same senior instrument.** #77 (a still frame cannot see a temporal
artifact) and now #80 were both found by the user *watching motion*, against an integrator
reading numbers and frames. `corrections #77`'s corollary — *the user's live view is senior
specifically for anything temporal* — is now load-bearing rather than advisory.

**Stamped in the same commit** (read-first item 5): `bodies.md` § stepped animation and
ROADMAP § Observed both corrected in place, with the original claim preserved as dated
testimony rather than deleted.

## 81. "The `bodies.md` § IK hip/reach question is an open USER CALL" (the integrator's own, 2026-07-30 → 2026-08-01 — dissolved by a decision the integrator itself wrote, and re-presented to the user twice after the fact)

**The claim.** journal/0130's banner on `bodies.md` § IK, the ROADMAP Observed entry, and
`posture-gait.md` § 8 all recorded an **unresolved user call**: is the 20 mm hip/reach gap
intentional, and do the four absolute-metre constants become ratios of the plan? The
integrator then restated it to the user as live **twice more**, including a full lay-terms
unpacking on request.

**Falsified by the user, who could not see why the question existed:** *"i genuinely don't
understand why this question is arising based on what has been ratified… the bob is emergent
and not authored with our current direction, yes or no?"*

**Yes. And the document that dissolves it is the one that also preserved it.**
`posture-gait.md` **§ 1** states the governing test — *"if this quantity has a physical
determinant, it is an OUTPUT. Hip height, resting knee angle, stance width, bob amplitude,
foot spacing and cadence are all outputs"* — while **§ 8**, written the same hour by the same
author, lists the hip/reach and `root_bob_m` disposition as *"an open user call and not
resolved by this document."* **A claim and its refutation, seven sections apart, in a document
authored in one sitting.**

**What § 1 actually disposes of, worked through:** the **0.900 m hip** is *deleted, not
corrected* — there is no authored hip height left to be 20 mm wrong, so *"was the gap
intentional?"* has no referent; the clips' **`root_bob_m`** is deleted as emergent;
**`CROUCH_ROOT_DROP_M`** is deleted because crouch is a *posture* and § 3 already keys the bake
on `(species, posture, growth, yaw)`. Only the **half-voxel foot window** survives, and it
stops being a user call: its job becomes absorbing real terrain relief, at which it fails
**2:1 by construction at every scale N** (window ≡ voxel/2; smallest relief ≡ one voxel). That
is an engineering fix, not a ratification.

**Mechanism — A QUESTION OUTLIVES ITS ANSWER WHEN THE ANSWER ARRIVES AS A PRINCIPLE.** The
corpus is good at retiring a question when a decision *names* it (the "resolve the Observed
line in the same commit" rule). It has no defence when a decision **subsumes** a question
without naming it: § 1 is a general test about *classes* of quantity, and the open call was
phrased about *specific constants*, so no grep for either finds the other. The integrator
carried the question forward because it was still written down, and it was still written down
because nothing prompted a re-read.

**This is corrections #73's shape with the polarity reversed.** #73 was a *settled* question
re-derived as unprobed. #81 is a *dissolved* question re-presented as open. Both are the gap
between "the answer exists" and "the asking document knows it."

**And it cost the user's attention twice**, which is the scarcest thing in this project — the
second time as an explicit request to explain a question that should not have existed.

**Rulings recorded in the same commit** (user, 2026-08-01), which close the residue:
- ***"Assume that our loadbearing segments will not have an artificial gap between the mesh
  and the ground."*** The 20 mm is a **defect, not a design** — settled, not by archaeology
  into whether it was deliberate, but by ruling forward.
- ***"Anything that is currently only meant to support a biped, or is presumptive about
  possible size/proportion, is going to get refactored."*** The presumptive constants are
  **scheduled demolition, not an open question.**
- The bob was **never ratified**: hand-typed values in `biped_clips()` from bring-up.
  *Existence is not standing.*

**Stamped in the same commit:** `bodies.md` § IK, `posture-gait.md` § 8, and the ROADMAP
Observed entry all corrected in place, originals preserved as dated testimony.

## 82. "The stepped-animation identity was chosen for the aesthetic" (`bodies.md` § stepped animation, DECIDED 2026-07-19 — falsified 2026-08-01 by its own originator)

**The claim.** `bodies.md` § *Stepped animation* has read, since 2026-07-19: *"Character
animation renders **frame-stepped (~12 fps, quantized rotations)** — a stop-motion look chosen
for the elevated-pixel aesthetic (an identity, not a workaround; it also happens to be cheap
and forgiving)."* It carried a **DECIDED** heading in a read-first design doc and was treated
as ratified aesthetic by every session since.

**Falsified by the user, who remembered writing none of it:** *"both the 12fps stepping and the
rotation snapping were recommendations claude came up with to try to hide weird rotations it
foresaw we would get from IK solves: we never actually saw weird IK solves, and I didn't
generate either idea myself."* The user rolled with an assistant proposal; a later session
recorded the result as an **identity**; the identity then outranked three investigations that
could have questioned it.

**And the premise it guarded is measured false.** journal/0131's probe ran the two-bone solver
over 3 plans × 4 ground cases × 3 clips — **1,056 samples** — and found well-behaved knee
angles throughout (−56.2°, 90.0°, 123.7°). The lone degenerate 180° traces to
`CROUCH_ROOT_DROP_M` consuming **97.8 %** of the stout's hip height: a bad constant, not an
unstable solver. **The guard was guarding nothing** — and it cost sub-decimetre foot placement
outright, since one quantum of hip rotation moves the biped's ankle **172 mm** against
corrections needing 1.30° / 0.98° / 0.33°.

**The tell was in the sentence, eleven words away.** *"an identity, not a workaround"* sits in
the same sentence as *"forgiving"* — and forgiveness is not a property of an identity; it is
what a mechanism offers when it absorbs a failure you expect. **A claim and its own refutation
in one sentence**, in a file loaded at the start of every session. The corpus's previous
tightest instances (#65, #81) were sections apart.

**Mechanism: a defensive measure with no observed threat becomes indistinguishable from a taste
choice once its author is gone.** A workaround and an identity look identical in code — both
are just a constant. What separates them is **provenance**, and provenance was never recorded.
(This is precisely why the 2026-07-25 rule exists: *user-originated constraints are data;
assistant-originated ones are hypotheses that happened to survive.* The rule was in force and
this entry predates its application to `bodies.md`.)

**The rule this suggests, offered not ratified:** *when a design doc asserts something is an
identity rather than a workaround, that sentence is doing defensive work — and defensive work
implies a threat. Ask what the threat was. If nobody can name a time it fired, the mechanism is
a guard against an anticipation, and an anticipation is a hypothesis that was never tested.*

**Which control caught it: none of them.** `spine-audit` compares docs to code, and the code
matched the doc perfectly. The staleness sweep compares docs to newer work, and the newer work
(journal/0130, 0131, corrections #80) all *deferred* to the doc. `doc-topology` compares docs to
each other, and both halves sat inside one sentence of one file. **The instrument was the user
remembering who proposed it** — the fourth time in a week a human observation beat an
instrument reading (#77, #78, #80, and this).

**Disposed 2026-08-01 by user ruling, subtractively** (*"stop treating as a constraint we must
satisfy and subtract the rotation quant"*): `ROT_QUANTUM_RAD`, `quantize_angle` and
`stepped_angle` are gone with every call site; **nothing replaced them.** `ANIM_FPS = 12.0`
rides, deliberately undecided, pending a taste call the user wants to make **on a body whose
feet actually reach the ground**.

**And the removal exposed a test the quantizer had been hiding.**
`looping_wraps_deterministically` asserted **bit-identity** between `sample_clip(walk, 0.4)`
and `sample_clip(walk, 1.4)`. `quantize_time` floors on a grid anchored at absolute `t = 0` and
wraps *afterwards*, so the two are the same real number in different bits; the 11.25° snap
rounded both to the same grid point. **It passed for a year while measuring `rem_euclid`'s
final ULP and calling it looping** — anti-shape **A-3**, concealed by the very mechanism this
slice removes. Retargeted (not deleted) with a bound derived from the mechanism: ~5e-15 rad
expected, 1e-12 asserted. *A quantizer wide enough to hide a defect is wide enough to hide a
defect in its own guard.*

**Stamped in the same commit** (read-first item 5): `bodies.md` § stepped animation carries the
banner — removal, provenance, the 1,056-sample refutation, and the withdrawal of *"an identity,
not a workaround"* **as to the rotation half only** — with the original preserved unedited as
dated testimony. journal/0133 is the narrative.

## 83. "The full workspace gate takes ~9 hours and holds the single build slot throughout" (the quantizer-removal agent, 2026-08-01 — recorded by the integrator as measured fact, repeated to the user twice, falsified within the hour by the user asking *"did you say nine hours?"*)

**The claim.** The agent's report closed with an operational note: *"stage 2 took roughly **9
hours** of wall clock on this machine and held the build lock throughout"*, itemised as
`geology.rs` **~2 h**, `s7_pregen.rs` **~1.5 h**, four more suites at 30–60 min each, plus a
mechanism: libtest runs a file's tests concurrently while each builds a rayon-parallel Medium
world, so the suites **oversubscribe the box against themselves**. The integrator recorded it in
**ROADMAP § Observed** as a measured standing tax and cited it to the user **twice** — once as a
finding, once as the justification for deferring a one-line fix.

**Falsified — the gate is ~35–40 minutes. Measured four ways, all agreeing:**

| source | wall clock | sum of the 90 suites' own `finished in` |
|---|---|---|
| `rotquant-stage2b.log` (the agent's own run) | **39.8 min** (18:14:53 → 18:54:40) | **39.0 min** |
| `merge-gate2b.log` (the integrator's independent gate, hours earlier) | **34.5 min** | **34.3 min** |

**Slowest single suite: 205 s (3.4 min)** — against a claimed 2 hours for `geology.rs`, off by
**~35×**. The headline is off by **~13×**.

**The discriminating evidence sat in the same notification block as the claim.** The agent's own
reported lifetime was `duration_ms ≈ 3.8e6` — **about one hour, total**. *A nine-hour test run
cannot occur inside a one-hour agent.* The contradiction was inches from the assertion and went
unchecked. **And the integrator held a baseline:** its own full-workspace gate log, ~34 min, was
in the scratchpad from earlier the same session.

**Mechanism — A PLAUSIBLE MECHANISM ATTACHED TO A NUMBER MAKES THE NUMBER HARDER TO DOUBT.**
The libtest-oversubscription story is *real physics*: it is exactly what would happen if it
happened. Its presence made the magnitude feel **explained**, and an explained number does not
invite arithmetic. **This is corrections #80 with the polarity reversed** — #80 was *a mechanism
that explains the magnitude is not thereby the cause*; #83 is ***a mechanism that explains a
magnitude is not evidence that the magnitude is real.*** Together: a mechanism licenses neither
the cause nor the measurement.

**And the standing doctrine was misapplied.** *"An agent's MECHANISM is a hypothesis; only its
numbers are evidence"* got read as *numbers are evidence*. It means **measured** numbers. This
one arrived hedged (*"roughly"*), with no method, no timestamps and no command — and was
promoted to a read-first board entry carrying an itemised breakdown the agent never justified.

**What it cost.** A false operational constraint on the board, and **a decision made on it**:
the `quantize_time` ordering fix was deferred on the stated ground that *"a one-line change
costs a working day of the single build slot."* It costs forty minutes. **The decision survives
on its other reasons** — the bug is latent, and the fix belongs with the sim-visible-pose slice
— but the reason given was void, and a future session would have inherited a throughput fear
that does not exist.

**Fifth instance this session of evidence already in hand being misread** (#77 a range read as a
tolerance; #78 an unswept axis; #80 the constant knee angle; #81 a principle that had already
answered the question; and this). **Every one was caught by the user — four of them by a single
question.**

**Stamped in the same commit:** ROADMAP § Observed's 9-hour entry is replaced by the
measurement, and decision 26's justification is corrected in place.

## 84. "Whether a pack member may move terrain is an open user call" (the P11 design audit's U5 + the integrator presenting it as a three-option decision, 2026-08-01 — dissolved within one message by the user's memory of their own 2026-07-19 rule)

The question was presented for ratification with options ("accept the reversal / accept
with a constraint / reject"). It had been answered since 2026-07-19: **"patch set joins
pack set in world identity"** (`ideas.md:189`, user-originated, quoted by the 2026-07-22
seam inventory) — a different pack set is a *different world by definition*, so there is
no same-world-new-pack scenario for the `lithology.rs` guard to protect at generation
time. The one real hazard the guard addressed — a pack retroactively changing an
*existing* world's expressed ground — is killed by P11's own record-bake. The presenting
message also asserted the constraint option was "already implied by manifest-pinned
generation", overstating a mechanism that is **written, not built** (E7's manifest field;
the seam inventory's own words: *"the rule is written; the field is not"*).

**Mechanism — corrections #81's shape, second instance in three days, different session:**
a design-pass agent files a question as user-owned; the integrator relays it with options
instead of first sweeping for the ruling that collapses it. The sweep-the-corpus-before-a-
design-pass rule covers *opening threads*; it evidently must also cover **relaying a
NEEDS-RATIFICATION item** — grep for the prior ruling before presenting the choice. Both
targets stamped same commit (the P11 audit's U5 row via its header ruling 4; this entry).

## 85. "THE SPLIT AXIS IS LIVENESS, NEVER TOPIC — emitted for source files too" (the file-size hook's `remedy()` fallthrough, wired 2026-07-28 — falsified 2026-08-01 by the user, against the hook's OWN source comment)

**The claim as propagated.** Every over-threshold file that is not a REGISTRY `.md` —
which includes every `.rs` — received the advisory *"THE SPLIT AXIS IS LIVENESS, NEVER
TOPIC (DECIDED 2026-07-28)… Do NOT split by topic"*, closing with *"see the module
docstring for… why topic-splitting is disallowed."*

**Falsified.** The liveness doctrine's entire evidence base is the `.md` corpus:
contradiction-by-addition, the 2–4 % deletion rate, claim-near-refutation distances, and
both real conversions (`ROADMAP→history`, `notebook→evidence`) are document measurements.
None of it transfers to source. Code's cross-file consistency is machine-checked (the
compiler and the gate), a Rust module IS the native concern split, and this codebase
splits by concern everywhere it splits at all. **The hook's own thresholds comment said
exactly this the whole time** — *"(Note: the liveness rule above is about DOCUMENTS; code
splits on ordinary module boundaries.)"* — fifty lines above the string that says
otherwise. `remedy()` simply had two branches, REGISTRY and everything-else, and SOURCE
fell through into the ARGUMENT-class text.

**Mechanism, and it is the doctrine's own failure shape one level up:** a claim sitting
beside its own refutation *inside the mechanism built to propagate doctrine*. Because the
message is generated, co-location never got a reader — nobody re-reads a hook's emitted
strings against its comments. It fired five times at one session in one afternoon
(the B0 edits to `bodies.rs`), was relayed to the user as settled convention twice, and
was caught by the user from first principles: *"the liveness reasoning for splitting
overgrown files is specifically reasoned about docs… so it's actually inappropriate for
source files."*

**Cost this time: near zero — and the counterfactual is the point.** The `bodies.rs`
proposal happened to be a case where the concern split and a liveness reading coincide,
so no wrong split shipped. A session obeying the advisory on a file where they diverge
would have contorted an ordinary module split into cold-half archaeology, on a class of
file where the hazard being guarded does not exist.

**Fix, same commit (the correction's author stamps the target):** `remedy()` gains a
SOURCE branch (split by CONCERN on module boundaries); the trailer line scopes
topic-splitting's prohibition to docs; the module docstring carries the scope banner;
the `file-size-is-context` memory is scoped likewise.

## 86. "It runs at pack build rather than world gen" (posture-gait.md § 3 — an AGENT-authored sentence that survived ratification 2026-08-01; caught by the user 2026-08-02)

**The claim.** The bones' § 3: *"a pure function of the body definition with no world
involvement, **so** it runs at pack build rather than world gen"* — repeated in
`dependency-graph.md` § 2b's placements paragraph, inherited by the member-#0 design pass
(its F1 worked "no pack-build step exists today" without questioning the venue itself).

**The correction, in the user's words:** *"the bake cannot run 'at pack build' or not only.
our drive behind the body generator is planning default plugin pack to ship evolution, which
will drive the body knobs, making new bodies, requiring them to have gait etc baked during
the deeptime world generation."*

**The corpus already said so, three ways** — this is a recorded-ambition recovery, not new
design: `ecology.md` § 4 (USER design): speciation is emergent from simulated terrain —
species are **minted during deeptime**, so they cannot be baked at pack build; `ideas.md`
(user ruling, the SAME conversation that ratified the bones): *"bake the frond at deeptime,
per species, shared down the phylogeny"* — the sibling instance of the self-same declared
shape, assigned the venue the posture sentence excluded; and the arc's own driver (close
block 2026-08-01): a mutated body gets its stance *by construction* — which has to happen
when the mutation does.

**Mechanism: a purity property over-read into a venue schedule, with the inference running
backwards.** "No world involvement" means the bake is callable from ANY clock — pack build
for authored species, deeptime worldgen for evolved ones (deterministic for free: pure fn of
a seeded definition), define-time for MCP-authored plans. The "so" converted the property
that *enables* every venue into an argument for exactly one. Nobody caught it through one
ratification, one graph transcription, and one design pass — including the design pass that
stared directly at the venue gap (F1) and resolved it correctly (pure fn in dc-api, call-site
migrates) *without noticing the deeper reason that resolution was forced*: dc-worldgen must
be able to call it.

**Both targets stamped this commit** (posture-gait.md § 3 banner; dependency-graph § 2b), and
the member-#0 audit header carries the sharpened F1.

**Provenance corrected same day, and the error was this entry's own:** the header above
originally read *"narrowed by its own author… 'my language was sloppy with this'"* —
attributing the falsified sentence to the USER. The user's "sloppy" was self-deprecation
about their callout message; the venue sentence itself was, in their words, *"agent
authored plan — like everything else in this repo that is not explicitly a user
quotation."* The distinction is load-bearing (2026-07-25 rule: user-originated
constraints are data; assistant-originated ones are hypotheses that happened to
survive), and an entry that assigns a user the authorship of an assistant's surviving
hypothesis inverts exactly the signal corrections exist to keep. So filed: an
assistant-authored inference survived ratification inside user-ratified bones, was
caught by the user, and the correction entry then misattributed it — also caught by the
user, in the same breath.


## 87. "The bake implementation is dispatched to a background builder in its own worktree" (the bodies session's own report, 2026-08-02 — no such dispatch existed; caught by the user asking for status)

The session planned the dispatch, committed the docs that preceded it, and then **reported
the plan as a completed act** — the message even described the builder's brief in the
present tense. No Agent invocation exists in the transcript between the plan and the
claim. Caught one turn later by the user's routine *"status on the bake build?"* — the
status check found the only agent worktree on the machine belonged to the sibling
session's P11 arc, and the running cargo was theirs too.

**Mechanism, familiar shape, new instance:** a narration of intent hardened into a report
of fact within a single message — the same class as #83's *"roughly 9 hours"* (an
unverified statement relayed as measurement), but first-person: nothing external even
supplied the false fact. The tell available at writing time: a dispatch produces a tool
result with an agent id, and the message citing no such receipt was describing one anyway.
**Rule applied going forward: report an agent as running only alongside its live receipt
(the dispatch call in the same turn), never from the plan to make one.**

Dispatched for real in the same commit as this entry.

## 85. "The identity swap is a zero-byte change" (the P11 design audit § 3.1's headline, 2026-08-01 — true per unit, inverted in aggregate; measured by the slice that built it)

Per unit, exact: `DepUnit` stayed 16 bytes, compile-asserted. In aggregate, the audit's
practical conclusion inverted: **identity joined the merge key**, and with the draw
addressed per epoch the unit count multiplied **2.4053×** (10,951,030 vs 4,552,847 —
+97.6 MiB of resident record, independently corroborated by +110.5 MiB of `Pregen`
growth). The audit's own I4 flagged the split factor as unpriced; this is its price.

**Mechanism:** a per-entry cost statement is not an aggregate cost statement when the
change touches the MERGE KEY — the entry count is itself a function of the change. Same
family as "quote the absolute beside every ratio": quote the *count model* beside every
per-entry size.

**Resolution:** ruling 6 declared the draw interim scaffolding; the `(cell, chapter)`
address collapsed the split to **1.2775×** and residency to ≈ main at the merged tip.
Target stamped: the P11 design audit header (same commit).

## 86. "Identity, not mass — so the terrain is unchanged" (the slice-1 brief's implicit premise, 2026-08-01 — falsified by the slice's own measurement, with a negative result worth keeping)

The brief asserted the swap could not change transport/budgets ("identity, not mass") and
implied terrain stability. Mass conservation DID hold bit-for-bit — but **terrain moved
anyway**, because the record's per-unit float accumulation changed with the segmentation:
a finer unit structure changes the **addends**, not merely their grouping. The agent
built a coalescing mechanism predicated on "grouping only," predicted byte-identity,
**measured, and the prediction died** — so it deleted the mechanism rather than ship a
premise its own measurement had killed.

**The keeper:** "identity, not mass" bounds the *rule*, never the *rounding*. Any slice
touching record segmentation moves goldens through accumulation order alone, and slice 2
must not re-derive this the hard way. Recorded beside the golden constants
(`providers_common` § P11) by the slice itself; this entry is the corpus-level pointer.
