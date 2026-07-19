# 0014 — the body learns which way it faces

*DRAFT — background agent, body-plan staircase step 3 (docs/design/bodies.md
§ staircase). Headless-and-gated; the walk photographs come when the main
session drives it. Branch `worktree-agent-a5c80c02a8d71a902`.*

Step 3 was always going to close a wound the first live viewer opened. Walk 8
(journal/0009) found that **nobody owned which way a body faced**: `SetMoveIntent`
writes only `dx/dz/speed`, `SetLook` is the only thing that touches yaw, and the
renderer rotated the *whole* body by the look yaw — so a driver who never called
`set_look` walked its body sideways, and a driver who did could never look one
way while walking another. And it found that you couldn't even *photograph* the
fix, because a faceless cuboid head is orientation-blind on a direction-symmetric
walk clip. This entry is the retargeting glue (bodies.md's step-3 IK), the
trunk/look split, parametric crouch across the determinism firewall, and the one
pixel of face that makes all of it verifiable.

## The split: the trunk faces travel, the head faces the look

The neck existed since step 1 for exactly this. The fix is two derivations, both
cosmetic and both firewall-legal one-way reads of sim state:

- **Trunk yaw follows horizontal velocity.** `AnimState` gains a smoothed
  `trunk_yaw` that chases the travel heading (`atan2(-vx, -vz)`, the bevy yaw
  that makes `view_dir` point along the velocity) over a short turn window
  (`TRUNK_TURN_WINDOW_S`), and holds its last value when the body is stationary —
  a stopped body keeps facing where it walked, not snapping to some default. The
  stored yaw is smooth; the *rendered* yaw is quantized to the same 11.25° grid
  as every joint, so turns read stepped like the rest of the stop-motion.
- **The head/neck follow the look.** `resolve_orientation` splits the look off
  the travel-facing trunk: within a cervical clamp (±75° yaw, ±45° pitch) the
  neck alone turns; past the clamp the *trunk* turns to make up the difference so
  the neck only ever bends its maximum, and the head still ends up aimed exactly
  where the look points. Player-driven characters are unchanged in feel: a human
  who steers by look moves in the look direction, so the velocity-trunk lands on
  the look anyway — the split only becomes visible when travel and gaze diverge,
  which is precisely the walk-8 case.

The root entity now rotates by the (stepped) trunk yaw instead of the look yaw;
the neck joint composes the look rotation on top of its clip pose. A driven
character walks facing its travel while its head looks elsewhere.

## Feet that find the ground: closed-form two-bone IK

`solve_leg_ik` is a pure, closed-form two-bone solver (unit-tested in `body.rs`,
no bevy). Given a target foot position relative to the hip and the two bone
lengths, it solves the sagittal-plane triangle by law of cosines, picks the knee
pole that bends *forward* (of the two mirror solutions, the one with the smaller
z), and clamps the reach into the solvable annulus a hair off the singular ends
so it is **never NaN** and the knee never locks dead straight. The test that
earns trust is FK∘IK: forward-kinematics the solver's own output and assert the
foot lands back on the (clamped) target — no hand-derived sign cases, just the
inverse relation proven numerically.

The renderer wires it defensively (bodies.md: *the 12 fps aesthetic hides IK
artifacts by design*). Per leg, per frame: forward-kinematic the clip's current
foot, read the ground height under that foot's world column from the loaded
`ChunkMap` solidity (a one-way read of what the renderer already sees), and only
if the ground differs from the clip foot by **more than a millimetre and less
than half a voxel** re-solve the leg to plant the foot on the step. Flat ground
leaves the clip untouched (no pole-flip fighting the animation); a real terrain
step seats the foot; anything past the half-voxel cap lets the foot **float
honestly** rather than stretch the bones. The output is snapped to the stepped
grid so the smooth solver never leaks past the stop-motion look.

> blogworthy: proving an IK solver with FK∘IK instead of hand-derived angle
> cases — the inverse relation *is* the test, and sign bugs can't hide from it.

## Parametric crouch: the firewall split, verbatim

Crouch is bodies.md's canonical example of the determinism firewall, and it is
implemented as exactly two halves that never touch:

- **Sim side.** A fourth controller verb, `dc:character/set_posture`
  (`standing` | `crouching`), appended to the `Payload` union and the schema
  registry (so every MCP surface grows the tool for free). `CharacterState` gains
  a `serde(default)` `posture` field; crouching scales the swept-AABB collider to
  0.6× height at the tick boundary — an honest hunker that clears a two-thirds gap
  a standing body can't. It is discrete, deterministic sim state, so replay stays
  bit-identical (proven by a new posture-transition replay case). Standing up is
  **guarded**: a `crouching → standing` change is refused with
  `RejectReason::PostureBlocked` when the taller collider would embed in solid —
  reusing the very `aabb_overlaps_solid` test the attach embed guard uses — so a
  body under a low ceiling stays crouched instead of clipping up through it.
- **Cosmetic side.** No authored clip. The spine lowers (a root drop), and the
  feet-placement IK from above bends the knees to keep contact — a crouch pose
  that falls out of the same machinery. The head keeps its look through the neck
  split, so a crouched body still looks where it's told.

"Raycast up, clamp collider height" is sim; "IK the spine and head under the
ceiling" is renderer — the exact sentence from the ratified doc, now code.

## The one pixel that makes it real

Walk 8's hardest lesson was epistemic: *observability is a feature of the model,
not the camera* — you cannot verify orientation on a featureless head. So the
head grows a **v0 face cue**: a thin dark brow band parented to its front (−Z)
face. It is deliberately the smallest honest thing — a hardcoded dark quad,
placeholder until head textures — but it makes facing photographable, which is
the whole point: without it, this milestone is unverifiable.

## What the sim still sees: one bit more, and it's honest

The mover gained exactly one thing the sim reads: a discrete posture that scales
one collider dimension at the tick boundary. Everything else — trunk yaw, neck
look, leg IK, the crouch bend, the face — is client-side cosmetic, derived from
sim state the renderer is allowed to read and never written back. Replay
bit-identity is extended, not dented.

## Walk 11: the face cue earns its keep

Side-on, driven +Z with look set 90° toward the camera
(`assets/0014-trunk-travel-head-aside.png`): **trunk squarely in profile
facing its travel, mid-stride, head turned to the camera with the brow
band unambiguous** — the walk-8 orientation gap closed, and *provably*
closed, because for the first time a photograph can testify which way a
head points. The 0009 lesson ("observability is a feature of the model")
pays off in one frame.

Crouch under a built stone eave
(`assets/0014-crouch-under-ceiling.png`): trunk lowered, legs IK-folded,
head level under the slab. Two honest findings:
- **The stand-up guard never fired** — standing (1.8 m) fits *flush*
  under a 2-voxel (1.8 m) ceiling by the same EPS rule that makes resting
  on ground not-embedded. And since passages at N=2 come in 0.9 m steps,
  a 0.6× crouch (1.08 m) fits nothing standing doesn't: **crouch cannot
  currently earn any passage**. Factor ≤ 0.5 would flush-fit 1-voxel
  crawlspaces — but that's game feel (crouch-as-crawlspace vs
  crouch-as-stealth-pose), so it's filed for the user, not tuned here.
  The guard's rejection path stays proven by test only
  (`stand_up_is_blocked_under_a_low_ceiling` uses a sub-flush ceiling).
- The max-bend leg fold reads compressed/tangled — the knee pole and
  fold distribution want a tuning pass from photographs (agent flagged
  it; confirmed).

