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
- **native field-solver passes** — tectonics (isostasy), hydrology (drainage),
  climate, thermal: global numerical methods that are *not* per-cell material
  selections. They stay native and **expose their outputs as world/cell API
  state** ("the tectonics pass plants its API on the cell"), which content passes
  then read. This is the boundary that keeps the whole model from becoming a
  mess: not everything is a data-shaped per-cell pass, and pretending so is the
  failure mode.
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

Two kinds, one interface:
- **native field-solvers** (core) — arbitrary computation, expose state.
- **declarative material-transform passes** (content) — select + transform.

**The `ctx` is a capability, not a god-object.** A pass can only touch the state
it *declared* — the `ctx` it is handed exposes nothing else, enforced by the
compiler. This is the load-bearing discipline: it is what keeps "it's all native
Rust, you can add arbitrary functions" from rotting back into bespoke passes that
reach into global state and recreate the carve-outs the whole model exists to
kill.

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

**Modding never requires open-sourcing the engine.** Publish an **SDK** (the
Pass / Material / `ctx` traits); keep the engine private. Standard practice
(Unity/Unreal expose modding SDKs over closed engines).

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
   the seam can carry and what it costs. **This locks the SDK shape** — do not
   guess it.

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

*(none yet)*
