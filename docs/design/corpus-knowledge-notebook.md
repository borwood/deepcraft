# Corpus knowledge notebook — how we work, and how our failures fall out of that shape

**Status: OPEN NOTEBOOK. Nothing here is decided.** Opened 2026-07-28 at the user's
direction. This file is the *evidence ledger* for a design pass that has not happened yet.

**The user's instruction that governs this file** (2026-07-28, verbatim intent):
> *"the above sketch is not law and requires your help to do deep reading and analysis of our
> git history, skills, process-encoded-in-docs, to form a triple-checked defensible theory of
> our how we work in this codebase and how our diagnosed failures fall out of that shape. i
> will not accept any immediate design opinions until the systematic deep reading is done by
> you personally… (a theory that does not habitually cycle through empiricism fails this task:
> you can understand how eagerness to pick a logic in the face of an impossible to read corpus
> is a known bad pattern here)"*

So: **§ 1 is the user's sketch (user-originated = data). § 2 is priors already in the corpus.
§ 3 is the evidence ledger, one row per artifact actually read. § 4 is theory, and it stays
EMPTY until the reading is done.** Any claim in § 4 that cannot name a `file:line` in § 3 is
inadmissible.

---

## 1. The user's sketch — USER-ORIGINATED (data, not hypothesis)

Recorded verbatim-in-substance so a later analysis of mine cannot quietly reconcile it away
(CLAUDE.md § *A user-originated design may not be superseded by an implementation slice*).

> Build a **knowledge graph with versioned nodes** and a **sanctioned means of interacting
> with it** — probably scripts bundled with a skill — such that we can:
> - do **validation on tags**;
> - **automatically record the version of other ref'd nodes at node write/edit** (only nodes
>   ref'd *in the diff*, preserving old ref versions);
> - **automatically flag stale refs at write / edit / read**;
> - run **periodic coherence sweeps that catch stale refs programmatically across the corpus**.
>
> Target properties:
> - *"when we work on any one feature, we are automatically aware of what bears on it and what
>   of that knowledge is possibly stale and where to read to authority."*
> - *"we're aware of **chains of authority** where one ref is stale, but the new version of the
>   ref'd doc also has stale refs, and so on: i should know the tree and when i arrive at the
>   **core ancestors** i should have maximally failure-free modes of **percolation of updated
>   concepts**."*
> - Aim: **make our failure modes impossible by construction, in a measurable way.**
> - *"core ontology for proposed solution is at this point open to creative proposals argued
>   from grounded theory. we dont know what we need to know, yet: that's the first and ongoing
>   challenge."*

**The sketch's own named risk, which is also this notebook's:** *eagerness to pick a logic in
the face of an impossible-to-read corpus is a known bad pattern here.*

---

## 2. Priors — what the corpus already holds (swept before opening the pass)

Per session-workflow § *Sweep the corpus BEFORE opening a design pass*: priors first, my own
observations after, demoted to what survived the sweep.

- **No prior art on tagging/frontmatter/model-versioning exists.** Grep over all 192 `.md`
  files for `frontmatter | yaml header | tags: | standing model | model version | versioned`
  returns only: the ROADMAP entry that books this very conversation, plus unrelated uses
  (`materials.md:4` versioned sidecar chunk sections; `north-star.md:302` mods as versioned
  artifacts; `PIPELINE.md:7,135` a versioned pipeline contract; `S20:498` a versioned derived
  artifact; `corrections.md:353` S11 save-versioning). **This is a green field.**
- **Corpus size, measured 2026-07-28:** 192 `.md` files, **58,340 lines**. Largest:
  `ROADMAP.md` 4,599 · `ROADMAP-history.md` 2,835 · `journal/corrections.md` 2,501 ·
  `docs/spines.md` 1,224 · `docs/design/stubs.md` 1,172 · `material-behavior.md` 1,036 ·
  `tectonics.md` 953 · `session-workflow/SKILL.md` 915 · `water.md` 809 · `flow.md` 806.
- **Three docs-ops controls already exist, and the corpus already separates the three failures
  they address** (`ROADMAP.md:621-655`): **VOLUME** (archive; "third in value"),
  **TOPOLOGY** (the `doc-topology` sweep; *"this is the one that cost an architecture"*),
  **AUTHORITY** (one CLAUDE.md rule; *"cheapest and highest-value of the three"*).
- **The archive's own honest limit is on the record** (`ROADMAP.md:650-655`): 4,410 lines is
  still past reading whole, and **neither half of it would have prevented journal/0119.**
- **The sweep's own verdict** (`docs/audits/2026-07-26-doc-topology-sweep.md:566-611`): five of
  the top eight findings were **unsuspected**, i.e. unreachable by grep by construction; the
  rules that did the work were *"read whole sections, do not grep-and-conclude"* and
  *"`file:line` on both sides"*.
- **The sweep proposed a sixth contradiction shape: *a summary that outran its source***
  (`:579-586`) — a paraphrase of doc X living in doc Y, where X was later amended in place and
  Y was not. *"The summary was never wrong when written and no decision was ever reversed —
  which is exactly why no reviewer catches it."* Four of its top eight were this one mechanism.
