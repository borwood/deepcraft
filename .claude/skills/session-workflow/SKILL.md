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

## Hold the build lock around the CARGO INVOCATION, not the work session (2026-07-22)

Two agents collided on `target\.agent-build.lock`: one held it across a long
stretch of reading and writing code, the other found it >40 min old with no
live `cargo`/`rustc`, judged it stale **per protocol**, and took it. Both
followed the rule. The rule was wrong.

- **Acquire immediately before a cargo invocation; release immediately after**,
  in a `finally`. Never hold it across thinking, editing, or waiting.
- A holder that must keep it across successive invocations **touches** it
  between them, so age reflects activity rather than acquisition.
- The staleness test stays age-based — "no live cargo" is not evidence of
  abandonment, because a legitimate holder is idle between invocations. That
  ambiguity is exactly what this rule removes.

## Design passes get journal entries too (user-directed, 2026-07-22)

The journal charter always asked for "the reasoning behind the decision — not
a changelog," but practice only ever journaled *builds*. The user, at the end
of the octree pass: *"reasoning like this isn't captured in them. is it
captured anywhere?"* It wasn't — DECIDED entries keep conclusions, the
conversation keeps the narrative, and the conversation is not a repo artifact.

So: **a substantive design pass gets a journal entry in the same session** —
the prompt, the sweep's findings, the corrections made mid-flight (an
integrator overreach the user catches is exactly the blog-worthy part), the
holes found and how, the roads not taken. The design doc holds what was
decided; the journal holds how it went. journal/0069 is the template.

## Sequence the slice AND reserve what it's a slice OF (user-directed, 2026-07-24)

When sequencing work, do not sequence only the **first slice**. Sequence a
**reserved continuation slot** naming what the slice is a slice *of* — the larger
arc, the provisional reasoning, and what comes next — **with the depth captured,
not compressed to a task line**.

*"We can't afford to lose what a completed slice was a slice of… that's just
healthy for AI workflows where each session must re-derive everything from
context"* (user). A first slice with no record of its arc gets picked up next
session as an orphan; the direction, the *why*, and the unification with larger
threads evaporate.

