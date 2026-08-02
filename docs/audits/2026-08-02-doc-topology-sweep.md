# doc-topology sweep — 2026-08-02 (FULL)

**Read at `d8407e1`** (branch `main`; this file added on top). Every quotation below was
re-read at that commit, per this skill's *"a quotation without a revision is not a
quotation"* rule (corrections #67). Source-file quotations name the file and line at the
same commit.

**Mode: FULL.** 128 commits since watermark `72fbe86` (≥25 threshold), and two whole arcs
shipped in the window — **P11** (design pass + slice 1) and **BODIES** (B0 + posture-bake
member #0). That is the *"after any batch of merges that ships an arc"* trigger, twice.

**Read-only pass.** Nothing outside this file and `docs/audits/.sweep-watermarks.json` was
modified. **No corrections.md entry is proposed** — see § 4; every finding here is banner,
strike-through, ordinal or citation work, except one referral to `spine-audit`.

---

## 0. Coverage — what was actually opened, and what was NOT

*Per this skill's rule that a null from an unopened file is not a result.*

**Read in full:** `docs/design/materials.md` · `docs/design/bodies.md` ·
`docs/design/posture-gait.md` · `docs/dependency-graph.md` ·
`docs/audits/2026-08-01-members-into-history-design.md` (header + §§ 0–1) ·
`journal/corrections.md` §§ 84–86′ (lines 3492–3648) + the full heading index ·
`ROADMAP.md` §§ 359–502 (the P11 arc) + 4060–4194 (both close blocks) ·
`docs/design/geology.md` §§ 380–423 · `docs/design/material-behavior.md` §§ 13.7–13.8b ·
`docs/audits/2026-07-29-fluvial-record-terms-priors.md` header ·
`docs/audits/2026-08-01-p2-calibration-derivation.md` header · `docs/ARCHITECTURE.md`
§§ 652–671 · `docs/design/stubs.md` §§ 30–34 · the full `north-star.md` window diff.

**Source files opened as the code-side anchor of a doc pair:**
`crates/dc-worldgen/src/deeptime/lithology.rs` (module head + `Litho` enum doc) ·
`crates/dc-worldgen/src/deeptime/erosion.rs:380` ·
`crates/dc-worldgen/tests/providers_common/mod.rs:188,423`.

**NOT opened — a null over these is not a result:** `docs/design/flow.md` ·
`docs/design/refinement.md` · `docs/design/ideas.md` (136 lines added in the window,
unread) · `docs/spines.md` (306 lines changed in the window, unread — **the largest
unexamined surface in this sweep**) · `.claude/skills/session-workflow/SKILL.md` and
`wrap/SKILL.md` (both changed) · `docs/design/ecology.md` · `docs/API.md` beyond one grep
hit · every `journal/` entry except by grep · `docs/audits/baseline-2026-07-28/`.

**Two sibling sweeps ran concurrently and own their own docs; nothing here touches them.**

---

## 1. Ranked contradiction pairs

Blast radius first, per the skill's ordering rule. **⚑ = read-first surface.**

### F1 ⚑ — `corrections.md` has TWO `#85` and TWO `#86`. Six live citations are now ambiguous.

| | claim |
|---|---|
| **A** | `journal/corrections.md:3512` — *"## 85. **THE SPLIT AXIS IS LIVENESS, NEVER TOPIC** … (the file-size hook's `remedy()` fallthrough … falsified 2026-08-01)"* — added by `634742e` (bodies session). Its sibling `#86` at `:3550` (*"It runs at pack build rather than world gen"*, `bc9b612`) and `#87` at `:3598` (`45b5014`). |
| **B** | `journal/corrections.md:3617` — *"## 85. **The identity swap is a zero-byte change** (the P11 design audit § 3.1's headline…)"* and `:3634` — *"## 86. **Identity, not mass — so the terrain is unchanged**"* — both added by `dcc6b43`, whose subject line reads *"wrap the geo session, final: **ordinals**, corrections #85-86"*. |

**Neither is senior; both are correctly-filed corrections.** The defect is that the geo
session's wrap assigned "next free" **after** the parallel bodies session had already
taken 85/86/87 — the identical mechanism, on the identical night, as the journal
0135 collision that `3cab931` fixed. The file now reads `84, 85, 86, 87, 85, 86`.

**The citations this breaks, all pointing at B while A answers first to a grep:**
`docs/audits/2026-08-01-members-into-history-design.md:46` (*"corrections #85"* = the
zero-byte entry) · `journal/0136:431` (*"Corrections #85/#86"*) · `ROADMAP.md:4143`
(*"Corrections **#74–76, #84–86** (all targets stamped)"*). Pointing at A:
`docs/audits/2026-08-02-posture-bake-member0-design.md:6` · `docs/design/bodies.md:322` ·
`docs/design/posture-gait.md:96` · `docs/dependency-graph.md:107` (all *"corrections #86"*
= the venue correction).

**Blast radius: maximum.** `corrections.md` is read-first item 4 and its *whole* interface
is the stable ordinal — this corpus holds ~5,000 stable-id citations and the notebook names
`corrections #N` as one of the four id schemes that actually work. A duplicated ordinal is
the one failure that scheme cannot survive.

**Recommendation (labelled as one): renumber B to `#88`/`#89`** — A's ordinals were
committed first (`634742e` 2026-08-01 vs `dcc6b43` 2026-08-02) and A has three external
citations to B's three, but A's are spread across four *live design docs* while B's are in
two audits and a journal. Renumbering B touches three lines; renumbering A touches seven.
Then re-point B's three citations. **Not applied here — an ordinal rewrite of read-first
item 4 with a parallel user session live is an integrator act.**

---

### F2 ⚑ — `stubs.md` has TWO `#34`, and its only external citation calls one of them `#33`.

| | claim |
|---|---|
| **A** | `docs/design/stubs.md:1463` — *"### 34. the-binding-key-that-is-a-name-wearing-a-role — added 2026-08-01 (posture-gait.md § 7b ruling)"*. **Ten live citations**: `dependency-graph.md:88`, `:89` · `bodies.md:42`, `:288`, `:305`, `:359` · `posture-gait.md:296` · `ideas.md:731` · `ROADMAP.md:584` · `journal/0135:96` · `docs/audits/2026-08-01-body-plan-structure-design.md:20`. |
| **B** | `docs/design/stubs.md:1421` — *"### 34. the-deep-tiers-content-set-is-hard-wired-to-vanilla — added 2026-08-01 (P11 slice 1); **ordinal assigned at merge 2026-08-02**"*. Assigned by `164be0f` (*"wrap remainder: close block + **stub #34**"*), a two-line edit. **Zero citations use `#34` for it.** |
| **C** | `docs/dependency-graph.md:79` — B's only external reference, and it calls it *"(**stubs #33**; heir = E7's per-world manifest)"*. `stubs.md:1454` `#33` is **`a-private-gauss-module-inside-draws`** — a different, live stub from journal/0128. |

