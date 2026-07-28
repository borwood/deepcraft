# S6 — `ROADMAP.md` § Observed: the first complete read

**Baseline sweep, slice 6 of 9.** Watermark commit `f652b60`; every `file:line` and every
source quotation below was read at that commit.

**Scope read IN FULL:** `ROADMAP.md:2882`–`5163` (2,281 lines) — § Observed plus the three
close blocks at the tail. **129 top-level entries**, matching the brief's count exactly
(`awk 'NR>=2882 && NR<=4880 && /^- /' ROADMAP.md | wc -l` → 129).

**What I did NOT read:** everything above `ROADMAP.md:2882` (another agent's slice);
`ROADMAP-history.md` in full (grepped and spot-read only); individual journal entries other
than by targeted grep; `docs/design/*.md` bodies other than `stubs.md` § 1 and `spines.md`
§ 3 rows.

**Method note.** Where I claim an entry is stale I verified **at source in `crates/`**, not
from a journal's claim about itself. Eight verdicts below rest on a source read that
contradicts the entry's own text. Where I could not get to source I said `CANNOT-DETERMINE`
rather than reasoning my way to a verdict — per the brief, a wrong `LIKELY-RESOLVED` on a
user field report is the worst outcome available here.

---

## 1. Counts

| Status | Count |
|---|---|
| `STILL-OPEN` | 69 |
| `LIKELY-RESOLVED (evidence)` | 18 |
| `RESOLVED-IN-PLACE — entry is its own closure notice` | 37 |
| `CANNOT-DETERMINE` | 4 |
| `SUPERSEDED-BY-RECALIBRATION` | 1 |
| **Total** | **129** |

`RESOLVED-IN-PLACE` is a category the section itself created and the brief's four buckets do
not have: an entry whose *body* already says ✅ DONE / RESOLVED / FIXED, kept deliberately
"for the record", never moved to `ROADMAP-history.md`. These are not findings — they are
**archive candidates**, and they are **29 %** of the section. I separated them from
`LIKELY-RESOLVED` so the integrator does not have to re-read them to tell the two apart. The
18 `LIKELY-RESOLVED` are the real yield: entries that still *read* as live work.

**Only 1 entry is superseded by journal/0111's ~1000× recalibration**, and that surprised me.
The section's magnitude claims are overwhelmingly about *residency*, *render* and *content
rosters* — none of which the erosional recalibration touches. The pre-0111 landform entries
("dismal mountains", the 5–460 m band) are **not** artifacts of the old scale, because the
calibration was **built, measured and switched OFF** (`ROADMAP.md:5072`: *"The joint
calibration (0114) — BUILT, MEASURED, AND OFF"*; `:5118`: *"Until this lands the engine
cannot run erosion at any realistic rate"*). The shipped world still runs at the old rate.
**A recalibration that is not enabled cannot supersede an observation.** That is the most
important structural fact about this section's staleness, and it cuts the opposite way from
the brief's hypothesis.

---

## 2. TOP 5 resolved-but-open

Ranked by cost of a cold reader believing them.

### 1. `ROADMAP.md:4530` — "The embedded HostWorld never evicts chunks"
> *"The embedded HostWorld never evicts chunks (~64 KiB per chunk ever streamed/edited);
> never-edited chunks are pure generator output and could be dropped freely (journal/0002)."*

**It evicts.** `crates/dc-api/src/host.rs:524` is
`fn enforce_chunk_budget(&mut self, protect: Option<ChunkPos>)`, a bounded LRU with
hysteresis (`:169-170`, `:530-531`), called from `set_chunk_budget` (`:504`) and from the
materialize path (`:562`); `:544` is the `self.chunks.remove(&pos)`. `:246-250` documents it
as a *"bounded LRU … edited entries are"* protected — **exactly the
generated-untouched-vs-edited split the entry asks for.**

**And the refutation is 729 lines above it, in this same section.** `ROADMAP.md:3801`:
*"FIXED 2026-07-21 (journal/0051): eviction landed, the march is flat (+27.4 → 0.00
MB/jump)."* This is the corrections-#65 shape — a claim and its own refutation coexisting in
one artifact — occurring **inside the artifact read at the start of every session.**

### 2. `ROADMAP.md:4790` — "The `history.rs` reject-don't-crash skip is SILENT"
> *"A skipped world-history collapse currently emits nothing; it owes a named warning. Small."*

`history.rs` does not exist. `ls crates/dc-worldgen/src/pregen/history.rs` → *No such file or
directory*; `grep -rn "RegionId" --include=*.rs crates/dc-worldgen/` → **zero hits**. The
bootstrap history content was removed 2026-07-28 (journal/0121, −713 Rust lines;
`ROADMAP.md:4895`). The entry files an owed warning on a code path that no longer exists.

### 3. `ROADMAP.md:3865` — "ruin-posts was UNDOCUMENTED … a loud code comment is owed at `collapse.rs::ruin_posts`"
`ruin_posts` is deleted. `docs/design/stubs.md:30`: *"### 1. ruin-posts — **RESOLVED BY
DELETION 2026-07-28 (journal/0121); no heir was ever built and none is owed**"*, and
`stubs.md:53` names `collapse.rs::ruin_posts` (~997 lines) among what went. The only
surviving `ruin` tokens in `collapse.rs` are a module-doc mention (`:13`) and a comment
reading *"there are no ruin…"* (`:2236`). **The entry's one "genuine discovery" is an owed
comment on a deleted function.**

### 4. `ROADMAP.md:4548` — "Site cap 240 (u8 `RegionId`) — concrete instance of S2's ledger-scale question (S7)"
Same deletion. `grep -rn "RegionId" --include=*.rs crates/dc-worldgen/` → **zero**; `sites` is
absent from `crates/dc-worldgen/src/pregen/mod.rs`. A constraint on content that no longer
exists, still filed as a live scale question. *(The `RegionId` hits that do remain are in
`crates/dc-sim/src/statistical/engine.rs:42` — an unrelated toy-world type.)*

### 5. `ROADMAP.md:4601` — "We cannot see where runtime goes"
> *"the project … has **NO** runtime frame/tick observability: no span-level profiling, no
> tick-time breakdown"*

`crates/dc-client/src/perf.rs` exists. `:26` documents it — *"Under `--features perf` it
expands to a real `bevy::log::info_span!(..)"* — `:40` is the expansion, and `:68-69` pulls in
`bevy::log::tracing::{Subscriber, span::Id}` for a custom collector. The entry's own successor
sits 1,072 lines **above** it at `ROADMAP.md:3529` — *"The perf instrument can't show the
frame-thread envelope … `PerfAggregate` sums self-time across the frame thread AND the
task-pool threads"* — which is a complaint **about the instrument this entry says does not
exist.** The live gap is per-thread-role attribution, not the absence of profiling.

---

## 3. The finding that is NOT a staleness — a live defect the section's own text hides

**`ROADMAP.md:3133` — "A PROBE THAT CAN FAIL IS INVISIBLE TO THE GATE … CLAUDE.md's own
remedy (a) is the real fix … and it is *not done*" — is stale for the family and TRUE for its
own subject.**

The mechanism shipped: `crates/dc-worldgen/Cargo.toml:19-25` carries the doctrine comment and
**eleven** `[[example]] … test = true` blocks follow it (`:27`–`:131`), plus
`crates/dc-client/Cargo.toml:55-57` for `identify_census`. CLAUDE.md § Gates teaches the
mechanism as decided (journal/0103, 35.0 s added to the gate).

**But `flow_cost_probe` — the probe the whole rule was earned on, named twice in that very
Cargo.toml comment as the cautionary tale — was never converted.**
`crates/dc-worldgen/examples/flow_cost_probe.rs:412` still holds a bare `assert_eq!` inside
`main`; the file contains no `#[test]` and no `mod gate`; and
`grep -n "name = " crates/dc-worldgen/Cargo.toml` lists every other probe and **not this
one**. The comment introducing the fix reads *"`flow_cost_probe` sat green through a 664-test
run while broken"* — and it still would.

So the probe that broke twice, that cost a CLAUDE.md rule, and that is cited by name in the
comment introducing the remedy, **is the one probe the gate still cannot see fail.** I
recommend this as the highest-value single action in the slice, and flag it as a candidate
`corrections.md` entry: the shipped remedy's own motivating case is unremediated. *(Stated as
a search, per the absence-needs-its-pathspec rule: `grep -rn "flow_cost_probe"
crates/*/Cargo.toml` returns exactly one line — the comment at `dc-worldgen/Cargo.toml:21` —
and no `[[example]]` block.)*

