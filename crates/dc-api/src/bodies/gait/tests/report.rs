//! **The predicted table, measured** — the design pass's § 3.5 Tables A, B and
//! C, hand-derived from source literals with no cargo command run, checked here
//! against the arithmetic they predict.
//!
//! The rule this file exists to honour: *if a measured value disagrees with the
//! prediction, do NOT adjust the prediction to match.* The predictions below
//! are transcribed from the design document verbatim and never edited; a
//! divergence is recorded as a finding about that document (immutable body,
//! mutable header — the banner obligation falls on whoever measures).
//!
//! The design table's cells are hand-derived to four decimal places, so they
//! are confirmed at **1e-3 relative** — the honest precision of the source.
//! The *structural* claims (dynamic similarity, the cadence ratio) are asserted
//! at **1e-12** in `tests.rs`, because those are properties of the derivation
//! rather than of a printed number.
//!
//! Run with `--nocapture` for the full report.

use super::super::super::{
    AnimClip, BodyPlan, biped_plan, longleg_plan, retired_biped_walk_clip, stout_plan,
};
use super::super::{GaitBakeOutcome, GaitKnobs, GaitVector, RootHeight, bake_gait, duty_exponent};
use super::{G_EARTH, G_WORLD, by};

fn baked(plan: &BodyPlan, g: f64) -> GaitVector {
    match bake_gait(plan, "stand", g, &GaitKnobs::default()) {
        GaitBakeOutcome::Baked(v) => v,
        other => panic!("expected Baked for `{}`, got {other:?}", plan.name),
    }
}

/// Confirm one predicted cell, print the comparison, return the relative error.
fn confirm(row: &str, cell: &str, measured: f64, predicted: f64, rel: f64) -> f64 {
    let err = if predicted == 0.0 {
        measured.abs()
    } else {
        ((measured - predicted) / predicted).abs()
    };
    println!(
        "  {row:<16} {cell:<10} predicted {predicted:>12.6}  measured {measured:>12.6}  rel {err:.2e}"
    );
    assert!(
        err <= rel,
        "DIVERGENCE — {row} {cell}: design predicts {predicted}, the arithmetic gives \
         {measured} (rel {err:.3e} > {rel:.0e}). Do NOT edit the prediction; record the finding."
    );
    err
}

/// **Table A — the derived comfortable walk at Fr = 0.25, at the design's
/// `g = 9.81`.** Predictions transcribed from the design pass § 3.5 Table A.
#[test]
fn table_a_the_derived_walk_at_earth_gravity() {
    // plan, L, v, λ, s, T, f, β, θmax, Δ, Δ/L
    #[allow(clippy::type_complexity)]
    let predicted: [(&str, BodyPlan, [f64; 9]); 3] = [
        (
            "biped",
            biped_plan(),
            [
                1.4691, 1.3353, 0.6677, 0.9090, 1.1002, 0.6000, 0.3891, 0.0658, 0.07475,
            ],
        ),
        (
            "stout",
            stout_plan(),
            [
                1.0388, 0.6677, 0.3338, 0.6427, 1.5559, 0.6000, 0.3891, 0.0329, 0.07475,
            ],
        ),
        (
            "longleg",
            longleg_plan(),
            [
                1.5816, 1.5478, 0.7739, 0.9786, 1.0219, 0.6000, 0.3891, 0.0762, 0.07475,
            ],
        ),
    ];
    println!("\nTABLE A — derived walk, Fr = 0.25, g = {G_EARTH} (the design pass's basis)");
    let mut worst = 0.0_f64;
    for (row, plan, p) in predicted {
        let v = baked(&plan, G_EARTH);
        let at = v.at(0.25);
        let RootHeight::Derived {
            amplitude_ratio,
            amplitude_m,
            ..
        } = at.root_height
        else {
            panic!("Fr 0.25 is a walk");
        };
        for (cell, measured, pred) in [
            ("v m/s", at.speed_m_s, p[0]),
            ("stride λ", at.stride_m, p[1]),
            ("step s", at.step_m, p[2]),
            ("period T", at.period_s, p[3]),
            ("cadence f", at.cadence_hz, p[4]),
            ("duty β", at.mean_duty, p[5]),
            ("θmax rad", at.theta_max_rad, p[6]),
            ("bob Δ m", amplitude_m, p[7]),
            ("Δ/L", amplitude_ratio, p[8]),
        ] {
            worst = worst.max(confirm(row, cell, measured, pred, 1e-3));
        }
    }
    println!("  worst relative deviation across Table A: {worst:.3e}");
}

