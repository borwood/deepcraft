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
- **`commit_chapter` = diff-and-append:** working-inventory vs the chapter-start
  derived state → the deltas ARE the facts. **Empty delta ⇒ no facts ⇒ record
  byte-identical** (the S17 identity default, already proven).
- **A fact's shape:** `(chapter, edge, (from-mat, from-form) → (to-mat, to-form),
  fraction)` — carries material change, form-only change (crumbling), and dissolution
  (`→ void`). Ordered by chapter; within a chapter a commutative batch (one
  synchronous pre-state).
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

---

## 4. Agents — rate-term contributors on edges (content)

An **agent** is a *named rate-term bound to an edge*:

> `agent term = environmental driver × per-material susceptibility → added to the edge's rate`

- Frost, abrasion, biotic, dissolution each add a term to (e.g.) `structure→loose`;
  **multiple agents on one edge sum** — this is the S16 fold (proven, correct; it
  is *not* a scheduling question).
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
pass-runner (topo-sorted, cycles rejected). Two shapes:

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
the deep-cell **working inventory** as a mutable span-list (the keystone).
