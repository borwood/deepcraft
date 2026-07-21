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
    CLASS_ACCESSORY_MAFIC, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE,
    CLASS_IGNEOUS_INTRUSIVE, CLASS_ORE_PLACER, CLASS_ORGANIC_COAL, CLASS_ORGANIC_PEAT,
    CLASS_ORGANIC_SOIL, FormationContext, GeoMemberIdx, GeologySet, settle_energy,
};
use dc_sim::statistical::rng::draw_f64;

use crate::deeptime::recorder::{Aridity, Biofacies, DepEnv, DepTag, DepUnit, EnergyBand};
use crate::pregen::{
    Provenance, SALT_GEO_ACC, SALT_GEO_DEEP, SALT_GEO_ORE, SALT_GEO_SELECT, SALT_GEO_THICK,
};

/// One deposition event: the selected member, its per-column thickness, and
/// the formation context it was deposited under. `ore` is a placer enrichment
/// riding *inside* this stratum (grain habit): `(member, eighths per voxel)`;
/// `accessory` is an igneous inclusion carried in the host rock's pore slots
/// (`(member, eighths per voxel)`, 3d pore partials).
///
/// The context and selection address (`depth_m`, `sel_salt`, `sel_tag`) are
/// recorded so the material tier can **re-resolve the host member per
/// voxel-column** with a boundary-dithered draw, smoothing family contacts off
/// the chunk grid (the chunk-line cutover fix, 3c-2). `member` is the
/// chunk-centre representative — the record's canonical identity — while the
/// dither interpolates the same selection field across the footprint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrataEvent {
    pub member: GeoMemberIdx,
    pub thickness_vox: u8,
    pub temp_c: f32,
    pub precip: f32,
    /// Emplacement/formation depth used for member fitness (meters).
    pub depth_m: f32,
    /// Selection-draw address of the class this event filled — the field the
    /// per-voxel dither re-samples.
    pub sel_salt: u64,
    pub sel_tag: u64,
    pub ore: Option<(GeoMemberIdx, u8)>,
    pub accessory: Option<(GeoMemberIdx, u8)>,
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
    /// Voxel edge (metres) — the deep-time record's metres are quantized to it.
    pub voxel_m: f64,
    /// **Recorded regolith thickness `H` at the column, metres** — the deep
    /// sim's loose-cover plane, bilinearly sampled (`DeepField::regolith_at_voxel`).
    /// `None` **only in the border wilds**, where no deep-time history exists;
    /// there the clastic veneer keeps its year-zero precipitation fallback
    /// (genesis synthesis, stubs.md § Genesis).
    pub regolith_m: Option<f64>,
    /// The deep-time strata record for this column (nearest 460 m deep cell),
    /// bottom-up units tagged at deposition. Empty in the wilds / where the deep
    /// sim laid nothing down (bare erosional uplands). The **at-deposition
    /// formation-context source** for depositional strata (3e-1): the clastic
    /// pass reads these instead of the year-zero climate shim.
    pub deep_units: &'a [DepUnit],
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

    /// Selection draw at the chunk-column **centre**, read from the smooth
    /// bilinear field the material tier reuses per voxel-column
    /// ([`interp_select_draw`]). Sampling the interpolated field (not a single
    /// per-chunk hash) is what keeps the record's representative member in
    /// agreement with the dithered footprint at its centre.
    fn draw(&self, salt: u64, tag: u64) -> f64 {
        interp_select_draw(self.seed, salt, tag, self.cx, self.cz, 0.5, 0.5)
    }

    fn push(&mut self, member: GeoMemberIdx, thickness_vox: u8, salt: u64, tag: u64, depth_m: f64) {
        self.strata.events.push(StrataEvent {
            member,
            thickness_vox,
            temp_c: self.temp_c as f32,
            precip: self.precip as f32,
            depth_m: depth_m as f32,
            sel_salt: salt,
            sel_tag: tag,
            ore: None,
            accessory: None,
        });
    }
}

