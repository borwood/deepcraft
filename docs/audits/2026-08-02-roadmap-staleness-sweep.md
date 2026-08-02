# ROADMAP staleness sweep — 2026-08-02 (FULL)

*Read-only research pass. **No ROADMAP, design-doc, `stubs.md`, `corrections.md` or
`dependency-graph.md` edits were made — the integrator applies.** No cargo was run by this
agent. This file and its watermark key are the only things written.*

**What was swept.** `ROADMAP.md` (In flight · Sequenced · Observed · the 2026-08-02 close
block, 4,427 lines) and `docs/dependency-graph.md`, against everything landed in
`git log 72fbe86..d8407e1` — **128 commits**, journals **0123–0137**, corrections
**#74–#87**, the **P11** arc (design pass + slice 1), the **bodies** arc (B0 + posture-bake
member #0), the **P10** design pass, the **P2** literature derivation, record-terms rulings
1–2, and the build-slot mutex hook.

**Mode: FULL**, per the hook's trigger (125+ commits since watermark, and a recalibration
in the window). See § 0 for the recalibration qualifier, which is the first thing the skill
requires and which changes what the rest of this sweep is allowed to void.

**Expected-red goldens are NOT reported as findings.** The P11 close block
(`ROADMAP.md:4111-4117`) lists the golden families that are deliberately red on main until
slice 2's single re-capture. That is recorded, accepted-as-is state.

---

## 0. THE RECALIBRATION QUESTION — answered first, per the skill

> **`calibrated_rates` is STILL OFF BY DEFAULT. The P2 work that landed in this window is a
> DOCS-ONLY derivation and changed no constant. It voids no prior observation.**

**Evidence, both sides:**

| claim | evidence |
|---|---|
| production ships the flag false | `crates/dc-worldgen/src/deeptime/field.rs:374` — `calibrated_rates: false`, inside `production_config_base`, with the three measured costs of flipping it in the comment above (`:355-373`) |
| the multiplier constant did not move | `EROSION_CALIBRATION: f64 = 45.0` at `field.rs:158`; `git log -S "EROSION_CALIBRATION: f64"` returns **one** commit, `cdd1036`, which predates the watermark |
| the P2 work was docs-only | `git show --stat ac1c952` and `git diff --stat ee9fb94^1 ee9fb94` → **one file changed**, `docs/audits/2026-08-01-p2-calibration-derivation.md`. Zero files under `crates/` |
| the derivation says so itself | `docs/audits/2026-08-01-p2-calibration-derivation.md:566` — *"**Do not flip `calibrated_rates`.** Appearance-class, user-owned"* |
| the ON world stays pinned by name | `crates/dc-worldgen/tests/calibrated_rates.rs:82` `calibrated_rates_off_is_production_and_on_moves_it`; `examples/walk_tour_0115.rs:150-151` still asserts *"production must still ship `calibrated_rates` OFF"* |

So corrections **#68**'s receipt holds for a third consecutive sweep, and the P2 derivation
strengthens rather than weakens it: it is a *bracket* (`M ∈ [60, 240]`) handed to a future
measurement run, not a pick.

### 0b. ⚠ But something else in this window IS enabled by default and DID move the world

