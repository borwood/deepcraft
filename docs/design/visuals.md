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

## Custom shaders

The pack system (ARCHITECTURE.md § Rendering) is the vibe-agnosticism valve:
our default pack IS this document; packs may take the same world elsewhere.
The S4 contract must expose: the three material textures as packed above,
mixture/splat inputs, weather/wetness state, time-of-day, fog/haze parameters,
emissive surfaces, and the pass points for sky, water, shadows, shafts, and
post (tonemap/fog).
