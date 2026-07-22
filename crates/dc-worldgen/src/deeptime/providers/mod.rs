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
//! [`Providers`] is **plain function pointers in `Option`s**, the same
//! discipline as [`crate::pipeline::PassBody`]: deterministic, no captured
//! state, no closures, no trait objects, no interior mutability. It is resolved
//! **once at world build** and carried in
//! [`DeepConfig`](super::grid::DeepConfig) alongside the rest of the run's
//! configuration, so it threads to the sim through the path
//! `production_config_with` → `build_field_with` → `PregenCtx` →
//! `Pregen::run_with` that `DeepOverrides` already proved.
//!
//! **`None` *is* the identity.** A slot holds `Some(f)` when an heir supplied
//! `f`, and `None` when nobody has — in which case the slot's accessor calls
//! the slot's own identity function. Absence is therefore a *shape* of the
//! struct, not a property inferred by comparing the slot against a reference
//! value. That distinction is the whole reason this file looks the way it does;
//! see [`Providers::non_identity_slots`].
//!
//! This is **not** a registry, a plugin loader, or a declaration/validation
//! system. The slots are named fields, and adding one is
//! a compile error at every site that has to answer for it — which is the same
//! structural guarantee [`Agent`](super::lithology::Agent) relies on. The
//! general mechanism is deliberately deferred until enough seams exist to design
//! it *from* rather than *for*.
//!
//! ## The file layout, and why the grouping is load-bearing
//!
//! One module per slot — [`outcrop_at`], [`wave_energy`], [`parent_p`],
//! [`depth_to_water`], [`burial_temp_c`] — each holding that slot's payload struct, its identity
//! function, and its unit tests. This module holds only what is genuinely
//! *about the set*: the [`Providers`] struct, its identity [`Default`], and the
//! [`Slot`] enumeration.
//!
//! That split exists for a mechanical reason discovered by doing it wrong: two
//! agents converting two different seams both had to edit one 440-line file, and
//! the file became the serialization point for work that is otherwise entirely
//! independent. Conversions are meant to be concurrent. So:
//!
//! - a new slot adds a **new file**, which cannot conflict with anything;
//! - the edits it must make *here* are grouped by **owing system** — hydrology,
//!   ecology, materials, structural — the same four buckets the 34-seam
//!   inventory uses, so two conversions insert at two different points and git
//!   merges them without a human.
//!
//! A slot is filed under the system that will **answer** it, not the one that
//! asks. `parent_p` is consumed by the biotic layer and filed under
//! *materials*, because parent-material petrology is what will supply it. The
//! grouping is a map of *who owes what*, which is the whole point of a seam.
//!
//! `burial_temp_c` is the sharpest case so far, and it is why no *diagenesis*
//! group was added for it (2026-07-22). Diagenesis **asks** the question —
//! coalification is the consumer — but a geotherm is a property of the crust,
//! and the crust is tectonics' to describe: the heir will read
//! [`t_crust`](field@super::grid::DeepGrid::t_crust), a tectonics plane, for the heat
//! flow it needs. A *diagenesis* bucket would have been a group named after an
//! asker, which is precisely the distinction the four buckets exist to hold.
//!
//! ## Two granularities, and why both are here
//!
//! The slice converts one seam of each kind on purpose, because the hot loop
//! decides the shape:
//!
//! | slot | granularity | why |
//! |---|---|---|
//! | [`Providers::outcrop_at`](field@Providers::outcrop_at) | **value-level** — called per cell per epoch | it was *already* a function call ([`exposed_litho`](super::lithology::exposed_litho)); a pointer indirection replaces a direct call, and nothing else changes |
//! | [`Providers::outcrop_shares`](field@Providers::outcrop_shares) | **value-level** — called per cell per epoch | the quantity `outcrop_at`'s verdict is the argmax of; erosion reads it for rates and blends the table by share (journal/0072). Same walk, same granularity, same heir — a pinned pair with `outcrop_at` |
//! | [`Providers::wave_energy`](field@Providers::wave_energy) | **value-level** — called per shore cell per epoch | the shore band is a thin fraction of the grid, and the heir's answer genuinely varies per cell per stand |
//! | [`Providers::parent_p`](field@Providers::parent_p) | **pass-level** — called `n` times *total*, at [`BioticSim::new`](super::biotic::BioticSim::new) | the value is a property of the parent material, constant over the run; materializing it once as a plane keeps the epoch loop a plain indexed read |
//! | [`Providers::depth_to_water`](field@Providers::depth_to_water) | **pass-level** — called **once per epoch**, at [`BioticSim::step`](super::biotic::BioticSim::step) | the water table moves with the surface, so it cannot be materialized once for the run like `parent_p`; but the heir is a *field* solved over a neighbourhood, so it cannot be a per-cell call either |
//! | [`Providers::burial_temp_c`](field@Providers::burial_temp_c) | **value-level** — called per *candidate unit*, at [`BioticSim::finalize`](super::biotic::BioticSim::finalize) | there is no loop to be hot: finalize runs **once at the end of the run**, and the answer varies per unit because burial depth does. A plane cannot hold it — the question is asked of a *unit*, and a column has as many units as its history had events |
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
//! [`Providers::default()`] is the identity set — every slot `None`, so every
//! slot's accessor runs exactly the computation that was inlined at its call
//! site before the seam existed, and a default-provider world is bit-for-bit
//! the pre-seam world. That is asserted against goldens captured from pre-slice
//! `main` in `tests/providers_golden.rs`, not against a post-change
//! self-comparison.