/// Bilinear interpolation of the per-chunk-column selection hash to a
/// fractional position `(fx, fz)` inside the chunk. The four samples are the
/// chunk-column **corner** hashes, so neighbouring chunks share edge values and
/// the field is C0-continuous across chunk borders: a class's member-partition
/// boundary becomes a smooth curve that wanders like a facies contact instead
/// of snapping to the 28.8 m chunk grid (the chunk-line family cutover fix).
///
/// Pure in `(seed, salt, tag, coords)` — no iteration-order entropy,
/// registration-order independent, and it consults no cells or columns, so the
/// lookahead bounds are untouched.
pub(crate) fn interp_select_draw(
    seed: u64,
    salt: u64,
    tag: u64,
    cx: i64,
    cz: i64,
    fx: f64,
    fz: f64,
) -> f64 {
    let corner =
        |dx: i64, dz: i64| draw_f64(&[seed, salt, tag, (cx + dx) as u64, (cz + dz) as u64]);
    let (u00, u10, u01, u11) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
    let a = u00 * (1.0 - fx) + u10 * fx;
    let b = u01 * (1.0 - fx) + u11 * fx;
    (a * (1.0 - fz) + b * fz).clamp(0.0, 1.0 - f64::EPSILON)
}

/// Fraction of an igneous column that carries an accessory inclusion (sparse
/// presence gate) and the pore eighths it fills when present. Subtle by
/// construction — the 3c-2 face dither renders the single pore eighth as
/// scattered cells with no renderer-side code (geology.md § inclusions).
const ACC_PRESENCE: f64 = 0.6;
const ACC_EIGHTHS: u8 = 1;

/// Emplace an accessory inclusion into the igneous event at `event_idx`:
/// province/depth-driven class selection, gated sparsely per column. No-op
/// when the accessory class is empty (define-time enforcement guarantees it is
/// not, for a registered pack) or the gate is closed.
fn emplace_accessory(ctx: &mut StrataCtx, event_idx: usize, depth_m: f64, tag: u64) {
    if ctx.draw(SALT_GEO_ACC, tag) >= ACC_PRESENCE {
        return; // this column's igneous rock is accessory-free
    }
    let form = ctx.formation(depth_m);
    let pick = ctx.geology.select(
        CLASS_ACCESSORY_MAFIC,
        &form,
        ctx.draw(SALT_GEO_ACC, tag + 1024),
    );
    if let Some((acc, _)) = pick
        && let Some(event) = ctx.strata.events.get_mut(event_idx)
    {
        event.accessory = Some((acc, ACC_EIGHTHS));
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
        ctx.push(
            member,
            INTRUSIVE_TOP_VOX,
            SALT_GEO_SELECT,
            0,
            INTRUSIVE_DEPTH_M,
        );
        let idx = ctx.strata.events.len() - 1;
        emplace_accessory(ctx, idx, INTRUSIVE_DEPTH_M, 0);
    }
    if matches!(ctx.provenance, Provenance::Rift | Provenance::Arc)
        && let Some((member, _)) = ctx.geology.select(
            CLASS_IGNEOUS_EXTRUSIVE,
            &ctx.formation(5.0),
            ctx.draw(SALT_GEO_SELECT, 1),
        )
    {
        let thickness = 2 + (ctx.draw(SALT_GEO_THICK, 1) * 3.0) as u8;
        ctx.push(member, thickness, SALT_GEO_SELECT, 1, 5.0);
        let idx = ctx.strata.events.len() - 1;
        emplace_accessory(ctx, idx, 5.0, 1);
    }
}

/// Fraction of the clastic budget that is coarse at flow energy `e` — the
/// graded split: high-energy proximal columns keep the coarse load, distal
/// low-energy columns receive fines.
pub fn coarse_fraction(e: f64) -> f64 {
    (e / 30.0).clamp(0.0, 0.85)
}

