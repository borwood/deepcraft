# deepcraft

Voxel game, bespoke Rust/Bevy stack. Internal codename; public name TBD.

## Read first

1. **[`ROADMAP.md`](ROADMAP.md)** — the living sequence (Shipped / In flight /
   Sequenced / Observed). Read before proposing work; update it in the same
   commit as any journal entry.
2. **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)** — decisions with dates;
   then `docs/API.md`, `docs/design/*.md`, `docs/rendering/PIPELINE.md`.
3. **[`journal/corrections.md`](journal/corrections.md)** — claims already
   falsified, with mechanisms. Check before re-deriving.
4. Spike results live in `docs/spikes/S*-results.md` — measured numbers,
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

## Conventions

- Headless crates (dc-core, dc-sim, dc-worldgen, dc-api, dc-physics) never
  depend on rendering/OS. dc-client is the only GPU/OS crate.
- Wire types: no `skip_serializing_if` (postcard is positional — corrections #3).
- All entropy flows from seeds owned by the caller; no wall clock, no ambient
  randomness in sim/worldgen code.
- Commits end with: Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
