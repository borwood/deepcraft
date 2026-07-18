//! Derived stratification (docs/design/materials.md § Stratification is
//! derived, not ticked).
//!
//! Stratification degree is a **pure function** of (contents, time
//! undisturbed) — nothing here ticks, stores, or mutates. The stored multiset
//! stays fully mixed; this module only *derives* the layered view an observer
//! (renderer, band-wise miner) would see. Agitation resets the clock by
//! resetting the `time_undisturbed` input; worldgen deposits pass a huge time
//! and arrive fully banded.
//!
//! Model (cheap version): of the `n` debris eighths, `k(t)` have settled into
//! clean density-sorted bands at the bottom; the remaining `n - k` ride on top
//! as one mixed band. `k` grows monotonically from 0 (fresh mix) to `n`
//! (fully banded) on an integer saturation curve — no floats, so the view is
//! bit-deterministic across platforms.
//!
//! Only debris stratifies: structural fill is rigid and pore fill is trapped.

use super::MaterialId;
use super::contents::VoxelContents;

/// Time (in ledger ticks) at which half the eighths have settled.
pub const STRATIFY_HALF_TIME: u64 = 1_000;

/// Time at which the view is fully banded.
pub const STRATIFY_FULL_TIME: u64 = 16_000;

/// A run of identical material in the derived view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    pub material: MaterialId,
    pub eighths: u8,
}

/// Layered view of a voxel's debris, bottom-up.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StratifiedView {
    /// Clean bands at the bottom, densest first (heavies sink). Mining these
    /// in order yields clean sequential bands.
    pub settled: Vec<Band>,
    /// The still-mixed remainder on top, as a multiset (ascending id — it has
    /// no meaningful internal order). Mining this yields by extraction
    /// resistance instead.
    pub mixed: Vec<Band>,
}

impl StratifiedView {
    /// Total eighths across both regions (equals the debris count).
    pub fn total_eighths(&self) -> u32 {
        self.settled
            .iter()
            .chain(&self.mixed)
            .map(|b| u32::from(b.eighths))
            .sum()
    }
}

/// Settled-eighth count: integer, monotone in `t`, `0` at `t = 0`, `n/2` at
/// [`STRATIFY_HALF_TIME`], saturating to `n` at [`STRATIFY_FULL_TIME`].
fn settled_count(n: u8, t: u64) -> u8 {
    if t >= STRATIFY_FULL_TIME {
        return n;
    }
    // Floor of n*t/(t + HALF): monotone nondecreasing in t, < n before the
    // saturation cutoff. n <= 8 and t < 16000, so no overflow anywhere.
    (u64::from(n) * t / (t + STRATIFY_HALF_TIME)) as u8
}

