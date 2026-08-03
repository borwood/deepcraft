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
    self, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_ORGANIC_CHARCOAL, CLASS_ORGANIC_COAL,
    CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL, FormationContext, FormationWindow, GeoHabit,
    GeoMemberDef, GeologySet,
};
use dc_core::{Block, ChunkPos, MaterialId, VoxelContents};
use dc_worldgen::deeptime::{Biofacies, DeepConfig, DeepField, EnergyBand};
use dc_worldgen::pregen::{CELL_VOXELS, CellGrid};
use dc_worldgen::{Extent, Pregen, WorldGenerator, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;

/// The row-major deep-cell index NEAREST a world voxel — the address
/// `record_at_voxel` answers, replicated so the test can name the SubCell
/// realizing that same cell inside a chunk (P11 slice 3: a chunk holds up to
/// nine records, and the seam this suite measures lives in one specific cell).
fn nearest_cell_index(deep: &DeepField, vx: i64, vz: i64) -> Option<u32> {
    let (gx, gy) = deep.deep_coords(vx, vz)?;
    let ix = (gx.round() as i64).clamp(0, deep.w as i64 - 1) as u32;
    let iy = (gy.round() as i64).clamp(0, deep.w as i64 - 1) as u32;
    Some(iy * deep.w as u32 + ix)
}

/// Minimum diggable-coal voxels the strongest low-energy seam must render in its
/// collapsed column. Re-baselined for the full-agents flip (journal/0047): the old
/// test froze a 15 m record floor AND required the record-thickest seam to also
/// render at least 15 collapse-voxels, but the flip moved deposition so the
/// record-thickest seams (over 15 m) now bury below the collapse column. Measured
/// post-flip: 7647 seams over 3 m record, the strongest rendering 19 diggable
/// voxels (record 11.9 m) — the mechanism is intact (equal to the pre-flip
/// "~19 voxels"), so the bar stays at the old 15, now met by selecting the
/// strongest seam that surfaces as diggable coal instead of the thickest record
/// seam.
///
/// **Held unchanged through journal/0063 — deliberately, and it was close.**
/// Moving coal promotion onto the burial axis cut coal by ~5.8×: the census
/// behind this constant went from **7 647** record seams over 3 m to **88**, and
/// the strongest diggable seam from 19 collapse-voxels to **16**. The bar is not
/// lowered, because the claim it defends — *a player can find and dig a coal
/// seam* — still holds with a voxel to spare. If a future slice makes coal
/// rarer still, the honest response is to say so and re-baseline with the census
/// printed, not to quietly slide this number down after each change.
///
/// **Re-baselined 15 → 10 by the susceptibility blend (journal/0072) — authorized,
/// and it IS the "future slice" the note above warned about; NEEDS RATIFICATION.**
/// The share-weighted rate gives any near-surface window that *contains* coal a
/// rate pulled up toward coal's (coal is the softest rock, so a coaly bed is
/// recessive), where the old argmax handed a coal-minority window the dominant
/// rock's slower rate. So near-surface coal is preferentially stripped, and this
/// Medium seed's census dropped from 88 record seams over 3 m (strongest 16
/// diggable) to **10 record seams over 3 m, strongest 11 diggable collapse-voxels**
/// (measured, `--nocapture`). The claim still holds — 11 voxels is ≈ 10 m of coal a
/// player can dig, and the walk-0071 seam the user cut by hand and called "looks
/// great" was 3 voxels — so the bar drops to 10 (one voxel of margin under the new
/// strongest), printed here rather than slid silently. The reduction in coal
/// diggability is a world-scale gameplay consequence flagged for the user.
///
/// **Re-baselined 10 → 6 by the geotherm (journal/0093) — authorized; NEEDS
/// RATIFICATION.** The geotherm recalibrated coalification onto a real temperature
/// (`COAL_ONSET_C` 8 → 22 °C) and **relocated** coal from the deepest-buried seams
/// to warm-crust ones; the deep thick seams that used to carry the strongest
/// diggable coal now often stay peat. This Medium seed's census dropped from 10
/// record seams over 3 m (strongest 11 diggable) to **10 record seams over 3 m,
/// strongest 8 diggable collapse-voxels** (record 5.8 m; measured, `--nocapture`).
/// The claim still holds — a player finds and digs a real coal seam — but a
/// thinner one, so the bar drops to 6 (margin under the strongest). This is the
/// coal *amount* the geotherm produces at onset 22 °C, and both the amount and
/// this floor are an **appearance call the user judges on the geotherm walk**
/// (§14: "we measure the shift as the reason to walk").
const MIN_DIGGABLE_COAL_VOX: u32 = 6;

/// N=2 voxel edge, metres. Since journal/0055 the record's thicknesses are
/// metres and the quantization happens once, per voxel span — so a test that
/// wants "how many voxels of coal" sums the metres and divides *once*, which is
/// the same `round Σ` discipline the generator now uses.
const VOXEL_M: f64 = 0.9;

/// The S10-measured coal site (docs/spikes/S10-results.md § 1) *was* a fixed
/// voxel, `(107_338, 58_787)` — a 24.03 m seam over a marine section. But a
/// hard-coded location is a golden that any upstream climate change invalidates:
/// the ratified **zonal circulation** slice (journal/0037) shifted precipitation
/// for every new world, and this world's thickest coal swamp moved off that exact
/// cell (the S10 voxel now records 0 m of coal). What this test *proves* — a coal
/// seam is biofacies-tagged, survives collapse as the COAL class, and ends up
/// diggable — is a claim about the pipeline, not a location, so we now find the
/// seam wherever the current climate puts it. See [`thickest_coal_voxel`].
fn medium() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    })
}

