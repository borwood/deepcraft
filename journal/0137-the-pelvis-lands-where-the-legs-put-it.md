# The pelvis lands where the legs put it

*2026-08-02 · posture-gait member #0, slice one · dc-api (+ one hoist in dc-client)*

The bones' § 1 named the inversion: today the pelvis is pinned at an authored hip height
and the legs are asked to reach the floor; reality pins the feet. This slice builds the
un-inversion, headless: `bake_resting_posture(plan, mode)` in dc-api — pure, deterministic,
callable from any clock (corrections #86: pack build, deeptime worldgen when evolution
mints a species, define time — the venue is the caller's, the purity is the contract).

It reads only declarations: the mode's bearing roles, the sole anchors B0 landed, segment
volume as the mass proxy. Stance chains derive by the parent-walk to the first branch
point — **hoisted to dc-api and shared with `leg_rigs`** rather than existing twice (the
design pass flagged its own build's characteristic-failure risk, F6, and the build obeyed).
The solve is honest static geometry: each stance chain takes the zero-torque vertical
column under its hip, the root lands at chain reach, and balance — CoM over the support
hull from bottom-face patches (ruling Q3) — is verified, never optimised.

## The numbers, measured against the design pass's prediction

| | authored hip | derived hip | resting knee |
|---|---|---|---|
| biped | 0.900 m | **0.880 m** | 0.0° |
| stout | 0.460 m | **0.440 m** | 0.0° |
| longleg | 0.900 m | **1.020 m** | 0.0° |

Exactly the predicted table, to 1e-9. Longleg — the body that squats at −56.2° under the
pinned hip — stands **140 mm taller than the biped** the moment the feet pin the pelvis
instead. Both hand-authored plans carried the identical +0.020 m hip/reach gap: one
mistake, copied, never a design; the bake deletes the class. The biped's CoM lands at
0.545 of stature against a published human band of 0.55–0.57 — in-band, from cuboids.

`straight_legs_are_the_answer` is a real test with the reason in its comment: for a
symmetric standing biped the zero-torque column IS the correct rest, and the assertion is
there so no later session "fixes" it into bent knees out of suspicion of degeneracy. The
refusals are equally honest: a mode-less plan is a legal absence (a tree), a distributed
bearing set is refused *naming the geometry* (a snake's line, per the standing-bodies
scoping), unequal chains wait for the first quadruped (A-1 ledger in the doc comments).

## What it took to land

Three mechanisms from this repo's own scar tissue fired in one evening. The build-slot
hook queued this slice's gates behind the sibling session's P11 build — refereeing, not
memo-ing. The builder agent died waiting for the slot and the harvest happened by hand
from its worktree, verified from the tree rather than from anyone's narration (the session
had just filed corrections #87 for narrating a dispatch that never happened; the lesson
was fresh). And the first test run produced the **impossible red** — the shared target dir
served a stale pre-B0 `dc-api` rlib, the compiler insisting `BodyPlan` still had `slots` —
resolved by exactly the `cargo clean -p` that CLAUDE.md § Gates prescribes for its
mirror-image false green.

Nothing changes on screen. The continuation slice hands the derived hip to the renderer —
that is the one where three bodies stand at three different heights and the feet finally
plant.

> blogworthy: the design pass predicted the build's numbers to nine decimals before a line
> was written, because the solve is closed-form over declared data — lens 3 (the right
> primitive makes the answer a derivation, not a tune), with a lens-1 thread on harvesting
> a dead agent's worktree under fresh fabrication discipline.
