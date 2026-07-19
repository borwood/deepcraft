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

## Clothing/armor overlays — DECIDED 2026-07-19

Clothing is textures / transformed inflated copies of plan segments, bound
by segment name — one chestplate def adorns every mob of every plan that has
those segments, mutations included, because it is expressed relative to
segment dimensions. **v1 boundary: form-fitting shells only**; skirts,
capes, dangling gear need their own segments/sim — later.

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

## Open questions (user-owned)

1. Are bodies diegetic items (craftable golem shells, found vessels) or pure
   definitions instantiated at spawn?
2. Socket loadout realism: is `back` + two hands the survival cap
   (encumbrance doctrine), creative bypass?
3. Does NPC-tier degradation on disconnect live in the controller layer
   (fallback controller) or the body (autonomic reflexes)? Interacts with
   the walk-5 disconnect-policy question (API.md open).
