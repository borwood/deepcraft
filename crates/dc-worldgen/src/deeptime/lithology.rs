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
//! **Since P11 slice 1 (2026-08-01) the deep tier resolves the material at
//! DEPOSITION**, so a recorded unit already knows which rock it is
//! (`DepUnit::species: MaterialId`) and the chain is
//!
//! ```text
//! DepTag  →  Litho (class)  →  fitness at deposition  →  MaterialId  →  MaterialProps  →  LithoResistance
//! ```
//!
//! [`litho_of_tag`] mirrors `crate::geology::deep_class` exactly (asserted by
//! test), so **the rock that resisted erosion is the rock a player will dig**.
//!
//! ## ⚠ The old guarantee here is GONE, deliberately, and this paragraph is its
//! replacement
//!
//! This module used to say the reference member is *fixed per class rather than
//! sampled from the live registry*, and that this meant **adding an organism or
//! material pack can never move terrain**. That was true and it is no longer:
//! deposition-time fitness reads the live registry, the deposited identity feeds
//! [`exposed_shares`] → [`susceptibility_table`] → the erosion rates, and a pack
//! that registers a new clastic member therefore *can* change how fast a hillside
//! wears down.
//!
//! **That is not a regression; it is the ruling.** The reason the old guard
//! existed — *"a pack must not retroactively change an existing world"* — was
//! answered from a different direction: **a different pack set IS a different
//! world** (the user's 2026-07-19 world-identity rule, `docs/design/ideas.md`),
//! and P11's record-bake kills the one real hazard by *storing* the identity
//! rather than re-deriving it at expression. A world generated with one pack set
//! keeps its rocks forever. A world generated with another was never the same
//! world. The question the guard was protecting against — U5,
//! *"may pack members move terrain?"* — is **DISSOLVED, not overruled**
//! (`journal/corrections.md` #84); this comment's own reason is what expired, and
//! rewriting it is the owed work that dissolution named.
//!
//! What packs still cannot do is inflate a class: abundance is normalised within
//! the class, so registering a member *diversifies* its share and never enlarges
//! it (`dc_core::materials::geology`, tested).
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

use dc_core::coarse::ShareVec;
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

    /// **The class a recorded material belongs to** — the interim parent view
    /// (P11 slice 1).
    ///
    /// ⚠ **THIS IS SCAFFOLDING WITH A DATED DEMOLITION ORDER.** Ruling 2 of the
    /// P11 design (2026-08-01) is *representation A-CLEAN: **no class view
    /// survives in storage or physics***. This function is a class view in
    /// physics. It exists for exactly one reason: the transport arithmetic is
    /// still `Litho::COUNT`-wide (four `n × SPECIES` planes, `susceptibility_table`,
    /// `settling_table`, [`WindowShares`]), so a `MaterialId`-grade record has to
    /// be bucketed back down before the erosion tables can index it.
    ///
    /// **Heirs, both already sequenced:** slice 2 (the budget planes go
    /// CSR-sparse over `MaterialId`, so the bucket has no consumer on the
    /// transport side) and slice 4 (the `Litho` roster dissolves). Do not build
    /// anything new on this.
    ///
    /// It is a **summary, and the doctrine's price is paid**: it is the exact
    /// inverse of the member→class edge the `GeologySet` already declares, and
    /// `lithology_buckets_agree_with_the_registry` asserts it agrees with that
    /// authority for every vanilla member. A material no geology member deposits
    /// answers [`Litho::Basement`] — the "not a depositional lithology" bucket the
    /// window walk already charges its deficit to.
    pub fn of_material(m: MaterialId) -> Litho {
        match m {
            MaterialId::MUDSTONE | MaterialId::SILTSTONE => Litho::ClasticFine,
            MaterialId::SANDSTONE | MaterialId::CONGLOMERATE => Litho::ClasticCoarse,
            MaterialId::CARBONACEOUS_MUDSTONE => Litho::OrganicSoil,
            MaterialId::PEAT => Litho::OrganicPeat,
            MaterialId::COAL => Litho::OrganicCoal,
            MaterialId::CHARCOAL => Litho::OrganicCharcoal,
            _ => Litho::Basement,
        }
    }

    /// The **reference member** whose property sheet stands for this whole
    /// class in deep time.
    ///
    /// ⚠ **P11 retired this from production identity.** A deposited unit's
    /// material is now chosen by fitness at deposition
    /// ([`super::recorder::DepositCtx`]); this survives as the *degenerate*
    /// answer — the door tests use when they have no content set
    /// ([`super::recorder::DeepStrata::deposit`]), and the S-5 identity default
    /// when a class has no registered member at all. Its heir is slice 4's
    /// dissolution of the roster.
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
///
/// **And since journal/0072 the verdict is no longer what erosion reads for its
/// *rates*.** Erosion consumes [`exposed_shares`] (through the
/// [`outcrop_shares`](super::providers::Providers::outcrop_shares) seam) and
/// blends the susceptibility table by window share, so the verdict here and the
/// shares are two faces of one [`window_walk`]. The two seams share a single heir
/// (structural deformation) and retire as a **pinned pair**: once beds dip, the
/// heir supplies the dipped shares and the verdict is again their argmax.
pub fn exposed_litho(units: &[super::recorder::DepUnit]) -> Litho {
    window_walk(units).1
}

