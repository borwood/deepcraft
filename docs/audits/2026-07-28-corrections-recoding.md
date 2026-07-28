# Independent adversarial re-coding of `journal/corrections.md` (2026-07-28)

**Coder:** a background agent with no prior involvement in any of the 67 entries,
working in an isolated worktree at `5485eaf` (main), read-only except for this file.

**Confirmation of the blind:** I did **not** open, grep, or `git show`
`docs/design/corpus-knowledge-notebook.md` or `docs/design/corpus-knowledge-evidence.md`,
nor `docs/audits/2026-07-26-doc-topology-sweep.md` § Verdict, nor the ROADMAP
"CORPUS ADDRESSABILITY" entry. No conclusion below is downstream of any of them.
I also did not read the first coding's counts before producing mine.

**Denominator note, up front.** The brief describes "all 65 entries plus the
unnumbered #63b" = 66 rows. The file on main carries **66 numbered entries plus
#63b = 67 rows**: **#66 was added 2026-07-28** (commit `0f5bd34`, journal/0121)
and may postdate the first coding. If the two codings report different
denominators, that is why. All percentages below are over **67**.

**Zero cargo invocations. `target/.agent-build.lock` untouched.**

---

## 1. The table

Rule 1 = where the falsifier already lived at the moment the claim was recorded.
Rule 2 = STALE (true when written, a pointable change made it false) ·
BF-U (born false, and not exercisable at the time) · BF-C (born false, and the
refuting fact already existed and was reachable).

`(2nd: X)` marks a genuine dual; the bucket outside the parentheses is primary.

