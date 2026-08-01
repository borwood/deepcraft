# What a body plan must DECLARE — a design pass against five bodies

**Status: DESIGN PASS. Nothing here is ratified and no code was changed.** The deliverable is
this document. It proposes a shape for `dc_api::bodies::{BodyPlan, SegmentDef}`; proposing a
change and making one are different acts, and the second is the user's call.

**The question.** *What must a body plan declare, for the engine to derive posture, gait,
colliders and damage from it — across body kinds we have not built?*

**The acceptance criterion, and it is the whole point.** All three plans in the tree are
bipeds (`dc:body/{biped,stout,longleg}`). Any rule derived from them works perfectly and
proves nothing — the exact failure of journal/0130 (*"retargets across differing
proportions"*, quantified over a set with one element), of corrections **#78** (a sweep that
held posture at its default), and of **#77** (an instrument that could not see the axis). So
every proposal below is gamed against **biped · quadruped · snake · bird · tree**, and § 3
says what each one breaks. **A proposal that only works for the first two is a finding, not a
failure**, and is labelled as one.

**Read with:** `posture-gait.md` (CAUTIOUSLY RATIFIED 2026-08-01 — §§ 2–6 are ratified bones,
and this pass is *expected* to report bones found missing machinery; § 6 below does that
loudly) · `bodies.md` (the data model; note DECIDED vs PROPOSED and the § IK / § stepped
animation banners) · `ideas.md` § *The segment tree as one primitive* (**sketch, not
decision**) · `north-star.md` § Refinement (bodies are a named core **data-model API**) and
§ Deviations 2 (**no capability tiering**) · `spines.md` S-3 S-5 S-6 S-9, A-1 A-4 A-7 ·
`dependency-graph.md` § 2b · journal/0130–0132 · corrections **#77 #78 #79 #80**.

**What this pass does NOT do.** It does not resolve the open user call in `bodies.md` § IK
(the 20 mm hip/reach gap; whether the four absolute-metre constants become ratios). § 5.7
says how the proposal *interacts* with it. **It is not resolved and may not be marked so.**

---

## 1. The proposed declaration

The shape, as plain data. New fields are marked `// NEW`; nothing existing changes type.

```rust
pub struct BodyPlan {
    pub name: String,
    pub doc: String,
    pub segments: Vec<SegmentDef>,
    pub slots: Vec<AnimSlot>,          // unchanged — but see § 6.H

    // NEW — the structure a bake needs and the plan cannot express today
    pub contacts: Vec<ContactDef>,     // where this body meets the substrate
    pub limbs:    Vec<LimbDef>,        // which chains are limbs, and what for
    pub modes:    Vec<ModeDef>,        // locomotor modes; which contacts cycle in each
}

pub struct SegmentDef {
    pub name:     String,   // NOW AN ADDRESS with a grammar: `trunk`, `arm.l/upper`,
                            // `trunk/branch[2]/leaflet[3]`. A flat name is the
                            // degenerate case. THE TYPE DOES NOT CHANGE. (§ 5.3)
    pub parent:   Option<String>,
    pub pivot_m:  [f64; 3],
    pub size_m:   [f64; 3],
    pub offset_m: [f64; 3],
    pub tint:     [f32; 3],
    pub collide:  bool,     // NEW — participates in the derived terrain-collider set.
                            // Identity default `true` reproduces today exactly. (§ 5.5)
    // DEFERRED, heirs named: `kind` (Box/Card — § 5.5) and `composition`
    // (the per-segment material MIXTURE — § 5.6). Neither is needed by any body
    // in § 3 except the tree, and neither has data to carry today.
}

pub struct ContactDef {
    pub segment:      String,      // address of the segment that touches
    pub local_m:      [f64; 3],    // the point ON that segment that meets the substrate
    pub bears_weight: bool,        // false = incidental (a dragged tail, a trailing knuckle)
}

pub struct LimbDef {
    pub tip:     String,           // address of the DISTAL segment; the chain is DERIVED
                                   // by walking parents to the first branch point (§ 5.2)
    pub purpose: LimbPurpose,      // Stance | Manipulate | Flight | Fin | Inert
}

pub struct ModeDef {
    pub mode:     Locomotion,      // Ground | Air | Water | Static
    pub contacts: Vec<String>,     // which contact segments cycle in THIS mode
}
```

**Four properties of this shape, stated because each one is load-bearing:**

