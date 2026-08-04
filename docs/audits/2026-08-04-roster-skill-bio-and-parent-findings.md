# Findings for the roster skill: the bio lens, and the parent-materials thread

> **✅ ANSWERED — ALL FIVE § 5 QUESTIONS DISPOSITIONED AND USER-RATIFIED 2026-08-04
> (geo session; banner stamped by the answering integrator, same commit as the
> application):**
> 1. **Scope: WIDENED** — the skill now governs the sim pack's MATERIALS; `GeologySet`
>    membership is one property (the can-this-be-deposited contract; § 2.2's
>    classlessness observation adopted verbatim).
> 2. **Proxy vs absence: SPLIT ADOPTED**, and it reaches parents explicitly
>    (absence-proxies carry heirs; opinions carry citations).
> 3. **Q3: the leaning is adopted AS a leaning** (relation, not kind — marked
>    tentative), and § 3.3's parentage-implies-no-relation constraint joins the
>    skill's step 2 unreservedly.
> 4. **Sequencing: filed live** — ROADMAP § Sequenced "PARENT MATERIALS" entry,
>    design pass after the correlation arc settles; wood/charcoal `combust→` first
>    build per the north-star de-risk; B6-c noted as second consumer, not
>    justification.
> 5. **Density: real, dissolved by the philosophy** — solid density + porosity as
>    axes, bulk derived; reconciliation rides the parent design pass; the skill
>    carries the interim caveat. (§ 4.2's sieve invariant is named in step 2 as an
>    existing derived-value instance.)
>
> All folds live in `.claude/skills/roster/SKILL.md` (watermark updated). This
> cross-session handoff shape — findings doc + pointer banner, ownership respected —
> worked end to end and is precedent.

**Written 2026-08-04 by the BODIES session, for the GEO session.** This is a **§ 0 sweep
result handed to `.claude/skills/roster/SKILL.md`**, which asks to be updated when new
rulings bear on its domain and carries a watermark at its creation the same day.

**Status: FINDINGS AND QUESTIONS. Nothing here adjusts the skill, and nothing here is a
ruling on the geo domain.** The roster, the term space and the material sheet are the geo
thread's; a bodies session narrowing them by edit would be `corrections.md` **#65** exactly.
Two items below are **user rulings/leanings** and are authoritative; everything else is the
bodies integrator's analysis — *a hypothesis until the geo session tests it*.

**What is being asked:** how the skill should adjust, and how the **parent-materials work**
should be sequenced. Both are the geo thread's calls.

---

## 1. What landed since the skill's watermark that bears on it

| what | where | bearing |
|---|---|---|
| **Bodies are in the engine/plugin field**, and *"the evolution system that will drive body creation lives in the **same earth sim pack** that geo named in the skill"* — **user, 2026-08-04** | `dependency-graph.md` § 0a | The skill is scoped by name to *"the sim pack's roster — the `GeologySet` members"*. Tissue materials will be in **that same pack** and will **not** be `GeologySet` members. The scope line needs widening or an explicit disambiguation |
| **The OPINION-vs-ABSENCE test** — an opinion is what two well-made packs answer *differently*; an absence is what every pack answers the *same* way if only we had built it. **User-ratified 2026-08-04** | `dependency-graph.md` § 0b | Bears on the skill's **proxy discipline** (§ 2 step 1). A proxy stands in for an **absence**; a pack-authored value expresses an **opinion**. The skill currently has one bucket for both |
| **B6-a shipped** — the mass integral | journal/0153 | **First consumer in the tree that multiplies `density_kg_m3` by a volume.** See § 4.1 — this is the most actionable item here |

---

## 2. The bio lens — and the crossing is already committed to, in the geo direction

The skill is geo-lensed by construction and says so. The blindspots are **boundaries it was
never asked about**, not defects.

**2.1 The crossing already exists in the roster, and its heir is bodies.** Peat is
*"waterlogged organic accumulation that outran decomposition"*; coal is peat buried; carbonaceous
mudstone is *"the lithified organic soil horizon — a paleosol"*; charcoal is a burned landscape;
bone is midden debris. **These are organisms that became ground.** The skill records that **coal
is a proxy whose named heir is life/organics** — so *"body material becomes geo material"* is not
hypothetical, it is a **sequenced commitment already on the books**, and the thing it waits on is
bodies. Worth the skill knowing which side owes it.

