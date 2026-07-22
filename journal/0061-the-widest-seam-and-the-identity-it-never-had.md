# 0061 — the widest seam, and the identity it never had

*2026-07-22 — converting `depth_to_water`: one seam, four consumers, and the
discovery that it was never one question.*

## Why this one, and why alone

journal/0060 converted three seams to teach the shape. This slice converts
**one**, because it is the one with the largest blast radius in the 34-seam
inventory and because it was the only seed-set seam with **no identity path at
all**.

The code, `biotic.rs::step_cell`, in its entirety:

```rust
let low_bonus = ((80.0 - surf) / 80.0).clamp(0.0, 1.0) as f32 * 0.20;
let area_bonus = (area[i] / 300.0).min(1.0) as f32 * 0.15;
let wet = (moist + low_bonus + area_bonus).clamp(0.0, 1.0);
```

Five unmeasured coefficients — 80 m, 0.20, 300 cell-units, 0.15, and the implicit
unit weight on `moist` — summed into one number, answering the question *"is the
water table at the surface here?"*. The comment above it was scrupulously honest
about being a proxy. That honesty is the problem: **prose cannot fail a build**
(corrections #29), and four separate downstream thresholds had already been
written against the proxy's output range as though it were the rule.

The heir was not something I had to invent. It is already **ratified**:
`water.md`, DECIDED 2026-07-20 (*one quantity, two regimes*), consequence 4 —

> S10's waterlogging proxy has a defined retirement: waterlogging becomes "the
> water table is at or near the surface here", read from the field. Biology
> reads the real quantity; the proxy is deleted.

So this seam is unusual among the 34: the user has already decided who owns it.
What was missing was a socket for the owner to plug into, and — more pointedly —
a **way for the stand-in to be absent**.

## The identity path it never had

`erosion.rs` has had this shape for a long time:

```rust
fn wmult_at(bio_weather: &[f32], i: usize) -> f64 {
    if bio_weather.is_empty() { 1.0 } else { f64::from(bio_weather[i]) }
}
fn frost_at(frost: &[f64], i: usize) -> f64 {
    if frost.is_empty() { 1.0 } else { frost[i] }
}
```

Empty plane + identity accessor. That pair is what makes the four deep-sim flags
(`biotic`, `erodibility`, `full_agents`, `tectonic_history`) *provably* free when
off: turning the layer off does not allocate a plane of `1.0`s and then multiply
by them, it takes a branch. `wet` had nothing of the kind. There was no state of
the world in which the proxy was not running; you could not ask "what would this
world look like without the waterlogging guess" because the guess had no off
switch and no seam to put one at.

It has both now:

```rust
pub fn identity_depth_to_water(_pass: WaterPass<'_>, out: &mut Vec<f32>) {
    out.clear();
}

#[inline]
pub fn wet_at(plane: &[f32], i: usize, moist: f32, surf: f64, area: f64) -> f32 {
    if plane.is_empty() {
        identity_wet_index(moist, surf, area)
    } else {
        plane[i]
    }
}
```

The registered identity's whole job is to leave the plane empty. That reads like
a joke until you notice what it buys: the identity world allocates nothing,
branches once per cell, and — because there is exactly *one* call site and *one*
branch — the identity path cannot drift away from the provider path by editing.
They are the same function with two arms.

## Granularity: the rule got sharper

journal/0060's parting lesson was *granularity is a property of the heir, not of
the call site*, learned the hard way from `wave_energy`, whose payload struct its
own named heir cannot fill. This seam is where that rule earns its keep, because
the call site argues loudly and wrongly.

The call site says **value-level**: `wet` is consumed at four per-cell
thresholds, deep inside the biotic hot loop, from purely local inputs. Every
instinct says "make it `fn(WetCell) -> f32`".

The heir says **plane**. A water table is the top of the saturated zone — a
solution over a neighbourhood, not a per-cell arithmetic. It needs the drainage
network to know where water *collects*. There is no per-cell payload that can
carry that; you cannot hand a cell its own catchment.

So `depth_to_water` is pass-level, and `WaterPass` carries planes: `precip`,
`r`, `h`, `area`, `recv`, `filled`, plus `w` and `epoch`. That last field is the
second half of the granularity question, and it is where this slot differs from
`parent_p`:

| slot | materialized | why |
|---|---|---|
| `parent_p` | **once per run**, at `BioticSim::new` | parent material does not change |
| `depth_to_water` | **once per epoch**, at `BioticSim::step` | the table follows the surface, and the surface is what the erosion sim spends the whole ritual rewriting |

"Pass-level" turns out not to be one thing. It is "materialized at a pass
boundary", and *which* boundary is a separate question with a separate answer per
seam. Two slots, two boundaries, and the module docs now say so.

## The agreement test, and why the golden alone was not enough

The existing acceptance test — the production world at `Extent::Small` must hash
to `surface 0x7B8968FD90E04062` / `record 0xA53BD77F769D7FF4`, goldens captured
from pre-slice `main` for journal/0060 — passed unchanged, first try, and that is
the headline result.

But it only exercises the **empty** branch. With default providers, `wet_at`
never reads a plane, so the entire pass-level apparatus — the `WaterPass`
construction, the once-per-epoch call, the indexed read — is untouched by the
proof it is supposedly being judged on. Shipping like that would mean the plane
path arrives untested and breaks the first time an heir fills it.

So there is a second test, and it is the one ARCHITECTURE.md § *A summary is not
an authority* actually asks for — *a test asserting the summary AGREES with the
authority*:

```rust
fn proxy_plane(pass: WaterPass<'_>, out: &mut Vec<f32>) {
    let n = pass.w * pass.w;
    out.clear();
    out.reserve(n);
    for i in 0..n {
        out.push(identity_wet_index(pass.precip[i], pass.r[i] + pass.h[i], pass.area[i]));
    }
}
```

Register that as a **non-identity** provider (`is_identity()` is asserted false,
so this cannot silently degrade into the default path), run the whole production
world through it, and land on the same two hashes. Both arms of the seam now
produce the same world on the real grid, not on a unit-test fixture. It also
quietly proves the `WaterPass` contract is complete: everything the proxy needs
is reachable from the payload, at the same bit values, at the right point in the
epoch.

> blogworthy: the byte-identity test that proves nothing about the code you just
> wrote — an identity default so cheap it bypasses the machinery it is meant to
> validate, and the second test that closes the loop.

## Socket the constant, not the call site

Practice 5, and journal/0060's `P_ROCK_INIT` scar, say to grep the *constant*.
Grepped, across `crates/dc-worldgen/src`: `low_bonus`, `area_bonus`, `80.0`,
`300.0`, `0.20`, and the literal expression. Findings:

- The five coefficients appear **once each**, only in the two lines replaced.
  No second consumer, unlike `P_ROCK_INIT`.
- `80.0` also appears as `DeepConfig::thickening_scale` and as a pregen
  `Provenance::Trench` depth. Unrelated coincidences of magnitude — not the same
  quantity, deliberately not unified.
- `0.20` appears as a niche `shade` coefficient. Unrelated.
- **`area[i]` has a second, independent threshold**: `FLOOD_AREA = 120.0`, the
  flood-disturbance test at the bottom of `step_cell`. It reads the same drainage
  plane and asks a related hydrological question ("is this a flood-prone valley
  cell?") — but it is *not* part of the wetness expression, it is not one of the
  four `wet` consumers, and it has its own constant. Left alone, and named here
  because the next person grepping `area` will hit it and should know it was
  seen. It is arguably a fifth seam for the same heir.

## What the heir must actually supply

This is the part of the slice that is worth more than the code. Having read all
four consumers, here is what a real water table has to return.

**The finding that matters: `wet` is not one quantity. It is three.** The proxy
collapsed three different questions into one number because one number was all
there was to answer them with. The heir does not replace `wet`; it *splits* it.

**Consumer 1 — the waterlog suitability gate** (`biotic.rs:791`).
```rust
let t_wet = if niche.waterlog > 0.5 { ((wet - 0.30) / 0.15).clamp(0.0, 1.0) } else { 1.0 };
```
Only the peat-former niche reads it. The real question is **rooting-zone water
availability**: can an obligate wetland plant keep its roots in saturated ground?
That is depth-to-table measured against a *root depth*, and it is a
growing-season **mean**, not an instant. Ramps 0 → 1 across `wet` 0.30 → 0.45.

**Consumer 2 — the decomposition drain factor** (`biotic.rs:877`).
```rust
let drain_factor = (1.0 - (wet - 0.30) / 0.40).clamp(0.10, 1.0);
```
This is the one that actually makes peat, and therefore coal. Its real question
is **what fraction of the year is the litter horizon anoxic** — the top decimetre
or so, where oxygen either reaches the litter or does not. It is a *duration*,
not a depth: a table that sits 5 cm down for eleven months and 2 m down for one
preserves organics; a table oscillating through the same mean does not. A single
scalar depth cannot answer this. The heir must supply a saturated **fraction of
time**, or the statistics needed to derive one.

**Consumer 3 — the fire dryness term** (`biotic.rs:940`).
```rust
let dryness = (1.0 - wet).clamp(0.0, 1.0);
let eff_fuel = fuel * (0.4 + 0.6 * dryness);
```
**This consumer's question is not about the water table at all.** Whether cured
grass burns is set by fuel moisture in the top few centimetres during the *dry
season* — a vadose-zone quantity, and a **minimum** over the epoch, not a mean. A
table 3 m down is irrelevant to a grass fire. The proxy answered it anyway,
because the proxy's leading term was climate moisture and that is at least
adjacent to the right thing. When the heir lands, this consumer should read
near-surface saturation, not depth-to-table, and it should read the dry extreme.
Wiring it to a water table because a water table is what arrived would be the
stand-in-becomes-definition failure repeated one level up.

**Consumer 4 — the `peat_site` hiatus test** (`biotic.rs:948`).
```rust
let peat_site = cell.cover[4] > 0.15 || (wet > 0.42 && net_org > 0.004);
```
Binary, and it decides a column's depositional-hiatus cap — i.e. whether an
organic horizon can be recorded at all under a given aggradation rate. Its real
question is **is this site at or above grade in standing water for most of the
year**, which is the one consumer the phrase "the water table is at or near the
surface here" describes exactly.

### The contract, stated

- **Units.** Metres below the local surface, **signed**, negative meaning
  standing water above grade. `f32` is ample (mm resolution at ±100 m). The
  identity's dimensionless 0..1 index is *not* this, and converting the four
  thresholds is a behaviour change — deliberately not in this slice. **The units
  mismatch is the seam's most useful output**: a socket whose plug does not fit
  is more informative than a socket that hides the fact.
- **Not one number.** At minimum: a growing-season mean depth (consumers 1, 4), a
  saturated-fraction-of-time at the litter horizon (consumer 2), and a
  dry-extreme near-surface moisture (consumer 3). Expect `depth_to_water` to
  become two or three slots when the heir lands. That is not a failure of this
  design; it is this design doing its job — the slot made the question askable,
  and the question turned out to have three answers.
- **Granularity.** Per cell, per epoch, whole plane. The consumers only read
  subaerial cells (the `surf <= SEA_LEVEL_M` early return precedes the read), but
  the provider must still fill the plane to `w * w` — the accessor indexes it and
  a short plane is a panic, not a fallback.
- **Determinism.** Pure in `WaterPass` plus caller-owned seeds. The biotic loop
  runs under rayon above 2^15 cells, and scalar↔parallel byte-identity is a
  standing invariant; a provider that reads a clock or a global breaks it.
- **Timescale.** One deep epoch is ~10^5 years; a water table equilibrates in
  years to decades. So the heir supplies the **equilibrium** table for the
  epoch's surface and climate, never a transient — which is precisely why S9
  could classify it as a *bounded relaxation* (haloable, C-refinable) rather than
  an advective field. The seam and the spike's classification agree, which is
  mild evidence both are right.
- **Cost, stated as a fact and not as a budget.** At `Extent::Medium` this is
  545² × 200 ≈ **59 million cell-epoch solves** per ritual. A whole-grid
  iterative relaxation every epoch will not be free. Recording the number here so
  the heir's cost is known in advance — *not* as a licence to cheapen the
  simulation to fit it. This project's standing position is that worldgen time is
  not the constraint and ready-made worlds are the sanctioned answer; if the real
  water table costs a minute, it costs a minute.

## Cost

Ritual = the production deep-time run at `Extent::Medium` (545², 200 epochs, full
agent roster), timed inside the test, machine otherwise idle, both sides warm,
`cargo clean -p dc-worldgen --release` between switches because the two worktrees
share one `CARGO_TARGET_DIR` (corrections #27, which fired live during the last
slice).

| | set A | set B | mean of 8 |
|---|---|---|---|
| pre-slice (`b63ee84`, the main checkout) | 15.896 / 15.891 / 15.957 / 15.850 | 16.247 / 16.290 / 16.009 / 15.944 | **16.011 s** |
| post-slice (this worktree) | 16.276 / 16.168 / 15.909 / 15.920 | 15.983 / 16.115 / 15.855 / 16.130 | **16.045 s** |

**+0.21 %.** Comfortably inside noise — and the second set is the reason the
number is trustworthy. After one alternation the reading was **+0.9 %**, which
would have been a real-looking result under the 2 % bar. Running a *second*
pre-slice set moved the pre-slice mean up by 1.4 % on its own, from 15.899 to
16.123, with no code involved at all. The machine drifts ~1 % between sets, which
is larger than the effect being measured, so a single A-then-B comparison of this
slice would have been measuring the machine. journal/0060 recorded exactly this
trap and it recurred here; two alternations is now the minimum.

The expected result: the identity path adds one `is_empty()` branch per
subaerial cell per epoch and removes two multiplies and a divide from the same
spot, and calls the provider 200 times total instead of 59 million.

## Found and deliberately not fixed

- **`FLOOD_AREA = 120.0`** — a second, independent drainage-area threshold in the
  same function, asking a related hydrological question with its own unmeasured
  constant. Not part of `wet`, not one of the four consumers, arguably a fifth
  seam for the same heir. Untouched.
- **Consumer 3 is mis-wired at the level of physics**, not of code: fire dryness
  reads a waterlogging index when it wants near-surface fuel moisture at the dry
  extreme. Correct today only because the proxy's leading term is climate
  moisture. Named in the contract above; fixing it is a behaviour change.
- **The `moist` term enters `wet` with an implicit coefficient of 1.0** while the
  two bonuses carry 0.20 and 0.15. So climate moisture is ~74 % of the maximum
  wetness a cell can have, and the two terms that make waterlogging *geographic*
  — low elevation and upslope drainage — are together worth 0.35. Whether that
  ratio was intended or is an artifact of "start from `moist` and add bonuses" is
  not knowable from the code. Not touched; it is a coefficient, and this slice
  changes no coefficients.

---

*Gates green. Byte-identity proven twice: the pre-slice goldens (`2434f37`, via
journal/0060's capture) pass unchanged through the identity path, and a
registered non-identity provider that materializes the proxy reproduces the same
two hashes through the plane path. `stubs.md` gains entry 7b — which by that
file's own doctrine should have existed before this slice, and did not.*
