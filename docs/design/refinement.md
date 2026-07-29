# Refinement — the presentation layer

**Status: ~~PROPOSED~~ CAUTIOUSLY RATIFIED — user, 2026-07-29, same day.** The ruling,
with its two qualifiers carried verbatim because they govern all downstream work:

> *"refinement draft is cautiously ratified. first members will require their own
> dedicated design attention (coarsefield, fluvial etc) built on the refinement bones
> you propose and maybe showing us we need additional machinery. **they should be
> revisited in light of this design, not taken as wherever they landed prior to
> this.**"*

**What "cautiously" binds:** the bones (§§ 3–6) are the ratified shape; **§ 7's members
are directions, not build orders.** Each member gets its **own design pass against this
document** before any build — and a member's prior plan (P4's adoption plan, any earlier
channel sketch) is **superseded as a plan**: re-derive it here, don't inherit it. A
member design pass that finds these bones missing machinery reports that loudly — that
is the "cautiously," working as intended.

*(Original draft status, kept for the record: PROPOSED, drafted 2026-07-29, the E5
design pass; sections carry their provenance individually.)* The tier's
*boundary* is already DECIDED — north-star § refinement resolved to **(c)** (user,
2026-07-28: engine owns the primitives + the runner; plugins declare the content) and
its *definition* is ruled (user, 2026-07-29, `ARCHITECTURE.md`: *"any primitive
responsible for the way things are actually drawn in the runtime game where we
interpolate/upscale fine chunks from coarse cells… totally necessary for a
plugin-agnostic engine"*). **What this document proposes is the surface**: the
authoring shape, the coupling, and the first members. That is exactly the slot
north-star § refinement left open: *"the boundary is decided; the surface is not."*

**Framing (user, 2026-07-29): refinement is the PRESENTATION layer.** The deep sim
thinks in ~460 m cells; the player walks voxels. Refinement is how recorded history
becomes visible ground — a *view* over the record, never a second simulation and
never a second geometry model.

Read with: `north-star.md` § The core/plugin boundary + § refinement ·
`flow.md` §§ 3–4 (the ratified contract this doc builds on) · `geology.md`
§ enhancement doctrine · `material-genesis-notebook.md` § 2 (the PROPOSED shapes this
doc consumes) · `docs/dependency-graph.md` E5 ·
**[`docs/audits/2026-07-29-refinement-coupling-priors.md`](../audits/2026-07-29-refinement-coupling-priors.md)**
— the evidence base: every supporting prior, every hazard, and the overlooked
machinery this design consumes, each with citation and ratification status.

---

## 1. The problem, stated as what a mod cannot do

A mod can add **materials** (which ride existing material→voxel rules) and **move
mass** (which rides the height field) — so it gets emergent *valleys* free. It
**cannot add a new KIND of visible structure** — a channel form, a cross-bed, a
lava tube, a sorted lamination — because sub-cell geometry lives in
`collapse.rs`: 2,415 lines of engine code with essentially zero declaration.
The engine hardcodes the vocabulary of expression.

The retiring exemplar: `collapse.rs::carve_rivers` (`:1628`) — a distance-based
bank carve from a `RiverSeg` chain. It *draws* a channel instead of *spending* a
budget, which `flow.md` has forbidden since 2026-07-25. It predates the doctrine;
existence is not standing; this tier is what retires it.

## 2. The governing priors (all ratified; none re-argued here)

1. **Pure-fn contract** (`flow.md` § 4): a refinement operator *"may be clever,
   must be a pure fn of (record, shared face data, position)"* — never a
   neighbour's refined output. One constraint, two payoffs: chunk determinism
   regardless of resident neighbours, and safe authorability.
2. **The record is the only seam.** Deeptime writes; refinement reads. If
   refinement needs something, deeptime must have recorded it.
3. **Spend, don't draw** (`flow.md` §§ 3–4; journal/0095, the killed
   `carve-along-gradient` sketch): *"There is no separate 'carving.' There is only
   the budget, spent somewhere."* Refinement solves a small boundary-value problem
   inside a cell — face fluxes as Dirichlet conditions, the load budget as mass.
4. **Everything in the ledger must be expressed** (`geology.md`, DECIDED
   2026-07-21), and an enhancement must be a *pure function of the recorded
   history* that *agrees with the deep process*. In what sense it must agree was
   left open there; § 4 below is this document's answer.
5. **The granularity table** (ROADMAP, amended 2026-07-29): invoke each question
   at the granularity at which its answer changes — per-cell driver = content ·
   spanning bridge = engine primitive · per-voxel draw = engine primitive.
   Operators are invoked per-cell/per-chunk, never per-voxel.
6. **No capability walls** (north-star Deviation 2). A mod can author anything a
   default can, including a whole new family. Conflicts warn loudly; they do not
   fail closed (`ARCHITECTURE.md` DECIDED 2026-07-22: the user, not the engine,
   owns which of two mods wins).
