# Corpus knowledge — EVIDENCE LEDGER

**Extracted 2026-07-28 from `corpus-knowledge-notebook.md`** when that file crossed the
filesize hook's threshold — the hook fired on the document analysing why mechanical controls
work, which is recorded here because it is the cleanest possible demonstration of § 3.6.

**This file holds the READING. The notebook holds the SKETCH, the PRIORS and the THEORY.**
Every claim in the notebook's § 4 must cite a row here. One row per artifact actually read,
with what it is evidence *of*, and declared coverage gaps at the end.

← back to [`corpus-knowledge-notebook.md`](corpus-knowledge-notebook.md)

---

## 3. Evidence ledger

One row per artifact **read**, with what it is evidence *of*. Coverage gaps are declared
explicitly at the end, per the doc-topology precedent (*"NOT OPENED AT ALL — declared so the
gap is legible"*).

### 3.1 `journal/corrections.md` — the failure corpus (65 numbered entries + #63b)

**READ IN FULL, 2026-07-28** (2,501 lines, entries #1–#65 plus #63b).

**Structural observation (a):** the *titles* of later corrections cite **the multiple sites
where the falsified claim was asserted**; early ones do not. Examples:
- #54 — *"flow.md § 2.6, spines § 3, `head.rs` docs"* (four sites named in the body)
- #62 — *"`stubs.md` #29, `tests/mfd_routing.rs::no_interior_cell_is_cut…`, the ROADMAP blocker entry, and the walk's own first probe"*
- #65 — *"`material-behavior.md` § 5…, `journal/0090`, `north-star.md` § Passes"*

**The corpus already records multi-site assertion sets as edges — but only at correction
time, i.e. after the failure has been paid for.** Retrospective, in prose, by hand.

**Structural observation (b) — the file has a strike-at-source practice, and it is
explicitly bounded.** #54 (`:1845-1856`): *"a correction that lives only in this file is a
correction the next author does not meet"* — so four sites were struck at source. And then
the boundary is stated: **two sites deliberately NOT struck because the journal is
append-only** (*"an entry records what was believed on the day it was written and rewriting
it would destroy the very thing the journal is for"*), a dated audit likewise, and
**`ROADMAP.md:~2740` flagged rather than edited because it belongs to the integrator, not
the slice.** This is the corpus already distinguishing **mutable-authority** documents from
**immutable-testimony** documents from **owned-by-another-actor** documents — three
different propagation rules, applied by hand, recorded once, in prose.

#### Classification A — what kind of artifact carried the false claim

Counted from the entries' own attributions. A claim usually sat in several kinds at once,
which is itself the finding.

| carrier | examples |
|---|---|
| design doc / spike-results / ROADMAP | #8, #9, #12, #14, #20, #22, #30, #31, #41, #53, #54, #55, #56, #58, #59, #62, #65 |
| **code comment / docstring** | #2, #11, #16, #29, #32, #35, #36, #40, #52, #56 (corollary), #57, #58, #59 |
| **a test or a test helper** | #32 (green for an unrelated reason), #46 (hand-fed magnitude), #51 (`production_field()` builds neither the production seed nor extent), #62 (a saturating guard) |
| **a dispatch brief** | #17, #33, #36, #38, #42, #46, #47 |
| **the read-first doctrine itself** (`CLAUDE.md` / skill) | #19, #21, #37 |
| **a measurement instrument / harness** | #15, #18, #25, #34, #50, #51, #60, #61, #62 |
| **an attribution of the user** | #13 (a paraphrase dressed as a direct quote, cited in a live design decision within a day) |

#### Classification B — WHY each false claim survived (the load-bearing axis)

Ranked by count, with the corpus's own words. This is the axis a control has to attack.

- **B1 · Prose cannot fail a build.** The single most-cited survival mechanism, and the
  corpus states it as a law. #29 (`:950-953`): *"the dormant capability's precondition was
  recorded as prose in a doc comment directly above the code — **which is the right place**
  — and still failed, because **prose cannot fail a build**."* Also #32, #35, #40, #48, #52.
  - **#35 (`:1138-1142`) names a hole in the corpus's own ontology:** *"a justification for
    **not building** something is a claim with a shelf life, and it expires silently. **A
    stub gets an inventory entry and an heir; a decision not to build gets a paragraph in a
    doc comment and no watcher.**"*
- **B2 · The instrument was structurally blind to the question.** #18/#19 (fullbright is
  blind to shape), #25 (sampled the two flattest places, concluded about the world), #26 (an
  extremum of a *rate* read as an extremum of a *state*), #34 (window shorter than the
  period), #15 (a boundary condition the real system does not have), #50 (a voxel census
  cannot carry provenance), #46 (a fold test hand-fed 1.3 m where production is 0.04 m),
  #51 (a guard green on a world nobody ships), **#61** (*"a criterion over a global
  aggregate cannot license a claim about local structure"* — relief +4.6 % while roughness
  went ×170), **#62** (*"a ranking test wearing a magnitude test's clothes"* — it
  **saturates**, so it degrades exactly as the defect generalises).
- **B3 · A summary outran its source.** #12 (*"nothing in the results doc said which driver
  produced the table"*), #40 (*"an audit that paraphrases a comment can manufacture the very
  defect it reports"*), **#53** (*"**a table stated verdicts without their condition**, while
  the condition sat in prose two subsections away… 'Nine sites fail' travelled into § 7's
  summary and into an implementation brief as a fact"* — blast radius 9× too large). The
  doc-topology sweep independently proposed this as its sixth shape.
- **B4 · The claim was never written as a sentence, so nothing could contradict it.**
  #55 opens *"**Never written as a sentence, which is why it survived**"*; #56 the same
  words; #31 (*"the unstated inference is what the claim was being used for"*). **An
  implicit premise has no address, therefore no version and no watcher.**
- **B5 · A category error across kinds of claim.** The corpus catches this five times in
  two days and names it each time:
  - #54 — *"**'X unlocks Y' is a physical claim wearing a schedule's clothes.**"* Also:
    *"**one word — potential — named two different fields**"*, and the killer:
    ***"The document contained both halves and the sequencing note read only one of them."***
  - #55 — a claim about **naming** read as a claim about **magnitude**.
  - #56 — a claim about **units** (and *"nobody ever divided one by the other"*).
  - #58 — a claim about **shape** smuggled in as a claim about **identity**; *"the two
    statements are **in the same document, forty lines apart**."*
  - #59 — *"when a threshold and the quantity it thresholds carry different things inside
    them, one of them is holding a constant it does not own."*
  - #65 — *"**a derivation whose inputs were constructed to produce the desired output is
    not a derivation.**"*
- **B6 · A justification whose premise expired, silently.** #35 (*"the comment was still
  perfectly argued the day it became false"*), #59 (an anchor derived from a constant already
  named on the ROADMAP as due for recalibration), **#57** — *"**a rule can be wrong and
  unreachable at the same time, and 'unreachable' is a property of the CURRENT magnitudes,
  not of the rule.** When a slice multiplies the throughput of a path by three orders of
  magnitude, every latent rule on that path becomes live at once."*
- **B7 · Closed-system self-consistency.** #56 (the ~10³ scale error: *"mass closes, goldens
  hold, passes are pure, and the simulation is perfectly self-consistent at the wrong
  scale"*), #59 (*"the constant was checked against a number that was itself unchecked"*),
  #60 (*"two instruments with completely different failure modes landed within 2.4 % **by
  coincidence of smallness**"* — *"a cross-check is only evidence if it survives the regime
  it is being used to license"*).
- **B8 · BOTH HALVES WERE ALREADY IN VIEW, AND NO READER HELD THEM TOGETHER.** ⚠ **This is
  the class the user's sketch is aimed at, and the evidence says the failures were NOT
  retrieval failures.** Every instance had the refutation *already open*:
  - #54 — *"the document contained both halves"* (same document).
  - #58 — *"the two statements are in the same document, **forty lines apart**"*, and the
    falsifier was **the slice's own unit test, two lines below the assertion**.
  - #65 — *"the refutation was already in the corpus, **two sentences into the entry that
    celebrated the replacement**."*
  - #19 — *"**I had the falsifying fact in my own context, from this same session**"* — the
    integrator had quoted the contradicting S4 result verbatim hours earlier.
  - journal/0119's collapse (`ROADMAP.md:625-629`) — the sketch, the reconciliation, the
    refutation and the unbuilt axis were *"**all in the corpus, all findable by grep**…
    What failed is that **no reader ever had all four in view at once**."*
