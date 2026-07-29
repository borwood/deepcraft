# Refinement coupling — corpus priors research (2026-07-29)

**Read at `dc96b4d`.** Produced by a read-only research agent for the E5 design pass, on
the user's direction: *"find prior plans, unratified or not, that exist on materials,
material behaviors, and flux, and beyond — places you or i have overlooked or forgotten."*
Consumed by [`docs/design/refinement.md`](../design/refinement.md) (~~PROPOSED~~ **CAUTIOUSLY
RATIFIED 2026-07-29, same day** — header updated per the mutable-header rule; doc-topology
F10), whose § 4/§ 5 are the synthesis of what is below. **Its § 5 item 6 (the face-pairing
"live blocker") is RESOLVED 2026-07-29 — re-argued fresh and cautiously ratified as the
three-mode confinement rule (`flow.md` § 11.5 banner; corrections #75: the "not ratified"
status this file relayed was stale at assertion — § 11.5 had ratified the principle
2026-07-25 and the flag it cites was never stamped).** **This is a dated research artifact** — quotations were
verified at `dc96b4d` and are not re-verified by later readers of this file; the immutable
body / mutable header rule applies.

Status legend: **DECIDED/RATIFIED** (user) · **PROPOSED** (recorded, not decided) ·
**passing remark** (prose, no ratification event).

---

## 1. Priors that SUPPORT the family/parent-inheritance coupling

### 1.1 The exact idea already exists one tier down, as the "most actionable finding" of an open edge

`material-genesis-notebook.md:174-186` — **PROPOSED** (2026-07-26): *"**(b) The
plug-and-play mechanism IS parent inheritance — so our default pack's PARENT materials are
a product surface.** A shallow modder's minimum viable mod is choose a parent…
**Nothing in the corpus says this, and it is the most actionable finding in the thread.**"*
Echoed into `north-star.md:58-67` as the 🔖 OPEN EDGE banner on the core/plugin boundary.
And `material-genesis-notebook.md:315-317` § 5 Q3 — **asked of the user, unanswered**:
*"Is `dc:mineral/vug_filling` — a parent whose job is to be inherited — a first-class
authoring concept?"* The refinement proposal answers an open question rather than opening one.

### 1.2 Per-parameter shadowing is user-originated

`material-genesis-notebook.md:119-137` — **user-originated, PROPOSED**: *"`transition["to_calcite"].min_temp = x` — shadow one term, inherit the rest. First ancestor wins up the
tree **for params instead of entire edges**. I can nullify a transition on a leaf this way
too."* With the hard requirement: parameters need **three** states — inherit · override ·
disable — the `Identity::Unrecorded` shape (journal/0101). ⚠ Protected text under
CLAUDE.md's user-originated-design rule.

### 1.3 Parent pointer + shadowing is ratified at the material tier

