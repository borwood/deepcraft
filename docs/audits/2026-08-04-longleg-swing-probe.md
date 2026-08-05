# The longleg mid-swing knee wobble — a diagnostic probe

**2026-08-04. WIP — harness written, measurement pending the build slot.**
Diagnostic only: no fix, no design, no ruling.

## The observation (user, 2026-08-04 walk, station 2, `--anim-fps 60`)

> *"the longer legged one has a little weirdness in the forward swing of walking leg
> which i dont see in the stout — and it may have to do with IK adjusting the leg
> mid-swing. what it looks like is the leg doing a **mini bend and correction back to
> straight and back to bent** during the swing forward. very hard to see in realtime,
> i would have to look at slow motion to really catch what's going on. it's non
> blocking at any rate."*

Conditions: `dc:body/longleg` and `dc:body/stout`, `speed: 0.3` × `walk_speed_m_s = 4.5`
= **1.35 m/s**, level stone, `--anim-fps 60`.

## The instrument

`crates/dc-client/src/swing_probe.rs` (a `#[cfg(test)]` module, wired at
`crates/dc-client/src/main.rs:67` — `dc-client` is a **binary** crate with no lib
target, so the `examples/ … test = true` shape in CLAUDE.md § Gates cannot reach
`body.rs`'s solver at all; this is the same *"prints from the test under
`--nocapture`"* answer `body::retarget_report` already uses).

It drives the production pose path end to end —
`body::pose_for`, `body::root_offset_m`, `body::fk_foot_local`,
`body::solve_leg_ik`, `body::leg_rigs`, `body::derived_gait` — and mirrors exactly
two pieces that need a bevy world: `character.rs::ground_top_m` (level ground has
a closed form) and `character.rs`'s window/limit decision at `character.rs:309-337`.

Two traces per body, and the separation is the whole discrimination:

- **continuous** — `GaitVector::limb_pose` at the raw phase: the underlying geometry.
- **rendered** — `pose_for` + `AnimState::stepped_phase`: the per-cycle quantized
  staircase the eye actually sees (journal/0155).

The jump statistic is the largest change between adjacent samples. For a
C0-continuous signal that is `|f′|·Δphase`, so **quadrupling the sample count
divides it by ≈ 4**; a genuine jump is invariant under refinement. That ratio is
the scale-free invariant the gate asserts —
`the_continuous_swing_geometry_refines`.

## Candidates under test

1. The half-voxel override window (`character.rs:313`).
2. B7's `d_min` sector boundary (`body.rs:884`).
3. Per-cycle quantization (`anim_rate.rs`, journal/0155).
4. A fourth thing — `swing_gain` (stubs #43), the clearance blend, or the
   gait/IK seam.

## Findings

*(pending — the build slot has been held by another session's
`cargo test --release -p dc-worldgen` since 19:21)*
