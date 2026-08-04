//! Shared fixtures for the provider-seam suites (journal/0060, 0061, 0063):
//! the seed, the pre-slice goldens, and the deterministic fingerprint.
//!
//! Not a test target — a directory under `tests/` with no `main.rs` is not
//! auto-discovered by cargo, so this compiles once into each suite that
//! declares `mod providers_common;`.
//!
//! It lives apart from the suites for the same reason the source module was
//! split: a seam conversion must never have cause to edit the file that holds
//! the byte-identity proof. The goldens are constants here, and
//! `providers_golden.rs` is the only place they are asserted.

// Each suite uses a different subset of these helpers; the module is compiled
// once per suite, so anything a given suite does not call is dead there.
#![allow(dead_code)]

use dc_core::materials::MaterialId;
use dc_worldgen::deeptime::{
    Aridity, Biofacies, DeepField, DepEnv, EnergyBand, Eolian, build_field,
};
use dc_worldgen::pregen::{Extent, Pregen, WorldParams};

/// The seed the goldens were captured at.
pub const SEED: u64 = 0x0B0A_57EE_0059;

// ---------------------------------------------------------------------------
// **§ P11 — WHY EVERY TERRAIN GOLDEN IN THIS FILE MOVED ON 2026-08-01.**
//
// P11 slice 1 took the deep record member-grade: `DepUnit::species` is a registry
// `MaterialId` chosen by member fitness **at deposition**, not a 7-value `Litho`
// class refitted at expression (journal/pending-p11-slice1). Two distinct things
// reach these hashes and it matters which is which:
//
// 1. **The RECORD halves moved because the record changed.** It names the rock
//    now — `dc:mudstone` where it used to say *fine clastic* — and identity joins
//    `deposit_as`'s merge key, so beds that coalesced as one class-grade unit
//    split when their members differ. Both are the slice, working.
//
// 2. **The SURFACE halves moved for a reason that is NOT a physics change, and
//    this is the part worth reading before trusting the diff.** The erosion rule
//    is untouched: `susceptibility_table` is still keyed by class and still built
//    from each class's reference material, so a siltstone bed still erodes at
//    mudstone's rate (slice 2 is what changes that). What moved is *rounding*.
//    The recorder accumulates `top.thickness_m += d` per unit and `erode`
//    subtracts per unit, so a finer segmentation produces different addends;
//    `window_walk` buckets them back to the same class and gets the same quantity
//    to within 1e-12, and the susceptibility blend reads that. One ulp, through
//    the incision rate, compounded over 200 epochs, is a different continent.
//
//    Bounded and pinned at the site where it enters:
//    `lithology::p11_bucket_tests::splitting_a_unit_within_its_class_preserves_the_outcrop_shares`.
//    **A walk that coalesced runs back to the pre-P11 segmentation was built,
//    measured and removed** — it did not restore the world, because the addends
//    differ and not merely their grouping.
//
// So: a *systematic* rate change would have to break mass conservation or move
// the denudation budget, and `deeptime.rs`'s ledger suite (`Δ(ΣR+ΣH) == uplift +
// biotic`) held through the whole re-grade. Read a moved surface hash here as
// **the same landscape rolled from a different ulp**, not as a re-tuned world —
// and note the claim's own limit: it is an argument from the mechanism plus a
// conservation check, **not** a measured before/after of relief, which nothing in
// the tree captures across commits.
// ---------------------------------------------------------------------------

