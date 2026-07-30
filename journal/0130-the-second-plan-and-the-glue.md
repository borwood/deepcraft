# The second plan, and what the glue actually did

> **⚠ PARTLY FALSIFIED THE SAME EVENING — read `corrections.md` #78 before quoting this
> entry.** This entry concludes that foot-placement IK *"has never once engaged"* and that
> *"nobody has ever seen this rig bend a joint under IK."* **That is true of STANDING only.**
> The stock biped's IK has **always** solved while **crouching** (88/88, knee 123.7°),
> because `CROUCH_ROOT_DROP_M = 0.45` puts the hip at 0.450 m against a 0.880 m reach.
> The probe below swept clip × plan × foot exhaustively and held **posture** at its default,
> so its 176/176 is complete over the axes it had and silent about the one that mattered.
> journal/0131 has the full 3 plans × 4 cases × 3 clips matrix. **Everything below about
> the retargeting claim, the absolute-metre constants and the hover stands** — the constant
> count went from three to four, not down.

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

The short version, before the story: **the claim half-holds, and the half that
fails was failing for the biped too.** The angle-based machinery retargets
cleanly — a body with half the leg length and 1.6× the arm length wears the
unmodified clips and produces artifacts that are *smaller in metres*, not
different in kind. But "feet to actual ground" turns out never to have worked at
all. On flat ground, across every stepped frame of all three clips and both
plans — **176 of 176 samples** — the foot-placement IK is handed a target beyond
the leg's reach, clamps to full extension, and leaves the foot in the air. The
second plan did not break foot placement. The second plan is what made somebody
finally measure it.


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

### The wrong turn: a parity test that was measuring more than its claim

The gate then failed in a way I did not predict, and the failure was more
interesting than the fix. `edit_session_parity_direct_vs_mcp_tool_layer` runs a
scripted edit session twice — once as typed envelopes through the player path,
once as JSON through the MCP tool layer — and asserts the two receipt logs are
identical modulo the consumer's identity. It had been green for as long as it has
existed. It went red the moment the body pack entered the command log.

The receipts were not *wrong*; they were in a different order. In the direct run
the edits arrived seq 1–8 and the pack seq 9–13; in the MCP run the pack came
first, seq 1–5, edits 6–13. The cause is the priority-ordered command queue doing
exactly its job: `ConsumerKind::Player` has priority 0, `Plugin` and `McpSession`
both have priority 1. So the *player*'s edits sort ahead of the `vanilla-pack`
plugin, while the *MCP session*'s edits tie with it and lose on submission order.

That is correct behaviour, and the test's assertion was quietly broader than the
claim it was named for: it asserted "these two paths produce identical receipt
logs" when what it means is "these two paths produce identical receipts *for the
script*". Adding a third consumer at boot is what made the difference legible. The
fix is one line per run — tick once after construction, so the pack lands at
seq 1–5 in both worlds and the comparison is about the script again — and the
comment explaining *why* is longer than the fix, deliberately, because the next
person to add a boot-time command batch will hit this and should not have to
re-derive the priority rule from a diff of two 13-element receipt logs.

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

One more choice worth recording, because it is the kind of thing that goes wrong
quietly: `dc:body/stout` does **not** ride in `vanilla_body_pack()`. It has its own
batch, `experiment_body_pack()`, submitted after vanilla because it binds vanilla's
clips and authors none of its own. The reason is *existence is not standing*: an
unratified body sitting inside the default pack is precisely how bootstrap
fabrication becomes something a session finds in the tree six months later and
assumes belongs there. Deleting the experiment is three named things and one
`chain` call. A test asserts the experiment pack registers **zero clips**, which is
both its design and the reason it is deletable.

## What the glue actually did

Ground at y = 0, both bodies standing on it, N = 2 (0.900 m voxels, so the foot-IK
correction window is ±0.450 m), every stepped 12 fps frame of every clip, both
legs. Sole heights in metres; positive = floating.

