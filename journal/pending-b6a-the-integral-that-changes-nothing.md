# B6-a — the integral that changes nothing

*Slug-only, unnumbered: a parallel session shares this checkout and ordinals have
collided six times in thirty hours. The integrator assigns the number.*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — the whole slice is a
> re-housing whose acceptance criterion is *"prove that nothing happened"*, and the
> only interesting failure it produced was a test that was **right to go red for a
> reason that was not about the code.** Also **lens 1 (AI-native development)**: a
> design pass hand-derived twelve numbers with no compiler, and the build confirmed
> every one it could reach to every printed digit — which is a fact about how the
> pass was written, not about luck.

## The thing that was wrong

`bake_resting_posture` computed a centre of mass like this, inline, thirty lines
into a function about balance:

```rust
let v = s.size_m[0] * s.size_m[1] * s.size_m[2];
…
vol_sum += v;
*m += v * c;
```

That is a mass integral at density ≡ 1, and everybody knew it — `stubs.md` #40 has
said so since 2026-08-02, the module doc named B6 as its heir *at the line*, and by
the time B7 shipped its derived joint limits the same proxy was feeding **three**
members: posture's CoM, the gait bake's cadence and duty, and the end-range law.

The defect was never the number. It was the **shape**. `ARCHITECTURE.md` § *a summary
is not an authority* asks one question of a cheap answer: *if the consumer that forced
it disappeared tomorrow, would this code still exist in this shape?* A volume-weighted
centroid written into the middle of a balance solve fails that test — it exists there
because the balance solve needed it, and nothing else could reach it, agree with it, or
disagree with it out loud. There was no authority for it to be a summary **of**.

So B6-a builds the authority and points the consumer at it. That is the entire slice.

## What it is, and the discipline of what it is not

`dc-api/src/bodies/mass.rs`:

```rust
pub fn segment_densities(plan: &BodyPlan) -> Vec<f64>;
pub fn mass_properties(plan: &BodyPlan, densities: &[f64], root_height_m: f64) -> MassProperties;
```

Two things about that signature are load-bearing and both were argued before they were
typed.

**The integral takes `&[f64]` and never sees a material.** The A-7 diagnostic — *what
property am I actually reaching for?* — answers **density**, so density is what the
function takes. No `MaterialId`, no material name, no registry lookup enters any body
module. This is not fastidiousness: it is what insulated the slice from a ruling that
landed the day after the design pass. `materials.md` § DECIDED 2026-08-04 made materials
**labels over a term space**, which may subsume the facet split the pass recommended in
its § 4 — and because the integral asks for a number rather than for a material, none of
that reaches it. **The function that does not know is the function that does not have to
be rewritten.**

**The density seam is a function whose identity is exactly `1.0`.** Not a constant, not a
default parameter, not an `Option` — a named function with a doc comment naming its heir
(B6-c: a declared `SegmentDef.composition` resolved against a roster). `v * 1.0 == v` in
IEEE 754 exactly, which is what makes the conversion byte-identical, and it holds *only*
for a density that is precisely one. There is a test asserting that as bits, because
"1.0" written as `1.0000000000000002` would silently move every world.

And three things were deliberately **not** built:

- **No `composition` field.** There is no tissue material in the roster and no
  `define_material` door, so the field would be unfillable. A field nobody can fill is
  A-4 with a schema attached.
- **No facet split of `MaterialProps`.** The design pass recommended it and then withdrew
  the recommendation in its own header, pending comparison against the term-space ruling.
  Building it anyway would be this project's characteristic failure — a second mechanism
  beside the one just ratified — performed on the one day everybody could see it coming.
- **No `subtree_inertia`.** This one deserves its own paragraph.

## The function I went looking for a reason to build, and did not build

The design pass proposes `subtree_inertia(plan, densities, seg, axis) -> f64` beside the
integral, and — to its credit — flags in its own § 11 that it has a day-one consumer
**only if the same slice derives `cadence_scale`**. This slice does not: the cadence
scale's `√(gMd/I)` denominator is now derivable, but its **muscle-power numerator does
not exist anywhere in the corpus**, and half a formula is not a derivation.

So I went looking for another caller, and I found one that is genuinely tempting.
`bodies/gait/tests/report.rs:306` already computes a leg's parallel-axis inertia about the
hip by hand, and prints `m = 0.028584, d = 0.4156, I_hip = 6.848e-3, T_nat = 1.523` — which
is P4, verbatim, already in the tree and already green. That is a duplicated derivation,
and *hoist, don't duplicate* is a rule this arc has used twice (`stance_chain` out of
dc-client, `offset_from_root` into one home).

I did not hoist it, and the reason is worth writing down because the call was close. Both
prior hoists moved a derivation with **two production consumers** into one home. This one
has **one consumer and it is a `#[test]`**. A public engine primitive justified by a report
is A-4 wearing a lab coat — and it would additionally force a choice on the `Axis`-versus-
tensor question the design pass explicitly lists as open (§ 14 item 12), pre-answering a
live question to save a duplication that costs eleven lines.

The invariance test needed an inertia anyway, so it grew a **test-local** `leg_omega`, with
a doc comment saying exactly why it is test-local. When `cadence_scale` gets its actuation
numerator, that helper and `report.rs`'s copy collapse into the real function together, and
the report's printed table becomes the acceptance for it.

*"No consumer, so I did not build it"* is the whole finding, and it is a first-class one.

## The acceptance: proving that nothing happened

Every animation-side output of this arc is **invariant to a uniform density** — not
approximately, algebraically. The centre of mass is `Σ(Vᵢρcᵢ)/Σ(Vᵢρ)`, and a common ρ
cancels. `root_height_ratio` is pure geometry. A limb's free-swing frequency is
`ω = √(gMd/I)` with `M ∝ ρ`, `I ∝ ρ`, `d` independent of ρ — ρ cancels *completely*.

