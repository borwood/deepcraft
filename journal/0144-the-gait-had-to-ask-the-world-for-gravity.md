# 0144 — the gait had to ask the world for gravity

*Slice one of posture-gait member #1: `bake_gait`, headless, in dc-api. The design
pass (`docs/audits/2026-08-02-gait-bake-member1-design.md`) hand-derived a numeric
table with no cargo command run and called it a hypothesis the build would test.
It survived — to within six parts in ten thousand across forty-three cells — and
in surviving it produced four findings about itself. This is the entry for those.*

> blogworthy: **procgen against the backdrop of priors** — a whole gait derived
> from four published relations and *one* number nobody thought to ask for; and
> **AI-native development** — a design pass that shipped a falsifiable table
> before it shipped a line of code, then got graded by it.

---

## The problem, as it arrived

A body in this game walks by playing `dc:anim/biped_walk`: five hand-typed
keyframes, a one-second loop, a 4 cm root bob, and a clock that increments with
wall time and knows nothing about how fast the character is actually moving. It
has been measured before and it is wrong in three independent ways — the feet
supply 1.84 m/s of travel while the mover moves at 4.5, so 2.66 m/s is skate; the
authored bob is 26 % of what the clip's own leg swing geometrically requires, so
the body hovers; and at double support the front leg pins the hip at 0.726 m
while the rear pins it at 0.772 m, so one foot is 46 mm off the ground no matter
what the root does. Three defects, one cause: **a pose set was typed rather than
derived, so nothing in it has to agree with anything else in it.**

Member #0 replaced the *standing* pose with a derivation. This slice is the
walking one. It is headless and nothing visible changes: the acceptance is the
predicted table, falsifiable before a frame is drawn, exactly as member #0's was.

## What it derives, and the one thing it refuses to own

The whole gait comes out of four published relations composed:

```
Fr = v²/(gL)                      the dimensionless speed
λ  = 2.3·L·Fr^0.3                 Alexander 1976, the trackway regression
f  = v/λ = √(g/L)·Fr^0.2 / 2.3    a COROLLARY, not an independent claim
β  = 0.5·(0.5/Fr)^e               Alexander & Jayes 1983, anchored at two points
```

The third line is the pleasing one. `posture-gait.md`'s bones *assert* that
cadence scales as `√(g/L)` — "a big animal takes slow steps". Compose (1) and
(2) and it falls out, with a weak `Fr^0.2` speed term nobody had noticed. The law
was never a premise; it was always a consequence of the stride regression.

And then there is `L` and there is `g`, and one of them turned out to be a trap.

## Wrong turn the first: the design derived the whole table at Earth gravity

Every number in the design pass's § 3.5 is computed at `g = 9.81`. That is the
right constant for the *literature* — Alexander measured dinosaurs on Earth,
Winter measured people on Earth — and it is the wrong constant for the *world*.
`CharacterConfig::default()` has carried **`gravity_m_s2: 25.0`** since bring-up,
a deliberate game-feel choice sitting next to `walk_speed_m_s: 4.5`, and nothing
in the design pass reads it.

This is the closed-system scale error one tier down, and it is worth naming
precisely because the design pass was *not careless*: it was rigorous, cited, and
internally consistent — at a gravity the game does not have. Every internal
check would have passed. Only stepping outside the derivation and asking what
number the *simulation* uses catches it.

So `bake_gait` takes gravity as an argument. Not a constant, not a default — an
argument the caller must state. The body does not own gravity; the world does.
The bake is still a pure function of the body definition plus one declared world
constant, which is what keeps it callable from pack build, deeptime worldgen and
define time alike.

What changes downstream is smaller than it sounds, and that is itself the
finding: **the structural claims do not move at all.** At equal Froude number,
hip excursion, duty and bob-to-reach ratio are identical across biped, stout and
longleg at either gravity, to 1e-12. Cadence ratios are `√(L_j/L_i)` at either
gravity, to 1e-12. Dynamic similarity is a property of the derivation, not of a
number. What moves is what a *given speed in metres per second* means: at
`g = 25` the biped's comfortable walk is 2.35 m/s rather than 1.47, and 4.5 m/s
is Fr 0.92 rather than Fr 2.35 — still a run, so the design's flag survives, but
a plausible one rather than an absurd one.

## Wrong turn the second: a rigid leg cannot have double support