pub mod burial_temp_c;
pub mod depth_to_water;
pub mod outcrop_at;
pub mod outcrop_shares;
pub mod parent_p;
pub mod wave_energy;

pub use burial_temp_c::{BurialColumn, BuriedUnit, identity_burial_temp_c};
pub use depth_to_water::{WaterPass, identity_depth_to_water, identity_wet_index, wet_at};
pub use outcrop_at::identity_outcrop_at;
pub use outcrop_shares::identity_outcrop_shares;
pub use parent_p::{ParentCell, identity_parent_p};
pub use wave_energy::{WaveCell, identity_wave_energy};

use super::lithology::Litho;
use super::recorder::DepUnit;

/// The near-surface window's per-[`Litho`] share vector — the payload the
/// `outcrop_shares` seam answers in (and the future `Interpolable` sample type of
/// `CoarseField`). Named so the `Option<fn>` slot type stays legible.
pub type WindowShares = [f64; Litho::COUNT];

/// The resolved provider set for one world, fixed at world creation.
///
/// `Option<fn>` per slot — deterministic, no captured state, `Copy`, trivially
/// carried in [`DeepConfig`](super::grid::DeepConfig). Each slot's doc names the
/// **question**, the **heir** that will answer it, and the **identity value**
/// that stands in until then.
///
/// **`None` means "no heir yet"** and routes the slot's accessor to its identity
/// function; `Some(f)` means `f` was supplied. Never read a field directly to
/// *call* it — use the accessor of the same name
/// ([`Providers::outcrop_at`](Self::outcrop_at()) and friends), which is the one
/// place the `None`→identity dispatch happens for that slot.
///
/// **The content set is frozen at world creation** (ARCHITECTURE.md, DECIDED
/// 2026-07-22): these are generation-affecting, so a world's provider set is
/// part of its identity and cannot be swapped on an existing world.
///
/// Fields are grouped by the system that **owes** the answer. Add a slot inside
/// its group; see the module docs for why that is not decoration.
#[derive(Clone, Copy, Debug, Default)]
pub struct Providers {
    // ───────────────────────────── hydrology ─────────────────────────────
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
    pub wave_energy: Option<fn(WaveCell) -> f64>,

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
    ///   [`Providers::parent_p`](field@Providers::parent_p): the table follows the surface, and the surface
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
    pub depth_to_water: Option<fn(WaterPass<'_>, &mut Vec<f32>)>,

    // ────────────────────────────── ecology ──────────────────────────────
    // (no slots yet — the 34-seam inventory says ecology owns 9. Insert here.)

    // ───────────────────────────── materials ─────────────────────────────
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
    pub parent_p: Option<fn(ParentCell) -> f64>,

    // ───────────────────────────── structural ────────────────────────────
    /// **Which rock is outcropping at this cell?**
    ///
    /// - *Identity:* [`identity_outcrop_at`] — the lithology dominating the
    ///   record's topmost
    ///   [`OUTCROP_DOMINANCE_WINDOW_M`](super::lithology::OUTCROP_DOMINANCE_WINDOW_M)
    ///   (0.9 m, one collapse voxel). Walks units down from the surface,
    ///   accumulating thickness per lithology, and returns the greatest;
    ///   [`Litho::Basement`] fills any deficit below a short record. A bed too
    ///   thin to fill the window cannot define the cell's rock — the thickness
    ///   rule that replaced a name-keyed charcoal carve-out (journal/0068).
    /// - *Heir:* **structural deformation** (the layer-cake / dip-fold term,
    ///   tectonics.md § 8 — per-unit dip re-derived analytically from the chapter
    ///   table at collapse resolution). Once beds dip, which units lie in the
    ///   near-surface window at a cell is a function of the fold/fault field and
    ///   the erosion surface, not of stacking order. `lithology.rs` already says
    ///   so in prose: *"This is the one function structural deformation will
    ///   change… every other part of this module carries over unaltered."* The
    ///   seam was pre-identified by its own author; this makes it a socket instead
    ///   of a sentence.
    /// - *Granularity:* value-level, per cell per epoch. It was already a call,
    ///   so the seam costs one indirection and no new work.
    pub outcrop_at: Option<fn(&[DepUnit]) -> Litho>,

    /// **How much of each rock fills the near-surface window at this cell?**
    ///
    /// - *Identity:* [`identity_outcrop_shares`] — the per-[`Litho`] thickness
    ///   *shares* of the topmost
    ///   [`OUTCROP_DOMINANCE_WINDOW_M`](super::lithology::OUTCROP_DOMINANCE_WINDOW_M),
    ///   summing to `1.0` (deficit below a short record → [`Litho::Basement`]).
    ///   This is the *quantity*
    ///   [`outcrop_at`](field@Providers::outcrop_at)'s verdict is the argmax of —
    ///   the two are one [`window_walk`](super::lithology::exposed_litho), and
    ///   erosion reads this one for its **rates** (blend the susceptibility table
    ///   by share, journal/0072) rather than collapsing to a label and stepping
    ///   the rate at the plurality crossover (the S-4 flag, walk-0071).
    /// - *Heir:* **structural deformation** — the *same* heir as
    ///   [`outcrop_at`](field@Providers::outcrop_at), and a **pinned pair** with
    ///   it: once beds dip, which units lie in the near-surface window (and how
    ///   much of each) is a function of the fold/fault field, so the heir supplies
    ///   the dipped shares *here* and the verdict slot must stay their argmax.
    ///   Seaming the quantity (not the verdict) is S-5's corollary — the erosion
    ///   rate field then dips with the beds, not just the outcrop map.
    /// - *Granularity:* value-level, per cell per epoch — it rides the same walk
    ///   the outcrop verdict already did, so no new work beyond the blend.
    pub outcrop_shares: Option<fn(&[DepUnit]) -> WindowShares>,

    /// **What temperature has this buried unit seen?**
    ///
    /// - *Identity:* [`identity_burial_temp_c`] — a **degenerate geotherm**:
    ///   0 °C at the surface, a gradient of exactly 1 °C/m, so the answer is
    ///   numerically the unit's overburden in metres and
    ///   `t >= `[`COAL_ONSET_C`](super::COAL_ONSET_C) is bit-for-bit the
    ///   pre-seam `overburden_m >= `[`COAL_BURIAL_M`](super::COAL_BURIAL_M)
    ///   (8.0 m). The gradient is 40× Earth's; it is arithmetic, not a claim
    ///   about the planet, and it is written down rather than hidden.
    /// - *Heir:* **a geothermal gradient.** There is no temperature anywhere in
    ///   this sim except surface air (`climate::air_temp_c`), which is why
    ///   coalification currently rides a *depth* threshold calibrated to this
    ///   record's own burial distribution rather than to Earth's 10²–10³ m
    ///   (`stubs.md` § 14, journal/0066). The heir supplies `T(z)` per column —
    ///   surface temperature plus a gradient set by crustal heat flow, for which
    ///   [`t_crust`](field@super::grid::DeepGrid::t_crust) is already populated
    ///   and read by nothing (`spines.md` § 3). With one, `COAL_ONSET_C` becomes
    ///   a real onset temperature, the 8 m calibration dies with the identity,
    ///   and the single `Coal` facies can split by **rank** —
    ///   lignite/sub-bituminous/bituminous/anthracite — which is what
    ///   `CLASS_ORGANIC_COAL`'s *"the depth axis is the rank axis"* contract has
    ///   been waiting for.
    /// - *Why a temperature and not a `is_coalified` predicate:* S-8, *one
    ///   quantity, many regimes*. Coal rank and metamorphic grade are the same
    ///   ladder at different heights; a predicate answers one rung and composes
    ///   with none. See [`burial_temp_c`]'s module docs for the full argument.
    /// - *Granularity:* value-level, per candidate unit, at
    ///   [`BioticSim::finalize`](super::biotic::BioticSim::finalize) — which runs
    ///   **once per run**, so the hot-loop rule has nothing to say here.
    pub burial_temp_c: Option<fn(BuriedUnit) -> f64>,
}

/// One provider slot, by name — the vocabulary a world's manifest needs.
///
/// This exists because "is the whole set the identity set?" is not the question
/// a save file asks. Under the frozen-content-set rule (ARCHITECTURE.md, DECIDED
/// 2026-07-22) a world records **which** providers were resolved and which fell
/// back, so that loading it can refuse when a generational provider has gone
/// missing. A bool cannot say that; [`Providers::non_identity_slots`] can, and
/// this enum is the alphabet it answers in.
///
/// Grouped by owing system, in the same order as [`Providers`]'s fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Slot {
    // hydrology
    WaveEnergy,
    DepthToWater,
    // ecology — none yet
    // materials
    ParentP,
    // structural
    OutcropAt,
    OutcropShares,
    BurialTempC,
}

