# 0009 — the companion grows a skeleton

*2026-07-19 · body-plan staircase steps 1–2 (background agent; integrated +
walk-verified by the main session, merge `dd9b802`, 36 suites green).
Implements docs/design/bodies.md steps 1 and 2, ratified the same day.*

The companion has been two cuboids since journal/0005: an orange torso and a
pale head that slid through the world facing its heading, rigid as a chess
piece. bodies.md ratified the plan to do better — "above Minecraft, still
cuboid" — and staircased it: step 1 gives characters a jointed skeleton defined
as registry data; step 2 makes the skeleton *move*, stepped at 12 fps, blending
between standing and walking. This is that work, headless-and-gated; the
photographs come when the main session walks it.

## The plan is a class; the clips are its members — but the arrow points backward

Every registry so far has the same shape: declare a contract, then register
things that satisfy it, validated at join time (geology members against a
stratum class, journal/0007). Body plans want the same backbone, and mostly
get it — but the verb→slot contract points the *other way*, and that turned out
to be the one real design knot.

A body plan declares "I support the verbs idle, walk, jump," and the
schema-checked-at-define-time rule (bodies.md) says: **defining the plan must
fail if a required verb has no clip.** So the plan asserts a fact about clips.
But a clip is authored *for* a plan — its keyframes name that plan's joints.
Plan needs clips to exist; clips need the plan's joint names. A cycle.

The honest cut: **clips are standalone data.** A clip names joints but not a
plan. You define the clips first; then you define the plan, and *that* define
validates — every required verb bound, every bound clip present, and every
joint a clip animates actually declared by the plan. Joint-compatibility is
checked exactly when a plan adopts a clip, which is the same "validate the
member against the contract at join time" move geology makes — just with the
plan playing the member's role of "the thing that must fit." The define order
(clips, then plan) falls out of the contract's direction rather than being an
arbitrary rule, and `vanilla_body_pack()` emits it in exactly that order.

This kept the closed-`Payload`-union promise intact. Two new variants
(`DefineBodyPlan`, `DefineAnimClip`), two new command ids under
`dc:registry/`, appended at the enum's end (postcard order is wire identity);
`Effects` and `RejectReason` grew only appended fields. A new plan — even one a
plugin ships in its own namespace, joining the machinery from outside — needs
zero new Rust types. There's a test for exactly that (`mod:body/blob`, a
one-segment plugin plan).

> blogworthy: the chicken-and-egg of a contract that points at its own
> members, and why "standalone clips, plan validates on adoption" is the same
> join-time-validation move seen from the other end.

## Re-expressing the companion surfaced a coordinate discipline

Replacing two cuboids with an eleven-segment skeleton (trunk root, neck, head,
two upper/lower arms, two upper/lower legs) forced the pivot/offset split into
the open. A segment is a *joint* — a pivot the segment rotates about — with a
cuboid *hung off* it. If you conflate the two (put the box at the pivot), a
limb rotates about its own center and the arm swings from its midpoint like a
propeller. So `SegmentDef` carries both `pivot_m` (the joint's offset from its
parent's pivot) and `offset_m` (the cuboid's center relative to that pivot).
The shoulder pivots at 1.35 m up and 0.33 m out; its box is offset −0.15 m down
from there, so the arm hangs and swings from the shoulder. That one distinction
is what makes authored rotations read as anatomy instead of spin.

The Bevy hierarchy mirrors it directly: a body root entity (feet + yaw), a
joint entity per segment (local translation = pivot, animated rotation), and a
static mesh child per joint (local translation = offset). Because `to_render`
is a plain meters→units cast, body-local transforms are just meters. Spawning
wires an arbitrary tree by spawning every joint first, then stitching parents
with `add_child` — order-independent, so it doesn't care that the plan happens
to list parents before children.

## Stepped sampling shaped the whole player