/// Paleo-precipitation proxy for a deep-time deposition tag: the recorder's
/// aridity axis (measured from the marched precip *at deposition*) mapped onto
/// the normalized-precip axis clastic-fine fitness reads. Subsea units carry the
/// recorder's `Humid` normalization, so marine bands read humid → fine mud,
/// which is the intent.
fn deep_precip(tag: DepTag) -> f64 {
    match tag.aridity {
        Aridity::Arid => 0.12,
        Aridity::Humid => 0.60,
    }
}

/// The content class a deep-time unit deposits into, from its measured facies
/// tags. Roster-independent — only *which* member fills the class is selection
/// (so adding a member never changes the class share, the invariant
/// tests/geology.rs asserts).
///
/// **The biotic facies axis is consulted first, and it wins where inhabited**
/// (journal/0026 — the S10 gap). An organic unit is a *different rock*, not a
/// clastic one: what made a coal seam is that peat accumulated faster than it
/// decayed and was then buried, and the energy of whatever flow happened to be
/// passing says nothing about that. Reading env/energy for an organic unit is
/// what made the measured 24 m seam at world voxel (107338, 58787) collapse as
/// ordinary sandstone. Where the biotic axis is `Mineral` — every unit in a
/// biology-off world, and every clastic unit in a biology-on one — the original
/// env/energy rule stands unchanged: marine (subsea) → fine mud; subaerial
/// high/medium energy → coarse proximal bodies; subaerial low energy → distal
/// fines.
///
/// [`Biofacies::Charcoal`] is deliberately **not** routed to a class of its
/// own: a fire bed is a *thin event bed* (measured mean ~0.035 m over 158 310
/// beds, and **none** of them survives the 0.9 m voxel quantization), so a
/// charcoal band cannot exist in a voxel column and a charcoal member would be
/// dead content. A charcoal-tagged unit therefore reads as the mineral host it
/// is a streak within. The honest representation is an inclusion (pore/debris
/// partial, geology.md § inclusions) — filed, not built. See journal/0026.
///
/// **Public because deep time now depends on it.** The erodibility coupling
/// (journal/0029) needs to know which rock resisted erosion at a cell, and that
/// must be the same rock the collapse layer will build there — otherwise the
/// world's shape stops explaining the world's rock. `deeptime::lithology::
/// litho_of_tag` mirrors this routing and a test asserts they agree over every
/// tag in the space.
pub fn deep_class(tag: DepTag) -> &'static str {
    match tag.biota {
        Biofacies::Coal => CLASS_ORGANIC_COAL,
        Biofacies::Peat => CLASS_ORGANIC_PEAT,
        // A retrogressive horizon IS an organic soil horizon; what makes it
        // "retrogressive" is the community's phosphorus starvation, which is an
        // ecological fact with no material expression in the property sheet.
        Biofacies::Soil | Biofacies::Retro => CLASS_ORGANIC_SOIL,
        Biofacies::Charcoal | Biofacies::Mineral => match tag.env {
            DepEnv::Subsea => CLASS_CLASTIC_FINE,
            DepEnv::Subaerial => match tag.energy {
                EnergyBand::High | EnergyBand::Medium => CLASS_CLASTIC_COARSE,
                EnergyBand::Low => CLASS_CLASTIC_FINE,
            },
        },
    }
}

/// Overburden (metres) attributed to the active surficial veneer above the
/// deep-time record, added to each deep unit's burial depth so the
/// formation-context depth axis is honest.
const DEEP_VENEER_MARGIN_M: f64 = 2.0;

