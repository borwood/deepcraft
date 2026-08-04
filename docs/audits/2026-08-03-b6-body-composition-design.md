# B6 — per-segment body composition: a design pass against the animation path

**Status: DESIGN PASS. Nothing here is ratified and no code was changed.** The deliverable is
this document. Proposing a change and making one are different acts, and the second is the
user's call. **No cargo command was run**; every number below is derived from source literals
by closed-form arithmetic, and is **a prediction the build checks** (B7's table was confirmed on
13 of 18 rows and **refuted on 4**, and the refutation was the most valuable thing it produced —
this is written so the same is possible).

**Read with:** `bodies.md` §§ *Body plans*, *Postures*, *THE SIM OWNS THE TARGET*, *Joint
rotation limits*, *Individual proportion variation* · `posture-gait.md` §§ 2–6, § 7 member 1's
ratified docket · `ideas.md` § *The segment tree as one primitive* (**sketch, not decision**) ·
`materials.md` §§ *Granular material property sheet*, *DECIDED 2026-07-22 — one namespace* ·
`material-behavior.md` § 2 (forms) · `docs/audits/2026-08-01-body-plan-structure-design.md`
§ 5.6 · `docs/audits/2026-08-02-gait-bake-member1-design.md` § 5 ·
`docs/audits/2026-08-02-joint-limits-b7-design.md` § 1, § 6 · `stubs.md` #21 #39 #40 #43 #44 #49 ·
`dependency-graph.md` § 0, § 2b row B6 · `spines.md` S-3 S-5 S-9, A-1 A-4 A-7 ·
corrections **#65 #86 #93 #94 #97**.

**Provenance convention** (the 2026-07-25 rule, as the three preceding bodies passes):
mechanisms are marked **[user-ruled]**, **[bones]** (stated in a ratified design doc), or
**[assistant-proposed]** — *a hypothesis until ratified*.

---

## ⚠ 0a. THE FRAME — the user's ruling that governs every line below [user-ruled, 2026-08-03]

> *"the materials you recommend are ~ **testing materials**. we don't know if there are
> different types of bone and muscle yet, etc."*
>
> *"the bone you see, and potsherds etc are actually **bootstrapping artifacts that aren't
> integrated in any ratified system** currently."*

Everything § 5 proposes is a **TEST MATERIAL**: a stand-in that exists so the mass integral has
real numbers to integrate, and **nothing more**. It is not a roster, not a taxonomy, not a claim
about what bodies are made of. **There may be many kinds of bone and many kinds of muscle** —
cortical and trabecular; slow and fast twitch; the pneumatic bone that makes a bird a bird. This
pass does not know, does not guess, and is shaped so that going from one `muscle` to seven costs
**a roster append and nothing else** (§ 8 prices that append exactly).

**Nothing may be tuned to these numbers.** No constant derived from them, no golden pinned to
them, no calibration seated against them. *What goes wrong if someone does:* they are recalled
literature for **human** tissue applied to a **caricature** body — § 5.3 measures the caricature
at **2.43×** a human's volume. A constant fitted to them would be fitted to an accidental
composite of a real measurement and a bring-up geometry, and would then have to be *defended*
when either half moves. That is exactly how `root_bob_m`, the 0.900 m hip, `CROUCH_ROOT_DROP_M`
and the half-voxel window became load-bearing — **A-1, four instances in this arc already**.

**And the disposal test applies to this pass's own proposal** (CLAUDE.md § *EXISTENCE IS NOT
STANDING*): *if this did not exist, would we build it today, in this shape?* For a **tissue
taxonomy** the answer is **no** — bio/eco is ON HOLD and the gate is a user call. For **four
densities so that an integral stops integrating the number 1** the answer is **yes**. That is
precisely the line § 5 draws, and it is why the entries are **born marked** (§ 5.5) rather than
marked later.

---

## 0b. The recommended shape, in five sentences

