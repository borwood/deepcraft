# doc-topology sweep — 2026-08-03 (FULL)

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Anything that later
refutes or re-scopes this file gets a banner **here**, stamped by the author of the
correction.

> **Mode:** FULL. **Delta:** `d8407e1..HEAD`, **143 commits** — spans the GEO-3 arc
> (P11 slices 1/2/3, the erosion split, FS-A, P2 measurement runs, E4 + E4-1, the
> stratigraphic-correlation pivot) and the bodies arc (posture bake, look ownership,
> gait bake + consumer, the derived-gait walk).
> **Every quotation below was read at `9fd6430`** (this worktree's tip; `git rev-parse
> --short HEAD`) — corrections #67's rule. `9fd6430` is `2985273` (main at dispatch)
> plus this file's own skeleton commit; no source or doc file differs between them.
> **NOT APPLIED — the integrator adjudicates.** Recommendations are labelled as such.
> **Corrections filed: ZERO.** See § 4 for why, and for the one finding that is
> correction-shaped if the integrator disagrees.

---

## 1. Ranked pairs

Ranked by **blast radius**, per the skill's rule — read-first surfaces first, journal
history last.

| # | shape | the pair | provenance | blast radius |
|---|---|---|---|---|
| **F1** | 2 + 6 | **journal/0141 says the 60 s pregen budget was NOT moved; it was moved 20×, at that slice's own merge** | user-originated on the deciding side | **HIGH** |
| **F2** | 6 + 5 | **`S13-results.md` § C is priced out by a `<60 s` bound that no longer exists, and carries no banner** — while the renegotiation record *names S13* | assistant | **HIGH** |
| **F3** | 5 | **`DEEP_MAX_WIDTH = 550`'s stated justification is the expired 60 s budget** (A-2, third corpus instance, expiry in a different file) | assistant | **HIGH** |
| **F4** | 1 | **`dependency-graph.md` B2 row cites `journal/0138` for the bodies consumer slice + walk** — 0138 is the P11 CSR foundation entry; the walk is 0140 | assistant | **HIGH** (read-first item 1c) |
| **F5** | 6 | **the correlation audit declares it supersedes the slice-3 audit's mechanism; the slice-3 audit carries no banner** — and that file's own header + I-6 demand one | both assistant; the *supersession* is user-originated | **HIGH** |
| **F6** | 4 + 2 | **"the 0147 wall"** — the interfingering wall is journal/**0149**; the correlation audit says 0147 three times, and cites the 0149 assets four lines from its own first use | assistant | MED |
| **F7** | 1 | **`gait.rs` / `report.rs` doc comments still assert "this world's gravity is 25.0"**, refuted by `ARCHITECTURE.md` § Gravity and by `world_constants.rs` — and one is a **printed test caption** the gate cannot check | user-originated on the refuting side | MED |
| **F8** | 2 | **journal/0144's headline finding ("wrong turn the first") was inverted ten minutes after its own build commit** and the entry carries no postscript, where 0136 and 0138 both got one | user-originated on the refuting side | MED |
| **F9** | 4 | **984 vs 980 workspace tests** in two live close blocks with no world named on either | assistant | LOW |
| **F10** | 1 | **ROADMAP's superseded 2026-08-02 BODIES close block cites `journal/0138` for the derived-hip walk**; same renumber as F4 | assistant | LOW |
| **F11** | 1 | **`gait-bake-member1-design.md:136` cites "journal/0137, 0138" for member #0 built-and-consumed**; the same file gets it right at `:680` | assistant | LOW |

---

### F1 — the 60 s pregen budget: "not moved" vs moved 20× · shapes 2 + 6 · HIGH

**Side A — `journal/0141-the-load-stopped-being-seven-things.md:271-283`** (read at `9fd6430`):

> `pregen_time_vs_extent`'s ratified
> 60 s budget was **not moved**; it is passing at 50.3 s with ~16 % headroom, where
> it had ~30 % before.
>
> **And the pregen budget is where this slice has to stop and ask.** The
> `pregen_time_vs_extent` assertion is a ratified 60 s, and it passes uncontended
> (50.3 s) and **fails inside the full workspace gate at 60.68 s** … Moving a gate to admit one's own work
> is not a fix, so it is not moved here either.

**Side B — `ROADMAP.md:539-541`** (read at `9fd6430`):

> Full workspace gate on the branch 928/929; the one red was the
> 60 s pregen budget under gate contention, **renegotiated by the user at merge
> ("as long as it doesn't take 20min") → 1200 s**, prior value kept for audit.

**Side B, corroborated in the tree** — `crates/dc-worldgen/tests/s7_measurements.rs:34-40`:

> `// Sanity: the pause must stay a ritual, not a wait. Budget renegotiated`
> `// 2026-08-02 at P11 slice 2b's merge (user, verbatim: "I don't care`
> `// whatsoever about this number as long as it doesn't take 20min") —`
> … `assert!(ms < 1_200_000.0, …)`

and in `docs/dependency-graph.md:79` (*"full workspace gate green after the user's
pregen-budget renegotiation (60 s → 1200 s)"*).

**Why it is a finding and not just history.** journal/0141 is the P11 slice-2 record,
cited by name from `ROADMAP.md:528` and `dependency-graph.md:79`. Its last substantive
paragraph is a *question to the user*, and the user answered it — at that same merge, in
the opposite direction. The two sibling entries of this arc, `journal/0136` (§ *Postscript
at merge*) and `journal/0138` (§ *Merge postscript*), both carry an integrator postscript
recording exactly this kind of at-merge resolution. **0141 does not.** A cold reader lands
on *"a ratified 60 s"* and *"moving a gate to admit one's own work is not a fix"* and
carries away a live constraint that has been 1,200 s since 2026-08-02.

This is read-first item 5's own stated failure mode, one file over: *"a close block
recording a RESOLUTION owes the asking document its banner in the same commit… nobody
re-reading the question's doc can find an answer recorded only there."*

**Recommendation (labelled — this is a recommendation, not a ruling):** add a merge
postscript to `journal/0141`, in the shape 0136 and 0138 already use, naming the
renegotiation, the user's verbatim words, and the new value. **Do not edit the body** —
the builder's argument was correct engineering and is worth keeping intact; what is
missing is the sentence saying the user overruled the constraint rather than the
reasoning.

---

### F2 — S13's option C was priced out by a bound that no longer exists · shape 6 + 5 · HIGH

**Side A — `docs/spikes/S13-results.md:435-440`** (read at `9fd6430`):

> **Cost, measured-adjacent:** Medium is 545² = 297 k cells at 15.5 s and ~52 MiB
> today. Halving the cell is 1090² = 1.19 M cells → ~**62 s** and ~**210 MiB**,
> and `DEEP_MAX_WIDTH = 550` would have to rise. That **breaks the
> `pregen_time_vs_extent` <60 s budget**, which is a ratified number.
> "Worldgen time is not a constraint" is the standing user position, so this is a
> budget renegotiation rather than a blocker — but it is a renegotiation.

**Side B — `ROADMAP.md:508-512`** (read at `9fd6430`), recording the resolution and
**naming S13 while doing it**:

> The 60 s is a ratified number
> (S13 called it that); **it has NOT been moved, because moving a gate to admit one's own
> work is not a fix.** Two resolutions, both user-owned: renegotiate the budget (the
> standing *"worldgen time is not a constraint"* position, which S13 already framed as a
> renegotiation rather than a blocker)…

— followed at `ROADMAP.md:540-541` by the renegotiation actually happening.

**Absence, with its pathspec** (corrections #67's corollary): `grep -n "1_200_000\|1,200
s\|1200 s\|renegotiat" docs/spikes/S13-results.md` returns only line 440's *"but it is a
renegotiation"* — i.e. **S13 carries no top-of-file banner and no pointer to the
2026-08-02 renegotiation anywhere in the file.** `grep -rn "S13" docs/spikes/S13-results.md
| head` confirms no header block was added in the delta (`git log d8407e1..HEAD --
docs/spikes/S13-results.md` is empty).

**Why it matters, and it is not clerical.** S13 § C is *"refine the deep tier instead — 230 m
cells"*, and S13's own framing is that it is **the only candidate that makes the summit
plateau into a landform rather than draping noise over it**. It was parked behind exactly
one obstacle, and that obstacle was removed a day ago by the user, for an unrelated reason,
in a different file. Nobody has recorded that S13 § C is now cheap. This is the corpus's
canonical one-directional-pointer failure: the citing document names the target, the target
holds no link, and *the stale end is where a cold session enters*.

**Recommendation (labelled):** stamp `S13-results.md` with a top-of-file banner —
immutable body, mutable header — naming the 2026-08-02 renegotiation (60 s → 1,200 s,
`s7_measurements.rs:34-40`, `ROADMAP.md:540`) and stating in one line that § C's stated
blocker is discharged. **Whether to actually take § C is a separate, user-owned call** and
this sweep does not recommend it.

---

### F3 — `DEEP_MAX_WIDTH`'s justification expired elsewhere · shape 5 (A-2) · HIGH

**Side A — `crates/dc-worldgen/src/deeptime/field.rs:28-33` and `:43-46`** (read at `9fd6430`):

> `//! `pregen_time_vs_extent` <60 s budget. So the production config **caps the`
> `//! grid width** at [`DEEP_MAX_WIDTH`]: Small/Medium keep 460 m as specified;`
> `//! Large gets a coarser cell (~1.8 km) that holds the ritual budget.`

> `/// Cap on the deep grid width (cells per side). Holds the boot ritual to`
> `/// ~300 k cells / ~15 s regardless of extent…`
> `pub const DEEP_MAX_WIDTH: usize = 550;`

**Side B —** the same `s7_measurements.rs:34-40` / `ROADMAP.md:540-541` renegotiation as F1.

**Reading.** `spines.md:1244-1272` § A-2 (read at `9fd6430`) states the rule this is an
instance of: *"decisions expire premises **elsewhere**, so a sweep must ask 'does the cited
constraint still hold?'"* — and lists exactly two instances (`lithology.rs`'s reference
member, `BEDROCK_SEAM_THICKNESS_M`). **This is a third, and A-2 does not list it.** The
constant is not *wrong* — a 550-cell cap may still be the right engineering answer for
residency, and Large's coarser cell has read-quality consequences S13 discusses. What is
false is the **stated reason**, and A-2's whole point is that a live justification asserting
an expired constraint is what costs an architecture the day someone believes it.

**Recommendation (labelled):** re-state the doc comment against a premise that still holds
(residency, or a fresh time bound derived rather than inherited), and add the instance row
to `spines.md` § A-2 in the same commit. **This is spine-audit's remit as much as mine** —
its watermark is `3cf8778`, which is *inside* this delta, so the constant was not re-read
against the renegotiation by that pass either.

---

### F4 — `dependency-graph.md` sends the reader to the wrong journal entry · shape 1 · HIGH

**Side A — `docs/dependency-graph.md:89`**, row **B2** (read at `9fd6430`):

> **B0 SHIPPED 2026-08-01; member #0 design pass + bake SHIPPED 2026-08-02 (journal/0137,
> measured == predicted to 1e-9) and the CONSUMER SLICE SHIPPED + WALKED same day
> (journal/0138) — user verdicts: rest "reads right", motion "fine with caveat".**

**Side B — `journal/0140-three-bodies-stand-at-their-own-heights.md:3`** (read at `9fd6430`):

> *(Renumbered 0138→0140 at 2026-08-02 integration: the geo session claimed and pushed 0138
> nineteen minutes earlier; 0139 = the erosion split. Fifth ordinal collision of the 24-hour
> window — see the close block.)*

`journal/0138` at `9fd6430` is **`0138-the-load-learns-the-rocks-name.md`** — P11 slice 2's
CSR foundation. A reader following `dependency-graph.md:89` lands on a deep-time storage
entry that says nothing about hips, walks or user verdicts.

**Blast radius.** `dependency-graph.md` is **read-first item 1c**, briefed to be read at
session start, throughout, and at wrap; `ROADMAP.md:4602` names *"§ 2b (rows B7, B8, and
B6's eight inbound edges)"* as the first thing next session reads. The B2 row is the row
next to those. The same row cites 0144/0147/0148 correctly at `:90-94`, so the error is
isolated to the one pre-renumber sentence.

**Recommendation (labelled):** retarget `:89`'s `(journal/0138)` → `(journal/0140)`. Cheap,
mechanical, and it should carry the renumber note so it is not "corrected" back.

---

### F5 — a supersession declared in one direction only · shape 6 · HIGH

**Side A — `docs/audits/2026-08-03-stratigraphic-correlation-design.md:6-9`** (read at `9fd6430`):

> Supersedes-as-plan the mechanism of
> `docs/audits/2026-08-02-p11-slice3-design.md` (the membership dither) — *existence is not
> standing, applied to our own day-old mechanism.*

**Side B — `docs/audits/2026-08-02-p11-slice3-design.md:67-69`** (read at `9fd6430`), which
asks for the back-pointer **by name**:

> **Immutable body, mutable header** (CLAUDE.md read-first item 5). Read at commit
> `665167e`. Anything that later refutes or re-scopes this file gets a banner **here**,
> stamped by the author of the correction.

and at `:565`, in its own same-commit obligations list (I-6):

> stamp targets in the same commits: stubs #25/#31/#37 banners, **this file's header when
> refuted**, priors header if the hint pick changes its "not foreclosed" line

**Absence, with its pathspec:** `grep -n "SUPERSED\|supersed\|correlation" docs/audits/2026-08-02-p11-slice3-design.md`
returns nothing about the correlation pass; the file's only added header in the delta is the
`BUILT 2026-08-02/03` block at `:48-65`. **No supersession banner exists.**

**Why this ranks high despite being a doc-hygiene item.** The slice-3 audit is a live,
recently-written, 600-line design document whose recommended mechanism the user rejected by
eye and whose plan the user replaced with his own sketch. Its § 5–§ 8 still read as the plan
of record. The *supersession itself* is user-originated (`journal/0149`; ROADMAP § Observed
`:2901-2927`) — so this is not two assistant opinions, it is a user ruling that has not
reached the document it overturns. And the target file **asked for the banner in advance,
twice.** This sweep exists to find one-directional edges; a file that pre-declares its own
banner slot and does not get one is the sharpest available instance.

**Recommendation (labelled):** banner `2026-08-02-p11-slice3-design.md` at its top, naming
`2026-08-03-stratigraphic-correlation-design.md`, `journal/0149`, and the fact that **P-5
(formal supersession ratification) is still a pending user pick** — the banner should say
*superseded-as-plan, ratification pending*, not *superseded*.

---

### F6 — "the 0147 wall" · shape 4 + 2 · MED

**Side A — `docs/audits/2026-08-03-stratigraphic-correlation-design.md`**, three occurrences
(read at `9fd6430`):

- `:439` — *"half of the fix at the 0147 wall."*
- `:539` — *"Stations: (1) the 0147 wall — the razor cliff must…"*
- `:590` — *"user eye owed because the 0147 wall is the exhibit that opened this pass"*

**Side B — the same file, `:5-6`**, four lines above its own first use of the phrase:

> its recorded station (wall pose feet **(82490, 215.5, 13346) m, yaw 0, pitch +0.12**;
> assets `0149-interfinger-contact-wall.png` + `-benchcut`)

**and `ROADMAP.md:2925`** (§ Observed) repeats it: *"P-4 the onlap feather at the record edge
(the 0147 wall becomes a wedge)"* — while **`ROADMAP.md:4546`** (the GEO-3 close block, same
file) has it right: *"P-4 (onlap feather at the record edge — the 0149 wall becomes a
wedge)"*.

