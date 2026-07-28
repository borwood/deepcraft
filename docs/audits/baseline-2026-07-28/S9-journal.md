# S9 — the journal (`journal/0000`–`0121`)

**Baseline `doc-topology` sweep, slice 9 of 9. Watermark commit `f652b60`.**
**Every quotation below was read at `f652b60`** (doc-topology § Rules, corrections #67).

> **Governing constraint, restated because it decides what is *not* here.** A journal entry
> is *dated narrative testimony*. It is never "stale," and an entry later demolished is the
> journal working correctly. Wrong turns, abandoned hypotheses and `> blogworthy:` lines are
> the journal's **charter**, not defects. This slice reports only:
> **(1)** a journal conclusion **cited as live authority** by a current doc where a later
> journal or correction overturned it; **(2)** an owed item or prediction that lives **only**
> in a journal; **(3)** two journal entries that contradict with **neither marked**.
> Nothing here proposes editing a journal entry. Journals are append-only history.

---

## 0. Coverage — say how much was actually read

| | |
|---|---|
| Entries present | **120** — `0000`–`0121` with **two numbers absent** (§ 5) |
| Read at framing depth (title, dateline, opening thesis) | **120 / 120** |
| Read at conclusion depth (final section, ~14 lines) | **0020–0079** (60 entries) + `0090`–`0121` selectively |
| Read in **full or near-full** | `0079`, `0111`, `0114`, `0116` (tail), `0101` (tail), `0104` (tail), `0110` (tail), `0113` (tail), `0118` (tail), `0006`, `0011`, `0032`, `0037`, `0038`, `0060`, `0062`, `0087` — **17** |
| Approx. journal lines read | **~4,600 of ~23,400** (targeted, as briefed), plus **9 whole-corpus greps** over all 120 entries |
| Live docs opened for the other side of a pair | `ROADMAP.md`, `ROADMAP-history.md`, `CLAUDE.md`, `docs/design/earth-processes.md`, `docs/design/stubs.md`, `docs/design/material-behavior.md`, `docs/design/geology.md`, `docs/design/flow.md`, `docs/spines.md`, `crates/dc-worldgen/src/deeptime/field.rs` |
| **Deliberately NOT opened** | **`journal/corrections.md`** (a sibling slice owns it — greps only, never read), `docs/audits/*` beyond filename listing, `docs/spikes/*` beyond two grep hits |

**The corpus is in much better repair than the sweep expected.** Fourteen candidate
"orphaned owed items" were checked and **twelve were already discharged** in `ROADMAP.md`,
`ROADMAP-history.md` or `stubs.md` — including several the entry itself said it was *not*
filing. That is a real result and it changes how the findings below should be weighted:
**this is not a corpus with a filing problem. It has an authority problem in exactly one
place — the erosion axis, where a 2026-07-23 conclusion survived the 2026-07-26
recalibration untouched.**

---

## 1. Shape 1 — a journal conclusion cited as live authority, overturned later

### 🔴 S9-1 — THE HEADLINE. `journal/0079`'s erosion mechanism is asserted as RATIFIED and CLOSED on the live board; `journal/0111` measured the opposite three days later and never cites it

| | |
|---|---|
| **Blast radius** | **Highest in this slice.** `ROADMAP.md`'s live board, in a `NEEDS RATIFICATION (user-owned)` block, closing an axis with the sentence *"Nothing further to ratify."* |
| **Provenance** | Both sides assistant-originated measurements; the *ratification* of 0079 is user-facing. |
| **Recommendation (labelled)** | Do **not** strike 0079's block — its null is empirically reproducible on the shipped world. **Add a banner** pointing at `journal/0111` + `journal/0114` + `stubs.md` § 27, and withdraw the clause *"Nothing further to ratify on the erosion axis."* Which side wins the *mechanism* is a measurement question already answered; whether the axis reopens is a **user call**. |

**Side A — live, unbannered, on the board:**

`ROADMAP.md:2513-2533` (`f652b60`), inside § *NEEDS RATIFICATION (user-owned — this CHANGES TERRAIN SHAPE)*:

> **"ANSWERED — NOT BY A WALK — 2026-07-23 (journal/0079; RATIFIED). The erosion budget is
> not the amplitude lever."** … *"A mechanism probe pinned **why**: the surface is **graded to
> base level** — mean lowering **41 m (max 1.4 km), flat across the whole sweep** — so it is
> **neither supply-limited** (it erodes a lot) nor iteration-starved"* … *"**relief is
> bottlenecked on the GENERATING side — deep-field elevation structure + uplift — not
> erosion.** Cranking erosion is pushing on a rope."* … *"**Nothing further to ratify on the
> erosion axis.**"*

Its source, `journal/0079:46-70` — § *The mechanism: equilibrium, not starvation* — offers
exactly two candidate mechanisms and picks one:

> *"**(a)** the surface is already graded to base level … **(b)** the surface is
> supply-limited … Decisively **(a)**."*

**Side B — the later journal, which never names 0079:**

`journal/0111:131-134`:

> *"**98 % of every metre of bedrock this world detaches leaves the land system.** Nothing is
> piling up. **At the shipped calibration the landscape is supply-limited** — the weathering
> constant *is* the denudation rate."*

`journal/0111:207-210`:

> *"**Denudation / uplift = 0.027.** In topographic steady state that ratio is 1. … the
> landscape's shape is 97 % tectonic. **Erosion has essentially no authority over the
> topography of this world.**"*

`journal/0111:256-260`:

> *"`erosion_budget` scales `weathering`, `k_transport` and `k_bedrock`. It does **not** scale
> `diffusion`. … **A knob that cannot move the thing it is named after** — corrections #56,
> stubs #24."*

**Why this is a finding and not a supersession the journal handled correctly.** Three
separate load-bearing halves of the ROADMAP block are now known false, and **none of them
carries a marker**:

1. **"graded to base level … neither supply-limited"** → `journal/0111` measured the world
   as **supply-limited**, by the same word, with a boundary-flux ledger and a per-cell
   rock-removal plane that agree to 2.4 %.
2. **"the rate only sets approach-to-grade"** (`ROADMAP.md:2521`) → `journal/0111:224-242`'s
   sweep: budget 100× **+ creep 10×** buys **132×**, *"59× more than their separate gains
   multiplied."* The rate is not an approach-speed; the **knob was blind to the process doing
   96 % of the work.** 0079's null was an artifact of the instrument's reach, not of grade.
3. **"relief is bottlenecked on the GENERATING side — not erosion"** → `journal/0114:100-106`
   measures relief **+4.6 % at 45×, +18 % at 100×, +52 % at 300×**. Erosion moves relief; it
   was simply never allowed to.

**And there is a second live site with the same content and no banner:**
`ROADMAP-history.md:2686-2694` —

> *"a mechanism probe pinned it to erosional **equilibrium** (graded to base level; mean
> lowering 41 m, flat). The amplitude lever is the deep-field relief GENERATOR, not erosion
> rate"*

