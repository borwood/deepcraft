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
standing beside an authority rather than deriving from it (A-3), a check that
cannot fail for the reason it claims (A-3)?

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

- **Read-only over the codebase.** No source edits, no refactors, no "while I
  was in there". If you find a bug, report it; do not fix it.
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

## Return

A short report: instances that no longer resolve · § 3 rows added, removed
(with what consumed them), and confirmed · anti-shapes found in changed files,
with `file:line` · expired justifications · anything you propose adding to the
doc but did not add. Plus the commit for the `spines.md` update.
