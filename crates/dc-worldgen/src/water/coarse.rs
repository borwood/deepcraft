//! **S15 spike** — coarse capacity: volume↔level without walking the container.
//!
//! S11 answered "persist bodies, derive voxels" against a toy fully-resident
//! volume ([`super::vox`]). Against the *real* world the missing half is the
//! capacity curve: "walk the container to learn volume↔level" is a generation
//! storm through a bounded-LRU chunk store (journal/0051), and the storm evicts
//! the ground the player is standing on while they dig.
//!
//! The mechanism measured here is the user's proposal (water.md § SPIKE SPEC
//! S15): *"if cells / the coarse regions know roughly their level (and remember
//! if it changes — remembering player edits) then that math could be simpler."*
//!
//! Capacity is **additive**, so the body's curve is a sum over coarse cells:
//!
//! ```text
//! V(L) = Σ_cells [ Σ_columns max(0, L − floor(column))     ← the hypsometric summary
//!                + Σ_edits  delta_y · clamp(L − y, 0, 1) ] ← the player's memory
//! ```
//!
//! Two properties make that cheap where a voxel walk is expensive:
//!
//! - the per-column floor comes from
//!   [`crate::WorldGenerator::coarse_surface`], which is ~1.377 µs/column,
//!   memoized, and **generates no chunks** (journal/0022) — and it is
//!   sub-sampled, so a cell costs [`CAP_SUBSAMPLE`]² samples for
//!   [`CAP_CELL`]² columns;
//! - an edit is a **delta hung on the audited write path** (dc-api's
//!   `set_block_raw`, journal/0051), not a rescan.
//!
//! Nothing in the production path calls this module — the S11 scope discipline,
//! unchanged. Results: `docs/spikes/S15-results.md`.

use std::collections::BTreeMap;

/// Columns per coarse capacity cell edge. 32 = one chunk footprint (28.8 m at
/// the N=2 scale), so a capacity cell is exactly the unit the chunk store
/// generates and evicts.
pub const CAP_CELL: i64 = 32;

/// Sub-samples per cell edge. 4 → 16 samples standing in for 1 024 columns
/// (1.6 %), which is the compression whose level cost group 1 measures.
pub const CAP_SUBSAMPLE: i64 = 4;

/// One coarse cell's hypsometric summary, plus its memory of player edits.
///
/// `counts[i]` is how many of the cell's columns have their **floor plane** —
/// the y of the lowest air voxel, i.e. `surface_height + 1` — at `h0 + i`.
/// Sub-sampled: each sample stands for `(edge/sub)²` columns, so the counts sum
/// to the cell's full column count and capacity is in true voxel volumes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapCell {
    /// Lowest sampled floor plane in this cell.
    pub h0: i32,
    /// Columns per floor plane, from `h0` upward.
    pub counts: Vec<u32>,
    /// Signed open-voxel deltas from player edits, by y. `+1` = a solid became
    /// air (capacity gained), `-1` = air became solid (capacity lost).
    ///
    /// Integer counts, not a float accumulator: this is why the incremental
    /// path cannot drift (group 2).
    pub edits: BTreeMap<i32, i64>,
}

impl CapCell {
    /// Voxel volume of this cell below a water surface at `level`.
    pub fn capacity(&self, level: f64) -> f64 {
        let mut v = 0.0;
        for (i, &c) in self.counts.iter().enumerate() {
            let f = f64::from(self.h0 + i as i32);
            if level > f {
                v += f64::from(c) * (level - f);
            }
        }
        for (&y, &d) in &self.edits {
            let t = (level - f64::from(y)).clamp(0.0, 1.0);
            if t > 0.0 {
                v += d as f64 * t;
            }
        }
        v
    }

    /// Wetted plan area (columns) of this cell at `level` — the denominator of
    /// the predicted `ΔV / area` level-error law.
    pub fn area(&self, level: f64) -> f64 {
        let mut a = 0.0;
        for (i, &c) in self.counts.iter().enumerate() {
            if level > f64::from(self.h0 + i as i32) {
                a += f64::from(c);
            }
        }
        a
    }

    /// Lowest floor plane the summary knows about, edits included.
    pub fn floor(&self) -> i32 {
        let dug = self
            .edits
            .iter()
            .filter(|(_, d)| **d > 0)
            .map(|(y, _)| *y)
            .min();
        match dug {
            Some(y) => y.min(self.h0),
            None => self.h0,
        }
    }
}

/// A sparse field of [`CapCell`]s over the world, built lazily from a surface
/// oracle and maintained by edit deltas.
#[derive(Clone, Debug)]
pub struct CapSummary {
    cell_edge: i64,
    sub: i64,
    cells: BTreeMap<(i64, i64), CapCell>,
    /// Instrumentation: surface samples taken since construction.
    pub stat_samples: u64,
    /// Instrumentation: edit deltas absorbed since construction.
    pub stat_edits: u64,
}

