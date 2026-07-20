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

- **Ordering**: applied at `(tick, consumer priority class, consumer id,
  per-consumer seq)` — the consumer-id tie-break (added by S5) makes the order
  total across consumers in the same class. Player input gets the highest
  priority class; plugins/MCP next; scheduled jobs last. Total order ⇒
  determinism.
- **Submission is async**: submit returns a `SubmitAck` immediately; the
  `CommandReceipt` is a tick-boundary artifact delivered afterwards (plugins
  drain receipts; MCP sessions poll/stream them). Normative as of S5.
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

**Pack-degradation doctrine — DECIDED 2026-07-19 (user).** Players will run
messy, experimental, willy-nilly plugin lists; that is anticipated, not
misuse. Pack-content problems therefore **inform and degrade — they never
fail the game**: the result is playable by construction, with warnings
naming what was skipped and why (loudness is the invariant — no world
content may *silently* depend on installed-pack coincidence, geology.md
§ unfilled slots). Individual malformed defs still reject at define time
with `SchemaViolation` — that rejection *is* the informing mechanism at
authoring time, and it never takes the world down with it. Instances so
far: unfilled framework classes (eventual warn-and-skip at world creation),
orphan anim clips (lint warning).

<!-- EDITED 2026-07-19 (geology backbone slice 1); accepted as-built,
     integrator review 2026-07-19 (engineering internals within the ratified
     geology design — see docs/design/geology.md for the user-owned frame) -->
**Content-class registry (added 2026-07-19).** A second
registry beside the command registry: **classes as contracts**
(docs/design/geology.md). Two verbs, both ordinary registry CommandSpecs (so
MCP surfaces grow the tools automatically):

- `dc:registry/define_content_class` — declares a namespaced class (e.g.
  `dc:stratum/clastic-fine`) **and its parameter contract**: a list of
  `ParamSpec { name, kind, required }` where `kind` is one of
  `Number{min,max}`, `Range{min,max}`, `Text`, `MaterialName`,
  `Choice{options}`.
- `dc:registry/define_class_member` — registers a member into a class,
  carrying `params: [(name, value)]` over the closed value vocabulary
  (`Number | Range | Text`), **validated against the class contract at
  define time** (unknown keys, missing required keys, kind mismatches, and
  out-of-bounds values all reject with `SchemaViolation`).

Namespace rule unchanged from S5: the grant must own the namespace of the
thing being *named* — for members that is the member's namespace, so a
plugin registers `foo:geo/…` into `dc:stratum/…` (joining a foreign class is
the point). `dc_api::classes` bridges registered defs into the canonically
ordered typed `GeologySet` (members sorted by namespaced id, abundance
normalized within class) that worldgen selection consumes. The vanilla geology
pack is generated from the typed set, so pack and model cannot drift.

<!-- EDITED 2026-07-19 (3d); accepted as-built, integrator review 2026-07-19 -->
**Per-class contracts (the formation-context correction, 2026-07-19).** The
geology contract is no longer one shared param set. Which context flows into
fitness is now class-dependent — the machinery is unchanged, the *contract*
is:

- **Clastic / placer** classes bind fitness to the deposition *weather*, so
  their contract carries the `temp_c`/`precip` ranges (year-zero climate is
  ratified-correct for the surficial veneer) alongside `depth_m`, `abundance`,
  `habit`, `hardness`, `erodibility`, `material`.
- **Igneous / accessory** classes are **province/depth-driven** and their
  contract carries **no weather params at all** (`geology_param_specs(class)`
  omits `temp_c`/`precip` for them). A granite is structurally prevented from
  ever being handed the weather: an igneous member supplying `temp_c` rejects
  as an unknown param at define time. The worldgen igneous windows leave those
  axes unbounded, so weather cannot enter selection even internally
  (`FormationWindow::igneous`). Province gating stays in the pass; depth is the
  member-differentiating axis. *(RATIFIED 2026-07-19: v1 keeps the province
  gate in the pass; no provisional tectonic-setting `Choice` param — the
  setting vocabulary is 3e's to design from pregen history, and 3e promotes
  `setting` into the class contract then. Per the no-bandaid razor.)*

