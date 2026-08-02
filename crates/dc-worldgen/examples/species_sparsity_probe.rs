//! **How many species does a deep cell's load actually hold?** — the one cheap
//! measurement that decides P11 slice 2's whole cost family.
//!
//! The P11 design audit filed it as **I3** and its § 9 item 5 says so in as many
//! words: *"the per-face species sparsity is unmeasured — § 3.4's dense-share
//! pricing may be a large overestimate"*. The record-terms priors (§ 4.4) priced
//! the composition term four ways over a **guessed** width. The user's ruling 3
//! (*"transport planes go SPARSE DAY ONE… a cell's in-transit load only holds its
//! catchment's species"*) is a claim about this number, and nobody had it.
//!
//! ## What it measures, and what a reader must not read into it
//!
//! Three [`SpeciesLayout`]s over the shipped world's **final** epoch:
//!
//! | layout | what a row is | who would read it |
//! |---|---|---|
//! | **window** | the materials in a cell's near-surface `OUTCROP_DOMINANCE_WINDOW_M` | the `shares` plane — what entrainment and creep lift |
//! | **transport** | the window closed downstream over the solve's own routing | `qs_sp` / `dep_sp` — the suspended load and what it set down |
//! | **creep** | the window dilated by one 4-neighbour ring | `creep_sp` — creep moves the *donor's* composition |
//!
//! **The transport row width IS the per-face species count**, and that is the
//! number the composition term is priced against: a face carries a subset of the
//! load its cell holds, so the cell's row width is an exact upper bound on any of
//! its faces' composition length, and equals it whenever the flow does not
//! fractionate (which the residual split guarantees it does not — every species
//! present in the load is offered to every weighted face).
//!
//! ⚠ **SINCE THE CONVERSION LANDED THIS PROBE READS THE SOLVE, NOT A MODEL OF IT**
//! (P11 slice 2). It used to rebuild all three layouts from the finished grid,
//! because the solve did not have any — the planes were dense. Now the solve owns
//! them, so the probe asks it (`Erosion::species_layouts` /
//! `Erosion::species_bytes`) instead of re-deriving. A probe that computed its own
//! layout would be a second construction that can silently disagree with the one
//! the world was actually built by (S-3), and the whole point of the measurement
//! is what the world *costs*.
//!
//! ⚠ **It is ONE epoch — the last one — not a run average.** The routing and the
//! record both move every epoch, and this probe reads the state the run ended in.
//! An early epoch has a thinner record (more of the window is basement deficit, so
//! *narrower* rows) and the same routing shape. So the number here is a **late-run,
//! mature-record** reading, which is the regime the cost question is about, and it
//! is not a claim about the mean over the run.
//!
//! ⚠ **It measures the LAYOUT, which is what can be present, not what is
//! non-zero.** That is deliberate and it is the honest thing to size a CSR
//! against: the row has to exist before the solve can put anything in it.
//!
//! Run: `cargo run --release -p dc-worldgen --example species_sparsity_probe`

use std::time::Instant;

use dc_worldgen::deeptime::species::SpeciesLayout;
use dc_worldgen::deeptime::{DeepConfig, production_config, run_cells};
use dc_worldgen::pregen::{CellGrid, Extent, Pregen, WorldParams};

const SEED: u64 = 1337;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn cfg(cells: &CellGrid) -> DeepConfig {
    DeepConfig {
        material_transport: true,
        material_creep: true,
        ..production_config(cells, SEED)
    }
}

/// One layout's sparsity, with the dense comparand it displaces.
struct Sparsity {
    occupied: usize,
    nnz: usize,
    mean: f64,
    max: usize,
    /// Row-width histogram, index = species in the row, last bucket is ">= 15".
    hist: [usize; 16],
}

fn sparsity(l: &SpeciesLayout, n: usize) -> Sparsity {
    let mut hist = [0usize; 16];
    for c in 0..n {
        let k = l.mask_at(c).count_ones() as usize;
        hist[k.min(15)] += 1;
    }
    Sparsity {
        occupied: l.occupied_cells(),
        nnz: l.nnz(),
        mean: l.mean_row_width(),
        max: l.max_row_width(),
        hist,
    }
}

