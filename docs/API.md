# dc-api command surface — v0 design

Status: pre-S5 design, 2026-07-18. S5 implements a thin slice of this and its
exit criterion is revising this doc into the v0 conventions spec. Prior art
deliberately mined: the author's Minecraft MCP mod (block/entity/level/events/
structure/schedule tool families) — a field-tested answer to what agent
consumers actually need.

## Principles

1. **One door.** Every gameplay mutation is a Command; every read is a Query;
   every push is an Event delivery. This includes vanilla gameplay: player
   input (break block, open door) becomes commands too. The renderer may read
   world state directly (rendering is not a consumer); nothing else touches
   sim state except through the door.
2. **Commands are data.** Serde-serializable envelopes, not trait calls. The
   same envelope crosses the WASM boundary, the MCP boundary, and (later) the
   network boundary unchanged.
3. **Determinism is the payoff.** Commands apply at tick boundaries in a
   defined total order; `seed + command log = replay`. This buys: save-game
   robustness, bug repros, multiplayer (later, command log IS the wire
   protocol), and time-travel debugging. This is why player input goes through
   the door.
4. **Capability-scoped, deny by default.** A consumer holds explicit grants;
   there is no ambient authority. Plugins declare needs in a manifest; users
   approve; MCP sessions and editors get granted scopes the same way.
5. **Reflectable.** Commands/queries register machine-readable schemas. The
   MCP tool list, WASM binding glue, and editor property UIs are *generated*
   from the registry — the surface cannot drift from its consumers.

## Envelope

```rust
CommandEnvelope {
    id: CmdId,                  // namespaced: "dc:world/set_block"
    source: ConsumerId,         // plugin instance | mcp session | editor | player
    grant: CapabilityToken,     // must cover id + payload scope
    payload: Payload,           // typed per-command; serde
    target_tick: Option<Tick>,  // default: next tick boundary
    txn: Option<TxnId>,         // atomic batch membership
}
→ CommandReceipt { seq, tick_applied, result: Ok(Effects) | Rejected(Reason) }
```

- **Ordering**: applied at `(tick, consumer priority class, per-consumer seq)`.
  Player input gets the highest priority class; plugins/MCP next; scheduled
  jobs last. Total order ⇒ determinism.
- **Transactions**: all-or-nothing batches ("save this blueprint" = one txn).
  No isolation beyond atomicity in v0 — txns apply within one tick.
- **Queries** run read-only against the last completed tick's snapshot; they
  never block the sim.

## Domains (v0 sketch)

| Domain | Commands (mutate) | Queries (read) | Events |
|---|---|---|---|
| `world` | set_block, fill, clone_region, stamp_structure | get_block, scan_region, raycast, column_summary, region_snapshot | block_changed, chunk_loaded/unloaded |
| `entity` | spawn, despawn, set_components, apply_effect, teleport | query (by volume/tag/component), get | spawned, died, entered_volume |
| `inventory` | set_slot, swap, transfer | get, count | changed |
| `registry` | define/update/retire: block_type, item, model, animation, blueprint, biome, feature | get_def, list(namespace) | def_changed (hot reload signal) |
| `sim` | observe (diegetic — see below) | inspect (out-of-band), query_distribution | collapse_occurred |
| `schedule` | at_tick, every_n_ticks, cancel | list | fired |
| `events` | subscribe(filter), unsubscribe | list_subscriptions | — (the delivery channel itself) |

Content packs are just recorded `registry` command batches — the in-game
editors emit them, plugins ship them, "vanilla" is the first one. Model +
animation payloads follow the transformed-cubes/keyframe format
(Blockbench-shaped, see ARCHITECTURE.md § Content model).

## Observation vs inspection (the constraint-ledger interaction)

Reading distant fluid state is not free — per the sim design (S2), an
observation *commits facts* to the constraint ledger and can force collapse.
The API therefore splits reads of non-full-sim state:

- `sim.observe` — **diegetic**. Requires an in-world vantage (an entity the
  consumer controls, a psionic power, a scrying artifact). Commits facts,
  triggers bounded collapse, is subject to game rules and costs. This is a
  *command* (it mutates the ledger), not a query.
