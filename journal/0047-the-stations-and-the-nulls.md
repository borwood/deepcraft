# 0047 — The stations, and the nulls

*2026-07-21. The user ratified turning the full erosion-agent roster on in
production — "we turn on agents next" — the same event class as the U8 tectonics
flip (0044) and the erodibility flip (0030). But this one carries an unusual
rider: the agent **magnitudes** (wind rates, dune/loess split, frost gain, wave
strength) are appearance-class numbers, and the user will judge them **live**,
standing in the world, station by station, in a guided co-walk. So this entry is
the flip, the re-baseline ledger, the measured cost, and the novel deliverable —
a **tour map**: the coordinates of the strongest exemplar of each of the five
agent signatures in the world the client boots, so the integrator can teleport a
player straight down the list. The flip is not deferred; the appearance verdict
is. That deferral is the first of its kind on this project.*

## The flip

One line in `production_config` (`deeptime/field.rs`), the same class of change
as the biotic / erodibility / tectonic flips stacked right above it:

```rust
full_agents: true,
```

Off, deserts had an arid *tag* but no arid *landform*, cold high ground weathered
at the same rate as a warm lowland, and coastlines did nothing. On (journal/0034),
the roster runs: eolian deflation redistributes loose cover into dune fields and
downwind loess; the temperature-gated frost multiplier strips extra regolith in
the freeze–thaw band about 0 °C; littoral wave attack cuts coasts toward the
current sea stand. As with every prior world-shape flip, the honest cost is stated
plainly: **every world created after this flip is unreproducible under any earlier
build**, and the byte-identical `DeepOverrides { full_agents: Some(false) }`
channel still reaches the pre-0034 path — the flip moves the *default*, not the
*mechanism* of reproducibility.

What is deliberately **not** touched: none of the seven magnitude knobs
(`eolian_deflation` 0.02, `eolian_arid_precip` 0.32, `eolian_deposit_frac` 0.25,
`frost_weathering_gain` 1.5, `frost_band_width_c` 12.0, `wave_erosion` 0.05,
`wave_band_m` 30.0). They ride at the 0034 defaults, for the user's eye. The
DECIDED addendum lives in `earth-processes.md § 4`; the `production_config`
comment says the tour, not the flip, ratifies the numbers.

## The re-baseline ledger

Two tests froze production content and had to move. Neither was a bug; each is the
flip doing its job, and in each case the test's subject survived intact.

**1. `deep_config_plumbing::full_agents_override_toggles_the_roster`** (was
`…_changes_the_surface`). This is the exact inversion tectonics hit in 0044. The
old test proved the override bites by building production (roster off) and an
override field (`Some(true)`, roster on) and asserting the surfaces differ. With
production now roster-*on*, both sides are on and the surfaces are identical — the
`assert_ne!` fails. The subject ("the `full_agents` override reaches the run") is
unchanged; I reworked the falsifier to prove it through the seam that now turns the
roster *off*: production carries the agents, and `Some(false)` reaches the pre-0034
surface. Same fix, same shape, as `tectonic_history_override_toggles_…`.

**2. `organic::the_measured_coal_seam_is_coal_a_player_can_dig`.** The heavyweight
proof that a coal seam is biofacies-tagged, survives collapse as the COAL class,
and ends up as diggable `Block::Coal`. It picked the **thickest record seam that
also renders ≥15 collapse-voxels of coal**. After the flip that pick found *no
cell* and panicked. I did not take the null on faith; I measured the whole field.
Post-flip the world grows **7647** coal seams over 3 m of record thickness, and the
strongest *diggable* one renders **19 collapse-voxels** (record 11.9 m, voxel
`(-7650, -117528)`) — equal to the pre-flip "~19 voxels". The record→blocks
mechanism is entirely healthy. What broke was the frozen coupling of two axes: the
flip's wind/frost/wave redistribution moved the *record-thickest* seams (over 15 m)
to spots where they bury below the collapse column, while a huge population of
8–12 m record seams surface as 15–19 diggable voxels. This is the 0044 collapse-
column situation and the 0030 coal-test situation exactly — a frozen single-point
pick landing on a degenerate spot while the mechanism is intact everywhere — and I
fixed it the same way: **select the strongest seam that actually surfaces as
diggable coal** (folding in section 3's low-energy-coal requirement so the picked
cell satisfies every sub-claim), not loosen a floor. The bar stays at 15 voxels; it
is now met by selection, not by hoping the thickest-record cell also digs.

Everything else in the workspace passed unchanged — the whole `full_agents.rs`
suite (it builds explicit configs, so the production flip does not reach it),
every byte-identity and determinism proof, and the tectonic/biotic/erodibility
fingerprints. Full gates green: `fmt --check`, `clippy -D warnings`,
`test --workspace`.

## The measured cost

At the production entry point (`build_field`), roster off vs on, seed-matched
same-build A/B via `DeepOverrides { full_agents: Some(false) }`, at Medium
(545², 460 m, seed 1337):

| | build | resident (kept `DeepField`) |
|---|---|---|
| roster OFF | 16.0 s | 52.78 MB |
| roster ON | 18.5 s | 145.54 MB |
| Δ | ~+2.0–2.5 s | **+92.76 MB** |

The ritual grows about two seconds — inside its class. The **+92.76 MB is the
surprise**, and it is real (a deterministic count over the field's own vectors,
not RSS): the wind agent lays eolian units across tens of thousands of cells, so
the strata record — the one variable-length plane the world keeps — roughly
**triples**. 0034 predicted the record grows; at production extent it grows a lot.
Worth flagging for the integrating session: the roster's cost is not in the clock,
it is in the kept record.