**The shape of a sequenced arc:** WHAT · WHY · how it **UNIFIES** with the larger
threads · **FIRST SLICE** · **CONTINUATION SLOT** (explicitly: "this is a slice OF
X; after this slice, the arc continues with…"). Record the "what" and the "why" —
the provisional reasoning developed in conversation — because the conversation is
not a repo artifact and the next session starts cold. Templates: ROADMAP Sequenced
"THE HONEST IDENTITY SURFACE" and "GENESIS-PASSES DRIVE ROCK DISTRIBUTION"
(2026-07-24). This is the corpus-outruns-the-assistant guard applied to sequencing:
the map must carry the arc, not just the next step.

## Accept by OUTCOME, and place a seam WHERE THE FULL THING LIVES (2026-07-24, the S18 walk)

Two failures stacked in one slice (S18 weathering; corrections #46/#47). The machinery was
correct and the gates green, yet what merged was a **pipework demo mislabelled as a
behavior**, and it was one headless probe away from being a *dangling, believed, untracked*
stub. The user's verdict: *"if we lay a seam, let's put it where the full thing is supposed
to live."* Both halves are process rules now.

**Accept by outcome, not by mechanism.** A slice's tests can be green and its RETURN spec
satisfied while the thing it *claims to do in the world* never happens. S18's fold test was
green because it **hand-fed a magnitude that never occurs** (1.3 m; production is 0.04 m —
sub-voxel, expresses as nothing). That is **anti-shape A-3** (green for a reason unrelated to
the claim) hiding an unverified outcome.
- A slice claiming a **world-visible / outcome** change ships a **production-scale outcome
  probe as an acceptance deliverable** — the tour-map/magnitude probe runs **inside the
  slice**, not as post-merge walk-prep. Had "report the band in voxels at production scale"
  been in the brief, the agent's own report would have said "0 eighths, invisible."
- At review, **demand the load-bearing number.** A report that gives a *recipe* for the
  effect ("argmax X") but never its *value* is itself the red flag. Do not record "it
  expresses" into ROADMAP/journal without the number — that is "an agent's mechanism is a
  hypothesis; test before recording," which the integrator violated here.

**Place the seam where the full thing will live — and never model a process as a snapshot.**
S18 ran weathering **once, post-hoc, over the finished record with frozen final-state
fields**, because the brief scoped it decoupled from the deep-time loop to dodge an
entanglement. But weathering is a **continuous process** (the height loop runs it every
epoch). A one-shot of a continuous process is a **category error, not a simplification** — a
snapshot of a movie. And a seam bolted onto a convenient spot is a **dangling extra step, not
a proxy** for the real system.
- For a slice modelling a natural **process**, ask *at design/brief time*: does it run **where
  and when the process runs** — in the loop, over the relevant span — or is it bolted on? A
  seam-first slice of a process is a **smaller-but-real run of the process** (fewer cells,
  coarser grid, still in the loop), never a one-shot demo standing in for it.
- "Keep it simple = run it once" is a false economy when the target is continuous: it does not
  buy a small version of the behavior, it buys a **non-behavior**.
- The antidote to the nightmare (a bolted-on stub, believed, untracked): when a slice is
  knowingly plumbing-not-behavior, say so in its own label, and file the **stub with its heir
  at the site the full thing will occupy** (stubs.md #17). The inventory that would have
  caught it is only as good as the honesty of the slice's self-description.

## Dispatch discipline — the brief template, journal numbers, the gate protocol (2026-07-24 fingerprints)

Distilled from the densest build day (pass-runner, R/H unification, LOD, geotherm — many
parallel agents). Three recurring costs, codified so they stop recurring.

**The dispatch-brief template.** Every implementation brief carries these sections — a
*missing* one is how S18 shipped a mislabeled stub:
- **Read-first** (docs, in order) · **Scope** (tight) + **do-NOT-touch** (explicit files /
  deferred movements) · the **north-star shape** the work rides + the **crossing constraint**
  (declarations = plain data + opaque ids) · **resource/lock rules** (one cargo, the mutex,
  *yield to a live `dc-client`*) · **gates** (crate-clean the crates you AND siblings changed;
  verify by test **name**) · **RETURN spec** (exactly what the report must contain) · **shape
  compliance** (spines ridden, anti-shapes guarded).
- **Two hard checks that MUST be in the RETURN spec** — the S18 lessons made structural:
  1. **Accept by OUTCOME:** *"what production-scale number proves the claim?"* A slice claiming
     a world-visible / magnitude effect ships the headless probe **inside the slice**, and the
     report quotes the number. (S18 was green on a hand-fed magnitude that never occurs.)
  2. **Process-not-snapshot:** for a slice modelling a natural **process**, *"does it run where
     and when the process runs (in the loop, over the span), or as a decoupled one-shot?"* A
     snapshot of a continuous process is a category error, not a simplification.

**Journal numbers for parallel dispatch.** Two concurrent agents pick the same next-free
number blind (the 2026-07-24 `0091` collision). **The integrator assigns the journal number in
the brief at dispatch** (hold the next-free, hand one to each agent), or briefs slug-only
filenames and numbers them at merge. Never let two live agents both "check `journal/` for the
next number." (Wrap § 6 still resolves any that slip through.)

**The integrator gate protocol** (the day's dominant *friction*). Before any merge-gate:
(1) `Get-Process cargo,rustc,dc-client` — no live build, and prefer to gate when no sibling
with an overlapping crate set is live; (2) `cargo clean -p` **the crates you changed AND any a
live sibling changed** (a sibling's stale artifact poisons yours — even a crate you didn't
touch); (3) run `--workspace` fmt/clippy/test; (4) verify by test **name/count**, never
`test result: ok` alone (a false green flatters); (5) confirm `Compiling <crate>` from
**main's** path. **Defer overlapping gates** while a concurrent track shares the target dir —
merge (git-only), run one clean combined gate when the slot frees.

**Don't over-calibrate a placeholder.** A stub's number that will be **recalibrated the moment
its real driver lands** (coal onset with no biology; a rate with no agent) needs only
**plausible-not-degenerate**, not a tuned seat. Seating a two-world calibration for a system
with no real inputs yet is wasted effort (user, 2026-07-24: the geotherm's coal seat "will just
be calibrated again"). Get it non-degenerate, flag it for the walk, move on.

## Recording discipline: quote the MEASURE, never the threshold (2026-07-25)

Two integrator failures on one day, same shape — the *bookkeeping* was less rigorous
than the work it recorded.

**Quote the measured number, never the assertion's floor.** The Movement 3 ROADMAP
entry read *"Production-scale band at argmax: **≥1 voxel**"* — that is the **A-3
guard's threshold** (`production_scale_saprolite_band_reaches_at_least_one_voxel`),
not the result. The integrator had independently verified **6.09 m / 6.77 voxels**
earlier the same day and then wrote the *assertion's floor* into the map, underselling
the slice by most of an order of magnitude. Caught by a walk (journal/0097), not by
review. **"Demand the load-bearing number" applies to what you WRITE DOWN, not only to
what an agent reports.** A threshold answers "did it pass"; a map needs "what is it".

**Quote the absolute beside every ratio.** The flow slice reported its record as
`1.25×` — true in its worktree, stale by the time it merged, because the
`shrink_to_fit` win had moved the denominator (162.57 → 108.55 MiB) after it forked.
The record's own 40.66 MiB never changed; only the baseline did. **A ratio silently
rots when its denominator moves, and an absolute cannot.** Any agent working in a
worktree is measuring against a frozen baseline — so the integrator re-measures
ratios on merged main before they enter a doc.

## Provenance: mark what is ASSISTANT-ORIGINATED (2026-07-25, user-surfaced)

The `identify(pos)` arc carried a Near/Mid/Far **tier** design for days. It was
retired in one exchange once the user asked *"if this is a world query then why tiered
at all?"* — and the decisive fact was its **provenance**: the tiering was
**assistant-originated**, an artifact of one request ("a way to get voxel composition
by looking at it") being split into several instruments, which the user *rolled with*,
and which then hardened into architecture that shaped every downstream decision (LOD
coupling, a "whose ladder?" problem, a tier flag in the payload) without ever being
re-challenged.

The ratification protocol prevents *unratified* assistant proposals from entering the
docs. It does **not** catch the next failure along: a proposal the user accepted
once, in passing, becoming load-bearing doctrine nobody revisits.

**So: when a design element originates with the assistant, RECORD THAT ALONGSIDE IT.**
One clause is enough — *"(assistant-proposed, user-accepted <date>)"*. Two payoffs:
the next reader knows it carries less weight than a user-originated constraint, and it
is a legitimate target for periodic re-challenge. **User-originated constraints are
data; assistant-originated ones are hypotheses that happened to survive.** Where a
retirement happens, keep the superseded reasoning struck through rather than deleted
if it remains load-bearing elsewhere (the LOD-ladder logic survives for a possible
*render-side* query even though it died for the world query).

## Agents die mid-flight — commit WIP, and read the lock before nudging (2026-07-25)

**Commit WIP early and often, because agents DIE, not merely because worktrees are
auto-cleaned.** The existing rule ("commit something within your first few minutes")
was justified by auto-cleanup. The sharper reason arrived when the head-field agent
lost its connection to an **API error** with `head.rs`, a probe, a test file and five
modified files **all uncommitted**. The work survived only because the worktree
persisted. Put *"commit WIP early and often; an agent lost a connection mid-run with
everything uncommitted"* in every implementation brief.

**A dropped agent is not a failed slice — resume it, do not re-brief it.** Check the
worktree for commits *and uncommitted changes* before concluding anything (`git -C
<worktree> status --short`). Then `SendMessage` it: the verified machine state, an
instruction to **commit first, before resuming work**, and a restatement of the RETURN
spec so it need not re-read. Its context is intact and it lands the slice.

**Before nudging a lock-blocked agent, establish whether the lock is LEGITIMATELY
HELD.** The discriminating check is **live `cargo`/`rustc` + lock age**, not the lock's
existence: a sibling holding it for 9 minutes with a live build is *correct* and the
waiting agent is *right to wait* — nudging it would mean taking a lock someone owns.
Only age > 40 min with no live compiler is stale. When the wait is legitimate, **the
integrator waits for the lock and then resumes the agent**, rather than either nudging
or abandoning it. And brief agents to **report what they have with the measurement
outstanding** rather than parking silently: a partial report with the implementation
committed beats silence.

## Assign STUB numbers at dispatch too, not just journal numbers (2026-07-25)

The "integrator assigns the journal number in the brief at dispatch" rule exists because
two concurrent agents pick the same next-free number blind. **The identical collision then
happened in `docs/design/stubs.md`**: the head-field slice and the weathering-profile slice
both filed a **stub #19**, and the integrator only caught it while cleaning up the merge.

**Any append-only numbered inventory has this hazard** — `stubs.md`, `corrections.md`,
`journal/`, spike ids. So: **hold the next-free number for every numbered artifact a brief
may produce, and hand each agent its own** — or brief slug-only and number at merge. The
integrator's merge checklist gains one line: **grep for duplicate ordinals in every numbered
doc the batch touched**, before the commit that folds it.

Renumbering afterward is cheap but not free: references live in ROADMAP, journals, and
in-code comments, so the fix is a corpus-wide grep, not a one-line edit.

## The gate has outgrown one tool call — run it as STAGES (2026-07-25)

The full gate now exceeds a single 10-minute invocation and was **killed mid-`test`**, which
is the worst possible shape: `fmt` and `clippy` had passed and no test had failed, but the
run produced **no verdict at all**. Two causes, both permanent: `cargo clean -p` across
`dc-worldgen` + `dc-api` + `dc-client` forces a **Bevy** rebuild, and the suite itself has
grown past ~870 s in `dc-worldgen` alone — plus the newly-gated probes.

**Run it as separate invocations**, each with its own verdict:
1. `clean -p <changed crates + any a live sibling changed>` **+ `fmt --all --check` + `clippy --workspace --all-targets --release -D warnings`** — the rebuild is paid here.
2. `cargo test --workspace --release` — fast now, because stage 1 already built everything.

A killed stage is then unambiguous: you know exactly which one lacks a verdict, and stage 2
re-runs in a fraction of the time because the artifacts exist. **This is "did it run?" vs
"did it pass?" arriving at the HARNESS level** rather than the shared-cache level — a
timeout is not a red gate, and must never be recorded as one, but it is equally not a green.

**Corollary:** never read a killed run as evidence in either direction. Check how far it got
(the staged log makes this trivial), then finish the missing stage.

## Walks: Claude drives the whole loop (user-directed, 2026-07-25, emphatic)

Recorded in CLAUDE.md § Agent walks as the canonical loop; repeated here because this skill
is what a session reads when planning a milestone. **The user does not launch the game and
does not teleport themselves** — *"the tooling is very clunky / near impossible for a human
right now."*

**Claude launches → teleports → measures → screenshots → briefs → PAUSES for the user →
moves to the next station on their word.** Never hand the user a command to run or
coordinates to type. Offering ("I'm happy to drive") is not the same as doing; when a walk
is owed and the user says go, **launch it**.

**Tour-map before spending live time.** A headless probe that finds each signature's
strongest exemplar and prints coordinates costs one background agent; a walk with nothing to
look at costs the user's session. The 2026-07-25 coal walk was cancelled before launch
because the tour map found **zero coal on the shipped world** — and that null was the most
valuable result of the day (corrections #51). **Brief a null honestly; never launch anyway
to have something to show.**

## The current world output is a SCRATCH SHEET, not a target (user, 2026-07-25)

*"We do not care about the current arbitrary state of the world, do not hold the current
shape to a teleology. It is a scratch sheet while we build a world-building engine."*

So: **a correct fix is not blocked by the fact that it moves every voxel.** Goldens are a
**regression detector** — they answer *"did this change what I expected it to change?"* —
and they are **not a specification of what the world should look like**. When a change is
right, move the goldens and say what moved and why; do not preserve an arbitrary output for
its own sake, and do not water down a fix to keep a hash stable.

The discipline that still binds: **byte-identity is the acceptance test for a conversion**
(a re-housing must not change behaviour), and any *deliberate* output change is an
appearance change — walk-gated, the user's to bless. The rule above frees us from
preserving accidents; it does not license unannounced ones.

## Byte-identity can become a force for DISHONESTY — know which one you are doing (user, 2026-07-25)

The user's standing caution, and it is sharper than the scratch-sheet rule it refines:

> *"The default seed world is a scratch pad, so be aware of where and when a compromise on
> byte identity is totally fine — **the effort to maintain it could be a pressure, under some
> contexts, to make a system less honest in order to conform to the current state of the
> scratch pad.**"*

**Two uses of byte-identity, and only one of them is a virtue:**

- **As a REGRESSION DETECTOR — always right.** *"This slice was supposed to be a re-housing /
  a layout change / a declaration fix. Did it change behaviour?"* Here a moved hash is a bug
  report, and the discipline is load-bearing (it caught a dropped sea-level assignment in
  journal/0090).
- **As a TARGET — a trap.** *"Choose the design that keeps the hash stable."* That silently
  optimises for an **arbitrary output of an unfinished engine**, and it will happily buy you a
  weaker declaration, a convenient-but-wrong constant, or a compatibility shim.

**The test:** ask *"am I holding this still to detect a surprise, or to avoid one?"* Detecting
is the job. Avoiding is how a scratch pad becomes a specification nobody voted for.

**In practice.** When a fix is correct and moves the world: **take the fix, move the goldens,
and say exactly what moved and why.** Never weaken the mechanism to keep a number. Two live
examples: `dc:field/head` declaring its real terrain read (user: *"it should declare what it
reads and **we eat it if it changes the physics**"*), and the hash-domain provider, where
preserving current output would mean preserving correlated draws.

**What still binds:** a *deliberate* output change is an **appearance change** — walk-gated,
the user's to bless — and it is announced, never silent. This rule frees us from preserving
accidents; it does not license unannounced ones.

## An AUDIT's prescription is a hypothesis too (2026-07-25, confirmed the hard way)

We already hold that *an agent's MECHANISM is a hypothesis; only its numbers are evidence.*
This sharpens it to the case that is hardest to doubt: **an audit agent's prescribed FIX gets
the same scepticism as any other agent's explanation** — and audits are the most persuasive
thing we produce, because they cite `file:line` and are usually right.

The spine-audit's prescription for `dc:field/head` was *"declaring `Forced` is **free, and it
pins it**."* Its diagnosis was correct, its citations were correct, and its **fix was half
wrong**: free, yes; **pins it, no.** The only reason it was caught is that the ROADMAP entry
carried one clause — ***"verify that claim before trusting it"*** — into the brief.

**The mechanism it missed, which is the durable lesson about revision-token schemes:** a
`reads` edge on a revision token orders you **after that token's producer** and says *nothing*
about the pass that next overwrites the same **plane**, because that pass writes a *different
token* — a different resource as far as the graph is concerned. Ordinary pipeline passes
survive this by accident: a forward edge into a later stage braces their far side. A
**sidecar** (writes only its own field, its one reader downstream anyway) **has no brace and
floats.**

> **Where one plane carries several revisions per epoch, declaring the revision you consume
> pins ONE SIDE ONLY. Pin the other side with an anti-dependency on the NEXT revision of that
> plane — never the last.** Lagging against the *first* writer covers the whole chain
> transitively; lagging against the *last* leaves you free to slide past everything before it.

**Practice:** when an audit hands you a fix, put *"verify this claim"* in the brief that
implements it, and require the agent to **re-derive the mechanism**, not just apply the
patch. Two slices this session were saved by exactly that clause.

## An empty `git` result from inside a worktree is not evidence of absence (2026-07-25)

Bare pathspecs (`git ls-tree HEAD:journal`, `git grep -- ROADMAP.md`) resolve **relative to
cwd**. Run from inside a worktree the integrator has since **pruned**, they silently look in
the wrong subtree and print **nothing** — which reads exactly like "the file isn't there."
Use root-relative pathspecs (`:/journal/`) when checking whether work landed.

Same episode, same lesson one level up: an agent's gate printed `Compiling dc-worldgen` from
**main's** path instead of its worktree's — the textbook corrections #34 signature. It
**stopped and checked** instead of accepting the green, and found the benign cause: the
integrator had merged the slice while its gate queued behind a sibling's lock, so the
worktree was pruned and paths resolved to main. **Not a false green — the run had validated
merged main, which is the stronger result.** Both halves are worth keeping: *investigate the
#34 signature every time*, and *absence in a tool's output is a claim about the tool as much
as about the world* (anti-shape A-5).

## Two brief-template clauses, earned 2026-07-25

**1. When a brief implements an AUDIT's fix, require the agent to RE-DERIVE the mechanism.**
Put the words *"verify this claim before building on it"* in the brief. An audit's *diagnosis*
is usually right and its *prescription* is a hypothesis like any other — and audits are the
most persuasive artifact we produce, because they cite `file:line`. Two slices were saved by
that clause this session; the spine-audit's `dc:field/head` prescription (*"declaring `Forced`
is free, and it pins it"*) was **half wrong** in the hardest way to catch.

**2. RESERVE every numbered artifact at dispatch — and record the reservation.**
Not just journals: `stubs.md`, `corrections.md`, spike ids — **any append-only numbered
inventory** has the concurrent-collision hazard. Two agents both filed a **stub #19** this
session. And a *reserved-but-unlanded* number looks identical to a *lost* one: journal **0105**
was absent at wrap because its agent was still running. **Say so in the close block** — a gap
nobody explained reads as a mistake. Merge checklist gains one line: **grep for duplicate
ordinals in every numbered doc the batch touched**, and account for every gap.

## The ROADMAP outgrew reading — and grep only finds what you already suspect

**Named by the user at the 2026-07-25 close, and the honest answer is yes:** the ROADMAP is
~5,600 lines and **no session reads it end-to-end.** It is *grepped*. So:

> **Grep surfaces what you already know to look for.** It cannot surface an item whose
> vocabulary has drifted, an item that now **contradicts** something ratified today, or an item
> nobody has thought about in weeks. Those are exactly the items that mislead.

**The evidence is on the record.** `DeepField::chapters` sat unlisted through **three** spine
audits — its own doc comment said *"read by nothing"* since U8, and three consecutive sweeps
added rows for its **immediate neighbours in the same struct** and walked past it. *A
self-declaring comment is not an index, and neither is a document nobody reads whole.*

**Two compensating controls, and the second matters more than the first:**

- **Archive by STATUS, not by age.** `Shipped` grows without bound and is the least often
  needed live — and the **journal already holds its narrative**, so ROADMAP `Shipped` is
  partly duplicative. Moving older `Shipped` entries to a history file (keeping the journal
  number as the stable pointer) shrinks the live board to `Sequenced` + `Observed` + the close
  block — the parts that must stay readable. **Age alone is the wrong axis: a two-week-old
  `Observed` may be the most live thing on the board.**
- **Make the staleness sweep RECURRING, like `spine-audit`.** Shrinking the doc helps someone
  who is already looking; it does nothing for the parts nobody thinks to look at. Only a
  mechanism that *forces* a re-read finds those. Today's sweep had to be **requested** — that
  is the gap. Run it after any batch of merges that ships a new arc, with the explicit job of
  finding items that are **SUBSUMED / STALE / UNBLOCKED / CONTRADICTED** by the new work.
