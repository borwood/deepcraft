# deepcraft

Voxel game, bespoke Rust/Bevy stack. Internal codename; public name TBD.

## Read first

0. **[`docs/design/north-star.md`](docs/design/north-star.md)** — the **engine
   shape everything converges to** (ratified 2026-07-23): a native engine whose
   core is cell storage + a pass-runner + native field-solvers + a stable API,
   with materials, their behavior, and the passes over them **authored in a
   uniform, self-declaring, compiler-validated shape and tuned by data** —
   plugin-first, safely moddable by untrusted third parties (native `abi_stable`
   for trusted, WASM sandbox for untrusted, **one authoring shape**). It is the
   *destination*, pursued **evolutionarily** via seam-first conversions. **All
   design flows through it; divergence is a loud plea, never silent** (§
   Compliance). It is the strategic companion to spines — read both first.
0b. **[`docs/spines.md`](docs/spines.md)** — the recurring **shapes** (S-1…S-8),
   the **anti-shapes** (A-1…A-6), and the index of **machinery that exists and
   nothing calls**. spines names the shapes of the code *as it is today*; the
   north star names where it is *going*. This project's characteristic failure is
   re-inventing a mechanism *next to* the one it already built. **Work is
   justified against these shapes**; a deviation is loud, discussed in main
   session, ratified by the user, and recorded in its § 4. Update it in the
   same commit as work that adds an instance or empties a row of § 3.
1. **[`ROADMAP.md`](ROADMAP.md)** — the living sequence (Shipped / In flight /
   Sequenced / Observed). Read before proposing work; update it in the same
   commit as any journal entry.
2. **[`docs/design/things-that-will-happen.md`](docs/design/things-that-will-happen.md)**
   — one-line concrete examples of what this game IS. Read before the design
   docs; it loads the mental model fastest. Append to it whenever a genuinely
   informative new example surfaces.
3. **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)** — decisions with dates;
   then `docs/API.md`, `docs/design/*.md`, `docs/rendering/PIPELINE.md`.
4. **[`journal/corrections.md`](journal/corrections.md)** — claims already
   falsified, with mechanisms. Check before re-deriving.
5. Spike results live in `docs/spikes/S*-results.md` — measured numbers,
   don't re-guess them.

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

- **A gate is only evidence about the code it actually ran.** Agent worktrees
  share one `CARGO_TARGET_DIR`, and a sibling's stale artifact can be served
  as fresh — producing a **false green**: exit 0, every suite `ok`, and the
  code you just wrote never built (corrections #27; the silent mirror of #21's
  impossible red). So before a merge gate, `cargo clean -p <each crate you
  changed> --release`, and **verify by test name or count**, never by
  `test result: ok` alone. "Did it run?" is a separate question from "did it
  pass?"
- Capture `error` / `panicked` / `FAILED` lines, not only `test result:` lines
  — a compile failure is invisible to a test-result filter.
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
  wait until `Get-Process cargo,rustc` is empty. **And re-read
  `.agent-build.lock` before each cargo call — it has been observed clobbered by
  a sibling; a create-file mutex you never read back is not a mutex.**

## Agent walks

- Connect: run the game (`cargo run --release -p dc-client` from repo root),
  MCP at `http://127.0.0.1:7777/mcp` (streamable HTTP).
- **Testing anything non-shader-related? Launch with `--fullbright`** —
  unlit materials, pure vertex color — so lighting/tonemap output never
  masquerades as a geometry or data defect (journal/0004).
- Check `eye_in_solid` in every pose response before trusting a screenshot;
  use `pose_set { surface: true }` for walker-safe teleports. Pitch:
  negative looks down.
- **dc-client's exit code is now honest** (journal/0054): `0` clean, `70`
  GPU device lost, `71` fatal render error, `101` a panic on any thread. The
  old "exit codes lie about GPU crashes" warning is retired *for dc-client*.
  Reading the log tail is still the better habit — it names the cause, not
  just the class — but it is no longer compensating for a broken signal.
- Screenshots land in `journal/assets/` — name them `NNNN-description` for
  the journal entry they belong to.
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
- Commits end with: Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
