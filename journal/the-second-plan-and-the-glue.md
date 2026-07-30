# The second plan, and what the glue actually did

`docs/design/bodies.md` has said this since 2026-07-19, marked DECIDED:

> Two-bone IK for limbs + neck look-at is the **retargeting glue** that makes one
> clip serve every mutation of a plan: feet to actual ground, hands to actual
> socket transforms, **across differing proportions**.

We built it. We wrote the two-bone solver, the neck look-at split, the
half-voxel foot-placement window, and eleven unit tests around them. And then we
never tested the claim, because there was exactly one body plan. `character.rs`
said so out loud, in a comment nobody had reason to distrust:

> v0: every character wears the one vanilla plan

"Across differing proportions" is a claim about a set with two elements in it. We
had a set with one. Every green test was a test of the solver's *arithmetic*, and
none of them was a test of the *claim*. This entry is about building the second
element and looking.

<!-- MEASUREMENTS-GO-HERE -->

## Before the second plan: a door we had built and never walked through

The first thing in the way was not the geometry. It was that the renderer did not
read body plans from anywhere — it *called* them.

```rust
// character.rs, before
let plan = biped_plan();
let clips = biped_clips();
```

Meanwhile `dc-api/src/bodies.rs` had this, with a docstring explaining the
principle it existed to serve:

```rust
/// The vanilla body content as a recorded command batch … Content packs are just
/// registry command batches; this is the bodies pack.
pub fn vanilla_body_pack() -> Vec<Payload> { … }
```

`vanilla_body_pack` was referenced from exactly one place in the tree:
`crates/dc-api/tests/bodies.rs`. The *running game* never touched it. So we had
built a door, written on it "vanilla is the first pack", proved with a test that
the door opens — and then walked around the side of the building. That is
`spines.md` § A-4, machinery that exists and nothing calls, and it is the
project's characteristic failure mode with a name.

So step one was not "add a plan". Step one was **make the existing plan arrive
the way a second plan would have to arrive**: submit `vanilla_body_pack()` as a
command batch at world construction under a `registry.define(dc)` grant, and have
the renderer read `HostWorld::body_plan` / `anim_clip`. Nothing about the frame
changes; everything about the *path* does. It also happens to satisfy the north
star's line that "first-party content ships through the same SDK, not a
privileged internal path" — which we had been asserting while shipping the
privileged internal path.

Proving a re-housing is a re-housing was the interesting part. A screenshot diff
would have been weaker than what is actually available: every line of
`character.rs` and `body.rs` downstream of the plan is a pure function of the
plan's segments and the clips' keyframes. So if the registry hands back a value
`==` to what the compiled-in call handed back, the rendered frame is identical
*by construction*, and the test is a structural equality rather than a
perceptual one:

```rust
assert_eq!(
    world.body_plan("dc:body/biped").unwrap().plan,
    biped_plan(),
    "the plan the renderer reads must equal the one it used to call"
);
```

One thing did move, and it is worth naming because it is the kind of thing that
gets discovered six months later: the pack is **submitted** at construction and
**applies at the first tick**, like every other command. There is a window of a
few frames at boot where no body plan is registered. It costs nothing — no
character can exist before a tick either, since spawning one is also a command —
but the renderer now has to tolerate "the registry cannot serve this plan yet"
rather than `expect()`-ing its way through. That tolerance is also the honest
report path if a pack genuinely fails to load.

## The second plan: how to pick numbers that are not tasteful

The instruction I gave myself was to make retargeting *work hard*, and
specifically not to make the second body look good. That is harder than it
sounds, because the instinct when authoring a body is to author a *nice* body.

`dc:body/stout` shares the biped's eleven joint names — that is forced, because
sharing the joint names is what lets `validate_plan` accept the biped's clips
unmodified, and the clips are the instrument. If I had authored stout-specific
clips I would have destroyed the measurement. So the plan binds
`dc:anim/biped_idle`, `dc:anim/biped_walk`, `dc:anim/biped_jump`, verbatim, and
only the geometry differs:

| measure                  | `dc:body/biped` | `dc:body/stout` | ratio |
|--------------------------|-----------------|-----------------|-------|
| leg (hip→sole)           | 0.88 m          | 0.44 m          | 0.50× |
| arm (shoulder→fingertip) | 0.58 m          | 0.93 m          | 1.60× |
| hip height               | 0.90 m          | 0.46 m          | 0.51× |
| trunk width × depth      | 0.50 × 0.28 m   | 0.78 × 0.50 m   | 1.56× / 1.79× |
| head edge                | 0.28 m          | 0.44 m          | 1.57× |
| standing height          | 1.80 m          | 1.60 m          | 0.89× |

I kept the **tints identical**. That was deliberate and it took an argument with
myself: a side-by-side frame is far easier to read if the two bodies are
different colours. But a colour difference is an art choice, and the moment the
second plan carries one, the frame is partly about palette. With one palette, the
only thing a viewer can see is proportion — which is the question.

The hanging arms end 0.08 m above the ground. It is a knuckle-dragger. It looks
absurd. That is the correct outcome.

## What the glue actually did

<!-- FINDINGS-GO-HERE -->

## The measurement that had to be built to see any of this

None of the above is visible from a screenshot, and none of it was visible from
the existing tests, which check the solver against its own forward kinematics.
What was missing was an instrument that asks the *composed* question: given this
plan and this clip, where does the foot end up, on flat ground, after the
renderer's own decision procedure has run?

That procedure is: sample the clip on the 12 fps grid → forward-kinematic the
leg from the sampled hip/knee angles → find the ground → if the foot is off the
ground by more than 1 mm and less than half a voxel, solve two-bone IK for the
corrected target → snap the solved angles to the 11.25° rotation quantum. Five
steps, three of which are quantizations or clamps, and the *residual after all
five* is the only number that describes what a player sees.

`retarget_report` in `body.rs` re-derives that procedure over flat ground and
reports it per (plan, clip). It carries one honest cost that I want on the
record: it is a **second implementation** of the renderer's placement decision,
because the real one needs a bevy world and a voxel query. It can silently
disagree with production. Everything it *shares* with production — `sample_clip`,
`solve_leg_ik`, `fk_foot_local`, `leg_rigs`, `stepped_angle` — is the real
function, and moving `LegRig` / `leg_rigs` / `fk_foot_local` out of the bevy glue
into the pure module was part of the price. What is duplicated is the five-step
*ordering*. That is a loose end, not a fudge, and it is listed as one.

The assertions the gate carries are deliberately not the numbers. A test pinned
to "the stout's foot floats 0.074 m" fails the first time somebody re-authors the
walk clip for a good reason, which is worse than the defect it guards. So the
gate asserts: nothing is ever non-finite; the plan-derived rig tracks the plan's
proportions (stout reach within 0.45–0.55× the biped's, or the experiment is not
measuring what it says); and where the IK ran and could reach, the residual is
bounded by `reach × ROT_QUANTUM_RAD` — one quantum's arc at full extension, a
*derived* bound rather than a fitted one. The numbers themselves live in the
printed report and in this entry, where a human reads them.

> blogworthy: **lens 1 (AI-native development) and lens 3 (reflexions in a
> deepsim codebase).** A DECIDED design claim sat green and untested for ten days
> because the claim quantified over a set the codebase could only populate with
> one element — and the code that would have exposed it was a `pub fn` with one
> test caller and a docstring explaining the principle it served. The lens-1
> angle: a ratified sentence is a hypothesis with a date on it, and "all the
> tests pass" is not evidence about a claim whose universe has one member. The
> lens-3 angle: the failures the second plan found are all in the same shape —
> a *scale-free* authored quantity (an angle) meeting an *absolute* engine
> constant (a metre) with nothing reconciling them, and that seam is invisible
> until two bodies of different size stand on it.