/// Deposit the deep-time depositional history below the active veneer: one
/// stratum per recorded deep unit (bottom-up), its class fixed by the measured
/// facies tag ([`deep_class`]) and its member selected under the
/// **at-deposition** formation context — paleo precipitation from the aridity
/// tag ([`deep_precip`]), temperature from the column's latitude (paleo-
/// temperature is a later 3e slice), and burial depth from the overlying
/// record. This is the point of 3e-1: a cut face reads the record of a landscape
/// that ran (marine mud under arid fill under the recent veneer), not the
/// year-zero climate shim.
///
/// **Returns the voxel thickness it actually expressed** — the whole-voxel part
/// of the loose column. journal/0053: the deep sim's regolith plane `H` equals
/// the sum of these units' metres exactly, so `round(H / voxel_m)` minus this
/// return value is the part of the loose column the 0.9 m sieve could not
/// resolve, which [`clastic_pass`] then expresses as the surficial veneer
/// instead of deleting it.
fn deposit_deep_history(ctx: &mut StrataCtx) -> u32 {
    let total_m: f64 = ctx.deep_units.iter().map(|u| u.thickness_m).sum();
    let mut expressed_vox = 0u32;
    let mut below_m = 0.0;
    for (k, u) in ctx.deep_units.iter().enumerate() {
        let depth_above = (total_m - below_m - u.thickness_m).max(0.0);
        below_m += u.thickness_m;
        let tv = (u.thickness_m / ctx.voxel_m).round();
        if tv < 1.0 {
            // Sub-voxel unit: a condensed couplet the record keeps but 0.9 m
            // blocks cannot resolve. Dropped consistently (roster-independent).
            continue;
        }
        let thickness_vox = tv.min(255.0) as u8;
        let class = deep_class(u.tag);
        let precip = deep_precip(u.tag);
        let depth_m = depth_above + DEEP_VENEER_MARGIN_M;
        let temp_c = ctx.temp_c; // latitude proxy; paleo-temp curve is 3e-later
        let form = FormationContext {
            temp_c,
            precip,
            depth_m,
        };
        let tag = k as u64;
        let u_draw = interp_select_draw(ctx.seed, SALT_GEO_DEEP, tag, ctx.cx, ctx.cz, 0.5, 0.5);
        if let Some((member, _)) = ctx.geology.select(class, &form, u_draw) {
            ctx.strata.events.push(StrataEvent {
                member,
                thickness_vox,
                temp_c: temp_c as f32,
                precip: precip as f32,
                depth_m: depth_m as f32,
                sel_salt: SALT_GEO_DEEP,
                sel_tag: tag,
                ore: None,
                accessory: None,
            });
            expressed_vox += u32::from(thickness_vox);
        }
    }
    expressed_vox
}

