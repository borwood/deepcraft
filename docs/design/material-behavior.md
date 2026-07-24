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

### Cadence: order × rate — RECONCILED 2026-07-24 (promoting `ideas.md § Pass cadence`, user)

The scheduler has **two orthogonal axes**, and the runner declares **both** per pass:

- **ORDER** — derived from `{reads, writes}` by **topo-sort**; rejects cycles,
  conflicting writers, missing deps. This *replaces* a hand-declared "canonical
  order": the order falls out of the declared dependencies and an illegal schedule
  is **caught**, not trusted.
- **RATE** — the **fractional-phase phase length**: how many sub-turns a pass takes
  per chapter, and the **`dt`** that scales its transformations. A chapter subdivides
  into sub-turns; a **high-rate** pass (weathering ×5) sees mid-chapter state evolve
  while a **low-rate** pass (tectonics ×1) runs once — the temporal-resolution knob
  topo-sort alone does **not** give. This is `ideas.md`'s fractional-phase sketch,
  reconciled: it is the RATE axis, *composed with* topo-sort, never replaced by it.

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

**The transform/formation discriminator** *(framing)*: is a **specific solid material
consumed**, or does the product **precipitate from something diffuse**? clay→mica consumes
a specific solid → **input-owned transform** (provenance stays on the material); emerald in
a vein / cement in pores / primordial rock from melt come out of fluid/melt/diffuse
conditions → **output-owned formation**. This already maps onto §3's edge types: **edges
*from* fluid/void** (cementation, deposition, crystallization) are output-declared
formation; **solid↔solid edges** (weathering, metamorphic replacement) are input-owned
transforms. The graph's edge type already encodes which end owns it.

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