The design says `h(phase)` — root height — is "scalloped, two troughs per cycle,
flat-bottomed for the double-support fraction 2β−1". The first two thirds are
right and the third is not derivable, and finding out why took the better part of
the build.

During stance the contact is pinned and the chain is a rigid link, so the
attachment joint traces a circle: `h = L·cos θ`. Fine. But if you let each limb's
angle be driven by its own duty-length stance window, then for the 20 % of the
cycle where both windows overlap you have **two pinned feet and two different
demands on one hip height** — which is, exactly and embarrassingly, the 46 mm
contradiction the authored clip carries. Deriving the pose set was supposed to
make that contradiction structurally impossible; a naive β-driven stance would
have reproduced it faithfully.

The resolution is that duty and stance *geometry* are different quantities and
must be decoupled. The bake derives a **compass window** per contact — the phase
gap to the next footfall — and these tile the cycle exactly by construction, so
`h(phase)` is continuous and single-valued and there is exactly one trough per
bearing contact. A biped gets two troughs (twice the limb cycle: the measured
0.5 s against a 1.0 s clip, now derived rather than typed); a quadruped gets four
shallower ones with no special case; a distributed bearer gets none. Duty β is
still emitted — contact scheduling and B5's limp need it — but it does not touch
the poses.

The gap between them is then a *measurement*, and the bake reports it: the rigid
chain supports for 50 % of the cycle where the literature says 60 %. That 10
points is stance-knee flexion, pelvic list and ankle roll — force, which density
≡ 1 does not have. It is the same shape as the design's own ballistic-swing
cross-check (a pendular swing of 0.762 s against a wanted 0.364 s, 2.10× — the
published finding that real walking is not ballistic above a slow walk) and it
gets the same treatment: **printed, cited, and never tuned to close.**

Consequence for the design doc: the trough is *cusped*, not flat-bottomed. The
amplitude is unaffected.

## The pose that could not be stored

Finding G2 in the design says speed is deliberately not a bake-key axis and every
gait term is a function of speed, so the bake emits coefficients. Its § 2 output
table then lists `contact` as a stored pose in radians. Those two rows cannot
both hold: the contact pose *is* `neutral` rotated by θmax, and θmax is a
function of Fr. Freezing it picks a speed.

The resolution costs one multiply. What is stored is the pose **gradient** —
radians of joint rotation per radian of hip excursion — and `contact(Fr) =
neutral + θmax(Fr)·gradient`. For a chain that is a rigid rotation about its
attachment joint (which is every chain member #0 accepts, since it refuses
anything that is not a vertical column) the linear form is *exact*, not an
approximation. `neutral` and `clearance` are genuinely speed-invariant and are
stored as poses, so the user's sparse-keyframes-with-the-server-between-them
design is preserved verbatim: three stored poses, and the sim is still "between
which keyframes, and how far".

`clearance` being speed-invariant is a small piece of luck worth stating: at
mid-swing the foot passes directly under the hip, so its pose depends only on the
clearance ratio and the chain's own bone lengths. It is a two-bone analytic solve
and the test does not check the angles it produces — it walks the joint tree
forward and asserts the anchor lands under the attachment at exactly the
clearance height, to 1e-12, at three different flexion settings. An instrument
that re-derives is worth more than one that re-reads.

## G3 was not a change to member #0 after all

The design predicted that the gait would need member #0's resting solve extended
to non-bearing chains — a swinging arm has no stance — and flagged it as "a
change to member #0's function, not a free read".

It isn't, and the reason is instructive. Member #0 solves chains that terminate
in a **declared contact anchor**. An arm declares nothing; its chain terminates
in the bottom face of its own last box. That is a different derivation, not an
extension of the same one, so it lives in the gait bake and `bake_resting_posture`
is untouched. Which limbs get one is derived, never declared: a leaf chain that
no bearing contact claims, that hangs as a vertical column under its attachment,
and whose attachment is **laterally displaced from the midline**. That last
clause is what keeps the head out of it — a midline appendage's carriage is a
taste knob with identity zero, not a derived counter-swing — and it is B0's rule
that geometry determines laterality, applied one level up.

The pairing rule is § 3.3's: each swinging chain takes the phase of the
*contralateral* bearing chain. For the biped that means the left arm runs half a
cycle from the left leg. The authored clip has `arm_l_upper = −0.5` where
`leg_l_upper = +0.6`. **The rule was written down before the clip was consulted
and it predicts the clip exactly.** So does the phase rule for the legs: `{0.0,
0.5}` derived from contact geometry alone, against a clip that measures
`leg_r(t) = leg_l(t + 0.5)` at every key on both bones, to 1e-12. Two
zero-parameter predictions against hand-typed data, both hits.

