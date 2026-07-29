# The dependency graph — what blocks what

**Purpose.** ROADMAP says *what order we chose*. This says **why that order is forced** —
which work is upstream of which, and what is genuinely ready to start today. It exists
because the board grew a *pickup* problem: entries were accurate and nothing converted an
accurate open item into a sequenced one, and because "is this engine or pack?" was being
answered from memory and got answered **wrong** on the highest-value item on the board
(corrections #71).

**Read it with `ROADMAP.md`, not instead of it.** ROADMAP holds the entries and their bodies;
this holds the edges. When they disagree, **ROADMAP wins on content and this wins on
sequence** — and the disagreement is a defect to fix in the same session, not to route around.

**Maintained in-session** — see § Process at the bottom. Last full pass: **2026-07-29**.

---

## 0. The partition, in the user's words

> *"This is work on the **default plugin pack** side, which is separate from the **engine/SDK**
> (which provides the primitives that plugins use to declare behaviors which the pass runner
> executes)."* — user, 2026-07-29

And read-first item 0, which settles the boundary cases:

> ***Every pass is content, including tectonics and erosion***; **pass ORDER is authored per
> world**.

**⚠ The trap this file exists to prevent.** On 2026-07-29 a cold session called the erosion
axis *"engine work"* because the previous night's close block said so, while `CLAUDE.md`
read-first item 0 said the opposite **by name**. Erosion is a **pass**; passes are **content**.
A close block is a **handoff, not an authority** (corrections #71). *If you are about to
classify something from memory, classify it from this table instead — and if it is not in this
table, that is the bug.*

**Never justify a placement by trust.** The trusted/untrusted split and ABI/WASM tiering are
**DEFERRED** (north-star § Deviations 1). There is **no difference in permission between
native and WASM**, so "the engine should own it because it's more trusted" is not an argument
that exists here.

---

## 1. ENGINE / SDK — the primitives plugins declare against

| # | thing | state | blocks |
|---|---|---|---|
| **E1** | **pass-graph kernel** — today **derives** order from `{reads, writes}` (Kahn + tie-break); validating an *authored* order is **E7** | **BUILT** (7 passes) | — |
| **E2** | **cell / record storage** | **BUILT** | — |
| **E3** | **RATE — per-pass cadence + a real `dt`** | **BUILT 2026-07-29** (journal/0123). Cadence is authored data (`CadenceTable`), sub-turns execute, `dt` is live in creep/uplift/thickening/inventory-weathering. Empty table = shipped world, hash-identical. **Follow-on BUILT 2026-07-29 (journal/0124): the `Schedule` sum type**, `deeptime::schedule` (`ARCHITECTURE.md` § Schedule; ROADMAP arc slot (f)) — seed = initial condition, epoch 0 fires for all, skip rule deleted. All three pre-loop incumbents audited to `Step`; the pre-loop blocks are gone. Terrain + strata hash-identical, the flow record's **vertical** faces moved (a fix) and gained `GOLDEN_FLUX`, which it had never had. `Seed`/`SeedAndStep` have **no production declarer** — heir is E7's sibling, declared epochs (ROADMAP slot (d)) | ~~E4's first extraction~~ **unblocked** · every pass that wants a phase length |
| **E4** | **field-solver primitives** — the S-10 gather; **the kernel owns its own stability bound** | **SHAPE NAMED 2026-07-29 (S-10), 2 instances, NOT EXTRACTED** | every future diffusing pass |
| **E5** | **refinement primitives** — coarse→fine reconstruction; **the presentation layer** | **CAUTIOUSLY RATIFIED 2026-07-29 (user)** — `docs/design/refinement.md` (record families + term-keyed operators + three laws; evidence `docs/audits/2026-07-29-refinement-coupling-priors.md`). **Members are directions, not build orders: each needs its OWN design pass against the bones, and prior member plans are superseded AS PLANS** (user, at ratification). **#0 has its design pass (`docs/audits/2026-07-29-member0-coarsefield-design.md`, rulings in its header) and its FIRST BUILD SLICE SHIPPED 2026-07-29 (journal/0125)** — the far site, K1 only; `summarize` ruled to the octree contract, move C (midpoint jitter) filed with an heir. Continuation slot: (a) the octaves `DitherSource` (co-requisite of U3, socket now worked), (b) MM-1 + MM-3 + the near-path restructure. **Live blocker for #1: flow.md's face-pairing rule, unratified** | the whole appearance cluster · `collapse.rs` decomp |
| **E6** | **open resource vocabulary** — `DeepAxis` retires; packs declare their own ids | **UNBLOCKED 2026-07-29** (E3 landed); sequenced | third-party packs |
| **E7** | **authored order + the validator** | **UNBLOCKED 2026-07-29** (E3 landed); sequenced. **It brings the per-world manifest**, which is the loader `CadenceTable` was shaped for | plugin-agnosticism |
| **E8** | **S2 statistical tier** | **HELD** — probable future primitive, zero consumers, *do not find it one* | nothing. **Gated on bio/eco, a USER call** |

**E4's rule, and it is the whole reason it is engine-side:** four inputs set the stability
threshold and they have four owners — the **stencil** (kernel), **`dx`** and **`dt`**
(engine), the **coefficient field** (content). Only the kernel can know its own constant.
Housed anywhere else, every plugin author needs a von Neumann analysis before writing a
diffusion pass. Housed in the kernel, **the unsafe call is inexpressible** — the same move as
`CoarseField` making the raw per-cell read unsayable.

---

## 2. DEFAULT PLUGIN PACK — the passes

| # | thing | state |
|---|---|---|
| **P1** | hillslope creep / erosion operator | **FIXED 2026-07-29** (journal/0122). Takes `dt` from RATE since journal/0123; still sub-cycles **in the pass** — `stubs.md` § 30's remaining half, heir **E4** |
| **P2** | `EROSION_CALIBRATION` re-pick + flag flip | **SEQUENCED.** 45 was fitted to the broken solve and inverts under the fixed one |
| **P3** | flow / hydrology — face-flux record, head field | **BUILT AND IDLE.** Zero production consumers, *on purpose* |
| **P4** | CoarseField **adoption** (U22 + U3) | **HALF SHIPPED 2026-07-29 (journal/0125), and the halves are now separately blocked.** Routed through E5 member #0; the design pass re-derived the plan and the first build slice landed the **FAR site**: `collapse.rs::surface_class` is a `CoarseField<ShareVec<6>>` window drawn through `sample_dithered`, `DitherSource` has its first production impl (`draws.rs::Coherent`), **U22 (the cake law) is discharged with a world-scale test**. ⚠ **U3 / the near site is NOT started and is no longer just "the other half"** — it needs MM-1 (a separately callable membership dither), MM-3 (the working sub-cell state, which has no declared type), and it is **co-requisite with the octaves source** — the U3 squares' dominant signal was settled 2026-07-24 as the single-octave member dither (corrections #45), so the near fix alone does not clear them |
| **P5** | genesis passes / **facies driver** | ratified concept, gated behind an `🔖 OPEN EDGE` |
| ~~**P6**~~ | ~~**octaves** as a `DitherSource` impl~~ **MOVED TO ENGINE 2026-07-29 → E5 member #0** — this row and § 3's "P6 is engine" said opposite things four rows apart (caught by the member-#0 design pass). `DitherSource` runs per voxel (granularity table: per-voxel draw = engine primitive) and is a crate-layering seam, not a plugin seam: **a pack SELECTS a source by id, never supplies one**. **The socket is now occupied 2026-07-29 (journal/0125): `draws.rs::Coherent` is the first impl**, and it fixed the API friction the design pass flagged — one salt dimension maps to `(domain fixed at construction, salt = tag)`. Octaves are the second impl and slot in beside it, unblocked | the U3 / near-site half of P4 (co-requisite) |
| **P7** | metamorphism (grade from `exhum`/`t_crust`) | unblocked by the geotherm; unstarted |
| **P8** | material **FORM** from provenance | `stubs.md` § 12 — the sub-voxel sieve deletes ~75 % of the sediment pile |
| **P9** | bio / eco / socia / civ | **NOTHING EXISTS. ON HOLD, not never.** Gate is a **USER call**; earth-science progress does not open it |

---

## 3. The edges that actually decide the order

**E3 (RATE) → P1. ✅ DISCHARGED 2026-07-29 (journal/0123).** `ARCHITECTURE.md` argued the
creep blocker was *what RATE's absence produces*; the pass, given no engine clock, **grew its
own** `dt`, confirming it from both sides. RATE landed against that known-good fixed point and
the hash comparison came back **identical** — `GOLDEN_SURFACE 0x15A6_B756_7A84_29FB` /
`GOLDEN_RECORD 0x820B_A198_49DD_234A`, `tests/rate_axis.rs`. The pass now takes its phase
length from the engine; **only the derived sub-step count stays inside it**, which is E4's half.

**E3 → E4, the edge the slice sharpened.** RATE deliberately does **not** own stability
substepping (user ruling, 2026-07-29). So the two divisions now sit four lines apart in
`Erosion::diffuse` — *authored* `dt` from outside, *derived* `n_sub` inside — which is exactly
the shape E4 has to hoist into the kernel. **E4's target is now a concrete two-line pattern in
one function rather than a design sketch**, and `sat.rs` is its second instance.

**P1 + `sat.rs` → E4.** Extraction needs two instances and a ruling on who owns the stability
bound. **Both landed 2026-07-29.** E4 is ready to design.

**P3 → E5.** *"The primitive the channel needs is the one FLOW spent three slices building"* —
face fluxes as Dirichlet conditions, the load budget as mass. **E5's inputs are built and
sitting idle.** This is the strongest ready-now signal on the board.

**E5 → `collapse.rs` decomp → the file-size chore.** 2,415 lines, *essentially zero
declaration*. One slice discharges two obligations.

**P4 is ~~INDEPENDENT of E5~~ NOW ROUTED THROUGH E5 — superseded 2026-07-29 by the
refinement ratification.** The user's ruling at ratification: first members *"should be
revisited in light of this design, not taken as wherever they landed prior."* CoarseField
IS member #0, so its adoption plan is superseded *as a plan*: the member-#0 design pass
(dispatched 2026-07-29) re-derives it against the bones — likely reaching the same two
user-report fixes, but that is its finding to make, not this file's to assume. The old
independence argument (seam-first, touch `collapse.rs` twice) is preserved as input to
that pass.

**P4 → P6. ✅ HALF DISCHARGED 2026-07-29 (journal/0125).** Octaves are a legal `DitherSource`
impl. The socket already exists; the octaves supply the **source**, `sample_dithered` supplies
the **draw**, `summarize` retires the residual bias. **They are not rivals — they compose**,
and nothing in the corpus said so, which is why two ratified designs read as contradictory for
a week. The far slice occupied the socket with a *coherent* impl and proved the composition
concretely; **two of the three parts moved and the third did not.** `summarize` is now ruled
OUT of this tier (→ octree node contract), so **the residual majority-amplification bias is
not retiring here** — it rides in the shipped far field, at both salts, exactly as it did
before. The edge that remains is **P6 → the U3 half of P4**, and it now runs the other way
from what this heading implies: the near-path fix alone will not clear the checkerboard, so
the octaves source is a **co-requisite of U3**, not a follow-on of P4.

**P5 is content, P6 is engine.** The user's remembered design splits cleanly across the
boundary already ratified: **facies driver → a declared field pass (content)**; **octaves +
dither + summarize → refinement primitives (engine)**.

**P1 → P2 → the flag flip.** Nothing downstream of erosion can be judged until the constant is
re-picked, and it must be picked **against the published literature**, never against a look.

---

## 4. Ready to start today, in order

1. ~~**E3 — RATE.**~~ **✅ SHIPPED 2026-07-29 (journal/0123)** — hash-identical, and E6/E7 are
   unblocked. *The rest of this list is unchanged in order; E3 leaving the top does not promote
   anything past what already justified it.*
2. **E5 — the refinement design pass.** A whole tier with zero members that the north star
   requires for plugin-agnosticism, with its inputs already built and idle.
3. ~~**P4 — CoarseField adoption.**~~ **HALF SHIPPED 2026-07-29 (journal/0125)** — the far
   site landed and U22 is discharged. What is left of P4 is **U3 / the near site, and it is
   no longer "blocked by nothing"**: it needs MM-1 + MM-3, and the **octaves `DitherSource`
   is a co-requisite** — the U3 checkerboard's dominant signal was settled 2026-07-24 as the
   **single-octave member dither**, so the near-path fix alone leaves the squares on screen
   (ROADMAP § Observed; corrections #45). Octaves are now the cheaper of the two and have a
   worked socket to land in.
4. **P2 — the calibration re-pick.** Needs a literature pass, not an engineering one.
5. *(cheap, rides alongside anything)* the **cold-tier appearance probe** — render a cold tile
   and the same ground loaded, diff them. Never once run.
6. *(owed by E3, small, and NOT a blocker)* **the remaining `dt` conversions** — stream
   transport, bedrock weathering, the wind and wave agents still assume `dt = 1.0` in their
   magnitudes. Each needs a **modelling** call, not a mechanical one (weathering is an
   exponential approach: `1 − exp(−k·dt)`, not `k·dt`), and picking one silently re-tunes a
   constant **P2 is about to re-pick anyway**. *Do these WITH P2, against the literature — not
   before it.*

**Blocked or deliberately parked:** P9 (user gate) · E8 (held) · P5 (OPEN EDGE) · `ores.md`'s
conceptual revisit (owed, genuinely unscheduled).

---

## 5. Process — how this file stays true

Added 2026-07-29 at the user's direction: *"fold in graph-building to the main session's
process, checking it at start and end of session and updating it throughout."*

- **START of session** — read this with the ROADMAP close block. It is the fastest available
  answer to *"what can I start right now, and what is it downstream of?"*
- **THROUGHOUT** — when a slice lands, moves a state, or reveals an edge, update the table in
  **the same commit**, exactly as ROADMAP is updated. *An edge discovered and not written down
  is the failure this file was built to stop.*
- **END of session** — the `wrap` ritual checks it. A slice that shipped and left a row stale
  is an unclosed loop.

**The two failure modes to watch, both already observed here:**
1. **A stale classification outliving its reason** — the erosion/engine miscoding lived one
   night in a close block and reached a cold session intact. **State the placement, and where
   it was ruled.**
2. **An edge nobody wrote down** — `sample_dithered` was built, named for a user law, and
   uncalled for seven days while being absent from all three loose-end loci. **A built-and-idle
   thing is an EDGE, not a rest state**; it is either about to be consumed or about to be
   deleted, and § 1/§ 2 must say which.
