# deepcraft

Voxel game, bespoke Rust/Bevy stack. Internal codename; public name TBD.

## Read first

0. **[`docs/design/north-star.md`](docs/design/north-star.md)** — the **engine
   shape everything converges to** (ratified 2026-07-23): a native engine whose
   core is cell storage + a pass-runner + field-solver **primitives** + a stable
   API, with materials, their behavior, and the passes over them **authored in a
   uniform, self-declaring, compiler-validated shape and tuned by data** —
   plugin-first and moddable by third parties, **one authoring shape**. *Every
   pass is content, including tectonics and erosion*; **pass ORDER is authored
   per world** (ARCHITECTURE.md, DECIDED 2026-07-26). **The trusted/untrusted
   split and the ABI/WASM backend tiering are DEFERRED** — north-star §
   Deviations 1, *"paused until we are anywhere near having modders"*; there is
   **no difference in permission between native and WASM**, so never justify a
   core/content placement by trust. It is the
   *destination*, pursued **evolutionarily** via seam-first conversions. **All
   design flows through it; divergence is a loud plea, never silent** (§
   Compliance). It is the strategic companion to spines — read both first.
0b. **[`docs/spines.md`](docs/spines.md)** — the recurring **shapes** (S-1…S-9),
   the **anti-shapes** (A-1…A-7), and the index of **machinery that exists and
   nothing calls**. spines names the shapes of the code *as it is today*; the
   north star names where it is *going*. This project's characteristic failure is
   re-inventing a mechanism *next to* the one it already built. **Work is
   justified against these shapes**; a deviation is loud, discussed in main
   session, ratified by the user, and recorded in its § 4. Update it in the
   same commit as work that adds an instance or empties a row of § 3.
1. **[`ROADMAP.md`](ROADMAP.md)** — the living sequence (In flight / Sequenced /
   Observed + the current close block). Read before proposing work; update it in
   the same commit as any journal entry. **Completed work and superseded close
   blocks live in [`ROADMAP-history.md`](ROADMAP-history.md)** — archived
   2026-07-26 **by status, not age**; each Shipped entry keeps its **journal
   number** as the stable pointer. *Archive an item when its status stops
   requiring it to be read live, never when it gets old.*
1b. **The corpus is checked against ITSELF, not only against the code.**
   `spine-audit` compares docs to code; the staleness sweep compares entries to
   newer work; **[`doc-topology`](.claude/skills/doc-topology/SKILL.md)** compares
   docs to **each other** — a decision superseded in one doc and still asserted in
   another, a claim refuted in its own neighbourhood, a user design reconciled away
   in a doc its author does not read. Added 2026-07-26 after a claim and its own
   refutation sat 400 lines apart for three days and cost an architecture
   (corrections #65, journal/0119). Run it after any batch of merges that ships an
   arc.
1c. **[`docs/dependency-graph.md`](docs/dependency-graph.md)** — **what blocks what**, and the
   **engine/SDK ↔ default-pack partition** stated once so it stops being answered from memory.
   ROADMAP holds *what order we chose*; this holds *why that order is forced* and what is
   genuinely startable today. **Read at session start, update in the same commit as any slice
   that moves a state or reveals an edge, checked by `wrap` at the end** (user, 2026-07-29).
   *Added after a cold session classified the highest-value item on the board as "engine work"
   because the previous night's close block said so, while read-first item 0 refuted it **by
   name** — corrections #71. **Every pass is content, including tectonics and erosion.***
2. **[`docs/design/things-that-will-happen.md`](docs/design/things-that-will-happen.md)**
   — one-line concrete examples of what this game IS. Read before the design
   docs; it loads the mental model fastest. Append to it whenever a genuinely
   informative new example surfaces.
3. **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)** — decisions with dates;
   then `docs/API.md`, `docs/design/*.md`, `docs/rendering/PIPELINE.md`.
4. **[`journal/corrections.md`](journal/corrections.md)** — claims already
   falsified, with mechanisms. Check before re-deriving.
