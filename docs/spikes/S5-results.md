# S5 — Plugin host + MCP over one surface: results

Status: spike complete, 2026-07-18. The core architectural bet — **one API,
many consumers** — holds: the same command sequence executed natively, from a
sandboxed WASM plugin, and through MCP tool calls produces bit-identical world
state and identical receipts, and none of the three layers is hand-written
twice (the MCP tool list and the plugin bindings both derive from the same
schema registry).

Code map (all kept — this is the v0 surface, not throwaway):

| Where | What |
|---|---|
| `crates/dc-api` | envelopes/receipts (`envelope.rs`), typed payloads (`payload.rs`), capability tokens (`capability.rs`), events (`event.rs`), schema registry (`schema.rs`), WASM ABI shapes (`abi.rs`), reference host world (`host.rs`) |
| `crates/dc-host` | wasmtime plugin host (`lib.rs`), the shared demo script (`demo_script.rs`), parity/determinism/verbosity tests |
| `crates/dc-mcp-dev` | generated MCP tool layer + rmcp server (lib), stdio bin, in-process MCP session test |
| `plugins/demo-builder` | the demo plugin, built to `wasm32-unknown-unknown` (excluded from the workspace; built by a separate cargo invocation that the dc-host tests trigger) |

Deps added (workspace-pinned): `wasmtime =46.0.1`, `rmcp =2.2.0`,
`serde_json 1`, `tokio 1.53`.

## Where the host world lives (scope decision)

The in-process world backing the API (`HostWorld`) lives **in dc-api**, not a
separate crate: it is the *reference implementation* of the surface's
semantics — tick quantization, the total order, txn atomicity, capability
enforcement — and the conformance target every consumer test runs against.
dc-host (wasmtime) and dc-mcp-dev (rmcp) are thin boundary layers over it.
When dc-sim grows the real world it must implement the same semantics;
HostWorld stays as the executable spec. World model: dc-core 32³ chunks
(lazily generated from the seed: grass-topped hill cells, surface between
y=−3 and y=−1), a `Vec` of simple entities, a manual `tick()` driver, no
rendering.

## The v0 slice

Commands: `dc:world/set_block`, `dc:world/fill`, `dc:entity/spawn`,
`dc:registry/define_item`, `dc:events/subscribe`.
Queries: `dc:world/get_block`, `dc:world/scan_region`, `dc:entity/query`,
`dc:events/poll`.

Semantics implemented and tested (`crates/dc-api/tests/host_semantics.rs`):

- **Tick quantization**: submit returns a `SubmitAck {consumer_seq,
  scheduled_tick}`; state is untouched until `tick()`; `target_tick` defers
  further; a target ≤ the current completed tick rejects
  (`TargetTickInPast`).
- **Total order**: `(tick, priority class, consumer id, per-consumer seq)` —
  the consumer id tie-break is an addition to API.md's documented triple (see
  § proposed revisions). Verified by interleaving player/plugin/scheduler
  writes to one voxel and checking the from→to chains in the effects.
- **Transactions**: all-or-nothing per `(source, txn)` within a tick, via an
  undo journal (blocks restored, entities unspawned, items undefined, subs
  removed); events from an aborted txn are never delivered; every member
  receipts `TxnAborted` with the failing command's id and reason.
- **Queries** run against the last completed tick (commands only apply inside
  `tick()`, so between ticks the live state *is* the snapshot — no copy), and
  never mutate world state. `events/poll` drains only the caller's own
  delivery channel; foreign subscriptions are indistinguishable from
  nonexistent ones.

## Envelope / receipt wire examples

The same shapes cross every boundary; JSON shown (postcard carries the
identical serde structure, ~7x smaller — see the measurement section).
An envelope from the demo session (the doorway punch-out), as the plugin
sends it (least-authority claimed grant):

```json
{
  "id": "dc:world/set_block",
  "source": {"kind": "Plugin", "name": "demo-builder"},
  "grant": {"grants": [{"WorldWrite": {"volume": {"min": {"x": 0, "y": 0, "z": 0},
                                                  "max": {"x": 10, "y": 15, "z": 10}}}}]},
  "payload": {"SetBlock": {"pos": {"x": 4, "y": 2, "z": 2}, "block": "dc:air"}},
  "target_tick": null,
  "txn": null
}
```

Submit acknowledgement (immediate), then the receipt (at the tick boundary):

```json
{"consumer_seq": 7, "scheduled_tick": 1}

{"seq": 8, "tick_applied": 1, "result": {"Ok": {
  "blocks_changed": [{"pos": {"x": 4, "y": 2, "z": 2}, "from": "dc:wood", "to": "dc:air"}],
  "entities_spawned": [], "items_defined": [], "subscriptions_created": []}}}
```

