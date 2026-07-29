# Sim light — what the voxels know

Status: **DESIGN PASS, 2026-07-20.** Produced at the user's instruction
("start the sim light design pass"), following the ratified shape: design
pass → spike → milestone. Decisions carry their date and are marked
DECIDED; everything else is explicitly open or proposed.

**The one-sentence thesis.** Sim light is a deterministic, derived,
per-voxel account of *how much light reaches here* — Minecraft-blocklight
grade, "the simulation level of our light, **not crisp dynamic shadows
etc, just what the voxels know**" (user, 2026-07-20). It is **not** the
renderer's lighting, is not derived from it, and does not feed it.

---

## 1. Priors in the corpus (swept before writing — do not re-derive)

- **S3 wrote the skylight query contract already** (S3-results.md
  § Skylight query contract; `column.rs` module docs). It solves the
  cubic-chunk problem — a cube cannot know what is above it — without
  vertical scans: a light query consults **only** resident chunks and
  cached column summaries, never loads/generates/walks the column above;
  every query is bounded to a caller-supplied `[y_min, y_max)`; **unknown
  volume is non-occluding ("optimistic sky") and poisons a
  `fully_resolved` flag**; answers carry the resolution they were derived
  at. Column summaries build lazily at ~20 µs — effectively free.
  **S3 OQ 6 explicitly defers `re-light-on-load` to this work.**
- **S4 anticipated a colored-light store**: LabPBR specular packs
  **emission in channel A** (0–254 meaningful, 255 reserved) and
  "emission color comes from albedo — which is exactly what the future
  colored-light store wants". The shader-pack uniform block also already
  carries a **time-of-day slot**, currently a fixed app-set constant
  (0.35) "sized for that future, not this present".
- **materials.md**: fluids occupy empty eighths and pores; per-voxel
  porosity is a real simulated field. Opacity therefore has a natural
  home as a material property, and partial fill has a natural meaning.
- **visuals.md § Mood** (2026-07-18, and DECIDED again 2026-07-20): real
  darkness underground, no floaty ambient minimum; warmth is earned.
  Reference is **modded Minecraft with shaders**.
- **API.md has no light surface at all** — the character sense surface
  (pose / raycast / surroundings) will need a light sense added; it does
  not exist today.
- **corrections #13**: an earlier "user wants no darkness" record was a
  misattribution arising from confusing the *fullbright diagnostic* with
  the *game's lighting*. Keep the registers distinct in this document.

## 2. The model — DECIDED 2026-07-20 (user)

1. **Derived, never stored.** Light attenuates, so reach has a hard bound
   *by construction* (reach = emission ÷ attenuation), unlike water whose
   locality had to be measured. Derive per chunk within that halo; do not
   persist. Also dodges ~32 KB/chunk of light bytes.
   - **Bonus argument**: a stored model must *un-propagate* when a source
     is removed — the classic stuck-light bug class in MC-likes. A derived
     model has **no removal path at all**: re-derive the halo, it is
     correct.
2. **Sky light and block light are separate channels.** Skylight
   propagation is time-invariant (it stores *how much sky reaches here*);
   time of day scales it at query. One channel would force re-propagating
   every torch at dusk.
3. **Emission and opacity are material properties**, not special cases.
   Lava glows because its material says so; bioluminescent fungi because
   their organism def says so; water attenuates less than stone because
   its sheet says so. No light-source-specific code — the placer pattern.
4. **Reach is per-source, not a global constant.** Emission strength is a
   material property; attenuation is per-material-per-step; reach is where
   the level falls below threshold. **Torch quality is therefore a
   material question** — crafting gains real stakes.
5. **No light field in deep time.** At deep-time scale surface is lit and
   subsurface is dark; the biotic layer treats it as binary.
   Bioluminescence exists in caves but is not a source for photosynthesis
   or heat, so it does not change this.
6. **Darkness gates spawning; light is equipment.** Darkness is a gameplay
   material with a fuel cost; how dangerous it is, is content-pack
   tunable.
7. **Caves get their own ecology**, falling out of biome-as-diagnosis once
   light is an axis — "no light, stable temperature, wet, fed from
   outside" is a set of conditions like any other. No cave-specific
   ecology system.
8. **Sim light is monochrome for now, but band-shaped** (see § 6).

## 3. Heavenly bodies — the light primitive

DECIDED 2026-07-20 (user). Skylight is **directional**, and the source of
direction is a table of heavenly bodies.

