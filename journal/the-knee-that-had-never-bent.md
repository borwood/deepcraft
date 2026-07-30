# The knee that had never bent

> **Journal number: UNASSIGNED.** 0128–0130 were taken when this slice ran; the
> integrator assigns the number at merge. `docs/design/bodies.md` cites this entry as
> `journal/the-knee-that-had-never-bent` and will need the number substituted.

The previous entry (journal/0130) ended on a refutation it could not act on. We had
built two-bone IK, wired it into foot placement, written eleven tests around it, and
shipped it — and then measured that **it had never once solved**. On flat ground,
across every stepped frame of all three clips and both body plans, **176 of 176
samples** were handed a target beyond the leg's reach. The solver clamped to full
extension every time. The rendered result was a hover: the sole oscillating over
`[+0.020, +0.035] m`, never touching anything.

The user watched it move and said the thing that set this slice going:

> the hover looks bad… it's not a bob, it's a hover, because the feet don't touch the
> ground. it would be interesting to see what it looked like if the feet DID touch the
> ground and bend at the knee.

**Nobody had ever seen this rig bend a joint under IK.** Not once, in the whole life
of the code. That is what this entry is about.


## The trap: the fix everybody reaches for is the one we were forbidden

The cause 0130 found is two numbers deep. `biped_plan()` puts the hip pivot at
**0.900 m**. Its leg bones are 0.45 and 0.43, so the leg reaches **0.880 m**. A sole
at `y = 0` is 20 mm further from the hip than the leg can extend — *before any
animation runs, before any terrain is queried*. The obvious move is to edit one of
those numbers. Or `CROUCH_ROOT_DROP_M`. Or the clips' `root_bob_m`. Or the half-voxel
correction window.

Every one of those is inside an **unresolved user call**. bodies.md § IK now carries a
banner asking whether the 20 mm gap is deliberate (feet clearing terrain seams) or an
off-by-a-half-thickness, and whether hip height, root bob and the correction window
should become **ratios of the plan** rather than absolute metres. That call moves how
every body in the game looks. An implementation slice does not get to answer it — and
if it edits any of those constants, it *has* answered it, whatever the commit message
says.

So the constraint was: **find out whether the solver works, without touching anything
the call owns.** The answer turned out to be pleasingly cheap.


## The instrument: a third plan, and exactly two numbers changed

Body plans are *data*. They arrive through the registry as a command batch. And there
is already a batch that exists purely to hold instruments — `experiment_body_pack()`,
where `dc:body/stout` lives, deliberately outside the default pack because *existence
is not standing*.

So: author a plan whose legs can actually reach the ground, put it in the experiment
batch, and change nothing else. `dc:body/longleg` is `biped_plan()` with **two numbers
different**:

| | `dc:body/biped` | `dc:body/longleg` |
|---|---|---|
| hip height | 0.900 m | 0.900 m — *identical* |
| upper bone `l1` | 0.450 m | 0.520 m |
| lower bone `l2` | 0.430 m | 0.500 m |
| reach `l1 + l2` | 0.880 m | 1.020 m |
| hip − reach | **+0.020 m** (unreachable) | **−0.120 m** (0.120 m of slack) |
| trunk, arms, head, neck | — | byte-identical |
| clips bound | idle/walk/jump | *the same three, unmodified* |

It is written as `let mut plan = biped_plan();` followed by a loop that rewrites the
leg segments' lengths, so the isolation is structural rather than a promise — and a
test walks every non-leg segment asserting equality with the biped's, so the control
fails loudly if somebody re-authors the trunk. Zero engine constants moved. No policy
decided. The plan is deletable content: one function plus one entry in a `vec!`.

At rest, un-IK'd, `longleg`'s soles hang **0.120 m below the floor**. That is not a
defect; it is the entire point. The IK is what has to lift them, and it can only do
that by bending the knee.

### Why 0.120 m of slack, and not 0.03 or 0.30

This is the part where a number could easily have become a number pretending to be a
mechanism. The IK target for a planted sole is `(0, −hip_y, fz)`, where `fz` is however
far forward the *clip* has swung that foot. So the reach the solver actually needs is
`hypot(hip_y, fz)` — and `fz` scales with the bone lengths. **The demand grows as the
legs do.** Adding slack chases its own requirement.

