# `ROADMAP.md` — classification of what REMAINS after the 2026-07-29 mechanical passes

**Read at commit `6eff1c8`** (`archive the four superseded close blocks (409 lines) to
ROADMAP-history`). Board size at that commit: **4,897 lines** against the class-REGISTRY
threshold of 2,500.

> **⚠ Line refs quoted from entries below are as-of that commit and some have drifted.** Row
> **O-1**'s `runner.rs:542-544` / `:889` / `:532-535` / `:1454-1456` were invalidated the same
> day by the `docs/design/pass-declaration-history.md` extraction (comment-only; the live values
> are `:579-581`, `:933`, `:569-572`, `:1497-1499`, refreshed in `ROADMAP-history.md` where that
> entry now lives). **Locate by content, never by number.**

**What this audit is.** The two mechanical classes were already taken on 2026-07-29: the 37
self-declared-resolved § Observed entries + 8 user strikes (morning, `scripts/roadmap_archive.py`,
sourced from `docs/audits/baseline-2026-07-28/S6-roadmap-observed.md`), and the four superseded
close blocks (afternoon). **Those passes touched § Observed and the close blocks only.** § In
flight (545 lines) and § Sequenced (2,845 lines) have never been swept for archivability at all,
and § Observed's remaining 85 entries have never been re-read past the S6 audit's verdicts.

This audit reads **every top-level entry in all three live sections at source** and assigns one
of five classes. It is the input to `scripts/roadmap_archive2.py`, which consumes classes 1–3
and **excludes class 4 entirely** (nothing in class 4 moves without the user's word).

**Method.** Where an entry claims a code fact, the fact was checked in `crates/` or in the doc
that owns it — not from the entry's own text and not from a journal's claim about itself. Where
the check could not be made cheaply, the entry defaults to **LIVE**. *An audit finding is a
hypothesis, not an authority: re-verify before applying.*

**⚠ Before running the mover, re-run `--check`.** `ROOT` points at the shared checkout
`B:/repos/borwood/deepcraft/`, and `--check` was run against that copy — its resolved ranges
matched this worktree's reads line for line, which is the evidence that the two agree. But this
agent could not read the shared checkout's git state, so **confirm main is at `6eff1c8` (or
re-run `--check` and eyeball the ranges) before applying.** An anchor that has drifted fails
loudly rather than moving the wrong text; that is the design.

**⚠ Locate entries by the quoted ANCHOR, never by line number.** Every line number below drifts
the moment the first range moves. The anchors are the addresses; they were checked for
uniqueness by `scripts/roadmap_archive2.py --check`.

---

## 1. Counts and yield

Counts below are **not estimates** — they are the output of `scripts/roadmap_archive2.py --check`
run against `ROADMAP.md` @`6eff1c8`, which resolves every anchor, proves each is unique in its
scope, and proves no two ranges overlap.

| Class | Ops | Lines moved |
|---|---|---|
| 1. MOVE-MECHANICAL | 28 | 496 |
| 2. MOVE-WITH-POINTER | 20 | 411 |
| 3. SHRINK-IN-PLACE (sub-ranges) | 11 | 440 |
| **Subtotal — the script's scope** | **59** | **1,347** |
| 4. STRIKE-CANDIDATE (user only) | 5 | ~95 (does not move) |
| 5. LIVE | ~120 entries | — |

Plus **6 FIXUPS** — in-place pointer repairs applied after removal, each proved unique in the
*surviving* text. Five entries point at something that moves; a pointer into a hole is worse
than no pointer, and this pass would otherwise author five of them.

**Post-archive board: ~3,545 lines** (4,897 − 1,347, before blank-line collapse).

**⚠ HONEST LIMIT, stated up front: this does NOT reach 2,500, and no honest classification can.**
After the moves, § Sequenced still holds ~1,900 lines of genuinely open arcs (FLOW, weathering,
identity, genesis, pass architecture, the erosion calibration) and § Observed ~1,050 lines of
genuinely open observations. **The remaining volume lever is not status — it is structure.** The
two candidates, both user calls and neither taken here:

- **Split § Sequenced by liveness the way `ROADMAP → ROADMAP-history` was split** — the entry's
  own reasoning-and-record half vs its head. The file-size convention (`scripts/filesize_hook.py`
  docstring, DECIDED 2026-07-28) says *the split axis is liveness, never topic*; a
  `ROADMAP-reasoning.md` would be the third liveness split and would be within the convention.
- **Accept that a registry over 2,500 lines is the correct size for this board** and raise the
  REGISTRY threshold, which is the number the same decision explicitly calls negotiable.

---

## 2. § In flight (545 lines)

