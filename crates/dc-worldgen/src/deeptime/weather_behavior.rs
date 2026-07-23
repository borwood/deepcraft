//! **S16 — the north-star material-behavior shape, on one real deep-time
//! behavior: subaerial bedrock → regolith weathering.**
//!
//! This module is the de-risk of the north star's core abstraction
//! (docs/design/north-star.md § Passes / § Materials): a **pass** that declares
//! its `{reads, writes}` and runs per cell; a **material behavior** that is
//! *pure* (read-only `ctx`, returns a value or a [`Transform`], never mutates);
//! and a **`ctx` capability** whose read side is the SDK surface a behavior may
//! touch and whose write side (`apply`) is owned by the pass alone. The whole
//! point is byte-identity as an **instrument**: if this shape reproduces the
//! existing `erosion::weather` phase to the bit (docs/spikes/S16 goldens
//! unmoved), the form is a clean *view* over the current height representation
//! and is potentially keepable; where it *cannot* stay byte-identical without
//! inventing deep state, that strain is the finding.
//!
//! **Fork 2 (ratified): a thin ctx-adapter over heights.** Deep time stores
//! bedrock/regolith as the scalar stocks `R` and `H` (grid.rs) plus a strata
//! record — *not* a per-cell material multiset. So this is **not** a storage
//! rewrite. The `ctx` is a per-cell *view*: [`WeatherCtx::regolith_depth`] is
//! `H`, the gate reads `R + H`, and the [`Transform::FormChange`] the behavior
//! asks for — `Structural → Loose` — *is* the existing `R -= x; H += x; dH += x`
//! transfer, performed by [`WeatherApply::apply`]. Nothing new is stored.
//!
//! **Where it strains (the diagnostic, in code).** The height tier has no single
//! outcropping *material*: `R` is basement everywhere, and the per-cell
//! weathering *rate* is a share-weighted blend over the near-surface window of
//! the `H`-record (`lithology::blend_susceptibility`), plus a second agent's
//! multiplier (periglacial frost). So the shape's `self.weatherability` (a
//! material property, dc-core) and its two-factor `base × (biotic ×
//! weatherability) × taper` rate **cannot** be the byte-identical source at this
//! tier: the adapter must deliver `weatherability` and `frost` as *per-cell ctx
//! reads* over the existing planes, and the "material" degenerates to one
//! synthetic per-cell blend. That gap is exactly the deep-cell material
//! inventory question filed as coupled to Crux 1 — out of scope to *fix* here,
//! in scope to *expose*.

use dc_core::materials::MaterialId;

/// A material's **form** on the two-plane deep grid: in-place bedrock stock `R`
/// vs loose regolith stock `H`. The axis the weathering transform moves along.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// Bedrock — the in-place `R` stock (the "structural" form).
    Structural,
    /// Regolith — the loose, mobile `H` stock (the "loose" form).
    Loose,
}

/// A **pure description** of the material change a behavior requests. The
/// behavior *names* it; the [`WeatherApply`] the pass owns *performs* it. Modeled
/// as a small closed enum rather than a closure so it is plain data that crosses
/// the (future) SDK seam unchanged (north-star § The crossing constraint).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transform {
    /// Convert `from`-form material into `to`-form material, *same material*,
    /// mass-neutral within the column. Weathering asks for
    /// `Structural → Loose` (bedrock rots to regolith in place).
    FormChange { from: Form, to: Form },
}

impl Transform {
    /// The bedrock→regolith form change weathering performs.
    #[inline]
    pub fn form_change(from: Form, to: Form) -> Transform {
        Transform::FormChange { from, to }
    }
}

/// The core context axes this pass declares it touches (north-star § Passes: a
/// pass "declares itself" over the cell/world/material API). Here as data so the
/// declaration is inspectable and, eventually, topo-checkable by the runner; the
/// runner integration is out of this spike's scope, but the *shape* of the
/// declaration is part of the deliverable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeatherAxis {
    /// The subaerial gate (`R + H` vs sea) — read.
    Subaerial,
    /// Regolith depth `H` — read (cover taper) and written (grows by the transfer).
    RegolithDepth,
    /// Bedrock stock `R` — written (shrinks by the transfer).
    BedrockDepth,
    /// The biotic-weathering multiplier plane (provider) — read.
    BioticWeathering,
    /// The outcropping bedrock material / its weatherability — read.
    BedrockMaterial,
    /// The net-ΔH strata-recorder accumulator — written.
    NetThickness,
}