1. **Everything is plain data + strings.** No naked references, no generics, no ids that mean
   anything to the engine. It survives the crossing constraint unchanged (north-star
   Deviation 1's *"cheap insurance"* shape).
2. **Laterality, fore/hind and pairing are NOT declared. They are derived from contact
   geometry** (§ 5.2). This is the single strongest result in the pass and it is what kills the
   `leg_l_*` / `leg_r_*` naming coupling.
3. **Repetition and symmetry are NOT in this struct.** They belong to a *producer* of a
   `BodyPlan`, which stays flat (§ 5.4). That is the only answer that survives the tree.
4. **Every new field has an identity default** that reproduces today's behaviour byte for
   byte: `contacts`/`limbs`/`modes` empty (fall back to the current `leg_rigs` derivation),
   `collide: true`. S-5.

---

## 2. What the engine derives from it

| output | derived from | needs which new field |
|---|---|---|
| **hip height / root offset** | contact points + the pivot chain | `contacts` |
| **resting joint angles** | contacts + segment volumes (mass proxy) | `contacts` |
| **base of support** | the convex hull of weight-bearing contacts, projected | `contacts.bears_weight` |
| **limb chains + bone lengths** | walk parents from `limbs[].tip` to the branch point | `limbs` (or contacts, for stance limbs) |
| **left/right, fore/hind, pairs** | the (x, z) of each limb's attachment, in body frame | *nothing* — derived (§ 5.2) |
| **gait phase assignment** | the sign/rank pattern of those (x, z) | *nothing* — derived |
| **which limbs participate in a cycle** | the mode | `modes` |
| **terrain-collider box set** | segments with `collide: true` | `collide` |
| **damage hitboxes** | every segment, always | *nothing* |
| **mass integral** | Σ volume × density | `composition` (**deferred; no data — § 5.6**) |

---

## 3. The five-body table — the centre of gravity

**Legend:** ✅ expressed · ⚠️ expressed with a named gap · ❌ breaks.

| | biped | quadruped | snake | bird | tree |
|---|---|---|---|---|---|
| **contacts** | ✅ 2 soles | ✅ 4 | ✅ N ventral (via the producer) | ✅ 2 feet | ⚠️ 1, at the base — but the physics is *anchored cantilever*, not balance |
| **limbs** | ✅ 2 Stance + 2 Manipulate | ✅ 4 Stance | ✅ **empty list, no degenerate default needed** | ✅ 2 Stance + 2 Flight | ✅ empty |
| **laterality / fore-hind** | ✅ derived (x = ±0.14, one girdle) | ✅ derived (2 z-ranks × 2 x-signs = the four roles, unnamed) | ✅ n/a — phase is a function of contact *index* | ✅ derived | ✅ n/a |
| **gait** | ✅ | ✅ trot/pace/bound are sign patterns | ⚠️ **only if the triple is per-CONTACT** (§ 6.A) | ❌ **two cycles** — wingbeat ≠ step (§ 6.B) | ✅ none |
| **posture bake** | ✅ | ✅ | ❌ **COM-over-base is vacuous** for a 1-D contact set (§ 6.C) | ⚠️ digitigrade: *"legs straight"* is the wrong rest pose (§ 6.J) | ❌ **statics, not balance** |
| **colliders** | ✅ 1 box | ✅ ~2 | ❌ **box set varies with PHASE**, not just yaw (§ 6.D) | ⚠️ needs `collide:false` on wings or the wingspan blocks a door (§ 6.E) | ✅ ~1 (rotationally symmetric; yaw buckets degenerate) |
| **damage** | ✅ | ✅ | ✅ | ✅ | ✅ per segment |
| **addresses** | ✅ flat names, unchanged | ✅ | ✅ | ✅ | ✅ **this is the case that requires them** |
| **repetition** | ✅ mirror | ✅ mirror | ✅ serial repeat | ✅ mirror | ❌ **recursive production — a generator, not a repeat** (§ 5.4) |
| **segment kinds** | ✅ all Box | ✅ | ✅ | ⚠️ feathers would want Card | ❌ **foliage needs Card** (§ 5.5) |
| **registers at all** | ✅ | ✅ | ✅ (binds a slither clip to `walk` — the *name* is a lie) | ❌ **`KNOWN_VERBS` has no `fly`** | ❌ **`REQUIRED_VERBS` demands `walk`** |

### What specifically breaks, per body

**Biped** — nothing. `leg_rigs`' `starts_with("leg_")` coupling (`dc-client/src/body.rs:373`,
flagged in its own doc comment at `:365`) is retired by declared contacts.

**Quadruped** — one real gap: **the spine**. The plan can hold spine segments, but nothing
says *this chain flexes with the gait*. The gait bake would produce a rigid-trunk quadruped,
which is correct for a walk and a trot and wrong for a bound or a gallop. Not fatal; naming it
now costs nothing and rediscovering it costs a slice. A tail is `purpose: Inert`,
`bears_weight: false` — expressed. **A kangaroo earns `ModeDef` from an animal rather than
from a bird:** its tail bears weight in the tripod stance and not in the hop, so contact
participation is genuinely **mode-dependent**.

**Snake** — the case that justifies making contact a *first-class declaration* rather than a
property of a limb. There are no limbs; `limbs` is empty and needs no degenerate member.
Two things break:
- **Posture.** `posture-gait.md` § 5's definition ("COM over the base of support at tolerable
  effort") is *satisfied everywhere* for a 1-D contact set, so the solve is vacuous and the
  interesting quantity — the resting coil — is a curve, not the minimiser of an effort
  function over a balance constraint.
- **Gait cardinality.** § 4's triple is stated **per limb**. A snake has zero. It survives
  only if the triple is **per contact** and phase may be a linear function of contact index —
  which is precisely what a travelling wave is. That reading strictly generalises the limbed
  case (a limb has one contact) and is identical on bipeds and quadrupeds. See § 6.A.

