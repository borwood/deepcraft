//! The thin deep-time history pass: peoples seeded at habitable sites, a
//! coarse settlement / expansion / conflict sim over [`NUM_EPOCHS`] epochs,
//! its outcomes written as **committed facts** into the S2 constraint ledger.
//!
//! This is the year-zero handoff made literal: the pass builds a
//! statistical-tier *overlay world* (`ToyWorld::with_graph`) whose regions are
//! settlement sites, and collapses hostile-pressure questions through
//! `engine::observe` — the exact machinery the live sim uses after year zero.
//! Whether a site is sacked in epoch 7 is decided by an S2 collapse; the
//! resulting pressure fact, and the site/polity facts recorded around it, sit
//! in the same append-only ledger the live sim conditions on forever. One
//! system, no seam.
//!
//! Fact kinds kept deliberately thin (see `dc_sim::statistical::ledger`):
//! `SiteExists` (founded / abandoned, causally ordered), `SitePolity`,
//! `SiteEvent` (Founded / Sacked), `PolityExtent` per epoch, plus the
//! `RegionPressure` facts the observes commit.

use std::collections::BTreeSet;

use dc_sim::statistical::rng::{draw_f64, mix};
use dc_sim::statistical::world::RegionId;
use dc_sim::statistical::{
    Aspect, Ledger, Params, SiteEventKind, Subject, ToyWorld, Value, observe,
};

use super::{CellGrid, SALT_EXPAND, SALT_OVERLAY, SALT_SACK, SALT_SITE_POS};

/// Historical epochs simulated before year zero.
pub const NUM_EPOCHS: u32 = 12;
/// The sim tick where live play begins; pregen facts live at ticks `0..=12`.
pub const YEAR_ZERO_TICK: u32 = NUM_EPOCHS;
/// Site-slot cap — keeps site ids inside `RegionId` (u8) for the overlay.
pub const MAX_SITES: usize = 240;
/// Sites founded per polity per epoch at most; expansion probability below.
const EXPAND_PROB: f64 = 0.7;

/// A potential settlement location, fixed at pregen time.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteSlot {
    pub cell: (i32, i32),
    /// World-voxel position (x, z) of the site centre.
    pub pos: (i64, i64),
    pub score: f64,
}

/// A site's final pregen record (derived from the same state the ledger
/// facts were committed from; the ledger remains the source of truth).
#[derive(Debug, Clone, PartialEq)]
pub struct SiteSummary {
    pub slot: u32,
    pub cell: (i32, i32),
    pub pos: (i64, i64),
    pub polity: u32,
    pub founded: u32,
    pub abandoned: Option<u32>,
}

pub struct History {
    pub ledger: Ledger,
    pub overlay: ToyWorld,
    pub slots: Vec<SiteSlot>,
    pub sites: Vec<SiteSummary>,
    pub n_polities: u32,
    pub observe_count: u32,
}

/// Collapse parameters for pregen observes: shallow and cheap — pregen makes
/// many small collapses, and S2 showed depth-1 balls are exact enough for
/// one-hop-coupled pressure fields.
const OBSERVE_PARAMS: Params = Params {
    k_samples: 48,
    depth: Some(1),
};

