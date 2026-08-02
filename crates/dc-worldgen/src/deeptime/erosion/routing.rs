//! **The routing phase** — the receiver and out-weight planes, the drainage-area
//! accumulation and the downstream processing order the transport chain walks.
//!
//! The law it applies lives in `super::mfd` / `super::mfd_law`; this file is the
//! pass that runs it over the grid and owns the planes it writes.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use rayon::prelude::*;

use super::mfd::{MFD_DIRS, partition_cell, route_cell};
use super::mfd_law::MfdParams;
use super::{Erosion, NEIGH8, coords_of, in_grid};

impl Erosion {
    /// Turn **multiple-flow-direction routing** on under the hybrid-`p` law
    /// ([`MfdParams`], flow.md § 2.6/§ 2.6.2). `None` is the single-receiver D8
    /// path — the pre-MFD solve, byte for byte, because nothing else in the class
    /// reads `mfd_w`. [`MfdParams::uniform`] is journal/0109's one-exponent solve.
    pub fn set_mfd(&mut self, p: Option<MfdParams>) {
        self.mfd = p;
        if p.is_some() {
            if self.mfd_w.len() != self.n * MFD_DIRS {
                self.mfd_w = vec![0.0; self.n * MFD_DIRS];
            }
        } else {
            self.mfd_w = Vec::new();
        }
    }

    /// Whether MFD routing is on.
    #[inline]
    pub fn is_mfd(&self) -> bool {
        self.mfd.is_some()
    }

    /// **The solve's own processing order** — cells by ascending filled surface,
    /// the array [`Self::transport`] walks in reverse.
    ///
    /// Public so an instrument can traverse the transport graph **without
    /// re-deriving it** (S-3: the summary derived from the authority, never beside
    /// it). A probe that computed its own D8 receivers would be a second routing
    /// rule that can silently disagree with the one the world was built by.
    pub fn processing_order(&self) -> &[u32] {
        &self.order
    }

    /// **Every cell this cell hands load to, this epoch** — the single D8 receiver
    /// off the MFD path, or every MFD direction carrying positive weight. Yields
    /// nothing for a sink.
    ///
    /// The companion to [`Self::processing_order`]; together they are the transport
    /// graph exactly as [`Self::transport`] walks it.
    pub fn out_edges(&self, c: usize, f: &mut dyn FnMut(usize)) {
        if self.mfd.is_none() {
            let rc = self.recv[c];
            if rc >= 0 {
                f(rc as usize);
            }
            return;
        }
        let base = c * MFD_DIRS;
        for d in 0..MFD_DIRS {
            if self.mfd_w[base + d] > 0.0 {
                f(self.mfd_neighbour(c, d));
            }
        }
    }

