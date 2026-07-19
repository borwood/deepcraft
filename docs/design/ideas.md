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
