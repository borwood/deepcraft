//! **Provider seams** — the points where the deep-time sim asks a question that
//! an *unbuilt* system is eventually going to answer, and answers it today with
//! a constant.
//!
//! ## The problem this exists to solve
//!
//! `docs/ARCHITECTURE.md` § *"A summary is not an authority"* names the failure
//! mode: a cheap stand-in, written because the real authority does not exist
//! yet, hardens into the definition of the thing it stood in for. **A stand-in
//! becomes the definition unless something stops it.** Four instances of that
//! were found in a single audit; none of them were caught by `stubs.md`, because
//! *a leaked requirement looks like working code that passes tests*.
//!
//! A provider slot is what stops it. The slot **names the question**
//! (`outcrop_at`, `wave_energy`, `parent_p`, `depth_to_water`), **names its heir** — the system
//! expected to supply the real answer — and **names its identity value**, the
//! constant it holds until the heir arrives. The constant is still there, but it
//! can no longer masquerade as the rule: it is visibly a default sitting in a
//! socket that has a labelled owner.
//!
//! ## The mechanism, and its deliberate limits
//!
//! [`Providers`] is **plain function pointers**, the same discipline as
//! [`crate::pipeline::PassBody`]: deterministic, no captured state, no closures,
//! no trait objects, no interior mutability. It is resolved **once at world
//! build** and carried in [`DeepConfig`](super::grid::DeepConfig) alongside the
//! rest of the run's configuration, so it threads to the sim through the path
//! `production_config_with` → `build_field_with` → `PregenCtx` →
//! `Pregen::run_with` that `DeepOverrides` already proved.
//!
//! This is **not** a registry, a plugin loader, or a declaration/validation
//! system. There are four slots, they are named fields, and adding a fifth is
//! a compile error at every site that has to answer for it — which is the same
//! structural guarantee [`Agent`](super::lithology::Agent) relies on. The
//! general mechanism is deliberately deferred until enough seams exist to design
//! it *from* rather than *for*.
//!
//! ## Two granularities, and why both are here
//!
//! The slice converts one seam of each kind on purpose, because the hot loop
//! decides the shape:
//!
//! | slot | granularity | why |
//! |---|---|---|
//! | [`Providers::outcrop_at`] | **value-level** — called per cell per epoch | it was *already* a function call ([`exposed_litho`](super::lithology::exposed_litho)); a pointer indirection replaces a direct call, and nothing else changes |
//! | [`Providers::wave_energy`] | **value-level** — called per shore cell per epoch | the shore band is a thin fraction of the grid, and the heir's answer genuinely varies per cell per stand |
//! | [`Providers::parent_p`] | **pass-level** — called `n` times *total*, at [`BioticSim::new`](super::biotic::BioticSim::new) | the value is a property of the parent material, constant over the run; materializing it once as a plane keeps the epoch loop a plain indexed read |
//! | [`Providers::depth_to_water`] | **pass-level** — called **once per epoch**, at [`BioticSim::step`](super::biotic::BioticSim::step) | the water table moves with the surface, so it cannot be materialized once for the run like `parent_p`; but the heir is a *field* solved over a neighbourhood, so it cannot be a per-cell call either |
//!
//! **A provider must never be called inside a hot loop to answer a question that
//! does not change inside that loop.** `parent_p` is in this slice specifically
//! to make that rule concrete rather than aspirational.
//!
//! And the sharper rule the first slice earned: **granularity follows the
//! HEIR, not the call site.** `depth_to_water` is consumed at four per-cell
//! thresholds inside the biotic loop, so the call site says "value-level"; the
//! heir is a saturation field over the drainage lattice, which says "plane". The
//! heir wins. Designing against the call site is how `wave_energy` acquired a
//! payload struct its own heir cannot fill.
//!
//! ## Byte-identity
//!
//! [`Providers::default()`] is the identity set: every slot holds exactly the
//! computation that was inlined at its call site before the seam existed, so a
//! default-provider world is bit-for-bit the pre-seam world. That is asserted
//! against goldens captured from pre-slice `main` in
//! `tests/providers.rs`, not against a post-change self-comparison.

use super::lithology::Litho;
use super::recorder::DepUnit;

