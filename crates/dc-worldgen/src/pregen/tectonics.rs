//! Plate tectonics on the bounded disc: seeded Voronoi plates with velocity
//! vectors, boundary classification (convergent / divergent / transform), and
//! elevation with provenance (orogeny, arcs, rifts, trenches, ridges,
//! shelves).
//!
//! The disc topology means this is *not* a closed spherical surface; the
//! world-ocean ring acts as the closure instead — the outer ~10% of the disc
//! radius is forced oceanic, so every continent is finite and every margin is
//! eventually passive. The trade-offs are discussed in the S7 results doc.

use dc_sim::statistical::rng::draw_f64;

use super::{Cell, CellGrid, Provenance, SALT_CELL, SALT_PLATE};

struct Plate {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    continental: bool,
}

/// Fraction of the half-extent beyond which cells are forced oceanic
/// (the world-ocean ring that closes the disc).
const OCEAN_RING: f64 = 0.78;

pub fn build(seed: u64, w: i32) -> CellGrid {
    let wf = f64::from(w);
    let center = wf / 2.0;
    let ocean_r = center * OCEAN_RING;
    let n_plates = ((w * w) / 20).clamp(3, 24) as usize;

    let plates: Vec<Plate> = (0..n_plates)
        .map(|p| {
            let p64 = p as u64;
            let x = draw_f64(&[seed, SALT_PLATE, p64, 0]) * wf;
            let y = draw_f64(&[seed, SALT_PLATE, p64, 1]) * wf;
            let ang = draw_f64(&[seed, SALT_PLATE, p64, 2]) * std::f64::consts::TAU;
            let speed = 0.4 + 0.6 * draw_f64(&[seed, SALT_PLATE, p64, 3]);
            let r = ((x - center).powi(2) + (y - center).powi(2)).sqrt();
            let p_cont = if r < wf * 0.33 { 0.85 } else { 0.25 };
            let continental = draw_f64(&[seed, SALT_PLATE, p64, 4]) < p_cont;
            Plate {
                x,
                y,
                vx: ang.cos() * speed,
                vy: ang.sin() * speed,
                continental,
            }
        })
        .collect();

    let n = (w * w) as usize;
    let mut cells = Vec::with_capacity(n);
    for gy in 0..w {
        for gx in 0..w {
            let cx = f64::from(gx) + 0.5;
            let cy = f64::from(gy) + 0.5;
            // Nearest plate seed; ties resolve to the lower index via strict <.
            let mut plate = 0usize;
            let mut best = f64::INFINITY;
            for (i, p) in plates.iter().enumerate() {
                let d = (p.x - cx).powi(2) + (p.y - cy).powi(2);
                if d < best {
                    best = d;
                    plate = i;
                }
            }
            let disc_d = ((cx - center).powi(2) + (cy - center).powi(2)).sqrt();
            let noise = draw_f64(&[seed, SALT_CELL, gx as u64, gy as u64]).mul_add(2.0, -1.0);
            let continental = plates[plate].continental && disc_d < ocean_r;
            let (mut elev, mut provenance) = if continental {
                (260.0 + 140.0 * noise, Provenance::Craton)
            } else {
                (-150.0 - 50.0 * noise, Provenance::OceanFloor)
            };
            // World-ocean ring: deepen beyond the disc.
            if disc_d > ocean_r {
                elev -= 45.0 * (disc_d - ocean_r);
                elev = elev.min(-60.0);
                provenance = Provenance::OceanFloor;
            }
            cells.push(Cell {
                plate: plate as u16,
                elev_m: elev,
                provenance,
                lat_deg: 0.0,
                temp_c: 0.0,
                precip: 0.0,
                filled_m: 0.0,
                flow_to: None,
                discharge: 0.0,
                river: false,
                lake: false,
            });
        }
    }
    let mut grid = CellGrid { w, cells };
    apply_boundaries(&plates, &mut grid);
    // Re-force the world-ocean ring *after* boundary uplift: orogeny near the
    // rim must not breach the closure. This is what makes the border wilds
    // continue seamlessly (deepening ocean) beyond the grid in any direction.
    for gy in 0..w {
        for gx in 0..w {
            let cx = f64::from(gx) + 0.5;
            let cy = f64::from(gy) + 0.5;
            let disc_d = ((cx - center).powi(2) + (cy - center).powi(2)).sqrt();
            if disc_d >= ocean_r {
                let i = grid.idx(gx, gy).expect("in grid");
                let c = &mut grid.cells[i];
                let cap = -60.0 - 45.0 * (disc_d - ocean_r);
                if c.elev_m > cap {
                    c.elev_m = cap;
                    c.provenance = Provenance::OceanFloor;
                }
            }
        }
    }
    apply_shelves(&mut grid);
    grid
}

/// One boundary-driven elevation contribution at a cell.
struct Effect {
    cell: usize,
    delta_m: f64,
    provenance: Provenance,
}