```
  plan   clip  hip    reach | clip sole        rendered         | samples = seated+corrected+refused | beyond-reach
  biped  idle  0.900  0.880 | [+0.020,+0.035]  [+0.020,+0.035]  | 48 = 0 + 48 + 0                    | 48
  biped  walk  0.900  0.880 | [+0.054,+0.129]  [+0.037,+0.102]  | 24 = 0 + 24 + 0                    | 24
  biped  jump  0.900  0.880 | [+0.033,+0.129]  [+0.020,+0.125]  | 16 = 0 + 16 + 0                    | 16
  stout  idle  0.460  0.440 | [+0.020,+0.035]  [+0.020,+0.035]  | 48 = 0 + 48 + 0                    | 48
  stout  walk  0.460  0.440 | [+0.049,+0.074]  [+0.028,+0.068]  | 24 = 0 + 24 + 0                    | 24
  stout  jump  0.460  0.440 | [+0.029,+0.125]  [+0.020,+0.125]  | 16 = 0 + 16 + 0                    | 16

  reach ratio stout/biped = 0.500 (hips 0.460 / 0.900)
  IK window ±0.450 m = 1.02x the stout leg, 0.51x the biped leg
  authored walk root bob 0.040 m = 8.7% of the stout's hip height, 4.4% of the biped's
  authored jump root bob 0.120 m = 26.1% of the stout's hip height, 13.3% of the biped's
```

### The part that holds

The plan-derived rig is exactly `0.500×`. Nothing in the pipeline needed telling.
`leg_rigs` reads the bone lengths out of the plan, `sample_clip` produces angles
(which are dimensionless), `solve_leg_ik` is parameterized by `l1`/`l2` — and the
result is that a body half as tall in the legs walks with the same gait, with
absolute artifacts that *shrink*: the walk's rendered float peaks at 0.102 m on
the biped and 0.068 m on the stout. No new *kind* of artifact appeared anywhere.
The `idle` row is the cleanest possible statement of it: the two plans produce
**byte-identical sole heights**, `[+0.020, +0.035]`, on completely different
bodies.

That matters, because the cheap prediction was "the second plan will look
broken in a new way". It does not. It looks broken in the *same* way, less so in
metres. One authored clip set genuinely dressed two bodies with a 2× leg-length
difference, through the registry, with the verb→slot validator accepting it and
without a single new keyframe. As a mechanism, plan-generic animation works.

### The part that fails — and it was failing before there was a second plan

`beyond-reach` is **every sample of every row**. Not one frame, in any clip, on
either body, ever has its foot placed on the ground. The IK runs (it is inside
the window), discovers the target is outside the annulus `[|l1−l2|, l1+l2]`,
clamps to full extension, and returns a leg pointing at a ground it cannot touch.
`residual ≤ 0.0000` in every row is not "perfect" — it is the probe reporting
that there were **no reachable samples to measure a residual over**. The
instrument's most useful column is the one that stayed empty.

The immediate cause is embarrassingly simple, and it is arithmetic that has been
sitting in `biped_plan()` since the day it was written: **the hip is 20 mm higher
than the legs are long.** Hip 0.900, reach 0.880. So "put the sole at y = 0" is a
request for 0.900 m of extension from an 0.880 m leg — out of reach *before any
animation runs at all*. The body has always stood 2 cm off the floor, and every
foot-placement solve has always been a clamp.

