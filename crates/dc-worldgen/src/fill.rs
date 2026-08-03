//! **Distribution-first strata expression**: integrate the column, then slice
//! it (docs/design/materials.md, DECIDED 2026-07-21; journal/0055).
//!
//! The recorded column ([`StrataRec`]) is a continuous stack of metres. This
//! module is the **one place** those metres are quantized, and it quantizes them
//! *per voxel*, not per recorded unit:
//!
//! > A voxel's eight eighths are filled from the recorded units overlapping its
//! > own 0.9 m span.
//!
//! ## Why the order matters
//!
//! The retired rule computed `Σ round(tᵢ / 0.9)` — each unit rounded on its own,
//! anything under half a voxel dropped, no remainder carried. Honesty requires
//! `round(Σ tᵢ / 0.9)`. The difference is not cosmetic: the errors *compound*
//! instead of cancelling, and on the production world that deleted about
//! three-quarters of the recorded sediment pile. A dune field recording 379
//! units summing to 7.99 m expressed nothing at all, because every individual
//! bed rounded to zero.
//!
//! ## The allocation, exactly
//!
//! For a voxel span, each overlapping material's *true share* in eighths is
//! `8 · overlap / coverage`. That is generally fractional, so:
//!
//! 1. every material takes its **guaranteed whole eighths** (`floor`);
//! 2. the leftover eighths — always exactly `8 − Σ floor` of them — go to the
//!    materials whose **fractional remainders win against one addressed draw**.
//!
//! Step 2 is *systematic sampling* over the remainders: lay the remainders end
//! to end on a line, offset by a single uniform `u ∈ [0,1)`, and take every
//! integer crossing. Two properties fall out and both are load-bearing:
//!
//! - **Exactly the right number of eighths is handed out.** The crossings of a
//!   segment of total length `L` under a `[0,1)` offset number exactly `L`, and
//!   `Σ remainders` is exactly the integer leftover. No clamping, no fixups.
//! - **It is unbiased.** `P(material i takes an extra eighth) = remainder_i`, so
//!   the *expected* composition of a neighbourhood equals the recorded
//!   composition. Deterministic flooring is a biased estimator that always loses
//!   mass — it deletes the thin bed *everywhere*. Stochastic rounding trades
//!   that bias for variance, which is the right trade for a record: the ash band
//!   should exist *somewhere* rather than uniformly nowhere. This is the
//!   user-decided "dither the material eviction across voxels", generalized from
//!   the >8-materials tie case to the whole allocation — with no special case
//!   for the tie condition, because it falls out.
//!
//! All arithmetic is **fixed point** (`FRAC_BITS` fractional bits per eighth) so
//! "exactly 8 eighths" is an integer identity, not a floating-point hope.
//!
//! ## Determinism
//!
//! The draw is `draw_f64(seed, SALT_GEO_FILL, world voxel x, y, z)`. It is a
//! function of the *world* position only: not of chunk order, not of chunk `y`,
//! not of the iteration order of any collection. Classic error diffusion
//! (Floyd–Steinberg) would be sequential and order-dependent and is therefore
//! forbidden here, as it is everywhere else in the generator.
//!
//! The candidate list itself is built from the record's own stratigraphic order
//! (top-down), which is canonical, so two generators with different cache warmth
//! build the same list in the same order and get the same answer.

use dc_core::materials::geology::{
    CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_ORE_PLACER, CLASS_ORGANIC_CHARCOAL,
    GeoMemberIdx, GeologySet,
};
use dc_core::{StructureShape, VoxelContents};
use dc_sim::statistical::rng::Draws;

use crate::draws::{GeoFill, GeoPore};
use crate::geology::StrataRec;

/// Eighths in a voxel.
const EIGHTHS: u64 = 8;
/// Fixed-point fractional bits per eighth.
const FRAC_BITS: u32 = 20;
/// One whole eighth in fixed point.
const ONE: u64 = 1 << FRAC_BITS;
/// A full voxel in fixed point.
const TOTAL: u64 = EIGHTHS << FRAC_BITS;