`journal/0147` at `9fd6430` is the **bodies** gait-consumer slice. A cold reader chasing "the
0147 wall" lands in the wrong arc entirely.

**Recommendation (labelled):** retarget all four sites to 0149. Note that ROADMAP contains
both the wrong and the right form, so a grep-and-replace is safe here.

---

### F7 — "this world's gravity is 25.0", still asserted · shape 1 · MED

**Side A — `crates/dc-api/src/bodies/gait.rs:320-322`** (read at `9fd6430`):

> `/// **Gravity is an argument, and that is a finding, not a convenience.** The`
> `/// design pass derived its whole predicted table at `g = 9.81`; this world's`
> `/// [`crate::CharacterConfig::gravity_m_s2`] is **25.0**.`

**and `crates/dc-api/src/bodies/gait/tests/report.rs:156-159`** — a **printed test caption**:

> `/// **The same table at the gravity this world actually runs** — `CharacterConfig``
> `/// default `gravity_m_s2 = 25.0`, not Earth's 9.81. The design derived`
> `/// everything at 9.81, which is right for the *literature* and wrong for the`
> `/// *world*: this is the finding, printed so it cannot be lost.`

**Side B — `docs/ARCHITECTURE.md:758-772`** (read at `9fd6430`), **user-originated**:

> ## Gravity is a WORLD constant, and it defaults to Earth — DECIDED 2026-08-02 (user)
> … **Gravity is a property of the WORLD** … It becomes a world-defined value with an
> **Earth default (9.81 m/s²)**, read by every consumer rather than restated by each.

— and `ARCHITECTURE.md:802`: *"**The gait table becomes CORRECT rather than needing
correction.**"*

**Verified in the tree at `9fd6430`:** `crates/dc-api/src/character.rs:68` reads
`gravity_m_s2: dc_core::DEFAULT_GRAVITY_M_S2`, and `crates/dc-api/src/bodies/gait/tests.rs:40`
reads `const G_WORLD: f64 = dc_core::DEFAULT_GRAVITY_M_S2`. So **`G_WORLD == G_EARTH` today**,
and the test named `the_same_bodies_at_this_worlds_gravity` prints a "contrast" table that is
now identical to the Earth one, under a caption asserting the contrast is *"the finding"*.

**This is CLAUDE.md § Gates' own named hazard**, one arc later: *"A printed caption is a
published claim the gate cannot check."* The `flux_record_probe` precedent is cited there
verbatim.

**Recommendation (labelled):** rewrite both captions to state the *historical* finding in
past tense and point at `ARCHITECTURE.md` § Gravity + `dc-core::world_constants`. Keep the
two-gravity sweep tests — `tests.rs:126,169` iterate `[G_EARTH, G_WORLD]` and remain a real
dimensional-similarity check even when the two coincide; say so, rather than deleting them.
**This is docs-vs-code and therefore also spine-audit's**, whose watermark (`3cf8778`) predates
the gravity flip commits (`36aaf02`/`474344b`/`bd0c82c`, all 2026-08-02).

