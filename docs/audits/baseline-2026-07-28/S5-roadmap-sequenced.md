# S5 — `ROADMAP.md` § In flight + § Sequenced (lines 1–2881)

**Baseline `doc-topology` sweep, slice 5 of 9. Watermark / read commit: `f652b60`.**
Every `file:line` below was read at `f652b60` (per `doc-topology` § Rules, corrections #67 —
a quotation without a revision is not a quotation).

**Read in full:** `ROADMAP.md:1–2881` (the header, § In flight, all of § Sequenced).
**NOT read:** `ROADMAP.md:2882–4880` (§ Observed — another agent's slice).
**Also read, for cross-checking only:** the 2026-07-28 close block (`ROADMAP.md:4881–4978`),
`ROADMAP-history.md`, `docs/design/north-star.md`, `docs/design/stubs.md`, `docs/spines.md` § 3,
`journal/corrections.md` (index + entries #55–#67), `journal/0110`, `/0112`, `/0113`, `/0121`,
`.claude/skills/session-workflow/SKILL.md`, `.claude/skills/spine-audit/SKILL.md`, and the
relevant `crates/` sites.

**This section has never been swept as a unit.** 11 findings. The dominant shape is the one the
brief predicted: **an entry describing as pending or blocked something that has since shipped**
(F2, F3, F4, F5, F6, F8) — six of eleven.

---

## Findings, ranked by blast radius

### F1 — 🔴🔴 § In flight still asserts the trusted/untrusted backend tiering as the model

| side | citation (`f652b60`) | claim |
|---|---|---|
| A | `ROADMAP.md:33-38` | *"**Plugin-first, closed-source-OK, untrusted-third-party safe** via a tiered backend behind ONE authoring shape: native `abi_stable` for trusted/first-party (incl. runtime), WASM sandbox for untrusted (gen-tier…)"* — unstruck, present tense, in the FIRST bullet of § In flight |
| B | `docs/design/north-star.md:420-427` | § Deviations **2** (user, emphatic): *"There is no trusted or untrusted until such a time as we need to…"*; the capability-tiering in § Passes and § Refinement is **"EXPLICITLY NOT THE MODEL and must not shape any design."** |
| B′ | `CLAUDE.md` read-first item 0 (worktree copy) | *"**The trusted/untrusted split and the ABI/WASM backend tiering are DEFERRED** … there is **no difference in permission between native and WASM**, so never justify a core/content placement by trust."* |

**Better supported: B.** User-originated, 2026-07-26, and already propagated into the two
highest-blast-radius docs in the corpus. **A is the paraphrase that outran its source**
(shape 6) — it was true when written 2026-07-23 and was never restruck when § Deviations 2 landed.

**Second half of the same entry:** `ROADMAP.md:46-47` calls the ABI/WASM spike *"the ABI/WASM
boundary research spike … **locks the SDK shape**"*, against `north-star.md:378-379`
(*"**AMENDED — NOT GATING** (user, 2026-07-24, § Deviations #1)"*). **This half is already on
the board as an unactioned residual** — `ROADMAP.md:805-806` — so it is *known*; the tiering half
above is **not**, and is the larger of the two.

**Blast radius:** § In flight is what a cold session reads immediately after the read-first set.
It currently hands the reader a retired trust model as the north star's live shape.
**Recommendation (labelled: assistant's):** strike/mark A the way `ROADMAP.md:27-29` already marks
the *"native field-solvers"* correction two lines above it — the shape is in the file already.

---

### F2 — 🔴🔴 Draw-domain part (a) is briefed as a live user-owned appearance slice over code that no longer exists

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1397-1417`, `:1431-1432` | *"**Still a user-owned appearance slice**…"*, then a live consumer chain: *"the sack roll (`pregen/history.rs:221`) → `abandoned` → `Pregen.sites` → `collapse.rs:1588`'s ruin posts → `Block::Wood` in `generate_chunk`… → meshed"*, and *"(a) **the region-step draw moves every world's ruin posts**"* |
| B | `ROADMAP.md:1078-1079` (same document) | *"**draw-domain part (a) is discharged** by removal rather than conversion"* |
| B′ | `journal/0121`, `journal/corrections.md:2562` (#66) | the bootstrap history content was **removed** 2026-07-28, −713 Rust lines |
| B″ | tree at `f652b60` | `grep -rn "ruin_posts\|Pregen.sites\|pregen/history" crates/ --include=*.rs` → **no matches**. `dc-sim/src/statistical/engine.rs:326-369` still holds the two hand-rolled draws |

**Better supported: B.** Measured, merged, gate-verified.

The residual *work* (converting `engine.rs`'s two draws onto `Draws::of`) is real and unbuilt.
What is false is everything the entry says about its **class**: there are no ruin posts to move,
so it is no longer an appearance slice and is no longer the user's. `ROADMAP.md:1408-1409`
already says the agent-step half *"re-rolls NOTHING… byte-identical housekeeping"*; after the
removal that is now true of **both** halves.

**Blast radius:** this entry is the brief for a slice. Dispatched as written, it sends an agent
looking for `collapse.rs::ruin_posts` and waiting on a user ratification that cannot be owed.

---

### F3 — 🔴 "The refinement tier was never decided" asserted present-tense, after it was decided — and 12 lines from the entry's own record of the decision

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1293-1296` | *"**THE HOLE.** `north-star.md`'s core/plugin boundary enumerates both columns and **the refinement tier is in neither.** It was never decided; it was arrived at *by default*."* |
| B | `docs/design/north-star.md:96` | *"### ✅ THE REFINEMENT TIER — **RESOLVED TO (c), user, 2026-07-28**"* |
| B′ | `ROADMAP.md:558`, `:746-751` (same document) | *"`north-star.md` § *The core/plugin boundary* — **the refinement tier RESOLVED to (c)**"*; *"The user's engine/plugin split **closed** north-star's *'an entire tier absent, never decided, arrived at by default'*"* |

**Better supported: B.** User ruling, 2026-07-28, shipped into `north-star.md`.

Note the entry is **internally** inconsistent as well: `ROADMAP.md:1304-1308` in the *same bullet*
already carries *"DIRECTION — DECIDED 2026-07-26 (user)… option **(c)**"*. So the entry records
the decision twice and still opens by saying it was never made. This is the corrections #65 shape
(a claim and its refutation inside one entry), at 12 lines rather than 400.

**Recommendation (assistant's):** the *contract* half of the hole (`flow.md` § 4 gives the tier a
contract, the ownership document did not know it existed) may still be live; the *"never decided"*
half is not. Separating them is a reading task, not a sweeper's call.

---

### F4 — 🔴 Hybrid `p` is marked "OWED … the nearest-term thing on this arc". It shipped 2026-07-26.

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1561-1569` | *"**🔴 OWED BY MFD, and it is the nearest-term thing on this arc** (2026-07-26, journal/0109): **hybrid `p`**, plus recalibrating `k_bedrock`/`k_transport`."* |
| B | `ROADMAP-history.md:91-137` | *"2026-07-26 — **FLOW (b′): HYBRID `p` — water that stays in its banks** (journal/0113)"*, peak catchment 84 → 298 |
| B′ | `ROADMAP.md:1274-1276` (same document) | *"must land **after** material-aware creep (journal/0112) and **hybrid-`p` (journal/0113)**"* — cites it as a completed predecessor |
| B″ | `crates/dc-worldgen/src/deeptime/grid.rs:562-570` | `mfd_exponent: 1.0` / `mfd_exponent_channel: 16.0` / `mfd_chi_lo` / `mfd_chi_hi`, doc-commented *"The hybrid-`p` law (journal/0113)"* — the law is the shipped default |

**Better supported: B.** In the tree and in the archive.

The `k_bedrock`/`k_transport` recalibration half is **also** at least partly discharged by
journal/0114 (`ROADMAP-history.md:47-90`, `EROSION_CALIBRATION = 45`), which the same ROADMAP
entry acknowledges at `:1235-1238`. So the 🔴 flag on `:1561` is stale on both terms.

---

### F5 — 🔴 Movement 2b continuation (b) (gravity/mass-wasting) is presented as an open user call. It shipped as journal/0112.

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1621-1625` | *"**🔀 THE CONTINUATION ORDER IS REROUTED BY MEASUREMENT, and this is the user's call.** … **(b) the GRAVITY/MASS-WASTING member of § 13.2 — newly promoted to FIRST.** Creep routes this world's sediment and **carries no identity**"* |
| B | `journal/0112` title + subtitle | *"2026-07-26 · **Movement 2b, continuation (b) — material-aware hillslope creep**"* |
| B′ | `ROADMAP.md:1274`, `:5065` | *"after material-aware creep (journal/0112)"*; *"**Material-aware creep (0112)** — provenance in the archive **0.000006 % → 65.206 %**"* |
| B″ | `crates/dc-worldgen/src/deeptime/grid.rs:574` | `material_creep: true` in the default `DeepConfig` |

**Better supported: B.**

**Secondary, and worth surfacing to the integrator:** journal/**0110**, /**0111** and /**0112**
have **no Shipped entry in `ROADMAP-history.md` at all** (`grep -n "0110\|0111\|0112"
ROADMAP-history.md` returns only incidental mentions inside the 0114 entry). The archive's own
design says *"each Shipped entry keeps its journal number as the stable pointer"*
(`ROADMAP.md:10-13`); three arcs' worth of shipped work has no such pointer. **Absence stated with
its pathspec, per the skill's corollary.**

---

### F6 — 🟠 A `NEEDS RATIFICATION` block asking for a decision on a default that already shipped, on a measurement that has since been superseded

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1614-1617` | *"**⚠ NEEDS RATIFICATION (user-owned):** `material_transport` **on by default** (it moves the goldens…); and **`COMPETENCE_SCALE = 420`**, the knob deciding *which grain sizes this world can move at all*, **whose measured answer today is 'mud, sometimes.'**"* |
| B | `crates/dc-worldgen/src/deeptime/grid.rs:572` | `material_transport: true` — already the shipped default |
| B′ | `ROADMAP-history.md:82-84` (journal/0114) | *"sand now moves on **0.095 %** of land where it was zero"* |
| B″ | `journal/corrections.md:2167` (#59) | *"The competence ceiling is fixed by an anchor that already ships, and is not a tuning knob"* — **falsified**; `energy_band`/`competence_ceiling` are now relative to `REFERENCE_KT` |

**Which side wins is genuinely a user call and I am not resolving it.** But the entry asks the
user to ratify (i) a flag that is already on in the shipped default and (ii) a number whose
stated evidence (*"mud, sometimes"*) is no longer the measurement. The ratification may still be
owed; **its supporting text is not current.**

---

### F7 — 🟠 The coal-evidence-base residual is *entirely* a set of pointers, and the pointers no longer resolve

`ROADMAP.md:807-814` is the *"THE COAL EVIDENCE BASE IS LABELLED 'THE PRODUCTION WORLD' IN FOUR
LIVE DOCS"* residual. Checked at `f652b60`:

| cited as | what is actually there at `f652b60` |
|---|---|
| `stubs.md:348-351` / `:360-362` — *"the sentence justifying `COAL_BURIAL_M = 8.0`"* | lines 348-362 are inside **stub #13, the wave-climate stub**. `COAL_BURIAL_M` is at `docs/design/stubs.md:380`; the *"production world"* label is at `:404` |
| `stubs.md:378-379` — *"a user ratification sits directly on top of it"* | lines 378-379 are the **`### 14 (original)` heading** |
| `corrections.md:1164` | inside the **charcoal** correction's *"general shape"* paragraph; the surviving *"production world"* uses are at `journal/corrections.md:1142`, `:1321`, `:1369`, `:2647` |
| `ROADMAP-history.md:1228/1234` | correct in substance (the `COAL_BURIAL_M` = 8 m derivation), though the phrase there is *"in the whole world"* |
| `geology.md:163` | correct in substance |

**This is corrections #67's shape exactly** — citations that were accurate when written and now
address different text, indistinguishable from a paraphrase. The finding itself (*"the label is
wrong with certainty; the correct VALUES are unknown — so this needs a **measurement**, not an
edit"*) is unaffected and still live. **Only the addresses rotted.**

---

### F8 — 🟠 An "integrator fix, no ruling needed" that was already fixed

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:716-718` | *"**`session-workflow/SKILL.md:889-894` still presents archive-by-status as a future proposal** (*'**Moving** older Shipped entries… **would** shrink the live board'*) — it shipped 2026-07-26. Integrator fix, no ruling needed."* |
| B | `.claude/skills/session-workflow/SKILL.md:934` | *"- **Archive by STATUS, not by age. — ✅ SHIPPED 2026-07-26 as `ROADMAP-history.md`**"* |

`SKILL.md:889-894` at `f652b60` is the *"two brief-template clauses"* section — the quoted text is
not there, and `grep -n "would shrink the live board\|Moving.*older Shipped"` over that file
returns **nothing**. Fixed; the ROADMAP row is the only thing left claiming otherwise.

---

### F9 — 🟠 Two competing "what the user owes" lists, and a pointer to a heading that does not exist

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:400-434` (§ In flight) | *"**Decisions waiting on the user (nothing else is blocked on them):**"* — 0. THE SOIL GAP · 2. Ores R1–R8 · 3. Roughness recalibration · 4. `classify` is material-keyed. A 2026-07-21 close-block remnant, unmarked, sitting inside § In flight |
| B | `ROADMAP.md:554-576` (§ Sequenced, first entry) | *"**Consolidated into ONE live entry** because the alternative — leaving them inside a `✅ DONE` block and a notebook — is the buried-in-a-closed-artifact failure this session spent the day measuring."* / *"**Nothing here is owed.**"* |

B is scoped to the five 2026-07-28 calls, so this is not a flat contradiction — but a reader
arriving at the entry that announces itself as *the* home of user-owed decisions will not learn
that four more sit 150 lines above it. **I verified the oldest is still genuinely live:**
`MaterialId::LOAM` appears at `crates/dc-core/src/materials/mod.rs:82` (the const),
`crates/dc-client/src/meshing.rs:1166` (a colour), `contents.rs:497` and
`s8_measurements.rs:327` (tests) — and **in no geology class member**. The soil gap is real,
unresolved, and seven days older than the "consolidated" list.

**Dangling pointer, same family:** `ROADMAP.md:1066` and `:4914` both cite
*"§ Sequenced → **USER DECISIONS OWED**"*. No such heading exists; the entry is titled
*"✅ ALL FIVE USER DECISIONS RULED 2026-07-28"* (`:554`). A reader greping the named string finds
only the two pointers, never the target. *(Note `:1066` calls this pointer "deliberately
reciprocal" — the reciprocity holds by position and fails by name.)*

---

### F10 — 🟠 File-size thresholds: "DECIDED (user)" and "still the hook's guesses", same document, same day

| side | citation | claim |
|---|---|---|
| A | `ROADMAP.md:1345-1348` (§ Sequenced) | *"**✅ CONVENTIONS DECIDED 2026-07-28 (user)** — was *'to be set, not guessed.'* Set from the corpus measurement rather than taste, and **shipped in `scripts/filesize_hook.py`**"* |
| B | `ROADMAP.md:4970` (2026-07-28 close block, § ⚠ Owed / unverified) | *"**File-size thresholds still the hook's guesses.**"* |

`scripts/filesize_hook.py` exists at `f652b60`. **A is better supported** — it names the user, the
date, the derivation and the shipped artifact; B reads as carried forward unchanged from
`ROADMAP.md:5043` (the **superseded** 2026-07-27 close block: *"still the hook's provisional
guesses, not the user's numbers"*), i.e. a close-block line that survived the ruling that retired
it. ⚠ **The B side sits in the close block, outside this slice** — flagged for whoever owns it.

---

### F11 — 🟢 The enumeration-completeness bullet still lists a resolved instance as open

`ROADMAP.md:936-938` (inside the corpus-addressability reading-pass record): *"`north-star.md`'s
two-column core/plugin boundary (**an entire tier absent**, *'never decided, arrived at by
default'*)"* — against `docs/design/north-star.md:96` and `ROADMAP.md:746-751`, which record it
resolved 2026-07-28. **Low blast radius**: `:746-751` is the live entry and already states the
resolution; `:936` is a dated record of a reading pass. Listed for completeness, not for action.

---

## CLAIM INVENTORY — § Sequenced entries: subject / stated status / confirmable

| `ROADMAP.md:` | subject | stated status | confirmed? |
|---|---|---|---|
| 554 | five user decisions from the history removal | ✅ ALL RULED 2026-07-28 | **YES** — journal/0120; north-star.md:96, CLAUDE.md item 5, spines § 3 third exit all present |
| 577 | (1) 102 wood posts removed | ✅ RULED — ACCEPTED | YES (journal/0121; `ruin_posts` absent from tree) |
| 598 | (2) three docs describing civ/history as a pipeline stage | ✅ RULED — SPLIT | YES (CLAUDE.md carries the doctrine) |
| 636 | (3) dc-sim S2 tier has zero production callers | ✅ RULED — KEEP AS CANDIDATE | YES (spines § 3 third exit) |
| 664 | (4) settlement/civ schema | ✅ RULED — KEEP with (3) | YES |
| 678 | (5) spike-doc immutability policy | ✅ RULED — immutable body / mutable header | YES (CLAUDE.md read-first item 5) |
| 711 | `spine-audit` still greps `JUSTIFIED-BY` | owed, main-session call | **YES, still true** — `spine-audit/SKILL.md:47` (check #4) unchanged |
| 716 | session-workflow archive-by-status as a proposal | owed, integrator fix | **NO — already fixed** → **F8** |
| 719 | `approx_resident_bytes` non-monotone in extent | new Observed candidate | not checked (Observed is another slice) |
| 725 | supersession-banner backlog | 🟠 open, deliberately bounded | YES — only `S10`/`S2` stamped |
| 746 | no enumeration checked for completeness | 🟠 recorded, not sequenced as a build | YES |
| 763 | doc-topology residuals (19 findings) | 🟠 partly done | mixed — 3 sub-rows ✅ DONE confirmed; `:805` ABI-spike row and `:807` coal row still open, the latter with rotted addresses (**F7**) |
| 818 | coal candidate counts its own half-thickness | 🟠 defect, blast = nothing today | YES (gated behind `calibrated_rates: false`, `grid.rs:578`) |
| 833 | corpus addressability / versioned standing models | 🔴 booked design conversation; 🟢 reading pass DONE | YES (notebook + evidence docs exist) |
| 936 | (sub-bullet) three enumeration instances | asserted open | **partly stale** → **F11** |
| 968 | doc-topology sweep + ROADMAP archive | ✅ DONE 2026-07-26 | YES |
| 1005 | pass architecture: authored ORDER, open vocabulary, RATE | 🔴🔴🔴 DECIDED, RATE unbuilt | YES — close block still names RATE the top blocker |
| 1047 | bootstrap history removal | ✅ DONE 2026-07-28 | YES (journal/0121, corrections #66) |
| 1091 | (original removal entry, preserved) | superseded framing, kept | n/a — exempt per brief rule 5 |
| 1119 | hillslope conveyor checkerboards the regolith (stubs #29) | 🔴🔴🔴 diagnosed, unfixed, blocking | YES |
| 1224 | transport operator ceiling (stubs #27) | 🔴 measured, unfixed | YES |
| 1235 | calibrate the deep-time clock | 🔴🔴 partly built, deliberately OFF, blocked | YES (`calibrated_rates: false`) |
| 1288 | refinement primitives | 🔴 "the hole… never decided" | **NO — resolved 2026-07-28** → **F3** |
| 1327 | file size is a correctness problem | ✅ conventions decided, hook shipped | YES, but contested by the close block → **F10** |
| 1392 | finish the draw-domain conversion, part (a) | "still a user-owned appearance slice" | **NO — discharged by removal** → **F2** |
| 1418 | draw-domain (b) | ✅ DONE 2026-07-26 | YES (journal/0118) |
| 1428 | draw-domain (c) tag space | open | YES, unchanged |
| 1442 | FLOW arc | open; slice 1 + (a) shipped | YES |
| 1546 | (b′) BVP refinement, (c) void intervals, (d) fluid identity, (e) retire receiver tree | unbuilt | YES |
| 1561 | hybrid `p` + `k_bedrock`/`k_transport` recalibration | 🔴 OWED, "nearest-term" | **NO — shipped** → **F4** |
| 1587 | Movement 2b slice (a) + facies null | 🔴 shipped, arc rerouted | YES |
| 1614 | `material_transport` default / `COMPETENCE_SCALE` | ⚠ NEEDS RATIFICATION | **stale support** → **F6** |
| 1621 | Movement 2b continuation (b) gravity/mass-wasting | promoted to FIRST, "user's call" | **NO — shipped as journal/0112** → **F5** |
| 1626 | continuations (c) Hjulström, (d) eolian load, (e) lineage, (f) solutes | unbuilt | YES |
| 1639 | Movement 2b (the reshaped arc entry) | sequencing DECIDED, build partly done | partly — see F5 |
| 1670 | aggregation window is a declared axis | RATIFIED, sequenced not built | YES — no `Pass` declares a window |
| 1701 | structure-aware fine expression | non-blocking, two scales, honestly statused | YES |
| 1736 | migrate "not real" fields into declared field passes | target list rewritten; `exhum`/`t_crust` survive | YES (spines § 3:1084 row still unconsumed) |
| 1763 | appearance walks owed | (1)…(5) all resolved | YES |
| 1812 | `FactLedger` CSR layout | ✅ DONE 2026-07-25 | YES (journal/0100) |
| 1849 | weathering is one process / saprolite is a state | requisites R1–R3 ✅ MET, design question open | YES |
| 1930 | S20 2c compact fact | ✅ HALF BUILT 2026-07-26 | YES (ROADMAP-history:198) |
| 1934 | S20 option 3, the pager | 🔒 reserved continuation | YES, unbuilt |
| 1982 | weathering front needs a profile | ✅ SHIPPED 2026-07-25 | YES (journal/0099) |
| 2016 | geotherm nonlinear / mantle heat | followup, non-blocking | YES, unbuilt |
| 2023 | honest identity surface / `identify(pos)` | slice 1 ✅ SHIPPED, arc open | YES (journal/0101) |
| 2183 | genesis passes drive rock distribution | arc open; 🔖 open edge, read the notebook | YES |
| 2187 | who owns clastic facies (row C-2) | ⚠ CONTRADICTED, user's call | **partly resolved** — `:2194-2197` records the 2026-07-26 settlement (*genesis = parent, weathering = loosening, transport = destination*); the `⚠ CONTRADICTED` banner at `:2187` and its twin at `:1581` are **unstruck**. Low confidence: reported, not resolved |
| 2243 | distance pyramid / LOD colour cascade | sequenced 2026-07-21 | **COULD NOT CONFIRM** |
| 2277 | edited chunks accumulate — save-layer heir | sequenced behind persistence | YES, unbuilt |
| 2286 | distance-evict re-derivable resident data | a knob for later | YES, unbuilt |
| 2296 | collapse-cache `evict()` unreachable | reassigned to the `collapse.rs` rewrite | **COULD NOT CONFIRM** |
| 2306 | METAMORPHISM — the grade axis | unblocked 2026-07-24, unbuilt | YES — spines § 3 rows `:1084` / `:1087` both still unconsumed |
| 2322 | tectonic expression at the collapse tier | (a) open, (b) moved to `:2306`, (c) now a deletion target | YES |
| 2342 | consume the ledger terms | (a) shipped, (b) moved, (c) priority, (d) resolved | YES |
| 2368 | roughness recalibration A/B/C | user picks from pictures | YES, still open |
| 2393 | a 5th/6th far LOD level | conditional | YES |
| 2399 | wave-magnitude retune | STRUCK — no retune | YES |
| 2406 | forms/partials emission slice | rider on Crux 1 | YES |
| 2414 | tectonics SPIKE | sequenced, VALIDATED-as-field-solver stamp | **COULD NOT CONFIRM** whether dispatched |
| 2438 | erosion-supply calibration | sequenced | **partly superseded** by journal/0114's joint calibration; not marked. Low confidence, reported only |
| 2445 | tectonic uplift-plane redesign | "superseded — done" | YES |
| 2453 | retire `client_player_pose_set` into `dc:character/pose` | sequenced after console/edges merges | **COULD NOT CONFIRM** |
| 2479 | erodibility NEEDS RATIFICATION 1–3 | 1 ✅, 2 ANSWERED 2026-07-23, 3 rides as-built | YES |
| 2549 | far-field range knobs | requested 2026-07-20 | **COULD NOT CONFIRM** |
| 2558 | sim light — design pass done, SPIKE next | sequenced | YES (`docs/design/light.md` exists) |
| 2637 | PBR-2 | DEFERRED by the user | YES |
| 2659 | S10 five open calls | 1 ✅, 2 ✅, 3/4/5 open (3 amended, its numbers flagged historical) | YES |
| 2707 | charcoal as an inclusion | blocked on a mechanism | YES |
| 2720 | coal rank when burial deepens | ⚠ RE-BLOCKED 2026-07-25 | YES |
| 2746 | 3e-2 C refinement | RECONCILED; downstream of FLOW | YES |
| 2785 | S11 four open calls | user-owned, open | YES |
| 2819 | water-model design pass | half superseded by `flow.md` | YES |
| 2841 | 3a / 3c-2 / 3d / 3e / 4–5 / 6 (the old numbered tail) | various | **COULD NOT CONFIRM** (6 entries) |

**Sequenced entries whose status I could NOT confirm either way: 11**
(`:719` — Observed, out of slice · `:2243` · `:2296` · `:2414` · `:2453` · `:2549` ·
plus the six-entry numbered tail at `:2841-2880`).

---

## What I could NOT attribute

1. **Whether `material_transport: true` was ever ratified.** It is the shipped default
   (`grid.rs:572`), journal/0110 moved goldens, and I found **no ratification record** in
   `ROADMAP-history.md`, the close blocks, or journal/0110–0112. Either it shipped default-on
   ahead of its ratification, or the ratification exists somewhere I did not read. **Stated as an
   open question, not a finding.** (Searches run: `grep -rn "material_transport" crates/`;
   `grep -n "0110\|0111\|0112" ROADMAP-history.md`.)
2. **Why journal/0110, /0111 and /0112 have no `ROADMAP-history.md` Shipped entries.** Recorded
   under F5; I cannot tell whether this is an archive omission or a deliberate choice.
3. **The `⚠ CONTRADICTED` C-2 banners** at `:1581` and `:2187`. `:2194-2197` reads as the
   settlement, but does not say so in those words, and the cited authority
   (`docs/audits/2026-07-25-roadmap-staleness-sweep.md` row C-2) is a file I did not open.
   Reported at low confidence in the inventory rather than raised as a finding.
4. **§ Observed (`:2882-4880`) was not opened at all**, per the brief. Several Sequenced entries
   point into it (`:1790` *"See the 🔴 Observed entry"*, `:2735`, `:2274`); those pointers are
   unverified by me.