/// What fills one voxel's span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Exactly one event's material occupies the span — the common case away
    /// from contacts, and the *only* case the pre-0055 world could produce.
    /// Carries the index into [`StrataRec::events`]. Contents are built by the
    /// unchanged per-event path (which is what keeps placer ore, igneous
    /// accessories and the per-voxel-column member dither working exactly as
    /// they did).
    Single(usize),
    /// The span straddles a contact: `(event index, fixed-point eighths)` in
    /// top-down stratigraphic order, summing to exactly [`TOTAL`].
    Mixed(Vec<(usize, u64)>),
}

/// The recorded column sliced into voxel spans, top-down. Built once per
/// chunk-column and shared by the block path and the material path, so the two
/// cannot disagree about where the record is or what is in it.
#[derive(Debug, Clone, Default)]
pub struct ColumnFill {
    /// Indexed by `depth - 1` (depth 1 = the voxel directly under the surface
    /// voxel).
    plans: Vec<Plan>,
}

impl ColumnFill {
    /// Slice a recorded column into voxel spans.
    ///
    /// The record's **top** is the surface, so voxel `d` covers the metre
    /// interval `[(d−1)·voxel_m, d·voxel_m)` measured downward from it. The
    /// record's *bottom* generally lands mid-voxel; the last voxel is claimed by
    /// the record when it is at least half covered (round-to-nearest, the same
    /// quantizer journal/0053 chose for `H`), and its shares are normalized over
    /// the covered part — below the record is unrecorded basement, which carries
    /// no contents at all, so there is nothing to mix with.
    pub fn build(rec: &StrataRec, voxel_m: f64) -> ColumnFill {
        // Top-down spans: (event index, top metres, bottom metres).
        let mut spans: Vec<(usize, f64, f64)> = Vec::with_capacity(rec.events.len());
        let mut acc = 0.0f64;
        for (i, e) in rec.events.iter().enumerate().rev() {
            let t = f64::from(e.thickness_m).max(0.0);
            spans.push((i, acc, acc + t));
            acc += t;
        }
        let total_m = acc;
        let depth_count = (total_m / voxel_m).round().max(0.0) as usize;
        let mut plans = Vec::with_capacity(depth_count);
        // Spans are sorted by depth and consulted in increasing depth order, so
        // one moving cursor covers the whole column.
        let mut cursor = 0usize;
        let mut parts: Vec<(usize, f64)> = Vec::new();
        for d in 0..depth_count {
            let (lo, hi) = (d as f64 * voxel_m, (d + 1) as f64 * voxel_m);
            while cursor < spans.len() && spans[cursor].2 <= lo {
                cursor += 1;
            }
            parts.clear();
            let mut cov = 0.0f64;
            for &(idx, stop, sbot) in &spans[cursor..] {
                if stop >= hi {
                    break;
                }
                let ov = sbot.min(hi) - stop.max(lo);
                if ov <= 0.0 {
                    continue;
                }
                cov += ov;
                // Merge events that would **express identically**: two events of
                // the same member competing separately for eighths would
                // under-represent that member. Adjacent same-member runs are
                // already coalesced in the recorder pass; this catches the
                // interleaved case.
                //
                // The **pore rider is part of the key** (journal/0099): a
                // weathering front is eight bands of one parent member whose
                // pore-slot product share *differs per band* — that difference is
                // the whole gradient, and merging them would attribute the
                // topmost band's share to every band's eighths. No two events in
                // a pre-0099 column can share a member and differ in
                // `accessory` (only the two igneous passes set one, and they
                // select from different classes, hence different members), so
                // widening the key here moves no existing world.
                let ev = &rec.events[idx];
                match parts.iter_mut().find(|(j, _)| {
                    rec.events[*j].member == ev.member && rec.events[*j].accessory == ev.accessory
                }) {
                    Some((_, w)) => *w += ov,
                    None => parts.push((idx, ov)),
                }
            }
            if cov <= 0.0 {
                // Unreachable while `d < depth_count`, but a degenerate record
                // must end the column rather than emit an empty voxel inside it
                // (which would classify to Air and open a hole under solid
                // ground).
                break;
            }
            if parts.len() == 1 {
                plans.push(Plan::Single(parts[0].0));
                continue;
            }
            plans.push(Plan::Mixed(fixed_weights(&parts, cov)));
        }
        ColumnFill { plans }
    }

