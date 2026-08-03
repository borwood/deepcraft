# The gait bake (member #1) — a design pass against the posture-gait bones

> **⚠ USER CALL #1 RULED 2026-08-02 — mutable header; the body below is testimony and is not
> rewritten.** *"Option A for sure! When we have controller support an analog stick can
> actually grade intent up the ladder."*
>
> **THE SPEED CONSTANT IS MISLABELLED, NOT WRONG.** `walk_speed_m_s = 4.5` stays as the
> **top** speed — it is a deliberate voxel-traversal game-feel choice and traversal feel is
> not being paid for fidelity. What changes is that the mover gains a **speed RANGE** and the
> gait is **selected by Froude across it**: low intent walks, full intent runs. The constant
> is re-typed, not lowered. Its name is now the lie and goes with it.
>
> **THE LADDER IS CONTINUOUS, AND THE ANALOG CHANNEL ALREADY EXISTS END TO END** (user's own
> extension, and it is a design fact this pass did not have). `SetMoveIntent.speed` is already
> *"a fraction of full walk speed, clamped to [0,1]"* — a real analog magnitude, carried
> through the one door, receipted and replayed. A controller stick maps to it with no new wire
> surface. **What throws that magnitude away is exactly the machinery this pass proposes to
> retire:** `WALK_SPEED_THRESHOLD_M_S = 0.35` (`body.rs:49`) and the two-state `enum Loco`
> (`:199`) collapse a continuous intent into Idle-or-Walk at `:240`. So the analog ladder is
> **not future work gated on controller support** — it is unblocked by this ruling, and the
> binary switch is now a *named artifact with a named replacement*, not merely an engineering
> nit. **Grade the gait continuously over Fr; never re-introduce a discrete gait switch.**
>
> **The known cost is unchanged and rides:** G5 — a run's flight phase is not cleanly
> derivable at density ≡ 1, so it waits on B6 (per-segment materials) or takes an interim.
> Name the interim as a stand-in with B6 as its heir, per the ratified docket's (c).

> **⚠ USER CALL #4 (G1) RULED 2026-08-02: THREE KEYFRAMES.** *"I agree with A."* The bones'
> two-keyframe count was assistant-originated and does not survive a real leg; the count
> changes and the mechanism does not. The user's own sparse-keyframes-with-the-server-between
> formulation is untouched — three is still sparse.
>
> **⚠ AND THE USER OPENED A NEW ITEM, WHICH IS STRONGER THAN THE FIX IT COMMENTS ON:**
> *"frankly, hyper-extension just shouldn't be possible if we set rotation limits on joints."*
> **Verified at source, 2026-08-02: JOINT ROTATION LIMITS DO NOT EXIST ANYWHERE.**
> `SegmentDef` (`dc-api/src/bodies.rs`) carries `name, parent, pivot_m, size_m, offset_m,
> tint, roles` — **no rotation range**; B0 landed roles and stopped there. `solve_leg_ik`
> (`dc-client/src/body.rs:485,493`) clamps only to keep `acos` in domain and the target inside
> reach — **numerical guards, not anatomical ones**. No plan, no solver and no validator
> prevents a knee inverting. **This is a missing PRIMITIVE, not a gait detail**, and it is
> upstream of this member: the bake, the IK solver, the validator, clip validation and every
> evolved body all want it. Scoped as its own thread; not folded into this pass silently.

> **⚠ USER CALL #2 RULED 2026-08-02: `Keyframe.root_bob_m` LEAVES THE SCHEMA.** *"A, remove
> it while it's still a recompile."* Not kept dormant (a field nothing reads is the stub this
> corpus catalogs — someone sets it, something reads it "just for jump", and the two-authority
> condition that produced corrections #80 is back), and not kept as a per-clip override (the
> clip/gait split already routes one-shots).
>
> **⚠ SEQUENCING, stated because the ruling is a WHAT and not a WHEN, and the difference has
> already cost this arc once:** the removal **rides with the derived `root_offset(posture,
> mode, phase)` in the same slice**. Deleting the field on its own is an unratified appearance
> change with no replacement behind it — the exact option the user declined earlier the same
> day (*"A it is"*, ride as-is until this pass ruled). **The deadline is real and is the
> window the user named: it is a recompile while nothing persists a clip, and a WIRE MIGRATION
> once B3 makes the pose a versioned sim asset** — the same deadline B7 and `stubs.md` #34
> ride. Three fields die together and none of them alone: the walk bob **retires with its
> clip**, the jump bob is **deleted as a double authority with the mover**, the idle bob is
> **deleted as a root translation** (breath belongs in joints).