**Bird** — the loudest structural break in the set.
- **Two cycles.** A wingbeat and a step are different frequencies. § 4 assumes one cycle with
  per-limb offsets within it. `ModeDef` is what fixes it; **without a mode layer the bird is
  not a tuning problem, it is a re-architecture.**
- **The verb vocabulary rejects it.** `KNOWN_VERBS = ["idle","walk","jump"]`
  (`crates/dc-api/src/bodies.rs:81`) and `verb_is_known` (`:254-256`) is consulted at
  `validate_plan` (`:337-342`). A bird cannot bind `fly`. This is a hard define-time rejection,
  not a cosmetic gap — and it collides with **north-star Deviation 2** (§ 6.H).
- **Colliders.** § 5's *"grow k until the boxes stop badly over-covering"* over a spread-wing
  bird yields a wingspan-wide box set that would block a doorway. `collide: false` on the
  flight limbs is the minimum fix.
- **Rest pose.** A bird's resting leg is deeply folded and held by tendon; the *"legs straight,
  hip at leg reach"* answer that ROADMAP correctly names as **the right answer, not a
  degenerate one** for a symmetric biped is simply the wrong answer here. The effort term
  matters sooner than `posture-gait.md` § 8 assumes (§ 6.J).

**Tree** — included precisely because it is where a name-keyed design dies, and it does.
- **Confirmed: name-keying passes 1–4 and dies here.** `character.rs:267`'s
  `if s.name == "neck"` and `:424`'s `if s.name == "head"` are plausible for all four animals
  and meaningless for a generated tree — and they fail **silently**, doing nothing at all,
  which is the worst available failure mode.
- **Addresses carry it.** `trunk/branch[2]/leaflet[3]` is a `String`; nothing downstream
  changes (§ 5.3).
- **`validate_plan` is O(n²)** at `:278` (uniqueness), `:305-314` (parent presence) and
  `:315-334` (cycle walk), all by linear string scan. At n=300 that is ~45 k compares; at
  n=5000 it is ~12.5 M. **It does not break** — this is *define time*, and gen time is free
  (the two clocks). Worth saying so nobody "optimises" it under a runtime argument.
- **The client's per-frame loop does not break either, for a reason worth naming:** plants do
  not articulate (`ideas.md`: *"wind is a shader"*), so a tree never enters
  `character.rs:244-280`'s per-segment hash lookup. The runtime hazard is real and is *not*
  triggered by the case that looks like it would trigger it.
- **It is rejected at define time.** `REQUIRED_VERBS = ["idle","walk"]` (`bodies.rs:88`,
  enforced at `:366-373`). A tree cannot walk. One line, hard failure.
- **The bake refuses it, and that is the right outcome.** A tree's support is an anchored
  cantilever; posture is a statics problem, not a balance problem. The honest disposition is
  **not** to invent `SupportKind::Anchor` today (the vegetation sketch is unratified and
  explicitly *"nothing above may be built without its own ratification"*) but to have the bake
  **declare its own domain and refuse** bodies outside it. That costs nothing and does not
  foreclose.
- **`Card` is a tree-only requirement.** No animal in the set needs it (§ 5.5).

---

## 4. Where the current shape already agrees with the bones — measured, free

Two claims in `posture-gait.md` can be checked against data already in the tree, and both come
back **confirmed**. This costs nothing and is the kind of check corrections #78 says to do
before trusting a derivation.

**§ 4's phase-offset parameterisation.** Read `dc:anim/biped_walk` (`bodies.rs:811-878`)
across its four keys:

| | t=0.00 | t=0.25 | t=0.50 | t=0.75 |
|---|---|---|---|---|
| `leg_l_upper` | 0.6 | 0.05 | −0.5 | −0.05 |
| `leg_r_upper` | −0.5 | −0.05 | 0.6 | 0.05 |
| `leg_l_lower` | −0.15 | −0.05 | 0.5 | 0.2 |
| `leg_r_lower` | 0.5 | 0.2 | −0.15 | −0.05 |

`leg_r(t) = leg_l(t + 0.5)` **exactly, at every key, on both bones.** And
`arm_r(t) = mirror_z(arm_l(t + 0.5))`. The shipped clip *is* one half-cycle of one limb plus a
phase offset plus a Z-mirror — hand-typed four times. **§ 4's claim that phase offset replaces
mirroring is not a hypothesis here; it is a description of the data that already exists.**

**§ 1's leaked-requirement claim about `root_bob_m`.** The authored bob runs
`0.0, 0.04, 0.0, 0.04, 0.0` — **period 0.5 s against a 1.0 s limb cycle, exactly 2×**. That is
precisely what two alternating stance legs of fixed length produce. The keyframes are a
hand-typed reproduction of an output. Confirmed.

---

## 5. The axes, worked

### 5.1 Roles and contact — declared, not derived

The candidate derivation is *"the lowest leaf of a descending chain"*. It works on the biped
and dies four times:

- **Quadruped:** a kangaroo's tail is a contact and a dog's is not. Both are lowest leaves of
  descending chains. Derivation cannot tell them apart.
- **Snake:** there is no descending chain. Derivation returns the tail tip; the truth is a
  *distributed set* along the ventral line whose participating subset depends on the gait.
