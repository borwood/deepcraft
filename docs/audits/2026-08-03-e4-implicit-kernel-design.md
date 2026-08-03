# E4 — the field-solver primitive and the implicit kernel: design pass

**Arc anchor:** `docs/dependency-graph.md:50` (E4, "SHAPE NAMED 2026-07-29 (S-10), 2 instances,
NOT EXTRACTED") + `:56-61` (§ "E4's rule") · `docs/spines.md:1078-1167` (S-10, RATIFIED
2026-07-29) · `docs/design/stubs.md:1295-1354` (#30, the DERIVED half still owed, heir = this
primitive) · `docs/audits/2026-08-02-p2-measurement-runs.md` header (RE-SEQUENCED 2026-08-03,
user: finding 7's **answer-shape is ruled: E4**) · `docs/ARCHITECTURE.md:748-756` (the two
divisions four lines apart).

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option is priced and
its trade-offs named; picks marked **user-owned** are the user's. Anything the agent
originated is marked **(assistant-proposed)**. **No cargo was invoked** — every number is
arithmetic over already-measured quantities cited to source, or a closed-form property of a
named discretisation, flagged as such.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (verified in code here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / stale input / needs
the user's eye).

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Read at the worktree of
`f3e5d17`. Anything that later refutes or re-scopes this file gets a banner **here**, stamped
by the author of the correction.

**Read with:** journal/0122 (the operator repair — the explicit sub-cycle, the von Neumann
derivation, and its own dated rejection of implicit, § 7.1 below) · journal/0139 (the
`erosion/` split — the pass/kernel cut this extraction lifts along) ·
`docs/audits/2026-08-02-p2-measurement-runs.md` (the forcing measurement) ·
`docs/audits/2026-08-01-p2-calibration-derivation.md` (whose § 6.2 acceptance instruments the
validation plan reuses) · `docs/design/north-star.md:77-86` (field-solver *primitives* are
core; the passes that call them are content).

---

## 0. The forcing fact, and which clock this is

The P2 ladder (`2026-08-02-p2-measurement-runs.md` § 2) measured the erosion solve's
sub-cycling scaling **linearly with the calibration multiplier M**: 2 sub-steps/epoch shipped
(M=1) → 92 @45 → 224 @100 → 336 @150 → 553 @250 → **896 @400** — measured slope
**n(M) ≈ 2.24·M** — driving Medium gen time **41 s → 2,306 s**. Every added second is
integrator tax: the sub-step count is `ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)`
(`erosion/creep.rs:181-187`), the explicit scheme's monotonicity bound (`a ≤ 1/8`,
`erosion/creep_kernel.rs:124`), and `eff_diff` is proportional to M. The user ruled the
in-band gen time *"actually unacceptable"* and the answer-shape **E4: the implicit,
unconditionally-stable kernel** (runs doc header, 2026-08-03).