- **A body carries**: a **path**, a colour, an intensity, and its own
  rise/set. **Sun and moon are the first two rows, not special cases.**
  A mod adding moons on other axes adds rows.
- **Day and night are EMERGENT from paths.** There is no separate
  day/night-cycle system, and no separate ephemeris — *the paths are the
  ephemeris*.
- **Paths are parameterised, not fixed**:
  `direction = path(body, world_time, observer_latitude)`, with a **long
  cycle over which the path itself changes**.
  - The long cycle is the **seasons** seam.
  - The latitude term is the **poles and tropics** seam — they fall out
    rather than being authored.
  - **BUILD NOW: only the path capability.** Seasons and latitude effects
    are explicitly not planned or built (user). The requirement is that
    the abstraction can express them without a rewrite — the same
    discipline that made karst possible in the erodibility milestone.
  - `lat_deg` already exists on pregen cells (`collapse.rs::climate_at`),
    so observer latitude is a cheap query today.
- **World time is a deterministic tick counter**, never a wall clock
  (project law).

## 4. Two terms, two mechanisms

The user's decomposition, and the key to affordability:

> we do a cheap sky bounce too: if a body is above the horizon, it
> contributes a small downward heavenly light (maybe adjusted for cloud
> cover) in addition to its directional.

- **DIRECT is a QUERY, not a field.** "Is this voxel lit by body N right
  now?" is a *visibility question along a known direction* — answered on
  demand against the S3 skylight contract (bounded, summary-consulting,
  optimistic-sky, resolution-tagged). Nothing stores it.
  - **Consequence: direct light needs no direction quantisation at all.**
    Quantisation is only forced if a directional field is stored. As a
    query it uses the body's exact current direction.
  - This is what makes *sun-into-the-cave-mouth*, *through-the-window*,
    *under-the-overhang* affordable — the thing MC conspicuously cannot do.
- **BOUNCE is the FIELD.** Direction-free, small, downward-diffuse while a
  body is above the horizon; propagates like MC skylight. This is the
  per-voxel derived quantity. Cloud cover is a future **weather** input —
  leave the hook, not the feature (weather is unbuilt).
- **A subtle inherited gotcha**: MC gets "open sky ⇒ full brightness at
  ground" by propagating skylight straight down undiminished. With real
  body directions that convenience disappears. **Decide deliberately what
  full daylight on open ground means** rather than discovering an
  accidental constant at implementation time.

## 5. Propagation and determinism

- Light is a **max-plus relaxation**:
  `light(v) = max( emission(v), max over neighbours n of (light(n) − attenuation(v)) )`
- This is **monotone with a unique fixpoint**, so run-to-convergence is
  **order-independent by construction**. That is the requirement, not a
  test outcome: an ordered flood-fill queue would diverge under replay and
  under the parallel path. Same family as S9b (scatter→gather) and S11's
  saturation.
- Attenuation is per-material-per-step, and should read **partial fill**
  (loose materials at partial height, water in pores/partials) rather than
  a binary solid/air test — consistent with the materials model.
- **Structural note**: bound-water saturation (S11), sim light, and
  plausibly heat are all *the same computational shape* — a bounded local
  relaxation over the voxel grid attenuated by a per-material property,
  with sources and sinks. S11 measured its locality at a 4–11 cell halo.
  **Check whether one machinery serves all three before building two of
  them.**
  - **✅ ANSWERED FOR TWO OF THE THREE — 2026-07-29 (reconciliation, not a new decision).**
    **Saturation and temperature already share one machinery**: they are
    **condition-fields** — named per-cell quantities with opaque ids, produced by field
    passes and read by cellular passes and formation predicates — **DECIDED 2026-07-24**
    (`docs/design/material-behavior.md` § 14), and `docs/design/flow.md` § *What survives*
    records it as settled: *"head, saturation, temperature are one shape; the geotherm
    proved it."* The vocabulary is explicitly **extensible**, and there are **no
    capability tiers** on field passes.
  - **What is still genuinely open is LIGHT specifically** — whether the condition-field /
    field-pass shape extends to a **max-plus** relaxation (light is not a diffusive or
    Laplacian solve, and § 2 item 1 rules it **derived, never stored**, which is the
    opposite storage posture from a `DeepField` plane). That is the live question; do not
    re-derive the saturation/temperature half.

## 6. Colour, and creatures that see bands we do not — OPEN, recommendation recorded

The user, on monochrome:

> the ACTUALLY COOL sim thing light color could give us: a creature (or
> character!) that can, say, SEE ULTRAVIOLET etc. do we kill it or is this
> actually deep esp if we go evolution route for bio?