/// The **window walk**, done once, yielding both faces of the near-surface
/// section: the per-[`Litho`] accumulated thickness (which always totals exactly
/// [`OUTCROP_DOMINANCE_WINDOW_M`] — the deficit below a short record is charged to
/// [`Litho::Basement`]) **and** the exact outcrop verdict (the dominant lithology,
/// nearest-surface tie-break).
///
/// This is the single source both [`exposed_litho`] (the verdict) and
/// [`exposed_shares`] (the quantity) draw from, so the two can never disagree
/// about what the window holds: the verdict is, by construction, the argmax of the
/// shares. Keeping it one walk is also why the `Litho`-verdict path stays
/// **byte-identical** to the pre-blend rule — the tie-break here is the same
/// strict-`>` scan over first-met (nearest-surface) order it always was.
fn window_walk(units: &[super::recorder::DepUnit]) -> ([f64; Litho::COUNT], Litho) {
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
        // **The unit's own species, not its tag's** (Movement 2b): a window full
        // of sand that a river delivered to a low-energy cell reads as sand
        // rather than as whatever its environment would have implied.
        //
        // ⚠ **The bucket, not the rock** (P11 slice 1). The record names a
        // `MaterialId` now, and the honest per-unit answer is
        // `resistance_of_material(u.species)` — but the accumulator and everything
        // downstream of it ([`WindowShares`], `susceptibility_table`, the four
        // `n × SPECIES` transport planes) are still `Litho::COUNT`-wide, so the
        // rock is coarsened back to its class here. **Slice 2 is what deletes this
        // call** — when the budgets go CSR-sparse over `MaterialId`, the window
        // accumulates per material and mudstone stops eroding at siltstone's rate.
        let l = Litho::of_material(u.species);
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
    (acc, best)
}

/// The **per-[`Litho`] shares** of the near-surface window: the fraction of the
/// topmost [`OUTCROP_DOMINANCE_WINDOW_M`] each lithology fills, summing to `1.0`
/// (the deficit below a short record is [`Litho::Basement`]'s share). This is the
/// *quantity* the outcrop verdict is the argmax of — the interpolable cause S-4
/// asks erosion to threshold late on, rather than collapsing to a single label and
/// stepping the rate at the plurality crossover (journal/0072).
///
/// It is the same walk as [`exposed_litho`], normalised: a window that is 100 %
/// one lithology returns `1.0` for that lithology and `0.0` elsewhere, so
/// [`blend_susceptibility`] over it reproduces the argmax lookup **bit for bit** —
/// argmax is the degenerate case of the blend.
///
/// Like [`exposed_litho`], this is the one function structural deformation will
/// change: once beds dip, which units lie in the near-surface window (and how much
/// of each) is a function of the fold/fault field, and *this* quantity — not just
/// the verdict — is what the heir must supply, so the blended rate field dips with
/// the beds too. It is the identity of the
/// [`Providers::outcrop_shares`](super::providers::Providers::outcrop_shares) slot.
pub fn exposed_shares(units: &[super::recorder::DepUnit]) -> WindowShares {
    // Divide (not multiply by a reciprocal): the accumulator of a uniform window is
    // bit-identical to `OUTCROP_DOMINANCE_WINDOW_M`, and `x / x == 1.0` exactly in
    // IEEE-754 — so a single-lithology window gets share exactly `1.0` and
    // [`blend_susceptibility`] reproduces the argmax lookup bit for bit. `w * (1/w)`
    // does not carry that guarantee.
    let (acc, _) = window_walk(units);
    ShareVec::from_shares(acc.map(|a| a / OUTCROP_DOMINANCE_WINDOW_M))
}

