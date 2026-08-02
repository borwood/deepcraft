# The resting-posture bake (member #0) — a design pass against the posture-gait bones

> **⚠ RULINGS LANDED 2026-08-02, same day (user) — mutable header; body is testimony.**
> **The build is GREENLIT** on this pass's shape, with both § 11 nods given: **Q2 yes**
> (pure fn in dc-api) and **Q3 yes** (bottom-face support patches). **And F1 is SHARPENED by
> corrections #86** (user: *"the bake cannot run 'at pack build', or not only"*): the venue
> sentence this pass inherited was sloppy — evolution mints species **during deeptime
> worldgen** (`ecology.md` § 4: speciation is emergent from terrain; the frond bake was
> always ruled "at deeptime"), so the bake must be callable from the worldgen clock. The
> pure-fn-in-dc-api placement this pass recommended is thereby not merely right but
> **forced** — dc-worldgen can call dc-api; it could never have called a pack-build stage.
> § 3's "call site migrates when packs land on disk" widens to: **every minting clock is a
> call site** (pack build, deeptime worldgen, define-time). **Q1 is ANSWERED 2026-08-02
> (user, sweep-checked): postures are pack-declared REGISTRY content by namespaced id,
> engine owns the contract (bakeable + collider disposition + transition semantics), the
> default pack ships the 2026-07-19 posture ladder — `bodies.md` § Postures.** The bake
> key's posture axis is therefore an id, not an enum; slice one's `Standing` placeholder
> maps to `dc:posture/stand` at the consumer migration (append-shaped — the wire enum is
> replay-critical).

**Status: DESIGN PASS. Nothing here is ratified and no code was changed.** The deliverable is
this document. It works ROADMAP § Sequenced's first-slice spec against `posture-gait.md`'s
ratified bones (§§ 2–6) and the post-B0 code, and proposes a concrete shape; proposing and
building are different acts, and the second is the user's call.

**Read with:** `posture-gait.md` (CAUTIOUSLY RATIFIED 2026-08-01 — bones; this pass is
*expected* to report bones found missing machinery, and § 8 below does) · ROADMAP § Sequenced
*"POSTURE AND GAIT ARE DERIVED…"* (the acceptance anchor) ·
`docs/audits/2026-08-01-body-plan-structure-design.md` + its header rulings (B0) ·
`bodies.md` (data model; **§ IK is CLOSED — corrections #81 — and nothing below re-presents
it**) · `dependency-graph.md` § 2b · journal/0130–0132, 0135 · corrections #77 #78 #80 #81
#84 · spines S-3 S-5 S-9, A-1 A-4.