struct Report {
    axis_w: usize,
    axis_names: Vec<&'static str>,
    n: usize,
    deep_secs: f64,
    window: Sparsity,
    transport: Sparsity,
    creep: Sparsity,
    /// Bytes: the four budget planes dense over the axis, against CSR-sparse.
    dense_planes_bytes: usize,
    sparse_planes_bytes: usize,
    /// The seven-class world the planes cost before member grade, for scale.
    class_planes_bytes: usize,
}

fn measure(cells: &CellGrid) -> Report {
    let t = Instant::now();
    let run = run_cells(cells, &cfg(cells), true);
    let deep_secs = t.elapsed().as_secs_f64();
    let grid = &run.grid;
    let n = grid.w * grid.w;
    let er = &run.erosion;
    let axis = er.species_axis();
    let w = axis.len();
    // **Asked of the solve, not re-derived.** These are the layouts the last epoch
    // actually ran on.
    let (window, transport, creep) = er.species_layouts();

    // The four budget planes: `qs_sp` and `dep_sp` over the transport layout,
    // `creep_sp` over the creep layout, `shares` over the window layout — plus the
    // three CSR row indices, which the solve also pays for.
    let sparse_planes_bytes = er.species_bytes();
    let dense_planes_bytes = n * w * 8 * 4;
    let class_planes_bytes = n * 7 * 8 * 4;

    Report {
        axis_w: w,
        axis_names: axis
            .materials()
            .iter()
            .map(|m| m.qualified_name())
            .collect(),
        n,
        deep_secs,
        window: sparsity(window, n),
        transport: sparsity(transport, n),
        creep: sparsity(creep, n),
        dense_planes_bytes,
        sparse_planes_bytes,
        class_planes_bytes,
    }
}

fn print_layout(name: &str, s: &Sparsity, n: usize, w: usize) {
    println!(
        "  {name:<10}  occupied {:>7} / {n}  nnz {:>9}  mean row {:>5.3}  max {:>2} / {w}",
        s.occupied, s.nnz, s.mean, s.max
    );
    print!("             widths:");
    for (k, &c) in s.hist.iter().enumerate() {
        if c > 0 {
            print!(" {k}:{c}");
        }
    }
    println!();
}

fn main() {
    let params = WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    };
    let pregen = Pregen::run(params);
    let r = measure(&pregen.grid);

    println!("\n=== P11 slice 2 — deep species sparsity (seed {SEED}, Medium) ===");
    println!("deep run {:.2} s · {} cells", r.deep_secs, r.n);
    println!(
        "\nspecies axis: {} materials (descending settling energy)\n  {}",
        r.axis_w,
        r.axis_names.join(", ")
    );
    println!("\nlayouts (FINAL epoch — see the module docs; not a run average):");
    print_layout("window", &r.window, r.n, r.axis_w);
    print_layout("transport", &r.transport, r.n, r.axis_w);
    print_layout("creep", &r.creep, r.n, r.axis_w);

    println!("\nTHE NUMBER THE DESIGN PRICED BLIND:");
    println!(
        "  per-face species count p = {:.3} mean (max {}), against a dense width of {}",
        r.transport.mean, r.transport.max, r.axis_w
    );
    println!(
        "  → the load is {:.1}x sparser than the axis it is drawn from",
        r.axis_w as f64 / r.transport.mean.max(1e-9)
    );

    println!("\nthe four budget planes:");
    println!(
        "  class-grade (7, dense, what shipped)   {:>8.2} MiB",
        mib(r.class_planes_bytes)
    );
    println!(
        "  member-grade dense ({} × n × f64)      {:>8.2} MiB   <- the wrong asymptote",
        r.axis_w,
        mib(r.dense_planes_bytes)
    );
    println!(
        "  member-grade CSR-sparse (values+index)  {:>8.2} MiB   <- ruling 3",
        mib(r.sparse_planes_bytes)
    );
    println!(
        "  sparse is {:.2}x the dense member-grade cost and {:.2}x the shipped class-grade cost",
        r.sparse_planes_bytes as f64 / r.dense_planes_bytes as f64,
        r.sparse_planes_bytes as f64 / r.class_planes_bytes as f64
    );
}

