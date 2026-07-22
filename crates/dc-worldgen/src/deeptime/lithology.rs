//! **Lithology-aware erosion: the agent-specific resistance model.**
//!
//! The deep-time engine used to incise every cell with one global `k_bedrock`,
//! so granite and mudstone eroded at exactly the same rate and the world had no
//! differential erosion anywhere (ROADMAP § Observed, "our dismal mountains",
//! cause 1). This module is what erosion asks instead.
//!
//! ## The trap this module exists to avoid
//!
//! The obvious design is one number per rock — an "erodibility" — and it is
//! wrong in a way that is very expensive to discover later. **Limestone is
//! mechanically competent AND chemically soluble.** It stands in vertical
//! cliffs *because* it is strong, and it hosts cave systems *because* it
//! dissolves. Those two facts are simultaneously true and they pull a single
//! scalar in opposite directions:
//!
//! - erodibility low (hard) → limestone cliffs work, karst is impossible;
//! - erodibility high (soft) → karst works, limestone cliffs are impossible.
//!
//! There is no value of one number that gives both, so a one-number model does
//! not merely approximate karst badly — it forecloses it, and the day the
//! dissolution agent is written the coupling has to be torn out and redone.
//!
//! So resistance here is **not a property of a rock**. It is a property of a
//! *(rock, agent)* pair. [`LithoResistance`] carries one axis per erosion
//! agent, each derived from the property-sheet field that actually governs
//! **that** agent, and [`Agent`] names the agents. An agent consults its own
//! axis and no other.
//!
//! ## The agents, and what each one reads
//!
//! | agent | axis | property sheet source | status |
//! |---|---|---|---|
//! | [`Agent::Abrasion`] | [`LithoResistance::abrasion`] | smash resistance × cohesion | **live** (fluvial incision, cover entrainment, hillslope creep) |
//! | [`Agent::Dissolution`] | [`LithoResistance::dissolution`] | `solubility` (inverse) | designed, unbuilt — earth-processes § 8, karst/speleogenesis |
//! | [`Agent::FrostIce`] | [`LithoResistance::frost_ice`] | smash resistance × permeability | **live** (periglacial weathering multiplier — journal/0034) |
//! | [`Agent::Wave`] | [`LithoResistance::wave`] | smash resistance × cohesion (jointing-dominated) | **live** (littoral cliff/platform cutting — journal/0034) |
//! | [`Agent::Eolian`] | [`LithoResistance::eolian`] | cohesion (crust vs loose grain) | **live** (wind deflation/deposition — journal/0034) |
//!
//! [`Agent`] is deliberately **not** `#[non_exhaustive]` and every consumer
//! matches it exhaustively: adding the fifth agent ([`Agent::Eolian`],
//! journal/0034) was a compile error at every site that had to answer for it,
//! including [`LithoResistance::to`]. That is the structural guarantee — a new
//! agent cannot silently inherit the mechanical answer.
//!
//! ## Which rock is at the surface
//!
//! At the deep-time tier there is no material yet — the recorder tags units by
//! *measured environment* ([`DepTag`]), and members are resolved at collapse
//! time. So the chain is
//!
//! ```text
//! DepTag  →  Litho  →  reference MaterialId  →  MaterialProps  →  LithoResistance
//! ```
//!
//! [`litho_of_tag`] mirrors `crate::geology::deep_class` exactly (asserted by
//! test), so **the rock that resisted erosion is the rock a player will dig**.
//! The reference member is *fixed per class* rather than sampled from the live
//! registry, which is deliberate: it means adding an organism or material pack
//! can never move terrain (ROADMAP § S10 follow-through call 5 — the
//! pack-addition blast radius). Packs diversify what fills a class; they do not
//! renegotiate how fast that class erodes.
//!
//! ## Structural deformation, and loose materials
//!
//! [`exposed_litho`] reads the *near-surface window* of the record — the
//! lithology that dominates the topmost [`OUTCROP_DOMINANCE_WINDOW_M`], not
//! merely the last unit — because a bed too thin to fill an erosion cell must not
//! define its rock (journal/0068). Today the record *is* a flat stack, so that
//! window is the last few units. When the deformation term lands (ROADMAP
//! § Observed, "the world is a LAYER CAKE"), "which units lie in the window here"
//! stops being "the last ones" — but every other part of this module is
//! indifferent to how that question is answered. Only the one function changes.
//!
//! Likewise the split between loose and lithified material is a distinction
//! [`Litho`] can grow a variant for; nothing here assumes a lithology is rock.

