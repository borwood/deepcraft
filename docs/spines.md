# The spines — recurring shapes, and where they already live

Created 2026-07-22 at the user's instruction, after a session in which the
corpus turned out to be ahead of the assistant **fourteen times**. Not because
the ideas were missing — because they were **already built and lost**.

*Last `spine-audit` sweep: **2026-07-24 (post-Movement-3)** — the weathering-as-a-
process merge (`7545643`) + the `palette_quant_tour` example (`0a8168d`/`4318463`).
Verdicts: **stub #17 is discharged-with-a-residual** (the in-loop half landed; the
"riding / unifying with the height-tier weathering" half of its own heir sentence is
deferred and lives in ROADMAP Movement 2 / material-behavior.md §11+§13.6 — the
discharge note frames that deferral as a satisfied invariant rather than a residual,
see A-2 below). **Three findings in the merged code:** an expired premise on
`BEDROCK_SEAM_THICKNESS_M` (A-2), a second fact-merger written beside the one that
existed (A-4), and a `reads_prev` declaration whose actual epoch is decided by the
id tie-break (S-6 flag). § 3: **no row was emptied** — the S17 keystone never had one
(its consumption is recorded in S-9, and spines.md has never contained the string
`WorkingInventory`); **two rows ADDED** — the exported `DeepField::geotherm` plane
(journal/0093) and **Movement 2a's derived `R`/`H` views** (journal/0092), both
built + tested with no production caller. `fits_in_pores`, `bound_eighths`/`is_occupancy_solid`, drainage
`recv/area/lake`, `exhum`/`t_crust` exported planes, `Agent::Dissolution`, the S11
water module and `column_summary` all re-confirmed uncalled in production this sweep.*

*Previous: 2026-07-24 (the deep-time pass-runner batch — `passgraph.rs` confirmed a
genuine shared kernel, both `pipeline::schedule` [require_creator=true] and
`runner::DeepSchedule::new` [false] call it, A-4 guarded; S-6's runner entry verified
accurate; occupancy row narrowed — the dev inspector now consumes
`free_eighths`/`open_pores`). 2026-07-23 (`column_summary` confirmed dormant and added
to § 3; far-field cold/warm split confirmed real; `weather_behavior.rs` confirmed
wired).*

## What this file is, and how it differs from the others

| file | answers |
|---|---|
| `ROADMAP.md` | what is happening, in what order |
| `docs/ARCHITECTURE.md`, `docs/design/*` | what we decided, and why, with dates |
| `journal/` | what happened, as narrative |
| `journal/corrections.md` | what we believed that was false |
| **this file** | **what SHAPE things take, where that shape already lives, and what exists that nothing calls** |

Decisions were findable this session; DECIDED entries did their job. **Built
machinery was not findable.** There was no index of what exists and what
consumes it, so the same mechanism kept being re-invented next to itself. § 3
exists to close that specifically.

## The rule this file carries

**Work is justified against named shapes.** A plan says which spines it rides
and which anti-shapes it is avoiding. A review checks that claim. A deviation
is not forbidden — it is **loud**: the worker states the deviation and the
argument for it, it goes to main-session discussion, and it ships only with
user ratification, recorded in § 4. Silence is the only disallowed answer.

**Update this file in the same commit as the work that changes it** — the same
discipline ROADMAP and the journal already carry. A spine that gains an
instance, an anti-shape caught in the wild, an item leaving § 3 because it
finally has a consumer: all of it lands with the change, not after.

---

# 1. The spines

## S-1. Bounded derivation with a synthesized coarse frontier

**The actual spine of this engine.** Nothing derives the world; everything
derives a *bounded neighbourhood* and synthesizes plausible boundary conditions
at the frontier rather than recursing.

- far field: an outer ring, budget-bounded meshing (`dc-client/src/farmesh.rs`)
- drainage: decided once at the coarse tier; refinement inherits pinned inflows
  and **never re-routes** (3e-2 decision 1, RATIFIED 2026-07-19)
- S2 collapse: bounded to depth N, frontier conditions synthesized from the
  statistical tier — *"observing one mind must not collapse the planet"*
  (ARCHITECTURE.md § Simulation: tiers)
- erosion halos 16–24 cells (S9); bound-water halo 4–11 cells (S11)

**Rule:** every system declares its halo, and **where there's a cell range,
there's a knob** (user doctrine, water.md § S11 ratification 2).

## S-2. Committed facts vs fluid state

One quantity of truth: facts that leaked to an observer are **committed and
immutable**; everything else is **fluid**, derived as a pure function of
`(seed, position-or-subject, committed facts, time)`, replaying identically.

- the constraint ledger (ARCHITECTURE.md, S2)
- **the chunk store, unnamed**: `HostWorld` pins *edited* chunks and evicts
  *untouched* ones because they re-derive byte-identically (journal/0051)
- water: persist bodies, derive voxels — 39 bytes rebuilt 38 358 wet voxels
  byte-identically (S11 scenario 7)
- the resolved provider table in world identity (DECIDED 2026-07-22)
- **the fact ledger's LAYOUT (journal/0100, 2026-07-25)** — the rule read as a
  storage shape, not only as a data-model choice. `FactLedger` was `Vec<Vec<Fact>>`
  keyed per (cell, slot): every slot paid a 24-byte `Vec` header to record that
  nothing had happened to it — a statement `derive(base)` already makes for free.
  Measured on the production world: **98.8 % of 5.83 M inner `Vec`s empty, 89 % of
  the record's 150 MiB heap in their headers**, against 16.6 MiB of real facts. Now
  flat exact-sized facts + a sparse `(slot, start)` CSR index emitted only for
  slots that *have* facts, so an unweathered cell allocates **nothing**. The same
  rule that says "don't store the derivable value" says "don't store the slot".

**Rule:** *store only what the derivation cannot predict.*