- **The single most load-bearing prior for "by construction"** — CLAUDE.md § Gates:
  > *"**'Re-run the probes by hand after a merge' was tried here and FAILED IN ONE DAY.**…
  > **The assertion caught it; the process did not.** Do not answer 'the gate cannot see X'
  > with a rule asking people to remember X."*
  Replaced by a *mechanism* (`test = true` on the example's Cargo target). **This is the
  project's own precedent that an advisory rule is not a control.**
- **Only ONE mechanically-enforced corpus control exists today** (`.claude/settings.json`): a
  single `PostToolUse` hook on `Write|Edit` running `scripts/filesize_hook.py`. Everything
  else in CLAUDE.md and the four skills is **advisory prose**. And its thresholds are
  *"still the hook's provisional guesses, not the user's numbers"* (close block, § Owed).

---

## 3. Evidence ledger → [`corpus-knowledge-evidence.md`](corpus-knowledge-evidence.md)

**Split out 2026-07-28** at the filesize hook's prompt (the notebook hit 1,026 lines against a
1,000-line threshold). The evidence is a genuinely separable concern: it is *read once, cited
often*, while §§ 1/2/4 are the live argument. Nothing was dropped in the move.

*Noted because it is on-topic: the one mechanically-enforced control in this repo fired on the
document arguing that mechanically-enforced controls are the ones that work (§ 3.6). It also
demonstrated its own doctrine — it did not edit anything, it asked at write time.*

## 4. Theory

**Status: PROVISIONAL after pass 1. Every claim below cites § 3. Nothing here is a design
proposal.** Per the user's constraint, this section characterises the problem field; it does
not pick a mechanism.

### 4.0 The characterization — three layers, three different states of health

The corpus is **not** a knowledge base that decayed. It is a **high-velocity, in-place,
additive amendment log** that is *excellent* at recording what happened and *absent* at
discharging what is owed. It has three layers and they are in wildly different condition —
and the corpus itself says so, in one sentence, at `spines.md:69-72`:

> *"**Decisions were findable** this session; DECIDED entries **did their job**. **Built
> machinery was not findable.**"*

| layer | what it holds | state | evidence |
|---|---|---|---|
| **1 · DECISION** | what we chose, why, when, on whose authority | **HEALTHY, and improving** | 418 `DECIDED` / 525 `RATIFIED`, dated and provenance-marked; superseded in place with the refuting citation; the newest record (§ 3.6e) carries its root premise, its alternatives, its supersession *and* its own indictment list |
| **2 · ARTIFACT / CAPABILITY** | what exists, what consumes it | **PARTIALLY REPAIRED, still leaks** | `spines.md` § 3 was built for exactly this and works (a row *departed* when FF2b consumed it) — yet `DeepField::chapters` sat unlisted through **three** sweeps while its neighbours in the same struct were added |
| **3 · OBLIGATION** | what is *owed*: heirs, indictments, ratified-but-unbuilt, negative decisions, expiring premises, acceptance criteria, enumeration completeness | **ABSENT** | 651 `heir`s · 29 stub heirs (one now **void**, § 3.6c) · *"what this indicts today"* → `DeepAxis` still present · *"ratified, never built"* → RATE still unbuilt · #35: *"a decision not to build gets a paragraph in a doc comment and **no watcher**"* · T6: no enumeration is ever checked for completeness |

> ### The one sentence
> **We have a decision log, a partial capability index, and no obligation ledger. Our expensive
> failures are overwhelmingly UNPAID OBLIGATIONS, not unfindable facts — and the obligations
> are already written down, in prose, in the same sentences that create them.**

### 4.0a ⚠⚠ THE RESULT THAT RE-AIMS EVERYTHING — staleness is ~8 %

Added 2026-07-28 after § 3.5g, which asked the sharper question: **for a watcher to fire, a change
event must exist — so did each claim *become* false, or was it false the day it was written?**