---

### F8 — journal/0144's headline finding, inverted ten minutes later · shape 2 · MED

**Side A — `journal/0144-the-gait-had-to-ask-the-world-for-gravity.md:51-58`** (read at `9fd6430`):

> ## Wrong turn the first: the design derived the whole table at Earth gravity
>
> Every number in the design pass's § 3.5 is computed at `g = 9.81`. That is the
> right constant for the *literature* … and the wrong constant for the *world*.
> `CharacterConfig::default()` has carried **`gravity_m_s2: 25.0`** since bring-up,
> a deliberate game-feel choice…

**Side B — `docs/ARCHITECTURE.md:760-765`** (read at `9fd6430`), user verbatim:

> **`25.0 m/s²` was never chosen.** … User: *"Nobody ever consciously chose 25 m/s². This is
> the first I'm hearing of it because it's a bootstrapping artifact…"*

**Timing, from git at `9fd6430`:** the gait-bake build is `5859976`, **2026-08-02 22:51:11
−0400**; the gravity ruling is `36aaf02`, **2026-08-02 23:01:20 −0400**. **Ten minutes.**

Two claims in 0144 are now false as statements about the world: *"a deliberate game-feel
choice"* (the user says nobody chose it) and *"the wrong constant for the world"* (the world
is Earth, so the design pass's table was right). The entry's § *Measured against predicted*
also reports *"The clip cross-check only lands at Earth gravity… At `g = 25` the same
comparison is 37.2 %, outside the band"* — a caveat that no longer describes the shipped
world. journal/0148's own numbers confirm the flip: Froude 2.35 at 4.5 m/s on a 0.88 m biped
is `9.81`, not `25`.

