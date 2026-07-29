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
rule. **Five exist** — `outcrop_at`, `wave_energy`, `parent_p`, `depth_to_water`
and `paleo_temperature` (stubs 7b, 8, 13, the layer-cake sibling gap, and #6
via journal/0078). A slot does not retire a stub; it stops the stub from
silently becoming the definition, which is ARCHITECTURE.md § *A summary is not
an authority* made structural.

> ### ⚠ THERE IS NO "GENERAL REGISTRY", AND THERE IS NOT GOING TO BE ONE
> **Struck 2026-07-28 (user).** This clause used to end *"the general registry is
> deliberately unbuilt — four conversions is not enough to design one from"*,
> which implied a future mechanism somebody was going to build. **Nobody ever
> proposed one.** It was an inference, it read as a veto on extending the list,
> and it went stale anyway (it said "four" for four days after the fifth landed).
>
> **A seam's success condition is that it DISAPPEARS.** When the heir arrives it
> replaces the slot with a real mechanism; it does not "fill" it forever.
> **The worked example is the only completed case we have:** `burial_temp_c`
> asked *"how hot is it at this burial depth?"*, and when the geotherm landed the
> answer turned out to be **a field, not a value a provider `fn` can hold** — so
> it retired as a **field pass** and left the file entirely (journal/0093).
> `providers/mod.rs` records the lesson: *"the answer was 'this is not a provider
> at all — it is a field.'"* `depth_to_water` is documented as heading the same
> way. **So the exit is: slot → heir arrives → slot dissolves into a field or a
> pass.** You do not design a third-party declaration for a pattern whose job is
> to vanish.
>
> **What exists today IS a list you extend**, and extending it is cheap and
> encouraged — `Providers` is named struct fields holding `Option<fn>`, one
> module per slot, `None` *is* the identity, and adding one is a new file plus a
> compile error at every site that must answer for it. **Nothing here discourages
> a sixth slot.**
>
> **⚠ AND "SLOT" MEANS TWO UNRELATED THINGS — this is the actual source of the
> confusion.** They share a code shape (`Option<fn>` + identity fallback) and
> nothing else:
> - **Provider seams** (*world-level*, this file, `deeptime/providers/`) — holes
>   for systems **that do not exist yet**. Scaffolding. **Temporary by design.**
> - **Material behavior slots** (*material-level*, `north-star.md` § Materials —
>   `can_combust?`, `combust_rate`, `weather→`) — **what a material author
>   writes.** This *is* the SDK content surface, ratified 2026-07-23, and
>   **permanent by design.**
>
> `north-star.md:189-191` calls the material version *"the `Providers` pattern
> generalized from world-level to material-level"* — true of the **shape**, and
> easy to misread as the world-level *system* being promoted into the SDK.
> **That misreading is where "the general registry" came from.**
>
> **Where world-level seams land after the engine/plugin split is UNDISCUSSED and
> deliberately UNDECIDED** (user, 2026-07-28: *"I genuinely don't know… I don't
> think anyone has had a direct conversation about it… I don't want to burden us
> with more half-baked designs."*). **No decision is owed.** The one completed
> case resolved itself; the next can too. Do not open this without a seam
> actually forcing it.

---

## Active stubs

### 1. ruin-posts — **RESOLVED BY DELETION 2026-07-28 (journal/0121); no heir was ever built and none is owed**

**A stub whose subject no longer exists does not survive the removal of its
subject.** `collapse.rs::ruin_posts`, `Pregen.sites`, `Pregen.{ledger, overlay,
n_polities, observe_count}`, `pregen/history.rs` and the `dc:pass/history` pass
are gone (DECIDED 2026-07-26, user: *"they are unratified zealous fabrications
from the early bootstrapping of the project and I DO NOT care about them, they
WILL be wholesale replaced, they should just be removed. We do NOT have any form
of evo/socia/civ modeling even at the design stage: they are NOTHING"*).
Measured outcome: **102 `Block::Wood` post voxels on production-Medium → 0.**

**This entry is the cleanest illustration in the file of a stub-inventory failure
mode, so read the two sentences below before writing another heir line.** The
entry did its job perfectly for a week — loud, listed, honest about the
rule-of-thumb, naming an heir — and *every one of those virtues argued for
keeping the content.* An heir is a promise that the thing being stood in for is
wanted; **"what expresses this better?" silently presumes it should be
expressed.** The question the inventory never asked is CLAUDE.md § *Existence is
not standing*: **if this did not exist, would we build it today, in this shape?**
For settlement ruins the answer was no, and had been no the whole time. *A stub
entry is not neutral about its subject's standing — it asserts it.*

*Original entry, preserved:*
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

> **⚠ The heir sentence above says "until ecology lands".** That phrasing is why
> this entry is **NOT** evidence that `docs/design/ecology.md` owes anything to a
> civilization model. Ecology is a ratified user design about earth processes; the
> social-sim half of that sentence names a thing that does not exist even at the
> design stage, and it is not owed. See CLAUDE.md § Conventions, the ⚠ under
> *Existence is not standing*.

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
cites `spines.md` § 3. ~~(The 2026-07-22 audit called the comment an A-2
overstatement by quoting it with its "WILL" dropped; corrections #40 records that
the comment was already correct.)~~
**⚠ AMENDED 2026-07-28 — corrections #67.** The audit did **not** drop a `WILL`: it quoted the
comment **exactly as it stood the day before**. `11d4385` (**2026-07-21**) replaced
*"the collapse tier **reads**"* with *"**WILL** read … currently consumed by nothing"*, and that
commit's own message says it *"**Fixed the lying field.rs doc-comments that claimed the collapse
tier 'reads' these axes**."* The audit ran **2026-07-22**. **So the A-2 was real and live**, and
`11d4385` had already fixed it — the audit was right about the defect and one day behind the
tree. *#40's author read the current comment and inferred a paraphrase; a quotation without a
revision cannot distinguish the two.* **Heir:** a metamorphism pass reading the
P/T path into grade classes (schist/slate/gneiss roster, geology.md) — with a
named arrival address, ~~the `providers::burial_temp_c` geotherm (journal/0067)~~
**the `dc:field/temperature` geotherm FIELD PASS (`deeptime/geotherm.rs`,
journal/0093)**. *Address corrected 2026-07-29 (baseline sweep S4/F4): § 14 below
**removed the `burial_temp_c` provider slot entirely** on 2026-07-24 — a real
`T(depth)` is a **field**, not a value a stateless `fn(unit)` slot could hold — so
this heir named an arrival address that no longer exists. This entry was itself
amended 2026-07-28 (corrections #67), four days after the address was gone, and the
heir sentence was not revisited.*
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

### 10. frontier-pressure closed-form — **BLAST RADIUS IS NOW ZERO, 2026-07-28**
`dc-sim/statistical/world.rs::prior_pressure_weights` (~280): frontier
boundary conditions from a closed-form of danger, self-described stand-in.
**Heir:** the statistical tier's real cached summaries (S2). **Blast:** ~~low,
indirect (edge-region history synthesis)~~ — **none.** The "edge-region history
synthesis" it fed was the bootstrap settlement-history pass, removed 2026-07-28
(journal/0121, entry § 1 above). The stand-in still exists and is still honest
about being one, but **nothing in production reaches it**: it is now inside the
S2 engine's zero-consumer boundary (spines § 3). Retained as a live entry rather
than resolved, because the shape it stands in for — synthesized frontier
conditions for a bounded collapse — is S-1, the actual spine of this engine, and
will need an answer the first time anything collapses again.

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
argued but ~~**not measured**~~ — **MEASURED 2026-07-26 (journal/0113)**. The mass
budget is untouched either way (renormalisation is exact, and the residual rule makes
the split exact).

**MEASURED — the shipped world under hybrid `p`, floor at 1 % against floor off**
(`examples/hybrid_p_probe.rs`, which turns it off through the new
`DeepConfig::mfd_min_weight` knob):

| | floor 1 % | floor off | delta |
|---|---|---|---|
| peak catchment | 298 | 305 | **−2.1 %** |
| simultaneous `(cell, epoch)` divergence | 7,228,964 | 7,248,627 | **−0.3 %** |
| record entries | 4,107,188 | 4,171,836 | −1.5 % |
| record residency | 63.80 MiB | 64.79 MiB | −1.5 % |

So the leak into the physics is **real, small, and in the opposite direction to the
argument above**: the floor leaves the world slightly *less* concentrated, not more,
because the shares it drops are on the dispersive side of a partition whose steepest
direction was going to take them anyway. It buys **1.5 %** of the record for **2 %**
of the peak catchment.

**The constant is now a knob** (`DeepConfig::mfd_min_weight`, default unchanged at
`0.01`) — not so that anyone moves it, but so this table can exist. **A knob is not
an heir and this entry stays open**: the leak is still a record requirement applied
inside the solve, and the alternative shape below is still the right one.

**Heir:** the per-epoch aggregation-window decision (flow.md § 9, item 7b — still the
user's call) and the marine-sink residency lever, both of which are about what the
record can afford. Whoever settles the record's size budget should re-derive this
number *from* that budget rather than inheriting `0.01`, and should say explicitly
whether the solve is allowed to see it at all — the alternative shape is a floor
applied **only when writing the record**, leaving the solve's partition untruncated,
which costs an extra `n × 8` pass and decouples the two concerns properly.

**Blast:** the shipped world. Changing or removing this constant moves the goldens.
Nothing at runtime reads it. *Loud code marker at the constant.*

### 23. a-settling-law-that-cannot-see-buoyancy — *added 2026-07-26 (Movement 2b, journal/0110)*
`dc-core/src/materials/geology.rs::settle_energy` — `sqrt(grain_size_mm ×
specific_gravity)` — is the ordering key Movement 2b's transport pass sorts its
suspended load by (`deeptime::lithology::settling_table`), and the same function the
shipped S8 placer pass has thresholded on since long before that. It is a good
proxy for the property it was built for (which grain drops out first as flow energy
falls) and it is **structurally unable to express buoyancy**, because it *multiplies*
size by density instead of differencing the grain's density against the fluid's.

**What that costs, in the one place it is visible today.** Peat's property sheet is
5 mm at 400 kg/m³, so `settle_energy` reads **1.414** — the second-heaviest thing in
the deep-time roster, above sandstone at 0.840. Real peat **floats**. Nothing in the
shipped world currently notices (peat is laid by the biotic layer, which carries no
load, and the transport pass never picks up enough of it to matter — measured at
0.109 % of all sediment routing, journal/0110), but the day a flow does entrain
organic material it will sink like gravel.

**Why it is a stub and not a bug.** The right fix is a **better property-derived
settling law** — a Stokes/drag form carrying `(ρ_grain − ρ_fluid)`, which needs the
fluid's own density, which needs `flow.md` § 2.5's **fluid identity** to be stored
rather than derived (`material-genesis-notebook.md` § 3, the live open edge). The
wrong fix is a material-named exception in the pass, which would be a pass-purity
violation and would still be wrong for the next low-density material a pack adds.

**Heir:** fluid identity (flow.md § 2.5 / the genesis notebook's § 5 item 1), whose
arrival makes `ρ_fluid` a real quantity a settling law can read.

**Blast:** the shipped world *and* the shipped placer mechanism, together — both
consumers read the same function, so changing it moves alluvial grain ordering and
ore concentration in the same commit. That coupling is the reason to change it
deliberately rather than opportunistically. *Loud marker at `settling_table`.*

### 24. an-erosion-amplitude-that-cannot-reach-the-eroding-process — **SCOPE FIXED 2026-07-26; the calibration itself is superseded by § 27 / § 29 (journal/0114)**
`DeepOverrides::erosion_budget` (`deeptime/field.rs`) is documented as *"the
TERRAIN (erosion) amplitude"* — the one knob the project has for "more erosion",
reached by the `--erosion-budget <mult>` dev flag and named in the ROADMAP as the
lever behind the standing "conservative amplitude" call. It scales `weathering`,
`k_transport` and `k_bedrock` together, deliberately, so the **relative** rates the
erodibility coupling expresses never move.

**It does not scale `diffusion`** — and hillslope creep carries **96 %** of this
world's denudation (journal/0111). Measured, production untouched: at **100×** the
knob moves catchment-averaged denudation by **1.4×**. A knob that cannot move the
quantity it is named after is a stand-in for the calibration nobody has done.

**Why raising `diffusion` inside it is not the fix either.** The measurement shows
supply and transport are **coupled through the cover taper** `exp(−H/H*)`, `H* = 3 m`:
raise supply alone and the regolith made shields the rock that made it
(export/bedrock-erosion collapses 0.98 → 0.36); raise transport alone and there is
nothing to carry (1.7×). Only **together** do they pay — 132× — which is 59× more than
their separate gains multiplied. So a corrected budget is not one more multiplicand in
the same list; it is a statement about which rates are jointly calibrated, and against
**what clock** (corrections #56: the engine's metres-per-iteration constants have never
been checked against the ratified 500 Myr Phanerozoic register, and land as a whole
denudes ~10³× slower than any real landscape).

**Heir:** the owed *"calibrate iteration↔Myr against a real orogen"* item that
`earth-processes.md` § 3e has carried since 2026-07-19. That calibration is what turns
this knob from a multiplier someone picks into a derived consequence of the register.
The `denudation_probe` is the instrument that will judge it — its literature band table
is the acceptance test, and the world should land in the **1–10 m/Myr** stable-craton
band rather than at 0.011.

**Blast:** every world made afterwards. Moving any of these rates moves terrain shape,
the strata record, and every golden — the same event class as the erodibility, biotic,
tectonic and full-agent flips.

---

**DISCHARGED 2026-07-26 by journal/0114**, in two halves, and the second half did not
land where this entry expected.

**(a) The scope defect is fixed and cannot recur.** `erosion_budget` now goes through
`field.rs::scale_erosion_rates`, the *single* function that scales all four rate
constants — `weathering`, `diffusion`, `k_transport`, `k_bedrock` — and the shipped
calibration calls the same function. The knob's scope and the default's scope are one
piece of code, so they cannot drift apart again. Falsifier:
`tests/calibrated_rates.rs::the_erosion_budget_reaches_every_rate_including_diffusion`,
which would have failed on every commit before that entry. *Answering this entry's own
warning — "do not fix this by adding `diffusion` to the list" — the fix is not a fifth
multiplicand, it is the observation that the four are **one clock** and belong behind one
name.*

**(b) The calibration was BUILT, MEASURED and left OFF** — and that half is not
discharged, it is **superseded by § 27 and § 29**. The entry said the world should land
in **1–10 m/Myr**. It does not, at `EROSION_CALIBRATION = 45` or at any value:
`calibrated_rates` moves catchment-averaged denudation **0.0110 → 0.4142 m/Myr** and
bedrock erosion **0.0107 → 0.1271** (into the published *floor* band, 0.1–1, and out of
"below every band"), and no multiplier reaches the craton band with a world left in it.

Two findings replace the "just calibrate it" expectation this entry carried:

* **Export is proportional to mean regolith thickness** (§ 27), because hillslope creep's
  flux limiter binds on ~89 % of the cells that have regolith to move *already, at the
  shipped rates*. The pass is a one-cell-per-epoch conveyor; raising `diffusion` cannot
  speed it up (100× buys 1.6×); only the shoreline ring exports. Reaching 1 m/Myr costs
  order 100 m of mean cover — the ladder measures 118.8 m at 100× and 359 m at 300×.
* **The incision clamp leaves deep closed depressions at any multiplier above 1×**
  (§ 29) — 0 pits at 1×, **44 at 5×**, 148 at 45×, deepest 112 m. That is the blocker on
  the flag, and it is a defect the slow world could never have exposed.

So the knob's *scope* is fixed and the calibration's *value* is derived and pinned, but
the flip itself is still owed. **It remains an appearance-class, user-owned call**, now
with the evidence attached (journal/0114) instead of the guesswork this entry assumed.

*Loud markers at `EROSION_CALIBRATION`, `scale_erosion_rates`, `erosion_budget` and
`DeepConfig::calibrated_rates`.*
### 25. the-record-knows-what-arrived-and-not-who-brought-it — *added 2026-07-26 (Movement 2b continuation (b), journal/0112)*
`dc-worldgen/src/deeptime/recorder.rs::DepUnit`: a recorded unit now carries the
**material that arrived** (`species`, journal/0110) and, since this slice, that
material can have been delivered by either of two movers — the fluvial load or
hillslope creep. It does **not** carry which. `erosion.rs::arriving_species` sums
the two mixtures and takes one argmax, deliberately: a cell that receives half a
metre of fine clastic from upstream and half a metre of the same rock off the slope
above holds a metre of that rock, and ranking the movers against each other would
make the answer depend on which agent was asked first.

**What that costs.** *Colluvium and alluvium are not distinguishable by a label in
the record.* They are distinguishable by **signature** — colluvium is locally
derived and poorly sorted, alluvium far-travelled and sorted, and
`examples/colluvium_probe.rs` measures exactly that, by drainage-area decile — but a
consumer that wants to *ask* a unit "were you a debris apron or a point bar?" cannot.
The two facies are also **genuinely different rocks to a geologist**, so this is a
real gap and not a philosophical one.

**Why it is a stub and not an omission.** The obvious fix is a `mover` byte, and it
is **not free**: `DepUnit` is exactly 16 bytes with **zero padding left**
(`tag` 5 + `thickness_m` 8 + `unconformity` 1 + `chapter` 1 + `species` 1), so a
seventeenth byte becomes 24 — **+8 B across ~5.5 M units, roughly +42 MiB on the
production world**, against a whole-field residency of ~169 MiB. That is a 25 %
residency increase to add one axis, and it is exactly the trade the S20 ledger work
refused (*"the compaction the residency crisis invited was an axis drop, and an axis
drop is A-1 wearing a fact's paperwork"* — the mirror of it is a residency blowout
wearing an axis's paperwork).

**Heir:** the free version is a **packed provenance byte** — `Litho` has 7
inhabitants (3 bits) and the mover vocabulary is `flux::FlowCause`'s 7 (3 bits), so
`(species, mover)` fits one byte with two to spare and `size_of::<DepUnit>()` never
moves. That is a representation change across ~38 read sites, which is its own slice
and wants to land with **§ 13.8's lineage history** (the `Fact::Move` chain of
custody), because a mover byte and a parent pointer are the same question asked at
two depths. Until then the movers are separable only statistically.

**Related, and measured rather than guessed:** the same `mover` axis would discharge
**stub #18**'s constant `FluxEntry::cause` if creep wrote gravity-caused entries into
the flow record. `colluvium_probe` prints what that would cost — the donor faces
carrying creep in the final epoch, times the chapter count, times 16 B — so the
affordability question has a number attached rather than a shrug. Nothing about this
slice forecloses it; `FlowCause::Gravity` has been enumerated since FLOW slice 1.

**Blast:** anything that wants to *query* facies by process — a geologist-facing
inspector, an ore model that cares whether a gravel is alluvial or colluvial, the
structure-aware fine expression's choice of fabric. Nothing expresses it at runtime
today. *Loud marker at `DepUnit::species` and at `arriving_species`.*

### 26. a-channelisation-threshold-fitted-to-one-world — *added 2026-07-26 (FLOW continuation (b'), hybrid `p`, journal/0113)*
`DeepConfig::mfd_chi_lo = 1e-4` / `mfd_chi_hi = 1e-2` are the ends of the ramp that
carries the MFD convergence exponent from its hillslope value to its channel value.
The **index** they threshold is real and cited — `χ = A · S²`, Montgomery & Dietrich
(1988, 1992)'s channel-initiation criterion, which is why the law puts hillslopes,
trunk rivers and delta tops in the right three places from one expression. **The two
numbers are not.** They were read off the `χ` percentiles of **one world** (seed 1337,
`Extent::Medium`, 460 m cells) printed by `examples/hybrid_p_probe.rs`, chosen so a
plausible fraction of land sits at each end.

**Why that is a stand-in and not a tuning knob.** `S` is dimensionless, but `A` is in
**cells**, so `χ` still carries the grid's resolution: the same landscape at
`Extent::Large` (coarser cells via `DEEP_MAX_WIDTH`) accumulates fewer cells of area
for the same physical catchment, so the same two constants describe a *different*
fraction of it as channelised. A genuinely mountainous world, or one whose erosion
budget has been raised to a real denudation rate (stub #24), moves the whole
distribution too. **The threshold is currently a property of the world it was fitted
to, wearing the clothes of a property of landscapes.**

**What the fit is defended by, so the next author does not redo it blind.** The probe
sweeps `chi_hi` and reports the curve rather than the point, and the curve is not
monotone: at `chi_hi = 3×10⁻²` (36 % of land channelised) the single largest catchment
is the *biggest* of the four settings while **both** honest concentration measures are
the *worst*, because thousands of parallel threads that never merge are not a drainage
network. The shipped `1.2×10⁻¹` maximises p99 catchment **and** the top-1 % area share
**and** retains the most simultaneous divergence of any concentrating setting. So it is
a fitted constant with a measured defence — which is not the same thing as a derived
one, and that gap is this entry.

**Two heirs, and they are different fixes.** (a) The **joint supply+transport
calibration** — once the engine's rates are anchored to the ratified 500 Myr register
(stub #24, corrections #56), `χ` acquires a physical scale and the threshold can be
*derived* from a channel-initiation stream power rather than fitted. (b) Failing that,
a **dimensionless** form: normalise `S` by cell size in metres so the index stops
carrying the grid, or express the threshold as a quantile of the world's own `χ`
distribution (which is deterministic per world but makes the pass globally coupled,
and that trade is the decision to take, not to assume).

**Blast:** the shipped world's drainage network — moving either constant moves how
much of the land is treated as channelised, and every golden with it. Nothing at
runtime reads them. *Loud marker at both constants and at `MfdParams`.*

### 27. the-one-cell-per-epoch-conveyor — *added 2026-07-26 (the joint supply+transport calibration, journal/0114)*

**⚠ THE CONVEYOR IS GONE 2026-07-29 (journal/0122) — and the calibration fitted on top of it
does not survive.** The cap was a symptom of the same defect as § 29: `diffuse` ran one
explicit step per epoch at a per-edge coefficient **100.8×** past the 1/8 monotonicity bound,
and the inventory limiter is what stopped that from blowing up — by making every saturated
cell ship exactly one cell's worth downslope. Sub-cycling to the bound removes both at once:
the limiter's binding fraction falls **96.0 → 77.5 %** calibrated and **88.7 → 79.1 %**
shipped, and transport is no longer capped. **But everything below about what a multiplier
buys was measured against the capped operator and is now wrong in the other direction**: at
45× the fixed pass strips the world to **1.40 m** of mean regolith rather than thickening it
to 43.9 m. The heir named here — *"a creep operator not capped at one cell per timestep"* — is
**built**; what is now owed is a re-derivation of `EROSION_CALIBRATION` against it, which is
user-owned and appearance-class.
`erosion.rs::diffuse` is the **only working long-distance sediment router this world
has**: hillslope creep carries **99.87 %** of everything that leaves the land, against
the fluvial pass's **0.02 %** (measured on the calibrated world; the shares are
unchanged from journal/0111's pre-calibration 96 % / 0.02 %). And it is an **explicit
diffusion whose flux limiter binds in 91 % of cell-epochs** *at the uncalibrated rates*
— `diffuse_scale_cell` clamps a cell's outflow to the regolith it actually has, so the
pass has already stopped being a diffusion and become **"move everything one cell
downslope this epoch"** almost everywhere.

**What that costs, measured.** Because transport saturates and supply does not, a uniform
scaling of the rate constants does *not* hold the cover thickness fixed the way the
calibration hypothesis expected. It thickens at every rung —
`mean H` **4.6 → 8.8 → 43.9 → 118.8 → 359 → 782 m** across multipliers 1 / 10 / 45 / 100 /
300 / 1000 — and catchment export tracks it, because export is set by how much cover the
shoreline ring can hand over. **Raising `diffusion` alone buys 1.6× at 100×.** The
published stable-craton denudation band (1–10 m/Myr) therefore costs roughly **150 m of
mean regolith**, which is not a landscape; the calibration stopped at 45× where cover is
43.9 m and relief has moved 4.6 %.

**Why the rate is nonetheless honest at the shipped setting.** One cell per epoch is
460 m per 2.5 Myr = **0.18 mm/yr**, squarely inside the measured range for real soil
creep (0.1–10 mm/yr). What the saturated operator loses is **sub-cell structure**, not
magnitude — so this is a **discretisation** stand-in, not a wrong physical law, and it is
why the shipped world's shape survives the calibration.

**Two heirs, and they are different fixes.**
(a) **Rivers that actually carry.** Fluvial yield is 0.02 % of export. On Earth the long
   -distance router is the channel network, not the hillslope; a world whose rivers moved
   a real share would drain its interior without burying it. That is the FLOW arc's, and
   hybrid `p` (journal/0113) is its first instalment.
(b) **A creep operator that is not capped at one cell per timestep** — sub-stepping, an
   implicit solve, or a finer deep cell. Note this is the same owed item as
   `earth-processes.md` § 3e's *"calibrate iteration↔Myr against a real orogen"* seen from
   the other end: ~~the cap is `cell_m / myr_per_epoch`, so it is a statement about the
   **register**, and re-anchoring the clock is a **user-owned** call this slice
   deliberately did not touch.~~
   **🔴 WITHDRAWN 2026-07-26 (corrections #63) — marked here 2026-07-29 (baseline sweep
   S4/F3).** § 29 below withdrew exactly this link and states the measurement three ways:
   *"`myr_per_epoch` does not exist as a knob and **the epoch length is measured NOT to be
   the register**"*; *"**(b) IT IS NOT A TIME-STEP LIMIT**"*; the limiter *"makes the
   transfer a function of **inventory, not of `rate × dt`**, so refining `dt` cannot reduce
   a transfer `dt` does not set."* **#27 and #29 share the LIMITER, a stronger link than
   the clock was.** *The withdrawal reached the entry that made the claim (#29) and never
   reached the entry it was a claim about (#27) — the corpus's own named failure mode, in
   its own registry. Anyone opening #27 read a live instruction to go re-anchor a clock
   that is not the lever.*

**Blast:** the ceiling stands wherever the constants are set, so every future erosion
calibration is bounded by it until an heir lands — and it is half of why
`calibrated_rates` ships **off** (the other half is § 29). Nothing at runtime reads the
limiter. *Loud markers at `TransportLedger::creep_limited_cell_epochs`,
`diffuse_scale_cell`, and `EROSION_CALIBRATION`.*

### 28. the-agent-magnitudes-the-calibration-left-behind — *added 2026-07-26 (journal/0114)*

> **✅ DISSOLVED 2026-07-29 — USER RULING, and the user WITHDREW THEIR OWN PRIOR
> RATIFICATION to do it.** *"I do not care if frost/wind magnitudes increase 45×, do not try
> for byte identicality on `calibrated_rates`, I do not care about my previous ratification on
> looks there. We just need to move fast."*
> **Scale the three excluded agents with everything else.** The exclusion's reasoning was
> sound and is now moot: it protected three appearance numbers the user judged live, and the
> user has released them. **This is not a stub any more and it is not a blocker on the flag
> flip** — it was filed as a *pre-flip* blocker precisely so it would be visible before
> `calibrated_rates` flipped, and it was, and the answer was "don't care".
> - **Byte-identicality on the calibrated arm is explicitly NOT wanted.** Do not spend a slice
>   preserving it. The shipped arm's goldens are a separate question and still hold.
> - **The live magnitudes tour is no longer owed as a gate on this.** It may still be wanted
>   as a walk; it is not a dependency.
> - *Recorded at length because a user withdrawing their own ratification is exactly the event
>   the corpus is worst at: the ratification is written down and the withdrawal usually is not,
>   so the stale ratification outlives it and blocks work nobody is blocked on. See
>   `earth-processes.md`, which carried this as a live constraint.*

`scale_erosion_rates` deliberately scales four rates and **not** `wave_erosion` (0.05 m
/epoch), `eolian_deflation` (0.02) or `frost_weathering_gain` (1.5). The reason is sound —
those are the **agent magnitudes**, explicitly unratified appearance numbers the user
judges live, station by station (`production_config` § full_agents: *"the LIVE MAGNITUDES
TOUR — not this line — ratifies the numbers"*), and folding them into a calibration would
smuggle three appearance calls into one derivation.

**But the consequence is real and is not recorded anywhere else.** The four core rates
moved 45× and these three did not, so **relative to the landscape they act on, the wave,
wind and frost agents are now 45× weaker than the day their magnitudes were chosen.**
Measured on the calibrated world: the wave quarry is **0.10 %** of export and eolian dust
**0.01 %**. They were never large, and they are now smaller by construction rather than by
finding.

**Heir:** the live magnitudes tour those numbers have always been waiting for — which is
now *more* owed, not less, because the walk will be judging them against a landscape that
moved underneath them. **Blast:** coastal cliff retreat, dune fields, the periglacial band.
*Loud marker in `scale_erosion_rates`' doc comment, which names the exclusion and why.*

### 29. the-hillslope-conveyor-that-checkerboards-the-regolith — *added 2026-07-26 (journal/0114); re-scoped by the walk (journal/0115, corrections #61/#62); **RENAMED AGAIN 2026-07-26 when the discriminators ran (journal/0116, corrections #63)** — was "the-clamp-that-was-green-because-nothing-eroded", then "the-solve-that-goes-grid-unstable-above-1×" (right about the symptom, wrong about the mechanism)*

**⚠ DISCHARGED 2026-07-29 (journal/0122) — the operator is fixed, and one inference below is
falsified (corrections #72).** The pass now sub-cycles each epoch to keep its per-edge
coefficient inside `CREEP_MAX_EDGE_COEFF = 1/8`, the bound below which an explicit
four-neighbour Laplacian cannot flip the grid-scale mode. **The (b) block below is wrong**:
this *was* a stability limit, and the 4× refinement was ~25× short of reaching it (peak
effective coefficient **12.60** calibrated against a 1/8 bound). Measured after the fix, on
production-Medium: calibrated `conc(h)` rms **62.42 → 2.66 m**, its ACF(1) **−0.821 → +0.001**,
the surface's concavity rms **40.42 → 0.30 m** and its ACF(1) **−0.867 → +0.185**, closed
hollows past 10 m **818 → 12**. **What is NOT discharged and is now the live item: the
multiplier itself.** `EROSION_CALIBRATION = 45` was fitted against the capped conveyor; with
the cap gone the same multiplier strips the world to 1.40 m of mean regolith. See § 27 and the
ROADMAP.

**⚠ READ THIS BLOCK BEFORE THE ORIGINAL ENTRY BELOW.** The original sized this defect at
**148 pits** using a *below-all-eight-neighbours* census, and diagnosed it as the incision
clamp being defeated by the four phases that run after it. **The number was a severe
undercount and the diagnosis is probably not the dominant mechanism.** Both were corrected
by a live walk over the calibrated world.

**The measurements that replace it** (production-Medium, seed 1337; shipped world as
control). Deep-cell concavity, `mean(8 neighbours) − self`:

| | mean | p10 | p50 | p90 | p99 | >1 m | >20 m |
|---|---|---|---|---|---|---|---|
| shipped | −0.14 m | −0.3 | −0.1 | +0.1 | +0.3 | **0.0 %** | **0.0 %** |
| calibrated | +0.73 m | **−50.8** | −0.1 | **+52.8** | **+106.3** | **35.8 %** | **23.2 %** |

Closed hollows by depression fill (`filled − routed`, which does **not** saturate):
**0 → 1,377 (3.1 % of land), deepest 112.8 m, 5.4 km³.** Regional density within 10 km of
the walk station: **7.8 %**, so they cluster ~2.5×.

> **Relief grew 4.6 %. Cell-to-cell roughness grew ~170×.** The landscape's *shape* survived
> the calibration exactly as journal/0114 claimed; the **grid** did not. Symmetric ±50 m
> tails with an unmoved median is adjacent cells oscillating against each other, not terrain
> becoming rugged.

**So what is stood in for is bigger than a clamp.** The premise being substituted is that
**the erosional solve produces drainable terrain at realistic rates**. It does not — above
1× it produces grid-scale noise, of which the closed pits are the tail that happened to have
no outlet.

**~~Hypothesis, explicitly not measured:~~ THE DISCRIMINATORS WERE RUN 2026-07-26
(journal/0116) AND THE HYPOTHESIS IS FALSIFIED.** The standing story was *an explicit scheme
past its stability limit* — creep's flux limiter binds on 89–96 % of cells (stubs #27), so a
saturated explicit operator overshoots. It was flagged as unmeasured, it was measured, and it
is wrong. What the three discriminators returned, all on production-Medium with the shipped
world as control:

- **(a) IT IS A CHECKERBOARD — confirmed, decisively.** Concavity lag-1 autocorrelation
  **+0.377 / +0.267 (shipped)** against **−0.867 / −0.912 (calibrated)**, with lag 2 back at
  +0.56 / +0.71 and a first-difference ACF of −0.86. Reference values are derivable in closed
  form: white noise is **−1/6**, a perfect checkerboard is **−1**. Both axes, so a true 2-D
  Nyquist mode, not striping.
- **(b) IT IS NOT A TIME-STEP LIMIT.** Refined 4× at fixed total simulated time (`k×` epochs,
  `1/k×` every per-epoch rate) the concavity rms goes **40.46 → 45.29 → 38.76** — a 4 %
  change under a 4× refinement, non-monotone — while the landscape holds (relief +3.9 %, mean
  surface −0.3 %) and the shipped control reproduces itself to three digits. The checkerboard
  gets **purer** as the step shrinks: ACF(1) −0.867 → −0.909 → **−0.947**. Closed hollows *do*
  converge (1,377 → 955 → 280), so the **pits** are partly a step artefact and the
  **oscillation is not**.
- **(c) ISOSTASY IS THE DAMPER, NOT THE DRIVER** (a mechanism proposed mid-flight and worth
  recording as killed). `iso_rate` 0.50 → 0.25 → 0.00 takes concavity rms **40.46 → 62.03 →
  90.34** and closed hollows **1,377 → 2,150 → 13,012** — monotone in the *opposite*
  direction. The clean point is 0.25 (relief moves 1 %, roughness worsens 53 %). And on the
  **shipped** world `iso_rate = 0` takes concavity rms 0.22 → 19.71 m and closed hollows
  **0 → 6,215**: the Airy relaxation toward a flexurally smoothed target is a strong
  grid-scale low-pass filter and it is currently the only one. **Do not touch `iso_rate`.**

**WHERE THE OSCILLATION LIVES — split `surf = r + h`, run the same Laplacian over each
summand, and read the creep limiter's binding fraction off `TransportLedger`:**

| | limiter bound | conc(**r**) rms · ACF(1) x/y | conc(**h**) rms · ACF(1) x/y | surf conc rms | mean h |
|---|---|---|---|---|---|
| shipped k=1 | 88.7 % | 3.42 m · −0.101 / −0.173 | 3.43 m · −0.102 / −0.183 | **0.22 m** | 4.58 m |
| calibrated k=1 | **96.0 %** | 23.77 m · −0.546 / −0.508 | 61.95 m · −0.819 / −0.851 | 40.46 m | 41.41 m |
| calibrated k=2 | **94.9 %** | 12.59 m · −0.344 / −0.219 | 55.21 m · −0.878 / −0.914 | 45.29 m | 36.59 m |
| calibrated k=4 | **94.7 %** | **5.88 m · −0.108 / +0.069** | **42.43 m · −0.929 / −0.945** | 38.76 m | 24.45 m |

- **The bedrock solve CONVERGES and was never the problem.** `conc(r)` goes 23.77 → 12.59 →
  5.88 m under 4× refinement — nearly `1/k` — and its ACF walks back to −0.11 / +0.07. There
  *is* a real time-step artefact in this world; it is in `r`, and D2 converged it away.
- **The regolith does the opposite.** `conc(h)` falls only 32 % while its ACF goes **−0.819 →
  −0.878 → −0.929**, toward a *pure* checkerboard. The 32 % is not convergence either — the
  refinement does not hold the cover fixed (mean `h` 41.4 → 24.5 m: splitting error against the
  nonlinear `exp(−H/h*)` weathering taper) — and normalised by the cover the operator actually
  moves, roughness **grows**: `conc(h)/h̄` 1.50 → 1.51 → **1.74**, `rms(h−h̄)/h̄` 1.69 → 1.74 →
  **2.06**.
- **So the flat surface total was two defects cancelling**: a converging bedrock artefact plus
  a sharpening regolith checkerboard. `corr(concavity, h − h̄) = −0.831` calibrated (+0.147
  shipped), `rms(h − h̄)` **70.1 m vs 4.0 m** — the concavity field essentially *is* the
  local-regolith-excess field.
- **On the SHIPPED world `r` and `h` roughness ANTI-CORRELATE almost exactly**: 3.42 m and
  3.43 m of grid-scale concavity summing to **0.22 m**. The blanket fills the bedrock's own
  grid-scale hollows. **That compensation is what has broken**, and it is why neither component
  was ever visible.

**Why (b) returned a null — MEASURED, not inferred: the limiter is deaf to the step.** Binding
fraction **96.0 % → 94.9 % → 94.7 %** across a 4× refinement. A limiter that caps export at
*the cover the cell has* makes the transfer a function of **inventory, not of `rate × dt`**, so
refining `dt` cannot reduce a transfer `dt` does not set.

**And saturation alone is NOT sufficient — do not brief it as if it were.** The limiter already
binds on **88.7 %** of *shipped* cells, whose `conc(h)` ACF is −0.10: no checkerboard at all.
What the calibration adds is **cover** (mean regolith 4.58 → 41.41 m). *The oscillation
amplitude scales with the inventory the limiter surrenders* — which is what "inventory, not
`rate × dt`" predicts, and why this could only appear once the world was made to erode.

**So the heir is a hillslope-transport slice — not a clamp slice, not a time-step slice.** The
register is the **flux limiter / donor-cell partition in `erosion.rs::diffuse`**: an implicit
or under-relaxed update, a limiter that cannot export more than *levels the pair*, or a
symmetric two-pass exchange. **Hypothesis for the heir to test first, explicitly not measured**
(same discipline as the block this replaces): a donor-cell scheme that moves everything
downslope has a period-2 mode by construction — A gives all its cover to B, B is now higher and
gives it back — a checkerboard in `h`, independent of `dt`, damped only by whatever low-pass
sits downstream (here, isostasy). ~~`(b) is the same register as stubs #27's heir (b)`~~ —
**withdrawn** (corrections #63): `myr_per_epoch` does not exist as a knob and the epoch length
is measured *not* to be the register. #27 and #29 share the **limiter**, a stronger link than
the clock was.

**Two instrument defects fall out of this and are part of the heir's scope:**
- `mfd_routing::no_interior_cell_is_cut_below_all_of_its_neighbours` is a **winner-take-all
  predicate that saturates** — as the defect generalises, neighbours sink too and stop
  qualifying each other, so the count falls toward zero exactly when damage becomes total
  (corrections #62). Re-assert on **fill depth and concavity**.
- journal/0114's binding acceptance criterion, *relief within 5 %*, is a **global extremal
  statistic** and cannot see spatial arrangement at all (corrections #61). Every erosional
  criterion pairs an aggregate with a **neighbour-relative** measure.

**Also measured and null, recorded so nobody re-runs it:** `apply_thickening`'s 1000 m
`t_crust` floor is engaged on **0.01 % of cells**, identically on every arm and at every
`iso_rate`. It is not a nonlinearity here.

**Measured by:** `dc-worldgen/examples/walk_tour_0115.rs` — `cargo run --release -p
dc-worldgen --example walk_tour_0115 -- --sweep` (the three ladders, ~12 min at Medium) and
`-- --fields` (which summand oscillates + the limiter's dt-response).

*Original entry follows, unedited — its mechanism is plausible and probably contributes; it
predicts isolated deep holes, which is a subset of what the world shows.*

### 29 (original). the-clamp-that-was-green-because-nothing-eroded — *added 2026-07-26 (the joint supply+transport calibration, journal/0114)*
`erosion.rs`'s **never-incise-below-the-lowest-receiver clamp** is applied inside the
transport phase, and **four later phases in the same epoch can lower a cell past it** —
weathering, hillslope creep, wave attack and eolian deflation all run after incision.
Over 200 epochs those metres compound, and the cell ends up in a hole its own outlets
cannot drain: the runaway knickpoint the clamp exists to forbid, arrived by a route the
clamp does not watch.

**Measured, on `tests/mfd_routing.rs`'s fixture, by sweeping the erosional amplitude:**

| uniform × | interior cells >1 m below every neighbour | deepest |
|---|---|---|
| **1 (shipped)** | **0** | 0.00 m |
| 5 | 44 | 44.85 m |
| 10 | 66 | 55.13 m |
| 20 | 87 | 124.56 m |
| 45 | 148 | 111.81 m |

**It is not a property of the multiplier.** Pits appear as soon as the amplitude leaves
1× and are already 45 m deep at 5×. The shipped world scores zero **because it barely
erodes at all** — 0.011 m/Myr, 9× slower than the slowest landscape ever measured
(journal/0111) — so there has never been enough motion for the clamp to be tested. This
is the closed-system lesson one level down from journal/0111's: *a system that has stopped
cannot detect its own logic errors either, because nothing exercises them.*

**Why it is a stub and not a bug report.** The shipped world is unaffected and provably
so (`pits == 0` is asserted, unchanged, on production). What is stood-in-for is the
**premise** that the clamp is sufficient — a premise every future erosion calibration
depends on, and the reason `DeepConfig::calibrated_rates` ships **off**.

**Heir:** re-apply the floor after the last phase that can lower a cell, or make the
lowering phases clamp-aware. That is a well-specified slice and it is **the blocking
dependency for journal/0114's flag flip** — until it lands, this engine cannot run erosion
at any realistic rate at all, which makes it the most valuable erosion work on the board.

**Blast:** terrain shape everywhere, once the flag flips; nothing today.
*Loud markers at `EROSION_CALIBRATION`, `DeepConfig::calibrated_rates`, and
`tests/mfd_routing.rs::no_interior_cell_is_cut_below_all_of_its_neighbours`.*

### 30. a-pass-that-hand-rolled-its-own-timestep — *added 2026-07-29 (journal/0122); **heir named by the user in the same breath as the spine that excludes it***

`Erosion::diffuse` sub-cycles itself: `n = ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)`,
with `n = 1` bit-identical to the pre-slice operator. It is correct, it is derived (from the
von Neumann bound `a = 1/8`, not fitted), and it is the reason the calibrated solve no longer
oscillates. **It is also a `dt`, computed inside a pass, because the engine does not have one.**

**What it stands in for: RATE.** Ratified 2026-07-24 from the user's 2026-07-23 sketch — a
pass's phase length, `dt` scaling its transformations, a high-rate pass running several
sub-turns while a low-rate one runs once — and **never built**; `dt` is pinned to `1.0` and no
transform scales by it. `ARCHITECTURE.md` had already argued that **this blocker is what RATE's
absence produces**. That argument is now confirmed from the other side: the pass, given no
engine clock, grew its own.

**HEIR — RE-POINTED 2026-07-29 (user), the same day it was first assigned.** The first ruling
sent the sub-cycle to **RATE** (*"the sub-cycle part belongs with RATE"*). The user re-opened it
rather than let RATE grow — *"I really hate to make RATE more complex now. Couldn't substepping
be solved within the field instead, where it takes `dt` from outside and calcs its own internal
multiplier in addition to that to stay within bounds?"* — and that is the better placement.
**Heir: the S-10 field-solver primitive itself.** The kernel takes `dt` from outside and
sub-divides internally to stay inside its own bound. **RATE is NOT expanded** and stays as
ratified: authored cadence plus a real `dt`. *The scope-expansion flag this entry carried for a
few hours is withdrawn — RATE is untouched.*

**Why the kernel owns it.** Four inputs set the threshold, four owners: the **stencil** (a
different constant for a 4- vs 8-neighbour Laplacian) is the *kernel's*; **`dx`** and **`dt`**
are the *engine's*; the **coefficient field** is *content*. Only the kernel can know its own
constant. Housed anywhere else, every plugin author needs a von Neumann analysis before writing
a diffusion pass — the exact prerequisite plugin-agnosticism forbids.

**And the bound is NOT material availability.** That is the *inventory* limiter, a different
mechanism; the stability bound is **numerical** and would exist if every cell held infinite
material. The inventory limiter **masks** it — capping the flipped mode at what the cell holds
turns a blow-up into a finite limit cycle, which reads as stable and is deaf to `dt`. `sat.rs`
needs no sub-cycle only because `lateral_c = 0.25` happens to sit inside its bound **by
parameter choice** — an accident of tuning, not a design, and precisely the trap the next author
inherits if the bound does not live with the kernel.

**Blast radius:** every future field pass that diffuses anything. Right now the only defence
against shipping past a stability bound is that one author did the analysis once, in one pass —
and the shipped world sat **2.1× past that bound** for weeks with every golden green.
**Heir:** RATE (`material-behavior.md` §5; ROADMAP § Sequenced, now sequenced ahead of the
refinement design pass by the same ruling). *Loud marker owed in `diffuse`'s doc comment naming
RATE as the heir.*

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
