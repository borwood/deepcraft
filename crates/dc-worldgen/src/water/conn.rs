//! The derived connectivity index — voxel → air component, in two levels.
//!
//! Free water is **connectivity**, and connectivity is not local at any radius
//! (water.md). But it is not per-voxel either. This index is the two-level
//! answer the spike tests:
//!
//! 1. **Per-chunk local labelling** (32³, the project's chunk). Recomputed from
//!    geometry for a dirty chunk only — an edit costs one chunk relabel per
//!    touched chunk, never a world flood-fill.
//! 2. **A union-find over (chunk, local label) nodes**, joined across shared
//!    chunk faces. This is the *coarse* graph, and it is thousands of nodes for
//!    a heavily-dug world, not millions. A kilometre-long channel is ~40 nodes.
//!
//! A query is: chunk lookup → local label → `find` → component root. O(1) with
//! no search of any length, which is scenario 2's requirement.
//!
//! The index is **derived**, never persisted (water.md's persistence rule): it
//! is a pure function of the geometry, so unload/reload rebuilds it.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::vox::VoxWorld;

/// Chunk edge, in voxels. Matches the project's chunk.
pub const CH: usize = 32;
/// Local labels reserved per chunk in the global node numbering.
pub const LABEL_STRIDE: usize = 256;

const CH3: usize = CH * CH * CH;

/// A pair of local labels that touch across a shared chunk face.
type LabelPair = (u16, u16);
/// One cached chunk-face adjacency, flattened for replay.
type FaceEntry = (u32, u32, Vec<LabelPair>);

struct ChunkLabels {
    /// 0 = solid, 1..=count = local air component.
    label: Vec<u16>,
    count: u16,
}

/// Two-level connectivity over a [`VoxWorld`].
pub struct ConnIndex {
    cx: usize,
    cy: usize,
    cz: usize,
    chunks: HashMap<u32, ChunkLabels>,
    uf: Vec<u32>,
    /// Cached face adjacency: for each ordered adjacent chunk pair `(c, nc)`
    /// with `c < nc`, the distinct local-label pairs that touch across the
    /// shared face. Recomputed **only** for pairs involving a relabelled chunk.
    ///
    /// This is what keeps an edit cheap. Rescanning every chunk face on every
    /// edit is O(labelled chunks × 1024) — measured at 19.6 ms per one-voxel
    /// edit on a 512³-ish dug world, which would have been a ship-blocker.
    /// Replaying a cache of distinct label pairs is O(#adjacencies).
    faces: BTreeMap<(u32, u32), Vec<LabelPair>>,
    /// Instrumentation: voxels visited by chunk relabels since construction.
    pub stat_voxels_labelled: u64,
    /// Instrumentation: chunk relabels since construction.
    pub stat_chunk_relabels: u64,
    /// Instrumentation: union operations in the last global rebuild.
    pub stat_last_unions: u64,
    /// Instrumentation: coarse nodes live after the last global rebuild.
    pub stat_nodes: u64,
}

impl ConnIndex {
    pub fn build(w: &VoxWorld) -> Self {
        let cx = w.dx.div_ceil(CH);
        let cy = w.dy.div_ceil(CH);
        let cz = w.dz.div_ceil(CH);
        let mut idx = Self {
            cx,
            cy,
            cz,
            chunks: HashMap::new(),
            uf: vec![u32::MAX; cx * cy * cz * LABEL_STRIDE],
            faces: BTreeMap::new(),
            stat_voxels_labelled: 0,
            stat_chunk_relabels: 0,
            stat_last_unions: 0,
            stat_nodes: 0,
        };
        for c in 0..(cx * cy * cz) {
            idx.relabel_chunk(w, c as u32);
        }
        let all: Vec<u32> = idx.chunks.keys().copied().collect();
        idx.refresh_faces(&all);
        idx.rebuild_global();
        idx
    }