// ═══════════════════════════════════════════════════════════════════════════
// § P11 SLICE 2 — THE CONVERSION (2026-08-02, journal/pending-p11-slice2b)
//
// **Every deep-time golden in this file moved, and two of the three mechanisms
// are SEMANTICS.** Naming which is which is the whole point of this block
// (corrections #89: a segmentation or ordering change moves goldens through float
// accumulation alone, and reporting that as "the rule changed" is as wrong as the
// reverse).
//
// 1. **RULE — the erosion rate stopped being a fact about a CLASS.** The four
//    budget planes went CSR-sparse over `MaterialId`, and with them the window
//    accumulator and the susceptibility table. A cell whose near-surface window
//    holds mudstone *and* siltstone used to blend one multiplier (both bucket to
//    `ClasticFine`, whose reference sheet is mudstone's); it now blends two. Every
//    material that IS its class's reference member keeps its old multiplier bit
//    for bit — so the move is exactly the members that were previously invisible.
//
// 2. **RULE — transported deposits stopped drawing for their identity**
//    (ruling 6). Where a mover outvoted the un-carried remainder, the arriving
//    composition IS the rock and no fitness draw runs. That changes what is
//    recorded, which changes the next epoch's window, which changes the rate.
//    The draw's own domain salt also moved (`DeepMember` 0x5900_0002 →
//    0x5900_0003, resolving a collision with `refine.rs`'s hand-rolled
//    perturbation salt), so every *remaining* draw re-rolled as well.
//
// 3. **ROUNDING — the CSR row is not a dense row with zeros in it.** A budget sum
//    over a 3-slot row and the same sum over a 7-slot row with four zeros are the
//    same number in exact arithmetic and not in IEEE-754 grouping; the
//    coarsest-first drawdown's early `break` moves where partial sums land.
//
// The arms below that pin an OLD reachable solve (`_SCALAR_LOAD`,
// `_ANONYMOUS_CREEP`, `_SINGLE_RECEIVER`, `_UNBOUNDED_CREEP`) moved under (1) and
// (3) even though their own semantics are untouched: what they pin is *"the world
// with this feature off"*, and the world with the feature off now has member-grade
// rates. That is the correct behaviour of a differential fixed point, not a lost
// one — the differential claim each of them makes still holds, and each is still
// reachable.
//
// **Scratch-pad doctrine** (CLAUDE.md § Conventions): goldens are regression
// tripwires, never ratified intent. A hash move produced by ratified semantics
// re-captures with the why recorded; it owes no byte-identicality.
// ═══════════════════════════════════════════════════════════════════════════

