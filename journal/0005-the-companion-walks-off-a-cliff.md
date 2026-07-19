# 0005 — The companion walks off a cliff

*2026-07-19 · character MCP surface (background agent)*

> blogworthy: the first AI-embodied game character — driven over MCP through
> a capability-attenuated session, walking the same collision the player
> walks — spent its first minute of existence strolling off a ravine lip and
> falling eighty meters. Real body, real physics, real consequences: the
> design working exactly as ratified, in the most undignified way possible.

## What shipped

The second MCP surface from API.md § Characters: `127.0.0.1:7778`
(`--mcp-character-port <n>`, `--no-mcp-character`; `--no-mcp` still kills
both; on by default — a loopback embodied surface is the feature, not a dev
convenience). A session there begins unbound, calls `character_attach` once
to spawn-or-attach to ONE named character, and from then on holds a token
**attenuated to exactly that character** — its controller verbs and its
senses, nothing else. Underneath, all of it is dc-api:

- **The character primitive** (dc-api, additive): a persistent named body —
  feet position, yaw/pitch, player-dimension AABB — stored in the reference
  host, stepped **on the host tick** with dc-core's swept-AABB collision
  (walk, gravity, jump; the client player's constants are the dc-api
  defaults, so a companion is exactly player-shaped). Controller verbs
  `dc:character/set_move_intent` (direction + speed fraction, persists until
  countermanded), `set_look`, `jump` (one-shot, consumed at the next step,
  fires only from the ground) are ordinary commands: tick-quantized,
  receipted, totally ordered. Tick-stepping rather than per-frame was the
  deliberate choice: commands already land on tick boundaries, so inputs and
  integration share one clock and `seed + command log + tick count =
  bit-identical trajectory` — the determinism test proves final poses match
  to the bit, and the same JSON session through the tool layer at client
  scale does too.
- **Diegetic senses** (queries, grant-scoped): `character_pose` (+
  `on_ground`, `eye_in_solid` — the walker proprioception from 0004, now on
  every body), `character_sense_raycast` (from its eyes along its gaze or a
  given direction, ≤ 50 m, returns block/entry-face/distance),
  `character_sense_surroundings` (block cube, radius ≤ 16 voxels around its
  feet). No `world_*` on this surface, no registry, no screenshots — the
  session sees through the body only.
- **The grant**: `Grant::CharacterControl { character: Option<name> }` —
  `None` is the broad dev/parent form, `Some(name)` the session form, and
  the attenuation partial order refuses every widening (`Some(a)` can reach
  neither `Some(b)` nor `None`; tested at the capability layer, the host
  layer, and over the HTTP wire). Control implies senses: one grant covers
  both, on the theory that your senses ride your body. Spawning is NOT in
  the session's reach — `spawn_character` needs the dev-tier `entity.spawn`
  grant, and the attach flow performs it with the surface's parent token
  before attenuating downward. Session identity: envelopes carry
  `ConsumerKind::Player` (`character-<name>`) — API.md says a character acts
  in the human-input priority class, and now it does.
- **Tool lists stay generated.** The seven new commands register schemas;
  dc-mcp-dev and the 7777 surface grew the tools with zero per-tool code
  (the dev token gained `CharacterControl { None }`, so the dev surface can
  puppet and debug any character). The 7778 list is the registry *filtered*
  to `dc:character/` minus spawn, plus the one hand-authored
  `character_attach`. Same registry, two grins.
- **Render**: a two-cuboid figure (signal-orange torso, pale head,
  fullbright-aware, origin-relative, yaw-facing) synced from the hosted
  world each frame. Minimal, but unmistakable against terrain — see the
  assets.

## The walk (both surfaces at once)

Game launched windowed with `--fullbright`, dev session on 7777, character
sessions on 7778, driven over raw streamable-HTTP (curl; initialize →
tools/call, session id in the `mcp-session-id` header).

