//! The deep-time strata recorder: a per-cell ordered deposition log, tagged by
//! what was **measured** at the moment of deposition — never by interpretation
//! (orogeny corrections #9/#11, earth-processes.md § method).
//!
//! Adapted from [`crate::geology::StrataRec`] (the same ordered-events-with-
//! context shape, run-length merge of like units, pop-on-erosion, and the
//! `sum(thickness) == H` finalize invariant), but the deep-time recorder tags
//! by depositional *environment* rather than a resolved material member: at
//! deep-time cell resolution the readable story is the **process** (a marine
//! band, an arid fan, a humid floodplain, an erosional gap), not the mineral.
//! The material-tier member selection (the shipped `StrataRec`) still runs at
//! collapse time under the context this record hands it.
//!
//! Recording is driven by the **net** per-cell thickness change each iteration
//! (deposition when `ΔH > 0`, erosion popping history when `ΔH < 0`), tagged by
//! the environment measured that iteration. Net-per-iteration keeps the event
//! count near the number of *tag changes* over deep time (run-length merged)
//! rather than one event per iteration, and makes the finalize invariant exact
//! by construction: the record mirrors every metre that entered or left `H`.

use dc_core::materials::MaterialId;
use dc_core::materials::geology::{FormationContext, GeoClass, GeologySet};
use dc_sim::statistical::rng::Draws;

use super::geotherm::{self, BurialColumn};
use super::lithology::{Litho, litho_of_tag};

/// **Which depositor is asking** — the tag space inside the
/// [`DeepMember`](crate::draws::DeepMember) domain (`draws.rs` module docs,
/// hole 1).
///
/// Two agents that deposit into the same cell in the same chapter must not share a
/// draw: if the wave agent and the wind agent read one stream, their member
/// picks are the same number and the two beds correlate for no physical reason.
/// One constant per depositing agent is the cheap, legible answer; a `Domain` per
/// decision is the expensive one the module docs already name.
pub mod dep_tags {
    /// The erosion recorder — the fluvial load plus hillslope creep (the argmax
    /// of everything that arrived). The bulk of the record.
    pub const TRANSPORT: u64 = 0;
    /// Wind: loess and dune beds.
    pub const EOLIAN: u64 = 1;
    /// Littoral wave attack's redeposition.
    pub const WAVE: u64 = 2;
    /// The biotic layer's fire beds (charcoal).
    pub const BIOTIC: u64 = 3;
    /// Pedogenesis — [`DeepStrata::overprint_top`], which *alters* the top unit
    /// rather than stacking on it, so its identity is re-picked too.
    pub const PEDOGENIC: u64 = 4;
    /// Burial diagenesis — [`DeepStrata::promote_coal`]. The one identity event
    /// that does **not** happen at the surface, and the reason its formation
    /// context carries a real burial depth.
    pub const DIAGENESIS: u64 = 5;
}

/// **The run-wide half of deposition-time identity** — what a deep-time pass
/// carries so it can build a [`DepositCtx`] per cell (P11 slice 1).
///
/// It is `Copy` and three words wide, so it rides into a pass as a value rather
/// than as another borrow of the runner's state.
#[derive(Clone, Copy)]
pub struct MemberCtx<'a> {
    pub geology: &'a GeologySet,
    pub draws: Draws,
    pub chapter: u64,
    /// **The deep tier's classes, resolved once per run**, indexed by
    /// `Litho::index()`.
    ///
    /// `GeologySet::select` finds its class in a `BTreeMap<String, _>`, which is
    /// a handful of string comparisons — free while selection ran once per
    /// (chunk, event), and *not* free now that it runs once per deposition event
    /// per cell per epoch. The roster is fixed for a run, so the lookup is hoisted
    /// out of the loop entirely. It is the set's own `GeoClass`, not a copy of
    /// one: no second authority, just no second lookup.
    classes: [Option<&'a GeoClass>; Litho::COUNT],
}

impl<'a> MemberCtx<'a> {
    /// Open the stream for a world. The **only** place a deposition-time member
    /// draw is seeded.
    pub fn new(geology: &'a GeologySet, seed: u64, chapter: u64) -> Self {
        let mut classes = [None; Litho::COUNT];
        for l in Litho::ALL {
            classes[l.index()] = geology.class(crate::geology::deep_class_of_species(l));
        }
        Self {
            geology,
            draws: Draws::of::<crate::draws::DeepMember>(seed),
            chapter,
            classes,
        }
    }

    /// The per-cell context under an explicit formation context.
    #[inline]
    pub fn at(&self, cell: usize, form: FormationContext) -> DepositCtx<'a> {
        DepositCtx {
            geology: self.geology,
            form,
            draws: self.draws,
            cell: cell as u64,
            chapter: self.chapter,
            classes: self.classes,
        }
    }

    /// **The common case: a bed laid at the surface.** Air temperature over the
    /// current ground, the marched precipitation, and `depth_m = 0` — burial is a
    /// later fact about a bed, never a condition of its formation.
    #[inline]
    pub fn surface(
        &self,
        cell: usize,
        temp_c: f64,
        precip: f64,
        litho: Litho,
        tag: u64,
        k: u64,
    ) -> MaterialId {
        self.at(
            cell,
            FormationContext {
                temp_c,
                precip,
                depth_m: 0.0,
            },
        )
        .material_for(litho, tag, k)
    }
}

/// **The identity-setting context a depositing agent hands the recorder**
/// (P11 slice 1).
///
/// Before P11 a unit's identity was a *class* and the member was invented at
/// expression, under the chunk-centre formation context of the day the chunk was
/// generated — a context 460 m wide and hundreds of millions of years late. The
/// ruling (2026-08-01) is that fitness runs **at deposition**, so the deep sim
/// has to carry the three things fitness needs: the registered content, the
/// formation conditions of *this* geological day, and an addressed draw.
///
/// It is `Copy` and holds a borrow, so it costs nothing to hand to a per-cell
/// closure; `&GeologySet` is `Sync`, so the parallel record phase reads it
/// without a lock and stays byte-identical to the scalar one (the address is the
/// cell index, never an iteration counter).
#[derive(Clone, Copy)]
pub struct DepositCtx<'a> {
    /// The registered geology content this world was built with.
    pub geology: &'a GeologySet,
    /// Surface conditions at the cell, this epoch: the marched precipitation and
    /// the air temperature over the current surface. `depth_m` is **0** for every
    /// depositional event — a bed is laid at the surface, and burial is a later
    /// fact about it, not a condition of its formation. The one event that
    /// overrides it is [`DeepStrata::promote_coal`], which *is* a burial event.
    pub form: FormationContext,
    /// The [`DeepMember`](crate::draws::DeepMember) stream for this world.
    pub draws: Draws,
    /// Deep cell (row-major index) — the spatial half of the draw address.
    pub cell: u64,
    /// **The tectonic chapter — the temporal half of the address.**
    ///
    /// ⚠ **INTERIM SCAFFOLDING. This draw exists only until the record can answer
    /// the question without rolling for it, and both of its heirs are sequenced:**
    ///
    /// - **P11 slice 2 DID retire it for transported deposits** (2026-08-02):
    ///   identity comes from the *arriving composition term* — what the mover
    ///   actually carried — so there is nothing left to pick. The draw is reached
    ///   only where the un-carried remainder won, or on a **transformation edge**
    ///   (basement → coarse detritus, detrital organics → carbonaceous mud), where
    ///   the destination member is a fact about the site and not about the parent.
    /// - **FS-A** retires it for **weathered** material: release spectra say what
    ///   a parent rock sheds, so the product's identity is derived, not drawn.
    ///   *(Status 2026-08-02: FS-A's first slice shipped the declared spectra +
    ///   the inventory-weathering emission — `dc-core::materials::release`,
    ///   `InvCtx::release` — but this draw still runs for weathered deposits;
    ///   the record-path retirement is the wire-up behind P11 slice 3's packed
    ///   `DepUnit`, with the grain write.)*
    ///
    /// Until then a class still has to be filled, and this is the tie-break inside
    /// the fitness distribution. **Chapter-grained, not epoch-grained**, and the
    /// difference is not cosmetic: an epoch-grained roll re-rolls the tie-break
    /// every step, so a cell in a *stable* environment records an alternating
    /// stack instead of one bed — and since identity is in the merge key, that
    /// multiplies the units. Chapter-grained, **the draw is fixed while the
    /// fitness weights keep moving every epoch**, so identity changes exactly when
    /// the shifting CDF crosses the fixed draw: at a real change in conditions,
    /// which is what a bed contact is. A chapter boundary is already a time
    /// surface the record refuses to merge across.
    pub chapter: u64,
    /// The deep classes, pre-resolved — see [`MemberCtx::classes`].
    classes: [Option<&'a GeoClass>; Litho::COUNT],
}

