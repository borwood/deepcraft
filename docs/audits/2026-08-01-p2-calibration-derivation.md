# P2 — the erosion-calibration re-pick: LITERATURE DERIVATION

*2026-08-01 · read-only research pass · no cargo was run by this agent*

> **RULING — the craton target (user, 2026-08-01): the PRESENT-STATE band governs**
> (1–10 m/Myr; derived target ≈ 2.6). The user's reasoning, recorded because it settles
> the run's *interpretation*, not just the number: *"our 500 Myr are not the entire
> history of 'the world' — they're where we draw the line of compromise instead of
> simming the boring billion."* The run models a mature craton state, not a full
> Wilson-cycle integral, so the run-integrated 10–20 band is the wrong referent; the
> 5–10 km Phanerozoic stripping figure stays a literature fact, not an acceptance bar.
> Multiplier bracket proceeds as derived: **M ∈ [60, 240], central 100–200.**
>
> **`uplift_scale` (§ 6's sibling flag) — integrator-settled, measurement-first:** the
> probe runs gain the sustainability column (rebound fraction vs the Airy prediction —
> a physics identity, not a taste); if the craton band proves unsustainable at the
> chosen M, `uplift_scale` returns as its own ratification topic WITH numbers. Not
> pre-decided. *(Mutable header; the body below is the dated derivation.)*

**What this is.** The literature half of P2 (`dependency-graph.md` § 2 row P2; § 4 item 4,
*"needs a literature pass, not an engineering one"*). It derives the **target**, brackets the
**multiplier**, splits the recorded sensitivity evidence into what survives journal/0122 and
what it voided, answers the **register-vs-rates** question, and hands the integrator a
runnable measurement plan.

**What this is NOT.** It does not pick the number. Every candidate below is a bracket with its
arithmetic shown and its unmeasured term named. **`calibrated_rates` is an appearance-class,
user-owned flip** (journal/0114, `production_config`, ROADMAP), and three of the five inputs a
final pick needs have never been measured under the fixed operator.

**Provenance rule for everything below.** Numbers are either (a) read out of the corpus with a
citation, (b) read out of the literature with a source and a confidence grade, or (c) arithmetic
over (a) and (b) with the chain shown. Nothing is a run of this repo's code by this agent.

---

## 0. The one-paragraph answer

The world's own tectonics fix the target: Airy compensation plus the measured rock uplift put
the **steady-state denudation fixed point at 2.63 m/Myr**, and that number lands on the
**cratonic bedrock-lowering band (1–4 m/Myr, Namib mean ≈ 2.5)** from two entirely independent
directions. The acceptance quantity must be **D3** (bedrock erosion), not D1 (corrections #60 —
D1 is a gross shoreline flux and inflates exactly when creep gets fast, which is what the fixed
operator did). Reaching it costs a uniform multiplier of **order 10²** — bracketed at
**M ∈ [60, 240], most likely 100–200** by three independent chains that overlap. **The
supply/transport PAIR collapses to one axis** under the fixed operator, and that collapse is a
derived result rather than an assumption. **The register does not move**: register and rates are
degenerate for the *rate* diagnostic and NOT degenerate for the *stripped-thickness* diagnostic,
and the literature anchor everyone quotes ("a craton strips 5–10 km") is on the thickness — so it
breaks the degeneracy and puts the entire correction on the rates. Two things block closing the
derivation: **the mean cover taper `⟨exp(−H/H*)⟩` has never been printed** (mean `H` is not a
substitute, and § 4.3 shows why by falsifying the mean-field model against the corpus's own
ladder), and **no denudation number exists under the fixed operator at all** — journal/0122's
ladder reports cover, concavity and hollows, and not one column of D1/D3/D4.

---

## 1. The target bands, cited, with per-number confidence

### 1.1 The bands

| band | m/Myr | source | confidence |
|---|---|---|---|
| **hyperarid / hypothermal floor** | **0.19 – 2.1** (regolith, McMurdo Dry Valleys) | Morgan et al. 2010, *JGR-Earth Surface* 115, F03027 — cosmogenic ¹⁰Be/²⁶Al regolith erosion rates, McMurdo Dry Valleys | **high** on the 0.19–2.1 range (re-verified this pass) |
| — Arena Valley specifically | **0.53** (steady-state surface denudation, Sirius Group depth profiles) | Morgan et al. 2010 | **medium-high**; ⚠ **this corrects the probe**, see § 1.3 |
| **hyperarid Atacama** | **~1** (Coastal Cordillera / southern desert) up to **46** (Western Cordillera) | Kober et al. 2007 *Geomorphology* 83, 97–120; Placzek et al. 2010 *EPSL* 295, 12–20; Starke et al. 2017 *JGR-ES* | **medium**; the *desert as a whole* spans ~0.5–40, so "Atacama = the floor" holds only for the hyperarid core |
| **stable-craton BEDROCK OUTCROP** | **1 – 4** (Namib mean ≈ **2.5**; Australian bedrock 0.3 – 5.7 max-limiting) | Bierman & Caffee 2001 *Am. J. Sci.* (Namib); Bierman & Caffee 2002 *GSA Bull.* (Australian landforms) | **high** on the 1–4 framing and the Namib ≈ 2.5 mean; **medium-high** on 0.3–5.7 |
| **tectonically stable regions, general** | **1 – 20** | Bierman & Caffee 2001/2002 as framed in the current literature | **high** |
| **global ¹⁰Be OUTCROP median** | median **5.4**, mean **12 ± 1.3**, **n = 450 outcrops** | Portenga & Bierman 2011, *GSA Today* 21(8), 4–10 | **high** — re-verified this pass |
| **global ¹⁰Be BASIN median** ← *the row that is missing* | median **54**, mean **218**, drainage basins | Portenga & Bierman 2011 | **high** — re-verified this pass |
| — the whole compilation | **n = 1599** (outcrops + basins together) | Portenga & Bierman 2011 | **high**; ⚠ **this corrects the probe**, see § 1.3 |
| **Phanerozoic global continental mean** | **16** (from preserved sedimentary-rock volumes, ≈5 Gt/yr) | Wilkinson & McElroy 2007, *GSA Bull.* 119, 140–156 | **high** — re-verified |
| — modern natural, ice-free continental | **≈ 60** (60 mm/kyr) | Wilkinson & McElroy 2007 | **high**; the probe's `62` is the same number to rounding |
| **passive-margin upland** | **27 ± 4** (Great Smokies, ¹⁰Be catchment-averaged) | Matmon, Bierman et al. 2003 *Geology* | **medium-high** — not re-verified this pass |
| **active orogen** | **3 000 – 12 000** | Dadson et al. 2003 *Nature* (Taiwan 3–6 mm/yr); Herman et al. 2013 *Nature*; Koppes & Montgomery 2009 *Nat. Geosci.* | **medium** — not re-verified this pass; the Taiwan figure is the best-established of the three |
| **craton stripping over a Phanerozoic span** | **5 – 10 km / 500 Myr = 10 – 20 m/Myr** | corpus claim (`corrections.md` #56; Kola 3–5 km; Pilbara, Morón et al. 2020; South African plateau ≥4.5 km since 130 Ma) | ⚠ **NOT re-verified, and internally inconsistent — see § 1.2** |

**Citations I could NOT confirm and would not put weight on:**

- **`Ritter et al. 2023 JGR-ES` (Atacama near-stasis, ²¹Ne exposure ages 9–37 Ma)** — cited in
  `denudation_probe.rs`'s `BANDS` table. I could not find this paper. The well-established
  references for Atacama antiquity are **Dunai et al. 2005** *Geology* 33, 321 (Oligocene–Miocene
  aridity from exposure dating of erosion-sensitive landforms) and **Placzek et al. 2010** *EPSL*.
  **Flag as unverified; do not cite it in a ratification.**
- **`Veselovskiy et al. 2019 Tectonics` (Fennoscandia AFT 1–2.5)** — unverified. The Fennoscandian
  AFT literature is real and the magnitude is plausible; the specific attribution is not
  something I can stand behind.
- **`Morón et al. 2020` (Pilbara)** — unverified.

### 1.2 ⚠ The corpus asserts two craton targets that disagree by 2–10×, and nobody has reconciled them

Both of these are live, in `corrections.md` #56, `earth-processes.md` § 3e-2's FLAG, and
`denudation_probe.rs`'s own closing paragraph — within a few hundred lines of each other:

1. *"stable-craton bedrock runs **1–10 m/Myr**"* — the acceptance band every P2 artifact names.
2. *"a real craton strips **5–10 km** over a Phanerozoic span"* — which over 500 Myr is
   **10–20 m/Myr**, i.e. **at or above the top of band (1)**, and squarely on Wilkinson &
   McElroy's Phanerozoic continental mean of 16.

Arithmetic on the corpus's own sub-figure makes it sharper: **4.5 km / 130 Myr = 34.6 m/Myr**
for the South African plateau — 3.5× above the *top* of band (1) and firmly in passive-margin
territory.

**They are not the same measurement, and the reconciliation decides what P2 is aiming at.**
Cosmogenic outcrop rates measure the *present-day* surface of a landscape at its slowest; AFT
and sediment-volume exhumation integrate over episodes (rifting, epeirogeny, glaciation, plateau
uplift) that are several times faster, and the km-scale numbers are dominated by rift flanks and
plateau margins rather than shield interiors.

**Which one governs is a USER call and it is not a small one:**

- **Target (1), "the shipped world's present state looks like a craton interior"** → D3 ≈ 1–10,
  preferably 1–4 (the bedrock-outcrop sub-band, § 1.4).
- **Target (2), "the 500 Myr run strips what a real craton strips"** → D3 ≈ 10–20, i.e. **another
  4–8× on top of target (1)**, and 5–10 km of exhumation over the run.

**This audit derives against target (1)**, for a reason that is itself derived rather than
chosen: § 4.1 shows the world's own tectonics can only *sustain* target (1). Reaching target (2)
requires `uplift_scale` to move as well, which is a second uncalibrated constant and a second
user call (§ 5.4).

### 1.3 ⚠ Three corrections owed to `denudation_probe.rs`'s `BANDS` table

The table at `crates/dc-worldgen/examples/denudation_probe.rs:970-1013` is the acceptance
instrument's literature anchor. Three rows need repair, and one row is **missing and is the most
important one on the page**:

1. **MISSING: the ¹⁰Be *basin* median, 54 m/Myr (mean 218).** The table carries only the
   **outcrop** median (5.4) and labels it *"global outcrop median (10Be, n=1599)"*. Portenga &
   Bierman 2011 report **outcrops n = 450, mean 12 ± 1.3, median 5.4** and **drainage basins mean
   218, median 54**; `n = 1599` is the whole compilation, not the outcrop subset.
   **Why this matters more than a footnote:** the probe's headline **D1 is explicitly a
   catchment-averaged quantity** — its own doc comment says it is *"the reading the literature
   measures (sediment yield at a gauging station; a cosmogenic ¹⁰Be catchment average)"* — and it
   is being compared against an **outcrop** band. That is a category mismatch of exactly one order
   of magnitude, in the instrument that decides P2. (It does not change the *direction* of
   journal/0111's finding — it makes it worse — but it does change which band a given arm "lands
   in", which is the acceptance test.)
2. **`Arena Valley ~0.19` is the range minimum wearing a site's name.** Morgan et al. 2010's
   regolith range is **0.19–2.1 m/Myr**; Arena Valley's own steady-state surface denudation from
   the Sirius Group depth profiles is **0.53 m/Myr**. The probe hard-codes `0.19` into its
   `READING IT` caption as *"McMurdo Dry Valleys bedrock, ~0.19"*.
3. **`Ritter et al. 2023 JGR-ES` is unverifiable** (§ 1.1). Replace with Dunai et al. 2005 /
   Placzek et al. 2010, or drop the sub-clause.

*These are corrections to a published claim inside a gate-run example, so per CLAUDE.md §
Gates — "a printed caption is a published claim the gate cannot check" — they ride the P2 build
slice, in the same diff as the re-pick.*

### 1.4 The band the acceptance should actually be stated on

**D3, against 1–4 m/Myr, centred on ≈ 2.5.**

Three reasons, all of them from the corpus or the sources rather than from preference:

- **D1 is not admissible as the acceptance quantity.** `corrections.md` **#60** withdrew the
  D1/D3 cross-check: D1 is a **gross** land→sea edge flux and the ±35 m sea-level sinusoid sweeps
  the shoreline four times over the run, so cover that crosses, is stranded and crosses again is
  counted every time. journal/0114 measured the gap opening as fluxes grow (D1/D3: 1.03 at 1×,
  3.26 at 45×, 41.23 at 1000×). **The fixed operator makes creep faster, so it makes D1 worse.**
  D3 is a per-cell rock-removal plane with no shoreline in it.
- **D3 is a bedrock-lowering plane, so its literature counterpart is the bedrock-outcrop rate,
  not the catchment rate.** Portenga & Bierman's outcrop/basin split (5.4 vs 54) and the
  cratonic-settings finding that *bedrock denudation (1–4 m/Myr) is systematically lower than
  basin-wide rates* are the same distinction the probe's D1/D3 split already makes internally.
  **Match the instrument to the band: D3 → outcrop bands; D1 → basin bands.**
- **The world's own Airy fixed point lands there anyway** (§ 4.1): **2.63 m/Myr**, against the
  Namib bedrock mean of ≈ 2.5. Two independent derivations, one from two densities and a measured
  uplift, one from a 2001 cosmogenic dataset, agreeing to 5 %.

---

## 2. The engine, in units — and the arithmetic nobody had done

| quantity | value | source |
|---|---|---|
| recorded span | 500 Myr | `earth-processes.md` § 3e-2 decision 5, **RATIFIED 2026-07-19 (user)** |
| iterations | 200 | `DEEP_ITERATIONS`, `field.rs:48` |
| **register** | **2.5 Myr / epoch** | 500/200; corroborated by `chapters = 8` × 62.5 Myr |
| `weathering` | 0.02 m/epoch = **8 mm/Myr** | `grid.rs:195` (its own doc states the 8 mm/Myr) |
| `diffusion` | 0.12 (dimensionless/epoch) | `grid.rs:166` |
| `k_transport` / `k_bedrock` | 0.0016 / 0.0011 | `grid.rs:149,151` |
| `h_star` (`H*`) | 3.0 m | `grid.rs:158` |
| `uplift_scale` | 3.0 m/epoch, unit-rate province | `grid.rs:147` |
| deep cell | 460 m | `DEEP_CELL_M` |
| `Extent::Medium` grid | 545 × 545 = 297,025 cells ≈ 250 km across | journal/0111 |
| land cells | 44,264 (14.9 %) ≈ 9,366 km² | journal/0111 |
| `CREEP_MAX_EDGE_COEFF` | 0.125 (the 1/8 monotonicity bound) | `erosion.rs:789` |
| `EROSION_CALIBRATION` | **45.0**, and `calibrated_rates` ships **false** | `field.rs:158` |

**The weathering law** (`erosion.rs::weather` → `weather_one_cell`, grouping preserved):

```
ΔH = weathering × ((biotic × weatherability) × frost) × exp(−H / H*)
```

so the **bare-rock nominal supply** is `0.02 / 2.5 = 0.008 m/Myr`, and the measured 1× bedrock
erosion is `D3 = 0.0107 m/Myr` — **1.34× the nominal**. That factor is the product of the three
modulators, the Jensen gap between `⟨exp(−H/H*)⟩` and `exp(−⟨H⟩/H*)`, and `k_bedrock` incision.
**It has never been decomposed, and § 4.3 shows it is the term that decides the answer.**

**One scaling knob, and the ladder is directly readable as candidate values.** `erosion_budget`
and the shipped calibration both go through `field.rs::scale_erosion_rates` (all four rates), and
the probe's `cfg_uniform(mult)` applies `erosion_budget` on top of `calibrated_rates: false` — so
**a ladder rung `M` is exactly "what would `EROSION_CALIBRATION = M` produce?"** (asserted
byte-identical at `M = 45` in `tests/calibrated_rates.rs`).

---

## 3. The stale-vs-valid split of the sensitivity evidence

journal/0122 fixed the hillslope operator (sub-cycling to the 1/8 bound) and **inverted the
asymmetry the entire pre-0122 calibration argument rests on**. Do not read the 0114 ladder as
evidence about the world we now ship.

### 3.1 VOID — measured against the capped operator, and inverted by the repair

| claim | where | why it is void |
|---|---|---|
| **cover THICKENS with the multiplier**: mean `H` 4.6 → 8.8 → 43.9 → 118.8 → 359 → 782 m at 1/10/45/100/300/1000× | journal/0114 ladder; `field.rs:69`; stubs #27 | journal/0122 measures **thinning**: 3.91 → 4.40 → 3.02 → 2.18 → 1.65 → **1.40** at 1/3/5/10/20/45×. **Opposite sign.** |
| *"the landscape buys denudation by burying itself"* | `field.rs:117`; journal/0114 headline | the mechanism it names (transport ceiling) is gone |
| **"catchment export ∝ mean regolith thickness"** (`D1/mean H` flat within 30 % over 10–100×) | journal/0114 § *What the `mean H` column says*; stubs #27 | the proportionality *was* the conveyor. `stubs.md` #27 is stamped **⚠ THE CONVEYOR IS GONE** |
| **"100× on transport alone buys 1.6×"** / *"raising `diffusion` cannot speed a conveyor up"* | journal/0114; `field.rs:73`; stubs #27; corrections #56 corollary table | the cap that made it a conveyor is removed; limiter binding falls 96.0 → 77.5 % calibrated, 88.7 → 79.1 % shipped. **This is the single most load-bearing void row** — it is the whole justification for the supply/transport *pair* |
| **"no multiplier reaches the band with a world left in it"** / *"1 m/Myr costs ~150 m of mean cover"* / *"the first row in the band is 100× and it carries 119 m"* | journal/0114; `field.rs:106-117`; stubs #24(b); ROADMAP | derived from the thickening ladder |
| **the whole D1/D3 ladder**: 0.0110/0.0107 · 0.0765/0.0449 · 0.4142/0.1271 · 1.3377/0.2767 · 10.5825/0.7108 · 57.1331/1.3857 | journal/0114 § *The ladder* | every row is a capped-operator world. **⚠ AND ITS 1× ROW TOO** — journal/0122 moved the *shipped* world (mean h 4.57 → 3.91), so even the baseline is stale, by an expected ~1.2× |
| **the pit ladder**: 0 at 1×, 44 at 5× (deepest 45 m), 66 at 10×, 148 at 45× (deepest 112 m) | journal/0114; `field.rs:122-128`; stubs #29 | journal/0122: **nothing anywhere is deeper than 10 m until 45×**, deepest at 5× is **2.9 m**. *"There is a safe multiplier underneath. There are five of them."* |
| **"45 is the largest multiplier that keeps relief within 5 % (+4.6 % at 45×)"** — journal/0114's **binding criterion** | journal/0114 § *The number that was not chosen* item 1 | already falsified once by the walk (**corrections #61** — an aggregate ridden without a neighbour-relative measure), and now measured on the wrong operator. **Relief under the fixed operator at any multiplier above 1× is UNMEASURED** — journal/0122's ladder has no relief column |
| **"mean regolith lands in the 30–60 m deeply-weathered-shield range"** (criterion 2) | journal/0114 | at 45× the fixed operator gives **1.40 m**. The criterion now points the other way: cover is *below* the shipped world's |
| **"D1/D4 = 0.91, the landscape approaches steady state"** (criterion 4) | journal/0114 | D1 is inadmissible (#60) and the number is capped-operator |

### 3.2 VALID — survives the operator repair

| claim | where | why it survives |
|---|---|---|
| the **1× baseline against the literature**: D1 0.0110, D3 0.0107, D4 0.4095, D2 −0.4084; 5.48 m stripped over 500 Myr; 493× below the outcrop median | journal/0111; corrections #56 | the ~10³ magnitude and every literature comparison. *(Expect a ~1.2× upward correction on re-measure — the shipped world moved.)* |
| **the cover taper `exp(−H/H*)`, `H* = 3 m`, is the coupling** | `erosion.rs::weather`, `grid.rs:158` | a code fact, not a measurement |
| **D1 is a gross upper bound, D3 is the sound instrument** | corrections #60; journal/0114 § *the two instruments separate* | strengthened: faster creep ⇒ more shoreline re-crossing |
| **the Airy arithmetic**: `f = (ρ_m − ρ_c)/ρ_m = 0.152`, ceiling ≈ 2.70 m/Myr | journal/0114; `isostasy::RHO_MANTLE` / `rho_crust` | two densities and a measured uplift. ⚠ *one caveat, § 4.1* |
| **"the shape is right and the clock is wrong"** — a weathering-limited, creep-routed, river-minor, regolith-armoured low-relief craton | journal/0111; corrections #56 | qualitative; journal/0122 changed magnitudes, not character |
| **the joint-lever finding as a statement about the OLD world** (100× supply → 1.4×; 10× transport → 1.7×; together → 132×) | corrections #56 | valid as history. **Invalid as a design input** — see § 4.4 |
| **`scale_erosion_rates` is the single scaling site**; stubs #24(a) discharged and cannot recur | `field.rs:88`; `tests/calibrated_rates.rs` | structural |
| **the pre-flip blocker is dissolved** — the three excluded agent magnitudes may scale; byte-identicality explicitly not wanted | user 2026-07-29; stubs #28 **DISSOLVED**; ROADMAP | a user ruling |

### 3.3 NEVER MEASURED (not stale — absent)

- **Any denudation number under the fixed operator, at any multiplier, including 1×.**
  journal/0122's ladder columns are `sub · peak eff_diff · conc(h) rms · conc(h)/h̄ · ACF · surf
  conc · surf ACF · hollows · deepest · mean h · gen`. **There is no D1, no D3, no D4, no relief,
  no land-cell count.** The entry says so itself: *"a derivation to redo against a published band."*
- **`⟨exp(−H/H*)⟩`, the area-weighted mean cover taper.** Never printed anywhere. § 4.3 shows
  mean `H` is not a usable substitute.
- **The modulator product `⟨(biotic × weatherability) × frost⟩`.** Never printed; it is a
  1.34-to-6× ambiguity sitting directly in the derivation.
- **The supply/incision split of D3** (`weathering` front descent vs `k_bedrock`).

---

## 4. The derivation chain

### 4.1 Chain A — the TARGET, from the world's own tectonics (register-invariant)

Airy compensation returns `1 − f = 84.8 %` of every eroded metre as rebound and lets the surface
drop by `f = 0.152` of it. Decompose the measured rock uplift into tectonic and isostatic parts:

```
U_tect = D4 − (1 − f)·E          = 0.4095 − 0.848 × 0.0107 = 0.4004 m/Myr
E*     = U_tect / f              = 0.4004 / 0.152          = 2.634 m/Myr
```

Self-consistency: at `E = E*`, `D4* = U_tect + 0.848·E* = 0.400 + 2.234 = 2.634 = E*` — at
topographic steady state rock uplift equals denudation, which is the check that the algebra is
the right algebra.

> **TARGET: D3 → 2.63 m/Myr**, with the acceptance band **1–4 m/Myr** (§ 1.4). Independent
> corroboration: the **Namib bedrock mean ≈ 2.5 m/Myr** (Bierman & Caffee 2001). Two derivations
> that share no inputs, 5 % apart.

**⚠ A defect in the probe's version of this, which matters exactly when the calibration lands.**
`denudation_probe.rs:769` computes `ceiling = d.rock_uplift / f_airy` — i.e. it divides the
**measured rock uplift (which already contains the rebound)** by `f`, instead of the **tectonic**
uplift. The two agree today only because erosion is negligible (`0.848 × 0.0107 = 0.009`, 2 % of
D4), so the printed 2.70 and the correct 2.63 differ by 2.6 %. **They diverge as soon as the
calibration works** — the formula is valid only in the regime the calibration exists to leave.
Fix it in the P2 slice: subtract `(1 − f)·D3` from `D4` before dividing.

**And a second, larger caveat that is a genuine finding.** The sim's isostasy does **not** track
Airy. From journal/0114's ladder (`D4 = D1 / (D1/D4)`):

| M | D3 measured | D4 measured | D4 Airy-predicted `0.400 + 0.848·D3` | ratio |
|---|---|---|---|---|
| 1 | 0.0107 | 0.4095 | 0.409 | 1.00 |
| 45 | 0.1271 | 0.455 | 0.508 | 0.90 |
| 100 | 0.2767 | 0.474 | 0.635 | 0.75 |
| 300 | 0.7108 | 0.528 | 1.003 | 0.53 |
| 1000 | 1.3857 | 0.617 | 1.575 | 0.39 |

The rebound the sim actually delivers falls to **~40 % of Airy** at high erosion — consistent
with `tectonic_history`'s *smoothed* Airy isostasy (a spatial filter removes local rebound), but
it has never been stated. **Consequence:** the sim's own fixed point (`D3 = D4`) sits *below*
2.63 — from the capped ladder it crosses at **M ≈ 160–200 with D3 ≈ 0.5 m/Myr**. Under the fixed
operator D3 rises faster with M, so the crossing moves, but the *shape* of the problem is:
**the world may not be able to hold 2.63 m/Myr in steady state without `uplift_scale` moving too**
(§ 5.4). This is measurable in one probe run and is on the plan.

### 4.2 Chain B — the MULTIPLIER, from the supply side

At the supply limit (cover thin, taper → 1) the bedrock-lowering rate is

```
D3_max(M) = (weathering / myr_per_epoch) · M · ⟨mod⟩ = 0.008 · M · ⟨mod⟩
```

and the 1× measurement pins the *product* but not the factors:

```
⟨mod⟩ · ⟨taper⟩₁ (+ incision share) = D3₁ / 0.008 = 0.0107 / 0.008 = 1.34
```

So the answer is a **one-parameter family in the unmeasured `⟨taper⟩₁`**:

| assumed `⟨taper⟩₁` | implied `⟨mod⟩` | supply ceiling `D3_max(M)` | **M for D3 = 2.63** |
|---|---|---|---|
| 0.22 *(the mean-field value at `H̄` = 4.57)* | 6.1 | 0.049 · M | **54** |
| 0.45 | 2.98 | 0.0238 · M | **110** |
| 0.75 | 1.79 | 0.0143 · M | **184** |
| 1.00 *(taper already open at 1×)* | 1.34 | 0.0107 · M | **246** |

These are **lower bounds** — the true M is higher than each row, because a ceiling is only
approached. And the last row is exactly the naive "D3 scales linearly with M" answer, which is
the same number the thickness axis gives independently: **1315 m of exhumation needed / 5.48 m
delivered = 240×**.

> **Chain B: M ∈ [54, 246], and the width is *entirely* the unmeasured `⟨taper⟩₁`.**

### 4.3 ⚠ The mean-field taper model is FALSIFIED — recorded because it is the obvious next move

The tempting closure is `D3(M) = D3₁ · M · exp(−H̄(M)/H*) / exp(−H̄₁/H*)`, using journal/0122's
measured `mean h` ladder. **Check it against journal/0114's own D3 ladder before trusting it:**

| M | `H̄` (0114) | `exp(−H̄/3)` | predicted D3 | **measured D3** | error |
|---|---|---|---|---|---|
| 1 | 4.62 | 0.2144 | 0.0107 | 0.0107 | — (anchor) |
| 10 | 8.81 | 0.0531 | 0.0265 | **0.0449** | 1.7× low |
| 45 | 43.87 | 4.45 × 10⁻⁷ | ~1 × 10⁻⁶ | **0.1271** | **10⁵ low** |

**The model is not slightly wrong; it is wrong by five orders of magnitude.** The mechanism is
Jensen's inequality on a convex function: `⟨exp(−H/H*)⟩ ≫ exp(−⟨H⟩/H*)` whenever `H` is
heterogeneous, and at `H̄ = 44 m` essentially *all* the weathering is happening in the thin-cover
tail (steep ground, shorelines, freshly scoured cells) while `H̄` is set by the thick-cover bulk
that contributes nothing.

> **`mean H` is not a proxy for the coupling term. The coupling term is `⟨exp(−H/H*)⟩`, and this
> repo has never printed it.** That single missing column is why chain B is a 4.5×-wide bracket
> instead of a number, and it is the cheapest thing on the measurement plan.

*Recorded rather than quietly dropped: this is the arithmetic a cold session will reach for
first, and it looks completely reasonable until it is checked against a ladder that already
exists.*

### 4.4 Chain C — the multiplier, from transport REACH

Under the fixed operator creep is a real diffusion again. Characteristic spread per epoch is
`√(2D)` cells; over `N = 200` epochs the run-integrated reach is

```
ℓ_run(M) = √(2 · D̄(M) · N) cells,   D̄ = diffusion · M = 0.12 M
         = √(48 M)  =  6.93 √M cells
```

Land is 44,264 cells; as one blob that is ~210 cells across, so the **characteristic distance
from the interior to the sea is ≈ 105 cells (48 km)**.

| M | `ℓ_run` (cells) | (km) | vs the 105-cell half-width |
|---|---|---|---|
| 45 | 46 | 21 | 0.44 — interior cannot reach the sea |
| 100 | 69 | 32 | 0.66 |
| **225** | **104** | **48** | **1.0 — transport stops binding** |
| 400 | 139 | 64 | 1.3 |

Using the *peak* `eff_diff` (0.278·M, journal/0122's derivation) instead of the config value
gives `10.5√M` and the crossing at **M ≈ 100**.

> **Chain C: transport ceases to be the limiter at M ≈ 100–230.** Caveat, stated: creep is
> downslope-**advective**, not an isotropic random walk, so this is an order-of-magnitude
> argument. It is included because it is *independent* of chains A and B and lands on the same
> decade.

### 4.5 The candidate pairs — and why the pair space collapses to one axis

**The pre-0122 case for a supply/transport PAIR was the transport cap**: neither lever paid
alone (100× supply → 1.4×; 10× transport → 1.7×) *because* `diffuse` was a one-cell-per-epoch
conveyor. journal/0122 removed the cap, and chain C says transport stops binding at M ≈ 100–230
— i.e. **inside the region of interest**. So:

| pair `(S, T)` | what it does | verdict |
|---|---|---|
| **`(M, M)` — uniform, one number** | what `scale_erosion_rates` expresses today | **RECOMMENDED.** With transport non-binding at the target M, a second free parameter has nothing left to buy |
| `(M, M/3)` — cheaper transport | sub-steps `n ∝ T`, so this is a **3× gen-time saving** | measure it once as a *cost* option, never as physics. Risk: re-thickens cover, re-shuts the taper, and walks back toward the 0114 regime |
| `(M, 3M)` — transport ahead of supply | thins cover further, opens the taper, raises the supply-limited D3 per unit S | costs 3× gen for a gain that chain C says is already saturated. **Not worth a rung** unless the ladder shows cover still thick at the target |
| `(M, M)` with `k_transport` split out | fluvial is **0.02 %** of yield | no measurable effect. Do not spend a rung |

> **This collapse is a derived result, not a simplification.** The pair existed because transport
> was capped; the cap is gone; therefore the pair is gone. **It is also the strongest single
> reason P2 is now tractable** — and it is exactly the finding journal/0122 handed forward and
> deliberately did not act on.

### 4.6 The candidate table, with gen cost derived

Sub-step count `n(M) = ceil(peak eff_diff / 0.125)`. Fitting `peak eff_diff ≈ 0.278·M` to
journal/0122's two anchors (0.261 at 1× post-run / n = 2 epoch-matched; 12.600 at 45× / n = 100)
gives **n ≈ 2.22 M**. Fitting `gen ≈ a + b·n` to journal/0122's six-row ladder
(n = 2 → 37.2 s … n = 100 → 233.1 s) gives **gen ≈ 33.2 + 2.00·n seconds** (reproduces the
interior rows to within 2 %).

| candidate M | sub-steps `n` | gen / world | chain B says D3 | chain C: transport binding? | notes |
|---|---|---|---|---|---|
| 45 (today's `EROSION_CALIBRATION`) | 100 | 233 s *(measured)* | 0.5 – 2.2 | **yes** | the incumbent; already known to strip cover to 1.40 m |
| 100 | 222 | 477 s | 1.1 – 4.9 | marginal | first plausible in-band rung |
| **150** | **333** | **699 s** | **1.6 – 7.4** | **near release** | **central estimate** |
| 250 | 555 | 1 143 s | 2.7 – 12.3 | no | brackets the top |
| 400 | 888 | 1 809 s | 4.3 – 19.6 | no | the target-(2) probe (§ 1.2) |

**A five-rung ladder plus both baselines is ≈ 73 minutes of deep-time solve per probe run** (plus
pregen). Gen time is explicitly not a constraint (`worldgen-time-not-a-constraint`), but it is a
workflow fact worth knowing before someone launches it in the foreground.

> **The honest summary: M is order 10², bracketed [60, 240], central estimate 100–200, and the
> bracket cannot be narrowed without two measurements this agent cannot take.**

---

## 5. The register ↔ Myr question

### 5.1 The degeneracy, stated exactly

```
D [m/Myr]  =  (metres removed per epoch) / (Myr per epoch)
```

The sim fixes the numerator. The register fixes the denominator. So for the **rate** diagnostic
the two are **perfectly degenerate**: compressing the register by `r` (`myr_per_epoch = 2.5/r`)
multiplies D1, D2, D3 **and D4** by `r`, exactly as multiplying the rate constants would.

Concretely: closing the 493× gap to the outcrop median by register alone needs
`myr_per_epoch = 2.5/493 = 5.1 kyr`, i.e. a **1.01 Myr total run**.

### 5.2 What breaks the degeneracy — and it is decisive

**The register does not change what is removed. It only relabels how long it took.**

| diagnostic | moves with rates? | moves with register? |
|---|---|---|
| denudation **rate** (m/Myr) | ✅ | ✅ **degenerate** |
| rock-uplift rate (m/Myr) | ✅ (via isostatic rebound only) | ✅ |
| **total thickness stripped over the run (metres)** | ✅ | ❌ **invariant** |
| relief, cover thickness, landform shape | ✅ | ❌ |
| chapters per orogeny, record span | ❌ | ✅ |

The corpus's own literature anchor is on the **invariant** row: *"a real craton strips 5–10 km
over a Phanerozoic span; this world strips **5.48 m**"* (corrections #56, quoting Kola, Pilbara,
the South African plateau). No choice of register turns 5.48 m into kilometres.

> **THE AXIS STATEMENT: the correction goes on the RATES. The register does not move.** The
> degeneracy is real for the rate diagnostic and broken by the thickness diagnostic, and the
> thickness diagnostic is the one the literature actually constrains for deep time.

### 5.3 What a literature-anchored register re-fit *would* use, if it were reopened

`earth-processes.md` § 3e's oldest owed item is *"calibrate iteration↔Myr against a real
orogen"*. For the record, the anchors it would have to weigh:

1. **Orogen life cycle.** `chapters = 8` × 62.5 Myr is already fitted to this: Alpine-style
   collision 30–50 Myr, a full Appalachian-style Wilson cycle ~100 Myr. A 30 Myr chapter forces a
   240 Myr register; a 100 Myr chapter forces 800 Myr. **The ratified 500 Myr sits mid-range and
   is defensible on this anchor** — it is the one calibration in the family that was already
   done, if implicitly.
2. **Craton denudation over Phanerozoic time.** 5–10 km at 1–20 m/Myr admits 250 Myr–10 Gyr.
   Far too loose to constrain anything.
3. **Rock-uplift rate.** `D4 = 0.4095·r` m/Myr against the epeirogenic band for stable interiors
   (≈1–10 m/Myr equivalent) would give `r ∈ [2.4, 24]` — a 21–208 Myr run, **contradicting the
   ratified register**. ⚠ **Do not use this anchor.** `uplift_scale = 3.0` is an S9 constant
   picked so *"orogenic belts build hundreds of metres of net relief over a few hundred
   iterations"* — it is **exactly as uncalibrated as the erosion rates** and of the same vintage.
   Anchoring the register to it is circular.
4. **The one hard bound the register does have** (chain A, § 4.1): the steady-state fixed point
   is `E* = 2.63·r` m/Myr, so the craton band 1–10 admits `r ∈ [0.38, 3.8]`. **The stipulated
   `r = 1` sits comfortably inside.** Past `r ≈ 3.8` the world's own tectonics would make it an
   active margin rather than a craton, and the "textbook low-relief craton" reading (journal/0111,
   surviving) would stop being available.

### 5.4 The second uncalibrated constant, named so it is not discovered later

**`uplift_scale = 3.0` has never been measured against the literature either**, and § 4.1's
isostasy table shows it becomes binding exactly when P2 succeeds:

- Published rock-uplift rates: stable interiors ~0.001–0.01 mm/yr (**1–10 m/Myr**); active
  orogens 1–10 mm/yr (**1 000–10 000 m/Myr**). *(Confidence: medium-high, standard textbook
  ranges; not re-verified this pass.)*
- The world measures **D4 = 0.4095 m/Myr** — **2.4–24× below the stable-interior band**.
- With the sim's rebound delivering only ~40 % of Airy at high erosion, a D3 in the craton band
  may not be *sustainable*: the world would net-lower rather than exhume.

> **`uplift_scale` is P2's sibling, not its follow-on. The probe run that measures D3 also
> measures D4 and the mean surface; if the mean land surface or land-cell count walks at the
> target multiplier, the pick is not complete without an uplift call — and that is a second
> user-owned, appearance-class number.** *This is the same "run the arithmetic once against the
> outside world" rule one constant over.*

---

## 6. The measurement plan for the integrator (needs the build slot)

Everything here is **read-only with respect to the physics** — `denudation_ledger` is
gate-asserted bit-inert (`the_denudation_ledger_is_inert`). None of it flips a flag.

### 6.1 Probe changes needed FIRST (small, and they are what unblock the derivation)

Two new printed columns in `Denudation` / the ladder row, plus two repairs:

| # | change | why | cost |
|---|---|---|---|
| **M1** | print **`⟨exp(−H/H*)⟩`**, the land-area-weighted mean cover taper, per ladder row | **the single number that closes chain B** and collapses a 4.5× bracket. § 4.3 proves `mean H` cannot substitute | one `map/sum` over land cells |
| **M2** | print the **modulator product** `⟨(biotic × weatherability) × frost⟩` and the **supply/incision split of D3** (`weathering`-front descent vs `k_bedrock`) | resolves the 1.34-vs-6× ambiguity in chain B directly | two counters in `weather` / the incision phase |
| **M3** | fix the **Airy ceiling formula**: `U_tect = D4 − (1 − f)·D3`, then `ceiling = U_tect / f` | § 4.1 — the shipped formula is valid only before the calibration works | one line |
| **M4** | fix the **`BANDS` table**: add the ¹⁰Be **basin** row (median 54, mean 218), correct the outcrop row to **n = 450**, correct Arena Valley to **0.53**, drop/replace the unverified `Ritter 2023` | § 1.3 — a published claim inside a gate-run example | table edit |
| **M5** | add a **relief + land-cell + mean-surface** column to the ladder | journal/0122's ladder dropped them; corrections #61 requires an aggregate to ride *beside* a neighbour-relative measure | already computed in `Denudation`, just unprinted |

### 6.2 The runs

Every run: `seed 1337`, `Extent::Medium`, 200 epochs, `denudation_ledger: true`,
**`creep_substep: true`** (the fixed operator — this is the whole point; the default since
journal/0122, but assert it in the report).

```
# R1 — the baseline nobody has under the fixed operator (~40 s)
cargo run --release -p dc-worldgen --example denudation_probe -- 1

# R2 — the derivation ladder, bracketing chains B and C (~73 min of solve)
cargo run --release -p dc-worldgen --example denudation_probe -- 45 100 150 250 400

# R3 — the pair probe: is a cheaper transport still in band? (one world, ~4 min)
#      needs a two-arg entry point (S, T) or a one-off `three_only` / `creep_only`
#      pair in main(); the probe already has both closures for the 0111 contrast.
#      (S, T) = (150, 50)  vs  R2's (150, 150)

# R4 — the unbounded control, ONE rung, to keep the operator repair visible in
#      the same report rather than quoted across two entries
#      (150x with creep_substep: false)
```

**What each run must report against, so the acceptance is stated before the numbers arrive:**

| criterion | quantity | bar | source of the bar |
|---|---|---|---|
| **primary** | **D3** | **1 – 4 m/Myr**, target 2.63 | § 1.4 / § 4.1 — cratonic bedrock band + the world's own Airy fixed point |
| corroborating | total exhumed over 500 Myr | 500 – 2 000 m at the target | `D3 × 500` |
| **steady state** | **D3 / D4** | → 1.0, and **D4 must rise with D3** | § 4.1; if D4 stalls, § 5.4's uplift call is owed |
| **shape (neighbour-relative)** | `conc(h)` rms, ACF(1) | inside journal/0122's ±0.15 | corrections #61/#62 — an aggregate alone licenses nothing |
| **shape (aggregate)** | relief, mean surface, land cells | **re-derive the bar** — 0114's "±5 %" was falsified as a sole criterion (#61) and measured on the wrong operator | § 3.1 |
| pits | hollows > 10 m | 0 | journal/0122's derived bar (order of magnitude above the ~1 m dimple floor, 4.5× below the 45 m failure scale) |
| reported, not asserted | D1 | **upper bound only** — never the acceptance | corrections #60 |
| reported | gen time, mean H, `⟨taper⟩`, `creep-lim` % | — | the derivation's own inputs |

### 6.3 Explicitly NOT part of this

- **Do not flip `calibrated_rates`.** Appearance-class, user-owned; the probe ladder is the
  evidence for the conversation, not the decision.
- **Do not re-fit anything to internal consistency**, and do not choose a rung because the world
  "looks right" — `measure-against-the-literature`.
- **Do not touch `mfd_chi_lo/hi`** (stubs #26) or `COMPETENCE_SCALE` (corrections #59) in the same
  slice; both are downstream of the multiplier and both are named heirs of it.
- **The remaining `dt` conversions** (`dependency-graph.md` § 4 item 6 — stream transport, bedrock
  weathering, wind and wave still assume `dt = 1.0`) are explicitly to be done **with** P2, not
  before: *"picking one silently re-tunes a constant P2 is about to re-pick anyway."* Note the
  weathering one is a **modelling** call (`1 − exp(−k·dt)`, not `k·dt`) and it lands directly on
  chain B's arithmetic.

---

## 7. What P2 unblocks when it lands

| row | what is gated | citation |
|---|---|---|
| **P1 → P2 → the flag flip** | *"Nothing downstream of erosion can be judged until the constant is re-picked, and it must be picked against the published literature, never against a look."* | `dependency-graph.md` § 3 |
| **E5 member #1 (the fluvial member) — its WALK ACCEPTANCE** | *"Design now, BUILD AFTER P2: the operator can only express recorded magnitudes (Law 1), and this world exports **0.02 %** of denudation by rivers — **a walk on the member is meaningless until the P2 literature re-pick lands.**"* The design pass proceeds and *"should tell P2 which quantities matter."* | ROADMAP § close block ruling 3 (2026-08-01); `dependency-graph.md` E5 row (*"#1's live blockers are now the RECORD TERMS … and P2 (build-after-P2 ruled same day)"*) |
| **P10 — the grain-size continuum** | **SEQUENCED 2026-08-01**; design pass startable today, **calibration gated on P2** — *"pre-P2, no cell can carry sand: competence ceiling **0.2832** against coarse clastic's **0.840**"*, and max transport capacity **6.74 × 10⁻⁴**, three times below the `energy_band` boundary `COMPETENCE_SCALE` is anchored on. **P2 is where Shields / Sternberg / gravel-sand-transition bands become real calibration targets** | `dependency-graph.md` P10 row; ROADMAP § *THE GRAIN-SIZE CONTINUUM*; ROADMAP:1564-1569 |
| **facies-gradient judgments** | the gradient reaches **0.000006 %** of the archive and the headwater→trunk grain ratio is **0.921 before and after**, unchanged to four decimals — because fluvial transport is **0.109 %** of routing and creep does **918×** what rivers do | ROADMAP:1556-1570; corrections #55; journal/0110 |
| **the ROADMAP's own demotion clause** | *"It DEMOTES the rest of the material-behavior arc's urgency… **a landscape moving a thousandth of the right amount of sediment will still not show a facies gradient.** The calibration should land first, or at least alongside."* | ROADMAP § *CALIBRATE THE DEEP-TIME CLOCK* |
| **stubs #26** (`mfd_chi_lo/hi` fitted to one world) | heir (a) is *"the joint supply+transport calibration — once the engine's rates are anchored, `χ` acquires a physical scale and the threshold can be **derived** from a channel-initiation stream power rather than fitted"* | `stubs.md` #26 |
| **corrections #59** (`COMPETENCE_SCALE`'s anchor sits outside the range the world occupies) | same mechanism | corrections #59; ROADMAP:1566-1569 |
| **`earth-processes.md` § 3e's oldest owed item** | *"calibrate iteration↔Myr against a real orogen"* — the **named heir of a measured 10³× error** | `earth-processes.md` § 3e-2 decision 5 FLAG; stubs #24 |
| **every golden** | *"This moves EVERY golden — same event class as the erodibility / biotic / tectonic / full-agent flips. Announced, never silent."* And per § Conventions, a hash move produced by **ratified semantics** re-captures the goldens with the why recorded — it owes no byte-identicality | ROADMAP; CLAUDE.md § *the testing world is a scratch pad* |

---

## 8. Everything unverifiable — "I cannot derive this without a measurement"

Listed first-class, per the RETURN spec.

1. **D3 (or any denudation number) under the fixed operator, at any multiplier including 1×.**
   Nothing recorded. journal/0122's ladder has no denudation column. **This is the gap.**
2. **`⟨exp(−H/H*)⟩` at 1×.** Never printed; § 4.3 falsifies the mean-field substitute against the
   corpus's own data. **It alone sets chain B's 4.5× bracket width.**
3. **The modulator product and the supply/incision split of D3.** The `1.34` factor in § 4.2 is a
   composite nobody has decomposed.
4. **The exponent of D3 vs M under the fixed operator.** Under the capped operator it fits
   `D3 ∝ M^0.7` (log-log over 0114's six rungs; slopes 0.62 / 0.69 / 0.98 / 0.86 / 0.55). Under
   the fixed operator, cover *thins* with M, so the taper opens and the exponent should exceed 1
   — **the direction is derivable, the magnitude is not.**
5. **Relief, mean surface and land-cell count at any multiplier under the fixed operator.**
   journal/0122 dropped all three from its ladder. journal/0114's binding criterion ("relief
   within 5 %") is therefore both falsified as a sole criterion (#61) *and* unmeasured on the
   current operator.
6. **Whether the world can SUSTAIN a craton-band D3.** § 4.1's isostasy table says the sim's
   rebound falls to ~40 % of Airy at high erosion. Whether that makes the target unreachable
   without moving `uplift_scale` is one probe run away and cannot be settled on paper.
7. **Whether `(S, T) = (M, M/3)` still lands in band.** § 4.5 argues transport is non-binding at
   the target M, so it *should* — but chain C is an order-of-magnitude argument over an advective
   process modelled as a random walk. **Measure it; do not assume it.**
8. **The three literature citations in § 1.1 I could not confirm** (Ritter 2023, Veselovskiy 2019,
   Morón 2020), and the four I did not re-verify this pass (Matmon 2003, Dadson 2003, Herman 2013,
   Koppes & Montgomery 2009). The load-bearing ones — Portenga & Bierman 2011, Morgan et al. 2010,
   Bierman & Caffee 2001/2002, Wilkinson & McElroy 2007 — **were** re-verified.
9. **Which craton target governs** (§ 1.2, 1–10 vs 10–20 m/Myr). Not an engineering question; a
   user call about what the 500 Myr run is supposed to be a record *of*.

---

## Sources (literature verified this pass)

- Portenga & Bierman 2011, *Understanding Earth's eroding surface with ¹⁰Be*, GSA Today 21(8), 4–10 — https://rock.geosociety.org/gsatoday/archive/21/8/pdf/i1052-5173-21-8-4.pdf
- Morgan et al. 2010, *Quantifying regolith erosion rates with cosmogenic nuclides ¹⁰Be and ²⁶Al in the McMurdo Dry Valleys, Antarctica*, JGR-Earth Surface — https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2009JF001443
- Bierman & Caffee 2001/2002 (Namib; Australian landforms), as framed in the current cratonic-denudation literature — https://serc.carleton.edu/vignettes/collection/31859.html · https://onlinelibrary.wiley.com/doi/10.1002/esp.70262
- Wilkinson & McElroy 2007, *The impact of humans on continental erosion and sedimentation*, GSA Bull. 119, 140–156 — http://geomorphology.sese.asu.edu/Papers/Wilkinson_2007_GSAB.pdf
- Kober et al. 2007 / Placzek et al. 2010 / Starke et al. 2017 (Atacama denudation rates) — https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2016JF004153 · https://www.sciencedirect.com/science/article/abs/pii/S0012821X10001779