Archived, so lower blast radius — but read-first item 5's doctrine (*"a one-directional
pointer is not a pointer… the stale end is exactly where a cold session enters"*) applies:
`journal/0114` and `stubs.md` § 27 both know; neither is reachable from here.

**The sharpest way to state it, because it is this project's own rule turned on itself:**
`journal/0079:29` opens by invoking the journal/0030 trap — *"a null from an instrument that
cannot see the question is worth nothing"* — and then spends a whole section proving the probe
faithful to the **launch path**. It was faithful to the launch path. It was **blind to
`diffusion`**, which is the term that carries 96 % of the export. CLAUDE.md's own *"corollary
for reading a null"* (added for `journal/0110`) fits `journal/0079` word for word and has
never been applied to it.

---

### 🟠 S9-2 — `earth-processes.md` still sequences a wave retune that the user STRUCK the same day, both sides citing `journal/0049`

| | |
|---|---|
| **Blast radius** | High. `docs/design/earth-processes.md` is read-first tier (CLAUDE.md item 3) and is the ratification of record for the erosion-agent roster. |
| **Provenance** | **The striking side is user-originated and quoted verbatim.** Not a tie (doc-topology § Rules, *provenance decides weight*). |
| **Recommendation (labelled)** | Strike the *"retune slice Sequenced"* clause in `earth-processes.md` and replace it with the user's own sentence + a pointer to `stubs.md` § 28. The heir-mechanism sentence beneath it already agrees and survives. |