---

## 4. Every user field report, listed explicitly

Rule 3 binds here: a field report stays open unless a journal entry, a shipped fix or a
measurement shows the world changed. Where the *mechanism* changed but nobody has looked, I
say exactly that rather than closing it.

| # | Entry | `file:line` | Date | Status |
|---|---|---|---|---|
| U1 | Far tier and near tier disagree about the rock on steep slopes | `ROADMAP.md:2913` | 07-26 | **STILL-OPEN.** No fix found; the entry's own first step (re-observe under `--fullbright`) explicitly not done. Pose recorded — correct practice. |
| U2 | Why does weathering lower height? (parked mid-walk) | `ROADMAP.md:2943` | 07-26 | **STILL-OPEN.** Filed verbatim, explicitly not investigated. Sits inside the top blocker (`ROADMAP.md:5105`). |
| U3 | Per-chunk palette quantization = visible checkerboard | `ROADMAP.md:3393` | 07-24 | **STILL-OPEN, diagnosed (T1 confirmed).** Its own 07-25 cross-ref: *"UNTOUCHED BY THE 2026-07-25 WORK."* Exact pose recorded. |
| U4 | Far-field LOD reconstructs material identity differently cold vs warm | `ROADMAP.md:3476` | 07-23 | **STILL-OPEN.** Diagnosed; fix direction user-authored; implementation held pending the mixture-representation plan. |
| U5 | Perf: throughput ceiling at terminal velocity | `ROADMAP.md:3519` | 07-23 | **STILL-OPEN.** |
| U6 | The far field is now BOXIER than the near field | `ROADMAP.md:3575` | 07-21 | **STILL-OPEN.** |
| U7 | HOLES IN THE GROUND — partial voxels missing side faces (user-diagnosed) | `ROADMAP.md:3624` | 07-21 | **LIKELY-RESOLVED, and now walk-confirmed.** Fixed at `:3599`; confirmed by eye at `ROADMAP.md:4814` — *"**no sky-holes** at two partial-rich stations (journal/0057 confirmed by eye)"*. See F1. |
| U8 | "Our dismal mountains" | `ROADMAP.md:4027` | 07-20 | **STILL-OPEN** (causes 2, 3, 4). Cause 3 is a *user decision* awaiting a picture-pick. **Not** recal-superseded — the calibration is OFF (`ROADMAP.md:5072`). |
| U9 | The world is a LAYER CAKE — no dip, no folding, no tilt | `ROADMAP.md:4063` | 07-20 | **STILL-OPEN.** Independently corroborated at `docs/spines.md:1088`, which names *"Sibling gap — layer-cake strata / no dip-fold"* as the same absence from the other side. |
| U10 | Thick units render as flawless monoliths | `ROADMAP.md:4081` | 07-20 | **STILL-OPEN.** |
| U11 | Loose materials do not exist in the world yet | `ROADMAP.md:4091` | 07-20 | **STILL-OPEN — but its stated premise is half falsified.** It says partial-height loose rendering is dormant *"because loose deposition never emits sub-8 columns"*. `ROADMAP.md:3636` records journal/0055 falsifying *"Worldgen does not yet emit sub-8 loose voxels"* **world-wide**. The renderer half is live; the user's actual ask (spreading on drop, angle of repose from the partials model) is untouched. **Correct the premise; do not close the entry.** |
| U12 | The sim must know about light | `ROADMAP.md:4102` | 07-20 | **STILL-OPEN.** Undesigned; no prior in the corpus. |
| U13 | No pooling/reuse of chunk or far-tile GPU resources | `ROADMAP.md:4112` | 07-20 | **LIKELY-RESOLVED as a question — answered, and the answer is "deliberately not built".** `ROADMAP.md:3561`: *"Mesh-buffer pooling: measured, deliberately NOT built (2026-07-21, journal/0051 — **the user asked for pooling; this is the numbered answer**)"*, decisive on `Mesh::insert_attribute` taking ownership. The two entries never reference each other. |
| U14 | Coal renders as pure black in the lit pass | `ROADMAP.md:4128` | 07-20 | **STILL-OPEN, correctly annotated un-walkable** (07-25 sweep row A-2; zero coal on the shipped world). Deliberate, not stale. |
| U15 | Peat and carbonaceous mudstone nearly the same colour | `ROADMAP.md:4168` | 07-20 | **STILL-OPEN — deliberately unfixed.** Do not flag. |
| U16 | Razor-straight km-scale grass/dirt frontier in the far field | `ROADMAP.md:4192` | 07-20 | **MECHANISM GONE, APPEARANCE UNVERIFIED — I will not call this resolved.** The entry pins the cause to `collapse.rs surface_sample`'s `let bare = riverbed \|\| precip < 0.10 \|\| (fringe && precip < 0.35)`. `grep -rn "let bare" crates/dc-worldgen/src/collapse.rs` → **no match**; `surface_sample` survives at `collapse.rs:778` but the binary precip threshold is gone, and `ROADMAP.md:3706` records `Block::Dirt` *"now survives only in the fallback paths"*. The named mechanism is retired — **but nobody has stood at the vantage.** Recommend re-shooting `0024-fb-ne.png`'s framing, then close or re-diagnose. |
| U17 | Texture steps toward albedo at range (+ ranges are player-facing knobs) | `ROADMAP.md:4577` | 07-22 | **STILL-OPEN.** The rider — *"rendering ranges arrive as config, not constants"* — is a standing user constraint on all future far-field work and must not be lost if the head is ever closed. |
| U18 | We cannot see where runtime goes | `ROADMAP.md:4601` | 07-22 | **LIKELY-RESOLVED as posed** — § 2 #5. Its residual is already its own entry at `:3529`. |
| U19 | Material identity illegible under splat blending (heightmap-SHAPE sketch) | `ROADMAP.md:4616` | 07-22 | **STILL-OPEN.** User sketch, explicitly *"not the only possible answer, just a thought"*. |
| U20 | Chunk gen time noticeable in vertical streaming | `ROADMAP.md:4628` | 07-22 | **LIKELY-RESOLVED as posed, with a named successor.** The suspect it sharpened to (`LOAD_BUDGET_PER_FRAME`, synchronous main-schedule gen) was addressed by the 0083/0084 offload, whose *outcome* is U5 at `:3519`: onset delayed, ceiling unmoved. Two entries, one thread, no cross-reference. |
| U21 | The dominance flip quantizes smooth gradients | `ROADMAP.md:4649` | 07-22 | **RESOLVED-IN-PLACE** (journal/0072), and **RATIFIED AS-BUILT** by the user at `:4675` (*"bless coal as-built"*). Archive candidate. |
| U22 | The cake observation: minority phases guillotine at cell perimeters | `ROADMAP.md:4681` | 07-22 | **STILL-OPEN.** Cure named, assigned to the CoarseField extraction. |
| U23 | Texel-edge dither bands on close-pressed walls, anisotropic | `ROADMAP.md:4697` | 07-22 | **STILL-OPEN.** Two candidate mechanisms and a stated falsifier (*"press against a wall near world origin"*); nobody has run it. |
| U24 | Olivine reads as exceedingly common and surface-visible | `ROADMAP.md:4755` | 07-20 | **STILL-OPEN, UNDIAGNOSED** by its own text (*"measure before touching"*). |
| U25 | "Ours is not a deep world right now — you can keep digging into no-variety" | `ROADMAP.md:4767` | 07-20 | **STILL-OPEN.** |
| U26 | Gen never writes Quarter/Slab shapes or debris volumes | `ROADMAP.md:4778` | 07-20 | **STILL-OPEN.** See U11 — the sub-8 *loose* half did light up; the Quarter/Slab *structure* half is untouched. |
| U27 | Walk 0059 — the skin is still two flat colours | `ROADMAP.md:4811` | 07-22 | **STILL-OPEN, explicitly HELD at the user's direction.** Deliberate. Two of its three questions closed in the same entry. |
| U28 | The bare-cell fallback: a walker stood on paint over nothing | `ROADMAP.md:4833` | 07-22 | **STILL-OPEN.** Its claim that `LOAM` *"is registered in no class"* is still true — `crates/dc-core/src/materials/mod.rs:82` defines it; its only other uses are a meshing colour (`meshing.rs:1166`) and tests. |
| U29 | Console v1 "still unusable" | `ROADMAP.md:3921` | 07-20 | **RESOLVED-IN-PLACE** (console v2, journal/0035). Archive candidate. |
| U30 | Thin bright seams · far LOD sheet buried · pixel gaps between far tiles (3 entries) | `ROADMAP.md:4224`, `:4246`, `:4258` | 07-19/20 | **RESOLVED-IN-PLACE**, all three walk-verified in their own entries. Archive candidates. |
| U31 | Zero coal on the shipped world — options (a)–(d) | `ROADMAP.md:2983` | 07-25 | **STILL-OPEN and explicitly USER-OWNED** (*"USER CALL REQUIRED — this is a world-content question, not a bug to quietly fix"*). The engineering half shipped (journal/0106); the content call has not been made. Its sibling `:3034` is the same call one file over, also open. |

