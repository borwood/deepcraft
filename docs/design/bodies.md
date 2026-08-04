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
> you claimed). **✅ BUILT, B0 same day (journal/0135):** `KNOWN_VERBS`/`REQUIRED_VERBS` are
> deleted, `slots` are `actions`, segments carry declared **roles** (open, anchored) and plans
> carry **modes** (bearing roles) — a tree and a bird now define. The define-time-checking
> *principle* stands, as *you-supplied-what-you-claimed*. **(2)** name-keyed binding — clips
> to joint names here, forking's clip inheritance by retained names, § Clothing's segment-name
> binding, § Sockets' segment mounts — is a **stand-in**: `posture-gait.md` § 7b ratifies that
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

> **▶ RE-SCOPED 2026-08-02 — THE PINNED HIP THE TABLES BELOW MEASURE IS GONE.** The
> posture bake's **consumer slice** landed: `build_plan_assets` reads the derived hip from
> `dc_api::bodies::bake_resting_posture` and the renderer stands every body at **chain
> reach** (biped 0.880 m, stout 0.440 m, longleg 1.020 m), applying the same delta to the
> rendered root and the IK hip; a plan the bake declines keeps the authored pivot (identity
> fallback). The measured tables below are **dated testimony about the pinned-hip world**
> (still true of the fallback path): under the derived hip, standing soles sit ON the
> annulus boundary and plant **by geometry, not by IK** (idle rest frames seat exactly;
> knees straight to ~0.2°), and **all three plans' flat-crouch cases now correct** —
> including longleg's, which the window used to refuse. New measured series + the
> derivations: dc-client `body.rs` `foot_placement_and_retargeting_are_measured` /
> `derived_hip_reaches_the_render_path`. One new mechanism the slice exposed: the 0.45 m
> crouch drop sinks the stout's **derived** 0.440 m hip to/below the ground, engaging the
> IK's **inner** annulus clamp (`|l1−l2|`, ~10 mm residual) — the fourth-constant hazard
> below, sharpened. The hover's **temporal** half (corrections #80, `hip_y` without
> `root_bob_m`) is untouched and rides.
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

