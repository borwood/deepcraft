# Posture and gait — derived, baked, sampled

**Status: CAUTIOUSLY RATIFIED — user, 2026-08-01.** The ruling, with its qualifier carried
verbatim because it governs all downstream work:

> *"cautiously ratified, let's write it down and then sketch out first slice."*
> — and, setting the direction: *"their posture and animation, while fully or partially
> baked, should be informed by physics such that it looks **plausible instead of
> arbitrary**. if per species there is a solve for the plausible resting hip, etc etc,
> which gets baked and isn't always solving at runtime…"*

**What "cautiously" binds** — the same reading `refinement.md` established 2026-07-29: §§ 2–6
are the ratified **bones**. § 7's members are **directions, not build orders**; each gets its
own design pass against this document before any build, and a member pass that finds these
bones missing machinery **reports that loudly** — that is the "cautiously" working.

**Provenance, marked because it changes how much weight each part carries**
(the 2026-07-25 rule: *user-originated constraints are data; assistant-originated ones are
hypotheses that happened to survive*).
**User-originated:** posture is what keeps the centre of gravity, so it must be physics-informed
rather than arbitrary · bake per species at gen time, never solve per-frame at runtime · sparse
keyframes with the **server** owning which keyframes we are between and how long the
interpolation takes · ping-pong for symmetric cycles · limb-specific damage so a creature can
carry an injury and a limp (Dwarf-Fortress-like) · one or more AABBs for terrain collision,
higher fidelity for damage · per-species bake with individual deltas **not foreclosed** · the
space-layering diagnosis of the hover · *"the squat does not read realistic."*
**Assistant-proposed (§§ 4–6 mechanisms):** the `(phase, amplitude, duty)` triple ·
`neutral`/`extreme` as the two keyframes · phase-offset replacing mirroring · the Froude/duty
literature anchor · derived collider box sets with bounded `k` and yaw buckets · damage
resolved against the nominal pose · the three non-foreclosure properties · duty coupling as a
bias rather than an override.

Read with: `bodies.md` (the plan/segment/clip data model this builds on) · `north-star.md`
§ *The two clocks* and § *Behavior is code; tuning is data* · `docs/spines.md` **S-9** ·
`refinement.md` (the same bake-and-sample shape, one tier over) · journal/0130, journal/0131,
`corrections.md` **#77 #78 #79 #80**.

---

## 1. The problem, stated as the inverted dependency

Today the pelvis is **pinned** at an authored hip height and the legs are asked to reach the
floor. Reality pins the **feet** and lets the pelvis land where the legs put it.

Every defect measured in journal/0130–0131 is a symptom of that inversion:

- The biped's hip sits at **0.900 m** with legs reaching **0.880 m**, so a planted foot was
  geometrically impossible before any animation ran.
- `dc:body/longleg` — the same body with 12 cm more leg — **squats at −56.2°**, because the
  surplus had nowhere to go but the knee. A real animal with those legs stands nearly
  straight and simply **stands taller**.
- `CROUCH_ROOT_DROP_M` is a fixed **0.45 m**: 50 % of the biped's hip height and **97.8 %** of
  the stout's, whose crouch therefore puts its hip at 0.010 m and folds its knee to 180°.

**And `root_bob_m` is a leaked requirement** in the sense `ARCHITECTURE.md` § *A summary is
not an authority* already forbids. A walk's vertical bob is not a curve — it is what happens
when you alternate stance legs of fixed length. Authoring it as keyframes creates **two
authorities for one quantity**, which is why they cannot be reconciled and why feeding the bob
into the solver would only have made them argue more politely.

**The test this document applies throughout:** *if this quantity has a physical determinant,
it is an OUTPUT.* Hip height, resting knee angle, stance width, bob amplitude, foot spacing
and cadence are all outputs. What remains authored is the body, the contact schedule, and the
limits.

## 2. The governing priors (all ratified; none re-argued here)