/// Derive the layered view of `contents` after `time_undisturbed` ticks.
///
/// Deterministic and pure; per-call cost is O(n log n) over at most 8
/// eighths. Settling order (which eighths band first) is densest-first, ties
/// by ascending id — the physical intuition that heavies sink soonest.
pub fn stratify(contents: &VoxelContents, time_undisturbed: u64) -> StratifiedView {
    let debris = contents.debris();
    let n = debris.len() as u8;
    if n == 0 {
        return StratifiedView::default();
    }

    // Global settling order: density descending, ties by id ascending.
    let mut order: Vec<MaterialId> = debris.to_vec();
    order.sort_by(|a, b| {
        b.props()
            .density_kg_m3
            .total_cmp(&a.props().density_kg_m3)
            .then(a.cmp(b))
    });

    let k = settled_count(n, time_undisturbed) as usize;
    let mut view = StratifiedView::default();
    for &m in &order[..k] {
        match view.settled.last_mut() {
            Some(band) if band.material == m => band.eighths += 1,
            _ => view.settled.push(Band {
                material: m,
                eighths: 1,
            }),
        }
    }
    // The unsettled remainder is an unordered mix; present it canonically.
    let mut rest: Vec<MaterialId> = order[k..].to_vec();
    rest.sort_unstable();
    for m in rest {
        match view.mixed.last_mut() {
            Some(band) if band.material == m => band.eighths += 1,
            _ => view.mixed.push(Band {
                material: m,
                eighths: 1,
            }),
        }
    }
    view
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixed_deposit() -> VoxelContents {
        // gravel 1800, sand 1600, snow 300, leaf 150 — distinct densities.
        VoxelContents::debris_only(&[
            MaterialId::LEAF_LITTER,
            MaterialId::SAND,
            MaterialId::GRAVEL,
            MaterialId::SNOW,
            MaterialId::SAND,
            MaterialId::GRAVEL,
        ])
        .unwrap()
    }

    #[test]
    fn fresh_deposit_is_fully_mixed() {
        let view = stratify(&mixed_deposit(), 0);
        assert!(view.settled.is_empty());
        assert_eq!(view.total_eighths(), 6);
    }

    #[test]
    fn deep_time_is_fully_banded_densest_first() {
        let view = stratify(&mixed_deposit(), STRATIFY_FULL_TIME);
        assert!(view.mixed.is_empty());
        assert_eq!(
            view.settled,
            vec![
                Band {
                    material: MaterialId::GRAVEL,
                    eighths: 2
                },
                Band {
                    material: MaterialId::SAND,
                    eighths: 2
                },
                Band {
                    material: MaterialId::SNOW,
                    eighths: 1
                },
                Band {
                    material: MaterialId::LEAF_LITTER,
                    eighths: 1
                },
            ]
        );
        // Even deeper time changes nothing.
        assert_eq!(view, stratify(&mixed_deposit(), u64::MAX));
    }

    #[test]
    fn settling_is_monotone_and_bands_refine() {
        let c = mixed_deposit();
        let mut prev_settled = 0u32;
        let mut prev_expansion: Vec<MaterialId> = Vec::new();
        for t in [
            0,
            100,
            500,
            1_000,
            2_000,
            4_000,
            8_000,
            15_999,
            16_000,
            1 << 40,
        ] {
            let view = stratify(&c, t);
            assert_eq!(view.total_eighths(), 6, "mass conserved at t={t}");
            let settled: u32 = view.settled.iter().map(|b| u32::from(b.eighths)).sum();
            assert!(
                settled >= prev_settled,
                "settled count must not shrink (t={t})"
            );
            // The settled region only ever *extends*: earlier settled eighths
            // stay exactly where they were.
            let expansion: Vec<MaterialId> = view
                .settled
                .iter()
                .flat_map(|b| std::iter::repeat_n(b.material, usize::from(b.eighths)))
                .collect();
            assert!(
                expansion.starts_with(&prev_expansion),
                "bands must refine, not rearrange (t={t})"
            );
            prev_settled = settled;
            prev_expansion = expansion;
        }
        assert_eq!(prev_settled, 6, "fully banded at the end");
    }

    #[test]
    fn stratify_is_deterministic_and_pure() {
        let c = mixed_deposit();
        for t in [0, 777, STRATIFY_HALF_TIME, 12_345] {
            assert_eq!(stratify(&c, t), stratify(&c, t));
        }
        // The stored contents are untouched (pure derivation).
        assert_eq!(c, mixed_deposit());
    }

    #[test]
    fn structure_and_pores_do_not_stratify() {
        let c = VoxelContents::new(
            crate::materials::contents::StructureShape::Slab,
            &[MaterialId::SCREE, MaterialId::GRAVEL],
            &[MaterialId::SILT],
            &[MaterialId::SAND, MaterialId::SNOW],
        )
        .unwrap();
        let view = stratify(&c, STRATIFY_FULL_TIME);
        assert_eq!(view.total_eighths(), 2, "only debris appears in the view");
    }

    #[test]
    fn half_time_settles_half() {
        let c = VoxelContents::debris_only(&[MaterialId::SAND; 8]).unwrap();
        let view = stratify(&c, STRATIFY_HALF_TIME);
        let settled: u32 = view.settled.iter().map(|b| u32::from(b.eighths)).sum();
        assert_eq!(settled, 4);
    }
}