/// **The near-surface window's shares at MEMBER grade** — the same walk as
/// [`exposed_shares`], accumulated per [`SpeciesAxis`] slot instead of per class
/// (P11 slice 2).
///
/// `out` is a dense row `axis.len()` wide, overwritten in full. The shares sum to
/// `1.0` by the same construction as the class-grade walk (divide, never multiply
/// by a reciprocal — `x / x == 1.0` exactly — so a single-material window is
/// exactly `1.0` and a blend over it reproduces a point lookup bit for bit), and
/// the deficit below a short record is charged to the axis's **basement** slot.
///
/// This is what makes the erosion rate a function of the *rock* rather than of its
/// class: today `window_walk` coarsens a recorded `MaterialId` back through
/// [`Litho::of_material`] because the tables downstream are class-keyed, and
/// mudstone and siltstone therefore erode at one rate. The class accumulator is
/// still computed beside this one — the far tier and the outcrop verdict read it
/// until slice 4 dissolves the roster — but the *quantity erosion blends* comes
/// from here.
///
/// **A material with no axis slot of its own** (nothing any registered geology
/// member deposits — an igneous body the veneer placed) is charged to basement,
/// through [`SpeciesAxis::slot_of`]'s own total answer. That is the same rule the
/// window already applies to its deficit, not a second one.
pub fn exposed_member_shares(
    axis: &super::species::SpeciesAxis,
    units: &[super::recorder::DepUnit],
    out: &mut [f64],
) {
    debug_assert_eq!(out.len(), axis.len());
    out.fill(0.0);
    let mut remaining = OUTCROP_DOMINANCE_WINDOW_M;
    for u in units.iter().rev() {
        if remaining <= 0.0 {
            break;
        }
        if u.thickness_m <= 0.0 {
            continue;
        }
        let take = u.thickness_m.min(remaining);
        out[axis.slot_of(u.species)] += take;
        remaining -= take;
    }
    if remaining > 0.0 {
        out[axis.basement_slot()] += remaining;
    }
    for v in out.iter_mut() {
        *v /= OUTCROP_DOMINANCE_WINDOW_M;
    }
}

/// **Per-material rate multipliers for one agent**, as a dense row indexed by
/// [`SpeciesAxis`] order — the member-grade heir of [`susceptibility_table`]
/// (P11 slice 2).
///
/// One row per (agent, epoch), never one per cell, so dense is honest here: the
/// `powf` is paid `axis.len()` times an epoch. The reference is the **named
/// anchor material** rather than a class: `REFERENCE_LITHO.reference_material()`
/// is `dc:mudstone`, so the value is bit-identical to the class-grade table's for
/// every material whose class reference it is, and *different* — correctly — for
/// every other member. That difference is the point of the slice.
pub fn member_susceptibility_table(
    axis: &super::species::SpeciesAxis,
    agent: Agent,
    contrast: f64,
    cap: f64,
) -> Vec<f64> {
    let reference = resistance_of_material(REFERENCE_MATERIAL).to(agent);
    axis.materials()
        .iter()
        .map(|&m| resistance_of_material(m).susceptibility(agent, reference, contrast, cap))
        .collect()
}