**Side A — the design doc, unstruck:**

`docs/design/earth-processes.md:406-408` (`f652b60`), inside *"Magnitude ratification — DECIDED 2026-07-21 (user, live guided tour, journal/0049)"*:

> *"the user directed 'more dramatic by default' — **retune slice Sequenced** (target class:
> cuts that survive the 0.9 m voxel and read as platforms/notches, **~10–40×** with
> `wave_band_m` widened for stranded terraces from the sea-level cycles)."*

**Side B — the ROADMAP, quoting the user:**

`ROADMAP.md:2399-2405`:

> *"**Wave-magnitude retune — STRUCK 2026-07-21 (user): no retune.** 'That whole mechanism
> changes after water machinery. that would be a **bandaid**, against our standing rule
> against bandaids. can revisit later.' The confirmed null (0.68 m, journal/0049 station 5)
> rides as-built until the fetch model … **wave expression is a consumer of the water design
> pass now, not a tuning slice.**"*

Same day, same journal cited by both. The design doc still names a sequenced slice, a target
class and a numeric target (`~10–40×`) that the user explicitly refused as a bandaid. This is
also the exact shape [[no-bandaid-tuning]] exists to prevent, sitting in the doc that owns
the decision.

---

### 🟠 S9-3 — the same `earth-processes.md` block ratifies appearance magnitudes that `journal/0114` measured as 45× weaker relative to the landscape; only `stubs.md` knows

| | |
|---|---|
| **Blast radius** | High, and it is a **one-directional pointer** in exactly the shape read-first item 5 names. |
| **Provenance** | The ratification is **user-originated** (live guided tour). The finding that undermines it is a measurement. **This one must go to the user, not be reconciled by an agent.** |
| **Recommendation (labelled)** | Add a back-pointer in `earth-processes.md` to `stubs.md` § 28 and `journal/0114`. **Do not restate or reinterpret the user's ratification** — flag it and stop. |

`docs/design/earth-processes.md:398-402`:

> *"**Magnitude ratification — DECIDED 2026-07-21 (user, live guided tour, journal/0049).**
> The wind and frost magnitudes are **RATIFIED as-built** … judged **live at their strongest
> stations**."*

`docs/design/stubs.md:984-989` (§ 28, *the-agent-magnitudes-the-calibration-left-behind*, added by `journal/0114`):

> *"**But the consequence is real and is not recorded anywhere else.** The four core rates
> moved 45× and these three did not, so **relative to the landscape they act on, the wave,
> wind and frost agents are now 45× weaker than the day their magnitudes were chosen.**"*

`stubs.md` says the consequence *"is not recorded anywhere else"* — correct, and the doc that
carries the ratification is the one place it is missing. `ROADMAP.md:5039-5040` also knows
(*"the live-magnitudes tour is more owed than before (stubs #28…)"*). Three artifacts hold
three pieces; the ratification site holds none of them.

*Qualification, stated because it decides urgency:* `EROSION_CALIBRATION = 45` is behind
`calibrated_rates`, **off in production** (`crates/dc-worldgen/src/deeptime/field.rs:104-128`,
read at `f652b60`). So the 45× relative weakening is a property of the *calibrated* world, not
today's shipped one. That makes this a **pre-flip blocker**, not a live defect — which is
precisely why it should be visible at the ratification site before the flag flips.

