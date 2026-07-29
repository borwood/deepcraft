# S1 — the core doctrine spine

> ## ⚠ ONE CITED FACT WENT STALE — stamped 2026-07-29 by journal/0123 (RATE)
>
> Finding **116** (`:406`) records *"RATE was ratified 2026-07-24 and never built (`dt` pinned to
> 1.0)"*. **RATE is BUILT as of 2026-07-29** — journal/0123, `dependency-graph.md` E3. The
> observation was true when this audit ran and its *body is not rewritten*; the argument it
> supports (a ratified-but-unbuilt decision with no watcher) is unaffected, and if anything is
> sharpened — the item was retired by a slice, never by a watcher. **The erosion measurements
> quoted alongside it are journal/0116's and stand.**

> ## 📋 DISPOSITION — applied 2026-07-29. Read this before acting on any finding below.
>
> | # | subject | state |
> |---|---|---|
> | 1 | `spines.md` § S-6 teaches order-derived-by-topo-sort | **⚠ PARTIAL / ESCALATED** — a **pointer banner** was added at S-6's head naming the DECIDED 2026-07-26 supersession and corrections #65, and saying *do not justify a derive-and-reject design against this section*. **The reconciliation of S-6's argument is NOT done and is owed to main session**, per the finding's own recommendation. |
> | 2 | north-star § Deviations 2 enumerates 3 of ≥6 tiering sites | **⚠ ESCALATED** — amending a user-authored, emphatic deviation block (and deciding which further sections it voids) is the user's. The unnamed sites are `north-star.md` §§ *Runtime*, *One authoring shape two execution backends*, *The triangle you cannot fully close*, and **the doc's own `blogworthy` header**. |
> | 3 | `ARCHITECTURE.md` asserts the social/history pipeline as live | **⚠ ESCALATED** — the finding says *"flag, do not apply"*, and CLAUDE.md's ruled fix is **mark ON HOLD, not strike**, on a user-ruled matter. `worldgen.md` is the template; ARCHITECTURE.md is its obvious second target. |
> | 4 | `ARCHITECTURE.md:3-4` demotes every decision to a hypothesis | **✅ APPLIED** — status line retired in place, with the DECIDED/RATIFIED reading stated. |
> | 5 | `API.md:3-4` calls itself pre-S5 | **✅ APPLIED** — status line retired; S5's exit criterion was performed in place. |
> | 6 | `SPIKES.md` has no completion state | **✅ APPLIED** — banner added: it cannot answer "has this landed", `docs/spikes/` and ARCHITECTURE.md's dated blocks can; the two named dependency gates are discharged. Verified against `ls docs/spikes/`. |
> | 7 | A-2 routes readers to § 5's `JUSTIFIED-BY`, refuted 450 lines later | **✅ APPLIED** — A-2's **Check:** now says the convention finds nothing and names the real check. |
> | 8 | fail-closed contrast premise expired | **✅ APPLIED** — cross-reference added at § Provider seams; both decisions survive, both justifications are marked. |
> | 9 | "effective reads" harm stated in derive-and-reject terms | **✅ APPLIED** — folded into the same block as #8. |
> | 10 | `API.md` advertises `world.column_summary`, no handler | **✅ APPLIED** — reciprocal ⚠ added at the domain table. **Re-verified at source:** `column_summary` exists in `dc-core/src/column.rs`; `grep -rn column_summary crates/dc-api` → nothing. |
> | 11 | `spines.md` § 6 lists 4 audits, `docs/audits/` holds 15+ | **✅ APPLIED** — index backfilled by listing (role line only), plus the baseline sweep itself. |
> | 12 | `wasmtime` deferred vs shipped | **⚠ ESCALATED** — the finding explicitly declines to attribute it. Needs someone who knows which seam `north-star.md` Deviation 1 means. |
> | 13 | crate roster omits `dc-host` / `dc-mcp-dev` | **✅ APPLIED (in scope half)** — note added at `ARCHITECTURE.md` § Shape of the system. **`CLAUDE.md`'s headless-crates list is out of this agent's scope and still omits them.** |

`doc-topology` BASELINE SWEEP, slice 1 of 9. **All quotations read at commit `f652b60`**
(working tree identical to `f652b60` for every file cited; the only diff is this file).

**Read in full:** `CLAUDE.md` (441) · `docs/design/north-star.md` (434) ·
`docs/ARCHITECTURE.md` (572) · `docs/spines.md` (1284) · `docs/API.md` (417) ·
`docs/SPIKES.md` (126) = **3,274 lines**. *(The brief's counts are each one lower —
consistent with a count that omits the final line; nothing was skipped.)*

**Opened for corroboration only, not swept:** `.claude/skills/doc-topology/SKILL.md`,
`docs/design/worldgen.md` (banner lines only), `docs/spikes/S1-results.md` (one number),
`ls docs/audits/`, `ls docs/spikes/`, `ls crates/`, one grep of `crates/`.

**13 findings.** None is resolved here — per the skill, which side wins is a user call.

---

## Ranked findings

### 1. `spines.md` § S-6 still teaches ORDER-IS-DERIVED; `ARCHITECTURE.md` retired it (user, DECIDED)

| side | citation | claim |
|---|---|---|
| A | `docs/spines.md:369-373` | *"S-6. Declared relations, never incidental order … Order exists; it must be **data** … `pipeline.rs`: passes declare reads/writes; Kahn's algorithm"* — and the entire instance list `:376-517` is written as compliance-by-topo-sort, closing `:516` *"incidental order is forbidden; declared order is fine."* |
| B | `docs/ARCHITECTURE.md:517-531` | **DECIDED 2026-07-26 (user):** *"**Derive-and-reject (today).** The kernel infers an order from `{reads, writes}` … each stage must write a distinct token naming its output, and the chain of tokens **is** the order."* vs *"**Author-and-validate (DECIDED).** Order is **data on the world** … `{reads, writes}` stop being the ordering input and become the **validator**"*, and *"The mechanism and the violation are the same choice."* |

**Provenance:** B is **user-originated and DECIDED**, and is the noun of corrections #65.
A is assistant-authored audit prose.