use dc_core::materials::{DamageType, MaterialId};

use super::recorder::{Biofacies, DepEnv, DepTag, EnergyBand};

/// An erosion agent — a *process* that removes material, not a material.
///
/// Each variant is coupled one-to-one with an axis of [`LithoResistance`].
/// Exhaustively matched everywhere on purpose: a new agent must be given an
/// explicit answer at every site rather than defaulting into the mechanical
/// one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Agent {
    /// **Mechanical abrasion**: fluvial incision, sediment entrainment,
    /// hillslope creep — grinding rock away with rock. The only agent the
    /// engine implements today.
    Abrasion,
    /// **Chemical dissolution**: carbonate and evaporite going into solution in
    /// circulating meteoric water. The karst/speleogenesis agent
    /// (earth-processes.md § 8, docs/design/water.md). Designed, unbuilt.
    Dissolution,
    /// **Frost shattering and glacial plucking**: water admitted into pores and
    /// joints, then frozen. The cryosphere agent (geology.md sequences glacial
    /// "later"). Designed, unbuilt.
    FrostIce,
    /// **Wave attack** at a coastline: hydraulic quarrying along joints plus
    /// abrasion by entrained clasts. The littoral agent (journal/0034).
    Wave,
    /// **Wind deflation**: entrainment of loose, dry, unvegetated cover and its
    /// downwind redeposition as loess/dune. The eolian agent (journal/0034) — the
    /// fifth agent, added when the arid-landform roster went live. Keys on
    /// cohesion: a cemented crust or sticky clay resists deflation, loose sand
    /// blows.
    Eolian,
}

impl Agent {
    /// Every agent, for exhaustive sweeps in tests and probes.
    pub const ALL: [Agent; 5] = [
        Agent::Abrasion,
        Agent::Dissolution,
        Agent::FrostIce,
        Agent::Wave,
        Agent::Eolian,
    ];

    /// Short name for probe output.
    pub fn name(self) -> &'static str {
        match self {
            Agent::Abrasion => "abrasion",
            Agent::Dissolution => "dissolution",
            Agent::FrostIce => "frost/ice",
            Agent::Wave => "wave",
            Agent::Eolian => "eolian",
        }
    }
}

/// One lithology's resistance **to each agent separately**. Higher = harder to
/// remove *by that agent*. The axes are independent by construction and there
/// is deliberately no method that collapses them to a single number.
///
/// A carbonate sheet is the case that proves the shape: high [`Self::abrasion`]
/// (it makes cliffs) with low [`Self::dissolution`] (it makes caves), at the
/// same time, with no contradiction.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LithoResistance {
    /// Resistance to mechanical abrasion — **the property sheet's `smash`
    /// extraction resistance, read straight**. That is the sheet's mechanical-
    /// competence axis, the one a player already feels through a pick, and
    /// keeping it unmodified is deliberate: an earlier version of this function
    /// tempered it by `cohesion`, which *measurably destroyed the very contrast
    /// the milestone exists to create* — mudstone is more cohesive than
    /// sandstone (0.95 vs 0.85), being sticky rather than strong, so the
    /// tempering pulled the two rocks from 9.5 % apart to 4 % apart. Cohesion is
    /// the right modifier for the wave agent and the wrong one here
    /// (journal/0029).
    pub abrasion: f64,
    /// Resistance to chemical dissolution — the inverse of the sheet's
    /// `solubility`. [`f64::INFINITY`] for an insoluble material, which is
    /// every material in today's roster, and is the honest value rather than a
    /// large placeholder: nothing we have *ever* dissolves.
    pub dissolution: f64,
    /// Resistance to frost wedging / glacial plucking. Scales with mechanical
    /// competence but is *reduced* by permeability: frost shattering needs
    /// water admitted into the pore space before it can freeze there, so a
    /// porous sandstone shatters where a tight granite endures.
    pub frost_ice: f64,
    /// Resistance to wave attack. Keys on **cohesion** far more strongly than
    /// the fluvial axis does: a sea cliff fails by hydraulic quarrying along
    /// joints and bedding, which is a question of how well the rock is held
    /// together, not how hard its grains are.
    pub wave: f64,
    /// Resistance to **wind deflation**. Keys on **cohesion** alone: wind cannot
    /// grind competent bedrock, it only lifts loose grain, so what matters is
    /// cementation/stickiness — a well-cemented sandstone or a crusted clay
    /// resists, an unconsolidated fine sand is a dune source. Grain competence
    /// (`smash`) is deliberately absent: a hard-but-loose sand deflates freely.
    pub eolian: f64,
}