**A is senior** on both first-use and citation weight. B was assigned blind, in the same
wrap commit and by the same mechanism as F1.

**Blast radius: maximum.** `stubs.md` is read-first item 6 — *the* loose-ends lookup, whose
entire doctrine is *"an unlisted loose end is the defect."* A duplicated ordinal plus a
mis-citation means the P11 content-set stub is currently **unreachable by number from
anywhere**: `#33` lands on the gauss module, `#34` lands on the binding key.

**Recommendation: renumber B to `#35` and correct `dependency-graph.md:79` in the same
commit.** B's own heading already documents that ordinals follow time, not position, so the
renumber costs nothing but the line. **Fix ordering note:** F1 and F2 are one act by one
session; doing them apart risks a third assignment.

---

### F3 ⚑ — `material-behavior.md` § 13.8b argues, as current doctrine, the exact premise P11 ruling 1 overturned — and P11 slice 2 is the next thing dispatched.

| | claim |
|---|---|
| **A** | `docs/design/material-behavior.md:906-909` — *"**The load is a multiset of `(Litho, quantity)`** — seven species, which is **the material granularity deep time can distinguish at all** … **Resolving it finer would be inventing identity the tier does not have.**"* |
| **B** | `docs/audits/2026-08-01-members-into-history-design.md:14-20` ruling 1 (user, *"certainly"*) — the record names `MaterialId`; and `:52-63` ruling 6 (user, 2026-08-02) — *"**MaterialId — not membership of a group — is the basic unit of deeptime**: identity is a conserved quantity flowing through the mass arithmetic (erosion releases → transport carries by id → deposition records what settled)."* |

**B is senior, twice over: it is user-originated and it is later.** Note precisely which
half of A is wrong, because they are in different registers:

- **A's FACT still holds.** `erosion.rs:380` is still `const SPECIES: usize = Litho::COUNT`
  at `d8407e1`; transport goes member-grade in slice 2, not slice 1. As an *as-built*
  statement A is accurate.
- **A's JUSTIFICATION is dead.** *"The material granularity deep time can distinguish at
  all"* and *"resolving it finer would be inventing identity the tier does not have"* are
  now false — the deep tier resolves `MaterialId` at deposition and slice 2's entire scope
  is *"identity comes from the arriving composition, propagated, never drawn"*
  (`ROADMAP.md:460-463`).

**This is shape 5 — a justification whose constraint expired, with the expiry recorded in a
different file.** It is also the most *operationally* dangerous pair in this sweep:
`material-behavior.md` § 13 is on the named read path of the fluvial arc
(`2026-07-29-fluvial-record-terms-priors.md:54`, *"Read with: … material-behavior.md
§ 13"*), and the P11 close block's **first** action item is *"Dispatch P11 slice 2, FRESH
agent"* (`ROADMAP.md:4120`). A fresh slice-2 agent reading its own scope's design doc finds
its scope argued against, in the present tense, with no banner.

**Recommendation: a banner on § 13.8b**, in the *"immutable body, mutable header"* shape —
keep the bullet as the dated 2026-07-26 record of what shipped, add the P11 pointer above
it. This is the highest-value single banner available right now.

---

### F4 — `lithology.rs`'s `Litho` doc comment refutes its own module docs, and points the reader at the refutation.

*Referred to `spine-audit` (docs-vs-code is its question, not this sweep's). Recorded here
because it is F3's source-side anchor and because it is corrections #65's exact shape.*

| | claim |
|---|---|
| **A** | `crates/dc-worldgen/src/deeptime/lithology.rs:247-250` — *"This is not the material registry — it is the handful of *classes* the deep-time record can distinguish. **Members within a class are chosen at collapse time and do not vary the erosion rate** (see module docs on the pack-addition blast radius)."* |
| **B** | `crates/dc-worldgen/src/deeptime/lithology.rs:47-70` (same file, ~190 lines above) — *"**Since P11 slice 1 (2026-08-01) the deep tier resolves the material at DEPOSITION** … deposition-time fitness reads the live registry, the deposited identity feeds `exposed_shares` → `susceptibility_table` → the erosion rates, and **a pack that registers a new clastic member therefore *can* change how fast a hillside wears down.**"* |

**B is senior** — it is the P11 slice's own rewrite (`d6393d1`, *"stamp the two docs that
still asserted the retired pack-safety guard"*), which reached the module head and missed
the enum. A's parenthetical *"see module docs on the pack-addition blast radius"* sends the
reader **to the paragraph that contradicts it**. Members are no longer chosen at collapse
time for deep history, and they now do vary the erosion rate.

**Recommendation: strike A's second sentence** in the slice-2 pass, which is already opening
this file.

---

### F5 ⚑ — `materials.md` still lists as *undecided* everything the P11 design pass ruled, and the P11 audit points readers at exactly those lines.

| | claim |
|---|---|
| **A** | `docs/design/materials.md:536-539` — *"**What is NOT decided:** the representation (identity per unit? member distributions? what happens to the member-fitness/dither machinery, the Litho roster, residency), the migration path, and the re-sequencing of slated work — all owned by the arc's **design pass**."* |
| **B** | `docs/audits/2026-08-01-members-into-history-design.md:21-35` rulings 2–3 (representation = **A-CLEAN**; transport **sparse day one**) and `:64-71` ruling 5 — *"**The design phase of P11 is CLOSED**."* Plus `ROADMAP.md:385` and `dependency-graph.md:79`, both *"design pass DONE."* |

**B is senior** — the rulings are user-originated and later. A's own paragraph is not
wrong-as-written (it was true on 2026-08-01 morning); it is shape 1, superseded and still
asserted, in the doc the audit itself names as its **companion**
(`2026-08-01-members-into-history-design.md:4`, *"Companion: `docs/design/materials.md`
**:522-541**"*) — i.e. a reader following the audit's own pointer lands inside the stale
paragraph.

**Recommendation: strike A's "What is NOT decided" clause and replace with the six rulings
+ a pointer to the audit header.** Keep the "What is decided" half and the user quotation
verbatim — they are the ratification record.

---

### F6 ⚑ — corrections #86 stamped two prose sites and missed the TABLE in each. Both tables still say "pack build".

| | claim |
|---|---|
| **A₁** | `docs/design/posture-gait.md:88` — the § 3 machine table, bake column: **`| when | pack build; gen-time free | …`** |
| **B₁** | `docs/design/posture-gait.md:95-101`, **seven lines below**, same section — *"~~so it runs at *pack build* rather than world gen~~ — **VENUE CORRECTED 2026-08-02 (user; corrections #86): purity makes the bake callable from ANY clock**… **deeptime worldgen for species the evolution pack mints**."* |
| **A₂** | `docs/dependency-graph.md:89` — the B2 row **title**: *"**posture + gait bake** — derived, per species, **at pack build**"* |
| **B₂** | `docs/dependency-graph.md:106-110`, ~17 lines below in the same file — *"~~produced at *pack build* rather than world gen~~ — **VENUE CORRECTED 2026-08-02 (user, corrections #86)** … **worldgen is a first-class caller**."* |

**B is senior in both** — user-originated, and `corrections.md:3550` records the ruling
verbatim. **The generalisable finding is the failure shape, not the two cells:** a
correction author stamped the prose at the claim site and did not scan the section's own
**summary table**, twice, in one commit. Tables are what a scanning reader reads, and both
are inside the same `##` section as their own refutation — this is shape 2 at the shortest
distance yet measured in this corpus (7 lines).

Cost is not hypothetical: `#86`'s own text records that the venue claim survived *"one
ratification, one graph transcription, and one design pass"* while stated in prose. It is
now still stated in two tables.

**Recommendation: edit both table cells** (`pack build; gen-time free` → `any clock — pack
build, deeptime worldgen, define-time`; B2's title → *"derived, per species, at any minting
clock"*). Mechanical; no ratification implicated.

---

### F7 ⚑ — Four live docs quote `GOLDEN_SURFACE`/`GOLDEN_RECORD` values that are two generations stale, while the ROADMAP close block says even the *current* values are expected-red.

| | claim |
|---|---|
| **A** | `docs/ARCHITECTURE.md:664-665` — *"an empty cadence table reproduces **the shipped world bit for bit** (`GOLDEN_SURFACE 0x15A6_B756_7A84_29FB` / `GOLDEN_RECORD 0x820B_A198_49DD_234A`)"*. Same pair at `docs/dependency-graph.md:135-136`, `docs/design/stubs.md:1280-1281` (*"the shipped world **is** hash-identical"*, present tense), `ROADMAP.md:929`. |
| **B** | `crates/dc-worldgen/tests/providers_common/mod.rs:188` — `pub const GOLDEN_SURFACE: u64 = 0xBF63_DA9D_2974_022A;` and `:423` — `pub const GOLDEN_RECORD: u64 = 0x6739_19DA_BBA4_EA86;` |
| **C** | `ROADMAP.md:449-455` and `:4111-4117` — *"the golden families are **EXPECTED RED on main**: the RECORD halves (`GOLDEN_RECORD` + 5 variants) … the SURFACE family … Accepted as-is by user ruling 2026-08-02."* |

**The claims in A are still TRUE as dated testimony** — RATE *did* reproduce the world bit
for bit on 2026-07-29, and immutable-body-mutable-header protects that. **What is stale is
the identifier**: those hex values have moved at least twice since (the octaves re-capture,
then P11 slice 1's), and per C the *current* constants do not describe main either.

**Blast radius: high and specifically shaped.** A cold session grepping the corpus for
`GOLDEN_SURFACE` gets `0x15A6…` in **four** places and `0xBF63…` in **zero**, across
read-first items 1c, 3 and 6. Nothing in the corpus records that the constants moved. This
is the *"the corpus was never swept for missing banners"* backlog CLAUDE.md read-first
item 5 names, landing on its highest-traffic instance.

**Recommendation: a one-line dated qualifier at each of the four sites** — *"(the values as
of 2026-07-29; the goldens have been re-captured since — see ROADMAP § the P11 arc)"*.
**Do not update the hex** — that would rewrite dated testimony, and the current values are
themselves expected-red until slice 2's single capture.

---

### F8 — `journal/pending-p11-slice1` is still unresolved in three files, one of them read-first.

| | claim |
|---|---|
| **A** | `docs/design/geology.md:413` — *"⚠ HALF-DISCHARGED 2026-08-01 by P11 slice 1 (**journal/pending-p11-slice1**)"*; `ROADMAP.md:396` — *"✅ BUILT 2026-08-01 (**journal/pending-p11-slice1**)"*; `docs/audits/2026-07-22-seam-inventory.md:14` — same token. |
| **B** | The file exists as **`journal/0136-the-day-the-record-started-naming-rocks.md`**, renamed by `3cab931`, whose message says *"All six corpus references to journal/0135 mean the B0 entry; only the geo close block's two lines meant the P11 journal — **both patched**."* |

**B is senior.** The renumber commit swept for `0135` — the *wrong* token. The
slug-only placeholder (correct practice under the parallel-session rule) was never swept
for at all, so three pointers now name a path that has never existed.

**Recommendation: replace all three with `journal/0136`.** Purely mechanical. *Worth noting
as process: slug-only briefing works, but "resolve the slug at merge" needs the slug in the
sweep list, not just the ordinal.*

---

### F9 ⚑ — `dependency-graph.md` § 2 and § 4 still describe two design passes as not-yet-run; both ran and merged in this window.

| | claim |
|---|---|
| **A₁** | `docs/dependency-graph.md:78` (P10 row) — *"**Design pass startable today**; calibration gated on P2"* |
| **B₁** | `docs/audits/2026-08-01-p10-grain-axis-design.md` exists (merged `1b74c21`, *"merge P10 grain-axis design pass: the Wentworth ladder was already built"*), with a U1 leaning in its header; `ROADMAP.md:4146` lists *"P10 design (U1 leaning in header)"* among the session's artifacts. |
| **A₂** | `docs/dependency-graph.md:203` (§ 4 item 4) — *"**P2 — the calibration re-pick.** Needs a **literature pass**, not an engineering one."* and `:70` (P2 row) — *"**SEQUENCED.** 45 was fitted to the broken solve…"*, naming no audit. |
| **B₂** | `docs/audits/2026-08-01-p2-calibration-derivation.md` exists (merged `ee9fb94`), header carries a **user ruling** (the craton present-state band, target ≈ 2.6, **M ∈ [60, 240]**) and a runnable measurement plan; `ROADMAP.md:4129` sequences *"P2 measurement runs (plan in the P2 audit)"*. The literature pass **is done**. |

**B is senior in both.** These are also **one-directional pointers**: both audits cite the
dependency-graph rows by name (`p2-calibration-derivation.md:20` cites *"§ 2 row P2; § 4
item 4"*) and neither row cites back.

**Blast radius: high.** `dependency-graph.md` is read-first item 1c, is read *at session
start* by directive, and its § 4 is literally titled *"Ready to start today, in order"* —
the artifact a cold session uses to pick work. It currently ranks P2's literature pass at
#4 and omits **P11 entirely**, which its own § 2 row marks **TOP PRIORITY (user)**. That is
the same class of miss as corrections #71, in the file built to prevent it.

**Recommendation: refresh the P2/P10 rows with their audits and their rulings, and re-cut
§ 4 against the 2026-08-02 close block.** The doc's own § 5 makes this owed in the same
commit as the merges; it is the largest single staleness surface found.

---

### F10 — `ROADMAP.md` says the P11 audit header holds "five rulings" and, 3,700 lines later, "six".

| | claim |
|---|---|
| **A** | `ROADMAP.md:385-392` — *"**✅ DESIGN PHASE CLOSED 2026-08-01** … **five rulings** in its header"*, followed by an enumeration of exactly five. |
| **B** | `ROADMAP.md:4099` (the live close block) — *"the P11 audit header's **six rulings**"*. Confirmed at source: `2026-08-01-members-into-history-design.md` carries rulings 1, 2, 3, 4, 6, 5 (ruling 6 added 2026-08-02 by `27340e2`; note it is **printed out of order, between 4 and 5**). |

**B is senior** (later, and matches the artifact). The arc entry does cite ruling 6 in its
slice-2 body (`:460`), so nothing is lost — but the *header sentence a reader skims* omits
the ruling the close block calls *"governs everything next."*

**Recommendation: update `:386` to six and add ruling 6 to the enumeration.** Separately,
consider re-ordering the audit header so 5 precedes 6 — an out-of-order ruling list in a
mutable header is a small ordinal hazard of its own, and this session already paid for two.

---

### F11 — `bodies.md`'s `## Plan parameters — PROPOSED (user sketch 2026-07-19)` heading was **overwritten**, not moved. Its body survives under a `DECIDED` heading, and four citations dangle.

> **✅ RESOLVED BY USER RULING 2026-08-02 — no action, deliberately.** *"Let the bodies
> session worry about bodies.md; if we accidentally committed their change then it was an
> in-progress or intended change and they will make sure it's right."* The finding is
> released to the owning session; the geo/integration side takes no restore. (Stamped by
> the ruling's recorder, same commit.)

| | claim |
|---|---|
| **A** | `docs/design/bodies.md:297` — `## Postures — DECIDED 2026-08-02 (user; sweep-checked…)`. `f815d69`'s diff replaces the line `-## Plan parameters — PROPOSED (user sketch 2026-07-19)` with this heading; the scalar/bool plan-parameter body at `:335-353` is **untouched** and now sits inside § Postures with no heading of its own. |
| **B** | Four live citations to a section that no longer exists: `docs/design/bodies.md:99` (*"the reason this matters to § **\"Plan parameters\"**"*) · `docs/API.md:348` (*"bodies.md § plan parameters (**PROPOSED**)"*) · `docs/design/ecology.md:189` (*"**PROPOSED** scalar/bool plan parameters"*) · `docs/audits/2026-08-01-body-plan-structure-design.md:417`. |

**Two distinct defects, and the second is the one that matters:**

1. **Four dangling cross-references** — mechanical.
2. **A provenance marker was deleted.** The heading was the only place recording that
   scalar/bool plan params are a **user sketch** and **PROPOSED**. Three of the four
   citations above still carry `PROPOSED` and now disagree with the file they cite; a
   reader of `bodies.md` alone now finds that content under **DECIDED 2026-08-02**.
   *The content is unchanged and no design was reconciled away* — so this is **not** a
   CLAUDE.md § *"a user-originated design may not be superseded by an implementation
   slice"* violation, and I am not marking it ⚠ CONTESTS. But it is the provenance-signal
   half of the same rule, and the corpus paid for provenance inversion **eight days ago**
   in corrections #86's own postscript.

**Recommendation: restore `## Plan parameters — PROPOSED (user sketch 2026-07-19)` above
`bodies.md:335`.** Almost certainly an accidental heading collision during the insert (the
new section was appended at the old heading's line), not a deliberate retirement — **but
confirm with the session that wrote `f815d69` before assuming so**, because the alternative
reading is that a user sketch was folded into a ruling.

---

### F12 — The record-terms priors banner names a gate that has already opened.

| | claim |
|---|---|
| **A** | `docs/audits/2026-07-29-fluvial-record-terms-priors.md:24` — *"**Ruling 3 (face-vs-unit) is HELD pending P11's design pass and ripple map**"*. |
| **B** | `ROADMAP.md:470` — *"**Record-terms ruling 3 (face-vs-unit) re-enters for ruling after **slice 2's** re-derived numbers land"*; also `dependency-graph.md:79`, *"Record-terms ruling 3 re-enters after slice 2."* The design pass and ripple map are **DONE** (audit ruling 5, *"the design phase of P11 is CLOSED"*). |

**B is senior.** A's condition is satisfied and its gate has moved one slice later. Low
blast radius (one audit header), but it is precisely the *"a close block recording a
resolution owes the asking document its banner"* case from CLAUDE.md read-first item 5 —
the answer lives in the ROADMAP and the asking document was never re-stamped.

**Recommendation: amend the banner's gate to "after P11 slice 2".** Header edit only; the
body is dated research and stays.

---

### F13 — `dependency-graph.md:14` — *"Last full pass: **2026-07-29**."*

The file has been edited through 2026-08-02 (P10, P11, B0, B2, the venue correction). Not a
contradiction of a *claim* so much as a header the file's own § 5 process makes false. Low
value alone; listed because F9 is in the same file and one commit fixes both.

---

## 2. A premise hiding inside a caveat (shape 7) — one candidate, NOT resolved

Per the shape-7 procedure: separate the measurement from the claim wrapped around it, and
ask what would have to be true **about the game** for the number to matter.

**Candidate — `docs/dependency-graph.md:78`, the P10 row:** *"calibration **gated on P2**
(pre-P2 no cell can carry sand)"*, echoed at `ROADMAP.md:501-502` (*"Calibration is gated on
P2 (pre-P2, no cell can carry sand — competence ceiling 0.283 vs 0.840)"*).

- **The measurement is sound** and cited (the competence ceiling numbers).
- **The premise riding inside it:** that a grain-size axis is not worth *building* until it
  can be *calibrated*. That is not written down as a decision anywhere. The corpus's
  standing position points the other way — *"gen time is not a constraint"*, and the P10
  arc's own stated reason for existing (`ROADMAP.md:487-488`) is *so it cannot be lost*,
  not so it can be tuned.
- Note that the row **already distinguishes the two** (*"Design pass startable today;
  calibration gated on P2"*) — so this is a weak instance, and the caveat may be doing no
  damage. **Reported, not resolved.** If it ever hardens into *"P10 is blocked on P2"*,
  that is the shape-7 failure and it will read as evidence.

**No instance found in the P11 or bodies arcs.** Both were checked against the shape-7 rule
(*"cheapest place to look: clauses blocking something user-owned"*): the P11 audit's U-list
is explicit that U5 was **dissolved by a prior user rule**, not gated, and the bodies
arc's blocking clauses (`stubs #34`'s "before the firewall moves") name a mechanism, not an
assumption about play.

---

## 3. Items flagged as UNVERIFIABLE rather than reconciled

1. **F11's intent.** Whether `f815d69` deliberately retired § Plan parameters or collided
   with its heading is not answerable from the diff. Reported both readings; recommended
   the restore **conditional** on the authoring session's confirmation.
2. **Which side of F1/F2 should renumber.** I have given a labelled recommendation based on
   commit order and citation count. **It is a coin-flip that costs real edits either way**,
   and F1 touches read-first item 4 — integrator/user call, not a sweeper's.
3. **The current truth of any golden hash (F7).** `ROADMAP.md:4112` records that the full
   workspace gate was **deliberately not run** on merged main. **No cargo was invoked by
   this sweep** (brief constraint), so I can state what the constants *are* in the tree but
   **not** which of them actually hold. Anyone acting on F7 must not convert "the constant
   is `0xBF63…`" into "the world hashes to `0xBF63…`".
4. **`spines.md` (306 lines changed, unread) and `ideas.md` (136 added, unread).** The
   window's two largest unexamined doc surfaces. `spines.md` is read-first item 0b and
   gained § 3 rows from both arcs; `ideas.md` is the primary locus for shape 3
   (user-originated design reconciled away) and this sweep therefore **did not run
   shape 3 at all**. A null on user-design reconciliation from this sweep is **not a
   result** — it was not looked for. *This is the single largest gap and the first thing
   the next run should take.*

---

## 4. Why no `corrections.md` entry is proposed

Per the deliverable rule — *a contradiction is not automatically a correction.* Testing each:

- **F1, F2, F8, F10, F13** are ordinal/citation defects. Nothing was ever believed false.
- **F6, F9, F12** are things that were true when written and were superseded; the
  superseding record already exists (#86, the audits, the ROADMAP). Filing again would
  create a fifth pointer to a fact already recorded four times.
- **F7's claims are true as dated testimony.** Only the identifier is stale — a banner, by
  the immutable-body rule.
- **F3/F4** are the closest call. But `material-behavior.md` § 13.8b's *fact* is still
  accurate at `d8407e1` (`erosion.rs:380`), and its *justification's* expiry is already
  filed as ruling 1 + ruling 6. **What is missing is the sentence saying so** — which is
  exactly the case the deliverable rule describes as "not a correction."
- **F5, F11** are strike / restore work.

**Two process observations, offered for main session, not recorded anywhere by this sweep:**

1. **Ordinal assignment at merge is now 0-for-3 across parallel sessions in 24 hours** —
   journal 0135 (caught, fixed), corrections 85/86 (live), stubs 34 (live). All three were
   the *same act*: a wrap assigning "next free" without re-reading the file at the tip. The
   two that were caught were caught by a human noticing, not by any check. *This is the
   shape the notebook calls a convention that dies because it asks the author to restate —
   except here the author is asked to re-**read**, at the moment they are least likely to.*
2. **corrections #86's stamps reached the prose and missed the table, twice (F6).** If
   there is one cheap addition to the writer-stamps-the-target rule, it is *"scan the
   section's own summary table."*

---

## 5. Watermark

`doc-topology` advanced `72fbe86` → `d8407e1`, written in this commit per the
same-commit rule. Coverage is **spine-following, not 100 %** — § 0 lists exactly what was
not opened, and § 3 item 4 names the gap that most needs the next run.