- **B9 · Shared build state / harness ambiguity.** #7, #21, #27, #32, #37 — infrastructure,
  not knowledge, but the same epistemic shape ("did it run?" ≠ "did it pass?").
- **B10 · Provenance was dropped from a value.** #13 (a quote nobody said), #28 (*"a
  coordinate written without its unit… **a bare number pair is not an address**"*), #12 (a
  cost without its code path), #48 (*"**a prose landmark is not a pose**"* — 39 km wrong,
  producing four confident null frames), #51 (*"a helper named for an environment must BE
  that environment"*), #50 (*"voxel contents carry no provenance"*).

#### Classification C — what class of CONTROL the fix used, and whether it held

This is the axis the user's *"impossible by construction, measurably"* lands on, and the
corpus has already run the experiment several times.

| control class | instances | verdict **from the corpus itself** |
|---|---|---|
| **C1 · a prose rule in a doc/docstring** | #29's parked precondition · #32's first remedy · CLAUDE.md's "re-run the probes by hand" | **FAILS, repeatedly and fast.** #32 (`:1059-1071`): *"**A rule in a docstring cannot fail a build**"* — the docstring rule was itself retired. CLAUDE.md § Gates: the probe advisory *"**was tried here and FAILED IN ONE DAY**… **Do not answer 'the gate cannot see X' with a rule asking people to remember X.**"* |
| **C2 · an assertion / test** | #57's `no_deposited_unit_claims_to_be_an_in_place_organic` · #29's heir · the `test = true` probe conversion | **Works only when the assertion can see the question** — and B2 is the list of times it could not (#46, #51, #62). |
| **C3 · make the illegal state unrepresentable in the TYPE** | #10 `true_surface_m → Option<f64>` · #32 `Option<fn>` with `None` = identity · #49 `Identity::Unrecorded` distinct *in the type* from `Mixture(EMPTY)` · #52 the `Domain`/`Draws` provider | **STRONGEST, and the corpus says why.** #32: *"**the constraint did not get enforced, it stopped existing**"*, plus *"that is the question the `Option` exists to make unaskable."* #10: *"when 'no answer' and 'an answer' share a type, **the silent path is the one that ships**."* #52 lesson 4: *"the durable fix was not a corrected comment but **a mechanism that makes the claim unnecessary**."* |
| **C4 · a hook the harness runs** | **exactly one exists** — `filesize_hook.py` on `Write\|Edit` | Untested as a class here; its own thresholds are *"still the hook's provisional guesses"* (close block § Owed). |
| **C5 · a recurring human sweep** | `spine-audit` · staleness sweep · `doc-topology` | Finds real things (**19 pairs, 5 of top 8 unsuspected**) — **and exists because C1 failed.** Also demonstrably misses: `DeepField::chapters` sat unlisted through **three** spine audits while three consecutive sweeps added rows for its neighbours **in the same struct**. |

#### Two more facts from this read that constrain any design

