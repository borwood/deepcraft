//! **Ritual cost** — the wall-clock measurement the provider slices report,
//! kept out of the slot suites because it measures the machine, not the code.

mod providers_common;
use providers_common::SEED;

/// Ritual time, on demand: the production deep-time run at `Extent::Medium` —
/// the "generating world history…" ritual the user waits through. `#[ignore]`d
/// because it costs ~17 s and measures the machine, not the code; run it with
/// `cargo test -p dc-worldgen --release --test providers_ritual -- --ignored --nocapture`
/// to reproduce the before/after numbers in journal/0060 and 0061.
#[test]
#[ignore = "17 s wall-clock measurement, not a correctness assertion"]
fn ritual_time_medium() {
    use dc_worldgen::deeptime::build_field;
    use dc_worldgen::pregen::{Extent, Pregen, WorldParams};
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let t = std::time::Instant::now();
    let f = build_field(&pregen.grid, SEED);
    let dt = t.elapsed();
    println!(
        "RITUAL medium w={} cell_m={} secs={:.3}",
        f.w,
        f.cell_m,
        dt.as_secs_f64()
    );
}