A rejection receipt (volume-scoped writer straying outside its grant):

```json
{"seq": 3, "tick_applied": 1, "result": {"Rejected": {"MissingCapability": {
  "needed": "world.write(16,1,1)..(16,1,1)"}}}}
```

A query receipt (from the stdio smoke session, `world_get_block` at a
seeded-terrain surface voxel):

```json
{"tick_observed": 0, "result": {"Ok": {"Block": {"block": "dc:grass"}}}}
```

## Capability model — outcomes

Deny by default; grants are `world.read/write(volume|anywhere)`,
`entity.spawn`, `registry.define(namespace)`, `events.subscribe`. Tokens
attenuate (`CapabilityToken::attenuate`): a child token must be
equal-or-narrower per grant, checked by the same subset relation
(`covers_token`) the trust boundaries use. All enforced and tested:

| Case | Outcome (test) |
|---|---|
| Empty token does anything | rejected `MissingCapability` (`deny_by_default…`) |
| Volume-scoped writer writes outside its volume | rejected; a fill *straddling* the boundary rejects atomically — not even the in-bounds corner applies |
| Volume-scoped reader scans outside | rejected (reads are scoped too) |
| `foo`-namespace token defines `bar:*` | rejected `MissingCapability(registry.define(bar))` (`namespace_ownership…`) |
| Attenuation widening (bigger volume, other namespace, unbounded from bounded, missing family) | refused at derivation (`attenuation_narrows…`) |
| WASM plugin claims a grant beyond its installed token | rejected **at the host boundary**, never reaches the world (`demo_plugin…` contraband + `narrower_install…`) |
| MCP session (`dev:*` token) defines `foo:*` | rejected through the wire with the receipt naming `registry.define(foo)` (`mcp_session…`) |

Honest v0 caveat: in-process, a token is plain serde data — enforcement is
real at the WASM/MCP boundaries (claimed ⊆ installed via `covers_token`, and
`source` must equal the installed identity), but an in-process consumer could
fabricate one. See proposed revision 2 (grant handles).

## WASM plugin ABI (as built)

Target: **wasm32-unknown-unknown** (not WASI). Plugins are pure compute
driving the world through envelopes; no clocks, no filesystem, no ambient
authority — WASI would only add surface to audit. Demo plugin artifact:
~59 KB (release, `opt-level = "s"`, stripped).

Encoding: **postcard** both directions (API.md § wire formats). One host
import, module `"dc"`:

```text
dc.call(req_ptr: u32, req_len: u32) -> u64        // (ptr << 32) | len
```

Request bytes are a postcard `PluginRequest`; the host writes a postcard
`PluginResponse` into guest memory it obtains by calling the guest's
`dc_alloc` (wasmtime re-entrancy — a host import calling back into a guest
export — works fine), and the guest owns the response buffer afterwards.

Guest exports: `memory`, `dc_alloc(len) -> ptr`, `dc_free(ptr, len)`,
`dc_run()` (session entry), `dc_tick()` (post-tick callback).

```rust
enum PluginRequest  { Submit(CommandEnvelope), Query(CommandEnvelope), DrainReceipts }
enum PluginResponse { Submitted(SubmitAck), SubmitRejected(CommandReceipt),
                      Query(QueryReceipt), Receipts(Vec<ReceiptEntry>), Error(String) }
```

`DrainReceipts` was forced by the tick model and is a finding in itself:
because receipts are tick-boundary artifacts, a submitting plugin *cannot*
get its receipt synchronously — it drains them from its `dc_tick` callback
(the demo plugin learns its subscription id this way). See proposed
revision 5.

The plugin claims **least authority per envelope** (an attenuation of its
manifest token computed per payload), which is what makes the
narrower-install test sharp: removing `registry.define` from the installed
token disables exactly the define command; the build still goes up.

Demo behavior (all proven in `wasm_plugin.rs`): subscribes to `BlockChanged`
in its yard, defines `demo:builder_wand`, builds a stone-floor/wood-wall hut
with a doorway, spawns a deer; after a *foreign* (editor) block change in the
yard it reacts by capping the block with wood — filtering self-caused events
by `cause`, so no feedback loop (verified quiescent).

Build plumbing: the plugin is excluded from the workspace (a host-target
cdylib with unresolved `dc.call` imports cannot link under
`--workspace --all-targets`); dc-host tests build it on demand with
`CARGO_TARGET_DIR` pointed at the plugin's own target dir, so it never
contends with the enclosing build lock.

## MCP server (generated, never hand-written)

