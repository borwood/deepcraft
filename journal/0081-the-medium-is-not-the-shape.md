# 0081 — the medium is not the shape

*2026-07-23. A design pass with no code — the session where the block↔material
collapse turned out to be step one of a much larger thing, and the user drew the
engine's north star out of a modder's instinct. `docs/design/north-star.md`
carries the ratified shape; this carries how we got there and the two moves that
made it defensible.*

> blogworthy (lenses: AI-native development; deepsim-reflexions; procgen against
> priors): **the engine whose extension model is designed for AI-authored
> content — and the insight that "data-driven" was never about the file format.**
> Twice now the person who does not read our code has drawn a load-bearing
> architecture decision out of the *shape of a consequence* (0077: block and
> material are one thing; 0081: the whole engine is passes over materials).

## It started as a migration and became a cornerstone

We opened the block↔material collapse because the user called it upstream of
everything — "a cornerstone every other system is going to consume." A read-only
recon (the block-consumer inventory) came back with a reassuring surprise: the
collapse is ~80% already done at the core. `dominant_material` already returns
`Option<MaterialId>`; `classify` is literally `dominant_material().map(block_twin)`;
the atlas is already keyed by material. The end-state — "delete `block_twin`,
answer with the material" — is a short tail, not a 135-site march. The genuine
work is an atom decision (where does *Air* live once the atom is a material) and
retiring four legacy blocks with no material twin.

That would have been a fine, bounded slice. Then the user gave a blue sky.

## The blue sky, and why it was not a detour

The vision: an engine with **only** cell storage and a runner for data-defined
passes; materials hierarchically defined with behavior methods (`can_combust?`,
`combust→`) that children shadow; passes that select materials by ancestry or
predicate over a cell context and transform them; cadence and epochs as world
config; providers with access to world/cell/material. Utterly consistent, utterly
data-driven, utterly plugin-based. The user asked me to *defeat* it — to test
whether it was a loose parallel to our shape or a mess waiting to happen.

I could not defeat it, and the reason is the finding: **it is the asymptote of
the march we are already on.** The pass graph, the `Providers` `Option<fn>`
pattern (six merged this week), `MaterialProps`, the 0077 transformation-axes,
categories-registrable, `build_checked`, the octree, the two-clock doctrine — ~70%
of the vision has an embryo in the tree, and the seam-first doctrine is the
incremental walk toward it. Naming it does not commit us to a rewrite; it lets
every conversion we are *already* making consciously serve a destination.

But there was a real mess-risk, and I named it plainly: the **behavior language**.
If `combust→` is arbitrary data, you have built a scripting engine — an
interpreter, a DSL to bound, determinism and perf and sandbox costs. That, not the
architecture, is where "huge mess" lives.

## The move that dissolved the mess: the medium is not the shape

The user's "naive question" was the hinge: *how could we keep all the logic native
— compiled Rust — while authoring it in the data-driven shape?* Define a pass by
implementing a Pass interface; define a material the same way; the native
functions exist, you can add arbitrary ones, it all compiles the same.

That separates two things I had let stick together. The value we want from
"data-driven" — uniform, composable, self-declaring, validated authoring — is a
property of the **shape**. The mess lived in the **medium** (interpreted data). So
the medium is **compiled Rust authored in the declarative shape**: native speed
(runtime stays sacred by construction), the compiler as validator (earlier and
stronger than a load-time check), and — crucially — no interpreter, no DSL, no
mess. And it is a *smaller* leap than an interpreted engine, because it is the
generalization of the pass graph and `Providers`, not the invention of something
new. A material becomes a `Providers`-of-behavior + a parent pointer + a slug. The
thing we build at world level slides down to material level unchanged.

## The second move: solving the ancestral space's own problem

The user then closed the last gap by *raising the bar*. They want the untrusted
third-party mod ecosystem — the Minecraft lineage — and they want it without
open-sourcing the engine. The honest constraint: native modules are arbitrary
machine code and cannot be sandboxed, so "untrusted native with full safety" is a
triangle you cannot close (the Minecraft space never closed it either — it runs on
curation and trust). But WASM closes it *one better*: mods compile to a sandbox
that can only call the host API you expose.

The resolution is a tiered backend behind **one authoring shape**. A mod is
written against an SDK and only ever touches the world through a `ctx`. Because it
speaks only through that socket, the backend chooses what the socket is wired to:
native (`abi_stable`, full speed, trusted/first-party) or WASM (`wasmtime`,
sandboxed, untrusted, gen-tier). Same source, same trait impls — only the compile
target and loader differ. Closed-source stays intact: you publish the SDK, not the
engine.

And the ergonomics worry answered itself. The crossing constraint (plain data and
opaque handles across the seam; no rich Rust references, generics, or slices)
applies **only to the socket**. Rich Rust lives on *both sides* — engine internals
and a mod's own private computation inside `run`. A content pass computes as richly
as it likes, then talks to the world in plain data. The tax lands on the SDK
surface *we* design, never on the content author's expressiveness — exactly the
user's hope that anything needing Rust ergonomics is engine-internal.

## What got ratified, and the discipline that guards it

The shape is ratified (`docs/design/north-star.md`), and with it the block-collapse
cruxes it now serves: render-first as the proof wedge; `Block = { Air,
Material(MaterialId) }` (air is a peer, keep the token, niche the atom for a free
chunk-RAM halving); and — the user's sharpest instruction — **retire the legacy
blocks, do not enshrine them.** *"I am not interested in accumulating debt and
slowing velocity to enshrine waste, accidents, bandaids, stepping-stones… for no
reason it seems to me other than 'they already exist :('."* That sentence is the
north star's enforcement clause in miniature.

The load-bearing design constraint the model rests on, recorded so it is not lost:
the pass `ctx` is a **capability**, not a god-object — a pass can only touch the
state it declared, compiler-enforced. That is what keeps "it is all native Rust,
add arbitrary functions" from rotting back into the bespoke carve-outs the whole
model exists to kill.

And the user made the process itself load-bearing: *"ensure all design flows
through it, no accidental divergence, always double-checked."* So the north star
joins spines.md as read-first (CLAUDE.md item 0) with the same compliance loop —
plan-time convergence, work-time loud-plea, review-time check, carve-outs through
the user. Spines names the shapes of the code today; the north star names where it
is going; work is checked against both.

## What we deliberately did not do

We did not start building the framework. It is the biggest architectural
commitment in the project — a multi-month *evolutionary* arc, and the honest thing
is to de-risk the make-or-break before betting the engine on it: prototype whether
`combust→` expresses real cases as bounded functions (wood/charcoal); reformulate
one existing pass (the fires proxy) onto the Pass interface byte-identically; and
spike the ABI/WASM boundary (`abi_stable` vs `repr(C)` vs `wasmtime`) to see what
the socket can carry and cost before the SDK shape is locked. The north star is
the destination that steers the conversions we are already making — not a
green-field we stop and build.