/// **The material whose property sheet defines the reference resistance** — the
/// rate every other material is expressed relative to.
///
/// It is `REFERENCE_LITHO`'s reference member spelled out as a rock, which is the
/// re-statement the P11 design audit's § 6b asked for: under member grade there is
/// no *"fine clastic"* to anchor on, so the anchor has to be **named**. The value
/// is unchanged — `Litho::ClasticFine.reference_material()` is `dc:mudstone` —
/// so this is a derivation restated, not a re-tune.
pub const REFERENCE_MATERIAL: MaterialId = MaterialId::MUDSTONE;

/// **The rock below the whole sedimentary pile**, named as a material.
///
/// [`Litho::Basement`]'s reference member spelled out, for the same reason
/// [`REFERENCE_MATERIAL`] is: under member grade there is no *"basement class"* to
/// fall back on, so the axis needs the rock by name. Asserted equal to the class
/// answer by `the_named_basement_is_the_class_reference` — the summary agrees with
/// the authority for as long as both exist (slice 4 removes the authority).
pub const DEEP_BASEMENT: MaterialId = MaterialId::GRANITE;

/// **What a material IS once a mover has carried it and set it down** — the
/// member-grade face of [`Litho::as_deposited`] (P11 slice 2).
///
/// Returns `None` when the rock is unchanged by the act of being deposited (the
/// common case: sandstone a river carried is sandstone), and `Some(class)` when
/// deposition is a genuine **transformation edge** — basement a river quarried
/// lands as coarse clastic detritus, and detrital peat/coal/charcoal lands as
/// carbonaceous mud. See [`Litho::as_deposited`] for why those two families are the
/// same rule.
///
/// The *destination* is a class, not a rock, and that is deliberate: which member
/// of the destination class a transformation produces is a question about the
/// conditions at the site, not about the parent — so the caller re-runs fitness
/// there. This is the one place P11 ruling 6's *"genuine-degeneracy remainder…
/// transformation edges under declared conditions"* survives on the transported
/// path.
///
/// ⚠ Rides [`Litho::of_material`], the interim class bucket, and retires with it
/// (slice 4).
#[inline]
pub fn deposited_transform(m: MaterialId) -> Option<Litho> {
    let from = Litho::of_material(m);
    let to = from.as_deposited();
    if to == from { None } else { Some(to) }
}

/// **Blend a per-material susceptibility table by a sparse window row** — the
/// member-grade heir of [`blend_susceptibility`] (P11 slice 2).
///
/// `axis_of` are the row's axis codes and `shares` its values; `tab` is a
/// [`member_susceptibility_table`] indexed by axis order. Anchored at the row's
/// dominant material exactly as the class-grade blend is anchored at its argmax,
/// and for the identical reason: `tab[d] + Σ shares[i]·(tab[i] − tab[d])` is
/// **exact at the two boundaries the invariants pin** — a uniform window returns
/// `tab[d]` bit for bit (so the blend is a strict generalisation of a point
/// lookup), and a uniform table returns that value bit for bit (so a neutralised
/// coupling is a perfect no-op). The bare dot is sub-ULP off at both, because
/// share normalisation does not sum to exactly `1.0`.
///
/// Ties in the argmax break by **axis order** — descending settling energy, then
/// `MaterialId` — which is the same total, content-derived order every other
/// order-sensitive rule in this slice uses.
#[inline]
pub fn blend_member_susceptibility(axis_of: &[u8], shares: &[f64], tab: &[f64]) -> f64 {
    let mut d = 0usize;
    let mut best = f64::NEG_INFINITY;
    for (i, &s) in shares.iter().enumerate() {
        if s > best {
            best = s;
            d = i;
        }
    }
    if shares.is_empty() {
        return 1.0;
    }
    let anchor = tab[axis_of[d] as usize];
    let mut acc = anchor;
    for (i, &s) in shares.iter().enumerate() {
        acc += s * (tab[axis_of[i] as usize] - anchor);
    }
    acc
}