impl DepositCtx<'_> {
    /// The material a bed of class `litho` deposits as **here, today** — fitness
    /// × normalised abundance × this event's addressed draw.
    ///
    /// `tag` names the depositor ([`dep_tags`]) and `k` separates several events
    /// from one depositor in one cell-chapter (a unit index, for instance).
    #[inline]
    pub fn material_for(&self, litho: Litho, tag: u64, k: u64) -> MaterialId {
        self.material_in(litho, &self.form, tag, k)
    }

    /// [`Self::material_for`] under an explicit formation context — for the
    /// events whose conditions are not the surface's (burial diagenesis).
    ///
    /// **The fallback is the class's reference material**, which is exactly what
    /// the record said before P11: a content set that leaves this class empty
    /// still produces a rock, and a world built with the vanilla set never
    /// reaches it (`Pipeline::check_class_satisfiability` refuses to build one
    /// that would). S-5 identity default.
    pub fn material_in(
        &self,
        litho: Litho,
        form: &FormationContext,
        tag: u64,
        k: u64,
    ) -> MaterialId {
        // INTERIM SCAFFOLDING, now HALF RETIRED — P11 slice 2 took the
        // transported deposits (2026-08-02); FS-A takes the weathered ones
        // (release spectra). What reaches here is the genuine-degeneracy
        // remainder. See `DepositCtx::chapter`. Addressed by CHAPTER, not
        // epoch: the draw is fixed while the fitness weights move, so identity
        // turns over when conditions do and not on a per-step coin.
        let u = self.draws.unit(&[tag, self.cell, self.chapter, k]);
        self.classes[litho.index()]
            .and_then(|c| self.geology.select_in(c, form, u))
            .map_or_else(|| litho.reference_material(), |(_, def)| def.material)
    }
}

/// **The mover axis** (stub #25's agent axis, funded by P11 slice 3's packed
/// `DepUnit`): which transport regime delivered a deposit, as
/// [`FlowCause`](super::flux::FlowCause)` as u8` (`0..=6`), or [`MOVER_NONE`]
/// for material made where it lies (in-place weathering, pedogenesis, primary
/// formation — nothing rode a mover).
///
/// Three bits exactly: `FlowCause` has 7 inhabitants and `MOVER_NONE` is the
/// eighth value. Kept a raw `u8` rather than an `Option<FlowCause>` because it
/// packs into the unit's bitfield and rides through the recorder's merge
/// arithmetic; [`DepUnit::mover`] is the typed read.
pub const MOVER_NONE: u8 = 7;

/// Depositional environment, measured at the event (surface vs. sea level).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DepEnv {
    /// Deposited above sea level (fluvial / colluvial / aeolian).
    Subaerial,
    /// Deposited at or below sea level (marine / lacustrine).
    Subsea,
}

/// Aridity of the depositional site, from the marched precipitation (the
/// paleoclimate axis). Meaningless subsea — normalized to `Humid` there so the
/// tag space stays small and marine bands merge regardless of overhead climate.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Aridity {
    Arid,
    Humid,
}

/// The energy band of the transporting flow at deposition (stream capacity).
/// The facies signal: high-energy proximal coarse bodies vs. low-energy distal
/// fines — the same axis the shipped clastic/placer passes read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EnergyBand {
    Low,
    Medium,
    High,
}

/// The **biotic facies** measured at deposition (S10 biotic layer). `Mineral` is
/// the always-present default: a run with the biotic layer OFF tags every unit
/// `Mineral`, which is byte-identical to the pre-S10 record (the enum is a new
/// merge-key axis whose only inhabited value is `Mineral` when biology is off,
/// so `x.deposit(mineral_tag, d)` merges exactly as before).
///
/// The inhabited values are the four read-quality target signals ecology.md § 3
/// asks the record to carry, each a *measurement* of the depositing community —
/// never an interpretation (earth-processes.md § method):
/// - [`Soil`](Self::Soil) — an organic horizon: litter outpaced erosion under a
///   living community. Buried (not the topmost unit), a `Soil`/`Peat`/`Retro`
///   unit is a **paleosol**, carrying its own at-deposition climate tag.
/// - [`Peat`](Self::Peat) — waterlogged organic accumulation that outran
///   decomposition (a swamp persisted); the proto-coal.
/// - [`Coal`](Self::Coal) — a `Peat` band buried and compacted past a threshold
///   (burial diagenesis promotes peat to coal, earth-processes.md § 5) — a
///   **coal seam** in the record.
/// - [`Charcoal`](Self::Charcoal) — a fire event: standing biomass burned, its
///   residue entering the record as a thin band (disturbance signature).
/// - [`Retro`](Self::Retro) — an organic horizon under a **retrogressive**
///   community: an ancient, phosphorus-starved surface carrying sclerophyllous
///   scrub (the Walker & Syers end state, ecology.md § 0) — community collapse
///   leaving a signature.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Biofacies {
    /// No biotic signature (default — the only value when biology is off).
    #[default]
    Mineral,
    /// Organic soil horizon (a living community's litter accumulation).
    Soil,
    /// Waterlogged organic accumulation (proto-coal).
    Peat,
    /// Buried, compacted peat — a coal seam.
    Coal,
    /// Fire-event residue band.
    Charcoal,
    /// Retrogressive (P-depleted, sclerophyllous) soil horizon.
    Retro,
}

impl Biofacies {
    /// Short two-char code for column printouts.
    pub fn code(self) -> &'static str {
        match self {
            Biofacies::Mineral => "--",
            Biofacies::Soil => "So",
            Biofacies::Peat => "Pt",
            Biofacies::Coal => "Co",
            Biofacies::Charcoal => "Ch",
            Biofacies::Retro => "Rt",
        }
    }

    /// Whether this facies is an organic horizon (i.e. a soil, in the paleosol
    /// sense): `Soil`, `Peat`, `Coal`, or `Retro`.
    pub fn is_organic(self) -> bool {
        matches!(
            self,
            Biofacies::Soil | Biofacies::Peat | Biofacies::Coal | Biofacies::Retro
        )
    }
}

/// The **aeolian facies** measured at deposition (the wind agent, journal/0034).
/// `None` is the always-present default: a run with the wind agent OFF tags every
/// unit `None`, which is byte-identical to the pre-wind record (the enum is a new
/// merge-key axis whose only inhabited value is `None` when wind is off, so two
/// otherwise-identical tags merge exactly as before — the same trick
/// [`Biofacies::Mineral`] plays).
///
/// Wind is a *distinct transport agent* — well-sorted, and it climbs gradients
/// the wrong way for water — so its deposits must be a distinguishable species in
/// the record even though the collapse tier still routes them to ordinary clastic
/// classes by grain size (loess → fine, dune sand → coarse, via
/// [`crate::geology::deep_class`]). At the deep tier's 460 m cells the readable
/// unit is the *region*: a loess sheet or a dune field, not an individual dune
/// (earth-processes.md § 4 — "dune-field/loess regions … individual dunes are
/// collapse-tier detail"). Carrying the marker now is what lets a future arid-
/// landform pass find "where did the desert lay its sheets" by reading the record
/// instead of re-deriving the paleo-wind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Eolian {
    /// Not wind-deposited (default — the only value when the wind agent is off).
    #[default]
    None,
    /// Wind-blown silt trapped at the downwind (vegetated/humid) margin of an
    /// arid source — a **loess** region. Fine clastic.
    Loess,
    /// Sand accumulating in the arid source zone itself — a **dune field**.
    /// Coarse clastic.
    Dune,
}

impl Eolian {
    /// Short code for column printouts (`--` when not aeolian).
    pub fn code(self) -> &'static str {
        match self {
            Eolian::None => "--",
            Eolian::Loess => "Lo",
            Eolian::Dune => "Du",
        }
    }
}

/// A measured depositional tag. Two units merge only when every measured axis
/// agrees (and the younger is not the first unit after an erosional strip).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DepTag {
    pub env: DepEnv,
    pub aridity: Aridity,
    pub energy: EnergyBand,
    /// The biotic facies measured at deposition (S10). `Mineral` for every unit
    /// when the biotic layer is off — byte-identical to the pre-S10 record.
    pub biota: Biofacies,
    /// The aeolian facies measured at deposition (the wind agent, journal/0034).
    /// `None` for every unit when the wind agent is off — byte-identical to the
    /// pre-wind record. Appended last, and defaulted through every constructor,
    /// so it is an additive axis (wire discipline — corrections #3).
    pub eolian: Eolian,
}

impl DepTag {
    /// A purely-mineral tag (biotic layer off / abiotic deposition). The erosion
    /// recorder builds every unit through this axis defaulted to `Mineral`, and
    /// non-aeolian (`Eolian::None`).
    pub fn mineral(env: DepEnv, aridity: Aridity, energy: EnergyBand) -> Self {
        Self {
            env,
            aridity,
            energy,
            biota: Biofacies::Mineral,
            eolian: Eolian::None,
        }
    }

    /// A short human-readable code for column printouts (`Sa/H/M`, `Ss/-/L`),
    /// with the biotic facies appended only when it is not `Mineral`
    /// (`Sa/H/L·Pt` for a humid low-energy peat).
    pub fn code(&self) -> String {
        let env = match self.env {
            DepEnv::Subaerial => "Sa",
            DepEnv::Subsea => "Ss",
        };
        let ar = match (self.env, self.aridity) {
            (DepEnv::Subsea, _) => "-",
            (_, Aridity::Arid) => "A",
            (_, Aridity::Humid) => "H",
        };
        let en = match self.energy {
            EnergyBand::Low => "L",
            EnergyBand::Medium => "M",
            EnergyBand::High => "H",
        };
        let base = if self.biota == Biofacies::Mineral {
            format!("{env}/{ar}/{en}")
        } else {
            format!("{env}/{ar}/{en}·{}", self.biota.code())
        };
        if self.eolian == Eolian::None {
            base
        } else {
            format!("{base}»{}", self.eolian.code())
        }
    }
}

