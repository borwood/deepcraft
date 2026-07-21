# 0048 — Substances, not deposit portraits

*2026-07-21. Asset-only slice, correcting journal/0045. No registry, no
placement, no rust — three placeholder packs redrawn in
`gen_placeholder_textures.py`, nothing else in the world moved.*

The 0045 ore pass shipped packs that looked, individually, convincing: a milky
quartz `gold-quartz` with warm gold flecks; a red-brown `redbed-copper` with
green malachite specks; a `bog-iron` with an oranger nodule matrix mottled into
the brown. Each was a little **portrait of a deposit** — host rock plus the ore
mineral, composited together inside one 16×16 texture.

That is the wrong object, and the reason is the whole point of the ratified
representation.

## The mechanism: composition belongs to the renderer

Geology.md § Ore (DECIDED 2026-07-20) fixes what an ore *is* in this engine:
not a deposit, but a **MATERIAL inside a HOST**, carried in the existing
eighths/partial mix system — the olivine precedent (journal/0011). Two
consequences follow that a baked-fleck texture violates head-on:

1. **The renderer already composites host and ore**, per fragment, from splat
   weights (PBR-1). If the ore's own texture *also* contains host rock with ore
   flecks in it, the host is drawn twice and blended against itself — a
   double-composite. The vein/stain/nodule look is supposed to *emerge* from
   `host_weight · host_tex + ore_weight · ore_tex`; you cannot pre-bake the sum
   into one of the summands.
2. **Grade IS the eighths count** — there is no separate grade mechanic. A
   1/8-rich cell and an 8/8 bonanza cell differ only in how many partials of the
   ore substance the mix carries. A texture that bakes in a *fixed* fleck
   density has already chosen a grade, and every cell that hosts it inherits
   that one density regardless of its actual eighths. The baked portrait
   **forecloses grade** — the exact axis the mix exists to vary.

So the fleck density, the green-on-red, the quartz-on-gold: none of that is the
substance's business. It is what the *world* composes when it puts N eighths of
the ore into a host of that rock. The texture's only job is to be the pure
substance, so the mix has an honest thing to blend. (`docs/design/ores.md` § R8
records the same finding from the naming side: the members were named as
deposits — `dc:geo/gold-quartz` — when the ratified representation is a
substance in a host.)

`banded-ironstone` and `rock-salt` were left untouched, and the distinction is
real: those are whole-voxel **rocks**, not ores-in-a-host. Their internal
structure — BIF's mm-scale banding, halite's crystalline facets — is genuine
sub-voxel reality that lives legitimately in-texture (the carbonaceous-mudstone
precedent). A banded ironstone voxel is banded ironstone all the way through;
nothing composites into it. The tell is whether the look is the substance's own
body (in-texture) or the *arrangement* of two substances (renderer's job).

## What was replaced

Same byte-identity discipline as 0045: every PNG not being replaced was sha256'd
before and after and is identical — all 28 untouched packs, now including
`banded-ironstone` and `rock-salt` from the previous pass. The manifest still
lists 30 materials (two renamed, one regenerated); the full 68-PNG seam sweep
and the shipped self-check (now explicitly sampling all three redone packs) are
green.

- **`gold-quartz` → `native-gold`** — pure metallic native gold. Warm saturated
  yellow, brighter crystalline speckle for dendritic facets, LabPBR metal F0
  (id 231) in specular because it *is* metal. Crucially **no emission**: the
  glint the placer `gold-dust` carries is the placer's; a raw gold substance
  reads as metal at close range and lets the sparse-eighths dither govern
  distance behaviour (the no-glint doctrine). Subtlety comes from partial
  *sparsity* in the mix, not from muting the metal.
- **`redbed-copper` → `malachite`** — pure malachite: deep green body with a
  lighter-green botryoidal/concentric-banding hint (rounded clumps at 16 px),
  satin — *not* metallic — specular. The green is the substance's own albedo,
  not a stain painted onto a red host.
- **`bog-iron`** (slug kept) — regenerated in place as the pure nodule
  substance: dense limonite/goethite, rusty ochre-brown, earthy-dull specular,
  fine granular variation only. The 0045 matrix/blotch composite is gone; the
  nodule-in-peat look is for the mix to make when it hosts these partials in a
  Peat host.

The generator changes are all gated on the new slugs (or the metal flag plus a
`native-gold` guard), so `gold-dust` and every other pre-existing pack stayed
byte-for-byte identical — the placer's glint path in particular is untouched.

> blogworthy: "the deposit portrait" — how a texture that looks *more* realistic
> in isolation is the wrong asset once composition is the renderer's job, and how
> grade-is-eighths makes any baked-in richness a lie the mix can't take back.
