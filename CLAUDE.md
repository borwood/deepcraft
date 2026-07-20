# deepcraft

Voxel game, bespoke Rust/Bevy stack. Internal codename; public name TBD.

## Read first

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
with a `> blogworthy:` line naming the angle. Field reports from walks go to
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

## Agent walks

- Connect: run the game (`cargo run --release -p dc-client` from repo root),
  MCP at `http://127.0.0.1:7777/mcp` (streamable HTTP).
- **Testing anything non-shader-related? Launch with `--fullbright`** —
  unlit materials, pure vertex color — so lighting/tonemap output never
  masquerades as a geometry or data defect (journal/0004).
- Check `eye_in_solid` in every pose response before trusting a screenshot;
  use `pose_set { surface: true }` for walker-safe teleports. Pitch:
  negative looks down.
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

- Headless crates (dc-core, dc-sim, dc-worldgen, dc-api, dc-physics) never
  depend on rendering/OS. dc-client is the only GPU/OS crate.
- Wire types: no `skip_serializing_if` (postcard is positional — corrections #3).
- All entropy flows from seeds owned by the caller; no wall clock, no ambient
  randomness in sim/worldgen code.
- Commits end with: Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