/// **The read-only capability surface** a weather behavior may touch, as a
/// per-cell view over the height adapter (Fork 2). A behavior receives `&self`
/// of this and *cannot* mutate the world through it — the write path
/// ([`WeatherApply`]) is a separate, pass-owned handle. That split is the
/// capability discipline made structural: purity is not a convention a behavior
/// is trusted to honor, it is the only thing the type it holds permits.
///
/// The reads marked *(adapter over heights)* are where a per-cell material
/// inventory would replace a plane lookup; today they are the existing
/// `erosion` planes surfaced by name.
pub struct WeatherCtx {
    // Cell state (copied by value — weathering is per-cell independent, so a
    // snapshot of R/H taken before the write is exactly what the old kernel read
    // before it mutated).
    r: f64,
    h: f64,
    sea: f64,
    // Config capabilities.
    base_weathering: f64,
    h_star: f64,
    // Per-cell provider / material reads (adapter over heights).
    biotic: f64,
    weatherability: f64,
    frost: f64,
}

impl WeatherCtx {
    /// **The gate.** Is this cell above the current sea stand? Weathering is a
    /// subaerial process — a drowned cell is doing something else.
    #[inline]
    pub fn is_subaerial(&self) -> bool {
        self.r + self.h > self.sea
    }

    /// Regolith depth `H` at this cell (metres) — the loose cover already present.
    #[inline]
    pub fn regolith_depth(&self) -> f64 {
        self.h
    }

    /// The base weathering rate (metres/epoch), a world config capability.
    #[inline]
    pub fn base_weathering(&self) -> f64 {
        self.base_weathering
    }

    /// The **biotic weathering multiplier** at this cell (provider; `1.0` when the
    /// biotic layer is off). Land plants accelerate chemical weathering
    /// several-fold (ecology.md § 1).
    #[inline]
    pub fn biotic_weathering(&self) -> f64 {
        self.biotic
    }

    /// The **weatherability** of the bedrock outcropping here *(adapter over
    /// heights)*. In the north-star this is `material.props().weatherability`
    /// read off the single outcropping material; at the height tier it is the
    /// share-weighted blend over the near-surface window (there is no single
    /// material — the S16 diagnostic). `1.0` when the erodibility coupling is off.
    #[inline]
    pub fn weatherability(&self) -> f64 {
        self.weatherability
    }

    /// The **frost (periglacial) weathering multiplier** at this cell *(adapter
    /// over heights)*. `≥ 1.0` when the frost agent is on, exactly `1.0` when it
    /// is off. This is a *second agent's* attack on the same rock: the two-factor
    /// `base × (biotic × weatherability)` sketch in north-star § Materials does
    /// not have a slot for it, so the byte-identical rate has to thread it here
    /// (the S16 diagnostic — weathering is a sum/product *over agents*).
    #[inline]
    pub fn frost_weathering(&self) -> f64 {
        self.frost
    }

    /// The **alluvial-cover taper** `exp(-H / H*)` — a config-owned capability
    /// (it needs `H*`), so it lives on the ctx rather than in the behavior body.
    /// A thick regolith cover shields the bedrock beneath it from further
    /// conversion.
    #[inline]
    pub fn cover_taper(&self) -> f64 {
        (-self.h / self.h_star).exp()
    }
}

/// **A material's weathering behavior — pure.** `weather_rate` returns a
/// quantity; `weather` returns the [`Transform`] to apply. Neither may mutate
/// (the `&WeatherCtx` they hold has no write path). "The defaults are the first
/// plugins": the first-party [`BedrockWeather`] implements this natively, exactly
/// as a third-party material would.
pub trait Weather {
    /// The rate (metres this epoch) at which this material weathers in `ctx`.
    fn weather_rate(&self, ctx: &WeatherCtx) -> f64;
    /// The material change weathering produces.
    fn weather(&self, ctx: &WeatherCtx) -> Transform;
}

/// The **first-party, native** bedrock→regolith weathering behavior.
///
/// The rate preserves the existing f64 grouping **exactly** (multiplication is
/// not associative, so the grouping is load-bearing for byte-identity — the same
/// discipline the provider conversions hold):
///
/// ```text
/// base × ((biotic × weatherability) × frost) × cover_taper
/// ```
///
/// which is the north-star `base × (biotic × weatherability) × taper` **with the
/// frost agent threaded in** at `(… × frost)` — the one place the two-factor
/// sketch had to grow to carry the real, four-flags-on production arithmetic
/// (`erosion::weather_cell`: `weathering × ((wmult × sus) × frost) × exp(-H/H*)`).
pub struct BedrockWeather;

