//! **The first real cellular behavior on the deep-cell working inventory:
//! subaerial weathering as a SUM of agent terms** (material-behavior.md §4,
//! DECIDED 2026-07-24).
//!
//! S16 ([`super::weather_behavior`]) taught the pass/behavior/`ctx` shape over the
//! scalar `R`/`H` height adapter, kept **byte-identical** to the legacy
//! `erosion::weather` by preserving its *product* arithmetic
//! (`base × biotic × weatherability × frost × taper`). That scaffold is discharged.
//! This module is its honest successor on the **real working inventory**
//! ([`super::inventory`]): the same `Structure→Loose` weathering, but the rate is
//! the **sum of agent terms**
//!
//! ```text
//! rate = cover_taper × Σ_a (driver_a × susceptibility_{m,a})
//! ```
//!
//! and it is deliberately **NOT byte-identical** to S16's product — a product
//! falsely zeroes frost where biota is zero (frost shatters bare rock), and only a
//! sum makes each agent's *share* well-defined:
//! `share_a = cover_taper × driver_a × susceptibility_{m,a}`, `Σ share_a = rate`.
//! Each agent applies **its own** edge through a cause-scoped [`InvCtx`], so the
//! commit yields **one fact per agent** (§1), carrying the agent as its `cause`.
//!
//! **What it weathers.** The record's units are all `Loose`, so the source is the
//! materialized bedrock `Structure` seam (STUB #16, [`super::inventory`]): weathering
//! converts basement `Structure` → basement `Loose` (a saprolite/regolith front at
//! the basement contact). The loose product inherits the bedrock's identity — which
//! *material* is a stand-in until the genesis/emplacement heir (stub #16).
//!
//! **Susceptibility axis — the refinement seam.** `MaterialProps` carries one
//! weathering-susceptibility field, `weatherability`. **Chemical** uses it as its
//! native axis. **Biotic** and **Frost** *reuse* `weatherability` as their
//! susceptibility here, because no distinct per-agent weathering-susceptibility axis
//! exists on `MaterialProps` yet (the `LithoResistance` axes are erosion-agent
//! *resistances* for the height tier — a different model). Growing those axes is a
//! checked material-sheet extension, deliberately **not** ballooned into this slice;
//! the reuse is the annotated seam its heir replaces.
//!
//! **Runtime clock.** This runs in **deeptime** (the compiler, gen-time is free).
//! Since **journal/0094 (Movement 3)** it is a per-epoch **process**: the
//! `dc:deep/weather_inventory` runner pass fires [`weather_epoch`] **every epoch
//! inside the deep-time loop**, weathering each subaerial cell's bedrock seam on that
//! epoch's live terrain and **accumulating** the band across the run (the per-cell
//! accumulator is keyed to a stable bedrock sentinel so the growing record cannot
//! shift its index — see [`weather_bedrock_epoch`]). It is gated behind
//! [`DeepConfig::weather_inventory`](super::grid::DeepConfig::weather_inventory);
//! off (the default) the pass is absent, the field carries no ledgers, and the
//! collapsed world is byte-identical, the S-5 identity default. `weather_cell` /
//! `weather_column` (the original one-shot-per-chapter helpers) survive for the
//! unit tests.

use dc_core::materials::MaterialId;

use super::grid::{DeepConfig, DeepGrid};
use super::inventory::{
    BEDROCK_SEAM_MATERIAL, Cause, FactLedger, InvForm, build_working, commit_chapter,
};
use super::recorder::DeepStrata;

/// The three weathering agents that SUM on the `Structure→Loose` edge, each a
/// `(cause, driver, susceptibility)` term. Dissolution is dormant (§10, no soluble
/// rock) and so is not summed here.
pub const WEATHERING_AGENTS: [Cause; 3] = [Cause::Chemical, Cause::Biotic, Cause::Frost];