## The tour map

The deliverable. `examples/tour_map.rs` builds the client's world (seed 1337,
Medium, full production config) and runs five isolated deep-time passes on the one
pregen — production (all agents, the surface a walker stands on and where the
eolian facies are recorded), all-off (baseline), and one per agent with the other
two rate knobs zeroed — exactly the isolation the `full_agents` test suite uses.
Deflation is `ΔH` (off→wind), frost stripping `ΔR` (off→frost), wave retreat
`Δsurf` (off→wave); dune/loess are read straight from the production record.

**Aggregate signatures over the whole world** (the 0034-comparable numbers, now at
production scale): dune **5617.6 m** + loess **3110.5 m** of eolian deposit; net
deflation **2909.0 m** over 44 232 arid cells; **42 992.5 m** extra bedrock
stripped over 41 239 periglacial-band cells; **41.3 m** total littoral lowering
over 1565 coast cells.

**The edge-pileup finding.** The single strongest loess / deflation / frost cells
all landed on grid column 544 — the **downwind land edge**. 0034's 1-D wind march
settles "whatever is still aloft at the downwind land edge" there, so the border
ring is a mass sink, not a landform. The tour map therefore reports the strongest
*interior* exemplar (border ring excluded); a walker sent to the raw maximum would
stand at the wilds boundary on a march artifact.

**The five stations** (world metres; teleport a player straight down the list):

| # | signature | world (x, z) m | voxel | surf | signal | knob |
|---|---|---|---|---|---|---|
| 1 | **Dune field** | (107183, 9672) | (119092, 10747) | 205 m | 2.09 m dune sand | `eolian_deflation` / `eolian_deposit_frac` |
| 2 | **Loess margin** | (82346, 24391) | (91495, 27101) | 14 m | 2.47 m loess silt; desert 0 m away (walkable) | `eolian_deposit_frac` |
| 3 | **Deflation basin** | (5993, 14732) | (6659, 16369) | 249 m | 13.06 m H blown out | `eolian_deflation` |
| 4 | **Periglacial band** | (-4586, -3206) | (-5096, -3562) | **998 m** | 11.81 m bedrock stripped (high-relief summit) | `frost_weathering_gain` / `frost_band_width_c` |
| 5 | **Wave-cut coast** | (95224, 22091) | (105805, 24546) | 3 m | **0.68 m** — a null | `wave_erosion` / `wave_band_m` |

What a walker should see, station by station, and the honest magnitude verdict:

- **Dune (2.09 m) and Loess (2.47 m)** are *modest* at interior sites. The desert
  interior deflates and the humid margin traps, and the transition at station 2 is
  crossable on foot — but the deposits are metres, not the dramatic dune-field
  the aggregate 5.6 km total suggests (that total is spread over the whole world,
  and much of the loudest signal is the edge artifact). The user should expect a
  low sand sheet, and decide at station 2 whether `eolian_deposit_frac` wants
  raising.
- **Deflation (13 m)** and **frost (12 m at a ~1 km summit, up to 23 m raw)** are
  the two **legible** stations. Station 4 is the best of the tour: a cold high
  summit in the freeze–thaw band shedding a dozen metres of extra regolith — the
  periglacial signature the design promised, on ground with the relief to show it.
- **Wave (0.68 m) is effectively a NULL station.** The strongest littoral cut in
  the whole world is sub-metre over 200 epochs; near the 0046 scarp (ground the
  user knows) it is 0.20 m. At the current `wave_erosion` 0.05 / `wave_band_m` 30,
  the coast does essentially nothing a walker can read. This is not a bug to fix
  silently — it is precisely the magnitude verdict the live tour exists to make. If
  the user wants visible sea cliffs, the wave knobs need to go up by more than an
  order of magnitude, or the mechanism is honest that a wave-cut *region* at 460 m
  over this run simply is a metre, as 0034 measured on the Small world.

## The protocol note

This is the first flip on the project whose appearance calls are deliberately
deferred to a **live co-walk** rather than gated on a spike screenshot or shipped
blind on "make progress" (0044). The discipline that makes it safe is the same one
that makes the reversible flips safe: the byte-identical override channel means the
production default can carry unratified magnitudes without the reproducibility
guarantee moving, and the tour map turns "judge it live" from a vague intention
into a teleport list with numbers attached. The user stands at five coordinates;
three of them will show something, one is a genuine question (the loess margin),
and one — the wave coast — the map says plainly *may show nothing*. Knowing which
before you stand there is the whole point.

> blogworthy: **a null station is a verdict.** The instinct on a tour map is to
> lead the user only to the loud exhibits. But reporting that the wave coast cuts
> 0.68 m — that this station shows *nothing* at the shipped magnitude — is more
> useful than any dune could be: it converts an appearance decision from "walk
> around and hope you notice" into "here is the number; the sea does nothing; do
> you want it to?" The honest negative is the deliverable.

## Files

`crates/dc-worldgen/src/deeptime/field.rs` (the flip + the override doc),
`docs/design/earth-processes.md` (§ 4 DECIDED addendum),
`crates/dc-worldgen/examples/tour_map.rs` (the new deliverable), the two
re-baselined tests (`deep_config_plumbing.rs`, `organic.rs`), and this entry.
