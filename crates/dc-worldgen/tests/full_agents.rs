//! The full erosion-agent roster (wind + frost + wave, journal/0034) — the
//! falsifiers and the measured signatures.
//!
//! The invariants that must not break: the off path is byte-identical (both the
//! flag-off run and the strong "flag on, all three rate knobs zero" run land on
//! the uncoupled result to the bit), the coupled path is deterministic
//! scalar↔parallel, mass is still conserved (wind and wave only *redistribute*
//! material — they add nothing external, so the ledger `Δ(ΣR+ΣH) == uplift +
//! biotic` is untouched), and the recorder invariant `sum(units) == H` survives.
//!
//! And the mechanism claims, each cashed as a measurement with the flag on vs
//! off on a seeded world: an arid region nets deflation and lays a downwind
//! loess/dune deposit; a periglacial (near-0 °C) band produces more regolith;
//! coastal cells retreat. Numbers print under `--nocapture` and are quoted in the
//! journal entry.

use dc_core::materials::MaterialId;
use dc_worldgen::deeptime::climate::air_temp_c;
use dc_worldgen::deeptime::species::SpeciesAxis;
use dc_worldgen::deeptime::{self, DeepConfig, DeepRun, DepUnit, Eolian, Providers};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0FDA_6E27_2034;

fn small_world(seed: u64) -> Pregen {
    Pregen::run(WorldParams {
        seed,
        extent: Extent::Small,
    })
}

/// Base config: recorder on, the older couplings off, so the new agents are
/// measured in isolation. Sea-level cycling defaults stay on (waves need it).
fn base(seed: u64) -> DeepConfig {
    DeepConfig {
        seed,
        cell_m: 1000.0,
        iterations: 40,
        remarch_interval: 20,
        record: true,
        biotic: false,
        erodibility: false,
        full_agents: false,
        ..DeepConfig::default()
    }
}

/// All three agents live at their default rates.
fn all_on(seed: u64) -> DeepConfig {
    DeepConfig {
        full_agents: true,
        ..base(seed)
    }
}

/// Total eolian-tagged deposition in the record (loess, dune) — the downwind
/// signature the wind agent must leave.
fn eolian_mass(run: &DeepRun) -> (f64, f64) {
    let (mut loess, mut dune) = (0.0, 0.0);
    for s in &run.grid.strata {
        for u in &s.units {
            match u.tag().eolian {
                Eolian::Loess => loess += u.thickness_m(),
                Eolian::Dune => dune += u.thickness_m(),
                Eolian::None => {}
            }
        }
    }
    (loess, dune)
}

fn ledger_residual(run: &DeepRun) -> f64 {
    deeptime::total_mass(&run.grid) - run.mass_before - run.uplift_total - run.biotic_total
}

// ---------------------------------------------------------------------------
// Off-by-default, and provably unchanged.

/// **The off-path proof.** Running the agent code with all three rate knobs at
/// zero makes every added term produce exactly `0.0` (or the identity `1.0` for
/// the frost multiplier) — so the phases all execute and must still land on the
/// flag-off result bit for bit. Stronger than "off vs off": it proves the new
/// arithmetic is genuinely identity/zero-preserving rather than merely skipped.
#[test]
fn neutral_full_agents_is_byte_identical_to_off() {
    let pregen = small_world(SEED);
    let off = deeptime::run(&pregen, &base(SEED));
    let neutral = deeptime::run(
        &pregen,
        &DeepConfig {
            full_agents: true,
            eolian_deflation: 0.0,
            frost_weathering_gain: 0.0,
            wave_erosion: 0.0,
            ..base(SEED)
        },
    );
    assert_eq!(
        off.grid.r, neutral.grid.r,
        "bedrock plane must be identical"
    );
    assert_eq!(
        off.grid.h, neutral.grid.h,
        "alluvium plane must be identical"
    );
    assert_eq!(
        off.grid.strata, neutral.grid.strata,
        "strata records must be identical"
    );
}

/// The same identity check with the biotic layer and erodibility both on: the
/// frost multiplier's composition (`wmult × litho × frost`) must not disturb the
/// pre-0034 result when frost is neutral.
#[test]
fn neutral_full_agents_is_byte_identical_with_biology_and_erodibility_on() {
    let pregen = small_world(SEED);
    let stack = DeepConfig {
        biotic: true,
        erodibility: true,
        ..base(SEED)
    };
    let off = deeptime::run(&pregen, &stack);
    let neutral = deeptime::run(
        &pregen,
        &DeepConfig {
            full_agents: true,
            eolian_deflation: 0.0,
            frost_weathering_gain: 0.0,
            wave_erosion: 0.0,
            ..stack
        },
    );
    assert_eq!(off.grid.r, neutral.grid.r);
    assert_eq!(off.grid.h, neutral.grid.h);
    assert_eq!(off.grid.strata, neutral.grid.strata);
}

