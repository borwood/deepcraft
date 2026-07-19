# Bodies, body plans, sockets, animation

Status: core decisions RATIFIED 2026-07-19 (marked DECIDED below); sections
marked PROPOSED are sketches, NOT decisions. Expands the ratified seeds in
materials.md § forms (body-as-plugin, sockets, transmog-as-body-swap) and
API.md § Characters (character/controller split). Sequenced item 3b.

## The three-layer split

```
character  — identity + persistence (who): account-owned, survives everything
body       — physical presence (what):    an instance of a registry body plan
controller — volition (who's driving):    human / MCP session / WASM / NPC-tier / nobody
```

The character surface (journal/0005) shipped layers 1 and 3 with a
hard-coded body. This doc is layer 2.

## Body plans — DECIDED 2026-07-19

Fidelity target: **above Minecraft, still cuboid** — jointed limbs, several
body segments, a neck. A **body plan** is a registry contract: a named joint
topology (segment tree). Concrete bodies are *parameterizations* of a plan —
segment scales, proportions, textures, stat-driven modifiers — so N humanoid
mobs are mutations of one humanoid plan, a quadruped plan covers most
animals, and so on. The fourth instance of the roles-as-contracts backbone
(geology classes, form archetypes, geology passes, now body plans).

- **Animations bind to the plan, not the mob** (processes bind to classes,
  not instances). Every mutation of a plan shares its animation set;
  proportion differences are absorbed procedurally (IK, below).
- **Forking**: a child plan that retains a parent's joint names inherits the
  parent's clips for those chains; overrides are allowed per plan and per
  body instance.
- **Verb → animation-slot contract**: a plan declares which driver verbs it
  supports (move forward/backward, strafe, jump, idle, prone, crouch, crawl,
  overhead swing, thrust, …) plus blendable states (afraid, agro, happy, …).
  Registration **fails at define time if a required anim slot is unfilled**
  — the same schema-checked-at-define-time rule as everything else.
- **Plans and clips are data, defined via editor/MCP** through the registry
  (keyframes are just payloads). A posing/keyframing GUI is a later tooling
  milestone; MCP-authored clips work from day one — including Claude
  authoring animation through the dev surface and iterating from
  screenshots.

## The determinism firewall — DECIDED 2026-07-19

**Animation is cosmetic. The sim sees only parametric posture states.**

- Sim side: discrete/parametric posture (standing / crouch / prone collider
  heights on the swept-AABB mover), action reach + timing as command data.
  Bit-identical replay untouched; headless crates stay render-free.
- Render side: the skeleton — clip playback, blending, IK, emotional
  additive layers — derived on the client, never read back into simulation.
- Example (crouch): "raycast up, clamp collider height" is sim;
  "IK the spine and head under the ceiling" is renderer.

## IK — DECIDED 2026-07-19 (role, not solver choice)

Two-bone IK for limbs + neck look-at is the **retargeting glue** that makes
one clip serve every mutation of a plan: feet to actual ground, hands to
actual socket transforms, across differing proportions. Solver technique is
an implementation choice; artifact suppression uses standard techniques
(pole vectors, joint limits) — cuboid rigs do not inherently pop.

## Stepped animation — DECIDED 2026-07-19 (aesthetic choice)

Character animation renders **frame-stepped (~12 fps, quantized rotations)**
— a stop-motion look chosen for the elevated-pixel aesthetic (an identity,
not a workaround; it also happens to be cheap and forgiving).

## Clothing/armor — DECIDED 2026-07-19 (refined same day)

Clothing is **authored as its own set of cuboids**, each parented to a body-
plan segment either **fully** (rigid follow — armor plates, shirts) or **at
a point** (pinned with constrained rotation — a floating cape pins to the
trunk and swings within limits; a soft-body deepening of pinned pieces is a
later option). The earlier textures / transformed-segment-copies method
survives as the **default/fallback authoring path** (a copy is the
degenerate case: one cuboid, fully parented, inheriting segment transform).

