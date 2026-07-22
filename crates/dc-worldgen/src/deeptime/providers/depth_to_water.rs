//! **Slot `depth_to_water`** — *how close to the surface is the water table at
//! this cell?*
//!
//! - Owing system: **hydrology** (the heir is S11's saturation field, a
//!   retirement already ratified by `water.md`, DECIDED 2026-07-20).
//! - Granularity: **pass-level, once per epoch**, materialized at
//!   [`BioticSim::step`](crate::deeptime::biotic::BioticSim::step).
//! - Identity: [`identity_depth_to_water`] — leave the plane *empty*, which
//!   routes [`wet_at`] to [`identity_wet_index`], the pre-seam three-term proxy.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::depth_to_water`](field@super::Providers::depth_to_water)). This is
//! the widest seam converted so far — four consumers threshold its output — and
//! journal/0061 § *what the heir must actually supply* states the contract,
//! including the finding that `wet` is not one quantity but three.

/// Everything the water-table pass is allowed to read: the epoch's
/// post-erosion, post-routing state of the deep grid.
///
/// This is a **pass-level** payload — a bundle of *planes*, not a cell — because
/// the heir ([`Providers::depth_to_water`](field@super::Providers::depth_to_water)) is
/// a field and not a per-cell answer. A water table is a solution over a
/// neighbourhood: it needs the drainage network and the filled surface to know
/// where water *collects*, which a per-cell payload structurally cannot carry
/// (journal/0060's lesson — **granularity follows the heir, not the call site**).
///
/// Borrowed slices rather than `&DeepGrid`, for the same reason the value-level
/// payloads are `Copy` structs: the provider sees the inputs it is contracted to
/// read and nothing else, so its declared reads and its actual reads are the
/// same list (ARCHITECTURE.md § *Provider seams*, "effective reads").
#[derive(Clone, Copy, Debug)]
pub struct WaterPass<'a> {
    /// Grid width; the planes are `w * w`, row-major.
    pub w: usize,
    /// The epoch about to be stepped. The water table moves with the surface,
    /// so unlike [`ParentCell`](super::ParentCell) this pass is re-run every
    /// epoch.
    pub epoch: u32,
    /// Climate moisture, 0..1 (the pregen precipitation plane).
    pub precip: &'a [f32],
    /// Bedrock elevation (m). Surface is `r[i] + h[i]`.
    pub r: &'a [f64],
    /// Regolith thickness (m).
    pub h: &'a [f64],
    /// Accumulated D8 drainage area (cell units) from this epoch's routing.
    pub area: &'a [f64],
    /// D8 receiver of each cell (`-1` = sink) — the drainage network.
    pub recv: &'a [i32],
    /// The depression-filled surface from this epoch's flood. A cell whose fill
    /// sits above its own surface is under standing water.
    pub filled: &'a [f64],
}

/// **Identity for [`Providers::depth_to_water`](field@super::Providers::depth_to_water)**:
/// leave the plane *empty*.
///
/// This is the `biotic` / `erodibility` / `full_agents` / `tectonic_history`
/// pattern the four deep-sim flags already prove — **empty plane + identity
/// accessor** ([`wet_at`]) — and it is what this seam conspicuously lacked. An
/// empty plane costs one `is_empty()` branch per cell and allocates nothing, so
/// "provider absent" is not merely byte-identical, it is *free*: the identity
/// world does not even build a plane it would then read back unchanged.
pub fn identity_depth_to_water(_pass: WaterPass<'_>, out: &mut Vec<f32>) {
    out.clear();
}

