# Material genesis, fluid, and pass purity — the open-edge notebook

> **⚠ THIS IS AN OPEN EDGE, NOT A DECISION RECORD.** Opened 2026-07-26 from a long
> design conversation. It follows `earth-processes.md`'s method rule 1 — *"field
> notebook first: every wanted feature begins as a written causal account, before any
> code"* — and `biomes-ecology-agenda.md`'s precedent of a doc that is explicitly a
> conversation, not a conclusion.
>
> **Read the status column before quoting anything here.** Three tiers:
> **RATIFIED** (user-decided; the authoritative copy lives in the named doc),
> **PROPOSED** (discussed favourably, *not* decided — assistant-originated unless
> marked), **OPEN** (genuinely unresolved, or an assumption already falsified).

**Why this doc exists.** The conversation crossed four arcs that each own part of the
answer and none of which owns the whole: `material-behavior.md` §12 (creation vs
transformation), `flow.md` (fluid, load, transport), the ROADMAP genesis-passes arc,
and `north-star.md` (the core/plugin boundary). A reader arriving at any one of them
needs the others. **Every one of those four now points here.**

---

## 1. What was RATIFIED (2026-07-25/26) — recorded elsewhere, summarised here

These are **decided** and their authoritative home is `material-behavior.md` §12.

