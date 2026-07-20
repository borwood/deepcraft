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

## Procedural languages (2026-07-19)

An **interlingua of concept-tokens**, honest about channel fidelity —
"broken english mixed with pictures," never uncanny fluency:

- Lexicon = pictographs for materials/items (their textures, for free) and
  actions (item animations rendered to animated glyphs) + generated proper
  names (people/places/events — history gen already makes them).
- A language is only `{lexicon mapping, word order (SOV/VSO/SVO), 2–3
  morphological toggles (particles vs affixes, reduplication, honorifics),
  name inventory}`. HARD SCOPE GUARD: no phonology, no morphology tables —
  never a full conlang. Styling varies by culture + speaker mood + styles
  propagated between historied actors.
- **Language is the knowledge system's wire format**: an utterance is a
  serialized fact-fragment (concept graph linearized by the speaker's
  grammar). Rumor noise, mythologization, whisper-chain distortion become
  mechanical (token substitution/drop/re-encoding). Old inscriptions in
  dead languages = archaeology of the ledger; expert consultations render
  their structured output (knowledge.md #6) through this layer.
- **Comprehension is a rendering setting**: character fluency (knowledge —
  learnable, inherited at creation) controls how much of an utterance
  renders as English vs glyphs/foreign names. Learning a language is
  watching text resolve. Player input: a composition bar over known
  concepts auto-cast into target grammar; LLM-assist may map free text onto
  the concept vocabulary. LLM companions emit concept-sequences
  (constrained decoding) — in-world by construction.
- Known costs: glyph-fatigue (mitigated by comprehension-rendering),
  accessibility (gloss/alt-text from day one), narrow tonal range
  (acceptable for an information-forward game).

## Structure & danger

- **Cave-ins / structural integrity** from structure density/porosity:
  mining engineering (props, pillars, reading strata), collapses producing
  conservative rubble (debris volumes, archaeology's raw material). Behind
  the realism-knob doctrine. S6 detachable props are half the machinery.

## Automation

- **An automaton is a body driven by a WASM-script controller** — the
  character/controller split makes player-programmable golems free, with
  plugin-sandbox security semantics. The "redstone answer."
- **Possession / animus override** (user, 2026-07-19): with bodies as
  diegetic items animated by an **animus** (the craftable carrier of a
  controller binding — bodies.md), overriding or supplanting another's
  animus is a natural mechanic: possession of golems, wresting a body from
  its driver, ghosts-in-machines. The controller-layer degradation ladder
  gives it mechanics for free (rebind = possess); costs/consent/defense are
  the design questions when this is sequenced.

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

## Namespace deltas (parked 2026-07-19 — user direction + Claude sharpening;
post-playable-demo by priority razor)

- The gap: fork-and-replace can't alter a `dc:*` def in place for downstream
  consumers (references keep naming the original), and shipped builds never
  get `define(dc:*)` (API.md § Capabilities). Render packs already cover
  textures; class-joining covers additive content; *modifying vanilla data*
  (distribution logic, params) has no shipped path.
- Mechanism sketch: **identity-preserving patches** — a plugin ships
  field-level diffs against foreign defs; registry state = base ⊕ ordered
  patches; downstream references stay `dc:...` and see the patched def.
  Provenance carried as a queryable chain on the def ("granite, as patched
  by my-mod"), NOT encoded in the id (that would re-break references).
- **Union-of-patches merge (user, from MC-mod-community experience): two
  patches touching disjoint fields of one def BOTH apply (distribution +
  hardness compose); pack order trumps only on per-param conflicts**, loudly.
  Field-level merge, never pack-level shadowing.
- Reversibility free by construction (defs are session-state rebuilt from
  base + patch packs at boot; removing the pack is the revert). Patches
  validate against class contracts; invalid patch = named warning + skip
  (pack-degradation doctrine). Patch set joins pack set in world identity.
- Grant shape: `registry.patch(dc:*)` as a SHIPPED capability, distinct
  from dev-only `define(dc:*)` — users mod vanilla attributably and
  reversibly; shipped defaults stay inviolate underneath.

## Uncollapsed history frontier (user sketch, 2026-07-19 — an architecture
challenge, explicitly maybe-impossible)

- Floated against the deep-time width cap: instead of one global coarse
  A-tier, generate **full history for a large region and gradate past its
  border into uncollapsed minimal information** — just enough constraint
  mass to justify what's collapsed inside — then *stream history* at the
  borders as the world grows. Deep-time history as first-class
  superposition, collapsed regionally on demand.
- Kinship: this is the S2 constraint-ledger "collapse under observation"
  concept applied to the pregen/deep-time tier itself (today S2 governs
  live sim state; pregen history collapses globally at world creation).
- Known hard part: deep-time processes are global (drainage/fluvial reach
  — corrections #8), so a border can't be "minimal information" for any
  process whose influence crosses it; the frontier would need per-process
  treatment (bounded processes stream freely; global ones need committed
  coarse skeletons — which is exactly what the A-tier cap already is).
- **User counter (2026-07-19, second pass): even global flow can be
  frontier-hacked.** A cross-border flux is a *committed fact that
  constrains the uncollapsed exterior*: "we don't know where all this
  water came from at this edge, but it does imply there's higher terrain
  that way which hasn't been collapsed yet." Seeded rolls at the edge
  commit inflow magnitudes; the exterior's later collapse must honor them
  (enough catchment mass, enough elevation, that way). This is exactly
  S2 ledger semantics — facts constraining distributions — applied to
  terrain itself. Still a challenge (conservation across a growing
  frontier; retro-consistency of many committed fluxes), not scheduled;
  the width cap + C refinement is the working answer.

## Posture ladder (user sketch, 2026-07-19 — crouch DECIDED separately in
bodies.md; everything below is unscheduled)

- Standing → crouch (0.6×, DECIDED: posture + sneak edge-walk-block; never
  passage) → **crawl** (all fours, faster, transitional) → **belly-prone**
  (slowest, strategic — the deliberate stance). Prone is what solves
  1-voxel-height passage.
- The interesting mechanism: **crawl as a geometry-triggered transition,
  not a keybind** — crouch-walking into a 1-height gap drops you to all
  fours automatically; posture as a consequence of intent + geometry.
  Trigger only on deliberate entry (walking into the gap face), never
  spontaneously, so stealth players near ledges don't get surprise posture
  changes. Belly-prone stays an explicit verb.
- Firewall-clean: triggers are geometric and deterministic, sim sees only
  the parametric posture ladder; all animation cosmetic per bodies.md.

## Misc noted

- Seasons/calendar as the missing middle timescale (weather ↔ deep time).
- Books/songs/legends UI as serialized ledger facts; NPC culture content
  generated from actual history.
- Sim-depth knobs as a general doctrine (extent, encumbrance, integrity…):
  one simulation, player-tunable depth.

## The bio slot on a substrate (user, 2026-07-20)

Raised while diagnosing the razor-straight grass/dirt frontier (ROADMAP
Observed): today "grass" is a whole *block type* chosen by a threshold —
ground cover is a paint decision, and there is no biota in the world at
all. The user's direction reframes it:

- **A voxel is substrate + a biotic occupancy** — "part of this block is
  vegetative, the rest is substrate or roots." Cover stops being a block
  identity (Grass vs Dirt) and becomes a *fraction on top of* the material
  that is actually there. A grass voxel is dirt with a living fraction;
  the boundary between grassland and bare ground becomes a gradient in
  that fraction, which dissolves the hard-threshold problem at its root
  rather than dithering over it.
- **Convenience the user flagged**: this wants to reuse the S8 loose-
  material partial machinery (per-voxel fractional contents) rather than
  invent a parallel system. **Known mismatch, explicitly noted by the
  user: the desired *presentation* does not align with current partials
  mixture logic** — mixtures today read as intermixed constituents
  (speckle/heightlerp), whereas vegetation wants to read as a *layer/
  canopy on top of* its substrate, with its own silhouette. Open: whether
  that is a rendering rule over the same data, a distinct occupancy
  channel, or partial-height geometry (the dormant sub-8 loose rendering).
- Couples to: S10's community vector (what species the fraction *is*),
  ecology.md's biology-as-a-rock-forming-term, and the water thread
  (moisture is the field cover actually responds to).

## Soil is loose, but packable into structural (user, 2026-07-20)

- **Soil should be a LOOSE material, not structural** — it is the S8
  loose/partial tier, not a solid block, contra today's `Block::Dirt`.
- **…except that it can transition to structural under weight + time.**
  Soil is a *packable* material: overburden pressure and duration convert
  loose soil into a packed, structural form. Worldgen consequence the
  user named: **most sub-surface soil levels generate already packed** —
  only the top horizons are genuinely loose, which is also what makes
  digging feel right (loose topsoil, firm subsoil).
- Mechanism candidates (undesigned): a packing/consolidation field
  derived from depth + time-under-load, plausibly the same machinery as
  the deep-time strata recorder; player-side, tamping/walking/building as
  the compaction verb (crafting-as-process § "pack" verb already exists).
  The reverse (structural → loose on disturbance) is the natural pair.
- Connects to the water thread: porosity/permeability differ sharply
  between loose and packed soil, so this is upstream of groundwater and
  of S8 porosity-driven wetness rendering.

## Entering a world without generating one (user, re-raised 2026-07-20)

Raised again while ratifying S10's 25 s ritual — the point being that
generation cost should be answered by *offering a way around it*, not by
capping simulation depth:

- **Optional ready-made worlds**, shipped pre-generated, where the player
  picks a locale and starts immediately — **no generation time at all** —
  with generating your own always available as the alternative.
- The entry modes discussed before, now attached to this: **join a
  settlement**, **assume a character from present history**, or **start as a
  hermit in a random wilderness**. These are not just spawn points; a
  pre-generated world has a *history*, so entry means entering an ongoing
  story at a chosen position in it.
- Why it fits the architecture: worlds are seed + committed ledger facts, so
  a shipped world is a distributable artifact rather than a special case, and
  the deep-time record (now including biology's coal, paleosols, charcoal)
  is exactly the content that makes a canned world worth inhabiting.
- Open: what a shipped world weighs, whether shipped worlds pin a
  seed-versioning contract (the filed "seed-stable worlds across releases"
  question becomes load-bearing here), and how player edits diverge from a
  canonical shipped world.

## Content packs and the default world's scope (user, 2026-07-20)

Stated while listing the water features the world should have ("literally
whatever you encounter on earth"). Recorded with the user's own hedge
intact — "probably" is theirs, so this is a strong direction, not a
closed decision.

- **The default pack is approximately EARTH**, and its history **ends
  before the age of mechanised industry** — the user puts the **tech
  cutoff around medieval**. So the default world is a real-Earth-like
  world simulated up to a pre-industrial horizon.
- **The fantasy pack is probably a separate addon** — and the user is
  explicit about what it is for: *"that's where we get our actual dwarf
  fortress, and it's what i actually want to play."* It would carry
  **fantasy materials and items** of its own.
- Architecturally this is already the shape of the thing: ARCHITECTURE.md
  holds that "vanilla content is just the first content pack going through
  the same door", and content packs are registry command batches
  (journal/0007). So *Earth-default + fantasy-addon* is a content
  packaging decision, not an engine one — the same door serves both.
- Consequences worth noticing early: the **completeness bar for the
  default pack is Earth itself** (every water feature, every rock, every
  climate), which is a much stronger content target than "enough for a
  game"; and the **fantasy pack's blast radius** is the same open question
  ecology.md § 5 fork 2 asks about organism packs — a pack that adds
  materials or organisms can change terrain and history, not just
  contents.
- **Tech cutoff clarified (user, same day)**: "at least for now i'm not
  interested in thinking about items past medieval. it's not out of the
  question but it dramatically changes game if we get electricity." So the
  cutoff is a **design-attention boundary**, not a law of the world — the
  reason is that industrial/electrical technology would *dramatically
  change the game*, not that the simulation forbids it. Treat medieval as
  the ceiling for item/tech thinking until the user reopens it.

### The default pack: pure Earth, or a bespoke setting? (REOPENED by the user, 2026-07-20)

The user reconsidered the "default pack ≈ Earth" framing within the hour,
and this is **user-owned territory** (setting, aesthetic, scope) — recorded,
not decided:

> i don't know whether default, on second thought, should be pure earth
> as-is or should be my own bespoke setting, which gives a very deep and
> honest nod to nature and the natural processes of the earth and of socia
> - but also evolves its own fitting aesthetic, balance, shape, charm.

- The bespoke option keeps the **honest nod to real process** — the
  natural processes of the earth *and of society* ("socia": the history
  sim is a natural-process system too, not a backdrop) — while **evolving
  its own aesthetic, balance, shape, charm**.
- **Already anticipated for it** (user, from a pre-repo conversation that
  was never written down — captured here on sight): **golems** ("script or
  mcp 'animus' driven made/found bodies"), **deep lore**, **ancient
  evils**.
- Note for whoever builds golems: the architecture is largely *already
  shipped*. Bodies are registry data (bodies.md), characters are bodies
  with controller bindings, and the character MCP surface (journal/0005)
  already lets an external agent drive a body. A golem is a made-or-found
  body whose controller is a script or an MCP "animus" — that is the
  existing seam, not a new one.

### The aesthetic thesis (user, 2026-07-20) — worth treating as a north star

> minecraft is ahistorical. our vibe inherits from that - elevated pixel,
> timeless - while being anything but timeless

- The visual language **inherits Minecraft's timelessness** — "elevated
  pixel", a look that doesn't date itself — while the **world is nothing
  but history**: deep time, strata, paleosols, fire records, ruins,
  lineage.
- The tension is the signature, not a problem to resolve: **it looks like a
  toy and reads like a core sample.** A world that presents as timeless and
  rewards excavation with 500 Myr of consequence.
- Cross-reference visuals.md (§ distance speaks the voxel language is the
  same instinct applied to LOD: the world stays in its own visual dialect
  at every range).

## Rock is not monolithic — defects, jitter, and loose materials (user, 2026-07-20)

Raised on seeing house-sized volumes of unbroken pure coal in walk
screenshots, then generalized by the user beyond coal.

- **"Most of these materials are not unbroken in the earth, there are
  defects."** The monolithic-purity problem is GENERAL, not a coal bug.
  Every thick uniform unit in a cross-section has the same defect: the
  record knows a unit, the voxel grid renders it as a flawless mass.
- **Use partials for defects.** We already have per-voxel fractional
  contents (S8 eighths). The user's direction: **jitter at the partials
  scale, dependent on a material property**, softening boundaries where
  it makes sense — **particularly for soft materials**. So a soft unit's
  interior and margins carry fractional impurity/void rather than being
  uniform fill, and hard materials stay crisper.
  - Precedent to reuse, not reinvent: the 3d **member-contact boundary
    dither** already wanders material contacts at the material tier
    (journal/0011). This extends the same instinct inward — from the
    boundary between units to the *interior* of a unit.
  - Distinguish from the coal-partings case below: partings are
    *structural* (bedded mineral bands from events); jitter is
    *textural* (the material is never perfectly pure or perfectly
    bounded).
- **Loose materials are needed in game — and are not there yet.** The
  user: needed "even if they don't fall with gravity yet." Note the
  renderer is already waiting: partial-height loose rendering shipped
  **built-but-dormant** in 3c-2 (journal/0010) because loose deposition
  never emits sub-8 columns. The missing half is content/simulation, not
  rendering.
- **Loose materials should SPREAD when dropped** (user, same pass):
  depending on material properties — **granularity + (something) +
  fall height** — a dropped loose material displaces **partials into
  surrounding empty space** rather than landing as a neat column. This is
  angle-of-repose behaviour obtained from the partials model instead of
  from a physics solver, and it composes with the existing eighths
  representation. Undesigned; the middle property in the user's
  "granularity + x + fall-height" is deliberately left open.

## Coal partings, and the general case of sub-epoch structure (2026-07-20)

- Real thick coal seams carry **partings** — clay/shale/ash bands from
  floods and eruptions interrupting the swamp. Coal geologists classify
  seams by parting structure; ours have none.
- **Why ours are pure** (diagnosed): (1) deep-time epochs are ~2.5 Myr
  (200 iterations over the ~500 Myr register), and real coal-forming
  cycles are Milankovitch-scale (10⁴–10⁵ yr) — **every parting-forming
  event is sub-epoch and invisible**; (2) the recorder deliberately
  **merges consecutive organic epochs into one horizon**, which is the
  optimization that took the record from 665 k units to 71 k (S10) and is
  also precisely what erases internal structure.
- **Where the fix belongs**: earth-processes **method rule 5** — the
  unsimulated remainder gets procedural tricks, and no grid/analytic
  boundary may reach the eye. Partings should be a **collapse-tier
  procedural detail** (deterministic, position-seeded mineral bands at
  realistic spacing inside thick organic units), never simulated at deep
  time. Pairs with the **charcoal-as-inclusion** item the organics agent
  filed-but-did-not-build: both are "the record knows something the voxel
  grid is too coarse to show."
- Size itself is NOT the defect: real seams reach 30 m (Powder River) to
  100 m+ (Latrobe Valley), and are laterally extensive sheets. Our shape
  is right (bedded sheets, deposited per-cell per-epoch, not blobs); the
  interior is what is wrong.