**Note which way this cuts:** the *mechanism* 0144 discovered — the bake must take gravity as
an argument because the body does not own it — **survived and is what produced the ruling**.
The entry is a good entry. What is missing is one line saying the constant it argued against
was retired within the hour.

**Recommendation (labelled):** a short postscript on `journal/0144`, matching 0136/0138's
merge-postscript shape. Journal bodies are append-only narrative and **must not be rewritten**.

---

### F9 — 984 vs 980 workspace tests · shape 4 · LOW

**Side A — `ROADMAP.md:4535-4539`** (GEO-3 close, read at `9fd6430`):

> **Main is FULLY GREEN at `090f778`+ … `test --workspace --no-fail-fast`
> = 984 passed / 0 failed / 94 suites, exit 0.**

**Side B — `ROADMAP.md:4615-4616`** (BODIES close, same file, read at `9fd6430`):

> Workspace **980 passed / 94 suites / 0 failed**.

Both are live close blocks in the same file; next session holds **both**
(`ROADMAP.md:4600`). Neither names the commit its count came from — side A names
`090f778+`, side B names none. Almost certainly honest (two threads, two tips, four tests
apart), and the skill's own rule for this shape is *"name the world every number came from"*.

**Recommendation (labelled):** add the commit to the BODIES block's count, or one clause
saying the two blocks were gated at different tips. **Not a defect; a missing sentence.**

---

### F10 / F11 — the same 0138→0140 renumber, two more sites · shape 1 · LOW

- **`ROADMAP.md:4704`** (inside the **struck** 2026-08-02 BODIES close block, read at
  `9fd6430`): *"the consumer slice put the derived hip on screen and the user walked it
  (journal/0138)"*. Low harm — the block is marked SUPERSEDED at `:4693` — but it is an
  archive candidate under the archive-by-status rule, and archiving it with a wrong pointer
  freezes the wrong pointer.