This is why the acceptance could not be *"the bob looks better"* or *"the gait changed"*.
For the three shipped near-uniform bipeds it will barely be different, **and that is
correct**. A changed number here is a bug, not progress.

So the acceptance was byte-identity, established the boring way: before touching anything,
a temporary test dumped the raw `f64` bit patterns of the entire `RestingPosture` — ratio,
root height, all three CoM components, every chain reach, every joint Euler — for all three
shipped plans. Then the change. Then the same dump. Then a diff.

```
biped  com  bc47c13184e78c29 3feefb63e4f12d1e 0000000000000000   (before)
biped  com  bc47c13184e78c29 3feefb63e4f12d1e 0000000000000000   (after)
```

Identical for all three plans, every field. `3feefb63e4f12d1e` is 0.96818728175404423 m, and
the design pass predicted **0.968187** by hand with no compiler.

The dump test was then deleted — a recorded magnitude in the gate is a snapshot, and the
rule here is *assert invariants, never snapshots*. What ships instead are two live
assertions: `the_com_loop_has_one_authority` (the bake's CoM equals the integral's **bit for
bit**, so the day two authorities exist is the day it goes red) and
`at_the_identity_density_mass_is_the_volume_proxy_bit_for_bit`.

## The one thing that went red, and why it was right to

`uniform_density_is_animation_invariant` was written from the design pass's § 7 test 2:
sweep ρ ∈ {1, 600, 1010, 16000} and assert every derived output equal within 1e-12
**relative**. It failed immediately, on all three plans, and only on the lateral axis:

```
plan `dc:body/biped` axis 0 at ρ = 600: CoM 0.00000000000000000 vs -0.00000000000000000
```

Two zeros that are not equal. The biped's `com_m[0]` is **−8.2e-19 m** and the stout's is
**+9.9e-19 m** — these bodies are mirror-symmetric by construction (B0 made the right side a
generated mirror of the left), so the lateral centre of mass is an **exact cancellation**,
and what survives is the last ulp of a sum whose true value is zero. Multiply every density
by 917.5, or reverse the segment order, and which of `+0.33·V` and `−0.33·V` lands in the
accumulator first changes — so the *sign* of a 1e-19 residual flips, and a relative
comparison of a number against its own negation returns **1.0**.

The test was asking whether a rounding residual kept its sign. That is not a fact about the
integral. It is A-3 inverted: not a green for a reason unrelated to the claim, but a **red**
for one — and the failure mode is more dangerous than it looks, because the obvious repair is
to loosen the tolerance to something like `1e-9` and move on, at which point the test stops
seeing real motion on the axes that *do* carry a value.

The fix is not a looser number, it is a **different denominator**. A CoM coordinate is a
*position*, and the meaningful scale for "did it move" is the **body**, never the
coordinate's own magnitude:

```rust
fn moved(a: f64, b: f64, body_scale_m: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(body_scale_m)
}
```

The floor is the plan's own root height — derived, not chosen, which is the difference
between evidence and a number picked until it went green. It is also the reading the bake's
own acceptance already used a slice earlier (`com_m[0].abs() < 1e-9`, *mirror symmetry is
structural*); the two tests now agree about what those axes mean, which they did not before.

That divergence is stamped on the design pass's header. Its § 7 test 2 as written is not
implementable, and the reason is a property of mirror-symmetric bodies rather than an error
in the pass's arithmetic — which was otherwise **exact**.

## What the numbers said

Every prediction B6-a could reach came back to every printed digit: total volume
**0.168388000 m³** and the whole eleven-row share table; **170.072 kg** at a uniform
1010 kg/m³; the leg about the hip at `M/ρ` 0.028584, `d` 0.415630, `I/ρ` 0.006848268,
**ω 4.125332 rad/s**, `T` 1.523074 s; the heterogeneous rows at 4.114713 (−0.257 %) and
3.985832 (−3.38 %); `com_m[1]` **0.968187 m**; the lungs-in-the-trunk body at 164.227 kg
and a CoM 20.7 mm lower; the radial-order bound at 1.277 % of the leg's hip inertia.

Which is the finding underneath the finding. **Real tissue-density variation across a
fleshy animal's segments moves the derived gait by ~0.25 % and the centre of mass by
~0.1 mm.** The animation-visible payoff of per-segment composition is not telling muscle
apart from fat — it is **void** (lungs, air sacs, pneumatic bone) and **armour**. Putting
lungs in the trunk moves the CoM two centimetres; making a shank out of bone slows its
swing by 3.4 %. Telling a thigh from a shank moves nothing anyone will ever see.

`stubs.md` #40 already said that in words — *"a hollow-boned flier, a heavy-tailed biped, an
armoured body"*. This is the number underneath it, and it is a useful thing to know before
anyone proposes a tissue taxonomy.

## What it did not close

Zero of the eight inbound heirs that name B6. That is not a disappointment, it is the
ledger, and it is filed as `an-inertia-with-no-actuation`: the integral supplies the
**load** half of every force-shaped hole and none of the **power** half. `cadence_scale` is
half-derived. `swing_flexion` is the one a later slice plausibly closes. `bob_damping` and
the soft-tissue end-range are **not absorbed at all** — density is not stiffness, which
`stubs.md` #50 filed a day earlier — and the flight phase is mass-independent to begin with.

The one thing B6-a closes completely is the one nobody listed: the CoM stopped being a
volume centroid pretending to be a centre of mass, and became a centre of mass whose density
happens, today, to be 1.