(The magnitudes do not match and were not asked to: the clip swings the arm at
0.83× the leg, the derivation says 1.0×. That is what `arm_swing_amplitude`
exists for, and it is taste, with no true value.)

## Measured against predicted

Forty-three cells of the design's Tables A, B and C, asserted at 1e-3 relative —
the honest precision of a table hand-derived to four decimal places. Worst
deviations: **5.9e-4** (Table A, longleg's bob, where the design rounded 0.076245
to 0.0762), **1.6e-4** (Table B), **5.1e-4** (Table C). Every cell inside the
band; every structural claim asserted at 1e-12 instead and exact.

Three cells are worth calling out.

**The cadence ratio is wrong in the design.** § 3.5 states
`f_biped/f_longleg = √(1.020/0.880) = 1.076750`. The square root of 1.1590909 is
**1.07661084**. The relation is right, the arithmetic slipped by 1.3e-4. The test
asserts the *derivation* — `f·√L` constant across all three plans to 1e-12 — so
the slip has no consequence beyond the document, but a number quoted to six
places in a ratified design doc is a number somebody will copy.

**The duty exponent is better derived than typed.** The design quotes 0.263, from
two published anchors: β = 0.50 at the transition and β ≈ 0.60 at Fr 0.25. Solve
for it rather than round it and it is **0.2630344058**, which makes β(0.25) land
on 0.600000000 instead of 0.5999847. It is a small thing and it is the doctrine
working: a constant with a derivation is evidence; the same constant typed to
three places is a number.

**The clip cross-check only lands at Earth gravity.** At `g = 9.81` the derived
stride is 16.9 % from the authored clip's own implied stride at the clip's own
speed — inside the design's 25 % band, and the direction is right (the clip sits
20 % above Alexander's regression, comfortably inside its published scatter). At
`g = 25` the same comparison is **37.2 %**, outside the band. That is not a fact
about the derivation; it is a fact about a hand-authored human walk being a
hand-authored *human* walk. The test runs at Earth gravity and says why, and
prints the world-gravity number beside it.

## What the bake refuses, and what it merely reports

The line is the docket's and it held up under contact: **report when a value
leaves a published band, refuse only when the solve has no answer at all** — a
clockwork golem may want to step wrong.

Reported and proceeded with: a stand-in knob outside its band (each report names
B6 as its heir); the geometric-versus-published duty gap; a stride the regression
wants that the chain cannot span; and the run regime itself.

Refused, loudly and by name: a resting solve that refused (the reason is
propagated, not swallowed); a three-bone chain, because the mid-swing solve is a
two-bone analytic IK and says so rather than fabricating a redundant-chain answer
— the refusal names B7's joint limits, since picking among a redundant chain's
solutions is exactly what limits are for.

And one thing is neither: **the run's root height is declined.** At the shipped
`walk_speed_m_s = 4.5` all three bodies sit above the walk/run transition at
either gravity, and a run has a flight phase whose apex is ballistic. There is no
takeoff force at density ≡ 1, so the bake returns `RootHeight::Declined` carrying
the Froude number, the band and `stubs.md` #39's heir. It does not guess.

Two of the design's own acceptance tests could not be written as specified, and
both are findings about the document rather than gaps in the build. Test 5 asked
that a distributed bearer produce "exactly zero root-height variation" — but the
resting solve *refuses* a distributed bearing upstream, so the gait never exists
to have a zero in it. The refusal, naming the geometry, is the stronger answer
and is what is asserted. Test 10 is the `BindTarget` vocabulary, which is a wire
change to `JointRot` and belongs to the slice that owns the schema.

## What is left standing

`bake_gait` is a pure function in dc-api beside `bake_resting_posture`. Nothing
visible changed; dc-client was not opened. `Keyframe.root_bob_m` still exists and
`dc:anim/biped_walk` is still registered, both by ruling — they leave *with their
derived replacement*, in the consumer slice, and that slice owes a walk.

The one composition is built and waiting for it: `root_height_ratio_at(Fr, phase)`
is a single function, and there is no additive bob term anywhere for a second
consumer to forget. The drift corrections #80 diagnosed is not fixed; it has
nowhere to happen.