/// Clastic deposition. The deep-time record (3e-1) supplies the depositional
/// **history** below an active surficial **veneer**: [`deposit_deep_history`]
/// lays the recorded units (at-deposition context) just above the igneous
/// basement, then the veneer below deposits the recent, still-forming alluvium
/// under the year-zero climate (ratified-legitimate for the veneer — geology.md
/// § formation context) and hands its graded coarse body to the placer. In the
/// wilds / bare uplands (no deep record) only the veneer remains — the
/// historyless border keeps the analytic year-zero behaviour.
///
/// Since journal/0053 the veneer's *thickness* is no longer a climate guess: it
/// is the un-whole-voxel remainder of the deep sim's loose column `H` (see
/// below). Its *member selection* still reads the year-zero climate, which is
/// the part geology.md ratifies for an actively-forming surficial body.
pub fn clastic_pass(ctx: &mut StrataCtx) {
    if ctx.elev_m <= 0.0 {
        return; // subaqueous sedimentation is the carbonate milestone
    }
    // The recorded deep-time history sits below the active veneer, and tells us
    // how many whole voxels of the loose column it managed to express.
    let expressed_vox = deposit_deep_history(ctx);

    // **The veneer budget, from the recorded cause** (journal/0053 — retiring the
    // veneer half of stubs.md § 3).
    //
    // The old budget was `1.0 + precip*2.5`, a present-day-rainfall guess with a
    // floor of one voxel, so no column in the world could ever be bare
    // (journal/0049 station 1: the user standing in the most wind-stripped
    // country on the map under topsoil, "always going to have topsoil").
    //
    // What the ledger actually says, measured: the deep sim's regolith plane `H`
    // is **exactly** the sum of the recorded units' metres — the recorder logs
    // every metre of loose cover the sim lays down, so the record IS the loose
    // column, decomposed into beds. That makes the honest veneer the part of the
    // column whole voxels could not express: `round(H / voxel_m)` minus what
    // `deposit_deep_history` just laid. Those are the beds thinner than half a
    // voxel that the sieve drops — and dropping them outright would delete
    // three-quarters of the world's loose cover (measured mean: 4.75 m of `H`
    // per subaerial cell, of which only ~1.15 m survives as whole-voxel units).
    //
    // Amalgamating them into one surficial body is not a fudge, it is the
    // physical process: bioturbation, creep and soil mixing homogenize thin beds
    // into a surficial mantle, which is why real soil is not laminated. And it
    // makes the fill **mass-conserving against the ledger**: expressed loose
    // voxels = `round(H / voxel_m)`, exactly, wherever the cap does not bite.
    //
    // The **fluvial term stays** on top: `H` is a ~460 m deep-cell quantity and a
    // channel with its fan is a sub-deep-cell feature the deep grid cannot
    // resolve. That is the enhancement doctrine's sanctioned case — a local
    // procedure answering "what does this point look like inside this regional
    // field", conditioned on the collapse tier's own river network. It is also
    // the term that keeps a graded coarse body under the placer.
    //
    // The lower clamp is **0**, not 1: bareness is expressible now.
    let base = match ctx.regolith_m {
        Some(h) => ((h / ctx.voxel_m).round() - f64::from(expressed_vox)).max(0.0),
        // Wilds only: no deep-time history to read (genesis synthesis —
        // stubs.md § Genesis, border-wilds cell synthesis).
        None => 1.0 + (ctx.precip * 2.5),
    };
    let fluvial = (ctx.flow_energy / 12.0).min(3.0);
    let total = (base + fluvial).round().clamp(0.0, 8.0) as u32;
    let coarse = (f64::from(total) * coarse_fraction(ctx.flow_energy)).round() as u32;
    let fine = total - coarse;

    if coarse > 0
        && let Some((member, _)) = ctx.geology.select(
            CLASS_CLASTIC_COARSE,
            &ctx.formation(2.0),
            ctx.draw(SALT_GEO_SELECT, 2),
        )
    {
        ctx.push(member, coarse as u8, SALT_GEO_SELECT, 2, 2.0);
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
        ctx.push(member, fine as u8, SALT_GEO_SELECT, 3, 1.0);
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

/// Resolve the **host member** of one event for a single voxel-column,
/// dithering the class selection across the chunk footprint (the material-tier
/// smoothing of the chunk-line family cutover). At the chunk centre this
/// reproduces `event.member`; away from it the interpolated selection field
/// ([`interp_select_draw`]) re-picks within the event's class under the
/// recorded formation context, so the contact between two members of a class
/// wanders like a facies boundary instead of snapping to chunk lines. The
/// block tier is unaffected (both members share a class, hence a block); only
/// the material albedo the mesher dithers changes.
pub fn dithered_member(
    geology: &GeologySet,
    seed: u64,
    event: &StrataEvent,
    cx: i64,
    cz: i64,
    x: usize,
    z: usize,
) -> GeoMemberIdx {
    let class = geology.member(event.member).class.as_str();
    let fx = (x as f64 + 0.5) / 32.0;
    let fz = (z as f64 + 0.5) / 32.0;
    let u = interp_select_draw(seed, event.sel_salt, event.sel_tag, cx, cz, fx, fz);
    let ctx = FormationContext {
        temp_c: f64::from(event.temp_c),
        precip: f64::from(event.precip),
        depth_m: f64::from(event.depth_m),
    };
    geology
        .select(class, &ctx, u)
        .map_or(event.member, |(i, _)| i)
}
