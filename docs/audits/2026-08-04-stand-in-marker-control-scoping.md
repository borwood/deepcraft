# The stand-in marker control — scoping survey

**Dated measurement, 2026-08-04, at `8cfce92`.** Immutable body, mutable header
(CLAUDE.md read-first item 5): the numbers below are what was measured on the day and
are never rewritten. Anything that later refutes or re-scopes them belongs in a banner
here.

**Commissioned by** `journal/corrections.md` **#97** and ROADMAP § Sequenced *"THE
STAND-IN MARKER CONTROL — owed 2026-08-03; **survey first, sweep second**"*. That entry
says the load-bearing deliverable is *"how many of those 200 are real, not a fix"*. This
is that survey. It also ships a **prototype**, `scripts/standin_locus_check.py`, wired
into nothing — because the numbers turned out to justify one, which was not the expected
outcome.

**Verdict in one line: a discriminator exists, it works on the tree as it is, it needs
no re-annotation of anything, and it fires on the exact four values of #97 at the exact
commit that shipped them — but it is a BACK-POINTER check over a 3-day-old convention
with 14 adopters, not a stub audit, and the `heir`-based check the correction imagined
is measurably unusable.**

---

## 0. What the control is for, and what already covers the neighbours

CLAUDE.md read-first item 6: a deliberate loose end is annotated **in code AND** listed
in a locus — `docs/design/stubs.md`, `docs/spines.md` § 3, or ROADMAP Owed/Observed.
**Nothing checks that the two halves agree.**

| control | compares | would it have caught #97? |
|---|---|---|
| `spine-audit` | `spines.md` ↔ code | **No** — it reads `spines.md`'s claims, not `stubs.md`'s completeness |
| staleness sweep | board entries ↔ newer work | **No** — structurally cannot see a thread that was never on the board |
| `doc-topology` | docs ↔ each other | **No** — the marker is in code, not in a doc |
| this control | in-code markers ↔ the loci | **Yes** (§ 5, replayed against history) |

It is the exact inverse of § 3's *"built, and nothing calls it"*: that index asks *what
exists with no consumer*; this asks *what is annotated with no entry*.

---

## 1. The population

**261 `.rs` files** under `crates/`, `plugins/`, `tools/`.

| pattern | sites | files | note |
|---|---:|---:|---|
| `TODO` / `FIXME` / `XXX` / `HACK` | **0** | **0** | see below — this is a finding, not a gap in the search |
| `\bheirs?\b` (word-boundary) | **213** | **52** | the corpus's real loose-end vocabulary |
| `heir` (substring, as `-i`) | 441 | 115 | **inflated by the word "their"** — do not use |
| `STAND-IN` (any position) | **15** | **7** | |
| `STAND-IN` in a doc comment (`///`/`//!`) | **14** | **7** | the discriminator, § 4 |
| `stubs.md` (bare mention) | 91 | 39 | |
| `stubs.md #NN` (ordinal cited) | 33 | **18** | |
| `stand-in` / `standin` (lowercase, prose) | 219 | 73 | mostly the words "stand in"/"stub" in ordinary prose |
| `A-1 ledger` (the second structured form) | 3 | 2 | `dc-api/.../bake.rs`, `.../mass.rs` |

Per crate, `\bheirs?\b` / doc-comment `STAND-IN`:

| crate | `heir` | `STAND-IN` |
|---|---:|---:|
| dc-worldgen | **147** | **1** |
| dc-api | 56 | 10 |
| dc-client | 9 | 3 |
| dc-core | 1 | 0 |

By file kind: `src` 195 · `tests` 11 · `examples` 7.

**Three population findings that change the shape of the problem.**

1. **`TODO` and `FIXME` do not exist in this codebase — zero occurrences in 261 files.**
   The conventional target of every "unfinished work" linter is absent. Whatever this
   corpus does with loose ends, it does with prose, and any control modelled on a
   TODO-scanner is aimed at nothing.
2. **The `heir`-appears-200+-times figure in corrections #97 is right (213 sites / 52
   files, vs its "200+ / ~40"), but its companion figure is low: 18 files cite a
   `stubs.md` ordinal, not "~12".** Corrected here rather than silently, per the same
   doctrine the correction itself invokes.
