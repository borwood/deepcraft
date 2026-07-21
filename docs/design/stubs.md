# The stub inventory

**Doctrine** (geology.md § Expression of the ledger + § Genesis addendum,
DECIDED 2026-07-21, user): *a stub is a candidate to be subsumed into a spine,
system, or process — the expresser.* Stubs are permanently legitimate in
exactly one place: **genesis**. Everywhere else a stub must be known, loud,
and listed here with its heir. **An unlisted stub is a defect in this
inventory, not a licence.**

Each entry names: what the rule fakes · the expresser that subsumes it ·
loudness at audit time · blast radius. First audit 2026-07-21 (read-only
sweep, background agent; integrator-curated). Update this file in the same
commit as any change that adds, removes, or subsumes a stub.

---

## Active stubs

### 1. ruin-posts — **was UNDOCUMENTED until this audit**
`collapse.rs::ruin_posts` (~997), rendered ~400.
An abandoned pregen site expresses as ≤10 procedurally-scattered wood posts,
2–4 voxels tall, at hashed angle/radius. The *abandonment fact* is genuine
committed history; the *structure* is a rule-of-thumb. **Heir (user, 2026-07-21):** the social sim +
ecology — dwarf-fortress-class civilization history; "that's the only way a
post gets there." Culture-related artifacts are placeholder as a class: do not
bandaid, do not think about, until ecology is done and the social sim gets its
design pass (the big one: NPCs and all — priors exist in the corpus). **Blast:**
the only world-visible form of settlement history. *Loud code marker added
2026-07-21.*

### 2. surface-veneer-block-rule
`collapse.rs::surface_sample` (~516): Grass/Dirt/Stone from year-zero climate,
hard thresholds (`precip < 0.10`, lapse-rate `t < −4 °C`), no slope, no record
consult. The cause of the razor-straight grass/dirt frontier. **Heir:** the
ecology system — vegetation-as-deep-time, placement by fitness (ecology.md
DECIDED 2026-07-21). **DO NOT BANDAID.** **Blast:** every surface block in the
world, out to the far horizon (`coarse_surface` shares the rule).

### 3. soil-depth-from-precip / the discarded `H` plane
`collapse.rs::column` (~902): soil depth = 3/2/1 voxels by present-day precip,
while the deep sim's computed regolith plane `H` is summed away
(`field.rs`: `surf = r + h`, `h` discarded). The clastic veneer thickness
budget (`clastic_pass`, `base = 1.0 + precip*2.5`) is the same derivation.
**Heir:** carry `H` into `DeepField`; read soil/regolith depth from the
recorded cause. Named OPEN work in the Expression decision. **Blast:** depth
of diggable soil everywhere; the Dirt band in ocean/wilds columns.

### 4. exhum / t_crust — the absent metamorphic expresser
`field.rs` (~164): shipped, documented in-code as "the metamorphic-grade axes
the collapse tier reads" — **and nothing reads them** (the comment overstates;
noted here so it isn't trusted). **Heir:** a metamorphism pass reading the
P/T path into grade classes (schist/slate/gneiss roster, geology.md).
**Blast:** zero today; lands the day a cut face should show an aureole and
shows plain basement. *Only populated in tectonic-history worlds (U8 gates
the payoff).*

### 5. igneous-emplacement-depth-constants — *omitted from the decision's holdout list; added by this audit*
`geology.rs` (~237): `INTRUSIVE_DEPTH_M = 250`, extrusive `5.0`,
`INTRUSIVE_TOP_VOX = 96` — one number where a per-column burial story belongs.
**Heir:** the paleo-context provider (igneous fitness consumes tectonic
setting + real emplacement depth; geology.md § formation context).
**Blast:** intrusive/extrusive member choice + basement thickness in every
Orogeny/Arc/Rift column.

### 6. paleo-temperature-is-present-day-latitude — *omitted from the holdout list; added by this audit*
`geology.rs::deposit_deep_history` (~379): a deep unit's at-deposition
temperature = today's column temperature (the aridity axis correctly reads the
recorder's tag; temperature does not). **Heir:** a paleo-temperature curve in
the deep record (the "later 3e slice"). **Blast:** temp-sensitive member
fitness inside deep strata.

### 7. s10-community-vector — biology with no vegetation
`deeptime/biotic.rs`: coal/peat/paleosol/charcoal from a community scalar;
fire burns a number, not a forest. **Heir:** ecology-pass vegetation members
with proliferation patterns, consumed by the same mechanisms (ecology.md Note
2026-07-21). **Blast:** the geography of all organic facies.

### 8. uniform-parent-material-phosphorus
`deeptime/biotic.rs` (initial P pool): every cell starts with the same P —
granite and basalt pretended equal. Documented only in S10-results (§ design
choice 12); no live code marker. **Heir:** P keyed on provenance. **Blast:**
retrogression/paleosol geography (currently player-invisible behind stub 2).

### 9. paleo-sea-level-sinusoid
`deeptime/grid.rs::sea_level_at`: one deterministic sinusoid (amp 35 m,
period 50) stands in for epoch-indexed sea-level/climate history; drives every
unit's subsea/subaerial tag. **Heir:** epoch-indexed pregen curves
(earth-processes § 6). **Blast:** the marine/terrestrial split of the entire
deep record — large, but *inside* the ledger (it is history the runtime then
faithfully expresses).

### 10. frontier-pressure closed-form
`dc-sim/statistical/world.rs::prior_pressure_weights` (~280): frontier
boundary conditions from a closed-form of danger, self-described stand-in.
**Heir:** the statistical tier's real cached summaries (S2). **Blast:** low,
indirect (edge-region history synthesis).

## Sibling gap (not a substitution — an unexpressed ledger term)

- **Layer-cake strata / no dip-fold.** Tectonic history is recorded; structural
  deformation of the record (earth-processes § 7) is unbuilt except the
  `unconformity` flag. Every stratum lies horizontal regardless of history.
  Same family as stub 4: the ledger holds what the runtime does not yet say.

## Genesis (permanently legitimate — affirmed, not defects)

Level-0 lattice corners (`corner0`) · border-wilds cell synthesis · initial
bedrock + plate seeding (`tectonics.rs`, `build_cells`) · `SEA_LEVEL_M = 0`
datum · pregen `Provenance` assignment.

## Audited and rejected (real mechanisms or ratified decisions, not stubs)

Legacy S1 terrain keys 3/4 (ratified dev affordance; journal/0017 verified
contained) · dormant erosion-agent axes ("not a placeholder" by construction)
· erodibility/placer/inclusion calibration knobs (tuning of real mechanisms)
· `DEEP_MAX_WIDTH` Large-extent coarsening (coarsen-the-cause, sanctioned) ·
`record_at_voxel` nearest-cell sampling (flagged, legitimate) · 12 fps stepped
animation + parametric crouch (ratified aesthetic/firewall) · placeholder
texture packs + vertex-color albedo (rendering-asset placeholders — heir is
authored art, outside this doctrine's scope; visuals.md owns the path).