| # | Rule 1 | Rule 2 | Justification |
|---|---|---|---|
| 1 | LITERATURE (2nd: USER) | BF-C | `depth_bias` never wrote depth — that is Bevy/wgpu's documented semantics, outside this repo; the user's eye supplied the symptom, not the mechanism. |
| 2 | CODE (2nd: MEASUREMENT) | BF-C | The system ordering that puts `position_chunks` ahead of the streamers was in the schedule and readable. |
| 3 | THE USER | BF-C | Only the user's review of the screenshots established the camera was inside solid; no instrument (`eye_in_solid`) existed yet — but a block query at the camera pose was available. |
| 4 | LITERATURE | BF-C | postcard's positional wire format is documented in the crate, not derivable from our tree. |
| 5 | CODE | BF-C | `block_in_column` calls `surface_height_m` directly — one field, so a same-column disagreement is impossible by construction; a code read settles it. |
| 6 | CODE | BF-C | `CELL_VOXELS = 16384` and `climate_at`'s bilinear interpolation were both in the tree; the claim named the wrong cell. |
| 7 | LITERATURE (2nd: MEASUREMENT) | BF-C | `env!` bakes a compile-time path — Rust semantics; the shared-`CARGO_TARGET_DIR` practice was already in CLAUDE.md from `99f0a25` (2026-07-18), the same day as S5-results. |
| 8 | MEASUREMENT | BF-C | "Minutes, paid once" and "erosion is bounded" are cost/boundedness claims; only S9's run could refute them. |
| 9 | MEASUREMENT | BF-C | The determinism tax per phase (flood 65–72 %, no byte-identical parallel form) is a measured quantity. |
| 10 | MEASUREMENT (2nd: CODE) | BF-C | Reproduction is what falsified it; but the mechanism half (the ceiling reads `ColumnRec.heights`, same lattice) was a pure code read. |
| 11 | THE USER (2nd: MEASUREMENT) | **STALE** | Change event verified: claim shipped in `64729c5` (2026-07-18) when far chunks were volumetric shells; `c087d39` (2026-07-19) made the far field a hollow sheet and the same offsets became see-through. |
| 12 | CODE (2nd: MEASUREMENT) | BF-C | `deeptime::run` → `run_with(.., false)` (scalar) vs production's `build_field(.., true)` was a two-hop code read; the parallel path already existed (S9b, 2026-07-19) when S10 published on 2026-07-20. |
| 13 | THE USER (2nd: SAME-ARTIFACT) | BF-C | Only the user could disown the quote — but visuals.md § Mood's own "real darkness underground, no floaty ambient minimum" contradicted the new bullet in the same document. |
| 14 | MEASUREMENT (2nd: CODE) | BF-C | Links 0-or-1 across seven scenarios is measured; the reason (air connectivity already carries the relation, so two bodies in one component *are* one body) is a substrate fact. |
| 15 | MEASUREMENT (2nd: LITERATURE) | BF-C | Falsified by its own harness output; the boundary-condition reasoning (a closed box with one drain has no local equilibrium) is textbook. |
| 16 | CODE (2nd: MEASUREMENT) | BF-C | The property sheet in the repo says mudstone 0.95 > sandstone 0.85 cohesion; multiplying by it demonstrably narrows the contrast — arithmetic on committed data, no run needed. |
| 17 | MEASUREMENT (2nd: CODE) | BF-C | The null needed the probe; the mechanism (diffusion is flux-limited by available regolith) is visible in `erosion.rs`'s limiter. |
| 18 | THE USER (2nd: SAME-ARTIFACT) | BF-C | The user's "the sun does not move whatsoever" is the falsifier — but journal/0030's own committed asset `0030-flank-before-fullbright.png` (a featureless grey field) proves the control is blind to shape. |
| 19 | DISTANT DOC | BF-C | S4-results' "fixed 0.35 time-of-day" was the falsifier — but the author had quoted it *hours earlier* in `light.md`, a shape neither DISTANT DOC nor SAME-ARTIFACT names (§ 5.2). |
| 20 | CODE | BF-C | Deep-time erosion re-derives drainage every iteration; the design agent found it by reading the source. |
| 21 | MEASUREMENT | BF-C | The discriminating observation was an experiment (`-p` target passing where `--workspace` failed); claim 2 died to the full capture's E0432/E0560. |
| 22 | CODE | BF-C | `collapse.rs` bares to Dirt below precip 0.10 while the Hadley belt sits at 0.2–0.3 — a threshold comparison in the source. |
| 23 | MEASUREMENT | BF-C | "The amplitude is the bottleneck" could only be refuted by the same-seed A/B walk. |
| 24 | MEASUREMENT (2nd: CODE) | BF-C | S13's 0.4 % agreement is the falsifier; but "bilinear is exact at cell centres and the two spacings differ by 0.17 %" is desk arithmetic. |
| 25 | MEASUREMENT | BF-C | Only S13's relief-vs-window table over median/steepest sites could show the sample was the extremum. |
| 26 | CODE (2nd: MEASUREMENT) | BF-C | `tour_map`'s deflation station is a **max-ΔH** search — reading the probe source shows the label is a rate, not a state. |
| 27 | MEASUREMENT | BF-C | The stale serve was established by the test count moving 99 → 102 after a clean; nothing in the repo said it. |
| 28 | CODE (2nd: MEASUREMENT) | BF-C | `soil_depth_probe::STATIONS` holds **metres** and divides by 0.9 — the unit is in the probe source that produced the number. |
| 29 | CODE | **BF-U** | Wrong when written (`440ee85`, 2026-07-19: culling was already block-tier boolean `neighbor.is_solid()`) and unexercisable (no sub-8 partials existed); `f286893`/journal/0055 made it *reachable*, not false. |
| 30 | CODE (2nd: MEASUREMENT) | BF-C | A dug void enters the coarse summary as an exact signed integer delta through the audited write path — a code property, not a measurement. |
| 31 | MEASUREMENT (2nd: CODE) | BF-C | The 288 m-beyond plug and 972/1215 already-joined basins are measured; that graph reachability has no halo is a structural fact. |
| 32 | LITERATURE (2nd: CODE) | **BF-U** | Rust guarantees `fn`-pointer address uniqueness in *neither* direction (external language semantics); the failure was latent on main and only a pure code-motion refactor changed the inlining that masked it. |
| 33 | CODE | BF-C | The budget derives from `UNLOAD_RADIUS_M = FULL_DETAIL_RADIUS_M + 32`, a constant 160 m — one grep. |
| 34 | MEASUREMENT | BF-C | Only a 700-jump run longer than the sawtooth period could distinguish flat from oscillating. |
| 35 | CODE (2nd: DISTANT DOC) | **STALE** | Change event verified: comment written `75445f2` (2026-07-20, journal/0026), addressed stochastic rounding landed `f286893` (2026-07-21 17:29, journal/0055), correction filed `167ca02` (2026-07-22). True when written. |
| 36 | MEASUREMENT | BF-C | That thickness and burial depth are near-complementary in *this* record is a property of the record; only the census showed collapse rather than redistribution. |
| 37 | MEASUREMENT | BF-C | Observed directly, with the log's `Compiling dc-worldgen` and no `Compiling dc-core` above it; not derivable from the tree. |
| 38 | CODE | BF-C | The always-on deep field landed `68059b4` (2026-07-19); the claim was made 2026-07-22 against `pregen/mod.rs`'s unconditional call. |
| 39 | MEASUREMENT | BF-C | Caught by the extracted `CoarseField`'s own test — but the inverse-CDF argument (`F` steep through ½ pushes any off-½ share further off) is pen-and-paper (§ 5.1). |
| 40 | CODE | **UNRESOLVED** | § 4.1 — the audit's quoted string is *verbatim* the comment as it stood until `11d4385` (2026-07-21 04:33), and that commit's own message calls that text a lie. I cannot attribute STALE vs BF-C without knowing which checkout the audit read. |
| 41 | MEASUREMENT (2nd: CODE) | BF-C | The faithful sweep is the falsifier; the clamp `max_inc = r − floor` (`erosion.rs:1149`) is a code read that predicts it. |
| 42 | CODE | BF-C | The near-field mesher already routed `contents → dominant_material` and never calls `block_layer`; a grep of the mesher settles both halves. |
| 43 | MEASUREMENT | BF-C | `chunk.gen` at 0.1 % / 14 µs per call is a profile; the premise was never measured, only asserted. |
| 44 | CODE (2nd: THE USER) | BF-C | `fully_resolved`'s only non-test caller is `--bench-storage`; the user's question triggered the grep that settled it. |
| 45 | CODE | BF-C | `interp_select_draw` hashes **absolute** chunk corners and interpolates — reading the function shows it is world-anchored and C0. |
| 46 | MEASUREMENT | BF-C | 0.04 m against 0.9 m voxels, `quantize_to_eighths → 0`, is a production-scale probe result. |
| 47 | CODE (2nd: THE USER) | BF-C | `weather_column` runs once, after the run, over the finished record — visible in `weather_inventory.rs` / `build_ledgers`; the user's question is what made anyone look. |
| 48 | THE USER | BF-C | The true pose existed **only** in an out-of-repo transcript the user recovered; nothing in the corpus held it (the diagnosis audit even listed the pose as unfilled). |
| 49 | CODE | BF-C | `chunk_contents` returns `Some` if *any* voxel in the 32³ is recorded, and `classify.rs:31-45` forbids the operation by name — all in the tree. |
| 50 | CODE (2nd: MEASUREMENT) | BF-C | "Voxel contents carry no provenance" is a type-level fact; the re-measure over 247 columns quantified what that fact already invalidated. |
| 51 | MEASUREMENT (2nd: CODE) | BF-C | "Still a diggable seam on Medium" is a magnitude claim only a census could refute; that `tests/geotherm.rs::production_field()` builds a different seed/extent was readable in the test. |
| 52 | MEASUREMENT | BF-C | Mutual information against a measured estimator floor and a contact-plane autocorrelation are irreducibly experimental. |
| 53 | SAME-ARTIFACT | BF-C | S20 § 4.3's final paragraph predicted the one site that would move — two subsections from the nine-row table that said nine. |
| 54 | SAME-ARTIFACT | BF-C | flow.md § 2.4 states that "never crosses a divide" binds FREE/surface only; § 2.6 asserted the opposite dependency. One document, both halves. |
| 55 | MEASUREMENT | BF-C | 0.109 % of routing vs creep's 918× is a probe result; nothing written could have said it. |
| 56 | LITERATURE | BF-C | Explicit and correct in the entry: "it took an anchor from outside the corpus." The internal division was computable; the published band was not in the repo. |
| 57 | SAME-ARTIFACT (2nd: DISTANT DOC) | **BF-U** | The falsifier is the same enum's own doc ("a thin event bed, capped at 0.04 m") plus `as_deposited`'s own "made where it lies" reasoning applied to three sibling variants; unreachable at 2b's 0.109 % until creep carried identity. |
| 58 | SAME-ARTIFACT (2nd: CODE) | BF-C | Verified: `journal/0109:99` says the old rule picks steepest **drop**; `:115` claims `p → ∞` is that rule **exactly** — 16 lines apart, with the falsifying test named in between. |
| 59 | CODE | BF-C | `0.002 = 1.25 × k_transport` is arithmetic over `erosion.rs`'s `ENERGY_LOW_MED` and `grid.rs:149`'s `k_transport`; refutable at the desk before anything moved. |
| 60 | SAME-ARTIFACT (2nd: MEASUREMENT) | BF-C | **§ 4.2.** `journal/0111:119-129` states the refuting mechanism ("±35 m of sea-level cycling shuffles cells between land and sea and the shoreline is not a clean control surface") in the two sentences *above* the claim, then concludes "That is what licenses reporting it." |
| 61 | MEASUREMENT (2nd: THE USER) | BF-C | The concavity table is the falsifier; the walk prompted it. The "relief is a global extremal statistic" half is armchair (§ 5.1). |
| 62 | THE USER | BF-C | "Absolutely pockmarked… honeycombed landscape" from a live flight; three instruments in the tree agreed with each other and were all wrong. |
| 63 | MEASUREMENT (2nd: CODE) | BF-C | The 4× refinement ladder is the falsifier; item (i) was a grep — and is itself imprecise (§ 4.5). |
| 63b | MEASUREMENT (2nd: CODE) | BF-C | Ablation runs on the arm nobody expected to matter; the sign of the loop closure was also readable in `erosion.rs::isostasy`. |
| 64 | CODE | BF-C | A read-only trace: `history.rs:82`/`:84-88` pass `vec![]` for `agent_home`, doc'd at `world.rs:132-133`. Nothing ran. |
| 65 | SAME-ARTIFACT (2nd: THE USER) | BF-C | journal/0090 is one of the three cited claim sites and contains the refutation ("you have to name each revision as a distinct resource"); the user's challenge made someone read it. |
| 66 | CODE | BF-C | Two lines of arithmetic on the samplers' own `for` loops: `cz ∈ [−60, 42]` against a nearest post at `cz = −727`. |