/// The near-surface window's per-[`Litho`] share vector, as the headless
/// [`ShareVec`](dc_core::coarse::ShareVec) the `CoarseField` extraction owns.
///
/// This is the **first concrete `Interpolable` `T`** the boundary type was
/// extracted from (journal/0075, audit site A1): a small fixed-width, `Copy`
/// share vector whose `blend` must be exact at the identities (a uniform window →
/// that rock's rate bit-for-bit). Erosion reads it through the `outcrop_shares`
/// seam and reduces it with [`blend_susceptibility`]; the verdict
/// [`exposed_litho`] is its argmax. `[f64; Litho::COUNT]` cannot implement a
/// dc-core trait (orphan rule), so the array becomes this newtype — which is
/// exactly how the quantity moves behind the type.
pub type WindowShares = ShareVec<{ Litho::COUNT }>;

/// The lithology holding the greatest share — the outcrop verdict recovered from a
/// share vector. Ties break by [`Litho`] table order (index), which differs from
/// [`exposed_litho`]'s nearest-surface tie-break *only* on exact-thickness ties (a
/// measure-zero event in a real record); use it where a cheap dominant label over
/// an already-computed share vector is wanted (e.g. the debug `exposed()` probe),
/// not where the byte-exact verdict is required — for that, call [`exposed_litho`]
/// or the [`outcrop_at`](super::providers::Providers::outcrop_at) seam.
pub fn dominant_litho(shares: &WindowShares) -> Litho {
    // `ShareVec::argmax` is the same strict first-maximum over index order this
    // function used to inline, so the verdict is bit-identical; the deficit-filled
    // window always has positive total, so `argmax` is never `None` in production
    // (an all-zero vector maps to the first lithology, matching the old start).
    Litho::ALL[shares.argmax().unwrap_or(0)]
}

/// **Blend a per-[`Litho`] susceptibility table by a window's shares** — the
/// share-weighted rate S-4 move A prescribes (journal/0072). A cell whose window
/// is 55 % basement / 45 % fine gets `0.55·basement + 0.45·fine`, so the per-agent
/// rate field follows the thickness contours continuously instead of stepping at
/// the plurality crossover. It is a dot product; a uniform window (one share
/// `1.0`, the rest `0.0`) returns exactly that lithology's table entry, so it is a
/// strict generalisation of the old `sus_tab[litho.index()]` argmax lookup.
///
/// Pure function of the shares and the table — no entropy (this is move A, not the
/// dithered move B). The share vector is [`Interpolable`]-shaped: this is the blend
/// operation the future `CoarseField<T>`'s `sample` would carry for a small
/// blendable struct (audit part 2, shape-teacher #1's witness).
///
/// **Anchored at the dominant lithology's rate** — `tab[d] + Σ shares[i]·(tab[i] −
/// tab[d])`, `d = argmax(shares)` — rather than the bare dot `Σ shares[i]·tab[i]`.
/// Algebraically identical when the shares sum to `1.0`, but the anchoring makes it
/// **exact at the two boundaries the invariants pin**, where the bare dot is
/// sub-ULP off because share normalisation (`acc / w`) does not sum to exactly
/// `1.0`:
/// - a **uniform window** (one share `1.0`, `d` = that lithology) → `tab[d]` bit
///   for bit, so the blend is a strict generalisation of the argmax lookup;
/// - a **uniform table** (contrast `0` → every entry equal) → that value bit for
///   bit, so a *neutralised* erodibility coupling is a perfect no-op — every
///   `tab[i] − tab[d]` is `0`, so no share residue can perturb it
///   (`erodibility.rs::a_neutral_coupling_is_byte_identical`).
///
/// Continuity across the old plurality flip is preserved: at the crossover `d`
/// switches, but the two anchor formulas agree there (the value is the same convex
/// combination), so the rate field has no step — see
/// `tests/outcrop_blend.rs::the_rate_is_continuous_across_the_old_flip`.
#[inline]
pub fn blend_susceptibility(shares: &WindowShares, sus_tab: &[f64; Litho::COUNT]) -> f64 {
    // Delegate to the anchored blend the `CoarseField` extraction owns
    // ([`ShareVec::blend_table`] → `f64::blend`): the shares are the weights, the
    // table the samples, argmax-anchored so a uniform window is `tab[d]` bit for
    // bit. This is the *witness* the `Interpolable` trait was extracted from
    // (journal/0075) — the operation is byte-identical to the dot this function
    // used to inline (same operands, same index-order fold).
    shares.blend_table(sus_tab)
}

