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
says"*). **AMENDED 2026-07-22 (journal/0074, the surface-branch removal):** the
near ground's surface voxel is the recorded column's **top span** through
`ColumnFill` (`plan(1)`, the same authority every buried voxel routes through);
its block is `classify(contents)` of the top-of-column remainder expressed as
partial fill. journal/0055 originally put this consult in `surface_sample` as a
kernel **shared** by the near ground and far horizon — but a shared kernel is a
summary that *became* the authority (spines § S-3), so the near path was routed
onto `ColumnFill` and `surface_sample` demoted to the far-field summary only
(`coarse_surface`), reading the deep record's top-window class and held to a
statistical agreement test against the ground. **Grass is not expressed at all**
(ratification 4). World-wide, at a 907-voxel stride over 12 135 land samples (the
journal/0055 far-summary distribution): Grass 81.8 % / Dirt 18.2 %
**→** Mudstone 91.4 % / CarbonaceousMudstone 6.3 % / Coal 2.0 % / Peat 0.3 % /
Granite 0.02 %.

*Surviving fallbacks — absence of a record, not stubs:* the **border wilds** (no
deep-time run exists out there — § Genesis), **subaqueous columns** (`clastic_pass`
returns early below sea level), and columns with neither a record nor a tectonic
province. Those keep the year-zero rule **minus its Grass branch**, so the
vocabulary is Dirt/Stone. Columns whose record rounds to nothing read as their
**basement** — journal/0053's barest land now skins as Granite, not painted Dirt.

**~~Absence-of-record is legitimate, but NO QUERY SURFACE CAN EXPRESS IT AT VOXEL
RESOLUTION~~ — DISCHARGED 2026-07-25 (journal/0101).** *(Original: the fallbacks
above are correct; what was missing was any way to* say so *to a caller.
`HostWorld::contents_at` answered a per-voxel question with a **per-chunk** presence
test, so its `Option::None` — the only channel that could mean "no record here" — was
already spent on a whole-32³-chunk condition inherited from the mesher's needs. An
unrecorded basement voxel therefore reported `has_contents: true` with
`classified: dc:air` **over solid stone**, 6.4 % of near-surface solid voxels,
corrections #49.)* **`dc_api::Identity::Unrecorded` is now that answer**, distinct in
the type from `Mixture(EMPTY)`; `HostWorld::identify(pos)` is the surface, and
`has_contents` / `classified` / `sense_raycast`'s `contents` / the F3 HUD all report
through it. `contents_at` survives as the raw chunk-granular **source**, documented as
such. **Residual (narrowed, not gone):** the answer is **edit-blind for composition** —
the source is a pure function of position, so an edit writes a `Block` and no mixture
moves under it. **Heir for the residual:** the `identify` arc's continuation slot — the
**runtime edit-fact overlay** (contents as derivable base + edit facts).

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
`field.rs`: shipped and populated since U8, **read by no downstream consumer**.
The in-code comment is honest about this — it says the collapse tier *WILL* read
these and that they are *currently consumed by nothing*, and as of journal/0078
cites `spines.md` § 3. (The 2026-07-22 audit called the comment an A-2
overstatement by quoting it with its "WILL" dropped; corrections #40 records that
the comment was already correct.) **Heir:** a metamorphism pass reading the
P/T path into grade classes (schist/slate/gneiss roster, geology.md) — with a
named arrival address, the `providers::burial_temp_c` geotherm (journal/0067).
**Blast:** zero today; lands the day a cut face should show an aureole and
shows plain basement. *Only populated in tectonic-history worlds (U8 gates
the payoff).* Note `t_crust` *is* read inside the sim by `isostasy()`; the
unconsumed thing is the exported plane, not the value.

### 5. igneous-emplacement-depth-constants — *omitted from the decision's holdout list; added by this audit*
`geology.rs` (~237): `INTRUSIVE_DEPTH_M = 250`, extrusive `5.0`,
`INTRUSIVE_TOP_VOX = 96` — one number where a per-column burial story belongs.
**Heir:** the paleo-context provider (igneous fitness consumes tectonic
setting + real emplacement depth; geology.md § formation context).
**Blast:** intrusive/extrusive member choice + basement thickness in every
Orogeny/Arc/Rift column.

### 6. paleo-temperature-is-present-day-latitude — **NOW A PROVIDER SLOT, 2026-07-23 (journal/0078)**
`geology.rs::deposit_deep_history`: a deep unit's at-deposition
temperature = today's column temperature (the aridity axis correctly reads the
recorder's tag; temperature does not — the asymmetry sits on adjacent lines).

**Now a slot:** `providers::Providers::paleo_temperature`, identity
`identity_paleo_temperature` (return the present-day `ctx.temp_c` — the *wrong
quantity*, named, not a neutral no-op). **Heir:** an epoch-indexed
paleo-temperature curve keyed by the unit's `chapter` (the tectonic epoch the
record already carries). **Value-level per unit** — the identity is constant per
column but the heir varies per epoch, and granularity follows the heir. **Blast:**
temp-sensitive member fitness inside deep strata. Byte-identical: the production
world still hashes to the pre-slice goldens. Surfaced that the collapse tier had
no `Providers` channel at all — now threaded through `WorldGenerator`/`StrataCtx`
(the mechanism previously reached only the deep-time sim via `DeepConfig`).

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

