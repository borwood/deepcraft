# Roadmap — history

**Archived from `ROADMAP.md` on 2026-07-26, by STATUS, not by age.** The live board is
`ROADMAP.md`: *Sequenced* + *Observed* + *In flight* + the current close block — the parts
that must stay readable. This file holds the parts that are least often needed live and
whose narrative is already carried elsewhere:

1. **Shipped** — every entry keeps its **journal number**, which is the stable pointer. The
   journal holds the story; this holds the ledger.
2. **Observed — archived** — § Observed entries that stopped requiring a live read: user field
   reports the user has **struck**, and entries whose own bodies already declared them closed.
3. **In flight · Sequenced · Observed — archived 2026-07-29 (pass 2)** — the first
   sweep of § In flight and § Sequenced for archivability, plus the § Observed entries the
   2026-07-29 morning pass annotated in place rather than moving.
4. **Superseded close blocks** — each was consumed by the one after it.

**Age is deliberately NOT the axis.** A two-week-old `Observed` may be the liveliest thing
on the board; a Shipped entry from this morning is already history. Archive when an item's
*status* stops requiring it to be read, never when it gets old.

**This file is append-only in practice — do not prune it.** It is the record that makes a
"we already did this" question answerable.

## Shipped

- 2026-07-26 — **`production_* → golden_*`: the fixture helpers now say which world they
  build** (opened by journal/0106, sequenced by the staleness sweep row D-2). Pure
  housekeeping, **byte-identical, no golden moved**. `providers_common::production_field` →
  `golden_field`, `production_pregen` → `golden_pregen`, `rh_unification::production_field` →
  `golden_field`; the two `rh_unification` tests are now `..._over_the_golden_field`, and
  **`providers_golden::the_production_world_still_hashes_to_the_pre_slice_goldens` is now
  `the_golden_world_still_hashes_to_the_pre_slice_goldens`** — every citation of the old name
  in `journal/` and in the Shipped lines below refers to this test, and the rename is recorded
  in its own doc comment so a grep of the old name lands somewhere.
  - **Left alone, deliberately:** `deeptime::production_config` / `production_config_with`,
    `water::coarse::CapSummary::production()`, `tests/geotherm.rs::production_field()` (that
    one really *is* seed 1337 / Medium since 0106), and
    `s18…::production_scale_saprolite_band_reaches_at_least_one_voxel` (honest, 1337/Medium).
    A **config** named production is not a **world** named production.
  - **⚠ The Sequenced entry named the wrong seed.** It said the misnamed helpers build
    `0x0D5EED572026`; they build **`0x0B0A_57EE_0059`** (journal/0106's audit table has it
    right). `0x0D5EED572026` is the *warm reference* world — geotherm's
    `warm_reference_field()`, `organic.rs`, and `s17_deep_cell_inventory.rs`. Both are worlds
    nobody ships, so the item's conclusion stood, but the transcription conflated two
    different non-shipped fixtures.
  - **Two prose-only residuals, NOT renamed** (no `production_*` symbol, outside the item):
    `s17_deep_cell_inventory.rs`'s *"over a real production `DeepField`"* (it is
    `0x0D5EED572026`/Small) and `s18_first_behavior_weathering.rs`'s *"the canonical
    production golden world"* (it is `0x0B0A57EE0059`/Small). Both helpers are honestly named
    `small_field` / `small_pregen`; only the module doc overreaches.

- 2026-07-26 — **THE JOINT CALIBRATION: BUILT, MEASURED, AND DELIBERATELY OFF**
  (journal/0114). `EROSION_CALIBRATION = 45` behind **`DeepConfig::calibrated_rates`,
  default `false`**, identity path pinned by name. **Production is byte-identical.**
  - **⚠ THE AGENT REFUSED THE BRIEF'S "DEFAULT ON", AND WAS RIGHT TO.** It found
    **stubs #29** (`ROADMAP.md` § Sequenced; re-scoped 2026-07-26 — the real defect is grid
    instability, and the true hollow count is **1,377**, not 148) and would not ship unfilled pits
    into the world on its own authority while the user was away, on a number that misses its
    target band. *Deviation is allowed; silence is not — the loud-plea doctrine working as
    designed.*
  - **Behind the flag:** denudation **0.0110 → 0.4142 m/Myr** (D3 0.0107 → 0.1271),
    **D1/D4 0.027 → 0.91**. It enters the published **floor band** (McMurdo/Atacama 0.1–1) —
    so the world stops being below *every* landscape measured on Earth — **but the 1–10 craton
    target was NOT reached, and is unreachable at any multiplier** (stubs #27).
  - **The multiplier was DERIVED, not chosen:** four criteria against three published bands
    (relief within 5 %; mean regolith inside the 30–60 m deeply-weathered-shield range; D1 in a
    published band; D1/D4 → 0.91), **independently corroborated by the world's own Airy
    ceiling**, `U/0.152 = 2.70 m/Myr`, from two densities and a measured uplift.
  - **stubs #24 CLOSED.** `erosion_budget` now reaches all four rates **including
    `diffusion`**, through one `scale_erosion_rates` with two consumers.
    `the_erosion_budget_reaches_every_rate_including_diffusion` **would have failed on every
    prior commit**, and `erosion_budget: Some(45)` is bit-identical to
    `calibrated_rates: Some(true)`.
  - **corrections #59 — a FORCED scope expansion, not an opportunistic one.** `energy_band`
    and `competence_ceiling` were **absolute thresholds secretly keyed to `k_transport`'s
    value**. Left alone, the calibration would have relabelled every site High-energy and
    **erased the facies gradient**. Now relative to `REFERENCE_KT`, bit-identical at the
    historical value.
  - **🔴 IT ALSO FALSIFIED A CLAIM FROM journal/0111 THAT THE INTEGRATOR PROPAGATED.** D1's
    `creep_to_sea_m` is a **gross** land→sea edge flux, and the ±35 m sea stand cycles four
    times — so cover ferried across, stranded and ferried again is **counted every time**. The
    arithmetic does not close (63.6 m bedrock + 39.2 m stored vs 207.1 m claimed export).
    **D3 is the sound instrument once fluxes are large; D1 is an UPPER BOUND**, and
    journal/0111's *"two independent instruments agree to 2.4 %"* was a **low-flux
    coincidence, not a cross-check.** Candidate **corrections #60**. *The denudation numbers
    above and in journal/0111 stand as an upper bound; the direction and the ~10³ magnitude of
    the defect are unaffected.*
  - Outputs **checked, never targeted**: sand now moves on **0.095 %** of land where it was
    zero (and it came from the landscape — the competence ceiling is invariant under the
    calibration by construction); the **facies gradient is still a NULL**, fining ratio
    unmoved. Reported rather than chased.
  - No golden moved; new `GOLDEN_*_CALIBRATED` pin the behind-the-flag world so it cannot drift
    unmeasured. Gen time −0.17 s (noise). Behind the flag residency would **fall 59 %**
    (107 → 44 MiB) because thicker beds merge. Gate +16 s.

- 2026-07-26 — **FLOW (b′): HYBRID `p` — water that stays in its banks** (journal/0113).
  The MFD convergence exponent is now **spatially varying**: `p` ramps **1 → 16** on the
  channelisation index **`χ = A·S²`** (Montgomery & Dietrich 1988/1992) and **switches to
  single-receiver above `χ = 1.2e−1`**. Fixes journal/0109's measured defect — uniform `p`
  applied *hillslope sheet-flow behaviour inside channels*.
  - **THE NUMBERS.** Peak catchment **84 → 298** (3.5×). p99 land catchment **70 → 164 —
    beating D8's 127**, because a tree is thin in the mid-range while dispersive hillslopes
    feed each channel from a *fan* and the channel then keeps it. **Land cells with a
    catchment > 100: 0 → 2,270** — *under uniform `p` the shipped world had **not one cell**
    draining more than 100 cells; there was no trunk network at all.*
  - **`A·S²` AND NOT `A` ALONE — the choice that saved the previous slice.** Deltas, fan tops
    and braid plains have the **largest** area, so an area-only law makes them the *most
    convergent* ground on the world and would have destroyed what journal/0109 bought.
    `A·S²` returns them to the dispersive side for the right physical reason: **a delta is
    where a channel loses its confinement.**
  - **Simultaneous divergence SURVIVES — 95.8 % retained** (7.23 M of 7.55 M within-epoch
    pairs), and the per-chapter count *rises*. Concentrating the trunk did not cost delta
    representability.
  - **The hard switch was MEASURED, not preferred:** `p = 16` everywhere lifts peak catchment
    only 84 → 145, because a rival at 90 % of the steepest slope keeps 19 % of its weight at
    `p = 16` and suppressing it needs `p > 44`. *A fifth per hop, down a fifty-hop chain, is
    everything.* And `χ_hi` was fitted **by a swept curve**, not by eye — 1.2e−1 maximises p99
    *and* top-1 % share *and* retained divergence; at 3e−2 the single peak is biggest while
    both honest concentration measures are **worst** (*parallel threads that never merge are
    not a network*).
  - **stub #22 measured rather than argued:** the 1 % floor costs 2.1 % of peak catchment and
    0.3 % of divergence, saves 1.5 % of record — and the leak is **opposite in sign to the
    stub's own argument** (the floor leaves the world slightly *less* concentrated). A knob is
    not an heir; **the stub stays open.** New **stubs #26** (thresholds fitted to one world).
  - Gen time **+0.04 s** vs uniform (the integer-rounded ramp keeps `powi`; the channel switch
    is *cheaper* than a partition). Record 58.13 → 63.80 MiB. Gate +13.8 s. Goldens moved on
    the shipped world; **three fixed points held bit-exact**. **No walk owed** — the world is
    ~1000× under-energetic until the calibration lands, and the visible null is not this
    slice failing.
  - **🔑 WHAT THE CALIBRATION SLICE MUST KNOW:** it must calibrate against **this** solve, not
    0109's. `mfd_exponent` is now the **hillslope** end (default 4.0 → 1.0) — also read
    `mfd_exponent_channel`, `mfd_chi_lo`, `mfd_chi_hi`. `k_bedrock`/`k_transport` were seated
    against D8 and the discharge they now see is **3.5× more concentrated at the trunk**, so a
    joint re-fit **will not scale linearly** from the 0109 numbers. And **stubs #26's
    thresholds should be re-derived from the calibrated stream power rather than re-fitted by
    eye.**
  - **🔓 UNBLOCKS REFINEMENT PRIMITIVES / VISIBLE CHANNELS.** The substrate now concentrates:
    the flux record hands refinement a trunk network with discharge on one or two faces per
    confined cell, and genuinely split faces where flow is unconfined — **the Dirichlet data a
    boundary-value problem needs in order to have a channel as its solution.** *A channel could
    not have been refined out of the record as it stood yesterday.*

- 2026-07-26 — **FLOW (b): MFD — *simultaneous* divergence becomes representable**
  (journal/0109; background agent, worktree). Holmgren (1994) × Quinn contour width on the
  **free-surface potential**, `p = 4`, **on by default**; `mfd: false` is the byte-identical
  pre-MFD path, pinned by name. `p` is the **convergence exponent** and ~~**p → ∞ *is* D8
  exactly**, so the old solve is a **limit, not a deletion**~~ — **🔴 FALSIFIED 2026-07-26,
  corrections #58 (journal/0113).** The limit of `p → ∞` selects the steepest **slope**;
  `route_cell` selects the steepest **drop** — and journal/0109's own
  `the_partition_follows_slope_not_drop` pins the disagreement. *The integrator propagated
  this sentence into this entry from the slice report without testing it.* **It matters
  structurally, not pedantically:** that sentence was the argument that `mfd: false` lives
  *inside* the model's family, and reading it that way would have licensed **deleting a pinned
  path**. The single-receiver path is a **separate, pinned control**, not a limiting case.
  - **THE NUMBER.** Simultaneous, **within one EPOCH**: **0 → 7,548,646** `(cell, epoch)`
    pairs carrying ≥2 lateral out-faces; **395,452** `(cell, chapter)` = **16.64 %**; max
    **8** faces. The zero before was **structural** — within one epoch the old solve returned
    exactly one receiver per cell, so **no cadence setting could ever have produced this**
    (flow.md § 2.6). Temporal/avulsion divergence survives as the control (lateral
    56,281 → 397,520). *Note the 175,320 from slice 1 does **not** reproduce and is not
    recoverable from either row — (a) added vertical faces and 0096 counted boundary faces
    that lateral-only excludes. The probe's caption claiming otherwise was a published claim
    the gate could not check, and was fixed to print its own measured values.*
  - **THE TRAVERSAL FINDING, which is the elegant part.** The priority-flood pop order
    **did not need to change**: it is strictly ascending in `filled`, and every routed edge
    descends `filled`, so reversed it is a topological order of the **DAG** for exactly the
    reason it was one of the tree. **The tree was never what licensed the traversal — the
    potential was.**
  - **Mass on a DAG:** the **last weighted direction takes the residual**
    (`share_last = q − Σ earlier`), exact by construction — normalised f64 weights sum to
    `1 ± 1 ulp` and that error would compound unattributably over a thousand hops. **The
    value written to the record is the same `share` variable added to the neighbour**, never
    re-derived from weights: the **A-3 defence made structural**. The incision clamp
    generalises to *below the **lowest** receiver* and **reduces exactly** to D8's rule in
    the single-receiver limit. Energy slope is the share-weighted mean, because stream power
    is `Q·S` and any other choice creates or destroys erosive work.
  - **ALL GOLDENS MOVED — authorized, and it is a NULL IN THE VIEWPORT.** **71.22 %** of land
    cells changed receiver; **1 cell of 297,025** moved > 1 m (mean **0.027 m**). Confirming
    asymmetry: on the record-less Small world only `blocks` moved — `materials` and `table`
    are byte-identical, the signature a *terrain* change predicts and a *record* change
    forbids. **Do not spend a live walk on 2.7 cm**; tour-map first, and expect a null.
  - **🔴 THE NUMBER THAT SHOULD WORRY A READER: peak catchment 1,245 → 84 cells.** Uniform
    `p` **does not concentrate flow** — which is *why* the terrain barely moved. MFD lowers
    both `Q` and `S` everywhere, so it is **systematically less erosive than D8 at fixed
    coefficients** (total load −16 % while load-carrying faces ×2.76 — mass conserved, less
    material *mobilised*). **Owed:** hybrid `p` (a function of area/slope, or single-receiver
    above a channel threshold) and recalibration of `k_bedrock`/`k_transport`, which were
    seated against D8. *The agent's verdict, and it is right: uniform `p` is the simplest
    correct thing and the wrong long-run shape.*
  - Landed with it: **corrections #54** (the *"head unlocks MFD"* claim, false, struck at all
    four sources) and **stub #22** (`MFD_MIN_WEIGHT` is a **record**-affordability constant
    applied **inside the solve** — filed as a **plea**, not assumed). **`recv()` upgraded
    SUPERSEDED → A-1**: it is now the *argmax share*, a projection existing only because two
    consumers want one arrow per cell — so **continuation (e) should DELETE, not supersede**,
    and should expect `area` to be the harder problem now that it is a dispersed quantity.
  - Gen **+4.56 s** (21.18 → 25.74), residency **+12.84 MiB**, gate **+22.7 s**.
  - **The process lesson, kept because the agent kept its own wrong prediction above its
    correction:** it wrote a confident forecast of braided fan heads and wider riparian bands
    **before** measuring and was wrong by orders. *"Routing is upstream of erosion so the
    world will move" is a **sequencing** argument, and sequencing arguments say nothing about
    **magnitude**.* Same error shape as *"head unlocks MFD"* — twice in one slice.

- 2026-07-26 — **S20 2c: the compact 8-byte fact** (journal/0108; background agent, worktree).
  `Fact` **16 B → 8 B, zero padding, every axis retained**. Resident ledger on the shipped
  world **17.45 → 9.57 MiB** measured; the per-depth projection **973.40 → 504.50 MiB**,
  reproducing S20 § 3 row 2c exactly — now from **shipped types** rather than the spike's
  candidate structs. **User-ratified as option 3 + 2c**; this is the 2c half, and the pager
  is the reserved continuation.
  - **The two levers only pay together, now a live assertion about a shipped type:** an edge
    id alone reclaims four bytes the `f64`'s alignment pads straight back, which
    `Fact<FracM>` still demonstrates at 16 B. **The edge id's real value is validation, not
    bytes** — its only constructor is the declared-transition predicate, and `apply_edge`
    refuses **the move**, not merely the record. **S-8 strengthened from a convention into a
    compile-enforced authority.**
  - **Eight of S20's nine predicted tolerance breaks did not happen** (corrections #53), and
    the reason is the design rather than luck: § 4.2 assumed `Fact` stores f32 everywhere,
    but the shipped design narrows **once at persist**, so the eight sites reading the f64
    accumulator are f64-vs-f64 and pass **untouched at their original bounds**. That includes
    **`weather_inventory.rs:504`** — the one site whose `1e-9` was a deliberately *physical*
    bound and which the spike correctly named as the only defensible casualty. **It
    survives.** The single tolerance that did move (`:549`, exactly as § 4.3 predicted) is
    **re-derived, not widened**: `8 · 2^-24 · |x| + 1e-12`, derivation published beside the
    constant. *Narrowing at every add instead of once at persist measures **570.6× worse** at
    200 firings — the discipline earning its keep, measured.*
  - **Per-world `EdgeDict`, derived by scan and never stored** (a second copy can disagree
    with the record it describes). Turns a registry change from *silently reinterpreted* into
    *detected*. **Exactly 1 entry** on the shipped world — and that dictionary is the
    instrument that will show **stub #16** being discharged, because one entry is what a
    single flat granite basement looks like.
  - **A-1 guarded, and it is the crux:** the cheap compaction here was an **axis drop**, and
    an axis drop is A-1 wearing a fact's paperwork. Nothing is deleted —
    `every_axis_survives_the_narrowing`. **Stub #21** filed (positional mixed-radix edge id,
    hard-capped at 51 materials by a compile-time assert).

