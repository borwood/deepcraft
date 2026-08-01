# The pelvis was pinned, and the legs were asked to reach

A design pass, not a build. It began as *"let's blue-sky the bio and evolution passes"* and
ended, four hours later, with a cautiously-ratified tier for posture and gait — having gone
nowhere near biology.

> blogworthy: **lens 1 (AI-native development) and lens 3 (reflexions in a deepsim
> codebase).** Three of this session's four corrections were the assistant's own, and all
> three were caught by a human *watching something move* while the assistant read numbers and
> still frames. The lens-3 angle is sharper: the defect the whole session circled turned out
> to be a **dependency inversion** — a quantity with a physical determinant being authored —
> and the codebase already had a rule against exactly that, filed under a different name in a
> different subsystem.

## How a bio conversation became a bodies conversation

The opening ask was ecology and evolution. The sweep said: `ecology.md` is a **ratified user
design** that has been dormant since 2026-07-19, biology *does* run in production (the S10
biotic layer is on, and it moves rock), and the gate for new bio work is a user call that
engine progress does not earn.

Then the user redirected, and the redirection contained the whole architecture:

> *"engine owns bodies, plugin pack can define them. therefore an evolution plugin pack can
> model evolution — of bodies."*

That is not a new boundary. `north-star.md` § Refinement has listed **bodies** among the core
data-model APIs since 2026-07-23, beside materials and blueprints, and it names *"the
evolutionary graduation of body declarations to agents"* as deferred-not-foreclosed. What the
user was pointing at was the gap between a ratified boundary and an **unexercised** one.

And it was unexercised in the most literal way. `vanilla_body_pack()` — the function whose
entire purpose is *"vanilla is the first pack"* — was referenced only from its own test. The
client called the authored source directly. The door existed; nothing had walked through it.

## The experiment, and what it was really testing

`bodies.md` has said since 2026-07-19, marked DECIDED, that two-bone IK is *"the retargeting
glue that makes one clip serve every mutation of a plan… **across differing proportions**."*

That is a claim quantified over a set. The set had **one element** — `character.rs` said so in
a comment nobody had reason to distrust: *"v0: every character wears the one vanilla plan."*
Every green test was a test of the solver's arithmetic; none was a test of the claim.

So we built a second element. `dc:body/stout`: half the legs, 1.6× the arms. The first half of
the sentence came back **measured true** — rig ratio exactly 0.500, `idle` poses byte-identical
across plans, artifacts shrinking in metres, and a second body that reads as a deliberate
creature. The second half — *"feet to actual ground"* — came back never once achieved, 176 of
176 frames beyond reach.

Then a third plan with 12 cm more leg made the knee bend for the first time (−56.2°), and the
same sweep found that the stock biped's IK **had always worked while crouching**, because a
0.45 m crouch drop puts the hip at 0.450 m against an 0.880 m reach. journal/0130's *"never
engaged"* was true of **standing only**, and it had been merged an hour earlier.

## Four corrections, and the two that share a shape

- **#77** (the assistant's): the foot float is *"sub-perceptual, so it is a design decision
  rather than a fire."* Falsified within the hour by the user, who had watched it move. **A
  still frame is structurally blind to a temporal artifact** — the offset is near-constant
  within a frame, so the *hover* is precisely the component no screenshot can carry. The number
  was in hand and misread: `idle` sole height `[+0.020, +0.035] m` is a **15 mm amplitude**,
  read as an error bar.
- **#78** (the assistant's): *"IK has never engaged."* The probe swept clip × plan × foot
  exhaustively and held **posture** at its default — a two-valued axis that moves the hip by
  half the body.
- **#79** (the user's catch): a comment in the mesher called the binary block-tier collider
  *"the accepted, documented visible mismatch"* and cited `visuals.md`, which actually says
  *"for now… until movement learns partials"* and names the heir. **A citation that drops its
  source's temporal qualifier converts an interim into a decision** — and is worse than an
  uncited claim, because the citation makes it look verified. The qualifier survived where
  designers look and died where coders look. The assistant then cited that invented permanence
  twice in a live design argument.
- **#80** (the assistant's): *"the rotation quantizer is THE DEEPEST CAUSE of the hover."* True
  as arithmetic, wrong as a diagnosis.

**#77 and #78 are the same failure one level apart**: *the instrument could not see the axis*
and *the sweep did not include the axis*. Both are an unswept axis reported as a property of
the system, and neither is visible from inside the measurement, because the itemisation is
always complete over the axes it has.