> **⚠ USER CALL #3 RULED 2026-08-02: `dc:anim/biped_walk` is RETIRED as shipped content and
> parked as a test fixture** — *"Sure, A. Mostly because it's not worth doing anything with."*
>
> **The user's reason is recorded verbatim because it is NOT this pass's reason, and the
> difference governs what may be built on it.** This pass argued the clip has evidentiary value
> (a measured human-authored attempt to check the derivation against). The user ruled A on the
> narrower ground that **removing it is not worth the effort** — parked, not enshrined.
> **So: no comparison harness, no "derivation agrees with the animator to within X" acceptance
> test, no ceremony around it.** It stops being content, it stays in the tree, and anything
> further wants its own ask. *(The evidentiary framing above is the assistant's; attributing it
> to the user would be corrections #86's error repeated.)*

> **⚠ USER CALL #5 (G7) RULED 2026-08-02: THREE QUANTITIES, AND TRUNK FACING SPLITS LIKE THE
> GAZE.** *"Agreed on (A) reasoning. Good pattern to stick with in bodies."* The sim owns the
> **target** facing (derived from travel or a held look — deterministic, replay-safe); the
> client keeps the **turn rate** toward it. G7's finding stands: corrections #94 split one noun
> into two and the real count is **three**, the third being where the walk-8 strafe lived.
>
> **AND THE RULING WAS WIDENED BY THE USER INTO A STANDING PATTERN FOR THE ARC** —
> *the sim owns the target; the client owns the approach* — now recorded as its own section in
> `bodies.md` with its three instances (gaze, trunk facing, pose) and the failure mode it
> forbids by name (two derivations of one quantity). **Whether it is a corpus-wide spine is
> filed and NOT assumed** — the user scoped it to bodies.

> **✅ BUILD GREENLIT 2026-08-02 (user): FIRST SLICE IS THE HEADLESS BAKE.** *"Go with your
> rec, whatever gets us from point A to point B fastest without trying to preserve any interim
> states indefensibly. We're in progress."*
>
> **Slice one: `bake_gait` in dc-api, headless.** No renderer change, no schema change, no clip
> retirement, nothing visible — the member #0 precedent, whose first slice was headless and
> whose consumer slice came after and got walked. **Acceptance is the predicted table**
> (§ 2), falsifiable before a frame is drawn, as member #0's was and met to 1e-9.
> **Slice two is the consumer** and it is where `root_bob_m`, `biped_walk`, the binary `Loco`
> switch and the trunk-facing split all land together — and it **owes a WALK**, because every
> ruling here is a judgment about motion and corrections #77 is explicit that a still frame
> cannot see a temporal artifact.
>
> **⚠ OVERRULED BY THE USER WITHIN THE HOUR — corrections #96. THE PARAGRAPH BELOW IS
> PRESERVED AS DATED TESTIMONY AND ITS CONCLUSION IS FALSE.** *"you can ship a broken main
> when the very next slice fixes it. Nobody is spawning a body and driving it except you and
> I."* **Never-break-main is a real principle imported at the wrong SCALE** — it prices a
> hazard belonging to a project with users, CI consumers or contributors pulling between
> slices, and this one has two participants who are both in the conversation that sequences
> the slices. **B7's declarations and the clip retirement MAY land ahead of the derived
> gait.** The sequence below is a **preference with a cost** (a bigger revert surface, more
> at once under a single walk verdict) — *not a wall*. **Second assistant-invented constraint
> in this banner presented as a property of the problem**; the other is the reversed B7
> ordering two paragraphs down, which this banner already catches. The rule recorded in #96:
> *a sequencing constraint must name whom it protects — if that is a user who does not exist
> yet, it is a preference offered with its cost.*
>
> ~~**THE USER'S PRINCIPLE GOVERNS THE SEQUENCING AND IS NOT A LICENCE TO BREAK MAIN.** *"Without
> trying to preserve any interim states indefensibly"* retires the instinct to shim bring-up
> content — the authored clips are **retiring, not being defended**. It does **not** license
> landing B7's declarations (or the clip retirement) before the derived gait exists: two of the
> pack's three clips fail a declared knee limit (B7 § 5), and `biped_walk` dying before its
> replacement leaves every body moving with no locomotion at all. **That is a broken main, not
> a preserved interim.**~~
>
> **⚠ AND THE SEQUENCE THIS PASS INHERITED IS REVERSED.** The board said B7 was upstream of this
> build. B7's design pass returned with **seven open user calls including its representation**,
> while this pass has **zero** — so the gait bake, fully specified, goes FIRST, and B7's calls
> are settled in parallel. **The dependency is additive, not blocking:** the bake's
> refuse-loudly-on-limit-violation enforcement point wires up when B7 lands. *An unspecified
> slice is not a blocker; it is a queue.*

**Status: DESIGN PASS. Nothing else here is ratified and no code was changed.** The deliverable is
this document. It works `posture-gait.md` § 7 member 1 — **including its user-ratified expanded
docket (a)–(e), 2026-08-02** — against the ratified bones (§§ 2–6) and the post-member-#0 code,
and proposes a concrete shape. Proposing and building are different acts; the second is the
user's call. **No cargo command was run** (a parallel session holds the build slot); every
number below is hand-derived from source literals and is a *prediction the build checks*.

**Read with:** `posture-gait.md` (bones §§ 2–6; § 7 member 1's ratified docket; the § 3 and
§ 5 corrected banners) · `docs/audits/2026-08-02-posture-bake-member0-design.md` + its header
rulings (member #0, **built and consumed** — journal/0137, 0140 (0138→0140 renumber)) ·
`docs/audits/2026-08-01-body-plan-structure-design.md` § 4 (the measured clip decomposition) ·
`bodies.md` §§ Postures, Stepped animation, Who owns the look, Clothing, Body plans, Sockets
(**§ IK is CLOSED — corrections #81 — and nothing below re-presents it**) · `stubs.md` § 34 ·
`dependency-graph.md` § 2b · corrections **#77 #78 #80 #81 #86 #93 #94** · spines S-3 S-5 S-9,
A-1 A-2 A-4 · journals 0130–0133, 0135, 0137, 0140, 0142.

**Provenance convention** (the 2026-07-25 rule, as member #0): mechanisms are marked
**[user-ruled]** (derived from a ratified ruling), **[bones]** (stated in posture-gait §§ 2–6),
**[docket]** (stated in § 7 member 1's user-ratified expansion — *data, not a hypothesis this
pass may reconcile away*), or **[assistant-proposed]** (this pass's own; a hypothesis until
ratified).

---

## 0. The proposed shape, in five sentences

A pure function in **dc-api** — `bake_gait(plan, key)` — takes a `BodyPlan` and the ratified
bake key `(species, posture-id, mode, yaw)` and returns a **`GaitVector`: three derived
keyframe poses per chain (`neutral`, `contact`, `clearance`), a per-limb `(phase, amplitude,
duty)` triple, and closed-form COEFFICIENTS over dimensionless speed** — never scalars frozen
at one speed, because speed is deliberately *not* a key axis and every gait term is a function
of it (finding **G2**). `neutral` **is** member #0's derived resting pose (confirmed, § 3.2,
with one extension the bake needs — **G3**); the two extremes are derived from stride geometry
and a published clearance band, not authored. Cadence, stride, duty and hip excursion come from
the Froude/dynamic-similarity chain (Alexander 1976; Alexander & Jayes 1983), so *"a big animal
takes slow steps"* is arithmetic and the bake can **report** when an authored modifier pushes an
output out of the published band. **The bob is not a term anywhere**: root height becomes a
single function `h(phase)` of the stance chains, read once and consumed by both the IK hip and
the render root — which makes corrections #80's two-expression drift *structurally impossible*
rather than merely fixed, and yields **zero bob by construction** for a body whose contacts never
alternate. And `stubs.md` § 34's heir lands here as **one `BindTarget` type across all four
binders**, because the gait bake does dissolve the many-to-one case *for locomotion* and does
not touch it for clothing, sockets or upper-body clips.

---

## 1. The honest input inventory — the boundary, stated BEFORE anything is derived [docket (a)]

The bake's entire input set today is **segment geometry plus volume as the mass proxy at
density ≡ 1**. Verified against the code, not assumed: `SegmentDef` carries `pivot_m`,
`size_m`, `offset_m`, `roles`; `composition` is *deliberately absent* with B6 named as heir
(`bodies.rs`, the DELIBERATELY-ABSENT comment). There is no strength, no muscle, no weight, no
metabolic term, no actuation cost. `CharacterConfig.walk_speed_m_s = 4.5` is a world-global
constant that knows nothing about the body (`dc-api/src/character.rs:63`).

| quantity | honest today? | why |
|---|---|---|
| cadence ∝ √(g/L), stride, phase offsets, hip excursion | **yes** | pure length scale + gravity |
| stance-phase root height `h(p)`, hence the walking bob | **yes** | rigid-chain geometry under a contact constraint |
| centre of mass, base of support, balance | **yes** (density ≡ 1) | member #0 already computes it |
| swing-leg pendular period (the ballistic cross-check) | **yes** | inertia is geometry × uniform density |
| **the Froude number's SPEED term** | **no** | speed comes from the mover, which is a world-global constant (**G4**) |
| **duty's dependence on speed** (the β(Fr) curve's shape) | **no** — band-anchored only | a fit, not a mechanism; the endpoints are published, the interior is not |
| **swing-knee flexion depth** | **no** | driven by swing-leg energetics = actuation cost |
| **stance-knee flexion / pelvic list** (the bob reducers) | **no** | joint stiffness = force |
| **any run** (duty < 0.5, a flight phase) | **no** | ballistic apex needs leg stiffness and takeoff force |
| effort minimisation, acceleration, load carrying, transition energetics | **no** | force |

**This bounds what a derived gait may CLAIM.** Everything in the "yes" rows is evidence.
Everything in the "no" rows is either a **band report** (the bake says the number is outside the
published range and proceeds) or a **stand-in knob with B6 as its named heir** (§ 5). The bake
never fabricates a force answer and never silently substitutes one.

---

## 2. WHAT is computed — outputs, units, consumers

Per bake key, from declared inputs only. Units doctrine inherited from member #0 [user-ruled,
decision 24]: **ratios and angles, never metres**; metres are re-derived at consumption from
the plan's own lengths.

| output | derivation | units | consumed by |
|---|---|---|---|
| **`neutral` pose** | member #0's zero-torque solve, extended to non-bearing chains (**G3**) | radians per joint | the sampler; the IK's base pose |
| **`contact` pose** | `neutral` rotated at the chain's attachment joint by the derived hip excursion θmax | radians per joint | stance arc endpoints (±) |
| **`clearance` pose** | chain solved so the contact anchor sits at ground + the clearance ratio at mid-swing | radians per joint | swing arc apex |
| **phase offset**, per bearing chain | contact-anchor geometry + the gait type's pattern (§ 3.3) | cycle fraction, mod 1 | the sampler |
| **amplitude**, per chain | **signed**, phase-driven traversal toward `contact` (**G6**) | dimensionless | the sampler |
| **duty** β, per chain | β(Fr) anchored on the published walk/run transition (§ 3.4) | fraction of cycle in stance | the sampler; contact scheduling; **B5's limp** |
| **cadence / stride coefficients** | `k_f = √(g/L)/2.3`, `k_λ = 2.3·L` (§ 3.1) | s⁻¹ and m, evaluated at Fr | the phase clock (**B3 owns the clock**) |
| **`h(phase)`** — root height | stance-chain geometry under the contact constraint (§ 4) | ratio of chain reach | **the ONE vertical composition** — IK hip and render root |
| **band report** | every derived term against its published band, with the citation | text + numbers | the author; the test report |

**Explicitly NOT computed here:** the sim-side phase tuple and who advances it (B3/member 2);
the 20 Hz vs 12 fps cadence question and the 12 fps taste call (deferred to the user, on a body
whose feet reach the ground); derived colliders (B4); per-segment damage (B5); anything behind
the § 1 "no" rows.

---

## 3. THE MATH, concretely

### 3.1 The scaling chain — four published relations, composed [bones prior 4 + docket (a)]

Let **L** = the governing stance chain's reach (member #0's `root_height_m`: 0.880 / 0.440 /
1.020 m for biped / stout / longleg), **v** = ground speed, **g** = 9.81 m/s².