impl CapSummary {
    pub fn new(cell_edge: i64, sub: i64) -> Self {
        assert!(cell_edge > 0 && sub > 0 && cell_edge % sub == 0);
        Self {
            cell_edge,
            sub,
            cells: BTreeMap::new(),
            stat_samples: 0,
            stat_edits: 0,
        }
    }

    /// The production shape: 32-column cells sub-sampled 4×4.
    pub fn production() -> Self {
        Self::new(CAP_CELL, CAP_SUBSAMPLE)
    }

    pub fn cell_edge(&self) -> i64 {
        self.cell_edge
    }

    pub fn sub(&self) -> i64 {
        self.sub
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// The cell containing a world voxel column.
    pub fn cell_of(&self, x: i64, z: i64) -> (i64, i64) {
        (x.div_euclid(self.cell_edge), z.div_euclid(self.cell_edge))
    }

    pub fn get(&self, key: (i64, i64)) -> Option<&CapCell> {
        self.cells.get(&key)
    }

    /// Cells in key order — a stable traversal for hashing/persistence.
    pub fn iter(&self) -> impl Iterator<Item = (&(i64, i64), &CapCell)> {
        self.cells.iter()
    }

    /// Build the cell's summary if absent. `floor_at(x, z)` must return the
    /// column's floor plane (lowest air y); in production that is
    /// `coarse_surface(x, z).0 + 1`, which generates no chunks.
    pub fn ensure(&mut self, key: (i64, i64), floor_at: &mut impl FnMut(i64, i64) -> i32) {
        if self.cells.contains_key(&key) {
            return;
        }
        let step = self.cell_edge / self.sub;
        let per_sample = (step * step) as u32;
        let ox = key.0 * self.cell_edge;
        let oz = key.1 * self.cell_edge;
        let mut samples: Vec<i32> = Vec::with_capacity((self.sub * self.sub) as usize);
        for j in 0..self.sub {
            for i in 0..self.sub {
                // Sample the middle of the sub-tile it stands for.
                let x = ox + i * step + step / 2;
                let z = oz + j * step + step / 2;
                samples.push(floor_at(x, z));
            }
        }
        self.stat_samples += samples.len() as u64;
        let h0 = *samples.iter().min().expect("sub >= 1");
        let h1 = *samples.iter().max().expect("sub >= 1");
        let mut counts = vec![0u32; (h1 - h0 + 1) as usize];
        for s in samples {
            counts[(s - h0) as usize] += per_sample;
        }
        self.cells.insert(
            key,
            CapCell {
                h0,
                counts,
                edits: BTreeMap::new(),
            },
        );
    }

    /// Absorb one voxel's change from the audited write path. `delta` is `+1`
    /// when a solid became air and `-1` when air became solid. Ensures the
    /// cell first — an edit is where the player is, so its own cell's summary
    /// is 16 surface samples away and still costs no chunk.
    pub fn apply_open(
        &mut self,
        x: i64,
        y: i64,
        z: i64,
        delta: i64,
        floor_at: &mut impl FnMut(i64, i64) -> i32,
    ) {
        let key = self.cell_of(x, z);
        self.ensure(key, floor_at);
        let cell = self.cells.get_mut(&key).expect("just ensured");
        let e = cell.edits.entry(y as i32).or_insert(0);
        *e += delta;
        if *e == 0 {
            cell.edits.remove(&(y as i32));
        }
        self.stat_edits += 1;
    }

    /// Volume below `level` over a body's cells.
    pub fn capacity(&self, keys: &[(i64, i64)], level: f64) -> f64 {
        let mut v = 0.0;
        for k in keys {
            if let Some(c) = self.cells.get(k) {
                v += c.capacity(level);
            }
        }
        v
    }

    /// Wetted plan area (columns) below `level` over a body's cells.
    pub fn area(&self, keys: &[(i64, i64)], level: f64) -> f64 {
        let mut a = 0.0;
        for k in keys {
            if let Some(c) = self.cells.get(k) {
                a += c.area(level);
            }
        }
        a
    }

    /// Lowest floor plane over a body's cells.
    pub fn floor(&self, keys: &[(i64, i64)]) -> i32 {
        keys.iter()
            .filter_map(|k| self.cells.get(k))
            .map(CapCell::floor)
            .min()
            .unwrap_or(0)
    }

    /// Invert the capacity curve: the level holding `volume`. Bisection over
    /// `[floor, hi]`; 64 rounds, so the answer is exact to the bracket's
    /// float resolution and independent of iteration count in any reported
    /// digit. Deterministic — no ordering freedom anywhere in the sum.
    pub fn level_for(&self, keys: &[(i64, i64)], volume: f64, hi: f64) -> f64 {
        let mut lo = f64::from(self.floor(keys));
        let mut hi = hi;
        if self.capacity(keys, hi) < volume {
            return hi;
        }
        for _ in 0..64 {
            let mid = 0.5 * (lo + hi);
            if self.capacity(keys, mid) < volume {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

/// The **exact** capacity curve: per-y counts of open voxels inside a
/// footprint, obtained by actually walking the voxels. This is the thing the
/// coarse path exists to avoid; it is kept here so both curves are inverted by
/// identical arithmetic and the comparison reads a mechanism difference, not a
/// formula difference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactCurve {
    pub y0: i32,
    /// `open[i]` = open voxels at `y0 + i` inside the footprint.
    pub open: Vec<u64>,
}

impl ExactCurve {
    pub fn new(y0: i32, span: usize) -> Self {
        Self {
            y0,
            open: vec![0; span],
        }
    }

    pub fn add(&mut self, y: i32) {
        let i = y - self.y0;
        if i >= 0 && (i as usize) < self.open.len() {
            self.open[i as usize] += 1;
        }
    }

    pub fn capacity(&self, level: f64) -> f64 {
        let mut v = 0.0;
        for (i, &c) in self.open.iter().enumerate() {
            let y = f64::from(self.y0 + i as i32);
            let t = (level - y).clamp(0.0, 1.0);
            if t > 0.0 {
                v += c as f64 * t;
            }
        }
        v
    }

    pub fn area(&self, level: f64) -> f64 {
        let mut a = 0.0;
        for (i, &c) in self.open.iter().enumerate() {
            let y = f64::from(self.y0 + i as i32);
            if level > y && level <= y + 1.0 {
                a += c as f64;
            }
        }
        // Level above the top of the walked span: fall back to the widest row.
        if a == 0.0 {
            a = self.open.iter().copied().max().unwrap_or(0) as f64;
        }
        a
    }

    pub fn level_for(&self, volume: f64, hi: f64) -> f64 {
        let mut lo = f64::from(self.y0);
        let mut hi = hi;
        if self.capacity(hi) < volume {
            return hi;
        }
        for _ in 0..64 {
            let mid = 0.5 * (lo + hi);
            if self.capacity(mid) < volume {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

/// FNV-1a over a summary's cells, for the byte-identity proofs.
pub fn summary_hash(s: &CapSummary) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
    };
    for (k, c) in s.iter() {
        eat(&k.0.to_le_bytes());
        eat(&k.1.to_le_bytes());
        eat(&c.h0.to_le_bytes());
        for n in &c.counts {
            eat(&n.to_le_bytes());
        }
        for (y, d) in &c.edits {
            eat(&y.to_le_bytes());
            eat(&d.to_le_bytes());
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat floor at y=10 over one cell: capacity is exactly area × depth.
    #[test]
    fn flat_cell_capacity_is_area_times_depth() {
        let mut s = CapSummary::production();
        s.ensure((0, 0), &mut |_, _| 10);
        let cols = f64::from((CAP_CELL * CAP_CELL) as u32);
        assert_eq!(s.capacity(&[(0, 0)], 10.0), 0.0);
        assert_eq!(s.capacity(&[(0, 0)], 14.0), cols * 4.0);
        assert_eq!(s.area(&[(0, 0)], 14.0), cols);
        // And the inverse round-trips.
        let l = s.level_for(&[(0, 0)], cols * 4.0, 100.0);
        assert!((l - 14.0).abs() < 1e-9, "level {l}");
    }

    /// An edit is a delta, and the delta is exact — a dug voxel is worth
    /// exactly one voxel of capacity once the level clears it.
    #[test]
    fn an_edit_is_an_exact_integer_delta() {
        let mut s = CapSummary::production();
        let mut oracle = |_: i64, _: i64| 10;
        s.ensure((0, 0), &mut oracle);
        let before = s.capacity(&[(0, 0)], 12.0);
        s.apply_open(3, 9, 3, 1, &mut oracle);
        s.apply_open(3, 8, 3, 1, &mut oracle);
        assert_eq!(s.capacity(&[(0, 0)], 12.0), before + 2.0);
        // Below the cell floor only the edits count, and a half-covered voxel
        // counts half: y=8 is fully under 9.5, y=9 is half under.
        assert_eq!(s.capacity(&[(0, 0)], 9.5), 1.5);
        // Sealing it again removes the entry entirely: no residue.
        s.apply_open(3, 9, 3, -1, &mut oracle);
        s.apply_open(3, 8, 3, -1, &mut oracle);
        assert_eq!(s.capacity(&[(0, 0)], 12.0), before);
        assert!(s.get((0, 0)).expect("cell").edits.is_empty());
    }

    /// Edit order cannot matter: the deltas are integer counts in a BTreeMap.
    #[test]
    fn edit_order_is_irrelevant_to_the_summary() {
        let mut oracle = |x: i64, z: i64| 8 + ((x + z).rem_euclid(5) as i32);
        let mut edits: Vec<(i64, i64, i64)> = Vec::new();
        for i in 0..200i64 {
            edits.push((i % 32, 4 + (i % 7), (i * 13) % 32));
        }
        let mut a = CapSummary::production();
        for &(x, y, z) in &edits {
            a.apply_open(x, y, z, 1, &mut oracle);
        }
        let mut b = CapSummary::production();
        for &(x, y, z) in edits.iter().rev() {
            b.apply_open(x, y, z, 1, &mut oracle);
        }
        assert_eq!(summary_hash(&a), summary_hash(&b));
    }
}