    /// Chunk indices adjacent to `c` along the three axes, both directions.
    fn neighbours(&self, c: u32) -> Vec<u32> {
        let ci = c as usize;
        let (cxi, czi, cyi) = (
            ci % self.cx,
            (ci / self.cx) % self.cz,
            ci / (self.cx * self.cz),
        );
        let mut out = Vec::with_capacity(6);
        let plane = (self.cx * self.cz) as u32;
        if cxi + 1 < self.cx {
            out.push(c + 1);
        }
        if cxi > 0 {
            out.push(c - 1);
        }
        if czi + 1 < self.cz {
            out.push(c + self.cx as u32);
        }
        if czi > 0 {
            out.push(c - self.cx as u32);
        }
        if cyi + 1 < self.cy {
            out.push(c + plane);
        }
        if cyi > 0 {
            out.push(c - plane);
        }
        out
    }

    /// Recompute the cached face adjacency for every pair touching one of
    /// `dirty`. Everything else in the cache stays valid, because a chunk's
    /// face labels only change when that chunk is relabelled.
    fn refresh_faces(&mut self, dirty: &[u32]) {
        let mut pairs: BTreeSet<(u32, u32)> = BTreeSet::new();
        for &c in dirty {
            for n in self.neighbours(c) {
                pairs.insert(if c < n { (c, n) } else { (n, c) });
            }
        }
        let li = |x: usize, y: usize, z: usize| (y * CH + z) * CH + x;
        for (a, b) in pairs {
            self.faces.remove(&(a, b));
            if !self.chunks.contains_key(&a) || !self.chunks.contains_key(&b) {
                continue;
            }
            // Which axis separates them? `b > a` always, so the offset tells us.
            let d = b - a;
            let axis = if d == 1 {
                0
            } else if d == self.cx as u32 {
                2
            } else {
                1
            };
            let mut seen: BTreeSet<LabelPair> = BTreeSet::new();
            for p in 0..CH {
                for q in 0..CH {
                    let (ia, ib) = match axis {
                        0 => (li(CH - 1, p, q), li(0, p, q)),
                        1 => (li(p, CH - 1, q), li(p, 0, q)),
                        _ => (li(p, q, CH - 1), li(p, q, 0)),
                    };
                    let la = self.chunks[&a].label[ia];
                    let lb = self.chunks[&b].label[ib];
                    if la != 0 && lb != 0 {
                        seen.insert((la, lb));
                    }
                }
            }
            if !seen.is_empty() {
                self.faces.insert((a, b), seen.into_iter().collect());
            }
        }
    }

    #[inline]
    fn chunk_of(&self, x: usize, y: usize, z: usize) -> u32 {
        (((y / CH) * self.cz + (z / CH)) * self.cx + (x / CH)) as u32
    }

    #[inline]
    fn chunk_origin(&self, c: u32) -> (usize, usize, usize) {
        let c = c as usize;
        let x = c % self.cx;
        let z = (c / self.cx) % self.cz;
        let y = c / (self.cx * self.cz);
        (x * CH, y * CH, z * CH)
    }

    /// Flood-label one chunk's air components from geometry.
    fn relabel_chunk(&mut self, w: &VoxWorld, c: u32) {
        let (ox, oy, oz) = self.chunk_origin(c);
        let mut label = vec![0u16; CH3];
        let mut count: u16 = 0;
        let mut stack: Vec<u32> = Vec::new();
        let li = |x: usize, y: usize, z: usize| (y * CH + z) * CH + x;
        let mut any = false;
        for ly in 0..CH {
            for lz in 0..CH {
                for lx in 0..CH {
                    let (gx, gy, gz) = (ox + lx, oy + ly, oz + lz);
                    if gx >= w.dx || gy >= w.dy || gz >= w.dz {
                        continue;
                    }
                    if w.is_solid(gx, gy, gz) || label[li(lx, ly, lz)] != 0 {
                        continue;
                    }
                    any = true;
                    count += 1;
                    assert!(
                        (count as usize) < LABEL_STRIDE,
                        "chunk {c} exceeded LABEL_STRIDE local components"
                    );
                    label[li(lx, ly, lz)] = count;
                    stack.push(li(lx, ly, lz) as u32);
                    while let Some(p) = stack.pop() {
                        let p = p as usize;
                        let px = p % CH;
                        let pz = (p / CH) % CH;
                        let py = p / (CH * CH);
                        for (dx, dy, dz) in [
                            (1i64, 0i64, 0i64),
                            (-1, 0, 0),
                            (0, 1, 0),
                            (0, -1, 0),
                            (0, 0, 1),
                            (0, 0, -1),
                        ] {
                            let nx = px as i64 + dx;
                            let ny = py as i64 + dy;
                            let nz = pz as i64 + dz;
                            if nx < 0
                                || ny < 0
                                || nz < 0
                                || nx >= CH as i64
                                || ny >= CH as i64
                                || nz >= CH as i64
                            {
                                continue;
                            }
                            let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                            let (gx, gy, gz) = (ox + nx, oy + ny, oz + nz);
                            if gx >= w.dx || gy >= w.dy || gz >= w.dz {
                                continue;
                            }
                            if w.is_solid(gx, gy, gz) || label[li(nx, ny, nz)] != 0 {
                                continue;
                            }
                            label[li(nx, ny, nz)] = count;
                            stack.push(li(nx, ny, nz) as u32);
                        }
                    }
                }
            }
        }
        self.stat_chunk_relabels += 1;
        self.stat_voxels_labelled += CH3 as u64;
        if any {
            self.chunks.insert(c, ChunkLabels { label, count });
        } else {
            self.chunks.remove(&c);
        }
    }