/// The cell the littoral agent is about to attack, as much of it as a provider
/// is allowed to see: no `&DeepGrid`, because a provider captures nothing and
/// borrows nothing that would make it non-trivially registrable.
///
/// `base_rate` is [`DeepConfig::wave_erosion`](super::grid::DeepConfig::wave_erosion)
/// — the global constant the identity provider hands straight back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveCell {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
    /// The configured global littoral rate (m/epoch at the waterline).
    pub base_rate: f64,
}

/// Everything the water-table pass is allowed to read: the epoch's
/// post-erosion, post-routing state of the deep grid.
///
/// This is a **pass-level** payload — a bundle of *planes*, not a cell — because
/// the heir ([`Providers::depth_to_water`]) is a field and not a per-cell
/// answer. A water table is a solution over a neighbourhood: it needs the
/// drainage network and the filled surface to know where water *collects*, which
/// a per-cell payload structurally cannot carry (journal/0060's lesson —
/// **granularity follows the heir, not the call site**).
///
/// Borrowed slices rather than `&DeepGrid`, for the same reason the value-level
/// payloads are `Copy` structs: the provider sees the inputs it is contracted to
/// read and nothing else, so its declared reads and its actual reads are the
/// same list (ARCHITECTURE.md § *Provider seams*, "effective reads").
#[derive(Clone, Copy, Debug)]
pub struct WaterPass<'a> {
    /// Grid width; the planes are `w * w`, row-major.
    pub w: usize,
    /// The epoch about to be stepped. The water table moves with the surface,
    /// so unlike [`ParentCell`] this pass is re-run every epoch.
    pub epoch: u32,
    /// Climate moisture, 0..1 (the pregen precipitation plane).
    pub precip: &'a [f32],
    /// Bedrock elevation (m). Surface is `r[i] + h[i]`.
    pub r: &'a [f64],
    /// Regolith thickness (m).
    pub h: &'a [f64],
    /// Accumulated D8 drainage area (cell units) from this epoch's routing.
    pub area: &'a [f64],
    /// D8 receiver of each cell (`-1` = sink) — the drainage network.
    pub recv: &'a [i32],
    /// The depression-filled surface from this epoch's flood. A cell whose fill
    /// sits above its own surface is under standing water.
    pub filled: &'a [f64],
}

/// The cell whose parent material is being characterized, at the one moment the
/// biotic layer asks: initialization, before any epoch has run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParentCell {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
}

/// **Identity for [`Providers::outcrop_at`]**: the top of the record, or
/// [`Litho::Basement`] when the column has been stripped past its whole
/// sedimentary history. This is the pre-seam
/// [`exposed_litho`](super::lithology::exposed_litho), unchanged and re-exported
/// under the slot's name so the identity is a *thing* and not a description.
pub use super::lithology::exposed_litho as identity_outcrop_at;

/// **Identity for [`Providers::wave_energy`]**: the configured global rate,
/// handed back unchanged — the world has one wave climate everywhere.
///
/// Note the pre-existing off-switch this preserves: the littoral agent returns
/// early when the *configured* rate is `<= 0.0`, which is the byte-identity
/// escape `tests/full_agents.rs` already leans on. The provider is consulted per
/// cell only after that gate, so `wave_erosion: 0.0` still means "no littoral
/// term at all", provider or no provider.
pub fn identity_wave_energy(cell: WaveCell) -> f64 {
    cell.base_rate
}

/// **Identity for [`Providers::parent_p`]**: `1.0` everywhere — a uniform,
/// maximally phosphorus-rich parent material. This is the true identity: the
/// pre-seam code seeded every cell's rock-P pool from one constant
/// (`biotic::P_ROCK_INIT`) and capped rejuvenation at the same constant.
pub fn identity_parent_p(_cell: ParentCell) -> f64 {
    1.0
}

/// **Identity for [`Providers::depth_to_water`]**: leave the plane *empty*.
///
/// This is the `biotic` / `erodibility` / `full_agents` / `tectonic_history`
/// pattern the four deep-sim flags already prove — **empty plane + identity
/// accessor** ([`wet_at`]) — and it is what this seam conspicuously lacked. An
/// empty plane costs one `is_empty()` branch per cell and allocates nothing, so
/// "provider absent" is not merely byte-identical, it is *free*: the identity
/// world does not even build a plane it would then read back unchanged.
pub fn identity_depth_to_water(_pass: WaterPass<'_>, out: &mut Vec<f32>) {
    out.clear();
}