/// The three-term waterlogging proxy, verbatim as it stood inline in
/// [`biotic::step_cell`](crate::deeptime::biotic) before this seam existed:
/// climate moisture, plus a bonus for sitting near base level, plus a bonus for
/// receiving upslope drainage, clamped to 0..1.
///
/// **This is a stand-in for a water table** (`docs/design/water.md`, DECIDED
/// 2026-07-20, consequence 4: *"waterlogging becomes 'the water table is at or
/// near the surface here', read from the field"*). The three coefficients
/// (`80 m`, `0.20`; `300` cell-units, `0.15`) are a guess, not a measurement,
/// and they answer in a **dimensionless 0..1 wetness index** rather than in
/// metres below the surface — which is the units mismatch the heir has to
/// resolve. See [`Providers::depth_to_water`](field@super::Providers::depth_to_water)
/// for what the heir must supply.
///
/// Kept bit-for-bit: same operations, same order, same `f64 → f32` cast points.
#[inline]
pub fn identity_wet_index(moist: f32, surf: f64, area: f64) -> f32 {
    let low_bonus = ((80.0 - surf) / 80.0).clamp(0.0, 1.0) as f32 * 0.20;
    let area_bonus = (area / 300.0).min(1.0) as f32 * 0.15;
    (moist + low_bonus + area_bonus).clamp(0.0, 1.0)
}

/// The wetness at cell `i`: the materialized
/// [`Providers::depth_to_water`](field@super::Providers::depth_to_water) plane when
/// one exists, and the exact pre-seam expression ([`identity_wet_index`]) when
/// the plane is empty.
///
/// The same shape as `erosion.rs`'s `wmult_at` / `frost_at`: an accessor whose
/// empty-slice branch *is* the identity, so the identity path cannot drift from
/// the provider path by construction — there is one call site and one branch.
#[inline]
pub fn wet_at(plane: &[f32], i: usize, moist: f32, surf: f64, area: f64) -> f32 {
    if plane.is_empty() {
        identity_wet_index(moist, surf, area)
    } else {
        plane[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registered identity for `depth_to_water` leaves the plane empty —
    /// that *is* the identity, because an empty plane selects [`wet_at`]'s
    /// pre-seam branch. It must also clear a plane a previous epoch filled.
    #[test]
    fn the_identity_depth_to_water_leaves_the_plane_empty() {
        let (precip, r, h) = (vec![0.5f32; 4], vec![10.0f64; 4], vec![1.0f64; 4]);
        let (area, recv, filled) = (vec![7.0f64; 4], vec![-1i32; 4], vec![11.0f64; 4]);
        let pass = WaterPass {
            w: 2,
            epoch: 3,
            precip: &precip,
            r: &r,
            h: &h,
            area: &area,
            recv: &recv,
            filled: &filled,
        };
        let mut plane = vec![0.9f32; 4];
        identity_depth_to_water(pass, &mut plane);
        assert!(
            plane.is_empty(),
            "the identity must not leave a stale plane"
        );
    }

    /// The empty-plane accessor reproduces the three-term proxy exactly, and a
    /// materialized plane is read by index instead — the whole seam, in one
    /// assertion pair.
    #[test]
    fn wet_at_falls_through_to_the_proxy_only_when_the_plane_is_empty() {
        for (moist, surf, area) in [
            (0.10f32, -20.0f64, 0.0f64),
            (0.35, 40.0, 150.0),
            (0.62, 900.0, 4_000.0),
            (0.95, 0.0, 300.0),
        ] {
            let expected = identity_wet_index(moist, surf, area);
            assert_eq!(
                wet_at(&[], 0, moist, surf, area).to_bits(),
                expected.to_bits(),
                "the empty plane must be the pre-seam expression, bit for bit"
            );
            let plane = vec![0.125f32, 0.25];
            assert_eq!(wet_at(&plane, 1, moist, surf, area), 0.25);
        }
    }

    /// The proxy's shape, stated as properties rather than as its own source:
    /// clamped to 0..1, monotone down in elevation and up in drainage area.
    #[test]
    fn the_wet_index_is_clamped_and_monotone() {
        assert_eq!(identity_wet_index(1.0, -500.0, 10_000.0), 1.0);
        assert_eq!(identity_wet_index(0.0, 5_000.0, 0.0), 0.0);
        assert!(identity_wet_index(0.2, 800.0, 10.0) < identity_wet_index(0.2, 10.0, 10.0));
        assert!(identity_wet_index(0.2, 800.0, 10.0) < identity_wet_index(0.2, 800.0, 250.0));
    }
}