### 14. coal-rank-is-burial-depth-with-no-geotherm — **RETIRED 2026-07-24 (journal/0093, the geotherm field pass)**

*Retired:* the geotherm (the first §5 field pass, `deeptime/geotherm.rs`) supplies a real
`T(depth)` from a per-cell tectonic gradient; `promote_coal` reads it at the seam's mid-depth
against `COAL_ONSET_C` (recalibrated 8 → 22 °C). **The `burial_temp_c` provider slot is removed
entirely** — the honest finding is that a real `T(depth)` is a **field**, not a value a
stateless `fn(unit)` slot could hold, so it *left* the provider set rather than fitting a socket
it never belonged in. Residual, honestly: burial in the record is shallow, so the geotherm ≈
surface temp at seam depth and the tectonic gradient barely moves *coal* — the gradient's real
payoff is **metamorphism** (deep crust, where `exhum` = P and geotherm = T give the P/T grade).
Original entry preserved below for audit.

### 14 (original). coal-rank-is-burial-depth-with-no-geotherm — **added 2026-07-22 (journal/0063), replacing a worse stub on the wrong axis; now a provider slot (journal/0067)**
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

### 15. far-node-synthesis-paints-the-column-with-the-surface-block — *added 2026-07-22 (journal/0070, FF2b-minimal)*
`dc-worldgen/src/far.rs::synthesize_far_node`.

A synthesized far node (a node whose full-res children were never generated —
most of the far field, forever) fills each column solid up to the FF2a
floor-quantized surface, and every voxel of that fill carries the **surface**
block. Height and surface block come from `coarse_surface`, which since
journal/0074 is an explicit far-field **summary** (no longer the kernel the near
ground collapses from — the near ground now expresses the record's top span
through `ColumnFill`, and the summary is held to a statistical agreement test
against it). The **sub-surface uniformity is the stand-in**: the coarse summary
answers only "where is the surface and what is it made of", so a far mesa's
exposed flank renders as its cap material. FF2a's top sheet had the identical
appearance (step sides already wore the surface block), so this fakes nothing
FF2a didn't — but in node form it is now data that *claims* the column, which
is why it gets an entry. **Heir:** the far-field **summarization** half of the
surface-branch work (the near-authority half landed in journal/0074; this
far half is a separate sequenced slice — octree-substrate.md § 3 names this
node-synthesis seam as its legal home): a coarse strata summary per node, so
synthesized spans can carry the recorded column's real vertical material
sequence. The reduction path already does this
(reduced spans render `classify` of `MixtureDownsampleRule` mixtures), so the
two sides of the contract will converge on it. **Loudness:** the agreement
test compares *tops* only; a strata-aware agreement test lands with the heir.
**Blast:** far-field side-face colour beyond played regions; no sim or replay
surface.

