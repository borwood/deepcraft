# S16 — Weathering behavior-shape spike results

**Date:** 2026-07-23 · **Branch:** `worktree-agent-a60c2e872cad13710` ·
**Write-set:** `dc-worldgen` (new `deeptime::weather_behavior` + the `erosion::weather`
site), `dc-core` (a `weatherability` `MaterialProps` axis).

The make-or-break de-risk of the north star's core abstraction
(`docs/design/north-star.md`): the **native, uniform, self-declaring
Pass / Material / ctx / Transform** shape, implemented on **one real deep-time
behavior** — the subaerial bedrock→regolith weathering conversion
(`erosion::weather`) — over the **thin ctx-adapter-over-heights** (Fork 2). The
primary deliverable is what it teaches; byte-identity is the **instrument**.

---

## 1. The shape as implemented

Four types, in `crates/dc-worldgen/src/deeptime/weather_behavior.rs`:

```rust
// FORM axis + a pure, plain-data TRANSFORM (crosses the SDK seam unchanged).
enum Form { Structural, Loose }                     // R stock vs H stock
enum Transform { FormChange { from: Form, to: Form } }
impl Transform { fn form_change(from: Form, to: Form) -> Transform }

// The ctx CAPABILITY — read side only (what a behavior may touch = the SDK).
struct WeatherCtx { /* per-cell view over the height adapter */ }
impl WeatherCtx {
    fn is_subaerial(&self) -> bool        // gate: R + H > sea
    fn regolith_depth(&self) -> f64       // H
    fn base_weathering(&self) -> f64      // config capability
    fn biotic_weathering(&self) -> f64    // provider plane (wmult)
    fn weatherability(&self) -> f64       // (adapter) blended abrasion sus
    fn frost_weathering(&self) -> f64     // (adapter) periglacial agent multiplier
    fn cover_taper(&self) -> f64          // exp(-H/H*), config-owned
}

// The BEHAVIOR — pure: read-only ctx, returns a value / a Transform, never mutates.
trait Weather {
    fn weather_rate(&self, ctx: &WeatherCtx) -> f64;
    fn weather(&self, ctx: &WeatherCtx) -> Transform;
}
struct BedrockWeather;                               // first-party = the first plugin
impl Weather for BedrockWeather {
    fn weather_rate(&self, ctx) -> f64 {
        ctx.base_weathering()
            * ((ctx.biotic_weathering() * ctx.weatherability()) * ctx.frost_weathering())
            * ctx.cover_taper()
    }
    fn weather(&self, _) -> Transform { Transform::form_change(Structural, Loose) }
}

// The WRITE side — pass-owned, a behavior never sees this type.
struct WeatherApply<'a> { r, h, dh: &'a mut f64, dt: f64 }
impl WeatherApply<'_> {
    fn apply(&mut self, t: Transform, rate: f64)     // scales rate·dt, does the write
}

// The PASS — declares {reads, writes}, runs per cell.
struct WeatheringPass;
impl WeatheringPass {
    const READS:  &[WeatherAxis];   // Subaerial, RegolithDepth, BioticWeathering, BedrockMaterial
    const WRITES: &[WeatherAxis];   // BedrockDepth, RegolithDepth, NetThickness
    fn run_cell(behavior: &dyn Weather, ctx: &WeatherCtx, apply: &mut WeatherApply) {
        if !ctx.is_subaerial() { return; }           // the gate
        let rate = behavior.weather_rate(ctx);       // ask
        apply.apply(behavior.weather(ctx), rate);    // PASS applies
    }
}
```

**The capability discipline is structural, not conventional.** A behavior holds a
`&WeatherCtx`, which exposes no write path; the only writer, `WeatherApply`, is
built and held by the pass and never handed to a behavior. Purity is therefore a
property of the *type a behavior can name*, not a rule it is trusted to follow —
which is precisely the north star's "the `ctx` is a capability, not a
god-object." `dt = 1.0` at the deep tier (the rate is already per-epoch), and
`rate × 1.0 == rate` for finite f64, so the scaling is byte-transparent.

---

## 2. The byte-identity VERDICT — **UNMOVED** (goldens hold)