3. **The `STAND-IN` sigil has spread from 3 files to 7 in one day** (#97 measured three
   on 2026-08-03; the B7 slice added `bodies.rs` and `limits.rs`, B6-a and the clock
   slice added more). It is a live, growing convention — which is what makes betting on
   it reasonable, and also what makes its 3-day evidence base thin.

---

## 2. The precision problem, quantified — census, not sample

**Method: full census, not a sample.** 213 is small enough to read every site, so the
sampling-bias question the brief raised does not arise. All 213 `\bheirs?\b` hits were
dumped with file and line and classified by hand against a stated rule:

- **G — genuine marker.** The site declares that *this* code is a simplification/seam
  and names an heir. Owes a locus.
- **P — prose.** The site talks *about* heirs, the heir mechanism, or another site's
  heir. Owes nothing.
- **T — test or report text.** The token is inside an assertion string, a test's doc
  comment, or a printed probe caption — it *checks* that some other site names an heir.
- **L — landed.** Names an heir that has already arrived.

| class | sites | share |
|---|---:|---:|
| **G** genuine marker | **~100** | **47 %** |
| **P** prose | ~90 | 42 % |
| **T** test / assertion / report string | 21 | 10 % |
| **L** landed heir | 2 | 1 % |

**So a bare `heir` grep is ~47 % precise, and the noise is structural, not incidental.**
The single largest contributor is `deeptime/providers/mod.rs`: **39 of the 213 sites in
one file**, of which **5** are genuine slot declarations (`- *Heir:* …`) and **34** are
the provider-seam design essay — *"granularity follows the heir, not the call site"*,
*"`None` means no heir yet"*, *"routes to `identity_wave_energy` when no heir has
supplied it"*. That file is the corpus's best writing about seams and it is
indistinguishable from a marker to any regex.

**Confidence.** The G/P boundary is judgement at maybe ±10 sites — several module-header
lines legitimately read both ways. The conclusion is insensitive to that: at 47 % ± 5 %
precision the naive check is unusable either way, and § 3 shows the *defect* rate is what
actually kills it, not the precision.

---

## 3. The measured defect rate — and why the naive control fails on it, not on precision

A locus-citation check was run over the containing comment block of all 220
marker-candidate sites (`heir` **or** `STAND-IN`). **Which pointer, if any, the block
carries:**

| pointer target | sites | share |
|---|---:|---:|
| a bare board/sequence tag (`B6`, `E4-1b`, `P11`, `continuation (c)`) | 98 | 44.5 % |
| **`journal/NNNN`** | **91** | **41.4 %** |
| **`stubs.md`** (locus 1) | 76 | 34.5 % |
| `spines.md` / an `A-n`/`S-n` shape (locus 2) | 69 | 31.4 % |
| **a `docs/audits/` design pass** | 56 | 25.5 % |
| `corrections #N` | 21 | 9.5 % |
| **`ROADMAP`** (locus 3) | **10** | **4.5 %** |
| **no pointer of any kind** | **34** | **15.5 %** |

**Only 38.6 % of marker sites cite one of the three loci read-first item 6 names.** The
most-cited target in the tree is the **journal** — which is not a locus — and **ROADMAP,
which is one, is cited by 4.5 %**. *That is a finding about item 6, not about the code:
the corpus's in-code pointers overwhelmingly aim at a **fourth and fifth** target
(journal entries and `docs/audits/*-design.md` passes) that the rule does not recognise.*
A control that demands one of exactly three loci would alarm on a large, well-annotated
population that is behaving reasonably.

**Now the number that decides it.** Hand-classifying the **34 no-pointer sites**:

| | count |
|---|---:|
| prose / assertion strings / meta-references (**false positives**) | **27** |
| genuine markers whose entry exists elsewhere (chase-and-clear) | 6 |
| **genuine markers reaching NO locus (true defects)** | **1** |

The one true defect is `crates/dc-core/src/materials/release_vanilla.rs:310` —
`// carbonaceous-mudstone (organic-rich; heir with the organics rework)`. *"Organics
rework"* returns **zero hits** in `stubs.md`, `spines.md` and `ROADMAP.md`. It is a small
one (an empty release-spectrum row), and it is the entire yield.