- **`docs/audits/2026-08-02-gait-bake-member1-design.md:136`** (read at `9fd6430`):
  *"rulings (member #0, **built and consumed** — journal/0137, 0138)"* — while the same file
  at `:680` correctly says *"absence cost the walk-8 strafe (journal/0140)"*. One file,
  both forms; the design pass was written across the renumber.

**Recommendation (labelled):** fix `:136`; leave `ROADMAP.md:4704` to the archiving pass and
fix it there.

---

## 2. Shape-7 candidate — a premise hiding inside a caveat

**REPORTED, NOT RESOLVED.** Per the skill: *"premises about what the player does are the
user's, always."* One candidate, and it sits in the cheapest place to look — a caveat
blocking a **user-owned** ratification fork.

### S7-a — the pits bar: "hollows > 10 m = 0" as a blocker on the P2 flip

**The caveat.** `docs/audits/2026-08-02-p2-measurement-runs.md:132` (read at `9fd6430`):

> | **pits: hollows >10 m = 0** | journal/0122's bar | **✗ FAILS at every in-band rung** —
> 33 @150, 406 @250, **8,878 @400 (deepest 27.3 m)**; shipped row: 0. See § 4 |

and `:158-162`, one of the **three named blockers on the flip**:

> **⚠ The pits bar and the band CONFLICT under the current transport/fill:** every rung
> that reaches the band violates hollows->10 m = 0 … Either the router/fill needs work
> before the flip, or the bar needs a ruled revision — user call, with journal/0122's
> derivation of the bar in hand.

Propagated to `docs/dependency-graph.md:70` (read-first item 1c) as one of *"three user
calls"*, and to `docs/audits/2026-08-03-e4-implicit-kernel-design.md:434`.

**The measurement is not in question.** 8,878 hollows deeper than 10 m at M = 400 is a
measured number from a probe.

**The premise wrapped around it, separated out.** journal/0122 derives the bar at `:286-296`
(read at `9fd6430`):

> The dimple floor is ~1 m and is structural … The failure scale is ~45 m and up —
> journal/0114 measured the clamp genuinely breaking at 45 m and 112 m. **10 m** is an order
> of magnitude above the first and 4.5× below the second, so dimple accumulation cannot
> reach it and a real clamp failure cannot hide under it.

So the bar is **not a landform judgment** — it is a *clamp-failure detector*, and it is valid
only while its two flanking scales (the ~1 m dimple floor, the ~45 m failure scale) stay put.
**Both were measured on the shipped world at M ≈ 45 on a broken operator.** The P2 flip is a
re-pick to M ≈ 375–380 *on a repaired operator*. Nothing in the corpus re-derives the dimple
floor or the failure scale at that regime, and the audits present the bar's failure as a
finding about the calibration rather than as a possible finding about the bar's own
domain of validity.

**What would have to be true about the GAME for the number to matter?** That a 10–27 m closed
depression is a numerical artefact rather than a landform. **That is not written down as a
decision anywhere** — pathspec: `grep -rn "closed hollow\|closed depression\|sinkhole\|kettle\|doline"
docs/design/ docs/ARCHITECTURE.md` at `9fd6430` returns no ruling, only
`docs/design/stubs.md:886,1179,1181` (measurements) and **`docs/design/flow.md:543`**, which
lists among the phenomena the model is required to represent:

> sinkhole & cenote

A cenote is, definitionally, a deep closed depression. So the corpus simultaneously holds a
**recorded ambition** naming deep closed depressions as wanted landforms and an **assistant-
derived acceptance bar** forbidding them at 10 m — and the bar is currently one of three
things holding a user-owned flip shut.

**Not resolved, and deliberately not.** Both halves may be right of different things (an
*erosion-solver* pit at a random hillslope cell is not a karst doline), and the sentence that
says so does not exist. Note also that the user has already ruled the right instrument —
*"go right to [the walk]… census on same world"* (`ROADMAP.md:4562-4564`) — so the pit safari,
not a desk argument, is what settles it.

**Recommendation (labelled):** when the pits-bar call is put to the user, put the *premise*
in front of them and not only the count: (a) is the 1 m / 45 m scale separation still valid at
M ≈ 375–380 under the repaired operator, and (b) is a deep closed depression a defect at all,
given `flow.md:543`. **Do not re-run a probe until (b) is answered** — that is the shape
corrections #70 cost a week.

