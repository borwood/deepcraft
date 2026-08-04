---
name: spine-audit
description: Deep file-by-file sweep checking docs/spines.md against the actual codebase - are the spine instances still real, is the built-but-unconsumed index still accurate, did today's changes introduce an unlisted instance or an anti-shape, and do cited justifications still hold. Run periodically (a few times a day on active days), and always after a batch of merges. Delegate to a background agent; it is read-only except for the doc it maintains.
---

# spine-audit

`docs/spines.md` is only worth having if it is **true**. This skill is the
half of the loop that keeps it true; the `session-workflow` skill's
§ "Shape compliance" is the half that keeps it *applied*.

**Run it:** after a batch of merges, and a few times a day on an active day.
**Delegate it** to a background agent, opus, read-only over the codebase.
It writes exactly one file: `docs/spines.md`.

## Why this exists

This project's characteristic failure is not forgetting decisions — DECIDED
entries work. It is **building a mechanism and losing it**, then writing a
second one beside it (anti-shape A-4). One session found the corpus ahead of
the assistant fourteen times, including four pieces of shipped, tested
machinery that nothing called. An index fixes that **only if it is audited**;
a stale index is worse than none, because it is trusted.

## The five checks

**1. Do the spine instances still resolve?** Every file reference in § 1 —
does the file still exist, does the symbol still exist, does it still do what
the entry claims? An instance that has drifted is a finding: either fix the
entry or file a correction if the *claim* was wrong.

**2. Is § 3 still accurate?** For each row, grep for real callers.
- Still uncalled → leave it, note the date checked.
- **Now consumed → remove the row and record what consumed it and when.**
  This is a good event and the whole point of the index.
- Newly uncalled machinery found anywhere in the tree → **add a row.** Look for
  the signature: a `pub` item with tests, a doc comment naming a future
  consumer, and no production call site.

**3. Did changed files introduce an unlisted instance or an anti-shape?**
Prioritise files changed since the last audit (`git log --since`, `git diff`).
For each: does it add a new instance of a spine that should be listed? Does it
hit an anti-shape — a stand-in hardening into a definition (A-1), a summary
standing beside an authority rather than deriving from it (**~~A-3~~ this is a
SPINE, `S-3`, violated — not an anti-shape**), a check that
cannot fail for the reason it claims (A-3)?
*(Corrected 2026-07-29, baseline sweep S7/F5: the first label was wrong twice — wrong number
**and** wrong class. `spines.md` `S-3` is "a summary is derived from the authority, never
beside it"; `A-3` is "a test green for a reason unrelated to what it asserts". Because this
skill files its findings **into `spines.md`**, an auditor following this check was filing
S-3 violations under the A-3 heading, where the genuine A-3 instances live.)*

