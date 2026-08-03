# Joint rotation limits (B7) — a design pass against the ruling

> **⚠ BUILT 2026-08-03 — SLICE ONE. The body below is testimony; this banner is what the build
> MEASURED against its predictions.** Every number in § 3.2, § 3.3 and § 5.3 was hand-derived
> with *"no cargo command run"* and labelled *"a prediction the build checks."* It has now been
> checked. Code: `crates/dc-api/src/bodies/limits.rs`; journal: `pending-b7-joint-limits`.
>
> **1. THE LAW AS WRITTEN IS AMBIGUOUS, AND § 3.2's ROWS ARE NOT SELF-CONSISTENT.** § 3.1 says
> *"the eight corners of the child subtree's TERMINAL box."* No reading of that sentence
> reproduces the table:
> - *"eight corners"* is **degenerate** — the terminal box's *proximal* corners are the very L1
>   failure the law exists to escape. Test them and the biped knee returns **0°**, not 155°.
>   Every row is reproducible only with the **distal** corners.
> - *"terminal box"* contradicts the table's own **hip** rows, which are derived from the
>   **thigh's** distal corner — the rotating segment's own box. The shank, the actual terminal
>   box, never re-enters the trunk at all; drop the rotating segment's own box and both hip rows
>   become `Undetermined`, taking § 3.2's headline finding (stout 77° vs biped 150°) and § 5.2's
>   number for gait member #1 with them.
> - The **neck** rows go the other way: derived from the **head's** distal corner with the neck's
>   own box omitted. Include it — as the hip rows do — and it binds first.
>
> Built as the one self-consistent reading: **the distal corners of EVERY box in the rotating
> subtree, plus declared role anchors, against every strict ancestor.**
>
> **2. THE MAGNITUDES ARE CONFIRMED. The neck rows and `head` are not.** Measured (derived `max`
> end, radians; `derived_limits_match_the_predicted_table` pins these):
>
> | row | § 3.2 predicted | **measured** | verdict |
> |---|---|---|---|
> | biped knee X | 2.70541 (155.009°) | **2.7056301 (155.021°)** | ✅ Δ 0.012° |
> | stout knee X | 2.00676 (114.979°) | **2.0067251 (114.977°)** | ✅ Δ 0.002° |
> | longleg knee X | 2.76546 (158.449°) | **2.7653693 (158.444°)** | ✅ Δ 0.005° |
> | biped hip X | 2.61431 (149.789°) | **2.6143472 (149.791°)** | ✅ Δ 0.002° |
> | stout hip X | 1.35304 (77.523°) | **1.3522680 (77.479°)** | ✅ Δ 0.044° |
> | longleg hip X | UNDETERMINED | **UNDETERMINED** | ✅ shaft-contact miss, as predicted |
> | biped/longleg elbow X | 2.71787 (155.722°) | **2.7178262 (155.720°)** | ✅ |
> | stout elbow X | 2.81076 (161.045°) | **2.8106925 (161.041°)** | ✅ |
> | shoulder X (all three) | UNDETERMINED | **UNDETERMINED** | ✅ |
> | **biped neck X (pitch)** | 2.46825 (141.420°) | **1.0427219 (59.744°)** | ❌ **the neck's own box binds first** |
> | **stout neck X (pitch)** | 2.28291 (130.801°) | **0.6287963 (36.027°)** | ❌ same cause |
> | biped neck Y (yaw) | UNDETERMINED | **UNDETERMINED** | ✅ |
> | **biped neck Z (roll)** | 2.17408 (124.566°) | **1.0427219 (59.744°)** | ❌ same cause |
> | biped knee Y (twist) | UNDETERMINED | **UNDETERMINED** | ✅ |
> | biped knee Z (lateral) | 2.75046 (157.590°) | **2.7503973 (157.586°)** | ✅ |
> | **biped `head` X** | UNDETERMINED (*"no child chain to collide"*) | **2.2142974 (126.870°)** | ❌ a segment's own box IS part of its rotating subtree |
> | biped `trunk` | UNDETERMINED | **WELDED** (zero DOFs) | ✅ stronger, per Q4 |
>
> The audit's own hand-arithmetic drifts by ~1e-4 rad throughout; the *stout hip* row is the
> worst at 7.7e-4 (0.044°). No row's conclusion changes on that.
>
> **3. § 3.3's LITERATURE ROWS MOVE WITH THE NECK.** Derived cervical flexion/extension is
> **59.74°**, not 141.42° — which lands **inside** the published 45–70° extension band rather
> than 3× over it, and cervical lateral at 59.74° vs 45° is ~1.3× over rather than 2.8×. Every
> other row stands, and § 3.3's *conclusion* — the derivation is good where the end-range is
> **bony** and useless where it is **ligamentous** — is unaffected and better supported.
> **The published figures themselves were NOT re-verified: this build had no network either, and
> they are carried forward with the pass's own disclaimer intact.**
>
> **4. § 5.3's `d_min` TABLE CONFIRMED, and the degenerate stout crouch is gone.** Measured
> biped **0.19131** (predicted 0.19143), stout **0.23664** (0.23660), longleg **0.19175**
> (0.19199) — 9.6× / 23.7× / 9.6× the numerical `|l1−l2|` they replace, matching the predicted
> ratios exactly. Each is the fully-folded foot distance to 1e-9, asserted as geometry rather
> than pinned as a magnitude. The stout's crouch target (`d = 0.010`, a sole *below the ground*)
> is now `Reach::BeyondFlexion` and the foot floats honestly; journal/0131's 180° knee cannot be
> reached. Across a 3198-target sweep of all three plans the solver is **bit-identical** to its
> pre-B7 self on the 2184 targets outside `d_min`; the 1014 inside are refused rather than
> clamped, which is the whole of the behaviour change.
>
> **5. A CASE THE PASS NAMED BUT DID NOT COST: `dc:body/stout`'s FLUSH SHOULDERS.** § 3.2 called
> it *"a zero-measure graze at x = 0.39 exactly."* In f64 the two shoulders land on **opposite
> sides** of that boundary, and read as an entry it derived a **0° bound**, welded both arms and
> **rejected the shipped `dc:anim/biped_idle` clip on the shipped pack.** A point already
> intersecting an ancestor at rest is now reported as a graze in the authored geometry, never as
> a limit.
>
> **6. THE POLE IS NOT FULLY DATA YET, and § 5.3(1) over-promised.** *"The `ka.1 <= kb.1`
> comparison is deleted, not generalised"* holds only where a range is one-sided. Every shipped
> plan's derived range is **symmetric** (§ 3.4's own finding), so **both** poles are admissible
> and something must break the tie — and if that something is not the pre-B7 forward convention,
> the identity default is not byte-identical and § 8's test 1 fails. Built as: the pole is the
> candidate the declared range admits; a tie falls back to the forward convention, marked
> `⚠ STAND-IN` with `stubs.md` B7-a as its heir.
>
> **7. § 5.4's MIGRATION IS BYTE-IDENTICAL — AND IT IMMEDIATELY REPORTS ITS OWN DEFECT.**
> `NECK_YAW_CLAMP_RAD` (75°) and `NECK_PITCH_CLAMP_RAD` (45°) are now declared on each plan's
> `look` joint. On `dc:body/stout` that declared ±45° pitch is **outside** its derived ±36.03°:
> its neck is an 0.08 m slab on a 0.62 m trunk and it genuinely cannot pitch that far. Reported,
> not refused (so the render is unchanged), and it is exactly the A-1 defect § 5.4 named — *"the
> stout's short thick neck gets the biped's numbers"* — now visible instead of invisible.
>
> **8. § 5.1's REJECTION TABLE CONFIRMED** on a fixture plan: a declared one-sided biped knee
> rejects `dc:anim/biped_walk` at keyframe 0 naming `leg_r_lower` at **+0.500000 rad (+28.648°)**,
> and `dc:anim/biped_jump` at its **+0.100000 rad** keyframe. `biped_idle` survives. Nothing is
> declared on the shipped pack's knees, per § 5.1's forced order.

