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
//! (`outcrop_at`, `wave_energy`, `parent_p`), **names its heir** — the system
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
//! system. There are three slots, they are named fields, and adding a fourth is
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
//!
//! **A provider must never be called inside a hot loop to answer a question that
//! does not change inside that loop.** `parent_p` is in this slice specifically
//! to make that rule concrete rather than aspirational.
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
}

impl Default for Providers {
    /// The identity set: bit-for-bit the pre-seam world.
    fn default() -> Self {
        Self {
            outcrop_at: identity_outcrop_at,
            wave_energy: identity_wave_energy,
            parent_p: identity_parent_p,
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
