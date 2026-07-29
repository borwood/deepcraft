# Material behavior — the substrate, forms, transitions, agents, and passes

**Status: ratified in design conversation 2026-07-23** (user, this session). This
is the detailed design of the north-star's **content layer** — the substrate that
material behavior is written *over*, and the execution model that runs it. It is
the spec the **Crux 1** keystone (the deep-cell material inventory) builds toward.

Read alongside:
- [`north-star.md`](north-star.md) — the engine shape this realizes (§ Refinement
  2026-07-23 names the two pass shapes; this doc fills them in).
- [`materials.md`](materials.md) — material *definitions* (the sheet, categories,
  the transformation-axes DECIDED 2026-07-22). This doc is the *behavior*
  companion.
- [`spines.md`](spines.md) — S-1 (bounded derivation), S-4 (coarse cause / fine
  expression), S-8 (one quantity, many regimes), S-9 (derive + facts).
- The fill contract (`ARCHITECTURE.md`, DECIDED 2026-07-21) — this doc lifts it
  one tier.

> blogworthy (lenses: deepsim-reflexions; procgen-against-priors): a substrate
> designed so that the behaviors downstream systems *want to write* — karst,
> weathering, diagenesis, seepage — are expressible without a carve-out, because
> the primitives (forms, a complete transition graph, agents, two pass shapes)
> were derived *from* those behaviors rather than guessed ahead of them.

---

## 0. The design principle

**We build a substrate that solves problems for other designs by giving the
behaviors they would like to write the proper substrate for expression** (user,
2026-07-23). The substrate is proven complete only if the flow / weathering /
dissolution / deposition / diagenesis behaviors named across the corpus can be
written cleanly over it — the "can it carry the defaults" self-validation, applied
to the substrate. Every primitive below was derived from a behavior the hydro /
cave / geology designs already want.

---

## 1. The deep-cell inventory IS the fill contract, one tier up

The keystone gap (recon `docs/audits/2026-07-23-block-consumer-inventory.md`): the
present voxel is already a full material inventory (`VoxelContents`), but the
**deep cell has no working inventory** — it stores R/H heights + an append-only
strata record (its *history*), so a behavior like "weathering consumes bedrock,
produces regolith" has nothing to read-modify-write. S16 had to thin-adapter one.

The fill contract already ratified the shape: *"a column's fill is an **ordered
list of spans**, each carrying (contents, form, fractional occupancy); today's
single-height solid-below is the degenerate case; the caves/water voids are one of
the four threads that forced it."* So:

- **A deep cell's inventory is a stack of `VoxelContents`-shaped spans, indexed by
  depth** (user, 2026-07-23) — a column, not a single mixture. The present voxel,
  the deep-cell inventory, caves (void spans), and the block↔material collapse are
  **one shape at different resolutions.** We are lifting a ratified shape, not
  inventing one.

Two roles, cleanly separated (S-2, S-9):

- **The strata record stays the temporal authority** — append-only *history*, the
  committed facts.
- **The deep cell gains a mutable *working inventory*** — the current composition,
  which behaviors read-modify-write freely.
- **Chapter boundaries commit the working inventory's deltas back into the
  record** — this *is* the deeptime-as-compiler step (a chapter compiles working
  state into world history).
- The **present voxel derives** from record + working inventory via the existing
  collapse path (`deposit_deep_history` → `ColumnFill`) — the deep→present S-9
  base→derived edge is already live.

**Granularity (DECIDED 2026-07-23):** deep-tier spans carry **finer-than-eighth
fractional quantities** (deep behaviors subtract/deposit sub-eighth amounts over
long time; "fractions come only from the ledger"), and **quantize to eighths only
at collapse.**

### Commit semantics — DECIDED 2026-07-24 (user, ratified)

Resolves the S17 seam (the strata record is keyed by depositional *environment*,
not material, so a material change cannot rewrite it):

- **In-place transformation → append a FACT** to the existing unit. A unit becomes
  its depositional base (`DepTag`, immutable) **+ an appended list of transformation
  facts**; current composition = `derive(tag)` then fold the facts. S-9 per unit.
- **Depositional arrival → append a new UNIT** (today's `deposit_deep_history`,
  unchanged). Transport is a *removal fact* here + a *new unit* at the receiver.
- **Facts persist; the working inventory is transient.** Facts are the outcome of
  stochastic, state-reading behaviors — not re-derivable without re-running the
  compile — so they *are* the compiled artifact; the working inventory is compiler
  scratch, re-derived from `base + facts` each chapter (S-2).
