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

## Proven practice additions (2026-07-21, session 5)

- **The live guided-tour ratification** (journal/0049, user-directed): for
  appearance-class magnitude/knob ratifications, prefer a LIVE co-walk over
  the screenshot-report loop — the user in the running game, the integrator
  driving teleports and station briefings (what the sim did here, the number,
  the owning knob), each move gated on the user's verdict, verdicts recorded
  per station in the same session. Prepare a **tour map** first (a headless
  probe that locates the strongest exemplar of each signature and prints
  coordinates). Null stations are verdicts — brief them honestly rather than
  inventing a reading.
- **The build-mutex lockfile** (proven across 4 concurrent agents):
  `<repo>\target\.agent-build.lock` — create-file loop to acquire, release
  in finally, >40 min = stale, take it. Every agent brief carries it; the
  integrator uses it too. Wait for no live `cargo`/`rustc`/`dc-client`
  BEFORE acquiring.
- **Exit codes lie about GPU crashes** — *fixed for dc-client 2026-07-21*
  (journal/0054: the panics were on Bevy worker threads, so the main loop
  wound down normally and genuinely returned success; `main` now returns a
  derived `ExitCode` — 0 clean / 70 device lost / 71 fatal render error /
  101 panic anywhere). Reading the log tail is still the better habit because
  it names the cause, not just the class. The durable lesson stands:
  corollary of #18/#19, direct observation outranks side-channel inference
  (a process list is not an attribution; a stopped agent cannot hold a fresh
  mutex).

## Proven practice additions (2026-07-21, session 6 — the autonomous session)

- **The impossible GREEN — verify a gate by test NAME, not by `ok`**
  (corrections #27). Agent worktrees share one `CARGO_TARGET_DIR`; a stale
  sibling artifact can be served as fresh, so a full run can report exit 0
  with every suite `ok` while the code just written **never built**. It is the
  mirror of #21's impossible red and worse, because it flatters instead of
  alarming and the recommended `test result: ok` filter is blind to it.
  Integrator practice: `cargo clean -p <every crate changed this session>
  --release` before the merge gate, then confirm the session's new tests are
  **present by name**. Ask "did it run?" separately from "did it pass?"
- **Resume a parked agent; do not re-brief it.** An agent that stops mid-slice
  saying it will wait on a build is the known lost-wake-up failure, not a
  failed slice — its worktree usually holds real, complete progress. Verify
  the machine state yourself (live `cargo`/`rustc`, mutex age), then
  `SendMessage` it that state plus explicit finish-and-report instructions.
  Its context is intact and it lands the work. Both halves matter: check
  before nudging, nudge rather than restart.
- **Verify the load-bearing claim, not the whole report.** Each agent this
  session had exactly one claim that carried its slice, and each was worth
  independent work: for eviction, "`set_block_raw` is the only voxel-writing
  path" (audited — it is); for the fill contract, "the goldens were captured
  pre-rewire" (re-ran the fingerprint against pre-merge main — they were, and
  a circular golden would have been undetectable otherwise). Pick the claim
  whose falsity would be worst and go after that one.
- **A worktree is FROZEN at its branch commit — a design doc you edit after
  dispatch is invisible to the agent, and its stale copy may actively
  contradict the amendment you sent** (caught 2026-07-21 by the user asking
  "you updated the doc while the agent is already working — will it know
  that?"; the answer was no, and its copy still carried the superseded
  sequencing line). So: **commit design docs BEFORE dispatch**, and when a
  decision changes mid-flight, do both halves — `SendMessage` the amendment
  *and* tell the agent to `git merge main`, naming the commits it is missing
  and confirming they are outside its write-set so there is no conflict risk.
  State precedence explicitly ("the message outranks your copy of the doc"),
  because an agent re-reading its spec later will otherwise follow the file.
- **Write the brief's premise as a hypothesis.** The carry-`H` brief asserted
  the deflation basin holds `H ≈ 0`; the agent measured 10.66 m and filed
  corrections #26. A brief inherits claims from the corpus, and the corpus can
  be wrong — say "the corpus says X; verify before relying on it" rather than
  stating X as fact, and an agent will check it instead of building on it.

## Seam-first: the process half of "a summary is not an authority"

