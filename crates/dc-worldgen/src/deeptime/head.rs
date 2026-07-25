//! **The head field** — the *potential* flow descends (`docs/design/flow.md`
//! § 2.4, FLOW continuation (a)). The second §5 **field pass**, modelled on the
//! geotherm ([`super::geotherm`]) and planting the **`head` condition-field**
//! (`dc:field/head`, §14).
//!
//! ## Why a potential and not an elevation
//!
//! *"Flow goes downhill"* forecloses **artesian basins, capillary rise,
//! thermohaline circulation and density/turbidity currents** (flow.md § 2.4). The
//! existing bound-water proxy `H = y + sat` (`water.md`, hydrology-priors § 321)
//! is explicitly **unconfined, not Darcy**: it *is* an elevation, so a confined
//! aquifer whose water stands **above** its own ground is not merely unmodelled
//! there — it is inexpressible. This field is the thing that makes it expressible.
//!
//! > **head = elevation + pressure head.** Where the water is unconfined and in
//! > contact with a free surface the pressure term is zero and head *coincides*
//! > with an elevation — that is physics, not an assumption. Where a confining bed
//! > caps a permeable one, head is set by the **recharge area up-dip**, transmitted
//! > laterally through the bed, and is free to exceed the local ground.
//!
//! **The free regime's divide rule does NOT bind this field** (flow.md § 2.4).
//! There is no divide term anywhere in the solve: the relaxation sees D4 adjacency
//! and material transmissivity, nothing else. Regional aquifers and karst conduits
//! genuinely cross surface divides (the Great Artesian Basin, the Ogallala, karst
//! piracy), and a field that could not would foreclose them.
//! `bound_head_crosses_a_surface_drainage_divide` pins it.
//!
//! ## The solve — steady confined/unconfined groundwater, relaxed
//!
//! Per cell the record already knows what the column is made of: every [`DepUnit`]
//! resolves through [`litho_of_tag`] to a [`Litho`] and thence to a reference
//! material whose sheet carries **`permeability`**. So the column's hydraulic
//! character is *derived*, never stored (S-2):
//!
//! - **transmissivity** `T = Σ(thickness · k)` over the record, plus a
//!   fractured-basement term — the **lateral** conductance;
//! - **vertical conductivity** `k_vert` = the thickness-weighted **harmonic** mean
//!   over the record — series flow, so the tightest bed governs, which is why an
//!   aquitard confines;
//! - **confinement** — a contiguous low-permeability cap of at least
//!   [`CONFINING_CAP_M`] lying over a permeable bed.
//!
//! The field then solves the steady groundwater equation `∇·(T ∇h) = 0` by
//! Gauss–Seidel with **alternating forward/reverse raster sweeps** (deterministic,
//! and information crosses the whole grid in one sweep rather than diffusing a cell
//! at a time), under two boundary conditions:
//!
//! | where | condition |
//! |---|---|
//! | submerged cell, or the domain border | **Dirichlet** at the sea stand / base level |
//! | **unconfined** cell holding free water — a lake, or a stream carrying at least [`STREAM_ANCHOR_AREA`] of drainage | **Dirichlet** at that free-water elevation: the water table *outcrops* there |
//! | **unconfined** cell elsewhere | free, but **capped at its own ground** — a water table cannot stand above the land; it discharges (a seepage face) |
//! | **confined** cell | free, and **uncapped** — this is the artesian degree of freedom |
//!
//! That last row is the whole point. An unconfined cell is pinned or capped by its
//! own surface; a confined one is not, so its head is whatever the material
//! transmits to it from elsewhere. **Artesian is not a special case in this code —
//! it is the absence of a cap.**
//!
//! > **Reading the field for artesian: exclude lakes.** A **lake** cell is pinned at
//! > its *water surface*, which stands above the ground by construction — that is
//! > the lake, not an aquifer, and it is correct. So "head > ground" is artesian
//! > only where no free water stands at the surface (`DeepField::lake`). The
//! > distinction is invisible in the field itself, which is why it is stated here
//! > and enforced in the test and the probe rather than left to a reader to notice.
//!
//! ## The currency, stated once
//!
//! The exchange rate this field drives ([`vertical_exchange`]) is dimensionless and
//! in **exactly the currency the lateral flux record already uses**: the drainage
//! solve seeds `1.0` unit of contributing area per cell per epoch, so
//!
//! > **one unit = one cell-epoch of the seeded source**, and a **unit hydraulic
//! > gradient through unit relative permeability moves one of them.**
//!
//! Gravity drainage through the vadose zone *is* a unit gradient, so recharge is
//! `k_vert` per epoch outright — no fabricated depth scale enters, which is why the
//! record needs no mode flag to say what a vertical magnitude means (flow.md
//! § 11.3). The property sheet's `permeability` is already a relative 0..1 number,
//! so the arithmetic stays dimensionally honest end to end.
//!
//! ## What is deliberately NOT here
//!
//! - **No change to lateral routing.** *(True of this slice. The lateral solve
//!   became multi-flow-direction on 2026-07-25 — flow.md § 2.6.1, journal/0109 —
//!   and `tests/head_field.rs` compares MFD-to-MFD, so the byte-identity claim
//!   still holds as written.)*
//!
//!   **The sentence that stood here — "a multi-flow-direction partition is what
//!   head *unlocks*" — is FALSE, and is corrections #54.** MFD needs *a*
//!   potential, and the **free** regime's potential was already there: the
//!   priority-flood `filled` surface is `z_bed + depth` (bare ground where the
//!   land drains, a flat spill-level water surface inside a depression), which is
//!   free-surface head exactly. The shipped MFD partition descends `filled` and
//!   **does not read this module at all**. Worse, reading it here would be
//!   *wrong*: this plane is the **bound** regime's potential and it is built to
//!   cross surface drainage divides (see
//!   `bound_head_crosses_a_surface_drainage_divide` below) — routing surface water
//!   down it would make rivers cross their own watersheds, which flow.md § 2.4
//!   forbids for the free regime. **Bound MFD — Darcy flux partitioned across
//!   faces on this plane — is real, unbuilt, and belongs to continuation (c) with
//!   the free/bound edge.**
//! - **No recharge source term.** `∇·(T∇h) = −R` needs `R/T` in real units, i.e. a
//!   real conductivity **and** a real precipitation depth; `grid.precip` is
//!   normalized 0..1 with no depth scale. So this is the **recharge-free steady
//!   limit**: the potentiometric surface interpolated through the material between
//!   fixed free-water boundaries. Seam; heir named in the journal.
//! - **No buoyancy/density term.** `head = z + p/(ρg)` with **ρ constant**
//!   ([`FLUID_DENSITY_REL`]). Thermohaline, brine and turbidity flows vary ρ and
//!   flow.md § 2.4 names them explicitly as things this must not foreclose — hence
//!   the constant is *named and multiplied in* rather than folded away, so the term
//!   has somewhere to land.
//! - **No transient storage, no unsaturated (Richards) flow, no anisotropy.** The
//!   solve is steady and saturated; the vertical term is Darcy's law in its 1-D
//!   form.
//! - **Nothing cached that must stay re-derivable.** flow.md § 10.2: paleo-elevation
//!   is a running sum only because nothing compacts. This field stores **no
//!   elevation** — it stores a potential, recomputed from the live surface and the
//!   live record at its own cadence, so compaction lands without staling a cache.

