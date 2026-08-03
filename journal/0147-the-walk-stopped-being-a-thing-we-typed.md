# 0147 — the walk stopped being a thing we typed

*2026-08-03 — gait member #1, slice two: the consumer*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — the difference between
> *fixing* a two-authority bug and *deleting the second authority*, and what it costs to
> tell them apart. Also **lens 2 (procgen against the backdrop of priors)**: a hand-typed
> 1.000-second walk cycle, played at every speed on every body, replaced by four published
> regressions and no keyframes at all.

Slice one put `bake_gait` in dc-api and nothing called it. The predicted table came out to
1e-5 and the bodies kept playing `dc:anim/biped_walk` — a one-second loop, hand-typed in
2026-07, four poses, identical for a 0.44 m stout and a 1.02 m longleg, running at the same
rate whether the body was strolling or sprinting. This is the slice where the renderer stops
reading that clip and starts reading the derivation. Five things had to land together, and the
reason they had to land *together* is the whole story.

## The thing that was actually wrong

`AnimState::advance` did this:

```rust
self.clock_s += dt;
let desired = if speed_m_s > WALK_SPEED_THRESHOLD_M_S { Loco::Walk } else { Loco::Idle };
```

Two defects sitting on top of each other. The obvious one is the threshold: an analog
movement intent — `SetMoveIntent.speed`, a `[0,1]` fraction that has ridden the command log,
receipted and replayed, since the character primitive landed — collapsed into one bit. The
user named that himself when he ruled the gait pass: *"when we have controller support an
analog stick can actually grade intent up the ladder."* `stubs.md` #42 was written that
afternoon and is discharged by this commit.

The less obvious one is the first line. `clock_s += dt` has **no speed term at all**. The clip
advanced at one cycle per second forever, so the feet contributed 1.84 m/s of stride while the
mover carried the body at 4.5 m/s, and **2.66 m/s was skate** — a body ice-skating across the
terrain with its legs politely keeping their own time. Nobody had ever written that number
down until the gait design pass decomposed the clip against its own geometry.

Both die to the same replacement. The phase now advances by `cadence(Fr)·dt`, and cadence is
`v/λ`, so `dphase = ds/λ`: **the phase clock is a distance clock wearing a rate.** One limb
cycle per stride travelled, at any speed, on any body, by construction.

## Idle turned out not to need saying

The part I expected to fight was the bottom of the ladder. Removing a two-state machine
usually means replacing it with a smoother two-state machine — a blend weight, an ease curve,
*something* that knows the difference between standing and walking.

It needed none. `Fr = v²/(gL)`; stride is `2.3·L·Fr^0.3`; hip excursion is
`asin(step/2L)`. At `v = 0` the stride is zero, so the excursion is zero, so every keyframe
collapses onto the derived resting pose — and the cadence is zero too, so the phase clock
stops. A stopped body stands in its own zero-torque column and holds there. **Nothing had to
be special-cased to make a stopped body stand still**, and that is the tell that the switch had
never been expressing anything: it was compensating for a clip whose amplitude was a constant.

I wrote the test as the property rather than the absence of an identifier, because the
prohibition is *never re-introduce a discrete gait switch* and an identifier is easy to rename:
sweep 900 speeds from 0 to the shipped top speed on all three plans and fail on any per-step
joint jump over 0.05 rad. A threshold, a crossfade, or a gait-name lookup all necessarily put a
step in that sweep. The measured worst step is three orders below the bound.

## Where the derivation was actually wrong, and it was mine to catch

The design pass is a hypothesis. It was wrong here in a way that only shows up at the consumer.

`clearance` — the mid-swing keyframe — is derived from a published foot-lift margin (Winter
1992, ≈ 1.3 cm, ≈ 1.5 % of leg length). It is therefore **speed-invariant**, while every other
term in the gait goes to zero with speed. Taken literally, the traversal
`+contact → clearance → −contact` lifts a foot to full clearance at *any* speed, so a body that
stops mid-swing **freezes with one foot in the air**. Which falsifies, for exactly half the
cycle, the thing I had just finished writing three paragraphs about.

The fix had to introduce no new number, or it would be a knob pretending to be a mechanism. It
scales the lift by `sin θmax(Fr)` against its value at the duty law's *already published*
normal-walking anchor, `Fr = 0.25`. `sin θmax` is literally the step as a fraction of twice the
leg's reach, so the rule reads: **a foot that is barely advancing does not lift.** Above a
normal walk the published clearance is reproduced exactly — `the_three_keyframes_compose_the_cycle`
still asserts mid-swing *is* `clearance` to 1e-12 — and below it the swing shrinks continuously
into the resting pose.

It is a stand-in and it says so at the line. What a real animal does between walking and
standing is a **stop transition**, which the design puts in a clip and explicitly does not
design. When transitions land, this gain is deleted rather than re-tuned.