/// FNV-1a-64 over the surface planes of the golden fixture `DeepField`.
///
/// **Moved 2026-07-22 by the share-weighted susceptibility blend (journal/0072) —
/// authorized (audit site A1).** Erosion stopped mapping the outcrop *verdict* to
/// one susceptibility-table entry and now blends the table by the near-surface
/// window's per-`Litho` **shares** (the `outcrop_shares` seam), so the per-agent
/// erosion rate field is continuous where the old argmax stepped at the plurality
/// crossover (the walk-0071 S-4 flag). The rates move the erosion *input*, so both
/// the surface planes and the strata record moved. Prior values — set by
/// journal/0068's thickness-dominance rule — kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x344C_89FF_023F_7BAE
/// GOLDEN_RECORD  0xEA71_458F_0AB9_7A05
/// ```
///
/// See `providers_golden.rs` to re-derive.
///
/// **`GOLDEN_SURFACE` did NOT move with the geotherm (journal/0093)** — the first
/// field pass touches no surface plane (`surf`/`regolith`/drainage/`exhum`/
/// `t_crust`), so this value still equals pre-slice `main`. That the surface holds
/// while the record moves is the independent check that the geotherm changed coal
/// and nothing else.
///
/// **Moved 2026-07-25 by MFD routing (journal/0109) — authorized, and this one is
/// a genuine physics change rather than a seam conversion.** Every prior move in
/// this file came from a rate or a tag; this one changes *where the water goes*, so
/// the surface, the drainage export and the record all moved together. The
/// single-receiver world is still reachable and still hashed — see
/// [`GOLDEN_SURFACE_SINGLE_RECEIVER`], asserted by name in `tests/mfd_routing.rs`,
/// which is what makes this an authorized move rather than a lost fixed point.
/// Prior value (pre-MFD `main`), kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x176D_40F1_1CCB_006A
/// ```
/// **Moved 2026-07-26 by material-aware transport (journal/0110) — authorized,
/// and, like MFD, a physics change rather than a seam conversion.** The suspended
/// load became a multiset of `(lithology, quantity)` and deposition became a
/// falling competence ceiling, so material the flow can no longer hold is set down
/// where it stops being holdable rather than where the mass budget happens to
/// overflow. The scalar-load world is still reachable and still hashed — see
/// [`GOLDEN_SURFACE_SCALAR_LOAD`], asserted by name in
/// `tests/material_transport.rs`. Prior value (pre-2b `main`), kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x6F83_4D53_DB89_8C36
/// ```
///
/// **Moved 2026-07-26 by material-aware hillslope CREEP (journal/0112, Movement 2b
/// continuation (b)) — authorized, and it is the largest-authority world move of
/// the three above.** Movement 2b gave the *rivers* an identity and reached
/// 0.000006 % of the archive, because fluvial transport is 0.109 % of this world's
/// sediment routing (corrections #55). Creep moves 918× more and carried none;
/// now it does, and 65.206 % of the archive records what actually arrived. The
/// terrain moves because identity is not a sidecar: the record's rock is what
/// `outcrop_shares` publishes, and that composition sets the erodibility blend, the
/// frost multiplier, the eolian deflation susceptibility, the wave attack rate —
/// and what the fluvial load entrains, which the competence ceiling then rains out.
/// The fluvial-only world is still reachable and still hashed, as
/// [`GOLDEN_SURFACE_ANONYMOUS_CREEP`]. Prior value (post-2b, pre-2b(b)), kept for
/// audit:
///
/// ```text
/// GOLDEN_SURFACE 0x60F0_A669_F4B9_23BD
/// ```
/// **Moved 2026-07-26 by hybrid `p` (journal/0113) — authorized, and a physics
/// change of the same class as MFD itself.** The convergence exponent became
/// spatially varying: dispersive where flow is unchannelised, and single-receiver
/// above a channel-initiation threshold on `χ = A·S²`. That changes where the
/// water goes — the shipped world's peak catchment rises 84 → 265 cells — so the
/// surface, the drainage export and the record all moved together.
///
/// Three fixed points are **deliberately untouched** by it, and that is what makes
/// this an authorized move rather than a lost one: [`GOLDEN_SURFACE_SINGLE_RECEIVER`]
/// (`mfd: false`), [`GOLDEN_SURFACE_SCALAR_LOAD`] and the anonymous-creep pair in
/// `tests/material_creep.rs`. The latter two are reached by pinning the flat ramp
/// `p_chan == p_hill == 4.0`, and they are **bit-identical** rather than merely
/// close, because `partition_cell` skips its `S/S_max` normalisation for a uniform
/// law precisely so that stays true. Prior value (pre-hybrid `main`), kept for
/// audit:
///
/// ```text
/// GOLDEN_SURFACE 0x1A57_A522_C3BA_9F0C
/// ```
///
/// **NOT moved by the joint supply + transport calibration (journal/0114), and that
/// is the entry's headline.** The calibration is built, measured and **off**: the
/// probe found the published 1–10 m/Myr craton band unreachable at any multiplier,
/// and found that turning it on opens deep closed depressions the incision clamp does
/// not hold — 0 pits at 1×, **44 at 5×**, 148 at 45×, deepest 112 m. That is a latent
/// defect in the solve which only a world that actually erodes could expose, and it
/// is not something to ship into the world the player walks on an agent's authority.
/// The calibrated world is reachable and pinned by name as
/// [`GOLDEN_SURFACE_CALIBRATED`], so the flip is one line once the pit defect is
/// fixed.
///
/// **Moved 2026-07-29 by the sub-cycled hillslope operator (journal/0122) —
/// authorized, and it is a correctness fix rather than a capability.** The
/// transport pass is an explicit Laplacian, and an explicit Laplacian has a
/// period-2 grid-scale mode whenever its per-edge coefficient exceeds `1/8`; its
/// flux limiter, capping export at the cell's whole inventory rather than at the
/// amount that would level the pair, turned that divergence into an exactly
/// amplitude-preserving flip-flop (isolated in
/// `erosion.rs::hillslope_operator_tests`). The pass now splits each epoch into
/// `ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)` steps.
///
/// **The shipped world moves only a little, and it moves for a reason worth
/// stating.** `diffusion = 0.12` sits *inside* the bound, so the config rate was
/// never past it — but `eff_diff` folds in the lithology's creep susceptibility,
/// and the shipped world's peak effective diffusivity is **0.261**, so it takes
/// two sub-steps. Measured on production-Medium: mean regolith 4.57 → 3.91 m,
/// relief 5521.9 → 5521.0 m, mean surface 515.2 → 515.0 m, closed hollows 0 → 0.
/// The unbounded world is still reachable and still hashed as
/// [`GOLDEN_SURFACE_UNBOUNDED_CREEP`], asserted by name in
/// `tests/creep_operator.rs`. Prior value (pre-0122 `main`), kept for audit:
///
/// ```text
/// GOLDEN_SURFACE 0x260E_074F_211C_936D
/// ```
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0x15A6_B756_7A84_29FB`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 1, the RULE: the erosion rate is now a function of the ROCK, not of its class, so a window holding mudstone and siltstone blends two multipliers where it blended one — and transported deposits take their identity from the load rather than from a draw. Case 3 rides along: the CSR row visits the species a cell actually holds, so the partial sums of a budget land differently from a dense row padded with zeros. Prior value, kept for audit: `0xBF63_DA9D_2974_022A`.
// Re-captured 2026-08-03 (P11 slice 3, journal/pending-p11-slice3): the packed
// `DepUnit` quantizes recorded thickness to 2^-10 m and the quantized record
// feeds back through the outcrop window into erosion rates, so the whole deep
// trajectory moves once — ratified quantization semantics (U5), NOT float order
// (fixed-point sums are exact and commutative; corrections #89's hazard class
// retires for record sums). The near-path restructure (3a) additionally moved
// the collapse-tier families (per-column record membership, ruling 5).
pub const GOLDEN_SURFACE: u64 = 0x0DB6_2869_EF90_E09B;

/// **The pre-journal/0122 hillslope operator, reachable and pinned.** The same
/// production fixture built with `DeepConfig::creep_substep = false`: one raw
/// explicit step per epoch at whatever per-edge coefficient the config states,
/// which is what every world before 2026-07-29 was generated with.
///
/// It is the same shape as [`GOLDEN_SURFACE_SINGLE_RECEIVER`],
/// [`GOLDEN_SURFACE_SCALAR_LOAD`] and [`GOLDEN_SURFACE_ANONYMOUS_CREEP`]: an old
/// solve that is still reachable, so a moved shipped golden is an authorized move
/// rather than a lost fixed point.
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0x260E_074F_211C_936D`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0x3866_6989_FA2D_FD09`.
// Re-captured 2026-08-03 (P11 slice 3): quantization semantics reach every
// config arm — the un-sub-cycled operator runs over the same packed record.
pub const GOLDEN_SURFACE_UNBOUNDED_CREEP: u64 = 0x5ADF_C706_CB10_BB7F;
/// The strata-record half of [`GOLDEN_SURFACE_UNBOUNDED_CREEP`].
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the rock (case 1). Prior value,
/// kept for audit: `0xACB6_1859_6AA3_F3A8`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0xE106_8099_C93B_1877`.
pub const GOLDEN_RECORD_UNBOUNDED_CREEP: u64 = 0x3911_0859_768D_07F7;

