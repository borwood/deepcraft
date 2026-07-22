//! **The entry-species measurement probe** — step 2 of the DECIDED material-
//! interface sequence (geology.md, DECIDED 2026-07-22: *"seam now · MEASURE the
//! class-aggregate with a probe, at zero terrain cost · ship f(substance, form)
//! ONCE"*).
//!
//! The deep sim resolves every erosion rate through
//! [`Litho::reference_material`](dc_worldgen::deeptime::Litho::reference_material):
//! six classes, six fixed proxy rocks, and the property sheets of ~30 other
//! registered materials are discarded (spines.md § A-1, § A-7; the
//! `Litho::reference_material` row of the seam inventory). This probe measures
//! **how big that gap is and where** — it changes NO generation code.
//!
//! For the production Medium record (seed `0x0D5EED572026`) it compares, per deep
//! cell and per erosion agent:
//!
//! - **(a) PROXY** — today's shipped susceptibility: the six
//!   [`reference_material`](dc_worldgen::deeptime::Litho::reference_material)
//!   sheets, blended over the near-surface window exactly as erosion does it
//!   (`susceptibility_table` → `blend_susceptibility(exposed_shares, tab)`,
//!   erosion.rs:855-888).
//! - **(b) TRUE AGGREGATE** — the susceptibility that results from aggregating the
//!   ACTUAL property sheets of the members that would fill each class's near-
//!   surface window: for each `Litho`, the abundance-weighted class-mean of
//!   `{smash, cohesion, permeability, solubility}` (geology.md's *"abundance-
//!   weighted class-mean properties"*), pushed through the *same*
//!   `resistance_of_material` formula (lithology.rs:332), then blended over the
//!   same window shares.
//!
//! It reports, per agent: the rate-ratio (true/proxy) distribution over all
//! recorded cells; the % of cells past ±1.1× / ±1.25× / ±1.5×; where the big
//! deltas live (which classes carry intra-class property variance — the only
//! place the aggregate can diverge from the proxy); worst-case columns; the
//! **intra-class variance table** (the mithril-headroom number); and the
//! **form-error** table (lithified proxy vs the loose-form counterpart the record
//! actually mirrors — `H` is the loose plane).
//!
//! Deterministic, seeded, no wall clock. Read-only over generation code.
//!
//! Run: `cargo run --release -p dc-worldgen --example entry_species_probe`

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_INTRUSIVE, CLASS_ORGANIC_CHARCOAL,
    CLASS_ORGANIC_COAL, CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL, FormationContext, GeoMemberDef,
    GeologySet, fitness, vanilla,
};
use dc_core::materials::{DamageType, MaterialId};
use dc_worldgen::deeptime::{
    Agent, Aridity, Litho, LithoResistance, REFERENCE_LITHO, blend_susceptibility, exposed_shares,
    resistance_of_material,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

const SEED: u64 = 0x0D5E_ED57_2026;
// Production abrasion knobs (`DeepConfig::default`), the contrast/cap every agent's
// susceptibility table is built with in erosion.rs (Abrasion/FrostIce/Wave/Eolian
// all use `erodibility_contrast`, `erodibility_max`; only abrasion-creep uses a
// contrast of 1.0). Dissolution has no shipped rate — every roster material is
// insoluble — so its table is identically zero.
const CONTRAST: f64 = 2.5;
const CAP: f64 = 5.0;

// -------------------------------------------------------------------------------
// The (substance, form) machinery under test, replicated at the *property* level.
// -------------------------------------------------------------------------------

/// The four property axes the resistance model reads (lithology.rs:332). Kept as a
/// tiny struct so the class-aggregate can be a mean of these, then pushed through
/// the *same* formula the shipped path uses.
#[derive(Clone, Copy, Default)]
struct Axes {
    smash: f64,
    cohesion: f64,
    permeability: f64,
    solubility: f64,
}

impl Axes {
    fn of(m: MaterialId) -> Axes {
        let p = m.props();
        Axes {
            smash: f64::from(p.resistance(DamageType::Smash)),
            cohesion: f64::from(p.cohesion),
            permeability: f64::from(p.permeability),
            solubility: f64::from(p.solubility),
        }
    }

    fn get(&self, k: usize) -> f64 {
        match k {
            0 => self.smash,
            1 => self.cohesion,
            2 => self.permeability,
            _ => self.solubility,
        }
    }
}

/// Build a [`LithoResistance`] from aggregated property axes — a verbatim mirror of
/// `deeptime::lithology::resistance_of_material` (lithology.rs:332), the only
/// difference being that its input is an *aggregate* of several members' sheets
/// rather than one material's. Property-level aggregation is what the DECIDED text
/// asks for (*"aggregating the ACTUAL property sheets … → resistance_of_material"*).
fn resistance_from_axes(a: Axes) -> LithoResistance {
    LithoResistance {
        abrasion: a.smash,
        dissolution: if a.solubility > 0.0 {
            1.0 / a.solubility
        } else {
            f64::INFINITY
        },
        frost_ice: a.smash * (1.0 - 0.5 * a.permeability),
        wave: a.smash * a.cohesion.max(0.05),
        eolian: a.cohesion.max(0.05),
    }
}

/// The content class a [`Litho`] resolves to at collapse. [`Litho::Basement`] has no
/// recorded tag, so it is aggregated over the **igneous-intrusive** class — the
/// petrologically honest composition of crystalline basement, and exactly what the
/// *recorded* intrusive basement itself resolves to (granite + diorite). Proxy for
/// basement is granite. The recorded lithologies match `geology::deep_class`.
fn class_of_litho(l: Litho) -> &'static str {
    match l {
        Litho::ClasticFine => CLASS_CLASTIC_FINE,
        Litho::ClasticCoarse => CLASS_CLASTIC_COARSE,
        Litho::OrganicSoil => CLASS_ORGANIC_SOIL,
        Litho::OrganicPeat => CLASS_ORGANIC_PEAT,
        Litho::OrganicCoal => CLASS_ORGANIC_COAL,
        Litho::OrganicCharcoal => CLASS_ORGANIC_CHARCOAL,
        Litho::Basement => CLASS_IGNEOUS_INTRUSIVE,
    }
}

