# E4-2 — the implicit field-solver scheme, and its convergence study

*2026-08-04 · the build half of E4-2 · executes
[`2026-08-03-e4-implicit-kernel-design.md`](2026-08-03-e4-implicit-kernel-design.md) § 5,
under that document's five ruled picks (U-1 … U-5, all closed 2026-08-04)*

> ## ⚠ THE CODE THIS DOCUMENT MEASURES IS **NOT ON MAIN** (integrator, 2026-08-04 wrap)
>
> This file was rescued to `main` because its **measurements are durable** and the corpus
> should not lose them; `Scheme::ImplicitBe` itself lives **unmerged** on branch
> `worktree-agent-a787b0d7fdeb61b53` (worktree kept). Nothing on main reaches the implicit
> arm — the shipped scheme is `Explicit`, unchanged.
>
> **The decision this document forces, owed to the user next session:** at the binding
> `A_ACC = 0.85` the implicit arm measured **2.2× SLOWER than the explicit scheme it was
> ruled to replace** (U-1, K2), and the only arm passing both structural fixtures is the
> expensive `picard = 4` one. So the choice is **re-price K2-alt vs K3 on these numbers**
> (U-1's ratification explicitly pre-committed to neither) **or park E4-2** and let the
> explicit scheme keep the world at a gen-time cost the ready-made-worlds doctrine already
> tolerates. `IMPLICIT_ACCURACY_MAX_EDGE_COEFF` was correctly left as the flagged
> placeholder — nothing was re-fitted to make a verdict come out.
>
> **§ 5's world runs are genuinely unrun** (the slot never freed for them), and § 3.2/§ 4
> were measured on a validated scalar mirror rather than through `creep_operator_probe`.


> *(Mutable header, per the immutable-body convention — CLAUDE.md read-first item 5.
> Nothing above this line is testimony; everything below it is dated measurement.
> Anything that later refutes or re-scopes this file gets a banner **here**, stamped by
> the author of the correction.)*
>
> # ✅ THE KERNEL HAS NOW BEEN COMPILED — and it moved this document's centre of gravity
>
> **2026-08-04, later the same day.** The build slot freed, the kernel compiled, and the
> two things this file could not know are now known. **Read § 3.1 and § 3.3 before
> anything else; the rest of the header below is the state it was written in and is kept
> because its "what an integrator must NOT do" list is still correct.**
>
> - **Monotonicity is NOT unconditional, and the cause is not the one § 1 predicts.** The
>   `solve → flux recovery → limiter-as-projection` chain is monotone at every coefficient
>   tested, to `a = 1e8`, and there is a closed form saying why. What breaks it is the
>   **Picard relinearization**, at a measured, grid-independent **`a = 2.0`**. § 3.1.
> - **`A_ACC` has two candidate values and they disagree by 2.4×**: the *monotonicity*
>   bound is **2.0**; the *anisotropy* bound this file's § 3.2 defines is **0.85**. The
>   binding one is 0.85, and at 0.85 the arm is **2.2× SLOWER than the explicit scheme it
>   replaces** at `picard = 4`. § 3.3 and § 6.
> - **`picard = 1` is not the escape it looks like.** It is the arm that keeps
>   monotonicity, but § 4's cascade shows it **overshoots the max principle by 150 m** on a
>   supply-limited chain. The Picard loop is load-bearing after all. **The only arm that
>   passes both structural fixtures is the sub-cycled `picard = 4` one** — which is the
>   expensive one. § 4 and § 8.
> - **`IMPLICIT_ACCURACY_MAX_EDGE_COEFF` is still the flagged placeholder.** Nothing was
>   re-fitted. Which number it takes — and whether K2 survives the answer — is a user
>   ruling, and § 8 states it as one rather than making it.
>
> # 🛑 INCOMPLETE — NOT A RESULT YET, AND THE KERNEL HAS NOT BEEN COMPILED
>
> *(superseded in part by the banner above — the compile happened, and § 3 and § 4 are
> measured; **§ 5's world runs are still genuinely unrun**, and § 3.2 / § 4 were measured
> on a validated scalar mirror rather than on `creep_operator_probe`)*
>
> **2026-08-04. The build slot was held continuously by other sessions for the whole
> of this slice**, so **not one line of `Scheme::ImplicitBe` has been through a
> compiler**, and **every `PENDING` below is genuinely unmeasured.** What IS complete
> and standalone:
>
> - **§ 2, the agreement bar** — arithmetic over the P2 ladder's published rows. No
>   cargo needed, no `PENDING`, and it is usable by the next session as-is.
> - **§ 3.1's closed form** and **§ 4's cascade-cap argument** — derivations, flagged
>   DERIVED, each with a fixture written to falsify it.
> - **§ 6's cost model** — arithmetic, with the break-even that decides whether K2 is
>   worth having stated explicitly (`A_ACC ≈ 1.9`).
>
> **What an integrator must NOT do:** merge this, or quote any number from it as a
> measurement. `IMPLICIT_ACCURACY_MAX_EDGE_COEFF` currently holds a **placeholder**
> (`A_ACC_PENDING_MEASUREMENT = 0.5`) that is a guess from § 3.2's closed form and has
> never been checked against the fixture that is supposed to derive it.
>
> **The order of work when the slot frees** (~10 min gets the first three):
> `check -p dc-core --all-targets` → `check -p dc-worldgen --all-targets` → `fmt` →
> `run --example creep_operator_probe -- --fixtures` (derive `A_ACC`, fill § 3) →
> the gate → `-- --world` (§ 5, ~30 min: explicit at M=45 is 288 s and at M=150 is
> 883 s by the P2 ladder, plus four implicit arms).

**What this is.** `Scheme::ImplicitBe` — the audit's **K2**, as ruled — landed in
`dc_core::field`, and this is the study U-1's ratification amendment ordered: the
fixture-level convergence work, the two structural fixtures (**anisotropy**,
**cascade**), the world-level agreement at M ∈ {45, 150} against the derived bar, and
the measured cost.

**What this is NOT.** Nothing ships through the new scheme. `DeepConfig::creep_scheme`
defaults to `Scheme::Explicit`, **no golden moved**, and no constant was re-fitted —
`EROSION_CALIBRATION` is untouched, exactly as the design says. Adoption is E4-3 and is
a user ruling (U-2, always-implicit, sequenced with the P2 flip).

**Status legend:** **MEASURED** (a number produced by a named command on a named commit)
· **DERIVED** (arithmetic or closed form over measured/published quantities, flagged as
such) · **CANNOT DETERMINE** (asked, not answered, and why) · ⚠ **FLAGGED** (needs the
user's or the integrator's eye).

---

## 0. Provenance

- **branch commit:** `PENDING` (worktree `agent-a787b0d7fdeb61b53`, branched from
  `74bb6c1`, which is `main` at dispatch)
- **machine:** the one machine, `CARGO_BUILD_JOBS=4`, `--release`, shared
  `CARGO_TARGET_DIR`
- **fixtures:** `cargo run --release --example creep_operator_probe -- --fixtures`
  (no world, no seed — every fixture is deterministic arithmetic on a stated initial
  condition)
- **world runs:** `cargo run --release --example creep_operator_probe -- --world`
  — seed **1337**, `Extent::Medium` (545² = 297,025 cells at 460 m), **200 epochs**,
  `denudation_ledger: true`, `calibrated_rates: false`, `erosion_budget: M`,
  `creep_substep: true`; the **only** difference between the paired arms is
  `DeepConfig::creep_scheme`
- **gate:** see § 7

---

## 1. What was built

`Scheme::ImplicitBe { picard, accuracy }` in `dc_core::field`
(`crates/dc-core/src/field/implicit.rs`), reached through the **same**
`plan → step × n → apply` driver loop the explicit scheme uses — the plan carries its
own scheme, so a caller's loop does not branch. One epoch under the implicit scheme is:

1. **`(I + L) s⁺ = s`**, factorized as `(I + Lx)(I + Ly) s⁺ = s` — two banks of
   independent tridiagonal (Thomas) solves, direct, O(n), no inner iteration. The whole
   epoch's rate is already inside the content-declared coefficient, so `L` is
   dimensionless and there is no `dt` in the kernel (the E4 ownership rule, unchanged).
2. a **fixed `picard` outer loop** re-freezing the donor-upwind selection and the
   per-cell export limiter from the latest iterate, under the **monotone rule** (a
   cell's limiter may only decrease within an epoch). The limiter starts **open**, so
   the sequence approaches the obstacle's fixed point **from above** — each pass
   tightens a limiter the previous pass measured on a surface that was too flat.
3. **the explicit scheme's own limiter arithmetic as the final projection**, run on the
   *solved* potential. Mass-exactness and `h ≥ 0` therefore hold by the same bit-level
   argument as the current operator — one stored antisymmetric edge value, an export
   capped at the cell's inventory — and **never by solver convergence**.

**Assistant originations, marked at site:** the transposed y-bank (a pure `f64` copy, so
no bits move) so both banks run on contiguous lines and the parallel arm is trivially
the scalar arm's bits; the closed-form Nyquist diagnosis in § 3.1; the
`Accuracy::Derived | Fixed` split, which is what lets the study vary `n_acc` while
keeping it unstateable by a production pass.

**One pass-side consequence, called out because it is a real semantic change.** Under
the implicit scheme the epoch's fluxes are `c_e·(s⁺_i − s⁺_j)` — recovered from the
**solved** surface, not the frozen one. Every pass-side instrument that re-derives an
edge flux from a potential drop therefore had to be pointed at the solved surface
(`Workspace::solved_potential`): the creep species split, the creep-to-sea tally, and
the outflux-face counter. Reading the frozen surface after an implicit solve would
attribute a flux the kernel did not compute, and the species itemisation audit would
report a leak that is really an instrument reading the wrong plane.

---

## 2. The agreement bar, derived

U-4 ruled the bar as **instrument-meaning continuity**, mechanised as: *explicit↔K2 at
the same M must differ by less than adjacent ladder rungs differ.*

**The derivation, stated once.** Every explicit-era statement taken off the P2 ladder
(`2026-08-02-p2-measurement-runs.md` § 2) has the form *"at rung A this instrument reads
X and at rung B it reads Y, and that difference is informative."* The **smallest**
adjacent-rung gap an instrument exhibits across the ladder is therefore the **finest
distinction that instrument was ever asked to resolve**. If changing the integrator
moves it by less than that, no ladder statement changes its truth value and every
explicit-era number keeps its meaning. If it moves by more, at least one distinction the
ladder drew is smaller than the integrator's own effect, and the ladder must be re-run
under K2.

**DERIVED** — arithmetic over the P2 ladder's own rows (M ∈ {45, 100, 150, 250, 400}),
nothing fitted:

| instrument | adjacent-rung \|Δ\|: 45→100, 100→150, 150→250, 250→400 | **bar = min** | median |
|---|---|---|---|
| D3 (m/Myr) | 0.3842, 0.3513, 0.6877, 1.0393 | **0.3513** (49.7 % rel.) | 0.5359 |
| D4 (m/Myr) | 0.3664, 0.3354, 0.6572, 0.9950 | **0.3354** (31.1 % rel.) | 0.5118 |
| D3/D4 | 0.21, 0.09, 0.09, 0.07 | **0.07** | 0.09 |
| mean H (m) | 0.24, 0.11, 0.70, 0.37 | **0.11** (6.7 % rel.) | 0.305 |
| ⟨taper⟩ | 0.0005, 0.0022, 0.0136, 0.0116 | **0.0005** | 0.0069 |
| conc(h) rms | 0.02, 0.07, 0.51, 0.72 | **0.02** | 0.29 |
| ACF(1) x | 0.027, 0.057, 0.078, 0.032 | **0.027** | 0.0445 |
| ACF(1) y | 0.261, 0.208, 0.347, 0.262 | **0.208** | 0.2615 |
| hollows > 10 m | 1, 20, 373, 8472 | **1** | 196.5 |

⚠ **Two of these bars are pathologically tight and must be read as such, not applied
mechanically.** ⟨taper⟩'s 0.0005 and conc(h)'s 0.02 both come from the 45→100 pair,
where the ladder measured *flatness*, not a distinction — the ladder's own published
claim there is a **band** (P2 finding 2: *"run-mean ⟨taper⟩ = 0.80–0.83 at every working
rung"*), width **0.027**. Where the ladder's statement is a band rather than a
comparison, the band's width is the honest bar and the min-gap is a coincidence of two
adjacent rungs happening to agree. Both readings are reported below. The same applies to
`hollows > 10 m`, whose min gap of 1 count is a rounding artefact of the two lowest
rungs; the P2 bar on hollows was in any case **retired as a blocker** (corrections #98,
2026-08-04), so it is reported and not scored.

**The median column is not a second bar.** It is here so a reader can see how much of
the min is a coincidence of one adjacent pair.

---

## 3. Fixture-level convergence (audit § 5.1)

*Command: `cargo run --release --example creep_operator_probe -- --fixtures`. No world,
no seed; each fixture is deterministic arithmetic on a stated initial condition.
Coefficients span the ladder: `a = 105` is the M ≈ 400 rung's peak per-edge coefficient,
**840× past the explicit monotonicity bound**, where the explicit scheme needs 896
sub-steps.*

### 3.1 The grid-scale (Nyquist) mode — and the study's sharpest structural finding

**DERIVED (closed form), then MEASURED.** On the pure grid-scale mode at a uniform
per-edge coefficient `a`, with `Lx`, `Ly` eigenvalues `4a` and `L`'s eigenvalue `8a`,
one epoch's **state** amplification is

```text
    exact backward Euler   g = 1 / (1 + 8a)             → 0    as a → ∞
    product-form ADI       g = (1 + 16a²) / (1 + 4a)²   → 1    as a → ∞
```

Both are **strictly positive at every coefficient** — the period-2 mode is
inexpressible under either, which is E4's non-negotiable and the entire reason this arm
exists. But the factorization's splitting error does not stay in the surface: because
the flux is recovered as `F = c_e·Δs⁺`, an over-damped solved surface yields an
under-sized transport, and the grid-scale mode **stagnates** where exact BE annihilates
it.

**Why this is a finding and not a bug.** The over-damping would be harmless if the
kernel returned *states*: over-damped and under-damped both go to zero and the
landscape is the same. The kernel returns **fluxes** — deliberately, because that is
what makes mass-exactness a property of IEEE-754 subtraction instead of a property of
solver convergence — and an over-damped surface has small drops, so the scheme
**under-transports exactly the modes it over-damps**. The ADI splitting error is
famously second order *in the increment*, which is why the Braun/FastScape lineage
uses it without apology; recovered as a flux, it is first order in the transport.
**⚠ This is worth carrying to the E4-3 conversation**: it is a real interaction
between two design choices, both of which are right on their own.

**MEASURED.** The closed form is confirmed. It was briefly not: the fixture read
`0.555472917` against a derived `0.555555556` at `a = 0.5` and failed. **That was a
defect in the measuring window, not in the scheme.** `MARGIN = 4` is sized for the
explicit operator, whose boundary travels one cell per step. A Thomas recurrence spans
its **whole line**, so under an implicit scheme the boundary is present in every interior
cell after **one** solve and decays only *spatially*, as `ρ^m` with

```text
    ρ(a) = ((1 + 2a) − √(1 + 4a)) / (2a)     →  1  as a → ∞
    ρ(0.5) = 0.268      ρ(5) = 0.642      ρ(50) = 0.868      ρ(105) = 0.907
```

Worst residue against the closed form over `a ∈ {0.5, 5, 50, 105}`:

| w | margin | residue |
|---|---|---|
| 24 | 4 | 4.268e-03 |
| 96 | 24 | 1.922e-04 |
| 128 | 32 | 6.653e-05 |
| 160 | 48 | 1.397e-05 |
| 192 | 64 | 2.933e-06 |
| **256** | **96** | **1.293e-07** |

The fixture now runs at `w = 256`, margin 96 — **derived from `ρ(105)`, not raised until
it went green** — and passes at `1e-6`.

### 3.1b ⚠ MONOTONICITY IS NOT UNCONDITIONAL — the study's actual sharpest finding

**MEASURED 2026-08-04**, and it contradicts § 1's second bullet and the `implicit`
module's own "the monotone rule is a damping rather than a bias".

**The chain the design worried about is fine.** `solve → flux recovery →
limiter-as-projection` at `picard = 1` never flips the grid-scale mode — measured to
`a = 1e8`, and derived rather than merely observed:

```text
    solved grid-scale amplitude      = A / (1 + 4a)²
    donor export / its inventory     = 4a / (1 + 4a)²   ≤  1/4   at every a
                                                        (max at a = 1/4)
```

**The limiter is inexpressible on this mode at any coefficient**, because the solve damps
the drop faster than the coefficient grows. What survives is the pure linear factor
`(1 + 16a²)/(1 + 4a)² ∈ [1/2, 1)`, positive by inspection.

**What breaks it is the Picard loop**, and the mechanism is a positive feedback nobody
predicted:

1. Pass 1 leaves the **donors** unlimited (above) — but hands `scale = inventory/out = 0`
   to every **empty** cell that the smeared solved surface leaves above any neighbour.
   At `a = 5` on the 24² fixture that is **46 of 576 cells, 42 of them interior**.
2. The monotone rule (`lim` may only decrease) makes that **permanent for the epoch**.
   Those cells' outgoing edges are severed, so the operator the next pass solves is not a
   gentler version of the first — it is a **disconnected** one.
3. A disconnected operator barely damps, so the solved surface **steepens**: amplitude
   `0.0496 → 1.76` over four passes, a factor of **35**.
4. The final projection recovers flux from that steep surface at the **full** coefficient,
   requests **10.5× the donor's inventory**, and the limiter caps it at exactly **100 %**.
   A full cell shipping *everything* to four bare neighbours, each receiving from four
   full neighbours, **is an exact swap** — the period-2 flip-flop rebuilt out of a limiter
   doing precisely what it promised.

Conservation and `h ≥ 0` hold throughout, at every rung.
**Monotonicity is simply not among the projection's guarantees**: the projection's
binding constraint *is* a 100 % export, and on the grid-scale mode a 100 % export *is* a
sign flip. (The integrator's hypothesis at dispatch was step 4 alone; steps 1–3 are why
it fires at `a = 5` when the unrelinearized scheme at `a = 1e8` does not.)

**Worst (most negative) signed grid-scale amplitude over the fixture's four steps**,
checkerboard cover 40, initial amplitude 20, `Accuracy::Fixed(1)`:

| a | picard 1 | picard 2 | picard 4 | picard 8 |
|---|---|---|---|---|
| 0.5 | 1.91 | 1.91 | 1.91 | 1.91 |
| 2.0 | 8.30 | 8.30 | 8.30 | 8.30 |
| 3.0 | 10.85 | 9.80 | **−12.38** | **−16.07** |
| 5.0 | 13.67 | 9.98 | **−18.28** | **−18.97** |
| 105 | 19.20 | **−16.30** | **−19.00** | **−19.04** |

Read the `picard = 1` column down and the ADI stagnation is visible beside the sign
guarantee: **1.91 at `a = 0.5`, 19.20 at `a = 105`.** The mode never flips and, at high
coefficient, never leaves either — monotone and **inert**, which § 6 prices.

**The bound**, bisected on the sign-flip predicate alone:

| grid | picard 4 | picard 8 | picard 16 |
|---|---|---|---|
| 16² | 2.000275 | 2.000275 | 2.000275 |
| 24² | 2.081732 | **2.000001** | **2.000001** |
| 32² | 2.134917 | **1.999999** | **1.999999** |
| 48² | 3.070995 | **1.999999** | **1.999999** |
| 64² | 4.090404 | **1.999999** | **1.999999** |

**The converged bound is `a = 2.0` to six figures, independent of grid size.** At
`picard = 4` the loop has not converged and the *apparent* bound is looser and **grows
with the grid** — a warning, not a reprieve: **more passes make it worse**, so no Picard
count buys the promise back. The closed form for 2.0 is **not yet derived**; it is
measured, and the derivation is owed.

**Guarded by**, in `dc-core`, all passing:
`the_implicit_step_never_flips_the_grid_scale_mode` (the promise, at `picard = 1`, with
its derivation) · `the_picard_relinearization_flips_the_grid_scale_mode_past_a_measured_bound`
(the bracket: holds at 2.0, flips at 2.5, at `picard ∈ {4, 8}`) ·
`the_accuracy_divisor_restores_monotonicity_under_relinearization`.

### 3.2 Anisotropy — the rotated-ridge fixture (a HARD-REJECTION criterion)

The splitting error is the cross term `Lx·Ly`, which is **axis-aligned by
construction** and therefore the one error a product-form ADI has that an exact solve
does not. It is measured on a sinusoidal ridge in the regolith, run axis-aligned
(`k ∥ x`) and rotated 45° (`k ∥ x+y`) at the *same* `|k|`, with an inventory large
enough that the limiter never binds.

**The control is the explicit scheme, not zero.** A 4-neighbour Laplacian is
anisotropic at O(k⁴) by itself, so demanding zero would demand something the
*validated* reference also fails. The criterion is therefore **no more anisotropy
than the reference already has**, which is exactly the S-4 square-phenomenon
criterion pointed at the right baseline.

**DERIVED, closed form, for reading the table** — for a mode `(kx, ky)` the aligned
arm has `λy = 0`, so `Lx·Ly = 0` and the aligned arm is **exact**; the rotated arm
has both large. The gap therefore grows with `λxλy/(λx+λy)`, i.e. with `a·kx²ky²/(kx²+ky²)`
— **short wavelengths and high coefficients**, and nothing at long wavelengths.

**DERIVED (exact), 2026-08-04** — taken from the closed form rather than from a ridge
fixture, because a rotated sinusoid at a *matched* `|k|` is not a grid harmonic and the
fixture's own projection error swamps the quantity being measured. The per-epoch state
amplification of each scheme over direction `θ` at fixed `|k|` is exact arithmetic:

```text
    explicit (sub-cycled n = ⌈a/0.125⌉) :  G = (1 − (2a/n)(2 − cos kx − cos ky))ⁿ
    product-form ADI (n_acc = 1)        :  G = 1 − (λx + λy)/((1 + λx)(1 + λy))
                                            λx = 2a(1 − cos kx),  λy = 2a(1 − cos ky)
```

Anisotropy is the spread of the **decay** `1 − G` over `θ ∈ [0, π/4]`. Largest per-edge
coefficient at which the implicit spread stays **inside** the explicit control's:

| wavelength | absolute-spread criterion | relative-spread criterion |
|---|---|---|
| **8 cells** | **0.854** | **0.800** |
| 12 cells | 1.210 | 1.160 |
| 16 cells | 1.571 | 1.524 |
| 24 cells | 2.301 | 2.255 |
| 32 cells | 3.033 | 2.988 |

The criterion is stated over **wavelengths 8–32 cells**, so the binding rung is the
shortest: **`A_ACC ≈ 0.85`**. Note the ADI is *better* than the reference below `a ≈ 0.5`
(at `a = 0.5`, λ=8, its spread is **0.0028** against the explicit control's **0.023**) and
crosses over between `a = 0.5` and `a = 1.0`; by `a = 5` it is **111× worse**.

### 3.3 The accuracy divisor `A_ACC`

**MEASURED — and the two criteria disagree, which is the finding.**

| criterion | bound | source |
|---|---|---|
| **anisotropy** (this file's § 3.2 definition, the one the constant's doc comment states) | **0.85** | § 3.2, closed form |
| **monotonicity** (not previously known to be a constraint at all) | **2.0** | § 3.1b, bisected |
| *the placeholder currently in the tree* | *0.5* | *a guess, never checked* |

**The binding one is 0.85**, and § 6 shows what it costs. The placeholder 0.5 was a
better guess than it had any right to be — it is the right order — but it is **1.7×
tighter than the measurement**, which on this constant is 1.7× of gen time.

⚠ **`IMPLICIT_ACCURACY_MAX_EDGE_COEFF` HAS NOT BEEN CHANGED.** Both numbers above come
from closed forms and a validated scalar mirror, not from
`creep_operator_probe -- --fixtures`, which is the instrument the constant's own doc
comment names as its heir and which has still not been run. Replacing the constant is
E4-3's business and § 8 states why the answer may be "none of these".

---

## 4. The strongly-limited cascade fixture (U-1's amendment)

The fixture is a steep bare-bedrock chain: bedrock falls linearly and steeply along
x, regolith exists **only in the source column**, every other cell starts empty. The
only way material reaches cell `k` is by passing through `1..k` **inside the same
epoch** — the within-epoch receive-and-reship the explicit sub-cycle performs.

**What K2 is expected to do here, stated before the numbers so the fixture cannot
be read backwards.** The final projection caps each cell's export at its
**start-of-epoch** inventory. A cell that starts empty therefore ships nothing,
whatever it receives during the epoch. K2's within-epoch cascade is bounded at one
cell per *sub-epoch*, so the accuracy divisor `n_acc` is the cascade lever — and this
cap is **structural, not a convergence artefact**: more Picard passes cannot lift it.

This is the approximation U-1's amendment named, whose heir is K3 (only a proper
complementarity solve removes it) — and it is the reason the amendment ordered it
measured rather than assumed.

**MEASURED 2026-08-04**, on the mirror. `w = 32 × 8`, bedrock falling 50 m/cell, 200 m of
regolith in column 0 and nothing anywhere else. Two quantities per arm: the **front**
(furthest column holding regolith after one epoch — the explicit reference clears the
whole 31) and the **overshoot** (the largest amount by which a cell's potential ends
*above* its uphill neighbour's — 0 means the max principle held; anything positive means
transport pushed material **past level**).

| a | arm | n_acc | front | overshoot (m) |
|---|---|---|---|---|
| 5 | `k=1`, `n_acc=1` | 1 | 1 | **150.0** |
| 5 | `k=4`, `n_acc=1` | 1 | 1 | 0.0 |
| 5 | `k=1`, `A_ACC=2.0` | 3 | 3 | **27.6** |
| 5 | **`k=4`, `A_ACC=2.0`** | 3 | 3 | **0.0** |
| 5 | `k=1`, `A_ACC=0.85` | 6 | 6 | 0.0 |
| 5 | **`k=4`, `A_ACC=0.85`** | 6 | 6 | **0.0** |
| 5 | *EXPLICIT* | 40 | **31** | 0.0 |
| 105 | `k=1`, `n_acc=1` | 1 | 1 | **150.0** |
| 105 | `k=4`, `n_acc=1` | 1 | 1 | **150.0** |
| 105 | `k=1`, `A_ACC=2.0` | 31 | 31 | **16.8** |
| 105 | **`k=4`, `A_ACC=2.0`** | 53 | **31** | **0.0** |
| 105 | `k=1`, `A_ACC=0.85` | 124 | 31 | **24.5** |
| 105 | **`k=4`, `A_ACC=0.85`** | 124 | **31** | **0.0** |
| 105 | *EXPLICIT* | 840 | **31** | 0.0 |

**Confirmed as predicted:** at `n_acc = 1` the front is exactly **one cell**, at every
coefficient and every Picard count, against the explicit reference's 31. `n_acc` is
indeed the lever, and it is exact — the front is `min(n_acc, W−1)`.

**⚠ AND THE FIXTURE ANSWERS THE QUESTION § 3.1b LEFT OPEN, AGAINST `picard = 1`.**
§ 3.1b's derivation covers the grid-scale mode only, and this is the surface where it runs
out. **`picard = 1` overshoots by 150 m** — the source column ships **100 % of its
inventory** into the single cell below it, burying it 150 m *above* the cell it came
from. Sub-cycling reduces but does not remove it (16.8–27.6 m). **`picard = 4` holds the
max principle exactly**, everywhere except the `n_acc = 1` corner at `a = 105` — which
sub-cycling then fixes.

**So the two arms fail in opposite places, and each one's failure is the other's fix:**

| | grid-scale mode (§ 3.1b) | steep supply-limited chain (§ 4) |
|---|---|---|
| `picard = 1` | **monotone at every `a`** (derived) | **overshoots, 150 m** |
| `picard ≥ 4` | **flips above `a = 2.0`** | **max principle exact** |
| **`picard = 4` + `n_acc = ⌈a/2⌉`** | **monotone** | **exact** |

The Picard relinearization is not decoration: it is what stops a supply-limited cell
shipping past level, which is precisely the obstacle it is there to approximate. It is
also what breaks the checkerboard. **Only the sub-cycled `picard = 4` arm passes both** —
and § 6 prices it.

**One identity worth stating plainly, because it bounds the whole scheme.** Both schemes
advance the cascade exactly one cell per sub-step, so

```text
    cascade reach ratio  =  (a / A_ACC) / (8a)  =  1 / (8·A_ACC)
```

which is **the same expression as the cost ratio at `picard = 1`**. K2 buys its speed by
doing proportionally less within-epoch transport — 1/16 the reach at `A_ACC = 2.0`. That
is not a defect to fix; it is what an epoch-solve *is*. Whether it matters depends on how
far a front must travel in 200 epochs (at `a = 5`, `n_acc = 3` gives 600 cells against a
545-cell grid, so plausibly not) — **and that is a § 5 world-run question, still unrun.**

---

## 5. World-level agreement at M ∈ {45, 150} (audit § 5.2)

`PENDING TABLE + verdict against § 2's bar`

---

## 6. Cost

**DERIVED cost model, to be checked against the measurement.** One implicit
sub-epoch is roughly `k` Picard passes at ~3.5 explicit-sub-step equivalents each
(build edges, two Thomas banks, three transposes, the limiter re-freeze) plus one
final projection at 1 equivalent — call it `3.5k + 1`, or **15 units at k = 4**. One
explicit epoch is `n = a/0.125 = 8a` units. So

```text
    implicit / explicit  =  (3.5k + 1) · n_acc / (8a)
```

Two consequences, both of which the measurement has to confirm or refute:

- at `n_acc = 1` the implicit arm is **flat in M** and the ratio falls as `1/a` —
  15 units against 840 at `a = 105`, which is the audit § 6 projection;
- if accuracy forces `n_acc = a/A_ACC`, the ratio collapses to `(3.5k+1)/(8·A_ACC)`,
  **independent of a**: break-even at `A_ACC ≈ 1.9`, and *slower than explicit*
  below it. **The entire value of K2 rests on `A_ACC` being large**, which is what
  § 3.3 measures.

**§ 3.3 measured it, and the answer is not large.** The model evaluated at the measured
bounds (`(3.5k + 1)/(8·A_ACC)`; **< 1 is cheaper than explicit**):

| | `A_ACC = 0.5` (placeholder) | **`A_ACC = 0.85` (anisotropy)** | `A_ACC = 2.0` (monotonicity) |
|---|---|---|---|
| `picard = 1` | 1.125 | **0.66** | 0.28 |
| `picard = 2` | 2.000 | **1.18** | 0.50 |
| **`picard = 4`** (the study's arm, the probe's arm) | 3.750 | **2.21** | 0.94 |

**At the criterion this file actually defines, and at the Picard count everything in the
tree uses, K2 is 2.2× SLOWER than the explicit scheme it replaces.** The break-even
`A_ACC` of 1.875 at `k = 4` is more than twice the anisotropy bound.

⚠ **Read the `picard = 1` row as a cost, not as an option.** It is the cheap row, and § 4
disqualifies it: that arm overshoots the max principle by 150 m on a supply-limited
chain. **The only row that passes both structural fixtures is `picard = 4`.**

Three further facts the model alone does not show, all measured:

- **The `ceil` bites at small `a`.** The ratio is the asymptote, not the value. At
  `a = 5`, `A_ACC = 2.0`, `k = 4`: `n_acc = ⌈5/2⌉ = 3`, ratio **1.125** — worse than 1,
  where the asymptote says 0.94. The arm is cheapest exactly where the coefficient is
  highest, which is at least the right way round.
- **`n_acc = 1` is cheap and useless.** At `a = 105`, `k = 1`, `n_acc = 1` the ratio is
  **0.0054** — 185× cheaper — and the fixture's grid-scale amplitude after four epochs is
  **19.2 against an initial 20**. It is monotone and **inert**: the ADI state factor
  `(1 + 16a²)/(1 + 4a)² = 0.995` at `a = 105`. The mode neither flips nor decays, which is
  precisely the "stable-looking limit cycle" journal/0116 was about, in a new costume.
- **Sub-cycled, it works.** Grid-scale amplitude after four epochs (initial 20), against
  the explicit reference:

| a | `n_acc = ⌈a/2⌉` | implicit `k = 4` | explicit reference |
|---|---|---|---|
| 5 | 3 | 9.1e-1 | 2.1e-4 |
| 50 | 25 | 5.6e-7 | 4.3e-7 |
| 105 | 53 | 3.8e-10 | 2.3e-10 |

  Monotone at every rung, and within a factor of ~2 of the reference at the top of the
  ladder. **The scheme is sound; it is the price that is the problem.**

---

## 7. The gate

The probe carries `test = true` already (journal/0103's mechanism), so the assertions
below run under `cargo test` while `cargo run --example creep_operator_probe --
--fixtures --world` still prints the full report. **All of them are scale-free
invariants, none is a snapshot**, and each says in its doc comment why:

*In `dc_core::field::tests_implicit` (kernel structure):*

| test | what it pins |
|---|---|
| `the_factorized_solve_inverts_its_own_two_banks` | the Thomas recurrence, the border rows and the transposed y-bank actually invert `(I+Lx)(I+Ly)` — a linear-algebra identity |
| `the_implicit_step_never_flips_the_grid_scale_mode` | **the non-negotiable**: no mode changes sign at `a ∈ {0.5, 5, 50, 105}` |
| `the_nyquist_amplification_is_the_derived_factorized_one` | the measured factor **is** the derived closed form — so § 3.1's accuracy gap is a number a reader can act on, not a warning to be trusted |
| `the_implicit_projection_conserves_mass_and_never_goes_negative` | mass exact to 1e-12 relative, `state ≥ 0`, at every coefficient, over 20 epochs |
| `scalar_and_parallel_implicit_steps_are_bit_identical` | bit equality across the Picard loop and the transposed bank |
| `the_implicit_plan_divides_for_accuracy_not_stability` | the divisor follows from `A_ACC`, and is never larger than the explicit scheme's — the inequality the whole cost argument rests on |

*In `creep_operator_probe::gate` (the study's own fixtures):*

| test | what it pins |
|---|---|
| `the_implicit_scheme_never_flips_the_grid_scale_mode` | the sign guarantee, on the report's own fixture |
| `the_derived_divisor_holds_the_splitting_error_inside_the_stencils_own_anisotropy` | **the hard-rejection criterion**, evaluated through `Accuracy::Derived` so it guards `A_ACC` itself |
| `the_implicit_cascade_is_bounded_and_the_accuracy_divisor_is_its_lever` | the cascade is positive, bounded by the reference, and **responds to `n_acc`** |

`PENDING: added gate wall-clock, per the probe doctrine.`

---

## 8. Verdicts, and what is owed

*2026-08-04, the diagnosis pass. **Every ruling below is the user's**; this section states
the decision, not the answer.*

### What is settled

1. **The kernel is correct, and the two red fixtures said two different things.** The
   Nyquist failure was a **measuring-window defect** (§ 3.1) — closed form confirmed to
   1.3e-7. The monotonicity failure is **real** (§ 3.1b).
2. **The unrelinearized scheme keeps E4's promise.** `picard = 1` never flips the
   grid-scale mode, at any coefficient, with a closed form saying why.
3. **The Picard relinearization breaks it at `a = 2.0`** — measured, six figures,
   grid-independent, and **worse with more passes**.
4. **Sub-cycling repairs it** (§ 6's last table). Monotone at every rung, within ~2× of
   the explicit reference at the top of the ladder.
5. **Accuracy sub-cycling is therefore REQUIRED, not optional** — which is the cost-model
   change the dispatch anticipated, arriving through a different mechanism than predicted.

### The decision, stated as one

**`A_ACC` has two measured candidates that disagree by 2.4×, and at the smaller one K2 is
slower than the scheme it exists to replace.**

| if `A_ACC` is… | because | cost at `k = 4` | cost at `k = 1` |
|---|---|---|---|
| **0.85** | this file's § 3.2 anisotropy criterion, which is the criterion the constant's doc comment states | **2.21× SLOWER** | 0.66× |
| **2.0** | § 3.1b's monotonicity bound | 0.94× (a wash) | **0.28×** |

**⚠ `picard = 1` IS NOT THE ESCAPE, AND THIS PARAGRAPH SAID IT WAS FOR ABOUT AN HOUR.**
The first draft of this section concluded that `picard = 1` at `A_ACC = 2.0` was "the only
configuration clearly worth having" — 0.28× and monotone — while flagging that its
derivation covered the grid-scale mode only and that § 4 had not been run. **§ 4 was then
run and refuted it**: `picard = 1` overshoots the max principle by **150 m** on the
supply-limited cascade, shipping 100 % of a source cell's inventory to 150 m *above* the
cell it came from, and sub-cycling only reduces it to 17–28 m. *Recorded rather than
silently repaired, because the shape of the error is the point: a cheap configuration that
passes the one fixture you have run is exactly what a cost model wants to be true.*

**The two failure modes are complementary, and only one configuration escapes both:**

| | grid-scale mode | steep chain | cost at best `A_ACC` |
|---|---|---|---|
| `picard = 1`, `n_acc = 1` | monotone | **overshoot 150 m** | 0.005× |
| `picard = 4`, `n_acc = 1` | **flips above `a = 2`** | overshoot 150 m at `a = 105` | 0.018× |
| `picard = 1` sub-cycled | monotone | **overshoot 17–28 m** | 0.28× |
| **`picard = 4` sub-cycled** | **monotone** | **exact** | **0.94×** (2.21× if `A_ACC = 0.85`) |

The Picard loop is load-bearing: it is what stops a supply-limited cell shipping past
level, which is the obstacle it exists to approximate. It is also what breaks the
checkerboard. **The only arm that passes both structural fixtures is the sub-cycled
`picard = 4` one, and it is a wash on cost at `A_ACC = 2.0` and 2.2× slower at the
anisotropy bound of 0.85.**

**So the honest summary is: K2, run in a configuration that is actually correct, does not
currently pay for itself.** That is a finding, not a verdict — § 5's world runs have not
happened, and the ladder's own tolerance for a shorter within-epoch cascade is exactly
what they would measure.

### What is owed, in order

1. **§ 5's world runs.** Now the highest-value unrun item: they are what says whether a
   1/16 within-epoch cascade reach (§ 4) changes any ladder statement, which is the only
   remaining argument that could make the 0.94× arm worth its complexity.
2. **A closed form for `a = 2.0`.** It is measured to six figures and grid-independent, so
   there is one; not having it means we cannot say how it moves with the stencil.
3. **Both structural fixtures ported from the mirror into the probe.** § 3.2 and § 4 were
   measured on a scalar mirror validated bit-for-bit against this kernel on both of its
   original failures. That is evidence, and it is not the repo's own instrument.
4. `IMPLICIT_ACCURACY_MAX_EDGE_COEFF` replaced in the same commit as the probe table that
   derives it, per its own doc comment. **Still the flagged placeholder today.**