1. **Froude number** `Fr = v²/(gL)` — the dimensionless speed. *Alexander (1984), "The gaits of
   bipedal and quadrupedal animals", Int. J. Robotics Research 3:49–59.*
2. **Relative stride length** `λ = 2.3·L·Fr^0.3`. *Alexander (1976), "Estimates of the speeds of
   dinosaurs", Nature 261:129–130* — the trackway regression, the single most-cited
   length-scale law in gait.
3. **Cycle period** `T = λ/v`, hence **cadence** `f = 1/T = √(g/L)·Fr^0.2 / 2.3`.
   **The bones' `f ∝ √(g/L)` falls out exactly**, with a weak `Fr^0.2` speed term — so the
   claim is not merely asserted, it is a corollary of (1)+(2).
4. **Duty factor** β: `β > 0.5` walking, `β < 0.5` running, transition at **Fr ≈ 0.5**.
   *Hildebrand (1965, 1989)* for the taxonomy; *Alexander & Jayes (1983), "A dynamic similarity
   hypothesis for the gaits of quadrupedal mammals", J. Zool. 201:135–152* for β as a function
   of Fr alone under dynamic similarity.

**The interior of the β curve is the one fitted thing here, and it is flagged as such.**
Proposed interim [assistant-proposed]: `β(Fr) = 0.5·(0.5/Fr)^0.263`, with the exponent set by
the two anchors that ARE published — β(0.5) = 0.50 (the transition, by construction) and
β ≈ 0.60 at Fr ≈ 0.25 (normal human walking, *Winter, Biomechanics and Motor Control of Human
Movement*). **This is a stand-in, not a mechanism**; it gets a `stubs.md` entry (§ 5, S2).
*The citations above are from the assistant's knowledge and were not network-verified in this
pass; the build should check the two exponents (0.3 and the β anchors) against the sources and
record any divergence as a finding about this document.*

**A free cross-check the bake can print and must not fit to** [assistant-proposed]: the swing
leg's natural pendular period, `T_nat = 2π√(I_hip/(m g d))`, is pure geometry at density ≡ 1.
Hand-derived for the biped's straight leg (upper box 0.18×0.45×0.20 at 0.225 below the hip,
lower 0.16×0.43×0.18 at 0.665): `m ∝ 0.028584`, `d = 0.4156 m`, `I_hip = 6.848e-3` →
**T_nat = 1.523 s**, so a purely ballistic swing (*Mochon & McMahon 1980, "Ballistic walking",
J. Biomech 13:49–57*) would take **0.761 s**. The Froude chain wants a swing of
`(1−β)·T = 0.40·0.909 = 0.364 s` — **2.1× shorter**. That is not a defect in either model: it
is the published finding that real walking is not ballistic above a slow walk, hip flexor
torque drives the swing, and **the discrepancy is precisely the missing FORCE term** of § 1.
The bake **reports the ratio**; it does not tune anything to close it.

### 3.2 Where `neutral` and `extreme` come from — CONFIRMING member #0, and one more pose