---

### 🟡 S9-4 — `ROADMAP.md` asserts `journal/0120` is a deliberate gap; `journal/0120` exists and says so itself

| | |
|---|---|
| **Blast radius** | Low content-wise, **but it is in the live wrap block** and it is the exact claim my brief was asked to verify — i.e. it propagated into an agent brief. |
| **Recommendation (labelled)** | One-line fix: `0117` remains a deliberate gap; `0120` was written 2026-07-28. |

`ROADMAP.md:5045-5046`:

> *"**journal/0117 and 0120 are deliberate gaps** — the rename slice and the sweep each judged
> a narrative entry unwarranted and said so. **Not lost entries.**"*

`journal/0120-the-ruling-the-sweep-was-holding-a-number-for.md:1-3, 32` — the entry exists,
dated **2026-07-28**, and its own § opening records the collision:

> *"Two days later the wrap block repeated it — *'journal/0117 and 0120 are deliberate
> gaps'*…"* — and its dateline: *"the entry number was reserved four days early by a sweep
> that declined to write a narrative, **on the explicit condition that 'if #1 goes to the user
> and produces a ruling, that is the entry.'** It did, and this is it."*

The condition fired. The board never re-read the condition.

---

### Cross-slice referrals (found here, owned by a sibling slice)

These are **not** journal findings — they are design-doc contradictions I tripped over while
tracing a journal claim. Handing them over rather than adjudicating:

- **`docs/design/geology.md:30-32`** still asserts, unstruck, *"Declared reads/writes let the
  pipeline **topo-sort** passes and detect cycles — the … coupling-order problem becomes a
  graph problem **instead of a hand-maintained list**."* `ARCHITECTURE.md:536-541` and
  `material-behavior.md:342-353` both record the opposite as DECIDED 2026-07-26 (corrections
  #65, `journal/0119`): *"**The order does not fall out of the declarations; it is fed into
  them.**"* → **design-doc slice.**
- **`docs/design/material-behavior.md:369-370`** — RATE is *"composed with **topo-sort**, never
  replaced by it"*, un-struck residue in a bullet whose sibling bullet 25 lines up is marked
  `🔴 SUPERSEDED` on precisely that word. → **design-doc slice.**
- **`crates/dc-worldgen/examples/mfd_probe.rs:372`** prints `"<- single-receiver D8 (p -> inf)"`
  — the caption corrections #58 falsified and `journal/0113` struck *"at all three sites."*
  Fourth site, in a probe. CLAUDE.md § Gates: *"a printed caption is a published claim the
  gate cannot check."* → **spine-audit.**
- `journal/corrections.md:1360` carries `journal/0079`'s *"graded to base level"* mechanism as
  correction #41. **Not read** (sibling slice owns the file); flagged from a grep hit only, so
  the corrections slice can decide whether #41 needs the S9-1 banner.

---

## 2. Shape 2 — owed items / predictions living **only** in a journal

**Fourteen candidates traced; twelve already discharged elsewhere.** The discharged set is
listed because a null from this sweep is suspicious and the reader deserves the denominator:
0006 chasm terracing (`ROADMAP.md:4525`) · 0011 class-presence quantization
(`corrections.md:100`) · 0022/0023 tile-edge stitch lines (`ROADMAP.md:4300`) · 0032
value-level completion (`ROADMAP.md:3931`) · 0036 biharmonic flexure (`tectonics.md:791`) ·
0038 circulation-not-legible (`ROADMAP.md:4798` — the entry said it was *not* filing it; someone
filed it) · 0060/0063 `is_identity` at ten slots (`ROADMAP-history.md:1197-1205`) · 0060
`P_FRESH` (`ROADMAP-history.md:1393`) · 0062 natural sill (`ROADMAP-history.md:1305`) · 0062
absent-chunk-is-UNKNOWN (`ROADMAP.md:470-471`) · 0087 ~80-site solidity drain
(`ROADMAP.md:68, 129`) · 0104 both `reads_prev` halves (`ROADMAP.md:2961, 2971`).

**Count of undischarged owed items found only in a journal: 3.**

| # | Journal site (`f652b60`) | What was named | Searched | Candidate destination |
|---|---|---|---|---|
| **O-1** | `journal/0116:263-274` | *"`walk_tour_0115.rs` is now **1,199 lines**, well past the provisional 700-line threshold … Splitting would mean either copying the census (**anti-shape A-1** …) or moving it into `src/`, which this brief put out of scope. **Recorded as an open extraction, not resolved.**"* | `grep -rn "walk_tour_0115\|open extraction\|1,199"` over `ROADMAP.md`, `ROADMAP-history.md`, `docs/**/*.md` → only hit is `stubs.md:1117`, which cites the file as a **measuring instrument**, not as an open extraction | **ROADMAP Owed.** It is a named anti-shape risk under a hook the project just shipped ([[file-size-is-context]]), and the entry explicitly declines to resolve it. |
| **O-2** | `journal/0038:155-157` (mechanism at `:48-56`) | *"The 720p console-panel overflow (help scrollback pushes the input/signature line off the bottom of the 45%-height panel) is an **appearance loose end** for whoever tunes the console layout."* Filed under a heading that says out loud: *"**Owed follow-ups (not filed here — this entry only)**"* | `grep -i "scrollback\|panel overflow\|720p"` over the whole `.md` corpus → **zero** hits outside `journal/` | **ROADMAP Observed.** Its sibling bullet under the same heading *was* lifted (see 0038 above); this one was missed. |
| **O-3** | `journal/0037:196-198` | *"The one **appearance call left open** is the small poleward-flat wettening (the ~60° storm-track floor, **Δ ≈ +0.05**) — whether the high-latitude interior should read that green — and it **rides as built pending that look**."* | `grep -i "wettening\|storm-track\|poleward"` over the whole `.md` corpus → **zero** hits outside `journal/` | **ROADMAP Observed / a walk station.** An explicitly user-owned appearance call with a number attached, parked in narrative only. |

*Note on O-2 and O-3: both are appearance-class and both are small. They are reported not
because they are expensive but because they are the exact species CLAUDE.md read-first item 6
was written for — "an unlisted loose end is the defect, not a licence" — and because
`journal/0038` **wrote a heading announcing it was leaving them unfiled**, which is the
cheapest possible detection signal and it was still missed.*

---

## 3. Shape 3 — two journal entries whose conclusions contradict, neither marked

**One pair. Reported, not adjudicated.**

### P-1 — `journal/0079` vs `journal/0111`, on the same word

| | |
|---|---|
| `journal/0079:49-62` | *"**(b)** the surface is **supply-limited** — uplift dominates and there is almost nothing to erode. … **Decisively (a)** [graded to base level]. The landscape erodes a *lot* … so it is **emphatically not supply-limited.**"* |
| `journal/0111:131-134` | *"At the shipped calibration the landscape is **supply-limited** — the weathering constant *is* the denudation rate."* |

Neither entry names the other. `journal/0111` cites `journal/0110` and `journal/0109` in its
own § *What this entry is really about*, and reaches back to `earth-processes.md § 3e`'s owed
list — but never to `0079`, which is the entry that had already asked its question and
answered it the other way. `journal/0114` then corrects `0111` in turn (*"the rates were never
the binding constraint"*), and also does not cite `0079`.

**I am not adjudicating this** — the two may well be describing different quantities
(0079's "supply" is *material available to incise*; 0111's is *the weathering constant capping
export*), in which case what is missing is the sentence saying so. That sentence does not
exist anywhere in the corpus, and until it does, `ROADMAP.md:2519` is quoting the losing side
as settled.

**Everything else checked came back marked.** For the record, because a null needs its
pathspec: `0022→0023` (far-field TIN → voxel language, named in 0023's dateline) · `0090`'s
self-refutation (→ corrections #65, `journal/0119`, banner in `material-behavior.md:342-353`) ·
`0099`'s +3.7 % estimator (→ `journal/0103`, `ROADMAP.md:3182-3196`) · `0100`'s dense-offset
carve-out (→ `journal/0102`, `ROADMAP.md:3130`) · `0109`'s `p → ∞` claim (→ corrections #58,
struck **in-entry** at `journal/0109:115`) · `0110`'s facies null (→ `journal/0111`,
`journal/0114`, `CLAUDE.md` § *corollary for reading a null*) · `0028`'s 415 ms capacity scan
(→ `journal/0062`, `S15-results.md:12`, `water.md:612`) · `0026`'s abundance census
(→ ⚠ banner at `ROADMAP.md:2687-2691`, corrections #51) · `0115`'s hypothesis (→ `journal/0116`,
which ran it as a test rather than building on it).

*Observation, not a finding:* the journal is **append-only in practice but not in the strict
sense** — `journal/0109:115-116` carries an in-place `~~strikethrough~~` + `**STRUCK —
corrections #58**`. That is the same amend-at-the-claim-site pattern the design docs use, and
it is a good pattern; it is noted only because anyone diffing journal text to detect
supersession will find the superseded sentence **still literally present** (doc-topology
shape 6's dead check, in the journal).

---

## 4. Cross-reference — the most-cited journal numbers, and whether the cited conclusion holds

Counted over `CLAUDE.md`, `ROADMAP.md`, `ROADMAP-history.md`, `docs/`, `.claude/` at
`f652b60` (`grep -ohE 'journal/0[0-9]{3}'`). This is the highest-value artifact in the slice:
these are the **stable pointers** the corpus resolves against.

| Cites | Entry | What the corpus cites it *for* | Status of the cited conclusion |
|---:|---|---|---|
| 44 | `0055` | integrate-then-slice; the `round(Σt/0.9)` doctrine; mixture-table cost | ✅ **Holds, and was re-confirmed under challenge** — `ROADMAP.md:3196, 3225`: *"journal/0055's doctrine is **confirmed, not falsified**."* |
| 30 | `0114` | the joint calibration; `EROSION_CALIBRATION = 45`; stubs #27 the transport ceiling | ✅ Current. Note the calibration ships **OFF** (`field.rs:104-128`) — cite it as a *measurement*, never as the shipped world. |
| 28 | `0121` | the bootstrap-history removal; *"existence is not standing"* | ✅ Current (2026-07-28). |
| 25 | `0097` | **`get_contents` not `scan_region` for material questions**; the band's hard perimeter | ✅ Holds. The instrument rule is in `CLAUDE.md:266`; the hard perimeter was **fixed** by `0099` and `ROADMAP.md:1782` says so. |
| 25 | `0010` | within-voxel speckle vs banding; the dormant partial-height renderer; perf numbers | ✅ Holds; still the live stand-in (`stubs.md` § 12, `ROADMAP.md:1703-1709`). |
| 22 | `0119` | ORDER is authored; corrections #65 | ✅ Current, and the reason `doc-topology` exists. |
| 22 | `0111` | **the ~1000× scale error; "a closed system cannot detect its own scale error"** | ✅ Holds — **and see S9-1: it is the entry that should be banner-linked from `ROADMAP.md:2513`.** Refined (not refuted) by `0114`: *"the rates were never the binding constraint."* |
| 22 | `0106` | `production_field()` was neither the production seed nor extent | ✅ Holds. |
| 21 | `0098` | the head/potential field; `dc:field/head` | ✅ Holds. One caveat lives in corrections #60 re: the ±35 m sea-cycle reading — **corrections slice owns it.** |
| 20 | `0096` | flux on faces; the flux record | ✅ Holds. |
| 19 | `0109` | MFD; the residual rule; *"a tree was never what licensed the traversal"* | ⚠️ **Holds except one clause**: *"`p → ∞` is D8 **exactly**"* is false (corrections #58, `journal/0113`). Struck at 3 of 4 sites; `mfd_probe.rs:372` still prints it. |
| 19 | `0100` | the CSR fact ledger | ✅ Holds; its own carve-out expires per `ROADMAP.md:3130` and that is recorded. |
| 18 | `0090` | the deep-time pass-runner | ⚠️ **The runner holds; its ORDER claim does not.** Every live citation now carries the corrections #65 banner. Do not cite `0090` for order. |
| 18 | `0022` | the first horizon; the far-field socket | ⚠️ **Superseded by design** — `0023` replaced the smooth-TIN far field, `0070` made it volumetric. Cite for the *socket*, never for the *representation*. Marked at both sites. |
| 17 | `0101` | `Identity::Unrecorded`; **`has_contents` is a per-voxel fact** | ✅ Holds; in `CLAUDE.md:271`. |
| 17 | `0053` | carry-`H`; the 0.9 m sieve ate ~75 % of sediment | ✅ Holds (`ROADMAP.md:2354`). |
| 16 | `0116` | the flux limiter is the register, not the clock, not isostasy | ✅ Current. **Carries O-1, the one undischarged owed item.** |
| 16 | `0093` | the geotherm field pass; `dc:field/temperature` | ⚠️ **The pass holds; its coal census does not** — those numbers came from the mis-seeded `production_field()` (`journal/0106`), and `ROADMAP.md:2992-2996` records the fix. |
| 16 | `0051` | chunk-store eviction; RAM flat over 147 jumps | ✅ Holds, and was **re-proven at horizon 6** by `0065` (`ROADMAP.md:383-387`). |
| 16 | `0026` | organic materials; the biotic flip; the abundance census | ⚠️ **Flip holds; the census is historical** — coal is **0 %** on the shipped world (corrections #51). ⚠-bannered at `ROADMAP.md:2687-2691`. |
| 13 | `0110` | the material-aware fluvial load; the facies null | ⚠️ **Honest about its mechanism, wrong about its cause** — `journal/0111`, `journal/0114`, and `CLAUDE.md:334`. Fully marked; the exemplar of the corpus doing this right. |
| 13 | `0060` | the first provider seams; *"granularity is a property of the heir"* | ✅ Holds. |
| 12 | `0108` | the 8-byte `Fact`; `EdgeId::declared` | ✅ Holds; the pager is the reserved continuation (`ROADMAP.md:1934`). |
| 12 | `0040` | tectonic amplitude bought no sub-km relief | ⚠️ **Read with S9-1.** It is the first leg of `0079`'s tripod; `journal/0114` measures relief responding to erosion after all (+18 % at 100×). The *tectonic* half stands; the *"therefore relief is generator-bound"* synthesis does not. |
| 12 | `0030` | **pick the control that can SEE your question** | ✅ Holds; `CLAUDE.md:295`. |
| 12 | `0029` | lithology-aware erosion; erodibility coupling | ✅ Holds. |
| 11 | `0103` | **`test = true` on the example target**; the +3.7 % was the instrument | ✅ Holds; `CLAUDE.md:166`. |
| 11 | `0049` | the first guided tour; wind/frost magnitudes RATIFIED; the wave null | ⚠️ **See S9-2 and S9-3.** The ratification is user-originated and stands *as a decision*; the landscape it was judged against moved (`stubs.md` § 28), and one doc still sequences a retune the user struck. |

---

## 5. Numbering gaps

`journal/` holds **120 numbered entries**, `0000` through `0121`. Two numbers are absent.

| Number | Accounted for? | Evidence |
|---|---|---|
| `0117` | ✅ **Deliberate.** | `ROADMAP.md:5045` — *"the rename slice … judged a narrative entry unwarranted and said so."* |
| `0120` | ✅ **Not a gap at all — the entry exists.** | `journal/0120-the-ruling-the-sweep-was-holding-a-number-for.md`, 266 lines, 2026-07-28. The board's claim that it is a gap is **finding S9-4**. |
| **`0059`** | 🔴 **NOT accounted for anywhere in the corpus.** | See below. |

### 🔴 S9-5 — `journal/0059` is a lost entry, and it left its assets behind

`ls journal/assets/ | grep ^0059` at `f652b60`:

```
0059-dune-field-fullbright-edges.png
0059-loess-margin-fullbright-edges.png
0059-loess-margin-topdown-fullbright.png
```

Three screenshots named, per the journal charter (*"screenshots in
`journal/assets/NNNN-description.png`"*), **for an entry that does not exist.** The walk itself
was real and substantive — `ROADMAP.md:4811-4830` carries it as an Observed bullet:

> *"**Walk 0059 — the holes and the chunk patches are gone; the skin is still two flat
> colours** (2026-07-22, live walk at `--horizon 3 --fullbright --edges`; assets `0059-*`).
> Two of the session-close checklist's five questions answered YES … The third answer is the
> defect: the surface reads as **exactly two flat colours with a hard one-voxel contact** …
> **HELD** pending the seam-first cleanup, at the user's direction."*

So a live walk produced a user-diagnosed defect with a named code locus
(`fill.rs:115`, `collapse.rs:786`), a fix shape, and a HELD status — and **its narrative exists
only as a ROADMAP bullet.** `grep -rn "0059" --include='*.md'` over the whole corpus returns
only this bullet, the assets, and the unrelated seed literal `0x0B0A_57EE_0059`. The wrap block
at `ROADMAP.md:5045` enumerates the deliberate gaps and **does not mention 0059**.

**Recommendation (labelled):** this is a **user call**, not an agent's. Either write the
entry retrospectively from the ROADMAP bullet + the three assets (the `journal/0000`
precedent — *"written by the AI collaborator from living context … because otherwise this
history exists nowhere"*), or add `0059` to the wrap block's deliberate-gap list so the
sequence is closed. **What must not happen is a third sweep re-discovering it**, which is what
the gap-accounting line at `:5045` was written to prevent and did not.

---

## Ranked summary

| Rank | ID | Finding | Blast radius | Side that is user-originated |
|---:|---|---|---|---|
| 1 | **S9-1** | `journal/0079`'s *"graded to base level / not supply-limited / nothing further to ratify"* is live and unbannered at `ROADMAP.md:2513-2533`; `journal/0111:131-134` measured **supply-limited** | live board, `NEEDS RATIFICATION` block | neither (the *ratification* is user-facing) |
| 2 | **S9-5** | `journal/0059` is a lost entry with three orphaned assets and a substantive walk finding that lives only in a ROADMAP bullet | journal integrity + a HELD defect | the diagnosis was **user-originated** |
| 3 | **S9-3** | `earth-processes.md:398-402` ratifies agent magnitudes; `stubs.md:984-989` says they are now 45× weaker and *"not recorded anywhere else"* — no back-pointer | read-first design doc | **the ratification is user-originated → flag, do not reconcile** |
| 4 | **S9-2** | `earth-processes.md:406-408` sequences a wave retune the user struck as a bandaid at `ROADMAP.md:2399-2405` | read-first design doc | **the strike is user-originated** |
| 5 | **S9-4** | `ROADMAP.md:5045` calls `journal/0120` a deliberate gap; the entry exists | live wrap block; propagated into an agent brief | neither |
| 6 | **O-1/2/3** | three owed items living only in journals (`0116` file-size extraction, `0038` console overflow, `0037` poleward wettening) | small, but the exact species read-first item 6 targets | O-3 is a **user-owned appearance call** |
| 7 | **P-1** | `0079` vs `0111` on *supply-limited*, neither marking the other | history; but it is S9-1's engine | — |

**Nothing in this slice was resolved, and no journal entry was edited.**