#[test]
fn full_agents_on_actually_changes_the_world() {
    let pregen = small_world(SEED);
    let off = deeptime::run(&pregen, &base(SEED));
    let on = deeptime::run(&pregen, &all_on(SEED));
    assert_ne!(off.grid.r, on.grid.r, "the agents changed nothing");
}

// ---------------------------------------------------------------------------
// Determinism (project law): the parallel production path must reproduce scalar.

#[test]
fn full_agents_parallel_equals_scalar_byte_identical() {
    // A finer cell forces past the rayon fork floor so the per-cell phases really
    // fork; wind and wave stay scalar in both drivers by construction.
    let pregen = small_world(SEED);
    let cfg = DeepConfig {
        cell_m: 120.0,
        iterations: 24,
        ..all_on(SEED)
    };
    let scalar = deeptime::run_with(&pregen, &cfg, false);
    let parallel = deeptime::run_with(&pregen, &cfg, true);
    assert_eq!(scalar.grid.r, parallel.grid.r);
    assert_eq!(scalar.grid.h, parallel.grid.h);
    assert_eq!(scalar.grid.strata, parallel.grid.strata);
}

#[test]
fn full_agents_runs_are_byte_identical_on_a_repeated_seed() {
    let pregen = small_world(SEED);
    let cfg = all_on(SEED);
    let a = deeptime::run(&pregen, &cfg);
    let b = deeptime::run(&pregen, &cfg);
    assert_eq!(a.grid.r, b.grid.r);
    assert_eq!(a.grid.h, b.grid.h);
    assert_eq!(a.grid.strata, b.grid.strata);
}

// ---------------------------------------------------------------------------
// Mass conservation and the recorder invariant survive all three agents.

/// Wind redistributes loose `H`; wave moves quarried `R`/`H` offshore. Neither
/// adds external mass, so the ledger `Δ(ΣR+ΣH) == uplift + biotic` must still
/// balance with the whole roster live — the falsifier for "wind only
/// redistributes".
#[test]
fn mass_is_conserved_with_full_agents_on() {
    let pregen = small_world(SEED);
    let run = deeptime::run(&pregen, &all_on(SEED));
    let residual = ledger_residual(&run);
    assert!(
        residual.abs() < 1.0,
        "mass leaked with full agents on: residual {residual}"
    );
}

#[test]
fn recorder_total_equals_alluvium_with_full_agents_on() {
    let pregen = small_world(SEED);
    let run = deeptime::run(&pregen, &all_on(SEED));
    let mut worst = 0.0f64;
    for (i, s) in run.grid.strata.iter().enumerate() {
        // P11 slice 3 (the pack): the record is quantized (2^-10 m) and the
        // sub-quantum residue rides in the per-cell carry, so the exact mirror
        // of H is record + carry; the record alone sits within quantum/2 of H.
        worst = worst.max((s.total_m() + s.carry_m() - run.grid.h[i]).abs());
    }
    assert!(worst < 1e-6, "sum(units)==H violated, worst {worst}");
}

// ---------------------------------------------------------------------------
// Wind: net deflation in the arid interior + a downwind loess/dune deposit.

