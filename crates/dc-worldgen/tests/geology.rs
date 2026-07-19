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
    self, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, FormationWindow, GeoHabit, GeoMemberDef,
    GeologySet,
};
use dc_core::{Chunk, ChunkPos, MaterialId};
use dc_worldgen::geology::{coarse_fraction, member_settle_threshold, ore_eighths};
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
        h ^= u64::from(*b as u16);
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
    let mudstone_v = vanilla.member_index("dc:geo/mudstone").unwrap();
    let mudstone_e = extended.member_index("dc:geo/mudstone").unwrap();
    let siltstone_e = extended.member_index("zz:geo/siltstone").unwrap();

    let mut g_v = WorldGenerator::with_geology(&pregen, vanilla.clone());
    let mut g_e = WorldGenerator::with_geology(&pregen, extended.clone());
    let cols_v = sample_columns(&mut g_v, 14);
    let cols_e = sample_columns(&mut g_e, 14);

    let fine_thickness = |set: &GeologySet, col: &dc_worldgen::collapse::ColumnRec| -> u32 {
        col.strata
            .events
            .iter()
            .filter(|e| set.member(e.member).class == CLASS_CLASTIC_FINE)
            .map(|e| u32::from(e.thickness_vox))
            .sum()
    };

    let mut new_member_hits = 0usize;
    let mut old_member_hits = 0usize;
    for (cv, ce) in cols_v.iter().zip(&cols_e) {
        // The class share (how much fine stratum exists) is untouched by
        // adding a member — only WHO fills it changes.
        assert_eq!(
            fine_thickness(&vanilla, cv),
            fine_thickness(&extended, ce),
            "adding a member must not inflate the class share"
        );
        for e in &ce.strata.events {
            if e.member == siltstone_e {
                new_member_hits += 1;
            }
            if e.member == mudstone_e {
                old_member_hits += 1;
            }
        }
        // Where the vanilla world chose mudstone, the extended world chose
        // SOME fine-clastic member at the same stratigraphic position.
        for (ev, ee) in cv.strata.events.iter().zip(&ce.strata.events) {
            assert_eq!(ev.thickness_vox, ee.thickness_vox);
            if ev.member == mudstone_v {
                assert_eq!(extended.member(ee.member).class, CLASS_CLASTIC_FINE);
            }
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
            } else if set.member(e.member).class == CLASS_CLASTIC_COARSE && e.thickness_vox > 0 {
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

#[test]
fn strata_records_are_province_and_climate_driven() {
    let pregen = medium();
    let set = geology::vanilla();
    let granite = set.member_index("dc:geo/granite").unwrap();
    let mut g = WorldGenerator::with_geology(&pregen, set.clone());

    let mut land_cols = 0usize;
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
            // Climate-driven: every land column deposits at least one
            // clastic stratum, tagged with its deposition climate.
            let clastic: Vec<_> = col
                .strata
                .events
                .iter()
                .filter(|e| {
                    let c = &set.member(e.member).class;
                    c == CLASS_CLASTIC_FINE || c == CLASS_CLASTIC_COARSE
                })
                .collect();
            assert!(!clastic.is_empty(), "land column with no clastic record");
            for e in &col.strata.events {
                assert!(e.precip >= 0.0 && e.precip <= 1.0);
                assert!(e.temp_c.is_finite());
            }
            // Province-driven: orogeny/arc columns carry the intrusive
            // basement at the BOTTOM of the record.
            if matches!(cellp, Some(Provenance::Orogeny | Provenance::Arc)) {
                orogenic += 1;
                if col.strata.events.first().map(|e| e.member) == Some(granite) {
                    orogenic_with_basement += 1;
                }
            }
        }
    }
    assert!(land_cols > 50, "sample found too little land: {land_cols}");
    assert!(orogenic > 0, "sample crossed no orogenic provinces");
    assert_eq!(
        orogenic, orogenic_with_basement,
        "every orogenic/arc land column must record an intrusive basement"
    );
}
