# Visual direction — S4's design brief

Captured 2026-07-18 from design discussion. Thesis: **an elevated pixel game**
— pixel-realism in the modded-Minecraft-with-shaders lineage: shaders elevate
the core pixel vibe (depth, light, atmosphere) rather than replace it with
realism. If you squint: ground, not tiles — but it never pretends not to be a
voxel game.

## Mood

Desaturated, moody. Dungeon diving, crypt exploration, mystery, isolation —
punctuated by cozy hearth interludes, sublime vistas, distant civilizations.
Real darkness underground (darkness is a gameplay material; no floaty ambient
minimum). Warmth is earned: firelight, hearths, emissives.

## Material/texture model

- **PBR + POM, LabPBR-compatible three-texture packing** (adopting the
  community standard buys an existing artist/shader ecosystem):
  - basecolor: albedo RGB + opacity A
  - normal: XY (reconstruct Z) + AO (B) + height (A) — height drives parallax
  - specular: perceptual smoothness (R), F0/metal (G), porosity/SSS (B),
    emission (A)
- **Resolution: try 16×16, expect 32×32** (at 0.9 m voxels: 5.6 vs 2.8
  cm/texel; MC vanilla is 6.25). One resolution per pack; keep them small.
- **Blended granular surfaces** (the materials-model mixtures): height/AO
  driven splat blending with contrast (heightlerp) — pixelated leaves visibly
  poke through sand rather than alpha-mushing. The blend reads as pixels
  composing ground.
- **Sim-driven channels**: per-voxel porosity from the materials system
  (structure fill, packed pores) feeds the porosity channel — rain-darkening
  and wetness respond to *simulated* material state, not artist guesses.
  Emission channel drives colored light emitters.

## Lighting

- Colored point lights: yes, first-class (lava, forges, bioluminescence).
- Shadowed sun + light shafts/godrays: yes, **with knobs/toggles** (chasm
  shafts are the signature shot; must degrade gracefully).
- GI: explicitly not pursued.
- Skylight under cubic chunks per S3's contract (summaries, optimistic-sky).

## Atmosphere & weather

- Aerial perspective/haze is load-bearing for the 1+ km vistas (and hides LOD
  seams); driven by weather + environment state.
- Ground fog: wanted; truly volumetric vs layered fake TBD by budget.
- **Dynamic weather is a committed feature ("when", not "if")** — and it
  couples to the materials sim (snow/sand deposition, wetness/porosity).

## Water

Semi-realistic in the MC-shader tradition: animated surface, depth fog, shore
blending. Dynamic detail (foam etc.) rendered as pixelated textures, not
smooth sprays — pixel-realism extends underwater. Water placement is physical
(aquifers, porous stone) and the look should honor that.

## PBR-1 walk-14 ratifications (2026-07-19, user)

- **`SPLAT_N = 4` RATIFIED** — the per-face material cap under the 8-slot
  storage. The reconciling mechanism (as-built) is exactly the condition
  the user set: each face gets its own weighted world-anchored roll of
  which ≤4 of the voxel's constituents paint it, so across faces all 8
  slots participate; no single face ever needs more than 4 to read as the
  mixture.
- **Sun/ambient calibration: rides as placeholder, un-ratified by intent**
  — lighting design awaits the day/night + shadows design (PBR-2+).
  *(A "noted user lean" toward **possibly no darkness at all, even
  underground**, with an attributed quotation, stood here from 2026-07-19
  to 2026-07-20. **The user states it is a MISATTRIBUTION** — corrections
  #13. Struck. The real position is DECIDED below: real darkness
  underground.)*
- **Placeholder texture tiling: cleanup pass ratified** — walk-14 photos
  show obvious per-block repetition/boundaries on same-material runs;
  worth a parallel texture/UV cleanup pass (dispatched same day).
- **Fullbright mixture visibility is REQUIRED** — the walk protocol and
  screenshot auditability need an AI viewer to *notice mixtures* without
  lighting noise: flat albedo + the old world-anchored speckle look,
  shader-side (no mosaic geometry). Albedo-only stays the default
  diagnostic register (flat color is easiest for shape detection);
  texture-confirmation shots use the lit path.

