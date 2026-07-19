# Idea inventory — sketches, NOT decisions

Captured 2026-07-18. Everything here is a direction the architecture makes
cheap, roundly agreed in discussion, and **deliberately not yet designed** —
details and sequence get worked out when each item's turn comes. Do not
treat any shape below as final.

## Knowledge is physical (the ledger as gameplay)

- **Divination = the observe/inspect split, diegetically.** Scrying: pay a
  cost, force bounded collapse at distance (commits facts). Augury: read the
  *distribution* without committing — glimpsing possibility. Mechanically
  honest prophecy; a whole magic school from existing machinery.
- **Fame is fact-propagation.** Reputation = which of your deeds have
  propagated where, with distortion (claims, rumors, songs) layered above
  ledger truth. The player is structurally a historical figure; strangers
  can know wrong versions of your story.
- **Maps as ledger views**: cartography/surveying gameplay; trading maps =
  trading committed facts; rival maps may contain lies (claims).
- NPC blackboards as personal fact-sets; gossip transfers facts; settlements
  share coarse-tier collective knowledge.

## Sound (high importance to the project)

- **Hearing is an observation channel**: a distant rockfall or battle heard
  through rock commits (coarse) facts, like sight does. Hearing range/
  acuity on the character sense surface.
- **Material-driven acoustics**: per-voxel density/porosity are the physical
  inputs for occlusion, muffling, reverb — cave acoustics from actual
  geometry, sound deadened by packed-earth walls. Darkness + hearing as the
  dungeon mood pairing.

## Crafting as process (sequenced: after character MCP)

- Verbs are typed work (heat, quench, grind, mix, pack, form, cut) —
  registry entries over material classes, reading/writing property-sheet
  fields (firing clay changes its porosity — the same field sim/render use).
- Items carry **process history** (the crafting analog of stratification);
  quality = fidelity of process to the material's window, not a roll;
  pattern-welding shows as literal banding.
- Stations grant capability windows (forge = temperature range), mapping
  onto capability tokens.
- **Recipes are discovered process windows — knowledge**, learnable/
  tradeable/lose-able; cultures hold different traditions in the history
  sim; techniques can die and be re-excavated.
- Failure is content: outcomes are drop distributions (fracture machinery
  reused); slag heaps are debris deposits.

### Accessibility synthesis (agreed 2026-07-19, still sketch-tier)

**The sim gates excellence, not function.** One simulation, four interfaces:

1. **Quick-craft** — a recipe is a *saved process plan* (knowledge object)
   executed in one click; the sim runs underneath. Your culture's plan-book
   is your starter kit — knowledge inheritance IS the accessibility
   mechanism. This is the Minecraft mode, and it is the default.
2. **Work the craft** — opt-in *per act*: open the process view, chase the
   window, read diegetic tells (metal color, strike sound — not gauges).
   Masterwork, pattern-welding, signatures live here only.
3. **Commission** — the simulated economy crafts for you; depth reaches
   adventure-mood players as world texture and trade, not homework.
4. **Automate** (later) — plans executed by apprentices/golem controllers.

Supporting rules: **window width scales with tier** (campfire/knapping are
unmissable by physics — one gesture, no UI; steel narrows); **low-tier
failure costs quality, never materials**. Litmus for every crafting
decision: can the indifferent player get a usable item in one interaction,
AND does the invested player have something real to master in the same
interaction? Both yes or redesign.

## The historied player (2026-07-19)

Character creation is a **ledger operation** — choices/knobs, with
multiplayer server policies over which modes are open (only hermits, only
historical figures, …):

- **Become a historical figure**: RPG-style rolls offer characters from the
  generated record; you inherit their committed facts — home, debts,
  enemies, reputation, knowledge. The record continues through you.
- **Join a historical community**: a fresh subject seeded with the
  community's fact-set — its plan-books, traditions, local knowledge. Your
  starting knowledge is your inheritance (the crafting plan-book is one
  instance of this general rule).
- **Hermit/outsider**: empty ledger, no inheritance, hardest and freest.

The character then *continues to accrue history* — deeds commit facts, fame
propagates (see Knowledge is physical) — which enables:

- **Dreams**: sleep runs a sampling pass over the character's personal
  fact-set — abstract associative imagery across history/experience layers
  (room for genuinely strange renderings under the elevated-pixel
  aesthetic). A **sensitivity stat** opens the inspect-tier channel during
  sleep: augury glimpses of distant superposed state, symbol-wrapped and
  deniable — prophecy that is true but not legible until confirmed.
- **Thoughts**: the player character has a blackboard (it is a body with a
  controller, like any NPC); blackboard-relevant events surface as visible/
  audible remarks or thoughts. Doubles as **diegetic tutorialization**:
  "this gravel looks water-sorted…" is simulated knowledge surfacing as
  cognition, not an authored tooltip — the character can know things the
  player doesn't yet.

## Structure & danger

- **Cave-ins / structural integrity** from structure density/porosity:
  mining engineering (props, pillars, reading strata), collapses producing
  conservative rubble (debris volumes, archaeology's raw material). Behind
  the realism-knob doctrine. S6 detachable props are half the machinery.

## Automation

- **An automaton is a body driven by a WASM-script controller** — the
  character/controller split makes player-programmable golems free, with
  plugin-sandbox security semantics. The "redstone answer."

## NPC intelligence (discussion 2026-07-18; needs its own design doc after
character MCP / bodies)

- Pipeline-as-controller: perception → blackboard → behavior producers
  (scripted nav A* / steering / small-NN locomotion / NN combat micro) →
  arbiter → move/look/jump signals. Every stage a registry plugin.
- Principle: **NNs at the bottom (feel), symbols at the top (inspectable
  intent)**. Inference is µs-cheap; the real scale costs are perception
  (event-driven sensing over the event bus, sensor LOD by tier) and nav.
- **Hierarchical nav mirroring the world pyramid**: voxel A*/flow fields in
  the near bubble (nav tiles as a third subscriber of the edit dirty-events,
  after mesh + physics); portal/waypoint graphs at locale/region scale
  (roads from the history sim); and far travelers as **fluid state** —
  journeys are distributions over routes; meeting one is bounded collapse
  conditioned on route + time (the proven S2 traveler machinery). First
  prototype candidate: portal graph + traveler collapse.

## Misc noted

- Seasons/calendar as the missing middle timescale (weather ↔ deep time).
- Books/songs/legends UI as serialized ledger facts; NPC culture content
  generated from actual history.
- Sim-depth knobs as a general doctrine (extent, encumbrance, integrity…):
  one simulation, player-tunable depth.