impl Slot {
    /// Every slot, in field order. Iterating this is how the set-level
    /// operations stay exhaustive without a registry.
    pub const ALL: &'static [Slot] = &[
        // hydrology
        Slot::WaveEnergy,
        Slot::DepthToWater,
        // ecology — none yet
        // materials
        Slot::ParentP,
        // structural
        Slot::OutcropAt,
        Slot::OutcropShares,
        Slot::BurialTempC,
    ];

    /// The slot's field name, verbatim — the token a manifest stores and a log
    /// line prints. Deliberately the Rust identifier and not prose, so a
    /// recorded name and a grep agree.
    pub fn name(self) -> &'static str {
        match self {
            // hydrology
            Slot::WaveEnergy => "wave_energy",
            Slot::DepthToWater => "depth_to_water",
            // ecology — none yet
            // materials
            Slot::ParentP => "parent_p",
            // structural
            Slot::OutcropAt => "outcrop_at",
            Slot::OutcropShares => "outcrop_shares",
            Slot::BurialTempC => "burial_temp_c",
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Providers {
    // ─────────────────────────── the dispatch points ──────────────────────────
    //
    // One accessor per slot, sharing the slot's name. `x.wave_energy(cell)` is a
    // method call; `x.wave_energy` is the field. Call sites use the former and
    // never see the `Option` — the `None`→identity choice is made here, once per
    // slot, and nowhere else.

    /// Ask the [`wave_energy`](field@Self::wave_energy) slot, falling through to
    /// [`identity_wave_energy`] when no heir has supplied it.
    #[inline]
    pub fn wave_energy(&self, cell: WaveCell) -> f64 {
        match self.wave_energy {
            Some(f) => f(cell),
            None => identity_wave_energy(cell),
        }
    }

    /// Ask the [`depth_to_water`](field@Self::depth_to_water) slot, falling through to
    /// [`identity_depth_to_water`] when no heir has supplied it.
    #[inline]
    pub fn depth_to_water(&self, pass: WaterPass<'_>, out: &mut Vec<f32>) {
        match self.depth_to_water {
            Some(f) => f(pass, out),
            None => identity_depth_to_water(pass, out),
        }
    }

    /// Ask the [`parent_p`](field@Self::parent_p) slot, falling through to
    /// [`identity_parent_p`] when no heir has supplied it.
    #[inline]
    pub fn parent_p(&self, cell: ParentCell) -> f64 {
        match self.parent_p {
            Some(f) => f(cell),
            None => identity_parent_p(cell),
        }
    }

    /// Ask the [`outcrop_at`](field@Self::outcrop_at) slot, falling through to
    /// [`identity_outcrop_at`] when no heir has supplied it.
    #[inline]
    pub fn outcrop_at(&self, units: &[DepUnit]) -> Litho {
        match self.outcrop_at {
            Some(f) => f(units),
            None => identity_outcrop_at(units),
        }
    }

    /// Ask the [`outcrop_shares`](field@Self::outcrop_shares) slot, falling through
    /// to [`identity_outcrop_shares`] when no heir has supplied it.
    #[inline]
    pub fn outcrop_shares(&self, units: &[DepUnit]) -> WindowShares {
        match self.outcrop_shares {
            Some(f) => f(units),
            None => identity_outcrop_shares(units),
        }
    }

    /// Ask the [`burial_temp_c`](field@Self::burial_temp_c) slot, falling through
    /// to [`identity_burial_temp_c`] when no heir has supplied it.
    #[inline]
    pub fn burial_temp_c(&self, unit: BuriedUnit) -> f64 {
        match self.burial_temp_c {
            Some(f) => f(unit),
            None => identity_burial_temp_c(unit),
        }
    }

    // ──────────────────────────── the identity report ─────────────────────────

    /// **Is this slot supplied by an heir?** — i.e. does it hold `Some`.
    ///
    /// A field check, exhaustively matched, so adding a slot without answering
    /// for it here is a compile error.
    pub fn is_supplied(&self, slot: Slot) -> bool {
        match slot {
            // hydrology
            Slot::WaveEnergy => self.wave_energy.is_some(),
            Slot::DepthToWater => self.depth_to_water.is_some(),
            // ecology — none yet
            // materials
            Slot::ParentP => self.parent_p.is_some(),
            // structural
            Slot::OutcropAt => self.outcrop_at.is_some(),
            Slot::OutcropShares => self.outcrop_shares.is_some(),
            Slot::BurialTempC => self.burial_temp_c.is_some(),
        }
    }

    /// **Which slots have been supplied by an heir** — in field order, empty
    /// when this world generates exactly as it would have before the seams
    /// existed.
    ///
    /// This is the reportable form, and the one journal/0060 asked for: at ten
    /// slots the useful sentence is not "the set is/isn't pristine" but *"this
    /// world was generated with `depth_to_water` and `parent_p` supplied"* —
    /// which is what the manifest must record for the frozen-set hard refusal to
    /// have anything to check against.
    ///
    /// **It cannot mis-report.** It reads which fields are `Some`. It was once a
    /// comparison of `fn` addresses against `default()`'s, and Rust guarantees
    /// `fn`-pointer address uniqueness in *neither* direction: identical-code
    /// folding can merge two functions onto one address, and an `#[inline]`
    /// function can be instantiated per codegen unit at several. The second of
    /// those actually fired, on the *default* set (corrections #32). No address
    /// is taken here any more, so neither failure mode has anything to act on.
    pub fn non_identity_slots(&self) -> Vec<Slot> {
        Slot::ALL
            .iter()
            .copied()
            .filter(|&s| self.is_supplied(s))
            .collect()
    }

    /// True when no slot has been supplied — the bool convenience over
    /// [`Providers::non_identity_slots`], kept because most call sites (tests,
    /// assertions) only want the yes/no.
    pub fn is_identity(&self) -> bool {
        self.non_identity_slots().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_set_is_the_identity_set() {
        let p = Providers::default();
        assert!(
            p.is_identity(),
            "the default set reported non-identity slots: {:?}",
            p.non_identity_slots()
        );
    }

    #[test]
    fn a_swapped_slot_is_not_the_identity_set() {
        fn calm(_: WaveCell) -> f64 {
            0.0
        }
        let p = Providers {
            wave_energy: Some(calm),
            ..Providers::default()
        };
        assert!(!p.is_identity());
    }

    /// **The `None` path calls the identity, with the same arguments.** This is
    /// the byte-identity claim stated at the dispatch point rather than only at
    /// the far end of a 17-second world build: for each slot, the accessor on a
    /// default set and the identity function itself must return the same bits.
    #[test]
    fn the_none_path_is_the_identity_function() {
        let p = Providers::default();
        for base_rate in [0.0, 0.05, 1.0, 12.5] {
            let c = WaveCell {
                index: 7,
                gx: 3,
                gy: 4,
                base_rate,
            };
            assert_eq!(
                p.wave_energy(c).to_bits(),
                identity_wave_energy(c).to_bits()
            );
        }
        for index in [0usize, 1, 999] {
            let c = ParentCell {
                index,
                gx: index % 32,
                gy: index / 32,
            };
            assert_eq!(p.parent_p(c).to_bits(), identity_parent_p(c).to_bits());
        }
        assert_eq!(p.outcrop_at(&[]), identity_outcrop_at(&[]));
        assert_eq!(p.outcrop_shares(&[]), identity_outcrop_shares(&[]));
        for overburden_m in [0.0, 7.999_999_999, 8.0, 512.0] {
            let u = BuriedUnit {
                index: 5,
                gx: 5,
                gy: 0,
                surface_temp_c: 11.5,
                overburden_m,
                thickness_m: 0.4,
            };
            assert_eq!(
                p.burial_temp_c(u).to_bits(),
                identity_burial_temp_c(u).to_bits()
            );
        }

        let (precip, r, h) = (vec![0.4f32; 9], vec![25.0f64; 9], vec![2.0f64; 9]);
        let (area, recv, filled) = (vec![12.0f64; 9], vec![-1i32; 9], vec![27.5f64; 9]);
        let pass = || WaterPass {
            w: 3,
            epoch: 0,
            precip: &precip,
            r: &r,
            h: &h,
            area: &area,
            recv: &recv,
            filled: &filled,
        };
        let (mut via_slot, mut direct) = (vec![0.5f32; 9], vec![0.5f32; 9]);
        p.depth_to_water(pass(), &mut via_slot);
        identity_depth_to_water(pass(), &mut direct);
        assert_eq!(via_slot, direct);
    }

    /// **`Some(identity)` is a resolution, not an absence.** Handing a slot the
    /// very function it would have fallen back to still reports the slot as
    /// supplied — the report answers *"did an heir answer this question?"*, not
    /// *"does the answer happen to equal the old one?"*. Under the address
    /// comparison those two questions were conflated; they are different
    /// questions, and only the first is decidable.
    #[test]
    fn explicitly_supplying_the_identity_function_still_counts_as_supplied() {
        let p = Providers {
            outcrop_at: Some(identity_outcrop_at),
            ..Providers::default()
        };
        assert_eq!(p.non_identity_slots(), vec![Slot::OutcropAt]);
        // …and it still generates identically, because it is the same function.
        assert_eq!(p.outcrop_at(&[]), identity_outcrop_at(&[]));
    }

    /// The reshaped report *names* the swapped slots, and names only those —
    /// the sentence a manifest records.
    #[test]
    fn the_report_names_exactly_the_swapped_slots() {
        fn calm(_: WaveCell) -> f64 {
            0.0
        }
        fn drowned(pass: WaterPass<'_>, out: &mut Vec<f32>) {
            out.clear();
            out.resize(pass.w * pass.w, 1.0);
        }
        let p = Providers {
            wave_energy: Some(calm),
            depth_to_water: Some(drowned),
            ..Providers::default()
        };
        assert_eq!(
            p.non_identity_slots(),
            vec![Slot::WaveEnergy, Slot::DepthToWater],
            "the report must list the swapped slots, in field order"
        );
        assert_eq!(
            p.non_identity_slots()
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>(),
            vec!["wave_energy", "depth_to_water"],
        );
    }

    /// Every slot is enumerable and every name is distinct — the property that
    /// makes `ALL` safe to iterate as "the whole set".
    #[test]
    fn every_slot_is_enumerated_once_with_a_distinct_name() {
        let mut names: Vec<&str> = Slot::ALL.iter().map(|s| s.name()).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate slot name in Slot::ALL");
        let mut slots = Slot::ALL.to_vec();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(slots.len(), count, "duplicate variant in Slot::ALL");
        assert_eq!(format!("{}", Slot::OutcropAt), "outcrop_at");
    }
}