/// One recorded unit, **PACKED** (P11 slice 3, U5/P-2 ruled L-8, user
/// 2026-08-02): a u32 bitfield of every measured axis + a u32 second word
/// holding **fixed-point thickness and the deposition epoch**. **8 bytes per
/// unit** against the retired 16 (record 116.31 → 58.15 MiB at the 2026-08-02
/// unit count), while ADDING two axes — the mover (stub #25's agent axis) and
/// the grain reservation (U4: 5 φ classes in 3 bits, UNSET until FS-A writes
/// real state).
///
/// ## The bitfield (LSB up; 30 of 32 bits used, 2 spare)
///
/// | bits  | axis        | inhabitants |
/// |-------|-------------|-------------|
/// | 0     | env         | 2 ([`DepEnv`]) |
/// | 1     | aridity     | 2 ([`Aridity`]) |
/// | 2–3   | energy      | 3 ([`EnergyBand`]) |
/// | 4–6   | biota       | 6 ([`Biofacies`]) |
/// | 7–8   | eolian      | 3 ([`Eolian`]) |
/// | 9     | unconformity| 2 |
/// | 10–15 | species     | 26 today; 6 bits cover stub #21's 51-material ceiling |
/// | 16–18 | mover       | 7 (`FlowCause`) + [`MOVER_NONE`] |
/// | 19–21 | grain       | U4's 5 φ classes + [`Self::GRAIN_UNSET`] |
/// | 22–29 | chapter     | u8 generality kept |
///
/// ## The second word: u24 thickness quanta + u8 epoch (the DEPOSITION CLOCK,
/// O-2b, RULED 2026-08-04 — `docs/audits/2026-08-04-deposition-clock-design.md`)
///
/// `packed = (epoch << 24) | quanta`. The high byte is the **raw runner tick**
/// the unit was first deposited on (`DeepStepCtx::epoch`, 0..`iterations`) —
/// **never a derived time** (P-D: the engine must not bake a pack's calendar
/// into E2 storage; how long an epoch is in years is a world/pack call). It is
/// **structurally out of the merge key** (`key_bits` masks `bits` only), so
/// the unit count is bit-identical to the pre-clock record by construction. A
/// merged run keeps the **bottom (first) epoch** — `add_quanta` never touches
/// the high byte — so unit *k*'s age interval is `[epoch(k), epoch(k+1))` by
/// stack order (F3: no writer inserts below the top, no writer reorders).
///
/// ## Thickness: u24 fixed point at 2⁻¹⁰ m, and the Law-3 carry
///
/// [`Self::THICKNESS_QUANTUM_M`] = 2⁻¹⁰ m ≈ 0.977 mm; the u24 caps at
/// 2²⁴−1 quanta = **16,384 m per unit**, against a measured production max of
/// 65.38 m (journal/0141) — **250× headroom**, loudly debug-asserted at
/// construction and on every merge. (Thickness bits may be donated again if a
/// future axis needs them — u20 keeps 15×; u16 is the known floor violation,
/// O-7.) The record is
/// *working state* (the outcrop window reads thickness back every epoch), so
/// quantization enters the sim loop — every deposit/strip quantizes through a
/// per-cell f64 remainder ([`DeepStrata`]'s `carry`), which keeps
/// `|carry| ≤ q/2` at all times and makes the closure against the delivered
/// budget a *derivable* bound (quantum × op count) instead of a
/// float-accumulation shape. Bonus the U5 ruling named: fixed-point addition
/// is exact and commutative, so corrections #89's hazard class (segmentation
/// moving terrain through float summation order) **retires for the record's
/// own sums**.
///
/// ## Access is by ACCESSOR, never by field (user-endorsed repo pattern)
///
/// The P-2 ruling's load-bearing premise: every read goes through accessors,
/// so a future widening (the ruling-3 hint byte; eco/civ-era axes) is an
/// accessor + one golden re-capture, not a ~141-site sweep. Fields are
/// private; construction is [`Self::new`] (tests/probes) or the recorder.
///
/// **The mover axis is IN the merge key (M-1)** — taken on M0's measured split
/// factor of 1.0308× (under the ≲1.1 bar; the #88 rule — an axis joined the key
/// unmeasured once and multiplied units 2.4×). A colluvial and an alluvial bed
/// of the same rock are two units. **Grain is in the key** — inert while every unit is
/// UNSET (the #88 tripwire asserts the unit count is identical), and exactly
/// the split instrument FS-A's writers will be gated on.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DepUnit {
    bits: u32,
    /// `(epoch << 24) | quanta` — see the struct docs. Access by accessor only
    /// ([`Self::thickness_quanta`] / [`Self::epoch`] / [`Self::add_quanta`] /
    /// [`Self::sub_quanta`]); a raw read of this field conflates two axes.
    packed: u32,
}

const ENV_SHIFT: u32 = 0;
const ARIDITY_SHIFT: u32 = 1;
const ENERGY_SHIFT: u32 = 2;
const BIOTA_SHIFT: u32 = 4;
const EOLIAN_SHIFT: u32 = 7;
const UNCONF_SHIFT: u32 = 9;
const SPECIES_SHIFT: u32 = 10;
const MOVER_SHIFT: u32 = 16;
const GRAIN_SHIFT: u32 = 19;
const CHAPTER_SHIFT: u32 = 22;
/// Everything that identifies a bed: the tag axes + species + **mover** +
/// grain + chapter. EXCLUDES `unconformity` (a property of the contact below,
/// exactly as the 16-byte merge ignored it).
///
/// **The mover is IN the key (M-1), and it earned its place by measurement**
/// (the #88 rule — an axis joined this key unmeasured once and multiplied
/// units 2.4×): M0 counted same-key runs whose dominant mover differs on the
/// shipped world — **split factor 1.0308×** (226,534 would-be splits over
/// 7,363,947 units, seed 1337 Medium, 2026-08-02) — well under the audit's
/// ≲1.1× bar, against the −58 MiB the pack banks. A colluvial and an alluvial
/// bed of the same rock are genuinely different rocks to a geologist
/// (stub #25); now they are two units.
const MERGE_KEY_MASK: u32 = (1 << ENV_SHIFT)
    | (1 << ARIDITY_SHIFT)
    | (0b11 << ENERGY_SHIFT)
    | (0b111 << BIOTA_SHIFT)
    | (0b11 << EOLIAN_SHIFT)
    | (0x3F << SPECIES_SHIFT)
    | (0b111 << MOVER_SHIFT)
    | (0b111 << GRAIN_SHIFT)
    | (0xFF << CHAPTER_SHIFT);

/// The low 24 bits of [`DepUnit`]'s second word: thickness quanta. The high
/// byte above it is the deposition epoch.
const THICKNESS_MASK: u32 = 0x00FF_FFFF;
const EPOCH_SHIFT: u32 = 24;

impl DepUnit {
    /// The thickness quantum: 2⁻¹⁰ m. Power of two, so f64 ↔ fixed conversion
    /// is exact on dyadics and quantum sums are exact in f64 far past any
    /// record's total.
    pub const THICKNESS_QUANTUM_M: f64 = 1.0 / 1024.0;
    /// "No grain recorded here" — the `Identity::Unrecorded` shape
    /// (journal/0101), loud rather than a plausible default. Every unit
    /// carries it until FS-A's release-spectrum writers land real state
    /// through [`Self::set_grain`].
    pub const GRAIN_UNSET: u8 = 7;

    /// A unit from parts (the recorder's constructor). `epoch` is the raw
    /// runner tick of this deposit (the deposition clock's stamp).
    fn from_parts(
        tag: DepTag,
        quanta: u32,
        unconformity: bool,
        chapter: u8,
        epoch: u8,
        species: MaterialId,
        mover: u8,
    ) -> DepUnit {
        let env = match tag.env {
            DepEnv::Subaerial => 0u32,
            DepEnv::Subsea => 1,
        };
        let aridity = match tag.aridity {
            Aridity::Arid => 0u32,
            Aridity::Humid => 1,
        };
        let energy = match tag.energy {
            EnergyBand::Low => 0u32,
            EnergyBand::Medium => 1,
            EnergyBand::High => 2,
        };
        let biota = match tag.biota {
            Biofacies::Mineral => 0u32,
            Biofacies::Soil => 1,
            Biofacies::Peat => 2,
            Biofacies::Coal => 3,
            Biofacies::Charcoal => 4,
            Biofacies::Retro => 5,
        };
        let eolian = match tag.eolian {
            Eolian::None => 0u32,
            Eolian::Loess => 1,
            Eolian::Dune => 2,
        };
        debug_assert!(mover <= MOVER_NONE, "mover is 3 bits");
        debug_assert!(
            quanta <= THICKNESS_MASK,
            "DEPOSITION-CLOCK OVERFLOW: a single unit of {quanta} quanta \
             ({:.1} m) exceeds the u24 thickness cap of 16,384 m — the measured \
             production max is 65.38 m (250x headroom); this is a defect, not a \
             capacity call. Escalation is O-1's 12 bytes or a declared epoch \
             coarsening, never another thickness byte (O-7).",
            f64::from(quanta) * Self::THICKNESS_QUANTUM_M
        );
        let bits = (env << ENV_SHIFT)
            | (aridity << ARIDITY_SHIFT)
            | (energy << ENERGY_SHIFT)
            | (biota << BIOTA_SHIFT)
            | (eolian << EOLIAN_SHIFT)
            | (u32::from(unconformity) << UNCONF_SHIFT)
            | (u32::from(species.raw()) << SPECIES_SHIFT)
            | (u32::from(mover) << MOVER_SHIFT)
            | (u32::from(Self::GRAIN_UNSET) << GRAIN_SHIFT)
            | (u32::from(chapter) << CHAPTER_SHIFT);
        DepUnit {
            bits,
            packed: (u32::from(epoch) << EPOCH_SHIFT) | (quanta & THICKNESS_MASK),
        }
    }