pub fn run(seed: u64, grid: &CellGrid) -> History {
    let slots = plan_slots(seed, grid);
    let adjacency = slot_adjacency(&slots);
    // The overlay world needs >= 1 region; a dead world gets a placeholder.
    let overlay = if slots.is_empty() {
        ToyWorld::with_graph(mix(&[seed, SALT_OVERLAY]), vec![vec![]], vec![])
    } else {
        ToyWorld::with_graph(mix(&[seed, SALT_OVERLAY]), adjacency.clone(), vec![])
    };
    let mut ledger = Ledger::new();
    let mut observe_count = 0u32;

    #[derive(Clone)]
    struct SiteState {
        polity: u32,
        founded: u32,
        abandoned: Option<u32>,
    }
    let mut site_state: Vec<Option<SiteState>> = vec![None; slots.len()];
    let mut polity_sites: Vec<BTreeSet<usize>> = Vec::new();

    let found = |s: usize,
                 p: u32,
                 epoch: u32,
                 ledger: &mut Ledger,
                 site_state: &mut Vec<Option<SiteState>>,
                 polity_sites: &mut Vec<BTreeSet<usize>>| {
        site_state[s] = Some(SiteState {
            polity: p,
            founded: epoch,
            abandoned: None,
        });
        polity_sites[p as usize].insert(s);
        let subj = Subject::Site(s as u32);
        ledger
            .append(epoch, subj, Aspect::SiteExists, Value::Exists(true))
            .expect("fresh site fact");
        ledger
            .append(
                epoch,
                subj,
                Aspect::SiteEvent,
                Value::Event(SiteEventKind::Founded),
            )
            .expect("fresh site fact");
        ledger
            .append(epoch, subj, Aspect::SitePolity, Value::PolityRef(p))
            .expect("fresh site fact");
    };

    // Epoch 0: the founding wave — one polity per founding site, at the
    // best-scoring well-separated slots.
    if !slots.is_empty() {
        let n0 = (slots.len() / 8).clamp(2, 20).min(slots.len());
        for p in 0..n0 {
            polity_sites.push(BTreeSet::new());
            // Slots are score-ordered and spaced by construction; polity p
            // starts at slot p.
            found(
                p,
                p as u32,
                0,
                &mut ledger,
                &mut site_state,
                &mut polity_sites,
            );
        }
    }
    let n_polities = polity_sites.len() as u32;

    for epoch in 1..=NUM_EPOCHS {
        // Expansion: each polity may settle the best unclaimed slot adjacent
        // (on the overlay graph) to its territory.
        for p in 0..polity_sites.len() {
            if polity_sites[p].is_empty() {
                continue;
            }
            let mut frontier: BTreeSet<usize> = BTreeSet::new();
            for &s in &polity_sites[p] {
                for &nb in overlay.neighbors(s as RegionId) {
                    if site_state[nb as usize].is_none() {
                        frontier.insert(nb as usize);
                    }
                }
            }
            let target = frontier.iter().copied().max_by(|&a, &b| {
                slots[a].score.total_cmp(&slots[b].score).then(b.cmp(&a)) // deterministic tie-break: lower index
            });
            if let Some(t) = target
                && draw_f64(&[seed, SALT_EXPAND, p as u64, u64::from(epoch)]) < EXPAND_PROB
            {
                found(
                    t,
                    p as u32,
                    epoch,
                    &mut ledger,
                    &mut site_state,
                    &mut polity_sites,
                );
            }
        }

        // Pressure & conflict: contested or dangerous sites get their hostile
        // pressure *collapsed through the S2 engine*, committing a
        // RegionPressure fact; a raided pressure can doom the site.
        for s in 0..slots.len() {
            let Some(state) = site_state[s].clone() else {
                continue;
            };
            if state.abandoned.is_some() {
                continue;
            }
            let contested = overlay.neighbors(s as RegionId).iter().any(|&nb| {
                site_state[nb as usize]
                    .as_ref()
                    .is_some_and(|st| st.abandoned.is_none() && st.polity != state.polity)
            });
            let threat = overlay.base_danger(s as RegionId) > 0.30;
            if !(contested || threat) {
                continue;
            }
            let (value, _) = observe(
                &overlay,
                &mut ledger,
                Subject::Region(s as RegionId),
                epoch,
                Aspect::RegionPressure,
                OBSERVE_PARAMS,
            )
            .expect("pregen pressure collapse must be consistent");
            observe_count += 1;
            if value == Value::Pressure(2) && state.founded < epoch {
                let doom = draw_f64(&[seed, SALT_SACK, s as u64, u64::from(epoch)]);
                let p_sack = if contested { 0.55 } else { 0.30 };
                if doom < p_sack {
                    let subj = Subject::Site(s as u32);
                    ledger
                        .append(
                            epoch,
                            subj,
                            Aspect::SiteEvent,
                            Value::Event(SiteEventKind::Sacked),
                        )
                        .expect("fresh sack fact");
                    ledger
                        .append(epoch, subj, Aspect::SiteExists, Value::Exists(false))
                        .expect("fresh sack fact");
                    site_state[s].as_mut().expect("settled").abandoned = Some(epoch);
                    polity_sites[state.polity as usize].remove(&s);
                }
            }
        }

        // Chronicle: each living polity's extent this epoch.
        for (p, sites) in polity_sites.iter().enumerate() {
            if !sites.is_empty() {
                ledger
                    .append(
                        epoch,
                        Subject::Polity(p as u32),
                        Aspect::PolityExtent,
                        Value::Extent(sites.len() as u32),
                    )
                    .expect("fresh extent fact");
            }
        }
    }

    // The year-zero census: chroniclers record the state of every living
    // settlement at the moment live play begins (idempotent where conflict
    // already observed this epoch).
    for (s, st) in site_state.iter().enumerate() {
        if st.as_ref().is_some_and(|st| st.abandoned.is_none()) {
            observe(
                &overlay,
                &mut ledger,
                Subject::Region(s as RegionId),
                YEAR_ZERO_TICK,
                Aspect::RegionPressure,
                OBSERVE_PARAMS,
            )
            .expect("year-zero census must be consistent");
            observe_count += 1;
        }
    }

    let sites = site_state
        .iter()
        .enumerate()
        .filter_map(|(s, st)| {
            st.as_ref().map(|st| SiteSummary {
                slot: s as u32,
                cell: slots[s].cell,
                pos: slots[s].pos,
                polity: st.polity,
                founded: st.founded,
                abandoned: st.abandoned,
            })
        })
        .collect();

    History {
        ledger,
        overlay,
        slots,
        sites,
        n_polities,
        observe_count,
    }
}