## The distance speaks the voxel language — DECIDED 2026-07-19 (user)

The far-field horizon's smooth heightfield TIN is REJECTED as a visual
direction: "the distance transition from block to smooth is obviously
wrong to any eye." The world is voxels at every distance. Decided shape:

- **FF2a — voxelize the far field** (next increment): quantize the same
  coarse column summaries to voxel steps and mesh stepped columns
  (Distant-Horizons-style). Because worldgen terrain is currently
  column-shaped (no caves/overhangs exist yet), this is FULL visual
  fidelity today, not an approximation. Keeps the zero-mismatch corner
  property and the summary architecture.
- **FF2b — coarse volumetric summaries**, paired and sequenced to land
  WITH underground/overhang features (the water→caves design thread):
  when the world grows things a heightfield cannot say, the far field
  grows the 3D summary to say them. Couples to S3 region storage for
  persistence.

Process note (integrator breach, guarded in session-workflow): this
direction shipped without the user's eye because the agent's flag said
"follows the promoted architecture" — architecture and APPEARANCE are
different ratification axes; appearance is always user-owned.

## Lit-mixture visibility — DECIDED 2026-07-19 (user mechanism)

Walk 16 proved the lit heightlerp ERASES low-fraction constituents (a ⅛
accessory never out-elevates a ⅞ host — 0021-mixture-* pair). Cell-hash
quantization REJECTED (user): an imposed grid shears authored texture
features (multi-pixel cobbles must never be cut). The decided mechanism:

- **Amplitude by rarity**: each constituent's height term scales inversely
  with its fraction — `elev_i = w_i + amp(w_i)·(h_i − ½)`, amp large when
  w small. Rare constituents win sparsely but decisively AT THEIR OWN
  HEIGHTMAP PEAKS — whole features pop in or stay out, nothing shears;
  coverage ≈ proportional for well-behaved heightmaps, and deviation is
  authorable: **a pack's height channel is its ore-clumping knob**.
- **Plus anti-wallpaper jitter**: heights tile per voxel, so peak-wins
  would repeat as a periodic lattice; a small per-voxel world-hash bias in
  the elevation battle (LOW-amplitude, never a cutting grid) decorrelates
  which peaks clear the bar, voxel to voxel.
- Consequence accepted: lit and fullbright agree **statistically**
  (fractions), not spatially (cell speckle vs height peaks) — audits
  compare presence/proportion, not positions.
- Calibration constants (amp curve, jitter magnitude) are engineering,
  tuned photographically; live smoke mandatory (WGSL invisible to gates).

## Mixture rendering road — DECIDED 2026-07-19

The splat-blend destination above stands. The road to it:

- **Interim vertex-color materialization first (3c-2)**: every material def
  carries an albedo in the registry (pack data); a mixed voxel's face renders
  a **world-anchored deterministic dither** — each face pixel-cell picks one
  constituent's color by position hash, weighted by its eighths fraction.
  Pixels composing ground, no assets required. The later splat pipeline
  replaces the color source, not the data plumbing.
- **Placeholder LabPBR packs begin in parallel**: procedurally generated
  16×16 three-texture sets per material, so the splat milestone starts with
  assets waiting. Real texture authoring decided when that milestone opens.
- **Loose (granular) materials render partial-height** by eighths
  (snow-layer style). Collider stays binary for now (solid ≥ 4/8) — the
  visible mismatch is accepted until movement learns partials; **"sinking"
  rules (knee-deep snow) are a deliberate future step**, as is body-driven
  compaction (packing the bottom layer underfoot atop solid — materials.md).
- **Ore is subtle**: member-level tint variation within class and sparse
  close-range speckle only — no distance glint. Recognizing ore-bearing
  ground is knowledge gameplay (prospecting, panning, surfacing thoughts),
  not a highlight shader.

## Custom shaders

