# Far-frontier tour map — 2026-07-29 evening walk (member-#0 far slice)

**Instrument:** `examples/palette_quant_tour.rs` (gated, `test = true`), run on the
production world (seed 1337, `Extent::Medium`) at `9a0942e`. Rescued from the session
scratchpad at wrap (wrap § 1). **Dated artifact — the stations below were scored by
record shares WITHOUT a land filter; station 1's failure mode is the finding.**

## Stations (search box x [92k,118k] × z [−12k,22k] m; WIN=3 cells ≈ 1,380 m)

| # | centre (world m) | elev | classes (3×3 window) | score | walked |
|---|---|---|---|---|---|
| ARGMAX 1 | (116382, −7346) | **−55.6 m** | o,S,P / o,m,m / P,S,m — 4 distinct, bal 0.99 | 5.281 | **SCRAPPED** |
| ALT 2 | (106263, −5506) | +19.2 m | S,P,m / o,S,m / o,S,m — 4 distinct, bal 0.95 | 5.218 | **✅ VERDICT STATION** |
| ALT 3 | (111783, −5506) | +12.0 m | m,o,m / o,P,o / S,S,S | 5.218 | unwalked |
| ALT 4 | (115002, −4586) | +40.3 m | S,o,m / o,S,m / o,S,P | 5.218 | unwalked |

Top-down framing: camera at ground+1,666 m, pitch −1.55, yaw 0 (π/4 vertical FOV frames
the 3×3 window). Close-in variant: ground+90 m. Poses in the ROADMAP § Observed field
report (walk 2026-07-29 evening).

## The instrument finding — a probe blind spot, caught by measurement mid-walk

**Station 1 sits below sea level (−55.6 m) and its near surface is the UNRECORDED
ocean-floor path** — `world_get_contents` at the surface voxel answered `dc:stone`,
`has_contents: false`. The deep record underneath genuinely holds the scored 4-class mix;
the expression discards it on the ocean-floor branch, so the walk frame reads bare grey.
**The score ranked record shares; the walk views expression.** Any future run of this
tour for walk purposes must filter (or at least flag) `elev < sea level`. The
`palette_quant_tour` NOTE about a strong window riding the search-box edge is also still
open — the box was not widened this session.

## Walk outcome (full record: ROADMAP § Observed, walk 2026-07-29 evening)

Station 2: **cell lines gone, confirmed by eye; the cell-wide blend's semantics rejected**
(*"the whole cake is swirled now"*) — rides as interim, heir = a far register derived from
refinement-operator budgets (user sketch, recorded verbatim in the field report).
Stations 3/4 unwalked — the verdict question was answered at station 2; they remain valid
future stations for the same signature.