**Class-satisfiability enforcement (two layers; geology.md § unfilled slots).**
A class a gen pass selects from can never be silently empty:

- **Define-time (`dc_api::classes::validate_pack`).** A content-pack batch must
  register at least one member for every class it *declares* in the same batch
  ("a pass is a pack"). Rejects by name. *(Decided 2026-07-19: rides as-built
  for now — NOT ratified as final doctrine. The eventual shape (user):
  unfilled framework classes don't hard-reject at define time; world creation
  raises a named warning and skips the unfilled class/pass — a LOUD skip,
  preserving geology.md's actual principle (no silent pack-coincidence
  content). Nothing in the current shape precludes that: it's a policy swap
  at the existing satisfiability checkpoint, which already names pass and
  class. Low priority; revisit when a real framework pack exists.)*
- **World-build-time (`Pipeline::check_class_satisfiability`, run by
  `WorldGenerator::try_with_geology[_owned]`).** A world refuses to build if any
  class a registered pass's `selects` names has zero members — naming pass and
  class, the same philosophy as the cycle/ambiguous-writer rejections. The
  infallible `with_geology[_owned]` constructors panic on an unsatisfiable set;
  vanilla is always complete.

**Accessory inclusions as pore partials.** The new `dc:accessory/mafic` class
(vanilla member `dc:geo/olivine`) is emplaced by the igneous pass into the host
rock's **pore slots** (structure = host, pore = accessory) — the placer pattern
in igneous dress, through the `VoxelContents` canonical constructors, rendered
sparse by the existing 3c-2 face dither with no renderer-side code.
<!-- END EDIT -->

<!-- EDITED 2026-07-19 (body-plan staircase steps 1–2) — the bodies registry;
     accepted as-built, integrator review 2026-07-19 (implements the ratified
     bodies.md design; schema shapes are engineering internals) -->
**Bodies registry (added 2026-07-19).** Body plans and
animation clips as registry data (docs/design/bodies.md; the fourth
roles-as-contracts instance). Two more ordinary registry CommandSpecs, so the
MCP surfaces grow the tools automatically — both `registry.define(namespace)`
grants, namespace-owned like every other def:

- `dc:registry/define_anim_clip` — a clip: `{ name, doc, duration_s, loops,
  keyframes: [{ t, root_bob_m, rotations: [{ segment, euler:[x,y,z] }] }] }`.
  Angles are XYZ Euler radians; keyframe `t` is strictly ascending in
  `[0, duration_s]`. Clips are standalone data (they name joints, not a plan),
  so they are defined **before** the plan that binds them. Validated at define
  time (finite positive duration, ascending in-range times).
