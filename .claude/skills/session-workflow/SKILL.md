---
name: session-workflow
description: The deepcraft collaboration workflow — design-partner conversation with ratify-before-recording, delegated background agents under resource rules, gated merges, and the journal/roadmap/corrections audit trail. Use at the start of any substantive deepcraft session, and consult when unsure how to run a milestone, record a decision, or handle a design idea.
---

# The deepcraft session workflow

Distilled 2026-07-19 from the founding sessions (journal/0001–0004). The
user has ratified this as the way sessions on this project run.

## The four hats

You wear all four, switching freely:

1. **Design partner** — engage ideas substantively; sharpen, connect to the
   architecture, find the mechanism. Contribute your own ideas *clearly
   labeled as proposals*.
2. **Delegator** — substantive implementation goes to background agents in
   isolated worktrees while design conversation continues in the gaps.
3. **Integrator** — review agent output skeptically, run gates on merged
   main yourself, fold results into the docs.
4. **Archivist** — journal, roadmap, corrections, memory. The audit trail
   is a deliverable, not overhead.

## Ratification protocol (the load-bearing rule)

- **Never fold your own proposals into docs or decisions until the user
  ratifies them.** Offer → discuss → they agree → record. Ideas of yours
  they haven't reacted to stay in conversation only.
- Ratified-but-unscheduled ideas go to `docs/design/ideas.md` under its
  "sketches, NOT decisions" banner — capture the mechanism, flag nothing as
  final.
- Ratified decisions get **dated DECIDED entries** in the relevant design
  doc, with rationale, in the same conversation they were made.
- Genuinely-user-owned choices (game feel, art, scope forks): present
  options with a recommendation; do not preempt.
- **When a decision lands in a design doc, resolve its ROADMAP Observed
  line in the SAME commit** (added 2026-07-19 after two decided questions
  were re-asked from a stale map). Observed entries that a decision closes
  get collapsed to a parenthetical citing the doc. Corollary: before
  re-asking anything that looks like an open question, grep the design
  docs for a DECIDED entry first — ROADMAP is the sequence, the design
  docs are the record of decisions.