    /// A unit stated in metres — the constructor tests and probes use.
    /// Thickness rounds to the nearest quantum (no carry: an isolated unit has
    /// no cell to carry for); mover [`MOVER_NONE`], grain [`Self::GRAIN_UNSET`].
    pub fn new(
        tag: DepTag,
        thickness_m: f64,
        unconformity: bool,
        chapter: u8,
        epoch: u8,
        species: MaterialId,
    ) -> DepUnit {
        let q = (thickness_m / Self::THICKNESS_QUANTUM_M).round().max(0.0) as u32;
        Self::from_parts(tag, q, unconformity, chapter, epoch, species, MOVER_NONE)
    }

    /// The measured depositional tag, reassembled from the bitfield.
    pub fn tag(&self) -> DepTag {
        let env = match (self.bits >> ENV_SHIFT) & 1 {
            0 => DepEnv::Subaerial,
            _ => DepEnv::Subsea,
        };
        let aridity = match (self.bits >> ARIDITY_SHIFT) & 1 {
            0 => Aridity::Arid,
            _ => Aridity::Humid,
        };
        let energy = match (self.bits >> ENERGY_SHIFT) & 0b11 {
            0 => EnergyBand::Low,
            1 => EnergyBand::Medium,
            _ => EnergyBand::High,
        };
        let biota = match (self.bits >> BIOTA_SHIFT) & 0b111 {
            0 => Biofacies::Mineral,
            1 => Biofacies::Soil,
            2 => Biofacies::Peat,
            3 => Biofacies::Coal,
            4 => Biofacies::Charcoal,
            _ => Biofacies::Retro,
        };
        let eolian = match (self.bits >> EOLIAN_SHIFT) & 0b11 {
            0 => Eolian::None,
            1 => Eolian::Loess,
            _ => Eolian::Dune,
        };
        DepTag {
            env,
            aridity,
            energy,
            biota,
            eolian,
        }
    }

    /// Thickness in metres: quanta × [`Self::THICKNESS_QUANTUM_M`], exact.
    #[inline]
    pub fn thickness_m(&self) -> f64 {
        f64::from(self.thickness_quanta()) * Self::THICKNESS_QUANTUM_M
    }

    /// Thickness in quanta of 2⁻¹⁰ m — the exact integer the sums close in.
    /// The low 24 bits of the second word; the epoch rides above.
    #[inline]
    pub fn thickness_quanta(&self) -> u32 {
        self.packed & THICKNESS_MASK
    }

    /// **The deposition epoch** — the raw runner tick (`DeepStepCtx::epoch`)
    /// this unit was **first** deposited on (the deposition clock, O-2b, ruled
    /// 2026-08-04). A merged run keeps the epoch it *started*, so unit *k*'s
    /// age interval is `[epoch(k), epoch(k+1))` by stack order (non-decreasing
    /// up-stack — debug-asserted at every push).
    ///
    /// **A raw tick, never a derived time** (P-D): what an epoch means in
    /// years, and whether chapters exist, are world/pack calls — the record
    /// stores the schedule-invariant loop counter and nothing else. Structurally
    /// out of the merge key (the key masks `bits` only), so this axis cannot
    /// move the unit count.
    ///
    /// **IMMUTABLE under pedogenic overprint** (F4, integrator ruling
    /// 2026-08-04): overprint is *alteration*, not deposition — see
    /// [`Self::set_tag_and_chapter`].
    #[inline]
    pub fn epoch(&self) -> u8 {
        (self.packed >> EPOCH_SHIFT) as u8
    }

    /// Recorder-internal: grow the thickness (the merge paths). The epoch byte
    /// is never touched — a merged run keeps its **bottom (first) epoch**,
    /// which is the ruled merge semantics (O-2b) and costs no code beyond this
    /// mask discipline.
    #[inline]
    fn add_quanta(&mut self, q: u32) {
        let t = self.thickness_quanta();
        debug_assert!(
            q <= THICKNESS_MASK - t,
            "DEPOSITION-CLOCK OVERFLOW on merge: {t} + {q} quanta exceeds the \
             u24 cap (16,384 m) — see DepUnit::from_parts"
        );
        self.packed = (self.packed & !THICKNESS_MASK) | ((t.wrapping_add(q)) & THICKNESS_MASK);
    }

    /// Recorder-internal: shrink the thickness (`erode`'s partial pop). The
    /// epoch byte is never touched — a truncated unit keeps its deposition
    /// tick; erosion removes rock, not history-of-when.
    #[inline]
    fn sub_quanta(&mut self, q: u32) {
        let t = self.thickness_quanta();
        debug_assert!(q <= t, "erode must never truncate below zero quanta");
        self.packed = (self.packed & !THICKNESS_MASK) | ((t - q) & THICKNESS_MASK);
    }

    /// True when this unit was deposited directly onto bedrock the column had
    /// been stripped to since the previous unit — a time gap you can see and
    /// reason about (earth-processes.md § 7). A property of the overlying unit.
    #[inline]
    pub fn unconformity(&self) -> bool {
        (self.bits >> UNCONF_SHIFT) & 1 == 1
    }

    /// The **tectonic chapter** (0-based) this unit was deposited in
    /// (tectonics.md § 3.3) — in the merge key, so units never merge across a
    /// chapter boundary. Always `0` when tectonic history is off.
    #[inline]
    pub fn chapter(&self) -> u8 {
        ((self.bits >> CHAPTER_SHIFT) & 0xFF) as u8
    }

    /// **The rock this bed is made of** — a registry [`MaterialId`] (P11
    /// slice 1, ruling 1: *history records the rock, not the road to it*),
    /// chosen at deposition under the epoch's own formation context (or taken
    /// from the arriving composition — ruling 6). 6 bits: stub #21's
    /// 51-material ceiling fits with headroom.
    #[inline]
    pub fn species(&self) -> MaterialId {
        MaterialId::from_raw(((self.bits >> SPECIES_SHIFT) & 0x3F) as u8)
            .expect("a recorded species is always a registry id")
    }

    /// **The mover** (stub #25's agent axis, funded by the pack): which
    /// transport regime delivered this bed, as `FlowCause as u8`, or
    /// [`MOVER_NONE`] for material made where it lies. Colluvium and alluvium
    /// stopped being separable-only-by-signature the day this landed.
    ///
    /// **In the merge key (M-1)** — joined after measurement, never before
    /// (#88): the M0 instrument counted a 1.0308× split on the shipped world
    /// (2026-08-02), see [`MERGE_KEY_MASK`].
    #[inline]
    pub fn mover(&self) -> u8 {
        ((self.bits >> MOVER_SHIFT) & 0b111) as u8
    }

    /// **The grain axis** (U4: 5 φ classes — scree/gravel/sand/silt/clay as
    /// `0..=4`), [`Self::GRAIN_UNSET`] until FS-A's writers land. In the merge
    /// key — inert while uniform (the #88 tripwire), and exactly the split
    /// gate FS-A is measured against.
    #[inline]
    pub fn grain(&self) -> u8 {
        ((self.bits >> GRAIN_SHIFT) & 0b111) as u8
    }

    /// Write the grain axis — **FS-A's write surface** (release spectra;
    /// nothing calls this in slice 3, deliberately: the bits are reserved,
    /// loud, and unset).
    #[inline]
    pub fn set_grain(&mut self, grain: u8) {
        debug_assert!(grain <= Self::GRAIN_UNSET, "grain is 3 bits");
        self.bits =
            (self.bits & !(0b111 << GRAIN_SHIFT)) | (u32::from(grain & 0b111) << GRAIN_SHIFT);
    }

    /// The merge-key view of the bitfield (tag axes + species + grain + mover +
    /// chapter; never unconformity — see [`MERGE_KEY_MASK`]).
    #[inline]
    fn key_bits(&self) -> u32 {
        self.bits & MERGE_KEY_MASK
    }

    /// Recorder-internal: re-identify the bed (pedogenic overprint, burial
    /// diagenesis — genuine transformations, never a convenience rewrite).
    fn set_species(&mut self, species: MaterialId) {
        self.bits =
            (self.bits & !(0x3F << SPECIES_SHIFT)) | (u32::from(species.raw()) << SPECIES_SHIFT);
    }

