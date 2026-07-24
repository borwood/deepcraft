---
name: wrap
description: Session-end ritual for deepcraft - close every loop before stopping. Rescue artifacts that live outside the repo, sequence anything deliberately skipped, resolve journal/corrections numbering, record conditional ratifications and deviations, account for in-flight agents and unverified constants, clean worktrees and machine state, rewrite the ROADMAP close block, update memory. Run when the user says to wrap up, close the session, or stop for the day - and before any long pause.
---

# wrap — closing the loops

Every check below exists because it was **missed once**. Work down the list;
each item names what it costs to skip. Do the work rather than reporting the
list — the deliverable is a clean repo and a short honest close report.

## 1. Rescue anything that lives outside the repo

Sweep the session's scratchpad and temp paths. Audits, inventories, probe
output, measurement logs — anything **cited during the session** must land in
the repo, in `docs/audits/` (sweeps) or `docs/spikes/` (spikes) or as an
example.

*Earned 2026-07-22: three audits totalling 1 390 lines — the 34-seam inventory,
the deeptime vector audit, the hydrology priors sweep — were cited all day and
sat in a temp directory. Anti-shape **A-4**, committed on the day A-4 was
written. An index only helps if what it indexes is in the repo.*

## 2. Excluded is not the same as sequenced

For everything deliberately **not** done: does it exist in ROADMAP as owed
work, **with its reason**? A "not in this batch" aside inside a close block is
not a record — it reads as a decision already taken and then evaporates.

*Earned same day: `block_twin` was excluded from a seam batch for a good reason
and appeared nowhere as owed work. The user caught it by asking.*

## 3. Deviations, carve-outs, and conditional ratifications

- Did any merged work deviate from a spine? Ratified into `spines.md` § 4, or
  filed as a rework? **A-7 deviations (a process naming a content identity) are
  never ratifiable** — they are defects and get a rework entry.
- Did the user accept anything **conditionally**? Record the condition verbatim
  next to the work, and track the outstanding half explicitly.

*Earned same day: a charcoal carve-out was accepted at integration, overruled
in discussion, and had to be filed after the fact; coal was accepted only on
condition of a geotherm seam.*

## 4. In-flight agents

For each still running: what it holds, what it is blocked on, what the next
session must **check rather than trust**. If one has parked, check the machine
before concluding it failed — worktree present, branch commits, live processes
— and remember an **empty worktree is not evidence of nothing**.

## 5. Unverified constants and trusted claims

Anything carried through a merge on trust — a golden, a fingerprint, a
threshold computed against a different base — is either **gated now** or named
loudly as unverified. Never let "probably fine" cross a session boundary
silently.

## 6. Numbering collisions

Concurrent agents pick journal numbers and corrections numbers independently.
Resolve **all** collisions, and fix inbound references — carefully: a blanket
rename will corrupt legitimate references to a different entry with the same
number.

## 7. Corrections for every falsified claim

Including — especially — the assistant's own, and any of the project's own
shipped numbers that turned out to be phase, luck, or a stale premise.

## 8. Spines and the deferred-thread sweep

- `spines.md`: new instances added, anti-shapes caught in the wild recorded,
  § 3 rows emptied where something finally consumed them.
- **Defer = write it now.** Anything either party deferred in conversation gets
  an Observed line or a design-doc open question **before stopping**. The
  corpus cannot protect what never entered it.

## 9. Docs closure

ROADMAP's close block rewritten for the next session and the **previous block
marked superseded**; decisions recorded in the design doc that owns them, not
only in conversation; `things-that-will-happen.md` fed if a genuinely
informative example surfaced.

## 10. Repo and machine hygiene

Merged worktrees removed and branches deleted; unmerged ones named with what
they hold. No orphaned `dc-client`, no held `.agent-build.lock`, no process
sitting on port 7777. Working tree clean.

## 11. The gate actually passed

Main green, verified by **test name and count**, with `cargo clean` covering
**the crates a sibling built** — not merely the crates you changed — and
ideally with no sibling building during the run.

## 12. Memory

Update auto-memory only if project state shifted **materially** — a new
read-first doc, a changed protocol, a reversed decision. Not a session diary.

## 13. Fingerprints — offer, and fold in what's greenlit

Before the close report, **offer fingerprints** (user-instituted 2026-07-24): a brief,
honest reflection on the session — the agent experience running main + orchestrating,
how the process and codebase served (or fought) the work — and **concrete proposed
improvements at any level** (a brief template, a process rule, a doc-status pass, a
skill/doc fix). Present them as **proposals the user may greenlight**, often as the
**last message of the session**. **Fold in the greenlit ones** this same session, into
the owning skill/doc; leave the rest as recorded proposals. The point is the corpus
improving itself — each session's friction becomes next session's guardrail. Do not skip
this when the session was substantive: a session that taught something and left no
fingerprint wasted the lesson.

*Instituted 2026-07-24, the densest build day — when the dispatch-brief template, the
journal-number-at-dispatch rule, and the integrator gate protocol were folded into
`session-workflow` from the day's frictions, and this step added so it recurs.*

---

## The close report

Short, and honest over tidy: what shipped · what was **ratified**, in the
user's own terms · what was **falsified**, the assistant's own errors first ·
what is still running and what to check about it · and the two or three things
the next session opens on.

**If a check finds nothing, say so.** A wrap that reports only successes has
usually not looked.
