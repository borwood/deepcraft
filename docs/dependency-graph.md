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
| **E3** | **RATE — per-pass cadence + a real `dt`** | **RATIFIED 2026-07-24, NEVER BUILT.** `dt` pinned to `1.0`; nothing scales by it | E4's first extraction · every pass that wants a phase length |
| **E4** | **field-solver primitives** — the S-10 gather; **the kernel owns its own stability bound** | **SHAPE NAMED 2026-07-29 (S-10), 2 instances, NOT EXTRACTED** | every future diffusing pass |
| **E5** | **refinement primitives** — coarse→fine reconstruction | **DECIDED 2026-07-28 (tier) + 2026-07-29 (definition). ZERO MEMBERS. Undesigned.** First slice is a **design pass, not code** | the whole appearance cluster · `collapse.rs` decomp |
| **E6** | **open resource vocabulary** — `DeepAxis` retires; packs declare their own ids | sequenced, after E3 | third-party packs |
| **E7** | **authored order + the validator** | sequenced, after E3 | plugin-agnosticism |
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
| **P1** | hillslope creep / erosion operator | **FIXED 2026-07-29** (journal/0122). Sub-cycles via a **stand-in** `dt` — `stubs.md` § 30, heir **E4** |
| **P2** | `EROSION_CALIBRATION` re-pick + flag flip | **SEQUENCED.** 45 was fitted to the broken solve and inverts under the fixed one |
| **P3** | flow / hydrology — face-flux record, head field | **BUILT AND IDLE.** Zero production consumers, *on purpose* |
| **P4** | CoarseField **adoption** (U22 + U3) | **REVIVED 2026-07-29.** Extraction shipped 2026-07-22; adoption never happened |
| **P5** | genesis passes / **facies driver** | ratified concept, gated behind an `🔖 OPEN EDGE` |
| **P6** | **octaves** as a `DitherSource` impl | not built |
| **P7** | metamorphism (grade from `exhum`/`t_crust`) | unblocked by the geotherm; unstarted |
| **P8** | material **FORM** from provenance | `stubs.md` § 12 — the sub-voxel sieve deletes ~75 % of the sediment pile |
| **P9** | bio / eco / socia / civ | **NOTHING EXISTS. ON HOLD, not never.** Gate is a **USER call**; earth-science progress does not open it |

---

## 3. The edges that actually decide the order

**E3 (RATE) → P1.** `ARCHITECTURE.md` argued the creep blocker was *what RATE's absence
produces*; on 2026-07-29 the pass, given no engine clock, **grew its own** `dt`. The argument
is now confirmed from both sides. **E3's acceptance test is already discharged by the
stand-in** — which makes the slice *sharper*: the axis lands against a known-good fixed point,
so "did RATE reproduce it" is a hash comparison, not a judgement call.

**P1 + `sat.rs` → E4.** Extraction needs two instances and a ruling on who owns the stability
bound. **Both landed 2026-07-29.** E4 is ready to design.

**P3 → E5.** *"The primitive the channel needs is the one FLOW spent three slices building"* —
face fluxes as Dirichlet conditions, the load budget as mass. **E5's inputs are built and
sitting idle.** This is the strongest ready-now signal on the board.

**E5 → `collapse.rs` decomp → the file-size chore.** 2,415 lines, *essentially zero
declaration*. One slice discharges two obligations.

**P4 is INDEPENDENT of E5.** CoarseField adoption is ratified, built, and fixes two live user
reports **without** waiting for the refinement design pass. Seam-first, which is the north
star's own method. *Cost, chosen knowingly: `collapse.rs` gets touched twice — once now, once
in the port.*

**P4 → P6.** Octaves are a legal `DitherSource` impl. The socket already exists; the octaves
supply the **source**, `sample_dithered` supplies the **draw**, `summarize` retires the
residual bias. **They are not rivals — they compose**, and nothing in the corpus said so,
which is why two ratified designs read as contradictory for a week.

**P5 is content, P6 is engine.** The user's remembered design splits cleanly across the
boundary already ratified: **facies driver → a declared field pass (content)**; **octaves +
dither + summarize → refinement primitives (engine)**.

**P1 → P2 → the flag flip.** Nothing downstream of erosion can be judged until the constant is
re-picked, and it must be picked **against the published literature**, never against a look.

---

## 4. Ready to start today, in order

1. **E3 — RATE.** Smallest it has ever been (the stand-in did the hard part), and it is the
   only thing gating E6/E7.
2. **E5 — the refinement design pass.** A whole tier with zero members that the north star
   requires for plugin-agnosticism, with its inputs already built and idle.
3. **P4 — CoarseField adoption.** Ratified, built, blocked by nothing, fixes two live user
   reports.
4. **P2 — the calibration re-pick.** Needs a literature pass, not an engineering one.
5. *(cheap, rides alongside anything)* the **cold-tier appearance probe** — render a cold tile
   and the same ground loaded, diff them. Never once run.

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
