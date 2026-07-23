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
    CLASS_IGNEOUS_INTRUSIVE, CLASS_ORE_PLACER, CLASS_ORGANIC_CHARCOAL, CLASS_ORGANIC_COAL,
    CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL, FormationContext, GeoMemberIdx, GeologySet,
    settle_energy,
};
use dc_sim::statistical::rng::draw_f64;

use crate::deeptime::providers::PaleoUnit;
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
///
/// **Thickness is METRES, not voxels** (materials.md DECIDED 2026-07-21,
/// distribution-first expression). The record is a continuous stack; the
/// quantization to eighths happens **once**, at contents construction, from the
/// units overlapping a voxel's own 0.9 m span (`crate::fill`). Rounding each
/// event to whole voxels here is the `Σ round` / `round Σ` defect journal/0055
/// removes — it deleted ~75 % of the world's recorded sediment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrataEvent {
    pub member: GeoMemberIdx,
    /// Recorded thickness in **metres**. Never rounded to voxels here.
    pub thickness_m: f32,
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
    /// Total recorded thickness, **metres**.
    pub fn total_m(&self) -> f64 {
        self.events.iter().map(|e| f64::from(e.thickness_m)).sum()
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
    /// The world's resolved provider set — the collapse tier's access to the
    /// same seam mechanism the deep-time sim carries in `DeepConfig`. `Copy`,
    /// default (all-identity) until an heir is resolved at world build. The
    /// clastic pass reads `paleo_temperature` through it (journal/0078); a
    /// default set reproduces the pre-seam `ctx.temp_c` byte-for-byte.
    pub providers: crate::deeptime::providers::Providers,
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

    fn push(&mut self, member: GeoMemberIdx, thickness_m: f64, salt: u64, tag: u64, depth_m: f64) {
        self.strata.events.push(StrataEvent {
            member,
            thickness_m: thickness_m as f32,
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
        let t = f64::from(INTRUSIVE_TOP_VOX) * ctx.voxel_m;
        ctx.push(member, t, SALT_GEO_SELECT, 0, INTRUSIVE_DEPTH_M);
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
        let thickness = f64::from(2 + (ctx.draw(SALT_GEO_THICK, 1) * 3.0) as u8) * ctx.voxel_m;
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
/// [`Biofacies::Charcoal`] routes to [`CLASS_ORGANIC_CHARCOAL`] **as of
/// journal/0063**, and the reason it did not before is a good illustration of a
/// justification outliving its mechanism. The old comment here read: *a fire bed
/// is a thin event bed (measured mean ~0.035 m over 158 310 beds, and none of
/// them survives the 0.9 m voxel quantization), so a charcoal band cannot exist
/// in a voxel column and a charcoal member would be dead content. The honest
/// representation is an inclusion (pore/debris partial) — filed, not built.*
///
/// Every clause of that is still true except the load-bearing one. Since
/// journal/0055 a bed does not have to *survive quantization* to be expressed:
/// `crate::fill` allocates a voxel's eighths from the units overlapping its
/// span by **unbiased addressed stochastic rounding**, so a 3.5 cm bed claims
/// `8 × 0.035 / 0.9 ≈ 0.31` of an eighth and therefore wins a whole eighth
/// about 31 % of the time it is asked. That is precisely the inclusion the old
/// comment called honest and filed as unbuilt — the filing was overtaken by a
/// slice aimed at something else.
///
/// So charcoal is now its own class, and the class contract says what it is: an
/// inclusion, never a stratum. See journal/0063 for what it actually measures
/// out to in the world.
///
/// **Public because deep time now depends on it.** The erodibility coupling
/// (journal/0029) needs to know which rock resisted erosion at a cell, and that
/// must be the same rock the collapse layer will build there — otherwise the
/// world's shape stops explaining the world's rock. `deeptime::lithology::
/// litho_of_tag` mirrors this routing **totally**, over every tag in the space,
/// with no exception (asserted by `tests/erodibility.rs::
/// litho_routing_matches_the_collapse_tier`). Charcoal used to be a deliberate
/// divergence here — the collapse tier expressed the carbon while deep time read
/// the host bed — but that was a *thickness* rule wearing a content name
/// (A-7): the real requirement is "a bed too thin to fill an erosion cell must
/// not define its lithology", and it now lives generally in
/// [`exposed_litho`](crate::deeptime::lithology::exposed_litho)'s dominance
/// window, so both tiers can agree the fire bed is charcoal while it still never
/// *outcrops* one (journal/0068).
pub fn deep_class(tag: DepTag) -> &'static str {
    match tag.biota {
        Biofacies::Coal => CLASS_ORGANIC_COAL,
        Biofacies::Peat => CLASS_ORGANIC_PEAT,
        Biofacies::Charcoal => CLASS_ORGANIC_CHARCOAL,
        // A retrogressive horizon IS an organic soil horizon; what makes it
        // "retrogressive" is the community's phosphorus starvation, which is an
        // ecological fact with no material expression in the property sheet.
        Biofacies::Soil | Biofacies::Retro => CLASS_ORGANIC_SOIL,
        Biofacies::Mineral => match tag.env {
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
/// tag ([`deep_precip`]), temperature through the
/// [`paleo_temperature`](crate::deeptime::providers::Providers::paleo_temperature)
/// provider seam (identity = the column's present-day temperature, so this is
/// byte-identical until a paleoclimate heir lands — journal/0078), and burial
/// depth from the overlying
/// record. This is the point of 3e-1: a cut face reads the record of a landscape
/// that ran (marine mud under arid fill under the recent veneer), not the
/// year-zero climate shim.
///
/// **No unit is rounded and no unit is dropped** (materials.md DECIDED
/// 2026-07-21). Until journal/0055 this function computed `Σ round(tᵢ / 0.9)`
/// where honesty requires `round(Σ tᵢ / 0.9)`: each unit was rounded to whole
/// voxels *independently* and skipped if it did not reach one, with no remainder
/// carried forward, so the errors compounded instead of cancelling. Measured
/// consequence: three-quarters of the world's recorded sediment pile deleted,
/// and the tour's dune field — 379 units summing to 7.99 m, averaging 0.021 m
/// each — expressing **zero**. Not because 0.9 m voxels cannot hold eight metres
/// of sand, but because the rounding question was asked 379 times instead of
/// once.
///
/// Metres now survive to the voxel boundary. `crate::fill` does the single
/// quantization, filling a voxel's eighths from the units overlapping its own
/// span with an addressed stochastic draw.
///
/// **Returns the metres it actually expressed.** Since `Σ(recorded unit
/// thicknesses) ≡ H` exactly (journal/0053) and nothing is dropped, this now
/// equals `H` for any column whose classes all resolve to a member — which is
/// what retires the clastic veneer's thickness budget without deleting a line of
/// it (see [`clastic_pass`]).
///
/// **Run-length coalescing.** Adjacent units that resolve to the *same member*
/// are merged into one event. This is lossless for expression — contents depend
/// on the member, not on how many recorder units contributed it — and it is what
/// keeps the per-column event vector (and the per-voxel candidate list) bounded
/// when a cell records hundreds of thin beds. It is *not* the pre-merge heir
/// filed in stubs.md § 12: that one merged unlike beds and lost them; this one
/// merges only beds that would express identically anyway.
fn deposit_deep_history(ctx: &mut StrataCtx) -> f64 {
    let total_m: f64 = ctx.deep_units.iter().map(|u| u.thickness_m).sum();
    let mut expressed_m = 0.0f64;
    let mut below_m = 0.0;
    // Index of the event this pass pushed last, for run-length coalescing.
    let mut last: Option<(usize, GeoMemberIdx)> = None;
    for (k, u) in ctx.deep_units.iter().enumerate() {
        let depth_above = (total_m - below_m - u.thickness_m).max(0.0);
        below_m += u.thickness_m;
        if u.thickness_m <= 0.0 {
            continue;
        }
        let class = deep_class(u.tag);
        let precip = deep_precip(u.tag);
        let depth_m = depth_above + DEEP_VENEER_MARGIN_M;
        // **At-deposition temperature — the `paleo_temperature` seam** (#11,
        // journal/0078). Pre-seam this read `ctx.temp_c` (the column's *present*
        // climate) for every unit — the wrong quantity, while the sibling
        // `precip` axis above already reads the recorder's own tag. The identity
        // provider returns that same `ctx.temp_c`, so a default world is
        // byte-identical; the heir is an epoch-indexed paleo curve indexed by the
        // unit's `chapter`. Value-level per unit because the heir's answer varies
        // per epoch even though the identity's does not (granularity follows the
        // heir — providers/mod.rs).
        let temp_c = ctx.providers.paleo_temperature(PaleoUnit {
            cx: ctx.cx,
            cz: ctx.cz,
            chapter: u.chapter,
            present_temp_c: ctx.temp_c,
        });
        let form = FormationContext {
            temp_c,
            precip,
            depth_m,
        };
        let tag = k as u64;
        let u_draw = interp_select_draw(ctx.seed, SALT_GEO_DEEP, tag, ctx.cx, ctx.cz, 0.5, 0.5);
        if let Some((member, _)) = ctx.geology.select(class, &form, u_draw) {
            expressed_m += u.thickness_m;
            if let Some((idx, prev)) = last
                && prev == member
            {
                let e = &mut ctx.strata.events[idx];
                e.thickness_m += u.thickness_m as f32;
                continue;
            }
            ctx.strata.events.push(StrataEvent {
                member,
                thickness_m: u.thickness_m as f32,
                temp_c: temp_c as f32,
                precip: precip as f32,
                depth_m: depth_m as f32,
                sel_salt: SALT_GEO_DEEP,
                sel_tag: tag,
                ore: None,
                accessory: None,
            });
            last = Some((ctx.strata.events.len() - 1, member));
        }
    }
    expressed_m
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
    let expressed_m = deposit_deep_history(ctx);

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
    //
    // **journal/0055 — the budget self-retires, and is left in place to prove
    // it.** The residue term above was `round(H / voxel_m) − expressed_voxels`
    // precisely because whole-voxel expression could not carry a thin bed. Now
    // that nothing is rounded or dropped, `expressed_m` *is* `H` for any column
    // whose classes all resolve, so the residue is **0 metres** and the veneer
    // contributes nothing on its own. The subtraction is written in metres now
    // (it no longer has to compensate for a quantizer), and it is deliberately
    // NOT deleted: it is the honest statement of "what the record could not
    // carry", it still fires in the degenerate case where a class fails to
    // resolve a member, and stubs.md § 12 asked for verification rather than
    // surgery.
    let base = match ctx.regolith_m {
        Some(h) => ((h - expressed_m) / ctx.voxel_m).max(0.0),
        // Wilds only: no deep-time history to read (genesis synthesis —
        // stubs.md § Genesis, border-wilds cell synthesis).
        None => 1.0 + (ctx.precip * 2.5),
    };
    let fluvial = (ctx.flow_energy / 12.0).min(3.0);
    // **The veneer is metres now too** (journal/0055). It used to round its
    // budget to whole voxels, and while the residue term was carrying 1–8 voxels
    // of amalgamated soil that rounding was invisible. With the residue at zero
    // the veneer *is* the fluvial fan and nothing else — and a modest river's
    // fan is a fraction of a voxel, so rounding it deleted the entire graded
    // coarse body, and with it every placer in the world. (Measured the hard
    // way: `placer_follows_the_sorted_gradient` went from passing to "no placer
    // deposits found in the sample".) The same rule as everywhere else in this
    // slice fixes it: keep the metres, let `crate::fill` quantize once.
    let total = (base + fluvial).min(8.0);
    let coarse = total * coarse_fraction(ctx.flow_energy);
    let fine = total - coarse;
    let (coarse_m, fine_m) = (coarse * ctx.voxel_m, fine * ctx.voxel_m);

    if coarse_m > 0.0
        && let Some((member, _)) = ctx.geology.select(
            CLASS_CLASTIC_COARSE,
            &ctx.formation(2.0),
            ctx.draw(SALT_GEO_SELECT, 2),
        )
    {
        ctx.push(member, coarse_m, SALT_GEO_SELECT, 2, 2.0);
        ctx.alluvium = Some(AlluviumRec {
            event: ctx.strata.events.len() - 1,
            energy: ctx.flow_energy,
        });
    }
    if fine_m > 0.0
        && let Some((member, _)) = ctx.geology.select(
            CLASS_CLASTIC_FINE,
            &ctx.formation(1.0),
            ctx.draw(SALT_GEO_SELECT, 3),
        )
    {
        ctx.push(member, fine_m, SALT_GEO_SELECT, 3, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deeptime::providers::Providers;
    use crate::deeptime::recorder::{Aridity, DepEnv, DepTag, DepUnit, EnergyBand};
    use dc_core::materials::geology::vanilla;

    fn one_mineral_unit() -> Vec<DepUnit> {
        // Subaerial / medium energy routes to a clastic class the vanilla set
        // fills, so `select` returns a member and an event is actually pushed.
        vec![DepUnit {
            tag: DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::Medium),
            thickness_m: 5.0,
            unconformity: false,
            chapter: 2,
        }]
    }

    fn ctx_over<'a>(
        units: &'a [DepUnit],
        geo: &'a GeologySet,
        providers: Providers,
        present_temp_c: f64,
    ) -> StrataCtx<'a> {
        StrataCtx {
            seed: 42,
            cx: 3,
            cz: -7,
            temp_c: present_temp_c,
            precip: 0.5,
            provenance: Provenance::OceanFloor,
            elev_m: 100.0,
            flow_energy: 0.0,
            voxel_m: 0.9,
            regolith_m: Some(5.0),
            deep_units: units,
            wilds: false,
            geology: geo,
            providers,
            strata: StrataRec::default(),
            alluvium: None,
        }
    }

    /// **The `paleo_temperature` seam is consulted in the real collapse fn, and
    /// the payload carries what the heir needs.** The golden proves an *absent*
    /// provider changes nothing — which is equally consistent with a slot that is
    /// never called (the `burial_temp_c` falsifier's argument, one tier over).
    /// This drives `deposit_deep_history` directly:
    ///
    /// - with the identity set, the deposited event's `temp_c` is the column's
    ///   present-day temperature (byte-identical to the pre-seam `ctx.temp_c`);
    /// - with a swapped provider returning `present*100 + chapter`, the recorded
    ///   temperature is exactly that — proving both `present_temp_c` **and** the
    ///   unit's `chapter` (the epoch key) reach the provider through the payload.
    #[test]
    fn deposit_deep_history_records_the_paleo_temperature_providers_answer() {
        fn present_and_chapter(u: PaleoUnit) -> f64 {
            u.present_temp_c * 100.0 + f64::from(u.chapter)
        }

        let units = one_mineral_unit();
        let geo = vanilla();

        // Identity: the recorded temperature is the present-day column temp.
        let mut id_ctx = ctx_over(&units, &geo, Providers::default(), 9.0);
        assert_eq!(deposit_deep_history(&mut id_ctx), 5.0);
        assert_eq!(
            id_ctx.strata.events.len(),
            1,
            "the mineral unit must deposit exactly one event"
        );
        assert_eq!(
            id_ctx.strata.events[0].temp_c, 9.0_f32,
            "the identity records the present-day column temperature"
        );

        // Swapped: the recorded temperature is the provider's answer, computed
        // from present_temp_c (9.0) and chapter (2) → 902.0.
        let mut hot_ctx = ctx_over(
            &units,
            &geo,
            Providers {
                paleo_temperature: Some(present_and_chapter),
                ..Providers::default()
            },
            9.0,
        );
        deposit_deep_history(&mut hot_ctx);
        assert_eq!(
            hot_ctx.strata.events[0].temp_c, 902.0_f32,
            "the swapped provider's answer, keyed by present temp and chapter, is \
             what the event records — the seam is consulted through the real fn"
        );
    }
}