/// Score habitable land cells and pick up to [`MAX_SITES`] well-separated
/// slots, best-first. Deterministic: sort keys are total orders.
fn plan_slots(seed: u64, grid: &CellGrid) -> Vec<SiteSlot> {
    let mut cands: Vec<(f64, i32, i32)> = Vec::new();
    for gy in 0..grid.w {
        for gx in 0..grid.w {
            let c = grid.get(gx, gy).expect("in grid");
            if !c.is_land() || c.temp_c < -3.0 || c.temp_c > 32.0 || c.precip < 0.10 {
                continue;
            }
            let coast = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dy)| grid.get(gx + dx, gy + dy).is_some_and(|n| !n.is_land()));
            let comfort = 1.0 - (c.temp_c - 14.0).abs() / 30.0;
            let score = 2.2 * c.precip
                + comfort
                + if c.river { 0.8 } else { 0.0 }
                + if coast { 0.5 } else { 0.0 }
                - c.elev_m / 3000.0;
            cands.push((score, gx, gy));
        }
    }
    cands.sort_by(|a, b| b.0.total_cmp(&a.0).then((a.1, a.2).cmp(&(b.1, b.2))));
    let mut slots: Vec<SiteSlot> = Vec::new();
    for (score, gx, gy) in cands {
        if slots.len() >= MAX_SITES {
            break;
        }
        let spaced = slots
            .iter()
            .all(|s| (s.cell.0 - gx).abs().max((s.cell.1 - gy).abs()) >= 2);
        if !spaced {
            continue;
        }
        let (cx, cz) = grid.cell_center_voxel(gx, gy);
        let jx = (draw_f64(&[seed, SALT_SITE_POS, gx as u64, gy as u64, 0]) - 0.5)
            * super::CELL_VOXELS as f64
            / 4.0;
        let jz = (draw_f64(&[seed, SALT_SITE_POS, gx as u64, gy as u64, 1]) - 0.5)
            * super::CELL_VOXELS as f64
            / 4.0;
        slots.push(SiteSlot {
            cell: (gx, gy),
            pos: (cx + jx as i64, cz + jz as i64),
            score,
        });
    }
    slots
}

/// Sites within Chebyshev distance 3 (cells) are graph neighbours.
fn slot_adjacency(slots: &[SiteSlot]) -> Vec<Vec<RegionId>> {
    let mut adj: Vec<Vec<RegionId>> = vec![Vec::new(); slots.len()];
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            let d = (slots[i].cell.0 - slots[j].cell.0)
                .abs()
                .max((slots[i].cell.1 - slots[j].cell.1).abs());
            if d <= 3 {
                adj[i].push(j as RegionId);
                adj[j].push(i as RegionId);
            }
        }
    }
    adj
}
