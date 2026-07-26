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

use super::geotherm::{self, BurialColumn};
use super::lithology::{Litho, litho_of_tag};

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

/// One recorded unit: a measured tag, its accumulated thickness (metres), and
/// whether it sits on an erosional surface (a proto-unconformity).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DepUnit {
    pub tag: DepTag,
    pub thickness_m: f64,
    /// True when this unit was deposited directly onto bedrock the column had
    /// been stripped to since the previous unit — a time gap you can see and
    /// reason about (earth-processes.md § 7, structural deformation of the
    /// record). Recorded as a first-class property of the overlying unit.
    pub unconformity: bool,
    /// The **tectonic chapter** (0-based) this unit was deposited in
    /// (tectonics.md § 3.3). A *measurement* (when), like every other tag axis,
    /// and the age label the Phanerozoic register requires: chapter `c` spans
    /// `[500 − c·(500/K), 500 − (c+1)·(500/K)]` Myr before present. It joins the
    /// merge key, so units do not merge across a chapter boundary (a real time
    /// surface). Always `0` when [`super::grid::DeepConfig::tectonic_history`] is
    /// off — every deposit passes chapter `0`, so `0 == 0` keeps the merge and
    /// therefore the record byte-identical to the pre-tectonic-history path.
    /// Appended last (wire discipline — corrections #3): fits `DepUnit`'s
    /// existing 8-byte padding, so `sizeof` is unchanged (verified in the spike).
    pub chapter: u8,
    /// **The material that actually arrived here** (Movement 2b, `material-behavior.md`
    /// § 13.3/§ 13.7).
    ///
    /// Every other axis of a unit is a *measurement of the environment* at the
    /// moment of deposition; this one is a measurement of **the load**. Before
    /// material-aware transport the record had no such axis and every consumer
    /// inferred the rock from the environment — [`litho_of_tag`], the
    /// *"`DepTag → reference_material` shortcut"* § 13.7 says half-dissolves. The
    /// inference is not wrong, it is just **blind to provenance**: it cannot know
    /// that the flow reaching a distal cell has no gravel left to drop, because
    /// the gravel rained out at the mountain front twenty cells upstream.
    ///
    /// With [`super::grid::DeepConfig::material_transport`] **off** this is
    /// exactly `litho_of_tag(tag)` at every construction site — a pure function of
    /// `tag`, so it adds nothing to the merge key and the record is byte-identical.
    /// With it **on**, transport deposition overrides it with the argmax species of
    /// what settled, and the unit's identity stops being derivable from its
    /// environment.
    ///
    /// Appended last (wire discipline — corrections #3); it fills `DepUnit`'s
    /// existing padding, so `size_of::<DepUnit>()` is unchanged (asserted).
    pub species: Litho,
}

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
}

impl DeepStrata {
    /// Total recorded thickness (metres) — must equal `H` at every point in the
    /// run (the finalize invariant, tested).
    pub fn total_m(&self) -> f64 {
        self.units.iter().map(|u| u.thickness_m).sum()
    }

    /// Record a net deposition of `d` metres (`d > 0`) under `tag`, deposited in
    /// tectonic `chapter`. Merges into the top unit when the tag **and chapter**
    /// agree and the column is conformable; starts a new (unconformity-flagged)
    /// unit otherwise. `chapter` is `0` on the pre-tectonic-history path, so the
    /// extra `top.chapter == chapter` guard is always satisfied and merging is
    /// byte-identical to before.
    pub fn deposit(&mut self, tag: DepTag, d: f64, chapter: u8) {
        self.deposit_as(tag, d, chapter, litho_of_tag(tag));
    }