impl LithoResistance {
    /// This lithology's resistance to one agent. Exhaustive by design — the
    /// single place that maps agents onto axes.
    #[inline]
    pub fn to(&self, agent: Agent) -> f64 {
        match agent {
            Agent::Abrasion => self.abrasion,
            Agent::Dissolution => self.dissolution,
            Agent::FrostIce => self.frost_ice,
            Agent::Wave => self.wave,
            Agent::Eolian => self.eolian,
        }
    }

    /// The rate multiplier this lithology imposes on `agent`, relative to a
    /// `reference` resistance on the same axis: `(reference / resistance) ^
    /// contrast`, clamped to `[1/cap, cap]`.
    ///
    /// `contrast` is the knob that turns the property sheet's compressed
    /// tool-time numbers into landform-scale contrast (real erodibility spans
    /// orders of magnitude; smash resistance spans a factor of ~3). `cap`
    /// bounds the feedback so no cell can run away or stall — see
    /// [`crate::deeptime::DeepConfig::erodibility_max`].
    ///
    /// An *immune* lithology (infinite resistance — e.g. anything insoluble
    /// asked about dissolution) returns exactly `0.0`, not the clamp floor:
    /// "does not dissolve at all" is a different statement from "dissolves
    /// slowly", and the clamp exists for numerical stability, not to invent a
    /// rate.
    #[inline]
    pub fn susceptibility(&self, agent: Agent, reference: f64, contrast: f64, cap: f64) -> f64 {
        let r = self.to(agent);
        if !r.is_finite() {
            return 0.0;
        }
        if r <= 0.0 {
            return cap;
        }
        (reference / r).powf(contrast).clamp(1.0 / cap, cap)
    }
}

/// A deep-time lithology: the coarse rock identity erosion can know about at
/// 460 m cells, one per content class the recorder's tags resolve to, plus the
/// basement below the record.
///
/// This is not the material registry — it is the handful of *classes* the
/// deep-time record can distinguish. Members within a class are chosen at
/// collapse time and do not vary the erosion rate (see module docs on the
/// pack-addition blast radius).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Litho {
    /// Fine clastics — mudstone/siltstone. The weak, slope-forming bed.
    ClasticFine,
    /// Coarse clastics — sandstone/conglomerate. The classic caprock.
    ClasticCoarse,
    /// Organic soil horizon (carbonaceous mudstone / paleosol).
    OrganicSoil,
    /// Peat — the softest thing in the world.
    OrganicPeat,
    /// Coal seams.
    OrganicCoal,
    /// Charcoal — a fire bed. Structurally a *thin event bed* (capped at 0.04 m,
    /// journal/0066), so it is the first member of the thin-lamina family the
    /// thickness-dominance rule in [`exposed_litho`] exists to keep out of an
    /// erosion cell's identity. It is a `Litho` so the deep-time↔collapse mirror
    /// is total (`litho_of_tag` agrees with `geology::deep_class` over the whole
    /// tag space); it essentially never *outcrops*, because it cannot dominate a
    /// [`OUTCROP_DOMINANCE_WINDOW_M`] window.
    OrganicCharcoal,
    /// **Basement**: unrecorded igneous/metamorphic rock below the whole
    /// sedimentary pile. What a column exposes once erosion has stripped its
    /// record — and the reason coupling should hand back resistant shield and
    /// craton landscapes without anyone writing a shield rule.
    Basement,
}

