# § 0 corpus sweep of `.claude/skills/passes/SKILL.md`

**Read-only sweep, 2026-08-05. Base commit `dd575b652deb828f9e724ff0381190fbb5658406`**
(worktree `agent-aa35cf3e4edf6dc44`; `git merge main` → *Already up to date*; local HEAD
identical to `main`). **No cargo was run.** Every quantity quoted is a pre-existing, dated
measurement cited to its own source.

**This audit DECIDES NOTHING AND NOTHING HERE IS APPLIED.** The skill's own § 0 requires a
user check before the skill is edited; the integrator adjudicates. Anything the sweeper
originated is marked **(sweeper's reading, unratified)**.

**D-2 was deliberately NOT re-verified** — a separate agent owns that trace, and
`docs/audits/2026-08-05-d2-deposition-trace-verification.md` was not opened. F10 touches
D-2's *citation*, not its trace, and is handed off.

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Anything that later
refutes or re-scopes this file gets a banner here, stamped by the author of the correction,
in the same commit.

---

## 0. Verdict, one sentence

**Not safe to rely on as-is; safe with the diff in § 4** — the skill is *substantially
right about the world* (D-9 confirmed at source, D-4 and D-1's quotes verbatim, D-8 and the
scope line clean) and *substantially wrong about the corpus*: it re-teaches a claim the
corpus formally superseded (F1), re-invents a two-week-old user ruling as a new assistant
formulation (F2), and says "never written down" / "nothing on the board describes it" /
"under-stated" about **four things the corpus states explicitly and in ratified form**
(F3, F4, F9, F12).

**The pattern is one pattern, and it is the project's named characteristic failure**
(`CLAUDE.md` read-first 0b): *re-inventing a mechanism next to the one it already built.*
The skill was written from an investigation of the **code**, and its corpus claims are the
part that did not get checked — which is exactly what a § 0 sweep is for. Its § 5's
in-code findings hold up well.

---

## 1. Coverage — stated honestly

**Read in full by the sweeper:** the skill · `docs/dependency-graph.md` (392 lines) ·
`docs/ARCHITECTURE.md:545-744` · `docs/design/north-star.md:220-275` ·
`docs/audits/2026-08-04-voxel-explainability-audit.md:1-60` ·
`docs/audits/2026-08-04-deposition-clock-design.md:1-55` · `docs/design/knowledge.md:1-45` ·
`docs/design/stubs.md` #25 and #52 in full · `ROADMAP.md:2376-2430` ·
`crates/dc-worldgen/src/deeptime/erosion/record.rs:60-185`.

**Grepped with targeted excerpt reads:** `ARCHITECTURE.md` DECIDED index · `north-star.md`
full-file · `stubs.md` heading index (all 54) · `crates/dc-worldgen/src/geology.rs` ·
`deeptime/{flux.rs,inventory.rs}` · `docs/audits/` listing (24 files).

**Covered by two delegated read-only agents, whose citations are reproduced here; the
sweeper independently spot-verified the ones a headline finding rests on** (marked ✔):
`ROADMAP.md` (5,494 lines) ✔`:2376-2430` · `journal/corrections.md` (4,097 lines) ·
`docs/design/{material-behavior,refinement,flow,worldgen,materials,geology,
earth-processes,pass-declaration-history}.md` · `docs/spines.md` · `journal/*.md` ·
`stubs.md` ✔`:896-902` ✔`:1978` · `geology.rs` ✔`:613-621` · `flux.rs` ✔`:27`.

**NOT OPENED — a null from these is not a result:**
`docs/audits/2026-08-05-d2-deposition-trace-verification.md` (excluded by brief) ·
`docs/spikes/` (S10, S20, the per-depth cost probe — cited second-hand only) ·
`docs/design/{ecology,ores,ideas,water,posture-gait,bodies}.md` · `docs/API.md` ·
`docs/rendering/PIPELINE.md` · `ROADMAP-history.md` · full bodies of the correlation,
deposition-clock, members-into-history and P2 audits (headers and cited lines only) · the
deep-time source tree beyond `record.rs`, `geology.rs`, `flux.rs`, `inventory.rs`.

---

## 2. Findings, ranked by cost-if-unfixed

Each cites **both sides**. **User-owned** means the disposal is the user's call.

---

### F1 — 🔴 § 3.1 RE-TEACHES THE CLAIM CORRECTIONS #65 EXISTS TO PREVENT, IN THE DOC A PASS AUTHOR READS FIRST

**Highest cost.** § 3 is *the procedure*. This teaches the model the corpus retired, and
teaches the workaround the retirement deletes.

| side | text |
|---|---|
| **skill** | `SKILL.md:113-114` — *"**Declare `{reads, writes}`.** The runner **derives order by topological sort; you never write the order.**"* |
| **corpus** | `ARCHITECTURE.md:545` § *The engine is plugin-agnostic, and pass ORDER is authored* — **DECIDED 2026-07-26 (user)**; `:572-575`: *"**Author-and-validate (DECIDED).** Order is **data on the world** — chosen per world, alongside seed and epoch count. `{reads, writes}` **stop being the ordering input and become the validator.**"* |

**Mirrored in four more places, all struck or superseded:** `material-behavior.md:384-399`
(*"🔴 SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD"*) · `north-star.md:226-229`
(*"~~The runner topo-sorts…~~ **SUPERSEDED**"*, and it records that **corrections #65 names
three sites**) · `spines.md:757-772` (S-6) · `journal/corrections.md:2611` (#65 itself).
**The skill is a fourth site.**

**And the workaround § 3.1 teaches is the artifact the ruling deletes.**
`ARCHITECTURE.md:579-580`: under author-and-validate *"the rejection that forces revision
tokens **disappears**."* `:584-586`: *"encoding it in engine-owned resource names is
precisely how `DeepAxis` came to exist. **The mechanism and the violation are the same
choice.**"* — so § 3.1's own ⚠ about `DeepAxis` (`:118-122`) is **downstream of the sentence
it opens with**, and the skill does not connect them.

**The corpus already models the right framing**, and the skill should copy it —
`spines.md:774-782`: *"⚠ **The mechanism IN THE TREE is still derive-and-reject** — this
section describes real code, and the code predates the decision… read the instances below as
**the best available compliance under derivation — not as the shape to copy into new
design**."*

Live on the board: `ROADMAP.md:1428` **🔴🔴🔴 THE PASS ARCHITECTURE — AUTHORED ORDER, OPEN
VOCABULARY, AND THE RATE AXIS BUILT**, *"(1) and (2) are still owed and this arc is still
open"* (`:1436-1438`).

**Disposal:** § 4.2. **Not user-owned** — applying an existing ruling.

---

### F2 — 🔴 THE ALTITUDE TEST IS ANTI-SHAPE **A-7**, A USER RULING FROM 2026-07-22, AND THE SKILL CREDITS IT AS A NEW ASSISTANT FORMULATION

**This is the duplication finding, and it is the project's named characteristic failure
landing on a read-first-adjacent doc.** A-7 lives in `spines.md`, which is **read-first item
0b** and against which *"work is justified"*.

| side | text |
|---|---|
| **skill** | `SKILL.md:129-135` — *"Ask the ALTITUDE question *(added 2026-08-05 from the weathering investigation; **assistant-formulated**, user-originated as a critique)*: is this pass formulated as the general process, or as one special case of it?… A pass that acts on one privileged slot, one hardcoded material, or one class of target will be re-implemented once per target later."* |
| **corpus** | `docs/spines.md:2279-2299` — **"A-7. Naming a content identity inside a process — DECIDED 2026-07-22 (user)":** *"'default is just the first content pack. So if you feel a need for naming a material directly: that's a want for a feature that makes the process more robust. **Naming directly will never, in any world, be correct.**'* … **This is not a ratifiable carve-out… it is a defect.** … **The check: when you feel the need to write a content name into a process, stop and ask what property of that content you are actually reaching for. That property is the feature the process lacks. Name it, and the special case dissolves for every future member of the same family.**"* |

**A-7 is strictly stronger than § 3.4 in two ways:**
1. It gives a **verdict**, not a question — such a case is a **defect**, not a deviation to
   weigh. § 3.4 says "ask"; A-7 says "this is already wrong."
2. It gives the **resolution procedure** — *name the property you were reaching for* —
   which is precisely what § 6 struggles toward for weathering and does not state.

**And § 5's findings are A-7 instances that the skill does not file as such:** D-1's
composition tier *"onto a single sentinel slot representing a hardcoded 50 m granite
basement"* and D-4's *"hardcoded product class"* are A-7 by definition. `spines.md:2315-2318`
lists the existing instances (`Litho::reference_material`, `classify::block_twin`); this
would add two more, and `CLAUDE.md` read-first 0b requires spines be updated **in the same
commit as work that adds an instance**.

Corroborating and independent: the same critique is **also** on the board in the user's own
words at `ROADMAP.md:2409-2411` (2026-07-25) — *"today the pass only weathers **bedrock**,
because the materialized seam is the only `Structure` in the inventory — **nature weathers
sediments, soils and transported clasts too** (that narrowing is stub #16's shadow, not a
physical claim)."*

**Disposal:** § 4.5. **Not user-owned** — but the *spines.md instance rows* are a corpus edit
the integrator owes.

---

### F3 — 🔴 D-0's "THE REASON WAS NEVER WRITTEN DOWN" IS FALSE, AND THE ENTRY IS LIVE IN § SEQUENCED

**Self-compounding cost:** D-0 is a durable instruction *not to look*.

| side | text |
|---|---|
| **skill** | `SKILL.md:167-171` — *"**D-0 — … (user-originated, lost from the record):** it wants to be a general process… **the reason was never written down** and was re-derived from scratch at the cost of most of one session."* |
| **corpus** | `ROADMAP.md:2376` (**§ Sequenced**; the section opens at `:381`… entry at `:2376`) — **"WEATHERING IS ONE PROCESS — SAPROLITE IS A STATE ALONG IT, NOT A SLICE"**, *"**USER'S STRONG LEANING on the destination, 2026-07-25**"*. The reason, in the user's own words at `:2393-2395`: *"**Saprolitification is a slice of a more general process of weathering** that ends in totally crumbling — **are we splitting that one process between multiple slice owners for any good reason?**"* And the WHAT at `:2389-2392`: *"Retire saprolite as a bespoke thing. Weathering becomes **one declared process** acting on whatever is exposed to reactants."* ✔ read in full by the sweeper. |

**Precision (A-5).** D-0 makes two claims; only one is falsified. *"The user declined the
flip for this reason"* — **not verified either way** by this sweep. *"The reason was never
written down"* — **falsified**. Whether the specific flip-declining *conversation* was
minuted is a different question and is not what the sentence says. **(Sweeper's reading,
unratified**, on that narrower question.)

**Disposal:** § 4.7. **Not user-owned.**

---

### F4 — 🔴 § 2 SAYS THE TWO-STORES RULE IS UNDER-STATED IN THE CORPUS; IT IS A RATIFIED USER DECISION FROM 2026-07-24, IN THE DOC THE SKILL POINTS TO

**The brief asked this question directly. The answer is REFUTED.**

| side | text |
|---|---|
| **skill** | `SKILL.md:81-82` — *"**This is the rule most likely to be got wrong, because only one store existed when the oldest passes were written**"*, and the skill's framing throughout § 2 presents the rule as newly formulated (the "USER FORMULATION, 2026-08-05" at `:97` is offered as *"the cleanest statement of the constraint we have"*). |
| **corpus** | `docs/design/material-behavior.md:76-85` — **§ "Commit semantics — DECIDED 2026-07-24 (user, ratified)":** *"**In-place transformation → append a FACT** to the existing unit. A unit becomes its depositional base (`DepTag`, immutable) **+ an appended list of transformation facts**; current composition = `derive(tag)` then fold the facts. · **Depositional arrival → append a new UNIT**… **Transport is a *removal fact* here + a *new unit* at the receiver.**"* |

That is the skill's § 2 rule in one ratified sentence pair — **plus** the transport case that
§ 2's user-formulation block re-derives (`SKILL.md:97-105`). Three more statements of it:
`material-behavior.md:58-66` (*"The strata record stays the temporal authority — append-only
history, the committed facts. The deep cell gains a mutable working inventory… Chapter
boundaries commit the working inventory's deltas back into the record"*) ·
`material-behavior.md:86-89` (*"Facts persist; the working inventory is transient"*) ·
`docs/spines.md:393-425` (**S-2, "Committed facts vs fluid state"**, with `FactLedger` as a
named instance).

**What is genuinely NOT in the corpus, and is the skill's real contribution:** the
**operational test** (*"did your process MOVE mass between places?"*) and the **diagnosis
that the height tier violates the rule**. Those are worth keeping and are new. The framing
that the corpus under-states the *rule* is not.

**One caveat the skill should carry, from `stubs.md:571-575` (#17's discharge):** *"the
**two-authorities split holds**… the pass READS `H` but WRITES ONLY the ledger… **The
'two-authorities split holds' line above states an implementation choice for this slice, not
yet an invariant.**"*

**Disposal:** § 4.4. **Not user-owned.**

---

### F5 — 🔴 D-1 COUNTS TWO WEATHERING AUTHORITIES; THE CORPUS ENUMERATES THREE, AND THE THIRD IS THE ONE THAT IS NOT PASS-SHAPED

**High cost:** D-1's aphorism is a two-body framing of a three-body problem, and a
generalisation designed against two will leave the third standing.

| side | text |
|---|---|
| **skill** | `SKILL.md:173-188` — *"**D-1 — there are TWO weathering systems**"*: the height tier + the composition tier. `emplace_weathering_front` appears only as D-4's **expression** stand-in (`:208-219`), never as an authority. |
| **corpus** | `ROADMAP.md:2395-2400` ✔ — *"Today it **is** split **three ways**: `dc:deep/weather_inventory` (declared, deep tier, produces a **scalar**) · `dc:deep/weather` (the scalar `R`/`H` height authority, still unreconciled) · `geology.rs::emplace_weathering_front` (**not declarative** — ordinary collapse code that **invents** the vertical distribution). **Only one of the three is authored in the shape everything is obligate to converge on.**"* |

Corroborated in code: `crates/dc-worldgen/src/geology.rs:613-621` ✔ carries D-4's quote
verbatim — **so the skill quotes the third authority's own source comment while not counting
it as an authority.**

**Disposal:** § 4.7. Keep the two-body aphorism as the sharpest *pair*; name the third.
**Not user-owned.**

---

### F6 — 🟠 § 3's PROCEDURE DECLARES TWO OF THE FOUR SCHEDULER AXES

| side | text |
|---|---|
| **skill** | `SKILL.md:113-128` — reads/writes, "run in the loop / **every epoch**", `dt`. No mention of WINDOW or `Schedule` anywhere in the file. |
| **corpus** | `material-behavior.md:373-379` — *"The scheduler has **four orthogonal axes**, and the runner declares **all four** per pass"*: **ORDER · RATE · WINDOW · SCHEDULE.** |

- **SCHEDULE** — `ARCHITECTURE.md:671` **DECIDED 2026-07-29 (user)**, built journal/0124:
  *"A sum type, not a flag beside a number: `Schedule::Seed` (pre-loop once, never in-loop) ·
  `Step(Cadence)` · `SeedAndStep(Cadence)`"* (`:712-713`). **So § 3.2's "every epoch" is
  literally wrong** for a `Step(Cadence)` with period > 1. Three rules the skill omits:
  *"**Epoch 0 fires for everyone; the skip rule is DELETED**"* (`:717-719`); the sharp
  discriminator *"**a seed is an initial condition only if something OBSERVES it before the
  pass itself first steps**"* (`:681-682`); and the gate invariant *"every pass's integrated
  `dt` over a run equals the world's elapsed time"* (`:720-722`). Also
  `material-behavior.md:431-438`: the runner enforces it *"by handing a seeding body
  `dt = 0.0`"*. And **`Seed`/`SeedAndStep` have no production declarer** (`ARCHITECTURE.md:688-691`,
  `spines.md` § 3) — the next author to reach for one is the first.
- **WINDOW** — `flow.md:730-760`, the ratification: *"`material-behavior.md` § 5 gives the
  scheduler ~~ORDER (topo-sort)~~ and **RATE**… It has no name for 'how many epochs sum into
  one record entry'… **A window that decides an acceptance number must be declared, not
  assumed.**"*

**Disposal:** § 4.3. **Not user-owned.**

---

### F7 — 🟠 § 6 SAYS "COST UNPRICED"; THE CORPUS PRICED IT, AND HOLDS THE CONSTRAINT THAT MAKES A NAIVE BUILD AN **A-1 WITH A DISGUISE**

| side | text |
|---|---|
| **skill** | `SKILL.md:290-291` — *"the multiple causes… **may** want splitting into separate passes — which runs into `DeepAxis` (§ 3.1). **Cost unpriced.**"* |
| **corpus** | `ROADMAP.md:2387-2389` ✔ — *"The numbers that frame the call: **gen time is affordable (25.7 s → 46.4 s); residency is not (`LedgerField` 17.45 MiB → 973 MiB, 55.8×, resident).** The S20 spike costed four options for that decision, including paged facts."* |

**And the split decision is not open either** — `material-behavior.md:349-371`: *"**Should
distinct transitions be separate passes or one monolith? Separate passes**, decided by
**coupling timescale vs cadence**: coupling coarser than cadence → separate passes… finer →
**fuse that pair** (a *measured* exception)… **The moddability tiebreaker:** a mod adds a
process by *declaring* it, not by patching a fold it cannot see."* § 6 treats as open a
question the corpus answers with a decision procedure.

**The requisite § 6 omits entirely** — `ROADMAP.md:2417-2421` ✔:
> *"**⚠ REQUISITES — the honest reason this is not scheduled yet.** The exponential emerges
> from **reactant transport**, and we have **no vertical fluid flux**. A depth-resolved
> weathering rate would have nothing honest to read, so 'emergent' gradation would emerge
> from a **fabricated depth term** — the same shape function with better camouflage
> (**A-1** with a disguise)."*

R1/R2 **MET** (`:2422-2426`, journal/0098; 307,364 vertical entries across 44.3 % of cells)
**with a caveat that survives into the arc**: that flux is **recharge-free** (`stubs.md` #19),
*"relatively shaped, absolutely uncalibrated."* R3 **MET** (`:2427+`, journal/0106,
`examples/perdepth_weathering_cost_probe.rs`) *"and it moved the blocker."*

Two more items from the same entry that § 5/§ 6 should carry:
- `:2408-2410` — *"the **7/8 cap is a definition wearing physics' clothes** — weathering does
  not halt at 7/8 retained fabric, so the cap makes complete weathering (laterite, oxisol,
  total crumbling) **unrepresentable**."* This **sharpens D-4**, which treats the 1/8→7/8
  grading as a stand-in for the *product class* but never says the **cap itself** is a
  modelling error.
- `:2406-2407` — modelling the cause makes `WEATHERING_PROFILE` and **stubs #20 DELETED, not
  tuned** — *"the disposal the stub doctrine wants."*

**Disposal:** § 4.8. Transcription is not user-owned; **the arc's sequencing remains
user-owned.**

---

### F8 — 🟠 THE REFINEMENT FIREWALL HAS FOUR RATIFIED PRIORS, A MEASURED VERDICT, AND TWO RECORDED VIOLATIONS — D-5 SAYS "NOBODY HAS CHECKED"

**Cost: moderate-to-high, and one part is USER-OWNED.**

| side | text |
|---|---|
| **skill** | `SKILL.md:221-223` — *"**D-5 — the refinement firewall is believed, not proven.** § 1's *by construction* clause needs verifying… Expected answer: no. **Nobody has checked.**"* And § 1 presents the 2026-08-05 ruling as sharpening only two existing rules. |
| **corpus — four priors** | **(1)** `refinement.md:74-76` = `flow.md:513-517`: *"**The record is the only seam.** Deeptime writes; refinement reads. If refinement needs something, **deeptime must have recorded it.**"* **(2)** `refinement.md:70-73` / `flow.md:508-511`: refinement is a *"**pure fn of (record, shared face data, position)**"* — **a pure fn structurally cannot write to the record, which is most of the "by construction" the ruling asks for, already enforced by type.** **(3)** Law 3, `refinement.md:218-223`: *"refinement **redistributes; it never creates or destroys**… over-expression goes negative and fails loudly."* **(4)** Law 1, `refinement.md:198-207`: *"An operator's expression varies **only** with recorded continuous quantities… An operator that renders the family instead of the record is `carve_rivers` with extra steps."* |

**A prior USER-ORIGINATED statement of the rule exists and is in neither § 0's grep list nor
§ 7** — `docs/design/knowledge.md` (*"seeded 2026-07-19… (ratified)"*), requirement 3 at
`:23-27`: *"**Computable anomaly.** Every natural feature carries process provenance… **'What
explains this?' is a real query; anomaly = no valid provenance match.** Freakiness is
physics, never a quest flag."*

**And it has been MEASURED, one day before the skill was written** —
`docs/audits/2026-08-04-voxel-explainability-audit.md:20-22`: *"**The sketch is roughly
one-quarter true, and the true quarter is narrower than it looks.**"* Three named walls, and
`:53-57`: *"the largest single hole is not in the record at all: **all sub-460 m relief in
the shipped world is midpoint-displacement noise**… whether a given voxel is rock or air,
below the deep grid's 460 m pitch, is answered by an addressed hash — honest 'statistics,
deterministically', **never by a modelled process**."*

**Two recorded prior VIOLATIONS:** `refinement.md:63-66` — *"`collapse.rs::carve_rivers`…
**It *draws* a channel instead of *spending* a budget**… It predates the doctrine; existence
is not standing."* · `stubs.md:652-673` (#20) — the front's *magnitude* is the ledger's but
*"the decay length, the 7/8 cap and the resulting 2.67× thickness ratio are **not measured
from anything**"*, which is the skill's arrangement-vs-substance corollary already worked and
stubbed. **D-4 discusses the same code site without citing #20.**

**⚠ THE RECONCILIATION THE SKILL OWES AND DOES NOT ATTEMPT — and it is USER-OWNED**, because
a strict reading of the 2026-08-05 ruling outlaws the shipped world's entire sub-cell relief:

| | |
|---|---|
| the ruling | *"…creating **factless states** (by construction should be impossible)"* (`SKILL.md:63-64`); *"every visible thing traces to a recorded fact"* (`:67`) |
| the ratified method item | `earth-processes.md:21-29`, **method item 5, RATIFIED 2026-07-19**: *"simulation stops at some resolution everywhere, and **no simulation-resolution edge may reach the eye as a square or analytic boundary** — '*if i see a square boundary I'll scream*.' **Every grid-scale contact… must be dressed by noise/dither/blend/sampling tricks before it is visible.**"* |
| the shipped mechanism | `refinement.md:277-281` — K3's midpoint jitter, filed as **move C, bounded stochastic detail synthesis**, *"**not expressible in this document's two-move vocabulary**… ***All* sub-460 m relief in the shipped world comes from move C"* |

They reconcile **only** through the skill's own *arrangement, never substance* corollary
(`SKILL.md:73-77`). The skill states the corollary and never applies it to the one case that
tests it. **(Sweeper's reading, unratified. The call is the user's.)**

**Disposal:** § 4.6 + § 4.9. **The reconciliation is USER-OWNED.**

---

### F9 — 🟠 D-6 IS "UNVERIFIED, NOTHING ON THE BOARD DESCRIBES IT"; IT IS A FILED STUB WITH A NAMED HEIR, FROM A SHIPPED SLICE

| side | text |
|---|---|
| **skill** | `SKILL.md:225-228` (D-6), under § 5's banner *"UNVERIFIED unless marked… recorded because nothing on the board describes it"* (`:164-166`). |
| **corpus** | `docs/design/stubs.md:1978` ✔ — **`### 52. an-epoch-that-alteration-cannot-move` — *added 2026-08-04 (the deposition-clock slice, journal/0154; F4 of `2026-08-04-deposition-clock-design.md`)***, with heir *"an **alteration-time axis**"* and a stated blast radius. |

D-6's arithmetic is **verified** by the sweeper: `deeptime/flux.rs:27` ✔ — *"is **25 epochs**
(200 iterations / 8 chapters)"*. So the number is right; the **status marker** is wrong.

Two sharpenings stubs #52 carries that D-6 does not: `set_tag_and_chapter` **rewrites chapter
in place**, so after an overprint *"a unit's chapter and its epoch can disagree
(`chapter != floor(epoch / chapter_length)`)"* — stronger than "coarser"; and *"**the
correlation partition uses epoch, not chapter**, so it is unaffected"* — which **narrows**
§ 2's claim (`SKILL.md:90`) that top-append monotonicity is load-bearing *for the correlation
join* via the deposition clock.

**Disposal:** § 4.7. **Not user-owned.**

---

### F10 — 🟠 D-2 CITES A STUB DISCHARGED THREE DAYS EARLIER, AND THE DISCHARGE BEARS ON D-2's BOLDED CONCLUSION

**Handed to the D-2 verification agent. This finding is about a CITATION — corpus-side, and
inside this sweep's remit. The trace was not re-run.**

| side | text |
|---|---|
| **skill** | `SKILL.md:198-200` — *"**no durable fact says 'made here, not brought here.'** … Related: `stubs.md` #25 (**the record carries no mover axis**…)"* |
| **corpus** | `docs/design/stubs.md:896` ✔ — *"`### 25.` … **✅ DISCHARGED 2026-08-02/03 (P11 slice 3, the packed `DepUnit`)**"*; banner `:897-902` ✔: *"**The mover axis is REAL: every recorded unit carries 3 mover bits (`FlowCause as u8`, `MOVER_NONE` = made in place)**, written by every production depositor… pedogenesis/diagenesis write NONE. **It is IN the merge key (M-1)**… **Colluvium and alluvium are now distinguishable by label, as separate units.**"* Same at `dependency-graph.md:217`. |

**Stated as a question for D-2's owner, NOT as a conclusion (A-5):** `MOVER_NONE = made in
place` reads like exactly the durable fact D-2 says does not exist. The relevant shape is
`crates/dc-worldgen/src/deeptime/erosion/record.rs:161-179` ✔, where the incumbent
(`best_m = dh - carried`) leaves `best: Option<usize> = None` and `best.map(…)` returns
`None` for an incumbent win. **Whether that `None` reaches the unit as `MOVER_NONE` was not
traced here.**

D-2's other half is unaffected either way: *"where arrivals outweigh it, the weathered
material is absorbed into a unit labelled with a mover's species"* — that is **D-9's
discard**, confirmed at source (§ 3).

**Disposal:** at minimum correct the pointer. The conclusion is the D-2 agent's finding.

---

### F11 — 🟡 § 3.1's PIN-THE-PAIR RULE IS MISATTRIBUTED, AND ITS ACTUAL HOME IS A 203-LINE DOC THE SKILL NEVER CITES

| side | text |
|---|---|
| **skill** | `SKILL.md:115-117` — *"pin the other with an anti-dependency on the **next** revision of that plane, never the last (`session-workflow`, **the audit that was half wrong about this**)."* |
| **corpus** | `docs/design/pass-declaration-history.md:158-167` — *"The 2026-07-25 audit's own prescription, 'declaring `Forced` is free and it pins it', was **half wrong**… **The fix is the pair**… declare the revision you consume as a `reads`, and the **next** revision of the same plane as a `reads_prev`. **The next, never the last.**"* Plus `:91-111` (`reads_prev` as WAR anti-dependency + the hostile-rename guard), `:113-124` (the `BioMod` fiction), `:126-192` (revision tokens, the floating sidecar). Spine form: `spines.md` § S-6. |

**This is the existing archaeology of authoring a pass declaration** — the single most
directly relevant document to § 3, and it is in neither § 0's grep list nor § 7.

**Disposal:** § 4.9. **Not user-owned.**

---

### F12 — 🟡 § 6 SAYS THE END STATE "HAS SIMPLY NEVER BEEN OWNED"; IT IS OWNED, WITH A CODE-LEVEL DIAGNOSIS AND A VERIFIED-STILL-OPEN ROADMAP ROW

| side | text |
|---|---|
| **skill** | `SKILL.md:284-288` — *"…which is already the corpus's stated end state (`material-behavior.md` § 11's continuation slot) and **has simply never been owned**."* |
| **corpus** | `stubs.md:576-585` (#17's **residual**) — *"`DeepField::derive_regolith_at` derives `H` from an **empty** ledger (`FactLedger::empty_with_bedrock`)… **do not read the discharge as 'the two authorities are reconciled.'**"* ✔ (the quote the skill uses for D-1 is from this same residual). `spines.md:34` (S-9 audit, 2026-07-29): *"**The residual itself is STILL LIVE and unchanged**: `field.rs:904` still builds the view from `FactLedger::empty_with_bedrock`."* `ROADMAP.md:4156-4171` (§ Observed, **✅ VERIFIED STILL OPEN 2026-07-29**): *"the derived `H` **structurally cannot see** the **6.09 m of `Loose`** the M3 weathering process commits per epoch… it is **the honest reason 'one authority' is not yet true**"*, at `field.rs:822`. |

So § 6's target is not unowned — **it is half-built and openly broken, with the breakage
located to a line.** That is a materially better starting point than "never owned", and it is
the thing a generalisation slice would start from.

**Disposal:** § 4.8. **Not user-owned.**

---

### F13 — 🟡 § 3.6's RATE GUIDANCE IS ONE LINE; THE CORPUS HAS A NAMED "ONLY LEGAL METHOD", AN ACCEPTANCE INSTRUMENT, AND THREE SHARPER RULES

| side | text |
|---|---|
| **skill** | `SKILL.md:137-138` — *"**Cite the rate's literature** (§ 1). The soil-production taper… is the in-tree example done right."* |
| **corpus** | `ROADMAP.md:1658-1663` — *"**How to pick it — the ONLY legal method.** Against the **published literature**, not against a look… **Do not re-fit to internal consistency — every internal instrument was green at 1000 × wrong.**"* |

And:
- **The acceptance instrument exists** — `earth-processes.md:380-386`:
  *"`examples/denudation_probe.rs` is its acceptance instrument (its literature band table
  **is** the test; the world should land in the 1–10 m/Myr stable-craton band). **The
  resulting numbers are appearance-class and user-owned.**"*
- **`corrections.md:2259` (#59):** *"**A derivation is only an anchor if the thing it is
  derived from cannot move**… 'the constant was checked against a number that was itself
  unchecked.'"*
- **`corrections.md:2028` (#56):** the Phanerozoic register the deep-time rates are calibrated
  to, naming which constants a recalibration scales (`weathering`, `k_transport`, `k_bedrock`)
  **and not `diffusion`**.
- **`corrections.md:3945` (#98)**, the user-rejected pits bar, whose **replacement is a general
  acceptance rule**: *"**a closed depression is legitimate iff a process owns it; an unowned
  one is a defect at any depth.**"*
- **`corrections.md:1980` (#55):** *"Fluvial transport is 0.109 % of the sediment routing.
  **Hillslope creep moves 918× more**"* — rule: *"before believing a mechanism matters,
  **measure how much authority it has** over the thing you are claiming it changes."*
- **Ratified magnitudes a rate change must not silently move** — `earth-processes.md:436-458`
  (wind/frost RATIFIED 2026-07-21; **wave NOT ratified and its retune struck by the user as a
  bandaid**) and `ROADMAP.md:1665-1672` (*"I do not care if frost/wind magnitudes increase
  45×… Byte-identicality on the calibrated arm is explicitly NOT wanted"*).
- **`[[no-bandaid-tuning]]` is a grep-able tag** (`earth-processes.md:456`, `stubs.md:628`,
  `water.md:511`). The skill states the rule without the tag, so a grep for the tag misses
  the skill.
- ⚠ **And one flag against the skill's own exemplar:** the delegated sweep found **no
  document citing the soil-production taper's literature source** — it is asserted in code
  only. Calling it *"the in-tree example done right"* may be correct but is **currently
  unevidenced in the corpus**. *(Sweeper's reading, unratified; not independently
  re-verified.)*

**Disposal:** § 4.10. **Not user-owned.**

---

### F14 — 🟡 `earth-processes.md`'s FIVE RATIFIED METHOD ITEMS AND ITS STANDING QUESTION ARE ABSENT

The brief asked what the method list says. It is **five items, RATIFIED 2026-07-19**, at
`earth-processes.md:9-29`, under the doctrine at `:3-7`:

| # | line | gist |
|---|---|---|
| 1 | `:10-12` | **Field notebook first** — feature → engines → sequence → what the record looks like. **Before any code.** |
| 2 | `:13-14` | **Mechanism fidelity over resolution fidelity** — coarsest state + process set that still generates the causal chain honestly. |
| 3 | `:15-17` | **The record is the world** — *"present-day fields may only drive present-day processes."* |
| 4 | `:18-20` | **Validation is a geologist's-eye walk** — *"a cut face must read; 'how did this get here?' must have a true, discoverable answer."* |
| 5 | `:21-29` | **The unsimulated remainder → procedural tricks** — no resolution edge may reach the eye as a square; every grid-scale contact must be dressed. |

**Item 3 is the prior for the skill's PROCESS-NOT-SNAPSHOT** and predates it by five days.
**Item 4 supports § 4** (a walk validates that the model *reads truly*, not that it is liked)
— worth saying, because § 4 currently reads as a new constraint on a practice rather than as
the practice's own ratified purpose. **Item 5 is F8's reconciliation target.**

**And the standing gate on every new pass is missing from § 3** —
`earth-processes.md:416-420`:
> *"## Standing question for every future pass — **'What does this look like in reality, and
> how did it get there?'** — if the pass can't answer in those terms, **it isn't ready to be
> declared in the graph**."*

⚠ **The skill's § 7 cites `earth-processes.md` as "(method items)" with no numbers**, and the
brief flagged that item 5 is cited nowhere. Note the ambiguity that creates: the doc's *engine*
list (`:31-258`, eight engines, explicitly *"working notes — sketches, NOT decisions"* at
`:31`) has an item 5 too (*Burial, diagenesis, metamorphism*). **A bare "item 5" is ambiguous
between a ratified method item and an explicitly non-decision sketch** — the citation must
name which list.

**Disposal:** § 4.3 + § 4.9. **Not user-owned.**

---

### F15 — 🟡 D-9's "NOT FOUND: ANY REASONING ABOUT THE DISCARD" — the discard, yes; the argmax, no

**Stated carefully, because the skill is mostly right.**

| side | text |
|---|---|
| **skill** | `SKILL.md:276-278` — *"**Not found:** any reasoning about the discard itself — only about how to pick the winner. Whether the argmax was ever a decision… is open."* |
| **corpus** | `stubs.md:899-903`, `:914-922` — the argmax is argued explicitly (*"**deliberately**: ranking the movers against each other would make the answer depend on which agent was asked first"*; *"**Why it is a stub and not an omission**… +42 MiB… a 25 % residency increase to add one axis"*) · `docs/audits/2026-08-01-members-into-history-design.md:310` — *"**Option B** — `DepUnit` carries a **share vector** instead of an argmax"*, costed at `:471` · `docs/audits/2026-07-29-fluvial-record-terms-priors.md:116,182` — *"**argmax'd to one `Litho`**… `stubs.md` #25 is **the record of that deliberate collapse**"* · `corrections.md:2153` (**#57**) — the worked failure case: *"Thin fire beds crept downslope, **won the argmax at a low-deposition cell**, and then merged across epochs under one mineral tag into a stratum the cap exists to forbid."* |

**The skill's substantive point survives** — the *composition* discard is distinct from the
*mover* discard, and **Option B was costed and not chosen rather than reasoned about as a
discard**. But *"whether the argmax was ever a decision"* is **answered: it was**, and the
skill's strongest available argument for D-9's remedy (#57's wrong world) goes unused.

**Disposal:** § 4.7. **Not user-owned.**

---

### F16 — 🟢 THE `weather_inventory` ON/OFF COLLISION HAS NO SINGLE HOME, AND THE SKILL IS THE RIGHT ONE

`ROADMAP.md:410` says the height-tier weathering is **default ON**; `ROADMAP.md:3817` says the
shipped world runs with **`weather_inventory` off**. Both true — D-1's two systems — but no
document states the disambiguation in one place, and a skimming reader will collide them.
D-1 already draws the distinction (`SKILL.md:176`, `:185`); one clause closes it.

**Disposal:** § 4.7. **Not user-owned.**

---

## 3. Nulls — suspicions checked, found clean

- **(c) THE SCOPE LINE IS RIGHT — the brief's biggest single risk, and it passes.**
  `SKILL.md:14-19` disclaims the pass runner, the ordering/validation machinery, the
  field-solver kernels, `DeepAxis` and the record's storage layout as engine. **Every
  disclaimed item is a `dependency-graph.md` § 1 E-row:** E1 `:185`, E2 `:186`, E4 `:188`,
  E6 `:190`, E7 `:191`. Against `north-star.md:254-258`: *"Core is only: cell storage +
  cell/space API · the pass-runner · the event ledger · the field-solver primitives · the
  data-model APIs · the stable SDK surface. **Every pass is content** — including tectonics
  and erosion."* Against the corrections #71 guard (`dependency-graph.md:29-34`): **the skill
  contains no sentence classifying erosion, weathering or tectonics as engine work**, and its
  § 3.1 ⚠ correctly places `DeepAxis`/E6 upstream rather than claiming the pass work is
  engine. **The scope line passes.**
  - One clause worth adding anyway *(sweeper's reading, unratified)*: the north star lists
    **the event ledger** as core (`:255`) while § 2 treats the fact ledger as a store passes
    write to. Compatible — the *organ* is engine, the *facts* are content — and P11 makes the
    same split explicitly for the record (`dependency-graph.md:217`: *"the **record** is E2
    storage, the **identities** are pack content"*). Saying it once prevents the next reader
    re-deriving it.
- **(d) DUPLICATION — the § 5 inventory is CLEAN and deliberately single-homed; § 3.4 is NOT
  (that is F2).** `ROADMAP.md:396-399` names the skill as the single home: *"Full inventory
  with provenance and confidence markers: **`.claude/skills/passes/SKILL.md` § 5** (drafted
  2026-08-05 at the user's direction; its own § 0 sweep is OWED and unrun)."* The ROADMAP
  entry carries the *arc*, the skill carries the *inventory* — one authority with a pointer.
  No incumbent doc should own §§ 1–2 instead: `material-behavior.md` owns the transformation
  *model*, `refinement.md` the refinement *laws*, `earth-processes.md` the process
  *catalogue*, `pass-declaration-history.md` the declaration *archaeology* — **none is a
  pass-authoring procedure, and the skill is a legitimate new home for one.** The defect is
  that it restates four of them without citing them (F1, F2, F4, F14).
- **D-9's mechanism: CONFIRMED AT SOURCE by this sweep.**
  `crates/dc-worldgen/src/deeptime/erosion/record.rs:74` `record_cell` → `:85-86`
  `s.deposit_moved(tag, dh, chapter, epoch, species, mover)` deposits the **full net `dh`**
  under a **single** species; `:160-169` is the argmax over the complete mixture array; the
  doc comment at `:93` names it *"the argmax of the arriving mixture"*. **D-9 is correct as
  written.**
- **D-4's in-tree quote: VERBATIM.** `crates/dc-worldgen/src/geology.rs:617-619` reads exactly
  as quoted; the 1/8→7/8 grading confirmed at `:67`. The *"documented in-tree, and the code
  says so"* marker is honest.
- **D-1's stubs #17 quote: VERBATIM** — `docs/design/stubs.md:570`.
- **D-6's arithmetic: CONFIRMED** — `deeptime/flux.rs:27`. (Only its *status* is wrong — F9.)
- **D-8: correct, including its honesty.** Matches `corrections.md:3977` (#99) and its
  *"not an owed fix"* (`:4000-4003`). **No change needed.**
- **§ 3.3's `dt` claim: accurate and current.** Matches `material-behavior.md:403-416` and
  `dependency-graph.md:360-365` — same four unconverted sites, same modelling caveat
  (`1 − exp(−k·dt)`, not `k·dt`). Two riders worth adding but the claim is clean: RATE
  *"does NOT own stability substepping"* (user, 2026-07-29 — a field solver subdivides `dt`
  internally inside its own von Neumann bound; `stubs.md` #30), and *"picking one silently
  re-tunes a constant **P2 is about to re-pick anyway** — do these WITH P2."*
- **§ 3.1's `DeepAxis` ⚠: accurate.** 18 variants, not retired, E6 unblocked and sequenced —
  `dependency-graph.md:190`, `ROADMAP.md:452`. Only its framing is wrong (F1).
- **§ 4 (a walk verdict is not a model verdict): no contradiction anywhere.** Consistent with
  the walk-loop doctrine (the user's live view is senior to a screenshot read — about
  *appearance*), and **positively supported** by `earth-processes.md` method item 4 (F14).
- **§ 1's CONSERVATION bullet: consistent** with `refinement.md:218-223` Law 3 and the
  drawdown ledger.
- **§ 7's code pointers: all resolve.** `runner.rs`, `erosion/weathering.rs`,
  `weather_inventory.rs`, `inventory.rs`, `recorder.rs`, `erosion/record.rs`, `flux.rs`,
  `geology.rs::emplace_weathering_front` — every one exists and holds what the skill says.

---

## 4. Proposed diff for the skill — NOT APPLIED

The skill's § 0 requires a user check. Ordered by § of the skill.

### 4.1 § 0 — the watermark

Replace `SKILL.md:36-38`:

> 3. Record the watermark here when you update: **last swept 2026-08-05 at base
>    `dd575b652deb828f9e724ff0381190fbb5658406` —
>    `docs/audits/2026-08-05-passes-skill-sweep.md`, 16 findings; F1–F5 were corpus
>    contradictions and are folded in below. D-2's trace is verified separately
>    (`docs/audits/2026-08-05-d2-deposition-trace-verification.md`). NOT swept: `docs/spikes/`,
>    `docs/API.md`, `ROADMAP-history.md`, the deep-time source tree beyond `record.rs` /
>    `geology.rs` / `flux.rs` / `inventory.rs`.**

Extend § 0.1's grep list with: `docs/design/pass-declaration-history.md` ·
`docs/design/knowledge.md` (req 3) · `docs/spines.md` **A-7** and **S-2/S-6** ·
`docs/audits/2026-08-0{3,4}-*` (correlation, deposition clock, explainability) ·
`ROADMAP.md:2376` (WEATHERING IS ONE PROCESS) · `ROADMAP.md:383` (THE HONEST RECORD).

### 4.2 § 3.1 — the headline correction (F1)

Replace the opening sentence:

> 1. **Declare `{reads, writes}` — and know what they are FOR, because it is DECIDED and it
>    has changed.** Today's runner *derives* order from them by topological sort (E1), but
>    **ORDER IS AUTHORED, PER WORLD — DECIDED 2026-07-26 (user)** (`ARCHITECTURE.md` § *The
>    engine is plugin-agnostic, and pass ORDER is authored*), and under it `{reads, writes}`
>    **stop being the ordering input and become the validator.** *"Order is derived from the
>    declarations"* is **falsified — corrections #65**, which names three sites; **do not write
>    a fourth.** E7 (authored order + validator) is unblocked and sequenced.
>    - Read today's mechanism the way `spines.md:774-782` reads it: *"the best available
>      compliance under derivation — **not** the shape to copy into new design."*
>    - **So the revision-token pairing below is a derive-and-reject ARTIFACT scheduled for
>      deletion** — *"the rejection that forces revision tokens disappears"*
>      (`ARCHITECTURE.md:579-580`). Use it today; do not design around it, and **never add a
>      `DeepAxis` variant to buy an ordering**: *"encoding it in engine-owned resource names is
>      precisely how `DeepAxis` came to exist. The mechanism and the violation are the same
>      choice"* (`:584-586`).

Keep the anti-dependency paragraph and the `DeepAxis` ⚠, **re-attributed** to
`pass-declaration-history.md:158-167` + `spines.md` § S-6 (F11).

### 4.3 § 3 — the missing axes, shapes and standing question (F6, F14)

Insert a new **§ 3.0**:

> 0. **Know which SHAPE you are writing** (`north-star.md` § Passes, ratified 2026-07-23;
>    `material-behavior.md:309-322`; **both are content**). **Cellular** — per-cell
>    select-and-transform, *"select materials matching P in context C, apply T at rate R"*;
>    **runs EDGES**, output is changed material state; the shape the corpus names for
>    weathering, diagenesis, decay, cementation. **Field** — declares reads/writes over
>    fields, body is arbitrary computation over **core solver primitives** (E4); **computes
>    fields, never runs edges, never touches the form inventory.** *For the cell world there is
>    no third.* And **`ctx` is a capability, not a god-object** — a pass may touch only what it
>    declared; that discipline is what keeps "it's all native Rust" from rotting back into
>    bespoke passes.

Replace § 3.2 (*"Run in the loop. Every epoch…"*):

> 2. **Declare all FOUR scheduler axes** — the runner declares ORDER · RATE · WINDOW ·
>    SCHEDULE per pass (`material-behavior.md:373-379`), and § 3 used to name two.
>    - **SCHEDULE** (`ARCHITECTURE.md` § *Schedule*, **DECIDED 2026-07-29 user**, built
>      journal/0124): `Seed` (pre-loop once, integrates **zero time**, handed `dt = 0.0`) ·
>      `Step(Cadence)` · `SeedAndStep`. **Epoch 0 fires for everyone; there is no skip rule.**
>      The discriminator, and it is sharp: *a seed is an initial condition **only if something
>      OBSERVES it before the pass itself first steps**.* All three pre-loop incumbents audited
>      to `Step`, so **`Seed`/`SeedAndStep` have no production declarer** (`spines.md` § 3) —
>      reach for one and you are the first, and you owe the argument.
>    - **WINDOW** (`flow.md:730-760`): how many epochs sum into one record entry. **A window
>      that decides an acceptance number must be declared, not assumed.**
>    - **The invariant, and it is a gate test, not a narration:** every pass's integrated `dt`
>      over a run equals the world's elapsed time.

Append to § 3 as a final gate:

> 10. **Answer the standing question** (`earth-processes.md:416-420`): ***"What does this look
>     like in reality, and how did it get there?"* — if the pass cannot answer in those terms,
>     it is not ready to be declared in the graph.** Upstream of it, method items 1–5
>     (`earth-processes.md:9-29`, RATIFIED 2026-07-19): **field notebook first, before any
>     code** · mechanism fidelity over resolution fidelity · **the record is the world**
>     (present-day fields may only drive present-day processes) · validation is a geologist's-
>     eye walk · the unsimulated remainder is dressed, never left as a square edge.

### 4.4 § 2 — the "under-stated" framing (F4)

Replace `SKILL.md:81-82` with a citation:

> **The rule is RATIFIED, not new — `material-behavior.md:76-85`, § *Commit semantics*,
> DECIDED 2026-07-24 (user):** *"In-place transformation → **append a FACT** to the existing
> unit… Depositional arrival → **append a new UNIT**… **Transport is a *removal fact* here +
> a *new unit* at the receiver.**"* Also `material-behavior.md:58-66`, `:86-89`, and
> `spines.md` § S-2 (*committed facts vs fluid state*). **What is NOT in the corpus, and is why
> this section exists: the operational test below, and the finding that the height tier
> violates the rule.** And the caveat from `stubs.md:571-575`: the two-authorities split as
> shipped is *"an implementation choice for this slice, **not yet an invariant**."*

Add to the merge row of § 2's table: **merge keeps the bottom epoch, and the record stores the
RAW TICK, never a derived time** — `docs/audits/2026-08-04-deposition-clock-design.md:47-52`,
**RULED 2026-08-04 (user)**. Correct § 2's *"the stratigraphic-correlation join depends on"*
to name the field: **the correlation partition keys on EPOCH, not chapter** (`stubs.md` #52).

### 4.5 § 3.4 — the altitude test is A-7 (F2)

Replace the parenthetical and strengthen the verdict:

> 4. **Ask the ALTITUDE question — and it is anti-shape A-7, DECIDED 2026-07-22 (user), not a
>    new formulation** (`spines.md:2279-2299`): *"'default is just the first content pack…
>    **Naming directly will never, in any world, be correct.**' **This is not a ratifiable
>    carve-out… it is a defect.**"* **So a pass that names a content identity, acts on one
>    privileged slot, or hardcodes a target is already wrong — the question is not whether, but
>    what.** A-7's resolution procedure: *"stop and ask **what property of that content you are
>    actually reaching for. That property is the feature the process lacks.** Name it, and the
>    special case dissolves for every future member of the same family."* The user's own
>    restatement for weathering: *"The forces that cause weathering do not magically only impact
>    basement rock"* (2026-08-05), and on the board since 2026-07-25 (`ROADMAP.md:2409-2411`).
>    **It is the same category error as running a process once** — *"Nature has no saprolite
>    **process**; modelling a stage as a mechanism is the same category error as S18's one-shot"*
>    (`ROADMAP.md:2401-2404`).

**And file the instances:** D-1's hardcoded granite sentinel and D-4's hardcoded product class
are A-7 instances; `spines.md:2315-2318` owes two new rows **in the same commit** (read-first
0b).

### 4.6 § 1 — the no-factless-states priors (F8)

After the user ruling, add:

> **This is not the rule's first statement.** `docs/design/knowledge.md` req 3 (*seeded
> 2026-07-19, ratified*): *"**Computable anomaly.** Every natural feature carries process
> provenance… 'What explains this?' is a real query; **anomaly = no valid provenance match.**"*
> And four ratified priors already constrain refinement: **the record is the only seam**
> (`refinement.md:74-76`), **refinement is a pure fn of (record, face data, position)** — which
> **already forbids the write path by type**, and is most of the "by construction" the ruling
> asks for (`:70-73`) — **Law 3 conservation** (`:218-223`), **Law 1 anti-carve** (`:198-207`).
> **Recorded prior violations:** `carve_rivers` (`refinement.md:63-66`) and the weathering
> profile's unmeasured shape (`stubs.md` #20).

### 4.7 § 5 — the inventory corrections

- **D-0:** delete *"lost from the record" / "never written down"*. Replace with: **"The reason
  WAS written down — `ROADMAP.md:2376` § Sequenced, USER'S STRONG LEANING, 2026-07-25, in the
  user's own words — and one session still re-derived it from scratch. Read that entry before
  touching weathering."** What D-0 adds is only the link to the flip.
- **D-1:** **THREE** authorities (`ROADMAP.md:2395-2400`), adding
  `geology.rs::emplace_weathering_front` — *"not declarative — ordinary collapse code that
  invents the vertical distribution. Only one of the three is authored in the shape everything
  converges on."* Keep the two-body aphorism as the sharpest *pair*. Add F16's ON/OFF clause.
- **D-2:** correct the `stubs.md` #25 pointer to **DISCHARGED 2026-08-02/03**, note
  `MOVER_NONE = made in place`, and defer the consequence to the D-2 verification.
- **D-4:** add the **7/8 cap** finding (`ROADMAP.md:2408-2410`) and that modelling the cause
  makes **stubs #20 DELETED, not tuned**; cite #20 alongside #16.
- **D-5:** split into **D-5a** (write path — narrower than stated, since purity forbids it by
  type; still unverified for the planes) and **D-5b** (read path — **measured 2026-08-04,
  "roughly one-quarter true"**), and add: **⚠ the reconciliation against method item 5 and
  move C is OPEN and USER-OWNED** — a strict reading outlaws all sub-460 m relief in the
  shipped world.
- **D-6:** re-mark **FILED as `stubs.md` #52** (2026-08-04, journal/0154, deposition-clock
  design pass F4); adopt its sharper statement (chapter is rewritten in place, so chapter and
  epoch disagree after overprint) and its narrowing (the correlation partition uses epoch).
  Correct § 5's banner: *"nothing on the board describes it"* is no longer true of D-6.
- **D-9:** soften *"Not found: any reasoning about the discard"* — the **discard** is
  unreasoned, but the **argmax was a decision** (`stubs.md:899-903`, `:914-922`;
  members-into-history `:310` costed **Option B, a share vector**), and **corrections #57** is
  the worked case where the argmax produced a wrong world. Use it; it is D-9's best argument.

### 4.8 § 6 (F7, F12)

- Replace *"Cost unpriced"* with the measured pair — **gen time 25.7 s → 46.4 s (affordable);
  residency `LedgerField` 17.45 MiB → 973 MiB, 55.8×, resident (not)**; S20 costed four
  options including paged facts.
- Add the **split decision procedure** — `material-behavior.md:349-371`: **separate passes,
  decided by coupling timescale vs cadence**, with the moddability tiebreaker. § 6 currently
  treats a ruled question as open.
- Add the **reactant-transport requisite** (`ROADMAP.md:2417-2421`): no vertical fluid flux
  means a depth-resolved rate has nothing honest to read, and emergent gradation from a
  fabricated depth term is **A-1 with a disguise**. R1/R2/R3 **MET**, with the recharge-free
  caveat (`stubs.md` #19) surviving into the arc.
- Replace *"has simply never been owned"*: **it is owned and half-built —
  `derive_regolith_at` derives `H` from an EMPTY ledger** (`stubs.md` #17's residual;
  `spines.md:34`; `ROADMAP.md:4156-4171`, verified still open; `field.rs:822` / `:904`).

### 4.9 § 7 — pointers

Add: `docs/design/pass-declaration-history.md` (the declaration archaeology) ·
`docs/design/knowledge.md` req 3 · `docs/spines.md` **A-7**, **S-2**, **S-6**, § 3 ·
`docs/design/earth-processes.md:9-29` **by number** (and disambiguate: *method* items, not the
*engine* sketches at `:31-258`, which are explicitly not decisions) and `:416-420` ·
`docs/audits/2026-08-04-voxel-explainability-audit.md` ·
`docs/audits/2026-08-04-deposition-clock-design.md` ·
`docs/audits/2026-08-03-stratigraphic-correlation-design.md` · `ROADMAP.md:2376` ·
`ROADMAP.md:383` · `journal/0152` (a rock name is a summary), `journal/0154`, `journal/0157`.

### 4.10 § 3.6 — rate constants (F13)

Append: the **only legal method** (`ROADMAP.md:1658-1663`) · the **acceptance instrument**
(`examples/denudation_probe.rs`, the 1–10 m/Myr stable-craton band;
`earth-processes.md:380-386`) · **#59** (a derivation is only an anchor if its source cannot
move) · **#56** (the Phanerozoic register; which constants scale, and that `diffusion` does
not) · **#98**'s replacement rule (*a closed depression is legitimate iff a process owns it*) ·
**#55** (measure a mechanism's authority before believing it matters) · the **ratified
magnitudes** a re-pick must not silently move (`earth-processes.md:436-458`) · the
**`[[no-bandaid-tuning]]`** tag, so a grep for it finds this skill. **And flag the skill's own
exemplar:** the soil-production taper's literature source is asserted in code and **cited in
no document** — either cite it or drop *"done right"*.

---

## 5. Ordinals — reported, NOT assigned

Measured at base `dd575b652deb828f9e724ff0381190fbb5658406`. **A parallel BODIES session
shares this checkout, so these MUST be re-verified at merge** — `stubs.md:1466` and
`corrections.md:3617`/`:3638` both record renumberings caused by exactly this.

- `journal/corrections.md` — highest **#100** (`:4006`); **next free = 101**. The file is not
  in numeric order at the tail (`#99b` at `:4068` follows `#100`); letter suffixes do not
  consume an ordinal.
- `docs/design/stubs.md` — highest **#54** (`:2034`); **next free = 55**.

**Does anything here warrant one? (Sweeper's reading, unratified.)**
- **F3** (D-0's *"never written down"*, falsified by a live § Sequenced entry) is the one
  correction-shaped finding — a false claim in a procedure doc that instructs the next reader
  *not to look*. If recorded, `ROADMAP.md:2376` owes a banner in the same commit (read-first
  item 5).
- **F1 is NOT a new ordinal** — it is a **fourth instance of #65** and belongs as a line in
  #65, which already counts its sites.
- **F2 is NOT a new ordinal** — it is an **A-7 instance**, and its home is `spines.md`'s
  instance list, which read-first 0b says must be updated in the same commit.
- The rest are skill edits, not falsifications. **The integrator decides.**
