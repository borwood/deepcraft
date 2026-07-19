//! S6 headless perf bench: tumble N dynamic cubes into a voxel pit at the
//! decided N=2 scale (voxel = 0.9 m) and measure step cost, bubble cost, and
//! collider counts; then time the detach -> settle -> reattach cycle for a
//! ~100-voxel prop.
//!
//! Run: `cargo run --release -p dc-physics --example s6_bench`
//!
//! The pit floor replicates the S1 terrain generator's rolling-hills layer
//! (fastnoise-lite OpenSimplex2 FBm, 4 octaves, frequency 0.008, seed 1337,
//! height = 8 m + 14 m * noise — the exact constants from
//! dc-client/src/worldgen.rs), sampled in meter space like S1 does, with an
//! enclosing wall ring so tumbling cubes stay in play. No dc-client
//! dependency — the closure is ~15 lines.

use std::time::Instant;

use dc_physics::{PhysicsConfig, PhysicsWorld};
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use glam::DVec3;

/// N=2 scale: voxel = 0.9 m (docs/ARCHITECTURE.md § Voxel scale).
const VS: f64 = 0.9;
/// Arena half-width in meters; beyond it, walls rise to +40 m.
const ARENA_HALF_M: f64 = 20.0;
const STEPS: u32 = 600; // 10 simulated seconds at 60 Hz

struct Pit {
    hills: FastNoiseLite,
}

impl Pit {
    fn new() -> Self {
        // The S1 hills layer, verbatim (dc-client/src/worldgen.rs).
        let mut hills = FastNoiseLite::with_seed(1337);
        hills.set_noise_type(Some(NoiseType::OpenSimplex2));
        hills.set_fractal_type(Some(FractalType::FBm));
        hills.set_fractal_octaves(Some(4));
        hills.set_frequency(Some(0.008));
        Self { hills }
    }

    fn surface_m(&self, xm: f64, zm: f64) -> f64 {
        8.0 + f64::from(self.hills.get_noise_2d(xm as f32, zm as f32)) * 14.0
    }

    fn is_solid(&self, vx: i64, vy: i64, vz: i64) -> bool {
        let xm = (vx as f64 + 0.5) * VS;
        let ym = (vy as f64 + 0.5) * VS;
        let zm = (vz as f64 + 0.5) * VS;
        if ym < self.surface_m(xm, zm) {
            return true;
        }
        // Wall ring so the tumble stays bounded.
        (xm.abs() > ARENA_HALF_M || zm.abs() > ARENA_HALF_M) && ym < 40.0
    }
}

struct Lcg(u64);

impl Lcg {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

struct Row {
    n: usize,
    mean_ms: f64,
    max_ms: f64,
    mean_bubble_ms: f64,
    max_tiles: usize,
    max_cuboids: usize,
    asleep: usize,
}

fn tumble(n: usize, pit: &Pit) -> Row {
    let mut world = PhysicsWorld::new(PhysicsConfig::default());
    let solid = |x: i64, y: i64, z: i64| pit.is_solid(x, y, z);

    let mut rng = Lcg(0x9E37_79B9_7F4A_7C15);
    let mut handles = Vec::with_capacity(n);
    for _ in 0..n {
        let pos = DVec3::new(
            (rng.next_f64() - 0.5) * 30.0,
            30.0 + rng.next_f64() * 12.0,
            (rng.next_f64() - 0.5) * 30.0,
        );
        let vel = DVec3::new(
            (rng.next_f64() - 0.5) * 8.0,
            0.0,
            (rng.next_f64() - 0.5) * 8.0,
        );
        handles.push(world.spawn_item(pos, DVec3::splat(0.2), vel));
    }

    let mut total_ms = 0.0f64;
    let mut max_ms = 0.0f64;
    let mut total_bubble_ms = 0.0f64;
    let mut max_tiles = 0usize;
    let mut max_cuboids = 0usize;
    for _ in 0..STEPS {
        let t0 = Instant::now();
        world.step_fixed(VS, &solid);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        total_ms += ms;
        max_ms = max_ms.max(ms);
        let s = world.stats();
        total_bubble_ms += s.last_bubble_ms;
        max_tiles = max_tiles.max(s.active_tiles);
        max_cuboids = max_cuboids.max(s.active_cuboids);
    }
    let asleep = handles.iter().filter(|h| world.is_asleep(**h)).count();
    Row {
        n,
        mean_ms: total_ms / f64::from(STEPS),
        max_ms,
        mean_bubble_ms: total_bubble_ms / f64::from(STEPS),
        max_tiles,
        max_cuboids,
        asleep,
    }
}

fn prop_cycle(pit: &Pit) {
    let solid = |x: i64, y: i64, z: i64| pit.is_solid(x, y, z);
    let mut world = PhysicsWorld::new(PhysicsConfig::default());

    // ~100-voxel prop (6x4x4 slab + 4-voxel tail) dropped from ~10 m above
    // the local surface.
    let surface_v = (pit.surface_m(4.5, 4.5) / VS).ceil() as i64;
    let base = (5i64, surface_v + 11, 5i64);
    let mut voxels = Vec::new();
    for x in 0..6 {
        for y in 0..4 {
            for z in 0..4 {
                voxels.push((base.0 + x, base.1 + y, base.2 + z));
            }
        }
    }
    for x in 0..4 {
        voxels.push((base.0 + x, base.1 + 4, base.2));
    }
    assert_eq!(voxels.len(), 100);

    let t0 = Instant::now();
    let prop = world.detach_prop(&voxels, VS).expect("prop detaches");
    let detach_ms = t0.elapsed().as_secs_f64() * 1e3;

    let t1 = Instant::now();
    let mut settle_steps = 0u32;
    while !world.is_asleep(prop) {
        world.step_fixed(VS, &solid);
        settle_steps += 1;
        assert!(settle_steps < 3600, "prop failed to settle in 60 s");
    }
    let settle_ms = t1.elapsed().as_secs_f64() * 1e3;

    let t2 = Instant::now();
    let reattached = world
        .reattach_prop(prop, VS)
        .expect("asleep prop reattaches");
    let reattach_ms = t2.elapsed().as_secs_f64() * 1e3;

    assert_eq!(reattached.len(), voxels.len(), "voxel count changed");
    println!("\nDetach -> settle -> reattach, 100-voxel prop (N=2 scale):\n");
    println!("| phase | result |");
    println!("|---|---|");
    println!("| detach (merge + compound build) | {detach_ms:.3} ms |");
    println!(
        "| settle to sleep | {settle_steps} steps ({:.2} sim s), {settle_ms:.1} ms wall |",
        f64::from(settle_steps) / 60.0
    );
    println!("| reattach (snap + write-back set) | {reattach_ms:.3} ms |");
    println!(
        "| voxel count preserved | {} -> {} |",
        voxels.len(),
        reattached.len()
    );
}

fn main() {
    let pit = Pit::new();
    println!(
        "S6 bench: tumbling cubes into the S1-noise pit, {STEPS} steps @ 60 Hz, \
         voxel = {VS} m (N=2), release build\n"
    );
    println!(
        "| N cubes | mean step | max step | mean bubble refresh | peak tiles | peak cuboids | asleep at end |"
    );
    println!("|---|---|---|---|---|---|---|");
    for n in [10usize, 100, 500] {
        let r = tumble(n, &pit);
        println!(
            "| {} | {:.3} ms | {:.3} ms | {:.3} ms | {} | {} | {}/{} |",
            r.n, r.mean_ms, r.max_ms, r.mean_bubble_ms, r.max_tiles, r.max_cuboids, r.asleep, r.n
        );
    }
    prop_cycle(&pit);
}