- **Sweep the corpus BEFORE opening a design pass** (added 2026-07-20 after
  the water notebook was opened cold and "discovered" four things the docs
  already held — including a *proven* karst recipe in the orogeny recon and
  S9's classification of the water table as a bounded relaxation). The
  existing rule ("grep for a DECIDED entry before re-asking") was too
  narrow: priors live in ARCHITECTURE bullets, materials/geology asides,
  spike results tables, recon docs, and process entries nobody filed as
  decisions. At the START of any design thread, grep the whole `.md` corpus
  for the thread's nouns, and open the notebook with a **priors section
  first** — your own observations go *after* it, demoted to what survived
  the sweep. The user should never have to say "we've discussed this."

## Delegation

- One milestone/spike = one background agent in a worktree
  (`isolation: "worktree"`). Design conversation continues while it runs.
- **Parallelize by write-set, serialize the machine** (user, 2026-07-20):
  run multiple agents concurrently whenever their WRITE-sets are disjoint
  (reading shared files is fine — only writes conflict). The compile /
  playtest slot stays global-singular: one cargo invocation and one
  dc-client instance (port 7777) across everything; agents are briefed to
  check for live cargo/rustc/dc-client processes and wait. Jobs still
  SEQUENCE when they build on each other's output or share files with high
  conflict potential — name the dependency in ROADMAP when you defer one
  for this reason (e.g. pose_set retirement waits on the console + edges
  merges because all three touch mcp.rs/player.rs/app.rs).
- **Resource rules are mandatory in every agent brief** (machine has hung
  before): one cargo invocation at a time anywhere; every cargo call sets
  `CARGO_TARGET_DIR=<repo>\target` and `CARGO_BUILD_JOBS=4` in the same
  shell invocation; prefer `--release`; never two build-heavy agents at
  once (code-only agents may run alongside one builder).
- A good brief contains: read-these-docs-first list, tight scope with
  explicit do-NOT-touch files, gates, deliverables (results doc or journal
  entry per CLAUDE.md), and a RETURN spec naming exactly what the final
  report must contain.
- **Model economy (re-ratified 2026-07-19, superseding the inherit-default)**:
  **every agent gets `model: "opus"` by default — including Explore/recon
  agents — AND every `agent()` call inside a Workflow script** (learned
  the expensive way same day: a stock deep-research workflow inherits the
  main session's Fable into every fan-out agent; stock/named workflows
  must be re-authored with explicit `model: 'opus'` before launch, never
  run as-is). Fable is the MAIN SESSION's model only; passing no model
  override inherits Fable and burns the usage budget (it cut into the week
  once — that's why this rule exists). **Scale is part of the same rule
  (user, 2026-07-19: "usage is a concern" — a ~100-agent research run
  for mod documentation was "unfathomable overkill for what a handful of
  articles could have told us"): before launching ANY fan-out, state the
  expected agent count and model to the user; size the harness to the
  question — uncontested documentation wants 2-3 readers, not
  per-claim adversarial panels. Full-width research harnesses are for
  genuinely contested claims, by explicit agreement.** The sole exception: an agent doing
  genuinely deep design work (novel algorithms, undecided architecture,
  spike-class uncertainty) may get Fable by deliberate, stated choice.
  Design conversation, integration judgment, and walk interpretation stay
  in the main session regardless.
- Agents sometimes stop while waiting on background builds and their
  completion notification can be lost — if a "waiting" agent goes quiet,
  check for live compiler processes; if none, verify the gates yourself and
  finish the bookkeeping rather than re-nudging forever.

## Integration

- Merge with `--no-ff`, then run the full gate suite on merged main
  yourself (fmt --check, clippy -D warnings, test — all `--release`).
  Grep test output case-sensitively; "0 failed" contains "failed".
- **Shared-cache poisoning: an impossible-looking red gate on main after
  sibling worktree builds is a STALE ARTIFACT until proven otherwise**
  (added 2026-07-20, corrections #21 — mechanism CONFIRMED by eviction).
  Symptom: post-merge `cargo test --workspace` fails with missing-symbol
  compile errors (E0432/E0560) against source that plainly has the
  symbols, while the targeted `-p <crate> --test <name>` run passes from
  the same tree. Mechanism: agent worktrees pinned at OTHER commits build
  the same packages into the shared `CARGO_TARGET_DIR`; under workspace
  feature-unification the lib unit differs from the `-p` unit, and a
  sibling's stale artifact for that unit carries a falsely-fresh
  fingerprint. It does NOT heal on re-run. Remedy: `cargo clean -p <merged
  crates> --release`, then re-run — this resolved it same-day (1.6 GiB
  evicted, suite green). Discipline: (a) `Set-Location` the repo root
  explicitly in every gate invocation; (b) capture `error`/`panicked`
  lines, not only `test result:` lines — a compile failure is invisible to
  a test-result filter; (c) on an impossible red: verify source integrity,
  run targeted, then EVICT — never start "fixing" code the compiler
  says lacks fields it visibly has; (d) two wrong hypotheses preceded the
  fix (transient-heals-itself; port-7777 test collision) — both died on
  the second identical failure; the -p-passes/workspace-fails split was
  the discriminating observation.
- Remove the worktree and branch after merge. Watch for agents whose HEAD
  branch differs from the auto-named worktree branch.
- Fold agent findings into the docs *you* own: decisions → design docs,
  loose ends → ROADMAP Observed, proposed API revisions → reviewed then
  applied. Update auto-memory when project state shifts materially.
- **Architecture and APPEARANCE are different ratification axes** (added
  2026-07-19 after the smooth-TIN far field shipped unseen): an agent's
  work can follow ratified architecture and still change how the world
  LOOKS — visual language, silhouettes, transitions. Any user-visible
  appearance change gets flagged for the user's eye at integration, even
  when the agent flags "NEEDS RATIFICATION: none." The user ratifies
  looks from screenshots, not from architecture descriptions. Corollary:
  **user-visible systems get their design conversation BEFORE dispatch**
  — drill-forward speed is for internals.
- **Defer = write it now** (added 2026-07-19 after the v0-era GPU
  far-mesh conversation was lost unrecorded): the instant a design
  thread is deferred — by either party, in any conversation — it gets an
  Observed line or design-doc open question in the same session. A
  session-end sweep catches stragglers. The corpus cannot protect what
  never entered it.
- **Replacements are briefed as replacements** (same day, same lesson):
  a milestone that replaces an existing system starts with a
  predecessor-property inventory — what the old system did well
  (visual language, capabilities, perf, its own deferred threads), from
  the docs AND from asking the user "what must survive?" — and the brief
  carries explicit no-regression axes. Agents optimize what the brief
  measures.

- **Triage the agent's NEEDS RATIFICATION list; never relay it whole**
  (added 2026-07-20, after two S11 engineering questions — body identity
  across merges, whether a derived index is persisted — were forwarded to
  the user, who rightly asked "how am i supposed to decide on that?").
  Ratification is for **user-owned** choices: game feel, appearance, scope
  forks, world-model commitments. Implementation questions with measurable
  answers are the integrator's to settle or to sequence as work. Sort the
  agent's list into user-owned / rides-as-built / needs-measurement before
  it reaches the user, and say which is which.

- **Keep `docs/design/things-that-will-happen.md` fed** (added 2026-07-20,
  user's instruction). A one-pager of concrete one-line examples of what
  this game *is*, appended whenever a genuinely informative example
  surfaces in conversation — not features, not promises, but the images
  that load the right mental model. **Read it at session start, before the
  design docs.** Append the moment an example lands; do not batch it.

- **Every gen slice gets a gameplay-impact + fidelity trace before it is
  sized** (user, 2026-07-20, after the wind-band flip was initially scored
  "cosmetic-ish": tracing what the player actually sees — a dead-straight
  vegetation line, dune fields migrating in opposite directions across one
  row — upgraded it to a merge-with-the-desert-belt slice). Two explicit
  questions, answered in writing when proposing or filing any change:
  (a) what Earth mechanism is this faithful to, at what tier; (b) what
  will the player see or do differently — walk the readouts (vegetation,
  landforms, strata, resources), don't stop at the field being changed.
  "Cosmetic" is a conclusion that requires the trace, never a substitute
  for it.

- **An agent's MECHANISM is a hypothesis; only its numbers are evidence**
  (added 2026-07-20, corrections #19, after the integrator wrote an agent's
  unverified "the sun moves between launches" into CLAUDE.md as doctrine —
  the sun is fixed, and the falsifying fact had been quoted by the
  integrator earlier the same session). Agents measure well and explain
  confidently; the explanation is where they err. **Test the causal story
  against the corpus before recording it**, and hold a much higher bar for
  CLAUDE.md and this skill than for an Observed line — every future session
  and every agent loads them, so a wrong rule there propagates silently.
  Corollary to the corpus-sweep guard: also check a fresh claim against
  **what you yourself wrote this session** — the fastest falsifier is often
  a fact you handled an hour ago.

## Audit trail (see CLAUDE.md for formats)

- Journal entries are narrative for a future reader and developer-blog
  feedstock: the problem as encountered, wrong turns, mechanism, reasoning.
  Flag standout threads `> blogworthy:`. Assets named `NNNN-description`.
- **corrections.md gets every falsified claim — especially your own.** The
  walk-3 episode (three misdiagnoses corrected by user review) is the
  template: file the correction with its mechanism, prominently.
- ROADMAP updates ride the same commit as the journal entry. Field
  observations land in Observed before anything is Sequenced.

## Verification culture

- Walks catch what metrics miss: after integration milestones, drive the
  running game (CLAUDE.md § Agent walks — `--fullbright` for non-shader
  tests, check `eye_in_solid` before trusting screenshots).
- The user reading your screenshots is part of the loop — surface the
  images, report honestly, and treat their read as data senior to yours.
- Report failures plainly with output; never smooth over a red gate.

## Rhythm

Kick off the milestone agent → design-converse in the gaps (this is where
API.md, materials, worldgen, visuals, and the idea inventory all came from)
→ integrate on landing → journal → next. Sequence lives in ROADMAP; consult
it at session start, leave it true at session end.
