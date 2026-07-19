//! Geology strata passes: the per-column deposition log and the v1 processes
//! that populate it (docs/design/geology.md, content set DECIDED 2026-07-18).
//!
//! A [`StrataRec`] is an **ordered deposition log** attached beside the
//! chunk-column record: events bottom-up, each tagged with the selected
//! class member and climate-at-deposition (`climate_at` temp/precip —
//! column-quantized, accepted). Three collapse-phase passes populate it,
//! ordered by the pass graph (crate::pipeline):
//!
//! 1. **igneous emplacement** — province-driven via [`Provenance`]:
//!    intrusive basement at depth in orogeny/arc provinces, extrusive flows
//!    at the surface in rift/arc provinces.
//! 2. **clastic deposition** — climate/hydrology-driven: a column's fluvial
//!    energy (discharge-scaled, decaying away from the channel) splits the
//!    sediment budget into a coarse body below and fines above — the
//!    fining-upward stack every fan shows.
//! 3. **placer** — the S8 alluvial grain-sorting mechanism lifted to
//!    production: grains settle when flow energy drops below their
//!    property-derived threshold (`dc_core` `settle_energy`, reproducing the
//!    measured gravel > gold-dust > sand > silt > clay ordering). The dense
//!    ore grain therefore sorts into the graded coarse body: carried straight
//!    through where energy still exceeds its threshold, concentrated hardest
//!    just below it, thinning toward the toe of the fan.
//!
//! Member selection goes through the content-class machinery
//! ([`GeologySet::select`]): fitness × normalized abundance × an
//! **addressed** seed draw (`draw_f64`, new SALT_GEO_* addresses) — no
//! iteration-order entropy anywhere, and registration order cannot change a
//! single byte of the world (proven in tests/geology.rs).

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE, CLASS_IGNEOUS_INTRUSIVE,
    CLASS_ORE_PLACER, FormationContext, GeoMemberIdx, GeologySet, settle_energy,
};
use dc_sim::statistical::rng::draw_f64;

use crate::pregen::{Provenance, SALT_GEO_ORE, SALT_GEO_SELECT, SALT_GEO_THICK};

/// One deposition event: the selected member, its per-column thickness, and
/// the climate it was deposited under. `ore` is a placer enrichment riding
/// *inside* this stratum (grain habit): `(member, eighths per voxel)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrataEvent {
    pub member: GeoMemberIdx,
    pub thickness_vox: u8,
    pub temp_c: f32,
    pub precip: f32,
    pub ore: Option<(GeoMemberIdx, u8)>,
}

/// The ordered per-column deposition log, bottom-up: `events[0]` is the
/// deepest recorded stratum; the last event ends at the surface. Below the
/// record the column is unrecorded basement (deep default: stone).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StrataRec {
    pub events: Vec<StrataEvent>,
}

impl StrataRec {
    /// Total recorded thickness, voxels.
    pub fn total_vox(&self) -> u32 {
        self.events.iter().map(|e| u32::from(e.thickness_vox)).sum()
    }

    /// The event containing a voxel `depth` voxels below the surface voxel
    /// (depth 1 = directly under the surface). `None` below the record.
    pub fn event_at_depth(&self, depth: u32) -> Option<&StrataEvent> {
        let mut remaining = depth;
        for e in self.events.iter().rev() {
            let t = u32::from(e.thickness_vox);
            if remaining <= t {
                return Some(e);
            }
            remaining -= t;
        }
        None
    }
}