Sweeping the authored clip set (idle 24 frames, walk 12, jump 8, both legs, at the
renderer's 12 fps grid) puts the fixed point just above **0.102 m**, and exposes a
trade nobody had seen because nobody had ever had a plan on the reachable side of it:

| leg slack | resting knee bend | samples still beyond reach (of 88) |
|---|---|---|
| −0.020 m (today's biped) | 0.0° — clamped dead straight | **88**, every one |
| +0.030 m | 29.2° | 32 |
| +0.050 m | 37.3° | 14 |
| **+0.120 m** (`longleg`) | **56.2°** | **2** |
| +0.180 m | 67.1° | 0 |

**There is no slack that both plants every frame and keeps the knee out of a squat.**
0.120 m was chosen as the smallest value that plants every `idle` and `jump` frame and
all but two of `walk`'s while keeping the resting bend under 60°. The two survivors are
walk's stride extremes, which want `hypot(hip, stride)` and not `hip`.

That trade is a finding, not a footnote. The degree of freedom nobody is spending is
the root's **vertical travel**: the clips author a bob *upward* where a real walk drops
the pelvis over the stance leg. Which is evidence for the open user call, and
deliberately not an answer to it.


## The instrument had to be able to see a temporal question

corrections #76, written the same session, is about the integrator's own error: a
**still frame is structurally blind to a temporal artifact**, and a range like
`[+0.020, +0.035]` is an **amplitude**, not an error bar. The probe as it stood
reported exactly such ranges. So it was rebuilt to keep the whole series: a `FrameRow`
per stepped frame, a `LegSample` per foot, carrying ground height, clip sole, rendered
sole, gap, hip angle, **knee angle in degrees**, and a four-valued verdict —
`Seated | Corrected | ClampedBeyondReach | Refused` — whose counts close on their own
total at two levels, asserted.

The knee angle is the column that did not exist before, and it is the one the user's
question was actually about.


## What the series says

### Does the knee bend? Yes — on the first frame it is asked.

`dc:body/longleg`, `idle`, flat ground. Twenty-four frames; two shown per distinct
value:

```
    f  t(s)    bob |  gnd    sole    gap    hip°  knee°   v
    0  0.000 +0.000 | +0.000 +0.004 +0.004  +22.5 -56.2 CORR
    3  0.250 +0.005 | +0.000 +0.009 +0.009  +22.5 -56.2 CORR
    6  0.500 +0.010 | +0.000 +0.014 +0.014  +22.5 -56.2 CORR
   11  0.917 +0.015 | +0.000 +0.019 +0.019  +22.5 -56.2 CORR
```

The same rows for `dc:body/biped`:

```
    0  0.000 +0.000 | +0.000 +0.020 +0.020   +0.0  +0.0 clmp
   11  0.917 +0.015 | +0.000 +0.035 +0.035   +0.0  +0.0 clmp
```

**−56.2° against 0.0°.** The biped's knee is dead straight on all 48 samples of idle.
`longleg`'s is bent 56.2° on all 48. Across the whole matrix — three plans × three
clips × three grounds — `longleg` plants **86 of 88** flat-ground samples; the biped
and stout plant **0 of 88 each**, exactly reproducing 0130.

On `walk` the contrast is sharper still, because there the knee has to *move*:

```
dc:body/biped   / walk / flat  — knee  +0.0° on every one of 24 samples,
                                 sole  +0.037 … +0.102 m off the ground
dc:body/longleg / walk / flat  — knee −22.5 … −45.0°, articulating per frame,
                                 sole  −0.017 … +0.052 m
```

The biped's foot is up to **10 centimetres** in the air mid-stride with a rigidly
straight leg. That is the hover, itemised.

### And here is the bad-looking part, which is the best thing we learned

`longleg`'s `idle` gap runs **+0.004, +0.009, +0.014, +0.019** — and the knee angle is
**−56.2° on every single frame, unchanged**. The gap is tracking the root bob exactly:
0.000, 0.005, 0.010, 0.015.

**The feet are still floating, in lockstep with the bob, by exactly the bob's
amplitude.** The offset collapsed — 20 mm down to 4 mm — and the **oscillation did not
move at all**: 15 mm of swing on the biped, 15 mm of swing on `longleg`.

The mechanism is the third quantization. `stepped_angle` snaps IK output to
`ROT_QUANTUM_RAD` = 11.25°, so the knee cannot change by less than one 11.25° step —
and at this geometry one step is worth far more than 5 mm of sole height. The bob asks
the solver for a 5 mm correction; the solver computes it exactly; the quantizer rounds
it away. **The bob passes straight through the IK to the floor.**

So the hover had **two independent causes**, and this slice isolated one:

1. **A constant offset** from `reach < hip`. Cause: plan geometry. Fixed here, by data.
2. **A 15 mm oscillation** from the root bob surviving the rotation quantum. Cause: the
   stepped-animation aesthetic meeting an absolute-metres bob. **Untouched, and it is
   not the solver's fault either.**

A screenshot of `longleg` standing still would have shown a foot 4 mm off the ground
and been read as "fixed". Only the time series shows the 15 mm breathing that is left.
corrections #76, demonstrated on the very next slice that cited it.


## The uneven-ground question 0130 could not ask — and a surprise

0130 measured flat ground only, and flat ground is the degenerate case: both feet want
the same answer, so a null there says nothing about *foot placement*, only about the
hip/reach arithmetic that is identical for both legs. So the probe grew ground cases: a
**synthetic +0.300 m step** under one foot, and a **one-voxel +0.900 m step** — the
smallest offset real N=2 terrain can produce.

The surprise is in the first one. On a 0.300 m step, the **unmodified biped's raised
foot plants** — 24 of 48 idle samples corrected, knee bending to **−90.0°**:

```
dc:body/biped / idle / step+0.300
    f  t(s)    bob | L: gnd    sole    gap   hip°  knee°   v | R: gnd    sole    gap  hip° knee°   v
    0  0.000 +0.000 |   +0.300 +0.278 -0.022 +45.0 -90.0 CORR |   +0.000 +0.020 +0.020 +0.0  +0.0 clmp
    6  0.500 +0.010 |   +0.300 +0.288 -0.012 +45.0 -90.0 CORR |   +0.000 +0.030 +0.030 +0.0  +0.0 clmp
```

**Foot placement does engage on uneven ground, and it always could have.** Raising the
ground under a foot moves the target *closer* to the hip — from 0.900 m to 0.600 m —
and 0.600 m is comfortably inside the biped's 0.880 m annulus. So the biped **plants
its uphill foot and hovers its downhill one**, asymmetrically, and has done so since
the day the code shipped. 0130's null was an artifact of the ground it chose. Its
conclusion ("the IK never engages") was true of every sample it took and **false as a
statement about the mechanism** — the corollary in CLAUDE.md about reading a null,
landing on the entry that established it.

Note also the sign: the raised foot lands **22 mm below** the step surface, sunk into
it. Same quantizer, opposite direction.

### And then the posture axis, which found the whole thing had been working all along

The uneven-ground surprise suggested the obvious next question: *what else moves the hip
toward the ground?* **Crouching does.** `CROUCH_ROOT_DROP_M` sinks the cosmetic root
**0.45 m**, and `character.rs` applies it to the hip before the IK runs. So the probe
grew a posture axis, and the result is the largest single finding in this entry:

```
dc:body/biped   / flat / crouch  — 88/88 samples CORRECTED, knee to −123.7°
dc:body/biped   / flat / stand   —  0/88, every one clamped, knee 0.0°
```

**The stock biped's foot-placement IK has always worked. While crouching.** Nobody had
ever looked at a crouching body. The 0.900 m hip becomes 0.450 m, which sits comfortably
inside the 0.880 m annulus, and every foot plants with a deep bend. journal/0130's
headline — *"foot-placement IK has never once engaged"* — is true of **standing** and
was never true of the system.

That is the second time in two entries that the same corollary bit: *before concluding
"system X never fires", check that you sampled the states where it would.* 0130 sampled
one posture on one ground and generalised to a mechanism. This entry sampled four cases
and found the mechanism fires in three of them.

And the axis is not decoration, because it also exposes a **fourth absolute-metres
constant** that bodies.md's units banner does not name. 0.45 m is 50.0% of the biped's
hip height and **97.8% of the stout's** — a crouching `dc:body/stout` puts its hip at
**0.010 m** and folds its knee to a degenerate **180.0°**, the leg doubled flat on
itself. The banner lists hip height, the clips' root bob and the IK window. There were
always four.

### Two gates, and no plan passes both

Laying the cases side by side gives the cleanest statement of what is actually wrong.
Foot placement has **two independent gates**: the ground must be inside the *half-voxel
window*, and the sole target must be inside the *annulus*. Different plans fail
different gates in different postures:

| | flat, standing | flat, crouching | +0.30 m step | one-voxel step |
|---|---|---|---|---|
| `biped` | annulus refuses, 0/88 | **88/88**, knee 123.7° | uphill foot plants, knee 90° | window refuses raised foot |
| `stout` | annulus refuses, 0/88 | 88/88, knee **180°** (degenerate) | uphill foot plants, knee 135° | window refuses raised foot |
| `longleg` | **86/88**, knee 56.2° | **window refuses, 4/88** | 87/88 | window refuses raised foot |

Read the diagonal. `longleg` standing works and crouching does not; `biped` crouching
works and standing does not. The plan I built to make the solver run *broke the posture
that already ran* — because its legs are now so long that crouching puts the clip's foot
0.57 m below the ground, and the window only admits 0.45 m.

**Nothing in the codebase reconciles those two gates.** One comes from the voxel scale,
one from the plan's bones, and they were never introduced to each other. That, rather
than any single constant, is what the open user call is really about.

### The third refusal, and this one is scale-free

The one-voxel step is where it stops being a tuning question. The correction window is
**half a voxel**. The smallest relief real terrain can have is **one whole voxel**. The
ratio is fixed at **2:1 by construction, at every scale N** — a finer world shrinks the
voxel and the window in lockstep. On a one-voxel step, every raised-foot sample is
refused: all three plans, all three clips, no exceptions. It is now an assertion.

Which reframes the window's units problem. It is not that half a voxel is the wrong
number. It is that **a voxel-derived tolerance can never express a terrain step**, so
the only offsets foot placement ever sees are *sub-voxel* ones — and today those are
produced by nothing except the plan's own hip/reach mismatch. The mechanism's only
customer is the defect.


## What the slack bought, and what it did not

It bought the answer to the question that was asked. **The solver was never the
problem.** Handed a reachable target it solves exactly, on the first frame, with a
knee bend nobody in this project had ever seen — and the retargeting half of
bodies.md § IK survives a third set of proportions with no new keyframes.

It did not buy planted feet, and it is important that the report says so. Two things
stand between `longleg` and a foot that stays on the ground: the 15 mm the rotation
quantum cannot resolve, and the stride extremes that want more reach than any
squat-free slack provides. Both are engine-and-authoring questions sitting squarely
inside the open user call. This slice's job was to remove *reachability* from that
call's list of unknowns, and that it did.

What it bought that nobody asked for is worth more: the plan was built to make the
solver run once, and instead it turned the probe into something that could enumerate
**when** foot placement fires. The answer — three of four cases, including one that has
been firing silently since the code shipped, and a fourth constant nobody had counted —
was only visible because the instrument grew an axis. The reachable plan is almost a
footnote to the matrix it made possible.

### Wrong turns

- **The first instinct was to nudge `l2` by 20 mm.** It is a one-character diff and it
  "obviously" fixes the hover. It would also have silently answered the user call, in
  the direction of "the gap was a bug", with no discussion — the exact shape
  corrections #65 was written about. Data instead of constants was not the elegant
  choice; it was the only one available.
- **The brief's suggested 0.120 m looked arbitrary and turned out to be right for a
  reason the brief did not give.** Deriving the fixed point first produced the
  slack-vs-bend table, which is the most useful thing in this entry — and it only
  exists because the number was checked instead of typed.
- **The step case was nearly authored at 0.900 m only**, as "the realistic one". It
  would have refused every sample and reported another clean null — and the biped's
  90° uphill knee would have stayed invisible. A synthetic case that no terrain can
  produce was the one that could see the question.
- **The posture axis was very nearly not built.** The brief did not ask for it and the
  plan was already done; adding it meant touching the probe's signature a second time.
  The only reason it happened is that the build slot was occupied by a sibling's gate, so
  the marginal cost of another edit was zero — and it produced the biggest result in the
  entry. That is an uncomfortable thing to record: **the most valuable measurement here
  was a consequence of being blocked**, not of judgement. The transferable version is
  that "does mechanism X ever fire?" is never answerable from one point in the state
  space, and the axes are usually cheap to enumerate once the harness exists.

> blogworthy: **the closed system that could not see its own hover.** Two independent
> defects, superimposed, in a quantity every internal check was blind to — one fixed by
> a data-only control, one exposed only by refusing to report a range. Lenses 1 (the
> instrument must be able to see a temporal question, and the probe that cited that
> rule then had to obey it) and 3 (three quantizers, an absolute-metres bob and a
> voxel-derived tolerance meeting in one seam nobody owned).
