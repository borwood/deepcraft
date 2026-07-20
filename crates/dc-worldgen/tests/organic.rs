//! Organic materials + the biotic production flip (journal/0026).
//!
//! The S10 spike left one honest gap: the deep-time recorder tagged units with a
//! `Biofacies` axis, but `geology::deep_class` selected a content class from the
//! `DepTag`'s env/energy alone — so an organic unit collapsed as ordinary
//! clastic and the measured 24 m coal seam was unminable. These are the
//! falsifiers for closing it:
//!
//! - the biotic layer is **on in production** (`production_config`), so the
//!   organic facies exist in every new world;
//! - the seam at the S10-measured site is **coal**, at the block tier and in the
//!   material sidecar — a player can dig it;
//! - routing is by **biofacies, not by flow energy**: the seam's own tag is
//!   `Sa/A/L` (subaerial, arid, LOW energy), which the pre-0026 rule sent to
//!   clastic-fine → mudstone. Same tag, different rock, because the biotic axis
//!   now wins;
//! - adding an organic member **diversifies its class, never inflates it**;
//! - and organic member selection is registration-order independent.

use dc_core::materials::geology::{
    self, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_ORGANIC_COAL, CLASS_ORGANIC_PEAT,
    CLASS_ORGANIC_SOIL, FormationContext, FormationWindow, GeoHabit, GeoMemberDef, GeologySet,
};
use dc_core::{Block, ChunkPos, MaterialId, VoxelContents};
use dc_worldgen::deeptime::{Biofacies, DeepConfig};
use dc_worldgen::pregen::CellGrid;
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

/// The S10-measured coal site (docs/spikes/S10-results.md § 1): a 24.03 m seam
/// over a marine section, capped by fire-bearing alluvium.
const COAL_SITE: (i64, i64) = (107_338, 58_787);

fn medium() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    })
}

/// The chunk-column containing a world voxel.
fn column_of(vx: i64, vz: i64) -> (i64, i64) {
    (vx.div_euclid(32), vz.div_euclid(32))
}

#[test]
fn production_config_runs_the_biotic_layer() {
    // The GO action itself (ecology.md § DECIDED 2026-07-20). A cheap, direct
    // assertion so the flip can never be reverted silently by a merge.
    let cells = CellGrid {
        w: 17,
        cells: Vec::new(),
    };
    let cfg = dc_worldgen::deeptime::production_config(&cells, SEED);
    assert!(
        cfg.biotic,
        "production worlds must run the biotic layer (S10 GO)"
    );
    assert!(cfg.record, "and must keep the strata record");
    // The default stays off: the spike's abiotic baseline is still reachable.
    assert!(!DeepConfig::default().biotic);
}

#[test]
fn every_biofacies_routes_to_a_class_the_vanilla_set_can_fill() {
    // Classes-as-contracts: the routing is only honest if every class it can
    // name actually has a member. (The pipeline enforces this at world build;
    // this asserts the content side directly, with no pregen cost.)
    let set = geology::vanilla();
    let ctx = FormationContext {
        temp_c: 12.0,
        precip: 0.5,
        depth_m: 8.0,
    };
    for (class, want) in [
        (CLASS_ORGANIC_COAL, MaterialId::COAL),
        (CLASS_ORGANIC_PEAT, MaterialId::PEAT),
        (CLASS_ORGANIC_SOIL, MaterialId::CARBONACEOUS_MUDSTONE),
    ] {
        let (_, m) = set
            .select(class, &ctx, 0.5)
            .unwrap_or_else(|| panic!("{class} must have a member"));
        assert_eq!(m.material, want, "{class}");
    }
}

#[test]
fn organic_member_selection_is_registration_order_independent() {
    let members = geology::vanilla_members();
    let build = |reversed: bool| {
        let mut ms = members.clone();
        if reversed {
            ms.reverse();
        }
        let mut b = GeologySet::builder();
        for c in geology::v1_classes() {
            b.declare_class(c).unwrap();
        }
        for m in ms {
            b.add_member(m).unwrap();
        }
        b.build()
    };
    let (a, z) = (build(false), build(true));
    assert_eq!(a.member_index("dc:geo/coal"), z.member_index("dc:geo/coal"));
    for k in 0..64 {
        let ctx = FormationContext {
            temp_c: -10.0 + f64::from(k),
            precip: f64::from(k) / 64.0,
            depth_m: f64::from(k) * 30.0,
        };
        let u = f64::from(k) / 64.0;
        for class in geology::v1_classes() {
            assert_eq!(
                a.select(class, &ctx, u).map(|(i, _)| i),
                z.select(class, &ctx, u).map(|(i, _)| i),
                "{class} moved under registration permutation"
            );
        }
    }
}

/// Vanilla plus a second coal member — the class-share invariant's organic
/// instance. Anthracite is a legitimate *future* member (the coal class's depth
/// axis is the rank axis); here it exists only to be diluted.
fn with_second_coal() -> GeologySet {
    let mut members = geology::vanilla_members();
    members.push(GeoMemberDef {
        id: "zz:geo/anthracite".into(),
        class: CLASS_ORGANIC_COAL.into(),
        material: MaterialId::COAL,
        window: FormationWindow {
            temp_c: (-10.0, 40.0),
            precip: (0.0, 1.0),
            depth_m: (0.0, 40_000.0),
        },
        abundance: 1.0,
        habit: GeoHabit::Blanket,
        hardness: 0.4,
        erodibility: 0.5,
    });
    let mut b = GeologySet::builder();
    for c in geology::v1_classes() {
        b.declare_class(c).unwrap();
    }
    for m in members {
        b.add_member(m).unwrap();
    }
    b.build()
}