    #[inline]
    fn node(&self, c: u32, l: u16) -> u32 {
        (c as usize * LABEL_STRIDE + (l as usize - 1)) as u32
    }

    fn find(&mut self, mut a: u32) -> u32 {
        while self.uf[a as usize] != a {
            let p = self.uf[a as usize];
            self.uf[a as usize] = self.uf[p as usize];
            a = self.uf[a as usize];
        }
        a
    }

    fn union(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        // Lowest node id wins — deterministic, independent of union order.
        let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
        self.uf[hi as usize] = lo;
    }

    /// Rebuild the coarse union-find from the (already correct) per-chunk
    /// labels. Cost is O(#coarse nodes + #chunk faces), never O(#voxels of the
    /// component) — this is what makes a split as cheap as a merge.
    /// Replay the cached face adjacency into a fresh union-find.
    ///
    /// A **split** (an edit that disconnects a component) is exactly as cheap
    /// as a merge here, because nothing incremental is being patched: the
    /// coarse graph is rebuilt from scratch every time, and "from scratch" is
    /// thousands of nodes, not millions of voxels. Union-find famously cannot
    /// un-union; this sidesteps that by never needing to.
    pub fn rebuild_global(&mut self) {
        let live: Vec<(u32, u16)> = self.labelled_chunks();
        let mut nodes = 0u64;
        for &(c, count) in &live {
            for l in 1..=count {
                let n = self.node(c, l);
                self.uf[n as usize] = n;
                nodes += 1;
            }
        }
        self.stat_nodes = nodes;
        let mut unions = 0u64;
        let entries: Vec<FaceEntry> = self
            .faces
            .iter()
            .map(|(&(a, b), v)| (a, b, v.clone()))
            .collect();
        for (a, b, v) in entries {
            for (la, lb) in v {
                let (na, nb) = (self.node(a, la), self.node(b, lb));
                self.union(na, nb);
                unions += 1;
            }
        }
        self.stat_last_unions = unions;
    }

    /// Apply an edit: relabel every touched chunk, then rebuild the coarse
    /// graph. Returns (chunks relabelled, coarse unions performed).
    pub fn apply_edit(&mut self, w: &VoxWorld, lo: [i64; 3], hi: [i64; 3]) -> (u64, u64) {
        let before = self.stat_chunk_relabels;
        let mut touched: Vec<u32> = Vec::new();
        let clamp = |v: i64, n: usize| v.clamp(0, n as i64 - 1) as usize;
        let (x0, x1) = (clamp(lo[0], w.dx) / CH, clamp(hi[0], w.dx) / CH);
        let (y0, y1) = (clamp(lo[1], w.dy) / CH, clamp(hi[1], w.dy) / CH);
        let (z0, z1) = (clamp(lo[2], w.dz) / CH, clamp(hi[2], w.dz) / CH);
        for cy in y0..=y1 {
            for cz in z0..=z1 {
                for cxi in x0..=x1 {
                    touched.push(((cy * self.cz + cz) * self.cx + cxi) as u32);
                }
            }
        }
        for &c in &touched {
            self.relabel_chunk(w, c);
        }
        self.refresh_faces(&touched);
        self.rebuild_global();
        (self.stat_chunk_relabels - before, self.stat_last_unions)
    }

