# Roadmap — history

**Archived from `ROADMAP.md` on 2026-07-26, by STATUS, not by age.** The live board is
`ROADMAP.md`: *Sequenced* + *Observed* + *In flight* + the current close block — the parts
that must stay readable. This file holds the two parts that are least often needed live and
whose narrative is already carried elsewhere:

1. **Shipped** — every entry keeps its **journal number**, which is the stable pointer. The
   journal holds the story; this holds the ledger.
2. **Superseded close blocks** — each was consumed by the one after it.

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
    **stubs #29** (Sequenced above; re-scoped 2026-07-26 — the real defect is grid
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
  entry *"A TIE-BREAK IS DECIDING PHYSICS AGAIN"*, now struck through below). `DeepPass::reads_prev`
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
    84.47 MiB heap, with **33,680 cells (11.3 %) holding an empty record**. See Observed.
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
  - **⚠ CORRECTED 2026-07-25 (corrections #51; staleness sweep row S-6).** *"Still a diggable seam
    on Medium"* was measured on the **warm reference** world `0x0D5EED572026`, **not** on the world
    the client boots. On seed **1337 / Medium** the recalibration produces **zero coal — 0 units
    across 297,025 cells**, the hottest peat candidate standing at **15.4 °C against a 22 °C
    onset**, i.e. 6.6 °C short. The cited guard no longer exists under that name: `tests/geotherm.rs`
    now carries `the_geotherm_rule_governs_coalification_on_the_production_world` (1337 / Medium,
    which **requires no coal**) and `coal_follows_the_warm_crust_on_the_warm_reference_world`
    (journal/0106). The recalibration itself stands; only the sentence about what a player would
    find was wrong. The content question — (a) accept a coal-free world … (d) — is open in Observed,
    and the appearance walk this entry owed is a **desk null**, not a walk (see APPEARANCE WALKS
    OWED item (3)).

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


---

# Superseded close blocks

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