**31 rows covering 33 entries. 23 stay open. 4 I judge answered elsewhere in the corpus
(U7, U13, U18, U20). 5 entries are resolved-in-place archive candidates. 1 (U16) has lost its
mechanism but keeps its observation.** No user field report is closed on reasoning alone.

---

## 5. Additional findings worth the integrator's time

**F1 — Two "UNWALKED" tags were discharged by walk 0059 and never updated.**
`ROADMAP.md:3610`: *"**UNWALKED** — nobody has seen the holes gone"*; `:3655`: *"**Unwalked**
— the fix landed after the user's session closed, so nobody has seen the patches gone."* Both
were walked. `ROADMAP.md:4814-4817`: *"**no sky-holes** at two partial-rich stations
(journal/0057 confirmed by eye), and **no 28.8 m chunk patches** — a 120 m top-down frame
shows organic blobs with wandering contacts (journal/0058 confirmed)."* Three entries, one
section; the confirmation sits ~1,160 lines below the tags it discharges.

**F2 — `ROADMAP.md:3379`, "Two declaration defects on `dc:deep/weather_inventory`" — BOTH
FIXED, verified at source.**
(a) `crates/dc-worldgen/src/deeptime/runner.rs:869-870`: *"epoch's BioMod as a real
within-epoch read** (see `WINV_READS_*` — the former `reads_prev` was a fiction the tie-break
decided; spine-audit 2026-07-25)"*, with `WINV_READS_AGENTS` at `:542` carrying `BioMod` in
`reads`.
(b) `runner.rs:532`: *"**`Exposed` is deliberately NOT declared**"* — the false declaration
was removed rather than made true, which is the honest disposal.
The entry still reads as owed work.