/// **The CALIBRATED world, reachable and pinned** (journal/0114). The same fixture
/// built with [`DeepOverrides::calibrated_rates`](dc_worldgen::deeptime::DeepOverrides)
/// = `Some(true)` — all four erosion rate constants multiplied by
/// `EROSION_CALIBRATION` (45).
///
/// It is the mirror of [`GOLDEN_SURFACE_SINGLE_RECEIVER`] and its siblings: those pin
/// an **old** solve that is still reachable, this pins a **future** one that is
/// already built. Asserted in
/// `tests/calibrated_rates.rs::calibrated_rates_off_is_production_and_on_moves_it`,
/// deliberately not here, which stays the cross-commit golden for the shipped
/// configuration and nothing else.
///
/// Pinning it now is what makes the flip cheap and honest later: when the pit defect
/// that keeps the flag off is fixed, this constant moves and the diff says so, rather
/// than the calibrated world arriving unmeasured alongside the repair.
///
/// **Moved 2026-07-29 by the sub-cycled hillslope operator (journal/0122), and this
/// is the constant it moved *most* — which is the pinning doing exactly its job.**
/// At 45× the pass takes **100 sub-steps** where the shipped world takes 2, so the
/// calibrated world is the one the fix actually reshapes: its regolith concavity rms
/// goes **62.42 → 2.66 m**, its lag-1 autocorrelation **−0.821 → +0.001**, the
/// surface's concavity rms **40.42 → 0.30 m** and closed hollows past 10 m
/// **818 → 12**. Prior value (the world 45× built under the unbounded operator), kept
/// for audit:
///
/// ```text
/// GOLDEN_SURFACE_CALIBRATED 0x8020_8FAF_68F0_6CCD
/// GOLDEN_RECORD_CALIBRATED  0xF17A_ABE0_FA95_9DC4
/// ```
///
/// **⚠ And the number 45 itself is now in question** — it was fitted against the
/// capped operator (journal/0114 measured 100× on transport buying 1.6×, because the
/// limiter had turned the pass into a one-cell-per-epoch conveyor). With the cap gone
/// the same multiplier strips the world to 1.40 m of mean regolith. So this constant
/// still pins "the world `calibrated_rates: Some(true)` builds", which is what it is
/// for; it no longer pins "the world we intend to ship when the flag flips".
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0x53AD_BCCE_B157_09A8`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 above.** Case 1, the RULE, at the calibrated rates: member-grade erosion rates plus the retired deposition draw, exactly as the shipped arm. This constant pins a FUTURE solve so the eventual flip is a diff and not a surprise; it re-captures for the same reason the shipped one does. Prior value, kept for audit: `0xCE38_7587_69F0_64F2`.
// Re-captured 2026-08-03 (P11 slice 3): the calibrated arm records through the
// same packed recorder — see GOLDEN_SURFACE.
pub const GOLDEN_SURFACE_CALIBRATED: u64 = 0x0850_6980_60C4_60F2;
/// The strata-record half of [`GOLDEN_SURFACE_CALIBRATED`].
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the rock (case 1). Prior value,
/// kept for audit: `0x830E_768D_3D1D_866B`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 above.** Case 1, the RULE, at the calibrated rates: member-grade erosion rates plus the retired deposition draw, exactly as the shipped arm. This constant pins a FUTURE solve so the eventual flip is a diff and not a surprise; it re-captures for the same reason the shipped one does. Prior value, kept for audit: `0x8AF6_5B99_579C_436A`.
pub const GOLDEN_RECORD_CALIBRATED: u64 = 0xE6F0_3350_F22B_FC3D;

/// **The pre-MFD fixed point, still reachable.** The same fixture world built with
/// [`DeepConfig::mfd`](dc_worldgen::deeptime::DeepConfig) **off** must reproduce
/// the goldens as they stood before FLOW continuation (b) (journal/0109).
///
/// This is what turns "MFD moved the world" from a lost fixed point into a
/// *declared* one: the old solve is a second path, not a deleted path, and it is
/// proven byte-identical rather than assumed to be. Asserted in
/// `tests/mfd_routing.rs::the_single_receiver_path_still_hashes_to_the_pre_mfd_goldens`
/// — deliberately **not** in `providers_golden.rs`, which stays the cross-commit
/// golden for the *shipped* configuration and nothing else.
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0x176D_40F1_1CCB_006A`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0x98E4_1972_0DA0_8EB6`.
// Re-captured 2026-08-03 (P11 slice 3): same mechanism as SCALAR_LOAD above.
pub const GOLDEN_SURFACE_SINGLE_RECEIVER: u64 = 0xD84C_6A4D_48A8_DCC1;
/// **The pre-2b fixed point, still reachable.** The same fixture world built with
/// [`DeepConfig::material_transport`](dc_worldgen::deeptime::DeepConfig) **off**
/// must reproduce the goldens as they stood before Movement 2b (journal/0110) —
/// the scalar-load solve, which is a second path and not a deleted one.
///
/// Asserted in
/// `tests/material_transport.rs::material_transport_off_is_the_pre_slice_world_and_on_moves_it`,
/// deliberately not here, for the same reason as the single-receiver pair.
///
/// Note the record half is **not** the pre-2b `GOLDEN_RECORD` constant, and that
/// is not a world change: `record_fingerprint` gained the unit's `species` byte
/// in the same commit, so every record hash in this file was re-derived. With the
/// flag off the species is `litho_of_tag(tag)` at every unit, so the *record* is
/// byte-identical and only the *hash* moved.
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0x6F83_4D53_DB89_8C36`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0xC013_3553_0033_93A5`.
// Re-captured 2026-08-03 (P11 slice 3): the scalar-load arm records through the
// same packed recorder (quantized thickness, mover axis), so the pre-2b fixed
// point moved once with the pack and holds again.
pub const GOLDEN_SURFACE_SCALAR_LOAD: u64 = 0x1CB0_C1BC_66BF_0EB7;
/// The strata-record half of [`GOLDEN_SURFACE_SCALAR_LOAD`].
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the rock (case 1). Prior value,
/// kept for audit: `0x3940_3AD9_C3A8_FD83`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0x98B6_DAD5_5796_8D6A`.
pub const GOLDEN_RECORD_SCALAR_LOAD: u64 = 0xE4F1_395B_565D_EFDF;

