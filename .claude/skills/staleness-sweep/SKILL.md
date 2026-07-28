---
name: staleness-sweep
description: Check ROADMAP entries against NEWER WORK - is a Sequenced item already shipped, is an Observed defect already fixed, did a decision land that closes an open question, did a recalibration turn an observation into an artifact. The third sweep beside spine-audit (docs vs CODE) and doc-topology (docs vs EACH OTHER); this one is docs vs WHAT HAPPENED SINCE. Run FIRST THING in a session, incrementally from the recorded watermark. Delegate to a background agent; read-only except its own audit doc.
---

# staleness-sweep — the board against what has since happened

**Existed as a proven procedure and two worked examples since 2026-07-24; packaged as a
skill 2026-07-28** so it has the same standing as its two siblings. *It ran twice without
being a skill, which is twice more than `spine-audit` — the procedure was never the
problem.*

## The triad — know which sweep you are

| sweep | compares | to |
|---|---|---|
| `spine-audit` | `spines.md` | the **codebase** |
| `doc-topology` | the docs | **each other** |
| **this one** | ROADMAP entries | **what has shipped since** |

If the question is *"is this entry still true given what we did?"* it is yours. If it is
*"do two docs disagree?"* it is `doc-topology`'s. If it is *"does the doc still match the
code?"* it is `spine-audit`'s.

## ⚠ WHEN TO RUN — FIRST THING, NOT AT WRAP (user, 2026-07-28)

**Run at session START.** Findings then arrive when there is a whole session to act on
them. A wrap-time sweep reports at the moment nothing can be done with what it found.

**Stagger by cost and relevance:** this sweep is the cheapest and most
session-relevant — run it every session. `spine-audit` only earns a run when **code**
moved. `doc-topology` earns one after a batch of merges that ships an arc, or when a
design thread reopens something old.

## THE WATERMARK — incremental by default (user, 2026-07-28)

Each sweep records the commit it covered, in `docs/audits/.sweep-watermarks.json`:

```json
{ "staleness": "f652b60", "spine-audit": "...", "doc-topology": "..." }
```

**Writing your watermark is part of producing your audit — not a second step to
remember.** That is the whole reason it will survive: this project measured that a
convention dies when it asks an author to restate something in a second notation
(`JUSTIFIED-BY`: documented in two places with a promised sweep, **3 uses, 0 in
`crates/`**), and survives when it is inseparable from an act already being performed.

**Your delta is `git log <watermark>..HEAD`** — the journals, corrections, design-doc
DECIDED entries and merges landed since. That is your *subject*: you are asking which
board entries those invalidate.

### When incremental is NOT valid — go full

A sweep's incremental mode holds only while its **reference side** is unchanged.

- **This sweep is inherently incremental** — it is *defined* as entries-vs-newer-work, so
  the delta is the whole point. Both 2026-07-24 and 2026-07-25 runs scoped themselves to
  one session's decisions.
- **Go FULL anyway when:** the board itself was restructured (an archive pass, a section
  moved), a **recalibration** landed that changes what old observations MEAN (see
  `journal/0111` below), or the watermark is missing/older than ~5 active days.
- **`spine-audit` goes full whenever `spines.md` itself changed** — every prior "this
  instance is fine" verdict was made against a different rule.

## ⚠ THE RECALIBRATION TRAP — the highest-value thing this sweep can catch

**`journal/0111`: the world was running ~1000× too slow on denudation and was
recalibrated.** Every Observed entry recorded *before* that about a magnitude, a rate, or
"system X doesn't seem to matter here" may be **an artifact of the wrong scale rather than
a real defect**. `journal/0110`'s facies null is the known case — *honest about its
mechanism and wrong about its cause: rivers did nothing, but partly because nothing did
anything.*

So a whole cohort of observations can be invalidated by one commit, and **no amount of
per-entry reading finds that** — you have to know the recalibration happened and then
re-read the cohort through it. **Check the delta for recalibrations FIRST**, before
walking entries.

## What to look for, in value order

1. **Already shipped, entry never closed.** Cross-check against `ROADMAP-history.md`
   (Shipped), the journal, and `journal/corrections.md`.
2. **Already DECIDED, Observed line never resolved.** `session-workflow` requires the
   Observed line be resolved in the *same commit* as the decision. Find where it wasn't —
   this is the most common miss, and it re-asks questions the user already answered.
3. **A recalibration turned an observation into an artifact** (above).
4. **An entry's stated BLOCKER was removed.** These are the unblocked-and-nobody-noticed
   items; they are the cheapest wins on the board.
5. **`NEEDS RATIFICATION` markers where the ruling already happened.**
6. **An arc living only in a close block or only in a journal.** The 2026-07-25 run
   rescued exactly this — an arc that *"until today existed only inside a close block."*

## Hard rules

1. **`file:line` for the entry AND for the evidence.** A staleness claim with one side is
   an opinion.
2. **You may NOT edit the ROADMAP.** Both prior runs say it: *"No ROADMAP or design-doc
   edits made — **the integrator applies**."* You produce a dated audit; the integrator
   decides. The sweeper never adjudicates what it finds.
3. **A USER FIELD REPORT IS DATA.** Never mark one resolved on reasoning alone — only on
   evidence that the world changed (a journal entry, a shipped fix, a measurement). If you
   cannot find that evidence, **it stays open and you say so.** Four user field reports
   from 2026-07-20 were still open on 2026-07-28; a wrong "resolved" on one of those is
   the worst outcome available to you.
4. **"Deliberately unfixed" is a valid state** — § Observed's own title says so.
5. **`CANNOT-DETERMINE` is a first-class answer.** A flagged, honestly-bounded unknown
   beats a confident reconciliation; the integrator owns closing it.
6. **Nothing rests on "feels stale"** — the 2026-07-25 run's own words. Every verdict
   carries a line, a test name, or a journal citation.

## Dispatch

**Delegate to a background agent, `model: opus`, `isolation: "worktree"`, read-only except
its own audit doc.** Pass the worktree flag explicitly — an agent without it shares the
integrator's checkout, and one swept an integrator's work-in-progress into its own commit
on 2026-07-28. Brief it to use **explicit pathspecs** on commit, never `git add -A`, and to
**commit a skeleton within its first few minutes** (agents die mid-run with everything
uncommitted).

**Output:** `docs/audits/YYYY-MM-DD-roadmap-staleness-sweep.md` — plus the watermark
update, in the same commit.

**Templates:** `docs/audits/2026-07-24-roadmap-staleness-sweep.md` and
`docs/audits/2026-07-25-roadmap-staleness-sweep.md`. Both open by naming exactly what they
swept against and stating that no edits were made. Copy that opening.

## Honest limit

This sweep finds entries invalidated by **newer work**. Measured 2026-07-28 across all 67
corrections: **staleness is 3–8 % of recorded failures; ~91 % were wrong the day they were
written.** So this is a real control with a **single-digit ceiling by construction** — it
keeps the board honest, and it is not the mechanism that prevents bad claims. Do not let a
green sweep read as "the corpus is sound."