/// **The settling-velocity ordering key, per lithology** — the number the
/// material-aware transport pass sorts its suspended load by (Movement 2b,
/// `material-behavior.md` § 13.5).
///
/// It is `dc_core`'s [`settle_energy`](dc_core::materials::geology::settle_energy),
/// `sqrt(grain_size_mm × specific_gravity)`, read off the class's reference
/// property sheet — **the machinery already in the tree**, not a second settling
/// model: the same function the shipped S8 placer pass thresholds on
/// (`crate::geology::member_settle_threshold`). It rises with grain size *and*
/// density, which is the whole content of "sorting": as the flow's competence
/// ceiling falls downstream the high-`w_s` species rain out first and the fines
/// ride on.
///
/// **An honest limit of the proxy, recorded rather than patched.** `settle_energy`
/// is *size-dominated*: it multiplies grain size by specific gravity instead of
/// modelling buoyancy, so a low-density, coarse-"grained" species (peat, at 5 mm
/// and 400 kg/m³) reads as settling faster than sand, where real peat floats. The
/// heir is a better property-derived settling law (a Stokes/drag form carrying the
/// fluid's own density — which arrives with `flow.md` § 2.5's fluid identity), not
/// a material-named exception here; and it would move the shipped placer pass too,
/// so it is a change to make deliberately and together. See `docs/design/stubs.md`
/// § 23.
///
/// Built once per run and read per cell, like [`susceptibility_table`].
pub fn settling_table() -> [f64; Litho::COUNT] {
    let mut out = [0.0; Litho::COUNT];
    for l in Litho::ALL {
        out[l.index()] = dc_core::materials::geology::settle_energy(l.reference_material().props());
    }
    out
}

impl Litho {
    /// **What this lithology IS once a flow has carried it and set it down.**
    ///
    /// Every recorded unit is a *deposit* — loose material that arrived — so the
    /// one lithology that cannot be one is [`Litho::Basement`], which names the
    /// unrecorded igneous/metamorphic rock *below* the pile. Basement a river
    /// quarried and carried is coarse clastic detritus when it lands: a gravel, not
    /// a granite. Recording it as basement would hand a loose bar the strength of
    /// bedrock at the next epoch's [`exposed_shares`] read, which is exactly
    /// backwards.
    ///
    /// This is where the deep tier's **provenance** is lost — the record keeps
    /// "coarse clastic", not "coarse clastic off basement". Carrying the parent
    /// through deposition is `material-behavior.md` § 13.8's **lineage history**
    /// (the `Move`-fact chain of custody), deferred with that layer.
    ///
    /// ## The in-place organics belong here for the identical reason
    /// *(added 2026-07-26, journal/0112 — and surfaced by a magnitude, not by a
    /// review; see corrections #57.)*
    ///
    /// [`Litho::OrganicPeat`], [`Litho::OrganicCoal`] and [`Litho::OrganicCharcoal`]
    /// are not sediments that arrive. **Peat is made where it lies** — a bog, not a
    /// delivery — coal is buried peat, and charcoal is a *fire event*, which this
    /// enum's own doc calls a "thin event bed (capped at 0.04 m)". A mover that
    /// picks any of them up is carrying **detrital organic matter**, and what it
    /// sets down is carbonaceous mud with plant fragments in it: a
    /// [`Litho::OrganicSoil`], which is exactly the "carbonaceous mudstone" slot.
    /// It is not a peat bog, and it is emphatically not a three-metre seam of
    /// charcoal.
    ///
    /// **Why this was latent until hillslope creep landed.** Movement 2b gave the
    /// *fluvial* pass an identity, and the fluvial pass moves 0.109 % of this
    /// world's sediment (corrections #55) — far too little for a transported
    /// organic ever to win a cell's mixture argmax. Creep moves 918× more, and the
    /// day it carried identity, `charcoal_reaches_the_voxel_as_an_inclusion_never
    /// _as_a_stratum` found a voxel that was **8/8 charcoal**: thin fire beds
    /// crept downslope, won the argmax at a low-deposition cell, and then *merged
    /// across epochs* under one mineral tag into a stratum the cap exists to
    /// forbid. The rule was always incomplete; only the magnitude was new.
    #[inline]
    pub fn as_deposited(self) -> Litho {
        match self {
            Litho::Basement => Litho::ClasticCoarse,
            Litho::OrganicPeat | Litho::OrganicCoal | Litho::OrganicCharcoal => Litho::OrganicSoil,
            other => other,
        }
    }
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

