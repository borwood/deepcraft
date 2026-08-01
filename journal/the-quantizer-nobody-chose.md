# The quantizer nobody chose

*2026-08-01. Slug only — the integrator numbers this at merge.*

> blogworthy: **lens 1 (AI-native development)** and **lens 3 (reflexions in a deepsim
> codebase)**. A defensive measure an assistant proposed against a failure it *anticipated*
> hardened into a ratified aesthetic, was cited as an identity in a read-first design doc for
> a year, and cost the project its foot placement outright — and the only thing that ever
> falsified it was the user remembering whose idea it had been.

## What was removed

`ROT_QUANTUM_RAD`, `quantize_angle`, `stepped_angle`, and every call site: `sample_clip`,
`blend`, `resolve_orientation`, the IK-output path in `character.rs`, and the headless
retarget probe's re-implementation of that path. Joint angles are now exact radians.

That is the whole slice. Nothing was added. No flag, no config knob, no positional snap
standing in for the angular one, no compatibility shim. The 12 fps time step
(`ANIM_FPS = 12.0`, `quantize_time`) is untouched.

## How a workaround became an identity

`bodies.md` § *Stepped animation* has read the same way since 2026-07-19:

> Character animation renders **frame-stepped (~12 fps, quantized rotations)** — a
> stop-motion look chosen for the elevated-pixel aesthetic (an identity, not a workaround;
> it also happens to be cheap and forgiving).

Every session since has read that as ratified. It is a DECIDED heading in a read-first design
doc; you do not relitigate those. Three separate investigations this week — journal/0130,
journal/0131, corrections #80 — all worked *around* the quantizer rather than at it, because
the doc said the aesthetic stands.

The doc was wrong about its own provenance. The user, today:

> both the 12fps stepping and the rotation snapping were proposed by an earlier session to
> hide weird rotations it anticipated from IK solves … neither was my idea … we never
> actually saw weird IK solves.

They rolled with it. That is all "ratified" ever meant here.

**The tell was in the sentence the whole time, and it is the word *forgiving*.** An identity
is not forgiving of anything. Forgiveness is what a workaround offers: it is the property of
a mechanism that absorbs a failure you expect. The clause "an identity, not a workaround"
and the evidence that it was a workaround were **eleven words apart**, in the same sentence,
in a file read at the start of every session, for a year. This corpus's expensive failures
keep having exactly this shape — a claim sitting beside its own refutation — and this is the
tightest instance yet recorded.

## The premise was never true

journal/0131's probe ran the two-bone solver across 3 plans × 4 ground cases × 3 clips —
**1,056 samples**, the full time series, both feet, every frame. The knee angles it produced
were well-behaved: −56.2°, 90.0°, 123.7°. Legs bending like legs.

There is exactly one degenerate result in the whole set, a 180° solve, and it is not the
solver's. It comes from `CROUCH_ROOT_DROP_M = 0.45 m` consuming **97.8 %** of the stout
plan's hip height: sink a hip to 2 % of its standing height and the leg has nowhere to go but
folded flat. That is a bad constant. The solver, handed a reachable target, returns a correct
answer every time.

So the quantizer was hiding an instability that does not exist, and — this is the part that
cost real work — it was **destroying the answers the solver did produce**. One quantum of hip
rotation moves the biped's ankle 172 mm. The foot corrections at issue need 1.30°, 0.98° and
0.33°. The quantizer is 9×, 12× and 35× too coarse to express any of them. The IK solved
correctly and the rounding threw the result away.

## Why this is a subtraction and not a redesign

The obvious engineering instinct here is to keep the aesthetic and exempt the correction
chain: quantize the clip's angles but let the IK output through, or quantize the foot's
*position* against the ground instead of the joint's *angle*. `bodies.md` recorded both as
unratified options on 2026-07-30.