---

## 2. Totals

### Rule 1 — where was the falsifier?

| bucket | n | % |
|---|---:|---:|
| MEASUREMENT | 25 | 37.3 % |
| CODE | 24 | 35.8 % |
| SAME-ARTIFACT | 6 | 9.0 % |
| THE USER | 6 | 9.0 % |
| LITERATURE | 5 | 7.5 % |
| DISTANT DOC | 1 | 1.5 % |
| **total** | **67** | **100 %** |

Three derived figures that matter more than the buckets:

- **Already in the repo, no experiment required (SAME-ARTIFACT + CODE + DISTANT DOC): 31/67 = 46.3 %.**
- **In-repo falsifier as primary *or* secondary: 44/67 = 65.7 %** (adds #10, #13, #14,
  #17, #18, #24, #31, #32, #41, #51, #62, #63, #63b).
- **SAME-ARTIFACT is only 6/67 (9 %)** — and I found a seventh (#60) whose own header
  attributes the falsification elsewhere. Even at 7/67, "claim and refutation in one
  document" is a minority shape, not the corpus's signature.
- **DISTANT DOC is 1/67**, and that one (#19) is a bad fit (§ 5.2). On my coding,
  *almost nothing here was refuted by a document the author had no reason to open.*

### Rule 2 — stale, or false when written?

| class | n | % |
|---|---:|---:|
| GENUINELY STALE | 2 | 3.0 % |
| BORN-FALSE, UNREACHABLE | 3 | 4.5 % |
| BORN-FALSE, CHECKABLE | 61 | 91.0 % |
| **I cannot attribute** | 1 | 1.5 % |
| **total** | **67** | **100 %** |

**GENUINELY STALE = #11 and #35 only**, and I can point at the change commit for
both (`c087d39`; `f286893`). #40 is the only other candidate and I have left it
unresolved rather than force it (§ 4.1). **BORN-FALSE, UNREACHABLE = #29, #32, #57.**

I was deliberately hostile to the STALE bucket and checked every entry whose prose
reads *"this was fine until X landed"*:

- **#7** — the shared-`CARGO_TARGET_DIR` practice was in CLAUDE.md from `99f0a25`
  (2026-07-18, day one), the same day as S5-results (`d57f423`). There is no
  pre-worktree era for the claim to have been true in. Not stale.
- **#27** — same: no era in which a green gate was trustworthy under a shared
  target dir. Not stale.
- **#42** — the near field was collapsed by journal/0010 (2026-07-19); the claim was
  freshly made 2026-07-23 against a mesher the code had already outrun. Not stale.
- **#12** — S9b's parallel path (2026-07-19) predates S10's table (2026-07-20). Not stale.
- **#32** — a claim about what *can* happen is falsified by possibility, not by
  occurrence. The fold that masked it was luck, not truth. BF-U, not stale.
- **#57** — its own header says "false since the function was written." Not stale.
- **#66** — a prediction about a future action, false at the moment of writing.

**The load-bearing consequence:** on my coding, a staleness watcher — an instrument
that requires a change event to fire — is the right instrument for **2 of 67 entries
(3 %)**, or at most 3 (4.5 %) if #40 resolves to STALE.

---

## 3. ⚠ Genuinely ambiguous entries — 29 of 67 (43 %)

This is the section I most want read. A single coder forced to pick on these
produces a number with a ±29-entry lever under it. Grouped by which rule is
ambiguous; **#32 appears in both.**

### 3a. Rule 1 ambiguous (20)

| # | competing readings |
|---|---|
| 1 | **LITERATURE** (Bevy's `depth_bias` semantics) vs **THE USER** (the continued z-fight report is what killed it). Mechanism and symptom lived in different places. |
| 10 | **MEASUREMENT** (reproduction) vs **CODE** (the ceiling reads `ColumnRec.heights`; the 1/0.9 unit factor is documented). The entry itself splits them. |
| 13 | **THE USER** (only they could disown the quote) vs **SAME-ARTIFACT** (visuals.md § Mood already asserted the opposite in the same file — a doc-topology pass could have fired without the user). |
| 14 | **MEASUREMENT** (links 0-or-1 over seven scenarios) vs **CODE** (the merge semantics that make a link impossible within a component). |
| 15 | **MEASUREMENT** (its own output) vs **LITERATURE** (closed-box boundary conditions are textbook hydrogeology). |
| 16 | **CODE** (arithmetic on the committed property sheet: 0.95 vs 0.85) vs **MEASUREMENT** (the +0.0 % null). I lean CODE and would not fight. |
| 17 | **MEASUREMENT** (the null) vs **CODE**/**LITERATURE** (flux-limited hillslopes are both in `erosion.rs` and in the geomorphology literature). Arguably three-way. |
| 18 | **THE USER** (the sun statement) vs **SAME-ARTIFACT** (the entry's own committed fullbright asset is a grey field) vs **DISTANT DOC** (S4-results). Genuine three-way. |
| 19 | **DISTANT DOC** vs *no bucket fits* — the falsifier was a document the author wrote hours earlier (§ 5.2). |
| 24 | **MEASUREMENT** (S13) vs **CODE** (bilinear exactness + 0.17 % spacing is desk arithmetic). |
| 26 | **CODE** (the station is a max-ΔH search — read the probe) vs **MEASUREMENT** (H = 10.66 m). |
| 31 | **MEASUREMENT** vs **CODE** (unbounded graph reachability; `HostWorld` has no absent state). |
| 32 | **LITERATURE** (Rust's non-guarantee) vs **CODE** (`pub use` of an `#[inline]` fn, visible) vs **MEASUREMENT** (only a build exposed it). Three-way; low confidence. |
| 39 | **MEASUREMENT** vs *no bucket* — pure inverse-CDF derivation (§ 5.1). |
| 41 | **MEASUREMENT** (the sweep) vs **CODE** (the incision clamp). |
| 47 | **CODE** (the one-shot placement is in the source) vs **THE USER** (the question, plus the domain knowledge that weathering is continuous). |
| 50 | **CODE** ("voxel contents carry no provenance" is a type fact) vs **MEASUREMENT** (the 247-column re-measure). |
| 51 | **MEASUREMENT** (only a census could find zero coal) vs **CODE** (`production_field()`'s wrong seed/extent, readable in the test). Two different claims in one entry. |
| 61 | **MEASUREMENT** (the concavity table) vs **THE USER** (the walk) vs *no bucket* (relief-is-extremal is armchair). |
| 63b | **MEASUREMENT** (ablation) vs **CODE** (the coordinator read `isostasy` and got only the *sign* wrong — the code was the falsifier too). |

### 3b. Rule 2 ambiguous (10)

| # | competing readings |
|---|---|
| 3 | **BF-C** vs "not reachable" — the refuting query was possible, but the project shipped `eye_in_solid` as the *fix*, which is weak evidence it was not conveniently checkable. |
| 7 | **BF-C** vs **BF-U** — the counterexample needs a worktree to be *deleted*. S5-results and the shared-target-dir rule landed the same day, so I cannot cleanly say the practice postdated the claim. |
| 11 | **STALE** vs **BF-C** — the claim's *consequence* ("no visible gaps") was true when written; its *mechanism* (per-tile differential translation) was false from `64729c5`. And the entry conflates two claims (§ 4.6). |
| 27 | **BF-C** vs **STALE** — "implicit in every gate run this project has ever done" has no true era, but the failure mode requires siblings. |
| 29 | **BF-U** vs **BF-C** — the boolean culling was right there to read; only the *counterexample* was unproducible. |
| 32 | **BF-U** vs **BF-C** — and the change that made it reachable was pure **code motion**, which is neither the "magnitude" nor the "feature" change the brief's BF-U definition names. |
| 40 | **UNRESOLVED** — § 4.1. |
| 48 | **BF-C** with an asterisk — the refuting fact existed, but *outside the repo* (a transcript). Rule 2's "reachable" is silent on repo-external facts. |
| 59 | **BF-C** vs **BF-U** — refutable by arithmetic the day it was written, but harmless until `k_transport` moved. |
| 60 | **BF-C** vs **BF-U** — same shape: the reasoning error was checkable, the *numerical* falsifier needed the 45× regime. |

**My working rule for the BF-U/BF-C boundary**, stated so it can be argued with:
BF-U means *the world could not have produced the counterexample at the time*
(#29 no sub-8 partials; #32 the fold held; #57 no transported organic could win an
argmax at 0.109 %). BF-C means *the error was in the reasoning, and the reasoning
was auditable that day* (#59, #60). A coder drawing the line at "the harm was not
yet realised" instead would move #59, #60, #7 and possibly #51 into BF-U and
roughly **double the BF-U count**. That is the sensitivity.

---

## 4. Entries whose own account is wrong or incomplete

### 4.1 #40 — the audit did not misquote. The comment was fixed under it.

`journal/corrections.md:1289-1303` says the 2026-07-22 seam inventory
*"quoted it as 'the collapse tier reads them', dropping the 'WILL' and the
'currently consumed by nothing' clause"*, and draws the lesson *"an audit that
paraphrases a comment can manufacture the very defect it reports."*

The diff at `11d4385` (2026-07-21 04:33) reads:

```
-    /// grade axes the collapse tier reads (§ 6.4). Empty when tectonic history is
+    /// grade axes the collapse tier WILL read (§ 6.4): exported and, as of U8,
```

The audit's quoted string (`docs/audits/2026-07-22-seam-inventory.md:196`,
*"the metamorphic-grade axes the collapse tier reads"*) is **character-for-character
the comment as it stood until that commit.** And `11d4385`'s own commit message
says: *"Fixed the **lying** field.rs doc-comments that claimed the collapse tier
'reads' these axes (stubs.md section 4)."* **The project itself classified that
text as a lie.** The A-2 the audit reported was live until 2026-07-21 04:33.

Two readings remain, and #40 acknowledges neither:

1. The audit read a checkout predating `11d4385` — a **stale read**, and exactly
   what a staleness watcher is for.
2. The audit read the fixed text and reconstructed the earlier wording verbatim
   while paraphrasing — possible, but "the paraphrase reproduces the exact prior
   string" is a coincidence worth stating rather than assuming.

Timing does not settle it: the audit cites journal/0055 (`f286893`, 2026-07-21
17:29) and corrections #29 (`4f3c941`, 2026-07-21 20:05), so it was authored
**after** the fix — which favours reading 2 — yet its quotation matches the
pre-fix text exactly, which favours reading 1. **I cannot attribute this**, and I
have left #40's rule-2 cell UNRESOLVED rather than pick.

Why it matters beyond one row: #40 is the file's *only* entry whose lesson is
"distrust the sweep," so it is the entry most likely to be cited against
corpus-checking tooling. On the evidence above, its transferable lesson should be
joined by — or possibly replaced with — **"an audit's code reading has a
timestamp; date it and re-verify against HEAD before acting on it."** A different,
and cheaper, control.

### 4.2 #60's falsifier was in the same section as the claim, and the entry says otherwise

#60's header attributes its falsification to *"journal/0114, by pushing the fluxes
up"* — a measurement in a later regime. But `journal/0111:119-129`, the section
that makes the claim, reads:

> D1 … *"inherits a real uncertainty: the land/sea budget closes to within
> 218,528 m, which is 90 % of the export term itself, **because ±35 m of
> sea-level cycling shuffles cells between land and sea and the shoreline is not
> a clean control surface**."* … *"Two instruments that share no arithmetic and
> can fail in unrelated ways land on the same number. That is what licenses
> reporting it."*

The double-count mechanism #60 discovers is **stated verbatim two sentences above
the conclusion it invalidates**, and then set aside as a handled uncertainty.
This is the SAME-ARTIFACT shape in its purest form, mis-filed as a measurement
finding. On my coding it takes SAME-ARTIFACT from 6 to 7 of 67.

### 4.3 #56 still asserts, unstruck, exactly what #60 withdraws

`journal/corrections.md:1949-1952` (inside #56) reads *"**Two independent
instruments agree.** … They agree **to 2.4 %**."* There is no strike, no `⚠`, and
no pointer to #60 — which sits 220 lines later in the same file for the sole
purpose of withdrawing that sentence. A grep for `#60` in `corrections.md`
returns nothing outside #60's own heading.

**corrections.md contains a live self-contradiction**, in the one file whose job is
to stop claims being re-derived. #54 established the doctrine that applies (*"a
correction that lives only in this file is a correction the next author does not
meet"* — struck at source in four places). #60 did not follow it, and the site it
most needed to strike was **its own neighbour**.

### 4.4 #35's "eleven days earlier" is wrong by an order of magnitude

`corrections.md:1129-1131`: *"had been built **eleven days earlier**, by a slice
aimed at something else entirely."* Verified:

| event | commit | date |
|---|---|---|
| the "cannot exist" comment written | `75445f2` (journal/0026) | 2026-07-20 |
| addressed stochastic rounding built | `f286893` (journal/0055) | 2026-07-21 17:29 |
| correction filed | `167ca02` (journal/0063) | 2026-07-22 |

**One day, not eleven.** Not pedantry: #35 is one of only two genuinely stale
entries in the file, and the length of its silent window is its entire argument
("a decision not to build gets a paragraph in a doc comment and no watcher"). An
eleven-day silence and a one-day silence license different instruments — the real
interval is short enough that *the same-session corpus sweep*, not a background
watcher, was the available control.

### 4.5 #63's item (i) is falsified by the grep it invokes

`corrections.md:2320-2323`: *"**`myr_per_epoch` does not exist.** There is no such
knob anywhere in the tree… A discriminator specified against a knob nobody has
ever grepped for is a discriminator nobody has run."*

`myr_per_epoch` exists at
`crates/dc-worldgen/examples/denudation_probe.rs:117`, added `f7871f2`
(2026-07-26) — **in the very probe that produced journal/0111's numbers**, i.e.
the instrument the hypothesis's author was working from. The substantive point is
correct (it is derived — `MYR_PER_RUN / cfg.iterations` — not a tunable, so
"halving it" means doubling `iterations`, which is what (ii) then does). The
rhetoric is not.

### 4.6 #11 codes two claims of different classes as one row

The original claim (`64729c5`, 2026-07-18) is genuinely stale. The
*re-affirmation* the entry also indicts — FF2a/journal-0023's "crack class cured
inherently — same-level seams watertight" — landed at `c93d75f` (2026-07-19),
**after** `c087d39` made the far field a hollow sheet, and was therefore born
false. One row, two classes, and only the flattering one is visible in the entry's
framing.

---

## 5. Failure shapes neither rule captures

### 5.1 ARMCHAIR DERIVATION — the biggest distortion in rule 1

A large population here was refutable by **pen-and-paper reasoning over facts
everyone already had**: no artifact to open, no experiment to run, no literature
to fetch. Rule 1 has no bucket for it, so each lands in MEASUREMENT (because that
is what eventually caught it) — which systematically **overstates how much
instrumentation this corpus needed**.

- #39 — an inverse CDF makes `F` steep through ½, so an off-½ share is pushed further off.
- #61 — relief is `max − min`; you can shuffle every interior cell and leave it unchanged.
- #63 — saturation is evidence *against* a CFL problem; a flux limiter makes the transfer a function of inventory, not `rate × dt`.
- #66 — two lines of arithmetic on a sampler's own `for` loop bounds.
- #34 — a window shorter than the period cannot distinguish flat from oscillating.
- #25, #26 — an extremum is the blind place to sample; an extremum of a *rate* is not an extremum of a *state*.
- #59 — a threshold and the quantity it thresholds carrying different dimensions.

That is **7–8 of the 25 MEASUREMENT rows.** If the question behind this exercise is
"what instrument would have caught these," the honest answer for this
sub-population is *a reviewer with a pencil*: no watcher, differ or probe appears
in it anywhere.

### 5.2 THE AUTHOR'S OWN RECENT OUTPUT

#19's falsifier was in a document the author had written **hours earlier in the
same session**. That is not SAME-ARTIFACT (different file) and not DISTANT DOC
(*"a different document the author had no particular reason to open"* — they had
every reason; they wrote it). #19's own lesson 3 names it: *"check new claims
against what you wrote today."* A taxonomy that cannot name it cannot count it,
and this is the shape most specific to how this project actually works.

### 5.3 A STALE READ IS NOT A STALE CLAIM

#40, and arguably the neighbourhood of #21/#37: the artifact was authored **after**
the change but from a snapshot that predated it. Rule 2 asks whether the claim was
true *when written*; here it was true of *the tree the author read* and false of
the tree at the same wall-clock instant. Neither STALE nor BORN-FALSE fits. Note
the irony: this is `CLAUDE.md` § Gates' cross-worktree stale-artifact failure —
forty lines of doctrine about builds — reappearing in the **documentation**, where
no `cargo clean` exists.

### 5.4 CLAIMS WITH NO ARTIFACT AT ALL

Four entries were never written as sentences: #55 (*"never written as a sentence,
which is why it survived"* — distributed by implication across
`material-behavior.md` § 13, `flow.md` § 8 and a brief), #56 (the same, explicitly),
#31 (the falsified part was *"the unstated inference"*), and #63b (never entered
the corpus). **Any instrument that diffs artifacts against each other is
structurally incapable of firing on these** — and #56 is the single most expensive
entry in the file (~1000×). 4/67 = 6 % of the corpus, weighted very heavily toward
cost.

### 5.5 IT WAS THE INSTRUMENT, NOT THE WORLD

Roughly a third of the file is not a false claim about the world at all — it is a
false claim about the **measuring apparatus**: #7, #15, #21, #26, #27, #34, #37,
#46, #50, #51, #53, #61, #62, plus the harness halves of #12 and #28. Neither rule
has this axis, and it is the one that predicts the control: an instrument defect is
invisible to *every* corpus watcher (staleness, doc-topology, spine-audit) and
visible only to a **second, differently-shaped instrument**. #62 is the purest case
— two probes and a gated assertion agreed with each other and were all wrong for
one structural reason; a person flying over the terrain was right in one sentence.

### 5.6 SEVERITY IS ABSENT FROM BOTH RULES

#40 (a misread comment, zero code wrong) and #56 (the world's clock off by ~1000×,
invalidating an arc) are one row each. #65 cost three days and produced `DeepAxis`;
#66 cost a prediction. If these counts are load-bearing on a design decision, an
unweighted histogram is the wrong summary. My impression, offered as an impression:
the **expensive** entries cluster in LITERATURE, SAME-ARTIFACT and "no artifact at
all"; the **cheap** ones cluster in CODE.

### 5.7 WHO HELD THE BELIEF

#45 was a premise the **user and assistant held jointly**; #13 and #48 were about
the user's own words and poses; #62's falsifier was the user; #33's false claim was
an *integrator instruction to an agent*. "Who authored the false belief, and who
could have refuted it" is orthogonal to both rules and predicts the control better
than either.

---

## 6. Confidence

### Would defend hard (24)

**#4, #5, #6, #20, #29, #33, #38, #42, #44, #45, #48, #49, #53, #54, #55, #56,
#57, #58, #64, #65, #66** — the falsifier's location is stated unambiguously by the
entry, and I verified the load-bearing ones independently (#38 against `68059b4`;
#58 against `journal/0109:99` vs `:115`; #66 against the samplers' loop bounds).

Plus three I verified against git and would defend *against the entries' own framing*:

- **#35 = STALE**, window = 1 day (`75445f2` → `f286893`), not 11.
- **#11 = STALE**, change event `c087d39`.
- **#60 = SAME-ARTIFACT**, falsifier at `journal/0111:119-129`.

And one negative claim I would defend hardest of all: **GENUINELY STALE is 2, at
most 3, of 67.** Every other stale-sounding entry has a documented no-true-era (§ 2).

### Coin-flips (the 29 in § 3)

I would not argue for my pick on any of them. The four most consequential:

- **#40** — left unresolved; the one row that could move the STALE count by 50 %.
- **#51** — two claims in one entry with different falsifier locations; splitting it changes both totals.
- **#59 / #60** — the BF-C/BF-U line. Mine is defensible, not unique; the alternative roughly doubles BF-U.
- **#18 / #19** — three-way rule-1 splits whose answer depends on whether *the author's own recent output* counts as a document at all.

### Not classified

**#40, rule 2 only** — "I cannot attribute this," per the brief, rather than forced.

### Methodological caveat about this whole exercise

The findings in § 4.1–4.5 came from reading whole entries and then asking git. Two
of them (#40's diff, #35's dates) *invert or materially change the entry's own
account*, and both sit in the small set of entries that bear directly on whether a
staleness watcher is the right instrument. **The self-reports are not uniformly
reliable about their own timelines** — which is the most important caveat I can
offer about coding this file at all, mine included.

---

## Correction candidates NOT filed

Per the brief, reported rather than filed (next free is #67):

1. **#56's unstruck "two independent instruments agree" paragraph**
   (`corrections.md:1949-1952`) directly contradicts #60 in the same file, with no
   strike and no pointer. A doc-topology defect *inside corrections.md*; it wants a
   one-line strike, not a new entry.
2. **#40's mechanism is very likely wrong** (§ 4.1). If the audit read a
   pre-`11d4385` checkout, #40's lesson points at the wrong control, and the A-2 it
   declares "never live" *was* live for three days. This is a genuine correction
   candidate, handed up rather than filed.
3. **#35's "eleven days"** and **#63's "no such knob anywhere in the tree"** are
   factual slips inside otherwise-sound entries; both want an inline fix, not an entry.