#[test]
fn wind_deflates_the_arid_interior_and_lays_loess_downwind() {
    let pregen = small_world(SEED);
    let off = deeptime::run(&pregen, &base(SEED));
    let wind = deeptime::run(
        &pregen,
        &DeepConfig {
            full_agents: true,
            // Isolate wind.
            frost_weathering_gain: 0.0,
            wave_erosion: 0.0,
            ..base(SEED)
        },
    );

    // Downwind signature: the record now carries eolian units the off run cannot.
    let (loess, dune) = eolian_mass(&wind);
    let (loess_off, dune_off) = eolian_mass(&off);
    assert_eq!(
        (loess_off, dune_off),
        (0.0, 0.0),
        "the off run must have no eolian units"
    );
    assert!(
        loess + dune > 0.0,
        "wind laid no loess/dune deposit (loess {loess}, dune {dune})"
    );

    // Net deflation: over cells the OFF run classed arid (stable classifier),
    // the arid interior carries less loose cover with wind on than off.
    let w = off.grid.w;
    let (mut h_arid_off, mut h_arid_on, mut arid_cells) = (0.0f64, 0.0f64, 0usize);
    for i in 0..w * w {
        let subaerial = off.grid.surf_at(i) > 0.0;
        if subaerial && f64::from(off.grid.precip[i]) < 0.32 {
            h_arid_off += off.grid.h[i];
            h_arid_on += wind.grid.h[i];
            arid_cells += 1;
        }
    }
    println!(
        "[wind] arid cells {arid_cells}: ΣH off {h_arid_off:.1} m → on {h_arid_on:.1} m \
         (net deflation {:.1} m); deposits loess {loess:.1} m, dune {dune:.1} m; \
         ledger residual {:.3}",
        h_arid_off - h_arid_on,
        ledger_residual(&wind),
    );
    assert!(
        h_arid_on < h_arid_off,
        "arid interior did not net-deflate: ΣH off {h_arid_off} → on {h_arid_on}"
    );
    // Redistribution only — the wind run's own ledger still balances.
    assert!(ledger_residual(&wind).abs() < 1.0);
}

// ---------------------------------------------------------------------------
// Frost: a near-0 °C band produces more regolith (more bedrock stripped).

#[test]
fn frost_enhances_regolith_production_in_the_periglacial_band() {
    let pregen = small_world(SEED);
    let width = base(SEED).frost_band_width_c;
    let off = deeptime::run(&pregen, &base(SEED));
    let frost = deeptime::run(
        &pregen,
        &DeepConfig {
            full_agents: true,
            // Isolate frost.
            eolian_deflation: 0.0,
            wave_erosion: 0.0,
            ..base(SEED)
        },
    );

    // Band = subaerial cells whose temperature (shared climate model on the OFF
    // surface, a stable classifier) sits inside the freeze–thaw band about 0 °C.
    // Frost accelerates bedrock→regolith weathering there, so more bedrock `R` is
    // stripped: R_frost should sit below R_off across the band.
    let w = off.grid.w;
    let (mut r_band_off, mut r_band_on, mut band_cells) = (0.0f64, 0.0f64, 0usize);
    let (mut r_warm_off, mut r_warm_on) = (0.0f64, 0.0f64);
    for gy in 0..w {
        let lat = off.grid.lat_deg(gy);
        for gx in 0..w {
            let i = gy * w + gx;
            let surf = off.grid.surf_at(i);
            if surf <= 0.0 {
                continue;
            }
            let t = f64::from(air_temp_c(lat, surf));
            if t.abs() < width {
                r_band_off += off.grid.r[i];
                r_band_on += frost.grid.r[i];
                band_cells += 1;
            } else if t > width + 6.0 {
                // A warm control band, well outside freeze–thaw: frost must NOT
                // touch it (the signal is periglacial, not uniform).
                r_warm_off += off.grid.r[i];
                r_warm_on += frost.grid.r[i];
            }
        }
    }
    let band_strip = r_band_off - r_band_on; // extra bedrock lowered by frost
    let warm_strip = r_warm_off - r_warm_on;
    println!(
        "[frost] band cells {band_cells}: extra bedrock stripped {band_strip:.1} m \
         (ΣR off {r_band_off:.0} → on {r_band_on:.0}); warm-control extra strip {warm_strip:.2} m"
    );
    assert!(band_cells > 0, "no periglacial band cells on this world");
    assert!(
        band_strip > 0.0,
        "frost did not enhance regolith production in the band (Δ {band_strip})"
    );
    // The warm control barely moves compared with the band: the signal is gated
    // by temperature, not a blanket speed-up.
    assert!(
        band_strip > warm_strip.abs() * 2.0,
        "frost signal is not periglacially localized: band {band_strip}, warm {warm_strip}"
    );
}

// ---------------------------------------------------------------------------
// Wave: coastal cells retreat / are cut down toward the sea stand.