**Verdict on the section as a whole: it is not "in flight."** Ten top-level bullets and ~220
lines of un-bulleted prose. Four of the bullets and five of the prose blocks are dated records
of shipped work or explicitly-superseded blocks; two prose blocks are 2026-07-21 appearance
ratifications the world has moved out from under. Only the north star, the material-behavior
substrate, the block↔material collapse, the threshold-quantization migration and the
forms/partials contract are live arcs.

| # | Anchor (unique substring) | Class | Evidence |
|---|---|---|---|
| IF-1 | `**Session 3 shipped, all gates green on merged main:**` | **MECHANICAL** | Self-declared shipped ledger for journals 0023–0030, all in `ROADMAP-history.md` § Shipped. 6 lines. |
| IF-2 | `**Session 5 shipped (2026-07-21, journal/0039` | **MECHANICAL** | Same shape, journals 0039–0049. 9 lines. |
| IF-3 | `**(SUPERSEDED by the 2026-07-22 close at the end of this file.)**` | **MECHANICAL** | The entry declares its own supersession in its first six words. A close block that survived the 2026-07-29 close-block archive because it is not under a `## NEXT SESSION` heading. 26 lines. |
| IF-4 | `**Wide horizons: the blocker is GONE and now PROVEN at 6**` | **MECHANICAL** | Self-declared resolved (journal/0065). It is also the refutation of § Observed's `GPU DeviceLost crash under a teleport storm` (O-6 below) — **the two must move in the same commit** or the claim outlives its refutation. 14 lines. |
| IF-5 | `*(Superseded by a session-close block; kept for the record.` | **MECHANICAL** | Self-declared superseded, plus the trailing `RE-SEQUENCED 2026-07-21` list and a `Read first next session` directive four close blocks stale (`ROADMAP.md` § NEXT SESSION is dated 2026-07-29). 36 lines. |
| IF-6 | `The octree substrate — DESIGN PASS OPENED, D1–D3 DECIDED` | **POINTER** | Body ends `**MERGED TO MAIN + LOOK RATIFIED AS-BUILT** (2026-07-22, integrator merge; user … "visually indistinguishable")`. FF2b-minimal landed (journal/0070). 41 lines. **⚠ OBLIGATION ON THE MOVER — four follow-ons live only in this entry and are owned nowhere else:** persistence + dirty-rail (far edits), synthesized sub-surface strata, partial-coverage composition, deep-span greedy merge. The banner carries them; **main session must re-file them as a live § Sequenced line in the same commit.** |
| IF-7 | `**Tectonics architecture RATIFIED 2026-07-20** — all of U1–U8` | **POINTER** | A pointer to a SPIKE that has its own § Sequenced entry (`Tectonics SPIKE** (per tectonics.md § SPIKE`), and to a dispatch gate ("behind the eolian agent's landing") discharged 2026-07-21. 4 lines. |
| IF-8 | `Tectonic design pass — DRAFT LANDED (historical entry)` | **MECHANICAL** | Self-labelled *"(historical entry)"* in its own header. `docs/design/tectonics.md` exists and now carries a supersession banner (verified `tectonics.md:3-8` @`6eff1c8`). 14 lines. |
| IF-9 | `**Zonal circulation profile** — dispatched same moment` | **POINTER** | Shipped 2026-07-20, journal/0037 — this board says so itself in § Sequenced (`**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037`), which is IF-9's own duplicate and is SEQ-23 below. 5 lines. |

**LIVE in § In flight** (no action): the north star bullet · the material-behavior substrate ·
the block↔material collapse · the ratified sequence after the migration · the
threshold-quantization migration · the forms/partials fill contract · the ore design-pass
parenthetical · *Slated by ratification 2026-07-20* · *Decisions waiting on the user* items 0/3/4
· *Design passes queued* · *Engineering still Sequenced*.

**⚠ Two live-entry defects found and NOT fixed here** (they need a one-line edit, not a move):