`north-star.md:205-209` (ratified direction 2026-07-23); origin `journal/0081:32-37` (the
user's sketch); `materials.md:519-520` **DECIDED 2026-07-22**: *"Categories stay, and
become registrable… Packs add classes, not only members."*

### 1.4 "Transport is a FAMILY" — the shipped, measured worked instance

`material-behavior.md:794-802` — **RATIFIED 2026-07-24**: water/wind/ice/gravity share
one load/entrain/sort/deposit machine, differing only in **field and competence curve**.
The second member (creep, BUILT 2026-07-26, journal/0112) joined by **nullifying** the
competence term — *"Giving creep a ceiling to make it look like the river would have
erased the one contrast the slice buys"* — and disagreement-with-environment mass went
**0.000006 % → 65.206 %** with *"no knob in this slice"*. journal/0112:33-54: *"Building
creep as its own system would have had to invent that contrast. Building it as a regime of
the family got it for free."* **The strongest empirical support in the corpus.**

### 1.5 The enhancement doctrine's open half

`geology.md:352-378` — **DECIDED 2026-07-21**: an enhancement is a pure function of
recorded history AND *"local process and deep process must agree. In what sense they must
agree is an explicitly open design question."* The family coupling is a concrete answer to
the open half.

### 1.6 Class-as-contract + selection-by-fitness ships today

`geology.md:13-22` (backbone 2026-07-18); `dc-core/src/materials/geology.rs:373-381`
(`select(class, ctx, u)`), `pipeline.rs:96-110` (`Pass.selects`). A record already declares
a family (class) and expression re-resolves a member per voxel column (`dithered_member`).

### 1.7 Form archetypes — retroactive family coverage precedent

`materials.md:114-118` (passing design note, 2026-07-18): *"New material × existing
archetypes = its whole item family for free; a new archetype retroactively covers every
qualifying material."*

### 1.8 The tier's ownership and contract pre-exist

north-star § refinement resolved to **(c)** (DECIDED 2026-07-28); the definition ruled
2026-07-29 (`ARCHITECTURE.md`, *"ZERO refinement primitives"* exist); the operator contract
**RATIFIED 2026-07-25** (`flow.md:499-520`): pure fn of (record, shared face data,
position); *"solving a small boundary-value problem inside a cell… the load budget as mass."*

### 1.9 Warn-not-forbid is ratified doctrine for content-vs-content

`ARCHITECTURE.md:486-492` **DECIDED 2026-07-22**: *"Conflicts warn, they do not fail
closed… because the user, not the engine, should own which of two mods wins."* Also
`:520-527` (headless: last-in-order + loud log); `API.md:92-102` **DECIDED 2026-07-19**
(pack-degradation: *"loudness is the invariant"*); `geology.md:228-234` (refuse-to-build is
*"the interim, not the end state"*); `ideas.md:188`; `ores.md:517`;
`material-genesis-notebook.md:169-172` (inert-is-legitimate, undischarged-edge-is-loud).

### 1.10 Read-only decorators that stack are SHIPPED — riders

`dc-worldgen/src/geology.rs:44-86` (`StrataEvent` riders: `ore` debris-slot, `accessory`
pore-slot); journal/0099:41-57 (*"the mechanism was already there"*); placer already renders
*"with zero renderer-side code"* (journal/0011:112-122).

---

## 2. Priors that CHALLENGE or complicate it

### 2.1 🔴 A generalized expression-operator vocabulary was proposed and REJECTED (journal/0095)

`journal/0095:63-81`: *"I had just finished arguing… for '**expression operators over
field-ids**' — a generalized vocabulary of sub-cell geometric operators, of which a river
would be `carve-along-gradient`… **exactly the thing the user was rejecting**: a mechanism
that deforms geometry to imitate a result rather than spending a material budget."*
Codified `flow.md:25-27` (**RATIFIED**): *"Any mechanism that draws a channel instead of
spending a budget is a second model of the carve and is forbidden by this document."*
**The family name makes the wrong side easy to write; the discriminator is the drawdown
budget ledger.** → refinement.md Law 1.

### 2.2 🔴 A family-on-the-record is structurally a stored verdict

`dc-core/src/coarse.rs:137-149, 185-188`: *"storing the winner is the plurality bug S-4
forbids, frozen into a type"*; journal/0072:105-113; `spines.md` S-4 (*"a feathered square
is still a square… Verdict-smoothing is never the fix"*). A categorical tag expressed
across a 460 m span is a square drawn by taxonomy. → refinement.md Law 2 (schema-level
family; voxels reached only through the two legal moves; boundaries blend by recorded
magnitudes via faces, never by feathering the tag).

### 2.3 S-3 / A-1 — the record-declares-its-expresser is a summary claiming authority

`ARCHITECTURE.md:349-380` **DECIDED 2026-07-21** (*"a stand-in becomes the definition
unless something stops it"*); live instance `fill.rs::is_loose` (class-string test as the
form rule). If a deeptime pass ever reads the family back (rate, ordering, budget split),
the geometry rule and the physics rule become one string. → refinement.md § 9 names this a
spine violation by construction.

### 2.4 A-7 — name-keyed coupling vs the user's own term-keyed preference

`spines.md:1499-1516` **DECIDED 2026-07-22** (naming a content identity inside a process
*"is not a ratifiable carve-out"*); `material-genesis-notebook.md:85-106` — **user-originated
core**: *"A pass selects edges by the TERMS they carry, not by a name they share… the
transition name not being known to our passes, just a token for reference by leaf
materials."* → refinement.md § 4 splits the roles: participation is record-side (the parent
declaration), operators key on TERMS, engine sees opaque ids only.

### 2.5 🔴 No inheritance hierarchy exists in code anywhere; the one closed vocabulary got a ruling against it