/// Per-cell environmental drivers + config the weathering rate reads. Each field is
/// a value the deeptime run already has in hand at a chapter boundary.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct WeatherInputs {
    /// Base chemical weathering rate (m/epoch) — `DeepConfig::weathering`. The
    /// **chemical** agent's driver, and the common rate unit the biotic/frost
    /// multipliers scale.
    pub weathering: f64,
    /// Alluvial-cover shielding scale `H*` (m) — `DeepConfig::h_star`. A thick loose
    /// cover shields the bedrock: `cover_taper = exp(-H / H*)`.
    pub h_star: f64,
    /// Regolith depth `H` (m) at this cell — the loose cover shielding the bedrock.
    pub regolith_h: f64,
    /// **Biotic** driver multiplier (`grid.bio_weather`; `1.0` when the biotic layer
    /// is off). Land plants/soil acids accelerate chemical attack several-fold.
    pub biotic: f64,
    /// **Frost** driver multiplier (`erosion.frost`; `1.0` when the frost agent is
    /// off). Independent of biota — the reason the model is a sum, not a product.
    pub frost: f64,
}

impl WeatherInputs {
    /// `cover_taper = exp(-H / H*)` — the shared cover-shielding factor.
    #[inline]
    pub fn cover_taper(&self) -> f64 {
        (-self.regolith_h / self.h_star).exp()
    }

    /// This agent's **driver** term (m/epoch). Chemical is the base rate; biotic and
    /// frost are that rate scaled by their environmental multiplier — so where a
    /// multiplier is `1.0` (agent off) the term is still a *base-rate* contribution,
    /// which is exactly the additive independence the sum model exists to give
    /// (`Dissolution` never reaches here — [`WEATHERING_AGENTS`]).
    #[inline]
    pub fn driver(&self, cause: Cause) -> f64 {
        match cause {
            Cause::Chemical => self.weathering,
            Cause::Biotic => self.weathering * self.biotic,
            Cause::Frost => self.weathering * self.frost,
            Cause::Dissolution => 0.0,
        }
    }
}

/// The **susceptibility** of `material` to `cause` (higher = weathers faster).
///
/// Chemical reads `weatherability` as its native axis. Biotic and Frost **reuse**
/// `weatherability` — the annotated refinement seam (module docs): no distinct
/// per-agent weathering-susceptibility axis exists on `MaterialProps` yet, and this
/// slice does not add one. The heir grows those axes; every call site here is where
/// its obligation reads.
#[inline]
pub fn susceptibility(material: MaterialId, cause: Cause) -> f64 {
    let weatherability = material.props().weatherability;
    match cause {
        // Native axis.
        Cause::Chemical => weatherability,
        // REFINEMENT SEAM: reuse `weatherability` until a biotic/frost weathering
        // susceptibility axis exists on the material sheet.
        Cause::Biotic | Cause::Frost => weatherability,
        Cause::Dissolution => 0.0,
    }
}

/// One agent's **share** of the weathering (m this chapter):
/// `cover_taper × driver_a × susceptibility_{m,a}`.
#[inline]
pub fn agent_share(inputs: &WeatherInputs, material: MaterialId, cause: Cause) -> f64 {
    inputs.cover_taper() * inputs.driver(cause) * susceptibility(material, cause)
}

/// The summed weathering **rate** (m this chapter): `Σ_a agent_share`. The quantity
/// the pass moves `Structure→Loose`, and `Σ share_a == rate` by construction.
#[inline]
pub fn weather_rate(inputs: &WeatherInputs, material: MaterialId) -> f64 {
    WEATHERING_AGENTS
        .iter()
        .map(|&a| agent_share(inputs, material, a))
        .sum()
}

/// **Run the weathering pass over one deep cell for one chapter** and commit the
/// facts into `ledger` (the S-2 compiled artifact).
///
/// The gate is subaerial-only in spirit; here it is expressed as "there is bedrock
/// to weather and the cell is above water" via the caller supplying a real cell
/// (the deeptime driver only calls this for subaerial cells). For each agent it
/// opens a **cause-scoped** [`InvCtx`] and moves its share `Structure→Loose` on the
/// bedrock seam span — so the commit lands **one fact per agent**.
///
/// Returns the total rate moved (a measurement).
pub fn weather_cell(
    strata: &DeepStrata,
    ledger: &mut FactLedger,
    chapter: u8,
    inputs: &WeatherInputs,
) -> f64 {
    let mut inv = build_working(strata, ledger);
    // The bedrock seam is the LAST span (index == strata.units.len()).
    let bedrock_span = inv.spans.len() - 1;
    let material = BEDROCK_SEAM_MATERIAL;
    let mut total = 0.0;
    for &cause in &WEATHERING_AGENTS {
        let share = agent_share(inputs, material, cause);
        if share <= 0.0 {
            continue;
        }
        let moved = inv.ctx_for(chapter, cause).move_form(
            bedrock_span,
            material,
            InvForm::Structure,
            InvForm::Loose,
            share,
        );
        total += moved;
    }
    commit_chapter(&mut inv, ledger);
    total
}

