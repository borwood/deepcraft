---
name: doc-topology
description: Sweep the .md corpus for claims that CONTRADICT other claims in the corpus - a decision superseded in one doc and still asserted in another, a number restated three ways, a design reconciled away in a doc its author does not read, a refutation sitting in the same entry as the claim. Fills the gap between spine-audit (docs vs CODE) and the staleness sweep (docs vs NEWER WORK): nothing else checks the docs against EACH OTHER. Run after any batch of merges that ships an arc, and whenever a design thread reopens something old. ALSO the door to the measured DIAGNOSIS of our docs-ops problem - read the top of this file before proposing any fix when the complaint is "the docs are a mess", "our knowledge base is drifting", "what is wrong with our process", "I thought we already fixed this", "the information loop keeps failing to close", or anyone proposes tags, frontmatter, doc version numbers, stale-link detection or a knowledge graph. Four docs-ops interventions have already shipped and the field has been measured; do not re-derive it.
---

# doc-topology — check the corpus against itself

> ## 🛑 SENT HERE BECAUSE "THE DOCS ARE A MESS"? READ THIS FIRST.
>
> **The field has been measured. Do not re-derive it, and do not propose a fix before reading
> [`docs/design/corpus-knowledge-notebook.md`](../../../docs/design/corpus-knowledge-notebook.md)**
> (the argument, ~500 lines) — evidence in
> [`corpus-knowledge-evidence.md`](../../../docs/design/corpus-knowledge-evidence.md).
> Measured 2026-07-28 over all 67 `corrections.md` entries, 776 commits and 193 `.md` files, and
> **independently re-coded by a second agent that was forbidden to read the first analysis**
> (`docs/audits/2026-07-28-corrections-recoding.md`). Directions are robust; every percentage is a
> band, because **43 % of entries were coin-flips for both coders.**
>
> **The four results that will change what you propose:**
> 1. **Staleness is 3–8 % of our recorded failures. ~91 % were WRONG THE DAY THEY WERE WRITTEN.**
>    For most of them *there is no change event to watch*, so a stale-ref detector has a single-digit
>    ceiling **by construction**. The binding failure is at the moment of **assertion**, not decay.
> 2. **"Surface the related docs" targets the smallest band measured — 1–5 %.** In the expensive
>    cases the refutation was already on the author's screen: forty lines away, two sentences away,
>    two subsections away, or in their own output from the same session. **Access was never the
>    problem.**
> 3. **A tagging convention has already been tried here and got ~0 % adoption.** `JUSTIFIED-BY`
>    had a documented convention, a stated validator, and a named sweep in *this* skill's sibling —
>    **3 occurrences, 0 in `crates/`**. Meanwhile `heir`, the bare word, has **651**. The law:
>    **a convention survives only if it is inseparable from something the author must do anyway, or
>    is the natural way to say the thing. It dies if it asks them to restate in a second notation.**
> 4. **The corpus already writes the graph; nothing reads it.** 651 `heir`, 525 `RATIFIED`,
>    418 `DECIDED`, ~5,000 stable-id citations against 154 whole-doc ones. **Authoring is solved.
>    Extraction and discharge are not.**
>
> **What is already shipped, so you do not propose it again:** the `ROADMAP-history.md` archive
> (volume) · this sweep (topology) · the *"a user design may not be superseded by an implementation
> slice"* rule (authority) · the filesize hook. **Their honest limit is on the record: none of them
> would have prevented journal/0119.**
>
> **The one open proposal** is § 5 of the notebook — a *reciprocity* check, marked
> assistant-originated and unratified. ~~**And `stubs.md:22` forbids designing the general
> mechanism before several real conversions have taught the shape.** That binds this thread
> too.~~ **STRUCK 2026-07-28 (user).** `stubs.md:22` is about **plugin-authorable engine
> sockets** — declaring provider slots as *data* so a mod can add one without recompiling. It
> shares **no mechanism** with a documents problem, and citing it here stretched an analogy past
> its domain. *(The user, reading that clause: "I honestly don't understand what the registry is
> supposed to be except for a list which we can extend" — correct about what exists, and the
> reason it should never have been a veto on this thread.)* **What still binds is
> `session-workflow` § Seam-first #6** — convert the cheapest cold seam and let it teach the
> shape — which is a rule about how to build anything, not a claim about this domain.

