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

**Provider slots (2026-07-22, journal/0060).** Some stubs now have a *mechanism*
and not merely an entry. A **provider slot** is a plain `fn` pointer in
`deeptime::providers::Providers` — resolved once at world build, carried in
`DeepConfig`, default-identity — that names the question, names its heir, and
holds the constant as an explicitly registered *identity* rather than as the
rule. Four exist (stubs 7b, 8 and 13, and the layer-cake sibling gap). A slot
does not retire a stub; it stops the stub from silently becoming the definition,
which is ARCHITECTURE.md § *A summary is not an authority* made structural. The
general registry is deliberately unbuilt — four conversions is not enough to
design one from.

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

### 2. surface-veneer-block-rule — **RETIRED WHERE A RECORD EXISTS, 2026-07-21 (journal/0055)**
*Was:* `collapse.rs::surface_sample`: Grass/Dirt/Stone from year-zero climate,
hard thresholds (`precip < 0.10`, lapse-rate `t < −4 °C`), no slope, no record
consult. The cause of the razor-straight grass/dirt frontier.

*Now:* **the record decides what the world is skinned with** (materials.md
§ Sequencing AMENDED 2026-07-21, user: *"we don't have to have this problematic
of deciding which material to skin the world with when the record already
says"*). `surface_sample` — still the kernel **shared** by the near ground and
the far horizon, so both inherit it structurally — reads the content class
holding the most metres in the recorded column's topmost 0.9 m, and the surface
voxel's block is `classify(contents)` of the top-of-column remainder expressed as
partial fill. **Grass is not expressed at all** (ratification 4). World-wide, at
a 907-voxel stride over 12 135 land samples: Grass 81.8 % / Dirt 18.2 %
**→** Mudstone 91.4 % / CarbonaceousMudstone 6.3 % / Coal 2.0 % / Peat 0.3 % /
Granite 0.02 %. Far-field cost 22.1 → 22.9 µs/sample.

*Surviving fallbacks — absence of a record, not stubs:* the **border wilds** (no
deep-time run exists out there — § Genesis), **subaqueous columns** (`clastic_pass`
returns early below sea level), and columns with neither a record nor a tectonic
province. Those keep the year-zero rule **minus its Grass branch**, so the
vocabulary is Dirt/Stone. Columns whose record rounds to nothing read as their
**basement** — journal/0053's barest land now skins as Granite, not painted Dirt.

*Residual:* 91.4 % of land skinning to one block is the ledger's honest answer
(the topmost deposition in most subaerial cells is low-energy hillslope creep →
clastic-fine) but it is monotonous, and *material* diversity under it is much
higher than the *block* vocabulary can summarize. Whether the block tier should
grow, or the recorder's energy banding should, is a live question and belongs to
the ecology/forms presentation pass. **Heir for the vegetation half:** unchanged
— the ecology system.

**SHARPENED 2026-07-21 (user, forms pass):** the veneer's *grass* half now has
an explicit disposition — **grass is suspended and must not be expressed at
all**. It is an ecology state riding on the substrate materials of a block or
its loose eighths, not a block identity chosen by a threshold. The forms slices
express the materials the sim says are present and paint no grass; the user
pre-ratified the appearance ("it'll be a mostly brown world for a bit"). So the
retirement of this stub is now *partly a deletion* rather than wholly a
replacement: the Grass branch goes away with ecology rather than being
reimplemented by it. Turf's eventual presentation is sketched (ligatures +
anisotropic variants — green on top faces, roots-texture additive on sides,
over a seeded dirt material in loose eighths or packed pores) and explicitly
**not to be built now**. Non-grasslike ground cover: suspended entirely.
See materials.md § "The forms design pass".

### 3. soil-depth-from-precip / the discarded `H` plane — **RETIRED 2026-07-21 (journal/0053)**
*Was:* soil depth = 3/2/1 voxels by present-day precip (`collapse.rs::column`)
and a clastic veneer budget of `1.0 + precip*2.5` (`clastic_pass`), while the
deep sim's regolith plane `H` was summed away by `field.rs` (`surf = r + h`).

*Now:* `DeepField` carries `regolith` (the `H` plane, +2.27 MB at Medium,
+1.5 % of the field). **Both** consumers read it — the legacy soil band and the
clastic veneer budget — so the heir landed whole; neither half remains. Rule:
round `H` to the nearest whole voxel, clamp `0..=8`; **0 is reachable**, which
is what the old floor of 1 made impossible. Two findings from the slice worth
carrying forward:

- **`Σ(recorded unit thicknesses) ≡ H` exactly.** The strata record *is* the
  regolith column, decomposed. So the veneer's honest budget is the part of the
  column whole voxels cannot resolve (`round(H/0.9)` minus what
  `deposit_deep_history` expressed), amalgamated as one surficial body —
  bioturbation, physically. The fill is now mass-conserving against the ledger.
- **The border wilds keep the precip rule**, explicitly scoped to genesis (there
  is no deep-time run out there to consult). That is the § Genesis case, not a
  surviving stub.

*Residual:* the 8-voxel veneer cap truncates the ledger on 14.3 % of land. That
was **stub 12**, below — a different defect, uncovered by this one, and **retired
the same day by journal/0055**, which also drove the veneer's own thickness
budget to zero at every named site without deleting it.

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

### 7b. s10-waterlogging-proxy — **added 2026-07-22 (journal/0061); now a provider slot**
*Numbered `7b` because it is the hydrological half of the same S10 shortfall as
stub 7 — biology asking a question no authority existed to answer — and
`water.md` already carried its retirement clause. It was nonetheless **unlisted
here**, which by this file's own doctrine is a defect in the inventory.*

`deeptime/biotic.rs::step_cell`: waterlogging — *is the water table at the
surface here?* — was three magic numbers summed inline: climate moisture, plus
`((80 - surf)/80).clamp(0,1) * 0.20` for sitting near base level, plus
`(area/300).min(1) * 0.15` for receiving upslope drainage. None of the five
coefficients is measured, and the sum answers in a **dimensionless 0..1 index**
where the question is asked in **metres below the surface**. It is the widest
stand-in in the deep sim by consumer count: **four separate thresholds** read it
— the peat-former waterlog suitability gate, the decomposition drain factor
(hence peat accumulation), the fire dryness term, and the `peat_site` test that
picks a column's depositional-hiatus cap — and through organic facies it reaches
`geology::deep_class` and the world's surface material.

**Heir named by ratified decision, not merely proposed:** `water.md` DECIDED
2026-07-20 (*one quantity, two regimes*), consequence 4 — *"S10's waterlogging
proxy has a defined retirement: waterlogging becomes 'the water table is at or
near the surface here', read from the field. Biology reads the real quantity;
the proxy is deleted."* The field is S11's saturation over the drainage-pinned
lattice; the table is the top of the saturated zone, a query rather than a
stored plane.

**Now a slot:** `providers::Providers::depth_to_water`, **pass-level, once per
epoch**, materialized at `BioticSim::step` — a water table follows the surface,
so unlike `parent_p` it cannot be built once for the run, and unlike
`wave_energy` it cannot be a per-cell call, because the heir is a *field* solved
over a neighbourhood. Identity `identity_depth_to_water` leaves the plane
**empty**, which routes `providers::wet_at` to `identity_wet_index` — the
three-term proxy, verbatim. That empty-plane + identity-accessor pair is the
`bio_weather` / `frost` shape the four deep-sim flags already prove, and it is
what this seam alone had no form of. **Blast:** every coal seam, peat bed,
paleosol and charcoal lamina in the record, and the surface material above them.

**Not retired by this slice.** The four thresholds are untouched and still read a
0..1 index; converting them to metres is a behaviour change. The units mismatch
is the seam's most useful output — see journal/0061 § *what the heir must
supply*.

### 8. uniform-parent-material-phosphorus — **NOW A PROVIDER SLOT, 2026-07-22 (journal/0060)**
`deeptime/biotic.rs` (initial P pool): every cell starts with the same P —
granite and basalt pretended equal. Documented only in S10-results (§ design
choice 12); no live code marker. **Heir:** P keyed on parent-material petrology
(provenance/lithology). **Blast:** retrogression/paleosol geography (currently
player-invisible behind stub 2).

*Change:* the constant is no longer the rule. `providers::Providers::parent_p`
is the slot; `identity_parent_p` (uniform `1.0`) is its registered identity, and
`biotic::P_ROCK_INIT` survives only as the value that identity returns. The slot
is **pass-level**: `BioticSim::new` materializes a per-cell plane from it once,
and *both* consumers — the initial pool and the rejuvenation cap that a stripped
surface restores toward — read the plane rather than the constant. The second of
those was a live second-order defect: "fresh rock" was capped globally, so even a
hypothetical P-poor parent material would have been rejuvenated to the P-rich
maximum. The code marker this entry noted as missing now exists, in the slot's
doc comment.

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

