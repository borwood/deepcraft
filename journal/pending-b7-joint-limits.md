# The joint that had no front

*B7, slice one — joint rotation limits. Built 2026-08-03 against
`docs/audits/2026-08-02-joint-limits-b7-design.md`.*

The ruling was short: a body's joints should have ranges, the ranges should be
declared where a pack knows them and **derived** where it does not, and the
derived default should be good enough that *"a mutated body gets plausible
limits by construction."* The design pass that unpacked it was long, and it
did something unusual: it derived **eighteen numbers by hand**, from the
literals in `default_pack.rs` and `experiments.rs`, without running a single
cargo command — and labelled all of them *"a prediction the build checks."*

This is the build. Most of the predictions held to four decimal places. The
ones that did not held a better story than the ones that did.

## The obvious law returns zero

Start where the design pass started, because it is the part a reader will want
to re-derive and should not.

You want to know how far a knee can bend before the shank hits the thigh. The
obvious answer is: rotate the shank and ask when its box first intersects the
thigh's box. Try it on our actual geometry and you get **0°. On every joint. On
every plan.**

The reason is structural, and it is a fact about cuboid rigs rather than about
our authoring. A parent and a child box **abut at the pivot** by construction —
the shank's top face *is* the knee, the thigh's bottom face *is* the knee. Rotate
the child by any ε > 0 and its proximal corner sweeps a tiny arc that goes
straight into the parent. The bound is zero because contact is already happening.

Real joints do not do this, because bone ends are rounded and tissue is
compliant, and we model neither. Any exclusion radius that rescues the naive law
is **a tuning constant with no derivation** — the thing this project has spent
weeks removing. So the design pass proposed testing only the **distal** extent:
the far end of the chain, a whole bone-length from the pivot, where the
degeneracy cannot arise.

That is right, and it is what got built. But the pass wrote the law down three
different ways in three different places, and none of them reproduces its own
table.

## Three readings of one sentence

§ 3.1 says: *"Test the eight corners of the child subtree's **terminal box**."*

- **"Eight corners"** is the degeneracy above, wearing a smaller hat. The
  terminal box's *proximal* corners abut just as hard as anything else's. Test
  them and the biped knee comes back 0°, not 155°. Every row in the table is
  reproducible only with the **distal** corners — so the prose says eight and
  means four.

- **"Terminal box"** contradicts the table's own **hip** rows. The pass derives
  the biped hip from *"the thigh's distal corner vs the trunk's z = ±0.14"* —
  that is the **rotating segment's own box**, not the terminal one. And it has
  to be: the shank, the actual terminal box of the hip's subtree, swings so far
  out that it **clears the trunk's top before its z re-enters** and never
  collides at all. Drop the rotating segment's own box and *both* hip rows
  become `Undetermined` — taking with them § 3.2's headline finding (the stout's
  77° hip against the biped's 150°, *"the derivation doing real work on real
  data"*) and § 5.2's one concrete number handed to gait member #1.

- The **neck** rows go the other way. They are derived from the **head's**
  distal corner, with the neck's own box omitted. Include it — as the hip rows
  do — and it binds first, hard.

So the law had to be settled rather than transcribed. The one self-consistent
reading is: **the distal corners of every box in the rotating subtree, plus
declared role anchors, against every strict ancestor.** Under it the knee, hip,
elbow, shoulder and knee-lateral rows all reproduce to about 1e-4 radians —
which is the design pass's own hand-arithmetic slop, not a disagreement — and
the neck rows and `head` move.

The neck moves a long way. The pass predicted the biped's cervical pitch at
**141.42°**; the measured answer is **59.74°**, because the neck box's own top
corner reaches the shoulder line before the head's does. The stout, whose neck
is an 0.08 m slab, comes back at **36.03°**.

And `head` — which the pass called `UNDETERMINED` on the grounds that it *"has
no child chain to collide"* — is bounded at **126.87°**, because a segment's own
box is part of its own rotating subtree. That is the same clause that saves the
hip rows, read consistently.

## What the literature check did with that

The standing rule here is that whenever a simulated quantity has a real,
published counterpart, you measure against the literature at least once, because
a closed system cannot detect its own scale error. The design pass ran that
check on paper and found the pattern that matters:

> the derivation is good where the end-range is **BONY** and useless where it is
> **LIGAMENTOUS**.

Knee flexion derives 155.02° against a published 135–150° active / ~160°
passive: in band, from cuboids, with nothing fitted. Elbow flexion 155.72°
against 145–150°: 4% over. Hip *extension* derives 149.79° against a published
20–30° — **over by five to seven times**, because what stops a hip extending is
the iliofemoral ligament, and at density ≡ 1 we have no ligaments, no tendons, no
capsules and no force to model them with.

That conclusion survives the build intact and is better supported by it. The one
row that moves is cervical flexion/extension, and it moves *toward* the
literature: 59.74° lands **inside** the published 45–70° extension band, where
the pass's 141.42° was 3× over. Cervical lateral goes from 2.8× over to 1.3×.

(The published figures themselves are carried forward with the pass's own
disclaimer — from the assistant's knowledge, not network-verified. This build had
no network either, and adding a citation it could not check would be worse than
carrying an honest one.)

## The stout has no shoulders

The expensive find was not in the table. It was one line of prose the pass
dropped in parentheses and did not cost:

> (Stout: `[0.39, 0.55]` vs `[−0.39, 0.39]` — a zero-measure graze at
> `x = 0.39` exactly)

`dc:body/stout`'s upper arms are drawn **flush** against its trunk. Their inner
face and the trunk's outer face are the same plane. In exact arithmetic that is
a measure-zero touch and nothing happens. In f64, `0.47 − 0.39` is not `0.08`,
and the two shoulders — one at `+0.47`, one at the mirrored `−0.47` — landed on
**opposite sides** of the boundary.

Read as an entry, that is a rotation limit of **0°**. So the derivation welded
both of the stout's arms, and then the validator did exactly what it was built to
do: it rejected `dc:anim/biped_idle`, which rotates `arm_l_upper` by 0.06 rad, on
a shipped plan in the shipped pack. The first full test run said, in effect,
*your default content no longer defines.*

The fix is not an epsilon hunt, it is a definition. **A point already inside an
ancestor at rest never *enters* it.** The authored rest pose is the reference the
law measures departure from; two boxes drawn overlapping is the author's
geometry, not a joint's range, and reporting it as `[0, 0]` welds a limb for a
reason that has nothing to do with rotation. The contact test needs a length
tolerance to be stable — it borrows the resting bake's `EPS_M`, a nanometre,
whose derivation is already written down beside it ("exactly equal, up to
accumulated f64 rounding", against authored geometry that differs by millimetres
at least).

The stout's shoulders now come back `Undetermined` with the reason spelled out:
*the chain's distal extent already intersects this segment at REST — a graze in
the authored geometry, not a rotation limit.*

## The pole that could not be deleted

§ 5.3 promised the cleanest thing in the whole slice: `solve_leg_ik` picks its
knee pole with a hard-coded `ka.1 <= kb.1` — *"the knee pole points forward
(−Z), so knees bend like knees"* — an anatomical assertion living in a solver,
world-global and blind to the plan. With a declared one-sided knee the pole is
**determined** by the sign of the range, so the comparison is *"deleted, not
generalised."*

It is determined by the sign of the range. And § 3.4, four pages earlier, is the
finding that **we have no signs**: every box in every shipped plan is centred in
z, `pivot_m[2] = offset_m[2] = 0.0` on every segment, so the bodies are
mirror-symmetric fore-and-aft as well as left-and-right. **They have no front.**
Every derived range comes back exactly symmetric, both poles are admissible, and
deleting the comparison leaves nothing to choose with — which would move the
render on the shipped pack and fail the S-5 acceptance in the same stroke.

So the heuristic survives, demoted: the pole is **the candidate the declared
range admits**, and when both are admitted the pre-B7 forward convention breaks
the tie. That is a genuinely smaller thing than it was — it is now the fallback
of a read rather than the whole mechanism — and it disappears the moment a plan
says which way its knee folds. It ships with a `⚠ STAND-IN` marker naming its
heir.

The user's ruling on this is what makes it comfortable rather than embarrassing:
*"a 'knee' could bend backwards: it's not actually a knee until constraint is
declared."* An undeclared joint hinging both ways is not a hyperextending knee.
It is a joint.

