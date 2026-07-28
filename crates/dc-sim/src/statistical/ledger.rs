//! The constraint ledger: an append-only log of committed facts.
//!
//! A **committed fact** is any piece of world state that has leaked to an
//! observer — visiting in person, a traveler's report, a letter. Once
//! committed it is immutable forever; all fluid state derived afterwards must
//! be consistent with every fact in the ledger.
//!
//! Schema (see docs/spikes/S2-results.md for the design discussion):
//! - a fact is `(seq, time, subject, aspect, value)`;
//! - `time` is the *sim time the fact is about*, not the time it was
//!   committed — a traveler's report commits facts about the past;
//! - `aspect` allows **partial** observation: "agent 12 was alive at t=10"
//!   commits strictly less than "agent 12 was farming at t=10", so later
//!   observations can narrow (but never contradict) earlier ones;
//! - `seq` is the append index; it never changes and gives the ledger a
//!   canonical content hash used to seed collapses.
//!
//! The ledger itself performs only *structural* validation on append (aspect/
//! value pairing, exact-key conflicts, alive-vs-behavior implication). Whether
//! a fact is consistent with the ledger *under the dynamics* is the engine's
//! job (see `engine::force_fact`).

use serde::{Deserialize, Serialize};

use super::rng::mix;
use super::world::{AgentId, AgentState, Behavior, RegionId, Tick};

/// What a fact is about.
///
/// `Agent`/`Region` are the S2 toy-world subjects.
///
/// **`Site`/`Polity` HAVE NO PRODUCER SINCE 2026-07-28** (journal/0121). They
/// were added by S7 so dc-worldgen's settlement-history pass could commit
/// pregen facts into the same ledger the live sim reads — "one system, no seam
/// at year zero". That pass was **removed** as unratified bootstrap content
/// (there is no evo/socia/civ model behind it, even at the design stage), and
/// nothing else ever constructed these variants outside tests. They are kept
/// only because deleting a serialized enum's variants is a wider change than
/// that removal was scoped for; **do not read this as a live schema**, and see
/// spines § 3 for the standing question of whether they survive at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Subject {
    Agent(AgentId),
    Region(RegionId),
    /// A settlement site (S7 worldgen history).
    Site(u32),
    /// A polity — a people/state owning sites (S7 worldgen history).
    Polity(u32),
}

impl Subject {
    pub(crate) fn key(self) -> u64 {
        match self {
            Subject::Agent(a) => (1 << 32) | u64::from(a),
            Subject::Region(r) => (2 << 32) | u64::from(r),
            Subject::Site(s) => (3 << 32) | u64::from(s),
            Subject::Polity(p) => (4 << 32) | u64::from(p),
        }
    }
}

/// Which slice of the subject's state was observed. Partial aspects
/// (`AgentAlive`) commit less than full ones (`AgentBehavior`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Aspect {
    /// Which region an agent is physically in.
    AgentRegion,
    /// The agent's full behavior state.
    AgentBehavior,
    /// Whether the agent is alive (partial view of behavior).
    AgentAlive,
    /// A region's hostile-mob pressure level.
    RegionPressure,
    /// Whether a site is settled (S7). Founding commits `Exists(true)`;
    /// abandonment commits `Exists(false)` at a strictly later time.
    SiteExists,
    /// Which polity holds a site (S7).
    SitePolity,
    /// A notable event at a site in a given epoch (S7).
    SiteEvent,
    /// How many cells a polity holds in a given epoch (S7).
    PolityExtent,
}

impl Aspect {
    pub(crate) fn key(self) -> u64 {
        match self {
            Aspect::AgentRegion => 1,
            Aspect::AgentBehavior => 2,
            Aspect::AgentAlive => 3,
            Aspect::RegionPressure => 4,
            Aspect::SiteExists => 5,
            Aspect::SitePolity => 6,
            Aspect::SiteEvent => 7,
            Aspect::PolityExtent => 8,
        }
    }
}

/// Kind of notable site event (S7 worldgen history).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SiteEventKind {
    Founded,
    Sacked,
    Abandoned,
}

impl SiteEventKind {
    pub(crate) fn key(self) -> u64 {
        match self {
            SiteEventKind::Founded => 0,
            SiteEventKind::Sacked => 1,
            SiteEventKind::Abandoned => 2,
        }
    }
}

/// The observed value of an aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Value {
    Region(RegionId),
    Behavior(Behavior),
    Alive(bool),
    Pressure(u8),
    /// Site settled or not (S7).
    Exists(bool),
    /// Owning polity id (S7).
    PolityRef(u32),
    /// Notable event kind (S7).
    Event(SiteEventKind),
    /// Cells held (S7).
    Extent(u32),
}