/// The heavyweight proof. One Medium pregen (the ~14 s ritual, biology ON) does
/// every world-level assertion, so the suite pays for world creation once.
#[test]
fn the_measured_coal_seam_is_coal_a_player_can_dig() {
    let pregen = medium();

    // ---- 1. the record still says what S10 measured -----------------------
    let rec = pregen
        .deep
        .record_at_voxel(COAL_SITE.0, COAL_SITE.1)
        .expect("the coal site is inside the pregen grid");
    let thickest_coal = rec
        .units
        .iter()
        .filter(|u| u.tag.biota == Biofacies::Coal)
        .map(|u| u.thickness_m)
        .fold(0.0f64, f64::max);
    assert!(
        thickest_coal > 20.0,
        "S10 measured a 24.03 m seam here; the record now holds {thickest_coal:.2} m"
    );

    let mut g = WorldGenerator::new(&pregen);
    let (cx, cz) = column_of(COAL_SITE.0, COAL_SITE.1);
    let col = g.column_record(cx, cz);
    let set = geology::vanilla();

    // ---- 2. the seam survives collapse as the COAL class -------------------
    let coal_vox: u32 = col
        .strata
        .events
        .iter()
        .filter(|e| set.member(e.member).class == CLASS_ORGANIC_COAL)
        .map(|e| u32::from(e.thickness_vox))
        .sum();
    assert!(
        coal_vox >= 20,
        "expected the 24 m seam as ~27 voxels of coal in the collapsed column, got {coal_vox}"
    );
    assert!(
        col.strata
            .events
            .iter()
            .any(|e| set.member(e.member).material == MaterialId::COAL),
        "the collapsed column must contain the coal MEMBER, not just the class"
    );

    // ---- 3. routing is by biofacies, NOT by flow energy --------------------
    // The seam's own tag is `Sa/A/L` — subaerial, arid, LOW energy. The pre-0026
    // rule sent every subaerial low-energy unit to clastic-fine, i.e. mudstone.
    // In the same column, `Sa/A/M` mineral units still take the clastic-coarse
    // road, so this is a biofacies win, not a blanket override.
    let seam = rec
        .units
        .iter()
        .find(|u| u.tag.biota == Biofacies::Coal && u.thickness_m > 20.0)
        .expect("the thick seam");
    assert_eq!(
        seam.tag.energy,
        dc_worldgen::deeptime::EnergyBand::Low,
        "the seam is a LOW-energy unit — the old rule would have made it mudstone"
    );
    let classes: Vec<&str> = col
        .strata
        .events
        .iter()
        .map(|e| set.member(e.member).class.as_str())
        .collect();
    assert!(
        classes.contains(&CLASS_ORGANIC_COAL),
        "coal reached the column"
    );
    assert!(
        classes
            .iter()
            .any(|c| *c == CLASS_CLASTIC_COARSE || *c == CLASS_CLASTIC_FINE),
        "mineral units in the same column still collapse as clastic"
    );

    // ---- 4. it is diggable: blocks + material contents ---------------------
    let top = g.surface_chunk_y(cx, cz);
    let mut saw_coal_block = false;
    let mut saw_coal_material = false;
    for cy in (top - 6)..=top {
        let pos = ChunkPos::new(cx as i32, cy, cz as i32);
        let (chunk, _) = g.generate_chunk_with_materials(pos);
        if chunk.blocks().contains(&Block::Coal) {
            saw_coal_block = true;
        }
        if let Some(grid) = g.chunk_contents(pos) {
            for z in 0..32 {
                for y in 0..32 {
                    for x in 0..32 {
                        if contents_contain(&grid.get(x, y, z), MaterialId::COAL) {
                            saw_coal_material = true;
                        }
                    }
                }
            }
        }
    }
    assert!(
        saw_coal_block,
        "no Block::Coal in the six chunks below the surface at the measured seam"
    );
    assert!(
        saw_coal_material,
        "no MaterialId::COAL in the voxel contents at the measured seam"
    );

    // ---- 5. the class-share invariant, organic instance --------------------
    // Adding a member to an organic class diversifies WHO fills it and never
    // changes HOW MUCH of it there is. Shares this test's pregen deliberately:
    // the biotic flip already put a ~14 s ritual behind every world-level suite,
    // so a second one to make the same point is not worth the wall clock.
    let extended = with_second_coal();
    let coal_class_vox = |set: &GeologySet, g: &mut WorldGenerator, cx: i64, cz: i64| -> u32 {
        g.column_record(cx, cz)
            .strata
            .events
            .iter()
            .filter(|e| set.member(e.member).class == CLASS_ORGANIC_COAL)
            .map(|e| u32::from(e.thickness_vox))
            .sum()
    };
    let mut ge = WorldGenerator::with_geology(&pregen, extended.clone());
    let mut total = 0u32;
    for dz in -2..=2i64 {
        for dx in -2..=2i64 {
            let (a, b) = (
                coal_class_vox(&set, &mut g, cx + dx, cz + dz),
                coal_class_vox(&extended, &mut ge, cx + dx, cz + dz),
            );
            assert_eq!(
                a, b,
                "adding a coal member changed the coal class share at ({dx},{dz})"
            );
            total += a;
        }
    }
    assert!(total > 0, "the sampled columns must actually contain coal");
}

/// Does a voxel's canonical contents hold this material in any role?
fn contents_contain(c: &VoxelContents, m: MaterialId) -> bool {
    c.structure().contains(&m) || c.pore_fill().contains(&m) || c.debris().contains(&m)
}