- **Bird:** a wingtip is a leaf; it must never be a contact. A perch is a contact that is not
  the ground.
- **Tree:** the contact is the **root of the tree, not a leaf.** Derivation returns the tips of
  the lowest branches — exactly inverted. And a banyan's aerial roots *are* leaves of descending
  chains, so derivation would accidentally succeed there **for the wrong reason**, which is worse.

So: **declared.** And not as a boolean on a segment — as a **point in segment-local
coordinates**, because the sole is the bottom face of the lower-leg box, not its centroid.

**A consequence worth naming:** once contact is declared, **the root origin no longer has to be
"the feet"** (`bodies.rs:405`'s convention). `trunk.pivot_m[1] = 0.9` stops being read as hip
height and becomes an arbitrary rest origin, with the ground offset derived. That is the concrete
mechanism by which the 0.900 m constant stops being authored data.

### 5.2 Limb structure — the geometry already knows

**Do not declare laterality, fore/hind, or pairing.** Derive them:

1. Accumulate each limb's **attachment point** down the pivot chain from the root. This is
   static plan geometry — available *before* any solve, so there is no circularity with the
   posture bake, which is the trap this derivation looks like it should fall into.
2. In the body frame (`+Y` up, `−Z` forward — `bodies.rs:405`): `lateral = sign(x)`,
   `longitudinal rank = rank of z`.
3. **Pairs** are equal `|x|`, opposite sign, same z-rank. **Diagonal pairs** are opposite sign
   in *both*.
4. Gait taxonomy then falls out as sign patterns: **trot** = diagonal pairs in phase; **pace** =
   lateral pairs; **bound** = the fore rank then the hind rank. No limb is ever named.

Checked: biped (x = ±0.14, one z-rank → the only available assignment is anti-phase ✅);
quadruped (two z-ranks × two x-signs = the four roles, unnamed ✅); snake (no limbs; phase is a
function of contact *index* along the chain ✅); bird (✅ for the legs, and see § 6.B); tree (n/a ✅).

**What must be declared is `purpose`** — a wing is a limb that is not for ground locomotion, and
no geometry tells you that. And **the chain itself is derived**: walk parents from `tip` to the
first segment with ≥ 2 children. For the biped that yields `[leg_*_upper, leg_*_lower]`, which is
exactly what `leg_rigs` computes from a string prefix today.

### 5.3 Addresses vs names — and the answer is that nothing breaks

**Proposal: keep `name: String` and give it a grammar.** `trunk`, `arm.l/upper`,
`trunk/branch[2]/leaflet[3]`. A flat name is the degenerate case.

**This is a zero-break change**, and that is the finding. Full inventory in § 7. The reason is
that every consumer treats the name as an **opaque equality key** — uniqueness, parent lookup,
cycle walk, clip joint membership, the client's `HashMap<String, Entity>`. A path is a `String`.

Three things would break, and all three should:

- **`leg_rigs`** (`dc-client/src/body.rs:367-395`): `starts_with("leg_")`, `ends_with("_upper")`,
  `name.replace("_upper","_lower")`. The one true naming coupling, already flagged in its own doc
  comment. Retired by declared contacts/limbs. **A-7** (a content identity named inside a process).
- **`character.rs:267`** `if s.name == "neck"` — needs a declaration (a `look_joint` address, or a
  limb purpose).
- **`character.rs:424`** `if s.name == "head"` — placeholder face art.

**What would break the wire, and therefore what NOT to do:** introducing a structured `SegAddr`
type, or a **pattern** binding in `JointRot.segment` (`bodies.rs:143-145`) so one clip row can
address `trunk/branch[*]`. **Neither is forced by any body in § 3** — the tree forces *address
generation and address-keyed queries* (harvest, collider, mass), not clip binding, because plants
do not articulate. Recommend both stay unbuilt with the requirement recorded.

### 5.4 Repetition and symmetry — one mechanism, and it is not in the struct

Three genuinely different kinds, and the brief is right that they are not one thing:

| kind | example | shape |
|---|---|---|
| **mirror** | a biped's arms | one subtree, reflected across a plane |
| **serial repeat** | a snake's 40 segments, vertebrae, centipede legs | one template × N along a chain, with a per-index transform |
| **recursive production** | a tree | a rule whose depth and branching are functions |

**Recommendation: all three live in a PRODUCER of a `BodyPlan`, and `BodyPlan` stays flat.**
The engine's data-model API is the flat plan; the producer is content-side (a pack function
today; a declarative `PlanRecipe` with its own define verb later, if ever). This satisfies the
crossing constraint, keeps the wire flat, and is the only answer the tree accepts —
`ideas.md` already says a body's generator *"is an explicit enumeration (which is exactly what
`biped_plan()` already is)"*.

**The objection, and its answer.** Flattening loses the fact that two limbs are mirror-partners
— which the gait bake would seem to need. It does not: **§ 5.2 derives phase assignment from
contact geometry, not from partnership.** So flattening is free.