**P11 slice 1 is on by default and the shipped terrain moved** — not by a rate change, by
record segmentation. `journal/corrections.md:3634` (#86) is explicit: mass conservation held
bit-for-bit, *"but **terrain moved anyway**, because the record's per-unit float accumulation
changed with the segmentation: a finer unit structure changes the **addends**, not merely
their grouping."*

**This is not the journal/0111 cohort-voider shape and must not be read as one** — no rate,
gradient or yield magnitude was recalibrated, so no *literature-relative* claim moves. What it
does bear on is the **appearance-observation cohort**, and the close block already says so
without calling it that: `ROADMAP.md:4130-4133` — *"the walk debt is now **TWO stacked
appearance changes** (octaves 2026-07-29 + P11 member diversity)"*. Any Observed entry
resting on **what a station looked like** is now two changes behind the world; any entry
resting on a **rate or magnitude** is untouched.

That split is exactly corrections #68's rule applied in the direction it was written for, and
it is why findings **F4** and **F8** below are graded as *mechanism closed / observation
owed a re-measurement* rather than resolved.

---

## 1. FINDINGS, ranked

### F1 — 🔴 CONTRADICTED. The parallel-session ordinal collision was diagnosed and fixed in ONE of three numbering spaces. Two are still colliding at HEAD.

**This is the highest-value finding on the board and it is mechanical to verify.**

At 04:42 on 2026-08-02, commit `3cab931` caught that the geo wrap had assigned journal
ordinal **0135** to the P11 slice-1 entry while the parallel bodies session had already taken
it for B0, and renumbered it to **0136**. Its commit message states the mechanism precisely:

> *"The geo wrap (`dcc6b43`) assigned 'next free' ordinal 0135 at merge without seeing
> `d29eb62` (parallel bodies session, 22:47 the night before) had already taken it."*

**The same wrap, in the same hour, did the same thing in two other numbering spaces, and
neither was checked.**

**(a) `journal/corrections.md` — TWO #85 and TWO #86, all four live:**

| line | entry | authored |
|---|---|---|
| `corrections.md:3512` | #85 *"THE SPLIT AXIS IS LIVENESS, NEVER TOPIC"* | `634742e`, 08-01 **23:03** |
| `corrections.md:3550` | #86 *"It runs at pack build rather than world gen"* | `bc9b612`, 08-02 **00:48** |
| `corrections.md:3598` | #87 *"The bake implementation is dispatched…"* | `45b5014`, 08-02 **01:51** |
| `corrections.md:3617` | **#85** *"The identity swap is a zero-byte change"* | `dcc6b43`, 08-02 **04:17** ← collision |
| `corrections.md:3634` | **#86** *"Identity, not mass — so the terrain is unchanged"* | `dcc6b43`, 08-02 **04:17** ← collision |

Neither duplicate carries the corpus's own `### N (original)` disambiguation idiom (the one
`stubs.md:1236` and `:433` use); both are plain headings, so they are unintentional, and the
two colliding entries sit **after** #87 in the file, which is where a reader scanning by
number will not look.

**The close block asserts the collision as fact:** `ROADMAP.md:4143` — *"Corrections
**#74–76, #84–86** (all targets stamped)"*.

**(b) `docs/design/stubs.md` — TWO #34, and the older one has 13 inbound references:**

| line | stub | authored |
|---|---|---|
| `stubs.md:1463` | 34. `the-binding-key-that-is-a-name-wearing-a-role` | `943978b`, 08-01 **21:15** (bodies session) |
| `stubs.md:1421` | **34.** `the-deep-tiers-content-set-is-hard-wired-to-vanilla` | `164be0f`, 08-02 **04:18** ← collision |

**The P11 slice explicitly did the right thing and the wrap undid it.** Commit `662c84d`
(08-01 22:54) is titled *"P11 slice 1: **stub ordinal left pending (parallel sessions
live)**"* — the correct call, made for the correct reason. Seven hours later `164be0f`
("wrap remainder: close block + **stub #34** + audit header stamp") resolved that pending
ordinal to a number the parallel session had already spent.

**Which meaning the corpus already carries — all thirteen point at `:1463`, none at `:1421`:**
`crates/dc-api/src/bodies.rs:42`, `:111`, `:212`, `:345` · `docs/design/bodies.md:42`,
`:288`, `:305`, `:359` · `docs/design/posture-gait.md:296` ·
`docs/design/ideas.md:731` · `docs/dependency-graph.md:88`, `:89` ·
`journal/0135-the-plan-learns-to-say-what-its-parts-are.md:96` ·
`docs/audits/2026-08-01-body-plan-structure-design.md:20`.
The only reference to the *new* `:1421` stub is the close block's own
`ROADMAP.md:4144-4145` (*"stubs **#31 narrowed, #32–34 added**"*) — and `docs/dependency-graph.md:79`
points at **#33**, which is a third number entirely (see **F3**).