/// The three-term waterlogging proxy, verbatim as it stood inline in
/// [`biotic::step_cell`](super::biotic) before this seam existed: climate
/// moisture, plus a bonus for sitting near base level, plus a bonus for
/// receiving upslope drainage, clamped to 0..1.
///
/// **This is a stand-in for a water table** (`docs/design/water.md`, DECIDED
/// 2026-07-20, consequence 4: *"waterlogging becomes 'the water table is at or
/// near the surface here', read from the field"*). The three coefficients
/// (`80 m`, `0.20`; `300` cell-units, `0.15`) are a guess, not a measurement,
/// and they answer in a **dimensionless 0..1 wetness index** rather than in
/// metres below the surface — which is the units mismatch the heir has to
/// resolve. See [`Providers::depth_to_water`] for what the heir must supply.
///
/// Kept bit-for-bit: same operations, same order, same `f64 → f32` cast points.
#[inline]
pub fn identity_wet_index(moist: f32, surf: f64, area: f64) -> f32 {
    let low_bonus = ((80.0 - surf) / 80.0).clamp(0.0, 1.0) as f32 * 0.20;
    let area_bonus = (area / 300.0).min(1.0) as f32 * 0.15;
    (moist + low_bonus + area_bonus).clamp(0.0, 1.0)
}

/// The wetness at cell `i`: the materialized [`Providers::depth_to_water`] plane
/// when one exists, and the exact pre-seam expression ([`identity_wet_index`])
/// when the plane is empty.
///
/// The same shape as `erosion.rs`'s `wmult_at` / `frost_at`: an accessor whose
/// empty-slice branch *is* the identity, so the identity path cannot drift from
/// the provider path by construction — there is one call site and one branch.
#[inline]
pub fn wet_at(plane: &[f32], i: usize, moist: f32, surf: f64, area: f64) -> f32 {
    if plane.is_empty() {
        identity_wet_index(moist, surf, area)
    } else {
        plane[i]
    }
}

/// The resolved provider set for one world, fixed at world creation.
///
/// Plain `fn` pointers — deterministic, no captured state, `Copy`, trivially
/// carried in [`DeepConfig`](super::grid::DeepConfig). Each slot's doc names the
/// **question**, the **heir** that will answer it, and the **identity value**
/// that stands in until then.
///
/// **The content set is frozen at world creation** (ARCHITECTURE.md, DECIDED
/// 2026-07-22): these are generation-affecting, so a world's provider set is
/// part of its identity and cannot be swapped on an existing world.
#[derive(Clone, Copy, Debug)]
pub struct Providers {
    /// **Which rock is outcropping at this cell?**
    ///
    /// - *Identity:* [`identity_outcrop_at`] — the last unit of the record, i.e.
    ///   the record is a flat layer-cake and "exposed" means "topmost deposited".
    /// - *Heir:* **structural deformation** (the layer-cake / dip-fold term,
    ///   tectonics.md § 8 — per-unit dip re-derived analytically from the chapter
    ///   table at collapse resolution). Once beds dip, the unit outcropping at a
    ///   cell is a function of the fold/fault field and the erosion surface, not
    ///   of stacking order. `lithology.rs` already says so in prose: *"This is the
    ///   one function structural deformation will change… every other part of
    ///   this module carries over unaltered."* The seam was pre-identified by its
    ///   own author; this makes it a socket instead of a sentence.
    /// - *Granularity:* value-level, per cell per epoch. It was already a call,
    ///   so the seam costs one indirection and no new work.
    pub outcrop_at: fn(Option<&DepUnit>) -> Litho,