Measured nulls at `dc96b4d`: `MaterialProps` has **no parent/class/tags field**
(`materials/mod.rs:274-348`); geology classes are 10 flat strings; **zero** hits for
`shadow`/`ancestor`/`archetype`/`decorator` in `crates/*/src/`. And journal/0119
(corrections #65, **DECIDED 2026-07-26**): `DeepAxis` — *"a closed enum, inside the engine,
whose variants are the names of the default pack's pipeline stages"* — is "the violation."
Escape: `ARCHITECTURE.md:646-649` — resource ids are opaque and open; a pack surface, not
an engine ABI.

### 2.6 🔴 journal/0119's razor cuts "one mass-expressor per family"

`journal/0119:121-125`: *"A derivation whose inputs were constructed to produce the desired
output is not a derivation."* A validator checking a partition the author drew produces
nothing. → refinement.md § 5: the check is property-based (two operators drawing down one
budget), taxonomy-independent.

### 2.7 The tier's freshest ruling is a layering, not an inheritance

ROADMAP (**AMENDED 2026-07-29, user**): per-cell driver = content · spanning octaves =
engine · per-voxel draw = engine; *"invoke each question at the granularity at which its
ANSWER changes."* The family is read at record granularity (where its answer changes).

### 2.8 The base of the borrowed hierarchy is itself unsettled

`material-behavior.md:755-761`: materials-are-minerals is *"the LEAST settled, most
revisable claim here."*

### 2.9 The one shipped taxonomy-driven selection measured near-flat

ROADMAP:2634-2635: *"the class system is self-admittedly scaffolding and measured
near-flat (entry-species probe: formation context barely changes which member)."* The
richness belongs in the pass physics, not the taxonomy — a hierarchy alone buys nothing.

---

## 3. Overlooked machinery consumable by the design (the wins)

1. 🏆 **`CoarseField<T>`** — built 2026-07-22, ratified (*"we finish this today", "this is
   foundational"*), **zero production callers for seven days** (spines § 3 row, verified:
   eleven workspace hits, all prose). Encodes the two legal coarse→fine moves, caller-owned
   entropy (`DitherSource`), the cake law, `summarize`, and a `compile_fail` doc-test on the
   raw read. Any design not behind this type builds a second sampling contract (A-4).
2. 🏆 **The contradiction already resolved one file away**: `ARCHITECTURE.md:621-624`
   worries octaves vs `sample_dithered` are rival ratified designs; `dependency-graph.md`
   § 3 already reconciled them (*"octaves supply the source, sample_dithered the draw,
   summarize retires the residual bias — they compose"*). Neither file points at the other.
3. **`DeepField::flux` + `DeepField::head`** — built and idle **on purpose**, intended
   consumer recorded as *"refinement as a boundary-value problem"* (spines § 3;
   dependency-graph § 3 P3→E5: *"the strongest ready-now signal on the board"*).
4. **`recorder.rs::overprint_top`** — the one in-tree transform-in-place operator, named
   *"the template for a diagenesis operator"* by the 2026-07-22 audit, never generalised.
5. **Riders + `PoreDraw`** — the stacking-decorator discipline with both failure modes paid
   for: merge key widened to `(member, accessory)` (journal/0099:120-141) and sibling-rider
   draw correlation (r = +0.4878, corrections) cured by per-decorator salt + event index in
   the draw address, frozen into a type (`fill.rs:276-318`).
6. 🏆 **`ideas.md:169-192` — identity-preserving patches with union-of-patches merge**
   (user-originated 2026-07-19, cited by nothing since): field-level diffs; *"two patches
   touching disjoint fields BOTH apply; pack order trumps only on per-param conflicts,
   loudly"*; invalid patch = named warning + skip; patch set joins world identity. The
   corpus's only stated composition semantics for two authors touching one declaration —
   ten days before notebook § 2.3 re-derived per-parameter shadowing independently.
7. **Selection-address inheritance is shipped** (journal/0099:92-106): the weathering front
   inherits member *and* address (`sel_salt`, `sel_tag`, `depth_m`); *"inheriting the
   address matters as much as the member."*
8. **Tri-state mechanism**: journal/0101's `UNRECORDED`, not `Providers`' two-state
   `Option<fn>` (⚠ the Providers *system* is scaffolding under a loud 2026-07-28 user
   correction — only the code shape may travel).
9. **The validator socket exists**: `pipeline.rs:236-258` (`check_class_satisfiability`) +
   `dc-api/src/classes.rs:319-352` (`validate_pack`) — home and message convention for the
   tier's load-time checks. ⚠ In code the precedent is uniformly REFUSE; **no logging
   crate is wired into any headless crate** — the warn tier has no emission channel and
   journal/0065 holds the false-green receipt for a WARN nobody reads.
10. **Coal partings** (`ideas.md:454-459`) — a second first-customer already scoped:
    *"collapse-tier procedural detail… the record knows something the voxel grid is too
    coarse to show."*

---

## 4. Direct answers

**(a) Has any prior conversation discussed linking a record/budget to its expresser?**
Yes — the most-developed thread in the corpus (geology.md's everything-must-be-expressed +
enhancement doctrine; flow.md's record-is-the-only-seam + boundary-value contract; the
spines § 3 intended-consumer lines; and the negative instance journal/0097's slab). **But
always as an obligation on the reader or writer — never as a declared binding. The
record-names-its-expresser inversion is new.**

**(b) Has operator/expression inheritance ever been named?** **No — not once** (measured
nulls: decorator 0, archetype 0 outside item-forms, template 0 relevant, `Operator` type
nonexistent). Nearest priors: journal/0099's address inheritance (shipped, never
generalised) and transformation axes — *"inheritable from a category, patchable per
material, deletable"* (user, journal/0077 → materials.md **DECIDED 2026-07-22**) —
inheritance of a *process*, the closest ancestor to inheriting an operator.

**(c) Validator warn-vs-forbid?** Split cleanly: **WARN is ratified for content-vs-content**
(seven citations, § 1.9; loudness the invariant; the user owns which mod wins) — **FORBID
is reserved for structural impossibility and reproducibility** (missing gen-affecting
plugin = hard refusal; A-7; compile-enforced edges; the CoarseField type wall; Deviation
2's no-walls). Complication: warn has **zero implementation** — no channel, everything
refuses — and a fallback that fires must be **recorded in world identity**
(`ARCHITECTURE.md:510-519`).

**(d) Run-once / seeded / pre-loop priors?** (1) *"pre-loop is not a pass"* — the only
ratified clause of the genesis three-way split (notebook § 5 Q5). (2) Seeded-before-loop +
fire-on-multiples is the shipped low-cadence shape (journal/0090:131-135 climate;
journal/0093:44-53 geotherm; the pre-loop seed is why `require_creator` is off).
(3) 🔴 *"a one-shot of a continuous process is a category error"* (corrections; measured
~152× — journal/0094) — binds deeptime passes, not per-chunk operators; a ratification
should say which it means. (4) Epochs with *"run N / until"* — user sketch
`ideas.md:597-604`, promoted into ratified north-star § tuning-is-data. (5) The one shipped
pass-owned sub-cadence: creep's internal split (journal/0122), filed stubs § 30, heir E4.

---

## 5. Contradictions noticed while reading (side product; not dispositions)

1. `ARCHITECTURE.md:621-624` vs `dependency-graph.md` § 3 — the octaves/sample_dithered
   "contradiction" is already resolved in the graph; ARCHITECTURE doesn't point there.
2. `material-behavior.md` § 14 (predicates are plain data) vs notebook § 2.4 (terms stay
   functions) — flagged in-place at `material-behavior.md:1021-1029`, still open.
3. `stubs.md:15-25` vs `spines.md:409-411` — the two provider-seam rosters disagree by one
   member (`paleo_temperature` vs the removed `burial_temp_c`).
4. `geology.md:215-234` (eventually warn-and-skip) vs code (refuses) vs ARCHITECTURE's
   provider-conflict warn — three positions on one axis, none cross-referenced.
5. `north-star.md:20-25` — the one-sentence shape still carries the struck
   `~~native field-solvers~~` mid-sentence.
6. `flow.md:503` (operator contract) vs `flow.md:133-145` — the face-pairing rule is
   UNDER-SPECIFIED / NOT RATIFIED and flagged *"resolve before any refinement/expression
   slice."* **Live blocker on the tier's first slice.**
7. `ideas.md:465-479` (2026-07-20: "do not design a plugin-pass mechanism yet") —
   superseded in practice by the north star; carries no supersession banner where its
   siblings do.
8. `materials.md:114-118` "form archetypes" vs `material-behavior.md` § 2 "forms are not
   SDK-registrable" — two senses of "form" sharing one word, never reconciled.
9. ROADMAP refinement entry self-flags that it *"recorded the decision and still opened by
   saying it was never made."*
10. `spines.md:1405` CoarseField row verified at `96ab14b`; null re-verified at `dc96b4d`
    (zero production callers), line numbers not re-checked.