    /// The plan for a voxel `depth` voxels below the surface voxel (depth 1 =
    /// directly under it). `None` below the record — unrecorded basement.
    pub fn plan(&self, depth: u32) -> Option<&Plan> {
        if depth == 0 {
            return None;
        }
        self.plans.get(depth as usize - 1)
    }

    /// Recorded depth in voxels.
    pub fn depth_count(&self) -> usize {
        self.plans.len()
    }

    /// Rough heap footprint (bytes) of the plan index — a residency report
    /// line, not an authority (P11 slice 3: `ColumnRec` holds several fills and
    /// `column_cache` is runtime-resident).
    pub fn heap_bytes(&self) -> usize {
        self.plans.capacity() * std::mem::size_of::<Plan>()
            + self
                .plans
                .iter()
                .map(|p| match p {
                    Plan::Single(_) => 0,
                    Plan::Mixed(w) => w.capacity() * std::mem::size_of::<(usize, u64)>(),
                })
                .sum::<usize>()
    }
}

/// Normalize overlaps to exactly [`TOTAL`] fixed-point eighths.
///
/// Rounding the **cumulative** sum (rather than each share on its own) is the
/// same `round Σ` discipline this module exists to install, one level down: it
/// makes the weights sum to `TOTAL` as an integer identity, with no residual to
/// sweep up.
fn fixed_weights(parts: &[(usize, f64)], cov: f64) -> Vec<(usize, u64)> {
    let mut out = Vec::with_capacity(parts.len());
    let (mut acc, mut prev) = (0.0f64, 0u64);
    for (i, &(idx, ov)) in parts.iter().enumerate() {
        acc += ov;
        let c = if i + 1 == parts.len() {
            TOTAL
        } else {
            ((acc / cov) * TOTAL as f64)
                .round()
                .clamp(0.0, TOTAL as f64) as u64
        };
        let c = c.max(prev);
        out.push((idx, c - prev));
        prev = c;
    }
    out
}

/// **Addressed stochastic rounding** of fixed-point shares to whole eighths.
///
/// `weights` are fixed-point eighths summing to exactly [`TOTAL`]; `u ∈ [0,1)`
/// is the addressed draw. Returns whole eighths summing to exactly 8 (module
/// docs for why that is an identity rather than a hope).
pub fn allocate(weights: &[(usize, u64)], u: f64) -> Vec<(usize, u8)> {
    let uq = fill_offset(u);
    let mut out = Vec::with_capacity(weights.len());
    let mut c = 0u64;
    for &(idx, w) in weights {
        let whole = w >> FRAC_BITS;
        let c_next = c + (w & (ONE - 1));
        let extra = ((c_next + uq) >> FRAC_BITS) - ((c + uq) >> FRAC_BITS);
        c = c_next;
        let n = whole + extra;
        if n > 0 {
            out.push((idx, n as u8));
        }
    }
    out
}

/// A fixed-point weight as a fractional number of eighths — the *recorded*
/// share a material is entitled to, against which the allocated whole eighths
/// are the unbiased estimator.
pub fn share_eighths(w: u64) -> f64 {
    w as f64 / ONE as f64
}

/// The addressed draw for one world voxel. Position only — no chunk identity,
/// no generation order, no wall clock.
pub fn fill_draw(seed: u64, vx: i64, vy: i64, vz: i64) -> f64 {
    Draws::of::<GeoFill>(seed).unit(&[vx as u64, vy as u64, vz as u64])
}

/// **Width of the eighth-allocation offset**, in bits — how much of the fill
/// draw [`allocate`] and [`allocate_partial`] consume. Public because the claim
/// "the pore rider's offset is disjoint from this" sat in a comment being false
/// for as long as it existed, and the probe that keeps it honest has to know how
/// wide the thing it must be disjoint from actually is.
pub const FILL_OFFSET_BITS: u32 = FRAC_BITS;

/// The fill draw quantized to the offset the allocation actually consumes.
/// Exposed so nothing has to keep a *copy* of this expression in order to reason
/// about what the allocation saw.
pub fn fill_offset(u: f64) -> u64 {
    ((u * ONE as f64) as u64).min(ONE - 1)
}