> `GOLDEN_SURFACE = 0x176D_40F1_1CCB_006A`
> `GOLDEN_RECORD  = 0xC9C6_D6F6_E908_9653`
> (`providers_common/mod.rs`; asserted by `providers_golden.rs`.)

**Gate evidence (all `--release`, from this worktree — `Compiling`/`Checking
dc-core`/`dc-worldgen` paths verified against `.claude\worktrees\agent-a60c…`):**

- `providers_golden::the_production_world_still_hashes_to_the_pre_slice_goldens
  ... ok` — **the goldens are unmoved.** (`the_fingerprint_is_reproducible_within_a_build
  ... ok` too.)
- `cargo clippy -p dc-core -p dc-worldgen --all-targets --release -- -D warnings`
  → `Finished`, zero warnings.
- `cargo fmt --all --check` → clean.
- dc-core lib: `test result: ok. 141 passed` incl.
  `materials::tests::weatherability_orders_soft_over_hard_with_the_reference_at_one`.
- dc-worldgen `weather_behavior` shape tests: 4 passed
  (`the_gate_blocks_a_drowned_cell`, `form_change_is_the_mass_neutral_r_to_h_transfer`,
  `the_material_property_drives_the_rate_in_the_single_material_path`,
  `dt_of_one_is_byte_transparent_and_grouping_is_pinned`).

The `--workspace` gate was scoped to the two changed crates: the concurrent
dc-client agent had the **running** `dc-client.exe` file-locked, so a workspace
relink of dc-client would fail for reasons unrelated to this change, and
dc-client is outside this spike's write-set.

The reformulation is byte-identical to `erosion::weather_cell`, so **both goldens
are unmoved** — verified by running `providers_golden` after `cargo clean -p
dc-worldgen dc-core`. The identity rests on two facts:

1. **`form_change(Structural → Loose)` maps cleanly onto the existing R→H scalar
   transfer with NO new deep state.** `apply` does exactly `R -= q; H += q;
   dH += q` (`q = rate·dt`, `dt = 1`), which is the old kernel's body verbatim.
   The mass-neutral in-column transfer *is* the transform; the strata recorder's
   `dH += q` (the record mirrors `H`) rides along untouched. This is the
   keepable-core result the diagnostic hoped for: the form is a clean **view**
   over the height representation, inventing nothing.

2. **The rate grouping is preserved to the bit.** f64 multiplication is not
   associative, so the grouping is load-bearing. The production golden runs
   **all four flags on** (biotic, erodibility, tectonic_history, full_agents), so
   the real per-cell rate is
   `weathering × ((wmult × sus) × frost) × exp(-H/H*)`. `BedrockWeather::
   weather_rate` reproduces that as `base × ((biotic × weatherability) × frost) ×
   cover_taper` with the **same operands in the same order**. The `WeatherCtx`
   copies `R, H` by value *before* the write, so the gate and taper read the same
   pre-update state the old kernel read. Scalar and parallel paths both route
   through `weather_one_cell`, so the S9b scalar↔parallel byte-identity is
   preserved too.

---

## 3. The ctx-adapter-over-heights verdict — **holds for the transfer, STRAINS on material identity**

The adapter holds **cleanly** on the state axes: `regolith_depth() = H`, the
gate = `R + H > sea`, and the write = the R→H transfer. No new plane, no new
per-cell field. Fork 2's bet — "form is a view over R/H" — is **confirmed for the
mechanism**.

It **strains, exactly as the brief predicted it might, on "what is the bedrock
material and what is its weatherability"** — and that strain is the finding, not
a failure:

- **There is no single outcropping material at a deep cell.** `R` is basement
  everywhere; the weathering *rate* is a **share-weighted blend** over the
  near-surface window of the `H`-record
  (`lithology::blend_susceptibility(shares, table)`), not a lookup on one
  material. So the north-star `for m in ctx.materials_with(Weather)` loop
  **degenerates to one synthetic per-cell body**, and `self.weatherability` — a
  material property — **cannot** be the byte-identical source. The adapter
  delivers `weatherability()` as the *blend* through the ctx. Making it a genuine
  loop over real materials reading `props().weatherability` per cell **is** the
  deep-cell material-inventory upgrade, which is coupled to Crux 1 and out of
  scope here. This spike **localizes** that upgrade to one method
  (`WeatherCtx::weatherability`) and one missing capability
  (`materials_with`).

