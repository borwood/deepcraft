# The spines — recurring shapes, and where they already live

Created 2026-07-22 at the user's instruction, after a session in which the
corpus turned out to be ahead of the assistant **fourteen times**. Not because
the ideas were missing — because they were **already built and lost**.

*Last `spine-audit` sweep: **2026-07-29 (evening), at `72fbe86`** — a **FULL** re-check, because
**the reference side moved**: `docs/spines.md` itself changed in ten commits since the `96ab14b`
watermark, so every prior verdict was void by the watermark file's own `_full_run_triggers` rule.
Delta: the RATE slice (journal/0123), the SCHEDULE slice (journal/0124), the member-#0 far slice
(journal/0125), the artifact-tripwire net (journal/0126), the `runner.rs` history extraction, and
the two applied-sweep commits. Audit: `docs/audits/2026-07-29-spine-audit-full.md`. **Findings:**
1. **NEW § 3 ROW — the RATE AUTHORING PATH has no production authorer.** Every production path
   hands `CadenceTable::empty()` (`field.rs:608`, `mod.rs:175`, `runner.rs:619`); the authoring
   verbs `CadenceTable::{with, unmatched}`, `Cadence::sub_turned` and `Schedule::integrated_dt`
   have **zero production callers** at `72fbe86`. `mod.rs:186-190` says so itself — *"the
   parameter exists and the loader does not."* **The discipline was applied one commit later and
   not here:** journal/0124 added a row for `Schedule::Seed` *the day it was built*; journal/0123
   added none for its own unconsumed half. Row added below.
2. **A citation-accuracy claim falsified BY ITS OWN COMMIT.** The `CoarseField` row's `where`
   cell reads *"verified at those exact lines 2026-07-29 at `96ab14b`; **the adoption did not
   move them**"* — written **in `0dcdadb`, the adoption commit, which moved two of the five**
   (`sample_dithered` 479→498, `summarize` 515→534). Corrected in place. This is A-2's
   *"did it ever?"* variant landing on a sentence whose whole subject is citation accuracy, in
   the same diff — the sharpest available argument for the skill's *"cite the commit you read it
   at"* rule, since a stamped commit is exactly what makes the error visible.
3. **Two of the three A-2 instances the `96ab14b` sweep filed are FIXED, and both entries still
   read as live** — `draws.rs`'s completeness overclaim (fixed `f10dc03`) and `Erosion::diffuse`'s
   *"the entire shipped configuration"* clause (fixed, `erosion.rs:3150-3160`, with a standing
   check `the_shipped_world_has_cells_past_the_bound`). Stamped below. *This is the third
   consecutive sweep to find this same failure mode in this file — see the 2026-07-29 morning
   block's finding #2. A sweep that writes a finding is writing a claim that will be fixed; it
   owes the entry a shape that can be stamped, and "NEW instance" is not that shape.*
4. **And the `draws.rs` fix is SHORT BY ONE SALT.** The sweep's finding named four surviving
   hand-rolled salts; the applied correction's residue 3 names only the three `SALT_TEC_*`.
   **`SALT_DT_PERTURB` (`refine.rs:29`, live call site `:180`) is named nowhere in `draws.rs`** —
   verified at `72fbe86` by `grep -rn "SALT_" --include=*.rs crates/dc-worldgen/src`. Reported,
   not applied (source file).
5. **`runner.rs`'s narrative citations have RE-DRIFTED WHOLESALE, eight hours after being
   refreshed.** The morning slice verified every non-§3 citation at `ec9858e`; the Schedule slice
   then moved `runner.rs` again (+824 lines net across the batch). Every S-6 line ref is off by
   20–60 lines. Refreshed here at `72fbe86` — **and the honest lesson is that refreshing them is
   not a durable act**: this file's `runner.rs` refs have now been refreshed three times in eight
   days and were stale within one merge each time.
6. **S-6 never listed the SCHEDULE axis** (journal/0124) — a ratified sum type on the pass
   declaration, and the axis that deleted the *"pre-loop, then skip epoch 0"* rule. Only § 3's
   row mentioned it. Added as an instance below.
7. **§ 3: no row removed. One row added** (finding 1). All fourteen prior rows re-confirmed
   uncalled in production at `72fbe86`; searches recorded per row.*

*Previous: **2026-07-29 (morning), at `96ab14b`** — the first sweep with a recorded
watermark (`docs/audits/.sweep-watermarks.json`; the key was `null` until this run, so this was
a **full pass**, not a delta). Delta since the previous sweep's tree: journal/0108–0122 plus the
2026-07-28 baseline batch and the 2026-07-28 bootstrap-content removal. **§ 3: no row added, no
row removed — all thirteen re-confirmed uncalled in production at `96ab14b`** (searches recorded
per row below; each is a `grep -rn --include=*.rs crates/` for the symbol with the defining file
excluded). **Findings:**
1. **S-10's first instance cited a path that does not exist** — `deeptime/sat.rs`; the file is
   `dc-worldgen/src/water/sat.rs`. Fixed below, **and the correction matters more than the
   typo**: the corrected path lands inside the S11 water module, which § 3 lists as *"not on
   the production path"*. The spine's exemplar instance is **unconsumed machinery**, and the
   entry's own argument (*`sat.rs` is stable only because `lateral_c = 0.25` happens to sit
   inside its bound*, verified `water/sat.rs:82`) is therefore a statement about code no world
   runs. That does not weaken the spine — two instances is still two — but a reader picking an
   exemplar to copy should know which one ships.
2. **Four `spines.md` entries still read as LIVE defects that were fixed days ago** — S-6's two
   `weather_inventory` declaration defects, A-2's `BEDROCK_SEAM_THICKNESS_M`, A-2's
   `flux_record_probe` caption, A-2's `S17` layout staleness. **The file's own header said all
   four were applied; the bodies did not.** This is CLAUDE.md read-first item 5's
   *one-directional pointer* failure committed inside the doc that indexes it — a cold reader
   enters at the section, not at the header. All four stamped in place this sweep.
3. **NEW A-2 instance: `draws.rs`'s completeness claim is false** (`crates/dc-worldgen/src/draws.rs:23`,
   *"There is no expression anywhere in worldgen that turns a number into randomness"*).
   Four hand-rolled salts survive with eight live `draw_f64` call sites; its § *"The one honest
   hole left, and the residue of the other"* names only one of them. See A-2.
4. **NEW finding in a file changed the day before this sweep: `Erosion::diffuse`'s doc comment
   contradicts its own journal entry** — see A-2.
5. **Line-reference drift is now the doc's largest correctness liability.** `field.rs`,
   `inventory.rs` and `runner.rs` have each moved by hundreds of lines since their citations
   were written (`FactLedger` cited at `inventory.rs:239-259`, actually `:659`;
   `derive_regolith_at` cited at `field.rs:598-606`, actually `:818`; `chapters` cited at
   `field.rs:367`, actually `:581`). **§ 3's refs are refreshed at `96ab14b`; the narrative
   sections' are NOT** — refreshing ~90 citations is a slice, not a sweep, and every one of them
   is a `git log -S` away for whoever needs it. Recorded as owed rather than silently half-done.
   **✅ DISCHARGED the same day — see the applied block immediately below.***

*Applied 2026-07-29 (a slice, not a sweep — verified at **`ec9858e`**, the merge of the sweep
above), the two items that sweep was not permitted to apply:*
1. ***S-6's argument is rewritten*** *— the pointer banner is folded into the section body.
   Its title's thesis (order is DATA) is unchanged; the section now teaches
   **author-and-validate** (`ARCHITECTURE.md`, DECIDED 2026-07-26) while stating plainly that
   the in-tree mechanism is still **derive-and-reject** and that authored order is
   `dependency-graph.md` **E7**, sequenced after E3/RATE.*
2. ***Every `file:line` citation OUTSIDE § 3 was verified against `ec9858e`*** *— **68 checked**
   (54 file-qualified, 14 bare-`:line` continuations), **20 refreshed**, **16 left as dated
   as-of testimony** with the drift now named in the entry that carries them.
   **One cited file no longer exists at all:** `pregen/history.rs` (`:82`, `:84-88`), deleted by
   journal/0121; the A-2 entry citing it is annotated in place rather than retargeted.
   `erosion.rs` refs (`:3132-3134`, `:3148`) were verified against MAIN as of `ec9858e` only —
   that file is in flight elsewhere.*