/// **The gate** (CLAUDE.md § Gates — an example that can fail belongs in the
/// gate; `[[example]] test = true`).
///
/// Run at `Extent::Small`, because every invariant asserted here is **scale-free**:
/// each is a per-cell or per-edge predicate about the layout construction, not a
/// magnitude. The *magnitudes* are what `main`'s Medium report is for.
#[cfg(test)]
mod gate {
    use super::*;

    fn small() -> Report {
        let params = WorldParams {
            seed: SEED,
            extent: Extent::Small,
        };
        measure(&Pregen::run(params).grid)
    }

    /// **The claim ruling 3 rests on**: a cell's load holds its catchment's
    /// species, not the world's. If the mean row were the axis width, sparse
    /// would be dense with extra bookkeeping and the ruling would be wrong.
    ///
    /// Scale-free: it is a statement about the *closure* — a species only reaches
    /// a cell that something upstream of it released — which is a property of the
    /// drainage graph at any size.
    #[test]
    fn the_load_is_sparser_than_the_axis() {
        let r = small();
        assert!(r.axis_w >= 8, "vanilla registers at least 8 deep materials");
        assert!(
            r.transport.mean < r.axis_w as f64,
            "the transport rows ({:.3}) are as wide as the axis ({}) — the closure is not \
             discriminating and sparse buys nothing",
            r.transport.mean,
            r.axis_w
        );
        assert!(
            r.window.mean <= r.transport.mean,
            "a cell's own window ({:.3}) cannot hold more species than its catchment \
             closure ({:.3})",
            r.window.mean,
            r.transport.mean
        );
    }

    /// **The measured residency is the one the solve pays**, and it must be
    /// strictly under the dense member-grade comparand — otherwise sparse is dense
    /// with extra bookkeeping and ruling 3 bought nothing. Asserted as an
    /// inequality, never as a MiB figure: the magnitudes move whenever the record
    /// or the routing does, and a test pinned to yesterday's number fails because a
    /// colleague improved the world.
    ///
    /// Scale-free: it is `nnz < n × width`, a statement about the row index.
    #[test]
    fn the_sparse_planes_cost_less_than_the_dense_ones_they_replace() {
        let r = small();
        assert!(
            r.sparse_planes_bytes < r.dense_planes_bytes,
            "CSR ({} B) is not cheaper than dense member grade ({} B)",
            r.sparse_planes_bytes,
            r.dense_planes_bytes
        );
    }

    /// **The window is always answered.** Every cell has a near-surface window,
    /// because the deficit below a short record is charged to basement — so no
    /// cell may be unoccupied, at any extent.
    #[test]
    fn every_cell_has_a_window_composition() {
        let r = small();
        assert_eq!(
            r.window.occupied, r.n,
            "a cell with an empty window means the basement deficit stopped being charged"
        );
    }

    /// **The itemisation equals its own total** — the CSR index and the row
    /// histogram are two derivations of the same layout, and they must agree.
    /// The classic false-green shape: an index that is right and a payload that
    /// is not.
    #[test]
    fn the_row_index_agrees_with_the_masks() {
        let r = small();
        for s in [&r.window, &r.transport, &r.creep] {
            let from_hist: usize = s
                .hist
                .iter()
                .enumerate()
                .map(|(k, &c)| k * c)
                .sum::<usize>();
            assert_eq!(
                from_hist, s.nnz,
                "the CSR row count disagrees with the presence masks it was built from"
            );
        }
    }

    /// **Creep is a superset of the window** — it moves the donor's composition
    /// across four edges, so a cell's creep row holds its own window and its
    /// neighbours'. Scale-free: a per-cell set inclusion.
    #[test]
    fn creep_rows_contain_their_own_window() {
        let r = small();
        assert!(
            r.creep.mean >= r.window.mean,
            "creep rows ({:.3}) are narrower than the windows they gather ({:.3})",
            r.creep.mean,
            r.window.mean
        );
    }
}