use super::grid::DeepGrid;
use super::lithology::{Litho, litho_of_tag};
use super::recorder::DeepStrata;

/// The opaque id of the **`head` condition-field** (§14) — the second member of
/// the vocabulary after `dc:field/temperature`. Consumers read the field *by this
/// id*; it is not an ad-hoc private plane.
pub const FIELD_HEAD: &str = "dc:field/head";

/// The pass cadence (epochs per firing) of the head field pass — the RATE axis
/// (§5 "order × rate").
///
/// Groundwater equilibrates in years to millennia; one epoch here is ~2.5 Myr. So
/// the potentiometric surface is **quasi-static** between firings and this cadence
/// is a *sampling* rate on the topography that drives it, not a relaxation rate.
/// It matches [`DeepConfig::remarch_interval`](super::grid::DeepConfig::remarch_interval)
/// (20) because the same thing is true of the climate field it is sampled beside.
pub const HEAD_PERIOD: u32 = 20;

/// Gauss–Seidel sweeps per firing, alternating forward/reverse raster order.
///
/// A **fixed** count, not a convergence check: the sim logic must be deterministic
/// and free of data-dependent iteration (the S9 rule — a timing harness may wrap
/// `Instant` around a solve, never inside one). Alternating sweep direction is what
/// makes this enough: a reverse sweep propagates a boundary value across the whole
/// grid in one pass, where pure Jacobi would need `O(w²)` iterations to do it. The
/// residual is *reported* ([`march`] returns it) rather than used to stop.
pub const HEAD_RELAX_SWEEPS: usize = 24;