- **The fact source is the applied-edge LOG (refined 2026-07-24), not a post-hoc
  diff.** Each edge logs itself as `apply_edge` runs; **the logged edges ARE the
  facts.** A diff of working-vs-baseline recovers the net delta — *equivalent for a
  single edge*, but a multi-edge chapter has **no unique factorization**, so a diff
  loses per-edge provenance. The log is the authority; the diff is only a validation
  check. `commit_chapter` = collect the chapter's logged edges (coalescing identical
  successive ones). **Empty log ⇒ no facts ⇒ record byte-identical** (the S17 identity
  default). *(A2's S17 plea, ratified.)*
- **A fact's shape:** `(chapter, cause, edge, (from-mat, from-form) → (to-mat, to-form),
  fraction)` — where **`cause` is the responsible agent** (deeptime: frost / biotic /
  dissolution / abrasion) **or actor** (present: player / NPC), so provenance is
  specific ("frost-weathering did Y", "player P deposited it"). Apply-time logging makes
  `cause` free: each agent/actor logs its own edge application, so a chapter where
  several agents drive one edge yields **one fact per agent** — the S16 fold sets each
  agent's *share*, not the fact count (preserving "frost did 3, biotic did 2"). Carries
  material change, form-only change (crumbling), and dissolution (`→ void`). Ordered by
  chapter; within a chapter a commutative batch.
  **In tree since 2026-07-25 (journal/0108) the endpoints are held as one `EdgeId`**
  (§3), and the resident fact is **8 B with zero padding** — `chapter: u8`,
  `cause: Cause`, `edge: EdgeId`, `fraction_m: f32`. **Every axis above survives**;
  only the fraction's *representation* narrows, and it narrows **once, at persist**
  (`LedgerField::from_accumulators`) — the gen-time accumulator stays `f64` so the
  per-epoch add never rounds, and every read widens back. Measured cost: max relative
  5.766e-8, at f32's own 2^-24, i.e. one part in 914 000 of an eighth of a voxel.
  This is the shape's compaction lever; the axis-drops that would have bought more
  bytes are the ones this section exists to forbid (S20 § 6, A-1).
- **The fact is the seed; the story is derived-and-displayed** (S-2). The fact stores
  only the non-derivable core above. The **pass** (from `chapter` + `cause` + the static
  schedule — topo-validation forbids two passes writing one edge, so it is unique), the
  **epoch**, and the **environmental driver** ("because heat/pressure/water was thus",
  re-derived from the field state at that chapter) are all **derived at read time and
  shown**, never stored. Same discipline as facts-vs-working-inventory, one tier up.
- **Per-voxel provenance falls out** (Sequenced item d, resolved): "started as X,
  weathering did Y at chapter Z" is just reading `base + facts`. A cave wall records
  how it was carved.

**Provenance addresses the material portion's lineage, not the cell** (user,
2026-07-24 — forward-looking, do not foreclose): when contents *move* — transport in
deeptime; a player/NPC picking up and depositing in the present — the **move is itself
a fact**, and the portion's fact-history **travels with it**: *"this X was placed by
<actor> at <time/place>, after it was picked up at <time/place>, after agent Y
deposited it…"*. The ledger addresses a portion's lineage; a move appends a move-fact
and relocates the address. Not built now; the fact shape must leave room for it.

**Correspondence note (S17 plea #3):** "VoxelContents-shaped spans" is a
*correspondence*, not a shared type — the deep tier is fractional-metres-plus-facts,
the present voxel is eighth-quantized; they share the form/material vocabulary, and
eighth-quantization happens at collapse.

---

## 2. Forms — the closed set of occupancy modes (the machine)

A **form is the *mode* a material-portion occupies volume in** — not the material,
not content. The mesher, collider, gravity, and water all must know how every mode
behaves, so **forms are a closed set the machine owns.** A mod picks a material and
a mode; it never invents a mode.

| Form (mode) | Rule that defines it | Stored as |
|---|---|---|
| **Structure** | coherent, load-bearing framework; reserves capacity; face-culls at ≥ threshold | structure role (a multiset) |
| **Loose** | granular, unreserved volume; **obeys gravity** (falls) | debris role (a multiset) |
| **Pore-fill** | material held inside another's reserved-but-unfilled capacity; **doesn't fall** | pore role (a multiset) |
| **Fluid** | liquid in pores + open space; **obeys settling, not weight** | (water model: bound water derived in a halo; a role the substrate accommodates) |
| **Void** | **not a stored role — the unoccupied complement** (`free_eighths`); edges *to* void reduce occupancy, edges *from* void increase it | complement |

**Occupancy is a separate, orthogonal axis** ("partial") — *how much*, not *which
mode*. A dune edge is **loose at partial occupancy**; a half-eroded wall is
**structure at partial occupancy**. Form × occupancy compose.

**The three zones of a voxel** (why loose and structure coexist without
contending), from `contents.rs`:

- **Structure** = filled reserved slots (`structure_len`).
- **Pores** = reserved-but-unfilled capacity (`capacity − structure_len`) — only
  pore-fill (grain-gated) or fluid may enter; **loose may not**.
- **Unreserved** = `8 − capacity` — loose and fluid live here.

So `Slab(4 stone) + 4 loose = 8` is a valid voxel: loose falls into the unreserved
volume above fractional structure. `free_debris_eighths = 8 − capacity − debris`
is the query the gravity march reads.

**Structure is a heterogeneous multiset** — a voxel may be half basalt, half
carbonate, all structural, no pore-fill. Behaviors act **per-material** within it
(the carbonate dissolves, the basalt stays).

**Future forms are machine additions, not a mod surface.** The set may grow
(capillary film, vapor) — that is *machine* work, and when it happens new
transitions fall out automatically (§ 3). Forms are **not** SDK-registrable: a form
is a mode every part of the machine must reason about, so "add a form the rest of
the system doesn't know exists" is not a coherent content operation.

---

## 3. The transition graph — complete, machine-provided

**Every form→form transition exists programmatically, always, ungated.** The graph
is complete by construction (5 forms → 20 directed edges). `loose→structure` never
waits to be declared; it just exists. **KISS.** Mods never add edges; when the
machine adds a *form*, its edges fall out automatically.

**The edges ARE the catalog of material processes:**

| Edge | Process |
|---|---|
| `structure → loose` | weathering / abrasion (emit loose; **shrinks the reserved shape**) |
| `structure → pore-fill` | **crumbling** (lose integrity in place; stays full; `structure_len↓`, `pore_len↑`, same material — material conserved) |
| `structure → void` | dissolution |
| `loose → structure` / `pore-fill → structure` | compaction / lithification / cementation |
| `loose → pore-fill` | infiltration (fines settle into pores; grain-gated) |
| `loose → fluid` / `fluid → loose` | melting / freezing (snow ↔ water) |
| `fluid → void` | **evaporation** (the honest sink) |
| `fluid → pore-fill` / `fluid → structure` | cementation / evaporite (precipitate into pores) |
| `void → loose` | deposition |

**Weathering shrinks the reserved shape; inherent porosity does not.** Two distinct
facts of the same material: a porous *intact* sandstone reserves 8 and fills 5 (3
pores hold bound water — `structure_density = filled/capacity`, a *formation*
fact); **weathering destroys framework**, pulling capacity down and emitting the
product as loose into the freed unreserved volume. Same model, two behaviors. This
is what makes an eroded wall a `Quarter`/`None` shell with loose come-to-rest and
fluid/pore-fill in the remaining pores — **soft-boundary caves fall out.**

Crumbling rides the existing **genesis exemption** to `fits_in_pores`
(materials.md): crumbling-in-place is formation, not infiltration, so a coarse
stone may fill its own pores without meeting the grain rule.

**IN TREE since 2026-07-25 (journal/0108) — this graph is now the compile-enforced
authority for what a fact may say.** `inventory::is_declared_edge` is the predicate
(complete by construction, as above; the **null edge** `from == to` and `Void → Void`
are the only refusals — nothing moves, so there is no transformation to record).
`EdgeId::declared` is its **only** constructor and a `Fact` can only be built from
an `EdgeId`, so **a fact structurally cannot name a transition this section does not
declare**. The strong form holds too: `InvCtx::apply_edge` on an undeclared edge
returns 0 and touches nothing, rather than performing a move it could not honestly
write down. The 20-edge count above is checked by the gate rather than quoted
(`a_fact_cannot_name_an_undeclared_transition`). The id is two bytes — one per
endpoint, mixed radix `material * 5 + form` — which is also what took the resident
fact from 16 B to 8 B, but the bytes were the consequence and this was the reason.

---

## 4. Agents — rate-term contributors on edges (content)

An **agent** is a *named rate-term bound to an edge*:

> `agent term = environmental driver × per-material susceptibility → added to the edge's rate`

- Frost, abrasion, biotic, dissolution each add a term to (e.g.) `structure→loose`;
  **multiple agents on one edge SUM** — each agent removes its own share of rock per
  unit time, and independent removal rates add.

**DECIDED 2026-07-24 (this session, ratified): agents SUM; byte-identity with the
product world is retired.** S16 kept the *product* arithmetic
(`base × biotic × weatherability × frost × taper`) only to stay bit-identical to the
legacy `erosion::weather` — a scaffold, now discharged. The honest model is the
**sum of agent terms**:

> `rate = cover_taper × Σ_a (driver_a × susceptibility_{m,a})`

A product falsely zeroes frost where biota is zero (frost shatters bare rock), and
only a sum makes each agent's *share* — hence **one fact per agent** (§1) —
well-defined: `share_a = cover_taper × driver_a × susceptibility_{m,a}`, and
`Σ share_a = rate`. The first real behavior through the working inventory (the
weathering slice, this session) adopts the sum and is **NOT byte-identical**; its
acceptance instrument is a **walk**, not goldens. *(velocity over a preserved legacy
number — user, 2026-07-24: "sum is honest and faithful; be brave.")*
- The **material declares its susceptibility per agent** — exactly the existing
  `LithoResistance` axes. The `Agent` set is deliberately closed-at-compile so
  adding one is a checked extension at every material.
- A behavior's read-modify-write targets a **`(MaterialId, Form)` fraction** drawn
  from the multiset — so differential weathering (carbonate dissolves, basalt
  stays) is native.

**Rate reads live state, not just the sheet.** The rate is
`f(material, environment, current cell state, neighbor halo)`. The self-excavation
feedback (trapped water → walls crumble → flow exports pore-fill → path opens)
proves the rate reads the cell's *evolving* state — so a behavior must **run where
it watches its own and its neighbors' state change** (§ 6). The feedback needs
**no special case**: it is the weighted-move model when every edge is expensive but
none is zero — given time, the least-impossible move accumulates and a path opens.

---

## 5. Passes — two shapes: cellular and field