## What the sector retires

The other half of § 5.3 is the part with a picture. `solve_leg_ik` clamps its
target into the annulus `[|l1 − l2|, l1 + l2]` — but `|l1 − l2|` was never an
anatomical bound, only the value that keeps `acos` in domain. The honest inner
bound is the distance at full flexion, and with a knee range it is computable:

| plan | `\|l1−l2\|` | `d_min` | ratio |
|---|---|---|---|
| biped | 0.020 | **0.19121** | 9.6× |
| stout | 0.010 | **0.23664** | 23.7× |
| longleg | 0.020 | **0.19174** | 9.6× |

Each is the fully-folded foot distance to 1e-9, asserted as geometry rather than
pinned as a magnitude.

And it retires journal/0131's degeneracy by construction rather than by patch.
The stout, crouching, drops its root 0.45 m from a 0.44 m hip — the sole target
is **below the ground**, `d = 0.010` — and the old clamp dutifully folded the
knee flat to a degenerate 180° and left ~10 mm of residual that nothing reported.
Under B7 that target is `Reach::BeyondFlexion`, the override is skipped, and the
clip pose stands: **the foot floats honestly instead of lying about contact.**

The sharp negative in the design is worth restating because it is the whole
reason the enum exists. *Solving freely and then clamping the knee into range is
forbidden.* That moves the foot off its target with no signal — a working call, a
passing test and a silently wrong output, which is exactly the anti-shape this
project keeps catching itself in.

## Two constants that were correct and homeless

`NECK_YAW_CLAMP_RAD = 75°` and `NECK_PITCH_CLAMP_RAD = 45°` had been sitting at
the top of `dc-client/src/body.rs` since the walk-8 orientation fix. They are
**anatomical joint limits** — 75° is inside the published 60–80° cervical
rotation band, 45° inside the 45–50° flexion band — implemented as world-global
constants that no body could disagree with. The stout's 0.08 m neck got the
biped's numbers.

They are now declared on each plan's `look` joint, and `resolve_orientation`
reads them. The migration is byte-identical: same numbers, different home, and a
body can finally state its own.

Then the mechanism immediately reported its own subject. The stout's declared
±45° pitch is **outside** its derived ±36.03° self-contact bound. That is
correct on both sides — a squat neck on a deep trunk genuinely cannot pitch that
far, *and* a declared bound wider than the derivation is reported rather than
refused (the gait docket's rule: a clockwork golem may want to bend wrong). So
the render is unchanged, the migration stays byte-identical, and the exact defect
§ 5.4 named in prose — *"the stout's short thick neck gets the biped's
numbers"* — is now a warning the game prints at plan-build time instead of a
sentence in a document.

That is the most satisfying thing in the slice. The A-1 entry did not just get an
heir; it got an instrument that points at itself.

## What is deliberately not here

Nothing is declared on the shipped pack's **knees**, and that is a sequencing
ruling, not an oversight. A declared one-sided biped knee rejects
`dc:anim/biped_walk` at five keyframes (it hyperextends both knees by **28.65°**
on every cycle, and has since bring-up) and `dc:anim/biped_jump` at one. Both
confirmed here, on a **fixture** plan. Land the declaration today and the default
pack fails to define and the game does not start. It lands with the gait slice
that retires the walk clip.

So slice one is a strict identity default: `dofs: None` everywhere but the neck,
zero rejections, and the solver bit-identical to its predecessor across a
2000-target sweep — everywhere outside `d_min`, where it differs on purpose.
Test 1 proves the mechanism is off; test 6 proves it is armed. Neither is worth
much without the other.

> blogworthy: **lens 2 (procgen against priors)** and **lens 3 (reflexions in a
> deepsim codebase)**. The three-readings-of-one-sentence problem is the sharpest
> illustration yet of *an audit's diagnosis is usually right and its prescription
> is a hypothesis* — the pass's law was ambiguous in a way that only building it
> could expose, and two of the three readings quietly delete its own best finding.
> The stout's flush shoulders are the other half: a parenthetical the author
> called "zero-measure" turned out to be the thing that broke the shipped pack,
> because f64 has no zero-measure sets. And the pole is a small, honest lesson in
> what "delete the heuristic" costs when the data it was standing in for does not
> exist yet.