/// Relative permeability at or above which a bed counts as an **aquifer**. The
/// deep tier's reference materials straddle this cleanly — sandstone 0.35, peat
/// 0.50, charcoal 0.75 above; mudstone 0.02, carbonaceous mudstone 0.03, coal
/// 0.05, granite 0.02 below.
pub const AQUIFER_K_MIN: f64 = 0.20;

/// Relative permeability at or below which a bed counts as **confining**.
/// Deliberately separated from [`AQUIFER_K_MIN`] by a gap: a bed in between is
/// *leaky* — it neither confines nor conducts — and ends the cap walk without
/// conferring confinement.
pub const AQUITARD_K_MAX: f64 = 0.10;

/// Minimum contiguous low-permeability cap (metres) that confines the bed beneath
/// it. **A calibration, not a law** — there is no Earth value to defer to at 460 m
/// cells; it is set to the thickness at which a bed is a mappable seal rather than
/// a lamina, in the same plausible-not-tuned register as S9's physics constants.
pub const CONFINING_CAP_M: f64 = 5.0;

/// Drainage area (contributing cells per epoch) at which a cell is treated as
/// carrying a **perennial stream**, so an unconfined water table outcrops there and
/// is pinned to the ground. Below it the channel is ephemeral and the table sits
/// beneath. ~25 cells ≈ 5.3 km² at 460 m — a catchment that keeps a stream wet.
pub const STREAM_ANCHOR_AREA: f64 = 25.0;

/// Thickness (metres) of **fractured basement** credited with the basement
/// material's own permeability, so every column has a non-zero lateral
/// transmissivity and the relaxation is well-posed even where the record is empty.
/// Real crystalline basement is permeable in its top tens of metres and effectively
/// impermeable below.
pub const BASEMENT_AQUIFER_M: f64 = 50.0;

/// The confining thickness used in the artesian gradient when the measured cap is
/// thinner (metres) — a numerical floor only; a confined column always carries at
/// least [`CONFINING_CAP_M`].
pub const MIN_CONFINING_M: f64 = 1.0;

/// Head tolerance (metres) below which a column counts as saturated to the surface
/// and rejects further recharge, or as exactly at the surface rather than artesian.
pub const HEAD_EPS_M: f64 = 1e-6;

/// Minimum standing-water depth (metres) that counts as a **lake** for the purpose
/// of pinning the water table above the ground.
///
/// Not a taste knob — a **guard against numerical dust**. The priority flood leaves
/// residues of order 10⁻⁵ m on cells that are not ponded at all, and pinning the
/// table at `ground + 2.5e-5` reads out as a column whose water stands above its own
/// land. That is *artesian* by the letter of the comparison and nonsense by the
/// metre, and it manufactured 232 false positives on the first small-world run
/// before this floor existed. One centimetre is below anything the world expresses
/// (a voxel is 0.9 m) and far above the flood's residual.
pub const PONDED_MIN_M: f64 = 0.01;

/// **The buoyancy/density seam** (flow.md § 2.4). `head = z + p/(ρg)`; with one
/// fluid at one density the ratio is the identity and multiplying by it is
/// provably inert (`x * 1.0 == x`). It is named and *applied* rather than folded
/// away so the day a brine, a thermohaline cell or a turbidity current needs
/// `ρ(fluid, T, S)`, the term has a place to land instead of a solve to
/// renegotiate. Heir: continuation (d), fluid identity.
pub const FLUID_DENSITY_REL: f64 = 1.0;

/// The **relative permeability** of a deep-tier lithology, read off its reference
/// material's property sheet. Derived, never a second table (S-2): the same sheet
/// [`super::lithology::resistance_of_material`] reads for erodibility.
#[inline]
pub fn permeability_of(litho: Litho) -> f64 {
    f64::from(litho.reference_material().props().permeability)
}