### 16. bedrock-materialized-as-a-flat-structure-span — *added 2026-07-24 (the first-real-behavior weathering slice); down-and-dirty by explicit user direction*
`dc-worldgen/src/deeptime/inventory.rs` (the working-inventory build): the deep
record's units all derive as **`Loose`**, and bedrock (`R`) is a single scalar
`basement: MaterialId`. Weathering is `Structure→Loose`, so the working inventory
had **no Structure source to weather from**. For this slice a bedrock Structure span
is materialized at the base of the column — the basement material, flat, at a
provisional depth — purely so the edge has a source. **This is a stub on the
*emplacement* axis:** bedrock is not really one flat basement material; it is
plutons, sills, and province lithologies, unroofed at real depths. **Heir:** a
**genesis/emplacement pass** (the ratified "genesis passes model the honest genesis
of rocks", ROADMAP 2026-07-24) that puts bedrock into the inventory as Structure
with real per-column material identity and depth — at which point this stand-in is
**deleted, not reimplemented**. **Blast:** which material a weathering rind is made
of in the deep tier (the loose product inherits the bedrock's identity); invisible
until the weathering facts express through the collapse. *Loud code marker required
at the materialization site.*

### 17. weathering-runs-once-post-hoc — **DISCHARGED 2026-07-24 (journal/0094, Movement 3)**
*Heir landed.* The inventory weathering is now a **live per-epoch cellular pass inside the
deep-time loop** — `dc:deep/weather_inventory` on the runner (`deeptime/runner.rs`), driven by
`weather_inventory::weather_epoch`, weathering each subaerial cell's bedrock seam **every
epoch on that epoch's live terrain and accumulating** across the run
(`weather_bedrock_epoch` → `finalize_ledgers`). The post-hoc `field.rs::build_ledgers`
one-shot is **deleted**. The **span-index crux** — the record grows every epoch, so the
bedrock's numeric index (`units.len()`) shifts — is solved by keying the accumulator to a
**stable bedrock sentinel** (bedrock-only ledger, slot 0, record-growth-invariant), re-keyed
onto the final record only at loop end. `dt` is now **live** (`share ∝ dt`, the pass's phase
length — journal/0090 M3 deferral closed). Production-scale band at the argmax cell: **≥1
voxel** (`production_scale_saprolite_band_reaches_at_least_one_voxel`, the A-3 guard). The
**two-authorities split holds** (material-behavior.md §11): the pass READS `H` but WRITES ONLY
the ledger — it never touches `R`/`H`, so the height-tier `dc:deep/weather` pass is untouched.
The flag `weather_inventory` stays a **walk-gated appearance** flip (default off → byte-identical);
that is not a stub. *Note the loose PRODUCT still inherits stub #16's stand-in bedrock identity
(one flat granite basement) — #16 is NOT retired by this; the material a rind is made of still
awaits the genesis/emplacement heir.*

**RESIDUAL — where the other half went (spine-audit 2026-07-24).** #17's heir sentence asked for
**two** things: a per-epoch in-loop cellular pass (**landed**) *and* "riding / unifying with the
height-tier weathering that already accumulates `H`" (**deferred**). The "two-authorities split
holds" line above states an *implementation choice for this slice*, **not yet an invariant** — and
the gap is visible in code, not merely doctrinal: `DeepField::derive_regolith_at`
(`deeptime/field.rs:545`) derives `H` from an **empty** ledger
(`FactLedger::empty_with_bedrock`), so Movement 2a's "the inventory is the authority, `R`/`H` are
its materialized views" is currently true of the **record half only** — the derived view
structurally cannot see the 6.09 m of `Loose` this pass commits. The unification half is tracked
as **ROADMAP Movement 2 / material-behavior.md §11 continuation slot + §13.6**; nothing further is
owed to *this* stub, but do not read the discharge as "the two authorities are reconciled."

*Original entry (added 2026-07-24, the S18 walk finding, corrections #46/#47; preserved for audit):*
`dc-worldgen/src/deeptime/field.rs::build_ledgers` + `weather_inventory::weather_column(_,
chapters=1, _)`: the inventory weathering runs a **single chapter, after the deep-time run,
over the finished record, with final-state fields frozen**. Weathering is a *continuous*
process — the height-tier loop runs it every epoch (→ meters of `H`) — so a one-shot over the
end-state is a **snapshot, not the process**; its product is sub-voxel (0.04 m) and expresses
as nothing (corrections #46). **This is a seam in the WRONG PLACE:** bolted onto the end
rather than living where weathering runs (inside the deep-time loop). **Heir:** weathering as
a **per-epoch cellular pass inside the deep-time compile**, riding / unifying with the
height-tier weathering that already accumulates `H` (the R/H-unification continuation slot,
material-behavior.md §11) — carrying material identity + per-agent `cause` on top of the
process the height loop already runs. **Blast:** the whole "first real behavior" claim —
until the heir lands, no weathering *behavior* is modeled; what exists is the keystone
**plumbing** (inventory + facts + fold), proven and valuable, with a one-shot demo standing on
it. *Loud: this stub IS the behavior, not a constant — the nightmare case (a bolted-on stub,
believed, untracked) this inventory exists to prevent.*

### 18. every-flow-is-free-fluvial-water — *added 2026-07-25 (FLOW slice 1, journal/0096)*
`dc-worldgen/src/deeptime/flux.rs` (the `FluxEntry` atom): three of the six fields of
flow.md § 1.3's atom are **carried but constant** in slice 1 — `form` is always
`FlowForm::Free`, `cause` always `FlowCause::Fluvial`, `fluid` always
`FluidId::WATER`. That is honest for what the solve *is* (surface water down a
gravitational head, and nothing else runs), and the fields exist precisely so the
shape does not have to move later — **but a reader must not mistake "one inhabited
value" for "the model has one regime."** `FluidId` in particular is a stand-in: it is
a small dense index with a single entry rather than a real fluid registry, because
`dc_core::MaterialId` is the *granular* registry and water is not in it.
**Heirs, all in the ratified FLOW continuation slot:** `form` → **(c)** the
free/bound edge + void intervals (which is also what makes a spring a derived outlet
and a cave free-phase flow); `cause` → **Movement 2b** material-aware transport
(eolian/glacial/gravity/marine/hydrothermal already enumerated); `fluid` → **(d)**
fluid identity with per-fluid competence curves, which is also the moment `FluidId`
becomes a real registry handle or is deleted in favour of one. **Blast:** any query
that filters the flow record by regime silently matches everything today; nothing
expresses the record at runtime yet, so it is currently invisible in game.
*Loud code markers at each field, naming the heir.*

**Related but NOT a stub — the honest empties.** The **atmospheric source** half of
the boundary face is structurally present and **zero**, because this solve's
precipitation source is a uniform 1.0 the derivation can predict exactly (S-2). An
empty field with a named heir is the S-5 identity floor, not a stand-in — there is no
*value* here that could quietly become the definition. The **vertical faces**
(`FaceKey::Down`/`Up`) were the other one, and **continuation (a) filled them**
(2026-07-25, journal/0098 — the head field); their emptiness assertion was deleted on
purpose, which is what it was written for. The **`load` on a vertical face** is now
the honest empty in that family — the bound phase carries solute, not suspended
clastic load, and dissolution is dormant until (c) — asserted by name in
`tests/head_field.rs::the_vertical_faces_are_no_longer_zero`.

### 19. the recharge-free water table — *added 2026-07-25 (FLOW continuation (a), journal/0098)*
`dc-worldgen/src/deeptime/head.rs`: the `head` condition-field (`dc:field/head`) solves
`∇·(T∇h) = 0` — the **R = 0 steady limit**. The real equation is `∇·(T∇h) = −R`, and
the recharge term is absent for a specific, stated reason: `R/T` needs a **real
hydraulic conductivity and a real precipitation depth**, and `grid.precip` is
normalized 0..1 with no depth scale. Inventing the scale would have made the slice's
own acceptance number (how far head stands above the ground) a tuning knob, which is
the failure mode `no-bandaid-tuning` names.
**What that costs, precisely:** with no recharge mounding the table beneath
interfluves, the potential a confined column inherits is its *neighbours'*, not a
distant highland's — so artesian excess heads come out in **metres** (60 columns,
max 2.94 m measured on the production world) where a real Great-Artesian-Basin
geometry gives hundreds. The field is a genuine potential and the *mechanism* is right (confinement
removes the seepage cap); the *magnitude* is a lower bound.
**Heir:** a real water-balance climate — the same missing precipitation-depth scale
that keeps the drainage solve's lateral source a uniform `1.0` per cell per epoch
(journal/0096 already flagged that pairing). Both halves want the same number, and
they must land **together** or the record's two halves disagree about what a unit is.
**Blast:** nothing expresses head at runtime yet. When (b)/(c) read it, a consumer
that treats `head` as "depth to water" gets a table that is systematically **too
deep** under ridges and correct at discharge zones. *Loud code marker in the module
docs § "What is deliberately NOT here".*

**Related but NOT a stub — the calibrations.** `CONFINING_CAP_M` (5 m to seal),
`STREAM_ANCHOR_AREA` (25 cells to be perennial) and `AQUIFER_K_MIN`/`AQUITARD_K_MAX`
are **knobs of a real mechanism**, in the same plausible-not-tuned register as S9's
physics constants — there is no measured Earth value at 460 m cells to defer to, and
each is stated with its reasoning at the constant. `PONDED_MIN_M` is neither: it is a
**numerical guard** against priority-flood residue, and it exists because 2.5×10⁻⁵ m
of dust manufactured 232 false artesian columns before it did.

### 20. the-weathering-profile-is-a-fixed-shape — *added 2026-07-25 (journal/0099, the front-is-a-profile slice)*
`dc-worldgen/src/geology.rs::WEATHERING_PROFILE` + `emplace_weathering_front`: the
front's **magnitude** is the deep model's (`FactLedger::weathering_product_m`, a real
time integral), but its **shape** is a constant — `round(7·exp(−j/3))` eighths of
product per band, so every front in the world has the same normalized profile and a
thickness of exactly `8/3 ×` its product metres. The exponential is the right *form*
(a reaction front consuming a downward-advecting reactant), and expressing the shape
scale-free is what lets one number set the depth — but the **decay length, the 7/8
cap and the resulting 2.67× thickness ratio are not measured from anything.** In the
field those are set by the balance of weathering-front descent against erosion rate,
by fracture density and permeability, and by climate; a granite under a wet tropical
saprolite and a granite under a stripped periglacial slope do not share a profile.
**Heir:** the deep tier carrying the front as a **depth-resolved term** rather than a
scalar — the ledger recording *where* in the column its product sits (the natural
continuation of the deep-cell inventory: a vertical distribution over the bedrock
seam, not one `FracM`), at which point this constant is deleted rather than tuned.
Sequenced-adjacent: it wants the same per-column basement lithology stub #16's heir
supplies, since the profile depends on the rock. **Blast:** the *look and dig* of
every weathering front — how deep the graded zone reaches and how fast it grades. Not
mass: the mass is conserved against the ledger whatever the shape
(`the_weathering_front_conserves_the_ledger_product_mass`). *Loud marker at the
constant, naming the heir.*

**Residue carried by the same entry — the structural pore rider still drops at a
contact.** `collapse.rs::mixed_at` now carries a **loose** pore rider through a mixed
voxel (the front's product), but a **structural** one — the sparse 1/8 igneous
accessory — is still dropped there, exactly as it was before 0099. That is deliberate
and narrow: carrying it would move every igneous contact voxel in the world (the
goldens), and the mass at stake is a mineral speck, not a modelled budget. It is
listed here so "the mixed path is rider-complete" is never assumed; the heir is
whichever slice next authorizes a golden move on the igneous contacts.

### 21. the-edge-id-is-positional-mixed-radix — *added 2026-07-25 (journal/0108, S20 option 2c)*
`dc-worldgen/src/deeptime/inventory.rs::EdgeId`: a `Fact`'s two endpoint bytes are
each `material.raw() * FORM_COUNT + form.raw()` — a **mixed-radix packing over the
material registry's POSITION**. Two consequences, and the first is loud by
construction while the second is not:

1. **A hard ceiling of `256 / FORM_COUNT = 51` materials** (26 today). It is a
   `const _: () = assert!(...)` beside the type, so it fails at **compile** time
   with a message naming this entry — it can never silently truncate. This is the
   *good* half: a stand-in that cannot become the definition without a build error.
2. **The id is only stable within one compiled binary.** Reorder or insert a
   material and the same two bytes name a different edge. Nothing serializes a
   ledger today (no `Serialize` on `Fact`/`FactLedger`/`LedgerField`/`DeepField`
   anywhere in the tree), so this costs nothing *now* — but *"ready-made worlds
   are the sanctioned answer"* means a ledger will be persisted, and a positional
   id read back against a changed registry is **silently reinterpreted**: granite's
   facts become diorite's and every test still passes.

**The exposure is inherited, not created.** The pre-slice `Fact` stored `MaterialId`
directly — the same positional `MatRepr` enum under the same `MATERIAL_COUNT == 26`
compile-time assert. The id did not add the instability; it moved it two bytes to
the left.

**Mitigation already shipped:** `EdgeDict` — the per-world dictionary `id →
(from-material-NAME, from-form, to-material-NAME, to-form)`, **derived** from the
facts (`EdgeDict::of_facts`, never a stored second copy that could disagree) and
carrying `validate()`, which re-resolves every row's id from its own *names* against
the live registry. That turns a registry change from *silently reinterpreted* into
**detected**. On the shipped world the dictionary has **one entry**
(`dc:granite`/structure → `dc:granite`/loose); in any plausible future, a few dozen.

**Heir:** the **interned** edge id — an `EdgeId` that indexes the per-world
dictionary rather than encoding registry positions. It removes the 51-material
ceiling entirely (65 536 inhabited edges regardless of registry width) and makes the
dictionary the authority rather than the witness. It has to be built anyway the
moment the ledger is first persisted, which is **the pager slice** (S20 option 3,
the reserved continuation of this one) — so the heir is *sequenced*, not
hypothetical. Until then `validate()` is the guard and nothing calls it in
production, only in the gate and the probe.

**Blast:** none today (in-process only, and the ceiling is a compile error). On the
day a ledger is written to disk: every provenance answer in every ready-made world,
silently, if the dictionary is not shipped and checked alongside it. *Loud marker at
the type and at the assert, both naming this entry.*
### 22. a-record-affordability-constant-that-changes-the-physics — *added 2026-07-25 (FLOW continuation (b), journal/0109)*
`dc-worldgen/src/deeptime/erosion.rs::MFD_MIN_WEIGHT = 0.01`: the MFD partition drops
any receiver allotted less than **1 %** of a cell's discharge and renormalises over
the survivors. Its stated justification is about the **archive**, not the landscape —
without it every land cell records an infinitesimal entry into every downslope
neighbour and the flux record pays megabytes for shares no consumer can distinguish
from zero.

**Why it is a stub and not a knob.** Apply ARCHITECTURE.md's test — *if the record
consumer disappeared tomorrow, would this constant exist in this shape?* **No.** And
it is not confined to the recorder: the floor is applied **inside `partition_cell`,
before renormalisation**, so the surviving receivers are handed the dropped share and
the *solve* moves water it would not otherwise have moved. A requirement of the
record has leaked into the physics — the exact shape "a summary is not an authority"
names, arriving from the other direction.

**What that costs, precisely.** It is a **hard threshold on a continuous partition**,
and Holmgren's form has none. On very flat ground — precisely the braid-plain and
delta-top regime the slice exists for — a cell with six near-equal downslope
neighbours has shares near `1/6`, far above the floor, so nothing is dropped and the
floor is inert. It bites in the *moderately* convergent regime, where a channel's
sidewall neighbours sit in the 0.2–1 % band: those are zeroed and their water is
handed to the channel. So the floor makes the drainage net **slightly more
channelised than the exponent alone specifies**, and the size of that bias has been
argued but **not measured**. The mass budget is untouched either way (renormalisation
is exact, and the residual rule makes the split exact).

**Heir:** the per-epoch aggregation-window decision (flow.md § 9, item 7b — still the
user's call) and the marine-sink residency lever, both of which are about what the
record can afford. Whoever settles the record's size budget should re-derive this
number *from* that budget rather than inheriting `0.01`, and should say explicitly
whether the solve is allowed to see it at all — the alternative shape is a floor
applied **only when writing the record**, leaving the solve's partition untruncated,
which costs an extra `n × 8` pass and decouples the two concerns properly.

**Blast:** the shipped world. Changing or removing this constant moves the goldens.
Nothing at runtime reads it. *Loud code marker at the constant.*

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
  of journal/0068, not merely the top unit. This
  does not close the gap — nothing dips yet — but it converts `lithology.rs`'s
  prose promise ("only the one function changes") into a compile-checked seam.

  **Since journal/0072 the socket was a PINNED PAIR** (audit site A1,
  shape-teacher #1): erosion no longer reads the verdict for its *rates* — it
  reads `providers::Providers::outcrop_shares`, whose identity
  `identity_outcrop_shares` is `lithology::exposed_shares` (the per-`Litho`
  window shares), and blends the susceptibility table by share at all four sites
  (fluvial incision + creep, periglacial frost, eolian deflation, littoral
  attack).

  **CONSOLIDATED to one slot 2026-07-22 (journal/0075, the `CoarseField`
  extraction).** The pair collapsed: `outcrop_at` is no longer an `Option<fn>`
  slot with its own identity — it is a **derived accessor**,
  `Providers::outcrop_at(units) = dominant_litho(outcrop_shares(units))` =
  `argmax ∘ outcrop_shares`. This is the extraction's own law (*categorical
  answers are the argmax OF the sample, never a stored field alongside it* —
  S-3), which made the second slot redundant. There is now **one** slot to
  supply (the quantity) and **one** heir socket; when the structural-deformation
  term lands it supplies the dipped `outcrop_shares` and the verdict dips with
  them automatically, because it *is* their argmax — the retire-together
  discipline is now structural rather than a paired obligation. Byte-identical:
  no generation consumer reads the verdict (erosion reads shares and takes its
  own `dominant_litho`), so the collapse moved no goldens.

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