5. **Spike results live in `docs/spikes/S*-results.md` — measured numbers, don't
   re-guess them. IMMUTABLE BODY, MUTABLE HEADER (DECIDED 2026-07-28, user).**
   A spike's measurements are **a dated record of what was measured on the day** and
   are **never rewritten** — not to correct them, not to update them. But a results
   doc **must carry, at its top, a pointer to anything that later refuted, superseded
   or re-scoped it.** *The testimony is frozen; the banner is not.*
   - **Why the rule exists — this collision was live for eight days and cost a
     ratification.** `corrections.md` #12 declared *"a spike result is a dated record…
     **this entry is the pointer**"*; this item said *go read the spike, trust its
     numbers*. Both reasonable, in different files, never reconciled. Meanwhile
     `S10-results.md`'s cost table reads **25.19 s** where production measures
     **13.79 s** (the spike drove the *scalar* path, production takes the *parallel*
     one) — **a user ratified a ship decision on the wrong number, and `S10` held no
     reference to its own correction.**
   - **The mechanism it fixes is structural, not clerical: a ONE-DIRECTIONAL POINTER
     IS NOT A POINTER.** It is a note to whoever already found the answer. Measured
     corpus-wide: **8 of 15** correction→file edges are one-directional and **14 of
     30** audit/spike files carry no staleness marker at all. A chain of authority
     cannot be walked from the stale end if the stale end holds no link — and the
     stale end is exactly where a cold session enters.
   - **The obligation lands on the WRITER OF THE CORRECTION**, in the same commit,
     while both documents are already open. *This is the one property that makes it
     likely to stick: stamping the target is part of the act of writing the
     correction, not a second notation to remember later. Conventions that ask an
     author to restate something die — `JUSTIFIED-BY` was documented in two places
     with a promised sweep and got 3 uses, 0 in `crates/`.*
   - **Applies to any dated-measurement artifact**: `docs/spikes/`, `docs/audits/`,
     probe reports. **Backlog:** the corpus was never swept for missing banners; only
     known refutations are stamped so far.
   - **AND TO CLOSE BLOCKS (2026-07-29, corrections #73, greenlit fingerprint): a close
     block recording a RESOLUTION ("settled — X") is a correction-shaped act and owes the
     asking document its banner in the same commit.** The 460-vs-28.8 answer lived only in
     a close block for five days; the audit that asked it was never stamped; a design
     agent then re-derived the question and a main session propagated the re-derivation —
     caught only by the user's memory. A close block is a handoff that gets archived, and
     it cuts both ways: nobody re-reading the question's doc can find an answer recorded
     only there.
6. **"What did we leave dangling here?"** — the loose-ends lookup, three loci,
   all surfaced by the corpus-grep that opens every design thread (session-workflow
   § "Sweep the corpus BEFORE opening a design pass"):
   **[`docs/design/stubs.md`](docs/design/stubs.md)** (stand-ins, each with its heir
   + blast radius), **[`docs/spines.md`](docs/spines.md) § 3 "Built, and nothing
   calls it"** (machinery built but not yet consumed), and **ROADMAP Owed /
   Observed**. A thread deliberately left open is annotated **in code** (a loud
   marker naming its heir) **and** listed in one of these — an unlisted loose end is
   the defect, not a licence (stubs.md doctrine). *Added 2026-07-24 after the
   integrator, holding the whole corpus, still had to hunt for two of these — if the
   lookup isn't a named pointer, "process points you there" is not real.*

## The journal

`journal/` is the project's memory **and the feedstock for a future developer
blog**. Append-only, numbered entries (`NNNN-slug.md`), screenshots in
`journal/assets/NNNN-description.png` (before/after pairs where relevant).

Write entries as narrative for a future reader: the problem as encountered,
the wrong turns taken, the mechanism discovered, the reasoning behind the
decision — not a changelog. Especially capture **unique problems and why we
solved them the way we did** (the combinatorial mixture cap, the year-zero
ledger handoff, the z-fight mechanism — that caliber). Flag standout threads
with a `> blogworthy:` line naming the angle — and name which **lens** it
speaks to (user, 2026-07-22; a thread can serve several):

1. **AI-native development** — emergent patterns and best practices of
   building with agents: briefs as hypotheses, ratification hygiene, the
   corpus outrunning the assistant.
2. **Procgen dev against the backdrop of priors** — the weighing of options,
   costs and benefits of competing architectures; the roads not taken and
   why.
3. **Reflexions in a deepsim codebase** — architecture philosophy: sorting
   areas of concern, who owns what, the right primitives and spines so
   nothing is bespoke and nothing is a carve-out.
4. **Respect for earth and anthropological processes** — full gamut
   geo·paleo·archae·eco·anthro·socia: we are students of it and want to be
   honest and faithful to it. Field reports from walks go to
ROADMAP **Observed** first; entries diagnosing them move work to Sequenced.
Falsified claims get a corrections.md entry in the same commit.

## Build rules (this machine)

- cargo is NOT on PATH: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" ...`
- **One cargo invocation at a time across ALL agents/sessions** — parallel
  heavy builds have hung this machine. Cap with `$env:CARGO_BUILD_JOBS='4'`.
- Agent worktrees share the main build cache:
  `$env:CARGO_TARGET_DIR='B:\repos\borwood\deepcraft\target'` (set env vars in
  the SAME shell invocation; they don't persist between tool calls).
- Prefer `--release` (cache is warm); never build two profiles concurrently.

## Gates (all must pass before merge)

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --release -- -D warnings
cargo test --workspace --release
```

- **GATES MAY BATCH ACROSS RELATED SLICES (user, 2026-08-02: "let's keep velocity up. we
  can gate for every couple slices if they're related to each other").** The full trio is
  owed per **arc-chunk**, not per merge: consecutive slices of one arc may merge on cheap
  evidence (fmt + clippy on the changed crates + the changed crates' own tests), with the
  full workspace gate run before the arc pauses, before anything ships beyond the arc, and
  at wrap. **The batch debt is recorded in each intermediate commit message and cleared
  loudly** — an unrecorded deferred gate is a false green wearing a schedule.
  share one `CARGO_TARGET_DIR`, and a sibling's stale artifact can be served
  as fresh — producing a **false green**: exit 0, every suite `ok`, and the
  code you just wrote never built (corrections #27; the silent mirror of #21's
  impossible red). So before a merge gate, `cargo clean -p <each crate you
  changed> --release`, and **verify by test name or count**, never by
  `test result: ok` alone. "Did it run?" is a separate question from "did it
  pass?"
- Capture `error` / `panicked` / `FAILED` lines, not only `test result:` lines
  — a compile failure is invisible to a test-result filter.
- **`cargo test` BUILDS examples but never RUNS them** (found 2026-07-25). Our
  measurement instruments — the residency probes, the tour maps — live in
  `examples/` and several carry `assert!`s. **The gate cannot see them fail.**
  `flow_cost_probe` was broken by the flow merge (its itemised baseline lost track
  of `resident_bytes`, off by the whole 42.6 MB flux record) and sat green through
  a full 664-test workspace gate, because the gate never executed it. The rule:
  **an example that can fail belongs in the gate.** A probe nobody runs is a probe
  that is silently wrong, and the numbers it produced are still sitting in the docs.
  - **"Re-run the probes by hand after a merge" was tried here and FAILED IN ONE
    DAY.** That advisory was added to this section on 2026-07-25; the next day
    `flow_cost_probe` broke *again, identically* — a missing `head` row after
    FLOW (a) merged. **The assertion caught it; the process did not.** Do not
    answer "the gate cannot see X" with a rule asking people to remember X.
    Retired in favour of the mechanism below (journal/0103).
  - **The mechanism is `test = true` on the example's Cargo target** (journal/0103).
    ```toml
    [[example]]
    name = "my_probe"
    test = true
    ```
    Cargo then *additionally* builds that example with the libtest harness, so its
    `#[test]`s run under `cargo test` — while `cargo run --example my_probe` still
    executes `main` and prints the full report. **One file, one set of measurement
    functions, two consumers.** Nothing moves to a library and nothing is
    duplicated: factor the measurement into a function returning a struct, let
    `main` print it and a `#[cfg(test)] mod gate` assert on it.
  - **Size the test, not the report.** Several probes build the production world
    (seed 1337, `Extent::Medium`) and take 20–90 s; a gate that grows five minutes
    gets worked around. Run the *test* at the **smallest extent that still
    exercises the invariant** — and say in the test's doc comment **why the
    invariant is scale-free** (a per-voxel predicate, a per-column arithmetic, a
    topological property). Keep `Medium` only where the claim is genuinely about
    production scale. Probe conversions report their added gate wall-clock.
    *Both `flow_cost_probe` failures were a **missing row in an itemisation** —
    wrong at every world size. The defects probes catch are usually structural,
    and structural is scale-free; the magnitudes are what the report is for.*
    Measured: all seven converted probes add **35.0 s** to the gate (journal/0103).
  - **Assert invariants, never snapshots.** An itemisation equal to its own total;
    a bound; a ratio. **Not** a MiB figure — ledger residency moved twice in one
    afternoon, and a test pinned to yesterday's number fails *because a colleague
    improved memory*, which is worse than the defect it was guarding.
  - **A printed caption is a published claim the gate cannot check.**
    `flux_record_probe` printed *"honestly EMPTY … heirs: the head field"* beside a
    non-zero count for a day after that heir landed. When a slice fills a hole a
    probe narrates, the caption is part of the diff.
- **READ THE LOG THE GATE WROTE, NEVER THE CONSOLE CAPTURE** (2026-07-26). A gate piped
  through anything that truncates — `| Select-Object -Last N`, `| tail` — writes the full
  run to its `Tee-Object` file while the *console* keeps only the tail. Reading the wrong
  one produced a **textbook false-green signature**: `test exit=0`, `STAGE2 EXIT=0`, **0
  tests passed, 2 binaries**, against 742/76 an hour earlier. It reads exactly like
  corrections #27 and it was pure instrumentation error. **Grep the Tee'd path**, and if a
  count looks impossible, suspect your own pipeline before the artifacts. *The discipline
  that caught it is the one that matters: "did it run?" is a separate question from "did it
  pass?" — and it applies to the harness, not only the compiler.*
- **And grep the build log for the crate you changed** (corrections #34) — with
  the right verb: `build`/`test` print **`Compiling dc-x`**, `clippy` prints
  **`Checking dc-x`**. **Do NOT anchor the pattern to line start** — cargo
  indents status lines by three spaces, so `^Compiling` matches nothing and the
  check reads zero forever (caught 2026-07-22 only because the raw count was
  reported rather than assumed). Grep the **path** too: the line names the
  checkout it built from, which is how you tell a worktree's artifact from
  main's. With several worktrees on one `CARGO_TARGET_DIR`, build
  state is shared and package *names* are ambiguous: a `cargo clean -p` plus a
  concurrent sibling build has been observed resolving this worktree's
  `dc-worldgen` against a **sibling's `dc-core`**. Before a gate that matters,
  wait until `Get-Process cargo,rustc` is empty.
- **THE BUILD-SLOT MUTEX IS A HOOK NOW — do not manage `.agent-build.lock` by
  hand** (2026-08-02; `scripts/cargo_mutex_hook.py`, wired PreToolUse on
  Bash|PowerShell). The advisory lock failed both directions in one night —
  unseen during a live gate, then deleted unread beside an ownership-blind
  `Stop-Process` (ROADMAP § Observed, 2026-08-01) — so the rule became a
  mechanism, per this section's own doctrine. The hook **denies any cargo
  command while cargo/rustc processes are alive (yours included) or another
  session's claim is <3 min old**, and stamps the lock itself on allow. Don't
  write the lock, don't delete it, don't re-read it. If a build is truly
  wedged, stop the **specific diagnosed PID** — never an unscoped
  `Get-Process cargo | Stop-Process`.

## Agent walks

### THE WALK LOOP — CLAUDE DRIVES, ALWAYS (user-directed, 2026-07-25, emphatic)

**The user does NOT launch the game and does NOT teleport themselves.** *"The tooling is
very clunky / near impossible for a human right now."* Never ask them to run a command,
never hand them coordinates to type, never wait for them to get somewhere. **When it is
walk time, Claude does all of it.** The loop, every time:

1. **Claude launches the game** — `cargo run --release -p dc-client -- <flags>` from the
   repo root, with the flags the question needs (`--fullbright` for material questions,
   `--edges` for geometry legibility, plus any feature flag such as
   `--weather-inventory`). Respect the one-cargo-at-a-time rule; take the build slot.
2. **Claude teleports the player to station 1** (`client_player_pose_set`, `surface: true`
   for walker-safe placement), and **checks `eye_in_solid` before trusting anything**.
3. **Claude takes the objective measurements** if the station needs them —
   `world_get_contents` (never `scan_region` for material questions), a bench cut via
   `world_fill`, whatever the question requires.
4. **Claude takes the screenshot** (`client_screenshot`, bare lowercase slug named for the
   journal entry it belongs to).
5. **Claude briefs the station and PAUSES** — what the sim did here, the number, the owning
   knob, and what to look for. **Then it waits for the user to poke around and give a
   verdict.** The user's live view is senior to the screenshot read.
6. **On their word, move to the next station** and repeat. Record each verdict per station,
   in the same session.

**BUILD THE STATION SO IT CAN ANSWER — three framing rules, each earned at the 0148 walk in one
sitting (greenlit fingerprint 2026-08-03).**
1. **SEPARATE THE SUBJECTS.** Comparing N bodies means N *lanes*, never one line. The 0148 walk
   put three bodies on one path; they occluded each other and the user said *"that makes it hard
   to study them."* A station that cannot separate its subjects yields a verdict about the
   framing, not about the sim.
2. **CAPTURE WITHIN SECONDS OF THE INTENT — Claude time is not real time.** Setting three intents
   then screenshotting is ~4 MCP round-trips ≈ several seconds ≈ **metres of travel**; the 0148
   walkers left frame before the shutter. Either shoot immediately, or aim the camera where the
   subject **will be**, or drive the subject **toward** the camera so framing improves as it
   moves.
3. **PICK THE VIEW THE SEPARATION AXIS ALLOWS.** Subjects separated along X and viewed along X
   overlap. Lanes want a head-on or from-behind camera; profile wants a single subject. *Vertical*
   questions (a bob) read against a horizontal ground line; *cadence* questions read head-on.

**And the walk loop has NO STOP CHANNEL** (ROADMAP § Observed, 2026-08-03): every intent commits
seconds of world motion before the driver can react, and the observer has none at all. The user
**built a wall** to stop bodies walking off a ledge. Until a leash / bounded intent / freeze verb
exists, **prefer intents that terminate safely** — drive toward the observer, or into open ground,
never toward an edge.

**Tour-map first, always.** Before spending any of the user's game time, run a headless
probe that finds the strongest exemplar of each signature and prints coordinates. A walk
that turns out to have nothing to look at is a walk that should never have been launched —
the 2026-07-25 coal walk was cancelled by a tour map that found **zero coal on the shipped
world**, which cost one background probe instead of a live session (corrections #51).
**A null from the tour map is a result; brief it honestly rather than launching anyway.**

- Connect: run the game (`cargo run --release -p dc-client` from repo root),
  MCP at `http://127.0.0.1:7777/mcp` (streamable HTTP).
- **Testing anything non-shader-related? Launch with `--fullbright`** —
  unlit materials, pure vertex color — so lighting/tonemap output never
  masquerades as a geometry or data defect (journal/0004).
- Check `eye_in_solid` in every pose response before trusting a screenshot;
  use `pose_set { surface: true }` for walker-safe teleports. **`yaw`/`pitch` are
  RADIANS** (not degrees — `-10.0` silently clamps to −1.55 rad ≈ straight down);
  negative pitch looks down. **Units differ across the tools:** `pose_*` speaks
  **metres**, while `world_fill`/`scan_region`/`get_contents` speak **voxels**
  (`voxel_y ≈ metres / 0.9` at N=2) — mixing them probes tens of metres off target.
- **For any MATERIAL question use `world_get_contents`, never `scan_region` /
  `get_block`** (journal/0097). The latter answer with the stored 1-byte `Block`
  summary, which collapses mudstone/sandstone/siltstone/granite alike into
  `dc:stone` — a walk once read a whole column as `dc:stone` and nearly reported
  "no band" from an instrument that structurally cannot see one. `get_contents`
  returns the real mixture. **`has_contents` is now a PER-VOXEL fact and is
  trustworthy** (fixed 2026-07-25, journal/0101; it used to be answered per-CHUNK —
  corrections #49). Read it as: `has_contents: false` on a **solid** block means
  *"no composition record here"* — the unrecorded basement, legacy soil, ocean
  floor, border wilds — and **NOT air**; `classified` then just echoes `block`.
  An air voxel honestly answers `has_contents: true` with an empty composition.
  `block` remains the field that reads the same `block_at` as `eye_in_solid`.
- **Cutting a cross-section?** `world_fill` a **bench** (a wide shelf, ~20k voxels
  of `dc:air`) rather than a narrow pit — a pit frames badly and a road-cut face
  reads at a glance. Screenshot names must be a bare lowercase slug.
- **dc-client's exit code is now honest** (journal/0054): `0` clean, `70`
  GPU device lost, `71` fatal render error, `101` a panic on any thread. The
  old "exit codes lie about GPU crashes" warning is retired *for dc-client*.
  Reading the log tail is still the better habit — it names the cause, not
  just the class — but it is no longer compensating for a broken signal.
- Screenshots land in `journal/assets/` — name them `NNNN-description` for
  the journal entry they belong to.
- **The moment a spot is called a REFERENCE, record its exact pose** — feet in
  world metres, `yaw`, `pitch` — in the Observed/station entry, beside the asset
  (corrections #48). *A prose landmark is not a pose.* The palette-quant station
  was recorded as "the east coast, ~110 km east of spawn"; the real spot is
  **~71 km**, and the 39 km error sent a reconstruction into grey single-class
  coast that does not carry the signature at all — producing four null frames and
  a confident wrong conclusion. This is "defer = write it now" applied to camera
  poses.
- **Pick the control that can SEE your question** (journal/0030,
  corrections #18/#19). The sun is FIXED (S4: a constant 0.35 time-of-day),
  so lit before/after comparisons across launches ARE valid.
  - **Shape / relief / geometry questions → the LIT pass.** Face
    orientation is what carries shape; `--fullbright` is unlit pure vertex
    colour, so every face of a block is one colour and a terraced hillside
    on single-material ground renders as a **featureless grey field**.
    Fullbright is blind to shape.
  - **Material / data questions → `--fullbright`.** Flat albedo with no
    lighting noise is what made the 0027 coal diagnosis possible.
  - **Geometry legibility inside fullbright → add `--edges`** (journal/0031):
    crease/silhouette outlines, distance-faded to zero past 1.4 km — so the
    far field carries NO edge signal by design; judge far silhouettes by
    the skyline, not by missing outlines. `--fullbright` alone remains the
    byte-identical pure-data control; never use an `--edges` frame for a
    colour/material diff. Fullbright also no longer applies distance fog
    (0031), so long-vista silhouettes are readable.
  - Using the blind control and reporting its null is how journal/0030
    published a wrong conclusion twice in one day.

## Conventions

- **THE TESTING WORLD IS A SCRATCH PAD** (user; promoted here 2026-07-29 because it kept
  getting lost — it lived in `session-workflow` § scratch-pad doctrine, and a fresh ruling
  got drafted from scratch in ARCHITECTURE.md by a session that could not see it. *"There
  is absolutely no inherent reason to think its current state at any point is intended."*).
  **Goldens are regression tripwires, never ratified intent.** An *unexplained* hash move
  is a defect to chase; a hash move produced by *ratified semantics* re-captures the
  goldens **with the why recorded** — it owes no byte-identicality and no ratification
  loop on the new bytes. Never argue a design or a hold from the fixture's current bytes
  (the fixture-state sibling of [[placeholder-state-is-not-intent]]).

- **A CLOSED SYSTEM CANNOT DETECT ITS OWN SCALE ERROR** (2026-07-26, journal/0111 —
  and it cost us ~1000×). The deep sim was **perfectly self-consistent at the wrong
  scale**: mass closed, every golden held, passes were pure, per-species budgets
  balanced to 1e-12 — and the world was denuding **9× slower than the slowest
  landscape ever measured on Earth**, stripping 5.48 m where a real craton strips
  5–10 km. **No internal instrument could ever have seen it**, because every
  internal instrument checks the sim against *itself*.
  - **The rule: whenever a simulated quantity has a real, published counterpart,
    measure it against the literature AT LEAST ONCE.** Denudation rates, erosion
    rates, geothermal gradients, sediment yields, grain-settling velocities — these
    are measured in the real world and the numbers are in the literature. **Those
    are the only errors a perfect internal audit is structurally blind to.**
  - **It also decides what a calibration may be fitted to.** A constant tuned until
    an output "looks right" is *a number pretending to be a mechanism*; a constant
    derived so a measured quantity lands in a **published band** is evidence. Same
    rule as the tolerance doctrine in § Gates — *a bound with a derivation is
    evidence; one chosen until green is not.*
  - **Corollary for reading a null.** Movement 2b's facies null (journal/0110) was
    honest about its mechanism and **wrong about its cause**: rivers do nothing —
    true — but partly because *nothing* does anything, and that was never a fact
    about rivers. Before concluding "system X is unimportant here", check that the
    world is running at the right scale at all.

- **EXISTENCE IS NOT STANDING** (user, 2026-07-26). *"This is a case of seeing that
  something exists and assuming that it should… a curse brought on from early bootstrap
  where content was added without ratification."* An agent tracing a call chain proves the
  code **runs**; it proves nothing about whether it **should**. Both halves cost real work
  the day this was written: a rider traced ruin-placement all the way into `generate_chunk`
  and the integrator converted "it renders" into "it is a constraint", then proposed
  *counting* the ruins — which already concedes that some number would matter. It would not.
  Unratified bootstrap content has **no standing at any magnitude**.
  - **The test:** *if this did not exist, would we build it today, in this shape?* If no, it
    is baggage, and the honest disposal is removal — not preservation, not measurement, not
    a golden protecting it.
  - Distinct from [[placeholder-state-is-not-intent]], which says *do not argue from current
    constants*. This says *do not argue from current **content***.
  - **We have no evo / socia / civ modelling even at the design stage** (user's words:
    *"we do NOT have any form of evo/socia/civ modeling even at the design stage: they are
    NOTHING"*, and *"we have not moved on to bio/evo/socia/civ… just open edge gestures so
    far"*). Anything in the tree that looks like one is early-bootstrap fabrication awaiting
    wholesale replacement. The project is working on **earth processes**, and fighting the
    unratified shapes left behind by bring-up.
    - **⚠ THE STATUS IS *ON HOLD*, NOT *NEVER* — and the difference is load-bearing** (user,
      2026-07-28, ruling on the removal's fallout). *"History is coming, eventually, for the
      reasons worldgen states (not exhaustive)… we do want these systems **eventually**: they
      are effectively **on hold**."* Read *"they are NOTHING"* as a statement about **what
      exists**, never about **what is wanted**.
    - **⚠ AND IT GOVERNS UNRATIFIED CONTENT, NEVER RECORDED AMBITION.** The two are easy to
      confuse and the cost lands in opposite directions:
      - **Unratified bootstrap CONTENT** — code and world content fabricated during bring-up
        that nobody voted for. *No standing at any magnitude; the honest disposal is removal.*
        The settlement-history pass was exactly this, and it went (journal/0121).
      - **RECORDED AMBITION** — a user-authored statement of what the engine and the default
        pack **will be**. `docs/design/things-that-will-happen.md` is *entirely* this by
        charter (user, 2026-07-28: *"correctly 'what kind of engine this WILL BE and what kind
        of experience the default pack WILL BE'… recorded with a high amount of user
        involvement and are **not claims about what we currently have built** as content or
        can support as an engine"*). **It is not fabrication and must not be swept.** Its
        looted-sword-from-a-ruin line names a civ system that does not exist, and **stays.**
      - **The tell, when they are hard to tell apart:** *does it RUN, or does it PROMISE?* A
        thing in the tree asserts it exists — challenge it. A thing in a design doc written in
        the future tense asserts we want it — that is a user-owned claim, and striking it
        **retires a goal**, which no sweeper and no slice may do.
      - **Where a doc did assert a live pipeline stage** (`worldgen.md` § *Above the region
        scale* item 4), the fix ruled by the user was **mark ON HOLD, not strike** — the
        implementation claim is corrected, the ambition is preserved. That doc's banner and its
        § *Sequencing* are the template.
    - **⚠ THIS DOES NOT REACH `docs/design/ecology.md`, AND AN EARLIER DRAFT OF THIS LINE
      WRONGLY SAID IT DID.** The word *"ecology"* was **not** the user's — it was inserted by
      the assistant transcribing the directive, and it unmoored a doc whose own header reads
      *"substrate **RATIFIED** 2026-07-19 (user: 'this reads absolutely right'); the evolution
      architecture below is **the USER'S DESIGN**"*. **Two** live `stubs.md` entries name ecology
      as their heir (**#2** `stubs.md:123-124`, **#7** `:214-215`). *This sentence said "four"
      until 2026-07-28: entry #1 was **resolved by deletion** that morning and carries an explicit
      note at `stubs.md:64-65` saying it is **NOT** evidence for this clause — so the count went
      stale inside the very argument it supports, on the same day, and was caught by the baseline
      sweep hours later. **The conclusion is unaffected and the correction strengthens it.**
      Recorded rather than silently repaired because a wrong count in a read-first justification
      is exactly what an enumeration-completeness check would catch, and we still have none.* Caught by the first `doc-topology` sweep, hours after the line was
      written — **by the very rule two bullets above this one.** The doctrine targets
      **unratified bootstrap CONTENT**; it has no authority over a ratified design doc.
      *That an assistant widened a user directive by one word, in a file every session and
      every agent loads, is the sharpest available illustration of why that rule exists.*

- **A USER-ORIGINATED DESIGN MAY NOT BE SUPERSEDED BY AN IMPLEMENTATION SLICE** (2026-07-26,
  corrections #65 — the gap that cost three days and produced `DeepAxis`). The ratification
  protocol forbids *recording an unratified assistant proposal*. It said nothing about the
  mirror: **an assistant reconciliation quietly overwriting a design the user authored.**
  The user's fractional-phase scheduler sketch was filed as *"carried forward to… discuss
  next session"*; the comparison then happened **inside an implementation slice**, and its
  ORDER half died in a single clause of a design doc the user does not read. It was never
  contested — only reconciled.
  - **The rule:** when a slice's reconciliation contradicts, narrows, or replaces something
    user-originated, that is a **loud plea to main session**, exactly like a spine deviation.
    Never a doc edit. Mark it `⚠ CONTESTS <the user thing>` in the report and stop.
  - **Integrator half:** a design doc's clause that supersedes a user sketch needs the same
    ratification the sketch got. Grep for the sketch before writing the reconciliation.
  - *Note which way this cuts: the reconciliation was **defensible engineering** and the
    author believed it. The defect is not that it was wrong — it is that it was **unilateral
    and invisible**.*

- **Runtime perf is a first-class axis, and it is currently under-fought**
  (user, 2026-07-22): *"efficiency/perf at runtime while not compromising on
  content is extremely important… our game is getting slow and we've barely
  cracked the surface."* The two clocks are opposites by doctrine: **gen
  time is not a constraint** (ready-made worlds are the sanctioned answer);
  **runtime is sacred**. Content is never cheapened to buy frames — perf
  work means better mechanisms, not less world. A slice that touches a hot
  path (per-frame, per-tick, per-chunk-load) reports its measured cost in
  its RETURN spec like any other gate.

- **A summary is not an authority** (DECIDED 2026-07-21, ARCHITECTURE.md).
  A cheap answer written because a consumer cannot afford the real one must be
  *derived from* the real one, never become it. Test before committing: **"if
  this consumer disappeared tomorrow, would this code still exist in this
  shape?"** If no, it is a summary wearing an authority's clothes — give it a
  `stubs.md` entry naming its heir, and a test asserting it AGREES with the
  authority. A leaked requirement looks like working code that passes tests,
  which is why the stub inventory does not catch it.
- Headless crates (dc-core, dc-sim, dc-worldgen, dc-api, dc-physics) never
  depend on rendering/OS. dc-client is the only GPU/OS crate.
- Wire types: no `skip_serializing_if` (postcard is positional — corrections #3).
- All entropy flows from seeds owned by the caller; no wall clock, no ambient
  randomness in sim/worldgen code.
- Commits end with `Co-Authored-By: Claude <model> <noreply@anthropic.com>`, **naming the
  model that actually did the work** — e.g. `Claude Opus 5`, `Claude Fable 5`. *This line
  used to hardcode `Fable 5` and went stale the first time a session ran on another model
  (caught 2026-07-26 by an agent that followed its brief, noticed the conflict with this
  file, and reported it rather than picking one). The trailer is provenance: pinning one
  name makes it a lie the moment the roster changes.*