/// Paleo-precipitation proxy for a deposition aridity — a verbatim mirror of the
/// private `geology::deep_precip` (geology.rs:289): the recorder's aridity axis
/// mapped onto the normalized-precip axis clastic fitness reads.
fn deep_precip(aridity: Aridity) -> f64 {
    match aridity {
        Aridity::Arid => 0.12,
        Aridity::Humid => 0.60,
    }
}

/// Selection weight of one member under a formation context: `fitness × abundance`,
/// exactly as `GeologySet::select` composes it (geology.rs:395). `ctx = None` means
/// abundance-only (context-free) weighting — the primary "class-mean properties".
fn member_weight(def: &GeoMemberDef, ctx: Option<&FormationContext>) -> f64 {
    match ctx {
        None => def.abundance,
        Some(c) => fitness(&def.window, c) * def.abundance,
    }
}

/// The abundance-(and-optionally-fitness-)weighted class-mean property axes for a
/// lithology's class. Single-member classes return that member's own axes, so the
/// aggregate is bit-identical to the proxy there by construction.
fn aggregate_axes(set: &GeologySet, l: Litho, ctx: Option<&FormationContext>) -> Axes {
    let class = class_of_litho(l);
    let members = set.class(class).expect("vanilla class present").members();
    let mut acc = Axes::default();
    let mut wsum = 0.0;
    for &idx in members {
        let def = set.member(idx);
        let w = member_weight(def, ctx);
        if w <= 0.0 {
            continue;
        }
        let ax = Axes::of(def.material);
        acc.smash += w * ax.smash;
        acc.cohesion += w * ax.cohesion;
        acc.permeability += w * ax.permeability;
        acc.solubility += w * ax.solubility;
        wsum += w;
    }
    if wsum <= 0.0 {
        return Axes::of(l.reference_material());
    }
    Axes {
        smash: acc.smash / wsum,
        cohesion: acc.cohesion / wsum,
        permeability: acc.permeability / wsum,
        solubility: acc.solubility / wsum,
    }
}

