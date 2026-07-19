# 0007 — the geology backbone lands (classes, passes, and a placer)

*2026-07-19 · geology v1 (background agent, integrated by the main session:
merge `1df2666`, all 35 suites green on merged main). The backbone
(classes-as-contracts, deterministic selection, processes-as-passes,
core-axes-only) and the v1 content set were ratified 2026-07-18; this entry
is about what building them actually surfaced. No screenshots — geology is
deliberately headless until the client integration (ROADMAP 3c) makes it
walkable; the next entry should have the first geology photograph.*

## The registry opening that wasn't an opening

S5 left a note: the `Payload` union "keeps an open path (schema-registered,
serde-tagged) so new commands don't ossify the enum." The geology class
registry was the first consumer to need it, and the tempting reading — make
`Payload` extensible, dynamic variants, registered decoders — would have
broken the one thing postcard gives us (positional wire identity) to gain
generality nobody had asked for.

The realization that shipped is smaller and, I think, more honest: **the
union stays closed; the opening lives inside one variant's contents.** A
`define_class_member` payload carries `params: [(name, value)]` where the
*values* come from a closed three-kind vocabulary (`Number | Range | Text`)
and the *keys* are legal exactly when the registered class contract says so.
The contract itself is data — `define_content_class` registers a list of
`ParamSpec`s with kinds like `Range{min,max}`, `MaterialName`,
`Choice{options}` — so validation at define time is exactly as strong as a
typed struct would have been, but a new class (evaporite, ley-line
crystal...) needs zero new Rust types, zero new Payload variants, zero ABI
motion. The classes *are* schemas; the schema registry the API principles
promised turns out to be the class registry wearing different clothes.

The bridge keeps the two worlds from drifting: the vanilla geology pack (a
recorded command batch, because content packs are just registry command
batches) is *generated from* the typed dc-core vanilla set, and a test
compiles the registered defs back and asserts member-for-member equality.
Drift is a test failure, not a code review hope.

> blogworthy: "the smallest honest opening" — how to add extensibility to a
> postcard-positional wire union without opening the union: open keys over
> closed values, schemas as registry data, and a generated vanilla pack that
> makes drift impossible.

## Topo-sorting four functions that were already in the right order

Wrapping tectonics → climate → hydrology → history as declared passes was
supposed to be the boring slice. The interesting part was discovering what
read/write declarations *can't* say. Reader-after-writer edges fall out
trivially; the hard case is two passes that both **write** the same
resource. The strata record has three writers (igneous creates it, clastic
deposits on top, the placer reworks the alluvium), and a naive
"writers-then-readers" graph either picks an arbitrary order (silent
nondeterminism — the exact thing this project exists to never do) or
produces edge cycles (clastic reads Strata which the placer writes which
reads Strata which…).

The resolution: split writers into a **creator** (writes without reading —
at most one per resource, enforced) and **modifiers** (read + write). The
creator precedes everyone; modifiers precede pure readers; and modifiers
among themselves must be ordered *by some other resource* — the pipeline
checks pairwise reachability after the sort and rejects the graph if any
writer pair is unconnected, naming both passes. For the v1 roster this
forced an honest declaration that was almost an afterthought: the clastic
pass doesn't just write strata, it produces **Alluvium** (the graded coarse
body + the energy that sorted it), and the placer *reads* that. The
dependency that orders them is a real geological statement, not a scheduling
hack. When a future soil pass and a karst pass both want to rework strata,
the graph will demand the same honesty or refuse to build.

Output-preservation was proven the cheap way: the four legacy declarations
force exactly the old hand order (Kahn's tie-break never fires on that
chain), and the S7 byte-identity suite runs unmodified.

> blogworthy: creator/modifier/reader semantics — why "declare reads and
> writes and topo-sort" is underspecified the moment two processes write one
> field, and how requiring modifier pairs to be connected through *other*
> resources turns scheduling ambiguity into a build error that teaches
> geology.

## What lifting the placer sort revealed

The S8 alluvial harness settled grains with a hand table: gravel 25, sand
15, silt 7, clay 2.5. Lifting it to production meant deriving the threshold
from the property sheet, and the derivation —
`settle_energy = sqrt(grain_size × specific_gravity)` — reproduces the S8
ordering *and* hands over the entire placer mechanism for free: gold-dust
(0.8 mm, SG 16) lands between sand and gravel despite being finer than
either. A placer deposit is nothing but that anomaly. No ore-specific code
exists in the pass; the ore concentrates in the coarse band because its
properties put it there.

The humbling part: the first calibration of property units to column
flow-energy units put the ore threshold at ~29, and channel energies cap at
40 only for enormous rivers — the world-level test found ore in *every*
alluvial body and a barren proximal zone in *none*. The mechanism was
"correct" and dead: the S8 zonation (carried through the channel core, peak
grade in the winnowing band just off it, thinning to the toe) existed only
above energies real rivers rarely reach. One constant moved (threshold ~14,
inside the width range every discharge-qualified river produces) and the
test could suddenly see all three zones. Lesson re-learned from the
`surface_height_m` affair: a plausible mechanism isn't a present mechanism —
quantify where the interesting regime actually sits in the world's own
units, then assert the *shape* (barren → peak → thinning, coarse fraction
monotone down-fan), never the numbers.

Also satisfying: registration-order independence is not an aspiration, it's
a fingerprint equality. Permuting member registration (and even declaring
the classes in a different order) regenerates byte-identical block bytes,
material sidecar bytes, and mixture-table bytes, because members are
canonically sorted by namespaced id before indices exist and every selection
draw is position-addressed. And the abundance-normalization promise ("mods
diversify worlds, never inflate them") turned out to be *structurally* true
at world level: strata thicknesses are climate/energy functions, selection
only decides who fills them, so adding a member provably cannot add a single
voxel of its class.

## Numbers

- Pregen (Medium): 18.3 ms before → 15.3 ms after (noise; the strata work
  happens per-column at collapse time, not in pregen).
- Cold chunk: 0.732 ms mean / 7.12 ms max before → 0.711 / 6.82 after —
  the three strata passes cost a handful of addressed draws per column.
- Materials output rides the decided S8 path unchanged: canonical
  `VoxelContents` via constructors only, interned `mixtures-v0` table,
  palette-compressed `slots-v0` chunks; debris-free chunks still pay zero.

## Deliberately not done (→ Observed)

- Sea-floor and wilds columns keep an empty record (legacy soil band);
  subaqueous sedimentation is the carbonate milestone's business.
- `def_changed`-style events for class/member defines (hot reload signal).
- The dev MCP session token holds only the `dev` namespace; defining `dc:*`
  vanilla needs an explicitly granted token (fine for tests, undecided for
  tooling).
- Client visibility: strata read as Dirt/Stone at block tier by class kind;
  the material sidecar carries the real members for ROADMAP 3c.
- The worldgen `MixtureTable` is per-generator and unbounded; region-file
  grouping (S3 OQ 7) still owns its lifecycle.