| | share |
|---|---|
| **BORN-FALSE, and checkable at the time** | **~85 %** (~55 / 65) |
| **BORN-FALSE but unreachable until a magnitude changed** | ~8 % (#29, #57, #60, `mfd_routing`'s guard, #51's guard) |
| **GENUINELY STALE** — true when written, later falsified | **~8 %** (#35, #11, #42, #59, #7) |

> **A staleness watcher has an ~8 % ceiling in this corpus, by construction — not because it
> would be badly built, but because for 85 % of these failures THERE IS NO CHANGE EVENT TO WATCH.**

**This is a second, independent reason `JUSTIFIED-BY` was never worth adopting**, and it converges
with § 3.5c from the other direction: that coding found the refuting text was already on the
author's screen (SAME-ARTIFACT 11 %, CODE 22 %); this one finds it was *already true* on the day.
**Two codings, two questions, one conclusion: the binding failure is at the moment of ASSERTION,
not at the moment of DECAY.**

**And it explains the shape of every control that has actually worked here.** All six C3 instances
(§ 4.2b) are **write-time impossibilities, not staleness alarms**: once `Identity::Unrecorded`
exists you cannot *write* `has_contents` as a chunk fact; once `draw_domains!` exists you cannot
*write* a duplicate salt; once `EdgeId::declared` exists you cannot *write* an undeclared
transition. **The project has been solving the 85 % all along — in code, and never once in the
corpus.**

**The corpus does, however, already have one write-time control for CLAIMS, and there is measured
evidence it works.** Marking a claim's epistemic status: `HYPOTHESIS` **118** uses / 33 files ·
`NEEDS RATIFICATION` 40 · `DEFERRED` 114 · `assistant-originated` 12. The proof is correction
**#63**, which credits its own author for it: *"flagged as a hypothesis by its own author and
falsified the same day… **which is why this correction cost two probe runs instead of a fix
slice**."* And **#63b never entered a doc at all** because it was labelled a hypothesis in
conversation. **A claim that carries its own epistemic status costs less when it turns out to be
wrong — measured, twice, in one day.** It is applied entirely by hand and entirely unevenly.

**⚠ INDEPENDENTLY RE-CODED 2026-07-28 — and it moved my numbers against me** (§ 3.5h; full audit
`docs/audits/2026-07-28-corrections-recoding.md`). Dispatched with this file and the evidence
ledger named as **forbidden reading**, my counts withheld; it confirmed it opened neither.

- **Rule 2 CONFIRMED and pushed further:** it found **GENUINELY STALE = 2 (3.0 %)**, not my ~5
  (8 %), keeping only #11 and #35 — both git-verified — and documenting a **no-true-era** for
  #7, #12, #27, #32, #42, #57, #66, several of which I had called stale. **My staleness count was
  an overcount in the direction that flatters tooling.** The ceiling is **3–8 %**, not 8 %.
- **Rule 1 differs materially, mostly in my favour:** **DISTANT DOC is ONE entry, and it calls
  even that a bad fit** (I had 3). CODE **36 %** vs my 22 %; MEASUREMENT **37 %** vs my 46 %; and
  it derives **46.3 % primary / 65.7 % with secondaries needed no experiment at all**, against my
  ~33 %. **It believes MORE was catchable from the repo than I did.**
- **⚠ AND 29 OF 67 ENTRIES (43 %) ARE COIN-FLIPS.** The BF-U/BF-C line *"drawn one plausible notch
  differently **doubles the UNREACHABLE count**."*

> **Both codings agree on every load-bearing DIRECTION and disagree on MAGNITUDE by up to ~2×,
> with 43 % of the file ambiguous. So: the directions are robust; the percentages are not.**
> Every figure in § 4 is a band. *This is "quote the measure, never the threshold" applied to my
> own analysis rather than to a slice's — and the honest form of that discipline here is to say
> the numbers cannot bear more weight than a direction.*

**It also found five entries that misdescribe themselves, four of which I verified directly** —
including that **corrections #40 is itself a misdiagnosis**, now filed as **corrections #67**: the
seam audit quoted the comment *character-for-character as it stood the day before* (`11d4385`,
whose own message calls the prior text *"lying"*), so the A-2 **was live** and #40's author had
read a *later* revision. **A stale READ, not a stale claim.** And **#56 still asserted, unstruck,
exactly what #60 withdraws, with no `#60` token anywhere in the file** — the sharpest instance of
§ 4.1c's one-directional edge, inside `corrections.md` itself. Both repaired 2026-07-28.

**And it named a shape neither rule captures that matters more than either total: CLAIMS WITH NO
ARTIFACT** (#31, #55, #56, #63b) — *"no differ can fire, and #56 is the most expensive entry in
the file."* **This is B4 confirmed as structural: an implicit premise has no address, so NO
mechanism — tag, version, watcher, differ — can ever attach to it.** Plus **ARMCHAIR DERIVATION**
(7–8 of 25 MEASUREMENT rows were refutable with a pencil), which shrinks the unreachable band
again; and the absence of **severity** and **belief-holder** from both taxonomies.

**The § 3.5c bands map onto the layers exactly, which is the test of this framing:**

- **DISTANT DOC ≈ 5 %** → layer 2 residue. Findability. The smallest band.
- **SAME-ARTIFACT ≈ 11 % + much of CODE ≈ 22 %** → layer 3. Every one is an *unmet obligation
  to reconcile or to check the source before asserting*, with the refuting text already on
  screen (#58 forty lines · #65 two sentences · #53 two subsections · #19 the author's own
  quotation from hours earlier · #64 *"the give-away was already written down and went
  unread… the caller passes `vec![]` twice on adjacent lines"*).
- **MEASUREMENT ≈ 46 %** → a *fourth* thing, and layer 3 still reaches it: none of these
  recorded **which instrument licensed the claim and what that instrument is blind to**. In
  #62 *"two probes and a gated assertion agreed with each other because they were the same
  instrument."* That blind spot is a recordable obligation about an unrecordable failure.
- **USER ≈ 12 % / LITERATURE ≈ 2 %** → outside any corpus mechanism, and #56 (the ~10³ error)
  is already answered by a C1 prose rule that T4′ predicts will decay.

**And why layer 3 has no mechanism, mechanically rather than morally** — the three measured
properties compose into it:

1. **T1′** amendments are in-place and additive, so **the superseded text is never removed** —
   which is why doc-topology's own shape-6 check (*diff the paraphrase against its source*)
   **structurally cannot fire** (§ 3.5f).
2. **T2′** the corpus addresses sub-document by stable id ~35:1, and versions by **hand-typed
   date** — a field that has already drifted by a day across three artifacts (§ 3.5b). So the
   only version we have is unvalidated, and supersession is decided by comparing it.
3. **T4′** every attempt to fix this by asking authors to *restate* something in a formal
   notation gets `JUSTIFIED-BY` adoption (3 occurrences, 0 in code) — while five conventions
   that cost nothing extra sit at 97–100 %.

**T5 is the consequence and the opening:** the obligations are *already authored*, densely,
in an idiom four authors independently converged on (§ 3.6d) and nobody named. **The gap is
extraction and discharge, not authoring.**

### 4.0b What the project's own doctrine says about how to proceed from here

Not my opinion — ratified, and it is the direct answer to *"we don't know what we need to know
yet"*:

- ~~`stubs.md:22-24` — *"**The general registry is deliberately unbuilt** — four conversions is
  not enough to design one from."*~~ **WITHDRAWN 2026-07-28 (user) — the clause was removed from
  `stubs.md` and never governed this thread.** Nobody proposed a registry; it was an inference,
  and a **provider seam's success condition is that it DISAPPEARS** (the heir replaces it with a
  field or a pass — `burial_temp_c`, journal/0093, the only completed case). *So one of the two
  pillars § 4.0b rested on is gone.* **The other stands and is sufficient** — see below.
- `session-workflow` § Seam-first, practice 6 — *"**Do not build the general mechanism
  first.** Convert the cheapest cold seam, let it teach the shape, convert three more, *then*
  generalize. **A registry designed before its callers exist is the same mistake in a new
  coat.**"*
- And the governance invariant, converged on **three** times independently (§ 3.5e): the
  sweeper may not mutate the taxonomy or adjudicate the conflict it finds.

**So: any ontology proposed here is itself subject to "do not build the general mechanism
first."** The honest first move is to take a small number of *real* obligations already in the
corpus — a `heir`, an indictment, a ratified-but-unbuilt, a void heir — and see what a
mechanism would have to know to discharge them. That is the seam-first march applied to the
knowledge layer, and it is what this project does to every other subsystem.

### 4.1 Four properties of the shape, each measured

- **T1 · ACCRETION WITHOUT RETRACTION.** Design docs delete 2–4 % of what they add (§ 3.4);
  only `ROADMAP.md` is genuinely revised (49 %). **So contradiction here is not produced by
  replacement — it is produced by addition.** The refutation gets appended; the claim is left
  standing. That is exactly the observed geometry: 400 lines apart (journal/0119), **40 lines
  apart** (#58), **two sentences apart** (#65), *same paragraph* (#54). **The steady state of
  an append-only corpus is self-contradiction**, and no amount of volume reduction changes
  that — which is why the archive *"would not have prevented journal/0119"* by its own
  admission (`ROADMAP.md:650-655`).
- **T2 · ADDRESS/LIFETIME MISMATCH.** The corpus cites sub-document stable ids over whole
  documents ~**35:1** (§ 3.5), and already versions by **date** (1,462 stamps). Meanwhile the
  thing that *goes stale* is neither the document nor the section — it is a **claim and its
  premise**, and a claim has no address at all. Consequence: **a document-level version would
  bump on every append (T1) and therefore carry almost no information about whether any
  particular claim is still true.** A staleness signal built on it would fire constantly and
  mean nothing — which is correction **#62**'s failure mode exactly (*a ranking test wearing a
  magnitude test's clothes*; a guard that saturates as the condition generalises).
- **T3 · ONE MEDIUM FOR MANY KINDS OF CLAIM.** Twelve of 65 corrections (B5 + B10) are
  **category errors across kinds**: schedule-vs-physics (#54), naming-vs-magnitude (#55),
  units (#56), shape-vs-identity (#58), ranking-vs-magnitude (#62), threshold-vs-coefficient
  (#59), derivation-vs-construction (#65), quote-vs-paraphrase (#13), landmark-vs-pose (#48),
  number-vs-address (#28), cost-vs-code-path (#12), name-vs-environment (#51). Prose has **one
  type**, so a claim about *when* and a claim about *how much* are the same kind of sentence
  and **nothing can reject the confusion.** This is the identical defect the code layer
  already beat with types (§ 4.2 C3) — **one tier up, and unsolved.**
- **T4′ · THE ADOPTION LAW, corrected after falsifying my own first version (§ 3.6b).**
  *~~A convention is adopted iff a machine consumes it~~* — **false**: five documented
  conventions with no consumer sit at 97–100 % (journal filenames 118/118 · screenshot names
  153/155 · `blogworthy` 117/118 · `Co-Authored-By` 753/776 · `heir` 651 uses). The real law:
  **a convention is adopted when it is inseparable from an act the author must perform anyway,
  or is the natural expression of the thought the author is already having; it dies when it
  asks the author to restate in a second formal notation something already said in prose.**
  `JUSTIFIED-BY` (3 occurrences, 0 in `crates/`) is the pure case of the latter, *with* a
  documented convention, a stated validator and a named sweep behind it. Machine consumption is
  sufficient because it supplies **write-time feedback** — that is what it was buying, not
  authority. Compatible with the corpus's own doctrine: *"do not answer 'the gate cannot see X'
  with a rule asking people to remember X"* (CLAUDE.md § Gates, after that advisory **failed in
  one day**) and *"a rule in a docstring cannot fail a build"* (#32).
- **T5 · THE CORPUS ALREADY AUTHORS THE GRAPH; NOTHING READS IT.** 651 `heir` obligations · 525
  `RATIFIED` · 418 `DECIDED` · 154 `falsified` · 118 `HYPOTHESIS` · 114 `DEFERRED` · 92
  `SUPERSEDED` · 93 `RETIRED` · 25 `user-originated` / 12 `assistant-originated` · ~5,000
  stable-id citations (§ 3.5) · a hand-applied 4-field schema over 29 stub nodes with four node
  states and an ad-hoc append-only node-versioning idiom (`### 29 (original)`, § 3.6c) ·
  multi-site assertion sets recorded as edges **at correction time** (§ 3.1). **The authoring
  problem is solved. The extraction problem and the obligation problem are untouched.** Any
  proposal that asks authors to author *more* is working on the solved half — and T4′ says it
  will get 0 % anyway.
- **T6 · NOTHING IN THE CORPUS CHECKS COMPLETENESS OF AN ENUMERATION.** `stubs.md` #29 listed
  four post-incision phases; one of them (**weathering**) *cannot do what the entry claims* and
  a fifth (**isostasy**) *was missing entirely* (§ 3.5d). This was in the corpus's **most
  structured artifact**, and it was found by a human's offhand question. **An enumeration is a
  claim about completeness; none of the five failure buckets in § 3.5c covers a missing member,
  and no sweep, test, or convention looks for one.**

### 4.2 The control-class ladder, from the corpus's own repeated trials

| class | verdict, in the corpus's words |
|---|---|
| **C1 prose rule** | **Fails, fast.** #29 *"prose cannot fail a build"* · #32's docstring rule was itself retired · the probe advisory *"FAILED IN ONE DAY"* · `JUSTIFIED-BY` 0 % |
| **C2 assertion/test** | Works **only when the assertion can see the question** — and B2 (11 instances) is the list of times it could not (#46 hand-fed magnitude, #51 wrong world, #62 saturating predicate) |
| **C3 make the illegal state unrepresentable** | **Strongest.** #32: *"the constraint did not get enforced, **it stopped existing**"* · #10: *"when 'no answer' and 'an answer' share a type, the silent path is the one that ships"* · #49 `Identity::Unrecorded` · #52 *"disjointness is a **compile-time property** rather than an assertion in prose"* · spines `:783-788` states it as the general answer to a whole anti-shape class |
| **C4 harness hook** | one instance, adopted instantly, thresholds still provisional |
| **C5 recurring human sweep** | Finds what nothing else can (**19 pairs, 5 of top 8 unsuspected**) — **and exists because C1 failed**, and demonstrably misses (`chapters` × 3 sweeps) |

### 4.3 ⚠ THE LOAD-BEARING AND MOST UNCOMFORTABLE FINDING

**Our worst failures were not RETRIEVAL failures.** In every one of the corpus's own most
expensive cases the refutation was *already open in front of the author*:

- #54 — *"the document contained both halves"* (one document).
- #58 — *"the two statements are in the same document, **forty lines apart**"*, and the
  falsifier was **the slice's own unit test, two lines below the assertion it contradicted**.
- #65 — *"the refutation was already in the corpus, **two sentences into the entry that
  celebrated the replacement**."*
- #19 — *"**I had the falsifying fact in my own context, from this same session**"* (the
  integrator had quoted the contradicting S4 result verbatim hours earlier).
- journal/0119 — *"all in the corpus, **all findable by grep**… no reader ever had all four in
  view at once."*
  - **⚠ CORRECTED 2026-07-28 after reading 0119 at source (§ 3.5d).** I had listed this as a
    pure in-view case. It is **both**: the refuting *pair* was co-located (two sentences apart,
    journal/0090) while the *four-way synthesis* — sketch · reconciliation · refutation ·
    unbuilt axis — *"live in four documents and one of them is seven thousand lines."* So 0119
    supports the obligation reading for its refutation half **and** the retrieval reading for
    its synthesis half. **This is the one case where retrieval genuinely mattered, and it is
    counted in the DISTANT-DOC band in § 3.5c accordingly.** Recording the correction rather
    than quietly restating it, because reaching 0119 through summaries and then finding the
    summary had flattened it is *this notebook committing the failure it studies.*

**A mechanism that surfaces related documents would have surfaced documents that were already
open.** So retrieval is *not* where the evidence says the leverage is. Optimising it first is
**correction #61 applied to this problem**: *a criterion over the axis that did not break
cannot license a claim about the axis that did.*

What WAS binding, in order of measured cost:

1. **A claim has no address, so it can carry no obligation.** The three most expensive claims
   of the last week were **never written as sentences at all** (#55 *"Never written as a
   sentence, which is why it survived"*; #56 the same words; #31). **No tagging scheme can tag
   an unwritten premise** — and #35 names the resulting hole in the ontology precisely: *"a
   stub gets an inventory entry and an heir; **a decision not to build gets a paragraph in a
   doc comment and no watcher**."* The corpus has 654 `heir` edges for stand-ins and **zero**
   node type for a negative decision, an expiring premise, or an acceptance criterion.
2. **Obligations are not one kind.** A-2's four-to-five variants need four-to-five *different*
   discovery procedures (§ 3.2): re-check the citation · **compute** the claim · ask what the
   anchor is anchored to · **trace what a shipped world executes** · exercise the magnitude
   the test assumes. **A single "stale ref" flag cannot express any of these**, and a scheme
   that flattens them into one will find variant 1 and miss the four that cost the most.
3. **Verification, not knowledge, is the weakest layer — and the knowledge layer can still
   help.** B2 (11) + B7 (3) are instruments structurally unable to see their question; #62 is
   the purest (*"two probes and a gated assertion agreed with each other **because they were
   the same instrument**"*, and a person flying the terrain was right in one sentence). A graph
   cannot fix a blind instrument. It *could* record **which instrument licensed a claim and
   what that instrument is blind to** — which is precisely what none of them recorded.

### 4.1b The obligation types the corpus already authors — an inventory, not a design

Layer 3 is absent as a *mechanism*, but its **content** is authored densely and in at least
seven distinguishable kinds. Recorded as an inventory because "obligation" is not one thing, and
a scheme with one arrow will catch one of these:

| kind | how the corpus writes it today | count / instance |
|---|---|---|
| **heir** — a stand-in owed a successor | the bare word `heir`, in prose | **651** uses / 129 files (the labelled `HEIR:` form: **48**, § 3.6h) |
| **indictment** — a decision naming what currently violates it | `### What this indicts today` | `ARCHITECTURE.md:501`; `DeepAxis` **still present** |
| **ratified-but-unbuilt** | `### RATE — ratified 2026-07-24, never built` | RATE, **still unbuilt** |
| **co-retirement** — two things that must discharge *together* | prose: *"pin the two halves as retiring **together** — an heir that lands one without the other is a **world-scale defect**"* | `spines.md` S-5 (`COAL_ONSET_C` + its identity) |
| **guard on an obligation** — a warning against its own obvious fix | *"**Before "fixing" nearest→bilinear, read `field.rs:374-383`**: nearest is a forced trade-off"* | `spines.md` S-4 live violation |
| **void heir** — an obligation whose successor was ruled out of existence | nothing; found by hand | ~~`stubs.md` #1's heir was *"the social sim + ecology"*, and the user ruled those **NOTHING** 2026-07-26~~ **⚠ THE SOLE INSTANCE IS NOT AN INSTANCE — corrected 2026-07-29 (baseline sweep S4/F6).** `CLAUDE.md` (user, 2026-07-28): *"**THE STATUS IS ON HOLD, NOT NEVER**… we do want these systems **eventually**… Read 'they are NOTHING' as a statement about **what exists**, never about what is wanted."* `stubs.md:64-69` already carries the correct reading. So this row's **successor is DEFERRED, not void**, and this obligation kind may have **zero** instances — while **S4/F4's re-homed-address kind** (`stubs.md` #4's heir named a provider slot that was deleted) has at least one and is not in this inventory at all. *This matters because the row is one of seven kinds § 5's reciprocity proposal argues from.* |
| **negative decision** — a reasoned choice *not* to build | nothing. #35: *"a stub gets an inventory entry and an heir; **a decision not to build gets a paragraph in a doc comment and no watcher**"* | charcoal's *"cannot exist"*, false for 11 days |

**And exactly one of the seven has a working ledger with a denominator:** `spines.md` S-5 —
*"34 seams inventoried; **5 converted**."* That is the only place in the corpus where an
outstanding obligation count is stated. It is also the only obligation class nobody complains
about losing.

### 4.1c Two structural types confirmed by two independent instances each

- **⚠ THE ONE-DIRECTIONAL EDGE (negation-at-a-distance).** A correcting artifact names what it
  voids; **the voided artifact carries no marker.** (i) `north-star.md` § Deviations #2 declares
  the capability-tiering in three *named* sibling sections *"EXPLICITLY NOT THE MODEL"* — and
  § Passes still reads *"trusted … first-party"*, unstruck (§ 3.6g). (ii) `corrections #12`
  declares itself *"the pointer"* for `S10-results.md`'s ~2×-wrong cost table, and `S10` contains
  no reference to it — while CLAUDE.md read-first item 5 sends every session to `S10` and tells
  them not to re-derive its numbers (§ 3.6j). **Both live, both in read-first artifacts.**
  > **A one-directional pointer is not a pointer. It is a note to whoever already found the
  > answer.** And it is the direct mechanical cause of the *chains of authority* problem: a chain
  > cannot be walked from the stale end if the stale end holds no link.
- **⚠ T6 — NO ENUMERATION IS EVER CHECKED FOR COMPLETENESS. Three instances, three tiers.**
  (i) `stubs.md` #29's four post-incision phases: one **could not do what was claimed**
  (weathering), and a fifth (**isostasy**) was **missing** (§ 3.5d). (ii) `north-star.md`'s
  two-column core/plugin boundary: **an entire tier absent**, *"never decided — arrived at by
  default"* (§ 3.6g). (iii) A pass's `reads` set: `Exposed` **declared and never read**;
  `current_chapter()` **read and never declared**, *"a project-wide idiom, not an exception"*
  (§ 3.6i). **All three found by a human noticing. And the asymmetry is damning:** the code layer
  *has* this control — `build_checked` **refuses to build a world if a class a pass selects from
  has zero members** — and `north-star.md` § *Validation by construction* calls it the whole
  meaning of the phrase. **The project believes in existence-checking its enumerations and has
  never applied it to the documents that specify them.**

### 4.2b The C3 ladder is not a hypothesis here — six instances, and a name

`spines.md` S-4 calls it ***"structural, not disciplinary."*** Full in-repo inventory:
`Option<f64>` (#10) · `Option<fn>`/`None`=identity (#32) · `Identity::Unrecorded` (#49) ·
`Domain`/`Draws` where a duplicate salt is a `const` assertion failure (#52) · **`CoarseField<T>`
whose raw per-cell read does not type-check, proven by a `compile_fail` doc-test** (S-4) ·
**`EdgeId::declared`, which makes an undeclared transition *unnameable*** (S-8, journal/0108).

**The sixth is the closest precedent to anything a knowledge layer would need**, and S-8 states
the generalisation itself: *"**S-6's rule applied to a data axis** rather than to pass order:
what edges exist is **declared**, never whatever a caller happened to pass."* The project has
already built *"you cannot assert an undeclared relation"* — **for material transitions, and for
nothing else.**

### 4.3b ⚠ RECIPROCITY, MEASURED — and my first instrument was broken

Because 4.1c claims the one-directional edge is the mechanical cause of the chains-of-authority
problem, that claim is measurable. **First attempt reported `0/67` corrections cited from
anywhere else, which is impossible** — `spines.md` and `CLAUDE.md` visibly cite corrections by
number. Cause: `\b` in a `git grep -E` pattern, which POSIX ERE does not support. **Recorded
because it is this notebook's own subject matter: absence in a tool's output is a claim about
the tool** (anti-shape A-5), and a 0 % that flattered the thesis was one publication away.

**Corrected measurement (2026-07-28, tracked files):**

| | |
|---|---|
| distinct correction numbers cited from **outside** `corrections.md` | **58 of 67** |
| edges where a correction names a file **by full path** | 15 |
| …**reciprocated** (that file cites the correction back) | **7 (47 %)** |
| …**one-directional** | **8 (53 %)** |

**The corpus back-links well in aggregate — 58/67 — so the strike-at-source practice works.**
What fails is the specific case: **where a correction names a particular file by path, about half
of those files never name it back.** In full:

`#12 → docs/spikes/S10-results.md` · `#19 → light.md` · `#20 → tectonics.md` ·
`#24 → roughness_probe.rs` · `#24 → S13-results.md` · `#25 → S13-results.md` ·
`#36 → stubs.md` · `#54 → 2026-07-25-roadmap-staleness-sweep.md`

**`#12 → S10-results.md` is the instance § 3.6j found by reading**, reproduced by an independent
instrument — the ~2×-wrong cost table a user ratified a ship decision on, whose correction
declares itself *"the pointer"* while the spike holds no reference to it.

**⚠ Denominator honesty:** 15 is small because most corrections cite by *bare name*
(`erosion.rs`, `flow.md § 2.6`) rather than full path, so this covers only the full-path subset.
The 53 % is a rate over that subset, not over the corpus. **What the measurement establishes is
that the failure is real, recurring, and mechanically detectable — not its exact frequency.**

---

## 5. First proposal — ⚠ ASSISTANT-ORIGINATED, 2026-07-28, unratified

Marked per the provenance rule: **user-originated constraints are data; assistant-originated ones
are hypotheses that happened to survive.** This is a hypothesis. It is deliberately **not an
ontology**, because ~~`stubs.md:22` and~~ `session-workflow` § Seam-first #6 forbids designing
the general mechanism first, and § 4.0b takes that as binding on this thread.
*(`stubs.md:22` **withdrawn 2026-07-28** — it never governed this thread; see § 4.0b. The
seam-first rule alone is sufficient and is the one that was always doing the work.)*

### 5.1 The user's sketch, re-aimed by the evidence — component by component

| the sketch says | the evidence says |
|---|---|
| **versioned nodes** | The version already exists and is a **date** (1,462 stamps). The unit that goes stale is a **claim**, not a document — and a document-level version would bump on every append (T1′, 97 % additive), producing a signal that saturates: **correction #62's failure mode, in metadata.** *Contested.* |
| **record ref versions at write/edit** | T4′ kills the hand-written form (`heir` 651 vs `HEIR:` 48). **But it survives if DERIVED: git already knows the exact version of every file at every commit.** We never needed authors to write it down; we needed something to read it. *Survives, re-aimed.* |
| **validation on tags** | `JUSTIFIED-BY` had a documented convention, a stated validator, and a named sweep → **3 occurrences, 0 in `crates/`**. Validation is not what drives adoption. *Contested.* |
| **flag stale refs at write/edit/read** | The **mechanically detectable subset is reciprocity** (§ 4.3b), computable today from existing citations with zero new authoring. *Survives, narrowed.* |
| **periodic coherence sweeps** | Already exists (`doc-topology`) — and its shape-6 check **cannot fire** because T1′ leaves the paraphrased sentence in place. *Survives; needs a different predicate.* |
| **chains of authority / percolation to core ancestors** | **The strongest half, and the one-directional edge is exactly why the chain cannot be walked** — a chain has no link at the stale end. *Survives; this is the prize.* |
| **"automatically aware of what bears on it"** | Targets the **5 %** DISTANT-DOC band. *Demoted, not dismissed.* |

### 5.2 The cheapest cold seam — the reciprocity check

**One check, over text that already exists, with a denominator. Not a graph, not a schema, not a
tag.**

> For every artifact that declares it **corrects / supersedes / voids / retires** another *by
> name*: does the named artifact carry a back-pointer?

**Why this one first, argued from the findings rather than from taste:**
1. **It requires zero new authoring** — the sole thing T4′ predicts will actually survive. Both
   ends of the sentence are already written; only the *reciprocity* is unchecked.
2. **It attacks the layer the evidence indicts** — obligation (§ 4.0), not retrieval (5 %).
3. **It is the mechanical cause of the user's own stated goal.** *"I should know the tree, and
   when I arrive at the core ancestors…"* — you cannot walk a tree whose edges exist only at one
   end.
4. **It has a denominator**, which is the single property that distinguishes the one obligation
   class that works (`S-5`: *"34 seams inventoried; 5 converted"*) from the six that don't.
5. **It already has 8 known failures** (§ 4.3b) including one where a user ratified a ship
   decision on a number the correcting entry knew was wrong.
6. **It fits the in-repo C3 precedent.** `EdgeId::declared` made an undeclared transition
   *unnameable*; the analogue is not "tag your refs" but *"a supersession that names a target the
   target does not acknowledge is an **unreciprocated edge**, and the check counts them."*

**What it must teach before anything general is built** — this is the seam's job, not a
deliverable:
- Is the right **granularity** the file, the `§`, or a stable entry id? (§ 3.5 says the corpus
  cites sub-document ~35:1, so the file is probably wrong.)
- Does the predicate need the **kind** of edge? *corrects* / *supersedes* / *voids* / *heir* /
  *co-retires* behave differently (§ 4.1b lists seven), and one arrow will flatten them.
- Can the **amendment idiom** (§ 3.6d, converged in four files) carry a back-pointer cheaply, or
  does adding one cost enough to trip T4′?
- **What is the right response to a detected gap?** Governance says the sweeper may **not**
  adjudicate (§ 3.5e, three independent statements). So the output is a *list*, not an edit.

### 5.2b ⚠ What § 4.0a does to this proposal — it SURVIVES, and for a better reason than I had

The born-false result (~85 %) looks at first like it kills the reciprocity check along with
staleness detection. It does not, and the distinction matters:

> **Reciprocity is not a staleness mechanism. It is a PROPAGATION mechanism.**

The `#12 → S10-results.md` case is the proof. `S10`'s cost table was **born-false** — it never
named the code path that produced it. Nothing changed; nothing decayed. What is missing is that
**a known, correct, already-written falsification never reached the artifact carrying the false
claim.** That is orthogonal to stale-vs-born-false, and it is *strengthened* by the 85 % result:
**if most claims are born false, then corrections are the primary knowledge artifact in this
corpus, and getting them to reach their sites is the primary need.**

**And it is precisely the user's own word for what they want.** *"Percolation of updated
concepts"* — reciprocity **is** percolation, measured: 58 of 67 corrections are cited from
somewhere, and **8 of 15** full-path edges are one-directional, meaning **the correction knows its
site and the site does not know its correction.**

**So the seam is re-aimed, not replaced:** it is not *"detect stale refs"* (8 % ceiling) but
*"a falsification must reach every site it names"* (the 85 %'s only cheap remedy).

### 5.2c The 85 % — what the evidence supports, and what I do not have

**Supported by measurement:** the one write-time control the corpus already applies to *claims* is
**epistemic status**, and correction #63 credits it with converting a fix slice into two probe runs
(§ 4.0a). It is authored in the natural expression (`HYPOTHESIS` 118 · `DEFERRED` 114 ·
`NEEDS RATIFICATION` 40), so it satisfies T4′(b) — **which is exactly why it has adoption and
`JUSTIFIED-BY` has none.**

**What I do not have, stated plainly rather than papered over:** a mechanism that *checks* it. The
detectable form of *"this claim carries no epistemic status"* requires knowing what the claims
**are**, and prose does not delimit them. **This is the hardest open problem in the field.** The
honest position is that the corpus has a *working, adopted, cost-reducing convention* with **no
denominator** — the same condition as six of the seven obligation kinds (§ 4.1b), and the one
property that separates them from `S-5`'s *"34 seams, 5 converted."* **Finding a denominator for
epistemic status is a better problem than designing a tag vocabulary, and I cannot yet say how.**

### 5.3 The second candidate, if the first teaches well

**The completeness check on enumerations** (T6, three instances, § 4.1c) — because the in-repo
precedent is unusually exact: **`build_checked` already refuses to build a world if a class a pass
selects from has zero members**, and `north-star.md` § *Validation by construction* calls that the
entire meaning of the phrase. Applying the project's own existence-check to the documents that
specify the code needs no new philosophy at all.

### 5.4 What I would NOT build first, and why

- **A retrieval / related-docs surfacer** — the 5 % band (§ 3.5c). It is the most *appealing*
  mechanism and the smallest measured payoff.
- **A tag vocabulary with a validator** — `JUSTIFIED-BY` is the experiment, already run, 0 %.
- **A general node/edge schema** — forbidden by ratified doctrine until several real conversions
  have taught the shape, and § 3.6c is explicit that *four* was judged not enough.
- **A doc-level version number** — T1′ + T2′ make it a saturating signal.

### 5.5 ⚠ What contests this proposal

Stated because a proposal without its own falsifiers is the thing this notebook exists to stop.
- **The reciprocity denominator is 15.** If the full-path subset is unrepresentative, the check's
  yield may be much smaller than the reading suggests.
- **Reciprocity is a syntactic proxy for an epistemic property.** A back-pointer proves an edge
  was acknowledged, **not** that the stale claim was fixed. It could become a box-ticking ritual —
  the exact shape of a summary wearing an authority's clothes.
- **It does nothing for the 46 % MEASUREMENT band**, which is the largest. The candidate there is
  recording *which instrument licensed a claim and what it is blind to*, and that is **not**
  derivable from existing text — so T4′ predicts it will fail as an authoring convention. **This
  is the hardest open problem in the field and I do not have a mechanism for it.**
- **The user's sketch may be right and my re-aiming wrong**, in one specific way worth naming: if
  the corpus is about to grow another 5× — which at 776 commits in 11 days is plausible — the
  DISTANT-DOC band may be 5 % *because* the corpus is currently small enough to hold in a few
  heads. **That is an argument from the design-target regime rather than from bring-up
  constants**, which this project takes seriously, and it would promote retrieval.

### 4.4 How to falsify this theory (required — it must keep cycling through evidence)

- **T1 falsified if** a per-section churn measurement shows design docs are substantially
  revised in place rather than appended. *Test:* re-run the churn measure at § granularity,
  not file granularity. **Not yet done.**
- **T2 falsified if** the claims that actually went stale turn out to be document- or
  section-scoped rather than premise-scoped. *Test:* take the 65 corrections and ask, for
  each, what the smallest artifact is whose change would have invalidated it. **Not yet done —
  this is the highest-value single measurement outstanding.**
- **T3 falsified if** the 12 category-error corrections turn out to share a subject-matter
  cause rather than a kind-of-claim cause. *Test:* re-read those 12 at source. **Partly done
  (read in this pass); not adversarially re-read.**
- **T4 falsified if** a documented-but-unconsumed convention in this repo nevertheless
  achieved high adoption. *Test:* pass 2 hunts for one deliberately. **Owed.**
- **4.3 falsified if** a material fraction of the 65 corrections turn out to have had their
  refutation in a document the author had *no* reason to open. *Test:* classify all 65 by
  "refutation was in view / one hop away / genuinely remote." **Owed — and it is the
  measurement that decides how much of the user's sketch is load-bearing.**

---

## 4. Theory *(duplicate heading — see the LIVE § 4 above)*

~~**EMPTY BY DESIGN.** Do not write here until § 3 coverage is declared complete.~~

**⚠ OVERTAKEN BY THE WORK, marked 2026-07-29 (baseline sweep S4/F7).** There are **two
headings named `## 4. Theory` in this file**, ~500 lines apart, in contradictory states: the
one above carries ~360 lines of theory marked *"Status: PROVISIONAL after pass 1"*, and § 5
sits *above* § 4.4, so the file no longer reads in its own declared order. The gate this stub
names — *"§ 3 coverage declared complete"* — is **still not met**
(`corpus-knowledge-evidence.md` § coverage: *"Still not opened: `north-star.md` bodies,
`PIPELINE.md`, all `docs/spikes/*`…"*), so the instruction was never wrong; it was simply never
retracted when the work outran it. **Neither side is being resolved here** — which § 4 governs,
and whether the provisional theory should have been written before the gate, is a call for main
session. *This is T1′ (accretion without retraction) inside the document that measured T1′.*