    /// [`Self::deposit`], but stating **which material arrived** rather than
    /// letting it be inferred from the tag (Movement 2b, `material-behavior.md`
    /// § 13.3).
    ///
    /// Only an agent that actually *carried* a load can answer that question, so
    /// only the material-aware fluvial transport pass calls this; every other
    /// depositor — the wind agent, the wave agent, the biotic layer, the tests —
    /// goes through [`Self::deposit`] and gets the tag-derived default, which is
    /// what the record has always said. (The **eolian** family genuinely has a load
    /// too and would honestly travel its own identity; it is deferred with the rest
    /// of § 13.2's wind/ice/gravity family and still reads its species off the tag.)
    ///
    /// `species` joins the merge key: two runs of the same environment that
    /// delivered *different rock* are two units, not one. That is the whole point —
    /// a sand sheet and the mud that followed it at the same tag are a contact you
    /// can see in a cliff.
    pub fn deposit_as(&mut self, tag: DepTag, d: f64, chapter: u8, species: Litho) {
        if !self.stripped
            && let Some(top) = self.units.last_mut()
            && top.tag == tag
            && top.chapter == chapter
            && top.species == species
        {
            top.thickness_m += d;
            return;
        }
        self.units.push(DepUnit {
            tag,
            thickness_m: d,
            unconformity: self.stripped,
            chapter,
            species,
        });
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
    pub fn overprint_top(&mut self, tag: DepTag, extra: f64, chapter: u8) {
        let charcoal_top = self
            .units
            .last()
            .is_some_and(|u| u.tag.biota == Biofacies::Charcoal);
        if self.units.is_empty() || charcoal_top {
            // Bare bedrock, or a fire bed we must not overwrite: start a unit.
            self.units.push(DepUnit {
                tag,
                thickness_m: extra,
                unconformity: self.stripped,
                chapter,
                species: litho_of_tag(tag),
            });
            self.stripped = false;
            return;
        }
        let top = self.units.last_mut().expect("non-empty");
        top.thickness_m += extra;
        top.tag = tag;
        top.chapter = chapter;
        // Pedogenesis **alters the material in place** — the horizon a community
        // built out of what was lying here is an organic soil whatever the flow
        // delivered, so the overprint takes the tag's own species and the
        // transported identity is genuinely overwritten rather than lost.
        top.species = litho_of_tag(tag);
        // Merge down into an identically-tagged predecessor of the same chapter.
        let n = self.units.len();
        if n >= 2
            && self.units[n - 2].tag == tag
            && self.units[n - 2].chapter == chapter
            && self.units[n - 2].species == litho_of_tag(tag)
            && !self.units[n - 1].unconformity
        {
            let t = self.units.pop().expect("non-empty").thickness_m;
            self.units.last_mut().expect("non-empty").thickness_m += t;
        }
    }

    /// Pop `amount` metres of recorded history off the top (erosion). When the
    /// record empties, the column is stripped to bedrock and the next deposit
    /// will record an unconformity.
    pub fn erode(&mut self, mut amount: f64) {
        while amount > 0.0 {
            let Some(top) = self.units.last_mut() else {
                break;
            };
            // Exact bookkeeping (no epsilon fudge): the record must mirror `H`
            // to the metre so the finalize invariant holds tightly.
            if top.thickness_m <= amount {
                amount -= top.thickness_m;
                self.units.pop();
            } else {
                top.thickness_m -= amount;
                amount = 0.0;
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
            if !seen.contains(&u.tag) {
                seen.push(u.tag);
            }
        }
        seen.len()
    }

    /// Count of unconformable contacts (units flagged as sitting on an
    /// erosional surface) — the readable time gaps.
    pub fn unconformities(&self) -> usize {
        self.units.iter().filter(|u| u.unconformity).count()
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
            .filter(|(k, u)| *k != last && u.tag.biota.is_organic())
            .count()
    }

    /// Count of coal seams (units tagged [`Biofacies::Coal`]) at or above
    /// `min_m` thickness — the legible ones.
    pub fn coal_seams(&self, min_m: f64) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag.biota == Biofacies::Coal && u.thickness_m >= min_m)
            .count()
    }

    /// Count of charcoal bands (fire-event units) in the record.
    pub fn charcoal_bands(&self) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag.biota == Biofacies::Charcoal)
            .count()
    }

    /// Count of retrogressive organic horizons (units tagged
    /// [`Biofacies::Retro`]) — collapsed, P-starved surfaces.
    pub fn retro_surfaces(&self) -> usize {
        self.units
            .iter()
            .filter(|u| u.tag.biota == Biofacies::Retro)
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
    /// facies can one day split by rank (a later slice; not built here).
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
    pub fn promote_coal(&mut self, col: BurialColumn, onset_c: f64) {
        // Top-down, accumulating overburden: one pass, no allocation.
        let top = self.units.len().saturating_sub(1);
        let mut overburden_m = 0.0f64;
        for (k, u) in self.units.iter_mut().enumerate().rev() {
            if k != top && u.tag.biota == Biofacies::Peat {
                // Burial depth to the slab's mid-point (the recorder residual the
                // geotherm integrates a linear gradient across).
                let depth_m = overburden_m + 0.5 * u.thickness_m;
                let t_c =
                    geotherm::temperature_c(col.surface_temp_c, col.gradient_c_per_m, depth_m);
                if t_c >= onset_c {
                    u.tag.biota = Biofacies::Coal;
                    // Diagenesis is a **material transformation**: the unit stops
                    // being peat, so its species follows its tag. (A peat unit's
                    // species is always the tag-derived one — peat is laid by the
                    // biotic layer, which does not carry a load.)
                    u.species = litho_of_tag(u.tag);
                }
            }
            overburden_m += u.thickness_m;
        }
    }

    /// Rough heap footprint of this record's unit vector (bytes).
    pub fn heap_bytes(&self) -> usize {
        self.units.capacity() * std::mem::size_of::<DepUnit>()
    }
}
