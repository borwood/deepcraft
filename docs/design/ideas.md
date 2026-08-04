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

> **✅ STATUS: THIS ONE IS DECIDED, not a sketch — marked 2026-07-29 (baseline sweep S4/F9).**
> *"Ready-made worlds are the sanctioned answer"* is stated as project doctrine in `CLAUDE.md`
> § Conventions (beside *"gen time is not a constraint"*), and `stubs.md` #21 uses it as a
> load-bearing premise for a sequenced heir (*"means a ledger **will** be persisted"*). This
> file's charter — *"sketches, NOT decisions"* — was still labelling it *"not final"*. The
> convention for this exists here already (cf. *ratified direction 2026-07-23* and
> *RECONCILED → material-behavior.md §5* elsewhere in this file) and was simply not applied.
> **The sketch text is kept as the origin record; only its status is marked.**

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

## Extensibility: the passes ARE the API being built (user, 2026-07-20)

Asked how passes are expressed as plugins and how a plugin would add new
passes to generation — a compartmentalization/moddability worry. The
user's own framing resolved the sequencing question:

> makes perfect sense that our passes right now are also *building that
> api* - or what will become it - alongside the passes. we're early in
> dev. plugins won't build the api, they're built on the api. eventually
> we stabilize our api, expose it to plugins, start versioning /
> developing backwards compatibly.

So: **do not design a plugin-pass mechanism yet.** Keep writing passes;
the API is the residue they leave behind. Stabilize → expose → version →
maintain backwards compatibility, in that order, later.

### What modders will actually want (the user's design target)

> this game gives modder imaginations a surface to dream about deep time,
> so if they picture a fantastic present that depends upon deeptime, or
> they imagine a deeptime scenario ... **some** affordance should be given
> to satisfy those emergent fantasies.

**The worked example, and a genuinely good API probe:**

> ancient aliens had impenetrable outposts built on the planet in far
> geological prehistory -> what api do i have as a modder to inject this
> in a way the sim understands.

Assistant analysis (PROPOSAL): decompose what that fantasy actually needs,
and most of it is machinery we already have or are about to build.

- **Emplace a thing at a place at an epoch** — deep time has no way to
  accept *exogenous* emplacement today. This is the genuinely missing
  primitive.
- **Declare it effectively unerodible** — this is *precisely* the
  erodibility-coupling milestone now sequenced first (per-cell erodibility
  read from the exposed material). An impenetrable outpost is a cell whose
  material has extreme hardness. **The dismal-mountains fix is also the
  first plank of the modder API.**
- **Then it rides existing machinery unchanged**: erosion refuses to cut
  it, sediment buries it, uplift and exhumation may expose it again on a
  scarp, and the strata recorder logs what accumulated over it. A modder
  gets "you dig down 200 m and hit an alien wall, and the *layers above it*
  tell you how long it has been there" without a single bespoke system.
- **What the record cannot yet say**: that a unit is an *artifact* rather
  than a rock. The recorder's vocabulary is depositional
  (env/energy/aridity/biofacies); provenance-as-artifact is a new axis.

The lesson to carry: the test of the deep-time API is not "can a mod add a
block" but **"can a mod inject a fact into prehistory and have the whole
downstream simulation take it seriously."** That is the acceptance
criterion when the API is eventually stabilized.

## The perf/debug overlay is player-facing (ratified direction 2026-07-23)

