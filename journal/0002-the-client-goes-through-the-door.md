# 0002 — The client goes through the door

*2026-07-18 · first post-spike integration*

> blogworthy: the moment a game becomes its own dev tool — the running client
> hosts the same command surface its spikes proved, player clicks and agent
> tool calls become the same envelopes, and the game can photograph itself
> into its own journal.

The spike era ended with a strange asymmetry. S5 proved "one API, many
consumers" three ways — native, WASM, MCP — but every consumer drove a
*reference* world in a test harness. The actual game, the thing with a window
and a walking player, still mutated nothing (S1 had no edits at all) and
answered to nobody. This milestone closes that gap: the client becomes a
dc-api consumer, and simultaneously an observable surface an agent can drive,
query, and photograph. Both halves matter — the first makes edits
deterministic, the second makes walks scriptable (orogeny's 0033 pattern:
the agent walks the world and files field reports).

## Who owns the world? (the generator reconciliation)

The client already had a world: `TerrainGen`, streamed into a `ChunkMap`,
meshed and collided against. dc-api already had a world too: `HostWorld`, the
executable spec of the surface's semantics, generating its own toy hill-field.
Two worlds, one game — the classic setup for a slow-motion consistency
disaster.

The wrong turn we didn't take: making `ChunkMap` the authority and teaching
dc-api to talk to it. That inverts the architecture — the reference host
implements tick quantization, the total order, txn rollback, capability
enforcement; reimplementing those against a render cache is exactly the drift
S5 exists to prevent.

Instead the host came to the client, and the terrain went to the host:
`HostWorld` gained a **pluggable generator closure** (additive change —
`HostWorld::with_generator(seed, Box<dyn Fn(ChunkPos) -> Chunk>)`; the
built-in hill-field remains the default so every S5 consumer and test is
untouched). The client builds its authority as `HostWorld` over a closure
that calls the same `TerrainGen` at the current scale. Now:

- **`HostWorld` is the authority for edits.** Player input and MCP tool calls
  both submit `dc:world/set_block` envelopes (player priority class for
  input); the host ticks on a fixed 20 Hz accumulator inside the Update
  chain, in the S5 total order.
- **`ChunkMap` demoted to a render/collision cache.** Chunks stream in as
  *clones of the host's chunks* (`HostWorld::chunk`, the second additive
  accessor) — terrain plus applied edits — and edits reach the cache only by
  receipt. There is no code path that writes a block into `ChunkMap` except
  "a receipt said so".
- dc-worldgen stays out of this milestone; `TerrainGen` remains the
  generator on both sides of the door.

The pleasant surprise: because the generator closure is the *same seeded
noise*, the cache's old "fall back to the generator for unloaded chunks"
trick (collision, border meshing) remains exactly correct for never-edited
chunks — no reconciliation code needed for the common case.

## The edit loop: click → command → receipt → mesh → collider

Breaking a block is now a five-stage pipeline, and every stage was already
designed by a spike:

1. **Aim**: a DDA voxel raycast (Amanatides & Woo, new additive `dc-core::
   raycast` module with axis/diagonal/negative/inside-solid tests) walks the
   same solidity query collision uses. A crosshair dot plus a gizmo outline
   on the targeted voxel makes the aim visible in screenshots.