A pass declares itself (`{reads, writes}`, cadence, epoch) and runs through the
pass-runner (~~topo-sorted~~ **authored order, validated** — superseded 2026-07-26,
see § 5's ORDER bullet and corrections #65; cycles still rejected). Two shapes:

- **Cellular passes RUN EDGES** — move material between forms in the inventory.
  Output = **changed material state**. Local (bounded halo). **Agents fold into
  edge rates.** (S16's `WeatheringPass` is the first instance.)
- **Field passes COMPUTE FIELDS** — a value per cell (uplift, precip, temperature,
  flow-accumulation) via a global / numerical solve calling **core solver
  primitives**. Output = **environment planted as cell/world API state**. They
  **never run edges; they don't touch the form inventory.** No agents (no edges to
  fold onto) — they have data-tuned parameters.

**The reciprocal loop (this is S-4):** field pass reads material → computes a field
→ plants it → cellular pass reads the field → runs edges → changes material → field
pass recomputes → … Coarse cause (the field), fine expression (the edges), iterated
over chapters.

|  | Cellular | Field |
|---|---|---|
| Body | runs edges | global/numerical solve |
| Reads | fields + material | material |
| Writes | **material** | **a field** (API state) |
| Locality | local halo | often global (drainage = sole advect) |
| Agents | **yes** | no |

**Processes that seem to be both are split** into a field half and a cellular half
— the move the corpus already uses for drainage+erosion:

- **Drainage** (field: `recv`/`area`) + **transport** (cellular: runs edges
  *following* the pinned receiver — **cannot re-route it**, preserving C1 / 3e-2
  decision 1: drainage is decided-once-coarse and is the sole advect).
- **Tectonics** (field: uplift/deformation) + **deposition/accretion** (cellular).

The advective non-locality lives in the **field**; the cellular pass runs edges
locally along the pinned receiver. That is how "transport is the sole advect" and
"cellular passes are local" hold at once.

### Scheduling: fold vs pass vs monolith

Two questions hide in "agents as passes vs all-in-one":

1. **Do co-determinant agents fold into one rate?** Yes (S16). Not a scheduling
   question — they are terms in one rate.
2. **Should distinct transitions be separate passes or one monolith?** **Separate
   passes**, decided by **coupling timescale vs cadence**:
   - coupling coarser than cadence → separate passes; the **chapter/tick loop
     carries feedback** (the loop *is* the relaxation); the pass-runner runs them
     in declared DAG order each step.
   - coupling finer than cadence → **fuse that pair** (a *measured* exception).
   - The pass-runner **rejects cycles by design** — this is the feature that forces
     one of those two correct outcomes; a cross-pass feedback (weather↔flow) plays
     out across chapters, not within a DAG.
   - Separate passes let each declare its **own cadence** — honest for feedback
     (nonlinear) processes, where a monolith's shared-cadence-with-scaled-rates is
     only equivalent for *linear* ones.

**The moddability tiebreaker (settles the default):** a mod adds a process by
*declaring* it, not by patching a fold it cannot see. A monolith has no declaration
surface. So the default is **separate declared passes**; S16's fold lives *inside*
each pass.

### Cadence: order × rate × window — RECONCILED 2026-07-24 (promoting `ideas.md § Pass cadence`, user); **third axis added 2026-07-25** (transcribing `flow.md` § 11.1, RATIFIED)

The scheduler has **three orthogonal axes**, and the runner declares **all three** per pass:

- **ORDER** — ~~derived from `{reads, writes}` by **topo-sort**; rejects cycles,
  conflicting writers, missing deps. This *replaces* a hand-declared "canonical
  order": the order falls out of the declared dependencies and an illegal schedule
  is **caught**, not trusted.~~
  **🔴 SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD.** See
  `ARCHITECTURE.md` § *The engine is plugin-agnostic, and pass ORDER is authored*
  (DECIDED) and **corrections #65**. The struck clause is kept rather than deleted
  because it is cited elsewhere and because **its falsity is the instructive part**:
  the order never did fall out of the declarations. journal/0090's own summary says a
  linear relaxation pipeline *"does **not** fall out of a dataflow graph for free —
  you have to name each **revision** as a distinct resource for the topo-sort to
  reproduce a fixed sequence."* Those seven revision tokens **are** the hand-declared
  canonical order, re-encoded so the graph appears to compute it — and re-encoding it
  in an engine-owned enum (`DeepAxis`) is what put the default pack's pass roster
  inside the engine.
  - **The replacement:** order is **data on the world**, chosen per world beside seed
    and epoch count. `{reads, writes}` become the **validator**, not the generator —
    unwritten resources, reads satisfied only by a later pass (an implicit lag, to be
    made explicit), genuine cycles. Every rejection class in `passgraph.rs` survives;
    only the claim that order is *derived* is withdrawn.
  - **This restores the user's 2026-07-23 sketch** (`ideas.md` § *Pass cadence*), whose
    ORDER half — *"a canonical start order (tectonics → hydro → weathering)"* — was
    reconciled away rather than contested. RATE, below, was the half that survived.
- **RATE** — the **fractional-phase phase length**: how many sub-turns a pass takes
  per chapter, and the **`dt`** that scales its transformations. A chapter subdivides
  into sub-turns; a **high-rate** pass (weathering ×5) sees mid-chapter state evolve
  while a **low-rate** pass (tectonics ×1) runs once — the temporal-resolution knob
  ~~topo-sort~~ **the ORDER axis** alone does **not** give. This is `ideas.md`'s
  fractional-phase sketch, reconciled: it is the RATE axis, *composed with* ~~topo-sort~~
  **the authored ORDER**, never replaced by it.
  *(Struck 2026-07-29 — the last un-struck `topo-sort` in this section, twenty-three lines
  below its own `🔴 SUPERSEDED` banner at § ORDER. Reported by the 2026-07-26 doc-topology
  sweep as its finding 15, and again by the baseline sweep S3/F3; corrections #65's site
  list is amended. **RATE itself is untouched and survives** — it was always the half that
  survived the reconciliation.)*
- **WINDOW — the aggregation window: how many epochs sum into ONE record entry.**
  RATE is a *sampling* rate (how often the pass fires); WINDOW is the record's *time
  granularity* (how coarsely what it produced is stored). They are different knobs and
  firing less often does not merge entries — it loses epochs. `flow.md` § 11.1 is the
  ratifying decision and § 2.6 the measurement behind it:

  > **A window that decides an acceptance number must be declared, not assumed.**

  The case that forced it: the flow record's slice-1 divergence count (**175,320
  divergent `(cell, chapter)` pairs, 7.378 %** — the number the slice was accepted on)
  was a product of an **implicit** constant, one tectonic chapter = 25 epochs. At a
  one-epoch window that count is **zero**, because within a single epoch the solve
  returns one receiver per cell and the tree structure reasserts. Set the window and you
  set the number.

  **The user's reasoning, recorded:** *"freedom to future mods / ourselves (we are the
  first modders)."* A mod authoring a pass must be able to state its own record
  granularity the same way it states order and rate — **an undeclared constant is exactly
  the surface a third party cannot reach.** This is the north star's *"authored in a
  uniform, self-declaring, compiler-validated shape and tuned by data"* applied to the
  **time** axis.

  **Resolution is a shipped DEFAULT, not an engine property** (`flow.md` § 11.2, RATIFIED):
  once the window is declarable, per-chapter vs per-epoch stops being an architecture
  question. Default to the **cheap end** (the coarser window) for dev-iteration speed and
  **expose the knob** for stress tests.

  **RIDER — the self-describing-record contract** (`flow.md` § 11.3, a **standing** contract
  on every future record, not a flow-only rule): *any mode that changes what an **absent**
  entry means must be carried **in the record***, never held as external knowledge.
  Otherwise absence is ambiguous across worlds and every consumer must know how a world was
  generated in order to read it — **S-9 one level up: the answer carries its resolution.**
  A window is exactly such a mode, which is why the two decisions arrived together.

  **Status: declared here, not built** — no `Pass` carries a window field today, and today's
  one aggregating record (`DeepField::flux`) buckets by chapter as an implicit constant.
  Sequenced on the ROADMAP as *"THE AGGREGATION WINDOW IS A DECLARED AXIS"*.

**`dt` = phase length is not new machinery** — it is exactly the `rate × dt` time-base
S16's behavior model was already written against; today `dt` is pinned to `1.0`, and
the fractional-phase clock makes it a real **per-pass** knob. So "raise a pass's rate
for finer feedback" is the concrete form of the "own cadence" this section already
endorsed, and the alternative to **fusing** coupled passes (a fused pair is only the
right answer when their coupling is finer than the *finest* rate available).

