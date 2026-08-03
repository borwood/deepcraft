# P2 — the erosion-calibration re-pick: MEASUREMENT RUNS

*2026-08-02 · the integrator half of P2 (needs-the-build-slot) · executes
[`2026-08-01-p2-calibration-derivation.md`](2026-08-01-p2-calibration-derivation.md) § 6*

> *(Mutable header, per the immutable-body convention — CLAUDE.md read-first item 5.
> Nothing above this line is testimony; everything below it is dated measurement.)*

**What this is.** The four runs § 6.2 of the derivation ordered, taken with the five probe
changes § 6.1 ordered (M1–M5), on one commit, one machine, one seed. **It does not pick
the number** — `calibrated_rates` stays `false`, nothing was re-fitted, `mfd_chi_lo/hi`
and `COMPETENCE_SCALE` were not touched, and the remaining `dt` conversions were not done
(§ 6.3, all four hard limits held).

**Provenance.**

- instrument commit: `82eb63a` (M1–M5, on the P2 worktree branch off `665167e`); the run
  was one invocation — `denudation_probe -- 1 45 100 150 250 400 150:50 u:150` — exit 0,
  full log Tee'd and read from the file (`p2-ladder-run.log`, 183 lines, reproducible
  from the seed + command)
- seed **1337**, `Extent::Medium` (545² = 297,025 cells at 460 m), 200 epochs,
  `denudation_ledger: true` in every run
- **`creep_substep: true` (the fixed operator) in every run except R4**, asserted per row
  by the probe's `sub` column (`!` marks the unbounded operator; only R4 carries it)
- gates on the instrument change, verified by name pre-run: fmt clean · clippy
  `-D warnings` clean (`Checking dc-worldgen` at the worktree path) · the probe's 5 gate
  tests green (`the_denudation_ledger_is_inert` proving bit-inertness with the new
  counters, `the_weather_coupling_counters_are_wired_and_bounded`,
  `the_calibration_actually_raises_the_measured_denudation`,
  `the_sink_itemisation_sums_to_the_sink_total`,
  `no_export_term_exceeds_the_flux_it_is_a_share_of`) — suite 5/5 in **77.67 s**;
  **no golden moved**

---

## 1. The instrument changes (M1–M5), what they added