/// **The fluvial-only fixed point, still reachable.** The same fixture world with
/// [`DeepConfig::material_creep`](dc_worldgen::deeptime::DeepConfig) **off** must
/// reproduce the goldens as they stood after Movement 2b's first slice
/// (journal/0110) and before its continuation (b) (journal/0112) — the world where
/// the *rivers* carried identity and the hillslopes did not.
///
/// This is the third member of the same family as
/// [`GOLDEN_SURFACE_SINGLE_RECEIVER`] and [`GOLDEN_SURFACE_SCALAR_LOAD`]: an old
/// solve kept as a second *path*, not a deleted one, and proven byte-identical
/// rather than assumed to be. Asserted in
/// `tests/material_creep.rs::material_creep_off_is_the_fluvial_only_world_and_on_moves_it`,
/// deliberately not here, for the same reason as its two siblings.
///
/// **⚠ NOT the pre-slice constants, and the difference is the honest part.** The
/// commit that added this pair also carries **corrections #57**: `Litho::as_deposited`
/// used to say the only lithology a deposit cannot be is basement, and peat, coal and
/// charcoal cannot be either — they are made *in place*, so a mover that picks one up
/// is carrying detrital organic matter (`material-behavior.md` § 12's four-way test:
/// a moved material is category 3, an in-place organic is 1/2). That fix applies to
/// **any** mover, so it moves the fluvial-only world too, and this constant is
/// therefore *"the fluvial-only world **plus** the organic-deposit correction"*.
///
/// **That the value moved at all is itself the evidence #57 was needed**: it means
/// the fluvial pass really had been filing transported organics as in-place seams,
/// rarely enough that no test could see it until creep multiplied the traffic by 918.
/// Prior value (post-2b, before #57), kept for audit:
///
/// ```text
/// GOLDEN_SURFACE_ANONYMOUS_CREEP 0x60F0_A669_F4B9_23BD
/// GOLDEN_RECORD_ANONYMOUS_CREEP  0x9DEE_8FAE_4550_F2D0
/// ```
///
/// The **scalar-load** pair above is unmoved by #57 and still asserted, which pins
/// the fix's own off-switch: with `material_transport` off nothing is ever carried,
/// so there is no carried winner for `as_deposited` to answer about.
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** Rounding, not rule (case 2). Prior value,
/// kept for audit: `0xDAB0_34AC_9984_209C`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0xFAC2_2A81_1311_2075`.
// Re-captured 2026-08-03 (P11 slice 3): same mechanism as SCALAR_LOAD above.
pub const GOLDEN_SURFACE_ANONYMOUS_CREEP: u64 = 0xA706_5161_068F_061D;
/// The strata-record half of [`GOLDEN_SURFACE_ANONYMOUS_CREEP`].
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the
/// rock (case 1). Prior value, kept for audit: `0x447D_E3D0_7675_8D21`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 3, ROUNDING not rule: this arm's semantics are unchanged, but the world it is an arm OF moved (member-grade rates), and the sums it re-runs are the sums of a different record. Prior value, kept for audit: `0xE77F_385F_D58E_F020`.
pub const GOLDEN_RECORD_ANONYMOUS_CREEP: u64 = 0x5DEE_9CC7_8C0C_8AF3;
/// The strata-record half of [`GOLDEN_SURFACE_SINGLE_RECEIVER`].
///
/// **Re-derived 2026-07-26 (journal/0110), and the record did NOT move.**
/// `record_fingerprint` gained the unit's `species` byte, so every record hash in
/// this file changed; the single-receiver *record* is byte-identical to pre-MFD
/// main as it always was (its surface half, which the new axis cannot reach, is
/// untouched and still asserted against the original value — that pairing is the
/// evidence). Prior value (before the species byte), kept for audit:
///
/// ```text
/// GOLDEN_RECORD_SINGLE_RECEIVER 0x4A20_745B_3879_7C8A
/// ```
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the
/// rock (case 1). Prior value, kept for audit: `0xAB2E_0CA4_2412_05C1`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 above.** Case 3, ROUNDING not rule: the single-receiver differential claim is untouched, but the world it is an arm OF now has member-grade rates. Prior value, kept for audit: `0x5ECD_3AC2_7468_5D5C`.
pub const GOLDEN_RECORD_SINGLE_RECEIVER: u64 = 0x0571_3759_07DA_44BB;
/// FNV-1a-64 over the strata record of the same field.
///
/// **Moved 2026-07-24 by the geotherm (journal/0093) — authorized.** The first
/// §5 field pass retired the degenerate `burial_temp_c` provider and recalibrated
/// coalification onto a real geotherm temperature (`COAL_ONSET_C` 8 → 22 °C), so
/// the record's Coal/Peat tags moved (coal follows warm crust now, ~60 % of the
/// Small world's peat candidates vs the old 8 m rule's ~12 %). This is an
/// intentional world change, not a byte-identity regression — which is why only
/// the *record* hash moved and [`GOLDEN_SURFACE`] held. Prior value (journal/0072
/// share-weighted blend), kept for audit:
///
/// ```text
/// GOLDEN_RECORD 0xC9C6_D6F6_E908_9653
/// ```
///
/// **Moved 2026-07-25 by MFD routing (journal/0109) — authorized.** See
/// [`GOLDEN_SURFACE`]. Prior value (pre-MFD `main`), kept for audit and still
/// asserted under `mfd: false` as [`GOLDEN_RECORD_SINGLE_RECEIVER`]:
///
/// ```text
/// GOLDEN_RECORD 0x4A20_745B_3879_7C8A
/// ```
/// **Moved 2026-07-26 by material-aware transport (journal/0110) — authorized,
/// and this hash moved for TWO reasons at once, which is worth stating rather
/// than blurring.** (1) The record gained an axis: `DepUnit::species`, the
/// material that actually arrived, which this fingerprint now hashes — so even a
/// byte-identical record hashes differently than it did yesterday. (2) The world
/// itself moved, because the competence ceiling changes where mass is set down.
/// The two are separated by [`GOLDEN_RECORD_SCALAR_LOAD`]: that constant is the
/// pre-2b *record* under the post-2b *hash*, so the difference between it and the
/// old value below is purely the new axis, and the difference between it and this
/// one is purely the physics. Prior value (pre-2b `main`, without the species
/// byte), kept for audit:
///
/// ```text
/// GOLDEN_RECORD 0x6CEB_947D_6207_3A1E
/// ```
/// **Moved 2026-07-26 by material-aware hillslope creep (journal/0112) —
/// authorized.** See [`GOLDEN_SURFACE`]. The record's *composition* moved further
/// than any prior slice has moved it: 91 % fine clastic → 26 % fine / 36 % coarse /
/// 38 % carbonaceous soil, because a hillslope no longer records "mud, because this
/// is a quiet place" but whatever crept down onto it. Unit count +21.5 %, all merge
/// key. The same commit carries **corrections #57**, which is a separate
/// correctness fix and moves the fluvial path too — see
/// [`GOLDEN_RECORD_ANONYMOUS_CREEP`], which is where the two are separated. Prior
/// value (post-2b, pre-2b(b)), kept for audit:
///
/// ```text
/// GOLDEN_RECORD 0x9DEE_8FAE_4550_F2D0
/// ```
/// **Moved 2026-07-26 by hybrid `p` (journal/0113) — authorized.** See
/// [`GOLDEN_SURFACE`]. Prior value (pre-hybrid `main`), kept for audit:
///
/// ```text
/// GOLDEN_RECORD 0x16EC_7D94_3A2E_D912
/// ```
/// **NOT moved by journal/0114** — see [`GOLDEN_SURFACE`]. The calibrated record is
/// [`GOLDEN_RECORD_CALIBRATED`].
///
/// **Moved 2026-07-29 by the sub-cycled hillslope operator (journal/0122) —
/// authorized.** See [`GOLDEN_SURFACE`]. Prior value (pre-0122 `main`), kept for
/// audit and still asserted under `creep_substep: false` as
/// [`GOLDEN_RECORD_UNBOUNDED_CREEP`]:
///
/// ```text
/// GOLDEN_RECORD 0xACB6_1859_6AA3_F3A8
/// ```
///
/// **Moved 2026-08-01 by P11 slice 1 — see § P11 above.** The record now names the rock (case 1). Prior value,
/// kept for audit: `0x820B_A198_49DD_234A`.
/// **Moved 2026-08-02 by P11 slice 2 (the conversion) — see § P11 SLICE 2 below.** Case 1, the RULE: the erosion rate is now a function of the ROCK, not of its class, so a window holding mudstone and siltstone blends two multipliers where it blended one — and transported deposits take their identity from the load rather than from a draw. Case 3 rides along: the CSR row visits the species a cell actually holds, so the partial sums of a budget land differently from a dense row padded with zeros. Prior value, kept for audit: `0x6739_19DA_BBA4_EA86`.
// Re-captured 2026-08-03 (P11 slice 3): see GOLDEN_SURFACE's note — the record
// itself is the quantized artifact (8 B packed units; mover axis in the key,
// measured 1.0308x before joining).
pub const GOLDEN_RECORD: u64 = 0x84E4_2349_B872_6684;