**And this yields the single cheapest item on the board.** `biped_plan()`'s `arm_r_*` and
`leg_r_*` rows are *exactly* the `_l_` rows with `pivot_m[0]` negated (±0.33, ±0.14; every other
number identical — verified against `bodies.rs:437-500`). A mirror helper in the authoring
function produces the **byte-identical plan** — which is the seam-first acceptance test — and
makes the symmetry structural. **It fixes the defect that opened this whole arc with no schema
change, no wire change and no behaviour change.**

**The clip half is NOT fixable the same way**, and § 4 already says why: `idle`'s arms are a pure
spatial Z-mirror, but `walk`'s limbs are a *half-cycle phase offset* (§ 4 above). Mirroring the
clip data would encode the wrong relationship. **That half genuinely waits for the gait bake.**

### 5.5 Segment kinds — the doctrine holds, the members are premature

`ideas.md` proposes a closed machine-owned set `{Box, Card}`, governed exactly like
`material-behavior.md` § 2's forms. **The doctrine transfers and the test passes:** forms are
closed because *"the mesher, collider, gravity and water all must know how every mode behaves"*;
a segment kind is consumed by the renderer, the collider, the mass integral and damage. Same
four-consumer argument. A pack **picks** a kind and never invents one. ✅

**What a kind changes, concretely:**
- **`Box`:** volume = `x·y·z`; collider = the box; hitbox = the box.
- **`Card`:** volume is **not** `x·y·z` — a sheet's mass is `area × areal density`, so the mass
  integral must read `size_m` differently. It is **excluded from the collider set** (a leaf is not
  solid), and per the user's billboard ruling it is **not individually addressable**, so damage and
  harvest aggregate rather than resolve per segment.

**But `Card` is exercised by exactly one body in § 3, and it is the one we are not building.**
Recommend: **do not add `kind` today.** Record it as a named heir. Adding a one-variant enum is
not a seam, it is a fossil.

**The bird's finding is separate and IS needed by animals:** a wing is a `Box`, individually
addressable, damageable — and must not be in the terrain-collider set. That is
`collide: bool`, not `kind`. *This is adjacent to the user's billboard ruling and does not
supersede it: the ruling fuses geometry kind with interaction granularity at sub-representation
scale; `collide` is about a fully-addressable box that should not sweep terrain.*

### 5.6 Per-segment material / mass — **the main session's belief is half right, and the
half that fails is decisive**

**Confirmed, on the semantics.** It is additive; the identity default is today's volume proxy;
`Σ(volume × position)` becomes `Σ(volume × density × position)` with density ≡ 1 reproducing the
current answer byte for byte. It is independent of addresses ✅, of contacts and limbs ✅, and of
growth ✅ (density is intensive, so it survives uniform scaling).

**Refuted, on being startable today, for three independent reasons — any one is sufficient:**

1. **There is no data to put in it.** `MaterialProps` does carry `density_kg_m3`
   (`crates/dc-core/src/materials/mod.rs:288`), and the roster is **26 materials** — sand,
   gravel, snow, leaf-litter, clay, silt, potsherd, knapping-debris, ash, loam, scree, **bone**,
   mudstone, sandstone, granite, basalt, gold-dust, siltstone, conglomerate, diorite, andesite,
   olivine, peat, coal, carbonaceous-mudstone, charcoal. **`bone` is the only body-relevant
   member.** There is no flesh, muscle, fat, keratin, chitin or wood. And there is **no
   `define_material` command** — the roster is a compiled-in
   `const [MaterialProps; MATERIAL_COUNT]`. So per-segment materials is blocked on a
   **materials-roster** addition, not on the body plan. *This is a measurement, not an opinion.*
2. **Its only consumer is the mass integral in a bake that does not exist.** Landing it first is
   **A-4** — built machinery with no consumer, and it would arrive without an index row.
   ROADMAP's first slice already sequences it correctly, as the **heir** of the volume proxy.
3. **Its shape is a user call.** `ideas.md` records, user-originated: *"Mixed materials per
   segment, radially ordered."* Shipping a bare `density: f64` would **narrow that** —
   corrections **#65**'s exact failure mode. If it is built, it lands mixture-shaped
   (`Vec<(slug, parts)>`) or as an explicitly-annotated stand-in with the mixture as its named
   heir in `stubs.md`. ⚠ See § 9.

**Net:** the *design* can be settled in parallel. The *build* cannot start today, and the reason
is the empty roster, not the plan.

### 5.7 Growth — the user's own sketch already answers it; do not build a second axis

`posture-gait.md` § 3 keys the bake on `(species, posture, growth, yaw)`; § 8 notes that dropping
growth keeps the table small and is re-addable. **This pass supports dropping it, with a reason
rather than a convenience:**

- **Animals grow by scaling proportions**; plants by extending topology (`ideas.md`/`ecology.md`:
  *"scalars = allometric drift; bools = segment gain/loss"*). So for animals a growth stage is a
  **parameterisation of one plan** — which is exactly `bodies.md` § *Plan parameters*
  (**PROPOSED, user-originated, 2026-07-19**). **Do not invent a `growth_stages` field beside it.**