**F3 — `ROADMAP.md:4861`, "Charcoal's premise expired and the code still encodes the
conclusion" — the code no longer encodes it.** The entry's two stated blockers were *"**no
charcoal material is registered at all**"* and *"`deep_class` routes a charcoal-tagged unit
to its mineral host"*. Both false now: `crates/dc-core/src/materials/mod.rs:133` defines
`pub const CHARCOAL: MaterialId = MaterialId(MatRepr::M25)` with a registry entry at `:686`
and the token `"dc:charcoal"` at `:213`; `crates/dc-worldgen/src/geology.rs:364` reads
*"[`Biofacies::Charcoal`] routes to [`CLASS_ORGANIC_CHARCOAL`] **as of**…"* with the live arm
at `:430`; and it is now loose-formed at `crates/dc-worldgen/src/fill.rs:419`. Shipped by
journal/0063.

**F4 — `ROADMAP.md:3053`, the `reads_prev`-tie-break entry, is fully discharged.** Both halves
carry ✅ inline (`:3057` option (a) shipped journal/0104; `:3079` head's under-declaration
shipped journal/0107) and only the "kept for the record" diagnoses remain. At 50 lines it is
one of the section's larger dead weights.

**F5 — an orphaned paragraph, and the only structural defect I found.**
`ROADMAP.md:4721-4728` — *"earth-processes § 2 (Igneous) is a sketch; nothing is built.
Arc/rift provenance raises elevation but builds no edifices… the fastest legal
short-gradation mountain on Earth (a stratovolcano is ~3 km of relief in a ~20 km
footprint)…"* — sits **inside** the texel-edge-dither entry (`:4697`) with no bullet of its
own. It is plainly the body of a separate "no volcanism / no constructive edifices"
observation whose header was lost in an edit. It is currently invisible to anyone scanning
entry headers, and it is not counted among the 129. **Recommend: give it its own bullet.**

