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
//! ## The file layout, and why the grouping is load-bearing
//!
//! One module per slot — [`outcrop_at`], [`wave_energy`], [`parent_p`],
//! [`depth_to_water`] — each holding that slot's payload struct, its identity
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
//! `tests/providers_golden.rs`, not against a post-change self-comparison.

pub mod depth_to_water;
pub mod outcrop_at;
pub mod parent_p;
pub mod wave_energy;

pub use depth_to_water::{WaterPass, identity_depth_to_water, identity_wet_index, wet_at};
pub use outcrop_at::identity_outcrop_at;
pub use parent_p::{ParentCell, identity_parent_p};
pub use wave_energy::{WaveCell, identity_wave_energy};

use super::lithology::Litho;
use super::recorder::DepUnit;

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
///
/// Fields are grouped by the system that **owes** the answer. Add a slot inside
/// its group; see the module docs for why that is not decoration.
#[derive(Clone, Copy, Debug)]
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
    pub wave_energy: fn(WaveCell) -> f64,

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
    pub parent_p: fn(ParentCell) -> f64,

    // ───────────────────────────── structural ────────────────────────────
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
}

impl Default for Providers {
    /// The identity set: bit-for-bit the pre-seam world.
    fn default() -> Self {
        Self {
            // hydrology
            wave_energy: identity_wave_energy,
            depth_to_water: identity_depth_to_water,
            // ecology — none yet
            // materials
            parent_p: identity_parent_p,
            // structural
            outcrop_at: identity_outcrop_at,
        }
    }
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
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Providers {
    /// The address held in one slot, as an integer.
    ///
    /// Cast to `usize` rather than compared with `==` on the pointers, so
    /// clippy's `fn_address_comparisons` has nothing to object to.
    fn address(&self, slot: Slot) -> usize {
        match slot {
            // hydrology
            Slot::WaveEnergy => self.wave_energy as usize,
            Slot::DepthToWater => self.depth_to_water as usize,
            // ecology — none yet
            // materials
            Slot::ParentP => self.parent_p as usize,
            // structural
            Slot::OutcropAt => self.outcrop_at as usize,
        }
    }

    /// **Which slots do not hold their identity function** — in field order,
    /// empty when this world generates exactly as it would have before the seams
    /// existed.
    ///
    /// This is the reportable form, and the one journal/0060 asked for: at ten
    /// slots the useful sentence is not "the set is/isn't pristine" but *"this
    /// world was generated with `depth_to_water` and `parent_p` supplied"* —
    /// which is what the manifest must record for the frozen-set hard refusal to
    /// have anything to check against.
    ///
    /// Two distinct functions with identical bodies may share an address after
    /// identical-code-folding, so a custom provider that is byte-identical to
    /// the identity one can be reported as identity — the harmless direction.
    pub fn non_identity_slots(&self) -> Vec<Slot> {
        // Compared against `default()`'s *fields*, which are already fn
        // pointers: casting a fn item straight to an integer is what clippy's
        // `fn_to_numeric_cast` objects to.
        let id = Self::default();
        Slot::ALL
            .iter()
            .copied()
            .filter(|&s| self.address(s) != id.address(s))
            .collect()
    }

    /// True when every slot still holds its identity function — the bool
    /// convenience over [`Providers::non_identity_slots`], kept because most
    /// call sites (tests, assertions) only want the yes/no.
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
            wave_energy: calm,
            ..Providers::default()
        };
        assert!(!p.is_identity());
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
            wave_energy: calm,
            depth_to_water: drowned,
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