/// Every deep cell holding a coal seam thicker than `min_m`, as the world voxel
/// at that cell's centre, sorted thickest-first. Inverts the deep-field
/// voxel↔cell mapping ([`dc_worldgen::deeptime::DeepField`]) so the
/// collapse/dig assertions downstream address the same cell. Returns
/// `(vx, vz, thickest_coal_m)` per candidate — the test walks them thickest-first
/// and takes the first that also survives collapse to diggable coal (a thick
/// *record* seam can be buried below the collapse column; the S10 site was
/// near-surface, and after the climate move we re-find a near-surface one).
fn coal_seam_candidates(pregen: &Pregen, min_m: f64) -> Vec<(i64, i64, f64)> {
    let deep = &pregen.deep;
    let (w, wp) = (deep.w, deep.wp);
    // Inverse of `DeepField::deep_coords`: note the half-extent is integer
    // `wp / 2` (matching `(self.wp / 2) as f64` there), NOT `wp / 2.0` — for odd
    // `wp` the half-cell difference lands on the wrong deep cell.
    let half = (wp / 2) as f64;
    let voxel = |c: f64| (((c + 0.5) * wp as f64 / w as f64 - half) * CELL_VOXELS as f64) as i64;
    let mut out: Vec<(i64, i64, f64)> = deep
        .strata
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let coal = s
                .units
                .iter()
                .filter(|u| u.tag().biota == Biofacies::Coal)
                .map(|u| u.thickness_m())
                .fold(0.0f64, f64::max);
            (coal > min_m).then(|| (voxel((i % w) as f64), voxel((i / w) as f64), coal))
        })
        .collect();
    out.sort_by(|a, b| b.2.total_cmp(&a.2));
    out
}

/// The chunk-column containing a world voxel.
fn column_of(vx: i64, vz: i64) -> (i64, i64) {
    (vx.div_euclid(32), vz.div_euclid(32))
}

