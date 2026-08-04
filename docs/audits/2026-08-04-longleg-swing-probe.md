# The longleg mid-swing knee wobble — a diagnostic probe

**2026-08-04. WIP — measurements not yet taken.** Status: harness being built.

## The observation (user, 2026-08-04 walk, station 2, `--anim-fps 60`)

> *"the longer legged one has a little weirdness in the forward swing of walking leg
> which i dont see in the stout — and it may have to do with IK adjusting the leg
> mid-swing. what it looks like is the leg doing a **mini bend and correction back to
> straight and back to bent** during the swing forward. very hard to see in realtime…
> it's non blocking at any rate."*

Conditions: `dc:body/longleg` and `dc:body/stout`, `speed: 0.3` × `walk_speed_m_s = 4.5`
= **1.35 m/s**, level stone, `--anim-fps 60`.

## Candidates under test

1. The half-voxel override window (`character.rs:313`).
2. B7's `d_min` sector boundary (`body.rs:884`).
3. Per-cycle quantization (`anim_rate.rs`, journal/0155).
4. A fourth thing — `swing_gain` (stubs #43), the clearance blend, or the
   gait/IK seam.

## Findings

*(pending)*