/// **The hydraulic character of one column**, derived from the strata record.
///
/// Everything here is a *derivation over units the recorder already stamped* — no
/// new stored state, and it re-derives correctly the day compaction changes unit
/// thicknesses (flow.md § 10.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnHydro {
    /// **Lateral transmissivity** `Σ(thickness · k)` over the record plus the
    /// fractured-basement term — metres × relative permeability.
    pub transmissivity: f64,
    /// **Vertical relative permeability**: the thickness-weighted harmonic mean
    /// over the record (series flow — the tightest bed governs). The basement
    /// value where the record is empty.
    pub k_vertical: f64,
    /// Thickness (m) of the contiguous low-permeability cap at the top of the
    /// record. Zero when the topmost bed is not an aquitard.
    pub cap_m: f64,
    /// **Confined**: a permeable bed lies beneath a cap of at least
    /// [`CONFINING_CAP_M`]. The one bit that decides whether this column's head is
    /// capped at its own ground or free to stand above it.
    pub confined: bool,
}

impl ColumnHydro {
    /// The bedrock-only column — an empty record. Unconfined by construction:
    /// there is nothing above the basement to seal it.
    pub fn bare_basement() -> Self {
        let k = permeability_of(Litho::Basement);
        Self {
            transmissivity: k * BASEMENT_AQUIFER_M,
            k_vertical: k,
            cap_m: 0.0,
            confined: false,
        }
    }
}

impl Default for ColumnHydro {
    fn default() -> Self {
        Self::bare_basement()
    }
}

/// Derive a column's hydraulic character from its strata record.
///
/// Walks the record **top-down** for the cap (which is what confinement is about —
/// the seal above the aquifer) and bottom-up for the bulk sums (which are
/// order-free). A unit's permeability comes from its measured tag through
/// [`litho_of_tag`], the same routing the erodibility coupling and the collapse
/// tier use, so erosion, the collapse and the head field never disagree about what
/// a bed is made of.
pub fn column_hydro(strata: &DeepStrata) -> ColumnHydro {
    if strata.units.is_empty() {
        return ColumnHydro::bare_basement();
    }
    let basement_k = permeability_of(Litho::Basement);
    let mut transmissivity = basement_k * BASEMENT_AQUIFER_M;
    let mut thickness = 0.0f64;
    // Series resistance Σ(t / k) — the harmonic-mean denominator.
    let mut resistance = 0.0f64;
    for u in &strata.units {
        let k = permeability_of(litho_of_tag(u.tag)).max(HEAD_EPS_M);
        transmissivity += u.thickness_m * k;
        thickness += u.thickness_m;
        resistance += u.thickness_m / k;
    }
    let k_vertical = if resistance > 0.0 && thickness > 0.0 {
        thickness / resistance
    } else {
        basement_k
    };

    // The cap: contiguous aquitard from the top down, and whether what it rests on
    // is permeable enough to be an aquifer.
    let mut cap_m = 0.0f64;
    let mut seals_an_aquifer = false;
    for u in strata.units.iter().rev() {
        let k = permeability_of(litho_of_tag(u.tag));
        if k <= AQUITARD_K_MAX {
            cap_m += u.thickness_m;
            continue;
        }
        // First non-confining bed under the cap ends the walk. Only a genuinely
        // permeable one is an aquifer; a leaky bed between the two thresholds is
        // neither (flow.md § 10.3 — permeability decides whether flow PASSES).
        seals_an_aquifer = k >= AQUIFER_K_MIN;
        break;
    }
    ColumnHydro {
        transmissivity,
        k_vertical,
        cap_m,
        confined: seals_an_aquifer && cap_m >= CONFINING_CAP_M,
    }
}

/// **The vertical exchange rate** across a column's top, per epoch, signed:
/// **positive = DOWN** (infiltration / recharge), **negative = UP** (artesian rise,
/// a spring's last step). Zero where neither happens.
///
/// This is Darcy's law in its one-dimensional form, `q = −K ∂h/∂z`, in the currency
/// the module docs fix (one unit = one cell-epoch of the seeded source):
///
/// - **Down.** Gravity drainage through the vadose zone runs at a **unit hydraulic
///   gradient**, so `q = k_vert` outright. No length scale, no fabricated
///   precipitation depth. It is **rejected** where the water table already stands at
///   the ground (saturation-excess runoff) — which is exactly the stream and lake
///   cells the solve pinned there, and is why a river cell recharges nothing.
/// - **Up.** The driver is the **excess head over the confining bed**:
///   `q = k_vert · (head − ground) / cap`. This term is **structurally impossible**
///   under `H = y + sat`, where head *is* the ground.
/// - **Submerged cells exchange nothing.** There is no vadose zone under the sea,
///   and submarine groundwater discharge is a real thing this slice does not model
///   — an honest zero, not an assumed one.
///
/// Pure in its arguments, so a test can re-derive the whole plane from the field
/// and the record and demand they agree (`the_exchange_plane_agrees_with_the_field`)
/// — the *summary-agrees-with-the-authority* discipline, since the plane is a cache
/// of this function and never an independent authority.
#[inline]
pub fn vertical_exchange(head_m: f64, ground_m: f64, sea_level: f64, hydro: ColumnHydro) -> f64 {
    if ground_m <= sea_level {
        return 0.0;
    }
    let excess = head_m - ground_m;
    if excess > HEAD_EPS_M {
        let cap = hydro.cap_m.max(MIN_CONFINING_M);
        -(hydro.k_vertical * excess / cap) * FLUID_DENSITY_REL
    } else if head_m < ground_m - HEAD_EPS_M {
        hydro.k_vertical
    } else {
        0.0
    }
}