// ---------------------------------------------------------------------------
// A deterministic fingerprint (FNV-1a 64), written by hand so it depends on
// nothing but the bytes: no `#[derive(Hash)]` discriminant encoding, no
// `DefaultHasher` (whose output is explicitly not stable across releases).

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.byte(b);
        }
    }
    fn f64(&mut self, v: f64) {
        // `to_bits` — a bit-for-bit read, so a 1-ULP drift cannot hide.
        self.u64(v.to_bits());
    }
    fn i32(&mut self, v: i32) {
        self.u64(v as u32 as u64);
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
}

// Explicit enum codes rather than derived `Hash`: the derived encoding is a
// rustc implementation detail, and a golden that silently changes with the
// toolchain is not a golden.
fn env_code(e: DepEnv) -> u8 {
    match e {
        DepEnv::Subaerial => 0,
        DepEnv::Subsea => 1,
    }
}
fn aridity_code(a: Aridity) -> u8 {
    match a {
        Aridity::Arid => 0,
        Aridity::Humid => 1,
    }
}
fn energy_code(e: EnergyBand) -> u8 {
    match e {
        EnergyBand::Low => 0,
        EnergyBand::Medium => 1,
        EnergyBand::High => 2,
    }
}
fn biota_code(b: Biofacies) -> u8 {
    match b {
        Biofacies::Mineral => 0,
        Biofacies::Soil => 1,
        Biofacies::Peat => 2,
        Biofacies::Coal => 3,
        Biofacies::Charcoal => 4,
        Biofacies::Retro => 5,
    }
}
/// The unit's **material** (Movement 2b) — an axis the record gained on
/// 2026-07-26 and that the fingerprint must be able to see. With material-aware
/// transport off it is a pure function of the tag, so it adds no information;
/// with it on it is the whole point, and a fingerprint blind to it would let the
/// slice move every rock in the world without moving a hash.
///
/// **P11 slice 1 widened it to the registry byte.** It used to hash a 7-value
/// *class* code, which the record no longer carries; hashing the class of a
/// recorded `MaterialId` would have kept the fingerprint blind to exactly the
/// distinction the slice creates — mudstone and siltstone are one class — and the
/// warning above would have come true in the same commit that wrote it. The raw
/// `MaterialId` is a stable compile-time ordinal (registry-id order), so this is
/// the same kind of value the code was, one byte wide, over a wider alphabet.
fn species_code(m: MaterialId) -> u8 {
    m.raw()
}
fn eolian_code(e: Eolian) -> u8 {
    match e {
        Eolian::None => 0,
        Eolian::Loess => 1,
        Eolian::Dune => 2,
    }
}