/// Everything a strata pass may see for one chunk-column — the bounded
/// context (cell-level climate/hydrology/provenance plus the column's own
/// state). Passes read and extend `strata`/`alluvium`; nothing else is
/// mutable.
pub struct StrataCtx<'a> {
    pub seed: u64,
    /// Chunk-column coordinates (the draw address).
    pub cx: i64,
    pub cz: i64,
    /// Sea-level temperature and precipitation at the column
    /// (`climate_at`, column-quantized).
    pub temp_c: f64,
    pub precip: f64,
    /// Tectonic provenance of the parent cell.
    pub provenance: Provenance,
    /// Mean surface elevation of the column footprint, meters.
    pub elev_m: f64,
    /// Fluvial energy at the column: discharge-scaled channel width decayed
    /// by distance from the nearest river segment; 0 = no fluvial influence.
    pub flow_energy: f64,
    /// True beyond the pregen grid.
    pub wilds: bool,
    pub geology: &'a GeologySet,
    // -- outputs --
    pub strata: StrataRec,
    /// Alluvial state: set by the clastic pass when it deposits a graded
    /// coarse body; consumed by the placer pass.
    pub alluvium: Option<AlluviumRec>,
}

/// The graded coarse body the clastic pass laid down, for the placer to
/// rework: which event it is and the energy that sorted it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlluviumRec {
    /// Index into `strata.events` of the coarse (graded) body.
    pub event: usize,
    pub energy: f64,
}

impl<'a> StrataCtx<'a> {
    fn formation(&self, depth_m: f64) -> FormationContext {
        FormationContext {
            temp_c: self.temp_c,
            precip: self.precip,
            depth_m,
        }
    }

    /// Addressed selection draw for `class_tag` at this column.
    fn draw(&self, salt: u64, tag: u64) -> f64 {
        draw_f64(&[self.seed, salt, tag, self.cx as u64, self.cz as u64])
    }

    fn push(&mut self, member: GeoMemberIdx, thickness_vox: u8, ore: Option<(GeoMemberIdx, u8)>) {
        self.strata.events.push(StrataEvent {
            member,
            thickness_vox,
            temp_c: self.temp_c as f32,
            precip: self.precip as f32,
            ore,
        });
    }
}

/// Emplacement depth (meters) used as the formation context for intrusive
/// selection: "at depth" for v1, one number rather than a per-column story.
const INTRUSIVE_DEPTH_M: f64 = 250.0;
/// Recorded thickness of the intrusive basement's *top*, voxels. Everything
/// below the record is unrecorded basement (default stone) anyway; this is
/// how much of the column reads as the selected pluton.
const INTRUSIVE_TOP_VOX: u8 = 96;

/// Igneous emplacement: province-driven. Orogeny/arc provinces get an
/// intrusive basement; rift/arc provinces get extrusive surface flows (an
/// arc gets both — pluton under lavas, which is what an arc is).
pub fn igneous_pass(ctx: &mut StrataCtx) {
    if ctx.elev_m <= 0.0 {
        return; // sea floor keeps its unrecorded basement in v1
    }
    if matches!(ctx.provenance, Provenance::Orogeny | Provenance::Arc)
        && let Some((member, _)) = ctx.geology.select(
            CLASS_IGNEOUS_INTRUSIVE,
            &ctx.formation(INTRUSIVE_DEPTH_M),
            ctx.draw(SALT_GEO_SELECT, 0),
        )
    {
        ctx.push(member, INTRUSIVE_TOP_VOX, None);
    }
    if matches!(ctx.provenance, Provenance::Rift | Provenance::Arc)
        && let Some((member, _)) = ctx.geology.select(
            CLASS_IGNEOUS_EXTRUSIVE,
            &ctx.formation(5.0),
            ctx.draw(SALT_GEO_SELECT, 1),
        )
    {
        let thickness = 2 + (ctx.draw(SALT_GEO_THICK, 1) * 3.0) as u8;
        ctx.push(member, thickness, None);
    }
}

/// Fraction of the clastic budget that is coarse at flow energy `e` — the
/// graded split: high-energy proximal columns keep the coarse load, distal
/// low-energy columns receive fines.
pub fn coarse_fraction(e: f64) -> f64 {
    (e / 30.0).clamp(0.0, 0.85)
}