**Determinism** holds: fixed rates → a fixed sub-turn schedule; all draws from
caller-owned seeds. Runtime note: this is the **deeptime** clock — the present VM is
event-driven, no fixed cadence.

**Live fork (named by the sketch, unresolved):** agents SUM in one pass at one
cadence (§4, DECIDED) for *co-cadence* agents; agents with genuinely **divergent
natural rates** (frost seasonal vs dissolution slow) are the case cadence would split
into separate passes at separate phase lengths — the "agents as terms in one pass vs
agents as passes with their own cadence" fork. A **measured** call for when those
agents land, not now; the sum decision holds and cadence marks its boundary.

---

## 6. The behavior execution shape

Composing §§ 4–5, a cellular behavior is:

> **read → fold agent terms → write**, over a **`(MaterialId, Form)` fraction**,
> with rates reading **material × environment × live cell state × neighbor halo**,
> **iterated** so it watches its own and its neighbors' state evolve.

That is a **bounded local relaxation** (S-1) over the **form-transition graph**
(S-8) — the same shape the corpus found for bound water, weathering, and
structurally for light. Flow is **least-resistance move-selection over the neighbor
halo** (3D — lateral and vertical, inside voxels), weighting candidate
`(neighbor, move)` pairs by resistance × state and choosing stochastically.

**Determinism:** the per-step update is synchronous (read old state, compute, write
new state), so it is order-independent by construction — the S9b/S11/light family.
All stochastic draws come from **seeds owned by the caller** (no wall clock).

---

## 7. The substrate's three access channels (the `ctx` as capability)

A pass's `ctx` exposes three channels with three different powers (north-star:
"`ctx` is a capability, not a god-object"):

1. **In-cell span state** — material · form · fraction. Cellular passes **mutate**.
2. **Pinned coarse fields** — drainage `recv`/`area`, water-table surface, uplift,
   temperature. Cellular passes **READ**; field passes **WRITE**. (`recv`/`area`/
   `lake` are already built and populated, consumed by nothing — the boundary field
   transport needs is waiting: spines § 3.)