impl Litho {
    /// The number of lithologies — the width of every per-lithology table.
    pub const COUNT: usize = 7;

    /// Every lithology, in table order.
    pub const ALL: [Litho; Litho::COUNT] = [
        Litho::ClasticFine,
        Litho::ClasticCoarse,
        Litho::OrganicSoil,
        Litho::OrganicPeat,
        Litho::OrganicCoal,
        Litho::OrganicCharcoal,
        Litho::Basement,
    ];

    /// Dense index for per-lithology lookup tables.
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Litho::ClasticFine => 0,
            Litho::ClasticCoarse => 1,
            Litho::OrganicSoil => 2,
            Litho::OrganicPeat => 3,
            Litho::OrganicCoal => 4,
            Litho::OrganicCharcoal => 5,
            Litho::Basement => 6,
        }
    }

    /// Short code for column printouts.
    pub fn code(self) -> &'static str {
        match self {
            Litho::ClasticFine => "fine",
            Litho::ClasticCoarse => "coarse",
            Litho::OrganicSoil => "soil",
            Litho::OrganicPeat => "peat",
            Litho::OrganicCoal => "coal",
            Litho::OrganicCharcoal => "charcoal",
            Litho::Basement => "basement",
        }
    }

    /// The **reference member** whose property sheet stands for this whole
    /// class in deep time. Fixed, not sampled from the live registry, so a
    /// content pack cannot move terrain by adding a member (module docs).
    pub fn reference_material(self) -> MaterialId {
        match self {
            Litho::ClasticFine => MaterialId::MUDSTONE,
            Litho::ClasticCoarse => MaterialId::SANDSTONE,
            Litho::OrganicSoil => MaterialId::CARBONACEOUS_MUDSTONE,
            Litho::OrganicPeat => MaterialId::PEAT,
            Litho::OrganicCoal => MaterialId::COAL,
            Litho::OrganicCharcoal => MaterialId::CHARCOAL,
            Litho::Basement => MaterialId::GRANITE,
        }
    }

    /// This lithology's agent-specific resistances.
    pub fn resistance(self) -> LithoResistance {
        resistance_of_material(self.reference_material())
    }
}

/// The lithology whose sheet defines the reference resistance — the rate every
/// other lithology is expressed relative to. Fine clastics: the most abundant
/// rock in the record, so the mean rate multiplier stays near 1 and switching
/// coupling on differentiates the world rather than uniformly slowing it.
pub const REFERENCE_LITHO: Litho = Litho::ClasticFine;

/// Derive a material's agent-specific resistances from its property sheet.
///
/// Each axis reads the field that governs **its own** agent; this is the
/// function where "resistance is a property of a (rock, agent) pair" is
/// actually cashed out. The coefficients are calibration on real property-sheet
/// values, in the same
/// plausible-not-tuned register as S9's physics constants and S10's rate
/// constants (no-bandaid: they ride as measured).
pub fn resistance_of_material(m: MaterialId) -> LithoResistance {
    let p = m.props();
    // Mechanical competence, as the world already models it: what it takes to
    // break the rock with a blunt instrument.
    let smash = f64::from(p.resistance(DamageType::Smash));
    let cohesion = f64::from(p.cohesion);
    let permeability = f64::from(p.permeability);
    let solubility = f64::from(p.solubility);
    LithoResistance {
        // Abrasion: mechanical competence, read straight (see the field docs for
        // why nothing is allowed to temper it).
        abrasion: smash,
        // Dissolution: purely chemical — competence is irrelevant, which is the
        // whole point. Insoluble rock is *immune*, not merely slow.
        dissolution: if solubility > 0.0 {
            1.0 / solubility
        } else {
            f64::INFINITY
        },
        // Frost/ice: competence, discounted by the pore space that lets water
        // in to freeze.
        frost_ice: smash * (1.0 - 0.5 * permeability),
        // Wave: jointing/cementation dominates. `max` keeps a cohesionless
        // material from reading as zero-resistance (it is still made of rock).
        wave: smash * cohesion.max(0.05),
        // Eolian: cohesion alone (no grain competence — wind cannot grind rock,
        // only lift loose grain). `max` floors a cohesionless material so it
        // reads as *most* deflatable rather than infinitely so.
        eolian: cohesion.max(0.05),
    }
}

