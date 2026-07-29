# The deep-time pass declarations: how they got this way

> **⚠ BANNER, stamped 2026-07-29 by journal/0124 (the `Schedule` slice), same commit — the
> obligation is the correction-writer's, CLAUDE.md read-first item 5.** Two claims below about
> what is *live in the code* have been superseded; the **body is not rewritten**, per this
> file's own header.
>
> 1. **`DeepPass::fires` and its "ratification flag" no longer exist.** The paragraph under
>    *"journal/0123's RATE commentary is not history"* lists `the `fires` ratification flag`
>    among the live obligations left in `runner.rs`. That flag was ratified (`ARCHITECTURE.md`
>    § *Schedule*) and built the same day: the `epoch > 0` skip rule is **deleted**, firing is
>    `epoch % period == 0` and nothing else, and `DeepPass::fires` itself was removed once
>    `DeepSchedule::run` began driving itself through `DeepSchedule::plan`.
> 2. **`DeepPass::cadence` is now `DeepPass::schedule`**, a `Schedule` sum type carrying the
>    `Cadence`. RATE is unchanged; a third axis sits above it.
>
> Nothing in § 1–§ 4 below is affected — the archaeology is about `reads`/`reads_prev`/the
> revision chain, which this slice did not touch.

**Extracted testimony, 2026-07-29.** This file is the **cold half** of
`crates/dc-worldgen/src/deeptime/runner.rs` — the narrative of how that file's pass
declarations reached their present shape. It was carved out under the file-size doctrine
(`scripts/filesize_hook.py`: *the split axis is LIVENESS, never topic*), because the
archaeology is still true, still worth reading once, and **no longer read to do today's
work**, while the contracts it grew out of are read every time someone adds a pass.

**Measured, so the next reader is not misled about what this bought.** `runner.rs` was
**1,694 lines** (2.4× the 700-line source threshold), of which **686 were comment**. The
extraction removed **128** comment lines and rewrote **88**, landing the file at **1,654**
(comment 686 → 646). That is the honest size of the cold half: the remaining comment volume
is *contract at its declaration site*, and the file is **still 2.4× over threshold because
927 of its lines are code**. **Comment extraction cannot fix this file's size** — the remedy
left on the table is an ordinary module split (the `DeepAxis` vocabulary, the pass bodies,
the schedule), which is out of scope for a comments-only slice and is not sequenced.

*journal/0123's RATE commentary is **not** history and was left whole* — the cadence/`dt`
contract on `DeepStepCtx::dt`, `DeepPass::cadence`, `deep_passes_with`, `DeepSchedule::run`
and the `fires` ratification flag all state live obligations at their declaration sites.

**The split rule that produced this file, stated so the next extraction can repeat it:**

- **Stays in `runner.rs`** — anything that states a **live contract at its declaration
  site**: why a read is or is not declared, what a revision token means, what an axis is
  for. Contract text lives beside the thing it governs, and `docs/spines.md` § S-6 quotes
  several of those blocks verbatim.
- **Moved here** — how the declarations *got* this way: the defect walkthroughs, the
  superseded defences, the "it used to say X" clauses, the slice-neutrality arguments that
  a test now proves.

**As-of, and what that means.** Everything below is **as of `3ee8b0e`** (the commit this
extraction was cut from — after the E3/RATE merge), and this file is **dated testimony, not a live pointer** — the same
discipline `docs/spikes/` and `docs/audits/` run under (CLAUDE.md read-first item 5:
*immutable body, mutable header*). It deliberately carries **no `file:line` references**:
every symbol it names is greppable, and a stale line number in a cold file is the exact
failure mode `docs/spines.md` spent a slice repairing. If something below is later refuted,
the refuting author stamps a banner at the top of this file in the same commit.

**Nothing here is a claim about what the code does today.** For that, read the
declarations and their comments in `runner.rs`.

---

## 1. journal/0090 — the loop re-housed as declared passes

The deep-time epoch loop was hand-written phase order inside `Erosion::step` and
`run_cells`. Movement 1 of the deep-time-loop → declared-passes conversion turned each
phase into a `DeepPass` that declares `{reads, writes}` over a deep-cell axis vocabulary
plus a cadence, and let the runner topo-sort them.

The conversion was **byte-identical by construction**: each pass body is *literally the
same call* the old loop made, in the same order the topo-sort reproduces, with the same
arguments, so the production world hashes to the same goldens. `Erosion::step` itself was
retained rather than deleted — profiling harnesses and the erodibility/deeptime tests still
call it — and the pass bodies drive the same public phase methods it composes.

**On the RATE axis it also made a claim that has since been superseded, in place.** Movement 1
*pinned* rate — every pass kept its effective cadence and `dt` was threaded but **inert**, no
transform scaling by it — which is what made the re-housing byte-identical. journal/0123
replaced that: cadence is authored data and `dt` is a real clock. The module header carried a
parenthetical saying which sentence had replaced which; the replacement is the only part still
worth reading, and it is in the code.

That "it reproduces the old order" framing is the thing journal/0119 later re-read: the
revision tokens are the **artifact of derivation**, the hand-declared canonical order
re-encoded so the graph appears to compute it. See `docs/spines.md` § S-6, which now teaches
**author-and-validate**; authored order is `docs/dependency-graph.md` **E7**.

## 2. journal/0104 — `reads_prev` was documentation until it was a mechanism

`DeepPass::reads_prev` was declared, typed and documented — and **handed to nothing**. It
appeared in `runner.rs` and in no other file in `crates/`; `passgraph` never received it.

