# Ores — the v1 ore materials design pass

> **DRAFT — NOTHING RATIFIED.** Produced 2026-07-21 by a dispatched design
> agent on the user's order ("design the v1 ore materials"), as the
> user-owned content design pass slated 2026-07-20 (ROADMAP, earth-processes
> § payoff layer). Everything here is a proposal with options; § NEEDS
> RATIFICATION is the list of calls that are the user's alone. The *which
> ores* question was already ratified (geology.md § Ore, DECIDED 2026-07-20)
> — this pass turns that roster into classes, members, genesis honesty,
> forms, and prospecting reads, and it does not reopen the roster except
> where a later decision or measurement collides with it (reported, not
> resolved — § 8).

---

## 0. Reality first — how ore actually happens

(The earth-processes method: the causal account before any code.)

An ore deposit is never a substance sprinkled into rock. It is a place where
an ordinary process ran an **enrichment step** — something moved a dispersed
element around and then something else made it stop, all at once, in one
place. Every deposit is therefore the terminus of a chain, and the chain is
what a prospector actually reads. The six chains behind the ratified v1
roster:

- **Placer (gold).** A source rock somewhere upstream carries traces of a
  dense, chemically inert mineral. Erosion frees the grains; the river
  carries everything; but transport sorts by settling behavior, and a dense
  grain behaves like a much larger light one. Wherever flow energy drops
  through the grain's threshold — inside bends, bar heads, the downstream
  side of riffles, the winnowing band just off the channel core — the heavy
  fraction concentrates while lighter sediment keeps moving. The record: a
  graded coarse body with the dense grains riding in it, richest just below
  the energy threshold, thinning down-fan. The read: flakes in the gravel
  mean a source *upstream*; grade increasing upstream means you are getting
  closer.
- **Orogenic (lode) gold.** In a collisional belt, rock buried to 10–20 km
  heats and devolatilizes; the expelled water carries dissolved silica and
  gold up through the shear zones and fractures of the deforming core, and
  it precipitates as quartz veins where pressure and temperature drop.
  Nobody sees this happen — it becomes visible only when isostatic rebound
  plus erosion strip the overburden and **exhume the cooked core**, veins
  and all. The record: gold-quartz hosted in the highest-grade metamorphic
  rock of an old belt's interior, zoned outward with the grade. The read:
  old worn mountains with schist/gneiss cores are gold country; young
  sharp belts are not (their veins are still ten kilometres down).
- **Bog iron.** Groundwater moving through soil under vegetation picks up
  iron as reduced, soluble Fe²⁺. Where that water surfaces at a wetland
  margin and meets air — often with iron-oxidizing microbes accelerating it
  — the iron oxidizes, becomes insoluble on the spot, and accumulates as
  rusty nodules and pans in and under the peat. Centimetres per century;
  renewable on a human timescale; never deep. The record: limonite nodules
  inside organic wetland facies. The read: bogs, rusty seeps, orange-slicked
  still water. This is the iron a starting culture smelts first, everywhere
  on Earth that had bogs.
- **Banded iron formation.** An ocean chemistry that no longer exists:
  seawater loaded with dissolved iron, precipitated in vast rhythmic sheets
  (iron oxide / silica couplets) on ancient continental shelves when free
  oxygen began appearing. Hundreds of metres thick, laterally enormous,
  found only in the oldest crustal blocks — cratons — because the process
  died with the ocean that fed it. The record: banded red-grey chemical
  sediment low in the oldest part of a craton's stack. The read: deep
  ancient shield country, not young belts, not recent basins.
- **Redbed (sediment-hosted) copper.** An arid basin fills with red,
  oxidized sand; saline oxidizing groundwater dissolves trace copper out of
  the basin fill and migrates for ages. Wherever it crosses a **reductant**
  — a buried organic-rich layer, plant trash on an old channel floor — the
  chemistry flips and the copper precipitates as sulfides right at the
  contact, later weathering to the green (malachite) stains that betray it.
  The record: grey/green ore-bearing horizons at the contact between red
  arid clastics and dark organic beds. The read: red rock country, then
  find the dark bed inside it, then follow the green.
- **Evaporites.** A basin whose water has no exit, in a climate where
  evaporation beats inflow, concentrates its brine until minerals
  precipitate in solubility order — carbonates, then gypsum, then rock
  salt. Sea-level cycles and climate swings stack these beds tens to
  hundreds of metres thick. The record: white/translucent chemical beds in
  arid closed-basin fills. The read: the flat floor of a dead basin, no
  outlet, arid tags all the way down.

And the one already shipped: **coal** — a swamp holding a subsiding site
long enough for peat to outrun decay, then burial (journal/0026). Coal is
the proof that the facies-routed pattern works end to end; most of this
document is the other five chains asking for the same honesty.

## 1. Priors — what the corpus already holds (cite, don't re-tell)