**Which clock: this is GEN time, not runtime.** Nothing here touches a per-frame, per-tick or
per-chunk-load path; the "runtime is sacred" doctrine has no authority over this pass (spines
S-10 carries the user's 2026-07-29 correction of exactly that conflation). What *does* bind is
the **pregen budget**: `pregen_time_vs_extent` at **1,200 s** (renegotiated 2026-08-02, *"as
long as it doesn't take 20min"*), which M ≈ 375–400 exceeds ~2× under the explicit scheme.
⚠ FLAGGED for the build slice: spines S-10's bullet *"the cost is gen-time, which is free by
doctrine… a diagnostic, not a perf guard"* is now in tension with the user's 2026-08-03
"actually unacceptable" ruling; the S-10 entry will want a banner in the same commit as the
first E4 slice (not editable from this read-only pass).

**Partition, stated once** (graph § 0; never justified by trust): the **kernel is ENGINE** —
E4's rule: four inputs, four owners — stencil (kernel), `dx` + `dt` (engine), coefficient
field (content); only the kernel can know its own constant, and housed there **the unsafe
call is inexpressible**. The **creep PASS stays content** (every pass is content, including
erosion — read-first item 0): it declares the coefficient field, the obstacle, and what to do
with the fluxes; the kernel solves. `EROSION_CALIBRATION` is P2's constant and is
**untouched by everything in this document** — no constant is re-fitted in passing.

---

## 1. The operator's true form — measured from the code, and it decides everything

Read at `erosion/creep_kernel.rs` + `erosion/creep.rs` + `erosion/flood.rs:162-174`
(post-journal/0139 paths; the S-10 entry's `erosion.rs` line refs predate the split).

**The flux law is LINEAR in the gradient.** Per edge, on the frozen surface
`s = r + h` (bedrock datum + regolith, `build_surface`, `flood.rs:168`):

```text
F(i→j) = eff_diff(donor) · (s_i − s_j) · scale(donor)        (creep_kernel.rs:62-85)
eff_diff(i) = diffusion · (1 − bio_resist_i) · sus_creep_i   (creep_kernel.rs:42-53)
scale(i)    = min(1, h_i / Σ requested outflux_i)            (creep_kernel.rs:129-150)
```

This is **Culling linear creep** (Culling 1963) — flux proportional to slope, no critical-slope
term, no depth dependence in the flux law itself. It is *not* the Roering et al. (1999)
nonlinear law and *not* depth-dependent creep; the implicit options for those (Perron 2011's
Newton wrapping) are **not needed here**. Three complications qualify the linearity, and they
are the whole design:

1. **Heterogeneous, FROZEN coefficient.** `eff_diff` varies per cell (biotic × lithology) but
   both planes are frozen for the epoch (the biotic pass rewrites `bio_resist` *after* erosion
   — the lagged coupling, journal/0122's parenthetical). Within one epoch the coefficient is
   data, not state. Heterogeneous-but-frozen keeps the operator **linear**.
2. **Donor-upwind coefficient selection.** The edge uses the *donor's* `eff_diff` and the
   *donor's* `scale`, and which cell is the donor depends on the sign of `s_i − s_j` — a
   state-dependent switch. Mild nonlinearity; linearizable by freezing the upwind selection
   from the epoch-start surface (the exact analogue of the Braun & Willett move: *implicit in
   elevation, explicit in coefficients* — one linearized solve per step).
3. **The `h ≥ 0` obstacle, and it is NOT a corner case.** The inventory limiter is the
   discrete form of "creep moves regolith only, and a cell cannot ship what it does not
   hold." The P2 runs measure the limiter binding (`creep-lim` column) on **70.7–79.1 % of
   regolith-bearing cell-epochs at every rung including shipped** — the working regime is
   **supply-limited almost everywhere** (finding 3: the weathering front is the governor).
   Any implicit design that treats `h ≥ 0` as a rare projection is designing for a world
   that does not exist. Mathematically this makes one epoch a **linear complementarity
   problem** (an obstacle problem), not a linear solve — which is precisely why
   journal/0122 declined implicit at the time (§ 7.1).

**The species split stays OUTSIDE the kernel — established, not open.** Creep is deliberately
unsorted: *"no competence ceiling, no settling draw, no coarsest-first — every edge moves the
donor's whole composition in proportion"* (`creep_kernel.rs:152-165`), and the split is
**linear in the flux**, with a standing test that would catch sorting creeping in
(`the_creep_split_is_linear_in_the_quantity_so_nothing_is_sorted`, `creep_kernel.rs:603`).
The competence ceiling / coarsest-first drawdown of journal/0141 § order belongs to the
**stream transport** pass, not creep. Two consequences:

- The kernel moves **bulk** and returns **per-edge fluxes**; the species split rides after,
  on those fluxes, exactly as today's per-sub-step split does — and since the composition
  plane (`self.shares`) is frozen per epoch (not rebuilt between sub-steps, verified in
  `diffuse`/`diffuse_step`), splitting the epoch's summed flux once is algebraically the
  per-sub-step split summed. **No coupled solve is forced**, and the split gets ~n× cheaper
  for free (it currently runs per sub-step, `creep_kernel.rs:375-428`).
- The **flux-form return is what keeps the door open** for a future sorting consumer
  (transport): per-flux species arithmetic needs fluxes, not end-states.

**The second S-10 instance fits the same form.** `water/sat.rs`'s lateral step is the same
frozen-snapshot antisymmetric gather on head `H = y + sat`, with a **two-sided** obstacle
(donor water, receiver pore space — `l_out`/`l_in`/`frozen_space`) and a per-pair
permeability rule. It is unconsumed machinery (spines § 3), so it is a **shape check on the
API, not a build order** — but the API must leave room for an upper obstacle and a declared
per-edge coefficient rule, or the second instance is inexpressible (§ 3.4).

---

## 2. Kernel options, priced

Common to all: the kernel owns its constants; the pass hands in `dt` (authored, RATE) and the
coefficient field (content) and can express nothing unstable. Costs are stated against the
measured explicit unit: **≈12.7 ms per explicit sub-step** at Medium (derived by hand from the
P2 gen-time column: (gen(M) − 41 s) / (200 · (n(M) − 2)) = 12.3–13.7 ms across five rungs;
flagged as arithmetic over measured quantities).

### K0 — the explicit sub-cycled scheme, wrapped (the extraction baseline)

Today's kernel, unchanged, behind the E4 API. Cost stays `O(M)`: ≈2,175 s at M≈375
(41 + 200·840·0.0127). **Not the answer to the forcing fact, but it is the byte-identical
extraction target** (§ 7) and remains the reference scheme for validation forever. Its
stability bound is real and stays kernel-owned; the E4 API ships it as `Scheme::Explicit`.

### K1 — linear implicit (backward-Euler ADI), obstacle as a one-shot flux clamp — **priced to be rejected**

Solve the linear diffusion implicitly, recover edge fluxes, clamp each donor's export at its
epoch-start inventory in one pass. Cheapest implicit (~2 tridiagonal sweeps/epoch), **and
wrong in the dominant regime**: a one-shot clamp against *start-of-epoch* inventory kills the
within-epoch cascade (a bare cell that receives and re-ships in the same epoch), reinstating
the **one-cell-per-epoch conveyor** at epoch scale — the exact defect stubs #27 named and
journal/0122 explicitly refused to reintroduce (*"capping would reinstate exactly the
one-cell-per-epoch conveyor"*). With the limiter active on ~70 % of cell-epochs, this is not
an edge-case error. **Rejected on the corpus's own precedent; recorded so it is not
re-proposed.**

### K2 — backward-Euler factorized ADI + fixed Picard outer loop on the obstacle/upwind — **(assistant-proposed) RECOMMENDED**

One epoch = a small **fixed** number `k` of Picard iterations; each iteration is a
Douglas-Gunn-factorized backward-Euler solve — `(I + a·Lx)(I + a·Ly) s⁺ = rhs` — i.e. two
banks of independent tridiagonal (Thomas) solves, **direct, O(n), no inner iteration,
unconditionally stable at any coefficient**. Between iterations the upwind selection and the
per-cell export scale are re-frozen from the latest iterate under a **monotone rule** (a
cell's scale may only decrease within an epoch — a deterministic damping that prevents
limited/unlimited flip-flop). After the last iteration, per-edge fluxes are recovered from
the solved surface and **today's limiter arithmetic is applied as the final projection**, so
`h ≥ 0` and mass-exactness are guaranteed by the same bit-level argument as the current
operator, not by solver convergence (§ 4).

- **Why backward-Euler factors and NOT Peaceman–Rachford:** PR-ADI's per-direction factor
  `(1 − aλ)/(1 + aλ) → −1` as `aλ → ∞` — at a ≈ 105 per edge (M≈375) the grid-scale mode is
  reflected with amplitude ≈ 1. That is the **Crank–Nicolson disease**, and it is the same
  period-2 checkerboard this whole arc exists to exorcise, returned through an A-stable-but-
  not-L-stable scheme. The BE factors are `1/(1 + aλ) ∈ (0, 1]` — **monotone per direction at
  any coefficient**, no mode can change sign. L-stability is a requirement here, not a taste
  (textbook: LeVeque 2007, *Finite Difference Methods for ODEs and PDEs*, § on A- vs
  L-stability).
- **Known weakness, named:** Douglas-Gunn factorization carries a splitting error
  (`a²·Lx·Ly` cross-term) that grows with `a` and is **axis-aligned** — a potential
  grid-anisotropy, which is S-4's square-phenomenon class. This is exactly what the § 5
  convergence study's rotated-ridge fixture exists to measure; if it fails, the kernel's own
  **accuracy sub-cycling** (§ 3.2) divides `a` down at `O(M)` slope but a ~80× cheaper
  constant than the explicit bound, or K2-alt takes over.
- **Cost:** per Picard iteration ≈ 2 tridiagonal banks + flux recovery ≈ 2–4 explicit
  sub-step equivalents; at fixed `k = 4`: ≈ 100–200 ms/epoch → **20–40 s per 200-epoch
  Medium run, independent of M**.
- **Literature anchors, cited honestly:** the *implicit in elevation, explicit in
  coefficients, one linearized solve per step* move is Braun & Willett (2013, Geomorphology
  180–181:170–179) — **note: their celebrated O(n) implicit trick is for the ADVECTIVE
  stream-power term ordered along the D8 receiver tree, not for diffusion**; the hillslope-
  diffusion term in that same FastScape lineage is solved by **ADI** (Braun's FastScape;
  fastscapelib's diffusion solver), and ADI for earth-surface diffusion is standard practice
  (Pelletier 2008, *Quantitative Modeling of Earth Surface Processes*; Peaceman & Rachford
  1955 for the method family; Douglas & Gunn 1964 for the factorized BE form). Landscape
  evolution models run implicit at exactly these rates — that is the lineage the user's
  ruling names, applied to the correct term.

### K2-alt — exact 2D sparse direct solve per Picard iteration — **(assistant-proposed) the no-anisotropy fallback**

Replace the factorized solve with an exact pentadiagonal SPD solve (sparse Cholesky, nested
dissection, fixed elimination order → deterministic). No splitting error, no axis bias, same
outer Picard/projection structure. Cost: factor + solve per iteration on a 545² grid is
sub-second but likely 3–10× K2's per-iteration cost, and it means either a new engine
dependency (e.g. `faer`, pure Rust) or bespoke banded-solver code — a dependency-policy call.
Take only if the § 5 study measures unacceptable anisotropy in K2.

### K3 — projected Gauss–Seidel / projected multigrid on the LCP — **priced as the "textbook-correct" heavy option**

Solve the obstacle problem properly (Cryer 1971 PSOR; Brandt & Cryer 1983 projected
multigrid): complementarity holds at convergence, the within-epoch cascade is exact, no
splitting error. Costs: plain PSOR's contraction is hopeless at these coefficients (diagonal
dominance ratio 4a/(1+4a) ≈ 0.998 at a≈105 — thousands of sweeps for smooth modes);
projected **multigrid** is the real O(n) version but is the largest new machinery on this
menu (coarsening, transfer operators, FAS cycles), the hardest determinism story (still
achievable with fixed cycles/orders), and the most code for the same physics K2 + final
projection delivers. **Not recommended for the first cut; named so a future consumer that
genuinely needs exact complementarity (a two-sided obstacle at high coefficient?) knows where
the road continues.**

**Recommendation (assistant-proposed, user-owned pick U-1):** **K2**, with K0 retained as
the reference scheme and K2-alt as the measured fallback. K1 rejected. K3 deferred.

---

## 3. The E4 primitive's API

### 3.1 Shape (assistant-proposed sketch — names illustrative, the contract is the content)

```rust
// engine side — see § 3.5 for where it lives
pub enum Stencil { FourNeighbour /* , EightNeighbour, … */ }   // kernel-owned constants
pub enum Scheme  { Explicit, ImplicitBE { picard: u8 } }        // kernel-owned numerics

pub struct DiffusionProblem<'a> {
    pub w: usize,                    // grid width; dx is the engine's (folded per § 3.3)
    pub state: &'a [f64],            // the conserved quantity (h)
    pub datum: Option<&'a [f64]>,    // potential = datum + state (r + h); None ⇒ state itself
    pub coeff: &'a [f64],            // per-cell coefficient, CONTENT-owned (eff_diff plane)
    pub coeff_rule: CoeffRule,       // DonorUpwind | PairMin | …  (declared, kernel-executed)
    pub lower: Option<Obstacle>,     // state ≥ 0 (creep inventory) — the donor limiter
    pub upper: Option<&'a [f64]>,    // per-cell capacity (sat.rs pore space)
    pub dt: f64,                     // authored phase length, ENGINE-owned (RATE)
}

pub struct EdgeFluxes { /* east + south planes; antisymmetric by construction */ }
pub struct SolveReport {
    pub steps: u32,          // explicit sub-steps taken, or Picard iterations run
    pub peak_coeff: f64,     // what the kernel divided down / solved at (diagnostic)
    pub residual: f64,       // final projection's relative correction (implicit only)
}

impl FieldKernel {
    /// Integrate one authored `dt` and return the epoch's antisymmetric edge fluxes.
    /// The caller applies them (and may split them by species, tally ledgers, …).
    pub fn solve(&self, p: &DiffusionProblem, out: &mut EdgeFluxes) -> SolveReport;
}
```

The pass keeps: applying fluxes to `h`/`dh`, the species split on the returned fluxes, the
denudation-ledger tallies, the creep-to-sea tally, the itemisation audits. The kernel takes:
the surface freeze, the limiter arithmetic, the sub-step/iteration decision, the bound.

### 3.2 What the kernel PROMISES, and what a pass DECLARES

Under the explicit scheme the kernel's promise is the S-10 ruling verbatim: *takes `dt` from
outside, derives its own internal multiplier to stay inside its own bound* — the stability
bound is the kernel's. Under an unconditionally stable scheme **"stability bound" becomes
"accuracy bound"**, and the promise re-types:

- **Promised unconditionally (any `dt`, any coefficient):** no mode changes sign
  (monotonicity — the checkerboard is inexpressible); `state ≥ 0` after application (bit-
  guaranteed by the final projection, § 4.1); exact mass conservation (flux-form, § 4.1);
  bit-determinism scalar↔parallel (§ 4.2).
- **Promised with a stated bound (the accuracy contract):** agreement with the well-resolved
  explicit reference within the § 5-derived bound. The kernel carries an **accuracy
  divisor** — `n_acc = ceil(peak_coeff / A_ACC)` implicit solves per epoch, with `A_ACC` a
  kernel constant **derived by the convergence study, not fitted** (gates doctrine: a bound
  with a derivation is evidence). If the study finds `n_acc = 1` acceptable at M≈400,
  `A_ACC = ∞` and cost is exactly O(1) in M; if not, cost is O(M) with a measured ~10–80×
  cheaper constant than the explicit bound. Either way the *decision lives in the kernel*
  and no pass author ever sees it.
- **A pass declares:** the coefficient field and rule, the obstacle(s), the datum — physics.
  It cannot state an integration step, a sub-step count, or an iteration count. **The unsafe
  call is inexpressible**, same move as `CoarseField` making the raw per-cell read unsayable.

### 3.3 The four inputs, the four owners — and where `dx` actually is today

The graph's rule assigns `dx` to the engine. Today `cfg.diffusion` is already **per-edge
dimensionless** (dx² folded into the authored constant; the grid's 460 m never appears in the
kernel). The extraction should **state this in the API doc** rather than silently normalise:
first cut keeps the folded convention (byte-identity requires it); the honest heir is a
`dx`-aware signature when a second grid pitch exists to force it. ⚠ FLAGGED as a recorded
deferral, not a decision.

### 3.4 Both S-10 instances expressible, plus the third-consumer sanity check

- **Creep** (`Erosion::diffuse`): `state = h`, `datum = r`, `coeff = eff_diff` plane,
  `coeff_rule = DonorUpwind`, `lower = Some(inventory)`, `upper = None`. Verified against
  `creep_kernel.rs:62-150` — every term of `diffuse_net_cell`/`diffuse_scale_cell` maps.
- **Bound water lateral** (`water/sat.rs`): `state = sat·porosity` per layer, `datum = y`,
  `coeff` from permeability with a **pair** rule, `lower` = water present, `upper` = pore
  space (`frozen_space`). Expressible given `CoeffRule::PairMin` and the `upper` obstacle —
  the two API features creep alone would not have forced. f32/3D adaptation is the
  *conversion's* cost, not the contract's; and since the water module is unconsumed
  (spines § 3), the conversion is a shape-proof slice, **not** on the critical path.
- **Third future consumer, sanity check:** P7 metamorphism / thermal diffusion (the geotherm
  family) is pure linear diffusion, no obstacle, no upwinding — the trivial case
  (`datum = None`, `lower = None`). The head field (P3) likewise. **The API does not bend to
  admit them; they are its degenerate case** — the shape check passes. (Stream-power
  incision is *advective* and is NOT this primitive — its implicit form is the per-node
  Newton walk along the receiver tree, a different kernel family; named so E4 is not asked
  to absorb it later.)

### 3.5 Where it lives — user-owned pick U-3

- **(a) `dc-core::field`** — the engine crate, precedent `dc_core::coarse::CoarseField`
  (E5 member #0 lives there). The kernel is pure slices + width, no worldgen deps; clean.
  **Recommended (assistant-proposed).**
- **(b) staged in `dc-worldgen::deeptime::field`** first, crate-move later — one fewer
  crate-boundary in the first diff, but it builds the engine primitive inside the pack crate
  and owes a second move; the A-1 grain ("extend the existing shapes") cuts *toward* (a),
  since `erosion/creep_kernel.rs` already isolated the kernel and the move is the extraction.

---

## 4. Conservation and determinism

### 4.1 Law-3 exactness — the flux-form contract does the work

The kernel **returns per-edge antisymmetric fluxes; it never returns a new state.** The pass
applies them, so every unit that leaves one cell arrives in exactly one other — mass-exact by
the identical bit-level argument as today (IEEE-754 subtraction is exactly antisymmetric,
`creep_kernel.rs:625-639`). For the implicit scheme, fluxes are recovered from the solved
surface (`F = a_donor·(s_i⁺ − s_j⁺)`) and then passed through **today's limiter arithmetic as
the final projection** — so `h ≥ 0` and conservation are guaranteed by the projection's
arithmetic, not by solver convergence. The solver only decides *accuracy*; it structurally
cannot leak mass or mint negative regolith. (The projection's relative correction is the
`SolveReport::residual` — a diagnostic and a gate assertion, § 5.)

**What replaces the per-sub-step budget itemisation:** it collapses. Today a sub-cycled epoch
accumulates `netdiff_acc` so the species audit closes against the summed ΔH
(`creep.rs:191-211`, `creep_kernel.rs:469-509`). Under flux-form there is **one flux set per
epoch**: the itemisation audit closes against the applied fluxes' net directly, and
`netdiff_acc` retires. The ledger instruments change *definition* honestly and it must be
said out loud: `diffused_m` (gain-side sum) and `creep_to_sea` are currently per-sub-step
tallies on intermediate surfaces; per-epoch fluxes give the **net** epoch tally, which is not
the same number when a cell's flux direction flips mid-epoch. ⚠ FLAGGED: the build slice's
probe captions and the flux-record face-count economics (stubs #18 — currently counted per
sub-step *on purpose*, `creep_kernel.rs:422-427`) must be re-worded with the semantics, per
the "a printed caption is a published claim" gate rule.

### 4.2 Bit-determinism

- **Explicit scheme:** unchanged; already asserted bit-identical scalar↔parallel
  (`the_sub_cycled_operator_is_bit_identical_scalar_and_parallel`, `creep.rs:564`).
- **Implicit scheme:** every ingredient is fixed-order. Thomas solves are sequential per
  line; lines are independent with disjoint writes, so parallel-across-lines is bit-identical
  to scalar-in-line-order by construction. The Picard count is **fixed** (`Scheme::ImplicitBE
  { picard }` — a kernel constant, not a convergence exit); the monotone scale rule is a
  deterministic per-cell update; the final projection is today's deterministic limiter. **No
  convergence-dependent early exit anywhere.** If a tolerance-based exit is ever wanted, the
  gates doctrine's price applies — a derived bound, computed with a fixed-order reduction —
  but the first cut should not pay it: a fixed count is testable, explainable, and cheap.
- The scalar↔parallel bit-identity test carries over to the implicit arm verbatim.

---

## 5. Accuracy against the explicit reference — the validation plan

**The reference is the explicit sub-cycled solve at low-to-mid M, where it is well-resolved**
(a/n ≤ 1/8 by construction). The implicit kernel owes agreement within a **derived** bound.

1. **Fixture-level convergence study** (no world): the journal/0122 checkerboard + a smooth
   hill + a **rotated ridge** (45°) fixture, explicit-reference vs `ImplicitBE` at
   `n_acc ∈ {1, 2, 4, 8}`, coefficients spanning a ∈ {0.5, 5, 50, 105}. Metrics: L∞/L2 on
   `h`, and the anisotropy diff between the axis-aligned and rotated ridge after equal
   integration (the S-4 square-phenomenon detector — this is the measurement that decides
   between K2 and K2-alt, and what `A_ACC` is derived from).
2. **World-level agreement** at M ∈ {45, 150}: one explicit and one implicit 200-epoch
   Medium run each, compared on the **P2 acceptance instruments** (D3, mean H, ⟨taper⟩,
   conc(h) rms, ACF(1), hollows, D3/D4) — the quantities a ratification actually reads. The
   agreement bar on D3 is proposed at the width the ladder's own rung-to-rung noise defines
   (assistant-proposed; the bar itself is **user-owned pick U-4**, since it decides what
   "same physics" means for the flip).
3. **Who carries it:** `creep_operator_probe` (exists, `examples/creep_operator_probe.rs`)
   gains an implicit arm; `test = true` per the probe doctrine. Gate assertions are
   **scale-free invariants, never snapshots**: mass exact to 1e-12 relative; no sign flip on
   the checkerboard fixture (reuse
   `the_calibrated_rate_has_an_exact_period_2_mode_and_the_bound_removes_it`'s structure);
   scalar↔parallel bit-equality; projection residual ≤ the derived bound; explicit↔implicit
   L∞ agreement ≤ the derived bound **on a small grid at a mid coefficient** (the invariant
   is a property of the discretisation, not of world size — say so in the doc comment). The
   world-scale M-ladder numbers stay in the report, not the gate.

**Goldens — expected-red families and the re-capture discipline.** At equal physics the
trajectories **will differ**: a different integrator is a different float path (corrections
#89's class — the addends change, not merely their grouping — and here the scheme genuinely
changes, so this is stronger than accumulation order). *If and when the implicit scheme
becomes the default* (§ 7's E4-3), the expected-red set is every family downstream of
terrain: `GOLDEN_SURFACE`/`GOLDEN_RECORD`, `GOLDEN_FLUX`, the chunk/contents goldens, the
providers goldens — the same 14-family set journal/0139/0141 re-captured. House discipline
applies verbatim: **ratified semantics → re-capture once with the why recorded; no byte
ratification loop; never argue from the fixture's current bytes.** The unbounded-creep
goldens (`GOLDEN_*_UNBOUNDED_CREEP`) and the explicit-scheme fixed point stay green
throughout — the explicit path remains reachable and hashed, exactly as journal/0122 kept
`creep_substep: false` reachable.

---

## 6. The win, projected honestly

**Cost model** (hand arithmetic over the P2 measured rows, flagged as such):

| | per epoch | 200-epoch Medium, M≈375 |
|---|---|---|
| explicit (measured trend) | n ≈ 2.24·M sub-steps × 12.7 ms ≈ 840 × 12.7 ms | ≈ **2,100–2,200 s** (2,306 s measured @400) |
| implicit K2, k=4, n_acc=1 | ~8–16 sub-step equivalents ≈ 100–200 ms | **≈ 60–80 s** (≈38 s non-creep baseline + 20–40 s solve) |
| implicit K2 if the study forces n_acc≈10 | ~1–2 s | **≈ 140–250 s** |

- **The P2 pregen-budget conflict (finding 7) dissolves in every projected case**: 60–250 s
  against the 1,200 s budget, versus ~2,200 s explicit. The § 5 register↔Myr lever stays
  untouched and un-consumed by this design — it remains available for physics reasons, no
  longer needed for schedule ones.
- **The deferred pit safari becomes cheap**: a high-M world drops from ~38 min to ~1–4 min
  of gen, which is what the user's re-sequencing ruling anticipated (*"the safari runs when
  worlds are cheap (post-E4)"*).
- **The M re-pick (finding 5) gets a cheap ladder**: re-running the eight-rung ladder costs
  ~10–30 min total instead of ~90.

**What it does NOT fix, stated so nobody buys more than is for sale:**

- **The ~41 s non-creep baseline** — untouched; E4 removes the M-scaling term only.
- **The pits-bar conflict (finding 6)** — hollows >10 m at in-band rungs are
  transport/routing/fill physics, not integrator tax; the walk-first ruling on them stands.
  E4 must not be reported as improving them (it may *change* the counts, since trajectories
  differ — which is a re-measurement, not a fix).
- **No other pass has M-dependent sub-cycling — measured, not assumed.** Grep over
  `deeptime/` finds sub-cycling only in creep (`creep.rs`/`creep_kernel.rs`; the other hits
  are the cadence axis and docs), and the runs table proves it from the other side: the
  `150 UNB` control (M=150, n=1) generates in **37.8 s** against the shipped 41.2 s — at
  n=1, gen time is flat in M. Stream transport and weathering scale their *magnitudes* with
  M, not their step counts (their owed `1 − exp(−k·dt)` conversions are graph item 6,
  rides with P2, not with E4).
- **⚠ The D3(M) ≈ 0.0070·M curve was measured under the explicit integrator.** The implicit
  trajectory will land near it (both converge on the same operator) but not on it; the
  **M ≈ 375–380 target must be re-confirmed with one cheap post-E4 ladder run before the
  flip is ratified**. This is owed to P2's finding-5 conversation, and it is cheap precisely
  because of E4.

---

## 7. Sequencing — extraction-first, and why

### 7.1 The journal/0122 implicit rejection, re-priced (not contested — re-measured)

journal/0122 considered implicit and declined it: *"`h ≥ 0` makes it an obstacle problem
rather than a linear solve… an iterative solve needs tens of sweeps: the same cost, for a
much larger surface of things to get wrong."* Every clause was true **at n ≈ 100 (M=45)**.
The P2 ladder moved the forcing an order of magnitude: at n = 896 the explicit scheme is
~2,300 s and "the same cost" is false by ~30×. The rejection was a dated engineering call
inside a slice, not a user ruling; the user's 2026-08-03 answer-shape ruling supersedes it
explicitly. **No ⚠ CONTESTS arises** — recorded here so the reversal is loud, dated, and
attributed to a measurement rather than to taste. (The clause of 0122 that *does* survive
untouched is the flux-recovery observation: *"exact mass conservation through an iterative
solve means recovering the fluxes and re-applying them antisymmetrically anyway"* — § 4.1 is
that sentence, promoted to the design.)

### 7.2 The two orders, priced — user-owned pick U-5

- **Extraction-first (recommended, assistant-proposed):** E4-1 lifts today's explicit kernel
  behind the API **byte-identically**, then E4-2 adds the implicit scheme beside it.
  - For: seam-first is the north star's own method (*"pursued evolutionarily via seam-first
    conversions"*); journal/0139 already made the cut clean (*"the two halves share nothing
    but the struct… that is the pass and the operator"* — finding 1), so the extraction is
    small and its gate is cheap (goldens green, no re-capture); stubs #30's heir language is
    the extraction verbatim (*"hoisting the derived multiplier into the S-10 field-solver
    kernel"*) and discharges the day E4-1 merges; the golden churn concentrates in one
    later, ratified adoption slice; A-1 is satisfied — the primitive is the existing
    `creep_kernel.rs` shapes moving, nothing built beside them.
  - Against: one extra merge of latency before the implicit work starts.
- **Kernel-first-in-place:** build the implicit solve inside `creep.rs`, extract later.
  - For: shortest path to a fast high-M world for P2's conversations.
  - Against: touches the same 650 lines twice; builds an engine-shaped mechanism *inside a
    content pass* the day after the corpus ruled the opposite placement; leaves `sat.rs` and
    every future diffusing pass without the primitive in the interim; the interim state is
    A-1's "built beside" wearing a schedule. **Not recommended.**

### 7.3 Proposed slices (assistant-proposed; gate plans per the batching rule)

1. **E4-1 — the extraction.** `creep_kernel.rs`'s gather kernels + bound + sub-cycle driver
   move behind the `FieldKernel` API (venue per U-3); `Erosion::diffuse` becomes: build
   problem → `solve` → apply fluxes → split species → tally → audit. **Byte-identical**
   (`Scheme::Explicit`, same arithmetic, same order) — asserted by the full golden set
   staying green, which IS the gate; plus fmt/clippy on changed crates. Discharges stubs
   #30; spines S-10 gains "EXTRACTED" with the site; graph E4 row moves; the S-10 gen-time
   bullet gets its § 0 banner. sat.rs is *not* converted here.
2. **E4-2 — the implicit kernel, not default.** `Scheme::ImplicitBE` lands behind the same
   API; fixture tests + the probe's implicit arm + the § 5.1 convergence study; `A_ACC` and
   the agreement bound derived and recorded. No golden moves (nothing ships through it).
   Cheap-evidence merge permitted (same arc), full workspace gate before the arc pauses.
3. **E4-2b (optional, cheap) — the sat.rs shape-proof.** Convert the lateral step; proves
   `PairMin` + `upper`. Unconsumed module, zero golden risk. Can ride E4-2 or wait.
4. **E4-3 — adoption.** User ruling on U-2 (below); if adopted, the deep solve's default
   scheme flips, all 14 golden families re-capture **once** with the why recorded, the § 6
   ladder re-run lands beside it, and its numbers go to P2's finding-5 conversation. **This
   slice should be sequenced against the P2 flip** so the world's goldens move once for
   both, not twice.

### 7.4 Adoption shape — user-owned pick U-2

- **(a) Always-implicit once adopted (recommended):** one code path for the shipped world;
  goldens re-capture once; the explicit scheme survives as the validation reference and the
  `creep_substep`-style reachable fixed point.
- **(b) Regime switch (explicit when n ≤ some N, implicit above):** keeps the shipped-world
  (n=2) goldens byte-stable through E4 entirely — at the price of two live paths forever and
  a trajectory discontinuity at the switch coefficient, a fixture-bytes argument the
  scratch-world doctrine explicitly declines to privilege. Priced for completeness; not
  recommended.

---

## 8. Picks, originations, contests

**User-owned picks (all NEEDS RATIFICATION; recommendations are assistant's):**

| # | pick | recommendation |
|---|---|---|
| U-1 | kernel scheme | **K2** (BE-factorized ADI + fixed Picard + today's-limiter final projection); K2-alt on measured anisotropy; K1 rejected; K3 deferred |
| U-2 | adoption shape | always-implicit at E4-3, goldens re-captured once, sequenced with the P2 flip |
| U-3 | where the primitive lives | `dc-core::field` (CoarseField precedent) |
| U-4 | the explicit↔implicit agreement bar on the P2 instruments | derive from the ladder's rung-to-rung noise; the bar is a physics-meaning call, hence user-owned |
| U-5 | sequencing | extraction-first (E4-1 → E4-2 → E4-3) |

**Assistant originations** (marked at site): the K-option menu and K2's monotone-Picard +
final-projection construction; the API sketch of § 3.1; `A_ACC` and the accuracy-divisor
framing; the § 5 study design; the § 6 cost model arithmetic; the slice plan.

**⚠ CONTESTS: none found.** Checked against: the user's 2026-07-29 S-10 ruling (*kernel takes
`dt` from outside and owns its own bound* — the implicit kernel is that ruling with the bound
re-typed stability→accuracy, and the 2026-08-03 answer-shape ruling names it); RATE's charter
(untouched — RATE is not expanded); S-10's *"do NOT make substepping locally adaptive"* (the
implicit kernel is not adaptive substepping; the accuracy divisor is global, like today's
`n`); journal/0122's implicit rejection (an assistant slice decision, superseded by
measurement + the user's own ruling — § 7.1, loud); `EROSION_CALIBRATION` (untouched);
the pits bar (explicitly not claimed, § 6). Near-miss worth naming: the spines S-10
"gen-time is free / diagnostic not perf guard" bullet vs the user's 2026-08-03
"unacceptable" — resolved as a doc-banner obligation (§ 0), not a contest, because the user
authored both ends and the later one wins.
