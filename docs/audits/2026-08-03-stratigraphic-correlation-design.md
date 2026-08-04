# Stratigraphic correlation — the record-read redesign: design pass

**Arc anchor:** ROADMAP § Observed, *"FIELD REPORT, slice 3's interfingering"*
(`ROADMAP.md:2838-2885`) — the live-view REJECT trending on slice 3's membership dither,
its recorded station (wall pose feet **(82490, 215.5, 13346) m, yaw 0, pitch +0.12**;
assets `0149-interfinger-contact-wall.png` + `-benchcut`), and the design conversation the
user opened from it. Supersedes-as-plan the mechanism of
`docs/audits/2026-08-02-p11-slice3-design.md` (the membership dither) — *existence is not
standing, applied to our own day-old mechanism.*

**THE ANCHOR — user sketch and principle, verbatim (2026-08-03). This pass EXECUTES the
sketch; the sketch is never an input to reconcile away.**

> **Sketch:** *"a deep cell is a construct. in reality there are no deep cells. how do
> regions continue over distances in reality? barring any discontinuity feature, assuming
> under most conditions different broad regions are continuous and not separate geological
> buckets, wouldn't we just join layer to layer across boundaries? layers created at the
> same time in two different deep cells should... blend? find a midpoint and smoothly grade
> their thickness, as well as whatever the physical drivers etc does to facies."*
>
> **Principle:** *"ideally our default case, barring any physical drivers, is utterly
> smooth interpolation between all deepcell boreholes. non-smooth detail is refinement
> content, and for the default deepsim plugin pack, it must model a process honestly."*
>
> **Framing ratified in conversation:** deep cells are BOREHOLES ~460 m apart; the read
> between them is the geologist's cross-section — Walther's-law lateral facies change,
> pinch-outs, truncations.

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option is priced
and its trade-offs named; picks marked **user-owned** are the user's. Anything the agent
originated is marked **(assistant-proposed)**. **No cargo was invoked** — every number is
arithmetic over already-measured quantities cited to source, or a hand computation flagged
as such.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (verified in code here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / stale input /
needs the user's eye).

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Written against the
slice-3 worktree state (post-`cd1058a` merge content). Anything that later refutes or
re-scopes this file gets a banner **here**, stamped by the author of the correction.

> *Clerical (2026-08-03, doc-topology F6): the body's three mentions of "the 0147
> wall" (§§ 4, 7 S3, 9 P-4) mean **journal/0149**'s wall station — the 0147→0149
> renumber landed the same day this was written; the § 0 asset citations
> (`0149-interfinger-*`) are correct. Body left as written.*

> **✅ P-1 RULED 2026-08-04 (user) — M-C, WITH THE PREDICATE DERIVED, NOT AUTHORED.**
> Resolved through the materials-roster philosophy ratified the same conversation
> (`materials.md` § DECIDED 2026-08-04: materials are labels over regions of a
> continuous term space): continuum iff two regions adjoin on a declared axis —
> computed from the FS-A release spectra + property sheet, never a hand pair-list —
> → mixture grading (M-A's channel); discrete pairs → the octaves cut (M-B's
> channel); sharpness only from declared breaks (§ 1.3's seam). § 3.3's tension
> resolves rather than picks a side: smoothness governs continua, dress-every-contact
> governs the discrete cut's texture. Note for the build: I-4 (the new draw domain)
> is conditional on the discrete branch, which vanilla exercises at every
> clastic-igneous and clastic-organic contact. P-2…P-5 remain open.

> **✅ P-2 RULED 2026-08-04 (user: "yes") — the inverted acceptance criterion + S3
> walk plan are RATIFIED as drafted (§ 7), with the two-register reading recorded as
> part of the ratification per P-1's M-C:** continuum pairs read LITERALLY per column
> (mixture fractions ramp smoothly — assertable arithmetic); discrete pairs read
> STATISTICALLY (realized fractions ramp monotonically, contacts carry the variogram's
> texture, no straight line, no full-column transplant). A sharp contact with no owning
> process is a defect, not a texture; today's complete legitimate-sharpness list:
> nothing inside the grid, the grid border in the wilds. "Names its owning process"
> stays a WALK JUDGMENT for now — the machine-checkable form (every sharpness traceable
> to a declared break or cut) is a probe candidate for when the § 1.3 discontinuity
> seam has its first real occupant. The criterion doubles as the § 2.3 instrument's
> spec, so build slices assert against it from S1. P-3…P-5 remain open.

> **✅ P-4 RULED 2026-08-04 (user: "yes, that's right. an empty stack is a parent
> whose every chap thickness is 0.") — B-1, the zero-partner blend (§ 4).** The
> record feathers out against basement as an onlap wedge; B-3's hard edge stays for
> the true grid border only. Station 1 of the ratified walk (the 0149 wall) judges
> the cliff-face caveat. Orthogonal to the deposition-clock question — thickness
> arithmetic either way.
>
> **⏸ P-3 PAUSED 2026-08-04 by the user's architecture question** (*"they were laid
> down on the same clock. did we throw the time away?"* — yes: only the 8-chapter
> clock survived packing, § 10.7's flag). The **deposition-clock design pass**
> (`2026-08-04-deposition-clock-design.md`) prices recording/deriving per-bed epoch
> time; if it lands, correlation matches true isochrons, R-C shrinks to the
> within-unit interpolant, and R-C vs R-C′ largely dissolves. P-5 remains open.

**Read with:** `docs/audits/2026-08-02-p11-slice3-design.md` (F2 mass coupling, § 1, § 4 —
the superseded mechanism) · journal/0145 (what shipped) · journal/0128 (the octaves /
truncated-Gaussian construction) · journal/0129 (the stencil arithmetic, "grep the field")
· `docs/design/earth-processes.md` method item 5 + the deposition-agents engine ("facies =
class selection under agent+energy context") · `docs/design/refinement.md` (Laws 1–3, the
tiers-layer granularity table at `ROADMAP.md:1520-1528`) ·
`docs/design/material-genesis-notebook.md` § 2 (term-keyed, field-sourced edges) ·
`journal/corrections.md` #39/#48/#88/#89.

---

## 0. The findings that shape the design

**F1 — the correlation clock ALREADY EXISTS in the record, and it is global.** Every
`DepUnit` carries its tectonic **chapter** — **8 bits** of the packed u32, `chapter()` at
`recorder.rs:459` (bits 22–29; ⚠ the dispatching brief said 3 bits — the packed layout
reserves the full u8, `CHAPTER_SHIFT = 22`, mask `0xFF`, verified at `recorder.rs:504,525`)
— plus **stack order** (`DeepStrata.units` is the bottom-up deposition log,
`recorder.rs:775-780`) and the **unconformity flag** (bit 9, *"a property of the contact
below"*, excluded from the merge key on purpose, `recorder.rs:505-507`). Chapters are the
one clock **shared by every cell**: the tectonic history advances them globally
(`field.rs:305-309`), deposition only appends, and merging requires chapter equality
(`recorder.rs:824-829`), so chapter is **non-decreasing up-stack in every cell**. Two
neighbouring boreholes therefore already agree on a coarse time frame — the correlation
rule does not have to invent one, only to subdivide within it. The shipped world runs
**8 chapters** over **~7.4 M units** in **297,025 cells** (545², ~460 m pitch, 250.7 km
world) — mean **~25 units/cell**, mean unit 0.053 m, mean recorded column ~1.36 m, max
single unit 65.38 m (journal/0141 + slice-3 M0). ⚠ Max *stack depth per cell* and the
per-cell H distribution are unmeasured — both are owed by this pass's M0′ (§ 6).

**F2 — LINEARITY DISSOLVES THE CONSTRAINT THAT FORCED ONE-CELL-PER-COLUMN.** The slice-3
audit's F2 said: `H` is exactly the record's thickness sum, the veneer is the difference,
the ledger's bedrock slot is `units.len()` — so a per-column choice must take record + `H`
+ ledger from ONE cell or Law 3 leaks. That is true **of a choice**. It is not true **of a
linear blend**, because interpolation commutes with summation:

```
H(p) := Σ_j w_j(p) · H_j          (bilinear weights, Σ w_j = 1)
record(p) := per-bed  h(p, bed) = Σ_j w_j(p) · h_j(bed)
⇒ Σ_beds h(p, bed) = Σ_j w_j(p) · Σ_beds h_j(bed) = Σ_j w_j(p) · H_j = H(p)
```

**Blend-of-sums equals sum-of-blends, exactly, in f64.** If the record is interpolated
bed-by-bed on a shared bed partition (§ 1) and `H` is interpolated with the *same weights*,
the finalize invariant `Σ units == H` holds at **every column** by arithmetic identity —
no leak, no invention, and the veneer subtraction stays mass-true. The F2 *bundle*
(`DeepField::cell_bundle`, `field.rs:915`) survives as the per-parent supply; what retires
is the theorem's premise that the column must pick. Holes in the argument are real and
enumerated in § 2.2 — none is fatal, one (the ledger fold) forces a design choice.

**F3 — CORRELATION IS ENTIRELY READ-SIDE.** The blend happens where the dither happened:
in `column()`, at expression. The deep sim, the recorder, the fixed-point quantum, the
per-cell carry (`|carry| ≤ q/2`, `recorder.rs:787-798`), the merge key, chapters — nothing
gen-tier moves. Consequences: (a) **every deep-time golden family must be STILL** under
this work (`GOLDEN_RECORD`+variants, `GOLDEN_GEOTHERM`, `GOLDEN_FLUX`, `GOLDEN_HEAD`,
chapters) — a family that moves is a defect, not a re-capture; only CONTENTS + SURFACE
(and possibly far, § 8) re-capture; (b) the U5 quantization holes the brief asks about
(§ 2.2) largely evaporate — the stored record stays quantized and carried exactly as
shipped; the blend is derived f64 over quantized inputs and stores nothing.

**F4 — THE COST DRIVER IS NOT `run_strata`; IT IS THE PER-COLUMN FILL.** M0 measured
`run_strata` at ~7 µs/call, 0.2 % of `column()` (≈14 ms/chunk by that ratio — hand
arithmetic, flagged). The restructure already runs it once per touched cell (≤ 9); this
pass keeps that. The blend arithmetic itself is provably cheap (§ 6: ~a dot product per
bed per column). The unknown is `ColumnFill`: today ONE fill per SubCell serves all its
columns (`subcell.rs:44-46`); a per-column thickness vector needs per-column fill state —
1024 where there were ≤ 9. Whether that is noise or a fight is exactly the `run_strata`
question of the last audit, and it gets the same treatment: **measure before the slice**
(§ 6).

**F5 — IDENTITY CANNOT BE INTERPOLATED, SO "UTTERLY SMOOTH" MEETS A DISCRETE AXIS.**
Thickness is a measure; blending it is closed. `species` is a `MaterialId`; there is no
midpoint of siltstone and mudstone **except** through the one channel the engine already
has for fractional composition — per-voxel MIXTURES (`VoxelContents`, `mixed_contents`).
So the smooth default is *expressible*, but it makes a specific physical claim (§ 3): a
voxel that is literally 50 % siltstone / 50 % mudstone, everywhere in the transition,
deterministically. The alternative the corpus already ratified machinery for — a
truncated-Gaussian / octaves cut at the blended shares (journal/0128, the tiers-layer
table) — is *statistically* graded and *locally* sharp. **The tension between the
principle's letter and geostatistics' notion of honesty is surfaced in § 3.3 and NOT
resolved here.**

**F6 — reported, not applied (source files; outside this pass's write set): two stale doc
comments in `recorder.rs` contradict the shipped M-1.** The `DepUnit` struct doc
(`recorder.rs:483-485`, *"The mover axis is recorded WITHOUT merge-key membership (M-2)
pending the measured mover split factor"*) and `key_bits`'s doc (`recorder.rs:709-710`,
*"never unconformity, never the mover"*) both predate the M-1 resolution; `MERGE_KEY_MASK`
itself includes the mover bits with the measured 1.0308× justification in its own comment
(`recorder.rs:509-525`). A-2's shape, five hours old. Whoever opens the file next owes the
two comments.

---

## 1. The correlation rule (design question 1)

### 1.1 What must be defined

Given the ≤ 4 (≤ 9 straddling) parent cells a chunk's stencil touches, define a **shared
bed partition**: a single ordered list of correlated bed intervals such that every parent's
stack maps onto it, and a column at position `p` realizes each interval at thickness
`Σ_j w_j(p)·h_j(interval)`. A parent with no material in an interval contributes 0 —
which *is* the pinch-out: a bed present in A and absent in B **tapers linearly to zero**
across the span, automatically, with no special case. Correspondence, not blending, is the
design content.

### 1.2 The candidates, priced

**R-A — index alignment (the strawman, priced to show why not).** Unit k of A ↔ unit k of
B. O(max(m,n)); falls apart the moment counts differ (a deposit-erode cycle in A shifts
every correlation above it by one — beds correlate to strangers *arbitrarily far* up-stack).
Worst case on the shipped world: adjacent cells commonly differ in count (mean 25, driven
by per-cell erosion history; the 226,534 mover splits alone show same-key runs differ
cell-to-cell). **Rejected on correctness, not cost.**

**R-B — bed-to-bed matching (sequence alignment).** Within each chapter, align A's and B's
unit lists by key similarity (species/tag distance), gaps = pinch-outs — Needleman–Wunsch
per neighbour pair. This is what a human geologist does with two measured sections.
Costs: (i) O(m·n) per chapter per pair — fine at the mean (25² ≈ 625) but the max stack is
**unmeasured** (M0′) and the worst case is a wide basin cell against a condensed one;
(ii) **the real problem is arity**: the stencil is 4-way (9 straddling), and multi-sequence
alignment is not pairwise-composable — align(A,B) and align(B,C) need not agree with
align(A,C), so a chunk's shared partition is order-dependent unless a proper multi-way
alignment is run (exponential in parties, or heuristic with seams **between chunks** —
reintroducing exactly the artifact class this pass exists to kill); (iii) a similarity
score over the packed axes is a **fitted taxonomy** — a tolerance chosen until sections
look right, the anti-pattern § Gates names. **Priced and not recommended
(assistant-proposed), with one honest virtue recorded: it is the only rule that correlates
by IDENTITY, which is how field correlation actually works when no ash bed dates the
section.**

**R-C — the shared-clock proportional rule (assistant-proposed; RECOMMENDED).** Use the
clock the record already carries (F1). Partition every stack by **chapter** — globally
shared, monotone up-stack, zero inference. Within a chapter, correlate by **cumulative
thickness fraction**: the bed boundary at fraction f of A's chapter-c thickness
corresponds to fraction f of B's chapter-c thickness. The shared partition for a chunk is,
per chapter, the **union of all parents' fractional breakpoints** — O(Σ units) to build,
once per chunk, no pairwise anything, arity-free (4-way and 9-way cost the same union),
and **chunk-seam-free** because the breakpoints depend only on the parents' stacks, which
neighbouring chunks sharing those parents also see.

- **Geology of it:** proportional correlation of preserved sections between isochrons is
  the standard construction when internal surfaces are undated — the chapters are our
  dated horizons, the fractions are our "assume steady preserved accumulation between
  them". It renders lateral facies change (same interval, different species → § 3),
  pinch-outs (fraction interval empty in one parent — no: an *interval* is never empty
  under pure proportion; pinch-outs arise from the **chapter** level, where a cell with
  zero chapter-c thickness contributes 0 to every interval of c, tapering the whole
  chapter package to zero — the truncation wedge), and thickness grading everywhere.
- **Its honest weakness:** within a chapter it correlates by *position, not identity* — a
  storm bed at 40 % of A's chapter 5 correlates with whatever sits at 40 % of B's, even
  when B carries the same distinctive bed at 60 %. The result is never a leak (mass is
  § 2's argument regardless) but can be a *doubled* facies transition where matching
  would have found one. At 460 m spacing and ~25-unit stacks this is the standard
  geologist's error too; it is bought consciously.
- **Deposit-erode cycles within a chapter** need one care: the record keeps only
  survivors, so the fraction clock is a *preserved-section* clock, not true time. Two
  neighbours with different erosion histories inside one chapter correlate their
  survivors proportionally — exactly what a field section forces on a geologist. Named,
  not hidden.

**R-C′ — R-C with unconformity anchors (the refinement, priced).** The unconformity flag
marks contacts where the record below was stripped (`recorder.rs:781-783`). Treat a
flagged contact as an **additional correlation anchor within its chapter** when both
parents carry one at the same chapter transition: split the chapter's fraction clock at
the anchor, correlating pre-strip survivors with pre-strip survivors. Where only one
parent carries the flag, no anchor — the surface dies out laterally, which is what
unconformities do (they pass into conformity basinward). Cost: a per-chapter scan, no
asymptotic change. **What an unconformity does to correlation under R-C/R-C′: it never
BREAKS it.** An erosional gap is a *time* discontinuity, and laterally it expresses as
truncation — beds tapering against the surface — which the chapter-package taper already
produces. A vertical wall is not a legitimate expression of an unconformity anywhere.

### 1.3 Where correlation legitimately FAILS

Per the principle, a failure must be an honest process, not an artifact. Inventory of
honest discontinuities available today: **none inside the record grid.** No faults are
modelled (tectonic deformation-to-strata is a Sequenced collapse-tier slice, `field.rs:602-608`
— the chapter table exists and nothing reads it); chapters are global; every cell pair is
correlatable under R-C by construction. So the rule is: **correlation never fails today,
and the mechanism carries a SEAM for the day it should** (S-5 shape): a
`discontinuity(cellA, cellB) -> Option<Discontinuity>` provider with **identity default
`None`**, its heir named as the fault/deformation pass — when a fault pass exists, *it*
declares the break, correlation renders the offset, and the sharp contact is bought by a
process. The record-extent boundary (§ 4) is deliberately NOT a failure — it blends.

### 1.4 Where the blend runs (two altitudes, one recommended)

- **D-1 — blend `DeepStrata` (deep units), then `run_strata` per column:** cleanest
  semantics, ruinous cost — 1024 × 7 µs ≈ 7 ms/chunk added (≈ 50 % of `column()`; hand
  arithmetic from M0's ratio), against runtime-is-sacred. Not recommended.
- **D-2 — `run_strata` per parent (as shipped, ≤ 9×), correlate at the `StrataRec` event
  level (assistant-proposed; RECOMMENDED):** the partition is built from the parents'
  *deep-history* events; per column, per-interval thickness is the weighted dot product.
  One widening required: `StrataEvent` (`geology.rs:84-97`) does not carry the chapter —
  the deep-history events are produced in record order, so either the event gains a
  chapter/interval tag or a parallel per-event partition index rides beside the
  `StrataRec` (the events already carry `sel_salt`/`sel_tag` addresses; the partition
  index is the same shape of sidecar). The **veneer** events (climate-driven, per-chunk
  ctx, identical inputs across parents) correlate trivially — same passes, same order —
  and the year-zero veneer keeps its existing dither legitimately (its formation context
  is the present, `geology.rs:101-105`).

---

## 2. Thickness grading and mass (design question 2)

### 2.1 The argument, formalized

F2 states it: with bilinear weights `w_j(p)` over the parent stencil and per-interval
blending `h(p, i) = Σ_j w_j(p)·h_j(i)`, the column total is `Σ_i h(p,i) = Σ_j w_j(p)·H_j`
— **exactly**, in f64, at every column, because interpolation is linear and summation is
linear. Take `H(p) := Σ_j w_j(p)·H_j` as the column's regolith-coupled total (replacing
`regolith_at_voxel`'s NEAREST — the S-4 live-violation entry's *"the fix must route around
[non-interpolability], not through it"* is discharged by construction: the record became
interpolable **bed-wise**, so `H` may finally go bilinear *in lockstep*). Then:

- **F2 coherence:** record sum = `H` per column — identity, not tolerance.
- **Global mass:** summed over any region, `Σ_p H(p) = Σ_j (Σ_p w_j(p))·H_j`; over a full
  cell footprint the bilinear weights integrate to the cell area (the 9/16 integral of
  journal/0125 is the same calculus), so **global recorded mass is conserved exactly in
  the continuum and to lattice-sampling error on the voxel grid** — and the *lattice* error
  is a fixed geometric constant per cell, not a per-op accumulation. At midpoints the
  pairwise statement in the sketch (`(H_A + H_B)/2`) is the 1-D case of the same identity.
- **The veneer:** surficial veneer = terrain − record-derived quantities; terrain (`surf`)
  is already bilinear, `H` becomes bilinear with the same continuity class → the
  subtraction is C0 and mass-true where it was cell-stepped.

### 2.2 The holes, found and priced

1. **World edges / stencil clamp.** `deep_coords` clamps to the grid
   (`field.rs:852-853,919-920`); clamped weights still sum to 1 (a duplicated edge parent
   absorbs its off-grid neighbour's weight), so no leak — but the "global conservation"
   claim holds with an **edge-band caveat**: the outermost half-cell band over-weights
   edge cells. Same behaviour as every clamped bilinear field in the tree; state it, don't
   fix it.
2. **The fixed-point quantum + carry.** Untouched (F3): blending is read-side f64 over
   exactly-representable quantized inputs (`q = 2⁻¹⁰` is dyadic; `quanta × q` is exact;
   weighted sums of exact f64s carry ordinary rounding, bounded by ~4 ULP per interval).
   The gen-tier carry invariant and the delivered-budget closure are not in this diff's
   blast radius at all. **The one interaction:** the per-interval fractions of R-C are
   computed over quantized thicknesses — fine, fractions of exact integers.
3. **Unit-count-dependent ledger indices — the real hole, and it forces D-2's corollary.**
   `LedgerView` slots are keyed per record slot with bedrock at `units.len()`; a blended
   column has no single unit list. Facts **cannot be blended pre-fold** (they are keyed to
   stacks that no longer exist at the column). **Resolution (assistant-proposed): fold per
   parent — each parent's weathering fold runs against its own record and ledger, inside
   its own `cell_bundle`, exactly as shipped — and blend the folded, per-depth OUTCOMES
   with the same weights.** Where the fold's effect is thickness-like (weathered metres),
   linearity holds and the blend is exact; where it is categorical (a retagged member),
   it joins § 3's identity question. ⚠ The fold's full output shape needs a verification
   read at build time (I-3).
4. **Pinch-out below expression resolution.** A tapering bed drops below one voxel, then
   below the eighths/mixture floor, then vanishes per column. Mass at the pre-voxel level
   is conserved by the § 2.1 identity; voxelization truncates exactly as today's
   expression already truncates sub-voxel spans (`ColumnFill` plans, partial eighths).
   **The gate asserts at the pre-voxel level** and leaves voxel rounding to the existing
   fill invariants.
5. **f32 events.** `StrataEvent.thickness_m` is f32; blending in f32 would reopen
   accumulation-order noise. Blend in f64, cast at the fill boundary as today.

### 2.3 What the gate asserts (invariants, never snapshots; each scale-free)

1. **Per column:** `Σ_i h(p,i) == Σ_j w_j(p)·H_j` within a derived f64 bound (ULP count ×
   interval count — derived, not fitted). *Scale-free: per-column arithmetic.*
2. **Continuity:** for adjacent columns, `|H(p) − H(p′)| ≤ Σ_j |∇w_j|·max_j H_j` with the
   gradient bound derived from the cell pitch (weights change by ≤ 1/cell-width per
   voxel). *Scale-free: a Lipschitz property of bilinear weights.*
3. **Pinch-out monotonicity:** a bed present only in parent A realizes thickness
   monotone in `w_A` along a transect. *Scale-free: per-bed predicate.*
4. **Stillness:** all deep-time golden families byte-identical (F3). *The strongest
   single tripwire this design admits.*
5. **The partition is seam-free:** two adjacent chunks sharing parents produce identical
   interval structure for their shared boundary columns. *Structural; the R-B rejection
   made concrete.*

---

## 3. Identity within a correlated bed (design question 3)

Same interval, A says siltstone, B says mudstone — Walther's law seen from a borehole
pair. Thickness has a midpoint; a `MaterialId` does not. Three expressible shapes, and a
tension to surface, not resolve.

### 3.1 The ratified machinery this lands on

The corpus already holds a ratified architecture for principled sub-cell structure, and
the blended shares slot into it **exactly** (the tiers-layer table, user 2026-07-29,
`ROADMAP.md:1520-1528`):

| granularity | tiers-layer says | under correlation |
|---|---|---|
| per-cell (~460 m) | the facies **driver** — a smooth field | the per-parent bed identities + weights → a **blended share vector per interval**, smooth by § 2 |
| spanning | the **octaves** bridging cell → chunk → voxel | journal/0128's `Octaves` — BUILT, unbiased, variogram-anchored (D ≈ 1.26 from published facies-boundary dimensions) |
| per-voxel | the **draw** — *"must vary here or it reads as a grid"* | `ShareVec`-style inverse-CDF cut at the blended shares |

And journal/0128 named the construction from the literature: thresholding a Gaussian field
at the quantiles of target proportions **is truncated-Gaussian facies simulation** — the
standard geostatistical answer to precisely this question ("what lies between two
boreholes whose facies differ").

### 3.2 The options

- **M-A — mixture grading (the principle's letter).** Express the interval as a per-voxel
  MIXTURE at the blended shares: `w_A`-fraction siltstone, `w_B`-fraction mudstone, via
  `VoxelContents` (exists today; `mixed_contents`/partial allocation are the shipped
  channel). Utterly smooth, deterministic, zero new stochastic machinery, and the
  acceptance instrument becomes a literal per-column continuity assert. **The physical
  claim it makes:** every voxel in the transition is *literally* interbedded/intergraded
  rock at exactly the interpolated fraction. For **clastic continua** (siltstone↔mudstone
  is a real continuum in silt/clay ratio; sandstone↔conglomerate likewise) this is honest
  — arguably *more* honest than bodies. For **discrete pairs** (limestone against
  sandstone; coal against anything) a 50/50 mixture voxel is a rock that does not exist —
  the real transition is interfingering bodies, and the mixture claims a homogenized
  fiction. Cost: near-zero runtime; the mixture cap (combinatorial mixture doctrine)
  bounds palette growth — ⚠ unmeasured how many distinct blended mixtures a transition
  zone mints (M0′).
- **M-B — the stochastic cut (the corpus's shipped shape).** Per voxel, draw the
  interval's identity from the blended shares through `Octaves` (truncated-Gaussian cut):
  bodies of siltstone interfingering with mudstone, realized fraction ramping with the
  weights, boundaries at the variogram's fractal dimension. Statistically graded,
  locally sharp. This is the tiers-layer table executed at bed scale, and it is what a
  cross-section of a real transition looks like. Cost: one registered draw domain (the
  `NearRecordMembership` lesson: registered, never hand-rolled), per-voxel draw on a path
  that already pays per-voxel draws.
- **M-C — split by material relation (assistant-proposed hybrid).** The registry knows
  more than "different id": material-genesis-notebook § 2's term-keyed edges give
  materials **comparable scalar terms**. Where the two identities are neighbours on a
  continuum (share a class/termed axis — silt/clay fraction, φ grade), grade as mixture
  (M-A); where they are discrete, cut (M-B). Honest per family; the cost is a
  relation predicate on the registry (a pack-authored fact, per A-7 — the engine must not
  hardcode which rocks form continua).

### 3.3 ⚠ THE TENSION, SURFACED FOR THE USER — smoothness's letter vs. the octaves cut
**(do not resolve; two user-owned rulings are in play and they pull apart at this joint):**

- **The principle (2026-08-03):** *"utterly smooth interpolation… non-smooth detail is
  refinement content, and… must model a process honestly."* Read literally, an octaves
  cut is non-smooth detail: is a stochastic realization "modelling a process honestly," or
  is it decoration? **Geostatistics' answer:** the truncated-Gaussian field is not
  decoration — it is a *declared model of uncertainty* about unsampled inter-borehole
  structure, with its one shape parameter anchored to published facies-boundary fractal
  dimensions (journal/0128's derivation, not a fit). It models the *statistics* of the
  process where the process itself was not simulated. That is the discipline's standard
  of honesty for exactly this situation. **The principle's letter, against it:** a
  statistics-of-a-process is still not a process; the default pack could hold the smooth
  mixture and let interfingering arrive as refinement content when a member models
  shoreline migration or channel avulsion honestly.
- **And the older ruling cuts the other way (both are the user's):** earth-processes
  method item 5 (RATIFIED 2026-07-19) — *"no simulation-resolution edge may reach the eye
  as a square or analytic boundary… every grid-scale contact must be DRESSED BY
  NOISE/dither/blend/sampling tricks"* — and the tiers-layer amendment's own words:
  the per-voxel draw *"MUST vary here or it reads as a grid."* Note that under M-A the
  50 %-crossing of a mixture between two *discrete* materials is not an edge at all (the
  mixture varies continuously), so item 5 may be satisfied vacuously — or the mixture
  gradient itself may read as an airbrushed band, which is a walk question no desk can
  answer.
- **What the record actually holds** bears on it too: within one parent cell, the stack
  is *already* beds of single species (the record is discrete by construction); M-A
  introduces mixtures that exist in NO parent borehole. M-B realizes only materials some
  parent recorded. Both readings of "honest" have a foot here.

**This is P-1, the pass's largest user-owned pick (§ 9), and the acceptance walk is where
it becomes decidable — the two candidate frames differ visibly at a bench cut.**

---

## 4. The record-extent boundary (design question 4)

The station fact (`ROADMAP.md:2858-2871`): at (82490, 215.5, 13346), recorded banded
strata meet `has_contents:false` basement **floor-to-rim, razor-vertical** — the dither
transplanted record EXTENT, not just contents. Where a neighbouring parent has no record
(empty `DeepStrata`, or a record whose base sits above/below), there is no blend partner
for the beds.

- **B-1 — the zero-partner blend (assistant-proposed; RECOMMENDED — and it is not a
  special case).** Under R-C an empty stack is a parent whose every chapter thickness is
  0. The blend then tapers every bed package linearly to zero approaching the recordless
  cell: the record **feathers out against basement** — an onlap/pinch-out edge, which is
  what a real depositional margin looks like. Mass: exact (blending with zero). Cost:
  zero additional machinery — R-C already does this. The razor cliff becomes a ≤ 460 m
  wedge. *Geological honesty check: real basement-onto-cover contacts are onlap surfaces
  or faults; we model no faults yet, so onlap is the only honest default.*
- **B-2 — renormalize onto recorded parents** (drop recordless cells from the stencil,
  re-weight the rest): keeps full record thickness right up to the last recorded cell,
  then a discontinuity where the last recorded parent leaves the stencil — **rebuilds the
  cliff one stencil-width over**. Priced to show why not.
- **B-3 — the wilds-border precedent as-is** (fallback SubCell with empty record,
  `subcell.rs:36-39`, hard edge): correct OUTSIDE the record grid (the border wilds are
  beyond the playable interior and the grid genuinely ends), wrong inside it. **Keep for
  the grid border, retire for interior recordless cells** — B-1 subsumes the interior
  case automatically since outside-grid columns have no stencil at all.

⚠ One honest caveat on B-1: where the *terrain* is also discontinuous for unrelated
reasons the feathered record can outcrop oddly on cliff faces — a walk question. And the
elevation half matters: the record occupies the top `H` metres; blending `H` moves the
**record base** smoothly too, so the basement surface itself grades — that is the visible
half of the fix at the 0147 wall.

---

## 5. What retires (design question 5 — existence is not standing, applied at age one day)

Priced honestly; the substrate mostly survives because slice 3 built the right bones and
the wrong last step.

**Survives (the substrate):**
- **`SubCell` + `ColumnRec.records: Vec<SubCell>`** — the per-touched-cell products ARE
  the boreholes; correlation consumes exactly this set. The type's guarantees (fill from
  its own strata, soil from the same cell's `H`) remain load-bearing per parent.
- **`DeepField::cell_bundle`** — the F2 bundle survives as the per-parent supply (F2 of
  this audit); the fold-per-parent resolution (§ 2.2.3) depends on it.
- **The accessor layer** (`record_for`, `centre_record`, packed `DepUnit` accessors) —
  the migration seam. `record_for(x,z)` changes *return semantics* (a blended view rather
  than a parent reference) — ~14 files talk to it, and this is where they are repaid.
- **`Octaves`** — if M-B/M-C is picked it is the cut's source; if M-A, it idles at this
  joint but keeps its far/member consumers. **The packed `DepUnit`'s chapter bits become
  the correlation clock** — the co-rider funded the redesign without knowing it.
- The M0 instruments, the 200-chunk timing harness, `heap_bytes`.

**Dies (the membership draw):**
- **`ColumnRec.cell_of`** (the one-cell-per-column pick) → replaced by per-column weights
  (or nothing, if weights are computed in place).
- **The `sample_source_cell` call site in `collapse.rs:1575-1610`** — MM-1's near
  consumer. The MM-1 kernel itself stays in dc-core (the far tier's `sample_dithered`
  membership half still uses it); its § 3-adjacent status should be rechecked by the next
  spine sweep once the near consumer goes.
- **The `NearRecordMembership` domain (`draws.rs:151`)** — retired, *never refilled*
  (draws.rs's own retired-range doctrine; the salt-collision finding is four days old).
  If M-B is picked, the facies cut registers a **new** domain.
- **The membership-dither structural asserts** in `appearance_tour_p11`'s gate (in-chunk
  off-lattice transitions; 16-voxel reach) — they assert impossibilities *of the
  chunk-centre read* through the dither's realization; under correlation the whole
  instrument is rewritten to § 2.3 + the § 7 criterion. The `every_domain_is_listed`
  count (18) moves with the domain change.
- **The F6/P-6 framing** of the slice-3 audit (walk-ratifies-cell-wide-blend) — the walk
  happened, the verdict is the anchor of this file; that audit's header owes a banner
  pointing here **when this pass's plan is ratified** (stamped by that correction's
  author, per read-first item 5).

**Goldens:** CONTENTS (Medium triples) + the SURFACE family move — ratified semantics if
this design is ratified, re-captured with the why; **all deep-time families must be
still** (F3) and the stillness is asserted in the same merge. `GOLDEN_FAR_SURFACE`: still
— the far path is untouched (§ 8).

---

## 6. Cost (design question 6 — runtime is sacred)

**Arithmetic that is already decidable (hand computation, flagged):** per chunk, the
shared partition is O(Σ parent events) built once (≤ 9 parents × ~tens of events); per
column, per-interval thickness is a ≤ 4-term (≤ 9 straddling) dot product — at an
estimated union of ~10² intervals that is ~10⁵ multiply-adds per 1024-column chunk,
microseconds. **The blend itself cannot be the fight.** The two real unknowns:

1. **Per-column fill state (F4).** Today ≤ 9 `ColumnFill`s per chunk; blended thickness
   varies per column. Mitigations to price at build: (i) a fill that takes the shared
   interval list + a per-column thickness vector (prefix sums per column, O(intervals));
   (ii) lazy per-column evaluation inside the voxel walk (the walk already visits spans
   in order); (iii) quantize weights to a small set and cache fills per weight-bucket —
   ⚠ (iii) reintroduces steps and should be priced only as a fallback, loudly.
2. **Expressed-interval count.** Mean recorded H is ~1.36 m (~1.5 voxels) but the
   distribution is unmeasured and the walls that matter are the thick tails.

**M0′ — the pre-slice measurement run (one build-slot session, extends existing probes):**
(a) per-cell **H histogram** + max stack depth (units and events per cell); (b)
**`ColumnFill::build` share of `column()`** (the F4 decider — the exact analogue of M0's
`run_strata` share, same 200-chunk harness); (c) **union-interval count** distribution
over adjacent 2×2 stencils (decides the per-column dot-product constant); (d) if M-A is
in play, **distinct blended mixtures minted** per transition zone (the mixture-cap
check). **RETURN spec of the build slice:** before/after on `contents_contract`'s 240
`generate_chunk_with_materials` calls (re-baselined — the 102.43 s figure is nine days
old), plus `SubCell`/`ColumnRec` heap lines (the per-column vectors add
1024 × interval-count × 4 B-ish of transient; state it measured, `column_cache` is
runtime-resident).

---

## 7. Sequencing + gates (design question 7 — proposal, integrator sequencing, veto welcome)

**S0 — M0′** (measurements above; no semantics).

**S1 — the correlation kernel, consumerless-for-a-day at most:** the shared-partition
builder (R-C, chapter-partitioned, fraction-union; R-C′ anchors if picked) + per-column
evaluation, as pure functions over `&[SubCell]`-shaped inputs, with § 2.3's invariants as
unit tests (including the seam-free assert). No golden moves. *Gate: cheap evidence per
the batching rule (fmt + clippy + the changed crates' suites), batch debt recorded.*

**S2 — wire into `column()` + the identity pick (P-1) + goldens:** `record_for` semantics
change, the ≤ 14-file consumer repayment, `cell_of`/domain retirement, B-1 at record
extent, fold-per-parent blending. CONTENTS + SURFACE re-captured with whys;
**deep-time stillness asserted in the same merge**; the rewritten acceptance instrument
gates in `appearance_tour_p11` (report added wall-clock per probe doctrine). *Gate: FULL
workspace trio here, clearing S1's debt — this is the arc-chunk boundary.*

**S3 — the acceptance walk (Claude drives).** Tour-map first: the § 2.3 instrument
doubles as the frontier finder (strongest H-gradient cell pair + the 0147 station
revisit — its pose is recorded now). Stations: (1) the 0147 wall — the razor cliff must
read as an onlap wedge; (2) the strongest same-elevation facies transition — thickness
grading + the P-1 identity treatment; (3) an unconformity-bearing pair if the tour map
finds one (truncation wedge). `--fullbright` for the material read, lit pass for the
wedge geometry, both per instrument-must-see-the-question.

**THE ACCEPTANCE CRITERION INVERTS** (drafted for ratification, P-2):

> Slice 3 asked the walk to confirm a *transition exists*. This asks the walk to confirm
> **no sharp contact exists anywhere that no process explains**: the default read at any
> frontier is smooth gradation — thickness wedges, laterally shifting facies — and every
> remaining sharpness must name its process (today's complete list: nothing inside the
> grid; the grid border itself in the wilds). Under M-B the criterion reads
> statistically: realized fractions ramp monotonically and contacts carry the variogram's
> texture, with no straight line and no full-column transplant; under M-A it reads
> literally per column. **A sharp contact with no owning process is a defect, not a
> texture.**

**Same-commit doc obligations:** ROADMAP § Observed field-report entry resolved to this
plan; slice-3 audit header banner (author of the ratification stamps it); spines § 3
recheck for MM-1's near consumer; stubs entry if the discontinuity seam (§ 1.3) ships as
a named provider; `dependency-graph.md` P11/P4 rows; corrections entry if the walk
falsifies a claim here.

---

## 8. What this pass does NOT do

- **No far-path work.** The far tier's `ShareVec<6>`/`surface_class` blend rides as
  rejected-interim with the far register as heir; correlation at the near tier neither
  fixes nor worsens it. (Its heir conversation gains an argument: the near tier will now
  demonstrate record-derived gradation done honestly.)
- **No faults, no deformation.** The discontinuity seam ships empty (identity default
  `None`); the fault pass that fills it is the chapter-table slice, unsequenced.
- **No gen-tier change.** The recorder, merge key, quantum, carry, chapters: untouched
  (F3). Deep-time goldens still.
- **No grain modelling, no P10 resumption** — though note the grain bits, when FS-A
  writes them, land in the merge key and thus in the correlation partition for free.
- **No veneer formation-context change** (stub #31 rides with its re-pointed heir).

---

## 9. NEEDS RATIFICATION

### 9a. User-owned

| # | question | the shape of the trade |
|---|---|---|
| **P-1** | **Identity within a correlated bed: M-A mixture / M-B octaves cut / M-C split-by-relation** (§ 3) | the smoothness-principle's letter vs. geostatistical honesty vs. both-by-family; § 3.3 surfaces the tension between two user rulings (2026-08-03 smoothness, 2026-07-19 dress-every-contact) — only the user can weigh his own two principles |
| **P-2** | **The inverted acceptance criterion + walk plan** (§ 7) | drafted for a yes/no; note its reading differs under M-A vs M-B |
| **P-3** | **Correlation rule: R-C vs R-C′ (unconformity anchors)** (§ 1.2) | R-C is strictly simpler; R-C′ buys honest truncation-surface correlation for a per-chapter scan — integrator-settleable if the user prefers to delegate, but the rule shapes every cross-section in the game |
| **P-4** | **Record-extent: B-1 onlap feather** (§ 4) | recommended and near-free; user eye owed because the 0147 wall is the exhibit that opened this pass |
| **P-5** | **Ratify the supersession**: the slice-3 membership dither is retired-as-plan and § 5's retirement list executes | the walk verdict is trending REJECT but P-6 said the walk is the ratification — this pass must not presume it; a formal verdict closes it |

### 9b. Integrator-settleable (recorded when done)

| # | item |
|---|---|
| I-1 | M0′'s four measurements before the S2 wiring |
| I-2 | D-2's event-partition carrier (chapter tag on `StrataEvent` vs. a parallel index) |
| I-3 | the fold-per-parent blend's exact output shape (verification read of the weathering fold; § 2.2.3) |
| I-4 | the new draw domain if M-B (registered, new salt, never the retired one) |
| I-5 | fill-state mitigation pick (i)/(ii) per M0′(b); (iii) only loudly |
| I-6 | stamp targets in the same commits: slice-3 audit banner, ROADMAP entry, spines MM-1 recheck, the F6 doc-comment pair in `recorder.rs` |

---

## 10. What this audit could not verify — flagged, not smoothed

1. **No cargo was run.** All cost figures are M0/journal arithmetic or hand estimates
   flagged as such; the per-column fill cost — the one number that could sink D-2's
   runtime story — is M0′(b)'s job.
2. **Max stack depth, per-cell H distribution, union-interval counts: unmeasured** —
   every worst-case in § 1.2 and § 6 inherits this.
3. **The weathering fold's output shape** was not fully traced (§ 2.2.3 assumes
   per-depth outcomes blend linearly; a categorical component would join § 3). I-3.
4. **The chapter monotonicity claim** (F1) is argued from the merge/append/overprint code
   paths read here (`recorder.rs:745,824-829`), not exhaustively proven against every
   writer — a cheap debug assert in S1 (`chapter non-decreasing up-stack`) converts the
   argument into a fact.
5. **Whether R-C's position-not-identity weakness is visible at 460 m spacing** is a walk
   question; no desk analysis can price the doubled-transition artifact's frequency.
6. **The mixture-cap interaction under M-A** (distinct minted mixtures per transition) is
   unmeasured — M0′(d).
7. **⚠ CONTESTS discipline check — nothing in this pass contradicts the anchor sketch;
   two mappings are wider than its literal words and are flagged as executions, not
   reconciliations:** (a) the sketch says *"find a midpoint"* (pairwise); the mechanism
   generalizes to the bilinear 4-corner stencil the shipped code already stands on —
   the midpoint is the 1-D section of it (§ 2.1); (b) the sketch's *"layers created at
   the same time"* is implemented as chapter + preserved-fraction time (§ 1.2, R-C's
   named weakness), because true per-epoch bed ages are not in the 8-byte record — a
   finer clock would be a record widening (accessor-cheap per the P-2 ruling, priced
   only if the walk demands it). And § 3.3 surfaces a genuine tension **between two
   user-owned rulings** (2026-08-03 smoothness vs. 2026-07-19 dress-every-contact +
   the tiers-layer "must vary per voxel") — surfaced for the user, resolved by no one
   here.

---

*Author: read-only design-pass agent, 2026-08-03. Read-only except this file; no cargo
invoked. §§ 1.2 (R-C/R-C′), 1.4 (D-2), 2.2.3 (fold-per-parent), 3.2 (M-C), 4 (B-1) and
the F-findings' syntheses are assistant-originated analysis, marked where they go beyond
arithmetic over cited measurements. The anchor is the user's; this file executes it.*