**Status: our most under-exploited spine** — implemented three times under
three names before anyone recognised it as one.

## S-3. A summary is derived from the authority, never beside it

A cheap answer written because a consumer cannot afford the real one must be
*derived from* the real one, never become it.

- compliance (2026-07-22, journal/0074 — **the marquee violation emptied**):
  `collapse.rs::surface_sample`'s surface branch was the far-field summarization
  need that had become the world's surface material rule. The surface voxel now
  **is** the record's top span through `ColumnFill` (`plan(1)`), the same
  authority every buried voxel routes through; `surface_sample`/`draw_class`
  survive only as the far-field *summary* (`coarse_surface`), typed as such and
  held to a **statistical agreement test** (`coarse_surface_agrees_with_the_
  near_column_surface`, 0.9171 near/coarse on geology-surfacing columns) that
  replaced journal/0055's shared-kernel structural guarantee. The
  disappearing-consumer test now answers *yes*: if the far field vanished, the
  surface rule would still exist unchanged, because it is the expression.
- compliance: S15's coarse capacity held against an exact voxel walk
- compliance (2026-07-23, journal/0078): `paleo_temperature` [#11] converted to a
  provider slot — `deposit_deep_history` read *today's* column temperature for a
  deep unit's at-deposition temperature (the summary), beside a sibling axis that
  already read the record (the authority). Now a seam whose identity is that same
  present-day value and whose heir is an epoch-indexed curve; byte-identical.
  Surfaced that the **collapse tier had no `Providers` channel** — the mechanism
  was only in the deep-time sim — now threaded through `WorldGenerator`/`StrataCtx`.
- compliance (2026-07-23, journal/0080 — **built to the shape, not a fix**): the
  runtime perf-observability slice. The `perf::PerfAggregate` (per-span self-time +
  call count), held as the `PerfHandle` resource, is the **authority**. The
  `docs/audits/` ranked-table dump (`write_baseline`) is the FIRST consumer; the
  ratified in-game perf/debug overlay is the named heir that reads the SAME
  resource live. The dump is derived from the aggregate, never a file-only dumper
  the overlay would have to re-instrument — S-3 applied to timing data. The
  disappearing-consumer test answers *yes* by construction: delete the docs dump
  and the aggregate still exists in this shape, because the overlay needs it.
- compliance (2026-07-23, journal/0087 — the block↔material collapse, A1): `block_twin`
  (the fifteen-name match + `_ => Stone`) is **deleted**; `classify` is now
  `dominant_material().map(Material)` — the dominant material IS the authority, the
  block-tier summary between is gone. The collapse also **surfaced and fixed a latent
  S-3 violation** that was invisible while `block_twin` shared it: `generate_chunk`
  derived a buried block from the event's *recorded* member while the voxel interned
  the *dithered* host (two derivations, one truth) — now `classify` of the same
  dithered host. **General lesson: deleting a shared summary reveals the
  disagreements it was hiding, it does not create them** (the far-field cold/warm
  identity split, S-9, is the same family, still open). Tail: the far-field span
  still carries a `Block` token (§ 3 migration tail, not yet a summary violation).

**Rule:** the doctrine test — *"if this consumer disappeared tomorrow, would
this code still exist in this shape?"* (ARCHITECTURE.md § "A summary is not an
authority"). Agreement is **exact** where expression is deterministic and
**statistical** where quantization is deliberately unbiased (S-7).

## S-4. Coarse cause, fine expression — and its mirror

- deep time → collapse: strata recorded at 460 m, expressed per voxel
- karst: the capture decided coarse, the sinkhole and spring placed fine
- **the mirror** — a player diverting a river: **fine cause, coarse
  propagation**; the coarse network carries it once the water leaves the cell

**Rule:** *coarsen the cause, never delete it and fake the appearance*, and
**no simulation-resolution edge may reach the eye as a square or analytic
boundary** (earth-processes.md, DECIDED).

**The user's sharpening (2026-07-22, walk-0071 session): smoothing a verdict
preserves the shape of the cell that voted it.** Anything decided from a bulk
summary of a cell — when one cell marginally crosses a threshold its
neighbors miss — stays a *square phenomenon* under smoothing (a feathered
square is still a square). So the rule's real content is: **threshold late,
at the fine scale, on interpolated causes** — the boundary then follows the
cause's contour, not the grid. Where the cause is non-interpolable (the
strata record — see the S-4 live-violation note below), **dither membership
instead**: unbiased stochastic assignment at the boundary (S-7's medicine at
a different joint), making the transition a statistical gradient rather than
a line. Verdict-smoothing is never the fix.