**2.2 The selection contract is doing load-bearing work the skill does not claim for it.** Step 3
says the class is a **selection contract** — formation window, abundance, habit — and implies
nothing about relations. Two consequences it does not draw:
- Tissue has **no formation window**: it is *grown*, not deposited. So a tissue material is a
  registry member belonging to **no class**, and the skill's step 3 has no analogue for it.
- That same mechanism is **exactly how a material can exist and never be deposited** — which is
  the whole supposed advantage of an abstract "parent-only" material (§ 3.2). *Class membership,
  not materialhood, is what decides whether something can appear in the ground.*

**2.3 Conserved-stock discipline holds in one direction and not the other.** *"Mechanical passes
move matter within a composition; only declared chemical edges change it."* **Decay is a declared
chemical edge**, so carcass → soil is already expressible and needs no new rule. **Growth is not**
— an organism adds matter. That end is genuinely open and is not the skill's to solve today, but
it is the boundary.

**2.4 The axes the ruling names are geo axes.** Grain spectrum, cementation, organic fraction,
QAPF composition. Tissue wants **mineralisation/porosity** (cortical ~1900 vs trabecular
200–1000 kg/m³ is *within-region variation on a porosity axis*, not two species — which is what
*"region, not point"* is for), **elastic modulus**, and **fibre anisotropy**. **Density is the
one axis already shared**, and it is the one bodies need first.
⚠ **Modulus is not optional:** `stubs.md` **#50** established that *density is not stiffness*, and
**#44 S3** (`bob_damping`) and **#49** (ligamentous end-range) both had B6 named as their heir
**wrongly** for that reason. A modulus axis is their real heir.

---

## 3. The parent-materials thread — the actual sequencing question

**3.1 State, verified.** Parent pointer + shadowing is **ratified at the material tier**
(`north-star.md:205-209`, origin `journal/0081`). **Per-parameter shadowing is USER-ORIGINATED
and PROPOSED** (`material-genesis-notebook.md:119-137`) — protected text under CLAUDE.md's
user-originated-design rule. It carries a hard requirement: parameters need **three** states —
*inherit* · *override* · *disable* — because *"unspecified"* and *"specified as nothing"* must be
distinguishable (the `Identity::Unrecorded` shape, journal/0101). **There is no code.** The only
`parent` in `dc-core/src/materials/` is the LOD octree's, unrelated.

**3.2 `material-genesis-notebook.md:315-317` § 5 Q3 is OPEN and addressed to the user:**
> *"Is `dc:mineral/vug_filling` — a parent whose job is to be inherited — a first-class authoring
> concept?"*

**USER LEANING, 2026-08-04 (authoritative; explicitly tentative, "may not survive"):**
> *"a parent is strictly just another material — being treated as a parent."*

So parenthood is a **relation**, not a **kind**. Supporting arguments (bodies integrator's, *not*
the user's): it preserves **one namespace** (the 2026-07-22 collapse existed to stop a second
material-like kind); an abstract-only parent's only apparent advantage — never being deposited —
is **already supplied by class membership** (§ 2.2), so it adds a kind without adding a
capability; and *existence is not standing* asks a new entity type to justify itself.

**3.3 A constraint that falls out of two ratified rules colliding — the skill's step 2 should
probably carry it.** Term space: *relations are derived from declared axes, never hand-paired; a
hand list of pairs is a fitted taxonomy and is forbidden.* The A-CLEAN bar: *class co-membership
implies nothing about material relations.*

> **Parentage must imply nothing about material relations either.** If anything downstream reads
> a parent pointer to infer that two materials are *related*, that is a hand-authored taxonomy
> entering through the inheritance door. Inheritance resolves to flat per-material values and
> then disappears — a leaf did not become *related* to its parent, it **got some of its numbers
> from there**.

Consequence: **inheritance and term space are ORTHOGONAL**, not two views of one mechanism. Term
space says *where a material sits*; inheritance says *how its declaration was authored*. A leaf
may parent off something far away in term space and override everything — inheritance imposes no
adjacency. *(This corrects an earlier bodies-session suspicion that they were the same mechanism;
that suspicion was an artifact of assuming the abstract-parent model.)*