### 11. placer-presence-is-source-blind — *added 2026-07-21 by the ores design pass*
`geology.rs` placer pass: gold-dust *presence* in a river is a function of
discharge + settle energy only — every big-enough river carries some. The
energy zonation (where in the gravel it concentrates) is real S8 mechanism;
the presence term is a field-blind constant. **Heir:** condition
presence/abundance on the upstream catchment's lode endowment (drainage
export × the R1 lode field — ores.md R2-B). **Blast:** rivers don't
differentiate; the "read the river, walk upstream" chain dead-ends.

### 12. the sub-voxel bed sieve — **RETIRED 2026-07-21 (journal/0055)**
*Was:* `geology.rs::deposit_deep_history` rounded each recorded deep unit to
whole voxels and dropped anything under half a voxel — `Σ round(tᵢ/0.9)` where
honesty requires `round(Σ tᵢ/0.9)`. Measured on the production Medium world:
mean `H` per subaerial cell **4.75 m**, of which only **1.15 m** survived —
**75.8 % of the recorded sediment pile deleted**, and **48.1 % of land cells
expressed nothing at all**. The dune-field station recorded 379 units summing to
7.99 m and expressed **zero**.

*Now:* the record's metres survive to the voxel boundary and are quantized
**once**, at contents construction: a voxel's eight eighths are filled from the
units overlapping its own 0.9 m span, by **addressed stochastic rounding**
(`crate::fill`). Sieve loss is **−0.2 % net** with a mean absolute error of
0.227 m/cell — the ≤ half-voxel rounding of the column's own total, signed, not
a one-way deletion. The dune field expresses **9 voxel spans, all nine mixed**;
the loess margin expresses **89 voxels** of section where the 8-voxel veneer cap
used to truncate it.

Two things retired with it, without a line of surgery:
- **the veneer's thickness budget** (stub 3's heir) — `round(H/0.9) − expressed`
  is now `0.00` voxels at every one of journal/0053's seven named sites, because
  the record expresses all of `H` on its own;
- **the 8-voxel cap's 14.3 % truncation** — the record is not capped, so the
  ledger's mass reaches the ground wherever it is thick.

The heir this file previously named — pre-merging adjacent sub-voxel units
*inside the record* — is **superseded**: it patched the symptom by
amalgamating unlike beds. `deposit_deep_history` does still run-length coalesce
adjacent units that resolve to the *same member*, which is lossless for
expression (identical member ⇒ identical contents) and bounds the per-column
event vector; that is a cost control, not a stub.