Added 2026-07-22 (user-directed, after an audit found four instances of the
same defect in one day). The architecture rule lives in ARCHITECTURE.md §
"A summary is not an authority"; **this is how to work so you stop producing
them.**

**The defect class.** A stand-in written because the real answer did not exist
yet **becomes the definition of the thing**. It is not caught by the stub
inventory, because a stub looks like a fake — it says placeholder, it names an
heir — whereas **a leaked requirement looks like working code that passes
tests**. Four shipped instances: a far-field summarization need that became the
world's surface material rule; six fixed reference rocks that made the sim
believe loose regolith is sandstone; a class-string test that became the form
rule; root cohesion as a rate multiplier instead of a material.

**Write the seam, not the value.** When a system needs an answer another
system will eventually own, do not inline a constant or a heuristic. Declare a
**provider**: a named function with an explicit contract, an identity default
that reproduces today's behaviour exactly, and a doc comment naming its
**heir** — the unbuilt system expected to supply it. The unbuilt system's
obligations then become readable at its call sites, which is the point:
*"the stubbed APIs would be telling us right now what an unbuilt hydro system
is supposed to supply"* (user).

**Practice, in order:**

1. **Ask the test before committing:** *"if this consumer disappeared tomorrow,
   would this code still exist in this shape?"* If no, it is a summary wearing
   an authority's clothes.
2. **Fallbacks must be identities**, not arbitrary constants, wherever the
   consumer composes. An identity makes "provider absent" provably free; an
   arbitrary constant means "absent" and "present but silent" are different
   worlds and nothing tells you which one you are in. The four deep-sim flags
   (`biotic`, `erodibility`, `full_agents`, `tectonic_history`) are the proven
   pattern: empty plane + identity accessor, byte-identity tested.
3. **Byte-identity is the acceptance test for a conversion**, and the goldens
   must be captured from *pre-slice* code. Self-captured goldens are circular
   and this project has a correction on file about exactly that. The integrator
   re-derives them independently.
4. **Granularity follows the hot loop.** A provider must never be called inside
   a hot loop to answer a question that does not change inside that loop —
   materialize it into a plane at a pass boundary instead. Left implicit, the
   tenth conversion lands in the innermost loop.
5. **Socket the constant, not the call site.** Grep the *constant* being
   replaced, not the function you are editing: a constant used twice (as a seed
   and as a cap) that is socketed once ships a brand-new leaked requirement
   inside the slice meant to stop them (journal/0060, `P_ROCK_INIT`).
6. **Do not build the general mechanism first.** Convert the cheapest cold seam,
   let it teach the shape, convert three more, *then* generalize. A registry
   designed before its callers exist is the same mistake in a new coat.