/// Build a fresh [`FactLedger`] for `strata` (bedrock-augmented) and weather it for
/// `chapters` chapters, re-deriving `base + facts` each chapter (S-2). Kept to one or
/// a few chapters this slice — the multi-chapter feedback loop is a later refinement
/// (material-behavior.md §5 loop-carried feedback).
pub fn weather_column(strata: &DeepStrata, chapters: u8, inputs: &WeatherInputs) -> FactLedger {
    let mut ledger = FactLedger::empty_with_bedrock(strata);
    for chapter in 0..chapters {
        weather_cell(strata, &mut ledger, chapter, inputs);
    }
    ledger
}

// ===========================================================================
// Movement 3 (journal/0094): weathering as an IN-LOOP, PER-EPOCH, ACCUMULATING
// process — the `dc:deep/weather_inventory` runner pass, discharging stub #17.
// ===========================================================================

/// **Weather the bedrock seam for ONE epoch firing** on this epoch's live inputs,
/// appending the moved `Structure→Loose` facts (one per agent) into `ledger` — the
/// per-cell **saprolite accumulator**.
///
/// **The span-index crux (journal/0094).** `ledger` is a **bedrock-only** ledger
/// (built against an empty record via [`FactLedger::empty_with_bedrock`] of a default
/// [`DeepStrata`]), so the bedrock seam sits at the **stable sentinel slot 0** and its
/// key does NOT shift as the deep-time record grows unit-by-unit across epochs (the
/// deposition pass appends a unit every epoch, so `strata.units.len()` — the numeric
/// bedrock index of the *record-keyed* ledger — moves; keying the accumulator to the
/// empty-record slot 0 is invariant). Fire it every epoch and the shares accumulate
/// into one growing band; [`finalize_ledgers`] later re-keys slot 0 onto the final
/// record's bedrock index for the collapse consumer.
///
/// **`dt` is live** (journal/0090 deferred this to Movement 3): each agent's share
/// scales by `dt` (the pass's phase length), so a coarser cadence weathers
/// proportionally more per firing — `share ∝ dt`. At `period = 1`, `dt = 1.0` and a
/// firing is one epoch. Returns the total metres moved this firing.
pub fn weather_bedrock_epoch(
    ledger: &mut FactLedger,
    chapter: u8,
    inputs: &WeatherInputs,
    dt: f64,
) -> f64 {
    // build_working over an EMPTY record ⇒ one span, the bedrock seam, at index 0
    // (== spans.len()-1). Re-derived from `base + facts` each firing (S-2), so the
    // accumulated Loose composes forward and the Structure source depletes honestly.
    let empty = DeepStrata::default();
    let mut inv = build_working(&empty, ledger);
    let bedrock_span = inv.spans.len() - 1;
    let material = BEDROCK_SEAM_MATERIAL;
    let mut total = 0.0;
    for &cause in &WEATHERING_AGENTS {
        let share = agent_share(inputs, material, cause) * dt;
        if share <= 0.0 {
            continue;
        }
        total += inv.ctx_for(chapter, cause).move_form(
            bedrock_span,
            material,
            InvForm::Structure,
            InvForm::Loose,
            share,
        );
    }
    // **A-4 fold (journal/0096): the second merger is gone.** This used to be
    // followed by a `coalesce_facts` sweep over every slot, because
    // `commit_chapter` merged only *consecutive* identical edges while successive
    // firings interleave the three agents (chem, biotic, frost, chem, …) — so
    // nothing ever merged and the facts grew 3-per-firing until the sweep reaped
    // them. `commit_chapter` now searches the slot, which is the same merge done
    // once at the point of writing; the accumulator stays bounded at ≤ one fact
    // per agent per chapter and the band (Σ fractions) is unchanged.
    commit_chapter(&mut inv, ledger);
    total
}