> **⚠ COUNTS IN THIS FILE WENT STALE ON 2026-07-28 AND ARE CORRECTED HERE, NOT ABOVE.**
> There are now **three** sweeps, not two — `staleness-sweep` was packaged as a skill that day,
> so "the two existing sweeps" below and "four docs-ops interventions" in the description are
> both one short (there are **six**: the archive, this sweep, the authority rule, the filesize
> hook, the staleness skill, and the SessionStart due-hook). And of the **three** one-directional
> pointers the block above calls *"confirmed live"*, **two were repaired the same day**:
> `corrections #56` now strikes and names `#60`, and `S10-results.md` now carries its banner.
> *Left standing rather than rewritten, because the block's ARGUMENT is unaffected and because
> this is the fourth enumeration in the corpus found stale in one afternoon — the pattern is
> worth more than a tidy number. A sweeper inheriting that list must re-verify each item before
> reporting it, which is `corrections #67`'s rule applied to this file.*

**The two existing sweeps both compare the corpus to something outside it.** `spine-audit`
checks `spines.md` against the **code**. The staleness sweep checks entries against **newer
work**. **Nothing checks the docs against each other** — so a claim and its own refutation
can coexist indefinitely, in two files, or in one paragraph, and no process will ever put
them side by side.

*Earned 2026-07-26 (journal/0119, corrections #65). `material-behavior.md` § 5 asserted that
deep-time phase order "falls out of the declared reads/writes"; `journal/0090` said, two
sentences into the entry that celebrated it, that a relaxation pipeline "does **not** fall
out of a dataflow graph for free — you have to name each revision as a distinct resource for
the topo-sort to reproduce a fixed sequence." Both true statements, 400 lines apart, one
literally the other's counterexample. They sat there for three days and cost an
architecture. **Every piece was in the corpus and findable by grep; what failed is that no
reader ever had all of them in view at once.***

## Why grep cannot do this

Grep returns what you already suspected. A contradiction between two documents is exactly
the thing nobody suspects — if anyone had, it would already be resolved. **This sweep must
READ, not search**, and its unit of work is a *pair* of statements, not a file.

## What to look for — six shapes, in value order

1. **Superseded-but-still-asserted.** A decision changed in doc A; doc B still states the old
   one as current. *The highest-frequency shape and the most expensive, because doc B reads as
   authoritative.* Check every `DECIDED` / `RATIFIED` / `SUPERSEDED` entry against everything
   that cites the same noun.
2. **A claim refuted in its own neighbourhood.** The corrections #65 shape. Read whole
   entries, especially the celebratory summary of an implementation slice — that is where a
   caveat gets stated honestly and then not propagated to the doc it invalidates.
3. **A user-originated design reconciled away.** *(**Grep IS correct for this one shape** —
   the exception to the rule above, because the target is a known marker rather than an
   unsuspected pair.)* Grep `ideas.md` and every "user sketch" /
   "user's reasoning, recorded" block, then check whether each half **survived**, was
   **contested**, or was quietly **replaced** in a design doc. A replacement with no recorded
   argument is a finding. *(CLAUDE.md: a user-originated design may not be superseded by an
   implementation slice.)*
4. **One number, several values.** The same measurement quoted differently in two places —
   usually because one is a *threshold* and the other a *result*, or one came from a
   **fixture** and the other is claimed as **production**. Both happened this project. Quote
   the measure, never the bound; name the world every number came from.
5. **A justification whose constraint expired** (anti-shape A-2) where the *expiry* is
   recorded in a different file than the *justification*.
6. **A SUMMARY THAT OUTRAN ITS SOURCE** — *added 2026-07-26 after the first run, which found
   four instances of it and ranked two of them in its top three.* Doc Y paraphrases doc X;
   X is later amended **in place**; Y is not. **Structurally different from shape 1**,
   because *the summary was never wrong when it was written and no decision was ever
   reversed* — which is exactly why nobody catches it, and why grep cannot: both halves use
   the same words in the same order. (`north-star.md`'s own opening line survived its
   § core/plugin boundary retiring it — twice, in the same file.)
   - **⚠ ITS PRESCRIBED CHECK CANNOT FIRE — measured 2026-07-28.** This shape used to end
     *"the procedure is finite and mechanical: **diff the paraphrase against its source**."*
     **That check is structurally dead in this corpus.** Measured across every design doc:
     they delete **2–4 %** of what they add and amend **in place, at the claim site** (hunk
     mean position 0.41–0.81). So an author supersedes a claim by adding a strikethrough or a
     dated note *beside* it — and **the superseded sentence stays in the file.** The diff
     therefore *succeeds*: doc Y's paraphrase still matches text that is still literally
     present in doc X. **Do not run it and read a null as clean.**
   - **What to do instead — check RECIPROCITY, not paraphrase-equality.** The failure that is
     actually detectable is the **one-directional edge**: an artifact that declares it
     corrects / supersedes / voids another *by name*, where the named artifact carries no
     back-pointer. Three confirmed live instances, all in read-first surfaces:
     `north-star.md` § Deviations #2 voids the capability-tiering in three *named* sibling
     sections and § Passes still reads *"trusted … first-party"*, unstruck · `corrections #12`
     declares itself *"the pointer"* for `S10-results.md`'s ~2×-wrong cost table and `S10`
     never names it · and **`corrections.md` #56 asserted, unstruck, exactly what #60
     withdraws, with no `#60` token anywhere in the file.** Measured: **8 of 15** full-path
     correction→file edges are one-directional. **A one-directional pointer is not a pointer;
     it is a note to whoever already found the answer.**

## Rules

- **Read whole sections. Do not grep-and-conclude.** If you find yourself confirming
  something you already believed, you are running the wrong instrument.
- **Prioritise by BLAST RADIUS, not by age.** A contradiction in `north-star.md`,
  `ARCHITECTURE.md`, `CLAUDE.md` or this skill set propagates into every future session and
  every agent brief; one in a journal entry is history and mostly harmless.
- **Report pairs with file:line on BOTH sides.** A finding that quotes one side is an
  opinion.
- **Do not resolve a contradiction you find.** Which side wins is frequently a **user call**
  — that is precisely why it survived. Report it, ranked, with a recommendation labelled as
  one.
- **Provenance decides weight.** *"User-originated constraints are data; assistant-originated
  ones are hypotheses that happened to survive."* When two claims conflict and one is
  user-originated, that is not a tie.
- **Re-read both sides AT SOURCE before publishing a pair.** Run one did, and it changed a
  verdict. A quote carried forward from an earlier step of your own sweep is a summary, and
  this sweep exists because summaries drift.
- **⚠ RECORD THE COMMIT BESIDE EVERY QUOTATION — a quotation without a revision is not a
  quotation** (added 2026-07-28, **corrections #67**, which cost a filed correction to learn).
  The 2026-07-22 seam audit quoted a doc comment accurately; `11d4385` had rewritten that
  comment **the day before**; corrections #40 then read the *current* text, concluded the audit
  had paraphrased, and recorded *"the A-2 was never live."* **It was live, and #40 was wrong.**
  Neither party misread anything — they were looking at the same file **at two different
  times**, and nothing in either artifact carried a revision for its quote.
  **So: when you quote source, cite `file:line` AND the commit you read it at** (`git rev-parse
  --short HEAD` is one call, and this sweep is already running git). This corpus holds **808**
  `file:line` citations, every one of them implicitly *"as of some unstated commit"*. **A stale
  READ is indistinguishable from a paraphrase, and only one of them is a finding.**
  *Corollary earned the same day: **an absence needs its pathspec.*** Correction #63 asserted
  a symbol *"does not exist anywhere in the tree"*; it existed in the very probe its author was
  using. State the search, not just the verdict.
- **A null is a result**, but a null from this sweep is *suspicious* — say how much you
  actually read, and which docs you did **not** open.

## Scope, and how to keep it finite

The `.md` corpus is large and this sweep cannot read all of it every time. Pick a spine and
follow it:

- **After a batch of merges — BOUND BY ARTIFACT COUNT, NOT BY NOUN** (*corrected
  2026-07-26 after run one: "every noun the batch touched" is not finite and cannot be
  planned*). The countable spine is: **the read-first set (6 files) + the batch's own
  journal entries + every doc they cite by name.** That is knowable in advance, so the sweep
  can be sized before it starts.
- **ALWAYS include `ROADMAP.md`'s current close block, checked against itself.** It is the
  highest-traffic artifact in the corpus, it is audited by nothing, and run one found **five
  findings inside that one artifact** — including four shipped things still reported as owed.
- **When a design thread reopens something old:** that thread's nouns, exhaustively, across
  every doc — this is the corpus-sweep the session workflow already requires at the start of
  a design pass, run as an audit rather than a lookup.
- **Periodically, unprompted:** the read-first set only — `CLAUDE.md`, `north-star.md`,
  `spines.md`, `ARCHITECTURE.md`, `ROADMAP.md`'s live board, and this skill set. Highest
  blast radius, smallest page count.

## Deliverable

`docs/audits/<date>-doc-topology-sweep.md`: a ranked table of contradiction **pairs**, each
with both `file:line` citations, the two conflicting statements quoted, which side is
user-originated (if either), the blast radius, and a labelled recommendation. Then file the
falsified half in `journal/corrections.md` **only where a claim is actually falsified** —
a contradiction is not automatically a correction; sometimes both statements are true of
different things and what is missing is the sentence saying so.

Delegate to a background agent (read-only except its own audit doc), and hold its
corrections number at dispatch like any other numbered artifact.