| # | change | where |
|---|---|---|
| M1 | `⟨exp(−H/H*)⟩` — run-integrated, area-weighted, accumulated in the weathering phase over the exact subaerial population the kernel gates on, plus the end-state taper over final land | `ledger.rs` (`weather_taper_sum`/`weather_cell_epochs`), `weathering.rs`, probe mechanism table |
| M2 | `⟨(biotic × weatherability) × frost⟩` run-mean + the supply/incision split of D3 (`weathered_m` vs `incised_m`, both pre-existing counters, now printed as m/Myr in D3's frame) | `ledger.rs` (`weather_mod_sum`), probe |
| M3 | Airy ceiling formula fixed: `U_tect = D4 − (1−f)·D3`, `ceiling = U_tect/f` (the shipped `D4/f` was valid only while erosion is negligible) + a per-rung Airy-sustainability column (measured D4 vs `U_tect + (1−f)·D3`) | probe |
| M4 | `BANDS` table: ¹⁰Be **basin** row added (median 54 / mean 218); outcrop row corrected to n = 450 (1599 is the whole compilation); Arena Valley 0.53 (0.19 was the range minimum wearing a site's name); Ritter 2023 dropped for Dunai 2005 + Placzek 2010; craton row restated as the 1–4 D3 acceptance band; instrument↔band matching note printed (D1↔basin, D3↔outcrop) | probe |
| M5 | relief / land-cell / mean-surface columns (already present in this probe's ladder — the audit's M5 was written against journal/0122's ladder, which had dropped them) **plus** the shape instruments the § 6.2 acceptance table needs: `conc(h)` rms, ACF(1) x/y, hollows >1 m / >10 m and deepest on the router's own fill, per rung, population-matched to `creep_operator_probe` | probe |

Also added: the pair entry point `S:T` (R3) and the unbounded control `u:M` (R4);
explicit-rung invocations skip the journal/0114 calibration-comparison and
single-lever-contrast arms (they answer 0114's question, not P2's). New gate test
`the_weather_coupling_counters_are_wired_and_bounded`; `the_denudation_ledger_is_inert`
extended to assert the new counters are exactly zero with the flag off.

Gate wall-clock: the two new/extended assertions ride the suite's existing world builds;
the whole 5-test probe suite measures **77.67 s** on this commit (not separately
attributable per test; the >60 s member is the pre-existing calibration test).

## 2. The runs — the three tables, verbatim

### The derivation ladder (R1 = rung 1 · R2 = 45–400 · R3 = 150:50 · R4 = 150 UNB)

```
x            D1(ub)        D3   D3/D4  D1/D3  meansurf   relief   mean H    land  creep-lim  band(D3)
1 SHIPPED      0.0109    0.0101   0.02   1.08    517.4   1286.5    3.91   44253   79.1 %  below
45             0.3387    0.3234   0.45   1.05    510.6   1271.8    1.39   43801   77.6 %  below
100            0.7301    0.7076   0.66   1.03    503.3   1255.2    1.63   43184   74.6 %  below
150            1.0895    1.0589   0.75   1.03    496.6   1240.2    1.52   42629   73.3 %  IN 1-4
250            1.7991    1.7466   0.84   1.03    483.1   1211.1    0.82   41554   71.8 %  IN 1-4
400            2.8870    2.7859   0.91   1.04    464.7   1168.3    0.45   39869   70.7 %  IN 1-4
150:50         0.6439    0.6390   0.63   1.01    506.6   1255.6    7.18   43521   58.1 %  below
150 UNB        2.9398    0.4140   0.84   7.10    546.5   1655.6  191.26   42945   93.7 %  below
```

### Mechanism & shape (M1/M2's new columns; the § 6.2 shape criteria)

```
x           sub  taper.run  taper.end   mod   supply   incis  conc(h)   ACF x   ACF y  h>1     >10  deepest       D4   gen s
1 SHIPPED     2    0.5862    0.3864  2.841   0.0104  0.0000     2.78  -0.117  -0.118      0      0     0.0   0.4105    41.2
45           92    0.8043    0.8428  1.730   0.3345  0.0007     2.64   0.022   0.881  15616     12    15.9   0.7138   288.3
100         224    0.8038    0.8531  1.743   0.7318  0.0017     2.62  -0.005   0.620  42361     13    14.9   1.0802   588.0
150         336    0.8060    0.8602  1.741   1.0958  0.0026     2.69  -0.062   0.412  42230     33    15.5   1.4156   882.8
250         553    0.8196    0.9100  1.682   1.8138  0.0045     3.20  -0.140   0.065  41563    406    17.2   2.0728  1478.5
400         896    0.8312    0.9556  1.623   2.9175  0.0076     3.92  -0.108  -0.197  39927   8878    27.3   3.0678  2306.5
150:50      112    0.2389    0.1924  3.141   0.6632  0.0002     3.20   0.027   0.394  32953     25    16.5   1.0104   310.9
150 UNB      1!    0.3369    0.3550  2.468   0.4482  0.0006   231.56  -0.804  -0.827   4371   3719   320.8   0.4904    37.8
```

### Airy sustainability (M3's per-rung column)

```
x            D4 meas   D4 Airy    ratio
1 SHIPPED    0.4105    0.4105     1.00
45           0.7138    0.6763     1.06
100          1.0802    1.0024     1.08
150          1.4156    1.3004     1.09
250          2.0728    1.8839     1.10
400          3.0678    2.7657     1.11
150:50       1.0104    0.9441     1.07
150 UNB      0.4904    0.7532     0.65
```

And the shipped-row ceiling, M3's fixed formula on the world's own measured uplift:
**`U_tect = D4 − (1−f)·D3 = 0.4020 m/Myr` → steady-state ceiling `U_tect/f = 2.653
m/Myr`** — *"THAT NUMBER WAS NOT CHOSEN. It falls out of two densities and a measured
uplift, and it lands inside the published cratonic bedrock band (1–4 m/Myr) on its own."*
(The pre-fix formula printed 2.709; the two diverge exactly as the calibration works.)

## 3. Against § 6.2's acceptance table

| criterion | bar | verdict |
|---|---|---|
| **primary: D3 in 1–4 m/Myr, target 2.63** | § 1.4/§ 4.1 | **Reachable and bracketed by measurement: 150/250/400 are all IN band; D3 is near-linear in M (≈0.0070·M), putting the 2.63 target at M ≈ 375–380** — see § 4's contradiction |
| corroborating: 500 Myr exhumation 500–2000 m | D3 × 500 | at M≈375: ≈1,315 m ✓ (in band) |
| **steady state: D3/D4 → 1.0, D4 rises with D3** | § 4.1 | ✓ **healthy** — D3/D4 climbs 0.02 → 0.91 across the ladder and D4 rises 0.41 → 3.07; the § 5.4 uplift-stall call is NOT triggered (Airy ratio 1.06–1.11: the sim returns slightly *more* than Airy, never less; only the broken-operator control fails it at 0.65) |
| shape (neighbour-relative): conc(h), ACF(1) vs journal/0122 (white noise −0.167) | ±0.15 | shipped row −0.117/−0.118 ✓; the ladder drifts with M (ACF-y 0.881 at 45 decaying to −0.197 at 400; conc(h) 2.6→3.9) — **shape at the working rungs is not the shipped world's shape**, expected but must be walked, not asserted |
| shape (aggregate): relief, mean surface, land | re-derive, never sole | reported per rung (relief 1286→1168, land 44,253→39,869 across the ladder) |
| **pits: hollows >10 m = 0** | journal/0122's bar | **✗ FAILS at every in-band rung** — 33 @150, 406 @250, **8,878 @400 (deepest 27.3 m)**; shipped row: 0. See § 4 |
| D1 | upper bound only | D1/D3 ≈ 1.03–1.05 everywhere under the fixed operator (balanced); 7.10 on the unbounded control — the operator repair is visible in-report as ordered |
| reported: gen time, mean H, ⟨taper⟩, creep-lim % | — | all per-rung above; **gen time is the second § 4 finding** |

## 4. What contradicts the derivation, and what confirms it

**Confirmed, strongly:**

1. **The band target is right for this world, now by two independent routes** — the
   twice-derived 2.63 m/Myr (§ 4.1) and the fixed Airy decomposition's 2.653 m/Myr from
   measured densities + uplift alone. Five percent apart, neither chosen.
2. **§ 4.3's Jensen warning is measured fact**: run-mean ⟨taper⟩ = **0.80–0.83** at every
   working rung (0.59 shipped), where `exp(−⟨H⟩/H*)` from the same rows would give wildly
   different values (mean H spans 0.45–3.91 m). Chain B's missing term now has a number.
3. **The supply/incision ambiguity (M2's question) resolves completely: it is supply.**
   Incision contributes ≤0.3 % of D3 at every rung (e.g. 1.0958 vs 0.0026 at 150). The
   weathering front is the governor; ⟨mod⟩ ≈ 1.7 at working rungs.
4. **R4**: the unbounded control shows the journal/0122 repair in one row — D1/D3 = 7.10,
   mean H = 191 m, conc(h) = 232, ACF ≈ −0.8. Kept in-report as ordered.

**Contradicted / new, all owed to the ratification conversation:**

5. **⚠ The bracket M ∈ [60, 240] is LOW by ~1.6×.** Measured D3(M) is near-linear at
   ≈0.0070·M; the 2.63 target lands at **M ≈ 375–380**, outside the derivation's § 4.6
   bracket. (Its chain-B arithmetic can now be re-run with the measured ⟨taper⟩/⟨mod⟩ —
   that re-derivation belongs to the re-pick conversation, not this record.)
6. **⚠ The pits bar and the band CONFLICT under the current transport/fill:** every rung
   that reaches the band violates hollows->10 m = 0, and it worsens super-linearly (33 →
   406 → 8,878 across 150→400). Either the router/fill needs work before the flip, or the
   bar needs a ruled revision — user call, with journal/0122's derivation of the bar in
   hand.
7. **⚠ Gen time at the in-band rungs:** 883 s @150, 1479 s @250, **2,306 s @400** for the
   200-epoch Medium solve (41 s shipped). Doctrine says gen time is not a constraint and
   ready-made worlds are the sanctioned answer — but the `pregen_time_vs_extent` budget
   (1,200 s, renegotiated 2026-08-02 *"as long as it doesn't take 20 min"*) would be
   exceeded ~2× at M≈375–400. The § 5 register ↔ Myr question (epoch count vs per-epoch
   rate) is the named lever; it was deliberately not touched here.
8. **§ 4.5's "the pair space collapses to one axis" is REFUTED at the measured point:**
   (S=150, T=50) gives D3 = 0.639 vs (150,150)'s 1.059 — cutting transport to a third
   cuts D3 by 40 % and flips the world visibly transport-limited (mean H 7.18 m,
   creep-lim 58 %). Transport strength is a real second axis at this end of the space; a
   cheaper-transport calibration does not stay in band.

**Net: the evidence for the flip conversation is complete; the flip itself is blocked on
three user calls** — the M re-pick against the measured curve (finding 5), the pits-bar
conflict (finding 6), and the gen-time/register trade (finding 7).
