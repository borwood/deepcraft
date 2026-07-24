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

A **native** engine whose core is only **cell storage + a pass-runner + a small
set of native field-solvers + a stable API surface**; everything else —
materials, their behavior, and the passes that manipulate them — is **authored in
a uniform, self-declaring, compiler-validated shape and tuned by data.** Utterly
consistent, plugin-first, and safely moddable by *untrusted* third parties.

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
body. The runner topo-sorts by declared reads/writes and rejects conflicts.

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
