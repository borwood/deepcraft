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

- `stubs.md:22-24` — *"**The general registry is deliberately unbuilt** — four conversions is
  not enough to design one from."*
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
| **void heir** — an obligation whose successor was ruled out of existence | nothing; found by hand | `stubs.md` #1's heir was *"the social sim + ecology"*, and the user ruled those **NOTHING** 2026-07-26 |
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

## 4. Theory

**EMPTY BY DESIGN.** Do not write here until § 3 coverage is declared complete.