---

## 3. Shape 3 — a user-originated design reconciled away

**Run, and the result is a NULL with one caveat.** Grep is the sanctioned instrument for this
shape.

Searched at `9fd6430`:
- `grep -rn "borehole\|smooth" docs/design/ideas.md` → **no hits**; `git log d8407e1..HEAD --
  docs/design/ideas.md` → **empty** (ideas.md did not move in this delta).
- `grep -rn "there are no deep cells\|utterly smooth\|non-smooth detail" --include=*.md docs/
  ROADMAP.md journal/` → the sketch and the principle appear **verbatim** in
  `journal/0149:52-63`, `ROADMAP.md:2901-2911` (§ Observed), `ROADMAP.md:4527-4532` (close
  block), and `docs/audits/2026-08-03-stratigraphic-correlation-design.md:14-24`, whose own
  header reads *"This pass EXECUTES the sketch; the sketch is never an input to reconcile
  away."*

**The two user designs of this delta both survived intact.** The correlation sketch was
executed rather than reconciled; the FS-A withheld writer was escalated and the user
**ratified the withholding** (`ROADMAP.md:4520-4523`). The correlation audit even surfaces
the tension between two of the user's *own* rulings (2026-07-19 dress-every-contact vs
2026-08-03 smoothness) at `:375-380` and `:587` and explicitly refuses to weigh them —
which is the rule working, not a finding.

**The one caveat, and it is a reciprocity gap rather than a reconciliation:** the smoothness
principle — which `journal/0149:57` says *"now governs the whole presentation stack"* —
does **not** appear in `docs/design/refinement.md`, the **ratified** doc that owns that stack
(E5, `dependency-graph.md:51`). Pathspec, stated exactly (corrections #63 — state the search,
not just the verdict): `grep -in "smooth\|borehole\|correlat" docs/design/refinement.md` at
`9fd6430` returns **two hits, neither of them this principle** — `:241`
*"correlated-rounding defect"* and `:288` *"the fluvial assemblage is correlated"*, both
unrelated senses of the word. **Zero hits on `smooth` and zero on `borehole`.** A principle
that governs a tier and lives only in a journal entry, a § Observed body, a close block and an
audit is one archive away from being unfindable from the tier's own doc.
**Recommendation (labelled):** one paragraph in `refinement.md`, quoting the user verbatim
and pointing at the correlation audit — *after* P-5 (formal supersession ratification) is
taken, not before, since the audit lists it as a pending pick.

---

## 4. Corrections filed: NONE, and why

The skill is explicit: *"a contradiction is not automatically a correction; sometimes both
statements are true of different things and what is missing is the sentence saying so."*

- **F1** is the only genuinely correction-shaped finding, and it is not a claim that was
  *wrong when written* — it is a claim the user overruled at merge. The corpus's own
  disposal for that is a **merge postscript**, which 0136 and 0138 both received. Filing it
  as a correction would misattribute an integrator omission to the builder's reasoning,
  which was sound.
- **F2/F3/F5** are missing **banners** and an expired **justification** — the disposal is a
  stamp and a re-worded premise, not a corrections number.
- **F4/F6/F10/F11** are stale pointers from renumbers that are already recorded —
  0138→0140 at `journal/0140:3` and `ROADMAP.md:3094`; 0147/0148→0149/0150 at
  `ROADMAP.md:4572-4573`.
- **F7/F8** are residue of a ruling already recorded in full at `ARCHITECTURE.md:758-802`.
  A second entry would restate a decision, not correct one.
- **S7-a** is explicitly not to be resolved by this sweep.

**If the integrator disagrees on F1**, the next free ordinal at `9fd6430` is **#98**
(`journal/corrections.md` tail: #97 at `:3869`). **⚠ RE-VERIFY AT MERGE** — a parallel
session may claim #98 first; the ordinal pre-commit guard has caught five collisions in this
delta's window (`ROADMAP.md:4572-4573`, `journal/0140:3`, corrections #95's note). And if it
is filed, **the target owes a banner in the same commit**: `journal/0141`, per read-first
item 5.

---

## 5. Coverage — what was read, what was NOT

**Read in full at `9fd6430`:**

- **The delta's journal entries, all 15:** 0136, 0137, 0138, 0139, 0140, 0141, 0142, 0143,
  0144, 0145, 0146, 0147, 0148, 0149, 0150 (2,303 lines).