/// **Table B — the same three bodies at the shipped world-global 4.5 m/s.**
/// Every cell is out of the derivation's *root-height* domain (all three are
/// runs) and the compass Δ is printed exactly as the design prints it: out of
/// domain, and that is the point.
#[test]
fn table_b_at_the_shipped_walk_speed() {
    let predicted: [(&str, BodyPlan, [f64; 7]); 3] = [
        (
            "biped",
            biped_plan(),
            [2.3457, 2.6139, 0.5809, 1.7216, 0.3330, 47.95, 0.2906],
        ),
        (
            "stout",
            stout_plan(),
            [4.6914, 1.6091, 0.3576, 2.7967, 0.2775, 66.10, 0.2617],
        ),
        (
            "longleg",
            longleg_plan(),
            [2.0237, 2.8985, 0.6441, 1.5525, 0.3462, 45.27, 0.3021],
        ),
    ];
    println!("\nTABLE B — at walk_speed_m_s = 4.5, g = {G_EARTH} (the design pass's basis)");
    let mut worst = 0.0_f64;
    for (row, plan, p) in predicted {
        let v = baked(&plan, G_EARTH);
        let fr = v.froude(4.5);
        let at = v.at(fr);
        let compass_m = v.governing_reach_m * (1.0 - at.theta_max_rad.cos());
        for (cell, measured, pred) in [
            ("Fr", fr, p[0]),
            ("stride λ", at.stride_m, p[1]),
            ("period T", at.period_s, p[2]),
            ("cadence f", at.cadence_hz, p[3]),
            ("duty β", at.mean_duty, p[4]),
            ("θmax deg", at.theta_max_rad.to_degrees(), p[5]),
            ("compass Δ*", compass_m, p[6]),
        ] {
            worst = worst.max(confirm(row, cell, measured, pred, 1e-3));
        }
        assert!(matches!(at.root_height, RootHeight::Declined { .. }));
    }
    println!("  * compass Δ is printed OUT OF DOMAIN: the bake declines it (flight phase).");
    println!("  worst relative deviation across Table B: {worst:.3e}");
}

/// **The same table at the gravity this world actually runs** — `CharacterConfig`
/// default `gravity_m_s2 = 25.0`, not Earth's 9.81. The design derived
/// everything at 9.81, which is right for the *literature* and wrong for the
/// *world*: this is the finding, printed so it cannot be lost.
#[test]
fn the_same_bodies_at_this_worlds_gravity() {
    println!("\nAT THIS WORLD'S g = {G_WORLD} (CharacterConfig::default)");
    println!("  plan     | Fr@4.5 | regime | comfy-walk v (Fr 0.25) | f | λ | bob Δ");
    for plan in [biped_plan(), stout_plan(), longleg_plan()] {
        let v = baked(&plan, G_WORLD);
        let run = v.at(v.froude(4.5));
        let walk = v.at(0.25);
        let RootHeight::Derived { amplitude_m, .. } = walk.root_height else {
            panic!()
        };
        println!(
            "  {:<8} | {:.4} | {:?} | {:.4} m/s | {:.4} Hz | {:.4} m | {:.4} m",
            plan.name,
            run.froude,
            run.regime,
            walk.speed_m_s,
            walk.cadence_hz,
            walk.stride_m,
            amplitude_m
        );
        // The § 3.5 flag survives the gravity correction: every shipped body is
        // still in the run regime at the world-global speed.
        assert_eq!(run.regime, super::super::Regime::Run);
    }
    println!(
        "  duty exponent, derived from the two published anchors: {:.10} (the design quotes 0.263)",
        duty_exponent()
    );
}