`ToolLayer::tools()` maps `dc_api::schema::registry()` → rmcp `Tool`s: name =
id with `dc:` dropped and `/`→`_` (MCP tool names can't contain `:` or `/`),
description = doc + command/query kind + id + capability requirement, input
schema = the registry's hand-rolled JSON schema object, verbatim. Dispatch
goes back through `id_for_mcp_tool` + the registry's `decode_json` into the
typed payload — there is no per-tool code anywhere in dc-mcp-dev, and the
session test asserts tool count == registry count and schema equality, so the
surface cannot drift.

Session trust: the MCP caller never passes grants; the layer stamps every
envelope with the session-ambient token and consumer id.

Tick model: `auto_tick = true` (dev default) completes a tick per submitted
command and returns the `CommandReceipt`; `auto_tick = false` returns the
`SubmitAck` (used by the parity test to align tick boundaries).

What was proven, and how: `dc-mcp-dev/tests/session.rs` runs a **real rmcp
client against the real rmcp server** over a tokio duplex pipe — actual MCP
wire protocol (initialize → tools/list → tools/call), identical to stdio
minus the OS pipes. The session: lists 9 generated tools, defines
`dev:probe`, places wood + fills stone, scans the row back
(`palette ["dc:wood","dc:stone"], indices [0,1,1,1]`), gets a namespace
rejection for `foo:sneaky`, and gets METHOD_NOT_FOUND for an unknown tool.
This test is the in-tree session transcript (machine-checked rather than
prose). The stdio bin (`dc-mcp-dev [seed]`, same handler +
`rmcp::transport::stdio()`) was additionally smoke-tested with a raw
JSON-RPC session piped through stdin/stdout — initialize handshake,
generated tools/list, and a `world_get_block` call returning
`{"result":{"Ok":{"Block":{"block":"dc:grass"}}},"tick_observed":0}` for a
seeded-terrain surface voxel.

## Parity proof

`dc-host/tests/parity.rs`: the shared script (`dc_host::demo_script`,
`include!`d by the plugin so all paths execute byte-identical payloads — 10
commands) is run three ways against seed-99 worlds:

- (a) native: typed envelopes into `HostWorld::submit`, one tick;
- (b) WASM: `dc_run` submits through postcard over `dc.call`, one tick;
- (c) MCP: JSON tool calls through the generated layer (auto_tick off), one
  tick.

Asserted identical: region hash (FNV-1a over a 27×27×27 voxel box including
lazily generated terrain), entity lists, item registry, and the receipt logs
**modulo source only** — global seq, per-consumer seq, tick_applied, and full
effect lists all coincide. The plugin's extra contraband attempt is boundary-
rejected and leaves no trace in world state or the receipt log, which is
itself part of the proof: an escalation attempt is invisible at the API
layer.

## Determinism

- `dc-api/tests/host_semantics.rs::replay_determinism`: same seed + same
  envelope log (multi-consumer, multi-tick, fills/spawns/priority
  interleavings) ⇒ identical region hash, identical receipt log, identical
  entities; different seed ⇒ different hash.
- `dc-host/tests/parity.rs::wasm_replay_is_deterministic`: two full plugin
  runs are bit-identical (hash + receipt log); seed+1 diverges.

The replay bet from API.md holds for this slice: `seed + command log` is a
complete description of world state.

## Receipt verbosity (open question 4) — measured

`dc-host/tests/receipt_verbosity.rs` (run with `--nocapture`), demo session,
bytes per message; `pc` = postcard (WASM boundary), `js` = JSON (MCP
boundary); `sum` = the same receipt with count-only `EffectsSummary`:

| command | env pc | env js | rcpt pc | rcpt js | sum pc | sum js |
|---|---|---|---|---|---|---|
| events/subscribe | 66 | 483 | 8 | 133 | 7 | 128 |
| registry/define_item | 135 | 506 | 25 | 151 | 7 | 128 |
| world/fill (25 changes) | 66 | 446 | 482 | 1631 | 7 | 129 |
| world/fill (15 changes) ×2 | 65 | 445 | 277 | 1016 | 7 | 129 |
| world/fill (9 changes) ×2 | 65 | 445 | 169 | 662 | 7 | 128 |
| world/set_block ×2 | 66 | 427 | 25 | 190 | 7 | 128 |
| entity/spawn | 85 | 433 | 8 | 134 | 7 | 129 |
| **TOTAL (10 commands)** | **744** | **4502** | **1465** | **5785** | **70** | **1284** |
| world/scan_region (query, 1936 voxels) | 64 | 443 | 1972 | 4029 | — | — |

Readings:

- Full-effects receipts scale with blocks touched (~19 B postcard / ~65 B
  JSON per block change). A single legal 64³ fill would receipt at ~5 MB
  postcard / ~17 MB JSON. Untenable as the default.
- Summary receipts are effectively free (7 B postcard) and constant-size.
- Envelope JSON (430–500 B) is dominated by the echoed grant token; the MCP
  boundary already avoids this (tool args are just the payload, 60–120 B;
  the envelope is server-internal), and grant handles (revision 2) would fix
  it everywhere.
- The scan query shows reads are the right place for bulk data: 1936 voxels
  cost 1972 B postcard (~1 B/voxel via palette+indices), *cheaper than the
  fill receipt per voxel touched*.

**Recommendation** (revision 3): receipts carry `EffectsSummary` plus a
capped effect sample (first N ≈ 8, enough for single-block commands to stay
fully informative); consumers that need the full delta scan the region or
subscribe to events — both already exist and are better shaped for bulk.

## Friction with the API.md design → proposed revisions

(No edits made to API.md; proposals only.)

1. **Total order is underdetermined between same-class consumers.** API.md
   says `(tick, priority class, per-consumer seq)`; per-consumer seqs from
   different consumers are incomparable. Implemented and proposed:
   `(tick, priority class, consumer id, per-consumer seq)` with `consumer id`
   ordered by (kind, name). Determinism requires *some* documented
   tie-break; lexicographic consumer id is inspectable and stable.
2. **Grant-by-value is a forgery hazard and a bandwidth tax.** The envelope
   carries the whole token; in-process nothing prevents fabrication, and it
   is ~350 B of every JSON envelope. v0 mitigates at trust boundaries
   (claimed ⊆ installed, source must match). Propose: hosts issue opaque
   **grant handles** (small ints) at install/session time; envelopes carry
   the handle; attenuation mints new handles. Value-tokens remain a
   description format for manifests/user approval UI.
3. **Receipts: summary + capped sample** (see measurement above). Resolves
   open question 4.
4. **`events/poll` is not a pure query.** It drains the caller's channel.
   Propose API.md distinguish *world-pure* queries (get/scan/entity.query)
   from *channel operations* (poll), or bless "queries may mutate only
   caller-owned delivery state". Also worth considering: push delivery into
   `dc_tick` for plugins (the poll round-trip is 2 messages per tick even
   when empty).
5. **Receipts are asynchronous for tick-quantized consumers — document the
   `SubmitAck` flow.** API.md's `CommandEnvelope → CommandReceipt` arrow
   reads synchronous. Reality: submit returns
   `SubmitAck {consumer_seq, scheduled_tick}`; receipts materialize at the
   tick boundary (plugins drain them via `DrainReceipts` from `dc_tick`; the
   dev MCP surface hides this with auto-tick). This shape should be
   normative — it is also exactly what a future network boundary wants.
6. **Wire-format rule: no `skip_serializing_if` on envelope/receipt types.**
   Postcard is not self-describing; conditional field skipping silently
   corrupts the WASM boundary (found the hard way: the plugin could not
   decode its own receipts; guarded by an ABI round-trip test). Add to
   API.md § wire formats.
7. **`registry.define` wants multi-namespace or pattern grants.** A dev MCP
   session realistically owns several namespaces; today that is N separate
   grants (workable but noisy). Propose allowing a set or glob
   (`registry.define({dev, dc})`) — attenuation stays subset-based.
8. **Document the MCP name mapping** `dc:domain/verb_noun` →
   `domain_verb_noun` as the canonical, reversible convention (MCP tool
   names cannot contain `:` or `/`).
9. **Envelope id / payload redundancy.** The string id and the typed payload
   variant must agree (host rejects mismatches). Fine at 9 commands;
   generated bindings (from the registry, like the MCP layer) should own
   envelope construction so the redundancy can't be misassembled by hand.
10. **`Payload`/`QueryData` enums are a closed world.** Third-party commands
    (plugins *adding* commands, not just calling them) don't fit the enum.
    v0 is deliberately closed; the registry entry should eventually carry
    encode/decode for open payloads (schema-validated bytes) — noted so the
    enum doesn't ossify into the contract.

Minor implementation notes: wasmtime 46's `Caller::get_export` +
re-entrant `TypedFunc::call` made the guest-alloc response path trivial;
rmcp 2.2's `ServerHandler` (native async trait methods, `structured_content`
results) fit the generated layer with no macros; `HostWorld::query` takes
`&mut self` purely for lazy chunk generation + poll draining — snapshot
semantics survive because mutations only happen inside `tick()`.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --release -- -D warnings`,
`cargo test --workspace --release`: all green (Windows dev box, 2026-07-18).
dc-client/dc-sim/dc-worldgen untouched.