**3.4 What inheritance buys, and what it does not.** It dissolves the *"a material must answer
every question to be admitted"* problem: a leaf writes what it knows and inherits the rest, and
the **disable** state makes *unanswered* representable for the first time (today `MaterialProps`
is a dense array — "not applicable" and `0.0` are the same bytes). That yields staging for free:
a body segment asks muscle for **density** → answered; a deposition pass asks muscle for **grain
size** → *unanswered* → **refuses to deposit it, loudly**. No tissue/geo split, no facets, no
invented numbers, and the § 2.1 crossing stays intact because nothing was partitioned.

**What it does not buy:** fabrication **relocates** rather than vanishing — a parent still has to
answer, and an invented number at a parent is inherited by every child *and looks more
authoritative for being shared*. The mitigation is real (fabrication at a parent is **singular and
auditable**; spread across N leaves it is not) but the skill's proxy discipline should probably
reach parents explicitly.

⚠ **There is no engine-level root and no engine defaults.** Every material is pack-declared,
parents included — the 2026-07-28 baseline audit calls parent materials **a PRODUCT SURFACE**. The
engine supplies the *resolution rule*, the *three states*, and — when a chain terminates undeclared
— **absence, never a substituted value.** An engine that helpfully returns `0.0` is inventing
content. *(This corrects a bodies-session error from earlier the same day, which posited engine
root defaults.)*

---

## 4. Two concrete things found in the sheet

**4.1 `density_kg_m3` mixes two conventions, and B6-a is the first consumer that would care.**
The field is documented *"**Bulk** density in kg/m³ — stratification sort key and weight."*
Outside tests it is read in exactly **two** places: `stratify.rs:85-86`'s sort comparator, and
`geology.rs:515`'s `settle_energy` = `sqrt(grain × density)`. **Both are ORDINAL** — they ask
*which is denser*, never *how much does this weigh*.

So nothing has ever forced the question, and the roster answers it inconsistently: **sand 1600**
is a *loose bulk* density (voids included) while **granite 2700** is *solid rock* density. Same
field, two meanings, depending on the material's form.

**This matters to the skill because step 1 names the property sheet as a day-one term source.** A
term definition built on `density` inherits whichever convention that member happened to use.
⚠ **Not verified as a defect** — it may be deliberate and handled in `packing.rs`. Flagged as a
question, not a finding. B6-a's accessor is named `bulk_density_kg_m3()` and carries a marker
saying its absolute value has never had a literature check, precisely because this was unresolved.

**4.2 `extraction_resistance` already contains a derived value stored as an authored one.** Its
own doc comment records a **tested registry invariant**: *"sieve resistance equals grain size, so
sieving always separates fines-first by construction."* That is the `settle_energy`
derive-don't-table precedent **already present and not listed as such**. Step 2 would catch it
today; it may be worth naming as an existing instance.

---

## 5. The questions, all of them the geo session's to answer

1. **Scope** — does the skill widen to "the sim pack's materials" (with `GeologySet` membership as
   one axis a member may or may not have), or stay `GeologySet`-scoped with tissue explicitly
   routed elsewhere?
2. **Proxy vs absence** — does step 1's proxy discipline split along the ratified
   opinion/absence line, and does it reach **parent** materials?
3. **Q3** — is the user's leaning (§ 3.2) adopted, and does § 3.3's *parentage-implies-no-relation*
   constraint join step 2?
4. **Sequencing the parent work** — it is ratified, user-originated, has **zero code**, and the
   north star's own de-risk item names *hierarchical material behavior resolution prototyped on
   wood/charcoal `combust→`* as the first thing to build. Bodies (**B6-c**) is a second consumer
   but **not the justification** — the argument stands without bodies.
5. **§ 4.1's density convention** — deliberate, or owed a reconciliation before term definitions
   are built on it?

**Not claimed here:** that the skill is wrong, that bodies should influence the roster's contents,
or that any of § 2–4 outranks a geo-session ruling. Where this conflicts with something the geo
thread has already settled, the geo thread is right and this document is the stale end.