**Why this is ranked first.** `3cab931` is a correction-shaped act under CLAUDE.md read-first
item 5 — it identified a *class* of failure ("a wrap assigns 'next free' from a stale view of
a parallel session") and then repaired exactly **one instance** of it, in the space it
happened to be looking at. The two unrepaired instances are in the two files the project uses
as its append-only **stable pointers**. A duplicate ordinal in `corrections.md` breaks the
one lookup CLAUDE.md read-first item 4 exists for ("check before re-deriving"), and a
duplicate in `stubs.md` breaks the loose-ends lookup (read-first item 6).

**Integrator call, not the sweeper's** — but note the asymmetry that makes it cheap: the
bodies-session entries hold every inbound reference in both files, so renumbering the
**geo-session** entries (corrections #85/#86 → #88/#89; the deep-tier stub → #35) touches
only the close block and the graph row.

---

### F2 — 🔴 STALE / one-directional pointer. P2's literature derivation SHIPPED; the ROADMAP entry that asked for it and the graph row that scheduled it were never stamped.

**The asking entries, both still reading as un-started:**
- `ROADMAP.md:1092-1115` — *"🔴 RE-PICK `EROSION_CALIBRATION` AGAINST THE FIXED OPERATOR"*,
  whose § *How to pick it* says only *"Against the **published literature**… The target is
  the craton denudation band journal/0111 established."*
- `docs/dependency-graph.md:70` — row P2: *"**SEQUENCED.** 45 was fitted to the broken solve
  and inverts under the fixed one"*
- `docs/dependency-graph.md:203` — § 4 item 4: *"**P2 — the calibration re-pick.** Needs a
  literature pass, not an engineering one."*

**The answering work, merged `ee9fb94` on 2026-08-01** —
`docs/audits/2026-08-01-p2-calibration-derivation.md`:
- **The target is derived twice from independent inputs and they agree to 5 %**: the world's
  own Airy fixed point (2.63 m/Myr) and the cratonic bedrock-outcrop band (1–4, Namib mean
  2.5) — audit `:39-45`.
- **The user RULED the target** (`63c0555`, in the audit's mutable header at `:5-12`): *"the
  **PRESENT-STATE band** governs (1–10 m/Myr; derived target ≈ 2.6)… our 500 Myr are not the
  entire history of 'the world' — they're where we draw the line of compromise instead of
  simming the boring billion."* **The run-integrated 10–20 band is ruled the wrong referent,
  and the 5–10 km stripping figure is ruled a literature fact rather than an acceptance bar.**
- **The multiplier is bracketed**: `M ∈ [60, 240]`, central 100–200, by three independent
  chains — and *the supply/transport pair collapses to one axis*, derived rather than assumed.
- **The acceptance quantity moved**: D3, not D1 (corrections #60), and against **outcrop**
  bands rather than catchment ones.
- **`uplift_scale` is named as P2's sibling**, integrator-settled measurement-first.
- **Two named blockers to closing it**: `⟨exp(−H/H*)⟩` has never been printed, and **no
  denudation number exists under the fixed operator at all**.

**The audit cites its asker by name** (`:20-21`: *"`dependency-graph.md` § 2 row P2; § 4 item
4, *'needs a literature pass, not an engineering one'*"*). **The asker holds no link back.**
This is precisely the one-directional-pointer shape CLAUDE.md read-first item 5 makes the
correction's author responsible for — and precisely the ROADMAP-body variant the 2026-07-29
sweep flagged as having *"no mechanism yet"* (`ROADMAP.md:682-687`).

**Aggravating:** `dependency-graph.md` § 5 (`:228-229`) states its own rule — *"END of session
— the `wrap` ritual checks it. A slice that shipped and left a row stale is an unclosed
loop."* The close block **does** know (`ROADMAP.md:4129`, *"P2 measurement runs (plan in the
P2 audit; needs the build slot) after slice 2"*; `:4145`, *"P2 derivation (craton ruling in
header)"*) — so the knowledge is in the corpus and only the **body entries a cold session
reads** are stale. That is the exact failure `dependency-graph.md` § 5's *"two failure modes
to watch"* item 1 names.

Also stale by the same landing: `dependency-graph.md:206-211` § 4 item 6 tells the reader to
do the remaining `dt` conversions *"WITH P2, against the literature — not before it"*; the
literature half is now done and the item does not say so.

---

### F3 — 🟠 CONTRADICTED. `dependency-graph.md:79` cites the wrong stub number for the P11 content-set gap.

`docs/dependency-graph.md:79` (row P11): *"the deep tier defaults to vanilla (**stubs #33**;
heir = E7's per-world manifest)"*.

- **#33 is `a-private-gauss-module-inside-draws`** (`stubs.md:1454`, added 2026-07-29 from
  journal/0128) — an unrelated entry about `draws.rs`'s normal-quantile table.
- The stub actually described is `stubs.md:1421`,
  `the-deep-tiers-content-set-is-hard-wired-to-vanilla`, whose own body names the same heir
  (*"the **per-world manifest** … This is E7's field"*, `stubs.md:1440-1441`).

**Mechanism:** the row was written while the ordinal was deliberately pending (`662c84d`) and
guessed the next free number; the wrap then assigned a different one (`164be0f`) and did not
walk back to the row. This is F1's collision producing a *second*, independent wrong pointer
— the graph now points at #33, the close block at #34, and #34 is already taken.

---

### F4 — 🟠 SUBSUMED (mechanism), OBSERVATION OWED. Two Observed entries name a mechanism P11 slice 1 closed by construction for the deep-history path.

Both entries rest on one mechanism: **`Single` voxels resolve their member through
`dithered_member`, `Mixed` voxels use the canonical `event.member`, so the two paths can
disagree.**

- `ROADMAP.md:3203-3214` — *"A front's parent alternates diorite/granite down a single
  column"* (measured 2026-07-25 by the weathering-profile slice)
- `ROADMAP.md:3447-3460` — *"Mixed voxels carry no member dither — watch for the chunk-line
  cutover coming back"* (journal/0055 judgment call 2, 2026-07-21, UNTESTED either way)

The entries already know they are one mechanism seen from two sides (`:3210-3214`: *"a slice
that touches either should settle both"*).

**P11 slice 1 touched it.** `crates/dc-worldgen/src/geology.rs:1043-1055`:

```rust
// **A recorded identity is not re-adjudicated** (P11 slice 1). …
if !event.dither {
    return event.member;
}
```

and `geology.rs:797` sets `dither: false` on deep-history events (`:260` keeps `dither: true`
for the year-zero veneer, *"legitimately, because the veneer really is forming now"*).
`collapse.rs:614-619` confirms the `Mixed` path uses the same canonical member. **Where an
event carries a recorded identity, the `Single` and `Mixed` paths now return the same
`MaterialId` by construction, so the divergence the entries describe cannot occur there.**

**What is NOT resolved, stated so the integrator does not over-read this:**
1. **The veneer still dithers** (`geology.rs:260`, `:1048-1052`) — the mechanism survives for
   year-zero material, and its heir is named: **slice 3's near-path restructure (ruling 5)**,
   which the entries do not currently cite.
2. **The observation itself is a MEASUREMENT, not reasoning, and I did not re-run it.**
   Diorite/granite is `igneous-intrusive` basement, which I believe is deep history — but I
   did not verify that the observed column's parent events carry `dither: false`. Per the
   sweep's hard rule 3 this stays **open with a named cheap falsifier**: re-probe the
   2026-07-25 station's column and check whether the alternation is gone. **It is also two
   appearance changes behind the world** (§ 0b), so a re-probe is owed regardless.

---

### F5 — 🟠 CONTRADICTED, and the debt GREW inside the window. The `"vanilla"` strike ruling.

`ROADMAP.md:2851-2859` records the user ruling (2026-07-29): *"the default pack is not
elsewhere referred to as 'vanilla'… **I would strike it**"*, with residual counts *crates
**145** · docs **37** · ROADMAP+history **8** · journal 30 (append-only, must NOT be swept)*
and the plan *"Sequenceable as one mechanical sweep **once the worldgen agents land**."*

**Measured at HEAD vs the watermark, same method both sides
(`git grep -ioc vanilla <rev> -- crates`, matching lines):**

| | `72fbe86` | `d8407e1` |
|---|---|---|
| `crates/` | **162** | **177** (+15) |

The body files **were** struck to zero inside this window (`dc-api/src/bodies.rs` 11 → 0,
`dc-api/tests/bodies.rs` 10 → 0, `dc-client/src/character.rs` 3 → 0), so the ruling was
honoured where it was aimed. **But P11 slice 1 added the term to eleven files that did not
have it**, and not only as call sites to the `geology::vanilla()` function — as *prose about
the default pack*, which is what the ruling governs:

- `crates/dc-worldgen/src/deeptime/mod.rs:221-226` — *"The production pregen path still
  defaults to **vanilla**… a **vanilla**-laid deep record under a custom expression set"*
- `crates/dc-worldgen/src/deeptime/lithology.rs:339` — *"authority for every **vanilla**
  member"*
- `crates/dc-worldgen/src/deeptime/recorder.rs:202`, `:780` · `refine.rs:221` ·
  `field.rs:622` · `examples/member_diversity_probe.rs` · `tests/biotic.rs`,
  `tests/deeptime.rs`, `tests/erodibility.rs`
- and `docs/design/stubs.md:1421` — the new stub's **slug is
  `the-deep-tiers-content-set-is-hard-wired-to-vanilla`**, so the struck word is now baked
  into a stable identifier that other docs will cite by name.
- `crates/dc-core/src/materials/geology.rs` went **15 → 21** matching lines — the file the
  entry itself named as *"the bulk **and is under active edit**"*.

**Two things follow, and they point opposite ways:**
1. **UNBLOCKED** — the entry's stated trigger (*"once the worldgen agents land"*) has fired.
   P11 slice 1 is merged; the geology cluster's big edit is done.
2. **The entry's numbers are stale in the wrong direction, and the window's largest slice
   was written in the struck vocabulary.** A one-line rider on the P11 arc entry — *new deep
   code does not name the default pack "vanilla"* — is cheaper than the sweep it is
   currently deferring, and slices 2–4 are about to write a lot more of this code.

---

### F6 — 🟠 STALE. The file-size chore still reads "(build it)"; the hook exists and was amended in-window.

`ROADMAP.md:1418-1435`, *"🔴 REQUIRED CHORE — FILE SIZE IS A CORRECTNESS PROBLEM"*, item
*"**THE MECHANISM (build it): a hook on file write that reminds**"* — with no shipped stamp
anywhere in the entry.

**It shipped.** `scripts/filesize_hook.py` is in the tree, and
`journal/corrections.md:3512` (#85) dates it — *"the file-size hook's `remedy()`
fallthrough, **wired 2026-07-28**"* — while *falsifying* one of its claims (the split axis is
a **docs** convention; source splits by concern), i.e. the mechanism is not merely built but
already once corrected. The close block uses it as live (`ROADMAP.md:4136-4140`,
*"`session-workflow/SKILL.md` crossed its ARGUMENT threshold (1,163/1,000)… **per the
hook**"*).

So the corpus contains the hook, a correction *about* the hook, and a close block *invoking*
the hook — while the Sequenced entry that ordered it still reads as unbuilt. Pure archive /
stamp work; no judgement needed.

---

### F7 — 🟠 STALE POINTERS. Three docs still cite `journal/pending-p11-slice1`, a file that no longer exists.

The P11 slice-1 journal was drafted under the slug-only convention, renamed to `0135` at the
wrap (`dcc6b43`) and to `0136` the next morning (`3cab931`). **The `pending-` references were
never resolved at either step:**

- `ROADMAP.md:396` — inside the P11 build-sequence item 1: *"✅ BUILT 2026-08-01
  (**journal/pending-p11-slice1**)"*
- `docs/design/geology.md:413` — *"⚠ HALF-DISCHARGED 2026-08-01 by P11 slice 1
  (**journal/pending-p11-slice1**)"*
- `docs/audits/2026-07-22-seam-inventory.md:14` — *"**journal/pending-p11-slice1**"*

`3cab931`'s message reads *"All six corpus references to `journal/0135` mean the B0 entry;
only the geo close block's two lines meant the P11 journal — **both patched**."* That is
accurate **for references spelled `journal/0135`** and is an undercount of the actual
dangling-pointer set, because the three above were spelled differently and so were invisible
to that grep. Two of the three are **read-first-adjacent** (the top ROADMAP arc; a design
doc's banner).

Cross-ref: this is a live instance of the Observed entry at `ROADMAP.md:3030`, *"🟠 THE GATE
CANNOT SEE BROKEN DOC LINKS — and doc comments are how this repo routes readers."* Filing it
here rather than as a new Observed line, because the parent already exists.

---

### F8 — 🟡 STALE TENSE. The P11 arc's `WHAT` paragraph describes the pre-slice-1 world in the present tense.

`ROADMAP.md:361-365`: *"**Today** the ~7-entry Litho roster carries all of deep time and
members **are invented at expression** (the member dither under formation context); after
this arc, identity is *recorded*, and expression expresses."*

Slice 1 shipped that half: `StrataEvent::dither = false` for deep history
(`geology.rs:797`), `deep_class_of_species` no longer runs on the deep-history expression
path, and the arc entry's own measured block (`:413-417`) reports both multi-member classes
recording both members at 55.3/44.7 and 62.5/37.5.

**Low rank on purpose** — the arc is genuinely still open (slices 2–4), the entry's own
sub-items are correctly stamped, and "the WHAT paragraph describes the starting state" is a
defensible convention. Flagged only because a cold session reads the WHAT paragraph first and
`Today` is now false. A three-word edit (*"Before this arc…"*) settles it.

**Adjacent, and NOT graded because I could not settle it:**
`ROADMAP.md:2941-2948` — *"The vanilla set caps the member dither at a coin flip in 4 of 10
classes"* — and `docs/dependency-graph.md:51` restating it. The census is still true *of the
veneer dither*; it is no longer the mechanism selecting members in **recorded deep history**,
where deposition-time fitness now produces measured 55/45 and 63/38 splits rather than a coin
flip. Whether that changes the entry's **conclusion** (that no member dither could have
produced U3's tan/grey/red-brown squares) I did not attempt to determine — the conclusion
turns on class-vs-member albedo, not on the draw, and it sits under a `⚠ CONTESTS A POSSIBLY
USER-ORIGINATED CLAIM` banner that explicitly forbids an implementation-side resolution.
**Integrator's, or the user's.**

---

## 2. CANNOT-DETERMINE — flagged rather than reconciled

1. **Is the diorite/granite alternation actually gone in the world?** F4 closes the
   *mechanism* by reading the code; it does not close the *observation*. A user-adjacent
   field measurement stays open until a measurement says otherwise (sweep hard rule 3). The
   falsifier is cheap and is owed by the next walk anyway.
2. **Does F8's adjacent U3 census claim need re-grading post-P11?** Under a CONTESTS banner;
   not the sweeper's.
3. **`ROADMAP.md:2941`'s "4 of 10 classes" census** was measured on the vanilla set as of
   journal/0129. I did not re-run it against the set as it stands after P11 slice 1 (no cargo
   permitted this run). If a member was registered since, the number moves.
4. **`ROADMAP.md:188` / `:202`'s two threshold-quantization `NEEDS RATIFICATION` flags** and
   `:3001`'s note that *"#39 has sat `NEEDS RATIFICATION` for 7 days"* — that is now **11
   days**. Nothing in this window ruled on them; recorded as still-open rather than stale, but
   the ageing note is itself stale.
5. **Golden state.** Not verified by execution (no cargo, by brief). The expected-red list at
   `ROADMAP.md:4113-4115` is taken as accurate on the close block's word.

---

## 3. What this sweep did NOT cover

A null over these is not a result:

- **`ROADMAP-history.md`** — not opened. The sweep's question is about *live* entries.
- **The three superseded close blocks** (`ROADMAP.md:4159`, `:4222`, `:4312`) — read only for
  the P2 and file-size cross-references. Superseded blocks are archive candidates, not
  staleness findings.
- **`journal/corrections.md` #1–#73** — only #74–#87 were read in full (the delta). Entries
  before the watermark were read only where a finding pointed into them.
- **The In-flight section's older arcs** (north star, material-behavior substrate, block↔material
  collapse, forms/partials) were read but produced no delta-driven finding — nothing in this
  window touched them. That is a read, not a null from a blind instrument.
- **Observed entries older than ~2026-07-25** were scanned by heading and opened selectively.
  The 2026-07-29 FULL sweep covered all 151 entries eleven days ago and the baseline covered
  100 % of the corpus; this run deliberately spent its budget on the delta's blast radius
  rather than re-reading that ground.

---

## 4. Meta — the shape this run found, stated once

**Four of the eight findings (F1, F3, F7, and half of F5) have one cause: two sessions sharing
one checkout, each assigning "next free" from a view that did not include the other.** It cost
one journal ordinal (caught), two correction ordinals, one stub ordinal, one wrong
cross-reference in the dependency graph, and three dangling journal pointers — **in a single
seven-hour window**, between two sessions that both knew the hazard and one of which had
explicitly deferred an ordinal *because* of it (`662c84d`).

The project's own doctrine answers this without needing anything new: *"do not answer 'the
tool cannot see X' with a rule asking people to remember X"* (CLAUDE.md § Gates), applied
twice already — to probes (`test = true`) and to the build slot (`cargo_mutex_hook.py`, this
same window). **A numbering space with two concurrent writers is the same shape as a build
slot with two concurrent writers.** The cheapest available mechanism is not a new instrument:
it is `wrap` grepping for a duplicate ordinal in the three append-only files before it
assigns one — the check is `grep -c "^## N\."`, and it would have fired four times this week.

Recorded here as an observation for the integrator, **not** filed as a proposal — the sweeper
does not adjudicate what it finds, and this is a process change, which is the user's.