fn apply_boundaries(plates: &[Plate], grid: &mut CellGrid) {
    let w = grid.w;
    let mut effects: Vec<Effect> = Vec::new();
    // Classify every plate-crossing 4-neighbour edge once (east and south).
    for gy in 0..w {
        for gx in 0..w {
            let a = grid.idx(gx, gy).expect("in grid");
            for (dx, dy) in [(1, 0), (0, 1)] {
                let Some(b) = grid.idx(gx + dx, gy + dy) else {
                    continue;
                };
                let (pa, pb) = (grid.cells[a].plate as usize, grid.cells[b].plate as usize);
                if pa == pb {
                    continue;
                }
                // Normal points from b toward a; convergence > 0 when the
                // plates close on each other.
                let (nx, ny) = (-f64::from(dx), -f64::from(dy));
                let conv =
                    (plates[pb].vx - plates[pa].vx) * nx + (plates[pb].vy - plates[pa].vy) * ny;
                let m = conv.abs().min(2.0);
                let (ca, cb) = (plates[pa].continental, plates[pb].continental);
                if conv > 0.25 {
                    match (ca, cb) {
                        (true, true) => {
                            for c in [a, b] {
                                effects.push(Effect {
                                    cell: c,
                                    delta_m: 1400.0 * m,
                                    provenance: Provenance::Orogeny,
                                });
                            }
                        }
                        (true, false) | (false, true) => {
                            let (cont, oce) = if ca { (a, b) } else { (b, a) };
                            effects.push(Effect {
                                cell: cont,
                                delta_m: 520.0 * m,
                                provenance: Provenance::Arc,
                            });
                            effects.push(Effect {
                                cell: oce,
                                delta_m: -320.0 * m,
                                provenance: Provenance::Trench,
                            });
                        }
                        (false, false) => {
                            // The lower-index plate overrides: island arc.
                            let (over, under) = if pa < pb { (a, b) } else { (b, a) };
                            effects.push(Effect {
                                cell: over,
                                delta_m: 260.0 * m,
                                provenance: Provenance::Arc,
                            });
                            effects.push(Effect {
                                cell: under,
                                delta_m: -220.0 * m,
                                provenance: Provenance::Trench,
                            });
                        }
                    }
                } else if conv < -0.25 {
                    for (c, cont) in [(a, ca), (b, cb)] {
                        if cont {
                            effects.push(Effect {
                                cell: c,
                                delta_m: -180.0 * m,
                                provenance: Provenance::Rift,
                            });
                        } else {
                            effects.push(Effect {
                                cell: c,
                                delta_m: 90.0 * m,
                                provenance: Provenance::Ridge,
                            });
                        }
                    }
                } else {
                    for c in [a, b] {
                        effects.push(Effect {
                            cell: c,
                            delta_m: 45.0 * m.max(0.3),
                            provenance: Provenance::Transform,
                        });
                    }
                }
            }
        }
    }
    // Spread each effect over rings 0..=2 with falloff; accumulate the sum and
    // keep the dominant contribution as the cell's provenance.
    const FALLOFF: [f64; 3] = [1.0, 0.5, 0.22];
    let n = grid.cells.len();
    let mut total = vec![0.0f64; n];
    let mut dominant: Vec<Option<(f64, Provenance)>> = vec![None; n];
    for e in &effects {
        let (egx, egy) = grid.coords(e.cell);
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let ring = dx.abs().max(dy.abs()) as usize;
                let Some(c) = grid.idx(egx + dx, egy + dy) else {
                    continue;
                };
                let d = e.delta_m * FALLOFF[ring];
                total[c] += d;
                let mag = d.abs();
                if dominant[c].is_none_or(|(m, _)| mag > m) {
                    dominant[c] = Some((mag, e.provenance));
                }
            }
        }
    }
    for (i, cell) in grid.cells.iter_mut().enumerate() {
        cell.elev_m += total[i];
        if let Some((mag, p)) = dominant[i]
            && mag > 60.0
        {
            cell.provenance = p;
        }
    }
}

fn apply_shelves(grid: &mut CellGrid) {
    let w = grid.w;
    let mut shelf = Vec::new();
    for gy in 0..w {
        for gx in 0..w {
            let i = grid.idx(gx, gy).expect("in grid");
            if grid.cells[i].elev_m > 0.0 {
                continue;
            }
            let near_land = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dy)| grid.get(gx + dx, gy + dy).is_some_and(|c| c.elev_m > 0.0));
            if near_land {
                shelf.push(i);
            }
        }
    }
    for i in shelf {
        let c = &mut grid.cells[i];
        c.elev_m = c.elev_m.max(-30.0);
        if matches!(
            c.provenance,
            Provenance::OceanFloor | Provenance::Ridge | Provenance::Craton
        ) {
            c.provenance = Provenance::Shelf;
        }
    }
}