**F6 — two entries confirmed STILL-OPEN at source**, so the integrator need not re-check
them. `ROADMAP.md:3163` — `DeepField::strata` is still `Vec<DeepStrata>`
(`crates/dc-worldgen/src/deeptime/field.rs:473`); the CSR collapse is **not** done.
`ROADMAP.md:3367` — `derive_regolith_at` still builds from
`FactLedger::empty_with_bedrock(strata)` (`field.rs:822`), so the derived `H` still cannot see
`weather_inventory`'s facts. Both are on the weathering arc's critical path and both are
cross-referenced from `ROADMAP.md:3175` as competing for one residency budget.

**F7 — `ROADMAP.md:4179`, "the client can only ever open seed 1337", is STILL TRUE.**
`crates/dc-client/src/bench.rs:17` — `pub const BENCH_SEED: i32 = 1337;` — consumed at
`app.rs:196`, `:206`, `:691`; no `--seed` argument exists. This underpins both open coal
entries (`:2983`, `:3034`), so it matters that it has not quietly changed.

**F8 — `ROADMAP.md:4399`'s "still open" half is genuinely live.** Sealing `TerrainGen` behind
`pub(in crate::authority)` has not happened: `crates/dc-client/src/worldgen.rs:27` is
`pub struct TerrainGen`. The entry is right, and it is right for the reason it gives (keys
3/4 still name it). Kept as OPEN rather than RIP for that reason.

---

## 6. Full status table — all 129 entries

`OPEN` · `RES` = likely-resolved, evidence in Note · `RIP` = resolved-in-place, archive
candidate · `CD` = cannot-determine · `RECAL` = superseded by recalibration. `U#`/`F#`
cross-reference §§ 4–5.

