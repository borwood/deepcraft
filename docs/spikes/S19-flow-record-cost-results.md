# S19 — the flow-record cost probe: measured results

> **⚠ STALENESS BANNER (stamped 2026-07-29, corrections #75).** This spike's slot-pairing
> note was cited by `flow.md` § 2.2's flag as an open question with a uniform 2× cost; the
> pairing principle was ratified the same day it was flagged (`flow.md` § 11.5) and
> **re-ratified fresh 2026-07-29 as the three-mode confinement rule** (§ 11.5 banner). The
> 2× is the gross-vs-net choice (both directed halves kept), which shipped; it is not a
> cost the pairing rule adds. The measurements below are untouched and correct for their day.

**Measured 2026-07-25.** Probe: `crates/dc-worldgen/examples/flow_cost_probe.rs`
(`cargo run --release -p dc-worldgen --example flow_cost_probe`). Wall clock
**45.6 s** for the whole probe: 19.1 s for `Pregen::run` at seed 1337 /
`Extent::Medium` (which includes the production deep-time run), 26.1 s for a
second `DeepField` with `weather_inventory` ON (the `s18_weathering_tour`
config), and < 0.1 s for all the arithmetic. Gates green: `cargo fmt --all
--check` exit 0; `cargo clippy -p dc-worldgen --all-targets --release -- -D
warnings` exit 0 with `Checking dc-worldgen (…worktrees/agent-a7cf73622547e4915
/crates/dc-worldgen)` in the log — i.e. it checked *this* tree, not a sibling's
artifact.

This answers `docs/design/flow.md` **§ 9 Q1 ("Cost. Face flux × slots × chapters
is unmeasured")** with geometry rather than guesses, and it bears on **Q2**
(sub-face parallel channels) and **Q4** (episodic events) by pricing them.
**It does not make the call.** It states what each layout costs.

The probe is read-only on the world: it builds the production field and does
arithmetic. It reuses the shipped `DeepField::resident_bytes` helper rather than
inventing a second accounting mechanism (A-4) — the itemised baseline table below
is asserted equal to `resident_bytes()` at run time, so it is a *view* of that
helper, and the assert fails loudly if the two ever diverge.

---

## 1. Measured grid geometry

| quantity | measured |
|---|---|
| deep grid | **545 × 545 = 297,025 cells** |
| cell edge (`DEEP_CELL_M`) | **460.0 m** |
| covered extent | 250.7 km per side |
| epochs (`DeepConfig::iterations`) | **200** |
| chapters `K` (`DeepConfig::chapters`) | **8** |
| epochs per chapter | **25.0** |
| `size_of::<DepUnit>()` | 16 B |
| `size_of::<DeepStrata>()` | 32 B |

**Extent caveat.** `DEEP_MAX_WIDTH = 550` caps the grid *width*, so `Extent::Large`
gets a coarser cell (~1.8 km) rather than more cells. Cell count is therefore
approximately extent-invariant (Medium is already at 545 of the 550 cap) and these
projections carry across extents in **cell** count. They are **not** verified to
carry in **slots per cell** at Large — a coarser cell integrates more deposition
history per column and could plausibly hold more units. Only Medium was measured.

---

## 2. The record shape — units (stratum slots) per cell

This is the multiplier that decides everything, and the mean is a lie: the
distribution is **strongly bimodal by land/marine**.

| population | cells | min | mean | median | p95 | max | total slots |
|---|---:|---:|---:|---:|---:|---:|---:|
| **all cells** | 297,025 | 0 | **18.64** | 5 | 113 | 478 | **5,535,837** |
| **land** (`surf > 0 m`) | 44,264 (14.9 %) | 1 | **96.01** | 80 | 238 | 478 | 4,249,552 (76.8 %) |
| **marine** (`surf ≤ 0 m`) | 252,761 (85.1 %) | 0 | **5.09** | 5 | 9 | 310 | 1,286,285 (23.2 %) |

- **Cells with an empty record: 33,680 (11.3 %)** — all marine (land min is 1).
- Land is 14.9 % of cells but **76.8 % of the slots**. The world's flow record is
  overwhelmingly a land record by slot count, and overwhelmingly a *marine* record
  by cell count. Those two facts pull the layout in opposite directions: a
  per-cell index is dominated by the marine 85 %; a per-slot payload is dominated
  by the land 15 %.
- The land p95 (238) is 2.5× the land mean (96) and the max is 478 — so any
  fixed-size-per-cell allocation either wastes 5× on the median column or truncates
  the interesting ones.

### 2b. The chapter axis — and why `slots × K` is the wrong dense ceiling

Slot chapter stamps (`DepUnit::chapter`):

| chapter | slots | share |
|---:|---:|---:|
| 0 | 1,115,952 | 20.2 % |
| 1 | 713,223 | 12.9 % |
| 2 | 490,983 | 8.9 % |
| 3 | 504,981 | 9.1 % |
| 4 | 530,416 | 9.6 % |
| 5 | 591,428 | 10.7 % |
| 6 | 660,494 | 11.9 % |
| 7 | 928,360 | 16.8 % |

(The U-shape is real: chapter 0 survives because the deepest units are protected
by burial; chapter 7 survives because nothing has yet eroded it. The middle
chapters are the ones deep time ate.)

A slot deposited in chapter `c` **cannot carry a flow fact from before `c`** — it
did not exist. So the honest dense ceiling is
`Σ_units (K − unit.chapter)`, not `slots × K`:

- naive `slots × K` = **44,286,696** (slot, chapter) pairs
- **causal `Σ (K − chapter)` = 25,536,276** — **57.7 % of naive**
- causal, **land only** = 19,721,285 (**77.2 % of causal**)

**Every projection below uses the causal count.** Using `slots × K` would inflate
every number by 1.73×.

> Note this does *not* contradict flow.md § 1.2's "the facts are cross-cutting" —
> a young fact on an old slot is exactly what `K − chapter` counts. What it
> excludes is an *old* fact on a *young* slot, which is not cross-cutting, it is
> impossible.

---

## 3. Today's baseline — the denominator

Production `DeepField`, seed 1337, `Extent::Medium`, `weather_inventory` OFF (the
shipped default). **T = 170,469,205 B = 162.57 MiB.**

| part | bytes | MiB | % of T |
|---|---:|---:|---:|
| `surf` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `regolith` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `area` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `exhum` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `t_crust` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `geotherm` (f64/cell) | 2,376,200 | 2.27 | 1.39 % |
| `recv` (i32/cell) | 1,188,100 | 1.13 | 0.70 % |
| `lake` (bool/cell) | 297,025 | 0.28 | 0.17 % |
| `strata` structs | 9,504,800 | 9.06 | 5.58 % |
| **`strata` heap (`DepUnit`)** | **145,222,080** | **138.49** | **85.19 %** |
| `ledgers` (off) | 0 | 0.00 | 0.00 % |
| **TOTAL** | **170,469,205** | **162.57** | 100 % |

**The record already IS the DeepField.** Every scalar plane put together is 8.9 %
of residency; the strata record is 90.8 % (heap + structs). The flow record is
therefore not being added to a field of planes — it is being added *next to the
one thing that already dominates*.

### Two layout findings that transfer directly to the flow record

**(a) `Vec` doubling slack is 39 % of the strata heap.** The heap number above is
`capacity()`, which is the honest resident figure:

- 5,535,837 **live** units × 16 B = **84.47 MiB**
- 9,076,380 **capacity** slots × 16 B = **138.49 MiB**
- **slack = 54.02 MiB = 39 % of the strata heap, 33 % of all of T**

A per-cell growable `Vec` pays this. A flow record built the same way pays it too.

**(b) The `FactLedger` — today's only fact-shaped record — is 89 % empty headers.**
With `weather_inventory` ON the field grows **+156.91 MiB** (162.57 → 319.48 MiB,
a **1.96×**), and the ledger heap is 150.11 MiB. Decomposed:

- 5,832,862 inner `Vec<Fact>` (one per slot + one bedrock seam per cell), of which
  **5,760,856 are EMPTY — 98.8 %**
- **headers alone = 133.50 MiB = 89 % of the ledger heap**
- payload = 16.61 MiB over **1,033,189 facts**

> **RESOLVED 2026-07-25 (journal/0100).** This finding was acted on: `FactLedger`
> was converted to the same flat-arrays + CSR-index layout `flux.rs` uses, so the
> numbers in this subsection describe a shape that **no longer exists in the
> tree**. They are kept verbatim because they are the *measurement that motivated
> the conversion*, and because they remain the correct warning about
> `Vec<Vec<T>>` keyed per (cell, slot) for anything built next. See journal/0100
> for the before/after.

This is the most directly transferable measurement in the probe, because
`Vec<Vec<Fact>>` keyed by (cell, slot) is *precisely* the naive shape of a
per-(cell, slot) flow record. Built that way, the flow record's dominant cost
would be the 24-byte header of the 99 % of slots that carry no flow — before a
single flux value is stored. That is what the CSR/sparse layouts (L3/L4) below
exist to avoid, and it is why their index floor is quoted separately.

---

## 4. Projections — L1 through L4

### Stated assumptions (each is a named `const` in the probe)

| # | assumption | value | why |
|---|---|---|---|
| **A1** | flux magnitude width | `f32`, **4 B** | flux is a Dirichlet BC for refinement; f32's ~7 digits far exceed what a 460 m cell's discharge is known to. `f64` **doubles every number below.** |
| **A2** | lateral fan-out, **D4** | **2 owned faces/cell** | von Neumann; faces are shared, cell `i` owns E + S |
| **A3** | lateral fan-out, **D8** | **4 owned faces/cell** | Moore; owns E, S, SE, SW. The existing drainage solve is D8 (`Erosion::recv`), so D8 is the like-for-like successor |
| **A4** | slot alignment across a face | reported **both ways** | slot 3 in cell A is not slot 3's depth in cell B. **"shared"** = `owned` entries per (cell, slot), assuming slot *index* pairs across the face; **"per-cell"** = `2 ×` that, every cell storing all directions. Exactly a **2× spread** everywhere. |
| **A5** | vertical faces | **1 per slot** | slot↔slot within a column is `slots − 1`; modelled as 1 (over-counts by one per non-empty cell, < 0.5 %) |
| **A6** | boundary faces | 1 top/cell/chapter + 1 seaward/marine-cell/chapter = **4,398,288** | no basement face (no slot below unit 0) |
| **A7** | sparse entry | `(u32 key, f32 flux)` = **8 B** | key packs (direction, slot) inside a per-cell-per-chapter CSR row |
| **A8** | sparse index floor | `u32` CSR row offset per cell per chapter = **9,504,800 B = 9.06 MiB** | paid at *any* sparsity, including zero |
| **A9** | § 1.3 atom extras | **+8 B/fact** | `u16 fluid` + `u8 cause\|form` + `u8 pad` + `f32 load`. `form/phase` is 1 bit, `cause` < 16 inhabitants, `fluid` is a 16-bit `MaterialId` in this tree |
| **A10** | sparsity fractions | 1 % / 5 % / 20 % / 100 % | unknowable until the sibling's solve exists. 1 % ≈ trunk channels; 5 % ≈ channel network; 20 % ≈ every cell that ever saw water |

Dense face universe used by L3/L4 (D8 per-cell + vertical + boundary):
`25,536,276 × 9 + 4,398,288 = ` **234,224,772 faces**. Land-only universe:
**177,845,677 faces**.

### The table (T = 162.57 MiB)

| layout | bytes | MiB | × today |
|---|---:|---:|---:|
| **L1** dense lateral, D4 shared | 204,290,208 | 194.83 | **1.20×** |
| **L1** dense lateral, D4 per-cell | 408,580,416 | 389.65 | 2.40× |
| **L1** dense lateral, D8 shared | 408,580,416 | 389.65 | 2.40× |
| **L1** dense lateral, D8 per-cell | 817,160,832 | 779.31 | **4.79×** |
| **L2** = L1 + vertical + boundary, D4 shared | 324,028,464 | 309.02 | **1.90×** |
| **L2** D4 per-cell | 528,318,672 | 503.84 | 3.10× |
| **L2** D8 shared | 528,318,672 | 503.84 | 3.10× |
| **L2** D8 per-cell | 936,899,088 | 893.50 | **5.50×** |
| **L3** sparse @ 1 % | 28,242,782 | 26.93 | **0.17×** |
| **L3** sparse @ 5 % | 103,194,709 | 98.41 | 0.61× |
| **L3** sparse @ 20 % | 384,264,435 | 366.46 | 2.25× |
| **L3** sparse @ 100 % (degenerate) | 1,883,302,976 | 1796.06 | 11.05× |
| **L4** sparse + atom @ 1 % | 46,980,764 | 44.80 | **0.28×** |
| **L4** sparse + atom @ 5 % | 196,884,618 | 187.76 | **1.15×** |
| **L4** sparse + atom @ 20 % | 759,024,070 | 723.86 | 4.45× |
| **L4** sparse + atom @ 100 % (degenerate) | 3,757,101,152 | 3583.05 | 22.04× |
| *land-only:* **L2** D8 per-cell dense | 711,382,708 | 678.43 | 4.17× |
| *land-only:* **L4** sparse + atom @ 5 % | 143,692,990 | 137.04 | **0.84×** |
| *land-only:* **L4** sparse + atom @ 20 % | 570,522,614 | 544.09 | 3.35× |

### The arithmetic, so it can be re-derived by hand

```
L1 D4 shared      = 25,536,276 slot-chapters × 2 faces × 4 B
L1 D4 per-cell    = 25,536,276 × 4 × 4 B
L1 D8 shared      = 25,536,276 × 4 × 4 B
L1 D8 per-cell    = 25,536,276 × 8 × 4 B
L2 D4 shared      = 25,536,276 × 3 × 4 B  +  4,398,288 boundary × 4 B
L2 D4 per-cell    = 25,536,276 × 5 × 4 B  +  4,398,288 × 4 B
L2 D8 shared      = 25,536,276 × 5 × 4 B  +  4,398,288 × 4 B
L2 D8 per-cell    = 25,536,276 × 9 × 4 B  +  4,398,288 × 4 B
L3 @ s            = 234,224,772 faces × s × 8 B   + CSR 9,504,800 B
L4 @ s            = 234,224,772 faces × s × 16 B  + CSR 9,504,800 B
L2 land-only      = 177,845,677 land faces × 4 B
L4 land-only @ s  = 177,845,677 × s × 16 B + CSR 1,416,448 B
```

Note **L1 D4 per-cell ≡ L1 D8 shared** (both 4 faces) and **L2 D4 per-cell ≡ L2 D8
shared** — the sharing question and the connectivity question are the *same* 2×,
so they can be traded against each other one-for-one.

---

## 5. The knee — the number that will actually drive the decision

### (a) Sparsity — for a sparse layout, what fraction of faces may be non-zero?

| layout | crosses **1× T** | crosses **4× T** | crosses **10× T** |
|---|---:|---:|---:|
| **L3** (flux only, 8 B/face) | **8.59 %** | 35.88 % | 90.47 % |
| **L4** (flux + atom, 16 B/face) | **4.30 %** | 17.94 % | 45.23 % |

**The CSR index floor alone is 9.06 MiB = 0.056× T, paid at any sparsity** — even
a record with zero non-zero faces costs that. It is 5.6 % of T, so it is not the
constraint; the payload is.

Read this the other way: **L4 stays under 1× T as long as fewer than ~4.3 % of
faces carry flow.** Whether the real solve lands under that is exactly the
unknown, and it is the single most decision-relevant unmeasured quantity left.

### (b) Slots per cell — for the dense layout (L2 D8 per-cell)

| crosses | mean units/cell |
|---|---:|
| **1× T** | 3.46 |
| **4× T** | 13.82 |
| **10× T** | 34.56 |
| *measured today* | **18.64** |

Today's world is already past the 4× knee for the fully-dense layout, and it is
**not** near the 10× knee. The dense-layout cost is not a cliff we are standing
on the edge of — we are already over the 4× line and there is 1.9× of headroom
before 10×. This crossing is also the one that moves if the record's slot count
ever grows (finer chapters, less merging, a coarser cell at `Extent::Large`).

### (c) Chapter count K — for the dense layout (L2 D8 per-cell)

| crosses | K |
|---|---:|
| **1× T** | 1.5 |
| **4× T** | 5.9 |
| **10× T** | 14.8 |
| *measured today* | **8** |

Holding the per-chapter slot population fixed. K is a design knob
(`DeepConfig::chapters`, currently 8, chosen for Earth-orogeny-length chapters):
**doubling K to 16 puts the dense layout at ≈ 10.8× T.** Any mechanism that
effectively multiplies the time axis — a sub-chapter cadence for § 5's episodic
events, for instance — prices out on this row.

---

## 6. What this means for the design

*(This section states costs. It does not choose.)*

**The record is already the field.** 90.8 % of today's `DeepField` residency is the
strata record. So the flow record is not "one more plane"; whatever multiple of T
it costs, it is essentially a multiple of *the strata record*.

**Every dense layout is between 1.2× and 5.5× today's whole field.** The spread is
entirely two binary choices — D4 vs D8 (2×) and shared vs per-cell (2×) — so the
dense family spans exactly 4× end to end, and the two knobs are interchangeable.
The cheapest honest dense layout (L2, D4, shared) is **1.90× T = 309 MiB**; the
most conservative (L2, D8, per-cell) is **5.50× T = 894 MiB**.

**Sparse is a different regime, not a discount.** L4 at a plausible 5 % is
**1.15× T**; at 20 % it is **4.45× T**. The whole question is where the real solve
lands between those, and nothing measured here can answer it — only the sibling's
flux solve can. What *is* settled is that the sparse layout's fixed index cost
(9.06 MiB) is negligible, so sparsity is not being bought with structural
overhead.

**Land-only is a real 23 % discount and a bad-value one.** Restricting the record
to land cells removes 22.8 % of the causal slot-chapters but forecloses turbidity
currents, contourites, submarine canyons and fans — all of which flow.md § 5
explicitly lists as *representable*. The numbers say: it saves less than a factor
of 1.3, in exchange for a named foreclosure. Reported for completeness because it
is the first thing anyone tries.

**Do not build it as `Vec` per (cell, slot).** This is measured, not asserted. The
existing `FactLedger` is `Vec<Vec<Fact>>` keyed exactly that way, and 98.8 % of its
inner vectors are empty; **89 % of its 150 MiB heap is empty headers**. The strata
record's own `Vec` doubling slack is another 54 MiB (33 % of T). The naive shape
would spend the majority of the flow record's bytes on the absence of flow.

### Which of flow.md § 9's open questions these numbers bear on

**Q1 (Cost) — answered as far as geometry can answer it.** The measured
multipliers are: 297,025 cells × 460 m, 8 chapters, **25,536,276 causal (slot,
chapter) pairs** (not 44.3 M — the causal correction is a free 1.73× saving), and
a 5.5M-slot record whose mass is 77 % on 15 % of the cells. Every layout's price
is in § 4. The residual unknown is sparsity, which is not measurable before the
solve exists. Note the ratified stance ("gen is free… resident memory is still
sacred") makes this specifically a *resident* question — and § 4's tier table
already says the record is "the only channel between tiers", which is the
architectural reason it cannot simply be re-derived on demand.

**Q2 (Sub-face parallel channels) — priced.** Allowing `n` channels per face is a
straight `n×` on the payload (the CSR index is unchanged). Concretely: at L4/5 %,
2 channels per face goes 1.15× T → **≈ 2.24× T**; the L4 1×-T sparsity crossing
halves from 4.30 % to **2.15 %**. In the dense layouts it doubles an already
3–5.5× cost, i.e. **sub-face channels are affordable in the sparse family and not
in the dense one.** That is a real coupling between two decisions § 9 currently
lists separately.

**Q4 (Episodic events within a chapter) — priced by knee (c).** Any sub-chapter
event mechanism multiplies the time axis. The chapter knee says K = 5.9 crosses 4×
and K = 14.8 crosses 10×, so **doubling the effective time resolution (K = 16) puts
the dense layout at ≈ 10.8× T**. A sub-chapter mechanism is therefore a
sparse-layout-only proposition at current K, or requires K to come down.

**§ 2.2's "a face is shared … agree from both sides by construction" — a flagged
gap, not a cost.** The claim is stated for faces, but the record is per **stratum
slot**, and slot indices do **not** align between adjacent columns (slot 3 in cell
A is not slot 3's depth in cell B). Sharing therefore holds for the *cell pair* but
needs a stated pairing rule at the *slot* level — by depth, presumably, not by
index. The measurement is agnostic about which is right and reports both: it is
uniformly a **2×**. Worth resolving in the design, because it is the same 2× as
the D4/D8 choice and the two get confused easily.

**§ 9 Q5 (which fluids ship first) — unaffected.** The atom carries one fluid id
per fact (A9); additional fluids add *facts*, not fields, so they land on the
sparsity axis already modelled, not on a new one.

---

## Reproducing

```
cargo run --release -p dc-worldgen --example flow_cost_probe
```

Deterministic in `(seed, extent)` = `(1337, Medium)`; ~46 s wall clock, of which
~45 s is the two deep-time runs. Nothing in the probe is on a generation path and
it writes no files. If any of A1–A10 is wrong, change the named `const` and re-run
— the whole table re-derives.