/// Clastic deposition: climate- and hydrology-driven. The sediment budget
/// scales with precipitation (the old soil-depth signal) plus a fluvial
/// bonus; the energy-graded split stacks a coarse body below fines — the
/// fining-upward sequence.
pub fn clastic_pass(ctx: &mut StrataCtx) {
    if ctx.elev_m <= 0.0 {
        return; // subaqueous sedimentation is the carbonate milestone
    }
    // Budget in voxels: arid columns get a thin veneer, wet columns a real
    // soil column, fan columns extra.
    let base = 1.0 + (ctx.precip * 2.5);
    let fluvial = (ctx.flow_energy / 12.0).min(3.0);
    let total = (base + fluvial).round().clamp(1.0, 8.0) as u32;
    let coarse = (f64::from(total) * coarse_fraction(ctx.flow_energy)).round() as u32;
    let fine = total - coarse;

    if coarse > 0
        && let Some((member, _)) = ctx.geology.select(
            CLASS_CLASTIC_COARSE,
            &ctx.formation(2.0),
            ctx.draw(SALT_GEO_SELECT, 2),
        )
    {
        ctx.push(member, coarse as u8, None);
        ctx.alluvium = Some(AlluviumRec {
            event: ctx.strata.events.len() - 1,
            energy: ctx.flow_energy,
        });
    }
    if fine > 0
        && let Some((member, _)) = ctx.geology.select(
            CLASS_CLASTIC_FINE,
            &ctx.formation(1.0),
            ctx.draw(SALT_GEO_SELECT, 3),
        )
    {
        ctx.push(member, fine as u8, None);
    }
}

/// Scale from the property-derived settle threshold (`settle_energy`, in
/// sqrt(mm·SG) units) to column flow-energy units. Calibration, not physics:
/// column flow energy equals channel width at the channel (6..40), and this
/// scale puts the ore threshold (gold-dust: ~3.6 units) at ~14 — inside the
/// energy of every real river (min width ~15.5 at the river discharge
/// threshold), so the S8 zonation actually occurs in worlds: barren
/// proximal channel core (still carried), peak grade in the winnowing band
/// just off it, thinning to nothing down-fan.
const SETTLE_TO_FLOW: f64 = 4.0;

/// Flow-energy threshold below which this member's grains settle out.
pub fn member_settle_threshold(set: &GeologySet, member: GeoMemberIdx) -> f64 {
    SETTLE_TO_FLOW * settle_energy(set.member(member).material.props())
}

/// Ore concentration (eighths per voxel, 0..=3) in a graded body sorted at
/// flow energy `e`, for an ore whose settle threshold is `threshold`.
///
/// The S8 mechanism, per column: while `e >= threshold` the flow still
/// carries the ore — nothing settles (the proximal channel is barren). Just
/// below the threshold the ore drops while remaining energy winnows the
/// light fraction away — peak concentration. Toward the toe the energy that
/// would concentrate it is gone — grade thins to nothing.
pub fn ore_eighths(e: f64, threshold: f64) -> u8 {
    if e >= threshold || e <= 0.0 {
        return 0;
    }
    let c = e / threshold; // in (0, 1): highest just under the threshold
    (c * 3.0).round() as u8
}

/// Placer pass: rework the graded alluvial body, settling the dense ore
/// grain into it by the energy-threshold rule.
pub fn placer_pass(ctx: &mut StrataCtx) {
    let Some(alluvium) = ctx.alluvium else {
        return; // no fan, no placer
    };
    let Some((ore_member, _)) = ctx.geology.select(
        CLASS_ORE_PLACER,
        &ctx.formation(2.0),
        ctx.draw(SALT_GEO_ORE, 0),
    ) else {
        return;
    };
    let threshold = member_settle_threshold(ctx.geology, ore_member);
    let eighths = ore_eighths(alluvium.energy, threshold);
    if eighths > 0 {
        ctx.strata.events[alluvium.event].ore = Some((ore_member, eighths));
    }
}