**4. Do cited justifications still hold?** Grep `JUSTIFIED-BY:` and, more
broadly, comments justifying a design by citing a constraint ("so that a pack
cannot…", "because none survive…", "since nothing yet emits…"). For each, ask
whether the constraint is still true. **Decisions expire premises elsewhere,
often the same day** — this check is anti-shape A-2 and it has caught two
shipped instances.

**5. Is the doc drifting into decoration?** Entries with no file reference, a
spine with one instance and no rule, an anti-shape with no *check*. Prune or
sharpen. The file earns its read-first slot or it loses it.

## Rules for the auditing agent

- **APPLY mechanical findings; report the rest** (user, 2026-07-29: *"we need
  the sweep findings applied"* — findings filed in a doc nobody reads are not a
  product, and the auditor has the correct context to apply them without
  blocking the main session). The tiers:
  - **Apply in your branch:** corrections of claims your sweep falsified —
    doc-comment text, doc prose, stale `file:line` citations, a caption that
    states the opposite of what the code beside it does. These change no
    behavior and the evidence is already in your hands.
  - **Report, never apply:** anything that changes code behavior (a refactor, a
    call-site conversion, a golden); anything **contested** (a finding that
    challenges a user-ratified claim gets a `⚠ CONTESTS` flag, per
    corrections #65); anything needing ratification; and any file the
    dispatching session names as **in flight** with another agent — route those
    findings to main session, which forwards them to the owning agent.
  - No refactors, no "while I was in there" — the apply tier is *corrections of
    false statements*, never improvements.
  - **Frame world-byte findings by the scratch-pad doctrine** (user, 2026-08-02,
    correcting a finding that called a salt re-roll "a ruling"): the shipped world
    is a fixture — a fix that moves world output under *ratified semantics* is an
    ENGINEERING item whose cost is the golden-capture window (free while a slice
    already owns the re-capture, a separate announced capture otherwise), never a
    byte ratification. Route it as sequencing ("fold into the slice that owns the
    capture"), not as a user decision. What IS user-owned stays so: unratified
    semantics, and appearance-as-a-design-axis (walk-gated).
- **Do not run cargo.** This is a reading task, and the build slot belongs to
  implementers. `git log`/`git show`/`git grep` are fine.
- **Cite `file:line` for every claim**, and distinguish what the code does from
  what a doc says it does. Where they disagree, say so loudly — that
  disagreement is the most valuable output.
- **⚠ AND CITE THE COMMIT YOU READ IT AT** (added 2026-07-28, **corrections #67**, which cost a
  filed correction to learn). **This sweep's own 2026-07-22 run is the cautionary case.** It
  quoted a doc comment accurately; `11d4385` had rewritten that comment **the day before**;
  corrections #40 then read the *current* text, concluded this sweep had paraphrased, and
  recorded *"the A-2 was never live."* **It was live, and #40 was wrong** — and the wrong verdict
  propagated into `stubs.md` § 4 and two rows of `spines.md`. **A quotation without a revision
  cannot distinguish a paraphrase from a stale read**, and this corpus holds **808** `file:line`
  citations, all implicitly *"as of some unstated commit."* `git rev-parse --short HEAD` is one
  call and you are already running git.
  *Corollary: **an absence needs its pathspec.*** Correction #63 asserted a symbol *"does not
  exist anywhere in the tree"*; it existed in the very probe its author was using. When you
  report that something is uncalled or absent, **state the search you ran.**
- **Do not add a spine on your own authority.** A genuinely new recurring shape
  is a *proposal* to the main session; name it and let the user ratify. The
  file's own § 4 rule binds the auditor too.
- **Report what you removed and why**, not only what you added. A shrinking
  § 3 is the success condition.

## Run the stand-in marker check — one command, first thing

```
python scripts/standin_locus_check.py
```

**Every `⚠ STAND-IN` in a doc comment must carry a resolvable pointer to a locus.** The check walks
the tree for the sigil and diffs it against the loci; ~166 ms. **Report its output in your return,
including a clean run** — a control nobody reports is a control nobody runs.

*Why this and not a wider grep:* measured 2026-08-04 over a full census of 213 `heir` sites
(`docs/audits/2026-08-04-stand-in-marker-control-scoping.md`). The **wide** check yields **27 false
positives, 6 filed-elsewhere and 1 true defect** out of 34 — a gate people route around. The
**narrow** one is **100 % precise across the sigil's entire history**, one lifetime false positive,
and replayed against 268 commits it fires **exactly the four `GaitKnobs` markers** at the commit
that produced `corrections.md` #97. **A hit is a real finding; investigate every one.**

**Loci are FIVE, not three** (user-ratified emergent practice, 2026-08-04 — read-first item 6 names
three and the tree cites five): `docs/design/stubs.md` · `docs/spines.md` § 3 · ROADMAP **Owed /
Observed** · **`journal/NNNN`** · **a `docs/audits/` design pass**. Measured share: journal **41.4 %**,
audits **25.5 %**, stubs+spines+ROADMAP **38.6 %** combined, with ROADMAP alone at **4.5 %**.

## Return

A short report: instances that no longer resolve · § 3 rows added, removed
(with what consumed them), and confirmed · anti-shapes found in changed files,
with `file:line` · expired justifications · anything you propose adding to the
doc but did not add. Plus the commit for the `spines.md` update.