/// Series (harmonic) conductance of the face between two cells of transmissivity
/// `a` and `b` — the standard inter-block harmonic mean, so a single aquitard cell
/// throttles the pair rather than being averaged away by a permeable neighbour.
#[inline]
fn harmonic(a: f64, b: f64) -> f64 {
    let s = a + b;
    if s <= 0.0 { 0.0 } else { 2.0 * a * b / s }
}

/// One Gauss–Seidel update of a free cell, returning `|Δh|`.
fn relax_cell(
    i: usize,
    w: usize,
    h: &mut [f64],
    t: &[f64],
    unconfined: &[bool],
    ground: &[f64],
) -> f64 {
    const D4: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let (gx, gy) = ((i % w) as i32, (i / w) as i32);
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for (dx, dy) in D4 {
        let (nx, ny) = (gx + dx, gy + dy);
        if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= w {
            continue;
        }
        let j = ny as usize * w + nx as usize;
        let c = harmonic(t[i], t[j]);
        num += c * h[j];
        den += c;
    }
    if den <= 0.0 {
        return 0.0;
    }
    let mut v = num / den;
    // The seepage face: an UNCONFINED water table cannot stand above its own
    // ground — it discharges. A CONFINED one can, and that is artesian. This one
    // line is the whole difference, and it is why artesian needs no special case.
    if unconfined[i] {
        v = v.min(ground[i]);
    }
    let d = (v - h[i]).abs();
    h[i] = v;
    d
}