- **For UNIFORM scaling the bake needs no growth key at all.** Bake **angles and ratios**, not
  metres: resting joint angles are scale-invariant, lengths scale linearly, and cadence is a
  published closed form in `L` (`√(g/L)`) evaluated at sample time. The axis is needed only for
  **allometric** (non-uniform) change — which is precisely what allometric drift is, and which is
  not scheduled.
- **This converges with `bodies.md` § IK's units banner from the other direction:** *"every plan
  parameter must declare its UNITS (ratio-of-plan vs absolute-metres)."* **Bake ratios, not
  metres**, and the growth axis disappears for free.
- **For plants, growth is a generator per stage** — confirming § 5.4's bifurcation.

**How this interacts with the open user call (and does not resolve it).** If a derived hip height
lands, `trunk.pivot_m[1]` ceases to be read as authored hip height, and the *first* of the four
absolute-metre constants stops being data. **That does not answer the call.** The call is whether
the 20 mm gap was intentional and whether the remaining three — the clips' root bob, the
half-voxel foot window, `CROUCH_ROOT_DROP_M` — become ratios. It changes the question's shape and
**leaves it open. It is not resolved here and must not be recorded as such.**

---

## 6. What this pass found MISSING OR WRONG in `posture-gait.md`'s bones — loudly

*This is the "cautiously" in "cautiously ratified" working as designed. §§ 2–6 are ratified bones;
these are the places a body we have not built pushes back on them. Provenance is marked, because
the `(phase, amplitude, duty)` triple, the collider derivation and the damage resolution are
**assistant-proposed** (§ Provenance), and refining those is not contesting a user design.*

- **A. § 4's triple is PER LIMB; it must be PER CONTACT.** A limbless body has zero limbs and a
  gait. Per-contact strictly generalises — a limb has one contact — and is identical on every
  limbed body, so it is a free widening. *(assistant-proposed; refinable.)*
- **B. § 4 assumes ONE CYCLE. A bird has two.** A wingbeat and a step run at different
  frequencies over disjoint limb sets. This is not a parameter gap; without a locomotor-mode
  layer it is a re-architecture. **This is the loudest finding in the pass.**
- **C. § 5's definition of posture does not generalise.** *"COM over the base of support at
  tolerable effort"* is exactly right for a balancing animal, vacuous for a 1-D contact set
  (snake) and the wrong physics for an anchored one (tree, statics). The generalisation is:
  *the resting configuration satisfying the body's support constraint at minimum effort*, where
  the constraint's form follows the contact set's dimensionality. **⚠ CONTESTS (narrows) a
  user-originated statement** — see § 9.
- **D. § 5's collider bake key is short by one axis.** `(posture, growth, yaw)` misses **gait
  phase**. Any body whose silhouette changes materially through the cycle needs it — a snake, and
  **also a galloping horse**, so this is not a limbless exotic. Combined with bounded `k` and 8
  yaw buckets, the product is what sits in memory; § 8 already flags the product as the thing to
  watch and this adds a factor to it.
- **E. § 5's collider derivation over-covers a bird.** *"Grow `k` until the boxes stop badly
  over-covering"* on spread wings yields a wingspan-wide set. Needs per-segment collider
  participation (`collide: bool`).
- **F. Growth need not be a bake key at all** for uniform scaling — bake ratios (§ 5.7). This
  *supports* § 8's instinct and supplies the reason it was missing.
- **G. Nothing in §§ 2–6 says WHERE CONTACT IS DECLARED, and the entire tier rests on it.** § 1's
  inversion is literally *"reality pins the feet"* — and the bones never say how the engine learns
  which segments are feet. **This is the largest single hole**, and it is why § 8's sequencing
  puts the declaration immediately behind the first slice.
- **H. `bodies.md`'s verb→slot contract rejects a bird and a tree.** `KNOWN_VERBS` (`bodies.rs:81`)
  and `REQUIRED_VERBS` (`:88`) are a **closed engine-owned vocabulary of content identities**, and
  they fail the forms test that `Card`/`Box` passes: the machine does not currently reason about
  *walk* versus *fly* at all — verbs are clip-slot keys, and the client picks idle/walk by speed.
  A closed set with no machine justification is **A-7**. It also sits badly beside **north-star
  Deviation 2** (*"assume mods can do anything"*): a mod cannot author a body with a locomotion
  its engine's list omits. **⚠ CONTESTS a DECIDED item** — see § 9. *The define-time-checking
  principle is right and is not what is contested; the closed vocabulary and the mandatory-verb
  half are.*
- **I. § 5's "a biped → 1 box, exactly today's collider (the identity default)" is not an identity
  default for two of the three shipped plans.** Today's collider is `CharacterConfig`, which is
  **world-global** — a 1.60 m stout is hit as 1.8 m. Already known (dependency-graph **B4**);
  recorded here so the identity claim is not read as literal.
- **J. § 8 assumes the effort term can be the cheapest thing that gets hip height right.** True for
  a biped. A bird's resting leg is deeply folded and tendon-held; *"legs straight"* is not merely
  imprecise there, it is a different animal. **This does not affect the first slice** (whose bodies
  are all bipeds) but the assumption should not be carried forward as general.

---

## 7. Breaking-change inventory — addresses vs names