impl Weather for BedrockWeather {
    #[inline]
    fn weather_rate(&self, ctx: &WeatherCtx) -> f64 {
        ctx.base_weathering()
            * ((ctx.biotic_weathering() * ctx.weatherability()) * ctx.frost_weathering())
            * ctx.cover_taper()
    }

    #[inline]
    fn weather(&self, _ctx: &WeatherCtx) -> Transform {
        // Bedrock rots to regolith, in place, same material.
        Transform::form_change(Form::Structural, Form::Loose)
    }
}

/// **The pass-owned write handle.** Holds `&mut` into the cell's `R`, `H` and the
/// recorder's net-ΔH accumulator; `apply` scales the transform by `rate · dt`,
/// performs the declared writes, and *nothing else can*. A behavior never sees
/// this type. `dt` is the epoch step: deep-time weathering runs at `dt = 1.0`
/// (the rate the behavior returns is already the per-epoch quantity), and
/// `rate × 1.0 == rate` for every finite f64, so the scaling is byte-transparent.
pub struct WeatherApply<'a> {
    r: &'a mut f64,
    h: &'a mut f64,
    dh: &'a mut f64,
    dt: f64,
}

impl WeatherApply<'_> {
    /// Apply `transform` at `rate` (metres/epoch). Enforces the pass's declared
    /// writes: only `Structural → Loose` is defined for weathering, and it is the
    /// mass-neutral `R -= q; H += q` transfer, with `dH += q` so the strata
    /// recorder sees the loose stack grow (the record mirrors `H`, not `R`).
    #[inline]
    pub fn apply(&mut self, transform: Transform, rate: f64) {
        match transform {
            Transform::FormChange {
                from: Form::Structural,
                to: Form::Loose,
            } => {
                let q = rate * self.dt;
                *self.r -= q;
                *self.h += q;
                *self.dh += q;
            }
            // No other form change is part of the weathering pass's declared
            // write-set; a behavior that returned one is asking for a capability
            // it did not declare, and the pass declines it.
            Transform::FormChange { .. } => {}
        }
    }
}

/// **The weathering pass.** Declares its `{reads, writes}` in the north-star
/// shape and runs per deep cell per epoch. The runner (out of scope here) would
/// topo-sort it by these declarations; today `erosion::weather` calls
/// [`WeatheringPass::run_cell`] directly, which is the byte-identity seam.
pub struct WeatheringPass;

impl WeatheringPass {
    /// What the pass reads.
    pub const READS: &'static [WeatherAxis] = &[
        WeatherAxis::Subaerial,
        WeatherAxis::RegolithDepth,
        WeatherAxis::BioticWeathering,
        WeatherAxis::BedrockMaterial,
    ];
    /// What the pass writes.
    pub const WRITES: &'static [WeatherAxis] = &[
        WeatherAxis::BedrockDepth,
        WeatherAxis::RegolithDepth,
        WeatherAxis::NetThickness,
    ];

    /// Run the pass over one cell: **the gate, then ask the behavior, then the
    /// pass applies.** This is the whole north-star `run(ctx)` body for a
    /// material-transform pass, on real data. Note the discipline: the behavior
    /// is *asked* (`weather_rate`, `weather`) and never writes; the pass calls
    /// `apply`.
    #[inline]
    pub fn run_cell(behavior: &dyn Weather, ctx: &WeatherCtx, apply: &mut WeatherApply) {
        if !ctx.is_subaerial() {
            return;
        }
        // At the height tier there is exactly one (synthetic, blended) outcropping
        // material per cell, so the north-star `for m in materials_with(Weather)`
        // loop degenerates to one body. A per-cell material inventory (Crux 1) is
        // what would make this a genuine loop over several materials.
        let rate = behavior.weather_rate(ctx);
        apply.apply(behavior.weather(ctx), rate);
    }
}

/// Build a per-cell [`WeatherCtx`] and [`WeatherApply`] over the height adapter,
/// and run [`WeatheringPass::run_cell`]. This is the single call `erosion::weather`
/// makes per cell (scalar or parallel), so the reformulation is byte-identical by
/// construction: the arguments are the exact operands the old `weather_cell` read,
/// in the exact order.
///
/// `weatherability` is the blended abrasion susceptibility (`sus`), `biotic` the
/// `wmult` plane, `frost` the periglacial plane — each already `1.0` when its
/// coupling is off, so the uncoupled path stays bit-identical.
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn weather_one_cell(
    r: &mut f64,
    h: &mut f64,
    dh: &mut f64,
    sea: f64,
    base_weathering: f64,
    h_star: f64,
    biotic: f64,
    weatherability: f64,
    frost: f64,
) {
    let ctx = WeatherCtx {
        r: *r,
        h: *h,
        sea,
        base_weathering,
        h_star,
        biotic,
        weatherability,
        frost,
    };
    let mut apply = WeatherApply { r, h, dh, dt: 1.0 };
    WeatheringPass::run_cell(&BedrockWeather, &ctx, &mut apply);
}

