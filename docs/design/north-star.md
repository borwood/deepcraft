# The North Star — the engine shape everything converges to

**Status: ratified direction (user, 2026-07-23).** This is the *destination*,
pursued **evolutionarily** through seam-first conversions — not a spec to build
wholesale, and not a rewrite. It exists so that every design decision can be
checked against one shape. **All design flows through this doc; divergence is
never silent** (§ Compliance). It is the strategic companion to
[`spines.md`](../spines.md): spines names the shapes and anti-shapes of the code
*as it is today*; the north star names where it is *going*. Check both.

> blogworthy (lenses: AI-native development; deepsim-reflexions): the engine
> whose extension model is designed for AI-authored content, and which solves —
> one better than its ancestral space — the untrusted-mod problem with a
> sandbox tier behind a single authoring shape.

---

## The one-sentence shape

A **native** engine whose core is only **cell storage + a pass-runner +
~~a small set of native field-solvers~~ field-solver PRIMITIVES + a stable API
surface**; everything else —
materials, their behavior, and the passes that manipulate them — is **authored in
a uniform, self-declaring, compiler-validated shape and tuned by data.** Utterly
consistent, plugin-first, and safely moddable by third parties.

> **⚠ "NATIVE FIELD-SOLVERS ARE CORE" WAS RETIRED 2026-07-23** by § *The core/plugin
> boundary*, which says so twice — *"that sentence conflated compute shape with trust
> tier"* — and **this opening line was never updated to match**, so two read-first surfaces
> copied the retired version forward (caught by the first `doc-topology` sweep, 2026-07-26).
> **The retirement became load-bearing on 2026-07-26** (user; `ARCHITECTURE.md` § *The
> engine is plugin-agnostic, and pass ORDER is authored*): **plugin-agnostic is the start
> and end of the conversation, and tectonics and erosion come from PLUGINS.** The core holds
> **primitives, not solvers** — the numerical kernels are core; every pass that calls them,
> including tectonics and erosion, is **content**.
>
> **A field pass may execute through a native backend for PERFORMANCE.** That is a
> **compute-shape** choice and **not a permission tier**: *there is no difference in
> permission between native and WASM* (user, 2026-07-26), and the **trust mechanism and
> boundary are DEFERRED** — § *Deviations* 1, *"paused until we are anywhere near having
> modders."* Do not justify a core/content placement by trust; that is the exact conflation
> the 2026-07-23 retirement removed.

## Native, not interpreted — the medium is not the shape

The value we want from "data-driven" — a uniform, composable, self-declaring,
validated authoring pattern — is a property of the **shape**, not of the
**medium**. The mess-risk (a scripting engine: an interpreter, a behavior DSL to
design and bound, determinism/perf/sandbox costs) lived entirely in choosing an
*interpreted* medium. So the medium is **compiled Rust, authored in the
declarative shape**. This keeps native speed (runtime stays sacred by
construction), pushes validation *earlier* (the compiler is a validator), and is
a *smaller* leap — it is the generalization of two patterns the engine already
ships (the pass graph and `Providers`), not the invention of an interpreter.

## The core / plugin boundary

> **🔖 OPEN EDGE — see [`material-genesis-notebook.md`](material-genesis-notebook.md) § 2.**
> A 2026-07-26 design conversation re-derived much of this section from scratch and then
> pushed past it. Three live threads land directly on this boundary: **pass purity**
> (the content layer's *"select materials matching predicate P"* is already
> material-agnostic — **and the running engine violates it**, measured); **term-keyed
> edges** (a pass selects by the *terms* an edge carries, not a shared slot name — which
> would make participation derive from declared properties); and **where "engine owns
> primitives" actually cuts**. Also recorded there: *the plug-and-play mechanism is
> **parent inheritance**, so the default pack's parent materials are a **product
> surface**, not an implementation detail.*

**Core** (irreducibly native, changes freely, private/closed-source):
- **cell storage** + the **cell API** (know/change what a cell contains; hold
  state other passes stamped on it).
