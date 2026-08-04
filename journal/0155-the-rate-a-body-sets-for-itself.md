# The rate a body sets for itself

*(pending — SLUG-ONLY, no ordinal: a parallel user session shares this checkout and
ordinal collisions have happened six times in thirty hours. The integrator numbers it.)*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — a constant that was correct
> for exactly as long as there was one clip at one cadence, and the shape of the thing that
> replaced it. Also **lens 1 (AI-native development)**: the ruling arrived complete, the
> slice's whole job was to find out which of its numbers were measurements and which were
> conversation.

## What was wrong, in one line

`ANIM_FPS = 12.0` quantized **per second** against a cadence that is derived **per body**.

Which is fine when every body walks at the same rate. It stopped being fine on 2026-08-02,
when locomotion became a closed-form function of the body's own leg length — and nobody
noticed, because the biped's frames-per-cycle landed at 6.97, which is within 0.03 of an
integer, and an almost-integer grid almost repeats. The stout, with the shortest legs and
therefore the fastest cadence, got **4.29** frames per cycle. Its samples walked around the
cycle instead of landing on it, and it hitched. The user saw that at the 0148 walk and
diagnosed the mechanism unprompted, from the hitch alone.

The fix the user ruled is one line: **`N = round(cycle_duration × target_fps)`, clamped to
`N ≥ 2`**, per cycle rather than per second, per bone rather than per body, and with the
target owned by the **client** rather than the pack.

## The thing worth writing down: the point is the integer, not the number

Every instinct says a hitching animation wants *more frames*. It does not. The stout at
4.29 and the stout at 4 sample the same number of poses per stride to within a rounding —
what changes is that 4 comes round to the same four phases every stride, and 4.29 does
not. The whole defect is the fractional part beating against the cycle.

That single sentence decided the whole implementation:

- **The gait is quantized in PHASE, not in time.** The phase clock is already a distance
  clock (`phase += cadence·dt`, one full cycle per stride travelled), so holding the phase
  at `floor(phase·N)/N` *is* "N poses per stride" exactly, at any speed, forever. The
  retired code held the phase whenever a world-global 1/12 s frame index ticked over —
  which is why the phases wandered.
- **A clip is quantized in phase too**, which turned out to close a defect the board had
  been carrying (below).
- **Nothing gets smoother.** At the 12 fps default the biped goes 6.970 → 7, the longleg
  7.729 → 8, the stout 4.291 → 4. Two of the three get *fewer* held poses per second than
  before, and that is correct.

Measured, over twenty strides at 4.5 m/s: the retired 1/12 s grid sampled **140** distinct
phases for the biped, **155** for the longleg, **86** for the stout. The per-cycle grid
samples **7**, **8** and **4**. That is the beat, as a number.

## Wrapping first was not a fix we chose — it is what per-cycle quantization *is*

ROADMAP § Observed carried a 🟠 item: `quantize_time` floored onto a grid anchored at
absolute `t = 0` and wrapped *afterwards*, so a looping clip whose duration was not a whole
number of frames sampled a different set of sub-frame phases every cycle. It was latent —
every shipped looping clip happened to be frame-aligned at 12 fps — and it was deliberately
deferred, with the note that the one-line alternative was *"wrap first, then quantize"*.

The entry also recorded something sharper: a test called `looping_wraps_deterministically`
had asserted **bit-identity** across a loop for a year, passing only because the rotation
quantizer was rounding the difference away. When that quantizer was removed the assertion
turned out to be **false by construction**, and it was retargeted to a 1e-12 tolerance.
Locally right, and it fixed the test to match the implementation — anti-shape **A-3**, found
and reported honestly by the agent who did it.

You cannot express that defect in phase space. A phase grid is anchored to the *cycle*; the
absolute clock is gone before quantization happens. So the ordering fix arrived for free,
and the test went back to `assert_eq!` — now with a second case on a 0.7 s clip, the exact
duration the Observed entry named as the one a pack author would trip over immediately.

*A test that was loosened because the implementation could not meet it, tightened by a
change that was not about it.* Worth remembering next time a tolerance looks like the
honest answer: sometimes it is a description of a shape that is about to be replaced.

## Per bone, and why that needed no precedence rule

Two anims can own one bone — the arms carry the gait's counter-swing *and* the idle
breath — and they have different cycles, so they have different `N`. The obvious designs
are all arbitration: whose rate wins, in what order, with what tie-break.

The user's ruling refuses the question. *"A weighted blend is a weighted blend, a single
owner is a single owner."* Blend the **time step**, weighted by the proportion of the anims
that own the bone. There is no precedence rule because it is interpolation.