**Provenance convention** (the 2026-07-25 rule): mechanisms below are marked
**[user-ruled]** (derived from a ratified ruling), **[bones]** (stated in posture-gait §§ 2–6),
or **[assistant-proposed]** (this pass's own; a hypothesis until ratified).

---

## 0. The proposed shape, in five sentences

A pure function in **dc-api** — `bake_resting_posture(plan, key)` — takes a `BodyPlan`, a
declared **mode** (B0's `ModeDef`), and the bake key's other axes as identity, and returns
either a `RestingPosture` (per-stance-chain joint **angles in radians** plus a **root-height
ratio** — never metres), a **legal absence** (the plan declares no modes: a tree), or a
**loud refusal** naming the geometry that took it out of domain (a distributed bearing set: a
snake's belly). For the three shipped bipeds the solve is closed-form static geometry: each
sole-bearing chain becomes a vertical zero-torque column under its hip, the root lands at
chain reach, and balance (CoM over base of support, volume as the mass proxy) is **verified,
not optimised**. Predicted outputs: biped hip **0.880 m**, stout **0.440 m**, longleg
**1.020 m** — longleg stands **taller** than the biped with straight legs, which is the
ratified acceptance. Nothing is stored in the registry or seen by the sim; the first real
consumer (named, § 5) is the renderer's `build_plan_assets`, which replaces the pinned
`trunk.pivot_m[1]`-as-hip read with the derived hip — the continuation slot. Where the bones
say "runs at pack build", today's engine **has no pack-build step**, and § 3 works that gap
loudly rather than reconciling it away.

---

## 1. WHAT is computed — outputs, derivations, units, consumers

Per bake key (§ 6 for the axes), from **declared inputs only**: the segment tree
(`SegmentDef` geometry), the declared roles with segment-local anchors (`RoleDef.at_m` —
B0), the mode's bearing set (`ModeDef.bearing`), and segment **volume** as the mass proxy
(per-segment material is the named heir — ROADMAP first-slice spec, dependency-graph B6).

| output | derivation | units | consumed by |
|---|---|---|---|
| **resting joint angles**, per stance-chain joint | the zero-torque configuration (§ 2) | **radians**, XYZ Euler, same frame as `JointRot.euler` | slice one: tests. Heirs: the gait bake's `neutral` keyframe (posture-gait § 4); the renderer's rest pose (§ 5) |
| **root height** (the derived hip) | vertical extent of the posed stance chain(s) from root to ground | **ratio** of the governing chain's reach (dimensionless); metres derived at consumption from the plan's own lengths | slice one: tests + the reported numbers. First real consumer: `character.rs` root placement (§ 5) |
| **balance verdict** | CoM (volume-weighted, at the resting pose) projected against the base of support | boolean + the numbers behind it | the bake itself (a standing body that cannot balance is refused loudly) |
| **base-of-support hull** | support patches of the bearing anchors (§ 2.3) | body-frame metres, **reported not stored** | *no consumer yet* — internal to the balance check; printed in the test report. Deliberately NOT an output struct field (A-4: machinery without a consumer). Named heirs if one appears: derived colliders (member 3), gait stance logic (member 1) |

**Units doctrine [user-ruled — decision 24 / posture-gait § 8: "bake ratios and angles, not
metres"].** The plan itself is authored in metres (bodies are fixed real-world size — that is
its charter and is untouched). The **bake's outputs** are scale-free: angles are invariant
under uniform scaling and the ratio × the plan's own reach reproduces metres at consumption.
This is what makes the growth axis droppable (body-plan-structure pass § 5.7) and it is
testable (§ 7, test 4).

**What is explicitly NOT computed** (ROADMAP's NOT-list, restated so this pass cannot be read
as promising it): the hover's temporal half (`character.rs:219`'s `hip_y` missing
`pose.root_bob_m` — corrections #80's operative defect — is a **separate call**); clips;
`CROUCH_ROOT_DROP_M` (crouch is a later **posture-axis key value**, § 6); the half-voxel
window; gait, colliders, damage.

---

## 2. THE MATH, concretely, for the three shipped bipeds

### 2.1 Stance chains — declared contact, derived chain [bones + B0, shipped]

For each segment carrying a role the mode's `bearing` names (here `sole`), walk parents to
the first branch point — exactly the derivation `leg_rigs` already ships
(`dc-client/src/body.rs:355-417`). The chain for every shipped plan is
`[leg_*_lower, leg_*_upper]`; bone lengths are `l1 = |lower.pivot_m|` and `l2 = |anchor|`
(knee pivot → declared sole anchor). **The solver takes the contact set as an explicit input
parameter** [user-ruled — body-plan-structure § 8 item 2, and its `leg_rigs` interim is
dissolved: the input is B0's `segments_with_role(plan, role)` resolved through the mode's
bearing list]. The chain-walk itself should be **shared with, not duplicated beside,**
`leg_rigs`' — see § 3.4 (an A-4-sibling hazard this pass flags on itself).

### 2.2 The resting solve — static geometry, with the derivation stated [assistant-proposed mechanism; the *class* of answer is user-anticipated: "the cheapest thing that gets hip height right", posture-gait § 8]

**The rule: each stance chain assumes the configuration in which gravity exerts zero moment
at every joint — the vertical column under its hip attachment.** For the shipped plans, whose
leg chains are authored pointing straight down with the anchor directly under the knee, this
configuration **is the identity rotation** (all stance angles exactly `0.0`), and the root
height is the chain reach.

This is not a shortcut wearing a solver's clothes; it is the exact minimiser of static joint
torque for a column supporting a load from above, which is why the ROADMAP hazard reads the
way it does: *for a symmetric standing biped, "legs straight, hip at leg reach" is THE RIGHT
ANSWER*. § 7 test 3 encodes that as an assertion so a later session cannot "fix" it into a
bent-knee answer out of suspicion of degeneracy.

**Predicted numbers** (hand-derived from the plan literals in
`dc-api/src/bodies/default_pack.rs` and `bodies/experiments.rs`; the build must print
measured values — the acceptance demands reported numbers, and these predictions are what
they are checked against):

| | l1 (hip→knee) | l2 (knee→sole anchor) | reach | authored hip (`trunk.pivot_m[1]`) | **derived hip** | resting knee | authored−derived |
|---|---|---|---|---|---|---|---|
| `dc:body/biped` | 0.450 | 0.430 | 0.880 | 0.900 | **0.880 m** | **0.0°** | +0.020 |
| `dc:body/stout` | 0.225 | 0.215 | 0.440 | 0.460 | **0.440 m** | **0.0°** | +0.020 |
| `dc:body/longleg` | 0.520 | 0.500 | 1.020 | 0.900 | **1.020 m** | **0.0°** | −0.120 |

- **The acceptance holds by prediction:** longleg's derived hip (1.020 m) exceeds the
  biped's (0.880 m) by **140 mm — standing taller, not squatting**, against journal/0131's
  measured −56.2° squat under the pinned hip. The knee that had to bend under the inversion
  does not bend when the feet pin the pelvis.
- **Both hand-authored plans carry exactly the same +0.020 m gap** — the identical absolute
  error copied from biped to stout. Corroborates § 1 of the bones: the gap was never a
  design, it was one mistake inherited by construction. The bake deletes the class.
- **Consequences worth naming now:** the biped's standing height becomes ~1.78 m under a
  world-global 1.8 m `CharacterConfig` collider — a pre-existing mismatch class
  (dependency-graph **B4**, a 1.60 m stout is already hit as 1.8 m), not a new one, and the
  presumptive constants are scheduled demolition [user-ruled]. And with hip == reach exactly,
  a standing sole target sits **on the IK annulus boundary**; `solve_leg_ik`'s existing
  `1e-6` clamp yields a sub-µm resting residual — an engineering footnote for the consumer
  slice (§ 11, rides-as-built), not a design question.

### 2.3 Balance — verified, not optimised; and a real finding about point anchors

CoM = Σ(segment box volume × posed box centre) / Σ volume, over **all** segments at the
resting pose, density ≡ 1 [bones: volume is the mass proxy; B6 is the heir and plugs in
here — this check is the mass integral's first posture-side consumer]. The stout's
knuckle-dragging arms participate in the CoM and **not** in support — bearing is declared
per mode, which is exactly the alligator amendment doing its job on a body we already ship.

**Predicted CoM for the biped** (hand-derived, same caveat as above): lateral and
fore-aft exactly `(0, 0)` by the mirror symmetry B0 made structural; height ≈ **0.97 m** at
the derived hip ≈ **0.545 of standing height**. Published anthropometry puts a standing
human's CoM near **0.55–0.57 of stature** — inside the band, from cuboids. This is the
measure-against-the-literature hook (CLAUDE.md; posture-gait prior 4) available to this
slice, and hip/stature ≈ 0.494 against the anthropometric leg-length convention (~0.53 of
stature) is the second. **Band checks, not calibrations** — nothing is tuned to hit them.

**The finding [assistant-proposed resolution]: point anchors make strict balance
measure-zero for a biped.** `RoleDef.at_m` declares the sole as a **point** (the bottom-face
centre). Two points give a base of support that is a **line segment** along x; the CoM's
fore-aft coordinate must then be *exactly* zero to sit "over" it — satisfiable only by
perfect symmetry, and violated by any forward head or tail. Real bipeds stand because feet
have **extent**. Proposed resolution, needing **no schema change**: the bake derives each
bearing contact's **support patch** as the bearing segment's bottom face (the box's x/z
extents around the declared anchor — the same geometry `with_sole` derives the anchor from).
The biped's hull becomes x ∈ [−0.22, 0.22] × z ∈ [−0.09, 0.09] and the check is honest.
Fallback if the patch reading is rejected: a point hull with a stated tolerance — worse,
because the tolerance is a number with no derivation. *This does not contest the
user-originated balance rule; it supplies machinery the rule needs to be checkable on our
declared data. Flagged loudly in § 8 as bones-missing-machinery.*

### 2.4 The effort term — static geometry now, a seam so the bird does not foreclose [bones § 8 assumption + § 6.J finding, carried]

Slice one's effort term **is** static geometry (§ 2.2), per the bones' own assumption. The
bird finding (body-plan-structure § 6.J) stands: a bird's resting leg is deeply folded and
tendon-held — the zero-torque column is *the wrong animal* there, so the effort term matters
sooner than § 8 assumed. What this slice owes is **non-foreclosure, at zero mechanism cost**
(the § 6 pattern of the bones): the resting-angle computation sits behind **one function**
whose output shape (angles per joint) does not change when the internals become a real
minimisation with preferred-angle/tendon terms. The seam is a function signature, not a
struct field. A bird declaring `stand [sole]` today gets the straight-leg answer — in-domain,
honest, and **wrong for a bird**, which is recorded here rather than discovered later. No
five-body table row breaks; the answer's *quality* degrades for exactly one body we have not
built.

---

## 3. WHERE it runs — the bones say "pack build"; no pack build exists

**The gap, stated loudly [this pass's most important bones-missing-machinery report]:**
posture-gait § 3 places the bake at *"pack build; gen-time free"* and § 9 places the baked
result as *"derived data in the pack's compiled form"*. Today **there is no on-disk pack and
no pack-build step**: plans are compiled-in Rust (`default_pack.rs`, `experiments.rs`)
emitted as a command batch and submitted through the one door at world construction
(`dc-client/src/authority.rs::load_body_packs`, applying at the first tick). The pack's
"compiled form" **is currently the Rust source**. The bones name a stage of a pipeline that
does not exist yet — same shape as `worldgen.md`'s ON-HOLD item 4, and the honest treatment
is the same: the ambition stands; the today-placement must be chosen deliberately.

Options worked:

**(a) Compute at define time in the host and store beside `BodyPlanDef`.** Rejected.
It writes derivable data into `HostState` — registry state the sim can reach — which is the
**S-3 summary-wearing-authority hazard** posture-gait § 9 itself names (*"if a sim pass ever
keys on the bake instead of the body…"*), taken on for no benefit. It also couples the
registry's wire/undo machinery to a result that is a pure function of what the registry
already holds.

**(b) A pure function in dc-api, computed on demand, memoized by its consumers.**
**Recommended [assistant-proposed].** `dc_api::bodies::posture::bake_resting_posture(plan,
key)` — pure f64 arithmetic on plan data, no world input, no RNG, no clock: deterministic by
construction, headless (dc-api has no rendering dependency — the crate-layering rule holds),
and placed exactly where `validate_plan`, `segments_with_role` and `unique_role_segment`
already live. The solver is thereby an **engine primitive** [user-ruled placement,
dependency-graph § 2b] in the crate that owns the body data model. Cost: closed-form
trigonometry over ≤ a dozen segments — microseconds, **once per plan per consumer**, never
per frame; the two-clocks doctrine is satisfied trivially (the client caches it in
`BodyAssets` at plan-asset build, the same place `leg_rigs` is cached today). When
packs-on-disk arrive, **the same function runs at pack compile and its result lands in the
compiled artifact — the seam that migrates is the call site, never the signature.** That is
the today-degenerate form of "derived data in the pack's compiled form", stated rather than
assumed.

**(c) A new crate (`dc-bake`).** Rejected. No second bake exists to justify it (the frond
bake is a design sketch); a crate with one function and one consumer is A-4 furniture. If the
gait bake (member 1) and collider bake (member 3) land and want a shared home, that is the
moment — three instances make a primitive.

**3.4 One hazard flagged on this pass's own recommendation:** the stance-chain walk of § 2.1
already exists as `leg_rigs` in **dc-client**. A dc-api bake that re-implements it creates
*two derivations of one truth* — the project's characteristic failure (spines header). The
build should **move the chain derivation down into dc-api** (pure data + arithmetic; nothing
bevy about it — `body.rs`'s own doc comment says as much) and have `leg_rigs` consume it, or
else must justify the duplication loudly. This is a build-order note, not a design fork.

---

## 4. WHERE the result lives — computed, cached by consumers, never authoritative

**Proposed [assistant-proposed]:** `RestingPosture` is a plain struct in dc-api, **returned,
not stored** — a *sibling* of `BodyPlan`, never a field of it. Two reasons, both from the
bones: (1) posture-gait § 6 property 2 — the seam is a **function signature**; baked data
living *inside* the plan is per-species by construction with nowhere for an instance delta to
speak; (2) a stored copy beside the definition is the S-3 shape (a summary beside its
authority) the moment anything reads the copy instead of re-deriving.

```rust
// Sketch — concrete SDK types are explicitly NOT decided by the bones (§ 8)
// and not decided here either.
pub struct BakeKey<'a> {
    pub mode: &'a str,        // must name a declared ModeDef — exercised in slice one
    pub posture: Posture,     // Standing only in slice one; the axis SHAPE carried (§ 6)
    // yaw: carried as identity — a resting posture is body-frame and yaw-covariant
    // by construction; the yaw axis does work at the collider member, not here (§ 6)
}
pub enum BakeOutcome {
    Baked(RestingPosture),
    NothingDeclared,                    // zero modes: a tree — legal absence, not an error
    Unsupported(UnsupportedReason),     // loud, names the offending geometry (§ 6)
}
pub struct RestingPosture {
    pub chains: Vec<ChainPose>,         // per stance chain: (segment name, [f64;3] radians)
    pub root_height_ratio: f64,         // root height / governing chain reach; metres at consumption
}
```

**Determinism and the firewall (S-3), stated as invariants:** the sim today sees only
`CharacterConfig` and parametric `Posture`, and **this slice changes nothing about that** —
the bake result is consumed client-side (and by tests) only. The baked posture is derived
*from* the body; **no sim pass may ever key on the bake instead of the body** [bones § 9,
restated as this slice's standing constraint]. When member 2 moves the firewall (the sim
learns the *nominal pose*), the nominal pose must be reconstructed from
`(plan, clip, phase)` — the same authorities — not read from a cached client bake; that is
member 2's design pass to hold, recorded here so the edge is not lost.

---

## 5. WHO consumes it first — the A-4 answer, explicit

**Slice one is headless: compute + tests + reported numbers.** Its consumers are the § 7
tests and the printed report. Under A-4 that is machinery whose *real* consumer must be
named, with the edge:

**First real consumer (the continuation slot): `dc-client/src/character.rs` /
`build_plan_assets`.** The edge, concretely: `build_plan_assets` calls the bake once per
plan and caches the result in `BodyAssets`; the renderer then (1) places the root's trunk
pivot at the **derived hip** (today `character.rs:257-261` writes the authored
`s.pivot_m[1]` = 0.9 for the trunk), and (2) `leg_rigs`' `hip_local` — read at
`character.rs:227` to build `hip_y` — derives from the same number. That slice **retires the
hover's geometric half**: the standing biped's sole target stops being unreachable-by-20 mm.
It does **not** retire the hover's temporal half — `hip_y` still omits `pose.root_bob_m`
(corrections #80), a separate ruled-on defect whose fix belongs to the bob-emergence work,
and conflating the two is exactly the #80 error class (a mechanism that explains a magnitude
is not thereby the cause). Second consumer in line: the gait bake (member 1) takes the
resting angles as its `neutral` keyframe.

The continuation slice will also have to face the clip-interplay question (clips were
authored against the 0.9 m rest; the derived hip shifts the shipped bipeds by 20 mm and
longleg by 120 mm) — named here as **that slice's** problem so this one is not tempted to
"fix the look" (the ROADMAP's escape-clause warning).

---

## 6. The MODE interplay, the identity default, and the five-body game

**The bake key is `(species, posture, MODE, yaw)` [user-ruled — the alligator amendment
added mode; decision 24 dropped growth].** Slice one exercises the axes as:

| axis | slice one | how the shape carries it |
|---|---|---|
| species | the `plan` argument | trivially |
| **mode** | **exercised** — the caller names a declared mode; the shipped plans declare exactly `stand [sole]` | `BakeKey.mode` |
| posture | **identity** — `Posture::Standing` only | the key field exists; `Crouching` is the named heir (its bake is what deletes `CROUCH_ROOT_DROP_M` — a later member act, per the NOT-list) |
| yaw | **identity, with a derivation** — resting angles are body-frame, so the posture result is yaw-covariant by construction; yaw does real work only where world-axis-aligned artifacts appear (AABBs, member 3) | documented on the key; not a parameter that changes this output |

**Zero modes declared → `NothingDeclared`, not an error.** B0 ratified the mode-less plan as
legal (the identity default: *"a mode-less plan is legal and declares nothing about
support"* — `bodies.rs:199-202`), and define time must not start refusing what B0 accepts.
The bake's absence-answer mirrors `unique_role_segment`'s `Ok(None)`: a legal absence,
distinct from an in-domain failure. A tree defines, binds no locomotion, and has no standing
posture to bake — all three facts independently true.

**Out-of-domain → loud refusal [user-ruled: "a non-balancing body is LOUDLY UNSUPPORTED,
never silently given a nonsense posture"; no SupportKind type exists or may be invented].**
Domain is determined by the **active bearing set's geometry**, computed, not declared:

- **in domain:** every bearing role resolves to segments with **anchored** (`at_m: Some`)
  contacts; the anchor set has ≥ 2 members (§ 8 flags the count as a slice-one narrowing);
  each contact's chain reaches the root.
- **out:** any bearing role carried **unanchored** (whole-segment: a distributed support —
  the snake's belly, a tree read as bearing on its trunk) → `Unsupported`, naming role and
  segments — the same name-the-contenders shape as `unique_role_segment`.

The five bodies, gamed:

| body | declares | outcome |
|---|---|---|
| **biped / stout / longleg** | `stand [sole]`, 2 anchored soles | **Baked.** The § 2 numbers |
| **quadruped** | `stand [sole]`, 4 anchored soles | **Baked** — the solve is per-chain and N-ary already; nothing biped-shaped in it. (Unequal fore/hind legs make the root solve non-trivial — the shortest chain bends or the body pitches; slice one has no such body and must not silently pretend the vertical-column answer covers it: refuse chains whose reaches differ beyond ε, with the named heir. § 8) |
| **snake** | `slither [belly]`, belly unanchored on many segments | **Unsupported("bearing role `belly` is distributed, not a standing point-set")** — the correct refusal; balance is vacuous on a line [bones § 5] |
| **bird** | `stand [sole]`, 2 anchored soles | **Baked, in-domain, and the straight-leg answer is wrong for the animal** (§ 2.4) — an honest degradation behind the effort seam, not a refusal |
| **tree** | no modes | **NothingDeclared** — a legal absence; its physics is statics, not balance, and the bake refusing to invent an answer is the ratified outcome |
| *(alligator, the amendment's own case)* | `high_walk [sole]` → Baked · `sprawl [sole, belly]` → **Unsupported** (mixed anchored + distributed) · `swim []` → Unsupported (empty bearing set) | the mode axis earning its keep on one body |

---

## 7. Acceptance tests, mechanically

The ROADMAP outcomes as assertions, plus this pass's additions — all invariants or
derivations, never snapshots (CLAUDE.md § Gates); all headless in dc-api; the numeric report
printed test-side like `retarget_report` (`--nocapture`), no `examples/` needed:

1. **`longleg_stands_taller_than_the_biped`** — `hip_m(longleg) > hip_m(biped)`; predicted
   1.020 vs 0.880. *The ratified acceptance, verbatim.*
2. **`no_resting_gap_between_sole_and_ground`** — for all three plans: pose the chains at
   the baked angles with the root at the baked height; every declared sole anchor lands at
   y = 0 within 1e-9 m. *The user's no-artificial-gap ruling as an executable invariant;
   kills the +0.020 class structurally.*
3. **`straight_legs_are_the_answer_for_a_symmetric_biped`** — every stance angle == 0.0 for
   the three shipped plans. *The ROADMAP hazard, encoded: the right answer, asserted as
   right, so it cannot be "fixed" into a degenerate-looking-therefore-wrong one.*
4. **`bake_is_scale_invariant`** — uniformly scale a plan's `pivot_m`/`size_m`/`offset_m`/
   anchors by k ∈ {0.4, 2.5}: angles equal within 1e-12 rad, `root_height_ratio` equal,
   `hip_m` scales by k within 1e-12 relative. *Decision 24's ratios-not-metres, falsifiable.*
5. **`hip_height_is_an_output`** — the bake result is computed with the authored
   `trunk.pivot_m[1]` **excluded** from the solve inputs (it is a rest origin, not a hip —
   body-plan-structure § 5.1); mutating it changes nothing in the baked output. *The
   inversion itself, as a test.*
6. **`mode_less_plan_is_a_legal_absence`** — strip `modes` → `NothingDeclared` (a tree).
7. **`distributed_bearing_is_loudly_unsupported`** — add `sprawl [sole, belly]` with an
   unanchored belly role → `Unsupported` whose message names `belly` and the segments.
8. **`balance_holds_and_is_reported`** — for the three plans: CoM (x, z) strictly inside the
   support hull; print CoM height, hull extents, hip/stature and CoM/stature ratios beside
   the anthropometric bands (band *check*, not calibration).
9. **`bake_is_deterministic`** — two calls, `==`.
10. **Report the numbers** — the § 2.2 table, measured, printed; the design-pass predictions
    above are what the first run is checked against, and a divergence is a finding about
    *this document*, recorded per the immutable-body rule.

Gate cost: microseconds — no world build, no extent question.

---

## 8. Loud findings — where the bones or the board are missing machinery

*The "cautiously" working. None of these is reconciled here; each either rides with a named
heir or needs a ruling (§ 11).*

- **F1. "Pack build" has no referent in today's engine** (§ 3). The bones' § 3 table and § 9
  placement name a pipeline stage that does not exist; the recommended today-form is the pure
  fn + consumer memoization, with the call site — not the signature — migrating when packs
  land on disk. *Bones missing machinery, reported; the § 3(b) placement needs the user's
  nod before build.*
- **F2. Point anchors cannot carry the balance rule for a biped** (§ 2.3). Strict
  CoM-over-base is measure-zero fore-aft over two point contacts. Proposed: support patches
  derived from the bearing segment's bottom face — no schema change. *The balance rule
  [user-originated] is not contested; it is under-supplied by the declared data, and the
  patch derivation is the missing machinery.*
- **F3. The posture axis has no declared vocabulary** (§ 6). Modes are declared per plan
  (B0); postures are nowhere declared — the sim's `Posture` enum is a closed engine-owned
  pair (Standing/Crouching), while the bones' own alligator example ("basking posture")
  implies an open per-species set. Slice one is untouched (Standing only) but **the key's
  type encodes the answer**, and a closed engine enum as the bake key would be the
  `KNOWN_VERBS` shape one axis over. *User-owned question, § 11 Q1. Not pre-answered here:
  the sketch's `posture: Posture` is the slice-one placeholder, explicitly marked.*
- **F4. Unequal stance chains are refused, not solved** (§ 6, quadruped row). The
  vertical-column solve is exact only when all active chains reach equally. Slice one has no
  unequal-chain body; refusing beyond ε with a named heir keeps the simplification from
  becoming the definition (A-1). *Rides; heir = the first quadruped's design pass.*
- **F5. Stance width is an output the bones name and this slice fixes at its identity.**
  Posture-gait § 1 lists stance width among the outputs; slice one places feet directly
  below hips (the zero-lateral-torque answer for vertical columns), so stance width ==
  authored hip spacing. That is the identity default, not the definition; a solver placing
  feet under the CoM for a narrow-hipped or leaning body is the heir. *A-1 guard, recorded.*
- **F6. The chain-walk exists twice if built naively** (§ 3.4). `leg_rigs` (dc-client)
  already derives stance chains from declared soles; the bake needs the same derivation in
  dc-api. Build note: hoist, don't duplicate — the characteristic-failure warning applied to
  this very slice.

**A-1 ledger for slice one** (each simplification, its identity, its heir): effort term =
zero-torque static geometry → real minimisation with preferred-angle/tendon terms (the bird,
§ 2.4) · mass = volume × 1 → per-segment materials (B6) · domain = ≥ 2 anchored point
contacts, equal reaches → distributed/anchored support rules (**gated on their own
ratifications** — the vegetation sketch is explicitly unratified), unequal-chain solve (F4) ·
posture axis = Standing → crouch-as-posture (deletes `CROUCH_ROOT_DROP_M`, later member) ·
support patch = bearing segment's bottom face → whatever the collider member derives, if
richer.

---

## 9. Compliance

**North star / two clocks:** derivation at build/load time, sampling at runtime; the bake is
microseconds once-per-plan and touches no per-frame path. Solver = engine primitive in
dc-api; body = pack content; baked result = derived data whose compiled-pack home exists the
day compiled packs do (F1). No capability tiering — a third-party plan gets the bake through
the same `validate_plan`-adjacent surface the defaults use.
**S-3:** the plan stays the authority; the bake result is returned-not-stored, consumed
client-side, never sim-read; the member-2 edge is recorded in § 4.
**S-5:** the consumer-side identity default is exact — until the continuation slice lands,
the renderer's behaviour is byte-identical (slice one adds no renderer read); `NothingDeclared`
preserves B0's mode-less legality.
**S-9:** the species bake is the derivable base; instance deltas remain the sparse overlay,
unforeclosed because the seam is a function signature and nothing here is over-quantized
(f64 throughout).
**A-1:** § 8's ledger. **A-4:** § 5 names the first real consumer and the concrete edge; the
support hull is deliberately not exported without one.

**⚠ CONTESTS flags: none.** No finding in this pass contradicts, narrows, or replaces a
user-originated design. F2 and F3 are missing-machinery reports on ratified bones — supplying
what a rule needs, not disputing the rule — and are routed to § 11 rather than edited into
any document. (Checked against corrections #81/#84 before filing: the § IK question is
CLOSED and not re-presented; the placements in dependency-graph § 2b are taken as given, not
re-asked.)

---

## 10. What I could not answer

*"I cannot answer this" is a first-class result.*

- **Whether the bird's folded rest should arrive as an effort-term upgrade or as
  species-authored preferred angles.** The seam (§ 2.4) carries either; choosing is the
  first bird's design pass, and I have no bird and no measurement.
- **What the posture-axis vocabulary is** (F3). The alligator's basking example suggests
  open-per-plan like modes; the sim's crouch machinery suggests the engine enum. Both
  readings are defensible and the choice is a vocabulary-ownership ruling of exactly the
  kind the user made for actions — not mine to make.
- **Whether `root_height_ratio`'s denominator (the governing chain's reach) survives contact
  with the first unequal-chain quadruped.** For equal chains every choice coincides; F4's
  refusal means no body can currently falsify it. Recorded so the quadruped pass re-derives
  rather than inherits it.
- **The precise continuation-slice mechanics of re-basing clips onto a derived hip**
  (§ 5's 20 mm / 120 mm shift). Deliberately unworked here — it is the consumer slice's
  design surface, and working it now would be this slice escaping its NOT-list.

## 11. Open questions, triaged

**User-owned (ratification, not measurement):**
1. **The posture-axis vocabulary** (F3): open per-plan strings (the modes shape) or the
   engine's `Posture` enum? Needed before the crouch bake, not before slice one.
2. **The § 3(b) placement** — pure fn in dc-api, consumer-memoized, migrating to pack
   compile when packs-on-disk land (F1). A nod, not a design session; recorded because
   "where the bake runs" was a ratified-sounding phrase with no today-referent.
3. **The support-patch reading of a sole anchor** (F2): may the bake read the bearing
   segment's bottom face as the contact patch, or must balance run on declared points with a
   tolerance? (No schema change either way.)

**Rides as-built (interim mechanisms, heirs named):**
4. Standing-at-annulus-boundary residual (~µm) once the consumer slice lands (§ 2.2) — the
   solver's existing ε handles it; watch for knee flicker in the live view (temporal ⇒ the
   user's eye is the senior instrument, corrections #77/#80).
5. World-global `CharacterConfig` vs derived body heights — owned by B4.
6. Unequal-chain refusal (F4), stance-width identity (F5), effort seam (§ 2.4) — the § 8
   A-1 ledger.

**Needs measurement (do not decide from this doc):**
7. The measured § 7 report vs this pass's predicted table — divergence is a finding about
   this document (immutable body; the banner obligation falls on whoever measures).
8. CoM/stature and hip/stature for the three plans against the anthropometric bands —
   printed by test 8; if outside band, that is *information about cuboid proxies*, not a
   license to tune.