- 2026-07-25 — **`dc:field/head` declares the terrain it reads — the revision AND the writer
  that supersedes it** (journal/0107; background agent, worktree; **USER-RATIFIED with the
  consequence attached**: *"it should declare what it reads and we eat it if it changes the
  physics. The world is a scratch pad right now."*). Closes the ⏳ second half of the Observed
  entry *"A TIE-BREAK IS DECIDING PHYSICS AGAIN"* (journal/0104 closed the first).
  `dc:field/head` declared `reads: [Routed]` while its body built the ground surface `R+H` via
  `grid.surf_at` and handed it to the solve as its **seepage cap, lake datum and free-surface
  boundary** — so which terrain revision it saw was the id tie-break's call, and the old
  defence (*"its position never affects the terrain"*) answered the wrong question: it affects
  **the head field's own values**, and the vertical flux recorded from them.
  - **The revision is `Forced`, in every cfg path** — determined from the code: between
    `forcing` and `transport` the roster runs only `drainage`/`frost`/`geotherm`, none of which
    mutates `R`/`H` (on the tectonic path `apply_thickening` grows `t_crust` alone). It is also
    the revision the pass **should** read: `filled`, `routed` and `area` are snapshots
    `build_surface` took from that same terrain, so a later `ground` would put the seepage cap
    and the free-water anchors on two different landscapes. No cfg-selected slices needed.
  - **The audit's own prescription — *"declaring `Forced` is free, and it pins it"* — was HALF
    WRONG, and the ROADMAP's *"verify that claim before trusting it"* is what caught it.** A
    `reads` edge on a revision token orders you after its *producer* and says nothing about the
    pass that overwrites the same plane next — that pass writes a **different token**. Every
    erosion pass is braced on the far side by a forward edge into the stages after it; a
    **sidecar** has none and floats.
  - **The fix is the PAIR, plus the chain's missing link.** `dc:deep/transport` mutated `R`/`H`
    while declaring only its `Energy`/`DeltaH` by-products, so nothing named the moment the
    ground surface first changes each epoch. **`DeepAxis::Incised`** closes that hole
    (`transport` writes, `weather` reads); `head` now declares `reads: [Routed, Forced]` **and**
    `reads_prev: [Recorded, Incised]` — after the writer that produced its terrain, before the
    writer that supersedes it. **Generalisation worth keeping:** where a plane has several
    revisions per epoch, declaring the revision you consume pins **one side only**; pin the
    other with an anti-dependency on the **next** revision (never the last).
  - **`dc:deep/climate`'s terrain lag declared too** (owed by journal/0104): `reads_prev:
    [Forced]` — the **first** revision of the epoch, ONE slice, no cfg selection. The obvious
    three-slice version (`Settled`/`Compensated`/`Diffused`) would pin strictly less, leaving
    climate free to slide past `forcing` and `transport`.
  - **NOTHING MOVED, and the slice was authorised to move things.** Pass order byte-unmoved,
    all 17: `climate · expose · tectonics · forcing · drainage · frost · geotherm · head ·
    transport · flow_record · weather · diffuse · isostasy · deposition · eolian · wave ·
    biotic`. Neutrality **proven directly, not by appeal to a hash** —
    `the_terrain_revision_declarations_are_schedule_neutral` rebuilds five rosters with the
    pre-slice declarations and asserts the order is identical. Vertical-flux record re-measured
    against the journal/0098 baselines (`examples/head_field_probe.rs`, seed 1337,
    `Extent::Medium`, run before and after within the hour): **307,364 entries · 306,227 DOWN /
    1,137 UP · 131,586 columns (44.301 %) · magnitude mean 0.501013 / p95 0.720093 / max 10.0 ·
    60 artesian (max excess 2.935 m) · 1,044 confined — every figure identical.** Terrain
    byte-identity held.
  - **The rename-proof guarantees.** `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename`
    picks up the `Incised` side (and `climate`, now a lagged reader too); the `Forced` side gets
    its own — `dc:deep/aaa_head` would win the tie-break against `dc:deep/forcing` and still
    cannot be scheduled ahead of it. Plus
    `the_head_field_is_pinned_into_the_terrain_revision_it_reads` across four configs.
  - **Why it is capabilities work:** a self-declaring pass's declaration is what a WASM sandbox
    will be built out of — reads become the capability grant. A pass that can reach a plane it
    never named has a hole in its sandbox, not a documentation defect.

- 2026-07-25 — **`reads_prev` is a mechanism: the anti-dependency edge in `passgraph`**
  (journal/0104; background agent, worktree; **USER-RATIFIED** as option (a) of the Observed
  entry *"A TIE-BREAK IS DECIDING PHYSICS AGAIN"*, now struck through in `ROADMAP.md` § Observed). `DeepPass::reads_prev`
  declared *"I read LAST epoch's value"* and was **handed to nothing** — it appeared in `runner.rs`
  and in no other file under `crates/`. Since the deep-time planes are overwritten **in place**,
  whether a lagged reader actually saw last epoch's value came down to whether it ran before or
  after this epoch's writer, and that was decided by `passgraph`'s **id-lexicographic tie-break**.
  `dc:field/head` read last epoch's strata record **only because `dc:deep/head` sorts before
  `dc:deep/deposition`**; renaming it `dc:deep/hydraulic_head` would have silently changed the
  physics with every test green. Found twice in two sweeps.
  - **The fix is a second EDGE KIND, not a declaration change.** `reads` is a true dependency
    (RAW): writer before reader. `reads_prev` is an **anti-dependency** (WAR): **reader before
    every writer**. Folding a lagged read into `reads` points it the wrong way *and* closes the
    loop-carried feedback into a within-epoch cycle — proven, not asserted, by
    `folding_a_lagged_read_into_reads_reverses_it_into_a_cycle`.
  - **BYTE-IDENTICAL. The production order is unmoved**, all 17 passes: `climate · expose ·
    tectonics · forcing · drainage · frost · geotherm · head · transport · flow_record · weather ·
    diffuse · isostasy · deposition · eolian · wave · biotic`. Six lagged readers produce twelve
    new edges (`expose`/`frost`/`head` each → `deposition`/`eolian`/`wave`, the writers of
    `Recorded`; `weather`/`diffuse`/`eolian` each → `biotic`) and **every one of them points from
    a pass already ahead of its target** — *the physics was right and merely unenforced*. Green by name, unmoved: `the_production_world_still_hashes_to_the_pre_slice_goldens`,
    `order_reproduces_the_erosion_step_phase_order`, `order_ignores_registration_order`,
    `the_biology_erosion_lag_must_be_loop_carried_or_the_runner_rejects_a_cycle`,
    `the_head_field_is_a_declared_field_pass_the_flow_record_reads`,
    `weather_inventory_is_absent_off_and_a_declared_cellular_pass_on`.
  - **The rename-proof test is the slice.**
    `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename` renames each of the six
    lagged readers to an id that **provably loses** the tie-break against every writer of the axis
    it lags on (asserted hostile before proceeding, so it cannot pass vacuously) and asserts the
    reader is still scheduled first — all six would fail under the old kernel. Plus
    `every_lagged_read_in_the_roster_is_ordered_before_its_writers` across five configs, and the
    kernel twin `a_lagged_read_outranks_the_id_tie_break` **with its negative control** (the same
    roster, lag undeclared, schedules the wrong way round).
  - **Audited all six `reads_prev` declarations against their bodies: every one is TRUE** — and
    none of them was true *for a reason*. `reads ∩ reads_prev` on one node is now a named
    rejection (`GraphError::ContradictoryLag`): against another writer it is a 2-cycle, but
    against itself the opposed edges cancel as a dropped self-edge and would pass silently.
    `pipeline.rs` hands `reads_prev: &[]` **structurally** — a one-shot DAG has no previous value.
- 2026-07-25 — **Two draws, one number: a randomness PROVIDER, and every caller gets its own band
  of the hash** (journal/0105; background agent, worktree). User-ratified twice — *"this needs
  fixed either way. **Decorrelate**"*, then *"we could have some utility system that is a
  **randomness provider** … ensuring that every caller gets allocated their own band of the hash …
  it would be optimal to fix it with a **construction guarantee**."* Discharges the Observed loose
  end journal/0103 filed against `pore_rider_share`'s comment, and retires the convention that
  produced it.
  - **THE MECHANISM.** `dc_sim::statistical::rng`: a `Domain` trait (one named purpose = one band),
    a `Draws` stream (`Draws::of::<D>(seed).unit(&[addr…])`), and a `draw_domains!` macro declaring
    a crate's whole set in one list. **Two ways to collide, both closed at compile time**: a
    duplicate *salt* trips the `const _: () = assert!(all_salts_distinct(ALL_DOMAINS))` the macro
    emits; a duplicate *name* is a duplicate type definition. A call site cannot reach a stream by
    writing a number — `Draws::of` takes a **type**. Reuse stays available (two sites of one
    conceptual draw *should* share a domain) but only by naming it.
  - **Salt values are explicit, not ordinals** — deriving them from list position would make
    duplicates structurally impossible but would **re-roll the planet whenever the list is
    sorted**. These numbers are baked into every world ever generated. The compiler checks
    distinctness instead.
  - **ALL 26 HAND-ROLLED SALTS CONVERTED**, from three files under three unrelated prefixes
    (`0x5700_*` `pregen/mod.rs`, `0x5900_*` `deeptime/grid.rs`, `0x5B00_*` `deeptime/biotic.rs` —
    three authors each inventing a prefix and hoping) into one list per crate. dc-worldgen
    (`src/draws.rs`, 20 domains): every call site in `pregen/{mod,tectonics,history}.rs`,
    `geology.rs`, `collapse.rs`, `fill.rs`; `interp_select_draw` → `draws::interp_corner_field`.
    dc-sim (`statistical/world.rs`, 6 domains): four call sites converted.
  - **EVERY WORLD IS BYTE-IDENTICAL ACROSS THE CONVERSION.** `Draws::bits` folds
    `(seed, salt, addr…)` through the same splitmix chain `mix(&[…])` did, so
    `Draws::of::<Plate>(seed).unit(&[p, 0])` **is** `draw_f64(&[seed, SALT_PLATE, p, 0])` — and the
    S7 / providers / contents goldens are the proof, unmoved.
  - **THREE HOLES, NAMED IN CODE RATHER THAN PAPERED.** (1) *Tag space inside a domain is still
    hand-laid* — `GeoSelect` tags 0–3, `GeoAccessory`'s `tag`/`tag + 1024`: hand-rolled
    sub-domains, same failure mode, smaller scale. (2) *Two `dc-sim/engine.rs` draws address
    `[seed, k, SALT, …]`* — the sample index before the domain, so converting them re-rolls every
    world's history layer; deliberate, world-changing, sequenced on its own rather than smuggled
    in here. The salt still has one spelling. (3) *A recorded salt is data*: `StrataEvent::sel_salt`
    replays a past selection, so `Draws::from_recorded_salt` exists with a name long enough to be
    friction. Plus `deeptime/`'s three domains are **registered** (so nothing can re-issue
    `0x5900_0001`) while their call sites still spell the constant locally, with
    `the_deeptime_constants_agree_with_their_registered_domains` holding the copy to the authority.
  - **THE CORRELATION, QUANTIFIED.** `pore_rider_share` sliced its 3-bit offset out of the very
    `fill_draw` `allocate_partial` consumes, under a comment claiming the two were disjoint. Over
    **464,521 real rider decisions** (production world, seed 1337, Medium, 48 chunk-columns at the
    strongest weathering cells): the pore offset was **100.00 % predictable** from bits 8–10 of the
    allocation's 20-bit offset — a *deterministic function*, zero conditional entropy. After:
    **no** 3-bit window of the allocation offset predicts it better than **12.57 %** (chance
    12.50 %), across all eighteen windows.
  - **AND THE COUPLING THE COMMENT FEARED MEASURED ZERO.** Residual-vs-residual `r = +0.0006`
    (retired) / `+0.0012` (decorrelated); mutual information 0.0609 bits against a **measured
    estimator floor of 0.0610**; the dither's entropy conditioned on `(band, allocation outcome)`
    **2.999 of 3.000 bits** both ways. Mechanism: the allocation's decision is a *contiguous
    interval* in its offset, bits 8–10 are a *fast sawtooth* across it (cycling every 2,048 of
    1,048,576), so they alias to uniform unless a band's fractional remainder is under 0.2 % of an
    eighth. **The comment was wrong about the mechanism and right about the outcome, for a reason
    it did not know.**
  - **THE DEFECT THAT WAS REAL IS A DIFFERENT ONE, and nobody had named it.** `mixed_at` drew `u`
    **once per voxel** and handed the same three bits to **every** band in it. A weathering front
    is many thin bands of one parent differing only in pore share (journal/0099 widened the merge
    key precisely to keep them separate), so a contact voxel routinely carries several rider
    decisions — **187,701 sibling pairs** in the sample. On a shared offset each is the same
    monotone step function of it, so they rounded in lockstep: **`r = +0.4878`**, and a multi-band
    voxel's total product carried **1.488× the second moment** independent roundings give. The
    errors *added* instead of cancelling — under an estimator whose entire justification
    (journal/0055, /0103) is that they cancel. After: `r = −0.0012`, ratio **0.999×**.
  - **IS THE BANDING VISIBLE? NO — measured, not asserted.** Banding is spatial structure, so the
    instrument measures spatial structure: one chunk-column's 32×32 contact plane, where all 1,024
    voxel columns share **one record and one fill plan**, so the only thing varying is the draw.
    Every autocorrelation at lags 1–4 in both axes is inside **±0.07** of zero, **before and
    after** (retired lag-1 `+0.0005, −0.0261`; decorrelated `−0.0099, +0.0113`); mean same-sign run
    along x **1.889 → 1.947** against 2.000 for no structure. *Structurally* absent, not merely
    subtle: both offsets are functions of a position hash, so a dependency between two decisions
    **at one voxel** cannot make structure **between** voxels. **"The fix was correct and the
    artifact was imperceptible"** is the honest result, and it retires the fullbright-walk next
    step Observed had filed.
  - **THE FIX, in the form that is hard to un-do.** (1) `SALT_GEO_PORE` — domain separation by
    salt, which stays disjoint whatever widths either offset grows into, where a bit-range
    carve-out is only disjoint for the widths it was written against. (2) The **event index** in
    the address — this is what fixes the defect that mattered. (3) `pore_rider_share` no longer
    takes an `f64` but a **`PoreDraw`**, field private to `fill.rs`, sole constructor `pore_draw`,
    sole mention of the salt: handing it the fill draw is now a **type error**. It also **moved
    from `collapse.rs` into `fill.rs`**, beside `allocate_partial` — the two quantizers of one
    voxel are checkable at a glance only on one screen. `fill_offset` / `FILL_OFFSET_BITS` exposed
    so nothing keeps a *copy* of the allocation's offset expression in order to reason about it.
  - **GATED THREE WAYS, and the gate carries its own control.**
    `no_window_of_the_fill_draw_predicts_the_pore_offset` and
    `two_bands_in_one_voxel_draw_independent_offsets` (fill.rs, 40,000 addresses),
    `the_pore_rider_share_is_unbiased_over_its_offset` (enumerated, exact), and in the new gated
    probe `the_pore_offset_is_no_longer_readable_out_of_the_fill_offset`,
    `sibling_riders_in_one_voxel_round_independently`,
    `the_decorrelation_moves_product_without_creating_it`. The probe reproduces the **retired**
    formula and asserts it still scores **100 %** on the same data — a before/after inside one run,
    so the control cannot silently stop being the control.
  - **WHAT MOVED: NOTHING IN ANY GOLDEN — and the reason is worth more than the fix.**
    `weather_inventory` is **off by default** (`grid.rs`: *"the production flip is the user's"*),
    and the weathering front is the **only** producer of a *loose* pore rider. So the world every
    golden hashes makes **no pore-rider decision at all**. **Measured, not inferred** (probe Part
    5, same seed, same chunk-columns, flag off vs on): **0 decisions** on the shipped default,
    **464,521** with `--weather-inventory`. This is corrections #51's shape a second time in two
    days — *the guard runs on a world nobody ships*. **Habit worth keeping: when a slice moves
    nothing, check whether the thing it moves exists in the default build before congratulating
    yourself on byte identity.**
  - **WHERE THE WORLD DOES MOVE** (behind `--weather-inventory`, which is where every weathering
    number in the corpus was measured): **32.0 % of rider decisions** (148,590 of 464,521),
    **45.5 % of rider voxels** (125,899 of 276,820) = **4.66 % of the 2,700,288 recorded voxels**
    sampled. **Net +332 eighths over 464,521 decisions (+0.0007 each)** — a redistribution, not a
    gain; `fill.rs` asserts the mean is `cnt·k8/8` by enumerating all eight offsets. **Visible
    character: none** — one eighth of product moving between parent and product *inside* a contact
    voxel, in a field that was white noise before and after.
  - Files: `crates/dc-sim/src/statistical/{rng.rs,world.rs,engine.rs}`,
    `crates/dc-worldgen/src/draws.rs` (new), `crates/dc-worldgen/src/{fill.rs,collapse.rs,
    geology.rs,lib.rs,pregen/{mod,tectonics,history}.rs,deeptime/biotic.rs (visibility only)}`,
    `crates/dc-worldgen/examples/pore_decorrelation_probe.rs` (new, `test = true`),
    `crates/dc-worldgen/Cargo.toml`, `docs/spines.md` (A-2).

- 2026-07-25 — **The per-cell ledger header collapses: ONE record for the grid, the cell as a CSR
  row** (journal/0102; background agent, worktree; **PURE LAYOUT CHANGE**). Discharges the OWED
  lever journal/0100 filed against itself. `DeepField::ledgers` was `Vec<FactLedger>` — a per-cell
  **owning container**, i.e. 48 B × 297,025 cells = **13.60 MiB paid before a single fact is
  stored**, in a field where **225,019 cells (75.8 %) carry no fact at all**. It is now one
  `LedgerField`: flat facts + journal/0100's sparse `(slot, start)` rows + a **dense
  `cell_row_start`** over those rows (the `flux.rs` shape, now affordable because every *cell*
  exists even though every *slot* does not — the outer index dense, the inner sparse, and the
  reason stated in code).
  - **MEASURED before/after, both real runs of `examples/flow_cost_probe.rs`** (seed 1337,
    `Extent::Medium`, production flags, same machine within the hour): **flag ON `DeepField`
    186.07 → 173.61 MiB**; **cost of turning the flag on +29.91 → +17.45 MiB (1.71× less)**;
    **the struct-overhead line 13.60 MiB → 0 B** — there is no per-cell struct. Per-cell index
    cost **48 B → 4 B (12×)**. Payload unchanged and exact at 15.77 MiB. *Reported honestly: the
    ledger's own heap went 16.31 → 17.45 MiB and its index fraction 3.4 % → 9.6 %, because the
    1.13 MiB of dense cell offsets moved INTO the heap from the 13.60 MiB that used to sit
    outside it as "structs".*
  - **The flag-OFF baseline is the same INTEGER in both runs — 163,748,661 B (156.16 MiB)** — the
    control that makes the ON comparison mean something.
  - **BYTE-IDENTICAL in the strongest form:** **1,033,189 facts across 72,006 non-empty slots,
    before and after** — the same million facts in the same slots. Green **by name**, unmoved:
    `the_production_world_still_hashes_to_the_pre_slice_goldens`,
    `identity_floor_off_flag_carries_no_ledgers_and_is_byte_identical`,
    `on_flag_is_purely_additive_record_and_surface_untouched`,
    `a_weathered_cell_carries_one_fact_per_agent_per_chapter_and_accumulates`,
    `production_scale_saprolite_band_reaches_at_least_one_voxel`, `one_fact_per_agent_per_firing`,
    `bedrock_facts_key_stably_as_the_record_grows`,
    `fact_order_within_a_slot_is_preserved_across_interleaved_slots`,
    `rekeying_moves_a_run_and_leaves_it_exact_sized`. New:
    `the_grid_record_reproduces_every_cell_fact_for_fact_and_in_order`,
    `a_cells_run_ends_at_its_own_boundary_not_the_next_cells` (absolute `start` across a **cell**
    boundary — the one place this could have been quietly wrong),
    `the_record_costs_its_facts_not_its_cells` (a residency **bound**, not a snapshot).
    283 tests, 35 suites, 0 failed.
  - **NOT ONE READER OF THE LEDGER CHANGED.** `ledger_at_voxel` returns a borrowed
    `LedgerView<'_>` with the read surface the owned struct had, so `collapse.rs`,
    `examples/s18_weathering_tour.rs`, `examples/weathering_profile_probe.rs` and
    `tests/s18_first_behavior_weathering.rs` all compile untouched (`get(i)` still returns an
    `Option` of something with the method; `iter()` still yields one item per cell). The rest is
    plumbing that *names the type* (`finalize_ledgers`' return, `DeepRun`/`DeepStepCtx`'s field,
    the re-export, `resident_bytes`). The only **test** edit in the tree is three lines in
    `bedrock_facts_key_stably_as_the_record_grows` that wrote `finalized[0]` — a `Vec` indexes, a
    record that hands out views does not. Same test, same name, same assertions.
    Files: `deeptime/{inventory,field,weather_inventory,mod,runner}.rs`, `examples/flow_cost_probe.rs`.
  - **THE PER-CELL CONTAINER IS STILL RIGHT AT GEN TIME, and that is the design.** `FactLedger`
    survives as the accumulator: the pass appends into one cell every epoch, and an insert into a
    grid-wide array would memmove every fact after that cell — up to a million, per firing. The
    split is by **clock**, not by structure. The defect was never "a per-cell owning container"; it
    was a per-cell owning container that is **resident**.
  - **GEN TIME: no measurable change** (flag-ON field 25.9 → 24.7 s, against a flag-OFF pregen that
    moved 20.2 → 22.3 s the *other* way on the same runs — noise between sibling agents).
  - **DEFECT FOUND AND FIXED EN ROUTE:** `examples/flow_cost_probe.rs` was broken **again**, the
    same way, **one day later** — `resident_bytes()` gained a `head` term when FLOW continuation
    (a) merged (journal/0098) and the itemisation had no row for it, so its agreement assertion
    panicked before printing a byte. CLAUDE.md's "re-run the probes by hand after any merge that
    changes what they measure" was already written *because of the flux-record instance* and did
    not fire. **The assertion caught it; the rule did not.** Row added.
  - **A-4 DISCHARGED BY PORTING** (spines.md § A-4 row extended): the shape was read out of
    `flux.rs` and applied one level up, not designed.
  - **OWED / next lever (filed, not done):** `DeepField::strata` is the same shape and bigger —
    `Vec<DeepStrata>` is **9.06 MiB of 32-byte structs** (5.8 % of the flag-off field) over an
    84.47 MiB heap, with **33,680 cells (11.3 %) holding an empty record**. See `ROADMAP.md` § Observed, *"the SAME lever, one record over"*.
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
  - **The mass verdict: NOISE** — see `ROADMAP.md` § Observed → DIAGNOSED (journal/0103), and corrections #50. The
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
  at the voxel tier the eighths are a *draw* (journal/0055's estimator doctrine) — see `ROADMAP.md` § Observed.
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
  product. **BUT accepted WITH a follow-up the walk found (see `ROADMAP.md` § Sequenced, "the
  weathering front needs a PROFILE"): the band has a HARD PERIMETER** — pure 8/8 product abutting pristine bedrock, because
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
  - **⚠ CORRECTED 2026-07-25 (corrections #51; staleness sweep row S-6).** *"Still a diggable seam
    on Medium"* was measured on the **warm reference** world `0x0D5EED572026`, **not** on the world
    the client boots. On seed **1337 / Medium** the recalibration produces **zero coal — 0 units
    across 297,025 cells**, the hottest peat candidate standing at **15.4 °C against a 22 °C
    onset**, i.e. 6.6 °C short. The cited guard no longer exists under that name: `tests/geotherm.rs`
    now carries `the_geotherm_rule_governs_coalification_on_the_production_world` (1337 / Medium,
    which **requires no coal**) and `coal_follows_the_warm_crust_on_the_warm_reference_world`
    (journal/0106). The recalibration itself stands; only the sentence about what a player would
    find was wrong. The content question — (a) accept a coal-free world … (d) — is open in
    `ROADMAP.md` § Observed, and the appearance walk this entry owed is a **desk null**, not a
    walk (see `ROADMAP.md` § Sequenced, APPEARANCE WALKS OWED item (3)).

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
  **⚠ APPEARANCE: unratified, world-wide — see `ROADMAP.md` § Observed.**

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
  `ROADMAP.md` § Observed. No user-visible change (no appearance, feel, or frame-rate
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
  +2.5 s; **resident +92.76 MB — see `ROADMAP.md` § Observed**.

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
  the resolved line in `ROADMAP.md` § Observed. Every climate-keyed read (veneer, soil tier,
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
  finding now in `ROADMAP.md` § Observed (corrections #22). Six assets `0038-*`.

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
  exhumed cores/forelands are illegible at ANY amplitude — see `ROADMAP.md`
  § Sequenced (erosion-supply calibration).

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
  `erodibility_max` (5×). **NEEDS RATIFICATION** (`ROADMAP.md` § Sequenced). Files:
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
  gating. **NEEDS RATIFICATION** (four calls, in `ROADMAP.md` § Sequenced). Files:
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
  **seed 1337** — see the item on the client's fixed seed in `ROADMAP.md` § Observed.

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
  24 m seam. **NEEDS RATIFICATION** (`ROADMAP.md` § Sequenced). Files: `biotic.rs`
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
  found mixed-face *appearance* changed (speckle loss — `ROADMAP.md` § Observed, Walk 14). Directional sun + hemispherical ambient only (PBR-2 owns shadows,
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


---

## Observed — archived

**Archived from `ROADMAP.md` § Observed on 2026-07-29**, by STATUS, not by age — the
same rule as § Shipped above. Sourced from the complete classification in
[`docs/audits/baseline-2026-07-28/S6-roadmap-observed.md`](docs/audits/baseline-2026-07-28/S6-roadmap-observed.md),
with every finding re-verified at source before the move.

**Entries are reproduced VERBATIM.** Recorded camera poses, world coordinates and asset
filenames travel with them (corrections #48 — *a prose landmark is not a pose*); nothing
was summarised away and nothing was deleted. Each entry keeps its **journal number**, which
is the stable pointer.

### Struck by the user, 2026-07-29 — eight field reports

The user reviewed the open field reports on 2026-07-29 and struck these eight. Their stated
general reason, which is the frame for all eight:

> *"plenty of these are going to be stale and were made in the context of old
> implementations and we didn't close the loop as the project moved on."*

These are **user field reports** and the audit trail matters, so they are struck here rather
than deleted. A struck report is **not** a resolved defect — it is an observation the user
has ruled no longer describes the world. Do not re-derive them.

**STRUCK (user, 2026-07-29)** — The (a)-(d) world-content call is **withdrawn**. Made against
an old implementation and the loop was never closed as the project moved on. *Its sibling entry
— "the only test defending 'a player can find and dig a coal seam' runs on a world no player
can open" — was NOT struck and stays live in `ROADMAP.md` § Observed.*

- **🔴 THE SHIPPED WORLD HAS ZERO COAL — and the guard that should have caught it runs on a
  different world** (measured 2026-07-25, `examples/coal_walk_tour.rs`; **corrections #51**).
  **USER CALL REQUIRED — this is a world-content question, not a bug to quietly fix.**
  - **The fact.** On the world `dc-client` boots (`BENCH_SEED = 1337`, `Extent::Medium`):
    **0 coal units across all 297,025 deep cells**, against **27,134 peat units in 14,596 cells**.
    The hottest coalification candidate is **15.4 °C** against `COAL_ONSET_C = 22 °C` — **6.6 °C
    short**. Trial-onset curve on this world is a cliff: `4 °C → 87 %`, `8 °C → 31 %`,
    `12 °C → 9 %`, **`16 °C+ → 0 %`**.
  - **The instrument is proven, so the zero is real.** The same census over `0x0D5EED572026 /
    Medium` — the world journal/0093's numbers came from — finds **1182 coal cells / 1834 runs**.
    It sees coal when coal exists.
  - ~~**Why the guard missed it (the root defect).** `tests/geotherm.rs::production_field()` builds
    **`seed 0x0B0A_57EE_0059, Extent::Small`** — *neither the production seed nor the production
    extent*.~~ **✅ FIXED 2026-07-25 (journal/0106)** — see the closing bullet.
  - **The geotherm's physics is NOT at fault.** On a world with coal it followed the warm crust
    exactly as claimed (coal cells mean gradient 41.9 °C/km vs peat-only 31.3; rift/arc ≥40 →
    13.4 % coal, craton <20 → 0 %). **Seed 1337 has no warm crust with peat on it.**
  - **OPTIONS (user's):** **(a)** accept a coal-free world for now — coal is a blessed placeholder
    and biology will recalibrate it anyway; **(b)** lower `COAL_ONSET_C` toward the cliff's live
    range (`≈8–12 °C` gives 31 %/9 % on this world) as an interim seat; **(c)** treat it as
    evidence that a single global onset temperature is the wrong shape and let the
    genesis-passes/property-driven arc subsume it; **(d)** change the shipped seed — **rejected by
    the integrator as backwards**, tuning the world to fit a constant.
  - **✅ DONE 2026-07-25 (journal/0106) — the half that was not a content question: `production_field()`
    is now the shipped world.** `tests/geotherm.rs::production_field()` builds **seed 1337 at
    `Extent::Medium`** (memoized per test binary), and the coal guard is **split in two**:
    - `the_geotherm_rule_governs_coalification_on_the_production_world` — on 1337/Medium. Asserts the
      `temperature` field is populated, that candidates exist (26 845 of them), and that **every
      candidate's coal state agrees unit-for-unit with `T ≥ COAL_ONSET_C`** — "coalification responds
      to the gradient field" in falsifiable form. It **requires no coal**, deliberately: the zero is
      an open content question (a)–(d) below, and a guard must not be a hostage to it. It reprints the
      onset sensitivity curve (`4 °C → 87 %` … `16 °C → 0 %`) every run, which is corrections #51
      lesson 3 made permanent.
    - `coal_follows_the_warm_crust_on_the_warm_reference_world` — on `warm_reference_field()`
      (`0x0D5EED572026`, Medium), **named for what it is**. Asserts coal exists, is not degenerate, and
      that coal units sit on **hotter crust** than the peat that stayed peat (measured 42.8 vs
      31.7 °C/km). A non-production fixture is fine; a non-production fixture called production is not.
    - `COAL_ONSET_C` untouched. **All three tests pass** (63 s; the Medium runs cost ~55 s more than the
      old Small ones — the price of the guard being about the shipped world).
    - Audit of siblings: `providers_common`/`rh_unification`'s `production_*` helpers name the same
      non-shipped world, but their claims (golden byte-identity, derived-vs-scalar agreement) are
      genuinely seed-independent, so they are **annotated, not re-seeded**; the `golden_*` rename ripples
      into `providers_golden.rs` + comments in `flux_record.rs`/`head_field.rs` and is left sequenced —
      **and as of 2026-07-25 it really was** (sweep row D-2 caught that the word "sequenced" was doing
      the work of an entry that did not exist — the same doctrine gap that hid row S-7 below).
      **✅ The rename SHIPPED 2026-07-26 — see `ROADMAP-history.md` § Shipped**; the helpers are `golden_field` /
      `golden_pregen` and the golden test is `the_golden_world_still_hashes_to_the_pre_slice_goldens`.
      `s18_first_behavior_weathering::production_scale_saprolite_band_reaches_at_least_one_voxel` is
      **honest** (1337/Medium) and is the shape to copy. `deeptime::production_config` /
      `water::coarse::production()` name a *config*, not a world — legitimate.

**STRUCK (user, 2026-07-29)** — Stale. Its own premise was already half-falsified (journal/0055
falsified *"worldgen does not yet emit sub-8 loose voxels"* world-wide, `ROADMAP.md` § Observed
"HOLES IN THE GROUND"), and the report as written describes an implementation the project has
moved past.

- **Loose materials do not exist in the world yet** (user, 2026-07-20:
  "needed — even if they don't fall with gravity yet"). The RENDERER is
  already waiting: partial-height loose rendering shipped built-but-dormant
  in 3c-2 (journal/0010) because loose deposition never emits sub-8
  columns. Missing half is content/simulation. User direction for when it
  lands: loose materials should **spread on being dropped** (granularity +
  fall height → partials displaced into surrounding empties) — angle of
  repose from the partials model rather than a physics solver. Sketch in
  ideas.md.

**STRUCK (user, 2026-07-29)** — Stale. Already annotated un-walkable on the shipped world (zero
coal, corrections #51) and now withdrawn outright. The **lighting/tonemap** question it raised
— no floor under the dark end of the lit path — is not carried by this entry; it belongs to
PBR-2's shadow work, which owns it.

- **Walk report (2026-07-20, journal/0027): coal renders as pure black in the
  lit pass — a hole in the screen, not a rock.**
  **⚠ STILL A VALID LIGHTING/TONEMAP QUESTION, BUT UN-WALKABLE ON THE SHIPPED WORLD**
  (2026-07-25, sweep row A-2): there is **no coal to photograph** — 0 units across 297,025 cells
  on seed 1337 / Medium (corrections #51). The 2026-07-20 frames were shot on a world the client
  can still open only because coal existed then; today a re-shoot would find nothing. **Do not
  launch for it.** The defect is about the dark end of the lit path, not about coal, so it can be
  re-photographed on any sufficiently dark material — or it waits on the coal-content call
  (a)–(d). Photographed at world voxel
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

**STRUCK (user, 2026-07-29)** — Stale. A user sketch (*"not the only possible answer, just a
thought"*) filed against the 2026-07-22 render shape; withdrawn.

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

**STRUCK (user, 2026-07-29)** — Stale. Filed 2026-07-20 against the then-current accessory
roster; withdrawn.

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

**STRUCK (user, 2026-07-29)** — Stale. **Its two YES answers are load-bearing and are preserved
here**: *no sky-holes* at two partial-rich stations (journal/0057 confirmed by eye) and *no
28.8 m chunk patches* (journal/0058 confirmed) — this entry is what discharges the two
**UNWALKED** tags still carried live in `ROADMAP.md` § Observed, which now cross-reference it
here. Assets `0059-*`.

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

**STRUCK (user, 2026-07-29)** — Stale. Coordinates preserved below (deep cell (488, 278), world
metres (106 938, 9 953)) and the falsified wind-banding hypothesis with them.

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

**STRUCK (user, 2026-07-29)** — **The user's own reasoning, and it is the stronger kill:**
*grass and dirt are not generated in the current shape of the default plugin pack, so the
observation itself is stale.* Not merely that the named mechanism retired — the thing observed
is not produced by the world any more.
  **⚠ The S6 audit's recommended re-shoot of `0024-fb-ne.png`'s framing is CANCELLED** by this
  ruling. Do not re-derive it: there is nothing at that vantage to re-photograph. Assets
  `0024-after-ne.png` / `0024-fb-ne.png` kept for the record.

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

### Resolved in place — 37 entries whose own bodies already said DONE

The S6 baseline sweep found **37 of 129** § Observed entries (29 %) already declaring
themselves ✅ DONE / RESOLVED / FIXED in their own text, kept live "for the record". They are
moved here whole. This is the archive-by-status rule applied mechanically: **no judgement
call, no information loss.** Where an archived entry carries a *measurement*, that
measurement is a **dated record** — it was not re-measured by this pass and must not be
propagated without checking (§ Read first item 5, immutable body / mutable header).

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- ~~**OWED (small) — `dc:deep/climate`'s lagged terrain read is undeclared**~~ **✅ DONE
  2026-07-25 (journal/0107), in the head-declaration slice as predicted — *"worth doing the
  next time that file is open"*.** Declared **`reads_prev: [Forced]`**, and the three
  cfg-selected slices this entry expected turned out to be **the wrong shape**: lagging against
  the *last* terrain revision (`Settled`/`Compensated`/`Diffused`) pins strictly **less** than
  lagging against the **first**, because it would leave `climate` free to slide past `forcing`
  and `transport`. One token, one slice, no cfg selection, and the whole chain covered
  transitively. Schedule-neutral, proven by
  `the_terrain_revision_declarations_are_schedule_neutral`. *(Original entry below, for the
  record.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **OWED (small, as diagnosed) — `dc:deep/climate`'s lagged terrain read is undeclared** (found by the
  journal/0104 `reads_prev` audit, 2026-07-25; deliberately not changed by that slice, whose
  mandate was the mechanism, not the declarations). `climate` declares `reads_prev: &[]` while
  its own comment (`runner.rs`) says it reads the **start-of-epoch topography**. That is a real
  lagged read of the terrain, undeclared. **It is not dangerous**, and that is the whole
  distinction the anti-dependency edge exists to draw: the ordering it needs is already pinned by
  a **true forward edge** — `dc:deep/forcing` reads `Climate`, and every terrain writer in the
  roster is downstream of `forcing` — so climate is provably ahead of all of them by the graph,
  not by the tie-break. Declaring it is **free and provably cannot move the schedule** (the edge
  is already implied by the existing transitive closure), but it needs three cfg-selected slices
  for the three terrain-revision rosters (`Settled` / `Compensated` / `Diffused`). Worth doing
  the next time that file is open; not worth a slice of its own.

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

S6 finding **F4**, verified 2026-07-29: both halves carry ✅ inline (option (a) shipped
journal/0104; head's under-declaration shipped journal/0107) and only the kept-for-the-record
diagnoses remained.

- **⚠ A TIE-BREAK IS DECIDING PHYSICS AGAIN — `reads_prev` is documentation, not a
  mechanism** (spine-audit 2026-07-25; **the SECOND instance in two sweeps**, and the
  auditor's own words: *"this is the only fix that stops a third"*). **USER RATIFICATION
  REQUIRED — not an agent's call.**
  - ~~**The hole.**~~ **✅ DONE 2026-07-25 — option (a) shipped, journal/0104.** `passgraph`
    now takes a second edge kind: a `reads_prev` declaration is a reader-before-writer
    **anti-dependency**, so a lagged read is *ordered*, not annotated. **Production order
    byte-unmoved** (all six new edges pointed from a pass already ahead of its target — the
    physics was right and merely unenforced), goldens green by name, and the guarantee is
    `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename`. Full detail in the
    Shipped entry. *(The paragraph below is the original diagnosis, kept for the record.)*
  - **The hole (as diagnosed).** `DeepPass::reads_prev` appears in `runner.rs` and **in no
    other file in `crates/`** — `passgraph` never receives it (`runner.rs:199-204`,
    deliberately). So a
    `reads_prev` declaration is **enforced by nothing**. `dc:field/head` declares
    `reads_prev:[Recorded]` and that is *true today* only because Kahn parks a ready node
    until it wins the id sort (`passgraph.rs:151-153`): head is ready once `drainage` writes
    `Routed`, while `deposition` waits on `isostasy`. **Rename the pass to any id sorting
    after `dc:deep/deposition` — `dc:deep/hydraulic_head`, say — and it silently starts
    reading THIS epoch's record.**
  - **Why this is the same defect one level up.** The last sweep caught a tie-break deciding
    a *read* (`weather_inventory`'s `BioMod`, fixed by declaring it). This one has a
    tie-break deciding **an epoch**, and it **cannot** be fixed the same way, because there
    is no declaration channel that binds. The honest fix is a **mechanism** — a
    reader-before-writer **anti-dependency edge** in `passgraph`, so a lagged read is
    ordered, not merely annotated.
  - ~~**⏳ STILL OPEN — second, smaller, and fixable today: `dc:field/head` UNDER-DECLARES.**~~
    **✅ DONE 2026-07-25 — journal/0107, user-ratified with the consequence attached.** The
    revision is **`Forced`** in every cfg path, and declaring it turned out to be **only half
    the fix**: a `reads` edge on a revision token pins you after its *producer* and says nothing
    about the pass that overwrites the same plane next (a different token = a different
    resource). So `dc:deep/transport`'s undeclared terrain mutation got its name
    (`DeepAxis::Incised`) and `head` declares the **pair** — `reads: [Routed, Forced]` +
    `reads_prev: [Recorded, Incised]`. **The auditor's *"declaring `Forced` is free, and it
    pins it"* was half wrong, and the *"verify that claim before trusting it"* below is what
    caught it.** Flux record byte-identical to journal/0098 on every figure; pass order
    unmoved; `dc:deep/climate`'s lag declared in the same slice. Full detail in
    `ROADMAP-history.md` § Shipped, journal/0107. *(The paragraph below is the original diagnosis, kept for the record.)*
  - **The hole (as diagnosed).** `reads:[Routed]`
    (`runner.rs:605`) does not cover the ground surface `R+H`, which the body builds via
    `grid.surf_at` (`runner.rs:428-430`) and the solve uses as its **seepage cap, lake datum
    and free-surface boundary** (`head.rs:434-467`). Which terrain revision it sees is again
    the id tie-break. The golden-order test's defence — *"its position never affects the
    terrain"* — is true and **is not the question**: it affects **the field's own values**.
    The auditor judges declaring `Forced` *"free, and it pins it"*. **Verify that claim
    before trusting it** — head is **ON by default** (`grid.rs:279`), so if the declaration
    moves which revision it reads, the **recorded flux changes** and that is a world change
    needing a walk. Acceptance: declare it, prove the flux record byte-identical; **if it
    moves, STOP.**

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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
  **✅ DONE 2026-07-25 (journal/0102): struct overhead 13.60 MiB → 0, per-cell index 48 B → 4 B,
  flag-ON `DeepField` 186.07 → 173.61 MiB, flag-OFF identical to the byte, 1,033,189 facts across
  72,006 slots before and after, and no reader changed (`ledger_at_voxel` returns a borrowed
  `LedgerView<'_>`).**
  **⚠ MY GENERALISATION ABOVE WAS TOO BROAD — corrected by the build.** The defect is **not**
  "a per-cell owning container"; it is **one that is RESIDENT**. `FactLedger` **survives as the
  gen-time accumulator**, and must: a grid-wide insert would memmove every fact after the cell,
  every epoch. **The split is by CLOCK, not by shape** — grow per-cell while compiling (gen time
  is free), compact to one grid-wide record for residency (runtime is sacred). Read the rule that
  way, or it forbids the very structure the compile needs. *(Related expiry, also from the build:
  journal/0100's carve-out — "dense offsets are unaffordable for the ledger" — **expires at the
  CELL level**, because every cell exists even though every slot does not. The record now carries
  **both** compressions, each where its premise holds: dense cell offsets + keyed slot rows.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- ✅ **DONE 2026-07-25 — shipped, see `ROADMAP-history.md` § Shipped (journal/0102).** *a per-cell OWNING CONTAINER is a
  header × 297,025 before it stores anything* (filed by journal/0100 against itself). Measured
  result: the struct-overhead line **13.60 MiB → 0 B**, per-cell index cost **48 B → 4 B (12×)**,
  flag-ON `DeepField` **186.07 → 173.61 MiB**, cost of the flag **+29.91 → +17.45 MiB**; world
  byte-identical (1,033,189 facts in 72,006 slots, before and after; flag-OFF baseline the same
  integer, 163,748,661 B). **The generalisation survives the slice and is the valuable part:**
  *any per-cell owning container in a 297 k-cell field costs a header per cell before it holds
  data* — so the default for anything per-cell is **one grid-wide record with CSR rows**, never
  `Vec<Something>` per cell. Refined by the slice: the defect is a per-cell owning container that
  is **RESIDENT**; per-cell is the right *gen-time* shape (a grid-wide insert would memmove every
  fact after the cell, every epoch), so compact at the seam where the compile ends — which already
  exists in this codebase and is called `finalize_*`.

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **✅ DIAGNOSED 2026-07-25 (journal/0103) — the front's voxel-tier mass error is NOISE, and the
  `+3.7 %` was the INSTRUMENT** (opened by the integrator's review of journal/0099; corrections
  #50). Re-run over **247 columns** (journal/0099 had 21), with the pipeline split into its **two
  quantizers** — the fill geometry and the eighth draw are not the same kind of error and only the
  second is an estimator — and with the census taught to exclude voxels it cannot attribute.
  - **INTEGRATOR'S HYPOTHESIS WAS WRONG, and it is worth recording which half.** I challenged
    journal/0099's *"unbiased estimator"* claim on the grounds that a `+3.7 %` mean and a `+16 %`
    median were **the signature of a floor effect** (a band thinner than one eighth cannot express
    as less than one eighth without vanishing). **The challenge was right and the hypothesis was
    wrong.** The stratification really does trace a textbook floor-effect curve — but the cause is
    **measurement contamination, not quantization**: the product is `CLASS_CLASTIC_FINE` and so is
    much of the pile directly above it, **222 of 247 columns** have a front voxel whose plan also
    holds a *non-front* event of the same material, and **voxel contents carry no provenance**, so
    the naive census credited overlying mudstone to the front. The decay with magnitude was *the
    contact voxel's share of the front shrinking as the front grows.* **journal/0055's doctrine is
    CONFIRMED, not falsified** — `allocate_to` is Cranley–Patterson systematic sampling with
    `P(extra) = remainder` exactly, and the one real floor (`clamp(1,7)`) cannot bite on a
    `[7,5,4,3,2,1,1,1]` profile. corrections #50 falsifies **the number, not the doctrine**.
  - **Second time today that asking the discriminating question mattered more than the hypothesis
    attached to it** (corrections #49 was the first — *"was it air, or `has_contents:false` read as
    air?"* offered two wrong answers and the truth was a third thing). Both times the *question*
    forced the measurement that produced the real answer. **Demand the measurement; hold the
    explanation loosely.**
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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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
  correlation is **visible** as a pattern at a contact.
  - **RESOLVED 2026-07-25 — decorrelated, and the visibility question answered NO** (journal/0105;
    user-ratified *"this needs fixed either way. Decorrelate."*). Quantified before the fix: the
    pore offset was not merely correlated with the allocation's offset, it was a **deterministic
    function** of it — 100.00 % predictable from bits 8–10 of the 20-bit offset, over 464,521 real
    decisions. But the coupling the comment feared measured **zero**: `r = +0.0006` between the two
    roundings' residuals, mutual information on the estimator floor, and the dither's entropy given
    everything the allocation decided still **2.999 of 3.000 bits** — bits 8–10 are a fast sawtooth
    across the allocation's contiguous decision interval, so they alias to uniform. **No banding,
    measured spatially**: on a 32×32 contact plane sharing one record and one fill plan, every
    autocorrelation at lags 1–4 is inside ±0.07 before *and* after. **The defect that was real was
    a different one**: `u` was drawn once per voxel and served **every** band in it, so a
    multi-band front voxel's riders rounded in lockstep (`r = +0.4878` over 187,701 sibling pairs)
    and their errors **added** — 1.488× the second moment independent roundings give. Fixed by
    `SALT_GEO_PORE` + the event index + a `PoreDraw` newtype the fill draw cannot be passed to.
    See `ROADMAP-history.md` § Shipped, journal/0105.

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

⚠ **Dated denominators inside — do not propagate.** This entry's *"today's total is 108.55
MiB"* and its `S19` note are 2026-07-25 figures taken **before** journal/0102's ledger collapse
moved `DeepField` residency again the same afternoon. The archived text is the dated record;
the numbers were **not** re-measured by this archive pass. Its closing claim that the
`FactLedger` `Vec<Vec<Fact>>` sibling defect *"is NOT fixed by this and remains"* was
**superseded hours later** by journal/0102 (archived above).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**Deep-cell-square surface-material frontiers checker the far field:
  RESOLVED** 2026-07-22, journal/0073 — the B1 shape-teacher. `surface_class`
  now dithers class membership from the top-window metre shares (S-4 move B), so
  the 460 m class frontier is a statistical gradient, not a razor-straight
  square. The record stays a non-interpolable unit list read NEAREST — the fix
  routed *around* interpolation exactly as spines S-4 flagged, by dithering
  membership rather than bilinear-ing the record. Re-shot at the `0070-*`
  vantages as `0073-*`; verdict in the entry.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**"The world systematically under-expresses `H`": RETRACTED** 2026-07-21,
  same day it was filed — corrections #28, journal/0058. The alarm was the
  integrator's metres/voxels inversion, not a defect. Verified headlessly:
  the generated column matches `round(H/0.9)` with an error of **0 or +1
  everywhere, never negative**, the +1 being the top-of-column partial voxel
  that a *block* scan must count whole. The client world is also byte-identical
  to the probe world, proven block-for-block at four addresses — the
  stop-the-world hypothesis is dead.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **~~Caves ↔ hydrology integration thread captured~~ — SUBSUMED 2026-07-25 by the FLOW arc**
  (sweep row B-1). All four of its work-shaped findings now have owners, and this entry is kept
  only as the pointer:
  - **two drainage opinions** → `flow.md` § 6 retires `pregen/hydrology.rs` **outright** —
    *"wrong resolution, wrong time, wrong topology"* — so there is no subsumption to design;
  - **"deep drainage is computed and discarded"**, wanting a per-chapter table recorder axis for
    erosional caves → **that is `DeepField::flux`, and it shipped** (journal/0096: 2,590,372
    entries, per chapter, 40.66 MiB, pinned by
    `per_chapter_history_is_retained_not_just_the_final_epoch`);
  - the **bounded-drainage-refinement spike question** that gated RiverSeg retirement → **S14 is
    superseded as posed** (flow.md § 9.6; water.md's own banner), and RiverSeg retirement is FLOW
    **continuation (e)**;
  - the **column-as-interval-log target contract** → flow.md § 7 promotes it from *proposed* to
    **necessary** (voids and conduits are intervals, not a heightfield).
  **Cave-specific residue rides FLOW continuation (c)** (the free/bound edge, void intervals, and
  the conduit pairing rule). *(Original capture, for the reasoning: full text in water.md § Session
  capture 2026-07-21 — nothing was decided there. Same session recorded the user's
  water-rendering directive (partials/structure, placeholder texture, data
  seams for flow/waves) in water.md, which is NOT subsumed and still stands.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **FIXED 2026-07-21 (journal/0051): eviction landed, the march is flat
  (+27.4 → 0.00 MB/jump). See `ROADMAP-history.md` § Shipped. Two numbers from this diagnosis were
  corrected on the way: a chunk is 64 KB, not ~33 KB (`Block` is `repr(u16)`),
  so the fill rate is ~335 chunks/jump, not ~750.** Original diagnosis below.

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**Sub-km relief / roughness decay: MEASURED** 2026-07-21, S13 + journal/0041
  — see `ROADMAP-history.md` § Shipped. Decay confirmed (5 % survives), bilinear falsified (#24), and
  the walk's own sampling corrected (#25). The remaining OPEN part is which
  recalibration to take — § Sequenced **above**, awaiting a user picture-pick.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**`climate_at` half-cell offset: CONFIRMED BUG, FIXED** 2026-07-21,
  journal/0043 — climate sat exactly 7 372.8 m north-east of the terrain it
  tinted, live at every preset (all odd `w`). One-expression fix,
  regression-guarded at Small+Medium; 7 surface columns flip Stone→Dirt in the
  22–33° band (the rock line was 7.4 km off). Heights are climate-independent,
  so all 0040/S13 elevation numbers stand. No prior conclusion falsified —
  the 0038 desert-null mechanism (corrections #22) is registration-independent.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**The far field cuts off at 1.2 km**: FIXED 2026-07-21, journal/0042 —
  `--horizon <km>`. Original report: user, 2026-07-21, "the cutoff is still too
  near, can't see macro shape of landscape".)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**Material placement rules are climate mocks: DECIDED 2026-07-21** — see
  geology.md § Expression of the ledger, ecology.md § DECIDED 2026-07-21, and
  ARCHITECTURE.md § Modularity and performance. Runtime gen is refinement over
  the ledger; everything recorded must be expressed; unexpressed only where the
  expresser is unbuilt, loudly temporary. The veneer rule is a placeholder —
  DO NOT BANDAID. Remaining OPEN engineering, now Sequenced: consume `exhum`/
  `t_crust`, carry the discarded `H` regolith plane, derive form from
  provenance, and the per-voxel provenance query. Original observation:)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**Console v1 field report: FIXED same day** — console v2 shipped, see
  `ROADMAP-history.md` § Shipped / journal 0035. Original report:)* **"still unusable" (user,
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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**INSTRUMENT: "lit before/after is invalid because the sun moves" —
  RETRACTED same day**, corrections #18/#19. The sun is FIXED (S4: constant
  0.35 time-of-day); no day/night cycle exists, which is exactly why the
  sim-light design had to invent heavenly-body paths. The lit pass was the
  trustworthy register all along. The real defect was the opposite one, and
  is recorded below.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **Walk 12's "blocking regression" was a misdiagnosis** (corrections #10,
  journal/0016): a pose in meters cross-checked against block queries in
  voxels. `true_surface_m` was already deep-time-aware (its ceiling reads
  `ColumnRec`). The investigation still paid: `eye_in_solid` was answering
  from the legacy S1 world on any ChunkMap miss, and a failed surface scan
  silently returned `analytic − 220 m`. Both fixed (merge `81a87b8`;
  `Option`-typed misses, authoritative solidity).

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- **Instrument fix: pose replies echo the voxel coordinate — DONE**
  (journal/0021, instrument batch, awaiting integration). `client_player_pose_
  {get,set}` and `dc:character/pose` now carry `pos_voxel` (feet, active-scale
  voxels via the authority's own `scale.voxel_at`) beside the meters `pos` —
  the walker's two languages both labelled, no mental unit conversion (the
  corrections #10 misread that cost a full agent cycle).

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- Walk 13 (journal/0018): *(**`surface_snapped` absent on the `surface:true`
  success path: RESOLVED** — journal/0021, instrument batch. The flag is now
  ALWAYS present in a `surface:true` reply: true when the scan seated the feet,
  false on a miss (position left as requested, `surface_error` string). Both
  paths leave through one `surface_teleport_reply` helper, so the flag can't be
  set on only one branch again.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- Walk 5 (journal/0005, character surface): characters have **no auto
  step-up** — a one-voxel rise halts a grounded walker until it jumps
  *(DECIDED 2026-07-19: stays jump-required; mover-feature vs
  controller-skill is the NPC-intelligence design's question)*.
  *(Disconnect policy: DECIDED 2026-07-19 — freeze for v1, NPC-tier
  degradation as a later controller binding; API.md § Characters,
  bodies.md formerly-open Q3. Attach placement guard: fixed,
  journal/0006.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**~2/3 of far-mesh triangles were sealed cave surfaces (S3): RESOLVED for
  the worldgen far field** — journal/0022. The worldgen horizon is a **top-surface
  heightfield** (2048 tris/tile, no interiors), not a volumetric shell. The S1 far
  mesh on keys 3/4 is unchanged / still volumetric.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**`client_player_pose_set` outside the one door: RATIFIED same day** —
  API.md Decisions log #5. Retirement = player controller through
  controller-verb commands; sequenced after the session-4 agents land,
  since it rewrites files they are touching. See Sequenced.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

- *(**Console follow-ups: RATIFIED same day** — API.md Decisions log #6
  (`completions` hook) and #7 (registry macro/derive). Dispatched as a
  session-4 background agent — see In flight.)*

**ARCHIVED 2026-07-29 — resolved in place** (the entry's own body already declared it closed;
moved by status, not by age).

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

### Resolved 2026-07-29 (journal/0126) — the shipped-artifact tripwire sweep, both halves

Archived on the day it resolved, by STATUS: the enumeration ran and the answer is recorded
here rather than left on the live board declaring itself ✅ DONE — which is the exact
condition 37 of this section's other entries were archived for.

**Two entries, filed hours apart by two agents from opposite sides of the collapse tier, and
they closed in one commit** — the record-side question journal/0124 filed, and the
expression-side 🔴 the member-#0 far slice (journal/0125) found. Both are reproduced below,
both resolved. Their common sentence: *what is not fingerprinted cannot have an authorized
move.*

- **✅ RESOLVED 2026-07-29 (journal/0126) — WHICH SHIPPED ARTIFACTS HAVE NO GOLDEN? The
  enumeration ran, and every shipped artifact now carries a tripwire.** Filed by
  journal/0124, which added `GOLDEN_FLUX` after the `Schedule` slice moved the **flow
  record** and a full 834-test gate did not notice — the record is a pure sidecar to both
  existing goldens (`GOLDEN_SURFACE` = terrain, `GOLDEN_RECORD` = strata), so neither could
  ever see it, and the move had to be measured with a throwaway harness run twice across a
  `git stash`.
  - **The generalisation this was filed for:** *an artifact the ritual ships with no tripwire
    on it cannot have an **authorized** move, because nobody can see it move.* The whole
    golden discipline rests on telling an explained move from an unexplained one, and that
    distinction is unavailable for anything ungoverned.
  - **The result — 16 `DeepField` members: 12 covered, 3 newly goldened, 1 not shipped.**
    `tests/artifact_tripwires.rs` — one shared fixture build behind a `OnceLock`, five tests,
    **+6.45 s** of gate wall-clock measured.
    - **COVERED (12).** `w`/`wp`/`cell_m` (the shape header), `surf`/`regolith`
      (`GOLDEN_SURFACE`), `strata` (`GOLDEN_RECORD`), `recv`/`area`/`lake` and
      `exhum`/`t_crust` (all inside `surface_fingerprint`), `flux` (`GOLDEN_FLUX`, on
      `flux_record.rs`'s own fixture).
    - **Newly goldened (3), one constant each so a move NAMES its artifact:**
      `GOLDEN_GEOTHERM`, `GOLDEN_HEAD`, `GOLDEN_CHAPTERS`. One blob hash would have said
      *"something moved"*, which is the throwaway-harness bisection journal/0124 had to do
      by hand.
    - **`chapters` is the one the enumeration nearly missed, and the lesson is portable.**
      `surface_fingerprint` hashes `chapters.len()`, which *reads* as coverage and is not:
      the chapter count is a config constant, so that byte pins the config and says nothing
      about plate positions, velocities or continentality. **A hashed length is not a hashed
      artifact** — asserted, not asserted-in-prose, by
      `the_chapter_length_byte_is_blind_to_what_the_chapter_table_says`. This is the same
      trap as `has_contents` answering per-CHUNK (corrections #49): a real number, about the
      wrong thing.
    - **The candidate list this entry shipped with was wrong about `exhum`/`t_crust`** —
      both **are** inside `surface_fingerprint`, and have been since before the flow record
      existed. Recorded because the list was written from the `spines.md` § 3 rows (exported,
      unread) and *unread* was silently read as *unguarded*; the two are independent axes,
      and this slice is the evidence.
    - **`ledgers` is NOT SHIPPED and is deliberately not goldened**: `weather_inventory` is
      off in every world a player gets, so the artifact is empty. Its emptiness is the
      tripwire, asserted in the enumeration test — **and the commit that flips the flag on
      owes a `GOLDEN_LEDGER` in the same diff**, because from that moment it is a shipped
      artifact with none. (Live residue, on the board.)
    - **`recv`/`area`/`lake` are the corpus's only DERIVED case** (`recv` is documented as
      the argmax of the MFD partition, i.e. a summary of `flux`). They are hashed by
      `surface_fingerprint` *and* their authority by `GOLDEN_FLUX` — a pre-existing
      double-pin, not one this slice introduced, and it survives only until FLOW
      continuation (e) deletes them.
  - **The enumeration is enforced by the compiler, not by a comment.** The test destructures
    `DeepField` exhaustively, so adding a member **stops it compiling** and the author must
    classify it. CLAUDE.md's read-first item on the stale ecology count observes that a wrong
    count in a justification is *"exactly what an enumeration-completeness check would catch,
    and we still have none"* — this is one, for the artifact inventory.
  - **Residue, not owed to this slice** (kept live in `ROADMAP.md` § Observed): `Pregen::grid`
    and `Pregen::pipeline` were not walked.

- **✅ RESOLVED 2026-07-29 (journal/0126) — 🔴 NO GOLDEN HASHES THE FAR FIELD — found
  2026-07-29 by an acceptance criterion that could not fire (journal/0125).** Member #0's
  slice brief said *"goldens move — re-capture with the why"*, which was the right instinct:
  it changed the far surface class draw twice over (nearest-cell → membership-dither, and a
  canonical class order change). **Not one hash in the workspace moved**, and the run
  confirmed it: `generated_world_is_byte_identical_to_the_pre_contract_goldens` passed
  untouched.
  - **The reason is structural.** `contents_contract`'s `world_fingerprint` hashes
    `generate_chunk_with_materials` only, and `generate_chunk` has not consulted
    `surface_class` since journal/0074. `providers_golden` / `rate_axis` / `creep_operator`
    hash the deep-time surface planes and strata record, upstream of the collapse tier.
    **`coarse_surface` — every metre of ground beyond the loaded radius — is fingerprinted
    by nothing.**
  - **Why it stayed invisible:** the far field has *behavioural* tests (a class-split
    floor, the near/far statistical agreement test, the far-tile mesh budget) and a
    behavioural test cannot notice that no fingerprint exists. The slice's own gate was
    green on the goldens **and** would have been green had it broken the far field
    outright.
  - **✅ CLOSED by `GOLDEN_FAR_SURFACE`** (`tests/artifact_tripwires.rs`, journal/0126) —
    36,864 columns of `(height, block)` from `WorldGenerator::coarse_surface` on a **509**-
    voxel stride, 192 per axis, ±48,864 voxels: the whole civilized extent plus ~7 km of
    border wilds, where the pyramid runs forever and the record does not. The stride is
    **prime on purpose** — the coarse cell (16,384) and the chunk (32) are both powers of
    two, so a power-of-two stride would put every sample on one phase of both lattices and
    the golden would be blind to exactly the seam artifacts this half of the world is prone
    to. It shares the record side's `Pregen`, so it costs **no extra world build**.
  - **⚠ AND THE COVERAGE IT BUYS IS PARTIAL, MEASURED, AND WRITTEN ON THE CONSTANT.** The
    `providers_common` fixture turns out to be an almost entirely submarine world: over the
    36,864 sampled columns the height runs **−2,642 … −3** and only **160 (0.43 %)** front
    with anything but the ocean block; the highest ground in a ±56,000-voxel scan is **+7
    voxels**. So the **height** field is exercised completely — it is the far field's
    dominant output — while the **surface-class draw, the thing journal/0125 actually
    changed, is exercised by 160 columns.** It would still trip; it is not the instrument a
    land-bearing world would give.
  - **Residue (live on the board): a far-field golden on a fixture with real continent.**
    Its home is `contents_contract.rs`, whose Medium seeds are the worlds with land — which
    is where this entry's own heir spec put it before the tripwire suite took it.
    *The first draft of the sample (stride 2039 × 48) caught **six** non-ocean columns and
    would have shipped looking identical. The difference between the two drafts is one
    printed histogram — which is the same lesson as the parent entry, one level down.*

## In flight · Sequenced · Observed — archived 2026-07-29 (pass 2)

**Archived from `ROADMAP.md` on 2026-07-29, by STATUS, not by age** — the same rule
as the sections above. This is the **second** pass of that day: the first moved 45
§ Observed entries and four superseded close blocks; this one reads § In flight and
§ Sequenced, **which had never been swept for archivability at all.**

Sourced from the complete classification in
[`docs/audits/2026-07-29-roadmap-classification.md`](docs/audits/2026-07-29-roadmap-classification.md), with every
finding re-verified at source before the move. *An audit finding is a hypothesis, not
an authority.*

**Entries are reproduced VERBATIM.** Recorded poses, world coordinates, measured
numbers and asset filenames travel with them; nothing was summarised away and nothing
was deleted. **Where an archived entry carries a measurement, that measurement is a
dated record** — it was not re-measured by this pass and must not be propagated
without checking (read-first item 5: immutable body, mutable header).

**Nothing user-owned moved.** Five strike-candidates — two 2026-07-21 appearance
ratifications, two field reports resting on a premise the user struck this morning,
and the ores fork — stayed on the live board awaiting the user's word. They are
listed in § 5 of the classification audit.

**Why this pass was worth its cost, stated as the finding rather than the tidy:**
§ Observed's measured failure was *closure-in-the-wrong-place* — a claim and its own
refutation coexisting in one artifact. § Sequenced and § In flight have the same
shape and it is worse, because the two halves sit in **different sections**: the
collapse-cache `evict()` claim and its refutation stood ~2,500 lines apart for eight
days; the erosion-supply calibration asked for a comparison another entry 1,300 lines
above it had already made; the GPU-pooling question and *"this is the numbered
answer"* never named each other. **The rule is unchanged and it is not about volume:**
discharge the old entry in the same commit as the new fact.

### Resolved in place — entries whose own bodies already said DONE

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

**Self-labelled *"(historical entry)"* in its own header.** `docs/design/tectonics.md` shipped 2026-07-20 and now carries a supersession banner of its own (added 2026-07-29, baseline sweep S2/F2).

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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

A shipped ledger for journals 0023-0030. Every entry it names is in `ROADMAP-history.md` § Shipped with its journal number as the stable pointer.

**Session 3 shipped, all gates green on merged main:** FF2a voxel far field
(0023) · far-seam uniform-push fix (0024) · S10 biotic layer (0025) ·
organic materials + biotic flip (0026) · organics photo walk (0027) ·
S11 water locality + body graph (0028) · erodibility coupling (0029) ·
erodibility production flip + walk (0030). Eight journal entries; seven
corrections filed (#11–#19, two of them the assistant's own).

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

A shipped ledger for journals 0039-0049. Same disposition as the session-3 ledger above.

**Session 5 shipped (2026-07-21, journal/0039–0049, corrections #23–#25),
all gates green on merged main:** deep-config plumbing (0039) · walk 0040 +
the amplitude answer "neither" (#23) · S13 roughness measurement (0041,
#24/#25) · `--horizon` knob (0042) · climate registration fix (0043) · **U8
tectonic flip (0044)** · ore textures (0045) + substance redo (0048) · steep
walk 0046 · **full_agents flip + tour map (0047)** · **the first live
guided-tour ratification (0049)**. Doctrine: ledger-expression + enhancement
+ perf-first + genesis/stubs (stubs.md, 11 entries). Ore design pass drafted
(ores.md R1–R8 pending).

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

**The entry declares its own supersession in its first six words.** A 2026-07-21 close block that survived the 2026-07-29 close-block archive only because it is not written under a `## NEXT SESSION` heading. Its walk directive was discharged the next day (journal/0057-0059) and its item (5) already carries `DONE 2026-07-21`.

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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared resolved (journal/0065, 2026-07-21). **It is also the refutation of the `GPU DeviceLost crash under a teleport storm at --horizon 6` entry archived below** -- the two are moved in the same commit so the claim does not outlive its refutation, which is the failure this whole pass exists to stop. *Its measurements are a dated record and were not re-taken.*

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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared superseded, with a `RE-SEQUENCED 2026-07-21` list whose items all carry their own `SHIPPED`/`DONE`/`MEASURED` stamps, and a `Read first next session` directive four close blocks stale.

*(Superseded by a session-close block; kept for the record. **The pointer used to read
"above" and no longer resolves:** the block that superseded this was the 2026-07-21 one,
written when close blocks sat at the top of the board — it was consumed by its successors
and is not retained anywhere. The surviving blocks are § **NEXT SESSION** **below**
(2026-07-27, and the 2026-07-26 morning block under it) plus the six archived in
`ROADMAP-history.md`.)*
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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ DECIDED 2026-07-28` and both halves shipped: the `staleness-sweep` skill and `scripts/sweep_due_hook.py`, both present in the tree. Kept live only for its reasoning, which is preserved here whole -- including the honest limit that *"a green sweep must never read as 'the corpus is sound'"*.

- **✅ SWEEPS RUN FIRST THING, INCREMENTALLY, AND THE HARNESS SAYS WHICH ARE DUE**
  (**DECIDED 2026-07-28, user**: *"sweeps should probably run first thing… additional sweeps
  should be able to focus mainly on new stuff since last time, or full audit if the underlying
  source has moved (update to spines, etc)"*).
  - **THE DIAGNOSIS THAT PRODUCED IT.** This repo has four corpus controls and **three share one
    trigger: the main session remembering.** Measured over eleven active days — `spine-audit`
    left **zero** artifacts despite its own *"run a few times a day"*; `doc-topology` ran **once**,
    the day it was created; the staleness sweep ran **twice** and was never even a skill. **The
    filesize hook is the only control not gated on memory, and the only one that fires
    reliably.** *And the remedy on file — "make the staleness sweep recurring **like
    spine-audit**" — was wrong in an instructive way: it assumed skill-packaging produces
    recurrence, and `spine-audit` is the disproof. **A skill still waits to be invoked.***
  - **SHIPPED (a): `staleness-sweep` skill** — the procedure existed since 2026-07-24 with two
    worked audits; this is packaging, and packaging alone was explicitly **not** the fix.
  - **SHIPPED (b): `SessionStart` hook** (`scripts/sweep_due_hook.py`) — states which sweeps are
    **DUE**, in which **MODE**, and **WHY**, computed from `docs/audits/.sweep-watermarks.json`.
    **This is the half that satisfies CLAUDE.md § Gates' rule** — *do not answer "the gate cannot
    see X" with a rule asking people to remember X.* **It reports; it does not dispatch** — an
    agent cannot self-dispatch and the spend is the session's and the user's call.
  - **THE RULE THE USER'S TWO HALVES IMPLY, stated so it is checkable:** a sweep's **incremental**
    mode is valid only while its **REFERENCE side** is unchanged; when the reference moves, every
    prior verdict was made against a different rule → **FULL**. `spine-audit`'s reference is
    `spines.md` (mechanical: does the diff touch it). The staleness sweep is *inherently*
    incremental — but goes full when a **recalibration** lands, because one commit can turn a
    whole cohort of old observations into artifacts (journal/0111) and no per-entry reading finds
    that. **`doc-topology` has NO reference side** and its unit is a *pair*, so incremental there
    is **new × ALL**, not new × new — done as: read changed docs in full, then let **their nouns**
    drive the search across everything else. *That inverts grep's known weakness — the search
    terms come from the diff rather than from the reader's suspicion.*
  - **The watermark is written BY the sweep, in the same commit as its audit** — never a separate
    step. Chosen from the measured adoption law: a convention survives when it is inseparable
    from an act the author must perform anyway (`JUSTIFIED-BY`, which asked for a restatement,
    got **3 uses, 0 in `crates/`**).
  - **⚠ HONEST LIMIT:** staleness is **3–8 %** of recorded failures; ~91 % were wrong the day
    they were written. **A green sweep must never read as "the corpus is sound."**
  - **Deliberately NOT built:** no claims index / knowledge graph — the noun-driven incremental
    mode needs no durable artifact, and *"the corpus already authors the graph; nothing reads
    it"* plus "don't build the general mechanism first" both bind here.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ DONE`; both shipments verified present (`.claude/skills/doc-topology/SKILL.md`, `ROADMAP-history.md`). Kept for the three-way VOLUME / TOPOLOGY / AUTHORITY diagnosis, which is preserved here -- including its own honest limit, *"the archive buys less than it looks like it does"*, a judgement this second archiving pass confirms.

- **✅ DONE 2026-07-26 — THE DOC-TOPOLOGY SWEEP + THE ROADMAP ARCHIVE** (user-directed at the
  close: *"our docs ops past critical mass, causing information loop to fail to close often"*
  — sequenced and executed the same session). *Entry kept for its reasoning, because the
  reasoning is what makes the sweep recurring rather than a one-off tidy.*
  - **THE DIAGNOSIS SEPARATES THREE FAILURES THAT LOOK LIKE ONE.** journal/0119's collapse
    was **not** a volume failure: the sketch (`ideas.md`), the reconciliation
    (`material-behavior.md` § 5), the refutation (`journal/0090`) and the unbuilt RATE axis
    were **all in the corpus, all findable by grep**. What failed is that **no reader ever
    had all four in view at once**, because the connections run *between* documents.
    - **VOLUME** — the board is grepped, not read, and grep returns only what you already
      suspected (`DeepField::chapters`, three spine audits). *Real, and third in value.*
    - **TOPOLOGY** — nothing compares docs to *each other*. `spine-audit` checks docs vs
      **code**; the staleness sweep checks entries vs **newer work**. **A claim and its own
      refutation can coexist forever.** *This is the one that cost an architecture.*
    - **AUTHORITY** — an assistant reconciliation could silently supersede a ratified user
      design. **Fixed the same day** by one CLAUDE.md rule; cheapest and highest-value of the
      three.
  - **SHIPPED (a): `.claude/skills/doc-topology/SKILL.md`** — five contradiction shapes in
    value order, prioritised by **blast radius not age**, requires `file:line` on **both**
    sides of a pair, and forbids the sweeper from resolving what it finds (which side wins is
    frequently a user call — that is *why* it survived). Provenance breaks ties:
    *user-originated constraints are data; assistant-originated ones are hypotheses that
    happened to survive.* **Run after any batch of merges that ships an arc**, and whenever a
    design thread reopens something old.
  - **SHIPPED (b): `ROADMAP-history.md`** — archived **by STATUS, not age**. Shipped (2,297
    lines) + six superseded close blocks moved out; the live board went **7,210 → 4,410**.
    Each Shipped entry keeps its **journal number** as the stable pointer, and the journal
    already holds the narrative. *Age is the wrong axis: a two-week-old `Observed` may be the
    liveliest thing on the board.*
  - **⚠ HONEST LIMIT — the archive buys less than it looks like it does.** 4,410 lines is
    still past reading whole, and **neither half of it would have prevented journal/0119**.
    The remaining volume lever is `Observed` (1,969 lines), which is *not* archivable by
    status — an Observed entry is live by definition. **Left open deliberately**: the
    thresholds and the split conventions are the user's to set, not the hook's provisional
    guesses.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ DONE`, merged and gate-verified (85 binaries / 808 passed), `corrections #66`, narrative in journal/0121. **Checked before moving: its one live residual -- `approx_resident_bytes` is non-monotone in extent -- is duplicated in the surviving tail of the five-rulings entry, so it stays on the live board.** Moving both copies would have dropped an observation.

- **✅ DONE 2026-07-28 — THE BOOTSTRAP HISTORY CONTENT IS REMOVED** (journal/0121,
  **corrections #66**; merged and gate-verified on main: fmt 0, clippy 0, **85 binaries / 808
  passed / 0 failed / 3 ignored**, reconciling exactly against the 86/811 baseline — −1 binary
  (`s7_handoff.rs`) and −3 tests, **all three named and confirmed absent**). Net **−713** Rust
  lines. *Entry kept below for its reasoning; two findings that OUTRANK the removal are recorded
  here because neither was suspected:*
  - **⚠ NOT ONE GOLDEN MOVED — corrections #66.** This entry said *"The goldens will move and
    that is correct"*, and corrections #64 said the fingerprints are *"structurally downstream of
    the posts"*. **Both false, and falsified by measurement, not argument:** 0 wood voxels in
    `contents_contract`'s sample set pre-removal, and `geology`'s sampler covers `cz ∈ [−20, 24]`
    while the nearest post sits at `cz = −727`. All four byte-identity goldens ran and passed
    **by name** on merged main. **The transferable lesson: *a pre-authorised golden move is
    indistinguishable from an unexplained one, which is the opposite of caution.*** A new A-2
    sub-shape — not a justification outliving its premise, but a **permission** outliving its
    justification, and never true. *`spines.md` A-2 records it: "structurally downstream is a
    statement about the call graph; whether a fingerprint moves is a statement about which chunks
    the sampler visits" — the same reflex produced both halves, six lines apart, in the entry that
    named the reflex.*
  - **🔴 FOUR OF THIS SLICE'S FINDINGS ARE USER DECISIONS AND LIVE IN ONE PLACE:
    § Sequenced → ~~"USER DECISIONS OWED"~~ **"✅ ALL FIVE USER DECISIONS RULED 2026-07-28"**
    *(pointer repaired 2026-07-29, baseline sweep S5/F9: no heading named "USER DECISIONS
    OWED" has ever existed, so a reader greping the quoted string found only the two pointers
    and never the target — and the target is now RULED, not owed)*
    (items 1–4: the 102-post appearance notification · the
    three design docs that still describe civ/history as a live pipeline stage · dc-sim's S2 tier
    now having zero production callers, with the slice's deviation plea · the producer-less
    settlement/civ schema). **Consolidated there rather than duplicated here**, because a live
    decision buried inside a `✅ DONE` block reads as closed — and because two copies of a
    decision are two things to drift. *This pointer is deliberately reciprocal: that entry names
    this one. Written this way on purpose — the session that wrote it had just measured that **8 of
    15** correction→file edges in this corpus exist only at one end.*
  - **Rides as built:** `Block::Wood` stays with no worldgen emitter (removing the variant would
    renumber block ordinals and move every golden — destroying the attribution this slice was able
    to make); five retired draw salts leave a **deliberate hole** at `0x5700_0005`…`0x5700_0009`
    with an in-code rule *take the next unused value, never fill a hole*; `stubs.md` #1 resolved
    by deletion, #10's blast radius restated as zero; **draw-domain part (a) is discharged** by
    removal rather than conversion.
  - **Accept-by-outcome, not by gate:** the 12 chunks that carried every post go **102
    `Block::Wood` → 0**, and a 200-chunk box around all four former clusters reads **0**.
  - **Residency, absolute:** the whole recorded settlement history — 153 facts, 13 sites, 2
    polities, 90 collapses — cost **5,520 bytes of 377,364,589** (0.0015 %), and the only
    production code that ever read it was the function measuring its size.
  - **New Observed item, not caused by this slice:** `approx_resident_bytes` is **non-monotone in
    extent** (Medium **377 MB** > Large **213 MB**) because the deep grid is width-capped
    (`cell_m = max(extent_m/DEEP_MAX_WIDTH, DEEP_CELL_M)`) and record size is deposition-dependent.
    A reader of `s7_measurements`' table would find that puzzling and has no note to reach.
  - *Original entry, preserved:*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

The *"Original entry, preserved"* half of the `✅ DONE` removal above. Verified at source 2026-07-29: `grep -rn "ruin_posts\|pregen/history" crates/ --include=*.rs` returns nothing.

- **🔴 REMOVE THE BOOTSTRAP HISTORY CONTENT — polities, sites, ruins, the history pass**
  (**DECIDED 2026-07-26, user**). *"They are unratified zealous fabrications from the early
  bootstrapping of the project and I DO NOT care about them, they WILL be wholesale
  replaced, they should just be removed. We do NOT have any form of evo/socia/civ modeling
  even at the design stage: they are NOTHING."*
  - **WHY IT IS A REMOVAL AND NOT A MIGRATION.** There is no design, no model, and no
    plugin-pack intent behind any of it. It is not built on the SDK pass shape and could not
    be — **we have never designed a mechanism for declaring structures/blueprints and
    spawning them in the world at all**. Keeping it means keeping goldens that protect
    content nobody voted for.
  - **THE CONSUMER GRAPH IS ALREADY TRACED** (journal/0118's rider, corrections #64): the
    history pass is an unconditional `vanilla_passes()` member; ruins reach the screen via
    `collapse.rs::ruin_posts` → `Block::Wood` in `generate_chunk`; and
    `Pregen.{ledger, overlay, n_polities, observe_count}` have **exactly one non-test reader
    in the workspace** — `approx_resident_bytes`, which only measures their size.
  - **SCOPE:** `pregen/history.rs`, `Pregen.sites` and the four fields above,
    `collapse.rs::ruin_posts` + its `Block::Wood` emission, dc-sim's region/agent-step draws,
    and the pass's `vanilla_passes()` membership. **The goldens will move and that is
    correct** — the scratch-pad rule applies exactly (*byte-identity is a regression
    detector, not a specification*).
  - **IT ALSO DISCHARGES draw-domain part (a)**, which was sequenced as a user-owned
    appearance slice solely to protect this content. With the content gone the residual
    `engine.rs` draw has nothing to re-roll.
  - **⚠ EXISTENCE IS NOT STANDING** (CLAUDE.md § Conventions, added the same day): the
    integrator proposed *counting* the ruins before the user's direction landed, which
    already concedes that some number would matter. It would not.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared shipped, and its Shipped entry is the first one in this file.

*(The `production_* → golden_*` rename that stood here — opened 2026-07-25 by journal/0106,
given a real entry by the staleness sweep row D-2 — **shipped 2026-07-26**; see `ROADMAP-history.md` § Shipped.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ DONE 2026-07-25` (journal/0100), with the measured result in its own head: `DeepField` 311.02 → 179.12 MiB, ledger heap 155.01 → 16.31 MiB, world byte-identical. Kept *"as shaped, for the record"*. **Its correction is worth carrying forward: triangularity is a budgeting tool for projecting an UNBUILT record, never a sizing rule for a BUILT one.** *The MiB figures are a dated record; residency has moved since and they were not re-measured.*

- ✅ **DONE 2026-07-25 — shipped, see `ROADMAP-history.md` § Shipped (journal/0100).** Measured result: flag-ON
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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ SHIPPED 2026-07-25 (journal/0099)`, kept for its reasoning. The flag-ON walk it earned was item (4) of APPEARANCE WALKS OWED, also archived here, also `✅ DONE` -- user at the station: *"Success on the gradation! ... Our world just got far deeper and more interesting to look at, just with this."*

- **✅ SHIPPED 2026-07-25 (journal/0099) — see `ROADMAP-history.md` § Shipped, journal/0099, for the result.** *(Entry kept for its
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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared SHIPPED 2026-07-21, journal/0042.

- *(**Far-field horizon knob: SHIPPED** 2026-07-21, journal/0042 — see `ROADMAP-history.md` § Shipped.
  `--horizon <km>`, default provably unchanged, measured to 10 km.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

**Struck by the user in its own header**, with the reasoning quoted: *"that whole mechanism changes after water machinery. that would be a bandaid, against our standing rule against bandaids. can revisit later."* The confirmed null (0.68 m) rides as-built until the fetch model replaces the constant -- which is the live littoral-heir line in § In flight.

- **Wave-magnitude retune — STRUCK 2026-07-21 (user): no retune.** "That
  whole mechanism changes after water machinery. that would be a bandaid,
  against our standing rule against bandaids. can revisit later." The
  confirmed null (0.68 m, journal/0049 station 5) rides as-built until the
  fetch model (wave energy from S11 body size/shape/depth × the 0037 wind
  field) replaces the constant outright — wave expression is a consumer of
  the water design pass now, not a tuning slice.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared SHIPPED. The duplicate of this entry in § In flight is archived above; the two sat in different sections of one file for nine days.

- *(**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037 — see
  `ROADMAP-history.md` § Shipped.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared SHIPPED. Its four launch flags are still the door every dev override rides through (`DeepOverrides` on top of `production_config`).

- *(**Deep-config flag plumbing: SHIPPED** 2026-07-20, journal/0039 — see
  `ROADMAP-history.md` § Shipped. The four launch flags (`--tectonics`, `--full-agents`,
  `--amplitude`, `--extent`) boot a flagged world; the combined amplitude /
  tectonic / full_agents walk is unblocked. The override channel is
  `DeepOverrides` on top of `production_config`, NOT a `WorldParams` field —
  the ~30 `{ seed, extent }` call sites were left untouched.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

**Self-labelled *"(superseded -- done)"*.** The direction it names shipped as the U8 tectonic flip (journal/0044).

- **Tectonic uplift-plane redesign — DESIGN PASS (superseded — done)** (direction ratified
  2026-07-20, earth-processes.md § 1 DECIDED entry): tectonic history
  (uplift(t), plate advection, chaptered boundary re-classification) +
  analytic boundary forcing (uplift from exact bisector distance at deep-grid
  resolution). Spike-class; upstream of dip/fold, volcanism, and the
  amplitude call. Includes the plate-count/scale-compression knob decision
  (user-owned).

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared SHIPPED 2026-07-20 (journal/0029). The user decisions it points at stay on the live board immediately below where this stood.

*(**Erodibility coupling — lithology-aware erosion: SHIPPED** 2026-07-20,
journal/0029 — see `ROADMAP-history.md` § Shipped. Cause 1 of the dismal mountains is closed:
erosion is lithology-aware, off by default, byte-identical when off,
agent-specific resistance so karst/glacial/littoral stay implementable.
What remains is **user decisions**.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Self-declared SHIPPED 2026-07-20 (journal/0026).

*(**Collapse-tier organic materials + the production flip: SHIPPED**
2026-07-20, journal/0026 — see `ROADMAP-history.md` § Shipped. The `Biofacies` → class routing is in,
coal/peat/carbonaceous-mudstone exist, and the 24 m seam at world voxel
(107338, 58787) is diggable. Two follow-ons fell out of it, both
measurement-backed:)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`✅ BOTH FIXED -- verified at source 2026-07-29` (S6 finding F2). (a) shipped as the honest fix -- `BioMod` moved into the within-epoch `reads` roster; (b) fixed by **deletion**, which is the honest disposal -- `Exposed` is no longer declared, and `runner.rs:1497-1499` asserts it stays undeclared.

- ~~**Two declaration defects on the new `dc:deep/weather_inventory` pass**~~ **✅ BOTH FIXED —
  verified at source 2026-07-29** (S6 finding F2). This entry read as owed work and was not.
  - **(a) shipped as the honest fix, exactly as diagnosed.** `crates/dc-worldgen/src/deeptime/runner.rs:579-581`
    now carries `BioMod` in the within-epoch **`reads`** rosters (`WINV_READS_AGENTS` /
    `_TEC` / `_LEG`), the pass declares `reads_prev: &[]` (`:933`), and the module comment at
    `:560-567` states the reasoning verbatim — *"a `reads_prev` declaration was a fiction …
    Declaring it as a real `reads` makes the graph state what actually happens and PINS the order
    instead of inheriting it from a tie-break."* Shipped by the spine-audit follow-through,
    2026-07-25.
  - **(b) fixed by DELETION, which is the honest disposal.** `Exposed` is no longer declared at
    all; `runner.rs:569-572` says so out loud — *"**`Exposed` is deliberately NOT declared**:
    susceptibility is a constant off `BEDROCK_SEAM_MATERIAL` … declare what you read, not what you
    intend to read"* — and `:1497-1499` **asserts** it (`assert!(!w.reads.contains(&DeepAxis::Exposed))`).
    The false declaration was removed rather than made true, and there is now a test that fails if
    anyone re-adds it without the genesis heir. *(Line refs re-verified 2026-07-29 at the
    `docs/design/pass-declaration-history.md` extraction, post-E3 — the mover stamps the citing
    doc. Original diagnosis below, kept for the record.)*
  - **(a) `reads_prev: &[BioMod]` is not what happens.** The pass reads `grid.bio_weather`, which
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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`FIXED` in its own body (occupancy-aware culling, seven tests by name) and its `UNWALKED` tag discharged in place: `✅ WALKED AND CONFIRMED` (S6 finding F1, applied 2026-07-29).

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
  ~~**UNWALKED** — nobody has seen the holes gone; re-walk
  `0056-holes-after-settle.png`'s coordinates, and if bands persist, distrust
  journal/0057 first.~~
  **✅ WALKED AND CONFIRMED — the tag was discharged on 2026-07-22 and nobody
  updated it** (S6 finding F1, applied 2026-07-29). The walk-0059 session at
  `--horizon 3 --fullbright --edges` answered it YES by eye: **no sky-holes** at
  two partial-rich stations, journal/0057 confirmed. The confirming entry sat
  ~1,160 lines below this tag in the same section for seven days and was never
  read back against it — the closure-in-the-wrong-place shape, inside the one
  file every session opens. **It is now in `ROADMAP-history.md` § Observed —
  archived** (struck by the user 2026-07-29 for its *third*, still-open answer;
  the two YES answers are preserved there verbatim). Assets `0059-*`.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Its own header ends `(FIXED -- see above)`, and the *above* is archived with it. **Its wider lesson is why it is preserved rather than deleted**: journal/0010 shipped partial-height rendering dormant and predicted it would *"light up for free"*; it lit up and did not work, because the assumption it rested on lived in a doc comment nobody re-read when the world changed underneath it.

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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

`FIXED` (journal/0058; chunk footprints expressing more than one surface member went 0/169 → 147/169) and its `Unwalked` tag discharged in place: `✅ WALKED AND CONFIRMED` (S6 finding F1).

- *(**"Surface material is quantized per chunk": FIXED** 2026-07-21,
  journal/0058 — the member is now drawn per voxel column inside the shared
  `surface_sample` kernel. Chunk footprints expressing more than one surface
  member went **0/169 → 147/169**. Block fingerprints unchanged in both
  recorded worlds, which is within-class invariance confirmed by an 80-chunk
  fingerprint that knows nothing about the argument. ~~**Unwalked** — the fix
  landed after the user's session closed, so nobody has seen the patches
  gone.~~
  **✅ WALKED AND CONFIRMED — the tag was discharged on 2026-07-22 and nobody
  updated it** (S6 finding F1, applied 2026-07-29). Walk 0059: **no 28.8 m chunk
  patches** — a 120 m top-down frame shows organic blobs with wandering
  contacts, journal/0058 confirmed. Same seven-day gap as the sky-holes tag
  above, discharged by the same entry, now in `ROADMAP-history.md` § Observed —
  archived.)*

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Struck in place `✅ IT EVICTS -- verified at source 2026-07-29` (`crates/dc-api/src/host.rs:524`, a bounded LRU with hysteresis in which edited entries are protected). It was kept live that morning **only** so that archiving its refutation would not leave the claim standing alone; both are now in this file together, which is the correct end state.

- ~~The embedded HostWorld never evicts chunks (~64 KiB per chunk ever
  streamed/edited); never-edited chunks are pure generator output and could
  be dropped freely (journal/0002).~~
  **✅ IT EVICTS — verified at source 2026-07-29** (S6 audit § 2 #1).
  `crates/dc-api/src/host.rs:524` is `fn enforce_chunk_budget(&mut self, protect: Option<ChunkPos>)`,
  a bounded LRU with hysteresis, called from `set_chunk_budget` and from the materialize path;
  `:246-250` documents it as a bounded LRU in which **edited entries are protected** — exactly the
  generated-untouched-vs-edited split this entry asked for. Shipped 2026-07-21 by journal/0051.
  **The refutation was 700+ lines above this line, in this same section, for eight days** (*"FIXED
  2026-07-21 (journal/0051): eviction landed, the march is flat (+27.4 → 0.00 MB/jump)"* — now in
  `ROADMAP-history.md` § Observed — archived). A claim and its own refutation coexisting in the one
  artifact every session opens: the corrections #65 shape. Discharged here so archiving the
  refutation does not leave the claim standing alone.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Struck in place `✅ THE SUBJECT IS GONE -- verified 2026-07-29`. A scale constraint on content removed 2026-07-28 (journal/0121). **S2's ledger-scale question itself is a different, live entry** -- the 🔔 TRIGGERED S2-checkpoint-facts item, which stays.

- ~~Site cap 240 (u8 RegionId) — concrete instance of S2's ledger-scale
  question (S7).~~ **✅ THE SUBJECT IS GONE — verified 2026-07-29** (S6 audit § 2 #4).
  `grep -rn "RegionId" --include=*.rs crates/dc-worldgen/` returns **zero hits** and `sites` is
  absent from `crates/dc-worldgen/src/pregen/mod.rs`; the bootstrap settlement-history content was
  removed 2026-07-28 (journal/0121). This was a scale constraint on content that no longer exists.
  *(The surviving `RegionId` hits in the tree are `crates/dc-sim/src/statistical/engine.rs` — an
  unrelated toy-world type.)* **S2's ledger-scale question itself is live and is not this entry** —
  it is the 🔔 TRIGGERED S2-checkpoint-facts entry below.

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Struck in place `✅ ANSWERED AS POSED -- verified at source 2026-07-29` (`crates/dc-client/src/perf.rs:26,40,68-69`). **Its live residual is its own entry and stays**: *"the perf instrument can't show the frame-thread envelope"* -- per-thread-role attribution is what is actually missing. The two sat ~1,000 lines apart never referencing each other, and **a complaint about an instrument is not evidence the instrument is absent.**

- ~~**We cannot see where runtime goes — the perf observability gap**~~ **✅ ANSWERED AS POSED —
  verified at source 2026-07-29** (S6 audit § 2 #5). The instrument exists:
  `crates/dc-client/src/perf.rs` — `:26` documents that *"under `--features perf` it expands to a
  real `bevy::log::info_span!(..)"*, `:40` is the expansion, `:68-69` pulls in
  `bevy::log::tracing::{Subscriber, span::Id}` for a custom collector. So the entry's flat claim of
  *"**NO** runtime frame/tick observability"* is false today.
  **The live residual is already its own entry, above:** *"the perf instrument can't show the
  frame-thread envelope"* — `PerfAggregate` sums self-time across the frame thread AND the
  task-pool threads, so per-thread-role attribution is what is actually missing. **A complaint
  about an instrument is not evidence the instrument is absent**, and the two entries sat ~1,000
  lines apart never referencing each other. *(Original entry below, for the record.)* (user,
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

**ARCHIVED 2026-07-29 — the entry's own body already declared it closed** (moved by status, not by age; reproduced verbatim).

Struck in place `✅ THE FILE IS DELETED -- verified 2026-07-29`. **The pack-degradation doctrine it invoked is explicitly untouched and still binds** (API.md: degradation must be LOUD) -- only this instance of it is void.

- ~~**The `history.rs` reject-don't-crash skip is SILENT**~~ **✅ THE FILE IS DELETED — verified
  2026-07-29** (S6 audit § 2 #2). `crates/dc-worldgen/src/pregen/history.rs` does not exist and
  `RegionId` has zero hits in `dc-worldgen`; the bootstrap history content went 2026-07-28
  (journal/0121, −713 Rust lines) under *existence is not standing*. This entry filed an owed
  warning on a code path that no longer exists. **The pack-degradation doctrine it invoked (API.md:
  degradation must be LOUD) is untouched and still binds** — only this instance of it is void.
  *(Original entry below, for the record.)* (2026-07-20,
  from the circulation merge). When a relocated-settlement graph
  over-constrains the S2 pressure collapse, `history.rs` now skips the
  observation instead of panicking (integrator-approved) — but the
  pack-degradation doctrine (API.md) says degradation must be LOUD. A
  skipped world-history collapse currently emits nothing; it owes a named
  warning. Small.

### Resolved by later work the entry did not know about

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Shipped and ratified in its own body:** *"MERGED TO MAIN + LOOK RATIFIED AS-BUILT (2026-07-22, integrator merge; user, from the 0070 screenshot set: 'visually indistinguishable from the previous version, for me. which is good!')"*. FF2b-minimal landed as journal/0070.

  ⚠ **FOUR FOLLOW-ONS TRAVEL WITH THIS ENTRY AND ARE OWNED NOWHERE ELSE** -- persistence + dirty-rail (far edits), synthesized sub-surface strata (the `surface_sample` summarization-half home), partial-coverage composition, and deep-span greedy merge. They were live when this moved. **Re-file them on the live board**; the archive is not a home for an unowned obligation.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

A pointer to a spike that has had its own § Sequenced entry since 2026-07-20 (*"Tectonics SPIKE (per tectonics.md § SPIKE, architecture ratified 2026-07-20)"*), and to a dispatch gate -- *"behind the eolian agent's landing"* -- discharged 2026-07-21.

- *(**Tectonics architecture RATIFIED 2026-07-20** — all of U1–U8, with
  U3 amended (ritual ceiling relaxed to "5 min if that's what it takes");
  see the tectonics.md banner. The SPIKE is next — sequenced below, behind
  the eolian agent's landing: both write `deeptime/erosion.rs`.)*

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Shipped 2026-07-20 as journal/0037. This board said so itself, one section over, for nine days -- the duplicate in § Sequenced is archived below.

- **Zonal circulation profile** — dispatched same moment (write-set
  disjoint from spike: climate.rs only). Smooth wind magnitude + subsidence
  aridity; kills the band-flip line and puts a desert belt at ~30° for the
  true reason. Changes every new world's climate — integrator presents the
  measured shift for the user's eye.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Its own header reads `✅ FIXED`, and `docs/design/stubs.md` § 29 agrees:** *"⚠ DISCHARGED 2026-07-29 (journal/0122) -- the operator is fixed, and one inference below is falsified (corrections #72)."* The live successor is `ROADMAP.md` § Sequenced *"THE HILLSLOPE OPERATOR IS FIXED -- SHIPPED 2026-07-29"*, which now points here rather than *"below"*.

  ⚠ **READ corrections #72 BEFORE CITING THE (b) BLOCK BELOW.** *"It is NOT a time-step limit"* is **wrong**: it was a stability limit, and journal/0116's 4× refinement was ~25× short of reaching the bound. Every measurement in the entry stands; that one inference does not. The `iso_rate` finding in (c) -- *"do not touch `iso_rate`, it is the only grid-scale low-pass in the solve"* -- is unaffected and is still worth reading.

- **~~🔴🔴🔴~~ ✅ FIXED — THE HILLSLOPE CONVEYOR CHECKERBOARDED THE REGOLITH ABOVE 1× — stubs #29,
  RE-SCOPED 2026-07-26 by the walk (journal/0115, corrections #61/#62) and then
  **DIAGNOSED 2026-07-26 by journal/0116 (corrections #63)**, and it BLOCKS the calibration
  below.** *This entry has been renamed twice. It read "THE INCISION CLAMP THAT WAS GREEN
  BECAUSE NOTHING ERODED" (sized at 148 pits — both wrong), then "THE EROSIONAL SOLVE GOES
  GRID-UNSTABLE ABOVE 1×" (right about the symptom, wrong about the mechanism: the
  discriminators say it is **not** a stability limit). The name now says where it is. The
  superseded framing is kept at the end because its mechanism is probably still real, just
  not dominant.*
  - **WHAT IS MEASURED (production-Medium, seed 1337, calibrated vs shipped as control).**
    Deep-cell concavity — `mean(8 neighbours) − self`:

    | | mean | p10 | p50 | p90 | p99 | >1 m | >20 m |
    |---|---|---|---|---|---|---|---|
    | shipped | −0.14 m | −0.3 | −0.1 | +0.1 | +0.3 | **0.0 %** | **0.0 %** |
    | calibrated | +0.73 m | **−50.8** | −0.1 | **+52.8** | **+106.3** | **35.8 %** | **23.2 %** |

    Closed hollows (`filled − routed`, the non-saturating census): **0 → 1,377 (3.1 % of
    land), deepest 112.8 m, 5.4 km³ of fill.** Regionally **7.8 % within 10 km** of the walk
    station — they cluster 2.5×.
  - **THE HEADLINE, and it is one sentence: relief grew 4.6 %, cell-to-cell roughness grew
    ~170×.** The shipped world's entire concavity distribution fits in ±0.3 m; the calibrated
    world's decile spread is ±50 m with the **median unchanged**. The landscape's *shape* is
    intact and the *grid* has become noise. Symmetric tails + untouched median = adjacent
    cells oscillating against each other.
  - **THE PITS ARE THE TAIL, NOT THE DEFECT.** A closed hollow is where the oscillation
    happened to bottom out with no outlet. Fixing the clamp would clamp the tail and leave
    23 % of cells 20 m off their neighbours — a slice that goes green and does not fix the
    world. **Do not brief the clamp fix as the blocker.**
  - **~~HYPOTHESIS~~ — THE DISCRIMINATORS RAN 2026-07-26 (journal/0116). THE STABILITY-LIMIT
    STORY IS FALSIFIED; THE DEFECT IS STRUCTURAL AND IT IS IN THE REGOLITH.** The standing
    hypothesis was *an explicit scheme past its stability limit*; it was flagged unmeasured,
    it was measured, and it is wrong.
    - **(a) It IS a checkerboard.** Concavity lag-1 autocorrelation **+0.377 / +0.267
      (shipped)** vs **−0.867 / −0.912 (calibrated)**, lag 2 back at +0.56 / +0.71,
      first-difference ACF −0.86. The three reference values are derivable in closed form —
      white noise **−1/6**, perfect checkerboard **−1** — so the discriminator is not "is it
      negative" but *how far past −1/6*. Both axes: a true 2-D Nyquist mode.
    - **(b) It is NOT a time-step limit.** Refined **4×** at fixed total simulated time
      (`k×` epochs, `1/k×` every per-epoch rate), concavity rms goes **40.46 → 45.29 →
      38.76** — 4 % under a 4× refinement, non-monotone — while the landscape holds (relief
      +3.9 %, mean surface −0.3 %) and the shipped control reproduces to three digits. The
      checkerboard gets **purer**: ACF(1) −0.867 → −0.909 → **−0.947**. Closed hollows *do*
      converge (1,377 → 955 → 280), so the **pits** are partly a step artefact and the
      **oscillation is not**. *`myr_per_epoch` does not exist as a knob; the register is
      `iterations` against per-epoch rates. The claimed shared register with stubs #27's
      heir (b) is **withdrawn** — they share the limiter, not the clock.*
    - **(c) ISOSTASY IS THE DAMPER, NOT THE DRIVER** (mechanism proposed mid-flight, killed).
      `iso_rate` 0.50 → 0.25 → 0.00 takes concavity rms **40.46 → 62.03 → 90.34** and hollows
      **1,377 → 2,150 → 13,012**. On the **shipped** world `iso_rate = 0` takes rms 0.22 →
      19.71 and hollows **0 → 6,215**. **Do not touch `iso_rate`** — it is the only
      grid-scale low-pass in the solve.
  - **WHERE IT LIVES — split `surf = r + h` and this is what the fix slice is briefed
    against.** Same Laplacian over each summand, plus the creep limiter's binding fraction:

    | | limiter bound | conc(**r**) rms · ACF(1) | conc(**h**) rms · ACF(1) | surf rms | mean h |
    |---|---|---|---|---|---|
    | shipped k=1 | 88.7 % | 3.42 m · −0.10 | 3.43 m · −0.10 | **0.22 m** | 4.58 m |
    | calibrated k=1 | **96.0 %** | 23.77 m · −0.55 | 61.95 m · −0.82 | 40.46 m | 41.41 m |
    | calibrated k=2 | **94.9 %** | 12.59 m · −0.34 | 55.21 m · −0.88 | 45.29 m | 36.59 m |
    | calibrated k=4 | **94.7 %** | **5.88 m · −0.11** | **42.43 m · −0.93** | 38.76 m | 24.45 m |

    - **The BEDROCK solve converges** (23.77 → 5.88 m, ~`1/k`, ACF back to −0.11): there is a
      real time-step artefact in this world, it is in `r`, and D2 converged it away.
    - **The REGOLITH does not.** `conc(h)` falls 32 % while its ACF sharpens to **−0.93**, and
      the refinement does not hold the cover fixed (mean `h` 41.4 → 24.5 m), so normalised by
      what the operator moves the roughness **grows**: `conc(h)/h̄` 1.50 → 1.51 → **1.74**.
      **The flat surface total was two defects cancelling.**
    - **The limiter is deaf to the step: 96.0 → 94.9 → 94.7 %.** Measured, not inferred. It
      caps export at *the cover the cell has*, so the transfer is a function of **inventory,
      not `rate × dt`** — which is why refining `dt` did nothing.
    - **Saturation alone is NOT sufficient — do not brief it as if it were.** The limiter
      binds on **88.7 %** of *shipped* cells and their `conc(h)` ACF is −0.10. What the
      calibration adds is **cover** (4.58 → 41.41 m mean regolith).
    - `corr(concavity, h − h̄) = −0.831` calibrated (+0.147 shipped); `rms(h − h̄)` 70.1 vs
      4.0 m. On the shipped world `r` and `h` roughness **anti-correlate almost exactly** —
      3.42 + 3.43 m of component concavity summing to 0.22 m. **That compensation is what
      broke.**
    - **Register: the flux limiter / donor-cell partition in `erosion.rs::diffuse`.**
      **Hypothesis for the slice to test first, explicitly not measured:** a donor-cell scheme
      that moves everything downslope has a period-2 mode by construction (A gives all its
      cover to B; B is now higher and gives it back), damped only by isostasy downstream.
  - **Until this lands the engine cannot run erosion at ANY realistic rate.** Unchanged, and
    now for a better-understood reason. It gates journal/0114's flag flip and every future
    calibration.
  - **⚠ THE ACCEPTANCE CRITERION THAT MISSED IT (corrections #61).** journal/0114's binding
    criterion was *"relief within 5 %"*. Relief is `max − min` — a **global extremal**
    statistic that is mathematically incapable of seeing spatial arrangement; you can shuffle
    every interior cell and leave it unchanged. **Any future erosional slice pairs its
    aggregate criterion with a neighbour-relative one** (Laplacian, gradient distribution,
    autocorrelation), or it is measuring the axis that did not break.
  - **⚠ AND THE GUARD CANNOT FAIL INFORMATIVELY (corrections #62).**
    `mfd_routing::no_interior_cell_is_cut_below_all_of_its_neighbours` counts cells below
    **all eight** neighbours — a winner-take-all predicate that **saturates**: as the defect
    generalises, neighbours sink too and stop qualifying each other, so the count falls back
    toward zero exactly when the damage becomes universal. Re-assert it on **fill depth and
    concavity**. *Found by the user flying the terrain, after two probes and a gated assertion
    all agreed with each other and were all wrong the same way.*
  - *(SUPERSEDED FRAMING, kept because it is probably a real contributing mechanism: the
    never-incise-below-the-lowest-receiver clamp is applied at incision, and weathering,
    creep, wave and eolian all run **after** it in the same epoch and can lower a cell past
    its floor. That predicts **isolated deep holes**, which is a subset of what the world
    shows. Its A-2 variant in spines — a test's unstated premise, "erosion is fast enough for
    this to mean anything" — stands on its own merits.)*

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Its headline mechanism is falsified.** `docs/design/stubs.md` § 27, 2026-07-29: *"⚠ THE CONVEYOR IS GONE 2026-07-29 (journal/0122) -- and the calibration fitted on top of it does not survive. The cap was a symptom of the same defect as § 29."* The entry still asserted *"the pass is a one-cell-per-epoch conveyor"* as a live finding. **Its surviving contribution -- journal/0114's six measured 200-epoch worlds -- is carried forward as INPUT by the live `RE-PICK EROSION_CALIBRATION AGAINST THE FIXED OPERATOR` entry**, whose own ladder inverts this one's headline (cover now *thins* with the multiplier where it used to thicken).

- **🔴 THE TRANSPORT OPERATOR HAS A CEILING — stubs #27** (journal/0114). Six full 200-epoch
  worlds measured (1×, 10×, 45×, 100×, 300×, 1000×): **export is proportional to mean regolith
  thickness**, and **creep's flux limiter already binds on ~89 % of cells that have regolith to
  move, at the SHIPPED rates.** The pass is a **one-cell-per-epoch conveyor**, so 100× on
  transport alone buys **1.6×**, and reaching the craton band costs order **100 m of cover**.
  - **This revises journal/0111's diagnosis.** *"A calibration, not an architecture"* was **half
    right**: the constants are wrong **and** the transport operator is capped. Named heirs:
    rivers that actually carry, or a non-capped creep operator.
  - The uniform-scaling hypothesis was **falsified by a diagnostic the agent added because the
    hypothesis needed a falsifier** — not by argument.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Nothing in it is owed.** (1) `✅ DONE -- walk-confirmed 2026-07-24`; (2) `✅ DONE -- walk-confirmed & ACCEPTED 2026-07-25`; (3) tour-mapped to a **null** -- the shipped world has zero coal, *"do not spend a walk on it"* (corrections #51); (4) `✅ DONE -- WALK-CONFIRMED & PASSED 2026-07-25`; (5) `NEVER OWED -- ANSWERED AT THE DESK`.

  ⚠ **ONE RESIDUE TRAVELS WITH THIS ENTRY:** *"the poke-through geometry check on the lit pass -- low priority"*, left over from walk (1). It was live when this moved. The next owed appearance walk should open a fresh tracker rather than resurrect this one -- an empty tracker is not a live entry, but a residue inside an archive is a lost obligation.

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
  `mixed_voxel_contents`); **(4)** ✅ **DONE — WALK-CONFIRMED & PASSED 2026-07-25** (user, at the
  station, `--weather-inventory --fullbright`, bench cut; assets
  `0099-weathering-front-profile-bench.png`, `-full-section.png`): *"Success on the gradation!
  Aesthetically, which is all I can judge here, this is a pass. **Our world just got far deeper and
  more interesting to look at, just with this. The spawn area isn't a shallow pile of rubble over a
  harsh boundary of uniform rock anymore.**"* Measured on the record **before** the screenshot so
  the picture could not flatter it: product **6→5→4→3→1** eighths downward, parent structure
  **4→5→7**, form flipping at 295/294 from debris to `structure`+`pore_fill`, deepest front voxel
  **7/8 parent + 1/8 product**. The journal/0097 hard perimeter is gone at **both** faces.
  *(Live bonus: `has_contents:false` on the basement below — journal/0101's `identify(pos)` fix
  working in the field, where the query used to claim `dc:air` over solid stone.)*
  ~~**(4)** the **weathering-front PROFILE** flag-ON walk~~ (journal/0099,
  shipped 2026-07-25) — the band is now a graded **19-voxel** front, ~**2.67×** deeper than the
  old slab, with **retained parent structure at the bottom contact** (7/8 parent + 1/8 product)
  instead of a hard perimeter; station world **(84185 m, 9212 m)**, voxels **y=299…281** — cut a
  **bench** (not a pit), `--fullbright`, read with `world_get_contents`; **(3)** ⚠️ **the
  geotherm's coal-distribution shift — THE WALK IS A NULL AND NEEDS NO GAME TIME** (tour-mapped
  2026-07-25, `examples/coal_walk_tour.rs`; corrections #51): **the shipped world has ZERO coal**,
  so there is nothing to look at. The appearance question the user was going to be asked ("is this
  seam thick enough?") is **replaced by a content question** — *"is a coal-free world acceptable
  for now?"* — answerable at the desk, not in-game. **Do not spend a walk on it.** See the
  🔴 Observed entry. *(A fallback peat station exists if the world is ever walked for organics
  anyway: world −8266, −45533, surface 263.7 m, 2.7 m of peat outcropping at the surface — no
  bench needed; nearest-to-Station-A alternative at 58867, −34963, 50.9 km away.)*
  **(5)** ~~the `pore_rider_share` **correlation** walk — a fullbright walk along a strong front
  looking for banding correlated with the parent's eighth~~ **NEVER OWED — ANSWERED AT THE DESK
  2026-07-25 (journal/0105).** Filed here 2026-07-25 by sweep row D-5, which correctly caught that
  a proposed walk was living in an Observed entry and not in this tracker — but the walk had
  already been retired by the hash-domain slice that merged the same day. It measured the question
  spatially instead: on one 32×32 contact plane sharing one record and one fill plan, **every
  autocorrelation at lags 1–4 in both axes is inside ±0.07 of zero, before *and* after** the fix.
  Structurally absent, not merely subtle — both offsets are functions of a position hash, so a
  dependency between two decisions **at one voxel** cannot make structure **between** voxels.
  *Kept struck rather than deleted: the lesson is the tracker's, not the walk's — a walk proposed
  in an Observed entry and not listed here is a loose end by stubs.md doctrine, whichever way it
  later resolves.*
  Screenshots to `journal/assets/` named for their entry.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Verified false at source 2026-07-29.** `crates/dc-worldgen/src/collapse.rs` calls `self.evict()` at `:451`, `:711`, `:997`, `:1012`, `:1022`, `:1033` and `:1041` -- reachable from every sampling path, not only `generate_chunk`. Fixed by journal/0052, whose § In flight entry records it by name: *"Also landed: the journal/0050 collapse-cache `evict()` gap (now reachable from `coarse_surface`, `column_record`, `surface_elev_m`, `lattice_point`, `surface_chunk_y`)"*. **A claim and its own refutation, ~2,500 lines apart, in the one file every session opens.**

- **Collapse-cache `evict()` is unreachable from far-field-only sampling**
  (diagnosed 2026-07-21, journal/0050; `evict()` fires only from
  `generate_chunk`, so a `coarse_surface`/`column_record` sweep can grow
  `lattice_memo`/`locale_cache`/`region_cache` between chunk generations).
  Bounded in normal play (a storm generates chunks constantly) — not the RAM
  march, which was `HostWorld.chunks` and is now fixed. **Reassigned
  2026-07-21** to the forms/partials `collapse.rs` rewrite, which owns that
  file. *(**Separate hardening item — DeviceLost degrades loudly: SHIPPED**
  2026-07-21, journal/0054.)*

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Its stated deliverable was done, and it is the largest measurement on the board.** The entry asks to *"compare model denudation against real orogen rates"*; journal/0111 did exactly that with `examples/denudation_probe.rs` and its cited literature table, finding the shipped world denudes at 0.0110 m/Myr -- **9× slower than the slowest landscape ever measured on Earth.** The live successors are `CALIBRATE THE DEEP-TIME CLOCK` and `RE-PICK EROSION_CALIBRATION`, both of which carry this entry's method rule (against a published band, never against a look) as their binding constraint.

- **Erosion-supply calibration** (from the S12 spike's new finding,
  2026-07-20): exhumation comes out metre-scale at shipped erosion rates,
  gating exhumed-core/foreland legibility independent of amplitude — the
  S9 calibration loose end, now load-bearing. Needs-measurement class:
  compare model denudation against real orogen rates, propose 2–3
  calibrations, render each — the user then chooses between pictures, not
  rate constants. Dispatch after the combined walk settles amplitude.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

FF2a shipped (journal/0023). The FF2b paragraph attached to it is superseded twice over by the octree-substrate entry archived above: *"this supersedes FF2b's earlier pairing with the caves/underground water thread for the minimal slice"*, and `FF2b-minimal LANDED` (journal/0070). **The two follow-ons it names -- async meshing and a persistent edit-tracked LOD store -- survive in § Observed's Voxy-vs-Distant-Horizons entry, which stays live.**

*(**FF2a — voxel-language far field: SHIPPED** 2026-07-19, journal/0023 — see
`ROADMAP-history.md` § Shipped. Stepped columns retired the smooth TIN; step 0 empirically confirmed
Bevy 0.19's GPU-driven multidraw engages for our custom material
(`mode=Culling`, 74 draws → 1 multidraw set); the buried-sheet and tile-crack
field reports are resolved, below.)* **FF2b — coarse volumetric
summaries** paired with the caves/underground thread of the water design
pass (when overhangs exist, the summary goes 3D; couples to S3 region
storage). FF2a left the extension point ready: the per-column payload is a
`ColumnSpan` the mesher already treats as one of a potential stack, and the
tile mesher is a pure function of plain span data (async-meshing / persistent
edit-tracked LOD store stay drop-in).

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Both of its stated blockers are false at source, verified 2026-07-29.** It says *"no charcoal material was shipped"* and that `deep_class` routes a charcoal-tagged unit to its mineral host. But `crates/dc-core/src/materials/mod.rs:133` defines `CHARCOAL`, `crates/dc-worldgen/src/geology.rs:430` routes `Biofacies::Charcoal => CLASS_ORGANIC_CHARCOAL`, and `crates/dc-worldgen/src/fill.rs:419` makes it loose-formed. Shipped by journal/0063. **§ Observed already carries this same verification** (S6 finding F3) on the entry that stays live for its wider point about a conclusion outliving its premise.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Superseded by `docs/design/flow.md` (RATIFIED 2026-07-25) and part-delivered by `dc:field/head` (journal/0098). **Its surviving half is stated in full in the entry that stood directly above it, which stays live**: *"what this pass still owes is the PRESENT/RUNTIME tier -- visible and flowing water, ponds and sub-resolution water, speleogenesis, and the free-water body-graph coupling. Caves ride FLOW continuation (c)."* Two entries, one thread, the newer one already complete.

**Water-model design pass** (ratified 2026-07-19, user; field-notebook
first per the earth-processes method): groundwater as "another dimension
for the flow to go" — water table / aquifers (S8 per-voxel porosity is
the waiting substrate), ponds and sub-resolution water (procedural-tricks
tail), visible/flowing water (couples to PBR-2 water), lakes/inland seas
already implicit as deep-tier flooded basins (spill levels known).
**Groundwater ↔ CAVES coupling flagged by the user** — speleogenesis as
the eventual cave story (today's caves are S1 noise carving). Design doc
before any code.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Its single `OWED` shipped**: *"a tier flag that can say UNRECORDED as a first-class answer"* is `Identity::Unrecorded` (journal/0101), and CLAUDE.md § Agent walks now teaches the fixed behaviour -- *"`has_contents` is now a PER-VOXEL fact and is trustworthy (fixed 2026-07-25, journal/0101; it used to be answered per-CHUNK -- corrections #49)"*. **The diagnosis is preserved whole** because its mechanism -- a per-voxel question answered with a per-chunk presence test -- is the *"a summary is not an authority"* shape caught in the query surface, and because its sibling *bare-cell fallback* entry is a different, genuinely-open defect.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Refuted by the wide-horizons measurement archived above** (journal/0065): re-measured at horizon 6 in both regimes, alternated 6/3/6/3 against machine drift, plus one unbroken 700-jump / 23-minute horizon-6 session -- *"All runs exited 0, no `DeviceLost`, no panic, no `ERROR`."* The root cause was host-RAM exhaustion from an unbounded chunk store, fixed by journal/0051. **Its secondary defect also shipped**: *"a DeviceLost should not cascade into unwrap panics"* → journal/0054's honest exit codes, which retired the *"exit codes lie about GPU crashes"* warning in CLAUDE.md.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Its *"one genuine discovery"* is already `✅ VOID` in place -- the owed comment was on `collapse.rs::ruin_posts`, deleted 2026-07-28. Its residual -- *"the `field.rs` doc-comment claims the collapse tier reads exhum/t_crust when nothing does"* -- is owned by the live § Sequenced entry `METAMORPHISM -- the grade axis`, which names the same two planes and retires stubs #4. **`docs/design/stubs.md` itself is the live inventory and is untouched by this move.**

- **Stub inventory filed** (`docs/design/stubs.md`, 2026-07-21, read-only audit
  agent + integrator). Ten active stubs, each with its heir. One genuine
  discovery: **ruin-posts was UNDOCUMENTED** — the only world-visible
  substitution with no placeholder marker anywhere; ~~a loud code comment is
  owed at `collapse.rs::ruin_posts`~~ **— ✅ VOID 2026-07-29: `ruin_posts` is DELETED**
  (S6 audit § 2 #3). `grep -rn "ruin_posts" --include=*.rs crates/` returns **zero hits**;
  `docs/design/stubs.md` #1 reads *"ruin-posts — RESOLVED BY DELETION 2026-07-28 (journal/0121); no
  heir was ever built and none is owed"*. The entry's one genuine discovery was an owed comment on
  a function that no longer exists. *(Note: `stubs.md:64-65` records explicitly that this deletion
  is **not** evidence for the ecology clause in CLAUDE.md — do not cite it that way.)* Two audit additions to the decision's holdout list:
  igneous emplacement-depth constants and paleo-temp-is-present-day. Also
  flagged: the `field.rs` doc-comment claims the collapse tier "reads"
  exhum/t_crust when nothing does — do not trust it.

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Both of its filed proposals shipped as journal/0031** -- crease/silhouette edges under a separate `--edges` flag (`crates/dc-client/src/edgepass.rs`, `shaders/edges.wgsl`), and fullbright no longer applying distance fog. Its (b) -- sun determinism -- was confirmed in the entry itself. **The doctrine it earned is read-first material in CLAUDE.md § Agent walks** (*"Pick the control that can SEE your question"*), which is the durable form of this finding.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Same finding, filed twice on 2026-07-20. Its *"proposed fix, filed not built"* is `--edges`, shipped journal/0031, and the lit-for-shape / fullbright-for-material rule is in CLAUDE.md.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Fixed by journal/0031; CLAUDE.md § Agent walks states it -- *"Fullbright also no longer applies distance fog (0031), so long-vista silhouettes are readable."*

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

**Answered, and the answer is *deliberately not built*.** The live § Observed entry `Mesh-buffer pooling: measured, deliberately NOT built (2026-07-21, journal/0051 -- the user asked for pooling; this is the numbered answer)` is the reply to this exact user question, and it is decisive on the mechanism: Bevy's `Mesh::insert_attribute` takes ownership, so pooled scratch buffers would have to be copied in -- zero copies per attribute becomes one. **That entry stays live**, including its surviving residual idea (reuse `MeshData`'s buffers *inside* `mesh_chunk`). The two entries never referenced each other.

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

Its subject -- the S1 phantom old world visible below the real terrain -- is gone under the worldgen authority (journal/0022), as the far-mesh entry says in its own text: *"the phantom old world ~1 km down is gone and there is a horizon."* **That entry stays live** for its unrelated open half, the `TerrainGen` seal (`✅ VERIFIED STILL OPEN 2026-07-29`).

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

**ARCHIVED 2026-07-29 — resolved by later work this entry did not know about** (moved by status, not by age; reproduced verbatim).

The suspect it sharpened to -- `LOAD_BUDGET_PER_FRAME = 8`, generated synchronously on the main schedule -- was addressed by the 0083/0084 offload. **Its outcome is the live § Observed entry `Perf: throughput ceiling at terminal velocity`**: *"the drop reaches the choking point later ... but at terminal velocity it still chokes, about as hard."* Two entries, one thread, no cross-reference. The ceiling, not the onset, is the live question.

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

### Closed sub-threads lifted out of entries that are STILL LIVE

**Every passage below was cut from an entry that remains on the live board.**
The parent entry's head, its open questions and its reasoning stayed; what moved
is a sub-thread the entry itself had already marked shipped, struck or
superseded. Each banner names the parent so the cut is walkable from this end.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** ALL FIVE USER DECISIONS RULED 2026-07-28 — journal/0120.**

**The five rulings and their reasoning.** The entry's own head says *"Nothing here is owed"*; all five carry `✅ RULED` and each shipped its record elsewhere (CLAUDE.md read-first item 5, `worldgen.md`'s ON-HOLD banner, `north-star.md` § Materials, `spines.md` § 3's third exit, banners on `S10-results.md` and `S2-results.md`). **The live board keeps the head and the *Also surfaced, NOT user-owned* tail**, which still holds two open obligations.

  *Original framing, kept: five calls surfaced 2026-07-28, NONE of them the integrator's*
  (user-directed the same day: *"keep the user owed part durable with pointer to its necessary
  context… I won't muddy waters by addressing that in this conversation"*). **Deliberately parked
  by the user, not forgotten.** Consolidated into ONE live entry because the alternative — leaving
  them inside a `✅ DONE` block and a notebook — is the buried-in-a-closed-artifact failure this
  session spent the day measuring. **Context is inline, not merely pointed at**, so this entry is
  actionable cold.
  **Sources:** `journal/0121` (the removal's narrative) · `journal/corrections.md` **#66** ·
  [`docs/design/corpus-knowledge-notebook.md`](docs/design/corpus-knowledge-notebook.md) §§ 4–5 ·
  [`corpus-knowledge-evidence.md`](docs/design/corpus-knowledge-evidence.md) §§ 3.6g/3.6j/3.6k ·
  the `✅ DONE` removal entry below, which points here.

  1. **APPEARANCE — the shipped world lost 102 wood posts at four sites** (seed 1337,
     `Extent::Medium`). Verified gone: the 12 chunks that carried every post read **102
     `Block::Wood` → 0**, and a 200-chunk box around all four former clusters reads **0**.
     **No before/after screenshot pair is possible** — the "after" is ordinary ground — so this is
     a **notification for the record**, not a walk. Nothing else visible changed and **no golden
     moved** (corrections #66). *Why it is yours: an appearance change is ratified by the user's
     eye, and removal from what a player could walk into is an appearance change even when there
     is nothing to photograph.*
     - **✅ RULED — ACCEPTED (user, 2026-07-28).** *"That's fine and it's what I wanted. That
       system was not designed — it will have a designed successor at one point, but I had
       nothing to do with it. Came entirely from Claude bootstrapping the project, attempting
       to satisfy the list of things I mentioned that I would like to be in it **eventually**."*
     - **⚠ AND THE FACT THAT CLOSES IT, from the user:** *"I never once saw a wooden post in
       the world and no images were captured — it predates journals and our walk protocols."*
       **So the missing before/after pair was never a limitation of the removal.** The content
       was **never observed by the one eye that ratifies appearance**, across its entire life.
       *This is the sharpest available statement of "existence is not standing": a thing can
       render in the shipped world for a week, be defended by three independent instruments,
       and still have no witness. Appearance content that no one has ever seen has not
       accrued standing by surviving — it has only accrued inertia.*

  2. **DESIGN-DOC RULING — three live docs still describe civ/history as a real pipeline stage,
     contradicting the 2026-07-26 ruling.**
     - `docs/design/worldgen.md` § *"Below the region scale"* item 4: *"**History** — peoples,
       polities, trade, wars, migrations: dc-sim's coarse tier run over pre-player millennia."*
     - `docs/design/things-that-will-happen.md:126` — the sword looted from a ruin.
     - `docs/design/ideas.md:384`.
     Against the user's *"we do NOT have any form of evo/socia/civ modeling **even at the design
     stage**: they are NOTHING."* **The removal slice deliberately did not touch any of them** and
     was right not to: it could not distinguish a ratified user design from bootstrap text, and
     editing a design doc from inside an implementation slice is **corrections #65's exact failure
     mode**. *Why it is yours: which side of a user-vs-user contradiction wins is the one thing a
     sweeper is forbidden to decide (`doc-topology` § Rules).*
     - **✅ RULED — SPLIT THE THREE, and the split is a doctrine, not a tidy (user, 2026-07-28).**
       - **`worldgen.md` → ON HOLD, not struck.** *"History is coming, eventually, for the
         reasons worldgen states (not exhaustive)."* Shipped: a top banner, six marked sites,
         and a new § *Sequencing*. **The doc had understated the problem** — history is that
         document's *thesis*, not one bullet: its title, its core decision, its extent knob
         and its borders contrast all rest on it. **And one leg of the boundedness argument
         is the history requirement**, so § *The core decision* now records that **closure**
         is what carries boundedness today (the other leg is pure earth science and is live).
         Left unmarked, that is a ratified decision visibly resting on a suspended premise —
         A-2 waiting to be "discovered".
       - **`things-that-will-happen.md` → UNTOUCHED, and the doc's standing recorded.** *"Things
         that will happen is correctly 'what kind of engine this WILL BE and what kind of
         experience the default pack WILL BE'. The ambitions there are recorded with a high
         amount of user involvement and are **not claims about what we currently have built**
         as content or can support as an engine."* The looted-sword line **stays**.
       - **`ideas.md:384` → UNTOUCHED.** A mood line in a bullet about visual language.
       - **THE GENERAL RULE, now in CLAUDE.md:** the doctrine governs **unratified bootstrap
         CONTENT**, never **RECORDED AMBITION**. *The tell: does it RUN, or does it PROMISE?*
         Striking a future-tense user-authored line **retires a goal**, which no sweeper and no
         slice may do. **This is the second time in three days that this directive was about to
         over-reach by one word** — the first was `ecology.md`. Both were caught by the rule the
         directive itself sits next to.
       - **⚠ AND THE STATUS IS *ON HOLD*, NOT *NEVER*.** *"We do want these systems
         **eventually**: they are effectively on hold."* Read *"they are NOTHING"* as a claim
         about **what exists**, never about **what is wanted**.

  3. **SCOPE — dc-sim's entire S2 statistical tier now has ZERO production callers.**
     `pregen/history.rs` was its only one; removing it left `engine::{query, observe, force_fact}`,
     `Ledger` and `ToyWorld` reached by nothing but their own `s2_torture` / `s2_measurements`
     suites (new `spines.md` § 3 row). **The removal entry's scope line said *"dc-sim's
     region/agent-step draws"*, but those draws ARE `simulate_sample`** — so following the scope
     literally deletes `engine.rs` (528 lines), `world.rs`, both suites, and the artifacts
     `docs/spikes/S2-results.md` reports on. **The slice filed a DEVIATION PLEA instead**, which
     is the correct move. **The call:** keep the tier as a shape reference, or dispose of it as
     the same class of thing as the content it served? *(Note `S2-results.md` carries **no
     supersession marker at all** — the spike whose implementation just died has nothing on it.)*
     - **✅ RULED — KEEP BOTH THE PRIMITIVE AND THE TOY (user, 2026-07-28).** *"I didn't even
       know this system existed… The statistical system is genuinely intended, though I can't
       say whether as-is it will fit the desired shape when we actually do move on to
       implementing the civ/socia part of the default pack and the engine affordances."*
     - **The ruling's operative half is the NOTE, not the keep.** Whoever stumbles on this
       module — or is sent looking — must read: **the code that read it is gone · the primitive
       is the deliverable · it is a CANDIDATE to be re-checked against requirements that do not
       exist yet, never adopted on sight · and when that thread may open is a USER CALL.**
       Landed in all three places a reader actually arrives: the module doc
       (`dc-sim/src/statistical/mod.rs`), `S2-results.md`'s banner, and the `spines.md` § 3 row.
     - **`spines.md` § 3 gained a THIRD EXIT because of this.** The index knew *consumed* (the
       good exit) and, since journal/0121, *deleted*. This row is neither: **HELD AS A
       CANDIDATE** — ratified as wanted, with nothing yet to judge it against, so it is neither
       owed a consumer nor eligible for disposal. *Without the third state a reader assumes the
       first and goes hunting for a consumer nobody wants found.*
     - **`S2-results.md` now carries its banner** — which is also the first application of the
       decision-5 policy below.

  4. **SCHEMA — delete dc-sim's settlement/civ types, or keep them?**
     `Subject::{Site, Polity}` · `Aspect::{SiteExists, SitePolity, SiteEvent, PolityExtent}` ·
     `SiteEventKind` · `Value::{Exists, PolityRef, Event, Extent}`. **Producer-less since the
     removal.** Left in place and marked in-code as *not a schema to build on*, because deleting
     variants of a `Serialize` enum is wider than a content removal's scope. *Why it is yours:
     same doctrine as item 3 — unratified bootstrap schema has no standing, but the disposal is a
     scope fork.*
     - **✅ RULED — KEEP, with item 3 (user, 2026-07-28).** It rides the same ruling: the tier
       stays as a candidate, and its vocabulary stays with it. **But the in-code marking is what
       carries the standing** — it is **producer-less example vocabulary, NOT a schema to build
       on**, kept because deleting `Serialize` variants exceeded the removal's scope, *not*
       because anyone ratified it as a design. Re-stated in the module doc and the § 3 row so
       the distinction survives without this ROADMAP entry.

  5. **POLICY COLLISION — is a spike-results doc immutable testimony, or live authority?**
     `corrections #12` states the policy: *"`S10-results.md` is **left unamended** — a spike result
     is a dated record of what was measured; **this entry is the pointer**."* `CLAUDE.md`
     read-first item 5 states the opposite: *"Spike results live in `docs/spikes/S*-results.md` —
     **measured numbers, don't re-guess them**."* **Both are reasonable, they are incompatible,
     they live in different files, and nothing has ever reconciled them.** The concrete cost is on
     record: `S10`'s cost table is **~2× the real production cost** (25.19 s claimed vs 13.79 s
     measured — the spike drove the *scalar* path, production takes the *parallel* one), **a user
     ratified a ship decision on it**, and `S10` holds no reference to its own correction.
     Measured corpus-wide: **8 of 15** full-path correction→file edges are one-directional, and
     **14 of 30** audit/spike files carry no staleness marker of any kind. *Why it is yours: this
     is a choice between two ratified-feeling policies, and it changes what read-first means.*
     - **✅ RULED — IMMUTABLE BODY, MUTABLE HEADER (user, 2026-07-28).** A spike's measurements
       are **never rewritten** — testimony about a day is not edited — but a results doc **must
       carry a top-of-file banner pointing at whatever refuted, superseded or re-scoped it.**
       Recorded in **CLAUDE.md read-first item 5** (the site that asserted the losing half) and
       as a supersession note on **corrections #12** (the site that asserted the other half).
       Neither is rewritten; both now agree.
     - **The obligation lands on the WRITER OF THE CORRECTION, in the same commit.** *That is
       the property doing the work, and it is chosen from measurement rather than taste: a
       convention survives when it is inseparable from an act the author must perform anyway,
       and dies when it asks them to restate something in a second notation. Stamping the target
       happens while both files are already open. The counter-example is on the record —
       `JUSTIFIED-BY`, documented in two places with a promised sweep, got **3 uses, 0 in
       `crates/`**.*
     - **Applied where a refutation is already known:** `S10-results.md` (→ corrections #12, the
       ~2× cost table a **user ratified a ship decision on**) and `S2-results.md` (→ the item-3
       ruling). **Not a sweep** — see the backlog entry below.
     - **What it actually fixes is structural:** *a one-directional pointer is not a pointer.*
       The stale end is exactly where a cold session enters, and a chain of authority cannot be
       walked from an end that holds no link.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** THE BASELINE SWEEP'S FINDINGS

**The four ruled user calls, and the five `HIGHEST BLAST RADIUS` items -- all five verified applied at source on 2026-07-29:** `spines.md` § S-6 rewritten to the ratified authored-order shape · the erosion-axis losing-side banner · `tectonics.md`'s supersession banner · the seam count corrected to *"31 LIVE, of 34 inventoried"* · ROADMAP's first bullet struck (*"THE TIERING IS RETIRED"*). **The live board keeps the head and the STRUCTURAL / BULK / `flow_cost_probe` / NOT-COVERED-BY-THIS-BASELINE bullets** -- `flow_cost_probe` is still the one probe the gate cannot see fail.

  - **✅ ALL FOUR USER CALLS RULED 2026-07-28** (unpacking session; applied to
    `ores.md`, `worldgen.md` § Sequencing, `ecology.md`, `stubs.md`):
    1. **BIOLOGY — the two decisions were never in conflict; the missing word was *building*.**
       The deep-time biotic layer stays **ON** and rides as-built — *"biology in this sense is
       seamed with heir"* — but *"**we aren't building bio-based rock formation any longer**
       until bio/eco stuff, which is waiting on the rest of the non-bio earth science stuff +
       engine capabilities."* **No new bio-driven rock-formation work opens before the gate.**
    2. **🔴 ORES — A LOAD-BEARING PREMISE IS REJECTED, and this is the biggest of the four.**
       *"We do not need to have ore 'exposed' — the default plugin pack will ship a voxel game
       **with digging**… absolutely no reason to treat it like everything needs to be
       discoverable on the surface. Weird and misconceived and likely very relatively old."*
       **Kills every "illegible until exhumation increases" caveat, the lode-gold A/B fork, and
       `probe 3` — the probe measured the wrong thing and is NOT owed.** *Exhumation stays real
       for **genesis honesty** (where an ore forms); what dies is exposure as a precondition for
       shipping one.* `ores.md` is **conceptually behind `materials.md` / `material-behavior.md`,
       which win on disagreement**; a revisit is owed and unscheduled. **⚠ Note how it survived:
       the assumption was never stated as a decision — it rode inside *measurement caveats*,
       which read as evidence rather than as premises, and held a user fork shut for a week.**
    3. **`material_transport` — RATIFIED as-is.** The user is already running a string of work
       on it and confirmed `COMPETENCE_SCALE`'s *"mud, sometimes"* is **not** to be treated as a
       knob to tune (it is downstream of the denudation rate; tuning it would be a number
       pretending to be a mechanism).
    4. **THERE IS NO "GENERAL REGISTRY" AND THERE NEVER WAS — the clause is DELETED, not
       reworded.** Surfaced by the user asking *"I honestly don't understand what the registry is
       supposed to be except for a list which we can extend."* **Correct — and nobody ever
       proposed one.** It was an inference that implied future work.
       - **A seam's success condition is that it DISAPPEARS.** The heir *replaces* the slot; it
         does not fill it forever. **Only completed case:** `burial_temp_c`'s heir turned out to
         be a **field**, so it retired as a **field pass** and left the file (journal/0093) —
         *"the answer was 'this is not a provider at all — it is a field.'"* `depth_to_water` is
         documented as heading the same way. **You do not design a third-party declaration for a
         pattern whose job is to vanish.**
       - **⚠ THE ROOT CAUSE — "slot" means two unrelated things**, sharing a code shape
         (`Option<fn>` + identity) and nothing else: **provider seams** (world-level, scaffolding,
         *temporary*) vs **material behavior slots** (`north-star.md` § Materials — what a
         content author writes, **the SDK surface, permanent**). north-star called the latter
         *"the `Providers` pattern **generalized** from world-level to material-level"* — true of
         the shape, **and read as the world-level SYSTEM being promoted into the SDK.** That
         misreading produced the phantom. **north-star now disambiguates it in place.**
       - **Where world-level seams land post-split is UNDISCUSSED and deliberately UNDECIDED**
         (user: *"I genuinely don't know… I don't think anyone has had a direct conversation about
         it… I don't want to burden us with more half-baked designs"*). **No decision is owed.**
       - *The assistant proposed a "policy injection" counter-argument and it is **dropped, not
         recorded** — neither party could name an instance, and writing down a hypothetical that
         shapes future thinking is the thing being avoided.*
       - **⚠ The integrator's OWN first two fixes of this were also wrong**, both from the same
         ambiguous north-star sentence (*"plugin-authorable engine sockets"*). Left visible at
         `doc-topology/SKILL.md` because being wrong twice from one sentence is the argument for
         disambiguating it. **Propagated to all six citation sites** — `stubs.md`,
         `providers/mod.rs`, `north-star.md`, `doc-topology/SKILL.md`,
         `corpus-knowledge-notebook.md`, `corpus-knowledge-evidence.md`. Journals **0060** and
         **0120** quote the old clause and are **left untouched: dated testimony, immutable body.**

  - *Original framing of the four calls, kept for the reasoning:*
    1. **`ecology.md:262-263` (DECIDED 07-20, user) vs `worldgen.md:189-191` (DECIDED 07-28,
       user)** — biology is *"a shipped part of world generation"* (confirmed live,
       `deeptime/field.rs:288`) vs *"engine + non-bio earth science → **then** ecology"*.
       **Both yours, eight days apart, reconciled nowhere.** Likely resolution: the S10
       deep-time biotic pass ≠ the ecology *design* pass — **but that sentence is written in
       neither doc, and it is not the assistant's to write** (corrections #65).
    2. **`ores.md`'s lode-gold NEEDS-RATIFICATION fork is held shut by an expired caveat** —
       *"until the erosion-supply calibration lands"*; **it landed 2026-07-26** (corrections
       #56, journal/0114). *A user decision has been available for two days and the doc says
       it is blocked.*
    3. **`material_transport: true` is the shipped default with no ratification record found**
       — not in the archive, the close blocks, or journals 0110–0112, while the board flags it
       `NEEDS RATIFICATION (user-owned)`.
    4. **`stubs.md:20-24` says "four conversions is not enough to design a registry from"; a
       fifth landed** (`:196-201`, journal/0078). That clause is quoted as binding doctrine in
       three places **including the argument for not designing the knowledge layer yet**, so
       whether five changes the judgement is a user call.
  - **🔴 HIGHEST BLAST RADIUS, integrator-applicable:**
    - **`spines.md` § S-6 still teaches order-derived-by-topo-sort as the exemplary compliant
      shape** across 148 lines, and does **not mention the 2026-07-26 authored-order decision
      anywhere** — no strike, no banner, no § 4 entry (verified by pathspec). **Read-first item
      0b; every brief that "names its shapes" has been naming a retired one.** *corrections
      #65's geometry, recurring inside the index built to prevent it.*
    - **The erosion axis is marked settled and is not.** `ROADMAP.md:2513-2533`, live
      `NEEDS RATIFICATION`, no banner: *"nothing further to ratify on the erosion axis."* Its
      null came from a probe **blind to `diffusion`** — 96 % of export (journal/0111:256-260,
      corrections #56) — so **the methodology is defective regardless of calibration**, and
      journal/0114 measures relief **+18 % at 100×**. ⚠ **But do NOT restate it as "the
      landscape is supply-limited today"**: that holds for the *calibrated* world, and
      `calibrated_rates` ships **OFF** (`walk_tour_0115.rs:150` asserts it). *Two agents each
      had half of this; the split matters.*
    - **`tectonics.md` (953 lines, largest design doc) carries NO staleness banner** and its
      § 7.3 still specifies the retired receiver tree as *"the carving source, one authority"*.
    - **`spines.md`'s "34 seams inventoried; 5 converted" is wrong — the inventory says 31**,
      and that figure is promoted as *the only obligation ledger with a denominator* in the
      docs-ops argument.
    - **ROADMAP's FIRST BULLET (`:33-38`) asserts the trusted/untrusted backend tiering** that
      north-star § Deviations 2 calls *"EXPLICITLY NOT THE MODEL"*.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** DOC-TOPOLOGY RESIDUALS — the 19 findings not actioned 2026-07-26

Three consecutive `✅ DONE 2026-07-28` sub-bullets: the 44 dangling pointers at 43 sites, `flow.md:715`'s RATIFIED stamp, and the `spines.md`/`stubs.md` scheduling heirs. **The live board keeps the residuals** -- the ABI-spike contest, the coal evidence base (which needs a measurement, not an edit), and the next sweep's spine.

  - **✅ DONE 2026-07-28 — the dangling cross-file pointers.** The audit estimated *"~25"*;
    the real count is **44 pointers at 43 sites**, all fixed. **The audit undercounted by 18.**
    - **23 in `ROADMAP.md` pointing at the archive** (audit said 16): 22 of the
      `"see § Shipped"` form, plus the last line of the 2026-07-26 morning close block —
      *"the 2026-07-25 block **below**"*, which was one of the six archived, so the live
      board's own close block pointed past its own end.
    - **19 in `ROADMAP-history.md` pointing back at the live board** (audit said 9).
    - **4 were already directionally wrong BEFORE the split** and are now doubly wrong: the
      roughness-decay entry's *"Sequenced below"* read from inside § Observed, and the three
      `NEEDS RATIFICATION (below, § Sequenced)` markers on the 2026-07-20 erodibility / water
      / biotic Shipped entries. *§ Sequenced has sat **above** § Shipped since at least
      `34d88f2`; these predate the archive, which merely made them unresolvable.*
    - **The `:515` orphan — *attributed*, not guessed.** The block it meant was the **2026-07-21
      close block** (verified at `34d88f2`, where it sat 19 lines above), and that block was
      consumed by its successors rather than archived — so it exists nowhere, and the line now
      says so instead of pointing at a block 3,800 lines the other way.
    - **The retargets are addresses only.** No claim was resolved and no contradiction
      adjudicated. Where a Shipped entry has a **journal number** it is kept as the stable
      pointer, per the archive's own design.
  - **✅ DONE 2026-07-28 — `flow.md:715`'s RATIFIED stamp.** The stamp is legitimate for
    § 11's WINDOW decision (DECIDED 2026-07-25, user); what wrongly inherited it was the
    *setup sentence* characterising the other two axes. The ORDER half is now **struck and
    marked SUPERSEDED 2026-07-26**, in the shape `material-behavior.md` § 5's ORDER bullet
    already uses, with a note on why § 11.1's argument survives the strike (it needs only
    that the scheduler had **no name for the window**, which holds either way).
    *Still open from the same finding (#15), deliberately not taken here:* `flow.md:459` and
    `material-behavior.md:367-369` carry the same `ORDER (topo-sort)` framing **without** a
    ratification stamp.
  - **✅ DONE — `spines.md` + `stubs.md` scheduling heirs for the removed bootstrap content.**
    Both resolved *into* the removal rather than surviving it, as this finding asked.
    `stubs.md` § 1 is now **RESOLVED BY DELETION** (journal/0121) and — the more useful half —
    was rewritten to carry the doctrine it cost: *a stub entry is not neutral about its
    subject's standing, it **asserts** it; ask the standing question before writing an heir.*
    `spines.md` § 3's row was replaced by the S2-tier row and, on 2026-07-28, given the user's
    **held-as-candidate** ruling plus the § 3 intro's **third exit**. *Note the shape: the
    finding asked for two deletions and what shipped was two doctrine changes — the heir lines
    were correct on their own terms, and the defect was the inventory's grammar, not the rows.*

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** THE PASS ARCHITECTURE — AUTHORED ORDER, OPEN VOCABULARY

**The pre-ship framing of the RATE slice**, which shipped 2026-07-29 as journal/0123. Its own opening says it is a *"record of how the slice was framed **before it shipped**"*. **The arc itself is LIVE and stays** -- (1) authored ORDER and (2) the open vocabulary are owed, the continuation slot (b)-(f) is open, and the `✅ RATE IS NOT EXPANDED` user ruling that follows this range stays on the live board because it is a standing constraint on the S-10 spine.

    *Record of how the slice was framed before it shipped, kept because the sequencing argument
    is the reusable part:* ~~with the creep limiter as its acceptance test~~ — **the acceptance
    test was met on 2026-07-29 by a stand-in, and that changed the slice for the better.**
    - **What happened.** journal/0122 fixed the creep blocker by **sub-cycling inside the
      pass**: `n = ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)`, derived from the von
      Neumann bound `a = 1/8`. Grid-scale oscillation is gone (surface concavity ACF(1)
      **−0.867 → +0.185** calibrated; closed hollows past 10 m **818 → 12**; five safe
      multipliers where journal/0114 found none). **The acceptance criterion above is
      discharged — by a `dt` the pass computed for itself because the engine has none.**
    - **So the slice is now sharper, not gone: replace the hand-rolled sub-cycle with the
      engine axis, and hold the goldens.** This is strictly better than the original framing
      — the axis now lands against a **known-good fixed point** instead of against a defect,
      so "did RATE reproduce it" is a hash comparison rather than a judgement call.
      `stubs.md` § 30 carries the stand-in with RATE named as its heir.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** REQUIRED CHORE — FILE SIZE IS A CORRECTNESS PROBLEM

`✅ CONVENTIONS DECIDED`, and **the entry itself names where the record now lives**: *"shipped in `scripts/filesize_hook.py`, whose module docstring is now the record (the hook is the only mechanically-enforced corpus control, so the convention lives where it is enforced)"*. **The live board keeps the chore's open half** -- immediate for new files, gradual refactor of old work when touched. *This archiving pass is that chore, executed on its largest subject.*

  - **✅ CONVENTIONS DECIDED 2026-07-28 (user)** — was *"to be set, not guessed."* Set from the
    corpus measurement rather than taste, and **shipped in `scripts/filesize_hook.py`**, whose
    module docstring is now the record (the hook is the only mechanically-enforced corpus
    control, so the convention lives where it is enforced).
    - **THE SPLIT AXIS IS LIVENESS, NEVER TOPIC.** Every split moves out the **cold half** —
      still true, still cited, no longer read to do today's work. **Both real conversions
      already did this and neither was by topic:** `ROADMAP → ROADMAP-history` split by
      **status**, `notebook → evidence` split by **read pattern**. The convention is the axis
      those two taught, not a new invention — which is what `stubs.md:22` requires.
    - **WHY TOPIC-SPLITTING IS DISALLOWED, and it is the non-obvious half.** Contradiction here
      is produced by **addition** (design docs delete 2–4 % of what they add), and every
      expensive failure was a claim sitting near its own refutation — two sentences apart
      (#65), forty lines (#58), two subsections (#53), 400 lines (journal/0119). **Topic-
      splitting a live doc converts an in-file contradiction into a cross-file one**, reachable
      only by the `doc-topology` sweep — five of whose top eight findings were unsuspected by
      construction. **That trades VOLUME (third in value) for TOPOLOGY (the one that cost an
      architecture).** *Stated at honest strength: co-location did not prevent those
      contradictions — access was never the problem. The claim is the weaker, sufficient one:
      topic-splitting costs the one condition under which a reader could notice and buys only
      line count. Liveness-splitting cannot do this, because the cold half has stopped
      accreting.*
    - **THE THRESHOLD APPLIES TO THE HOT FILE ONLY** — archives and evidence files are exempt
      by designation. *The old hook flagged `ROADMAP-history.md` for being exactly what it was
      built to be: crying wolf on a file doing its job, with no correct action available.*
    - **THREE CLASSES OF `.md`, by READ PATTERN** — **NARRATIVE** (journals, audits, spikes:
      written once, read whole, never revised) **exempt**, and splitting one is *harmful*
      (measured: 120 entries, median **177** lines, max 536) · **REGISTRY** (`ROADMAP`,
      `corrections`, `spines`, `stubs`: looked up by ordinal, not read) **2,500**, split =
      **archive resolved entries** · **ARGUMENT** (`docs/design/*`, skills: read in sections,
      actively revised) **1,000** — the only class where the threshold bites and the only class
      where topology failures happen.
    - **Verified on the real corpus:** flags `ROADMAP` (5,120) and `corrections` (2,681) as
      registries with a real action, and `material-behavior` (1,036) as an argument doc; silent
      on `ROADMAP-history`, the evidence file, every journal entry, every spike, and `spines`
      (1,283, under the registry bar). **Signal went from "everything large" to three files with
      a correct move each.**
    - **DELIBERATELY NOT BUILT:** no taxonomy registry, no frontmatter marking file class, no
      validator. **Two conversions is below this project's own bar** (`stubs.md:22`;
      session-workflow § Seam-first #6). Class is derived from path, which suffices until the
      next two or three splits teach more.
    - **⚠ HONEST LIMIT, recorded so this is not oversold:** volume is the **third** most
      valuable of the three docs-ops failures and the archive *"would not have prevented
      journal/0119."* **This buys agent context efficiency; it is not a correctness fix.** The
      open **corpus-addressability** thread may subsume part of it — *a file addressable by
      section may not need to be small* — so the ARGUMENT threshold is the negotiable number if
      that lands.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** Finish the draw-domain conversion: the residual hand-rolled sites

**The 2026-07-26 consumer trace, which the entry itself labels a dated record** (*"Read the trace below as the 2026-07-26 record it is"*). Every consumer it traces -- `pregen/history.rs`, `Pregen.sites`, `collapse.rs::ruin_posts`, the `Block::Wood` emission -- was deleted 2026-07-28 (journal/0121). **The live board keeps (a), (b) and (c)**, which are now byte-identical housekeeping.

    (corrections #64, a read-only trace taken
    2026-07-26 during (b); **not re-scoped by that agent — the integrator's and the user's call**):
    - **The region-step draw (`engine.rs:331`) is the real one, and it IS visible.** Collapsed
      pressure → the sack roll (`pregen/history.rs:221`) → `abandoned` → `Pregen.sites` →
      `collapse.rs:1588`'s ruin posts → `Block::Wood` in `generate_chunk` (`collapse.rs:483-488`)
      → meshed (`dc-client/src/meshing.rs:214`). **No flag anywhere** — the history pass is an
      unconditional `vanilla_passes()` member (`pipeline.rs:359-366`).
    - **The agent-step draw (`engine.rs:355`) re-rolls NOTHING.** The pregen overlay is built with
      an empty agent roster (`history.rs:82`/`:84-88` pass `vec![]` as `with_graph`'s `agent_home`),
      so the loop never executes outside dc-sim's own tests. **That half is byte-identical
      housekeeping and could ride with anything.**
    - **"Polities" is not re-rolled.** The count is fixed at epoch 0 (`history.rs:134-149`);
      `PolityExtent` facts move but live only in `Pregen.ledger`, which **nothing in production
      reads** (`pregen/mod.rs:300-304`; sole non-test reader `approx_resident_bytes`,
      `mod.rs:367`) — a spines § 3 "built, and nothing calls it" cluster.
    - **Goldens that would move:** `contents_contract.rs:70-86`, `s7_walk.rs:32-40`,
      `geology.rs:32` (all hash `generate_chunk` blocks, structurally downstream of the posts),
      and `s7_handoff.rs:118`, which pins a **seed-specific sack** and is the likeliest break.
      `GOLDEN_SURFACE` / `GOLDEN_RECORD` are **not** downstream (deep-time field only).

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** FLOW IS ONE PROCESS — flux on FACES, facts on STRATA

Already struck in place on 2026-07-29 (baseline sweep S5/F4): *"✅ SHIPPED 2026-07-26 -- both terms are stale."* Hybrid-`p` is the shipped default and the `k_bedrock`/`k_transport` recalibration rode the joint calibration. **The FLOW arc stays live** -- (b'), (c), (d), (e) are unbuilt and its three riders are still in the slot.

  - ~~**🔴 OWED BY MFD, and it is the nearest-term thing on this arc (2026-07-26, journal/0109):**~~
    **✅ SHIPPED 2026-07-26 — struck 2026-07-29 (baseline sweep S5/F4). Both terms are stale.**
    **Hybrid `p` shipped** as FLOW (b′) (`journal/0113`, `ROADMAP-history.md`; peak catchment
    84 → 298) and is **the shipped default** — `crates/dc-worldgen/src/deeptime/grid.rs` sets
    `mfd_exponent: 1.0` / `mfd_exponent_channel: 16.0` with `mfd_chi_lo`/`mfd_chi_hi`,
    doc-commented *"the hybrid-`p` law (journal/0113)"*. **The `k_bedrock`/`k_transport`
    recalibration** is at least partly discharged by the joint calibration
    (`journal/0114`, `EROSION_CALIBRATION = 45` — which this same entry acknowledges elsewhere),
    though that ships **behind `calibrated_rates`, OFF**. *This document cites hybrid `p` as a
    **completed predecessor** two hundred lines above while flagging it 🔴 OWED here.*
    Original text, kept as the record of what was owed:
    ~~**hybrid `p`**, plus **recalibrating `k_bedrock`/`k_transport`**.~~ Uniform `p` **does not
    concentrate flow** — peak catchment fell **1,245 → 84 cells** — because MFD lowers both `Q`
    and `S` at every cell, making it **systematically less erosive than D8 at fixed coefficients**
    (total load −16 % while load-carrying faces ×2.76). The literature's answer is `p` as a
    function of area/slope, or single-receiver above a channel threshold. **This gates whether the
    world ever SHOWS what the record now holds** — today the record carries 7.5 M simultaneous
    divergences and the viewport carries 2.7 cm. *Uniform `p` was the simplest correct thing and
    is the wrong long-run shape; that is the shipping agent's own verdict, not a later critique.*

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** THE HONEST IDENTITY SURFACE — retire the stored `Block` summary

**Three consecutive blocks the entry itself marks superseded** -- `SUPERSEDED SAME DAY`, `DISSOLVED 2026-07-25 with the tiers themselves`, and `PRESERVED FOR THE RENDER-SIDE QUERY` -- all retired by the user's UNTIERED decision stated fifty lines above them (*"if this is a world query then why tiered at all when we can inspect chunk and read the voxel?"*). **The arc stays live**: its continuation slot (storage/wire migration, the runtime edit-fact overlay, far-span `Block`→material, legacy-S1 retire) is unbuilt.

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
  - **~~THE FAR TIER'S PAYLOAD IS A MIXTURE, NOT A WINNER~~ — PRESERVED FOR THE RENDER-SIDE QUERY,
    like its two siblings above** (folded 2026-07-25, sweep row D-4). *This bullet is written in the
    tier language the **UNTIERED** decision retired the same day, and its surviving content is
    already stated up in that decision (**"payload is uniformly a mixture… there is no far tier left
    to special-case"**). What is **unique** to it and still live: the **render-side speckle
    direction**, and the cross-reference to the journal/0091 **LOD fix (b)** cold/warm material
    agreement, both in its last two sentences.* **DECIDED 2026-07-25 (user):** *"LOD may
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

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** § Sequenced (un-bulleted prose)

Struck in place: *"RATIFIED AND DONE 2026-07-20 (user: 'flip it, i want to see')"*. **The live board keeps items 2 and 3, the 2026-07-29 losing-side banner, and journal/0079's record** -- item 2 is an explicit open user call (*"whether the axis reopens, and on what terms, is the USER'S call"*).

1. ~~**Flipping `production_config`'s `erodibility` to true**~~ **RATIFIED AND
   DONE 2026-07-20** (user: "flip it, i want to see"; journal/0030). It is on;
   every new world has a different shape. The appearance answer came back
   **negative** — the ground-level surface is measurably re-terraced, but the
   predicted ledges/benches at hard beds and the resistant basement core are real
   in the data and **absent on screen** at the shipped amplitude (summit
   silhouette identical; 5 km lattice ±2 m; the ground dropped one voxel). Which
   makes item 2 below the live question, not a footnote.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** § Sequenced (un-bulleted prose)

**Self-labelled *"(for the record)"***, describing a gap closed 2026-07-20: `erosion.rs` incising bedrock with a single global `k_bedrock`. Erosion has been lithology-aware since journal/0029.

Original charter (for the record):
The gap was `erosion.rs` incising bedrock with a single global `k_bedrock` —
granite and mudstone eroded identically, so the world had no differential
erosion anywhere. The data already existed (the recorder knew the exposed unit
per cell per epoch); erosion never asked. Design notes honoured: agent-specific
resistance (NOT a scalar — the limestone cliffs-vs-caves trap); composition order
with `resist`/`wmult` stated; basement/shield prediction checked and holds;
feedback stability clamped; off-by-default and byte-identical.

**ARCHIVED 2026-07-29 — a closed sub-thread of an entry that is STILL LIVE on the board.** Only this passage moved; its parent entry stays in `ROADMAP.md` (reproduced verbatim).

**Parent entry (still live in `ROADMAP.md`):** § Sequenced (un-bulleted prose)

S10 ratification calls 1 and 2, both struck and done. **Call 1 carries a correction worth keeping**: the 25 s figure came from the spike harness's *scalar* driver and production takes the parallel path -- the real ritual is 13.79 s (corrections #12; `S10-results.md` now carries the banner). **Calls 3, 4 and 5 stay live** on the board.

1. ~~**The world-creation ritual grows 15 s → 25 s (+66 %)**~~ — **RATIFIED
   2026-07-20, and the number was pessimistic**: measured on the shipped path
   the ritual is **13.79 s** (biology's marginal cost +2.6 s, not +10 s). The
   25 s figure came from the spike harness's *scalar* driver; production takes
   S9b's byte-identical parallel path — **corrections #12**, journal/0026.
2. ~~**Flipping `production_config`'s `biotic` to true**~~ — **DONE**
   2026-07-20 (journal/0026). Biology runs in every new world; the `DeepField`
   changed for every world created from here.


### Struck by the user, 2026-07-29 EVENING — five entries (classification pass, docs/audits/2026-07-29-roadmap-classification.md)

The five STRIKE-CANDIDATE entries from the 2026-07-29 classification pass, each put to
the user with its evidence line and struck by ruling: *"strike all five."* Three are
from § In flight, two from § Observed. Entries verbatim; a struck report is an
observation the user has ruled no longer describes the world, not a resolved defect.

#### Ores R1-R8 (both halves: the numbered item and the parenthetical)

**STRUCK (user, 2026-07-29 evening).** The 2026-07-28 ruling (*"we do not need to have ore exposed"*) rejected a load-bearing premise, killing the lode-gold fork and probe 3. The surviving obligation is `ores.md`'s conceptual revisit, carried by `docs/dependency-graph.md` (owed, genuinely unscheduled) — not this entry.

*(**Ore design pass: DRAFT LANDED** 2026-07-21, `docs/design/ores.md` —
engineering pass over the DECIDED 2026-07-20 roster; R1–R7 awaiting the user.
Two collisions reported for the record: the ratified lode-gold flagship vs
S12's metre-scale exhumation (probe-conditioned in R1), and BIF vs the
Phanerozoic register (reconciled in R3). Placer source-blindness filed as
stubs.md § 11.)*


#### WALK THE NEW WORLD

**STRUCK (user, 2026-07-29 evening).** Overtaken: journal/0122 moved mean regolith 4.57 -> 3.91 m after these stations were mapped, and the pre-flip appearance hold was released by the user the same day (*"I do not care about my previous ratification on looks there"*). The *"revertible: one merge commit each"* claim was also false at strike time (~70 merges since).

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

#### UNRATIFIED APPEARANCE CHANGE (carry-H) + its tour map

**STRUCK (user, 2026-07-29 evening).** Same grounds as WALK THE NEW WORLD above: the station `H` values and the *"deeper on average rather than barer"* claim predate journal/0122's recalibration, and the hold itself was withdrawn by its own author. The units warning (corrections #28) stays live in corrections.md, not here.

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


#### Material placement rules are climate mocks

**STRUCK (user, 2026-07-29 evening).** Its mechanism is `surface_sample` picking Grass/Dirt/Stone, and the user's 2026-07-29 morning ruling on a sibling governs: *"grass and dirt are not generated in the current shape of the default plugin pack."* The two facts this entry also carried survive elsewhere: `exhum`/`t_crust` unconsumed = dependency-graph **P7**; no form-from-provenance rule = **P8** / stubs § 12.

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

#### Circulation is fidelity-correct but surface-invisible

**STRUCK (user, 2026-07-29 evening).** USER-OWNED appearance call, ruled. The `> blogworthy:` line (*"a climate the map can't see"*) travels with the entry, preserved below per the verbatim rule.

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



# Superseded close blocks


<!-- archived 2026-07-29 (second pass): four blocks moved verbatim -->

## NEXT SESSION — written at the 2026-07-28 EVENING close (SUPERSEDED by the block above)

**A `SessionStart` hook will already have told you which sweeps are DUE. Run them first** — that
is now the standing rule (user). Then read `corrections.md` **#68–#70**, then this block.

### The one sentence that matters
**The corpus got its first 100 % sweep and its first unprompted trigger — but the board's
*pickup* problem is untouched, and today's own work proves it: ~9 hours of shipped work had no
path into a cold session until this block was written.**

### Ratified (user's terms)
- **File-size conventions — SPLIT BY LIVENESS, NEVER BY TOPIC.** Move out the *cold* half; the
  threshold applies to the **hot** file. Three `.md` classes: **NARRATIVE** exempt (splitting a
  journal entry is harmful), **REGISTRY** 2,500 (split = archive resolved entries), **ARGUMENT**
  1,000. *Topic-splitting is disallowed because it trades the cheapest docs-ops failure (volume)
  for the most expensive (topology).*
- **Sweeps run FIRST THING**, incrementally from a watermark, FULL when the reference side moved.
- **Spike/audit docs: IMMUTABLE BODY, MUTABLE HEADER** — never rewrite a measurement; the
  **correction's author** stamps a banner on its target **in the same commit**.
- **Bio-based rock formation: no NEW work until the bio/eco gate.** The shipped biotic layer is a
  **seam with an heir** and rides as-built. Gate order: engine + all non-bio earth science in the
  ratified SDK-plugin shape → ecology → social. ***"Sufficiently complete" is a USER call*** — no
  checkable test, and **progress on earth science does not entitle anyone to open it.**
- **ORE DOES NOT NEED TO BE EXPOSED** — *"a voxel game **with digging**… no reason to treat it
  like everything needs to be discoverable on the surface."* Kills the lode-gold fork's blocker
  and `probe 3`. `ores.md` is **conceptually behind `materials.md`/`material-behavior.md`**.
- **`material_transport` ratified as-is**; `COMPETENCE_SCALE`'s *"mud, sometimes"* is **not** a
  knob to tune.
- **There is NO "general registry"** — deleted, not reworded. **A seam's success condition is that
  it DISAPPEARS.**
- **Refinement tier → candidate (c)**: engine owns primitives (field **and refinement** kernels)
  + runner; plugins declare all content. **The user flagged the list as non-exhaustive.**

### Falsified — the assistant's own first (#68–#70)
- **#68 — the recalibration rule I shipped and briefed was wrong within hours.** `calibrated_rates`
  is **OFF in production**; a recalibration nobody enabled voids nothing. **The hypothesis was in
  the brief going out, not in the report coming back.**
- **#69 — "the general registry" never existed**, and I then got the withdrawal wrong **twice**
  from the same ambiguous north-star sentence.
- **#70 — ore-exposure was an unstated premise riding inside measurement caveats.** ***A caveat is
  where an unexamined premise hides**, and none of the three sweeps look inside one.*
- **Four stalenesses I created during the session itself**, all caught by readers running against a
  frozen worktree. **The corpus goes stale from the inside, during the work.**

### First things next session
1. **`spine-audit` — the one sweep the baseline did NOT satisfy** (its watermark is `null` on
   purpose; all nine slices were docs-vs-docs). It has a finding already waiting: **`spines.md`
   § S-6 still teaches order-derived-by-topo-sort as the exemplary compliant shape**, with no
   strike anywhere — in read-first item 0b, against which every brief justifies itself.
2. **The erosion axis is marked settled and is not** — `ROADMAP:2513-2533`. Its null came from a
   probe **blind to `diffusion`** (96 % of export). ⚠ **Do NOT restate it as "supply-limited
   today"** — that holds only of the *calibrated* world, which does not ship. ~~**This is engine
   work and it is the highest-value thing on the board.**~~ **CORRECTED 2026-07-29 (user ruling;
   `journal/corrections.md` #71): erosion is DEFAULT-PLUGIN-PACK work, not engine work.**
   Read-first item 0 says it by name — *"every pass is content, including tectonics and
   erosion"* — and this close block contradicted it. It remains **the highest-value thing on
   the board**; only its side of the engine/content cut was wrong.
   - **✅ ITS BLOCKER IS GONE 2026-07-29 (journal/0122).** The hillslope operator that
     checkerboarded the regolith (stubs #29, the 🔴🔴🔴 item) is fixed: `diffuse` sub-cycles to
     the 1/8 monotonicity bound, the calibrated world's surface concavity rms goes **40.42 →
     0.30 m** and its ACF(1) **−0.867 → +0.185**, and closed hollows past 10 m go **818 → 12**.
     **What is now in front of the axis is a NUMBER, not a defect:** `EROSION_CALIBRATION = 45`
     was fitted against the capped operator and does not survive fixing it (45× now strips the
     world to 1.40 m of mean regolith). The ladder in journal/0122 is the input; **the
     multiplier is user-owned and appearance-class** and `calibrated_rates` still ships false.
     Also falsified in passing: `corrections.md` **#72** — journal/0116's "not a stability
     limit" was a 4× refinement against a register 100.8× away.
3. **The bulk-mechanical backlog**, if you want cheap wins: **37 Observed entries (29 %) already
   say ✅ DONE in their own bodies** — a pure archive job with no judgement calls; **8 spike/audit
   files need supersession banners** (drafted).

### 👁 OLDEST UNTOUCHED USER FIELD REPORTS — 8 days, and nothing schedules them
*New close-block line, 2026-07-28. **The sweeps keep the board accurate; nothing converts an
accurate open item into work.** Only the close block does — so it now carries the age of the
oldest thing the user personally saw and reported. **23 of 33 user field reports are open**;
these four are the oldest, all from **2026-07-20**:*
- *"Our dismal mountains"* (`ROADMAP:~4100`) — DIAGNOSED, four causes, unfixed.
- *"Thick units render as flawless monoliths"*
- *"Loose materials do not exist in the world yet"* — ⚠ **premise half-falsified**: journal/0055
  made sub-8 loose voxels world-wide, so the *renderer* half is live and **the user's actual ask
  is untouched.**
- *"The sim must know about light"*

**None may be closed on reasoning alone** — only on evidence the world changed. *A wrong
"resolved" on a user field report is the worst outcome a sweep can produce, which is why the
baseline reader refused to close the razor-straight grass/dirt frontier and asked for a re-shoot
instead.*

### ⚠ Owed / unverified — deliberately not done
- **~75 baseline findings are FILED, NOT APPLIED** (`docs/audits/baseline-2026-07-28/`, 9 audits,
  ~3,900 lines, ranked in § Sequenced). Only the integrator's own same-day stalenesses and the
  `ARCHITECTURE.md` banner were applied.
- **The two `sweep_due_hook.py` thresholds are UNVERIFIED GUESSES** made this afternoon:
  `STALE_AFTER_COMMITS = 25`, and a **keyword match on commit bodies** for "recalibrat". **The
  keyword rule already false-positived** — this session's commits *discuss* a recalibration and
  read identically to one that *performs* one. Tune from experience, not taste.
- **`ores.md` needs a conceptual revisit** against `materials.md` / `material-behavior.md` —
  **owed, unscheduled**, and larger than the one premise struck today.
- **`journal/0059` is a LOST entry** (orphaned assets, a substantive walk surviving only as
  `ROADMAP:4811-4830`); **journals 0110/0111/0112 have no archive entry at all**, including
  **0111, the scale recalibration.**
- **`flow_cost_probe` was never converted to `test = true`** — the probe the entire rule was
  earned on. A live defect, candidate corrections entry.
- **18 broken rustdoc links across two crates**; the gate is structurally blind to them. **Booked
  for a design conversation** — *"add `cargo doc` to the gate" is NOT the decided answer.*
- **The full test suite was NOT run today** and is **not claimed green**. The only Rust changes
  were doc comments; `fmt` + `clippy --all-targets --release -D warnings` covered them.

### Running
**Nothing.** All nine sweep agents in, worktrees removed, branches deleted, tree clean, pushed,
no lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-28 MORNING close (SUPERSEDED by the block above)

**Read first: `.claude/skills/doc-topology/SKILL.md`'s opening stop-block** (new, and it is the
door to everything below), then `docs/design/corpus-knowledge-notebook.md` §§ 4–5, then
`corrections.md` **#66 and #67**.

### The one sentence that matters
**We spent the session measuring our own docs problem and it is not the problem we thought.**
**Staleness is 3–8 % of our recorded failures; ~85–91 % of them were wrong the day they were
written.** So a stale-ref detector has a single-digit ceiling *by construction*, and the target is
**assertion-time, not decay**. Two independent codings agree on every direction and differ on
magnitudes by ~2×, with **43 % of entries ambiguous** — so every number is a band.

### Shipped
- **The bootstrap history content is REMOVED** (journal/0121, −713 Rust lines). Gate green on
  merged main, **85 binaries / 808 passed / 0 failed**, reconciling exactly against 86/811 with all
  three dropped tests named and confirmed absent.
- **44 dangling cross-file pointers fixed** (the audit had estimated ~25 — it undercounted by 18,
  and 4 were already wrong *before* the archive split).
- **`corrections #67` — the first entry filed against another entry.** #40 was itself a
  misdiagnosis.
- **Three process fixes + the discoverability fix** (below), and the **push** standing instruction.

### Ratified (user's terms)
- **Keep `origin/main` up to date from here out** — *"push it and we'll continue to keep the remote
  up to date from here out."* Standing authorisation; folded into `session-workflow` § Integration
  and `wrap` § 10. *`origin` had sat 45 commits behind, 19 of them from before this session.*
- **The docs diagnosis must be discoverable from a SKILL, not from CLAUDE.md** — *"if i tell an
  agent the docs are a mess… this ought to be discoverable near immediately, in our process, likely
  via an appropriate sounding skill that may already exist."* Done: `doc-topology`'s description now
  names the trigger phrasings and its body opens with the four results. **CLAUDE.md deliberately
  unchanged.**
- **The five user-owed decisions are PARKED, not forgotten** — *"i won't muddy waters by addressing
  that in this conversation."* § Sequenced → ~~**USER DECISIONS OWED**~~ **"✅ ALL FIVE USER
  DECISIONS RULED 2026-07-28"** (pointer repaired 2026-07-29 — no such heading existed; and
  they are ruled, not parked), context inline.

### Falsified — the assistant's own first
- **My adoption law, falsified by me mid-session.** *"A convention is adopted iff a machine consumes
  it"* — **false**: five documented conventions with **no** consumer sit at 97–100 %. What survives:
  *a convention is adopted only if it is inseparable from something the author must do anyway, or is
  the natural way to say the thing.* `JUSTIFIED-BY` (3 uses, 0 in `crates/`) vs `heir` (651).
- **My staleness count was an overcount** — I said ~8 %, the independent re-coder found **3.0 %**,
  in the direction that flatters tooling.
- **My first reciprocity instrument reported `0/67`, which is impossible** (`\b` in a POSIX-ERE
  grep). A 0 % that flattered the thesis was one publication away.
- **I quoted my own single-coded figures as settled in this board for several hours after the
  re-coding revised them** — the *summary-that-outran-its-source* shape, committed by the author of
  the finding. Fixed at the wrap.
- **`corrections #40`** (assistant, 2026-07-23): the seam audit did not paraphrase; it quoted the
  comment *as it stood the day before*. **A stale READ, not a stale claim.** → #67.
- **`corrections #66`** (the removal): *"the goldens will move and that is correct"* — **not one
  moved.** *A pre-authorised golden move is indistinguishable from an unexplained one.*
- **`#56` asserted, unstruck, exactly what `#60` withdraws**, 220 lines away, with **no `#60` token
  anywhere in the file.** In the artifact whose whole job is recording falsified claims.

### First things next session
1. **RATE, with the creep limiter as its acceptance test** — unchanged and still the 🔴🔴🔴 top
   blocker. Acceptance: concavity ACF(1) back toward **+0.38** with closed hollows at **zero**,
   paired with a neighbour-relative measure (corrections #61). Brief must **name what the criterion
   is NOT**. *It is also a live specimen of the "ratified-but-unbuilt" obligation class — authored
   in prose, consequence named, untracked for four days while its absence produced the blocker.*
2. **A decision on the knowledge layer, not more analysis.** Notebook § 5 proposes a **reciprocity
   check** (assistant-originated, unratified): *does every artifact that supersedes another by name
   carry a back-pointer?* Zero new authoring, 8 known failures. Or park it. ~~**`stubs.md:22`
   forbids designing the general mechanism first, and that binds this thread.**~~ **WITHDRAWN
   2026-07-28 (user)** — there was never a registry to defer, and that clause never governed this
   thread. **`session-workflow` § Seam-first #6 still binds** and says the same thing about how to
   build anything.
3. **A `doc-topology` sweep is due** — an arc shipped today and the skill's shape-6 check changed.

### Gate
**Green on merged main at `a378d47`** — fmt 0, clippy 0, **85 binaries (78 + 7 doc-tests) / 808
passed / 0 failed / 3 ignored**, 0 errors/panics/FAILED, all four byte-identity goldens present by
name and `ok`, `Checking`/`Compiling dc-worldgen` from **main's** path. Commits after it are
docs-only.

### ⚠ Owed / unverified
- **I dispatched two agents WITHOUT `isolation: "worktree"`** — both ended up in the main checkout
  and one nearly swept an untracked notebook into its commit. My error, twice. Pass `isolation`
  explicitly.
- **One re-coding claim I did NOT verify myself:** that #60's falsifier sits two sentences above the
  claim in `journal/0111:119-129`. Four of its five I verified directly; this one rides on its
  citation.
- **The removal agent's gate/probe logs died with its worktree.** The numbers are in journal/0121
  and the merge commit; the logs are not recoverable.
- **Still not read, declared:** `north-star.md` bodies beyond §§ boundary→Deviations,
  `material-behavior.md` § 5 at source, the domain-doc bodies, spikes, `ROADMAP-history.md`, and
  individual journal entries other than 0119.
- **Owed on the analysis:** a second reader on the § 3.5c coding rule *(one done — a third would
  settle the 43 % ambiguity)*, and the 65 re-coded at § granularity.
- **`JUSTIFIED-BY`'s fate is a main-session call** — `spines.md` § 5 left it there 2026-07-24 and
  `spine-audit` check #4 still tells auditors to grep it. Today's number (3 uses, 0 in `crates/`)
  argues for rewriting § 5 around the prose form, but § 4's rule binds me as much as an auditor.
- **`erosion.rs` ~4,000 lines**, split sequenced not done. **File-size thresholds still the hook's
  guesses.** **CI remains deleted.**

### Running
**Nothing.** All agents in, all worktrees removed, all branches deleted, working tree clean,
**pushed — `main` == `origin/main`**, no held lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-27 close (SUPERSEDED by the block above)

**Read first: `journal/0116` and `journal/0119`.** The erosion blocker is localised, and the
pass architecture changed underneath it. Then `ARCHITECTURE.md` § *The engine is
plugin-agnostic, and pass ORDER is authored*, and `corrections.md` **#61–#65**.

### The one sentence that matters
**The blocker is `erosion.rs::diffuse_scale_cell`** — it caps a cell's hillslope export at
its **entire regolith inventory**, with **no `dt` in the expression**. A donor-cell scheme
that moves everything downslope has a **period-2 mode by construction** (A gives all its
cover to B; B is now higher and gives it back), independent of step size. Measured concavity
ACF(1) **−0.87 / −0.91** against the shipped world's **+0.38**. **It is not the incision
clamp, not the timestep, and not isostasy** — all three were tested and killed (journal/0116).

### Ratified (user's terms)
- **THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP, EMPHATICALLY.** Passes are **plugins**,
  viewed through the lens of third-party mods; *we are our own first modders*. **ORDER is
  authored per world**; `{reads, writes}` become the **validator**, not the generator.
  `DeepAxis` is the named violation.
- **The product:** a voxel crafting-game **generator**, each world its own sim composed from
  declared plugins, whose **first pack** is the earth-like generator.
- **Remove the bootstrap history content** (polities/sites/ruins) — *"unratified zealous
  fabrications… they are NOTHING."* **Existence is not standing.**
- **Trust is DEFERRED** — no difference in permission between native and WASM; do not
  justify a core/content placement by trust.
- **`ecology.md` stays a ratified user design**, but is dormant and open to reconsideration.
- **Burial-dominant coal rank is correct physics**; the half-thickness term is a **defect**.

### Falsified — the assistant's own first (#61–#65)
**#61** a *global aggregate* ("relief within 5 %") cannot license a claim about *local
structure* — relief +4.6 % while cell-to-cell roughness went **×170**. **#62** the pit census
**saturates**: "below all eight neighbours" is a ranking test wearing a magnitude test's
clothes, and it undercounted 2.6× — **caught by the user flying the terrain**, after three
instruments agreed because they were the same instrument. **#63** the stability hypothesis,
specified against `myr_per_epoch`, *a knob that does not exist*. **#63b** isostasy is the
only grid-scale **damper**, not the driver. **#64** part (a)'s blast radius was assembled by
symmetry. **#65** the phase order never "fell out of the declarations" — it was **fed in**.

### First things next session
1. **RATE, with the creep limiter as its acceptance test.** *Not* a patch beside the
   architecture — journal/0116's prescription is *"a transfer that stays a function of
   `rate × dt`"*, which **is** the RATE axis (ratified 2026-07-24, never built, `dt` pinned
   to 1.0). Acceptance pairs an aggregate with a **neighbour-relative** measure: concavity
   ACF(1) back toward **+0.38** with hollows at **zero**.
2. **The corpus-addressability design pass** — frontmatter, tags, query scripts, and
   **versioned standing models**, booked by the user. Third leg of docs ops beside the
   archive and the sweep.
3. **The bootstrap-content removal** (also discharges draw-domain part (a)).
4. **Doc-topology residuals** — ~25 dangling cross-refs from the split, and the coal
   evidence base mislabelled "the production world" in four docs (needs a **measurement**).

### Gate
**Green on merged main at `8bcca7b`** — fmt 0, clippy 0, **86 binaries / 811 passed / 0
failed**, verified by name, reconciling exactly across all four merges (+1 binary and +4
tests are the discriminators'). *The first attempt reported "exit 0" while having run **10
binaries of 86** — the harness exit was the task's not cargo's, `$LASTEXITCODE` was empty
because a cmdlet ended the pipeline, and the lock was gone because `;` is unconditional.
**Only the impossible count caught it.***

### ⚠ Owed / unverified
- **The live-magnitudes tour** is more owed than before (stubs #28: wave/wind/frost are 45×
  weaker relative to the landscape than the day their numbers were chosen).
- **`erosion.rs` ~4,000 lines**, three separable concerns; split sequenced, not done.
- **`ROADMAP` § Observed (~1,970 lines)** is the largest unswept surface — next sweep's spine.
- **File-size thresholds are still the hook's provisional guesses**, not the user's numbers.
- **CI remains deleted**; if wanted it needs designing, not resurrecting.
- **journal/0117 is a deliberate gap. ~~and 0120~~ — 0120 WAS WRITTEN 2026-07-28** (the five
  user decisions), which is exactly what the sweep reserved it for. **And the real gap is
  `journal/0059`** — found by the baseline sweep: three orphaned assets
  (`journal/assets/0059-dune-field-*`, `0059-loess-margin-*`), a substantive walk with a
  user-diagnosed defect surviving only as `ROADMAP.md:4811-4830`, and **nobody ever decided to
  skip it.** *A reserved number and a lost number look identical from the outside; that is why
  gaps get accounted for in the close block — and 0059 slipped through the accounting that was
  built to catch it.* — the rename slice and the sweep each judged
  a narrative entry unwarranted and said so. Not lost entries.

### Running
**Nothing.** All agents in, all worktrees removed, all branches deleted, working tree clean,
no held lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-26 MORNING close (SUPERSEDED by the block above)

**Read first: `journal/0111` and `journal/0114`** — the world is ~10³× too slow, and the
reason is now known to be **both** the constants *and* a capped transport operator. Then
`docs/design/material-genesis-notebook.md` (opened today) and `north-star.md` §
*"The refinement tier is in neither list"*.

### Shipped (journals 0108–0114; corrections #53–#60; stubs #21–#29)
- **The clock nobody checked (0111).** The world denudes at **0.0110 m/Myr** — **9× slower
  than the slowest landscape ever measured on Earth**, stripping 5.48 m where a craton strips
  5–10 km. *The single most active cell of 44,264 is still below the global floor.*
- **Material-aware creep (0112)** — provenance in the archive **0.000006 % → 65.206 %**.
  §13.2's gravity member is the fluvial one **with the competence curve removed**; that
  absence *is* why colluvium is unsorted. Hillslope columns +64 % distinct species vs +11 % in
  valleys — **the contrast landed six times harder where colluvium belongs**.
- **Hybrid `p` (0113)** — peak catchment **84 → 298**, p99 **beating D8**, 95.8 % of
  simultaneous divergence retained. *Under uniform `p` the world had **no trunk network at
  all**: zero land cells draining >100.*
- **The joint calibration (0114) — BUILT, MEASURED, AND OFF.** See the blockers below.
- **S20 2c (0108)** — `Fact` 16 → 8 B, every axis kept; per-depth projection 973 → 504 MiB.
- **MFD (0109)** — simultaneous divergence **0 → 7.5 M**; *the tree never licensed the
  traversal, the potential did.*
- **Movement 2b (0110)** — identity travels + sorted deposition; its null diagnosed the above.

### Ratified (user's terms)
- **The genesis discriminator is RESOLUTION, not phase** — *"can the prior be named as a
  material we track?"* The material ontology is a **sieve**: some origins are transformations
  of **molecular parts**, and we have no molecular parts. **That is the whole case for genesis
  passes.** Four-way test, with **"transform whose driver isn't built yet"** as its own
  category, and filing one of those as a genesis named as the **irreversible** error.
- **Clastic facies settled:** genesis makes the *parent* honest · weathering the *loosening* ·
  transport the *destination*.
- **S20 option 3 + 2c** (paged facts + compact encoding). **2b now, on today's faces.**
- **File size is a correctness problem** — *"Claude must grep, missing context or reading too
  much irrelevant context"*. **Hook live and proven to fire.** Immediate for new files,
  gradual for old when touched.
- **Visible river channels are the first thing built on refinement primitives.**

### Falsified — the assistant's own first (#53–#60)
**Three of today's corrections are the integrator's own bookkeeping**, and two were
propagated onto this board: **#58** — *"p → ∞ **is** D8 exactly"* (the limit takes steepest
**slope**, `route_cell` takes steepest **drop**; it was the argument that `mfd:false` lives
*inside* the model's family, and would have licensed **deleting a pinned path**). **#60** —
*"two independent instruments agree to 2.4 %"* was a **low-flux coincidence, not a
cross-check**; D1 double-counts cover that re-crosses a shoreline cycling ±35 m four times.
Also: **#53** eight of S20's nine predicted tolerance breaks **did not happen**; **#55** the
2b null was honest about mechanism and **wrong about cause**; **#57** peat/coal/charcoal
cannot be *deposited* — §12's four-way test caught in the wild **hours after being written**;
**#59** `energy_band` was an absolute threshold **secretly keyed to `k_transport`**.

### ⛔ THE TWO BLOCKERS — read before planning anything erosional
1. **stubs #29 — ⚠ RE-SCOPED BY THE WALK (journal/0115) AND DIAGNOSED BY THE
   DISCRIMINATORS (journal/0116, corrections #63). It is NOT "the incision clamp", it was NOT
   148 pits, and it is NOT a time-step stability limit.** Above 1× the surface carries a
   **checkerboard** — concavity lag-1 autocorrelation **+0.38 (shipped) → −0.87
   (calibrated)**, against derivable references of −1/6 for white noise and −1 for a pure
   oscillation. **Relief grew 4.6 %, cell-to-cell roughness ~170×**; closed hollows
   **0 → 1,377 (3.1 %), 5.4 km³**. Split `surf = r + h` and the two summands separate: the
   **bedrock converges under 4× time-step refinement** (23.8 → 5.9 m) while the **regolith
   sharpens** (ACF −0.82 → −0.93), because the creep flux limiter is **deaf to the step**
   (96.0 → 94.7 % binding) — it caps export at *inventory*, not at `rate × dt`. **The register
   is the flux limiter / donor-cell partition in `erosion.rs::diffuse`.** Do **not** brief a
   clamp fix, a time-step fix, or anything touching `iso_rate` (isostasy is the only
   grid-scale damper in the solve; turning it off puts **6,215 hollows in the SHIPPED
   world**). **Until this lands the engine cannot run erosion at any realistic rate.**
2. **stubs #27 — the transport operator has a ceiling.** Export ∝ mean regolith thickness, and
   creep's limiter **already binds on ~89 % of cells at shipped rates**. 100× on transport
   buys **1.6×**. *So journal/0111's "a calibration, not an architecture" was **half right**.*

### Running
**Nothing. All agents in, all worktrees removed, all branches deleted, working tree clean.**

### Gate
**Green on merged main at `a05008f` — fmt 0, clippy 0, 807 passed / 0 failed / 85 binaries**,
verified by name, reconciling exactly across all five slices. Production is **byte-identical**
(`the_production_world_still_hashes_to_the_pre_slice_goldens`).

### ⚠ Owed / unverified
- **An appearance walk is owed** — creep's interbedded colluvium is the first thing in three
  slices worth standing in front of. Tour-map first; stations: hillslope road cut, scarp-foot
  apron beside a channel deposit, the re-baselined coal site. **Look at the pits first.**
- **`erosion.rs` is ~4,000 lines** and hosts three separable concerns (drainage/MFD · fluvial
  transport · diffusion+creep). Split **sequenced, not done** — deliberately, siblings were live.
- **ROADMAP is ~6,850 lines** and the hook flags it on every edit. The archive-by-status
  scheme is still undesigned.
- **CI was deleted** (day-one scaffolding, 3-OS debug matrix, failing on Bevy's Linux deps and
  burning 2.5 h Windows runs). If wanted, it needs designing, not resurrecting.
- **Commit trailer mismatch:** CLAUDE.md § Conventions says `Claude Fable 5`; this session's
  commits say `Claude Opus 5` (the model that did the work). **User's call which is canonical.**

### First things next session
1. **stubs #29 — the REGOLITH-CHECKERBOARD blocker** (re-scoped by the walk 2026-07-26; then
   **diagnosed the same day, journal/0116, corrections #63 — the discriminators are RUN and
   the stability-limit hypothesis is dead**). It gates every erosional number. Brief the fix
   against the **flux limiter / donor-cell partition in `erosion.rs::diffuse`** — *not* the
   clamp, *not* the time step, and **not `iso_rate`**.
2. **stubs #27's heirs** — rivers that carry, or a non-capped creep operator. **These two
   share the LIMITER, not the clock** — #29's "same `cell_m / myr_per_epoch` register" claim
   is withdrawn (corrections #63; the knob does not exist and the epoch length is measured not
   to be the register). A non-capped creep operator is now plausibly **one slice for both**.
3. ~~The appearance walk (creep)~~ — **DONE 2026-07-26, journal/0115.** Flip
   `calibrated_rates` once #29 is fixed.
4. **Refinement primitives design pass** — unblocked by hybrid `p`; visible channels.

*(The 2026-07-25 block is consumed; preserved as history — **archived 2026-07-26 to
`ROADMAP-history.md`**, where it is the first of the six superseded close blocks. It used to
sit directly below this line.)*




## NEXT SESSION — written at the 2026-07-25 close (SUPERSEDED by the 2026-07-26 close above)

**Read first: `docs/design/flow.md`** (ratified this session, incl. §§ 10–11) + the 🔴 Observed
coal entry + `docs/audits/2026-07-25-roadmap-staleness-sweep.md` (see *Running* below).

### Shipped (journals 0094, 0096–0104, 0106–0107; corrections #48–#51)
- **FLOW arc opened and two slices landed.** `flow.md` **ratified**: flow is one process, an atom
  of *(cell, stratum-slot, faces, form, load, fluid, cause, chapter)*. **Slice 1** (0096) records
  **flux on 3D FACES per chapter** — **divergence is representable** (175,320), which a receiver
  tree forbids **by construction**. **(a) the head field** (0098) — `dc:field/head`, vertical flux
  **0 → 307,364** across 44.3 % of cells, **artesian representable *and* occurring** (60 columns).
- **Weathering became a PROCESS then a PROFILE.** 0094 put it in the loop (accumulating, ≥1 voxel);
  **0099 graded it** — a 19-voxel front, `structure→pore_fill`, retained parent fabric at the
  bottom contact — **and the user WALKED and PASSED it**.
- **`identify(pos)`** (0101, UNTIERED) — `UNRECORDED` is a first-class answer; phantom air
  **702 → 0**.
- **Residency: down while gaining two records.** `strata` shrink (−54.02 MiB) + `FactLedger`
  flat+CSR (0100) + one grid-wide record (0102): flag-ON **311.02 → 173.61 MiB**.
- **Declarations now BIND.** `reads_prev` is enforced by real anti-dependency edges and is
  **rename-proof** (0104); `dc:field/head` declares what it reads (0107).
- **The gate can see its instruments** (0103) — 7 probes, 16 tests, +35 s.
- **`production_field()` now IS production** (0106).

### Ratified (user's terms)
- **`identify(pos)` is UNTIERED** — *"if this is a world query then why tiered at all?"* The
  tier design was **assistant-originated** and had hardened unchallenged. Payload is **uniformly a
  mixture**, so far-field speckle falls out **by construction**.
- **"WEATHERING IS ONE PROCESS, saprolite is a state along it"** — strong leaning, **R1/R2/R3 now
  ALL MET**. Recalibrate after biology, **never by tweaking onset temp — real Earth numbers**.
- **The aggregation window is DECLARED, never assumed** (*"we are the first modders"*); the
  **record self-describes** its completeness; pairing is **two-mode green** + a conduit third mode
  in (c).
- **The world output is a SCRATCH PAD** — goldens are a regression detector, **not a target**;
  *"we eat it if it changes the physics."*
- **Claude drives the WHOLE walk loop** — launch, teleport, measure, screenshot, brief, pause.
- Hash draws need a **construction guarantee**, not a convention.

### Falsified — assistant's own first (#48–#51)
`~110 km` station landmark, wrong by 39 km (#48) · `has_contents` per-chunk, a **reporting** hole
not a record hole (#49) · the front's `+3.7 %` mass was **the instrument**, doctrine confirmed
(#50) · **the shipped world has ZERO coal**, and the A-3 guard ran on a world nobody ships (#51).
Also: my per-cell-container rule was **too broad** (the defect is *resident*, not per-cell); my
floor-effect hypothesis was **wrong**; the spine-audit's own prescription was **half wrong**.

### Running — ONE AGENT STILL OUT (integrator error: an earlier draft of this block said "nothing is running")
**✅ LANDED AND MERGED:** the hash-domain provider (journal/0105, corrections #52 — *nothing
moved in the goldens, and that was the finding*) and the ROADMAP staleness sweep
(`docs/audits/2026-07-25-roadmap-staleness-sweep.md`).

**⏳ STILL OUT — the one thing to collect first next session:**
- **S20 spike — fact-ledger residency** (worktree `a83b3917a1061f32c`, **still live at close**).
  Feeds the per-depth weathering decision with four costed options incl. **paged facts**.
  **Spike only — verify it did NOT change the shipped `FactLedger`/`LedgerField` layout**, and
  that its probes are gated. Its worktree is intact; **check `git -C <worktree> status` and its
  branch commits before concluding anything** — an empty worktree is not evidence of nothing, and
  bare `git` pathspecs from inside a pruned worktree lie.

### PROCESS THREAD the user opened at the close — gh issues alongside the ROADMAP
*"We could leverage gh issues more… we do want to preserve whatever benefits we are getting from
the roadmap doc, but we have other tools to explore to make our knowledge and planning and
consistency and integration-over-time maximally effective with maximal agent experience."*
**Not decided — next session's design pass.** The integrator's framing, recorded so it is not
re-derived:
- **The ROADMAP is doing FOUR jobs**, and only some suit a doc. **Shipped** = history (the journal
  already holds the narrative — archive it). **Sequenced** = arcs whose value *is* long-form
  reasoning (WHAT/WHY/UNIFIES/SLICE/CONTINUATION) — **keep as a doc**. **Observed** = an inbox of
  discrete, statusful findings — **this is an issue tracker's native shape**, and it is the part
  that demonstrably rots. **Close block** = handoff — stays.
- **The hard constraint is AGENT EXPERIENCE, and it cuts toward in-repo.** Subagents work in
  **worktrees**; every brief this session said *"read ROADMAP § X"*, which works because it is a
  **file**. `gh issue view` needs auth + network inside a worktree — a real friction multiplier
  across ~15 agents/day. Losing **atomicity** matters too: today a slice's commit updates ROADMAP
  + stubs + spines + journal *together*; an issue closed separately can drift from the commit.
- **What issues genuinely add:** status as a **first-class queryable field** (ours is prose —
  ✅/⏳/strikethrough — which is exactly what rots), labels, and commit/PR auto-linking.
- **A third option worth costing before choosing:** keep everything in-repo but make status
  **machine-checkable** — one file per Observed item with frontmatter, or a small structured
  index — which buys queryability **without** losing offline-readability or atomicity.
- **Viable hybrid:** the *integrator* (networked) queries issues and **inlines** the relevant text
  into each brief — which is exactly what already happens with ROADMAP quotes today.

### ⚠ OWED / unverified across the boundary
- **A full `--workspace` gate has NOT run on current main.** dc-worldgen is verified on **merged
  main** (307 by name, 0107's re-run); dc-api/dc-client unchanged since their last green (719 /
  72 binaries). **Run the combined gate as STAGES** (clean+fmt+clippy, then test) — it now exceeds
  one 10-minute call.
- **Journal 0105 is reserved**, not missing.
- **Coal: the user's call is (a) accept a coal-free world.** Do **not** tune `COAL_ONSET_C`.

### PROCESS ITEM the user opened at the close — the ROADMAP outgrew reading
**~5,600 lines, and no session reads it end-to-end — it is *grepped*.** The user named the
risk and it is real: *"things could go unnoticed."* **They do.** `DeepField::chapters` sat
unlisted through **three** spine audits while sweeps added rows for its **immediate neighbours
in the same struct**. Grep surfaces only what you already know to look for — never an item
whose vocabulary drifted, one that now **contradicts** something just ratified, or one nobody
has thought about in weeks. Those are precisely the ones that mislead.
**Two controls, and the second matters more:** **(a)** archive by **STATUS, not age** —
`Shipped` grows without bound, is least often needed live, and the **journal already holds its
narrative**; moving old `Shipped` to a history file (journal number as the stable pointer)
leaves `Sequenced` + `Observed` + the close block, which must stay readable. *Age is the wrong
axis — an old `Observed` may be the most live thing on the board.* **(b) Make the staleness
sweep RECURRING, like `spine-audit`** — shrinking the doc helps someone already looking and
does nothing for what nobody thinks to look at. **Today's sweep had to be requested; that is
the gap.** Design the archive scheme next session *before* it grows another thousand lines.

### 🔴 THE STALENESS SWEEP LANDED — read it before planning
`docs/audits/2026-07-25-roadmap-staleness-sweep.md`, 22 rows. **All agents are in; nothing is
running.** The three that would most mislead:
1. **`3e-2 C refinement` carries THREE contradictory stamps** — Sequenced says *"Nothing open —
   implementable"*, the FLOW entry ~1,000 lines earlier says its **expression half is
   superseded**, and the 2026-07-24 sweep marked it **VALIDATED**. Two Sequenced entries give
   **opposite dispatch advice**. *Does anything of 3e-2 remain dispatchable, or only the
   qualified constraint?* **User call.**
2. **Movement 2b has NO Sequenced entry** — it lives only in a close block, and its ratified
   mechanism (§13.1, transport "along the **pinned receiver**") is **exactly the spanning tree
   FLOW retires**. Ready-to-paste replacement is in the audit.
3. **corrections #51, ONE FILE OVER, and journal/0106's audit missed it.**
   `tests/organic.rs:31` runs `the_measured_coal_seam_is_coal_a_player_can_dig` on
   `SEED = 0x0D5E_ED57_2026` — **a seed the client structurally cannot open** (it does not fit
   `BENCH_SEED`'s `i32`). `MIN_DIGGABLE_COAL_VOX` was re-baselined **15→10→6** on that world,
   twice under NEEDS RATIFICATION. Unlike the helpers 0106 *did* clear, **this claim is not
   seed-independent.** Fix it the way `production_field()` was fixed.
Also: **flow.md §11.1's ratified third scheduler axis (`ORDER × RATE × WINDOW`) appears NOWHERE
in ROADMAP**, and material-behavior §5 is unamended — a ratified decision with no home.

### First things next session
1. **Fold the staleness sweep**, then the staged gate (`./scripts/gate.ps1`).
   ✅ **THE FOLD LANDED 2026-07-25** — all 22 rows applied or accounted for; the three missing
   Sequenced entries (**Movement 2b**, **the aggregation window**, **metamorphism**) now exist,
   material-behavior.md § 5 carries the WINDOW axis, and C-1 / C-2 / C-3 are marked but left for
   the user. Still owed: the staged gate.
2. **The per-depth weathering arc** — gate open; the decision is **residency axes** (S20 informs).
3. **Movement 2b — material-aware transport** (§13), the big appearance-changer.
4. **MFD / simultaneous divergence** — unblocked by the head field; today's divergence is
   **avulsion only** (aggregation-window), never concurrent distributaries.

*(The 2026-07-24 BUILD-DAY block below is consumed; preserved as history.)*

## NEXT SESSION — written at the 2026-07-24 BUILD-DAY close (SUPERSEDED by the 2026-07-25 close above)

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
   *(2026-07-25: this now has a real Sequenced entry — **"METAMORPHISM — the grade axis"** — so it
   no longer depends on a close block surviving a rewrite. Sweep row D-1.)*
4. **Igneous emplacement + the formation-predicate evaluator** (F1 ratified): the
   predicate-as-data machinery + exhum-driven outcrop structure, retiring stubs #5/#16.

### Owed / carried
**⚠ "geotherm coal" IS NOT A WALK — it is a desk null (corrections #51); see docs/audits/2026-07-25-roadmap-staleness-sweep.md row C-3. This superseded block must not be read as carrying it forward.**
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