    /// Receivers on the filled free-surface potential. Sea and border cells are
    /// sinks (`recv = -1`). Per-cell independent → byte-identical parallel.
    ///
    /// **Two routings, one phase.** With MFD off this is the historical D8
    /// steepest-descent rule and `recv` *is* the routing. With MFD on the phase
    /// computes the full eight-way partition into `mfd_w` — that is what the
    /// accumulation and transport chains then read — and `recv` becomes the
    /// **argmax share**: a projection of the partition, kept because the exported
    /// drainage network and the biotic layer still consume a single receiver. It is
    /// no longer the routing, and that is the interesting fact about it (see the
    /// [`Self::recv`] doc).
    ///
    /// ## The one-epoch lag on drainage area, and why it is not a cheat
    ///
    /// The hybrid-`p` law ([`MfdParams`]) needs the cell's drainage area to decide
    /// how channelised it is — and drainage area is computed by
    /// [`Self::accumulate_area`], which runs *after* this phase and *from* the
    /// weights this phase writes. The dependency is genuinely circular, and there
    /// is no fixed point to iterate to: a second accumulation pass would use an
    /// area that its own weights then invalidate.
    ///
    /// So the exponent reads `self.area` as this phase finds it — **the previous
    /// epoch's accumulation**. This is the same shape as the S10 biotic coupling
    /// (`grid.bio_weather` is a lagged plane for exactly this reason) and it costs
    /// nothing: no extra storage, no extra pass, and `accumulate_area` re-seeds
    /// `area` to `1.0` at its own start so nothing stale survives into the sums.
    ///
    /// At epoch 0 the plane is all zeros, so `χ = 0`, the exponent is `p_hill`
    /// everywhere and the first epoch is maximally dispersive. That is the honest
    /// initial condition rather than a guess: nothing is channelised until water
    /// has run once, which is also the physical statement.
    pub fn route(&mut self) {
        let (w, sea, cell_m) = (self.w, self.sea_level, self.cell_m);
        let parallel = self.par();
        let Some(mp) = self.mfd else {
            let surf = &self.surf;
            let filled = &self.filled;
            if parallel {
                self.recv
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, r)| *r = route_cell(i, w, surf, filled, sea));
            } else {
                for i in 0..self.n {
                    self.recv[i] = route_cell(i, w, surf, filled, sea);
                }
            }
            return;
        };
        // Disjoint field borrows: `area` is read-only here (so the parallel path
        // stays byte-identical), `recv`/`mfd_w` are written per cell.
        let Erosion {
            surf,
            filled,
            area,
            recv,
            mfd_w,
            ..
        } = self;
        let (surf, filled, area) = (&*surf, &*filled, &*area);
        let step = |i: usize, r: &mut i32, ws: &mut [f64]| {
            let best = partition_cell(i, w, surf, filled, sea, area[i], cell_m, &mp, ws);
            *r = if best < 0 {
                -1
            } else {
                let (dx, dy) = NEIGH8[best as usize];
                let (gx, gy) = coords_of(i, w);
                in_grid(gx + dx, gy + dy, w).expect("a weighted direction is in-grid") as i32
            };
        };
        if parallel {
            recv.par_iter_mut()
                .zip(mfd_w.par_chunks_mut(MFD_DIRS))
                .enumerate()
                .for_each(|(i, (r, ws))| step(i, r, ws));
        } else {
            for (i, (r, ws)) in recv.iter_mut().zip(mfd_w.chunks_mut(MFD_DIRS)).enumerate() {
                step(i, r, ws);
            }
        }
    }

    /// The neighbour cell index of direction `d` from cell `i`, valid only where
    /// `mfd_w[i * 8 + d] > 0` (a weighted direction is in-grid by construction).
    #[inline]
    pub(super) fn mfd_neighbour(&self, i: usize, d: usize) -> usize {
        let (dx, dy) = NEIGH8[d];
        (i as isize + dy as isize * self.w as isize + dx as isize) as usize
    }

    // ---- phase 5: drainage-area accumulation (SCALAR — flux chain) --------

    /// Drainage area (in cell units) accumulated downstream in descending
    /// filled order. A receiver must see all upstream contributions before it is
    /// routed on — a serial dependency chain; a level-parallel gather would
    /// reorder the sums and break byte-identity.
    ///
    /// ## Why the priority-flood order is still a topological order under MFD
    ///
    /// This was the part MFD was expected to break, and it does not. `self.order`
    /// is the priority-flood pop order, which is **strictly ascending in
    /// `(filled, index)`**: each cell is pushed exactly once, with its final
    /// `filled` value, and popped in heap order. Reversed, it is strictly
    /// *descending*. Every routed edge — single-receiver or MFD — goes to a
    /// neighbour with **strictly smaller `filled`** (`partition_cell` weights only
    /// `drop > 0`). So every out-edge points to a cell that comes strictly later in
    /// the reversed scan, and a cell is processed only after every contributor.
    ///
    /// The tree gave a *convenient* traversal; what actually licensed it was the
    /// potential ordering, and that licenses the **DAG** identically. MFD needs no
    /// new topological sort — it needs the observation that the old one was never
    /// about the tree.
    pub fn accumulate_area(&mut self) {
        self.area.iter_mut().for_each(|a| *a = 1.0);
        if self.mfd.is_none() {
            let record = !self.out_area.is_empty();
            if record {
                self.out_area.iter_mut().for_each(|v| *v = 0.0);
            }
            for k in (0..self.order.len()).rev() {
                let i = self.order[k] as usize;
                let rc = self.recv[i];
                if rc >= 0 {
                    self.area[rc as usize] += self.area[i];
                    if record {
                        let (gx, gy) = coords_of(i, self.w);
                        let (jx, jy) = coords_of(rc as usize, self.w);
                        let d = NEIGH8
                            .iter()
                            .position(|&(dx, dy)| (gx + dx, gy + dy) == (jx, jy))
                            .expect("the receiver is a D8 neighbour");
                        self.out_area[i * MFD_DIRS + d] = self.area[i] as f32;
                    }
                }
            }
            return;
        }
        let record = !self.out_area.is_empty();
        if record {
            self.out_area.iter_mut().for_each(|v| *v = 0.0);
        }
        for k in (0..self.order.len()).rev() {
            let i = self.order[k] as usize;
            let base = i * MFD_DIRS;
            let a = self.area[i];
            // The last weighted direction takes the **residual**, so the shares sum
            // to `a` exactly rather than to `a · Σw` with Σw off by an ulp. Mass
            // conservation down a DAG is a chain of these; a per-hop rounding error
            // would compound over a thousand hops and leak silently.
            let Some(last) = (0..MFD_DIRS).rev().find(|&d| self.mfd_w[base + d] > 0.0) else {
                continue;
            };
            let mut given = 0.0;
            for d in 0..MFD_DIRS {
                let wt = self.mfd_w[base + d];
                if wt <= 0.0 {
                    continue;
                }
                let share = if d == last { a - given } else { wt * a };
                given += share;
                let j = self.mfd_neighbour(i, d);
                self.area[j] += share;
                if record {
                    self.out_area[base + d] = share as f32;
                }
            }
        }
    }

    // ---- phase 6: stream-power transport (SCALAR — flux chain) ------------

    /// The drainage area (in cell units) accumulated this step — read by the S10
    /// biotic disturbance phase to find flood-prone valley cells, and exported as
    /// the final-chapter discharge field (§ 7.3, drainage export).
    pub fn area(&self) -> &[f64] {
        &self.area
    }

    /// The single receiver of each cell as of the last routing (`-1` = sink) — the
    /// exported final drainage network (§ 7.3). This is the last iteration's
    /// routing exactly (no recompute).
    ///
    /// **Under MFD this is no longer the routing** (flow.md § 2.6). With MFD off it
    /// is the D8 steepest-descent receiver and the solve genuinely sends the cell's
    /// whole discharge there. With MFD on the solve sends the discharge to *several*
    /// neighbours, and this plane holds the **argmax share** — a projection of the
    /// partition, retained only because two consumers still want one arrow per cell
    /// (the biotic layer's valley test and the `DeepField` drainage export).
    ///
    /// That is a change of *kind*, and it is worth stating for FLOW continuation
    /// (e), the retirement slice: before MFD, `recv` was an authority the record
    /// shadowed; after MFD it is a **summary of an authority** in the exact sense
    /// ARCHITECTURE.md warns about — it would not exist in this shape if those two
    /// consumers vanished. It is not *meaningless* (the argmax of a partition is a
    /// well-defined thing and still answers "which way does most of the water go"),
    /// but it can no longer be read as "where the water went", and every new
    /// consumer should read the flux record instead.
    pub fn recv(&self) -> &[i32] {
        &self.recv
    }

    /// The pre-erosion surface snapshot from the last routing.
    pub fn routed_surface(&self) -> &[f64] {
        &self.surf
    }

    // ---- phase 8: hillslope diffusion (PARALLEL — gather form) ------------
}
