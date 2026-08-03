# The stout was the body that could not be sampled

*2026-08-03 · the derived-gait walk (member #1 slice two's acceptance) · stations with the
user live*

First walk since locomotion stopped being an authored clip. Three bodies on a cut stone
bench, then in lanes, then running. Four verdicts, two of them new findings the corpus did
not predict.

## Station 1 — rest (`0148-station1-rest-three-derived`; player feet (−33, 913.55, −5.5), yaw 0, pitch −0.08)

Biped, stout, longleg standing at their derived hips on flat stone. **User verdict: "the bob
is gone, they're just swinging their arms idly. soles are planted."**

That closes the thread journal/0130 opened as *"the hover looks bad… it is not a bob, it is a
hover"*, and it closes it by **deletion rather than correction** — `Keyframe.root_bob_m` left
the schema and root height became one `root_offset(posture, mode, phase)` read by both the IK
hip and the render root. Corrections #80's two-authority drift is now unrepresentable, not
patched.

## Station 2 — walking, and a framing mistake worth recording

Driven at intent 0.27 (1.215 m/s measured), all three in the walk band. The pose readback
showed `facing_yaw` and `yaw` identical at −1.5708 with `look_held: false` — the trunk-facing
split working, the walk-8 strafe gone by construction rather than by a driver remembering to
steer.

**But the station was badly built and the user said so:** all three walked the same line and
occluded each other. *"That makes it hard to study them."* Rebuilt as three lanes 6 m apart
walking toward the camera. Obvious in hindsight; recorded because a station that cannot
separate its subjects produces a verdict about the framing rather than about the sim.

**User verdict on the rebuilt station: "they genuinely look pretty good… the cadence
difference is very clear, i like it."** The cadence spread is the thing the whole tier exists
to produce — same ground speed, three leg lengths, roughly 1.8 steps of the stout to one of
the longleg's, and nobody chose the ratio.

**On the bob, which we had predicted would read high:** *"the bob may read a tad exaggerated
but — they're block people. it doesn't look weird for them."* The prediction was right and
the verdict is *accept*. We derive ≈ 6.6 cm against a published human 4.6 cm (Saunders, Inman
& Eberhart 1953) because the rigid compass model over-predicts; `bob_damping` is the knob and
B6 is its heir. **Predicting a walk verdict from the literature before taking the walk is the
measure-against-the-literature rule paying forward instead of backward** — judged on internal
consistency alone, 6.58 cm looks flawless.

## Station 3 — running, and the finding nobody predicted

Full intent, 4.5 m/s, Froude 2.35 / 4.69 / 2.02 — deep in the run regime, where the bake
**declines** to compute a root height at all rather than invent a flight-phase parabola it
cannot derive at density ≡ 1. The user accepted that caveat and then found two things by
watching.

### 1. The stout hitches, and it is ALIASING

> *"the stout in particular appears to have a weird hitch in its arm and leg oscillations,
> which the others don't, and my hypothesis is it has to do with the animation framerate being
> out of phase with the animation."*

The hypothesis is right and the mechanism is sharper than phase. `ANIM_FPS = 12.0` is
**fixed**, while cadence is now **derived per body** — so frames-per-cycle is no longer a
constant anybody chose:

| | cadence at 4.5 m/s | frames per cycle at 12 fps |
|---|---|---|
| biped | 1.72 Hz | 6.97 |
| longleg | 1.55 Hz | 7.73 |
| **stout** | **2.80 Hz** | **4.29** |

Shortest legs → fastest cadence → **fewest samples per cycle**. The biped lands at essentially
7, near-integer, so its sample points nearly repeat each cycle and read stable. The stout's
4.29 is both low and non-integer, so the samples drift through the cycle and beat against it.
Barely above Nyquist for the fundamental.

**This is anti-shape A-1 again, and it is the fourth instance this arc has retired.** Twelve
fps was chosen when there was ONE authored clip at ONE cadence, where "frames per cycle" was a
fixed number nobody had to think about. Derived cadence makes it body-dependent, and the
smallest fastest body aliases first. Same family as the four absolute-metre constants, the
world-global speed, and the hand-typed gravity.

**And it landed on a deferred call whose precondition had just arrived.** When the rotation
quantizer was removed (journal/0133) the 12 fps step was explicitly left alone, to be judged
*"on a body whose feet actually reach the ground."* As of this walk the feet reach the ground.

**The user's leaning, and their own insight into the only preserving fix:**

> *"we'll have to make a call on 12fps - probably we just forfeit it. the only way to preserve
> the bring-up claude's toy aesthetics suggestion would be to quantize per-animation per-body.
> (i think). and i just don't care enough."*

The diagnosis inside that is correct and worth keeping even if the aesthetic goes: **the fault
is quantizing per SECOND against a cadence that varies per BODY.** Quantize per *cycle* — N
poses per stride rather than N per second — and every body gets the identical stop-motion
look regardless of how fast its legs turn over, because the sampling is in the cycle's own
coordinate. That is the shape that would have made 12 fps survive derived gait. Recorded as a
**leaning, not a ruling** — the call is still owed.

### 2. Straight legs through the run

> *"legs are still quite straight during running, which looks like speed walking… no knee bend
> on kick-off from ground, just flapping straight legs."*

Confirmed, and it is the knob describing itself: `swing_flexion: 0.0`, whose own doc reads
*"the compass, **near-straight swing**."* Identity default, band to 1.0, **heir B6** — swing-leg
energetics and push-off are *force*, and a knee bending on kick-off is the leg doing work
against inertia. At density ≡ 1 there is nothing to compute it from.

**Standing back: the user found three of the four B6 stand-ins BY EYE in one session** — the
bob reading high, the missing push-off flexion, and the run's absent vertical. None of them
were pointed out first. That is an independent confirmation that the stand-ins were marked in
the right places, and the strongest argument yet that B6 is the next thing that matters rather
than a distant heir.

## The station-keeping finding: Claude time vs real time

> *"i blocked them off, by the way, because there's no way for me to tell you to stop them
> before they reach a ledge. claude time vs realtime."*

**The user built a wall.** The walk loop's whole premise is that Claude drives while the user
observes — but every intent commits several seconds of world motion before the driver can
react, and the observer has no channel at all. The wall is a workaround for a missing
affordance, and it will bite every future bodies walk identically. Candidates: a leash radius,
a duration- or distance-bounded intent (*walk 3 m then stop*), a dev freeze-all verb.

> blogworthy: two lenses. Lens 3 — a fixed sampling rate is a *stand-in that only reveals
> itself when the thing it samples stops being fixed*; the stout was undersampled the whole
> time and nothing could show it until cadence became derived. And lens 1 — the observer had
> to modify the world to compensate for the agent's latency, which is a real property of
> human-agent collaboration that no amount of tooling politeness hides.