**scout** attached with the default position (two meters in front of the
player, one up), settled onto the ravine shelf, and was photographed from
above (`0005-companion-attached.png` — the orange figure standing on the
terraced chasm lip). Its senses immediately produced the walk's best
diegetic datum: `on_ground: true`, yet a straight-down `sense_raycast` from
its eyes reported **13 m of air** — it was standing centered on the lip
with its weight on half its footprint. Then I gave it `set_move_intent`
north and it did the honest thing: walked over the edge and fell ~80 m to
the chasm floor (pose telemetry caught it at −15 m, then −47 m, vel.y
−50 m/s, then `on_ground: true` at −76.8 m). A jump requested mid-fall was
correctly dropped; a grounded jump on the floor fired and landed.
`sense_surroundings` down there read mostly stone with dirt pockets;
its gaze raycast hit a dirt bank 1.53 m ahead. The chasm pocket proved too
tight to place an unburied camera — every vantage I probed reported
`eye_in_solid: true` — so the floor episode is documented by telemetry, not
film. The scout remains down there. It cannot climb out. This is canon.

**pathfinder** (second session — sessions are per-connection, each freshly
unbound) was attached at a guessed position and spawned **embedded in the
hillside**: `on_ground: true` at exactly its spawn height, `eye_in_solid:
true` — its own proprioception caught it, and two frames show it buried to
the chest in the slope. An embedded body is permanently stuck: no despawn
verb, no teleport on the character surface (correctly!), and swept collision
refuses to move an interpenetrating box. Filed to Observed: attach wants a
safe-spawn story, and "clamp to `surface_height_m`" is not it while that
helper under-reports (the same under-report buried the *player's* camera at
session start, again, caught by `eye_in_solid`, again).

**walker** got the staged finale: the dev surface built a 13×13 stone
platform in open air (`world_fill`, 169 blocks — the two surfaces working
as a duet: dev builds the stage, the embodied session performs), walker
attached onto it, and the camera caught it mid-stride
(`0005-companion-walking.png`, vel.x −3.6 m/s in the same-frame pose) and
at the platform edge after a jump (`0005-companion-jump.png`). Between the
two frames it crossed half the platform — and then, off camera, walked off
that edge too. Zero for three on cliff avoidance. The bodies are honest;
the minds are not attached yet.

## Findings → Observed

- **No auto step-up**: a one-voxel rise stops a grounded walker dead; the
  player and companions must jump. Whether step-up belongs in the mover or
  stays a controller skill (jump timing) is a design question for the
  NPC-intelligence work.
- **Attach placement is unguarded**: an embedded spawn is a permanent
  statue. Wants surface-clamped/validated spawn (blocked on the
  `surface_height_m` under-report already in Observed) and possibly a
  detach/despawn verb for cleanup.
- **Autonomous motion vs screenshots**: photographing a moving body over
  two async surfaces is luck without staging; fine for now, an event
  subscription (`character_moved`?) would let a camera follow.

## API.md § Characters — revisions the implementation suggests (not applied)

1. `entity.control(selector)` is implemented as
   `CharacterControl { character: Option<name> }` — exact-name or any, no
   selector patterns yet. The doc's selector language should either narrow
   to this or the grant should grow patterns when a use case appears.
2. The doc lists control and senses as separate grant ingredients; the
   implementation folds senses into control (one grant). If spectate-only
   sessions (senses without control) are ever wanted, the grant splits.
3. The session attach flow (spawn-or-attach, one character per session,
   parent-token spawn + attenuation) worked well and deserves a sentence in
   the doc; so does safe-spawn semantics once designed.
4. The open question "disconnect → NPC-tier or freeze" now has a concrete
   v0 answer by default: the body freezes (intents persist, so it actually
   *keeps walking* on its last intent — arguably worse than freezing; the
   scout only stopped because it hit a dirt bank). Worth deciding soon.

## Status

- [x] character primitive + grants + schemas (dc-api, additive; 7 commands)
- [x] host-tick simulation, dc-core collision, deterministic replay (tested
      typed, JSON tool-layer, and HTTP-wire)
- [x] capability cage proven at three layers; dev surface unrestricted
- [x] second server on 7778; visuals; walk with photographic confirmation
- [ ] safe attach placement, step-up question, disconnect policy (Observed)