## The bob: deleting the term, not reconciling the expressions

Corrections #80 was two expressions of one vertical composition. `character.rs` built the IK
hip as `feet.y + hip_local[1] + root_delta − crouch` and the render root as
`t.y += root_delta; t.y += (root_bob − crouch)`. They drifted by exactly `root_bob`, which is
why a foot could hover in perfect time with the walk.

The falsified fix — an earlier session's — was to add the bob to the hip as well. That preserves
two expressions and asks them to agree, which is a thing you can only ask of code once. What
landed instead is that **`root_bob_m` left the schema** (user call #2: *"A, remove it while
it's still a recompile"*), `Pose` lost its vertical field, and there is now exactly one
function:

```rust
root_offset_m(gait, root_delta_m, froude, phase, posture) -> f64
```

Read once per body per frame, and read by *both* consumers from the same binding. There is no
second place to forget it. The three authored bobs died with it and each for its own reason:
the walk bob retired with its clip; the jump bob was a double authority with the mover, which
already owns a jumping body's vertical translation; the idle bob was a root translation where
breath belongs in joints — a resting animal's chest moves, not its whole body sliding up 15 mm.

The durable artifact is the test, and it is a source-text assertion, which I do not do lightly.
A value test cannot catch this class of defect: two expressions agree until somebody edits one.
So `the_render_root_and_the_ik_hip_read_one_root_offset` reads `character.rs` and asserts the
*shape* — exactly one composition site in the file; inside the per-frame pose loop
`root_delta_m` appears exactly once and `CROUCH_ROOT_DROP_M` not at all (the crouch is passed
as the posture it is, so there is no second place to sink a root); and both consumers read the
same binding by name. The property being protected is textual — that two expressions do not
exist — and no value check can express it.

And the vertical is now scale-free, which is the fourth of the arc's absolute-metre constants
to go: the authored 0.040 m bob was 4.5 % of a biped's leg and 9.1 % of a stout's, against a
derived 7.475 % for both.

## The facing, split

The fifth landing is the one that looks unrelated and is not. Trunk facing was client-side,
derived from velocity, guarded by the *same* `WALK_SPEED_THRESHOLD_M_S` — so retiring the
threshold meant deciding who owns facing at all.

The user's ruling on the gait pass's G7 widened into a standing pattern for the arc: **the sim
owns the target; the client owns the approach.** So `CharacterState.facing_yaw` is now derived
in `step_character` from travel, deterministic and replay-safe, and `AnimState::steer` chases
it over `TRUNK_TURN_WINDOW_S` and does nothing else. The speed test it used to carry did not
move — it *dissolved*: the sim simply stops moving the target when the body stops, and the
client's chase converges on a stationary target with no threshold anywhere.

Doing it now rather than later is cheap in the way these things stop being cheap: B4 buckets
colliders per yaw and B5 resolves damage against the nominal pose, so facing decides *which*
collider and *whether you were hit in the back*. Today it is a recompile; after B3 it is a wire
migration. It is also exactly where the walk-8 strafe lived — nobody owned it, so it inverted
on the path nobody built for.

One small casualty, and it is the point: `yaw_from_velocity` in the client is deleted, because
the same convention now lives once, in `dc_api::character::yaw_from_travel`, called by both the
gaze's follow-travel default and the facing target. Two copies of a facing convention in two
crates is the two-derivations-of-one-quantity failure the pattern forbids by name, and it was
sitting there while we wrote the rule.

## What the three bodies do now

At `Fr = 0.25` — a comfortable walk — the rendered gait carries dynamic similarity outright:
duty, hip excursion and root excursion *as a fraction of leg length* are identical to 1e-12
across biped, stout and longleg, and only the metres differ. Cadence goes as `√(g/L)` exactly,
so `f_stout/f_biped = √2` to the last bit. A big animal takes slow steps because of arithmetic
now, not because somebody typed a longer clip duration — and the stout, whose 0.44 m legs made
it the worst-served body under a shared one-second loop, is the one the change flatters most.

`dc:anim/biped_walk` is retired as shipped content and parked as a fixture. The user's ground
for that was recorded verbatim because it is narrower than the one the design pass argued:
*"mostly because it's not worth doing anything with."* Parked, not enshrined — so there is no
comparison harness and no "the derivation agrees with the animator to within X" acceptance test
around it. It stays in the tree because two tests already decompose it and deleting it would
cost more than leaving it.

## What this slice does not answer

Whether it *looks* right. Every ruling in the gait pass is a judgment about motion, a still
frame is structurally blind to a temporal artifact (corrections #77), and the derived root
excursion is about 65 % larger than the authored bob it replaces. The user's live view is the
senior instrument here and the walk is owed.