7. **When a stand-in's justification is a measured constraint, the justification
   expires when the constraint does.** Charcoal was excluded because no bed
   survived whole-voxel quantization; partial voxels made that false and the
   code still encoded the old conclusion, in prose, with nothing to fail.
   Prose cannot fail a build (corrections #29).
8. **The identity default can be so cheap it bypasses the machinery it
   validates.** A pass-level provider whose identity is "leave the plane empty"
   means the byte-identity test never exercises the plane path at all — it
   proves the fallback, not the seam. Every pass-level conversion has this
   hole. So a conversion must also ship an **agreement test**: register a
   *non-identity* provider that materializes the identity answer into a real
   plane, run the world down the provider path, and land on the same
   fingerprint. Both arms proven, on the real grid (journal/0061).
9. **Measure a ritual delta across alternating sets, never A-then-B.** In
   journal/0061 a single alternation read +0.9 % — plausible, under the bar,
   and wrong: the second pre-slice set moved the baseline 1.4 % with no code
   involved. Machine drift between sets is routinely larger than the effect
   being measured. Alternate, take the mean of several, and confirm each side
   is the binary you think it is (by test count) *before* timing it.

## Shape compliance — the closed loop (user-directed, 2026-07-22)

`docs/spines.md` names the recurring shapes (S-1…S-8), the anti-shapes
(A-1…A-6), and the index of machinery that exists and nothing calls. It is
read-first in CLAUDE.md. This section is how it stays true instead of becoming
another document nobody consults.

**At PLAN time — every brief names its shapes.** A dispatch brief states which
spines the work rides and which anti-shapes it is guarding against, and points
at § 3 if the work should be *consuming* something already built rather than
writing a second one. A brief that names no shapes is not ready to send.

**At WORK time — deviation is loud, never silent.** A worker who believes a
deviation is strictly correct **says so, in the report, as a plea**: the shape
being deviated from, the argument, and what breaks if the shape is followed.
It is not theirs to approve. Silence is the only disallowed answer — shipping
a deviation unremarked is the failure, not the deviation itself.

**At REVIEW time — compliance is part of integration.** Before merging, the
integrator checks: does the work ride the shapes its brief claimed · did it
introduce a new instance worth listing · did it hit an anti-shape · does it
empty a row of § 3 (a *good* event — record what consumed it and when) · does
any comment it touched cite a constraint that has since expired (A-2).

**Carve-outs pass through the user.** A plea goes to main-session discussion,
and ships only with user ratification, recorded in `spines.md` § 4 with its
date and reasoning. Ratified deviation is just a decision; undocumented
deviation is the failure mode.

**Update in the same commit** — the discipline ROADMAP and the journal already
carry. A spine gaining an instance, an anti-shape caught in the wild, a § 3 row
emptied: all land with the change, not after.

**Periodically, delegate `spine-audit`** (its own skill) — a deep file-by-file
sweep checking the doc against the codebase, prioritising files changed that
day. That is the other half of the loop: this section keeps the doc *applied*,
the audit keeps it *true*.

## Measurement agents must write to their worktree EARLY (2026-07-22, incident)

A worktree given with `isolation: "worktree"` is **auto-cleaned if unchanged**.
A *measurement* agent — one that runs a binary, watches memory, times a ritual —
naturally writes nothing until it has an answer, so it looks unchanged and its
worktree can be **deleted out from under it mid-run**. That happened to the
horizon-6 agent; it restored the worktree itself and kept going, and the only
reason anyone learned of it was the user asking the agent directly.

**Brief every measurement agent to commit something within its first minutes** —
a stub results file, the harness, even the plan. Two payoffs: the worktree stops
looking disposable, and a park costs numbers instead of everything.

**And the integrator lesson, which is the larger one:** an empty worktree is
**not** evidence that an agent has produced nothing. I read three attempts of
"empty worktree" as three failures and attached a stopping rule to it; the
actual cause was external deletion, and the agent was working the whole time.
This is anti-shape **A-5 in `docs/spines.md`** — an observation with an assumed
cause, reported as a diagnosis. Before concluding an agent is failing: check
whether the worktree still exists in `git worktree list`, whether its branch has
commits, and whether its processes are burning CPU. Ask the agent. Do not infer
failure from absence.

## Clean the crates a SIBLING built, not the crates you changed (2026-07-22)

Sharpening of corrections #21/#27, learned from a false red on merged main.

A merge gate failed on `palette_len_matches_atlas` — a dc-client test asserting
a shader constant against `MATERIAL_COUNT`. Main had 25 materials and asserts
29 layers: **self-consistent**, and the merge had touched neither dc-core nor
dc-client. A concurrent sibling adding a charcoal material had **26**, and its
dc-core artifact was served to the integrator's dc-client build out of the
shared `CARGO_TARGET_DIR`.

The standing practice — `cargo clean -p <each crate you changed> --release` —
**cannot catch this**, because the poisoned crate is one you did *not* change.
So:

- **Clean the dependency closure your tests actually read**, not your diff. If
  a sibling is touching dc-core, the integrator cleans dc-core even when the
  merge was pure dc-worldgen.
- **The build mutex does not prevent this.** It serializes *invocations*;
  poisoning comes from artifacts persisting *between* them. Concurrent tracks
  plus one target dir are mutually corrupting wherever their crate sets
  overlap — the real cost of parallel tracks, and it lands on the integrator.
- **Prefer to gate when no sibling with an overlapping crate set is live**, and
  say in the report which siblings were building during the run.
- **On any red, check source consistency FIRST** (does main's own source
  satisfy the assertion?) before reading it as a defect. Two numbers from two
  worktrees settled this one in a single grep — no rebuild required.
