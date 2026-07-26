//! **Why is the erosion budget inert?** (journal/0076 follow-up.)
//!
//! ## ⚠ ITS QUESTION WAS ANSWERED ELSEWHERE, AND THE ANSWER WAS NEITHER OPTION
//! (journal/0111, journal/0114)
//!
//! This probe offers a two-way fork — equilibrium or supply-limited — and the real
//! answer was a third thing it could not express: **the knob did not reach the
//! process doing the eroding.** `erosion_budget` scaled `weathering`,
//! `k_transport` and `k_bedrock` and not `diffusion`, and hillslope creep carries
//! 96 % of this world's denudation (stubs #24, corrections #56). At 100× the budget
//! moved catchment-averaged denudation by 1.4×. So "the budget barely moves the
//! surface" was a fact about the *knob*, not about the landscape — and reading it as
//! a fact about the landscape is what let a world denuding 9× slower than anything
//! ever measured on Earth pass for graded.
//!
//! **Both of this file's numbers are therefore stale**, and deliberately not
//! re-derived here: the knob now covers `diffusion` too (journal/0114) and the
//! shipped rates moved under it. Re-run it before quoting it. The instrument that
//! can actually answer "is this landscape running at the right rate" is
//! `examples/denudation_probe.rs`, which measures against published bands rather
//! than against the world's own counterfactual — CLAUDE.md § *a closed system
//! cannot detect its own scale error*.
//!
//! `amplitude_tour` established, faithfully (1× == shipped `build_field`), that on
//! the client's boot world (seed 1337, Medium) the erosion budget barely moves the
//! surface — max 3.5 m anywhere even at 30×, relief unchanged. This probe asks the
//! *mechanism* question the tour cannot: is the budget inert because the landscape
//! is **(a) already graded to base level** — erosion has done all it can, and the
//! rate only sets how fast it gets there — or **(b) supply-limited** — there is
//! almost nothing to erode, uplift dominates and the rate is irrelevant because
//! the material is not there?
//!
//! The discriminator is **total lowering against the no-erosion counterfactual**.
//! The counterfactual surface is what a cell would reach with *zero* erosion:
//! initial bedrock `R` plus every metre of uplift the run adds
//! (`uplift × iterations`). Lowering = counterfactual − final surface = how much
//! erosion actually removed.
//!
//!  - Lowering **large and budget-insensitive** ⇒ **(a) equilibrium**: the surface
//!    erodes to grade at every budget; the rate only sets the approach speed. The
//!    lever for relief is uplift / base level / iterations, NOT the erosion rate.
//!  - Lowering **small at every budget** ⇒ **(b) supply / uplift-dominated**: there
//!    is little to remove; the budget is inert because the material is absent.
//!
//! Regolith `H` is reported alongside: if weathering ×30 makes more regolith but
//! the surface does not lower, the bottleneck is transport (regolith made, not
//! removed); if `H ≈ 0`, weathering itself is starved.
//!
//! `cargo run --release -p dc-worldgen --example amplitude_mechanism`

use dc_worldgen::deeptime::{self, DeepOverrides, production_config_with};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 1337;
const EXTENT: Extent = Extent::Medium;
const MULTS: [f64; 4] = [1.0, 3.0, 10.0, 30.0];

fn main() {
    println!(
        "=== erosion-budget MECHANISM probe — seed {SEED}, {} ===",
        EXTENT.label()
    );
    println!(
        "Q: budget inert because the surface is (a) graded to base level [erodes a lot, equally] \
         or (b) supply/uplift-dominated [barely erodes at all]?\n\
         counterfactual = initial R + uplift×iterations (zero-erosion surface); lowering = counterfactual − final surf\n"
    );

    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: EXTENT,
    });

    println!(
        "  {:>4}  {:>10}  {:>10}  {:>10}  {:>9}  {:>9}  {:>10}  {:>9}",
        "mult", "mean low", "max low", "p50 low", "mean H", "max H", "mean surf", "land"
    );
    for &m in &MULTS {
        let ov = DeepOverrides {
            erosion_budget: Some(m),
            ..DeepOverrides::default()
        };
        let cfg = production_config_with(&pregen.grid, SEED, &ov);
        let initial = deeptime::build(&pregen, &cfg);
        let run = deeptime::run_with(&pregen, &cfg, true);
        let it = f64::from(cfg.iterations);
        let n = run.grid.w * run.grid.w;

        let mut low: Vec<f64> = Vec::new();
        let (mut sh, mut mh, mut ss) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..n {
            let s = run.grid.surf_at(i);
            if s <= 0.0 {
                continue;
            }
            let cf = initial.r[i] + initial.uplift[i] * it;
            low.push(cf - s);
            let h = run.grid.h[i];
            sh += h;
            mh = mh.max(h);
            ss += s;
        }
        let land = low.len().max(1) as f64;
        let mean_low = low.iter().sum::<f64>() / land;
        let max_low = low.iter().cloned().fold(f64::MIN, f64::max);
        let mut sorted = low.clone();
        sorted.sort_by(f64::total_cmp);
        let p50 = sorted[sorted.len() / 2];
        println!(
            "  {m:>3}×  {mean_low:>10.1}  {max_low:>10.1}  {p50:>10.1}  {:>9.2}  {mh:>9.2}  {:>10.1}  {:>9}",
            sh / land,
            ss / land,
            low.len()
        );
    }

    println!(
        "\nread: if 'mean low' is large and flat across budgets → (a) equilibrium — erosion already wins at 1×,\n\
         \x20     the amplitude lever is uplift / base level / iterations, not the erosion rate.\n\
         \x20     if 'mean low' is small at every budget → (b) supply/uplift-dominated — nothing to carve."
    );
}