/// The lithology a recorded unit's measured tag resolves to.
///
/// **Mirrors `crate::geology::deep_class` with no exception** — asserted by
/// `tests/erodibility.rs::litho_routing_matches_the_collapse_tier` over the whole
/// tag space. The two must not drift: if erosion thinks a bed is sandstone and
/// the collapse layer builds it out of mudstone, the world's shape stops
/// explaining the world's rock.
///
/// This is a per-*unit* fact — the rock this one bed is made of — and it is total
/// because every facies the recorder can tag has a lithology, charcoal included.
/// The thin-lamina concern that once lived here as a `Charcoal` special case does
/// **not** belong in this routing: "a 3 cm fire bed must not set the strength of a
/// 460 m erosion cell" is a statement about *thickness*, not about *charcoal*, and
/// it is enforced generally in [`exposed_litho`], where a unit competes for the
/// outcrop by how much of the near-surface window it fills. Charcoal is capped at
/// 0.04 m so it essentially never wins that competition; ash falls and marker beds
/// are the same family and get the same treatment for free (journal/0068, A-7).
pub fn litho_of_tag(tag: DepTag) -> Litho {
    match tag.biota {
        Biofacies::Coal => Litho::OrganicCoal,
        Biofacies::Peat => Litho::OrganicPeat,
        Biofacies::Charcoal => Litho::OrganicCharcoal,
        Biofacies::Soil | Biofacies::Retro => Litho::OrganicSoil,
        Biofacies::Mineral => match tag.env {
            DepEnv::Subsea => Litho::ClasticFine,
            DepEnv::Subaerial => match tag.energy {
                EnergyBand::High | EnergyBand::Medium => Litho::ClasticCoarse,
                EnergyBand::Low => Litho::ClasticFine,
            },
        },
    }
}

/// The near-surface depth of section over which a unit must *dominate* to define
/// a cell's outcropping lithology.
///
/// **This is a CALIBRATION — a knob, not a law.** It is set to one collapse voxel
/// (0.9 m), the smallest depth of section the expressed world can distinguish, on
/// the statement of shape: **a unit too thin to dominate an erosion cell must not
/// define its lithology.** That is general and unnamed — a 3 cm charcoal lamina, a
/// volcanic ash fall, a marker bed are the same family, and none of them should
/// hand a 460 m erosion cell the strength of its thinnest bed. Widen it and the
/// outcrop is set by deeper, thicker units (more inertia, less surface detail);
/// narrow it and thin surface beds start to count. There is no measured Earth
/// value here to defer to; it is calibrated to the world's own voxel resolution.
pub const OUTCROP_DOMINANCE_WINDOW_M: f64 = 0.9;