/// **The head field-pass solve.** Relax the `head` condition-field into
/// `grid.head`, and its companion vertical-exchange plane into
/// `grid.head_exchange`, from the current topography and the current strata record.
///
/// Arguments are the *live* solve state the runner hands over, never re-derived
/// here:
/// - `filled` / `routed` — the depression-filled surface and the surface it was
///   filled from. **Standing water is their difference**, not `filled` alone: the
///   two come from one solve, so `filled − routed` is a self-consistent lake depth
///   that stays meaningful when it is carried onto a `ground` from a later moment
///   (the finalize re-march). Using `filled` directly against a newer ground reads
///   every cell erosion has since lowered as a lake, which is a whole world of
///   fictitious ponds. Both may be empty (the pre-loop seed, before any routing) —
///   then there is no standing water anywhere, which is the honest initial state.
/// - `ground` — the ground surface `R + H`. **In the loop this is the `Forced`
///   revision** — the same terrain `build_surface` snapshotted `filled`/`routed`
///   from, which is not a coincidence but a requirement: the seepage cap and the
///   free-water anchors have to describe one landscape or they contradict each
///   other. The runner pins that window with a declaration rather than leaving it
///   to the pass order (journal/0107). (The finalize re-march hands a *later*
///   ground with the *same* solve outputs on purpose — hence `filled − routed`
///   above, which survives the mismatch where `filled` alone would not.)
/// - `area` — this epoch's drainage area, for the stream anchors. Empty pre-loop.
///
/// Returns the **final sweep's maximum |Δh|** (metres) — reported, never used to
/// stop (see [`HEAD_RELAX_SWEEPS`]).
pub(crate) fn march(
    grid: &mut DeepGrid,
    filled: &[f64],
    routed: &[f64],
    ground: &[f64],
    area: &[f64],
    sea_level: f64,
) -> f64 {
    let w = grid.w;
    let n = w * w;
    if ground.len() != n {
        return 0.0;
    }
    let mut t = vec![0.0f64; n];
    let mut unconfined = vec![true; n];
    let mut pinned = vec![false; n];
    let mut h = vec![0.0f64; n];
    let mut k_vert = vec![0.0f32; n];
    let mut cap = vec![0.0f32; n];

    for i in 0..n {
        let hydro = grid
            .strata
            .get(i)
            .map_or_else(ColumnHydro::bare_basement, column_hydro);
        t[i] = hydro.transmissivity;
        unconfined[i] = !hydro.confined;
        k_vert[i] = hydro.k_vertical as f32;
        cap[i] = hydro.cap_m as f32;

        let g = ground[i];
        // Standing water is `filled − routed`, both from ONE solve — never
        // `filled` against a `ground` from a later moment (see the doc comment).
        let ponded = match (filled.get(i), routed.get(i)) {
            (Some(f), Some(r)) => (f - r).max(0.0),
            _ => 0.0,
        };
        let is_lake = ponded > PONDED_MIN_M;
        let border = {
            let (gx, gy) = (i % w, i / w);
            gx == 0 || gy == 0 || gx == w - 1 || gy == w - 1
        };
        if g <= sea_level {
            // Submerged: the sea is the persistent base level (flow.md § 5).
            pinned[i] = true;
            h[i] = sea_level;
        } else if border {
            // The domain border is the model's base level.
            pinned[i] = true;
            h[i] = g.max(sea_level);
        } else if !hydro.confined && is_lake {
            // Standing water, and the column is open to it: the table is the lake.
            pinned[i] = true;
            h[i] = (g + ponded).max(sea_level);
        } else if !hydro.confined && area.get(i).copied().unwrap_or(0.0) >= STREAM_ANCHOR_AREA {
            // A perennial stream runs here and the column is open to it: the water
            // table *outcrops* at the ground, exactly — never above it.
            pinned[i] = true;
            h[i] = g;
        } else {
            // Free to float. Start at the ground and relax downward; a confined
            // column may relax back *above* it, which is artesian.
            h[i] = g;
        }
    }

    // The cells the relaxation may move, in raster order — built once.
    let free: Vec<usize> = (0..n).filter(|i| !pinned[*i]).collect();
    let mut residual = 0.0f64;
    for sweep in 0..HEAD_RELAX_SWEEPS {
        residual = 0.0;
        // Alternating sweep direction: a reverse pass carries a boundary value
        // across the whole grid in one go, where a forward-only sweep (or Jacobi)
        // would need O(w²) iterations to diffuse it that far.
        if sweep % 2 == 0 {
            for &i in &free {
                residual = residual.max(relax_cell(i, w, &mut h, &t, &unconfined, ground));
            }
        } else {
            for &i in free.iter().rev() {
                residual = residual.max(relax_cell(i, w, &mut h, &t, &unconfined, ground));
            }
        }
    }

    let exchange: Vec<f32> = (0..n)
        .map(|i| {
            let hydro = ColumnHydro {
                transmissivity: t[i],
                k_vertical: f64::from(k_vert[i]),
                cap_m: f64::from(cap[i]),
                confined: !unconfined[i],
            };
            vertical_exchange(h[i], ground[i], sea_level, hydro) as f32
        })
        .collect();
    grid.head = h;
    grid.head_exchange = exchange;
    residual
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deeptime::recorder::{Aridity, DepEnv, DepTag, EnergyBand};

    /// A marine mud (fine clastic → mudstone, k = 0.02) — the aquitard.
    fn mud() -> DepTag {
        DepTag::mineral(DepEnv::Subsea, Aridity::Humid, EnergyBand::Low)
    }

    /// A high-energy subaerial bed (coarse clastic → sandstone, k = 0.35) — the
    /// aquifer.
    fn sand() -> DepTag {
        DepTag::mineral(DepEnv::Subaerial, Aridity::Humid, EnergyBand::High)
    }

    /// Build a column bottom-up through the recorder's own `deposit`, so the test
    /// fixture is a record the sim could actually have written.
    fn strata(beds: &[(DepTag, f64)]) -> DeepStrata {
        let mut s = DeepStrata::default();
        for (tag, m) in beds {
            s.deposit(*tag, *m, 0);
        }
        s
    }

    /// The sheet contrast the whole model rests on: the deep tier's two clastic
    /// lithologies straddle the aquifer/aquitard thresholds, so a marine
    /// transgression over a fluvial bed **is** a confined aquifer with no
    /// landform-specific code anywhere.
    #[test]
    fn the_reference_lithologies_straddle_the_aquifer_and_aquitard_thresholds() {
        let fine = permeability_of(Litho::ClasticFine);
        let coarse = permeability_of(Litho::ClasticCoarse);
        assert!(
            fine <= AQUITARD_K_MAX,
            "fine clastic k={fine} must confine (<= {AQUITARD_K_MAX})"
        );
        assert!(
            coarse >= AQUIFER_K_MIN,
            "coarse clastic k={coarse} must conduct (>= {AQUIFER_K_MIN})"
        );
        assert!(permeability_of(Litho::Basement) <= AQUITARD_K_MAX);
    }

    /// Confinement is a *derivation over the record*: mud over sand confines, sand
    /// over sand does not, and a mud cap too thin to seal does not.
    #[test]
    fn confinement_is_derived_from_the_record_never_stored() {
        let confined = column_hydro(&strata(&[(sand(), 40.0), (mud(), 30.0)]));
        assert!(confined.confined, "30 m of mud over sand must confine");
        assert!(confined.cap_m >= CONFINING_CAP_M);

        let open = column_hydro(&strata(&[(mud(), 30.0), (sand(), 40.0)]));
        assert!(!open.confined, "sand at the top cannot confine itself");

        let lamina = column_hydro(&strata(&[(sand(), 40.0), (mud(), 1.0)]));
        assert!(
            !lamina.confined,
            "a 1 m mud lamina is not a seal ({} m cap)",
            lamina.cap_m
        );

        // Series flow: the vertical conductivity of the confined column is dragged
        // toward the aquitard, not the aquifer.
        assert!(
            confined.k_vertical < permeability_of(Litho::ClasticCoarse),
            "the aquitard must govern the series path"
        );
        assert!(ColumnHydro::bare_basement().k_vertical > 0.0);
    }

    /// **The artesian case, on a constructed world** — head standing ABOVE the
    /// local ground, which the unconfined `H = y + sat` proxy cannot express at
    /// all. The mechanism is the seepage cap's absence, nothing else.
    #[test]
    fn head_can_exceed_the_local_surface_which_is_artesian() {
        // A 9-wide grid; every row identical, so it reads as a 1-D profile.
        // A concave-up valley: a steep flank falling to a flat floor.
        let w = 9usize;
        let profile = [300.0, 240.0, 180.0, 120.0, 60.0, 20.0, 15.0, 12.0, 10.0];
        let mut r = vec![0.0f64; w * w];
        for gy in 0..w {
            for gx in 0..w {
                r[gy * w + gx] = profile[gx];
            }
        }
        let mut grid = DeepGrid::from_parts(w, 460.0, r.clone(), vec![0.0; w * w]);
        // Every column is an open sandstone aquifer EXCEPT x == 5, which carries a
        // thick marine mud cap over it: the confined cell.
        grid.strata = (0..w * w)
            .map(|i| {
                if i % w == 5 {
                    strata(&[(sand(), 40.0), (mud(), 30.0)])
                } else {
                    strata(&[(sand(), 40.0)])
                }
            })
            .collect();
        // Every cell carries a perennial stream, so the unconfined ones anchor at
        // their own ground and the confined column is the only free one.
        let area = vec![STREAM_ANCHOR_AREA; w * w];
        let residual = march(&mut grid, &r, &r, &r, &area, -1000.0);
        assert!(
            residual < 1e-3,
            "the relaxation did not settle ({residual})"
        );

        let i = 4 * w + 5; // mid-row, the confined column
        let (head, ground) = (grid.head[i], r[i]);
        assert!(
            head > ground + 1.0,
            "head {head} m must stand above the ground {ground} m — that is \
             artesian, and it is precisely what `H = y + sat` cannot say"
        );
        // And the exchange plane must read it as UP-flux (a spring), not recharge.
        assert!(
            grid.head_exchange[i] < 0.0,
            "an artesian column must discharge upward, not recharge"
        );
        // Its unconfined neighbours are capped at their own ground.
        let j = 4 * w + 4;
        assert!(
            grid.head[j] <= r[j] + 1e-9,
            "an unconfined water table stood above its own ground"
        );
    }

    /// **The free regime's divide rule does not bind this field** (flow.md § 2.4).
    /// A ridge separating two catchments, confined throughout: the potential must
    /// fall *continuously* from the high recharge side to the low discharge side
    /// straight **through** the topographic divide — as the Great Artesian Basin,
    /// the Ogallala and karst piracy do. A field that had baked in "never crosses a
    /// drainage divide" could not produce this.
    #[test]
    fn bound_head_crosses_a_surface_drainage_divide() {
        let w = 9usize;
        // A low border ring; inside it a high plateau (x=1), a ridge (x=2..6), and
        // a low valley (x=7).
        let profile = [30.0, 300.0, 400.0, 500.0, 600.0, 500.0, 400.0, 40.0, 30.0];
        let mut r = vec![0.0f64; w * w];
        for gy in 0..w {
            for gx in 0..w {
                r[gy * w + gx] = if gy == 0 || gy == w - 1 {
                    30.0
                } else {
                    profile[gx]
                };
            }
        }
        let mut grid = DeepGrid::from_parts(w, 460.0, r.clone(), vec![0.0; w * w]);
        // Confined everywhere except the plateau (x=1) and the valley (x=7), the
        // two anchors — the recharge outcrop and the discharge zone.
        grid.strata = (0..w * w)
            .map(|i| match i % w {
                1 | 7 => strata(&[(sand(), 40.0)]),
                _ => strata(&[(sand(), 40.0), (mud(), 30.0)]),
            })
            .collect();
        let mut area = vec![0.0f64; w * w];
        for gy in 0..w {
            area[gy * w + 1] = STREAM_ANCHOR_AREA;
            area[gy * w + 7] = STREAM_ANCHOR_AREA;
        }
        march(&mut grid, &r, &r, &r, &area, -1000.0);

        let row = 4 * w;
        let heads: Vec<f64> = (0..w).map(|x| grid.head[row + x]).collect();
        // Head falls monotonically from the recharge outcrop to the discharge
        // zone, straight through the 600 m divide at x = 4.
        for x in 2..=7 {
            assert!(
                heads[x] < heads[x - 1],
                "head is not falling across the divide at x={x}: {heads:?}"
            );
        }
        // And it is nowhere near the topography it crosses — the potential is not
        // a subdued replica of the ridge, it is a through-going gradient.
        assert!(
            heads[4] < r[row + 4] - 100.0,
            "head {} tracks the 600 m crest — the field followed the surface",
            heads[4]
        );
    }

    /// The exchange plane is a **cache of [`vertical_exchange`], never an
    /// independent authority**: re-deriving it from the field and the record must
    /// reproduce it exactly (ARCHITECTURE.md — a summary must agree with the
    /// authority it summarises).
    #[test]
    fn the_exchange_plane_agrees_with_the_field() {
        let w = 7usize;
        let mut r = vec![0.0f64; w * w];
        for (i, v) in r.iter_mut().enumerate() {
            *v = 100.0 + (i % w) as f64 * 30.0;
        }
        let mut grid = DeepGrid::from_parts(w, 460.0, r.clone(), vec![0.0; w * w]);
        grid.strata = (0..w * w)
            .map(|i| {
                if i % 3 == 0 {
                    strata(&[(sand(), 40.0), (mud(), 30.0)])
                } else {
                    strata(&[(sand(), 20.0)])
                }
            })
            .collect();
        let area = vec![0.0f64; w * w];
        march(&mut grid, &r, &r, &r, &area, 0.0);
        for (i, ground) in r.iter().enumerate() {
            let hydro = column_hydro(&grid.strata[i]);
            let want = vertical_exchange(grid.head[i], *ground, 0.0, hydro) as f32;
            assert_eq!(
                grid.head_exchange[i], want,
                "cell {i}: the cached exchange disagrees with the field"
            );
        }
    }

    /// Recharge is **rejected** where the table stands at the ground — a stream
    /// cell discharges, it does not infiltrate (saturation-excess runoff). And a
    /// submerged cell exchanges nothing at all: an honest zero, since submarine
    /// groundwater discharge is not modelled.
    #[test]
    fn saturated_and_submerged_columns_exchange_nothing() {
        let hydro = column_hydro(&strata(&[(sand(), 40.0)]));
        assert_eq!(vertical_exchange(100.0, 100.0, 0.0, hydro), 0.0);
        assert_eq!(vertical_exchange(50.0, -10.0, 0.0, hydro), 0.0);
        // Unsaturated: recharge at the unit gravity gradient — exactly k_vert.
        let q = vertical_exchange(40.0, 100.0, 0.0, hydro);
        assert!(
            (q - hydro.k_vertical).abs() < 1e-12,
            "recharge {q} != k_vert"
        );
    }
}
