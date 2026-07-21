# 0045 — Textures ahead of the registry

*2026-07-21. Asset-only slice. The order: "I just want the placeholder pbrs
made for the ores. Get them made. They're placeholders. Try for realism,
follow the process for priors to make seamlessly tileable. Not solving
placement / the upstream engineering for this yet, just getting textures
ready."*

The five new v1 ore materials proposed in `docs/design/ores.md` (DRAFT) needed
placeholder LabPBR packs, the same procedural 16×16 three-texture sets
`gen_placeholder_textures.py` has produced for every material since PBR-1.
Nothing else: no `MaterialId`, no atlas layer, no registry row. Those are the
deliberately-deferred upstream engineering, and this slice does not touch them.

## Where do textures for materials that don't exist go?

The obvious worry was that landing packs for unregistered materials would trip
something. It does not, and the reason is worth writing down. The atlas builder
(`dc-client/src/terrain_material.rs`) assembles its layers by walking the
**registry** — `MaterialId::all()` plus `BLOCK_ONLY_SLUGS` — and for each it
reads `assets/textures/placeholder-labpbr/<slug>/`. It never enumerates the
directory. A slug the registry does not know is therefore never looked up: the
five new pack dirs are inert on disk until the day the material is registered,
at which point the loader finds them already built. No self-check scans the
tree (that is the shader-pack loader, `shaderpack.rs`, a different thing
entirely), and nothing at runtime reads the generator's `manifest.json`.

So the packs land beside their siblings — the least-surprise home, exactly
where the wiring day will expect them — rather than in a staging pen. They are
pre-staged in place.

## Keeping the other 29 packs byte-identical

The hard constraint was that regenerating must not perturb a single existing
byte. The existing derivation reads a 9-field property row; the ores need
character that derivation cannot express (flecks, nodular mottle, the banded-
iron stripe, halite translucency). Rather than widen the row format and risk
touching every material, the new behaviour hangs off a slug-keyed `ORE_STYLE`
table: a slug absent from it takes the untouched legacy path. Since no existing
slug is a key, every one of the 29 old packs regenerates bit-for-bit — verified
by hashing all 87 PNGs before and after (87/87 identical; only the 15 new PNGs
appear and `manifest.json` grows its material count 25 → 30).

The one genuinely new tiling problem was banded-ironstone. Isotropic value
noise tiles for free; horizontal colour bands do not — a band boundary landing
on the vertical wrap edge reads as a seam. The fix is to phase the band index
so the wrap falls *inside* a band, not on a boundary: `floor((y+phase)·n/16) mod
n` with `phase` at half a band width puts the same band on rows 15 and 0, so the
wrap-edge gradient is within-band noise. The seam self-check (extended to sample
all five ores) passes; a full sweep of all 68 basecolor/normal PNGs across 34
packs reports zero seam failures.

## The five looks

- **gold-quartz** — milky grey-white vein quartz with sparse warm gold flecks;
  the flecks read a touch glossier in specular, no emission (not the placer).
- **bog-iron** — rusty brown limonite with clustered oranger nodule blotches
  from a low-frequency mottle mask.
- **banded-ironstone** — the iconic BIF stripe: alternating hematite-red,
  pale-chert, and steel-grey horizontal bands, tiling seamlessly top-to-bottom.
- **redbed-copper** — red-brown clastic host with sparse malachite-green
  specks, doctrine-subtle (ores.md R5 option a: green cells only, never a
  glint).
- **rock-salt** — off-white crystalline with faint cool facet mottle; the
  specular B channel carries a LabPBR subsurface value (190) so halite reads
  faintly translucent rather than porous.

`gold-dust` already ships a pack (the placer material, since journal/0007) —
verified present with all three textures and left untouched.

## Files

`tools/gen_placeholder_textures.py` (5 rows + `ORE_STYLE` table + the
fleck/mottle/band/SSS paths, all gated on the new slugs) and the 15 new PNGs
under `assets/textures/placeholder-labpbr/{gold-quartz,bog-iron,
banded-ironstone,redbed-copper,rock-salt}/`, plus the regenerated (byte-stable)
`manifest.json`.