/// The lithology **outcropping** at a cell: the lithology holding the greatest
/// thickness in the topmost [`OUTCROP_DOMINANCE_WINDOW_M`] of the record, or
/// [`Litho::Basement`] when the record is empty (the column has been stripped
/// past its whole sedimentary history), thin, or absent.
///
/// **The rule, and why it is a thickness rule.** Walk units from the top of the
/// record downward, accumulating thickness per [`Litho`] until the window is
/// full (clipping the last unit to the window boundary so the window is exact).
/// Any deficit — a record shorter than the window — accrues to [`Litho::Basement`],
/// the rock below the pile. The winner is the lithology with the most accumulated
/// thickness; ties go to the one encountered **nearest the surface**. So a bed
/// too thin to fill much of the window cannot define the cell's rock even if it is
/// the topmost unit — which is exactly the property that lets charcoal (and ash,
/// and any marker bed) be its own honest `Litho` in the record without ever
/// hijacking an erosion cell's strength. It is *dominance*, not a per-unit
/// thickness floor: forty 2 cm beds of the same lithology stacked together do
/// dominate, because the rule integrates thickness rather than rejecting thin
/// units one at a time (journal/0068).
///
/// This is the one function structural deformation will change. Today the record
/// is a flat stack and "the topmost window" means "the last units"; once beds dip,
/// which units lie in the near-surface window at a cell is a function of the
/// fold/fault field and the erosion surface, and every other part of this module
/// carries over unaltered.
///
/// **Since 2026-07-22 that sentence is a socket rather than a promise**
/// (journal/0060): erosion no longer calls this function directly, it calls
/// [`Providers::outcrop_at`](super::providers::Providers::outcrop_at), for which
/// this function is the registered **identity**. The heir — the layer-cake /
/// dip-fold term — replaces the slot instead of editing this body, and
/// `docs/design/stubs.md` carries the entry.
pub fn exposed_litho(units: &[super::recorder::DepUnit]) -> Litho {
    let mut acc = [0.0f64; Litho::COUNT];
    // The distinct lithologies, in the order they are first met walking down from
    // the surface — so the tie-break ("nearest the surface wins") is a strict-`>`
    // scan over this order, with no float-equality comparison.
    let mut order = [Litho::Basement; Litho::COUNT];
    let mut seen = [false; Litho::COUNT];
    let mut n_order = 0usize;
    let mut remaining = OUTCROP_DOMINANCE_WINDOW_M;

    for u in units.iter().rev() {
        if remaining <= 0.0 {
            break;
        }
        if u.thickness_m <= 0.0 {
            continue;
        }
        let l = litho_of_tag(u.tag);
        let idx = l.index();
        if !seen[idx] {
            seen[idx] = true;
            order[n_order] = l;
            n_order += 1;
        }
        acc[idx] += u.thickness_m.min(remaining);
        remaining -= u.thickness_m.min(remaining);
    }

    // A record shorter than the window: the deficit is basement, below the pile —
    // and, being deepest, it is met last and loses every tie.
    if remaining > 0.0 {
        let idx = Litho::Basement.index();
        if !seen[idx] {
            order[n_order] = Litho::Basement;
            n_order += 1;
        }
        acc[idx] += remaining;
    }

    // At least one entry always exists (the deficit fills an empty record with
    // basement), so `order[0]` is present; the scan keeps the earliest (nearest
    // surface) of any tied maxima by comparing with a strict `>`.
    let mut best = order[0];
    let mut best_acc = acc[best.index()];
    for &l in &order[1..n_order] {
        let a = acc[l.index()];
        if a > best_acc {
            best = l;
            best_acc = a;
        }
    }
    best
}