7. **Runtime is sacred.** Refinement runs on the runtime clock. Kernels are
   native; operator granularity is bounded by design (prior 5); nothing here may
   grow a per-voxel content call.

## 3. The machine — two authored pipelines, one shape, two clocks

The north star already says a field pass "plants its API on the cell" — coupling
between deeptime passes is by declared resource id over the cell API. Refinement
is **the same authoring shape run by a second executor**:

| | deeptime (compiles the world) | refinement (presents it) |
|---|---|---|
| executor | epoch loop, gen-time free | per-chunk on demand, runtime sacred |
| unit of state | cell record | **working sub-cell state** |
| content authors | passes (cellular + field) | **operators** |
| engine owns | pass runner + field kernels | **refinement runner + refinement kernels** |
| order | authored per world (E7) | **authored per world, same mechanism** |
| purity | declared reads/writes, validated | pure fn of (record, faces, position), validated |

**The working sub-cell state** is the substrate operators transform in authored
order before a single voxelization at the end: the fine height field, the column
interval structure (the `ColumnRec` interval log is the degenerate case), and the
material assignment. It is initialized by the base reconstruction — which is
`CoarseField` sampling (§ 7, member #0) — and every operator sees what earlier
operators did *within the chunk's own cells* (that is within-cell composition,
which purity permits; cross-cell coupling remains faces-only).