*Previous: **2026-07-25 (post-FLOW batch)** — **line refs in this block are as-of 2026-07-25
and have all drifted** (re-checked at the `pass-declaration-history.md` extraction: the `BioMod`
reasoning is `runner.rs:560-567`, the
un-declared `Exposed` note `:569`, `BEDROCK_SEAM_THICKNESS_M` `inventory.rs:1278-1290`,
`DeepField::chapters` `field.rs:575-581`); left as written because this is a dated sweep record,
not a live pointer — the five merges since
the last sweep: FLOW slice 1 (`0096`), the weathering profile (`0099`), the `FactLedger`
CSR (`0100`), FLOW (a) the head field (`0098`), and `identify(pos)` (`0101`).
**Both findings of the 2026-07-24 sweep were applied** and verified in code: the
`reads_prev: [BioMod]` fiction is now a real within-epoch `reads` with the reasoning
at `runner.rs:460-467`, `Exposed` is un-declared with the heir named
(`runner.rs:469-472`), and `BEDROCK_SEAM_THICKNESS_M`'s expired premise is rewritten
verbatim as proposed (`inventory.rs:593-605`). **§ 3: no row emptied, three rows
touched, ONE row ADDED** — `DeepField::chapters`, built since U8, self-documented as
*"exported and read by nothing"* at `field.rs:359-366`, and **missed by every prior
sweep** including the two that added rows for its neighbours `geotherm` and
`exhum`/`t_crust`. `flux`/`head` re-confirmed unconsumed-on-purpose; `recv`/`area`/`lake`,
the `R`/`H` views, `fits_in_pores`, `bound_eighths`/`is_occupancy_solid`,
`Agent::Dissolution`, the S11 water module, `column_summary` and the 8-pass `Resource`
vocabulary all re-confirmed. **Three findings:** `dc:deep/head` under-declares (S-6,
below) and its `reads_prev` is pinned by nothing because **`reads_prev` is consumed by
no mechanism at all** — it appears in `runner.rs` and nowhere else in `crates/`
(**both findings are now CLOSED — journal/0104 gave the kernel the anti-dependency
edge, and journal/0107 closed the `dc:deep/head` under-declaration of its LIVE reads;
note this sweep's own prescription for the latter, "declaring `Forced` is free and it
pins it", was HALF WRONG — see S-6**);
`dc:deep/flow_record` by contrast is **honest**. An expired caption in
`flux_record_probe.rs` (A-2). And, outside this file's remit but reported to the
integrator: **all four in-code `stubs.md #19` markers are stale** — they were written
by `8c08d43` before the collision renumber and now point at the water-table stub
instead of the weathering-profile one.*

*Previous: **2026-07-24 (post-Movement-3)** — the weathering-as-a-
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
  (ARCHITECTURE.md § Simulation: tiers). **The shape is ratified; the running
  instance is not currently running** — the toy engine's last production caller
  went with the bootstrap history pass on 2026-07-28 (§ 3, journal/0121). Cited
  here as the *shape* the live sim's far tiers will re-instance, not as code on a
  path today
- erosion halos 16–24 cells (S9); bound-water halo 4–11 cells (S11)

**Rule:** every system declares its halo, and **where there's a cell range,
there's a knob** (user doctrine, water.md § S11 ratification 2).

## S-2. Committed facts vs fluid state

One quantity of truth: facts that leaked to an observer are **committed and
immutable**; everything else is **fluid**, derived as a pure function of
`(seed, position-or-subject, committed facts, time)`, replaying identically.

- the constraint ledger (ARCHITECTURE.md, S2) — the **rule**; its S2 toy
  implementation has had no production producer or consumer since 2026-07-28
  (§ 3). The rule's live instances are the four bullets below
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
  **Extended one level up (journal/0102, 2026-07-25):** don't store the *cell*
  either. The record was still `Vec<FactLedger>` — an owning struct per cell,
  48 B × 297,025 = **13.60 MiB** to say nothing, with 75.8 % of cells saying it.
  One grid-wide `LedgerField` with the cell as a CSR row; a cell is read as a
  borrowed `LedgerView`, and no call site changed. The corollary the two slices
  produce together, which is the transferable part: **a per-cell container is a
  gen-time shape.** Per-cell is right *while compiling* (`FactLedger` is still the
  accumulator — a grid-wide insert would memmove every fact after the cell, every
  epoch); it is wrong the moment it *ships*. Compact at the seam where the compile
  ends, which in this codebase already exists and is called `finalize_*`.

### S-2's storage corollary — **the house layout for sparse per-cell data**

Named 2026-07-25 by the sweep, because it has now been reached for **three times in
four days and never once designed** — and an unnamed pattern is one an author
re-derives. It is recorded here rather than as its own spine because it *is* S-2's
rule read as a layout ("don't store the slot" is "don't store the derivable value"),
and splitting it out would split the rule. **Whether it earns its own S-number is a
question for the main session, not the auditor** (§ 4 binds here too).

**The shape:** a **flat, exact-sized payload array** + a **sparse index** — never
`Vec<Vec<T>>` keyed per (cell, slot), and never the dense rectangle. Two variants,
and which one is a *measured* choice, not a taste:

| variant | index | use when | instance |
|---|---|---|---|
| **dense row-pointer** | `Vec<u32>` of length `n + 1`; row `i` is `start[i]..start[i+1]` | **every row exists** — the row key is the array position | `FluxRecord::{entries, cell_start}` (`deeptime/flux.rs:415-424`), keyed per cell |
| **keyed rows** | `Vec<{key, start}>`, one entry per **non-empty** row, ascending | rows are themselves sparse — a dense pointer array *is* the rectangle you are avoiding | `FactLedger::{facts, rows: Vec<SlotRun>}` (`deeptime/inventory.rs:659-664`, `SlotRun` at `:674-679`), keyed per stratum slot |

**The numbers that make it a rule rather than a preference** (don't re-derive them):
the CSR **index floor is 0.056× of total residency** (S19-flow-record-cost-results
§ 3), so the index is never the constraint — the payload is; and the anti-pattern's
cost is **98.8 % empty inner `Vec`s, 89 % of the record's heap in their headers**
(journal/0100), 311.02 → 179.12 MiB when it was removed.

**The check, and it is the A-4 check:** before designing a sparse layout, grep the
tree for the one that already exists. The ledger conversion did exactly that and is
recorded under A-4 as *discharge-by-porting*. A fourth author writing a fourth layout
is the failure this row exists to prevent.

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

- compliance (2026-07-25, journal/0101 — **the doctrine INVERTED, and the tier
  nobody had audited**): `dc-api`'s `has_contents` was a **chunk-level** summary worn
  as a **voxel-level** authority. `HostWorld::contents_at`'s `Option::None` — the only
  channel that could mean *"no record for this voxel"* — had been spent on
  `chunk_contents`'s whole-32³ condition, which exists because the **mesher** wants a
  grid or nothing. The query inherited the mesher's shape and, silently, its
  *meaning*: an unrecorded basement voxel reported `has_contents: true` +
  `classified: dc:air` over solid stone (**6.4 %** of near-surface solid voxels,
  corrections #49). The disappearing-consumer test names the culprit exactly — if the
  mesher vanished, `contents_at` would not be shaped this way. Fixed by
  `HostWorld::identify(pos) -> Identity` (`dc-api/src/identify.rs`): `contents_at` is
  demoted **in its own doc** to the raw source, and every reporting surface derives
  from `identify`. **Note the direction:** where the other S-3 instances are a
  *summary* standing in for an *authority*'s content, this is a summary standing in
  for an authority's **resolution** — same rule, and it is not enough to ask *"is
  this derived from the authority?"* without also asking *"at what granularity was
  the authority asked?"*

- compliance (2026-07-26, journal/0110 — Movement 2b): **`litho_of_tag` demoted
  from authority to default.** A recorded unit's rock was *inferred from the
  environment measured at deposition* — the `DepTag → reference_material` shortcut
  `material-behavior.md` § 13.7 said would half-dissolve. The inference is not
  wrong, it is **blind to provenance**: it structurally cannot know that a distal
  cell has no gravel to drop because the gravel rained out upstream. `DepUnit` now
  carries `species` — the material that actually arrived, written by the only agent
  that can know it, the pass that carried it — and `litho_of_tag` survives as the
  **default fill** for every depositor that carries no load (the wind and wave
  agents, the biotic layer, pedogenic overprint, coalification). Off the flag the
  default is used everywhere, so the demotion is byte-identical and the record's new
  axis costs nothing (it fits `DepUnit`'s existing padding, asserted). Note what the
  *measurement* then found: on the shipped world the identity that travels reaches
  **0.025 m of a 440,595 m archive**, because fluvial transport is 0.109 % of this
  world's sediment routing — the authority is now correctly shaped and has almost
  nothing to say, which is a fact about the landscape and not about the shape.

**Rule:** the doctrine test — *"if this consumer disappeared tomorrow, would
this code still exist in this shape?"* (ARCHITECTURE.md § "A summary is not an
authority"). Agreement is **exact** where expression is deterministic and
**statistical** where quantization is deliberately unbiased (S-7). And a summary
can wear an authority's clothes by **resolution** as well as by content — ask
both questions.

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
once. — EXTRACTED 2026-07-22 (journal/0075); FIRST ADOPTED 2026-07-29 (E5 member
#0, journal/0125) at the FAR site only.** *The two words are kept apart on
purpose: "EXTRACTED" alone read as done to every sweep for seven days (§ 3's own
indictment of this line), so the status now names which half shipped and which
site. The near site (U3) is still un-adopted.* A boundary type at the
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
unbiased end-state is the far field *summarizing* the shares, which **since
2026-07-29 belongs to the octree node contract, not this type's adoption** (user
ruling, member-#0 MM-4: LOD machinery stays engine and the far/LOD synthesizer is
a second executor `refinement.md` does not model). ⚠ The coherent-source deviation
is therefore **not** discharged by member #0 — the adopted far path inherits it
unchanged, at both salts.** **That hand-rolled draw is now the type itself
(journal/0125): `surface_class` calls `sample_dithered` over a
`CoarseField<ShareVec<6>>`, and the cake law arrived with it — the membership
dither of the source cell, which the hand-rolled version never had. ⚠ And it is a
**cell-wide blend, not a perimeter treatment**: integrated over a cell,
`E[weight on the home cell] = 9/16`, so ~44 % of far samples read a NEIGHBOURING
deep cell's shares everywhere. The type's own doc said *"away from a boundary one
weight ≈ 1"*, which is true at the centre and misleading in aggregate; it misled
the first adopter and only the near/far agreement test caught it (0.8225 → 0.7922,
floor re-derived 0.80 → 0.70 from the mechanism). `coarse.rs` now carries the
integral beside the sentence.** Residue the type cannot absorb, named rather than hidden: the sim's
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
nearest→bilinear, read `field.rs:788-806` (**refreshed 2026-07-29 evening at `72fbe86`**; cited
`:735-744` when written): nearest is a forced trade-off —
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
- ~~34 seams inventoried~~ **31 LIVE, of 34 inventoried** *(corrected 2026-07-29, baseline
  sweep S8/C2 — the seam inventory retracted its own count on 2026-07-28:*
  `docs/audits/2026-07-22-seam-inventory.md` *banner, "later docs cite '34 seams' and that
  count is this audit's, not today's. **Live seams: 31, not 34.**" The banner on the cold end
  could not reach the docs that quote it, and this is the read-first one.
  **The `5 converted` numerator was NOT re-verified against `crates/` and is carried
  forward as-is** — quote it as this file's, not as a fresh count)*; **5 converted** — `outcrop_at`, `wave_energy`,
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

*Argument rewritten 2026-07-29 to state the ratified shape (the 2026-07-29 baseline sweep
left a pointer here and flagged the rewrite as owed; this is that rewrite, applying a
decision already made — not making one).*

Order exists; it must be **data**, never an artifact of how a loader enumerated
files. **Since 2026-07-26 it must be data on the WORLD** — authored, not inferred.

`ARCHITECTURE.md` § *The engine is plugin-agnostic, and pass ORDER is authored*
(DECIDED 2026-07-26, user) chose **AUTHOR-AND-VALIDATE** over **DERIVE-AND-REJECT**.
Order is chosen per world, alongside seed and epoch count; `{reads, writes}` stop being
the **ordering input** and become the **validator** — a pass reading a resource nothing
writes, a read satisfied only by a later pass (a lag: make it explicit), a genuine cycle.
Filed as `journal/corrections.md` **#65**, narrated in `journal/0119`.

The section's title survives that decision intact and is the reason for it: **order is
data.** What the decision withdraws is the claim that the *kernel* is where that data comes
from. ARCHITECTURE.md's own two sentences carry it: *"an engine cannot derive a third
party's intended order"*, and requiring it to try *"means the declarations must encode that
order, and encoding it in engine-owned resource names is precisely how `DeepAxis` came to
exist. The mechanism and the violation are the same choice."*

**⚠ The mechanism IN THE TREE is still derive-and-reject** — this section describes real
code, and the code predates the decision:

- `passgraph.rs`, the shared kernel: classify each resource's writers, build the edges,
  and **topo-sort by Kahn's algorithm with a lexicographic-id tie-break, *"never
  registration order"*** (`passgraph.rs:1-20`; the tie-break is `ready.sort_by_key(id)`
  + `ready.remove(0)` at `passgraph.rs:230-231`) — rejecting duplicate ids, multiple
  creators, cycles, and **writer pairs the declarations leave unordered**
  (`GraphError::AmbiguousWriters`, `passgraph.rs:88`).
- `pipeline.rs` (the pregen DAG, `require_creator = true`, *"never of registration
  order"* at `pipeline.rs:25`) and `deeptime/runner.rs` (the epoch loop,
  `require_creator = false`) **both call that one kernel** — not a second runner beside
  it (A-4 guarded).

Authoring the order is **E7** in `docs/dependency-graph.md` (*"authored order + the
validator"*), **sequenced after E3/RATE**; retiring `DeepAxis` so packs declare their own
resource ids is **E6**, sequenced with it. Until those land, read the instances below as
*the best available compliance under derivation* — **not** as the shape to copy into new
design.

**The exemplar, reframed.** The deep-time runner's revision tokens — `Forced → Incised →
Weathered → Diffused → Compensated → Windblown → Settled` (`runner.rs:89-90`, the enum at
`:111`; **refreshed 2026-07-29 evening at `72fbe86`** — cited `:66-95`/`:97` at `ec9858e` eight
hours earlier, moved by the Schedule slice) — were cited here as this section's hardest-won compliance. They are the **artifact
of derivation.** N passes that transform one shared terrain *in place* are unschedulable
under `AmbiguousWriters`, so each stage must write a distinct token naming its output, and
**the chain of tokens IS the order**. Ask the graph why weathering runs after incision and
it answers *because `Weathered` consumes `Incised`* — and that token exists **because
weathering ran after incision** (journal/0119). It is the hand-declared canonical order
re-encoded so the graph appears to compute it, and the re-encoding is what put the default
pack's pass roster inside an engine enum: `DeepAxis`, which ARCHITECTURE.md names **"the
violation."** The id-lexicographic tie-break is the same pressure one level down — an
ordering decision the *kernel* had to invent because no author had made it, which is how a
pass **rename** came to be able to change the physics (the three audits below).

**What survives, and it is most of this section.** Under author-and-validate the
declarations **validate an authored order instead of generating one**, and *a validator
needs honest declarations exactly as much as a generator did*. corrections #65 says which
parts are kept: the pass-graph kernel, `reads`/`reads_prev`, the WAR/RAW distinction
(journal/0104) and **every rejection class** — *"they become the validator rather than the
generator"*; only the claim that the order is *derived* is withdrawn. So the
anti-dependency mechanism below and every declaration-honesty audit in it stay live: an
under-declared read, an over-declared read, and a lag nothing consumes are defects against
a validator too. What does **not** survive is *"the tie-break settled it"* as a resting
place — against an authored order there is nothing left for a tie-break to decide.

**Instances — declaration honesty, live under either model:**

- `deeptime/runner.rs` (journal/0090, 2026-07-24): the **deep-time pass-runner** —
  the epoch loop's phases (climate, tectonics, and the decomposed erosion
  sub-passes weather/transport/diffuse/agents/deposition, plus biotic) declare
  `{reads, writes}` over a deep-cell axis vocabulary **plus a cadence** (order ×
  rate). Same topo-sort math as `pipeline.rs`, **extracted into the shared
  `passgraph` kernel and called by both — not a second runner beside it** (A-4
  guarded). The biology↔erosion one-epoch lag is a declared **loop-carried edge**
  (`reads_prev`, handed to the sort as a reader→writer **anti-dependency** since
  journal/0104 — see the entry below; it was handed to nothing before that); declare it
  within-epoch and the runner rejects the cycle. `climate`'s `remarch_interval` is a
  low-rate pass. Re-housing
  is byte-identical (the production goldens are unmoved).
  - **The RATE half is real since 2026-07-29** (journal/0123): cadence is **authored data**
    (`deeptime/cadence.rs`'s `CadenceTable`, applied over each pass's declared default by
    `deep_passes_with`), a firing pass takes its declared **sub-turns** with state carried
    between them, and `dt` is consumed by the rate-shaped passes rather than assigned and
    ignored. Still byte-identical under an empty table — the goldens are *still* unmoved, which
    is what let the axis land against journal/0122's fixed point as a hash comparison.
    *~~`declared_passes` is the one place the table is applied~~ — **corrected 2026-07-29
    (evening), the two functions were the wrong way round.** `deep_passes_with`
    (`runner.rs:640-651`) is the one place the table is **applied**; `declared_passes`
    (`runner.rs:655`) is the one place a pass is **declared**, and its own comment states the
    reason — *"split out so `deep_passes_with` has exactly one place to apply the table and cannot
    miss a pass"*. The property the sentence was reaching for is real and survives the fix: a new
    pass cannot be added past the table.*
    - **⚠ The AUTHORING half has no authorer — new § 3 row, 2026-07-29 (evening).** Every
      production path hands `CadenceTable::empty()`. `mod.rs:186-190` states it plainly: *"the
      parameter exists and the loader does not."* The seam is deliberate (state 3, held as a
      candidate); it is listed so the absence is indexed rather than only self-declared.
  - **The SCHEDULE axis is the second half, and it is a first-class instance of this spine**
    (journal/0124, `ARCHITECTURE.md` § Schedule, user-ratified): a pass's schedule is a **sum
    type** on its declaration — `Seed` · `Step(Cadence)` · `SeedAndStep(Cadence)`
    (`deeptime/schedule.rs:82-88`) — because *a `seeded: bool` cannot express run-once-only*.
    What it **deleted** is the reason it belongs here: the runner used to seed three passes
    pre-loop *in `mod.rs`* and then skip them at epoch 0, i.e. **membership of the setup epoch
    was an artifact of which file called what, and a pass's first firing was decided by hosting
    rather than by declaration.** The audit that shipped it found all three incumbents were
    *first-steps in disguise*, so nothing seeds today and the roster asserts it by name
    (`runner.rs:1698`, `tests/schedule_axis.rs:131`). Order-as-data read one level in: not *when
    relative to other passes*, but *whether this firing integrates time at all*. `Seed` /
    `SeedAndStep` are § 3's `Schedule::Seed` row — **executed, undeclared**, state 3.
  - **The declaration ARCHAEOLOGY now lives in `docs/design/pass-declaration-history.md`**
    (extracted 2026-07-29 under the file-size doctrine — `runner.rs` was **1,694 lines**,
    2.4× the 700-line source threshold, and its cold half was 0090/0104/0107 narrative).
    **The CONTRACT text this section quotes stayed in the code**, at the declaration sites
    it governs; what moved is the defect walkthroughs, the superseded defences and the
    slice-neutrality arguments a test now proves. **journal/0123's RATE commentary is live
    contract and was left whole.** Line refs below are refreshed to that extraction commit.
    *Honest limit: the file is **still** 2.7× over (**1,908 lines at `96778eb`**, raw count —
    the integrator first "arbitrated" two sweeps' 1,654-vs-1,908 disagreement with a
    blank-skipping counter and got 1,818; the doc-topology figure was right all along and
    the stale 1,654 predates the Schedule slice; the **927 code lines** figure also dates
    from the extraction commit)
    — comment extraction cannot fix its size, and the remedy left on the table is an
    ordinary module split. Not sequenced.*
- `dc:deep/weather_inventory` (journal/0094, 2026-07-24): the **first *cellular*
  pass** on that runner — declares `reads {Settled|Compensated|Diffused, Frosted,
  Exposed}`, `writes {Saprolite}`, `schedule: Schedule::EVERY_EPOCH` *(field renamed
  `cadence` → `schedule` by journal/0124; quote refreshed 2026-07-29, doc-topology F5 —
  the same-day citation refresh caught line numbers and missed the rename)* (`period = 1` until
  journal/0123), body a bare `fn`. The crossing
  constraint holds (declaration is plain data + `&'static str` ids + a bare `fn`
  pointer, `runner.rs:963-971` — **refreshed 2026-07-29 evening at `72fbe86`**, was `:929-936` at
  the extraction — no closure crosses the seam), and `Saprolite` is an
  honest pure-write sink token in the `Geotherm` mould, so the pass is orderable
  without perturbing the erosion pipeline. **✅ BOTH DECLARATION DEFECTS BELOW ARE CLOSED —
  verified in code 2026-07-29 at `96ab14b`** (they were fixed 2026-07-25 and this section never
  said so; the file header did, ~400 lines up, which is the one-directional pointer failure).
  `runner.rs:566-571` (**refreshed 2026-07-29 evening at `72fbe86`**, was `:560-567`) now carries
  the reasoning verbatim — *"a `reads_prev` declaration was a
  fiction: with no edge … `biotic → weather_inventory`, and nothing reads `Saprolite`, so there
  is no cycle"* — and `runner.rs:573` states *"**`Exposed` is deliberately NOT declared**:
  susceptibility is a constant off …"*. **The two entries below are preserved as the record of
  what was found and why; neither is a live defect.** *(**Line refs inside the two entries
  below are as-of 2026-07-24 and have ALL drifted** — re-checked 2026-07-29 **(evening) at
  `72fbe86`**, after the Schedule slice moved `runner.rs` again:
  `runner.rs` has moved by hundreds of lines (the `weather_inventory` declaration is now
  `:963-971`, its reasoning `:566-573`, the `Exposed` axis `:123` — these read `:929-936`,
  `:560-572`, `:107-109` at the `pass-declaration-history.md` extraction eight hours earlier,
  which is how fast a `runner.rs` ref rots); `passgraph.rs`'s
  tie-break is `:230-231`, not `:152-153` (**unmoved since morning** — `passgraph.rs` was not
  touched by this batch); `weather_epoch`'s `grid.bio_weather` read is
  `weather_inventory.rs:283` and `biotic`'s in-place overwrite is `biotic.rs:727` (**both
  unmoved**). Left
  unrefreshed on purpose — this is dated testimony about what was found, not a live
  pointer.)* **Two declaration defects
  found by the 2026-07-24 sweep, both cheap to fix and neither behavioural then:**
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
- **the two FLOW passes (journal/0096 + 0098), audited 2026-07-25 — one honest, one
  under-declared, and a systemic hole under both:**
  1. **`dc:deep/flow_record` is HONEST** — verified line by line against its body
     (`runner.rs:483-495`; the declaration `:818-833`; **both refreshed 2026-07-29 evening at
     `72fbe86`**, were `:479-495`/`:789-802`). `reads: [Routed, Energy, Head]` covers `erosion.recv()`,
     `.area()`, `.routed_surface()` (the drainage solve → `Routed`), `.out_load()`
     (transport → `Energy`) and `grid.head_exchange` (→ `Head`); `writes: [FlowFlux]`
     covers `ctx.flux` and nothing else; and its claim not to read the strata record
     (`runner.rs:476-478`) holds — `slot_for_chapter` runs at `finish`, outside the
     pass. Reading `Head` is a **declared** edge, not a tie-break, which is the
     correct form of exactly what the 2026-07-24 sweep caught being done wrong.
     *Residue, shared with `dc:deep/weather_inventory` and pre-existing:*
     `erosion.current_chapter()` is the chapter stamp `tectonics` writes
     (`DeepAxis::Forcing`) and no pass that reads it declares it. Transitively
     ordered, so not behavioural — but it is a project-wide idiom, not an exception.
  2. ~~**`dc:deep/head` UNDER-DECLARES its within-epoch reads.**~~ **CLOSED
     2026-07-25 (journal/0107), user-ratified with the consequence attached.**
     `reads: [Routed]` covered `filled`/`routed_surface`/`area` and **not** the
     ground surface `R + H`, which the body builds from `grid.surf_at` and the solve
     uses as its seepage cap, its lake datum and its whole free-surface boundary
     (built at `runner.rs:529-537`, consumed at `head.rs:462-495`; **refreshed 2026-07-29 evening
     at `72fbe86`**, were `:524-530`/`:456-485`). The comment's defence, *"its position never affects the
     terrain"*, was true and was **not the question**: it affects the field's own
     values, and the vertical flux recorded from them. **The revision is `Forced`,
     in every cfg path** — nothing between `forcing` and `transport` mutates `R`/`H`
     — and it is also the revision the pass *should* read, because `filled`,
     `routed`, `area` and `ground` must describe one landscape or the seepage cap
     and the free-water anchors sit on two.
     - **This entry's own prescription — *"declaring `Forced` is free and pins
       it"* — was HALF WRONG, and the ROADMAP's *"verify that claim before trusting
       it"* is what caught it.** Free: yes. Pins it: **no.** A `reads` edge on a
       revision token orders you after the pass that *produced* it and says nothing
       about the pass that overwrites the same plane next, because that pass writes a
       **different token** — a different resource to the graph. Every erosion pass
       survives this because a forward edge into the stages after it braces the far
       side; a **sidecar** (writes only its own field, its one reader downstream of
       the terrain anyway) has no brace and floats.
     - **The fix is the PAIR:** `reads: Forced` (after the writer that produced it)
       **plus** `reads_prev: Incised` (before the writer that supersedes it). And a
       new axis: `dc:deep/transport` mutated `R`/`H` while declaring only its
       `Energy`/`DeltaH` by-products, leaving the revision chain with a hole exactly
       where the ground surface first changes each epoch. `DeepAxis::Incised` closes
       it (`transport` writes, `weather` reads, `head` lag-reads).
     - **The generalisation:** where one plane has several revisions per epoch,
       *declaring the revision you consume pins one side only*. Pin the other with an
       anti-dependency on the **next** revision of that plane — the next, never the
       last, or you leave yourself free to slide past every writer before it.
     - **Schedule-neutral, proven directly** by
       `the_terrain_revision_declarations_are_schedule_neutral` (five rosters rebuilt
       with the pre-slice declarations, order identical), and the vertical-flux record
       is unmoved to the entry: 307,364 entries, 44.301 % of cells, 60 artesian.
  3. ~~**The systemic hole: `reads_prev` is declared, typed, documented — and consumed
     by nothing.**~~ **CLOSED 2026-07-25 (journal/0104)** — the fix flagged here is the
     one that shipped. It appeared in `runner.rs` and in no other file in `crates/`;
     `passgraph.rs` never received it, so a `reads_prev` claim was true only by
     accident of the rest of the graph, and renaming `dc:deep/head` to any id sorting
     after `dc:deep/deposition` would have silently flipped it to reading THIS epoch's
     record. Same defect class as the previous sweep's `BioMod`, one level up: there
     the tie-break decided a *read*, here it decided an *epoch* — and unlike `BioMod`
     it could **not** be fixed by declaring honestly, because a `reads` edge points
     the wrong way (see the anti-dependency entry below).
- **the anti-dependency edge — `reads_prev` is a MECHANISM** (journal/0104,
  2026-07-25; user-ratified). `passgraph` now takes **two edge kinds**, and the
  distinction is the spine stated in graph terms:
  - `reads` — a **true dependency** (RAW). "I need the value this schedule produces":
    every writer is ordered **before** the reader.
  - `reads_prev` — an **anti-dependency** (WAR). "I need the value from *before* this
    schedule ran". Because the deep-time planes are overwritten **in place**, that is
    only true if the reader runs **before** the writer, so the edge points
    **reader → writer** — the reverse direction.

  Folding a lagged read into `reads` is therefore not a smaller fix, it is the wrong
  one: it orders the reader *after* the writer (the value it explicitly did not want)
  and closes the loop-carried feedback into a within-epoch cycle the kernel rejects.
  Proven both ways in `passgraph::tests` —
  `folding_a_lagged_read_into_reads_reverses_it_into_a_cycle` and
  `a_lagged_read_outranks_the_id_tie_break` (which carries its own **negative
  control**: the same roster with the lag undeclared schedules the wrong way round,
  which is what the pre-slice behaviour was). The roster-level guarantee is
  `runner::tests::a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename`:
  every lagged reader renamed to an id that **provably loses** the tie-break against
  its axis's writers, and still scheduled first. `reads ∩ reads_prev` on one node is a
  named rejection (`GraphError::ContradictoryLag`) because against *itself* the two
  opposed edges would cancel as a dropped self-edge and pass silently.
  **The production order is byte-unmoved** — all twelve new edges (from six lagged
  readers) pointed from a pass already ahead of its target, i.e. the physics was right
  and merely unenforced, which is the outcome this spine exists to produce. `pipeline.rs` hands `reads_prev: &[]`
  *structurally*, not as a placeholder: a one-shot DAG has no previous value.
  ~~**Residual (ROADMAP Owed):** `dc:deep/climate`'s lagged terrain read is still
  undeclared.~~ **CLOSED 2026-07-25 (journal/0107): `reads_prev: [Forced]`** — an
  anti-dependency against the **first** terrain revision of the epoch, one slice, no
  cfg selection. The obvious declaration (the *last* revision — `Settled` /
  `Compensated` / `Diffused`, three cfg-selected slices) would pin strictly less: it
  would leave `climate` free to slide past `forcing` and `transport`. Lag against the
  first writer and the rest of the chain is covered transitively. Schedule-neutral by
  construction (`climate → forcing` is already a true forward edge) and proven so.
- members canonically ordered by namespaced id (geology.md)
- patch plugins: declared order, last-in-order wins, **recorded in world
  identity** (DECIDED 2026-07-22)
- effective reads = a pass's declared reads **∪** its providers' reads

**Rule:** incidental order is forbidden — that has never changed, and naming the
distinction is what freed last-in-order-wins without touching determinism. What changed on
2026-07-26 is **where declared order comes from**: **authored, per world, and validated**
by `{reads, writes}` — never inferred from them. **A brief citing S-6 to justify a new
derive-and-reject design is citing it wrong**, and must cite `ARCHITECTURE.md` § *The
engine is plugin-agnostic, and pass ORDER is authored* alongside it.

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
  - **the transitions became DECLARED and COMPILE-ENFORCED 2026-07-25
    (journal/0108, S20 option 2c).** If the transitions *are* the phenomena, then
    the set of legal transitions is a thing the code should be able to state and
    check — and until this slice it could not: a `Fact` stored four free endpoint
    bytes, so it could name any `(material, form) → (material, form)` pair
    whatsoever, including the null edge that moves nothing. It now stores an
    `EdgeId`, whose **only** constructor is `EdgeId::declared`, which returns
    `None` unless `is_declared_edge` holds (material-behavior.md §3's graph: 5
    forms → 20 directed edges, plus the same-form material-change class; the null
    edge and `Void → Void` are refused). A fact therefore **structurally cannot
    name an undeclared transition** — and `InvCtx::apply_edge` refuses the move
    outright rather than performing a change it could not honestly write down
    (`an_undeclared_edge_does_not_run_at_all_not_merely_goes_unrecorded`). This is
    also S-6's rule applied to a data axis rather than to pass order: what edges
    exist is **declared**, never whatever a caller happened to pass.
- **transport: one load, several movers** (material-behavior.md § 13.2, *"an agent
  moves material along a driving field"* — water, wind, ice, gravity). Named as a
  family in the design since 2026-07-24; **first cashed out as one 2026-07-26
  (journal/0112)**, when hillslope creep became the second member on the *same*
  species multiset, the *same* composition seam and the *same* `split_by_shares`
  budget as the fluvial pass, differing only in its driving field and its
  competence curve. **And the difference is the phenomenon, which is what makes it
  this spine and not merely code reuse:** gravity's competence curve is that
  **there isn't one**. Creep is diffusive rather than selective, so it moves the
  donor's whole composition in proportion — and *that* is why a colluvial apron is
  poorly sorted and locally derived while an alluvial bar is sorted and
  far-travelled. Two facies out of one mechanism with one knob absent. Building
  creep as a bespoke system would have had to *invent* the contrast; building it as
  a regime got it for free. The remaining members (wind's low ceiling → loess and
  dunes, ice's indiscriminate one → till) are the same shape with a third and
  fourth curve, and the shipped eolian agent is still a bespoke system waiting to
  be folded in.
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
    (S18 → Movement 3, journal/0094):** `collapse.rs:1467` (**refreshed 2026-07-29 evening at
    `72fbe86`**; `:1417-1420` before the member-#0 slice added ~50 lines above it) reads
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
    (`field.rs:900-905`, **re-checked 2026-07-29 evening at `72fbe86`**; `:818-824` at `ec9858e`
    that morning) materializes the "one authority" `H` view from
    `FactLedger::empty_with_bedrock`, i.e. **with an empty ledger**, so the derived
    view structurally cannot see the 6.09 m of `Loose` the M3 process committed.
    Movement 2a's claim that "the inventory is the authority and `R`/`H` are its
    materialized views" (`field.rs:876-892`, refreshed from `:796-810`) is therefore true of the *record* half
    only; M3 added inventory content no view reflects. Two derivations of one
    quantity that cannot agree is this section's own consistency law — the
    unification is what closes it.
    **The overlay's REPRESENTATION narrowed 2026-07-25 (journal/0108, S20 option
    2c), and nothing was deleted.** The resident `Fact` went 16 B → **8 B with zero
    padding** by two levers that only pay in company (an `EdgeId` for the four
    endpoint bytes, `f32` for the fraction) — every axis survives: chapter (*when*),
    cause (*who*), edge (*what*), fraction (*how much*). The compaction the residency
    crisis invited was an **axis drop**, and an axis drop is A-1 wearing a fact's
    paperwork (S20 § 6); this is the version of "make it smaller" that S-9 permits,
    because the sparse overlay still carries every fact a derivation cannot predict.
    The narrowing happens **exactly once, at persist**
    (`LedgerField::from_accumulators`) — the gen-time `FactLedger` accumulator stays
    `Fact<FracM>` at 16 B *precisely so* the per-epoch `*q += share` never rounds, so
    the error is a single rounding (measured max relative 5.766e-8, at f32's own
    2^-24) rather than an accumulating one. **The pager (S20 option 3) is the
    reserved continuation** and moves the overlay off the resident side entirely;
    this slice deliberately builds none of it. The
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

- **the record's provenance became REAL rather than nominal (2026-07-26,
  journal/0112).** S-9's sparse-facts half has had a `DepUnit::species` axis since
  journal/0110, and the honest reading of that slice is that the axis existed and
  said almost nothing: **0.0259 m of a 440,578 m archive** disagreed with what its
  own environment implied, because the only mover carrying identity was the fluvial
  one and it moves 0.109 % of the sediment. Giving the **gravity** member the same
  identity took that to **275,626.9 m of 422,703 m — 65.206 %**. The shape did not
  change; the fraction of the world it describes went from six millionths of a
  percent to two thirds. Recorded here because it is the cleanest instance of a
  distinction this file keeps needing: **"the mechanism is right" and "the mechanism
  has authority over the world" are different claims, and only the second one is a
  measurement.** The price is `stubs.md` #25 (the record knows *what* arrived and not
  *who brought it*) and +18.16 MiB of merge key — paid in units, not in `DepUnit`
  width, because the axis finally varies.
- **live violation (the law's motivating counterexample):** the far-field LOD
  reconstructs a coarse box's *material identity* differently on cold-synthesize
  vs warm-reduce (ROADMAP Observed, 2026-07-23) — two derivations of the same base
  **disagree**, exactly what the consistency law forbids. A violation to fix
  (under Crux 1's far-span migration), not an instance done right.

---

## S-10. The frozen-snapshot, doubly-limited gather — *RATIFIED 2026-07-29 (user)*

**A field relaxes toward its neighbours by reading a frozen snapshot of the previous state,
computing an antisymmetric per-edge flux, and limiting that flux twice — once by what the donor
actually has, once by what the exchange would overshoot.** Mass-exactness and
order-independence fall out of the construction rather than being tested for: the snapshot makes
the gather order-free, and the antisymmetric edge flux makes every unit that leaves one cell
arrive in exactly one other.

**Instances (2):**
- `dc-worldgen/src/water/sat.rs` — bound water. Permeability-limited lateral redistribution
  on head `H = y + sat`; its own header (`water/sat.rs:9-14`) claims mass-exact and
  order-independent *by construction*, and that claim is the shape stated in the code before it
  was named here. **⚠ Path corrected 2026-07-29 (spine-audit at `96ab14b`) — this entry read
  `deeptime/sat.rs`, which does not exist — and the correction carries a fact worth having:
  the real path is inside the S11 water module, which § 3 lists as *"not on the production
  path"*. This exemplar is UNCONSUMED machinery.** Its `lateral_c = 0.25` (`water/sat.rs:82`,
  read at `96ab14b`) is the "safe by parameter choice" the section below argues from — safe in
  a module no world runs.
- `dc-worldgen/src/deeptime/erosion.rs::diffuse` (`erosion.rs:3175` at `72fbe86`; was `:3148` at
  `96ab14b`) — hillslope
  creep (journal/0122). The inner step
  **already was** this shape, which is why mass-exactness and order-independence came free when
  the operator was rebuilt.

**⚠ STABILITY IS THE KERNEL'S OWN BUSINESS — REVISED 2026-07-29 (user), superseding the same
day's first ruling.** The first cut placed substepping with **RATE** (*"the sub-cycle part
belongs with RATE"*). The user then re-opened it rather than let RATE grow: *"I really hate to
make RATE more complex now. Couldn't substepping be solved within the field instead, where it
takes `dt` from outside and calcs its own internal multiplier in addition to that to stay within
bounds?"* **Yes — and that is the better placement.**
**The kernel takes `dt` from outside and sub-divides internally to stay inside its own bound.**
RATE stays exactly as ratified (authored cadence + a real `dt`); it is **not** expanded.

**✅ The outside half exists now** (journal/0123): `Erosion::diffuse` reads
`let rate = cfg.diffusion * dt;` — authored, from the runner — four lines above
`let diff_sub = rate / f64::from(n_sub);` — derived, its own. **The extraction target for this
spine is therefore a concrete pattern in a named function, not a sketch**, and the two divisions
are already in the order and the ownership the ruling above requires.

**Why the kernel and not the clock — ownership of knowledge.** Four inputs set the threshold and
they have four different owners: the **stencil** (4- vs 8-neighbour → a different constant) is
the *kernel's*; **`dx`** and **`dt`** are the *engine's*; the **coefficient field** is *content*.
Only the kernel knows the constant, because it is a property of the discretisation the kernel
implements. Put it anywhere else and every plugin author inherits a von Neumann analysis as a
prerequisite for writing a diffusion pass — which is exactly the plugin-agnosticism the north
star forbids trading away. Housed in the kernel, the unsafe call is **inexpressible**: the same
move as `CoarseField` making the raw per-cell read unsayable.

**⚠ AND THE BOUND IS NOT ABOUT HAVING ENOUGH MATERIAL.** Three limits live in this shape and
conflating them is what hid the defect: **inventory** (don't ship more than you hold — physical,
prevents negative mass) · **overshoot** (don't ship more than would level the pair — physical,
kills the period-2 flip) · **stability** (`a ≤ 1/8` — **numerical**, a property of the
discretisation that would exist if every cell held infinite material). *The inventory limiter
MASKS a stability violation*: the coefficient makes the grid mode flip sign, the limiter caps
the flip at the cell's inventory, and a blow-up becomes a **finite limit cycle**. A bounded wrong
answer reads as a stable one — and that is also why refining `dt` did nothing.

**So: implementing S-10 does NOT by itself make a pass stable — the kernel must carry its own
bound.** The two instances prove it from opposite sides: `sat.rs` is stable only because
`lateral_c = 0.25` happens to sit inside its bound **by parameter choice**, and `erosion.rs` was
**2.1× past that bound on the shipped world** — same shape, one accidentally safe, one silently
not, for weeks, with every golden green. Stand-in: `stubs.md` § 30, heir **this primitive**.

**Two properties the first cut must have, and one it must not.**
- **Report the sub-step count.** Calibrated, `n = 100`. **The cost is gen-time, which is free by
  doctrine** — *"gen time is not a constraint; ready-made worlds are the sanctioned answer"* —
  so this is a **diagnostic**, not a perf guard, and must not be argued as one. *(An earlier
  draft of this reasoning invoked "runtime is sacred", which governs the **gameplay** clock and
  has no authority over deeptime. User-corrected 2026-07-29 before it reached a doc.)*
- **`n` is set by the max cell over the whole grid** — one hot coefficient sub-divides
  everywhere, which is why the calibrated cost is 6.9× rather than proportional to the affected
  area.
- **Do NOT make substepping locally adaptive in the first cut.** It is the obvious optimisation
  and it **breaks order-independence** unless sub-domains are handled with care — and
  order-independence is half of what makes this a spine.

**Why it is a spine and not a utility.** It is the concrete shape behind
`material-behavior.md` § 6's ratified *"bounded local relaxation attenuated by a per-material
property"* — permeability for water, erodibility for creep. `flow.md` reports head, saturation
and temperature as **one shape, proven by the geotherm**; this names the *computational* form
those three share. It is therefore the leading candidate for the **first engine field-solver
primitive** under the north star's (c) split, and the reason a third bespoke instance should not
be written. *Light is correctly **out**: its relaxation is max-plus, not diffusive, and it is
ruled derived-never-stored (`light.md`, narrowed 2026-07-29).*

**What a future instance must not do:** re-derive the limiter pair. The overshoot limit is what
kills the period-2 mode; the inventory limit alone produces a conveyor (`stubs.md` § 27) and a
finite limit cycle that is **deaf to `dt`** and therefore invisible to timestep refinement —
the defect that cost journal/0114 through 0116 and corrections #61, #62, #63, #72.

# 2. The anti-shapes

## A-1. A stand-in becomes the definition

Four shipped instances found in one audit: `surface_sample`'s branch,
`Litho::reference_material`, `fill.rs::is_loose`, `bio_resist`.

**Check:** S-3's disappearing-consumer question, asked at write time.

**Why the stub inventory misses it:** a stub *looks* like a fake — it says
placeholder, it names an heir. A leaked requirement **looks like working code
that passes tests**.

- **blast radius widened, not a new instance (2026-07-25, journal/0098):**
  `Litho::reference_material` — the six named rocks standing for every material in
  the world, an A-1 instance since the first audit — **gained a whole new dependent**.
  `head.rs::permeability_of` (`head.rs:214-216`) routes the head field's entire
  hydraulic model through it, and the head field's own docs correctly cite S-2 for
  doing so (*"Derived, never a second table … the same sheet
  `resistance_of_material` reads for erodibility"*, `head.rs:210-212`). Both readings
  are true and they point opposite ways: **not writing a second permeability table
  was right**, and it means confinement, transmissivity and every artesian column in
  the world now rest on a six-rock stand-in. Recorded so the day
  `reference_material` is retired, hydrology is on the list of what moves — the
  compliant choice deepened the dependency, which is the normal and easily-missed
  price of S-2.
- guarded (2026-07-25, journal/0101): *"empty"* had become the definition of
  *"unrecorded"* at the query surface — `VoxelContents::EMPTY` was the only value
  available for both, so the stand-in **was** the definition. `Identity::Unrecorded`
  is now a distinct **value in the type**, not a convention about how to read a
  `bool`. **And the overcorrection is the same anti-shape mirrored:** making air
  `Unrecorded` too (defensible — the generator writes no record for air either) would
  let *"unrecorded"* stand in for *"empty"*. Air is a positive, complete composition
  statement derivable from the block alone, so it answers `Mixture(EMPTY)`. When
  splitting a conflated value, check **both** directions before shipping the fix.

## A-2. A justification outlives its premise

Charcoal excluded because no bed survived whole-voxel quantization — true until
partial voxels. `reference_material` fixed so packs could not move terrain —
true until the content-set freeze, **the same day**.

**Check:** ~~§ 5's convention.~~ **⚠ § 5's convention FINDS NOTHING — do not route a sweep
through it** *(pointer added 2026-07-29, baseline sweep S1/#7; § 5's own drift note has said
so since 2026-07-24, ~450 lines below this line, and the flag it raised was never
discharged)*. The `JUSTIFIED-BY` marker has **3 occurrences corpus-wide and 0 in `crates/`**,
measured 2026-07-28 (`.claude/skills/doc-topology/SKILL.md`), and *"every A-2 caught so far
was caught by **reading**, not grepping."* **The real check is reading the justification and
asking whether its premise still holds.** Note that decisions expire premises *elsewhere*,
so a sweep must ask "does the cited constraint still hold?"

- **instance (2026-07-24, caught by the post-M3 sweep):
  `BEDROCK_SEAM_THICKNESS_M`'s justification expired the day Movement 3 merged.**
  **✅ APPLIED — verified 2026-07-29 at `96ab14b`: the proposed wording is in the tree, at
  `inventory.rs:1278-1290`** (the constant moved from `:425` to `:1290`), and it goes further
  than proposed — it keeps an explicit `**A-2 correction (spine-audit 2026-07-24)**` paragraph
  naming the retired claim, which is the shape every A-2 fix should take. *Preserved below as
  the finding; the "still reads" is a statement about 2026-07-24, not about today.*
  `inventory.rs:425-429` then read *"a made-up depth chosen only to be an
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
- **instance (2026-07-25, caught by the post-FLOW sweep): a measurement instrument
  that prints a caption contradicting its own number. ✅ FIXED — verified 2026-07-29 at
  `96ab14b`:** `flux_record_probe.rs:111` now prints `"<- filled by the head field"`, and
  `:104-109` carries a comment recording *why* the caption was wrong (*"the named … probe was
  printing a non-zero count beside the word EMPTY. Nothing failed"*). The proposed wording
  shipped essentially verbatim.
  `dc-worldgen/examples/flux_record_probe.rs:98-101` *(as-of 2026-07-25; the vertical row
  is now the corrected `:110-115`)* printed the vertical-face count
  labelled *"structurally present, honestly EMPTY (no infiltration term in this solve;
  **heirs: the head field** + the free/bound edge)"*. That heir **landed the next day**
  — `dc:deep/head` fills those faces with 307,364 crossings (journal/0098), the flag is
  **on by default** (`DeepConfig::head_field`, `grid.rs:341`; `head_field: true` at
  `grid.rs:580`), and the probe's own `c.vertical` now prints
  non-zero directly beside the word EMPTY. Textbook A-2, with two aggravations worth
  naming: the expired premise is on an **output line**, so it does not wait for a
  reader of the source to notice — it is *published* into whatever doc quotes the
  probe; and per CLAUDE.md's 2026-07-25 rule **`cargo test` builds examples but never
  runs them**, so no gate can see it. *Proposed:* `"<- filled by the head field
  (dc:field/head, journal/0098); zero only when --no-head-field"`. The sibling three
  sites that describe the same history are **not** instances and were checked
  individually — `field.rs:564`, `flux.rs:48-53` and `grid.rs:327` (refs refreshed
  2026-07-29 at `ec9858e`) all speak in the past tense (*"which FLOW slice 1 **left** …"*,
  *"the vertical faces **were** slice 1's honest empty, and continuation (a) filled …"*),
  which is a true statement about a prior slice.
- **staleness, adjacent to A-2 (2026-07-25): a spike result doc describing a layout
  that no longer exists. ✅ APPLIED — verified 2026-07-29 at `96ab14b`:**
  `S17-deep-cell-inventory-results.md:285` carries an inline `// ^ SUPERSEDED 2026-07-25
  (journal/0100)` **inside the code block itself**, and `:294-298` the full banner. Note the
  inline marker: the body stayed immutable and the *pointer* went where the wrong shape is
  actually read — the letter of read-first item 5's "immutable body, mutable header", applied
  mid-document. Original finding preserved:
  `docs/spikes/S17-deep-cell-inventory-results.md:284,305`
  still states, present tense, *"`struct FactLedger { facts: Vec<Vec<Fact>> }`"* and
  *"The ledger is a **sidecar** `Vec<Vec<Fact>>` keyed by unit index"* — converted to
  flat + CSR by journal/0100. Its sibling `S19-flow-record-cost-results.md:152-158`
  **did** get a `> RESOLVED 2026-07-25` note, which is exactly the right treatment and
  is why the omission is visible. This matters more than an ordinary stale comment
  because CLAUDE.md read-first item 5 tells readers spike results are the numbers not
  to re-derive. *Proposed:* the same one-line `> **SUPERSEDED 2026-07-25
  (journal/0100)** — the ledger is now flat facts + a sparse `SlotRun` CSR index; the
  shape below is the one the spike built and the one 0100 measured.*
- **instance, and a variant worth naming (2026-07-25, journal/0105 — RETIRED in the
  same commit): a justification that never had a premise to outlive.**
  `collapse.rs`'s `pore_rider_share` documented its offset as *"a **low digit** of the
  voxel's own fill draw, not its high bits: `allocate_partial` consumes the high end,
  and reusing it here would correlate … into a visible pattern."* It was reusing them:
  `(u * 4096.0) as u64 & 7` is bits 8–10 of the 20-bit offset `allocate_to` consumes,
  so the pore offset was a **deterministic function** of the allocation offset —
  measured at **100.00 % predictable** over 464,521 real decisions. The variant: this
  is not a justification that expired, it is one that was **false when written and had
  no way to be checked**, because a claim about bit ranges has no gate. The A-2 sweep
  asks "does the cited constraint still hold?"; this one asks the prior question,
  **"did it ever?"**, and the answer is only available by *computing* the claim. Two
  lessons kept from the fix: the claim is now asserted (`fill.rs`
  `no_window_of_the_fill_draw_predicts_the_pore_offset` — over *every* 3-bit window, not
  the two the comment happened to name, so it cannot rot when a field widens), and the
  disjointness is **structural** (own salt, `PoreDraw` newtype) rather than documented.
  **And looking for the harm the comment named is what found the harm it did not**: one
  `u` per voxel served every band in it, so sibling riders at a front rounded in lockstep
  (`r = +0.4878`, 1.488× the variance independent roundings give). *A wrong justification
  is worth chasing even when its stated consequence turns out to be nil.*
  **The fix was not a better comment.** The user's escalation — *"every caller gets
  allocated their own band of the hash … fix it with a construction guarantee"* —
  turned it into a mechanism: `dc_sim::statistical::rng`'s `Domain`/`Draws` provider
  and the `draw_domains!` list, where a duplicate salt is a `const` assertion failure
  and a duplicate name is a duplicate type. **This is the general answer to A-2 for a
  whole class of claims**: when a justification asserts a property the language could
  enforce instead, the durable move is to make the property structural and delete the
  claim. Twenty-six hand-rolled salts across three files and three invented numbering
  prefixes (an **A-1** instance in its own right — three authors each building the
  same registry beside the others') are now one list per crate.
- ~~not-an-instance (2026-07-23, journal/0078)~~ **🔴 IT WAS AN INSTANCE — RESTORED 2026-07-28,
  corrections #67.** *(Original: the 2026-07-22 audit flagged the `exhum`/`t_crust` comment [#28]
  as an A-2 ("claims the collapse tier reads them"), but the comment already said "WILL read …
  currently consumed by nothing" (rewritten `11d43859`, before the audit) — the audit quoted it
  with "WILL" dropped. Prose that already states "consumed by nothing" is not a justification
  outliving its premise. Tightened to cite § 3 anyway; corrections #40.)*
  **The audit did not drop a `WILL`. It quoted the comment exactly as it stood the day before.**
  `11d4385` is **2026-07-21** and the audit is **2026-07-22**; the diff removes *"the collapse
  tier **reads**"* and the commit message says it *"**Fixed the lying field.rs doc-comments that
  claimed the collapse tier 'reads' these axes**."* So the A-2 **was live** until `11d4385` ended
  it, and the audit was **right about the defect and one day behind the tree**. *This row had the
  timing right — it correctly noted the rewrite came "before the audit" — and drew the wrong
  inference from it, because "before the audit" is exactly what makes the audit's quote a stale
  READ rather than a paraphrase.*
  **The check that survives, sharpened:** verify the *quotation*, not just the claim about it —
  **and verify it AS OF WHEN THE SWEEP RAN.** A quotation without a revision cannot distinguish a
  paraphrase from a stale read, and this corpus holds **808** `file:line` citations, every one of
  them implicitly *"as of some unstated commit"*.

- **instance (2026-07-26, caught while landing the erosional calibration —
  corrections #59): `COMPETENCE_SCALE`'s anchor expired the moment `k_transport`
  moved.** The constant was defended at length as *"not a tuning knob"* because it was
  derived from `energy_band`'s Low/Medium boundary — *"`0.002` is **already the world's
  stated 'energy at which sand stops moving'**"*. But transport capacity is
  `cap = k_transport · A^m · S^n`, so `0.002` was never a physical statement: it was
  `1.25 × k_transport`, and it read as one only because `k_transport` had not moved since
  the day it was written. journal/0114 multiplies it by 45. Left absolute, the boundaries
  would have relabelled **every** depositional site on the world High energy and the
  facies gradient would have been erased by the commit meant to make the world erode.
  Fixed by stating both thresholds relative to a named `REFERENCE_KT` (bit-identical at
  the historical value, since `x / x` is exactly `1.0`).

  **The variant worth naming, because § 5's check does not catch it as written.** The
  check asks *"does the cited constraint still hold?"* — and here the *citation* was
  fine and the **form** was wrong. The premise was not a decision elsewhere that
  expired; it was **an unstated assumption that another constant in the same system
  would never change**. This is journal/0111's *closed system cannot detect its own
  scale error* one level down: **a constant checked against a number that was itself
  unchecked.** *Added check:* when a threshold is defended by derivation, ask what the
  thing it is derived FROM is anchored to — and if a threshold and the quantity it
  thresholds carry different things inside them (an absolute rate vs a coefficient times
  a dimensionless index), one of them is holding a constant it does not own.

- **instance (2026-07-26, journal/0114) — the sharpest one, and it is not about a
  justification at all but about a TEST's unstated premise.** `mfd_routing`'s
  incision-clamp falsifier (*"no interior cell more than a metre below every
  neighbour"*) had been green since it was written. It is green because **the world does
  not erode**: at the shipped rates the clamp is never stressed, and the moment the
  erosional amplitude leaves 1× the assertion fails — **44 pits at 5×, 148 at 45×,
  deepest 112 m** (stubs #29). Four phases run *after* incision and can lower a cell past
  the floor it was clamped to.

  **The A-2 shape is exact, with one twist: the premise — "erosion is fast enough for
  this test to mean anything" — was never written down, and it was false the whole
  time.** § 5's check (*does the cited constraint still hold?*) cannot catch that,
  because nothing was cited. *Added check, and it generalises well past this file:*
  **for a guard on a process, ask what magnitude of that process the test actually
  exercises.** A green assertion over a quantity that is ~zero is not evidence — it is a
  brake that is quiet because the car is parked. journal/0111 said *a closed system
  cannot detect its own scale error*; this is its twin — **a system that has stopped
  cannot detect its own logic errors either, because nothing exercises them.**

- **live risk, flagged rather than fixed (2026-07-26, journal/0114).** The
  `BEDROCK_SEAM_THICKNESS_M` instance above named *"a cranked `--erosion-budget` walk"*
  as where its 50 m ceiling would bite first. `--calibrated-rates` is that crank in a
  named form: `weathering` behind it is **45×** what it is, against a measured saprolite
  draw of 6.094 m of 50 m (12 %) at the shipped rate. Production is unaffected twice over
  — `weather_inventory` is off *and* the calibration is off — but **a
  `--weather-inventory --calibrated-rates` walk should be expected to hit the seam
  ceiling**, and a number from one is not comparable to journal/0094's. Not re-measured:
  it is a two-flag path, and re-deriving the seam depth is that arc's slice.

- **instance — a THIRD variant: a justification assembled by SYMMETRY (2026-07-26,
  journal/0118, corrections #64).** The ROADMAP defended part (a) of the draw-domain
  conversion as *"converting **them** changes the key and re-rolls every world's history
  layer: polities, sites, ruins"*. The load-bearing half is true — the region-step draw
  reaches `Block::Wood` ruin posts inside `generate_chunk` behind no flag. But **"two
  draws" is false**: the agent-step draw beside it re-rolls nothing, because the pregen
  overlay is built with an empty `agent_home` (`pregen/history.rs:82`, `:84-88` — **that
  file no longer exists; journal/0121 deleted it, and these refs are frozen as-of
  2026-07-26**), so its
  loop never executes outside dc-sim's tests. And **"polities" is false**: the count is
  fixed at epoch 0 and the extents live only in a ledger nothing reads (new § 3 row).
  The variant matters because the **remedy differs**. A-2 proper is found by re-checking
  a cited premise; the "never true" variant (the pore-rider comment above) is found by
  *computing* the claim. This one has **no premise to check and nothing to compute** — the
  sentence generalised over two call sites that *looked* alike (same address shape, same
  loop, same subsystem name) and reached for that subsystem's nouns. It is found only by
  **tracing what a shipped world executes**. The give-away was already written down and
  went unread: `dc-sim/src/statistical/world.rs:132` says *"Agents are optional — an empty
  `agent_home` gives a pressure-field-only world"*, and the caller passes `vec![]` twice on
  adjacent lines. *Sibling call sites are not evidence about each other.*
  - **DISCHARGED 2026-07-28 (journal/0121), and by removal rather than by correction.**
    The carve-out comment on `engine.rs`'s two step draws named the thing it would
    re-roll — *"every world's history layer"* — and that thing no longer exists, so the
    premise expired in the most literal available way. Both draws are now free to move
    onto the provider at zero world cost. **The tail of the same error is worth keeping:**
    corrections #64 went on to predict the removal would need `contents_contract`,
    `s7_walk` and `geology` re-baselined because they *"all hash `generate_chunk` blocks
    and are structurally downstream of the posts"*. **Not one golden moved** (corrections
    #66). Structurally downstream is a statement about the call graph; whether a
    fingerprint moves is a statement about which chunks the sampler visits, and the
    samplers visit none of the twelve that carried posts. *The same reflex — reasoning
    from the shape of the code instead of from what it executes — produced both halves,
    six lines apart, in the entry that named the reflex.*

- **NEW instance (found 2026-07-29 by the first watermarked sweep, read at `96ab14b`) — a
  completeness claim that was FALSE WHEN WRITTEN, in the file whose whole purpose is
  completeness.** `crates/dc-worldgen/src/draws.rs:23` asserts *"There is no expression anywhere
  in worldgen that turns a number into randomness"*, and its § *"The one honest hole left, and
  the residue of the other"* (`:32-60`) inventories the residue: tag space inside a domain, and
  `SALT_DT_ROUGH` surviving for `deeptime/refine.rs`'s fourth call site. **The inventory is
  short by four salts and eight call sites.** Live at `96ab14b`, all reached through
  `dc_sim::statistical::rng::draw_f64` directly:
  - `deeptime/tectonics.rs:51-53` — `SALT_TEC_POS` / `SALT_TEC_VEL` / `SALT_TEC_CRUST`, under a
    **third** numbering prefix (`0x5D00_*`), with seven call sites at `:115-121` and `:409`.
    These are plate seeding and crustal jitter — not a spike, the shipped tectonic history.
  - `deeptime/refine.rs:29` — `SALT_DT_PERTURB` (`0x5900_0002`), call site `:180`.

  **The claim is not merely stale — `draws.rs` landed 2026-07-26 (journal/0105/0118) and
  `tectonics.rs`'s three constants predate it.** So this is the *"did it ever?"* variant the
  pore-rider entry above named, and it arrived by the same mechanism as corrections #64's
  *"two draws"*: the author enumerated the sites the conversion **touched** and generalised to
  the sites that **exist**. Note where it lands — the paragraph immediately above this one, and
  A-2's own `pore_rider_share` entry, both celebrate this conversion as *"twenty-six hand-rolled
  salts across three files and three invented numbering prefixes … now one list per crate."*
  **This file was repeating the overclaim.** *Nothing is known to be wrong with the world:*
  the surviving salts are distinct from each other and from the `draw_domains!` values, so
  no stream collides — the defect is that **the property is asserted rather than enforced**
  for those four, which is exactly the state journal/0105 said it had left. ~~**Not fixed here
  (read-only sweep); the honest fixes are to convert the four call sites, or to correct the
  claim to name them.**~~ A `const` assertion covering only the converted set cannot see them.
  - **✅ THE CLAIM WAS CORRECTED — `f10dc03`, verified 2026-07-29 (evening) at `72fbe86`.** The
    second fix was taken (correct the claim, don't convert): `draws.rs:23-28` now says the
    sentence *"was **false when written**"* and names the mechanism (*"enumerated what the
    conversion touched and generalised to what exists — the corrections #64 mechanism"*), and a
    new **residue 3** (`:66-75`) carries the three `SALT_TEC_*` with their prefix, their seven
    call sites and the conversion owed. **That is the shape an A-2 fix should take** — the retired
    claim is kept beside the correction, like `BEDROCK_SEAM_THICKNESS_M`'s.
  - **🔴 AND THE FIX IS SHORT BY ONE SALT (found 2026-07-29 evening, at `72fbe86`).** The finding
    named **four** surviving salts; the correction names **three**. **`SALT_DT_PERTURB`
    (`refine.rs:29`, `0x5900_0002`, live call site `:180`) appears nowhere in `draws.rs`** —
    residue 2 accounts for its neighbour `SALT_DT_ROUGH` and stops there. Search:
    `grep -rn "SALT_" --include=*.rs crates/dc-worldgen/src`, which returns exactly
    `SALT_DT_ROUGH` (`grid.rs:34`), `SALT_DT_PERTURB` (`refine.rs:29`) and the three
    `SALT_TEC_*` (`tectonics.rs:51-53`). *And note the shape of the miss: the corrected paragraph
    enumerates **the salts the correction addressed** and generalises to the salts that exist —
    **the same mechanism it was written to retire**, one iteration later, in the sentence
    retiring it.* Reported to the integrator, not applied (source file; sweeps apply only to
    this doc as of the 2026-07-29 standing rule).
- **instance (found 2026-07-29 morning, in a file changed the day before): `Erosion::diffuse`'s
  doc comment contradicts its own journal entry, about which worlds it changes.**
  **✅ FIXED — verified 2026-07-29 (evening) at `72fbe86`: `erosion.rs:3150-3160` now opens
  *"**The shipped configuration is NOT such a world** — and the sentence that used to end this
  paragraph said it was (anti-shape A-2, caught by the 2026-07-29 spine-audit)"***, states the
  2.1× measurement, the `n = 2`, the default-on flag and *"its goldens moved with the fix"*, names
  the reachable `n = 1` fixed point (`creep_substep: false`, pinned by name in
  `tests/creep_operator.rs` against `GOLDEN_SURFACE_UNBOUNDED_CREEP`), **and installs a standing
  check** — `creep_operator.rs:119::the_shipped_world_has_cells_past_the_bound`, so the claim
  cannot silently re-expire. *That last part is the difference between a corrected comment and a
  fixed A-2: the premise now has a test, not a reader.*
  **⚠ One residue, reported not applied:** `erosion.rs:4557-4559`, the doc comment on
  `inside_the_bound_the_driver_is_the_raw_step`, still says *"The claim that **the shipped world
  is untouched** rests entirely on `x / 1.0 == x`"* — the retired claim, surviving 1,400 lines
  below its own retraction, in the test that proves the *predicate* rather than the world. The
  honest wording is *"the claim that a world inside the bound is untouched"*.
  *Original finding preserved:* `erosion.rs:3132-3134` (at `96ab14b`) read: *"`n = 1`
  reproduces the previous operator **bit for bit** … so every world whose peak effective
  diffusivity already sat inside the bound is untouched — **which is the entire shipped
  configuration**."* The trailing clause is false. journal/0122's own table (`journal/0122…:72`)
  measures the **shipped** world at peak `eff_diff` **0.261 against a 0.125 bound — 2.1× past**;
  `journal/0122:189` states it plainly: *"the shipped world takes **two** sub-steps and its
  goldens moved."* And `grid.rs:563` defaults `creep_substep: true`, so the sub-cycle is on.
  **The shipped configuration is precisely the case the sentence says is untouched.**
  - *What the sentence is probably reaching for* is the true and much better claim one line up:
    `n = 1` is bit-identical **where** the bound already held — a statement about a **predicate
    over worlds**, which the clause then collapses into a **claim about one world**, and picks
    the wrong one.
  - **Why it belongs to A-2 rather than to ordinary staleness:** the premise it rests on is
    *"our rate sits inside the bound"*, and that premise was **falsified by the very measurement
    that motivated the slice** — the same document, ~120 lines apart. It is journal/0111's
    closed-system lesson in miniature: the operator's doc comment is checking the operator
    against itself.
  - **This is also A-3-adjacent and worth saying so:** a moved golden is the loudest available
    signal, and `journal/0122:194-195` calls re-baselining *"the discipline that makes a moved
    golden an **authorized** move rather than a lost fixed point"* — while a reader of
    `erosion.rs` alone would conclude no golden could have moved.
- **not-an-instance, and the good version of the shape (2026-07-26, journal/0118).**
  Finishing journal/0105's hole 2 deleted an agreement test — `SALT_BIO_FIRE` /
  `SALT_BIO_FLOOD` had no production reader once their call sites reached `Draws::of`, so
  the test compared a constant to itself under another name and kept it alive to do so.
  That is **not** a justification expiring; it is **S-3 scaffolding retiring on success**.
  When "a summary is not an authority" is applied to a duplicate *spelling* rather than a
  cheaper *derivation*, the goal state is the duplicate's deletion and the test's with it —
  a copy that does not exist cannot drift. Worth recording so a later audit does not read
  the deletion as a lost guard. (Noted: **the assertion did not catch this — the dead-code
  lint did.**)

## A-3. A test green for a reason unrelated to what it asserts

corrections #27 (a stale artifact served as fresh: exit 0, every suite `ok`,
the code never built) and #32 (function-pointer identity folded by the
optimizer).

**Check:** verify by test **name and count**, never by `test result: ok`. Ask
*"did it run?"* separately from *"did it pass?"*

- **instance, and the purest form of it (2026-07-25, journal/0103):
  `cargo test` BUILDS examples and never RUNS them.** Every measurement instrument
  in this repo lives in `examples/` — the residency probes, the acceptance probes,
  the tour maps — and one of them (`dc-client/examples/identify_census.rs`) carried
  a literal `assert_eq!` **in `main`**, where no gate could reach it. `flow_cost_probe`
  was broken by the FLOW merge (off by the whole 42.6 MB flux record) and sat green
  through a **664-test** workspace gate. Not a lenient gate — a gate that never
  executed the code. **Closed by `[[example]] test = true`**, which builds the
  example twice: normally for `cargo run --example` (it still prints), and with the
  libtest harness so its `#[test]`s run in the gate. One file, two consumers, no
  copy to drift (which is why the obvious fixes — move it to the library, or clone
  it into `tests/` — were both wrong; the second is A-1 wearing a lab coat).
  **The generalisation:** *a thing which can fail must be run by the gate, and
  "green" is only ever a claim about the code that actually ran.*
- **instance (2026-07-25, corrections #51 / journal/0106): the guard ran on a world
  nobody ships — and the helper that built it was *called* `production_field`.**
  `tests/geotherm.rs::production_field()` built `seed 0x0B0A57EE0059, Extent::Small`;
  `dc-client` boots `BENCH_SEED = 1337` at `Extent::Medium`. **Neither the seed nor
  the extent matched.** The A-3 guard `the_geotherm_coal_shift_is_plausible_not_degenerate`
  — written precisely to stop an unverified coal *magnitude* from being believed —
  was green, correct, and about a different planet: the shipped world has **zero
  coal** (0 units against 27 134 peat; hottest candidate 15.4 °C against a 22 °C
  onset). This is A-3's **fixture** form, and it is nastier than the stale-artifact
  form because the code genuinely ran and genuinely passed. **Check:** for any test
  whose claim is a magnitude, a count, or "a player will find X", ask *which world
  did it run on, and is that the world we ship?* — and **a helper named for an
  environment must BE that environment**, because every future claim routed through
  it inherits the lie. Closed by splitting the guard: a **production** test on
  1337/Medium that asserts the rule governs and requires no coal, and a **mechanism**
  test on an explicitly-named `warm_reference_field()`. *(A non-production fixture
  is fine. A non-production fixture called production is not.)*
- **instance, and its MEDICINE, in one slice (2026-07-29, journal/0126): a hash that was green
  about the wrong thing, and an enumeration the compiler checks.**
  `surface_fingerprint` hashed `f.chapters.len()` (`tests/providers_common/mod.rs:469`) and
  nothing inside the table. **A real number about the wrong thing reads as coverage**: the chapter
  count is a config constant, so that byte pinned the config — the table could have been rebuilt
  from a different seed salt, advected at a different rate, or had every plate flipped oceanic,
  and it would not have twitched. Purest A-3: not a lenient assertion, an assertion *about
  something else*. Closed by `GOLDEN_CHAPTERS` **plus a negative control** —
  `artifact_tripwires.rs:501::the_chapter_length_byte_is_blind_to_what_the_chapter_table_says`,
  which asserts the *old* byte cannot see what the *new* one does, so the new golden's reason to
  exist is proven rather than argued.
  - **And the generalisation is structural, which is what makes it worth a spine entry.** The
    same file's claim *"every artifact the ritual ships has a tripwire"* is a **completeness**
    claim — the exact class that produced `draws.rs`'s overclaim two entries above (A-2, *"did it
    ever?"*). It is not asserted in prose: `artifact_tripwires.rs:544::
    every_deepfield_member_is_classified` **exhaustively destructures `DeepField`**, so adding a
    member **stops the suite compiling** until it is classified COVERED / UNCOVERED / NOT SHIPPED
    with its evidence. That is A-2's own prescription — *when a justification asserts a property
    the language could enforce instead, make the property structural and delete the claim* —
    applied to an audit's coverage table, and it is the same move as `EdgeId::declared` (S-8),
    `CoarseField`'s `compile_fail` doctest (S-4) and `draw_domains!`'s duplicate-salt `const`
    assertion. **A completeness claim about a set the compiler can see should never be prose.**
- **its sibling, one layer up (same entry): a printed caption is a published claim
  no gate can check.** `flux_record_probe` printed *"vertical … honestly EMPTY
  (heirs: the head field …)"* beside a **non-zero** count for a day after that heir
  landed (journal/0098). No test asserts on a `println!`. **Check:** when a slice
  fills a hole that a probe narrates, the caption is part of the diff — and this is
  A-2 in the probes rather than in the source.

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
- **the same discharge, one level up (2026-07-25, journal/0102) — and the
  divergence expires.** `DeepField::ledgers` was `Vec<FactLedger>`, a per-cell
  *owning container*: 48 B × 297,025 cells = **13.60 MiB before a fact is
  stored**, in a field where 75.8 % of cells never weather. Ported again rather
  than designed: `LedgerField` is flat facts + journal/0100's sparse slot rows +
  **`flux.rs`'s dense `cell_row_start`**. Note what happened to the carve-out
  above — at the *cell* level the dense offsets array is affordable again, because
  every cell exists even though every slot does not. So the record now carries
  **both** compressions, each where its premise holds, and the reason is stated in
  `inventory.rs` beside the struct. *A justification that names its premise
  (A-2) can be re-checked when the premise changes; this one was, one level up,
  one day later.* Measured: struct-overhead line 13.60 MiB → 0, per-cell index
  48 B → 4 B, world byte-identical (1,033,189 facts in 72,006 slots either way).
- **discharge-by-extraction (2026-07-26, journal/0112) — the third mover of a
  metre of rock, and the first not to write its own budget.** Movement 2b's
  fluvial pass split a bulk quantity into named species in **two** places
  (entrainment by the cell's `outcrop_shares` composition, incision by the
  composition below the record), each open-coding journal/0109's residual rule:
  find the last non-zero share, hand it `total − Σ earlier`. Material-aware creep
  is a **third** such split, on a different driving field, and the reflex is to
  write it inline a third time — at which point the rule that *"each species runs
  its own budget"* would live in three copies and the next mover would make four.
  It was extracted to one `split_by_shares(total, shares) -> [f64; SPECIES]`
  instead, byte-identically (the two existing call sites' arithmetic is unchanged
  term for term), and creep calls the same function. **The check the extraction
  buys is not stylistic:** journal/0110 records that the *tempting* generalisation
  of this rule leaks silently, so a fourth author re-deriving it is a fourth chance
  to re-derive the wrong one. Now no caller can invent its own budget, and the
  anti-leak unit tests (`a_composition_split_closes_to_the_bit`,
  `the_creep_split_is_linear_in_the_quantity_so_nothing_is_sorted`) sit on the one
  function all three movers route through.
- **not-an-instance, noted for the record:** `weather_epoch` open-codes
  `grid.r[i] + grid.h[i] <= sea_level` (`weather_inventory.rs:286`) where
  `DeepGrid::surf_at` (`grid.rs:718-722`) exists — but so do five other sites in
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
the `spine-audit` skill.

**A row leaves this list THREE ways, and only the first is what the index was
originally built to expect.** Getting this wrong turns the index into an argument
for its own contents — *an artifact becoming a requirement because somebody wrote
it down*, which is A-1 read backwards.

1. **CONSUMED** — something calls it. The good exit. Record what consumed it and when.
2. **DELETED** — nobody voted for it, and the honest disposal is removal
   (journal/0121: the settlement-history pass and its ruin posts). **A zero-consumer
   row is not automatically a backlog item.** Ask the standing question — *if this did
   not exist, would we build it today, in this shape?* — before the design question.
3. **HELD AS A CANDIDATE** — ratified as *wanted* but with no requirements to judge it
   against yet, so it is neither owed a consumer nor eligible for deletion (added
   2026-07-28 by the user's S2 ruling, the row below). **A row in this state must say
   so explicitly, and must name what unblocks the judgement**, because it is otherwise
   indistinguishable from state 1 waiting to happen — and a reader who assumes 1 will
   go looking for a consumer that is not wanted yet.

**⚠ IDLE AND UNGUARDED ARE INDEPENDENT AXES, and reading them as one cost this index a
wrong candidate list** (added 2026-07-29, journal/0126). The tripwire sweep that ran the day
after journal/0124 was briefed from these rows, and it inferred *exported and unread ⇒
probably also unhashed*. Half true: `geotherm`, `head` and `chapters` were indeed unhashed —
but `exhum`/`t_crust` have been inside `surface_fingerprint` the whole time, and `flux` gained
`GOLDEN_FLUX` the day before. **A row's presence here says nothing about whether a golden can
see it move.** Rows that are now tripwired say so in their `called by` cell, and the note is
worded so it cannot be misread as an exit: *a golden is a test reader, and a test reader is
not a consumer* — **an idle artifact with a golden is still idle, and the row stays.**

**And the axis runs the other way too: the worst-guarded artifact in the corpus was not in
this index at all.** `WorldGenerator::coarse_surface` — the entire far field — is heavily
*consumed*, and was fingerprinted by nothing until the same day (`GOLDEN_FAR_SURFACE`,
journal/0126). It was found by the member-#0 far slice's acceptance criterion failing to
fire: the slice changed the far class draw twice over and **no hash in the workspace moved**.
So § 3 is not a guardedness index in either direction, and nothing should be briefed from it
as though it were.

**All thirteen rows re-confirmed uncalled in production 2026-07-29 at `96ab14b`**, by the first
sweep with a recorded watermark. *(**Fourteen rows now** — the `Schedule::Seed` row was added
later the same day by journal/0124, after that sweep ran (**reference corrected 2026-07-29 by
journal/0126: `2d6ab26`'s numbering split renumbered this one to `0125` by mistake — the
`Schedule::Seed` row belongs to the Schedule slice, which KEPT `0124`; the far slice is what
moved**). Noted rather than silently
renumbered: a count stated as evidence goes stale the moment the list grows, and this doc
already carries two rows added the day the thing was built.)* **⚠ FIFTEEN ROWS as of 2026-07-29
(evening) — and the count went stale AGAIN inside the same day, exactly as the note above
predicted.** The evening FULL sweep re-confirmed all fourteen uncalled at `72fbe86` and **added
one**: the RATE authoring path (last row). *Three separate counts in one file in one day is the
argument for not stating one; left in place because each is stamped with its commit, which is what
makes the drift readable rather than merely wrong.* The searches were `grep -rn --include=*.rs crates/` for each
row's symbol with its defining file excluded; where a row's "called by" cell already names its
test/example/probe readers, that set was re-checked and is unchanged unless the cell says
otherwise. **Line references in the `where` column are refreshed to `96ab14b`** — `field.rs`,
`inventory.rs` and `contents.rs` had drifted by 200–450 lines, which for an index whose whole
job is to be *followed* is the difference between a pointer and a decoration.

| what exists | where | called by | intended consumer |
|---|---|---|---|
| `fits_in_pores` / `K_PORE` — DECIDED, built, 7 tests green | `dc-core/src/materials/packing.rs` | **no production caller** (the module now *declares* its expected consumers in-code — journal/0078 — so an infiltration author finds it, but still nothing calls it) | **hydrology** (groundwater infiltration) + **diagenesis** (cement / ore deposition) — the transport-time depositing processes |
| `recv` / `area` / `lake` — final drainage, populated in every world — **SUPERSEDED 2026-07-25 (journal/0096, FLOW slice 1)**, retirement sequenced | `deeptime/field.rs` | **still nothing in production (re-confirmed 2026-07-25** — the only reads of the exported planes are `examples/tectonic_spike.rs:170` and `tests/tectonic_history.rs:336-343`. `head.rs`/`flux.rs` read `erosion.area()`/`erosion.recv()`, which is the *in-sim* value, not the export — the same distinction `exhum`/`t_crust` carries**)**; `recv` now has one *test* consumer, the agreement check `flux_record.rs::the_receiver_export_agrees_with_the_final_chapters_faces`, which pins it as a shadow of the flux record rather than a rival authority | The intended consumers moved to the record that replaced it. `recv` is one out-edge per cell — a spanning tree that **cannot represent divergence at all** — and was "the *last* routing" after 200 discarded epochs. Its heir is `DeepField::flux` (row below). **Deletion of `recv`/`area`/`lake` and `Cell::{flow_to,river,discharge}` is continuation (e)** of the FLOW arc, not done here (this slice touched no expression). **UPGRADED FROM SUPERSEDED TO A-1 2026-07-25 (journal/0109, FLOW continuation (b)):** with MFD on, `recv` is no longer *a* routing that the record happens to replace — it is the **argmax of the partition**, a summary computed only because two consumers want one arrow per cell (the biotic valley test, the `DeepField` export). It passes ARCHITECTURE.md's test in the failing direction: *if those consumers disappeared tomorrow it would not exist in this shape.* Still not meaningless (it answers "which way does most of the water go") and still pinned as a shadow by the agreement test, now also by `mfd_routing.rs::the_receiver_projection_is_still_among_the_recorded_faces` — but **no new consumer may read it**, and (e) should now *delete* rather than *supersede*. **Re-checked 2026-07-26 (journal/0113, hybrid `p`): still no new consumer** — the spatially varying exponent reads `area` (the in-sim plane, one epoch lagged) and never `recv`, and the argmax projection is written exactly as before. **TRIPWIRED (2026-07-29, journal/0126): all three planes are inside `surface_fingerprint`, and their authority `flux` is inside `GOLDEN_FLUX` — the corpus's only DERIVED case, double-pinned. Pre-existing, not introduced by the sweep; it expires with (e). Still uncalled — a golden is a test reader, not a consumer** |
| **`DeepField::flux` — the face-flux record** (FLOW slice 1, journal/0096): per-chapter directed flux on 3D faces, the representation that supersedes the receiver tree. **Row added the day it was built, deliberately** — the slice is the *recording* half only. **Its vertical (slot↔slot) faces, honestly zero at slice 1, were filled by continuation (a) 2026-07-25** (journal/0098 — 307,364 crossings from the head field); the record itself is still unconsumed in production | `deeptime/flux.rs`; `deeptime/field.rs` (`flux`); `dc:deep/flow_record` on the runner | **no production consumer, ON PURPOSE** (re-confirmed 2026-07-25) — only `tests/flux_record.rs`, `tests/head_field.rs` and `examples/flux_record_probe.rs`. Expressing it now would mean a second fake river beside the honest absence of one, which is the exact thing flow.md § 3 forbids. **TRIPWIRED 2026-07-29 by `GOLDEN_FLUX` (journal/0124)** — on `flux_record.rs`'s own fixture, not the `providers_common` one; deliberately not re-pinned by journal/0126's sweep, because a second hash of one artifact is a second thing to re-capture rather than a second thing detected. Still uncalled | **refinement as a boundary-value problem** (continuation (b): face fluxes as Dirichlet conditions, the load budget as mass, solved *inside* a cell) · §13.8's flow-biased sub-cell fill · the paleo-channel/layering read. The record is *"the rich thing others bias to, never a second geometry model"* |
| **`DeepField::head` — the exported `head` condition-field** (`dc:field/head`, metres of hydraulic potential per cell), added 2026-07-25 by journal/0098 (FLOW continuation (a)). **Row added the day it was built, deliberately** — the same discipline the `flux` row above set | `deeptime/head.rs`; `deeptime/field.rs` (`head`); `dc:deep/head` on the runner | **no consumer of the exported plane** — only `tests/head_field.rs`, `examples/head_field_probe.rs` and `resident_bytes`. The **in-sim** field *is* consumed, and that is the point: `dc:deep/flow_record` reads it every epoch (a **declared** `reads: [Head]` edge, not a tie-break) to fill the vertical faces. Exactly the `geotherm` shape one row down, one field newer. **TRIPWIRED 2026-07-29 by `GOLDEN_HEAD` (`tests/artifact_tripwires.rs`, journal/0126) — and this row is the sweep's headline:** journal/0124's slice *moved this field's epoch-0 solve*, traced it into the flow record's vertical faces and reported the exported plane bit-identical — measured with a throwaway harness, because nothing pinned it. Still uncalled | **refinement as a boundary-value problem** (continuation (b) — head is the Dirichlet data *inside* a cell) · the **free/bound edge + void intervals** (continuation (c): a spring is where the potential meets the surface) · ~~an **MFD solve** (flow.md § 2.6 — head is what lets flux partition across several receivers, and therefore what makes *simultaneous* divergence representable)~~ **STRUCK 2026-07-25 — corrections #54.** MFD shipped that day (journal/0109) and **does not read this plane at all**: it partitions the *free*-surface potential (the priority-flood `filled` array, `z_bed + depth`), which was already computed. Using this plane would have been actively wrong — it is the **bound** regime's potential and is built to cross surface divides, so surface water routed on it would cross its own watersheds. The genuine unbuilt heir is **bound MFD** — Darcy flux partitioned across faces on `dc:field/head` — which belongs to continuation (c) with the free/bound edge · formation predicates over `dc:field/head` (§14) |
| `exhum` / `t_crust` — populated since U8; the doc comment is now **honest** (states "consumed by nothing", cites this row — journal/0078; ~~the 2026-07-22 audit's "claims the collapse tier reads them" quoted an already-corrected comment with its "WILL" dropped — corrections #40~~ **corrected 2026-07-28, corrections #67: the audit quoted the comment exactly as it stood the day before — `11d4385` (07-21) fixed what its own message calls the "lying" doc-comment, the audit ran 07-22, so the A-2 was live and the audit was one day behind the tree**) | `deeptime/field.rs` | **no consumer of the exported plane** (`t_crust` *is* read inside the sim by `isostasy()`, `erosion.rs` — the unconsumed thing is the exported plane, not the value) | metamorphic grade (stubs.md § 4) — and, since 2026-07-22, a **named socket** to arrive through: the geotherm heir of `providers::burial_temp_c` reads crustal heat flow (journal/0067). Still unconsumed; it now has an address. **ALREADY TRIPWIRED, and journal/0126's sweep was briefed otherwise:** both planes are hashed by `surface_fingerprint` (`providers_common/mod.rs`) and have been since before the flow record existed, so `GOLDEN_SURFACE` catches any move in them. The sweep's candidate list named them as *probably unguarded* purely because they sit in this index — see the § 3 header's warning that **idle and unguarded are independent axes** |
| occupancy primitives — `bound_eighths`, `is_occupancy_solid` (the two still uncalled; `free_eighths`/`open_pores` now read by the dev inspector `dc-api/src/payload.rs:367-368`, `loose_eighths` by `dc-client/src/meshing.rs:281` — 2026-07-24 sweep, **both re-verified at those exact lines 2026-07-29 at `96ab14b`**) | `dc-core/src/materials/contents.rs:286` (`bound_eighths`), `:307` (`is_occupancy_solid`) | `bound_eighths`/`is_occupancy_solid`: **no production caller — re-confirmed 2026-07-29 at `96ab14b`; the search was `grep -rn "bound_eighths\|is_occupancy_solid" --include=*.rs crates/`, whose ONLY hits outside `contents.rs` are that file's own doc-comment cross-references (`:38`, `:250`, `:252`) — zero call sites of any kind, test or production** | the four sim consumers named at authorship: water fill, loose gravity, compaction, sim light — none built yet (the inspector/meshing reads above are the dev HUD + render path, not those) |
| **Movement 2a's derived `R`/`H` views** — `DeepField::derive_regolith_at` / `derive_bedrock_at`, `WorkingInventory::derived_regolith_m` / `derived_structure_stock_m`, `surface_regolith_m` / `structure_stock_m` (journal/0092, the positional cave-excluding §13.6 rule, 4 tests green) — **row added by the 2026-07-24 post-M3 sweep** | `deeptime/field.rs:900,915`; `deeptime/inventory.rs:1622,1634,1645,1665` (**`field.rs` refreshed AGAIN 2026-07-29 evening at `72fbe86`** — cited `:818,833` that morning at `96ab14b`, moved ~82 lines by the RATE/Schedule batch; the `inventory.rs` four are unmoved. Their 2026-07-25 values `field.rs:600,615` / `inventory.rs:936,948,959,979` had drifted by ~220 and ~690 lines) | **no production caller — re-confirmed 2026-07-29 (evening) at `72fbe86`**, search `grep -rn "derive_regolith_at\|derive_bedrock_at\|derived_regolith_m\|derived_structure_stock_m" --include=*.rs crates/` — only `tests/rh_unification.rs:56,83` (the byte-identity agreement tests). Production still reads the scalar `surf`/`regolith` planes; the views exist to *prove* the inventory is the authority, not yet to *be* it | **the R/H unification** (ROADMAP "Movement 2", material-behavior.md §13.6): retire the scalar planes into the inventory so `H`/`R` derive. Until then this is the authority-half of a two-authorities split (see S-9's M3 residual), and `derive_regolith_at` reads an **empty** ledger, so it cannot see `dc:deep/weather_inventory`'s facts |
| `DeepField::geotherm` — the exported `temperature` condition-field plane (°C/m per cell), **added 2026-07-24 by journal/0093**; the field's doc already says so honestly but nothing pointed at this index — **row added by the 2026-07-24 post-M3 sweep** | `deeptime/field.rs:555` (declaration; cloned from `run.grid.geotherm` at `field.rs:648`) — **declaration unmoved, clone site refreshed 2026-07-29 evening at `72fbe86`** (was `:634` that morning at `96ab14b`, `:321`/`:382` before that) | **no consumer of the exported plane — re-confirmed 2026-07-29 (evening) at `72fbe86`** — only `tests/geotherm.rs`, `tests/flux_record.rs:326` / `tests/head_field.rs:244` (byte-identity guards), `examples/coal_walk_tour.rs`, `examples/flow_cost_probe.rs` and `resident_bytes`. The *in-sim* value **is** consumed: coal rank reads `grid.geotherm` at run finalize (`biotic.rs:765-775`). Exactly the `exhum`/`t_crust` shape, one field newer. **TRIPWIRED 2026-07-29 by `GOLDEN_GEOTHERM` (`tests/artifact_tripwires.rs`, journal/0126).** The two byte-identity guards named above assert this plane is unmoved by *their own flag*, which is a different claim from a fixed point across commits — a pass could have changed the geothermal gradient of the whole world and no test would have said so. Still uncalled | **metamorphic grade** (`exhum` = P, this = T → schist/gneiss/marble; ROADMAP "Metamorphism — now UNBLOCKED by the geotherm") and the measurement probes |
| **`DeepField::chapters` — the exported chapter table** (§ 8: plate state per chapter, `Vec<Vec<Plate>>`), populated since the tectonic-history slice. **Row added by the 2026-07-25 post-FLOW sweep — and it is the sweep's own indictment:** the field's doc comment has said *"today the table is exported and read by nothing"* (`field.rs:574-580`, **re-read verbatim 2026-07-29 at `96ab14b`**; cited as `:360-366` when the row was written) for longer than this index has existed, and three sweeps added rows for its two immediate neighbours in the same struct (`geotherm`, `exhum`/`t_crust`) without noticing it. **A self-declaring comment is not an index** — that is the whole premise of § 3, demonstrated against § 3 | `deeptime/field.rs:581` (declaration), populated from `run.chapters` — **refreshed 2026-07-29 at `96ab14b`**, was `:367`/`:421` | **no production caller — re-confirmed 2026-07-29 at `96ab14b`** — only `tests/tectonic_history.rs:339,362`, `tests/deep_config_plumbing.rs:65,87,109,126,227`, and `tests/providers_common/mod.rs:469` (which hashes only its `.len()` into the fingerprint — **citation corrected 2026-07-29 from `:169`, stale by 300 lines**; the 07-29 sweep refreshed the `where` column and not this one, which is the pointer that actually had to be *followed* to notice the length trap below). Add `tests/artifact_tripwires.rs` (the golden + the blindness falsifier). **TRIPWIRED 2026-07-29 by `GOLDEN_CHAPTERS` (`tests/artifact_tripwires.rs`, journal/0126) — and this row's `.len()` note above is exactly what nearly stopped the sweep from goldening it.** A hashed length **is not** a hashed artifact: the chapter count is a config constant, so that byte pins the config and says nothing about plate positions, velocities or continentality; the table could have been rebuilt from a different seed salt, advected at a different rate, or had every plate flipped oceanic, and the length would not have twitched. Asserted rather than argued, by `the_chapter_length_byte_is_blind_to_what_the_chapter_table_says`. **A second self-indictment for the same row, and of the same kind as the first** — a real number about the wrong thing reads as coverage. Still uncalled | **per-unit deformation re-derived analytically at collapse resolution** — dip / provenance / fault traces in cut faces, the ~5 KB that replaces stored per-cell dip vectors (ROADMAP Sequenced, the collapse-tier slice; stubs.md "Sibling gap — layer-cake strata / no dip-fold" is the same absence seen from the other side) |
| `Agent::Dissolution` + `LithoResistance.dissolution` | `deeptime/lithology.rs` | **zero call sites** in production (`examples/entry_species_probe.rs:325` is a probe; the *separate* `inventory::Cause::Dissolution` is deliberately excluded from `WEATHERING_AGENTS`, `weather_inventory.rs:61-63`) | karst — hard-gated on a carbonate that does not exist |
| the S11 water module | `dc-worldgen/src/water/` (exported `lib.rs:48`) | **not on the production path — re-confirmed 2026-07-29 at `96ab14b`**: every reader of `dc_worldgen::water::*` outside the module is a test or example (`tests/water.rs`, `tests/water_coarse.rs`, `examples/water_spike.rs`, `examples/water_coarse_spike.rs`). *The `providers/depth_to_water.rs` re-export at `providers/mod.rs:142` is a **different** module and is not evidence of consumption.* **⚠ `water/sat.rs` is one of S-10's two named instances** (2026-07-29) — the spine's exemplar lives in this row | free/bound water |
| pass-graph `Resource` vocabulary | `pipeline.rs` | 7 passes (was 8; `dc:pass/history` and its `History` axis were deleted 2026-07-28, journal/0121) | 26 of 34 inventoried seams are **value-level and invisible to it** |
| `column_summary` / `open_air_below` / `ColumnSummaries` — the S3 skylight query (S-9 deterministic-derive), built + 6 tests green | `dc-core/src/column.rs` | **no production caller** — only the `--bench-storage` timing harness (`dc-client/src/bench_storage.rs`); `docs/API.md` lists a `world.column_summary` query but **no dc-api handler exists** for it (2026-07-23 sweep) | **sim-light / skylight** — the "is this column under open sky" query; the lighting/sim-light consumer is unbuilt |
| **the whole S2 statistical tier — `engine::{query, observe, force_fact}`, `Ledger`, `ToyWorld`** (`dc-sim/src/statistical/`, minus `rng`). **Row added 2026-07-28 by journal/0121, on the day its last consumer was deleted** — the honest successor to the S7 row that departed below | `dc-sim/src/statistical/{engine,ledger,world}.rs` | **zero production callers workspace-wide.** Its one caller was `pregen/history.rs`, removed as unratified bootstrap content; everything left is `dc-sim`'s own `s2_torture` / `s2_measurements` suites. What dc-worldgen still uses from this module is the sibling `rng` (the draw provider), which is on every worldgen path and is **not** in this row | the live sim's far tiers — S2's *shapes* are ratified and instanced all over the tree (§ S-1's bounded collapse, § S-2's committed facts, § S-9's fallback query); what has no consumer is this **toy implementation**, whose own module doc calls the toy world "disposable". **✅ RULED 2026-07-28 (user) — KEEP, and this row does NOT want a consumer found for it.** *"The statistical system is genuinely intended, though I can't say whether as-is it will fit the desired shape when we actually do move on to implementing the civ/socia part of the default pack and the engine affordances."* Both the primitive and the toy stay, **as a candidate to be re-checked against requirements that do not exist yet** — not as work owed. The `Subject::{Site, Polity}` / `Aspect::{SiteExists, SitePolity, SiteEvent, PolityExtent}` / `SiteEventKind` / `Value::{Exists, PolityRef, Event, Extent}` vocabulary stays too, as **producer-less example vocabulary, NOT a schema to build on**. **Its exit condition is neither of § 3's two normal ones:** it leaves this index when the user opens the eco/socia/civ thread and the shape is judged against real requirements — gated per `worldgen.md` § *Sequencing* (all non-bio earth science in the ratified SDK-plugin shape → ecology → social), and *"much work and reflection will be done before the **USER** decides it is time."* **⚠ Do not read this row as a backlog item, and do not infer the gate is met from progress on earth science** |

| **the `CoarseField<T>` boundary API — ~~`sample_dithered`~~, ~~the `DitherSource` trait~~, ~~`CoarseField<T>` itself~~, `sample`, `summarize`.** **⚠ ADOPTION BEGAN 2026-07-29 (E5 member #0, journal/0125) — this row is now PARTIAL, and it is kept rather than deleted because two of the five are still uncalled.** Row added 2026-07-29, and it was § 3's second self-indictment after `chapters`: the **largest built-and-unconsumed mechanism in the palette-quant thread**, shipped **2026-07-22**, absent from *all three* loose-end loci — this index, `stubs.md`, and ROADMAP — for **seven days**. Read-first item 6 is explicit that an unlisted loose end is *the defect, not a licence*. **The mechanism that hid it is worth more than the row:** § S-4 recorded this as the *"Ratified end-state — **EXTRACTED**"*, and **extraction is not adoption** — a status word that names the half that shipped reads as *done* to every subsequent sweep. It took **eight days and a design pass** to get from that word to a caller | `dc-core/src/coarse.rs` (`:353` the struct, `:446` `sample`, **`:498`** `sample_dithered`, **`:534`** `summarize`, `:296` the `DitherSource` trait) — **refreshed 2026-07-29 (evening) at `72fbe86`. ⚠ This cell read *"verified at those exact lines at `96ab14b`; the adoption did not move them"* and that clause was FALSE IN THE COMMIT THAT WROTE IT** — `0dcdadb` added ~19 lines of cell-integral doc above `sample_dithered` (the `E[w_home] = 9/16` derivation, `:482-489`), moving `sample_dithered` 479→498 and `summarize` 515→534 in the same diff that asserted nothing had moved. The two unmoved refs (`:353`, `:446`, `:296`) are genuinely unmoved. *Recorded rather than silently repaired: a false claim specifically about citation accuracy, inside § 3, is this file's own subject matter — and it is exactly what `spine-audit`'s "cite the commit you read it at" rule exists to make visible* | **NOW CALLED (2026-07-29): `CoarseField<T>` + `from_cells` + `sample_dithered` + `Registration`, from `collapse.rs::surface_class` / `class_window`** — the far field's surface class is a `CoarseField<ShareVec<6>>` window of deep-cell top-voxel shares, drawn through move B. **`DitherSource` has its first production impl**, `dc-worldgen/src/draws.rs::Coherent` (wrapping `interp_corner_field`) — journal/0075's *"dc-worldgen wraps its own `draw_f64`"* stopped being a plan. **STILL UNCALLED, and this is what keeps the row alive: `sample` (move A) and `summarize`.** `summarize`'s absence is now *ruled*, not pending — **its home is the octree node contract** (user, 2026-07-29, member-#0 MM-4), so it is uncalled because its consumer is a different executor that does not exist, not because nobody got to it. `sample`'s nearest live candidate is `lithology.rs:592,614`'s anchored blend, which is the same operation written as a plain function (seam-first #6). The sibling `ShareVec<N>` was always consumed (`lithology.rs:551,565`, `providers/`) and is not in this row | **U22 (the cake law, far field) — ✅ DISCHARGED 2026-07-29** by the adoption above, with a world-scale acceptance test (`collapse.rs::far_class_dither_interfingers_across_a_real_deep_cell_frontier`) beside `coarse.rs`'s synthetic one. **U3 (the per-chunk palette checkerboard, near field) — STILL OPEN**: same root (`record_at_voxel` reads NEAREST at the ~460 m deep cell), different site, and the member-#0 design pass found the near path needs machinery that does not exist (MM-1 the separately-callable membership dither, MM-3 the working sub-cell state) plus a co-requisite octaves source. Ratified 2026-07-22 in the strongest language in the thread — *"we finish this today"*, *"this is foundational"*, end-state ***"solved by construction, approximately once."*** journal/0075 § *What was NOT done* assigned adoption to *"those owners"* and **those owners were never named**; an assistant clause then handed apparent ownership of the palette-quant fix to the genesis-passes arc (struck 2026-07-29). Now owned by **ROADMAP § Sequenced "COARSEFIELD ADOPTION"** → E5 member #0's continuation slot. Note `summarize` carries a *second* intended consumer: it is corrections #39's named honest heir for the coherent source's majority-amplification, which the adopted far path still lives with |

| **`Schedule::Seed` / `Schedule::SeedAndStep`** — two of the three variants of the SCHEDULE axis (`deeptime/schedule.rs`), **BUILT 2026-07-29 (journal/0124), row added the day they were built.** They are **executed**, not stubs: `DeepSchedule::run` opens with a setup pass over `plan(None)` at `dt = 0.0`, and two runner tests drive a synthetic roster through it. What is missing is a **declarer** | `deeptime/schedule.rs` (the enum), `deeptime/runner.rs` (`DeepSchedule::plan`, `DeepSchedule::run`) | **no production declarer — every pass in every shipped roster is `Step`**, asserted by name (`runner.rs::no_shipped_pass_seeds_and_every_one_of_them_fires_at_epoch_zero`, `tests/schedule_axis.rs::epoch_zero_fires_every_pass_in_every_roster`). Only the synthetic rosters in those two files construct them | **declared epochs** (ROADMAP pass-architecture slot (d); named as the heir in `Schedule::Seed`'s own doc comment). `Seed` is *"member of the setup epoch"* said small. **⚠ Read this row in state 3, HELD AS A CANDIDATE, not state 1** — the variants exist because the *axis* is a sum type by ratification (`ARCHITECTURE.md` § Schedule: a `seeded: bool` cannot express run-once-only), and the audit that shipped them found the three pre-loop incumbents were all first-steps in disguise. **A zero-declarer count here is the audit's FINDING, not a gap to fill**: the roster's genuine setup work — the biotic layer's identity planes, the tectonic chapter table — is not pass-shaped yet, and making it so is (d)'s job, not a search for someone to seed |

| **the RATE AUTHORING path — `CadenceTable::{with, unmatched}`, `Cadence::sub_turned`, `Schedule::integrated_dt`.** **BUILT 2026-07-29 (journal/0123 RATE + journal/0124 SCHEDULE); row added 2026-07-29 (evening) by the FULL spine-audit at `72fbe86` — five commits late, and the lateness is the finding.** The *reading* half of RATE is fully consumed: `Schedule`/`Cadence` are on every pass declaration, `deep_passes_with` applies the table on every production path, and the rate-shaped passes consume `dt`. What has no production caller is the **authoring** half — the verbs a world would use to say *"weathering ×5 while tectonics ×1"*. **Sibling of the `Schedule::Seed` row below, one axis over, and journal/0124 filed that one *the day it was built* while journal/0123 filed none for its own** | `deeptime/cadence.rs` (`:206` the `impl`, `:251` `unmatched`, `:120` `Cadence::sub_turned`), `deeptime/schedule.rs:169` (`integrated_dt`); the empty-table production entry points are `field.rs:608`, `mod.rs:175`, `runner.rs:619` | **zero production callers of the authoring verbs — search `grep -rn "CadenceTable\|sub_turned\|unmatched\|integrated_dt\|with_cadence" --include=*.rs crates/` at `72fbe86`.** Every hit that *constructs a non-empty table* is a test: `tests/rate_axis.rs:105,109`, `tests/schedule_axis.rs:109-113`, `runner.rs:1728-1733,1776`, `cadence.rs:314-316`. `unmatched` has **no caller at all** outside `runner.rs:1759` (a test) and its own — and `runner.rs:633` is a doc comment naming its future one (*"there for a caller that wants to be strict"*), the § 3 signature verbatim. `Schedule::integrated_dt` likewise: only `tests/schedule_axis.rs:79,115` and `schedule.rs`'s own suite; the runner does not compute `dt` through it. **`with_cadence` IS consumed** (`runner.rs:645`) and is not in this row | **the per-world MANIFEST** — named in-code, precisely, at `mod.rs:186-190`: *"**This is the seam, not the format.** When authored ORDER lands (the pass architecture's continuation slot (b)) it brings a per-world manifest with it, and the manifest's cadence section constructs the `CadenceTable` handed here. Inventing that format now would be building the general mechanism ahead of its caller, so **the parameter exists and the loader does not**."* That is `dependency-graph.md` **E7** (authored order + the validator), the same gate S-6 names. **⚠ Read this row in state 3, HELD AS A CANDIDATE:** the absence is *ruled*, not owed — a loader without a format would be the general mechanism ahead of its caller, which is the thing the seam-first practice forbids. **And it is § 3's third self-indictment in a row, of the identical kind as `chapters` and `CoarseField`: a comment that honestly declares its own hole is not an index**, and this one is as explicit as the corpus gets — five days after the `chapters` row made exactly that argument |

**Departed (the good event):**

- **the S7 pregen history handoff — `Pregen.ledger`, `.overlay`, `.n_polities`,
  `.observe_count`** — **DELETED 2026-07-28** (journal/0121), two days after this
  index first listed it. Not consumed: *removed*, with `pregen/history.rs`,
  `Pregen.sites`, `collapse.rs::ruin_posts` and the `dc:pass/history` pass, on the
  user's 2026-07-26 direction. **This is the second way a § 3 row leaves, and the
  index had only ever recorded the first.** The header says *"record what consumed
  it and when"* — but the honest disposal of built-and-unconsumed machinery is
  sometimes that nothing ever should consume it. Reading the row as *"find this a
  consumer"* is the A-4 mirror image of A-1: instead of a stand-in becoming the
  definition, an **artifact becomes a requirement** because it was written down.
  The test is CLAUDE.md § *Existence is not standing*, and § 3 rows are exactly
  where it wants asking. *The row's own measurement is what made the deletion
  cheap: it had already established there was one non-test reader and that all it
  read was the size.*
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
both costs.
**⚠ SETTLED UNEVENLY, 2026-07-29 (E5 member #0, journal/0125) — one of the three
heirs landed, one left the tier, one is all that remains.** The **boundary
source-cell membership dither shipped**: `surface_class` now calls
`CoarseField::sample_dithered`, so the cake cost is discharged (world-scale test,
`collapse.rs::far_class_dither_interfingers_across_a_real_deep_cell_frontier`).
The **far-`summarize` register did NOT** — `summarize` was ruled to the octree
node contract the same day (user, member-#0 MM-4), so it is not a refinement-tier
heir any more. **The deviation itself therefore STANDS, unchanged and now at TWO
salts** (the membership draw reads the same coherent field, deliberately — a
white-noise membership draw speckles *which cell* each far sample reads from,
which is this row's own aliasing argument one level up). Remaining heir for the
bias: **a CDF-corrected source**, and nothing else inside this tier.
Ratified by the user in-session with the observation attached:
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

**A worked instance of the diagnostic working *forwards*, 2026-07-26
(journal/0110).** Movement 2b's material-aware transport pass needed three
identities and named none of them, and the third is the one worth recording
because the obvious answer was a constant:

- *what a flow entrains* → the composition of the cell's own near-surface window,
  through the `outcrop_shares` seam that already existed for the erodibility blend;
- *how the load sorts* → `settle_energy` over the property sheet, the same function
  the shipped placer pass already thresholds on;
- *what incision detaches* → the first draft wrote `Litho::Basement`. Applying the
  diagnostic — *what property of that content am I reaching for?* — the answer is
  **"whatever lies below the record"**, and the composition seam already answers
  exactly that question when handed an **empty section**: a window containing no
  units is entirely the material beneath the pile. So the pass asks
  `outcrop_shares(&[])` and the constant disappears. The day basement stops being
  uniform, the seam's heir answers and the pass does not change.

The one place the roster is still named is `Litho::as_deposited` — *basement is the
one lithology a deposit cannot be* — which lives in the lithology **adapter**, not
in a pass, and is a statement about the record's own vocabulary rather than about
any pack's content.

---

# 6. The audits (`docs/audits/`)

Standing inventories, produced by read-only sweeps. Like `docs/spikes/*`, they
hold **measured numbers and file:line citations** — consult before re-deriving.

- **`2026-07-22-seam-inventory.md`** — all **34 seams** *(as inventoried on the day;* **31
  are live today** *— that audit's own 2026-07-28 banner; corrected here 2026-07-29. Every
  ratio below is over the audit's 34 and is left as the audit stated it)*: what each stands in
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

**⚠ THE FOUR ENTRIES ABOVE ARE THE 2026-07-22 COHORT AND THIS INDEX FELL ELEVEN BEHIND ITS
OWN RULE.** Caught 2026-07-29 (baseline sweep S1/#11) — *an index that states its own
maintenance rule and does not follow it is self-refuting in exactly the A-4 sense the section
below names.* Backfilled by listing, **each with a one-line role only** (their own headers are
the authority; several now carry supersession banners of their own — read the banner first):

- **`A1-collapse-slice-plan.md`** — the block↔material collapse plan. ✅ executed 2026-07-23
  (journal/0087); historical, not a to-do list.
- **`2026-07-23-block-consumer-inventory.md`** — the pre-collapse `Block` consumer census
  (135 sites / 31 files). ⚠ every count and address is pre-collapse.
- **`2026-07-23-perf-baseline-vertical-drop.md`** — the vertical-drop perf baseline.
- **`2026-07-22-entry-species-probe.md`** — the entry-species probe.
- **`2026-07-24-lod-pre-post-visit-diagnosis.md`** · **`2026-07-24-palette-quant-generation-diagnosis.md`**
  — two rendering/LOD diagnoses.
- **`2026-07-25-contents-empty-over-solid-diagnosis.md`** — the `has_contents` instrument
  diagnosis. ✅ fixed the same day (journal/0101, corrections #49); banner on the file.
- **`2026-07-24-roadmap-staleness-sweep.md`** · **`2026-07-25-roadmap-staleness-sweep.md`**
  — dated board snapshots; the 2026-07-25 one is named as a snapshot by corrections #55.
- **`2026-07-26-doc-topology-sweep.md`** — the first docs-vs-docs sweep (19 findings).
- **`2026-07-28-corrections-recoding.md`** — the adversarial re-coding of `corrections.md`
  (the measurement behind *staleness is 3–8 %; most failures were wrong when written*).
- **`baseline-2026-07-28/S1–S9`** — the nine-slice baseline `doc-topology` sweep, ~3,900
  lines. **Findings carry an application state** (`✅ APPLIED` / `⚠ ESCALATED` /
  `❌ FINDING WRONG`) as of 2026-07-29; read the state before acting on a finding.

**⚠ AND IT FELL FIVE BEHIND AGAIN WITHIN A DAY OF THE BACKFILL** — caught 2026-07-29 (evening) by
the FULL spine-audit. *The backfill above was the fix for being eleven behind; four audits written
the same day and one written that evening were unlisted when it landed. **A backfill is not a
watcher** — the rule needs the mechanism it asks for, and § 5's dead `JUSTIFIED-BY` marker is the
standing evidence that a convention asking an author to remember something dies. The candidate
mechanism is the one A-3 above just praised: the enumeration a compiler (or a sweep with a
directory listing) checks, not a list a human appends to.* Listed, one line each:

- **`2026-07-29-roadmap-classification.md`** — the 59 verified archive ops + the 5 for the user.
- **`2026-07-29-refinement-coupling-priors.md`** — the coupling-priors research behind
  `refinement.md` (PROPOSED → cautiously ratified with the members qualifier).
- **`2026-07-29-member0-coarsefield-design.md`** — E5 member #0's design pass (MM-1…MM-4; MM-4 is
  the ruling that sent `summarize` to the octree node contract, cited by § 3's `CoarseField` row).
- **`2026-07-29-far-frontier-tourmap.md`** — the tour map for the member-#0 far walk.
- **`2026-07-29-spine-audit-full.md`** — this sweep. The **first `spine-audit` artifact under
  `docs/audits/`**: every prior run wrote only the header block above, which is why the baseline
  note recorded *"spine-audit had left no artifacts at all."*

**These were nearly lost.** They lived in a session scratchpad and were cited
all day; nothing in the corpus pointed at them. That is **A-4 committed on the
day A-4 was written** — proof that an index only helps if the artifacts it
indexes are *in the repo*. Any future sweep lands here, in the same commit as
the work that used it. *(And the rule needs a watcher: it was stated here on 2026-07-22 and
was eleven entries behind it six days later.)*