**Verdict up front: making names addresses is a ZERO-BREAK change today.** Adding fields to
`SegmentDef` is a **recompile, not a migration** — today — and that window closes the moment a
pack lands on disk or a third-party mod exists.

| site | what it does | address-safe? |
|---|---|---|
| `dc-api/src/bodies.rs:99-113` `SegmentDef.name: String` | the identity | ✅ a path is a `String` |
| `:278-283` `validate_plan` uniqueness | O(n²) string equality | ✅ (define-time; gen time is free) |
| `:298-314` root count + parent presence | string equality | ✅ |
| `:315-334` cycle walk (`hops > n`) | string equality | ✅ |
| `:355-364` clip joint ∈ plan | string equality | ✅ exact · ❌ if a *pattern* binding is introduced |
| `:143-145` `JointRot.segment: String` | clip → joint | ✅ exact · ❌ pattern |
| `:746-753` `longleg_plan` prefix mutation | `starts_with("leg_")` | ❌ — an experiment, disposable |
| `dc-client/src/body.rs:367-395` `leg_rigs` | `starts_with("leg_")`, `ends_with("_upper")`, `replace("_upper","_lower")` | ❌ **the one true coupling**; retired by declared contacts |
| `body.rs:85` `Pose.joints: HashMap<String,[f64;3]>` | per-frame, keyed by name | ✅ semantically; a per-frame string hash per joint is a **runtime-perf** item at large n (runtime is sacred) — intern once at plan build, client-side, no wire change |
| `character.rs:213-242` `leg_overrides: HashMap<String,_>` | keyed off `LegRig` | ✅ once `leg_rigs` is replaced |
| `character.rs:244-247` `instance.joints.get(&s.name)` | per-frame lookup | ✅ |
| `character.rs:267` `s.name == "neck"` | the look-at joint | ❌ **needs a declaration** |
| `character.rs:424` `s.name == "head"` | face-cue quad | ❌ placeholder art |
| `character.rs:340-354`, `:396-440` | asset + entity maps keyed by name | ✅ |
| `dc-api/src/schema.rs:278-295` `s_body_plan` | `segments` is `{"type":"array"}` with **no item schema** | schema ✅ — but its `description` **enumerates the fields verbatim** and goes stale on any addition. *A printed caption is a published claim* (CLAUDE.md § Gates). |
| `dc-api/src/host.rs:1085-1108` define + `Undo::RemoveBodyPlan` | keyed by **plan** name | ✅ |

