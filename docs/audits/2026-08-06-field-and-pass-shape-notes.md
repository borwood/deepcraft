# Field and pass shape — live working notes (2026-08-06)

> **⚠ LIVE WORKING DOC, written during the conversation.** Successor in kind to
> `2026-08-05-debt-walkthrough-notes.md`. Unreconciled until the wrap pass; placement owed.
>
> **Provenance per item.** ✅ = user ruling · ⬦ = integrator analysis, unratified ·
> ⚠ = open/unknown · 📌 = verified at source this session · 🔬 = measured by a dispatched
> agent (audit doc cited).
>
> **Evidence base — three read-only code readings, all merged:**
> `2026-08-06-field-inventory-water.md` · `-solid-earth.md` · `-climate-surface.md`.
> Every 🔬 number below is quoted from one of those and carries its own method there.

---

## 0. The frame — the user's direction, 2026-08-06

✅ **STANDARDIZE ALL PASSES to a shape aligned with the north star and the engine/plugin
divide, and provide the SDK surface that makes it possible.** *"Build the useful primitives
on the engine side and put the opinions on the plugin side."*

✅ **Field passes are to be standardized specifically.**

✅ **Store field information — for future use cases, not only present consumers.** The user
explicitly discounted the no-current-consumer finding as a statement about partial
implementation, not about worth.

✅ **Passes want to declare much more than they currently can.**

✅ **The vocabulary must be open** (the standing E6 direction, restated).

✅ **Working definition offered by the user, and it is the one to design against:**
> *"Field passes adjust a field (a world vector space); cell passes read the field when
> marching over cells, generally."*

---

## 1. 📌 THERE IS NO CELLULAR-VS-FIELD DISTINCTION IN THE CODE AT ALL

Verified at source. `grep -rni "cellular|CellPass|FieldPass|PassShape|PassKind"` over
`crates/` returns **7 hits, every one of them prose** — six doc comments and one test name
(`runner.rs:1632`). `DeepPass` (`runner.rs:311`) carries `id`, `reads`, `reads_prev`,
`writes` — all over the same closed `DeepAxis` — plus cadence/schedule. **No shape marker
exists.**

⬦ So the north star names two shapes and the engine implements one. Every pass is the same
struct; the difference lives only in what a reader infers from the body.

---

## 2. ⬦ "FIELD" IS THE RIGHT NOUN; WE HAD BEEN USING IT FOR THE WRONG THING

**Field does NOT mean diffusion.** The session repeatedly slid from *"field pass"* to
*"pass that needs a solver kernel"*, and the three readings show those are unrelated
populations.

⬦ **The confusion has a clean fix: two orthogonal axes.**

| axis | question | values |
|---|---|---|
| **WHAT IT TOUCHES** | does it read/write a *field* (a world-scale quantity over cells) or the *cell's own contents* (materials, the record)? | field · cell contents · **both** |
| **HOW IT COMPUTES** | what numerical shape does the body have? | pointwise · order-independent stencil · **ordered traversal** · closed-form/analytic · neighbourhood reduce |

**The north star's two shapes are the first axis. The kernel question is the second.** A
pointwise pass that writes a plane is still a field pass — it just needs no solver. Naming
them by their numerics is what made frost, the geotherm and weathering look "surprising."

⬦ **Answer to the user's check — none of the surprising passes is a mislabelled cellular
pass.** Frost and the geotherm write planes and touch no material: field passes with
pointwise bodies. Height-tier weathering is the genuinely mixed one (it moves `R`→`H`, i.e.
material, while reading and writing planes) and is *already* the pass the corpus is arguing
about on altitude grounds.

### 2.1 ⚠ IS THERE A THIRD KIND? The candidate is the RECORD↔FIELD CROSSING

⬦ Not "cellular vs field" but a third relationship, and it has two directions:

- **record → field**: `expose` reads ragged column records and writes susceptibility planes
  (🔬 both agents flagged it as not fitting their group; solid-earth called it *"a record
  query, not a field solve"*).
- **field → record**: `deposition`/the recorder turns per-cell field deltas into units;
  `weather_inventory` turns field-driven rates into facts.

⚠ **Open: is that a third SHAPE, or just a pass declaring both resource kinds?** ⬦ The
integrator's reading is the latter — one pass shape, two declarable resource *kinds* — but
that is exactly the sort of call that should be made deliberately rather than inherited.
North star § Passes says *"for the cell world there is no third"*; it was written before
the record and the fields were understood to be two stores.

---

## 3. 🔬 THE KERNEL INVENTORY — the load-bearing measurement

**One extracted primitive (`dc-core::field::FieldKernel`) with exactly ONE consumer in the
tree** (hillslope creep; 🔬 climate agent, grep over `crates/` = 20 hits, all non-`dc-core`
ones in `erosion/creep*`).

| operation | passes | note |
|---|---|---|
| order-independent stencil diffusion | creep | the one we own |
| conductance relaxation + pins + **upper** obstacle | head | 🔬 *"one primitive away, not a second primitive"* |
| separable windowed smoothing | isostasy | 🔬 only **rhymes** with creep — four separable reasons (conserves nothing · no stability bound · radius in km not a rate · box ≠ Gaussian) |
| scatter-with-kernel point evaluation | tectonics | closed form in `(plates, cfg, x, y)`; no grid term |
| priority-flood returning **pop order** | flood | order is the semantics |
| ordered DAG scan + residual split | drainage accumulation, transport | 🔬 **genuinely share one** |
| directional 1-D prefix march, carried state | climate, eolian | order is the semantics |
| bounded-neighbourhood reduce (3×3 max) | biotic | 🔬 *"not a solver: no potential, no flux, no conservation, no stability bound"* |
| per-cell neighbourhood partition | route, wave's sink pick | 🔬 *"arguably too thin to be a kernel"* |
| **pointwise map — NO kernel** | frost, height weathering, geotherm, weather_inventory | 🔬 the **largest** population |

### 3.1 🔬 THE ORGANIZING AXIS IS ORDERING, AND IT AROSE INDEPENDENTLY IN TWO GROUPS

> **`FieldKernel` is order-independent *by construction and says so deliberately*
> (`dc-core/src/field.rs:276-279`) — which makes it structurally unable to host the
> traversal passes, where the order IS the physics.**

⬦ **These are two engine contracts, not one.** Folding the traversals into the
order-independent kernel would destroy the property that makes it safe. Any SDK design that
offers "the field solver" as a single thing is already wrong.

⬦ And the **pointwise** population needs no solver at all — what it wants from the engine is
a **per-cell driver with a bit-identity promise plus a ctx** (each pass currently rolls its
own `par_iter_mut`). 🔬 The shape already exists in-tree as `weather_behavior::WeatherCtx`.

---

## 4. WHAT A PASS CANNOT DECLARE TODAY (🔬 water agent; ⬦ framing)

Beyond `{reads, reads_prev, writes}` over a closed enum, a pass cannot say:

1. **LIFETIME** — does this plane die with the loop, or survive into the world?
2. **STORAGE SHAPE** — 🔬 four shapes in use with no vocabulary: per-cell plane · per-face
   record · sparse per-species CSR · small world-level table. (⬦ a fifth exists in the
   record: **per-layer-in-column**.)
3. **AUTHORITY vs DERIVED SUMMARY** — see § 5.
4. ⬦ **TEMPORAL RESOLUTION** — final value · keyframes at a declared cadence · a rule +
   parameters re-evaluated on demand. (Integrator proposal from this session's conversation;
   not ratified.)

---

## 5. ⬦ DERIVED SUMMARY — what it means, and when it is defensible

**A derived summary is a plane computed *from* another plane that is the real authority, and
which cannot answer everything its source can.**

The worked case is `recv`. The **authority** is the flux record, which can represent water
splitting several ways out of one cell. `recv` is **one out-edge per cell** — the argmax of
that partition — kept because two consumers wanted a single arrow. It structurally cannot
represent divergence. `spines.md` already classifies it as A-1 and *"a summary in
ARCHITECTURE.md's sense."*

**It is defensible — under three conditions**, which are the ratified doctrine (`ARCHITECTURE.md`
§ *A summary is not an authority*) applied to fields:

1. **Derived, never parallel.** It must be computed *from* the authority. Two independent
   models of one quantity is the failure mode, not the summary itself.
2. **Declared as derived**, so no reader mistakes it for the authority.
3. **A test asserting it AGREES** with the authority.

📌 Today `recv` meets (1) and (3) — the agreement tests exist and are named in `spines.md` —
and **cannot** meet (2), because there is no way to declare it.

⬦ **Which is precisely why the declaration is the fix rather than deletion.** An *undeclared*
summary is dangerous exactly because it looks like any other plane: a pack author reading it
has no way to learn they hold a lossy projection. Declaring it moves the lossiness into the
contract, where a validator can see it and a reader can be warned. **The pass is defensible;
the silence is not.**

---

## 6. 🔬 RESIDENCY — the surprise, and it inverts the session's worry

The conversation spent considerable effort on whether field history is affordable. The
readings say **nothing is being kept at all**:

- 🔬 `DeepField` carries **none** of `precip` / `bio_weather` / `bio_resist` / `frost` — all
  four die with the grid, all four have real in-sim consumers (climate agent read the full
  struct, `field.rs:480-614`).
- 🔬 Every plane that **does** survive — drainage `recv`/`area`/`lake`, `head`, `flux`,
  `exhum`, `t_crust`, `geotherm`, the chapter table — has **zero production readers**
  (searches recorded in both audits; `collapse.rs` → 0 hits). Only `regolith` has one
  (`geology.rs:175`).
- 🔬 The one exported store with production readers, `DeepField::ledgers`
  (`collapse.rs:1625,2916`), **always reads empty** because its pass does not run in the
  shipped world.

⬦ **So the question was never "can we afford to keep fields." Nothing ever asked to.** The
tree-shaking idea (user, 2026-08-06) remains right and is currently a no-op: shaking on
declared readers would load almost nothing.

### 6.1 🔬 Costs found incidentally, all unremarked before today

| | measurement | source |
|---|---|---|
| tectonics | chapter table **5 400 B** vs precomputed forcing planes **20.39 MiB** — **3 960×** | solid-earth |
| isostasy | **~9.5 MB allocated and freed per epoch ≈ 1.9 GB per run** | solid-earth |
| head | transient working set **3.75×** its output (13.36 MB vs 3.56 MB) | water |
| biotic | state-to-output **25:1** (≈57.8 MiB state for 2.27 MiB of planes) | climate |

⚠ Whether tectonics' 20.39 MiB precompute beats on-demand evaluation is **unverified** — the
agent flagged it rather than asserting.

---

## 7. 🔬 CORRECTION OWED — the tilt derivation, and it is the integrator's

**The integrator told the user, with confidence, that bed attitude re-derives from ~5 KB of
plate parameters because "tilt is the accumulated gradient of the forcing field."** A blind
check (the agent was given the claim neutrally and told there was no expected answer)
**refuted the load-bearing half.**

🔬 **`forcing_at` returns crustal THICKENING, not surface uplift.** Between it and elevation
sit (a) the isostatic smoothing at a **100.7 km window**, which flattens *exactly* the ~25 km
orogen wavelength the forcing carries, and (b) the erosion feedback via `track_exhumation`.
**So ∇F_c is not the gradient of surface uplift, and beds do not tilt with it.**

⬦ **Consequence, and it reopens the storage question rather than closing it:** the quantity
tilt needs — differential *surface* rise per chapter — is **path-dependent**, so it is not
recoverable from plate parameters alone. Something per chapter must be kept. The cheap
candidate is one small per-chapter surface-change summary; that is a sizing question.

🔬 **What survives of `tectonics.md` § 8.2:** the chapter stamp and the no-stored-deformation
halves are **built** (`recorder.rs:507-513,524,545`). The shape is right; the formula was
optimistic. A second, purely mechanical gap: the derivation needs six `DeepConfig` scalars
that reach neither `DeepField` nor `Pregen` (grep of `DeepConfig` over pipeline/pregen/
collapse = 1 hit, a doc comment).

---

## 8. Defects surfaced, none filed (write-set discipline — the integrator adjudicates)

1. 🔬 **`dc:deep/biotic` declares `Frosted` in all three read-sets (`runner.rs:595-597`) and
   never reads the frost plane** (grep of `frost` over `biotic.rs` = 2 hits, both prose).
   Over-constrains the topo-sort. Whether the emitted order actually moves is **unverified**
   (the agent ran no cargo).
2. 🔬 **Two precipitations.** The collapse tier samples the **pregen** `CellGrid` precip
   (`collapse.rs:1335-1366`), not the deep-time marched plane — so the walked world's rainfall
   never saw 200 epochs of orogeny. ⬦ **Candidate contributor to the user's standing
   observation that "tectonics doesn't do what I expect" — a candidate, not a diagnosis.**
3. 🔬 **`Erosion::exposed()` (`weathering.rs:125`) has zero call sites** (`grep -F 'exposed()'`
   = 2 hits, both comments; `\.exposed(` = 0), and its doc names a measurement probe that does
   not exist.
4. 🔬 **Two `head`s in one crate** — `water/sat.rs:366`'s `self.head()` is the *retired*
   `H = y + sat` proxy that `deeptime/head.rs` exists to supersede.
5. 🔬 **Flux-record residency figure is stale** — the corpus's 42.6 MB predates P11 slices 1–3.

---

## 9. ⬦ Where this leaves the SDK question the user asked

**"Do we need one or more field-solver primitives, one or more storage solutions?"**

- **Solvers: more than one, and the split is by ORDERING, not by physics.** At minimum an
  order-independent stencil family (own it; creep and head are both in it) and an ordered-
  traversal family (flood, accumulation+transport, the marches). Plus a **pointwise driver**
  that is not a solver at all and serves the largest population.
- **Storage: at least four shapes**, and the missing thing is not a fifth shape but a
  **vocabulary** for declaring which one you mean (§ 4).
- **And the open-vocabulary work is upstream of both** — a pass cannot publish an axis the
  engine does not name (`Resource`, `DeepAxis`), so none of the above is declarable today.

⚠ **Not designed here. This document is the input to a design pass, not the pass.**

---

## 10. Session day 2 (2026-08-06, after the readings) — three things established

### 10.1 ✅ THE FIELD/CELL INVARIANT, in the user's words — **NOT yet ratified into a design doc**

> *"Field passes adjust a field (a world vector space); cell passes read the field when
> marching over cells, generally."*
>
> *"I can't imagine a field pass touching materials at all except reading the record —
> never changing them."*

⬦ **Stated as the working invariant:** a field pass may read fields, write fields, and **READ**
the record; it may **not** change materials or write the record. A pass that changes cell
contents is a **cell pass**, which reads fields while marching cells.

⬦ **This DISPOSES of § 2.1's open third-kind question.** Record→field (`expose`) is legal —
it is *reading*. Field→record (deposition) is **not a field pass**; it is a cell pass that
reads fields, which is exactly the user's own definition of one. **No third shape is needed.**

⚠ **Recorded here and deliberately NOT written into `north-star.md` or `material-behavior.md`
as DECIDED.** The user's framing that day opened *"not making judgements, just exploring."*
The invariant is being **applied as an analysis lens** by the pass-compliance table
(`2026-08-06-pass-io-and-compliance-table.md`); promoting it to a decision is a separate ask.

### 10.2 ⬦ WHY A SUMMARY FIELD EXISTS — three reasons, ranked

A summary is a **materialized read-time reduction**: anything it answers, a reader could
compute from the authority on the spot. What materializing buys:

1. **One canonical reduction instead of several.** Three readers each taking "the dominant
   direction" with their own tie-break can disagree with each other. Materializing makes the
   reduction a single decision. *Strongest reason, least obvious.*
2. **Cost, under the granularity rule** — never recompute inside a hot loop something that
   does not change inside it.
3. **A narrower contract** — a reader needing only the dominant direction is not coupled to
   the authority's full shape.

**Not justified when:** there is one reader (compute it there), or **the consumers moved and
the plane stayed** — which is `recv` today, and why it fails *"if this consumer disappeared
tomorrow, would this code still exist in this shape?"* **Summaries are not the defect;
orphaned and undeclared ones are** (§ 5).

### 10.3 📌 CORRECTION — HEIGHT WEATHERING IS A PER-EPOCH IN-LOOP PASS

The user's recalled *"operates once after loop end"* is **true of its sibling, not of it.**
📌 `dc:deep/weather` sits in the epoch chain `forcing → drainage → transport → **weather** →
diffuse → isostasy → deposition → eolian → wave → biotic`, **asserted by a live test** as
exactly `Erosion::step`'s phase order (`runner.rs:1222-1230`). The post-hoc one-shot over
frozen state was **inventory** weathering (S18), rebuilt per-epoch in Movement 3
(journal/0094, corrections #46/#47).

⬦ **The user's underlying concern survives with a different mechanism.** Products *do*
accumulate over time — as **height**. What does not accumulate is the product's **identity**,
because at this tier the product is metres of regolith, not a material. The tier that would
give it one is the inventory tier, and that is **OFF in the shipped world**. So: not *"runs
once"* but *"runs every epoch and produces something anonymous."*

⬦ **The user's other three points on this pass all land**, and each already has a home:
(B) no material edge → D-1's *"acts in the right place, records in the wrong shape"*;
(C) should be several agent passes → the ruled split criterion (coupling timescale vs
cadence, `material-behavior.md:349-371`), compatible with *"weathering is ONE process,
saprolite is a state along it"* — one process, several agents; (D) writing a field **and**
reaching the record is what § 10.1's invariant forbids, and is already filed as D-2.