Two centimetres is invisible, which is exactly why it survived. But it means the
whole foot-IK path has been running in its degenerate branch for its entire life,
and the ratified sentence "feet to actual ground" describes something that has
never happened. (Honesty note: the stout's 20 mm is *mine* — I chose its hip
height to give the same rest clearance as the biped, which is itself an
absolute-metre choice I made without thinking about it, and I only noticed when
the probe printed 48/48. The biped's 20 mm predates this work and is what makes
the biped's own count 100%.)

### The shape of every failure: scale-free authoring meets an absolute constant

Once you have two bodies, all three defects turn out to be the same defect wearing
different clothes. In each case something the *plan* expresses proportionally
collides with something the *engine* or the *clip* expresses in metres, and
nothing reconciles them:

1. **The hip/reach deficit is metres.** 20 mm on a 0.88 m leg is 2.3%; the same
   20 mm on a 0.44 m leg is 4.5%. Halving a body doubles the relative error.

2. **The authored root bob is metres.** The walk's 0.040 m bob is 4.4% of the
   biped's hip height and 8.7% of the stout's. The jump's 0.120 m is 13.3% and
   **26.1%**. Look at what that does to the jump row: *both* bodies' feet peak at
   **exactly +0.125 m**. Identical absolute float, on a body with half the legs.
   The stout does not so much jump as launch out of its own legs, and the number
   says the bob is doing it, not the solver.

3. **The foot-IK correction window is metres, and worse, it comes from the voxel
   grid.** Half a voxel is 0.450 m at N = 2. That is 0.51× the biped's leg and
   **1.02× the stout's entire leg**. A tolerance intended as "only fix small
   discrepancies" is, for the smaller body, larger than the body part it is
   correcting. It is a statement about the world's resolution masquerading as a
   statement about anatomy — and note that it would change if we changed voxel
   scale, which is not a thing that should move a character's foot.

The uncomfortable observation is that none of these are the *solver*. The two-bone
IK is fine; its unit tests are honest and they pass. The retargeting glue's
failure is not in the glue — it is in the three places where somebody wrote a
length instead of a ratio, in a system whose whole premise is that proportions
vary. And you cannot see any of them with one body plan, because with one body
plan a length *is* a ratio.

### One more thing, found while reading the sampler

Not a retargeting finding, but it fell out of the same reading and belongs on the
record: the `idle` clip's arm poses are **entirely erased by quantization**. The
authored splay angles are 0.05, 0.06 and 0.08 rad; the rotation quantum is
`TAU/32 = 0.196` rad, so anything under 0.098 rad rounds to zero. All of them do.
The only thing that survives `dc:anim/biped_idle` is its 15 mm root bob — the
breathing arm motion has never rendered, on any body. That is hand-derived from
the two constants rather than measured by the probe, and it is stated as such.

## Where this leaves the ratified claim

The honest rewrite of bodies.md's sentence, if the user wants one, is two
sentences instead of one: *IK and the clip sampler are proportion-generic and one
clip set does serve differently-proportioned plans* — measured, true, and now with
evidence. *Foot-to-ground contact is not achieved, for any plan, because the
authored hip heights, the clips' root bob, and the correction window are absolute
lengths.* Also measured, also true, and not a fact about the second plan.

Which of those to act on is not a slice's call. What the slice can say is that the
first sentence needed a second body plan to become evidence, and the second
sentence needed one to become visible at all.


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

---

## The walk, and the instrument that could not see the defect

The slice landed with an honest caveat: *"I have not seen either body rendered."*
So the integrator drove the game — built from the worktree, launched with `--edges`
on the **lit** pass (never `--fullbright`: body segment materials take
`unlit: fullbright.0`, so fullbright flattens every cuboid face to one colour and
the shape question dies), spawned one character in each plan, and looked.

Four frames: `0130-two-plans-idle` (both at rest, stout left), `0130-two-plans-walk`
(both mid-stride), `0130-two-plans-rest-reframed` (the stout's arm hanging nearly
to its feet), `0130-two-plans-jump` (both airborne). Two frames came back **empty**
— the pair walked out of shot before the stop command landed, then a wrong yaw
missed them entirely. Both were the driver's framing errors, not results, and are
recorded as such because a null frame that is really a mis-aimed camera is exactly
the thing that gets published as a finding.

**The first half of the claim confirmed by eye, not only by arithmetic.** The stout
reads as a deliberate creature — a stocky, big-headed, long-armed hominid — with
coherent joints, no detached or inverted segments, a correct turn-to-face-travel,
and a real stride. One clip set, two bodies, no new keyframes.

**And then the integrator got the important thing wrong.** Reading the frames, it
reported the 20 mm float as *sub-perceptual* and recommended treating the defect as
a design decision rather than a fire. The user, who has watched this body actually
move, answered:

> *"the hover looks bad. i do not like the body hover, the body is oscillating
> gently up and down. it's not a bob, it's a hover, because as you said, the feet
> don't touch the ground… note the defects are sub-perceptual to claude who can
> only look at screenshots, but human can see the problems jumping out."*

The number was already in the report. `idle`'s sole height ranges
**`[+0.020, +0.035] m`** — that is not an error bar, it is a **15 mm vertical
oscillation with the feet never planted**, which is the definition of a hover. The
integrator read a range as a tolerance instead of as an amplitude, and then let a
still frame overrule it.

**A still frame is structurally blind to a temporal artifact.** The float is
near-constant within any one frame, so the *hover* — the thing a human sees
instantly — is precisely the component no screenshot can carry. This is
`journal/0030`'s lesson (corrections #18/#19) arriving on a new axis: we knew to ask
whether the *lighting mode* could see the question, and not to ask whether the
*medium* could. For a motion question the screenshot is the blind control, and its
null proves nothing. `corrections.md` #77 carries it.

The right instrument was sitting in the same report: a **per-frame trace** of the
quantity, which the probe prints. Motion questions want a time series or a
recording, never a frame.

## What the walk therefore leaves owed

The user's next question is the good one — *"it would be interesting to see what it
looked like if the feet DID touch the ground and bend at the knee."* That is the
experiment this one earns: give the annulus room to exist, let the solver actually
solve, and watch a knee absorb the difference. Nobody has ever seen this rig bend a
joint under IK, because on flat ground the target has always been out of reach.