**Engine-owned kernels** (the `texture()` analogy — candidates, first members in
§ 7): `CoarseField::sample` / `sample_dithered` + `DitherSource` ·
~~`summarize`~~ **RULED OUT of this tier 2026-07-29 (user): `summarize` belongs to the
octree node contract — LOD machinery stays engine, not plugin-owned, and the far/LOD
synthesizer is a different executor this document does not model** (member-#0 pass,
MM-4) · the sub-cell boundary-value solver · position-addressed noise · interval fill.
**`DitherSource` is engine-internal with engine impls; a pack SELECTS a source by id and
parameters, never supplies per-voxel code** (user, 2026-07-29, after full unpacking —
the per-voxel loop is engine-executed for first-party and packs alike; the octaves
source's lineage runs to the 2026-07-24 member-stepping diagnosis, *"fix = octaves,
not resolution"*).
The kernel owns its own stability/validity bounds (the E4 rule, applied to this
tier from birth rather than retrofitted).

## 4. The coupling — record families

*(The load-bearing proposal. Provenance: assembled from user-originated pieces —
parent inheritance + per-parameter shadowing (`material-genesis-notebook.md`
§§ 2.3/2.5b), term-keyed participation (§ 2.2), union-of-patches merge
(`ideas.md:169-192`, 2026-07-19) — with the assembly itself assistant-proposed
and gamed against mod scenarios in the 2026-07-29 design session.)*

**A record declares its expression FAMILY by choosing a parent; the family is a
TERM SCHEMA plus inherited expression machinery; operators read TERMS.**

```
# default pack
record dc:record/channelized_flow {
    terms: { face_flux, load_budget, grain_distribution, mobility_hint }
    expressed_by: dc:refine/fluvial          # the family's mass-expressor
}

# a lava mod — participation chosen by the author who knows the physics
record mymod:lava_flux {
    parent: dc:record/channelized_flow       # opts IN: inherits the fluvial
    terms:  { viscosity = ..., cooling = ...}  # expression, shadows terms
}                                            # → channelized lava with natural
                                             #   levees, for free (physically real)

# a glacier mod — the same choice made the other way
record icemod:ice_flux {
    parent: icemod:viscous_sheet             # declines: no accidental V-channel
}                                            #   carved through the ice
```

The three roles, and why each is where it is:

- **Participation is chosen record-side** (the parent declaration). Term-matching
  alone cannot know that ice flux must not express as a river — matching terms,
  wrong physics. The record's author knows. This is the glacier test.
- **Operators key on TERMS, never on names** (notebook § 2.2, user-originated:
  *"the transition's name is a token for authors and heirs — not something a pass
  keys on"*). Adding a record never changes any operator's read-set. The family
  *name* is a token; the family *schema* is the contract.
- **The engine sees opaque ids only.** The family taxonomy is entirely pack-side —
  `dc:record/channelized_flow` is a **default-pack product surface** with the
  ordinary obligations of one (exactly as the default pack's parent materials
  are), never an engine enum. Anything else rebuilds `DeepAxis`.

**Shadowing semantics** — adopted whole from the user's 2026-07-19 patch sketch
(`ideas.md:169-192`), which stated them before the notebook re-derived half:
field-level diffs, **union-of-patches merge** (two authors touching disjoint terms
of one def BOTH apply), order trumps only on per-term conflicts, **loudly**.
Parameters are tri-state — *inherit* (unspecified) · *override* (a value) ·
*disable* (explicitly nothing) — journal/0101's `UNRECORDED` shape, **not** the
two-state `Option<fn>` of `Providers` (whose *system* is scaffolding and off
limits; only the code shape ever travels).

**Expression inherits the ADDRESS, not just the value** (journal/0099's shipped
lesson, promoted to a rule): what a child family inherits includes the selection
addresses its expression resolves through, so a tuned family stays the same
draw-for-draw structure as its parent wherever it doesn't shadow.

## 5. The laws that keep it honest

These three answer the three named hazards from the corpus research. They are the
part of this design most worth ratifying explicitly, because each guards against
a failure the corpus has already paid for once.

**Law 1 — the anti-carve law** *(hazard: journal/0095's rejected
`carve-along-gradient`; the family name makes the wrong side easy to write).*
An operator's expression varies **only** with recorded continuous quantities —
fluxes, budgets, inventories, spent through the drawdown ledger — plus
position-addressed noise. **The family selects which machinery runs; it never
selects what gets drawn.** A `channelized_flow` cell with near-zero recorded flux
expresses a near-zero channel. If the sim's rivers do nothing, the world honestly
shows rivers doing nothing (journal/0111: 0.02 % fluvial export today) — the fix
is upstream calibration, never a cosmetic operator. An operator that renders the
family instead of the record is `carve_rivers` with extra steps.

**Law 2 — no verdict painting** *(hazard: the stored-verdict defect `CoarseField`
exists to forbid — a categorical tag expressed across a 460 m span is a square
drawn by taxonomy).* The family is schema-level, attached to the *record
declaration*, not a per-cell field — but its expression must still reach voxels
through the two legal coarse→fine moves (interpolate a quantity, or dither
membership at source-cell granularity). Where family-bearing records meet at cell
boundaries, expression blends by *recorded magnitudes through shared face data*,
never by feathering the tag.

**Law 3 — conservation, as a gate test.** The working sub-cell state, integrated
over the cell, equals the cell's recorded totals after every operator: refinement
**redistributes; it never creates or destroys**. The drawdown ledger enforces it
incrementally (an operator draws budget down; over-expression goes negative and
fails loudly; decorators cannot touch it), and the invariant is scale-free —
assert it at the smallest extent that exercises it.

**The validator: property-based, warn-loudly** *(hazard: journal/0119's razor —
"one mass-expressor per family" checks a partition the author drew and produces
nothing).* The check is on the **property**: two operators drawing down the same
budget is the flag, regardless of anyone's taxonomy. Per ratified doctrine the
flag **warns and degrades, never fails closed** — with two obligations the corpus
insists on: the warn is **recorded in world identity** (a fallback that fired is
a permanent fact about that world), and it needs a real **emission channel** — the
headless crates currently have none, every existing validator hard-refuses, and
journal/0065 holds the receipt for a WARN nobody reads becoming a false green.
Designing that channel is part of this tier's first engine slice, not an
afterthought. The satisfiability socket (`pipeline.rs::check_class_satisfiability`
/ `dc-api::validate_pack`) is where the tier's load-time checks live: an operator
whose terms no record carries is *inert, loud*; a record whose family has no
expressor is *inert, loud* (the plug-and-play mirror — notebook § 2.5a).

## 6. Operator kinds

- **The mass-expressor** — spends the family's budget through the ledger. One per
  family *in practice* (the property check above is the enforcement; the count is
  not).
- **Decorators** — read the record and the working state, add or reassign without
  touching the mass budget: placer concentration inside the channel, beaver dams
  on a discharge range, vegetation roughness, speleothem. **Stack freely, in
  authored order.** They reuse the shipped rider discipline: per-decorator salt
  domain + decorator index in the draw address (`fill.rs::PoreDraw` — the
  correlated-rounding defect is already paid for and its cure is a type).
- **Transforms-in-place** — rework a prior unit rather than stacking
  (diagenesis-shaped). The one in-tree instance is `recorder.rs::overprint_top`,
  named "the template" by the 2026-07-22 audit and never generalised; it
  generalises here.

## 7. First members

> **⚠ RULED 2026-07-29 (user, at ratification): each member below requires its own
> dedicated design pass built on these bones, and is revisited in light of this design —
> never taken as wherever it landed prior.** The listings below are the *candidates and
> their evidence*, not specs.

0. **`CoarseField<T>`** — built 2026-07-22, ratified in the strongest language in
   its thread, **zero production callers for seven days** (spines § 3). It is the
   base reconstruction and the two legal coarse→fine moves; adoption (P4) is the
   tier consuming its member #0, and the octaves slot in as a `DitherSource`
   (P6 — the graph already reconciled them: octaves supply the source,
   `sample_dithered` the draw, ~~`summarize` retires the residual bias~~ —
   **`summarize` was ruled OUT of this tier at member #0's design pass**, so the
   residual bias has no heir inside refinement).
   - **✅ DESIGN PASS DONE + FIRST BUILD SLICE SHIPPED, 2026-07-29** —
     `docs/audits/2026-07-29-member0-coarsefield-design.md` (rulings in its mutable
     header), built in journal/0125. **The far site only**, and the member is
     **K1 only**: `sample`/`sample_dithered` + `DitherSource`. The design pass
     found the fused "base reconstruction" is *three* separable kernels and split
     them — K2 `summarize` → the octree node contract, K3 (`collapse.rs::lattice`'s
     midpoint jitter, the oldest coarse→fine move in the tree and **not expressible
     in this document's two-move vocabulary**) filed as **move C — bounded
     stochastic detail synthesis**, with a named heir and no owner yet. *All
     sub-460 m relief in the shipped world comes from move C, which § 3's kernel
     list does not name.*
   - **What the slice proved about these bones, honestly:** the base-reconstruction
     step ran with **no runner** — which is the seam-first order working, and also
     evidence that "initialized by the base reconstruction" (§ 3) is currently a
     description of one call site rather than of a runner. **MM-3 (the working
     sub-cell state has no declared type) is still owed and is now the near site's
     blocker**, exactly as the design pass predicted.
1. **The channel operator** — the worked acceptance example. Inputs: the
   face-flux record + head field (P3 — built and idle *on purpose*, "the
   primitive the channel needs is the one FLOW spent three slices building"),
   the load budget, grain distribution, the mobility hint, and the column
   inventory (the channel incises the regolith the weathering passes actually
   piled — bank shape falls out of material sheets, not a river template).
   Mechanism: boundary-value solve inside the cell, faces as Dirichlet data,
   budget spent along the solution path; deposition graded by distance
   (levee→floodplain fines); scars from mobility + position noise; terraces
   from chapter history. **One operator; the fluvial assemblage is correlated
   views of one record** (user, 2026-07-29: the floodplain operator *is* the
   channel operator). Named limit honored: the record carries a mobility hint,
   never the meander path.
2. **Coal partings** (`ideas.md:454-459`, already scoped): deterministic,
   position-seeded mineral bands inside thick organic units — "the record knows
   something the voxel grid is too coarse to show." Small, decorator-shaped, and
   it exercises the tier without the BVP solver.

## 8. Blockers and sequencing honesty

- ~~**The face-pairing rule** (`flow.md` § 2 flag): under-specified, not ratified,
  a uniform 2× on record cost, and *"resolve before any refinement/expression
  slice; a slice that picks a rule must say so loudly."* This document does not
  pick it; the channel slice must, loudly, as its first act.~~
  **✅ RESOLVED 2026-07-29 — and this bullet was STALE AT ASSERTION (corrections #75):**
  the principle had been ratified 2026-07-25 (`flow.md` § 11.5, 360 lines below the
  flag this bullet echoed, never cross-stamped), mode 1 was already built, and the
  2× belongs to gross-vs-net, not pairing. Re-argued fresh at the fluvial member
  design pass and **CAUTIOUSLY RATIFIED as the three-mode confinement rule** —
  full record in `flow.md` § 11.5's banner. The channel slice rides **mode 1
  (chapter pairing)** and states so loudly; nothing is left for it to pick.
- **Mover attribution** (stubs #25: the record says *what* arrived, not *who*
  brought it): v1 spends the **total** export budget along the flux path;
  the packed `(species, mover)` byte is the named successor when budgets must
  split by process. The conservation law holds either way.
- **`collapse.rs` decomposition** rides this arc (one slice, two obligations —
  E5 → decomp → the file-size chore), but *after* the first members prove the
  shape on new expression rather than by rewriting old.
- **What this doc does NOT decide:** the concrete SDK types (a build slice, after
  ratification); the warn-channel design (named engine slice); which additional
  kernels earn primitive status (bottom-up, from members, per notebook § 2.4's
  method); anything about deeptime pass authoring (unchanged by this doc).

## 9. Compliance

North star: this is § refinement candidate (c) built out — one authoring shape,
engine keeps kernels + runner, content declares everything else; the crossing
constraint holds (declarations are plain data + opaque ids; operator bodies are
backend-compiled code). Spines: S-3 (the family never becomes an authority the
physics reads back — if a deeptime pass ever keys on a family, that is the
summary-wearing-authority defect and a named violation); S-4 (coarse cause, fine
expression — this tier IS the fine half, and Law 2 is S-4 said for taxonomy);
A-4 guarded by building on the built-and-idle index rather than beside it
(CoarseField, flux, head, riders, overprint_top all consumed, not rivaled);
A-7 (no content identity inside the engine — families are pack ids). Deviation 2
(no walls) shapes the validator throughout.