The one implementation note that mattered: blend `Δt = duration/N`, **not** frames-per-cycle,
which is not a commensurable quantity across cycles of different lengths. And the reason it
never looks broken is arithmetic: every `Δt` in the world derives from the same target, so
they differ only by rounding. Measured on the biped at 4.5 m/s — legs **0.08298 s** (the
gait alone owns a bearing chain), arms **0.08316 s** (gait + idle), the idle clip's own
step **0.08333 s**. Three rates within 0.4 % of each other. The body is not one exact
snapshot and it does not need to be.

Ownership turned out to be the subtle part, and it is a property of the **anim**, not of the
current keyframe bracket. A clip that names a joint in one keyframe owns it for the whole
cycle — otherwise a bone's rate would flicker as keyframes came and went, which would be a
new kind of beat inside the fix for a beat.

## The firewall question, and why the answer is a pin rather than a guard

The slice's one open call: quantization must never feed back into anything sim-visible, or
two clients at different targets would resolve hits differently.

We went looking for the path. There isn't one — no crate depends on `dc-client`, and a
sampled pose's only consumers are bevy `Transform` writes. So there was nothing to guard,
and a guard on a consumer that does not exist is exactly the machinery-with-no-caller this
project keeps building by accident.

What shipped instead is a pin in **two directions**, because one direction is a note to
whoever already knows:

- **Upstream**, the unquantized target is asserted **bit-identical at 1, 6, 12, 30, 144 and
  1000 fps**. The client's setting reaches the *hold* and nothing else.
- **Downstream**, `CharacterState`'s wire field names are asserted to contain no pose, no
  phase, no frame. *That half fails the day B3 or B5 puts a pose on the wire* — which is
  precisely the moment the guard has to become real. The failure message says so, and names
  `AnimState::phase` as the thing such a consumer must read.

And the structural half, which is worth more than either: **the stepped phase stopped being
state.** It used to be latched in `AnimState` beside the Froude number because both rode one
global time grid. Under per-cycle quantization the phase's grid belongs to the cycle, so the
held phase is *derived on demand* from the live one. There is no stored quantized value for
a future sim consumer to reach for by mistake.

The Froude number is still latched, and the asymmetry is deliberate: phase is a cycle-domain
quantity that can be held as a pure function of the live value, while speed is a time-domain
signal whose hold needs memory — and the cycle grid cannot be its clock, because a body at
rest has a frozen phase and would never notice itself starting to move.

## Two things that cost time

**The perf regression, found by measuring rather than by suspecting.** The per-bone rate is a
real derivation — resolve ownership, blend, sample once per distinct rate — and it runs per
body per frame. The first honest implementation collected each bone's owners into a `Vec`
and blended the slice: **6445 ns** per body per frame against a **3312 ns** baseline, nearly
double. Replacing the per-bone `Vec` with a streaming accumulator got it to 4945; replacing
the `HashMap<&str, _>` with short name lists — a clip names two segments, a limb has three
joints, and `Vec::contains` on `&str` beats sip-hashing at those sizes — got it to **4036 ns**,
**+22 %**.

That is where it shipped, with the number in the test's own doc comment and the heir named:
ownership is constant per (plan, clip set) and only the gait's step moves with speed, so the
entire pre-pass is per-plan work being redone per body per frame. Hoisting it into
`BodyAssets` is a signature change, and this slice is an appearance change the user has not
ruled on yet.

*The baseline was measured, not remembered:* the pre-slice client sources were checked out
into the same worktree and the same test run. The doc comment above that test claimed the
cost sat *"~two orders below"* its 20 µs bound; it was 6× below when the claim was written.
A printed caption is a published claim the gate cannot check.

**And a self-inflicted one:** `git checkout <wip-commit> -- crates/dc-client/src`, used to
restore after the baseline measurement, silently reverted four uncommitted fixes made after
that commit. Committing before measuring would have cost nothing. The rule that already
exists — *commit WIP early and often* — is the one that would have caught it, and "early and
often" apparently needs to include *before you check anything out*.

## What is deliberately not here

- **The pack-declared per-body MAX target fps.** Ruled the same day, wants a `BodyPlan`
  field, so it rides the B3 wire window; filed in `stubs.md` with its heir. A cap composes
  with the client's setting as `min(...)` and needs no change to the kernel — a cap is just
  a different target.
- **Importance-weighted frame selection.** The user foreclosed it explicitly: a player who
  lowers their target can lose the full swing of a fast kick, and *"a hand-authored anim for
  low fps actions will beat an engine baked locomotion anim in those cases."* Not owed, not
  a gap.
- **The verdict.** The acceptance criterion is *the stout's hitch gone, the biped visually
  unchanged*, and that is a judgement about **motion** that no still frame and no test can
  make (corrections #77). The numbers are in and they are what the ruling predicted. The look
  is the user's, from the running game — `--anim-fps <n>` puts 6, 12 and 24 one relaunch
  apart.
