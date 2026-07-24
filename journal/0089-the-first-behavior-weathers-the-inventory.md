# 0089 — the day the working inventory got its first real behavior

The S17 keystone (journal/0088) built the deep cell a *working material
inventory* and a *transformation-fact ledger* — and then nothing called them.
They were tested to the hilt and consumed by no production path: the classic
built-but-unconsumed shape the whole `spines.md` §3 index exists to shame. This
is the slice that makes them real. Subaerial weathering — the corpus's oldest
named cellular behavior — now runs as a **sum-agent pass over the real strata
record**, commits **cause-carrying facts**, and the present collapse **folds those
facts into the material a player digs**. End to end, for the first time.

## What "the sum" means, and why it is not the product

S16 (`weather_behavior.rs`) taught the pass/behavior/`ctx` shape, but it kept the
*product* arithmetic of the legacy height kernel —
`base × biotic × weatherability × frost × taper` — purely to stay byte-identical
to `erosion::weather`. That scaffold was always meant to be discharged, and the
material-behavior spec ratified how (§4, DECIDED 2026-07-24): **agents SUM.**

> `rate = cover_taper × Σ_a (driver_a × susceptibility_{m,a})`

The product is not merely a different number; it is *wrong*. A product zeroes
frost wherever biota is zero — but frost shatters bare rock, that is the entire
point of frost. Only a sum makes each agent's **share** well-defined —
`share_a = cover_taper × driver_a × susceptibility_{m,a}`, `Σ share_a = rate` —
and a well-defined share is what lets the ledger carry **one fact per agent**
instead of one blended fact that has forgotten who did what. "Frost did 3, biotic
did 2" survives as two facts, not a single 5.

As implemented, the three summed agents are:

| agent (`Cause`) | driver | susceptibility |
|---|---|---|
| Chemical (base) | `weathering` (the config rate) | `weatherability` (native axis) |
| Biotic | `weathering × bio_weather[i]` | `weatherability` (**reused — refinement seam**) |
| Frost | `weathering × frost[i]` | `weatherability` (**reused — refinement seam**) |

Chemical reads `weatherability` as its native material axis. Biotic and frost
*reuse* it, because `MaterialProps` carries no distinct per-agent weathering-
susceptibility axis yet — the `LithoResistance` axes are erosion-agent *resistances*
for the height tier, a different model. Growing those axes is a checked material-
sheet extension, and the discipline (`no-bandaid`, "don't balloon the slice") says
leave it as an **annotated seam**, not a speculative axis. Every `Cause::Biotic |
Cause::Frost` arm in `weather_inventory::susceptibility` is where that heir's
obligation reads. This is deliberately **NOT byte-identical** to S16's product —
its acceptance instrument is a walk, not a golden (user, 2026-07-24: *"sum is honest
and faithful; be brave"*).

## The bedrock nobody had

The first wall: the record's units are *all `Loose`*. Weathering is
`Structure → Loose`. There was **no `Structure` in the inventory to weather from** —
bedrock lived only as the scalar `basement: MaterialId`. So the pass would have had
nothing to act on.

The fix is a stand-in, and an honest one: `build_working` now materializes a single
flat basement `Structure` span at the base of every column
(`BEDROCK_SEAM_MATERIAL = GRANITE`, `BEDROCK_SEAM_THICKNESS_M = 50`), purely so the
edge has a source. This is **stub #16** (`docs/design/stubs.md`), a stub on the
*emplacement* axis — real bedrock is plutons and province lithologies unroofed at
real depths, not one flat granite slab — with a loud in-code marker and a named heir:
the genesis/emplacement pass. When that lands, the stand-in is **deleted, not
reimplemented**, and the loose weathering product inherits the bedrock's *real*
identity instead of granite's.

## The log, not the diff

S17 committed facts by *diffing* the mutated inventory against a chapter-start
baseline. That is fine for one edge, but a chapter where three agents drive the
*same* `Structure→Loose` edge has **no unique factorization**: the diff sees one net
delta and cannot recover that three agents contributed. The refined DECIDED
(2026-07-24) says the **applied-edge log is the fact source**: each `apply_edge`
records itself as it runs, stamped with the ctx's `cause` and `chapter`, and
`commit_chapter` **drains the log**, coalescing identical successive edges into one
fact each. Because the weathering pass opens a *cause-scoped* `ctx_for(chapter,
cause)` per agent, three agents on one edge yield three facts, each with its own
cause and its own share.

`diff_facts` did not die — it was demoted to a `#[cfg(test)]` validation check
(`reconciles_with_diff`): for a single edge the drained log must still reconcile
with the net delta the diff recovers. The authority is the log; the diff is the
cross-check.

`Fact::InPlace` grew a `cause: Cause` field. `Cause` is the closed set of deeptime
weathering agents (`Chemical`/`Biotic`/`Frost`/`Dissolution`), with a forward-note
leaving room for a present-tier `Actor(ActorId)` — the player/NPC party the
material-behavior spec's move-fact will one day need, mirroring the existing
`Fact::Move` forward-note.

## The consumer — where weathering finally shows

A behavior that commits facts nobody reads is still built-but-unconsumed. The fold
lives in `geology::deposit_deep_history`: before it lays the recorded pile, it reads
`ctx.deep_weathering_m` — the collapse-tier fold of the deep cell's `base + facts`
(`FactLedger::weathering_product_m`, which composes the bedrock seam and reads off
its `Loose` quantity) — and, if positive, **emplaces a basal weathering-front band**
at the basement contact, below the recorded strata. The band is *not* counted in
`expressed_m`, because that budget accounts for the loose column `H` and the band is
new material the pass produced *from bedrock*, not part of `H`. Off (the default), or
where the ledger is empty, `deep_weathering_m` is `0.0` and the emplacement is a
no-op — the world is byte-identical.

Concretely, on the `one_mineral_unit` fixture: with weathering off the column
collapses to a single coarse-clastic event; with a 1.3 m weathering product it
collapses to **two** events — a fine-clastic saprolite band at the base, the coarse
record unit stacked above it, the record unit's member unchanged. A cut face that
used to read "sediment, then plain basement" now reads "sediment, then a saprolite
rind, then basement". (The product's *class* — fine clastic saprolite — is itself
tied to stub #16: the loose product should inherit the bedrock's identity, but the
deep tier's bedrock is one flat granite, so the class is a stand-in the genesis heir
replaces.)