impl Value {
    pub(crate) fn key(self) -> u64 {
        match self {
            Value::Region(r) => (1 << 16) | u64::from(r),
            Value::Behavior(b) => (2 << 16) | b.key(),
            Value::Alive(a) => (3 << 16) | u64::from(a),
            Value::Pressure(p) => (4 << 16) | u64::from(p),
            // S7 additions use a 32-bit payload field so u32 payloads cannot
            // bleed into the tag bits. Existing variants keep their original
            // shape (changing them would change committed content hashes).
            Value::Exists(e) => (5 << 32) | u64::from(e),
            Value::PolityRef(p) => (6 << 32) | u64::from(p),
            Value::Event(k) => (7 << 32) | k.key(),
            Value::Extent(n) => (8 << 32) | u64::from(n),
        }
    }

    fn matches_aspect(self, aspect: Aspect) -> bool {
        matches!(
            (aspect, self),
            (Aspect::AgentRegion, Value::Region(_))
                | (Aspect::AgentBehavior, Value::Behavior(_))
                | (Aspect::AgentAlive, Value::Alive(_))
                | (Aspect::RegionPressure, Value::Pressure(_))
                | (Aspect::SiteExists, Value::Exists(_))
                | (Aspect::SitePolity, Value::PolityRef(_))
                | (Aspect::SiteEvent, Value::Event(_))
                | (Aspect::PolityExtent, Value::Extent(_))
        )
    }
}

/// One committed, immutable fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// Append index — assigned by the ledger, never reused.
    pub seq: u64,
    /// The sim time this fact is *about* (not the commit time).
    pub time: Tick,
    pub subject: Subject,
    pub aspect: Aspect,
    pub value: Value,
}

impl Fact {
    /// Does a concrete agent state satisfy this fact? (Only meaningful for
    /// agent-subject facts.)
    pub fn satisfied_by_agent(&self, state: &AgentState) -> bool {
        match (self.aspect, self.value) {
            (Aspect::AgentRegion, Value::Region(r)) => state.region == r,
            (Aspect::AgentBehavior, Value::Behavior(b)) => state.behavior == b,
            (Aspect::AgentAlive, Value::Alive(a)) => state.is_alive() == a,
            _ => true,
        }
    }

    /// Does a concrete pressure level satisfy this fact? (Region facts only.)
    pub fn satisfied_by_pressure(&self, level: u8) -> bool {
        match (self.aspect, self.value) {
            (Aspect::RegionPressure, Value::Pressure(p)) => level == p,
            _ => true,
        }
    }

    fn content_key(&self) -> u64 {
        mix(&[
            u64::from(self.time),
            self.subject.key(),
            self.aspect.key(),
            self.value.key(),
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LedgerError {
    #[error("value {value:?} does not belong to aspect {aspect:?}")]
    AspectValueMismatch { aspect: Aspect, value: Value },
    #[error(
        "conflicting fact for {subject:?} {aspect:?} at t={time}: committed {committed:?}, proposed {proposed:?}"
    )]
    ConflictsWithCommitted {
        subject: Subject,
        aspect: Aspect,
        time: Tick,
        committed: Value,
        proposed: Value,
    },
}

/// Outcome of an append attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendOutcome {
    /// The fact was appended with this sequence number.
    Appended(u64),
    /// An identical fact was already committed; the ledger is unchanged.
    AlreadyCommitted(u64),
}

/// Append-only log of committed facts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    facts: Vec<Fact>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Exact-key lookup: the committed value for `(subject, time, aspect)`.
    pub fn get(&self, subject: Subject, time: Tick, aspect: Aspect) -> Option<&Fact> {
        self.facts
            .iter()
            .find(|f| f.subject == subject && f.time == time && f.aspect == aspect)
    }

    /// All facts about `subject` at `time`.
    pub fn facts_about(&self, subject: Subject, time: Tick) -> impl Iterator<Item = &Fact> {
        self.facts
            .iter()
            .filter(move |f| f.subject == subject && f.time == time)
    }

    /// Order-sensitive hash of the full ledger content. Seeds collapses:
    /// same seed + same ledger content => same collapse result.
    pub fn content_hash(&self) -> u64 {
        let mut h = 0x5EED_1ED6_E200_0000u64;
        for f in &self.facts {
            h = mix(&[h, f.content_key()]);
        }
        h
    }

