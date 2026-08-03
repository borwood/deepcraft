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

- commit: (filled at commit time — the M1–M5 instrument changes ride this same commit)
- baseline the diff was taken against: `665167e` (worktree of main)
- seed **1337**, `Extent::Medium` (545² = 297,025 cells at 460 m), 200 epochs,
  `denudation_ledger: true` in every run
- **`creep_substep: true` (the fixed operator) in every run except R4**, asserted per row
  by the probe's new `sub` column (an `!` suffix marks the unbounded operator; only R4
  carries it)
- instrument: `crates/dc-worldgen/examples/denudation_probe.rs` after M1–M5; the
  `denudation_ledger` is gate-asserted bit-inert (`the_denudation_ledger_is_inert`,
  re-run green on this commit) — **no golden moved**; full `dc-worldgen` suite green
  (counts below)

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

Gate wall-clock added by the new test: (filled below).

## 2. The runs

*(numbers below are pasted from the Tee'd logs, named per run; the logs are the raw
record, this table is the reading)*

### R1 — the baseline under the fixed operator (`-- 1`)

(to be filled)

### R2 — the derivation ladder (`-- 45 100 150 250 400`)

(to be filled)

### R3 — the pair probe (`-- 150:50`)

(to be filled)

### R4 — the unbounded control (`-- u:150`)

(to be filled)

## 3. Against § 6.2's acceptance table

(to be filled)

## 4. What contradicts the derivation, and what confirms it

(to be filled)