/// **Table C — the (now RETIRED) `dc:anim/biped_walk`, decomposed as the
/// cross-check.** Measured from the clip's own keys and the plan's own
/// geometry; compared against what the derivation says at the clip's *own*
/// implied speed. Earth gravity, because the clip is a hand-authored human walk
/// and the regression it is being checked against is an Earth-gravity one.
#[test]
fn table_c_the_authored_clip_decomposed() {
    let plan = biped_plan();
    // The PARKED fixture. It stopped being shipped content 2026-08-02 (user
    // call #3) and this decomposition is the only reason it is still in the
    // tree — the testimony below is dated and does not move.
    let clip: AnimClip = retired_biped_walk_clip();
    let reach = baked(&plan, G_EARTH).governing_reach_m;

    // Measured from the clip: the widest hip spread, on the leg bones.
    let hip_at = |t: f64, seg: &str| {
        clip.keyframes
            .iter()
            .find(|k| (k.t - t).abs() < 1e-12)
            .unwrap()
            .rotations
            .iter()
            .find(|r| r.segment == seg)
            .unwrap()
            .euler[0]
    };
    let fwd = hip_at(0.0, "leg_l_upper");
    let back = hip_at(0.0, "leg_r_upper");
    let step = reach * (fwd.sin() + (-back).sin());
    let speed = step / (clip.duration_s / 2.0);

    let v = baked(&plan, G_EARTH);
    let fr = v.froude(speed);
    let at = v.at(fr);
    let derived_step = at.step_m;
    let bob_from_clip_geometry = reach - reach * fwd.cos();
    let front_pins = reach * fwd.cos();
    let rear_pins = reach * (-back).cos();

    println!("\nTABLE C — dc:anim/biped_walk, decomposed (g = {G_EARTH})");
    let mut worst = 0.0_f64;
    worst = worst.max(confirm("clip", "period", clip.duration_s, 1.000, 1e-3));
    worst = worst.max(confirm("clip", "hip fwd", fwd, 0.60, 1e-3));
    worst = worst.max(confirm("clip", "hip back", back, -0.50, 1e-3));
    worst = worst.max(confirm("clip", "step m", step, 0.9188, 1e-3));
    worst = worst.max(confirm("clip", "speed m/s", speed, 1.8376, 1e-3));
    worst = worst.max(confirm("clip", "Fr", fr, 0.3911, 1e-3));
    worst = worst.max(confirm("derived", "period", at.period_s, 0.8311, 1e-3));
    worst = worst.max(confirm("derived", "θmax", at.theta_max_rad, 0.4488, 1e-3));
    worst = worst.max(confirm("derived", "step m", derived_step, 0.7636, 1e-3));
    worst = worst.max(confirm("derived", "duty β", at.mean_duty, 0.5334, 1e-3));
    let RootHeight::Derived { amplitude_m, .. } = at.root_height else {
        panic!("Fr {fr} is a walk")
    };
    worst = worst.max(confirm("derived", "bob m", amplitude_m, 0.0871, 1e-3));
    worst = worst.max(confirm(
        "clip geometry",
        "demands bob",
        bob_from_clip_geometry,
        0.1537,
        1e-3,
    ));
    worst = worst.max(confirm(
        "clip",
        "self-error",
        rear_pins - front_pins,
        0.045977,
        1e-3,
    ));
    // The authored-bob row is GONE, and its absence is the point: the clip's
    // own leg swing demanded the `bob_from_clip_geometry` above while the
    // keyframes authored 0.040 m — 26 % of it, which *was* the hover. That
    // field left the schema 2026-08-02 (user call #2), so there is no longer a
    // number here to compare, and the derived `root_offset` owns the quantity.
    // The dated measurement stays in the design pass § 3.5, unrewritten.
    println!("  worst relative deviation across Table C: {worst:.3e}");

    // The conversion's acceptance, band-softened because the original was
    // hand-typed rather than generated: the derived stride sits within 25 % of
    // the authored clip's own implied stride at the clip's own speed.
    let gap = (derived_step - step).abs() / step;
    assert!(
        gap < 0.25,
        "derived step {derived_step:.4} m against the clip's {step:.4} m — {:.1} % apart",
        gap * 100.0
    );
    println!(
        "  derived-vs-authored stride gap at the clip's own speed: {:.1} %",
        gap * 100.0
    );

    // …and the same comparison at THIS WORLD's gravity.
    //
    // ⚠ This caption used to read "far worse, because the clip encodes a human's
    // Earth-gravity gait" — TRUE when written against `G_WORLD = 25.0`, and
    // FALSE within hours: gravity became a world constant defaulting to Earth
    // (DECIDED 2026-08-02), so the two rows are now IDENTICAL by construction.
    // Kept, and kept honest, rather than deleted: it is the row that would go
    // loud again the day a world sets a different gravity, which is exactly the
    // capability the ruling bought. *A printed caption is a published claim the
    // gate cannot check* (CLAUDE.md § Gates) — this one was wrong for a few
    // hours and nothing but a human read could have caught it.
    let vw = baked(&plan, G_WORLD);
    let at_w = vw.at(vw.froude(speed));
    println!(
        "  the same comparison at g = {G_WORLD}: derived step {:.4} m, {:.1} % from the clip",
        at_w.step_m,
        100.0 * (at_w.step_m - step).abs() / step
    );
}