1. **The forms/partials fill contract's `NEXT SLICE` says it is `BLOCKED on a user-owned
   decision` — and that decision was made eight days ago.** The stated blocker is *"retire the
   veneer for recorded columns first (user in the room)"*; the veneer retirement is recorded
   **DONE 2026-07-21** thirteen lines below in this same section (*"the user made the call live …
   and it shipped in journal/0055"*). The fractional-top slice is unblocked and unowned. A claim
   and its own discharge, 150 lines apart, in § In flight.
2. **`The ratified sequence after the migration`'s trailing clause** *"remaining CoarseField
   migration follows the type freeze"* was, until 2026-07-29, the **only** live home of the
   CoarseField adoption (the § Sequenced revival entry says so by name). Now that the revival
   entry exists, this clause should point at it rather than carry it.

---

## 3. § Sequenced (2,845 lines)

| # | Anchor (unique substring) | Class | Evidence |
|---|---|---|---|
| SEQ-1 | block `✅ ALL FIVE USER DECISIONS RULED 2026-07-28`, range `*Original framing, kept: five calls surfaced 2026-07-28` → `walked from an end that holds no link.` | **SHRINK** | The entry's own head says **"Nothing here is owed."** All five carry `✅ RULED`, and the two follow-ups they created are separate live entries (the banner backlog, the completeness gap). **The head and the `Also surfaced, NOT user-owned` tail STAY** — the tail holds two live obligations (the `JUSTIFIED-BY` main-session call; the `approx_resident_bytes` non-monotone Observed candidate). 144 lines. |
| SEQ-2 | block `THE BASELINE SWEEP'S FINDINGS`, range `**✅ ALL FOUR USER CALLS RULED 2026-07-28**` → `Deviations 2 calls *"EXPLICITLY NOT THE MODEL"*.` | **SHRINK** | The four user calls are `✅ RULED`. **All five `HIGHEST BLAST RADIUS` items verified applied at source @`6eff1c8`:** `spines.md` § S-6 rewritten (`docs/spines.md:430-433`, *"Argument rewritten 2026-07-29 to state the ratified shape"*) · the erosion-axis banner (`ROADMAP.md` § Sequenced, `THE BLOCK BELOW IS THE LOSING SIDE OF A MEASUREMENT`) · `tectonics.md` banner (`docs/design/tectonics.md:3-8`) · the seam count (`docs/spines.md:403`, *"~~34 seams inventoried~~ **31 LIVE, of 34 inventoried** (corrected 2026-07-29)"*) · ROADMAP's first bullet (`ROADMAP.md:39`, *"🔴 THE TIERING IS RETIRED — struck 2026-07-29"*). 92 lines. Head + the STRUCTURAL / BULK / `flow_cost_probe` / NOT-COVERED bullets stay live. |
| SEQ-3 | `SWEEPS RUN FIRST THING, INCREMENTALLY, AND THE HARNESS SAYS WHICH ARE DUE` | **MECHANICAL** | `✅ DECIDED`; both halves shipped and verified present: `scripts/sweep_due_hook.py` exists @`6eff1c8`, and `.claude/skills/staleness-sweep/SKILL.md` is in the skill roster. Kept only for reasoning. 39 lines. |
| SEQ-4 | block `DOC-TOPOLOGY RESIDUALS — the 19 findings not actioned 2026-07-26`, range `**✅ DONE 2026-07-28 — the dangling cross-file pointers.**` → `not the rows.*` | **SHRINK** | Three consecutive `✅ DONE 2026-07-28` sub-bullets. The live residuals (the ABI-spike contest, the coal evidence base, the next sweep's spine) stay. 37 lines. |
| SEQ-5 | `✅ DONE 2026-07-26 — THE DOC-TOPOLOGY SWEEP + THE ROADMAP ARCHIVE` | **MECHANICAL** | `✅ DONE`; both shipments verified (`.claude/skills/doc-topology/SKILL.md`, `ROADMAP-history.md`). Kept for reasoning. 35 lines. |
| SEQ-6 | block `THE PASS ARCHITECTURE — AUTHORED ORDER, OPEN VOCABULARY`, range `*Record of how the slice was framed before it shipped` → ``stubs.md` § 30 carries the stand-in with RATE named as its heir.`` | **SHRINK** | RATE shipped today (journal/0123, merge `a9146dd`); this sub-block is explicitly *"Record of how the slice was framed **before it shipped**"*. **The arc itself is LIVE** — (1), (2), (b)–(f) owed — and the `✅ RATE IS NOT EXPANDED` user ruling immediately after this range **stays**, because it is a live constraint on the S-10 spine. 14 lines. |
| SEQ-7 | `✅ DONE 2026-07-28 — THE BOOTSTRAP HISTORY CONTENT IS REMOVED` | **MECHANICAL** | `✅ DONE`, merged and gate-verified, `corrections #66`. Its one live residual (`approx_resident_bytes` non-monotone) is duplicated in SEQ-1's surviving tail — **checked deliberately, because SEQ-1 and SEQ-7 both carry it and moving both would drop it.** 47 lines. |
| SEQ-8 | `REMOVE THE BOOTSTRAP HISTORY CONTENT — polities, sites, ruins, the history pass` | **MECHANICAL** | The *"Original entry, preserved"* half of SEQ-7. Verified at source: `grep -rn "ruin_posts\|pregen/history" crates/ --include=*.rs` → zero hits @`6eff1c8`. 26 lines. |
| SEQ-9 | `THE HILLSLOPE CONVEYOR CHECKERBOARDED THE REGOLITH ABOVE 1× — stubs #29` | **POINTER** | Header is `✅ FIXED`. `docs/design/stubs.md:1092-1096` @`6eff1c8`: *"**⚠ DISCHARGED 2026-07-29 (journal/0122)** — the operator is fixed."* The live successor is the entry directly above it (`THE HILLSLOPE OPERATOR IS FIXED — SHIPPED 2026-07-29`). 104 lines — **the single largest archivable block on the board.** **⚠ Requires three in-place fixups** (F1–F3 in the script) because the successor entry points *"below"* at it three times. |
| SEQ-10 | `THE TRANSPORT OPERATOR HAS A CEILING — stubs #27` | **POINTER** | Its headline mechanism is falsified. `docs/design/stubs.md:986-991` @`6eff1c8`: *"**⚠ THE CONVEYOR IS GONE 2026-07-29 (journal/0122)** — and the calibration fitted on top of it does not survive."* The entry still asserts *"The pass is a **one-cell-per-epoch conveyor**"* as a live finding. Its surviving input (the ladder) is carried by `RE-PICK `EROSION_CALIBRATION``. 10 lines. **⚠ Requires fixup F4** (the deep-time-clock entry says *"read stubs #27 before assuming a bigger number fixes it"*). |
| SEQ-11 | block `REQUIRED CHORE — FILE SIZE IS A CORRECTNESS PROBLEM`, range `**✅ CONVENTIONS DECIDED 2026-07-28 (user)**` → `that lands.` | **SHRINK** | `✅ CONVENTIONS DECIDED`, and the entry itself names where the record now lives: *"shipped in `scripts/filesize_hook.py`, whose module docstring **is now the record**"* — verified present @`6eff1c8`. The chore's live half (immediate for new files, gradual for old when touched) stays. 46 lines. |
| SEQ-12 | block `Finish the draw-domain conversion: the residual hand-rolled sites`, range `(corrections #64, a read-only trace taken` → `` `GOLDEN_SURFACE` / `GOLDEN_RECORD` are **not** downstream (deep-time field only).`` | **SHRINK** | The entry itself says *"Read the trace below as the 2026-07-26 record it is"* — its whole visible consumer chain was deleted 2026-07-28. (a)/(b)/(c) heads stay. 19 lines. |
| SEQ-13 | `` *(The `production_* → golden_*` rename that stood here `` | **MECHANICAL** | Self-declared *"shipped 2026-07-26; see `ROADMAP-history.md` § Shipped"* — and it is there (`ROADMAP-history.md:22`). 2 lines. |
| SEQ-14 | block `FLOW IS ONE PROCESS — flux on FACES, facts on STRATA`, range `~~**🔴 OWED BY MFD, and it is the nearest-term thing on this arc` → `not a later critique.*` | **SHRINK** | Already struck in place 2026-07-29: *"✅ SHIPPED 2026-07-26 — struck 2026-07-29 (baseline sweep S5/F4). Both terms are stale."* The FLOW arc stays live. 19 lines. |
| SEQ-15 | `APPEARANCE WALKS OWED` | **POINTER** | **Nothing in it is owed.** (1) `✅ DONE — walk-confirmed 2026-07-24`; (2) `✅ DONE — walk-confirmed & ACCEPTED 2026-07-25`; (3) tour-mapped to a null, `Do not spend a walk on it` (corrections #51); (4) `✅ DONE — WALK-CONFIRMED & PASSED 2026-07-25`; (5) `NEVER OWED — ANSWERED AT THE DESK`. 48 lines. **⚠ OBLIGATION ON THE MOVER — one residue survives and is named in the banner:** *"the poke-through geometry check on the lit pass — low priority."* The next owed walk gets a fresh entry. |
| SEQ-16 | `` `FactLedger` IS 89 % EMPTY HEADERS `` | **MECHANICAL** | Head is `✅ **DONE 2026-07-25 — shipped, see `ROADMAP-history.md` § Shipped (journal/0100).**`, kept "as shaped, for the record". 36 lines. |
| SEQ-17 | `THE WEATHERING FRONT NEEDS A PROFILE, NOT A SLAB` | **MECHANICAL** | Head is `✅ SHIPPED 2026-07-25 (journal/0099)`, kept for reasoning; its earned walk is SEQ-15 item (4), also `✅ DONE`. 33 lines. |
| SEQ-18 | block `THE HONEST IDENTITY SURFACE — retire the stored `Block` summary`, range `**~~TIER BOUNDARIES DERIVE FROM THE LOD LADDER~~ — SUPERSEDED SAME DAY` → `still owed from journal/0091.` | **SHRINK** | Three consecutive blocks the entry itself marks `SUPERSEDED SAME DAY` / `DISSOLVED` / `PRESERVED FOR THE RENDER-SIDE QUERY`, retired by the `UNTIERED` decision (user, 2026-07-25) stated 50 lines above them. The arc stays live (continuation slot unbuilt). 45 lines. |
| SEQ-19 | `` Collapse-cache `evict()` is unreachable from far-field-only sampling `` | **POINTER** | **Verified false at source @`6eff1c8`:** `crates/dc-worldgen/src/collapse.rs` calls `self.evict()` at `:451, :711, :997, :1012, :1022, :1033, :1041` — reachable from every sampling path. Fixed by journal/0052, which this board records in § In flight (*"Also landed: the journal/0050 collapse-cache `evict()` gap (now reachable from `coarse_surface`, `column_record`, `surface_elev_m`, `lattice_point`, `surface_chunk_y`)"*). **A claim and its own refutation, 2,500 lines apart, in one file.** 9 lines. |
| SEQ-20 | `` `--horizon <km>`, default provably unchanged, measured to 10 km. `` | **MECHANICAL** | Self-declared SHIPPED, journal/0042. 2 lines. |
| SEQ-21 | `Wave-magnitude retune — STRUCK 2026-07-21 (user): no retune.` | **MECHANICAL** | **User-struck** in its own header, with the reasoning quoted. Rides as-built until the fetch model — which is § In flight's *"the littoral heir: fetch × wind into `Providers::wave_energy`"*. 7 lines. |
| SEQ-22 | `**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037` | **MECHANICAL** | Self-declared SHIPPED; duplicate of IF-9. 2 lines. |
| SEQ-23 | `**Deep-config flag plumbing: SHIPPED** 2026-07-20, journal/0039` | **MECHANICAL** | Self-declared SHIPPED. 6 lines. |
| SEQ-24 | `Erosion-supply calibration** (from the S12 spike's new finding,` | **POINTER** | Its stated deliverable is *"compare model denudation against real orogen rates"* — **done, and it is the largest measurement on the board**: journal/0111's `examples/denudation_probe.rs` with its cited literature table, `0.0110 m/Myr` against the 1–10 m/Myr craton band. The live successor is `RE-PICK `EROSION_CALIBRATION`` + `CALIBRATE THE DEEP-TIME CLOCK`, both of which state the method this entry asked for. 7 lines. |
| SEQ-25 | `Tectonic uplift-plane redesign — DESIGN PASS (superseded — done)` | **MECHANICAL** | Self-labelled *"(superseded — done)"*. 7 lines. |
| SEQ-26 | `*(**FF2a — voxel-language far field: SHIPPED** 2026-07-19, journal/0023` | **POINTER** | FF2a shipped; the FF2b paragraph attached to it is superseded by the octree entry's *"This supersedes FF2b's earlier pairing with the caves/underground water thread"* + `FF2b-minimal LANDED` (journal/0070). 11 lines. |
| SEQ-27 | `*(**Erodibility coupling — lithology-aware erosion: SHIPPED**` | **MECHANICAL** | Self-declared SHIPPED, journal/0029. 5 lines. |
| SEQ-28 | section-range `1. ~~**Flipping `production_config`'s `erodibility` to true**~~ **RATIFIED AND` → `makes item 2 below the live question, not a footnote.` | **SHRINK** | Struck in place: `RATIFIED AND DONE 2026-07-20`. **Items 2 and 3, the 2026-07-29 losing-side banner, and the journal/0079 record all STAY** — item 2 is an explicit live user call (*"Whether the axis reopens, and on what terms, is the USER'S call"*). 8 lines. |
| SEQ-29 | section-range `Original charter (for the record):` → `feedback stability clamped; off-by-default and byte-identical.` | **SHRINK** | Self-labelled *"(for the record)"*, describing a gap closed 2026-07-20. 8 lines. |
| SEQ-30 | section-range `1. ~~**The world-creation ritual grows 15 s → 25 s (+66 %)**~~ — **RATIFIED` → `changed for every world created from here.` | **SHRINK** | S10 calls 1 and 2 both struck-and-done (`RATIFIED 2026-07-20`, `DONE 2026-07-20`). Calls 3/4/5 stay live. 8 lines. |
| SEQ-31 | `*(**Collapse-tier organic materials + the production flip: SHIPPED**` | **MECHANICAL** | Self-declared SHIPPED, journal/0026. 5 lines. |
| SEQ-32 | `**Charcoal as an inclusion, not a band** (journal/0026, measured)` | **POINTER** | **Verified false at source @`6eff1c8`:** its two blockers are *"no charcoal material was shipped"* and *"`deep_class` routes a charcoal-tagged unit to its mineral host"*. `crates/dc-core/src/materials/mod.rs:133` defines `CHARCOAL`; `crates/dc-worldgen/src/geology.rs:430` routes `Biofacies::Charcoal => CLASS_ORGANIC_CHARCOAL`; `crates/dc-worldgen/src/fill.rs:419` makes it loose-formed. Shipped by journal/0063 — and § Observed already carries this verification (S6 finding F3). 12 lines. |
| SEQ-33 | `**Water-model design pass** (ratified 2026-07-19, user; field-notebook` | **POINTER** | Superseded by `flow.md` (RATIFIED 2026-07-25) and by `dc:field/head` (journal/0098). The surviving half — *"what this pass still owes is the PRESENT/RUNTIME tier"* — is stated in full in the entry **directly above it**, which stays. Two entries, one thread, the newer one already complete. 9 lines. |

**LIVE in § Sequenced** (no action, ~35 entries): the supersession-banner backlog · the
enumeration-completeness gap · the coal half-thickness defect · corpus addressability · the pass
architecture (head + continuation slot) · the hillslope-operator-fixed entry · `RE-PICK
EROSION_CALIBRATION` · `CALIBRATE THE DEEP-TIME CLOCK` · CoarseField adoption · refinement
primitives · the file-size chore head · draw-domain (a)/(b)/(c) heads · FLOW · Movement 2b's
slice-(a) reroute · Movement 2b · the aggregation window · structure-aware fine expression ·
migrate-every-not-real-field · weathering-is-one-process · geotherm enrichment · the honest
identity surface · genesis passes · the distance pyramid · edited-chunk spill · distance-evict ·
metamorphism · tectonic expression · consume-the-ledger · roughness recalibration · the 5th/6th
far LOD level · forms/partials emission · the tectonics SPIKE · `client_player_pose_set` retire ·
far-field range knobs · sim light · PBR-2 · S10 calls 3–5 · coal rank · 3e-2 · S11 calls 1–4 ·
water.md supersession note · the 3a/3c-2/3d/3e/4/5/6 tail.

---

## 4. § Observed (1,415 lines, 85 entries)

The S6 baseline audit read this section in full and its verdicts were applied on 2026-07-29
morning; **the 17 below are what its own `RES` (likely-resolved) column left standing**, plus
five entries the morning pass annotated `✅ VERIFIED` in place rather than moving. Each was
re-checked here.

| # | Anchor (unique substring) | Class | Evidence |
|---|---|---|---|
| O-1 | `Two declaration defects on the new `dc:deep/weather_inventory` pass` | **MECHANICAL** | Head is `✅ BOTH FIXED — verified at source 2026-07-29` (S6/F2), with `runner.rs:542-544`, `:889`, `:532-535`, `:1454-1456` cited. Nothing owed. 26 lines. |
| O-2 | `` `world_get_contents` reports `dc:air` and `has_contents: true` over `` | **POINTER** | Its single `OWED` — *"a tier flag that can say **unrecorded** as a first-class answer"* — shipped as `Identity::Unrecorded` (journal/0101). CLAUDE.md § Agent walks carries it: *"`has_contents` is now a PER-VOXEL fact and is trustworthy (fixed 2026-07-25, journal/0101)"*. 25 lines. |
| O-3 | `HOLES IN THE GROUND: FIXED** 2026-07-21, journal/0057` | **MECHANICAL** | `FIXED` + `✅ WALKED AND CONFIRMED` (S6/F1, applied 2026-07-29). Both halves closed in its own body. 23 lines. |
| O-4 | `HOLES IN THE GROUND — partial voxels are missing side faces` | **MECHANICAL** | Its own header ends `(FIXED — see above)`; the "above" is O-3, which moves with it. 19 lines. |
| O-5 | `"Surface material is quantized per chunk": FIXED** 2026-07-21,` | **MECHANICAL** | `FIXED` + `✅ WALKED AND CONFIRMED`. 14 lines. |
| O-6 | `` GPU DeviceLost crash under a teleport storm at `--horizon 6` `` | **POINTER** | Refuted by IF-4 (journal/0065): storm *and* idle re-measured at horizon 6, *"All runs exited **0**, no `DeviceLost`, no panic, no `ERROR`"*, and the root cause (host-RAM exhaustion) fixed by journal/0051. Its secondary ask — *"a DeviceLost should not cascade into unwrap panics"* — shipped as journal/0054's honest exit codes (CLAUDE.md § Agent walks: *"the old 'exit codes lie about GPU crashes' warning is retired"*). **Moves with IF-4.** 16 lines. |
| O-7 | `` Stub inventory filed** (`docs/design/stubs.md`, 2026-07-21, read-only audit `` | **POINTER** | Its *"one genuine discovery"* is `✅ VOID` in place (`ruin_posts` deleted); the residual `field.rs` doc-comment flag is owned by the § Sequenced `METAMORPHISM — the grade axis` entry, which names the same two planes. 13 lines. |
| O-8 | `` `--fullbright` is blind to geometry, and that cost a walk its conclusion `` | **POINTER** | Both filed proposals shipped as journal/0031: `crates/dc-client/src/edgepass.rs` and `shaders/edges.wgsl` exist @`6eff1c8`, and CLAUDE.md § Agent walks teaches the resulting doctrine. 25 lines. |
| O-9 | `` INSTRUMENT: `--fullbright` is BLIND TO SHAPE `` | **POINTER** | Same: its *"proposed fix, filed not built"* is `--edges`, shipped. The doctrine it earned is CLAUDE.md read-first material (*"Pick the control that can SEE your question"*). 15 lines. |
| O-10 | `` INSTRUMENT: `--fullbright` does not disable distance fog `` | **POINTER** | Fixed journal/0031; CLAUDE.md states it: *"Fullbright also no longer applies distance fog (0031), so long-vista silhouettes are readable."* 12 lines. |
| O-11 | `No pooling/reuse of chunk or far-tile GPU resources` | **POINTER** | Answered, and the answer is *"deliberately not built"* — the § Observed entry `Mesh-buffer pooling: measured, deliberately NOT built` (journal/0051) is **the numbered answer to this exact user question** and is decisive on `Mesh::insert_attribute` taking ownership. That entry stays live. The two never referenced each other. 15 lines. |
| O-12 | `Walk 7 loose ends (journal/0008): **unloaded-neighbour and far-mesh` | **POINTER** | Its subject — the S1 phantom old world — is gone by the far-mesh entry's own text (*"the phantom old world ~1 km down is gone"*, journal/0022), which stays live for its unrelated `TerrainGen` seal. 11 lines. |
| O-13 | `The embedded HostWorld never evicts chunks` | **MECHANICAL** | Struck in place with `✅ IT EVICTS — verified at source 2026-07-29` (`host.rs:524`). Kept 2026-07-29 only so archiving its refutation would not leave the claim alone; both are now archived together. 13 lines. |
| O-14 | `Site cap 240 (u8 RegionId)` | **MECHANICAL** | Struck in place with `✅ THE SUBJECT IS GONE — verified 2026-07-29`. 8 lines. |
| O-15 | `We cannot see where runtime goes — the perf observability gap` | **MECHANICAL** | Struck in place with `✅ ANSWERED AS POSED — verified at source 2026-07-29` (`perf.rs:26,40,68-69`). Its live residual is its own § Observed entry (`The perf instrument can't show the frame-thread envelope`), which stays. 24 lines. |
| O-16 | `Chunk gen time is now noticeable in vertical streaming` | **POINTER** | The suspect it sharpened to (`LOAD_BUDGET_PER_FRAME`, synchronous main-schedule gen) was addressed by the 0083/0084 offload. Its *outcome* is the live § Observed entry `Perf: throughput ceiling at terminal velocity` — *"the drop reaches the choking point later … but did not raise the ceiling"* — which stays. Two entries, one thread, no cross-reference. 20 lines. |
| O-17 | `` The `history.rs` reject-don't-crash skip is SILENT `` | **MECHANICAL** | Struck in place with `✅ THE FILE IS DELETED — verified 2026-07-29`. The doctrine it invoked is explicitly noted as untouched. 13 lines. |

**Deliberately LEFT LIVE though the S6 audit marked them resolved-adjacent:**

- `` Charcoal's premise expired and the code still encodes the conclusion `` — the entry's own
  2026-07-29 annotation says *"The entry's wider point … **stands and is the reason it is kept
  rather than archived**."* Honouring that.
- `` Far mesh generated from S1 `TerrainGen` under the worldgen authority `` — `✅ VERIFIED STILL
  OPEN 2026-07-29` (`crates/dc-client/src/worldgen.rs:27` is still `pub struct TerrainGen`).
- `` Walk 14 (journal/0019 § walk 14) `` and `` Placeholder texture tiling: cleanup pass DONE `` —
  both resolved *and* both carry a still-owed photograph. A photo debt is a live debt.
- `` `world_scan_region` / `world_get_block` are structurally blind to stratigraphy `` — structural
  and deliberate; the doctrine is in CLAUDE.md and the `identify(pos)` arc owns the heir.

---

## 5. STRIKE-CANDIDATES — user ratification required, NOTHING here moves

Five items. Each is user-owned (a field report, an appearance ratification, or a content call).
The script **excludes all five**. One evidence line each, as asked.

1. **`**⚠ UNRATIFIED APPEARANCE CHANGE AWAITING THE USER'S EYE (2026-07-21):**` (§ In flight, ~35
   lines including its tour map).** Filed against carry-`H` (journal/0053). *Evidence:* the four
   station `H` values it quotes (`10.66 m`, `7.99 m`, `80.49 m`, `0.17 m`) and its headline claim
   *"the world got **deeper on average rather than barer**"* were measured before journal/0122,
   which moved the shipped world's mean regolith **4.57 → 3.91 m** and its relief −0.9 m. The
   ratification is eight days stale and the tour map addresses a world that no longer exists.
   **The question for the user: re-shoot the tour against today's world, or withdraw the
   ratification as overtaken?**

2. **`**WALK THE NEW WORLD — the biggest unratified appearance change this` (§ In flight,
   ~9 lines).** *Evidence:* same date, same two merges (0053 carry-`H`, 0055 record-skinned
   surface), same superseding fact. It also says *"Both are revertible: one merge commit each"* —
   which stopped being true across ~70 merges.

3. **`Material placement rules are climate mocks, and below ~460 m there is no` (§ Observed,
   17 lines).** *Evidence:* its named mechanism is `collapse.rs::surface_sample` picking
   *"Grass/Dirt/Stone from year-zero climate"* — and the user's own 2026-07-29 strike ruling on a
   sibling entry reads *"grass and dirt are not generated in the current shape of the default
   plugin pack, so the observation itself is stale"* (`ROADMAP-history.md` § Observed — archived).
   Its two other halves are re-homed (`exhum`/`t_crust` → the METAMORPHISM entry; form-from-
   provenance → consume-the-ledger (c)). **Same premise, same ruling — but it is the user's to
   extend, not a sweeper's.**

4. **`Circulation is fidelity-correct but surface-invisible` (§ Observed, 12 lines).**
   *Evidence:* its entire observation is *"it renders as grass"* via `collapse.rs`'s
   `precip < 0.10` bare threshold — the same generation path the user's 2026-07-29 ruling calls
   stale. It is also explicitly labelled `USER-OWNED appearance`. **Note the honest counterweight:
   its `> blogworthy:` line — *"a climate the map can't see"* — is worth preserving wherever it
   lands.**

5. **`2. **Ores R1–R8** (ores.md draft).` (§ In flight, 1 line) — and its companion
   `*(**Ore design pass: DRAFT LANDED** 2026-07-21`.** *Evidence:* the 2026-07-28 ruling
   (§ Sequenced) **rejected a load-bearing premise of the whole doc** — *"we do not need to have
   ore 'exposed' … kills every 'illegible until exhumation increases' caveat, the lode-gold A/B
   fork, and `probe 3`"* — and recorded that *"`ores.md` is conceptually behind `materials.md` /
   `material-behavior.md`, which win on disagreement; a revisit is owed and unscheduled."*
   **Whether R1–R8 still constitutes the user's open fork, or whether the fork is now "re-run the
   ore design pass", is the user's call.**

---

## 6. What this section says about the board, and what to avoid repeating

**The § Observed lesson recurs one section over, at larger scale.** The S6 audit's finding was
*closure-in-the-wrong-place* — a claim and its own refutation in one file, never reconciled.
§ Sequenced and § In flight have exactly the same shape and it is **worse**, because the two
halves sit in *different sections*:

- `Collapse-cache `evict()` is unreachable` (§ Sequenced) vs *"the journal/0050 collapse-cache
  `evict()` gap (now reachable from …)"* (§ In flight) — **2,500 lines apart, eight days.**
- `Erosion-supply calibration … compare model denudation against real orogen rates`
  (§ Sequenced) vs `CALIBRATE THE DEEP-TIME CLOCK` (§ Sequenced, 1,300 lines above), which **did
  that comparison** and is the largest measurement on the board.
- `No pooling/reuse of chunk or far-tile GPU resources` (§ Observed) vs `Mesh-buffer pooling:
  measured, deliberately NOT built — the user asked for pooling; **this is the numbered
  answer**` (§ Observed, 400 lines above).
- `Chunk gen time is now noticeable` vs `Perf: throughput ceiling at terminal velocity` — the
  second is the *measured outcome* of fixing the first, and neither names the other.
- **And once inside a single entry:** the forms/partials contract's `NEXT SLICE` is `BLOCKED` on a
  decision recorded `DONE` 150 lines later in the same section.

**The transferable rule is unchanged and it is not about volume:** discharge the old entry in the
same commit as the new fact. Every instance above would have cost one line at the moment the fact
landed, and instead cost a full-board re-read.

**One thing § Sequenced does well and should be said:** every arc that used a **CONTINUATION
SLOT** (FLOW, the identity surface, genesis passes, the pass architecture) survived its own first
slice shipping without losing what the slice was a slice *of*. That is the slice-of convention
working exactly as designed — and it is precisely why those entries are LIVE here rather than
archivable, which is the correct outcome.