- **Read-first:** `.claude/skills/doc-topology/SKILL.md` (including the measured-field
  header); `docs/dependency-graph.md` (all 255 lines); `ROADMAP.md` § Observed `:2865-2984`,
  the arc entry `:495-560`, and **both live close blocks + the superseded 08-02 bodies block**
  `:4507-4710` (checked against themselves, per the always-include rule).
- **Cited targets, in the relevant sections:** `docs/spikes/S13-results.md:420-449`;
  `docs/ARCHITECTURE.md:758-802`; `docs/spines.md:1244-1275` (A-2) and its heading map;
  `docs/audits/2026-08-02-p11-slice3-design.md:1-70,:519,:552,:565`;
  `docs/audits/2026-08-03-stratigraphic-correlation-design.md:1-30` + the four "0147 wall"
  sites; `docs/audits/2026-08-02-p2-measurement-runs.md:125-178`;
  `journal/0122:278-299`; `docs/design/flow.md:528-555`; `journal/corrections.md` tail
  (#83–#97 headers).
- **Code read as the second side of a pair** (not audited as code):
  `crates/dc-worldgen/tests/s7_measurements.rs:20-42`,
  `crates/dc-worldgen/src/deeptime/field.rs:28-49`,
  `crates/dc-api/src/bodies/gait.rs:318-330`, `.../gait/tests.rs:26-50,:126,:169`,
  `.../gait/tests/report.rs:150-168`, `crates/dc-api/src/character.rs:48,:68`,
  `crates/dc-core/src/world_constants.rs:1-60`.

**NOT opened — a null from any of these is not a result:**

- **`docs/design/north-star.md`** (459 lines). Untouched in the delta
  (`git log d8407e1..HEAD -- docs/design/north-star.md` empty); read-first item 0 and
  therefore the **highest** blast radius in the corpus. Skipped on the artifact-count bound,
  not because it is clean.
- **`docs/spines.md`** beyond A-2 and the heading map (2,248 lines, heavily edited in the
  delta by the 2026-08-02 spine-audit). Its own full pass ran at `3cf8778`, inside this
  delta — but *docs-vs-docs* on spines was not run here.
- **`docs/design/stubs.md`** (1,725 lines) — only the four rows grepped for F-series
  citations. **The ordinal space was not audited**; the 2026-08-02 staleness sweep reported a
  duplicate **#34** whose repair I did not verify.
- **`docs/design/material-behavior.md`, `materials.md`, `geology.md`, `bodies.md`,
  `posture-gait.md`, `refinement.md`, `flow.md` (beyond § 5), `ecology.md`, `ores.md`,
  `earth-processes.md`, `things-that-will-happen.md`** — all touched or cited in the delta;
  only grepped, not read. `bodies.md` and `posture-gait.md` in particular carry the bodies
  arc's rulings and were **not** read against journals 0142/0144/0147/0148.
- **`docs/audits/2026-08-02-gait-bake-member1-design.md`, `2026-08-02-joint-limits-b7-design.md`,
  `2026-08-03-e4-implicit-kernel-design.md`, `2026-08-02-appearance-tour-p11.md`,
  `2026-08-01-*`** — grepped for specific nouns only.
- **`ROADMAP.md` § In flight `:20-356` and § Sequenced `:357-494,:560-2864`** — ~2,700 lines,
  read only where the P11 arc entry lives. The staleness sweep owns this axis and is
  **also overdue FULL**.
- **`.claude/skills/session-workflow/SKILL.md` and `wrap/SKILL.md`** — the skill set is
  nominally in the read-first bound; not opened this run.
- **`journal/corrections.md` bodies #1–#97** — only the headers were read.

**Honest summary of the null-risk:** this run followed the two arcs and the read-first spine.
Its findings cluster in *pointer reciprocity* and *expired premises*, which is what reading
journals-against-graph surfaces. **Shape 1 across the design docs (`material-behavior.md`,
`materials.md`, `geology.md`, `bodies.md`, `posture-gait.md`) was effectively not run**, and
two arcs' worth of rulings landed in them during this delta. That is the largest known hole.

---

## 6. Run record

- Mode: **FULL** · delta `d8407e1..HEAD` (143 commits)
- All quotations read at **`9fd6430`** (worktree `agent-a82d628ce6a56c778`, branch
  `agent-a82d628ce6a56c778`)
- Watermark `doc-topology` advanced to the commit this audit lands at, in the same commit
  (`docs/audits/.sweep-watermarks.json`)
- No cargo invoked; no source file modified; read-only except this file and the watermark