    /// Recorder-internal: an in-place alteration (pedogenic overprint) has no
    /// mover — the horizon is made where it lies.
    fn set_mover(&mut self, mover: u8) {
        self.bits =
            (self.bits & !(0b111 << MOVER_SHIFT)) | (u32::from(mover & 0b111) << MOVER_SHIFT);
    }

    /// Recorder-internal: burial diagenesis retags peat to coal in place.
    fn set_biota(&mut self, biota: Biofacies) {
        let b = match biota {
            Biofacies::Mineral => 0u32,
            Biofacies::Soil => 1,
            Biofacies::Peat => 2,
            Biofacies::Coal => 3,
            Biofacies::Charcoal => 4,
            Biofacies::Retro => 5,
        };
        self.bits = (self.bits & !(0b111 << BIOTA_SHIFT)) | (b << BIOTA_SHIFT);
    }

    /// Recorder-internal: the pedogenic overprint retags the whole
    /// environment axis set + chapter in place (the horizon is a new bed).
    ///
    /// ⚠ **STAND-IN — chapter and epoch DISAGREE after an overprint, on
    /// purpose** (F4, deposition-clock design, integrator ruling 2026-08-04,
    /// consistent with P-D). This rewrites the CHAPTER byte (as-built S10
    /// behaviour: "the horizon is a new bed") but the EPOCH in the second word
    /// is **immutable** — the unit's epoch is its DEPOSITION tick, and
    /// overprint is *alteration*, not deposition. So an overprinted unit can
    /// read `chapter 4` beside `epoch 37` (chapter 1): the chapter says when
    /// the horizon was last altered, the epoch says when its substrate was
    /// laid. **Heir: a future alteration-time axis** — a second tick recording
    /// when pedogenesis/diagenesis reworked a bed, at which point the chapter
    /// rewrite here retires and both questions get honest answers. Until then
    /// this comment is the marker; the stubs entry names the blast radius.
    fn set_tag_and_chapter(&mut self, tag: DepTag, chapter: u8) {
        let fresh = DepUnit::from_parts(tag, 0, false, chapter, 0, self.species(), MOVER_NONE);
        const TAG_MASK: u32 = (1 << ENV_SHIFT)
            | (1 << ARIDITY_SHIFT)
            | (0b11 << ENERGY_SHIFT)
            | (0b111 << BIOTA_SHIFT)
            | (0b11 << EOLIAN_SHIFT)
            | (0xFF << CHAPTER_SHIFT);
        self.bits = (self.bits & !TAG_MASK) | (fresh.bits & TAG_MASK);
    }
}

/// **The pack, compiler-checked** (P11 slice 3; the successor of slice 1's
/// 16-byte pin). 8 bytes, align 4 — the whole L-8 residency argument
/// (116.31 → 58.15 MiB) rests on these two lines being facts.
const _: () = {
    assert!(size_of::<DepUnit>() == 8, "the packed DepUnit is 8 bytes");
    assert!(
        align_of::<DepUnit>() == 4,
        "u32 pair: align 4, no tail padding"
    );
    assert!(
        size_of::<MaterialId>() == 1,
        "the identity byte stays a byte (6 bits of it are packed)"
    );
    // The quantum is a power of two (exact dyadic f64 conversion) and the u24
    // range covers any geology: 2^24-1 quanta = 16,384 m of one bed against a
    // measured production max of 65.38 m (the deposition clock's O-2b split;
    // the high byte is the epoch).
    assert!(DepUnit::THICKNESS_QUANTUM_M == 1.0 / 1024.0);
    assert!(THICKNESS_MASK == (1 << EPOCH_SHIFT) - 1, "u24 + u8 split");
};

/// The ordered per-cell deposition log, bottom-up. `units[0]` is the deepest
/// recorded stratum; the last unit ends at the current surface. Below the
/// record is unrecorded bedrock (`R`).
#[derive(Clone, Default, PartialEq, Debug)]
pub struct DeepStrata {
    pub units: Vec<DepUnit>,
    /// Set when erosion strips the whole record to bedrock; the next deposit is
    /// flagged as an unconformity, then this clears.
    stripped: bool,
    /// Count of erosional strips over the run (proto-unconformity generator);
    /// a read-quality metric, not part of the record's mass.
    pub strips: u32,
    /// **The sub-quantum remainder carry** (P11 slice 3, the pack's Law-3
    /// half): every deposit/strip quantizes through this per-cell f64, so
    /// `|carry| ≤ quantum/2` at all times and `record + carry` closes against
    /// the delivered budget to a *derivable* bound (quantum × op count) — the
    /// U5 ruling's stated condition. Gen-transient; not part of the record's
    /// mass and never expressed.
    ///
    /// *(The M0 mover-split instrument that briefly lived beside this —
    /// counting the extra units the mover axis would create in the merge key —
    /// did its one job on 2026-08-02: measured 1.0308×, the mover joined the
    /// key (see [`MERGE_KEY_MASK`]), and the counter went with it.)*
    carry: f64,
}

impl DeepStrata {
    /// Total recorded thickness (metres) — must equal `H` within the carry
    /// (|Δ| ≤ quantum/2) at every point in the run (the finalize invariant,
    /// tested). Exact: an integer quantum sum scaled once.
    pub fn total_m(&self) -> f64 {
        self.total_quanta() as f64 * DepUnit::THICKNESS_QUANTUM_M
    }

    /// Total recorded thickness in quanta — the integer the closure laws are
    /// exact in.
    pub fn total_quanta(&self) -> u64 {
        self.units
            .iter()
            .map(|u| u64::from(u.thickness_quanta()))
            .sum()
    }

    /// The sub-quantum remainder this cell is carrying (metres). Bounded:
    /// `|carry| ≤ quantum/2` after every recorded op — asserted by the gate.
    pub fn carry_m(&self) -> f64 {
        self.carry
    }

    /// Record a net deposition of `d` metres (`d > 0`) under `tag`, deposited in
    /// tectonic `chapter`. Merges into the top unit when the tag **and chapter**
    /// agree and the column is conformable; starts a new (unconformity-flagged)
    /// unit otherwise. `chapter` is `0` on the pre-tectonic-history path, so the
    /// extra `top.chapter == chapter` guard is always satisfied and merging is
    /// byte-identical to before.
    /// **The reference-material door — no registry is consulted.**
    ///
    /// Deposits the *reference member* of the tag's class, which is what the
    /// record said for every depositor before P11. It survives for exactly two
    /// customers and neither is a shipped depositor:
    ///
    /// - **tests and probes** that want mass in a column and have no content set
    ///   to hand (a `DeepStrata` built in isolation);
    /// - reasoning about the **degenerate content set**, where the class a bed
    ///   belongs to has no member at all.
    ///
    /// Every production depositor now states the material it picked
    /// ([`Self::deposit_as`]) after running fitness under its own formation
    /// context. **If you are adding a depositing agent, you want `deposit_as`.**
    ///
    /// ⚠ **Heir: this goes with `Litho`** (P11 slice 4). It is the last live
    /// caller of [`Litho::reference_material`] outside the S-5 fallback in
    /// [`DepositCtx::material_in`], and both die when the class roster does.
    pub fn deposit(&mut self, tag: DepTag, d: f64, chapter: u8, epoch: u8) {
        self.deposit_as(
            tag,
            d,
            chapter,
            epoch,
            litho_of_tag(tag).reference_material(),
        );
    }

    /// [`Self::deposit`], but stating **which material arrived** rather than
    /// letting it be inferred from the tag (Movement 2b, `material-behavior.md`
    /// § 13.3).
    ///
    /// **This is the writer every production depositor uses** (P11 slice 1).
    ///
    /// Two different questions arrive here and both end in a `MaterialId`:
    ///
    /// - *what did a mover bring?* — the erosion recorder answers with the argmax
    ///   of everything that arrived at the cell (the fluvial pass's deposit plus
    ///   what hillslope creep brought down the slope, journal/0112, § 13.2's
    ///   **gravity** member), resolved to a member under the epoch's own
    ///   formation context;
    /// - *what does this environment make?* — the wind agent, the wave agent and
    ///   the biotic layer carry no identity, so their class comes from the tag,
    ///   but the **member** is still chosen by fitness at deposition
    ///   ([`DepositCtx::material_for`]) rather than fixed to a reference. (The
    ///   **eolian** family genuinely has a load too and would honestly travel its
    ///   own identity; that is deferred with the rest of § 13.2's wind/ice
    ///   members.)
    ///
    /// `species` joins the merge key: two runs of the same environment that
    /// delivered *different rock* are two units, not one. That is the whole point —
    /// a sand sheet and the mud that followed it at the same tag are a contact you
    /// can see in a cliff. **Member grade sharpens the key**: two beds that merged
    /// as one `ClasticFine` unit split when one is mudstone and the other
    /// siltstone, so the unit count rises with the diversity. That is the record
    /// getting more honest, and it is the second-order residency cost the design
    /// audit left unpriced (§ 6a, I4).
    pub fn deposit_as(&mut self, tag: DepTag, d: f64, chapter: u8, epoch: u8, species: MaterialId) {
        self.deposit_moved(tag, d, chapter, epoch, species, MOVER_NONE);
    }

