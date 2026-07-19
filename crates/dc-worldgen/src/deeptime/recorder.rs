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

/// A measured depositional tag. Two units merge only when every measured axis
/// agrees (and the younger is not the first unit after an erosional strip).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DepTag {
    pub env: DepEnv,
    pub aridity: Aridity,
    pub energy: EnergyBand,
}

impl DepTag {
    /// A short human-readable code for column printouts (`Sa/H/M`, `Ss/-/L`).
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
        format!("{env}/{ar}/{en}")
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

    /// Record a net deposition of `d` metres (`d > 0`) under `tag`. Merges into
    /// the top unit when the tag agrees and the column is conformable; starts a
    /// new (unconformity-flagged) unit otherwise.
    pub fn deposit(&mut self, tag: DepTag, d: f64) {
        if !self.stripped
            && let Some(top) = self.units.last_mut()
            && top.tag == tag
        {
            top.thickness_m += d;
            return;
        }
        self.units.push(DepUnit {
            tag,
            thickness_m: d,
            unconformity: self.stripped,
        });
        self.stripped = false;
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

    /// Rough heap footprint of this record's unit vector (bytes).
    pub fn heap_bytes(&self) -> usize {
        self.units.capacity() * std::mem::size_of::<DepUnit>()
    }
}