2. **Command**: LMB submits `set_block(dc:air)`, RMB places `dc:stone`
   against the hit face (skipped if it would intersect the player's AABB).
   The input handler never touches the world.
3. **Tick**: the host applies at the 20 Hz tick boundary in the S5 total
   order. Receipts come back with full effect lists.
4. **Cache + mesh**: receipt effects are applied to loaded `ChunkMap` chunks;
   the dirty set is the containing chunk plus face-adjacent chunks when the
   voxel sits on a border (culled meshing only consults the 6-neighborhood,
   so edge/corner diagonals don't dirty — tested). Dirty chunks remesh
   in-place next frame.
5. **Physics**: each changed voxel drops the covering dc-physics collider
   tile — the S6 open question ("world edits vs live bubbles"), closed with
   the predicted one-`BTreeMap`-remove plus one wrinkle S6 didn't predict:
   tile colliders are parentless statics, so removing one wakes nobody.
   `PhysicsWorld::invalidate_voxel` (additive) therefore explicitly wakes
   dynamic bodies whose bubble overlaps the tile; without that, a sleeping
   item floats forever on the memory of a block you removed (the new
   dc-physics test proves both halves: stale-without-invalidation, then
   wake-and-fall-with).

Latency budget: worst case ~50 ms command-to-remesh at 20 Hz. Imperceptible
today; if it ever isn't, the cadence is one constant.

## The observability harness

The client now runs an MCP server *inside the game*: rmcp over **streamable
HTTP** on `127.0.0.1:7777` (`--mcp-port` to move it, `--no-mcp` to kill it),
on a dedicated tokio thread. Transport choice: stdio is taken (the game's
stdout is a log), and streamable HTTP is what MCP clients expect for attach-
to-a-running-process; rmcp's server is a tower service, so a minimal hyper
HTTP/1 accept loop drives it — no axum, no router, loopback-only Host
validation by default (DNS-rebinding guard comes free from rmcp).

The tool surface is two layers:

- **The dc-api surface, generated.** The same registry-driven generation as
  dc-mcp-dev — literally the same code: `registry_tools()` and
  `envelope_for_tool_call()` were extracted in dc-mcp-dev's lib (the refactor
  this milestone owed) and are now shared by the stdio dev server and the
  in-client server. Adding a command to the schema registry adds the tool to
  both servers; neither can drift.
- **Three client-shell tools**, deliberately *outside* the registry because
  they touch the window, not the world: `client_screenshot { name }` (captures
  the next rendered frame to `journal/assets/<name>.png` and returns the
  absolute path — names are validated as bare slugs, `[a-z0-9_-]{1,64}`,
  because MCP callers are semi-trusted and the name becomes a filesystem
  path), `client_player_pose_get`, and `client_player_pose_set` (teleport +
  look, a dev-grant tool).

Threading is channels all the way down: tool calls land on the tokio thread,
cross to the ECS as bridge requests, and await oneshot replies. Queries
answer within a frame from the last completed tick; commands answer when
their receipt materializes at the tick boundary — the S5 `SubmitAck`/receipt
split, surfaced honestly instead of hidden behind auto-tick. Screenshots
answer only after the PNG is on disk (we encode it ourselves rather than
using Bevy's `save_to_disk` observer precisely so failures reach the caller).

**The session grant set** (documented as promised): the in-client MCP session
holds the same dev-grant token as dc-mcp-dev — `world.read(anywhere)`,
`world.write(anywhere)`, `entity.spawn`, `events.subscribe`,
`registry.define(dev)`. This is the *dev surface*; the character surface with
diegetic grants is next on the roadmap and is a different token, not a
different mechanism.

## Determinism held

The S5 parity pattern extends to the client's hosted configuration, headless:
a scripted edit session (eight `set_block`s including a chunk-border pair and
an edit/un-edit collision) produces an **identical region hash and identical
receipts modulo source** whether submitted as typed envelopes through the
player path or as JSON through the MCP tool layer — and a third run over the
real streamable-HTTP wire (initialize → tools/list → tools/call, hand-built
requests against the actual service, an ECS stand-in pump driving the host
tick) lands on the same hash. Seed+1 diverges, so the generator really is in
the loop.

## Honest limits

- **Edits don't survive a scale switch** (2/3/4 keys rebuild the authority —
  the lattice the edits lived on is gone). Pending MCP calls get a "world
  reset" error. Fine for a dev harness; a persistent world will want the
  command log instead (which is the whole point of the command log).
- **The far field doesn't see edits.** farmesh still samples `TerrainGen`
  LOD grids directly; a broken block un-breaks itself at 96 m. The S3
  "far field should become summary-shaped" item will subsume this.
- **The host never evicts chunks.** Every chunk ever streamed (or edited)
  stays in the authority's map, ~64 KiB each. Roaming far accumulates
  memory. Needs an eviction policy for never-edited chunks (they're pure
  generator output and can be dropped freely) — noted in Observed.
- **An edited-but-unloaded neighbor can mis-cull a border face** until it
  streams in (the meshing fallback samples the generator, not the host).
  Self-healing on load; not worth a fix before async meshing.
- **One MCP consumer identity** (`client-http`) for all concurrent HTTP
  sessions — they share a command seq. Per-session identities are trivial
  when needed.
- `client_screenshot` resolves `journal/assets` against the process cwd, so
  run the client from the repo root if you want the journal to receive the
  images (which is the point).

## What a first agent self-walk should try

Connect (`claude mcp add --transport http deepcraft http://127.0.0.1:7777/mcp`
against a running `cargo run --release -p dc-client`), then:
`client_player_pose_get` → `client_screenshot` to see where you are →
`world_scan_region` around your feet → teleport somewhere interesting →
break/place a few blocks via `world_set_block` → screenshot again → compare
what the scan claims against what the screenshot shows. The gap between
those two is where field reports come from.