> **⚠ Q4 RULED PROVISIONALLY 2026-08-03 (user): WELD THE ROOT — *"i'm willing to go A for now"*
> — AND THE TWO RESERVATIONS ARE PART OF THE RULING, NOT COMMENTARY.** The root segment has no
> rotational DOF of its own; body orientation belongs to the facing system, and a clip keying the
> root is rejected at define time. Free today: no shipped clip keys it. Same two-authorities-become-
> one move as `root_bob_m`, the gaze and gravity.
>
> **Reservation 1 — option B is what many games actually do, and the assistant dismissed it too
> fast.** User: *"the character is facing one direction, but their idle animation has them look
> around or something that appears to rotate root a little."* That is a real expressive technique,
> not a mistake. **The weld forecloses it**, and if idle root-motion is later wanted, this is the
> rule to revisit — with a composition rule (add? override?) rather than by quietly un-welding.
>
> **Reservation 2 — THE WELD SILENTLY DECIDES SOMETHING NOBODY DISCUSSED.** User: *"it does seem to
> force a preference for physics for death/knock which we haven't discussed."* Correct, and it is
> the sharper of the two. Toppling, knockdown and death rotate the **whole body**; welding the root
> against clips means that motion **must** come from physics or the facing system. **No such ruling
> was ever made — the death/knockdown/ragdoll question is genuinely UNOPENED**, and this weld
> constrains its answer in advance. *An implicit ruling riding inside an explicit one is exactly the
> class this session has been catching all night* (corrections #93's fix-framing, #96's invented
> constraint). **Filed as an open thread, not resolved here; the weld is provisional against it.**
>
> **⚠ Q6 CLOSED 2026-08-03 (user): batch or separate BY CONVENIENCE — the question was
> over-thought.** *"i don't care. if the recompile takes a while, then batch them so we don't wait
> as long."* Measured: nothing persists a `BodyPlan`, so all three pre-B3 changes
> (`SegmentDef.dofs`, `root_bob_m`, stubs #34's binding key) are **recompiles, not migrations**, and
> `root_bob_m` is landing separately anyway in the consumer slice. **Batch for wall-clock, not for
> safety.**

> **⚠ Q1 / J1 RULED 2026-08-02 (user) — AND THE FINDING IS DISSOLVED, NOT ANSWERED. Mutable
> header; the body below is testimony.** J1 asked how the derived default can give an evolved
> body *"plausible limits by construction"* when geometry supplies a magnitude and never a
> sign, so a fore-aft-symmetric evolved body is not protected from hyperextension — the
> ruling's own opening motive. **The user reframed the premise out of existence:** *"a 'knee'
> could bend backwards: it's not actually a knee until constraint is declared."*
> **Hyperextension is a concept that exists only relative to a declaration.** An undeclared
> joint hinging both ways is not a hyperextending knee; it is a joint. The derivation is
> therefore not failing to prevent anything, and the engine owes no guard against it.
>
> **The engine's contract, ruled:** *"on the engine side, limits are declared and we only derive
> what we can from geometry if declaration is absent or incomplete. This does mean that a limb,
> in a vacuum, that does not have a declared constraint, can hinge any way and we only know
> magnitude."* That is **complete and honest**, not a degraded mode.
>
> **⚠ PARTITION CORRECTION, same ruling:** the assistant proposed **phylogenetic inheritance as
> the ENGINE's answer to J1**. Wrong side. *"On the plugin side / evolution: of course evolution
> can declare what side it bends or not at all, and the mechanism for inheritance is to take the
> previous generation's constraint and either copy it forward or mutate it"* — **pack behaviour
> sitting on the engine's declaration vocabulary**, exactly like every other content concern
> (`dependency-graph.md` § 0). A novel limb with nothing to inherit from simply **has no
> constraint**: legal, honest, not a gap owed a loud refusal.
>
> **Consequence for the default pack, unchanged and now correctly located:** the walk clip's
> 28.65° hyperextension becomes catchable **when the default pack declares its biped's knee** —
> an *authoring* act in content, never an engine default.

**Status: DESIGN PASS. Nothing here is ratified and no code was changed.** The deliverable is
this document. It works `bodies.md` § *Joint rotation limits* (**DECIDED 2026-08-02, user**)
against the post-member-#0 code and proposes a concrete shape for the three things the ruling
explicitly left owed: the **representation**, the **derivation**, and the **mechanics** of
declared-overrides-derived. Proposing and building are different acts; the second is the user's
call. **No cargo command was run** (a parallel session holds the build slot); every number below
is hand-derived from source literals and is **a prediction the build checks** — member #0's
table was confirmed to 1e-9 and that is the bar.

**Read with:** `bodies.md` §§ *Joint rotation limits* (the ruling — data, not re-arguable),
*THE SIM OWNS THE TARGET*, *Postures*, *Who owns the look*, *Body plans*, *Clothing*
(**§ IK is CLOSED — corrections #81 — and nothing below re-presents it**) ·
`posture-gait.md` bones §§ 2–6, § 7 member 1's ratified docket, § 7b ·
`docs/audits/2026-08-02-posture-bake-member0-design.md` + header (member #0, **built and
consumed**) · `docs/audits/2026-08-02-gait-bake-member1-design.md` + its five ruling headers ·
`stubs.md` §§ 34, 39–42 · `dependency-graph.md` § 2b (B7's row) ·
corrections **#77 #78 #80 #81 #93 #94** · spines **S-3 S-5**, **A-1 A-3 A-4** ·
`crates/dc-api/src/bodies.rs`, `bodies/bake.rs`, `bodies/default_pack.rs`,
`bodies/experiments.rs`, `crates/dc-client/src/body.rs`, `character.rs`.

**Provenance convention** (the 2026-07-25 rule, as the two preceding passes): mechanisms are
marked **[user-ruled]** (derived from the ratified ruling), **[bones]** (stated in
`posture-gait.md` §§ 2–6), or **[assistant-proposed]** (this pass's own — *a hypothesis until
ratified*).

---

## 0. The proposed shape, in five sentences

A joint's range is declared **on the segment** as an ordered list of degrees of freedom —
`SegmentDef.dofs: Option<Vec<DofDef>>`, each DOF naming a **segment-local axis (X/Y/Z, the only
axes the Euler pose can express)** and an independently-optional **min and max**, so a
declaration can supply exactly one bound and inherit the rest. The **derived default** is a pure
function in dc-api beside the bake — `derive_joint_limits(plan)` — whose law is *the child
chain's distal extent must not enter an ancestor segment's box*: tuning-free, proportion-
sensitive (biped knee **±155.01°**, stout knee **±114.98°**, stout hip **±77.52°** against the
biped's **±149.79°**), and an **outer bound** that forbids only true impossibility. That
derivation supplies the *magnitude* of every hinge and **cannot supply the SIGN**, because every
shipped plan is mirror-symmetric in **Z as well as X** — our bodies have no front — so the one
bit that separates a knee from an elbow is a **declaration**, and where it is absent the
derivation answers `Undetermined{reason}` loudly rather than inventing a side (the
`NothingDeclared` shape member #0 already ships). Enforcement lands at the three ruled points
with no firewall move and no per-frame cost: `validate_plan`'s existing keyframe walk rejects an
out-of-range clip at define time, `bake_resting_posture` gains one check on its own output, and
`solve_leg_ik` gains the limits as a **required argument** — which turns its hard-coded
knee-pole into a read of declared data and shrinks the reachable annulus from
`[|l1−l2|, l1+l2]` to `[d_min(limits), l1+l2]`, retiring by construction the degenerate 180°
stout crouch journal/0131 measured. With nothing declared anywhere the whole thing is an
**identity default** (S-5): zero rejections on the shipped pack, byte-identical render — and the
moment the biped's knee sign is declared it rejects **2 of the default pack's 3 clips**, which is
`dc:anim/biped_walk` hyperextending both knees by **28.65°** on every cycle, unnoticed since
bring-up.

---

## 1. The honest input inventory — stated BEFORE anything is derived [docket (a)'s discipline, applied here]

Verified at source, not assumed. `SegmentDef` carries `name, parent, pivot_m, size_m, offset_m,
tint, roles` — **no rotation range** (`bodies.rs:143-172`); `composition` is *deliberately
absent* with B6 named at the line. `solve_leg_ik`'s clamps (`body.rs:485,493`) keep `acos` in
domain and the target inside the annulus — **numerical guards, not anatomical ones**. Mass is
volume at density ≡ 1 (`stubs.md` #40). There is no muscle, no tendon, no ligament, no joint
capsule, no articular surface.

| quantity | derivable today? | why |
|---|---|---|
| **that a two-bone chain's REST is one END of the intermediate joint's range** | **yes** | member #0's zero-torque column is the extended configuration; a straight chain can only fold |
| **the magnitude of a fold before self-contact** | **yes** | closed-form box geometry (§ 3) |
| **that a hinge's other two axes are not DOFs** | **no** | two abutting boxes say nothing about articular shape; `solve_leg_ik` *assumes* a sagittal hinge and never declares it |
| **which SIDE a hinge folds toward** | **no — and structurally so** | every shipped plan has `pivot_m[2] = offset_m[2] = 0` on **every** segment: the bodies are fore-aft symmetric, so no anterior datum exists (§ 3.4) |
| **a ligamentous end-range** (hip extension, cervical rotation) | **no** | passive soft tissue is force; geometry over-predicts by 3–7× where it is the real limit (§ 3.5) |
| **a coupled limit** (knee twist range as a function of flexion; the screw-home) | **no** | needs articular surfaces |
| **a swing CONE** (a real ball joint's reachable set) | **no** | needs a capsule model; a 3-DOF box over-approximates it (§ 2.4) |
| **sibling collision** (the shank swinging through the other thigh) | **not under the recommended law** | it tests ancestors only (§ 3.6) |

**This bounds what a derived limit may CLAIM.** Everything in the "yes" rows is evidence. The
"no" rows are either a **loud `Undetermined`** or a **stand-in with a named heir** (§ 6). The
derivation never fabricates an anatomy answer.

**⚠ And it is the first thing to state about the ruling's own purpose.** The ruling wants the
derived default to make the primitive *"survive evolution — a mutated body must get plausible
limits by construction."* What is achievable today is an **impossibility bound**, not a
plausibility model: it keeps a body out of poses that would put its own boxes through each
other, and on a fore-aft-symmetric body it does **not** by itself forbid hyperextension.
That is finding **J1** and user call **Q1**; it is not a contest (the ruling says the derivation
"is owed a design pass and is not settled here" — this is that pass), but it is the gap a build
must not paper over.

---

## 2. THE REPRESENTATION — the hard question

### 2.1 What a "joint limit" has to constrain, here

Rotation is stored and applied per segment relative to its parent as an **XYZ Euler triple**:
`JointRot.euler` on the wire, `Pose.joints: HashMap<String, [f64;3]>` in the sampler, and
`Quat::from_euler(EulerRot::XYZ, x, y, z)` at the transform (`character.rs:321`). So a limit is a
constraint on a **subset of SO(3)**, and the three candidate representations disagree about
*which* subset — that disagreement is the whole question.

### 2.2 Candidate A — a per-axis min/max box in Euler space

`limits: [[f64;2];3]`. Cheapest possible; composes with the existing triple with no conversion.

- **Exact** where at most one component is non-zero — which is every hinge, and is where the
  order-dependence and gimbal objections evaporate: a rotation about a single coordinate axis is
  a one-parameter subgroup, and the box's face is exactly its boundary.
- **Order-dependent and non-invariant** the moment two components are live. `EulerRot::XYZ`'s
  composition order is glam's, not ours, and *"the same physical pose has two triples"* is a real
  hazard for a ball joint.
- **Cannot say "this joint has no twist"** — only "twist is pinned to [0,0]", which is
  operationally the same and semantically weaker: it produces the error *"y = 0.12 exceeds range
  [0,0]"* where the truth is *"a knee has no twist."*

### 2.3 Candidate B — swing cone + twist

Decompose the relative rotation into **swing** (where the child's bone axis points, relative to
rest) and **twist** (rotation about that axis); limit swing by an elliptical cone and twist by a
range. This is the standard rig representation and is **exact for a ball joint**.

- Requires a conversion at every check (quaternion swing-twist) and a **rest axis** per joint —
  which member #0 supplies, so that part is free.
- **It is exactly the thing the brief warns against for a hinge**: a knee becomes a cone with one
  half-angle 0 and an *offset* rest direction to carry the asymmetric range. A hinge expressed as
  a degenerate ball is a hinge whose one-DOF-ness is an accident of two numbers being zero — the
  same silent-coincidence shape as `arm_l_upper`/`arm_r_upper`'s hand-typed symmetry that opened
  this whole arc.
- Buys nothing today: no shipped joint's reachable set is measured by anyone, and the only ball
  joints we have (shoulder, hip) are geometrically **unbounded or near-unbounded** (§ 3.5), so
  the cone would be constraining nothing more precisely than the box.

### 2.4 Candidate C — a declared DOF list with per-DOF ranges — **RECOMMENDED** [assistant-proposed]

```rust
/// A segment-local rotation axis. X/Y/Z ONLY — the pose is an XYZ Euler triple,
/// so an oblique hinge axis is INEXPRESSIBLE (heir: a pose representation that
/// is not an Euler triple; see § 6, stub B7-d).
pub enum Axis { X, Y, Z }

/// One degree of freedom of a joint. Each END is independently declared or
/// derived — the finest granularity that is still meaningful (§ 4.2).
pub struct DofDef {
    pub axis: Axis,
    /// `None` = take the derived bound for this end.
    pub min_rad: Option<f64>,
    pub max_rad: Option<f64>,
}

pub struct SegmentDef {
    // …existing fields…
    /// `None`  = derive everything: three DOFs, both ends derived (IDENTITY DEFAULT).
    /// `Some([])` = welded: zero DOFs, this segment does not rotate at all.
    /// `Some(v)`  = exactly these DOFs; any axis not listed is NOT a DOF.
    #[serde(default)]
    pub dofs: Option<Vec<DofDef>>,
}
```

**Why this and not A.** C *is* A plus one thing: the DOF **set** is declared, so "absent" and
"pinned to zero" stop being the same statement. That distinction pays three times:

1. **`solve_leg_ik` already assumes the knee is a 1-DOF sagittal hinge** — it solves in the
   (y, z) plane and returns only `upper_x`/`lower_x`. Today that assumption is *undeclared code*;
   under C it is *declared data the solver can check*, and a plan whose knee is not a sagittal
   hinge gets a loud refusal instead of a silently wrong pose. **That is the same move
   `bake_resting_posture` already makes** with its *"not authored as a vertical column"* refusal
   (`bake.rs:422-433`) — same file, same shape, same author.
2. The **validator's error names the truth**: *"`knee_fl` declares one degree of freedom (X); a
   hinge has no twist"*, not *"0.12 exceeds [0,0]"*.
3. It is the **cardinality question one level down** from `stubs.md` #34 and answers it the same
   way B0 did: declare the capability, check the claim at define time, name the contenders.

**What it makes INEXPRESSIBLE — the E4 test, answered honestly in both directions.**

- ✅ **A rotation on an axis the joint does not declare cannot survive a define.** Not
  "detected and warned" — the plan is rejected, exactly as a clip animating an unknown joint is
  today (`validate_plan:509-514`).
- ✅ **The solver cannot be called without limits.** Recommended signature change:
  `solve_leg_ik(rig: &LegRig, target)` where `LegRig` carries the resolved limits, with **no
  unlimited overload**. There is then no way to produce an IK pose that ignores the joint —
  which is the one place in this design where inexpressibility is genuinely available, because
  the solver is *code we own*, not data flowing through a door.
- ⚠ **For authored DATA, inexpressibility is not available and should not be claimed.** A clip is
  a payload; the door's form of "cannot be written" is *rejected at define time*, which is
  precisely what the ruling specifies. Saying otherwise would be the E4 test misapplied.
- ❌ **A coupled range** (knee twist bounded by knee flexion) is inexpressible. Real; heir named.
- ❌ **A true swing cone** is inexpressible: a 3-DOF box's corners are poses a ball joint cannot
  reach, so the box **over-approximates** — it never falsely rejects, it under-catches. Recorded
  as an A-1 entry (§ 6, stub B7-c) with candidate B as the named heir, to land when a body exists
  whose shoulder range anyone measures.
- ❌ **An oblique hinge axis** (a bird's ankle, an insect's) is inexpressible, and the reason is
  worth stating as a design law: **the DOF vocabulary can only be as rich as the pose
  representation.** X/Y/Z is exactly what an Euler triple can carry; an arbitrary axis needs the
  pose to become axis-angle or quaternion, which is a wire change of a different order and is
  **not** in this window.

---

## 3. THE DERIVATION — what a plausible default range is, and from what

### 3.1 Three candidate geometric laws, all computable today [assistant-proposed]

**L1 — full box-vs-box self-intersection. DEGENERATE, and this is shown rather than guessed.**
A parent and child box **abut at the pivot** by construction (the shank's top face is at the knee;
the thigh's bottom face is at the knee). Rotating the child about X by any ε > 0 sweeps its
proximal-posterior corner `(y'=0, z'=+h_z)` to `(h_z sin ε, h_z cos ε)` — `y > 0` and `|z| < h_z`
— which is **inside the parent box**. So L1 returns **0° for every joint on every plan**. The
obvious derivation is unavailable on exactly our data, for a structural reason: *a cuboid rig's
boxes are a rendering of the limb, not its flesh, and real joints do not immediately collide
because bone ends are rounded and tissue is compliant — we have neither.* Any exclusion radius
that rescues L1 is **a tuning constant with no derivation**, which the doctrine forbids.

**L2 — the child's bone AXIS must not re-enter the parent's box.** The axis is the ray from the
pivot; it enters the parent's `y > 0` half-space at exactly 90°, and for small parameter values
its `|z|` is always inside the parent's depth. So L2 returns **exactly 90°, for every joint, on
every body, regardless of proportion.** A constant wearing a derivation's clothes. Rejected on
that criterion, which is the sharp one: *a law whose output does not vary with its input is not a
derivation.*

**L3 — the child chain's DISTAL EXTENT must not enter an ancestor segment's box. RECOMMENDED.**
Test the eight corners of the child subtree's terminal box (and its declared contact anchor,
where it has one) against each ancestor's box, as a function of the rotation; the limit is the
first angle at which any tested point is inside any ancestor box. Tuning-free (no exclusion
radius — the distal points start far from the pivot, so the L1 degeneracy cannot arise),
closed-form (each test is `A sin θ + B cos θ ≤ C`, one `asin`), **proportion-sensitive**, and
strictly an **outer bound**: it forbids only poses that are physically impossible for the body as
drawn, never a pose a real animal of that shape could hold.

### 3.2 The predicted numeric table — hand-derived from the plan literals, checkable at build

`g`-free pure geometry. Sagittal (X) axis, the axis every shipped clip keys. Angles are the
**symmetric** derived bound `±f_max` (L3 gives the same magnitude both ways for every shipped
plan, because every box is z-centred — see § 3.4).

| plan | joint (segment) | derived ±f_max | radians | first-contact geometry |
|---|---|---|---|---|
| `dc:body/biped` | knee `leg_*_lower` | **155.01°** | 2.70541 | shank distal-posterior corner `(−0.43, +0.09)` meets the thigh's posterior face `z = +0.10` |
| `dc:body/stout` | knee | **114.98°** | 2.00676 | `(−0.215, +0.13)` vs `z = +0.14` |
| `dc:body/longleg` | knee | **158.44°** | 2.76546 | `(−0.50, +0.09)` vs `z = +0.10` |
| `dc:body/biped` | hip `leg_*_upper` | **149.79°** | 2.61431 | thigh distal corner vs the trunk's `z = ±0.14`, within `y ∈ [0, 0.5]` |
| `dc:body/stout` | hip | **77.52°** | 1.35304 | short thigh (0.225) vs a deep trunk (`z = ±0.25`) |
| `dc:body/longleg` | hip | **UNDETERMINED** | — | the 0.52 m thigh's distal corner clears the trunk's **top** (`y = 0.5`) before its `z` re-enters — **L3's known miss, reported** (§ 3.6) |
| `biped` / `longleg` | elbow `arm_*_lower` | **155.72°** | 2.71787 | `(−0.28, ∓0.055)` vs the upper arm's `z = ±0.065` |
| `dc:body/stout` | elbow | **161.04°** | 2.81076 | `(−0.45, ∓0.07)` vs `z = ±0.08` |
| all three | shoulder `arm_*_upper` | **UNDETERMINED** | — | the arm's `x ∈ [0.265, 0.395]` and the trunk's `x ∈ [−0.25, 0.25]` **never overlap** — the arms are shouldered clear of the torso and geometry constrains them not at all. (Stout: `[0.39, 0.55]` vs `[−0.39, 0.39]` — a zero-measure graze at `x = 0.39` exactly) |
| `dc:body/biped` | neck `neck` — pitch (X) | **141.42°** | 2.46825 | head's distal corner `(0.40, ∓0.14)` vs the trunk's top |
| `dc:body/stout` | neck — pitch (X) | **130.78°** | 2.28291 | `(0.52, ∓0.22)` vs `z = ±0.25` |
| `dc:body/biped` | neck — yaw (Y) | **UNDETERMINED** | — | the head sits entirely above the trunk; a yaw never collides |
| `dc:body/biped` | neck — roll (Z) | **124.57°** | 2.17408 | `(0.40, ∓0.14)` vs the trunk's `x = ±0.25` |
| all | knee — twist (Y) | **UNDETERMINED** | — | a pure twist moves the distal corners in the `x–z` plane, below the parent; no contact at any angle |
| `dc:body/biped` | knee — lateral (Z) | **157.59°** | 2.75046 | `(x' = ±0.08, y' = −0.43)` vs the thigh's `x = ±0.09` |
| all | `head`, `trunk` | **UNDETERMINED** | — | `head` has no child chain to collide; `trunk` is the root (§ 4.4) |

*Every row is hand-derived from `default_pack.rs` / `experiments.rs` literals. **A divergence at
build is a finding about THIS document**, per the immutable-body rule — the banner obligation
falls on whoever measures.*

**What the table already earns.** The stout's **77.52°** hip against the biped's **149.79°** is
the derivation doing real work on real data: a squat body with a deep trunk and short thighs
genuinely cannot swing a leg as far forward, and nobody typed that. Same for the knee's
155 / 115 / 158 spread. **This is the property L2 lacked and the reason L3 is the recommendation.**

### 3.3 The measure-against-the-literature check [CLAUDE.md standing rule; `posture-gait.md` prior 4]

Published human ranges of motion (AAOS/AMA goniometric norms as reported in *Norkin & White,
Measurement of Joint Motion: A Guide to Goniometry*; passive knee flexion per *Kapandji, The
Physiology of the Joints*). **⚠ These are from the assistant's knowledge and were NOT
network-verified in this pass; the build should check them against the sources and record any
divergence as a finding about this document** — the same disclaimer the gait pass carries on its
Alexander/Hildebrand citations.

| joint | derived (biped) | published human | verdict |
|---|---|---|---|
| **knee flexion** | 155.01° | **135–150° active; ~160° passive** (heel-to-buttock) | **IN BAND.** From cuboids, with nothing fitted |
| **knee extension** | 155.01° | **0°** (hyperextension beyond is *genu recurvatum*, pathological past ~10°) | **the derivation is wrong by the whole range** — and this is the end the user's ruling is about |
| **elbow flexion** | 155.72° | **145–150°** | in band, ~4 % over |
| **elbow extension** | 155.72° | **0°** | wrong by the whole range — *and the sign is OPPOSITE the knee's* |
| **hip flexion** | 149.79° | **120°** (knee flexed) | ~25 % over — plausible-shaped |
| **hip extension** | 149.79° | **20–30°** | **over by ~5–7×** — the iliofemoral ligament, not bone |
| **cervical rotation (Y)** | undetermined | **60–80°** | geometry says nothing; **the client already hard-codes 75°** |
| **cervical flexion/extension (X)** | 141.42° | **45–50° / 45–70°** | over by ~3×; **the client already hard-codes 45°** |
| **cervical lateral (Z)** | 124.57° | **45°** | over by ~2.8× |
| **shoulder flexion** | undetermined | **180°** | the one place "unbounded" is nearly right |

**The pattern is the finding, and it is exactly the kind of thing only a literature check can
show: the derivation is good where the end-range is BONY and useless where it is LIGAMENTOUS.**
Knee and elbow *flexion* stop when the calf meets the thigh — geometry, and we get it right.
Hip extension and cervical rotation stop when a ligament stops them — force, and we have none
(`stubs.md` #40: mass is volume until B6). A closed system could never have told us which rows
were which.

### 3.4 The SIGN — the one bit geometry does not supply, and why [assistant-proposed; J1]

The magnitudes above are **symmetric**: L3 returns the same `f_max` in both directions for every
shipped joint, because **every box in every plan is centred in z**. Checked exhaustively at
source: in `biped_plan`, `stout_plan` and `longleg_plan`, **every** segment has
`pivot_m[2] = 0.0` and `offset_m[2] = 0.0`. Sole anchors are derived from `offset_m` and inherit
it (`with_sole`). **The shipped bodies are mirror-symmetric fore-and-aft as well as
left-and-right — they have no front.**

That is not a cosmetic observation. A knee folds posteriorly and an elbow folds anteriorly, on
chains of *identical topology and near-identical proportion*; the difference is which side the
flexors are on, which is anatomy we do not model. **On a body with no anterior asymmetry there is
no datum from which to derive it at all.**

Three ways to supply the bit, worked:

- **(a) Declare it.** One bound per hinge (`max_rad: Some(0.0)` for a knee). Trivial for a
  hand-authored body, and it is exactly what "declared overrides derived" is *for*. **Recommended
  for the default pack.**
- **(b) Take it from the plan frame's forward convention.** `default_pack.rs` states *"+Y up, −Z
  forward"*, and `solve_leg_ik` already hard-codes *"the knee pole points forward (−Z), so knees
  bend like knees"* — an undeclared joint limit living in a solver. Promoting it to the derived
  default gives every sagittal hinge "folds posteriorly": **right for the knee, wrong for the
  elbow.** Rejected as a default; it is however the strongest argument that the pole belongs in
  declared data (§ 5.3).
- **(c) Derive it from anatomical asymmetry, when a body has any.** The rule that works and is
  worth writing down now even though no shipped body exercises it: *a bearing chain's hinge folds
  so the distal segment moves **away** from the direction its terminal segment extends* — a
  plantigrade foot sticking forward is what makes the knee fold back. This becomes live the
  moment a plan has a foot, a snout or a tail segment offset in z. **An evolved body minted by
  deeptime will have such asymmetry; our three bring-up bodies do not.**

**The disposition [assistant-proposed]: where the sign is not derivable, the derivation returns
`Undetermined { reason }` for that end — a legal, loudly-named absence, treated as unbounded by
every consumer.** This is the shape the corpus already uses three times (`BakeOutcome::
NothingDeclared`, `unique_role_segment`'s `Ok(None)`, the bake's `Unsupported`) and it is what
makes B7 a strict **S-5 identity default**: with nothing declared, every plan validates exactly
as today and the render is byte-identical. It is *not* a silent nonsense answer — the define
emits one warning per undetermined end, naming the joint and the reason.

### 3.5 What the derivation CANNOT know — stated so a build cannot oversell it

At density ≡ 1 with no soft tissue: **passive tissue end-range** (hip extension, cervical
rotation — the 3–7× rows above) · **the fold sense** (§ 3.4) · **whether a joint is a hinge or a
ball** at all · **coupled ranges** (knee twist as a function of flexion) · **the true reachable
set of a ball joint** (a cone, not a box) · **anything about strength, so nothing about how far a
joint can be driven *under load* versus held passively**. Each has a proposed `stubs.md` entry in
§ 6.

### 3.6 L3's own known misses, reported rather than hidden

1. **Shaft contact.** L3 tests distal extent only, so a long child whose *tip* sails past an
   ancestor while its *shaft* would graze it returns `Undetermined`. **`dc:body/longleg`'s hip is
   exactly this case** and the table says so. Heir: a swept-volume test with the pivot
   neighbourhood handled properly (which is the L1 problem, and needs an articular model — B6
   adjacent).
2. **Siblings.** L3 tests **ancestors only**. The biped's knee lateral bound of 157.59° is
   against its own thigh; swinging the shank 90° medially would put it through the *other* leg,
   and L3 does not see it.
3. **Grandparent-and-beyond chains** are testable by the same law (walk all ancestors) and should
   be, but no shipped joint exercises more than one ancestor.

---

## 4. The MECHANICS of declared-overrides-derived

### 4.1 Where the declaration lives [assistant-proposed]

**On `SegmentDef`, as § 2.4's `dofs` field.** A segment *is* a joint (`bodies.rs:135-142`:
"a segment is a joint (a pivot) with a cuboid hung off it") and the rotation is relative to the
parent, so the range belongs to the child. The considered alternative — a separate `Vec<JointDef>`
on `BodyPlan`, keyed by segment name — was rejected: it mints **a second address-keyed binding
table**, which is `stubs.md` #34's exact defect (a name wearing a role) in a new costume, and it
lets a plan define with a limit for a segment that does not exist.

### 4.2 How partial declaration composes — **per END of per AXIS** [assistant-proposed]

Not per joint, not all-or-nothing, not even per axis: **each bound resolves independently**.

| `dofs` | meaning |
|---|---|
| `None` | three DOFs, all six bounds derived — **the identity default** |
| `Some([])` | **welded**: zero DOFs; any non-zero rotation on this segment is a define-time error |
| `Some([DofDef{X, min: None, max: Some(0.0)}])` | one DOF (X); **max declared, min derived** — the biped's knee, and the whole declaration is *one number* |
| `Some([DofDef{X, min:.., max:..}, DofDef{Y, None, None}])` | two DOFs; X fully declared, Y fully derived; **Z is not a DOF** |

**Why this granularity and no coarser.** The only thing the default pack actually needs to say
about its knee is *"it does not go past straight"* — one bound. Under per-joint or per-axis
composition an author must **restate the derived magnitude** (`−2.70541`) in order to change the
other end, and the moment that number is restated it is a copy that can drift from its authority
(**S-3**), *and* it is a convention asking an author to repeat something — which CLAUDE.md's own
`JUSTIFIED-BY` evidence says dies (documented twice, promised sweep, 3 uses, 0 in `crates/`).
Per-end composition means **a declaration is never larger than the fact it is asserting.**

### 4.3 What the validator checks, at define time [user-ruled: *"the validator rejects a clip that keys a joint past its range at define time"*]

**The hook already exists and already walks exactly the right thing.** `validate_plan`'s action
loop (`bodies.rs:491-517`) resolves every bound clip and iterates every keyframe's every
`JointRot` to check joint membership. B7 adds checks inside that loop — **no new traversal, no new
door, no new call site.** Note the consequence: **a clip is limit-checked against a PLAN, never
standalone** — `validate_clip` cannot see limits because limits live on the plan, and that is
correct rather than a gap (a clip is standalone until a plan adopts it — the module's own
documented define-order argument).

Checks, in order:

1. **Well-formedness of the declaration.** Each declared axis appears at most once per segment;
   every declared bound is finite; `min ≤ max`; and **`min ≤ 0 ≤ max`** — the authored rest is the
   Euler origin, so a range excluding zero would be a joint that can never be at rest. (Where
   member #0's bake returns a non-zero rest — the bird's folded leg behind the effort seam — the
   check becomes `min ≤ rest ≤ max`; recorded as the seam, not built now.)
2. **DOF membership.** For every `JointRot` in every bound clip: any Euler component on an axis
   the joint does not declare must be **exactly 0.0**. Non-zero ⇒ reject.
3. **Range.** Each component on a declared axis within `[min, max]`, where an `Undetermined` end
   is unbounded.
4. **Derived-limit self-consistency.** `derive_joint_limits` runs at define; a declared bound
   **outside** the derived one (a knee declared to fold 200° where its own boxes collide at 155°)
   is **reported, not refused** — same rule the gait docket ratified for band violations
   (*"the bake REPORTS when a modifier pushes an output past the published band — never refuses
   it; a clockwork golem may want to step wrong"*). The report names both numbers.

**What a violation says** (sketch; the shape of `validate_plan`'s existing errors):

```
plan `dc:body/biped` action `walk`: clip `dc:anim/biped_walk` keyframe 0 (t = 0.000)
  rotates joint `leg_r_lower` about X by +0.500000 rad (+28.648°), outside its range
  [-2.705414, +0.000000] rad ([-155.009°, +0.000°]).
  The upper bound is DECLARED (this joint folds one way); the lower is DERIVED from
  self-contact of the chain's distal extent against `leg_r_upper`.
```

```
plan `dc:body/wolf` action `walk`: clip `dc:anim/wolf_walk` keyframe 2 rotates joint
  `knee_fl` about Y by +0.120000 rad, but `knee_fl` declares one degree of freedom (X)
  — a hinge has no twist. Declare a Y DOF or key only X.
```

### 4.4 The root segment

`trunk` has no parent; its transform is the body frame. The renderer already writes the
**trunk facing** onto the body root entity (`Quat::from_rotation_y(orient.trunk_yaw)`,
`character.rs:236`) *and* applies the clip's Euler to the `trunk` **segment** transform
(`:321`) — so a clip keying `trunk` is a second authority over body orientation beside the
one `bodies.md` § *THE SIM OWNS THE TARGET* just ruled on. No shipped clip keys `trunk`, so
nothing is broken today. **Proposed [assistant-proposed]: the root segment's derived default is
`Some([])` — welded — so a clip keying the root is a define-time error naming trunk facing as
the owner.** Flagged as **J4** and routed to a user call rather than assumed, because it touches
a ruling made this week.

### 4.5 The window — stated explicitly [user-ruled sequencing]

`SegmentDef` is wire data (`DefineBodyPlan`, postcard, **positional** — corrections #3). Adding
`dofs` is:

- **Today: a recompile.** Nothing persists a `BodyPlan`; the default pack is compiled-in Rust
  re-emitted as a command batch at every world construction (`authority.rs::load_body_packs`).
  No golden moves — the field is `None` everywhere and changes no output.
- **After B3: a wire migration.** Once member 2 moves the firewall and the pose becomes a
  versioned sim asset, **the limits are part of what a pose validates against**, so a limit
  change becomes a world-identity change.

**This is the same deadline `stubs.md` #34 rides and the same one the gait pass's
`Keyframe.root_bob_m` removal rides.** Three wire-shaped changes, one window, and the
recommendation is explicit: **land them as ONE change, not three.** (`dofs` on `SegmentDef` ·
`JointRot.segment` → `BindTarget` · `Keyframe.root_bob_m` deleted.)

---

## 5. The three ENFORCEMENT POINTS, concretely

### 5.1 The VALIDATOR — and the day-one catch table

Under **derived-only** limits (nothing declared), the shipped content's catch rate is **zero**,
and that is the correct property for an outer bound that forbids only impossibility:

| clip | most extreme keyed angle | joint | vs derived bound |
|---|---|---|---|
| `dc:anim/biped_jump` | **−2.400 rad (−137.51°)** | `arm_*_upper` (shoulder) | passes — shoulder is **Undetermined** |
| `dc:anim/biped_jump` | −0.900 rad (−51.57°) | `leg_*_lower` (knee) | passes (155.01°) |
| `dc:anim/biped_walk` | +0.600 rad (+34.38°) | `leg_*_upper` (hip) | passes (149.79°) |
| `dc:anim/biped_idle` | +0.080 rad (+4.58°) | `arm_*_upper` Z | passes |
| every clip, `dc:body/stout` | −0.900 rad | knee | passes (114.98°) |

*The largest angle in any shipped clip is a −137.5° shoulder rotation, on the one joint geometry
cannot bound. That is a coincidence worth noticing rather than a design.*

**Under a DECLARED one-sided knee `[−2.705414, 0.0]`, B7 rejects 2 of the default pack's 3 clips:**

| clip | keyframe | joint | value | verdict |
|---|---|---|---|---|
| `dc:anim/biped_walk` | t = 0.00 | `leg_r_lower` | **+0.50 rad (+28.65°)** | **REJECT** |
| `dc:anim/biped_walk` | t = 0.25 | `leg_r_lower` | +0.20 (+11.46°) | **REJECT** |
| `dc:anim/biped_walk` | t = 0.50 | `leg_l_lower` | **+0.50 (+28.65°)** | **REJECT** |
| `dc:anim/biped_walk` | t = 0.75 | `leg_l_lower` | +0.20 (+11.46°) | **REJECT** |
| `dc:anim/biped_walk` | t = 1.00 | `leg_r_lower` | +0.50 | **REJECT** |
| `dc:anim/biped_jump` | t = 0.30 | `leg_*_lower` | +0.10 (+5.73°) | **REJECT** |

**`dc:anim/biped_walk` hyperextends both knees by 28.65° on every cycle, and has since bring-up.**
That is the user's *"hyper-extension just shouldn't be possible"* with a number against it, and
it is the strongest available evidence that B7 is a primitive rather than a nicety.

**Sequencing consequence, flagged loudly.** If B7's schema *and* the biped's knee declaration land
together, **the default pack fails to define and the game does not start.** The gait pass's
**user call #3** already retires `dc:anim/biped_walk` as shipped content (parked as a fixture) —
so the two rulings are consistent, and the order is forced:

> **B7's schema + derivation + validator land first with `dofs: None` everywhere (identity, zero
> rejections). The default biped's knee declaration lands with the gait slice that retires
> `biped_walk` and re-binds `walk` to the derived gait, in the same commit.** `biped_jump`'s
> single +0.10 keyframe is a one-line content fix at that point.

### 5.2 The BAKE — checks its own output and refuses loudly [user-ruled]

`bake_resting_posture` already answers in `BakeOutcome::{Baked, NothingDeclared, Unsupported}`.
B7 adds one check before `Baked`: **every joint angle in every `ChainPose` must be inside that
joint's limits**, else `Unsupported { reason }` naming the joint, the angle, the range, and which
end was declared vs derived.

**For the shipped plans this check is vacuous by construction** — the resting angles are all
exactly `0.0` and `0 ∈ [min, max]` is a validator invariant (§ 4.3 check 1). That is the right
kind of vacuous: it is a guard positioned *before* its trigger, and it goes live at the two places
already sequenced —

- **member #0's effort seam** (audit § 2.4): the moment the resting solve returns a non-zero rest
  (a bird's tendon-held folded leg), the rest can be outside a declared range.
- **member #1's derived poses**: the gait bake's `contact` and `clearance` are exactly where a
  derived pose can exceed a limit, and it is why the ruling says B7 is **upstream** of it.

**One concrete number to hand member #1 now.** The gait pass's Table B derives a hip excursion
`θmax` of **66.10°** for `dc:body/stout` at the shipped `walk_speed_m_s = 4.5`. The stout's
derived hip bound is **77.52°**. The run is legal — **at 85 % of the joint's entire range.** The
biped (47.95° vs 149.79°) and longleg (45.27° vs undetermined) are nowhere near. So the stout is
the body that will hit a limit first, which is exactly the body the arc exists to respect
(`stubs.md` #41's *"the worst-served body"*).

### 5.3 The IK — what "solve within the limit set" means for the shipped closed form

`solve_leg_ik(l1, l2, target)` today: clamps the planar distance into `[|l1−l2|+ε, l1+l2−ε]`,
computes the knee by law of cosines, **picks the pole with the smaller z** ("so knees bend like
knees"), returns `(upper_x, lower_x, reachable: bool)`.

**Three changes, and the solver's SHAPE does not change — a hard-coded choice becomes a read.**

**(1) The pole becomes data, not a heuristic.** With a declared one-sided knee the pole is
*determined* by the sign of the range. The `ka.1 <= kb.1` comparison is deleted, not
generalised. **This is the single cleanest thing B7 does to the solver**: the undeclared
anatomical assumption at `body.rs:497-503` becomes the declared bit of § 3.4.

**(2) The reachable set shrinks from an annulus to an annulus SECTOR.** With the knee restricted
to `[−f_max, 0]`, the honest inner bound is the distance at full flexion,
`d_min = √(l1² + l2² − 2·l1·l2·cos(π − f_max))`, replacing the *numerical* `|l1 − l2|`:

| plan | l1 | l2 | today's inner bound `\|l1−l2\|` | derived `f_max` | **honest `d_min`** | ratio |
|---|---|---|---|---|---|---|
| `biped` | 0.450 | 0.430 | 0.020 | 155.01° | **0.19143** | 9.6× |
| `stout` | 0.225 | 0.215 | 0.010 | 114.98° | **0.23660** | 23.7× |
| `longleg` | 0.520 | 0.500 | 0.020 | 158.44° | **0.19199** | 9.6× |

**And it retires a shipped degeneracy by construction.** `CROUCH_ROOT_DROP_M = 0.45` against the
derived hips:

| plan | crouch hip | knee flexion needed | today | **under B7** |
|---|---|---|---|---|
| `biped` | 0.880 − 0.45 = **0.430** | 121.55° (< 155.01°) | solves | solves, unchanged |
| `stout` | 0.440 − 0.45 = **−0.010** | — target is *below the ground* | inner-annulus clamp; **knee folds to a degenerate 180°**, ~10 mm residual (`bodies.md` § IK banner; journal/0131) | **honestly refused — `d = 0.010` is far inside `d_min = 0.2366`; the foot floats and says so** |
| `longleg` | 1.020 − 0.45 = **0.570** | 112.08° (< 158.44°) | solves | solves, unchanged |

*(The biped's 121.55° is the same quantity journal/0131 measured as a 123.7° crouching bend at
the pinned 0.900 m hip with the clip's forward foot offset — the same order, from independent
arithmetic, which is the cross-check that this pass's sign convention matches the corpus's
measured numbers.)*

**(3) `reachable: bool` becomes insufficient and should become a reason.** "Too far" and "too
folded" are opposite failures and the caller wants to distinguish them:

```rust
pub enum Reach { Ok, BeyondExtension, BeyondFlexion { joint: String }, JointBlocked { joint: String, axis: Axis } }
```

**What it does to the honest-float behaviour: it EXTENDS it, exactly as the ruling says.**
`character.rs:266-278` already applies an override only when `|adjust| ≤ half_voxel` and the
solution is finite, letting the foot float otherwise. B7 adds one condition — skip the override
when the solve leaves the limit set — leaving the clip pose. Three lines, no new failure mode,
and the same discipline one level up: *the foot floats honestly rather than lying about contact.*

**What "solve within" does NOT mean, said sharply:** solving freely and then clamping `lower_x`
into range. That moves the foot off its target with no signal — a working call, a passing test,
a silently wrong output, which is the exact failure the ruling forbids (**A-3**;
`ARCHITECTURE.md` § *A summary is not an authority*).

**The hip is checked, not solved into.** Restricting the *upper* joint turns the reachable set
from an annulus sector into a lune, which the closed form does not cover. Recommendation:
**check the hip limit and report `JointBlocked`; do not attempt to solve into it.** Costs
nothing today — the largest hip angle in any shipped clip is 34.38° against a 149.79° bound.

### 5.4 The fourth consumer nobody has named — `resolve_orientation`'s cervical clamp

`NECK_YAW_CLAMP_RAD = 75°` and `NECK_PITCH_CLAMP_RAD = 45°` (`body.rs:52-58`) are **anatomical
joint limits, hard-coded, world-global, and blind to the plan** — the stout's short thick neck
gets the biped's numbers. They are **A-1 in the family of the four absolute-metre constants this
arc has been retiring**, and § 3.3 shows they are *correct anatomy in the wrong place*: 75° sits
inside the published 60–80° cervical rotation band, 45° inside the 45–50° flexion band.

**B7 absorbs them: declare them as the biped's `look`-joint limits and the migration is
byte-identical (S-5).** Two notes:
- The **yaw** clamp is a *redistribution*, not a hiding clamp — the trunk absorbs the excess, so
  nothing is lost. That is legal under the ruling and under `bodies.md` § *THE SIM OWNS THE
  TARGET* (the gaze target survives; only the bend is clamped).
- The **pitch** clamp *is* a silent truncation (`look_pitch.clamp(±45°)`, with "no trunk pitch in
  v0"): a body can look somewhere its renderer cannot show it looking, and nothing says so. Under
  B7 it becomes a declared limit whose overflow is the same honest-float story. **Not a contest** —
  it predates the ruling and is precisely what B7 exists to absorb — but it is the one place a
  runtime clamp on a joint range is live in the tree today.

A fifth consumer is already written down and unbuilt: `bodies.md` § *Clothing* — a cape *"pinned
with constrained rotation… swings within limits."* Same primitive.

---

## 6. Proposed `stubs.md` entries — DRAFTED HERE, NOT WRITTEN [the integrator applies]

| id | title | what it fakes | heir | loudness owed |
|---|---|---|---|---|
| **B7-a** | `the-fold-sense-is-declared-because-our-bodies-have-no-front` | Every shipped plan has `pivot_m[2] = offset_m[2] = 0` on every segment, so no anterior datum exists and the derivation cannot say which way a hinge folds. The default pack **declares** it; an evolved fore-aft-symmetric body would get an unsigned range and hyperextension would remain possible for it | anatomical asymmetry (a foot, snout or tail segment supplies the direction — § 3.4(c)), plus **B6** for the flexor-side fact | the `Undetermined{reason}` warning at define, naming the joint |
| **B7-b** | `the-derived-limit-tests-distal-extent-only` | L3 tests the child chain's distal corners against **ancestors**. It misses **shaft contact** (`dc:body/longleg`'s hip returns Undetermined) and **siblings** (a shank swinging through the other thigh) | a swept-volume self-collision test with a derived articular neighbourhood — which needs an articular model, so it is B6-adjacent | the table's `UNDETERMINED` rows must print the reason, not a blank |
| **B7-c** | `the-euler-box-over-approximates-a-ball-joint` | A 3-DOF per-axis box's corners are poses a real swing cone cannot reach, so the limit **under-catches** for shoulders and hips. Never falsely rejects | swing-cone + twist (§ 2.3), to land when a body exists whose shoulder range anyone measures | in-module, at the `DofDef` definition |
| **B7-d** | `a-dof-axis-can-only-be-x-y-or-z` | The DOF vocabulary is bounded by the pose representation: an oblique hinge (a bird's ankle) is inexpressible while the pose is an XYZ Euler triple | a non-Euler pose representation — a wire change of a different order, explicitly **not** in the B3 window | in-module, at `Axis` |
| **B7-e** | `a-limit-that-cannot-know-soft-tissue` | The derivation over-predicts by 3–7× wherever the real end-range is ligamentous (hip extension 149.79° derived vs 20–30° published; cervical rotation unbounded vs 60–80°) and is in-band wherever it is bony | **B6** — passive tissue is force; the same integral that serves harvest yield, fitness, posture and flotation | the band report at define (§ 4.3 check 4) |

*Also widened rather than new: `stubs.md` **#40** (`mass-is-volume-until-b6`) gains a third
member — member #0's CoM, the gait bake's cadence/duty, and now B7's end-range.*

---

## 7. Which side of the firewall — the answer the brief asks for

**Limits are neither the target nor the approach; they are the DOMAIN both are drawn from**
[assistant-proposed, against `bodies.md` § *THE SIM OWNS THE TARGET* — DECIDED 2026-08-02].

- The **target** must be inside the limit set, or the sim is asking for a pose that does not
  exist. The **approach** must also stay inside it, or the client renders a pose B5's damage
  resolution never considered. Neither half owns the set.
- Concretely that means: **limits ride with the `BodyPlan`, which is ALREADY sim-side registry
  data** (`BodyPlanDef` in `HostState`), defined through the one door, receipted and replayed.
  Enforcement is at **define time**, which is already sim-side and already replay-critical.
- **No new tick state, no new wire message, no per-frame cost, and B7 does not move the
  firewall.** The whole sim-visible cost is the appended `SegmentDef` field — which is why the
  window (§ 4.5) is the entire sequencing argument.
- **Consequence for B3, recorded:** once the pose is a versioned sim asset, a limit change is a
  world-identity change. That is an argument for landing the field now and for landing the
  *declarations* deliberately rather than incrementally.

**Venue and placement** [user-ruled, corrections #86; member #0's placement is forced precedent].
`derive_joint_limits` is a **pure function in dc-api** beside `bake_resting_posture` — pure f64
arithmetic over plan data, no world, no RNG, no clock — callable from pack build, **deeptime
worldgen** (evolution mints species there) and define time. No new crate: two bakes plus a
derivation is not three instances of a shared home. **Returned, not stored** (S-3): a sibling of
`BodyPlan`, never a field of it, memoized by consumers exactly as `build_plan_assets` already
memoizes the resting bake.

**A-4 — the first real consumers, named, with edges.** Unlike member #0's slice, B7 has **three
from day one**: `validate_plan` (define time, in the loop that already walks every keyframe),
`bake_resting_posture` (one check before `Baked`), and `solve_leg_ik` (via `LegRig`, whose
construction in `leg_rigs` is where the limits get resolved). A fourth is a same-window
migration: `resolve_orientation`'s cervical constants (§ 5.4).

**Runtime cost, bounded not measured** (perf is first-class): the derivation is a handful of
`asin` calls per joint per ancestor — microseconds, **once per plan**, cached in `BodyAssets`
beside `legs` and `root_delta_m`. The per-frame path gains **zero** work: the IK's clamp bounds
are precomputed into the rig, and the pole comparison it replaces was itself two float compares.

---

## 8. Acceptance tests, mechanically

Invariants and derivations, never snapshots (CLAUDE.md § Gates); headless in dc-api except the
solver ones; the numeric report printed test-side like member #0's. Gate cost: microseconds.

1. **`nothing_declared_is_byte_identical`** — with `dofs: None` on every segment, `validate_plan`
   accepts all three plans and all three clips unchanged, and `leg_rigs`/`solve_leg_ik` produce
   **bit-identical** output to today across the existing measured sweep. *The S-5 acceptance.*
2. **`derived_limits_match_the_predicted_table`** — § 3.2's eighteen rows, to 1e-9 rad. *Member
   #0's bar. A divergence is a finding about this document.*
3. **`derived_limits_are_scale_invariant`** — uniformly scale a plan by k ∈ {0.4, 2.5}: every
   derived angle equal within 1e-12. *Angles are dimensionless; if a scale changes one, the law
   has a length in it that should not be there.*
4. **`stout_folds_less_than_the_biped`** — `f_max(stout knee) < f_max(biped knee)` and
   `f_max(stout hip) < f_max(biped hip)`, asserted as an **ordering**, not as magnitudes. *The
   property that makes L3 a derivation rather than a constant; it is what L2 fails.*
5. **`an_unbounded_joint_is_loudly_undetermined`** — the shoulder, the longleg hip and the neck
   yaw return `Undetermined` whose reason names the geometry (no x-overlap / clears the trunk top
   / no vertical overlap). *A legal absence, never a silent ∞.*
6. **`a_hyperextending_clip_is_rejected`** — with the biped's knee declared `[−2.705414, 0.0]`,
   `validate_plan` rejects `dc:anim/biped_walk`, and the error names the joint, the keyframe, the
   value and both bounds. **A-3 guard: this test is the reason the suite is not green for an
   unrelated reason** — test 1 proves the mechanism is off by default, this one proves it is
   armed.
7. **`an_undeclared_axis_is_rejected`** — a clip keying Y on a 1-DOF hinge fails, naming the DOF
   set. *DOF membership, distinct from range.*
8. **`a_welded_segment_refuses_any_rotation`** — `Some([])` rejects a non-zero key.
9. **`declared_overrides_derived_per_end`** — declaring only `max` leaves `min` at the derived
   value exactly; declaring both ignores the derivation; a declared bound outside the derived one
   is **reported and accepted**, not refused (§ 4.3 check 4).
10. **`the_ik_reachable_set_is_the_sector_not_the_annulus`** — `d_min` matches the § 5.3 table to
    1e-12; the stout's crouch target (`d = 0.010`) returns `BeyondFlexion`, and the caller leaves
    the clip pose. *The degenerate 180° knee, asserted gone.*
11. **`the_knee_pole_comes_from_the_declaration`** — flipping the declared sign flips the solved
    knee direction, with no code path selecting it. *The undeclared assumption, now data.*
12. **`the_resting_pose_is_inside_every_limit`** — for all three plans, all six bounds contain
    `0.0`, and the bake's own check passes. *Vacuous today by construction; it is the guard that
    goes live at the effort seam.*
13. **Report the numbers** — § 3.2, § 3.3 and § 5.3's tables, measured, printed `--nocapture`.

---

## 9. Loud findings — where the ruling or the code is missing machinery

*The practice working; the two preceding passes both did this. None is reconciled here.*

- **J1. The derived default is an IMPOSSIBILITY bound, not a plausibility model** (§ 1, § 3.4).
  The ruling's stated purpose for the derived default is that *"a mutated body must get plausible
  limits by construction."* Achievable today: a tuning-free outer bound that forbids self-
  intersection and is in-band where the real limit is bony. **Not achievable today: the fold
  sense** — so on a fore-aft-symmetric evolved body the derived default does **not** prevent
  hyperextension, which is the ruling's own opening motive. Routed to **Q1**; the ruling
  anticipated this pass and this is what it found.
- **J2. `dc:anim/biped_walk` hyperextends both knees by 28.65°, on every cycle, since bring-up**
  (§ 5.1). The defect the user named exists, in shipped default-pack content, with a number. And
  the clip is *internally* the same object the gait pass found to be inconsistent by 46 mm —
  two independent derivations landing on one hand-typed clip.
- **J3. `solve_leg_ik`'s knee pole is an undeclared joint limit living in a solver** (§ 5.3).
  *"The knee pole points forward (−Z), so knees bend like knees"* is an anatomical assertion in a
  comment, world-global, blind to the plan. B7 is where it becomes data.
- **J4. The root segment has two authorities over body orientation** (§ 4.4). The body root
  entity carries trunk facing; the `trunk` **segment** also receives a clip Euler. No shipped clip
  keys it, so nothing is broken — but the ruling that split target from approach was made this
  week and this is the third expression of the same noun. Proposed: weld the root. **Q4.**
- **J5. `NECK_YAW_CLAMP_RAD` / `NECK_PITCH_CLAMP_RAD` are correct anatomy in the wrong place**
  (§ 5.4). Two world-global absolute constants, in the published band, applied to every plan
  including a body with a 0.08 m neck. **A-1, in the family of the four this arc is retiring**,
  and the migration is byte-identical.
- **J6. `LegIk.reachable: bool` cannot carry the second failure** (§ 5.3). "Too far" and "too
  folded" want opposite caller behaviour and today share one flag. Small; it is a shipped type.
- **J7. The naive derivation is degenerate on our own data** (§ 3.1, L1). Worth recording as a
  finding rather than a footnote: the obvious answer returns 0° for every joint on every plan
  because cuboid segments abut at the pivot, and every rescue is a tuning constant.

**A-1 ledger for B7** (each simplification, its identity, its heir): fold sense = declared →
anatomical asymmetry + B6 (**B7-a**) · derived law = distal extent vs ancestors → swept volume,
siblings (**B7-b**) · ball joint = 3-DOF box → swing cone (**B7-c**) · DOF axis = X/Y/Z → a
non-Euler pose (**B7-d**) · end-range = geometry only → soft tissue, B6 (**B7-e**) · hip in the
IK = checked, not solved into → a solver that covers the lune · mass = volume × 1 → B6 (#40).

---

## 10. Compliance

**North star / two clocks:** derivation at define/bake — free clock, microseconds, once per plan;
the runtime samples and **gains zero per-frame work**. **Behavior is code** (the derivation) and
**tuning is data** (the declared bounds), cleanly split. Solver and derivation are **engine
primitives** in dc-api; bodies and their declarations are **pack content**; the derived limits
are derived data. No capability tiering — a third-party plan gets the derivation through the same
`validate_plan`-adjacent surface the defaults use.
**S-3:** the plan stays the authority; limits are *returned, not stored*, memoized by consumers,
never a copy beside their source. Declared bounds are authored data on the plan — authority, not
summary — which is why per-end composition matters (§ 4.2): a coarser granularity would force an
author to copy a derived number.
**S-5:** `dofs: None` is an exact identity default — test 1 asserts byte-identity — and
`Undetermined` preserves today's unbounded behaviour for every joint the derivation cannot reach.
**S-9:** unaffected; limits are per species (per plan), and nothing here quantizes or forecloses
a per-instance delta (an injured joint with a reduced range is a sparse overlay over the same
`[min, max]`, f64 throughout).
**A-1:** § 9's ledger; B7 retires two live instances (the cervical constants, the solver's pole).
**A-3:** test 1 and test 6 are deliberately paired — one proves the mechanism is off by default,
the other proves it is armed; a suite with only the first would be green for a reason unrelated
to what it asserts. **A-4:** three consumers from day one, § 7, with edges.

**⚠ CONTESTS flags: none.** Checked before filing. The ruling in `bodies.md` § *Joint rotation
limits* is taken as data throughout and nothing below it is re-argued; § IK remains CLOSED
(corrections #81) and is not re-presented; the postures ruling, the look-ownership ruling and the
sim-owns-target ruling are taken as given. **J1 is the one place a contest was possible** — the
ruling's expectation of the derived default versus what is derivable at density ≡ 1 — and it is
reported as a missing-machinery finding routed to a user call, *supplying* what the rule needs
rather than narrowing it, exactly as member #0's F2 did with the balance rule. Had this pass
resolved it by editing a document, that would have been corrections #65.

---

## 11. What I could not answer

*"I cannot answer this" is a first-class result.*

- **Whether the fold sense should ever be derived, or is permanently declared.** § 3.4(c)'s
  asymmetry rule is untestable — we have no body with a foot, a snout or a tail. Recorded so the
  first asymmetric body's pass re-derives rather than inherits.
- **Whether a 3-DOF Euler box's over-approximation ever costs anything.** No shoulder range in
  this project has been measured or looked at by anyone. The cone is the heir; the trigger is a
  body that needs it.
- **The right derived range for a joint whose rest is NOT the extended column** — the bird's
  folded leg behind member #0's effort seam. The check generalises (`min ≤ rest ≤ max`); the
  *derivation* does not, because L3 measures from a rest the effort term would move.
- **Whether the run-regime hip excursions are a problem.** The stout at 85 % of its derived hip
  range at 4.5 m/s is a number, not a verdict; the gait pass's user call #1 already re-typed that
  constant and this pass has no standing to read a taste call out of it.
- **The verdict on how a limited body LOOKS.** Every visible consequence here is temporal (a foot
  that stops floating, a crouch that refuses instead of folding) and **the user's live view is the
  senior instrument for anything temporal** — corrections #77/#80, three times over.

---

## 12. Open questions, triaged

**USER CALLS (ratification, not measurement):**

1. **Q1 — the derived default's honest scope (J1).** It is an impossibility bound, not anatomy;
   on a fore-aft-symmetric evolved body it does not forbid hyperextension. Accept that scope with
   **B7-a** as the named stand-in, or hold B7 for a derivation that can supply the sign (which
   needs anatomical asymmetry we do not have)?
2. **Q2 — the representation** (§ 2): the recommended **DOF list with per-axis, per-end ranges**,
   versus a plain Euler box (cheaper, weaker errors, no DOF concept) versus swing-cone+twist
   (exact for balls, wrong shape for hinges, buys nothing today).
3. **Q3 — the sequencing (§ 5.1), and it is the one that can break the game.** Schema +
   derivation + validator land with **nothing declared** (identity, zero rejections), and the
   biped's knee declaration lands **with the gait slice that retires `dc:anim/biped_walk`**? Or
   declare on day one and accept that the default pack stops defining until the clip goes?
4. **Q4 — weld the root segment** (J4), making a clip that keys `trunk` a define-time error
   naming trunk facing as the owner. Touches a ruling made this week.
5. **Q5 — migrate `NECK_YAW_CLAMP_RAD` / `NECK_PITCH_CLAMP_RAD` into the biped's declared `look`
   limits** (J5). Byte-identical at 75°/45°; it moves two engine constants into pack content.
6. **Q6 — one wire change or three** (§ 4.5): `dofs` on `SegmentDef` + `stubs.md` #34's
   `BindTarget` + the gait pass's `Keyframe.root_bob_m` removal all ride the same pre-B3 window.
   Recommended: one change.
7. **Q7 — the five proposed `stubs.md` entries B7-a…e** (§ 6), drafted here and *not* written into
   `stubs.md`; the integrator applies, with ordinals at merge.

**ENGINEERING (decide with a number, no ratification owed):**

8. **Verify glam's `EulerRot::XYZ` composition order** and confirm the per-axis box is
   order-invariant for our data (it is wherever at most one component is non-zero — every hinge;
   the neck is the one live two-component joint).
9. **Verify the § 3.2 table at build** — eighteen hand-derived angles. A divergence is a finding
   about this document (immutable body; the banner obligation falls on whoever measures).
10. **Verify the § 3.3 literature values against the cited sources.** This pass had no network
    access; the ranges are from knowledge, exactly as the gait pass's Alexander/Hildebrand
    citations were.
11. **`LegIk.reachable: bool` → a `Reach` reason enum** (J6), and `solve_leg_ik`'s signature
    taking the rig rather than two bare lengths (the E4 half of § 2.4).
12. **Whether `derive_joint_limits` walks all ancestors or just the parent.** All ancestors is
    correct and costs nothing at eleven segments; no shipped joint exercises more than one.
13. **Where the derivation is memoized** — `BodyAssets` beside `legs` and `root_delta_m` is the
    obvious slot and is exactly the consumer-memoization member #0's audit § 3(b) placed.