**Wire format — is it a breaking change?** **No, today.**
- `BodyPlan` is postcard-encoded and postcard is **positional**, so `#[serde(default)]` does not
  help decoding (corrections #4). A field addition *does* change the encoding.
- **But nothing persists a `BodyPlan`.** The pack is generated from compiled Rust at world
  construction (`dc-client/src/authority.rs:323-325` chains `default_body_pack()` +
  `experiment_body_pack()` through the one door). `dc-core/src/format.rs` and the material intern
  chunks carry no plan. The only encoder is the WASM ABI (`dc-api/src/abi.rs`), which has no
  third-party consumers.
- **Therefore: a field addition is a recompile.** No migration, no version bump, no golden move.

**Goldens / world identity.** Bodies sit behind the determinism firewall — the sim sees only
`CharacterConfig` and parametric posture — so **no golden moves.** ⚠ **`posture-gait.md` § 5
changes this**: once damage resolves against the nominal pose, a plan becomes world-identity
relevant, and § 5 already states the cost (*"converts animation from free-to-edit into a versioned
sim asset"*). **Do the shape change before that line moves, not after.**

---

## 8. Sequencing

1. **The mirror helper in `biped_plan()`** — byte-identical output, no schema change, no wire
   change, fixes the defect that opened this arc (§ 5.4). Half a slice. **Do it first.**
2. **Posture member #0 can proceed NOW, unchanged.** ROADMAP's *"blocked on nothing"* is
   **confirmed**: for the three shipped bipeds the contact set is derivable from `leg_rigs`, which
   already exists. **One condition:** the solver must take **contacts as an explicit input
   parameter**, even while a caller computes them from `leg_rigs`. Otherwise the derivation-from-
   names becomes the definition inside the new solver — **A-1**, in the very slice whose ROADMAP
   entry names A-1 as what it retires.
3. **The declaration slice** — `contacts`, `limbs`, `collide`, and the address grammar. **Do it
   while the wire is free** (§ 7): before any pack lands on disk, before the firewall moves.
4. **Modes and the verb question** (§ 6.B, § 6.H) — needs the user call in § 9 first.
5. **Per-segment materials** — after the materials roster grows (§ 5.6), never before.

**Answer to "can posture's first slice proceed before this pass lands?"** **Yes**, with item 2's
condition. It is not blocked and should not wait.

---

## 9. Compliance, and every ⚠ flag

**Spines ridden.** **S-5** — every new field has an identity default reproducing today byte for
byte. **S-6** — declared relations, never incidental order: contact and purpose are *declared*; what
geometry genuinely determines (laterality, pairing) is *derived*, and neither is inferred from a
naming convention. **S-9** — the derivable base (species bake) + sparse facts (instance deltas)
shape is untouched by this proposal. **S-3** — the plan stays the authority; contacts and limbs are
declarations *on* it, not a summary beside it.

**Anti-shapes guarded.** **A-1** — § 8 item 2 is specifically the guard against the volume/name
derivation becoming the definition inside the new solver. **A-4** — § 5.6 refuses to land a field
whose only consumer does not exist. **A-7** — § 5.3 retires `leg_rigs`' content identity inside a
process, and § 6.H names `KNOWN_VERBS` as the same shape one level up.

**North star.** Bodies are a named core **data-model API**; the declaration is plain data + opaque
strings (crossing constraint, Deviation 1's cheap-insurance shape). **No capability tiering**
(Deviation 2) — everything proposed is authorable by a mod exactly as by the default pack, and
§ 6.H flags the one place today's code violates that.

### ⚠ CONTESTS flags — main session, please rule

1. **⚠ CONTESTS `bodies.md` § "Verb → animation-slot contract" (DECIDED 2026-07-19).** The
   *required*-verbs half and the closed `KNOWN_VERBS` vocabulary reject a bird (`fly`) and a tree
   (no locomotion) at define time, and sit badly beside north-star Deviation 2. The define-time-
   checking *principle* is not contested. **Not edited anywhere. § 6.H.**
2. **⚠ CONTESTS (narrows) the user-originated statement that posture keeps the centre of gravity**
   (journal/0132: *"posture actually is responsible for keeping center of gravity"*, carried into
   `posture-gait.md` § 5). It is exactly right for a balancing animal; this pass finds it vacuous
   for a limbless body and the wrong physics for an anchored one, and proposes a **scope**, not a
   replacement. **Flagged rather than reconciled, per corrections #65. § 6.C.**
3. **⚠ WOULD CONTEST `ideas.md`'s user-originated *"mixed materials per segment, radially
   ordered"*** if per-segment materials ships as a scalar `density`. Not proposed here; recorded so
   the hazard is visible to whoever builds it. **§ 5.6.**

**Explicitly NOT contested:** the plan-parameters sketch (§ 5.7 *uses* it rather than replacing
it); the billboard threshold (§ 5.5 keeps `collide` adjacent to it, not over it); the
stepped-animation identity (untouched); ping-pong for symmetric cycles (survives per-contact
phase); the `bodies.md` § IK user call (**left open**).

---

## 10. What I could not attribute or decide

*"I cannot answer this" is a first-class result.*

- **Whether `SupportKind::Anchor` should exist.** It depends entirely on whether the vegetation
  sketch is ever ratified, and `ideas.md` says *"nothing above may be built without its own
  ratification."* I recommend the bake declare its domain and refuse, but I cannot decide it.
- **Whether the effort term is real minimisation.** The bird says it matters sooner than § 8
  assumes. I have no measurement, and I did not build one — this pass took no builds.
- **Phase when one limb has multiple contacts** — a knuckle-walking hand, a bat's wing-claw. Per
  contact and per limb-chain diverge there. Unresolved.
- **Whether "radially ordered" (user) has any consequence for the mass integral.** A sum is
  order-free, so probably none — but I cannot attribute the intent, and guessing at it is exactly
  what corrections #65 forbids.
- **Whether the user's plan-parameters sketch is intended to cover growth stages** (§ 5.7 assumes
  it does, because a growth stage is a stat-driven scalar). **User-owned.**
- **The quadruped spine's flexion** — I can say it is not expressed; I cannot say whether it should
  be a declared chain, a derived one, or a bake output.

## 11. Open questions, triaged

**User-owned (ratification, not measurement):**
1. Does the closed `KNOWN_VERBS` / mandatory `REQUIRED_VERBS` contract stand, given that it rejects
   a bird and a tree and collides with Deviation 2? (§ 6.H)
2. Is `posture-gait.md` § 5's posture definition **scoped to balancing bodies**, or is the bake
   expected to cover anchored/limbless bodies eventually? (§ 6.C)
3. Does the plan-parameters sketch own the growth axis, or is growth separate? (§ 5.7)
4. Is contact **declared** (this pass's recommendation) or does the user want it derived with an
   escape hatch? (§ 5.1)

**Rides as-built (no ratification needed; interim mechanisms as-is):**
5. `character.rs:267`'s `== "neck"` and `:424`'s `== "head"` — placeholders; they ride until the
   declaration slice, and § 7 records them.
6. `validate_plan`'s O(n²) checks — define-time, gen time is free. Ride.
7. `CharacterConfig` being world-global — already owned by dependency-graph **B4**.

**Needs measurement (do not decide from a doc):**
8. **The bake table's size** once the phase axis is added (§ 6.D): `k` × yaw buckets × posture ×
   phase buckets, per species. Measure before choosing bucket counts.
9. **The per-frame cost of string-keyed joint lookup** at plan sizes above ~50 segments
   (`character.rs:244-280`, `body.rs:85`) — runtime is sacred, and this is the one place this
   proposal touches a hot path.
10. **Whether the derived-contact posture solve reproduces `dc:body/biped` byte-identically** when
    contacts come from `leg_rigs` — the acceptance test for § 8 item 2's identity default.
