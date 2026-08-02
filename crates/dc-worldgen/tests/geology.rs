//! Geology backbone proofs (slice 5):
//!
//! - byte-identical regeneration with the geology passes on (blocks AND
//!   material sidecars AND the mixture table);
//! - registration-order independence: permuting class-member registration
//!   produces an identical world, byte for byte;
//! - abundance normalization at world level: adding a member diversifies its
//!   class's share, never inflates it (strata thickness identical, member
//!   identity redistributed);
//! - the placer follows the sorted gradient (shape, not magic numbers);
//! - strata records exist and respect province/climate driving.

use dc_core::materials::geology::{
    self, CLASS_ACCESSORY_MAFIC, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE,
    CLASS_IGNEOUS_INTRUSIVE, FormationWindow, GeoHabit, GeoMemberDef, GeologySet,
};
use dc_core::{Chunk, ChunkPos, MaterialId};
use dc_worldgen::geology::{
    StrataEvent, coarse_fraction, dithered_member, member_settle_threshold, ore_eighths,
};
use dc_worldgen::{Extent, Pregen, Provenance, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

fn medium() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    })
}

fn block_hash(chunk: &Chunk) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in chunk.blocks() {
        h ^= u64::from(b.ordinal());
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// Generate a deterministic sample of surface chunks (with materials) and
/// fingerprint everything: block bytes, material sidecar bytes, table bytes.
fn world_fingerprint(pregen: &Pregen, geology: GeologySet) -> (Vec<u64>, Vec<Vec<u8>>, Vec<u8>) {
    let mut g = WorldGenerator::with_geology(pregen, geology);
    let mut blocks = Vec::new();
    let mut materials = Vec::new();
    for k in 0..48i64 {
        let (cx, cz) = (k * 7 - 160, (k % 5) * 11 - 20);
        let cy = g.surface_chunk_y(cx, cz);
        let (chunk, mat) = g.generate_chunk_with_materials(ChunkPos::new(cx as i32, cy, cz as i32));
        blocks.push(block_hash(&chunk));
        materials.push(mat.encode());
    }
    (blocks, materials, g.mixture_table().encode())
}

/// Vanilla plus one extra member per clastic class, registered in the given
/// order — the multi-member set the permutation/abundance proofs need.
fn extended_set(member_order_reversed: bool) -> GeologySet {
    let mut members = geology::vanilla_members();
    members.push(GeoMemberDef {
        id: "zz:geo/siltstone".into(),
        class: CLASS_CLASTIC_FINE.into(),
        material: MaterialId::SILT,
        window: FormationWindow {
            temp_c: (-5.0, 35.0),
            precip: (0.15, 1.0),
            depth_m: (0.0, 80.0),
        },
        abundance: 1.0,
        habit: GeoHabit::Blanket,
        hardness: 0.3,
        erodibility: 0.75,
    });
    members.push(GeoMemberDef {
        id: "aa:geo/greywacke".into(),
        class: CLASS_CLASTIC_COARSE.into(),
        material: MaterialId::GRAVEL,
        window: FormationWindow {
            temp_c: (-10.0, 40.0),
            precip: (0.05, 1.0),
            depth_m: (0.0, 100.0),
        },
        abundance: 1.0,
        habit: GeoHabit::Blanket,
        hardness: 0.55,
        erodibility: 0.45,
    });
    if member_order_reversed {
        members.reverse();
    }
    let mut b = GeologySet::builder();
    for class in geology::v1_classes() {
        b.declare_class(class).unwrap();
    }
    for m in members {
        b.add_member(m).unwrap();
    }
    b.build()
}

#[test]
fn geology_world_regenerates_byte_identically() {
    let p1 = medium();
    let (b1, m1, t1) = world_fingerprint(&p1, geology::vanilla());
    // Fresh pregen + generator from the same seed: identical everything.
    let p2 = medium();
    let (b2, m2, t2) = world_fingerprint(&p2, geology::vanilla());
    assert_eq!(b1, b2, "block bytes must regenerate identically");
    assert_eq!(m1, m2, "material sidecar bytes must regenerate identically");
    assert_eq!(t1, t2, "mixture table bytes must regenerate identically");
    // A different seed produces a different world.
    let p3 = Pregen::run(WorldParams {
        seed: SEED + 1,
        extent: Extent::Medium,
    });
    let (b3, _, _) = world_fingerprint(&p3, geology::vanilla());
    assert_ne!(b1, b3, "different seeds must differ");
}

#[test]
fn class_registration_order_cannot_change_world_bytes() {
    let pregen = medium();
    let (b1, m1, t1) = world_fingerprint(&pregen, extended_set(false));
    let (b2, m2, t2) = world_fingerprint(&pregen, extended_set(true));
    assert_eq!(b1, b2, "registration order changed block bytes");
    assert_eq!(m1, m2, "registration order changed material bytes");
    assert_eq!(t1, t2, "registration order changed the mixture table");
}

/// Sample the strata of a grid of land columns around the world centre.
fn sample_columns(
    g: &mut WorldGenerator,
    n: i64,
) -> Vec<std::sync::Arc<dc_worldgen::collapse::ColumnRec>> {
    let mut out = Vec::new();
    for cz in -n..n {
        for cx in -n..n {
            out.push(g.column_record(cx * 3, cz * 3));
        }
    }
    out
}

#[test]
fn adding_a_member_diversifies_but_never_inflates_its_class() {
    let pregen = medium();
    let vanilla = geology::vanilla();
    let extended = extended_set(false);
    // The vanilla-side mudstone index is no longer needed: the positional
    // event-for-event comparison it served became meaningless once
    // `deposit_deep_history` run-length coalesces same-member runs (journal/0055).
    let _mudstone_v = vanilla.member_index("dc:geo/mudstone").unwrap();
    let mudstone_e = extended.member_index("dc:geo/mudstone").unwrap();
    let siltstone_e = extended.member_index("zz:geo/siltstone").unwrap();

    let mut g_v = WorldGenerator::with_geology(&pregen, vanilla.clone());
    let mut g_e = WorldGenerator::with_geology(&pregen, extended.clone());
    let cols_v = sample_columns(&mut g_v, 14);
    let cols_e = sample_columns(&mut g_e, 14);

    let fine_thickness = |set: &GeologySet, col: &dc_worldgen::collapse::ColumnRec| -> f64 {
        col.strata
            .events
            .iter()
            .filter(|e| set.member(e.member).class == CLASS_CLASTIC_FINE)
            .map(|e| f64::from(e.thickness_m))
            .sum()
    };

    let mut new_member_hits = 0usize;
    let mut old_member_hits = 0usize;
    for (cv, ce) in cols_v.iter().zip(&cols_e) {
        // The class share (how much fine stratum exists) is untouched by
        // adding a member — only WHO fills it changes.
        // Metres, not voxel counts, since journal/0055 — so this compares with a
        // tolerance. The two sets coalesce *adjacent same-member* runs into
        // different groupings (that is the whole point of adding a member), so
        // the same total is summed in a different order and the last bits of the
        // f32→f64 accumulation differ. 1e-4 m is 0.1 mm against ~7.6 m; a real
        // class-share inflation is a whole recorded bed.
        let (a, b) = (fine_thickness(&vanilla, cv), fine_thickness(&extended, ce));
        assert!(
            (a - b).abs() < 1e-4,
            "adding a member must not inflate the class share: {a} vs {b}"
        );
        for e in &ce.strata.events {
            if e.member == siltstone_e {
                new_member_hits += 1;
            }
            if e.member == mudstone_e {
                old_member_hits += 1;
            }
        }
        // Where the vanilla world laid a fine-clastic bed, the extended world
        // laid one of the same class and thickness at the same stratigraphic
        // position.
        //
        // **Compared as a class profile, not event-for-event** (journal/0055):
        // `deposit_deep_history` run-length-coalesces adjacent units that resolve
        // to the *same member*, and diversifying a class is precisely a change to
        // which runs are adjacent-and-equal — so the two sets legitimately hold
        // different numbers of events describing the same column. The invariant
        // that must not move is the **class geography**: the same classes in the
        // same order for the same metres.
        let profile = |set: &GeologySet, col: &dc_worldgen::collapse::ColumnRec| {
            let mut out: Vec<(String, f64)> = Vec::new();
            for e in &col.strata.events {
                let c = set.member(e.member).class.clone();
                match out.last_mut() {
                    Some((k, m)) if *k == c => *m += f64::from(e.thickness_m),
                    _ => out.push((c, f64::from(e.thickness_m))),
                }
            }
            out
        };
        let (pv, pe) = (profile(&vanilla, cv), profile(&extended, ce));
        assert_eq!(pv.len(), pe.len(), "class profile length moved");
        for ((kv, mv), (ke, me)) in pv.iter().zip(&pe) {
            assert_eq!(kv, ke, "class order moved");
            assert!(
                (mv - me).abs() < 1e-4,
                "class thickness moved: {mv} vs {me}"
            );
        }
    }
    assert!(
        new_member_hits > 0,
        "the added member must actually appear (diversification)"
    );
    assert!(
        old_member_hits > 0,
        "the original member must survive (redistribution, not replacement)"
    );
}

#[test]
fn placer_follows_the_sorted_gradient() {
    let set = geology::vanilla();
    let ore = set.member_index("dc:geo/gold-dust").unwrap();
    let threshold = member_settle_threshold(&set, ore);
    // The threshold must sit INSIDE the achievable energy range (channel
    // widths cap at 40), or the mechanism would be dead code in worlds.
    assert!(
        threshold > 5.0 && threshold < 40.0,
        "ore settle threshold {threshold} outside the fluvial energy range"
    );

    // Sweep energy down the fan: proximal (high E) to toe (E → 0).
    let energies: Vec<f64> = (1..=80).map(|k| f64::from(k) * 0.5).collect();
    let mut prev_coarse = f64::INFINITY;
    let mut prev_ore = u8::MAX;
    let mut peak_ore = 0u8;
    let mut peak_e = 0.0f64;
    for &e in energies.iter().rev() {
        // Walking DOWN the energy gradient:
        let c = coarse_fraction(e);
        let k = ore_eighths(e, threshold);
        // (1) grain size sorts monotonically: the coarse fraction only falls.
        assert!(
            c <= prev_coarse + 1e-12,
            "coarse fraction must fall down-fan"
        );
        prev_coarse = c;
        if e >= threshold {
            // (2) above the settle threshold the flow still carries the ore.
            assert_eq!(k, 0, "ore must not settle at energy {e} >= {threshold}");
            assert_eq!(prev_ore, u8::MAX, "barren zone must be proximal only");
        } else {
            // (3) below it, concentration only falls toward the toe.
            if prev_ore != u8::MAX {
                assert!(k <= prev_ore, "ore grade must thin toward the toe");
            }
            prev_ore = k;
            if k >= peak_ore {
                peak_ore = k.max(peak_ore);
                if k == peak_ore && peak_e == 0.0 {
                    peak_e = e;
                }
            }
        }
        if k > peak_ore {
            peak_ore = k;
        }
    }
    assert!(
        peak_ore > 0,
        "somewhere on the fan the ore must concentrate"
    );
    // (4) the peak sits just below the threshold — the winnowing band —
    // which is also where the coarse fraction is highest among ore-bearing
    // reaches: the ore rides the coarse body.
    let first_below = energies.iter().rev().find(|&&e| e < threshold).unwrap();
    assert_eq!(
        ore_eighths(*first_below, threshold),
        peak_ore,
        "peak grade must sit just below the settle threshold"
    );

    // World-level: every enrichment the placer pass wrote lives inside a
    // coarse (alluvial) event, grades stay in 1..=3 eighths, and both
    // ore-bearing and barren alluvial columns exist. Fluvial energy decays
    // within a chunk of the channel, so sample across real river segments:
    // columns around the midpoints of the first few river chords.
    let pregen = medium();
    let mut river_midpoints = Vec::new();
    for (i, c) in pregen.grid.cells.iter().enumerate() {
        if !(c.river && c.is_land()) {
            continue;
        }
        let Some(to) = c.flow_to else { continue };
        let (agx, agy) = pregen.grid.coords(i);
        let (bgx, bgy) = pregen.grid.coords(to as usize);
        let a = pregen.grid.cell_center_voxel(agx, agy);
        let b = pregen.grid.cell_center_voxel(bgx, bgy);
        river_midpoints.push(((a.0 + b.0) / 2, (a.1 + b.1) / 2));
        if river_midpoints.len() >= 4 {
            break;
        }
    }
    assert!(!river_midpoints.is_empty(), "medium world has no rivers?");
    let mut g = WorldGenerator::with_geology(&pregen, set.clone());
    let mut cols = Vec::new();
    for (mx, mz) in river_midpoints {
        let (ccx, ccz) = (mx.div_euclid(32), mz.div_euclid(32));
        for dz in -8i64..8 {
            for dx in -8i64..8 {
                cols.push(g.column_record(ccx + dx, ccz + dz));
            }
        }
    }
    let mut ore_columns = 0usize;
    let mut barren_alluvial = 0usize;
    for col in cols {
        for e in &col.strata.events {
            if let Some((m, k)) = e.ore {
                assert_eq!(set.member(m).class, "dc:ore/placer");
                assert_eq!(
                    set.member(e.member).class,
                    CLASS_CLASTIC_COARSE,
                    "ore must ride the graded coarse body"
                );
                assert!((1..=3).contains(&k), "grade out of range: {k} eighths");
                ore_columns += 1;
            } else if set.member(e.member).class == CLASS_CLASTIC_COARSE && e.thickness_m > 0.0 {
                barren_alluvial += 1;
            }
        }
    }
    assert!(ore_columns > 0, "no placer deposits found in the sample");
    assert!(
        barren_alluvial > 0,
        "every alluvial body carried ore — the threshold is not selecting"
    );
}

/// ROADMAP 3c-2 seam proof: a worldgen chunk with a placer body produces
/// **mixed-voxel render data** at the mesher input. `chunk_contents` resolves
/// the order-dependent mixture table away into a canonical `ContentsGrid`; a
/// placer-enriched clastic voxel therefore surfaces as contents mixing the
/// clastic host with gold-dust — exactly what the mesher dithers. Air chunks
/// carry no contents at all (the debris-free zero-cost invariant).
#[test]
fn render_contents_carry_the_placer_mixture() {
    let pregen = medium();
    // River midpoints — the placer fans hang off channels (same discovery as
    // the gradient test above).
    let mut midpoints = Vec::new();
    for (i, c) in pregen.grid.cells.iter().enumerate() {
        if !(c.river && c.is_land()) {
            continue;
        }
        let Some(to) = c.flow_to else { continue };
        let (agx, agy) = pregen.grid.coords(i);
        let (bgx, bgy) = pregen.grid.coords(to as usize);
        let a = pregen.grid.cell_center_voxel(agx, agy);
        let b = pregen.grid.cell_center_voxel(bgx, bgy);
        midpoints.push(((a.0 + b.0) / 2, (a.1 + b.1) / 2));
        if midpoints.len() >= 6 {
            break;
        }
    }
    assert!(!midpoints.is_empty(), "medium world has no rivers?");

    let mut g = WorldGenerator::with_geology(&pregen, geology::vanilla());
    let mut mixed_with_gold = 0usize;
    let mut saw_uniform_clastic = false;
    'search: for (mx, mz) in midpoints {
        let (ccx, ccz) = (mx.div_euclid(32), mz.div_euclid(32));
        for dz in -8i64..8 {
            for dx in -8i64..8 {
                let (cx, cz) = (ccx + dx, ccz + dz);
                let surf_y = g.surface_chunk_y(cx, cz);
                // The placer rides shallow clastic; scan the surface chunk and
                // a few below so we cover its depth window across the footprint.
                for cy in (surf_y - 3)..=surf_y {
                    let pos = ChunkPos::new(cx as i32, cy, cz as i32);
                    let Some(grid) = g.chunk_contents(pos) else {
                        continue;
                    };
                    for c in grid.palette() {
                        let slots = c.filled_slots();
                        if slots.is_empty() {
                            continue;
                        }
                        // A uniform loose clastic band (sandstone / mudstone) —
                        // the ground the ore is disseminated into.
                        if slots.iter().all(|&m| m == slots[0])
                            && matches!(
                                slots[0],
                                MaterialId::SANDSTONE
                                    | MaterialId::MUDSTONE
                                    | MaterialId::SILTSTONE
                                    | MaterialId::CONGLOMERATE
                            )
                        {
                            saw_uniform_clastic = true;
                        }
                        if slots.contains(&MaterialId::GOLD_DUST) {
                            // The dense grain rides inside a clastic host: the
                            // mixture the mesher dithers.
                            assert!(
                                slots.iter().any(|&m| m != MaterialId::GOLD_DUST),
                                "placer ore must ride inside a clastic host, not fill a voxel"
                            );
                            mixed_with_gold += 1;
                            if mixed_with_gold >= 3 {
                                break 'search;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        mixed_with_gold > 0,
        "no placer-mixed render contents found — the seam is not carrying mixtures"
    );
    assert!(
        saw_uniform_clastic,
        "expected uniform clastic contents alongside the placer mixtures"
    );

    // A chunk far above any surface is debris-free: no render contents, zero
    // cost, exactly as it attaches no storage sidecar.
    assert!(
        g.chunk_contents(ChunkPos::new(0, 4000, 0)).is_none(),
        "an all-air chunk must carry no material contents"
    );
}

#[test]
fn strata_records_are_province_and_climate_driven() {
    use dc_core::materials::geology::CLASS_IGNEOUS_INTRUSIVE;
    let pregen = medium();
    let set = geology::vanilla();
    let mut g = WorldGenerator::with_geology(&pregen, set.clone());

    let mut land_cols = 0usize;
    let mut clastic_cols = 0usize;
    let mut bare_cols = 0usize;
    let mut orogenic_with_basement = 0usize;
    let mut orogenic = 0usize;
    for cz in -20i64..20 {
        for cx in -20i64..20 {
            let (col, cellp) = {
                let col = g.column_record(cx * 4, cz * 4);
                let (gx, gy) = pregen
                    .grid
                    .cell_of_voxel(cx * 4 * 32 + 16, cz * 4 * 32 + 16);
                let p = i32::try_from(gx)
                    .ok()
                    .zip(i32::try_from(gy).ok())
                    .and_then(|(a, b)| pregen.grid.get(a, b))
                    .map(|c| c.provenance);
                (col, p)
            };
            let land = col.heights.iter().sum::<i32>() > 0;
            if !land || col.wilds {
                continue;
            }
            land_cols += 1;
            // The clastic veneer is deposited where there is loose cover to
            // deposit. **This used to assert "every land column"** — pre-0053
            // the veneer budget was `1 + precip*2.5` with a floor of one voxel,
            // so a clastic band was unconditional. Now the budget comes from the
            // deep sim's carried regolith plane `H` and a column the sim scoured
            // bare gets none (journal/0053), so the honest claim is
            // *predominance*, asserted below, plus the deposition-context
            // invariants per event.
            let clastic: Vec<_> = col
                .strata
                .events
                .iter()
                .filter(|e| {
                    let c = &set.member(e.member).class;
                    c == CLASS_CLASTIC_FINE || c == CLASS_CLASTIC_COARSE
                })
                .collect();
            if clastic.is_empty() {
                bare_cols += 1;
            } else {
                clastic_cols += 1;
            }
            for e in &col.strata.events {
                assert!(e.precip >= 0.0 && e.precip <= 1.0);
                assert!(e.temp_c.is_finite());
            }
            // Province-driven: orogeny/arc columns carry an intrusive basement
            // (granite or diorite — the roster widened) at the BOTTOM of the
            // record. The specific member is province/depth-selected, never
            // weather-selected.
            if matches!(cellp, Some(Provenance::Orogeny | Provenance::Arc)) {
                orogenic += 1;
                let basement_is_intrusive = col
                    .strata
                    .events
                    .first()
                    .is_some_and(|e| set.member(e.member).class == CLASS_IGNEOUS_INTRUSIVE);
                if basement_is_intrusive {
                    orogenic_with_basement += 1;
                }
            }
        }
    }
    assert!(land_cols > 50, "sample found too little land: {land_cols}");
    println!("land columns {land_cols}: {clastic_cols} with a clastic veneer, {bare_cols} bare");
    assert!(
        clastic_cols * 2 > land_cols,
        "most land columns should still carry a clastic veneer: {clastic_cols} of {land_cols}"
    );
    assert!(orogenic > 0, "sample crossed no orogenic provinces");
    assert_eq!(
        orogenic, orogenic_with_basement,
        "every orogenic/arc land column must record an intrusive basement"
    );
}

/// 3d mechanic 2: the chunk-line family cutover, smoothed. The material-tier
/// boundary dither re-selects a class's host member per voxel-column from a
/// bilinear field whose corners are the chunk-column hashes — so a member
/// contact wanders like a facies boundary and can fall INSIDE a chunk, which a
/// per-chunk selection (constant across the whole 32-column footprint) can
/// never do. Isolated with a two-member, equal-fitness class so only the
/// dithered draw decides.
#[test]
fn family_contacts_wander_off_the_chunk_grid() {
    let mut b = GeologySet::builder();
    b.declare_class(CLASS_CLASTIC_FINE).unwrap();
    for id in ["dc:geo/aaa", "dc:geo/bbb"] {
        b.add_member(GeoMemberDef {
            id: id.into(),
            class: CLASS_CLASTIC_FINE.into(),
            material: MaterialId::MUDSTONE,
            window: FormationWindow::ANY,
            abundance: 1.0,
            habit: GeoHabit::Blanket,
            hardness: 0.5,
            erodibility: 0.5,
        })
        .unwrap();
    }
    let set = b.build();
    let m_a = set.member_index("dc:geo/aaa").unwrap();
    let m_b = set.member_index("dc:geo/bbb").unwrap();
    let seed = 0xDEED_1234_5678u64;
    let event = StrataEvent {
        member: m_a,
        thickness_m: 3.6,
        temp_c: 10.0,
        precip: 0.5,
        depth_m: 1.0,
        sel_salt: 0x5700_000A,
        sel_tag: 3,
        ore: None,
        accessory: None,
        // This suite is *about* the veneer's per-voxel dither, so the event
        // declares itself ditherable (P11 slice 1; a deep-history event would
        // not, and `dithered_member` would hand back the recorded member).
        dither: true,
    };
    // Addressed by the absolute voxel since journal/0129 (the octaves selection
    // field is world-addressed; the chunk/offset split was arithmetic the caller
    // was doing for it).
    let sample = |cx: i64, x: usize| dithered_member(&set, seed, &event, cx * 32 + x as i64, 0);

    // Determinism.
    assert_eq!(sample(3, 7), sample(3, 7));

    // Sweep a long transect (12 chunks × 32 columns): the contact interleaves
    // both members and falls interior to at least one chunk.
    let mut saw_a = false;
    let mut saw_b = false;
    let mut interior_transitions = 0usize;
    let mut border_only = true;
    for cx in -6i64..6 {
        let mut prev: Option<_> = None;
        for x in 0..32usize {
            let m = sample(cx, x);
            saw_a |= m == m_a;
            saw_b |= m == m_b;
            if let Some(p) = prev
                && p != m
            {
                interior_transitions += 1;
                border_only = false;
            }
            prev = Some(m);
        }
    }
    assert!(
        saw_a && saw_b,
        "the transect must cross both members (interleaving), not one flat family"
    );
    assert!(
        interior_transitions > 0 && !border_only,
        "a family contact must fall INSIDE a chunk — a per-chunk selection could not"
    );

    // Registration-order independence: the canonical (id-sorted) order wins, so
    // swapping which member registers first cannot move a single contact.
    let mut b2 = GeologySet::builder();
    b2.declare_class(CLASS_CLASTIC_FINE).unwrap();
    for id in ["dc:geo/bbb", "dc:geo/aaa"] {
        b2.add_member(GeoMemberDef {
            id: id.into(),
            class: CLASS_CLASTIC_FINE.into(),
            material: MaterialId::MUDSTONE,
            window: FormationWindow::ANY,
            abundance: 1.0,
            habit: GeoHabit::Blanket,
            hardness: 0.5,
            erodibility: 0.5,
        })
        .unwrap();
    }
    let set2 = b2.build();
    for cx in -6i64..6 {
        for x in 0..32usize {
            assert_eq!(
                dithered_member(&set, seed, &event, cx * 32 + x as i64, 0),
                dithered_member(&set2, seed, &event, cx * 32 + x as i64, 0),
                "registration order moved a dithered contact"
            );
        }
    }
}

/// 3d mechanic 4: accessory inclusions as pore partials. The igneous pass
/// emplaces an accessory mineral sparsely into its event; the render seam
/// surfaces it as a host-rock structure with the accessory in the **pore
/// slots** — the placer pattern in igneous dress, rendered by the existing
/// mixture dither with zero renderer-side code.
#[test]
fn accessory_inclusions_ride_igneous_pores() {
    let pregen = medium();
    let set = geology::vanilla();
    let mut g = WorldGenerator::with_geology(&pregen, set.clone());

    let mut igneous_events = 0usize;
    let mut with_accessory = 0usize;
    let mut accessory_cols: Vec<(i64, i64)> = Vec::new();
    for cz in -20i64..20 {
        for cx in -20i64..20 {
            let (ccx, ccz) = (cx * 4, cz * 4);
            let col = g.column_record(ccx, ccz);
            if col.wilds {
                continue;
            }
            let mut col_has_accessory = false;
            for e in &col.strata.events {
                let class = set.member(e.member).class.as_str();
                if class == CLASS_IGNEOUS_INTRUSIVE || class == CLASS_IGNEOUS_EXTRUSIVE {
                    igneous_events += 1;
                    if let Some((acc, k)) = e.accessory {
                        // **The pore slot carries two riders** (journal/0099).
                        // A weathering-front band is an igneous-class event whose
                        // rider is the LOOSE weathering product, up to 7/8 — not
                        // an accessory mineral, and not what this test is about.
                        // Unreachable here (this world runs `weather_inventory`
                        // off, so no front is emplaced); the guard is so a
                        // flag-on world fails somewhere honest instead of here.
                        if set.member(acc).class == CLASS_CLASTIC_FINE {
                            continue;
                        }
                        assert_eq!(
                            set.member(acc).class,
                            CLASS_ACCESSORY_MAFIC,
                            "accessory must come from the accessory class"
                        );
                        assert!((1..=2).contains(&k), "accessory eighths out of range: {k}");
                        with_accessory += 1;
                        col_has_accessory = true;
                    }
                }
            }
            if col_has_accessory {
                accessory_cols.push((ccx, ccz));
            }
        }
    }
    assert!(igneous_events > 0, "sample crossed no igneous provinces");
    assert!(with_accessory > 0, "no accessory inclusions were emplaced");

    // The render seam surfaces the accessory as a pore partial of the host
    // igneous rock (structure slots host, pore slots accessory).
    let mut olivine_in_host_pores = false;
    'scan: for &(ccx, ccz) in accessory_cols.iter().take(12) {
        let surf_y = g.surface_chunk_y(ccx, ccz);
        for cy in (surf_y - 5)..=surf_y {
            let Some(grid) = g.chunk_contents(ChunkPos::new(ccx as i32, cy, ccz as i32)) else {
                continue;
            };
            for c in grid.palette() {
                if c.pore_fill().contains(&MaterialId::OLIVINE) {
                    assert!(
                        !c.structure().is_empty()
                            && c.structure().iter().all(|&m| m != MaterialId::OLIVINE),
                        "accessory must ride the host rock's pores, not fill the structure"
                    );
                    olivine_in_host_pores = true;
                    break 'scan;
                }
            }
        }
    }
    assert!(
        olivine_in_host_pores,
        "accessory olivine never surfaced in a host igneous rock's pore slots"
    );
}