> **A `heir`-based control has a false-positive rate of 79 % and surfaces one real defect
> in thirty-four alarms.** CLAUDE.md § Gates is explicit that *a gate that grows five
> minutes gets worked around*; a gate that cries wolf 33 times out of 34 gets worked
> around faster. **This is a null: the check corrections #97 imagined — walk the tree for
> `heir`, diff against the loci — does not work and should not be built.**

**A second, near-miss defect worth naming.** `bake.rs:470`/`:482`,
`gait/geometry.rs:90` and `bake/tests.rs:294` all name *"the first quadruped's design
pass"* as their heir. **`quadruped` appears zero times in `stubs.md`, `spines.md` and
`ROADMAP.md`** — it is recorded only in
`docs/audits/2026-08-02-posture-bake-member0-design.md` (F4). By item 6's letter that is
an unlisted loose end; by the corpus's actual practice it is filed in the fifth locus.
**Flagged, not reconciled** — whether a design audit counts as a locus is a user call,
and it is the same question the 25.5 % row above asks.

---

## 4. The discriminator — yes, and it needs no re-annotation

**Candidates evaluated, against the tree as it is:**

| candidate | works? | rate on today's tree |
|---|---|---|
| the word `heir` | **no** | 47 % precision, 79 % FP on the alarm set (§ 2, § 3) |
| proximity of `heir` to `stubs.md` | **no** | catches filed markers, misses unfiled ones — inverted |
| a `stubs.md #NN` ordinal being present | **no** | 18 of 52 marker files; absence is the normal case, not the defect |
| position (doc comment vs inline) alone | **no** | 195 of 213 `heir` sites are already in `src` doc comments |
| **a required sigil: `STAND-IN` in a doc comment** | **YES** | **14 sites, 7 files, 100 % genuine markers — zero prose** |

**`STAND-IN` in a doc comment is 100 % precise on today's tree and across the sigil's
entire history.** Every one of the 14 sites is a real declaration. The *only* non-marker
occurrence the token has ever had in this repo is
`dc-client/src/body.rs:743` — `// comment's STAND-IN marker.`, an inline `//` reference
back to the doc comment 48 lines above — and **requiring the sigil to sit in a `///` or
`//!` line removes it without a single false negative.** That is the whole discriminator:
one predicate, no new convention, no re-annotation.

**What "resolvable pointer" has to mean, and this is where the design is load-bearing.**
A bare `stubs.md` substring is not enough, and the tree already contains the proof:
`gait/evaluate.rs:221` reads *"(Owed a `stubs.md` entry — the integrator applies; this
slice may not write that file.)"* A substring check reads that as satisfied. **It is a
declaration of debt wearing a citation's clothes** — precisely #97's failure shape one
layer up. The check therefore requires the locus **plus a handle**: an ordinal
(`stubs.md #40`), a section (`stubs.md § 34`), a board tag (`stubs.md B7-a`), or a
hyphenated slug (`` `the-fold-sense-is-declared-because-our-bodies-have-no-front` ``).

*(A first draft of the handle pattern accepted any backticked lowercase token, and
`` `clearance` `` — an ordinary identifier three lines earlier — silently satisfied it on
exactly the one marker in the tree that says its entry is still owed. Recorded because it
is the same class of defect as the substring hole it was written to close.)*

**Recall, stated honestly.** The sigil covers **14 of ~100** genuine markers ≈ **14 %**.
It is essentially a `dc-api`/`dc-client` bodies-arc habit: **dc-worldgen has 147 `heir`
sites and 1 `STAND-IN`.** A simplification whose author never writes the word is
invisible to this control, and always will be. The corpus also carries a *second*
structured form — the **`A-1 ledger`** block in `bake.rs` and `mass.rs`, a bulleted
*"each simplification, its identity, its heir"* list — which is equally greppable and not
covered here.

---

## 5. Would it have caught #97? — replayed against git, not argued

The rule was replayed over **all 268 commits from `d29eb62` (2026-08-01, the sigil's
first appearance) to HEAD**, evaluating every `STAND-IN` site at every commit.