## The diagnosis that redirected everything

The user, on seeing the fixed body still hovering:

> *"the knee rotation does not change at all during this, meaning the hover offset is not
> changing coordinates of the body in a way legible to the IK solve — a layering of global and
> local space, perhaps."*

Exactly right, and two lines of code apart. `character.rs:219` builds the hip the solver sees
as `feet.y + hip_local[1] - crouch_drop`; `:257` adds `root_bob_m` to the rendered root.
`crouch_drop` appears in both. `root_bob_m` appears only in the second. The solver was handed
a hip that never moves, returned a correct and **constant** leg pose, and the renderer hovered
the whole body underneath it.

**The discriminating evidence was free and sitting in journal/0131's own tables: knee angle
constant while the gap tracks the bob one-for-one.** A quantizer eating a varying correction
and a solver never being handed one look identical in the *gap* column and completely
different in the *angle* column. Both columns had been printed. The lesson worth keeping is
that **a mechanism that explains the magnitude is not thereby the cause** — and the quantizer
arithmetic was persuasive *because* it was correct, checkable, and the right size.

## The inversion

Then the sentence that made it a tier rather than a bug:

> *"the squat does not read realistic. part of the reason for this is that posture actually is
> responsible for keeping center of gravity."*

`dc:body/longleg` squats because we pinned the pelvis at an **authored** hip height and made a
longer leg absorb the surplus by bending. A real animal with those legs stands nearly straight
and **stands taller**. Hip height is an *output* of leg length, never an input beside it.

> **We pin the pelvis and ask the legs to reach the floor. Reality pins the feet and lets the
> pelvis land where the legs put it.**

And that reframes `root_bob_m` as **a leaked requirement** — a walk's bob is what alternating
stance legs *do*, so authoring it as keyframes is a summary standing in for an authority, and
creates two authorities for one quantity that then cannot agree. This repo has had a rule
against that since 2026-07-21. It was filed under geology.

Four absolute-metre constants had become definitions — the 0.900 m hip, the clips' root bob,
the half-voxel foot window, `CROUCH_ROOT_DROP_M` — for one reason: **with a single body plan,
a length is a ratio.** Nothing could distinguish them. Anti-shape A-1, invisible until N=2.

## The road not taken, and why

The obvious response was *run physics on the characters*. The user cut that off immediately —
*"it's out of the question to be doing constant physics solving on characters all the time"* —
and replaced it with the version that fits the engine: **bake it.** Solve per species at pack
build, ship the result as data, sample cheaply.

Which is the two-clocks doctrine, and *structurally the same move* as an idea from earlier in
the same conversation: baking a fern's fractal frond into a small texture per species at
gen time and inheriting it down the phylogeny. Two instances of one shape in one session.

The last knot was the user's: ping-pong works for symmetric cycles, but **a limp is
asymmetric** — so how do you parameterise gait and still have two keyframes? The answer turned
on noticing that **a limp is a timing asymmetry, not a pose asymmetry**: you do not hold the
hurt leg differently so much as you get off it faster. So the two keyframes become `neutral`
and `extreme`, and the gait becomes a per-limb `(phase, amplitude, duty)` triple — at which
point **mirroring disappears entirely**, because every limb plays the same cycle shifted in
time. That deletes, structurally, the very defect that opened the arc: `arm_l_upper` and
`arm_r_upper` as two independent lists of magic numbers whose symmetry was a coincidence of
hand-typing.

And it is checkable. Gait taxonomy *is* phase offsets; duty factor is the walk/run
discriminator; cadence scales as √(g/L). A baked gait can be falsified against published bands
instead of tuned until it looks right — which is this project's standing bar for a constant
being evidence.

## What is written down

`docs/design/posture-gait.md`, cautiously ratified, bones only, members explicitly directions
rather than build orders. The first slice is the resting-posture bake, and its acceptance
criterion is falsifiable in one line: **`longleg` must come out standing taller than the
biped, not squatting.**

The thing that makes it matter beyond bodies is the edge nobody had written down until today:
authored clips break the moment topology changes, so an evolution pack would generate bodies
**nobody could animate**. A bake from `(segment tree + masses)` means a mutated body gets a
plausible stance and gait *by construction*. The session began by asking how an evolution pack
could work. It ends holding the reason one couldn't — and the shape that unblocks it.