- the **pass-runner** — runs passes in declared order at declared cadence within
  declared epochs; topo-validated (rejects cycles, conflicting writers, missing
  producers).
- **field-solver *primitives*** — the native numerical kernels (a bounded
  relaxation, an advection step, flow accumulation) that a *field pass* calls.
  These are core the way a shader language's `texture()` is core; the passes that
  orchestrate them (tectonics, hydrology, climate, thermal) are **content**, not
  core (2026-07-23 refinement below — the earlier "native field-solvers are core"
  framing conflated *compute shape* with *trust tier*). A field pass **exposes its
  outputs as world/cell API state** ("the tectonics pass plants its API on the
  cell"), which other passes read. What stays irreducibly core is the *kernel*,
  and the discipline it protects: not everything is a per-cell data-shaped pass,
  and pretending so is the failure mode.
- **the event ledger** — provenance of what each pass did to a cell.
- the **world API** (world settings, epoch configuration).

**Content layer** (authored against the SDK; first-party *or* third-party):
- **material definitions** — hierarchical, behavior + sheet + slug→assets.
- **declarative material-transform passes** — "select materials matching
  predicate P in cell context C, apply transform T at rate R." The sweet spot:
  AI-authorable, validatable, and where combustion, weathering, diagenesis,
  decay, cementation naturally live.
- **world / epoch config** — data (below).

### ✅ THE REFINEMENT TIER — **RESOLVED TO (c), user, 2026-07-28**

> **DECIDED (user, 2026-07-28).** The boundary is stated, and it is **candidate (c)
> below** — the 2026-07-23 field-pass split applied to this tier:
>
> - **Engine owns the PRIMITIVES — field kernels *and* refinement kernels — plus
>   the RUNNER.**
> - **Plugins declare all content: fields · field passes and cell passes · pass
>   ORDER and RATE · materials · REFINEMENT · etc.**
>
> **⚠ The user stated that list is NOT exhaustive** (*"this list may not be
> exhaustive"*). It is the *shape*, not the inventory — do not cite it as a closed
> enumeration, and treat an absent item as **unlisted**, not as **excluded**. *This
> section is itself the corpus's headline example of an enumeration nobody checked
> for completeness; the fix is not to pretend the new one is complete.*
>
> **⚠ PROVENANCE, stated because it changes how much weight this carries.** The
> user gave this while ruling on **when bio/social work may begin**
> ([`worldgen.md`](worldgen.md) § *Sequencing*) — i.e. as a statement of the engine
> shape that must hold *first*, not as a deliberate adjudication of the three
> candidates below. It matches **(c)** exactly and (c) was already the direction of
> travel, so it is recorded as the ruling. **If a narrower reading was meant, this
> is the line to contest** — recorded this way rather than silently, per
> corrections #65.
>
> **What is still open:** the *mechanism* — which kernels are primitives, what the
> operator authoring shape is, and how the pure-fn contract is enforced across the
> backends. **The boundary is decided; the surface is not.** ROADMAP § Sequenced
> "REFINEMENT PRIMITIVES" still owns that work.

*The original framing is kept below unstruck, because its argument is what makes
the ruling legible — and because the "vocabulary of expression" problem it names is
**not** solved by deciding the boundary; it is solved by building the operators.*

**Neither column above mentions the refinement / collapse tier**, and it was never
decided — it was arrived at *by default*. `flow.md` § 4 names it as one of four
tiers and even gives it a contract (*"refinement operators — **may be clever**, must
be a **pure fn of (record, shared face data, position)**"*), but this boundary does
not know it exists. In code it is `collapse.rs`: **2,415 lines of ordinary engine
code with essentially zero pass-shaped declaration.**

**Why it matters, stated as the question the user asked:** *"how would a mod
hand-roll emergent rivers on their own via the SDK, that are **visible**?"*

Today, partially and then not at all. A mod can declare materials and a deeptime
pass that moves mass; the mass movement lowers the rock/regolith heights, and the
collapse tier turns heights into voxels — **so a mod gets emergent VALLEYS for
free.** But a valley on a ~460 m cell is not a channel, and a channel is *sub-cell
geometry*, which lives entirely in engine code. So:

> **The engine hardcodes the VOCABULARY OF EXPRESSION.** A mod can add **materials**
> (which ride existing material→voxel rules) and **move mass** (which rides the
> height field). A mod **cannot add a new KIND of visible structure** — a channel
> form, a cross-bed, a sorted lamination — because that needs new expression logic,
> and expression logic is not authorable.

**This also separates two nulls we had been treating as one.** Movement 2b's missing
facies gradient is a **magnitude** problem (a strong signal *would* show, because it
is material variation and materials have a path). The missing channel is a **path**
problem — *no magnitude of flux record produces a channel*, because sub-cell geometry
has no declarative route to a voxel at all.

**Three candidate answers** *(assistant-framed 2026-07-26; the user has chosen the
direction, not the mechanism — see ROADMAP § Sequenced "REFINEMENT PRIMITIVES"):*
**(a)** engine owns refinement outright — mods reach the eye only via heights and
materials, and could never ship a new landform *kind*; **(b)** refinement becomes
fully declarative — maximally open, but refinement is on the **runtime** hot path and
*runtime is sacred*; **(c)** **split it exactly as the field passes were split** —
*refinement **primitives** are core* (the sub-cell boundary-value solver,
position-addressed noise, interval fill) *the way `texture()` is core to a shader
language*, and *refinement **operators** are content*.

**(c) needs no new philosophy — it is this document's own 2026-07-23 move applied to
a tier that got skipped**, and two things fit it unusually well:
1. **The pure-fn constraint is already the sandbox contract.** Refinement must be a
   pure fn of `(record, shared face data, position)` for **chunk determinism** — so a
   chunk renders identically regardless of which neighbours are resident. That is
   *also* exactly what makes something safely sandboxable in WASM: no ambient state,
   no global reads, all inputs declared. **One constraint, two payoffs, already paid
   for.**
2. **Granularity is the perf question, and seam-first already answers it** — *"a
   provider must never be called inside a hot loop to answer a question that does not
   change inside that loop."* Refinement operators must be invoked **per-cell or
   per-chunk, never per-voxel**. A sandboxed call per chunk is affordable; per voxel
   is fatal. Worth stating **before** anyone builds it.

## Materials

A material is **a property sheet (data) + behavior slots (functions) + a parent
pointer + a slug**:
- **Sheet:** density, grain size, cohesion, permeability, solubility, damage
  resistances… (already `MaterialProps`).
- **Behavior slots:** `can_combust?`, `combust_rate`, `combust→`, `weather→`,
  … — functions taking cell context, returning predicates / quantities /
  transforms. This is the [`Providers`](../../crates/dc-worldgen/src/deeptime/providers/)
  pattern (`Option<fn>` slots with an identity fallback) **generalized from
  world-level to material-level.**
- **Parent pointer + shadowing:** to answer `combust→` for `oak`, walk up the
  parent chain (`oak → wood → organic`) and use the first ancestor that supplies
  it. Children shadow parents. Categories are the upper levels of this hierarchy;
  **packs add classes (parents), not only members (leaves)** (materials.md
  DECIDED 2026-07-22).
- **Slug → assets:** the material declares `slug: "dc:wood/oak"`; an asset layer
  matches the slug to files (textures, etc.). The atlas already keys layers by
  material id — extend to slug resolution.

## Passes

A pass **declares itself**: `{reads, writes}` over the cell/world/material API,
its **cadence** (runs per unit of deep-time), its **epoch**, and a `run(ctx)`
body. ~~The runner topo-sorts by declared reads/writes and rejects conflicts.~~
**SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD**; `{reads, writes}`
became the **validator**, not the generator (`ARCHITECTURE.md` § *The engine is
plugin-agnostic, and pass ORDER is authored*; **corrections #65**, which names *this
section* as one of three sites of the falsified claim).

Two **shapes**, one interface — **both are content** (2026-07-23 refinement):
- **cellular passes** — per-cell select-and-transform ("select materials
  matching P in context C, apply T at rate R"). The AI-authorable sweet spot
  (combustion, weathering, diagenesis, decay, cementation).
- **field passes** — declare reads/writes over fields; the body is arbitrary
  computation calling **core solver primitives**. Tectonics/hydrology/climate/
  thermal. Native-backend and first-party — because they are trusted and hot,
  *not* because they are core.

**The `ctx` is a capability, not a god-object.** A pass can only touch the state
it *declared* — the `ctx` it is handed exposes nothing else, enforced by the
compiler. This is the load-bearing discipline: it is what keeps "it's all native
Rust, you can add arbitrary functions" from rotting back into bespoke passes that
reach into global state and recreate the carve-outs the whole model exists to
kill.

## Refinement (2026-07-23): two shapes, two runtimes, one declaration

Ratified in design conversation 2026-07-23. Sharpens the core/content boundary
above; where the two disagree, this governs.

**The cut is machine-vs-content, not compute-shape.** Core is only: **cell
storage + cell/space API · the pass-runner · the event ledger · the field-solver
primitives · the data-model APIs (materials / items-recipes / blueprints /
bodies) · the stable SDK surface.** *Every pass is content* — including tectonics
and erosion. A field pass is thin orchestration over core kernels; it runs native
because it is trusted and hot, not because it is part of the core. (This retires
the earlier "native field-solvers are core" framing — that sentence conflated
compute shape with trust tier.)

**Passes come in two shapes** (§ Passes): **cellular** (per-cell
select-and-transform) and **field** (global solve over the grid). For the *cell
world* there is no third.

**Capability, not core, gates the tiers.** What a tier may author is a capability
set, matching "`ctx` is a capability, not a god-object": the untrusted tier gets
material declarations + cellular passes + data; registering a new field pass (a
global solver) is a trusted-tier capability. Same authoring shape, different
granted powers.

**Deeptime compiles; the present executes.** The two clocks are two *runtimes
over one set of declarations*: **deeptime is an ahead-of-time compiler** for the
world (seed + config + declarations → compiled cell state + ledger; gen-time is
free), and **the present runtime is the live VM** that loads it and re-invokes
the *same* declarations sparsely and event-driven (runtime is sacred). **One
declaration, two executors:** `wood`'s `combust→` runs as a bulk fire-record
transform over chapters in deeptime and as a live per-voxel event in the present
— the declaration is the single authority (S-3); the runtimes are two
interpreters of it (materials.md DECIDED 2026-07-22).

**Deferred, named so it is not foreclosed:** actors/agents (DF-style civ + a
historical ledger, and the evolutionary graduation of body declarations to
agents) introduce a **second substrate** — an entity/actor table beside the cell
world — and stepping it is a **third pass shape** (discrete-actor stepping,
neither cellular nor field). Not built; a *cellular shim spawner* covers mobs
today and grows into the actor sim without changing the content slot. Provenance
and observation-collapse ("where is the duke?") are the same overlay shape —
**spines S-9**.

## The two clocks (runtime is sacred)

- **Gen / deep-time passes:** perf is *not* a constraint (ready-made worlds are
  the sanctioned answer). Most content passes run here — they shape the *world*.
- **Runtime:** sacred. Only sparse / event-driven passes; first-party native.
- **Untrusted third-party content lives at the gen tier** — world-shaping, where
  a sandbox's overhead is acceptable and it never enters the per-frame loop.
  Trusted / signed content can earn the native runtime path.

## Behavior is code; tuning is data

The **logic** (pass bodies, material behavior methods) is compiled Rust. The
**numbers** (rates, thresholds, magnitudes; epoch config — "run N chapters" or
"until a condition / seeded roll"; world size) are **data sheets, tunable without
recompiling.** Code defines the machine; data tunes it. `DeepConfig` is the
embryo. This dissolves the "recompile to change a rate?" objection — no, that is
data — while keeping logic in the fast, validated medium.

## The mod SDK boundary + the tiered backend

> **Status (2026-07-23 refinement):** the trusted/untrusted **backend** split
> below is **deferred product infrastructure**, not a live architectural fork.
> The only part that binds design decisions now is **the crossing constraint**
> (plain data + opaque handles across the seam) — keep that, defer the rest.

**Modding never requires open-sourcing the engine.** Publish an **SDK** (the
Pass / Material / `ctx` traits); keep the engine private. Standard practice
(Unity/Unreal expose modding SDKs over closed engines).

**Everything is built on the SDK route mods take** (user, 2026-07-23). The default
passes/materials are *the first plugins* — first-party content ships **through the
same SDK** (native backend, since it is trusted), not a privileged internal path.
Consequence: the SDK surface must be complete enough to carry the **entire default
content set**, not just toy third-party mods — a higher bar, but **self-validating**
(if the whole game is built on the SDK, the SDK is proven complete by
construction; dogfooding in its strongest form). Not built now, but every SDK-shape
decision is measured against *"can it carry the defaults."*

**One authoring shape, two execution backends — a loader choice, not an
architecture fork:**
- **trusted / signed + first-party → NATIVE** (`abi_stable` over a stable
  boundary): full speed, including the runtime path.
- **untrusted third-party → WASM sandbox** (`wasmtime`): the mod can only call
  the host API you expose; safe by containment. Lives at the gen tier; some
  marshaling overhead, which the two-clock doctrine makes acceptable.

**The crossing constraint** (the source of the ergo tax, and why it is bounded):
the SDK API surface must be expressible across *both* backends = the intersection
of `abi_stable`-safe and WASM-marshalable = **plain data + opaque handles (ids),
no naked references / generics / slices across the seam.** Rich Rust stays on
*both sides* of the boundary — engine internals *and* a mod's own private
computation inside `run` — so the constraint lands on **the socket we design, not
on the content author's expressiveness.** A content pass computes richly
internally and speaks to the world in plain data.

**The triangle you cannot fully close:** native speed + untrusted code + full
safety — pick two. Native modules are arbitrary machine code and cannot be
sandboxed; "untrusted native" is only safe if made *accountable* (curation /
signing), never *contained*. The tier system means we never need that corner: we
never run untrusted native. (Same limit as the Minecraft modding space; the WASM
tier is the one-better.)

**Validation + hosting** (a trustworthy-mod store, signing) is later product
infra; the architectural hook is that **mods are versioned artifacts declaring
their SDK version and their reads/writes**, so a store can validate compatibility
and sign, and the load-time validator can refuse an incompatible one.

## Validation by construction

Declared contracts — a pass's reads/writes, a material's declared behavior, a
mod's SDK version — are checked: **at compile time where the backend is native,
at load time where it is WASM / data.** This generalizes today's `build_checked`
(a world refuses to build if a class a pass selects from has zero members). "By
construction compatible" is not a slogan; it is that check, extended to the whole
content surface.

## What exists today — the embryo (this is why it is evolutionary)

| North-star piece | In-tree today |
|---|---|
| cell storage + basic API | `VoxelContents` (structure/debris/pore multisets of `MaterialId`) + `Chunk` |
| pass-runner, declared order | `pipeline.rs`: passes declare reads/writes, topo-sorted, conflicts rejected |
| providers (shared fns, context) | `Providers` `Option<fn>` slots — six merged this week |
| material sheet | `MaterialProps` |
| material behavior methods | transformation-axes DECIDED 0077 (inheritable, patchable, deletable) — designed |
| material hierarchy | categories-registrable (0077) |
| cadence / epochs | deep-time chapters; the `tectonic_history → biotic → full_agents` flag layering *is* progressive epoch enabling (hardcoded) |
| validation by construction | `build_checked` + pass-graph topo checks |
| octree as world API | the octree-substrate pass, FF2b built |
| two clocks | the runtime-is-sacred / gen-time-is-free doctrine |

**~70% is embryonic in-tree, and the seam-first march is the path.** We do not
stop and rewrite; we keep converting, now with the destination named — which lets
each conversion consciously serve it (the block↔material collapse is step 1 of
"materials are the universal substance the API hangs off").

## Not yet proven — de-risk before committing the arc

1. **Hierarchical material behavior resolution.** Does `combust→` express *real*
   cases (combustion, weathering, coal-rank) as bounded functions/data, or do
   they need arbitrary code? Prototype on `wood`/`charcoal`.
2. **Pass-as-self-declaring on a real example.** Reformulate one existing pass
   (the fires pass — already the un-avowed proxy) onto the Pass interface,
   byte-identical to the Rust one as the acceptance test.
3. **The SDK / ABI boundary research spike.** `abi_stable` vs `repr(C)` vs
   `wasmtime` / the component model, measured on a toy Pass/Material, to see what
   the seam can carry and what it costs.
   **AMENDED — NOT GATING (user, 2026-07-24, § Deviations #1):** this spike no
   longer blocks the arc. Build **native `abi_stable`-shaped now** (plain data +
   opaque ids — the crossing constraint), **`wasmtime` to follow**; the spike is
   worth doing "when we have free time, in case of surprises," but the *shape* does
   not require it first. Build in the way one would reasonably expect to be fine
   and adjust later if needed.

## Compliance — all design flows through this (user-directed, 2026-07-23)

*"Ensure all design flows through it, no accidental divergence, always
double-checked."* The mechanism, mirroring the spines.md loop:

- **Read-first** — CLAUDE.md item 0, beside `spines.md`.
- **Plan time** — every brief states how the work *converges toward* the north
  star (or is a deliberate step on the path to it). A brief that names no
  relationship to it is not ready.
- **Work time** — a worker who believes a divergence is right makes a **loud
  plea**, never a silent divergence.
- **Review time** — the integrator checks convergence; a slice that moves *away*
  from the shape is flagged and does not merge unremarked.
- **Carve-outs pass through the user** — recorded in § Deviations below with date
  and reasoning. Ratified deviation is a decision; undocumented deviation is the
  failure.
- **Complement, do not duplicate, spines.md** — spines = current shapes; north
  star = destination. A conversion is checked against both: does it follow the
  present shapes *and* move toward the destination.

## Deviations (ratified carve-outs)

**1. (2026-07-24) The trusted/untrusted safety split + the ABI/WASM spike are
DEFERRED, not gating** (user; *"last session failed to record"* this). We have no
mod ecosystem and no trust/permission system gamed out — that is a **far concern
that does not need solving soon**, and the whole trusted/untrusted discussion is
**paused until we are anywhere near having modders.** The pass/SDK **shape in
principle does not require the spike now.** So: build **native `abi_stable`-shaped**
(the crossing constraint — plain data + opaque ids — still held, because it is cheap
insurance and it *is* the honest shape), **`wasmtime`/sandbox to follow** when there
are untrusted mods to sandbox. Do the boundary spike opportunistically ("in case of
surprises"), not as a blocker. Reasoning: build in a way one would reasonably expect
to be fine, adjustable later — velocity now, safety when it becomes real.

**2. (2026-07-24) NO capability tiering — a mod can author ANYTHING, including a field
pass** (user, emphatic). *"There is no trusted or untrusted until such a time as we need to
figure out what that means because we anticipate mods. ASSUME MODS CAN DO ANYTHING. Including
author a field pass. Do not bake in any limitations on a trust model I DO NOT RATIFY. We're
pro modder freedom and creativity — it would be GREAT if a modder wanted to figure out the
field-pass API. Setting up these walls is pointless; it stifles creativity and they crack in
anyway (see Minecraft)."* So the **capability-tiering** in § "The core / plugin boundary",
§ Passes ("field passes … trusted … first-party"), and § Refinement ("a global solver is a
trusted-tier capability") is **EXPLICITLY NOT THE MODEL** and must not shape any design.
**One authoring shape, no tiers:** materials, agents, cellular passes, AND field passes are
authored the same way, by defaults and mods alike. The **crossing constraint** (plain data +
opaque ids for *declarations*; pass *bodies* are backend-compiled code) stays only as the
*technical* backend-agnostic shape (cheap insurance, per #1) — **never as a trust wall.**
Trust/sandboxing is revisited only if/when a real mod ecosystem forces the question — not
now, not baked in.