/// Every plane the world *keeps* except the record: elevation, regolith, the
/// drainage export, exhumation/crust, and the shape metadata.
pub fn surface_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.w);
    h.usize(f.wp);
    h.f64(f.cell_m);
    h.usize(f.surf.len());
    for v in &f.surf {
        h.f64(*v);
    }
    h.usize(f.regolith.len());
    for v in &f.regolith {
        h.f64(*v);
    }
    h.usize(f.recv.len());
    for v in &f.recv {
        h.i32(*v);
    }
    for v in &f.area {
        h.f64(*v);
    }
    for v in &f.lake {
        h.byte(u8::from(*v));
    }
    h.usize(f.exhum.len());
    for v in &f.exhum {
        h.f64(*v);
    }
    for v in &f.t_crust {
        h.f64(*v);
    }
    h.usize(f.chapters.len());
    h.0
}

/// The tagged deposition log, unit by unit: every tag axis, thickness bits,
/// unconformity flag, chapter, **epoch**, and the per-cell strip count.
///
/// **The epoch byte joined 2026-08-04 (the deposition clock, O-2b ruled —
/// § 3 branch B of `docs/audits/2026-08-04-deposition-clock-design.md`).** The
/// fingerprint hashes by accessor, so a new axis is invisible until a line is
/// added — and an axis without a tripwire is the `flow_cost_probe` shape (an
/// axis the gate structurally cannot see), which this repo has been bitten by
/// twice (CLAUDE.md § Gates). Adding it re-captured exactly the six
/// `GOLDEN_RECORD*` families once; every SURFACE/GEOTHERM/FLUX/HEAD/CONTENTS/
/// far golden is byte-still because the epoch is write-only to the sim —
/// nothing in the erosion pipeline reads it back.
pub fn record_fingerprint(f: &DeepField) -> u64 {
    let mut h = Fnv::new();
    h.usize(f.strata.len());
    for s in &f.strata {
        h.usize(s.units.len());
        h.u64(u64::from(s.strips));
        for u in &s.units {
            h.byte(env_code(u.tag().env));
            h.byte(aridity_code(u.tag().aridity));
            h.byte(energy_code(u.tag().energy));
            h.byte(biota_code(u.tag().biota));
            h.byte(eolian_code(u.tag().eolian));
            h.f64(u.thickness_m());
            h.byte(u8::from(u.unconformity()));
            h.byte(u.chapter());
            h.byte(u.epoch());
            h.byte(species_code(u.species()));
        }
    }
    h.0
}