1. **The corpus already has a self-aware epistemic vocabulary, applied unevenly by hand.**
   #63 is *"flagged as a hypothesis by its own author and falsified the same day"* and the
   entry **credits the flagging** (*"which is why this correction cost two probe runs instead
   of a fix slice"*). #63b was *never written into a doc at all* because it was labelled a
   hypothesis in conversation. #27 states its epistemic status explicitly (*"the integrator
   did not reproduce the stale serve and cannot distinguish it from…"*). #23 marks its own
   mechanism **HYPOTHESIS, not yet measured**. So the distinction *measured / derived /
   asserted / hypothesised / user-ratified* **exists in practice and has demonstrably
   changed outcomes** — but it is carried in free prose, not in any queryable field.
2. **Time-to-falsification is collapsing, and that is a control working.** #54, #53, #57,
   #58, #60, #61, #62, #63 are all *"falsified the same day"* or *"the same week"*, several
   **by the slice that was building on the claim**. Early entries ran for days across
   multiple walks (#5 survived three walks). Whatever is producing that acceleration is the
   thing to keep.

---

### 3.2 `docs/spines.md` — read: head, § 2 (A-1/A-2/A-4/A-5/A-6), § 3, § 4, § 5, A-7, § 6

- **Its origin is a knowledge failure, and it names the discriminating fact** (`:3-5`, `:69-72`):
  created *"after a session in which the corpus turned out to be ahead of the assistant
  **fourteen times**. Not because the ideas were missing — because they were **already built
  and lost**."* And: ***"Decisions were findable this session; DECIDED entries did their job.
  Built machinery was not findable."*** → **the corpus's DECISION layer worked; its
  ARTIFACT/CAPABILITY layer did not.** That asymmetry is a fact about which layer needs
  addressability.
- **`:61-67` is the corpus's own document-type ontology**, hand-maintained: ROADMAP = what &
  in what order · ARCHITECTURE + design/* = what we decided, why, with dates · journal = what
  happened, as narrative · corrections = what we believed that was false · spines = what
  SHAPE, where it lives, and what exists that nothing calls. **A five-type schema already
  exists in prose.**
- **Naming is itself a control, stated twice.** `:142-147` — S-2's storage corollary was
  *"reached for three times in four days and never once designed… **an unnamed pattern is one
  an author re-derives**"*; S-2's status line (`:171-172`) — *"our most under-exploited spine
  — **implemented three times under three names** before anyone recognised it as one."*
- **A-2 has FOUR distinct variants, each with a DIFFERENT discovery procedure.** This is the
  single sharpest constraint the corpus puts on any "stale-ref flag":
  1. **proper** — a cited premise expired → found by *re-checking the citation* (`:707-731`).
  2. **never true** — false when written, *"had no way to be checked, because a claim about
     bit ranges has no gate"* → found only by ***computing* the claim** (`:759-778`).
  3. **unstated-assumption** — *"an unstated assumption that another constant in the same
     system would never change… **a constant checked against a number that was itself
     unchecked**"* → found by *asking what the anchor is anchored to* (`:811-820`).
  4. **assembled by symmetry** — *"**no premise to check and nothing to compute**… found only
     by **tracing what a shipped world executes**. Sibling call sites are not evidence about
     each other"* (`:851-868`).
  5. *(and a fifth, on a TEST)* — *"the premise — 'erosion is fast enough for this test to
     mean anything' — **was never written down, and it was false the whole time**"* (`:822-839`).
- **The general answer to A-2 the corpus itself states** (`:783-788`): *"when a justification
  asserts a property **the language could enforce instead**, the durable move is to make the
  property structural and delete the claim."* Measured instance: 26 hand-rolled salts across
  three files → one list per crate where *a duplicate salt is a `const` assertion failure and
  a duplicate name is a duplicate type.*
- **§ 3's own indictment** (`:1043`): `DeepField::chapters` — *"the field's doc comment has
  said 'exported and read by nothing' for longer than this index has existed, and three
  sweeps added rows for its two immediate neighbours in the same struct without noticing it.
  **A self-declaring comment is not an index** — that is the whole premise of § 3,
  demonstrated against § 3."*
- **§ 4 and § 5 both bind the AUDITOR.** § 5 (`:1125-1126`): *"Left as a **flag to the main
  session**, not a unilateral rewrite (§ 4's rule binds the auditor too)."* § 3's storage
  corollary: *"whether it earns its own S-number is a question for the main session, not the
  auditor."* **A sweeper may not change the taxonomy it sweeps against.**
- **A-7 is a non-ratifiable defect class**, explicitly distinguished from a carve-out
  (`:1140-1143`): *"Unlike other deviations it does not go to § 4 for the user's blessing,
  because **there is no world in which it is correct**. It is a defect."*
- **⚠ § 5 IS THE MOST IMPORTANT SINGLE FINDING IN THIS NOTEBOOK — see 3.6.**

### 3.3 `.claude/skills/doc-topology/SKILL.md` — read in full

- **The gap it fills is stated as a triad**: `spine-audit` = docs vs **code**; staleness sweep
  = entries vs **newer work**; doc-topology = docs vs **each other**.
- **`:23-27` — "Why grep cannot do this": *"Grep returns what you already suspected. A
  contradiction between two documents is exactly the thing nobody suspects — if anyone had,
  it would already be resolved. This sweep must READ, not search, and its unit of work is a
  *pair* of statements, not a file."*** ← the unit of work is already **not the document**.
- Shape 6, added after run one: **"a summary that outran its source"**, with a *finite,
  mechanical* procedure — *"for each read-first doc, list what it paraphrases from elsewhere,
  and diff the paraphrase against its source."*
- **Governance:** *"Do not resolve a contradiction you find. Which side wins is frequently a
  user call — **that is precisely why it survived**."* And *"Provenance decides weight:
  user-originated constraints are data; assistant-originated ones are hypotheses that
  happened to survive."*
- Shape 3 is the **one** shape where grep is correct, *because the target is a marked block*
  rather than an unsuspected pair. The skill says so explicitly after run one flagged the
  tension. **Marked → greppable; unmarked → unreachable.**

### 3.4 Git history — measured 2026-07-28

| measure | value |
|---|---|
| commits | **776 in 11 days** (69 / 105 / 82 / 81 / 98 / 34 / 65 / **147** / 88 / 5 / 2) |
| commits touching **only** `.md` | **386** |
| commits touching only `crates/` | 104 |
| mixed | 133 |
| `ROADMAP.md` | **272 commits — 35 % of all commits in the repo** |
| `journal/corrections.md` | 68 · `docs/spines.md` 63 · `docs/design/stubs.md` 49 |
| `.claude/skills/session-workflow/SKILL.md` | **38** · `CLAUDE.md` **31** |
| `spines.md` changes alongside `crates/` | 30 of 63 (**48 %**) |
| `CLAUDE.md` changes alongside `crates/` | 6 of 31 (**19 %**) |

**⚠ THE CHURN MEASUREMENT — and it reframes the problem.** Added vs later-deleted lines:

| doc | +added | −deleted | % of adds ever deleted |
|---|---|---|---|
| `journal/corrections.md` | 2,533 | 30 | **1 %** |
| `.claude/skills/session-workflow/SKILL.md` | 933 | 18 | **2 %** |
| `docs/design/ideas.md` | 609 | 14 | **2 %** |
| `docs/design/flow.md` | 830 | 24 | **3 %** |
| `docs/design/material-behavior.md` | 1,068 | 32 | **3 %** |
| `docs/ARCHITECTURE.md` | 594 | 26 | **4 %** |
| `docs/design/north-star.md` | 418 | 17 | **4 %** |
| `docs/spines.md` | 1,336 | 112 | **8 %** |
| `CLAUDE.md` | 430 | 42 | **10 %** |
| `docs/design/stubs.md` | 1,336 | 162 | **12 %** |
| **`ROADMAP.md`** | 8,761 | 4,257 | **49 %** |

**Every design doc is append-only IN PRACTICE, without anyone having decided that.** Only the
ROADMAP is genuinely revised. The journal is append-only by charter; the *design layer* is
append-only by behaviour.

### 3.5 Citation practice and existing vocabulary — measured 2026-07-28 (tracked files only)

**What the corpus actually cites** (193 `.md` files, 58,608 lines):

| address form | count |
|---|---|
| `§ <section>` | **1,381** |
| `journal/NNNN` | **1,326** |
| spike id (`S9`, `S17`…) | 1,130 |
| dated stamp `2026-MM-DD` | **1,462** |
| `S-N` / `A-N` (spine shapes) | 511 |
| `foo.rs:123` | 808 |
| `::test_or_fn_name` | 625 |
| `corrections #N` | 354 |
| `stubs #N` | 113 |
| **bare `docs/….md` path (the whole document)** | **154** |

→ **Sub-document, stable-id citation outnumbers whole-document citation by roughly 35:1. And
the de facto version stamp already exists and is a DATE (1,462 uses).**

**Existing relational / epistemic vocabulary, all in free prose:**

`heir` **654** in 129 files (the densest relation — a *forward* obligation: "this stand-in is
owed a replacement by X") · `RATIFIED` 525 · `DECIDED` 418 · `falsified` 154 · `HYPOTHESIS`
118 · `DEFERRED` 114 · `RETIRED` 93 · `SUPERSEDED` 92 · `NEEDS RATIFICATION` 40 · `STRUCK` 33
· `user-originated` 25 · `assistant-originated` 12.

### 3.5b Four findings generated 2026-07-28 by the residuals slice — including one by accident

The mechanical-residuals agent (merged 2026-07-28) produced evidence about **address
stability** that was not available before, one item of it by breaking something itself.

- **⚠ `file:line` IS A SELF-BREAKING ADDRESS, DEMONSTRATED LIVE.** The agent's first pass
  reflowed two fixes onto extra lines, shifting `ROADMAP-history.md` by **+1 line from `:496`
  onward** — which **silently broke the live citation at `ROADMAP.md:570`**, the coal bullet
  that addresses `ROADMAP-history.md:1228/1234`. It caught this itself and reflowed both fixes
  back so the file's line count is byte-identical to `main`. *"A cleanup that breaks a live
  line-number citation is not a cleanup."*
  **This is the sharpest available fact about address granularity: `file:line` is the corpus's
  second-most-common precise address (808 uses, § 3.5) and ANY edit above a cited line silently
  invalidates it — with no error, no diff at the citing site, and nothing that can fail.** It
  was preserved here by one agent's diligence inside one run. It is not preserved in general.
- **⚠ THE CORPUS'S ONLY VERSION STAMP — THE DATE — DEMONSTRABLY DRIFTS.** The agent found the
  MFD slice's date restated three ways: `ROADMAP.md:1123` and `:1129` say *"FALSIFIED
  2026-07-25"* / *"MFD SHIPPED 2026-07-25"*; `ROADMAP-history.md:138` dates the same slice
  (journal/0109) **2026-07-26**, and `:185` puts corrections #54 with it.
  **§ 3.5 measured 1,462 date stamps and concluded the version already exists and is a date.
  This says that version is hand-typed, unvalidated, and has already drifted by a day across
  three artifacts.** It is doc-topology shape 4 (*one number, several values*) applied to the
  one field the corpus uses as a version — which is materially worse than an ordinary number
  drifting, because supersession is decided by date comparison (`DECIDED 2026-07-26` beats
  `DECIDED 2026-07-22`). *Not filed as a correction: it is a pair to reconcile, not a falsified
  claim.*
- **⚠ A READ-FIRST SKILL STILL DESCRIBES A SHIPPED THING AS A PROPOSAL — 4 DAYS LATER.**
  `.claude/skills/session-workflow/SKILL.md:889-894` presents archive-by-status as a future
  proposal (*"**Moving** older Shipped entries to a history file … **would** shrink the live
  board"*); `ROADMAP-history.md` shipped 2026-07-26. **This is doc-topology shape 6 (a summary
  that outran its source) live in the highest-blast-radius artifact class**, and it is **T1
  caught in the act in the process layer itself**: the skill *accreted* the proposal and never
  retracted it, exactly as the churn measurement predicts (`session-workflow` deletes **2 %**
  of what it adds). Handed to the integrator, not edited by the sweeper (§ 4's rule).
- **A CAREFUL HAND-COUNTED INVENTORY UNDERCOUNTED BY 76 %.** The doc-topology audit reported
  *"~25"* dangling pointers (16 + 9 + 1). The real count is **44 pointers at 43 sites** —
  23 / 19 / 1 + 1. Two whole classes were missed, one of which is *the live board's own
  archived close block pointing past the end of the file*. And **4 of the 44 were already
  directionally wrong BEFORE the split** (`§ Sequenced` has sat *above* `§ Shipped` since
  `34d88f2`); the archive only made them unresolvable rather than merely wrong.
  **Relevance:** the audit that produced this count was the corpus's *best* instrument, run
  deliberately, by a reader told to count. An inventory of addresses maintained by reading is
  off by a factor of ~1.8 — which is an argument about **who counts**, not about that agent.

### 3.5c ⚠ THE DECIDING MEASUREMENT — where the falsifier actually was, all 65 corrections

**Coding rule** (stated so this is re-runnable and challengeable): for each correction, ask
*"at the moment the false claim was recorded, where did the thing that would refute it already
exist?"* Six exclusive-ish buckets; entries with two real causes are counted in both and the
double-counting is declared.

| bucket | meaning | entries | share |
|---|---|---|---|
| **SAME-ARTIFACT** | the refutation was in the *same document or file* as the claim | **#19, #35\*, #40, #53, #54, #58, #65** (clear) | **~7 / 65 ≈ 11 %** |
| **CODE** | the refutation was in the source the claim described — reachable by reading or grepping code, no experiment | **#2, #10, #20, #33, #38, #42, #44, #45, #47, #49, #64**, partly #6, #62, #63 | **~14 / 65 ≈ 22 %** |
| **DISTANT DOC** | the refutation was in a *different* document the author had no particular reason to open | **#35** (journal/0055 had built the mechanism 11 days earlier — *"nobody went back to re-read the filings"*), **#59** (`k_transport` was *already named on the ROADMAP as due for recalibration*), and the four-way synthesis half of journal/0119 | **~3 / 65 ≈ 5 %** |
| **MEASUREMENT** | nothing in the repo could have said; it required running an experiment | #4, #5, #7, #8, #9, #14, #15, #17, #21, #22, #23, #24, #25, #26, #27, #29, #30, #31, #32, #34, #36, #37, #39, #41, #43, #46, #50, #52, #55, #60, #63b | **~30 / 65 ≈ 46 %** |
| **THE USER** | required the user's eye, memory, or domain knowledge | #3, #11, #13, #18, #44, #47, #48, **#62** | **~8 / 65 ≈ 12 %** |
| **LITERATURE** | required an anchor from outside the project entirely | **#56** — *"It took an anchor from outside the corpus to see it"* | **1 / 65 ≈ 2 %** |

*(\*#35 is counted in both SAME-ARTIFACT and DISTANT DOC: the expired justification sat in the
doc comment, the mechanism that expired it sat in a journal entry.)*

**Aggregate: roughly HALF (~30/65) of our diagnosed failures had their falsifier already
present in the repo — and of those, the overwhelming majority were in the SAME ARTIFACT or in
the CODE, not in a distant document.**

#### What this does to the design field — the numbers, not opinions

- **A "surface the related documents" mechanism targets the ~5 % band.** Three of sixty-five.
  That is not nothing — #35 and #59 were both expensive — but it is the smallest band measured,
  and it is a *fifth* the size of the SAME-ARTIFACT band and a *ninth* the size of MEASUREMENT.
  **Any design that spends its first effort on retrieval is optimising the 5 %.**
- **The largest corpus-addressable band is SAME-ARTIFACT + CODE ≈ 33 %** — and neither is a
  retrieval problem. In every SAME-ARTIFACT case the refuting text was already on the author's
  screen (#58: forty lines · #65: two sentences · #53: two subsections · #19: the author's own
  quotation from hours earlier). **What was missing was not access. It was an obligation to
  reconcile before writing.**
- **MEASUREMENT (~46 %) is unreachable by any knowledge mechanism** — but it is *not* unreachable
  by a knowledge mechanism that records **which instrument licensed a claim and what that
  instrument is blind to.** Eleven of these are B2 (instrument structurally blind to its
  question), and in #62 *"two probes and a gated assertion agreed with each other because they
  were the same instrument."* None of them recorded their instrument's blind spot beside the
  number. That is a **recordable** fact about an unrecordable failure.
- **LITERATURE is 1 of 65 and it was the single most expensive error in the project's history**
  (~10³ scale error, every internal check green). It is already answered by a CLAUDE.md rule —
  and that rule is C1 prose, so under T4 it is expected to decay.

**⚠ Limitation, stated:** this is a **single-coder classification** of 65 entries, coded from
their own accounts, by the same assistant whose errors most of them record. It is directionally
robust — the 5 % band would have to be wrong by ~6× to overtake SAME-ARTIFACT+CODE — but the
bucket assignments are contestable and several entries are genuinely multi-causal. **The coding
rule is stated above precisely so a second reader can re-run it and disagree.**

### 3.5d `journal/0119` read at source — and it corrects my own summary

Declared in § 3.7 as a gap I had reached *only through summaries*. Read in full 2026-07-28.
Three things the summaries did not carry:

- **The vocabulary implied machinery that did not exist.** *"There is no revision-token
  **mechanism**. A revision token is a resource axis, declared and read exactly like `Climate`
  or `Routed`. It is a **naming convention inside a closed enum**"* (`:67-70`). A whole
  architecture was reasoned about as though `DeepAxis` were a subsystem. **A named thing reads
  as a built thing** — the mirror of spines' *"an unnamed pattern is one an author re-derives."*
- **A STRUCTURED INVENTORY ENTRY CARRIED A FALSE *ENUMERATION*, AND NOTHING CHECKS
  COMPLETENESS.** `stubs.md` #29 listed *"four phases run after incision and can lower a cell:
  weathering, hillslope creep, wave attack and eolian deflation."* **Weathering cannot lower a
  cell** (`*r -= q; *h += q`, `surf` unchanged to the bit) — and reading the phase order to
  check turned up *"a fifth post-incision phase nobody had listed at all — **isostasy**"*
  (`:34-36`). So the corpus's **most structured artifact** — the one with numbers, heirs and
  blast radii — held an enumeration that was both **wrong** and **incomplete**, and it took a
  human's offhand question to find. **An enumeration is a claim about completeness, and nothing
  in this corpus checks completeness of anything.** No band above covers this: it is not a
  stale ref, it is a *missing member*.
- **It sharpens my § 4.3, and I was slightly wrong.** `:172-176`: *"the sketch, the
  reconciliation, the refutation, and the unbuilt axis… **they live in four documents and one of
  them is seven thousand lines**."* So 0119 is **both**: the *refuting pair* was co-located (two
  sentences, journal/0090) while the *four-way synthesis* was distributed. My § 4.3 said "the
  refutation was already open" — true of the pair, **not** true of the synthesis. Corrected in
  § 4.3.
- Its own closing verdict, which is the mandate for this whole notebook (`:178`):
  ***"That is the honest diagnosis, and it is not solved by anyone reading harder."***

### 3.5e `wrap` and `spine-audit` skills read in full — and a live open contradiction

- **`wrap/SKILL.md` is a 13-item checklist whose header is the whole point:** *"**Every check
  below exists because it was MISSED once.** Work down the list; each item names what it costs
  to skip."* Eight of thirteen items carry an explicit *"Earned \<date\>"* provenance line.
  **This is the corpus's single largest accumulation of pure C1 — thirteen rules asking a reader
  to remember thirteen things — and under T4 it should decay.** It has a measured decay already:
  § 6 exists because two agents collided on journal `0091`; the *identical* collision then
  recurred in `stubs.md` (**both filed a stub #19**) **after** the rule existed, because the
  rule had been scoped to one artifact and the failure simply moved to another numbered
  inventory. *A rule that names the instance rather than the class decays at the first
  neighbour.*
- **⚠ A LIVE OPEN CONTRADICTION I FOUND BY READING, not by grep.** `spine-audit/SKILL.md:47-52`
  instructs the auditor: *"**Grep `JUSTIFIED-BY:`** and, more broadly, comments justifying a
  design by citing a constraint."* § 3.6 measures `JUSTIFIED-BY` at **3 occurrences in 2 files,
  zero in `crates/`** — so check #4 of a read-first skill directs every future auditor to grep
  a marker that does not exist. `spines.md` § 5 **flagged exactly this on 2026-07-24** and
  deliberately left it as *"a flag to the main session, not a unilateral rewrite"*. **Four days
  later the skill still carries the instruction.** This is T1 in the process layer for the
  second time today (cf. § 3.5b's `session-workflow:889-894`): the flag was *appended*, the
  instruction was *not retracted*, and the honest deferral became indistinguishable from
  neglect. **Both halves are now overdue and both belong to the main session.**
- Governance is consistent across all three sweep skills and worth naming as a **found
  invariant**: `spine-audit` *"Do not add a spine on your own authority… the file's own § 4 rule
  binds the auditor too"* · `doc-topology` *"Do not resolve a contradiction you find"* ·
  `spines.md` § 5 *"left as a flag to the main session, not a unilateral rewrite."* **The corpus
  has already decided, three times independently, that the sweeper may not mutate the taxonomy
  or adjudicate the conflict it finds.** Any mechanism proposed here inherits that constraint.

### 3.5f ⚠ THE T2 FALSIFIER RUN — where additions LAND inside a doc (measured 2026-07-28)

§ 4.4 owed this: file-level churn cannot distinguish **pure append at the end** from **in-place
amendment that only adds**. Measured every hunk in every commit touching each doc, as a
fraction of that revision's own file length.

| doc | hunks | mean position | in last 10 % | in first 80 % |
|---|---|---|---|---|
| `ROADMAP.md` | 564 | 0.46 | 9 % | **90 %** |
| `docs/spines.md` | 90 | 0.51 | **3 %** | **84 %** |
| `CLAUDE.md` | 37 | 0.48 | **3 %** | 78 % |
| `docs/design/north-star.md` | 14 | 0.41 | 14 % | 79 % |
| `docs/design/geology.md` | 18 | 0.60 | 17 % | 72 % |
| `docs/design/flow.md` | 23 | 0.61 | 13 % | 61 % |
| `docs/design/water.md` | 17 | 0.64 | 18 % | 59 % |
| `docs/design/material-behavior.md` | 27 | 0.59 | 30 % | 56 % |
| `docs/design/stubs.md` | 68 | 0.66 | 16 % | 46 % |
| `docs/ARCHITECTURE.md` | 17 | 0.71 | 12 % | 41 % |
| `.claude/skills/session-workflow/SKILL.md` | 40 | 0.76 | **48 %** | 42 % |
| `docs/design/ideas.md` | 22 | 0.81 | **41 %** | 18 % |

**T2 is not falsified. It is sharpened, and the sharpened form is more useful than the
original.** Edits are **distributed through the body**, not appended at the end — mean position
0.41–0.81, and in `spines.md` / `CLAUDE.md` / `ROADMAP.md` only **3–9 %** of hunks touch the
final tenth. Combined with the 2–4 % deletion rate (§ 3.4):

> **T1′ · The corpus is amended IN PLACE, AT THE CLAIM SITE, ALMOST PURELY BY ADDITION.**
> Authors *do* go to the right place. They add a strikethrough, a `SUPERSEDED 2026-07-26`, an
> amendment paragraph — **and leave the superseded text sitting there.**

**⚠ This is the exact mechanism behind "a summary that outran its source" (B3 / doc-topology
shape 6), and it explains why that shape is undetectable.** The doc-topology skill prescribes:
*"for each read-first doc, list what it paraphrases from elsewhere, and **diff the paraphrase
against its source**."* But **the source still literally contains the old sentence** — it was
never removed, only annotated nearby. So the diff *succeeds*: doc Y's paraphrase still matches
text present in doc X. **The check the corpus designed for this shape cannot fire, because
in-place additive amendment preserves the very string the paraphrase was made from.** That is a
mechanical, measured reason — not a diligence problem.

**And the table splits the docs into two maintenance regimes, which nobody has named:**

- **TIMELINE-STRUCTURED** — `session-workflow` (48 % tail, mean 0.76) and `ideas.md` (41 %,
  0.81). Organised as **dated strata**: `## Proven practice additions (2026-07-21, session 5)`,
  `## …(2026-07-22)`, and so on. New practice is appended as a new stratum and **nothing ever
  revisits an earlier one.** This is precisely where § 3.5e's four-day-stale
  archive-as-proposal text lives (`:889-894`) — inside a dated stratum, structurally
  unreachable by the doc's own growth pattern.
- **BODY-AMENDED** — `spines.md` (3 % tail, 84 % in first 80 %), `CLAUDE.md` (3 %, 78 %),
  `ROADMAP.md` (9 %, 90 %). Genuinely maintained throughout, and these are exactly the three a
  recurring sweep or a wrap ritual touches.

**The two read-first process artifacts sit in DIFFERENT regimes** — `CLAUDE.md` is body-amended
and swept; `session-workflow` is timeline-accreted and is not. Both are loaded by every session
and every agent. That is a structural explanation for why the two live contradictions found
today (§ 3.5e) are both in the *timeline-accreted* one and its companion skill.

### 3.6g `north-star.md` read (§§ core/plugin boundary → Deviations) — the sharpest live instance

- **⚠ NEGATION-AT-A-DISTANCE: one section voids three named sibling sections, and none of them
  knows.** § Deviations #2 (`:387-401`, user, emphatic) declares: *"So the **capability-tiering**
  in § 'The core / plugin boundary', § Passes (*'field passes … trusted … first-party'*), and
  § Refinement (*'a global solver is a trusted-tier capability'*) is **EXPLICITLY NOT THE MODEL**
  and must not shape any design."*
  **I then read § Passes at source (`:186-187`): it still says *"Native-backend and first-party
  — because they are trusted and hot."* Unstruck. No marker. No pointer to § Deviations.**
  So a reader arriving at § Passes — which is what `corrections #65` names as a *cited site* and
  what every brief consults — reads live design that a later section of the same file voided.
  *(doc-topology finding #12 flagged this class; confirmed at source, still open.)*
  > **This is a NEW structural type the § 3.5c buckets do not name: a one-to-many negation edge
  > that exists only at the negating end.** It is exactly the *chain of authority* the user
  > described — and here the chain is one hop long and already broken, inside the single
  > highest-blast-radius document in the corpus.
- **⚠ T6's SECOND INDEPENDENT INSTANCE, and it is a two-column enumeration.**
  `### 🔴 THE REFINEMENT TIER IS IN NEITHER LIST — OPEN, named 2026-07-26 (user)`:
  *"**Neither column above mentions the refinement / collapse tier**, and it was never decided —
  **it was arrived at *by default***… In code it is `collapse.rs`: **2,415 lines of ordinary
  engine code with essentially zero pass-shaped declaration.**"* The core/plugin boundary is the
  corpus's most load-bearing enumeration and **an entire tier was missing from it**, found by
  the user asking a question. With `stubs.md` #29's missing isostasy (§ 3.5d) that is **two
  independent T6 instances in the corpus's two most structured claims, both found by a human
  noticing, neither detectable by any existing control.**
- **⚠ AND THE ASYMMETRY THAT MAKES T6 DAMNING.** § *Validation by construction* (`:306-313`):
  *"This generalizes today's `build_checked` (**a world refuses to build if a class a pass
  selects from has zero members**). 'By construction compatible' is not a slogan; it is that
  check."* **The CODE layer has an existence check on its enumerations and refuses to build
  without it. The DOC layer has none.** The project already believes in exactly this control —
  it just never applied it to the documents that specify the code.
- **An explicit PRECEDENCE edge between two sections of one document** (`:196-199`):
  *"Refinement (2026-07-23)… **Sharpens the core/content boundary above; where the two disagree,
  this governs.**"* The corpus authors conflict resolution in prose when it knows two sections
  overlap. Measured corpus-wide: `governs` 26 / 17 files · `supersedes` 24 / 14 ·
  `outranks` 7 / 6 · `takes precedence` **0**.
- **Ratification has GRANULARITY, and the corpus marks it** (`:126-127`): *"**Three candidate
  answers** (assistant-framed 2026-07-26; **the user has chosen the direction, not the
  mechanism**)."* So a node can be *direction-ratified* while its mechanism is open — a state
  no simple ratified/unratified flag can hold.
- **`OPEN EDGE` is a real forward-pointer convention** — `> 🔖 OPEN EDGE — see <notebook> § 2`
  at `:58`, pointing at the live thread that *pushed past* the section. **7 occurrences in 7
  files.** It is the inverse of a supersession edge: it warns from the *stale* end.

### 3.6h ⚠ T4′ CONFIRMED INSIDE A SINGLE RELATION — `heir` vs `HEIR:`

The cleanest possible test of T4′, because it holds the relation constant and varies only
whether the author is asked to *restate* it in a marker:

| form | what it is | occurrences |
|---|---|---|
| **`heir`** — the bare word, in prose | the natural expression of the thought | **651** (129 files) |
| **`HEIR:`** — the labelled marker form | a restatement of the same relation in a notation | **48** (13 files) |

**7.4 % adoption for the labelled form of the corpus's densest relation.** Same relation, same
authors, same files. The only variable is whether it must be restated as a marker. And the two
markers that *did* achieve adoption fit T4′(a) exactly: **`STUB #` (58 / 22 files)** is a
stub's *identity* — you cannot refer to one without it — and SKILL frontmatter is read by the
harness at selection time.

### 3.6i `spines.md` S-4…S-8 + A-2's new entry + A-3 — the C3 inventory, and T6's third instance

- **⚠ SIX C3 INSTANCES EXIST, AND THE CORPUS HAS A NAME FOR THE PATTERN: *"structural, not
  disciplinary"*** (S-4, `:295-297`). Full inventory:
  | # | mechanism | what became unrepresentable |
  |---|---|---|
  | 1 | `true_surface_m → Option<f64>` (#10) | a miss sharing a type with an answer |
  | 2 | `Providers` slots → `Option<fn>`, `None` = identity (#32) | asking "is this slot supplied" by address |
  | 3 | `Identity::Unrecorded` (#49) | *unrecorded* sharing a value with *empty* |
  | 4 | `Domain` / `Draws` + `draw_domains!` (#52) | a duplicate salt (a `const` assertion failure) or a duplicate name (a duplicate type) |
  | 5 | **`CoarseField<T>`** (S-4, journal/0075) | **the raw per-cell read — it does not type-check, proven by a `compile_fail` doc-test** |
  | 6 | **`EdgeId::declared`** (S-8, journal/0108) | **naming an undeclared `(material, form) → (material, form)` transition at all** |
  - **#6 is the closest in-repo precedent to what a knowledge layer would need**, and S-8 says
    so in its own words: *"a fact **structurally cannot name an undeclared transition**… This is
    also **S-6's rule applied to a data axis** rather than to pass order: what edges exist is
    **declared**, never whatever a caller happened to pass."* The project has already built
    *"you cannot assert an undeclared relation"* — **for material transitions, not for claims.**
- **⚠ T6's THIRD INSTANCE, and it is in the CODE's own declarations.** A pass's `reads` set is
  an enumeration, and nothing checks it against the body:
  - **`Exposed` is declared and never read** (`:409-416`) — *"Over-declaring a read is safe (it
    only adds order) but it is a **false statement on a self-declaring pass**."*
  - **`erosion.current_chapter()` is read by passes that never declare it** (`:427-430`) —
    *"Transitively ordered, so not behavioural — but it is a **project-wide idiom, not an
    exception**."*
  - Both were found by **a human reading the body line by line against the declaration.** So the
    self-declaring architecture has systematic under- and over-declaration that is invisible
    precisely because it is behaviourally harmless — the same shape as a stale doc claim.
- **A CO-RETIREMENT edge type, which no `heir` pointer can express** (S-5, `:365-367`): *"pin
  the two halves of the resulting calibration (`COAL_ONSET_C` and the identity) as retiring
  **together** — an heir that lands one without the other is a **world-scale defect**, not a
  drift."* An obligation graph therefore needs at least `heir(A) → B` **and**
  `must-discharge-with(A, B)`.
- **An obligation can carry a GUARD against its own obvious fix** (S-4, `:330-336`):
  *"**Live violation:** `regolith_at_voxel` samples NEAREST while `surface_at_voxel` beside it is
  bilinear… **Before "fixing" nearest→bilinear, read `field.rs:374-383`:** nearest is a forced
  trade-off… The fix must route around that, not through it."*
- **S-5 gives the one existing OBLIGATION LEDGER WITH A DENOMINATOR:** *"34 seams inventoried;
  **5 converted**."* That is layer 3 working — for exactly one obligation class — and it is the
  only place in the corpus where an outstanding count is stated.
- **"Hand-rolled four times before it was named"** (S-5, `:344`) — the *third* independent
  statement of the naming law, after S-2's *"implemented three times under three names"* and
  S-2's corollary *"an unnamed pattern is one an author re-derives."*
- **A-2's newest entry (added by today's removal slice) is the sharpest sentence in the file**
  (`:881-887`): the ROADMAP said the goldens would move, corrections #64 said they were
  *"structurally downstream of the posts"*, and **not one moved** — *"**Structurally downstream
  is a statement about the call graph; whether a fingerprint moves is a statement about which
  chunks the sampler visits.** The same reflex — reasoning from the shape of the code instead of
  from what it executes — produced both halves, **six lines apart, in the entry that named the
  reflex**."*
- **A-3's generalisation** (`:921-922`): *"**a thing which can fail must be run by the gate, and
  'green' is only ever a claim about the code that actually ran**."* And a note worth more than
  it looks: on the retiring-scaffolding case, *"**the assertion did not catch this — the
  dead-code lint did**."* **The strongest control in that instance was the one nobody wrote.**

### 3.6j `docs/spikes/` + `docs/audits/` — 30 files, ~10,000 lines, and a read-first contradiction

Structural census rather than a full read (these are *measured-number* artifacts; their format is
what bears on this notebook). **14 of 30 carry zero supersession/staleness marker of any kind.**

**⚠ THE VERIFIED FINDING — and I narrowed it twice by re-reading at source, which is the point.**
My first inference was *"falsified spike docs carry no pointers"*; a keyword count said 2–12 hits
each, so I read the hits. Most are incidental. What survives is specific and clean:

- **`S10-results.md` carries a cost table whose headline is ~2× the real production cost, and a
  user ratified a ship decision on it.** corrections #12 measured 15.17→25.19 s as actually
  10.82→13.79 s (the spike drove the *scalar* path; production takes the byte-identical
  **parallel** one). And corrections #12 states the policy in as many words:
  > *"S10-results.md is **left unamended** — a spike result is a dated record of what was
  > measured; **this entry is the pointer**."*
  **Read at source, `S10-results.md` contains no reference to corrections #12.** Its only two
  matches for *falsif\** are incidental remarks about biology tests.
- **Meanwhile `CLAUDE.md` read-first item 5 says:** *"Spike results live in
  `docs/spikes/S*-results.md` — **measured numbers, don't re-guess them**."* And `spines.md` § 6
  says the same of audits. **So the read-first instruction grants these files authority and
  sends readers to the end of the pointer that does not exist.**
  > **The corrections policy treats a spike as immutable TESTIMONY; the read-first instruction
  > treats it as live AUTHORITY. Both are reasonable and they are incompatible, and the corpus
  > has never noticed because each is stated in a different file.**
- **`S2-results.md`: zero markers**, and today's removal slice made its entire implementation
  zero-consumer (§ 3.6k). The spike whose code just died carries no note.
- **The GOOD shapes exist and prove the omission is a choice, not a limit:**
  `S9-results.md:296` carries its own `## Falsified assumptions (→ corrections.md candidates)`
  section — a spike self-reporting its own falsifications. And `S19-flow-record-cost-results.md`
  got a one-line `> RESOLVED 2026-07-25` note, which `spines.md`'s A-2 entry explicitly calls
  *"exactly the right treatment, and **why the omission is visible**"* in its sibling `S17`.

**⚠ AND THIS IS THE SECOND INSTANCE OF A CONFIRMED STRUCTURAL TYPE.** § 3.6g found
*negation-at-a-distance* in `north-star.md` (§ Deviations #2 voids three named sibling sections;
none of them carries a marker). This is the same shape across files: **the edge exists only at
the correcting end.** Two instances, both in artifacts CLAUDE.md marks read-first.
> **A one-directional pointer is not a pointer. It is a note to whoever already found the
> answer.**

### 3.6k Today's removal slice (journal/0121, corrections #66) — evidence generated live

The bootstrap-content removal produced three findings that bear directly on this notebook:

- **corrections #66 — a PRE-AUTHORISED expectation, falsified.** The ROADMAP said *"the goldens
  will move and that is correct"*; corrections #64 said the fingerprints are *"structurally
  downstream of the posts"*. **Not one golden moved.** Measured, not inferred: 0 wood voxels in
  `contents_contract`'s sample set pre-removal, and `geology`'s sampler covers `cz ∈ [−20, 24]`
  while the nearest post sits at `cz = −727`. The filed lesson: ***a pre-authorised golden move
  is indistinguishable from an unexplained one, which is the opposite of caution.***
  **This is a new sub-shape of A-2: not a justification that outlived its premise, but a
  PERMISSION that outlived its justification — and was never true.** Falsifier band: **CODE**
  (§ 3.5c), confirming that band.
- **A whole ratified subsystem was reached by exactly one caller, and nothing said so.**
  `pregen/history.rs` was the **only production caller of dc-sim's entire S2 statistical tier**;
  removing it leaves `engine::{query, observe, force_fact}`, `Ledger` and `ToyWorld` with **zero
  production callers workspace-wide.** This is `DeepField::chapters` **at subsystem scale** — a
  layer-2 (capability index) miss that `spines.md` § 3 exists precisely to catch, found by an
  *implementation* slice rather than by any of the sweeps.
- **The trace's own claim was weaker than it read.** *"Exactly one non-test reader"* was true
  **collectively**; three of the four fields had **no reader at all**, and a third test file the
  trace never named also read all four. *An aggregate statement about a set read as a statement
  about each member* — a small, clean instance of correction #61's shape.

### 3.6 ⚠ THE ADOPTION LAW — three natural experiments already run in this repo

The single most useful thing found. Same question (does a convention get adopted?), three
outcomes, one discriminating variable.

| marker | is a MACHINE consuming it? | adoption |
|---|---|---|
| **`JUSTIFIED-BY:`** — designed *specifically* as a greppable marker, documented in `spines.md` § 5 **and** in the `spine-audit` skill, promising *"a sweep greps these and asks, one by one, whether the cited constraint is still true"* | **NO** — the promised grep was never wired to anything | **3 occurrences in 2 files. ZERO in `crates/`.** § 5's own drift check (`:1117-1126`): *"a corpus grep returns this file and the skill that describes it — nothing in `crates/`… **every A-2 caught so far was caught by READING, not grepping**"* |
| **SKILL.md YAML frontmatter** | **YES** — the harness reads `description` to decide relevance | **100 % — all 5 skill files, and they are the ONLY files with frontmatter in all 193** |
| **`scripts/filesize_hook.py`** | **YES** — a `PostToolUse` hook on `Write\|Edit` | adopted instantly; its own docstring argues the point: *"**WHY A HOOK AND NOT A RULE**… A reminder the harness issues is a mechanism; a line in a doc is not."* |

> ~~**In this codebase, a convention's adoption tracks whether a machine consumes it.**~~
> **🔴 FALSIFIED 2026-07-28, by me, deliberately — see § 3.6b.** This was T4 as first stated
> and it is **wrong**. Five documented conventions with **no machine consumer whatsoever** sit
> at 97–100 % adoption. Machine consumption turns out to be *sufficient* but not *necessary*,
> and stating it as the law would have sent the design straight at the wrong lever.
> *Kept struck rather than deleted: the three-experiment table above is still sound evidence,
> and the corrected law in § 3.6b is built from it plus the falsifiers.*

### 3.6b ⚠ T4 FALSIFIED — the deliberate hunt, and the corrected law

§ 4.4 owed a falsification attempt for T4: *find a documented-but-unconsumed convention that
nevertheless achieved high adoption.* **Five exist, measured 2026-07-28:**

| convention | documented in | machine consumer | adoption |
|---|---|---|---|
| journal filenames `NNNN-slug.md` | CLAUDE.md § The journal | **none** | **118 / 118 = 100 %** |
| screenshot names `NNNN-description` | CLAUDE.md § The journal | **none** | **153 / 155 = 99 %** |
| `> blogworthy:` + which lens | CLAUDE.md § The journal | **none** | **117 / 118 entries = 99 %** |
| `Co-Authored-By: Claude <model>` | CLAUDE.md § Conventions | **none** | **753 / 776 commits = 97 %** |
| **`heir`** (the successor relation) | `stubs.md` doctrine | **none** | **651 uses across 129 files** |
| — versus — | | | |
| **`JUSTIFIED-BY:`** | `spines.md` § 5 **+** `spine-audit` check #4 | none *(a sweep was promised and never wired)* | **3 occurrences, 0 in `crates/`** |

**So the discriminator is not the machine.** All six lack a consumer; five are near-universal and
one is dead. What actually separates them:

| | marginal cost to the author | payoff, and when |
|---|---|---|
| filenames / screenshots | **zero** — the file had to be named *something*; the convention only constrains a form the author could not skip | immediate: the name **is** the artifact's identity |
| `blogworthy`, `heir` | **zero** — it is the word for the thought the author is already having | immediate: it **is** the expression, with no translation step |
| `Co-Authored-By` | zero — it rides inside the commit template, at the instant of committing | immediate, embedded in the act |
| **`JUSTIFIED-BY:`** | **a specific token, in a specific format, restating in a second formal notation something the author had already written in prose** | **deferred, external, and to somebody else** — a sweep, later, for another reader |

> **T4′ · A convention is adopted when it is either (a) INSEPARABLE from an act the author must
> perform anyway, or (b) the NATURAL EXPRESSION of the thought the author is already having. It
> is not adopted when it asks the author to RESTATE, in a second formal notation, something they
> have already said in prose — no matter how good the validator, and no matter what the
> restatement is promised to enable.**
>
> Machine consumption (frontmatter, the filesize hook) is one way to satisfy (a)/(b), because it
> supplies **feedback at write time**. That is what it was actually buying. It is not the law.

**Two consequences, both sharp, both measured rather than argued:**

1. **Any scheme that asks an author to hand-write a version number or a tag beside prose they
   have already written will get `JUSTIFIED-BY` adoption (≈0 %)** — and this codebase has
   already run that exact experiment, with a documented convention, a stated validator, and a
   named sweep behind it. Metadata that must survive here has to be **derived from what the
   author already wrote, or produced by the act itself** (the filename, the commit, the number
   the integrator assigns at dispatch).
2. **⚠ THE CORPUS ALREADY AUTHORS THE GRAPH. NOTHING READS IT.** `heir` is 651 densely-used
   forward obligations — *"this stand-in is owed a replacement by X"* — plus 525 `RATIFIED`,
   418 `DECIDED`, 154 `falsified`, 118 `HYPOTHESIS`, 114 `DEFERRED`, 92 `SUPERSEDED`, 93
   `RETIRED`, 25 `user-originated`, 12 `assistant-originated`, and ~5,000 stable-id citations
   (§ 3.5). **The authoring problem is solved and has been for weeks. The extraction problem
   and the obligation problem are untouched.** That asymmetry, not retrieval, is where the
   measured leverage is.

### 3.6c `stubs.md` — the corpus's one real structured inventory, read for its SCHEMA

29 numbered entries. **A four-field schema, declared in prose at `:10-13` and applied by hand:**
*"Each entry names: **what the rule fakes · the expresser that subsumes it · loudness at audit
time · blast radius**… Update this file in the same commit as any change that adds, removes, or
subsumes a stub."* Plus a doctrine that makes it a *closed-world* claim (`:6-8`): *"a stub must
be known, loud, and listed here with its heir. **An unlisted stub is a defect in this inventory,
not a licence.**"*

What it already does that any proposal must account for:

- **Lifecycle status lives in the heading string** — `RETIRED`, `DISCHARGED`, `NOW A PROVIDER
  SLOT`, `SCOPE FIXED`, `RENAMED AGAIN`, `was UNDOCUMENTED until this audit`.
- **⚠ IT ALREADY VERSIONS NODES, APPEND-ONLY, BY DUPLICATION.** `### 14.` sits beside
  `### 14 (original).`; `### 29.` beside `### 29 (original).` **The superseded version of a node
  is kept in the same file with `(original)` appended.** This is a working, in-use, append-only
  node-versioning idiom — invented ad hoc, never designed, and exactly consistent with T1.
- **The stable id survived three renames.** #29 was *"the-clamp-that-was-green-because-nothing-
  eroded"* → *"the-solve-that-goes-grid-unstable-above-1×"* → *"the-hillslope-conveyor-that-
  checkerboards-the-regolith"*, and the heading records the whole chain plus *"right about the
  symptom, wrong about the mechanism"*. **The number held identity while the name churned** —
  a direct, in-repo argument for opaque stable ids over slugs.
- **It has FOUR node states, not one:** `## Active stubs` · `## Sibling gap (not a substitution
  — an unexpressed ledger term)` · `## Genesis (permanently legitimate — affirmed, not defects)`
  · `## Audited and rejected (real mechanisms or ratified decisions, not stubs)`. **That last
  section is a NEGATIVE record** — a place for *"we looked, and this is not a defect"* — which
  partially answers correction #35's complaint. Only partially: it holds rejected *stub
  candidates*, not decisions-not-to-build in general.
- **`#1 ruin-posts`' heir is now void.** Its recorded heir is *"the social sim + ecology —
  dwarf-fortress-class civilization history"*, and the user ruled 2026-07-26 that no evo/socia/
  civ modelling exists *"even at the design stage: they are NOTHING."* **A dangling heir is a
  new failure mode this inventory has no state for** — the sibling removal slice is resolving
  this one by hand.

**⚠ AND THE MOST IMPORTANT SENTENCE IN THE FILE, for this notebook** (`:22-24`): *"A slot does
not retire a stub; it stops the stub from silently becoming the definition, which is
ARCHITECTURE.md § *A summary is not an authority* **made structural**. **The general registry is
deliberately unbuilt — four conversions is not enough to design one from.**"*

That is **ratified project doctrine directly governing how this very design pass must proceed**,
and it is echoed verbatim in `session-workflow` § Seam-first practice 6: *"**Do not build the
general mechanism first.** Convert the cheapest cold seam, let it teach the shape, convert three
more, *then* generalize. **A registry designed before its callers exist is the same mistake in a
new coat.**"* It is the project's own answer to *"we don't know what we need to know yet."*

### 3.7 Coverage — declared, per the doc-topology precedent

**Read IN FULL today:** `journal/corrections.md` (2,501 ll., all 65 entries + #63b) ·
`.claude/skills/doc-topology/SKILL.md` · `.claude/skills/session-workflow/SKILL.md` (915 ll.,
loaded) · `CLAUDE.md` · `.claude/settings.json` · `scripts/filesize_hook.py` ·
`docs/audits/2026-07-26-doc-topology-sweep.md` §§ ranked-index/18/19/nulls/coverage/verdict.

**Read IN PART:** `docs/spines.md` (head, § 2 A-1/A-2/A-4/A-5/A-6, § 3 full table, § 4, § 5,
A-7, § 6 — **not** S-1…S-9 bodies beyond S-2/S-3, **not** A-3) · `ROADMAP.md` (In-flight head,
§ Sequenced head + 5 entries, both close blocks).

**Measured quantitatively rather than read:** full git history (776 commits), per-file churn,
citation-form census, vocabulary census, frontmatter census.

**NOT OPENED AT ALL — declared so the gap is legible:** every `journal/` entry (0001–0119)
individually, including **0119**, the canonical collapse — reached only through corrections
#65, the ROADMAP and the doc-topology skill, i.e. **through summaries, which is the exact
failure mode under study** · `docs/design/{north-star, material-behavior, flow, water,
tectonics, ideas, stubs, geology, materials, ecology, earth-processes, visuals, light, ores,
bodies, knowledge, octree-substrate, things-that-will-happen}.md` bodies · `docs/ARCHITECTURE.md`
· `docs/API.md` · `docs/rendering/PIPELINE.md` · `.claude/skills/{spine-audit, wrap,
tour-map}/SKILL.md` · all `docs/spikes/*` · all `docs/audits/*` except the one above ·
**`ROADMAP.md` § Observed (~1,970 ll.)** · `ROADMAP-history.md` (2,835 ll.) · `scripts/gate.ps1`.

**This is pass 1 of 2.** Pass 1 targeted the *failure corpus* + the *process layer* + the
*quantitative history*. Pass 2 owes the **design-doc layer** and the **three unread skills** —
and specifically owes an attempt to FALSIFY T4 (below) by hunting for a documented-but-
unconsumed convention that nevertheless achieved high adoption.

**PASS 2 STATUS, 2026-07-28.** Done: `journal/0119` (full, § 3.5d) · `spine-audit` + `wrap`
skills (full, § 3.5e) · `stubs.md` head + full heading structure + schema (§ 3.6c) ·
`ARCHITECTURE.md` heading structure + the 2026-07-26 decision record in full ·
`ideas.md` heading structure + § *Pass cadence* in full (§ 3.6d) · `north-star.md` heading
structure · the **T4 falsification** (§ 3.6b) · the **T2 falsifier run** (§ 3.5f) · the
heading-metadata census · § Observed characterised quantitatively (§ 3.6e).
**Still not opened:** `north-star.md` bodies (§ Compliance, § Deviations, § core/plugin
boundary), `material-behavior.md` § 5 at source, `flow.md`/`water.md`/`geology.md`/
`materials.md` bodies, `docs/API.md`, `PIPELINE.md`, all `docs/spikes/*`, remaining
`docs/audits/*`, `ROADMAP-history.md`, individual journal entries other than 0119, and
`spines.md` S-1/S-4…S-9 bodies + A-3.

### 3.6d ⚠ THE AMENDMENT IDIOM — one convention, converged in FOUR files, never designed

`ideas.md` § *Pass cadence* read at source (`:528-595`) is the corpus's most-amended single
node, and it shows the whole idiom in one place. **One section carries:**

- heading provenance + date: `## Pass cadence — the fractional-phase scheduler (user sketch, 2026-07-23)`
- an outbound supersession edge: `**RECONCILED 2026-07-24 → material-behavior.md §5**`
- **three separate inline strikethrough amendments**, one of which *reverses* an earlier one:
  `~~two orthogonal axes~~ **three**` · `~~topo-sorted ORDER (derived from reads/writes)~~
  **ORDER — AUTHORED, PER WORLD**` · `~~not yet reconciled~~ (reconciled, above)`
- a dated blockquote amendment with three citations: `> ⚠ THE ORDER HALF OF THIS SKETCH WAS
  RESTORED 2026-07-26 (user; ARCHITECTURE.md § …; corrections #65)`
- **the original sketch retained verbatim below** — *"Sketch retained below as the origin."*

**It is a hand-maintained CRDT, and it works** — the entire three-day history of the most
expensive design thread in the project is reconstructable from one section. **And the same
idiom has now been converged on independently in four files:** `stubs.md` (`### 29` beside
`### 29 (original)`) · `ideas.md` (inline strikethrough + dated blockquote) ·
`material-behavior.md` § 5 (struck bullet + `🔴 SUPERSEDED <date> (user)`) · and `flow.md`
§ 11.1 — where today's residuals agent explicitly reported copying
*"`material-behavior.md`'s strike shape **deliberately rather than inventing a second
convention**."*

**Four authors, one convention, never designed, never named.** By spines' own law — *"an
unnamed pattern is one an author re-derives"* — **the amendment idiom is itself an unnamed
re-derived pattern**, and the fourth author only avoided a fifth variant by noticing.

**And the governance invariant fired on the highest-stakes node in the corpus** (`:553-555`):
> *"This note was written only after the user confirmed the wording: correcting the doc that
> holds the user's own design is itself the move the new supersession rule guards against,
> **which is why the sweep flagged it rather than fixing it**."*

### 3.6e The decision-record schema has been IMPROVING — the 2026-07-26 entry

`ARCHITECTURE.md:488-568` (*The engine is plugin-agnostic, and pass ORDER is authored*) is the
richest node schema in the corpus, and it is the newest. Its sub-structure:

`## <claim> — DECIDED <date> (<who>)` · the user's **verbatim quote** · **"The product this
serves"** (the root premise *"because every argument below descends from it"*) ·
**`### What this indicts today`** · `### The two models, and why the second one wins`
(alternatives) · **`### The claim that was replaced, and the sentence that refutes it`**
(explicit supersession + the refuting citation + `corrections #65`) ·
`### What is engine-owned and what is not` (boundary) ·
**`### RATE — ratified 2026-07-24, never built, and now first in line`**.

**Two of those fields are forward OBLIGATIONS, authored in prose and watched by nothing:**
*"what this indicts today"* names `DeepAxis` as the violation — **still present** — and
*"ratified, never built"* names RATE — **still unbuilt**. This is correction #35's complaint
answered halfway: the decision *does* carry its obligation list; nothing discharges it.

### 3.6f `ROADMAP.md` § Observed — characterised quantitatively rather than read (1,970 lines)

Declared above as the largest unswept surface. Measured instead of read:

- **1,970 lines · 128 top-level entries**
- **only 37 of 128 (29 %) carry a date on the entry line** — the other 91 are undated
- lifecycle appears as prose adjectives *inside bodies*: `RESOLVED` 17 · `DIAGNOSED` 13 ·
  `FIXED` 11 · `DECIDED` 10 · `falsified` 9 · **`SUPERSEDED` 0**
- **no stable ids at all** — unlike `journal/NNNN`, `corrections #N`, `stubs #N`, `S-N`/`A-N`

> **§ Observed is the least addressable region of the corpus: no ids, 71 % undated, lifecycle
> recorded as adjectives in prose.** It is also, by the archive's own reasoning, the one
> section that *cannot* be archived by status — an Observed entry is live by definition. So the
> part of the board that must stay readable is the part with the least structure, and roughly
> 60 of its 128 entries already contain a word suggesting they are resolved.

---