const ZERO_RES: LithoResistance = LithoResistance {
    abrasion: 0.0,
    dissolution: 0.0,
    frost_ice: 0.0,
    wave: 0.0,
    eolian: 0.0,
};

/// Per-lithology proxy resistance table (the shipped path: `l.resistance()`).
fn proxy_table() -> [LithoResistance; Litho::COUNT] {
    let mut out = [ZERO_RES; Litho::COUNT];
    for l in Litho::ALL {
        out[l.index()] = l.resistance();
    }
    out
}

/// Per-lithology TRUE class-aggregate resistance table.
fn true_table(set: &GeologySet, ctx: Option<&FormationContext>) -> [LithoResistance; Litho::COUNT] {
    let mut out = proxy_table();
    for l in Litho::ALL {
        out[l.index()] = resistance_from_axes(aggregate_axes(set, l, ctx));
    }
    out
}

/// Susceptibility table for one agent from a resistance table, referenced to
/// `ref_litho_res`'s resistance on that agent's axis — the shipped normalization
/// (`susceptibility_table`, lithology.rs:618, reference = `REFERENCE_LITHO`).
fn sus_table(
    res: &[LithoResistance; Litho::COUNT],
    agent: Agent,
    ref_litho_res: &LithoResistance,
) -> [f64; Litho::COUNT] {
    let reference = ref_litho_res.to(agent);
    let mut out = [1.0; Litho::COUNT];
    for l in Litho::ALL {
        out[l.index()] = res[l.index()].susceptibility(agent, reference, CONTRAST, CAP);
    }
    out
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[(((sorted.len() - 1) as f64) * q).round() as usize]
}

/// Two-sided deviation factor: `max(r, 1/r)` — "how many × off, either direction".
fn dev(r: f64) -> f64 {
    if r > 0.0 && r.is_finite() {
        r.max(1.0 / r)
    } else {
        f64::NAN
    }
}