**At `5859976` — gait bake slice one, the commit corrections #97 names as the origin —
there were 6 markers and the control fires 4 alarms:**

```
crates/dc-api/src/bodies/gait.rs:153  **STAND-IN (S1), heir B6.**   <- cadence_scale
crates/dc-api/src/bodies/gait.rs:156  **STAND-IN (S2), heir B6.**   <- duty_exponent
crates/dc-api/src/bodies/gait.rs:159  **STAND-IN (S3), heir B6.**   <- bob_damping
crates/dc-api/src/bodies/gait.rs:165  **STAND-IN (S4), heir B6.**   <- swing_flexion
```

Those are **exactly** the four values #97 is about, **nothing else**, at the commit that
shipped them — and `stubs.md` at that commit topped out at **#42**, with no #43 and no
#44. The other two markers at that commit both cite `stubs.md` properly and stay silent.

**The alarm then persists, unchanged, through every commit since** — the merge to main
(`1ee38b42`), the second slice (`40d1dc23`), and today. Over the sigil's whole life the
alarm set has held **exactly three distinct members**: the four gait knobs, the
`body.rs:743` inline reference (the lone false positive, now excluded by the doc-comment
rule), and `recorder.rs:832` from 2026-08-04.

> **Lifetime false positives across 268 commits: one, and it is excluded by construction.**

---

## 6. What the prototype reports today

`scripts/standin_locus_check.py`, **wired into nothing**. Runs in **166 ms** over 261
files. `python scripts/standin_locus_check.py .` → 14 markers, **7 alarms, 0 false
positives**:

| site | status |
|---|---|
| `dc-api/src/bodies/gait.rs:155,158,161,167` (×4) | **filed as `stubs.md` #44 — and the code still does not say so.** #97's own fix wrote the entry and never stamped the source. A one-directional pointer is not a pointer (read-first item 5); this is that rule landing on its own remediation. |
| `dc-api/src/bodies/gait.rs:132` | says *"`stubs.md` S2 territory"* — no walkable entry; S2 is #44's internal label |
| `dc-worldgen/src/deeptime/recorder.rs:832` | **filed as `stubs.md` #52** (deposition clock, 2026-08-04) — code carries no back-pointer |
| `dc-api/src/bodies/gait/evaluate.rs:221` | says *"Owed a `stubs.md` entry"*; **#43 exists** — the marker is stale in the direction that reads as unfinished |

**Every alarm is real and none is an unfiled stub.** The measured defect the control
finds *today* is not a missing entry — it is that **5 of 14 markers cannot be walked from
the code to their entry**, three days after a correction was written about exactly that
asymmetry. **35 % of the sigil's adopters carry a one-directional pointer.**

---

## 7. Recommendation

**Ship the narrow control; do not build the wide one; change no convention.**

1. **DO NOT build a `heir`-based check.** 79 % false positives, 1 defect in 34 alarms
   (§ 3). It is a null and it should be recorded as one so nobody re-derives it.
2. **DO adopt `STAND-IN`-in-a-doc-comment as the checkable form.** It costs **nothing**:
   14 sites already comply, 100 % precision, no file needs re-annotating. This matters —
   CLAUDE.md records `JUSTIFIED-BY` dying at 3 uses / 0 in `crates/` because it asked
   authors to restate something. **This asks for nothing new; it makes an existing habit
   load-bearing.**