A segment declares an **ordered, outward-going mixture** — `SegmentDef.composition:
Option<Vec<CompLayer>>`, a layer being `{ material: String, share: f64 }` — which is
`ideas.md`'s *"mixed materials per segment, radially ordered"* written down as data, with the
order **carried** even though the animation-side integral is order-invariant (§ 3.3 measures
what the order is worth to animation: **≤ 2.6 %** of a limb's swing inertia). One pure function
in dc-api beside the two existing bakes — `mass_properties(plan, densities, root_height_m)` plus
`subtree_inertia(...)` — returns mass, centre of mass and moment of inertia about any joint, and
is **returned, never stored** (S-3), exactly as `bake_resting_posture` and `derive_joint_limits`
are. The identity default `composition: None` means **density ≡ 1.0**, byte-identical to today's
proxy (`stubs.md` #40), and the property that makes this an **S-5** conversion is stronger than
an identity default — it is an *invariance*: **every animation-side output is independent of a
UNIFORM density**, so the entire animation payoff of B6 comes from heterogeneity and nothing
else. **The central design call is not a bodies question at all**: `MaterialProps` is a **total**
sheet of granular-and-geological axes with no way to say *"this axis does not apply to me"*, so
adding one living-tissue material forces answers to grain size, cohesion, sieve resistance,
permeability-when-packed-into-pores, solubility, weatherability **and** a compiler-forced
`RELEASE_DECLARATIONS` row — the recommendation is to **split the sheet into a universal core
plus declared facets, as its own materials-system slice, BEFORE B6 adds any material** (§ 6).
Finally and loudly: **this supplies the INERTIA half of every force-shaped hole and none of the
POWER half** — `cadence_scale` goes from wholly assumed to half derived, `swing_flexion` is the
one B6 plausibly closes, `bob_damping` is **not** absorbed by density at all (§ 9.2 — density is
not stiffness, and the gait pass's absorption table says otherwise), and push-off flexion is
pure actuation that a mass integral structurally cannot produce.

---

## 1. Priors — what the corpus already holds, before this pass says anything

*Swept first, per session-workflow § "Sweep the corpus BEFORE opening a design pass". The user
should never have to say "we've discussed this."*

| # | prior | where | provenance |
|---|---|---|---|
| 1.1 | *"**Mixed materials per segment**, radially ordered — legal here precisely because this is not the voxel renderer and can have its own domain-specific shape. Cost is bounded because the mixture is **species-level data**, not per-instance."* | `ideas.md:675-677` | **[user-ruled]** (sketch; *"nothing above may be built without its own ratification"*) |
| 1.2 | *"The same mass integral serves harvest yield, evolutionary fitness, and standing posture"* — arrived at independently from three directions | `posture-gait.md:205-208` | **[bones]**, cautiously ratified |
| 1.3 | **Buoyancy is a fourth rule over the same inputs** — centre of buoyancy against centre of mass *"needs exactly the per-segment volume and density the mass integral already wants"* | `posture-gait.md:187-192` | **[bones]** |
| 1.4 | The bake's whole input set is **segment geometry plus volume as the mass proxy at density ≡ 1**; length-scale facts are honest today and **anything depending on FORCE is not** | `posture-gait.md:274-283` (docket (a)) | **[user-ruled]** (assistant reasoning, user-ratified: *"i want the repo to stay aligned to it on this sprint"*) |
| 1.5 | Kind-3 knobs are **stand-ins for physics we have not built**; *"design the two sets so B6 **ABSORBS** the stand-ins rather than colliding with them"* | `posture-gait.md:310-315` (docket (c)) | **[user-ruled]** |
| 1.6 | **B6's build is blocked on the ROSTER, not the plan** — 26 materials, `bone` the only body-adjacent one, no flesh/muscle/fat/keratin/chitin/wood, and **no `define_material` command** | `2026-08-01-body-plan-structure-design.md` § 5.6, *"a measurement, not an opinion"* | **[assistant-proposed]**, measured |
| 1.7 | *"Shipping a bare `density: f64` would **narrow** the user's design — corrections #65's exact failure mode"* | same, § 5.6.3 and § 9 flag 3 | **[assistant-proposed]** |
| 1.8 | *"Whether **radially ordered** has any consequence for the mass integral… a sum is order-free, so probably none — **but I cannot attribute the intent**"* | same, § 10 | **[assistant-proposed]**, explicitly unresolved — **§ 3.3 answers the measurable half and leaves the attribution alone** |
| 1.9 | `mass-is-volume-until-b6` now feeds **three** members: member #0's CoM, the gait bake's cadence/duty, **and B7's derived end-range** | `stubs.md` #40 | filed |
| 1.10 | The derived joint limit is **good where the end-range is BONY and useless where it is LIGAMENTOUS** (hip extension 149.79° derived vs a published 20–30°) — heir **B6** | `stubs.md` #49; B7 § 3.3 | measured against the literature |
| 1.11 | The four `GaitKnobs` are **force-shaped holes wearing taste clothes**; `bob_damping` is the one with a measured target — **4.6 cm real vs 6.6 cm compass** (Saunders, Inman & Eberhart 1953), and the shipped biped derives **6.58** | `stubs.md` #44 | filed **LATE** — corrections #97 |
| 1.12 | The **run's flight phase** is not derivable at density ≡ 1; heir B6, its *"fifth consumer"* | `stubs.md` #39 | filed |
| 1.13 | **One namespace: block IS material** (DECIDED 2026-07-22) — *never let materials borrow a block identity* | `materials.md:525`, memory | **[user-ruled]** |
| 1.14 | **Forms are a closed set the machine owns**; *"a mod picks a material and a mode; it never invents a mode"* | `material-behavior.md:141-181` | **[bones]** |
| 1.15 | **Uniform scale is FREE**: the resting bake returns **scale-free outputs — angles plus a height ratio** (`bake.rs:81`) | `bodies.md` § *Individual proportion variation* | **[user-ruled]**, 2026-08-03 |
| 1.16 | The material registry has a **hard ceiling of 51 entries**, compile-time enforced (`MATERIAL_COUNT × FORM_COUNT ≤ 256`) | `stubs.md` #21; `inventory.rs:189-200` | measured |

**What the priors already settle, so this pass does not re-argue it:** that the integral is
wanted (1.2, 1.3); that a bare scalar narrows the user's design (1.1, 1.7); that the blocker is
the roster (1.6); that **force** is the boundary of what is derivable (1.4); that the knobs are
force-shaped and want absorbing rather than colliding (1.5, 1.11).

**What no prior holds, and what this pass therefore originates:** any *quantitative* statement
about what per-segment density is **worth** to the animation path. Grepped: the corpus contains
**zero** occurrences of "compound pendulum", "moment of inertia" or "natural frequency" outside
the two `GaitKnobs` doc lines that use *"muscle power against limb inertia"* as a phrase.
§§ 3.3, 9.1 and 10 are the first numbers.

---

## 2. The honest input inventory — stated BEFORE anything is derived
### [B7 § 1's method, applied here. Verified at source, not assumed.]

`SegmentDef` (`crates/dc-api/src/bodies.rs:202-247`) carries `name, parent, pivot_m, size_m,
offset_m, tint, roles, dofs` and **no composition**. The absence is deliberate and marked at
`crates/dc-api/src/bodies.rs:241-246`:

> `// DELIBERATELY ABSENT, heirs named (B0 ratification, 2026-08-01): … `composition` waits on`
> `// the materials roster (dependency-graph B6).`

*(The brief cites this marker at ~line 171; B7's `dofs` field pushed it to **241-246**. Recorded
because a stale line number in a brief is exactly the drift `doc-topology` exists to catch.)*

The mass proxy lives at `crates/dc-api/src/bodies/bake.rs:487-518`:
`v = size_m[0] * size_m[1] * size_m[2]`, volume-weighted box centres, divided by the volume sum.
`bake.rs:36-38` names B6 as the heir **at the line**. There is **no density, no material, no
muscle, no tendon, no elastic modulus and no metabolic term** anywhere in the bodies tree.

| quantity | derivable today? | derivable after B6-as-proposed? | why |
|---|---|---|---|
| **segment volume** | **yes** | yes | `size_m` product, `bake.rs:494` |
| **whole-body volume** | **yes** — biped **0.168388 m³** (P1) | yes | sum of the above |
| **centre of mass, uniform density** | **yes** | yes, unchanged | volume-weighted; `bake.rs:514-518` |
| **centre of mass, heterogeneous** | **no** | **yes** | needs a per-segment density |
| **absolute mass in kg** | **no** — at density ≡ 1 it reads "0.168 kg" | **yes** | the one thing a density buys outright |
| **moment of inertia about a joint** | **only up to an unknown scale** — the *ratio* is derivable, the value is not | **yes** | parallel-axis over boxes |
| **pendulum frequency of a limb, UNIFORM density** | **yes, exactly** — ρ cancels (§ 3.2) | yes, identically | `ω = √(gMd/I)` with `M` and `I` both linear in ρ and `d` independent of it |
| **pendulum frequency of a limb, HETEROGENEOUS** | **no** | **yes** | the whole animation-side payoff — and § 9.1 measures how small it is for a near-uniform body |
| **buoyancy / centre of buoyancy** | **no** | **yes** (`posture-gait.md` § 5's fourth rule) | needs density against the fluid's — **out of this pass's scope; named, not designed** |
| **muscle force / joint torque** | **no** | **NO — the load-bearing row** | density gives *inertia*; **actuation** is a property of the actuator, and density is not one |
| **joint stiffness / passive tissue end-range** | **no** | **NO** | stiffness is an elastic modulus. `MaterialProps` has no modulus axis and this pass does not add one (§ 9.2) |
| **metabolic cost of transport** | **no** | **partially** — *mechanical* external work is computable from masses + kinematics (Cavagna); *metabolic* cost needs a muscle efficiency we do not model | offered as a candidate, **claimed as nothing** |
| **the flight phase of a run** | **no** | **NO — and `stubs.md` #39 may over-attribute this to B6.** The ballistic *arc* of a CoM is mass-INDEPENDENT; what needs force is the **take-off impulse** (§ 9.3) | |

**This bounds what a build may CLAIM.** Every "yes" row is evidence. Every "no" row is either a
loud absence or a stand-in with a named heir (§ 11). **The integral never fabricates an
actuation answer**, and a summary that names a finished half must name the unfinished one —
§ 9 is that naming.

---

## 3. Where composition is declared, and in what shape

### 3.1 The declaration [assistant-proposed, implementing prior 1.1]

```rust
/// One layer of a segment's composition, ORDERED OUTWARD from the segment's
/// long axis: `[bone core, muscle, fat, skin]` reads exactly as authored
/// (ideas.md, user: "mixed materials per segment, radially ordered").
///
/// ⚠ The ORDER is carried and, today, read by NOTHING: the mass integral is
/// order-invariant (a sum is). It is preserved because it is the AUTHOR'S
/// declaration, not because a consumer exists — see the drafted stub
/// `a-composition-whose-radial-order-nothing-reads`, which names the order's
/// three prospective readers and BOUNDS what it is worth to the animation path
/// (≤ 2.6 % of a limb's swing inertia; § 3.3).
pub struct CompLayer {
    /// Namespaced material id, e.g. `dc:tissue/muscle`. **A STRING through the
    /// door**, resolved against the registry at define time — never a
    /// `MaterialId` constant inside body logic (A-7).
    pub material: String,
    /// Fraction of the SEGMENT'S VOLUME this layer occupies. Shares sum to 1.0,
    /// checked at define time: a sum that misses is a REJECTION, never a silent
    /// renormalisation (a silent renormalisation is A-3's shape — a passing
    /// test measuring nothing).
    pub share: f64,
}

pub struct SegmentDef {
    // …existing fields…
    /// `None` = **the identity default** (S-5): uniform density ≡ 1.0, i.e.
    /// today's volume proxy byte for byte (`stubs.md` #40).
    /// `Some(v)` = this ordered mixture.
    #[serde(default)]
    pub composition: Option<Vec<CompLayer>>,
}
```

Four properties, each load-bearing:

1. **It is the user's shape, not a narrowing of it.** A mixture, ordered, per segment,
   species-level — and species-level falls out for free, because `SegmentDef` is per *plan*,
   which is exactly prior 1.1's *"the mixture is species-level data, not per-instance."*
2. **Plain data plus opaque strings.** Survives the crossing constraint unchanged (north-star
   Deviation 1's cheap-insurance shape), same as B0's `RoleDef` and B7's `DofDef`.
3. **`share` is a fraction of VOLUME, not a thickness.** A radius/thickness encoding needs a
   cross-section model the box does not have, and it would make `share` *derived from* the
   segment's dimensions — coupling the author's declaration to geometry, which breaks the moment
   a body is scaled. Volume fractions are dimensionless and scale-invariant, which is what keeps
   **B8's size axis free** (prior 1.15).
4. **Every new field has an identity default.** `None` everywhere reproduces today exactly.

**Deliberately NOT in this struct:** a per-instance composition (per 1.1 it is species-level);
a `Vec<CompLayer>` on `BodyPlan` keyed by segment name — that mints a **second address-keyed
binding table**, which is `stubs.md` #34's defect in a new costume and the same rejection B7
§ 4.1 made; and any composition *derived* from role or segment kind — a `sole` is not made of one
thing, and a rule saying so is **A-1** waiting to happen.

### 3.2 The property that makes this an S-5 conversion — and it is stronger than an identity default

**A UNIFORM density factors out of every animation-side output.** Not approximately —
algebraically:

- **Centre of mass:** `Σ(Vᵢ ρ cᵢ) / Σ(Vᵢ ρ) = Σ(Vᵢ cᵢ) / Σ(Vᵢ)` when ρ is common. Unchanged for
  any ρ.
- **`root_height_ratio`:** pure geometry (`bake.rs:484`). Density never enters.
- **Pendulum frequency:** `ω = √(g M d / I)` with `M ∝ ρ`, `I ∝ ρ`, and `d` independent of ρ —
  so ρ cancels **completely**. Verified numerically (P4): the biped leg gives
  **ω = 4.125332 rad/s** at ρ = 1, at ρ = 600 and at ρ = 1010, to every printed digit.

Two consequences, and they decide how the slice is accepted:

- **The acceptance test is an invariance, not a snapshot.**
  `uniform_density_is_animation_invariant` — sweep ρ ∈ {1.0, 1010.0, 16000.0} on all three plans
  and assert every derived output equal within 1e-12 relative. *Exact byte-identity holds only
  at ρ = 1.0, where `v * 1.0 == v`; at other ρ the algebra cancels but the floating-point
  rounding does not, and a test asserting byte-identity there would be asserting an untruth.*
- **The whole animation payoff of B6 is heterogeneity.** A build that adds a uniform density and
  reports the gait unchanged has not found a defect — it has confirmed an identity. § 9.1
  measures how much heterogeneity a *fleshy* animal actually has, and the answer is **very
  little**, which is the most important finding in this pass.

### 3.3 "Radially ordered" — what it buys, measured, and why this pass does not consume it

**This is a SCOPING choice, stated as one. It is not a correction of the user's design, and the
order is preserved in the declaration exactly as authored** (§ 3.1). Prior 1.8 left the question
at *"probably none, but I cannot attribute the intent"*; the **measurable** half can be answered
without attributing anything.

Radial redistribution moves mass **within a segment's cross-section** and never along its long
axis. For a box `(x, y, z)` rotating about a horizontal X axis — the sagittal swing every gait
keys — the centroidal term is `m(y² + z²)/12`; only the **z** part can move, and neither the `y`
(long-axis) part nor the parallel-axis term `m·d²` can. So the order's entire reachable
influence is bounded by the extremes of the cross-section second moment: from `0` (all mass on
the axis) to `m·z²/4` (all mass at the two faces), against a uniform `m·z²/12`.

Measured on the shipped biped's leg (`default_pack.rs:156-171`; thigh 0.18 × 0.45 × 0.20, shank
0.16 × 0.43 × 0.18):

| quantity | value |
|---|---|
| leg moment of inertia about the hip, uniform ρ | **0.006848268** ρ·m⁵ |
| of which the radially-movable z-term | **0.000087437** — **1.277 %** |
| extreme range of `I` under radial redistribution | **[−1.277 %, +2.554 %]** |
| ⇒ extreme range of the pendulum frequency | **[−1.253 %, +0.645 %]** |
| (lateral Z axis, for completeness) movable x-term | 0.000070159 — **1.027 %** |

**Read this honestly in both directions.** It says radial ordering is worth **at most ±1.3 %** of
a derived cadence, and only under a physically impossible redistribution (all the mass at the
skin, or all of it on the bone axis) — so for the **animation-only** scope it is far below any
bar this project would accept a slice on. It does **not** say the ordering is unimportant: it is
exactly right for the consumers `ideas.md` and `posture-gait.md` already name — **harvest** (you
butcher inward: skin, fat, meat, bone), **damage penetration** (an arrow reaches different tissue
at different depth), and **buoyancy** (a blubber shell is a floating body's whole story). Those
are the order's real readers, and none of them is in scope here.

**So: carry the order, consume the shares, name the readers, record the bound.** Drafted as a
`stubs.md` entry (§ 11).

**⚠ Checked against corrections #65 before filing, and it is NOT a contest.** Nothing is
narrowed: the declaration is the user's shape, the ordering survives in the data, and what is
deferred is a **reader**, not the design. Had this pass proposed `density: f64` instead, *that*
would have been the contest — and prior 1.7 already says so.

---

## 4. The mass integral — one function, in dc-api, returned not stored

**Venue** [user-ruled by precedent, corrections #86; forced by member #0 and B7]. A pure `f64`
function over plan data — no world, no RNG, no clock — so it is callable from **pack build**,
**deeptime worldgen** (evolution mints species there) and **define time**. It lives in dc-api
beside `bake_resting_posture` and `derive_joint_limits`. **No new crate**: three pure
derivations is not three instances of a shared home.

**S-3, and it is the whole placement argument.** The result is a **sibling** of `BodyPlan` —
never a field of it, never registry state, never sim-read. If a sim pass ever keys on the
integral instead of on the body, that is the summary-wearing-authority defect
(`posture-gait.md` § 9 states this for the bake; it transfers verbatim).

```rust
/// Bulk density of each segment, index-aligned with `plan.segments`, resolved
/// from each segment's declared `composition` against a material registry.
/// `composition: None` → 1.0 — `stubs.md` #40's proxy, now a NAMED SEAM with an
/// identity default instead of a literal in the middle of a CoM loop.
///
/// A-7: materials are resolved BY DECLARED NAME. No `MaterialId` constant
/// appears in this module or in any other body module.
pub fn segment_densities(plan: &BodyPlan) -> Result<Vec<f64>, CompositionError>;

pub struct MassProperties {
    pub volume_m3: f64,
    pub mass_kg: f64,
    /// Ground frame, y = 0 at the ground — the frame `RestingPosture.com_m`
    /// already uses (`bake.rs:496-500`).
    pub com_m: [f64; 3],
}

/// Mass, volume and centre of mass over ALL segments at the resting pose.
pub fn mass_properties(plan: &BodyPlan, densities: &[f64], root_height_m: f64) -> MassProperties;

/// Second moment of the subtree rooted at `seg`, about the segment-local `axis`
/// through `seg`'s pivot, in the resting pose. Parallel-axis over boxes.
pub fn subtree_inertia(plan: &BodyPlan, densities: &[f64], seg: &SegmentDef, axis: Axis) -> f64;
```

**Why the density resolution is a separate step, and it is not ceremony.** It keeps the integral
**registry-free**: a third-party pack whose materials the engine has never heard of gets the same
integral, and the single place that touches `dc_core::materials` is a lookup by declared string
(`MaterialId::from_qualified_name`, `materials/mod.rs:178-183`). That is the A-7-clean shape —
*a lookup of a name the CONTENT supplied* is the exact inverse of *a name the PROCESS supplied*.
The integral itself never sees a material at all: it takes `&[f64]`.

**First consumers, day one — A-4 answered.** `bake_resting_posture`'s CoM loop
(`bake.rs:487-518`) is **rewritten in terms of `mass_properties`** rather than duplicating it —
the hoist-don't-duplicate rule (member #0 audit F6) that already moved `stance_chain` and
`offset_from_root` into one home. `subtree_inertia`'s first consumer is the gait bake's
derivation of `cadence_scale` (§ 9.4) — **and if that derivation is not in the same slice,
`subtree_inertia` does not ship in it.** A second function with no caller is A-4 with a nicer
signature.

**Runtime cost: zero.** Everything here is bake-clock (gen time is free; runtime is sacred).
Nothing is added to the per-frame path; `BodyAssets` memoizes exactly as it already does for the
resting bake and the derived limits.

---

## 5. The test materials — how few, and how they are marked so they cannot harden

### 5.1 The figures, with the disclaimer that governs them

> **⚠ NOT NETWORK-VERIFIED.** This pass had no network access. Every figure below is from the
> assistant's own knowledge, carried with the same disclaimer B7 § 3.3 and the gait pass's
> Alexander/Hildebrand citations carry. **The build should check them against the cited sources
> and record any divergence as a finding about THIS document** (immutable body, mutable header —
> the banner obligation falls on whoever measures).

| test material | density kg/m³ | recalled source |
|---|---|---|
| `dc:tissue/muscle` | **1060** | skeletal muscle, 1050–1060; the standard biomechanics segment-density literature (Dempster 1955; Clauser et al. 1969; Winter, *Biomechanics and Motor Control of Human Movement*) |
| `dc:tissue/fat` | **920** | adipose tissue, 900–950 (commonly 916) |
| `dc:tissue/bone` | **1900** | wet **cortical** bone ≈ 1.9 g/cm³. **Trabecular bone is 200–1000 apparent** — a wholly different number, and itself the sharpest illustration of the user's *"we don't know if there are different types of bone"* |
| `dc:tissue/void` | **1.225** | air, 15 °C at sea level (ISA) |

### 5.2 How few — three span it, four is the recommendation, and the arithmetic shows why

The set has exactly one job: let a mixture land on a **published whole-body density** while
being able to express heterogeneity in both directions. Worked (P2):

| mixture, by volume | whole-body density |
|---|---|
| all muscle | 1060.000 |
| muscle .72 / fat .28 | 1020.800 |
| **muscle .81 / bone .08 / void .11** | **1010.735** |
| muscle .78 / fat .06 / bone .08 / void .08 | 1034.098 |

**Three are the spanning minimum: muscle, bone, void.** Muscle is the bulk; bone is the only
thing *above* it; void is the only thing meaningfully *below* it, and it is what lets a body with
lungs land under muscle's own 1060. The third row reaches **1010.7 kg/m³** — within 0.1 % of the
figure the brief names — at fractions (8 % skeleton by volume, 11 % air) that are plausible
rather than fitted.

**Fat is the fourth, on the user's own sketch and not on necessity.** It is the layer
`ideas.md`'s radial ordering names by position (*bone at the core, muscle around it, fat and skin
outside*), and it is the one axis separating a heavy build from a light one without touching the
skeleton. **Recommend four**; record that **three** is the spanning minimum, so a user who wants
the smallest possible test set knows exactly which one to drop and what it costs.

**Not proposed, deliberately:** skin, keratin, chitin, cartilage, tendon, blood, ligament, wood.
Each is a real material with a real density, and **none changes a single animation output beyond
what a density between 920 and 1900 already expresses.** Adding them would be building a
taxonomy, which is exactly what § 0a forbids.

### 5.3 The free literature check — on DENSITY, never on MASS

Whole-body density is a **published, measurable, intensive** quantity, and it is the only
external check available to a system that is otherwise checking itself (CLAUDE.md § *A closed
system cannot detect its own scale error*). **A plan whose mixture integrates far off ~1010
kg/m³ is telling you the mixture is wrong** — checked against published data rather than against
ourselves.

**And the corresponding whole-body MASS must never be used as the check.** Measured (P1, P3): the
shipped biped's volume is **0.168388 m³**, against a real ~70 kg human's ≈ 0.069 m³ — the block
person is **2.43×** a human by volume, so at 1010.7 kg/m³ it weighs **170.2 kg**. That is not a
defect; it is a caricature, and the user's own verdict on the derived bob was *"a tad exaggerated
but — they're **block people**"* (journal/0148). **Density passes the literature check; mass
fails it by 2.4×; only one of the two is a claim about the mixture.**

*A precision worth stating, because two published numbers are easy to confuse: a
**hydrostatic-weighing** body density of 1040–1070 kg/m³ is measured after correcting for lung
gas, while the ~1000–1010 figure is the whole body **including** residual lung volume — which is
why humans are very nearly neutrally buoyant. The § 5.2 mixture is aimed at the second. The build
should confirm which convention the cited sources use.*

### 5.4 The `bone` in the roster is NOT evidence for any of this

`MaterialId::BONE` (`crates/dc-core/src/materials/mod.rs:87`, sheet at `:496-507`) carries
**density 1100, grain size 60 mm, cohesion 0.0**, and every extraction resistance of a loose
clastic. That sheet describes **bone fragments lying in soil** — a bulk pile — not a femur.
Cortical bone is **1900**; the 1100 is a packing fraction wearing a material property's clothes.

**And its disposal is not this pass's.** `bone`, `potsherd`, `knapping-debris` and `ash` are
bootstrap artifacts with **no standing** [user-ruled]; whether they are removed is its own
decision, and **burying it inside a B6 slice would hide it**. Named here so nobody reads
`dc:bone`'s sheet as a precedent — not proposed for removal, not proposed for repair.

### 5.5 How they are marked so they cannot harden into a ratified roster

Five mechanisms, and **the first four are structural rather than notational** — a convention
asking a future author to remember something dies (CLAUDE.md's own `JUSTIFIED-BY` evidence:
documented twice, promised sweep, 3 uses, 0 in `crates/`).

1. **The namespace says it.** `dc:tissue/*`, never `dc:muscle`. A slug carrying a `tissue/`
   prefix cannot be mistaken for a member of the geology roster it sits beside, and the
   qualified-name guard (`materials/mod.rs:707-720`) keeps the prefix honest.
2. **They belong to NO content class.** `GeologySet` selects deposition members from declared
   classes, so a material with no class membership is **never selectable by any pass** and a
   tissue can never appear in the ground. That is a *structural* guarantee, not a note — and it
   is also why no world golden should move (§ 8, P7).
3. **Their `RELEASE_DECLARATIONS` row is EMPTY, and the emptiness is asserted.** A test —
   `no_tissue_declares_an_edge_product` — pins that every `dc:tissue/*` material releases
   nothing, so no weathering or transport pass can begin consuming them by accident.
4. **They have no granular facet at all** (§ 6). Under the recommended sheet split a tissue
   material **structurally cannot answer** a grain-size or sieve question: the unsafe read is
   *inexpressible*, the same move `CoarseField` made for the raw per-cell read and `FieldKernel`
   made for the stability bound.
5. **One `stubs.md` entry covers the whole set**, with the user's caveat quoted verbatim as its
   *what it fakes*, plus an in-module doc comment at the four registry rows: *these are TEST
   densities so an integral has something to integrate; there may be many kinds of bone and many
   kinds of muscle; nothing may be tuned to them.*

---

## 6. THE CENTRAL DESIGN CALL — `MaterialProps` is TOTAL
### *A materials-system question, not a bodies one — and it is why B6 cannot start today.*

### 6.1 The problem, at source

`MaterialProps` (`crates/dc-core/src/materials/mod.rs:281-352`) has **ten** fields and every one
is mandatory:

`name` · `albedo` · `density_kg_m3` · `grain_size_mm` · `cohesion` · `extraction_resistance[5]` ·
`permeability` · `insulation` · `solubility` · `weatherability`

For `dc:tissue/muscle` exactly **three** are honest questions — name, albedo, density. The rest
are **category errors**:

- *"Characteristic grain size in mm — what fits into which pores"* — muscle has no grain.
- *"Permeability contribution when packed into pores"* — muscle is not packed into pores.
- *"Insulation contribution when packed into pores"* — same.
- `extraction_resistance[Sieve]` — and the registry carries an **invariant test**
  (`sieve_resistance_equals_grain_size`, `:735-747`) forcing sieve resistance to *equal* the
  invented grain size. **A lie propagated by a passing test.**
- `properties_are_sane` (`:750-771`) asserts `grain_size_mm > 0`, so the honest value (absent)
  is not even representable.
- `solubility` — the field doc says every entry is *"honestly 0.0"* because *"none of them
  dissolve"*. Tissue in water is not zero on any timescale, and asserting it in
  `nothing_in_the_current_roster_dissolves` (`:774-788`) would be asserting something false
  **about a mechanism we do not model for it**.
- `weatherability` — *"how readily this material, when it **outcrops as bedrock**…"*. Muscle
  does not outcrop.
- **And one the sheet does not even contain:** `RELEASE_DECLARATIONS`
  (`crates/dc-core/src/materials/release_vanilla.rs:285`) is `[&[EdgeProducts]; MATERIAL_COUNT]`,
  so the **compiler forces a row** declaring what muscle produces when it weathers.

**The sheet has no way to say "this axis does not apply to me."** That is fine while every
material is granular and it breaks the instant one is living tissue. It will break again on wood,
on ice, on metal and on every worked material this game eventually has — **which is why the
answer belongs to the material system, not to bodies.**

### 6.2 The options, worked

**A — fill every field with a plausible-ish number.** *Rejected, decisively.* Cheapest by far,
and it does not merely lie in a comment: it makes muscle a **geological material** — a legal
`Structure`/`Loose` portion, a candidate in every `MaterialId::all()` enumeration, a row in the
release table, a layer in the terrain atlas, and a **placeable block name** through
`block_from_name` (§ 8). It also fails § 0a's own test on its own terms: the project already has
one generation of unratified material content it is fighting, and this creates a second — *while
being the option that is invisible in a diff.*

**B — `Option<T>` on every field.** *Rejected on semantics before cost.* `None` cannot
distinguish *"does not apply to me"* from *"nobody has measured it yet"*, and those want opposite
consumer behaviour (refuse vs fall back). It also spreads the decision over **69 read sites**
(§ 6.3) instead of one decision per material.

**C — split the sheet: a universal core plus declared FACETS. RECOMMENDED** [assistant-proposed].

```rust
pub struct MaterialProps {
    pub name: &'static str,
    pub albedo: [f32; 3],
    /// Bulk density. UNIVERSAL: every material that can exist has one, and it is
    /// a property of MATTER rather than of granularity.
    pub density_kg_m3: f32,

    /// Present iff this material occupies volume as GRAIN. A material without it
    /// cannot be sieved, packed into pores, or asked its angle of repose — the
    /// question is INEXPRESSIBLE rather than answered wrongly.
    pub granular: Option<GranularProps>,  // grain_size_mm, cohesion,
                                          // extraction_resistance,
                                          // permeability, insulation
    /// Present iff this material can outcrop as bedrock and be weathered.
    pub geologic: Option<GeologicProps>,  // solubility, weatherability
}
```

- It is the **north star's own shape**: *materials, their behavior, and the passes over them
  authored in a uniform, **self-declaring**, compiler-validated shape.* A material declares what
  it **is**; a consumer that needs a facet asks and gets a loud absence.
- It is the **forms doctrine one level out** (`material-behavior.md` § 2, prior 1.14): a closed
  machine-owned set of modes a material *picks*. A facet set is the same thing for property
  axes, bounded by the same argument — the machine must reason about each facet, so facets are
  machine additions, not a mod surface.
- It is the corpus's **honest-absence** pattern, already used four times:
  `BakeOutcome::NothingDeclared`, `Reach::Undetermined{reason}`, `unique_role_segment`'s
  `Ok(None)`, and `has_contents: false` meaning *no record* and never *air*.
- **And it makes the wrong read inexpressible**, which is the property this project consistently
  rates above a check.

**D — a separate `TissueProps` registry; tissues are not `MaterialId`s.** *Rejected, and it
contests a DECIDED ruling.* Zero blast radius, no niche pressure, no atlas layer, no release row
— genuinely the cheapest. But **one namespace: block IS material** (DECIDED 2026-07-22, prior
1.13: *never let materials borrow a block identity*), and P11's ruling 1 put `MaterialId` in the
deep record for the same reason. A fork also breaks the crossing case this whole thread is about:
a butchered carcass yields **meat**, a felled tree yields **wood**, a bone becomes a **tool** —
each must be storable, droppable, cookable, i.e. a material. **Named and rejected on the ruling,
not on taste.**

**E — a `domain` tag beside the fields saying which are meaningless.** *Rejected as strictly
weaker than C at the same cost.* The values stay readable, so a consumer that forgets the tag
reads a sentinel and does arithmetic with it. That is a lie with a note attached, which is the
shape `ARCHITECTURE.md` § *A summary is not an authority* exists to forbid.

### 6.3 The measured cost of C, and the sequencing it forces

**Read sites, measured** (in `crates/`, `.field` form, excluding the `REGISTRY` literal itself):

| axis group | read sites | files |
|---|---|---|
| granular + geologic (`grain_size_mm`, `cohesion`, `permeability`, `insulation`, `extraction_resistance` / `resistance()`, `solubility`, `weatherability`) | **69** | **14** — heaviest external readers: `deeptime/lithology.rs` 12, `deeptime/weather_behavior.rs` 9, `examples/entry_species_probe.rs` 20, `extract.rs` 2, `packing.rs` 2, `geology.rs` 2 |
| `density_kg_m3` | **6** | 4 |

A real but **bounded and mechanical** slice: the compiler finds every site, every existing
material keeps both facets `Some(..)`, so the conversion is **byte-identical** and no world
golden may move. It is exactly the seam-first shape this project uses.

**⇒ The sequencing recommendation, which is the practical output of this section:**

| slice | what it does | roster? | renderer? | goldens |
|---|---|---|---|---|
| **B6-a** | `mass_properties` + `subtree_inertia` in dc-api; `bake_resting_posture`'s CoM loop rewritten in terms of it; a `segment_densities` seam whose identity is 1.0 | **no** | **no** | byte-identical |
| **B6-b** | the `MaterialProps` facet split (option C) — a **materials-system** slice with its own justification, not a bodies one | no | no | byte-identical |
| **B6-c** | the four test materials + `SegmentDef.composition` + the default pack's declarations | **yes** | **yes** (§ 8) | see P7 |

**B6-a is startable today and blocked on nothing.** It is the honest first slice: two consumers
from day one, no output changes, and it converts `stubs.md` #40 from *a literal in the middle of
a CoM loop* into *a named seam with an identity default* — the S-5 move the corpus has made 31
times.

**And it is worth stating what B6-a explicitly does NOT do**, so a build cannot drift: it does
**not** add `composition`, because with no tissue materials in the registry there would be
nothing legal to put in it. **A field nobody can fill is A-4 with a schema attached.**

**The `define_material` door is the named heir and this pass does not design it.** The roster is a
compiled-in `const [MaterialProps; MATERIAL_COUNT]`, and *"a registry designed before its callers
exist is the same mistake in a new coat."* Four compiled-in test materials are the seam-first
answer; the general door lands when a pack that is not ours needs one.

---

## 7. The identity default and the byte-identity story — the acceptance tests

Invariants and derivations, never snapshots (CLAUDE.md § Gates). Headless in dc-api; gate cost
microseconds.

1. **`nothing_declared_is_byte_identical`** — with `composition: None` on every segment,
   `bake_resting_posture` returns a **bit-identical** `RestingPosture` for all three plans
   against its pre-B6 self, and `derive_joint_limits` is untouched. *The S-5 acceptance.*
2. **`uniform_density_is_animation_invariant`** — ρ ∈ {1.0, 1010.0, 16000.0} uniform: CoM,
   `root_height_ratio`, every `subtree_inertia` **ratio** and every derived cadence equal within
   1e-12 relative. *§ 3.2's invariance, asserted as the property rather than as a number. This is
   the test that stops a build accepting "the gait changed" as evidence.*
3. **`mass_scales_linearly_and_com_does_not`** — scale every density by k: `mass_kg` scales by k
   exactly; `com_m` unchanged within 1e-12. *Separates the extensive output from the intensive
   ones.*
4. **`the_com_loop_has_one_authority`** — `bake_resting_posture`'s CoM equals
   `mass_properties(..).com_m` bit for bit. *S-3: a consumer that re-derives, never a copy that
   can diverge — the failure `root_bob_m` cost us twice (corrections #80, #93).*
5. **`shares_must_sum_to_one`** — a composition summing to 0.99 is **rejected at define time**,
   naming the segment and the sum; never renormalised. *A-3: a silent renormalisation is a
   passing test measuring nothing.*
6. **`an_unknown_material_is_refused_with_a_receipt`** — a composition naming
   `dc:tissue/unobtanium` is refused, not defaulted. *S-5's sharp edge, which the per-character
   body-plan seam already earned: an identity default covers the ABSENT case and says nothing
   about the WRONG one.*
7. **`the_derived_whole_body_density_is_in_the_published_band`** — the default pack's biped
   integrates to a whole-body density inside **[950, 1100] kg/m³**. *The one external check.
   Asserted as a BAND, never as a value; and the band is published, not chosen.*
8. **`no_tissue_declares_an_edge_product`** + **`no_tissue_belongs_to_a_content_class`** — § 5.5's
   two structural guarantees that keep tissue out of the ground.
9. **`radial_order_does_not_change_the_mass_integral`** — reversing a segment's layer list leaves
   `mass_properties` bit-identical. *Pins § 3.3's order-invariance as a property, so the day a
   consumer DOES read the order, this is the test that fails and says so.*
10. **`derived_inertia_matches_the_predicted_table`** — § 10's rows, to 1e-9. *Member #0's bar. A
    divergence is a finding about this document.*
11. **Report the numbers** — §§ 3.3, 5.2, 9.1 and 10's tables, measured, printed `--nocapture`,
    asserting nothing where the comparison is to a published band (a published band is not ours
    to assert).

---

## 8. The roster-widening blast radius, traced at source

*Read, not assumed. Every row is `file:line`.*

| # | site | what a 4-material append costs |
|---|---|---|
| 1 | `dc-core/src/materials/mod.rs:42` `MATERIAL_COUNT = 26` | → **30** |
| 2 | `dc-core/src/materials/mod.rs:55-61` `MatRepr`, `#[repr(u8)]` fieldless, variants `M0..M25` | → `M0..M29`. **The niche is SAFE**: it exists because the compiler knows `MATERIAL_COUNT..=255` are invalid bit patterns, so it survives any count ≤ 255. `Block` stays **one byte** and the 64→32 KiB/chunk win is untouched |
| 3 | `dc-core/src/materials/mod.rs:63-66` `const _: () = assert!(MATERIAL_COUNT == 26, …)` | must be bumped in the same edit; **compile-time**, never silent |
| 4 | `dc-core/src/materials/mod.rs:190` `MATERIAL_QUALIFIED_NAMES: [&str; MATERIAL_COUNT]` | 4 compiler-forced rows |
| 5 | `dc-core/src/materials/mod.rs:363` `REGISTRY: [MaterialProps; MATERIAL_COUNT]` | 4 compiler-forced rows — **§ 6's entire subject** |
| 6 | `dc-core/src/materials/mod.rs:735` `sieve_resistance_equals_grain_size` · `:750` `properties_are_sane` (`grain_size_mm > 0`) · `:774` `nothing_in_the_current_roster_dissolves` | three registry-invariant tests a tissue row must satisfy honestly or the slice must restructure. **Under option A they are satisfied by lying** |
| 7 | `dc-core/src/materials/release_vanilla.rs:285` `RELEASE_DECLARATIONS: [&[EdgeProducts]; MATERIAL_COUNT]` | 4 compiler-forced rows. Recommended value **`&[]`**, asserted (§ 5.5.3) |
| 8 | `dc-worldgen/src/deeptime/inventory.rs:195-200` `assert!(MATERIAL_COUNT * (FORM_COUNT as usize) <= 256)` | **THE HARD CEILING: 51 materials** (`256 / 5`), `stubs.md` #21. 26 today → **30**, leaving **21**. Fails at compile time, never silently. *A tissue taxonomy of the size § 0a warns about — seven muscles, four bones — consumes half the remaining headroom, which is a real argument for keeping the TEST set at four* |
| 9 | `dc-core/src/materials/geology.rs:273,350` `by_material: [Option<GeoMemberIdx>; MATERIAL_COUNT]` | array grows; no code change |
| 10 | `dc-worldgen/src/deeptime/species.rs:98,126,140,161` `slot: [u8; MATERIAL_COUNT]`, `present: [bool; MATERIAL_COUNT]` | arrays grow; no code change |
| 11 | `dc-api/src/host.rs:50-59` `block_from_name` · `:61-70` `block_name` · `:103-108` `KNOWN_BLOCK_NAMES` | **every tissue becomes a placeable, diggable block name and a completion entry.** That is the cost of *block IS material* (prior 1.13) — a consequence to state, not a defect, and worth the user knowing before it shows up in the dev surface |
| 12 | **CLIENT** `dc-client/src/meshing.rs:86` `ATLAS_LAYER_COUNT = MATERIAL_COUNT + BLOCK_ONLY_SLUGS.len()` | 30 → **34** layers in three `texture_2d_array`s |
| 13 | **CLIENT** `dc-client/src/terrain_material.rs:43` `PALETTE_LEN` · `:234-240` `layer_slugs()` · `:244-263` `load_layer` | the loader tries `assets/textures/placeholder-labpbr/<slug>/{basecolor,normal,specular}.png` per layer; **missing files WARN and fall back to flat albedo** — four tissues = **12 startup warnings** unless textures are generated (`tools/gen_placeholder_textures.py`) |
| 14 | **CLIENT, and this is the one that breaks:** `dc-client/src/terrain_material.rs:388` `assert_eq!(PALETTE_LEN, 30);` **paired with** `dc-client/src/shaders/terrain_fullbright.wgsl:19-23` `array<vec4<f32>, 30>` | **the WGSL size is hardcoded and naga only compiles it at PIPELINE BUILD.** The test's own comment: *"this guard is the only CPU-side gate that sees the mismatch (journal/0020)."* Miss the shader edit and the fullbright pass fails **on the GPU at runtime**, not in the gate. **A roster append reaches the renderer, and it reaches it through a shader constant** |
| 15 | `dc-worldgen/src/deeptime/recorder.rs:1233`, `examples/member_diversity_probe.rs:82-84`, `examples/entry_species_probe.rs` | probes and reports enumerating `MaterialId::all()` gain rows; several are gate-run examples (`test = true`), so their printed tables change |

**End to end, a roster append costs:** 3 compiler-forced tables (4, 5, 7) · 1 hand-bumped
compile-time assert (3) · 1 shader constant and 1 Rust guard **that must move together** (14) ·
3 registry-invariant tests to satisfy honestly (6) · 12 optional texture files (13) · a widened
block-name vocabulary (11) · **21 slots of headroom** against a compile-time ceiling (8).
**Nothing is silent** — every one fails loudly — which is the good news, and it is why the append
is a *decision* rather than a *risk*.

---

## 9. What this CANNOT do — the unfinished half, named as loudly as the finished one

### 9.1 The measured payoff, and it is smaller than the arc's framing implies

Per-segment density changes an animation output **only through heterogeneity** (§ 3.2). So the
honest question is: *how heterogeneous is a fleshy animal?* Measured on the shipped biped's leg
swinging about the hip (P4, P5):

| per-segment densities | ω (rad/s) | Δ vs uniform |
|---|---|---|
| uniform, **any** ρ | **4.125332** | — |
| Dempster-like: thigh 1050 / shank 1090 | 4.114713 | **−0.257 %** |
| hollow-boned flier: uniform 600 | 4.125332 | **0.000 %** |
| armoured shank: thigh 1060 / shank 1900 | 3.985832 | −3.38 % |

And on the whole-body CoM (P6), root-relative:

| densities | mass | CoM y | Δ |
|---|---|---|---|
| uniform 1010 | 170.072 kg | **+0.088187 m** | — |
| Dempster-like per segment | 178.601 kg | +0.088071 m | **−0.116 mm** |
| lungs in the trunk (800) / rest 1100 | 164.227 kg | +0.067496 m | **−20.7 mm** |

**The finding, stated plainly: real tissue-density variation across a fleshy animal's segments
moves the derived gait by ~0.25 % and the centre of mass by ~0.1 mm.** The animation-visible
payoff of B6 comes from **VOID** — lungs, air sacs, pneumatic bone — and from **armour**, not
from telling muscle apart from fat. `stubs.md` #40 already says this in words (*"a hollow-boned
flier, a heavy-tailed biped, an armoured body"*); this is the number underneath it.

**Which is exactly why an acceptance criterion must not be "the gait looks different."** For the
three shipped near-uniform bipeds it will barely be different, and that will be **correct**.

### 9.2 ⚠ Density is not stiffness — a correction to the gait pass's absorption table

`docs/audits/2026-08-02-gait-bake-member1-design.md` § 5 states, for **S3 `bob_damping`**:

> *"**B6 gives joint stiffness** → the stance chain flexes under load and the reduction emerges;
> `bob_damping` collapses to 1.0"*

**This pass finds that claim false, and the mechanism is simple: stiffness is an ELASTIC MODULUS,
and density is not one.** Two materials of identical density can differ in modulus by orders of
magnitude. `MaterialProps` carries **no modulus axis**, this pass does not propose one, and
nothing else in the corpus does either. B6 gives the **load** the stance chain carries; it does
not give what the chain *does* under that load.

**So `bob_damping` (S3) is NOT absorbed by B6 as specified** — narrowed at best. Its real heir is
a **mechanical-property axis on the material sheet** (modulus, yield) plus a compliant joint
model, neither of which exists or is designed. Drafted as a `stubs.md` entry (§ 11), and **the
gait audit owes a banner** in the same commit as any ruling on this (immutable body, mutable
header; the obligation lands on the writer of the correction).

*Not a `⚠ CONTESTS`: the absorption table is **[assistant-proposed]** — `posture-gait.md`
§ Provenance marks §§ 4–6's mechanisms as assistant-originated — so this refines an assistant
hypothesis, which the "cautiously ratified" reading explicitly invites. Were it user-originated
it would be a plea and this pass would stop.*

**The same correction reaches `stubs.md` #49** (`a-limit-that-cannot-know-soft-tissue`, B7's), which
names **B6** as its heir. A ligamentous end-range is a *force* produced by a *compliant tissue*;
B6 gives mass. **#49's honest heir is the same modulus axis as S3's**, not the mass integral.

### 9.3 ⚠ The flight phase may be over-attributed to B6

`stubs.md` #39 says a run's flight phase *"is ballistics — which wants **mass**, not just
volume"*, naming B6 as heir. **The ballistic arc of a centre of mass is mass-INDEPENDENT** — it is
`y(t) = y₀ + v₀t − ½gt²`, and `m` cancels out of `mg = ma`. What genuinely needs force is the
**take-off impulse** that sets `v₀`, which is actuation. So B6 does not close #39 either: it
supplies the inertia the impulse would act on and nothing that generates the impulse. Reported as
a finding about that entry, not resolved here.

### 9.4 Precisely which `GaitKnobs` remain stand-ins after B6 lands

| knob | today | after B6-as-proposed |
|---|---|---|
| **S1 `cadence_scale`** | wholly assumed, identity 1.0 | **HALF DERIVED.** The inertia denominator becomes real (`√(gMd/I)` from `subtree_inertia`); the **muscle-power numerator** does not exist. **Still a stand-in**, narrowed. *And note the shape of the answer: for a uniform body the derived scale is exactly **1.0** (§ 3.2), so B6 closes the mechanism without moving the shipped number — the right outcome and a terrible acceptance test* |
| **S2 `duty_exponent`** | a regression between two published anchors | **CANDIDATE, not closed.** *Mechanical* external work (Cavagna) is computable from masses + kinematics and could derive β(Fr); *metabolic* cost needs a muscle efficiency we do not model. Untested — a direction, claimed as nothing |
| **S3 `bob_damping`** | identity 1.0 (the honest upper bound); the biped derives 6.58 cm against a published 4.6 cm | **NOT ABSORBED — § 9.2.** Needs an elastic modulus. **Still a full stand-in** |
| **S4 `swing_flexion`** | identity 0.0 (compass, near-straight swing) | **THE ONE B6 PLAUSIBLY CLOSES.** A ballistic swing of a limb with its real inertia is a double pendulum; given masses and gravity the free-swing solution is computable with **no actuation at all** (Mochon & McMahon's ballistic-walking result). The driven part still wants a hip torque |
| `foot_clearance_ratio`, `transition_fr`, `arm_swing_amplitude` | **TASTE** (kinds 1–2) | untouched — these have no true value and never get an heir |
| `stubs.md` **#43** `swing_gain` | stop-transition stand-in | **untouched.** Its heir is the stop transition, which `posture-gait.md` § 4 explicitly does not design |
| `stubs.md` **#39** flight phase | assumed | **NOT closed — § 9.3** |
| `stubs.md` **#49** soft-tissue end-range | derived limits over-predict 3–7× where ligamentous | **NOT closed — § 9.2.** Heir re-attributed to a modulus axis |

**Net: of the eight named inbound heirs, B6-as-proposed closes ZERO outright, half-closes two
(S1, S4), and leaves five needing an actuation or elasticity model that nothing designs.** The
one it closes *completely* is the one nobody listed: **`stubs.md` #40's own mass proxy** — the CoM
stops being a volume centroid and becomes a real centre of mass, and every derived quantity that
reads it inherits that honestly.

### 9.5 ⚠ WHAT THE ACCEPTANCE CRITERION IS *NOT*

Stated so a build cannot chase a famous nearby metric that another slice owns.

- **It cannot produce push-off flexion.** The push-off half of `swing_flexion` is pure actuation;
  a mass integral has no term that can generate it, at any density.
- **It cannot close all four `GaitKnobs`.** § 9.4 is the precise ledger: S1 half, S4 half, S2 a
  candidate, S3 not at all.
- **It cannot be validated by "the bob looks better."** The bob's amplitude has a mass term and a
  **kinematic** term — the determinants of gait: stance-knee flexion, pelvic list, ankle rocker —
  and **only the mass term is here**. A walk verdict on the bob is measuring S3, which § 9.2 says
  B6 does not own.
- **It must not be accepted on a number tuned until it looked right.** The only external evidence
  available is the whole-body **density band** (§ 5.3). A constant derived so a measured quantity
  lands in a published band is evidence; one tuned until an output looks right is a number
  pretending to be a mechanism.
- **It must not be validated on whole-body MASS.** § 5.3: the biped is 2.43× a human by volume,
  so its mass is *supposed* to be wrong by that factor.
- **It cannot be validated by "the goldens moved."** They must not (P7).
- **It cannot be validated by "the gait changed" on the shipped bipeds.** § 9.1: it will change by
  ~0.25 %, and that is the correct answer for near-uniform bodies.

---

## 10. Every falsifiable prediction, with the arithmetic

*Derived from source literals by closed-form arithmetic; no cargo run. **A divergence at build is
a finding about THIS document** (immutable body — the banner obligation falls on whoever
measures).*

**Geometry source:** `crates/dc-api/src/bodies/default_pack.rs:135-218` (`biped_plan`).
**Convention:** `offset_from_root` excludes the root's own pivot (`bake.rs:172-202`); root height
= chain reach = 0.45 + 0.43 = **0.880 m**.

| id | prediction | value |
|---|---|---|
| **P1** | shipped biped total segment volume, and the share table | **0.168388000 m³** — trunk 41.57 %, head 13.04 %, each `leg_*_upper` 9.62 %, each `leg_*_lower` 7.35 %, each `arm_*_upper` 3.01 %, each `arm_*_lower` 2.01 %, neck 1.40 % |
| **P2** | whole-body density of the four candidate mixtures (§ 5.2) | all-muscle **1060.000** · muscle .72 / fat .28 **1020.800** · **muscle .81 / bone .08 / void .11 → 1010.735** · muscle .78 / fat .06 / bone .08 / void .08 **1034.098** kg/m³ |
| **P3** | shipped biped mass at that mixture, uniform | **170.2 kg** (170.072 at exactly 1010.0) — **2.43×** a 70 kg human's ≈ 0.069 m³. *The number that must NOT be used as a check* |
| **P4** | biped leg about the hip, **uniform** density | `M/ρ = 0.028584 m³` · `d = 0.415630 m` · `I/ρ = 0.006848268 m⁵` · **ω = 4.125332 rad/s**, `T = 1.523074 s` — **identical at ρ = 1, 600 and 1010** |
| **P5** | the same leg, heterogeneous | thigh 1050 / shank 1090 → **ω = 4.114713** (−0.257 %) · thigh 1060 / shank 1900 → **ω = 3.985832** (−3.38 %) |
| **P6** | whole-body CoM, root-relative y (add 0.880 for the bake's ground frame ⇒ **`com_m[1]` = 0.968187 m today**) | uniform → **+0.088187 m** · Dempster-like per segment → +0.088071 (**−0.116 mm**) · lungs-in-trunk 800 / rest 1100 → +0.067496 (**−20.7 mm**) |
| **P7** | a 4-material append with no class membership and empty release rows moves **no world golden** | *a prediction, not a guarantee* |
| **P8** | radial redistribution's extreme influence on the leg's hip inertia | movable z-term **0.000087437** (1.277 % of I) ⇒ `I` ∈ **[−1.277 %, +2.554 %]** ⇒ ω ∈ **[−1.253 %, +0.645 %]** |
| **P9** | the facet split's blast radius | **69** read sites of the granular+geologic axes across 14 files; **6** of `density_kg_m3` across 4 |
| **P10** | registry headroom after four tissues | `MATERIAL_COUNT` 26 → **30** against a compile-time ceiling of **51** ⇒ **21 slots left** |
| **P11** | client-side consequence | `PALETTE_LEN` / `ATLAS_LAYER_COUNT` 30 → **34**; `terrain_material.rs:388`'s `assert_eq!(PALETTE_LEN, 30)` **fails**, and `terrain_fullbright.wgsl:23`'s `array<vec4<f32>, 30>` must move with it or the fullbright pipeline fails at GPU build |
| **P12** | derived free-swing period of the biped leg against the published human figure | **1.523 s** derived against a recalled **~1.2–1.4 s** for a human leg (Mochon & McMahon, *Ballistic walking*) — **~10–25 % long**, with a predicted cause: our shank is proportionally heavier than a real one (block thigh : shank volume **1.31** against a human's ≈ 1.8 mass ratio), i.e. **a caricature body is distal-heavy**. ⚠ NOT network-verified |

*P12 is the row to watch: the only one where the derivation can be checked against a published
number **and** where a divergence would say something about the bodies rather than about the
arithmetic.*

---

## 11. Proposed `stubs.md` entries — DRAFTED, UNNUMBERED [the integrator assigns]

Appended to `docs/design/stubs.md` under a literal `<!-- STUBS: unnumbered -->` block, each headed
`### NN.` as a placeholder. **No existing entry was renumbered, reordered or edited.**

| slug | what it fakes | heir |
|---|---|---|
| `an-inertia-with-no-actuation` | the integral supplies the **inertia** half of every force-shaped hole and none of the **power** half | an actuation model (joint torque / muscle force), which nothing designs |
| `the-tissue-materials-are-test-stand-ins` | four densities so an integral is not integrating the number 1; **not a roster, not a taxonomy** | a ratified tissue vocabulary — a **user call**, gated on bio/eco |
| `a-composition-whose-radial-order-nothing-reads` | the layer list is ordered per the user's design; the mass integral is order-invariant, and the order is worth **≤ 2.6 %** of a limb's swing inertia | harvest depth · damage penetration · buoyancy's shell term |
| `density-is-not-stiffness` | B6 supplies mass and **not** an elastic modulus, so `bob_damping` (S3) and B7's #49 soft-tissue end-range are **not** absorbed as their entries claim | a mechanical-property axis on the material sheet + a compliant joint model |

*Also owed rather than new:* `stubs.md` **#40** should record that B6 closes **it** while only
half-closing two of the eight named inbound consumers; **#39** and **#49** should record § 9.3's
and § 9.2's re-attribution; and the gait audit owes a banner for § 9.2.

---

## 12. Compliance

**North star.** Bodies are a named core **data-model API**; the **integral is an engine
primitive** (dc-api, pure, callable from any clock — the field-kernel argument: only the kernel
knows its own bound), the **body and its composition are pack content**, the **result is derived
data**. The declaration is plain data plus opaque strings (crossing constraint, Deviation 1's
cheap-insurance shape). **No capability tiering** (Deviation 2): a mod declares a composition
exactly as the default pack does, and the facet split makes a third-party material declare *what
it is* through the same door. **Nothing here is justified by trust anywhere.**

**S-3.** The plan stays the authority; `MassProperties` is a **sibling**, returned, memoized by
consumers, never a field. Acceptance test 4 pins the single authority — the exact failure that
produced `root_bob_m` twice.

**S-5.** `composition: None` is an exact identity default; test 1 asserts byte-identity and test 2
asserts the stronger **invariance**. The `segment_densities` seam has an identity of 1.0 and names
its heir — the shape 31 live seams already use. S-5's sharp edge is honoured: an **unknown**
material is refused with a receipt (test 6), never silently defaulted.

**S-9.** Untouched and not foreclosed: composition is **per species** (prior 1.1), and an instance
overlay would be a sparse delta over the same `Vec<CompLayer>`, f64 throughout. Nothing here
quantizes a composition, so `posture-gait.md` § 6 property 3's *"a 3 % limp is inexpressible"*
hazard is not reintroduced.

**A-1 ledger** (each simplification, its identity, its heir): mass = volume × 1 → declared
composition (**#40**, closed by this) · composition read as shares only → the radial readers ·
four test materials → a ratified vocabulary · inertia without actuation → an actuation model ·
stiffness absent → a modulus axis. *And § 0a is the guard against this pass minting a fifth: the
test materials are born marked, structurally (§ 5.5), because A-1 is what happens when a stand-in
is only marked in prose.*

**A-4.** § 6.3 refuses to land `composition` in a slice where no material can fill it, and refuses
to land `subtree_inertia` in a slice with no caller. Every proposed function has a named day-one
consumer.

**A-7 — checked specifically, per the brief.** **No `MaterialId` constant appears in any body
module under this proposal.** Composition names materials by **declared string**, resolved through
`MaterialId::from_qualified_name` (`materials/mod.rs:178-183`) — a lookup of a name the *content*
supplied, the exact inverse of a name the *process* supplied. The integral itself never sees a
material: it takes `&[f64]`. And the A-7 diagnostic was applied where it was tempting — *what
property of `dc:tissue/bone` is the integral reaching for?* **Density.** So density is what it
takes, and the special case dissolves for every future member of the family, including ones no
pack of ours will ever ship.

**⚠ CONTESTS flags: NONE.** Checked before filing.

- The `ideas.md` mixed-materials entry is **implemented, not narrowed** (§ 3.1): its ordering is
  carried in the data, and what § 3.3 defers is a **reader**, stated as a scoping choice with a
  measured bound, with the entry preserved as the named heir. A bare `density: f64` was considered
  and **rejected** on exactly prior 1.7's ground.
- §§ 9.2 and 9.3 refine **[assistant-proposed]** claims in a prior audit and two stubs, which the
  "cautiously ratified" reading explicitly invites. Neither is user-originated.
- § 6's option D is named and rejected **because** it contests a DECIDED ruling (block IS
  material) — naming it is the point.
- The user's § 0a caveat is treated as binding data throughout and is nowhere re-argued.
- `bodies.md`, `ROADMAP.md` and `docs/dependency-graph.md` were **not touched**.

---

## 13. What I could not determine

*"I cannot answer this" is a first-class result.*

- **Whether four test materials is the number the user wants.** § 5.2 shows three span the range
  and four match the user's own radial sketch. Which ships is a ratification, not a measurement.
- **Whether the `MaterialProps` facet split should happen at all**, or whether the user prefers to
  live with a total sheet and accept option A's cost. § 6 recommends C with a measured cost; the
  call is the user's, and it is a **materials-system** call that outlives bodies entirely.
- **Whether "radially ordered" means concentric shells around the long axis** — this pass's
  reading, on which § 3.3's bound rests — **or something else**: layers along the segment, or a
  nesting order for harvest. The bound holds for the first reading; I **cannot attribute the
  intent**, and guessing at it is what corrections #65 forbids. *This is prior 1.8's question,
  answered for one reading and left open as to which reading is meant.*
- **The recalled literature densities and the ~1.2–1.4 s human leg period.** No network. Flagged
  at every use.
- **Whether the whole-body density target is the ~1010 "with lungs" figure or the 1040–1070
  hydrostatic one** (§ 5.3). They measure different things and the sources should settle it.
- **Whether B6-a's `subtree_inertia` has a day-one consumer.** It does **iff** the same slice
  derives `cadence_scale` from it. Whether that is wanted in the first slice is a sequencing call
  I can frame but not make.
- **Whether a tissue material should be a placeable block** (§ 8 row 11). It falls out of the
  one-namespace ruling; whether the dev-surface completion should filter it is a UX question
  nobody has asked.
- **The facet split's cost in wall-clock.** 69 sites is a count, not a duration, and no build was
  run.
- **Whether density belongs in `MaterialProps` at all under option C**, or whether even *density*
  is a facet (a `Void` material's "density" is the fluid's, not its own). I put it in the core on
  the argument that every material that occupies volume has one; I cannot rule out a body that
  makes that wrong.

---

## 14. Open questions, triaged

**USER CALLS (ratification, not measurement):**

1. **Q1 — the `MaterialProps` totality answer** (§ 6): the recommended **facet split** (C), as its
   own materials-system slice before B6 adds anything; versus filling every field (A); versus a
   separate tissue registry (D, which contests a DECIDED ruling).
2. **Q2 — the test set** (§ 5): four (`muscle`, `fat`, `bone`, `void`) or the spanning three (drop
   `fat`)? And the `dc:tissue/*` namespace.
3. **Q3 — the sequencing** (§ 6.3): B6-a (integral only, byte-identical, startable today) → B6-b
   (facet split) → B6-c (materials + `composition`)? Or hold all three for one landing?
4. **Q4 — radial ordering's disposition** (§ 3.3): carry the order, consume the shares, name the
   readers, record the ≤ 2.6 % bound — or is a radial-inertia term wanted in scope now?
5. **Q5 — the four drafted `stubs.md` entries** (§ 11), unnumbered; the integrator assigns.
6. **Q6 — `bone` / `potsherd` / `knapping-debris` / `ash`.** Deliberately **not** touched here
   (§ 5.4). Their disposal is its own decision and this pass recommends it stay one.

**ENGINEERING (decide with a number, no ratification owed):**

7. Verify § 10's twelve predictions at build; a divergence is a finding about this document.
8. Verify the § 5.1 densities and P12's period against the cited sources — **no network here**.
9. Confirm P7 (no golden moves) by running the world goldens after the append.
10. Decide whether to generate the 12 placeholder textures (§ 8 row 13) or accept 12 startup
    warnings on a dev build.
11. Whether `share` is `f64` or a fixed-point fraction. **f64** recommended —
    `posture-gait.md` § 6 property 3's over-quantization hazard applies to any per-body number a
    delta may later ride on.
12. Whether `subtree_inertia` takes an `Axis` (B7's `X`/`Y`/`Z`, the only axes an Euler pose can
    express) or a full inertia tensor. **Axis** recommended for the same reason B7 gave: the DOF
    vocabulary can only be as rich as the pose representation.
