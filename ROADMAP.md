# Roadmap

The living sequence — the only non-append-only document besides code. Update
in the same commit as any journal entry: finished work moves to **Shipped**;
walk/field findings land in **Observed** first (the walk reports, the
diagnosis measures); only diagnosed work gets **Sequenced**.

## Shipped

- 2026-07-25 — **The measurement instruments are in the gate, and the front's mass claim is
  settled** (journal/0103; background agent, worktree; corrections #50). Two halves of one thing:
  a gate that could not see its instruments fail, and a mass number that was wrong because nobody
  ran the instrument that produced it.
  - **`cargo test` BUILDS examples and never RUNS them**, so every probe assertion in the repo was
    unreachable — including a literal `assert_eq!` sitting in
    `dc-client/examples/identify_census.rs`'s `main`, which was journal/0101's whole acceptance.
    Fixed with **Cargo's `[[example]] test = true`**, which builds the example *twice*: normally,
    so `cargo run --example` still prints the full report, and with the libtest harness, so its
    `#[test]`s run in the gate. **One file, one set of measurement functions, two consumers** — no
    library pollution, no duplicated copy to drift (A-1).
  - **Now gated, 16 tests over 7 probes, green by name:** `weathering_profile_probe` (3 —
    `the_fill_geometry_hands_the_front_the_metres_the_record_owes`,
    `the_eighth_draw_does_not_delete_the_thin_front`,
    `the_front_is_graded_over_many_voxels_and_never_pure_product`),
    `contents_air_over_solid_probe` (2), `flux_record_probe` (2), `head_field_probe` (2),
    `s18_weathering_tour` (1), `identify_census` (2), `palette_quant_tour` (4, pure-function only).
  - **The advisory this replaces was falsified in ONE DAY.** After the first `flow_cost_probe`
    break, CLAUDE.md gained *"re-run the probes by hand after any merge that changes what they
    measure."* The next day the same probe broke the same way (a missing `head` row after FLOW (a)):
    **the assertion caught it; the process did not.** Recorded in CLAUDE.md § Gates — *do not
    answer "the gate cannot see X" with a rule asking people to remember X.*
  - **Assert invariants, never snapshots.** Both `flow_cost_probe` failures were a *missing row in
    an itemisation* — wrong at every world size, which is why `Extent::Small` catches this whole
    class. And no converted test pins a MiB figure: ledger residency moved twice in one afternoon
    (per-cell struct 13.60 MiB → 0), and a test pinned to yesterday's number would fail **because a
    colleague improved memory**.
  - **Added gate wall-clock: 35.0 s, measured** — dc-worldgen's six probes **25.1 s** (contents
    2.71 · flux 5.34 · head 5.47 · palette 0.00 · s18 6.19 · weathering 3.64) and dc-client's
    `identify_census` **9.9 s**. **Well under the ~2 min ceiling**, against a workspace gate where
    dc-worldgen's suite alone is ~870 s. Every world-building test runs at
    **`Extent::Small`** and states in its
    doc comment **why the invariant is scale-free** (a per-voxel predicate; a per-column
    arithmetic; a topological property of the primitive). Production magnitudes stay in the
    examples at `Extent::Medium`. Each binary builds its world **once** behind a `OnceLock` shared
    by its tests.
  - **Deliberately NOT gated:** `palette_quant_tour`'s world-scale station search — a station
    finder's output is a *recommendation judged by the eye*, not a claim. Its **ranking function**
    is gated instead (four fixture tests, no world built, ~0 s), because a mis-aimed reference is
    what corrections #48 cost us.
  - **A stale caption caught in the wild, and it is the same blindness one layer up:**
    `flux_record_probe` printed *"vertical … honestly EMPTY (heirs: the head field …)"* beside a
    **non-zero** count for a day after that heir landed (journal/0098). No test asserts on a
    `println!` and no gate runs an example. Rule added to CLAUDE.md § Gates: **when a slice fills a
    hole a probe narrates, the caption is part of the diff.**
  - **The mass verdict: NOISE** — see Observed → DIAGNOSED, and corrections #50. The
    voxel-tier expression is unbiased (stage-1 −0.02 %, stage-2 −0.60 % over 247 columns); the
    reported `+3.7 %` was a material census crediting an overlying mudstone bed to the front.
    **Nothing was fixed and nothing needs to be** — but flow.md § 3's mass budget inherits a
    requirement: *audit against the fill plan, never against a census of finished contents.*

- 2026-07-25 — **A weathering front is a PROFILE, not a slab** (journal/0099; background agent,
  worktree; **collapse-tier only — `deeptime/` untouched, stub #16 NOT retired**). Closes the
  walk finding of journal/0097. The fold no longer emplaces the scalar
  `weathering_product_m` as **one stratum of one class**; it grades the **same mass budget**
  down a front. At the production argmax cell (84185 m, 9212 m) the front is **8 bands ×
  2.031 m = 16.25 m over 19 voxels**, product eighths top→bottom
  `[4,7,6,5,5,4,4,4,3,2,2,2,1,1,1,1,1,1,1]`, and the **form flips at 295/294 from debris
  (parent as clasts) to `structure` + `pore_fill`** — the user's proposed
  `structure → pore_fill` shape. **The hard perimeter is gone at BOTH faces:** the deepest
  front voxel is **7/8 parent structure + 1/8 product**, never 8/8 product against pristine
  basement. Shape = `WEATHERING_PROFILE [7,5,4,3,2,1,1,1] == round(7·exp(−j/3))` — exponential
  because a weathering front is a **reaction front** (downward-advecting reactant, first-order
  kinetics), scale-free so the ledger sets only the size, capped at 7/8 because **saprolite is
  *defined* by retained parent fabric**. The front grows **downward into unrecorded basement**
  (nothing above moves) and is ~**2.67×** the product metres.
  **Distinguished from the `mixed_voxel_contents` artifact by three separators** (the trap
  journal/0097 warned about): 19 voxels not one · the ladder is read off **`Single`** plans
  (voxels wholly inside one band — nothing to straddle) · and **it scales with the model** (the
  median cell, 1.16 m, grades `[5,4,2,1]` over 4 voxels — quantization would be one voxel at
  both magnitudes, a decorative gradient identical at both). Mass: **exact at the record tier**
  (`the_weathering_front_conserves_the_ledger_product_mass`, five magnitudes over three orders);
  at the voxel tier the eighths are a *draw* (journal/0055's estimator doctrine) — see Observed.
  Flag-OFF **byte-identical, no golden edited** (`generated_world_is_byte_identical_to_the_pre_contract_goldens`,
  `geology_world_regenerates_byte_identically`, `the_production_world_still_hashes_to_the_pre_slice_goldens`).
  Gates: 261 passed after `cargo clean -p dc-worldgen`. **Constants are a stand-in → stubs #20**
  (decay length / the 7/8 cap / the 2.67× ratio are measured from nothing; heir = the deep tier
  carrying a depth-resolved term). **PLEA recorded, not acted on:** the ledger should eventually
  carry the profile, because the collapse can only impose a **universal** shape — granite under
  wet tropical saprolite and under a stripped periglacial slope get the same normalized curve,
  while the physical controls (front-descent vs erosion rate, fracture density, climate) already
  live in the deep sim. Follow-on slice; the fold shape and mass contract here survive it.
  **OWED: the flag-ON appearance walk** — this is an appearance change and the picture is the
  user's. Station: world (84185 m, 9212 m), front at voxels y=299…281; cut a **bench**,
  `--fullbright`, read with `world_get_contents`. **A flag-ON chunk-latency number was NOT
  taken — unmeasured** (flag-off is byte-identical ⇒ zero; flag-on adds 8 events per banded
  column and a 2.67× deeper record, nothing per-frame or per-tick).
- 2026-07-25 — **`FactLedger` gets the CSR layout `flux.rs` proves — a PURE LAYOUT CHANGE**
  (journal/0100; background agent, worktree). The ledger was `Vec<Vec<Fact>>` keyed per
  (cell, slot): **98.8 % of 5,832,862 inner `Vec`s EMPTY, 86–89 % of its ~150 MiB heap in their
  headers** (S19 § 3b). It is now **flat exact-sized facts + a sparse `(slot, start)` CSR index**
  emitted **only for slots that carry facts** — so an unweathered cell (the large majority)
  allocates **nothing at all**.
  - **MEASURED before/after, both real runs of `examples/flow_cost_probe.rs`** (seed 1337,
    `Extent::Medium`, production flags; the pre-slice sources were rebuilt and re-measured, not
    quoted): **flag ON `DeepField` 311.02 → 179.12 MiB**; the **cost of turning the flag on
    falls +161.81 → +29.91 MiB (5.41×)**; **ledger heap 155.01 → 16.31 MiB (9.5×)**; index is
    **0.55 MiB = 3.4 % of the ledger** (below `flux.rs`'s 5.6 % floor — a row is paid only where
    facts exist); payload **15.77 MiB, exact-sized** (was 21.51 MiB, 5.74 MiB of it `Vec` slack).
    The shared flag-OFF baseline is **149.21 MiB, identical in both runs** — the control.
    *Struct cost went the other way and is reported: per-cell `FactLedger` 24 → 48 B = +6.80 MiB.*
  - **BYTE-IDENTICAL, and in the strongest available form:** **1,033,189 facts across 72,006
    slots in both runs** — the same million facts in the same slots, not merely a green suite.
    Green **by name**, unmoved: `the_production_world_still_hashes_to_the_pre_slice_goldens`,
    `identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical`,
    `on_flag_is_purely_additive_record_and_surface_untouched`,
    `a_weathered_cell_carries_one_fact_per_agent_per_chapter_and_accumulates`,
    `production_scale_saprolite_band_reaches_at_least_one_voxel`, `one_fact_per_agent_per_firing`,
    `bedrock_facts_key_stably_as_the_record_grows`.
  - **THE TRAP WAS ORDER.** Fact order *within a slot* is observable — `commit_chapter` merges
    into the **earliest** `(chapter, cause, from, to)` match and `compose_unit` folds in order
    through a clamping `apply_move`. `append_merged` scans the slot's run in order and inserts a
    new fact at the run's **end**, bumping later rows' offsets. New test states it directly:
    `fact_order_within_a_slot_is_preserved_across_interleaved_slots`; plus
    `the_ledger_costs_its_facts_not_its_slots` (a residency **bound**, not a snapshot) and
    `rekeying_moves_a_run_and_leaves_it_exact_sized`.
  - **A-4 DISCHARGED — ported, not designed** (spines.md § A-4). One reasoned divergence from
    `flux.rs`, stated in code: `FluxRecord` can afford a **dense** offsets array because every
    cell exists; the ledger cannot, because dense-offsets-per-slot **is** the rectangle being
    avoided — so its rows carry their own slot key (doubly compressed).
  - **GEN TIME FELL 85.6 → 34.7 s** for the flag-ON field (~2.5×), unasked-for: `finalize_ledgers`
    used to allocate and zero 5.8 M `Vec` headers (133 MiB) it never wrote to. Reported, not
    celebrated — gen time is free by doctrine.
  - **DEFECT FOUND AND FIXED EN ROUTE (its own line, not incidental):** `examples/flow_cost_probe.rs`
    **was broken on main** — `DeepField::resident_bytes()` gained `+ self.flux.resident_bytes()`
    when FLOW slice 1 merged, but the probe's itemisation had no flux row, so its
    `assert_eq!(itemised, resident_bytes())` panicked before printing a byte (113,820,517 vs
    156,454,637). Invisible to the gate because **`cargo test --workspace` builds examples but
    never runs them** — a runtime assertion in an example is unguarded. Row added; the flux record
    now appears in the residency table (40.66 MiB, 27.25 % of the OFF field) for the first time.
  - **OWED / next lever (filed, not done):** the per-cell `FactLedger` struct is still two `Vec`
    headers × 297,025 cells = 13.60 MiB. The *full* `flux.rs` shape — **one** record for the whole
    grid with the cell as the CSR row — collapses that too, but it moves `DeepField::ledgers` and
    `ledger_at_voxel`, which sat in a sibling's write-set this cycle. Generalises: **any per-cell
    owning container in a 297 k-cell field is a header × 297 k before it stores anything.**

- 2026-07-25 — **FLOW slice 1: flux on FACES — the RECORDING half** (journal/0096; background
  agent, worktree; **the arc STAYS OPEN — continuation slot (a)–(e) intact**). The drainage
  solve's **output representation** is replaced; the solve itself (priority-flood → D8 route →
  accumulate) is untouched, as flow.md § 7 requires. New `deeptime/flux.rs` + `dc:deep/flow_record`,
  a declared pass on the runner (`reads [Routed, Energy]`, `writes [FlowFlux]` — a pure write axis
  no in-epoch pass reads, exactly the geotherm/saprolite shape, so erosion is byte-unchanged).
  - **ACCEPTANCE MET on the production world** (seed 1337, `Extent::Medium`, production flags,
    `examples/flux_record_probe.rs`): **DIVERGENCE 175,320** `(cell,chapter)` junctions with ≥2
    out-faces — **7.378 %** of all pairs, max 6 out-faces — against a receiver tree's *identically
    zero, by construction, forever*. **CONVERGENCE 60,915** (2.564 %, max 8 in-faces).
  - **WHERE THE DIVERGENCE COMES FROM — and it needed no new numerics.** A chapter is 25 epochs;
    the terrain moves under the flow every one of them, so a cell's steepest-descent receiver
    *switches*. Accumulating each epoch's discharge onto the face it crossed and totalling **per
    chapter** records "this much left eastward AND this much left southward" — which is
    **avulsion**, the physical origin of braid plains and fans. So the "keep every chapter" fix for
    the deepest defect (*the process was run 200× and only the last frame kept*) **is** the
    mechanism that produces divergence. Same fix, both problems.
  - **MEASURED RESIDENCY** (gen is free, residency is not — flow.md § 9.1's open question, now
    answered with a number): **40.66 MiB** = 2,590,372 entries × **16 B** + a 1.13 MiB CSR index;
    **143.54 B/cell**, **17.94 B/cell/chapter**; **face sparsity 8.386 %** (13.627 % of the
    lateral-only rectangle) — the number S19's cost model was missing. **RE-MEASURED ON MERGED
    MAIN 2026-07-25: DeepField 108.55 → 149.21 MiB (1.37×).** *(The slice reported 162.57 →
    203.23 MiB / 1.25×; its worktree forked before the `shrink_to_fit` free win landed, so the
    ratio was taken against the pre-shrink baseline. The record's own 40.66 MiB is unchanged —
    only the denominator moved.)* **Net effect on the session: 162.57 → 149.21 MiB — residency
    went DOWN 13.4 MiB while gaining a whole per-chapter flow record**, because the free win
    (−54.02 MiB) more than paid for it. Layout heeds S19: flat exact-sized arrays + CSR,
    **never** the `Vec<Vec<…>>` shape
    the probe measured at 98.8 % empty inner Vecs.
  - **OBSERVED / OWED — the one residency lever, sized but NOT pulled (the user's call):** **79.38 %
    of entries (31.37 MiB) are marine sink faces carrying only the cell's own seeded `area = 1.0`**
    — derivable from their own absence by the same S-2 argument that keeps the atmospheric *source*
    out. Dropping them leaves **9.29 MiB** (5.7 % of the DeepField instead of 25 %). Not taken: it
    changes what an *absent* entry MEANS, which future consumers must live with.
  - **WHAT IS POPULATED vs HONESTLY EMPTY.** Lateral (494,997) and boundary ocean/base-level
    (2,094,960 / 415) are real. **Vertical (slot↔slot) faces are structurally present and ZERO** —
    this solve has no infiltration/percolation/Darcy term, so there is no honest number; heirs are
    continuation (a) + (c). The atmospheric **source** is deliberately unstored (uniform seeded
    `1.0`, exactly derivable — S-2); the atmospheric **sink** (endorheic evaporative termini)
    measured **0** on this world. Of the atom, `load` is live (**494,296 entries = 99.86 % of
    lateral**, 1,864.8 m total) but **bulk only** — composition is Movement 2b's seam; `form`/
    `cause`/`fluid` are carried-but-constant (**new stub #18**, heirs (c)/(d)/Movement 2b).
  - **THE SLOT IS DERIVED, NOT STORED (S-2) — and the measurement vindicated it.** `DepUnit` already
    stamps the chapter and units never merge across one, so `flux::slot_for_chapter` derives it. On
    the small world **six in seven chapter-fluxes have NO surviving stratum** (29,204 resolved vs
    175,596 `None`): net-erosional chapters deposit no unit, and stripped units leave an
    unconformity. A **stored** index would have gone stale and pointed at a stranger's stratum; and
    a per-(cell,slot) archive would have silently dropped 86 % of the flux — the very defect this
    arc exists to fix, re-committed one layer down.
  - **PLEA — flow.md § 2.2 is SILENT on the slot-pairing rule, and the whole seamlessness claim
    rests on it.** "A face is shared by construction" holds at the *cell-pair* level only; adjacent
    columns have no aligned slot indices and § 1.2 forbids correlating by slot index (surfaces are
    diachronous). Implemented and documented: **lateral faces pair by CHAPTER** (a global time
    surface; each column then binds that chapter to its own slot independently — the § 1.2
    discipline applied, not violated). **RESIDUAL, the user's to rule on:** the *bound* regime
    likely pairs by **paleo-elevation**, not chapter (aquifers cross surface divides, § 2.4). Slice
    1 records no bound flux so nothing is forced — but the rule must NOT be assumed to extend.
    Filed with continuation (c).
  - **A-4 DISCHARGED — the two fact-mergers folded to one.** `inventory.rs::commit_chapter` now
    searches the slot instead of matching `facts.last_mut()`; `weather_inventory::coalesce_facts` is
    **deleted**. Verified equivalent (the search merges into the *earliest* match = the post-hoc
    sweep's first-occurrence order; `Σ fraction_m` is invariant), suites green **by name**. Takes an
    O(slots) post-pass off a per-cell/per-epoch path.
  - **BYTE-IDENTITY PROVEN BY NAME:** `the_production_world_still_hashes_to_the_pre_slice_goldens`
    (the cross-commit golden, untouched) + `the_flow_record_is_a_sidecar_and_the_world_is_byte_identical`
    (flag on vs off: `surf`/`regolith`/`area`/`exhum`/`t_crust`/`geotherm` bit-for-bit, `recv`/
    `lake`/`strata` equal). New suite `tests/flux_record.rs`, **12/12 green**, incl.
    `divergence_is_representable_which_a_receiver_tree_forbids`, `convergence_is_representable`,
    `per_chapter_history_is_retained_not_just_the_final_epoch`,
    `a_face_is_shared_so_bs_in_flux_is_exactly_as_stored`,
    `vertical_faces_are_structurally_present_and_honestly_empty`,
    `the_stratum_slot_is_derived_from_the_chapter_stamp`,
    `the_receiver_export_agrees_with_the_final_chapters_faces` (pins `recv` as a **shadow**, not a
    rival authority), `the_flow_record_is_deterministic_on_a_repeated_seed`,
    `parallel_and_scalar_record_the_same_flux`, `a_flux_entry_is_sixteen_bytes`.
  - **NOTHING IS EXPRESSED AT RUNTIME, deliberately.** `RiverSeg`/`carve_rivers`/`BANK`/
    `RIVER_REACH`, `pregen/hydrology.rs`, `Cell::{flow_to,river,discharge}` all **untouched** — the
    world stays honestly river-less rather than gaining a second fake. Their retirement is
    continuation **(e)**. spines § 3: `recv`/`area`/`lake` row updated (superseded, heir named); new
    row for `DeepField::flux` (**built, nothing calls it — on purpose**).
  - Rides S-9, S-2, S-4, the § 5 field/cellular split, the north-star pass-runner (plain data +
    opaque ids + bare `fn`; no closures cross the seam). Guards A-3 (acceptance is a divergence
    count on a production world) and A-1 (the honest empties are asserted *as* empty, by name).
  - Gates: `cargo test -p dc-worldgen --release` **34 targets, 0 failures**; fmt + clippy
    (`-p dc-worldgen --all-targets -D warnings`) clean. **`--workspace` gate LEFT FOR THE
    INTEGRATOR.**

- 2026-07-24 — **Movement 3: weathering-as-a-PROCESS — the first CELLULAR pass, accumulating**
  (journal/0094; background agent, worktree; **discharges stub #17**). S18's post-hoc one-shot
  (`field.rs::build_ledgers`, **deleted**) is relocated into the deep-time loop as
  `dc:deep/weather_inventory` — a declared **cellular** pass on the runner, gated behind
  `weather_inventory` (absent = off ⇒ byte-identical). It weathers each subaerial cell's bedrock
  `Structure` seam → `Loose` saprolite **every epoch on that epoch's LIVE terrain**
  (contemporaneous regolith `H`/frost/biotic, `reads_prev BioMod`), **accumulating** cause-carrying
  facts across the whole run. **The span-index crux** (the record grows every epoch, so the bedrock's
  numeric index `units.len()` shifts) is solved by keying the accumulator to a **stable bedrock
  sentinel** (bedrock-only ledger, slot 0, record-growth-invariant), re-keyed onto the final record
  at loop end (`finalize_ledgers`). **`dt` goes LIVE** (`share ∝ dt`, the pass's phase length —
  journal/0090 M3 deferral closed). **Two-authorities split HELD** (material-behavior.md §11): the
  pass READS `H` but WRITES ONLY the ledger sink (`DeepAxis::Saprolite`, no in-epoch reader, like
  the geotherm) — it never touches `R`/`H`, so the height-tier `dc:deep/weather` pass and the erosion
  result are **byte-unchanged** (`on_flag_is_purely_additive...`). **Production-scale band at argmax:
  6.09 m = 6.77 voxels @ 0.9 m (54 eighths), and 24.2 % of land (71,748 / 297,025 cells) carries
  some band, mean 1.61 m over banded cells** — ~152× the S18 sub-voxel miss it redeems. *(Corrected
  2026-07-25: this entry previously read "≥1 voxel", which is the **A-3 guard's threshold**
  (`production_scale_saprolite_band_reaches_at_least_one_voxel`), **not the measured result** — the
  integrator had verified 6.09 m independently and then wrote the assertion's floor into the map.
  Caught by the walk, journal/0097. "Demand the load-bearing number" applies to the bookkeeping too,
  not only to the agent report.)* Tests by name: byte-identity off
  (`the_production_world_still_hashes_to_the_pre_slice_goldens`,
  `identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical`),
  `weathering_accumulates_across_epochs`, `bedrock_facts_key_stably_as_the_record_grows`,
  `one_fact_per_agent_per_firing`, `dt_scales_the_share_linearly`,
  `weather_inventory_is_absent_off_and_a_declared_cellular_pass_on`. Rides S-1 (the LOOP is the
  relaxation), S-2/S-9/S-5; discharges A-4, guards A-3. **Loose product still inherits stub #16's
  stand-in bedrock identity** (flat granite basement) — #16 NOT retired. Per-crate gates green
  (dc-worldgen); **`--workspace` gate run by the integrator on merged main — green, 664 tests.**
  **✅ WALK DONE & ACCEPTED 2026-07-25 (user; journal/0097, assets
  `0094-weathering-band-flag-{on,off}.png` + `-pit-topdown-on.png`).** Same column, same seed, flag
  flipped: **7 voxels ≈ 6.3 m of loose `dc:mudstone` at the basement contact** against a predicted
  6.09 m, veneer above and basement below **unmoved** — the flag's product and only the flag's
  product. **BUT accepted WITH a follow-up the walk found (see Sequenced "the weathering front needs
  a PROFILE"): the band has a HARD PERIMETER** — pure 8/8 product abutting pristine bedrock, because
  the collapse folds the scalar `weathering_product_m` into a **single stratum of one class**. A
  scalar cannot carry a profile, and a front without a gradient is not a front. Not a blocker.

- 2026-07-24 — **The geotherm — the FIRST field pass** (journal/0093; background agent,
  worktree; **world-changing: coal moves**). New `deeptime/geotherm.rs` + `dc:deep/geotherm`
  declared **field pass** on the runner (reads Climate+CrustThick, writes Geotherm, `period=40`
  low rate; **plants a field, runs NO edges** → erosion byte-unchanged). The §5 field-pass half,
  landed. First condition-field: **`dc:field/temperature`**, a per-cell geothermal gradient
  (v1 **linear**, tectonic setting + `t_crust` thickness modifier; rift hot, craton cold,
  clamped 12–55 °C/km). **Retires the degenerate `burial_temp_c` stub (#14) ENTIRELY** — the
  honest finding: a real `T(depth)` is a **field**, not a value a `fn(unit)` provider-slot could
  hold, so it *left* the provider set (journal/0093, blogworthy: "the seam that told you it
  wasn't a seam"). `promote_coal` reads the geotherm at seam mid-depth vs onset. **Coal
  recalibrated** (`COAL_ONSET_C 8→22 °C`): 12% → 60% of peat candidates, relocated to warm
  crust, still a diggable seam on Medium (`the_geotherm_coal_shift_is_plausible_not_degenerate`).
  **`GOLDEN_SURFACE` UNCHANGED** (independent coal-only proof), `GOLDEN_RECORD` re-captured. Gate
  green by name; **NO capability tiering** (north-star Deviation #2). **Rides-as-built:** burial
  is shallow so the geotherm ≈ surface temp at seam depth → the tectonic gradient barely moves
  coal (~surface-temp-thresholded); its real payoff is **metamorphism** (deep crust: `exhum` = P,
  geotherm = T → grade). **Appearance walk owed but low-priority** — coal is a user-blessed
  placeholder; don't over-calibrate (it recalibrates when biology lands).

- 2026-07-24 — **Movement 2a: R/H are derived views of the inventory** (journal/0092;
  background agent, worktree). The per-cell working inventory (== the strata record) is now the
  **authority for surface material; `R`/`H` are DERIVED.** `H = Σ surface Loose above the
  topmost Structure` (positional — **cave fill / buried loose excluded**, which the scalar plane
  structurally cannot represent). **Scratch-first reconcile:** erosion stays on the fast scalar
  planes; deposition reconciles net ΔH into the record as `void→Loose`/`Loose→void` facts once
  per epoch at the boundary (no per-inner-loop walk), material = the **real current `deep_class`
  rule** (not an anonymous stub). **BYTE-IDENTICAL** — goldens unmoved by name; agreement over
  the 25 600-cell production field to **8.3e-8 m** (f64 round-off); `buried_loose_below_a_structure_is_excluded_from_surface_h`.
  **Finding (rides-as-built, byte-identity held):** the engine's `R` is a bedrock-top *elevation
  datum*, so `R = surf − H` (topmost-Structure contact), not `Σ Structure`; `Σ Structure` exists
  beside it. **Zero added resident cost** (materialize-on-demand). Combined gate green by name
  (fmt/clippy/test `--workspace`, both crates cleaned). Next: 2b (material-aware transport), 3
  (weathering-as-a-process), and the geotherm (first field pass).

- 2026-07-24 — **Far-LOD band-inversion + constant-coupling fix** (journal/0091; background
  agent, worktree; **render-only, goldens unmoved**). One **`LodLadder`** — nearfield border +
  each step's range **as a distance past the border**, all named knobs (structured to become
  in-game per-player perf settings); ring edges + near/unload radii all **derive** from it (grep-
  proven no duplicate range constants; the `176 m` standoff and `RING_LADDER` gone). **Warm
  floored geometry-safe** (`floor_known_surface` — cold and warm floor to the identical height).
  **Standoff removed:** warm where the node subtree is resident, cold only where not (A-5 guard
  intact). Kills the **finest-band-cold-over-resident-warm-data inversion** + the §6.3 coupling
  defect (audit `2026-07-24-lod-pre-post-visit-diagnosis.md`). Fix (b) — cold/warm **material**
  agreement (S-9) — stays for the mixture arc (now a material seam, not a geometry one). Combined
  gate green by name. **Appearance walk owed.**

- 2026-07-24 — **Movement 1: the deep-time pass-runner — the erosion loop re-housed as
  self-declaring passes** (journal/0090; background implementation agent, worktree; the first
  north-star pass-runner instance at the deep tier). `erosion.step`'s monolith decomposed into
  **14 declared passes** on a new `deeptime/runner.rs`, each declaring `{reads, writes, rate}`.
  **ORDER** = topo-sort over a `DeepAxis` vocabulary via a **shared `passgraph.rs` kernel that
  `pipeline.rs` was refactored onto** (A-4 guarded — one runner, not two; verified by the
  spine-audit). **RATE** = `period` (epochs between firings) + `dt` (phase length, threaded but
  **inert this movement** — Movement 3 makes it live). The field-relaxation pipeline is expressed
  via **revision tokens** (`Forced→Weathered→Diffused→Compensated→Windblown→Settled`) since a
  single "Terrain" axis can't order readers between successive terrain-writers. **The biology↔
  erosion one-epoch lag is a declared `reads_prev` loop-carried edge** — a test proves the runner
  *rejects* it as a cycle if declared a within-epoch `reads` (the ecology.md guarantee now enforced,
  not commented). **`climate` is a low-rate pass** (`period = remarch_interval`). **BYTE-IDENTICAL**
  — production goldens unmoved (`the_production_world_still_hashes_to_the_pre_slice_goldens`,
  `generated_world_is_byte_identical_to_the_pre_contract_goldens`, `full_agents`/`tectonic_history`/
  `deep_config_plumbing` green); byte-identity *as the check* caught the agent's own dropped
  sea-level assignment. Crossing-constraint held (declarations plain data + ids, bodies bare `fn`).
  Gates green on merged main (fmt/clippy/test `--workspace --release`, dc-worldgen cleaned first).
  **One fusion held:** transport+incision stay one pass (interleaved flux chain — splitting is an
  algorithm change, not a re-housing). R/H unchanged; no behavior change. **Next: Movement 2
  (R/H unification + material-aware transport, material-behavior.md §13).**

- 2026-07-24 — **S18: the first real behavior — sum-agent weathering on the deep-cell
  inventory** (journal/0089, `docs/spikes/S18-*-plan.md`; background implementation
  agent, worktree for the integrator). **Consumes the S17 keystone** (`WorkingInventory`
  / `FactLedger` went from tested-only to a production consumer — A-4 discharged). Weathering
  now runs as a **cellular pass over the working inventory** (`deeptime/weather_inventory.rs`):
  `rate = cover_taper × Σ_a (driver_a × susceptibility_{m,a})` — the **sum-agent** model
  (chemical/biotic/frost), **not** S16's product (byte-identity with the legacy weather phase
  deliberately retired, DECIDED 85eee85). One **cause-carrying fact per agent**, sourced from an
  **apply-time edge log** (`InvCtx::ctx_for(chapter, cause)`; `diff_facts` demoted to a
  `#[cfg(test)]` reconciliation check). `Fact::InPlace` gained `cause: Cause` (Chemical/Biotic/
  Frost/Dissolution, room left for a present-tier `Actor`). **The consumer:** `DeepField` carries
  a `Vec<FactLedger>` sidecar; `collapse.rs` folds `base + facts` into a **basal saprolite band**
  below the recorded pile (not counted in `expressed_m`). **Bedrock materialized as a flat
  `Structure` span** so the `Structure→Loose` edge has a source — **stubs.md #16**, heir = a
  genesis/emplacement pass, loud code marker present. **Flag `weather_inventory` OFF by default ⇒
  byte-identical production world** — gates green on merged main (fmt ✓, test ✓ `--workspace
  --release`, `cargo clean -p dc-worldgen` first, `Compiling dc-worldgen` from main's tree):
  `the_production_world_still_hashes_to_the_pre_slice_goldens ok`,
  `identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical ok`, +
  `one_fact_per_agent_and_shares_sum_to_the_move`, `frost_acts_even_where_biota_is_absent`,
  `the_weathering_front_folds_into_a_basal_band`. Rides S-9/S-2/S-1/S-5; no dc-core touched.
  **OWED: the flag-ON walk** (the saprolite band is an appearance flip, walk-gated — exemplar =
  argmax `FactLedger::weathering_product_m`, cut down a scarp/gorge to the basement contact).
  Rides-as-built (integrator-settled, no user call): the subaerial gate reads the *record* not
  final `surf` (post-isostasy `surf ≈ −460 m` would weather nothing); the product class is a
  fine-clastic saprolite stand-in under #16; one chapter only (multi-chapter feedback is the
  §5 later refinement).
  **CORRECTED 2026-07-24 (corrections #46/#47; walk finding, headless tour probe
  `examples/s18_weathering_tour`):** the band is **SUB-VOXEL** at production scale (max
  **0.04 m** vs 0.9 m voxel ⇒ 0 eighths ⇒ **invisible**; confirmed in-client). And the deeper
  miss — this runs weathering **once, post-hoc, over the finished record with frozen fields**,
  so what shipped is keystone-consumption **plumbing + a one-shot behavior STUB (stubs.md #17),
  NOT weathering-as-a-process.** Do NOT read "folds into a band" above as a shipped visible
  effect. The **real first behavior** is weathering run per-epoch in the deep-time loop /
  riding the `H` process (the R/H unification), carrying material + `cause` — sequenced next.
  **DONE 2026-07-24 (Movement 3, journal/0094, above): stub #17 discharged, band ≥1 voxel.**

- 2026-07-23 — **The per-task generator: neighbour fill and far derive leave the
  frame thread** (journal/0084; the filed follow-on to the async-offload slice
  journal/0083; background implementation agent, worktree for the integrator).
  **Landed the fix 0083 measured-then-filed:** push `neighbor_fill.gen` (~5 ms/call
  frame-thread) and the bulk of `far_tile.derive` (the biggest single frame-thread
  span, ~1.9 ms/call) off-thread by **minting a per-task `WorldGenerator` from the
  shared `Arc<Pregen>`** — each mesh task resolves its own neighbour contents /
  derives its own far surface against its own generator, with **no contention on
  the shared generator `Mutex`** (the trap that blocked 0083). **Byte-identical
  world:** generation is a pure function of `(pregen, pos)`, so a per-task
  generator over the same pregen produces byte-identical contents / coarse surface
  to the shared one — proven by
  `authority::tests::per_task_generator_is_byte_identical_to_the_shared_one`,
  `meshing::tests::plan_backed_mesh_equals_direct_mesh` (near), and
  `farmesh::tests::offloaded_far_derive_matches_on_thread` (far). **Near** (PRIMARY):
  `NeighborShellPlan::gather` reads each border neighbour's edit-aware **block** on
  the frame thread (cheap; edits must stay authority-sourced); the pure-terrain
  **contents** resolution is deferred into the task via the per-task generator.
  **Far** (SECONDARY, offloaded — it *was* cleanly separable): the frame thread
  snapshots the 3×3 patch of `known_node_grids` the tile touches (the ONE
  `&mut FarPyramid` read, a pure function of tile coords — recorded under a new
  `far_tile.snapshot` span), and the task derives the 34² `coarse_surface` samples
  against the per-task generator. Per-task construction cost measured **9.9 µs**
  (`per_task_generator_construction_is_cheap`, Medium pregen) — off the frame
  thread, amortized over the task's many generator queries; pooling filed as a
  non-need. No headless crate touched; only observable change remains appearance
  order/timing (already accepted, 0083). **Re-capture owed to the integrator:** a
  fresh `--perf-drop 20` should show `neighbor_fill.gen` (3.1 %) and
  `far_tile.derive` (7.0 %) LEAVE the frame `schedule` self-time envelope (they now
  record on task threads), with a small `far_tile.snapshot` residue on the frame
  thread (the pyramid derive, formerly folded inside the on-thread derive; the next
  target if it is large). Gates green — fmt/clippy/test all `--release`, both clippy
  paths incl. `--features perf`, `cargo clean -p dc-client --release` before the
  test gate, `Compiling dc-client` confirmed from this worktree, 118 dc-client tests
  pass (0 failed).

- 2026-07-23 — **S16: weathering wears the north-star behavior shape, byte-for-byte**
  (journal/0085, `docs/spikes/S16-weathering-behavior-shape-results.md`; background
  spike agent, worktree for the integrator; the make-or-break de-risk of the
  material-behavior model, north-star de-risk item (2)'s sibling — a real behavior,
  not the fires proxy). The subaerial bedrock→regolith **weathering** conversion
  (`erosion::weather`) reformulated onto the ratified **Pass / Material / ctx /
  Transform** shape over the **thin ctx-adapter-over-heights** (Fork 2, ratified):
  a pass declaring `{reads, writes}`, a *pure* `BedrockWeather` behavior
  (`weather_rate` / `weather → Transform`), and a `WeatherCtx` capability whose read
  side is the SDK surface and whose `apply` (the R→H transfer) is pass-owned —
  purity enforced *structurally* (the behavior holds a `&WeatherCtx` with no write
  path). **Byte-identity VERDICT: goldens UNMOVED** (`GOLDEN_SURFACE
  0x176D_40F1_1CCB_006A`, `GOLDEN_RECORD 0xC9C6_D6F6_E908_9653`;
  `providers_golden::the_production_world_still_hashes_to_the_pre_slice_goldens`
  ok). **`form_change(Structural→Loose)` maps cleanly onto `R -= q; H += q; dH += q`
  with ZERO new deep state** — the transfer is a clean *view* over the height
  stocks; keepable. **Where it strained (the diagnostic, both reported not faked):**
  (1) the height tier has no single outcropping material — the rate is a
  share-weighted *blend* over the near-surface window, so `self.weatherability`
  can't be the byte-identical source and the `materials_with(Weather)` loop
  degenerates to one synthetic body; (2) weathering is a sum over *agents* — the
  two-factor `base × (biotic × weatherability) × taper` sketch had to grow the
  periglacial **frost** factor (`× frost`) to stay bit-exact. Both localize to one
  ctx method + one missing capability, and **both are the deep-cell material
  inventory question already coupled to Crux 1** — the spike named the seam, didn't
  invent state to hide it. dc-core gains a `weatherability` `MaterialProps` axis
  (distinct from mechanical `smash`, like `solubility` — avoids the
  one-number-erodibility trap; ordered soft→hard, reference clastic pinned at 1.0,
  pinned to agree in ordering with the abrasion proxy: authority, not summary).
  Gates green on the changed crates (dc-core + dc-worldgen): fmt `--all --check`,
  clippy `--all-targets --release -D warnings`, tests `--release` (goldens +
  4 new shape tests + dc-core `weatherability_orders_soft_over_hard_with_the_reference_at_one`),
  every `Compiling`/`Checking dc-core`/`dc-worldgen` line verified from this
  worktree. Write-set dc-worldgen + dc-core only; dc-client untouched (concurrent
  agent). **Recommendation: KEEPABLE shape, one seam (per-cell material inventory)
  left open.**

- 2026-07-23 — **The render-first wedge was already driven — a falsified premise,
  a guard instead of a deletion** (journal/0082; background implementation agent,
  worktree for the integrator; step 1 of the north star / block↔material collapse,
  crux (a)). The dispatched wedge — delete `meshing.rs::block_layer`'s geology
  re-translation and route `classify → material → atlas` — rested on a mental
  model the code had outrun. **The near-field mesher's contents-bearing path
  already routes `contents → material → material_layer` directly** (`top_splat`
  emits `material_layer(m)` per constituent; it never calls `block_layer`). The
  `block_layer` geology arms are **not dead**: they are the block-only
  (contents-absent) render summary, live in **production far rendering**
  (`farmesh.rs::push_quad` over the far pyramid's `MajorityNonAir` block spans),
  in the benches, and in the near field's absent-contents geology fallback — the
  *same* mechanism as the four legacy `grass/dirt/stone/wood` arms the crux keeps
  (the brief's own reason for keeping those applies verbatim). Deleting them in
  isolation would break the far field (out of scope) or make `block_layer`
  non-total, so **no deletion** — the brief's own escape hatch (loud plea, don't
  force a mess). **Landed instead:** a guard test —
  `meshing::tests::block_only_geology_layer_agrees_with_direct_material_layer` —
  pinning `material_layer(m) == block_layer(block_twin(m))` for the seven primary
  geology blocks (so the direct and block-only routes cannot drift to different
  atlas layers), plus the siltstone corollary asserting a secondary member stays
  *distinct* (the material route never degenerates back into the block route —
  the walk-10 member-identity kill). Byte-identical: **test-only, zero production
  change.** Findings: the brief's proposed acceptance relation only holds for the
  seven *primary* materials (`block_twin` is many-to-one); the one residual real
  Block→layer round-trip for geology is the far-field top face
  (`farmesh.rs::push_quad`), a separate slice feasible via the far pyramid's
  material store; Crux 1 (`Block = {Air, Material(MaterialId)}`) subsumes all
  residuals at once and is the honest next move. Gates green — fmt/clippy/test all
  `--release`, `cargo clean -p dc-client --release` before the test gate, verified
  `Compiling dc-client` from this worktree.

- 2026-07-23 — **Async-offload: CPU meshing leaves the frame thread** (journal/
  0083; background implementation agent, worktree for the integrator; gates green
  — fmt/clippy/test all `--release`, both clippy paths incl. `--features perf`,
  `cargo clean -p dc-client --release` before the test gate; `Checking dc-client`
  confirmed from this worktree). **The perf baseline retargeted this slice:** the
  `--perf-drop 20` capture (docs/audits/2026-07-23) overturned the "synchronous
  chunk gen is the killer" hypothesis this slice was sequenced against — **gen is
  14 µs / 0.1 %, leave it alone**; the per-frame killer is **CPU meshing**
  (`far_tile.derive` 10.3 %, `mesh_chunk` 8.2 %, `neighbor_fill.gen` 4.1 %,
  `far_tile.mesh` 2.6 %). So the offload moves the **pure meshing**, not gen, onto
  `bevy::tasks::AsyncComputeTaskPool`: `meshtasks.rs` (in-flight `Task` maps +
  outputs + a `drain_finished` poll helper) plus a rewire of `streaming.rs`
  (`stream_chunks` gathers owned inputs → spawns a `mesh_chunk` + `to_bevy_mesh`
  task; `drain_near_meshes` does the main-thread GPU tail — `Assets<Mesh>` insert
  + entity spawn) and `farmesh.rs` (same shape for `build_far_tile_mesh`, whose
  purity journal/0070 had already filed as an async drop-in). **Determinism
  untouched — render path only; the world stays byte-identical.** Meshing is a
  pure function of owned data, pinned by
  `meshing::tests::shell_backed_mesh_equals_direct_mesh` (mesh via a pre-resolved
  owned `NeighborShell` == mesh via the live authority closure). **The
  `neighbor_fill.gen` crux, measured then decided (option b):** the 4.7 ms/call is
  `chunk_contents` locking the single `WorldGenerator` `Mutex`; sharing that mutex
  across threads would stall the frame thread's own `chunk.gen` behind a
  background lock-holder (a contention regression unmeasurable without a windowed
  client), and `far_tile.derive` additionally reads the `&mut FarPyramid`
  resource. So neighbour coverage + far derivation are resolved on the frame
  thread as OWNED data and only the pure mesh is offloaded. **The only observable
  change is chunk/tile appearance ORDER** (async completion is not strictly
  nearest-first; task *spawn* still is). **Re-capture owed to the integrator:** a
  fresh `--perf-drop 20` should show `mesh_chunk` (8.2 %) + `far_tile.mesh`
  (2.6 %) LEAVE the frame-thread `schedule` self-time envelope (the layer is
  per-thread — perf.rs); they still fire on a task thread. **Filed follow-on:**
  push `neighbor_fill.gen` + `far_tile.derive` off-thread via a per-task
  `WorldGenerator` minted from the shared `Arc<Pregen>` — no mutex contention,
  recomputation free under two-clocks; wants live-client contention measurement,
  so sequenced not forced.

- 2026-07-23 — **The perf window opens — runtime span profiling, built to the
  overlay heir** (journal/0080, docs/audits/2026-07-23-perf-baseline-vertical-
  drop.md; spines § S-3 gains a compliance instance; background implementation
  agent, worktree for the integrator; gates green — fmt/clippy/test all
  `--release`, both clippy paths incl. `--features perf`, with `cargo clean -p
  dc-client --release` before the test gate). The project measured gen-time
  rigorously but had **zero** runtime span profiling — the walk-0071 vertical-drop
  hitch could not be attributed to gen vs meshing vs tick. Now a `perf` cargo
  feature on dc-client (OFF by default → **zero runtime cost** in normal play: the
  `perf_span!` macro compiles to a zero-sized guard, no aggregating layer, no
  `bevy/trace`) turns on `tracing` spans on the hot paths + bevy's own `trace`
  spans + a self-time aggregating `tracing_subscriber::Layer` installed via
  `LogPlugin::custom_layer`. **S-3 applied to timing data:** the aggregate
  (`perf::PerfAggregate`, per-span exclusive self-time + call count) is the
  authority, held as the `PerfHandle` resource; the `docs/audits/` ranked-table
  dump is the first consumer and the ratified in-game perf/debug overlay is the
  named heir that reads the SAME resource live (not built — clean seam). Spans:
  `stream_chunks` → `chunk.gen`, `chunk.contents`, `far_pyramid.insert_l0`,
  `mesh_chunk` → `neighbor_fill.gen` (the hidden lazily-generated-neighbour cost),
  `to_bevy_mesh`; plus `host.tick`, `physics.step`, `far_tile.build` →
  `far_tile.derive`/`far_tile.mesh`, `far_chunk.build`. A `--perf-drop <secs>`
  capture mode scripts the drop deterministically (teleport −30 m/0.5 s, no
  physics), resets the aggregate after warm-up, writes the ranked artifact, exits
  `0`. **No headless crate touched; determinism untouched (observability only).**
  Baseline **numbers PENDING** — a windowed GPU client could not run in the agent
  environment; the artifact is a schema-complete stub the integrator fills by
  running `--perf-drop 20`. **Zero cost off is verified** (default clippy clean,
  all `enabled` items behind `#[cfg(feature = "perf")]`). **Followed by
  async-offload (journal/0083, above): the baseline this instrument first
  produced retargeted that slice away from "sync gen is the killer" — gen is
  14 µs; CPU meshing was the killer.**

- 2026-07-23 — **`paleo_temperature` becomes a seam, and the collapse tier grows
  a provider socket** (journal/0078; background implementation agent, worktree
  for the integrator; gates green — fmt/clippy/test all `--release`, with
  `cargo clean -p dc-worldgen -p dc-core --release` before the final gate).
  **Zero behaviour change** — the production world still hashes to the pre-slice
  goldens `surface 0x176D40F11CCB006A` / `record 0xC9C6D6F6E9089653`.
  `geology.rs::deposit_deep_history` read *today's* column temperature
  (`ctx.temp_c`) as every deep unit's at-deposition temperature, while the
  sibling aridity axis (`deep_precip`) already read the record — the asymmetry
  sat on adjacent lines. Now `providers::Providers::paleo_temperature` — *"what
  temperature did this cell see when this unit was deposited?"* — identity =
  present-day `temp_c` (the wrong quantity, named, not a neutral no-op); heir = an
  epoch-indexed paleo curve keyed by the unit's `chapter`. **Value-level per
  unit** (not per column): the identity is constant across a column's units but
  the heir varies per epoch, and granularity follows the heir. **Finding:** the
  collapse tier had **no** `Providers` channel at all — the mechanism was only
  ever plumbed into the deep-time sim via `DeepConfig`. `WorldGenerator` +
  `StrataCtx` now carry a `Providers` (default = identity, resolved at world
  build), so both tiers can be handed one resolved set. Tests: 2 slot-identity, 1
  white-box **consultation** test in `geology.rs` (the golden proves the *absent*
  provider changes nothing, which is consistent with a slot never consulted; the
  white-box test proves it IS consulted through the real fn), 1 none-path arm, 3
  in `providers_paleo_temperature.rs`. Two **doc riders** in the same commit:
  `fits_in_pores` [S9] now declares its expected consumers (hydrology
  infiltration, diagenesis cement/ore) so an infiltration author finds it instead
  of writing a second rule; the `exhum`/`t_crust` [#28] comment cites `spines.md`
  § 3 and states its no-consumer status crisply (it was already substantially
  honest — see corrections #40). **`material_properties` [S2] / `is_granular`
  [S3] remain queued and design-pass-pending** — they couple to the
  block↔material collapse and were left entirely untouched.

- 2026-07-22 — **The surface-branch removal — the summary stopped being the
  author** (journal/0074; background implementation agent, worktree for the
  integrator; **empties spines § S-3's marquee "violation, shipped" line**).
  The S-3 violation is dead: `collapse.rs::surface_sample`'s branch was a
  far-field cheap-surface need that had become the world's surface *material*
  rule. The near surface voxel now **is** the record's top span through
  `ColumnFill` (`plan(1)`), the same authority every buried voxel routes
  through — the buried column shifted down one record span to make room for the
  surface it now owns; heights byte-identical. **`draw_class` survives, re-homed
  as the far-field summary's class picker** (`coarse_surface` — the far field
  cannot afford to build a `StrataRec`), typed as a summary and held to a
  **statistical agreement test** that replaced journal/0055's shared-kernel
  structural guarantee: **0.9171** near/coarse agreement on geology-surfacing
  columns (floor 0.88), the ~8 % disagreement being the fluvial veneer (near is
  *more* correct) and B1's coherent-source bias. **Goldens: both Medium moved on
  all three hashes (authorized); Small unchanged** — record-less columns are
  byte-identical under this slice by construction (the mechanism, not luck —
  corrections #38). Perimeter-guillotine signature **neutral**: the far field is
  byte-unchanged, and the near ground is also nearest-per-460 m in class shares,
  so it neither inherits nor cures the cake edge (cure stays the CoarseField
  heir). Three tests reworked (agreement→statistical; member-dither→
  routes-through-ColumnFill; class-dither-liveness→far). `surface_fill`/`surface_
  member` deleted. dc-worldgen suite green (55 lib + all integration, exit 0);
  full workspace gates run before merge. Far side (`far.rs`/`farpyramid.rs`)
  untouched — its full node-synthesis adoption is the filed follow-on (stubs
  § 15).

- 2026-07-22 — **Coalification becomes a geotherm seam** (journal/0067;
  `stubs.md` § 14 now names a slot; background implementation agent, worktree for
  the integrator; gates green — fmt/clippy/test all `--release`, with
  `cargo clean -p dc-worldgen -p dc-core --release` before the final gate).
  **Zero behaviour change; the calibration was deliberately not touched.** The
  user accepted 0066's coal-on-burial *conditionally*: *"i can accept the coal
  etc as long as uses geotherm seam with heir etc… the calibration is fine, we
  aren't answering deep questions about it right now."* So the 8 m overburden
  test is now `providers::Providers::burial_temp_c` — **"what temperature has
  this buried unit seen?"** — asked per candidate unit at `BioticSim::finalize`
  and compared against `COAL_ONSET_C`. **The slot asks for the quantity, not the
  verdict:** `is_coalified` was the simpler shape and was rejected, because coal
  rank and metamorphic grade are one thermal-maturity ladder (S-8, *one quantity
  many regimes*) — a temperature answers every rung, a predicate answers one and
  composes with none, and it gives the built-but-unconsumed `exhum`/`t_crust`
  planes (`spines.md` § 3) their first named consumer path. **The identity is a
  degenerate geotherm** — 0 °C at the surface, 1 °C/m, so the answer is the
  overburden in metres and the test is bit-for-bit the shipped
  `overburden_m >= 8.0`. The gradient is 40× Earth's and is written down as
  arithmetic rather than dressed as physics: a real gradient would put rounding
  steps between the two comparisons and turn the byte-identity proof into a hope.
  Filed under **structural**, not a new *diagenesis* group — groups name who
  **answers**, and a geotherm is crustal (its input is `t_crust`, a tectonics
  plane); diagenesis merely asks. Byte-identity: `GOLDEN_RECORD` /
  `GOLDEN_SURFACE` pass unchanged. New falsifiers: a **frozen** geotherm leaves
  the production world with zero coal and conserves peat + coal (promotion is a
  retagging), a **molten** one promotes every buried peat *except* each column's
  living surface — the guard that is structural rather than thresholded, and only
  observable at infinity. **Nothing to ratify.** Carried, filed not fixed:
  `overburden_m` is depth below the *present* surface, so an exhumed unit reads
  as shallow where real rank is irreversible (needs a high-water mark in the
  record); and rank is time-at-temperature, but `BuriedUnit` carries no age
  because the record stores a chapter and not a duration — a field the heir could
  not fill is the `wave_energy` mistake.

- 2026-07-22 — **`None` means identity: provider absence becomes structural**
  (journal/0064; corrections #32 remedy superseded; gates green — fmt/clippy/
  test all `--release`, with `cargo clean -p dc-worldgen --release` before the
  final gate). **Pure refactor, zero behaviour change.** 0063 found that slot
  identity, decided by comparing `fn` addresses against `Providers::default()`,
  silently mis-reported (corrections #32), and repaired it with a non-`#[inline]`
  wrapper plus **a rule in a docstring** — which cannot fail a build, on a
  mechanism the user had just made load-bearing (*the resolved provider table is
  part of world identity*, DECIDED 2026-07-22: a mis-report there is a spurious
  load refusal, or silent acceptance of a world generated by providers you no
  longer have). So each slot is now `Option<fn(..)>` with **`None` = identity**:
  `Default` derived all-`None`, `non_identity_slots()` an exhaustive `is_some()`
  field check, `Providers::address` deleted, **no address taken anywhere in the
  crate**. Call sites keep their shape via one accessor per slot sharing the
  slot's name (`providers.outcrop_at(top)` — the field is `x.outcrop_at`, the
  method is `x.outcrop_at(..)`), so the `Option` never escapes `providers/mod.rs`
  and there is exactly one `None`→identity dispatch point per slot. The
  non-inline-identity rule was **retired, not enforced** — an identity may now be
  `#[inline]` or a `pub use` because its address is never observed; that is the
  point of the change. Byte-identity: goldens `surface 0x7B8968FD90E04062` /
  `record 0xA53BD77F769D7FF4` pass unchanged, as does the `depth_to_water`
  materialized-plane agreement test (both arms still proven). New tests:
  `a_default_set_that_cannot_be_constant_folded_still_reports_no_slots` (rebuilds
  #32's exact conditions — separate crate, `black_box` behind `#[inline(never)]`
  — and now cannot fail, which is the argued outcome), `the_none_path_is_the_identity_function`,
  `explicitly_supplying_the_identity_function_still_counts_as_supplied`. One
  deliberate semantic shift, pinned by that last test: `Some(identity_fn)` reports
  the slot as **supplied**, because the manifest's question is "did an heir answer
  this?", not "does the answer equal the old one?" — only the first is decidable.
  **Nothing to ratify.** Carried: `grid.rs:230`'s docstring still names the
  now-split `tests/providers.rs`; `Slot::ALL` is still hand-maintained; the
  DECIDED text in ARCHITECTURE.md § *Provider seams* still says "a `Providers`
  struct of **plain fn pointers**" where the shape is now `Option<fn>` — a
  wording refinement inside a user-owned DECIDED block, so left for the user
  rather than edited by an agent; and the field-vs-method doc links in
  `biotic.rs` / `lithology.rs` (outside this write-set) are now ambiguous to
  rustdoc, which the ones inside `providers/**` were disambiguated with
  `field@`.

- 2026-07-22 — **The provider file stops being a mutex** (journal/0063;
  corrections #32; gates green — fmt/clippy/test all `--release`, with
  `cargo clean -p dc-worldgen --release` before each). **Pure refactor, zero
  behaviour change**, asked for by journal/0060's and 0061's own retrospectives:
  `providers.rs` had become the serialization point for concurrent seam work —
  four seams converted, thirty left in the inventory, and any two conversions
  collided on the same four regions of one file. Now
  `deeptime/providers/{mod,outcrop_at,wave_energy,parent_p,depth_to_water}.rs`,
  one file per slot (payload + identity + unit tests), with `mod.rs` holding
  only the `Providers` struct, its `Default`, and the new `Slot` enumeration —
  **grouped by owing system** (hydrology / ecology / materials / structural, the
  34-seam inventory's own buckets, `ecology` deliberately present and empty as
  the next insertion point), so two concurrent conversions insert at different
  points and git merges them. Filed by **who will answer, not who asks**:
  `parent_p` sits under *materials* though ecology consumes it, so reading down
  the file gives a map of who owes what. `tests/providers.rs` split the same way,
  with the byte-identity goldens **alone in `tests/providers_golden.rs`** — no
  conversion has any reason to open the file holding them. `is_identity()`
  reshaped as 0060 asked: `Providers::non_identity_slots() -> Vec<Slot>` plus
  `Slot::{ALL, name}` (the field name verbatim), because a manifest's
  frozen-content-set refusal needs *which* providers were resolved, not a bool;
  `is_identity()` survives as a one-line convenience. Still **no** registry,
  loader or selection channel. **A latent bug fell out of the code motion**
  (corrections #32): `identity_outcrop_at` was a `pub use` of the `#[inline]`
  `exposed_litho`, which rustc may instantiate per codegen unit, each with its
  own address — so `Providers::default().is_identity()` returned **false**. It
  had been passing on `main` only because the flatter file let the optimizer
  fold both sides of the address comparison; the test was green for a reason
  unrelated to what it asserted. Fixed with a plain wrapper (free — a provider
  is always called through a pointer and never inlined at its call site). New
  rule: **a slot's identity must be a plain, non-inline function in the slot's
  own module.** Byte-identity: goldens `surface 0x7B8968FD90E04062` /
  `record 0xA53BD77F769D7FF4` pass unchanged, as does the materialized-plane
  agreement test. **Nothing to ratify.** Carried: `grid.rs:230`'s docstring
  still names the now-split `tests/providers.rs` (outside this slice's
  write-set), and `Slot::ALL` is hand-maintained — pinned by a test rather than
  by a derive, the expensive half still correctly deferred.
- 2026-07-22 — **Two organic facies get the right axis: coal by burial,
  charcoal at all** (journal/0063; background implementation agent, worktree for
  the integrator; gates green — fmt/clippy/test all `--release`, 51 suites /
  531 passed / 0 failed, +3 new tests). Both of journal/0060's carried findings
  in one slice, because they are the same doctrine case from opposite sides.
  **Coal.** `promote_coal` tested the seam's own *thickness* (`>= 0.4 m`) where
  burial diagenesis is a function of **depth** — a statement about how long the
  swamp lasted standing in for a statement about what happened to it afterwards,
  while `CLASS_ORGANIC_COAL`'s own contract said *"the depth axis is the rank
  axis"* and the overburden was derivable from the record in one pass. Now
  `Σ` of the overlying units, threshold `COAL_BURIAL_M`. **Measured: coal
  collapses rather than moves** — 12 892 → 2 216 coal-bearing deep cells
  (4.52 % → 0.78 %), 19 008 → 3 888 units, 22 459 → 3 773 m, thickest seam
  15.74 → 14.01 m — because thickness and burial are close to *anti*-correlated
  here (a thick peat is one that sat at a quiet, low-aggradation surface). The
  **number** remains a stub (`stubs.md` § 14): there is **no geotherm** in this
  project, so this is burial depth and not a P/T path, and it cannot express coal
  **rank**; nor can it use Earth's 10²–10³ m, because 13 of 35 382 peat-derived
  units in the whole world lie under 50 m of section. 8 m is the ~90th percentile
  of *this* record's burial distribution. Heir: a geotherm.
  **Charcoal.** `deep_class` deliberately did not route the `Charcoal` facies,
  reasoning that a fire bed cannot survive voxel quantization so a charcoal
  member would be dead content — *and that reason expired on 2026-07-21*, eleven
  days before anyone re-read it. Since journal/0055 the quantization is unbiased
  **addressed stochastic rounding**, under which a 2.9 cm bed claims ~0.26 of an
  eighth and wins a whole one about a quarter of the times it is asked. Verified
  before building on it: 102 113 beds, mean **0.0289 m**, max **0.0400 m** (a
  hard structural cap — `SOIL_MAX × FIRE_CHAR_FRAC`), **zero** reaching one
  eighth alone. Now `CLASS_ORGANIC_CHARCOAL` + `MaterialId::CHARCOAL` +
  `dc:geo/charcoal`, marked `loose` so it rides the debris multiset like a placer
  grain rather than claiming a voxel's block identity. **Measured expression:
  0.394 % of recorded voxel spans carry a charcoal eighth; 0.0495 % of all
  allocated eighths.** Small, real, exactly the inclusion the retired comment
  called honest.
  **One deliberate mirror break, asserted by name:** `litho_of_tag` keeps reading
  a charcoal unit as its clastic host while `deep_class` expresses the carbon —
  a 3 cm lamina answers *"what fills these metres"* and has no answer to *"what
  rock resists this agent over a 460 m cell"*. Making it a `Litho` would hand a
  whole erosion cell the strength of its thinnest lamina.
  **Goldens moved, authorized:** both Medium worlds in `contents_contract.rs`
  (blocks + materials + mixture table; the Small control did not move, no deep
  record); `providers.rs::GOLDEN_RECORD` — and **`GOLDEN_SURFACE` did not**,
  which is the informative half, since promotion runs at finalize and cannot move
  a metre of ground. **Falsified:** corrections #32 (the expired charcoal excuse)
  and #33 (the coal axis, including this slice's own brief predicting coal would
  *move* when it collapses).
  Files: `deeptime/recorder.rs`, `deeptime/biotic.rs`, `deeptime/lithology.rs`,
  `geology.rs`, `fill.rs`, `pipeline.rs`, `collapse.rs`, `dc-core/materials/
  {mod,geology}.rs`, `dc-core/classify.rs`, `dc-client/{terrain_material.rs,
  shaders/terrain_fullbright.wgsl}` (palette-size lockstep guard only),
  `tools/gen_placeholder_textures.py` + the new `charcoal` pack,
  `examples/coal_charcoal_probe.rs` (new), tests in `biotic.rs`/`organic.rs`/
  `erodibility.rs`.
  **Also falsified in passing: corrections #34** — a `cargo clean -p` plus a
  concurrent sibling build resolved this worktree's `dc-worldgen` against a
  **sibling worktree's `dc-core`**, and `target/.agent-build.lock` was observed
  clobbered by another agent. CLAUDE.md § Gates now says to verify by the crate
  the log says it built (`Compiling` for build/test, `Checking` for clippy) and
  to re-read the mutex.

- 2026-07-22 — **S15 — coarse capacity against a lazily generated, evicting
  world** (journal/0062, docs/spikes/S15-results.md; background spike agent,
  worktree branch for the integrator; gates green — fmt/clippy/test all
  `--release`, 51 suites / 528 passed / 0 failed). The spike water.md dispatched
  because **S11's 415 ms capacity scan is an honest number for a world that does
  not exist**: it walked a fully-resident toy volume, and against the real
  generator a capacity scan is a generation storm that evicts the ground the
  player is digging. **Agent recommendation: GO.** Additive and standalone in
  `dc-worldgen/src/water/coarse.rs`; nothing in the production path calls it,
  deep time and the renderer untouched.
  **Group 1 — the coarse path ships.** A per-cell hypsometric summary (32-column
  cells = one chunk footprint, 4×4 sub-samples standing in for 1 024 columns, a
  64× compression) reproduces the exact voxel-walked level within **half a voxel
  for every body above 26 m²** — 781 of 791 real bodies measured against the
  production world, up to 212 000 m². Cost over those 791 queries: **coarse
  109 ms / 0 chunks generated** vs **exact 64 104 ms / 6 444 chunks**. The
  fallback the rule creates is bounded: the largest body still handed to the
  exact walk is 23 m², at **0.2–3.2 ms and ≤2 chunks**.
  **Group 2 — edits are deltas, and they do not drift.** 20 000 voxels dug
  through the audited `world/set_block` path, fed in as signed integer deltas by
  y: **drift 1.9 × 10⁻⁵ m**, the residual compression error held constant across
  the whole session, order-independent over shuffled batches.
  **Group 3 — the connectivity claim broke, on its second clause.**
  *"Connectivity only changes where someone edits"* is TRUE (terrain is a pure
  function of the seed). *"…and therefore the consequence is loaded"* is FALSE:
  removing **one plug voxel** from a 640-voxel tunnel joins a body **288 m
  beyond the edit**, and 972 of 1 215 far-apart basin floors are already one body
  through terrain nobody ever loaded. Two adversarial cases were NOT
  constructible and the reasons are dated: the *natural sill* needs caves (the
  world is a pure heightfield today), and the *absent neighbour* needs a store
  that can fail to answer (`block_at` materializes on demand, always).
  **Group 4 — eviction is invisible.** Capacity curve and derived water
  byte-identical across 3 120 evictions at an 8-chunk budget, with an edit
  pinned inside the body; 127 chunks regenerated, and the re-walk was *cheaper*
  than the cold first walk (153 ms vs 254 ms).
  **Falsified:** corrections #30 (the narrow flooded shaft is the mechanism's
  *best* case, not its worst — a dug void is an exact integer delta; error
  0.000000 m) and #31 (the connectivity inference above).
  Files: `dc-worldgen/src/water/coarse.rs`, `examples/water_coarse_spike.rs`,
  `tests/water_coarse.rs` (all new); `water/mod.rs` (module + re-exports);
  `dc-worldgen/Cargo.toml` (+`dc-api` as a **dev**-dependency — examples and
  tests only, no cycle, the whole point being to run against the real store).

- 2026-07-22 — **`depth_to_water`: the widest seam, and the identity path it
  never had** (journal/0061). The second conversion slice, one seam.
  `biotic.rs::step_cell` computed waterlogging from three magic numbers inline —
  climate moisture + `((80-surf)/80)*0.20` + `(area/300)*0.15` — where the
  question is *"is the water table at the surface here?"*. It is the
  highest-blast-radius seam in the 34-seam inventory: **four** thresholds read it
  (the peat-former waterlog gate, the decomposition drain factor, the fire
  dryness term, the `peat_site` hiatus-cap test), and it flows through organic
  facies → `geology::deep_class` → the world's surface material. Its heir is
  already **ratified**, not proposed: water.md DECIDED 2026-07-20, consequence 4
  — *"waterlogging becomes 'the water table is at or near the surface here', read
  from the field"*. Now `Providers::depth_to_water`, and it fixes the one gap the
  first slice left: this seam had **no identity path at all**, unlike
  `bio_resist`/`frost`/`wmult`. It has one now — `identity_depth_to_water` leaves
  the plane **empty**, and `providers::wet_at`'s empty-slice branch *is*
  `identity_wet_index`, the three-term proxy verbatim. Empty plane + identity
  accessor, the shape the four deep-sim flags already prove, so "provider absent"
  is not merely byte-identical but free (no allocation). **Granularity:
  pass-level, once per epoch**, materialized at `BioticSim::step` — the sharper
  version of 0060's rule: *granularity follows the **heir**, not the call site*.
  The call site is four per-cell thresholds in the hot loop and says
  "value-level"; the heir is a saturation field over the drainage-pinned lattice
  and says "plane"; the heir wins. `WaterPass` therefore hands the provider the
  drainage network (`recv`, `area`, `filled`) that a per-cell payload
  structurally cannot carry. Byte-identity: the pre-slice goldens
  `surface 0x7B8968FD90E04062` / `record 0xA53BD77F769D7FF4` (captured from
  `2434f37` for journal/0060) pass unchanged — **and** a new agreement test runs
  the whole production world down the *materialized-plane* path with a provider
  that fills the proxy, landing on the same two hashes, so both branches of the
  seam are proven and not just the empty one. No behaviour change: all four
  thresholds and all five coefficients are untouched. **Nothing to ratify.**
  Carried forward: the **units mismatch** — the identity answers a dimensionless
  0..1 index, a real water table answers metres below the surface, and every one
  of the four consumers thresholds the index. Journal/0061 § *what the heir must
  supply* states the contract the hydrology system has to meet. Ritual cost at
  Medium: **16.011 s → 16.045 s (+0.21 %)**, over two alternations of four runs
  each — the first alternation read +0.9 %, and a second pre-slice set moved the
  baseline 1.4 % with no code involved, so two alternations is now the minimum
  for this measurement.

- 2026-07-22 — **The provider seam: three sockets where constants were rules**
  (journal/0060; ARCHITECTURE.md § "A summary is not an authority" + § "The
  content set is frozen at world creation"). `deeptime::providers::Providers` —
  three plain `fn` pointers, same discipline as `pipeline::PassBody`
  (deterministic, no captured state, `Copy`), resolved once at world build and
  carried in `DeepConfig` along the proven `production_config_with` →
  `build_field_with` → `PregenCtx` → `Pregen::run_with` path. Each slot's doc
  names its **question**, its **heir**, and its **identity value**; the constant
  survives only as a registered identity, so it can no longer masquerade as the
  rule. Converted: **`outcrop_at`** (identity `exposed_litho` = top of the
  record; heir = the layer-cake/dip-fold term — the seam `lithology.rs` had
  already named in prose, now compile-checked, at all four sites that read
  exposed lithology), **`wave_energy`** (identity = the global
  `DeepConfig::wave_erosion`; heir = fetch from the S11 body graph × the zonal
  wind field — **previously unlisted anywhere**, now stubs.md § 13), and
  **`parent_p`** (identity = uniform `1.0`; heir = parent-material petrology;
  stubs.md § 8). The last is deliberately **pass-level** — a plane materialized
  once at `BioticSim::new`, never a call inside the epoch loop — which is how the
  rule *"a provider must never be called in a hot loop to answer a question that
  does not change inside that loop"* got stated rather than assumed.
  **Byte-identity is the acceptance test and it was proven the non-circular way:**
  FNV-1a fingerprints over every kept plane and every recorded unit, captured
  from pre-slice `main` (`2434f37`) *before* the slice existed and independently
  reproduced from a checked-out pre-slice worktree —
  `surface 0x7B8968FD90E04062` / `record 0xA53BD77F769D7FF4` — and reproduced
  exactly by the default-provider world (`tests/providers.rs`). Ritual cost at
  Medium: **15.93 s → 15.98 s (+0.3 %)**, inside noise. No behaviour change of
  any kind; **nothing to ratify.** Deliberately NOT built: a registry, a plugin
  loader, a declaration/validation pass, or a `DeepOverrides` selection channel —
  three seams is enough to design *for* and not enough to design *from*, and a
  selection channel with no selectors is the same defect in miniature. Two
  findings carried, unfixed by design: `promote_coal` promotes on seam
  *thickness* where burial diagenesis is a function of *depth* (**fixed
  2026-07-22, journal/0063**), and `P_FRESH` (the rejuvenation *rate*) is still
  global now that the pool it restores toward is a plane.

- 2026-07-21 — **Distribution-first expression: the record skins the world**
  (journal/0055; user-DECIDED design, materials.md § "integrate the column,
  then slice it"). The sieve was a quantization-**order** defect — `Σ round(tᵢ)`
  where honesty needs `round(Σ tᵢ)` — so metres now survive to the voxel
  boundary and quantize **once**, filling each voxel's eighths from the units
  overlapping its 0.9 m span by **addressed stochastic rounding** (unbiased;
  deterministic flooring is biased and always loses). **Sieve loss 75.8 % →
  0.2 %**; the 48.1 % of land cells that expressed *nothing* now express their
  record; the dune field's 379 units / 7.99 m go from **zero voxels to nine,
  all mixed**. The **surface is skinned by the record** (user: *"we don't have
  to have this problematic of deciding which material to skin the world with
  when the record already says"*) via the shared `surface_sample` kernel, so
  the horizon and the ground inherit it structurally — **integrator-added test
  asserts they agree on surface MATERIAL**, not only height, which nothing did
  before (the sibling test discards the block). Stubs **2 (where a record
  exists)**, **3's heir** and **12** retired; the veneer's thickness budget
  self-retired to `0.00` voxels *without surgery*, and the 8-voxel cap's
  14.3 % truncation is gone. Cost measured not assumed: 11 → 325 distinct
  mixtures (0.74 % of the S8 combinatorial cap), sidecar 0.330 → 0.621 B/m³
  (still 17.7× under raw dense), chunk gen +43 %, far field free, ritual and
  `DeepField` unchanged. The fill-contract's absent-contents exception shrank
  `{Air, Stone, Dirt}` → `{Air, Stone}`. Regression caught by an existing test
  and fixed in-slice: with the residue at zero the veneer *is* the fluvial fan,
  and whole-voxel rounding made **every placer in the world vanish**.
  **⚠ APPEARANCE: unratified, world-wide — see Observed.**

- 2026-07-21 — **HostWorld chunk eviction — the RAM march is flat**
  (journal/0051; fixes the 0050 diagnosis). `HostWorld.chunks` is now a
  bounded LRU over *generated-and-untouched* chunks (droppable — they
  re-derive byte-identically; the S11 "store only what the derivation cannot
  predict" doctrine) plus a **pinned** set of *edited* chunks, flagged in
  `set_block_raw`, the single audited voxel-writing path. Budget arrives as
  data from the client (`Authority::chunk_budget_for`, ~2 360 chunks at the
  boot scale, clamped [1 024, 16 384]) so dc-api stays headless.
  **Measured A/B on one binary** (`DC_CHUNK_BUDGET` huge = pre-fix
  behaviour), `DC_MEM_PROBE=1 --horizon 3`, fresh ±15 km jump every 1.8 s:
  **+27.4 MB/jump → 0.00 MB/jump** (flat over 147 jumps after warm-up;
  ~210 jumps / 9 min total, ±20 MB band). Store fills to cap and stays:
  `host_chunks` 2 129–2 351 against budget 2 360 while 99 752 chunks were
  evicted. Correctness: byte-identical regenerate proven over 3 seeds ×
  3 positions and by a budget-4-vs-budget-100k region-hash equality, plus a
  live MCP edit surviving 40 fresh-ground jumps
  (`crates/dc-api/tests/chunk_eviction.rs`). 0050's "~33 KB/chunk" corrected:
  `Block` is `repr(u16)`, so a chunk is **64 KB** and the fill rate is
  ~335 chunks/jump. **Pooling: measured and deliberately NOT built** — see
  Observed. No user-visible change (no appearance, feel, or frame-rate
  difference; mesh build rate unchanged across the A/B).

- 2026-07-21 — **The first guided tour — five stations, five verdicts**
  (journal/0049; the LIVE co-walk protocol's first run — user at every
  station, verdicts gating each move). Wind + frost magnitudes **RATIFIED
  as-built** (earth-processes.md addendum); wave **NULL CONFIRMED → retune
  Sequenced** (user: "more dramatic by default"; fetch-model heir recorded —
  S11 bodies × wind field). Tour's cross-cutting finding: mechanisms fine,
  legibility owed by the same three debts everywhere — carry-`H` (station 1's
  "always going to have topsoil" is its verbatim trace; **SHIPPED 2026-07-21,
  journal/0053 — though it made that station DEEPER, not barer: the station's
  13.06 m is `ΔH`, not remaining cover, corrections #26**), the veneer's
  ecology replacement, and the **forms/partials pass** (station 2's directive:
  sand = first loose material, partials-first emission — materials.md DECIDED).
  Station 4's frost story reads IN THE COLUMN (dug variety — a win). New
  symptom filed: texture smearing at `--horizon 3` after heavy teleports —
  the leak isn't horizon-6-exclusive.

- 2026-07-21 — **full_agents ON in production + the guided-tour map**
  (journal/0047, background agent; combined gates GREEN on fully-merged main —
  46 suites, 0 failed, covering this flip plus both texture merges). Wind +
  frost + wave run in every new world; the SEVEN MAGNITUDES ride at 0034
  defaults, UNRATIFIED — to be judged in the first LIVE co-walk (user present
  at every station, comment/confirm gating each move; the protocol upgrade
  over screenshot-walks). `examples/tour_map.rs` locates the stations on the
  client world: deflation basin 13.1 m (legible) · dune field 2.1 m (modest) ·
  loess margin 2.5 m with walkable desert edge (modest — the open question) ·
  periglacial summit 11.8 m stripped at 998 m (best station) · **wave coast
  0.68 m (NULL — the standing magnitude verdict unless the user's eye says
  otherwise)**. Two re-baselines, 0030-discipline (the coal-seam pick went
  degenerate under redistribution; world still grows 7647 seams >3 m,
  strongest diggable renders 19 voxels — no floor loosened). Cost: ritual
  +2.5 s; **resident +92.76 MB — see Observed**.

- 2026-07-21 — **Ore texture redo — substances, not portraits** (journal/0048;
  correcting journal/0045 same-day after the user's review: the first pass
  baked deposit portraits — host-with-flecks — where the ratified
  eighths/partial representation composes host+substance in the RENDERER
  (grade-is-eighths; a painted fleck forecloses grade). gold-quartz →
  **native-gold** (LabPBR metal, no emission), redbed-copper → **malachite**
  (dielectric botryoidal green), bog-iron regenerated as pure limonite;
  banded-ironstone/rock-salt kept (whole-voxel rock / already a substance).
  28 untouched packs byte-identical; deterministic double-run proven. Root
  cause filed as **ores.md R8**: the draft's member NAMES are deposit names,
  contradicting geology.md § Ore — rename to substances is a user call.

- 2026-07-21 — **Ore placeholder PBR packs — textures ahead of the registry**
  (journal/0045, background agent; asset/python-only; merged with the
  workspace gate deferred BY INTEGRATOR DECISION to the combined post-
  full_agents-flip merged-main run — to be confirmed there). Five
  deterministic 16×16 LabPBR packs beside the existing 29: gold-quartz
  (milky vein quartz, sparse warm flecks), bog-iron (limonitic nodular
  mottle), banded-ironstone (hematite/chert/steel stripe, wrap edge phased
  inside a band), redbed-copper (R5-option-a subtle malachite specks),
  rock-salt (LabPBR subsurface B=190 — reads faintly translucent). Tiling
  self-check green across all 34 packs / 68 tiling PNGs; **87/87 pre-existing
  PNGs sha256-identical**; gold-dust verified already packed. The new slugs
  are INERT until registry wiring day — the atlas loader iterates the
  registry, never the directory. The user judges the looks when the ores
  render (wiring day), not from files.

- 2026-07-21 — **Walk 0046 — the scarp and the plateau** (journal/0046, eight
  assets; the corrections-#25 debt paid on a STOCK production world — first
  walk needing no flags post-U8, post-climate-fix, `--horizon 6`). The
  probe-located steepest cell delivers real terrain: terraced flanks, a
  coastal scarp dropping 241 m into a dry sub-sea basin (bare stone floor,
  dirt shoreline stripe, green rim — the veneer's elevation banding drawn as
  a coastline), and distant climate zonation visible on the horizon. The
  crest re-shot at 6 km stays the 0040 prairie — the plateau is real AND the
  world is not flat. One new instrument lesson: looking DOWN a grade is the
  blind vantage (the fall-line frame kept as negative exhibit); slopes read
  up or across. Verdict for the recalibration: the steep world is already
  legible from the deep field alone; more noise (A/B) cannot shape the
  plateau, only more simulation (C) can.

- 2026-07-21 — **U8: tectonic history ON in production** (journal/0044,
  "the flip is pomp"; background agent; gates green on merged main, 46
  suites / 470 tests, 0 failed). Ratified by the user WITHOUT gating on the
  walk; the walk documents. Every new world runs the chaptered kinematic
  history; `DeepField` keeps drainage export, `exhum`/`t_crust`, and the
  chapter table (+20.1 MiB at Medium; ritual 15.2 s, 1.06×, inside S12's
  prediction). U7 amplitude rides at default 80 (corrections #23). Four
  re-baselines, each documented per the 0030 discipline — including
  `SEAM_TOLERANCE_VOXELS` 6→12 (measured max interior step rose to 7, a
  legitimate cliff; all 10 000 chunk-border crossings still ≤6, the actual
  seam invariant intact). NOTE for the roughness recalibration: S13's
  "3.0× headroom" was computed against the old tolerance — the binding
  constraint has moved and candidate A's seam-failure arithmetic needs
  re-checking against the new measured baseline. Worlds made before this
  flip are not reproducible under it.

- 2026-07-21 — **Climate registration fix** (journal/0043; gates green on
  merged main, 46 suites 0 failed). The S13 parity flag was a real bug — see
  the resolved Observed line. Every climate-keyed read (veneer, soil tier,
  strata formation context) now lands on the terrain it describes.

- 2026-07-21 — **S13 — where the roughness goes: 5 % of the budget reaches the
  ground** (journal/0041, `docs/spikes/S13-results.md`, corrections #24/#25;
  background agent; gates green on merged main, 46 suites 0 failed).
  Measurement-class, nothing flipped. Reproduced journal/0040's transect
  headlessly to the decimetre at both amplitudes (constant 1.0 m offset: the
  client pose sits one voxel above the surface it reports). **Decay hypothesis
  CONFIRMED to the factor** — `AMP_DECAY^L_DEEP = 0.55⁵ = 1/19.8`, 5.0 % of the
  scheduled roughness budget survives, at every site and every provenance
  (`rms|Δ| = 0.577 × amplitude` = 1/√3, the SD of the uniform draw). **The
  sharper mechanism the dispatch did not guess:** it is the decay *composed
  with* the `L_DEEP` override — levels 1–4 (7.4 km → 921 m wavelengths, i.e.
  **mountain shape**) are computed and then **discarded** when level 5 replaces
  elevation with the deep-time surface. **Bilinear hypothesis FALSIFIED**
  (corrections #24): the level-5 lattice reproduces the raw deep grid to 0.4 %
  on relief / 0.3 % on mean step — nothing is smoothed away below 460 m because
  the source holds nothing below 460 m. **Third mechanism, unnamed by either
  hypothesis:** the 0040 summit is a *genuine simulated plateau* (adjacent deep
  cells differ 0.29 m across a 10 km box), so fixing the decay fixes the
  100–500 m band and **cannot** make that plateau a range. Rivers contribute
  exactly zero. **Rejected by number before anyone built it:** gradient-derived
  self-scaling jitter — it drives summit roughness to ~zero and makes 0040's
  photograph strictly worse. Two bugs found in passing: `DeepField::deep_coords`
  integer-centring put the probe's first run 7.4 km off ground (round-trip
  assertion now ships), and corrections #21 shared-cache poisoning reproduced
  and cleared again.

- 2026-07-21 — **The far-field horizon is a knob** (journal/0042, background
  agent; gates green on merged main — 46 suites, 0 failed). `--horizon <km>`
  (0.2–64 km): the ring geometry moves from `const RING_EDGES_M`/`FAR_MAX_M` to
  a runtime `HorizonConfig` resource. **The default is proven unchanged** by
  exact-float assertion (`[112, 256, 512, 1024, 1200]`, near-cover 112 m, camera
  far 3000, fog 150/1100) and corroborated by reproducing journal/0023's 188
  tiles / ~21 MiB. Measured (not projected) cost sweep, worst case with no
  coverage cull: **1.2 km** 188 tiles / 20.7 MiB / 3.72 ms-frame; **3 km** 288 /
  31.0; **5 km** 540 / 57.1 / 4.02; **10 km** 1648 / 172.4 / 6.10 ms, fill time
  1.6 s → 13.7 s. All six FF2a/0024 predecessor properties re-verified (stepped
  voxel language, no cracks — plus a NEW 10 km no-sky-holes test, no buried
  sheet, no same-level seams, multidraw batching, budget-bounded meshing, the
  last now *asserted* to be horizon-independent). Two deviations, both required
  and both default-preserving: the **camera far plane** and the **lit-pass
  distance fog** now travel with the horizon — a fixed 1.1 km fog would have
  whited-out the very landform the wider horizon exists to show, which is the
  journal/0030 blind-instrument failure and fails *silently*. `--fullbright`
  still disables fog entirely (journal/0031 preserved), so the byte-identical
  pure-data control survives. Interior ring ladder deliberately NOT scaled
  (proportional scaling would put ~4300 L1 tiles at 10 km and blow the want-set
  scan to 349² per level per frame — rejected on arithmetic). **Legibility is
  better than 0023 projected**: coarsest step stays 14.4 m at every setting.
  **NEW LIMITS FOUND:** (a) the 4-level scheme's honest ceiling is **~10 km /
  172 MiB** — past that the answer is 0023's *add rings*, a 5th/6th LOD level,
  not a longer L4; (b) **haze, not geometry, is the practical limit** — at
  `--horizon 8` the outer third washes toward white and silhouette reading works
  to ~5–6 km, so whether the fog *curve* (not just its range) wants its own knob
  is a **user-owned visual call**, deliberately not made.

- 2026-07-21 — **First flagged walk: the amplitude call, answered "neither"**
  (journal/0040, corrections #23; the deep-config plumbing's first use). Two
  worlds, same seed, `--tectonics` on both, only `--amplitude` differing.
  The knob is *correct* — continental elevation +714…+1079 m, abyssal plain
  unmoved (−2492.1 → −2490.3 m, since orogenic thickening rightly does not
  drive ocean floor) — and *irrelevant at the scale relief is read*: 250 m
  sampling across the world's highest crest gives **7.2 m over 1.75 km at
  BOTH amplitudes, identical to the decimetre**, and the two ground
  screenshots are visually indistinguishable. Continental structure is
  excellent (912 → −3523 m margin-to-abyssal, ~4.5 km range); landform scale
  is absent (whole belt ~380 m over 40 km, ≈1 % grade; summit plateau flatter
  than the macro). **Method note:** the 1.2 km render horizon is blind to
  macro shape, so the walk used `pose_set {surface:true}` as a *numeric*
  instrument — surface height at any (x,z), horizon-independent — with the
  lit pass kept for what it can see. Corrections #18/#19's lesson again: pick
  the control that can see the question, even when it isn't a camera. Also
  fixed en route: the game MCP was **registered nowhere** (every prior walk
  connected ad hoc) — now a committed project `.mcp.json`.

- 2026-07-20 — **Deep-config flag plumbing** (journal/0039, background agent).
  The sealed gen path is open: a new `DeepOverrides { tectonic_history,
  full_agents, thickening_scale }` (each `Option`, `None` = production default)
  threads `production_config_with` → `build_field_with` → `PregenCtx` →
  `Pregen::run_with`, with `Pregen::run` now a `run_with(&Default)` wrapper so no
  existing `{ seed, extent }` call site changed. Empty overrides are proven
  byte-identical to the old path (config, `DeepField`, and `Pregen` seam). Four
  launch flags on dc-client: `--tectonics`, `--full-agents`, `--amplitude <n>`
  (only bites with `--tectonics`), `--extent <small|medium|large>`; bundled as a
  `GenOptions` resource so a key-2 scale switch rebuilds with them. **The
  amplitude / tectonic / full_agents walk is now unblocked** — a walker can boot
  a flagged world. Files: `deeptime/field.rs`, `deeptime/mod.rs`, `lib.rs`,
  `pregen/mod.rs`, `pipeline.rs`, dc-client `authority.rs`/`app.rs`/`main.rs`.

- 2026-07-20 — **Record-walk** (journal/0038): console v2, `--edges`, and
  circulation shot into the visual record on current main. Console-v2
  signature/hint/live-completion verified working (driven via OS keystroke
  injection — the console has no MCP door, only real KeyboardInput);
  `--edges` re-confirmed moiré-free at range; circulation surfaced the
  finding below (Observed / corrections #22). Six assets `0038-*`.

- 2026-07-20 — **Zonal circulation profile** (journal/0037, session-4
  background agent; gates green on merged main). The `wind_dx` sign bit is
  dead: C¹-continuous `zonal_wind` (sin² lobes, trades 1.0 / westerlies
  0.9 / polar 0.45, ~6°-wide calm belts at 30°/60°), Gaussian `subsidence`
  (0.75 @ 30° — the Hadley desert belt on FLAT terrain, proven by test;
  ITCZ kept wet; 60° calm-but-wet, the mechanism unity), latitude-shaped
  convective floor. Eolian deflation now scales by |wind| — dune fields
  fade in calm belts. Measured: mean |Δprecip| 0.082; 28–38° band −0.18
  (the new desert), mid-latitudes ~unchanged. UNFLAGGED by ratified
  decision — every new world's climate shifts; the walk photographs the
  30° desert (no mountain upwind) and the former 30° seam, lit pass. Two
  bonus finds: a latent half-cell bug in the coal test's voxel↔cell
  inverse (fixed); relocated settlements can over-constrain the S2
  pressure collapse — `history.rs` now skips (reject-don't-crash, the
  engine's own policy) instead of panicking, integrator-reviewed and
  approved; the skip is currently SILENT — a loud warning is owed per the
  degradation doctrine (loose end).

- 2026-07-20 — **Tectonic-history SPIKE — the architecture works**
  (S12-results, journal/0036, session-4 background agent; gates green on
  merged main with corrections-#21 eviction prophylaxis). `tectonic_history`
  implemented per the ratified tectonics.md: kinematic chapters (K=8),
  analytic bisector forcing, crustal columns, smoothed-load Airy isostasy,
  chapter-stamped recorder, drainage export. Headlines: **the 50 km
  gradation artifact is dead** — 20.9 km at default W=25, tracking W
  linearly with no cell term; byte-identical off + deterministic on (all
  fingerprint suites untouched); cost 16.2 s Medium/200 iters (1.13×),
  31 s at 400 — relief builds monotonically with iterations (953 m → 1.5 km
  Medium; 3.7 km Large/400); recorder ×3.2 at K=8, inside bound. Honest
  deviations recorded in S12 (two exact ledgers, Eulerian advection).
  **Pending user: U7 amplitude from walk renders (80 vs 160), U8 flip.**
  **New finding: exhumation is metre-scale at shipped erosion rates**, so
  exhumed cores/forelands are illegible at ANY amplitude — see Sequenced
  (erosion-supply calibration).

- 2026-07-20 — **Pore packability rule** (dc-core `packing.rs`,
  materials.md DECIDED entry is the record — deliberately no journal
  entry; gates green). `K_PORE=0.25` + `fits_in_pores` shared helper,
  7 falsifier tests, genesis exemption comment at the olivine member.

- 2026-07-20 — **Console v2 — arg discoverability + scrolling**
  (journal/0035; dispatched on the user's same-day field report "still
  unusable... do not know the shape of args"; gates green in worktree and
  re-run on merged main). All schema-generated, zero per-command code:
  persistent signature line once a command is recognized; per-arg hint at
  the caret (type, required, description, live example values); Tab lists
  param keys with descriptions and completes VALUES via the 0033
  `Completer` hook (`block=<TAB>` → real block names from the live world,
  read-only `Res<Authority>` — completers take `&HostWorld` accessors only,
  cannot tick or stall); `help <cmd>` renders a reference card + generated
  example invocation; PageUp/PageDown/wheel scrolling over 500 retained
  lines; held-movement-key restore on close. Deep array leaves still take
  the raw-JSON escape hatch (unchanged v1 limit). Appearance (hint layout,
  colors, phrasing) is dev-tool default — user restyles at will.

- 2026-07-20 — **full_agents: wind (agent #5) + frost/wave activation**
  (journal/0034, session-4 background agent; gates re-run green on merged
  main by integrator). The erosion roster completed except karst:
  `Agent::Eolian` with a cohesion-keyed resistance axis; frost as a
  temperature-gated weathering multiplier (freeze-thaw peaking near 0°C,
  honest gate derived from the same `air_temp_c` the biotic layer uses);
  littoral wave cutting at the current sea stand, mass-neutral. All behind
  `DeepConfig::full_agents`, OFF in production, byte-identical off — proven
  the strong way (flag ON with zero rates == flag off, bit for bit).
  Measured on a seeded Small world: arid cells deflate −104.3 m ΣH into
  71.4 m loess + 65.5 m dune deposits (ledger residual −0.000); periglacial
  band strips +668.5 m extra regolith with a −0.00 warm control; coastal
  cells retreat ~1.3 m/40 epochs (deliberately modest — magnitude is a
  user knob). New additive `DepTag` Eolian axis (defaulted → wire-safe).
  **NEEDS RATIFICATION (user): the production flip itself + the
  appearance-class magnitudes (wind rates, dune/loess split, frost gain,
  wave strength).**

- 2026-07-20 — **Registry `commands!` macro + `completions` hook**
  (journal/0033, session-4 background agent; gates green on final merged
  main — the run that first compiled the console against the generated
  registry). API decisions #6/#7 built: one macro table row per command
  emits the id const, the `Payload` variant, and the `CommandSpec`
  together, so a missing registry entry is now a rustc error, not a test
  artifact. `Payload` variant order verified identical by the integrator
  (postcard is positional — corrections #3); wire schemas byte-identical
  (the dc-mcp-dev session test asserts tool schema == spec schema, still
  green). `Completer::{Static, World}` hook populated with real sources:
  block names, live character names, content classes, postures, event
  kinds. Sole observable change: `registry()` order now follows wire order
  (cosmetic; nothing looks up by position). Follow-up owned by a future
  slice: wire the hook into the console so `block=<TAB>` completes.

- 2026-07-20 — **In-game dev console (T key)** (journal/0032, session-4
  background agent; gates green on merged main — fmt/clippy/test all
  `--release`, 42 suites 0 failed, console core 14 unit tests). The full
  dc-api surface in a game session: T opens, `help` renders the whole
  surface, Tab completes commands and dotted param paths, `key=value` args
  assemble into schema-typed JSON through each spec's `decode_json`,
  receipts arrive async at the tick boundary. **Everything is generated
  from `dc_api::schema::registry()`** — no per-command console code exists,
  so a command added to the registry appears with completion and help for
  free (API.md principle 5's third consumer, after MCP and WASM). The three
  client-shell tools ride along the way `mcp.rs` appends them. One-door
  dispatch through the same bridge/authority path as MCP (`spawn_servers`
  now always creates the bridge channel, so the console works without
  `--mcp`). v0 limits filed at merge: no value-level completion (the
  registry hook is in flight), deep array payloads take a raw-JSON escape
  hatch at the leaf, ~24-line scrollback. **Appearance/syntax await the
  user's own test drive** (panel look, `key=value` dotted-path syntax).

- 2026-07-20 — **`--edges` crease/silhouette diagnostic + fullbright fog
  fix** (journal/0031, session-4 background agent; gates green on merged
  main). Fullbright is no longer blind to shape when you ask it not to be:
  `--edges` adds a renderer-owned post pass (after the pack post stage,
  outside the frozen hook-format-0 contract) outlining depth and
  depth-reconstructed-normal discontinuities — crease/silhouette, never
  per-cube — faded 350→1400 m so the 3.5 km massif renders as clean nested
  contours with **no moiré** (verified by integrator eye on
  `0031-massif-fullbright-edges.png`). With `--edges` off the plugin
  builds nothing at all, so `--fullbright` alone stays byte-identical —
  the 0027 colour-in-colour-out control survives. And `--fullbright` now
  disables distance fog data-side (fog range pushed past the far plane in
  the `PostStage` uniform; sky-haze and lit-pass fog untouched), so
  landform-scale silhouette photography is finally possible. Resolves the
  three 0030 INSTRUMENT lines' proposed fixes.

- 2026-07-20 — **Erodibility coupling turned ON in production** (journal/0030;
  gates green — fmt/clippy/test all `--release`, 0 failed). The user ratified the
  appearance call ("flip it, i want to see"), so `production_config` now carries
  `erodibility: true` beside `biotic: true`. **Every world created from here on
  has a different shape; worlds made before today are not reproducible under this
  build.** No rate, contrast or clamp was touched — the amplitude question stays
  the user's. **The appraisal: the surface changed everywhere and improved
  nowhere.** Four exact vantages (summit silhouette, stripped granite upland, bare
  hillside, green lowland) re-shot before and after, lit and `--fullbright`. The
  *lit* pairs differ broadly — block-mean |Δ| 11.7–32.5 surviving 16×16 averaging,
  with ≈ zero *signed* mean — which is **face-orientation change**: the
  ground-level surface is substantially re-terraced. Macro landform is
  **unchanged**: the summit silhouette is identical and a 110-point 5 km lattice
  across the main massif moved only **−2 to +2 m** (mean −0.34 m), relief
  **2,615 → 2,614 m**, 38 of 110 samples unmoved, the walking surface down
  **exactly one voxel** at all three ground vantages. The two reconcile through
  **0.9 m quantization**: a sub-voxel elevation change re-rounds which faces point
  up, re-cutting every terrace on a slope while moving the landform by nothing.
  **No bench, ledge or resistant core attributable to lithology at any vantage** —
  the terraces that moved moved across single-rock-type ground too. This is what
  0029's own numbers predicted (modest until rates ×10, where a hard bed stood
  44.7 m proud): **the model is not the bottleneck, the amplitude is.**
  **Method correction — see corrections #18:** this entry first reported the
  fullbright pairs (0.08–0.93 % different) as the honest geometry comparison and
  blamed the lit difference on the sun moving. There is no day/night cycle. In
  `--fullbright` every face of a block is the same flat colour, so on
  single-material terrain it **cannot see geometry at all** —
  `0030-flank-before-fullbright.png` renders a whole terraced hillside as a
  featureless grey field. The right control for a *material* question (0027's
  coal) was the blind one for a *geometry* question. **Two tests re-baselined, both legitimate consequences, neither a
  bug:** `dc-client::authority::worldgen_surface_seating_never_embeds` probed the
  *centre* column under a spawn, but `true_surface_m` returns the max over the
  body's *footprint* — post-flip the origin column sits one voxel below all four
  neighbours, so the centre probe hit air while the seating was correct; it now
  probes the footprint corners too. `dc-worldgen::organic::the_measured_coal_seam_is_coal_a_player_can_dig`
  held a 24.03 m seam lithology-blind and holds **17.04 m** coupled (19 vox
  through collapse, was ~27) because differential weathering strips its soft
  cover faster; its three magnitude thresholds dropped 20→15 as a floor on "still
  a thick seam", the test's actual subject (biofacies routing → COAL class →
  diggable `Block::Coal`) unchanged. **The only world fingerprint deliberately
  loosened.** Files: `deeptime/field.rs` (the flip), `dc-client/src/authority.rs`
  and `dc-worldgen/tests/organic.rs` (the two re-baselines).

- 2026-07-20 — **Erodibility coupling — lithology-aware erosion** (journal/0029,
  background agent, worktree branch for the integrator; gates green —
  fmt/clippy/test all `--release`, 42 suites 0 failed). Closes cause 1 of the
  *dismal mountains* diagnosis: `erosion.rs` incised every cell with one global
  `k_bedrock`, so there was no differential erosion anywhere. Now erosion is
  modulated per cell per epoch by the resistance of the lithology outcropping
  there (`deeptime::lithology`). **Off by default and byte-identical when off**
  (`DeepConfig::erodibility`, same flip class as the S10 biotic layer;
  `production_config` inherits `false`, so every world today is unchanged);
  flipping it on changes `DeepField` — and terrain shape — for every new world.
  **The load-bearing design decision — resistance is agent-specific, never a
  single scalar.** A one-number "erodibility" cannot represent limestone, which
  is mechanically competent (cliffs) AND chemically soluble (caves) at once — the
  scalar forces a choice between them and forecloses karst. So the property sheet
  gained a `solubility` axis beside its mechanical `extraction_resistance`, and
  `LithoResistance` carries one resistance **per erosion agent** (abrasion /
  dissolution / frost-ice / wave), each derived from the property field that
  governs *that* agent. Only the mechanical (abrasion) agent is wired to live
  erosion — the dissolution/frost/wave axes are populated and dormant, so the § 8
  karst agent, the cryosphere, and littoral erosion each land by adding a term,
  not by a rewrite. `Agent` is exhaustively matched everywhere, so a fifth agent
  is a compile error until every site answers for it. **Where the contrast
  rides:** the coupling scales the fluvial terms *and* — the mechanism, found by
  measurement — the bedrock→regolith **weathering** phase, which is the
  rate-limiting step on hillslopes (diffusion is flux-limited by available
  regolith, so lowering collapses to the conversion rate). Coupling incision
  alone left the world statistically unchanged; coupling weathering is what
  differentiates it, and it is also the correct long-run home (in-place
  weathering is the sum over agents' attacks — dissolution adds a term there).
  **Landform evidence** (Medium, 460 m, biology on, ON vs OFF, same seed): along
  a 250-change transect, mudstone stands +3 to +12 m above carbonaceous-mudstone
  cell-by-cell; the sharpest contact **inverts a contour** — a hard cell 14.6 m
  *below* its soft neighbour with coupling off stands 1.1 m *above* it on. The
  **basement/shield prediction holds for free**: an empty record exposes igneous
  basement (hardest in the world), basement outcrops stand ~5,750 m proud, and
  coupling *widens* exposed basement (729 → 758 cells) as soft cover strips
  faster around hard cores. Aggregate contrast at the shipped erosion rate is
  modest (relief +2 m, steep +0.5 pp) because the whole landscape only removes a
  few metres against hundreds of metres of uplift — a **headroom** experiment
  (all rates ×10, relative rates fixed) scales it right up (relief +20 m, a hard
  bed **44.7 m** proud), proving the model waits on an amplitude decision (cause
  3), not a fix. **Composition order stated** (journal/0029): weathering
  `× (wmult × litho) × taper`, diffusion `× (1−resist) × litho` — biotic factor
  left, lithic right, load-bearing for byte-identity (f64 non-associativity).
  **Feedback bounded**: the self-reinforcing erode-soft→expose-hard loop is
  clamped to `[1/max, max]` (`erodibility_max`, default 5×); an 80-iter stress
  test at 4× contrast shows no runaway, no stall, nothing non-finite.
  **Cost +0.5 s** on the shipped path (13.89 → 14.39 s); test-suite ~503 → ~541 s
  (the new suite's own runs), no `--ignored` gating. Determinism intact:
  double-run byte-identical, scalar↔parallel byte-identical coupled (± biology),
  registration-order independence inherited (the coupling rides inside
  `dc:pass/deep-time`, adds no pass). Knobs: `erodibility`,
  `erodibility_contrast` (2.5), `erodibility_diffusion_contrast` (1.0),
  `erodibility_max` (5×). **NEEDS RATIFICATION** (below, § Sequenced). Files:
  `deeptime/lithology.rs` + `tests/erodibility.rs` + `examples/erodibility_probe.rs`
  (new); `materials/mod.rs` (`solubility` axis); `deeptime/{erosion,grid,mod}.rs`
  (the `expose` phase, config knobs, exports); `geology.rs` (`deep_class` made
  public); `docs/design/{geology,earth-processes}.md`.

- 2026-07-20 — **S11 — water locality + the free-water body graph** (journal/0028,
  docs/spikes/S11-results.md; background agent, worktree branch for the
  integrator; gates green — fmt/clippy/test all `--release`). The spike the
  water notebook dispatched to falsify **"persist bodies, derive voxels."** It
  did not falsify. **Agent recommendation: GO.** Additive and standalone in
  `dc-worldgen/src/water/` — nothing in the production path calls it, deep time
  untouched, no renderer work.
  **Q1 — bound water is local, comfortably.** A saturation relaxation over the
  S8 pore model (gather-from-frozen-snapshot in every phase, the S9b
  reformulation) with the water table *read* as the top of the saturated zone.
  Perturbing it with a dug seepage shaft gives a halo of **4–11 cells** at a
  bounded post-edit budget, and the **player-visible** halo (the integer water
  table moving a whole voxel) is **0–6 cells**. Decay is geometric —
  `d0=19.7 d2=2.49 d4=0.09 d6=0.001 d8=0` — with **none** of the isolated deep
  spikes S9 measured for fluvial erosion, because bound water has no advective
  term. **A ~12-cell derivation halo covers every case measured**, smaller than
  erosion's 16–24. The counter-intuitive result: **a sharper aquitard gives a
  SMALLER halo** (contrast 10 000 → 5 cells vs contrast 100 → 6–10), so the
  loose-vs-packed soil contrast flagged as the risk is the most local case, not
  the least. Conservation drift 2.4e-7; relaxation byte-identical on double-run.
  **Q2 — the graph does not grow with edits at all.** The structural finding:
  in a voxel world **connectivity does most of the graph's work** — two bodies
  in the same air component *are* one body, so links only exist between
  components and measured **0 or 1 in every scenario**. Through **1 624 edits
  and 508 084 dug voxels** (a maze, ~100 separate channels, a spiral shaft, a
  comb of trenches) the body count stayed at **1**: digging creates space, not
  water. Forcing the true ceiling — one body per component — tops out at
  **217 bodies / 4 400 B / 20.3 B per body**. Bodies are bounded by *components*,
  which track the derived coarse index, not the edit count.
  **All seven scenarios end in the right state**, including the two that broke
  the derive-everything model: the far end of a **1 145 m** dug channel reads wet
  in **45.9 ns** (one union-find `find`, no search at any radius), and standing at
  the bottom of a **1 073 m** chasm the level answers in **2.8 µs**. Unload, drop
  everything, reload from **39 persisted bytes**: derived water **byte-identical**.
  **OCEAN SCALE — the character distinction is real AND cheaper.** A finite
  7.3 M-voxel sea breached into a void half its volume **drops 13.18 m** (a
  shoreline retreating because someone dug a cellar). A level-pinned sea does not
  move — and because a pinned body's level comes from outside, **nothing ever
  needs its capacity curve**, so the hypsometry scan refuses to walk it:
  **415 ms → 0.0 ms**. Level-pinning is the *cheaper* implementation, and cheaper
  in proportion to the biggest body in the world.
  Two mechanisms worth remembering: **splits are cheap because we stopped being
  incremental** (union-find cannot un-union, so the coarse graph — thousands of
  nodes — is rebuilt wholesale per edit and a split costs what a merge costs,
  0.3 ms, while per-chunk voxel labelling stays incremental where the millions
  of voxels are); and caching chunk-face label pairs took the per-edit cost from
  a ship-blocking **19.56 ms to 0.249 ms (78×)** with byte-identical answers.
  Event-storm worst case **2.30 ms** (one event cascading a 1 000-body chain in
  2 fixpoint rounds). **Determinism holds by construction** — events are
  commutative monotone mutations plus one deterministic fixpoint solve — and two
  real violations were found by writing the adversarial case, not by the shuffle
  passing: a `Breach` and an `OutletBlocked` on the same link in one batch, and
  non-associative float addition across several `RegimeCross` on one body. Both
  fixed; **256 shuffled orders byte-identical**, double-run and edit-order
  identical too. Test-suite delta **+0.24 s** (12 new tests); no `--ignored`
  gating. **NEEDS RATIFICATION** (four calls, below in § Sequenced). Files:
  `water/{mod,vox,sat,conn,body}.rs`, `examples/water_spike.rs`,
  `tests/water.rs` (all new); `Cargo.toml` (+postcard), `lib.rs` (module).

- 2026-07-20 — **Organic materials + the biotic production flip** (journal/0026,
  background agent, worktree branch for the integrator; gates green —
  fmt/clippy/test all `--release`, 40 suites 0 failed). **The flip is live**:
  `production_config`'s `biotic` is ON, so every new world runs the S10 ecology
  and its `DeepField` carries organic facies. And the S10 gap is closed —
  `geology.rs::deep_class` now **consults the `Biofacies` axis first and lets it
  win where inhabited**, so an organic unit resolves to an organic content class
  instead of collapsing as whatever the transporting flow was doing. The measured
  seam argues for "wins" over "blends": the 24 m seam at world voxel
  (107338, 58787) is tagged `Sa/A/L` — subaerial, **arid**, **low** energy — which
  the old rule sent to clastic-fine, i.e. mudstone (coal swamps sit where
  drainage collects, not where rain falls; S10 design choice 8). **What a player
  can now dig**: that column reads 3 vox mudstone / 8 vox sandstone+conglomerate /
  **27 vox coal** / 7 vox conglomerate / **2 vox coal** / marine mudstone below —
  `Block::Coal` and `MaterialId::COAL` in the voxel contents, and coal's smash
  resistance is 2.2 against granite's 5.5 (real property sheet), so it yields to
  a tool that would barely scratch the cap. **Three materials added**, each a
  new class filled by one vanilla member (classes-as-contracts, so a pack
  diversifies without moving a seam): `dc:stratum/organic-coal` → coal,
  `dc:stratum/organic-peat` → peat, `dc:stratum/organic-soil` → carbonaceous
  mudstone (fills both `Soil` and `Retro`). **Three deliberately NOT added, on
  measurement**: a whole-grid census of what survives the 0.9 m voxel decided the
  roster — coal 89.6 % of units survive, Soil 48.5 %, Retro 6 % of units but 56 %
  of thickness, Peat 2.3 % (72 units world-wide — rare because thick *and*
  unburied is the definition of not-yet-coal), and **Charcoal 0 of 158 310**
  (mean bed ~3.5 cm against a 90 cm voxel). So **no charcoal material** — a
  charcoal band cannot exist in a voxel column and the material would be dead
  content; the honest form is an inclusion (pore/debris eighths, the placer
  pattern), filed to Sequenced with the measurement. **No coal rank ladder** —
  lignite→anthracite is a real burial-depth progression but the transitions live
  at 1–2 km and our deepest overburden is ~100 m, so a rank window would be
  decoration on an axis the data never visits; the class documents that its
  **depth axis is the rank axis** for the day the record carries kilometres.
  **No distinct retrogressive material** — retrogression is a phosphorus fact
  about a community, and the property sheet has no nutrient axis. Invariants
  intact: class-share unchanged (tested with a second coal member),
  registration-order independence re-proven roster-agnostically, determinism
  untouched. **Ritual measured 13.79 s** with biology on — see **corrections
  #12**: the ratified 25 s was the spike harness's *scalar* path; production takes
  S9b's byte-identical *parallel* path, so biology's real marginal cost is
  **+2.6 s**, not +10 s. Test-suite 381.9 s → 503 s (+121 s, of which ~106 s is
  the flip itself putting a ~14 s ritual behind every world-level suite and ~15 s
  is the new proof); no `--ignored` gating used. **NEEDS RATIFICATION**
  (appearance): three new rock colours enter cut faces, and carbonaceous mudstone
  is now the second most abundant facies in the world (185 km of surviving
  thickness) — the user should eyeball a cut face before this is settled. Files:
  `materials/mod.rs`, `materials/geology.rs`, `voxel.rs`, `worldgen/geology.rs`
  (the routing), `pipeline.rs`, `collapse.rs`, `deeptime/field.rs` (the flip),
  `dc-api/host.rs`, `meshing.rs`/`terrain_material.rs`/`terrain_fullbright.wgsl`
  (atlas 26 → 29), `gen_placeholder_textures.py` + 3 packs, `tests/organic.rs` +
  `examples/organic_probe.rs` (new).
  **PHOTOGRAPHED 2026-07-20 (journal/0027, photo walk, no code changed)** — the
  appearance question above now has images. Two of three read fine, one does not.
  **Carbonaceous mudstone reads well**: an ordinary 988 m hillside cut
  (`0027-carbonaceous-mudstone-ordinary-lit.png`) puts 3 voxels of it as the
  thickest unit in the soil profile, and it is a distinct chocolate brown against
  mudstone's red-brown and basalt's blue-black — the profile is *more* legible,
  not less. **Coal renders as pure black** with zero legibility, including on a
  fully sunlit up-facing bench floor (`0027-coal-seam-cut-lit.png`) — filed to
  Observed; the fullbright control proves the data is fine. **The world is not
  darker** (`0027-vista-lit.png`): nothing organic reaches the surface, so a wide
  view is unchanged green. **Peat was found** despite 0026's expectation —
  world voxel (-22983, 24546) on the client's seed carries 16 voxels of peat over
  1 of coal over carbonaceous mudstone, all three new materials in one section
  (`0027-peat-coal-mudstone-section-lit.png`). Note the sites are re-derived on
  **seed 1337** — see the Observed item on the client's fixed seed.

- 2026-07-20 — **S10 — the biotic layer on the deep-time A-tier** (journal/0025,
  docs/spikes/S10-results.md; background agent, worktree branch for the
  integrator; gates green — fmt/clippy/test all `--release`, 39 suites 0 failed).
  ecology.md § 3's **community vector + the six processes** implemented on the
  3e-1 A tier, additive in dc-worldgen and **off by default**
  (`DeepConfig::biotic`; with it off every path is byte-identical to pre-S10 —
  the modifier planes stay empty and read their identity values, and every unit
  tags `Biofacies::Mineral`). All six processes are real, none stubbed:
  suitability (`min` over tolerances — Liebig), dispersal (bounded neighbour
  kernel), competition (finite capacity, incumbency-weighted), nutrient cycling
  (the Walker & Syers curve **emerges**), niche construction, disturbance (fire
  real, flood a succession reset whose signature is the mineral overbank band
  erosion already writes). The **lagged coupling** ecology.md mandates is the run
  loop: `erosion.step` consumes last epoch's biotic modifiers, then `biotic.step`
  reads the fresh terrain and writes the next epoch's — which is what keeps the
  biology↔erosion cycle out of the pass graph. No new pregen pass: biology rides
  inside `dc:pass/deep-time` (already the creator of `DeepStrata`, already read by
  `dc:pass/clastic-deposition`), so registration-order independence is inherited.
  **All four target signals present and legible** (A tier, seed `0x0D5E_ED57_2026`):
  **coal** 1 133 cols / thickest seam **24.03 m**; **paleosols** 22.9 % of columns
  (the best is a genuine cyclothem — soils and peats alternating with marine
  bands, each carrying its at-deposition climate tag); **charcoal** 20.2 % (one
  upland column carries 19 fire beds, all arid-tagged); **retrogression** 23.5 %
  (an ancient 1148 m surface, soil at cap, available P 0.0097, rock-P drawn to
  0.44 — a Walker & Syers chronosequence nobody scripted, with the *geography*
  right: erosion rejuvenates slopes and floodplains, only untouched surfaces
  starve). **Cost: the ritual 15.17 s → 25.19 s (1.66×), and the world KEEPS only
  +6 MiB** — the +68.6 MiB peak is transient working set dropped when the run
  ends. The curve is **linear in cells** and the ratio *falls* as grids grow
  (1.80 → 1.66): the biotic step is a flat per-cell pass with no heap and no
  global dependency chain, while erosion carries the flood's `n log n`. Because
  `DEEP_MAX_WIDTH` already caps the deep grid, **+10 s is the same at Large as at
  Medium**. Determinism intact including scalar↔parallel byte-identity; biotic
  carbon is tracked as a genuine external mass input (`Δ(ΣR+ΣH) == uplift +
  biotic`). Two mechanisms worth remembering: **soil is a pedogenic OVERPRINT,
  not a deposited layer** (retag + thicken + merge down — this alone took the
  record from 665 k units back to 71 k, within 0.2 % of the biology-free run),
  and **soil horizons require a depositional hiatus** (correct pedology *and* the
  cost control, the same mechanism). Test-suite delta +16 s; no `--ignored`
  gating needed. **Agent recommendation: GO** — flip `production_config`'s
  `biotic` and sequence the collapse-tier organic materials immediately behind
  it. **The honest gap: the record contains coal, the world does not** — an
  organic unit still collapses as ordinary clastic (`deep_class` reads only
  env/energy, and there is no coal material), so a player cannot yet mine that
  24 m seam. **NEEDS RATIFICATION** (below, § Sequenced). Files: `biotic.rs`
  (new), `recorder.rs` (`Biofacies` + `overprint_top` + signal queries),
  `grid.rs`/`erosion.rs`/`mod.rs` (modifier planes + lagged step order),
  `examples/biotic_spike.rs` + `tests/biotic.rs` (new).

- 2026-07-20 — **Far-seam fix cycle: uniform-per-level depth push** (journal/0024,
  background agent; worktree branch for the integrator; gates green — fmt/clippy/test
  all `--release`; walk-verified lit + fullbright). The anti-z-fight push in
  `farmesh.rs` moved each far tile/chunk along its *own* center→viewer direction,
  so adjacent same-level tiles shifted along slightly different directions and
  reopened the (mesh-space-watertight) shared edge as a 0.08–0.9 m world-space slot
  — a see-through bright seam once the far field was a hollow top sheet
  (corrections #11). Replaced with **one shared push vector per LOD level per
  frame** along the camera-forward axis (magnitude unchanged, `DEPTH_PUSH_FRAC ×
  coarse voxel`): identical rigid motion for every tile of a level closes
  same-level seams by construction, per-level magnitude still separates overlapping
  rings, and it stays a true world-space depth offset (corrections #1, never a
  bias). Both far paths fixed (S1 `far_transform_translation` + FF2a
  `far_tile_translation`, via a shared `level_depth_push`). Transform-only — one
  shared material / standard `Mesh` untouched, so the FF2a multidraw batching is
  unaffected. **NEEDS RATIFICATION**: none (transform-only; no appearance change
  beyond removing the defect) — **user-confirmed fixed 2026-07-20** ("seam fix
  good"). File: `farmesh.rs` (both translation fns + shared
  push helper + module docs + world-space seam test).

- 2026-07-19 — **FF2a — the voxel-language far field** (journal/0023, background
  agent; worktree branch for the integrator; gates green — fmt/clippy/test all
  `--release`; walk-verified live, RTX 3070 / Vulkan). Replaces journal/0022's
  smooth-TIN far tiles with **voxel-stepped columns** (visuals.md § distance
  speaks the voxel language): each coarse column's summary height is FLOOR-
  quantized to the level's coarse-voxel step, meshed as stepped prisms — greedy-
  merged top faces + exposed side faces between neighbour columns of differing
  height. **Step 0 (empirical, DECIDED-substrate check):** Bevy 0.19's GPU-driven
  multidraw path **engages fully** for our custom `TerrainMaterial` with custom
  vertex attributes — live probe read `mode=Culling`, `Opaque3d batches=74 sets=1`
  (all near + far terrain draws, one shared material, merged into a *single*
  multidraw set); Vulkan backend; custom attributes only choose the allocator
  slab, they don't disqualify batching. Nothing bespoke needed. **Three wins fall
  out of the voxelization:** (a) *near/far parity, no sink* — flooring makes the
  far top ≤ the near surface by construction, so the opaque near terrain wins the
  overlap band and journal/0022's half-voxel sink is deleted; a stride-aligned
  column matches the near voxel top exactly (quantized-exact). (b) *crack class
  cured inherently* — a step's side face is emitted once, by the taller column
  only, and a tile samples its neighbours' shared boundary columns, so same-level
  seams are watertight with no skirt (grazing-angle shot: no pixel cracks);
  ring-to-ring and near/far edges get a modest 2-voxel skirt ("skirt only what
  remains"). (c) *no buried sheet* — far columns the near field covers are CULLED
  (`near_covers`, altitude-aware 3-D distance < 112 m), not lapped underneath, so
  digging never exposes a phantom floor (dig test: only real near geology; a
  fully-covered tile meshes to empty). The per-column payload is an extensible
  `ColumnSpan` (top + block today, persistence-shaped POD, FF2b extends to a span
  stack); the tile mesher is a **pure function** of (spans + coverage mask + ring
  flags) with the impure derivation in the streamer, so filed async far-meshing
  and edit-driven re-derivation stay drop-in (coordinator amendments). Perf:
  ~1.87 ms/tile derive+mesh, budgeted 2 tiles/frame, no hitch; greedy merging
  makes stepped tiles *cheaper* than the smooth sheet (worst-case ~698 vs fixed
  2048 tris/tile). Scale checkpoint (dev measurement, radius unchanged): current
  1.2 km field 188 tiles / ~21.5 MiB; projected ~10 km ≈ 376 tiles / ~43 MiB,
  per-frame meshing stays budget-bounded (~3.7 ms) regardless of radius. Keys 3/4
  keep the untouched S1 far mesh. **RATIFIED 2026-07-20 (user, from the
  0023/0024 images)**: the stepped-horizon look is approved ("fine for now"),
  the 112 m coverage inset is good; the skirt depth (2 coarse voxels) rides
  as-built (no bandaid). Follow-up filed to Sequenced: user wants **knobs to
  adjust the far-field ranges**. Files: `farmesh.rs` (stepped mesher, `ColumnSpan`,
  `quantize_top`, `near_covers`, coverage-refresh streamer), `app.rs`
  (`GpuProbePlugin`, env-gated).

- 2026-07-19 — **The far-field horizon — the worldgen world gets a far field**
  (journal/0022, background agent; merged `--no-ff`, gates re-run green on
  merged main, walk 17 photographs below). Under the worldgen authority
  the far LOD rings drew the legacy S1 terrain (~8 m) while the real world sat
  ~1000 m up: a phantom old world below and — worse — **no horizon at all**
  beyond the load radius (journal/0018 § the empty horizon). Now the far field is
  the authority's **own** surface, sampled coarsely: a new
  `WorldGenerator::coarse_surface` runs the per-column collapse kernel (elevation
  lattice + river carving + surface rule — factored out of `column()` so near and
  far are provably one function) for a single column, O(pyramid depth), memoized,
  **no full-res chunk**. The lattice already folds the deep-time `DeepField`
  surface in at the locale level (journal/0015), and runs into the wilds, so the
  summary needs no second source and never dissolves into empty sky. The far mesh
  is now a **2-D annulus of heightfield tiles** (top surface only, 2048 tris/tile
  vs the old shell's ~⅔-sealed-cave triangles), gated on the active authority:
  key 2 → the worldgen heightfield (`farmesh::stream_far_surface`), keys 3/4 →
  the untouched S1 far mesh. **Near/far boundary height mismatch: 0 voxels** at
  coinciding corners (height is climate-independent, so a far sample equals its
  near column exactly); residual is only the sub-coarse relief dropped between
  corners, dressed by a half-coarse-voxel downward sink + depth push + S4 haze.
  Perf: world-create unchanged (streams post-spawn); derivation **1.377 µs/column
  → a full 1.2 km 4-ring field in ~0.22 s**, budgeted 2 tiles/frame, no hitch.
  Tripwire: the worldgen summary path never touches `TerrainGen` (S1 authority
  returns `None`). **NEEDS RATIFICATION**: none new — the heightfield-vs-shell
  far-field shape follows the journal/0017 summary-pyramid direction the user
  already promoted; the aesthetic sink/push/haze constants ride as-built (no
  bandaid). Files: `collapse.rs` (`coarse_surface`, `surface_sample`),
  `authority.rs` (`far_field_is_worldgen`, `worldgen_coarse_surface` + tripwire),
  `farmesh.rs` (heightfield path), `meshing.rs` (`face_color` opened),
  `app.rs` (wiring).

- 2026-07-19 — **PBR-1: the real material renderer** (journal/0019, merged
  `c49566f`, gates green on merged main; **walk 14 done** — which caught a
  ±Z-face NaN in the shader's tangent frame within three frames, fixed in
  integration, journal/0019 § walk 14). The interim vertex-color dither/mosaic
  (journal/0010) is replaced by a **custom Forward+ LabPBR material**: three
  `texture_2d_array`s (basecolor / normal+AO / specular), layer index = material
  id, blended per fragment by height/AO contrast (heightlerp). **Mixed faces
  return to single quads** — the mosaic's 16× geometry is gone
  (196 608 → 12 288 tris on a fully-mixed 32³ chunk); constituents ride as
  per-vertex splat attributes (`Uint32x4` layers + `Float32x4` weights), top-N
  chosen by the world-anchored hash (the >N path is test-only headroom).
  **Uniform-contents voxels now sample their MATERIAL pack, not their block's**
  — this **kills the walk-10 "member identity is render-invisible" item**
  (siltstone renders differently from mudstone). The atlas is widened past the
  material count with four **block-only** layers (grass/dirt/stone/wood) so a
  *single* material renders the whole lit world — near geology, uniform strata,
  the far LOD rings, and the legacy S1 terrain — with no seam. `--fullbright`
  path unchanged: the same mesh carries both vertex color and splat data, and
  the streamer picks the unlit `StandardMaterial` under the flag — but walk 14
  found mixed-face *appearance* changed (speckle loss, Observed below). Directional sun + hemispherical ambient only (PBR-2 owns shadows,
  point lights, tonemap/HDR, POM, water; never GI). Placeholder packs regenerated
  21 → 26 to cover the full `MaterialId` registry (the 3d roster widening:
  siltstone/conglomerate/diorite/andesite/olivine got packs). **NEEDS
  RATIFICATION**: `SPLAT_N = 4` (the per-face material cap, like 0010's 4×4 dither
  flag) and the sun/hemi-ambient calibration constants (aesthetic, user-owned —
  the walk judges them). Files: `terrain_material.rs`, `shaders/terrain.wgsl`
  (new); `meshing.rs` (splat rewrite); `app.rs`/`streaming.rs`/`farmesh.rs`/
  `authority.rs` (material wiring); `tools/gen_placeholder_textures.py` + the 5
  new packs + manifest.

- 2026-07-19 — S1-fallback sweep (journal/0017, merged `--no-ff` to main,
  gates re-run on merged main by the integrator): closed the defect class
  journal/0016 named. The client now
  has ONE world-answer surface for solidity — `Authority::is_solid_voxel`
  (lazily generating the hosted world, edits included) — and the six near-field
  consumers that answered from the legacy S1 `TerrainGen` on a ChunkMap miss now
  route through it: player collision, character foot-IK grounding, crosshair
  edit raycast, physics collider tiles, and mesh-border culling
  (`streaming.rs` + `remesh_dirty`). Guard: `ChunkMap::is_solid` (the fallback
  method) deleted; the ambient `Terrain` resource deleted (no system can
  `Res<Terrain>` a wrong world); tripwire test
  `empty_cache_solidity_paths_read_the_worldgen_authority` fails on old main and
  catches all six sites; doctrine drafted in ARCHITECTURE.md (**NEEDS
  RATIFICATION**). Meshing measured *faster* through the authority (0.93 vs 1.17
  ms/chunk — generate-once-and-memoize beats per-voxel S1 noise). Keys 3/4 (S1
  authority) unchanged; all 38 suites green. **Far mesh left on S1** (residue
  below) — sourcing coarse far rings from worldgen is renderer/storage-scale
  work, not cheap. **Walk-13 verified live** (journal/0018): unstreamed-edge
  collision (mid-air character spawn → landing at a never-streamed column),
  authoritative `eye_in_solid`, edits-included `surface:true` seating, and
  the first deep-time cut-face photographs (0018 assets — mudstone/basalt/
  olivine-speckled granite at outcrop scale).

- 2026-07-19 — Body staircase step 3 (journal/0014, walk 11, merge
  `f0e9f2d`): trunk-follows-travel / head-follows-look via the neck
  (walk-8 orientation gap closed, photographically verified thanks to
  the new v0 brow face cue); two-bone leg IK with stepped output and
  half-voxel offset cap; parametric crouch split exactly on the firewall
  (`dc:character/set_posture`, 0.6× collider, stand-up guard,
  PostureBlocked receipt; procedural pose cosmetic-side); posture replay
  bit-identity proven.

- 2026-07-19 — S9b parallelism spike (journal/0013, merge `9910c45`):
  the determinism tax measured — flood has no byte-identical parallel
  form (98.5% serial floor at B); deterministic-parallel phases 1.2×
  whole-step, bandwidth-saturated by 8 threads; **A+C confirmed**,
  corrections #9 (S9's 2-minute flip condition falsified on ≤12-thread
  hardware; reopenable via deeptime_par --full-b elsewhere). Scatter→
  gather diffusion reformulation, byte-identity proven scalar-vs-parallel.

- 2026-07-19 — S9 deep-time spike (journal/0012, merge `3fd00bb`): the
  spike era reopens and pays off — two-plane erosion + measurement-tagged
  strata recorder + orographic march, additive in dc-worldgen. A/B/C
  measured (A 14 s; B ~63 min/~3 GiB scalar — corrections #8; C region
  refinement 3.6 s); decay length 21 cells (hillslope), drainage the sole
  advect; 500 m columns already tell true stories. Verdict: A+C unless
  parallelism flips B (S9b decides). Integration fought and won a
  stale-worktree env!-path bomb in the parity tests (corrections #7).

- 2026-07-18 — Repo, architecture, CI (3-OS), five-crate workspace.
- 2026-07-18 — S1 voxel scale (N=2 decided) · S2 constraint ledger (GO) ·
  S3 chunk format v1 + LOD + far mesh · S4 Forward+ + shader packs ·
  S5 dc-api parity (wasm/MCP/native) · S7 worldgen pregen + lazy pyramid +
  year-zero handoff · S8 materials storage (GO, free-form mixtures).
- 2026-07-18 — Rendering fixes from walk 2: spawn-frame flash, LOD z-fight.
- 2026-07-18 — S6 physics bubble: dc-physics (rapier3d 0.34 direct,
  enhanced-determinism), 4³-voxel collider tiles with set-difference refresh,
  bit-identical replay, detach→settle→reattach via the 24 integer lattice
  rotations, G-key client demo. **Spike era closed: all eight spikes shipped.**
- 2026-07-18 — Client through dc-api + observability harness (journal/0002):
  HostWorld embedded as the client's edit authority (pluggable generator over
  the S1 TerrainGen; ChunkMap demoted to receipt-driven cache); LMB/RMB edits
  as player-class `dc:world/set_block` commands (dc-core DDA raycast,
  crosshair + target gizmo); edit→remesh dirty sets + S6 collider-tile
  invalidation wired; in-client MCP over streamable HTTP :7777 (registry-
  generated tools shared with dc-mcp-dev + client_screenshot to
  journal/assets + player pose get/set); direct/MCP-layer/HTTP-wire edit
  parity proven headless.

- 2026-07-18 — First agent self-walk (journal/0003): MCP session against the
  running game — pose/scan/edit/screenshot loop proven; six findings filed
  to Observed. The practice is established.

- 2026-07-18 — Walk-3 corrections + walker proprioception (journal/0004,
  `0ba292f`): three of four walk-3 "renderer defects" were camera-inside-
  block misdiagnoses (corrections.md #3); shipped eye_in_solid, surface-
  clamped teleport, pitch docs, open-ground spawn; MCP edit pipeline
  photographically verified.

- 2026-07-19 — 3d: geology post-v1 slice (journal/0011, walk 10, merge
  `ba666e6`): igneous fitness weather-blind (per-class contracts —
  corrections #6: the seam's real quantizer was the chunk-column collapse
  unit, not cell-stepped climate); member-contact boundary dither
  (data-proven wandering contacts); two-layer class-satisfiability
  enforcement with named culprits; olivine pore-partial inclusions
  (photographed, assets 0011-*); roster proof 17→22 materials, 10 vanilla
  members across 6 classes.

- 2026-07-19 — 3c-2: material-tier geology visible (journal/0010, walk 9,
  merge `8c9f67f`): registry albedos on all 17 materials; sidecar channel
  through the seam (order-dependent MixtureTable ids resolved to a
  render-only ContentsGrid at the generator boundary — no intern id
  crosses the lock); 4×4 world-anchored deterministic dither on mixed
  faces (subtle ore with zero ore-specific code); partial-height loose
  rendering built-but-dormant. First mixture photographs from a placer
  cut 60 km out (assets 0010-*); `examples/river_cells.rs` dev tool born
  of the hunt.

- 2026-07-19 — Body staircase steps 1–2 (journal/0009, walk 8, merge
  `dd9b802`): body plans + anim clips as namespace-owned registry data
  (fourth roles-as-contracts instance; clips standalone-before-plan, flagged
  in API.md); the 11-segment `dc:body/biped` replacing the two-cuboid
  companion; stepped 12 fps sampler (32-step rotations, 5 mm bob — the bob
  quantization also fixed a ULP loop-wrap divergence), crossfade idle↔walk,
  verb→anim-slot contract enforced at define time. Walk-photographed
  standing and mid-stride (assets 0009-*).

- 2026-07-19 — Placeholder LabPBR texture packs (merge `78e9d95`, asset-
  only): 21 deterministic 16×16 three-texture sets generated from the
  property sheets; seamless tiling verified after user query and now
  asserted by the generator's self-check (`ea7251d`).

- 2026-07-19 — 3c-1: geology walkable at block tier (journal/0008, walk 7,
  merge `ee639f9`): dc-worldgen through the client seam (PregenSource
  Arc-opening, Rc→Arc caches, one Mutex two worlds — seam-level order-
  independence proven); Mudstone/Sandstone/Granite/Basalt blocks; N=2 boot
  restored (scale keys ratified: 2 = worldgen, 3/4 = legacy S1 dev
  affordance); surface machinery authority-aware; freeze-on-disconnect
  (Drop-fires-once → zero-intent command on the receipted rail), proven
  live mid-stride in walk 7. First geology photographs: quarry cut showing
  granite → basalt → mudstone → soil (assets 0008-*).

- 2026-07-19 — Geology v1 backbone (journal/0007, merge `1df2666`): content-
  class registry (classes-as-contracts; the Payload opening as open keys
  over a closed value vocabulary — NEEDS RATIFICATION with the class-sheet
  fields and pass vocabulary, marked in API.md); pass graph with
  creator/modifier/reader semantics replacing `Pregen::run` (output-
  preserving, S7 byte-identity unchanged); strata recording with climate-
  at-deposition tags; clastic/igneous/placer passes (placer from property-
  derived settle energies — gold-dust lands mid-gravel with zero
  ore-specific code); registration-order independence proven as fingerprint
  equality; perf cost is noise (cold chunk 0.711 ms mean post-geology).

- 2026-07-19 — Surface-truth fix + attach guard (journal/0006, walk 6): the
  walks-3–5 "under-report" diagnosed and closed — analytic field vs its own
  voxelization (½-voxel top-face offset) compounded by footprint-over-slope
  (14.49 m worst; 61% of columns would embed a body). `true_surface_m`
  (footprint-max per-column voxel scan, edits included) now seats spawn,
  `surface:true` teleport, and character attach; embed-guarded attach with
  `obstructed` receipt + opt-in surface-snap; photographically verified at
  the measured worst case. Corrections #5 (the "missing octave" story).

- 2026-07-19 — Bodies/sockets design doc (docs/design/bodies.md), Sequenced
  3b: body plans as registry contracts (anims bind to the plan; fork
  inherits by retained joints; verb→anim-slot validated at define time),
  the animation-is-cosmetic determinism firewall (sim sees parametric
  posture states only), IK as retargeting glue, 12 fps stepped-animation
  aesthetic, clothing as segment-copy shells — all ratified 2026-07-19;
  sockets/transmog mechanics remain PROPOSED.

- 2026-07-19 — Character MCP surface (journal/0005): the second surface from
  API.md § Characters, embodied sessions on :7778. Character primitive in
  dc-api (named body, host-tick swept-AABB stepping, controller verbs as
  commands, diegetic pose/raycast/surroundings senses, all schema-
  registered); `Grant::CharacterControl` with attenuate-to-one-character
  sessions (cage proven at capability/host/HTTP layers; dev surface keeps
  full reach incl. any-character control); two-cuboid companion rendering;
  bit-identical scripted-session replay. Walk: companion attached, driven,
  sensed, photographed standing/walking/jumping (assets 0005-*) — and it
  walked off a cliff, which is the feature.

## In flight

- **THE NORTH STAR — the engine shape everything converges to (RATIFIED
  2026-07-23; `docs/design/north-star.md`; now CLAUDE.md read-first item 0).**
  A **native** engine whose core is only cell storage + a pass-runner + native
  field-solvers + a stable API surface; materials (sheet + behavior slots +
  parent-shadowing hierarchy + slug→assets — the `Providers` pattern dropped to
  material level), their behavior, and the passes over them are **authored in a
  uniform, self-declaring, compiler-validated shape and tuned by data**. Behavior
  is code, tuning is data. **Plugin-first, closed-source-OK, untrusted-third-party
  safe** via a tiered backend behind ONE authoring shape: native `abi_stable` for
  trusted/first-party (incl. runtime), WASM sandbox for untrusted (gen-tier — the
  two-clock reconciliation; the Minecraft-space one-better). The crossing
  constraint (plain-data + handles, no rich Rust across the seam) taxes the SDK
  *surface we design*, never the content author's expressiveness. Pursued
  **evolutionarily** — the seam-first march IS the path (~70% embryonic in-tree:
  pass graph, Providers, MaterialProps, transformation-axes, categories,
  build_checked, octree, two-clock). **Compliance loop live: all design flows
  through it, double-checked at plan/work/review, carve-outs through the user
  (strategic companion to spines).** Narrative: journal/0081. **De-risk before
  committing the arc (sequenced, not urgent):** (1) hierarchical material behavior
  resolution prototyped on wood/charcoal `combust→`; (2) one existing pass (fires)
  reformulated onto the Pass interface, byte-identical; (3) the ABI/WASM boundary
  research spike (`abi_stable` vs `repr(C)` vs `wasmtime`) — locks the SDK shape.

- **THE MATERIAL-BEHAVIOR SUBSTRATE — design pass RATIFIED 2026-07-23
  (`docs/design/material-behavior.md`, journal/0086).** The spec for the
  north-star content layer and the Crux 1 build. The deep-cell inventory IS the
  fill contract one tier up (a stack of spans indexed by depth; strata record =
  temporal authority, mutable working-inventory = current, chapter-commit = the
  compiler step). **Forms** are a closed machine set (structure/loose/pore-fill/
  fluid; void = the complement) with fractional occupancy orthogonal; **edges**
  (all form→form moves) are machine-complete and ungated — they ARE the process
  catalog; **agents** are content rate-terms folded on an edge (S16); **passes**
  come in two shapes — **cellular** (run edges, local) and **field** (compute a
  field, plant it) — with the reciprocal loop (S-4) carried by the chapter loop
  and dual processes split into field+cellular halves. Ownership: forms+edges =
  machine, materials+agents+passes = content; forms are NOT SDK-registrable.
  **Progress 2026-07-23:** (A1) present-tier collapse **MERGED** (`de984eb`,
  journal/0087, gate-verified on merged main) — 1-byte atom (64→32 KiB/chunk;
  intermediate `{Air,Material,+4 legacy S1}` on the ratified path, 2-variant at
  Crux 2), `block_twin`+`_=>Stone` deleted, `classify→dominant material identity`,
  and a latent S-3 violation the collapse surfaced was fixed. **User ratified the
  appearance change (within-class member variety now visible at block/far tier)
  on the identity argument, sight-unseen.** Tail: solidity→occupancy drain (~80
  sites, deferred), far-span `Block` token migration, legacy S1 retire (Crux 2),
  categories-registrable. (A2) deep-cell working-inventory spike **COMPLETE**
  (S17, `docs/spikes/S17-deep-cell-inventory-results.md`, branch unmerged):
  byte-identical under identity default, **per-stratum** granularity, and the
  inventory is derivable → a **per-chapter transient** (no permanent memory cost).
  **Open, blocking merge:** the record is keyed by depositional *environment* not
  material, so material-changing behaviors must **commit as appended facts (S-9)**,
  not tag rewrites — a design call that shapes the keystone. commit-semantics
  **RATIFIED 2026-07-24** (append facts; fact carries `cause`; apply-time edge
  logging). **Progress 2026-07-24 — keystone + instrument MERGED, gate-verified:**
  the **keystone** (`37b0398`, journal/0088) — the fact-ledger, `commit_chapter`
  diff-and-append, provenance read, byte-identical under the identity default;
  single-edge shape, apply-time logging + `cause` land with the first behaviors.
  The **contents inspector** (`f594580`) — `world_get_contents` + F3 HUD (see the
  Shipped dev slice). The inspector surfaced that the runtime stores only `Block`;
  contents re-derive from gen (edit-blind, S-2), so **break-gives-mixture needs the
  runtime edit-fact overlay** — the north-star's first runtime-process milestone,
  designed (commit-as-facts) but unbuilt.

- **The block↔material collapse — DECIDED, one namespace (user, 2026-07-22
  late session; materials.md two new DECIDED entries)**. Block collapses
  into material + Air; no twin field is ever built (the block_twin slice as
  drafted is SUPERSEDED); classify answers with the dominant material
  wearing its own face; categories become pack-registrable (direction);
  transformation axes live on material definitions and BOTH sims read the
  same rule (S-3 applied to processes — the fires pass's soil-scalar proxy
  is explicitly un-avowed, heir chain: ecology → entry-species record →
  burned-axis on organics; coal/charcoal ride as-built meanwhile). First
  slice when dispatched: kill the fifteen-name match + `_ => Stone` arm in
  classify (dominant material's own identity), byte-identity where faces
  exist today; the Block-token consumer migration (storage palette,
  far-field span, solidity checks, mesher layer, player name) is the
  long-tail arc.
  **NOW FRAMED AS STEP 1 OF THE NORTH STAR** (2026-07-23): "materials are the
  universal substance the API hangs off." **Cruxes RATIFIED (user, recon:
  `docs/audits/` block-consumer inventory, journal/0081):** (a) ~~**render-first
  wedge** — delete `meshing.rs::block_layer`'s geology re-translation, route
  `classify → material → atlas`.~~ **INVESTIGATED 2026-07-23, journal/0082: the
  premise was falsified — no deletion.** The near-field mesher's contents-bearing
  path already routes `contents → material → material_layer` directly (via
  `top_splat`; it never calls `block_layer`). `block_layer`'s geology arms are
  **not** dead: they are the block-only (contents-absent) summary, exercised by
  the far-field pyramid `push_quad` (production render), the benches, and the
  absent-contents geology fallback — the *same* mechanism as the four legacy arms
  the crux keeps, so they cannot be deleted in isolation without breaking the far
  field (out of scope) or making `block_layer` non-total. Crux 1 (below) subsumes
  them. Landed instead: a guard test
  (`block_only_geology_layer_agrees_with_direct_material_layer`) pinning
  `material_layer(m) == block_layer(block_twin(m))` for the seven primary geology
  blocks against silent drift (A-7). The only residual *real* Block→layer
  round-trip for geology is the far-field top face (`farmesh.rs::push_quad`) — a
  separate slice, feasible via the far pyramid's material store. (b) **Crux 1 —
  air is a peer:** redefine `Block` in
  place to `enum { Air, Material(MaterialId) }`, keep the token for is_solid/==Air,
  delete `block_twin`; niche-optimise `MaterialId` for a **1-byte atom** (64→32
  KB/chunk, a storage/perf win — verify). (c) **Crux 2 — retire, do not enshrine:**
  the four legacy S1 blocks (grass/dirt/stone/wood) and the `TerrainGen` path are
  **retired**, not migrated to materials ("it already exists" is not a reason;
  carve-outs kept for sentiment are the stumbling blocks the shape can't afford) —
  verify no live fallback first, re-point it at the real worldgen in the same
  slice. Order: render-first · redefine Block · drain solidity→occupancy · storage
  atom · retire legacy · then categories-registrable. Sibling forks
  (`block_uses_contents` trust gate; the render-edit-writes-materials question)
  parked downstream of the atom. Journal 0081 written (design pass).
  **COUPLED (user, 2026-07-23, from the weathering-spike shaping):** the
  **deep-cell material-inventory** question is the same question as Crux 1's
  storage atom, one tier up. Deeptime stores bedrock/regolith as *heights* (R, H)
  + a strata record, NOT a material multiset — so a behavior that "consumes
  bedrock, produces regolith" has no inventory to operate on. The weathering spike
  works around it with a thin ctx-adapter over heights (deliberately), but the real
  fix — a material inventory for the deep cell — **needs solving soon** and rides
  with the material↔block collapse. Filed so it is not the spike's silent debt.

- **The ratified sequence after the migration (user, 2026-07-22: "both
  revisions greenlit")**: **1. The perf window** — one dc-client cluster:
  ~~profiling slice (Tracy/tracing spans + vertical-drop baseline → ranked
  killer list)~~ **LANDED 2026-07-23, journal/0080 (`perf` feature + self-time
  aggregating layer + `--perf-drop` capture; baseline artifact stubbed pending an
  integrator GPU run)** · **async-offload slice is now the sequenced NEXT** (chunk
  gen + far-mesh onto AsyncComputeTaskPool; the synchronous `stream_chunks`
  gen+mesh loop is the span-named suspect — `chunk.gen`/`chunk.contents`/
  `neighbor_fill.gen`/`mesh_chunk` on the frame thread) · ~~erosion-budget dev flag
  (the walkable cranked world → the standing amplitude call)~~ **LANDED
  2026-07-22, journal/0076 (`--erosion-budget <mult>`)** · albedo-at-
  range once baseline numbers exist, carrying the ranges-as-player-config
  rider. This PROMOTES perf over the queued seam batch per the runtime-is-
  sacred convention; **the seam-conversion batch (materials/form,
  paleoclimate, doc riders) runs in PARALLEL** — headless, disjoint
  write-sets. **2. The dressing arc** — the user's filed anticipation
  ("back to block primitives + the artful procedural dressing") named as
  next-after: material heightmap SHAPES for legibility, form-dependent
  textures, turf presentation, block primitives; the settings-plumbing
  slice (game config menu for render knobs) opens with it. FF2b
  persistence + dirty-rail slots into the first free gap; remaining
  CoarseField migration follows the type freeze.

- **The threshold-quantization migration — GREENLIT, "we finish this today"**
  (user, 2026-07-22; audit: docs/audits/2026-07-22-threshold-quantization-
  audit.md; end-state: spines § S-4 "Ratified end-state"). Route: **A1
  dispatched** (share-weighted susceptibility blend over the dominance
  window — dissolves the S-4 contour by construction; journal/0072) → FF2b
  integration → **B1 LANDED** (surface_class plurality → membership dither, the
  move-B witness; journal/0073 — resolves the 10 km checkerboard Observed item;
  **NEEDS RATIFICATION**: the draw is the *coherent* bilinear field, not the
  audit's white noise — white noise doubled the far-tile mesh because the far
  field point-samples it, and the unbiased end-state is deferred to the type's
  share-summary read) → **CoarseField<T> — EXTRACTED 2026-07-22 (journal/0075).**
  Built in dc-core (headless): `CoarseField<T>` + `Interpolable` (move A, anchored
  blend exact at identities) + `ShareVec` (the quantity both moves read) +
  `sample_dithered` (cake-law boundary membership dither) + `summarize` (coarse
  read) + `DitherSource` (the source axis), with generic law-tests and a
  `compile_fail` doc-test proving the raw per-cell read is inexpressible. A1
  migrated behind it **byte-identically** (goldens unmoved); the pinned pair
  `outcrop_at`+`outcrop_shares` **consolidated to one slot** (verdict derived =
  `argmax ∘ outcrop_shares`). Boundary-dither + summarize falsifiers pass;
  cake-law adoption in collapse.rs/far.rs is follow-on for those owners, and the
  node-payload sampling vocabulary adopts the type in a follow-on
  (octree-substrate.md § 3). **NEEDS RATIFICATION (a finding):** the coherent
  source's aggregate bias is *majority-amplifying (away from 50/50)*, the opposite
  sign of carve-out 1's "toward-50/50" characterisation — it sharpens minority
  phases rather than flattening them (see spines § 4 amendment; Observed).
  Audit shortlist: A1 EXTRACTED · B1 landed · A2 (rides A1's table) / A3 / A4 / B3
  / octree node = held for their owners. User's framing on the record: "there are
  things i'd rather do but this is foundational."

- **The octree substrate — DESIGN PASS OPENED, D1–D3 DECIDED** (2026-07-22,
  live session; `docs/design/octree-substrate.md`). The water.md leaning is
  resolved: **one substrate = the existing S3 chunk pyramid**, named and given
  a payload contract (node = `(level, ChunkPos)`; first two consumers —
  renderer, water — define the payload; statistical/social recorded as
  intended extension). **First build slice ratified: FF2b-minimal** — the two
  existing reduction pyramids (block + `MixtureDownsampleRule`, spines § 3
  row 1) through FF2a's stepped mesher, coarse volumetric far chunks replacing
  the top-sheet-only far field; persistence + dirty-rail is the follow-on
  slice. **Stepped all the way** ratified for the far register (alternatives
  recorded in the doc § 5; vista-as-augury is complementary — it governs far
  *live* state, not terrain). This supersedes FF2b's earlier pairing with the
  caves/underground water thread for the *minimal* slice — the node contract's
  water stratum stays requirements-only and the hydrology pause holds. **Node
  contract v0.1 RATIFIED** (same session — the gap-hunt added two-sided
  derivation: reduce upward where children exist, synthesize top-down from
  the worldgen authority where they don't, statistical agreement where they
  meet; ungenerated ≠ empty; strata independent; seeded synthesis).
  Narrative: journal/0069. **FF2b-minimal LANDED** (2026-07-22, background
  agent, worktree branch — journal/0070): the two-sided derivation is real.
  `dc_core::farfield` carries the span-stack payload (`ColumnSpan` gains
  `bottom`; `quantize_top` moves to dc-core so synthesis and the streamer
  share ONE quantization); `dc-worldgen/src/far.rs` synthesizes nodes
  top-down from `coarse_surface` (seeded); the client's `FarPyramid` feeds
  every streamed chunk up the block + `MixtureDownsampleRule` pyramids
  (**spines § 3 row 1 consumed** — far spans render `classify` of reduced
  mixtures) and the FF2a mesher now speaks span stacks (bottom faces,
  interval walls) with an A-5 guard (`subtree_fully_inserted`) and a
  reduction standoff inside the near field's draw radius. Agreement measured:
  n=8192 columns, mean(reduced−synth)=+0.938 coarse voxels, 100 % within 1
  (the majority-vote-rounds-up vs floor mechanism, journal/0070). Tripwire at
  10 km (stretched L4): 1 648 tiles / ~168 MiB / ~4.9 ms per tile — mesh path
  nowhere near the Aokana wire; past ~10 km the answer stays "more rings"
  (journal/0042). Follow-ons unchanged: persistence + dirty-rail (far edits),
  synthesized sub-surface strata (the `surface_sample` summarization-half
  home), partial-coverage composition, deep-span greedy merge.
  **MERGED TO MAIN + LOOK RATIFIED AS-BUILT** (2026-07-22, integrator merge;
  user, from the 0070 screenshot set: "visually indistinguishable from the
  previous version, for me. which is good!" — the reduced-tile patches ride;
  the 10 km checkerboard is pre-existing, filed in Observed, and is B1's
  cure to claim; merged-main gate evidence in the merge-gate-0070 log).

- **The forms/partials fill contract — DESIGN PASS RATIFIED, FOUNDATION
  DISPATCHED** (2026-07-21, live session; ARCHITECTURE.md § "The fill
  contract", materials.md § "The forms design pass"). The user ratified
  **contents-as-authority: `classify(contents) -> Block`, invariant
  `block == classify(contents)`**, after a consumer audit (135 block-consuming
  sites across 31 files, of which **80 across 27 files are solidity-shaped
  checks** — `!= Block::Air` / `== Block::Air` / `is_solid`) showed four future
  systems (water fill, loose gravity, compaction, sim light) all want one
  per-voxel occupancy primitive rather than each re-deriving occupancy from
  that bool. Also ratified in the same conversation: fractions come
  **only from the ledger** (recorded quantity/variance, never cosmetic noise —
  this scopes the defect-jitter sketch); **sand is a FORM of the existing
  clastic-coarse material**, not a new identity (accepted cost: loose and
  structural clastic are visually indistinguishable until a later
  form-dependent-texture visuals decision); the **root-lattice soil model**
  (roots as porous STRUCTURE holding soil in its pores — an anti-erosion
  mechanism for free, since matrix-less loose soil falls); and **grass is
  suspended, not expressed** (an ecology state riding on substrate materials;
  the user pre-ratified the appearance: "it'll be a mostly brown world for a
  bit" — turf's ligature/anisotropic-variant presentation sketch is captured
  but explicitly NOT to be built). Background agent dispatched for the
  foundation: dc-core `classify` + occupancy primitives, contents-first
  rewire of `collapse.rs`, the byte-identity + invariant regression proof,
  then (if clean) the first ledger-justified fractional top on the eolian
  sand blanket. Client wiring (lighting up journal/0010's dormant
  partial-height renderer, moving collision onto the occupancy threshold) is
  a deliberate follow-on slice, not this one.
  - **FOUNDATION LANDED 2026-07-21 (journal/0052).** `dc_core::classify`
    ships with `block_twin` / `dominant_material`, plus the occupancy
    primitives (`free_eighths`, `loose_eighths`, `bound_eighths`,
    `open_pores`, `is_occupancy_solid`, `SOLID_EIGHTHS = 4`) whose doc block
    names their four intended consumers so none re-derives occupancy
    privately. `collapse.rs` is rewired contents-first. **Byte-identical:
    proven by goldens captured against the pre-rewire generator — and
    INDEPENDENTLY RE-VERIFIED by the integrator, who ran the agent's own
    fingerprint against pre-merge main and reproduced all three triples
    exactly, so the goldens are not circular.** The invariant holds over
    3.64 M contents-bearing voxels. Note the pre-existing
    `geology_world_regenerates_byte_identically` proves *determinism*, not
    invariance across a change — the goldens carry that claim alone, which is
    why the independent check mattered. Also landed: the journal/0050
    collapse-cache `evict()` gap (now reachable from `coarse_surface`,
    `column_record`, `surface_elev_m`, `lattice_point`, `surface_chunk_y`).
  - **Design decision worth knowing**: `classify` is keyed on the **material**,
    not the geology class — contents carry only `MaterialId`s by design, and
    passing a `GeologySet` in would make the block tier a function of the
    registry and destroy the purity the invariant rests on. Byte-neutral for
    every set in the repo; the one divergence from the retired class table is
    a pack binding a material into a class whose block disagrees with the
    material's own nature (e.g. `PEAT` registered as clastic-fine), where the
    material is now authoritative. Flagged for the user, not blocking.
  - **NEXT SLICE — the fractional top, and it is BLOCKED on a user-owned
    decision.** The eolian remainder is real ledger (2.09 m / 0.9 m → 2 voxels
    + ~2.6 eighths discarded at `.round()`), but a partial voxel with solid
    material above it is a void underground: the only honest home for a
    fraction is the top of the column, and that voxel is the **surface-veneer
    stub**, whose retirement ratification 4 explicitly reserves for its own
    slice. Doctrine ("fractions come only from the ledger") forbids
    manufacturing a remainder for the veneer, whose thickness is a
    whole-voxel analytic budget. So the sequence is: **retire the veneer for
    recorded columns first** (user in the room), then `StrataEvent.top_eighths`
    emits the real remainder as `debris_only` — a quantity change on one
    voxel, since clastic strata are *already* emitted loose. The golden
    fingerprints move in that slice, and that move is its deliverable.
- *(**Tectonics architecture RATIFIED 2026-07-20** — all of U1–U8, with
  U3 amended (ritual ceiling relaxed to "5 min if that's what it takes");
  see the tectonics.md banner. The SPIKE is next — sequenced below, behind
  the eolian agent's landing: both write `deeptime/erosion.rs`.)*
- **Tectonic design pass — DRAFT LANDED (historical entry)**
  (docs/design/tectonics.md, merged 2026-07-20; Fable design agent). The
  inversion: surface uplift stops being the input — plate kinematics
  (advected Voronoi seeds, K~8 chapters) drive analytic boundary forcing →
  crustal-column thickening → smoothed-load Airy isostasy derives
  elevation, buying rebound/exhumation/forelands from one mechanism.
  Deformation re-derived analytically at collapse resolution (no per-cell
  event storage); sparse event list for what kinematics can't re-derive
  (unblocks the § 2 igneous flag). Eight user decisions U1–U8 pending
  (plate-scale knob, chapter count, ritual-length fork, orogen widths,
  advection scale, punctuation budgets, amplitude re-sequencing, flip).
  § 14 corrections verified by integrator (corrections #20; Erosion::new
  uplift_sum cache; Large-extent province-density inversion). Next:
  user ratifies architecture → SPIKE per its § SPIKE.
- **Zonal circulation profile** — dispatched same moment (write-set
  disjoint from spike: climate.rs only). Smooth wind magnitude + subsidence
  aridity; kills the band-flip line and puts a desert belt at ~30° for the
  true reason. Changes every new world's climate — integrator presents the
  measured shift for the user's eye.

*(**Ore design pass: DRAFT LANDED** 2026-07-21, `docs/design/ores.md` —
engineering pass over the DECIDED 2026-07-20 roster; R1–R7 awaiting the user.
Two collisions reported for the record: the ratified lode-gold flagship vs
S12's metre-scale exhumation (probe-conditioned in R1), and BIF vs the
Phanerozoic register (reconciled in R3). Placer source-blindness filed as
stubs.md § 11.)*

**Slated by ratification 2026-07-20 (all six unknown-unknowns landed;
sequenced, not yet scheduled):** marker beds + punctuation event types ride
the volcanism design (earth-processes § 2 flag); ore-genesis roster is a
user-owned content design pass (earth-processes § "payoff layer") to be run
as main-session design conversation; impact structures ride punctuation
hooks.

**Session 3 shipped, all gates green on merged main:** FF2a voxel far field
(0023) · far-seam uniform-push fix (0024) · S10 biotic layer (0025) ·
organic materials + biotic flip (0026) · organics photo walk (0027) ·
S11 water locality + body graph (0028) · erodibility coupling (0029) ·
erodibility production flip + walk (0030). Eight journal entries; seven
corrections filed (#11–#19, two of them the assistant's own).

**Session 5 shipped (2026-07-21, journal/0039–0049, corrections #23–#25),
all gates green on merged main:** deep-config plumbing (0039) · walk 0040 +
the amplitude answer "neither" (#23) · S13 roughness measurement (0041,
#24/#25) · `--horizon` knob (0042) · climate registration fix (0043) · **U8
tectonic flip (0044)** · ore textures (0045) + substance redo (0048) · steep
walk 0046 · **full_agents flip + tour map (0047)** · **the first live
guided-tour ratification (0049)**. Doctrine: ledger-expression + enhancement
+ perf-first + genesis/stubs (stubs.md, 11 entries). Ore design pass drafted
(ores.md R1–R8 pending).

**(SUPERSEDED by the 2026-07-22 close at the end of this file.)** NEXT SESSION — rewritten at the 2026-07-21 session close, AFTER the live
walk (supersedes every earlier same-day block). Read first:**
**journal/0050–0058** — the leak, the eviction, the fill contract, carry-`H`,
DeviceLost, distribution-first, the holes, the eight-kilometre typo — and
**corrections #26–#29**, three of which are the integrator's own errors.
`docs/design/stubs.md` for the doctrine registry.
**Walk wiring:** game MCP is a checked-in `.mcp.json`; launch the game FIRST
(`cargo run --release -p dc-client -- --horizon 3`), then `/mcp` reconnect.

> **⚠ THE FIRST THING TO DO IS WALK.** Six merges today changed what the world
> is made of, how deep it digs, and what its surface looks like — and **the two
> most visible fixes (surface dither, the holes) landed after the user's
> session closed, so nobody has seen them.** Everything below is downstream of
> that walk. **Station coordinates are METRES** — pass them to `pose_set`
> directly; the integrator multiplied by 0.9 and spent an afternoon 8.6 km from
> every station (corrections #28).
>
> What to check, in order: (1) are the sky-holes gone (`--fullbright --edges`,
> re-walk `journal/assets/0056-holes-after-settle.png`'s view; if bands persist,
> distrust journal/0057 first); (2) are the chunk-shaped surface patches gone
> (compare against `0056-surface-quantized-per-chunk.png`); (3) does the
> **loess margin (82346, 24391) m** — the deepest section in the world, ~90
> sediment blocks, 76 mixed spans — read as *sediment* or as noise; (4) do
> contact bands change material on chunk lines (the boundary-dither loose end,
> still open for **mixed** voxels); (5) a long `--horizon 6` session — **DONE
> 2026-07-21, journal/0065: it survives.**

**Wide horizons: the blocker is GONE and now PROVEN at 6** (journal/0065,
2026-07-21). The DeviceLost crashes were host-RAM exhaustion from an unbounded
chunk store, now evicting (journal/0051, flat over 210 teleports at
`--horizon 3`). Re-measured at 6, in both regimes, alternated 6/3/6/3 against
machine drift: **teleport storm +0.542 / +0.529 MB/jump at horizon 6 versus
+0.531 / +0.542 at horizon 3** — the horizon signal is zero — and **idle at
horizon 6 drifts under 7 MB in seven minutes** with every probe count frozen,
which is the regime that used to die at ~4.5 min. Plus one unbroken
**700-jump / 23-minute** horizon-6 session: RSS sawtooths in a ~980–1 170 MB
band, long-run slope **+0.11 MB/jump**, 71 928 evictions, exit 0. All runs
exited **0**, no `DeviceLost`, no panic, no `ERROR`. Horizon 6's real cost is a **constant**
~155 MB / 704 resident far tiles (vs 288 at horizon 3), paid once at fill.
**Wide-horizon walks are unblocked**; 8–10 should hold, and the number to
watch there is `far_tiles` at fill, not slope.

**Decisions waiting on the user (nothing else is blocked on them):**
0. **THE SOIL GAP — the biggest design hole the walk exposed.** The user, in
   world: *"the world does not generate dirt anywhere apparently — was only a
   veneer placement previously."* Verified: `MaterialId::LOAM` is registered in
   **no** geology class member, and the `organic-soil` class's only member is
   carbonaceous **mudstone**, a rock. **The world has no soil substance at
   all**, which is why 91.4 % of land skins to one block — both candidate
   explanations were downstream of that. It compounds with form: organics sit
   in the *structure* bucket, so soil would render as solid cubes even once it
   exists. **Fix order: substance first, then form-from-provenance**
   (consume-the-ledger piece (c)). This is the missing substrate under the
   already-ratified root-lattice soil model (materials.md § forms design pass)
   — that model currently has no material to hold in its pores. Content-set
   work, so the member roster is a **user call**.
1. *(**The surface-veneer retirement: DONE** 2026-07-21 — the user made the
   call live ("fold them together... the record already says") and it shipped
   in journal/0055. What replaces it as the live user question is the
   **appearance walk** above.)*
1b. **WALK THE NEW WORLD — the biggest unratified appearance change this
   project has made.** Two merges changed every surface and every dig depth
   (journal/0053 carry-`H`, journal/0055 the record-skinned surface) and the
   user has seen neither. The tour map for 0053 is below; for 0055 the sites
   to add are the **bare granite shoulder** (101663, 5073 — basement at grade,
   with a horizon that agrees), the **dune field** (107183, 9672 — nine mixed
   spans where there were zero), and **any cut face** for the new contact
   voxels. LIT pass for shape and depth; `--fullbright` for the material
   question specifically. Both are revertible: one merge commit each.
2. **Ores R1–R8** (ores.md draft).
3. **Roughness recalibration** — three costed candidates, picked from
   pictures; C is the only one that changes which landforms exist. Sequence
   the `--horizon 6` landform walk first so the pick is made against real
   terrain.
4. **`classify` is material-keyed, not class-keyed** (journal/0052) — a
   judgment call the integrator accepted; byte-neutral today, diverges only
   for a pack that binds a material into a contradicting class.

**Design passes queued (main-session conversations, not dispatches):**
ecology (starts from biomes-ecology-agenda.md) · social sim (the big one) ·
the water/caves thread (water.md § Session capture 2026-07-21 — the
two-drainage-opinions finding and the bounded-drainage-refinement spike
question are the live items).

**Engineering still Sequenced:** *(**sub-voxel facies / the sieve: SHIPPED**
2026-07-21, journal/0055 — stubs § 12 retired)* · tectonic expression
(dip/fold + metamorphic — *the deep-time half now has a socket waiting for it:
`Providers::outcrop_at`, journal/0060*) · paleo-context provider (subsumes stubs
4/5/6) · **the littoral heir: fetch × wind into `Providers::wave_energy`**
(journal/0060; needs the S11 body graph's open-water field, so it will likely
arrive as a materialized plane and convert that slot from value-level to
pass-level) · **the waterlogging heir: the S11 saturation field into
`Providers::depth_to_water`** (journal/0061 — the slot and its `WaterPass`
contract exist; what remains is the field, and the units decision, since the
four consumers threshold a 0..1 index and a water table speaks metres) ·
`HostWorld` edited-chunk spill to the save layer (edited chunks are retained
unboundedly by design; ~640 MB per 10 000 edited chunks, asserted by test) ·
**the coarse-capacity void axis** (S15 design choice 3, journal/0062 — capacity
below a cell's floor plane comes only from edits today, which is complete for a
heightfield world and wrong the moment caves exist; the collapse tier's
"recorded conduit capacity → void intervals per column" is the same axis, so
decide it once in water.md rather than twice) · **re-run S15's natural-sill
falsifier when the cave families land** (not constructible today only because
the world has no 3-D structure — corrections #31) · **when the store can fail
to answer (async streaming / disk-backed regions), an absent chunk must read
UNKNOWN, never solid**, or eviction manufactures false component boundaries in
the connectivity index (corrections #31) · reconcile S15's cell-granularity body
footprint with S11's air-component container (S15 design choice 2).

**⚠ UNRATIFIED APPEARANCE CHANGE AWAITING THE USER'S EYE (2026-07-21):**
carry-`H` (journal/0053) changed dig depth across the whole world and the
user has not seen it. Arid basins and dune fields went from 1–3 voxels of
dirt over stone to 8–12 voxels of loose fill; 0.2 % of land is now bare rock
at grade (e.g. 101663, 5073 — basalt, no soil). Soil depth now correlates
with erosion history instead of rainfall, which is the ratified *direction*,
but the magnitude and the fact that the world got **deeper on average rather
than barer** is the opposite of what the tour verdict anticipated. **Walk it
before ratifying** — it is one merge commit and trivially revertible.

*Tour map for that walk* (seed 1337 / Medium; regenerate with
`cargo run --release -p dc-worldgen --example soil_depth_probe`).

**⚠ UNITS — read before teleporting (corrections #28).** The station
coordinates below are **world METRES**, which is what `pose_set` takes, so pass
them **directly**. `soil_depth_probe::STATIONS` holds metres and *divides* by
0.9 to reach voxels; the integrator multiplied instead and walked every station
of the 2026-07-21 live tour **8.6 km off target**, then "verified" it by
confirming `pos_voxel` matched what he aimed at — a check that could not tell
the two hypotheses apart. The voxel address of a station is `metres / 0.9`
(loess margin = voxel 91 496, 27 101).

Use the LIT pass — this is a dig-depth/section question:
- **(101663, 5073) m — bare rock at grade**, `H` 0.17 m. The new extreme:
  basement at the surface, no soil at all. Did not exist before this slice.
  *Start here.*
- **(5993, 14732) m — tour station 1**, `H` 10.66 m. The station that motivated
  carry-`H` and moved the opposite way (corrections #26).
- **(107183, 9672) m — tour station 2 dune field**, `H` 7.99 m; the record
  holds 379 units that expressed as *zero* whole-voxel strata before
  journal/0055. Best place to see what the sieve was eating.
- **(82346, 24391) m — loess margin**, `H` 80.49 m → **188 voxel spans, 76 of
  them mixed, ~90 sediment blocks**. The deepest, richest section in the world
  and the best cut face available. *(Verified headlessly in journal/0058 — the
  "only 3 voxels here" alarm was the integrator standing 8.6 km away.)*

*(Superseded by the session-close block above; kept for the record.)*
**RE-SEQUENCED 2026-07-21 by the journal/0040 walk.** The amplitude call is
**answered: neither 80 nor 160** (corrections #23) — `thickening_scale` acts
at ~25 km and above and buys *zero* sub-km relief, so it cannot fix dismal
mountains and no longer blocks anything. The order that replaces it:

1. *(**Roughness decay: MEASURED** 2026-07-21, S13/journal/0041. What remains
   is the **user's pick between three costed candidates** — see Sequenced.)*
0. *(**Steep-site re-walk: DONE** 2026-07-21, journal/0046 — the #25 debt
   paid, on a stock post-U8 production world. The scarp is legible (terraced
   stone basin at −77 m, dirt shoreline stripe, green rim; ~80 m/km of real
   deep-field relief); the crest is still the 0040 prairie from a 6 km
   horizon — both halves of #25 confirmed by eye. The distant lapse-rate
   banding (green→brown→stone) is visible at the horizon for the first time.
   The recalibration pick now has its context pictures: A/B would texture a
   plateau that has no shape to reveal; C is the only candidate that changes
   which landforms exist in the 460 m–7.4 km band.)*
2. *(**Far-field horizon knob: SHIPPED** 2026-07-21, journal/0042 — the
   landform-shape walk is now possible and is owed: re-walk the amplitude
   vantages at `--horizon 6` on the LIT pass.)*
3. **Erosion-supply calibration** (Sequenced) — S12's metre-scale exhumation
   finding, now co-equal with (1) as a relief-generating lever.
4. **The amplitude value itself** — deferrable. Rides as-built at 80 until
   (1) and (3) change what the knob is multiplying.
5. **Sim light SPIKE** — design pass is done (`docs/design/light.md`);
   § 10 of that doc states exactly what the spike must measure.

**Read first next session:** `docs/design/things-that-will-happen.md` (new
this session, and now item 2 in CLAUDE.md's read-first), then
corrections #18 and #19 — both are about choosing an instrument that can
see the question you are asking.

## Sequenced

<!-- Two arcs sequenced 2026-07-24 with full reasoning + a reserved continuation
slot each, per the user's "slice-of" principle: never lose what a completed slice
was a slice OF. Each names WHAT, WHY, how it UNIFIES with the larger threads, its
FIRST SLICE, and the CONTINUATION SLOT that outlives that slice. -->

- **FLOW IS ONE PROCESS — flux on FACES, facts on STRATA; retire the receiver tree**
  (arc opened + **RATIFIED 2026-07-25**, user "standing blessed"; design doc
  **`docs/design/flow.md`**, reasoning **journal/0095**). *Supersedes the river half of
  `water.md` and the **expression half** of `earth-processes.md` § 3e-2; § 3e-2's
  constraint half survives **qualified** — the no-divide-crossing rule binds the FREE
  regime only, or regional groundwater is foreclosed.*
  - **WHAT.** One process — *matter moves down a **potential** gradient through a
    resisting medium, carrying a load it exchanges with the substrate.* Recorded as one
    atom: **at (cell, stratum-slot), flux crossed faces F with magnitude M, in form P
    (free/bound), carrying load L of fluid f, under cause C, at chapter K.** Three
    ratified collapses: **(1)** free/bound is an **occupancy** — Fluid is already a form,
    so free↔bound is an **edge on the form graph (S-8)**, which makes a spring a derived
    outlet, a cave free-phase flow below the surface, and karst the already-dormant
    `Cause::Dissolution`; **(2)** in a stratigraphic record **depth IS time** (an
    unconformity is a flow signature, not missing data); **(3)** flux lives on **FACES,
    not cells** — lateral (cell↔cell per slot), **vertical (slot↔slot in a column)**, and
    boundary (atmosphere/ocean).
  - **WHY.** `flow_to: Option<u32>` is one out-edge per cell — a spanning tree. A tree
    represents convergence and **cannot represent divergence at all**: no distributaries,
    braids, fans or deltas. **The Mississippi delta is unrepresentable**, and no threshold
    or grid change fixes a primitive. Today's carve is also **half-wired and invisible**
    (`let (elev, _riverbed) = carve_rivers(…)` — riverbed discarded at both call sites) and
    is routed at ~14.7 km on **pre-erosion** topography, i.e. across a landscape deep time
    then destroys. And a **face is shared**, so refinement built on face data agrees from
    both sides *by construction* — **the seamlessness requirement and the divergence
    requirement have the same answer.**
  - **UNIFIES.** = the paleo-channel/layering need · **§13.8 flow-biased sub-cell fill**
    (which finally has a path to bias toward — the flux record is *the rich thing others
    bias to*, never a second geometry model) · **Movement 2b material-aware transport**
    (the load-exchange half; wind/ice/lava/turbidity are the same machinery per §13.2) ·
    **karst/caves** (corrections #17: dissolution is a term on the weathering edge, not a
    new incision model) · **S11's runtime** (bodies-not-links, `sat.rs`, zero cave-specific
    code) · the **interval-log fill contract** (voids are intervals, not a heightfield) ·
    **`identify(pos)`** (a voxel that is "air that is a cave that was a phreatic tube" is
    exactly what a 1-byte summary cannot say).
  - **THE DEEPEST DEFECT IT FIXES.** *The record is the ONLY seam between deeptime and
    runtime — if refinement needs something, deeptime must have recorded it.* Today
    drainage is solved **every epoch and discarded** (`recv`/`area` are documented as "the
    **last** routing"): we run the process 200× and keep the final frame. Every paleo-flow
    signature the design wants was computed and thrown away.
  - **STATUS 2026-07-25: slice 1 (journal/0096) and continuation (a) (journal/0098) have
    SHIPPED — the ARC STAYS OPEN.** Slice 1: divergence 175,320 / convergence 60,915 on the
    production world; record 40.66 MiB at 8.386 % face sparsity. **(a) the head field:** the
    `head` condition-field (`dc:field/head`) as the second §5 field pass, and its consumer —
    the **vertical faces slice 1 left honestly zero now carry 307,364 crossings** on 131,586
    columns (44.3 % of cells), and **artesian occurs naturally**: 60 subaerial columns stand
    their water above their own ground under a confining bed (max excess 2.94 m), which
    `H = y + sat` cannot express at any resolution — and 24,935 more carry a water table
    *below* ground, a depth-to-water derived from the rock rather than from present-day
    precipitation. Terrain byte-identical; +6.96 MiB (149.21 → 156.16), +0.5 s gen.
    **(b)–(e) are untouched and still owed**, and three riders remain in the slot: the
    **slot-pairing rule for the BOUND regime** (free flow pairs by chapter; bound may need
    paleo-elevation — the user's call; **note (a) did NOT force it**, because vertical faces
    pair *within* a column), the **marine-sink residency lever** (79.38 % of entries,
    31.37 MiB, sized and deliberately not pulled), and now the **recharge term** (a) left as a
    seam — `∇·(T∇h) = −R` needs `R/T` in real units, which is the same missing
    precipitation-depth scale that keeps the lateral source uniform, and it is why (a)'s
    artesian excesses are metres rather than the hundreds of metres of a real basin.
  - **FIRST SLICE — the RECORDING half only.** Face flux per chapter in deeptime: replace
    the receiver output with **face-flux records** (3D faces), attach flow facts to unit
    slots, keep the existing priority-flood/route/accumulate **solve** (good numerics —
    only its output representation changes). **Nothing expressed at runtime yet**; the
    world stays honestly river-less rather than gaining a second fake. Acceptance =
    a **convergence AND a divergence** present in the record on a production world (a
    tree cannot produce the latter), plus a **measured resident cost** (gen time is free;
    residency is not).
  - **CONTINUATION SLOT** (this is a slice OF *"flow is one process, faithfully recorded
    and purely refined"*): after recording, the arc continues with **~~(a)~~ SHIPPED
    2026-07-25 (journal/0098) — the potential/head field pass.** `dc:field/head` is the
    second §5 field pass: it derives each column's transmissivity / vertical conductivity /
    **confinement** from the strata record's own permeabilities (a marine mud over a fluvial
    sand *is* a confined aquifer — no landform code path), relaxes `∇·(T∇h) = 0` under
    Dirichlet conditions at the sea, lakes and perennial streams, and leaves **confined**
    columns **uncapped** — which is the whole artesian mechanism, expressed as *the absence
    of a seepage cap*. It fills the vertical faces (307,364 crossings; 1,137 of them
    artesian **rise**). It leaves as named seams: **recharge** (`R/T` needs real units),
    **precipitation weighting** (would desync the record's two halves, and the lateral half
    moves terrain), **buoyancy/density** (ρ named and multiplied in, inert at 1.0 — heir is
    (d)), and **unsaturated/transient Darcy** (capillary rise needs it). **It did NOT do
    MFD** — deliberately, since that changes routing and therefore the world.
    *The directive that governed it, kept for the intervening slices that still apply:*
    **⚠ USER DIRECTIVE 2026-07-25:** *"build assuming that we want the head field to exist
    soon — i.e. do not foreclose, leave seams where possible, keep direction in mind."* So
    every slice must (i) treat "flow descends **potential**" as the target, never hard-code
    elevation-descent, (ii) keep the **vertical faces** structurally present, and (iii) not
    bake the free/surface **no-divide-crossing** rule into anything the bound regime will
    inherit (flow.md §2.4 — bound flow genuinely crosses surface divides; (a)'s solve has no
    divide term at all, pinned by `bound_head_crosses_a_surface_drainage_divide`).
    **The head field is also what unlocks a multi-flow-direction solve**, and therefore
    *simultaneous* divergence (flow.md §2.6) — today's divergence is still aggregation-window
    avulsion only, and **MFD is now the nearest-term continuation**;
    **(b) refinement as a boundary-value problem** — face fluxes as Dirichlet conditions,
    the load budget as mass, solved *inside* a cell, pure-of-position (never reading a
    neighbour's refined output, so the pure-fn chunk holds); **(c)** the **free/bound edge
    + void intervals** (caves, conduits, springs, `Cause::Dissolution` switched on) — **and the
    CONDUIT PAIRING RULE** (ratified into (c) 2026-07-25): a karst conduit is confined by **its own
    void geometry**, not by a depositional horizon nor a potential surface, so it pairs by **void
    connectivity** — a **third mode** that flow.md §11.5's two-mode green explicitly does NOT cover.
    So (c) ships **three** obligations, not two;
    **(d) fluid identity** (lava/ice/brine on the same atom, own competence curves);
    **(e)** retiring `pregen/hydrology.rs`, `RiverSeg`/`carve_rivers`, and
    `Cell::{flow_to,river,discharge}`. **Do not close the arc when the first slice lands.**
  - **NAMED LIMITS (honest, from the 2026-07-25 foreclosure sweep — flow.md § 5):**
    oscillatory/tidal flow nets to ≈0 and needs a **gross-energy** term · **episodic**
    catastrophes (turbidite/jökulhlaup/lahar) average away inside a chapter though the
    graded bed IS the signature · **sub-cell meander migration** is refinement-tier ·
    **evaporites** precipitate because the *carrier left*, a distinct trigger from a
    falling competence ceiling.
  - **ACCEPTANCE (the prize, un-gameable by statistics):** cut a cliff face and read
    *channel gravel with a placer streak → floodplain silt → an unconformity where the
    river left → carbonate dissolved into a phreatic tube, now dry → collapse breccia on
    its floor → a spring line downslope.* Each is the same atom at a different slot.
    **If any of it needs a landform-specific code path, it is not faithful.**
  - **NOT A CONSTRAINT (user, 2026-07-25, emphatic):** socia/civ/eco/bio consumers
    (settlement siting et al.) are **stubs and baggage to be replaced** — they do not
    exist as designed systems and **must not constrain the flow design at all**. Future
    designs compose with flow; flow does not bend around unavowed stubs.

- **STRUCTURE-AWARE FINE EXPRESSION — the sub-resolution the collapse randomizes but
  physics structures** (filed 2026-07-24 at the user's direction; unifies the flow-biased-fill
  deferral of material-behavior.md §13.8 with the journal/0055 + journal/0010 within-voxel
  order residual and stubs.md § 12's residual).
  - **WHAT.** The collapse expresses a coarse record at fine resolution by **addressed
    stochastic rounding** (journal/0055) — mass-honest but **spatially unbiased**. Two scales
    of one gap:
    - **Within-VOXEL order (sub-0.9 m):** a mixed voxel loses the vertical ORDER of what it
      mixes, so a sharp contact renders as **journal/0010 speckle, not a band** (stubs.md § 12
      residual; ratified non-blocking, "belongs to the forms presentation work").
    - **Within-CELL distribution (0.9 m–460 m):** a deep cell's deposited material is spread
      ~uniformly across its column, **not biased** to where it physically settled — coarse near
      the paleochannel, fines in the distal lows, cross-beds dipping downflow
      (material-behavior.md §13.8).
  - **WHY THEY ARE ONE ITEM.** Both are **S-4 "coarse cause → fine expression"** where the fine
    expression **randomizes where the physics would structure**. Same class, two axes (vertical
    bed order vs horizontal flow bias), same eventual home — the **forms / presentation pass**.
    The fix reads a **structuring signal** (contact orientation for order; the flow field `recv`
    + local topography for placement) instead of the current unbiased sieve.
  - **INPUTS ALREADY EXIST.** Material-aware transport (§13) produces exactly what biased fill
    needs — the cell's composition **and** its flow vector (`recv`) — so deferring the biased
    expression loses nothing; the record just needs to also carry the within-voxel order the
    sieve currently drops.
  - **STATUS: non-blocking, and the two scales differ in value** (user, 2026-07-24). The
    within-VOXEL order loss is **ratified-acceptable** — "granted micro-scale lossiness, still
    reads realistic for most purposes"; **not a defect to fix.** The within-CELL flow-biased
    placement is an **interesting-but-unsettled enhancement** — "super interesting… not sure
    about that one, food for thought"; NOT a committed direction. This entry keeps both
    discoverable and honestly-statused; the biased-fill inputs already exist (§13) so nothing is
    lost by leaving it open.

- **MIGRATE EVERY "NOT REAL" FIELD INTO A REAL DECLARED FIELD PASS** (reminder, user-directed
  2026-07-24). As the §5 **field-pass half** lands (geotherm = the first, this session), the
  ad-hoc / stub / unconsumed proto-fields must each migrate into a **real declared field pass
  writing to the condition-field vocabulary** — not stay ad-hoc planes: `exhum` + `t_crust`
  exported planes (spines §3, unconsumed), the **degenerate `burial_temp_c` geotherm** (stubs.md
  #14 — the real geotherm subsumes it, coal recalibration rides along), drainage `recv`/`area`/
  `lake` (spines §3, unconsumed — the flow field material-aware transport consumes), and any
  field computed inline that a formation predicate or cellular pass ought to read by id. **Each
  migration empties a §3 row and grows the SDK's condition-field vocabulary.** The
  condition-field vocabulary IS the formation-predicate SDK surface (§12 formation is
  output-owned; the predicate is plain data over field-ids — crossing constraint satisfied).

- **APPEARANCE WALKS OWED** (tracking, user: "we do that when able" — journal screenshots for
  appearance-changing work). **(1)** ✅ **DONE — walk-confirmed 2026-07-24** (user, aerial
  fullbright: *"I can now confirm the LOD is fixed!!"* — fine near field transitions cleanly into
  the warm coarse far LOD, no cold-dither ring; screenshots `journal/assets/0091-lod-walk-*`).
  The only residue is the poke-through geometry check on the lit pass — low priority. Still owed:
  **(2)** ✅ **DONE — walk-confirmed & ACCEPTED 2026-07-25** (user; journal/0097). Flag-ON vs the
  byte-identical flag-OFF control on the same column: **7 voxels ≈ 6.3 m** of loose product at the
  basement contact (predicted 6.09 m), **24.2 % of land banded**, veneer and basement unmoved.
  Instrument `--fullbright` (a material question). **Accepted with a follow-up, not a blocker** —
  the band has a **hard perimeter** (see Sequenced "the weathering front needs a PROFILE"), and the
  gradational-looking top contact is **boundary quantization, not weathering** (one voxel deep,
  `mixed_voxel_contents`); **(4)** the **weathering-front PROFILE** flag-ON walk (journal/0099,
  shipped 2026-07-25) — the band is now a graded **19-voxel** front, ~**2.67×** deeper than the
  old slab, with **retained parent structure at the bottom contact** (7/8 parent + 1/8 product)
  instead of a hard perimeter; station world **(84185 m, 9212 m)**, voxels **y=299…281** — cut a
  **bench** (not a pit), `--fullbright`, read with `world_get_contents`; **(3)** the geotherm's
  coal-distribution shift when it lands (coal
  moves as the real gradient replaces the degenerate stub). Screenshots to `journal/assets/`
  named for their entry.

- ✅ **DONE 2026-07-25 — shipped, see Shipped (journal/0100).** Measured result: flag-ON
  `DeepField` **311.02 → 179.12 MiB**, the flag's own cost **+161.81 → +29.91 MiB (5.41×)**,
  ledger heap **155.01 → 16.31 MiB (9.5×)**, index 3.4 % of the ledger, world byte-identical
  (same 1,033,189 facts in the same 72,006 slots). *Entry kept below as shaped, for the record.*
  **`FactLedger` IS 89 % EMPTY HEADERS — give it the CSR layout `flux.rs` already proves**
  (shaped 2026-07-25 at the user's direction; measured in `docs/spikes/S19-flow-record-cost-results.md`).
  - **WHAT.** `FactLedger` is `Vec<Vec<Fact>>` keyed per (cell, slot). Measured on a production
    world: **5,832,862 inner `Vec`s of which 5,760,856 (98.8 %) are EMPTY**; **89 % of its
    ~150 MiB heap is empty `Vec` headers**, against a real payload of **16.6 MiB over 1.03 M
    facts**. Turning `weather_inventory` ON therefore costs **+156.91 MiB** — and after the
    `shrink_to_fit` win that is **~1.4× the entire rest of the `DeepField`** (108.55 MiB bare).
  - **WHY NOW.** The walk **blessed the band** (journal/0097), so the flag is on its way to
    becoming a default rather than a dev toggle — and the moment it is, this is the single
    largest residency item in the world. Runtime residency is first-class (CLAUDE.md); gen time
    is free, so the conversion cost is free.
  - **THE FIX IS ALREADY PROVEN IN-TREE — do not design a new one (A-4).** `deeptime/flux.rs`
    (journal/0096) stores a far larger sparse per-(cell,chapter,face) record as **flat
    exact-sized arrays + a CSR index**, and measured the index floor at **0.056× of total** —
    i.e. *the index is free and the payload is the whole constraint*. Port that layout. Facts are
    also **causally triangular** (a slot deposited in chapter `c` cannot carry a fact from before
    `c` — 57.7 % of the naive rectangle, a free 1.73×), so never allocate the rectangle.
    **⚠ CORRECTED BY THE BUILD (journal/0100):** the triangular exploit is **SUBSUMED, not
    applied** — exact-sizing stores the **1,033,189 facts that actually exist**, which is **4 % of
    even the causal ceiling**. Triangularity is a **budgeting tool for projecting an UNBUILT
    record, never a sizing rule for a BUILT one**: once you can count the real entries, any
    formula over the possible ones is a ceiling you have already beaten. Keep that distinction
    when using S19's projections for the flow record's later slices.
  - **SCOPE.** `deeptime/inventory.rs` + its readers. **Pure layout change: byte-identical
    world, identical facts, identical `weathering_product_m`** — the goldens and every
    fact-count test must pass **unmoved and by name**. Acceptance = the measured before/after
    residency with `weather_inventory` ON, plus byte-identity proven by test name.
  - **BLOCKED-ON:** `inventory.rs` sits inside `deeptime/`, which the in-flight **head field**
    slice owns. Launch when that lands, or carve `inventory.rs` out of its write-set explicitly.
  - **NOTE THE SHAPE, not just the number:** this is the same defect the flow record was warned
    off in-flight and avoided. Fixing it here closes the loop — the measurement that protected
    the new record should also repair the old one.

- **WEATHERING IS ONE PROCESS — SAPROLITE IS A STATE ALONG IT, NOT A SLICE**
  (**USER'S STRONG LEANING on the destination, 2026-07-25** — *"strong enough that it should pop
  into sequence when the requisites are met"*. Not scheduled; **gated on the requisites below, and
  it enters Sequenced the moment they are met.**)
  - **WHAT.** Retire saprolite as a bespoke thing. Weathering becomes **one declared process**
    acting on whatever is exposed to reactants at a rate set by *material susceptibility ×
    driver × access* — the gradation **emerging from the rate**, not imposed by a shape function.
    Saprolite is then simply the region of the continuum where parent fabric is still recognisable.
  - **WHY (the user's reasoning, recorded).** *"Saprolitification is a slice of a more general
    process of weathering that ends in totally crumbling — are we splitting that one process
    between multiple slice owners for any good reason?"* Today it **is** split three ways:
    `dc:deep/weather_inventory` (declared, deep tier, produces a **scalar**) · `dc:deep/weather`
    (the scalar `R`/`H` height authority, still unreconciled) · `geology.rs::emplace_weathering_front`
    (**not declarative** — ordinary collapse code that *invents* the vertical distribution). Only
    one of the three is authored in the shape everything is obligate to converge on.
  - **THE THREE ARGUMENTS THAT CARRY IT.** **(1)** Nature has no saprolite *process* — it has
    weathering, and saprolite is a **named state along a continuum** (intact → fractured → saprock
    → saprolite → residual soil); modelling a stage as a mechanism is the same category error as
    S18's one-shot. **(2)** The gradation would become **emergent**: fronts are exponential
    *because* first-order kinetics consume a downward-advecting reactant, so modelling the cause
    makes `WEATHERING_PROFILE` and **stubs #20 DELETED, not tuned** — the disposal the stub doctrine
    wants. **(3)** The **7/8 cap is a definition wearing physics' clothes** — weathering does not
    halt at 7/8 retained fabric, so the cap makes complete weathering (laterite, oxisol, total
    crumbling) **unrepresentable**. Also: today the pass only weathers **bedrock**, because the
    materialized seam is the only `Structure` in the inventory — nature weathers sediments, soils
    and transported clasts too (that narrowing is stub #16's shadow, not a physical claim).
  - **IT DISSOLVES THE PROFILE-vs-NORMALIZATION PROBLEM** (the user's own conclusion, and it is
    right): *a scalar only needs a profile invented for it when the process forgot to record depth.*
    This is the same finding as journal/0099's PLEA, reached from the other direction — the collapse
    can only impose a **universal** shape while the physical controls already live in the deep sim.
  - **⚠ REQUISITES — the honest reason this is not scheduled yet (the counter-case is about
    ORDERING, not destination).** The exponential emerges from **reactant transport**, and we have
    **no vertical fluid flux**. A depth-resolved weathering rate would have nothing honest to read,
    so "emergent" gradation would emerge from a **fabricated depth term** — the same shape function
    with better camouflage (**A-1** with a disguise). It pops into Sequenced when:
    **(R1)** the **potential/head field** lands (flow continuation (a), in flight) and
    **(R2)** **vertical flux is real** rather than the honest zeros of flow slice 1 — i.e. there is
    an infiltration/percolation term a per-depth rate can read. **(R3)** measure the cost first:
    per-depth × per-cell × per-epoch over 297 k cells with multi-slot columns is far larger than
    today's per-cell scalar, and the deep run is already 35–85 s.
  - **ONE CONSTRAINT ON "ONE PROCESS" (integrator, agreed at the same time):** *one process must
    NOT mean one pass.* §5's split still binds — reactant **transport is a FIELD**, the **edge is
    CELLULAR**. Collapsing them would rebuild the monolith the pass-runner exists to prevent.
  - **WHEN IT LANDS:** stubs **#20 deleted** (not tuned), the 7/8 cap gone, `WEATHERING_PROFILE`
    deleted, the collapse fold demoted from *inventing* a distribution to *expressing* a recorded
    one, and the two-authorities split (material vs height) becomes derivable rather than
    maintained — which is Movement 2a's stated direction anyway.

- **✅ SHIPPED 2026-07-25 (journal/0099) — see § Shipped for the result.** *(Entry kept for its
  reasoning; the flag-ON walk it earned is now item (4) of APPEARANCE WALKS OWED.)*
  ~~THE WEATHERING FRONT NEEDS A PROFILE, NOT A SLAB~~ (walk finding, user, 2026-07-25;
  journal/0097). **WHAT.** Movement 3's band is correctly *magnituded* and wrongly *shaped*: the
  collapse folds the scalar `FactLedger::weathering_product_m` into **one stratum of one class**
  (`CLASS_CLASTIC_FINE`, `geology.rs::emplace_weathering_front`), so the record→voxel path expresses
  a span that wholly contains a voxel as `Single` — **8/8 of one member**. Result: pure product
  above, **pristine contents-free basement below, a hard perimeter on both faces.** User: *"the
  layer of degraded bedrock has a hard perimeter and then pure bedrock, which does not make sense
  for the natural process it claims to model… nature does not in-place degrade a bulk unit of rock
  to another via weathering."*
  - **WHY IT HAPPENS.** The inventory edge is honest at its own tier — `(GRANITE, Structure) →
    (GRANITE, Loose)`, a **form** change on one material, mass-conserving, one fact per agent. The
    loss is at the **fold**: `weathering_product_m` is a **scalar, and a scalar cannot carry a
    profile.** What the model computed is a rate integrated over depth and time; what got emplaced
    is a slab. The downward gradient — intact rock → corestones → grus → clay — *is* what makes
    saprolite legible as saprolite, and it is exactly what the fold discards.
  - **HEIR SHAPE (user-proposed, expressible in TODAY's vocabulary — which is what makes this a
    follow-up and not a research project).** `structure → pore_fill` rather than
    `structure → structure`: **retained parent structure with weathering product in its pores**, the
    structure share falling with height through the front. `VoxelContents` already carries
    `structure[]` / `pore_fill[]` / `open_pores` / `debris[]` in eighths — the walk read them
    straight off `world_get_contents`. Today's band says `structure: []`, `debris: [mudstone 8/8]`:
    **the parent rock is simply gone.**
  - **COUPLES TO** stub #16 (a rind's *material identity* — the inventory says granite-loose while
    the collapse expresses mudstone; the two disagree today and #16 owns that half) and to the
    deep-cell inventory's **form vocabulary** (§2 forms / §3 transition graph — this is a
    form-transition question, so it belongs to the same machine).
  - **NOTE THE FLATTERING ARTIFACT (journal/0097, worth not re-deriving):** the band's *top* contact
    already mixes and reads convincingly — but that is **boundary quantization**
    (`mixed_voxel_contents` / `allocate_partial`, journal/0055), **exactly one voxel deep, wherever
    any two units meet**, and would look identical at the contact of two units that never
    interacted. It is not a weathering gradient and must not be mistaken for progress on this item.

- **GEOTHERM: nonlinear / mantle-heat enrichment** (followup, user-directed 2026-07-24). The v1
  geotherm (material-behavior.md §14) is a per-cell **linear** gradient `T(depth) = surface_T +
  gradient·depth`. Enrich later: non-linear `T(depth)` + a mantle-heat contribution (deep
  thermal structure, not just a surface-anchored line). Non-blocking; the linear v1 is the
  honest first field pass and the vocabulary/consumer shape it lands is unchanged by the
  enrichment.

- **THE HONEST IDENTITY SURFACE — retire the stored `Block` summary; one
  `identify(pos)`** (arc opened 2026-07-24; priors: materials.md
  block-is-material DECIDED, the block-consumer recon, ARCHITECTURE § "a summary is
  not an authority", S-3/S-9).
  - **✅ SLICE 1 SHIPPED 2026-07-25 — journal/0101. THE ARC STAYS OPEN.**
    `HostWorld::identify(pos) -> Identity` (`dc-api/src/identify.rs`): untiered,
    position-addressed, payload uniformly a mixture, with **`Identity::Unrecorded`
    as a first-class value** distinct in the TYPE from `Mixture(EMPTY)`. Wired into
    `dc:world/get_contents` (both lying fields fixed), `character_sense_raycast`,
    and the F3 HUD (its `(no contents record here)` branch is now reachable).
    Enabler: the stored `Block` already disambiguates — empty record + `Air` ⇒
    genuinely empty, empty record + anything else ⇒ unrecorded — so **no
    dc-worldgen change was needed**. A *non-empty* record still wins outright, so
    the block/classified divergence that signals an edit stays legible. Census
    through the real query path (`dc-client/examples/identify_census.rs`): phantom
    air **702 → 0** (of 10 985 solid voxels over 169 columns; the 702 land in
    `UNRECORDED` beside the 4 211 that were already honest, and the 6 072 recorded
    mixtures do not move). **Still owed by this arc:** everything in
    the continuation slot below — nothing was drained, deleted, or migrated, and
    the path remains **edit-blind for composition** (an edit writes a `Block`; no
    mixture is stored, so none can move).
  - **WHAT.** The honest answer to "what is this voxel" is its full `VoxelContents`
    (up to 8 partials), never one arbitrary component. One **position-addressed**
    function `identify(pos) -> payload`, where the payload is **uniformly a mixture**.
    Raycast composes it (`raycast(camera) → pos → identify`) — a separate step, because
    "what is at X" is a world question, not a camera question.
  - **⚠ UNTIERED — DECIDED 2026-07-25 (user), SUPERSEDING the Near/Mid/Far tier design.**
    This entry previously specified `identify(pos) -> { tier, payload }` with tiers
    derived from the LOD ladder. **Retired.** The user's challenge: *"if this is a world
    query then why tiered at all when we can inspect chunk and read the voxel?"* — and
    the provenance matters: **the tiering was assistant-originated**, an artifact of one
    request ("a way to get voxel composition by looking at it") being split into several
    instruments, which then hardened into architecture without ever being re-challenged.
    - **The defect it encoded: two different questions were CONFLATED.** *"What is at
      world position X"* is a **world** question with one true answer, to which distance
      is irrelevant. *"What is the renderer showing at X / which rung is resident"* is a
      **render** question — legitimately per-viewer, legitimately vaguer, and the only
      one with any business reading `LodLadder`. This entry's own line already said so
      ("a world question, not a camera question") and then built a camera concept into
      the world query anyway.
    - **Cost was the strongest counter-argument, and it fails.** Gen is **pure-of-position**,
      so the true answer is *always* derivable — there is no fundamental barrier, only a
      latency one. Cost must change the **policy or the latency, never the ANSWER**;
      returning a vaguer truth because the honest one was expensive is precisely the
      summary-wearing-an-authority's-clothes this arc exists to retire.
    - **THE SHAPE:** `identify(pos)` returns the honest full mixture at `pos`, **always** —
      **resident chunk first (INCLUDING dirty/edited state)**, derived if absent,
      **`UNRECORDED`** where no record backs it. **No tier flag, no `LodLadder` coupling,
      no distance thresholds.** Dropping tiers also dissolves the "whose ladder?" problem
      (per-viewer dc-client state vs a headless world query) — there was no good answer
      because the question was malformed.
    - **What survives unchanged:** payload is **uniformly a mixture** (so far-field
      *speckle* falls out by construction — the renderer speckles, the query reports the
      mixture; there is no far tier left to special-case), and **`UNRECORDED` as a
      first-class answer** (corrections #49's measured 6.4 %).
    - **"Reads the dirty rails edits too" (user) is the sharp half**, and it lands on a
      known gap: contents are re-derived from worldgen and are **edit-blind** today
      (journal/0088 — the runtime `Chunk` stores only the 1-byte `Block`, so a mixture does
      not survive an edit). That *is* the **runtime edit-fact overlay** already in this
      arc's CONTINUATION SLOT — contents as *derivable base + edit facts* — and it is what
      makes "break gives you the real mixture" true.
  - **THE F3 INSTRUMENT ALREADY EXISTS, and is fine as-is** (user, 2026-07-25). The
    look-at contents HUD (`dc-client/src/inspector.rs`, journal/0088) fulfils some version
    of the original request: *"I just use F3 to know what's in front of my face when doing
    the walks."* It **rides the looking-at-voxel highlight box, so it has a limited range**
    — and that is **honestly fine for now**. **NOT A PRIORITY (user, explicit):** when
    `identify` is better, the HUD *could* raytrace and ride that too — which would mean
    generating a chunk from a **LOD octree node intersection** and then finding the voxel
    intersection from the same ray. Recorded so the idea is not re-derived; **do not
    schedule it.** *(Note: this HUD carries the corrections #49 defect identically — its
    correct "no contents record here" branch is unreachable — so the `UNRECORDED` fix
    repairs it for free.)*
  - **INFORMATION-AVAILABILITY IS A GAMEPLAY LAYER, NEVER A QUERY LIMITATION** (agreed
    2026-07-25; **no plan, not scheduled**). If the game ever wants "you cannot identify
    distant strata without a survey instrument", that rides **on top of** an honest query.
    User: it *"would come through layers of honest obscurity due to how player perception
    and knowledge will be modeled"* — i.e. it belongs to the (undesigned) perception /
    knowledge model, **not** to the world query. Recorded specifically so nobody
    re-derives distance-tiering later by mistaking a game rule for an engine constraint.
  - **~~TIER BOUNDARIES DERIVE FROM THE LOD LADDER~~ — SUPERSEDED SAME DAY by the UNTIERED
    decision above. Preserved because the reasoning is still load-bearing for the
    *render-side* query, if one is ever wanted.** ~~DECIDED 2026-07-25 (user):~~ *"the
    boundaries should fall out of LOD bands, which already reduce material contents
    depth / honesty."* The tiers are **not a second, independently-tuned threshold set**
    — that would be A-4 (a mechanism beside the one we have) and would drift out of sync
    with what is actually on screen. `LodLadder` (`dc-client/src/farmesh.rs:126`,
    journal/0091 — one ladder, all named knobs, every ring edge already **derived** from
    it) is the authority; `identify`'s Near/Mid/Far read *it*. Payoff: the ladder is
    already structured to become **in-game per-player perf settings**, so honesty
    automatically tracks the player's own quality setting — turn the view distance down
    and the answers get *honestly* coarser, with no second knob to forget.
  - **~~⚠ OPEN, and it must be settled before dispatch — WHOSE ladder?~~ DISSOLVED
    2026-07-25 with the tiers themselves** (there was no good answer because the
    question was malformed — see the UNTIERED block above). Preserved only for the
    *render-side* query, if one is ever wanted.** `LodLadder` is
    **dc-client** state and is **per-viewer**, but `identify(pos)` is a **world** question
    reachable headlessly (dc-api agents, mods, tests) where there is no camera and no
    ladder. So the signature cannot simply read ambient client state. Candidate
    resolutions (not chosen): **(i)** `identify` takes an explicit *tier/observer* argument
    and the client passes the one its ladder implies — keeps the world query pure and makes
    the client the only place that knows about cameras; **(ii)** it defaults to
    **finest-resident** and the client narrows; **(iii)** the ladder (or a headless-safe
    projection of it) moves somewhere both crates can see. **(i) is the integrator's lean**
    — it preserves "what is at X is a world question, not a camera question", which this
    very entry already asserts. User call at dispatch time.
  - **THE FAR TIER'S PAYLOAD IS A MIXTURE, NOT A WINNER — DECIDED 2026-07-25 (user):** *"LOD may
    be textured by a **speckled mix** in the future, not just single material as it is now. So
    leave the seam for speckle — or better yet have it fall out by construction."* **It falls out
    by construction, and that is the design:** make the payload **uniformly a mixture at every
    tier**, so the tiers differ in **RESOLUTION** (how many components survive, at what precision),
    **never in KIND**. A single dominant material is then just *a one-component mixture at 8/8* —
    today's `classify` answer expressed in the general shape, with **no special case to migrate**
    the day the far field goes speckled. Writing `Far = one MaterialId` would bake exactly the
    "one arbitrary component" assumption this whole arc exists to retire (and would need an A-2
    correction the moment speckle lands). **Corollary:** the same shape carries the **UNRECORDED**
    answer below — an empty mixture is not the same value as a one-component `Air` mixture.
    **Ties into** the far-field speckle direction the genesis-passes / octave arc is heading for,
    and the LOD fix (b) cold/warm **material** agreement (S-9) still owed from journal/0091.
  - **FIRST CONCRETE REQUIREMENT, now measured rather than argued (corrections #49,
    2026-07-25):** the tier flag must be able to say **"UNRECORDED"** as a first-class
    answer, distinct from both *"air"* and *"recorded"*. Today `has_contents` is answered
    **per-chunk**, so an unrecorded basement voxel reports `has_contents: true` +
    `classified: dc:air` **over solid stone** — **6.4 % of near-surface solid voxels**,
    globally. `Option::None`, the only channel that could have meant "no record here", was
    already spent on a whole-chunk condition inherited from the mesher. **A summary is not
    an authority — including when it is a `bool` named after the thing it is not
    measuring.**
  - **WHY.** "Nobody wants one arbitrary material component of a mixture" (user). The
    summary existed for the **surface-only** far field; the far field went **volumetric**
    (FF2b), so its justification **expired — A-2**. The renderer already re-derives full
    contents (`chunk_contents`) and splats them; the stored 1-byte `Block` is a
    storage/sim relic the render **bypasses**, and it loses the mixture on edit.
  - **UNIFIES.** = the block↔material collapse tail (storage palette · far-span ·
    ~80 solidity · mesher layer pick · player-facing name) + the **distance pyramid**
    (near/mid/far *are* the `identify` tiers) + **S-3** (`classify` demoted to a derived
    rung + the pyramid's coarsest tier, never a stored authority) + **S-9** (self-labeled
    honesty).
  - **FIRST SLICE — ✅ SHIPPED 2026-07-25 (journal/0101), see the slice block at the top
    of this entry.** As dispatched: `identify(pos)` as one **untiered** surface behind
    `world_get_contents` / `sense_raycast` / the F3 HUD, with `UNRECORDED` first-class.
    **Not** done, and deliberately: draining the `Block`-token consumers (recon list) —
    that moved to the continuation slot with the rest of the retire. Contents stay
    re-derivable (unedited).
  - **CONTINUATION SLOT** (this is a slice OF *"the runtime canonical is contents, not a
    summary"*): after `identify`, the arc continues with the **storage/wire migration**
    (edited voxels carry contents/edit-facts, not a `Block`), the **runtime edit-fact
    overlay** (break/place = move-facts — the north-star's *first runtime-process
    milestone*, which also makes break-gives-the-real-mixture true), the **far-span
    `Block`→material** migration, and **legacy-S1 retire** (Crux 2). Do not close the
    arc when the first slice lands.

- **GENESIS-PASSES DRIVE ROCK DISTRIBUTION — retire class-member into hierarchy +
  property-driven passes** (arc opened 2026-07-24; priors: materials.md "the class
  system's fixed-constant roster is scaffolding" + transformation-axes DECIDED, north-star
  declarative-materials, the entry-species probe, the seam-inventory `[S2]`).
  - **WHAT.** Retire the class-member-fitness abstraction. Rock **distribution + physical
    facies** come from **deeptime genesis passes** that select a **parent material**, march
    its **leaf** materials, and derive fitness **purely from properties stored on each
    material** (grain size, solubility, density…). Adding a leaf → it participates
    automatically. Diagenesis/metamorphism = **transform edges** (thermo/pressure agents)
    evolving the distribution across deeptime (the fact-ledger, applied to genesis). Output:
    where materials are · how distributed · the facies · **octave**-materialized → runtime.
    The material **hierarchy** (parent→leaf) serves as ~classes where needed.
  - **WHY.** The class system is self-admittedly *scaffolding* and **measured near-flat**
    (entry-species probe: formation context barely changes which member). North-star wants
    materials+passes+hierarchy; property-derivation makes it **modder-extensible** (leaves
    free); and it is the honest model — a rock's identity is set by its **genesis, at
    genesis, by the pass**.
  - **UNIFIES.** This **IS the member-selection/octave arc done right**: the genesis pass
    produces the distribution (property-derived, physical), the octave dither materialises
    it — so "retire class-member", "fix the member squares", and "physical facies from
    passes" are **one arc**, and it fixes the palette-quant + LOD root cause (coarse facies
    *point-sampled*) by driving distribution from a smooth physical field + octaves instead
    of a chunk-grid dither. Rides the material-behavior model (cellular passes, edges,
    agents, hierarchy).
  - **STEELMAN (recorded so it isn't re-lost):** keep the **distribution concept** (a rock
    unit *is* a mix of related materials — honest, housed in the hierarchy); class-fitness
    is a genesis **model** the pass *formalises*, not deletes; the **richness lives in the
    pass's physics** (property→distribution), which is near-flat today, so re-housing alone
    buys nothing — the win is encoding real depositional/petrological physics; **leaves are
    free, a new genesis PROCESS is a new pass**.
  - **FIRST SLICE.** **Seam-first, byte-identical:** convert the `DepTag→material` /
    `deep_class`/`dithered_member` determination (the seam-inventory `[S2]`
    `material_properties`) into a declared pass/provider that reads material **properties**
    — no behavior change, goldens hold.
  - **CONTINUATION SLOT** (this is a slice OF *"genesis is a property-driven pass
    pipeline"*): after the seam conversion — (1) **enrich the property→distribution
    physics** so energy/depth/thermo genuinely swing the mix; (2) **octave materialisation**
    (multi-octave + physical driver, retiring the single-octave chunk dither — the member
    arc); (3) **diagenesis/metamorphism as transform edges** (thermo/pressure agents, on
    the fact-ledger); (4) the **modder-authors-a-leaf** path (properties only, no bespoke
    code). Do not close the arc at the seam conversion.

- **The distance pyramid: consume the mixture LOD by band, drop the dither
  with range** (SEQUENCED 2026-07-21 at the user's direction; design captured
  in visuals.md § "LOD colour cascade"). Three bands, knobs on the dropoffs
  (user doctrine: *"where there's a cell range, there's a knob"*):
  **near** = texture / the 4×4 world-anchored dither over the real mixture;
  **mid** = the *averaged* mixture, one blended colour per coarsened voxel;
  **far** = the winning material only, one flat colour.
  - **Most of the data machinery already exists and is unrendered.** S8's
    `MixtureDownsampleRule` / `DominantClassDebrisAware`
    (`dc-core/src/materials/lod.rs`) already reduces a 2×2×2 cell of child
    `VoxelContents` (64 eighths) to one parent (8 eighths) — occupancy voted in
    eighths, class by volume with the losing class *folded in rather than
    lost*, slots by largest remainder, mirroring the block LOD's octant layout
    voxel-for-voxel with a proved invariant. The far band's "winning colour" is
    `dominant_material`/`classify` (journal/0052). **What is missing is a
    renderer that consumes the pyramid by distance**, not the pyramid.
  - **The perf case is measured, not assumed**: journal/0010 clocked a
    fully-mixed chunk at **16× the triangles** of a uniform one (196 608 vs
    12 288), purely because each mixed face becomes 16 dither quads. Journal/0055
    made mixed faces world-wide, so that ceiling is now being paid at range.
    Dropping the dither by band saves exactly on the faces that just got
    expensive.
  - **Enabling piece, cheap, do it first**: cache the blended colour **per
    interned `MixtureId`** rather than recomputing per voxel — the region
    `MixtureTable` is tiny (journal/0055 measured 325 mixtures / 3 160 bytes),
    so it is a rounding error and is computed once. (The user's instinct was
    per-material precompute; per-material albedo is already registry pack data
    from 3c-2, so per-mixture is the version that actually buys something.)
  - **Sequencing**: lands after the in-flight `meshing.rs` side-face fix
    (same file). Pairs naturally with **FF2b** (coarse volumetric summaries)
    and with the Observed *"far field is boxier than the near field"* item —
    a banded pyramid is the obvious place to carry sub-voxel height too.
    Band distances are an **appearance call: the user picks from pictures**,
    so ship the knobs before the values.
- **Edited chunks still accumulate for the session — the save-layer heir**
  (surfaced 2026-07-21 by journal/0051, which bounded everything *else* in
  `HostWorld.chunks`). Eviction deliberately pins edited chunks, so a session
  that edits 10 000 distinct chunks holds ~640 MB until quit (64 KB/chunk,
  measured). Correct-as-designed and now *visible* (`host_edited` in the
  `DC_MEM_PROBE` line, asserted by a dc-api test), but the real answer is the
  decided save/persistence layer: spill an edited chunk and let the slot go
  back to being evictable. Sequenced behind persistence itself.

- **Distance-evict re-derivable resident data — a knob for later** (user, 2026-07-24).
  The world is massive; a player 100 km away is holding resident chunk + warm-LOD data
  that is **re-derivable** (no edits) and just taking space (S-2: store only what the
  derivation can't predict). *Untouched* `HostWorld.chunks` already evict (journal/0051)
  and the far pyramid has a budget (`FAR_PYRAMID_L0_BUDGET`); the ask is a **distance
  knob** letting unedited resident data (near chunks AND the warm-reduced LOD nodes) be
  reclaimed as the player recedes, while edited chunks stay pinned (→ the save-layer
  above). Not now — a knob to add later; couples to the LOD warm-cache lifetime the
  haunted-LOD diagnosis flagged as a needs-live-probe.

- **Collapse-cache `evict()` is unreachable from far-field-only sampling**
  (diagnosed 2026-07-21, journal/0050; `evict()` fires only from
  `generate_chunk`, so a `coarse_surface`/`column_record` sweep can grow
  `lattice_memo`/`locale_cache`/`region_cache` between chunk generations).
  Bounded in normal play (a storm generates chunks constantly) — not the RAM
  march, which was `HostWorld.chunks` and is now fixed. **Reassigned
  2026-07-21** to the forms/partials `collapse.rs` rewrite, which owns that
  file. *(**Separate hardening item — DeviceLost degrades loudly: SHIPPED**
  2026-07-21, journal/0054.)*

- **Tectonic expression at the collapse tier — the layer-cake redemption**
  (promoted 2026-07-21 after the user's callout: dip/fold non-expression
  "slipped by without my understanding or ratification" — an integrator
  sequencing failure, acknowledged; the Observed LAYER-CAKE line was filed but
  never surfaced during the tectonics ratification). The ratified architecture
  always intended it: `DeepField.chapters` ships ~5 KB of plate state per
  chapter EXPLICITLY so per-unit deformation (dip, fault traces) re-derives
  analytically at collapse resolution — built in S12, consumed by nothing.
  One family, one slice-group, all UNGATED as of the U8 flip (landed 2026-07-21):
  (a) chapters → strata dip/fold/fault expression in cut faces (the
  "coal seam dead-ends at a fault" line of things-that-will-happen);
  (b) `exhum`/`t_crust` → metamorphic-grade classes;
  (c) drainage export (`recv`/`area`/`lake`) → the 3e-2 macro drainage
  consumers. Note: this is the CUT-FACE sin, not the silhouette sin — terrain
  shape flatness is the separate S13/roughness thread.
- **Consume the ledger terms the runtime throws away** (geology.md § Expression
  of the ledger, DECIDED 2026-07-21). Four concrete, independently shippable
  pieces: (a) *(**carry `H`: SHIPPED** 2026-07-21, journal/0053 — see Shipped.
  Both consumers read the recorded plane; stubs.md § 3 retired whole.)*;
  (b) **consume `exhum`/`t_crust`**, which ship
  explicitly as "the metamorphic-grade axes the collapse tier reads" and are
  read by nothing; (c) **derive material FORM** (loose / pore-partial / whole /
  inclusion) from provenance rather than leaving it implicit — sub-voxel facies
  express as inclusions (the charcoal sieve: 0 of 158 310 beds survived 0.9 m).
  **(c) is now the priority piece and much larger than the charcoal framing
  suggested**: journal/0053 measured the sub-voxel sieve deleting **~75 % of
  the whole sediment pile** (mean `H` 4.75 m, only ~1.15 m surviving as
  whole-voxel strata; the tour's dune field records 379 units summing to
  7.99 m and expresses *zero*). Filed as **stubs.md § 12**, heir named:
  amalgamate adjacent sub-voxel units **inside the record**, not in the
  veneer. It needs no user decision and rides the landed contents-authority
  machinery — the best next forms-adjacent slice;
  (d) **per-voxel provenance query** — read a voxel's ledger: started as X,
  heat/pressure did Y, moved because of Z. **RESOLVED 2026-07-24 by the
  commit-as-facts semantics** (material-behavior.md § Commit semantics DECIDED):
  provenance = `base + facts`, addressed to the material portion's lineage (a move
  is itself a fact that travels with it) — it falls out of the deep-cell inventory
  keystone and needs no separate design pass. The earlier "needs a design pass,
  PROPOSED not ratified" framing is retired.
- **Roughness recalibration — three costed candidates, USER PICKS FROM PICTURES**
  (S13 § 6, 2026-07-21; nothing flipped on). Binding constraint measured: the
  `s7_walk` seam test prints max interior step **2 of 6 voxels**, so there is
  **3.0× headroom** on the finest levels.
  - **A. Uniform re-anchor** `AMP_DECAY^(L−2)` (×3.31). Predicts 18/51/66 m at
    100 m/250 m/1 km at the walk site. Zero gen cost — but **measurably fails
    the seam test** (2 × 3.31 = 6.6 > 6); a ×2.5 variant fits with nothing to
    spare.
  - **B. Band-limited re-anchor**, boosting levels 6–10 by `[4.0, 3.4, 2.8,
    2.2, 1.6]`, unity from 11. Predicts 14/58/75 m. Zero cost, **seam-safe by
    construction**. Risks: 58 m over 250 m ≈ 23 % grade may hit character slope
    limits, and it is five hand-chosen numbers — *a fractal answer to a
    simulation question*.
  - **C. Refine the deep tier to 230 m cells.** ~62 s / ~210 MiB at Medium (from
    15.5 s / 52 MiB); needs `DEEP_MAX_WIDTH` raised and **breaks the ratified
    `pregen_time_vs_extent` < 60 s budget** (note: U3 already relaxed the ritual
    ceiling to "5 min if that's what it takes", so this may be cheaper than it
    reads). No predicted relief — S13 states plainly it cannot predict this
    without running it. **The only candidate that makes the *mountain* legible
    rather than the *ground*.**
  A and B garnish the ground; only C touches the 460 m–7.4 km band the `L_DEEP`
  override discards. Sequence the steep-site re-walk FIRST so the pick is made
  against the world's real terrain.
- *(**Far-field horizon knob: SHIPPED** 2026-07-21, journal/0042 — see Shipped.
  `--horizon <km>`, default provably unchanged, measured to 10 km.)*
- **A 5th/6th far LOD level — for horizons past ~10 km** (found by the
  journal/0042 measurements, 2026-07-21). The 4-level ring scheme's honest
  ceiling is ~10 km / 172 MiB / 13.7 s to fill, because cost is quadratic
  inside the stretched L4. Journal/0023 already named the right answer — *add
  rings*, don't lengthen the last one. Only worth doing if the design target
  wants vistas past 10 km; note the haze limit below may bind first.
- **Wave-magnitude retune — STRUCK 2026-07-21 (user): no retune.** "That
  whole mechanism changes after water machinery. that would be a bandaid,
  against our standing rule against bandaids. can revisit later." The
  confirmed null (0.68 m, journal/0049 station 5) rides as-built until the
  fetch model (wave energy from S11 body size/shape/depth × the 0037 wind
  field) replaces the constant outright — wave expression is a consumer of
  the water design pass now, not a tuning slice.
- **The forms/partials EMISSION slice** (user directive, journal/0049 station
  2; materials.md DECIDED 2026-07-21). **The forms DESIGN is now done —
  `material-behavior.md` §§ 2–3 IS that pass** (the closed form set structure/loose/
  pore-fill/fluid/void, the machine-complete transition graph, agents as folded
  rate-terms). What remains is the **emission slice**: world-gen emitting partials —
  sand as the first spawned loose material — which is a **rider on the deep-cell
  inventory keystone** (Crux 1), NOT a standalone design pass. The 0010 dormant loose
  renderer and S8 mechanics are the substrate.
- **Tectonics SPIKE** (per tectonics.md § SPIKE, architecture ratified
  2026-07-20). **[VALIDATED as FIELD-SOLVERS, 2026-07-24 (user-confirmed; north-star/
  material-behavior): the SOLVER half — tectonic deformation/uplift, drainage,
  climate, thermal — is a FIELD pass (compute+plant a field, native-core); do NOT
  recast *that* as cellular. But each has a cellular EXPRESSION pass that reads the
  field (ctx channel 2) and moves material — erosion/transport reads drainage; an
  **upheaval** pass reads the tectonic field. That split IS the design (material-
  behavior §5 dual-process split), NOT a recast. Same stamp covers erosion-supply,
  3e-2, and drainage below.]** implement `DeepConfig::tectonic_history` behind the flag and
  produce the eight measurement groups (clamp stability under ramped
  repaints, recorder growth vs K, ritual wall/memory at 200/300/400 iters
  reporting tradeoffs not optimizing to a cap per amended U3, landform
  evidence with pass bars + negative controls, forcing-wavelength kill of
  the 50 km artifact, byte-identity off, relief distributions for the
  re-sequenced amplitude call, drainage-export fidelity). **Dispatch after
  the eolian agent merges** — write-sets collide in `deeptime/erosion.rs`.
- *(**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037 — see
  Shipped.)*
- *(**Deep-config flag plumbing: SHIPPED** 2026-07-20, journal/0039 — see
  Shipped. The four launch flags (`--tectonics`, `--full-agents`,
  `--amplitude`, `--extent`) boot a flagged world; the combined amplitude /
  tectonic / full_agents walk is unblocked. The override channel is
  `DeepOverrides` on top of `production_config`, NOT a `WorldParams` field —
  the ~30 `{ seed, extent }` call sites were left untouched.)*
- **Erosion-supply calibration** (from the S12 spike's new finding,
  2026-07-20): exhumation comes out metre-scale at shipped erosion rates,
  gating exhumed-core/foreland legibility independent of amplitude — the
  S9 calibration loose end, now load-bearing. Needs-measurement class:
  compare model denudation against real orogen rates, propose 2–3
  calibrations, render each — the user then chooses between pictures, not
  rate constants. Dispatch after the combined walk settles amplitude.
- **Tectonic uplift-plane redesign — DESIGN PASS (superseded — done)** (direction ratified
  2026-07-20, earth-processes.md § 1 DECIDED entry): tectonic history
  (uplift(t), plate advection, chaptered boundary re-classification) +
  analytic boundary forcing (uplift from exact bisector distance at deep-grid
  resolution). Spike-class; upstream of dip/fold, volcanism, and the
  amplitude call. Includes the plate-count/scale-compression knob decision
  (user-owned).

- **Retire `client_player_pose_set` into `dc:character/pose`** (API.md
  Decisions #5, ratified 2026-07-20): make the player a dc-api character
  and route the player controller through controller-verb commands — the
  work that makes player input replayable (decision 2). Explicitly
  sequenced AFTER the session-4 console and edges agents merge: it rewrites
  `mcp.rs` and `player.rs`, both in those agents' write-sets.

*(**FF2a — voxel-language far field: SHIPPED** 2026-07-19, journal/0023 — see
Shipped. Stepped columns retired the smooth TIN; step 0 empirically confirmed
Bevy 0.19's GPU-driven multidraw engages for our custom material
(`mode=Culling`, 74 draws → 1 multidraw set); the buried-sheet and tile-crack
field reports are resolved, below.)* **FF2b — coarse volumetric
summaries** paired with the caves/underground thread of the water design
pass (when overhangs exist, the summary goes 3D; couples to S3 region
storage). FF2a left the extension point ready: the per-column payload is a
`ColumnSpan` the mesher already treats as one of a potential stack, and the
tile mesher is a pure function of plain span data (async-meshing / persistent
edit-tracked LOD store stay drop-in).


*(**Erodibility coupling — lithology-aware erosion: SHIPPED** 2026-07-20,
journal/0029 — see Shipped. Cause 1 of the dismal mountains is closed:
erosion is lithology-aware, off by default, byte-identical when off,
agent-specific resistance so karst/glacial/littoral stay implementable.
What remains is **user decisions**.)*

**NEEDS RATIFICATION (user-owned — this CHANGES TERRAIN SHAPE):**
1. ~~**Flipping `production_config`'s `erodibility` to true**~~ **RATIFIED AND
   DONE 2026-07-20** (user: "flip it, i want to see"; journal/0030). It is on;
   every new world has a different shape. The appearance answer came back
   **negative** — the ground-level surface is measurably re-terraced, but the
   predicted ledges/benches at hard beds and the resistant basement core are real
   in the data and **absent on screen** at the shipped amplitude (summit
   silhouette identical; 5 km lattice ±2 m; the ground dropped one voxel). Which
   makes item 2 below the live question, not a footnote.
2. **The contrast is currently modest at the shipped erosion amplitude** (relief
   +2 m aggregate) because the world barely erodes against its uplift; the
   headroom test shows the model produces real cliffs (44.7 m) the moment erosion
   is allowed to cut. Whether to raise `erodibility_contrast`, raise the global
   erosion rates (cause 3, "conservative amplitude"), or both, is the user's
   amplitude call — the lever now exists and is a knob.
   *(2026-07-22: the user noted they have never SEEN a cranked erosion
   budget — the headroom world exists only inside `erodibility_probe`
   experiment B; no launch path walks one. Note the term collision: the
   journal/0040 "amplitude call, answered neither" was TECTONIC amplitude
   (`thickening_scale`, --amplitude), a different lever. Enabling slice
   filed: extend 0040's deep-config launch plumbing with a dev erosion-budget
   flag (weathering / k_transport / k_bedrock ×N, optionally
   erodibility_contrast) so THIS call can be made from vistas on a same-seed
   pair.)*
   **ENABLING SLICE LANDED 2026-07-22 (journal/0076): `--erosion-budget <mult>`
   on dc-client rides the 0039 `DeepOverrides` door as `erosion_budget:
   Option<f64>`, multiplying `weathering`/`k_transport`/`k_bedrock` together at
   world build (experiment B's exact semantics; relative rates fixed). `1.0` is
   byte-identical to no flag (falsifier `erosion_budget_one_is_byte_identical_to_no_flag`);
   goldens did not move. The walk is UNBLOCKED — the amplitude call can now be
   made from vistas on a same-seed pair (`--erosion-budget 3` vs `10`). The knob
   is the erosion budget only; `erodibility_contrast` was left for a later slice
   (the budget alone is what experiment B moved). The DECISION is still the
   user's; this slice only makes it seeable.**
   **ANSWERED — NOT BY A WALK — 2026-07-23 (journal/0079; RATIFIED). The
   erosion budget is not the amplitude lever.** A faithful probe (1× asserted
   byte-identical to shipped `build_field`) swept `--erosion-budget` 1×/3×/10×/30×
   on the client world: relief **1287 m at every budget**, mean |Δsurf| vs 1×
   climbing only to **0.35 m at 30×**, strongest hard-bed contact gaining 3.5 m,
   lithologic separation unmoved. A mechanism probe pinned *why*: the surface is
   **graded to base level** — mean lowering **41 m (max 1.4 km), flat across the
   whole sweep** — so it is neither supply-limited (it erodes a lot) nor
   iteration-starved (flat, not growing); the rate only sets approach-to-grade,
   capped by `inc_pot.min(room).min(max_inc)` (`erosion.rs:1149`). Weathering ×30
   thickens regolith 5→13 m but does not lower the surface. This is the third leg
   with journal/0040 (tectonic amplitude bought no relief) and S13 (5 % of
   roughness reaches the ground; the summit is a genuine plateau): **relief is
   bottlenecked on the GENERATING side — deep-field elevation structure + uplift —
   not erosion.** Cranking erosion is pushing on a rope. `--erosion-budget` stays
   a dev tool (a faithful lever on regolith / approach-to-grade). **Cause 3
   "conservative amplitude" is REFRAMED, not retired: it is a deep-field
   relief-generation problem** (couples to S13's owed 100–500 m landform band and
   the plateau) — its own design pass, Sequenced below. Instruments committed:
   `examples/amplitude_tour.rs`, `examples/amplitude_mechanism.rs`.
   corrections #41. **Nothing further to ratify on the erosion axis.**
3. **The four rate coefficients** in `resistance_of_material` (how smash /
   solubility / permeability / cohesion map to each agent's resistance) and the
   contrast/clamp defaults ride **plausible-not-tuned**, same status as S9's
   physics constants and S10's rate constants (no-bandaid). Engineering, rides
   as-built unless the user wants a different look.

Original charter (for the record):
The gap was `erosion.rs` incising bedrock with a single global `k_bedrock` —
granite and mudstone eroded identically, so the world had no differential
erosion anywhere. The data already existed (the recorder knew the exposed unit
per cell per epoch); erosion never asked. Design notes honoured: agent-specific
resistance (NOT a scalar — the limestone cliffs-vs-caves trap); composition order
with `resist`/`wmult` stated; basement/shield prediction checked and holds;
feedback stability clamped; off-by-default and byte-identical.

**Far-field range knobs** (user-requested 2026-07-20 at the FF2a
ratification): expose the draw-distance geometry as adjustable settings —
`FULL_DETAIL_RADIUS_M`, the ring edges, `FAR_MAX_M` (today compile-time
constants in farmesh.rs) — so the user can push the horizon and feel the
FF2a scale-checkpoint numbers (~10 km ≈ 376 tiles / ~43 MiB, meshing stays
budget-bounded) instead of reading them. Small; a dev-console/config
surface question more than a rendering one; the ring-membership hysteresis
and coverage tests must hold at any setting.

**Sim light — what the voxels know** (user-sequenced 2026-07-20.
**DESIGN PASS DONE 2026-07-20 → `docs/design/light.md`**, which is now the
doc of record; visuals.md holds the dated DECIDED entries it consolidates.
**Next step is the SPIKE**, per the ratified shape design → spike →
milestone; § 10 of light.md states exactly what the spike must measure.)
This is a new sim field, not a renderer feature, and deliberately NOT
PBR-2.

- **What it is**: a deterministic, seed-driven, propagated per-voxel light
  level — Minecraft-blocklight-grade, "the simulation level of our light,
  not crisp dynamic shadows etc, just what the voxels know" (user).
- **Why it unlocks things**: it is the missing axis for photosynthesis and
  plant growth at the collapse tier (S10's biology gates on moisture and
  temperature but has no light term, and it is what makes caves lightless
  *in the simulation*); for creature/spawn behaviour; for stealth and NPC
  vision; and for **embodied-agent perception parity** — an MCP-driven
  character must not see better than a human player (visuals.md). It is
  also what makes "darkness is a gameplay material" true rather than
  decorative.
- **Renderer light and sim light are two different things with two
  different consumers, neither derived from the other** (DECIDED,
  visuals.md). The shader may do shadows/HDR/godrays; the sim carries a
  coarse level. They need not agree.
- **PRIOR — S3 already wrote the skylight query contract** (S3-results.md
  § Skylight query contract; also `column.rs` module docs). It solves the
  cubic-chunk problem (a cube chunk cannot know what is above it) without
  vertical scans: (1) a sky/light query consults ONLY resident chunks and
  cached column summaries — it never loads, generates, or walks "the
  column above"; (2) every query is bounded to a caller-supplied
  `[y_min, y_max)`, and no unbounded vertical scan exists anywhere;
  (3) **unknown volumes are non-occluding — "optimistic sky" — and poison
  the answer's `fully_resolved` flag**, and a consumer needing certainty
  schedules LOD derivation rather than forcing loads from inside a light
  query; (4) answers carry the resolution they were derived at. Column
  summaries build lazily at ~20 µs, so they are effectively free.
  **S3 OQ 6 explicitly defers a decision to this work**: "optimistic sky
  is a policy, not a truth... a consumer that ignores the flag will light
  caves as if open to sky until data arrives. **The lighting spike must
  decide re-light-on-load.**" That decision is this milestone's to make.
- **Open design questions** (answer in the design pass): resolution (per
  voxel? per column?); whether sky light and block light are separate
  channels (Minecraft separates them so a day/night cycle need not
  re-propagate block light — likely the same reason applies here);
  propagation/update cost on edits, which is the same dirty-rail shape as
  remeshing and collider tiles; and persistence vs re-derivation.
- **Noted structural parallel** (assistant, unproven): a propagated light
  field and water.md's bound-water saturation field are the same
  computational shape — a bounded local relaxation over the voxel grid
  attenuated by a per-material property (opacity vs permeability), with
  sources and sinks. S11 measured that relaxation as genuinely local
  (4–11 cell halo); the machinery may serve both. Check before building
  either twice.
- Per the knob doctrine: any propagation radius/level count ships as a
  knob, not a baked constant.
- **SHAPE RATIFIED 2026-07-20 (user: "go with your suggested shape"),
  mirroring how water went**: (1) a **design pass** producing the doc —
  including the heavenly-body *path* abstraction; (2) a **spike** measuring
  derive-per-chunk cost and proving the relaxation's order-independence by
  construction; (3) the **milestone**. No implementation before the doc.
- **Day/night needs no separate system** — it emerges from body paths, so
  the ephemeris IS the path primitive (user, 2026-07-20). Paths must be
  parameterised over world time AND observer latitude, with a long cycle
  that changes the path itself, so **seasons, poles and tropics can land
  later without a rewrite** — build the path capability now, not the
  features. `lat_deg` already exists on pregen cells.
- **Block light is unblocked from the item system** by a **dev-light block**
  (user): a point-light emitter that never spawns naturally, so propagation
  can be built and tested before torches are items. Plant light budgets
  deferred.
- Determinism requirement for the spike: light is a **max-plus relaxation**
  (`light(v) = max(emission, max over neighbours(light(n) − attenuation))`),
  monotone with a unique fixpoint, so run-to-convergence is
  order-independent **by construction**. An ordered flood-fill queue would
  diverge under replay/parallel — same family as S9b and S11.
- Note in favour of deriving: a stored model needs *un-propagation* when a
  source is removed (the classic stuck-light bug class in MC-likes); a
  derived model has no removal path at all — re-derive the halo and it is
  correct.

**PBR-2 — lit-world completion** *(**DEFERRED by the user 2026-07-20** — not
next, despite wanting it: "although i want this: it's going to gum up the
pipeline for the actual game and gen stuff. so not yet on pbr2." The
integrator's earlier "PBR-2 gates whether the underground is lookable at"
claim was based on a misreading of the 0027 walk and is **withdrawn** — the
underground IS currently lookable at; one face orientation is black
everywhere, surface and depth alike, and depth attenuates nothing. The
darkness TARGET is DECIDED — visuals.md: real darkness underground,
reference modded Minecraft with shaders — but its renderer half waits.
**SIM light is sequenced ahead of it** (below): it unlocks gameplay, PBR-2
polishes appearance.)* (the deferred half of the renderer, opened by
PBR-1 shipping): sun **shadows** (Bevy cascades + our knobs, the chasm-shaft
signature shot), **colored point lights** (lava/forge/bioluminescence via Bevy's
clustered path), **tonemap/HDR** (`Camera::hdr` + exposure — real darkness and
earned firelight), **POM** (the LabPBR height channel is already packed),
**light shafts/godrays**, **water**, and **weather/wetness** coupling
(sim-driven porosity darkening — the specular B channel is already sampled).
The `opaque.terrain` shader-pack HOOK (PIPELINE.md § 5, format ≥ 1) also lands
here: PBR-1's material follows the § 5 texture + vertex-attribute contract but is
not yet runtime-overridable via a pack (needs a `HOOK_FORMAT` bump + prelude
composition, like the `post` stage). Never: GI (doctrine).

**S10 follow-through — the biotic layer's five open calls** *(the spike
itself SHIPPED 2026-07-20, journal/0025 + docs/spikes/S10-results.md; it
answered the gate question — all four signals present and legible, cost
+10 s once and +6 MiB kept. What remains is **user decisions**, and one
implementation slice.)*

**NEEDS RATIFICATION (user-owned, from S10-results.md § Recommendation):**
1. ~~**The world-creation ritual grows 15 s → 25 s (+66 %)**~~ — **RATIFIED
   2026-07-20, and the number was pessimistic**: measured on the shipped path
   the ritual is **13.79 s** (biology's marginal cost +2.6 s, not +10 s). The
   25 s figure came from the spike harness's *scalar* driver; production takes
   S9b's byte-identical parallel path — **corrections #12**, journal/0026.
2. ~~**Flipping `production_config`'s `biotic` to true**~~ — **DONE**
   2026-07-20 (journal/0026). Biology runs in every new world; the `DeepField`
   changed for every world created from here.
3. **Signal abundance is aesthetic, not correctness**: coal 0.55 % of
   columns, paleosols 22.9 %, charcoal 20.2 %, retrogression 23.5 %. Is a
   fifth of the world carrying a fire record the texture we want?
   *(Amended 2026-07-20, journal/0026 — the question is now narrower than it
   looked. Measured against the 0.9 m voxel, the **fire record is entirely
   invisible** to a player (0 of 158 310 beds survive quantization), so the
   "fifth of the world burning" texture does not reach the eye at all today.
   What DOES reach the eye is organic **soil**: 185 km of surviving thickness,
   making carbonaceous mudstone the second most abundant rock in cut faces.
   That, not charcoal, is the appearance call to make.)*
4. **The 7-species roster and ~25 rate constants** ride as
   plausible-not-tuned (no-bandaid: they are the mechanism's calibration,
   not a patch) — same status as S9's physics constants.
5. **Pack-addition blast radius** (ecology.md § 5 fork 2) **has stopped
   being hypothetical**: biology is now demonstrably an erosion term, so
   adding an organism pack would change *terrain*, which materials packs
   never did. The doc filed this as "needs deciding before biology couples
   to erosion" — it now does. Options unchanged: bake biotic erosion at
   world creation (pack-add affects ungenerated regions only, matching the
   DF-like seed policy) or accept landform drift.

*(**Collapse-tier organic materials + the production flip: SHIPPED**
2026-07-20, journal/0026 — see Shipped. The `Biofacies` → class routing is in,
coal/peat/carbonaceous-mudstone exist, and the 24 m seam at world voxel
(107338, 58787) is diggable. Two follow-ons fell out of it, both
measurement-backed:)*

**Charcoal as an inclusion, not a band** (journal/0026, measured): the fire
record is the third most numerous facies (158 310 beds) and **none of it
survives the collapse tier** — mean bed ~3.5 cm against a 0.9 m voxel, 0 of
158 310 kept. A charcoal *band* is therefore impossible at this voxel scale, so
no charcoal material was shipped. The honest representation is the one geology.md
§ inclusions already ratified: a few dark eighths riding inside the host stratum
above the burn, exactly as the placer puts gold in gravel and 3d puts olivine in
basalt. Blocked on a mechanism, not a decision — `deposit_deep_history` currently
*drops* sub-voxel units, so this needs a redistribute-into-host rule that touches
every dropped unit (mineral ones included) plus an inclusion channel on
`StrataEvent` distinct from the placer's `ore`. Its own slice, with its own
invariant work.

**Coal rank when burial deepens** (journal/0026): `dc:stratum/organic-coal`
ships one member because rank transitions (lignite → sub-bituminous →
bituminous → anthracite) live at ~1–2 km of burial and our deepest recorded
overburden is ~100 m — a rank ladder now would be a discriminator on an axis the
data never visits. The class's **depth axis is the rank axis** and the
class-share invariant means members can be added later without moving an
existing seam. Revisit when the record carries kilometres (3e-later / thicker
basins).

Also filed from S10 (not blocking): parent-material phosphorus from pregen
provenance instead of a uniform pool; individual plant placement from the
community vector by addressed hashing (ecology.md's derivation-chain
terminus — note the community vector is currently **dropped** after the run,
so C-refinement would have to re-derive it); vegetation → channel planform
("vegetation invented meandering rivers") needs 3e-2's finer corridor to
have any planform to bend.

**3e-2 — C refinement** (DECIDED 2026-07-19 — earth-processes.md § 3e-2
decisions — and implementable): drainage coarse-at-A with the
river-conditioning mechanism (corridor wander toward refined lows +
descent-along-flow as hard constraint, no divide-crossing); width cap
permanent; 0015-mechanism elevation stitch + interior-commit records over
the 16–24-cell halo; proximity approach trigger, order-independent by
construction; **contact-softening in scope per method rule 5** (no
grid/analytic boundary reaches the eye); calibration RATIFIED — the
Phanerozoic register (~500 Myr recorded span, basement ages procedural
— "procedural hacks for the boring billion"; knob deferred). **Nothing
open — implementable.**

**S11 follow-through — the water model's four open calls** *(the spike itself
is SPIKE COMPLETE 2026-07-20, journal/0028 + docs/spikes/S11-results.md; it
answered the gate question — bound water's halo is 4–11 cells and the body
graph does not grow with edits. What remains is **user decisions**.)*

**NEEDS RATIFICATION (user-owned, from S11-results.md § Recommendation):**
1. **The two body characters — `Finite` vs `Pinned` — as a design commitment.**
   This is the load-bearing one and it is not a performance question. Pinning
   is the claim that *some water has a level set by the world rather than by
   its own volume*. The measurement says the distinction is necessary (a finite
   sea drops 13.18 m when breached into a large void) and that pinning is also
   ~415 ms cheaper per resolve. Seas and fed river reaches obviously; **where
   the boundary sits — a big lake? a spring-fed pool? — is the user's call.**
2. **The ~12-cell bound-water derivation halo**, if it becomes a constant. Wide
   margin over everything measured, but it is calibration on a model whose rate
   constants are plausible-not-tuned — same status as S9's physics constants and
   S10's rate constants (no-bandaid: rides as measured).
3. **Body identity is not stable across a merge** (S11 design choice 2): bodies
   in one air component merge, lowest id surviving. If lakes are ever to be
   named, findable, or referred to by quest/ledger state, that needs deciding
   **before** the graph ships.
4. **Whether the connectivity index is truly never persisted.** The spike
   asserts it is derived and proves reload identity from it (6 ms rebuild for a
   1.77 M-voxel world), but at real world scale that becomes a streaming cost
   nobody has measured. The alternative — persist it beside the S3 region files
   — is the *same* open question the far-field summary store carries, and the
   two should probably be answered together.

*(Not decided by S11 and still open in water.md: the bulk-flow octree — though
note the spike found free water in equilibrium is **static data with a level**,
which is what the notebook's "creates no new blocks so long as its outlet
connects" predicted; sub-resolution water; capillary action; which cave family
ships first; and where the deep-time water field lives.)*

**Water-model design pass** (ratified 2026-07-19, user; field-notebook
first per the earth-processes method): groundwater as "another dimension
for the flow to go" — water table / aquifers (S8 per-voxel porosity is
the waiting substrate), ponds and sub-resolution water (procedural-tricks
tail), visible/flowing water (couples to PBR-2 water), lakes/inland seas
already implicit as deep-tier flooded basins (spill levels known).
**Groundwater ↔ CAVES coupling flagged by the user** — speleogenesis as
the eventual cave story (today's caves are S1 noise carving). Design doc
before any code.

3a. Form archetypes + drop distributions (materials.md § forms) — implement
   with the first inventory/interaction milestone.
3c-2. Geology client integration, material tier (visuals DECIDED
   2026-07-19, visuals.md § mixture road): sidecar channel through the
   seam + interim world-anchored dither materialization (registry albedos,
   eighths-weighted), partial-height loose-material rendering with binary
   threshold collider; subtle ore. Runs after body steps 1–2. Watch the
   order-dependent MixtureTable ids at the seam (journal/0008).
   **(SUPERSEDED 2026-07-22 by block-is-material):** there is no separate block
   registry — block IS material (one namespace), so the atlas/enum is fed by
   **material definitions**, never a `define block_type`. A1 (journal/0087) already
   routes the render through material identity and deleted the block twin.
3d. Geology post-v1 slice (geology.md §§ formation-context +
   roster/inclusions/slots, DECIDED 2026-07-19): strip climate windows from
   igneous class sheets (formation-context shim correction — fitness goes
   province/depth-driven); chunk-line family cutover diagnosis + smoothing
   (context interpolation / boundary dither); two-layer class-satisfiability
   enforcement in pipeline validation; accessory inclusions as pore
   partials; then the rich vanilla mineral roster. Carbonate follows.
3e. Epoch-indexed formation context (the deep-time turn): paleo-context
   axes from pregen history feeding deposition/metamorphism; igneous from
   tectonic setting + emplacement depth. Orogeny-repo recon 2026-07-19 as
   idea quarry; couples to S2 checkpoint-facts design (Observed).
4/5. **Ecology + organisms** — designed 2026-07-19, docs/design/ecology.md
   (supersedes the old biomes/ecology seeds and the agenda doc): species
   is the primitive, biome is a diagnosis; the six-process biotic layer on
   the deep-time tier; biology as an erosion/weathering/rock-forming term;
   a "biome pack" is an organism pack. Evolution recorded as the user's
   teleology design (org defs pin a form at a horizon; the lineage between
   is simulated; ahistorical override; fossils = mid-horizon pins) —
   post-v1, but v1 must not foreclose it.
   - *(**S10 biotic-layer spike — the gate — PASSED** 2026-07-20,
     journal/0025 + docs/spikes/S10-results.md: community vector + all six
     processes on the A-tier, all four read-quality signals present and
     legible, cost +10 s once / +6 MiB kept, determinism intact. Evolution
     stayed out of scope as specified. The substrate evolution will ride is
     proven; the five ratification calls are above.)*
6. Body plans implementation staircase (docs/design/bodies.md § staircase,
   ratified 2026-07-19) — six steps from joint-tree skeleton to authoring
   editor; not yet scheduled against the geology track.

## Observed (undiagnosed or deliberately unfixed)

- **OWED / next residency lever — a per-cell OWNING CONTAINER is a header × 297,025 before it
  stores anything** (journal/0100, 2026-07-25). The CSR conversion cut the ledger heap 9.5×, but
  the per-cell `FactLedger` **struct** grew 24 → 48 B (two `Vec` headers per cell) = **13.60 MiB
  paid whether or not a cell has a single fact** — an honest regression the slice flagged itself.
  The full `flux.rs` shape collapses it: **one record for the whole grid, with the cell as a CSR
  row**. Deferred only because it moves `DeepField::ledgers` and `ledger_at_voxel`, which sat in a
  live sibling's write-set. **The generalisation is the valuable part and applies far beyond this
  struct:** *any per-cell owning container in a 297 k-cell field costs a header per cell before it
  holds data* — so the default for anything per-cell is **one grid-wide record with CSR rows**,
  never `Vec<Something>` per cell. Same family as the `Vec<Vec<Fact>>` defect, one level up.

- **✅ DIAGNOSED 2026-07-25 (journal/0103) — the front's voxel-tier mass error is NOISE, and the
  `+3.7 %` was the INSTRUMENT** (opened by the integrator's review of journal/0099; corrections
  #50). Re-run over **247 columns** (journal/0099 had 21), with the pipeline split into its **two
  quantizers** — the fill geometry and the eighth draw are not the same kind of error and only the
  second is an estimator — and with the census taught to exclude voxels it cannot attribute.
  - **Stage 1, record → fill geometry: aggregate −0.02 %, mean +0.03 %, median +0.00 %, p5 −1.22 %,
    p95 +0.86 %.** The record-bottom round-to-nearest, which lands on the front *every time*
    because the front is what sits at the record's bottom, is centred.
  - **Stage 2, the draw over attributable voxels: aggregate −0.60 %, mean −0.10 %, median −0.00 %,
    p5 −30.00 %, p95 +28.72 %** (Mixed-plan voxels only, the only ones carrying a draw: −0.84 %).
    Symmetric about zero, spread **widening** as the front thins — which is what an unbiased
    estimator over a one-eighth quantum does, not a floor.
  - **Stratified by front magnitude the fake trend disappears.** Naive means run +76 % (2–4
    eighths) → +31 % (4–8) → +11 % (8–24) → +4 % (≥24), a textbook floor-effect curve. Stage-2
    means run +9.1 % → +2.9 % → −0.1 % → −0.7 %, no trend. The naive decay was **the contact
    voxel's share of the front shrinking as the front grows**, not quantization.
  - **The mechanism.** The product is `CLASS_CLASTIC_FINE` (mudstone) and so is much of the pile
    directly above it; at the top contact they share a `Mixed` voxel and **voxel contents carry no
    provenance**. **222 of 247** columns have a front voxel whose plan holds a non-front event made
    of the product's own material, so a material census credited the neighbour's mudstone to the
    front.
  - **From the code** (asked for separately, and it holds independently of the data):
    `fill::allocate_to` is systematic sampling / Cranley–Patterson with `P(extra) = remainder`
    exactly; `allocate_partial` floors the **cumulative**, so errors cancel along the run;
    `pore_rider_share`'s mean is exactly `cnt·k8/8`. **Genuinely stochastic-proportional; nothing
    rounds up at the floor.** journal/0055's doctrine is **confirmed, not falsified**.
  - **flow.md § 3's mass budget can build on this** — with one requirement that is the real
    deliverable: *a conservation audit at the voxel tier must compare against the **fill plan**,
    never against a census of the finished contents.* The plan knows which event owns which
    fraction of which voxel; the voxel does not.
  - **NOT fixed, and nothing to fix** — the expression is unbiased. **Residual, stated:** 222 of
    2 077 front voxels (10.7 %) are unattributable and are systematically the *top contact*, not a
    random tenth; this instrument cannot weigh them. The verdict rests on stage 1 covering 100 % of
    voxels and on the code-level proof. **Loose end filed:** `pore_rider_share`'s comment claims its
    offset is disjoint from `allocate_partial`'s ("a low digit… not its high bits"); the bits
    overlap (bits 10–12 vs the top 20), so the two draws are correlated. Not a mass defect — each
    is marginally unbiased and the measurement above is the joint case — but the comment is wrong
    and changing the address would move every contact voxel in the world, so it is **the user's
    call**.

- **`pore_rider_share`'s offset is NOT disjoint from `allocate_partial`'s, but its comment says it
  is** (found 2026-07-25 while diagnosing the mass claim, journal/0103; **not a mass defect**).
  `collapse.rs`'s `pore_rider_share` documents its draw as *"a **low digit** of the voxel's own fill
  draw, not its high bits: `allocate_partial` consumes the high end, and reusing it here would
  correlate 'this band won an extra eighth' with 'the product won an extra eighth of it' into a
  visible pattern."* The arithmetic does not deliver that: `(u * 4096.0) as u64 & 7` is **bits
  10–12** of the fraction, and `allocate_to`'s `uq = (u * ONE) as u64` is the **top 20**. They
  overlap, so `cnt` and the rider's offset **are** correlated — exactly the coupling the comment
  says it avoided. **Measured consequence on mass: none detectable** (journal/0103's stage-2 figure
  is the joint case: −0.60 % aggregate, median −0.00 % over 247 columns), because each draw is
  marginally unbiased. The open question is the one the comment actually cared about: whether the
  correlation is **visible** as a pattern at a contact. **Not touched** — re-addressing the rider's
  draw moves every contact voxel in the world, which is an appearance change and the user's call.
  Cheap next step: a fullbright walk along a strong front looking for banding correlated with the
  parent's eighth, before any code moves.

- **A front's parent alternates diorite/granite down a single column** (observed 2026-07-25 by the
  weathering-profile slice; **pre-existing, merely made visible**). `Single` voxels resolve their
  member through the per-voxel-column **member dither** while `Mixed` voxels use the **canonical**
  member, so a column whose voxels alternate plan kinds also alternates rock identity. Invisible
  until a front produced many contact voxels in one column. Same family as the palette-quant
  member-stepping thread (a *within-class member* choice reaching the eye), and it belongs with the
  genesis-passes arc that retires class-member selection.
- **✅ FIXED 2026-07-25 (journal/0101) — the query can now say `UNRECORDED`.** `identify(pos)`
  landed as the arc's first slice: `has_contents` is a **per-voxel** fact, `classified` echoes
  the stored block for an unrecorded voxel instead of naming a mixture that does not exist,
  `character_sense_raycast` answers `None` (its documented promise) instead of `Some(<empty
  view>)`, and the F3 HUD's `(no contents record here)` branch is **reachable**. Census through
  the real query path (`dc-client/examples/identify_census.rs`, same lattice as the diagnosis
  probe): phantom air **702 → 0** of 10 985 solid voxels (6.4 % → 0.0 %), 39/169 columns →
  0; every phantom voxel converted to `UNRECORDED` (4 913 = 702 + the 4 211 already honest),
  recorded mixtures (6 072) and sky unmoved. At journal/0097's own station the band
  288–299 now reads `UNRECORDED` and **agrees with 287**, which the chunk floor used to
  split. **Zero
  dc-worldgen change** — the stored `Block` already disambiguates. *Entry kept, marked, because
  the numbers below are the measured baseline.*

- **DIAGNOSED 2026-07-25 — `world_get_contents` reports `dc:air` and `has_contents: true` over
  solid, correctly-unrecorded rock** (walk observation journal/0097; diagnosis
  `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`, probe
  `dc-worldgen/examples/contents_air_over_solid_probe.rs`). **The original premise is falsified**
  (corrections #49): the world was never empty there, and **`eye_in_solid` was the honest
  instrument** — it and the query's own `block` field read the same `block_at` and both said
  `dc:stone`. Unrecorded basement is `Block::Stone` by construction (`collapse.rs:457-459`) and no
  voxel below a column's height can be `Block::Air` at all (`collapse.rs:434-436`).
  **Mechanism — a dc-api query-surface defect:** `contents_at` answers a **per-voxel** question
  with a **per-chunk** presence test (`host.rs:320-326`), so an *unrecorded* basement voxel sharing
  a 32³ chunk with any recorded voxel returns `Some(VoxelContents::EMPTY)` — reported as
  `has_contents: true` (`host.rs:83`, contradicting `payload.rs:426-429` / `schema.rs:754-756`)
  with `classified: dc:air` (`host.rs:76-79`, the operation `classify.rs:31-45` explicitly
  forbids). Reproduced **to the voxel** at journal/0097's own station: phantom band 288–299, honest
  from 287 down — the transition is the **chunk floor `9×32`**, not anything in the world.
  **Global:** 702/10,985 solid voxels (**6.4 %**) across 169 columns; 39/169 columns affected;
  worldgen authority only. **Blast radius:** the F3 HUD (`inspector.rs:125-133`) and
  `character_sense_raycast` (`host.rs:1424-1426`) carry it identically — their correct "no contents
  record here" branch is **unreachable** in this case; the mesher (`meshing.rs:295-306`) and far
  field (`farfield.rs:134-152`) are **immune**, which is why only the *diagnostic* surfaces ever
  showed it. **NOT the bare-cell fallback** — that is a real thin-record *generation* artifact;
  this is a *reporting* artifact over a healthy record. Both entries stay.
  **OWED — and it is the `identify(pos)` arc's first concrete requirement:** a tier flag that can
  say **"unrecorded"** as a first-class answer, distinct from both "air" and "recorded". Not fixed
  here (diagnosis-only agent).

- **A column expressing ONE recorded voxel over 64 unrecorded** (measured 2026-07-25 by the
  contents probe at journal/0097's station: `dc:peat` at `y = 300`, unrecorded basement below).
  The **thin-record** family — one notch less extreme than the bare-cell zero below, and unlike the
  reporting defect above this one **is genuinely about the world**. Undiagnosed; likely the same
  root as bare-cell (sub-voxel record thickness falling off the record path).

- **`world_scan_region` / `world_get_block` are structurally blind to stratigraphy** (walk
  observation, 2026-07-25, journal/0097). Both answer with the stored **1-byte `Block` summary**,
  which collapses mudstone, sandstone, siltstone, carbonaceous mudstone and granite alike into
  `dc:stone` — so a column scan from veneer to basement returned `palette: ["dc:stone"]` and the
  walk would have concluded *"no band"* from an instrument that **cannot see one**. `world_get_contents`
  is the honest surface and showed all of it. This is ARCHITECTURE.md's *"a summary is not an
  authority"* **caught in the field, by an agent using the tool wrong** — and it is a live argument
  for the Sequenced **HONEST IDENTITY SURFACE / `identify(pos)`** arc: a tier-flagged answer is what
  a walk needs, and the untiered `Block` token is a trap for exactly the reader who does not already
  know it is one. *(CLAUDE.md § Agent walks updated in the same commit: use `world_get_contents`,
  never `scan_region`, for any material question.)*

- **✅ DONE 2026-07-25 — FREE WIN TAKEN: 54.02 MiB reclaimed, 33.2 % of all `DeepField`
  residency, zero behaviour change.** `build_field` now `shrink_to_fit`s every cell's
  `units` Vec at the end of the compile. **Measured before → after: 162.57 MiB → 108.55
  MiB**, slack 54.02 MiB → **0.00 MiB (0 %)**, `DeepField::resident_bytes` agreeing;
  asserted by `strata_is_shrunk_to_fit_after_the_compile` (capacity == len for every
  cell, so the reclaim is enforced, not merely intended). The one-time copy is gen-time,
  therefore free. *Note: `docs/spikes/S19-*` quotes the pre-fix 162.57 MiB baseline —
  that figure is now historical; today's total is 108.55 MiB, which is the correct
  denominator for the flow-record projections (every `× today` multiple in S19 is
  correspondingly ~1.5× larger against the new baseline).* The sibling defect — the
  `FactLedger` `Vec<Vec<Fact>>` at **98.8 % empty inner Vecs / 89 % of its heap in empty
  headers (+156.91 MiB when `weather_inventory` is on)** — is NOT fixed by this and
  remains the reason the flow record must be sparse/CSR.

  *Original entry:* **`strata` Vec capacity doubling wastes 54.02 MiB: 39 % of the record heap
  and 33 % of ALL current `DeepField` residency** (measured 2026-07-25,
  `docs/spikes/S19-flow-record-cost-results.md`). Live 84.47 MiB vs capacity 138.49 MiB
  on a production world (seed 1337, Medium). Pure allocator slack from growth doubling —
  the record is built once per world and then read-only, so a `shrink_to_fit` (or an
  exact-size second pass / arena) at the end of the deep-time compile should recover most
  of it with **no behaviour change and no content cost**. This is the cheapest residency
  win on the board and it is worth taking before the flow record adds a second large
  store. **Runtime perf is first-class and this is pure waste** — but measure the actual
  reclaim (and the one-time copy cost, which is gen-time, therefore free) rather than
  assuming. *Related, same measurement:* `FactLedger`'s `Vec<Vec<Fact>>` is **98.8 % empty
  inner Vecs, 89 % of its heap empty headers** (+156.91 MiB when `weather_inventory` is
  on) — the same allocation-shape defect one level up, and the reason the flow record must
  be sparse/CSR rather than per-(cell,slot) Vecs.

- **`derive_regolith_at` derives `H` from an EMPTY ledger — Movement 2a's derived views
  cannot see `dc:deep/weather_inventory`'s facts** (spine-audit 2026-07-24, post-M3).
  `DeepField::derive_regolith_at` (`deeptime/field.rs:545`) builds its working inventory from
  `FactLedger::empty_with_bedrock(strata)`, so the "inventory is the authority; `R`/`H` are its
  materialized views" claim (journal/0092) currently holds for the **record half only** — the
  derived `H` structurally cannot see the **6.09 m of `Loose`** the M3 weathering process commits
  per epoch. This is the **concrete content of the R/H-unification residual** left by stub #17's
  discharge (the deferred half of its heir sentence). Not a defect *of* M3 — the two-authorities
  split was the briefed design — but it is the thing Movement 2 must actually close, and it is
  the honest reason "one authority" is not yet true. **Tracked:** ROADMAP Movement 2 /
  material-behavior.md §11 continuation slot + §13.6 / stubs.md #17 Residual.

- **Two declaration defects on the new `dc:deep/weather_inventory` pass** (spine-audit
  2026-07-24; both free to fix, neither affects production — the flag is off by default).
  **(a) `reads_prev: &[BioMod]` is not what happens.** The pass reads `grid.bio_weather`, which
  `dc:deep/biotic` overwrites **in place** each epoch; no edge is declared against `biotic`, so
  which epoch's plane it sees is decided by `passgraph`'s id-lexicographic tie-break
  (`dc:deep/biotic` < `dc:deep/weather_inventory` ⇒ it reads **this** epoch's, not last's).
  *Rename the pass and the physics changes* — the declaration is a fiction the graph does not
  enforce. Honest fix is free: move `BioMod` into within-epoch `reads` (adds only
  `biotic → weather_inventory`; nothing reads `Saprolite`, so no cycle). Note
  `weather_inventory_is_absent_off_and_a_declared_cellular_pass_on` asserts the **declaration**,
  not the behaviour — green about the wrong thing (**A-3** shape). **(b) `Exposed` is declared
  but never read** (susceptibility is a constant off `BEDROCK_SEAM_MATERIAL` until stub #16's
  genesis heir lands); the comment claiming it reads "the exposed lithology" is untrue today.

- **STATION — per-chunk material-palette quantization makes the chunk grid a
  visible checkerboard** (user field report + diagnostic station, 2026-07-24;
  `journal/assets/0088-palette-quantization-chunk-seams.png`).
  **THE REFERENCE STATION — EXACT POSE, recorded 2026-07-24 (user-recovered).** This is
  the standing reference point for the palette-quantization issue; re-shoot it to compare
  before/after any fix:
  > **feet (world m): `x 71291.7, y 372.1, z -2420.9`** · voxel `79213, 413, -2690` ·
  > **`yaw 21.9968`, `pitch -1.5475`** (looking ~straight down) · fly on.
  > Re-shot frame: `journal/assets/0088-palette-quant-reference-station.png` (matches the
  > original capture: tan/grey/red-brown/dark-speckled squares, yellow inclusion flecks).
  **CORRECTION (2026-07-24):** this station was previously described only as *"the east
  coast, ~110 km east of spawn"* — that distance was **wrong by ~39 km** (the true spot is
  **≈71 km east**, `x ≈ +71.3 km`), and the prose landmark sent a reconstruction attempt
  into grey single-class coastline at 108–110 km, where the checkerboard is absent. The
  pose was never recorded at capture time (the diagnosis audit § 7 even listed "record the
  camera pose" as an unfilled needs-a-live-probe item). *A prose landmark is not a pose;
  "defer = write it now" applies to camera poses too.* The user noted the weathering regime
  here reads visibly different from the plateau spawn — worth its own look.
  A top-down view shows the
  near/mid field as a **grid of chunk-sized squares, each a distinctly different
  overall tint** (tan / grey / red-brown / dark-speckled), while the **structure is
  continuous across the seams** — diagonal dune/wave bands of structure blocks (with
  yellow inclusion flecks) cross chunk boundaries unbroken, dithered mixed loose
  voxels between them. So the weathering/dune *pattern is continuous* across seams
  but the **materials are discontinuous** — an **S-4-square violation on the MATERIAL
  axis** (a chunk-resolution edge reaching the eye as a square; distinct from the
  far-field LOD material-identity split below, S-9, and the "far field boxier"
  geometry thread). **Two theories, to be measured:** (T1) the material *families*
  genuinely differ per chunk — same kind of mix, but a dominant family swapped
  (sandstone here, mudstone there) — a **generation-side** per-chunk material
  selection. (T2) the full contents vary *smoothly* (same materials, shifting
  weights) but voxels hold **more than the 4 materials a mixture can splat**, and the
  visible **winner reduction (>4 → 4) is salted per-CHUNK instead of per-voxel-
  position**, so the survivors are uniform within a chunk instead of jittered — a
  **render/materialization** bug (the >4→4 cut is the likely locus; journal/0055
  world-anchored dither, 0008 seam-order `MixtureTable` ids, the combinatorial mixture
  cap). **The discriminating measurement:** sample the *full* top-layer contents (all
  materials + weights, NOT the classified winner) across adjacent chunks at this
  station — smoothly-shared materials with jumping winners ⇒ T2; genuinely different
  material sets ⇒ T1. **This needs full-contents access, which the live
  `get_block`/`scan_region` do NOT expose** (they return the classified winner only) —
  see the look-at-contents dev slice below. **DIAGNOSED 2026-07-24**
  (`docs/audits/2026-07-24-palette-quant-generation-diagnosis.md`): **T1 confirmed, and
  it is the SAME ROOT CAUSE as the haunted-LOD blobs.** A chunk's entire strata
  composition is built from a **single point-sample of the deep record at the chunk
  centre** (`record_at_voxel(cx*32+16, cz*32+16)`, `collapse.rs:1474`), NEAREST at the
  **~460 m deep-cell grid** (`DEEP_CELL_M`); `run_strata` shares that one sample across
  all 1024 columns. So composition tiles are **~460 m deep cells with boundaries
  chunk-quantized to 28.8 m** (a staircase); climate/elevation/member-dither/mesher are
  all smooth/per-voxel — hence *continuous pattern, jumping composition*. The far LOD
  blobs are the **same coarse ~460 m NEAREST deep-facies field point-sampled instead of
  interpolated/area-summarized**, at a different site (near `record_at_voxel`→
  `run_strata`; far `surface_class`→`draw_class`) — one phenomenon family, two tiers.
  It is the **S-4 rule unmet**. **One live-probe question left:** are the visible squares
  the 460 m deep cells or the 28.8 m chunk staircase — settle by reading dominant surface
  material per chunk across an east-coast patch (`world_get_block` / the inbound
  inspector: change every 28.8 m ⇒ member stepping; only at ~460 m ⇒ deep-cell tiles).
  **Planning held.** **DIAGNOSTIC STATION — return here to
  check any fix:** feet `pos {x: 71291.7, y: 372.1, z: -2420.9}` / voxel
  `{x: 79213, y: 413, z: -2690}`, `yaw 21.9968`, `pitch -1.5475` (looking ~straight
  down), fly on. A correct fix dissolves the chunk-square tint grid into continuous
  ground with the diagonal structure bands unbroken. **Diagnose before touching.**

- **Dev slice — look-at-voxel contents inspector — SHIPPED 2026-07-24**
  (`f594580`, journal/0088; `world_get_contents` + `character_sense_raycast` contents
  + F3 HUD; `ContentsSource` seam re-derives contents since the runtime stores only
  `Block`). *(Original ask, for the record:)* (user-requested 2026-07-24;
  enabling the palette-quantization measurement above AND a standing dev tool):
  *"check the contents of a voxel just by looking at it."* Two parts: (a) a
  **full-contents query** returning a voxel's whole `VoxelContents` mixture —
  materials, forms, weights — NOT the classified `Block` (`get_block` returns only
  the winner, which is exactly what hides T1-vs-T2); (b) a **look-at readout** —
  `character_sense_raycast` already returns the hit voxel + face but only the block
  name, so extend it to contents and surface it as an on-screen HUD. Small (dc-api
  query + dc-client HUD/raycast), dual-use: it makes the T1/T2 measurement doable
  live and gives every future material diagnosis a direct instrument. **Sequence it
  ahead of the palette-quantization diagnosis — it is that diagnosis's instrument.**

- **Far-field LOD reconstructs a coarse box's MATERIAL IDENTITY differently on
  cold-gen vs warm-regen** (user field report, 2026-07-23; **mechanism sharpened
  2026-07-23**). The coarse box for an area shows one material on its *first*
  (cold) generation and a *different* one after the area has been resident and its
  LOD is regenerated: the user watched mudstone coarse boxes gain **granite**
  patches on the second gen — **the sub-surface layer falls into the sample on
  warm-reduce but not on cold-synthesize.** So the two producers of the *same*
  coarse box pick a different winning material: **cold-synthesize (surface-sampled)
  vs warm-reduce (`dominant`/`classify` over resident voxels, subsurface
  included)** disagree — the reduce-vs-synthesize discrepancy (journal/0070
  measured a +0.938 coarse voxel bias). This is a **material-identity**
  disagreement, not a resolution/lighting one: it lives in **block-is-material /
  `classify` / `MixtureDownsampleRule`** and belongs with the **far-field span
  migration under Crux 1**. It is a live violation of **spines S-9's consistency
  law** (a collapse/reduction must agree with the base it refines). Couples to the
  albedo-at-range thread. **DIAGNOSED 2026-07-24**
  (`docs/audits/2026-07-24-lod-pre-post-visit-diagnosis.md`): it is ONE path with
  ONE distance gate — `REDUCTION_STANDOFF_M = 176 m` (`farmesh.rs:594`) sits *inside*
  the L1 ring (112–256 m), so L1's inner shell (112→176 m) synthesizes **cold**
  (surface-dither) while every band ≥ 256 m (L2–L4) **warm-reduces** (dominant
  subsurface) — the "band inversion" is **static/deterministic, not an eviction race**.
  Cold blobs = a coherent bilinear field point-sampled at the coarse stride (adjacent
  cells land on different members); post-visit faithful = the reduction sees real
  subsurface strata; the two producers were only ever held to tops/occupancy + class
  agreement, never material identity (the S-9 violation). Root: two unreconciled eras
  (FF2a synth-dither vs FF2b reduce) made *visible* by the 0087 block-collapse; the
  176 m standoff was geometry-motivated and never meant to flip *material*.
  **FIX DIRECTION (user, 2026-07-24):** (1) *never cold over warm-capable data* — the
  standoff is backwards; warm must reach the nearfield edge (loaded chunks → warm, no
  cold shell). (2) Cold's dither must be **replaced with a continuous method** (it is
  discontinuous with both the real chunks and the warm band). (3) The deeper truth:
  **both warm and cold fail to represent MIXTURE** — the real fix is the
  **blended-mixture-albedo cascade**: textured mixture (near) → averaged mixture
  albedo, one blended colour per coarsened voxel (mid) → flat winning material (far),
  pure mixed albedo becoming *necessary* the further out the band. This **converges
  the distance-pyramid Sequenced item + the palette-quant station (T1) + the contents
  inspector** into one mixture-representation arc — plan them together. **The hard part
  is representation, not data** (user, 2026-07-24): a blended mixture still comes out as
  **quantized colour patches unless dithered**, and the current dither technique looks
  bad on its own — so "draw the blended mixture" is genuinely hard to make look good,
  and the arc's real difficulty is the dither/blend method, not the plumbing.
  Implementation held pending that plan.

- **Perf: throughput ceiling at terminal velocity** (user field report,
  2026-07-23, on the 0083/0084 offload). Noticeably improved — the drop reaches
  the choking point *later* — but at terminal velocity it **still chokes, about as
  hard** once it does. So the meshing offload raised throughput/delayed onset but
  did not raise the *ceiling*: at max fall speed the streaming pipeline still
  saturates. Next perf targets from here: the `far_tile.snapshot` residue, the
  ~47 % un-instrumented `schedule` self-time (render/transform systems), and the
  streaming budget itself (`LOAD_BUDGET_PER_FRAME`) — but **measure with the
  per-thread-attributed instrument first** (below).

- **The perf instrument can't show the frame-thread envelope** (filed 2026-07-23,
  journal/0084). `PerfAggregate` sums self-time across the frame thread AND the
  task-pool threads, so after the offload the `%` table can't show spans "leaving
  the frame thread" — frame-count is the only clean signal. Owed: **per-thread-role
  attribution** (frame vs task-pool) in the aggregate, which is also exactly what
  the perf/debug overlay heir needs. Do this before the next perf slice so its win
  is directly visible.

- **Async-offload accepted corner — an edit to a chunk in its first ~1–2
  streaming frames meshes the PRE-edit snapshot** (journal/0083, ACCEPTED by the
  user 2026-07-23; must remain VISIBLE). Self-healing: any later edit to that
  chunk or a neighbour re-meshes it. Player edits on a just-appearing chunk are
  rare, but the real exposure is **simulation/NPC edits that fire the instant a
  chunk loads** — whose frequency is unknown until gameplay matures, and which
  will be **extremely hard to notice after the fact** (user: "we can accept it but
  we must know about it"). **Remedy is a DETECTION HOOK, not a doc note** (a note
  is exactly what gets lost): instrument the edit path with a counter on the
  perf/debug surface (journal/0080, the overlay heir) that fires when an edit
  targets an in-flight mesh task — so the corner self-announces the moment
  sim-editing begins, instead of being an invisible latent bug. Instrument-must-
  see-the-question applied to a deferred correctness risk. **Owed: build the hook**
  (small dc-client slice; sequence with or before the first sim-editing feature).

- *(**Deep-cell-square surface-material frontiers checker the far field:
  RESOLVED** 2026-07-22, journal/0073 — the B1 shape-teacher. `surface_class`
  now dithers class membership from the top-window metre shares (S-4 move B), so
  the 460 m class frontier is a statistical gradient, not a razor-straight
  square. The record stays a non-interpolable unit list read NEAREST — the fix
  routed *around* interpolation exactly as spines S-4 flagged, by dithering
  membership rather than bilinear-ing the record. Re-shot at the `0070-*`
  vantages as `0073-*`; verdict in the entry.)*

- **Mesh-buffer pooling: measured, deliberately NOT built** (2026-07-21,
  journal/0051 — the user asked for pooling; this is the numbered answer).
  New churn instrument in the `DC_MEM_PROBE` line: during a teleport storm the
  client builds ~130 near-chunk + ~32 far-tile meshes/second and spends
  **3.5 s of every 10 s (35 % of wall time) inside those builds** — but that
  window is per-voxel compute (32 768 voxels × 6 faces; far tiles dominated by
  coarse-summary sampling), not allocation. Decisive against pooling at this
  seam: Bevy's `Mesh::insert_attribute` **takes ownership** of each attribute
  `Vec` and moves it to the render world, so pooled scratch buffers would have
  to be **copied** in — zero copies per attribute becomes one. Residual idea
  that survives: reuse `MeshData`'s buffers *inside* `mesh_chunk` to kill the
  growth-reallocs — lives in `meshing.rs`, worth at most a couple of percent of
  that 35 %, unowned and unstarted.

- **The far field is now BOXIER than the near field** (user, live walk
  2026-07-21). Partial-height voxels give the near ground sub-voxel height
  variety; the far field is a whole-voxel column summary and cannot express it,
  so the LOD transition now differs in **geometry** as well as detail. Note
  what is and is not guarded: `coarse_surface_matches_near_column_height`
  asserts whole-voxel height agreement and
  `..._surface_block` asserts material agreement — **neither can see a
  sub-voxel height difference**, so this passed every test while being visible
  to the eye. Not yet diagnosed for severity; candidate answers range from
  "carry a partial-height byte in the far summary" to "accept it, the far
  field is a summary". Related: FF2b coarse volumetric summaries.
- **Organics render as solid boxes because form is keyed on CLASS**
  (found 2026-07-21 answering the user's question about why some surfaces
  quantize to boxes and others are partial). `fill::is_loose` returns true only
  for `clastic-fine` / `clastic-coarse` / `ore-placer`; everything else —
  including **organic soil, peat and coal** — goes to the structure bucket, and
  `meshing::height_frac` renders anything that is not loose-only at full
  height. So a carbonaceous-mudstone or peat surface is always a full cube
  while a mudstone surface can be a partial plate. Real soil and peat are
  loose. The user's instinct that this "has to do with form: structure vs
  loose, and provenance of how they got there" is correct **and is already the
  sequenced work**: consume-the-ledger piece (c), *derive material FORM from
  provenance* rather than from class. This is that item's first concrete,
  visible symptom.
- *(**HOLES IN THE GROUND: FIXED** 2026-07-21, journal/0057 + corrections #29.
  Culling is now **occupancy-aware**: the neighbour predicate returns `f32`
  coverage and faces resolve **by span** — emit the band `[cover, frac]`, cull
  only when `cover >= frac`, so exactly one side owns each band and nothing is
  coplanar-doubled. Bottom faces get the mirrored fix. `height_frac` had been
  re-deriving loose-only privately, so the mesher was holding **the very second
  opinion the fill contract exists to prevent**; one `cover_frac` rule now
  serves both the interior path and the cross-chunk closure. Cost **+30 %
  triangles on a synthetic worst case** (randomised loose depth per column) —
  an upper bound, since real depths are spatially correlated and equal-height
  pairs still cull. Seven tests by name including both border cases.
  **UNWALKED** — nobody has seen the holes gone; re-walk
  `0056-holes-after-settle.png`'s coordinates, and if bands persist, distrust
  journal/0057 first.)*
- **Two more assumptions the partials-first world falsified** (journal/0057
  § i, recorded so nobody re-derives them as bugs). **(1)** Partial-height
  expression is now silently conditional on *which block the record produced*:
  a voxel whose block resolved to the 0055 fallback vocabulary (Dirt/Stone)
  renders full height regardless of a sub-8 loose contents value, because
  `block_uses_contents` gates it. Coverage stays consistent on both sides, so
  it is not a hole — but it is a decision nobody made deliberately.
  **(2)** The far field is block-tier and carries no contents, so the horizon
  draws every column full-height while the near field draws partial tops — a
  systematic sub-voxel step at the LOD boundary. Same root as the "far field is
  boxier" item; the distance-pyramid slice is where both get answered.
- **HOLES IN THE GROUND — partial voxels are missing side faces** (found in
  the live walk 2026-07-21, **diagnosed by the user**; FIXED — see above).
  Symptom: sky-blue bands straight through the terrain in a rectilinear
  pattern, persistent (identical screenshots 20 s apart — not a streaming
  transient). Assets `0056-nearfar-check-after-3km.png`,
  `0056-holes-after-settle.png`. The user's read, confirmed against the code:
  *"the bands you see are missing side faces. these partials mostly have no
  side faces - some of them do, following no apparent pattern."*
  **Mechanism:** `meshing.rs` culls side faces on a **block-tier boolean**
  (`neighbor_solid: &dyn Fn(..) -> bool`), so a 5/8 partial beside a 3/8
  partial has its whole face culled and the exposed 2/8 band is drawn by
  nobody. Its own header states the assumption that made this safe —
  *"Worldgen does not yet emit sub-8 loose voxels"* — which journal/0055
  falsified world-wide this morning. The dc-core occupancy primitives added by
  journal/0052 exist precisely for this. **The wider lesson:** journal/0010
  shipped partial-height rendering dormant and predicted it would "light up
  for free the day deposition produces its first sub-full column". It lit up
  and did not work, because the assumption it rested on lived in a doc comment
  nobody re-read when the world changed underneath it.
- *(**"The world systematically under-expresses `H`": RETRACTED** 2026-07-21,
  same day it was filed — corrections #28, journal/0058. The alarm was the
  integrator's metres/voxels inversion, not a defect. Verified headlessly:
  the generated column matches `round(H/0.9)` with an error of **0 or +1
  everywhere, never negative**, the +1 being the top-of-column partial voxel
  that a *block* scan must count whole. The client world is also byte-identical
  to the probe world, proven block-for-block at four addresses — the
  stop-the-world hypothesis is dead.)*
- *(**"Surface material is quantized per chunk": FIXED** 2026-07-21,
  journal/0058 — the member is now drawn per voxel column inside the shared
  `surface_sample` kernel. Chunk footprints expressing more than one surface
  member went **0/169 → 147/169**. Block fingerprints unchanged in both
  recorded worlds, which is within-class invariance confirmed by an 80-chunk
  fingerprint that knows nothing about the argument. **Unwalked** — the fix
  landed after the user's session closed, so nobody has seen the patches
  gone.)*
- **Mixed voxels carry no member dither — watch for the chunk-line cutover
  coming back** (journal/0055 judgment call 2, 2026-07-21; UNTESTED either
  way). The 3c-2 boundary dither (`dithered_member`) exists to wander material
  family contacts *off* the chunk grid; the agent deliberately did not apply it
  inside mixed voxels, on sound reasoning — the dither re-picks within a class,
  `classify` breaks ties on the lower material id, so dithering a 4–4
  cross-class split could flip the block and break
  `block == classify(contents)`. Single-event voxels keep the dither. The
  unexamined consequence: mixed bands may now show **chunk-aligned member
  selection**, i.e. the artifact 3c-2 was built to kill, confined to contacts.
  Nobody has looked. **Add it to the appearance walk**: stand at a cut face and
  check whether contact bands change member on chunk lines.
- **91.4 % of land skins to ONE block, and the diagnosis is not settled**
  (journal/0055, 2026-07-21). With the record deciding the surface, the
  world-wide skin goes Grass 81.8 % / Dirt 18.2 % → **Mudstone 91.4 %** /
  CarbonaceousMudstone 6.3 % / Coal 2.0 % / Peat 0.3 % / Granite 0.02 %. The
  agent filed this as a *block-vocabulary* question (the block tier cannot
  summarize the material diversity beneath it). **The integrator does not
  accept that as the whole diagnosis**, and the competing reading matters
  because it points at a different fix: the surface block takes the
  **metre-dominant class of the top 0.9 m**, so a thin organic soil horizon
  over thicker loess *loses the vote even when it is present in the voxel's
  eighths*. That is summarization hiding soil. The rival reading is that
  pedogenesis is simply under-modelled and soil is rarely generated at all —
  real Earth has soil nearly everywhere subaerial, and this record gives an
  organic surface to 6.3 %.
  **The discriminating measurement, cheap and unrun:** what fraction of
  surface *contents* contain organic material, versus what fraction of surface
  *blocks* read organic? If contents ≫ blocks, it is summarization (fix at the
  block/presentation tier). If contents ≈ blocks ≈ 6 %, the record itself
  lacks soil (fix in the biotic layer — adjacent to stubs 7/8).
  - **LARGELY ANSWERED 2026-07-21 by the user's live observation** (*"the world
    does not generate dirt anywhere apparently — was only a veneer placement
    previously"*), verified in the content set: **there is no soil material to
    place.** `MaterialId::LOAM` is defined but registered in **no** geology
    class member (its only other appearance in the tree is a unit test), so
    worldgen can never select it. And the `dc:stratum/organic-soil` class has
    exactly one member — `dc:geo/carbonaceous-mudstone` — whose material is a
    lithified **rock**. So every soil horizon the deep sim records is expressed
    as mudstone because mudstone is the only thing registered to express it.
    The monotony's two candidate causes were both downstream of this: there is
    nothing being hidden by summarization, because there is no soil substance
    in the world. `Block::Dirt` now survives only in the fallback paths
    (subaqueous, wilds, no-record columns) — the user's "veneer placement"
    read, confirmed.
  - **This is a CONTENT-SET gap, not a code bug**, and it is the missing
    substrate under an already-ratified design: the user's soil model (roots as
    porous *structure* holding soil in its pores; loose horizons above, packed
    below — materials.md § forms design pass, ideas.md § soil is loose but
    packable) has no material to operate on. It compounds with the form gap:
    `fill::is_loose` covers only clastic and placer, so organics fall in the
    **structure** bucket and would render as solid cubes even once a soil
    material exists. Two stacked gaps — **no soil substance, and organics
    formed as structure** — and the fix order is substance first, then form
    from provenance (consume-the-ledger piece (c)).
- **Caves ↔ hydrology integration thread captured** (2026-07-21, off-thread
  session; full capture in water.md § Session capture 2026-07-21 — nothing
  decided). The work-shaped findings: **two drainage opinions** (pregen cell
  hydrology vs the deep tier's per-epoch drainage — subsumption candidate,
  with a history-pass resequencing consequence); **deep drainage is computed
  and discarded** (paleo-channel + per-chapter table recorder axis wanted for
  erosional caves; cost unmeasured, eolian memory FLAG adjacent); the
  **bounded-drainage-refinement spike question** that gates RiverSeg
  retirement; the **column-as-interval-log target contract** proposal
  (user-owned ARCHITECTURE call). Same session recorded the user's
  water-rendering directive (partials/structure, placeholder texture, data
  seams for flow/waves) in water.md.
- **The eolian strata record costs +92.76 MB at Medium** (journal/0047
  measurement, 2026-07-21; UNDIAGNOSED in detail). The roster flip took the
  kept `DeepField` 52.8 → 145.5 MB — the record roughly triples because wind
  lays thin units across tens of thousands of cells per chapter. The clock
  cost is trivial (+2.5 s); the memory cost is the real bill. Perf-doctrine
  shape (recorder run-length merge / eviction tuning for eolian units is the
  obvious lever — the S10 soil-overprint precedent), but needs measurement,
  not a bandaid.
- **The 1-D wind march dumps residual load at the downwind land edge**
  (journal/0047, 2026-07-21). The raw strongest loess/deflation/frost cells
  all sit on grid col 544 — an edge-pileup artifact of 0034's march, real in
  every production world. The tour map excludes the border ring; the march's
  boundary handling is the defect. Small, mechanical, undiagnosed beyond the
  symptom.
- **UPDATE 2026-07-21 (second occurrence): the teleport-storm hypothesis is
  FALSIFIED as the trigger.** A `--horizon 6` session crashed with the
  IDENTICAL signature (DeviceLost 10:20:59 → buffer-map panic → cluster
  PoisonError) while **completely idle** — booted, streamed its field, sat
  untouched ~4.5 min, died. Two for two at `--horizon 6` (~10 min with
  activity, ~4.5 min idle); default-horizon sessions historically run long.
  Revised suspicion: resource/VRAM accumulation in the wide-horizon far-field
  path (or a driver interaction it provokes) — a leak-shaped bug, not a
  burst-load bug. Repro is now cheap: boot `--horizon 6`, wait five minutes.
  Diagnosis slice should instrument GPU memory over idle time. Until fixed,
  walks run `--horizon 3` (stations are close-range reads; only skyline
  vistas need 6+). **THIRD SYMPTOM (journal/0049 tour): texture SMEARING
  visible to the human eye after heavy teleporting at `--horizon 3`** —
  user field report, live session. The leak is not horizon-6-exclusive,
  just slower; smearing may be the pre-crash state. Strengthens the
  resource-accumulation hypothesis; the diagnosis slice should reproduce
  via teleport churn while instrumenting GPU memory AND watching for
  texture degradation as the early warning. **Lifetime bound (same day):
  the smearing `--horizon 3` session ran ~44 min through the whole tour and
  exited CLEANLY (verified: no DeviceLost in the log), vs 4.5–10 min to
  death at `--horizon 6` — accumulation scales with far-field size, and
  smearing is the degraded-but-alive state well before the cliff.**
- **CONFIRMED AT HORIZON 6, 2026-07-21 (journal/0065): the fix holds at the
  width that used to kill it, and the residual is horizon-independent.**
  Storm and idle regimes, alternated 6/3/6/3. Storm slope +0.542/+0.529
  MB/jump at 6 vs +0.531/+0.542 at 3 (200 jumps each, RSS high-water 1.10 GB,
  `host_chunks` 2 108–2 332 against `host_budget=2360`, ~19 000 evictions per
  run). Idle at 6: every probe count frozen, RSS drift < 7 MB in 7 min, at the
  full 704-tile far field. All exits **0**, no `DeviceLost`/panic/`ERROR`.
  Two findings ride along: (a) **the teleport storm is nearly blind to
  `--horizon`** — the field never fills under motion (`far_tiles` oscillates
  2–40 at *either* width), which is the mechanism behind 0050's unexplained
  "h3 and h6 slopes are near-identical", so idle is the only regime where the
  horizon is a real variable; (b) **`Authority::chunk_budget_for` does not
  scale with the horizon** — it derives from the *near*-field constant
  `UNLOAD_RADIUS_M`, resolving to 2 360 at every width (correct, but the
  opposite of what the brief assumed).
- **RESOLVED IN THE SAME ENTRY: the "+0.54 MB/jump residual" was a windowing
  artifact.** A single **700-jump / 23-minute** horizon-6 session shows RSS
  **sawtoothing** in a ~980–1 170 MB band — climb ~250 jumps, drop 120–180 MB,
  repeat. Full-run slope **+0.113 MB/jump** (jumps 50–700), **+0.074** over the
  last 300; every 200-jump window sat inside one tooth and read ~+0.54.
  0051's 0.00 and 0065's 0.54 are the same oscillation at different phases.
  Methodological rule now: **measure ≥ 250 jumps or you are measuring a
  tooth.** Final state 71 928 evictions, `host_chunks=2224/2360`, high-water
  1 166 MB, exit 0. The ~0.1 MB/jump that survives is plausibly allocator
  hysteresis; if anyone chases it, 0050's open gap is the candidate (the
  collapse caches' `coarse_surface` / `column_record` paths never trigger
  `evict()` — position-keyed and horizon-independent, exactly this shape).
- **FIXED 2026-07-21 (journal/0051): eviction landed, the march is flat
  (+27.4 → 0.00 MB/jump). See Shipped. Two numbers from this diagnosis were
  corrected on the way: a chunk is 64 KB, not ~33 KB (`Block` is `repr(u16)`),
  so the fill rate is ~335 chunks/jump, not ~750.** Original diagnosis below.
- **DIAGNOSED 2026-07-21 (journal/0050): the leak is host-RAM, not the GPU
  and not the far-field pooling.** Instrumented our own allocation counts
  (`DC_MEM_PROBE` plugin, merged) beside external RSS + VRAM sampling.
  Findings, all measured: (1) **idle `--horizon 6` does NOT leak** — every
  count flat, RSS ~978 MB, VRAM ~3158 MB, survived 6.5 min (the "idle 4.5 min
  death" did not reproduce; the leak is motion-driven, not time-driven).
  (2) A **teleport storm to fresh distant coords leaks RSS linearly, ~25 MB/
  jump (~13 MB/s), unbounded**, while VRAM stays flat AND our render counts
  (meshes/entities/far_tiles) only oscillate, never grow — so the far-field
  pooling and wgpu/VRAM are NOT the site (pooling-doctrine failure-class
  hypothesis FALSIFIED). (3) The decisive cut: the same storm cycling **four
  FIXED coords keeps RSS flat within 4 MB for 142 jumps** — the leak is keyed
  by world *position*, not by render churn. Mechanism (as corrected at
  integration — the draft's "two unbounded caches" died against
  `collapse.rs`): **`HostWorld.chunks` is the one genuinely unbounded store**
  (`dc-api/src/host.rs:178`, materialized `chunk_at` `:367–373`: every chunk
  any query touches, retained forever, ~33 KB each). The `WorldGenerator`
  collapse caches are **bounded** by `evict()` caps (`collapse.rs:609–624`,
  fired per `generate_chunk`) and contribute steady-state footprint, with one
  real gap: far-field-only sampling paths (`coarse_surface`/`column_record`)
  never trigger `evict`. h3 and h6 jump slopes are near-identical (streaming
  budget caps new-world-per-jump); the horizon-scaled *lifetime* the tour saw
  is a steady-state-footprint + continuous-far-field-sweep effect. Fix =
  eviction on `HostWorld.chunks`, which must distinguish generated-untouched
  (droppable — "store only what the derivation cannot predict", the S11
  doctrine) from edited (persist/spill to the save layer); generator-cache
  work is residual (close the coarse_surface gap, distance-aware caps if
  measured). *(**Secondary defect RESOLVED** 2026-07-21, journal/0054 —
  DeviceLost now degrades loudly; the poison cascade is gone at source and
  the exit code no longer lies.)*
- **The shared `CARGO_TARGET_DIR` can serve a STALE binary and produce a
  FALSE-GREEN gate** (found 2026-07-21 by the journal/0054 agent; the mirror
  of corrections #21's impossible *red*, and worse because it does not
  announce itself). Symptom: a full `cargo test --workspace --release`
  reported 489 passed / exit 0 while running **none** of the three tests just
  written — cargo reused a `dc_client-*.exe` timestamped minutes before the
  new source file existed, out of the target dir two agent worktrees share.
  Caught only by grepping for the new **test names**; `cargo clean -p
  dc-client --release` fixed it and the crate's count went 99 → 102.
  Remedy now practised by the integrator: clean every crate changed in the
  session before the merge gate, and verify by test name/count rather than by
  `test result: ok`. Undiagnosed in detail (which fingerprint input goes
  stale); see corrections #27.
- **GPU DeviceLost crash under a teleport storm at `--horizon 6`**
  (2026-07-21, live session, user present). ~65 s after a 10-jump ~28 km
  teleport sequence: `DeviceLost ("driver implementation is at fault")` →
  swap-chain loss → wgpu buffer-map panic → bevy_pbr cluster PoisonError
  cascade. Suspicion (UNDIAGNOSED — needs reproduction, not a bandaid):
  far-field rebuild churn — each long jump rebuilds toward a ~900-tile field
  plus near chunks/colliders — hitting either a Windows TDR (one >2 s GPU
  frame) or VRAM/allocator exhaustion. This is the failure class the
  perf-first doctrine (ARCHITECTURE.md § Modularity and performance,
  DECIDED 2026-07-21) exists for: pooling/recycling of far tiles and chunk
  meshes is the designed answer; voxy-dh-recon's pooled-vertex-buffer row is
  the prior art. Note the process exit code was 0 — the crash is invisible
  to exit-code monitoring; the panic cascade also poisons instead of
  degrading loudly. Secondary defect either way: a DeviceLost should not
  cascade into unwrap panics. Repro suggestion: scripted teleport storm via
  MCP at `--horizon 6+`, watched with GPU memory instrumentation.

- **Stub inventory filed** (`docs/design/stubs.md`, 2026-07-21, read-only audit
  agent + integrator). Ten active stubs, each with its heir. One genuine
  discovery: **ruin-posts was UNDOCUMENTED** — the only world-visible
  substitution with no placeholder marker anywhere; a loud code comment is
  owed at `collapse.rs::ruin_posts` (deferred: collapse.rs is in the climate
  agent's write-set). Two audit additions to the decision's holdout list:
  igneous emplacement-depth constants and paleo-temp-is-present-day. Also
  flagged: the `field.rs` doc-comment claims the collapse tier "reads"
  exhum/t_crust when nothing does — do not trust it.

- *(**Sub-km relief / roughness decay: MEASURED** 2026-07-21, S13 + journal/0041
  — see Shipped. Decay confirmed (5 % survives), bilinear falsified (#24), and
  the walk's own sampling corrected (#25). The remaining OPEN part is which
  recalibration to take — Sequenced below, awaiting a user picture-pick.)*
- *(**`climate_at` half-cell offset: CONFIRMED BUG, FIXED** 2026-07-21,
  journal/0043 — climate sat exactly 7 372.8 m north-east of the terrain it
  tinted, live at every preset (all odd `w`). One-expression fix,
  regression-guarded at Small+Medium; 7 surface columns flip Stone→Dirt in the
  22–33° band (the rock line was 7.4 km off). Heights are climate-independent,
  so all 0040/S13 elevation numbers stand. No prior conclusion falsified —
  the 0038 desert-null mechanism (corrections #22) is registration-independent.)*
- *(**The far field cuts off at 1.2 km**: FIXED 2026-07-21, journal/0042 —
  `--horizon <km>`. Original report: user, 2026-07-21, "the cutoff is still too
  near, can't see macro shape of landscape".)*
- **The haze curve, not geometry, limits the usable vista** (journal/0042
  measurement, 2026-07-21). With `--horizon 8` the outer third of the field
  washes toward white and silhouette reading works only to ~5–6 km, even though
  the geometry is there and paid for. The fog *range* now scales with the
  horizon; the fog **curve** (its falloff shape) does not, and whether it should
  is a **user-owned visual call** the agent deliberately did not make. Cheap to
  change, needs the user's eye on a before/after.
- *(**Material placement rules are climate mocks: DECIDED 2026-07-21** — see
  geology.md § Expression of the ledger, ecology.md § DECIDED 2026-07-21, and
  ARCHITECTURE.md § Modularity and performance. Runtime gen is refinement over
  the ledger; everything recorded must be expressed; unexpressed only where the
  expresser is unbuilt, loudly temporary. The veneer rule is a placeholder —
  DO NOT BANDAID. Remaining OPEN engineering, now Sequenced: consume `exhum`/
  `t_crust`, carry the discarded `H` regolith plane, derive form from
  provenance, and the per-voxel provenance query. Original observation:)*
- **Material placement rules are climate mocks, and below ~460 m there is no
  history to read** (user design observation + integrator analysis,
  2026-07-21 — NOT yet a design pass, nothing ratified). The surface veneer
  rule (`collapse.rs::surface_sample`) picks Grass/Dirt/Stone from year-zero
  climate + a 6.5 °C/km lapse against a −4 °C threshold — no slope term, no
  consultation of the record. geology.md § formation context already ratified
  that year-zero climate is legitimate **only** for the surficial veneer, so
  this is the documented last holdout of a dead shim. Two further gaps found in
  the same sweep: `exhum`/`t_crust` ship in `DeepField` explicitly as "the
  metamorphic-grade axes the collapse tier reads" and **nothing consumes them**;
  and there is **no rule deriving material *form*** (loose / pore-partial /
  whole block) from provenance, though the representation exists (S8 mixtures,
  pore partials). **Integrator's framing, unratified:** the collapse layer
  *samples and dresses* rather than re-simulating — shape below the 460 m deep
  cell is lattice jitter and material below it is member dither, so the
  sub-km-relief finding (Observed above) and the material-mock question are the
  same defect. Wants a priors-first design notebook before any work.
- *(**Console v1 field report: FIXED same day** — console v2 shipped, see
  Shipped / journal 0035. Original report:)* **"still unusable" (user,
  2026-07-20, first test drive).** Two defects, both discoverability-of-what-exists rather
  than missing data: (a) the arg surface is invisible in practice — no
  per-arg help while typing, no visible arg shapes/expected inputs, so a
  user cannot form a valid command without already knowing it; (b) the
  output pane cannot scroll. DIAGNOSED at dispatch (same day): v0 rendered
  help only on explicit `help <cmd>` and completion stopped at param keys;
  the fix is presentational (inline signature + per-arg hints from the
  schemas already carried) plus wiring the merged `Completer` hook
  (decision #6) for value completion. Console-v2 agent dispatched — see In
  flight.

- **Unlock ranking (assistant view, unratified, 2026-07-20)** — raised at
  the user's "what else do we need to get moving to unlock things?" and not
  yet answered, so recorded rather than lost. Ranked by how many downstream
  things they gate: (1) **persistence** — three systems have now
  independently filed "must persist" as an open question (far-field
  summaries, S11's water body graph, the generation-order-dependent
  `MixtureTable`), and edits still do not survive a scale switch; it is one
  answer being asked for three times, and everything built before it gets
  retrofitted after. (2) **inventory/items/interaction** (filed as 3a,
  never scheduled) — you can mine a coal seam and nothing happens; every
  content system terminates at a wall because nothing can be held.
  (3) **loose materials** — half-built already, since partial-height loose
  rendering shipped dormant in 3c-2 and only the content/sim half is
  missing. (4) **water present-tier** — S11 says GO.

- **Test-suite time is trending up: 382 s → 502 s → 541 s** across three
  2026-07-20 milestones (organics' biotic flip put a real ~14 s ritual
  behind every world-level suite; erodibility added its own). Each
  increment was individually justified and none was gated behind
  `--ignored` on the argument that gating would make the suite
  unrepresentative of the shipped world. No policy exists; the cheapest
  moment to set one is before it hurts.

- **An untold user reference: the "culture-language-balrog example"**
  (2026-07-20). The user cited it as load-bearing for how they picture
  darkness, danger and equipment — "you don't know about [it] but you'd get
  it" — then deferred telling it ("it'll cloud your context"). Recorded so
  the thread is not lost: it belongs in
  `docs/design/things-that-will-happen.md` when told.


- **`--fullbright` is blind to geometry, and that cost a walk its conclusion**
  (2026-07-20, journal/0030 + corrections #18). DIAGNOSED, not yet fixed. In
  fullbright every face of a block is one flat vertex colour, so on terrain made
  of a single material there is no cue distinguishing a top face from a side
  face: `0030-flank-before-fullbright.png` renders an entire terraced hillside as
  a **featureless grey field** while the lit frame of the same geometry shows
  every step. The pass that correctly proved a *material* claim in 0027 silently
  answered "no change" to a *geometry* question in 0030.
  **User proposal, 2026-07-20: give block faces dark borders in fullbright** —
  "would give you more sense of dimension and help distinguish block positions."
  Agreed, and it is the direct fix for the failure above. Design notes from the
  agent that hit it:
  1. Prefer **crease/silhouette edges** (outline depth- and normal-discontinuities)
     over per-cube wireframe. What makes a bench legible is the *step*, not the
     grid, and per-voxel outlines at 3.5 km would alias into moiré where a voxel
     is sub-pixel. Fade the edge term out with distance.
  2. Ship it as a **separate flag** (e.g. `--fullbright --edges`) so the pure
     "colour in = colour out" control that 0027 depends on still exists unmodified;
     borders are a renderer-added signal and the data pass should stay data.
  Two adjacent asks from the same walk, both cheap and both currently blocking
  landform photography: **(a) `--fullbright` should also disable distance fog** —
  the 3.5 km summit vista washed to near-white in *both* passes, so silhouette
  work at landform scale is presently impossible; **(b)** nothing is needed for
  sun determinism — confirmed with the user that the sun is static, which is what
  makes the lit pass trustworthy for before/after diffing after all.

- *(**INSTRUMENT: "lit before/after is invalid because the sun moves" —
  RETRACTED same day**, corrections #18/#19. The sun is FIXED (S4: constant
  0.35 time-of-day); no day/night cycle exists, which is exactly why the
  sim-light design had to invent heavenly-body paths. The lit pass was the
  trustworthy register all along. The real defect was the opposite one, and
  is recorded below.)*

- **INSTRUMENT: `--fullbright` is BLIND TO SHAPE** (journal/0030,
  corrections #18). It renders unlit pure vertex colour, so every face of a
  block is the same colour — on single-material terrain a fully terraced
  hillside renders as a **featureless grey field**
  (`0030-flank-before-fullbright.png`, whose every step is plainly visible
  in the lit frame of identical geometry). A near-zero fullbright pixel-diff
  therefore does **not** mean "the shape did not change"; it means this
  control cannot see shape. **Choose the control that can see the question**:
  lit for shape/relief, fullbright for material/data. Proposed fix, filed
  not built: **crease/silhouette edge outlining under a separate flag**
  (`--fullbright --edges`) — outline depth and normal discontinuities only,
  distance-faded, never per-cube (per-voxel outlines alias into moiré where
  a voxel is sub-pixel at km range); separate flag so the pure
  colour-in-colour-out control that the 0027 coal diagnosis depended on
  survives unmodified.

- **INSTRUMENT: `--fullbright` does not disable distance fog** (journal/0030).
  The 3.5 km massif vista washed to near-white in *both* passes, so
  landform-scale silhouette assessment is currently impossible — fog, not
  lighting, destroyed the frame (`0030-massif-*-fullbright.png`). Since
  fullbright exists to be a pure-data diagnostic register, atmospheric
  haze does not belong in it. Cheap fix; blocks silhouette work, which is
  exactly what the dismal-mountains thread needs.
  *(Related walk suggestion, not filed as a defect: crease-aware dark face
  borders under a separate flag — outlining silhouette/depth-discontinuity
  edges only, distance-faded, never per-cube, which would alias at range.
  Kept separate from `--fullbright` so the pure-data control survives.)*


- **"Our dismal mountains"** (user, 2026-07-20) — DIAGNOSED, four causes,
  all absences rather than bugs, which is why the terrain reads as flat in
  character rather than visibly broken:
  1. ~~**Erosion is lithology-blind**~~ **CLOSED AND SHIPPED ON 2026-07-20**
     (journal/0029 built it, journal/0030 flipped it). Erosion is lithology-aware
     (`deeptime::lithology`), with agent-specific resistance so the fix does not
     foreclose karst/glacial/littoral, and `production_config` now runs it.
     Differential erosion is proven present in the numbers (hard beds stand
     proud, basement shields for free) and bounded (stability clamp). **But the
     photographs say the flip bought no landform** — the ground-level surface is
     visibly re-terraced (real, measured in the lit pairs), yet the summit
     silhouette is identical, a 5 km lattice moved ±2 m on 2,615 m of relief, and
     no bench, ledge or resistant core is attributable to lithology at any
     vantage. **Cause 1 is closed and the mountains are still dismal**, which
     localises the remaining problem precisely: causes 2 and 3 are the
     load-bearing ones.
     **Remaining: 2 (no dip/fold), 3 (conservative amplitude), 4 (no glacial).**
     Cause 3 is now the highest-value next move and it is a **user decision, not
     a build** — the lever exists as knobs (`erodibility_contrast`, the global
     erosion rates), and 0029's headroom test (all rates ×10) already showed a
     hard bed standing 44.7 m proud. Nothing more should be photographed here
     until the amplitude call is made.
  2. **No dip/fold** (the layer-cake item below): even differentiated beds
     would only give horizontal benches and mesas; **dipping** hard beds
     are what produce hogbacks, cuestas and flatirons. Second in the
     user's ordering; S9 already classified fold-phase as a cheap *relax*
     term.
  3. **Conservative amplitude / no voxel-scale cliffs** (filed since S7).
  4. **No glacial agent** — cirques, arêtes, horns and U-valleys are the
     most dramatic alpine landforms and we have no cryosphere. Explicitly
     LAST (geology.md already sequences glacial as "later"); it needs ice
     as a modelled agent.
  Causes 1 and 2 multiply: hardness contrast supplies *what* stands out,
  structural dip supplies *how* it stands out.


- **The world is a LAYER CAKE — no dip, no folding, no tilt** (user,
  2026-07-20, from reading walk cross-sections: "the seams have to match
  orientation properly if this was pushed up in a tectonic event,
  subducted, etc."). **Confirmed by source read**: the only structural
  feature implemented anywhere is `DepUnit::unconformity: bool`
  (recorder.rs). Strata are per-column stacks of thickness, so every layer
  in the world is horizontal regardless of the tectonic history that
  produced it — a seam thrust up by an orogeny lies as flat as one that
  never moved. earth-processes § 7 (structural deformation:
  fold/fault/unconformity) is DESIGNED but only the unconformity flag was
  built; S9 classified fold-phase as a *relax* term (`fold-phase =
  f(uplift)` at sample time), so the cheap path was identified and not
  taken. Lateral variation exists only because adjacent columns read
  different 460 m deep cells. **This matters disproportionately for a game
  about reading strata**: dipping beds, folds and truncations are how a
  cross-section tells you what happened to it. Undesigned; pairs with the
  3e structural-deformation work.

- **Thick units render as flawless monoliths** (user, 2026-07-20; general
  case, noticed as house-sized unbroken pure coal). Two mechanisms, both
  ours: deep-time epochs are ~2.5 Myr so parting-forming events are
  sub-epoch and invisible, and the recorder merges consecutive like epochs
  into one horizon (the 665 k → 71 k unit optimization). Real thick seams
  carry mineral partings; real rock generally carries defects. Fix belongs
  at the **collapse tier as procedural detail** (method rule 5), not in
  deep time. Sketches in ideas.md § rock is not monolithic and § coal
  partings; pairs with the filed-not-built charcoal-as-inclusion item.

- **Loose materials do not exist in the world yet** (user, 2026-07-20:
  "needed — even if they don't fall with gravity yet"). The RENDERER is
  already waiting: partial-height loose rendering shipped built-but-dormant
  in 3c-2 (journal/0010) because loose deposition never emits sub-8
  columns. Missing half is content/simulation. User direction for when it
  lands: loose materials should **spread on being dropped** (granularity +
  fall height → partials displaced into surrounding empties) — angle of
  repose from the partials model rather than a physics solver. Sketch in
  ideas.md.


- **The sim must know about light** (user, 2026-07-20; NEW thread, see
  visuals.md § open thread). Distinct from render-side lighting: a
  deterministic sim-side light field is wanted for (candidates)
  photosynthesis/plant growth, spawn behaviour, stealth/NPC vision. No
  prior existed in the corpus. Noted parallel: a propagated light field
  and water.md's bound-water saturation field are the same computational
  shape (bounded local relaxation attenuated by a per-material property),
  so S11's locality result may transfer. Undesigned.


- **No pooling/reuse of chunk or far-tile GPU resources** (user question,
  2026-07-20; read from source, not measured). Every chunk load
  `commands.spawn`s a fresh entity with `meshes.add(to_bevy_mesh(..))` — a
  newly allocated `Mesh` asset — and every unload `despawn()`s it, freeing
  the asset. Far tiles (`LoadedFarTile`) follow the same churn. Partly
  mitigated for free: Bevy's `MeshAllocator` slab-allocates vertex buffers
  (FF2a step 0 found our custom attributes only *select* a slab), and the
  ECS recycles entity ids — so the unmitigated cost is CPU-side `Vec` +
  `Mesh` asset churn on every chunk-boundary crossing. **Filed prior**:
  voxy-dh-recon transfer map already lists "persistently-mapped pooled
  vertex buffers (AZDO) → far-tile buffer management when tiles churn."
  **Unmeasured.** Per placeholder-state-is-not-intent, measure at the
  design-target scale (10 km field, hundreds of tiles churning while
  walking), not at today's 1.2 km — the far-field range knobs milestone is
  the natural vehicle for that measurement.

- **Walk report (2026-07-20, journal/0027): coal renders as pure black in the
  lit pass — a hole in the screen, not a rock.** Photographed at world voxel
  (-76133, -80221) on the client's world: an 18-voxel seam four voxels under
  turf, cut to an open bench under full sky. `0027-coal-seam-cut-lit.png` shows
  grass / mudstone / carbonaceous mudstone and then black for the lower
  two-thirds of the frame — including the **bench floor**, which is an up-facing
  sunlit surface, so this is not shadowing. **The fullbright control
  (`0027-coal-seam-cut-fullbright.png`) shows coal as an ordinary mid-dark grey**,
  so the block, the atlas and the 29-layer palette are all correct. Mechanism:
  `meshing.rs` gives `Block::Coal` a vertex colour of `[0.07, 0.065, 0.06]` —
  7 % linear, ≈ 0.29 sRGB, which is exactly what fullbright draws. The lit path
  multiplies that by the directional term and tonemaps, and 7 % albedo has
  nowhere to go but zero. **The number is physically right** (real coal is
  0.04–0.08) — the defect is that the lit path has **no floor under the dark
  end**, so a correct dark material becomes an absence of image. **Do not fix by
  brightening coal.** This is a lighting/tonemap question (an ambient/sky floor,
  or a tonemap that preserves shadow separation), and it belongs with PBR-2's
  shadow work. Second-order finding, **CORRECTED by the user 2026-07-20** — the
  walk's "excavation interiors receive no light" reading (and the integrator's
  "the underground is unlookable-at" amplification of it) was WRONG. The user:
  *"underground is lit by global sun right now, depending which way the face
  faces it has one of six levels of face light... one block face is dark (idk if
  it's N, S, E, or W) whether on the surface or deep in a hole - the others are
  degrees of well lit."* So the real shape of the defect is:
  **(a) ONE face orientation is black everywhere** — on an open plain exactly as
  much as at the bottom of a shaft — because the face pointing away from the
  directional sun has no ambient floor under it; and
  **(b) there is NO DARKNESS UNDERGROUND AT ALL** — depth does not attenuate
  anything, because nothing occludes. `0027-pit-interior-unlit-lit.png` was
  photographing (a), not a property of pits. Being underground is currently
  *lit exactly like being outside*, which is the deeper problem and the one the
  darkness decision (visuals.md) is about.

- **Walk report (2026-07-20, journal/0027): peat and carbonaceous mudstone are
  nearly the same colour.** `[0.24, 0.17, 0.11]` vs `[0.21, 0.18, 0.15]` — a
  difference you can measure and not really one you can see, and the fullbright
  control makes it worse rather than better (side by side and unlit they are
  nearly the same taupe; `0027-peat-coal-mudstone-section-fullbright.png`).
  Deliberately unfixed for now: peat is rare (17 cells world-wide on the seed
  walked), and the pair that actually matters — organic soil against ordinary
  mudstone — is well separated. Revisit when the placeholder texture packs are
  replaced with authored art, which is the right moment to space the organic
  materials across the palette on purpose.

- **Walk report (2026-07-20, journal/0027): the client can only ever open seed
  1337, so every seed-specific coordinate in the docs is unreachable in game.**
  `dc-client` takes no `--seed`; `app.rs` boots `Authority::new(BENCH_SEED, ..)`
  with `BENCH_SEED: i32 = 1337`, and the worldgen authority takes `seed as u64`
  — so `0x0D5E_ED57_2026` (the seed every spike and the whole S10/0026 site list
  is measured on) **does not fit in the client's `i32` seed** and world voxel
  (107338, 58787) is a headless-test address, not a place a player can stand.
  journal/0026's "stand at world voxel (107338, 58787) and dig down" is not
  actionable from the game. Cheap fix available (a `--seed` arg on the same
  path as `--pack`), not taken on a photo walk because it is client code and
  this walk changed none. Until then, walks must re-site their own subjects on
  seed 1337, which journal/0027 did.

- **User field report (2026-07-20, filed at the FF2a/0024 ratification): a
  razor-straight, kilometer-scale grass/dirt frontier cuts the far field**
  (visible in `0024-after-ne.png` / `0024-fb-ne.png`; present in fullbright,
  so it is surface-block DATA, not a seam or lighting). The user identifies
  this as **the original cause of the walk-8 complaint** — material families
  "appearing to change immediately across some kind of boundary" — now
  legible at full extent because the far field renders the surface rule at
  km scale: "obviously bad / not natural appearing." **DIAGNOSED
  2026-07-20 (read from source, not yet fixed)** — and it is NOT the
  quantization class this entry first guessed. `collapse.rs`
  `surface_sample`: `let bare = riverbed || precip < 0.10 || (fringe &&
  precip < 0.35)` — a **hard binary threshold on a very smooth field**.
  `climate_at` bilinearly interpolates precip between climate-cell centres,
  and a cell is `CELL_VOXELS` = 16 384 voxels ≈ **14.7 km** at N=2, so over
  any near-field view the field is essentially locally linear: its 0.10
  isoline is a geometrically straight line running for kilometres, and the
  threshold gives it **zero transition width**. Two independent defects
  (both must be fixed): (1) *no transition* — grass/dirt needs a
  probabilistic/fractional band around the threshold, not a step; (2) *no
  detail in the boundary itself* — even a soft edge would be a smooth
  km-scale arc, so the isoline wants domain warp / octave noise on precip
  (or on the threshold) to make the frontier wander at 10–100 m scale.
  The 3d member-contact dither is the material-tier precedent; surface
  BLOCK selection has no analogue. **Any fix must live inside
  `surface_sample`**, which near and far provably share (journal/0022), so
  the horizon heals with the ground. Related: walk-8 "material families cut
  hard on chunk lines" — the user identifies THIS as that complaint's
  origin (a different mechanism from the walk-10 per-chunk flow_energy
  rounding, which stays open separately). Couples to the biotic layer: the
  real cure may be that ground cover stops being a paint decision at all
  (see docs/design/ideas.md § the bio slot).

- *(**User field report (2026-07-20, post-FF2a): thin bright seams between far
  patches — RESOLVED** 2026-07-20, journal/0024, fix cycle, background agent;
  worktree branch for the integrator; gates green. Mechanism confirmed
  (corrections #11): the anti-z-fight push translated adjacent same-level tiles
  along their *own* center→viewer directions, differing by the tiles' angular
  separation, so mesh-space-watertight seams reopened at the TRANSFORM stage as
  0.08–0.9 m (L1→L4) world-space slots — see-through once the far field became a
  hollow top-surface sheet (0022/FF2a). Fix: **per-level UNIFORM push** — one
  shared vector per LOD level per frame, along the camera-forward axis, magnitude
  unchanged (`DEPTH_PUSH_FRAC × coarse voxel`). Every tile of a level undergoes
  the identical rigid translation, so shared edges cannot separate *by
  construction*; magnitude differs per level, so overlapping ring pairs still
  separate in the lap band (corrections #1 honored — a true world-space offset,
  never a bias). Applied to BOTH far paths (S1 chunks + FF2a tiles). World-space
  seam proof past the transform: `uniform_push_keeps_same_level_seams_watertight_in_world_space`
  (exact-zero shared-edge gap for a nasty off-axis high vantage; the retired
  radial scheme fails the same check). Walk-verified live (lit + fullbright, high
  vantage −45° yaw sweep): far field continuous, no bright light-through slivers
  anywhere; no z-fight at lap bands or the near/far boundary at grazing or
  top-down angles. Assets `0024-after-{ne,se}`, `0024-fb-ne`,
  `0024-fb-zfight-graze`, `0024-zfight-topdown`.)*

- *(**User field report: the far LOD sheet is buried under the near field —
  RESOLVED** 2026-07-19, FF2a journal/0023. Mechanism confirmed: the walk-17
  one-tile inner lap slid the L1 far sheet under the near field (from ~54 m),
  sunk half a coarse voxel — present but hidden, so digging exposed a phantom
  floor. Fix: **coverage logic, not buried geometry** — a far column the near
  volumetric field covers (`near_covers`, altitude-aware 3-D distance < 112 m)
  is CULLED, so a fully-covered tile meshes to *nothing*. Floor-quantized far
  tops (≤ near surface) let the near field win the thin [112, 128] m occluded
  overlap with no sink. Dig test photographed: only real near geology, no
  phantom floor (`0023-fb-dig-no-phantom-floor`); headless proof: fully-covered
  tile meshes empty.)*

- *(**User field report: clear pixel gaps between far-field tiles — RESOLVED**
  2026-07-19, FF2a journal/0023. The prediction held: voxelization cures the
  crack class **inherently**. Stepped prisms share face planes; a step's side
  face is emitted once by the taller column only, and a tile samples its
  neighbours' shared boundary columns, so same-level tile seams are watertight
  with no skirt (grazing-angle fullbright shot `0023-fb-grazing-horizon`: no
  cracks). Only differing-stride ring-to-ring edges and the near/far coverage
  boundary get a modest 2-coarse-voxel skirt — "skirt only what remains." The
  faint one-sided-normal stitch *lines* (cosmetic, haze-hidden) stay filed
  below, unchanged.)*

- **The Voxy-vs-Distant-Horizons thread** *(submission question DECIDED
  2026-07-19 session 3 — docs/design/voxy-dh-recon-2026-07-19.md
  § Addendum: far field rides Bevy 0.19's engine GPU-driven path on the
  one-shared-material `Mesh` substrate; vertex pulling/bespoke cmdgen
  rejected; packed quad survives only as a candidate storage format.
  **Step-0 CONFIRMED** 2026-07-19, FF2a journal/0023: the multidraw path
  engages for our custom material — live `mode=Culling`, `Opaque3d
  batches=74 sets=1` on Vulkan; custom vertex attributes don't disqualify
  batching. Substrate decision is real on this hardware.)*
  Still open from the doc: persistent edit-updated LOD store (summaries
  beside S3 region files, dirty-rail subscription — still owed; FF2a kept
  the mesher pure + the `ColumnSpan` payload persistence-shaped so it's a
  drop-in); Aokana SVDAG ray-march as the FF2b volumetric candidate. Vista-as-augury
  constraint stands (rendering is never an observer — binds when
  live-sim state becomes far-visible).

- Walk 17 (journal/0022 § walk 17 + § the holes were a partition): **far
  sheet parallelogram sky holes — RESOLVED** 2026-07-19 (fix cycle,
  worktree branch). Mechanism: a level's far tiles *partition* the ground
  plane (no overlap), so a point has exactly one tile per level; center-
  distance ring assignment let an inter-ring boundary cell be rejected by
  BOTH the finer ring (center past its outer edge) and the coarser ring
  (center short of its inner edge), punching a fixed-position sky hole with
  no fallback tile. NOT winding, NOT a missing index. Fix: each ring laps
  its inner edge one own-tile inward (`far_tile_in_ring`), restoring
  between-ring redundancy; the same lap under the near field also cured the
  **grazing-angle near/far slivers**. Headless proof:
  `far_tiles_cover_the_rings_without_seams` (88 463 uncovered points
  pre-fix → 0). Verified live hole-free at both walk-17 vantages + 3 yaw
  sweeps (fullbright). *(Phantom old world + empty horizon: RESOLVED
  earlier, verified at the 0018 framings.)* **Left filed:** faint
  tile-edge stitch lines (one-sided-normal seam) — cosmetic, haze-hidden;
  a cross-tile normal halo is deferred polish.


- Walk 16 (journal/0021 § walk 16): **the lit path erases low-fraction
  mixtures** — DIAGNOSED by same-framing lit/fullbright pair
  (0021-mixture-* assets): olivine wins whole cells in fullbright, zero
  pixels in lit; same splat attributes, so the heightlerp buries it
  (elevation = weight + texture height; a 1/8 accessory can't out-elevate
  a 7/8 host anywhere). Contradicts the ratified "grains poke through"
  intent. RESOLVED 2026-07-19: amplitude-by-rarity + jitter shipped and
  photographically verified (0021-*-v2 pair — olivine visible lit,
  statistical agreement with fullbright; amp constant is the tuning knob
  if ore should read louder). Cell quantization was rejected (grids shear
  authored features). Residual note:
  the houndstooth fix's narrowed albedo spread (0.17→0.11) reduced
  constituent contrast in the lit path generally.

- Walk 14 (journal/0019 § walk 14): *(fullbright lost the mixture speckle:
  **RESOLVED by PBR-1 render polish**, journal/0020 — a `FullbrightTerrainMaterial`
  (unlit custom `Material`) now renders uniform/block faces as one flat registry
  albedo and mixed faces as the world-anchored 4×4-cell constituent speckle,
  computed shader-side from the splat attributes + a world-anchored cell hash +
  the albedo palette uniform, NOT the LabPBR textures. Zero mosaic geometry.
  Live-verified: uniform spawn strata read flat under fullbright, both shaders
  compile/render error-free. **In-world speckle photograph still owed** — needs a
  real mixed face.)* **Member-contact + placer closeup shots owed** — not
  findable by eye under ground cover; wants the pregen-introspection dev MCP tools
  (walk-9 Observed item). The fullbright speckle photo rides with them (same
  scan-guided-site problem).

- **Placeholder texture tiling: cleanup pass DONE** (journal/0020, PBR-1 render
  polish). The walk-14 "houndstooth" was diagnosed as **legible per-voxel
  repetition of a low-frequency directional texture signature, NOT a seam
  failure** — the tiles wrap (the generator's `_seam_check` is still green;
  offline 4×4 tiling shows no boundary discontinuity), so the "seamless tiling
  verified" claim is **not** falsified (no corrections.md entry). Two mechanisms
  shipped: (1) **world-anchored UVs** in the mesher (tile origin = world voxel
  coord — continuous across same-material blocks and greedy quads, and the source
  of the fullbright speckle's world-stable cell grid; near visually-neutral in the
  lit path on its own for the current seamless tiles); (2) **de-directionalized
  placeholder textures** (`gen_placeholder_textures.py`: freq floor 2→4 + a
  three-octave fractal weighted to high frequency, albedo spread 0.17→0.11, relief
  2.6→1.5, softer AO ramp — regenerated all 26 packs, seam self-check still green)
  — this is the mechanism that actually removes the visible artifact. Live spawn
  (grass/dirt) reads as isotropic pixel grain with no legible period. **Geology
  outcrop before/after at the walk-14 framing still owed to the milestone walk.**

- PBR-1 loose ends (journal/0019):
  - **Terrain lighting is hand-rolled** (directional sun + hemispherical
    ambient from a plain uniform, not Bevy's clustered path — PBR-1 has no point
    lights). Sun/ambient calibration photographed in walk 14 (0019 assets:
    lit-vista, textured-outcrop — no top-face blowout at this calibration);
    **user read owed for the SPLAT_N=4 + calibration ratification**; the
    real curve arrives with PBR-2's HDR + tonemap.
  - **Far field + legacy S1 are now textured too** (one material renders the
    whole lit world). *(The phantom-far-world it painted is **gone** under the
    worldgen authority — journal/0022's summary-shaped far field; the same
    terrain material now lights the horizon that lights the ground.)*
  - **Normal-map tangent frame is a per-face axis-aligned approximation**; the
    normal X/Y orientation may be inconsistent across the six faces (cosmetic on
    the subtle placeholder relief). Revisit with authored textures + POM (PBR-2).
  - **The 16×16 atlas has no mipmaps** (nearest, no mip) — distance aliasing on
    the far rings; accepted for placeholder, revisit with real textures/POM.
  - **World-anchored UVs are f32 world voxel coordinates** (journal/0020) —
    integer at corners, so tiles resolve exactly to ~±16 M voxels and acceptably
    across the playable range; extreme deep-time coordinates would eventually lose
    texel precision. Accepted (the far field, the other extreme-coordinate
    consumer, is a known defect). One tile per *current-scale* voxel, so far-mesh
    coarse voxels still stretch a single tile — subsumed with the summary-shaped
    far field.
  - **Embedded WGSL shaders are compiled by naga at pipeline-build time** — no CPU
    gate (`fmt`/`clippy`/tests) sees them (journal/0020: a `active` reserved-word
    typo passed every gate and would have blanked fullbright; caught only by the
    live smoke run). The live launch + log read is the shader's only compiler;
    keep it in the walk protocol for any shader change.
  - Specular **porosity/emission channels are sampled but wetness is not wired**
    (no weather → no sim-driven porosity darkening yet; PBR-2 + materials sim).
  - Placeholder packs widened **21 → 26**; the historical placeholder-textures
    Shipped line still reads "21 deterministic packs" (not amended — dated
    record).
  - `MeshData` now carries UV + splat attributes on **every** chunk (far field
    and benches included), a small per-vertex memory bump over block-only meshes;
    accepted (the far field is getting subsumed anyway).

- **Walk 12's "blocking regression" was a misdiagnosis** (corrections #10,
  journal/0016): a pose in meters cross-checked against block queries in
  voxels. `true_surface_m` was already deep-time-aware (its ceiling reads
  `ColumnRec`). The investigation still paid: `eye_in_solid` was answering
  from the legacy S1 world on any ChunkMap miss, and a failed surface scan
  silently returned `analytic − 220 m`. Both fixed (merge `81a87b8`;
  `Option`-typed misses, authoritative solidity).
- **Instrument fix: pose replies echo the voxel coordinate — DONE**
  (journal/0021, instrument batch, awaiting integration). `client_player_pose_
  {get,set}` and `dc:character/pose` now carry `pos_voxel` (feet, active-scale
  voxels via the authority's own `scale.voxel_at`) beside the meters `pos` —
  the walker's two languages both labelled, no mental unit conversion (the
  corrections #10 misread that cost a full agent cycle).
- *(**Far mesh generated from S1 `TerrainGen` under the worldgen authority:
  RESOLVED** — journal/0022, the far-field horizon. Key 2 now samples the
  authority's OWN coarse surface (`WorldGenerator::coarse_surface`, the collapse
  kernel at a coarse stride — the lattice already folds in the deep-time surface)
  into a heightfield far field; the phantom old world ~1 km down is gone and there
  is a horizon. Keys 3/4 keep the S1 far mesh (`FarFieldTerrain`), which was never
  wrong for the S1 world. **Still open**: sealing `TerrainGen` behind
  `pub(in crate::authority)` — keys 3/4 still name it, so the seal waits on
  retiring S1 entirely, not on the far field. **Deferred by design** (journal/0022,
  noted here): (a) *persisted* summaries — the far field derives in-memory on
  demand this milestone (~0.22 s/full field), region-file storage couples to S3
  grouping and is the follow-on; (b) the heightfield is a **top surface**, so
  looking up from deep in a chasm loses the far field — the volumetric shell did
  extend down a chasm; a volumetric/skirted summary is the follow-on.)*
- Walk 12 residue: *(far-mesh visual assessment: DONE, walk 13 —
  journal/0018 § the empty horizon: at worldgen altitude the horizon is
  sky; the S1 phantom shows only from steep angles through haze. The
  far-field milestone is "build the horizon", not "fix the phantom".)*
  Still open: test-suite time +~6 min (deep-time on every Medium/Large
  pregen — wants a cost-insensitive fast path); Large extent runs a
  coarsened (~1.8 km) deep cell under the width cap until 3e-2's C
  refinement restores landform detail on approach.

- Walk 13 (journal/0018): *(**`surface_snapped` absent on the `surface:true`
  success path: RESOLVED** — journal/0021, instrument batch. The flag is now
  ALWAYS present in a `surface:true` reply: true when the scan seated the feet,
  false on a miss (position left as requested, `surface_error` string). Both
  paths leave through one `surface_teleport_reply` helper, so the flag can't be
  set on only one branch again.)*

- Walk 11 + step-3 loose ends (journal/0014): *(crouch factor: DECIDED
  2026-07-19, bodies.md § Crouch semantics — 0.6× stands; crouch is
  posture + the future sneak edge-walk-block verb, never crawlspace
  access; sub-standing clearances belong to prone/crawl, sketched in
  ideas.md § posture ladder.)* Max-bend leg fold reads
  tangled — knee pole/fold distribution wants a photo-driven tuning
  pass. *(Posture not exposed in `character_pose`: **RESOLVED** — journal/0021,
  instrument batch; the pose reply now carries `posture` in `set_posture`'s own
  `standing`/`crouching` vocabulary, so a readback round-trips into a command.)*
  Foot-IK ground read can momentarily see the S1
  far-mesh phantom at the extreme load-radius edge.

- Walk 5 (journal/0005, character surface): characters have **no auto
  step-up** — a one-voxel rise halts a grounded walker until it jumps
  *(DECIDED 2026-07-19: stays jump-required; mover-feature vs
  controller-skill is the NPC-intelligence design's question)*.
  *(Disconnect policy: DECIDED 2026-07-19 — freeze for v1, NPC-tier
  degradation as a later controller binding; API.md § Characters,
  bodies.md formerly-open Q3. Attach placement guard: fixed,
  journal/0006.)*

- Geology v1 loose ends (journal/0007): sea-floor/wilds columns keep empty
  strata records (subaqueous sedimentation = the carbonate milestone); no
  `def_changed` events for class/member defines (hot-reload signal; the
  events schema still enumerates only the original three kinds); dev-token
  `dc:*` grant *(DECIDED 2026-07-19, API.md § Capabilities: dev builds
  yes, shipped builds never — build-config gated; implementation owed)*;
  worldgen `MixtureTable` is per-generator,
  unbounded, and its ids are generation-order-dependent — a save layer must
  persist the table, never re-derive it (lifecycle belongs to region-file
  grouping, S3 OQ 7).

- Walk 10 + 3d loose ends (journal/0011): *(member identity is
  render-invisible: **RESOLVED by PBR-1**, journal/0019 — uniform-contents
  voxels now sample their material atlas layer, so siltstone renders
  differently from mudstone; the 3d contact smoothing is now visible. Walk
  confirmation owed with the PBR-1 milestone walk.)* **Class-presence
  quantization still cuts on chunk lines** (flow_energy rounding
  per-chunk) — most likely what walk 8 actually saw; needs per-voxel-
  column context. Igneous tectonic-setting fitness axis deferred to 3e.
  Olivine albedo reads close to grass green from afar — registry-data
  polish with the real texture milestone. Accessory gate/pore-fraction
  values picked, not tuned.

- Walk 9 + 3c-2 loose ends (journal/0010): dither cell count (4×4) and
  the resolve-at-boundary sidecar transport accepted as-built (integrator
  review 2026-07-19, per the no-bandaid razor). Far field and the
  legacy S1 authority render no dither (near-field worldgen only).
  Partial-height rendering is dormant until loose deposition emits sub-8
  columns; edits don't update materials (safe via the block gate).
  Mixed-heavy meshing worst case 16× triangles (real placer bands are
  thin). The walk wants **pregen introspection as dev MCP tools**
  (hydrology/province queries) instead of a side-channel example binary.
  Coarse clastic is rare at world scale (5 river segments on the Medium
  seed) — honest sedimentology vs knob, revisit when rivers refine.

- Walk 8 + user live observations (journal/0009): **nobody owns body
  orientation** — `SetMoveIntent` never touches yaw, `SetLook` is the only
  writer, so un-looked bodies strafe/moonwalk; v0 fix is a design call
  (renderer-side trunk-toward-velocity, cosmetic, vs controller
  convention), real answer is staircase step 3's trunk/look split.
  **Faceless heads make orientation unphotographable** — a face cue
  (texture or v0 asymmetry) is prerequisite to walk-verifying facing.
  **Material families cut hard on chunk lines** — candidate mechanism: at
  N=2 the chunk (28.8 m) equals the S7 column-quantization cell, so
  cell-stepped climate context flips selection exactly on chunk borders;
  diagnosis + smoothing (context interpolation / boundary dither) filed.
  **Third-person view** needed eventually to verify the player's own body
  (dev affordance; not slated). `jump` remains a fallback pose, not an
  authored clip. Plans/clips are define-only (no get/list queries yet).

- Walk 7 loose ends (journal/0008): **unloaded-neighbour and far-mesh
  fallbacks still sample S1 `TerrainGen`** — near-field loaded chunks are
  worldgen, but the far LOD rings and load-radius border faces show the old
  hill-field; user-sighted in walk 8 as a *phantom old world ~500 m below*
  the real terrain that dissolves on approach. A visible artifact until the
  far field becomes worldgen/summary-shaped (pairs with the existing
  far-mesh Observed items). Single-material
  faces under fullbright are featureless color fields — information arrives
  with the 3c-2 dither and later the splat pipeline. The `Terrain` resource
  is retained solely as the 3/4-key legacy fallback. *(Scale-3 boot default:
  fixed in 3c-1; freeze wire-drop path: proven live in walk 7.)*

- Walk 6 loose ends (journal/0006): the surface scan window is
  S1-terrain-sized (8 m headroom / 220 m depth) — structures stacked >8 m
  above the analytic surface won't be snapped to; revisit when worldgen
  amplitude grows. `find_open_spawn` uses terrain-only solidity — correct
  at startup, wrong if ever reused post-edits. Dev `spawn_character` stays
  unguarded (dev keeps full reach) — *ratified 2026-07-19 along with
  `surface:true` attach semantics (API.md § Characters records it; this
  line was stale until the 2nd-session doc sweep).*

- Walks 3–4 (journal/0003, 0004 — corrected record in corrections.md #3,
  #5): near-field lighting blows out pale top faces (flat shading +
  near-vertical sun + no tonemap shoulder) — art calibration, belongs to
  the visuals pass. Chasm cliff "speckle" now photographed from verified
  clean air (0006 asset): reads as genuine single-voxel terracing on the
  near-vertical carve, not a mesh defect — diagnosis still owed.
  *(`surface_height_m` under-report: diagnosed and fixed, journal/0006 —
  the "missing octave" hypothesis falsified, corrections #5.)*

- The embedded HostWorld never evicts chunks (~64 KiB per chunk ever
  streamed/edited); never-edited chunks are pure generator output and could
  be dropped freely (journal/0002).
- Far field doesn't see edits — the worldgen far field is a coarse *summary*
  (not a cache), so a broken block un-breaks beyond the full-detail radius; a
  summary that tracks edits is a follow-on (journal/0002, 0022). (Same for the
  S1 far mesh on keys 3/4.)
- Edits don't survive a 2/3/4 scale switch (authority rebuilt); the command
  log is the eventual persistence answer (journal/0002).
- *(**~2/3 of far-mesh triangles were sealed cave surfaces (S3): RESOLVED for
  the worldgen far field** — journal/0022. The worldgen horizon is a **top-surface
  heightfield** (2048 tris/tile, no interiors), not a volumetric shell. The S1 far
  mesh on keys 3/4 is unchanged / still volumetric.)*
- Far meshing is main-thread, budgeted (S1/S3, and the worldgen heightfield) —
  wants async tasks (journal/0022 keeps it budgeted/incremental).
- Rivers are straight cell-chords (S7); course refinement needs the 2-ring
  argument re-proved at finer levels.
- Terrain amplitude conservative — no voxel-scale cliffs (S7).
- Site cap 240 (u8 RegionId) — concrete instance of S2's ledger-scale
  question (S7).
- Sparse sidecar encoding for thin debris drapes (S8) — index array dominates.
- Seed-stable worlds across releases: versioning policy undecided (S7).
- HDR/exposure: v0 post grades LDR; sky-as-pass needs hook format 1 (S4).
- S2 checkpoint facts (deep-time re-derivation cost) — design owed before
  ledgers densify.

- *(**`client_player_pose_set` outside the one door: RATIFIED same day** —
  API.md Decisions log #5. Retirement = player controller through
  controller-verb commands; sequenced after the session-4 agents land,
  since it rewrites files they are touching. See Sequenced.)*

- *(**Console follow-ups: RATIFIED same day** — API.md Decisions log #6
  (`completions` hook) and #7 (registry macro/derive). Dispatched as a
  session-4 background agent — see In flight.)*

- **Texture steps toward albedo at range** (user, 2026-07-22, on ratifying
  FF2b's look: "the moire is very strong and only nearfield really needs full
  texture… at range, step toward albedo. (5 km bluff may as well just be
  red)"). Priors already pointing here: `--edges` distance-fades to zero past
  1.4 km because per-voxel signal aliases into moiré (journal/0031); the
  houndstooth fix cured a range artifact by *narrowing* albedo spread
  (journal/0020); and every material already carries a flat registry albedo
  (journal/0010 — fullbright's register), so the far target color is
  existing data, and the lerp lives in the ONE shared material
  (pack-compatible, batch-safe). **User hypothesis, to be measured: this is
  also a render-perf WIN** (far pixels skip the splat/texture work
  entirely) — A-6-clean, since the measurement picks the shader path.
  Sequence with the profiling slice so the before/after is real numbers.
  **Rider (user, same conversation): rendering ranges are PLAYER-FACING
  config knobs.** "even at medium distance texture barely registers … may as
  well skip the stack. like anything, i would prefer to have rendering
  ranges as knobs. this is a game people will run on different hardware.
  (goes for the octree LOD bands too). this is a game, it comes with game
  config settings, one must imagine." — S-1's where-there's-a-range-there's-
  a-knob doctrine extended to the render tier AND promoted from dev CLI
  flags to game settings: texture/detail-stack fade range, LOD band radii,
  horizon. Filed as a constraint on the albedo slice and on all future
  far-field work: ranges arrive as config, not constants.

- **We cannot see where runtime goes — the perf observability gap** (user,
  2026-07-22: "it's not easy for us to target where the perf killers are";
  same conversation as the CLAUDE.md runtime-is-sacred convention). The
  project measures gen-time rigorously (spike results, ritual A/Bs, budget
  counters) but has NO runtime frame/tick observability: no span-level
  profiling, no tick-time breakdown, no way to attribute the chunk-drop
  chop (Observed above) to gen vs meshing vs tick contention. **Enabling
  slice filed: wire `tracing` spans through the hot paths (chunk gen,
  meshing, collider tiles, far-field derive, sim tick) with Bevy's Tracy
  integration (`trace_tracy`), then capture a BASELINE profile of the
  known-bad scenario (vertical drop) into `docs/audits/` as the first
  ranked perf-killer list.** Instrument-must-see-the-question applied to
  time. Sequenced after FF2b-minimal merges (dc-client write-set overlap);
  pairs naturally with the erosion-budget flag slice already queued there.

- **Material identity is illegible under splat blending — heightmap SHAPE as
  a fix candidate** (user, walk 0071, station 4, 2026-07-22): the charcoal
  specks pass their regression check but are hard to *identify* because
  "everything is honestly so blended." The user's sketch: most material
  heightmaps are currently a random scramble; if each material's heightmap
  carried a **characteristic shape**, heightmap-based splat blending would
  let the eye decode *which* materials were blended, not just that blending
  happened. Explicitly "not the only possible answer, just a thought" — a
  visuals/materials design thread, not a decision. Couples to the
  form-dependent-texture visuals decision already accepted as deferred cost
  in the fill contract (loose vs structural clastic indistinguishable).

- **Chunk gen time is now noticeable in vertical streaming** (user field
  report, walk 0071, 2026-07-22): dropping from height — so the adaptive
  load volume streams chunks *below* — gets so choppy that "time appears to
  slow to a crawl, sometimes." Observation only, no diagnosis: the symptom
  (sim time dilating, not just frame hitching) suggests generation work is
  contending with the tick rather than merely the renderer, but that is a
  hypothesis to test, not a finding. *(Sharpened same day with a prime
  suspect: `dc-client/src/streaming.rs:42` — `LOAD_BUDGET_PER_FRAME = 8`,
  generated SYNCHRONOUSLY on the main schedule; no AsyncComputeTaskPool
  anywhere in streaming. Far-mesh is the same pattern (2 tiles/frame,
  main-thread; journal/0023 filed "async is a drop-in", unclaimed). Chunk
  gen is a pure seeded function, so task-pool offload does not threaten
  determinism. Still profile before building — but the profiling slice and
  the async-offload slice are now an obvious pair, and the client runtime
  is otherwise nearly single-threaded against a deep sim that already
  proved byte-identical rayon parallelism.)* Distinct from the pregen-time
  non-constraint (that covenant covers world *creation*; this is runtime
  streaming). Couples forward to the octree substrate (coarse-below is
  exactly what FF2b-class nodes eventually provide while true chunks
  generate) — but likely wants profiling before any architecture is blamed.

- **The dominance flip quantizes smooth gradients — a potential S-4 edge**
  (user, walk 0071, 2026-07-22) — **RESOLVED by the susceptibility blend
  (journal/0072), shipped 2026-07-22.** The candidate continuous variant is
  now the mechanism: erosion's four consumption sites blend the per-agent
  susceptibility table by the near-surface window's per-`Litho` *shares*
  (`providers::outcrop_shares`) instead of argmax-then-lookup, so the rate
  field is continuous where the plurality crossover stepped it — argmax is
  its limiting case (a uniform window blends to that rock's rate bit for bit).
  Measured (`examples/outcrop_blend_probe.rs`, Medium): ~18 k former-flip
  adjacencies dropped into the sub-0.1 rate-jump buckets while genuine
  basement↔sediment contacts stay sharp; per-epoch refresh +6 % (negligible).
  Goldens re-baselined (all Medium hashes + the Small block hash, geometry
  only). Audit site A1, shape-teacher #1 of the threshold-quantization
  migration.
  - **NEEDS RATIFICATION (world-scale gameplay consequence):** coal
    diggability fell. A coaly near-surface window is now recessive (coal is
    the softest rock, so its *share* pulls the blended rate up, where the old
    argmax handed a coal-minority window the dominant rock's slower rate), so
    near-surface coal is preferentially stripped. Medium seed
    `0x0D5EED572026` census dropped from 88 record seams over 3 m (strongest
    16 diggable) to **10 record seams over 3 m, strongest 11 diggable
    collapse-voxels** (≈ 10 m — still eminently diggable; the walk-0071 seam
    the user cut and called "looks great" was 3). `MIN_DIGGABLE_COAL_VOX`
    re-baselined 15 → 10 with the census printed (organic.rs), not slid
    silently. The mechanism rides as-built (no-bandaid); this flags the
    reduced-coal *appearance* for the user's blessing.
    **RATIFIED AS-BUILT (user, 2026-07-22, live session: "bless coal
    as-built").** Exposed coal is genuinely recessive; scarcity reads as
    value; deep coal below the window is untouched. If scarcity ever feels
    wrong at play, the lever is calibration (contrast/cap or coal's property
    sheet), never the blend.

- **The cake observation: minority phases guillotine at cell perimeters
  under the coherent draw** (user, 2026-07-22, on ratifying B1's look —
  the swirl-cake-slice metaphor). B1's dither made the SOURCE continuous
  but the SHARES it thresholds are still nearest-per-cell, so minority
  swirls die at the 460 m line while shared majorities flow through —
  the S-4 sharpening catching the residual half of its own fix (the cause
  is still cell-quantized; only the expression got smooth). Suspected
  harder-to-see sibling: the journal/0058 MEMBER dither should guillotine
  minority members identically at perimeters (unexamined — check when the
  cure lands). Cure named, assigned to the CoarseField extraction: near
  boundaries, seeded membership dither of the SOURCE CELL (bilinearly
  weighted), then draw within that cell's shares — the residue-(b) paired
  pattern at the surface. Also owed there: the toward-50/50 bias of
  interpolated-uniform noise (dice-sum CDF distortion) — either a
  CDF-corrected coherent source or the far-summarize register retires it.

- **Texel-edge dither bands on close-pressed walls, anisotropic** (user field
  report, walk 0071, 2026-07-22; asset
  `journal/assets/0071-artifact-texel-edge-jitter-wall.png`; observed at
  world x ≈ 99,341 m, y ≈ 198 m — a basalt hole wall). At extreme
  magnification, VERTICAL texel boundaries dissolve into noisy dither bands
  while HORIZONTAL boundaries stay razor-straight. Not new, per the user; a
  rendering fix/mitigation dogear, not a walk finding. Two candidate
  mechanisms, one discriminator:
  (a) **f32 precision exhaustion in world-position-derived UVs** — at
  |x| ≈ 99 km an f32 ULP is ~8–16 mm vs a 56 mm texel, so the U axis
  quantizes noisily while V (from y ≈ 198 m, µm-precise) stays clean —
  matches the anisotropy exactly; ARCHITECTURE's floating-origin rule
  protects geometry but not any shader path reconstructing absolute world
  position in f32;
  (b) **heightmap-based splat mixing** (user's hypothesis) — near-equal
  weights on 4 same-texture splats let the per-pixel winner flip; standalone
  it does not predict the anisotropy, but if the heightmap/splat selector is
  itself world-position-sampled, (b) inherits (a) and they are one bug.
  **Falsifier: press against a wall near world origin** — (a) predicts both
  edge directions straight there and band width growing with |x|; (b) alone
  predicts the artifact everywhere. Then read the WGSL UV/splat-height
  derivation. Candidate mitigations if (a): camera-relative or
  origin-rebased UVs, f64-split world coords on the CPU side, or
  per-region UV rebasing.
  earth-processes § 2 (Igneous) is a sketch; nothing is built. Arc/rift
  provenance raises elevation but builds no edifices. What it would buy:
  the fastest legal short-gradation mountain on Earth (a stratovolcano is
  ~3 km of relief in a ~20 km footprint), calderas, lava caprock → mesas,
  ash beds as strata events, hotspot island chains. Constructive
  point-process that feeds the existing erosion sim naturally. Note:
  hotspot tracks REQUIRE plate motion — couples to the tectonic-history
  question (one-shot upheaval, below).

- **The 5–460 m band has no process — only decayed noise** (agreed
  2026-07-20, session-4 gen review; the primary mechanism behind "locally
  everything looks flat", and it is world-wide, not a mountain thing).
  Below the deep sim's 460 m floor, elevation is midpoint jitter off a
  per-provenance roughness budget decaying ×0.55 per halving: total
  sub-460 m relief ≈ ±26 m in an orogeny, ≈ ±4 m at walking wavelengths,
  ≈ ±1 m on a craton. No gullies, ravines, outcrops, knickpoints, talus,
  or hillslope-scale stream incision — the band where travel gets
  interesting is empty. S9's C-refinement (bounded regional refinement,
  halo theorem, `deeptime/refine.rs`) was designed for exactly this and
  sits unbuilt; it is the ratified extension point, not a bandaid.

- **The next arc after planet gen: back to block primitives + the artful
  procedural dressing** (user, 2026-07-20, stated as anticipation — filed
  so the sequence is on the record). "Jittering structural partials,
  deposits of loose partial mixes from grinding and hydrology below
  surface, stepped partials for loose material landscapes (the topsoils,
  sand dunes, etc.)." This is the collapse-tier rendezvous of four filed
  threads: the dormant 3c-2 partial-height loose rendering (content/sim
  half missing), method rule 5 (unsimulated remainder → procedural
  tricks), the monolith problem (thick units need partings/defects), and
  the new eolian dune-FIELD regions (journal/0034) whose individual dunes
  are exactly the stepped-partial landscape the user names. Also where the
  sub-460 m process band's outputs get their look.

- **Olivine reads as exceedingly common and surface-visible (user field
  report, 2026-07-20, during the ore conversation).** Olivine is the SOLE
  accessory-inclusion member (CLASS_ACCESSORY_MAFIC's only entry), so every
  igneous accessory event in the world is olivine — mono-culture by roster,
  not by mechanism. Its presence gate + fixed pore-eighths
  (`emplace_accessory`, geology.rs ~207) and any-depth formation window are
  the tuning surface; whether "exceedingly common" is a gate constant, the
  sole-member effect, or surface exposure bias of extrusives is UNDIAGNOSED
  — measure before touching. The ore work (geology.md § ore DECIDED) will
  both diversify the inclusion roster and make grade meaningful, which may
  resolve the perception without a tuning bandaid.

- **"Ours is not a deep world right now — you can keep digging into
  no-variety" (user field report, 2026-07-20).** Below the recorded
  deep-time strata the column is undifferentiated basement; depth is
  monotonous by construction. Connects to: recorded overburdens ≤ ~100 m
  (coal-rank note in dc-core geology.rs — the record itself is shallow),
  the § 5 burial/diagenesis/metamorphism engine (sketch), exhumation
  (ratified — will surface deep rock but also needs deep rock to BE
  something), and § 8 groundwater/karst (slated deep hydrology). The
  vertical dimension is a mostly-unbuilt frontier; no diagnosis filed
  beyond this inventory.

- **Gen never writes Quarter/Slab shapes or debris volumes (user,
  2026-07-20, "need to think on that" — deferred, filed).** The contents
  model supports partial structure shapes and loose debris eighths, and
  3c-2's partial-height loose rendering ships dormant — but worldgen emits
  only Full-shape voxels with no debris, so none of it is exercised in a
  generated world. This is the generation half of the filed
  block-primitives arc (stepped-partial loose landscapes: topsoils, dunes,
  talus; jittered structural partials at outcrops). Design owed: WHERE gen
  should emit partial shapes (weathered outcrop edges, scree, soil
  horizons) and what the deep-time/collapse seam is. Pairs with the
  sub-460 m process band and the pore-packability rule (materials.md).

- **The `history.rs` reject-don't-crash skip is SILENT** (2026-07-20,
  from the circulation merge). When a relocated-settlement graph
  over-constrains the S2 pressure collapse, `history.rs` now skips the
  observation instead of panicking (integrator-approved) — but the
  pack-degradation doctrine (API.md) says degradation must be LOUD. A
  skipped world-history collapse currently emits nothing; it owes a named
  warning. Small.

- **Circulation is fidelity-correct but surface-invisible** (journal/0038
  record-walk, corrections #22). The ~30° Hadley desert belt is arid in the
  data (precip ~0.2–0.3, past the 0.32 biome threshold) but `collapse.rs`
  only bares the surface below precip 0.10, so it renders as grass — the
  eye reads elevation/temperature, and 30° is the greenest band. Small
  reconciliation slice, USER-OWNED appearance: decide how bare a subtropical
  desert should read (lower the bare threshold in the subsidence band, or
  raise subsidence magnitude, or add a distinct arid surface material short
  of full bare Dirt). Pairs with the amplitude walk once flag-plumbing
  lands — until then no flagged gen feature is walkable anyway.
  > blogworthy: "a climate the map can't see" — the gap between a
  simulation being correct and being legible.

- **Walk 0059 — the holes and the chunk patches are gone; the skin is still
  two flat colours** (2026-07-22, live walk at `--horizon 3 --fullbright
  --edges`; assets `0059-*`). Two of the session-close checklist's five
  questions answered YES: **no sky-holes** at two partial-rich stations
  (journal/0057 confirmed by eye), and **no 28.8 m chunk patches** — a 120 m
  top-down frame shows organic blobs with wandering contacts (journal/0058
  confirmed). The third answer is the defect: the surface reads as **exactly
  two flat colours with a hard one-voxel contact**, no mixed voxel anywhere on
  the skin. Cause (user-diagnosed, integrator-confirmed in code): the surface
  voxel is **not sliced from the column at all** — `ColumnFill::build`
  (`fill.rs:115`) lays the record's top at the surface voxel's *floor*, so
  plans[0] is the voxel *below* it, and `surface_class` (`collapse.rs:786`)
  paints the surface voxel with the **dominant class of that lower voxel**,
  dithered to one member and emitted as a one-element `mixed_contents` call.
  journal/0055 changed the surface's *source* and kept its *branch*. Fix shape:
  `ColumnFill::build` takes the top partial, depth 0 covers record metres
  `[0, frac)`, `allocate_partial` (already exists, `fill.rs:329`) fills its `n`
  eighths from the units actually overlapping. Golden fingerprints move — that
  is the slice's deliverable, as already filed. **HELD** pending the seam-first
  cleanup, at the user's direction: the branch is a leaked LOD requirement and
  wants a declared provision, not another bespoke call.

- **The bare-cell fallback: a walker stood on paint over nothing**
  (2026-07-22, `record_hole_probe.rs`, uncommitted). At deep cell (488, 278)
  — world metres (106 938, 9 953) — the column reads **one voxel of
  `dc:dirt` over 49+ voxels of `dc:stone`**. `dc:dirt` twins only `LOAM`,
  which is registered in **no** class, so it cannot come from the record: it is
  the year-zero fallback, i.e. veneer paint, still under the player. `dc:stone`
  there is *contents-free* (granite/basalt/mudstone/sandstone/coal/peat/carb-
  mudstone all have their own blocks), i.e. unrecorded basement. The cell is
  real but thin: `H 0.25 m · 7 units`, beside a neighbour at **8.0 m / 379
  units**. Census: **0.2 % of land (91 of 44 265 cells) expresses 0 voxels**;
  **0.03 % of adjacent pairs (23 of 87 962)** are bare-beside-≥4-voxels. The
  user walked onto one on the first walk. **Hypothesis FALSIFIED in the same
  probe**: the integrator predicted north-south banding from `wind`'s per-row
  1-D transport lanes (`erosion.rs`: `load` is declared inside the `gy` loop
  and never crosses rows). Measured anisotropy **1.08× ON / 1.06× OFF** — the
  null. Wind does raise overall roughness ~29 % and *halves* the bare count
  (428 → 91) by depositing into scoured cells. Two real defects remain, both
  expression: (a) `regolith_at_voxel` samples **NEAREST** cell while
  `surface_at_voxel` beside it is **bilinear**, so soil depth is a hard-edged
  460 m Voronoi mosaic under smooth terrain — the DECIDED "no simulation-
  resolution edge may reach the eye" doctrine; (b) a column under half a voxel
  of record falls off the record path entirely into fallback paint, so the rare
  scoured cell renders as dirt-over-nothing rather than as honestly thin
  ground. The fractional-top slice fixes (b) by construction. Open design
  tension: `H` is a scalar and interpolable, the **record is not** (a
  variable-length unit list has no midpoint — the documented reason nearest
  was chosen).

- **Charcoal's premise expired and the code still encodes the conclusion**
  (2026-07-22, user-flagged, integrator-confirmed). `geology.rs:312`
  deliberately does not route `Biofacies::Charcoal` to a class, reasoning:
  *"mean ~0.035 m over 158 310 beds, and none of them survives the 0.9 m voxel
  quantization… a charcoal member would be dead content. The honest
  representation is an inclusion (pore/debris partial) — filed, not built."*
  **The load-bearing clause is now false.** One eighth is 0.1125 m, so a 3.5 cm
  bed is 0.31 eighths, and since journal/0055 allocation is *unbiased addressed
  stochastic rounding* — so that share wins a real eighth ~31 % of the time.
  Charcoal is expressible today, as exactly the inclusion the comment
  describes. What blocks it is small and specific: **no charcoal material is
  registered at all**, and `deep_class` routes a charcoal-tagged unit to its
  mineral host — so the fire bed's metres already flow through the mixture
  path wearing mudstone's identity. **Fourth instance in one day** of the
  ARCHITECTURE.md § "A summary is not an authority" class: a conclusion
  justified by a constraint we later removed, recorded in prose, with nothing
  to fail when the constraint went away (cf. corrections #29).

---

## NEXT SESSION — written at the 2026-07-24 BUILD-DAY close (supersedes every earlier block)

**Read first: `docs/design/material-behavior.md` §§12–14 + `north-star.md` § Deviations.** This
session turned the north-star's content layer into a *running engine*: the pass-runner landed at
the deep tier, R/H became **views of the inventory**, and the **first field pass** (geotherm)
stood up the condition-field vocabulary. The trust model was corrected to **no walls**.

### Shipped (journals 0089–0093, corrections #44–#47)
- **S18 — first-behavior weathering PLUMBING** (0089): sum-agent weathering on the keystone
  inventory. The walk found it **sub-voxel + a one-shot snapshot** — plumbing, *not* a behavior
  (#46/#47, stubs #17). This finding reshaped the whole arc.
- **Movement 1 — the deep-time pass-runner** (0090): `erosion.step` → 14 self-declaring passes
  on a shared `passgraph` kernel (A-4 guarded), byte-identical. The north-star runner, deep tier.
- **Movement 2a — R/H are views of the inventory** (0092): the inventory (== the record) is the
  authority; R/H derived (H = surface Loose, positional, cave-excluded). Byte-identical.
- **Far-LOD band-inversion fix** (0091): one `LodLadder` (coupled knobs → future player perf
  settings), warm floored geometry-safe, standoff gone. **Walk-CONFIRMED by the user.**
- **The geotherm — the FIRST field pass** (0093): `dc:field/temperature` from a tectonic
  gradient; retires `burial_temp_c` ENTIRELY (a real `T(depth)` is a field, not a `fn(unit)`
  value); coal moved (world-changing, user-blessed placeholder).

### Ratified (user's terms)
- **Agents SUM, not product** ("sum is honest, be brave"; byte-identity with the legacy retired).
- **Materials are minerals; named rocks are derived mixtures-in-forms** (strong lean, the mithril
  test; §12). **Ownership:** transformation edges input-owned (on the material), formation
  output-owned (a predicate), recipes registry-owned (separate); **geo = all on-material, two
  directions** (§12, user's strong leaning).
- **Material-aware transport + R/H unification** (§13): identity travels; sorting/placers/
  provenance/facies emergent = **clastic genesis**. Scratch-first reconcile.
- **Condition-fields + formation predicates as plain data over field-ids** (§14) — a **general
  primitive** (ecology/civ too), the geotherm its first field pass.
- **NO capability tiers** (Deviation #2, emphatic): mods author ANYTHING incl. field passes.
  **Safety/trust + the ABI/WASM spike DEFERRED, not gating** (Deviation #1).
- **Cadence = order (topo-sort) × rate (fractional-phase, `dt` = phase length)** — the ideas.md
  sketch reconciled into material-behavior.md §5.

### Falsified (assistant's own first) — #44–#47
`fully_resolved` dormant (#44) · member-dither single-octave, not chunk-anchored (#45) · S18
band sub-voxel — accept-by-mechanism not outcome, A-3 hand-fed magnitude (#46) · S18 is a
one-shot snapshot, not a process; seam in the wrong place (#47).

### Nothing running (agent-wise)
All agents completed + integrated; worktrees removed; main green (geotherm gate by name;
docs-only commits since). **A live `dc-client` on 7777 is the USER's `cargo run` — theirs to
close, not an orphan.**

### First things next session (all sequenced / unblocked)
1. **Movement 3 — weathering-as-a-process** (the visible band that failed sub-voxel): weathering
   runs *every epoch* on the runner riding `H`, real `dt`, accumulating. Acceptance = **≥1-voxel
   band probed headless in-slice**. 2a's inventory authority is landed for it.
2. **Movement 2b — material-aware transport** (§13, the big one): load multiset, Hjulström
   entrainment, settling deposition → sorting/placers/provenance. Appearance-changer → user's eye.
3. **Metamorphism — now UNBLOCKED by the geotherm:** `exhum` = P, geotherm = T → grade
   (schist/gneiss/marble). Where the tectonic gradient finally bites (deep crust). Retires stub #4.
4. **Igneous emplacement + the formation-predicate evaluator** (F1 ratified): the
   predicate-as-data machinery + exhum-driven outcrop structure, retiring stubs #5/#16.

### Owed / carried
Appearance walks (S18 band once M3; geotherm coal — low-priority placeholder) · **field-pass
migration** reminder (exhum/t_crust/drainage → real declared field passes) · geotherm
nonlinear/mantle-heat · **LOD fix (b)** — cold/warm material S-9 agreement (mixture arc) ·
structure-aware-fine-expression (within-cell, unsettled) · the poke-through lit-pass check ·
the **roster refactor toward minerals-as-atoms** (strong lean, seam-first) · don't
over-calibrate placeholders.

*(The 2026-07-24 MORNING block below is consumed; preserved as history.)*

## NEXT SESSION — written at the 2026-07-24 MORNING close (SUPERSEDED by the build-day close above)

**Read first: `docs/design/material-behavior.md`** — the substrate spec ratified this
session (companion to `north-star.md`) — then `docs/spines.md`. This session turned the
north star's content layer into a buildable spec, landed its keystone, and diagnosed the
rendering the user kept finding into a single root cause.

### Shipped 2026-07-24 (journals 0086–0088, corrections #44–#45)
- **The material-behavior substrate SPEC** (`material-behavior.md`): forms (closed machine
  set: structure/loose/pore-fill/fluid; void=complement) + the machine-complete transition
  graph (edges = the process catalog) + agents (folded rate-terms carrying a `cause`) +
  passes (cellular run-edges / field compute-and-plant) + the deep-cell inventory as the
  fill contract one tier up + commit-as-**appended-facts** (apply-time edge logging;
  provenance = base+facts, addressed to the material's lineage; pass/driver derived-and-
  displayed).
- **THE KEYSTONE MERGED** (0088, gate-verified by name): deep-cell working inventory +
  transformation-fact ledger — `commit_chapter` diff-and-append, byte-identical identity
  default over 25 600 real cells, provenance read. Single-edge shape; apply-time logging +
  `cause` land with the first real behaviors.
- **The contents inspector MERGED**: `world_get_contents` + F3 HUD; surfaced that the
  runtime stores only `Block` and re-derives contents (edit-blind, S-2).
- **The rendering "haunting" DIAGNOSED to ONE root cause** (read-only, planning held): the
  haunted far LOD (a 176 m standoff *inside* the L1 ring), the chunk checkerboard, and the
  loud 16-voxel member squares are all the **coarse ~460 m facies field point-sampled
  instead of interpolated** (S-4 unmet); the member squares are **one octave of value noise
  at chunk wavelength** — fix = octaves, not resolution.
- Earlier this arc: **A1 block↔material collapse**, the **north-star refinement**
  (field-solvers→content; cellular/field passes; tier deferral), **S-9**, and the **ROADMAP
  staleness reconciliation**.

### Ratified 2026-07-24 (user's terms)
- **Retire the stored `Block` summary.** *"Nobody wants one arbitrary material component…
  the only honest answer is the contents."* → one `identify(pos)` tiered by honesty; `classify`
  demoted to a derived rung. (Sequenced with reasoning + continuation slot.)
- **Retire class-member into hierarchy + genesis passes.** *"Rock distribution should come from
  passes that model the honest genesis of rocks… a modder authors a leaf, the pass marches the
  parent's leaves and derives distribution purely from properties… richer model, richer model,
  richer model."* Seam-first; the win is richer genesis physics, not re-housing. (Sequenced with
  the steelman recorded.)
- **commit-as-appended-facts + `cause` + apply-time logging**; the **six-variant intermediate
  atom rides** to the 2-variant end-state at Crux 2; **A1's within-class appearance change**
  accepted on the identity argument, sight-unseen.
- **The "slice-of" sequencing principle** (added to the session-workflow skill): sequence not
  only a first slice but a reserved continuation slot — never lose what a slice was a slice *of*.

### Falsified 2026-07-24 (assistant's own first)
- **#44** — assistant claimed `column_summary`'s `fully_resolved` is live in the renderer; it is
  dormant in dc-core (two unrelated systems welded). Verify live-code claims against code, not
  the doc of intent.
- **#45** — assistant + user both assumed the member dither is chunk-anchored/neighbour-blind;
  it is world-anchored and C0-continuous, the defect is single-octave. Read the noise function
  before prescribing its replacement.

### Nothing running
All agents completed and integrated; both worktrees removed; main green (verified by test
name/count), tree clean, no lock, no port-7777 process.

### First things next session (all Sequenced with full reasoning)
1. **The honest identity surface** — `identify(pos)` unifying `world_get_contents` (Near) and
   `classify` (Far), tier-flagged; start draining the `Block`-token consumers. Continuation:
   the runtime edit-fact overlay (break-gives-the-real-mixture — the first runtime-process
   milestone).
2. **Genesis-passes, seam-first** — convert `DepTag→material` (`[S2]`) to a property-reading
   declared pass, byte-identical; then enrich the genesis physics + octave materialisation
   (which *is* the member-squares fix).
3. **Owed:** a **spine-audit** (much shape landed live this session); the LOD **cold-over-warm**
   correctness fix (warm to the nearfield edge) when the mixture arc opens; the LOD **warm-cache
   lifetime** live-probe residue (the palette-quant 460-vs-28.8 was settled — member stepping).

*(The 2026-07-23 close below is consumed; preserved as history.)*

## NEXT SESSION — written at the 2026-07-23 close (SUPERSEDED by the 2026-07-24 close above)

**Read first: `docs/design/north-star.md`** (now CLAUDE.md read-first item 0) —
the ratified target architecture, and this session's spine. Then `docs/spines.md`.

This was a landmark session: the north star went from *design* to *ratified and
de-risked on real code.*

### Shipped 2026-07-23 (journals 0078–0085, corrections #40–#43, spike S16)
- **The north star** — designed, ratified, CLAUDE.md read-first, compliance-wired
  (0081; `docs/design/north-star.md`). Native engine, uniform self-declaring
  Pass/Material/`ctx`, tuning-as-data, tiered backend (native `abi_stable` /
  WASM sandbox) behind ONE authoring shape, **everything through the SDK route**
  (defaults are the SDK's completeness proof).
- **The north star, DE-RISKED** — weathering wears the Pass/Material/`ctx`/
  Transform shape **byte-identical** (0085/S16), purity enforced structurally.
  The keystone (deep-cell material inventory = Crux 1's storage atom) is named,
  and the behavior-rate-is-a-fold-over-agents refinement surfaced.
- **The perf window** — observability instrument (0080; it overturned its own
  suspect, #43), async-offload of all meshing (0083) + per-task generator (0084)
  → **+20 % frames** (942→1135), world byte-identical.
- **Amplitude retired** (0079, #41): erosion budget is not the relief lever
  (equilibrium); reframed to deep-field relief generation.
- paleo_temperature seam + collapse-tier Providers channel (0078); render-first
  falsified into a guard test (0082, #42); the audit-misquote correction (#40).

### First things next session (all ratified-ready)
1. **Crux 1 / the deep-cell material inventory — THE KEYSTONE.** Both the
   block↔material collapse's storage atom AND the material-behavior model
   converge here (S16 named it; the weathering spike thin-adaptered around it).
   Recon: `docs/audits/2026-07-23-block-consumer-inventory.md`. Ratified atom:
   `Block = {Air, Material(MaterialId)}`, niche for a 1-byte atom; the deep cell
   needs a per-cell material multiset. **This is the next foundational slice.**
2. **The cadence model** — user's fractional-phase scheduler sketch
   (`ideas.md § Pass cadence`). Compare to the actual deep-sim loop; the first
   fork is "agents as terms in one pass vs agents as passes with own cadence"
   (S16's finding meets the scheduler). Design thread, not a build yet.
3. **The ABI/WASM boundary spike** — the other north-star de-risk; independent of
   runtime sim; locks the SDK shape (`abi_stable` vs `repr(C)` vs `wasmtime`).
4. **The agent-set-reduction refinement** to fold into `north-star.md` (behavior
   rate = fold over agents, not fixed product; the karst/dissolution door).

### Field reports to diagnose (Observed, this session)
- **Far-field LOD reconstructs differently pre-visit vs post-visit** (user; a
  reduce-vs-synthesize material-distribution discrepancy) — user will elaborate.
- **Perf throughput ceiling at terminal velocity** (offload delayed onset, same
  ceiling) — measure with a per-thread-attributed instrument first.

### Owed / carried
- The **perf instrument's per-thread-role attribution** (also the overlay's need).
- The **detection hook** for the async-offload edit-corner (sim/NPC exposure).
- **Fires-pass onto the shape waits on ecology** (ex-nihilo vegetation — deferred,
  not the fires proxy; weathering was the shape-teacher instead).
- The **duality (define-once-run-in-both) validation is deferred to the first
  runtime-process milestone** — there is no runtime process sim yet (only block
  edits); the shape was de-risked in deeptime only, with the `ctx` kept
  granularity-agnostic so it isn't accidentally deeptime-only.
- A **spine-audit** is owed — the north-star shape is now instantiated in real
  code (`weather_behavior.rs`); the doc wasn't updated by the (unmerged-then-
  merged) spike.

*(The 2026-07-22 evening close below is fully consumed; preserved as history.)*

## NEXT SESSION — written at the 2026-07-22 EVENING close (SUPERSEDED by the 2026-07-23 close above)

**Read first: `docs/spines.md`** (item 0 in CLAUDE.md) and, for material/
identity work, **`materials.md`'s two new DECIDED entries** (one namespace;
transformation axes). This was the densest day in the journal: **nine entries,
0068–0077**, two corrections (#38 the Small control has a record; #39 the
integrator's own bias-sign error, majority-amplifying not toward-50/50).

### Shipped 2026-07-22 (evening arc)
The **threshold-quantization migration, complete**: the S-4 square is now
*inexpressible* — `dc_core::coarse::CoarseField<T>` (journal/0075) forbids the
raw per-cell read at compile time, extracted from two shape-teachers (A1
share-blend 0072, B1 membership dither 0073). Two **marquee violations dead**:
the A-7 charcoal carve-out became a general thickness rule (0068, walked and
ratified 0071), and the **S-3 surface-branch violation** (0074) — the near
surface now routes through `ColumnFill` like every voxel; killing it bought
**−10.5 % collapse perf** and fixed a latent off-by-one. **FF2b-minimal**
(0070): the far field went volumetric, spines § 3 row 1 **consumed** (first
departure from the index). **The octree substrate** named and its node contract
v0.1 ratified (0069). The **erosion-budget dev flag** (0076) makes the
amplitude call walkable. The **entry-species probe** (audit) proved substance
≈proxy (≤12 %) but **form is the signal (1.8–4.8×)**.

### First things next session (both user-ratified, ready)
1. ~~**The amplitude walk**~~ **DONE — RESOLVED WITHOUT A WALK 2026-07-23
   (journal/0079).** The faithful probe found the erosion budget inert on the
   client world (relief unchanged 1×→30×) and a mechanism probe pinned it to
   erosional **equilibrium** (graded to base level; mean lowering 41 m, flat).
   The amplitude lever is the deep-field relief GENERATOR, not erosion rate —
   ROADMAP's oldest open NEEDS-RATIFICATION (erodibility amplitude, item 2) is
   answered and reframed as a relief-generation design pass. No walk of two
   identical worlds was spent. corrections #41.
2. **The block↔material collapse, first slice** (DECIDED 0077 / materials.md) —
   kill `classify`'s fifteen-name match + `_ => Stone` arm; dominant material
   wears its own identity; byte-identical where faces exist today. Then the
   Block-token consumer migration as a long-tail arc.

### Carve-out to track (spines § 4 #1)
B1's class draw uses the **coherent** source, not white noise (far-mesh cost);
its bias is majority-amplifying (#39) and **compounds** the user's cake
observation (minority phases guillotine at cell perimeters). Heir: the
`CoarseField` far-`summarize` register / a CDF-corrected source. Also owed
there: check whether the journal/0058 **member dither** guillotines identically
(unexamined).

### The rest, sequenced
- **Perf window** (promoted over the seam batch, which runs parallel): profiling
  slice (Tracy + vertical-drop baseline → ranked killer list; prime suspect
  `streaming.rs:42` sync main-thread gen), async-offload, then albedo-at-range
  (with the ranges-as-player-config rider). Then **the dressing arc**.
- Remaining `CoarseField` migration (A2–A4, B3, the conservation-pinned
  soil-depth mosaic behind an S-7 ledger proof); FF2b persistence + dirty-rail.

*(The morning close block that stood here — the spines/seams/five-falsified-
claims block — is fully consumed and preserved below the line as history.)*

---

## NEXT SESSION — written at the 2026-07-22 MORNING close (SUPERSEDED by the evening close above)

**Read first: `docs/spines.md`** — new today, and now item 0 in CLAUDE.md.
Eight shapes, six anti-shapes, and § 3, the index of **machinery that exists
and nothing calls**. It exists because the corpus was ahead of the assistant
**fourteen times** in one session — not because ideas were missing, but because
they were **already built and lost**. Then `journal/0060–0067` and
**corrections #30–#37**, five of which are the integrator's own errors.

**The compliance loop is live.** Briefs name the spines they ride; a worker who
believes a deviation is right makes a **loud plea** rather than shipping it
silently; compliance is part of integration review; carve-outs pass through
main-session discussion and user ratification into `spines.md` § 4. The
`spine-audit` skill is the other half — a periodic read-only sweep that keeps
the doc **true** while the workflow keeps it **applied**. It has never been
run; running it is a cheap first act.

### Shipped 2026-07-22

Provider seams (`outcrop_at`, `wave_energy`, `parent_p`, `depth_to_water`) ·
the module split that made conversions concurrent · `Option<fn>` slots, so
absence is **structural** rather than inferred from fn addresses (#32) ·
**S15** coarse capacity, GO · **horizon 6 proven survivable** · coal on burial ·
charcoal as an inclusion · `spines.md` and the closed loop.

### The one outstanding ratification condition

The user accepted coal **conditionally**: the 8 m threshold must become a
**provider seam with the geotherm as its heir**. Dispatched at the close; if it
did not land, it is the first thing to finish. The calibration itself is
explicitly **not** under review — *"the calibration is fine, we aren't
answering deep questions about it right now."*

### The live design thread

**The recorder's entry species** — and it is now ONE decision, not two: where
**form** lives in the record, and what the **erosion sim reads**. If `Litho`
should become *(substance mixture, form)* rather than six proxy rocks, those
are the same question. Decided sequence for the material interface
(geology.md, DECIDED 2026-07-22): **seam now · MEASURE the class-aggregate with
a probe, at zero terrain cost · ship `f(substance, form)` ONCE.** Never
aggregate-then-form: each is a terrain-shape flip, and that pays the cost twice
for one conceptual change.

> **Step 2 MEASURED (2026-07-22, `docs/audits/2026-07-22-entry-species-probe.md`,
> probe `entry_species_probe.rs`).** Production Medium record. The **substance**
> error (the class-aggregate) is noise-scale: **≤ ±12 %**, **0 % of cells past
> ±1.25×** on any agent, and **4 of 7 classes ship a single member** so their
> aggregate ≡ proxy. The **form** error (loose `H` charged as lithified rock) is
> **1.8–4.8×**, saturating the ±5× clamp on wave/eolian/frost — **20–50× the
> substance term**, cohesion-driven (loose gravel 0.02 vs conglomerate 0.80).
> Dissolution: identically 0 in both worlds (no soluble member) — unbounded latent
> headroom for a carbonate pack member. **Verdict:** do NOT ship the aggregate
> step (measured support for skipping aggregate-then-form); go straight to
> `f(substance, form)` with **form the load-bearing half**. The rework's
> justification rests on the measured form error + the already-ratified
> pack-signature argument, not on vanilla's (small) substance headroom.

### Hydrology, on a hard pause the user called

Opens from `water.md` — *not* beside it. `wet` is **three quantities**, one of
which (fire dryness) is not about a water table at all. The drainage wall:
**capture is coarse, expression is fine**, and its mirror for player
diversions, **fine cause, coarse propagation**. Most of the user's encounters
list does **not** collide with the wall. And the parked leaning: **one octree**
shared by FF2b, bulk flow and the statistical tier — recorded, deferred,
discoverable.

### Queued, none blocked

Seam conversions, partitioned by owing-system group so they run concurrently:
*materials/form* (`material_properties` + `is_granular`) · *paleoclimate*
(`paleo_temperature`) · doc-only riders (the `fits_in_pores` declaration, the
`exhum`/`t_crust` comment correction). **Not** `block_twin` — it touches the
ratified fill contract and wants thought, not a brief. The **surface-branch
fix** is still held, and it splits: the expression half is independent, the
far-field summarization half waits on the octree question.

### Operational lessons that cost real time today

- **Clean the crates a SIBLING built, not the crates you changed** — a false
  red on a dc-client atlas test came from a sibling's dc-core.
- **Hold the build mutex around the cargo invocation, not the work session.**
- **Measurement agents must commit something early** — an unchanged worktree is
  auto-cleaned, and one was deleted mid-run.
- **An empty worktree is not evidence that an agent produced nothing.**
- **A window shorter than the period cannot tell flat from oscillating**
  (corrections #34 — journal/0051's famous `0.00 MB/jump` was phase, not
  flatness).

### FIRST THING NEXT SESSION (filed 2026-07-22 at the user's direction)

**Rework the charcoal carve-out into a thickness rule — DONE (journal/0068).**
Both name-keyed `Biofacies::Charcoal` exceptions (`litho_of_tag`, `deep_class`)
are deleted, the mirror test is restored to full agreement, and the general
**thickness-dominance rule** lives in `outcrop_at`'s identity `exposed_litho`
(`OUTCROP_DOMINANCE_WINDOW_M = 0.9 m`, a stated calibration). Charcoal is now its
own `Litho` and never outcrops a cell (measured: 0 of 297 025 Medium cells).
Authorized goldens moved, including — for the first time — the Small block hash,
which the brief predicted would not move; see journal/0068 for why (Small runs
always-on deep time; erosion *geometry* shifted). **Flagged NEEDS RATIFICATION**:
the Small control moving, and the ~40 % Medium outcrop change (deep-time terrain
shape). Coal-dig margin improved (16 → 19 vox over a floor of 15).

- **`block_twin` — a process naming fifteen materials** (anti-shape A-7,
  `docs/spines.md`; `dc-core/src/classify.rs:60`). Fifteen named identities plus
  a `_ => Block::Stone` arm whose own comment admits it swallows nine materials
  — so **any pack's new material summarizes to generic stone**. Deliberately
  **not** batched with the other seam conversions: it touches the ratified fill
  contract (`block == classify(contents)`, and "every member of a class shares
  a block twin", asserted in exactly one test), so a careless conversion could
  quietly weaken an invariant. **Wants thought, not a brief** — but it is owed
  work, not a decision already taken. Sequenced here so it stops living only as
  an exclusion note. Full entry: `docs/audits/2026-07-22-seam-inventory.md` (#7
  on the ranked shortlist).