    /// Append a fact after structural validation. Appending an *identical*
    /// fact is an idempotent no-op; a conflicting fact for an already
    /// committed key (or one structurally contradicting a committed fact
    /// about the same subject-time, e.g. `Alive(true)` vs `Behavior(Dead)`)
    /// is rejected and the ledger is left untouched — committed facts are
    /// immutable, so contradictions must bounce off, never overwrite.
    pub fn append(
        &mut self,
        time: Tick,
        subject: Subject,
        aspect: Aspect,
        value: Value,
    ) -> Result<AppendOutcome, LedgerError> {
        if !value.matches_aspect(aspect) {
            return Err(LedgerError::AspectValueMismatch { aspect, value });
        }
        if let Some(existing) = self.get(subject, time, aspect) {
            if existing.value == value {
                return Ok(AppendOutcome::AlreadyCommitted(existing.seq));
            }
            return Err(LedgerError::ConflictsWithCommitted {
                subject,
                aspect,
                time,
                committed: existing.value,
                proposed: value,
            });
        }
        // Cross-aspect structural implication: Alive <-> Behavior at the same
        // subject-time must agree.
        for f in self.facts_about(subject, time) {
            let contradicts = match (f.value, value) {
                (Value::Alive(a), Value::Behavior(b)) | (Value::Behavior(b), Value::Alive(a)) => {
                    (b != Behavior::Dead) != a
                }
                _ => false,
            };
            if contradicts {
                return Err(LedgerError::ConflictsWithCommitted {
                    subject,
                    aspect: f.aspect,
                    time,
                    committed: f.value,
                    proposed: value,
                });
            }
        }
        let seq = self.facts.len() as u64;
        self.facts.push(Fact {
            seq,
            time,
            subject,
            aspect,
            value,
        });
        Ok(AppendOutcome::Appended(seq))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_is_idempotent_and_conflicts_bounce() {
        let mut l = Ledger::new();
        let s = Subject::Agent(3);
        let v = Value::Behavior(Behavior::Farming);
        assert_eq!(
            l.append(5, s, Aspect::AgentBehavior, v),
            Ok(AppendOutcome::Appended(0))
        );
        assert_eq!(
            l.append(5, s, Aspect::AgentBehavior, v),
            Ok(AppendOutcome::AlreadyCommitted(0))
        );
        assert_eq!(l.len(), 1);
        let err = l
            .append(5, s, Aspect::AgentBehavior, Value::Behavior(Behavior::Dead))
            .unwrap_err();
        assert!(matches!(err, LedgerError::ConflictsWithCommitted { .. }));
        assert_eq!(l.len(), 1);
    }

    #[test]
    fn cross_aspect_implication_is_enforced() {
        let mut l = Ledger::new();
        let s = Subject::Agent(9);
        l.append(2, s, Aspect::AgentAlive, Value::Alive(true))
            .unwrap();
        let err = l
            .append(2, s, Aspect::AgentBehavior, Value::Behavior(Behavior::Dead))
            .unwrap_err();
        assert!(matches!(err, LedgerError::ConflictsWithCommitted { .. }));
        // A live behavior is a legal narrowing.
        l.append(
            2,
            s,
            Aspect::AgentBehavior,
            Value::Behavior(Behavior::Fortifying),
        )
        .unwrap();
        assert_eq!(l.len(), 2);
    }

    #[test]
    fn worldgen_fact_kinds_append_and_conflict() {
        let mut l = Ledger::new();
        let s = Subject::Site(7);
        l.append(0, s, Aspect::SiteExists, Value::Exists(true))
            .unwrap();
        l.append(
            0,
            s,
            Aspect::SiteEvent,
            Value::Event(SiteEventKind::Founded),
        )
        .unwrap();
        l.append(5, s, Aspect::SiteExists, Value::Exists(false))
            .unwrap();
        l.append(
            3,
            Subject::Polity(1),
            Aspect::PolityExtent,
            Value::Extent(4),
        )
        .unwrap();
        // Aspect/value pairing is enforced for the new kinds too.
        let err = l
            .append(1, s, Aspect::SiteExists, Value::Pressure(1))
            .unwrap_err();
        assert!(matches!(err, LedgerError::AspectValueMismatch { .. }));
        // Exact-key conflicts bounce.
        let err = l
            .append(5, s, Aspect::SiteExists, Value::Exists(true))
            .unwrap_err();
        assert!(matches!(err, LedgerError::ConflictsWithCommitted { .. }));
        assert_eq!(l.len(), 4);
    }

    #[test]
    fn content_hash_tracks_content() {
        let mut a = Ledger::new();
        let mut b = Ledger::new();
        assert_eq!(a.content_hash(), b.content_hash());
        a.append(
            1,
            Subject::Region(2),
            Aspect::RegionPressure,
            Value::Pressure(1),
        )
        .unwrap();
        assert_ne!(a.content_hash(), b.content_hash());
        b.append(
            1,
            Subject::Region(2),
            Aspect::RegionPressure,
            Value::Pressure(1),
        )
        .unwrap();
        assert_eq!(a.content_hash(), b.content_hash());
    }
}