- `dc:registry/define_body_plan` — a plan: `{ name, doc, segments, slots }`.
  `segments` is a joint-tree of cuboids `{ name, parent|null, pivot_m:[3],
  size_m:[3], offset_m:[3], tint:[3] }` (`pivot_m` is the joint's offset from
  its parent's pivot; `offset_m` places the cuboid relative to that pivot).
  `slots` binds driver verbs to clips `{ verb, clip }`.

The **verb→slot contract** is the schema-checked-at-define-time rule
(bodies.md): `define_body_plan` **rejects** (`SchemaViolation`) if the joint
tree is malformed (no root / bad parent / cycle / dup name / non-positive
size), if a slot names an unknown verb, or if a required verb's slot is
unfilled or bound to a missing/joint-incompatible clip. **v0 verb vocabulary**:
`idle`, `walk`, `jump`; **required**: `idle` + `walk` (locomotion is
mandatory), `jump` optional (a minimal authored pose or documented fallback).
The vanilla `dc:body/biped` (trunk, neck, head, two upper/lower arms, two
upper/lower legs) with hand-authored `idle`/`walk`/`jump` clips is the first
bodies pack, generated from the in-repo authored source so pack and model
cannot drift. **Firewall**: plans/clips are pure data; the stepped ~12 fps
sampler + crossfade player + segment renderer are client-only (dc-client
`body.rs`/`character.rs`) and never read back into sim (the sampler reads body
velocity one-way to pick idle vs walk). `Payload`/`Effects`/`RejectReason`
grew only appended variants/fields (postcard wire identity preserved).
*(Resolved 2026-07-19: **clips standalone RATIFIED** — clips name joints and
predate plans, so forked plans reuse ancestor clips by retained joints and
clips share across plans; orphan clips are a **warning, never a failure**
(lint concern). Schema fields, verb vocabulary + required set, and the
render quantization constants (12 fps / 32-step / 5 mm) ride as-built —
engineering internals and tuning within the ratified bodies design; the
vocabulary grows with gameplay verbs, the constants await the filed
photo-driven tuning pass.)*
<!-- END EDIT -->

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

Disconnect — v0 DECIDED 2026-07-19: **freeze**. A session disconnect zeroes
the character's move intent; the body stands where it was left. NPC-tier
degradation is a later, richer *controller* binding on the same rail
(bodies.md § formerly open questions) — never body-side behavior. Still
open: whose compute runs a companion's cognition in multiplayer?

**Attach placement (added 2026-07-19; RATIFIED 2026-07-19).** The character
surface's `character_attach` guards placement: if the body's AABB at the target
feet overlaps solid voxels the attach is refused with a machine-readable receipt
(`{ ok: false, code: "obstructed", ... }`) and nothing is spawned — an embedded
body is a permanent statue (no despawn verb, no character-surface teleport, both
by design). An opt-in `surface: true` first snaps the feet to the **true voxel
surface** (top solid voxel under the footprint, edits included) at the requested
x/z, mirroring the player's `surface` teleport. Ratified 2026-07-19: the
`surface: true` attach semantics; dev-grant `spawn_character` staying
unguarded (dev surface keeps full reach); the refusal receipt as the attach
flow's JSON `{ ok, code }` convention (a typed dc-api `RejectReason` can
supersede it if the pattern recurs).

<!-- EDITED 2026-07-19 (body staircase step 3) — accepted as-built, integrator
     review 2026-07-19 -->
**Posture (added 2026-07-19).** A fourth controller verb joins the character
domain: `dc:character/set_posture` (`{ character, posture }`, posture one of
`standing` | `crouching`) — `character.control(character)` like the other
controller verbs, appended at the `Payload`/`RejectReason` ends (postcard order
is wire identity; `CharacterState` gains a `serde(default)` `posture` field so
old logs decode to `standing`). This is the **sim** half of bodies.md's crouch
firewall split: crouching scales the swept-AABB collider height (0.6×) at the
tick boundary; the cosmetic bend (spine lowered, knees via IK, head keeps its
look) is client-only and never read back. Standing up is **guarded** — a
`crouching → standing` change is refused with `RejectReason::PostureBlocked`
when the taller collider would embed in solid (the same `aabb_overlaps_solid`
test the attach embed guard uses), so a body under a low ceiling stays
crouched instead of clipping up. Replay bit-identity extends to posture
(`character_semantics::posture_transitions_replay_identically`).
<!-- END EDIT -->


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
- **Dev token vs `dc:*` — DECIDED 2026-07-19 (user).** In **development
  builds**, the dev MCP surface may `registry.define(dc:*)` — agents
  live-change vanilla, photograph, and port the verified change into the
  in-repo typed sources (which remain the sole durable truth; defs are
  session-state, boot regenerates vanilla). **Shipped builds carry no such
  grant** — build-config gated, not runtime-flagged: end users author
  plugins in their own namespaces over MCP (forking dc content into their
  namespace is fine and expected) but never redefine shipped `dc:*`
  defaults. The game is not open source; the dev surface is ours, not a
  shipped mod loader.
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

