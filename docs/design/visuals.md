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
  Noted user lean for that design: **possibly no darkness at all, even
  underground** ("truly i don't think any darkness" — a legibility-first
  stance, not yet a decision).
- **Placeholder texture tiling: cleanup pass ratified** — walk-14 photos
  show obvious per-block repetition/boundaries on same-material runs;
  worth a parallel texture/UV cleanup pass (dispatched same day).
- **Fullbright mixture visibility is REQUIRED** — the walk protocol and
  screenshot auditability need an AI viewer to *notice mixtures* without
  lighting noise: flat albedo + the old world-anchored speckle look,
  shader-side (no mosaic geometry). Albedo-only stays the default
  diagnostic register (flat color is easiest for shape detection);
  texture-confirmation shots use the lit path.

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