**Why this is the top finding.** `spines.md` is read-first item 0b, and its S-6 section is
the longest single argument in the corpus for the *exact mechanism the user retired*. The
revision tokens S-6 celebrates as its hardest-won compliance — `Forced` / `Incised`
(`spines.md:431-461`), the `reads_prev` anti-dependency (`:476-510`) — are, in B's reading
(`ARCHITECTURE.md:519-521`), the *artifact* of derive-and-reject, the same pressure that
produced `DeepAxis`, which B names as **"the violation"** (`ARCHITECTURE.md:506-511`).

**Verified absence, with its pathspec:** `rg -n '(authored|plugin-agnostic|0119|#65|AmbiguousWriters)' docs/spines.md`
at `f652b60` returns **only** `:796`, `:868`, `:890`, `:1084` — all `corrections #64/#67`
text about a different matter. **`spines.md` does not mention the 2026-07-26 decision
anywhere**, in any form: no strike, no banner, no § 4 carve-out.

**Blast radius:** maximal. Every brief is *"justified against these shapes"*
(`spines.md:76-80`); a worker who reads S-6 and not `ARCHITECTURE.md`'s last section will
build derive-and-reject and be told they complied.

*Recommendation (labelled: assistant's):* the two are reconcilable — declared-not-incidental
survives author-and-validate intact; only the *generator* half died. What is missing is the
sentence saying so, in `spines.md`. **But note the shape of this gap is precisely
corrections #65 recurring**, so the wording belongs in main session, not in a sweep.

---

### 2. `north-star.md` § Deviations #2 voids the capability tiering — and names 3 of at least 6 live sites

| side | citation | claim |
|---|---|---|
| A (user, emphatic) | `docs/design/north-star.md:419-433` | *"**NO capability tiering — a mod can author ANYTHING, including a field pass** … ASSUME MODS CAN DO ANYTHING … the **capability-tiering** in § 'The core / plugin boundary', § Passes … and § Refinement … is **EXPLICITLY NOT THE MODEL** and must not shape any design."* |
| B1 | `north-star.md:218-219` | *"field passes … Native-backend and first-party — because they are **trusted** and hot"* (named by A; **unstruck**) |
| B2 | `north-star.md:246-250` | *"**Capability, not core, gates the tiers** … the untrusted tier gets material declarations + cellular passes + data; registering a new field pass (a global solver) is a **trusted-tier capability**."* (named by A; **unstruck**) |
| B3 | `north-star.md:275-278` | *"**Runtime:** sacred … **first-party native.** — **Untrusted third-party content lives at the gen tier** … Trusted / signed content can earn the native runtime path."* (**NOT named by A**) |
| B4 | `north-star.md:309-315` | *"**One authoring shape, two execution backends** … trusted / signed + first-party → NATIVE … untrusted third-party → WASM sandbox"* (**NOT named by A**) |
| B5 | `north-star.md:326-331` | *"**The triangle you cannot fully close** … **The tier system means we never need that corner**: we never run untrusted native."* (**NOT named by A**) |
| B6 | `north-star.md:11-14` | the doc's own `blogworthy` header: *"the engine … which solves … the untrusted-mod problem with a **sandbox tier** behind a single authoring shape"* (**NOT named by A**) |

**Aggravation:** `north-star.md:230-232` gives § Refinement precedence — *"Sharpens the
core/content boundary above; **where the two disagree, this governs**"* — and § Refinement
is where B2 lives. The doc's own conflict-resolution clause points at a voided section.

**Provenance:** A is **user, emphatic, ratified** and is repeated in `CLAUDE.md:16-18`
(*"there is **no difference in permission** between native and WASM, so never justify a
core/content placement by trust"*). B1–B6 are assistant-authored.

**Known-but-live:** the skill already lists B1 as a confirmed reciprocity instance
(`SKILL.md:106-108`). **What is new here is that the void list is incomplete** — three
further sites, one of them the document's own opening blurb. A reader who trusts the
enumeration in A will believe B3–B6 survived the deviation.

*Recommendation (assistant's):* whoever discharges this should not patch the three named
sections and stop; the enumeration itself is the defect.

---

### 3. `ARCHITECTURE.md` asserts the social/history pipeline as live; the ON HOLD ruling reached only `worldgen.md`

| side | citation | claim |
|---|---|---|
| A | `docs/ARCHITECTURE.md:164-166` | *"**Deep-time history (geological, then social/territorial)** = dc-sim's coarse and statistical tiers run over pre-player time. Worldgen history and live far-simulation are one system, not two."* — and `:105-107` *"**Deep cultures accumulate ledger history** without ever being fully simulated until approached"*, `:153-154` *"The committed/fluid distinction **exists in the sim from v1**."* No banner anywhere in the file. |
| B | `CLAUDE.md:359-383` (user, 2026-07-28) | *"**THE STATUS IS ON HOLD, NOT NEVER** … Where a doc did assert a live pipeline stage (`worldgen.md` § Above the region scale item 4), **the fix ruled by the user was mark ON HOLD, not strike** … **That doc's banner and its § Sequencing are the template.**"* |
| B' | `docs/design/worldgen.md:6`, `:37`, `:90`, `:116` | the four ON HOLD markers that were applied. |
| C | `docs/spines.md:1093` | *"the whole S2 statistical tier … **zero production callers workspace-wide.** Its one caller was `pregen/history.rs`, removed as unratified bootstrap content"* — and `:104-106` *"the running instance is **not currently running**."* |

**Provenance:** B is user-originated and dated **2026-07-28** (the watermark day). A is
2026-07-18 assistant sketch, never revised.

**Blast radius:** high. `ARCHITECTURE.md` is read-first item 3 and is the *only* place the
social-history pipeline is still stated in the indicative. The template B names exists and
was applied to exactly one doc.

*Recommendation (assistant's):* this is the template's second obvious target, but marking
it is a doc edit on a user-ruled matter — flag, do not apply.

---

### 4. `ARCHITECTURE.md`'s own header demotes every decision in it to a hypothesis

| side | citation | claim |
|---|---|---|
| A | `docs/ARCHITECTURE.md:3-4` | *"Status: **pre-spike sketch, 2026-07-18. Everything here is a working hypothesis** until the spike named next to it lands (see SPIKES.md)."* |
| B | same file, throughout | `:64` *"DECIDED by S4 (2026-07-18)"* · `:92` *"DECIDED 2026-07-18: chunks stay 32³"* · `:111` *"DECIDED 2026-07-18 (S1 feel pass)"* · `:168` *"RATIFIED 2026-07-19"* · `:206`, `:238`, `:311`, `:355`, `:417`, `:456`, `:466`, `:491` — **twelve dated DECIDED/RATIFIED blocks, eight of them marked `(user)`.** |
| C | `CLAUDE.md:50-51` | read-first item 3: *"`docs/ARCHITECTURE.md` — **decisions with dates**"* |
| D | `docs/spines.md:64` | *"`docs/ARCHITECTURE.md`, `docs/design/*` \| what we **decided**, and why, with dates"* |

**Shape 6 (summary outran its source), inverted:** the header was true when written and no
decision reversed it — it simply stopped describing the file. Both read-first surfaces (C, D)
already carry the corrected reading; only the file itself does not.

**Blast radius:** high and *invisible*, because the sentence sits above everything and reads
as governance. A cold agent instructed to check a claim against `ARCHITECTURE.md` is told
by line 3 that user ratifications there are provisional.

*Recommendation (assistant's):* the header is the falsified half. Better-supported side: B/C/D.

---

### 5. `API.md` calls itself pre-S5; its body records S5's results as normative

| side | citation | claim |
|---|---|---|
| A | `docs/API.md:3-4` | *"Status: **pre-S5 design**, 2026-07-18. **S5 implements a thin slice of this and its exit criterion is revising this doc** into the v0 conventions spec."* |
| B | same file | `:46-47` *"the consumer-id tie-break (**added by S5**)"* · `:52` *"**Normative as of S5.**"* · `:347-351` *"**DECIDED by S5 measurement** … measured at ~21× the bytes"* · `:390` *"**v0 implementation notes (adopted from S5**, details in `docs/spikes/S5-results.md`)"* · `:395-397` *"the host owns token contents … (**proven**: contraband defines leave zero trace)"* |

Same shape as #4. The exit criterion A states — *revise this doc* — **was performed**,
in place, and the status line was not touched.

**Blast radius:** medium-high. `API.md` is read-first item 3's first follow-on, and A
invites a reader to discount `:390-416` as speculation when it is measured spike output.

---

### 6. `SPIKES.md` is a backlog with no completion state, and `ARCHITECTURE.md:4` defers to it

| side | citation | claim |
|---|---|---|
| A | `docs/SPIKES.md:1-3, 8-125` | *"# Spike backlog … Ordered by risk"* — S1–S8, each with an open **Exit** criterion; `:114-120` *"**Deliberately not spiked yet** — Multiplayer/networking … no further work until the surface stabilizes (**post-S5**)"*; `:122-125` *"**Suggested order.** S1 and S2 first … Then S3 …, S5 …, S4, S6, S7."* **No spike carries a landed/superseded marker.** |
| B | `docs/ARCHITECTURE.md:111` | *"**DECIDED 2026-07-18 (S1 feel pass)**: player height = 2 voxels, voxel = 0.9 m"* — i.e. S1's exit (`SPIKES.md:19`) is discharged and recorded. Same for S4 (`ARCHITECTURE.md:64` vs `SPIKES.md:60`). |
| C | `ls docs/spikes/` at `f652b60` | **S1, S2, S3, S4, S5, S6, S7, S8, S9, S9b, S10, S11, S12, S13, S15, S16, S17, S19, S20 results docs exist** — the corpus ran twelve spikes past `SPIKES.md`'s enumeration. |
| D | `docs/ARCHITECTURE.md:3-4` | *"Everything here is a working hypothesis **until the spike named next to it lands (see SPIKES.md)**"* |

**The chain is what makes this load-bearing:** D makes `SPIKES.md` the authority on which
architectural claims are settled, and `SPIKES.md` cannot answer — it has no state field.
Findings #4 and #6 are one defect read from two ends.

**Also inside A:** `:118` *"In-game editor UX — depends on S1 (scale), S5 (api)"* and
`:114-116`'s post-S5 gate both name dependencies that have landed.

---

### 7. `spines.md` § 5's `JUSTIFIED-BY` convention is pointed at by A-2 and refuted 460 lines later, in the same file

| side | citation | claim |
|---|---|---|
| A | `docs/spines.md:713-720` | A-2's prescribed check: *"**Check:** § 5's convention."* |
| A' | `docs/spines.md:1163-1174` | *"# 5. The justification convention … `// JUSTIFIED-BY:` … **A `spine-audit` sweep greps these** and asks, one by one, whether the cited constraint is still true."* |
| B | `docs/spines.md:1176-1185` | *"**Drift check (2026-07-24): the literal marker has zero instances.** A corpus grep … returns this file and the skill … nothing in `crates/` … **every A-2 caught so far was caught by reading, not grepping** … Left as a **flag to the main session**."* |
| C | `.claude/skills/doc-topology/SKILL.md:27-31` | measured 2026-07-28: *"**A tagging convention has already been tried here and got ~0 % adoption.** `JUSTIFIED-BY` had a documented convention, a stated validator, and a named sweep … **3 occurrences, 0 in `crates/`.**"* |

**Shape 2 exactly** (a claim refuted in its own neighbourhood), and it is the corrections #65
geometry — pointer and refutation in one file, ~460 lines apart, both true when written.
The flag B raised on 2026-07-24 was never discharged; C then measured it as a general law.

**Blast radius:** medium-high — A is the standing instruction for the second-most-cited
anti-shape, and it routes readers to a mechanism its own file proves finds nothing.

---

### 8. `ARCHITECTURE.md` § Provider seams cites a fail-closed rule that `ARCHITECTURE.md` says disappears

| side | citation | claim |
|---|---|---|
| A | `docs/ARCHITECTURE.md:448-454` | *"**Conflicts warn, they do not fail closed** … deliberately softer than **the pass graph's fail-closed rule for two creators of one resource**"* — the contrast case that carries the argument. |
| B | `docs/ARCHITECTURE.md:523-526` | *"'Ambiguity' is only a defect when the engine is trying to infer a sequence; given an authored one it is just the sequence, and **the rejection that forces revision tokens disappears**."* |

Same file, 70 lines apart, no cross-reference. The *decision* in A survives; its stated
**justification by contrast** is premised on a rule B retires. Shape 5 (an expired premise),
with the unusual property that the expiry and the justification are in one document.

---

### 9. `ARCHITECTURE.md` § Provider seams' "effective reads" harm is stated in derive-and-reject terms

| side | citation | claim |
|---|---|---|
| A | `docs/ARCHITECTURE.md:442-447` | *"**Effective reads.** … Without this a provider is a **hidden edge in the pass graph** … the **topo-sort is silently wrong**, and the failure is order-dependent."* |
| B | `docs/ARCHITECTURE.md:520-521` | *"`{reads, writes}` stop being the ordering input and become the **validator**."* |

Under B there is no topo-sort to be silently wrong; the same declaration is still needed, but
as a **validation** input. The mechanism survives, the stated harm does not. Lower-ranked than
#8 because the decision's *content* is unaffected — this is a wording debt, and I flag it only
because §5-shape premises are what this sweep exists to surface.

---

### 10. `API.md` advertises `world.column_summary`; `spines.md` records that no handler exists

| side | citation | claim |
|---|---|---|
| A | `docs/API.md:62` | the `world` domain's Queries cell lists *"get_block, scan_region, raycast, **column_summary**, region_snapshot"* — no marker, no caveat. |
| B | `docs/spines.md:1092` | *"`docs/API.md` lists a `world.column_summary` query but **no dc-api handler exists** for it (2026-07-23 sweep)"* |

**One-directional pointer** (skill § 6): B names A; A carries nothing. Five days old at the
watermark. This is the `corrections #12 / S10` shape, and it is now governed by
`CLAUDE.md:54-82`'s DECIDED 2026-07-28 rule — *"a results doc must carry, at its top, a
pointer to anything that later refuted, superseded or re-scoped it"* — whose own
`:80-82` **Backlog** line concedes the corpus was never swept for missing banners.

---

### 11. `spines.md` § 6 lists 4 audits; `docs/audits/` holds 15, against § 6's own rule

| side | citation | claim |
|---|---|---|
| A | `docs/spines.md:1253-1277` | *"# 6. The audits (`docs/audits/`). Standing inventories … **consult before re-deriving**"* — then exactly four entries, all dated 2026-07-22. |
| A' | `docs/spines.md:1279-1283` | *"**These were nearly lost** … That is A-4 committed on the day A-4 was written … **Any future sweep lands here, in the same commit as the work that used it.**"* |
| B | `ls docs/audits/` at `f652b60` | **15 entries**, incl. `2026-07-23-block-consumer-inventory.md`, `2026-07-24-roadmap-staleness-sweep.md`, `2026-07-25-contents-empty-over-solid-diagnosis.md`, `2026-07-26-doc-topology-sweep.md`, `2026-07-28-corrections-recoding.md`. |

An index that states its own maintenance rule and is eleven entries behind it. Self-refuting
in the A-4 sense the section was written to name.

---

### 12. ⚠ I CANNOT ATTRIBUTE THIS — `wasmtime` deferred, and `wasmtime` shipped

| side | citation | claim |
|---|---|---|
| A | `docs/design/north-star.md:407-417` (Deviations #1, user 2026-07-24) | *"the trusted/untrusted safety split **+ the ABI/WASM spike are DEFERRED** … build **native `abi_stable`-shaped** … **`wasmtime`/sandbox to follow** when there are untrusted mods to sandbox."* And `:376-383`: the SDK/ABI spike is listed under *"Not yet proven"*, amended *"NOT GATING"*. |
| B | `docs/API.md:414-416` | *"**Plugin ABI: wasm32-unknown-unknown** (no WASI, no ambient authority); one host import `dc.call` carrying postcard request/response; guest exports `dc_run`/`dc_tick`"* — filed under *"v0 implementation notes (**adopted from S5**)"*, i.e. built and measured. |
| B' | `docs/ARCHITECTURE.md:15-16` | the system diagram's *"WASM plugins (wasmtime)"* arrow into dc-api, drawn as a shipped consumer. |
| B'' | `rg -l 'wasmtime' crates/` at `f652b60` | `crates/dc-host/{src/lib.rs,Cargo.toml,tests/wasm_plugin.rs,tests/parity.rs}` — a real host with a plugin parity suite. |

**Why I am not calling this a contradiction.** A plausibly speaks about the *content-SDK*
seam (Pass / Material / `ctx`), B about the *dc-api command* seam. If that is right, both are
true and **the missing artifact is the sentence distinguishing them** — the skill's explicit
"not automatically a correction" case. If it is wrong, A defers work that shipped before it
was written, which would be a significant misstatement in a read-first doc. **I cannot
adjudicate which seam A means from the text**, and I decline to guess. Someone who knows what
S5's host actually crosses should rule.

---

### 13. Low: the crate roster in `CLAUDE.md` and `ARCHITECTURE.md` omits two shipped crates

`CLAUDE.md:430-431` — *"Headless crates (**dc-core, dc-sim, dc-worldgen, dc-api,
dc-physics**) never depend on rendering/OS"* — and `ARCHITECTURE.md:9-35`'s diagram (dc-client
· dc-api · dc-sim · dc-worldgen · dc-core). `ls crates/` at `f652b60` also holds **`dc-host`**
(the wasm plugin host, headless) and **`dc-mcp-dev`**. Strictly doc-vs-code and therefore
`spine-audit`'s remit, but recorded because the two rosters are *also* inconsistent with each
other (`dc-physics` appears in `CLAUDE.md` and in `ARCHITECTURE.md` § Physics prose, never in
its diagram). No decision rests on it today.

---

## Claim inventory — S1 slice

Load-bearing claims each doc asserts **as current**, at `f652b60`. Terse; transient artifact
for stage 2.

### `CLAUDE.md`
1. `:7-21` north-star is read-first item 0; core = cell storage + pass-runner + field-solver **primitives** + stable API; ratified 2026-07-23.
2. `:13-15` every pass is content incl. tectonics/erosion; **pass ORDER is authored per world** (DECIDED 2026-07-26).
3. `:15-18` trusted/untrusted split and ABI/WASM tiering **DEFERRED**; no permission difference native vs WASM; never justify core/content placement by trust.
4. `:22-29` spines carries S-1…S-9, A-1…A-7, plus the built-and-uncalled index; work is justified against it; deviation is loud, user-ratified, recorded in its § 4.
5. `:30-36` ROADMAP is the live sequence; completed work archived to `ROADMAP-history.md` **by status, not age**.
6. `:37-45` three sweeps exist and check different things (spine-audit=code, staleness=newer work, doc-topology=docs vs each other).
7. `:50-51` `ARCHITECTURE.md` = decisions with dates; then API.md, design/*, rendering/PIPELINE.md.
8. `:54-59` **DECIDED 2026-07-28 (user):** spike results are IMMUTABLE BODY, MUTABLE HEADER; a results doc must carry a top-of-file pointer to what refuted it.
9. `:68-73` a one-directional pointer is not a pointer; measured 8 of 15 correction→file edges one-directional, 14 of 30 audit/spike files carry no staleness marker.
10. `:74-79` the stamping obligation lands on the **writer of the correction**, in the same commit.
11. `:80-82` the rule applies to `docs/spikes/`, `docs/audits/`, probe reports; **the corpus was never swept for missing banners**.
12. `:83-93` the loose-ends lookup is exactly three loci: `stubs.md`, `spines.md` § 3, ROADMAP Owed/Observed.
13. `:97-122` journal is append-only narrative + blog feedstock; four named lenses.
14. `:126-132` build rules: cargo off PATH, one invocation at a time, shared `CARGO_TARGET_DIR`, prefer `--release`.
15. `:134-140` three gates: fmt, clippy `-D warnings`, test — all `--release`, workspace-wide.
16. `:142-149` a gate is evidence only about code it ran; `cargo clean -p` first; verify by test **name or count**.
17. `:152-159` `cargo test` builds examples and never runs them; an example that can fail belongs in the gate.
18. `:166-177` the mechanism is `[[example]] test = true`; one file, two consumers.
19. `:178-188` size the test not the report; converted probes add **35.0 s**.
20. `:189-192` assert invariants, never snapshots.
21. `:193-196` a printed caption is a published claim the gate cannot check.
22. `:197-205` read the Tee'd log, never the console capture.
23. `:206-219` grep the build log for `Compiling`/`Checking` unanchored, and the path; wait for `Get-Process cargo,rustc` empty; re-read `.agent-build.lock`.
24. `:223-252` **the walk loop: Claude drives every step**; tour-map first, always; a null from the tour map is a result.
25. `:256-258` non-shader tests launch `--fullbright`.
26. `:259-264` check `eye_in_solid`; yaw/pitch are RADIANS; `pose_*` = metres, `world_fill`/`scan_region`/`get_contents` = voxels.
27. `:265-276` material questions use `world_get_contents`; `has_contents` is now per-voxel and trustworthy (journal/0101).
28. `:277-279` cut a bench, not a pit; screenshot names are bare lowercase slugs.
29. `:280-284` dc-client exit codes are honest (0/70/71/101).
30. `:287-294` the moment a spot is a REFERENCE, record its exact pose.
31. `:295-313` pick the control that can see the question: LIT for shape, `--fullbright` for material, `--edges` for legibility (faded to zero past 1.4 km); the sun is fixed at 0.35.
32. `:317-338` a closed system cannot detect its own scale error; measure against the literature at least once; a constant fitted until it "looks right" is not evidence.
33. `:340-352` **EXISTENCE IS NOT STANDING** (user): unratified bootstrap content has no standing at any magnitude; the test is *would we build it today, in this shape?*
34. `:353-358` no evo/socia/civ modelling exists even at design stage.
35. `:359-363` **the status is ON HOLD, not NEVER** (user, 2026-07-28).
36. `:364-379` the doctrine governs unratified CONTENT, never RECORDED AMBITION; `things-that-will-happen.md` is entirely ambition and must not be swept; the tell is *does it RUN, or does it PROMISE?*
37. `:380-383` where a doc asserted a live pipeline stage, the ruled fix is **mark ON HOLD, not strike**; `worldgen.md`'s banner is the template.
38. `:384-393` the doctrine does **not** reach `docs/design/ecology.md`.
39. `:395-410` **a user-originated design may not be superseded by an implementation slice** (corrections #65); mark `⚠ CONTESTS` and stop.
40. `:412-420` runtime perf is first-class and under-fought; gen time is free, runtime is sacred; content is never cheapened for frames.
41. `:422-429` a summary is not an authority; the disappearing-consumer test.
42. `:430-431` headless crates never depend on rendering/OS; dc-client is the only GPU/OS crate.
43. `:432` wire types never use `skip_serializing_if`.
44. `:433-434` all entropy from caller-owned seeds; no wall clock.
45. `:435-440` commit trailer names the model that did the work.

### `docs/design/north-star.md`
46. `:3-9` status: ratified direction (user, 2026-07-23); a destination pursued evolutionarily, not a spec to build wholesale.
47. `:20-25` one-sentence shape: core = cell storage + pass-runner + field-solver **primitives** + stable API; everything else authored uniformly.
48. `:27-42` "native field-solvers are core" **RETIRED 2026-07-23**; primitives are core, every pass that calls them is content; a native backend is compute shape, never a permission tier.
49. `:44-54` native, not interpreted — the medium is compiled Rust, the value is the shape.
50. `:58-67` 🔖 OPEN EDGE on the core/plugin boundary; the plug-and-play mechanism is **parent inheritance**, so parent materials are a product surface.
51. `:69-87` core column: cell storage + cell API · pass-runner · field-solver primitives · event ledger · world API.
52. `:88-95` content column: material definitions · declarative material-transform passes · world/epoch config.
53. `:96-124` **the refinement tier RESOLVED TO (c), user, 2026-07-28**: engine owns primitives (field + refinement kernels) + the runner; plugins declare fields, passes, ORDER and RATE, materials, refinement. **The list is explicitly NOT exhaustive.** The mechanism is still open.
54. `:130-135` the refinement tier was arrived at by default; `collapse.rs` is 2,415 lines with ~zero pass-shaped declaration.
55. `:146-150` the engine hardcodes the **vocabulary of expression**; a mod cannot add a new KIND of visible structure.
56. `:152-156` two nulls separated: the facies gradient is a **magnitude** problem, the missing channel is a **path** problem.
57. `:170-180` (c)'s two fits: the pure-fn constraint is already the sandbox contract; refinement operators run per-cell or per-chunk, never per-voxel.
58. `:184-200` a material = property sheet + behavior slots + parent pointer + slug; packs add classes, not only members.
59. `:203-210` a pass declares `{reads, writes}`, cadence, epoch, `run(ctx)`; **the topo-sort half is SUPERSEDED 2026-07-26 — order is authored, `{reads, writes}` are the validator**.
60. `:212-219` two pass shapes, both content: cellular and field.
61. `:221-226` the `ctx` is a capability, not a god-object; enforced by the compiler.
62. `:230-232` § Refinement sharpens the boundary and **governs where the two disagree**.
63. `:234-240` the cut is machine-vs-content, not compute-shape; every pass is content incl. tectonics and erosion.
64. `:252-260` deeptime compiles, the present executes: two runtimes over one set of declarations.
65. `:262-269` deferred and named: actors/agents are a second substrate and a third pass shape; a cellular shim spawner covers mobs today.
66. `:273-278` the two clocks: gen-time free, runtime sacred.
67. `:282-287` behavior is code, tuning is data; `DeepConfig` is the embryo.
68. `:291-306` modding never requires open-sourcing the engine; **everything is built on the SDK route mods take** (user 2026-07-23); the SDK must carry the entire default content set.
69. `:317-324` **the crossing constraint**: plain data + opaque handles across the seam; rich Rust on both sides.
70. `:333-336` mods are versioned artifacts declaring SDK version and reads/writes.
71. `:338-345` validation by construction, at compile time (native) or load time (WASM/data); generalizes `build_checked`.
72. `:349-365` the embryo table — ~70 % embryonic in-tree; the seam-first march is the path.
73. `:368-383` three not-yet-proven items; #3 (the ABI spike) **amended NOT GATING (user, 2026-07-24)**.
74. `:385-403` compliance loop: read-first, plan time, work time, review time, carve-outs through the user.
75. `:407-417` **Deviation 1 (user, 2026-07-24):** trusted/untrusted split + ABI/WASM spike DEFERRED, not gating.
76. `:419-433` **Deviation 2 (user, 2026-07-24, emphatic):** NO capability tiering; assume mods can do anything, including author a field pass.

### `docs/ARCHITECTURE.md`
77. `:3-4` status: pre-spike sketch, everything a working hypothesis until its spike lands. *(contested — finding #4)*
78. `:37-44` the load-bearing bet: one API, many consumers; everything below dc-client is headless and deterministic.
79. `:47-54` Windows/macOS/Linux first-class; consoles not precluded.
80. `:57-61` distant terrain is first-class; chunk storage is LOD-aware from v1.
81. `:62-71` **DECIDED by S4:** clustered forward (Forward+), deferred rejected; pack overridability via named stages.
82. `:74-90` cubic chunks: 3D lattice; skylight is non-local; LOD is a 3D lattice; floating origin from day one; depth is a worldgen axis.
83. `:92-107` **DECIDED 2026-07-18:** chunks stay 32³; the tall-chunk instinct is policy over cubes; sim tiers are 3D volumes.
84. `:111-118` **DECIDED 2026-07-18 (S1):** player height = 2 voxels, voxel = 0.9 m; 2.74 B/m³, ~31 tris/m². *(agrees with `docs/spikes/S1-results.md:39`)*
85. `:121-127` characters use swept-AABB owned by us; Rapier only in bubbles around dynamic bodies.
86. `:131-154` three sim tiers (Full/Coarse/Statistical); committed fact / fluid state / collapse; collapse bounded to depth N. *(contested as live — finding #3)*
87. `:158-166` worldgen is hierarchical, lazy, bounded; worldgen history and live far-simulation are one system.
88. `:168-196` **RATIFIED 2026-07-19:** one world-answer surface; caches never fall back to a generator they do not own; the fallback method is deleted, pinned by a tripwire test.
89. `:199-204` content model: items/blocks/biomes/blueprints/models/animations are data-driven registry entries, Blockbench-shaped.
90. `:206-219` **DECIDED 2026-07-21 (user):** build every system API-first, in remove-and-plugin shape; the honest-placeholder rule (accepted as-is and marked temporary, never patched toward looking finished).
91. `:221-236` performance is standing doctrine: recycling/pooling, mipmaps, perf-first memory in all domains.
92. `:238-244` **DECIDED 2026-07-21 (user):** contents are authoritative, the block is derived; `block == classify(contents)`.
93. `:246-263` scope as built: the invariant holds for voxels with **non-empty contents**; the exception list is enumerated and pinned by test (ruin posts left it 2026-07-28 by deletion).
94. `:276-281` a column's fill is an ordered list of spans; today's single-height column is the degenerate case.
95. `:285-303` consequences: `Block` does not grow form-aware variants; solidity moves onto an occupancy threshold; contents promoted to authoritative; one occupancy answer.
96. `:305-309` fractions come only from the ledger, never cosmetic noise.
97. `:311-348` **DECIDED 2026-07-21 (user):** a summary is not an authority; the disappearing-consumer test; four instances found in one audit.
98. `:355-379` **DECIDED 2026-07-22 (user):** the generation-affecting content set is frozen at world creation; supersedes geology.md's seed-stability line; no post-hoc history rewrites.
99. `:383-395` the partition is "does it participate in generation?"; freezing removes the re-derivation failure mode.
100. `:397-403` the content set must be recorded in the save and validated on load; a missing generation-affecting plugin is a hard refusal. *(whether anything records it is "unverified — under audit")*
101. `:405-415` **AMENDED 2026-07-22 (user):** plugins are MARKED, not banned; history is written forward, never backward.
102. `:417-423` provider seams: a named provider with an explicit contract; 34 seams inventoried.
103. `:425-441` **DECIDED (user):** `Providers` = `Option<fn>` slots, resolved once at world build, `None` means identity; absence is structural (corrections #32).
104. `:442-447` effective reads = declared ∪ providers' reads. *(justification contested — finding #9)*
105. `:448-454` conflicts warn, they do not fail closed; the user is asked at world creation. *(contrast case contested — finding #8)*
106. `:456-465` **DECIDED 2026-07-22 (user):** the resolved provider table is part of world identity.
107. `:466-473` **DECIDED 2026-07-22 (user):** headless creation resolves last-in-order and logs loudly.
108. `:475-482` PROPOSED, NOT RATIFIED: providers produce planes; a purity contract.
109. `:484-487` the first conversion slice builds the struct and identity defaults only.
110. `:491-502` **DECIDED 2026-07-26 (user):** the engine is plugin-agnostic, fullstop; the product is a plugin-based world *generator*; the default pack is one pack.
111. `:506-511` what it indicts: `DeepAxis` is a closed enum naming our own pipeline stages — the violation.
112. `:515-531` derive-and-reject vs **author-and-validate (DECIDED)**; author-and-validate is strictly more expressive; an engine cannot derive a third party's intended order.
113. `:536-542` `material-behavior.md` §5's "order falls out of `{reads, writes}`" is **superseded**; filed as corrections #65.
114. `:546-551` engine owns: the pass-graph kernel, the epoch clock and `dt`, solver primitives, cell/record storage, refinement primitives. Content owns: passes, order, cadence, **the fields themselves**, materials, behaviours.
115. `:553-556` resource ids are opaque and open; a pack declares its own.
116. `:558-571` RATE was ratified 2026-07-24 and never built (`dt` pinned to 1.0); the erosion period-2 mode (ACF(1) −0.87/−0.91 vs +0.38) makes RATE the architecture's first real consumer.

### `docs/spines.md`
117. `:3-5` created 2026-07-22 after the corpus outran the assistant fourteen times.
118. `:7-32` last `spine-audit` sweep: **2026-07-25 (post-FLOW)**; both 2026-07-24 findings applied; one § 3 row added (`DeepField::chapters`); all four in-code `stubs.md #19` markers are stale.
119. `:62-72` the file-role table: this file answers what SHAPE things take and what nothing calls.
120. `:76-85` work is justified against named shapes; deviation is loud, user-ratified, recorded in § 4; update in the same commit.
121. `:91-110` **S-1** bounded derivation with a synthesized coarse frontier; every system declares its halo. The S2 collapse instance is cited as a *shape*, not code on a path (its last production caller went 2026-07-28).
122. `:113-144` **S-2** committed facts vs fluid state; *store only what the derivation cannot predict*; the ledger CSR result (98.8 % empty inner `Vec`s, 89 % of heap in headers); **a per-cell container is a gen-time shape**.
123. `:146-178` S-2's storage corollary: flat exact-sized payload + sparse index, two variants (dense row-pointer vs keyed rows); CSR index floor 0.056× of residency; 311.02 → 179.12 MiB.
124. `:180-265` **S-3** a summary is derived from the authority, never beside it; seven compliance instances; a summary can wear an authority's clothes by **resolution** as well as content.
125. `:267-336` **S-4** coarse cause, fine expression; threshold late on interpolated causes, or dither membership; `CoarseField<T>` extracted (journal/0075); **live violation:** `regolith_at_voxel` samples NEAREST beside a bilinear `surface_at_voxel`.
126. `:338-367` **S-5** seams with identity defaults; the fallback must be an identity, never a constant; 34 inventoried, **5 converted**; seam the quantity, not the verdict.
127. `:369-517` **S-6** declared relations, never incidental order; the deep-time runner, the two FLOW passes, the `reads_prev` anti-dependency mechanism; *"incidental order is forbidden; declared order is fine."* *(contested — finding #1)*
128. `:519-529` **S-7** distribution-first quantization; sieve loss 75.8 % → 0.2 %; agreement tests over expressed data are statistical.
129. `:531-572` **S-8** one quantity, many regimes; free/bound water, substance/form, one load several movers; transitions are declared and compile-enforced (`EdgeId::declared`).
130. `:574-673` **S-9** derivable base + sparse committed facts + fallback query, up to observation-collapse; the consistency law; the `Fact` narrowed 16 B → 8 B losing no axis; **live violation:** far-field cold-synthesize vs warm-reduce disagree.
131. `:679-711` **A-1** a stand-in becomes the definition; four shipped instances; `reference_material`'s blast radius widened into hydrology.
132. `:713-908` **A-2** a justification outlives its premise, with five variants named (expired · never-true · symmetry-assembled · a test's unstated premise · a constant anchored to an unchecked constant).
133. `:795-812` **corrections #67 (2026-07-28):** the `exhum`/`t_crust` row is RESTORED as an instance — a quotation without a revision cannot distinguish a paraphrase from a stale read; 808 `file:line` citations in the corpus.
134. `:910-955` **A-3** a test green for a reason unrelated to what it asserts; the examples-never-run form and the **fixture** form (a helper named `production_field` that was not production).
135. `:957-1034` **A-4** built machinery with no consumer and no index; discharge-by-porting and discharge-by-extraction recorded as the good outcomes.
136. `:1036-1052` **A-5** locality of cause ≠ locality of effect; **A-6** measuring what cannot change a decision.
137. `:1061-1076` **a § 3 row leaves THREE ways** — CONSUMED · DELETED · HELD AS A CANDIDATE (added 2026-07-28 by the user's S2 ruling); a candidate row must say so and name what unblocks judgement.
138. `:1078-1093` the § 3 index: 13 live rows, incl. the whole S2 statistical tier (**user-RULED 2026-07-28: KEEP, does not want a consumer found**).
139. `:1095-1116` departed rows: the S7 pregen history handoff (deleted 2026-07-28) and `MixtureDownsampleRule` (consumed 2026-07-22).
140. `:1120-1126` § 4: a deviation ships only after a loud plea, main-session discussion, user ratification, and a dated entry.
141. `:1128-1159` carve-out 1: B1's class draw uses the coherent bilinear source; **amendment NEEDS RATIFICATION** — the bias sign is the opposite of "toward 50/50".
142. `:1163-1174` § 5 the `JUSTIFIED-BY` convention, greppable by `spine-audit`. *(refuted at `:1176` — finding #7)*
143. `:1187-1249` **A-7** naming a content identity inside a process — **not a ratifiable carve-out**; the diagnostic is *what property are you reaching for?*
144. `:1253-1283` § 6 the audits index; any future sweep lands here in the same commit. *(contested — finding #11)*

### `docs/API.md`
145. `:3-7` status: pre-S5 design; S5's exit criterion is revising this doc. *(contested — finding #5)*
146. `:10-29` five principles: one door · commands are data · determinism is the payoff · capability-scoped deny-by-default · reflectable.
147. `:33-56` the `CommandEnvelope`/`CommandReceipt` shape; total ordering with the S5 consumer-id tie-break; async submission **normative as of S5**; txns all-or-nothing within one tick; queries run against the last completed tick.
148. `:60-73` seven domains (world/entity/inventory/registry/sim/schedule/events); content packs are recorded `registry` batches.
149. `:75-85` **DECIDED 2026-07-19 (user):** pack-degradation — problems inform and degrade, never fail the game; malformed defs still reject at define time.
150. `:90-112` the content-class registry: `define_content_class` + `define_class_member`, validated at define time; namespace rule; the vanilla geology pack is generated from the typed set.
151. `:116-134` per-class contracts: clastic/placer carry weather params, igneous/accessory carry none — a granite structurally cannot be handed the weather.
152. `:137-154` class-satisfiability in two layers (define-time `validate_pack`, world-build `check_class_satisfiability`); the define-time layer **rides as-built, NOT ratified as final doctrine**.
153. `:156-160` accessory inclusions ride the host rock's pore slots.
154. `:166-206` the bodies registry: `define_anim_clip` + `define_body_plan`; the verb→slot contract checked at define time; v0 verbs `idle`/`walk`/`jump`; clips standalone **RATIFIED**; the client-side firewall.
155. `:211-226` `sim.observe` (diegetic, commits facts) vs `sim.inspect` (out-of-band, commits nothing).
156. `:230-254` **Decided 2026-07-18:** AI-driven characters are first-class; Character is a primitive, Controller a binding; two MCP surfaces over one API.
157. `:256-260` **v0 DECIDED 2026-07-19:** disconnect = freeze.
158. `:262-273` **RATIFIED 2026-07-19:** attach placement guards embedding; `surface: true` snaps to the true voxel surface.
159. `:277-290` posture as a fourth controller verb; standing up is guarded; replay bit-identity extends to posture.
160. `:296-319` the capability vocabulary; tokens scoped and attenuable; **DECIDED 2026-07-19 (user)** dev builds may `registry.define(dc:*)`, shipped builds carry no such grant.
161. `:323-326` wire formats per boundary (typed enums · postcard · JSON · the envelope over a transport).
162. `:330-333` versioning: pre-1.0 the whole surface is v0; from 1.0 additive-only per domain.
163. `:337-345` decisions 1–3 CONFIRMED 2026-07-18 (id naming · player input as commands · two MCP servers).
164. `:347-351` **DECIDED by S5 measurement:** receipts carry a summary plus a capped effect sample (N≈8).
165. `:353-366` **DECIDED 2026-07-20 (user):** `client_player_pose_set` retires into the door; `client_screenshot`/`pose_get` stay outside by design.
166. `:368-375` **DECIDED 2026-07-20 (user):** an additive per-`CommandSpec` completion source.
167. `:377-388` **DECIDED 2026-07-20 (user):** registry self-consistency moves from tests to the compiler; wire-visible schemas keep their shape.
168. `:390-416` v0 implementation notes adopted from S5: no `skip_serializing_if`; grant **handles** not by-value tokens; `events/poll` is a channel op; the open `Payload` path; the wasm32 plugin ABI.

### `docs/SPIKES.md`
169. `:1-6` spikes are ordered by risk; each has an exit criterion recorded in `ARCHITECTURE.md`; throwaway code is allowed, conclusions are not.
170. `:8-20` **S1** voxel-scale walking skeleton — exit: ratio chosen, budget table, bevy pinned.
171. `:22-35` **S2** constraint-ledger prototype — exit: ledger schema + collapse algorithm + go/no-go on one system for live sim and deep-time.
172. `:37-49` **S3** LOD-aware chunk storage — exit: chunk format v1 spec, measured numbers, far-mesh renders.
173. `:51-61` **S4** shader hook contract — exit: pipeline decision + hook surface v0 + a loadable pack.
174. `:63-71` **S5** plugin host + MCP over one surface — exit: dc-api conventions doc, demo plugin, MCP transcript.
175. `:73-79` **S6** physics bubble — exit: collider-bubble note + perf numbers.
176. `:81-98` **S7** worldgen coarse pregen + lazy pyramid — exit: level/resolution spec, pregen-time table, demonstrated ledger handoff.
177. `:100-112` **S8** material volume model storage — exit: go/no-go on free-form mixtures, measured bytes/m³, sidecar schema v1.
178. `:114-120` deliberately not spiked yet: multiplayer (post-S5), editor UX, console ports.
179. `:122-125` suggested order: S1+S2, then S3, S5, S4, S6, S7.
*(Claims 169–179 are all stated as pending; see finding #6.)*

---

## What I could not attribute or resolve

- **Finding #12** (`wasmtime` deferred vs `wasmtime` shipped) — I cannot tell from the text
  which seam `north-star.md` Deviations #1 means. Explicitly unresolved.
- **`ARCHITECTURE.md:400-403`** — *"Whether anything records the content set today is
  unverified — under audit."* Six days old at the watermark; I did not check the code
  (out of remit) and found no doc in this slice that discharges it. Not counted as a
  finding: it is honestly marked open, which is correct practice.
- **`spines.md:1128-1159`** carries a **NEEDS RATIFICATION** amendment (the B1 bias sign)
  from 2026-07-22, still undischarged at `f652b60`. Correctly marked, so not a
  contradiction — but it is a six-day-old open ratification sitting in a read-first doc,
  and stage 2 may want it beside whatever else is pending user ruling.
- I did **not** verify any `spines.md` § 3 row against `crates/` — that is `spine-audit`'s
  remit and would have been the wrong instrument.