/// Per-lithology rate multipliers for one agent, as a dense table indexed by
/// [`Litho::index`]. Built once per epoch and read per cell, so the `powf` is
/// paid six times per epoch rather than once per cell.
pub fn susceptibility_table(agent: Agent, contrast: f64, cap: f64) -> [f64; Litho::COUNT] {
    let reference = REFERENCE_LITHO.resistance().to(agent);
    let mut out = [1.0; Litho::COUNT];
    for l in Litho::ALL {
        out[l.index()] = l
            .resistance()
            .susceptibility(agent, reference, contrast, cap);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_lithology_is_exactly_neutral() {
        // Any contrast, any cap: the reference must not itself bias the world,
        // or "coupling on" would silently mean "erosion faster/slower".
        for contrast in [0.5, 1.0, 2.5, 4.0] {
            let t = susceptibility_table(Agent::Abrasion, contrast, 8.0);
            assert_eq!(t[REFERENCE_LITHO.index()], 1.0);
        }
    }

    #[test]
    fn basement_is_the_most_abrasion_resistant_thing_in_the_world() {
        // The falsifiable prediction the milestone rests on: strip a column past
        // its record and what it exposes is hard, so shields and cratons come
        // out resistant without a shield rule.
        let t = susceptibility_table(Agent::Abrasion, 2.5, 8.0);
        let basement = t[Litho::Basement.index()];
        for l in Litho::ALL {
            if l != Litho::Basement {
                assert!(
                    t[l.index()] > basement,
                    "{} should erode faster than basement",
                    l.code()
                );
            }
        }
        assert!(basement < 1.0);
    }

    #[test]
    fn soft_rock_erodes_faster_than_hard_rock() {
        let t = susceptibility_table(Agent::Abrasion, 2.5, 8.0);
        assert!(t[Litho::OrganicPeat.index()] > t[Litho::OrganicCoal.index()]);
        assert!(t[Litho::OrganicCoal.index()] > t[Litho::ClasticFine.index()]);
        assert!(t[Litho::ClasticFine.index()] > t[Litho::ClasticCoarse.index()]);
        assert!(t[Litho::ClasticCoarse.index()] > t[Litho::Basement.index()]);
    }

    #[test]
    fn the_cap_bounds_every_multiplier_both_ways() {
        // The stability guard: however extreme the contrast knob, no cell can
        // erode more than `cap`× or less than `1/cap`× the reference rate.
        for contrast in [1.0, 4.0, 12.0] {
            for cap in [2.0, 5.0, 8.0] {
                for l in Litho::ALL {
                    let s = susceptibility_table(Agent::Abrasion, contrast, cap)[l.index()];
                    assert!((1.0 / cap..=cap).contains(&s), "{} {s}", l.code());
                }
            }
        }
    }

    #[test]
    fn nothing_in_the_roster_is_susceptible_to_dissolution() {
        // Not a placeholder: the roster contains no carbonate, so every rock is
        // genuinely immune and the dissolution axis is genuinely dormant.
        for l in Litho::ALL {
            assert_eq!(l.resistance().dissolution, f64::INFINITY, "{}", l.code());
            assert_eq!(
                l.resistance()
                    .susceptibility(Agent::Dissolution, 1.0, 1.0, 8.0),
                0.0
            );
        }
    }

    /// **The trap, tested.** A carbonate property sheet — mechanically strong
    /// (it stands in cliffs) and highly soluble (it hosts caves) — must be able
    /// to say both things at once. Under a single "erodibility" scalar this test
    /// is unsatisfiable, which is precisely why the scalar was rejected.
    #[test]
    fn a_limestone_can_be_cliff_forming_and_cave_forming_at_once() {
        // A plausible limestone sheet, written the way the carbonate milestone
        // will write it. Not registered — this asserts the *model's* shape, so
        // it holds before the material exists.
        let limestone = LithoResistance {
            abrasion: 4.5 * (0.5 + 0.5 * 0.9),
            dissolution: 1.0 / 0.85,
            frost_ice: 4.5 * (1.0 - 0.5 * 0.2),
            wave: 4.5 * 0.9,
            eolian: 0.9,
        };
        let mudstone = Litho::ClasticFine.resistance();

        // Mechanically it out-resists the slope-forming mudstone: cliffs.
        assert!(
            limestone.abrasion > mudstone.abrasion,
            "limestone must be able to hold a cliff"
        );
        // Chemically it is the most attackable rock in the world: caves.
        assert!(
            limestone.dissolution < mudstone.dissolution,
            "limestone must be able to host karst"
        );
        // And the two statements do not interfere: the mechanical agent's
        // answer is unchanged by the chemical one and vice versa.
        let hypothetical_insoluble_limestone = LithoResistance {
            dissolution: f64::INFINITY,
            ..limestone
        };
        assert_eq!(
            hypothetical_insoluble_limestone.to(Agent::Abrasion),
            limestone.to(Agent::Abrasion),
            "changing solubility must not change mechanical resistance"
        );
    }

    #[test]
    fn frost_and_wave_rank_rocks_differently_from_abrasion() {
        // If every axis produced the same ordering, the multi-axis model would
        // be decoration. Permeable sandstone must lose ground to frost that it
        // does not lose to abrasion.
        let sandstone = Litho::ClasticCoarse.resistance();
        let fine = Litho::ClasticFine.resistance();
        assert!(sandstone.abrasion > fine.abrasion);
        assert!(
            sandstone.frost_ice / fine.frost_ice < sandstone.abrasion / fine.abrasion,
            "porous sandstone should fare relatively worse against frost"
        );
    }

    #[test]
    fn every_agent_has_an_axis_and_they_are_distinct_reads() {
        for a in Agent::ALL {
            let r = Litho::ClasticCoarse.resistance();
            assert!(r.to(a) > 0.0, "{}", a.name());
        }
        assert_eq!(Agent::ALL.len(), 5);
    }
}