The runtime-observability slice (ROADMAP § perf window) builds `tracing` spans
on the hot paths and an aggregating layer that dumps a ranked self-time table to
`docs/audits/`. That file dump is the **first** consumer, for our own use — but
the ratified final shape is an **in-game perf/debug overlay with a profiling
toggle** (user: *"final shape for game includes profiling with a switch in-game
on a perf/debug overlay; players like that kind of thing"*). Same spirit as the
ranges-as-player-config doctrine (S-1 extended to the render/debug tier): a dev
surface players get to see.

The architecture consequence, load-bearing for the slice: the span aggregation
must be a **queryable in-memory resource** (the authority), with the
`docs/audits/` text dump as one consumer and the overlay widget as a second —
never a file-only dumper that the overlay would have to re-instrument. "A summary
derived from the authority, never beside it" (S-3), applied to timing data. The
overlay is the named heir; do not build it now, but do not foreclose it.

## Pass cadence — the fractional-phase scheduler (user sketch, 2026-07-23)

> **✅ THE RATE HALF OF THIS SKETCH IS BUILT — 2026-07-29, journal/0123.** Both directions of it:
> the coarse one (`period`, epochs between firings) and the **fine** one (`sub_turns`, several
> turns per firing, each handed the state the previous one left) — the *"tectonics ×1 →
> hydro ×1 → weathering ×3"* half that had never been expressible. `dt = period / sub_turns` is
> the *"duration is a scalar on its transformations"* of the third bullet below, and it is now
> consumed rather than assigned and ignored. The cadence numbers are **data** a world authors
> (`deeptime::CadenceTable`), not constants in the pass bodies.
>
> **Nothing in this sketch was reconciled away by that slice, and two of its questions are still
> open and still the user's**: the ORDER half (restored 2026-07-26, still unbuilt), the
> **epochs** half below (*"declared with pass members + a chapter count OR a terminating
> condition"* — never built), and the *sharp open fork* at the end (**one weathering pass at one
> cadence, or one pass per agent at its own?**) — which the axis now makes answerable in code
> for the first time, and which remains a design question nobody has answered.

**RECONCILED 2026-07-24 → `material-behavior.md` §5 (Cadence: order × rate × window).** The
scheduler is ~~**two orthogonal axes**~~ **three** (a third was ratified 2026-07-25, below):
~~topo-sorted **ORDER** (derived from a pass's reads/writes)~~ **ORDER — AUTHORED,
PER WORLD** × fractional-phase **RATE** (`dt` = phase length) × the aggregation **WINDOW**
(how many epochs sum into one record entry — `flow.md` § 11.1, RATIFIED 2026-07-25; RATE is a
*sampling* rate, WINDOW is the record's *time granularity*). This sketch is the
**RATE** axis — and the
sub-chapter multi-rate (weathering ×5 while tectonics ×1) is preserved as the whole
point. `dt` = phase length is the `rate × dt` S16 already assumed. Sketch retained
below as the origin.

> **⚠ THE ORDER HALF OF THIS SKETCH WAS RESTORED 2026-07-26** (user; `ARCHITECTURE.md`
> § *The engine is plugin-agnostic, and pass ORDER is authored*; **corrections #65**).
> The 2026-07-24 reconciliation replaced this sketch's *"canonical start order"* with a
> topo-sort — **and that was a reconciliation, never a contested decision.** It was
> falsified by `journal/0090`'s own summary: a linear relaxation pipeline *"does **not**
> fall out of a dataflow graph for free — you have to name each revision as a distinct
> resource."* **The order never fell out of the declarations; it was fed in**, as the seven
> `DeepAxis` revision tokens. `{reads, writes}` are now the **validator**, not the
> generator. **RATE was the half that survived reconciliation** — ratified, still unbuilt,
> and now the lead sequenced item with the erosion blocker as its first consumer.
> **WINDOW is unaffected.**
>
> *This note was written only after the user confirmed the wording: correcting the doc that
> holds the user's own design is itself the move the new supersession rule guards against,
> which is why the sweep flagged it rather than fixing it.*

Carried forward to compare against the actual deep-sim loop and discuss next
session; ~~not yet reconciled with how deeptime runs today~~ (reconciled, above). For
a single epoch that runs N chapters, over a fixed pass list:

- Each pass has a **phase length** = a fraction of a chapter's duration (a
  "rate"): e.g. tectonics 1.0, hydro 0.33, weathering 0.1.
- A **canonical start order** (tectonics → hydro → weathering) = the order each
  pass takes its *first* turn. Passes run **serialized**; a short-phase pass runs
  many times between long-phase ones, evenly distributed. Example within one
  chapter: tectonics×1 (whole duration) → hydro×1 (.33) → weathering×3 (.1 each)
  → hydro×1 (.33) → weathering×3 → … ; any remainder phase gets the leftover
  fraction as its duration.
- **A phase is handed the cell state at its start** (as other passes' phases left
  it) and **duration is a scalar on its transformations.** Shorter phase = higher
  temporal resolution / more able to respond to mid-chapter changes. Tectonics has
  nothing to respond to mid-chapter → runs once; weathering runs often.

**Why it matters / convergences already established this session:**
- "Duration scales the transformation" **is the `rate × dt` time-base** the
  behavior model (S16) was written against — the fractional phase *is* the `dt` a
  pass hands its behaviors. The scheduler is the deeptime clock for behaviors.
- Per-pass phase length = **temporal resolution as a first-class, per-pass
  knob** — more expressive than the current uniform per-iteration tick (verify).

**The sharp open fork (touches S16's finding from the other side):** is
weathering **one pass** (agents summed at one cadence — how S16 stayed
byte-identical) or **one pass per agent** (frost seasonal, dissolution slow —
each at its own phase length, which this scheduler enables)? "Agents as terms in
one pass" vs "agents as passes with their own cadence" is the first question to
resolve when cadence opens for real.

**Epochs** (the user called this the "sloppy" half, but it's the cleaner one):
declared with pass members + **(a chapter count OR a world-API terminating
condition)** — the "run N / until" mechanism, with emergent epoch length from
world state (count of a material, landform variability, …). Determinism holds as
long as the terminating condition reads only deterministic world state. Multiple
epochs (deepest runs tectonics/hydro/therm/weather → next adds eco → next adds
socia) are the layering; **deliberately not thought about yet** (user: "don't
think about multiple epochs for a second").

## The segment tree as one primitive for bodies AND vegetation (sketch, 2026-08-01)

*From the same design pass that produced [`posture-gait.md`](posture-gait.md). **Sketch, not
decision** — the posture/gait half was cautiously ratified; this half was discussed and not
put to a vote. Provenance is mixed and marked inline.*

**The proposal:** a creature's limb chain and a plant's branch are the same primitive — an
**addressed segment tree with parametric geometry, produced by a rule**. A body's generator is
an explicit enumeration (which is exactly what `biped_plan()` already is); a plant's is a
recursive production. Addresses rather than names, so a name (`arm_l_upper`) is the degenerate
case of a path (`trunk/branch[2]/leaflet[3]`) — and `refinement.md` § 4 already promoted
*"expression inherits the ADDRESS, not just the value"* to a rule.

**Three things fall out as integrals over the same tree, identically for both kingdoms:**
harvest yield (segment material × volume — a felled tree gives the wood that was in it, no
loot tables); **colliders** (per-segment solidity policy); and **fitness readouts** — leaf area
for a fern, surface-area-to-volume for a wolf. That last one is what lets an evolution pack
*score* a body. Same mass integral as posture (`posture-gait.md` § 5), arrived at independently
from three directions.

**Where they diverge:** articulation (animal joints move per-frame; plants don't — wind is a
shader), and **growth mode** — plants grow by *extending topology*, animals by *scaling
proportions*. Those are exactly `ecology.md` § 4's two evolution operators (*"scalars =
allometric drift; bools = segment gain/loss"*), so **growth and evolution are the same two
operators at different timescales.**

**Segment KINDS are a closed set the machine owns** — the `material-behavior.md` § 2 forms
doctrine, copied verbatim: a kind is a mode the renderer, collider and mass integral must all
reason about, so a pack **picks** a kind and never invents one; the set may grow as machine
work. Opening set: `Box` and `Card`.

**Why vegetation cannot be voxels** (measured, not supposed): forms are a closed set of five
*occupancy modes* with no oriented/shaped member, and `StructureShape` reserves *capacity*, not
geometry — the mesher never consults it. And at 0.9 m voxels with 0.1125 m eighths, moss
(~0.02 m), a flower stem (~0.005 m) and a vine (~0.03 m) are all below the grid entirely. So
foliage is sub-voxel geometry, which is what a body already is.

**User-originated rulings from that conversation, recorded because they are the load-bearing
parts:**
- **The billboard threshold.** *"If it's a candidate to be a billboard instead of 3d, it's
  below a representation resolution threshold where one does not assume they can interact with
  its individual parts: they harvest the whole leaf or whole frond and get the mixed materials
  all at once."* — so **choosing `Card` over `Box` IS the declaration that a part is not
  individually addressable**; geometry kind and interaction granularity are one authored choice.
- **Card, not a thin box.** An extruded box's rim cannot meet a frond drawn in the middle of an
  alpha-cut broad face. *"My elevated pixel is not boxy, it's pixelated with dimension."* POM
  for relief; the silhouette stays the alpha cut. (Assistant note: POM is stable on a
  **fixed-orientation** card and swims on a camera-facing one — and a segment tree supplies
  authored orientation for free. `terrain.wgsl` already packs a height channel in the normal
  atlas's alpha and has a tangent-frame helper; it drives material heightlerp, never parallax.)
- **Bake the frond at deeptime**, per species, shared down the phylogeny by genetic history —
  so the fractal parameters are the genome and the texture is a cached phenotype, and you never
  mutate pixels. *(Assistant flag: that makes generated assets part of world identity, which
  wants fingerprinting the way journal/0126 just did for everything else.)*
- **Mixed materials per segment**, radially ordered — legal here precisely because this is not
  the voxel renderer and can have its own domain-specific shape. Cost is bounded because the
  mixture is **species-level data**, not per-instance.

**The unifying axis, and it appeared three times in one conversation:** identity is a **LOD**.
A meadow is a population and the tuft you pick collapses to an individual; a swarm collapses to
the fly you swat; and *within* one organism, a tree's branching must be topology (you climb it)
while a frond's must be texture (nobody interacts with a leaflet). That is **S-9's
observation-collapse**, and the S2 statistical tier (dependency-graph **E8**, HELD, *"zero
consumers, do not find it one"*) is the primitive built for it. **Unholding E8 is a USER call
gated on bio/eco; nothing here earns it.**

**Not sequenced. Nothing above may be built without its own ratification.**

## Plants: growth shaped by where you grew (flag, 2026-08-01 — user, deferred by the user)

> *"plants having a salted growth pattern / a pose that is expressed differently for different
> members of the same species based on location, surroundings, etc kinds of inputs. Thus
> phototropic canopies etc. **Deserves a deeper talk**, just flagging it while thought of it."*

**Filed, not designed.** Recorded now because it is a live constraint on a tier being built:
`posture-gait.md` bakes **per species**, and this names a class where the per-instance result
is not a small delta but the *dominant* signal — two oaks on the same hillside are different
shapes because one grew in shade and one in wind.

**Why it probably still fits, sketched only:** the bake would hold the **growth RULE**, not the
pose; the rule is then **evaluated per instance against local conditions** (light, wind, slope,
crowding) plus a position salt. That stays S-9 — a pure function of (species rule, address,
sampled environment) with **nothing stored** — and it is the same shape as animal posture,
which also bakes a rule and resolves it against local terrain. The difference is *degree*: an
animal's posture is dominated by its own body, a plant's shape by its environment over time.

**The open part, and why it needs the deeper talk:** growth is a *history*, not a state. A
phototropic canopy is the accumulated record of where the light was over decades — so either
the rule integrates a history cheaply, or the environment is sampled once and the tree is a
snapshot of present conditions, which would get the *shape* right and the *story* wrong.
Related: **[procedural attacks from evolved morphology]** (same file, filed 2026-08-01) and
`posture-gait.md` § 6's individual-delta seam. **Not sequenced.**

## Procedural attacks from evolved morphology (flag, 2026-08-01 — user, deferred by the user)

> *"for the mod pack in the future, we would like to solve procedural scorpion stinger attacks
> etc from an **evolved** scorpion-like animal: **not something to think about yet**, just a
> note."*

**Filed, not designed.** The point is the word *evolved*: nobody authored that species, so
nobody authored its sting. The attack has to fall out of **declared functional parts** —
`posture-gait.md` § 7b's role system, which is the same declaration that carries a stinger, a
snout, an ear and a support kind.

**Why it is worth having been flagged now rather than later:** it is a second, independent
consumer of the role vocabulary, arriving from combat rather than from animation. A vocabulary
designed for *one* consumer is how this project produces stand-ins that become definitions —
and the role system is being folded into the body-plan structure work **this week**, while its
shape is still cheap to widen. Related: `posture-gait.md` § 7b (roles, and the
**cardinality** question — *ruled 2026-08-01: enforcement travels with the gait-bake
binding heir, stubs #34; this flag was itself part of the argument for deferring it*),
decision 17 (functional parts declared, not derived), and
**[Plants: growth shaped by where you grew]** above. **Not sequenced.**

## The WINDOW axis: a closed aggregator vocabulary (assistant-proposed, user-accepted as design-pass INPUT 2026-08-01 — not decided)

Input for the future WINDOW design pass (flow.md § 11.1's ratified-but-unbuilt third
scheduler axis), captured when the fluvial record-terms ruling deferred χ recording
behind it. The user's open question it addresses: *"the best balance of freedom and
small bug surface for window declaration."*

**The sketch — the `DitherSource` balance applied to time-aggregation:** the engine owns
a **closed aggregator vocabulary** (Sum · Max · Mean · perhaps Last), each implemented
and tested once, engine-side; a pass **declares, per recorded quantity, which aggregator
and what window**; the record **self-describes both** (flow.md § 11.3 — a reader can
never hold a number without its meaning). Freedom where it is cheap: any quantity, any
blessed meaning, any granularity, all authorable by a mod. Bug surface where it stays
small: no author-supplied aggregation code, no unstated meanings. The residual failure
mode is honest — picking Mean where Max was wiser is a *tuning* mistake, visible in
data, not a silent-meaning defect.

First forced customers, in likely order: χ (the confinement mobility hint — non-additive,
deferred 2026-08-01 exactly for this axis), peak discharge, the episodic-event limit
(flow.md § 5 limit 2 — a turbidite averages away inside a 25-epoch bucket while its
graded bed is the whole signature).

## The engine is general at heart — the pack decides what kind of game it is (USER SKETCH, 2026-08-03)

Captured verbatim from the geo session's refinement conversation (context: the
borehole story, the engine-vs-pack partition analysis, and the subtraction test —
"subtract the pack and the world must lose every feature while keeping every
conservation property"):

> *"we could write a pack that just **is minecraft** - no history, just diverse
> biomes. or we could write a pack that just is a superstructure, refined in
> stages, no story or earth. the engine gives you storage buckets and does to
> them what you tell it to do to them, including iteratively, and
> interpolates/refines/presents them however you say, by layers of authored
> refinement machinery, built on engine primitives, that reads the record left
> by the deeptime passes. a pack could use the deeptime iteration to determine
> different themed areas ('this area is maze-like, this area is cathedral like'),
> and construct most of the content of that ahistorical labyrinth world entirely
> in the refinement. (i'm thinking past our stated plans right now, for what
> refinement might mean, for how structures might be implemented eventually in
> our pack, and so on.)*
>
> *while it will first be an engine very good at earth science simulation - it is
> something more general at heart, following from the engine/pack split. it
> provides primitives for space, time, physics, bodies, materials... (items,
> structures, recipes, communities, agents, etc... eventually...). the catered
> blending of packs will lead to some bizarre and compelling experiences in the
> modding community."*

**Status: recorded ambition** (the CLAUDE.md doctrine: future-tense, user-authored,
retires no goal and builds nothing). Explicitly *thinking past stated plans* — this
does not re-prioritize the board, and the structures/communities/agents tail sits
behind the P9 gate, which stays a user call.

*Assistant notes, marked as such (what the two hypothetical packs would stress in
the bones as built — useful the day either is attempted, and as engine-shape
probes today):*
- *The Minecraft pack is a world that is pure initial condition + presentation:
  the `Schedule` variants `Seed`/`SeedAndStep` — built 2026-07-29 with **no
  production declarer** (graph E3) — are exactly its shape; the biome field is the
  facies driver row of the tiers-layer table, and "no history" = an empty cadence
  table.*
- *The labyrinth pack stresses: move C (bounded stochastic detail synthesis — the
  third coarse→fine move, still ownerless), Law 3's conserved quantity (mass-shaped
  today; a structure pack wants per-term conservation contracts declared by the
  family), Law 2 (themes are categorical drivers — "maze-like here" is a tag, and
  tag boundaries need the same discipline as facies: correlate/dither/earn the
  sharpness), and E6/E7 (its resource vocabulary and pass order are nothing like
  vanilla's).*
- *The engine-side honesty tests are domain-free already: the subtraction test,
  Law 1/2/3, runtime-is-sacred, purity. Nothing in them names earth.*