/// The **reference material** whose `weatherability` property the height-tier
/// blend generalizes — the outcropping-material identity the adapter would hand a
/// behavior if the deep cell tracked one. Fixed per class (see
/// `lithology::Litho::reference_material`); this is the single-material authoring
/// path the shape's unit test exercises.
pub const REFERENCE_WEATHER_MATERIAL: MaterialId = MaterialId::MUDSTONE;

#[cfg(test)]
mod tests {
    use super::*;

    /// A ctx built the way a *single-material* authoring path would build it —
    /// reading `material.props().weatherability` straight, no blend. This is the
    /// north-star shape at full strength (the behavior consumes the material
    /// property), proving the field is a genuine consumer, not a leaked summary.
    fn single_material_ctx(m: MaterialId, h: f64) -> WeatherCtx {
        WeatherCtx {
            r: 100.0,
            h,
            sea: 0.0,
            base_weathering: 0.01,
            h_star: 2.0,
            biotic: 1.0,
            weatherability: m.props().weatherability,
            frost: 1.0,
        }
    }

    #[test]
    fn the_gate_blocks_a_drowned_cell() {
        let mut ctx = single_material_ctx(REFERENCE_WEATHER_MATERIAL, 0.0);
        ctx.r = -10.0;
        ctx.h = 0.0;
        ctx.sea = 0.0;
        assert!(!ctx.is_subaerial());
        let (mut r, mut h, mut dh) = (ctx.r, ctx.h, 0.0);
        let mut apply = WeatherApply {
            r: &mut r,
            h: &mut h,
            dh: &mut dh,
            dt: 1.0,
        };
        WeatheringPass::run_cell(&BedrockWeather, &ctx, &mut apply);
        assert_eq!(
            (r, h, dh),
            (-10.0, 0.0, 0.0),
            "drowned cell must not weather"
        );
    }

    #[test]
    fn form_change_is_the_mass_neutral_r_to_h_transfer() {
        // The keepable core: Structural→Loose is exactly `R -= q; H += q; dH += q`.
        let ctx = single_material_ctx(REFERENCE_WEATHER_MATERIAL, 0.5);
        let (mut r, mut h, mut dh) = (ctx.r, ctx.h, 0.0);
        let mut apply = WeatherApply {
            r: &mut r,
            h: &mut h,
            dh: &mut dh,
            dt: 1.0,
        };
        WeatheringPass::run_cell(&BedrockWeather, &ctx, &mut apply);
        let q = ctx.base_weathering()
            * ((ctx.biotic_weathering() * ctx.weatherability()) * ctx.frost_weathering())
            * ctx.cover_taper();
        assert_eq!(r, 100.0 - q);
        assert_eq!(h, 0.5 + q);
        assert_eq!(dh, q);
        // Mass neutral in the column: R falls and H rises by the *same* q (the one
        // operand `apply` computed), which is what the transfer conserves — pinned
        // via the shared `q` above, not a float-lossy `(100-r) == (h-0.5)`.
    }

    #[test]
    fn the_material_property_drives_the_rate_in_the_single_material_path() {
        // Softer rock (higher weatherability) weathers faster under an identical
        // ctx — the behavior genuinely reads `props().weatherability`.
        let peat = single_material_ctx(MaterialId::PEAT, 0.5);
        let granite = single_material_ctx(MaterialId::GRANITE, 0.5);
        assert!(
            BedrockWeather.weather_rate(&peat) > BedrockWeather.weather_rate(&granite),
            "peat must weather faster than granite basement"
        );
    }

    #[test]
    fn dt_of_one_is_byte_transparent_and_grouping_is_pinned() {
        // Pin the exact f64 grouping the byte-identity proof rests on.
        let ctx = single_material_ctx(MaterialId::SANDSTONE, 0.3);
        let expected = 0.01
            * ((1.0 * MaterialId::SANDSTONE.props().weatherability) * 1.0)
            * (-0.3f64 / 2.0).exp();
        assert_eq!(BedrockWeather.weather_rate(&ctx), expected);
    }
}