- **The discriminator is RESOLUTION, not phase.** *Can the prior be named as a material
  we track?* — not *"is the prior diffuse."* The old phase test (*"does it precipitate
  from something diffuse"*) is retired to a heuristic: it makes a claim about **nature**
  that is false (ion-scale processes are processes everywhere), where the resolution
  test makes a claim about **our model**, which is what we can defend. *(user)*
- **The material ontology is a SIEVE.** Some origins genuinely *are* transformations —
  of the **molecular parts** of materials — and our model has no molecular parts. A
  crystalline inclusion is not `rock→crystal`. **That is the whole case for genesis.**
  *(user — this is the thread's founding insight and it is not derivable from the
  older corpus.)*
- **The four-way test**, with category 2 supplied by the user: transform (driver
  built) · **transform with a SEAMED driver** · transport · genesis.
- **The pathology: filing a (2) as a (4) is irreversible** — a formation predicate
  deletes the edge, and with it the identity and mass chains.
- **The floor MOVES.** Genesis is relative to a declared ontology and rises as systems
  land. Drivers already declared intended are **presumed seamed**; it remains a
  per-pass conversation.
- **Only the temporal dropoff (t=0) is permanently legitimate**, which is why
  `stubs.md`'s affirmed list correctly contains only t=0 and boundary items.
- **The completeness test** — every material traceable to (a) t=0, (b) a formation
  predicate reading recorded conditions, or (c) a chain from those. **Traceability
  audited first; four-way classification later**, once the declarative-pass migration
  has moved the out-of-band cases. *(user)*
- **Genesis reads fields; a proxy reads tags.**
- **Who owns clastic facies — SETTLED.** Genesis makes the *parent* honest · weathering
  the *loosening* · transport the *destination*. Recorded at `material-behavior.md`
  §13.7.

---

## 2. The PROPOSED shape — the live edge

**Status: PROPOSED, not ratified.** Assistant-derived except where marked. Nothing in
this section has been built or scheduled.

### 2.1 Pass purity — passes are material-agnostic *(user-originated)*

> *"Passes should be pure in that they are material agnostic: transforming and moving
> materials along the edges they already have… modelling a local or global force, a
> driver."* — user

This turns out to be **already the ratified north-star shape**, stated as the content
layer's cellular pass: *"select materials matching **predicate P** in cell context C,
apply transform T at rate R."* A pure pass selects by predicate; it never names a
material.

**And the running engine violates it. Measured 2026-07-26:**

```
biotic.rs:177      pub const COAL_ONSET_C: f64 = 22.0            # material-specific threshold IN a pass
inventory.rs:1294  pub const BEDROCK_SEAM_MATERIAL = GRANITE     # stub #16
inventory.rs:1369  basement: MaterialId::GRANITE
```

**PROPOSED WORK — a pass-purity audit.** Grep every pass for `MaterialId::`,
`MatRepr::`, and material-named constants; each hit is either an impurity to socket or
a justified carve-out to record. Cheap, concrete, and it measures how far the running
engine has drifted from the ratified shape. *Not yet sequenced.*

### 2.2 TERM-KEYED edges — the load-bearing idea

The corpus's behaviour slots are **named hookups**: a weathering pass asks each material
for `weather→`; pass and material agree on a name. The proposal is different:

> **A pass selects edges by the TERMS they carry, not by a name they share.** The
> transition's name is a token for authors and heirs — *not* something a pass keys on.
> *(user: "transition name not being known to our passes, just a token for reference by
> leaf materials.")*

**Two payoffs.**

1. **It solves competitive ordering.** Several materials drawing from one depleting pool
   in an order set by a continuous physical property — evaporite sequences (solubility),
   Bowen's reaction series (liquidus temperature) — cannot be ordered by topo-sort,
   because the order is a *scalar comparison*, not a dependency. A pure pass evaluates
   each candidate edge's **ordering key** at the cell, sorts, and draws the pool down in
   order. **A modder's new mineral slots into the sequence with one declared number.**
2. **It dissolves the topo-sort problem entirely.** A pass declares reads over
   **fields**, not materials — so **adding a material never changes any pass's
   read-set.** No unions, no false cycles, no scheduling failure caused by an unrelated
   leaf.

**This is why the earlier "material-as-pass vs. environment-pass" fork dissolved:** both
were answers to a problem that only exists under slot-keying. The resulting shape is
**the pass is the driver · the material is data · participation is derived from declared
properties** — which is verbatim what the ROADMAP genesis arc already asks for
(*"derive fitness purely from properties stored on each material… adding a leaf → it
participates automatically"*).

**Consequence:** genesis stops being a pass *type* and becomes an **edge whose source is
a field/environment rather than a material** — same declaration shape, same runner, one
fewer concept.

### 2.3 Per-parameter shadowing, with parent-value access *(user-originated)*

> *"`transition["to_calcite"].min_temp = x` — shadow one term, inherit the rest. First
> ancestor wins up the tree **for params instead of entire edges**. I can nullify a
> transition on a leaf this way too."* — user

Strictly better than edge-level shadowing (`north-star.md`'s *"children shadow
parents"*) and better than rate composition, because it **needs no new rule** — it is
the existing first-ancestor-wins, resolved at finer granularity.

- **Composition is still reachable** *(user)*: if the SDK exposes
  `parent.transition[name].param` **as a readable value**, a leaf's own function can
  compute from it. So we get composition *without* composition-order semantics — the
  leaf does the arithmetic explicitly.
- **REQUIREMENT this creates:** parameters need **three** states — *inherit*
  (unspecified) · *override* (a value) · *disable* (explicitly nothing). "Unspecified"
  and "specified as nothing" must be distinguishable, or a leaf cannot null a
  transition. Same distinction as `Identity::Unrecorded` vs. an empty value — a shape
  we already got right once (journal/0101).

### 2.4 Candidate engine primitives — derived bottom-up from worked examples

| primitive | shape | earned by |
|---|---|---|
| threshold on a field | scalar vs. field value | pyrolysis `min_temp` |
| **ordering key** | scalar, **comparable across edges the engine has never seen** | evaporite sequence, Bowen's series |
| response curve | `f(field…) → rate` | solubility, temperature-scaled rate |
| yield / stoichiometry | mass fraction in→out | charcoal ≈0.25 |
| source binding | material · fluid · field/environment | all of them — this is what makes genesis an edge type |
| affinity over composition | match against a tracked mixture | emerald host rock; the sieve's edge |

**The ordering key carries the architecture.** It is the only primitive that must be
*comparable between edges the engine has never heard of*, and it is what lets a pure
pass sequence a third party's mineral correctly.

**Terms should stay FUNCTIONS, not a closed formula vocabulary.** A liquidus temperature
is pressure-dependent, so the "key" is a function evaluated at the cell, then sorted.
This is compatible with `north-star.md`'s behaviour slots (already functions) and avoids
committing to a `linear|exponential` formula enum.

**PROPOSED boundary for "engine owns primitives":** the engine owns **primitive** terms
(scalars over cell context) the way it already owns field-solver primitives; **a pack may
declare derived terms together with the passes that read them.** Since passes are content
(`north-star.md`: *"every pass is content — including tectonics"*), this keeps the
vocabulary open at pack level without opening the core, and the crossing constraint holds
(a term is an opaque id + plain data).

### 2.5 Two findings about the SDK as a *product*

**(a) Plug-and-play means the identity default is INERT, never ERROR.** A material
declaring **no** edges is legitimately inert and still fully present — placeable,
mineable, renderable. A material declaring an edge **no pass walks** should be **loud**,
per `north-star.md`'s *"mods declare their SDK version and their reads/writes, so a store
can validate compatibility."*

**(b) The plug-and-play mechanism IS parent inheritance — so our default pack's PARENT
materials are a product surface.** A shallow modder's minimum viable mod is *choose a
parent*:

```
material mymod:azurite { parent: dc:mineral/vug_filling }   # inherits the parent's transitions; appears in vugs
material mymod:azurite { parent: dc:mineral }               # inherits nothing useful; never occurs naturally
```

This reframes materials.md's *"packs add classes (parents), not only members (leaves)"*
from a **permission** into an **obligation on us**: a shallow modder's entire experience
is which parent they pick, so our parents must ship good, well-termed edges. **Nothing in
the corpus says this, and it is the most actionable finding in the thread.**

**(c) Interop and interference are the same mechanism.** Term-keying means a third
party's pass automatically picks up *our* materials wherever ours declare compatible
terms — emergent interop, and emergent interference (their "cryo-fracturing" pass
shatters our granite because our granite honestly declares cohesion). `ctx`-as-capability
is the obvious hook (a pass declares reads; perhaps also a material *scope*). **OPEN.**
Not urgent while we are the only pack.

---

## 3. OPEN — fluid, solutes, and an assistant assumption the user CORRECTED

**This is the least settled part of the notebook and the most likely to mislead.**

**What is true in the tree.** `InvForm` is the closed set
`{Structure, Loose, PoreFill, Fluid, Void}` — **`Fluid` is already a form**, not a
material. `flow.md` §1.1 builds on exactly this: *free* = fluid in open space, *bound* =
fluid in pore space, and **free↔bound is an edge on the form-transition graph (S-8)**.

**What is NOT true, and what the older corpus will mislead you into assuming.** Priors
reflect an era of *"there is only water, maybe magma."* `InvForm::Fluid` is documented
*"accommodated, never stored by the identity default — the water model derives it"*, and
**`material-behavior.md` §12's own open list already carries *"Fluid as a stored role vs
derived."*** So fluid-as-stored is unsettled, not assumed.

**The user's direction — and it is ALREADY RATIFIED in `flow.md` §2.5**, which the
assistant had not connected when this conversation began:

> *"We are working toward fluid as one **form** of a material, just like loose and
> structure… free/bound and flow could be generalised systems, **any material able to go
> into fluid form can ride**, water being the first example material in the default pack
> to do so."* — user, 2026-07-26

> *"The flow carries a **fluid material id**, not an assumption of water. Otherwise lava
> tubes, magma, brine, CO₂ and ice are foreclosed. **Form = occupancy; fluid = material;
> rheology = material properties** (a glacier is a solid whose viscosity is ~10¹³ — flow,
> not a special case)."* — `flow.md` §2.5, RATIFIED 2026-07-25

**So "fluid is a form, not a substance" is DECIDED.** What is open is narrower and
concrete: the **inventory** still treats `InvForm::Fluid` as *derived, never stored*, so
the record cannot yet hold "this cell contains 3 m of brine-in-fluid-form." The
*declaration* generalised before the *storage* did — which is the ordinary shape of an
evolving seam here, not a defect.

**On solutes — no established shape.** The user's working model:

> *"A solution basically would exist to deeptime in the **transport of loose in
> fluid**."* — user

i.e. a solution is the **load multiset (§13.3) riding a fluid**, not a separate solute
concept.

**🔴 CORRECTION — an assistant assumption, caught by the user before it was recorded.**
The worked examples in §4 below were written as edges `from: <fluid: brine>` /
`<fluid: hydrothermal>`, which **assumes a fluid-identity model that does not exist.**
On the user's model there is no "brine" object to have an edge from; precipitation would
be **carried load exchanging into a solid form** — an edge from *material-in-transit*,
not from a named fluid. `flow.md` §2.5 (*"the fluid has an IDENTITY"*) is the live thread
and it is **evolving**. **The examples are kept as written, with this correction attached,
because the fluid binding is exactly the part still being designed.** Do not read them as
a proposed API.

---

## 4. Worked examples — the fidelity altitude for the default pack

*(Illustrative only — **not committed syntax**, following §12's precedent. Read §3 first:
the `from: <fluid: …>` bindings are the corrected assumption.)*

### 4.1 Charcoal — terms selecting between OUTCOMES

Charcoal is **pyrolysis**, not combustion: heat *without* enough oxygen. Burn wood in air
and you get ash. The discriminator is oxidant availability — **and we have no oxygen
field.** We do have burial.

```
material dc:wood
  transition "pyrolyse" { to: dc:charcoal, min_temp: …, requires: buried,  yield: 0.25 }
  transition "combust"  { to: dc:ash,      min_temp: …, requires: exposed, yield: 0.03 }
```

**Two edges from one material, one thermal driver, discriminated by a term over a field
we already compute.** The pass names none of wood, charcoal or ash. The burial-for-oxygen
substitution is an honest coarsening ("coarsen the cause") and is **visible in the
declaration** rather than buried in pass code — which is the point.

### 4.2 Hydrothermal quartz — the easy case

`ores.md` already holds the mechanism: *"the expelled water carries dissolved silica and
it precipitates as quartz veins where pressure and temperature drop."* Driver pass reads
`{thermal, head}`, walks candidate edges, evaluates each solubility at the cell,
precipitates as the fluid cools.

### 4.3 The evaporite sequence — the case that broke the earlier answer

```
dc:gypsum   saturation_threshold: 4.8
dc:halite   saturation_threshold: 11.0
dc:sylvite  saturation_threshold: 60.0
```

Driver reads `{aridity, basin closure, water balance}`, concentrates, precipitates in
**ascending threshold order**, drawing the pool down.

> **A modder adds `mymod:borax` with `saturation_threshold: 45` and it slots between
> halite and sylvite. No pass edit. No registry. The pass never learns what borax is.**

The altitude is honest: concentration factor is evaporation-vs-inflow — dimensionless and
computable, **not ion chemistry**.

### 4.4 Emerald — the actual sieve point, and it is NARROW

Be-meets-Cr is trace chemistry we do not track and probably should not. But writing it
out shows the sieve sits lower than expected: **host lithology, provenance and fluid
identity are all things we DO track**, so the coarsening lands on a *narrow* seam rather
than requiring a whole genesis pass. (We already ship
`accessory_inclusions_ride_igneous_pores` — existing machinery, not a blank page.)

---

## 5. Open questions, ranked

1. **Fluid identity and the solute shape** (§3) — the live blocker for every
   precipitation example here. Owned by `flow.md` §2.5 + §12's *"Fluid as a stored role
   vs derived"*.
2. **The primitive-vs-derived term boundary** — what exactly is engine-owned. §2.4
   proposes packs may declare derived terms *with* the passes reading them.
3. **Is `dc:mineral/vug_filling` — a parent whose job is to be inherited — a first-class
   authoring concept?** If the shallow-modder path is "pick a good parent," designing
   that hierarchy is a **product surface**, not an implementation detail. *Asked of the
   user; unanswered.*
4. **Material scope for passes** (§2.5c) — interop vs. interference.
5. **Does the ROADMAP genesis arc's first slice survive?** If genesis is (a) an initial
   condition + (b) a name for a seamed driver + (c) folded competitive precipitation,
   then *"genesis pass"* may not be a pass type at all, and the arc's first slice should
   be re-scoped. **The three-way split of "genesis" was explicitly NOT ratified** — the
   user agreed only that **pre-loop is not a pass**, was *"NOT sold"* on ongoing
   formation being untractable as material-as-pass, and held emplacement open because it
   *"could be modelled with field+cell transformation and transport machinery."*
6. **Emplacement as transform → transport → transform.** Partial melting (T,P — both
   axes exist) → ascent (melt as a load on a potential gradient) → crystallisation
   (cooling). If that holds, **igneous emplacement needs no genesis pass at all.**
   *Assistant-proposed; the user held it open for discussion, not agreement.*

---

## 6. Pointers in

Anyone arriving from these should be sent here; all four now carry a link:
`material-behavior.md` §12 · `flow.md` §2.5 · ROADMAP *GENESIS-PASSES DRIVE ROCK
DISTRIBUTION* · `north-star.md` § the core/plugin boundary.