**Recommendation: do not build it, do not foreclose it.**

- With the direct/bounce split, **colour rides the source, not the
  propagated field** — a body has a colour, an emitting material has a
  colour. So direct light is already coloured for free while the expensive
  per-voxel term stays monochrome.
- Model **emission and sensitivity as band vectors**, with exactly one
  band in v1. "Sees ultraviolet" then becomes a *data relationship*
  between an organism's sensitivity vector and a source's spectrum — the
  roles-as-contracts pattern — rather than new machinery.
- Cost today ≈ a one-element array instead of a scalar. Payoff: UV /
  infrared vision, and its arrival **via the evolution route as a lineage
  trait**, never needs a rewrite.
- This is the light-model twin of the erodibility/limestone trap: one
  scalar would make a whole category impossible later.

## 7. Consumers, and their very different cadences

| consumer | needs | cadence |
|---|---|---|
| Plant growth / photosynthesis | a light **budget over time** | bulk, slow — probably a per-chunk pass, not point queries |
| Spawning | level at a point | near-instantaneous, frequent |
| Stealth / NPC vision | level at a point | on demand |
| Embodied agent perception (MCP) | level at the character | on demand, via the character sense surface |

- Because light is derived, **cost is per-query** — so the *query load*
  matters more than storage. A plant pass wants bulk evaluation over a
  chunk, not a million point queries.
- **Perception parity is a design principle** (visuals.md): an MCP-driven
  character must not see better than a human player merely because it
  reads a different channel. The dev/debug surface keeps full reach, as it
  already does for character control.
- **Block light has an item dependency** (torches are placeable objects;
  items are unbuilt) — **unblocked by a dev-light block** (user): a
  point-light emitter that never spawns naturally, so propagation can be
  built and tested before items exist. Plant light budgets deferred
  ("we can hash out plants later").

## 8. Water and light — a free unlock

If opacity is a material property and water is a material occupying pores
and partials, then **light attenuates with depth in water automatically**.
That yields the **photic zone** with no new machinery, and with it depth
zonation for aquatic ecology — plants near the surface, nothing below.
Falls out of decisions already made; worth stating so nobody builds it
twice.

## 9. Open questions (for the spike, or for the user)

1. **Re-light-on-load policy** — S3's deferred decision. Optimistic sky is
   a policy, not a truth: a consumer ignoring `fully_resolved` will light
   caves as if open to sky until data arrives. What re-derivation happens
   when the data does arrive?
2. **Resolution**: per voxel (0.9 m at N=2) or coarser? Per-voxel is the
   MC analogue; coarser loses torch-scale detail.
3. **Level quantisation and the cap**: how many levels, and is the cap a
   knob? (Knob doctrine says any range is a knob.)
4. **What full daylight on open ground means** once skylight is directional
   (§ 4).
5. **Coarse / LOD light** for distant regions — S3 mentions "far-light
   decisions"; if sim ever runs coarsely far away, light needs a coarse
   form. Vista-as-augury territory.
6. **Derivation halo size and cache invalidation shape** — the dirty rail
   already serves remeshing, collider tiles and (owed) far-field
   summaries; light joins it.
7. **Does the renderer ever consult sim light?** DECIDED that they are
   separate and neither is derived from the other — recorded here so the
   temptation is answered in advance.

## 10. What the spike must measure

1. **Derive-per-chunk cost** at a realistic halo, and the query cost for
   each consumer cadence in § 7 (point query vs bulk chunk pass).
2. **Order-independence by construction** — byte-identical results under
   shuffled update order, double-run, and scalar↔parallel (the S9b/S11
   method). This is a proof, not a benchmark.
3. ~~**Whether one relaxation machinery serves light and bound water**~~ —
   **NARROWED 2026-07-29.** Bound water and temperature are already one
   machinery: **condition-fields, DECIDED 2026-07-24**
   (`material-behavior.md` § 14; `flow.md` § *What survives*). What this
   spike must measure is narrower — **whether the condition-field /
   field-pass shape extends to LIGHT**, whose relaxation is **max-plus**
   rather than diffusive and which § 2 item 1 rules **derived, never
   stored** — or an honest statement of why not. *A spike dispatched off
   the un-narrowed wording would re-derive a decided result.*
4. **The direct-light query cost** against the S3 contract, including the
   unresolved/optimistic-sky path.
5. **Halo/locality**: light's bound is analytic, so *verify* it rather than
   discover it — and report control drift alongside, per corrections #15.