3. **Venue — a script the sweeps run, not a gate.** Recommended, in preference order:
   - **(a) A recorded command, run by `spine-audit` and by `wrap`.** This is the shape
     `spines.md` § 3 already uses for its own row count (*"The command is recorded here
     so the next sweep runs a check instead of re-typing a number"*), and it earned that
     shape by getting the number wrong three times. Cheapest, zero build cost, no gate
     latency, and it lands where a human is already reading the loose-end board.
   - **(b) A `#[test]` in `dc-api`, per the `test = true` mechanism.** Makes it a hard
     gate at ~0.2 s. **Recommended only after (a) has run clean for a while** — a gate
     that red-lights a merge over a doc-comment citation, on a 3-day-old convention with
     14 adopters, is the shape that gets worked around, and CLAUDE.md is explicit that a
     worked-around gate is worse than none.
   - **(c) A PreToolUse/commit hook.** Rejected: the alarm is not per-command, and the
     hook surface here is already carrying the build mutex.
   - **⚠ Whichever venue: the choice is a user call.** The brief forbids wiring, and
     turning a survey into a gate is a standing-orders decision, not a slice's.
4. **The immediate, separate fix the survey uncovered — stamp the 5 back-pointers.**
   `gait.rs:132,155,158,161,167` → `stubs.md #44`; `recorder.rs:832` → `#52`;
   `evaluate.rs:221` → `#43` (and delete *"Owed"*). Small, mechanical, and it is
   corrections #97's own remediation finishing the second direction. **Not done here** —
   this brief is read-only on source.
5. **Two things to put to the user, both surfaced by the numbers, neither reconcilable
   by an agent:**
   - **Item 6 names three loci; the tree points at five.** journal/NNNN (41.4 %) and
     `docs/audits/*-design.md` (25.5 %) both out-cite ROADMAP (4.5 %). Either item 6
     widens, or a large well-annotated population is out of compliance with a rule
     nobody is actually following.
   - **The `A-1 ledger` form** (`bake.rs`, `mass.rs`) is a second structured convention
     doing the same job. Fold it into the sigil, or check it too.

**No convention change is required for the recommendation above.** If one were ever
wanted — extending the sigil to dc-worldgen's 147 `heir` sites — the cost is ~100 hand
classifications and ~50 edits across a crate a parallel session is live in, for a
population whose measured unfiled-defect rate is **1**. That is not worth it, and the
honest answer is to leave dc-worldgen's prose alone.

---

## 8. What this control would NOT catch — the boundary, stated explicitly

- **A simplification nobody annotated.** It checks markers that were *written*. The
  unannotated stand-in is invisible to it, to `spine-audit`, and to everything else. It
  is the residual risk and no mechanism proposed here reduces it.
- **A stand-in annotated in prose without the sigil** — ~86 % of genuine markers today,
  and essentially all of dc-worldgen.
- **Whether the locus entry actually exists.** It reads a pointer's *shape*, not its
  target. `stubs.md #99` would pass. *(Closing this needs a resolver against `stubs.md`
  headings — cheap, deliberately out of scope here, and it would have changed no number
  in this survey.)*
- **Whether the entry still describes the code.** That is the staleness sweep's job.
- **Whether the entry contradicts another doc.** That is `doc-topology`'s job.
- **Whether the spine claims about it still hold.** That is `spine-audit`'s job.
- **Markers in `.md`.** 166 `STAND-IN`/`stand-in` occurrences across 56 docs are out of
  scope; the docs are covered by the three existing sweeps.
- **The A-1 anti-shape itself** — *"a stand-in becomes the definition"*. This control
  says a stand-in is *filed*. It says nothing about whether it is *quietly becoming
  load-bearing*, which is the failure `spines.md` A-1 exists for.

---

## 9. Could not determine — flagged, not reconciled

- **Whether a `docs/audits/*-design.md` pass counts as a locus.** 25.5 % of markers
  behave as though it does; item 6 says it does not. Ruling needed. The `quadruped`
  markers (§ 3) hang on it.
- **Whether `release_vanilla.rs:310`'s "organics rework" is owed an entry or is simply a
  content note.** It reaches no locus; whether it should is a roster question, and
  `crates/dc-core/src/materials/` is out of scope for this brief (parallel session).
- **The G/P boundary at ~±10 sites** (§ 2). Judgement, stated, conclusion insensitive.
- **A doc-topology-flavoured nit, unreconciled:**
  `.claude/skills/session-workflow/SKILL.md:75-79` reads *"marked four `GaitKnobs`
  values … mentioned **one** in its report … The **other four** reached no locus"* —
  four minus one is not four. `corrections.md` #97's own mechanism paragraph has it
  right (`swing_gain` flagged in prose → #43; four *other* markers written silently → 5
  items total). The skill's summary is internally inconsistent. **Not edited** — skills
  are out of scope here.
- **Nothing was built or gated.** No cargo was run; none was needed.
