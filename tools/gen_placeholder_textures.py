#!/usr/bin/env python3
"""Deterministic placeholder LabPBR texture-pack generator for deepcraft.

Design source: docs/design/visuals.md § Material/texture model and § Mixture
rendering road (DECIDED 2026-07-19). These are ASSETS-IN-WAITING for the future
splat-blend milestone -- procedurally generated 16x16 three-texture sets, one
per material (and per block that has no material twin), so the milestone opens
with something to blend. Real texture authoring is decided when that milestone
opens; nothing here is art direction.

LabPBR three-texture packing (restated from visuals.md):
  basecolor.png : albedo R, G, B          + opacity A
  normal.png    : tangent-normal X (R), Y (G) + AO (B) + height (A)
  specular.png  : perceptual smoothness (R) + F0/metal (G)
                  + porosity/SSS (B)       + emission (A)

Determinism: every material seeds a value-noise field from a STABLE hash of its
slug (sha256, NOT Python's per-process-randomized hash()). Regeneration is
byte-for-byte reproducible. No wall clock, no ambient randomness (mirrors the
dc-core entropy rule: all randomness flows from an explicit seed).

Texture character derives from the material property sheet (mirrored from
crates/dc-core/src/materials/mod.rs -- prototype values, kept in sync by hand):
  * grain size  -> noise frequency (fine silt = smooth/high-freq; gravel/scree
                   = chunky low-freq blobs) and basecolor ramp length,
  * hardness    -> normal-map relief amplitude (mean of the dig/chop/smash/cut
                   extraction resistances; harder rock = deeper relief),
  * cohesion+grain -> perceptual smoothness (specular R),
  * density     -> porosity (specular B); low-density loose material (snow, ash,
                   leaf litter) reads as high porosity, dense rock as low,
  * gold-dust   -> metallic F0 (LabPBR metal id 231 "gold") + sparse gold flecks
                   + a slight emission so it glints as placeholder.

Usage:
  python gen_placeholder_textures.py          # generate everything + self-check
  python gen_placeholder_textures.py --check   # verify existing PNGs only

PNGs are written with a dependency-free pure-Python encoder (zlib + struct) so
output never drifts with a Pillow version. Self-check reads a sample back with
Pillow when present, otherwise with a tiny built-in PNG reader, and asserts
16x16 RGBA.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import struct
import zlib

# --------------------------------------------------------------------------
# Output layout
# --------------------------------------------------------------------------
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
OUT_DIR = os.path.join(REPO, "assets", "textures", "placeholder-labpbr")
SIZE = 16
PACK_FORMAT_VERSION = 1

# --------------------------------------------------------------------------
# Property sheets.
#
# The 17 material rows MIRROR crates/dc-core/src/materials/mod.rs REGISTRY
# (density_kg_m3, grain_size_mm, cohesion, extraction_resistance[dig,chop,
# smash,cut,sieve]). Only the fields the generator reads are copied; if the
# Rust registry changes, update these to match. Base albedos: geology/block
# materials use the in-game palette from crates/dc-client/src/meshing.rs
# (face_color); the loose granular materials get sensible placeholder colors.
#
# Block-only rows (no material twin: stone, dirt, grass, wood) carry SYNTHETIC
# property sheets so the same derivation drives them; they are flagged kind=
# "block". The four geology blocks (mudstone/sandstone/granite/basalt) share the
# material packs -- see BLOCK_SHARES below -- and are not regenerated.
# --------------------------------------------------------------------------

# fields: slug, display_name, base_rgb(0-1), density, grain_mm, cohesion,
#         resist=[dig,chop,smash,cut], kind, metal
MATERIALS = [
    # 12 S8 debris materials --------------------------------------------------
    ("sand",            "sand",            (0.80, 0.72, 0.52), 1600,  0.5,   0.05, [1.0, 6.0, 4.0, 5.0], "material", False),
    ("gravel",          "gravel",          (0.50, 0.48, 0.45), 1800, 20.0,   0.02, [1.6, 7.0, 3.0, 6.0], "material", False),
    ("snow",            "snow",            (0.92, 0.94, 0.98),  300,  1.0,   0.30, [0.4, 5.0, 1.5, 2.0], "material", False),
    ("leaf-litter",     "leaf litter",     (0.40, 0.30, 0.14),  150, 25.0,   0.15, [0.2, 1.0, 1.2, 0.8], "material", False),
    ("clay",            "clay",            (0.62, 0.48, 0.38), 1750,  0.002, 0.90, [2.6, 4.5, 3.5, 2.2], "material", False),
    ("silt",            "silt",            (0.58, 0.50, 0.38), 1500,  0.02,  0.50, [1.4, 4.0, 3.2, 2.0], "material", False),
    ("potsherd",        "potsherd",        (0.60, 0.34, 0.24), 1900, 40.0,   0.00, [2.0, 5.5, 1.0, 4.0], "material", False),
    ("knapping-debris", "knapping debris", (0.42, 0.42, 0.46), 2300, 15.0,   0.00, [1.8, 6.5, 2.5, 5.5], "material", False),
    ("ash",             "ash",             (0.32, 0.31, 0.30),  700,  0.05,  0.10, [0.6, 3.0, 2.8, 1.6], "material", False),
    ("loam",            "loam",            (0.36, 0.26, 0.17), 1300,  0.1,   0.35, [0.9, 3.5, 3.0, 1.8], "material", False),
    ("scree",           "scree",           (0.48, 0.46, 0.44), 2000,100.0,   0.05, [2.4, 8.0, 3.8, 7.0], "material", False),
    ("bone",            "bone",            (0.86, 0.82, 0.70), 1100, 60.0,   0.00, [1.2, 2.0, 1.1, 1.5], "material", False),
    # v1 geology set (also the shared packs for the geology block tier) --------
    ("mudstone",        "mudstone",        (0.46, 0.26, 0.20), 2400,  0.004, 0.95, [3.2, 5.0, 4.2, 2.8], "material", False),
    ("sandstone",       "sandstone",       (0.76, 0.66, 0.44), 2350,  0.3,   0.85, [3.6, 6.5, 4.6, 5.2], "material", False),
    ("granite",         "granite",         (0.66, 0.56, 0.58), 2700,  3.0,   1.00, [6.0, 9.0, 5.5, 8.0], "material", False),
    ("basalt",          "basalt",          (0.14, 0.14, 0.16), 2900,  0.05,  1.00, [5.5, 9.5, 5.0, 8.5], "material", False),
    ("gold-dust",       "gold dust",       (0.80, 0.66, 0.28),16000,  0.8,   0.02, [1.1, 6.0, 4.4, 5.0], "material", True),
    # 3d roster-proof widening (second member per v1 class + one accessory) ----
    # MIRROR crates/dc-core/src/materials/mod.rs ids 17..=21.
    ("siltstone",       "siltstone",       (0.52, 0.47, 0.40), 2300,  0.02,  0.90, [3.0, 5.2, 4.0, 2.6], "material", False),
    ("conglomerate",    "conglomerate",    (0.60, 0.52, 0.44), 2500,  8.0,   0.80, [3.8, 6.8, 4.8, 5.6], "material", False),
    ("diorite",         "diorite",         (0.55, 0.55, 0.57), 2800,  2.0,   1.00, [6.2, 9.2, 5.6, 8.2], "material", False),
    ("andesite",        "andesite",        (0.42, 0.40, 0.40), 2650,  0.08,  1.00, [5.6, 9.4, 5.2, 8.4], "material", False),
    ("olivine",         "olivine",         (0.42, 0.52, 0.28), 3300,  1.5,   0.90, [5.0, 8.5, 5.0, 7.5], "material", False),
    # organic rocks (journal/0026) -- the biotic layer's facies at the material
    # tier. MIRROR crates/dc-core/src/materials/mod.rs ids 22..=24.
    ("peat",            "peat",            (0.24, 0.17, 0.11),  400,  5.0,   0.45, [0.5, 1.6, 1.8, 1.0], "material", False),
    ("coal",            "coal",            (0.07, 0.065,0.06), 1350,  0.05,  0.90, [2.4, 4.0, 2.2, 3.0], "material", False),
    ("carbonaceous-mudstone", "carbonaceous mudstone",
                                           (0.21, 0.18, 0.15), 2200,  0.004, 0.92, [3.0, 4.8, 3.9, 2.6], "material", False),
    # charcoal (journal/0063) -- the fire-residue INCLUSION. MIRROR
    # crates/dc-core/src/materials/mod.rs id 25.
    ("charcoal",        "charcoal",        (0.045,0.042,0.04),  350,  2.0,   0.05, [0.3, 0.8, 0.5, 0.4], "material", False),
    # v1 SUBSTANCE placeholders -- PRE-REGISTRY (docs/design/ores.md). These
    # materials are NOT yet in crates/dc-core/src/materials/mod.rs; the
    # registry/atlas wiring is the deliberately-deferred upstream engineering.
    # Their packs are pre-staged here so the wiring day finds them already
    # built. The atlas loader (dc-client/terrain_material.rs) iterates the
    # REGISTRY, never this directory, so a slug the registry doesn't know is
    # inert -- these dirs are never read until the material is registered.
    #
    # journal/0048 correction (undoing 0045): ore is a MATERIAL inside a HOST
    # (grade-is-eighths, geology.md § Ore, DECIDED 2026-07-20); the deposit look
    # (vein / stain / nodule-in-peat) is COMPOSED BY THE RENDERER from host+ore
    # splat weights (PBR-1), never baked into a substance texture -- baking it
    # double-composites and forecloses grade. So these are PURE substances:
    # native gold and malachite are ore materials the mix will host; bog iron
    # keeps its slug but is regenerated as the pure limonite/goethite nodule
    # substance; banded ironstone and rock salt are whole-voxel ROCKs whose
    # internal structure is legitimate sub-voxel texture (ores.md R8).
    ("native-gold",      "native gold",     (0.85, 0.66, 0.22),19300,  1.0,   1.00, [1.4, 6.0, 4.4, 5.0], "material", True),
    ("bog-iron",         "bog iron",        (0.52, 0.34, 0.16), 2800,  0.1,   0.18, [2.0, 3.5, 2.5, 2.0], "material", False),
    ("banded-ironstone", "banded ironstone",(0.45, 0.16, 0.13), 3000,  0.5,   0.95, [5.5, 9.0, 5.0, 8.0], "material", False),
    ("malachite",        "malachite",       (0.11, 0.40, 0.21), 4000,  0.3,   0.80, [3.0, 5.5, 4.0, 4.5], "material", False),
    ("rock-salt",        "rock salt",       (0.90, 0.89, 0.85), 2170,  3.0,   0.70, [2.5, 4.0, 2.0, 3.0], "material", False),
    # block-only packs (no material twin) -- synthetic property sheets --------
    ("stone",           "stone",           (0.52, 0.52, 0.54), 2600,  2.0,   1.00, [5.0, 9.0, 5.0, 8.0], "block", False),
    ("dirt",            "dirt",            (0.42, 0.30, 0.19), 1300,  0.1,   0.35, [0.9, 3.5, 3.0, 1.8], "block", False),
    ("grass",           "grass",           (0.30, 0.62, 0.25), 1200,  0.5,   0.40, [0.5, 2.0, 2.0, 1.0], "block", False),
    ("wood",            "wood",            (0.44, 0.33, 0.17),  700,  1.0,   0.60, [1.0, 4.0, 3.0, 2.0], "block", False),
]

# Block id -> pack slug. Geology blocks alias the geology material packs;
# stone/dirt/grass/wood own their packs. Air is never rendered (no pack).
BLOCK_SHARES = {
    "stone": "stone",
    "dirt": "dirt",
    "grass": "grass",
    "wood": "wood",
    "mudstone": "mudstone",     # shares material pack
    "sandstone": "sandstone",   # shares material pack
    "granite": "granite",       # shares material pack
    "basalt": "basalt",         # shares material pack
    "coal": "coal",                                     # shares material pack
    "peat": "peat",                                     # shares material pack
    "carbonaceous-mudstone": "carbonaceous-mudstone",   # shares material pack
}

# --------------------------------------------------------------------------
# Pre-registry SUBSTANCE character (docs/design/ores.md R8; redone in
# journal/0048 after the 0045 deposit-portrait failure). Keyed by slug; a slug
# that is NOT a key takes the byte-identical legacy path, which keeps every
# existing pack unchanged.
#
# DOCTRINE (geology.md § Ore, DECIDED 2026-07-20): ore is a MATERIAL inside a
# HOST, expressed in the eighths/partial mix system; grade IS the eighths
# count; the composite deposit look (vein / stain / nodule-in-peat) EMERGES
# from the renderer compositing host+ore per fragment from splat weights
# (PBR-1) -- it must NEVER be baked into a substance texture. The 0045 pass
# baked host-rock-plus-flecks "deposit portraits" (gold-quartz, redbed-copper,
# a mottled bog-iron matrix); that double-composites and forecloses grade, so
# those were replaced by PURE substances (native-gold rides the metal path,
# malachite the MOTTLE below, bog-iron the generic granular path -- no longer
# an ORE_STYLE key). banded-ironstone (a whole-voxel banded ROCK) and rock-salt
# (a base evaporite substance) are legitimately internal-structure-in-texture
# (the carbonaceous-mudstone precedent, ores.md R8) and are UNCHANGED.
#
# Directive layers a SUBSTANCE texture may carry (host+ore composition is the
# renderer's job, never these):
#   MOTTLE -- clustered two-tone blotching from a low-freq tiling mask
#             (soft-thresholded so it stays seamless): botryoidal malachite,
#             halite facet clumps. `strength` caps the blend.
#   BANDS  -- horizontal stratified color bands (a whole-voxel rock's own
#             sub-voxel structure, the iconic BIF stripe). Band index is
#             periodic over the tile AND phase-centered so the vertical wrap
#             edge falls INSIDE a band -> tiles seamlessly.
#   SSS    -- overrides specular B (porosity 0-64) with a LabPBR subsurface
#             value (65-255), so halite reads faintly translucent.
#   FLECK  -- sparse colored specks; retained (with _FLECK_SEED_XOR, which the
#             metal path also uses) for a future substance, used by no pack now.
FLECK = "fleck"
MOTTLE = "mottle"
BANDS = "bands"
SSS = "sss"

# Fleck picks reuse the metal-fleck seed constant so basecolor and specular
# agree on which texels are flecked.
_FLECK_SEED_XOR = 0xA24BAED4963EE407

ORE_STYLE = {
    # malachite (Cu-carbonate SUBSTANCE): deep green body with a lighter-green
    # botryoidal/concentric-banding HINT -- clustered rounded lighter clumps at
    # 16 px. Satin, NOT metallic (F0 stays dielectric); the green is the
    # substance's own albedo, not a host stain baked into a deposit portrait.
    "malachite": {
        MOTTLE: {"rgb": (0.34, 0.66, 0.42), "threshold": 0.45,
                 "strength": 0.70, "freq": 4, "seed_xor": 0x5A1AC17E00000003},
    },
    # banded iron formation: alternating hematite-red / steel-grey / pale chert.
    "banded-ironstone": {
        BANDS: {
            "colors": [
                (0.30, 0.31, 0.34),  # band 0 (spans the wrap): dark steel-grey
                (0.52, 0.17, 0.13),  # band 1: hematite red
                (0.61, 0.54, 0.47),  # band 2: pale chert
                (0.48, 0.16, 0.14),  # band 3: hematite red (2nd couplet)
            ],
            "nbands": 4,
            "phase": 2,
        },
    },
    # halite: off-white crystalline with faint cool facet clumps + subsurface.
    "rock-salt": {
        SSS: 190,
        MOTTLE: {"rgb": (0.80, 0.85, 0.92), "threshold": 0.55,
                 "strength": 0.35, "freq": 4, "seed_xor": 0x5A1700000000000A},
    },
}

# Grain-size normalization bounds (mm), log scale (clay .002 .. scree 100).
GRAIN_MIN = 0.002
GRAIN_MAX = 100.0

# --------------------------------------------------------------------------
# Deterministic value noise
# --------------------------------------------------------------------------


def slug_seed(slug: str) -> int:
    """Stable 64-bit seed from the slug (sha256 -- process-independent)."""
    d = hashlib.sha256(slug.encode("utf-8")).digest()
    return int.from_bytes(d[:8], "big")


def _vhash(seed: int, ix: int, iy: int) -> float:
    """Deterministic hash of a lattice point -> float in [0,1)."""
    n = (ix * 374761393 + iy * 668265263 + (seed & 0xFFFFFFFF) * 2246822519) & 0xFFFFFFFF
    n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
    n = (n ^ (n >> 16)) & 0xFFFFFFFF
    return (n & 0xFFFFFF) / float(0xFFFFFF)


def _smooth(t: float) -> float:
    return t * t * (3.0 - 2.0 * t)


def _lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def value_noise(seed: int, x: float, y: float, freq: int) -> float:
    """Bilinearly interpolated value noise that TILES seamlessly over SIZE.

    `freq` is the number of lattice cells across the texture; cell coordinates
    wrap modulo `freq`, so opposite edges match.
    """
    gx = x * freq / SIZE
    gy = y * freq / SIZE
    x0 = math.floor(gx)
    y0 = math.floor(gy)
    fx = gx - x0
    fy = gy - y0
    x0m, y0m = x0 % freq, y0 % freq
    x1m, y1m = (x0 + 1) % freq, (y0 + 1) % freq
    v00 = _vhash(seed, x0m, y0m)
    v10 = _vhash(seed, x1m, y0m)
    v01 = _vhash(seed, x0m, y1m)
    v11 = _vhash(seed, x1m, y1m)
    sx, sy = _smooth(fx), _smooth(fy)
    return _lerp(_lerp(v00, v10, sx), _lerp(v01, v11, sx), sy)


# --------------------------------------------------------------------------
# Property -> generation parameters
# --------------------------------------------------------------------------


def clamp01(v: float) -> float:
    return 0.0 if v < 0.0 else (1.0 if v > 1.0 else v)


def derive_params(row) -> dict:
    slug, name, base_rgb, density, grain, cohesion, resist, kind, metal = row
    seed = slug_seed(slug)

    # grain size -> feature frequency (fine = high freq fine speckle,
    # coarse = low freq chunky blobs). freqs divide SIZE for seamless tiling.
    #
    # journal/0020: the ladder floor was raised from 2 to 4. At one 16×16 tile
    # per 0.9 m voxel a freq-2 low-frequency blob reads as a strong directional
    # "houndstooth" the moment the tile repeats block-to-block. A higher floor
    # (plus the fractal high-freq octaves in build_height) keeps the tile reading
    # as isotropic grain, so per-voxel repetition is far less legible.
    g_norm = clamp01(
        (math.log10(grain) - math.log10(GRAIN_MIN))
        / (math.log10(GRAIN_MAX) - math.log10(GRAIN_MIN))
    )
    freqs = [16, 8, 8, 4]
    freq = freqs[min(3, int(round(g_norm * 3)))]

    # hardness = mean of the four extraction resistances (sieve excluded --
    # sieve == grain size by construction). Normalized against the registry's
    # observed span (~0.8 leaf-litter .. ~7.1 granite/basalt).
    hardness = sum(resist) / len(resist)
    h_norm = clamp01((hardness - 0.8) / (7.2 - 0.8))
    # harder -> deeper normal relief. Range softened (journal/0020): the old
    # (0.8, 2.6) span cast strong per-texel shadows that, tiled block-to-block
    # under the lit path, sharpened the directional houndstooth. Gentler relief
    # keeps a pixel-game surface without a legible repeat.
    relief_amp = _lerp(0.5, 1.5, h_norm)

    # cohesion + fineness -> perceptual smoothness (specular R)
    smoothness = clamp01(0.14 + 0.46 * cohesion + 0.20 * (1.0 - g_norm))

    # density -> porosity (low-density loose material reads porous; snow high,
    # dense rock low, metal ~0). LabPBR porosity occupies B in [0,64].
    porosity = clamp01(1.0 - density / 3200.0)
    porosity_b = int(round(porosity * 64.0))
    # Pre-registry ore override: halite substitutes a LabPBR subsurface value
    # (65-255) for the porosity byte, so the specular B channel authored into
    # the PNG and the value the manifest reports stay in agreement.
    _style = ORE_STYLE.get(slug)
    spec_b = porosity_b
    if _style is not None and SSS in _style:
        spec_b = _style[SSS]

    # F0 / metal (specular G). Dielectric F0 ~0.04 -> ~10/255; metal uses the
    # LabPBR predefined-metal id 231 ("gold").
    f0_g = 231 if metal else 10

    # emission (specular A). LabPBR: 255 == no emission; 0..254 == strength.
    # Only the gold-dust PLACER gets a slight glint. Native gold as a raw
    # SUBSTANCE carries NO glint (ores.md R5 / no-glint doctrine, journal/0048):
    # distance behaviour is the sparse-eighths dither's job, not an emissive
    # texture -- so a close-range gold cell reads as metal without glinting.
    emission_a = 8 if (metal and slug != "native-gold") else 255

    # basecolor ramp length: coarser material -> more shades to show chunk
    # variation. 4 (fine) / 5 (mid) / 6 (coarse).
    levels = 4 + min(2, int(g_norm * 3))

    return {
        "slug": slug,
        "name": name,
        "kind": kind,
        "metal": metal,
        "base_rgb": list(base_rgb),
        "seed": seed,
        "grain_mm": grain,
        "grain_norm": round(g_norm, 4),
        "noise_freq": freq,
        "hardness": round(hardness, 4),
        "relief_amp": round(relief_amp, 4),
        "smoothness": round(smoothness, 4),
        "porosity": round(porosity, 4),
        "spec_R_smoothness": int(round(smoothness * 255)),
        "spec_G_f0_metal": f0_g,
        "spec_B_porosity": spec_b,
        "spec_A_emission": emission_a,
        "ramp_levels": levels,
    }


# --------------------------------------------------------------------------
# Texture synthesis
# --------------------------------------------------------------------------


def build_height(p: dict):
    """Continuous height field H[y][x] in [0,1], three octaves, seamless.

    journal/0020: reweighted toward the higher-frequency octaves so the tile
    reads as isotropic grain rather than a single low-frequency directional
    blob. The material's `noise_freq` octave (grain character) still leads, but
    fixed freq-8 and freq-16 octaves now carry more of the field, dissolving the
    per-block "houndstooth" the low-freq blob produced when tiled. All octave
    frequencies wrap modulo their cell count, so the field stays seamless.
    """
    seed = p["seed"]
    freq = p["noise_freq"]
    mid_seed = seed ^ 0x9E3779B97F4A7C15
    fine_seed = seed ^ 0xD1B54A32D192ED03
    H = [[0.0] * SIZE for _ in range(SIZE)]
    for y in range(SIZE):
        for x in range(SIZE):
            lo = value_noise(seed, x, y, freq)
            mid = value_noise(mid_seed, x, y, 8)
            fine = value_noise(fine_seed, x, y, 16)
            H[y][x] = clamp01(0.40 * lo + 0.24 * mid + 0.36 * fine)
    return H


def _ore_basecolor(style: dict, p: dict, x: int, y: int, factor: float,
                   cr: float, cg: float, cb: float):
    """Layer pre-registry ore character (bands -> mottle -> flecks) onto the
    host ramp color. Only called for slugs present in ORE_STYLE, so existing
    packs never reach this and stay byte-identical."""
    if BANDS in style:
        bs = style[BANDS]
        n = bs["nbands"]
        bi = int(math.floor((y + bs["phase"]) * n / SIZE)) % n
        br, bg, bb = bs["colors"][bi]
        # keep the height ramp as within-band grain
        cr, cg, cb = br * factor, bg * factor, bb * factor
    if MOTTLE in style:
        ms = style[MOTTLE]
        mask = value_noise(p["seed"] ^ ms["seed_xor"], x, y, ms["freq"])
        thr = ms["threshold"]
        m = clamp01((mask - thr) / (1.0 - thr)) * ms["strength"]
        mr, mg, mb = ms["rgb"]
        cr = _lerp(cr, mr * factor, m)
        cg = _lerp(cg, mg * factor, m)
        cb = _lerp(cb, mb * factor, m)
    if FLECK in style:
        fs = style[FLECK]
        if _vhash(p["seed"] ^ _FLECK_SEED_XOR, x, y) > fs["threshold"]:
            cr, cg, cb = fs["rgb"]
    return cr, cg, cb


def gen_basecolor(p: dict, H) -> bytes:
    """Albedo RGB (quantized pixel ramp around the palette color) + opaque A."""
    r, g, b = p["base_rgb"]
    levels = p["ramp_levels"]
    # journal/0020: the albedo shade spread was narrowed (0.17 -> 0.11) so the
    # per-texel value steps are gentler. The strong old contrast made the tiled
    # low-frequency pattern read as a bold repeating grid; a tighter ramp keeps
    # the pixel-game quantization while softening the legibility of the repeat.
    spread = 0.11
    metal = p["metal"]
    flake_seed = p["seed"] ^ 0xA24BAED4963EE407
    style = ORE_STYLE.get(p["slug"])
    out = bytearray(SIZE * SIZE * 4)
    i = 0
    for y in range(SIZE):
        for x in range(SIZE):
            hq = round(H[y][x] * (levels - 1)) / (levels - 1)  # quantized shade
            factor = 1.0 - spread + 2.0 * spread * hq
            cr, cg, cb = r * factor, g * factor, b * factor
            if metal:
                # sparse bright gold flecks so it reads as placer dust
                if _vhash(flake_seed, x, y) > 0.86:
                    cr, cg, cb = 0.98, 0.84, 0.42
            if style is not None:
                cr, cg, cb = _ore_basecolor(style, p, x, y, factor, cr, cg, cb)
            out[i] = int(round(clamp01(cr) * 255))
            out[i + 1] = int(round(clamp01(cg) * 255))
            out[i + 2] = int(round(clamp01(cb) * 255))
            out[i + 3] = 255  # opaque placeholder
            i += 4
    return bytes(out)


def gen_normal(p: dict, H) -> bytes:
    """Tangent normal XY (R,G) from the height gradient, AO (B), height (A)."""
    amp = p["relief_amp"]
    out = bytearray(SIZE * SIZE * 4)
    i = 0
    for y in range(SIZE):
        for x in range(SIZE):
            # central differences with wrap (matches the tiling height field)
            hl = H[y][(x - 1) % SIZE]
            hr = H[y][(x + 1) % SIZE]
            hd = H[(y - 1) % SIZE][x]
            hu = H[(y + 1) % SIZE][x]
            nx = -(hr - hl) * amp
            ny = -(hu - hd) * amp
            nz = 1.0
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny, nz = nx * inv, ny * inv, nz * inv
            # AO from height: low spots are more occluded. Contrast softened
            # (journal/0020) so baked occlusion doesn't re-draw the low-freq
            # blob as a dark repeating grid across tiled blocks.
            ao = 0.72 + 0.28 * H[y][x]
            out[i] = int(round((nx * 0.5 + 0.5) * 255))
            out[i + 1] = int(round((ny * 0.5 + 0.5) * 255))
            out[i + 2] = int(round(clamp01(ao) * 255))
            out[i + 3] = int(round(H[y][x] * 255))  # height for parallax
            i += 4
    return bytes(out)


def gen_specular(p: dict, H) -> bytes:
    """Smoothness (R), F0/metal (G), porosity/SSS (B), emission (A)."""
    base_r = p["spec_R_smoothness"]
    gch = p["spec_G_f0_metal"]
    bch = p["spec_B_porosity"]
    ach = p["spec_A_emission"]
    metal = p["metal"]
    dith_seed = p["seed"] ^ 0x27D4EB2F165667C5
    style = ORE_STYLE.get(p["slug"])
    # pre-registry ore: a fleck reads slightly glossier where spec_boost > 0
    # (gold-quartz); the halite SSS already rides in bch via derive_params.
    fleck = style.get(FLECK) if style is not None else None
    out = bytearray(SIZE * SIZE * 4)
    i = 0
    for y in range(SIZE):
        for x in range(SIZE):
            # tiny smoothness dither so the surface isn't dead-flat spec
            d = int(round((_vhash(dith_seed, x, y) - 0.5) * 6))
            r = max(0, min(255, base_r + d))
            g = gch
            a = ach
            if metal:
                # flecks read as brighter metal (crystalline facets). The
                # gold-dust PLACER also glints (a=4); native gold as a raw
                # substance does NOT (journal/0048 no-glint doctrine).
                if _vhash(p["seed"] ^ 0xA24BAED4963EE407, x, y) > 0.86:
                    r = min(255, base_r + 40)
                    if p["slug"] != "native-gold":
                        a = 4
            if fleck is not None and fleck.get("spec_boost"):
                if _vhash(p["seed"] ^ _FLECK_SEED_XOR, x, y) > fleck["threshold"]:
                    r = min(255, base_r + fleck["spec_boost"])
            out[i] = r
            out[i + 1] = g
            out[i + 2] = bch
            out[i + 3] = a
            i += 4
    return bytes(out)


# --------------------------------------------------------------------------
# Minimal PNG encoder (dependency-free, deterministic) + reader for self-check
# --------------------------------------------------------------------------


def write_png(path: str, w: int, h: int, rgba: bytes) -> None:
    assert len(rgba) == w * h * 4
    stride = w * 4
    raw = bytearray()
    for y in range(h):
        raw.append(0)  # filter type 0 (None)
        raw.extend(rgba[y * stride:(y + 1) * stride])
    comp = zlib.compress(bytes(raw), 9)

    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)  # 8-bit RGBA
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", comp)
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as f:
        f.write(png)


def _read_png_dims_channels(path: str):
    """Return (width, height, channels) reading the IHDR. Uses Pillow if
    available, else parses the header directly."""
    try:
        from PIL import Image  # type: ignore

        with Image.open(path) as im:
            ch = len(im.getbands())
            return im.width, im.height, ch
    except Exception:
        with open(path, "rb") as f:
            data = f.read(33)
        assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
        w, h = struct.unpack(">II", data[16:24])
        color_type = data[25]
        ch = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
        return w, h, ch


# --------------------------------------------------------------------------
# Driver
# --------------------------------------------------------------------------


def generate() -> dict:
    os.makedirs(OUT_DIR, exist_ok=True)
    packs = {}
    for row in MATERIALS:
        p = derive_params(row)
        slug = p["slug"]
        d = os.path.join(OUT_DIR, slug)
        os.makedirs(d, exist_ok=True)
        H = build_height(p)
        write_png(os.path.join(d, "basecolor.png"), SIZE, SIZE, gen_basecolor(p, H))
        write_png(os.path.join(d, "normal.png"), SIZE, SIZE, gen_normal(p, H))
        write_png(os.path.join(d, "specular.png"), SIZE, SIZE, gen_specular(p, H))
        packs[slug] = p
    write_manifest(packs)
    return packs


def write_manifest(packs: dict) -> None:
    materials = {}
    for slug in sorted(packs):
        p = packs[slug]
        materials[slug] = {
            "name": p["name"],
            "kind": p["kind"],
            "files": {
                "basecolor": f"{slug}/basecolor.png",
                "normal": f"{slug}/normal.png",
                "specular": f"{slug}/specular.png",
            },
            "base_color_srgb": p["base_rgb"],
            "seed": p["seed"],
            "params": {
                "grain_mm": p["grain_mm"],
                "grain_norm": p["grain_norm"],
                "noise_freq": p["noise_freq"],
                "hardness": p["hardness"],
                "relief_amp": p["relief_amp"],
                "ramp_levels": p["ramp_levels"],
                "smoothness": p["smoothness"],
                "porosity": p["porosity"],
                "metal": p["metal"],
            },
            "specular_channels": {
                "R_smoothness": p["spec_R_smoothness"],
                "G_f0_or_metal": p["spec_G_f0_metal"],
                "B_porosity": p["spec_B_porosity"],
                "A_emission": p["spec_A_emission"],
            },
        }

    block_map = {}
    for block, slug in sorted(BLOCK_SHARES.items()):
        block_map[block] = {
            "pack": slug,
            "shared_with_material": slug in {
                "mudstone", "sandstone", "granite", "basalt",
            },
        }

    manifest = {
        "pack_format_version": PACK_FORMAT_VERSION,
        "placeholder": True,
        "note": (
            "Procedurally generated placeholder LabPBR packs (visuals.md "
            "mixture road, 2026-07-19). Assets-in-waiting for the splat-blend "
            "milestone. NOT art direction -- regenerate via "
            "tools/gen_placeholder_textures.py."
        ),
        "resolution": [SIZE, SIZE],
        "channel_packing": {
            "basecolor": "R,G,B = albedo; A = opacity",
            "normal": "R,G = tangent normal X,Y (reconstruct Z); B = AO; A = height",
            "specular": (
                "R = perceptual smoothness; G = F0 (0-229) / metal id (230-255); "
                "B = porosity(0-64)/SSS(65-255); A = emission (255 = none)"
            ),
        },
        "generation": {
            "seed_source": "sha256(slug)[:8] big-endian (process-independent)",
            "grain_bounds_mm": [GRAIN_MIN, GRAIN_MAX],
            "freq_ladder": [16, 8, 4, 2],
            "derivations": {
                "noise_freq": "from grain size (fine=high freq, coarse=low)",
                "relief_amp": "from hardness = mean(dig,chop,smash,cut resist)",
                "smoothness": "from cohesion + fineness",
                "porosity": "from density (loose=high, rock=low, metal~0)",
                "ramp_levels": "from grain size (4 fine / 5 mid / 6 coarse)",
            },
        },
        "material_count": len([p for p in packs.values() if p["kind"] == "material"]),
        "block_only_count": len([p for p in packs.values() if p["kind"] == "block"]),
        "materials": materials,
        "block_to_pack": block_map,
    }
    with open(os.path.join(OUT_DIR, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2, sort_keys=False)
        f.write("\n")


def _read_png_pixels(path: str) -> list[list[tuple[int, int, int, int]]]:
    """Decode one of OUR PNGs (8-bit RGBA, filter 0 on every row — exactly
    what write_png emits). Not a general decoder."""
    with open(path, "rb") as f:
        data = f.read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, w, h, idat = 8, None, None, b""
    while pos < len(data):
        ln = struct.unpack(">I", data[pos:pos + 4])[0]
        tag = data[pos + 4:pos + 8]
        if tag == b"IHDR":
            w, h = struct.unpack(">II", data[pos + 8:pos + 16])
        elif tag == b"IDAT":
            idat += data[pos + 8:pos + 8 + ln]
        pos += 12 + ln
    raw = zlib.decompress(idat)
    stride = w * 4
    px = []
    for y in range(h):
        row = raw[y * (stride + 1):(y + 1) * (stride + 1)]
        assert row[0] == 0, "self-decoder only handles filter 0"
        px.append([tuple(row[1 + x * 4:5 + x * 4]) for x in range(w)])
    return px


def _seam_check(path: str) -> None:
    """Assert the texture tiles: luminance gradients across the wrap edges
    must be statistically indistinguishable from internal gradients. Guards
    the lattice-wrap property against future edits (checked 2026-07-19)."""
    px = _read_png_pixels(path)
    h, w = len(px), len(px[0])

    def lum(p):
        return 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]

    internal, wrap = [], []
    for y in range(h):
        for x in range(w - 1):
            internal.append(abs(lum(px[y][x]) - lum(px[y][x + 1])))
        wrap.append(abs(lum(px[y][w - 1]) - lum(px[y][0])))
    for x in range(w):
        for y in range(h - 1):
            internal.append(abs(lum(px[y][x]) - lum(px[y + 1][x])))
        wrap.append(abs(lum(px[h - 1][x]) - lum(px[0][x])))
    mi = sum(internal) / len(internal)
    mw = sum(wrap) / len(wrap)
    # 1.6x tolerance: measured worst honest ratio is 1.33x (gravel); a real
    # non-wrapping seam lands at 2-4x.
    assert mw <= mi * 1.6 + 1.0, (
        f"{path}: wrap-edge gradient {mw:.1f} vs internal {mi:.1f} — "
        f"texture no longer tiles seamlessly"
    )


def self_check(packs: dict | None = None) -> None:
    """Read a sample of PNGs back and assert 16x16 RGBA + seamless tiling."""
    if packs is None:
        slugs = [row[0] for row in MATERIALS]
    else:
        slugs = list(packs)
    # sample: first, middle, last, plus gold-dust (the metal special case),
    # snow, and every pre-registry ore (their bands/flecks/mottle are the
    # riskiest new tiling paths, so the shipped self-check must cover them).
    sample = {slugs[0], slugs[len(slugs) // 2], slugs[-1], "gold-dust", "snow"}
    sample |= {s for s in ORE_STYLE if s in slugs}
    # the journal/0048 redone substances: native-gold rides the metal path and
    # bog-iron the generic granular path, so neither is an ORE_STYLE key -- name
    # them explicitly so the shipped self-check seam-tests all three.
    sample |= {s for s in ("native-gold", "bog-iron") if s in slugs}
    checked = 0
    for slug in sorted(sample):
        d = os.path.join(OUT_DIR, slug)
        for tex in ("basecolor.png", "normal.png", "specular.png"):
            path = os.path.join(d, tex)
            w, h, ch = _read_png_dims_channels(path)
            assert (w, h, ch) == (SIZE, SIZE, 4), f"{path}: got {(w, h, ch)}"
            if tex != "specular.png":  # flat channels; seam test is meaningless
                _seam_check(path)
            checked += 1
    print(f"self-check OK: {checked} PNGs verified as {SIZE}x{SIZE} RGBA, "
          f"basecolor/normal seam-checked (sampled {len(sample)} packs)")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true",
                    help="verify existing PNGs only (no generation)")
    args = ap.parse_args()
    if args.check:
        self_check()
        return
    packs = generate()
    total = len(packs) * 3
    print(f"generated {len(packs)} packs ({total} PNGs) into {OUT_DIR}")
    self_check(packs)


if __name__ == "__main__":
    main()