- **Weathering is a sum/product over agents, and the two-factor sketch has no
  slot for the second agent.** The north-star `base × (biotic × weatherability) ×
  taper` has room for one material factor; production weathering already carries
  **two** agent multipliers — abrasion (the `weatherability` stand-in) and the
  periglacial **frost** agent. Byte-identity forced the shape to grow the frost
  factor at `(… × frost)`. That is the honest shape of the physics
  (`lithology.rs`: "in-place weathering is the sum of every agent's attack"), and
  it means the material-behavior interface must eventually express **a reduction
  over an agent set**, not a fixed product — the same door the karst/dissolution
  agent will walk through.

Neither strain required inventing deep state to *fake* byte-identity, so the
verdict is the clean one: **the height representation is a faithful view for the
weathering transfer; it is lossy about material identity, which is the already-
filed inventory question.**

---

## 4. Authoring ergonomics

Adding the *first* behavior costs the scaffolding (the four types above, ~130
lines with docs). Adding the **next** behavior — say combustion or diagenesis —
costs only: one `impl Weather`-style trait's two methods, plus any new ctx reads
it needs and any new `Transform` variant. The behavior body reads cleanly and
declaratively:

```rust
fn weather_rate(&self, ctx) -> f64 {
    ctx.base_weathering() * ((ctx.biotic_weathering() * ctx.weatherability()) * ctx.frost_weathering()) * ctx.cover_taper()
}
fn weather(&self, _) -> Transform { Transform::form_change(Structural, Loose) }
```

**Boilerplate to add one behavior:** the trait impl (2 methods) + a unit struct.
Everything hazardous — the write, the `rate·dt` scaling, the gate, the parallel
driver — lives in the pass and the ctx, authored once. The **ergo tax is the
crossing constraint**, not the behavior author's expressiveness: the ctx surface
is plain-data getters, which is exactly the `abi_stable`-∩-WASM shape the SDK
seam will need. The one wart is that at the height tier the ctx must expose
adapter-blend reads (`weatherability`, `frost`) that a cell-inventory world would
replace with genuine `material.props()` reads — the author sees a slightly
richer ctx than the final shape wants.

**The `weatherability` axis reads well as a material property** and is genuinely
consumed: `BedrockWeather::weather_rate` reads it, the single-material shape
tests drive it off `MaterialId::PEAT/GRANITE/SANDSTONE.props().weatherability`,
and it is pinned (dc-core `weatherability_orders_soft_over_hard_with_the_reference_at_one`)
to order soft→hard with the reference clastic at exactly `1.0`. It is a **distinct
axis from mechanical `smash` resistance** (a basalt is tough to dig yet rots to
clay readily), for the same reason `solubility` is distinct — so it does **not**
reintroduce the one-number-erodibility trap `lithology.rs` exists to avoid.

---

## 5. Keepable or spike-only? — **KEEPABLE (the shape), with one seam left open**

Recommendation to the integrator: **keep the shape.** The Pass / Material / ctx /
Transform types are a faithful, byte-identical reformulation of a real production
phase; they move the code *toward* the north star (a self-declaring pass, a pure
behavior, a capability ctx) with zero regression and zero new deep state. The
`weatherability` axis is a real, ordered, consumed material property.

The one thing **not** to over-read: `WeatherCtx::weatherability()` and
`frost_weathering()` are **adapter blends**, not per-material reads, because the
height tier has no cell material inventory. That is the deep-cell-inventory work
(Crux 1). Until it lands, the material property is the **authority** and the
deep-time blend is its **summary** — pinned to agree in ordering, per the
"a summary is not an authority" convention.

**North-star convergence:** this converges the deep-time weathering phase onto the
ratified Pass/Material/ctx/Transform shape byte-for-byte, proving `form_change`
is a clean view over the height stocks while pinpointing the single remaining
seam (a per-cell material inventory) that a full material-behavior world needs.
