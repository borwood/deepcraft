# Bodies, body plans, sockets, animation

> **▶ POSTURE AND GAIT MOVED OUT, 2026-08-01 — see
> [`posture-gait.md`](posture-gait.md) (CAUTIOUSLY RATIFIED).** This document remains the
> **data model**: plans, segments, sockets, clips, the determinism firewall, the staircase.
> **What a body's resting posture, hip height, knee angle, stance width, bob amplitude and
> cadence should be is no longer authored here — those are OUTPUTS** of a per-species bake from
> `(segment tree + masses)`. Where this document's § IK and § stepped animation describe the
> current clip-plus-correction pipeline, they describe **what exists**, not the direction; both
> carry banners recording what was measured against them (journal/0130, journal/0131,
> corrections #77 #78 #80). **And § stepped animation is now half retired: the ROTATION
> quantization was removed 2026-08-01 by user ruling; only the 12 fps time step remains, and
> that too is an open taste call.**

Status: core decisions RATIFIED 2026-07-19 (marked DECIDED below); sections
marked PROPOSED are sketches, NOT decisions. Expands the ratified seeds in
materials.md § forms (body-as-plugin, sockets, transmog-as-body-swap) and
API.md § Characters (character/controller split). Sequenced item 3b.

## The three-layer split

```
character  — identity + persistence (who): account-owned, survives everything
body       — physical presence (what):    an instance of a registry body plan
controller — volition (who's driving):    human / MCP session / WASM / NPC-tier / nobody
```

The character surface (journal/0005) shipped layers 1 and 3 with a
hard-coded body. This doc is layer 2.

## Body plans — DECIDED 2026-07-19

> **⚠ TWO HALVES SUPERSEDED 2026-08-01 (user; log: ROADMAP 2026-08-01 close block,
> `posture-gait.md` § 7b):** **(1)** the verb→slot contract's *closed vocabulary* is retired —
> the action vocabulary **OPENS** (packs declare actions; the engine checks you supplied what
> you claimed; `KNOWN_VERBS`/`REQUIRED_VERBS` go with the B0 structure slice). The
> define-time-checking *principle* stands. **(2)** name-keyed binding — clips to joint names
> here, forking's clip inheritance by retained names, § Clothing's segment-name binding,
> § Sockets' segment mounts — is a **stand-in**: `posture-gait.md` § 7b ratifies that
> animations bind to ROLES. The binding-key migration is `stubs.md` § 34's heir, decided at
> the gait bake, **before the firewall moves**.

Fidelity target: **above Minecraft, still cuboid** — jointed limbs, several
body segments, a neck. A **body plan** is a registry contract: a named joint
topology (segment tree). Concrete bodies are *parameterizations* of a plan —
segment scales, proportions, textures, stat-driven modifiers — so N humanoid
mobs are mutations of one humanoid plan, a quadruped plan covers most
animals, and so on. The fourth instance of the roles-as-contracts backbone
(geology classes, form archetypes, geology passes, now body plans).

- **Animations bind to the plan, not the mob** (processes bind to classes,
  not instances). Every mutation of a plan shares its animation set;
  proportion differences are absorbed procedurally (IK, below).
- **Forking**: a child plan that retains a parent's joint names inherits the
  parent's clips for those chains; overrides are allowed per plan and per
  body instance.
- **Verb → animation-slot contract**: a plan declares which driver verbs it
  supports (move forward/backward, strafe, jump, idle, prone, crouch, crawl,
  overhead swing, thrust, …) plus blendable states (afraid, agro, happy, …).
  Registration **fails at define time if a required anim slot is unfilled**
  — the same schema-checked-at-define-time rule as everything else.
- **Plans and clips are data, defined via editor/MCP** through the registry
  (keyframes are just payloads). A posing/keyframing GUI is a later tooling
  milestone; MCP-authored clips work from day one — including Claude
  authoring animation through the dev surface and iterating from
  screenshots.

## The determinism firewall — DECIDED 2026-07-19

**Animation is cosmetic. The sim sees only parametric posture states.**

- Sim side: discrete/parametric posture (standing / crouch / prone collider
  heights on the swept-AABB mover), action reach + timing as command data.
  Bit-identical replay untouched; headless crates stay render-free.
- Render side: the skeleton — clip playback, blending, IK, emotional
  additive layers — derived on the client, never read back into simulation.
- Example (crouch): "raycast up, clamp collider height" is sim;
  "IK the spine and head under the ceiling" is renderer.

## IK — DECIDED 2026-07-19 (role, not solver choice)

> **⚠ MEASURED 2026-07-29 (journal/0130) — THIS SECTION IS HALF-CONFIRMED AND HALF-REFUTED.**
> **Confirmed:** the retargeting role is real. One unmodified clip set drives
> `dc:body/biped` and `dc:body/stout` (legs 0.50x, arms 1.60x) with no new keyframes;
> the derived leg rig comes out at exactly 0.500x, `idle` poses are byte-identical
> across the two plans, and the walk's artifacts *shrink* in metres (0.102 -> 0.068 m).
> No new kind of artifact appeared, and the second body reads as a coherent creature.
> **Refuted: "feet to actual ground" has never happened, for ANY plan.** 176/176 sampled
> frames are beyond the leg's reach and clamp to full extension. The biped's hip sits at
> **0.900 m** while its legs reach **0.880 m** (0.45 + 0.43), so a sole at y=0 is
> **geometrically unreachable before any animation runs** — a 20 mm gap present since
> `biped_plan()` was written. The rendered result is a **hover**: `idle` sole height
> oscillates over `[+0.020, +0.035] m`, feet never planted (user, on seeing it move:
> *"the hover looks bad… it is not a bob, it is a hover"*).
>
> **The mechanism generalises past IK and is the reason this matters to
> § "Plan parameters":** three quantities are **absolute metres** in a pipeline whose
> premise is that proportions vary — hip height, the clips' **root bob** (jump 0.120 m =
> 13% of the biped's hip height and **26%** of the stout's; both bodies' feet peak at
> *exactly* +0.125 m), and the **foot-IK correction window**, which is **half a voxel**
> (0.450 m at N=2 = **1.02x the stout's entire leg**) — a tolerance derived from voxel
> resolution wearing the clothes of one about anatomy. **With exactly one body plan a
> length IS a ratio**, which is why none of it was visible: anti-shape **A-1**, a stand-in
> becoming the definition, sitting under the body builder's foundation.
>
> **Consequence for authoring: every plan parameter must declare its UNITS**
> (ratio-of-plan vs absolute-metres) as well as which side of the determinism firewall it
> touches. The units axis is undetectable while there is one plan and load-bearing the
> instant there are two.
>
> **⚠ RESOLVED 2026-08-01 — corrections #81. THE "USER CALL" BELOW NO LONGER EXISTS**; it was
> dissolved by `posture-gait.md` § 1 (*if a quantity has a physical determinant it is an
> output*) on the day that document was ratified, and then re-presented to the user twice
> before anyone noticed. **Hip height, the root bob and the crouch drop are all deleted rather
> than corrected** — they are outputs of the per-species bake. The user's rulings: *"assume
> that our loadbearing segments will not have an artificial gap between the mesh and the
> ground"* (so the 20 mm is a **defect**, settled by ruling forward rather than by archaeology
> into intent) and *"anything that is currently only meant to support a biped, or is
> presumptive about possible size/proportion, is going to get refactored."* Only the
> **half-voxel foot window** survives, as an engineering fix. **Nothing here is owed to the
> user.** Preserved below as dated testimony:
>
> **⚠ ~~USER CALL, unresolved~~ (struck — see above):** whether the 20 mm gap is intentional (feet visually
> clearing terrain seams) or an off-by-a-half-thickness in `l2`. It decides whether these
> three constants become **ratios of the plan** (an engine change that invalidates the
> authored clip bobs and moves how every body looks) or whether *"feet to actual ground"*
> is **retired from this section** as never-intended. **Nothing downstream may assume
> ground contact until it lands.**
>
> **⚠ MEASURED AGAIN 2026-07-29 (journal/0131) — THE SOLVER WAS
> NEVER THE PROBLEM, AND THE CALL ABOVE IS STILL THE USER'S.** A third plan,
> `dc:body/longleg`, was authored *purely as a control*: the biped with its two leg bone
> lengths changed and **nothing else** (reach 0.880 → 1.020 m against an unchanged 0.900 m
> hip), riding the same three unmodified clips, in `experiment_body_pack()` — **no engine
> constant was touched, and no part of the call above is answered here.** With the sole
> target inside the annulus the closed-form IK solves on the first frame it is asked:
> **86 of 88 samples plant** (a sample is one foot on one stepped frame; 88 per plan,
> which is exactly half of journal/0130's 176 across two plans), **and the knee bends
> to −56.2°.** The two failures are `walk`'s stride extremes, which need
> `hypot(hip, stride_forward)` of reach, not `hip`.
>
> **The new fact, and it constrains the call:** the resting knee bend and the stride
> coverage are **locked together** and trade against each other. Measured over the
> authored clip set — `idle` 24 frames, `walk` 12, `jump` 8, both legs, at the 12 fps grid:
>
> | leg slack (reach − hip) | resting knee bend | samples still beyond reach (of 88) |
> |-------------------------|-------------------|-----------------------------------|
> | −0.020 m (today's biped) | 0.0° — clamped straight | **88**, every one |
> | +0.030 m | 29.2° | 32 |
> | +0.050 m | 37.3° | 14 |
> | +0.120 m (`longleg`) | 56.2° | **2** |
> | +0.180 m | 67.1° | 0 |
>
> There is **no slack that both plants every frame and keeps the knee out of a squat**,
> because the stride's reach demand grows with the bones that serve it. The degree of
> freedom nobody is spending is the root's **vertical travel** — the clips author a bob
> *upward* where a walk needs the pelvis to *drop* on the stance leg. That observation is
> offered as evidence for the call, not as a resolution of it.
>
> **⚠ THE UNITS LIST ABOVE IS SHORT BY ONE: there are FOUR absolute-metres constants,
> not three.** The paragraph beginning *"three quantities are absolute metres"* names hip
> height, the clips' root bob, and the foot-IK window. It misses
> **`CROUCH_ROOT_DROP_M` = 0.45 m** (dc-client `body.rs`) — which is **50.0% of the
> biped's hip height and 97.8% of the stout's**, so a crouching `dc:body/stout` puts its
> hip at **0.010 m** and folds its knee to a degenerate **180°**. That constant is not a
> footnote to the others: because it moves the hip *toward* the ground, it is **the one
> posture in which the stock biped's foot IK has always worked** — 88/88 samples
> corrected, knee bending to 123.7°. journal/0130's *"never engaged"* is true of
> **standing**, and was never true of crouching. (Corollary in CLAUDE.md § reading a
> null, landing on the entry that established it.)
>
> **⚠ AND FOOT PLACEMENT HAS TWO INDEPENDENT GATES, WHICH NOTHING RECONCILES.** A
> correction happens only if the ground is inside the **half-voxel window** *and* the
> sole target is inside the **annulus** `[|l1−l2|, l1+l2]`. The three plans fail
> different gates in different postures, and **no plan passes both in every posture**:
>
> | | flat, standing | flat, crouching | +0.30 m step (synthetic) | one-voxel step |
> |---|---|---|---|---|
> | `biped` | annulus refuses, 0/88 | **88/88 plant**, knee 123.7° | uphill foot plants, knee 90° | window refuses raised foot |
> | `stout` | annulus refuses, 0/88 | 88/88 plant, knee **180°** (degenerate) | uphill foot plants, knee 135° | window refuses raised foot |
> | `longleg` | **86/88 plant**, knee 56.2° | **window refuses, 4/88** | 87/88 plant | window refuses raised foot |
>
> `longleg` standing works and crouching does not; `biped` crouching works and standing
> does not. That complementarity is the sharpest available statement that the window
> (voxel-derived) and the geometry (plan-derived) were never reconciled.
>
> **⚠ AND A THIRD, INDEPENDENT REFUSAL, SCALE-FREE:** the correction window is **half a
> voxel** and the smallest relief real terrain can have is **one whole voxel**, at *every*
> scale N — the ratio is fixed at **2:1 by construction**. So **no real terrain step ever
> fits inside the window**: on a one-voxel step every raised-foot sample is refused, on all
> three plans, for all three clips (asserted in dc-client `body.rs`
> `foot_placement_and_retargeting_are_measured`). Foot placement engages only on
> *sub-voxel* offsets — which, today, are produced by nothing but the plan's own hip/reach
> mismatch. **The window's units problem is therefore not "half a voxel is the wrong
> number"; it is that a voxel-derived tolerance can never express a terrain step.**

Two-bone IK for limbs + neck look-at is the **retargeting glue** that makes
one clip serve every mutation of a plan: feet to actual ground, hands to
actual socket transforms, across differing proportions. Solver technique is
an implementation choice; artifact suppression uses standard techniques
(pole vectors, joint limits) — cuboid rigs do not inherently pop.

## Stepped animation — DECIDED 2026-07-19 (aesthetic choice)

> **▶ THE ROTATION QUANTIZATION IS REMOVED, 2026-08-01, BY USER RULING. THE 12 FPS STEP
> RIDES UNCHANGED.** *"let's stop treating as a constraint we must satisfy and subtract the
> rotation quant. 12fps can ride until we have more opportunity for human to see action in
> game and make a more informed taste call."* `ROT_QUANTUM_RAD`, `quantize_angle` and
> `stepped_angle` are gone from `dc-client/src/body.rs`, along with every call site
> including the IK output path in `character.rs`. **Joint angles are now exact radians.**
> `ANIM_FPS = 12.0` and `quantize_time` are untouched, as are `BOB_QUANTUM_M` (a positional
> snap with a float-robustness job) and `BLEND_STEPS` (a blend *weight*, not an angle).
> Nothing was added in the quantizer's place — no flag, no knob, no replacement.
>
> **Why: the premise was assistant-originated and is measured false.** The user has stated
> the provenance — *"neither was my idea"*, *"we never actually saw weird IK solves"*. Both
> the 12 fps stepping and the rotation snapping were proposed by an earlier session to hide
> weird rotations it **anticipated** from IK solves; the user rolled with it, and every later
> session read the result as a ratified aesthetic. journal/0131's probe then ran the two-bone
> solver over **3 plans × 4 ground cases × 3 clips = 1,056 samples** and found well-behaved
> knee angles throughout (−56.2°, 90.0°, 123.7°). The single degenerate 180° result traces to
> `CROUCH_ROOT_DROP_M` consuming **97.8 %** of the stout's hip height — **a bad constant, not
> an unstable solver.** The guard was guarding nothing.
>
> **⚠ *"an identity, not a workaround"* IS WITHDRAWN AS TO THE ROTATION HALF.** The prose
> below is preserved as dated testimony, but that clause was never the user's, and the word
> **"forgiving"** in the same sentence is the tell: it is what a workaround says about itself.
> A justification that outlived its premise (anti-shape **A-2**), load-bearing for a year
> because nobody re-read who wrote it. **Full record: `corrections.md` #82; narrative in
> journal/0133.**
>
> **And removing it exposed a test the quantizer had been hiding.**
> `looping_wraps_deterministically` asserted **bit-identity** between `sample_clip(walk, 0.4)`
> and `sample_clip(walk, 1.4)`; `quantize_time` floors on a grid anchored at absolute `t = 0`
> and wraps *afterwards*, so the two are one real number in different bits, and the 11.25° snap
> rounded both to the same grid point. **It passed for a year while measuring `rem_euclid`'s
> final ULP and calling it looping** — anti-shape **A-3**, concealed by the very mechanism this
> ruling removes. Retargeted with a bound derived from the mechanism, not deleted. *A quantizer
> wide enough to hide a defect is wide enough to hide a defect in its own guard.*
>
> **What is NOT decided here: the 12 fps step itself.** It is deferred, deliberately, to a
> taste call the user wants to make **on a body whose feet actually reach the ground** — a
> judgement about motion that no still frame can answer (corrections #77). Until then it
> rides as-is, and this section's DECIDED status covers it and nothing else.
>
> **⚠ CORRECTED 2026-08-01 — corrections #80. THE BANNER BELOW IS TRUE AS ARITHMETIC AND
> WRONG AS A DIAGNOSIS, and it is preserved unedited as dated testimony.** The quantizer is
> the **second** wall, not the operative one. The hover's actual cause is a space-layering
> defect: `character.rs:219` builds the IK's hip as `feet.y + hip_local[1] - crouch_drop`
> — **excluding `root_bob_m`** — while `character.rs:257` adds `root_bob_m` to the rendered
> root. The solver is handed a hip that never moves, returns a correct and **constant** leg
> pose, and the renderer then translates the whole body, feet included, by the bob. Fix that
> and the quantizer becomes the next wall (a ~1° correction against an 11.25° step, and
> `stepped_angle()` is applied to the IK output). **The discriminating evidence was already
> in the report and was misread: the knee angle is CONSTANT across frames while the gap
> tracks the bob one-for-one.** Found by the user watching the body move — the third time a
> temporal observation has beaten an instrument reading (corrections #77, #78, #80).
>
> **⚠ QUANTIFIED 2026-07-30 (ROADMAP § Observed; journal/0130 + journal/0131) — THIS CHOICE
> IS WHY FEET DO NOT PLANT, AND NEITHER JOURNAL KNEW IT.** `ROT_QUANTUM_RAD = TAU/32 =
> 11.25°` means **one quantum of hip rotation moves the ankle 172 mm** on the biped's 0.88 m
> leg. The foot corrections at issue need **1.30°** (the 20 mm standing gap), **0.98°** (the
> 15 mm hover oscillation) and **0.33°** (`longleg`'s residual) — the quantizer is **9×, 12×
> and 35× too coarse.** The IK solves correctly and the rounding discards the answer.
> **Sub-decimetre foot placement is inexpressible under stepped rotation, at any body
> proportion.** This section is not wrong and is not retired — the aesthetic stands — but it
> is now a **known cost with a number**, and it is **upstream of** the § IK hip/reach user
> call: fixing reach cannot plant a foot the quantizer will not move. Options, unratified:
> exempt the IK correction chain from quantization while clips stay stepped; or quantize the
> foot's **position** against the ground instead of the joint's **angle**.

Character animation renders **frame-stepped (~12 fps, quantized rotations)**
— a stop-motion look chosen for the elevated-pixel aesthetic (an identity,
not a workaround; it also happens to be cheap and forgiving).

## Clothing/armor — DECIDED 2026-07-19 (refined same day)

Clothing is **authored as its own set of cuboids**, each parented to a body-
plan segment either **fully** (rigid follow — armor plates, shirts) or **at
a point** (pinned with constrained rotation — a floating cape pins to the
trunk and swings within limits; a soft-body deepening of pinned pieces is a
later option). The earlier textures / transformed-segment-copies method
survives as the **default/fallback authoring path** (a copy is the
degenerate case: one cuboid, fully parented, inheriting segment transform).

Binding is by **segment name**, and authored cuboids are expressed
**relative to segment dimensions** (mechanism note: this is what keeps "one
chestplate def adorns every mob of every plan with those segments,
mutations included" true — absolute-meter authoring would break on scaled
mutations). Pinned/swinging pieces are cosmetic-side per the determinism
firewall. v1 boundary softens accordingly: pinned-rigid pieces (capes) are
in; soft-body is later.

## Plan parameters — PROPOSED (user sketch 2026-07-19)

Body plans may declare **scalar and bool parameters** that bodies bind to
character state:

- **Scalar**: a segment expands/contracts within a plan-declared range,
  driven by a stat (weight, strength) — visible bulk from simulation state.
- **Bool**: a segment is present/absent by character property (e.g.
  sex-dimorphic segments) — a bool param is effectively a *fork you can
  flip per instance*, unifying with plan forking.

Interaction notes (Claude, for discussion — not decided):

- Absent segments drop their animation channels — the same rule as fork
  inheritance — and sockets on absent segments vanish (missing-socket ⇒
  item-to-inventory rule from transmog).
- Each param must declare **which side of the firewall it touches**: purely
  cosmetic scale (visual bulk) is free; a param that alters collider
  extents or reach is sim-visible and must be quantized + deterministic
  (stats are sim state, so this is legal — but stepwise, never smooth
  per-frame).

## Sockets — PROPOSED

A socket is a named, typed mount on a plan segment: `hand.r`, `back`, `head`.

- Acceptance is a predicate on **form archetypes** (materials.md § forms),
  not an item whitelist: `hand.*` accepts `graspable`, `back` accepts
  `strappable`. Sockets × archetypes yield the equipment matrix for free.
- Socketed items are physically present (a torch in-hand is the world's
  torch; drop = detach at the socket transform). Interaction verbs anchor to
  the active socket.

## Transmog / body swap — intent DECIDED (materials.md), mechanics PROPOSED

`entity.set_body(character, body)` as a normal command: controller binding
untouched, sockets re-mapped by name (missing socket ⇒ item to inventory),
mover posture re-validated against new extents (the attach embed guard
applies — growing into solid is the embedded-attach case).

## Implementation staircase — DECIDED 2026-07-19 (order)

1. Joint-tree skeleton + body plan as registry def; companion re-expressed
   as a biped; hand-authored idle/walk as data clips (MCP-defined).
2. Locomotion set + crossfade blending + the verb→slot define-time contract.
3. Two-bone IK: foot placement, neck look-at; parametric crouch (sim state +
   cosmetic pose).
4. Plan forking/inheritance + scale/stat mutations (IK retargeting proves
   out here).
5. Clothing shells.
6. Authoring editor (GUI); MCP data authoring exists from step 1.

Not scheduled against geology; slots in when 3b's implementation is
sequenced.

## Crouch semantics — DECIDED 2026-07-19

Crouch is **not** crawlspace access: the 0.6× factor stands as-built
(walk 11 measured that it earns no passage at N=2's 0.9 m steps — and
that is fine, because passage-through-low-spaces belongs to the future
**prone/crawl** verbs, which will own sub-standing clearances). Crouch's
role is posture: stability, stealth, aim, working under things you can
already fit beneath.

Reaffirmed 2026-07-19 (second session — the ROADMAP entry had gone stale
and re-asked this): crouch's world-interaction verb is the **sneak
edge-walk-block** behavior (a crouched body will not walk off a block
edge), not passage. The prone/crawl posture family is sketched in
ideas.md § posture ladder; nothing there is scheduled.

## Formerly open questions — DECIDED 2026-07-19

1. **Bodies are diegetic items**: found vessels, and eventually craftable
   from materials plus an **animus** — the item that carries a controller
   binding (an MCP session, a WASM script, …). The automation sketch's
   golem is exactly body + script-animus; the animus makes "who's driving"
   a diegetic, craftable, presumably stealable thing.
2. **Survival loadout cap: two hands + back** (encumbrance doctrine;
   creative bypasses). Containers extend it diegetically: a backpack in the
   `back` socket extends inventory; belts/pockets and similar arrive as
   clothing that *adds container capacity* — clothing defs can contribute
   carry slots, not just cuboids.
3. **NPC-tier degradation lives in the controller layer** (a fallback
   controller binding), never in the body: the body stays pure physics +
   anim slots, so transmog never carries behavior and any controller can
   drive any plan. **Disconnect v0 (decided with it): freeze** — a
   session's disconnect zeroes move intent and the body stands where it was
   left (implementation queued in 3c). The degradation ladder (MCP session
   → local NPC controller → null/freeze) is just re-binding down the same
   rail; richer NPC-tier controllers arrive with the NPC-intelligence
   design.
