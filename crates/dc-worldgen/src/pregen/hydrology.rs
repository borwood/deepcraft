//! Drainage planned on the region graph, guaranteed to reach the sea.
//!
//! Priority-flood depression filling (Barnes et al. 2014) from the ocean
//! inward: every land cell's *filled* elevation is strictly greater than that
//! of some neighbour on a path to the ocean, so steepest-descent flow routing
//! on filled elevations can never cycle and always terminates at an ocean
//! cell — rivers reach the sea **by construction**, which
//! `tests/s7_pregen.rs` proves over the whole map at every extent.
//!
//! Discharge is precipitation accumulated down the flow forest; edges above a
//! discharge threshold are rivers (realized as carved channels by the lazy
//! pyramid). Cells the filling raised are lakes.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::CellGrid;

/// Strictly-descending fill increment (metres).
const EPS: f64 = 0.01;
/// Discharge (accumulated precip units) above which an edge is a river.
pub const RIVER_MIN_DISCHARGE: f64 = 0.9;

/// Heap item ordered by filled elevation (min-heap via `Reverse`).
#[derive(PartialEq)]
struct Item {
    filled: f64,
    idx: usize,
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.filled
            .total_cmp(&other.filled)
            .then(self.idx.cmp(&other.idx))
    }
}

const NEIGH8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub fn apply(grid: &mut CellGrid) {
    let w = grid.w;
    let n = grid.cells.len();
    let mut filled = vec![f64::INFINITY; n];
    let mut heap: BinaryHeap<Reverse<Item>> = BinaryHeap::new();

    // Seeds: the ocean (and the grid border, which is ocean by construction —
    // the world-ocean ring guarantees it; asserted by the pipeline test).
    for (i, f) in filled.iter_mut().enumerate() {
        let (gx, gy) = grid.coords(i);
        let border = gx == 0 || gy == 0 || gx == w - 1 || gy == w - 1;
        if grid.cells[i].elev_m <= 0.0 || border {
            *f = grid.cells[i].elev_m;
            heap.push(Reverse(Item { filled: *f, idx: i }));
        }
    }
    let mut done = vec![false; n];
    while let Some(Reverse(item)) = heap.pop() {
        let i = item.idx;
        if done[i] {
            continue;
        }
        done[i] = true;
        let (gx, gy) = grid.coords(i);
        for (dx, dy) in NEIGH8 {
            let Some(j) = grid.idx(gx + dx, gy + dy) else {
                continue;
            };
            if done[j] || filled[j].is_finite() {
                continue;
            }
            filled[j] = grid.cells[j].elev_m.max(filled[i] + EPS);
            heap.push(Reverse(Item {
                filled: filled[j],
                idx: j,
            }));
        }
    }

    // Flow routing: steepest descent on filled elevations; land only.
    for i in 0..n {
        grid.cells[i].filled_m = filled[i];
        grid.cells[i].lake = grid.cells[i].is_land() && filled[i] > grid.cells[i].elev_m + 1.0;
        if !grid.cells[i].is_land() {
            grid.cells[i].flow_to = None;
            continue;
        }
        let (gx, gy) = grid.coords(i);
        let mut best: Option<(f64, usize)> = None;
        for (dx, dy) in NEIGH8 {
            let Some(j) = grid.idx(gx + dx, gy + dy) else {
                continue;
            };
            if filled[j] < filled[i] && best.is_none_or(|(bf, _)| filled[j] < bf) {
                best = Some((filled[j], j));
            }
        }
        grid.cells[i].flow_to = best.map(|(_, j)| j as u32);
    }

    // Discharge: process land cells in descending filled order.
    let mut order: Vec<usize> = (0..n).filter(|&i| grid.cells[i].is_land()).collect();
    order.sort_by(|&a, &b| filled[b].total_cmp(&filled[a]).then(a.cmp(&b)));
    for i in 0..n {
        grid.cells[i].discharge = if grid.cells[i].is_land() {
            grid.cells[i].precip
        } else {
            0.0
        };
    }
    for &i in &order {
        if let Some(j) = grid.cells[i].flow_to {
            let d = grid.cells[i].discharge;
            grid.cells[j as usize].discharge += d;
        }
    }
    for i in 0..n {
        grid.cells[i].river = grid.cells[i].is_land()
            && grid.cells[i].flow_to.is_some()
            && grid.cells[i].discharge >= RIVER_MIN_DISCHARGE;
    }
}
