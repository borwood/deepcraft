# S4 — stubs / corpus-knowledge / ideas / visuals / PIPELINE

> ## 📋 DISPOSITION — applied 2026-07-29.
>
> | # | state |
> |---|---|
> | **F1** CLAUDE.md counts FOUR ecology-heir stubs; `stubs.md` has TWO | **✅ ALREADY FIXED before this pass** — `CLAUDE.md` now reads **"Two"** and records *why* the count went stale, in the argument it supports. (`CLAUDE.md` is out of this agent's scope in any case.) |
> | **F2** `stubs.md` header says FOUR provider slots, there are FIVE | **✅ ALREADY FIXED before this pass** — the header now reads *"**Five exist** — `outcrop_at`, `wave_energy`, `parent_p`, `depth_to_water` and `paleo_temperature`"*, and the *"four conversions is not enough"* clause was **withdrawn entirely by the user** (corrections #69 — there was never a registry). |
> | **F3** `stubs.md` #27 asserts the epoch clock IS the register; #29 withdrew it | **✅ APPLIED** — #27(b) struck in place, naming #29, corrections #63, and the three measurements. *Anyone opening #27 was reading a live instruction to re-anchor a clock that is not the lever.* |
> | **F4** `stubs.md` #4's heir names an address #14 deleted | **✅ APPLIED** — address corrected to the `dc:field/temperature` geotherm **field pass**; the deletion of the `burial_temp_c` slot and its mechanism recorded. |
> | **F5** `heir` 654 vs 651; 192 vs 193 files | **⚠ ESCALATED** — the finding itself says *"better-supported side: undetermined"* and that neither figure carries its pathspec. **Settling it needs a fresh census**, which is a new measurement, not an application. |
> | **F6** the sole "void heir" is void on grounds the user narrowed to ON HOLD | **✅ APPLIED** — both corpus-knowledge sites annotated with the 2026-07-28 user ruling. **Also recorded the consequence the finding drew:** the kind may have zero instances, while F4's *re-homed address* kind has at least one and is not in the inventory. Rows annotated, not rewritten. |
> | **F7** § 4 is written, and the file says twice § 4 must stay empty | **✅ APPLIED AS A MARKER ONLY** — the trailing duplicate `## 4. Theory / EMPTY BY DESIGN` now says it was overtaken, names the live § 4, and confirms the gate it cites is **still unmet**. **Which § 4 governs is explicitly left to main session.** |
> | **F8** `visuals.md` "do not re-derive" priors describe a superseded far field | **✅ APPLIED** — struck with the `stubs.md` #15 / FF2b quotation; the honest caveat (true when written of FF2a's hollow sheet) preserved. |
> | **F9** a ratified item sits in the "NOT decisions" doc unmarked | **✅ APPLIED** — status marker added, using the convention the file already uses twice. The sketch text is kept as the origin record. |
> | *nulls* | The declared nulls (PIPELINE × visuals; the civ/socia sketches; epoch arithmetic) were **not** revisited — a declared null with its reasoning is a result. |

Baseline `doc-topology` sweep, slice 4 of 9. **Watermark commit `f652b60`; every quotation
below was read at `f652b60`** (per the skill's revision rule, corrections #67).

**Read IN FULL:** `docs/design/stubs.md` (1,217) · `docs/design/corpus-knowledge-evidence.md`
(1,059) · `docs/design/corpus-knowledge-notebook.md` (615) · `docs/design/ideas.md` (596) ·
`docs/design/visuals.md` (513) · `docs/rendering/PIPELINE.md` (288). **Also read in full for
cross-checking:** `.claude/skills/doc-topology/SKILL.md`, `CLAUDE.md`. **= 4,288 lines of slice
+ 622 of method.**

**Read-only sweep. Nothing here is resolved** — several of these are user calls, which is
why they survived. Recommendations are labelled as such.

---

## 1. Findings — contradiction pairs, ranked by blast radius

### F1 · `CLAUDE.md` counts FOUR live ecology-heir stub entries; `stubs.md` has TWO — and one of the four was resolved by deletion the same day
**Shape 6 (a summary that outran its source) + shape 1.** Highest blast radius in the slice.

| side | text |
|---|---|
| `CLAUDE.md:388-389` | *"Four live `stubs.md` entries name ecology as their heir."* — the load-bearing evidence sentence in the ⚠ that **withdraws** *existence-is-not-standing* from `docs/design/ecology.md` |
| `docs/design/stubs.md:57,59` (entry **#1**) | that entry is `### 1. ruin-posts — **RESOLVED BY DELETION 2026-07-28 (journal/0121); no heir was ever built and none is owed**` (`:30`) — **not live**, and `:64-69` explicitly says it *"is **NOT** evidence that `docs/design/ecology.md` owes anything"* |
| `docs/design/stubs.md:123-124, :133` (**#2**) and `:214-215` (**#7**) | the only **live** entries naming ecology as heir — **two**, not four |

Every `ecolog` occurrence in `stubs.md` at `f652b60` (`grep -n -i ecolog docs/design/stubs.md`,
9 hits) falls in entries **#1, #2, #7**. So the count is wrong two ways: three entries, of which
one is resolved.

- **Provenance:** the CLAUDE.md ⚠ block is *about* a user directive but the count sentence is
  assistant-authored; `stubs.md` is the registry and is the source the count paraphrases.
- **Better-supported side:** `stubs.md`.
- **Blast:** `CLAUDE.md` loads into every session and every agent brief, and this sentence is
  the entire recorded defence of a ratified user design against a doctrine that would sweep it.
  A number that is demonstrably wrong weakens the defence it exists to make.
- **Recommendation (labelled):** the *conclusion* is not in doubt — ecology.md is ratified and
  two live entries do name it. Only the count needs main-session repair, and it should be
  re-derived rather than decremented, because `#1`'s deletion is what moved it.

---

### F2 · `stubs.md`'s header enumerates FOUR provider slots and there are FIVE — and the "four conversions" premise is quoted as binding doctrine in three other places
**Shape 5 (a justification whose constraint expired) + T6 (an unchecked enumeration).**

| side | text |
|---|---|
| `docs/design/stubs.md:20-24` | *"**Four exist** (stubs 7b, 8 and 13, and the layer-cake sibling gap)… **The general registry is deliberately unbuilt — four conversions is not enough to design one from.**"* |
| `docs/design/stubs.md:196-201` (**#6**) | `### 6. paleo-temperature-is-present-day-latitude — **NOW A PROVIDER SLOT, 2026-07-23 (journal/0078)**` … *"**Now a slot:** `providers::Providers::paleo_temperature`"* — **a fifth, landed four days after the header sentence, and absent from its list** |

Confirmed by `grep -n "Providers::" docs/design/stubs.md` at `f652b60`: live slots are
`paleo_temperature` (#6), `depth_to_water` (#7b), `parent_p` (#8), `wave_energy` (#13),
`outcrop_shares` (layer-cake gap) — **five**; `burial_temp_c` was a sixth and was **removed**
(#14, `:367-377`).

**Why the blast radius is far larger than a miscount:** the "four conversions is not enough"
clause is currently the ratified doctrine **governing the live docs-ops design pass**, quoted
verbatim in three read-first surfaces:

- `.claude/skills/doc-topology/SKILL.md:43` — *"**`stubs.md:22` forbids designing the general
  mechanism before several real conversions have taught the shape.** That binds this thread too."*
- `docs/design/corpus-knowledge-notebook.md:240-244` (§ 4.0b, *"what the project's own doctrine
  says about how to proceed"*) and `:571`.
- `docs/design/corpus-knowledge-evidence.md:939-948` — *"**THE MOST IMPORTANT SENTENCE IN THE
  FILE**, for this notebook."*

- **Better-supported side:** § 6 (dated, journal-cited, code-named).
- **Note the irony, and it is evidence, not decoration:** `evidence.md:403-412` cites
  `stubs.md` #29's incomplete phase list as the canonical T6 instance — *"the corpus's most
  structured artifact held an enumeration that was both wrong and incomplete."* **This is a
  second T6 instance in the same file, in its header, unfound until now.**
- **Recommendation (labelled):** main session. Whether *five* changes the "not enough to design
  one from" judgement is a user call, not a sweeper's; the enumeration is the part that is
  simply false.

---

### F3 · `stubs.md` #27 still asserts the epoch clock IS the register; #29 withdrew exactly that, 130 lines below, with no marker at #27
**Shape 1 + the one-directional edge.** Both halves in one file, both live.

| side | text |
|---|---|
| `docs/design/stubs.md:963-968` (**#27**, heir (b)) | *"Note this is the same owed item as `earth-processes.md` § 3e's 'calibrate iteration↔Myr against a real orogen' seen from the other end: **the cap is `cell_m / myr_per_epoch`, so it is a statement about the register**, and re-anchoring the clock is a **user-owned** call."* — unstruck, no marker |
| `docs/design/stubs.md:1099-1102` (**#29**) | *"~~`(b) is the same register as stubs #27's heir (b)`~~ — **withdrawn** (corrections #63): `myr_per_epoch` does not exist as a knob and **the epoch length is measured *not* to be the register**. #27 and #29 share the **limiter**, a stronger link than the clock was."* |

#29's own measurement is the falsifier and it is stated twice more in the same entry —
`:1038-1044` *"**(b) IT IS NOT A TIME-STEP LIMIT**"* and `:1081-1084` *"the limiter is deaf to
the step… makes the transfer a function of **inventory, not of `rate × dt`**, so refining `dt`
cannot reduce a transfer `dt` does not set."* That is precisely the claim #27(b) needs in order
for the one-cell cap to be *"a statement about the register."*

- **Better-supported side:** #29 (measured; corrections #63 filed).
- **Provenance:** both assistant-originated; #27(b) additionally routes a **user-owned** call
  (re-anchoring the clock) through a premise that has been withdrawn.
- **Blast:** #27 and #24 (`:792-796`) both name the iteration↔Myr calibration as the heir of
  the erosion work, which is the **most valuable erosion work on the board**. A slice that
  opens #27 reads a live instruction to go re-anchor a clock that #29 measured is not the lever.
- **This is the corpus's own named failure mode caught in its own registry:** the withdrawal
  reached the entry that made the claim and never reached the entry it was a claim *about*
  (`corpus-knowledge-notebook.md:393-402`, *"a one-directional pointer is not a pointer"*).

---

### F4 · `stubs.md` #4's heir names an arrival address that #14 removed from existence
**Shape 1, in-file, 190 lines apart.**

| side | text |
|---|---|
| `docs/design/stubs.md:180-182` (**#4**) | *"**Heir:** a metamorphism pass reading the P/T path into grade classes… — with **a named arrival address, the `providers::burial_temp_c` geotherm (journal/0067)**."* |
| `docs/design/stubs.md:370-373` (**#14**) | *"**The `burial_temp_c` provider slot is removed entirely** — the honest finding is that a real `T(depth)` is a **field**, not a value a stateless `fn(unit)` slot could hold, so it **left** the provider set rather than fitting a socket it never belonged in."* |

#4 was amended on **2026-07-28** (`:173-179`, corrections #67) — so the entry was open in an
editor after the address had been gone for four days, and the heir sentence was not revisited.

- **Better-supported side:** #14 (dated, journal/0093-cited, states the mechanism).
- **Blast:** modest in code terms (#4's blast is *"zero today"*), **but structurally it is a
  second instance of the exact failure mode `corpus-knowledge-notebook.md:383` claims has
  exactly ONE known instance** (see F6). An heir pointing at a deleted address is a dangling
  heir whether the successor is void or merely re-homed.

---

### F5 · `heir` is 654 in one paragraph and 651 in four others — the same census, the same day, the same denominator
**Shape 4 (one number, several values), across both corpus-knowledge docs.**

| side | text |
|---|---|
| `corpus-knowledge-evidence.md:294` | *"`heir` **654** in 129 files"* (§ 3.5, *measured 2026-07-28, tracked files only*) |
| `corpus-knowledge-notebook.md:356` | *"The corpus has **654** `heir` edges for stand-ins"* |
| — against — | |
| `corpus-knowledge-evidence.md:558`, `:866`, `:897` | **651** (129 files), three times |
| `corpus-knowledge-notebook.md:124`, `:284`, `:294`, `:378`, `:478` | **651**, five times |

Same measurement, same file count (129), no re-measure recorded between them.

**And the corpus-size row disagrees with itself the same way:**

| side | text |
|---|---|
| `corpus-knowledge-notebook.md:57,63` | *"Grep over all **192** `.md` files…"* · *"**Corpus size, measured 2026-07-28:** **192** `.md` files, **58,340 lines**"* |
| `corpus-knowledge-evidence.md:274` | *"**What the corpus actually cites** (**193** `.md` files, **58,608 lines**)"* |
| `.claude/skills/doc-topology/SKILL.md:14` | *"Measured 2026-07-28 over all 67 `corrections.md` entries, 776 commits and **193** `.md` files"* |

- **Better-supported side:** undetermined — this is *"quote the measure, name the world"*
  applied to the analysis's own numbers, and neither figure carries the pathspec it was
  measured under. The **651** form has 8 sites against 2 and is the one the SKILL banner
  propagates; the **654/193/58,608** figures are all in the *later*-written evidence file.
- **Blast:** these figures are the entire quantitative basis of *"the corpus already authors
  the graph; nothing reads it"* (T5), which is the finding the docs-ops design pass turns on,
  and `SKILL.md:32-34` republishes them to every future reader of that skill. The **direction**
  is not in doubt at any of these values; the doc's own rule (`notebook:182-186`) already says
  *"the percentages are not"* robust. **This is that rule's own numbers failing it.**
- *(Not filed as a correction candidate: it is a pair to reconcile, not a falsified claim —
  same disposition `evidence.md:323` gave the date-drift instance.)*

---

### F6 · The one recorded "void heir" is recorded as void on grounds the user has since narrowed to ON HOLD
**Shape 1, and it crosses `CLAUDE.md`.**

| side | text |
|---|---|
| `corpus-knowledge-notebook.md:383` (§ 4.1b, the obligation-type inventory) | *"**void heir** — an obligation whose successor was **ruled out of existence** \| nothing; found by hand \| `stubs.md` #1's heir was *"the social sim + ecology"*, and **the user ruled those NOTHING 2026-07-26**"* |
| `corpus-knowledge-evidence.md:933-937` | *"**`#1 ruin-posts`' heir is now void.**… **A dangling heir is a new failure mode this inventory has no state for**"* |
| `CLAUDE.md:359-363` (**user, 2026-07-28**) | *"**⚠ THE STATUS IS *ON HOLD*, NOT *NEVER* — and the difference is load-bearing**… *'History is coming, eventually… we do want these systems **eventually**: they are effectively **on hold**.'* **Read *"they are NOTHING"* as a statement about what exists, never about what is wanted.**"* |

- **Provenance decides this one:** `CLAUDE.md:359-363` is a **direct user ruling**, dated one
  day after the notebook rows; the notebook and evidence rows are assistant readings of an
  earlier user quote. **User-originated = data.**
- **Better-supported side:** `CLAUDE.md`.
- **Blast:** the "void heir" row is one of the seven obligation kinds in the inventory that
  the reciprocity proposal (§ 5) is being argued from, and it is the *sole* instance of its
  kind. If the heir is **deferred** rather than **void**, the kind may have zero instances —
  or, per **F4**, its real instances may be the re-homed-address kind instead.
- **Note in the corpus's favour:** `stubs.md:64-69` already carries the correct reading and
  says so explicitly. The stale reading survives only in the two corpus-knowledge docs.

---

### F7 · The notebook's § 4 is written, and the notebook says twice that § 4 must stay empty
**Shape 1, wholly inside one file — and it is a duplicate heading, so no reader meets both.**

| side | text |
|---|---|
| `corpus-knowledge-notebook.md:15-18` | *"**§ 4 is theory, and it stays EMPTY until the reading is done.** Any claim in § 4 that cannot name a `file:line` in § 3 is inadmissible."* |
| `corpus-knowledge-notebook.md:613-615` | `## 4. Theory` — *"**EMPTY BY DESIGN.** Do not write here until § 3 coverage is declared complete."* |
| `corpus-knowledge-notebook.md:104-462` | `## 4. Theory` — *"**Status: PROVISIONAL after pass 1.**"* followed by ~360 lines of theory (§§ 4.0–4.3b), with § 5 (`:466`) sitting **above** § 4.4 (`:592`) |

Two headings named `## 4. Theory` in one file, 509 lines apart, in contradictory states; and
`evidence.md:950-989` shows § 3 coverage is explicitly **not** complete (*"Still not opened:
`north-star.md` bodies… `PIPELINE.md`, all `docs/spikes/*`…"*), which is the condition
`:615` names as the gate.

- **Better-supported side:** neither is *false*; the governing instruction (`:15-18`, restating
  the **user's** own constraint at `:6-13`) was overtaken by the work and never retracted —
  T1′ in the document that measured T1′.
- **Blast:** medium and rising. A cold reader arriving at the bottom of a 615-line notebook is
  told the theory is empty; the anchor ordering (§ 5 before § 4.4) means the file no longer
  reads in its own declared order. This is the live docs-ops thread's primary artifact.

---

### F8 · `visuals.md` tells a future LOD slice "do not re-derive" a far-field description that FF2a/FF2b superseded
**Shape 1, and the passage is explicitly marked as authoritative priors.**

| side | text |
|---|---|
| `docs/design/visuals.md:407, :428-430` | `### PRIORS THE SWEEP FOUND (do not re-derive)` … *"**What is missing is the renderer consuming the pyramid at distance bands**, not the pyramid. **Today's far field is a top-surface *heightfield of Blocks*** (journal/0022), coloured per block."* (section dated user 2026-07-21) |
| `docs/design/stubs.md:443-456` (**#15**, added 2026-07-22, journal/0070) | *"A synthesized far node (a node whose full-res children were never generated — most of the far field, forever) **fills each column solid** up to the FF2a floor-quantized surface, and every voxel of that fill carries the **surface** block… **in node form it is now data that *claims* the column**."* |
| `docs/design/visuals.md:87-94` (same file, DECIDED 2026-07-19) | *"**FF2a — voxelize the far field**… quantize the same coarse column summaries to voxel steps and **mesh stepped columns**"* — shipped 2026-07-19 (`ROADMAP.md:2460`, journal/0023) |

- **Better-supported side:** `stubs.md` #15 (later, code-sited, journal-cited).
- **Honest caveat:** the `:429` sentence was **true when written** of FF2a's hollow stepped
  sheet; FF2b node synthesis is what filled the columns. This is a genuine staleness instance —
  the 3–8 % band — not a born-false claim.
- **Blast:** medium. `visuals.md` § *Open, unratified* (`:508-512`) is the live spec for an
  unbuilt LOD-band slice, and its priors block is labelled *do not re-derive*. A slice taking
  it at its word plans against a far field that no longer exists in that shape.

---

### F9 · A sketch in the "NOT decisions" doc has been ratified doctrine for a week, and the sketch does not say so
**The inverse of `ideas.md`'s charter — flagged per the brief, not as a charter violation.**

| side | text |
|---|---|
| `docs/design/ideas.md:1-6` | *"# Idea inventory — **sketches, NOT decisions**… **deliberately not yet designed**… Do not treat any shape below as final."* |
| `docs/design/ideas.md:293-301` | `## Entering a world without generating one (user, re-raised 2026-07-20)` — *"**Optional ready-made worlds**, shipped pre-generated"* — carries no ratification marker |
| `CLAUDE.md:415-416` | *"**gen time is not a constraint** (**ready-made worlds are the sanctioned answer**)"* — stated as project doctrine |
| `docs/design/stubs.md:643-645` (#21) | *"**'ready-made worlds are the sanctioned answer'** means a ledger **will** be persisted"* — the sketch used as a load-bearing premise for a sequenced heir |

- **Provenance:** user-originated on **both** sides; no conflict of authority, only of status.
- **Better-supported side:** `CLAUDE.md` — the item is decided.
- **Blast:** low-to-medium, and asymmetric: the risk is not that someone builds it, it is that
  `stubs.md` #21's heir (the pager slice) rests on a premise its own source still labels
  *"not final."* Compare `ideas.md:510` and `:528`, which **do** carry status markers
  (*ratified direction 2026-07-23*, *RECONCILED → material-behavior.md §5*) — the convention
  exists in this file and simply was not applied here.

---

### Checked and NOT flagged (nulls, declared)
- **`ideas.md`'s civ / socia / history-sim sketches** (`:14-21`, `:71-100`, `:102-130`,
  `:194-221`) against CLAUDE.md § *Existence is not standing*. **Not a finding:** `CLAUDE.md:376-379`
  draws the tell explicitly — *"does it RUN, or does it PROMISE?"* — and these promise. The
  2026-07-28 ON HOLD ruling (`:359-363`) covers them.
- **`stubs.md` #1's original heir sentence** (`:56-60`) — a preserved-verbatim original inside a
  RESOLVED entry, with its own ⚠ at `:64-69`. Excluded by the skill's rule 5.
- **`evidence.md:23` "65 entries" vs `:738` "67 rows"** — corrected in place, explicitly, by the
  document itself. Excluded by rule 5.
- **`visuals.md:66-72` the struck "no darkness" lean** (corrections #13) — struck and dated,
  with the superseding DECIDED at `:161`. Excluded by rule 5.
- **`PIPELINE.md` against `visuals.md`** — read both in full for the material/lighting contract.
  **NULL: no contradiction found.** `PIPELINE.md:30` (*real darkness, no ambient floor*),
  `:192-211` (LabPBR triplet), `:11` (*the default pack IS that document*) all match
  `visuals.md:13`, `:18-23`, `:154`. `PIPELINE.md:59` (*sun shadow cascades — Bevy, off in v0*)
  is a v0 state, not a denial of `visuals.md:161-190`'s decided end state. The one gap is an
  **omission, not a pair**: `PIPELINE.md:221-225` describes splat inputs as *"up to a few
  materials"* and never names `SPLAT_N = 4`, RATIFIED at `visuals.md:60-65`.
- **Epoch/register arithmetic across the slice** — `ideas.md:436-438` (*"~2.5 Myr (200
  iterations over the ~500 Myr register)"*), `stubs.md:952-954` (*"460 m per 2.5 Myr = 0.18
  mm/yr"*), `visuals.md:388` (*"500 Myr of consequence"*). **Consistent.**

---

## 2. Claim inventory — load-bearing claims asserted as CURRENT
*(Transient, for stage 2. `file:line` at `f652b60`.)*

### `docs/design/stubs.md` — see § 3 for the per-entry heir table
- `:6-8` doctrine: a stub must be known, loud, listed with its heir; **an unlisted stub is a
  defect in this inventory, not a licence** — a closed-world claim.
- `:10-13` the four-field schema (fakes · expresser · loudness · blast) + same-commit update rule.
- `:15-24` provider slots exist as a *mechanism*; **four exist**; the general registry is
  deliberately unbuilt. ⚠ **F2.**
- `:47-50` a stub entry is **not neutral about its subject's standing — it asserts it** (the
  #1 lesson; the single most important sentence in the file for this baseline).
- `:1163-1199` layer-cake sibling gap: one slot (`outcrop_shares`), verdict = its argmax.
- `:1201-1205` genesis stubs are permanently legitimate (5 named).
- `:1207-1216` audited-and-rejected: 7 named non-stubs, incl. vertex-color albedo → visuals.md.

### `docs/design/corpus-knowledge-notebook.md`
- `:3` OPEN NOTEBOOK, nothing decided; `:15-18` § 4 stays empty. ⚠ **F7.**
- `:22-45` the user's sketch, verbatim-in-substance (**user-originated = data**).
- `:57-62` no prior art on tagging/frontmatter — **green field**.
- `:63-66` corpus size 192 files / 58,340 lines. ⚠ **F5.**
- `:112-129` three layers: DECISION healthy · CAPABILITY partially repaired · **OBLIGATION absent**.
- `:131-149` staleness ~8 %; born-false ~85 %. `:167-186` re-coded → **3–8 %**, 43 % coin-flips,
  *directions robust, percentages are bands*.
- `:255-308` T1 accretion-without-retraction · T2 address/lifetime mismatch · T3 one medium ·
  T4′ the adoption law · T5 the graph is authored, unread · T6 no enumeration is checked.
- `:310-318` control ladder C1–C5; C3 strongest.
- `:320-347` **the worst failures were not retrieval failures.**
- `:370-389` seven obligation kinds; only S-5 has a denominator. ⚠ `:383` **F6.**
- `:393-413` the one-directional edge; T6's three instances.
- `:429-462` reciprocity measured: 58/67 cited, **8 of 15** full-path edges one-directional.
- `:466-590` § 5 — **assistant-originated, unratified**: reciprocity as the cheapest cold seam.
- `:592-609` five falsifiers for the theory; three still **Owed**.

### `docs/design/corpus-knowledge-evidence.md`
- `:21-43` corrections read in full; multi-site edges recorded only **at correction time**;
  three propagation regimes (mutable authority / immutable testimony / owned-by-another).
- `:45-135` classifications A (carrier), B1–B10 (why it survived), C1–C5 (control class).
- `:169-217` spines: decisions findable, **built machinery was not**; A-2 has 4–5 variants with
  **different discovery procedures**; a sweeper may not mutate the taxonomy.
- `:238-270` git: 776 commits/11 days; ROADMAP = 35 % of commits; **design docs delete 2–4 %**.
- `:272-297` citation census; sub-document ids beat whole-doc **~35:1**; the version is a **date**.
  ⚠ `:274`/`:294` **F5.**
- `:299-341` `file:line` is a self-breaking address; the date stamp drifts; a hand inventory
  undercounted by 76 %.
- `:343-391` where the falsifier was (single-coder, /65). `:451-505` T1′ — amended **in place,
  at the claim site**, so shape 6's prescribed diff **cannot fire**; timeline-structured vs
  body-amended regimes.
- `:507-549` north-star § Passes still reads *trusted/first-party*, unstruck.
- `:551-565` `heir` 651 vs `HEIR:` 48 = 7.4 %. ⚠ **F5.**
- `:621-658` `S10-results.md` holds no pointer to corrections #12; **testimony vs authority**
  incompatibility. *(Note: `CLAUDE.md:54-82` DECIDED 2026-07-28 now answers this — immutable
  body, mutable header — and the evidence row does not yet reflect it. Not flagged as a pair:
  the row is dated reading, and its own § 3.7 declares CLAUDE.md read on that date.)*
- `:660-682` journal/0121: a ratified subsystem with exactly one caller, now zero.
- `:905-948` `stubs.md` schema: heading-string lifecycle · `(original)` node versioning ·
  **the number held identity through three renames** · four node states.
- `:950-989` coverage declared; **`PIPELINE.md`, `stubs.md` bodies, `ideas.md` bodies, `visuals.md`
  NOT OPENED** — i.e. **this slice is that declared gap being read.**

### `docs/design/ideas.md` — *user-originated except where marked*
- `:1-6` charter: sketches, NOT decisions.
- `:8-31, :102-130, :132-150, :152-167, :239-245` knowledge/sound/language/NPC/misc sketches.
- `:33-69` crafting-as-process + the accessibility synthesis (*agreed 2026-07-19, sketch-tier*).
- `:71-100` the historied player. `:194-221` uncollapsed history frontier (user counter, 2nd pass).
- `:223-237` posture ladder (crouch DECIDED separately in bodies.md).
- `:247-272` **the bio slot on a substrate** — a voxel is substrate + biotic occupancy.
- `:274-291` soil is loose but packable into structural.
- `:293-314` ready-made worlds. ⚠ **F9.**
- `:316-349` default pack ≈ Earth, tech cutoff ~medieval as a *design-attention boundary*;
  `:351-375` **REOPENED** — pure Earth vs bespoke setting, user-owned, undecided.
- `:377-391` **the aesthetic thesis** — *"it looks like a toy and reads like a core sample."*
- `:393-429` rock is not monolithic — jitter at the partials scale; loose materials must spread.
- `:431-454` coal partings belong to the **collapse tier**, never deep time.
- `:456-508` *the passes ARE the API being built* — **do not design a plugin-pass mechanism yet**;
  the alien-outpost probe; *"can a mod inject a fact into prehistory"* as the acceptance criterion.
- `:510-526` perf/debug overlay is player-facing (**ratified direction 2026-07-23**); the
  aggregation must be a queryable resource, the file dump one consumer.
- `:528-595` pass cadence — **RECONCILED → material-behavior §5**, ORDER half **RESTORED
  2026-07-26** (corrections #65), RATE survived, WINDOW unaffected; sketch retained as origin.

### `docs/design/visuals.md`
- `:3-7` elevated pixel thesis; `:9-14` mood, real darkness, warmth is earned.
- `:16-33` LabPBR three-texture packing; 16×16 expect 32×32; **porosity is sim-driven**.
- `:35-56` lighting (GI explicitly not pursued) · atmosphere · water.
- `:58-81` PBR-1 ratifications: **`SPLAT_N = 4` RATIFIED**; sun/ambient rides as placeholder;
  fullbright mixture visibility REQUIRED.
- `:83-104` **DECIDED — the distance speaks the voxel language**; FF2a/FF2b; + the recorded
  integrator breach (appearance is always user-owned).
- `:106-127` **DECIDED — amplitude by rarity** + anti-wallpaper jitter; lit and fullbright agree
  **statistically, not spatially**.
- `:129-150` mixture rendering road; ore is subtle, no distance glint.
- `:161-190` **DECIDED — real darkness underground**, reference is modded MC + shaders; the
  coal-reads-black fix is the lighting model, not the albedo.
- `:192-246` OPEN — the sim must know about light; sim light ≠ render light, **by design**.
- `:248-288` **DECIDED — the sim light model** (7 items): light is derived not stored; emission
  and opacity from material properties; reach per-source; **deep time gets no light field**.
- `:290-352` heavenly-bodies field; body paths → day/night emergent, seasons/latitude seam only.
- `:384-506` LOD colour cascade; the mixture LOD pyramid **already exists** (S8 `lod.rs`);
  **CORRECTED by the user** — seeing at 5 km never collapses; the far field is permanently on
  the `inspect` side. ⚠ `:429` **F8.**
- `:508-512` **Open, unratified** — band distances, mid-band blend vs dither, derived vs persisted.

### `docs/rendering/PIPELINE.md`
- `:3-9` v0 contract, decided by S4; packs survive because the shape is a **versioned contract**.
- `:15-50` **DECIDED — clustered forward (Forward+), no deferred pass**, with 7 justifications.
- `:52-70` Bevy 0.19 ownership table; tonemapping + post + shader loading are **ours, implemented**.
- `:72-106` 8 stages; only `post` is live (format 0); 4 stability promises; **exact format match,
  no compatibility heuristics**.
- `:108-152` pack manifest + loading rules; prelude owns bindings; naga-validated at load;
  fallback to the **built-in** default pack; **a pack failure is never a panic**.
- `:154-190` the `post` interface, verbatim from the prelude (normative copy); growth within
  format 0 prohibited.
- `:192-236` LabPBR triplet adopted verbatim from visuals.md; texture arrays, layer = material id;
  **DECIDED for v0 — per-voxel sim data rides vertex attributes** (storage buffer / index texture
  rejected, with revisit conditions).
- `:238-263` six reserved hooks with open questions; colored-light storage does **not** block.
- `:265-287` `HOOK_FORMAT = 0`; the 4-target portability gate runs in CI.

---

## 3. Stub heirs — is the heir still a live plan?

30 entry headings (`1`–`29` plus `7b`, plus preserved `14 (original)` / `29 (original)`).
**Live entries owing an heir: 24.** Confirmed live = the heir is named in a doc that exists and
is not itself retired. **Could NOT confirm as a live plan: 6** (marked ⚠).

| # | status | heir | live plan? |
|---|---|---|---|
| 1 | RESOLVED BY DELETION | social sim + ecology civ history | ⚠ **NO — explicitly none owed** (`:30`); see **F6** |
| 2 | retired-where-a-record-exists; **residuals live** | ecology system (vegetation) · the `identify` arc's continuation slot = runtime edit-fact overlay | ecology **yes**; overlay ⚠ **unconfirmed** (no doc address given) |
| 3 | RETIRED 2026-07-21 | — landed whole | n/a |
| 4 | live | metamorphism pass (geology.md) **via `providers::burial_temp_c`** | doc **yes**, address ⚠ **NO** — see **F4** |
| 5 | live | paleo-context provider (geology.md § formation context) | ⚠ unconfirmed (no sequenced item cited) |
| 6 | live (slot) | epoch-indexed paleo-temperature curve keyed by `chapter` | ⚠ unconfirmed (no doc address) |
| 7 | live | ecology-pass vegetation members (**ecology.md** Note 2026-07-21) | **yes** |
| 7b | live (slot) | S11 saturation field — **water.md DECIDED 2026-07-20, consequence 4** | **yes** (ratified) |
| 8 | live (slot) | P keyed on parent-material petrology | ⚠ unconfirmed (no doc address) |
| 9 | live | epoch-indexed pregen curves (**earth-processes § 6**) | **yes** |
| 10 | live, **blast now zero** | the S2 tier's real cached summaries | ⚠ **questionable** — S2's engine now has **zero production callers** (`:290-296`; `evidence:673-678`). *Existence is not standing* applies to the heir as much as the stub |
| 11 | live | upstream lode endowment (**ores.md R2-B**) | **yes** |
| 12 | RETIRED 2026-07-21 | prior heir **superseded** and said so | n/a |
| 13 | live (slot) | fetch × zonal wind (S11 body graph; ROADMAP) | **yes** |
| 14 | RETIRED 2026-07-24 | geotherm landed; slot removed | n/a |
| 15 | live | far-field summarization half (**octree-substrate.md § 3**) | **yes** |
| 16 | live | genesis/emplacement pass (**ratified, ROADMAP 2026-07-24**) | **yes** |
| 17 | DISCHARGED; **residual live** | R/H unification — **material-behavior.md §11 slot + §13.6**, ROADMAP Movement 2 | **yes** |
| 18 | live | FLOW continuation **(c)** form · **Movement 2b** cause · **(d)** fluid | **yes** (ratified slot) |
| 19 | live | a real water-balance climate (the precipitation-depth scale) | **yes** — paired with journal/0096's lateral source |
| 20 | live | depth-resolved front in the ledger; wants #16's heir | **yes** (sequenced-adjacent) |
| 21 | live | interned `EdgeId` at **the pager slice (S20 option 3)** | **yes** (reserved continuation) |
| 22 | live (knob ≠ heir) | flow.md § 9 item 7b aggregation window — **still the user's call** | **yes**, user-owned |
| 23 | live | fluid identity (**flow.md § 2.5** / genesis notebook § 5 item 1) | **yes** |
| 24 | scope DISCHARGED; **(b) superseded by §27/§29** | earth-processes § 3e iteration↔Myr calibration | **yes**, but see **F3** |
| 25 | live | packed provenance byte, landing with **§ 13.8 lineage history** | **yes** |
| 26 | live | (a) joint calibration · (b) a dimensionless χ | **yes** |
| 27 | live | (a) rivers that carry (FLOW) · (b) an uncapped creep operator | (a) **yes**; (b)'s stated *register* ⚠ **withdrawn** — **F3** |
| 28 | live | **the live magnitudes tour** (user-owned, appearance-class) | **yes**, more owed than before |
| 29 | live | a hillslope-transport slice (the limiter / donor-cell partition) | **yes** — named the most valuable erosion work |
| gap | live | structural-deformation term supplying dipped `outcrop_shares` (earth-processes § 7) | **yes** |

---

## 4. What I could NOT attribute
- **The `heir` 654-vs-651 and 192/193-file splits (F5).** Neither figure carries the pathspec
  or the tracked/untracked scope it was measured under, and both docs date the measurement to
  the same day. I did not re-run the census: this sweep is docs-against-docs, and a fresh count
  at `f652b60` would answer a different question than *which of the two the authors meant.*
- **Whether five provider slots (F2) changes the "not enough to design one from" judgement.**
  That is explicitly a user call and the skill forbids me resolving it.
- **Six stub heirs' standing** (#2 residual, #5, #6, #8, #10, and #1) — see the ⚠ rows in § 3.
  Four of them name no doc address at all, so there is nothing in the corpus to check the heir
  *against*; that absence is itself the finding, and it is `corrections #35`'s hole
  (*"a decision not to build gets a paragraph and no watcher"*) arriving from the heir side.
- **Whether `visuals.md:142-146`'s partial-height loose rendering is still dormant.**
  `ideas.md:417-421` says it shipped built-but-dormant because loose deposition never emits
  sub-8 columns; `stubs.md:598-628` (journal/0099) now emplaces a weathering front **in
  eighths**. Resolving that is docs-vs-**code** — `spine-audit`'s question, not this sweep's.
- **`PIPELINE.md`'s Bevy 0.19 / wgpu 29.x pins** (`:17`, `:285-287`) — a docs-vs-code claim,
  out of scope here, and flagged only so the next reader knows it was seen and skipped.

---

*Slice S4 complete. Nine findings, one declared null (PIPELINE × visuals), 4,288 slice lines
read in full at `f652b60`.*