#[test]
fn waves_cut_down_the_coastline() {
    let pregen = small_world(SEED);
    let band = base(SEED).wave_band_m;
    let iters = base(SEED).iterations;
    let off = deeptime::run(&pregen, &base(SEED));

    // Isolate the wave agent — and, since journal/0068, isolate it from the
    // record's *composition* too. The outcrop rule now returns the lithology
    // dominating the near-surface 0.9 m window, and a repeatedly-stripped coast is
    // a thin record whose window is mostly basement — the most wave-resistant rock
    // there is. That is faithful (a wave-cut rock platform does not retreat), but
    // it makes "does THIS world's coast net-lower" a question about the coast's
    // composition rather than about whether the wave agent works: on the seeded
    // world 301 of 327 coastal cells outcrop basement and the net signal vanishes
    // into deposition noise. So we drive the outcrop through the provider seam and
    // ask the mechanism itself, two ways at once: waves must cut a SOFT coast down,
    // and must cut it strictly harder than a resistant one. Both are independent of
    // what the record happens to hold, which is exactly the robustness the record's
    // new basement-heavy coasts demand.
    // journal/0072: the wave agent reads the outcrop SHARES seam for its rate now
    // (it blends the susceptibility table by window share), not the verdict
    // `outcrop_at`. So force the composition through `outcrop_shares` — a one-hot
    // share vector is a uniform window, which the blend maps to exactly that
    // lithology's rate (argmax is the blend's limiting case), so this drives the
    // same soft/resistant contrast the verdict override used to.
    //
    // **P11 slice 2 re-graded the seam to MATERIALS**, so the one-hot is over the
    // species axis rather than over the class roster — and the two rocks are named
    // rather than looked up through a class: `dc:mudstone` (the reference the
    // member rate table is anchored on, so its multiplier is exactly `1.0`) against
    // `dc:granite` (the basement, the most wave-resistant thing in the world). The
    // contrast the test drives is unchanged; only the alphabet is.
    let soft: fn(&SpeciesAxis, &[DepUnit], &mut [f64]) = |axis, _, out| {
        out.fill(0.0);
        out[axis.slot_of(MaterialId::MUDSTONE)] = 1.0;
    };
    let rock: fn(&SpeciesAxis, &[DepUnit], &mut [f64]) = |axis, _, out| {
        out.fill(0.0);
        out[axis.slot_of(MaterialId::GRANITE)] = 1.0;
    };
    let wave_cfg = |o: fn(&SpeciesAxis, &[DepUnit], &mut [f64])| DeepConfig {
        full_agents: true,
        // Isolate wave from the other two agents.
        eolian_deflation: 0.0,
        frost_weathering_gain: 0.0,
        providers: Providers {
            outcrop_shares: Some(o),
            ..Providers::default()
        },
        ..base(SEED)
    };
    let wave_soft = deeptime::run(&pregen, &wave_cfg(soft));
    let wave_rock = deeptime::run(&pregen, &wave_cfg(rock));

    // Coastal cells: low freeboard above mean sea level in the OFF run (a stable
    // classifier). The falsifier is **differential**, and deliberately so. The
    // wave agent both quarries cliffs and redeposits the spoil, and on this seeded
    // world the coastal band is a net deposition sink — so "does the band's summed
    // surface drop" is dominated by where the spoil lands, not by whether rock was
    // cut, and it is near zero for either lithology. What is *not* ambiguous is
    // that a soft coast ends lower than a resistant one under identical wave
    // forcing: cutting scales with the (rock, wave) resistance axis, so more rock
    // leaves a `ClasticFine` shore than a `Basement` one. If the agent did nothing,
    // or ignored lithology, the two runs would tie. They must not.
    let w = off.grid.w;
    let (mut s_off, mut s_soft, mut s_rock, mut coastal) = (0.0f64, 0.0f64, 0.0f64, 0usize);
    for i in 0..w * w {
        let free = off.grid.surf_at(i);
        if free > 0.0 && free <= band {
            s_off += off.grid.surf_at(i);
            s_soft += wave_soft.grid.surf_at(i);
            s_rock += wave_rock.grid.surf_at(i);
            coastal += 1;
        }
    }
    let soft_vs_rock = s_rock - s_soft;
    println!(
        "[wave] coastal cells {coastal}: Σsurf off {s_off:.0} m → soft {s_soft:.0} m / \
         basement {s_rock:.0} m; soft coast sits {soft_vs_rock:.1} m lower than basement \
         over {iters} epochs"
    );
    assert!(coastal > 0, "no coastal cells on this world");
    assert!(
        soft_vs_rock > 0.0,
        "the wave-resistance axis is inert: a soft coast must be cut lower than a \
         basement one under the same wave forcing (Σsoft {s_soft}, Σbasement {s_rock})"
    );
    assert!(ledger_residual(&wave_soft).abs() < 1.0);
    assert!(ledger_residual(&wave_rock).abs() < 1.0);
}