**Ratified end-state (user, same day): solved by construction, approximately
once. — EXTRACTED 2026-07-22 (journal/0075).** A boundary type at the
sim→expression seam (`dc_core::coarse::CoarseField<T>`) whose fine-scale API
offers exactly the two legal moves — interpolated `sample` (`T: Interpolable`)
and seed-addressed `sample_dithered` (share-vector fields) — and never the
raw per-cell read, so the square is *inexpressible* downstream (the S-6 /
`Option<fn>` pattern: structural, not disciplinary; proven by a `compile_fail`
doc-test that the raw read does not type-check). The type carries the two
teachers' laws: the anchored `Interpolable::blend` exact at identities (A1),
the inverse-CDF `ShareVec::draw` + a `summarize` coarse read + a `DitherSource`
source axis + the cake-law boundary membership dither (B1 + the cake
observation). A1 migrated behind it byte-identically, and the pinned pair
`outcrop_at`+`outcrop_shares` **collapsed to one slot** — the verdict is now a
derived `argmax ∘ outcrop_shares`, the extraction's own law (*categorical
answers are argmax OF the sample, never a stored field*) making the second slot
redundant. Route per seam-first
practice #6: shape-teacher conversions first, then freeze the type and
migrate. **Shape-teacher #2 landed (B1 converted, journal/0073):
`collapse.rs::surface_class` no longer returns the top-window plurality — it
**draws** the surface class from the window's per-class metre shares
(`SALT_GEO_CLASS`), so the 460 m class frontier is an interfingered gradient (the
move-B witness for `CoarseField::sample_dithered`). NEEDS RATIFICATION: the draw
reads the *coherent* bilinear field (not the audit's white noise) because the far
field point-samples it and white noise doubled the far-tile mesh — the honest
unbiased end-state is the far field *summarizing* the shares, which the type
extraction owns.** Residue the type cannot absorb, named rather than hidden: the sim's
internal cell-scale verdicts (cell-honest by design; the principle governs
the EXPRESSION boundary), and conservation-constrained dither choices, which
stay per-quantity physics. Audit: docs/audits/
2026-07-22-threshold-quantization-audit.md.

**A1 converted (shape-teacher #1, journal/0072, 2026-07-22):** the walk-0071
flag — erosion's argmax-then-lookup susceptibility — now blends the per-agent
table by the near-surface window's per-`Litho` *shares* (`providers::
outcrop_shares`, the quantity seam paired with `outcrop_at`'s verdict), so the
coherent rate boundary is continuous by construction (argmax is the limiting
case). The blend is a plain function, not `CoarseField` yet (seam-first #6);
its `exposed_shares → blend_susceptibility` signature is the `Interpolable`
witness the type will be extracted from.

**Live violation:** `DeepField::regolith_at_voxel` samples NEAREST while
`surface_at_voxel` beside it is bilinear, so soil depth is a hard-edged 460 m
mosaic under smooth terrain (ROADMAP Observed, 2026-07-22). *Before "fixing"
nearest→bilinear, read `field.rs:374-383`: nearest is a forced trade-off —
the record is a non-interpolable variable-length unit list and bilinear would
break mass conservation. The fix must route around that, not through it
(2026-07-22 audit).*

## S-5. Seams with identity defaults

A system needing an answer another system will own declares a **provider**: a
named function, an **identity default** reproducing today's behaviour exactly,
and a doc comment naming its **heir**.

- hand-rolled four times before it was named: `biotic`, `erodibility`,
  `full_agents`, `tectonic_history` — each an empty plane plus an identity
  accessor, each with its own byte-identity proof
- named: `deeptime/providers/` (ARCHITECTURE.md § "Provider seams")
- 34 seams inventoried; **5 converted** — `outcrop_at`, `wave_energy`,
  `parent_p`, `depth_to_water`, and `burial_temp_c` (journal/0067: the
  coalification threshold, whose heir is a geotherm)

**Rule:** the fallback must be an **identity**, never an arbitrary constant —
otherwise "absent" and "present but silent" are different worlds and nothing
tells you which one you are in. Pass-level seams need an **agreement test**:
"leave the plane empty" proves the fallback, not the seam (journal/0061).

**Corollary earned by `burial_temp_c` (2026-07-22):** when a stub is a
*threshold*, seam the **quantity it thresholds**, not the verdict. `is_coalified`
would have been simpler and would have answered exactly one rung of a ladder
(peat → lignite → … → anthracite → metamorphic grade) that the same geotherm
answers all of — S-8 applied to seam design. The price is that the identity must
answer in the quantity's units while only knowing the proxy: `burial_temp_c`'s
identity is a **degenerate 1 °C/m geotherm**, i.e. metres wearing degrees, the
same deliberate mismatch `depth_to_water` carries. Pay it in the docstring, and
pin the two halves of the resulting calibration (`COAL_ONSET_C` and the identity)
as retiring **together** — an heir that lands one without the other is a
world-scale defect, not a drift.

## S-6. Declared relations, never incidental order

Order exists; it must be **data**, never an artifact of how a loader enumerated
files.

- `pipeline.rs`: passes declare reads/writes; Kahn's algorithm with a
  lexicographic tie-break; *"never of registration order"*
- `deeptime/runner.rs` (journal/0090, 2026-07-24): the **deep-time pass-runner** —
  the epoch loop's phases (climate, tectonics, and the decomposed erosion
  sub-passes weather/transport/diffuse/agents/deposition, plus biotic) declare
  `{reads, writes}` over a deep-cell axis vocabulary **plus a cadence** (order ×
  rate). Same topo-sort math as `pipeline.rs`, **extracted into the shared
  `passgraph` kernel and called by both — not a second runner beside it** (A-4
  guarded). The biology↔erosion one-epoch lag is a declared **loop-carried edge**
  (`reads_prev`, not handed to the sort); declare it within-epoch and the runner
  rejects the cycle. `climate`'s `remarch_interval` is a low-rate pass. Re-housing
  is byte-identical (the production goldens are unmoved).
- `dc:deep/weather_inventory` (journal/0094, 2026-07-24): the **first *cellular*
  pass** on that runner — declares `reads {Settled|Compensated|Diffused, Frosted,
  Exposed}`, `writes {Saprolite}`, `period = 1`, body a bare `fn`. The crossing
  constraint holds (declaration is plain data + `&'static str` ids + a bare `fn`
  pointer, `runner.rs:596-604` — no closure crosses the seam), and `Saprolite` is an
  honest pure-write sink token in the `Geotherm` mould, so the pass is orderable
  without perturbing the erosion pipeline. **Two declaration defects found by this
  sweep, both cheap to fix and neither behavioural today:**
  1. **`reads_prev: &[BioMod]` is not what happens** (`runner.rs:601`, comment
     `runner.rs:592-593` — *"loop-carried, like `weather`/`diffuse` — last epoch's
     BioMod"*). `weather_epoch` reads `grid.bio_weather`
     (`weather_inventory.rs:307`), a plane `biotic` overwrites in place each epoch
     (`biotic.rs:717`). The pass declares no edge against `biotic`, so which epoch's
     values it observes is settled by `passgraph`'s **id-lexicographic tie-break**
     (`passgraph.rs:152-153`, `ready.remove(0)`): `dc:deep/biotic` sorts before
     `dc:deep/weather_inventory`, so it in fact reads **this** epoch's plane. Rename
     the pass and the physics changes — **incidental order deciding a data
     dependency, the exact thing this spine forbids.** The honest declaration is free:
     `BioMod` as a within-epoch `reads` adds only the edge `biotic →
     weather_inventory` (nothing reads `Saprolite`, so no cycle) and makes the graph
     say what the code does.
  2. **`Exposed` is declared and never read.** `Exposed` is the outcropping-lithology
     susceptibility plane (`runner.rs:74-76`); the pass's susceptibility comes from
     `BEDROCK_SEAM_MATERIAL.props().weatherability`, a constant
     (`weather_inventory.rs:117-127, 232`). Over-declaring a read is safe (it only
     adds order) but it is a false statement on a self-declaring pass, and the
     comment at `runner.rs:363` asserting it reads "the exposed lithology" is simply
     untrue. It becomes true when stub #16's genesis heir gives bedrock a real
     per-column identity — until then, declare it or don't, but don't narrate it.
- members canonically ordered by namespaced id (geology.md)
- patch plugins: declared order, last-in-order wins, **recorded in world
  identity** (DECIDED 2026-07-22)
- effective reads = a pass's declared reads **∪** its providers' reads

**Rule:** incidental order is forbidden; declared order is fine. Naming that
distinction is what freed last-in-order-wins without touching determinism.

## S-7. Distribution-first quantization

Integrate the whole quantity, then slice it. Quantize **once**, at the
boundary, by **unbiased addressed stochastic rounding**.

- `fill.rs`: `ColumnFill::build`, `allocate`, `allocate_partial`; sieve loss
  75.8 % → 0.2 % (journal/0055)
- consequence, unexploited: a 3.5 cm charcoal bed is 0.31 eighths and wins a
  real eighth ~31 % of the time — expressible *because* the rounding is unbiased
- consequence, methodological: agreement tests over expressed data are
  **statistical** — they match an expectation, not an exact value

## S-8. One quantity, many regimes

Do not build two systems and a coupling layer; build one quantity whose
**transitions are the phenomena**.

- water: free / bound — *"one quantity, two regimes"* (DECIDED 2026-07-20); a
  spring, a drip, absorption and waterlogging are all regime transitions
- materials: **substance / form** — not recognised as the same shape until the
  user said *"materials are substance and not form"* (2026-07-22)
- terrain: committed / fluid (S-2)

## S-9. Derivable base + sparse committed facts + fallback query — up to observation-collapse

Generalises S-1 and S-2 into one shape and names its hardest form. **The base of
an answer is *derivable*; a sparse overlay carries the *facts* a derivation
cannot predict; a query returns a fact if present, else derives; every answer
carries a resolved/resolution flag.** Two regimes on one axis:

- **deterministic-derive** (the easy degenerate case) — the base is a pure
  function; re-derive gives one answer. Facts are edits appended on top.
  - provenance: a voxel's history re-derives from the deeptime compile; only
    *edited* voxels append facts (DECIDED 2026-07-23). This **is** ROADMAP
    Sequenced item (d), and the fact-overlay **is the save layer** — S-2's
    edited-chunk pinning is the same overlay under another name. **BUILT (deeptime
    tier) 2026-07-24, journal/0088:** the deep-cell fact-ledger — `commit_chapter`
    diff-and-append (apply-time edge logging is the ratified fact source; each fact
    carries its `cause`), byte-identical under the identity default over 25,600 real
    cells, provenance read = `base + facts`. **CONSUMED as a LIVE PROCESS 2026-07-24
    (S18 → Movement 3, journal/0094):** `collapse.rs:1487` reads
    `FactLedger::weathering_product_m` into a basal weathering-front band, and the
    facts are now grown by the **`dc:deep/weather_inventory` runner pass running
    inside the deep-time loop, every epoch, accumulating** on each epoch's live
    terrain (`weather_inventory::weather_epoch` → `finalize_ledgers`) — replacing
    S18's post-hoc one-shot (`field.rs::build_ledgers`, deleted). The band is now
    ≥1 voxel; **stub #17 discharged** (the plumbing stub is a real behavior). Still
    **gated behind `--weather-inventory` (default off, S-5 identity default →
    byte-identical)** — a walk-gated appearance change, not a stub.
    **The residual, named here so it is not lost (audit 2026-07-24):** #17's heir
    sentence asked for two things — the per-epoch in-loop pass (*landed*) **and**
    "riding / unifying with the height-tier weathering". The second is *deliberately
    deferred*: the pass reads `H` and writes only the ledger sidecar, so `dc:deep/
    weather` still owns the `R`/`H` budget (the two-authorities split,
    material-behavior.md §11). That deferral has a home — **ROADMAP "Movement 2 —
    R/H unification"**, material-behavior.md §11 continuation slot + §13.6, and
    `docs/spikes/movement2a-rh-unification-plan.md` — but the split is **not yet the
    invariant the discharge note calls it**: `DeepField::derive_regolith_at`
    (`field.rs:545-551`) materializes the "one authority" `H` view from
    `FactLedger::empty_with_bedrock`, i.e. **with an empty ledger**, so the derived
    view structurally cannot see the 6.09 m of `Loose` the M3 process committed.
    Movement 2a's claim that "the inventory is the authority and `R`/`H` are its
    materialized views" (`field.rs:521-537`) is therefore true of the *record* half
    only; M3 added inventory content no view reflects. Two derivations of one
    quantity that cannot agree is this section's own consistency law — the
    unification is what closes it. The
    **runtime tier** (edits as facts over
    the gen-derivable base; a break = a move-fact) is the same shape one tier down —
    designed (commit-as-facts), unbuilt; the inspector proved the runtime stores only
    `Block` and re-derives contents (the derivable base, edit-blind).
  - S3 skylight: `column_summary`'s `fully_resolved`/`sky_exposed`
    (`dc-core/src/column.rs`) — base derived from resident data, unknown volumes
    poison the resolved flag. **Built and tested, dormant** (no lighting consumer;
    sim-light unbuilt) — **confirmed and moved to § 3 by the 2026-07-23 sweep**
    (only caller is the `--bench-storage` timing harness).
- **observation-collapse** (the general, unsolved case) — the base is an
  *underdetermined proposition* (many consistent answers); a **read triggers a
  write**: the query samples one answer and must *pin* it or the next query
  contradicts it. "Where is the duke?"; "is the northern province on fire?" The
  sample draws from a **seed owned by the query context** (entropy doctrine — no
  wall clock), so replay/multiplayer collapse identically. The deferred frontier
  is the **consistency web**: pins are not independent (the duke's location
  constrains who rules the province) — lazy-CSP / DF-generates-history-on-zoom.

**Rule:** *store only what the derivation cannot predict* (S-2), and **a collapse
must not contradict the base it refines or any prior pin** (the consistency law).
A committed-fact or collapse system that does not route through this shape is
reinventing it — the loud check.

- **live violation (the law's motivating counterexample):** the far-field LOD
  reconstructs a coarse box's *material identity* differently on cold-synthesize
  vs warm-reduce (ROADMAP Observed, 2026-07-23) — two derivations of the same base
  **disagree**, exactly what the consistency law forbids. A violation to fix
  (under Crux 1's far-span migration), not an instance done right.

---

# 2. The anti-shapes

## A-1. A stand-in becomes the definition

Four shipped instances found in one audit: `surface_sample`'s branch,
`Litho::reference_material`, `fill.rs::is_loose`, `bio_resist`.

**Check:** S-3's disappearing-consumer question, asked at write time.

**Why the stub inventory misses it:** a stub *looks* like a fake — it says
placeholder, it names an heir. A leaked requirement **looks like working code
that passes tests**.

## A-2. A justification outlives its premise

Charcoal excluded because no bed survived whole-voxel quantization — true until
partial voxels. `reference_material` fixed so packs could not move terrain —
true until the content-set freeze, **the same day**.

**Check:** § 5's convention. Note that decisions expire premises *elsewhere*,
so a sweep must ask "does the cited constraint still hold?"

- **instance (2026-07-24, caught by the post-M3 sweep):
  `BEDROCK_SEAM_THICKNESS_M`'s justification expired the day Movement 3 merged.**
  `inventory.rs:425-429` still reads *"a made-up depth chosen only to be an
  **effectively-inexhaustible** `Structure→Loose` source **over the one chapter this
  slice weathers**"*. Weathering is no longer one chapter: `dc:deep/weather_inventory`
  fires every epoch for the whole run and **accumulates**, and the measured
  production band at the argmax cell is already **6.094 m of the 50 m seam (12 %)**
  (journal/0094). The number is no longer a shrug — it is a **live ceiling on how
  deep saprolite can get anywhere in the world**, and it will bite first exactly
  where the process is strongest (thin cover, long subaerial residence, or a cranked
  `--erosion-budget` walk). Textbook A-2: the decision that expired the premise was
  in a different file, the same day. **Proposed wording:** *"…chosen to outlast the
  accumulated `Structure→Loose` draw of a full run (measured max 6.094 m of 50 m at
  production scale, journal/0094) — it is a **ceiling on saprolite depth**, not an
  inexhaustible source; the genesis/emplacement heir supplies a real per-column
  unroofing depth and retires it."*
- not-an-instance (2026-07-23, journal/0078): the 2026-07-22 audit flagged the
  `exhum`/`t_crust` comment [#28] as an A-2 ("claims the collapse tier reads
  them"), but the comment already said "WILL read … currently consumed by
  nothing" (rewritten `11d43859`, *before* the audit) — the audit quoted it with
  "WILL" dropped. Prose that already states "consumed by nothing" is not a
  justification outliving its premise. Tightened to cite § 3 anyway; corrections
  #40. **The check works the other way too:** verify the *quotation*, not just the
  claim about it.

## A-3. A test green for a reason unrelated to what it asserts

corrections #27 (a stale artifact served as fresh: exit 0, every suite `ok`,
the code never built) and #32 (function-pointer identity folded by the
optimizer).

**Check:** verify by test **name and count**, never by `test result: ok`. Ask
*"did it run?"* separately from *"did it pass?"*

## A-4. Built machinery with no consumer and no index

See § 3. **This is the anti-shape this file exists for.** Its sibling failure — the
one the header calls this project's characteristic defect — is **a second mechanism
written beside the one that already existed**:

- **instance (2026-07-24, post-M3 sweep): two fact-mergers, 300 lines apart —
  DISCHARGED 2026-07-25 (journal/0096, FLOW slice 1).** Folded exactly as
  prescribed: `commit_chapter`'s lookup now searches the slot instead of matching
  `facts.last_mut()`, and `weather_inventory::coalesce_facts` is **deleted**. The
  two were verified equivalent by their own suites, green **by name** across the
  fold (`one_fact_per_agent_per_firing`, `weathering_accumulates_across_epochs`,
  `bedrock_facts_key_stably_as_the_record_grows`, and the byte-identity golden
  `the_production_world_still_hashes_to_the_pre_slice_goldens`): the search merges
  into the *earliest* match, which is the first-occurrence order the post-hoc sweep
  produced, and `Σ fraction_m` — the only thing the band composes from — is
  invariant either way. The O(slots) post-pass is off the per-cell, per-epoch path.
  *Original entry, preserved:*
  `inventory.rs::commit_chapter` (`593-624`) already merges a drained edge into the
  preceding fact when `(chapter, cause, from, to)` match — but only against
  `facts.last_mut()`, so it merges *consecutive* edges only. Movement 3 fires three
  interleaved agents per epoch, which defeats that, so it added
  `weather_inventory.rs::coalesce_facts` (`263-281`) — **the same merge, keyed on
  the same tuple, generalized to non-consecutive** — and calls it in a loop over
  every slot after every commit (`254-256`). The right shape is one merger:
  generalize `commit_chapter`'s lookup from `last_mut()` to a search over the slot
  and delete `coalesce_facts`. Semantically identical (the merge is order-insensitive
  and mass-preserving), and it removes an O(slots) post-pass from a per-cell,
  per-epoch path. Caught while it was six lines old, which is the whole point of
  sweeping after a merge rather than a month later.
- **discharge-by-porting (2026-07-25, journal/0100) — the shape working as
  intended.** `FactLedger`'s residency defect wanted a sparse layout, and the
  reflex is to design one. It was **not designed**: `deeptime/flux.rs`
  (journal/0096) had shipped a larger sparse record as flat exact-sized arrays +
  a CSR index *the day before*, with its index floor measured at 0.056× of total,
  so the slice's whole job was to read that file and do the same thing. The one
  deliberate divergence is stated where it lives (`flux.rs` can afford a *dense*
  offsets array because every cell exists; the ledger cannot, because dense
  offsets per slot **is** the rectangle it is avoiding — so its rows carry their
  own slot key). Recorded here because A-4 is usually written up as a failure,
  and "the in-tree precedent was found and copied" is the outcome it exists to
  produce.
- **not-an-instance, noted for the record:** `weather_epoch` open-codes
  `grid.r[i] + grid.h[i] <= sea_level` (`weather_inventory.rs:310`) where
  `DeepGrid::surf_at` (`grid.rs:412-413`) exists — but so do five other sites in
  `erosion.rs`. That is a pre-existing idiom, not a new mechanism; it belongs to
  whoever next touches `erosion.rs`, not to M3.

## A-5. Locality of cause mistaken for locality of effect

"Connectivity only changes where someone edits" — **true**. "…therefore the
consequence is inside the loaded set" — **false**: one edit joined a body 288 m
away, and a container over never-generated ground is the *default* case
(S15, corrections #31).

**Check:** state cause-locality and effect-locality as two separate claims, and
measure the second.

## A-6. Measuring what cannot change a decision

The user cut S15's "measure the storm" group: *there is no world in which the
answer comes back "the storm is fine."*

**Check:** before specifying a measurement, name the action each possible
outcome leads to. If they are the same action, it is theatre.

---

# 3. Built, and nothing calls it

The index A-4 exists for. **Audit this against the codebase regularly** — see
the `spine-audit` skill. Leaving this list is a *good* event: record what
consumed it and when.

| what exists | where | called by | intended consumer |
|---|---|---|---|
| `fits_in_pores` / `K_PORE` — DECIDED, built, 7 tests green | `dc-core/src/materials/packing.rs` | **no production caller** (the module now *declares* its expected consumers in-code — journal/0078 — so an infiltration author finds it, but still nothing calls it) | **hydrology** (groundwater infiltration) + **diagenesis** (cement / ore deposition) — the transport-time depositing processes |
| `recv` / `area` / `lake` — final drainage, populated in every world — **SUPERSEDED 2026-07-25 (journal/0096, FLOW slice 1)**, retirement sequenced | `deeptime/field.rs` | **still nothing in production**; `recv` now has one *test* consumer, the agreement check `flux_record.rs::the_receiver_export_agrees_with_the_final_chapters_faces`, which pins it as a shadow of the flux record rather than a rival authority | The intended consumers moved to the record that replaced it. `recv` is one out-edge per cell — a spanning tree that **cannot represent divergence at all** — and was "the *last* routing" after 200 discarded epochs. Its heir is `DeepField::flux` (row below). **Deletion of `recv`/`area`/`lake` and `Cell::{flow_to,river,discharge}` is continuation (e)** of the FLOW arc, not done here (this slice touched no expression) |
| **`DeepField::flux` — the face-flux record** (FLOW slice 1, journal/0096): per-chapter directed flux on 3D faces, the representation that supersedes the receiver tree. **Row added the day it was built, deliberately** — the slice is the *recording* half only | `deeptime/flux.rs`; `deeptime/field.rs` (`flux`); `dc:deep/flow_record` on the runner | **no production consumer, ON PURPOSE** — only `tests/flux_record.rs` and `examples/flux_record_probe.rs`. Expressing it now would mean a second fake river beside the honest absence of one, which is the exact thing flow.md § 3 forbids | **refinement as a boundary-value problem** (continuation (b): face fluxes as Dirichlet conditions, the load budget as mass, solved *inside* a cell) · §13.8's flow-biased sub-cell fill · the paleo-channel/layering read. The record is *"the rich thing others bias to, never a second geometry model"* |
| `exhum` / `t_crust` — populated since U8; the doc comment is now **honest** (states "consumed by nothing", cites this row — journal/0078; the 2026-07-22 audit's "claims the collapse tier reads them" quoted an already-corrected comment with its "WILL" dropped — corrections #40) | `deeptime/field.rs` | **no consumer of the exported plane** (`t_crust` *is* read inside the sim by `isostasy()`, `erosion.rs` — the unconsumed thing is the exported plane, not the value) | metamorphic grade (stubs.md § 4) — and, since 2026-07-22, a **named socket** to arrive through: the geotherm heir of `providers::burial_temp_c` reads crustal heat flow (journal/0067). Still unconsumed; it now has an address |
| occupancy primitives — `bound_eighths`, `is_occupancy_solid` (the two still uncalled; `free_eighths`/`open_pores` now read by the dev inspector `dc-api/src/payload.rs:367`, `loose_eighths` by `dc-client/src/meshing.rs:281` — 2026-07-24 sweep) | `dc-core/src/materials/contents.rs` | `bound_eighths`/`is_occupancy_solid`: **no production caller** | the four sim consumers named at authorship: water fill, loose gravity, compaction, sim light — none built yet (the inspector/meshing reads above are the dev HUD + render path, not those) |
| **Movement 2a's derived `R`/`H` views** — `DeepField::derive_regolith_at` / `derive_bedrock_at`, `WorkingInventory::derived_regolith_m` / `derived_structure_stock_m`, `surface_regolith_m` / `structure_stock_m` (journal/0092, the positional cave-excluding §13.6 rule, 4 tests green) — **row added by the 2026-07-24 post-M3 sweep** | `deeptime/field.rs:545,560`; `deeptime/inventory.rs:764,776,787,807` | **no production caller** — only `tests/rh_unification.rs:52,79` (the byte-identity agreement tests). Production still reads the scalar `surf`/`regolith` planes; the views exist to *prove* the inventory is the authority, not yet to *be* it | **the R/H unification** (ROADMAP "Movement 2", material-behavior.md §13.6): retire the scalar planes into the inventory so `H`/`R` derive. Until then this is the authority-half of a two-authorities split (see S-9's M3 residual), and `derive_regolith_at` reads an **empty** ledger, so it cannot see `dc:deep/weather_inventory`'s facts |
| `DeepField::geotherm` — the exported `temperature` condition-field plane (°C/m per cell), **added 2026-07-24 by journal/0093**; the field's doc already says so honestly (`field.rs:312-321`) but nothing pointed at this index — **row added by the 2026-07-24 post-M3 sweep** | `deeptime/field.rs:321` (cloned from `run.grid.geotherm`, `field.rs:382`) | **no consumer of the exported plane** — only `tests/geotherm.rs` and `resident_bytes`. The *in-sim* value **is** consumed: coal rank reads `grid.geotherm` at run finalize (`biotic.rs:765-775`). Exactly the `exhum`/`t_crust` shape, one field newer | **metamorphic grade** (`exhum` = P, this = T → schist/gneiss/marble; ROADMAP "Metamorphism — now UNBLOCKED by the geotherm") and the measurement probes |
| `Agent::Dissolution` + `LithoResistance.dissolution` | `deeptime/lithology.rs` | **zero call sites** in production (`examples/entry_species_probe.rs:325` is a probe; the *separate* `inventory::Cause::Dissolution` is deliberately excluded from `WEATHERING_AGENTS`, `weather_inventory.rs:61-63`) | karst — hard-gated on a carbonate that does not exist |
| the S11 water module | `dc-worldgen/src/water/` | **not on the production path** | free/bound water |
| pass-graph `Resource` vocabulary | `pipeline.rs` | 8 passes | 26 of 34 inventoried seams are **value-level and invisible to it** |
| `column_summary` / `open_air_below` / `ColumnSummaries` — the S3 skylight query (S-9 deterministic-derive), built + 6 tests green | `dc-core/src/column.rs` | **no production caller** — only the `--bench-storage` timing harness (`dc-client/src/bench_storage.rs`); `docs/API.md` lists a `world.column_summary` query but **no dc-api handler exists** for it (2026-07-23 sweep) | **sim-light / skylight** — the "is this column under open sky" query; the lighting/sim-light consumer is unbuilt |

**Departed (the good event):**

- `MixtureDownsampleRule` / `derive_material_lod_chunk` — **consumed
  2026-07-22 by FF2b-minimal** (journal/0070): the client's far pyramid
  (`dc-client/src/farpyramid.rs`) derives coarse material chunks through it
  for every fully-generated node, and the far mesh renders
  `classify(contents)` of the reduced mixtures
  (`dc_core::farfield::node_column_spans`). First row to leave the index.

---

# 4. Carve-outs

A deviation from a named shape ships only after: a **loud plea** stating the
deviation and the argument for it · **main-session discussion** · **user
ratification** · an entry here, dated, with the reasoning.

Undocumented deviation is the failure. A ratified one is just a decision.

**1. (2026-07-22) B1's class draw uses the coherent bilinear source, not
white noise — a bounded S-7 deviation with a named heir.** White noise (the
unbiased prescription) doubled far-tile mesh (21.5→43.5 MiB: coarse far
sampling aliases white noise into unmergeable speckle); runtime-is-sacred
chose the coherent source the journal/0058 member dither already uses
(+13 %, in budget). Costs, on the record: a small toward-50/50 bias
(interpolated uniforms are middle-heavy — the dice-sum CDF distortion), and
the user's cake observation (minority phases guillotine at cell perimeters
because the SHARES are still nearest — see ROADMAP Observed). **Heir: the
`CoarseField` extraction** — far-summarize register (agreement statistical),
boundary source-cell membership dither, and/or a CDF-corrected source retire
both costs. Ratified by the user in-session with the observation attached:
*"a lot better than before… the salient information is that the bilinear
field we already use doesn't behave in a strictly satisfactory way."*

> **Amendment flagged by the `CoarseField` extraction (2026-07-22, journal/0075,
> corrections #39 — NEEDS RATIFICATION): the bias sign is the opposite of
> "toward 50/50".**
> Measured through the inverse-CDF draw in dc-core
> (`white_noise_is_unbiased_but_coherent_amplifies_the_majority`), a middle-heavy
> `u` gives `F(s) > s` for the class straddling the cumulative-½ point, so a
> 2-class cell renders the **majority amplified and the minority
> under-represented** — the mix is pushed *away* from 50/50, not toward it. The
> coherent source therefore **sharpens** minority phases (compounding the cake
> observation, not easing it), rather than "flattening mixes toward 50/50". The
> minorities still *appear* (better than the plurality's zero — journal/0073's
> cream specks are real), but at less than their true areal share. This does not
> change the ratified decision to ship the coherent source (the far-mesh
> argument stands); it corrects the *characterisation* of its cost, and
> re-weights the heirs: the far-`summarize` register and/or a CDF-corrected
> source are the honest fixes, and the boundary membership dither eases the
> *perimeter* guillotine but not the *within-cell* under-representation.

---

# 5. The justification convention

When a comment justifies a design by citing a constraint, **name the
constraint**, so a later reader can ask whether it still holds instead of
having to notice. Both A-2 instances would have been caught the day their
premises changed.

    // JUSTIFIED-BY: packs may be added to an existing world and must not move
    // terrain. (RETIRED 2026-07-22 by ARCHITECTURE.md § content-set freeze.)

A `spine-audit` sweep greps these and asks, one by one, whether the cited
constraint is still true.

**Drift check (2026-07-24): the literal marker has zero instances.** A corpus grep
for `JUSTIFIED-BY` returns this file and the skill that describes it — nothing in
`crates/`. That is not a failure of the *convention* (the corpus does name its
constraints, loudly and in prose: `STUB #16`, `REFINEMENT SEAM:`, "consumed by
nothing, cites spines § 3"), but it does mean the grep the convention promises finds
nothing, and every A-2 caught so far was caught by **reading**, not grepping — this
sweep's `BEDROCK_SEAM_THICKNESS_M` instance included. Either the marker earns its
first real uses on the next stub-adjacent constant, or this section should be
rewritten around the prose form that is actually in use. Left as a **flag to the main
session**, not a unilateral rewrite (§ 4's rule binds the auditor too).

## A-7. Naming a content identity inside a process

**DECIDED 2026-07-22 (user):** *"default is just the first content pack. So if
you feel a need for naming a material directly: that's a want for a feature
that makes the process more robust. Naming directly will never, in any world,
be correct."*

The backbone this enforces is `geology.md`'s: **"Processes bind to classes, not
instances. Vanilla geology is just the first geology pack."** A process that
names `Charcoal`, or `SANDSTONE`, or any specific identity has bound itself to
one pack's roster and is wrong for every other pack — including ours, later.

**This is not a ratifiable carve-out.** Unlike other deviations it does not go
to § 4 for the user's blessing, because there is no world in which it is
correct. It is a defect. What *is* a live question is the one underneath it:
**which capability is the process missing?**

**The check, and it is a diagnostic rather than a prohibition:** when you feel
the need to write a content name into a process, stop and ask what property of
that content you are actually reaching for. That property is the feature the
process lacks. Name it, and the special case dissolves for every future member
of the same family.

**Worked instance (2026-07-22, shipped and owed a rework).** `litho_of_tag`
gained `Biofacies::Charcoal` as a hardcoded exception so a 3 cm fire lamina
would not define a 460 m erosion cell's lithology. The *reasoning* was sound
and never mentioned charcoal: **a unit too thin to dominate the cell should not
define its rock**. That is a **thickness** rule. Charcoal is structurally capped
at 0.04 m so it is *always* a lamina — the first member of a family, not an
exception, and volcanic ash and marker beds arrive next. The rule belongs in
`outcrop_at`'s identity (already a provider seam, identity `units.last()`),
where it applies to every thin bed with no name in it. **Reworked journal/0068,
2026-07-22:** both name-keyed exceptions deleted, mirror restored to total, and
`exposed_litho` now returns the lithology dominating a 0.9 m near-surface window
— charcoal never outcrops (0 of 297 025 cells), ash/marker beds handled for free.

**Two more instances the principle finds on its own** — corroboration that it is
real rather than a slogan, since both were independently flagged by the seam
inventory: `Litho::reference_material` (six named rocks standing for every
material in the world) and `classify::block_twin` (fifteen named materials, with
a `_ => Block::Stone` arm that silently swallows any pack's additions).

---

# 6. The audits (`docs/audits/`)

Standing inventories, produced by read-only sweeps. Like `docs/spikes/*`, they
hold **measured numbers and file:line citations** — consult before re-deriving.

- **`2026-07-22-seam-inventory.md`** — all **34 seams**: what each stands in
  for, the owing system, the identity fallback (**19 of 34 are arbitrary, not
  identities** — flagged individually), pass-level vs value-level granularity,
  blast radius, and a ranked shortlist of 12. Also the finding that
  `pipeline::Resource` declares *fields*, so **26 of 34 are invisible to the
  pass graph**.
- **`2026-07-22-deeptime-vector-audit.md`** — all **24 state-mutating vectors**
  in the deep sim: reads, writes, whether they reach the record, material- and
  form-awareness. Headline: **form-awareness is zero across all 24**, and only
  6 write anything a player can dig.
- **`2026-07-22-hydrology-priors.md`** — the exhaustive water corpus sweep,
  including **16 priors that contradict or constrain** a unified-transport
  framing, and 20 items of genuine blank space.
- **`2026-07-22-threshold-quantization-audit.md`** — the S-4 square-verdict
  sweep: every site where a bulk-cell verdict can reach the eye, classified
  threshold-late / dither-membership / harmless / already-compliant (4/3/3/6,
  plus 7 sim-internal and 2 conservation-pinned residue), with the
  `CoarseField<T>` by-construction recommendation, the shape-teacher
  sequencing, and the finding that it and the octree node payload's sampling
  vocabulary should be ONE type.

**These were nearly lost.** They lived in a session scratchpad and were cited
all day; nothing in the corpus pointed at them. That is **A-4 committed on the
day A-4 was written** — proof that an index only helps if the artifacts it
indexes are *in the repo*. Any future sweep lands here, in the same commit as
the work that used it.