## The flag, and why byte-identity still matters here

The whole pass is gated behind `DeepConfig::weather_inventory`, default **off** —
the same empty-plane/identity-accessor shape the four deep-sim flags already prove.
Off: no per-cell ledger is built, `DeepField.ledgers` is empty, `ledger_at_voxel`
returns `None`, `deep_weathering_m` is `0.0`, and every collapsed voxel is
byte-identical to pre-slice main. This is not timidity about the ratified
non-byte-identity — the *sum model* is deliberately not byte-identical to the
*product*. It is the S-5 identity default: turning the behavior **on** is a
walk-gated appearance change (a new saprolite band under the world's sediment), and
that flip is the user's to make from a screenshot, not the agent's to bake into
every world. The field-level test proves the flip is **purely additive**: on vs off,
`strata` and `surf` and `regolith` are identical — only the `ledgers` sidecar
appears. The material-transformation layer runs *after* the erosion loop and never
touches the `R`/`H` height weathering, which stays exactly where it was (the §11
continuation slot: height budget and material composition are different authorities,
and unifying them is a reserved later arc).

## The keystone, consumed

`WorkingInventory` and `FactLedger` were tested-only after S17. This slice makes
them a production artifact: the deep-time distillation builds a working inventory
per subaerial cell, weathers it, and stores the resulting cause-carrying ledger on
`DeepField`; the collapse reads it back and expresses it. The A-4 guard — never let
built machinery sit unconsumed — is discharged by the fold being a *real* consumer,
not a demo.

> blogworthy (lenses: deepsim-reflexions; AI-native): the arc from "we built the
> substrate" (S17) to "the substrate carries its first behavior" (this slice) is the
> cleanest possible illustration of seam-first evolution — and of why the log beats
> the diff the moment more than one agent touches the same edge. The sum-vs-product
> decision is a small, sharp example of choosing honesty over a preserved legacy
> number.

## Shapes

Rides **S-9** (derivable base + sparse committed facts — the fold *is* `base +
facts`), **S-2** (facts persist as the compiled artifact; the working inventory is
transient scratch), **S-1** (bounded local relaxation — the per-cell weathering
edge), **S-5** (seams with identity defaults — the bedrock seam and the empty-ledger
/ flag-off floor). Guards **A-1** (the bedrock stand-in is registered stub #16 with a
named heir, loud in code, never silently the truth) and **A-4** (this slice consumes
the keystone). No deviations.