    /// [`Self::deposit_as`], but stating **which mover delivered it** — the agent
    /// axis (stub #25), as `FlowCause as u8` or [`MOVER_NONE`] for material made
    /// in place. Every production depositor states its mover: the erosion
    /// recorder passes the dominant arriving mover (fluvial vs hillslope
    /// gravity), the wind agent [`FlowCause::Eolian`], the wave agent
    /// [`FlowCause::Marine`]; pedogenesis and the fitness-draw remainder are
    /// [`MOVER_NONE`].
    ///
    /// **The mover is in the merge key (M-1), and it earned its place by
    /// measurement** (see [`MERGE_KEY_MASK`]: split factor 1.0308×, M0
    /// 2026-08-02) — so a colluvial and an alluvial bed of the same rock under
    /// the same environment are two units, which is what they are to a
    /// geologist (stub #25, discharged).
    ///
    /// [`FlowCause::Eolian`]: super::flux::FlowCause::Eolian
    /// [`FlowCause::Marine`]: super::flux::FlowCause::Marine
    pub fn deposit_moved(
        &mut self,
        tag: DepTag,
        d: f64,
        chapter: u8,
        epoch: u8,
        species: MaterialId,
        mover: u8,
    ) {
        // Quantize through the per-cell carry (the pack's Law-3 half): the
        // metres that don't make a whole quantum ride here until they do, so
        // |carry| ≤ quantum/2 after every op and nothing is silently dropped.
        self.carry += d;
        let qf = (self.carry / DepUnit::THICKNESS_QUANTUM_M).round();
        if qf < 1.0 {
            return; // sub-quantum: rides in the carry
        }
        let q = qf as u32;
        self.carry -= f64::from(q) * DepUnit::THICKNESS_QUANTUM_M;
        let unit = DepUnit::from_parts(tag, q, self.stripped, chapter, epoch, species, mover);
        if !self.stripped
            && let Some(top) = self.units.last_mut()
            && top.key_bits() == unit.key_bits()
        {
            // The merge keeps the BOTTOM (first) epoch: `add_quanta` never
            // touches the high byte, so the run records the tick it started.
            top.add_quanta(q);
            return;
        }
        // The clock's monotonicity, converted from an argument into a fact
        // (deposition-clock I-4, discharging the correlation design's § 10.4
        // owed chapter assert in the same line): no writer inserts below the
        // top and no writer reorders, so both stamps are non-decreasing
        // up-stack in every cell at all times.
        debug_assert!(
            self.units
                .last()
                .is_none_or(|t| t.epoch() <= epoch && t.chapter() <= chapter),
            "the deposition clock ran backwards: pushing epoch {epoch} \
             chapter {chapter} over epoch {} chapter {}",
            self.units.last().map_or(0, |t| t.epoch()),
            self.units.last().map_or(0, |t| t.chapter()),
        );
        self.units.push(unit);
        self.stripped = false;
    }

    /// **Pedogenic overprint** (S10): soil formation is not a deposit stacked on
    /// top of the column — it *alters the material already at the surface*. This
    /// adds `extra` metres of organic matter to the topmost unit and retags it
    /// with `tag`, then merges it down into the unit below if that unit carries
    /// the identical tag (so a surface that stays stable for a hundred epochs
    /// records **one thick horizon**, not a hundred laminae).
    ///
    /// Two refusals keep the record honest:
    /// - a `Charcoal` top is never retagged — a fire band is a preserved event
    ///   bed, so the soil starts a *new* unit above it;
    /// - a unit flagged `unconformity` is never merged downward, which would
    ///   erase the time gap.
    ///
    /// Preserves `sum(units) == H` exactly: `extra` is added once, and the merge
    /// only moves thickness between units.
    /// `species` is the material the overprint *makes*: pedogenesis is a genuine
    /// identity event (the horizon is a new rock), so its member is picked by
    /// fitness at the epoch that built it, under [`dep_tags::PEDOGENIC`].
    pub fn overprint_top(
        &mut self,
        tag: DepTag,
        extra: f64,
        chapter: u8,
        epoch: u8,
        species: MaterialId,
    ) {
        // The organic gain quantizes through the same per-cell carry every
        // deposit does; the RETAG below happens regardless (a sub-quantum
        // horizon still alters the surface bed's identity).
        self.carry += extra;
        let qf = (self.carry / DepUnit::THICKNESS_QUANTUM_M).round();
        let q = if qf >= 1.0 {
            self.carry -= qf * DepUnit::THICKNESS_QUANTUM_M;
            qf as u32
        } else {
            0
        };
        let charcoal_top = self
            .units
            .last()
            .is_some_and(|u| u.tag().biota == Biofacies::Charcoal);
        if self.units.is_empty() || charcoal_top {
            if q == 0 {
                return; // nothing expressible yet: the gain rides in the carry
            }
            // Bare bedrock, or a fire bed we must not overwrite: start a unit.
            // This branch IS a deposition, so the new unit takes the current
            // epoch (the same monotonicity fact as deposit_moved's push).
            debug_assert!(
                self.units
                    .last()
                    .is_none_or(|t| t.epoch() <= epoch && t.chapter() <= chapter),
                "the deposition clock ran backwards under an overprint push"
            );
            self.units.push(DepUnit::from_parts(
                tag,
                q,
                self.stripped,
                chapter,
                epoch,
                species,
                MOVER_NONE,
            ));
            self.stripped = false;
            return;
        }
        let top = self.units.last_mut().expect("non-empty");
        // The altered bed's EPOCH is untouched (F4 ruling: the epoch is the
        // deposition tick, immutable under overprint — alteration is not
        // deposition; the chapter rewrite below is the as-built stand-in, see
        // `set_tag_and_chapter`).
        top.add_quanta(q);
        top.set_tag_and_chapter(tag, chapter);
        // Pedogenesis is an in-place alteration: nothing rode a mover, so the
        // horizon's mover is NONE whatever delivered the material it reworked.
        top.set_mover(MOVER_NONE);
        // Pedogenesis **alters the material in place** — the horizon a community
        // built out of what was lying here is an organic soil whatever the flow
        // delivered, so the overprint takes the soil's own identity and the
        // transported one is genuinely overwritten rather than lost.
        top.set_species(species);
        // Merge down into an identically-keyed predecessor of the same chapter.
        // The SURVIVOR is units[n-2], so the merged horizon keeps the OLDER
        // (bottom) epoch — the same first-epoch semantics as deposit_moved's
        // merge, with no code beyond add_quanta's mask discipline.
        let n = self.units.len();
        if n >= 2
            && self.units[n - 2].key_bits() == self.units[n - 1].key_bits()
            && !self.units[n - 1].unconformity()
        {
            let t = self.units.pop().expect("non-empty").thickness_quanta();
            self.units.last_mut().expect("non-empty").add_quanta(t);
        }
    }

    /// Pop `amount` metres of recorded history off the top (erosion), through
    /// the sub-quantum carry. When the record empties, the column is stripped
    /// to bedrock and the next deposit will record an unconformity. Demand the
    /// record cannot pay is dropped, exactly as the pre-pack path dropped it
    /// (`H` clamps to zero in the same place upstream).
    pub fn erode(&mut self, amount: f64) {
        self.carry -= amount;
        let qf = (-self.carry / DepUnit::THICKNESS_QUANTUM_M).round();
        if qf >= 1.0 {
            self.carry += qf * DepUnit::THICKNESS_QUANTUM_M;
            let mut q = qf as u64;
            while q > 0 {
                let Some(top) = self.units.last_mut() else {
                    break; // record spent: the leftover demand is dropped
                };
                let tq = u64::from(top.thickness_quanta());
                if tq <= q {
                    q -= tq;
                    self.units.pop();
                } else {
                    // A truncated unit keeps its epoch: erosion removes rock,
                    // never the surviving bed's deposition tick.
                    top.sub_quanta(q as u32);
                    q = 0;
                }
            }
        }
        if self.units.is_empty() && !self.stripped {
            self.stripped = true;
            self.strips += 1;
        }
    }

    /// Number of distinct tags present in the record (a variety metric).
    pub fn tag_variety(&self) -> usize {
        let mut seen: Vec<DepTag> = Vec::new();
        for u in &self.units {
            if !seen.contains(&u.tag()) {
                seen.push(u.tag());
            }
        }
        seen.len()
    }

    /// Count of unconformable contacts (units flagged as sitting on an
    /// erosional surface) — the readable time gaps.
    pub fn unconformities(&self) -> usize {
        self.units.iter().filter(|u| u.unconformity()).count()
    }

    /// Count of **paleosols**: buried organic horizons — a `Soil`/`Peat`/`Coal`/
    /// `Retro` unit that is *not* the topmost unit (something was deposited over
    /// it, so it is a fossil soil rather than the living surface). Each carries
    /// its own at-deposition climate tag (`env`/`aridity`) on `DepUnit::tag`.
    pub fn paleosols(&self) -> usize {
        if self.units.is_empty() {
            return 0;
        }
        let last = self.units.len() - 1;
        self.units
            .iter()
            .enumerate()
            .filter(|(k, u)| *k != last && u.tag().biota.is_organic())
            .count()
    }