    /// **The named rocks agree with the class authority they restate**
    /// (CLAUDE.md § *A summary is not an authority*). [`REFERENCE_MATERIAL`] and
    /// [`DEEP_BASEMENT`] are member-grade spellings of two class references; while
    /// both spellings exist they must not drift. Slice 4 deletes the authority, at
    /// which point these become the only spelling and the test retires with it.
    #[test]
    fn the_named_rocks_are_the_class_references() {
        assert_eq!(REFERENCE_MATERIAL, REFERENCE_LITHO.reference_material());
        assert_eq!(DEEP_BASEMENT, Litho::Basement.reference_material());
    }

    /// **The member-grade blend is a strict generalisation of a point lookup**, at
    /// the two boundaries the class-grade blend's own invariants pin: a uniform
    /// window returns that material's entry bit for bit, and a uniform table
    /// returns its value bit for bit (so a neutralised coupling is a perfect
    /// no-op). Both are exact-equality assertions, because the anchoring is what
    /// makes them exact — the bare dot is sub-ULP off at each.
    #[test]
    fn a_uniform_member_window_is_that_rocks_rate_bit_for_bit() {
        let tab = vec![0.7, 1.3, 2.9, 0.4];
        for k in 0..4usize {
            let mut row = vec![0.0; 4];
            row[k] = 1.0;
            let codes: Vec<u8> = (0..4u8).collect();
            assert_eq!(
                blend_member_susceptibility(&codes, &row, &tab).to_bits(),
                tab[k].to_bits(),
                "a window that is 100 % material {k} did not blend to its own rate"
            );
        }
        let flat = vec![1.75; 4];
        let row = vec![0.21, 0.4, 0.09, 0.3];
        let codes: Vec<u8> = (0..4u8).collect();
        assert_eq!(
            blend_member_susceptibility(&codes, &row, &flat).to_bits(),
            1.75f64.to_bits(),
            "a uniform table must be a perfect no-op whatever the shares"
        );
    }

    /// **A transformation edge is named, and only where deposition really changes
    /// the rock.** Basement a river quarried lands as clastic detritus; detrital
    /// organics land as carbonaceous mud; everything else is itself.
    #[test]
    fn only_the_transformation_edges_transform() {
        assert_eq!(
            deposited_transform(MaterialId::GRANITE),
            Some(Litho::ClasticCoarse)
        );
        assert_eq!(deposited_transform(MaterialId::PEAT), Some(Litho::OrganicSoil));
        assert_eq!(deposited_transform(MaterialId::COAL), Some(Litho::OrganicSoil));
        assert_eq!(
            deposited_transform(MaterialId::CHARCOAL),
            Some(Litho::OrganicSoil)
        );
        for m in [
            MaterialId::MUDSTONE,
            MaterialId::SILTSTONE,
            MaterialId::SANDSTONE,
            MaterialId::CONGLOMERATE,
            MaterialId::CARBONACEOUS_MUDSTONE,
        ] {
            assert_eq!(deposited_transform(m), None, "{} transformed", m.qualified_name());
        }
    }

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