The pack system (ARCHITECTURE.md § Rendering) is the vibe-agnosticism valve:
our default pack IS this document; packs may take the same world elsewhere.
The S4 contract must expose: the three material textures as packed above,
mixture/splat inputs, weather/wetness state, time-of-day, fog/haze parameters,
emissive surfaces, and the pass points for sky, water, shadows, shafts, and
post (tonemap/fog).

## DECIDED 2026-07-20 (user) — real darkness underground; the reference is modded MC + shaders

Confirms and sharpens § Mood ("real darkness underground ... no floaty
ambient minimum"). A contradicting "user lean" toward no darkness at all
sat in § PBR-1 walk-14 ratifications from 2026-07-19; the user identifies
it as a **misattribution** (corrections #13) — it was never their position,
so there was no real contradiction to resolve, only a bad record to strike.

The user, on seeing the 0027 walk (coal rendering black, excavation
interiors unlit):

> underground is lit by global sun right now, depending which way the face
> faces it has one of six levels of face light, essentially. but I *don't*
> like that, our visuals inherit from modded minecraft with shaders: much
> nicer looking, much darker underground, with shadows, etc. shaders can't
> own light totally, of course - the sim needs to know about light.

- **Reference point: modded Minecraft with shaders** — not vanilla MC, not
  a generic PBR look. That is the target for "much nicer looking".
- **Much darker underground, with shadows.** The present model (a global
  sun giving each face one of ~six brightness levels by orientation, no
  occlusion) is explicitly rejected as the end state — it is why an
  excavation interior is unlit and why coal reads as a void.
- Consistent with PIPELINE.md's "real darkness, no ambient floor" as a
  lighting-model property, and with § Mood's "warmth is earned".
- **Consequence for the 0027 findings**: the fix for coal-reads-black is
  NOT to brighten coal (its 0.07 albedo is physically right). It is the
  lighting model — which is PBR-2's tonemap/HDR + shadows + earned
  firelight. This makes PBR-2 the gate on the underground being *lookable
  at*, not merely prettier.

### Open thread, NEW 2026-07-20: the sim must know about light

> shaders can't own light totally, of course - the sim needs to know about
> light.

No prior in the corpus on a **sim-side light field** (light as simulation
data, MC-style propagated light levels) as distinct from render-side
lighting. Recorded now as an open design thread, undesigned:

- **The first real consumer, from the user 2026-07-20: embodied agent
  perception.** Clarifying corrections #13, the user distinguished two
  registers that had been conflated:
  - **Dev/debug visuals, and visuals for AIs driving characters over MCP
    for development** — fullbright, no darkness, maximum legibility. This
    is the existing albedo-only diagnostic register and it stays.
  - **An AI that is IN-GAME rather than dev/debug** — "probably they do
    actually need darkness, to be fair to players." Perception parity is
    the principle: an MCP-driven character must not see better than a
    human player simply because it reads a different channel.
- **What sim light IS, per the user**: "maybe this is a simulated
  dark/light like minecraft blocklight — the simulation level of our
  light, **not crisp dynamic shadows etc, just what the voxels know**."
  So the fidelity target is deliberately coarse: a propagated per-voxel
  light level, not a sampling of the rendered image.
- **Therefore renderer light and sim light are two different things with
  two different consumers, by design** — the shader may do shadows,
  godrays, HDR and tonemapping; the sim carries a blocklight-grade field.
  They must not be assumed to agree, and neither is derived from the
  other. (The natural seam for the embodied case is the character sense
  surface, which already exposes diegetic pose/raycast/surroundings —
  a light sense joins those, and the dev surface keeps full reach as it
  already does for character control.)
- **This is also what makes "darkness is a gameplay material" true rather
  than decorative** (§ Mood): if only the shader knows about darkness,
  darkness is a look; if the voxels know, darkness can gate behaviour —
  perception, spawning, growth, stealth.
- **Other candidate consumers** (unconfirmed, listed for the design):
  photosynthesis and plant growth (S10's biology already gates on
  moisture/temperature — light is the missing axis, and it is what makes
  caves lightless *in the simulation*, not merely visually); creature and
  spawn behaviour; stealth and NPC vision.
- **Determinism**: a sim light field must be deterministic and replayable,
  entropy from seeds only — the same law as every other sim field.
- **Assistant observation (PROPOSAL)**: a propagated sim light field and
  water.md's bound-water saturation field are **the same computational
  shape** — a bounded local relaxation over the voxel grid, attenuated by
  a per-material property (opacity for light, permeability for water),
  with sources and sinks. If S11 finds the saturation relaxation is local
  and cheap, that machinery is a candidate for both. Worth checking before
  either is built twice.
- Open: resolution (per voxel? per chunk-column?), whether sky light and
  block light are separate channels (MC keeps them separate for a reason —
  day/night cycles must not require re-propagating block light), update
  cost on edits, and how the sim field relates to the *rendered* lighting
  (they are not the same thing and must not be assumed to match).

## DECIDED 2026-07-20 (user) — the sim light model

Answers to the sim-light design questions. Ratified except where marked
OPEN.

1. **Light is DERIVED, not stored.** Light attenuates, so it has a hard
   locality bound *by construction* (reach = emission ÷ attenuation) —
   unlike water, whose locality had to be measured. Derive per chunk
   within that halo; do not persist. Matches the house pattern (derive
   columns/voxels, persist edits/bodies) and dodges ~32 KB/chunk of light
   bytes.
2. **Sky and block light are separate channels — with a large amendment
   from the user: skylight is DIRECTIONAL, and there is a heavenly-bodies
   field.** See § below; this is the biggest departure from the Minecraft
   model.
3. **Emission and opacity come from material properties, not special
   cases.** Lava glows because its material says so; bioluminescent fungi
   because their organism def says so; water attenuates less than stone
   because its sheet says so. No light-source-specific code — the placer
   pattern. (The LabPBR specular channel already carries emission.)
4. **Reach is per-source, not a global constant** (user): "would be nice
   if it depended on the source attenuation tbh. a better material for a
   torch - more surrounding lit." Falls out of 3 — emission strength is a
   material property, attenuation is per-material-per-step, reach is where
   it drops below threshold. Gives crafting real stakes: **torch quality
   is a material question.** The only global is the representable cap,
   which is a knob per the knob doctrine.
5. **Deep time does NOT get a light field** (user: "quite right"). At
   deep-time scale surface is lit and subsurface is dark; the biotic layer
   treats it as binary. Bioluminescence exists in caves but is not a light
   source for photosynthesis or heat, so it does not change this.
6. **Darkness gates spawning; light is equipment.** User: "torches are
   definitely equipment... tools and equipment mean something in this
   game, and the dark may be quite dangerous (esp depending on content
   pack)." So darkness is a gameplay material with a fuel cost, and its
   danger level is content-pack-tunable.
7. **Caves get their own ecology** (user: "if it happens on earth we give
   it an honest gesture here"). Falls out of biome-as-diagnosis: "no
   light, stable temperature, wet, fed from outside" is a set of
   conditions like any other, so the ecology sim diagnoses cave communities
   once light is an axis. No cave-specific ecology system.

### The heavenly-bodies light field (user, 2026-07-20) — the big amendment

> it would be nice if skylight were quantized directional... hard to
> imagine a sim world where sun on the horizon doesn't shine into the cave
> mouth or the house's windows or the overhang, and the sim doesn't know
> it. additionally it may be a vector of multiple solar sources (foresee a
> future mod that wants to add more moons on different axes, different
> colored, different brightness.. heavenly bodies field as a light
> primitive). sun and moon as the first two bodies feeding the heavenly
> light field. we do a cheap sky bounce too: if a body is above the
> horizon, it contributes a small downward heavenly light (maybe adjusted
> for cloud cover) in addition to its directional.

- **A heavenly body is the light primitive**: direction (a function of
  time), colour, intensity, and its own rise/set. **Sun and moon are the
  first two entries, not special cases.** A mod adding moons on other axes
  is adding rows to a table — cheap and open by construction.
- **Each body contributes two terms**, exactly as the user split them:
  a **direct** directional term, and a **cheap sky bounce** — a small
  downward-diffuse contribution while the body is above the horizon,
  plausibly modulated by cloud cover.
- **Assistant proposal (not ratified): direct light is a QUERY, bounce is
  the FIELD.** The bounce term is direction-free and propagates like
  Minecraft skylight — cheap, per-voxel, the thing worth storing/deriving.
  The direct term is a *visibility question along a known direction*
  ("is this voxel lit by body N right now?"), which is a shadow ray
  answered on demand, not a field to maintain. If direct light is a query,
  it need not be quantized at all and can use the body's exact current
  direction — the user's apologetic "quantized" may be unnecessary. This
  is what makes sun-into-the-cave-mouth affordable.
- Ties to the S3 skylight contract (ROADMAP): direct-sky queries are
  exactly the bounded, summary-consulting, optimistic-sky queries S3
  specified.

### Body PATHS — the day/night cycle is emergent, and the seasons seam (user, 2026-07-20)

> day-night cycle would be emergent from heavenly bodies paths. by the way:
> seasons. don't need to flesh that out or build yet, but have the seam in
> mind - the path of a heavenly body can change on a cycle like the
> perceived path of the sun, additionally we could actually change the
> perceived path of a heavenly body depending on region latitude (our poles
> and tropics fall out). again - not planning all of that now, just building
> robust path ability for heavenly bodies.

- **There is no separate day/night cycle system.** Day and night *emerge*
  from body paths. This closes the "we have no ephemeris" gap raised in
  design discussion — the ephemeris **is** the paths, and it is the same
  primitive, not an extra one.
- **A body's path is parameterised, not a fixed direction function.** Build
  it as `direction = path(body, world_time, observer_latitude)`, with:
  - **a long cycle over which the path itself changes** — the seam for
    **seasons** (the sun's perceived arc shifting across a year);
  - **a latitude term** — so the perceived path differs by region, and
    **poles and tropics fall out** rather than being authored.
- **BUILD NOW: only the robust path capability.** Seasons and latitude
  effects are explicitly NOT planned or built yet (user). The requirement is
  that the path abstraction can express them later without a rewrite — the
  same non-preclusion discipline as the erodibility/limestone case.
- **The latitude input already exists**: pregen cells carry `lat_deg`
  (`collapse.rs::climate_at` bilinearly interpolates it for the climate
  field), so an observer's latitude is already a cheap query.
- World time must be a deterministic tick counter, never a wall clock
  (project law).

### Block light without items: the dev-light block (user, 2026-07-20)

Block light was noted as blocked on placeable sources, which need the
unbuilt item system. The user's unblock: **"we can always make a dev-light
block that does not naturally spawn but is just a point-light block, pretty
trivially."** So block-light propagation can be developed and tested
against a dev-only emissive block long before torches exist as items. Plants
and their light budgets are explicitly deferred ("we can hash out plants
later").

### OPEN — colour, and creatures that see bands we do not

The user pushed back on monochrome sim light:

> the ACTUALLY COOL sim thing light color could give us: a creature (or
> character!) that can, say, SEE ULTRAVIOLET etc. do we kill it or is this
> actually deep esp if we go evolution route for bio?

Assistant recommendation (NOT ratified): **do not build it, do not
foreclose it.** With the direct/bounce split, colour is nearly free *on
the direct term* — colour is a property of the heavenly body or the
emitting material, so it rides the source, not the propagated field. Model
a source's emission and a sensor's sensitivity as a **band vector** (v1
may have exactly one band), so "sees ultraviolet" becomes a data
relationship between an organism's sensitivity and a source's spectrum —
the roles-as-contracts pattern — rather than new machinery. Cost today is
approximately a one-element array instead of a scalar; the payoff is that
UV/infrared vision, and its arrival via the evolution route, never needs a
rewrite. Same discipline as the erodibility/limestone non-preclusion case.