1. **The two clocks.** Gen time is free; **runtime is sacred**. Expensive derivation happens
   at bake; the runtime samples. This document is that doctrine applied to bodies.
2. **Behavior is code; tuning is data.** The solver is code, the baked result is data —
   which is what lets a baked posture reach the sim without the sim ever solving.
3. **S-9 — derivable base + sparse committed facts + fallback query.** The species bake is
   the base; per-instance injury or variation is the sparse overlay. *Third instance of this
   spine in one design conversation* (meadow↔tuft, swarm↔individual, species gait↔this wolf's
   limp), which is why individual deltas are not a new mechanism to invent.
4. **Measure against the literature.** A constant tuned until an output looks right is a
   number pretending to be a mechanism. Gait has published scaling laws — this is what makes
   § 4 checkable rather than fitted.
5. **The determinism firewall** (`bodies.md`): the sim must not depend on client-sampled
   state. § 5 moves *where* the line sits without weakening it.
6. **Seam-first.** Identity defaults, byte-identity as the acceptance test for a conversion,
   and a stand-in annotated in code with its named heir.

## 3. The machine — three layers, two clocks

| | bake (per species) | sim | client |
|---|---|---|---|
| when | pack build **or deeptime worldgen (venue corrected — corrections #86)**; gen-time free | per tick, 20 Hz | per frame |
| owns | posture, gait vector, collider sets, keyframes | **phase** + injury deltas **+ the GAZE (see ⚠ below)** | cosmetic refinement |
| cost | irrelevant | a handful of scalars per entity | a lerp per joint |
| determinism | pure fn of the body definition | replay-critical | free |

- **Bake** takes `(segment tree + masses)` and emits the resting posture, the gait vector, the
  keyframe pair, and the collider box sets. It is a **pure function of the body definition
  with no world involvement**, ~~so it runs at *pack build* rather than world gen~~ —
  **VENUE CORRECTED 2026-08-02 (user; corrections #86): purity makes the bake callable from
  ANY clock, and the arc's own driver requires the deeptime clock** — pack build for authored
  species (cacheable, shareable, a third-party pack gets it for free), **deeptime worldgen
  for species the evolution pack mints** (deterministic for free; inherited down the
  phylogeny exactly like the frond bake below, which was always ruled "at deeptime"), and
  define-time for MCP-authored plans. The venue is the caller's; the purity is the contract.
- **Sim** owns the small, quantized **phase** — which keyframes we are between and how far —
  plus any per-instance deltas. This is the user's proposal and it is what makes the pose a
  pure function reconstructible identically in sim, client and replay.
- **Client** adds what is genuinely cosmetic: ~~look-at~~ **the neck BEND (see below)**,
  expressive layers, and the IK foot adjustment onto the actual terrain under this creature
  right now.

> **⚠ CORRECTED 2026-08-02 — corrections #94. "LOOK-AT" NAMES TWO MECHANISMS AND ONLY ONE OF
> THEM IS CLIENT-SIDE.** The **GAZE** (`CharacterState.yaw/pitch`) is **sim state**: it aims
> perception — `sense_raycast` with no explicit direction rays along `view_dir()`
> (`host.rs:1430-1432`) — rides the command log, is replay-tested, and since 2026-08-02 is
> *derived in the tick step* when unheld (look ownership, `bodies.md` § who owns the look,
> journal/0142). The **NECK BEND** — how the declared look joint expresses that gaze, i.e.
> `resolve_orientation`'s cervical clamp and the trunk drag beyond it — is the client-side,
> cosmetic, free-to-tune half § 5's argument actually needs. Read every "look-at" in this
> document as the *bend*. **The layer table's vocabulary is fixed at the gait-bake design
> pass**, which owns the firewall-adjacent naming; it is not repaired unilaterally here.

**Structurally identical to the frond bake** (the vegetation thread, same conversation):
expensive derivation per species at build time, inherited down the phylogeny, sampled cheaply
at runtime. Two instances of one shape; if a third appears it is a primitive.

## 4. The gait — two keyframes and a per-limb triple

**The two keyframes are `neutral` and `extreme`**, not "left forward" and "right forward". A
limb's motion is a phase-driven traversal between them; ping-pong falls out for free.

**The gait is a per-limb triple, and it is where all the expressiveness lives:**

| term | meaning | composition |
|---|---|---|
| **phase offset** | where in the cycle this limb sits | additive, mod 1 |
| **amplitude** | how far toward `extreme` it travels | multiplicative |
| **duty factor** | fraction of the cycle spent in **stance** | additive, then renormalised (§ 6) |

**Phase offset replaces mirroring, and that is the point.** Every limb plays the *same* cycle,
shifted in time — left leg at 0.0, right at 0.5. They are not mirror images. This deletes the
left/right asymmetry hazard structurally: today `arm_l_upper` and `arm_r_upper` are two
independent lists of magic numbers whose symmetry is a coincidence of hand-typing, and that is
precisely the defect that opened this arc.

**Why this parameterisation and not another — it is the one biomechanics already uses**
(prior 4, and this is the clause that makes a baked gait falsifiable):

- **Gait taxonomy *is* phase offsets.** Walk, trot, pace, bound, canter and gallop are the
  same four limbs at different relative phases. Nothing else changes.
- **Duty factor is the walk/run discriminator** — above 0.5 walking, below 0.5 running — and
  the transition occurs near a predictable **Froude number** (`v²/gL`), with cadence scaling
  as `√(g/L)`. So *"a big animal takes slow steps"* is derivable, not tuned.
- Consequence: **the bake's output can be checked against published bands**, exactly as
  denudation rates were. This is the standing bar for a constant being evidence.

**A limp is a TIMING asymmetry, not a pose asymmetry.** You do not hold the hurt leg in a
different shape so much as you **get off it faster**. So an injury perturbs `amplitude` and
especially `duty` — same two keyframes, different numbers. This is what makes the
Dwarf-Fortress-like injury loop affordable, and it is *legible*: a player sees **which** leg is
hurt because the gait derives from which leg is impaired.

**Scope limit, stated so "two keyframes" is not oversold.** This covers **periodic
locomotion only**. Transitions (start, stop, turn), one-shots (jump, swing, attack) and
upper-body action during a walk are **not gait** and continue to use the clip machinery of
`bodies.md`. This is a specialised layer, never a replacement for all animation.

## 5. Posture, colliders, and damage

> **⚠ SCOPED 2026-08-01 (user, cautiously ratified) — BALANCE IS THE RULE FOR *STANDING*
> BODIES, NOT THE DEFINITION OF POSTURE.** Posture is determined by **how a body is
> supported**. **⚠ AMENDED 2026-08-01 (user, same day): SUPPORT IS A PER-SEGMENT CAPABILITY,
> ACTIVATED PER MODE — there is no "a body declares its support kind", and no separate
> support-kind concept at all.** A **segment declares the support it is *capable* of**; a
> **mode declares which capabilities are *active*** — capability on the part, activation by the
> mode, exactly the shape of the open action vocabulary (a body declares what it *can* do;
> what it is *doing* is separate). So support collapses into **roles (§ 7b) + modes**, both of
> which already exist, and **modes become load-bearing rather than a gait detail.**
> **The alligator is what exposed it**, and it breaks the earlier framing twice over: it has at
> least four support modes (high walk on four feet; sprawling walk with feet **and belly**;
> belly slide; swimming with **no ground contact at all**), so *"the body's support kind"* has
> no referent — **and a single stance can MIX kinds** (sprawling: feet *standing*, belly
> *distributed*, simultaneously), which every earlier test case was too pure to reveal. The
> same part also changes role between modes: the belly bears weight sprawling, none in a high
> walk, and touches nothing swimming — **so support is not a static property of a segment
> either.**
> **Cost, flagged rather than hidden: the bake gains a MODE axis** (an alligator's high-walk
> posture and its basking posture are different postures). Net table size is unchanged — this
> trades for the growth axis dropped in decision 24.
> **Bonus: buoyancy is a fourth rule over the same inputs** — how a body floats is centre of
> buoyancy against centre of mass, which needs exactly the per-segment volume and density the
> mass integral already wants. Four consumers of one integral now: harvest yield, evolutionary
> fitness, standing posture, flotation. Balance (CoM over base) governs **standing**
> bodies. Other support kinds get other rules: **anchored** — structural bending under self
> weight and wind (a tree); **lying/distributed** — shape, with no balance problem at all (a
> snake, whose CoM is supported everywhere, so the balance rule is *satisfied by every possible
> shape and therefore determines nothing*); **suspended/airborne** — neither.
> **Left universal, the balance rule is not merely vacuous but specifically WRONG for plants:
> it implies a straight tree**, and real trees lean and sag because statics, not balance,
> shapes them.
> **Built as scoped-now, implemented-narrow:** the support kind is **declared today** so the
> vocabulary exists and nothing is foreclosed; only the **standing** rule is implemented, and a
> non-balancing body is **loudly unsupported** rather than silently given a nonsense posture.
> *Without this, "posture = balance" hardens into the definition of posture — anti-shape A-1,
> the same failure as the four fixed distances and the closed verb list.*

**Posture is the joint configuration that puts the centre of mass over the base of support at
tolerable effort** *(for a **standing** body — see the scoping banner above)*. It is computable because each segment carries volume and — once the
material work lands — mass. *The same mass integral serves harvest yield, evolutionary
fitness, and standing posture*; that is the strongest argument for per-segment materials and
it was arrived at independently from three directions.

**Colliders are derived, not authored.** The bake emits a small ordered set of AABBs covering
the segment tree, growing `k` until the boxes stop badly over-covering the true segment volume.
A biped → **1 box, exactly today's collider** (the identity default). A horse → 2. A snake →
several. **The engine caps `k`**, and that cap is the honest place to spend a limit. Movement
sweeps each box and takes the most constrained result. Posture and growth stage select a
different baked set; the existing stand-up embed guard generalises directly.
**AABBs do not rotate** — bake per **yaw bucket** (4 or 8), which degenerates to one entry for
anything rotationally symmetric.

**Damage resolves per segment, against the NOMINAL pose** — the pose the sim can reconstruct
from `(clip, phase)`, not the frame the client happens to draw. Both sides can do this because
the clips already live sim-side (`AnimClipDef` in `HostState`); only the *sampler* was ever
client-only. So the firewall's line moves from *"the sim knows no pose"* to **"the sim knows
the nominal pose; the client refines it cosmetically"** — a better line, because it is the one
that decides fairness and replay. Locational damage then falls out **for any body plan,
including evolved ones**, because the hitboxes are the creature's own parts.

**The cost of that move, stated plainly: it converts animation from free-to-edit into a
versioned sim asset.** Retiming a clip changes hit detection, hence replay and world identity.
This is why ~~look-at~~ **the neck bend (corrections #94: the GAZE is sim state and aims
perception — it is the *bend* that is cosmetic)**, expressive layers and foot IK must stay
firmly *outside* the sim-visible set — so at least those remain free to tune.

## 6. Not foreclosing per-instance deltas — three properties, no machinery

The bake is **per species** (user, 2026-08-01: *"of course… it's not baking for every
individual wolf in existence"*). Individual deltas are wanted eventually; today we only owe
**not foreclosing** them. That costs three properties and zero mechanism:

1. **Bake PARAMETERS, not FRAMES.** Emitting final per-frame poses destroys duty and phase
   permanently. Emitting `(neutral, extreme)` + the triple makes a delta arithmetic. *A
   "make it fast" pass would quietly destroy this by pre-composing poses — the single
   highest-risk foreclosure.*
2. **The seam is a FUNCTION SIGNATURE, not a struct field.** Gait living *inside* `BodyPlan`
   is per-species by construction with nowhere for an instance to speak. `pose(species_gait,
   instance_delta)` with an identity default costs nothing and is the proven pattern (the four
   deep-sim providers: empty plane + identity accessor, byte-identity tested). **Acceptance:
   an uninjured creature reproduces the baked gait byte-identically.**
3. **Do not over-quantize the baked gait.** If duty is stored in 4 bits, a 3 % limp is
   *inexpressible* — and we would rediscover `corrections.md` #80 in a new costume, having
   just paid for it twice.

**The one coupling that is not free: duty factors are not independent.** A biped cannot have
both legs reduce stance time arbitrarily — somebody must be on the ground, or it is a
different gait. So an injury delta is a **bias** (*"favour this limb by k"*) followed by a
deterministic renormalisation, **never a per-limb override**. Deciding that shape now matters,
because per-limb override is the obvious API and it is the one that produces physically
impossible gaits.

## 7. Members — directions, not build orders

> **⚠ Each requires its own design pass against §§ 2–6 before any build, and is revisited in
> light of this document rather than taken as wherever it landed prior.**

0. **The resting-posture bake.** The first slice; scoped in ROADMAP § Sequenced.
1. **The gait bake** — duty and cadence from leg length and speed, against the published
   Froude band; phases from the gait type.

   > **⚠ DOCKET EXPANDED 2026-08-02 — user-ratified** (*"your reasoning here is sound and i
   > want the repo to stay aligned to it on this sprint"*), in answer to two user questions
   > this document could not answer as written. **Provenance: assistant-originated reasoning,
   > user-ratified** — so it is **data** for the member pass, not a hypothesis the pass may
   > quietly reconcile away.

   **(a) The honest input inventory — state it before deriving anything.** The bake's entire
   input set today is **segment geometry plus volume as the mass proxy at density ≡ 1**
   (member #0's design pass; B6 is the named heir at that very line). There is **no strength,
   no muscle, no weight, no metabolic term**, and `CharacterConfig.walk_speed_m_s = 4.5` is a
   world-global constant that knows nothing about the body (B4 retires it). Therefore:
   **length-scale facts are derivable and honest today** — cadence ∝ √(g/L), stride, phase
   offsets, hip height, stance width — and **anything depending on FORCE is not**: the Froude
   number's speed term, effort minimisation, acceleration, load carrying, transition
   energetics. The pass states this boundary explicitly, because it bounds what a derived
   gait may *claim*, and because both of the user's questions turned out to hinge on it.

   **(b) Authored KNOBS are wanted, and the doctrine permits them.** *"A constant tuned until
   an output looks right is a number pretending to be a mechanism"* forbids a knob that
   **replaces** the derivation; it does not forbid one that **rides on top of** it. Four
   kinds, ranked by how safe they are:
   1. **Underdetermined style parameters — the largest and safest space.** Length scale fixes
      *when* feet land and says almost nothing about how a body carries itself between
      landings: foot lift above minimum clearance, trunk counter-rotation, head bob against
      the trunk, tail carriage, shoulder roll. Physics does not pin these, so authored taste
      here **fills a genuine void rather than fighting a mechanism**. Most of "shape it to my
      liking" should live here.
   2. **Gait-type selection.** Walk / trot / pace / bound / canter are all physically valid
      for four limbs; *which* a species uses and *where* it transitions is a **biological**
      fact, not a geometric one (a giraffe paces where a horse trots). Expressive, honest,
      and it is just phase-offset data.
   3. **Bounded dimensionless modifiers on derived terms** (`cadence_scale: 1.15` — "quicker-
      stepping than its size predicts"), with an **identity default** so no-knob is
      byte-identical to the pure derivation (S-5). **The bake REPORTS when a modifier pushes
      an output past the published band** — never refuses it (a clockwork golem may want to
      step wrong), but says so. That makes *measure against the literature* an **instrument
      for the author** rather than a cage.
   4. **An authored-clip override for a `(mode, speed band)`** — the escape hatch for things
      that should move wrong. It costs the evolution property (an override cannot survive
      topology mutation), so it must **degrade to the derived gait rather than break**.

   **(c) The distinction that must not be missed: TASTE knobs vs STAND-IN knobs.** Kinds 1–2
   are permanent taste. Kind 3 is mostly a **stand-in for physics we have not built** —
   *"quicker than its size predicts"* plausibly becomes *"more muscle in its legs"* once B6
   lands, derived rather than authored. **Design the two sets so B6 ABSORBS the stand-ins
   rather than colliding with them, and give each stand-in a `stubs.md` entry naming B6 as
   its heir.** A knob that becomes a lie the moment materials arrive is precisely what
   `root_bob_m` just cost us (corrections #93).

   **(d) Non-locomotion animation stays keyframed, and BOTH authoring routes keep it.** § 4's
   scope limit is load-bearing: transitions, one-shots, emotes and upper-body action during a
   walk ride `bodies.md`'s clip machinery, reachable from pack build **and from a live MCP
   session** through the same `DefineAnimClip` door. The pass states the **composition rule**
   (gait and clip over **disjoint role sets** — § 7b's centaur) and the
   **coexistence/migration** answer for authored *locomotion* clips specifically: legal
   override per (b)4, or retired. Today's three bootstrap clips are the first case to name.

   **(e) The layer table's vocabulary** — corrections #94's **gaze** (sim state, aims
   perception) vs **neck bend** (cosmetic) split — is **fixed here**, since member 2 moves
   the firewall line that naming sits on.
2. **The sim-side phase tuple** — and with it the firewall's new line and the 20 Hz / 12 fps
   cadence question (0.05 s and 0.0833 s do not divide; the stepping must land evenly, and
   *that is a choice about the stop-motion identity, not a technicality*).
3. **Derived collider sets** — and the retirement of the world-global `CharacterConfig`.
4. **Per-segment damage** and the injury→gait-delta loop.

## 7b. Identity: address and role (CAUTIOUSLY RATIFIED 2026-08-01, user — *"fold in with caution"*)

A segment carries **two** identifiers, not one:

- **address** — *where it sits in the body*. Unique, structural, generated-friendly
  (`trunk/branch[2]/leaflet[3]`). This is what makes a fern's 300 leaflets expressible.
- **role** — *what it is*. Declared, shared across different bodies, **and this is what
  animations and systems bind to.**

Today's `SegmentDef.name` fuses the two, which is why it can express neither a fern (needs many
addresses) nor a stinger (needs an open role vocabulary). **A name is a poor man's role: it
works when everyone types the same string and fails SILENTLY when they don't.**

**The user's worked example is the proof.** An upper-body clip should drive a centaur, because
a centaur's trunk/arms/head fill the same roles as a humanoid's. Under roles it wears a
humanoid upper-body animation *and* a quadruped gait simultaneously, over disjoint role sets,
and the engine checks at define time that both are satisfied — the *"did you supply what you
claimed"* shape of the open action vocabulary. **Roles preserve the centaur property more
strongly than names, because a mismatch is caught rather than silent.**

Same mechanism serves the scorpion's **stinger**, the wolf's **snout**, the rabbit's **ear**,
a fern's **foliage**, and a body's **support kind** — all one declaration.

**Named sub-question, not hand-waved:** roles are **many-to-one over addresses** (300 `foliage`
is the point), but a clip animating *the* left upper arm needs it to resolve to exactly one. So
roles need a **cardinality** notion the validator enforces. Designable, not a footnote, and the
part most likely to be got wrong first.

**Timing:** segment identity is a **free** change today — nothing persists a `BodyPlan`, so it
is a recompile rather than a migration — and that window **closes** when damage resolution
moves the firewall (§ 5).

**RULED 2026-08-01 (user), disposing of the named sub-question above — after a defeat-check
sweep of every sketch that touches binding.** **B0 lands roles for SYSTEMS** — contact anchors,
support capability, functional parts, the look-at joint — and **clip binding stays
address-exact**. Role binding for animation is a **named heir, decided at the gait-bake
member's design pass** (`stubs.md` § 34), and it must land **before member 2 moves the
firewall** — after that, the binding key of a versioned sim asset is a migration rather than a
recompile. **Cardinality enforcement travels with the heir.** It is not a clip-only question:
B0's first system consumers resolve a role to **exactly one segment or fail loudly at query
time** — already stronger than the name lookups they replace, which miss silently.
Two findings carried the call:
**(1)** § 4 makes it likely the gait bake *dissolves* the many-to-one case for locomotion —
every segment carrying a role plays the *same* cycle at a derived phase offset, and the
shipped walk clip already IS that (measured: body-plan-structure design pass § 4) — so a
selector grammar built now risks being machinery the next tier obsoletes;
**(2)** the clip is not the only name-keyed binder: clothing binds by segment name
(`bodies.md` § Clothing, DECIDED), plan forking inherits clips by retained joint names
(§ Body plans, DECIDED), and sockets mount on segments (PROPOSED). A binding vocabulary
designed today from the clip case alone is a vocabulary designed for one consumer — the exact
hazard `ideas.md` § procedural-attacks flags — and all four migrate through the same heir.
Until then, cross-plan clip reuse rides as it ships today: shared names, working and
unchecked (measured, journal/0130).

## 8. What this document does NOT decide

The concrete SDK types · whether the effort term is a real minimisation or static geometry
(§ 5 assumes the cheapest thing that gets hip height right) · the growth-stage axis (four bake
keys — species, posture, growth, yaw — and the product is what sits in memory; **if growth is
not happening soon, dropping that axis keeps the table small and it is re-addable**) · whether
`k > 1` collider boxes ever need to articulate (assumed rigid; let the first creature that
needs otherwise make the case) · ~~the hip/reach and `root_bob_m` disposition in `bodies.md` § IK, which remains an open user
call~~ — **STRUCK 2026-08-01, corrections #81: § 1 ALREADY DISPOSES OF IT, and this sentence
contradicted § 1 in the same sitting.** *If a quantity has a physical determinant it is an
output* deletes the 0.900 m hip (nothing authored is left to be 20 mm wrong), deletes the
clips' `root_bob_m` as emergent, and deletes `CROUCH_ROOT_DROP_M` because crouch is a
**posture** and § 3 already keys the bake on one. Only the **half-voxel foot window** survives,
and it is an engineering question, not a ratification: its job becomes absorbing real terrain
relief, at which it fails **2:1 by construction at every N**.
**User rulings, 2026-08-01, which close the residue:** *"assume that our loadbearing segments
will not have an artificial gap between the mesh and the ground"* — the 20 mm is a **defect,
not a design**; and *"anything that is currently only meant to support a biped, or is
presumptive about possible size/proportion, is going to get refactored"* — the presumptive
constants are **scheduled demolition, not an open question**. The bob was never ratified.

## 9. Compliance

**North star:** the two clocks, applied to bodies; bodies are already named a core data-model
API, so the **solver is an engine primitive** (same argument as the field kernels — only the
kernel knows its own bound) while **bodies are pack content** and the **bake is derived data in
the pack's compiled form**. Behavior is code; tuning is data. No capability tiering (Deviation
2) — a pack authors a body the same way the defaults do.
**Spines:** S-9 (§ 6, third instance) · S-5 identity defaults (§ 6 property 2) · S-3 — the
baked posture must never become an authority the physics reads back; it is derived *from* the
body, and if a sim pass ever keys on the bake instead of the body that is the
summary-wearing-authority defect. **A-1 is what this whole document retires**: `root_bob_m`,
the 0.900 m hip, `CROUCH_ROOT_DROP_M` and the half-voxel window are four stand-ins that became
definitions because, with one body plan, *a length is a ratio*.