Binding is by **segment name**, and authored cuboids are expressed
**relative to segment dimensions** (mechanism note: this is what keeps "one
chestplate def adorns every mob of every plan with those segments,
mutations included" true — absolute-meter authoring would break on scaled
mutations). Pinned/swinging pieces are cosmetic-side per the determinism
firewall. v1 boundary softens accordingly: pinned-rigid pieces (capes) are
in; soft-body is later.

## Plan parameters — PROPOSED (user sketch 2026-07-19)

Body plans may declare **scalar and bool parameters** that bodies bind to
character state:

- **Scalar**: a segment expands/contracts within a plan-declared range,
  driven by a stat (weight, strength) — visible bulk from simulation state.
- **Bool**: a segment is present/absent by character property (e.g.
  sex-dimorphic segments) — a bool param is effectively a *fork you can
  flip per instance*, unifying with plan forking.

Interaction notes (Claude, for discussion — not decided):

- Absent segments drop their animation channels — the same rule as fork
  inheritance — and sockets on absent segments vanish (missing-socket ⇒
  item-to-inventory rule from transmog).
- Each param must declare **which side of the firewall it touches**: purely
  cosmetic scale (visual bulk) is free; a param that alters collider
  extents or reach is sim-visible and must be quantized + deterministic
  (stats are sim state, so this is legal — but stepwise, never smooth
  per-frame).

## Sockets — PROPOSED

A socket is a named, typed mount on a plan segment: `hand.r`, `back`, `head`.

- Acceptance is a predicate on **form archetypes** (materials.md § forms),
  not an item whitelist: `hand.*` accepts `graspable`, `back` accepts
  `strappable`. Sockets × archetypes yield the equipment matrix for free.
- Socketed items are physically present (a torch in-hand is the world's
  torch; drop = detach at the socket transform). Interaction verbs anchor to
  the active socket.

## Transmog / body swap — intent DECIDED (materials.md), mechanics PROPOSED

`entity.set_body(character, body)` as a normal command: controller binding
untouched, sockets re-mapped by name (missing socket ⇒ item to inventory),
mover posture re-validated against new extents (the attach embed guard
applies — growing into solid is the embedded-attach case).

## Implementation staircase — DECIDED 2026-07-19 (order)

1. Joint-tree skeleton + body plan as registry def; companion re-expressed
   as a biped; hand-authored idle/walk as data clips (MCP-defined).
2. Locomotion set + crossfade blending + the verb→slot define-time contract.
3. Two-bone IK: foot placement, neck look-at; parametric crouch (sim state +
   cosmetic pose).
4. Plan forking/inheritance + scale/stat mutations (IK retargeting proves
   out here).
5. Clothing shells.
6. Authoring editor (GUI); MCP data authoring exists from step 1.

Not scheduled against geology; slots in when 3b's implementation is
sequenced.

## Formerly open questions — DECIDED 2026-07-19

1. **Bodies are diegetic items**: found vessels, and eventually craftable
   from materials plus an **animus** — the item that carries a controller
   binding (an MCP session, a WASM script, …). The automation sketch's
   golem is exactly body + script-animus; the animus makes "who's driving"
   a diegetic, craftable, presumably stealable thing.
2. **Survival loadout cap: two hands + back** (encumbrance doctrine;
   creative bypasses). Containers extend it diegetically: a backpack in the
   `back` socket extends inventory; belts/pockets and similar arrive as
   clothing that *adds container capacity* — clothing defs can contribute
   carry slots, not just cuboids.
3. **NPC-tier degradation lives in the controller layer** (a fallback
   controller binding), never in the body: the body stays pure physics +
   anim slots, so transmog never carries behavior and any controller can
   drive any plan. **Disconnect v0 (decided with it): freeze** — a
   session's disconnect zeroes move intent and the body stands where it was
   left (implementation queued in 3c). The degradation ladder (MCP session
   → local NPC controller → null/freeze) is just re-binding down the same
   rail; richer NPC-tier controllers arrive with the NPC-intelligence
   design.