*Residual:* a mixed voxel loses the **internal order** of what it mixes, so a
contact renders as journal/0010's speckle rather than as a banded contact. That
is **ratified as non-blocking** (materials.md, user: *"it's lost order
information in presentation but it's far more honest than it was before and
presentation can be reconsidered later"*) and belongs to the forms presentation
work, not here.

### 13. one-wave-climate-for-the-whole-planet — **added 2026-07-22 (journal/0060); now a provider slot**
`deeptime/erosion.rs::Erosion::wave`: the littoral agent cut every shore cell at
the single global constant `DeepConfig::wave_erosion` (0.05 m/epoch at the
waterline), tapered by freeboard and scaled by the rock's wave-resistance axis —
but with **no dependence on where the shore is**. A lee shore inside a 40 km
inland sea and a west-facing ocean coast at 45° S receive identical attack. Wave
height is set by **fetch** (open water upwind) and **wind**, and the sim has both
in reach, so this was the model's most conspicuous "one number where a field
belongs". Unlisted before this entry — which is exactly the defect the doctrine
names.

**Now a slot:** `providers::Providers::wave_energy`, identity
`identity_wave_energy` (hand the configured constant straight back). **Heir:**
fetch from the S11 body graph × the zonal wind field the eolian agent already
reads (ROADMAP). **Blast:** coastline morphology — where cliffs retreat, where
platforms cut, and the marine reworking recorded in every coastal column. Note
the pre-existing off-switch is preserved and is *not* the seam: `wave_erosion <=
0.0` still short-circuits the whole agent before any provider is consulted, which
is what `tests/full_agents.rs`'s byte-identity control depends on.

### 14. coal-rank-is-burial-depth-with-no-geotherm — **added 2026-07-22 (journal/0063), replacing a worse stub on the wrong axis; now a provider slot (journal/0067)**
`deeptime/biotic.rs::COAL_BURIAL_M` (8.0 m) and `COAL_ONSET_C`, applied by
`recorder.rs::DeepStrata::promote_coal` through
`providers::Providers::burial_temp_c`.

*Was:* buried peat became coal when **the seam was thick enough**
(`thickness_m >= 0.4`). That is not a stub with a bad constant, it is a stub on
the **wrong axis** — a statement about how long a swamp lasted standing in for a
statement about what happened to it afterwards — and it contradicted
`CLASS_ORGANIC_COAL`'s own contract (*"the class's depth axis is the rank
axis"*). The authority it was summarizing was already in the record: burial depth
is `Σ` thickness of the overlying units, one pass.

*Now:* the control is each unit's own overburden. Measured consequence
(production Medium): coal-bearing deep cells **12 892 → 2 216**, units
**19 008 → 3 888**, recorded coal **22 459 → 3 773 m**, thickest seam
**15.74 → 14.01 m**. Nothing gains coal — thickness and burial are close to
*anti*-correlated here, because a thick peat is one that sat at a quiet,
low-aggradation surface.

*What remains a stub, and it is a narrower one:* the **number**, and the
dimension it is missing. Coalification is a pressure/temperature path, and this
project has **no geotherm** — surface air temperature is the only temperature
anywhere in the sim. So this is burial *depth*, and it cannot discriminate coal
**rank**. Nor can it use Earth's calibration: the peat→lignite transition wants
10²–10³ m, and of 35 382 peat-derived units in the production world exactly 13
lie under 50 m of section and one under 100 m, so an honest Earth threshold
yields a world with no coal at all. 8.0 m is the ~90th percentile of *this*
record's burial distribution — "coal is what happens to the peat that got buried
deepest". **Heir:** a geotherm, which turns this into a P/T path and lets the
single `Coal` facies split into the lignite/sub-bituminous/bituminous/anthracite
members `CLASS_ORGANIC_COAL`'s depth-is-rank contract has been waiting for.
**Blast:** the abundance and depth of every coal seam a player can dig.

**Now a slot (2026-07-22, journal/0067):** `providers::Providers::burial_temp_c`
— *"what temperature has this buried unit seen?"* — asked once per candidate
unit at `BioticSim::finalize`, and compared against `COAL_ONSET_C`. Identity
`identity_burial_temp_c` is a **degenerate geotherm**: 0 °C at the surface, a
gradient of exactly 1 °C/m, so the answer is numerically the overburden in
metres and the test is bit-for-bit the shipped `overburden_m >= 8.0`. The
gradient is 40× Earth's and is written down as arithmetic rather than dressed up
as physics — the same deliberate units mismatch `depth_to_water` carries.

The **calibration did not change and is not under review** (user, 2026-07-22:
*"we knew coal/charcoal would depend on eco + geotherm and need to accept just
buildin the seams in for now… the calibration is fine"*). What changed is that
landing the heir is now **supplying a provider** rather than rewriting
`promote_coal`.

*Why a temperature slot and not an `is_coalified` predicate:* coal rank and
metamorphic grade are one thermal-maturity ladder (peat → lignite → … →
anthracite → greenschist), so a temperature composes with `stubs.md` § 4 and
gives the built-but-unconsumed `exhum`/`t_crust` planes (`spines.md` § 3) their
first named consumer path. A predicate would answer one rung and compose with
none. Full argument in `providers/burial_temp_c.rs`'s module docs.

*Two obligations that ride with it.* (1) `COAL_ONSET_C` and the identity
geotherm are **one calibration in two places and retire together** — a real
0.025 °C/m gradient against an unchanged onset of 8.0 turns the whole record to
coal. (2) The payload's `overburden_m` is depth below the **present** surface;
the record keeps no memory of section deposited and later stripped, so an
exhumed unit reads as shallow where real coal rank is irreversible. Neither is
fixed here.

## Sibling gap (not a substitution — an unexpressed ledger term)

- **Layer-cake strata / no dip-fold.** Tectonic history is recorded; structural
  deformation of the record (earth-processes § 7) is unbuilt except the
  `unconformity` flag. Every stratum lies horizontal regardless of history.
  Same family as stub 4: the ledger holds what the runtime does not yet say.

  **A socket exists for it as of 2026-07-22 (journal/0060).** The single function
  the deformation term has to replace — "which units lie in the near-surface
  window here" — is now `providers::Providers::outcrop_at`, whose identity
  `identity_outcrop_at` is `lithology::exposed_litho` — the lithology dominating
  the record's topmost `OUTCROP_DOMINANCE_WINDOW_M` (0.9 m), a thickness rule as
  of journal/0068, not merely the top unit. Erosion asks the slot,
  not the function, at all four sites that consult exposed lithology (fluvial
  incision + creep, periglacial frost, eolian deflation, littoral attack). This
  does not close the gap — nothing dips yet — but it converts `lithology.rs`'s
  prose promise ("only the one function changes") into a compile-checked seam.

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