The ratified aesthetic is stop-motion: ~12 fps, quantized rotations
(bodies.md). Building that *in from day one* rather than bolting it onto smooth
animation changed the sampler's spine. Three quantizations, all in the pure
`body.rs` core (no Bevy, so it's trivially testable):

1. **Time → 12 fps grid** before sampling, so a pose only changes twelve times
   a second no matter the frame rate.
2. **Angles → 32 steps per revolution** (11.25°) after interpolation, so joints
   click between orientations.
3. **Blend weight → 4 steps** across a short (0.18 s) crossfade window, so
   idle↔walk transitions read as stop-motion too, driven by the body's actual
   horizontal velocity (a legal one-way read of sim state — the firewall lets
   cosmetics read sim, never the reverse).

The determinism firewall made this cheap to trust: the sampler is a pure
function of `(clip, time)` and the player a pure function of a `(dt, speed)`
history, so "same input, same pose stream" is a unit test, not a worry — and
nothing here is a sim input, so replay bit-identity is untouched by
construction (the existing `character_session_replays_identically` proof passes
unchanged).

One small surprise: the **root bob** (the vertical bounce of a walk) is a
translation, not a rotation, so it escaped the angle quantization — and at the
loop-wrap boundary `rem_euclid` returns a phase that differs from the
un-wrapped phase by a floating-point ULP, so "one full loop later is the same
pose" failed on the bob alone by ~1e-17 m. The fix was also the aesthetic-right
one: quantize the bob to a 5 mm grid. Now it steps like everything else and the
loop repeats bit-for-bit.

## What the sim still sees: nothing new

The mover is untouched. A character is still one swept AABB derived from body
extents, stepped on the host tick (dc-api `character.rs`); per-segment
collision is not in scope, and no animation state is readable by sim or replay
code. The skeleton is a client-side view of the same authoritative pose the
two cuboids showed — just with more joints and a clock.

## Walk 8: the body reads — and hides one thing

Main session, merged main (`dd9b802`, 36 suites green), `--fullbright`,
rebuilt exe (timestamp checked). Character `dancer` attached on the surface
near the walk-7 quarry: the plan-driven biped stands where two cuboids used
to — head, neck, orange trunk, sleeved upper arms with skin-tone hands,
legs (`assets/0009-biped-idle.png`). Driven side-on, the walk cycle is
real: legs scissored mid-stride, arms counter-swinging, and the pose holds
between 12 fps steps so stills catch honest keyframes
(`assets/0009-biped-stride-a.png`, `-facing.png`).

**The walk's finding came from the user, watching live**: the body moved
one way while facing another — "appears to strafe." The code confirms the
mechanism: `SetMoveIntent` stores only `dx/dz/speed`; yaw changes solely
via `SetLook`; the renderer rotates the whole body by that look yaw. So
**nothing owns turning the trunk toward travel** — any controller that
doesn't call `set_look` walks its body sideways or backward. The user's
follow-up sighting ("latest run faced its travel — what changed?") was
answered by the second finding: with a direction-symmetric walk clip on a
**faceless head**, forward and backward walks are photographically
indistinguishable — the apparent fix was viewpoint coincidence. Orientation
is currently *unverifiable* from stills; bodies need a face cue (free with
textures, or a v0 asymmetry) before facing claims can be photographed at
all. Both filed to Observed; the trunk-follows-travel /
head-follows-look split belongs to staircase step 3 (the neck exists for
exactly this), with the v0 answer a design call: renderer-side
trunk-toward-velocity (cosmetic, firewall-legal) vs controller convention.

Also sighted live by the user, filed to Observed: the far-mesh S1 phantom —
the old hill-field renders ~500 m *below* the worldgen terrain in far LOD
rings until approach replaces it (the known TerrainGen fallback, now with
its full weirdness on display); and harsh material-family cutovers on chunk
lines — candidate mechanism: at N=2 a chunk (32 × 0.9 m = 28.8 m) exactly
matches the S7 column-quantization cell, so cell-stepped climate context
makes selection flip families precisely on chunk boundaries. Diagnosis
task filed; smoothing (per-column context interpolation and/or boundary
dither) looks tractable, not fundamental.

The jump fallback pose went unphotographed (facing ambiguity made the
session about orientation instead) — promotion to a real clip stays open.

> blogworthy: "the walk found what the tests couldn't" — define-time
> contracts, bit-identical samplers, 36 green suites, and the first live
> viewer immediately caught that nobody owns which way a body faces. Also
> a photography lesson: you cannot verify orientation on a faceless
> cuboid; observability is a *feature of the model*, not the camera.