/// The **golden fixture** world at [`SEED`], `Extent::Small`, built under the
/// production *config* — the world the goldens describe.
///
/// **NOT the world `dc-client` boots** (that is seed `1337` at `Extent::Medium`;
/// see `tests/geotherm.rs`). The distinction is load-bearing: **corrections #51**
/// records a coal magnitude claim that went unfalsified for a day because a helper
/// named `production_field` was mistaken for the shipped world. What lives here is
/// legitimately seed-independent — byte-identity
/// and derived-vs-scalar agreement hold on *any* fixed world — so this fixture is
/// sound for the goldens and **must not be used to accept a magnitude, a count, or
/// any claim about what a player will find**.
///
/// It was called `production_field` until 2026-07-26; the name is what misled the
/// audit that missed the `organic.rs` defect, so it now says what it is.
pub fn golden_field() -> DeepField {
    let pregen = Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    });
    build_field(&pregen.grid, SEED)
}

/// The pregen grid the deep run is built from, for suites that need to swap a
/// provider into `production_config` rather than take the default world.
///
/// Same fixture world as [`golden_field`] — see its doc comment for what this
/// world is and, more importantly, what it is not.
pub fn golden_pregen() -> Pregen {
    Pregen::run(WorldParams {
        seed: SEED,
        extent: Extent::Small,
    })
}