So which epoch a lagged reader actually observed was an accident of the
**id-lexicographic tie-break** in the kernel's ready pool. Every lagged read in the roster
happened to be true, but only because the alphabet happened to park the reader ahead of the
writer. **Rename a pass and the physics changed silently, with every test green.**

The fix gave the kernel a second edge kind: a `reads_prev` becomes a reader → writer
**anti-dependency** (WAR), the reverse direction from a `reads` (RAW). Folding a lagged read
into `reads` is not a smaller fix — it points the edge the wrong way, and for the
biology↔erosion feedback it closes a within-epoch cycle the runner correctly rejects.

The guard that keeps it honest is
`a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename`: every lagged reader in
the production roster is renamed to an id chosen to **lose** the alphabet fight against
every writer of the axis it lags on, and the schedule must still place it first. Under the
old kernel `dc:deep/zzz_head` would have slid behind `dc:deep/deposition` and started
reading **this** epoch's strata record.

## 3. spine-audit 2026-07-25 — the `BioMod` fiction on `dc:deep/weather_inventory`

The same defect class, one level down: `dc:deep/weather_inventory` declared
`reads_prev: &[BioMod]` while `weather_epoch` read `grid.bio_weather`, a plane
`dc:deep/biotic` overwrites **in place** each epoch. With no edge against `biotic`, the
tie-break decided which epoch's plane it saw. It also declared `Exposed` and never read it.

Both were fixed 2026-07-25 — `BioMod` became a real within-epoch `reads`, `Exposed` was
**removed** rather than made true. **The reasoning for both stayed in the code**, at the
`WINV_READS_*` const block, because it is a live contract on those declarations and
`spines.md` § S-6 quotes it verbatim. The full audit record is in
`docs/spines.md` § S-6 and `ROADMAP.md`'s (struck) *"Two declaration defects"* entry.

## 4. journal/0107 — the terrain plane had several revisions and one hole

### The gap at `Incised`

`dc:deep/transport` **mutated the terrain** — bedrock incision lowers `R`, entrainment and
deposition move `H` — while declaring only its `Energy`/`DeltaH` by-products. That left the
erosion pipeline's revision chain with a hole exactly where the ground surface first changes
each epoch: nothing in the vocabulary named the moment between `Forced` and `Weathered`, so a
pass reading `R + H` there could not say *when* it read, and had no writer to be ordered
against. `DeepAxis::Incised` closes the chain (`transport` writes it, `weather` reads it,
`head` lag-reads it).

### The sidecar that floated

A revision token orders a reader **after** the stage that produced it, and says nothing about
the stage that overwrites the same plane **next** — that stage writes a *different token*, a
different resource to the graph. Inside the erosion pipeline this never mattered, because
every stage is braced on its far side by a forward edge into the stages after it.

A pure **sidecar** field pass has no such brace. `dc:field/head` read the ground surface
`R + H` and handed its field to nothing the terrain consumes, so which revision it saw was
decided by the id-lexicographic tie-break — **the third instance of a tie-break deciding
physics.**

Its declaration named only `reads: [Routed]`, covering `filled` / `routed_surface` / `area`
and **not** the ground surface the body builds through `grid.surf_at` and the solve uses as
its seepage cap, its lake datum and its whole free-surface boundary.

The old defence — *"its position never affects the terrain"* — was **true, and was answering
the wrong question**. It affects the head field's **own values**, and therefore the vertical
flux the flow record keeps from them.

### The prescription that was half wrong

The 2026-07-25 audit's own prescription, *"declaring `Forced` is free and it pins it"*, was
**half wrong**, and the ROADMAP's *"verify that claim before trusting it"* is what caught it.
Free: yes. Pins it: **no** — a `reads` edge pins one side only.

**The fix is the pair**, and that generalisation is what stayed in the code as a contract:
declare the revision you consume as a `reads`, and the **next** revision of the same plane as
a `reads_prev`. The next, never the last, or you leave yourself free to slide past every
writer before it.

### The two other terrain lags

`dc:deep/climate` marches on the **start-of-epoch** topography — `climate::march` reads
`grid.surf_at` — and used to say nothing about it. It now lag-reads `Forced`. Lagging against
the *last* revision instead (`Settled`/`Compensated`/`Diffused`, one cfg-selected slice each)
would have been three declarations that pin strictly less: they would let climate slide past
`forcing` and `transport`.

`dc:deep/weather` reads the incised terrain and now says `Incised`; the ordering that adds
(`transport → weather`) was already carried by the `DeltaH` chain.

### Neutrality, and why it is not argued here any more

The slice added four declarations — `climate.reads_prev = [Forced]`, `transport.writes +=
Incised`, `weather.reads += Incised`, and head's `Forced`/`Incised` pair. Every edge they
introduce was already implied by the existing transitive closure, so the schedule could not
move; `climate → forcing`, for instance, was already a true forward edge because `forcing`
reads `Climate`. **It was safe before; it was *stated* then.**

That argument is no longer carried in prose because a test carries it:
`the_terrain_revision_declarations_are_schedule_neutral` rebuilds five rosters with the
**pre-slice** declarations and asserts the order is identical — the direct check, rather than
an appeal to a golden hash computed elsewhere. The vertical-flux record was unmoved: 307,364
entries, 44.301 % of cells, 60 artesian.

---

## Related, and still live

- `docs/spines.md` § S-6 — the order-is-data spine, the declaration-honesty audits, and the
  anti-dependency entry. **Live.**
- `docs/dependency-graph.md` — **E6** (retire `DeepAxis` so packs declare their own resource
  ids) and **E7** (authored order + the validator).
- `journal/0090`, `journal/0104`, `journal/0107`, `journal/0119` — the entries themselves,
  which are narrative and are never revised.