The user closed that door explicitly. New machinery here is *"an additional bug surface"* and
*"not necessary at this time"*. And the deeper reason is that we do not yet know what we
want: the 12 fps step is a live taste question, and it cannot be answered honestly by anyone
right now, because **a body whose feet do not reach the ground is not a body you can judge the
motion of.** Fix the feet first, watch it move, then decide. Building a selective-exemption
mechanism today would be committing to an answer in order to defer the question.

So the ruling is: *"12fps can ride until we have more opportunity for human to see action in
game and make a more informed taste call."* Subtract the thing that was never chosen; leave
the thing that is still being decided; add nothing.

## What the tests said, before and after

The gate held one test whose only subject was the quantum — `angles_are_quantized`, asserting
sampled angles land on the `TAU/32` grid. It is deleted. There is nothing left for it to be
about.

The two interesting cases were the tests that *tolerated* the quantizer, and both tightened
sharply:

- The retarget probe's invariant (4) bounded the achieved foot residual by **one quantum's
  arc at full extension**, `reach × ROT_QUANTUM_RAD` — about **172 mm** on the biped. That
  bound is now the solver's own annulus-clamp epsilon: `d` is clamped to `l1 + l2 − 1e-6`
  while a target is admitted out to `l1 + l2 + 1e-9`, so the reconstruction can miss by those
  two and by f64 slop and by nothing else. **~1 µm.** Five orders of magnitude, and it is the
  assertion that would catch a quantizer being quietly reintroduced.
- `orientation_is_stepped_and_splits_look_from_trunk` asserted the head aims at the look
  *within 1.5 quanta* — a 17° tolerance, which is not much of an assertion. Renamed to
  `orientation_splits_look_from_trunk_exactly` and tightened to `1e-12`: the neck now takes
  the whole delta exactly, and trunk + neck sums to the look exactly.

Both are derived bounds, not snapshots — the distinction CLAUDE.md § Gates insists on. The
old ones were derived too. They were derived from a constant that should not have existed.

## The shape

Anti-shape **A-2** — a justification that outlives its premise. The premise was *"IK solves
will look weird"*; it was never observed, and by the time it was measured false the
justification had been promoted to an identity in a read-first doc and was being cited by
sessions that had no way to know where it came from.

What retired it was not an audit. `spine-audit` compares docs to code and the code matched
the doc perfectly. The staleness sweep compares docs to newer work and the newer work
(0130, 0131, #80) all deferred to the doc. `doc-topology` compares docs to each other and
both halves of the contradiction were inside one sentence of one file. **The instrument that
worked was the user remembering who proposed it** — the fourth time in a week that a human
observation has beaten an instrument reading (corrections #77, #78, #80, and now this).

The lesson is not "audit provenance", which is unfalsifiable advice. It is narrower and
checkable: **when a design doc asserts that something is an identity rather than a
workaround, that sentence is doing defensive work, and defensive work implies a threat.**
Ask what the threat was. If nobody can name a time it fired, the mechanism is a guard against
an anticipation, and an anticipation is a hypothesis that was never tested.

## What this does not fix

corrections #80's defect is still live and still upstream of everything here:
`character.rs:219` builds the IK's hip as `feet.y + hip_local[1] - crouch_drop`, **excluding
`root_bob_m`**, while the renderer adds `root_bob_m` to the root at line ~257. The solver is
handed a hip that never moves, so it returns a correct and *constant* leg pose, and the
renderer then translates the whole body — feet included — by the bob. Removing the quantizer
removes the second wall. The first one is untouched, deliberately: this slice was
subtractive, and that is an addition.

Which means the honest statement about what the body looks like now is: **we do not know, and
a screenshot cannot tell us.** Whether joints read smoother or the same, whether the
crossfade reads differently, whether removing the angular grid changes the stop-motion
character at 12 fps — those are all motion questions, and corrections #77 is explicit that a
still frame is structurally blind to a temporal artifact. The next thing that happens here is
somebody watching it move.