- `sim.inspect` — **out-of-band**. Dev/creative tooling: returns
  distributions or samples WITHOUT committing anything. Capability-gated
  separately (`sim.inspect` is a creative-mode/dev grant); a survival-mode
  MCP agent session simply doesn't hold it.

Whether a given MCP session is diegetic (an agent playing the world, whose
glances collapse it) or an out-of-band tool is purely a question of which
grants it holds — the mechanism supports both.

## Characters, controllers, and the two MCP surfaces

**Decided 2026-07-18.** AI-driven characters are a first-class feature, not a
dev affordance: players may create multiple characters on a world and hand any
of them to an AI (start two characters, give one to Claude).

- **Character** is a primitive: a persistent entity owned by a player account,
  existing in the world independent of who (or what) is driving it.
- **Controller** is a binding: whoever holds `entity.control(character)` right
  now — human input, an MCP session, a WASM script, or nobody (idle/NPC-tier).
  Handing a character to an AI is just re-granting control; no special path.
- A character acts through the same command door with the same priority class
  as human player input, and its observations are diegetic: an AI companion
  seeing something commits facts exactly as a human-driven character would.
  Same world, same consequences, same physics of attention.

This yields **two MCP surfaces** (likely two servers, to keep tool lists tidy):

| | `dc-mcp-dev` | `dc-mcp-character` |
|---|---|---|
| Audience | building the game, creative tooling | AI companions in the shipped game |
| Grants | broad world.*/registry.*/`sim.inspect` | `entity.control(one character)` + that character's senses |
| Reads | out-of-band, commit nothing | through the character's senses only — raycasts from its eyes, hearing range, view capture; all diegetic |
| Precedent | the author's `minecraft` server MCP | the author's `minecraft-client` sense/view MCP |

Both are thin skins over dc-api — the character surface is dc-api filtered
through a grant set, not a second API.

Open (deferred to the sim/character spike): when an AI-driven character's
session disconnects, does it degrade to coarse-tier NPC behavior or freeze?
And whose compute runs a companion's cognition in multiplayer?

## Capabilities

```
world.read(volume | anywhere)      registry.define(namespace)
world.write(volume | anywhere)     registry.read
entity.control(selector)           sim.observe(via entity selector)
entity.spawn(types)                sim.inspect          # dev/creative
schedule.manage(own)               events.subscribe(filters)
```

- Tokens are scoped and attenuable (a plugin can hand a narrower token to a
  sub-component, never a broader one).
- `registry.define` is namespace-owned: plugin `foo` writes `foo:*` defs only.
- Editors run with the player's grants; in survival that means an editor can
  author *definitions* but placing the result in-world costs materials like
  any build (design intent — revisit in the editor spike).

## Wire formats

- In-process (editors, client input): typed Rust enums, zero serialization.
- WASM boundary: compact serde (postcard) + bindings generated from schemas.
- MCP: JSON; the rmcp tool definitions are generated from the same schemas.
- Network (later): the envelope over a transport; nothing new to design.

## Versioning

- Pre-1.0: the whole surface is `v0`, breakable while spikes land.
- From 1.0: additive-only per domain; commands carry no per-call version —
  instead retired commands keep accepting old payloads through serde defaults
  for one deprecation cycle. Schema registry records `since`/`deprecated`.

## Decisions log

1. **Id naming** — CONFIRMED 2026-07-18: `dc:world/set_block`
   (namespace:domain/verb_noun).
2. **Player input as commands** — CONFIRMED 2026-07-18: yes, from day one.
   Rationale beyond determinism: replay is the substrate for an AI-native dev
   pipeline — reproducing a bug or scaffolding a test-world state is
   "replay this command log," which agents can generate and rerun.
3. **MCP posture** — CONFIRMED 2026-07-18: dev-tool surface while building;
   embodied character surface as a first-class shipped feature (see
   § Characters). Two MCP servers over one API.

## Open questions

4. **Effects in receipts**: how much does a receipt echo back (full effect
   list vs summary counts)? Affects WASM boundary chattiness; measure in S5.