    /// **How hard does the sea work at this cell?**
    ///
    /// - *Identity:* [`identity_wave_energy`] — the global constant
    ///   [`DeepConfig::wave_erosion`](super::grid::DeepConfig::wave_erosion),
    ///   i.e. one wave climate for the whole planet.
    /// - *Heir:* **fetch × wind** (ROADMAP): fetch from the S11 body graph — how
    ///   much open water lies upwind of this shore, which is what actually sets
    ///   wave height — crossed with the zonal wind field the eolian agent already
    ///   reads. A lee shore inside an inland sea and a west-facing ocean coast
    ///   at 45° are the same number today, and should not be.
    /// - *Granularity:* value-level, per shore cell per epoch. Only the thin
    ///   freeboard band consults it.
    pub wave_energy: fn(WaveCell) -> f64,

    /// **How much phosphorus is in this cell's parent material?**
    ///
    /// - *Identity:* [`identity_parent_p`] — `1.0` everywhere (`P_ROCK_INIT`).
    /// - *Heir:* **parent-material petrology** — the rock-P endowment of the
    ///   lithology the soil is forming on (basalt is P-rich, quartz sand is
    ///   nearly P-free). The Walker & Syers retrogression clock the biotic layer
    ///   runs is *driven* by the size of that finite pool, so a uniform pool
    ///   means every surface of the same age retrogresses at the same rate
    ///   regardless of what it is made of. This is the ecology-side twin of the
    ///   `Litho::reference_material` stand-in in ARCHITECTURE.md's audit table.
    /// - *Granularity:* **pass-level**. Parent material does not change over the
    ///   run, so the plane is materialized once at
    ///   [`BioticSim::new`](super::biotic::BioticSim::new) and read by index
    ///   thereafter — the epoch loop never calls this pointer.
    pub parent_p: fn(ParentCell) -> f64,