/// The offset for **one pore-rider rounding decision** — a draw of its own, from
/// its own salt, for one `(voxel, recorded event)` pair.
///
/// This type exists to make a past defect **unrepresentable**. Until
/// journal/0105 [`pore_rider_share`] took a bare `f64` and was handed
/// [`fill_draw`]'s value, out of which it sliced three bits; the two decisions in
/// a contact voxel were therefore drawn from one number. A `f64` will no longer
/// type-check there, the field is private to this module, and the only
/// constructor is [`pore_draw`], which is the only place `SALT_GEO_PORE` is
/// spelled. Re-correlating the two draws now takes a deliberate edit to this
/// file rather than an innocent one at the call site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoreDraw(f64);

impl PoreDraw {
    /// The draw quantized to the three bits [`pore_rider_share`] actually
    /// consumes (`0..8`). Exposed for the decorrelation probe, which has to
    /// compare this offset against every 3-bit window of the fill draw.
    pub fn offset(self) -> u64 {
        ((self.0 * EIGHTHS as f64) as u64).min(EIGHTHS - 1)
    }
}

/// The pore-rider offset draw for one world voxel and one **recorded event**.
///
/// Two separations, both load-bearing (journal/0105):
///
/// - **`SALT_GEO_PORE` vs `SALT_GEO_FILL`** makes this independent of the
///   eighth-allocation draw in the same voxel. Domain separation by salt rather
///   than by bit range is the form that cannot rot: a bit-range carve-out is
///   only disjoint for the widths it was written against, and
///   [`allocate_partial`]'s offset is `FRAC_BITS` wide *today*.
/// - **`event` in the address** makes each band's decision independent of its
///   neighbours' *within the same voxel*. A weathering front is many thin bands
///   of one parent member differing only in pore share, so one contact voxel
///   routinely carries several rider decisions; on a shared offset they all
///   round the same way and their errors add instead of cancelling.
///
/// `event` is the index into [`crate::StrataRec::events`] — the record's own
/// canonical order, not an iteration order.
pub fn pore_draw(seed: u64, vx: i64, vy: i64, vz: i64, event: usize) -> PoreDraw {
    PoreDraw(Draws::of::<GeoPore>(seed).unit(&[vx as u64, vy as u64, vz as u64, event as u64]))
}

/// A **pore rider's** whole eighths inside a host that won `cnt` of this voxel's
/// eight: `cnt · k8 / 8`, stochastically rounded, so the rider's share of a
/// contact voxel is *proportional* to its host's share of it.
///
/// Proportionality is what makes the weathering profile conserve mass through
/// the contacts: over a neighbourhood the expected product eighths equal the
/// recorded product fraction, exactly the unbiasedness argument this module's
/// allocation rests on (journal/0055). Deterministic flooring would delete the
/// deep front's 1/8 tail everywhere — the same bias that once deleted the
/// world's thin beds. The offset is uniform on `0..8`, so
/// `E[share] = cnt · k8 / 8` exactly, which is the property that survived
/// journal/0105's rewiring untouched.
///
/// The offset comes from [`pore_draw`] — **its own** addressed draw. It used to
/// come from bits 8–10 of the voxel's [`fill_draw`], under a comment claiming
/// that was "a low digit… not its high bits" and therefore disjoint from what
/// [`allocate_partial`] consumes. It was not disjoint: `allocate_partial` takes
/// the top `FRAC_BITS` = 20 bits, which contains bits 8–10, so the pore offset
/// was a deterministic function of the allocation offset. See journal/0105 for
/// what that was and was not worth.
pub fn pore_rider_share(cnt: u8, k8: u8, d: PoreDraw) -> u8 {
    let n = (u64::from(cnt) * u64::from(k8.min(8)) + d.offset()) / EIGHTHS;
    (n as u8).min(cnt)
}