4. **Receipt verbosity** — DECIDED by S5 measurement: receipts carry a
   **summary plus a capped effect sample** (N≈8). Full effect echo was
   measured at ~21× the bytes on the demo session and ~5 MB for one legal
   64³ fill; bulk deltas belong to scan queries (cheaper per voxel) or event
   subscriptions.

5. **`client_player_pose_set` retires into the door** — DECIDED 2026-07-20
   (user ratified the session-4 analysis). It is a sim mutation living
   outside the surface: no capability check, no tick quantization, no
   receipt, no replay entry — while decision 2 says player input becomes
   commands, and the registry already holds its near-twin
   `dc:character/pose`. The seam is "the player isn't a dc-api character
   yet" (`player.rs` has no Character reference), not architectural
   conviction. Retirement = routing the player controller through
   controller-verb commands — the same work that makes player input
   replayable at all, so implied scope of decision 2 rather than new scope.
   `client_screenshot` and `client_player_pose_get` stay outside by design:
   disk I/O and camera-state reads have no replay meaning, and dc-api is
   headless. Sequenced, not immediate — it rewrites `mcp.rs`/`player.rs`,
   which two in-flight agents are touching.

6. **Registry `completions` hook** — DECIDED 2026-07-20 (user). Value-level
   completion (`block=<TAB>` → registered block names) has no source in the
   registry: parameter *names* and types come free from `payload_schema()`,
   but legal *values* are dynamic world/registry content. An additive
   per-`CommandSpec` completion source closes that so value completion is
   generated like everything else and stays free for future commands.
   Signature is implementation-designed under two constraints: wasm-safe
   (dc-api compiles for wasm32) and consumable by the in-client console.

7. **Registry self-consistency moves from tests to the compiler** — DECIDED
   2026-07-20 (user). Today "every command is registered" rests on
   discipline plus the schema.rs completeness tests (sample-list length ==
   registry length); payload schemas are hand-rolled `fn() -> Value`, not
   derived from the Rust types, so a forgotten entry is a runtime/test
   artifact. One declarative site (macro) that emits the `ids::` const, the
   `Payload` variant, and the `CommandSpec` per command makes a missing
   entry a compile error. Hard constraint: the wire-visible JSON schemas
   keep their current shape (inline, no `$ref` — consumers parse property
   descriptions for help/completion) and the public surface (`ids::`,
   `Payload`, `CommandSpec`, `registry()`) is unchanged, so no consumer
   edits ride along.

## v0 implementation notes (adopted from S5, details in docs/spikes/S5-results.md)

- Wire types never use `skip_serializing_if` — postcard is positional and
  silently corrupts on omitted fields (found the hard way; regression-tested).
- Consumers hold **grant handles**, not by-value tokens; the host owns token
  contents, so a WASM guest cannot fabricate or escalate grants (proven:
  contraband defines leave zero trace).
- `events/poll` is a channel operation, not a query.
- `registry.define` grants accept namespace patterns (multi-namespace owners).
- Envelope construction is registry-generated per consumer; MCP tool names map
  canonically from command ids (`dc:world/set_block` → `dc_world_set_block`).
- The `Payload` union keeps an open path (schema-registered, serde-tagged) so
  new commands don't ossify the enum across the ABI.
  <!-- EDITED 2026-07-19 (geology backbone slice 1) — the open path's first
       realization; accepted as-built, integrator review 2026-07-19 -->
  First realized (minimally) by `define_class_member`: the union stays a
  closed enum, but that payload's *contents* are open — parameter keys over a
  closed value vocabulary, legal shape defined by the registered class
  contract rather than a compile-time struct. New classes therefore need
  zero new Payload variants; wire encoding stays postcard-positional-safe
  (every field always serialized); validation runs at define time against
  registry-owned schema. New variants/fields appended at enum/struct ends
  only — postcard order is wire identity.
  <!-- END EDIT -->
- Plugin ABI: wasm32-unknown-unknown (no WASI, no ambient authority); one
  host import `dc.call` carrying postcard request/response; guest exports
  `dc_run`/`dc_tick`; receipts drained asynchronously.