    /// **How close to the surface is the water table at this cell?**
    ///
    /// - *Identity:* [`identity_depth_to_water`] — leave the plane empty, so
    ///   [`wet_at`] falls through to [`identity_wet_index`], the three-term
    ///   proxy (`moisture + low-elevation bonus + drainage-area bonus`) that
    ///   stood inline in the biotic loop. Empty plane + identity accessor: the
    ///   proven byte-identity shape the four deep-sim flags use, and the one
    ///   thing this seam had no form of at all.
    /// - *Heir:* **the hydrology field** — S11's saturation field over the
    ///   drainage-pinned lattice. `water.md` (DECIDED 2026-07-20) already
    ///   names this retirement in so many words: *"S10's waterlogging proxy has
    ///   a defined retirement: waterlogging becomes 'the water table is at or
    ///   near the surface here', read from the field. Biology reads the real
    ///   quantity; the proxy is deleted."* Water there is one conserved
    ///   quantity in two regimes, and the table is the **top of the saturated
    ///   zone** — a query over the field, not a stored plane, which is why this
    ///   slot hands the provider [`WaterPass`]'s network and filled surface
    ///   rather than a lone cell.
    /// - *Granularity:* **pass-level, once per epoch.** Not once per run like
    ///   [`Providers::parent_p`]: the table follows the surface, and the surface
    ///   is what the erosion sim is busy rewriting. Not per cell either — the
    ///   heir is a field, and journal/0060's lesson is that granularity follows
    ///   the heir rather than the call site. `BioticSim::step` materializes the
    ///   plane at the pass boundary; `step_cell` does an indexed read.
    /// - *Units, unresolved:* the identity answers a **dimensionless 0..1
    ///   wetness index** (1 = saturated at the surface). A real water table
    ///   answers **metres below the surface**. The four consumers all threshold
    ///   the index, so converting them is a behaviour change and is explicitly
    ///   not part of this slice — the slot name is `depth_to_water` because that
    ///   is the question, and the mismatch between the question and today's
    ///   answer is the seam's most useful output.
    pub depth_to_water: fn(WaterPass<'_>, &mut Vec<f32>),
}

impl Default for Providers {
    /// The identity set: bit-for-bit the pre-seam world.
    fn default() -> Self {
        Self {
            outcrop_at: identity_outcrop_at,
            wave_energy: identity_wave_energy,
            parent_p: identity_parent_p,
            depth_to_water: identity_depth_to_water,
        }
    }
}

impl Providers {
    /// True when every slot still holds its identity function — i.e. this world
    /// generates exactly as it would have before the seams existed.
    ///
    /// Compared by address (cast to `usize` rather than `==` on the pointers, so
    /// clippy's `fn_address_comparisons` has nothing to object to). Two distinct
    /// functions with identical bodies may share an address after
    /// identical-code-folding, so this can say "identity" about a custom
    /// provider that is byte-identical to the identity one — which is the
    /// harmless direction.
    pub fn is_identity(&self) -> bool {
        // Compared against `default()`'s *fields*, which are already fn
        // pointers: casting a fn item straight to an integer is what clippy's
        // `fn_to_numeric_cast` objects to.
        let id = Self::default();
        self.outcrop_at as usize == id.outcrop_at as usize
            && self.wave_energy as usize == id.wave_energy as usize
            && self.parent_p as usize == id.parent_p as usize
            && self.depth_to_water as usize == id.depth_to_water as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_set_is_the_identity_set() {
        assert!(Providers::default().is_identity());
    }

    #[test]
    fn a_swapped_slot_is_not_the_identity_set() {
        fn calm(_: WaveCell) -> f64 {
            0.0
        }
        let p = Providers {
            wave_energy: calm,
            ..Providers::default()
        };
        assert!(!p.is_identity());
    }

    #[test]
    fn the_identity_wave_energy_is_the_configured_rate() {
        for base_rate in [0.0, 0.05, 3.25] {
            let c = WaveCell {
                index: 0,
                gx: 0,
                gy: 0,
                base_rate,
            };
            assert_eq!(identity_wave_energy(c).to_bits(), base_rate.to_bits());
        }
    }

    /// The registered identity for `depth_to_water` leaves the plane empty —
    /// that *is* the identity, because an empty plane selects [`wet_at`]'s
    /// pre-seam branch. It must also clear a plane a previous epoch filled.
    #[test]
    fn the_identity_depth_to_water_leaves_the_plane_empty() {
        let (precip, r, h) = (vec![0.5f32; 4], vec![10.0f64; 4], vec![1.0f64; 4]);
        let (area, recv, filled) = (vec![7.0f64; 4], vec![-1i32; 4], vec![11.0f64; 4]);
        let pass = WaterPass {
            w: 2,
            epoch: 3,
            precip: &precip,
            r: &r,
            h: &h,
            area: &area,
            recv: &recv,
            filled: &filled,
        };
        let mut plane = vec![0.9f32; 4];
        identity_depth_to_water(pass, &mut plane);
        assert!(
            plane.is_empty(),
            "the identity must not leave a stale plane"
        );
    }

    /// The empty-plane accessor reproduces the three-term proxy exactly, and a
    /// materialized plane is read by index instead — the whole seam, in one
    /// assertion pair.
    #[test]
    fn wet_at_falls_through_to_the_proxy_only_when_the_plane_is_empty() {
        for (moist, surf, area) in [
            (0.10f32, -20.0f64, 0.0f64),
            (0.35, 40.0, 150.0),
            (0.62, 900.0, 4_000.0),
            (0.95, 0.0, 300.0),
        ] {
            let expected = identity_wet_index(moist, surf, area);
            assert_eq!(
                wet_at(&[], 0, moist, surf, area).to_bits(),
                expected.to_bits(),
                "the empty plane must be the pre-seam expression, bit for bit"
            );
            let plane = vec![0.125f32, 0.25];
            assert_eq!(wet_at(&plane, 1, moist, surf, area), 0.25);
        }
    }

    /// The proxy's shape, stated as properties rather than as its own source:
    /// clamped to 0..1, monotone down in elevation and up in drainage area.
    #[test]
    fn the_wet_index_is_clamped_and_monotone() {
        assert_eq!(identity_wet_index(1.0, -500.0, 10_000.0), 1.0);
        assert_eq!(identity_wet_index(0.0, 5_000.0, 0.0), 0.0);
        assert!(identity_wet_index(0.2, 800.0, 10.0) < identity_wet_index(0.2, 10.0, 10.0));
        assert!(identity_wet_index(0.2, 800.0, 10.0) < identity_wet_index(0.2, 800.0, 250.0));
    }

    #[test]
    fn the_identity_parent_p_is_uniform_one() {
        for index in [0usize, 1, 4_242] {
            let c = ParentCell {
                index,
                gx: index % 64,
                gy: index / 64,
            };
            assert_eq!(identity_parent_p(c), 1.0);
        }
    }
}