/// The free cross-check the design says to print and never fit to: the swing
/// leg's natural PENDULAR period against the period the Froude chain wants.
/// The gap is the missing force term, stated as a ratio.
#[test]
fn the_ballistic_swing_cross_check_is_reported_not_fitted() {
    let plan = biped_plan();
    let v = baked(&plan, G_EARTH);
    let at = v.at(0.25);
    // Uniform-density boxes about the hip, sagittal (X) axis.
    let mut m_sum = 0.0;
    let mut md = 0.0;
    let mut i_hip = 0.0;
    for name in ["leg_l_upper", "leg_l_lower"] {
        let s = by(&plan, name);
        let m = s.size_m[0] * s.size_m[1] * s.size_m[2];
        // Distance from the hip joint down to the box centre.
        let d = if name == "leg_l_upper" {
            -s.offset_m[1]
        } else {
            -(s.pivot_m[1] + s.offset_m[1])
        };
        m_sum += m;
        md += m * d;
        i_hip += m * (s.size_m[1] * s.size_m[1] + s.size_m[2] * s.size_m[2]) / 12.0 + m * d * d;
    }
    let d = md / m_sum;
    let t_nat = std::f64::consts::TAU * (i_hip / (m_sum * G_EARTH * d)).sqrt();
    let ballistic_swing = t_nat / 2.0;
    let wanted_swing = (1.0 - at.mean_duty) * at.period_s;
    println!("\nBALLISTIC SWING CROSS-CHECK (g = {G_EARTH}, reported, never fitted to)");
    println!("  m ∝ {m_sum:.6}, d = {d:.4} m, I_hip = {i_hip:.6e}");
    println!(
        "  T_nat = {t_nat:.4} s → a purely ballistic swing takes {ballistic_swing:.4} s; the \
         Froude chain wants {wanted_swing:.4} s ({:.2}× shorter)",
        ballistic_swing / wanted_swing
    );
    for (label, measured, predicted) in [
        ("m", m_sum, 0.028584),
        ("d", d, 0.4156),
        ("I_hip", i_hip, 6.848e-3),
        ("T_nat", t_nat, 1.523),
        ("swing", ballistic_swing, 0.761),
        ("wanted", wanted_swing, 0.364),
    ] {
        confirm("pendulum", label, measured, predicted, 2e-3);
    }
    // The discrepancy IS the missing force term. Nothing is tuned to close it.
    assert!(ballistic_swing / wanted_swing > 1.5);
}