/// **The `dc:deep/weather_inventory` pass body over the whole grid for one epoch.**
/// For each cell standing above the **contemporaneous** sea stand (`r + h >
/// sea_level` this epoch — the honest live version of `build_ledgers`' post-hoc
/// record proxy), weather its bedrock seam once on this epoch's live drivers
/// (regolith `H`, biotic multiplier, frost multiplier), accumulating into
/// `ledgers[i]`. Purely subaqueous cells this epoch are skipped.
///
/// `ledgers` are the per-cell **bedrock-only** saprolite accumulators (empty ⇒ the
/// pass is off ⇒ nothing happens). Reads terrain/frost/biota; **writes only the
/// ledger sidecar — never `r`/`h`/the record** (the two-authorities split,
/// material-behavior.md §11: this pass owns *material composition*, the height-tier
/// `dc:deep/weather` pass owns the `R`/`H` budget, and they do not touch each other).
pub fn weather_epoch(
    ledgers: &mut [FactLedger],
    grid: &DeepGrid,
    frost: &[f64],
    chapter: u8,
    sea_level: f64,
    dt: f64,
    cfg: &DeepConfig,
) {
    if ledgers.is_empty() {
        return;
    }
    let bio = &grid.bio_weather;
    for (i, ledger) in ledgers.iter_mut().enumerate() {
        // Contemporaneous subaerial gate: this cell stood above THIS epoch's stand.
        if grid.r[i] + grid.h[i] <= sea_level {
            continue;
        }
        let inputs = WeatherInputs {
            weathering: cfg.weathering,
            h_star: cfg.h_star,
            regolith_h: grid.h[i],
            biotic: bio.get(i).map_or(1.0, |&b| f64::from(b)),
            frost: frost.get(i).copied().unwrap_or(1.0),
        };
        weather_bedrock_epoch(ledger, chapter, &inputs, dt);
    }
}

/// **Re-key the per-cell bedrock-only accumulators onto the final record.** During
/// the loop each accumulator holds its bedrock facts at the stable sentinel slot 0
/// ([`weather_bedrock_epoch`]); the collapse consumer expects them at the bedrock
/// index of the *final* record (`strata.units.len()` — the LAST slot, what
/// [`FactLedger::weathering_product_m`] and `ledger_at_voxel` read). This moves
/// slot 0 → that slot, producing the index-parallel [`FactLedger`] sidecar the
/// `DeepField` carries. Index-parallel to `strata`; a never-weathered cell yields an
/// empty (identity) ledger, so the sidecar stays byte-identical where nothing fired.
pub fn finalize_ledgers(accumulators: Vec<FactLedger>, strata: &[DeepStrata]) -> Vec<FactLedger> {
    accumulators
        .into_iter()
        .zip(strata)
        .map(|(acc, s)| {
            let mut ledger = FactLedger::empty_with_bedrock(s);
            let bedrock_slot = s.units.len();
            // The accumulator's slot 0 is the bedrock seam (empty-record build).
            if let Some(bedrock_facts) = acc.facts.into_iter().next()
                && !bedrock_facts.is_empty()
            {
                ledger.facts[bedrock_slot] = bedrock_facts;
            }
            ledger
        })
        .collect()
}

