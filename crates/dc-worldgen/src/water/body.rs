//! The free-water **body graph** — persist bodies, derive voxels.
//!
//! water.md's hypothesis: nothing asks per frame whether water could be
//! somewhere. A body node carries the one thing no derivation can reconstruct
//! (that water *is* here, and how much), and per-voxel water is derived from
//! the body's level plus local geometry.
//!
//! ## Order-independence by construction
//!
//! Events do not *do* anything. Each event is a **commutative, idempotent
//! mutation** of graph state (a flag set, a volume added, a link inserted under
//! a canonical key, a dirty mark). Applying a batch in any order reaches the
//! same state. Then a single deterministic **fixpoint solve** — which iterates
//! bodies in ascending id and merges by lowest id — computes every level. So
//! resolution cannot depend on event arrival order, the way S9b made the
//! deep-time phases independent of the iteration driver. Proven by shuffling in
//! `tests/water.rs`.
//!
//! ## Two body characters (the ocean question)
//!
//! - [`BodyKind::Finite`] — a real volume of water in a container. Breaching it
//!   moves that volume somewhere else; it can be emptied.
//! - [`BodyKind::Pinned`] — a level-pinned reservoir (sea, inland sea, a fed
//!   river reach). Its level is set by global/deep-time state; a breach is a
//!   *source*, not a withdrawal. Breaching it cannot drain it.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use super::conn::ConnIndex;
use super::vox::VoxWorld;

pub type BodyId = u32;

/// The character distinction water.md asked S11 to exercise rather than assume.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BodyKind {
    /// Conserved volume in voxel-volumes.
    Finite,
    /// Level pinned by outside state; volume is not a withdrawal account.
    Pinned,
}

/// A persisted free-water body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    pub kind: BodyKind,
    /// The voxel this body re-binds to on reload — the only geometric anchor
    /// persisted. Everything else about *where* the water is comes from the
    /// derived connectivity index.
    pub anchor: [i32; 3],
    /// Free-water volume, voxel-volumes. Meaningless for `Pinned`.
    pub volume: f64,
    /// Solved surface elevation. Authoritative input for `Pinned`, output
    /// otherwise; persisted either way so a reload answers before any solve.
    pub level: f32,
    /// True once the body has been merged into another; kept so ids are stable.
    pub merged_into: Option<BodyId>,
}

/// A persisted spill/gate relation between two bodies that are *not* the same
/// air component (a waterfall lip, a pipe, a controlled outlet).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub a: BodyId,
    pub b: BodyId,
    /// Elevation above which `a` spills into `b`.
    pub sill: f32,
    pub open: bool,
}

/// The event vocabulary from water.md § 4, made commutative.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaterEvent {
    /// An outlet was blocked — the link stops carrying.
    OutletBlocked { a: BodyId, b: BodyId },
    /// An inlet was cut — same mutation, opposite direction, named separately
    /// because the player-facing cause differs.
    InletCut { a: BodyId, b: BodyId },
    /// New space was opened somewhere touching this body: its container
    /// hypsometry is stale.
    SpaceOpened { body: BodyId },
    /// A receiving volume reached saturation (bound regime full).
    Saturated { body: BodyId },
    /// A regime crossing moved `delta` voxel-volumes into (+) or out of (−) the
    /// free regime for this body.
    RegimeCross { body: BodyId, delta: f64 },
    /// Two bodies were connected at `sill`.
    Breach { a: BodyId, b: BodyId, sill: f32 },
}

/// The persisted half of the world's water state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BodyGraph {
    pub bodies: Vec<Body>,
    pub links: Vec<Link>,
    next_id: BodyId,
}

/// Instrumentation from one `resolve`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ResolveStats {
    pub events: usize,
    pub fixpoint_rounds: u32,
    pub bodies_touched: usize,
    pub merges: usize,
    pub transits: usize,
}