/// Thickest LOW-energy coal unit (metres) in the deep record at a world voxel, or
/// `None` if the cell carries no low-energy coal — the biofacies-routing subject
/// of section 3, folded into selection so the picked cell always has one.
fn low_energy_coal_m(pregen: &Pregen, vx: i64, vz: i64) -> Option<f64> {
    let rec = pregen.deep.record_at_voxel(vx, vz)?;
    rec.units
        .iter()
        .filter(|u| u.tag().biota == Biofacies::Coal && u.tag().energy == EnergyBand::Low)
        .map(|u| u.thickness_m())
        .fold(None, |acc, t| Some(acc.map_or(t, |a: f64| a.max(t))))
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
        // journal/0063: charcoal stopped wearing its host's identity.
        (CLASS_ORGANIC_CHARCOAL, MaterialId::CHARCOAL),
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
    let set = geology::vanilla();
    let mut g = WorldGenerator::new(&pregen);

    // ---- 1. the world still grows a thick, diggable coal seam -------------
    // History of this site: S10 measured 24.03 m at a fixed voxel with erosion
    // lithology-blind; the erodibility flip (journal/0030) took the same cell to
    // 17.04 m; the zonal-circulation climate change (journal/0037) shifted precip
    // for every new world and moved the thickest seam to a *different* cell. What
    // this test is *about* is that a coal seam is biofacies-tagged, survives
    // collapse as the COAL class, and ends up as diggable `Block::Material(COAL)`
    // — a claim
    // about the pipeline, not a fixed location. So we take the thickest coal seam
    // that also collapses to diggable coal (some thick record seams are buried
    // below the collapse column; the S10 seam was near-surface).
    let candidates = coal_seam_candidates(&pregen, 3.0);
    assert!(
        !candidates.is_empty(),
        "the world grew no coal seam thicker than 3 m at all"
    );
    // Select the candidate whose *collapsed* column carries the MOST diggable coal.
    // A thick record seam can bury below the collapse column, so "thickest record
    // seam" and "most diggable coal" need not co-occur — and the full-agents flip
    // (journal/0047) moved deposition enough that they no longer do (the old
    // "first record seam ≥15 record-m that also renders ≥15 collapse-voxels" pick
    // found no cell). This is the 0044/0030 situation exactly: a frozen single-
    // point pick landing on a degenerate spot while the mechanism is intact, fixed
    // by selecting the strongest seam that *surfaces as diggable coal* rather than
    // by loosening a floor. `MIN_DIGGABLE_COAL_VOX` is re-baselined to the new
    // world's strongest exemplar.
    let coal_collapse_vox = |g: &mut WorldGenerator, vx: i64, vz: i64| -> u32 {
        let (cx, cz) = column_of(vx, vz);
        // P11 slice 3: the chunk holds one record per touched deep cell; the
        // seam belongs to the cell nearest this voxel, so measure THAT SubCell.
        let target = nearest_cell_index(&pregen.deep, vx, vz);
        let col = g.column_record(cx, cz);
        let Some(sub) = col
            .records
            .iter()
            .find(|s| s.cell_index().is_some() && s.cell_index() == target)
        else {
            return 0; // the dither realized no column of the seam's cell here
        };
        (sub.strata()
            .events
            .iter()
            .filter(|e| set.member(e.member).class == CLASS_ORGANIC_COAL)
            .map(|e| f64::from(e.thickness_m))
            .sum::<f64>()
            / VOXEL_M)
            .round() as u32
    };
    let mut ranked: Vec<(i64, i64, f64, u32)> = candidates
        .iter()
        .copied()
        .map(|(vx, vz, m)| (vx, vz, m, coal_collapse_vox(&mut g, vx, vz)))
        .collect();
    ranked.sort_by_key(|r| std::cmp::Reverse(r.3));
    println!(
        "[coal] {} record candidates >3 m; top diggable seams (record_m, collapse_vox): {:?}",
        ranked.len(),
        ranked
            .iter()
            .take(6)
            .map(|&(_, _, m, v)| (format!("{m:.1}"), v))
            .collect::<Vec<_>>()
    );
    // Prefer the strongest-diggable cell that also carries a LOW-energy coal unit
    // in its record, so section 3's biofacies-vs-energy claim lands on the same
    // cell (0044: "the richest column that also surfaces the cliff").
    let (coal_x, coal_z, thickest_coal, coal_vox_pick) = ranked
        .iter()
        .copied()
        .find(|&(vx, vz, _, v)| {
            v >= MIN_DIGGABLE_COAL_VOX && low_energy_coal_m(&pregen, vx, vz).is_some()
        })
        .expect("no diggable low-energy coal seam survived collapse");
    println!(
        "[coal] strongest diggable low-energy seam: {coal_vox_pick} collapse-vox (record {thickest_coal:.2} m) at voxel ({coal_x}, {coal_z})"
    );

    let rec = pregen
        .deep
        .record_at_voxel(coal_x, coal_z)
        .expect("the coal site is inside the pregen grid");
    let (cx, cz) = column_of(coal_x, coal_z);
    let col = g.column_record(cx, cz);
    // P11 slice 3: name the SubCell of the seam's own cell (the selection above
    // guaranteed the chunk realizes it — `coal_collapse_vox` returned nonzero).
    let target = nearest_cell_index(&pregen.deep, coal_x, coal_z);
    let sub = col
        .records
        .iter()
        .find(|s| s.cell_index().is_some() && s.cell_index() == target)
        .expect("the picked chunk realizes the seam's own deep cell");

    // ---- 2. the seam survives collapse as the COAL class -------------------
    let coal_m: f64 = sub
        .strata()
        .events
        .iter()
        .filter(|e| set.member(e.member).class == CLASS_ORGANIC_COAL)
        .map(|e| f64::from(e.thickness_m))
        .sum::<f64>();
    let coal_vox = (coal_m / VOXEL_M).round() as u32;
    assert_eq!(
        coal_vox, coal_vox_pick,
        "section 2 must re-measure the very seam section 1 picked"
    );
    assert!(
        coal_vox >= MIN_DIGGABLE_COAL_VOX,
        "expected a diggable coal seam in the collapsed column, got {coal_vox}"
    );
    assert!(
        sub.strata()
            .events
            .iter()
            .any(|e| set.member(e.member).material == MaterialId::COAL),
        "the collapsed column must contain the coal MEMBER, not just the class"
    );

    // ---- 3. routing is by biofacies, NOT by flow energy --------------------
    // The seam's own tag is LOW energy (a coal swamp is a low-energy setting). The
    // pre-0026 rule sent every subaerial low-energy unit to clastic-fine, i.e.
    // mudstone. In the same column, medium/high-energy mineral units still take the
    // clastic road, so this is a biofacies win, not a blanket override. Selection
    // guaranteed the picked cell carries such a unit (`low_energy_coal_m`).
    let seam = rec
        .units
        .iter()
        .filter(|u| u.tag().biota == Biofacies::Coal && u.tag().energy == EnergyBand::Low)
        .max_by(|a, b| a.thickness_m().total_cmp(&b.thickness_m()))
        .expect("the low-energy coal seam (selection guaranteed one)");
    assert_eq!(
        seam.tag().energy,
        EnergyBand::Low,
        "the seam is a LOW-energy unit — the old rule would have made it mudstone"
    );
    let classes: Vec<&str> = sub
        .strata()
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
        if chunk.blocks().contains(&Block::Material(MaterialId::COAL)) {
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
        "no Block::Material(COAL) in the six chunks below the surface at the measured seam"
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
        // P11 slice 3: per-column records; the centre column stands in, and the
        // two sets share the seed and therefore the same membership dither.
        let col = g.column_record(cx, cz);
        let Some(s) = col.centre_record() else {
            return 0;
        };
        (s.strata()
            .events
            .iter()
            .filter(|e| set.member(e.member).class == CLASS_ORGANIC_COAL)
            .map(|e| f64::from(e.thickness_m))
            .sum::<f64>()
            / VOXEL_M)
            .round() as u32
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

/// **Charcoal reaches the ground, as an inclusion and never as a stratum**
/// (journal/0063).
///
/// The claim this replaces was in `deep_class`'s own doc comment: *a charcoal
/// band cannot exist in a voxel column, so a charcoal member would be dead
/// content.* That was true of the pre-0055 quantizer, which rounded every
/// recorded unit to whole voxels on its own; it is false of the addressed
/// stochastic allocation, under which a 2.9 cm bed claims ~0.26 of an eighth and
/// wins a whole one about a quarter of the times it is asked.
///
/// Two halves, and the second is the one that keeps the fix honest:
///
/// 1. charcoal **is present** in the world's contents at all — the falsifier for
///    "dead content";
/// 2. and it is **never more than a minority of a voxel** — the falsifier for
///    the opposite failure, a fire lamina inflating into a bed. A charcoal bed
///    is structurally capped at `SOIL_MAX × FIRE_CHAR_FRAC = 0.04 m`, i.e. 0.356
///    of an eighth, so a charcoal-dominant voxel would mean the expression path
///    invented mass.
#[test]
fn charcoal_reaches_the_voxel_as_an_inclusion_never_as_a_stratum() {
    use dc_core::materials::geology::GeoMemberIdx;
    use dc_worldgen::fill::{Plan, allocate, fill_draw};

    let pregen = medium();
    let set = geology::vanilla();
    let mut g = WorldGenerator::new(&pregen);
    let charcoal_members: Vec<GeoMemberIdx> = (0..set.members().len())
        .map(|i| GeoMemberIdx(i as u16))
        .filter(|m| set.member(*m).material == MaterialId::CHARCOAL)
        .collect();
    assert!(
        !charcoal_members.is_empty(),
        "vanilla must register a charcoal member"
    );

    // Slice recorded columns the way the generator does, over a stride wide
    // enough to cross several fire-bearing deep cells.
    let mut spans = 0usize;
    let mut charcoal_spans = 0usize;
    let mut worst_eighths = 0u8;
    for cz in (-2400..2400).step_by(149) {
        for cx in (-2400..2400).step_by(149) {
            let col = g.column_record(cx as i64, cz as i64);
            // P11 slice 3: the chunk-centre column's SubCell stands in for "the
            // chunk's record" in this span census (I-5 judgment site).
            let Some(sub) = col.centre_record() else {
                continue;
            };
            if sub.strata().events.is_empty() {
                continue;
            }
            let cf = sub.fill();
            let (vx, vz) = (cx as i64 * 32, cz as i64 * 32);
            let surf = i64::from(col.heights[0]);
            for d in 1..=cf.depth_count() as u32 {
                spans += 1;
                let parts: Vec<(usize, u8)> = match cf.plan(d) {
                    Some(Plan::Single(i)) => vec![(*i, 8u8)],
                    Some(Plan::Mixed(w)) => {
                        allocate(w, fill_draw(SEED, vx, surf - i64::from(d), vz))
                    }
                    None => continue,
                };
                let n: u8 = parts
                    .iter()
                    .filter(|(i, _)| {
                        set.member(sub.strata().events[*i].member).material == MaterialId::CHARCOAL
                    })
                    .map(|(_, k)| *k)
                    .sum();
                if n > 0 {
                    charcoal_spans += 1;
                    worst_eighths = worst_eighths.max(n);
                }
            }
        }
    }
    println!(
        "[charcoal] {charcoal_spans} of {spans} recorded voxel spans carry charcoal \
         ({:.4} %); most eighths in any one voxel: {worst_eighths}",
        charcoal_spans as f64 * 100.0 / spans.max(1) as f64
    );
    assert!(
        spans > 10_000,
        "the sample must be big enough to mean something"
    );
    assert!(
        charcoal_spans > 0,
        "no voxel in the world carries charcoal — the expression path is dead \
         content again"
    );
    assert!(
        worst_eighths <= 4,
        "a charcoal lamina claimed {worst_eighths}/8 of a voxel; a fire bed is \
         capped at 0.04 m (0.356 of an eighth), so the expression path invented mass"
    );
}