| # | `ROADMAP.md:` | Entry | Date | Status | Note |
|---|---|---|---|---|---|
| 1 | 2884 | Gate cannot see broken doc links | 07-28 | OPEN | Booked for a user conversation; one day old |
| 2 | 2913 | Far/near tier disagree on steep slopes | 07-26 | OPEN | U1 |
| 3 | 2943 | Why does weathering lower height? | 07-26 | OPEN | U2 |
| 4 | 2960 | `dc:deep/climate` lagged read (struck) | 07-25 | RIP | ✅ journal/0107 |
| 5 | 2970 | Same — "original entry, for the record" | 07-25 | RIP | Deliberate duplicate of #4 |
| 6 | 2983 | Shipped world has ZERO COAL | 07-25 | OPEN | U31; user call (a)–(d) |
| 7 | 3034 | Coal-dig test runs on a world no player can open | 07-25 | OPEN | Same user call, one file over |
| 8 | 3053 | Tie-break deciding physics / `reads_prev` | 07-25 | RIP | F4 — both halves ✅ |
| 9 | 3103 | `DeepField::chapters` built, nothing calls it | 07-25 | OPEN | Confirmed live: `spines.md:1088` |
| 10 | 3110 | Per-cell owning container = a header × 297,025 | 07-25 | RIP | ✅ journal/0102 inline |
| 11 | 3133 | A probe that can fail is invisible to the gate | 07-25 | OPEN | **§ 3** — stale for the family, **true for `flow_cost_probe`** |
| 12 | 3150 | Same lever, ✅ DONE restatement | 07-25 | RIP | Duplicate of #10 |
| 13 | 3163 | `DeepField::strata` is `Vec<DeepStrata>` | 07-25 | OPEN | F6 — `field.rs:473` |
| 14 | 3182 | Front's voxel-tier mass error is NOISE | 07-25 | RIP | ✅ DIAGNOSED journal/0103 |
| 15 | 3240 | `pore_rider_share` offset not disjoint | 07-25 | RIP | RESOLVED inline, journal/0105 |
| 16 | 3268 | Front's parent alternates diorite/granite | 07-25 | OPEN | Member-dither family; twin at #39 |
| 17 | 3280 | `identify(pos)` can say `UNRECORDED` | 07-25 | RIP | ✅ FIXED journal/0101 |
| 18 | 3294 | `get_contents` reports air over solid rock | 07-25 | RES | Its OWED half (a tier flag that can say "unrecorded") shipped as #17 |
| 19 | 3320 | One recorded voxel over 64 unrecorded | 07-25 | OPEN | Thin-record family; undiagnosed |
| 20 | 3326 | `scan_region`/`get_block` blind to stratigraphy | 07-25 | OPEN | Structural and deliberate; doctrine now in CLAUDE.md § Agent walks |
| 21 | 3338 | `strata` Vec capacity slack 54.02 MiB | 07-25 | RIP | ✅ DONE; note the stale S19 denominator flagged inside |
| 22 | 3367 | `derive_regolith_at` derives `H` from an empty ledger | 07-24 | OPEN | F6 — `field.rs:822` |
| 23 | 3379 | Two `weather_inventory` declaration defects | 07-24 | RES | **F2 — both fixed**, `runner.rs:532`, `:869` |
| 24 | 3393 | Palette-quant checkerboard station | 07-24 | OPEN | U3; exact pose recorded |
| 25 | 3461 | Look-at-voxel contents inspector | 07-24 | RIP | SHIPPED `f594580` |
| 26 | 3476 | Far-field LOD material identity, cold vs warm | 07-23 | OPEN | U4; held pending the mixture-representation plan |
| 27 | 3519 | Perf throughput ceiling at terminal velocity | 07-23 | OPEN | U5 |
| 28 | 3529 | Perf instrument can't show the frame-thread envelope | 07-23 | OPEN | The real residual of #114 |
| 29 | 3537 | Async-offload accepted corner (pre-edit snapshot) | 07-23 | OPEN | ACCEPTED by the user; **the detection hook is still owed** |
| 30 | 3552 | Deep-cell-square surface-material frontiers | 07-22 | RIP | RESOLVED journal/0073 |
| 31 | 3561 | Mesh-buffer pooling: measured, deliberately NOT built | 07-21 | OPEN | Sanctioned non-fix; it is the answer to U13 |
| 32 | 3575 | Far field boxier than near field | 07-21 | OPEN | U6 |
| 33 | 3586 | Organics render as boxes (form keyed on CLASS) | 07-21 | OPEN | `fill.rs:414-419` still class-keyed; charcoal since added, organic soil/peat/coal not |
| 34 | 3599 | HOLES: FIXED — tagged "UNWALKED" | 07-21 | RES | **F1 — walked, journal/0059** |
| 35 | 3613 | Two assumptions the partials-first world falsified | 07-21 | OPEN | Recorded so nobody re-derives; its (2) is #32's root |
| 36 | 3624 | HOLES — partial voxels missing side faces | 07-21 | RES | U7 — fixed and now walked |
| 37 | 3643 | "World under-expresses `H`": RETRACTED | 07-21 | RIP | corrections #28 |
| 38 | 3651 | "Surface material quantized per chunk": FIXED — "Unwalked" | 07-21 | RES | **F1 — walked, journal/0059** |
| 39 | 3659 | Mixed voxels carry no member dither | 07-21 | OPEN | Untested twin of #16; cross-ref already present |
| 40 | 3676 | 91.4 % of land skins to ONE block | 07-21 | OPEN | Largely answered inline (no soil substance exists); `LOAM` still classless, `mod.rs:82` |
| 41 | 3719 | Caves ↔ hydrology integration thread | 07-25 | RIP | SUBSUMED by the FLOW arc; kept as a pointer |
| 42 | 3738 | Eolian strata record costs +92.76 MB | 07-21 | CD | Undiagnosed; the residency baseline moved twice since (#10, #21) so the *number* is almost certainly stale, but I did not re-measure |
| 43 | 3746 | 1-D wind march dumps residual at the downwind edge | 07-21 | OPEN | Small, mechanical, undiagnosed |
| 44 | 3752 | Teleport-storm hypothesis FALSIFIED as the trigger | 07-21 | RIP | Superseded by #45–#48 |
| 45 | 3774 | Confirmed at horizon 6 — the fix holds | 07-21 | RIP | journal/0065 |
| 46 | 3789 | "+0.54 MB/jump residual" was a windowing artifact | 07-21 | RIP | Same entry; yields the ≥250-jump rule |
| 47 | 3801 | FIXED: eviction landed, the march is flat | 07-21 | RIP | **This is what refutes #99** |
| 48 | 3805 | DIAGNOSED: the leak is host-RAM | 07-21 | RIP | journal/0050 |
| 49 | 3835 | Shared `CARGO_TARGET_DIR` serves a stale binary | 07-21 | OPEN | corrections #27; doctrine live in CLAUDE.md, root cause still undiagnosed |
| 50 | 3848 | GPU DeviceLost under a teleport storm at h6 | 07-21 | RES | Superseded by #45: storm *and* idle at h6, all exits 0, no `DeviceLost` |
| 51 | 3865 | Stub inventory filed / ruin-posts undocumented | 07-21 | RES | **§ 2 #3 — `ruin_posts` deleted, `stubs.md:30`** |
| 52 | 3875 | Sub-km relief / roughness decay: MEASURED | 07-21 | RIP | The open part lives in § Sequenced, not here |
| 53 | 3879 | `climate_at` half-cell offset: FIXED | 07-21 | RIP | journal/0043 |
| 54 | 3886 | Far field cuts off at 1.2 km: FIXED | 07-21 | RIP | `--horizon` |
| 55 | 3889 | The haze curve limits the usable vista | 07-21 | OPEN | Explicitly a user-owned visual call; cheap, needs the user's eye |
| 56 | 3896 | Material placement rules are climate mocks: DECIDED | 07-21 | RIP | Pointer to the decision |
| 57 | 3904 | Same — the original observation | 07-21 | OPEN | Engineering residue live: consume `exhum`/`t_crust`, form-from-provenance |
| 58 | 3921 | Console v1 "still unusable" | 07-20 | RIP | U29 |
| 59 | 3934 | Unlock ranking (assistant, unratified) | 07-20 | OPEN | (1) persistence (2) inventory (3) loose (4) water — none shipped |
| 60 | 3949 | Test-suite time trending 382 → 502 → 541 s | 07-20 | OPEN | The figures are stale (the 07-28 gate is 808 tests, `ROADMAP.md:4948`); **the absence of a policy is not** |
| 61 | 3957 | The untold "culture-language-balrog example" | 07-20 | OPEN | Awaits the user telling it; destination named |
| 62 | 3965 | `--fullbright` is blind to geometry | 07-20 | RES | Both its proposals shipped as journal/0031 — `crates/dc-client/src/edgepass.rs` and `shaders/edges.wgsl` exist |
| 63 | 3991 | "Lit before/after is invalid": RETRACTED | 07-20 | RIP | corrections #18/#19 |
| 64 | 3998 | INSTRUMENT: fullbright is blind to shape | 07-20 | RES | Its *"proposed fix, filed not built"* is `--edges`, shipped; the doctrine is now in CLAUDE.md |
| 65 | 4014 | INSTRUMENT: fullbright does not disable distance fog | 07-20 | RES | Fixed journal/0031 — CLAUDE.md: *"Fullbright also no longer applies distance fog (0031)"* |
| 66 | 4027 | "Our dismal mountains" | 07-20 | OPEN | U8; causes 2/3/4. Not recal-superseded — the calibration is OFF |
| 67 | 4063 | The world is a LAYER CAKE | 07-20 | OPEN | U9; corroborated `spines.md:1088` |
| 68 | 4081 | Thick units render as flawless monoliths | 07-20 | OPEN | U10 |
| 69 | 4091 | Loose materials do not exist in the world yet | 07-20 | OPEN | U11 — premise half-falsified, the ask untouched |
| 70 | 4102 | The sim must know about light | 07-20 | OPEN | U12 |
| 71 | 4112 | No pooling/reuse of chunk or far-tile GPU resources | 07-20 | RES | U13 — answered by #31 |
| 72 | 4128 | Coal renders as pure black in the lit pass | 07-20 | OPEN | U14; correctly annotated un-walkable |
| 73 | 4168 | Peat vs carbonaceous mudstone colour | 07-20 | OPEN | U15 — deliberately unfixed |
| 74 | 4179 | The client can only ever open seed 1337 | 07-20 | OPEN | **F7 — verified still true**, `bench.rs:17` |
| 75 | 4192 | Razor-straight grass/dirt frontier in the far field | 07-20 | CD | U16 — mechanism gone, appearance unverified |
| 76 | 4224 | Thin bright seams between far patches | 07-20 | RIP | U30; corrections #11 |
| 77 | 4246 | Far LOD sheet buried under the near field | 07-19 | RIP | U30 |
| 78 | 4258 | Clear pixel gaps between far-field tiles | 07-19 | RIP | U30 |
| 79 | 4269 | The Voxy-vs-Distant-Horizons thread | 07-19 | OPEN | Persistent edit-updated LOD store still owed; Aokana SVDAG still a candidate |
| 80 | 4285 | Walk 17 — far-sheet parallelogram sky holes | 07-19 | RIP | Residual: cosmetic stitch lines, filed |
| 81 | 4304 | Walk 16 — the lit path erases low-fraction mixtures | 07-19 | RIP | Photographically verified |
| 82 | 4318 | Walk 14 — fullbright lost the mixture speckle | 07-19 | OPEN | Resolved, but **the in-world speckle photograph + member-contact closeups are still owed** |
| 83 | 4331 | Placeholder texture tiling: cleanup DONE | 07-19 | OPEN | Same shape — the geology outcrop before/after is still owed to a milestone walk |
| 84 | 4348 | PBR-1 loose ends (8 sub-items) | 07-19 | OPEN | Mostly accepted-as-placeholder; revisit with authored textures + POM |
| 85 | 4386 | Walk 12's "blocking regression" was a misdiagnosis | 07-19 | RIP | corrections #10 |
| 86 | 4393 | Pose replies echo the voxel coordinate — DONE | 07-19 | RIP | journal/0021 |
| 87 | 4399 | Far mesh from S1 `TerrainGen`: RESOLVED | 07-19 | OPEN | **F8** — the seal is still not done, `worldgen.rs:27` |
| 88 | 4413 | Walk 12 residue (test time, Large coarsening) | 07-19 | OPEN | |
| 89 | 4422 | Walk 13 — `surface_snapped` absent | 07-19 | RIP | journal/0021 |
| 90 | 4429 | Walk 11 + step-3 loose ends | 07-19 | OPEN | Leg-fold tuning pass; foot-IK can still see the S1 phantom at the load edge |
| 91 | 4441 | Walk 5 — no auto step-up | 07-19 | RIP | DECIDED: stays jump-required |
| 92 | 4450 | Geology v1 loose ends | 07-19 | OPEN | `MixtureTable` persistence still owed; `def_changed` events still absent |
| 93 | 4461 | Walk 10 + 3d loose ends | 07-19 | OPEN | Per-chunk `flow_energy` rounding still open |
| 94 | 4473 | Walk 9 + 3c-2 loose ends | 07-19 | OPEN | Pregen introspection as dev MCP tools still wanted |
| 95 | 4485 | Walk 8 + user live observations | 07-19 | OPEN | Nobody owns body orientation; faceless heads block facing verification |
| 96 | 4500 | Walk 7 loose ends — S1 phantom old world | 07-19 | RES | The phantom is gone by #87's own text: *"the phantom old world ~1 km down is gone"* |
| 97 | 4512 | Walk 6 loose ends | 07-19 | OPEN | Surface-scan window still sized to S1 amplitude |
| 98 | 4521 | Walks 3–4 (blowout, chasm speckle) | 07-19 | OPEN | The speckle diagnosis is still owed |
| 99 | 4530 | The embedded HostWorld never evicts chunks | 07-19 | RES | **§ 2 #1 — `host.rs:524`; refuted by #47 in this same file** |
| 100 | 4533 | Far field doesn't see edits | 07-19 | OPEN | |
| 101 | 4537 | Edits don't survive a 2/3/4 scale switch | 07-19 | OPEN | |
| 102 | 4539 | ~2/3 of far-mesh triangles were sealed caves | 07-19 | RIP | Resolved for the worldgen far field |
| 103 | 4543 | Far meshing is main-thread, budgeted | 07-19 | CD | Chunk meshing was offloaded (0083/0084); whether far-tile meshing followed I did not verify. `ROADMAP.md:4638` still calls the far-mesh async drop-in *"unclaimed"* |
| 104 | 4545 | Rivers are straight cell-chords (S7) | — | OPEN | RiverSeg retirement is FLOW continuation (e) |
| 105 | 4547 | Terrain amplitude conservative — no voxel-scale cliffs | — | OPEN | Same lever as #66's cause 3 |
| 106 | 4548 | Site cap 240 (u8 `RegionId`) | — | RES | **§ 2 #4 — subject deleted, journal/0121** |
| 107 | 4550 | Sparse sidecar encoding for thin debris drapes | — | OPEN | |
| 108 | 4551 | Seed-stable worlds across releases | — | OPEN | Versioning policy still undecided |
| 109 | 4552 | HDR/exposure; sky-as-pass needs hook format 1 | — | OPEN | |
| 110 | 4553 | S2 checkpoint facts — 🔔 TRIGGERED | 07-25 | OPEN | Correctly promoted by the 07-25 sweep; now blocks the weathering arc, with a ratified standing constraint from `flow.md` § 11.3 |
| 111 | 4568 | `client_player_pose_set` outside the one door | 07-19 | RIP | RATIFIED same day |
| 112 | 4573 | Console follow-ups | 07-19 | RIP | RATIFIED same day |
| 113 | 4577 | Texture steps toward albedo at range | 07-22 | OPEN | U17; carries a standing user constraint on all far-field work |
| 114 | 4601 | We cannot see where runtime goes | 07-22 | RES | **§ 2 #5 — `perf.rs:26,40,68`** |
| 115 | 4616 | Material identity illegible under splat blending | 07-22 | OPEN | U19 |
| 116 | 4628 | Chunk gen time noticeable in vertical streaming | 07-22 | RES | U20 — the offload shipped; the residue is #27 |
| 117 | 4649 | The dominance flip quantizes smooth gradients | 07-22 | RIP | U21; RATIFIED AS-BUILT |
| 118 | 4681 | The cake observation | 07-22 | OPEN | U22 |
| 119 | 4697 | Texel-edge dither bands, anisotropic | 07-22 | OPEN | U23; falsifier stated and unrun. **Contains the orphan, F5** |
| 120 | 4730 | The 5–460 m band has no process — only decayed noise | 07-20 | RECAL | **The one recalibration-sensitive entry.** Its magnitudes (±26 m orogeny, ±4 m walking, ±1 m craton) are pre-0111 numbers off the old erosion budget. The *absence* of process is real and S9's C-refinement is still unbuilt — so **re-measure, do not close** |
| 121 | 4742 | The next arc: block primitives + procedural dressing | 07-20 | OPEN | User anticipation, filed for the sequence |
| 122 | 4755 | Olivine reads as exceedingly common | 07-20 | OPEN | U24 |
| 123 | 4767 | "Ours is not a deep world right now" | 07-20 | OPEN | U25 |
| 124 | 4778 | Gen never writes Quarter/Slab shapes or debris | 07-20 | OPEN | U26 |
| 125 | 4790 | The `history.rs` reject-don't-crash skip is SILENT | 07-20 | RES | **§ 2 #2 — the file is deleted** |
| 126 | 4798 | Circulation is fidelity-correct but surface-invisible | 07-20 | CD | Turns on the same surface rule whose replacement is #75's evidence; needs a look, not a grep |
| 127 | 4811 | Walk 0059 — the skin is still two flat colours | 07-22 | OPEN | U27; HELD at the user's direction |
| 128 | 4833 | The bare-cell fallback | 07-22 | OPEN | U28; `LOAM` still classless |
| 129 | 4861 | Charcoal's premise expired | 07-22 | RES | **F3 — `materials/mod.rs:133`, `geology.rs:364,430`, `fill.rs:419`** |

---

## 7. What this sweep says about the section as a whole

**43 % of the section is not an observation.** 18 `RES` + 37 `RIP` = 55 of 129 entries are
either closed or closed-and-annotated. A reader entering cold — which is every session — is
reading 2,300 lines in which nearly half the entries describe a world that no longer exists,
interleaved with 69 that describe the one that does, and **the two are not distinguishable
without opening the source.**

**The dominant failure here is not staleness-by-decay; it is closure-in-the-wrong-place.**
Every one of § 2's top five had its refutation *already in the corpus*, and three of them
already inside **this same file** (#99 refuted by #47; #114 refuted by #28; #34 and #38
discharged by #127). That matches `doc-topology`'s measured stop-block: access was never the
problem. The missing act is discharging the old entry in the same commit as the new fact —
and here the discharge would not even have required a second file to be open.

**The `RIP` mass is a mechanical, low-risk archive job** — 37 entries whose own bodies already
declare them closed. Moving them to `ROADMAP-history.md` by the archive-by-status rule would
cut the section by roughly a third with no judgement calls and no information loss, and would
make the 18 `RES` findings above visible instead of buried.

**One thing the section does exceptionally well, and it should be said:** entries carrying a
recorded *pose*, a named *falsifier*, or an explicit *"deliberately not done"* aged far better
than those carrying a prose landmark or an implicit owe. U3's and U1's poses make them
re-checkable today, eight and two days on. U23 states its own discriminating experiment. U15
and U31 say plainly who owns the call. Those are the entries a future reader can act on
without archaeology — and they are the template for the rest.
