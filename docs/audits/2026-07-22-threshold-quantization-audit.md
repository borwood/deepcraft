# Threshold-quantization audit — the S-4 "square verdict" sweep

> **⚠ STALENESS BANNER — stamped 2026-07-29 (E5 member #0, journal/0124). Immutable body,
> mutable header.** This document's **Part 2** ("the type extraction") shipped
> 2026-07-22 as `dc_core::coarse` (journal/0075) and then had **zero production callers for
> seven days**. The first adoption landed 2026-07-29 at **site B1 only** —
> `collapse.rs::surface_class`, the far field — through a design pass that **re-derived the
> adoption plan rather than inheriting it** (`docs/audits/2026-07-29-member0-coarsefield-design.md`,
> ratified by the user as E5 member #0). Read Part 2 as the origin of the type, not as a
> live plan: it fused three separable kernels (fine read · `summarize` · midpoint jitter),
> and the 2026-07-29 pass split them — `summarize` was ruled to **the octree node
> contract**, and the midpoint jitter was named **move C**, a third coarse→fine move this
> audit's two-move vocabulary cannot express. Site A1's blend is still a plain function.

*2026-07-22. Read-only sweep of `dc-worldgen` (exhaustive over deeptime + pregen
+ fill + collapse) and `dc-core` where the seam reaches it. No source changed.
Measured numbers and file:line citations in the `docs/audits/` register; this
document proposes, it does not ratify.*

## Why this exists

The user's sharpening of S-4 (spines.md § S-4, 2026-07-22, walk-0071):
**"smoothing a verdict preserves the shape of the cell that voted it."** A
decision taken from a *bulk summary of a cell* — one cell marginally crossing a
threshold its neighbours miss — stays a **square phenomenon** under any amount
of downstream smoothing (a feathered square is still a square). The two legal
cures are stated in the spine:

- **A — threshold late, at fine scale, on interpolated causes.** The boundary
  then follows the cause's contour, not the grid.
- **B — dither membership** where the cause is non-interpolable/categorical:
  unbiased seed-addressed stochastic assignment at the boundary (S-7's medicine
  at a different joint), so the transition is a statistical gradient, not a line.

Verdict-smoothing is never the cure. The immediate trigger is journal/0068's
thickness-dominance outcrop rule, flagged in walk-0071 as an S-4 edge: it flips
adjacent 460 m cells to *discontinuously different* erosion rates at the
plurality crossover, and unlike the old top-unit rule's noise-like flips this
boundary is **spatially coherent** — it follows thickness contours, exactly the
kind of line S-4 forbids reaching the eye.

**This audit was commissioned with a by-construction amendment** (user, ratified
post-dispatch; spines.md § S-4 "Ratified end-state"): the fix is not an opt-in
helper family per site, but **a boundary type at the sim→expression seam** whose
API offers *only* the two legal moves and never exposes the raw per-cell read —
so painting a cell verdict onto a finer scale becomes *inexpressible* downstream.
Part 2 proposes it; the residue section names what it cannot absorb.

---

## Part 1 — the inventory

**Scope of the seam.** In this engine the "cell" that votes is almost always a
**460 m deep-time cell** (`DEEP_CELL_M = 460.0`, capped at `DEEP_MAX_WIDTH = 550`
per side — field.rs:41-45). The A-tier erosion/biotic/climate sim
(`deeptime/{erosion,biotic,climate,lithology}.rs`) runs *at* that resolution;
its distilled output is `DeepField` (field.rs:198-251), and the **expression
boundary** is `DeepField`'s `*_at_voxel` accessors, read by `collapse.rs`. That
boundary — not the sim interior — is where S-4 binds. A verdict computed at
460 m and *baked into the committed record* is cell-honest (the sim IS cellular);
the question S-4 asks is whether that verdict **reaches the eye as a 460 m square
or an analytic line**, and if so whether the boundary can be made to follow the
cause's contour (A) or be dithered (B).

### Class counts

| class | meaning | count |
|---|---|---|
| **A** — threshold-late | cause interpolable; blend/threshold at fine scale | **4** |
| **B** — dither-membership | cause categorical/non-interpolable; addressed dither | **3** |
| **C** — harmless | never reaches the eye | **3** |
| **D** — already-compliant | cites a working mechanism | **6** |

Plus **7 sim-internal verdicts** that are cell-honest by design (residue (a)),
and **2 conservation-pinned nearest-samplings** (residue (b)). Enumerated below
and in the residue section.

### The 5 most consequential sites (called out inline)

1. **`exposed_litho` argmax → erosion susceptibility**
   (`deeptime/lithology.rs:442-494`, consumed `deeptime/erosion.rs:874-879`,
   957). Class **A**. The walk-0071 flag itself: a winner-take-all `Litho` over
   the topmost `OUTCROP_DOMINANCE_WINDOW_M = 0.9 m` (lithology.rs:409) is
   `index()`ed into a 6-entry susceptibility table (erosion.rs:855-864), flipping
   the fluvial + weathering + creep + frost rates discontinuously along a
   thickness contour. Reaches the eye through **erosion → terrain shape**. The
   cause (per-`Litho` thickness *shares* in the window) is interpolable and the
   consequence is a scalar rate, so the cure is a **share-weighted susceptibility
   blend** (the ROADMAP-named "55/45 cell gets a 55/45 rate"), of which argmax is
   the limiting case.

2. **`surface_class` argmax → surface block**
   (`collapse.rs:786-829`, via `surface_sample` collapse.rs:681-770). Class
   **B**. The eye-reaching twin: the dominant content class by metres over the
   record's topmost 0.9 m becomes the surface **block** a player sees, read from
   `record_at_voxel` (**nearest**, field.rs:404-412). The *member within* the
   class is already dithered off the chunk grid (journal/0058,
   `interp_select_draw` collapse.rs:729), but the **class boundary** steps at
   460 m. Non-interpolable cause (the strata list), so the cure is B.

3. **`regolith_at_voxel` nearest under bilinear terrain**
   (`deeptime/field.rs:387-395`). The S-4 **live violation** of record. Soil
   depth `H` is a hard-edged 460 m mosaic while `surface_at_voxel` beside it is
   bilinear (field.rs:364-366). **Residue (b)** — nearest is a forced
   mass-conservation trade-off (field.rs:374-383): `H` must name the *same cell*
   as the strata subtraction or loose material leaks at every boundary. The type
   can *force the choice to be named* but cannot prove a dithered variant
   physically conservative; see residue.

4. **`energy_band` capacity thresholds** (`deeptime/erosion.rs:450-458`) and the
   **aridity gate** `precip < 0.32` (erosion.rs:469, biotic.rs:804). Class **C /
   residue (a)**. Both are per-cell verdicts, but they are recorded *into the
   `DepTag`* at the sim tier and only ever re-read through the strata record —
   they are sim-internal facies decisions, cell-honest, and reach the eye only
   after passing through site 2's boundary (which the type governs). Cell-scale
   here is correct; the leak, if any, is downstream at site 2.

5. **`zonal_wind` band edges** (`pregen/climate.rs:84-96`). Class **D** — the
   proven precedent. A three-way easterly/westerly/polar **sign bit** printed a
   dead-straight vegetation/dune line at 30°/60° (the "analytic-boundary scream",
   climate.rs:8-13); the fix made the *cause continuous* — a C¹ squared-sine
   magnitude passing through zero at each boundary (calm belt) — so no hard line
   exists **by construction**. This is exactly the A-shaped medicine applied at
   the cause, and the template the by-construction type generalizes.

### Full inventory

#### Class A — threshold-late (interpolable cause, blend/threshold fine)

| # | site | cause quantity | cell → expression | reaches eye? | note |
|---|---|---|---|---|---|
| A1 | `exposed_litho` argmax, `lithology.rs:442-494` → susceptibility table `erosion.rs:855-878` | per-`Litho` thickness *shares* in the 0.9 m window (interpolable) | 460 m verdict → scalar erosion rate | **yes**, via terrain shape (bilinear surf feathers a real rate step) | the walk-0071 flag; blend susceptibility by window share |
| A2 | frost agent outcrop read, `erosion.rs:944-958` | same window shares | 460 m `Litho` → `frost_tab[index]` weathering multiplier | yes, via periglacial relief | shares A1's fix: same blended susceptibility feeds `frost_tab` |
| A3 | `surface_sample` fallback bands `elev<=-1 / >-35 / t<-4` → Dirt/Stone, `collapse.rs:750-762` | continuous elev (bilinear lattice) + temp | per-voxel analytic threshold on interpolated cause | yes (the shoreline/snow stripe — "veneer banding drawn as a coastline", journal/0046) | *already* thresholds late on an interpolated cause (boundary is a curve, not a grid square) — but it is a **hard categorical block flip** on a summary, so it reads as an analytic line; dithering the flip zone (B-flavoured) removes the line. Fallback-only (no record), low blast radius |
| A4 | `coarse_fraction` split, `geology.rs:280-282` | continuous `flow_energy` | fraction coarse vs fine clastic per column | yes (material) | analytic `clamp(e/30, 0, 0.85)` — already continuous in the cause; the *class assignment* it feeds is dithered downstream (fill). Borderline D; listed for completeness |

#### Class B — dither-membership (categorical/non-interpolable cause)

| # | site | cause | cell → expression | reaches eye? | note |
|---|---|---|---|---|---|
| B1 | `surface_class` plurality, `collapse.rs:786-829` | strata unit list (non-interpolable) sampled `record_at_voxel` **nearest** | dominant class by metres → surface **block** | **yes, directly** (surface colour) | member dither exists *within* class (0058); the **class** boundary steps at 460 m. Cure: dither class membership in the boundary zone with a position-addressed draw à la `SALT_GEO_FILL` |
| B2 | `record_at_voxel` nearest, `field.rs:404-412` | variable-length unit sequence | 460 m facies mosaic under all buried expression | yes (buried strata, cut faces) | **residue (b)-adjacent** but distinct from B1: the record cannot be interpolated, but *which cell's record a fine voxel reads* can be dithered in the boundary band (the doc comment at field.rs:399-403 already notes the per-voxel member dither smooths *within* a facies; the *between-facies* contact still steps) |
| B3 | `promote_coal` burial threshold, `biotic.rs:713-751`, `COAL_ONSET_C` biotic.rs:150 | `burial_temp_c` (a **quantity**, seamed — journal/0067) | per-unit peat→coal retag at 460 m | yes (diggable coal), but see note | **mostly residue (a)** (sim-internal record retag). Listed under B because the coal/peat contact follows a burial-temperature contour that is spatially coherent; if it ever reads as a hard line the cure is to dither membership in the onset band, not to smooth the verdict. Low priority — coal is sparse (0.78 % of readable columns, biotic.rs:107) |

#### Class C — harmless (never reaches the eye)

| # | site | why harmless |
|---|---|---|
| C1 | `energy_band` thresholds `erosion.rs:450-458` | recorded into `DepTag.energy`; only re-read through the strata → routed through B1's governed boundary. Sim-internal facies decision |
| C2 | aridity gate `precip<0.32` `erosion.rs:469`, `biotic.rs:804` | recorded into `DepTag.aridity`; feeds member *fitness* (a continuous selection weight via `deep_precip` geology.rs:289-294), not a hard spatial verdict at expression |
| C3 | `subsidence`/`convective_floor` band centres `pregen/climate.rs:115-134` | Gaussian limbs — continuous everywhere; produce the *input* `precip` field that is then bilinearly sampled (`climate_at`). No boundary of their own |

#### Class D — already-compliant (cite the mechanism)

| # | site | mechanism |
|---|---|---|
| D1 | `zonal_wind` `pregen/climate.rs:84-96` | cause made C¹-continuous (calm belt); analytic line gone by construction. Test `zonal_wind_never_reverses_without_passing_through_zero` climate.rs:210-225 |
| D2 | `surface_at_voxel` `field.rs:364-366` | bilinear; "continuous by construction — the elevation lattice can sample it per point without seams" |
| D3 | `climate_at` `collapse.rs:1037-1056` | bilinear temp/precip between cell centres; registration-guarded (journal/0043) |
| D4 | `ColumnFill` + `allocate` + `allocate_partial` `fill.rs:116-377` | **S-7 reference**: single quantization at the voxel boundary by unbiased addressed stochastic rounding, `draw = f(seed, SALT_GEO_FILL, vx, vy, vz)` (fill.rs:248). The canonical B-medicine implementation |
| D5 | per-voxel member dither `SALT_GEO_DEEP` `geology.rs:432`, `interp_select_draw` geology.rs:191-206 | bilinear selection field over corner hashes → family contacts wander off the 28.8 m chunk grid (corrections #6 fix) |
| D6 | lattice elevation refinement `collapse.rs:950-1003` | midpoint displacement; adjacent columns share ancestors → no seams; deep surface injected bilinearly at `L_DEEP` (collapse.rs:995-1000) |

**What was sampled more lightly, and skipped:** `deeptime/tectonics.rs`,
`isostasy.rs` (`box_smooth` is a flexural load smooth, not a verdict —
erosion.rs:781-798), `pregen/{history,hydrology,tectonics}.rs` (site placement,
drainage routing — categorical but not painted onto a finer scale at
expression), and `water/*` (S11/S15 not on the production path per ROADMAP;
`water/sat.rs` relaxation is continuous). `dc-core/classify.rs::block_twin` and
`Litho::reference_material` are A-7 name-binding defects already tracked in
spines § A-7 / § 3, not quantization edges. None showed an unlisted eye-reaching
square.

---

## Part 2 — the recommendation: a by-construction boundary type

### The shape

Every site above shares one anatomy: a **coarse producer** (the 460 m sim) hands
a value to a **fine consumer** (per-voxel collapse), and the defect is always the
consumer performing a *raw per-cell read* and then either thresholding it or
painting it across the fine span. The fix the user ordered is to make that raw
read **inexpressible**: the producer publishes a typed field whose only
fine-scale accessors are the two legal moves.

```rust
// Home: dc-core, a new module `coarse.rs` (headless; dc-worldgen and the octree
// node both depend on dc-core). The sim→expression seam lives where BOTH the
// deep field and the octree node can name it — A-4 says they are one thing (below).

/// A coarse-resolution field with a fine-scale sampling contract. Construct from
/// the coarse producer (the sim); the ONLY reads are the two S-4-legal moves.
/// There is deliberately no `at_cell(ix, iy)` and no `Index` impl in the public
/// surface — a fine consumer physically cannot ask for a cell's raw verdict.
pub struct CoarseField<T> {
    data: Vec<T>,
    w: usize,
    // registration: continuous cell-space mapping shared with the producer
    // (the `deep_coords` convention, field.rs:345-359 — carried in, not re-derived)
    origin: CellSpace,
}

impl<T: Interpolable> CoarseField<T> {
    /// Move A — threshold LATE on an interpolated cause. Bilinear at a fine
    /// world position. The boundary a downstream threshold draws now follows T's
    /// contour, not the grid. (T: Interpolable is implemented for f64/f32 and for
    /// small blendable structs — e.g. a per-Litho share vector.)
    pub fn sample(&self, pos: WorldVoxel) -> Option<T>;
}

impl<T: Member> CoarseField<T> {
    /// Move B — dither MEMBERSHIP for a categorical/non-interpolable T. Returns
    /// the coarse cell's category, but WHICH cell a fine voxel is assigned to is
    /// an unbiased position-addressed draw across the boundary band — the S-7
    /// medicine. `seed` is caller-owned (no ambient entropy); the salt is the
    /// caller's, matching SALT_GEO_DEEP/SALT_GEO_FILL discipline.
    pub fn sample_dithered(&self, pos: WorldVoxel, seed: u64, salt: u64) -> Option<&T>;
}
```

Two traits gate the two moves, so the *type system* enforces the choice:
`sample` exists only where `T: Interpolable` (you may blend it), `sample_dithered`
only where `T: Member` (you may not blend it, but you may dither which cell wins
the contact). A `T` that is neither exposes **no fine accessor at all** — which
is the correct answer for `exhum`/`t_crust`/`chapters` (spines § 3 rows 3-4):
they have no consumer yet, and the type refuses to invent one.

Rationale, one line each:
- **`CoarseField<T>`** — one type, one seam, so "coarse cause, fine expression"
  (S-4) is a *type*, not a discipline re-remembered per site. Mirrors S-6
  (declared relations, not incidental order) and the providers' `Option<fn>`
  slots (journal/0064: structural, not disciplinary — `None` = identity because
  the address is never observable; here, raw cell read never callable).
- **`sample` (A)** — the single home for every bilinear read already scattered
  across `surface_at_voxel`/`climate_at`; the interpolation is *named in the
  type* (`Interpolable`), so a reviewer sees "this cause is blendable" as a
  compile fact.
- **`sample_dithered` (B)** — the single home for the `SALT_GEO_FILL` /
  `SALT_GEO_DEEP` addressed-draw pattern (fill.rs:248, geology.rs:432), lifted
  from "a convention three sites re-implement" to "the only way to read a
  categorical coarse field finely." Entropy stays caller-owned by signature.

### Convergence with the octree node contract (A-4 pressure says: same thing)

The octree substrate doc (`docs/design/octree-substrate.md` v0.1, RATIFIED
2026-07-22) defines a node payload with, per stratum, a **declared reduction rule
(child→parent) and a declared source**, and the rule: *"Derivation is two-sided …
where children don't exist the node **synthesizes top-down** from the worldgen
statistical authority … the two directions must **agree** where they meet
(S-3/S-7 — falsifiable)"* (octree-substrate.md:113-124). It further names the
*sampling* obligations: *"Agreement tests are the acceptance tests: coarse claims
match exact walks — exactly where derivation is deterministic, **statistically
where quantization is deliberately unbiased**"* (:143-145), and *"Synthesis is
seeded … a pure function of (world seed, node coords, committed facts)"*
(:134-136).

**That is the same two-move vocabulary.** A node stratum read at a finer scale is
either an interpolable reduction (deterministic agreement — move A) or a
categorical one that must be *statistically* unbiased at the contact (move B,
seeded). **Recommendation: the node payload's fine-scale sampling vocabulary and
`CoarseField<T>`'s two moves are one type.** A node stratum *is* a
`CoarseField<T>` (contents = `CoarseField<MaterialChunk>` reduced by
`MixtureDownsampleRule`; occupancy = `CoarseField<u16>` interpolable; water
summary strata likewise). Building two sampling contracts — one for the deep
field, one for the octree node — would be A-4 committed *on the day the octree
pass named A-4 as the shape it fixes*. The type belongs in **dc-core**, where the
octree substrate is decided to live (octree-substrate.md:148-149: *"the substrate
lives in dc-core"*), and `DeepField` becomes its first producer.

Implication to state plainly: `DeepField`'s public `surf`/`regolith`/`strata`/…
`Vec`s and its `*_at_voxel` methods (field.rs:198-251, 364-412) are the *current*
raw-read surface. Migrating them behind `CoarseField<T>` is what makes
`surface_class`'s nearest read of `record_at_voxel` (B1) *stop compiling in its
current shape* — the consumer would be handed a `CoarseField<DeepStrata>` whose
only fine read is `sample_dithered`, and the 460 m class-step would be dithered
by construction. That is the by-construction win the amendment asks for.

### Sequencing — route to the type (seam-first practice #6)

Seam-first #6 (session-workflow SKILL § "Seam-first"): *do not build the general
mechanism first — convert the cheapest cold seam, let it teach the shape, convert
three more, THEN generalize.* **A `CoarseField` registry designed before its
callers exist is the named anti-pattern** (spines § A-4; and the providers slice
refused exactly this — "three seams is enough to design *for* and not enough to
design *from*", journal/0060). So the type is *extracted from* conversions, not
committed ahead of them.

**Shape-teacher #1 — SHIPPED 2026-07-22 (journal/0072).** The blend landed as a
plain function (`exposed_shares` + `blend_susceptibility`) seamed through a new
value-level `providers::outcrop_shares` slot (the quantity, paired with
`outcrop_at`'s verdict under one structural-deformation heir); erosion's four
sites blend rather than argmax-lookup; goldens re-baselined; coal diggability
fell (NEEDS RATIFICATION, ROADMAP). The `Interpolable` witness for the eventual
`CoarseField<T>` is `exposed_shares(units) -> [f64; COUNT]` +
`blend_susceptibility`. Original plan, kept for the record:

**Shape-teacher #1 (convert first — the cheapest hot seam that teaches move A):
A1, `exposed_litho` → susceptibility blend.** It is self-contained
(`lithology.rs` + one consumer in `erosion.rs`), it is the walk-0071 flag so the
motivation is ratified, and its cure is *move A on a small blendable struct* (a
per-`Litho` share vector) — which is the harder of the two traits to get right,
so learning it first de-risks the `Interpolable` bound. It also has a clean
golden story (below). **It does not need `CoarseField` yet** — do the blend
in-place as a plain function, and let its signature (`shares(window) -> [f64;
COUNT]` then a share-weighted `susceptibility`) *become* the `Interpolable`
example.

**Shape-teacher #2 (convert second — teaches move B at the eye): B1,
`surface_class` class-membership dither.** It is the highest-visibility site
(surface colour, every column) and its cure is *move B* — a position-addressed
draw in the class-contact band, structurally identical to the `SALT_GEO_FILL`
draw the fill already owns (fill.rs:248), so it reuses proven machinery. Doing it
second means both traits (`Interpolable` from #1, `Member` from #2) are learned
before the type is written.

**Then — and only then — extract `CoarseField<T>` in dc-core**, with A1's blend
as the `Interpolable` witness and B1's dither as the `Member` witness, and
migrate `DeepField`'s accessors behind it. **Hold until the family stabilizes:**
A2 (frost — rides A1's blended table for free once A1 lands), A3/A4 (fallback/low
blast radius), B3 (coal — sparse, mostly sim-internal), and the octree node
strata (their own slice, FF2b-minimal, which *consumes* the type rather than
teaching it). Converting the octree first would freeze the API against a consumer
whose numbers aren't measured yet (octree-substrate.md § 6 open questions).

---

## Part 3 — ranked shortlist (visibility × blast radius × cheapness)

| rank | site | class | visibility | blast radius | cheap? | golden impact |
|---|---|---|---|---|---|---|
| 1 | **A1 `exposed_litho` blend** | A | med (terrain shape, needs amplitude to read — journal/0030) | high (every erosion cell, 4 rate terms) | yes (1 fn + 1 table build) | **BEHAVIOUR CHANGE — needs golden re-baseline.** Moves `GOLDEN_SURFACE`/`GOLDEN_RECORD` (erosion input changes, cf. journal/0068 which moved both). Coal-dig + Small-control tests re-baseline as in 0068 |
| 2 | **B1 `surface_class` dither** | B | **high (surface colour, direct)** | high (every surfaced column) | yes (reuse `SALT_GEO_FILL` pattern) | **pure-expression at the contact, but statistically visible** — block identity changes in boundary bands. Needs a *statistical* golden (agreement-in-expectation, S-7 style), not a byte-identity golden; the byte goldens WILL move and that is correct |
| 3 | A2 frost outcrop | A | low-med (periglacial only) | med (frost cells) | **free after A1** (shares the blended table) | rides A1's re-baseline; no separate golden |
| 4 | B2 `record_at_voxel` contact dither | B | med (buried cut faces) | high (all buried strata) | med (touches the sampling accessor — do *with* the type, not before) | statistical golden; **residue (b) overlap — see below** |
| 5 | A3 fallback shoreline bands | A | med (visible stripe) but fallback-only | low (no-record columns only) | yes | pure-expression; goldens for wilds/subaqueous columns only |
| 6 | B3 coal onset dither | B | low (0.78 % of columns) | low | yes | defer; mostly sim-internal (residue a) |

**Status (2026-07-22):** rank 1 (A1) dispatched — journal/0072. **Rank 2 (B1)
LANDED — journal/0073**: `surface_class` now draws the surface class from the
top-window metre shares (`SALT_GEO_CLASS`, move B), the byte goldens re-baselined,
near/far agreement holds *exactly* (both paths read the same `surface_sample(vx,
vz)`), and the 10 km checkerboard resolved. **One finding amends this doc's
prescription (NEEDS RATIFICATION):** the recommended white-noise draw ("à la
`SALT_GEO_FILL`") **doubled the far-tile mesh** (21.5 → 43.5 MiB at 1.2 km,
measured) because the far field POINT-SAMPLES this class at a coarse stride and
white noise aliases into unmergeable speckle. B1 shipped the *coherent* bilinear
field instead (the member dither's `interp_select_draw`; far mesh +13 %, in
budget), accepting a small toward-50/50 bias. The lesson for Part 2:
`sample_dithered` cannot be white-noise-per-position alone — a coarse consumer
needs a **share-summary read** (`summarize(region) -> ShareVec`) distinct from the
fine dither, at which point near/far agreement becomes statistical by design (as
this doc's octree convergence section already anticipates). B1 is the
`Member`/`sample_dithered` witness, and its wrong turn is half the lesson.

**Load-bearing vs pure-expression flag:** ranks 1-3 are **behaviour changes**
(they alter the erosion *input*, hence terrain geometry) and require golden
re-baselines with ratification, exactly the journal/0068 discipline (which moved
56.5 % of a world and was ratified from the ground, journal/0071). Ranks 2, 4, 5
at the *expression* layer are pure-expression *in mechanism* but still move byte
goldens and must be validated by **statistical** agreement tests (S-7: match an
expectation, not a value), because unbiased dither is only equal *in
distribution*.

---

## The residue — what the type cannot absorb

The residue list is as load-bearing as the type. Two families:

### (a) Sim-internal cell-scale verdicts — cell-honest by design

The A-tier sim is a genuine 460 m cellular automaton; a verdict it takes *about
its own cell and records into its own state* is not an S-4 violation, because the
principle governs the **expression boundary**, not the sim interior. These stay
cell-scale and must **not** be dithered inside the sim (dithering a
mass-conserving update breaks the ledger). They reach the eye only *through* a
governed boundary (site 2 / the type), which is where the cure belongs.

Classified sim-internal (residue a), NOT type targets:

- `energy_band` capacity → `DepTag.energy` (erosion.rs:450-475) — C1
- aridity gate `precip<0.32` → `DepTag.aridity` (erosion.rs:469, biotic.rs:804) — C2
- `DepEnv` sea threshold `surf<=sea_level` → subsea/subaerial (erosion.rs:464) —
  the coastline facies verdict; the sea itself is a real elevation contour
- biofacies gates: `peat_site` (biotic.rs:1047), `retro` (biotic.rs:1023),
  waterlog/drain thresholds on `wet` (biotic.rs:890-891, 976) → `Biofacies`
- fire ignition `roll < ignition_p` / `CHAR_MIN_BAND` (biotic.rs:1080-1090) —
  already addressed-draw seeded (`SALT_BIO_FIRE`), a *temporal* verdict per cell
- `promote_coal` onset (biotic.rs:713-751) — B3's sim-internal half
- `deep_class`/`litho_of_tag` routing (geology.rs:348-365,
  lithology.rs:381-395) — a per-*unit* categorical map, correct at the unit

The discriminator: **does the verdict get painted onto a finer scale at
expression, or only re-read through a boundary the type already governs?** If the
latter, it is residue (a) and the type fixes it *once*, at the boundary, not at
each internal site.

### (b) Conservation-constrained dither choices — the type can force a choice, not prove it right

`regolith_at_voxel` (A3-inventory / field.rs:387-395, the S-4 **live violation**)
is nearest **not by oversight but by a mass-conservation forcing** documented at
field.rs:374-383: `H` is *exactly* the sum of the cell's own `record_at_voxel`
unit thicknesses (journal/0053), and the collapse tier subtracts the two (total
column minus whole-voxel expression) as its surficial veneer — a subtraction that
conserves mass **only if both terms name the same cell** (collapse.rs:524-529,
geology.rs:488-502). Interpolating `H` while the record steps nearest would
"leak or invent loose material at every cell boundary."

`CoarseField<T>` can make this **honest** — it forces `regolith` to declare
whether it is `Interpolable` (offer `sample`) or `Member` (offer
`sample_dithered`) — but it **cannot prove** that a dithered `H` stays
conservative against the paired strata subtraction. That is a physics obligation
the type does not discharge:

- The **legal** move here is B (dither *which cell* a fine voxel's `H` and its
  record both read — *jointly*, so the subtraction still names one cell), **not**
  A (bilinear `H` alone, which is the exact forbidden leak). If `CoarseField`
  exposes `H` as `Member` and requires the record and `H` to be sampled through
  **one** `sample_dithered` call over a *paired* `(H, record)` field, the
  conservation invariant is preserved by construction and the mosaic is
  dithered. That is the route around field.rs:374-383, not through it.
- Until that paired-field shape is built and its ledger proven (an S-7 agreement
  test: dithered expression conserves `Σ` against the recorded `H`), `regolith`
  stays nearest and stays flagged. **The type does not license "fix nearest→
  something" — it licenses "name the choice and prove the ledger."**

Same caveat, weaker, on B2 (`record_at_voxel`): the record is non-interpolable
full stop, so its only legal fine move is membership-dither at the contact — and
that dither must be the *same draw* that moves `H`, or the two disagree about
where the cell boundary is. B1, B2 and the `regolith` case are therefore **one
conversion**, not three, and that conversion is gated on the conservation proof.

---

## Anything that resisted classification

- **A3 (`surface_sample` fallback bands).** Genuinely ambiguous between A and B:
  the threshold is *already late on an interpolated cause* (continuous elev/temp,
  so the boundary is a curve — the A prescription), yet it is a **hard
  categorical block flip**, so it still reads as an analytic line the way S-4
  forbids ("square *or* analytic boundary", spines § S-4 rule). The honest answer
  is that "threshold late" removes *grid* squares but not *analytic lines* from a
  categorical output — you need the B dither *on top of* the A interpolation to
  dissolve the line. Filed as A with a B-flavoured cure. It is the one site that
  shows the two moves are not mutually exclusive: A picks *where* the contour is,
  B dissolves *the line along it*.

- **A1's dual nature (A vs B).** The verdict is categorical (`Litho`) but its
  *consumption* is purely numeric (a rate multiplier). Classified **A** because
  the cure is to never form the category on the rate path — blend the numeric
  consequence by the interpolable shares — but a reader expecting "categorical ⇒
  B" will trip. The resolution: **B is for categories you must express as
  categories** (a block, a facies); **A is for categories whose only downstream
  use is numeric**, where you blend the number and skip the category entirely.
  `exposed_litho` is the second kind. (Note the frost agent A2 reads the *same*
  outcrop for a numeric multiplier too — same resolution.)

- **B3 (coal onset).** Sits on the residue-(a)/B boundary: sim-internal retag,
  yet its spatial contact follows a burial contour and could read as a line. Left
  in B with a "low priority, mostly (a)" note rather than forcing a side.

---

## Sites in the FF2b agent's likely write-set — post-merge re-check

The FF2b/far-field agent is concurrently editing far-field/client + `dc-core`
LOD. This audit read **main's** checkout. Re-check these after its merge:

1. **`dc-core/src/materials/lod.rs` (`MixtureDownsampleRule` /
   `DominantClassDebrisAware`)** and **`dc-core/src/lod.rs` (`MajorityNonAir`)** —
   the octree reduction rules. **Directly implicated by Part 2**: these ARE the
   move-A/move-B reductions the `CoarseField<T>`/node-stratum unification is built
   on (octree-substrate.md:96-100, 116-118). If FF2b changes the reduction
   contract or `derive_material_lod_chunk`'s missing-child handling
   (octree-substrate.md:126-130, the "ungenerated is not empty" A-5 fix), the
   type's `Member` reduction path must be re-reconciled.
2. **`collapse.rs::coarse_surface` / `surface_sample`** (collapse.rs:681-770,
   852-860) — the shared near/far surface kernel. B1's cure lives *inside*
   `surface_sample`, and FF2b consumes `coarse_surface` for the far heightfield;
   the octree doc names this kernel as the legal home of the leaked
   summarization (octree-substrate.md:121-124). Any FF2b edit here collides with
   the B1 conversion — sequence them.
3. **`dc-client/src/farmesh.rs`** — no site in *this* inventory lives there, but
   it is the eye that B1/A1 changes reach; re-shoot the far-field agreement after
   both merges (heights are climate-independent, so a far sample must still land
   on the near column — journal/0022, and the `coarse_surface`↔`column` height
   agreement).

None of the ranked shortlist's *primary* files (`deeptime/lithology.rs`,
`deeptime/erosion.rs`) are in FF2b's write-set, so A1 is safe to sequence first
regardless of the FF2b merge.

---

## Provenance / commit

- Read-only over the codebase; no cargo run. Citations are to main's checkout as
  read 2026-07-22.
- Register: `docs/audits/` standing format (spines § 6), alongside the seam,
  deeptime-vector, and hydrology-priors audits.
- Branch: `worktree-agent-a69e960ab299ced1d`. Commit hash recorded in the merge.