> **▶▶ THE 12 FPS CALL IS TAKEN — DECIDED 2026-08-04 (user). `ANIM_FPS` AS A FIXED CONSTANT IS
> RETIRED; QUANTIZATION BECOMES PER-CYCLE, PER-BONE, AND CLIENT-OWNED.** The deferred taste call
> below became takeable when the feet reached the ground (journal/0148), and the walk supplied a
> reason that is not taste: **a fixed rate aliases a derived cadence.** Mechanism (user-originated
> unless marked):
>
> **1. `N = round(cycle_duration × target_fps)`, clamped to `N ≥ 2`.** *"A cycle gets N poses per
> cycle where N is a quantization of allotted time."* The **floor is the user's** (*"there can't be
> fewer than 2 frames per cycle"*) and it converts the Nyquist edge into a defined outcome instead
> of a degenerate one — `round()` alone returns **1** for a cycle near `1/fps` and **0** below
> `1/(2·fps)`.
>
> **The fix is that N is an INTEGER, not that it is larger.** At a 12 fps target the three shipped
> bodies barely move: biped **6.977 → 7**, longleg **7.742 → 8**, stout **4.286 → 4**. *The hitch
> the user saw at the 0148 walk was the `.29`, not the 4* — a fractional N drifts through the cycle
> and beats against it; an integer N lands on the same phases every cycle. It also explains why the
> biped always read stable: it was already within 0.03 of an integer.
>
> **2. Framerate is CLIENT-SIDE. The engine owns the target only** (user) — § *THE SIM OWNS THE
> TARGET* applied to time. **Target fps is a client performance/visual setting and is NOT pack
> content.** *(This corrects an assistant proposal made and withdrawn the same day, which had put
> the rate in the pack.)*
>
> **3. The pack MAY declare a per-body MAX target fps** — *"preserves toy-like or mechanical
> aesthetics for a robot in a smooth world."* A **cap**, not a value, so the client's setting still
> governs downward and the two compose rather than fight. Clean under `dependency-graph.md` § 0b:
> two well-made packs would answer *"is this body mechanical"* differently, so it is an **opinion**.
>
> **4. Per-bone dynamic fps, by proportion of the anims owning the bone** (user) — *"a weighted
> blend is a weighted blend, a single owner is a single owner."* This dissolves the
> two-blended-cycles problem without a precedence rule: it is interpolation, not arbitration.
> *[assistant, implementation note]* blend the **time step** `Δt = duration / N`, weighted by
> ownership — frames-per-cycle across different cycle durations is not a commensurable quantity.
> *[assistant, observation]* because every `N` derives from the **same** target fps, every `Δt` is
> `≈ 1/fps` and differs only by rounding, so per-bone rates cluster tightly around the target no
> matter how many layers exist. The body is not one exact snapshot and **the user is explicitly not
> worried about that reading as buggy**; this is why.
>
> **5. A ONE-OFF IS JUST A CYCLE THAT DOES NOT REPEAT** (user). If its length is known, identical
> formula. *This collapses a case split the design conversation had been carrying* — there is no
> cycle-vs-one-off rule, only "do we know the duration."
>
> **6. ⚠ ACCEPTED COST, AND IT FORECLOSES WORK — record before anyone re-opens it.** A player who
> lowers their target fps **can lose detail**: a fast kick may not show its full swing. That raises
> *which* frames matter most, and the user ruled there is **no engine answer worth building**:
> *"a hand-authored anim for low fps actions will beat an engine baked locomotion anim in those
> cases, which maybe just is what it is."* Importance-weighted frame selection is **not** owed and
> is not a gap.
>
> **REJECTED, recorded so it is not re-proposed** *(assistant proposal, killed by the user)*: a
> single shared time grid at `fps = N × f_fastest_live_cycle`. It buys one exact whole-body
> snapshot and needs no per-bone rule — and it is **fatally globally coupled**: *"add one fast
> cycle and suddenly the world is smoother."* A local addition with a world-wide effect, not
> fixable by tuning.
>
> **OPEN for the slice, not decided here:** quantization must never feed back into anything
> **sim-visible** — if B5's damage resolution ever reads a pose it reads the sim's *unquantized*
> target, or two clients at different fps resolve hits differently. Implied by the partition;
> owed an explicit guard.

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
> **⚠ THE PRECONDITION ARRIVED 2026-08-03 AND THE CALL IS NOW TAKEABLE (journal/0148).** The feet
> reach the ground: the derived gait shipped, `root_bob_m` left the schema, and the user's walk
> verdict was *"the bob is gone… soles are planted."* **And the walk produced a reason to take
> the call that is not about taste at all: a FIXED 12 fps ALIASES A DERIVED CADENCE.** Frames per
> cycle is no longer a constant anyone chose — at 4.5 m/s the biped gets 6.97 (near-integer, reads
> stable), the longleg 7.73, and the **stout 4.29**, barely above Nyquist, so its samples drift
> through the cycle and beat against it. The user saw the hitch and diagnosed it unprompted.
> **A-1's fourth instance in this arc**: 12 fps was chosen when there was one clip at one cadence.
> **User leaning is FORFEIT — recorded as a LEANING, not a ruling** (*"probably we just forfeit
> it… i just don't care enough"*). **Their diagnosis survives either way and is the thing to keep:
> the fault is quantizing per SECOND against a cadence that varies per BODY — quantize per CYCLE
> (N poses per stride) and every body gets the identical stop-motion look regardless of leg
> turnover.** That is the shape that would have let the aesthetic survive derived gait. Board:
> ROADMAP § Observed; adjacent to B3's 20 Hz vs 12 fps question (`posture-gait.md` § 7 member 2).
>
> **⚠ CORRECTED 2026-08-01 — corrections #80. THE BANNER BELOW IS TRUE AS ARITHMETIC AND
> WRONG AS A DIAGNOSIS, and it is preserved unedited as dated testimony.** The quantizer is
> the **second** wall, not the operative one. The hover's actual cause is a space-layering
> defect: `character.rs:219` builds the IK's hip as `feet.y + hip_local[1] - crouch_drop`
> — **excluding `root_bob_m`** — while `character.rs:257` adds `root_bob_m` to the rendered
> root. The solver is handed a hip that never moves, returns a correct and **constant** leg
> pose, and the renderer then translates the whole body, feet included, by the bob. Fix that
> and the quantizer becomes the next wall (a ~1° correction against an 11.25° step, and
> `stepped_angle()` is applied to the IK output). **⚠ "Fix that" is NOT a work order
> (corrections #93, 2026-08-02): the authored clip bob itself has no contemporary
> ratification (user: "i don't even know that it makes sense as something we want for all
> bodies, or uniformly for all bodies"), and #81's resolution below already deletes the bob
> as an authored constant. The finding here is the space-layering FACT; the bob's disposal —
> per-species output, or absent — is the gait-bake design pass's.** **The discriminating evidence was already
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

Binding is by **segment name** *(the binding KEY rides the `stubs.md` § 34 role-binding
migration — see the § Body plans banner; the mechanism here is untouched)*, and authored
cuboids are expressed **relative to segment dimensions** (mechanism note: this is what keeps "one
chestplate def adorns every mob of every plan with those segments,
mutations included" true — absolute-meter authoring would break on scaled
mutations). Pinned/swinging pieces are cosmetic-side per the determinism
firewall. v1 boundary softens accordingly: pinned-rigid pieces (capes) are
in; soft-body is later.

## Postures — DECIDED 2026-08-02 (user; sweep-checked against the 2026-07-19 corpus first)

**Postures are pack-declared REGISTRY content, referenced by namespaced id** — the
materials/geology-classes pattern, NOT the per-plan open strings that actions and modes use.
The asymmetry is principled: actions and modes are plan-local declarations (clip bindings,
bearing geometry — meaningless across bodies), while posture is the one vocabulary the sim
(collider policy, replay), the input layer (the crouch key on ANY body), and the bake key all
key on **across every body in the world**. A shared coordinate needs registry identity;
string coincidence is the poor-man's-role failure this arc already paid for (stubs § 34).

- **The engine owns the posture CONTRACT, never any posture's name**: a declared posture must
  be bakeable for every plan that binds it, and carries its sim-visible disposition —
  collider/clearance policy plus **transition semantics** (entry/exit: explicit verb or a
  geometry-triggered rule; the engine enforces guards — `PostureBlocked` is the shipped
  first instance, replay-tested).
- **The default pack ships the basic set, and it is the 2026-07-19 posture ladder**:
  `dc:posture/{stand, crouch, crawl, prone}` — crouch with its DECIDED semantics (0.6×,
  sneak edge-walk, never passage); crawl and prone are declared-unscheduled until their
  slices, with the ladder's crawl-as-geometry-triggered-transition mechanism (deliberate
  entry only) as the declared entry rule when built.
- **Cross-pack inheritance is a declared reference**: a mod's plan *binds* `dc:` posture ids
  and the engine checks the reference at define time — never an engine default, never a
  string coincidence. A body that binds no `crouch` cannot crouch; the key no-ops loudly
  (you-supplied-what-you-claimed at the input layer). Evolution composes by construction:
  postures are data through the define door, and the define door is callable from the
  deeptime clock (corrections #86) — an evo pack minting a species can mint or bind
  postures right there.
- **The controller surface follows**: capability **introspection** (list a body's declared
  postures/actions/modes) + **invoke by id**, controller-agnostic — the same two verbs
  serve the player, the MCP animus (a golem's Claude driver discovers what an arbitrary
  body can do), and future script/NPC controllers. Possession/assume-form UX is UI over
  the same surface (lowest prio, user ruling).
- **Migration is NOT free and is flagged as such**: `dc:character/set_posture` ships a
  closed wire enum and posture is replay-bit-identity-tested
  (`posture_transitions_replay_identically`), so enum→id rides the character surface's
  appended-field discipline, sequenced with the sim-visible slices — not a B0-style
  free-window recompile.

### Locomotion lean is NOT a posture — DECIDED 2026-08-03 (user; the fork gamed out, then agreed)

**A posture is a collider-and-support disposition: declared, discrete, registry content. Lean is
a continuous derived term applied to whichever posture is active.** There is no
`dc:posture/run`.

**Why the lean exists at all** (and it is sharper than *"because running"*): a body accelerates
forward only if the ground pushes it forward, and the ground reaction force runs from the contact
through the centre of mass — so the CoM must sit **ahead of** the contact. It is a torque balance,
`tan θ ≈ a / g`. **Therefore the dramatic lean is an ACCELERATION phenomenon, not a running
one:** a sprinter leaving the blocks is at ~45°, a distance runner at steady pace is nearly
upright, leaning only against drag. Modelling it on acceleration gets both for free with no
sprint-start special case. **It is the resting-posture solve with one horizontal force term
added** — the same primitive, one term richer, and the arc's own test applied again (*a quantity
with a physical determinant is an output*).

**The four cases that settled the fork** (declared posture vs continuous modifier):
1. **The crouch-run.** A declared posture forces `crouch_run`, then `crawl_run`, then
   prone-anything: posture × locomotion is a **product**, and products in a registry are how
   vocabularies die. As a modifier, crouch keeps owning the collider and lean applies on top.
2. **The accelerating walker.** A walker accelerating hard leans, *below any run threshold* — so a
   declared `run` posture does not remove the continuous term, it stacks a discrete layer on one
   you still need. The threshold was never real: lean tracks acceleration, and acceleration does
   not respect gait boundaries.
3. **The evolved quadruped.** Nobody declares a run posture for a species deeptime minted last
   epoch. As a modifier it falls out of the solve like its stance and gait; as a declared posture
   an undeclared species **cannot run** — the exact failure the bake exists to prevent.
4. **The alligator's belly slide** — the case expected to rescue the declared option, and it does
   not: it is already **a MODE** (a different active support set, § 5), as are swimming and
   climbing.

**The synthesis, and it is why the fork was less balanced than it first looked:** everything that
*feels* like a locomotion posture is already covered by a ruled axis — **fundamentally different
support is a MODE**, **limb timing is a GAIT**, **body-lowering is the CROUCH posture we already
have**. What remains after those three is exactly one thing: a continuous lean tracking
acceleration. **No fourth axis, no product explosion**, and the continuous-Froude-ladder ruling
extends rather than competing with the postures ruling.

- **It is gravity-dependent** (`atan(a/g)`), so a low-gravity world's creatures lean **more** for
  the same acceleration — an emergent consequence the 2026-08-02 gravity ruling made expressible,
  and checkable against published sprint-start and steady-state trunk angles.
- **Sim owns the TARGET, client owns the APPROACH** (§ above). The lean target is a *deterministic
  function of sim state* (acceleration and gravity), so it is sim-ownable without breaking replay —
  it is **not** the "smooth per-frame" the firewall forbids, because it is derived rather than
  ambient. The smoothing toward it is cosmetic.
- **ENGINE owns the solve** — only the solver knows its own statics, the field-kernel argument
  again. **PACK owns the posture vocabulary, the body, and the stylistic DISTRIBUTION**, which is
  genuinely underdetermined: physics fixes the CoM offset and says nothing about whether it comes
  from ankle, hip, trunk or neck. A human leans from the ankles; a chicken pitches its trunk and
  counter-rotates its neck. That is knobs-taxonomy **kind 1** (underdetermined style), per § 7
  member 1's ratified docket.
- **Selection rides the existing ladder**: the same dimensionless speed that selects gait informs
  lean, continuously. **Never a discrete switch** — user call #1's explicit prohibition.

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

## Who owns the look — DECIDED 2026-08-02 (user; the walk-8 strafe's design half)

**Move intent defaults to look-follows-travel; an explicit look is HELD until released.**
The engine derives an unheld gaze from the direction of travel (yaw = travel heading, pitch
level, computed in `step_character` so senses and renderer read the same gaze); when
stationary it keeps its last heading, like the trunk. `dc:character/set_look` is the
explicit override and **holds** until `dc:character/clear_look` releases it — deliberate
gaze (a predator tracking prey while circling) is a driver's stated intent, never a stale
default. No hold timer: decay would be a tuning constant nobody ratified. The pose readback
reports `look_held`, so a driver knows which regime the gaze it reads is in.

Why this side: the alternative (every driver owns the look) is the arrangement that
actually failed — S6 row 95 carried *"nobody owns body orientation"* OPEN from 07-19 until
the user re-sighted it at the 0140 walk as the strafe, because a driver obligation is
invisible and its failure is silent. Same shape as the postures ruling: the engine owns the
contract and the correct default; drivers invoke deviations explicitly by verb.

**Compartmentalization caveat (user, at ratification):** *"in the future we may revisit the
issue of animals with multi-segment necks. as long as the look-at can be replaced with a
different function in the future / is compartmentalized enough, I don't think this is an
issue."* The seam is `resolve_orientation` (dc-client `body.rs`) — a pure
`(trunk_yaw, look_yaw, look_pitch) → Orientation` function, the one place the look becomes
joint angles; a multi-segment neck replaces that function's body (distributing the clamp
along the cervical chain) without touching who owns the look.

## THE SIM OWNS THE TARGET; THE CLIENT OWNS THE APPROACH — DECIDED 2026-08-02 (user: *"Good pattern to stick with in bodies"*)

**The arc's standing rule for any quantity that sits near the determinism firewall.** Split it
into the **target** — the value the body is heading for, which is derived from sim state, is
deterministic and replay-safe — and the **approach**, the cosmetic path taken toward it
(smoothing rates, easing, expression within a clamp), which stays client-side and genuinely
free to tune. **Neither half is "the" quantity, and the word for the whole thing is the trap.**

**Three instances, which is this project's own threshold for a shape being real**
(`posture-gait.md` § 3: *"Two instances of one shape; if a third appears it is a primitive"*):

| quantity | target (SIM) | approach (CLIENT) |
|---|---|---|
| **gaze / look** | `CharacterState.yaw/pitch` — aims perception (`sense_raycast`), rides the command log, follows travel when unheld | the **neck bend**: `resolve_orientation`'s cervical clamp and the trunk drag beyond it |
| **trunk facing** | the target facing derived from travel or a held look | the **turn rate** toward it (`TRUNK_TURN_WINDOW_S`) |
| **pose** | the **nominal pose** the sim reconstructs from `(clip, phase)` — what damage resolves against (`posture-gait.md` § 5) | foot IK onto actual terrain, expressive layers |

**Why it keeps paying.** Every defect this rule would have prevented had the same signature: a
quantity that *reads* cosmetic while sim consumers were already sequenced against it.
- The **gaze** was documented as client-owned and cosmetic while it aimed every creature's
  perception (corrections #94) — the doc asserted the opposite of what the arc shipped the same
  week.
- **Trunk facing** is client-side today only because our colliders are axis-aligned boxes that
  do not rotate. **B4 bakes collider sets per yaw bucket and B5 resolves damage against the
  nominal pose** — so facing decides *which collider* and *whether you were hit in the back*.
  It is fairness-critical and replay-critical the moment either lands, and it is **exactly
  where the walk-8 strafe lived** (journal/0140): nobody owned it, so it inverted on the path
  nobody built for.
- Doing it late is the expensive version: before a consumer exists this is a recompile; after
  B3 makes the pose a versioned sim asset it is a wire migration.

**The failure mode it forbids by name:** *two derivations of one quantity.* Letting B4 and B5
each derive facing for themselves is the two-authority condition that produced `root_bob_m`
(corrections #80, #93) and the gaze confusion (#94) — three instances in one day, which is why
this is a rule and not an observation.

**⚠ Scoped to BODIES by the ruling.** Whether this is a corpus-wide **spine** (it has its three
instances, and `spines.md` § 4 wants deviations and additions ratified deliberately) is a
separate, unasked question — filed, not assumed.

## Joint rotation limits — DECIDED 2026-08-02 (user; the primitive B0 stopped short of)

**A joint carries a rotation range, and exceeding it is a VALIDATION FAILURE, never a runtime
clamp.** Opened by the user while ruling the gait pass's keyframe count — *"frankly,
hyper-extension just shouldn't be possible if we set rotation limits on joints"* — and the
sweep that followed found the hole is total: `SegmentDef` carries `name, parent, pivot_m,
size_m, offset_m, tint, roles` and **no rotation range**; `solve_leg_ik`'s clamps
(`dc-client/src/body.rs:485,493`) keep `acos` in domain and the target inside reach, which are
**numerical guards, not anatomical ones**. No plan, no solver, no validator prevents a knee
inverting today.

**It is a PRIMITIVE, not a gait detail.** Its consumers are the resting-posture bake, the gait
bake, the IK solver, clip validation, plan forking, and every body an evolution pack mints. It
is upstream of `posture-gait.md` § 7 member 1, which is where it surfaced.

- **Enforcement is at the free clock, never the sacred one** (the two-clocks doctrine, and the
  reason this is option A rather than a clamp). The **bake** checks its own output against the
  limits and **refuses loudly** if a derived gait would exceed them. The **validator** rejects
  a clip that keys a joint past its range **at define time** — so an authored clip that would
  break a creature is caught when it is authored, not watched. The **IK solver** treats the
  limit as part of the reachable set and solves *within* it, rather than solving freely and
  correcting afterwards. **The runtime never silently clamps.**
- **Why not a clamp** (it was the cheaper option and it is the wrong one): a clamp *hides* the
  error. The gait asks for a pose, the joint quietly refuses, and the foot lands somewhere the
  gait never intended with **no signal anywhere** — working code, passing tests, silently
  wrong output, which is the failure mode this corpus has paid for repeatedly (`ARCHITECTURE.md`
  § *A summary is not an authority*; anti-shape **A-3**).
- **The existing honest-failure precedent EXTENDS, and is not replaced:** when the foot-IK
  terrain correction exceeds what the leg can do, the foot **floats honestly** rather than
  lying about contact (`character.rs`, the half-voxel window). Limits are the same discipline
  one level up.
- **Declared per joint, with a DERIVED DEFAULT** (user-ruled). Declared is simple and honest
  for a hand-authored body; a **derived default is what makes the primitive survive evolution**
  — nobody is present to author ranges for a species the deeptime sim invented, so a mutated
  body must get plausible limits by construction, exactly as it gets a posture and a gait.
  Declared overrides derived; the derivation itself (from segment geometry, and what "plausible"
  means for a joint nobody has seen) is **owed a design pass** and is not settled here.

### Fold sense — DECIDED 2026-08-02 (user): a SETTING on the joint, and unspecified means NO PREFERENCE

**Geometry cannot supply it and we stop trying.** The derivation gives a joint's **magnitude**
(how far it folds before the child chain enters an ancestor's box — 155° for the biped knee,
in the published band); it cannot give the **sign**, because **our bodies have no front**.
Every shipped plan is mirror-symmetric in Z as well as X, so a knee and an elbow are *the same
object* to the derivation. Real anatomy carries the arrow in its asymmetry — a knee folds away
from travel *because the foot extends forward of the ankle* — and our feet are centred boxes
with no toe and no heel.

- **The bend is DECLARED, at any level of specificity, by any author** (user): *"hand authored,
  mcp authored, evolution authored can all config the bend of a knee at different levels of
  specificity."* One vocabulary, three minting clocks — the same shape as postures and the bake.
- **⚠ AND THE FRAMING THAT DISSOLVES THE WHOLE PROBLEM (user, 2026-08-02): *"it's not actually
  a knee until constraint is declared."*** An undeclared joint that hinges both ways is **not a
  hyperextending knee — it is a joint**. *Hyperextension is a concept that exists only relative
  to a declaration*, so the geometric derivation is not failing to prevent anything; there is
  nothing yet to prevent. **This retires B7's J1 as a tension** (which asked how the derived
  default could deliver *"plausible limits by construction"* when it cannot supply a sign) and
  it retires the assistant's framing of a backwards-bending joint as a defect the engine owes a
  guard against. The engine's contract is exactly: **declared limits, plus whatever geometry can
  supply where a declaration is absent or incomplete — magnitude only, both directions, and that
  is a complete and honest answer.**
- **ENGINE / PACK PARTITION, ruled in the same breath and correcting an assistant error:**
  phylogenetic **inheritance of constraints is PACK behaviour, not engine machinery.** Evolution
  mints a generation by taking the previous generation's constraint and *"either copy it forward
  or mutate it, generally speaking"* (user) — an evolution-pack concern, sitting on the engine's
  declaration vocabulary like any other content. **The assistant had proposed inheritance as the
  engine's answer to J1**, which put a pack mechanism in the engine's lap and is the partition
  error `dependency-graph.md` § 0 exists to prevent. A novel limb with nothing to inherit from
  simply **has no constraint** — legal, honest, and not a gap needing a loud refusal.
- **Unspecified is not "unlimited" and not "assume a hinge": the joint HONESTLY HAS NO
  PREFERENCE** (user's words). It keeps the derived magnitude bound in **both** directions and
  declares no sign — the honest-absence pattern this corpus uses everywhere (`Undetermined{reason}`,
  the IK's honest float, `has_contents: false` meaning *no record*, never *air*). A consequence
  to state plainly: **derived-only limits catch a backwards knee ZERO times**, and that is
  correct — an outer bound with no sign cannot reject one. Declaring is what makes the walk
  clip's 28.65° hyperextension catchable.
- **Express it in SEGMENT-LOCAL axes plus a sign, never relative to a body forward.** Strictly
  more general and it is what homes the ask: *"the fore and aft limbs of horses and lizards and
  dolphins and humans."* A lizard's sprawled knee folds about its own local axis whatever the
  limb's orientation to the trunk; a body-forward-relative encoding would need a reference frame
  that sprawl and flippers immediately break.
- **Feet make the sign derivable LATER, which demotes the declaration to a default.** Once a
  foot has a toe and a heel the arrow is in the geometry. **Add feet to the testing models**
  (user); for the deep-sim pack, **feet exist if they evolved** — the engine only ever needs
  the *support declaration* (`sole`, already B0's role vocabulary), never a named body part.

**⚠ AND A GAP THIS OPENS, verified at source 2026-08-02: BODY-LOCAL FORWARD IS AN UNSTATED
CONVENTION.** `BodyPlan` declares `name, doc, segments, modes, actions` — **no forward axis**.
World-space forward exists (move intent, and the trunk chasing it), but the plan's local **−Z
is forward purely by inherited camera convention**, undeclared and uncontradictable. User:
*"clearly we should just know what way a body's forward is."* **A-1 in its usual costume** —
one body orientation, so a convention became a definition. Its consumers are not hypothetical:
**B5** resolves whether a hit landed on the front or the back, asymmetric anatomy needs a side
to be asymmetric *about*, and the fold-sense derivation above needs it the moment feet gain
toes. Filed as its own item; not folded into B7 silently.

**Sequencing:** upstream of the gait member's build, and it wants the segment-identity window
while it is still cheap — a range on `SegmentDef` is a recompile today and a **wire migration**
once B3 makes the pose a versioned sim asset, the same deadline `stubs.md` #34 rides.

## Individual proportion variation — DECIDED 2026-08-03 (user): SIZE FIRST, declared axes deferred

**Every biped in the world is dimensionally identical today** — `SegmentDef` carries exact
`pivot_m` / `size_m` / `offset_m`, so "this wolf" and "that wolf" differ in nothing but position.
The wanted end state is a plan that declares **ranges**, and an individual that is a **salted
sample**: this one's ears slightly smaller, that one's muzzle longer. Structurally this is **S-9**
(derivable base + sparse committed facts), fourth instance — species is the base, an individual
is a seed plus a few numbers.

**Partition, ruled** (user): **RANGE is ENGINE** vocabulary — what may vary and within what
bounds — and **DISTRIBUTION is PACK**, *"pack is what spawns things in the world, this only makes
sense."* **Sim-visible variation is fine, and the PACK owns fairness** — the engine does not owe a
guarantee that a smaller individual is hit fairly; that is the pack's problem, which unblocks this
from waiting behind **B4**.

**The mechanism family is ALLOMETRY** (user): *"infant-adult growth etc."* Juvenile→adult and
inter-individual proportion are **the same mechanism**, which is what makes the unit of variation
an **axis** rather than a parameter.

**Four thought experiments settled the shape, and each one breaks "put a range on any param":**
1. **The disconnected limb.** Independent ranges on `pivot_m` and `size_m` let a thigh shrink while
   its knee pivot stays put — the shin floats off the femur or interpenetrates it. Nothing declares
   that a child's pivot lives on its parent's surface, because with fixed numbers it always did by
   construction. **Proportions are RELATIONSHIPS, not numbers.**
2. **The wolf pup.** Jitter every param independently by ±15 % and you get a slightly different
   *adult*, never a juvenile — a pup's head is proportionally bigger and its legs proportionally
   shorter, and those move **together, in a specific direction**. Independent variation cannot
   express growth **at any range width**, because growth is a correlated trajectory, not a cloud.
3. **The evolution substrate.** Independent params give the evo pack a search space that is mostly
   nonsense (a leg 3× longer with an unchanged foot and pelvis); declared axes give a space that is
   nearly all viable but only holds variations someone anticipated. **These are two mechanisms at
   two timescales:** declared axes for cheap always-viable *within-species* variation; **evolution
   mutates the PLAN itself, including its axes**, over deep time. Individual jitter and speciation
   must not share a mechanism.
4. **The cost gradient — and it chose the first slice.** **Uniform scale is FREE**: the resting bake
   returns **scale-free outputs, angles plus a height ratio** (`bake.rs:81`, decision 24's *bake
   ratios and angles, not metres*), so a 1.05× wolf has **identical joint angles** and its hip
   height falls out of one multiply — no re-bake, no correction. An allometric axis (longer legs,
   same trunk) changes angles and needs a correction or a re-bake. Arbitrary per-param jitter needs
   a **full re-bake per individual**, which the per-species ruling explicitly refuses (*"it's not
   baking for every individual wolf in existence"*).

**SEQUENCED BY COST, ruled 2026-08-03: SIZE FIRST; declared allometric axes are LATER** (user:
*"fair — size first, later think about declared axes"*). Size is free today and immediately yields
big and small individuals. **The question this arc must not re-derive:** it is *not* "which
parameters may vary" — it is **"what axes do they co-vary along"**, and only the size axis is
ruled so far.

**⚠ Texture-side variation — coat, markings, eye shape — is a DIFFERENT PIPELINE** (user: *"mostly
new and a whole other thing"*), sharing only the salt. Do not fold it in here.

## Sockets — PROPOSED

A socket is a named, typed mount on a plan segment: `hand.r`, `back`, `head`. *(A socket name
is role-shaped — "the right hand" across a humanoid and a centaur is one socket. When built,
it binds through the `stubs.md` § 34 role migration, not by raw segment name.)*

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