impl BodyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, kind: BodyKind, anchor: [i32; 3], volume: f64, level: f32) -> BodyId {
        let id = self.next_id;
        self.next_id += 1;
        self.bodies.push(Body {
            id,
            kind,
            anchor,
            volume,
            level,
            merged_into: None,
        });
        id
    }

    pub fn get(&self, id: BodyId) -> &Body {
        &self.bodies[id as usize]
    }

    /// Follow merges to the surviving body.
    pub fn resolve_id(&self, mut id: BodyId) -> BodyId {
        while let Some(n) = self.bodies[id as usize].merged_into {
            id = n;
        }
        id
    }

    /// Live (unmerged) node count — the number the sparsity question is about.
    pub fn live_nodes(&self) -> usize {
        self.bodies
            .iter()
            .filter(|b| b.merged_into.is_none())
            .count()
    }

    pub fn live_links(&self) -> usize {
        self.links.len()
    }

    /// Get-or-create a link under its canonical key.
    ///
    /// Creation is part of what makes the event batch commutative: an
    /// `OutletBlocked` naming a link a `Breach` in the *same batch* has not
    /// created yet must not silently no-op, or the two orders diverge. Both
    /// events therefore ensure the link exists, and then perform mutations that
    /// are individually monotone (`open` only ever falls; `sill` only ever
    /// falls), so any interleaving reaches the same state.
    fn ensure_link(&mut self, a: BodyId, b: BodyId, sill: f32, open: bool) -> usize {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        if let Some(i) = self.links.iter().position(|l| l.a == lo && l.b == hi) {
            return i;
        }
        self.links.push(Link {
            a: lo,
            b: hi,
            sill,
            open,
        });
        self.links.len() - 1
    }

    /// Apply one event as a commutative state mutation. Returns the bodies made
    /// dirty.
    fn apply(&mut self, ev: WaterEvent, dirty: &mut BTreeSet<BodyId>) {
        match ev {
            WaterEvent::OutletBlocked { a, b } | WaterEvent::InletCut { a, b } => {
                let (ra, rb) = (self.resolve_id(a), self.resolve_id(b));
                if ra != rb {
                    let i = self.ensure_link(ra, rb, f32::INFINITY, false);
                    self.links[i].open = false;
                }
                dirty.insert(ra);
                dirty.insert(rb);
            }
            WaterEvent::SpaceOpened { body } | WaterEvent::Saturated { body } => {
                dirty.insert(self.resolve_id(body));
            }
            // NOTE: RegimeCross is NOT applied here. Float addition commutes
            // but does not associate, so summing several crossings into one
            // body in arrival order would make the low bits depend on that
            // order. They are accumulated and summed in a canonical order by
            // `resolve` instead. See the comment there.
            WaterEvent::RegimeCross { body, .. } => {
                dirty.insert(self.resolve_id(body));
            }
            WaterEvent::Breach { a, b, sill } => {
                let (ra, rb) = (self.resolve_id(a), self.resolve_id(b));
                if ra != rb {
                    let i = self.ensure_link(ra, rb, sill, true);
                    // Monotone: the lowest sill named for this pair wins, so
                    // two breaches of the same pair commute.
                    self.links[i].sill = self.links[i].sill.min(sill);
                }
                dirty.insert(ra);
                dirty.insert(rb);
            }
        }
    }

    /// Apply a batch of events, then solve to a fixpoint.
    ///
    /// `hyps` maps a component root → its air-voxel-count-by-y histogram
    /// (derived from geometry; see [`ConnIndex::hypsometry`]). `comp_of` maps a
    /// body to its component root.
    pub fn resolve(
        &mut self,
        events: &[WaterEvent],
        comp_of: &HashMap<BodyId, u32>,
        hyps: &HashMap<u32, Vec<u32>>,
    ) -> ResolveStats {
        let mut stats = ResolveStats {
            events: events.len(),
            ..Default::default()
        };
        let mut dirty: BTreeSet<BodyId> = BTreeSet::new();
        for ev in events {
            self.apply(*ev, &mut dirty);
        }
        // Regime crossings, summed in a canonical order. Grouping by body id
        // (BTreeMap) and sorting each group's deltas by their bit pattern makes
        // the sum a pure function of the *set* of events, not of their arrival
        // order — the same discipline S9b applied to the deep-time phases when
        // it made the parallel driver byte-identical to the scalar one.
        let mut crossings: BTreeMap<BodyId, Vec<f64>> = BTreeMap::new();
        for ev in events {
            if let WaterEvent::RegimeCross { body, delta } = *ev {
                crossings
                    .entry(self.resolve_id(body))
                    .or_default()
                    .push(delta);
            }
        }
        for (id, mut deltas) in crossings {
            deltas.sort_by(|a, b| a.total_cmp(b));
            let mut acc = 0.0f64;
            for d in deltas {
                acc += d;
            }
            self.bodies[id as usize].volume += acc;
            dirty.insert(id);
        }

        // --- Phase 1: same air component ⇒ one body. Merge by lowest id.
        // Deterministic: BTreeMap iteration is by component root, and the
        // survivor is always the minimum id.
        let mut by_comp: BTreeMap<u32, Vec<BodyId>> = BTreeMap::new();
        for b in &self.bodies {
            if b.merged_into.is_none()
                && let Some(c) = comp_of.get(&b.id)
            {
                by_comp.entry(*c).or_default().push(b.id);
            }
        }
        for ids in by_comp.values() {
            if ids.len() < 2 {
                continue;
            }
            let survivor = *ids.iter().min().expect("non-empty");
            // A pinned member makes the whole merged body pinned — a sea that
            // absorbs a cave pool is still a sea.
            let pinned = ids
                .iter()
                .any(|i| self.bodies[*i as usize].kind == BodyKind::Pinned);
            let pinned_level = ids
                .iter()
                .filter(|i| self.bodies[**i as usize].kind == BodyKind::Pinned)
                .map(|i| self.bodies[*i as usize].level)
                .fold(f32::NEG_INFINITY, f32::max);
            for &id in ids {
                if id == survivor {
                    continue;
                }
                let v = self.bodies[id as usize].volume;
                self.bodies[id as usize].merged_into = Some(survivor);
                self.bodies[survivor as usize].volume += v;
                stats.merges += 1;
            }
            if pinned {
                self.bodies[survivor as usize].kind = BodyKind::Pinned;
                self.bodies[survivor as usize].level = pinned_level;
            }
            dirty.insert(survivor);
        }
        // Re-point links at survivors and drop self-links.
        for i in 0..self.links.len() {
            let a = self.resolve_id(self.links[i].a);
            let b = self.resolve_id(self.links[i].b);
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            self.links[i].a = lo;
            self.links[i].b = hi;
        }
        self.links.retain(|l| l.a != l.b);
        self.links.sort_by_key(|l| (l.a, l.b));
        self.links.dedup_by_key(|l| (l.a, l.b));

        // --- Phase 2: fixpoint level solve with spill transit along open links.
        let live: Vec<BodyId> = self
            .bodies
            .iter()
            .filter(|b| b.merged_into.is_none())
            .map(|b| b.id)
            .collect();
        let mut rounds = 0u32;
        loop {
            rounds += 1;
            let mut changed = false;
            for &id in &live {
                let (kind, vol) = {
                    let b = &self.bodies[id as usize];
                    (b.kind, b.volume)
                };
                if kind == BodyKind::Pinned {
                    continue;
                }
                let hyp = comp_of.get(&id).and_then(|c| hyps.get(c));
                let lvl = match hyp {
                    Some(h) => level_for_volume(h, vol),
                    None => self.bodies[id as usize].level,
                };
                if (lvl - self.bodies[id as usize].level).abs() > 1e-6 {
                    self.bodies[id as usize].level = lvl;
                    changed = true;
                    stats.bodies_touched += 1;
                }
                // Spill: anything above an open link's sill transits.
                let outs: Vec<(usize, BodyId, f32)> = self
                    .links
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.open && (l.a == id || l.b == id))
                    .map(|(i, l)| (i, if l.a == id { l.b } else { l.a }, l.sill))
                    .collect();
                for (_, other, sill) in outs {
                    let b = &self.bodies[id as usize];
                    if b.level <= sill {
                        continue;
                    }
                    // Water only spills DOWNHILL. Without this the link is
                    // symmetric and a chain of pools sloshes volume back and
                    // forth forever — the first version of this solve hit the
                    // 512-round cap on a 10-body chain and logged 8 742
                    // transits for what should be 9.
                    let other_r = self.resolve_id(other);
                    if self.bodies[other_r as usize].level >= sill {
                        continue;
                    }
                    let Some(h) = hyp else { continue };
                    let cap_at_sill = volume_below(h, sill);
                    let excess = b.volume - cap_at_sill;
                    if excess <= 1e-9 {
                        continue;
                    }
                    let other = self.resolve_id(other);
                    if other == id {
                        continue;
                    }
                    self.bodies[id as usize].volume -= excess;
                    self.bodies[id as usize].level = sill;
                    if self.bodies[other as usize].kind == BodyKind::Finite {
                        self.bodies[other as usize].volume += excess;
                    }
                    stats.transits += 1;
                    changed = true;
                }
            }
            if !changed || rounds > 512 {
                break;
            }
        }
        stats.fixpoint_rounds = rounds;
        stats
    }

    /// Derive per-voxel water: is voxel `(x, y, z)` wet?
    ///
    /// The whole hypothesis in one function — O(1), no search, no history.
    pub fn water_at(
        &self,
        conn: &mut ConnIndex,
        body_of_comp: &HashMap<u32, BodyId>,
        x: usize,
        y: usize,
        z: usize,
    ) -> bool {
        let Some(c) = conn.component(x, y, z) else {
            return false;
        };
        let Some(&bid) = body_of_comp.get(&c) else {
            return false;
        };
        let b = &self.bodies[self.resolve_id(bid) as usize];
        (y as f32) < b.level
    }

    /// Derive the whole wet set — used by the unload/reload identity proof.
    pub fn derive_wet(
        &self,
        w: &VoxWorld,
        conn: &mut ConnIndex,
        body_of_comp: &HashMap<u32, BodyId>,
    ) -> Vec<u64> {
        let n = w.cells();
        let mut bits = vec![0u64; n.div_ceil(64)];
        for y in 0..w.dy {
            for z in 0..w.dz {
                for x in 0..w.dx {
                    if self.water_at(conn, body_of_comp, x, y, z) {
                        let i = w.idx(x, y, z);
                        bits[i >> 6] |= 1u64 << (i & 63);
                    }
                }
            }
        }
        bits
    }

    /// Persisted bytes (postcard, the project's wire format).
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("body graph serializes")
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        postcard::from_bytes(b).expect("body graph deserializes")
    }
}

/// Volume of container below `level`, from an air-voxel-per-y histogram.
pub fn volume_below(hyp: &[u32], level: f32) -> f64 {
    let mut v = 0.0;
    for (y, &c) in hyp.iter().enumerate() {
        let top = (y + 1) as f32;
        if level >= top {
            v += c as f64;
        } else if level > y as f32 {
            v += c as f64 * (level - y as f32) as f64;
        }
    }
    v
}

/// Level at which the container holds exactly `vol`. Monotone and exact at
/// voxel granularity; the partial layer is linear.
pub fn level_for_volume(hyp: &[u32], vol: f64) -> f32 {
    if vol <= 0.0 {
        return 0.0;
    }
    let mut acc = 0.0f64;
    for (y, &c) in hyp.iter().enumerate() {
        if c == 0 {
            continue;
        }
        if acc + c as f64 >= vol {
            let frac = (vol - acc) / c as f64;
            return y as f32 + frac as f32;
        }
        acc += c as f64;
    }
    hyp.len() as f32
}