/// Canonical contents for a **mixed** voxel: the allocated eighths, split by
/// each member's form.
///
/// Loose clastic materials are granular; everything else (igneous, lithified
/// organic) is structural. A contact voxel therefore reads as:
///
/// - **structure-dominant** (≥ 4/8 structural): the rock, with the loose grains
///   in its **pores** — the same shape the igneous-accessory path already emits,
///   and physically what a weathered rock head with sediment worked into it is.
///   [`dc_core::classify`] gives structure priority, so the voxel's block is the
///   rock, which is the honest summary.
/// - **loose-dominant**: all eight eighths as debris, rock fragments included —
///   which is what a basal conglomerate *is*: clasts in a sediment.
///
/// **Riders** are carried in by substituting into their host event's own
/// allocated eighths (see `WorldGenerator::mixed_at`), the same substitution
/// `contents_for_event` performs: a **placer ore**, which is why a thin fluvial
/// fan still pans gold, and a **loose pore rider**, which is why a weathering
/// front's contact voxels still carry their product (journal/0099) instead of
/// reading as pure parent rock. A *structural* accessory — the sparse 1/8
/// igneous inclusion — is still not: it rides the thick basement bodies, which
/// are voxel-aligned and take the [`Plan::Single`] path where the unchanged
/// per-event constructor handles them (stubs.md #20 names the residue).
/// **Partial fills are first-class here.** `parts` may total fewer than eight
/// eighths — that is the top-of-column remainder (the surface voxel holds only
/// the metres between the voxel floor and the actual ground surface). A loose
/// partial gets [`StructureShape::None`], which is exactly
/// [`VoxelContents::is_loose_only`]: journal/0010's dormant partial-height
/// render, lit up for the first time by real data. A structural partial gets the
/// smallest shape that reserves it, so a 3/8 rock ledge is a slab rather than a
/// full cube pretending to be 5/8 hollow.
pub fn mixed_contents(set: &GeologySet, parts: &[(GeoMemberIdx, u8)]) -> VoxelContents {
    let mut structure: Vec<dc_core::MaterialId> = Vec::with_capacity(8);
    let mut loose: Vec<dc_core::MaterialId> = Vec::with_capacity(8);
    for &(m, n) in parts {
        let def = set.member(m);
        let bucket = if is_loose(set, m) {
            &mut loose
        } else {
            &mut structure
        };
        for _ in 0..n.min(8) {
            bucket.push(def.material);
        }
    }
    structure.truncate(8);
    loose.truncate(8 - structure.len());
    if structure.len() >= loose.len() && !structure.is_empty() {
        let shape = smallest_shape(structure.len() + loose.len());
        VoxelContents::new(shape, &structure, &loose, &[])
            .expect("structure + pore fill fits the shape chosen to hold it")
    } else {
        structure.append(&mut loose);
        VoxelContents::debris_only(&structure).expect("at most eight debris eighths")
    }
}

/// Is this member's material *granular* in a voxel — debris rather than
/// structure? The clastic classes are, and so is a placer ore grain: gold dust
/// rides **inside** the gravel it settled into, which is exactly how
/// `contents_for_event` places it (into the debris multiset), so a mixed voxel
/// must not promote it to structure.
///
/// **Charcoal is loose too** (journal/0063), and this is the whole substance of
/// calling it an inclusion rather than a stratum. Charcoal is friable carbon
/// fragments, not a load-bearing rock; a single charcoal eighth in a bed of mud
/// must ride in the debris multiset the way a gold grain rides in gravel, not
/// stand up as structure and claim the voxel's block identity through
/// `classify`'s structure-first rule.
pub fn is_loose(set: &GeologySet, m: GeoMemberIdx) -> bool {
    let class = set.member(m).class.as_str();
    class == CLASS_CLASTIC_FINE
        || class == CLASS_CLASTIC_COARSE
        || class == CLASS_ORE_PLACER
        || class == CLASS_ORGANIC_CHARCOAL
}

/// The smallest structure shape whose reserved capacity holds `k` eighths.
fn smallest_shape(k: usize) -> StructureShape {
    match k {
        0 => StructureShape::None,
        1..=2 => StructureShape::Quarter,
        3..=4 => StructureShape::Slab,
        _ => StructureShape::Full,
    }
}