    /// Component root of a voxel, or `None` if it is solid.
    pub fn component(&mut self, x: usize, y: usize, z: usize) -> Option<u32> {
        let c = self.chunk_of(x, y, z);
        let cl = self.chunks.get(&c)?;
        let li = ((y % CH) * CH + (z % CH)) * CH + (x % CH);
        let l = cl.label[li];
        if l == 0 {
            return None;
        }
        let n = self.node(c, l);
        Some(self.find(n))
    }

    /// Number of live coarse nodes (the persisted graph's derived twin).
    pub fn coarse_nodes(&self) -> u64 {
        self.stat_nodes
    }

    /// Sorted (chunk, local-label count) pairs — a stable traversal order.
    fn labelled_chunks(&self) -> Vec<(u32, u16)> {
        let mut k: Vec<(u32, u16)> = self.chunks.iter().map(|(c, cl)| (*c, cl.count)).collect();
        k.sort_unstable();
        k
    }

    /// Number of distinct components.
    pub fn component_count(&mut self) -> usize {
        let mut roots = std::collections::BTreeSet::new();
        for (c, count) in self.labelled_chunks() {
            for l in 1..=count {
                let n = self.node(c, l);
                roots.insert(self.find(n));
            }
        }
        roots.len()
    }

    /// Per-component histogram of air voxels by y — the container's hypsometry.
    /// This is the derived, *unbounded* cost that motivates level-pinned
    /// reservoirs: a sea's histogram is the whole sea.
    pub fn hypsometry(&mut self, w: &VoxWorld) -> HashMap<u32, Vec<u32>> {
        self.hypsometry_skipping(w, &std::collections::BTreeSet::new())
    }

    /// As [`Self::hypsometry`], but skipping the named components entirely.
    ///
    /// This is what a **level-pinned reservoir** buys: a sea's level comes from
    /// outside, so nothing ever needs its capacity curve, and the scan can
    /// refuse to walk it. Since an ocean is most of the open volume in the
    /// world, that refusal is most of the cost.
    pub fn hypsometry_skipping(
        &mut self,
        w: &VoxWorld,
        skip: &std::collections::BTreeSet<u32>,
    ) -> HashMap<u32, Vec<u32>> {
        let chunks = self.labelled_chunks();
        // Resolve every (chunk, label) → root once, so the dense voxel scan
        // below needs no mutable borrow.
        let mut root_of: HashMap<u32, u32> = HashMap::new();
        for &(c, count) in &chunks {
            for l in 1..=count {
                let n = self.node(c, l);
                let r = self.find(n);
                root_of.insert(n, r);
            }
        }
        let mut out: HashMap<u32, Vec<u32>> = HashMap::new();
        for &(c, count) in &chunks {
            // If every component in this chunk is skipped, skip the whole
            // chunk — never touch its voxels. An ocean is most of the open
            // volume in a world, so this is where the pinned form's saving is.
            let all_skipped = (1..=count).all(|l| {
                let n = (c as usize * LABEL_STRIDE + (l as usize - 1)) as u32;
                skip.contains(&root_of[&n])
            });
            if all_skipped {
                continue;
            }
            let (ox, oy, oz) = self.chunk_origin(c);
            let cl = &self.chunks[&c];
            for ly in 0..CH {
                for lz in 0..CH {
                    for lx in 0..CH {
                        let (gx, gy, gz) = (ox + lx, oy + ly, oz + lz);
                        if gx >= w.dx || gy >= w.dy || gz >= w.dz {
                            continue;
                        }
                        let l = cl.label[(ly * CH + lz) * CH + lx];
                        if l == 0 {
                            continue;
                        }
                        let n = (c as usize * LABEL_STRIDE + (l as usize - 1)) as u32;
                        let r = root_of[&n];
                        if skip.contains(&r) {
                            continue;
                        }
                        out.entry(r).or_insert_with(|| vec![0u32; w.dy])[gy] += 1;
                    }
                }
            }
        }
        out
    }
}
