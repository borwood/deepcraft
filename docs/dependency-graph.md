# The dependency graph — what blocks what

**Purpose.** ROADMAP says *what order we chose*. This says **why that order is forced** —
which work is upstream of which, and what is genuinely ready to start today. It exists
because the board grew a *pickup* problem: entries were accurate and nothing converted an
accurate open item into a sequenced one, and because "is this engine or pack?" was being
answered from memory and got answered **wrong** on the highest-value item on the board
(corrections #71).

**Read it with `ROADMAP.md`, not instead of it.** ROADMAP holds the entries and their bodies;
this holds the edges. When they disagree, **ROADMAP wins on content and this wins on
sequence** — and the disagreement is a defect to fix in the same session, not to route around.

**Maintained in-session** — see § Process at the bottom. Last full pass: **2026-07-29**.

---

## 0. The partition, in the user's words

> *"This is work on the **default plugin pack** side, which is separate from the **engine/SDK**
> (which provides the primitives that plugins use to declare behaviors which the pass runner
> executes)."* — user, 2026-07-29

And read-first item 0, which settles the boundary cases:

> ***Every pass is content, including tectonics and erosion***; **pass ORDER is authored per
> world**.

**⚠ The trap this file exists to prevent.** On 2026-07-29 a cold session called the erosion
axis *"engine work"* because the previous night's close block said so, while `CLAUDE.md`
read-first item 0 said the opposite **by name**. Erosion is a **pass**; passes are **content**.
A close block is a **handoff, not an authority** (corrections #71). *If you are about to
classify something from memory, classify it from this table instead — and if it is not in this
table, that is the bug.*

**Never justify a placement by trust.** The trusted/untrusted split and ABI/WASM tiering are
**DEFERRED** (north-star § Deviations 1). There is **no difference in permission between
native and WASM**, so "the engine should own it because it's more trusted" is not an argument
that exists here.

---

## 1. ENGINE / SDK — the primitives plugins declare against

| # | thing | state | blocks |
|---|---|---|---|
| **E1** | **pass-graph kernel** — today **derives** order from `{reads, writes}` (Kahn + tie-break); validating an *authored* order is **E7** | **BUILT** (7 passes) | — |
| **E2** | **cell / record storage** | **BUILT** | — |
| **E3** | **RATE — per-pass cadence + a real `dt`** | **BUILT 2026-07-29** (journal/0123). Cadence is authored data (`CadenceTable`), sub-turns execute, `dt` is live in creep/uplift/thickening/inventory-weathering. Empty table = shipped world, hash-identical. **Follow-on BUILT 2026-07-29 (journal/0124): the `Schedule` sum type**, `deeptime::schedule` (`ARCHITECTURE.md` § Schedule; ROADMAP arc slot (f)) — seed = initial condition, epoch 0 fires for all, skip rule deleted. All three pre-loop incumbents audited to `Step`; the pre-loop blocks are gone. Terrain + strata hash-identical, the flow record's **vertical** faces moved (a fix) and gained `GOLDEN_FLUX`, which it had never had. `Seed`/`SeedAndStep` have **no production declarer** — heir is E7's sibling, declared epochs (ROADMAP slot (d)) | ~~E4's first extraction~~ **unblocked** · every pass that wants a phase length |
| **E4** | **field-solver primitives** — the S-10 gather; **the kernel owns its own stability bound** | **E4-1 EXTRACTED 2026-08-03** (byte-identically, all goldens green): `dc-core::field::FieldKernel` — explicit scheme, flux-form, the bound + sub-cycle derivation kernel-owned, the pass declares only its coefficient field; **⚠ NEEDS RATIFICATION (venue, audit U-3: dc-core)**. Creep declares against it; `sat.rs` NOT converted (E4-1b — needs `PairMin` + upper obstacle, the audit's E4-2b shape-proof). Design pass 2026-08-03 (`docs/audits/2026-08-03-e4-implicit-kernel-design.md`; user: in-band gen time "actually unacceptable" — the P2 ladder measured sub-steps ∝ M, 2 → 896, pure explicit-stability tax; the implicit unconditionally-stable kernel is the ruled answer-shape for P2 finding 7, Braun & Willett lineage). **OPEN: E4-2** (the implicit `Scheme::ImplicitBE`, picks U-1/U-4) · **E4-3** (adoption, pick U-2, sequenced against the P2 flip). Also the named fix for the deferred pit-safari's 38-min world cost | every future diffusing pass · P2's flip (gen-time conflict) · cheap high-M worlds |
| **E5** | **refinement primitives** — coarse→fine reconstruction; **the presentation layer** | **CAUTIOUSLY RATIFIED 2026-07-29 (user)** — `docs/design/refinement.md` (record families + term-keyed operators + three laws; evidence `docs/audits/2026-07-29-refinement-coupling-priors.md`). **Members are directions, not build orders: each needs its OWN design pass against the bones, and prior member plans are superseded AS PLANS** (user, at ratification). **#0 has its design pass (`docs/audits/2026-07-29-member0-coarsefield-design.md`, rulings in its header) and its FIRST BUILD SLICE SHIPPED 2026-07-29 (journal/0125), WALKED same evening** — cell lines gone by eye (U22 discharged); **the cell-wide blend's semantics REJECTED by eye and ride as INTERIM** (user: *"the whole cake is swirled now"*), heir = a far register **derived from refinement-operator budgets** (user sketch, § Observed field report — a post-operators design conversation, not a slice). The far site is K1 only; `summarize` ruled to the octree contract, move C (midpoint jitter) filed with an heir. Continuation slot: ~~(a) the octaves `DitherSource`~~ **✅ SHIPPED 2026-07-29 (journal/0128)** · (b) **PART SHIPPED (journal/0129): MM-1 ✅** (`sample_source_cell`, with a caller and its first law test) **+ the octaves adopted at the near member dither ✅** (goldens re-captured, cost measured neutral) — **REMAINING: MM-3's type + the per-column record restructure — FOLDED INTO P11 2026-08-01 (its ruling 5; slice 3 of the P11 build sequence)** — one slice, blast radius measured at 13 files / ~40 sites, cost **~4× `run_strata` per chunk** rather than the design pass's *"typically 1"* (the bilinear stencil is always 2×2). ⚠ **The claim that routed slot (b) is refuted at the site** — only 4 of 10 vanilla classes have >1 member and U3's own pose has a `Mixed` top span, so "member stepping dominates U3" is unsettled — and the new station it needed is a **PROVEN NULL** (stamped 2026-08-03, staleness F5: `2026-08-02-appearance-tour-p11.md`, 0 `Single`-movable surface spans world-wide; the question is mooted by the correlation redesign). ~~Live blocker for #1: flow.md's face-pairing rule, unratified~~ **face-pairing RESOLVED 2026-07-29 (three-mode confinement rule, cautiously ratified — `flow.md` § 11.5 banner; corrections #75: it was never actually unratified for the mode the channel needs). #1's live blockers are now the RECORD TERMS (grain distribution + mobility hint, ruled records-first 2026-07-29) and P2 (build-after-P2 ruled same day). Record terms are now FULLY SPECCED — ruling 3 (face, 2026-08-02) closed the last held question, so the recording slice is buildable; and the user's ruling-3 rider stands on #1's resumption: revisit the partially implemented refinement machinery against the member-grade record** | the whole appearance cluster · `collapse.rs` decomp |
| **E6** | **open resource vocabulary** — `DeepAxis` retires; packs declare their own ids | **UNBLOCKED 2026-07-29** (E3 landed); sequenced | third-party packs |
| **E7** | **authored order + the validator** | **UNBLOCKED 2026-07-29** (E3 landed); sequenced. **It brings the per-world manifest**, which is the loader `CadenceTable` was shaped for | plugin-agnosticism |
| **E8** | **S2 statistical tier** | **HELD** — probable future primitive, zero consumers, *do not find it one* | nothing. **Gated on bio/eco, a USER call** |

**E4's rule, and it is the whole reason it is engine-side:** four inputs set the stability
threshold and they have four owners — the **stencil** (kernel), **`dx`** and **`dt`**
(engine), the **coefficient field** (content). Only the kernel can know its own constant.
Housed anywhere else, every plugin author needs a von Neumann analysis before writing a
diffusion pass. Housed in the kernel, **the unsafe call is inexpressible** — the same move as
`CoarseField` making the raw per-cell read unsayable.

---

## 2. DEFAULT PLUGIN PACK — the passes

| # | thing | state |
|---|---|---|
| **P1** | hillslope creep / erosion operator | **FIXED 2026-07-29** (journal/0122). Takes `dt` from RATE since journal/0123; ~~still sub-cycles **in the pass**~~ **the sub-cycle is the kernel's since 2026-08-03 (E4-1)** — the pass declares its coefficient field and applies the returned fluxes; `stubs.md` § 30 fully discharged |
| **P2** | `EROSION_CALIBRATION` re-pick + flag flip | **MEASUREMENT RUNS DONE 2026-08-02** (`docs/audits/2026-08-02-p2-measurement-runs.md`; derivation header stamped). Target **doubly confirmed** (2.63 derived ↔ 2.653 from the world's own Airy decomposition); measured D3(M) ≈ 0.0070·M puts the target at **M ≈ 375–380** — the derivation's [60, 240] bracket was low 1.6×. Chain B closed (⟨taper⟩ 0.80–0.83, supply-dominated). **The FLIP is blocked on three user calls** (runs doc § 4): the M re-pick · the pits-bar conflict (every in-band rung violates hollows>10 m = 0; 8,878 @400) · the gen-time/register trade (2,306 s @400 vs the 1,200 s budget; § 5's register↔Myr lever untouched). 45 was fitted to the broken solve and inverts under the fixed one |
| **P3** | flow / hydrology — face-flux record, head field | **BUILT AND IDLE.** Zero production consumers, *on purpose* |
| **P4** | CoarseField **adoption** (U22 + U3) | **HALF SHIPPED 2026-07-29 (journal/0125), and the halves are now separately blocked.** Routed through E5 member #0; the design pass re-derived the plan and the first build slice landed the **FAR site**: `collapse.rs::surface_class` is a `CoarseField<ShareVec<6>>` window drawn through `sample_dithered`, `DitherSource` has its first production impl (`draws.rs::Coherent`), **U22 (the cake law) is discharged with a world-scale test**. **U3 / the near site is now HALF SHIPPED too (journal/0129), and split along a line nobody had drawn:** its **28.8 m member-stepping half is done** — `interp_select_draw` retired into an `Octaves`-backed `selection_field`, **MM-1 discharged** (`CoarseField::sample_source_cell`, with `sample_dithered` rewritten in terms of it, plus the bilinear-weight law test that never existed), the dither hoisted per-VOXEL → per-(column, event) so the 6× hash cost measures **−1.2 %** on 240 chunk generations, 2 of 3 chunk goldens re-captured. Its **~460 m record half SHIPPED 2026-08-02 (P11 slice 3a)**: `ColumnRec.strata` dissolved into per-column `records: Vec<SubCell>` + `cell_of` (blast radius re-measured 14 files / ~71 sites before the cut — the count had grown since 0129's 13/~40), **MM-3's `SubCell` shipped WITH its consumer** exactly per journal/0129's withdrawal condition, and the presence-border acceptance instrument gates in `appearance_tour_p11`. ⚠ **And the claim that routed this work is refuted at the site:** the vanilla set gives only **4 of 10 classes** more than one member (all exactly two, all albedo-adjacent pairs), and at U3's own pose the surface top span is `Mixed` everywhere so the member dither is **inert there** — so "member stepping dominates U3" is unsettled and the recorded pose cannot settle it (ROADMAP § Observed). ~~Owed: a tour map that finds a `Single` top span in a two-member class~~ **answered 2026-08-02 with a PROVEN NULL** (stamped 2026-08-03, staleness F5: `2026-08-02-appearance-tour-p11.md` — 0 of 4,792 world-wide, positive control intact; that station does not exist and the walk should not launch for it) |
| **P5** | genesis passes / **facies driver** | ratified concept, gated behind an `🔖 OPEN EDGE` |
| ~~**P6**~~ | ~~**octaves** as a `DitherSource` impl~~ **MOVED TO ENGINE 2026-07-29 → E5 member #0** — this row and § 3's "P6 is engine" said opposite things four rows apart (caught by the member-#0 design pass). `DitherSource` runs per voxel (granularity table: per-voxel draw = engine primitive) and is a crate-layering seam, not a plugin seam: **a pack SELECTS a source by id, never supplies one**. **The socket is now occupied 2026-07-29 (journal/0125): `draws.rs::Coherent` is the first impl**, and it fixed the API friction the design pass flagged — one salt dimension maps to `(domain fixed at construction, salt = tag)`. **✅ THE SECOND IMPL SHIPPED 2026-07-29 (journal/0128): `draws.rs::Octaves`** — a normal-score-transformed fBm over a pairwise-coprime prime stride ladder (509 … 13 voxels, head = the deep cell). Two things the build decided that no prior plan had: **the naive fBm SUM is unusable for a source feeding an inverse-CDF draw** (σ ≈ 0.085 around ½ — classes outside the middle half of the interval would never be drawn), so the octaves are **Gaussian corners + `Φ(S/σ)`**, which is the geostatisticians' truncated-Gaussian facies construction and makes the marginal **uniform by construction**; and **prime, not dyadic, strides**, because a dyadic ladder keeps a 32-voxel kink lattice. Measured: on/off-lattice curvature ratio at the chunk scale **1.36** against `Coherent`'s **1.0e14** | the U3 / near-site half of P4 (co-requisite) — **consumed there in journal/0129** |
| **P7** | metamorphism (grade from `exhum`/`t_crust`) | unblocked by the geotherm; unstarted |
| **P8** | material **FORM** from provenance | `stubs.md` § 12 — the sub-voxel sieve deletes ~75 % of the sediment pile |
| **P9** | bio / eco / socia / civ | **NOTHING EXISTS. ON HOLD, not never.** Gate is a **USER call**; earth-science progress does not open it |
| **P10** | **grain-size continuum** — release spectra on materials + size state evolved by transport (abrasion + sorting), recorded at deposition | **SEQUENCED 2026-08-01** (user, fluvial record-terms ruling 1 — ROADMAP § Sequenced "THE GRAIN-SIZE CONTINUUM"). **Design pass DONE 2026-08-01; rulings landed: U1 grain-is-an-AXIS · U5 packed-`DepUnit` funding · U7 RULED 2026-08-02 ("R2 — authored edges"): release spectra are pack-authored EDGE PRODUCTS, primitive general (any `MaterialId` product; provenance-keeping is vanilla's authoring — `material-behavior.md` § 3). U6 collapsed by U1. U4 RULED 2026-08-02 (user): 5 φ classes, the ladder's rungs. **FS-A ✅ SHIPPED 2026-08-03 (journal/0146, merge `4635056`)** — the `EdgeProducts` primitive + 8 literature-cited vanilla tables + `InvCtx::release` with bit-identical vanilla collapse; **the grain WRITER was withheld on semantic grounds** (grade-from-own-spectrum is O-1 in disguise; split factor measured ≤1.0329×, gate passes, semantics wait) — **the honest writer is the PROPAGATED-grain transport slice, which is now P10's live next build** (seam marked: `grain_write_seam` + `set_grain`; every unit verified GRAIN_UNSET). Still open U2/U3 + the grade/form legibility presentation question (§ Observed 2026-08-02, user-led).** **Calibration gated on P2** (pre-P2 no cell can carry sand). Named heir of: the composition-not-size distinction (record-terms priors § 1.6), stubs #23, `earth-processes.md` § 3e's "grain continuum". Blocks: fining-upward beds, placers, member #1's full § 7.1 grading. **⚠ In P11's ripple** — its load state has the same class-vs-member grade question |
| **P11** | **MEMBERS INTO DEEP HISTORY** — the deep record goes member-grade; identity is recorded, expression expresses | **DECIDED 2026-08-01 (user), TOP PRIORITY** — ROADMAP § Sequenced "MEMBERS INTO DEEP HISTORY" + `materials.md` § DECIDED 2026-08-01. **Design pass DONE (`docs/audits/2026-08-01-members-into-history-design.md`, rulings in header): the record names `MaterialId` (ruling 1) · representation = A-CLEAN, no class view survives storage or physics (ruling 2 — D killed by the user's entrenchment challenge) · transport planes SPARSE DAY ONE (ruling 3 — dense is the wrong asymptote for an open registry).** ~~Still open before build: pack-members-move-terrain · MM-3 disposition · the build sequencing~~ **ALL CLOSED 2026-08-01: U5 dissolved (corrections #84 — the 2026-07-19 world-identity rule answered it), MM-3 FOLDS IN (ruling 5), and the 4-slice BUILD SEQUENCE is drafted in the ROADMAP arc.** Ripple map in the audit (24 rows). Record-terms ruling 3 re-enters after slice 2. Spans the partition on purpose — the *record* is E2 storage, the *identities* are pack content. **SLICE 1 BUILT 2026-08-01** (identity swap + deposition-time fitness): the deep sim is now **content-aware** — `GeologySet` threaded into the deep-time runner, fitness at deposition, `DepUnit.species: MaterialId`. **New edge this reveals:** the pack set must reach *pregen*, not only collapse — `Pregen::run(WorldParams)` carries no content set, so the deep tier defaults to vanilla (stubs #35 (renumbered from a wrong #34→#33 pointer 2026-08-02); heir = E7's per-world manifest). **SLICE 2 SHIPPED 2026-08-02** (foundation journal/0138 + erosion/ split journal/0139 + conversion journal/0141, merge `07bd694`): CSR planes 0.70× the class-grade dense they replace (p = 3.721/8 max) · ruling 6 executed — 77 % of recorded metres identity-by-propagation, **31.9 % of the record would have been named differently by site climate** · salt collision closed · all 14 golden families re-captured once · full workspace gate green after the user's pregen-budget renegotiation (60 s → 1200 s, *"as long as it doesn't take 20min"*). Record units 1.9064× — recovered by slice 3's ratified packed `DepUnit` (U1/U5 rulings 2026-08-02: grain is an AXIS on the loose form; grain + agent axis funded from the pack, gated on a measured grain split factor). **U1 also re-shapes FS-A** (emit loose source-identity + grain state, not ladder ids) — FS-A is now startable. **Record-terms ruling 3 RULED 2026-08-02 (user): FACE** — sparse CSR sidecar on the flux record, `DepUnit` stays single-species; rationale is directional provenance for the refinement operators (priors header carries the ruling + the corrected pointer to the scattered numbers). **Slice 3's design pass DONE same day** (`docs/audits/2026-08-02-p11-slice3-design.md` — 8 user picks pending; packed layout closes at 8 B/unit; the grain split-factor gate transfers to FS-A's writers, the mover split is what slice 3 must measure). **SLICE 3 ✅ SHIPPED 2026-08-03** (journal/0145, merge `cd1058a`; full trio 938/2 with both reds accounted; **the ~460 m tile is dead in code — and the membership DITHER that replaced it is REJECTED by the user's live view and SUPERSEDED-AS-PLAN the same day by the stratigraphic-correlation design (`docs/audits/2026-08-03-stratigraphic-correlation-design.md`, user sketch + smoothness principle; read-side, deep goldens must stay still; 5 picks pending its § 9) — the `SubCell` stencil, accessors, and packed unit survive as its substrate**): per-column record membership (`SubCell` + `cell_of` + `DeepField::cell_bundle`) AND the 8 B packed `DepUnit` in one merge (the audit's priced combined option); M0 measured mover split **1.0308×** → mover IN the merge key (M-1); grain bits named + UNSET for FS-A (`grain()`/`set_grain()`/`GRAIN_UNSET`); stubs #25 discharged, #31 heir re-pointed. Slice 4 (Litho dissolution) remains |

---

## 2b. BODIES — engine data-model API + pack content (added 2026-08-01)

| # | thing | state |
|---|---|---|
| **B1** | body plans / clips / validator / registry / per-character selection | **BUILT**, and **B0 reshapes it.** journal/0130 routed the **default pack** through the registry door (A-4 discharged); `dc:body/{biped,stout,longleg}` register, plan selection works end to end |
| **B0** | **body-plan STRUCTURE — the declaration slice** | **✅ SHIPPED 2026-08-01 (journal/0135), shape ratified by the user the same day (collide deferred to B4 — left open for its machinery).** The five rulings collapsed into *one mechanism*: `RoleDef` (open vocabulary, segment-local anchors) + `ModeDef` (bearing roles per mode) + `actions` (open — `KNOWN_VERBS`/`REQUIRED_VERBS` **deleted**; the check is *you supplied what you claimed*). A tree and a bird now define. Three name-couplings retired onto declarations: `leg_rigs`' string surgery, `== "neck"` / `== "head"` (now **unique-or-loud** role queries), longleg's prefix mutation. Mirror helper reproduces the hand-typed right side byte for byte. Design pass: `docs/audits/2026-08-01-body-plan-structure-design.md` (rulings in its header). **Clip binding stays ADDRESS-EXACT** — role binding for animation (+ clothing, fork inheritance, sockets) is **stubs #34**, heir = the gait-bake design pass, deadline = before B3/B5 move the firewall; cardinality enforcement travels with it. *The segment-identity window was used while it was free: recompile, no migration, no golden moves* |
| **B2** | **posture + gait bake** — derived, per species, ~~at pack build~~ **at pack build OR deeptime worldgen (corrections #86)** | **CAUTIOUSLY RATIFIED 2026-08-01** — `docs/design/posture-gait.md`. Bones only; **members are directions, not build orders.** ~~First slice sequenced, blocked on nothing~~ — **SUPERSEDED the same day: sequencing is `structure → posture → gait` (user).** ~~The resting-posture bake is **B0's successor, not the next thing**~~ — **B0 SHIPPED 2026-08-01; member #0 design pass + bake SHIPPED 2026-08-02 (journal/0137, measured == predicted to 1e-9) and the CONSUMER SLICE SHIPPED + WALKED same day (journal/0140; 0138→0140 renumber) — user verdicts: rest \"reads right\", motion \"fine with caveat\".** ~~The caveat is the STRAFE (Observed 2026-08-02)~~ **→ the STRAFE is RESOLVED 2026-08-02 (next session): look ownership DECIDED (user) — move intent defaults to look-follows-travel; `set_look` HOLDS until the new `clear_look` releases it; shipped sim-side in `step_character` with `look_held` in the pose readback (bodies.md § who owns the look, journal/0142).** **✅ MEMBER #1 IS BUILT, CONSUMED AND WALKED (2026-08-03).** Slice one = headless `bake_gait`
(journal/0144, predicted table confirmed to ~1e-5); slice two = the consumer (journal/0147) —
the renderer samples the derived gait, `root_bob_m` left the schema for one
`root_offset(posture, mode, phase)` read by both the IK hip and render root, `biped_walk`
retired as content, the binary `Loco` switch replaced by the continuous Froude ladder
(stubs #42 discharged), trunk facing split sim/client. **Walked and verdicted (journal/0148):**
bob gone at rest, cadence spread *"very clear"*, derived bob accepted as *"a tad exaggerated
but — they're block people."* Cost 2759 ns/body/frame (1.62× the two-clip sample it replaces).
**New stand-in: `swing_gain` (stubs #43)** — the design's `clearance` was speed-invariant, so a
body stopping mid-swing would freeze with a foot in the air; heir is the **stop transition**,
which posture-gait § 4 explicitly does not design. **New finding: a fixed 12 fps ALIASES the
derived cadence** (stout 4.29 frames/cycle vs biped 6.97) — ROADMAP § Observed; the user's
leaning is forfeit, not yet ruled. ~~THE GAIT-BAKE DESIGN PASS SHIPPED 2026-08-02~~ (`docs/audits/2026-08-02-gait-bake-member1-design.md`; five user calls ruled in its header — the speed constant re-typed with a continuous Froude ladder, three keyframes not two, `root_bob_m` out of the schema with its derived replacement, `biped_walk` retired as content, trunk facing split sim/client). **BUILD NOT GREENLIT — the user is reading the pass first** (member #0's precedent: an explicit greenlight before code). **NEXT in arc: B7's joint-limits design pass (upstream of the gait BUILD — the bake must check its output against limits that exist), then the gait build.** *(Superseded line follows.)* ~~NEXT in arc: the gait-bake design pass~~ (which owns stubs #34 before B3 moves the firewall, **and now also the BOB: ~~"corrections #80's temporal bob, the last un-glued-feet item"~~ was a fix-shaped framing falsified by the user 2026-08-02 (corrections #93) — the authored clip bob has no contemporary ratification, and whether any body bobs, which, and how much are per-species bake outputs (`posture-gait.md` § 1). #80's space-layering fact rides until that pass rules.**) The solver's contacts input reads **declared soles** (B0's `segments_with_role`), dissolving the design pass's § 8 `leg_rigs` interim — the N=1 trap (corrections #78) is structurally closed. Bake key is `(species, posture, MODE, yaw)` — mode added by the alligator, growth dropped by decision 24. **The gait-bake design pass additionally owns stubs #34** (clip role-binding + cardinality, ruled 2026-08-01) — it must land before B3 moves the firewall |
| **B3** | sim-side animation **phase** + the firewall's new line | direction (§ 7 member 2). Brings the 20 Hz vs 12 fps cadence choice |
| **B4** | derived collider sets (bounded `k`, yaw buckets) | direction (§ 7 member 3). Retires the world-global `CharacterConfig`, under which a 1.60 m stout is hit as 1.8 m |
| **B5** | per-segment damage → injury → gait delta (the limp) | direction (§ 7 member 4). Needs B2 + B3 |
| **B7** | **joint rotation LIMITS** — declared per joint, derived default | **DECIDED 2026-08-02 (user), UNBUILT — `bodies.md` § Joint rotation limits.** Opened by the user at the gait pass's keyframe ruling (*"hyper-extension just shouldn't be possible if we set rotation limits on joints"*); the sweep found **no limits anywhere** — `SegmentDef` has no rotation range, and `solve_leg_ik:485,493` clamps only for `acos` domain and reach. **A validated constraint, never a runtime clamp**: bake refuses loudly, validator rejects out-of-range clips at define time, IK solves *within* the limit set. **UPSTREAM of B2's gait member**, and it wants the segment-identity window before **B3** makes the pose a versioned sim asset — same deadline as stubs #34. The **derived default** is what makes it survive evolution (nobody authors ranges for a species deeptime invented); the derivation itself is owed a design pass |
| **B8** | **individual proportion variation** — plan-declared ranges, salted per individual | **DECIDED 2026-08-03 (user), UNBUILT — `bodies.md` § Individual proportion variation.** **SIZE FIRST** (declared allometric axes deferred). **RANGE is engine, DISTRIBUTION is pack; sim-visible is fine and the PACK owns fairness** — so this does **NOT** wait on B4. Mechanism family is **allometry**: juvenile→adult growth and inter-individual proportion are the same thing, which makes the unit of variation an **axis**, never a per-param range (independent jitter cannot express growth at any width). **Size is FREE today** — the resting bake returns angles + a height ratio (`bake.rs:81`), so a scaled body has identical joint angles and needs no re-bake. Fourth **S-9** instance. **Evolution mutates the PLAN including its axes** — a deep-time mechanism that must not share machinery with per-individual jitter. Texture-side variation is a separate pipeline |
| **B6** | per-segment **materials/mass** — ⚠ **EIGHT named inbound heirs, and four of them shipped 2026-08-03 without reaching this table until the user asked (corrections #97).** Consumers of the one mass integral: harvest yield · evolutionary fitness · standing posture · flotation · and the four `GaitKnobs` force-shaped stand-ins `cadence_scale` / `duty_exponent` / `bob_damping` / `swing_flexion` (`stubs.md` #44), plus the run's flight phase (#39) and the mass-is-volume proxy (#40). **`bob_damping` is the one with a measured target: 4.6 cm real vs 6.6 cm compass (Saunders/Inman/Eberhart 1953), and the shipped biped derives 6.58.** | unstarted. **Feeds B2's mass integral** — and the *same* integral serves harvest yield and evolutionary fitness |

**The edge that matters most, and it is the one nobody had written down:** **B2 → the whole
bio/evolution arc.** Authored clips break the moment topology changes, so an evolution pack
would generate bodies **nobody could animate**. A bake from `(segment tree + masses)` means a
mutated body gets a plausible stance and gait **by construction**. Bodies are engine (a named
core data-model API); the bake's **solver** is an engine primitive, the **body** is pack content,
the **baked result** is derived data in the pack's compiled form. *This does not open P9 — the
bio/eco gate is a USER call and engine progress does not earn it.*

**Placements ruled 2026-08-01, recorded with WHERE so they are not re-derived from memory**
(corrections #71's lesson): the **bake's solver is an engine primitive** — same argument as the
field kernels, only the kernel knows its own bound; the **body is pack content**; the **baked
result is derived data in the pack's compiled form** ~~produced at *pack build* rather than world
gen~~ — **VENUE CORRECTED 2026-08-02 (user, corrections #86): the purity that was cited as the
reason for pack-build-only is what makes the bake callable from ANY clock, and evolution mints
species DURING DEEPTIME WORLDGEN, so worldgen is a first-class caller** (authored species bake at
pack build; evolved species bake when minted, inherited down the phylogeny like the frond). Ruled in
`posture-gait.md` § 9 and § 3. **Segment KINDS** (`Box`/`Card`) are a **closed machine-owned
set**, governed exactly like `material-behavior.md` § 2's forms — sketch only, `ideas.md`.

**Postures DECIDED 2026-08-02 (user, sweep-checked first): registry content by namespaced
id; engine owns the contract; default pack ships the 2026-07-19 posture ladder; controller
surface = introspect + invoke-by-id** (`bodies.md` § Postures — the F3/Q1 answer). The
enum→id wire migration is append-shaped and rides the sim-visible slices (replay-critical).

**⚠ The `bodies.md` § IK "user call" is CLOSED, not open** (corrections #81). It was dissolved by
`posture-gait.md` § 1 on the day that document was ratified and then re-presented to the user
twice regardless. Hip height, root bob and crouch drop are **deleted, not corrected**. Do not
re-open it.

**B2 is upstream of the `bodies.md` § IK user call, not downstream.** The 20 mm hip/reach gap
and whether four absolute-metre constants become ratios are answerable **with a number** once
hip height is derived.

---

## 3. The edges that actually decide the order

**E3 (RATE) → P1. ✅ DISCHARGED 2026-07-29 (journal/0123).** `ARCHITECTURE.md` argued the
creep blocker was *what RATE's absence produces*; the pass, given no engine clock, **grew its
own** `dt`, confirming it from both sides. RATE landed against that known-good fixed point and
the hash comparison came back **identical** — `GOLDEN_SURFACE 0x15A6_B756_7A84_29FB` /
`GOLDEN_RECORD 0x820B_A198_49DD_234A`, `tests/rate_axis.rs`. The pass now takes its phase
length from the engine; **only the derived sub-step count stays inside it**, which is E4's half.

**E3 → E4, the edge the slice sharpened.** RATE deliberately does **not** own stability
substepping (user ruling, 2026-07-29). So the two divisions now sit four lines apart in
`Erosion::diffuse` — *authored* `dt` from outside, *derived* `n_sub` inside — which is exactly
the shape E4 has to hoist into the kernel. **E4's target is now a concrete two-line pattern in
one function rather than a design sketch**, and `sat.rs` is its second instance.

**P1 + `sat.rs` → E4.** Extraction needs two instances and a ruling on who owns the stability
bound. **Both landed 2026-07-29.** E4 is ready to design.

**P3 → E5.** *"The primitive the channel needs is the one FLOW spent three slices building"* —
face fluxes as Dirichlet conditions, the load budget as mass. **E5's inputs are built and
sitting idle.** This is the strongest ready-now signal on the board.

**E5 → `collapse.rs` decomp → the file-size chore.** 2,415 lines, *essentially zero
declaration*. One slice discharges two obligations.

**P4 is ~~INDEPENDENT of E5~~ NOW ROUTED THROUGH E5 — superseded 2026-07-29 by the
refinement ratification.** The user's ruling at ratification: first members *"should be
revisited in light of this design, not taken as wherever they landed prior."* CoarseField
IS member #0, so its adoption plan is superseded *as a plan*: the member-#0 design pass
(dispatched 2026-07-29) re-derives it against the bones — likely reaching the same two
user-report fixes, but that is its finding to make, not this file's to assume. The old
independence argument (seam-first, touch `collapse.rs` twice) is preserved as input to
that pass.

**P4 → P6. ✅ HALF DISCHARGED 2026-07-29 (journal/0125).** Octaves are a legal `DitherSource`
impl. The socket already exists; the octaves supply the **source**, `sample_dithered` supplies
the **draw**, `summarize` retires the residual bias. **They are not rivals — they compose**,
and nothing in the corpus said so, which is why two ratified designs read as contradictory for
a week. The far slice occupied the socket with a *coherent* impl and proved the composition
concretely; **two of the three parts moved and the third did not.** `summarize` is now ruled
OUT of this tier (→ octree node contract), so **the residual majority-amplification bias is
not retiring here** — it rides in the shipped far field, at both salts, exactly as it did
before. The edge that remains is **P6 → the U3 half of P4**, and it now runs the other way
from what this heading implies: the near-path fix alone will not clear the checkerboard, so
the octaves source is a **co-requisite of U3**, not a follow-on of P4.

**P5 is content, P6 is engine.** The user's remembered design splits cleanly across the
boundary already ratified: **facies driver → a declared field pass (content)**; **octaves +
dither + summarize → refinement primitives (engine)**.

**P1 → P2 → the flag flip.** Nothing downstream of erosion can be judged until the constant is
re-picked, and it must be picked **against the published literature**, never against a look.

---

## 4. Ready to start today, in order

1. ~~**E3 — RATE.**~~ **✅ SHIPPED 2026-07-29 (journal/0123)** — hash-identical, and E6/E7 are
   unblocked. *The rest of this list is unchanged in order; E3 leaving the top does not promote
   anything past what already justified it.*
2. ~~**E5 — the refinement design pass.** A whole tier with zero members that the north star
   requires for plugin-agnosticism, with its inputs already built and idle.~~ **✅ DONE
   2026-07-29 — see the E5 row above** (ratified doc, member #0 shipped + walked; items 1
   and 3 of this list were struck for the same day's work and this one was missed —
   doc-topology F6). What is startable now: the member-#0 continuation pair (octaves +
   near path, in flight), the fluvial member design pass (open, main session), E7, E4.
3. ~~**P4 — CoarseField adoption.**~~ **HALF SHIPPED 2026-07-29 (journal/0125)** — the far
   site landed and U22 is discharged. What is left of P4 is **U3 / the near site, and it is
   no longer "blocked by nothing"**: it needs MM-1 + MM-3, and the **octaves `DitherSource`
   is a co-requisite** — the U3 checkerboard's dominant signal was settled 2026-07-24 as the
   **single-octave member dither**, so the near-path fix alone leaves the squares on screen
   (ROADMAP § Observed; corrections #45). Octaves are now the cheaper of the two and have a
   worked socket to land in.
4. **P2 — the calibration re-pick.** ~~Needs a literature pass, not an engineering one.~~
   **The literature pass SHIPPED 2026-08-02 (`ee9fb94`); what it needs now is the
   measurement runs (build slot), queued behind P11 slice 2's merge.**
5. *(cheap, rides alongside anything)* the **cold-tier appearance probe** — render a cold tile
   and the same ground loaded, diff them. Never once run.
6. *(owed by E3, small, and NOT a blocker)* **the remaining `dt` conversions** — stream
   transport, bedrock weathering, the wind and wave agents still assume `dt = 1.0` in their
   magnitudes. Each needs a **modelling** call, not a mechanical one (weathering is an
   exponential approach: `1 − exp(−k·dt)`, not `k·dt`), and picking one silently re-tunes a
   constant **P2 is about to re-pick anyway**. *Do these WITH P2, against the literature — not
   before it.*

**Blocked or deliberately parked:** P9 (user gate) · E8 (held) · P5 (OPEN EDGE) · `ores.md`'s
conceptual revisit (owed, genuinely unscheduled).

---

## 5. Process — how this file stays true

Added 2026-07-29 at the user's direction: *"fold in graph-building to the main session's
process, checking it at start and end of session and updating it throughout."*

- **START of session** — read this with the ROADMAP close block. It is the fastest available
  answer to *"what can I start right now, and what is it downstream of?"*
- **THROUGHOUT** — when a slice lands, moves a state, or reveals an edge, update the table in
  **the same commit**, exactly as ROADMAP is updated. *An edge discovered and not written down
  is the failure this file was built to stop.*
- **END of session** — the `wrap` ritual checks it. A slice that shipped and left a row stale
  is an unclosed loop.

**The two failure modes to watch, both already observed here:**
1. **A stale classification outliving its reason** — the erosion/engine miscoding lived one
   night in a close block and reached a cold session intact. **State the placement, and where
   it was ruled.**
2. **An edge nobody wrote down** — `sample_dithered` was built, named for a user law, and
   uncalled for seven days while being absent from all three loose-end loci. **A built-and-idle
   thing is an EDGE, not a rest state**; it is either about to be consumed or about to be
   deleted, and § 1/§ 2 must say which.
