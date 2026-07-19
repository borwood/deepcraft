# Placeholder LabPBR texture packs

> **PLACEHOLDER — NOT ART DIRECTION.** These are procedurally generated
> assets-in-waiting for the future splat-blend milestone (see
> `docs/design/visuals.md` § Mixture rendering road, DECIDED 2026-07-19). The
> textures exist so that milestone opens with *something* to blend; they are
> deliberately flat, pixel-art placeholders derived from material property
> sheets, not authored art. `visuals.md` owns the real destination. Real
> texture authoring is decided when that milestone opens. **Do not treat any
> pixel here as a look we've committed to.**

## What this is

One 16×16 three-texture set per material (17) plus per block type that has no
material twin (4: stone, dirt, grass, wood). The four geology blocks
(mudstone/sandstone/granite/basalt) **share** the corresponding material packs.
Everything is generated deterministically by
[`tools/gen_placeholder_textures.py`](../../../tools/gen_placeholder_textures.py).

```
placeholder-labpbr/
├── manifest.json        # pack format, per-material params, block↔pack map
├── README.md            # this file
└── <slug>/
    ├── basecolor.png    # 16×16 RGBA
    ├── normal.png       # 16×16 RGBA
    └── specular.png     # 16×16 RGBA
```

## Channel packing (LabPBR-compatible, from visuals.md)

| File          | R                     | G                       | B                          | A                       |
|---------------|-----------------------|-------------------------|----------------------------|-------------------------|
| `basecolor`   | albedo R              | albedo G                | albedo B                   | opacity (255 = opaque)  |
| `normal`      | tangent normal **X**  | tangent normal **Y**    | ambient occlusion          | height (parallax)       |
| `specular`    | perceptual smoothness | F0 (0–229) / metal (230–255) | porosity (0–64) / SSS (65–255) | emission (**255 = none**) |

Notes on the LabPBR conventions used:

- **Normal**: X,Y are stored; Z is reconstructed by the shader
  (`z = sqrt(1 − x² − y²)`). B carries AO, A carries height for POM/parallax —
  the standard LabPBR reuse of the normal map's B/A.
- **Specular G**: dielectrics store a small linear F0 (~10). Metals use the
  predefined-metal range; **gold-dust uses id 231 ("gold")**.
- **Specular B (porosity)**: placeholders use the porosity sub-range (0–64).
  Low-density loose material reads porous (snow ≈ 58), dense rock low
  (granite ≈ 10), metal ≈ 0.
- **Specular A (emission)**: 255 means *no emission* (so nothing glows by
  default). Only gold-dust carries a slight non-255 value (8) as a placeholder
  glint.

## How the character is derived

Each pack seeds a value-noise field from a **stable** hash of its slug
(`sha256`, not Python's per-process-randomized `hash()`), so regeneration is
byte-for-byte reproducible. Texture character comes from the material property
sheet (mirrored from `crates/dc-core/src/materials/mod.rs`):

- **grain size → noise frequency + ramp length**: fine silt/clay = smooth,
  high-frequency fine speckle; gravel/scree/potsherd = chunky low-frequency
  blobs, with more basecolor shades.
- **hardness → normal relief amplitude**: hardness is the mean of the
  dig/chop/smash/cut extraction resistances; harder rock gets deeper relief.
- **cohesion + fineness → perceptual smoothness** (specular R).
- **density → porosity** (specular B): loose = porous, rock = low, metal ≈ 0.
- **gold-dust** additionally gets metallic F0, sparse gold flecks in basecolor,
  and the slight emission glint noted above.

Basecolors are quantized to a 4–6 shade ramp around the material's palette
color so they read as pixels, not noise mush. Palette colors for blocks and
geology match the in-game vertex palette (`crates/dc-client/src/meshing.rs`,
`face_color`).

## Regenerate

From the repo root (no Rust build involved — pure Python, stdlib only for
writing; Pillow used only for the read-back self-check if present):

```
python tools/gen_placeholder_textures.py          # generate all + self-check
python tools/gen_placeholder_textures.py --check   # verify existing PNGs only
```

PNGs are written by a dependency-free encoder (zlib + struct), so output does
not drift with a Pillow version.

## Flagged for the real texture milestone

- Height and AO are both derived from the *same* noise field — fine for
  placeholder, but real packs will want AO baked from actual meso-geometry.
- Grass has a top/side albedo split in the mesher; this placeholder uses only
  the top (green) color. Real grass wants a distinct side pack.
- Wood is treated as generic granular noise — no directional grain. A real
  wood pack wants anisotropic grain lines.
- Opacity is 255 everywhere; no translucency/foliage cutouts yet.
- Emission is off everywhere except the gold-dust glint; real emitters (lava,
  forges, bioluminescence — see visuals.md § Lighting) will drive the A channel.