Nothing in this section is new; the user has decided or the project has
measured all of it. The design below is constrained by every line here.

- **The roster is ratified** — geology.md § Ore (DECIDED 2026-07-20, user):
  coal (shipped) · banded iron · bog iron · redbed copper · orogenic gold
  (lode → placer, the flagship chain) · evaporites. Deferred **with their
  engines**: tin/tungsten (plutons), porphyry copper (intrusions),
  fault/hydrothermal veins (faulting). "No ore before its process."
- **Representation is ratified** — same entry: ore is a MATERIAL inside a
  HOST via the existing eighths machinery (olivine precedent, journal/0011);
  three forms (pore partials in host / loose partials / entire blocks);
  **grade IS the eighths count** — no separate grade mechanic. Raw-form
  appearance stays realistic; **ore is subtle** (visuals.md — no glint;
  a pack's height channel is its ore-clumping knob).
- **Endowment is ratified** — soft guarantee at Medium+: every v1 ore
  present at least marginally, quality/abundance honestly uneven. (The
  *mechanism* of the guarantee is not designed — § 6 R6.)
- **Identification/naming is ratified** — system ids `dc:ore/...` /
  `dc:geo/...`; display naming belongs to the future culture/language
  layer; genuinely distinct raw forms are DISTINCT materials refining to
  the same metal (bog iron vs banded iron), because identification
  knowledge is real knowledge.
- **The Expression doctrine binds everything here** — geology.md
  § Expression of the ledger + Genesis addendum + Enhancement doctrine
  (all DECIDED 2026-07-21, user): placement derives from recorded cause;
  form follows provenance; an unexpressed ledger term is loudly temporary
  or it is a defect; a local rule is honest exactly to the degree it is a
  deterministic function of the recorded field; stubs live in
  `docs/design/stubs.md` with named heirs.
- **The methodological template is journal/0026**: classes filled by one
  vanilla member; roster decided by MEASUREMENT (the charcoal precedent —
  0 of 158,310 beds survived the 0.9 m sieve, so the material was not
  shipped and the honest form is the inclusion; no coal rank ladder
  because the record never visits the discriminating axis, but the class
  documents the axis so a pack can fill it without moving a seam).
- **The built ore machinery**: the placer pass (journal/0007 — gold dust
  lands mid-gravel from property-derived `settle_energy`, zero
  ore-specific code, S8 zonation barren→peak→thinning proven in-world);
  accessory pore partials (`dc:accessory/mafic`/olivine, photographed
  0011); the pore-packability genesis exemption (materials.md,
  `K_PORE = 0.25` — worldgen-authored inclusions bypass the mechanical
  rule); facies routing via `Biofacies` (the coal pattern); mixture
  storage proven GO (S8 — the combinatorial cap makes ore partials cheap).
- **The recorded context axes that exist today** (`recorder.rs`): `env`
  (subaerial/subsea), `aridity`, `energy`, `biota`
  (Mineral/Soil/Peat/Coal/Charcoal/Retro), `eolian`, `unconformity`,
  `chapter` (0-based tectonic chapter — the age label). Plus, in
  tectonic-history worlds: `exhum`, `t_crust`, the ~5 KB chapter table,
  and drainage export (`recv`/`area`/`lake`).
- **U8 flipped 2026-07-21** (user, this session): production worlds now
  populate `exhum`/`t_crust`/`chapters`/drainage export. The EXPRESSION
  slices that consume them — metamorphic-grade classes, dip/fold/fault in
  cut faces, the 3e-2 drainage consumers — are Sequenced ("tectonic
  expression at the collapse tier") and **unbuilt**.
- **A measured warning on the flagship** — ROADMAP/S12 (2026-07-20):
  **exhumation comes out metre-scale at shipped erosion rates**, so
  exhumed-core signals are illegible at any amplitude until the
  erosion-supply calibration (Sequenced) lands. § 8 discusses what this
  does to orogenic gold.
- **Class satisfiability** (API.md): every class a pass selects from must
  have a member or the world refuses to build, naming pass and class; a
  pass that introduces a class registers its fallback member in the same
  batch. Every class proposed below ships with its vanilla member for
  this reason.
- **Scope neighbors, not ours**: marker beds ride the volcanism design
  (earth-processes § 2); smelting/economy/tools are separate passes;
  prospecting *actions* (panning as a mechanic) are gameplay design — this
  pass provides what those actions will read.

## 2. The v1 roster proposal

The shape of the proposal in one sentence: **two ores ride entirely on
BUILT vectors and ship whole (bog iron, redbed copper); two are
facies/stratum classes needing one small, honest deep-sim addition each
(evaporite, banded iron); the flagship (orogenic gold) splits into a BUILT
placer half and a lode half that is the first consumer of the U8 axes —
with an explicit fork the user must pick (§ 6 R1); coal is done.**

Per the 0026 method, every class gets exactly one vanilla member, and each
class documents its future-discriminating axis without shipping a ladder
the data cannot back.

### 2.1 The roster table

| ore | class (new unless noted) | vanilla member | genesis vector | expresser status | form(s) | the player read |
|---|---|---|---|---|---|---|
| coal | `dc:stratum/organic-coal` (shipped) | `dc:geo/coal` | biofacies routing (swamp→burial) | BUILT (shipped 0026) | whole voxel | lowland stack; seams under flood caps |
| placer gold | `dc:ore/placer` (shipped) | `dc:geo/gold-dust` | settle-energy sorting in alluvium | BUILT (shipped 0007) — **presence term source-blind, see § 8.2** | loose partials in gravel | gravel bar → pan → walk upstream |
| lode gold | `dc:ore/orogenic-vein` | `dc:geo/gold-quartz` | exhumed-core hosting via `exhum`/orogenic history | PARTIAL (axes populated post-U8; no collapse consumer; fracture geometry STUB) | pore partials in basement host; 8/8 saturation = rare massive | old worn belt, high-grade core, grade zoned outward |
| bog iron | `dc:ore/bog-iron` | `dc:geo/bog-iron` | biofacies routing (Peat/waterlogged organic units) | BUILT (the coal pattern, inclusion form) | loose nodule partials in peat/organic-soil host | wetlands, rusty seeps; the starter iron |
| banded iron | `dc:stratum/banded-iron` | `dc:geo/banded-ironstone` | earliest-chapter marine chemical sediment on cratons | PARTIAL (chapter+env+province BUILT post-U8; ocean chemistry STUB — proxy gate, heir named) | whole voxel (formations survive the sieve trivially) | deep cuts in old shield country |
| redbed copper | `dc:ore/redbed-copper` | `dc:geo/redbed-copper` | arid coarse clastic unit adjacent to organic (reductant) unit in the same column record | BUILT (record-adjacency is a pure function of the ledger) | pore partials in the clastic host at the contact | red rock country → the dark bed → the green stain |
| evaporite | `dc:stratum/evaporite` | `dc:geo/rock-salt` | arid closed-basin (lake) deposition written into the record | PARTIAL (arid tags + lake mask BUILT post-U8; the deposition process needs a small deep-sim slice — § 2.4) | whole voxel | dead basin floor, no outlet, arid all the way down |

### 2.2 Gameplay-impact + fidelity trace (mandatory per session rule; per ore)

Walk the readouts, not the fields:

- **Coal** (baseline, shipped): the player digs a soft black seam whose
  position is a swamp's biography; smash resistance 2.2 vs granite 5.5 —
  it yields to a tool the sandstone above ignores.
- **Placer gold**: already visible (0010 photograph — warm cells in
  sandstone patches along the bedding). What this pass changes: nothing
  mechanical; § 6 R2 decides whether *where* it appears becomes a true
  statement about upstream geology, which converts a texture curiosity
  into the game's first navigation-by-geology loop.
- **Lode gold**: Earth mechanism — metamorphic devolatilization veins
  exposed by exhumation (tier: deep-time tectonics + collapse selection).
  Player-visible: basement in old belt cores stops being uniform granite
  pink; occasional quartz-white flecks with rare warm cells ride in it,
  denser toward the belt interior. A player who has learned "old worn
  mountains, cooked rock" digs there and is right. Without it, the
  upstream walk from a placer terminates at nothing (§ 8.2).
- **Bog iron**: Earth mechanism — groundwater iron oxidizing at wetland
  margins (tier: biofacies routing at collapse). Player-visible: digging
  peat/wetland soil occasionally yields rusty nodule eighths (sieve/dig
  extraction order puts them after the light organics — the S8 ordering
  does the panning). Iron stops being "find a mountain" and becomes
  "know what a bog is" — the historically correct starter metal.
- **Banded iron**: Earth mechanism — Precambrian ocean chemistry on
  cratons (tier: deep record, oldest chapters). Player-visible: a
  distinct banded rock deep in shield-country cuts; a major smeltable
  iron source whose *location rule* (old flat craton, not young belts) is
  learnable and true. Also the first material whose look announces
  *age* — bands = deep time.
- **Redbed copper**: Earth mechanism — oxidized basin brines reduced at
  buried organics (tier: pure record adjacency at collapse).
  Player-visible: in red arid sandstone country, the grey/dark organic
  horizon carries green-stained ore partials at its contact. Teaches the
  game's core skill — read the *stack*, not the surface: the ore is at a
  **contact**, and contacts are what cut faces are for.
- **Evaporite**: Earth mechanism — closed-basin brine concentration
  (tier: small deep-sim deposition rule + record). Player-visible: white
  beds in dead-basin fills; salt as a findable, mineable material with
  obvious future gameplay (preservation, trade). Deserts gain a payoff
  that only exists *because* the aridity is real.

### 2.3 Class contracts and documented axes (the rank-ladder discipline)

Each class documents the axis a richer pack would discriminate on, per the
0026 precedent, without shipping members the data cannot separate:

- `dc:ore/orogenic-vein` — discriminating axis: **peak P/T + exhumation
  depth** (grade of host, fertility of vein). One member now; a pack adds
  e.g. stibnite/arsenopyrite associations along the same axis later.
- `dc:ore/bog-iron` — axis: **maturity/induration** (loose ochre → pan →
  nodular bed). One member.
- `dc:stratum/banded-iron` — axis: **oxidation facies** (oxide vs
  carbonate vs silicate BIF). One member.
- `dc:ore/redbed-copper` — axis: **redox zonation** (chalcocite →
  bornite → chalcopyrite outward from the reductant). One member.
- `dc:stratum/evaporite` — axis: **brine concentration order**
  (carbonate → gypsum → halite). One member (rock salt); gypsum is the
  *first candidate second member* the day the record carries a
  concentration/residence discriminator (§ 6 R4).
- `dc:ore/placer` — axis: already implicit — **settle energy**; any dense
  inert mineral a pack registers placers correctly with zero new code
  (cassiterite would, the day tin has a source process).

### 2.4 The two small process additions this roster asks for

Both are deep-sim slices, named here as scope the roster implies (spike/
milestone sizing is not this document's job):

1. **Evaporite deposition** (for `dc:stratum/evaporite`): where a deep
   cell is a **lake** (depression-filled) AND **arid** at deposition
   time, the recorder writes an evaporite-facies unit instead of a
   clastic one (rate: small constant vs the clastic budget — coarsen the
   cause). This is the coal pattern for chemistry: a real process,
   recorded, then routed at collapse. The alternative — painting salt at
   collapse time onto arid closed basins — is rejected by "no ore before
   its process" and by the Expression doctrine (the ledger would not
   hold the cause). Needs a facies value the recorder can carry —
   either a new `DepTag` axis (chemical facies) or a `Biofacies`-sibling
   enum; data-model choice for the implementing pass, with the 0026
   merge-explosion lesson attached (measure record growth).
2. **BIF gating** (for `dc:stratum/banded-iron`): a selection rule at
   collapse reading axes that already exist post-U8 — unit is **Subsea**,
   unit's **chapter is the earliest** (chapter 0, optionally 0–1), column
   province is **Craton**. No new sim process; the *honesty caveat* and
   its heir are in § 3 and § 8.3.

Everything else in the roster needs **zero new process** — bog iron and
redbed copper are routing rules over the existing record, lode gold is a
selection rule over the U8 axes, placer is shipped.

## 3. The genesis-vector honesty table

Per the Expression doctrine: what places each ore, whether that expresser
is real today, and what the upgrade buys. **BUILT** = the recorded cause
exists and the expresser reads it. **PARTIAL** = the cause is recorded but
the reader is missing or a component is proxied. **STUB** = the cause is
not simulated; anything standing in must be loudly temporary with a named
heir (stubs.md).

| ore | placement vector | status today | what v1 does | the heir / what the upgrade buys |
|---|---|---|---|---|
| placer gold | settle-energy sorting in graded alluvium | **BUILT** (0007) | ships as is | — (mechanism is real) |
| placer gold — *presence* | "is there gold in this river's sediment supply at all?" | **STUB** — today every discharge-qualified river carries gold; presence is field-blind | ride as-built, **listed in stubs.md** (owed — § 7) | heir: upstream-endowment conditioning — scale placer eighths by lode fertility integrated over the upstream catchment (drainage export `recv`/`area`, post-U8). Buys: the upstream walk is *true*; barren rivers exist; "famously rich gold country" becomes geography |
| lode gold | exhumed metamorphic core hosting | **PARTIAL** — `exhum`/`t_crust`/chapter table populated (U8); no collapse consumer; metamorphic-grade classes unbuilt | § 6 R1 fork: disseminated-inclusion form gated on `exhum` + orogenic history now, or defer to the metamorphic slice | heir 1: metamorphic-grade classes (Sequenced) — the host *rock* becomes schist/gneiss, so gold country looks like gold country; heir 2: erosion-supply calibration — gives `exhum` legible dynamic range (§ 8.1) |
| lode gold — vein *geometry* | fracture networks in the deforming core | **STUB** — no deformation expression at collapse (layer-cake) | not faked: v1 form is disseminated pore partials (grade zonation without planar veins); no painted vein shapes | heir: tectonic expression slice (a) — dip/fold/fault at collapse; veins become planar features along the derived structural grain; the fault-offset lode-reading puzzle arrives with it |
| bog iron | wetland+biotic precipitation | **BUILT as routing** — `Biofacies::Peat` (and waterlogged organic soil) is the recorded cause; the microbial oxidation step is legitimately coarsened *into* the facies (coarsen the cause) | ships, pending the § 5 census | heir: none needed for honesty; a future groundwater/seep sim would refine *within-wetland* placement |
| banded iron | marine chemical precipitation from an iron-rich early ocean | **STUB for chemistry**, BUILT for the gate axes (chapter/env/province) | earliest-chapter+Subsea+Craton proxy gate, loudly documented; § 8.3 reports the register collision | heir: epoch-indexed paleo-ocean chemistry (same family as the sea-level-sinusoid stub #9) — the gate stops being a proxy and becomes a read of a recorded curve |
| redbed copper | reduction of basin brines at organic contacts | **BUILT** — arid+coarse clastic units and organic units are both recorded; adjacency in one column's stack is a pure function of the ledger (an honest enhancement by the doctrine's own test) | ships, pending the § 5 census | heir: a real fluid-flow/diagenesis pass would move ore *laterally* along aquifers (today it sits exactly at the recorded vertical contact — a stated coarsening, not a fake) |
| evaporite | closed-basin evaporation | **PARTIAL** — aridity recorded, lake mask exported (U8); the precipitation event is not yet written into the record | § 2.4 slice 1 adds it as a real recorded process; ships after | heir: none beyond the slice; sea-level cycles already stack beds via the existing machinery |
| coal | swamp→burial | **BUILT** (shipped) | — | rank ladder waits for kilometre-scale burial (0026) |

The doctrine consequence, stated plainly: **v1 paints no appearance where
no cause exists.** No vein shapes without deformation expression; no salt
without a recorded evaporation event; no BIF without at least an
age+environment+province gate that is a deterministic read of the ledger
— with its proxy status and heir declared out loud.

## 4. Forms — what survives the 0.9 m sieve (charcoal-style reasoning)

The 0026 lesson: literature thickness vs the voxel decides whole-block vs
inclusion, and the verdict is a *measurement*, not a vibe. No census can be
run from this pass (doc-only); each row states the literature-based
expected verdict and is marked **needs-measurement** where a probe must
confirm before the member ships (§ 5).

| ore | real-world unit thickness | expected sieve verdict | v1 form(s) |
|---|---|---|---|
| coal | seams 0.5–30 m | survives (measured: 89.6 % of seams) | whole voxel (shipped) |
| placer gold | grains, in a coarse body metres thick | host survives; ore is grains by nature | loose partials in gravel/sandstone (shipped) |
| lode gold | individual veins cm–2 m (mostly < 0.9 m); vein *networks* span tens–hundreds of m of host | a single vein FAILS the sieve; the mineralized volume survives | pore partials in basement host (the charcoal/olivine form — exactly what "form follows provenance" prescribes for sub-voxel reality); 8/8 saturation is the rare bonanza voxel, no special rule |
| bog iron | pans/nodule beds 0.02–0.5 m, rarely 1 m | expected: FAILS the sieve almost everywhere — **needs-measurement** (census of qualifying wetland units) | loose nodule partials inside peat / organic-soil voxels; NO whole-voxel member unless the census surprises |
| banded iron | formations 10s–100s m (individual bands mm–cm; the *formation* is the unit) | survives trivially | whole voxel; the mm-scale banding is texture (pack art), not sub-voxel content — the bands are below the sieve *and* below gameplay relevance, which is what texture is for |
| redbed copper | mineralized horizons 0.3–3 m at the contact | marginal — expect a mix; the *host bed* survives, mineralization may be thinner — **needs-measurement** | pore partials in the clastic host, concentrated in the 1–2 voxels at the recorded contact; no whole-voxel form |
| evaporite | bed sets 1–100s m | survives (pending the § 2.4 slice existing to measure) — **needs-measurement** on how many arid-lake cells actually accumulate ≥ 1 voxel | whole voxel |

Grade everywhere is the ratified eighths count: 1/8 = a show, 3/8 = worth
working, 8/8 = the story you tell other players. Extraction rides the
existing S8 typed-damage ordering with **zero new mechanics**: dense ore
partials separate from light host debris exactly the way gold separates
from sand in a pan, because sieve resistance ≡ grain size and settle
energy already order them. (Panning as an *action* = the sieve damage type
plus water; the action's design is the gameplay pass's, the ordering it
reads is already built.)

## 5. Needs-measurement gates (the 0026 probes, named before code)

Each is an `organic_probe`-style census over a production world, cheap,
and each has a stated kill condition — a member that fails its census is
not shipped in that form (the charcoal precedent):

1. **Bog iron census**: count Peat/waterlogged-Soil units; how many
   columns qualify, at what thickness. Kill: if qualifying wetland volume
   is so rare the starter iron cannot be "the starter iron" (compare
   peat's own near-death: 72 units world-wide), the routing widens to
   humid organic soil, or bog iron's roster seat is reported back to the
   user — not silently padded.
2. **Redbed adjacency census**: count arid coarse-clastic units directly
   adjacent to organic units in the same column stack. Kill: if the
   count is ~0 (arid basins and swamps may rarely share a column — S10
   put coal on arid lowlands, which *helps* here, § 8.4), the vector is
   reported and the ore deferred rather than the gate loosened.
3. **`exhum` dynamic-range histogram** (tectonic-history world): does the
   exhumed-core gate discriminate anything at shipped erosion rates
   (S12 says metre-scale — § 8.1)? Kill/defer input to § 6 R1.
4. **Earliest-chapter Subsea craton census** for BIF: does the gate
   select a non-trivial, non-everywhere set of units?
5. **Evaporite accumulation** (after § 2.4 slice 1): arid-lake cell count
   and accumulated bed thickness through the sieve.
6. **Endowment census** (all ores, per world): the soft-guarantee check —
   § 6 R6 decides what the world build does with a zero.

## 6. NEEDS RATIFICATION — the user-owned calls

Each: options, a recommendation, and what it will look like in game (in
plain language, per the standing rule).

**R1 — Lode gold: ship the minimal honest form now, or defer to the
metamorphic slice?**
- *Option A — ship now (disseminated form):* a new `dc:ore/orogenic-vein`
  class whose member is emplaced as pore partials in basement, gated on
  the U8 axes (`exhum`, orogenic chapter history) — the olivine machinery
  reading real recorded causes. No vein shapes (that expresser is a stub).
  Caveat: S12 measured metre-scale exhumation, so until the
  erosion-supply calibration lands the gate may barely discriminate
  (§ 8.1) — probe 3 measures this first.
- *Option B — defer:* gold exists only as placer until the
  metamorphic-grade slice ships; the upstream walk dead-ends (loudly
  documented).
- *Recommendation:* **A, conditioned on probe 3** — if `exhum` has usable
  range, ship; if not, A collapses to B with the measurement on file.
  The flagship chain is the ratified reason gold is in v1.
- *In game:* today, every deep dig in mountain cores shows uniform pink
  granite. Under A, in the worn-down heart of an OLD mountain belt, the
  deep rock carries occasional white quartz flecks with rare warm gold
  cells among them — more of them the deeper into the old core you go.
  Standing in a young sharp mountain range you'd find none, and that
  difference is the game teaching you real geology. Under B, nothing
  changes underground yet; gold stays a river-gravel phenomenon.

**R2 — Placer presence: leave source-blind (status quo), or condition on
upstream endowment once R1-A exists?**
- *Option A — status quo, listed as a stub:* every sufficiently large
  river carries some gold; rich vs poor is energy zonation only.
- *Option B — condition presence/abundance on the upstream catchment's
  lode endowment* (drainage export gives the catchment; the lode field
  gives fertility; the scaling is a pure function of both — an honest
  enhancement).
- *Recommendation:* **B, in the same milestone as R1-A** (it is a small
  multiplier on an existing pass, and it is the payoff half of the
  flagship). If R1 defers, A rides with a stubs.md entry either way.
- *In game:* under A, panning any big river eventually shows color, so
  rivers don't differentiate. Under B, most rivers pan barren; the one
  draining old-belt country pans rich, gets richer upstream, and the
  flakes STOP above the vein outcrops — the exact moment the
  things-that-will-happen line promises: you read the river, and walk
  upstream.

**R3 — BIF gating: earliest-chapter record proxy, or pre-record basement
lore?**
- *Option A — earliest-chapter proxy (in the record):* Subsea + chapter 0
  + Craton units collapse to banded ironstone. Gets real strata geometry
  (beds, contacts, burial) and honest age labels; carries the § 8.3
  anachronism loudly (BIF inside life-adjacent time).
- *Option B — pre-record basement flavor:* BIF as a craton *basement*
  member ("procedural hacks for the boring billion" — the sanctioned
  lore-below-the-record lane), no deposition history, present as deep
  basement bodies in cratons.
- *Recommendation:* **A** — a chemical sediment should read as sediment
  (bands, lateral extent, a floor and a roof); the anachronism is a
  documented calibration compression, same family as advection-scale
  compression, and the paleo-ocean-chemistry heir subsumes it.
- *In game:* either way, deep cuts in old flat shield country reveal a
  striking red/grey banded rock that smelts to iron. Under A it lies as
  proper beds low in the stack with everything younger on top of it —
  you can follow the horizon. Under B it appears as bodies inside the
  deep basement without a stratigraphic story.

**R4 — Evaporite member: rock salt only, or salt + gypsum?**
- *Option A — rock salt only:* one member; the class documents the
  brine-concentration axis for later.
- *Option B — both:* gypsum precipitates before halite in reality, but
  the record carries no brine-concentration axis, so member choice
  between them would be a coin flip wearing a lab coat — exactly the
  rank-ladder fiction 0026 refused.
- *Recommendation:* **A** (rock salt — the gameplay-heavier material),
  by direct application of the no-ladder precedent.
- *In game:* dead arid basins carry white beds you can quarry for salt.
  Under B you would *also* see a second white mineral whose placement
  relative to the salt would be arbitrary — plausible-looking, but a lie
  the first time a player asks why.

**R5 — Redbed copper appearance: how loud is the green?**
The ratified visuals doctrine says ore is subtle (no glint, close-range
speckle). Malachite staining is the one v1 ore whose real-world tell IS a
vivid color. Options: (a) stain only the ore partials themselves (doctrine
default — green cells in the dither, nothing else); (b) allow the member's
albedo to tint slightly beyond its eighths share via the pack height
channel (the ratified ore-clumping knob — still deterministic, pack-owned).
- *Recommendation:* **(a)** to start; (b) is a pack-side knob the user can
  turn after seeing (a) in a cut face — no code either way.
- *In game:* under (a), you notice the green only when close to a cut
  face, as scattered green pixels along the dark horizon in red rock.
  Under (b), the horizon reads faintly green from a few metres further
  out. Neither ever glints at distance.

**R6 — The endowment guarantee mechanism** (the ratified *principle*
needs a mechanism):
- *Option A — census-and-warn:* world build runs the § 5.6 endowment
  census; a missing ore raises a named warning (the pack-degradation
  pattern: loud, playable, never silent).
- *Option B — endowment-aware retry:* world creation re-salts specific
  gen draws until every v1 ore is present at Medium+.
- *Option C — nothing (document only).*
- *Recommendation:* **A now** (doctrine-consistent, cheap, and the
  census is wanted anyway), with B held until a measured world actually
  fails the census — building a retry loop for a failure mode nobody has
  observed is bandaid-shaped.
- *In game:* under A, a rare unlucky world tells you at creation "this
  world has no known banded iron" — which is itself information a
  hardcore player may enjoy. Under B, every Medium+ world quietly has
  everything, always.

**R7 — The roster confirmation itself.** The roster was ratified
2026-07-20; this pass's § 8 reports two collisions discovered since (the
S12 exhumation finding against lode gold; the Phanerozoic register against
BIF). Confirming the roster *with* those caveats — or amending it — is the
user's call. Recommendation: confirm, with R1's probe-conditioned shape
and R3-A's documented anachronism; both caveats have named heirs already
Sequenced.

## 7. Rides-as-built / needs-measurement (integrator triage)

**Rides as-built** (interim mechanisms, no bandaids, per standing rule):
- The source-blind placer presence term — **owes a stubs.md entry in the
  implementing commit** (heir: R2-B upstream conditioning). Under the
  Genesis addendum an unlisted stub is an inventory defect; the listing
  is the fix available today, R2 is the subsumption.
- Coal, exactly as shipped (0026), including no rank ladder.
- The visuals-doctrine ore subtlety defaults, pending R5.

**Needs-measurement** (before the corresponding member ships — § 5):
bog-iron census (1) · redbed adjacency census (2) · `exhum` histogram
(3, gates R1) · BIF gate census (4) · evaporite accumulation (5) ·
endowment census (6, feeds R6).

## 8. Where the priors collide (reported, not resolved)

1. **Ratified flagship vs measured exhumation.** geology.md (2026-07-20)
   ratifies orogenic gold as "veins in exhumed metamorphic cores → placer
   downstream," the v1 flagship. S12 (same week) measured **metre-scale
   exhumation at shipped erosion rates** — the exhumed-core signal the
   lode gate must read may have almost no dynamic range until the
   erosion-supply calibration (Sequenced) lands. The roster decision does
   not reference the finding; R1 is written to absorb it (probe first),
   but the tension is the user's to see.
2. **things-that-will-happen vs the shipped placer.** "The vein they
   eroded out of is somewhere upstream — so you read the river, and walk
   upstream" — but the shipped placer is source-blind (gold in every
   discharge-qualified river; `geology.rs::placer_pass` conditions on
   energy only) and there is no vein to find. Under the Expression
   doctrine (decided *after* the placer shipped) the presence term is a
   field-blind constant — and it is **not in stubs.md** (an inventory
   defect by the doctrine's own rule). § 7 owes the entry; R2 is the fix.
3. **Ratified BIF vs the ratified Phanerozoic register.** "Banded iron on
   old craton chapters" (2026-07-20) collides with "the recorded span
   calibrates to ~500 Myr, every recorded unit is life-adjacent time"
   (2026-07-19): Earth's BIF is overwhelmingly pre-Phanerozoic, so
   in-record BIF is an anachronism and pre-record BIF has no strata. Both
   decisions are user-ratified; R3 offers the reconciliations.
4. **An accidental gift, noted as such**: S10's waterlogging model puts
   coal swamps on **arid lowlands** (the honest 0026 finding), which
   makes arid-clastic-over-organic columns — redbed copper's exact trap
   geometry — *more* likely than Earth-intuition suggests. Probe 2 will
   quantify it; flagged so nobody later "fixes" the aridity tag and
   silently kills the copper without noticing the coupling.
5. **Doc hygiene**: geology.md § v1 content (DECIDED 2026-07-18) still
   says v1 has "one ore vector (placer)"; the 2026-07-20 ore decision
   supersedes it but the older line was never annotated. Also the
   ratified roster calls evaporites an "ore" while the backbone's
   candidate-class roster files evaporite under chemical sediment — this
   pass uses `dc:stratum/evaporite` (it is a rock you quarry, like coal),
   which follows the coal precedent rather than the roster's word choice.

## 9. Scope fence — what v1 ores explicitly are NOT

- **No smelting, refining, metal economy, or tool balance** — separate
  passes; this pass ends at "a distinct raw material exists in the world
  where its process put it, extractable by the existing typed-damage
  machinery."
- **No marker beds** — they ride the volcanism design (earth-processes
  § 2); the fault-offset seam puzzle arrives with tectonic expression +
  volcanism, not here. (Interface note: when marker beds land, the
  lode/fault read of § 3 gains its correlation tool for free.)
- **No new prospecting mechanics** — panning/assaying actions belong to
  gameplay/knowledge design; this pass guarantees the *world-side truth*
  those actions will read (§ 4's extraction note).
- **No new property-sheet axes** — every proposed member fills the
  existing axes (density, grain size, settle energy via the sheet,
  per-damage extraction resistance, albedo). Nutrient/chemistry axes are
  explicitly not added (the Retro precedent).
- **No display naming** — culture/language layer (ratified).
- **No deferred-engine ores** — tin/tungsten, porphyry copper,
  fault/hydrothermal veins stay deferred with their engines (ratified);
  nothing here pre-builds for them beyond the class-contract discipline
  that makes their later arrival additive.

## 10. Open questions

1. The evaporite facies' data model — new `DepTag` axis vs `Biofacies`
   sibling — and its measured record-growth cost (the 0026 merge lesson).
2. Whether lode-gold emplacement should also gate on host class once the
   metamorphic roster exists (vein fertility differs by host grade in
   reality) — waits for that slice; the class documents the axis.
3. Whether the redbed rule should read adjacency strictly (touching
   units) or within-N-metres of the contact — probe 2's histogram will
   suggest the honest reading.
4. Placer generalization: when a second dense mineral exists (e.g. a
   future cassiterite), does `dc:ore/placer` select per-locale among
   members by source availability (needs R2-B machinery generalized) or
   by fitness alone? Deferred until a second member is real.
5. Whether bog iron should regenerate in player-era time (real bog iron
   re-accumulates in decades–centuries — a lovely renewable-resource
   hook) — belongs to the shallow-time materials loop, noted for it.

## Cut list (considered for v1 and not proposed, with reasons)

- **Tin/cassiterite** — no source process (plutons/greisen unbuilt);
  its placer half would be free the day a source exists (ratified
  deferral upheld).
- **Porphyry copper** — needs intrusion emplacement events (recorder
  event-entries) and contact-aureole geometry; deferred with its engine
  (ratified deferral upheld).
- **Galena / lead-silver** — the classic form is fault/hydrothermal
  veins; fracture networks and fluid flow are STUB (ratified deferral
  upheld).
- **Native copper** — flood-basalt hosted; punctuation events are hooks
  only.
- **Gypsum as a second evaporite member** — the record never visits the
  brine-concentration axis (R4, the 0026 no-ladder rule).
- **A distinct "rich massive gold" member** — 8/8 partial saturation
  already IS the massive form (grade-is-eighths, ratified); a separate
  member would duplicate a mechanism.
- **Charcoal-anything** — settled by measurement in 0026; not revisited.

## R8 — MEMBER NAMING CONTRADICTS THE RATIFIED REPRESENTATION (added 2026-07-21, integrator, after the texture pass failure)

Found the hard way: this doc's members are named as DEPOSITS (`dc:geo/gold-quartz`,
`dc:geo/redbed-copper`) while geology.md § Ore (DECIDED 2026-07-20) ratifies ore
as a SUBSTANCE inside a HOST via the eighths/partial system — the composite look
must EMERGE from the mix (grade-is-eighths), never be baked into a member. The
first texture pass generated deposit portraits from these names and had to be
redone as base substances (journal/0048). **Proposed amendment (user call):**
rename members to substances — `dc:geo/native-gold`, `dc:geo/malachite`,
`dc:geo/limonite` (bog-iron's nodule substance) — with the deposit geometry
(vein, stain, nodule-in-peat) supplied by emplacement context + host, exactly as
gold-dust-in-gravel already works. `banded-ironstone` and `rock-salt` stay
whole-voxel ROCK members (their internal structure is sub-voxel reality,
legitimately in-texture — the carbonaceous-mudstone precedent).