/// An empty per-cell **bedrock-only** saprolite accumulator — the stable-keyed
/// ledger [`weather_bedrock_epoch`] accumulates into (bedrock seam at slot 0,
/// invariant to record growth). One per deep cell, built before the loop.
pub fn empty_accumulator() -> FactLedger {
    FactLedger::empty_with_bedrock(&DeepStrata::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deeptime::inventory::compose_bedrock;
    use crate::deeptime::recorder::{Aridity, DepEnv, DepTag, EnergyBand};

    fn tag(env: DepEnv, energy: EnergyBand) -> DepTag {
        DepTag::mineral(env, Aridity::Humid, energy)
    }

    fn sample_record() -> DeepStrata {
        let mut s = DeepStrata::default();
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 2.7, 0);
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::Low), 1.1, 0);
        s
    }

    fn inputs() -> WeatherInputs {
        WeatherInputs {
            weathering: 0.02,
            h_star: 2.0,
            regolith_h: 1.0,
            biotic: 1.5,
            frost: 2.0,
        }
    }

    #[test]
    fn rate_is_the_sum_of_the_three_agent_shares() {
        let inp = inputs();
        let m = BEDROCK_SEAM_MATERIAL;
        let sum = agent_share(&inp, m, Cause::Chemical)
            + agent_share(&inp, m, Cause::Biotic)
            + agent_share(&inp, m, Cause::Frost);
        assert!((weather_rate(&inp, m) - sum).abs() < 1e-15);
    }

    #[test]
    fn frost_acts_even_where_biota_is_absent() {
        // The whole reason the model is a SUM: with biota off (multiplier 1.0) the
        // rate is strictly larger than the base chemical rate, because frost adds its
        // own independent term. A product world would have collapsed frost's
        // contribution the moment biota were absent.
        let mut inp = inputs();
        inp.biotic = 1.0;
        inp.frost = 3.0;
        let m = BEDROCK_SEAM_MATERIAL;
        let chem_only = agent_share(&inp, m, Cause::Chemical);
        assert!(
            weather_rate(&inp, m) > chem_only * 2.0,
            "frost + biotic add real independent terms on top of chemical"
        );
    }

    #[test]
    fn one_fact_per_agent_and_shares_sum_to_the_move() {
        let strata = sample_record();
        let inp = inputs();
        let ledger = weather_column(&strata, 1, &inp);
        // Bedrock slot is the last (index == units.len()).
        let bedrock = ledger.facts_for(strata.units.len());
        assert_eq!(bedrock.len(), 3, "one fact per weathering agent");
        let mut causes: Vec<Cause> = bedrock.iter().map(|f| f.cause()).collect();
        causes.sort_by_key(|c| c.name());
        assert_eq!(causes, vec![Cause::Biotic, Cause::Chemical, Cause::Frost]);
        // Every fact is the Structure→Loose edge on the basement material.
        for f in bedrock {
            assert_eq!(f.from(), (BEDROCK_SEAM_MATERIAL, InvForm::Structure));
            assert_eq!(f.to(), (BEDROCK_SEAM_MATERIAL, InvForm::Loose));
        }
        // Σ shares == total move == the composed Loose product.
        let sum_shares: f64 = bedrock.iter().map(|f| f.fraction_m()).sum();
        let product_m = ledger.weathering_product_m(strata.units.len());
        assert!((sum_shares - product_m).abs() < 1e-12);
        assert!((sum_shares - weather_rate(&inp, BEDROCK_SEAM_MATERIAL)).abs() < 1e-12);
    }

    #[test]
    fn mass_conservation_bedrock_structure_to_loose() {
        let strata = sample_record();
        let inp = inputs();
        let ledger = weather_column(&strata, 1, &inp);
        let comp = compose_bedrock(ledger.facts_for(strata.units.len()));
        let total: f64 = comp.iter().map(|p| p.quantity_m).sum();
        // Structure→Loose is a form change: total metres conserved at the seam's
        // provisional thickness (nothing minted, nothing dissolved).
        assert!(
            (total - crate::deeptime::inventory::BEDROCK_SEAM_THICKNESS_M).abs() < 1e-9,
            "form change conserves mass"
        );
        let loose: f64 = comp
            .iter()
            .filter(|p| p.form == InvForm::Loose)
            .map(|p| p.quantity_m)
            .sum();
        assert!(loose > 0.0, "weathering produced loose regolith");
    }

    #[test]
    fn empty_inputs_are_the_identity_floor() {
        // Zero base rate ⇒ zero shares ⇒ no facts ⇒ the bedrock is unweathered and
        // the product is zero (the S-5 identity default at the behavior level).
        let strata = sample_record();
        let inp = WeatherInputs {
            weathering: 0.0,
            ..inputs()
        };
        let ledger = weather_column(&strata, 1, &inp);
        assert!(ledger.is_empty(), "no weathering ⇒ empty ledger");
        assert_eq!(ledger.weathering_product_m(strata.units.len()), 0.0);
    }

    // --- Movement 3 (journal/0094): the in-loop accumulating process -----------

    /// The bedrock band (composed `Loose`) held in a bedrock-only accumulator (slot 0).
    fn acc_band(acc: &FactLedger) -> f64 {
        compose_bedrock(acc.facts_for(0))
            .iter()
            .filter(|p| p.form == InvForm::Loose)
            .map(|p| p.quantity_m)
            .sum()
    }

    #[test]
    fn one_fact_per_agent_per_firing() {
        // A single firing on a fresh accumulator commits exactly one fact per active
        // agent (the S18 §1 invariant, preserved by the in-loop path).
        let mut acc = empty_accumulator();
        let moved = weather_bedrock_epoch(&mut acc, 0, &inputs(), 1.0);
        let bedrock = acc.facts_for(0);
        assert_eq!(
            bedrock.len(),
            3,
            "one fact per weathering agent, one firing"
        );
        let mut causes: Vec<Cause> = bedrock.iter().map(|f| f.cause()).collect();
        causes.sort_by_key(|c| c.name());
        assert_eq!(causes, vec![Cause::Biotic, Cause::Chemical, Cause::Frost]);
        // Σ shares == the composed band == the returned move.
        assert!((acc_band(&acc) - moved).abs() < 1e-12);
        assert!((moved - weather_rate(&inputs(), BEDROCK_SEAM_MATERIAL)).abs() < 1e-12);
    }

    #[test]
    fn weathering_accumulates_across_epochs() {
        // N firings grow the band strictly and monotonically — the whole point of
        // Movement 3 (a snapshot cannot do this). Same chapter each firing, so the
        // commit coalesces to one fact per agent while the band keeps growing.
        let inp = inputs();
        let mut acc = empty_accumulator();
        let mut last = 0.0;
        for n in 1..=10 {
            weather_bedrock_epoch(&mut acc, 0, &inp, 1.0);
            let band = acc_band(&acc);
            assert!(band > last, "band grows at firing {n}: {band} !> {last}");
            last = band;
        }
        // Same-chapter firings coalesce ⇒ still one fact per agent, band = N × rate.
        assert_eq!(acc.facts_for(0).len(), 3, "coalesced to one fact per agent");
        let single = weather_rate(&inp, BEDROCK_SEAM_MATERIAL);
        assert!(
            (last - single * 10.0).abs() < 1e-9,
            "10 firings = 10× one firing"
        );
    }

    #[test]
    fn dt_scales_the_share_linearly() {
        // `dt` is live: doubling the phase length doubles the metres moved in a firing
        // (share ∝ dt), so the rate becomes a real per-pass knob (journal/0090 M3).
        let inp = inputs();
        let mut a1 = empty_accumulator();
        let mut a2 = empty_accumulator();
        let m1 = weather_bedrock_epoch(&mut a1, 0, &inp, 1.0);
        let m2 = weather_bedrock_epoch(&mut a2, 0, &inp, 2.0);
        assert!((m2 - 2.0 * m1).abs() < 1e-12, "dt=2 moves twice dt=1");
        assert!((acc_band(&a2) - 2.0 * acc_band(&a1)).abs() < 1e-12);
    }

    #[test]
    fn bedrock_facts_key_stably_as_the_record_grows() {
        // THE SPAN-INDEX CRUX. The accumulator keys bedrock at the invariant sentinel
        // slot 0 while the (pretend) record grows across epochs. After the loop we
        // re-key onto a GROWN record: the band composes to exactly the same value it
        // had in the accumulator — no misalignment as `units.len()` shifted.
        let inp = inputs();
        let mut acc = empty_accumulator();
        for _ in 0..5 {
            weather_bedrock_epoch(&mut acc, 0, &inp, 1.0);
        }
        let band_in_accumulator = acc_band(&acc);
        assert!(band_in_accumulator > 0.0);

        // A grown record with several units (the deposition pass appended these while
        // weathering ran) — its bedrock index is units.len(), far from slot 0.
        let mut grown = DeepStrata::default();
        for _ in 0..7 {
            grown.deposit(tag(DepEnv::Subaerial, EnergyBand::Low), 0.6, 0);
        }
        let finalized = finalize_ledgers(vec![acc], std::slice::from_ref(&grown));
        // The consumer reads the bedrock band at the FINAL record's bedrock slot.
        let band_after_finalize = finalized[0].weathering_product_m(grown.units.len());
        assert!(
            (band_after_finalize - band_in_accumulator).abs() < 1e-12,
            "stable key: {band_after_finalize} != {band_in_accumulator}"
        );
        // And it landed at the right slot (units.len()), not slot 0.
        assert!(
            finalized[0].facts_for(0).is_empty(),
            "record unit 0 carries no bedrock facts"
        );
        assert_eq!(finalized[0].facts_for(grown.units.len()).len(), 3);
    }
}