    /// **The summary agrees with the authority** (CLAUDE.md § *A summary is not an
    /// authority*). [`Litho::of_material`] is a class view beside the
    /// `GeologySet`, which already declares each member's class; this asserts the
    /// two say the same thing for every registered member the deep tier can
    /// deposit.
    ///
    /// It is the test that makes the interim bucket legitimate rather than an S-3
    /// parallel rule — and it is what fails loudly if a pack adds a fine clastic
    /// the bucket has never heard of.
    #[test]
    fn lithology_buckets_agree_with_the_registry() {
        let set = dc_core::materials::geology::vanilla();
        for m in set.members() {
            let bucket = Litho::of_material(m.material);
            if bucket == Litho::Basement {
                // Not a depositional lithology at the deep tier (the igneous
                // members, the placer and accessory grains): the deep record
                // never names them, so there is nothing to agree about.
                continue;
            }
            assert_eq!(
                crate::geology::deep_class_of_species(bucket),
                m.class.as_str(),
                "bucket for {} disagrees with its registered class",
                m.id
            );
        }
    }

    /// **Why every terrain golden moved, measured at the site where it enters.**
    ///
    /// P11 put the `MaterialId` in `deposit_as`'s merge key, so a bed that used to
    /// be one `ClasticFine` unit is two when its members differ. [`window_walk`]
    /// buckets both back to the same class and gets the same *quantity* — but
    /// `4.1 + 3.2` is not bit-identical to `7.3`, IEEE addition is not associative,
    /// and the susceptibility blend reads these shares. One ulp, through the
    /// incision rate, compounded over 200 epochs, is a different continent.
    ///
    /// **This is stated as a bound, not as an identity, because bit-identity is
    /// not achievable and a version of this walk that coalesced runs back to the
    /// pre-P11 segmentation was BUILT, MEASURED AND REMOVED.** It did not restore
    /// the world: the per-unit thicknesses themselves differ, because the
    /// recorder accumulates `top.thickness_m += d` per unit and `erode` subtracts
    /// per unit, so a finer segmentation changes the addends and not merely their
    /// grouping. Keeping the coalescing would have been keeping a mechanism whose
    /// premise the measurement had already falsified.
    ///
    /// The bound is what is true: **the shares agree to well inside 1e-12
    /// relative**, so the erosion *rule* is unchanged and only its rounding is.
    /// Slice 2 dissolves the question — the shares go per material, and a
    /// member-grade subdivision starts to mean something.
    #[test]
    fn splitting_a_unit_within_its_class_preserves_the_outcrop_shares() {
        use crate::deeptime::recorder::{DepEnv, DepTag, DepUnit, EnergyBand};
        let tag = DepTag::mineral(
            DepEnv::Subsea,
            crate::deeptime::Aridity::Humid,
            EnergyBand::Low,
        );
        let unit = |m, t| DepUnit {
            tag,
            thickness_m: t,
            unconformity: false,
            chapter: 0,
            species: m,
        };
        // One thick fine-clastic bed, against the same metres split between two
        // members of that same class — exactly what the new merge key produces.
        let whole = [unit(MaterialId::MUDSTONE, 7.3)];
        let split = [
            unit(MaterialId::MUDSTONE, 4.1),
            unit(MaterialId::SILTSTONE, 3.2),
        ];
        let (a, b) = (exposed_shares(&whole), exposed_shares(&split));
        for (x, y) in a.shares().iter().zip(b.shares()) {
            assert!(
                (x - y).abs() <= 1e-12,
                "a within-class split moved the outcrop shares by more than                  rounding: {x} vs {y}"
            );
        }
        assert_eq!(
            dominant_litho(&a),
            dominant_litho(&b),
            "the outcrop verdict changed under a within-class split"
        );
    }

    /// Every class the deep record CAN deposit buckets *all* of its members back
    /// to itself — otherwise a recorded unit would coarsen into a class the
    /// erosion tables index differently from the one it was laid as.
    #[test]
    fn every_depositional_class_round_trips_through_the_bucket() {
        let set = dc_core::materials::geology::vanilla();
        for l in Litho::ALL {
            if l == Litho::Basement {
                continue;
            }
            let class = crate::geology::deep_class_of_species(l);
            let members = set.class(class).expect("vanilla declares it").members();
            assert!(!members.is_empty(), "{class} has no member");
            assert!(
                members
                    .iter()
                    .all(|&i| Litho::of_material(set.member(i).material) == l),
                "a member of {class} buckets somewhere other than {l:?}"
            );
        }
    }
}