    /// Count of coal seams (units tagged [`Biofacies::Coal`]) at or above
    /// `min_m` thickness — the legible ones.
    pub fn coal_seams(&self, min_m: f64) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag().biota == Biofacies::Coal && u.thickness_m() >= min_m)
            .count()
    }

    /// Count of charcoal bands (fire-event units) in the record.
    pub fn charcoal_bands(&self) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag().biota == Biofacies::Charcoal)
            .count()
    }

    /// Count of retrogressive organic horizons (units tagged
    /// [`Biofacies::Retro`]) — collapsed, P-starved surfaces.
    pub fn retro_surfaces(&self) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag().biota == Biofacies::Retro)
            .count()
    }

    /// **Burial diagenesis** (earth-processes.md § 5): promote a peat unit to
    /// coal once the temperature it has seen reaches `onset_c`.
    ///
    /// The temperature comes from the **geotherm** — the `temperature`
    /// condition-field ([`super::geotherm`], §14): for a candidate unit at
    /// mid-slab burial depth `z`, `T = surface_T + gradient·z`, where the
    /// per-column `surface_T` and `gradient` ride on [`BurialColumn`]. The
    /// gradient is set by crustal heat flow — steep under rifts/arcs, shallow
    /// under thick cratonic crust — so **where** the crust is hot now shifts
    /// where coal forms, not merely how deep it is buried. This retired the
    /// degenerate `burial_temp_c` provider (a 1 °C/m identity where "temperature"
    /// *was* the overburden in metres; journal/0093, `stubs.md` §14).
    ///
    /// **The axis is burial depth, since journal/0063** — coal is made by burial,
    /// not by a swamp's duration. `dc_core::materials::geology::CLASS_ORGANIC_COAL`'s
    /// contract — *"the class's depth axis is the rank axis"* — is what a real
    /// geotherm finally lets this become: a P/T path down which the single `Coal`
    /// facies can one day split by rank.
    ///
    /// **P11 slice 1 is what makes that expressible, and it is the one identity
    /// event that is not a surface event.** Every other depositor picks its member
    /// under a formation context whose `depth_m` is 0 — a bed is laid at the
    /// surface. This one is a *burial* transformation, so it re-runs fitness under
    /// the depth and the geotherm temperature it has just computed: the rank axis
    /// is the real burial depth of this slab in this column. Vanilla ships one
    /// coal member so the pick cannot vary yet; the day a pack registers lignite
    /// and anthracite, they separate here with no further work.
    ///
    /// **The topmost unit is never promoted**, unconditionally. Its overburden
    /// is zero, so every *positive* threshold excludes it anyway — but the guard
    /// is written out rather than implied, because "the living surface is not
    /// coal" is a statement about the world and not an artefact of which number
    /// the threshold happens to be. (A test pins it at `onset_c == 0.0`,
    /// the one value where the implication fails.)
    ///
    /// The temperature is computed **only for candidates** — non-top peat units —
    /// which keeps the work proportional to the peat in the record rather than to
    /// the record. Called once at run finalize. Pure in the record; preserves
    /// `sum(units) == H` (only the tag changes, never a thickness).
    pub fn promote_coal(&mut self, col: BurialColumn, onset_c: f64, ctx: &DepositCtx) {
        // Top-down, accumulating overburden: one pass, no allocation.
        let top = self.units.len().saturating_sub(1);
        let mut overburden_m = 0.0f64;
        for (k, u) in self.units.iter_mut().enumerate().rev() {
            if k != top && u.tag().biota == Biofacies::Peat {
                // Burial depth to the slab's mid-point (the recorder residual the
                // geotherm integrates a linear gradient across).
                let depth_m = overburden_m + 0.5 * u.thickness_m();
                let t_c =
                    geotherm::temperature_c(col.surface_temp_c, col.gradient_c_per_m, depth_m);
                if t_c >= onset_c {
                    u.set_biota(Biofacies::Coal);
                    // Diagenesis is a **material transformation**: the unit stops
                    // being peat, so its identity is re-picked — under *this*
                    // slab's burial P/T, which is the coal class's own rank axis.
                    let form = FormationContext {
                        temp_c: t_c,
                        precip: ctx.form.precip,
                        depth_m,
                    };
                    let sp = ctx.material_in(
                        litho_of_tag(u.tag()),
                        &form,
                        dep_tags::DIAGENESIS,
                        k as u64,
                    );
                    u.set_species(sp);
                }
            }
            overburden_m += u.thickness_m();
        }
    }

    /// Rough heap footprint of this record's unit vector (bytes).
    pub fn heap_bytes(&self) -> usize {
        self.units.capacity() * std::mem::size_of::<DepUnit>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(env: DepEnv, energy: EnergyBand) -> DepTag {
        DepTag::mineral(env, Aridity::Humid, energy)
    }

    const Q: f64 = DepUnit::THICKNESS_QUANTUM_M;

    /// **The pack round-trips every axis** — all tag combinations, both
    /// unconformity states, every registry species, mover and grain values,
    /// and the chapter byte. Scale-free: a per-value encode/decode identity.
    #[test]
    fn the_packed_unit_round_trips_every_axis() {
        let envs = [DepEnv::Subaerial, DepEnv::Subsea];
        let arids = [Aridity::Arid, Aridity::Humid];
        let energies = [EnergyBand::Low, EnergyBand::Medium, EnergyBand::High];
        let biotas = [
            Biofacies::Mineral,
            Biofacies::Soil,
            Biofacies::Peat,
            Biofacies::Coal,
            Biofacies::Charcoal,
            Biofacies::Retro,
        ];
        let eolians = [Eolian::None, Eolian::Loess, Eolian::Dune];
        let mut checked = 0usize;
        for &env in &envs {
            for &aridity in &arids {
                for &energy in &energies {
                    for &biota in &biotas {
                        for &eolian in &eolians {
                            let t = DepTag {
                                env,
                                aridity,
                                energy,
                                biota,
                                eolian,
                            };
                            for unconf in [false, true] {
                                for chapter in [0u8, 7, 255] {
                                    for epoch in [0u8, 137, 255] {
                                        let sp = MaterialId::SILTSTONE;
                                        let u = DepUnit::from_parts(
                                            t, 3, unconf, chapter, epoch, sp, 2,
                                        );
                                        assert_eq!(u.tag(), t);
                                        assert_eq!(u.unconformity(), unconf);
                                        assert_eq!(u.chapter(), chapter);
                                        assert_eq!(u.epoch(), epoch);
                                        assert_eq!(u.species(), sp);
                                        assert_eq!(u.mover(), 2);
                                        assert_eq!(u.grain(), DepUnit::GRAIN_UNSET);
                                        assert_eq!(u.thickness_quanta(), 3);
                                        checked += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(checked, 2 * 2 * 3 * 6 * 3 * 2 * 3 * 3);
        // Every registry species survives the 6-bit field.
        for raw in 0..dc_core::materials::MATERIAL_COUNT as u8 {
            let sp = MaterialId::from_raw(raw).unwrap();
            let u = DepUnit::new(
                tag(DepEnv::Subaerial, EnergyBand::Low),
                1.0,
                false,
                0,
                0,
                sp,
            );
            assert_eq!(u.species(), sp);
        }
        // Grain writes round-trip without disturbing any sibling axis
        // (FS-A's write surface, exercised before it has a caller).
        let mut u = DepUnit::new(
            tag(DepEnv::Subsea, EnergyBand::High),
            2.5,
            true,
            9,
            42,
            MaterialId::CONGLOMERATE,
        );
        let before = (
            u.tag(),
            u.unconformity(),
            u.chapter(),
            u.epoch(),
            u.species(),
            u.mover(),
        );
        u.set_grain(3);
        assert_eq!(u.grain(), 3);
        assert_eq!(
            (
                u.tag(),
                u.unconformity(),
                u.chapter(),
                u.epoch(),
                u.species(),
                u.mover()
            ),
            before,
            "set_grain must not disturb any sibling axis"
        );
    }

    /// **The carry laws** (design audit § 2.3, the U5 ruling's gate):
    /// `|carry| ≤ quantum/2` after every op, and `record + carry` closes
    /// against the delivered budget within `quantum × op-count`-scaled
    /// round-off (each quantize step is exact on dyadics; the bound covers
    /// the f64 adds). Scale-free: per-cell arithmetic, no world.
    #[test]
    fn the_carry_stays_sub_quantum_and_the_budget_closes() {
        let mut s = DeepStrata::default();
        let t = tag(DepEnv::Subaerial, EnergyBand::Medium);
        let mut delivered = 0.0f64;
        // A deliberately awkward series: sub-quantum drips, big beds, and
        // partial strips, all non-dyadic.
        let ops: [f64; 12] = [
            0.0003, 0.0004, 1.2345, -0.5001, 0.00049, 0.7, -0.0002, 3.24159, -1.618, 0.0333,
            0.00021, -0.9999,
        ];
        let mut n_ops = 0u32;
        for (i, &d) in ops.iter().enumerate() {
            if d >= 0.0 {
                // Chapter and epoch advance with i (the clock's monotonicity
                // debug-assert holds in every cell, this one included); the
                // chapter split still varies the merge key across the series.
                s.deposit_as(t, d, (i / 4) as u8, i as u8, MaterialId::MUDSTONE);
            } else {
                s.erode(-d);
            }
            delivered += d;
            n_ops += 1;
            assert!(
                s.carry_m().abs() <= Q / 2.0 + f64::EPSILON,
                "op {i}: |carry| = {} > quantum/2",
                s.carry_m().abs()
            );
        }
        let closure = (s.total_m() + s.carry_m() - delivered).abs();
        assert!(
            closure <= Q * f64::from(n_ops) * 1e-9,
            "record + carry drifted {closure} m from the delivered budget"
        );
    }

    /// **The #88 tripwire: the grain axis in the key splits NOTHING while no
    /// writer sets a grain.** The grain bits are in the merge key (that is the
    /// instrument FS-A's writers are gated on), and with every unit
    /// `GRAIN_UNSET` the recorded unit count equals a count-model that ignores
    /// grain entirely. Scale-free: a per-merge predicate over a sequence.
    #[test]
    fn the_grain_axis_in_the_key_splits_nothing_while_unset() {
        let mut s = DeepStrata::default();
        let a = tag(DepEnv::Subaerial, EnergyBand::Low);
        let b = tag(DepEnv::Subsea, EnergyBand::Low);
        let seq = [
            (a, MaterialId::MUDSTONE, 0u8),
            (a, MaterialId::MUDSTONE, 0),
            (a, MaterialId::SILTSTONE, 0),
            (b, MaterialId::MUDSTONE, 1),
            (b, MaterialId::MUDSTONE, 1),
            (a, MaterialId::MUDSTONE, 1),
        ];
        // The grain-blind count model: a new unit exactly when (tag, species,
        // chapter) changes.
        let mut model = 0usize;
        let mut prev: Option<(DepTag, MaterialId, u8)> = None;
        for &(t, sp, ch) in &seq {
            s.deposit_as(t, 1.0, ch, 0, sp);
            if prev != Some((t, sp, ch)) {
                model += 1;
            }
            prev = Some((t, sp, ch));
        }
        assert!(
            s.units.iter().all(|u| u.grain() == DepUnit::GRAIN_UNSET),
            "no writer exists in slice 3, so every unit must be GRAIN_UNSET"
        );
        assert_eq!(
            s.units.len(),
            model,
            "the grain axis split the record while every unit is UNSET — \
             corrections #88's silent multiplication, caught"
        );
    }

    /// **The mover is in the merge key (M-1)**: a same-environment run
    /// delivered by a different mover is a new unit (a colluvial bed atop an
    /// alluvial bed is a contact), same-mover runs still merge, and the
    /// recorded mover reads back per unit. Joined the key AFTER measurement
    /// (1.0308× on the shipped world — the #88 rule). Scale-free.
    #[test]
    fn the_mover_is_in_the_merge_key_and_splits_only_when_it_differs() {
        let mut s = DeepStrata::default();
        let t = tag(DepEnv::Subaerial, EnergyBand::Medium);
        let sp = MaterialId::SANDSTONE;
        s.deposit_moved(t, 1.0, 0, 0, sp, 0); // fluvial
        s.deposit_moved(t, 0.5, 0, 0, sp, 0); // fluvial: merges
        assert_eq!(s.units.len(), 1, "same key + same mover must merge");
        s.deposit_moved(t, 0.25, 0, 0, sp, 3); // gravity: a colluvial contact
        assert_eq!(s.units.len(), 2, "a differing mover is a new unit (M-1)");
        assert_eq!(s.units[0].mover(), 0);
        assert_eq!(s.units[1].mover(), 3);
        assert!((s.units[0].thickness_m() - 1.5).abs() <= Q / 2.0 + f64::EPSILON);
        s.deposit_moved(t, 0.25, 0, 0, sp, 3); // gravity again: merges into the top
        assert_eq!(s.units.len(), 2);
        assert!((s.units[1].thickness_m() - 0.5).abs() <= Q / 2.0 + f64::EPSILON);
    }

    /// Erosion pops quantized history exactly and strips to bedrock; a deposit
    /// after a full strip records an unconformity — unchanged semantics under
    /// the pack, re-asserted on the packed path.
    #[test]
    fn erosion_pops_quanta_and_strips_flag_unconformity() {
        let mut s = DeepStrata::default();
        let t = tag(DepEnv::Subaerial, EnergyBand::Low);
        s.deposit_as(t, 2.0, 0, 0, MaterialId::MUDSTONE);
        s.deposit_as(
            tag(DepEnv::Subsea, EnergyBand::Low),
            1.0,
            0,
            0,
            MaterialId::MUDSTONE,
        );
        assert_eq!(s.units.len(), 2);
        s.erode(1.5); // eats the marine bed and half a metre of the first
        assert_eq!(s.units.len(), 1);
        assert!((s.total_m() - 1.5).abs() <= Q / 2.0 + f64::EPSILON);
        s.erode(10.0); // strips to bedrock; excess demand is dropped
        assert!(s.units.is_empty());
        assert_eq!(s.strips, 1);
        s.deposit_as(t, 0.5, 0, 0, MaterialId::MUDSTONE);
        assert!(
            s.units[0].unconformity(),
            "the first bed after a strip sits on an erosional surface"
        );
    }

    /// **THE DEPOSITION CLOCK'S #88 TRIPWIRE (O-2b, ruled 2026-08-04): the
    /// epoch axis is structurally OUT of the merge key, so it splits NOTHING.**
    /// A same-key run deposited across several epochs is ONE unit — the count
    /// equals an epoch-blind model — and the merged unit keeps its **bottom
    /// (first) epoch**, which is the ruled merge semantics. Scale-free: a
    /// per-merge predicate over a sequence.
    #[test]
    fn the_epoch_axis_splits_nothing_and_a_merged_run_keeps_its_first_epoch() {
        let mut s = DeepStrata::default();
        let a = tag(DepEnv::Subaerial, EnergyBand::Low);
        let b = tag(DepEnv::Subsea, EnergyBand::Low);
        // (tag, species, chapter, epoch) — epochs advance every op; the
        // first-word key changes only twice.
        let seq = [
            (a, MaterialId::MUDSTONE, 0u8, 0u8),
            (a, MaterialId::MUDSTONE, 0, 1),
            (a, MaterialId::MUDSTONE, 0, 2),
            (b, MaterialId::MUDSTONE, 0, 3),
            (b, MaterialId::MUDSTONE, 0, 7),
            (a, MaterialId::SILTSTONE, 1, 9),
        ];
        // The epoch-blind count model: a new unit exactly when the first-word
        // key (tag, species, chapter) changes.
        let mut model = 0usize;
        let mut prev: Option<(DepTag, MaterialId, u8)> = None;
        for &(t, sp, ch, ep) in &seq {
            s.deposit_as(t, 1.0, ch, ep, sp);
            if prev != Some((t, sp, ch)) {
                model += 1;
            }
            prev = Some((t, sp, ch));
        }
        assert_eq!(
            s.units.len(),
            model,
            "the epoch axis moved the unit count — it must be structurally \
             out of the merge key (corrections #88's silent multiplication)"
        );
        // Bottom epochs: the first run started at epoch 0, the marine run at
        // 3 (not 7 — the merge keeps the FIRST epoch), the siltstone at 9.
        let epochs: Vec<u8> = s.units.iter().map(|u| u.epoch()).collect();
        assert_eq!(epochs, vec![0, 3, 9], "a merged run records its start");
        // And the stamps are non-decreasing up-stack (I-4's fact, read back).
        assert!(epochs.windows(2).all(|w| w[0] <= w[1]));
    }

    /// **F4's ruled seam: the epoch is IMMUTABLE under a pedogenic overprint.**
    /// The overprint rewrites tag + chapter in place (as-built S10 behaviour)
    /// and grows the bed, but the unit's epoch stays the tick its substrate
    /// was deposited on — overprint is alteration, not deposition. The
    /// chapter/epoch disagreement this creates is the recorded stand-in whose
    /// heir is an alteration-time axis (see `set_tag_and_chapter`).
    #[test]
    fn overprint_alters_chapter_but_never_the_deposition_epoch() {
        let mut s = DeepStrata::default();
        let t = tag(DepEnv::Subaerial, EnergyBand::Low);
        s.deposit_as(t, 2.0, 1, 37, MaterialId::SANDSTONE);
        assert_eq!(s.units[0].epoch(), 37);
        let soil = DepTag {
            biota: Biofacies::Soil,
            ..t
        };
        s.overprint_top(soil, 0.5, 4, 101, MaterialId::MUDSTONE);
        assert_eq!(s.units.len(), 1, "overprint alters, it does not stack");
        assert_eq!(s.units[0].chapter(), 4, "the chapter rewrite is as-built");
        assert_eq!(
            s.units[0].epoch(),
            37,
            "the deposition epoch must survive the overprint (F4 ruling)"
        );
        assert_eq!(s.units[0].tag().biota, Biofacies::Soil);
        // A bare-bedrock overprint IS a deposition: the new unit takes the
        // current epoch.
        let mut bare = DeepStrata::default();
        bare.overprint_top(soil, 0.5, 4, 101, MaterialId::MUDSTONE);
        assert_eq!(bare.units.len(), 1);
        assert_eq!(bare.units[0].epoch(), 101);
    }
}