fn main() {
    println!("=== entry-species measurement probe — seed {SEED:#x}, Medium ===\n");
    let set = vanilla();
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Medium,
    });
    let deep = &pregen.deep;
    let n_cells = deep.strata.len();
    let recorded: Vec<usize> = (0..n_cells)
        .filter(|&i| !deep.strata[i].units.is_empty())
        .collect();
    println!(
        "grid {}×{} = {} deep cells, {} with a recorded column ({:.1}%)\n",
        deep.w,
        deep.w,
        n_cells,
        recorded.len(),
        recorded.len() as f64 / n_cells.max(1) as f64 * 100.0
    );

    // -----------------------------------------------------------------------
    // PART 1 — the intra-class variance table (the mithril-headroom number).
    // -----------------------------------------------------------------------
    println!("== PART 1 — vanilla intra-class property variance (the mithril headroom) ==");
    println!(
        "  per class: members, abundance-weighted mean of each axis, and the min..max spread.\n  \
         spread = max/min (or Δ when min is 0). spread 1.00× ⇒ single-member ⇒ aggregate ≡ proxy.\n"
    );
    let axis_name = ["smash", "cohesion", "permeab.", "solub."];
    for l in Litho::ALL {
        let class = class_of_litho(l);
        let members = set.class(class).unwrap().members();
        let proxy = l.reference_material();
        println!(
            "  {:<9} class {:<26} proxy = {:<22} ({} member(s))",
            l.code(),
            class,
            proxy.props().name,
            members.len()
        );
        let axes: Vec<Axes> = members.iter().map(|&idx| Axes::of(set.member(idx).material)).collect();
        let mean = aggregate_axes(&set, l, None);
        for (k, an) in axis_name.iter().enumerate() {
            let vals: Vec<f64> = axes.iter().map(|a| a.get(k)).collect();
            let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let spread_s = if lo > 0.0 {
                format!("{:.2}×", hi / lo)
            } else {
                format!("Δ{:.3}", hi - lo)
            };
            println!(
                "      {an:<9} mean {:>7.3}   min {lo:>7.3}  max {hi:>7.3}   spread {spread_s}",
                mean.get(k)
            );
        }
    }

    // -----------------------------------------------------------------------
    // PART 2 — per-agent rate-ratio distribution (the substance error).
    // -----------------------------------------------------------------------
    println!("\n== PART 2 — per-cell, per-agent rate ratio (TRUE class-aggregate / PROXY) ==");
    println!(
        "  self-consistent reference (both worlds normalize to their own {} rock — the shipped\n  \
         `REFERENCE_LITHO`), so a pure-fine cell is 1.000 by construction and any deviation\n  \
         elsewhere is a genuine differential change to the rate field.\n",
        REFERENCE_LITHO.code()
    );
    let res_proxy = proxy_table();
    let res_true = true_table(&set, None);
    let ref_proxy = res_proxy[REFERENCE_LITHO.index()];
    let ref_true = res_true[REFERENCE_LITHO.index()];

    for agent in Agent::ALL {
        let tab_proxy = sus_table(&res_proxy, agent, &ref_proxy);
        let tab_true = sus_table(&res_true, agent, &ref_true);

        if agent == Agent::Dissolution {
            let anynz = tab_proxy.iter().chain(tab_true.iter()).any(|&x| x != 0.0);
            println!(
                "  {:<11} PROXY ≡ TRUE ≡ 0 across the whole roster (no soluble member) — ratio\n  \
                 {:<11} UNDEFINED (0/0). This is the axis of *maximal* latent headroom: a\n  \
                 {:<11} carbonate/evaporite pack member takes a cell from 0 to nonzero, an\n  \
                 {:<11} unbounded change no proxy can approximate. (any-nonzero = {anynz})\n",
                agent.name(),
                "",
                "",
                ""
            );
            continue;
        }

        let mut ratios: Vec<f64> = Vec::with_capacity(recorded.len());
        for &i in &recorded {
            let shares = exposed_shares(deep.strata[i].units.as_slice());
            let p = blend_susceptibility(&shares, &tab_proxy);
            let t = blend_susceptibility(&shares, &tab_true);
            if p > 0.0 && t.is_finite() {
                ratios.push(t / p);
            }
        }
        ratios.sort_by(f64::total_cmp);
        let n = ratios.len() as f64;
        let mean = ratios.iter().sum::<f64>() / n.max(1.0);
        let past = |thr: f64| ratios.iter().filter(|&&r| dev(r) >= thr).count() as f64 / n.max(1.0) * 100.0;
        println!(
            "  {:<11} p01 {:.4}  p10 {:.4}  p50 {:.4}  p90 {:.4}  p99 {:.4}  min {:.4}  max {:.4}  mean {:.4}",
            agent.name(),
            pct(&ratios, 0.01),
            pct(&ratios, 0.10),
            pct(&ratios, 0.50),
            pct(&ratios, 0.90),
            pct(&ratios, 0.99),
            ratios.first().copied().unwrap_or(f64::NAN),
            ratios.last().copied().unwrap_or(f64::NAN),
            mean
        );
        println!(
            "  {:<11} cells past ±1.10× {:>6.2}%   ±1.25× {:>6.2}%   ±1.50× {:>6.2}%",
            "",
            past(1.10),
            past(1.25),
            past(1.50)
        );
    }

    // Fixed-reference decomposition: both worlds normalize to the PROXY reference,
    // so single-member classes are exactly 1.000 and each multi-member class shows
    // its own intra-class substance error in isolation.
    println!(
        "\n  -- fixed-reference decomposition (both use the PROXY reference; isolates each\n  \
         class's own substance error — single-member classes are exactly 1.000) --"
    );
    for agent in [Agent::Abrasion, Agent::FrostIce, Agent::Wave, Agent::Eolian] {
        let tab_proxy = sus_table(&res_proxy, agent, &ref_proxy);
        let tab_true = sus_table(&res_true, agent, &ref_proxy); // PROXY reference for both
        println!("    {}:", agent.name());
        for l in Litho::ALL {
            let p = tab_proxy[l.index()];
            let t = tab_true[l.index()];
            let r = if p > 0.0 { t / p } else { f64::NAN };
            println!("      {:<9} proxy {p:>7.4}  true {t:>7.4}   ratio {r:>7.4}", l.code());
        }
    }

    // -----------------------------------------------------------------------
    // PART 3 — worst-case columns.
    // -----------------------------------------------------------------------
    println!("\n== PART 3 — worst-case columns (largest |ln ratio|, abrasion) ==");
    let tab_proxy = sus_table(&res_proxy, Agent::Abrasion, &ref_proxy);
    let tab_true = sus_table(&res_true, Agent::Abrasion, &ref_true);
    let mut worst: Vec<(f64, usize, [f64; Litho::COUNT])> = recorded
        .iter()
        .map(|&i| {
            let shares = exposed_shares(deep.strata[i].units.as_slice());
            let p = blend_susceptibility(&shares, &tab_proxy);
            let t = blend_susceptibility(&shares, &tab_true);
            let r = if p > 0.0 { t / p } else { 1.0 };
            (r.ln().abs(), i, shares)
        })
        .collect();
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    let cell_m = deep.cell_m;
    let half = deep.w as f64 * 0.5;
    for (rank, (_lnr, i, shares)) in worst.iter().take(4).enumerate() {
        let gx = i % deep.w;
        let gy = i / deep.w;
        let wx = (gx as f64 - half + 0.5) * cell_m;
        let wy = (gy as f64 - half + 0.5) * cell_m;
        let p = blend_susceptibility(shares, &tab_proxy);
        let t = blend_susceptibility(shares, &tab_true);
        let mut dom = Litho::Basement;
        let mut domv = 0.0;
        for l in Litho::ALL {
            if shares[l.index()] > domv {
                domv = shares[l.index()];
                dom = l;
            }
        }
        println!(
            "  #{} deep-cell ({gx},{gy}) ≈ world ({wx:>9.0},{wy:>9.0}) m   dominant {} {:.0}%   \
             proxy {p:.4} → true {t:.4}   ratio {:.4}",
            rank + 1,
            dom.code(),
            domv * 100.0,
            t / p,
        );
    }

    // -----------------------------------------------------------------------
    // PART 4 — the FORM error (loose form vs lithified proxy).
    // -----------------------------------------------------------------------
    println!("\n== PART 4 — the FORM error: lithified proxy vs the loose form the record mirrors ==");
    println!(
        "  the deep record mirrors `H`, the LOOSE regolith plane, but every clastic proxy is a\n  \
         LITHIFIED rock (the sim asks how hard loose river sand is and answers with sandstone).\n  \
         these loose↔lithified pairs are ONE substance in two forms (the ratified forms pass);\n  \
         the ratio is the size of the form error, per agent, for the substance-matched pair.\n"
    );
    let pairs: [(&str, MaterialId, MaterialId); 4] = [
        ("fine mud", MaterialId::CLAY, MaterialId::MUDSTONE),
        ("silt", MaterialId::SILT, MaterialId::SILTSTONE),
        ("sand", MaterialId::SAND, MaterialId::SANDSTONE),
        ("gravel", MaterialId::GRAVEL, MaterialId::CONGLOMERATE),
    ];
    // Reference for the form comparison: the lithified fine proxy (mudstone), so the
    // numbers sit on the same axis as PART 2's proxy world.
    let ref_lith = resistance_of_material(MaterialId::MUDSTONE);
    println!(
        "  {:<9} {:<26} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "pair", "loose → lithified", "abras.", "frost", "wave", "eolian", "worst"
    );
    for (name, loose, lith) in pairs {
        let rl = resistance_of_material(loose);
        let rk = resistance_of_material(lith);
        let mut worst = 1.0f64;
        let mut cells = [0.0f64; 4];
        for (k, agent) in [Agent::Abrasion, Agent::FrostIce, Agent::Wave, Agent::Eolian]
            .into_iter()
            .enumerate()
        {
            let sl = rl.susceptibility(agent, ref_lith.to(agent), CONTRAST, CAP);
            let sk = rk.susceptibility(agent, ref_lith.to(agent), CONTRAST, CAP);
            let r = if sk > 0.0 { sl / sk } else { f64::NAN };
            cells[k] = r;
            if dev(r) > dev(worst) {
                worst = r;
            }
        }
        println!(
            "  {name:<9} {:<26} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>7.2}×",
            format!("{} → {}", loose.props().name, lith.props().name),
            cells[0],
            cells[1],
            cells[2],
            cells[3],
            dev(worst)
        );
    }
    println!(
        "  (ratio = loose-form rate ÷ lithified-proxy rate; >1 ⇒ the record's loose material\n  \
         actually erodes FASTER than the proxy the sim charges it as.)"
    );

    // -----------------------------------------------------------------------
    // PART 5 — context sensitivity of the aggregate (robustness of PART 2).
    // -----------------------------------------------------------------------
    println!("\n== PART 5 — context sensitivity: does the class-mean move with formation context? ==");
    println!(
        "  PART 2 used abundance-only weights. Selection is fitness×abundance, and the clastic\n  \
         members' windows overlap heavily, so context should barely move the aggregate. Bracket\n  \
         it with the two paleo-precip regimes the record carries (arid 0.12 / humid 0.60,\n  \
         geology.rs::deep_precip), shallow depositional depth, representative 15°C.\n"
    );
    let ctx = |precip: f64| FormationContext {
        temp_c: 15.0,
        precip,
        depth_m: 3.0,
    };
    for (label, aridity) in [("arid ", Aridity::Arid), ("humid", Aridity::Humid)] {
        let c = ctx(deep_precip(aridity));
        let res_ctx = true_table(&set, Some(&c));
        let ref_ctx = res_ctx[REFERENCE_LITHO.index()];
        let tab_p = sus_table(&res_proxy, Agent::Abrasion, &ref_proxy);
        let tab_t = sus_table(&res_ctx, Agent::Abrasion, &ref_ctx);
        let fine = aggregate_axes(&set, Litho::ClasticFine, Some(&c)).smash;
        let coarse = aggregate_axes(&set, Litho::ClasticCoarse, Some(&c)).smash;
        let mut sh_coarse = [0.0; Litho::COUNT];
        sh_coarse[Litho::ClasticCoarse.index()] = 1.0;
        let mut sh_base = [0.0; Litho::COUNT];
        sh_base[Litho::Basement.index()] = 1.0;
        let rc = blend_susceptibility(&sh_coarse, &tab_t) / blend_susceptibility(&sh_coarse, &tab_p);
        let rb = blend_susceptibility(&sh_base, &tab_t) / blend_susceptibility(&sh_base, &tab_p);
        println!(
            "  {label}: fine-mean smash {fine:.4}  coarse-mean smash {coarse:.4}   \
             pure-coarse ratio {rc:.4}  pure-basement ratio {rb:.4}"
        );
    }
    let fine0 = aggregate_axes(&set, Litho::ClasticFine, None).smash;
    let coarse0 = aggregate_axes(&set, Litho::ClasticCoarse, None).smash;
    println!("  abundance-only baseline: fine-mean smash {fine0:.4}  coarse-mean smash {coarse0:.4}");
}