3. **The free-water body graph** — non-local connectivity. Behaviors **QUERY**
   ("is this cell below the table / in a flooded body"), never store an edge. A
   channel joined to a river *is* the river (corrections #14); a spring is a
   derived outlet, not an authored edge.

**Two things stay out of the inventory** (S-2, "store only what derivation cannot
predict"): **free water** (body graph, derive voxels) and the **transport load**
(suspended/dissolved sediment — pass-transient, carried down the receiver chain
during a pass, not a per-cell field between passes).

---

## 8. Two runtimes: deeptime compiles, present executes

The two clocks are two *runtimes over one set of declarations*:

- **Deeptime = ahead-of-time compiler** — seed + config + declarations → compiled
  cell state + ledger; gen-time is free; behaviors run as **bulk population folds
  over chapters**.
- **Present = the live VM** — loads the compiled world and re-invokes the **same**
  declarations sparsely and event-driven; runtime is sacred; behaviors run as
  **live per-voxel events**.

**One declaration, two executors:** a material's `combust→` / `weather→` runs as a
bulk transform in deeptime and a live event in the present — the declaration is the
single authority (S-3), the runtimes are two interpreters of it (materials.md
DECIDED 2026-07-22). *(The define-once-run-in-both duality is validated at the first
runtime-process milestone; no runtime process sim exists yet — only block edits.)*

---

## 9. Ownership — two machine layers, three content layers

| | Layer | Owns | Extended by |
|---|---|---|---|
| **Machine** | **Forms** | the closed set of occupancy modes | machine only (rare: a new form) |
| **Machine** | **Edges** | the complete graph of all form→form moves | machine only (auto from forms) |
| **Content** | **Materials** | property sheet + per-agent susceptibilities | mods |
| **Content** | **Agents** | a rate-term on an edge (driver × susceptibility) | mods |
| **Content** | **Passes** | schedule edges at cadence/epoch, `{reads,writes}` | mods (rarely) |

Most content is **materials + agents.** "Acid-rain weathering" is a new *agent* (a
term on `structure→loose`, riding the existing weathering pass's cadence), not a
new pass. A mod adds a *pass* only for a genuinely new *schedule*. It never touches
forms or edges.

---

## 10. Open questions and deferrals

- **Void as a stored role vs the complement — RESOLVED as the complement** (§ 2):
  void is `free_eighths`, not a 4th role; edges to/from void change occupancy. Water
  occupies it either way. (This resolves the 2026-07-23 "yikes I don't know.")
- **Coupling timescale vs cadence** (§ 5) is the load-bearing assumption behind
  "separate passes, loop-carried feedback." It is ultimately **measured, not
  decided** — the first fusion exception, if any, must be justified by a coupling
  finer than its cadence.
- **Fluid as a stored role vs derived** — bound water is *derived in a halo* (S11);
  whether the working inventory ever stores a fluid role, or always derives it, is
  tied to the water model. The substrate accommodates a fluid form; the storage
  question rides the water design pass.
- **Deep-cell span granularity** (per-collapse-voxel vs per-stratum) — a
  measurement call; the `ctx` stays granularity-agnostic so it isn't accidentally
  deeptime-only.
- **Entity substrate / actor-stepping** (DF civ, agents-as-mobs) is a *second
  substrate* and a *third pass shape*, deferred (north-star § Refinement). A
  cellular shim spawner covers mobs today.
- **Dissolution is carbonate-gated** (C7): every material's `solubility = 0.0`;
  karst waits on the carbonate milestone. The dissolution *agent* exists and is
  dormant.

---

## 11. In-tree embryo (what exists to build from)

| This doc's piece | In-tree today |
|---|---|
| present-tier inventory (roles, multisets, occupancy primitives) | `VoxelContents` (`dc-core/src/materials/contents.rs`) — 3 roles (structure/pore/debris) |
| first cellular pass (agent-fold, pure behavior + apply) | `WeatheringPass` (`dc-worldgen/src/deeptime/weather_behavior.rs`, S16) |
| deep record = temporal authority | strata record / `deposit_deep_history` |
| deep→present derivation | `ColumnFill` |
| field-pass outputs, planted | drainage export `recv`/`area`/`lake` (built, **unconsumed** — spines § 3) |
| fill contract (span list) | `ARCHITECTURE.md` DECIDED 2026-07-21 |

**~70% embryonic; the seam-first march is the path.** The first buildable slices:
`Block={Air,Material}` + solidity→occupancy drain (present tier, ~80% done); then
the deep-cell **working inventory** as a mutable span-list (the keystone); then the
**first real cellular behavior** — subaerial weathering run as a sum-agent pass over
the working inventory, committing cause-carrying facts the present collapse folds
into contents (this session).

**Continuation slot (loose end, annotated 2026-07-24):** the first-behavior slice
runs weathering on the inventory as a *material-transformation* layer and leaves the
existing `erosion.rs` `R`/`H` height loop in place (they compute different things —
height budget vs material composition — so this is not a summary-beside-authority).
The **unification** — retiring the scalar `R`/`H` planes into the inventory, with
`H = Σ Loose` and `R = Σ Structure` *derived*, so there is one authority for both
height and composition — is the reserved next arc. It is entangled with the shared
erosion loop (multiple phases mutate `R`/`H`), which is why it is a slice of its own,
not folded into the first behavior. Heir home for the annotation: this section + the
ROADMAP continuation slot.

---

## 12. Creation & ownership — working direction (STRONG LEANING, **NOT RATIFIED**)

> **STATUS (2026-07-24).** This section is a **strong working direction, not a ratified
> decision.** It is recorded — with its reasoning, not just its conclusion — so the
> genesis-pass work is designed against a captured frame rather than re-derived cold
> next session (the slice-of-sequencing / corpus-outruns-the-assistant discipline). It
> is **open to revision by ongoing decisions.** Provenance is marked per claim: *(user's
> strong leaning)* = the user's own conclusion; *(framing)* = the assistant's framing,
> a strong lean the user endorsed as a direction but did not stamp final. Do not treat
> any of it as a §-DECIDED until it is promoted to one.

**The organizing principle: a transition is owned by whichever end can own it.**
Creation and transformation are opposite directions, and they are owned at opposite ends:

- **Transformation → input-owned.** The subject already exists and is being changed, so
  **the edge lives on the input material.** Weather, oxidize, combust, dissolve, crumble,
  metamorphose. `wood` owns `combust→`; `limestone` owns `dissolve→`. *S18's weathering is
  the first live instance* (bedrock owns `weather→`, gated by agents). combust/oxidize are
  siblings — more edges + agents on the same shape, "add a term, not a mechanism."
  ***(user's strong leaning: transformation edges live on the material being transformed.)***
- **Formation → output-owned.** The subject does not exist yet, so there is no input to
  own it; **the output material declares its own formation predicate** ("I form when
  [conditions]"), read by the genesis/emplacement pass. Emerald owns its conditions. This
  is the "exception" that isn't one — it's the only end that *can* own a creation. *(framing.)*
- **Recipe → registry-owned.** Multiple inputs, possibly multiple outputs, no single
  subject → a **separate recipe registry keyed on ingredients** (crafting; bracketed —
  does not exist yet). Answers "which material do you store it on?" with *none of them*.
  ***(user's strong leaning: recipes get a separate registry, NOT an on-material edge.)***

**Separate by concern — do not thread all creation through one API** *(fork resolved
toward separation)*. Reasons: (a) a material may have **multiple genesis methods** (natural
glass via a formation predicate; crafted glass via a recipe) so it can't own *the* one
genesis; (b) multi-ingredient recipes have no owner material; (c) new creation kinds
(bio-events, etc.) drop in as separate concerns; (d) it costs **no** north-star uniformity —
these are genuinely different shapes (one-input-one-change vs one-output-one-predicate vs
many-in-many-out), and forcing one API would bloat the edge with optional multi-input
fields — the *over*-unification version of the re-invent-next-door sin. Uniformity holds
**within** each kind. *(framing.)*

### The transform/formation discriminator — DECIDED 2026-07-25 (user)

> **🔖 OPEN EDGE — see [`material-genesis-notebook.md`](material-genesis-notebook.md).**
> The conversation that produced this section ran on past it into **pass purity**,
> **term-keyed edges**, **per-parameter shadowing**, and the **fluid/solute shape** — none
> of which is decided, and one of which (fluid identity) would change how every
> precipitation example is bound. The notebook is the dog-eared page.

> **~~is a specific solid material consumed, or does the product precipitate from something
> diffuse?~~** — **SUPERSEDED 2026-07-25.** The phase test was kept as long as it was because
> it agrees with the resolution test on the examples originally chosen. It is retired as *the*
> discriminator and survives only as a **heuristic**, because it makes a claim about **nature**
> that is not true: the silica in pore cement is exactly as causal as the granite in a boulder,
> and ion-scale processes are processes everywhere. It also cannot decide the case it most needs
> to — metamorphic replacement is solid↔solid to us but molecular recrystallisation with ion
> exchange in fact, so "did it come from a fluid?" has no clean answer.

**THE DISCRIMINATOR IS RESOLUTION, NOT PHASE. Everything in the natural world is the result
of process — there is an infinite regress of prior states of matter. A creation predicate is
therefore never a claim that causation stopped; it is an honest admission that OUR MODEL
bottoms out here.** *(user, 2026-07-25 — the reasoning is the user's; this section transcribes
it.)*

> **The test: can the prior be named as a material we track?**
> Not *"is the prior diffuse"* — **"is the prior below the resolution of the material
> ontology."** The material concept is itself a **sieve**: some origins genuinely ARE
> transformations, but transformations of the **molecular parts** of materials, and our model
> has no molecular or atomic parts. A crystalline inclusion in rock is not `rock→crystal`.
> That, and not phase, is the whole case for genesis passes.

**THE FOUR-WAY TEST.** Ask of any material: *what did it come from, **in the ontology we intend
to have**?* There are exactly four answers, and the third and fourth are the ones people
conflate:

| | answer | shape | today's examples |
|---|---|---|---|
| **1** | "that material, transformed" — **driver exists** | **transform edge, input-owned** | bedrock→saprolite (S18) |
| **2** | "that material, transformed" — **driver not built yet** | **transform edge with a SEAMED driver** — declare the edge *now*, socket the driver as a provider with an identity default and a named heir | peat→coal; quartz cement from solute |
| **3** | "that material, **moved**" | **transport** (§13) | sand off a granite ridge |
| **4** | "no material — it formed under conditions, from a sub-material prior environment" | **genesis / output-owned formation predicate** | the t=0 basement |

**CATEGORY 2 IS THE ONE A PROJECT-IN-PROGRESS ACTUALLY LIVES IN, and omitting it is how the
taxonomy goes wrong.** Not every transformation process is modelled yet — bio, solutions in
fluids, volcanism/magma are all outstanding — and a stated intention to model the driver
later means the thing is a **transformation with a seam**, *not* a genesis. We have inherited
several (peat / coal / charcoal). *(user, 2026-07-25.)*

> ### The pathology: filing a (2) as a (4). This is the irreversible one.
> Writing *"coal forms when temperature > X"* as a **formation predicate** means the peat→coal
> **edge does not exist** — and with it go the identity chain, the mass chain and the
> provenance. You cannot later "add the driver", because there is no edge to drive; you would
> have to re-derive what was never recorded. **This is A-1 in this domain** — a creation
> predicate becoming the authority for something that is really an edge — and it is seductive
> precisely because *"it forms under these conditions"* is unfalsifiable in a way *"it came
> from that rock"* is not.
>
> **The discipline: when the prior is nameable but the driver is missing, DECLARE THE EDGE AND
> SEAM THE DRIVER.** Never reach for a formation predicate to paper over an absent system.
> This is just *"write the seam, not the value"* applied to the genesis/transform split, and it
> pays the same dividend: the unbuilt system's obligations become readable at its call sites.

**THE FLOOR MOVES — genesis is relative to a declared ontology, not permanent in general.**
The floor sits wherever the material ontology bottoms out, and it **rises as systems land**.
`flow.md` § 0's regime table already names **`solute`** and **`melt`** as loads in the flow
family, and § 13.3's load is a multiset of `(material, quantity)` covering *"suspended/**dissolved**
sediment"* — so **two of the three examples this section originally gave as formation (cement in
pores, primordial rock from melt) are category 2, not category 4.** They are transformations
whose driver is unbuilt.

- **Drivers already declared intended are PRESUMED SEAMED** — solute, melt, biomass. If a
  driver is on the books, the honest filing is a transform edge with a seam.
- **It is still a per-pass conversation.** Each pass must honestly answer whether its prior can
  be represented with a driver **we should model in the future**, or one **we do not intend to
  model**. The presumption resolves the common case; the design pass resolves the rest.
  *(user, 2026-07-25: recorded here **so a future session is not prompted into re-deriving it
  every time.**)*
- A category-4 pass should therefore **declare its floor and its conversion condition**, the way
  a stub declares an heir — *"genesis relative to an ontology with no X; if X becomes tracked,
  this becomes a transform edge."* **Unlike a stub, that conversion may legitimately never come,
  and then it is not a debt** — it is a stated ontological boundary.

**TWO KINDS OF GENESIS, and only one of them is permanent.**

| | **temporal dropoff (t=0)** | **sub-material prior** |
|---|---|---|
| why it is honest | the prior is **pre-record** | the prior is **below the ontology** |
| when it runs | once, at t=0 | throughout the ~500 Myr |
| honesty requirement | **plausible and self-consistent** — it may be procedural, because there is no record to read | **must read recorded conditions** — a record exists, so a predicate that ignores it is a proxy |
| permanent? | **yes, by construction** | **no — the floor moves** |

This is why `stubs.md` § *"Genesis (permanently legitimate — affirmed, not defects)"* contains
**only** t=0 and boundary items (basement + plate seeding, `corner0`, border wilds, the
sea-level datum). That list is not missing its sub-material entries; it **correctly excludes
them**, because only the temporal dropoff is permanent.

**THE GUARD — genesis reads fields; a proxy reads tags.** A formation predicate consuming real
recorded conditions (temperature, head, grade, host composition) is genesis. One consuming
`earliest-chapter + Subsea + Craton` is **a stub wearing a formation predicate's paperwork** —
which is what `ores.md` § 8 already says of banded iron in as many words (*"STUB for chemistry"*,
heir = epoch-indexed paleo-ocean chemistry), while filing bog iron's microbial step as
*"legitimately coarsened into the facies — heir: none needed for honesty."* **`ores.md` drew
this line per-mechanism before it was a rule.**

**THE COMPLETENESS TEST (RATIFIED 2026-07-25, user).** Every material in the world must be
traceable to one of: **(a)** the t=0 inherited basement · **(b)** a formation predicate reading
recorded conditions · **(c)** a chain of transforms and transports from (a) or (b) — where a
link in that chain may be **seamed** (category 2) provided the *edge* is declared. **Anything
else is a stub wearing genesis's clothes.** *Traceability is to be audited first; the four-way
classification audit follows once the declarative-pass migration has moved the out-of-band
cases (see below).*

**AN ORTHOGONAL AXIS — do not confuse a migration with a defect.** Whether an origin is
correctly *classified* is independent of whether it is correctly *shaped*. Peat / coal /
charcoal are **correctly classified** as transformations but **live in out-of-band code**, not
the declarative pass shape; their move is **north-star convergence work**, not a taxonomy
problem. *A correctly-classified transformation in the wrong shape is a migration; a
misclassified one is a defect.* *(user, 2026-07-25.)*

**What the edge graph still gives you for free.** §3's edge types remain a fast structural
hint — **edges *from* fluid/void** lean formation, **solid↔solid** edges lean transform — but
the hint is now subordinate to the resolution test, and where they disagree the resolution test
wins.

**Scope now = geo, and geo is entirely on-material in two directions** *(framing, following
the user's scope call)*: a geo material declares its **formation predicate** (output-owned)
and the **transform edges** it plays into as input (input-owned) — **no registry needed for
geo.** The recipe registry is a *crafting* concern; bio / organic / liquid / gas are
bracketed (no water yet; magma later, possibly several kinds). Bronze simply has no geo
formation predicate — it is absent from geo and appears later in the recipe registry.

**Distribution emerges from formation predicates over local conditions** *(framing)* — so
emerald ≠ garnet ≠ ruby with no even sprinkle and no co-location, because their conditions
differ (emerald needs Be-meets-Cr, a rare conjunction). As rich as the geo state we track
(metamorphic grade, host composition, trace-element proxies); seam-first, enriched as packs
demand. Priors: `ores.md` (lode fields, veins, drainage-export placers) already owns this
territory.

**Substance-kind is derived, not tagged** *(framing)*: metal / crystal / organic / … is a
**named region of property space**, taken as the argmax of the material's properties against
category prototypes (spines S-3 — a categorical answer is the argmax *of* the sample, never
a field stored beside it), so the tag can never contradict the numbers. Crafting keys on
**leaf** (specific: `mithril`) + **derived kind** (generic: "any metal smelts"). Pure
game-fiction categories with no physical-property signature (enchantable, edible) stay
**declared tags**. The carrying axes (luster, tenacity/ductility, conductivity, transparency)
are cheap author-now/consume-later data, honest-default like `solubility`.

**Materials-are-minerals** *(framing — the LEAST settled, most revisable claim here)*: the
lean is that first-class materials are **minerals** (leaves authored by properties + a
formation rule), and named rocks (granite, sandstone) are **derived mixtures-in-forms**
(granite = a mineral assemblage; sandstone = quartz-in-structure + cement; "sand" = quartz
loose). Genesis produces a **new identity only at a real mineralogical change**; form changes
(sand→sandstone) keep the same mineral. Strong lean because the mithril test falls hard for
it (*"the simulation never met a stand-in"*), but flagged as the least settled — the roster
refactor it implies is large, and it rides seam-first, not a day-one rewrite.

*Provisional illustrations (NOT committed syntax):* `combust→` (wood→charcoal), `oxidize→`
with replacement modes (structure→pore-fill / structure→loose / material change) — dreamt-up
examples of the input-owned edge shape, kept only as illustrations.

---

## 13. The erosion cycle as material-aware transport (design, 2026-07-24)

> **STATUS.** The **R/H unification** and **material-aware transport** are **ratified this
> session** (user; scratch-first reconcile, identity-travels, "not scared of a balloon —
> the anonymous reconcile would be a stub"). The **entrainment/deposition mechanics** below
> are the assistant's framing, **user-endorsed as "sound and promising"** — a strong design
> direction, refine on build. The **three deferred layers** are named future work, not this
> arc. This is the design a Movement-2/3/clastic-4 build brief points at.

**The cycle, end to end** — one coupled loop, all knobs derived from properties we already
have (`grain_size_mm`, `density_kg_m3`, `cohesion`):

> **weather** (structure→loose, supplies material) → **entrain** (lift loose into the load)
> → **carry** (down the flow field) → **sort** → **deposit** (drop downstream) → the fresh
> bedrock exposed underneath weathers faster (self-limiting feedback).

### 13.1 Flow is a FIELD; transport is a CELLULAR pass that walks it
Drainage is a **field pass** (`recv`/`area`, pinned once, the non-local structure).
Transport is a **cellular pass** that carries a **transient load** cell-to-cell *in
downstream order along the pinned receiver* — each cell's op is local ("read my load +
capacity, entrain/deposit, hand the rest to my receiver"), but the chain moves material
arbitrarily far downstream **within one pass.** "Transport is the sole advect" + "cellular
passes are local" hold at once: locality is per-cell, the global structure is the field.
The current `erosion.rs` already does this scalar-ly; we add **identity**.

### 13.2 Transport is a FAMILY, not a water thing
"An agent moves material along a driving field." Water (drainage), **wind** (wind field),
**ice** (glacier), **gravity/mass-wasting** (slope + cohesion) — all share the *same*
load/entrain/sort/deposit machinery, differing only in **their field and their competence
curve.** Wind's competence ceiling is *low* (silt + fine sand, never gravel) — so **loess
and dunes fall out of the identical mechanism**, and the existing eolian agent becomes
"transport with a wind field," not a bespoke system. **Tectonic drift is NOT in this
family** — it is bulk advection of whole crustal columns (the tectonics field pass), a
different scale.

### 13.3 The material-aware load
The load is a **multiset of `(material, quantity)`** — a suspended inventory riding the
chain, pass-transient (§7). Entrainment adds specific materials; deposition drops specific
materials; identity travels between source and receiver.

### 13.4 Entrainment (Hjulström) — the pickup, mirror of deposition
`τ_c(material)` = the energy to lift a grain, **U-shaped in (velocity vs grain size)**:
sand lifts easiest; **finer rises (cohesion binds clay); coarser rises (weight).** Derived
from `grain_size_mm` (curve) + `cohesion` (fine-side rise). Per cell, a loose material lifts
iff `E > τ_c`, **fines-first when energy is marginal** (selective entrainment / winnowing).
Consequences: **lag / armoring / desert pavement** (fines stripped, coarse left);
**cohesive persistence** (mud, hard to re-lift once settled, forms lasting beds). **Only
LOOSE entrains** — removing **structure** (bedrock) is *incision* (detachment), which needs
**weathering to make it loose first** (the weathering↔transport coupling).

### 13.5 Deposition — capacity + competence + a settling sort
Two limits, both needed: **capacity** (`C ∝ discharge × slope`, total mass) and
**competence** (the size/density ceiling energy `E` can suspend). The load is kept sorted by
**settling velocity** `w_s(material)` (rises with grain size + density, Stokes→drag). As `E`
falls downstream the ceiling lowers monotonically and **progressively finer material rains
out** — gravel in steep reaches, sand on the fan, mud in the still lake. **Sorting is the
falling ceiling; we write the ceiling, not the sort.** Payoffs as consequences: **facies**
(where each grain settled), **placers** (dense gold → high `w_s` → drops with the coarse
fraction in the gravel), **provenance** (grains carry their source lithology's identity).

### 13.6 R/H unification + the reconcile
The **inventory is the single authority**; **R = Σ Structure**, **H = surface Loose = the
Loose above the topmost Structure** (a *positional* query, NOT a whole-column sum). **Cave
fill is valid loose but NOT H** — it is loose *below* the first structure; the inventory
distinguishes surface regolith from cave fill by position, which scalar R/H cannot (the
unification is *more* honest, per the user's cave question). **Scratch-first reconcile**
(ratified): the transport pass mutates the working-inventory surface-loose spans (fast
scratch), committing net facts at the boundary — **entrainment = `loose→load` removal at the
source; deposition = `void→loose` arrival at the receiver; cause = the mover
(fluvial/eolian/…).** The **derived `H` plane** is materialized from Σ surface-loose at pass
boundaries for height-only passes (no hot-loop regression — the seam-first "materialize at a
pass boundary" rule). Gen-time cost only; the present VM never runs transport.

### 13.7 This IS clastic sedimentary genesis
Deposition-with-sorting is *how sedimentary material comes to be where it is*, so this
**delivers most of Movement-4 genesis for clastics as a consequence of the erosion loop** —
the `DepTag → reference_material` shortcut half-dissolves (deposited material = what was
transported and sorted here, not a tag lookup). Weathering makes the *source* honest;
transport makes the *deposited* material honest.

**Extended one link back, which settles "who owns clastic facies" (DECIDED 2026-07-25, user).**
On §12's four-way test, clastics answer **"that material, *moved*"** — sand is granite that
went somewhere. So **transport owns clastics**, and genesis owns the **bedrock the sand came
off**. The two arcs were never competing for one output; they sit at **different points in the
same regress**:

> **Genesis makes the *parent* honest · weathering makes the *loosening* honest · transport
> makes the *destination* honest.**

The genesis-passes arc therefore does not shrink — it **relocates**, from *"which rock is at
this facies"* to *"which rock was emplaced here in the first place"*, and it inherits
**stubs #16** (bedrock materialized as one flat basement span) as its first real customer.

### 13.8b BUILT 2026-07-26 — the first slice (journal/0110)

**Shipped, on by default** (`DeepConfig::material_transport`). Off is the scalar-load
solve byte for byte; the pre-2b goldens are still reachable and still asserted
(`tests/material_transport.rs`).

- **The load is a multiset of `(Litho, quantity)`** — seven species, which is the
  material granularity deep time can distinguish at all (`lithology.rs`: *"not the
  material registry — the handful of classes the record can distinguish"*).
  Resolving it finer would be inventing identity the tier does not have.
- **Identity travels**, § 13.6's scratch-first reconcile: entrainment takes the
  cell's own near-surface composition (the `outcrop_shares` seam — one window walk,
  now with two consumers), incision takes the composition of an **empty** section
  (which is how the pass asks "what is below the record" without naming a rock).
- **Deposition is the falling ceiling.** Competence — `w_s > COMPETENCE_SCALE ·
  (REFERENCE_KT / k_transport) · cap` — rains a species out wherever it is; capacity
  then draws the excess **coarsest-first**. Nothing is sorted; the ceiling falls and
  the load is what is under it. `COMPETENCE_SCALE = 420` is anchored on the *shipped*
  facies rule's own Low/Medium capacity boundary (`energy_band`'s `0.002` at the
  reference coefficient) crossing coarse clastic's `settle_energy` (`0.84`) — not a fit.
  **The `REFERENCE_KT` term arrived 2026-07-26 (corrections #59, journal/0114) and the
  ceiling it produces at the historical `k_transport` is bit-identical.** `cap` is
  `k_transport · A^m · S^n`, so an *absolute* capacity threshold is a rate constant
  times a position in the drainage network, not a physical statement about grains —
  and the erosional calibration moves `k_transport`. Stated absolutely, it would have
  classified every depositional site on the world as High energy and carried basement
  to the sea. The thresholds describe **where in a network you are**, so they scale
  with the coefficient.
- **The sweep runs LAST**, on the load actually leaving, so the invariant is clean:
  *nothing leaves a cell that the cell could not carry*. A grain the cell prises
  loose and cannot lift goes straight back into ordinary loose cover — deliberately
  **not** armouring, which needs § 13.4's selective entrainment.
- **`DepUnit` carries `species`** — the material that arrived — and `litho_of_tag`
  is demoted to the *default fill* for depositors that carry no load. Fits the
  existing padding; the record costs nothing.
- **Mass closes per species.** journal/0109's residual rule is applied **per
  species independently**; splitting the total exactly and apportioning by fraction
  leaks silently, because each species rounds against a shared denominator while the
  total stays perfect. `exchange_cell` remains **one** function for both routing paths.

> **⚠ THE OUTCOME IS A NULL, AND IT IS THE MOST USEFUL THING THE SLICE PRODUCED.**
> On the shipped world the identity that travels reaches **0.025 m of a 440,595 m
> archive (0.000006 %)** and the grain-size gradient is unchanged to four decimals.
> Measured cause, in order of size: **fluvial transport is 0.109 % of this world's
> sediment routing** (rivers pick up 659.5 m over 200 epochs; hillslope creep moves
> 605,117 m — a factor of 918); **no cell anywhere can carry sand** (max competence
> ceiling 0.283 against coarse clastic's 0.840, and the world's maximum transport
> capacity is 6.74 × 10⁻⁴, *three times below* the shipped Low/Medium band boundary
> it is anchored on); and downstream of both, journal/0109's uniform convergence
> exponent. **Nothing was tuned.** The implication for this arc is that the next
> slice with a visible payoff is the **gravity/mass-wasting** member of § 13.2's
> family, not a refinement of the fluvial one — creep is what moves this world.

### 13.8c BUILT 2026-07-26 — the second slice, the gravity member (journal/0112)

**Shipped, on by default** (`DeepConfig::material_creep`, gated on
`material_transport`). Off is the fluvial-only world byte for byte; those goldens
are still reachable and still asserted (`tests/material_creep.rs`).

§ 13.8b's own warning was the brief: *"the next slice with a visible payoff is the
gravity/mass-wasting member of § 13.2's family, not a refinement of the fluvial
one — creep is what moves this world."* This is that slice.

- **Creep joins the family, and its competence curve is that there isn't one.**
  § 13.2 says the members share the load machinery and differ in *field* and
  *competence*. Gravity's field is the surface gradient the diffusion phase already
  descends; its competence ceiling is **absent**, because creep is diffusive rather
  than selective. So **identity travels and nothing is sorted** — every edge moves
  the donor's whole composition in proportion.
- **That is the point, not a shortcut.** Colluvium is *locally derived and poorly
  sorted*; alluvium is *far-travelled and sorted*. Giving creep a ceiling to make
  it look like the river would have erased the one contrast the slice buys.
- **One composition seam, now three consumers.** What creep moves is the same
  `outcrop_shares` near-surface window the erodibility blend and fluvial
  entrainment already read — no second walk.
- **One split function, three callers.** Entrainment, incision and creep all route
  through `split_by_shares`, which applies journal/0109's residual rule on the
  **species** axis (last non-zero share takes `total − Σ earlier`). journal/0110
  had to state the anti-leak rule twice; now no caller can invent its own budget.
- **The conservation proof is different in kind, because a diffusion junction has
  no downstream order.** There is no "cell hands its load to its receivers"
  moment to residue-check. What replaces it: each edge flux is **antisymmetric to
  the bit** (IEEE-754 subtraction is exactly antisymmetric), and both endpoints
  split that identical flux by the identical donor composition — so what leaves a
  cell of a species is bit for bit what arrives next door. Asserted as two running
  maxima over the whole run: `Σ_species` per cell equals the scalar metres the
  terrain moved, and `Σ_cells` per species is zero.
- **Identity is not a sidecar, and there is no configuration in which it is.**
  The record's rock is what `outcrop_shares` publishes; that composition sets the
  erodibility blend, the frost multiplier, the eolian deflation susceptibility, the
  wave attack rate — *and* what the fluvial load entrains, which the competence
  ceiling then rains out by settling velocity. So the goldens move. The claim is
  pinned where it is exactly true: within the epoch that produces it, the identity
  moves no terrain at all (`creep_identity_moves_no_terrain_in_the_epoch_it_is
  _measured_in`).
- **The record still says *what* arrived and not *who brought it*** — `stubs.md`
  #25. `DepUnit` is 16 B with no padding left; the free version is a packed
  `(species, mover)` byte and it wants to land with § 13.8's lineage history.

> **✅ THE OUTCOME, AND IT IS THE MIRROR OF § 13.8b's.** On the shipped world
> (`examples/colluvium_probe.rs`, seed 1337, `Extent::Medium`), recorded mass whose
> material disagrees with what its own environment would have implied goes from
> **0.0259 m of 440,578 m (0.000006 %)** to **275,626.9 m of 422,703 m
> (65.206 %)** — a factor of eleven million, from making the *other* member of the
> family honest. **71.3 % of it sits in the lower five drainage deciles**, the
> hillslopes, which is where colluvium belongs. The archive's composition moved with
> it: 91 % fine clastic → **26 % fine / 36 % coarse / 38 % carbonaceous soil**,
> because a hillslope no longer records *"mud, because this is a quiet place"* — it
> records what came down onto it. Distinct species per hillslope column
> **1.091 → 1.952** (hillslope 1.256 → 2.061 against valley 3.167 → 3.504, a
> hillslope/valley sortedness ratio of 0.396 → 0.588): the poorly-sorted signature,
> measured, and landing six times harder on the hillslopes than in the valleys — which
> is the colluvium/alluvium contrast itself. Per-species mass closes
> at **3.19 × 10⁻¹⁵** (itemisation vs the metres the terrain moved) and
> **7.07 × 10⁻¹⁶** (any species created or destroyed), both f64 summation-order
> noise. Cost: deep run **under this machine's noise floor** (two runs, +1.38 s then
> −2.65 s on ~31 s), residency **+18.16 MiB (+10.7 %)**, units **+21.5 %** — all merge
> key, because the axis finally varies.
> **Nothing was tuned; there is no knob in this slice.**
>
> ⚠ **And it surfaced a latent defect by MAGNITUDE — corrections #57.**
> `Litho::as_deposited` said the only lithology a deposit cannot be is basement.
> Peat, coal and charcoal cannot be either (§ 12's four-way test: a *moved* material
> is category 3, an in-place organic is 1/2), and at 0.109 % of sediment routing a
> transported organic could never win a cell's argmax to prove it. At 918× it did,
> within one run, producing a voxel that was 8/8 charcoal against a 0.04 m fire-bed
> cap. **A rule can be wrong and unreachable at once, and "unreachable" is a
> property of the current magnitudes.**

### 13.8 Three deferred layers (named, not this arc)
- **3D volumetric flow** (underwater rivers, turbidity currents, cave streams): a richer
  *flow field* (the free-water body graph, §6/§7); transport-follows-the-field is unchanged.
- **Flow-biased sub-cell fill** (S-4 fine expression): the collapse should read the flow
  direction (`recv`) + local topography to bias grain placement (coarse near the paleochannel,
  fines in distal lows, cross-beds downflow) instead of unbiased addressed stochastic
  rounding (journal/0055). Transport already produces the inputs (composition + flow vector).
  **Tracked: ROADMAP Sequenced "STRUCTURE-AWARE FINE EXPRESSION"**, which unifies this
  (within-*cell* flow bias) with the journal/0010 within-*voxel* order residual (stubs.md §12)
  — one S-4 gap at two scales, one forms-presentation home.
- **Lineage history** (the `Move`-fact chain of custody): identity *travels* now; the *full
  history* ("deposited, exhumed, re-transported…") rides the `Move` variant when we need to
  *read* it (the `inventory.rs` forward-note).

---

## 14. Condition-fields, formation predicates, and the geotherm (DECIDED 2026-07-24)

Ratified this session — the §5 **field-pass half** lands, and with it the general
**condition-field vocabulary** formation predicates read.

**Condition-fields.** A **field** = a named per-cell quantity with an **opaque id**
(`dc:field/temperature`, `dc:field/depth`, `dc:field/tectonic_setting`, `dc:field/exhum`,
`dc:field/pressure`). Produced by **field passes**, read by **cellular passes** and by
**formation predicates**. Initial members: `tectonic_setting` (province), `depth`, `exhum`,
**`temperature`** (new — the geotherm), `pressure` (derived). **Extensible** — every field
pass grows the vocabulary; the migration of today's ad-hoc/stub proto-fields into real
declared field passes is ROADMAP-tracked.

**Formation predicates are plain data over field-ids** (§12: formation is output-owned). A
<!-- Reciprocal pointer added 2026-07-29 (baseline sweep S3/F6): `material-genesis-notebook.md`
§ 2.4 (PROPOSED, 2026-07-26) argues the opposite for TERMS — "terms should stay FUNCTIONS, not
a closed formula vocabulary" — because a liquidus temperature is pressure-dependent, so the
ordering key must be a function evaluated at the cell. That notebook lists §12 as its inbound
link and NOT §14, and §14 pointed nowhere. The reconciling reading is available in neither doc:
predicates are DECLARATIONS (plain data), while terms inside a pass BODY are backend-compiled
code (north-star § The crossing constraint). WHETHER THAT RECONCILIATION IS RIGHT IS NOT
SETTLED HERE — the notebook is an open edge and this clause is DECIDED; the edge is flagged,
not resolved. -->
A
predicate is a conjunction of `(field_id, comparator, range)` conditions the engine evaluates
against local field values — **no code crosses the SDK; the fields are the interface**
(crossing constraint met by construction). **This is a general engine primitive** (user):
the same declarative-condition-over-fields model is expected to serve geology's formation
predicates *and* future **ecology, civilization**, etc. — not a geology carve-out.

**NO capability tiers** (north-star § Deviations #2, user-emphatic): field passes are **not**
trusted-only — **a mod authors field passes exactly as the defaults do.** One authoring shape,
no walls.

**The geotherm — the FIRST field pass** (DECIDED). A **low-rate** field pass. **Reads**
`{surface_temperature (climate), crustal_thickness (t_crust), crust_kind, tectonic_setting}`;
**writes** the `temperature` field as a per-cell **geothermal gradient**, `T(depth) =
surface_T + gradient · depth`; `gradient = f(tectonic heat flow)` — rifts/arcs steep
(~40–50 °C/km), old cratons/thick crust shallow (~15–20). **v1 is linear**;
nonlinear/mantle-heat is a ROADMAP followup. **Subsumes the degenerate `burial_temp_c`
geotherm** (stubs.md #14) — the real gradient replaces the 1 °C/m stub, with **`COAL_ONSET_C`
recalibrated in the same slice** (stub #14's retire-together warning). **NOT byte-identical**
— coal distribution moves (coal is a placeholder anyway — no real bio yet — so the *number*
is not precious; **we measure the shift as the reason to walk**). **Unblocks:** metamorphism
(`exhum` = P, geotherm = T → grade), formation predicates (mineral stability by T), and
eventually melting/magma.