**`neutral` = member #0's derived resting pose. CONFIRMED, not merely adopted.** The reason is
geometric, not convenient: in a walk cycle **midstance is exactly the configuration in which
the stance chain is a vertical column under its attachment joint** — which is the
zero-torque configuration member #0 solves for, and for the three shipped plans is the identity
rotation with the root at chain reach. `neutral` is therefore per-`(posture, mode)`, which the
ratified key already carries, and a crouched walk's neutral is the crouch bake's — no new axis.
The consistency check that makes this more than a coincidence: **where member #0's answer is
honestly wrong (the bird's tendon-held folded rest, its § 2.4), the gait's `neutral` is wrong in
exactly the same way and behind exactly the same effort seam.** One degradation, not two.

**⚠ The bones' TWO keyframes are 2/3 right — the pass's loudest finding (G1).** A naive
ping-pong `neutral ↔ extreme` **cannot express a limb cycle**: stance and swing are different
waveforms on the same joints — the hip oscillates about neutral (signed, roughly sinusoidal)
while **the knee flexes only in swing and does not hyperextend**. Under one (neutral, extreme)
pair with a multiplicative amplitude, the knee is driven negative for half of every cycle. The
shipped clip proves the asymmetry: `leg_l_lower` runs `−0.15, −0.05, +0.5, +0.2` — not a
sinusoid about zero (body-plan-structure § 4).

**Proposed resolution, which preserves everything user-originated** [assistant-proposed]: a
cycle has **two turning points, not one**, so the bake emits **three derived poses per chain**:

| pose | what it is | derived from |
|---|---|---|
| **`neutral`** | midstance / the zero-torque column | member #0's bake (+ **G3**) |
| **`contact`** | touchdown = liftoff mirrored: the chain rotated at its attachment joint by **±θmax** | `sin θmax = s/(2L)`, s = λ/2 |
| **`clearance`** | mid-swing: chain solved so the contact anchor sits at ground + clearance | 2-bone IK (`solve_leg_ik` already ships) + the clearance ratio |

Traversal: **stance** (duration β) is `−contact → neutral → +contact`, a signed ping-pong about
neutral — *exactly the bones' mechanism*; **swing** (duration 1−β) is
`+contact → clearance → −contact`. **The user's sparse-keyframe design is preserved verbatim**:
the sim is still "between which keyframes, and how far", with three stored poses instead of two.
*This deliberately does NOT generate the stance arc procedurally — a generated arc would leave
nothing for the sim to be "between", which would narrow a user-originated formulation to save a
keyframe. The bones' "two keyframes" is assistant-originated; the user's between-keyframes
formulation is not; where they collide, the user's wins.*

For a vertical-column stance chain, `contact` is exactly `neutral` rotated by θmax at the hip —
so the third pose costs nothing for our three bodies and earns its keep the moment a chain's
touchdown is not a rigid rotation of its rest (a bird, an alligator's sprawl).

**`clearance` needs one number physics does not pin: the foot-lift margin.** Published anchor:
minimum toe clearance in level human walking ≈ **1.3 cm** (*Winter 1992*), ≈ **1.5 % of L**.
This is the bones' own docket (b)1 example — *"foot lift above minimum clearance"* — and it is
a **kind-1 taste knob** with the published value as the identity default, expressed as a ratio
of L so it survives scaling.

### 3.3 Phase offsets — derived from contact geometry, selected by gait type [assistant-proposed]

The producer already anticipates this: `default_pack.rs`'s mirror comment states *"the gait
bake never needs to know two limbs are partners (phase assignment derives from contact
geometry)"*. Made concrete:

1. **Partition the mode's bearing contacts into girdles** by clustering their anchor **z**
   (fore-aft) coordinate. Biped: one girdle of 2. Quadruped: two girdles of 2.
2. **Order within a girdle** by anchor **x** (lateral), then z, then y, then plan order — a
   total order derived from declared geometry, never from names (B0's rule: laterality,
   fore/hind and pairing are DERIVED).
3. **The gait type is a phase pattern over (girdle, index)**, and that is *all* a gait type is
   (Hildebrand). Default with none authored: offsets `i/n` within a girdle, girdles offset by
   `1/(2·n_girdles)`.
4. **Non-bearing limbs take the contralateral bearing chain's phase** — counter-swing.

**Zero-parameter prediction, confirmed by shipped data.** For all three plans (one girdle, two
contacts) the rule gives offsets **{0.0, 0.5}** — and the authored walk clip measures
`leg_r(t) = leg_l(t + 0.5)` *exactly, at every key, on both bones*, with
`arm_r(t) = mirror_z(arm_l(t + 0.5))` (body-plan-structure § 4). Rule 4 predicts the arms are
counter-phase to the same-side leg; the clip has `arm_l_upper = −0.5` where
`leg_l_upper = +0.6`. **Both hold. Nothing was fitted.**

### 3.4 Duty, and the one coupling that is not free [bones § 6]

Duty is per-limb but **not independent**: a biped's double-support fraction is `β_L + β_R − 1`,
which must stay ≥ 0 or the gait is a different gait. Per the bones, an injury delta is a **bias
plus deterministic renormalisation, never a per-limb override**. Concretely
[assistant-proposed]: given the derived mean `β̄(Fr)` and per-limb biases `b_i`,

```
β_i = clamp01( β̄ + (b_i − mean(b)) )      // mean-preserving by construction
```

then one re-centring pass if a clamp fired. Mean preservation is the right invariant because
**β̄ is what Fr pins** — cadence and stride are unchanged by a limp, which is why a limp reads
as *timing* and not as a different speed. A 3 % limp is `b = ±0.03` and must stay expressible
(bones § 6 property 3: f64 throughout, no quantization).

### 3.5 The predicted numeric table — checkable at build, cited, never fitted to a look

Hand-derived from the plan literals (`default_pack.rs`, `experiments.rs`) via § 3.1. `g = 9.81`.

**Table A — the derived comfortable walk, at Fr = 0.25 (mid-walk).**

| | L (m) | v (m/s) | stride λ (m) | step s (m) | cycle T (s) | cadence f (Hz) | duty β | θmax | bob Δ (m) | Δ/L |
|---|---|---|---|---|---|---|---|---|---|---|
| `dc:body/biped` | 0.880 | 1.4691 | 1.3353 | 0.6677 | 0.9090 | 1.1002 | 0.6000 | 0.3891 rad (22.29°) | **0.0658** | 7.475 % |
| `dc:body/stout` | 0.440 | 1.0388 | 0.6677 | 0.3338 | 0.6427 | 1.5559 | 0.6000 | 0.3891 rad | **0.0329** | 7.475 % |
| `dc:body/longleg` | 1.020 | 1.5816 | 1.5478 | 0.7739 | 0.9786 | 1.0219 | 0.6000 | 0.3891 rad | **0.0762** | 7.475 % |

- **Dynamic similarity is visible in the table and is the check on it**: at equal Fr, θmax, β and
  Δ/L are **identical across the three bodies** — only the metres differ. Alexander & Jayes'
  hypothesis reproduced by our own arithmetic, and the strongest available evidence that these
  outputs are ratios wearing metres, exactly as decision 24 requires.
- **Cadence ratios ARE the √(g/L) law, exactly** (the `Fr^0.2` term cancels at equal Fr):
  `f_stout/f_biped = √(0.880/0.440) = 1.414214`; `f_biped/f_longleg = √(1.020/0.880) = 1.076750`
  — assertable to 1e-12, not merely to the table's printed places.
- **The 0.040 m authored bob is 4.5 % of the biped's L and 9.1 % of the stout's** — against a
  derived **7.475 % for both**. The A-1 defect measured against its replacement.

**Table B — the same three bodies at the shipped world-global `walk_speed_m_s = 4.5`.**

| | Fr | regime | λ (m) | T (s) | f (Hz) | β | θmax | compass Δ (m) | Δ/L |
|---|---|---|---|---|---|---|---|---|---|
| biped | **2.3457** | RUN | 2.6139 | 0.5809 | 1.7216 | 0.3330 | 47.95° | 0.2906 | 33.0 % |
| stout | **4.6914** | RUN | 1.6091 | 0.3576 | 2.7967 | 0.2775 | 66.10° | 0.2617 | 59.5 % |
| longleg | **2.0237** | RUN | 2.8985 | 0.6441 | 1.5525 | 0.3462 | 45.27° | 0.3021 | 29.6 % |

**⚠ FLAGGED, NOT RULED — the docket asked for exactly this.** At 4.5 m/s **all three shipped
plans sit above the published walk/run transition**; the literature calls it a run, and duty
falls below 0.5 — which means a **flight phase**, whose root height is ballistic and therefore
**not derivable at density ≡ 1** (§ 1). The honest bake behaviour on day one, for every shipped
body, is *report out-of-band and decline the run bob*; the compass Δ column is printed **out of
domain** and is why (a 0.26 m excursion on a 0.44 m body is a model past its edge). Three
dispositions exist and **this pass chooses none**: (i) a bring-up artifact that B4's
body-derived speed retires; (ii) deliberate genre taste — 4.5 m/s is within a whisker of
Minecraft's walk speed — in which case the default biped genuinely runs and should *look* like
it; (iii) speed becomes a per-mode band with a species-declared gait-type transition ((b)2).
*Both directions of `placeholder-state-is-not-intent` apply: this pass neither argues the design
from 4.5 nor silently preserves it.*

**Table C — the shipped `dc:anim/biped_walk`, decomposed as the cross-check.**

| measured from the clip | value | the derivation says |
|---|---|---|
| cycle period | 1.000 s (fixed, speed-independent) | 0.8311 s at the clip's own implied speed |
| hip excursion | +0.60 / −0.50 rad (asymmetric) | ±0.4488 rad = 25.71° (symmetric) |
| implied step length | 0.9188 m | 0.7636 m |
| implied ground speed | **1.8376 m/s → Fr 0.3911** | in-band: a brisk walk, below the 0.5 transition |
| duty | — (a clip has none) | β = 0.5334 — a walk, correctly |
| authored `root_bob_m` | 0.040 m, period 0.500 s = **2× the limb cycle** | 0.0871 m; **the 2× is reproduced exactly, by construction** |

Three findings fall straight out, and they are the strongest corroboration in this pass:

- **The authored clip is a physically coherent walk at Fr ≈ 0.391 and sits 20 % above
  Alexander's stride regression** — inside the published scatter. The clip is not wrong; **the
  pairing is**: the mover runs it at 4.5 m/s, so the feet supply 1.84 m/s of the travel and
  **2.66 m/s is skate**. (`AnimState::advance` increments `clock_s += dt` with no speed term at
  all — `dc-client/src/body.rs:237`. The clip period is a constant.)
- **The clip's own geometry demands a 0.154 m bob and authors 0.040 m** (peak `h = L = 0.880`;
  trough pinned by the front leg at `L·cos 0.6 = 0.7263`). The authored bob is **26 % of what
  the clip's own leg swing requires** — that gap *is* the hover, arriving from a third,
  independent direction (cf. corrections #77's measured `[+0.020, +0.035] m`).
- **The clip is internally inconsistent by 46 mm.** At double support the front leg pins
  `h = 0.88·cos 0.6 = 0.7263` and the rear pins `h = 0.88·cos 0.5 = 0.7723`. **Two contacts
  demanding two different hip heights at the same instant**: one foot must be 46 mm off the
  ground no matter what the root does. Hand-typed asymmetry, invisible until the geometry was
  written down — and structurally impossible under a derived, phase-offset gait, which is § 4's
  entire argument for why this is a bake and not a better clip.

---

## 4. The BOB, as an OUTPUT — read corrections #93 first [user-ruled disposal, routed here]

Corrections #93 is unambiguous: the authored bob has **no contemporary sketch or ratification**,
the user has said so twice, and *"whether any body bobs, which, and how much"* are per-species
questions § 1 already routes to the bake. **This pass does not propose wiring the authored bob
anywhere.** An earlier session did and it was falsified; that proposal is not repeated.

### 4.1 What vertical root motion each (body, mode) produces — including none

**The derivation.** During stance the contact anchor is pinned and the chain is a rigid link, so
the attachment joint traces a circle: `h(θ) = L·cos θ`, peaking at midstance (`h = L`) and
troughing at the contact extremes (`h = L·cos θmax`). With several chains, `h(p)` is the
**tightest** constraint over all chains in contact at phase `p`. Amplitude
`Δ = L(1 − cos θmax)` with `sin θmax = s/(2L)`; frequency = (number of stance onsets per cycle).
**Nothing is authored and there is no toggle.**

| body / mode | stance chains in contact | derived `h(p)` | bob |
|---|---|---|---|
| biped `stand`, walk (phases 0, 0.5) | 2, alternating | scalloped, **two troughs per cycle**, flat-bottomed for the double-support fraction `2β−1` | **2× the limb cycle** — *exactly the measured 0.5 s against a 1.0 s clip (body-plan-structure § 4)*. Amplitude per Table A |
| biped, both feet in phase (a hop) | 2, together | one dip per cycle + a flight phase | **1×**, and the flight arc is **not derivable** (§ 1) — reported, not fabricated |
| quadruped, lateral-sequence walk (0, ¼, ½, ¾) | 4, staggered | four shallow dips; the support polygon rolls | **4×**, amplitude much smaller — derived, not a special case |
| single stance chain (a pogo, a one-legged bird stance) | 1 | one dip per cycle | **1×** |
| snake `slither [belly]`, alligator `sprawl` belly, any **distributed** bearing | contact never breaks | `h` constant | **ZERO, by construction** |
| alligator `swim []`, a tree | none | no stance derivation exists | **NONE** — member #0 already answers `Unsupported` / `NothingDeclared` here |
| **any body at β < 0.5** (a run) | flight phase present | ballistic apex needs takeoff force | **DECLINED with a band report** (heir B6) |

**That is how "not known to be wanted for all bodies, or uniformly" is honoured without
inventing a knob**: three of the seven rows produce no bob at all, and none of them needed a
flag to say so. A body bobs iff its stance contacts alternate, and by exactly as much as its own
geometry says.

**One published band check, and it is where the honest limit shows.** The rigid-chain
("compass") model **over-predicts** real vertical CoM excursion: measured human walking is
≈ 4.6 cm at normal speed against a compass prediction near 6.6 cm — the classic gap that
*Saunders, Inman & Eberhart (1953), "The major determinants in normal and pathological gait"*
attributes to stance-knee flexion, pelvic list and ankle rocker. **All three are joint-stiffness
mechanisms = FORCE = § 1's "no" column.** So the derived bob is the honest upper bound today,
and the reduction is a **stand-in knob with B6 as heir** (§ 5, S3), *not* a number tuned until
it looks right. Noting the consequence for the walk verdict: **the derived bob will read as more
vertical motion than today's clip** (0.0658 m vs 0.040 m for the biped at a comparable speed),
and the user's live view is the senior instrument on that (corrections #77/#80).

### 4.2 Disposal of the three bootstrap clips' authored `root_bob_m` [assistant-proposed]

| clip | keyframed bob | disposal |
|---|---|---|
| `dc:anim/biped_walk` | 0.0 / 0.04 / 0.0 / 0.04 / 0.0 | **RETIRED with the clip** (§ 6). It is a hand-typed reproduction of an output — measured 2× period, confirmed by derivation. Its value as evidence is preserved as a **test fixture**, not as a shipped asset |
| `dc:anim/biped_jump` | 0.0 / **0.12** / … | **DELETED as a double authority.** The mover already owns the character's vertical translation during a jump; a clip that also translates the root is two authorities for one quantity — the same defect as the walk bob, in a different costume. The clip keeps its joint rotations (the tuck and the reach), which are genuinely its |
| `dc:anim/biped_idle` | 0.0 / **0.015** / 0.0 | **DELETED as a root translation; the breath survives as JOINTS.** A resting animal's breath is chest and shoulder motion, not the whole body sliding up 15 mm. Idle is non-locomotion and keeps riding as a clip (§ 6) — only its root translation goes |

**Consequently `Keyframe.root_bob_m` leaves the schema** [assistant-proposed, and this is the
one that needs a nod]: with all three uses disposed, the field's only remaining job would be to
let a future clip re-introduce an unratified second vertical authority. **A clip animates
joints; root height belongs to the gait and to the mover.** *Fallback if retiring the field is
rejected: keep it but redefine it as a **ratio of the plan's own root height**, which at least
kills the A-1 half. This is a wire-shaped change to `DefineAnimClip` (postcard is positional —
corrections #3), so it is cheapest **now**, in the same window as the § 7 binding-key migration,
and it is a migration after B3.*

### 4.3 Corrections #80's space-layering fact — the ONE composition

The fact stands untouched: `character.rs` builds the solver's hip as
`feet.y + leg.hip_local[1] + assets.root_delta_m − crouch_drop` while the render root writes
`t.y += assets.root_delta_m; t.y += (pose.root_bob_m − crouch_drop)`. **Two expressions for one
vertical composition, able to drift** — and today they do, by exactly `root_bob_m`.

**The one composition, stated** [assistant-proposed]:

```
root_offset(posture, mode, phase)  →  ONE f64, evaluated once per body per frame
```

It is the **whole** vertical story: member #0's static resting delta is its `phase`-invariant
part, the bob is its `phase`-varying part, and the crouch drop is not a term at all — crouch is
a **posture id** and its own bake key, so a crouched body's root offset simply *is* the crouch
bake's (`CROUCH_ROOT_DROP_M` was already scheduled demolition, corrections #81). Three terms
that can disagree collapse into one call that both consumers read.

**Why this is the right answer rather than a tidier version of the falsified one.** The rejected
fix *added* the bob to the hip — preserving two expressions and asking them to agree. This
*deletes the additive term*, so the drift is **structurally impossible**: there is no second
place to forget it. That is the S-3 shape (one authority; a consumer that re-derives, never a
copy that can diverge) and it is why the disposal had to wait for the pass where the bob is an
output. **No wiring fix is licensed by this document** — this states what the composition IS,
for the slice that builds it.

---

## 5. Knobs — TASTE vs STAND-IN, and how B6 absorbs the stand-ins [docket (b), (c)]

The doctrine forbids a knob that **replaces** the derivation, not one that **rides on top of**
it. Every knob below has an **identity default** (S-5) such that no-knob is byte-identical to
the pure derivation, and every kind-3 knob is **reported when it pushes an output past a
published band — reported, never refused** (docket (b)3; *a clockwork golem may want to step
wrong*).

**TASTE — permanent, kinds 1–2. These never get an heir; they are filling a genuine void.**

| knob | kind | identity default | why physics does not pin it |
|---|---|---|---|
| `gait_type` per (mode, Fr band) | 2 | the § 3.3 default pattern | a giraffe paces where a horse trots — biological, not geometric. It is *just phase-offset data* |
| `transition_fr` | 2 | **0.5** (published) | *where* a species changes gait is biology; the 0.5 anchor is the band, not the answer |
| `foot_clearance_ratio` | 1 | **0.015** (≈1.3 cm / L, Winter 1992) | above the geometric minimum, clearance is style |
| `trunk_counter_rotation`, `shoulder_roll`, `head_stabilisation`, `tail_carriage`, `arm_swing_amplitude` | 1 | 0 / derived counter-phase | length scale fixes *when* feet land and says almost nothing about carriage between landings. **Most of "shape it to my liking" should live here** |

**STAND-IN — kind 3. Each is physics we have not built, wearing a number. Each gets a
`stubs.md` entry naming B6 as heir.** *(Proposed below — this pass does not write `stubs.md`.)*

| id | knob | identity | band | what it stands in for | how B6 ABSORBS it |
|---|---|---|---|---|---|
| **S1** | `cadence_scale` | 1.0 | report outside [0.8, 1.25] | *"quicker-stepping than its size predicts"* = muscle power vs limb inertia | B6 gives per-segment density → real limb inertia + an actuation term; `cadence_scale` becomes **derived** and the knob's identity default becomes its permanent value |
| **S2** | `duty_bias_curve` (the β(Fr) exponent, § 3.1) | 0.263 | anchors β(0.5)=0.5, β(0.25)≈0.60 | a regression, not a mechanism | B6 + a cost-of-transport minimisation derives β(Fr) per body; the exponent is deleted, not re-tuned |
| **S3** | `bob_damping` | **1.0** (pure compass) | [0.5, 1.0]; default pack's biped ≈ **0.70**, derived from 4.6/6.6 cm published | stance-knee flexion, pelvic list, ankle rocker (Saunders 1953) | B6 gives joint stiffness → the stance chain flexes under load and the reduction **emerges**; `bob_damping` collapses to 1.0 |
| **S4** | `swing_flexion` | **0.0** (compass, near-straight swing) | [0, 1] | swing-leg energetics — minimising the work of swinging a limb | B6 + inertia gives a real swing solve; the § 3.1 ballistic cross-check becomes the *instrument*, and this knob becomes its output |

**The absorption property, stated so a build cannot get it wrong** [assistant-proposed]: every
stand-in is a **dimensionless multiplier or exponent on a term that survives B6**, never a
replacement for the term. So B6's arrival is a *derivation of the multiplier*, not a collision
with it — the knob's slot stays, its value stops being authored. **A knob that becomes a lie the
moment materials arrive is precisely what `root_bob_m` just cost us** (docket (c), verbatim).
The tell that separates the two tables: a **taste** knob has no true value; a **stand-in** knob
has a true value we cannot currently compute.

**Kind 4 — the authored-clip override for a `(mode, speed band)`** [docket (b)4]. Shape: a
binding `(mode, fr_lo, fr_hi) → clip`, checked at define time. **Degradation, not breakage**:
the override replaces the derived gait **only for the roles the clip binds**; every role it does
not bind keeps the derived gait, and a bound target that no longer exists on a mutated body is
**dropped, and that role falls back to derived** (the same rule fork inheritance already uses).
**This is why (b)4 and `stubs.md` § 34 are the same mechanism**: under today's address-exact
binding, an evolved body with renamed addresses loses the *whole* override silently; under role
binding it degrades **per role, loudly**. The escape hatch survives topology mutation only
because the binding key does.

---

## 6. Non-locomotion animation stays keyframed — both routes [docket (d)]

**Nothing here narrows `bodies.md`'s clip machinery.** Transitions (start, stop, turn),
one-shots (jump, swing, attack), emotes and upper-body action during a walk are **not gait**
(bones § 4's scope limit) and ride clips, reachable from pack build **and from a live MCP
session through the same `DefineAnimClip` door** — that door is unchanged by everything above
except § 4.2's `root_bob_m` proposal.

**The composition rule** [assistant-proposed, resolving § 7b's centaur]:

1. **The gait OWNS** every segment on a stance chain of an active bearing role, plus the
   non-bearing limbs it derives a counter-swing for (§ 3.3 rule 4).
2. **A clip OWNS** the roles/addresses it binds.
3. **On bearing chains a clip's contribution is REFUSED at define time.** You cannot author a
   leg over a locomotion gait — that request is a gait-type choice ((b)2) or an override
   ((b)4), and saying so at define time is cheaper than blending two authorities at runtime.
4. **On non-bearing segments the clip is an ADDITIVE layer over the gait's base** — which is how
   a sword swing rides a walk, and it is already the shape `character.rs` uses for the look
   joint (`base + orient`).
5. **The centaur**: a humanoid upper-body clip binds `{arm.*, head}`; a quadruped gait binds the
   four `sole` chains. **Disjoint → both play**, and the engine checks at define time that both
   are satisfied — *the "did you supply what you claimed" shape*. Rule 4 resolves the one real
   contact point (the trunk: gait wants counter-rotation, the clip wants a swing).

**Coexistence / migration for authored LOCOMOTION clips — today's three are the first case:**

| clip | verdict |
|---|---|
| `dc:anim/biped_walk` | **RETIRED**, not kept as an override. Keeping it as a (b)4 override for its Fr ≈ 0.391 band would preserve the exact A-1 stand-in this arc exists to retire (absolute metres, hand-typed asymmetry, a 46 mm internal contradiction — § 3.5). Its real value is **as a test**: *the derived gait reproduces the authored walk's phase structure exactly and its stride within the regression's scatter at Fr = 0.391* — the seam-first acceptance shape, softened from byte-identity to a band because the original was hand-typed rather than generated |
| `dc:anim/biped_idle` | **RIDES** — non-locomotion. Root translation deleted (§ 4.2); breath re-expressed as joints |
| `dc:anim/biped_jump` | **RIDES** — one-shot. Root translation deleted as a double authority with the mover |

**Consequence for the locomotion driver, flagged for B3 not designed here:**
`WALK_SPEED_THRESHOLD_M_S = 0.35` and the binary `Loco::{Idle, Walk}` switch become artifacts —
Fr is continuous and idle is its degenerate limit (amplitude → 0). And the phase clock must
advance with **distance travelled**, `dp = ds/λ`, not with wall time, or the skate of § 3.5
survives the whole bake. **Both are member 2's to own**; recorded here as requirements the gait
bake places on it, not as proposals.

---

## 7. `stubs.md` § 34 — the binding key. Decided here, deadline before B3 [user-ruled 2026-08-01]

### 7.1 Testing § 7b's finding 1 against § 3 — does the gait bake dissolve many-to-one?

**Yes for locomotion. No for the vocabulary.** Both halves matter and only the first was
anticipated.

**Dissolved:** § 3.3's phase assignment consumes `segments_with_role(plan, "sole")` **as a
set**, derives an order from contact geometry, and gives each member the same cycle at a derived
offset. It never needs to name *the* left leg. A biped (2), a quadruped (4) and a millipede
(100) are the same code path, and the shipped clip already **measures** as exactly this
(`leg_r(t) = leg_l(t+0.5)`, body-plan-structure § 4). So a selector grammar built to let a
locomotion clip say "the left leg" **would be machinery the gait bake obsoletes** — finding 1
was right, and the deferral bought the right thing.

**Not dissolved, and the count is three of the four binders:** a wave animates *the* left upper
arm; a left glove binds *the* left hand; `hand.r` mounts *one* segment. Cardinality-1 is
genuinely required for clips-that-are-not-gait, clothing and sockets. **A vocabulary designed
from the gait alone would be a vocabulary designed for one consumer** — the identical hazard,
mirrored.

### 7.2 The proposed vocabulary — one type, four binders [assistant-proposed]

```rust
pub enum BindTarget {
    Address(String),          // today's exact segment name — the IDENTITY DEFAULT
    Role { role: String, select: Select },
}
pub enum Select {
    All,            // every carrier; the consumer supplies a per-member derivation (GAIT)
    Unique,         // exactly one, or a define-time error naming the contenders
    Side(Side),     // exactly one within a DERIVED lateral partition (Left/Right/Medial)
    Ordinal(u16),   // exactly one at a derived serial index (segment 7 of a millipede)
}
```

Four properties, each load-bearing:

1. **The partition is DERIVED, never declared.** `Side` reads the sign of the role anchor's
   **x**; `Ordinal` reads the § 3.3 total order (x, then z, then y, then plan order). B0's
   ratified rule — *geometry determines laterality, fore/hind and pairing* — applied one level
   up. Authoring `hand.l` as a string would be **the poor man's role wearing a role's clothes**,
   the defect this entry exists to kill.
2. **Cardinality is a property of the BINDING, not of the role declaration.** The same role
   `sole` is `All` for the gait and `Side(Left)` for a left boot; declaring cardinality on the
   role would force a plan to anticipate its consumers. The engine checks each *binding's* claim
   at define time — *you supplied what you claimed*, the shape B0 already ships; `Unique`/
   `Side`/`Ordinal` at ≠ 1 is an error **naming the contenders**, generalising the shipped
   `unique_role_segment`.
3. **`Address` is the identity default.** Today's clips are `Address`, byte-identical under the
   new type (the S-5 acceptance test). Address binding stays legal forever for plan-specific
   content; roles are what makes content **portable**.
4. **One type serves all four binders**: **clips** (`JointRot.segment` → `JointRot.target`) ·
   **clothing** (§ Clothing's segment-name parent — mechanism untouched, key migrates, as its
   own banner says) · **fork inheritance** (a child inherits when it retains the clip's *bound
   targets* — **strictly stronger** under roles, since a fork that renames addresses but keeps
   roles still inherits) · **sockets** (§ Sockets' *"a socket name is role-shaped"*, satisfied
   literally: `Role { role: "hand", select: Side(Right) }`).

**Why now and not later** [user-ruled deadline]: `JointRot` is wire data
(`DefineAnimClip`/postcard, positional — corrections #3) and clips become **versioned sim
assets** the moment member 2 moves the firewall, because retiming a clip then changes hit
detection, hence replay and world identity. Today it is a recompile; after B3 it is a
migration. **§ 4.2's `root_bob_m` retirement is the same window and should travel in the same
change** — one wire move, not two.

**What this pass deliberately does NOT design:** a selector *grammar* (globs, predicates,
paths). `Select` has four variants because there are exactly four demonstrated consumers; a
fifth needs a fifth body to argue for it. *Three instances make a primitive; four enum variants
with four named callers is not a grammar.*

---

## 8. The layer table's vocabulary [docket (e), corrections #94 — fixed here because member 2 moves the line]

corrections #94 split one noun into two. **There are three**, and the third is the one whose
absence cost the walk-8 strafe (journal/0140).

| durable name | what it is | side | free to tune? |
|---|---|---|---|
| **GAZE** | `CharacterState.yaw/pitch` — aims perception (`sense_raycast` along `view_dir()`), rides the command log, replay-tested, derived in `step_character` when unheld, held by `set_look` until `clear_look` | **SIM** | **No.** Replay-critical |
| **TRUNK FACING** | `AnimState.trunk_yaw` — the smoothed heading the body's chest points, chasing travel over `TRUNK_TURN_WINDOW_S` | **CLIENT** | Yes |
| **NECK BEND** | `resolve_orientation`'s cervical clamp, and the trunk drag beyond it | **CLIENT** | Yes |

**Proposed replacement rows for § 3's table** [assistant-proposed; the docket assigns this pass
the naming, so this is the concrete proposal]:

- **sim owns:** **phase**, the **GAZE**, the **posture id**, and per-instance deltas.
- **client owns:** the **NECK BEND**, the **TRUNK FACING**, expressive layers, foot IK onto the
  actual terrain, and the sub-cycle interpolation between the gait's three keyframes.
- **bake owns:** posture, the gait vector, keyframes, collider sets.
- **Every "look-at" in `posture-gait.md` reads as the BEND**, and the word *look-at* is retired
  from the table entirely — it is the noun that carried two referents.

**And one more noun retired by § 4.3: "root bob".** Under the one composition there is no bob
*term*; there is **root height**, a function of phase, and "bob" is the name of the *observable*
oscillation. **A quantity gets a name; an observation does not get a field.** That is
corrections #94's second-order finding (*a noun standing for two things*) applied prospectively
rather than after the fact — the three tells in that entry were "look-at", "the bob" and
"posture", and this document is the pass that owns all three.

---

## 9. Venue, placement, and the first real consumer

**Venue is the caller's** [user-ruled, corrections #86; member #0's placement is forced
precedent]. `bake_gait` is a **pure function in dc-api** beside `bake_resting_posture` —
callable from pack build (authored species), **deeptime worldgen** (species the evolution pack
mints; deterministic for free, inherited down the phylogeny), and define time. dc-worldgen can
call dc-api; it could never have called a pack-build stage. No new crate (member #0's option (c)
reasoning holds: two bakes is not three instances).

**Returned, not stored** [bones § 6 property 2]. `GaitVector` is a sibling of `BodyPlan`, never
a field of it — gait *inside* the plan is per-species by construction with nowhere for an
instance delta to speak. Signature: `pose(species_gait, instance_delta, phase)`, identity
default, byte-identity tested.

**Bake PARAMETERS, not FRAMES** [bones § 6 property 1, raised one level by **G2**]: the bake
emits **coefficients over Fr** (`k_f = √(g/L)/2.3`, `k_λ = 2.3·L`, the `β̄` anchors, the
clearance ratio) and the runtime evaluates. *A pre-computed LUT over Fr is the obvious "make it
fast" pass and is exactly the foreclosure the bones name as highest-risk* — it quantizes duty
and would make a 3 % limp inexpressible.

**Runtime cost, bounded not measured** (perf is first-class): per entity per tick, ~2 `powf` +
1 `sqrt` + ~10 flops ≈ 60 ns → 1 000 entities at 20 Hz ≈ 0.06 ms/tick, and only when speed
changes. Per-frame sampling is three keyframes and one lerp per joint — identical to today's
`sample_clip`.

**First real consumer (A-4, with the edge):** `dc-client/src/body.rs::pose_for` / `AnimState` —
`sample_clip(walk, t)` becomes `sample_gait(&assets.gait, phase)`, and `character.rs`'s `hip_y`
and root translation both read the single `root_offset` of § 4.3. Second: **member 2**, which
reconstructs the nominal pose from `(plan, gait params, phase)` — **the same authorities, never
a cached client bake** (S-3, the edge member #0 recorded).

---

## 10. Acceptance tests, mechanically

Invariants and derivations, never snapshots (CLAUDE.md § Gates); headless in dc-api; the numeric
report printed test-side. Gate cost: microseconds — no world build.

1. **`cadence_scales_as_root_g_over_l`** — `f_i·√(L_i)` constant across the three plans at equal
   Fr, within 1e-12. *The bones' scaling law, falsifiable.*
2. **`dynamic_similarity_holds`** — at equal Fr, `θmax`, `β` and `Δ/L` **identical** across all
   three plans within 1e-12. *Alexander & Jayes, reproduced by our own arithmetic.*
3. **`phase_offsets_reproduce_the_authored_walk`** — § 3.3 on `dc:body/biped` yields `{0.0, 0.5}`
   for the soles and counter-phase arms, **matching the shipped clip exactly**. *A zero-parameter
   prediction against hand-typed data.*
4. **`gait_is_scale_invariant`** — scale a plan by k ∈ {0.4, 2.5}: angles, `β` and every ratio
   equal within 1e-12; metres scale by k.
5. **`no_bob_without_alternating_stance`** — a distributed-bearing plan and a mode-less plan both
   produce **exactly zero** root-height variation. *§ 4.1's "including none", asserted.*
6. **`bob_period_is_two_over_the_limb_cycle_for_a_biped`** — derived `h(p)` has exactly two minima
   per cycle. *The measured 0.5 s / 1.0 s, derived rather than typed.*
7. **`run_regime_is_declined_loudly`** — at `v = 4.5` every shipped plan reports out-of-band with
   the Froude number, the band, and **B6 as heir**. *§ 3.5's flag, executable.*
8. **`identity_knobs_are_byte_identical`** — knobs at identity reproduce the pure derivation bit
   for bit (S-5); a 3 % duty bias is expressible and mean-preserving (§ 3.4).
9. **`derived_gait_reproduces_the_authored_walk_at_its_own_speed`** — at Fr = 0.391 the phase
   structure matches exactly and the stride sits within 25 %. *The conversion's acceptance,
   band-softened because the original was hand-typed.*
10. **`unique_binding_fails_loudly`** — `Select::Unique`/`Side` at ≠ 1 is a define-time error
    naming contenders; `Address` bindings byte-identical to today.
11. **Report the numbers** — Tables A/B/C, measured. **A divergence from the predictions above is
    a finding about THIS document**, per the immutable-body rule.

---

## 11. Loud findings — where the bones are missing machinery

*The "cautiously" working. None is reconciled here; each rides with a named heir or needs a
ruling (§ 14).*

- **G1. `(neutral, extreme)` is under-parameterised for a limb cycle** (§ 3.2). A ping-pong
  between two poses drives the knee into hyperextension for half of every cycle; the shipped
  clip's own asymmetric lower-leg keys prove the waveform is not sinusoidal about neutral.
  Proposed: **three derived poses** (`neutral`, `contact`, `clearance`), stance as a signed
  ping-pong about neutral, swing as the arc between contacts. The bones' mechanism survives; the
  count does not. *Bones missing machinery — needs a nod.*
- **G2. Speed is not in the bake key, and every gait term is a function of speed** (§ 3.1). The
  ratified key `(species, posture, mode, yaw)` is **correct and stays** — the resolution is to
  bake **coefficients over dimensionless speed**, not scalars. Recorded because the obvious
  alternative (a speed-band key axis) would multiply the table and was rejected on that ground.
- **G3. `neutral` needs a resting solve for NON-bearing chains.** Member #0's bake solves stance
  chains only; a swinging arm has no `neutral`. The same zero-torque rule extends (a hanging
  limb's zero-torque configuration is straight down), but it is a **change to member #0's
  function**, not a free read. *Rides; heir = the gait bake's own build.*
- **G4. The Froude number's speed term is an input the bake does not have** (§ 1). The gait is
  honest only *relative to* a speed the body does not own. *Heir: B4.*
- **G5. β < 0.5 has a flight phase whose root height is ballistic and is not derivable at
  density ≡ 1** — and with `walk_speed_m_s = 4.5` this fires for **all three shipped plans on
  day one** (§ 3.5). The bake must report and decline, never fabricate. *Heir: B6.*
- **G6. "Amplitude is multiplicative toward `extreme`" cannot express a signed excursion**
  (bones § 4's table). The stance arc needs a signed, phase-driven amplitude. Small, but it is a
  ratified table row and is wrong as written.
- **G7. The layer table has a third client-owned quantity nobody named: TRUNK FACING** (§ 8).
  corrections #94 split one noun into two; the count is three, and the missing one is exactly
  where the strafe defect lived.

**A-1 ledger for the gait bake** (each simplification, its identity, its heir): the β(Fr)
interior = a two-point fit → cost-of-transport minimisation (**S2**, B6) · bob = pure compass →
stance-knee/pelvis reduction (**S3**, B6) · swing = near-straight → swing-leg energetics
(**S4**, B6) · cadence = pure length scale → limb inertia + actuation (**S1**, B6) · run =
declined → **G5**, B6 · effort term = member #0's static geometry → its own effort seam
(inherited) · mass = volume × 1 → B6 · `Select` = four variants → whatever a fifth demonstrated
binder needs.

---

## 12. Compliance

**North star / two clocks:** derivation at bake (any minting clock — corrections #86), sampling
at runtime; **behavior is code, tuning is data** — solver = engine primitive in dc-api, gait
vector = derived data, § 5's knobs = the tuning half. No capability tiering.
**S-3:** the plan stays the authority; the gait vector is returned-not-stored; member 2
reconstructs from `(plan, params, phase)`, never from a cached client bake; **§ 4.3's single
`root_offset` is S-3 applied to a quantity that currently has two expressions.**
**S-5:** every knob's identity reproduces the pure derivation byte-identically, as does
`BindTarget::Address`. **S-9:** third instance — species gait as derivable base, the limp as the
sparse overlay, unforeclosed (function-signature seam, f64 throughout).
**A-1:** § 11's ledger; the pass retires four absolute-metre stand-ins. **A-2:** the 12 fps step
is *deliberately not* re-argued — premise already corrected (#82), taste call is the user's, on
a moving body. **A-4:** § 9 names the first real consumer and the edge.

**⚠ CONTESTS flags: none.** The one place a contest was possible is recorded rather than
resolved: the bones' **two-keyframe count** (assistant-originated) collides with the user's
**sparse-keyframes-with-the-server-between-them** formulation (user-originated) once the stance
arc is examined. § 3.2 resolves it **in favour of the user's formulation** — three stored
keyframes rather than a generated arc — so nothing user-originated is narrowed. Had it gone the
other way it would have been a ⚠ CONTESTS and a stop. Checked before filing: § IK is CLOSED
(corrections #81) and is not re-presented; the postures ruling and the look-ownership ruling are
taken as given; corrections #93's disposal is *applied*, not re-argued.

---

## 13. What I could not answer

- **Whether `walk_speed_m_s = 4.5` is a bring-up artifact or genre taste.** Within a whisker of
  Minecraft's walk speed, which makes "wrong constant" and "deliberate feel" equally readable
  from the number alone. § 3.5 states three dispositions; choosing is not mine.
- **What a quadruped's `h(p)` actually looks like.** The four-contact superposition is derivable
  and § 4.1 states its shape, but we have **no quadruped**, so nothing can falsify it. Recorded
  so the first quadruped's pass re-derives rather than inherits.
- **Whether β(Fr) should be fitted at all**, versus the bake emitting a band and the runtime
  picking within it. Both defensible; the second is more honest and more expensive.
- **The verdict on the derived bob's *look*** — ~65 % larger than today's authored bob for the
  biped, and the user's live view is the senior instrument for anything temporal (#77/#80).

## 14. Open questions, triaged

**USER CALLS (ratification, not measurement):**

1. **`walk_speed_m_s = 4.5` vs the Froude band** (§ 3.5, **G4/G5**). All three shipped bodies are
   in the *run* regime at the world-global speed, and a run is the one regime this bake cannot
   honestly derive. Three dispositions stated; none chosen. **This is the flag the docket asked
   for and it blocks nothing until the build reaches the band report.**
2. **Retiring `Keyframe.root_bob_m` from the schema** (§ 4.2). All three authored uses are
   disposed; keeping the field keeps a second vertical authority available. Wire-shaped, so
   cheapest **now**, alongside § 7's migration. Fallback stated (redefine as a ratio).
3. **Retiring `dc:anim/biped_walk` as a shipped clip, converting it to a test fixture** (§ 6).
   The alternative — keeping it as a (b)4 override for its own Fr band — preserves the A-1
   stand-in the arc is retiring.
4. **G1's three keyframes** (§ 3.2). The bones say two; a cycle has two turning points. The
   resolution preserves the user's between-keyframes design at the cost of an
   assistant-originated count.
5. **The § 8 layer-table naming** (GAZE / TRUNK FACING / NECK BEND, and "root height" replacing
   "root bob"). The docket assigns this pass the naming; it is still a design statement in a
   cautiously-ratified document.
6. **The four proposed `stubs.md` entries S1–S4** (§ 5) — written as drafts here, *not* written
   into `stubs.md`; the integrator applies.

**ENGINEERING (decide with a number, no ratification owed):**

7. **G3** — extending member #0's resting solve to non-bearing chains. Mechanical.
8. **The β(Fr) exponent 0.263 and the stride exponent 0.3** — verify against the cited sources;
   this pass had no network access and the citations are from knowledge. A divergence is a
   finding about this document (immutable body; the banner obligation falls on whoever measures).
9. **Coefficients vs a LUT over Fr** (§ 9). Coefficients recommended; a LUT is a later
   optimisation *with* the no-over-quantization test (bones § 6 property 3).
10. **Where the phase clock lives and whether it advances with distance rather than time** (§ 6).
    **Member 2's, explicitly** — recorded as a requirement the gait places on B3, not designed.
11. **`WALK_SPEED_THRESHOLD_M_S` and the binary `Loco` switch** become artifacts under a
    continuous Fr (§ 6). Member 2's.
12. **Whether `Select::Ordinal` earns its variant today.** No shipped body needs it; a millipede
    would. Cheapest to include with the enum, cheapest to omit if no consumer is named.