/// Allocate `n` eighths (rather than a full eight) from `weights`, for a
/// **partially filled** voxel. Same addressed stochastic rounding, same exact-sum
/// identity — the shares are rescaled to `n` first, by the same cumulative
/// rounding [`fixed_weights`] uses.
pub fn allocate_partial(weights: &[(usize, u64)], u: f64, n: u8) -> Vec<(usize, u8)> {
    if n >= 8 {
        return allocate(weights, u);
    }
    let target = u64::from(n) << FRAC_BITS;
    let mut scaled = Vec::with_capacity(weights.len());
    let (mut cum, mut prev) = (0u64, 0u64);
    for (i, &(idx, w)) in weights.iter().enumerate() {
        cum += w;
        let c = if i + 1 == weights.len() {
            target
        } else {
            ((u128::from(cum) * u128::from(target)) / u128::from(TOTAL)) as u64
        };
        let c = c.max(prev);
        scaled.push((idx, c - prev));
        prev = c;
    }
    allocate_to(&scaled, u, target)
}

fn allocate_to(weights: &[(usize, u64)], u: f64, total: u64) -> Vec<(usize, u8)> {
    let uq = fill_offset(u);
    debug_assert_eq!(weights.iter().map(|(_, w)| w).sum::<u64>(), total);
    let mut out = Vec::with_capacity(weights.len());
    let mut c = 0u64;
    for &(idx, w) in weights {
        let whole = w >> FRAC_BITS;
        let c_next = c + (w & (ONE - 1));
        let extra = ((c_next + uq) >> FRAC_BITS) - ((c + uq) >> FRAC_BITS);
        c = c_next;
        let n = whole + extra;
        if n > 0 {
            out.push((idx, n as u8));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The allocation always hands out exactly eight eighths, for every offset —
    /// the identity the fixed-point arithmetic exists to guarantee.
    #[test]
    fn allocation_always_totals_eight_eighths() {
        let cases: [Vec<(usize, u64)>; 5] = [
            fixed_weights(&[(0, 0.9)], 0.9),
            fixed_weights(&[(0, 0.45), (1, 0.45)], 0.9),
            fixed_weights(&[(0, 0.3), (1, 0.3), (2, 0.3)], 0.9),
            // Thirteen materials in one voxel — more than eighths can carry, the
            // case the user's "dither the eviction" decision is about.
            fixed_weights(&(0..13).map(|i| (i, 0.9 / 13.0)).collect::<Vec<_>>(), 0.9),
            // A wildly uneven span: one thick bed and a scatter of laminae.
            fixed_weights(
                &[(0, 0.61), (1, 0.02), (2, 0.11), (3, 0.03), (4, 0.13)],
                0.9,
            ),
        ];
        for w in &cases {
            assert_eq!(w.iter().map(|(_, x)| x).sum::<u64>(), TOTAL);
            for k in 0..512 {
                let u = k as f64 / 512.0;
                let got: u32 = allocate(w, u).iter().map(|(_, n)| u32::from(*n)).sum();
                assert_eq!(got, 8, "weights {w:?} at u={u}");
            }
        }
    }

    /// **Unbiasedness, at the unit**: averaged over the offset, every material
    /// gets back exactly its recorded share. This is the whole argument for
    /// stochastic over deterministic rounding — flooring would give the 0.19-of-
    /// an-eighth laminae zero eighths *everywhere*, forever.
    #[test]
    fn allocation_is_unbiased_over_the_draw() {
        // Thirteen equal laminae: 0.615 eighths each, so every one of them
        // floors to zero and only stochastic rounding can express any of them.
        let parts: Vec<(usize, f64)> = (0..13).map(|i| (i, 0.9 / 13.0)).collect();
        let w = fixed_weights(&parts, 0.9);
        const N: usize = 4096;
        let mut sum = [0u64; 13];
        for k in 0..N {
            let u = (k as f64 + 0.5) / N as f64;
            for (i, n) in allocate(&w, u) {
                sum[i] += u64::from(n);
            }
        }
        for (i, &s) in sum.iter().enumerate() {
            let mean = s as f64 / N as f64;
            let want = w[i].1 as f64 / ONE as f64;
            assert!(
                (mean - want).abs() < 1e-3,
                "material {i}: mean {mean} eighths, recorded share {want}"
            );
            assert!(s > 0, "material {i} never expressed — that is the old bug");
        }
    }

    /// A span wholly inside one event is a `Single` — the fast path that keeps
    /// ore, accessories and the member dither byte-identical to pre-0055.
    #[test]
    fn a_span_inside_one_event_is_single() {
        let w = fixed_weights(&[(3, 0.9)], 0.9);
        assert_eq!(w, vec![(3, TOTAL)]);
        assert_eq!(allocate(&w, 0.0), vec![(3, 8)]);
        assert_eq!(allocate(&w, 0.999), vec![(3, 8)]);
    }

    /// Voxel addresses spread over a real slab of world, used by the
    /// decorrelation tests below. Deterministic, so these tests are exact
    /// statements about this hash rather than samples that might flake.
    fn addresses() -> impl Iterator<Item = (i64, i64, i64)> {
        (0..40).flat_map(|x| (0..40).flat_map(move |y| (0..25).map(move |z| (x, y, z - 200))))
    }

    /// **The disjointness the retired comment claimed, now asserted — in the one
    /// form that cannot rot.**
    ///
    /// The defect journal/0105 fixed was not "the wrong three bits"; it was that
    /// the pore offset was derived from the fill draw *at all*, so it could never
    /// be independent of it however the bits were carved. The assertion is
    /// therefore not "the bit ranges do not overlap" — that is a claim about
    /// today's `FRAC_BITS` — but **no 3-bit window anywhere in the fill draw
    /// predicts the pore offset better than chance**. Slice the fill offset every
    /// way it can be sliced; each window must agree with the pore offset about
    /// 1 time in 8. The old code scored 100 % at shift 8.
    #[test]
    fn no_window_of_the_fill_draw_predicts_the_pore_offset() {
        let n = addresses().count() as f64;
        for shift in 0..=(FILL_OFFSET_BITS - 3) {
            let mut hits = 0usize;
            for (vx, vy, vz) in addresses() {
                let uq = fill_offset(fill_draw(7, vx, vy, vz));
                if (uq >> shift) & 7 == pore_draw(7, vx, vy, vz, 0).offset() {
                    hits += 1;
                }
            }
            let rate = hits as f64 / n;
            assert!(
                (rate - 0.125).abs() < 0.015,
                "the pore offset agrees with bits {shift}..{} of the fill offset \
                 {:.2} % of the time (chance is 12.5 %) — the two draws in a contact \
                 voxel have been re-coupled",
                shift + 3,
                100.0 * rate,
            );
        }
    }

    /// Two bands **in the same voxel** decide independently. A weathering front
    /// is many thin bands of one parent differing only in pore share, so a
    /// contact voxel routinely holds several rider decisions; when they shared an
    /// offset they rounded in lockstep and their errors added rather than
    /// cancelled. The event index in the address is what breaks that, and this is
    /// the test that fails if it is ever dropped from the key.
    #[test]
    fn two_bands_in_one_voxel_draw_independent_offsets() {
        let n = addresses().count() as f64;
        let same = addresses()
            .filter(|&(vx, vy, vz)| {
                pore_draw(7, vx, vy, vz, 3).offset() == pore_draw(7, vx, vy, vz, 4).offset()
            })
            .count();
        let rate = same as f64 / n;
        assert!(
            (rate - 0.125).abs() < 0.015,
            "sibling bands in one voxel draw the same offset {:.2} % of the time \
             (chance is 12.5 %) — the event index has fallen out of the pore address \
             and every band in a contact voxel rounds in lockstep again",
            100.0 * rate,
        );
    }

    /// **The property the rewiring had to preserve**: averaged over the offset a
    /// rider gets back exactly `cnt · k8 / 8` eighths. Enumerated, not sampled —
    /// the offset has eight values and integer division makes the sum exact.
    #[test]
    fn the_pore_rider_share_is_unbiased_over_its_offset() {
        for cnt in 0..=8u8 {
            for k8 in 0..=8u8 {
                let total: u32 = (0..EIGHTHS)
                    .map(|i| {
                        let d = PoreDraw((i as f64 + 0.5) / EIGHTHS as f64);
                        assert_eq!(d.offset(), i);
                        u32::from(pore_rider_share(cnt, k8, d))
                    })
                    .sum();
                assert_eq!(
                    total,
                    u32::from(cnt) * u32::from(k8),
                    "cnt {cnt}, k8 {k8}: the eight offsets must average to cnt·k8/8 exactly"
                );
            }
        }
    }
}
