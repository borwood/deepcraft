# The deposition clock — carrying per-bed TIME in the deep record: design pass

**Read at commit `0ac5b7f`** (worktree `agent-aaf7e4dad3d110a7a`, merged with `main` at that
tip before reading — `main` was already at `0ac5b7f`). **No cargo was run**; every number
below is arithmetic over an already-measured quantity cited to its source, or a hand
computation flagged as such.

**THE ANCHOR — user, 2026-08-04, verbatim. This pass EXECUTES it; the anchor is never an
input to reconcile away.**

> Of stratigraphic correlation's R-C proportional rule: *"i don't quite understand the
> underlying architecture apparently that we need to re-derive which layers match layers in
> others. **they were laid down on the same clock. did we throw the time away?**"*
>
> Greenlight: *"yes, go on that. **i tend to think there's some cleverness we aren't
> considering yet for storage / accurately deriving this.** but preserving the clock — as
> you say — pays for two other threads."*

The two threads the clock pays for: the **voxel-explainability WHEN axis**
(`docs/audits/2026-08-04-voxel-explainability-audit.md`, hole 1 — the funnel) and
**`docs/design/knowledge.md:25-28` requirement 3**, *computable anomaly*.

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option is priced
and its trade-offs named; picks marked **user-owned** are the user's. Anything the agent
originated is marked **(assistant-proposed)**.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (verified in code here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / needs the user's
eye).

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Anything that later
refutes or re-scopes this file gets a banner **here**, stamped by the author of the
correction, in the same commit.

**Read with:** `docs/audits/2026-08-03-stratigraphic-correlation-design.md` (F1, § 1.2 R-C
and its named weakness, § 10.7's flag; **P-1 and P-2 are RULED** — banners in its header) ·
`docs/audits/2026-08-04-voxel-explainability-audit.md` (the funnel, ranked hole 1) ·
`docs/audits/2026-08-02-p11-slice3-design.md` (L-8's packing ruling) · journal/0141 +
journal/0145 · `crates/dc-worldgen/src/deeptime/recorder.rs` ·
`crates/dc-worldgen/src/deeptime/cadence.rs` (RATE's base-unit ruling).

---

## 0. The answer to the anchor, in one paragraph

**Yes, the time was thrown away — but not where the question assumed, and the correction
makes the fix cheaper rather than harder.** The epoch was never *packed and discarded*: it
was **never handed to the recorder at all**. `DeepStrata::deposit_moved` takes
`(tag, d, chapter, species, mover)` (`recorder.rs:901-908`); the epoch is not an argument,
and the caller that could supply it — the erosion pipeline — deliberately stores only
`cur_chapter` (`erosion/mod.rs:275, :543-546`). Meanwhile **the chapter is a pure function
of the epoch**: `TectonicSchedule::blend_into` computes
`c = min(floor(epoch / (iterations/chapters)), chapters−1)` (`mod.rs:487-503`) and is the
sole writer of `cur_chapter`, firing `Schedule::EVERY_EPOCH` (`runner.rs:729-738`). So the
record spends **8 bits** storing a **3-bit quantity** (8 chapters, `grid.rs:582`) that is
itself a lossy projection of the 8-bit quantity we now want. **The record is not missing a
clock; it is storing a coarsened shadow of one, in a field wide enough to have held the
original.** And there is a second, entirely unspent home: the thickness word's top byte
(§ 2, O-2b). **The clock can be restored at zero bytes, zero unit-count change, and — if
the record fingerprint is left alone — zero golden movement.** That is the "cleverness for
storage" the greenlight suspected; it is § 2's O-2b, and it is this pass's recommendation.

---

## 1. Findings

### F1 — The epoch is available at every depositing call site, and is dropped one level above packing

The runner's loop counter **is** the epoch: `DeepStepCtx.epoch: u32` (`runner.rs:226`), set
by the runner before each pass fires (`:225`). Seven production deposit sites exist in
`crates/dc-worldgen/src/`:

| site | call | pathspec |
|---|---|---|
| erosion recorder | `deposit_moved` | `deeptime/erosion/record.rs:85` |
| eolian agent (×3) | `deposit_moved` | `deeptime/erosion/agents.rs:98, :165, :194` |
| wave agent | `deposit_moved` | `deeptime/erosion/agents.rs:305` |
| biotic overprint | `overprint_top` | `deeptime/biotic.rs:760` |
| biotic charcoal | `deposit_as` | `deeptime/biotic.rs:773` |

All six erosion/agent sites read their chapter from `Erosion::cur_chapter`
(`erosion/record.rs:196`, `agents.rs:49, :233`); the biotic pair read it through
`Erosion::current_chapter()` (`biotic.rs:729`, `erosion/mod.rs:552-556`). **None of them has
the epoch in scope today** — `Erosion` carries `cur_chapter` and no epoch field
(`erosion/mod.rs:268-275, :503`).

⚠ **The plumbing that this forces, and it is not free-of-thought:** `set_tectonic` — the one
place `cur_chapter` is written — is called only from `tectonics_pass`, which is **registered
only when `cfg.tectonic_history`** (`runner.rs:729-738`). On the off path `cur_chapter`
stays at its `0` initializer (`erosion/mod.rs:503`) forever. So an epoch stamp cannot ride
`set_tectonic`; it needs its **own unconditional setter driven from the epoch loop**
(`Erosion::set_epoch(ctx.epoch)`, one line in the loop body, ~3 lines total). Small, but it
is the difference between an honest clock and one that silently reads 0 on the legacy path —
exactly the failure mode the chapter axis already has and nobody noticed, because chapter 0
is *correct* there and epoch 0 would not be.

### F2 — CHAPTER IS DERIVABLE FROM EPOCH; THE CONVERSE IS FALSE

`blend_into` (`mod.rs:487-503`), verbatim in structure:

```
ipc = iterations.max(1) / chapters.max(1)          // 200 / 8 = 25  (production)
c   = min(floor(epoch / ipc), chapters − 1)
```

Corroborated independently in the flux module's own docs — *"a chapter is **25 epochs**
(200 iterations / 8 chapters)"* (`flux.rs:26-27`). The pass fires every epoch
(`Schedule::EVERY_EPOCH`, `runner.rs:735`), so `cur_chapter` is exact, never lagged.

Two consequences, both load-bearing:

1. **On the tectonic path the chapter byte carries no information the epoch does not.**
   Storing the epoch subsumes it.
2. **On the non-tectonic path they genuinely differ** — chapter is pinned at 0 while the
   epoch runs 0..199. So chapter cannot simply be *replaced* by epoch without a
   config-aware derivation (this is what makes O-5 more invasive than it first looks, § 2).

### F3 — The epoch is NON-DECREASING up-stack, and the argument covers every writer

Read writer by writer in `recorder.rs`. The claim: if each unit stores the epoch of its
**first** deposit, `units[k].epoch ≤ units[k+1].epoch` holds in every cell at all times.

| writer | what it does to order | pathspec |
|---|---|---|
| `deposit_moved` — append | pushes at the top with the current epoch, which is ≥ every epoch below (the loop counter is monotone, `runner.rs:226`) | `:927` |
| `deposit_moved` — merge | `top.quanta += q`; the top keeps its own stored epoch ⇒ **bottom (oldest) age**, with no code | `:920-926` |
| `overprint_top` | touches only the top (and, on merge-down, the top two); merge-down pops the top **into** `units[n-2]`, so the survivor keeps `n-2`'s (older) epoch | `:949-1001`, merge-down `:993-1000` |
| `erode` | pops from the top and decrements the top's quanta; never inserts, never reorders | `:1008-1032` |
| `promote_coal` | rewrites `biota`/`species` in place; no positional or temporal write | `:1132-1164` |

**No writer inserts below the top and no writer reorders.** That is the whole proof.
It is the same argument correlation-design F1 makes for chapter, and it is *stronger* for
epoch, because chapter's monotonicity additionally depends on `blend_into` being monotone
while epoch's depends on nothing but the loop.

⚠ **Not exhaustively proven** — read from five writers and seven call sites, not from a
machine check. The honest conversion is a `debug_assert!` in the build slice (the same fix
correlation-design § 10.4 already owes for chapter; **one assert discharges both**).

### F4 — `overprint_top` rewrites the CHAPTER of an existing unit, and that is where the epoch semantics need a ruling

`overprint_top` calls `set_tag_and_chapter(tag, chapter)` (`:983`), which rewrites the top
unit's tag axes **and its chapter byte** in place (`:745-754`) — *"the horizon is a new bed"*.
Under an additively-stored epoch (O-2b) the epoch sits in the other word and is **not**
rewritten unless we choose to. So after a pedogenic overprint the stored chapter can say
*chapter 4* while the stored epoch says *epoch 37* (chapter 1). **That is an internal
inconsistency the design must resolve explicitly, not discover in a probe.** Two coherent
answers, both cheap:

- **(a) rewrite the epoch alongside the chapter** — the horizon's recorded age becomes its
  last alteration. Consistent with today's chapter semantics; costs the bottom-age reading
  for overprinted units (a soil stable across 100 epochs records only the last one).
- **(b) stop rewriting the chapter and derive it from the epoch** — internally consistent by
  construction, and the record's age becomes the *first* deposit for every unit uniformly.
  This is O-5's shape and it moves goldens (the chapter byte changes for overprinted units).

**⚠ Integrator-settleable but NOT free (I-2 below).** Monotonicity (F3) survives either way.

### F5 — The FUNNEL claim is VERIFIED AT SOURCE, including the cross-chapter coalescing

The explainability audit's hole 1 (`2026-08-04-voxel-explainability-audit.md:411-421`) checks
out on both halves:

- **`StrataEvent` has no time field.** `crates/dc-worldgen/src/geology.rs:84-119` — the
  struct is `member`, `thickness_m`, `temp_c`, `precip`, `depth_m`, `sel_salt`, `sel_tag`,
  `ore`, `accessory`, `dither`. No `chapter`, no `mover`, no `unconformity`.
- **The chapter is consumed and discarded.** `geology.rs:755-760` passes `u.chapter()` into
  the `paleo_temperature` provider to fetch a temperature; the index itself never reaches the
  event.
- **Coalescing merges across chapters, and the condition proves it.** `geology.rs:776-782`:
  the merge fires on `prev == member` **alone** — no chapter test, no mover test, no
  unconformity test. Two beds from chapter 1 and chapter 6 that resolve to the same member
  become one `StrataEvent` whose `thickness_m` is their sum. The doc comment at `:716-723`
  argues this is *"lossless for expression"*, which is true and is **not** the same claim as
  lossless for provenance.

### F6 — The unit count is disputed; both numbers are priced below, and the gap has a mechanism

| figure | instrument | date | source |
|---|---|---|---|
| **7,363,947** units | the M0 mover-split counter in the recorder | 2026-08-02 | `recorder.rs:509-516`, journal/0145:9, `2026-08-02-p11-slice3-design.md:60` |
| **10,951,030** units | `examples/member_diversity_probe.rs` | 2026-08-01 | journal/0136:280 |

Both are seed 1337, `Extent::Medium`, 297,025 cells. `ROADMAP.md:2937-2938` carries the
discrepancy; the explainability audit flags it as out of its remit (`:485-487`).

**⚠ HYPOTHESIS, unverified (no cargo) — the two are not contemporaneous, and the mechanism is
in this file.** The 10.95 M figure predates P11 slice 3, which introduced the **sub-quantum
carry**: `deposit_moved` now *returns without minting a unit* when the deposit is below one
quantum of 2⁻¹⁰ m ≈ 0.977 mm (`recorder.rs:912-916`), letting it ride in the per-cell `carry`
until it makes a whole quantum. That converts many thin beds into fewer thick ones **without
changing the mass**, and the arithmetic agrees: 404,898.8 m ÷ 10,951,030 = **0.0370 m** mean
unit at the old count, ÷ 7,363,947 = **0.0550 m** at the new — and the correlation audit's
independently-quoted mean unit is **0.053 m** (`:99`). *"Metre totals roughly agree"* is
exactly what the explainability audit observed (`:301`). **If this is right the discrepancy
is not a defect and 7,363,947 is the current number.** Stated as a hypothesis with its
falsifier: re-run `member_diversity_probe` at today's tip; if it reports ~7.4 M, the two docs
were never in conflict, only out of order. **Every table below prices both.**

### F7 — There is nothing to migrate, because the record is never persisted

`grep -c "Serialize" crates/dc-worldgen/src/deeptime/recorder.rs
crates/dc-worldgen/src/deeptime/field.rs` → **0 and 0**. `DepUnit`, `DeepStrata` and
`DeepField` carry no serde derives; `pregen/mod.rs` serializes `WorldParams`/`CellGrid`-class
types only (`:46, :98, :137, :169`). The record is built by the boot ritual and lives in RAM
for the life of the process (`field.rs:504-505`). **A world is seed + config; the record is
re-derived every boot.** Therefore: **no migration exists, none is owed, and the entire
backfill story is one sentence** — worlds created before the change are not reproducible
under it, the same event class as the erodibility / tectonic-history / full-agents flips
already recorded in `field.rs:285-342`, and already covered by the 2026-07-19 world-identity
rule (corrections #84, cited at `dependency-graph.md:79`).

### F8 — The record fingerprint hashes by ACCESSOR, which is what makes the golden story a choice

`record_fingerprint` (`crates/dc-worldgen/tests/providers_common/mod.rs:625-643`) hashes
field by field through the public accessors — env, aridity, energy, biota, eolian,
`thickness_m()`, `unconformity()`, `chapter()`, species — **not raw bytes**. So a new axis
that no existing accessor exposes is invisible to the golden unless somebody adds a line.
§ 3 prices both branches.

---

## 2. The options

Common facts used by every row:

- **8 B/unit today** (`recorder.rs:760-773` — a compile-time `const _` assertion on
  `size_of::<DepUnit>() == 8`, `align_of == 4`).
- **Unit counts:** A = 7,363,947 · B = 10,951,030 (F6). Bytes below are over `len`; the
  shipped residency figure (58.15 MiB, `recorder.rs:441-443`) is over `Vec` **capacity**
  (`heap_bytes` uses `capacity()`, `:1167-1169`), ~3.5 % above len — apply the same factor to
  any delta.
- **Merge key = a mask over the FIRST word only**: `key_bits() = bits & MERGE_KEY_MASK`
  (`:712-714`, mask at `:517-525`). Anything stored outside `bits` is **structurally** out of
  the key — not out of it by convention.
- **Bit budget in `bits`:** 30 of 32 used, **2 spare** (`:446-459`). The **chapter field is
  8 bits holding 8 chapters** (`grid.rs:582`) — 3 bits used, **5 bits of slack**, kept
  deliberately (*"u8 generality kept"*, `:459`).
- **Max single unit measured on the shipped world: 65.38 m** = 66,949 quanta
  (journal/0141, quoted at `2026-08-03-stratigraphic-correlation-design.md:99`).
- **Every `iterations:` literal in the tree is ≤ 200** (measured by grep over `crates/`:
  200, 80, 40, 30, 24, 7, 1 — plus `DEEP_ITERATIONS` = 200, `field.rs:52`).

### O-1 — the naive baseline: a third field on `DepUnit`

`bits: u32, quanta: u32, epoch: u8` → `size_of` **12** (align 4, 3 bytes tail padding).
Epoch out of the merge key (structurally — `key_bits` masks `bits` only).

| | A = 7.36 M | B = 10.95 M |
|---|---:|---:|
| today (8 B) | 56.18 MiB | 83.55 MiB |
| under O-1 (12 B) | 84.27 MiB | 125.32 MiB |
| **delta** | **+28.09 MiB** | **+41.77 MiB** |

- **Unit count / goldens:** count **bit-identical** (the key is untouched). Golden story per
  § 3.
- **Hot path:** the ALU cost is nil; the real cost is **+50 % memory traffic on the record's
  working set**, which is touched per-epoch by `expose` (`reads_prev: [Recorded]`,
  `runner.rs:765-772`), per-op by `erode`'s pop loop, and whole-stack by `total_quanta()`
  (`:811-816`). ⚠ **Unmeasured, and it is the one number that could sink O-1.** Framed
  honestly: this is a **residency** cost (28–42 MiB resident for the life of the process),
  not a gen-seconds cost, and gen time is not a constraint (CLAUDE.md § Conventions) while
  residency is a runtime-perf number.
- **Kills the pack argument it inherits.** L-8's whole ruling was 116.31 → 58.15 MiB
  (`recorder.rs:441-443`); O-1 gives back half of that saving for one byte of payload.
- **Merge semantics:** keep FIRST epoch (F3) — falls out of `top.quanta += q` with no code.
- **Verdict (assistant-proposed): priced, NOT recommended.** Nothing it buys is unavailable
  at zero bytes. It stays on the board only as the fallback if O-2b's thickness cap is ever
  rejected (§ O-7).

### O-2a — the no-widening layout, chapter-relative (2 spare bits + the chapter's slack)

Store `epoch − chapter_start` in the 2 spare bits plus bits reclaimed from the chapter field.
At the shipped config the within-chapter range is 0..24, so **5 bits** suffice, drawn as
2 spare + 3 from chapter — which **caps chapters at 8** (3 bits), a regression on a `u32`
config field.

- **Storage: +0 B.** Unit count identical (the reclaimed bits stay inside `bits`, so
  `MERGE_KEY_MASK` must be re-derived carefully — see below).
- **It encodes an undeclared constant into the schema, and the corpus already ruled against
  exactly that.** `cadence.rs:38-43`: *"the chapter length is currently an **implicit
  constant** (25 epochs; `flow.md` § 11.1 is the decision that undeclared constants of exactly
  this kind must become declared)"*. The field width `ceil(log2(iterations/chapters))` is
  config-derived: a world with `chapters: 4` has ipc = 50 and needs 6 bits, and the layout
  cannot widen at runtime. **The record schema would silently constrain the world config.**
- **And it perturbs the merge key.** The chapter bits are *in* `MERGE_KEY_MASK` (`:525`);
  splitting the field means the mask changes shape and the within-chapter offset must be
  masked *out* of it — recoverable, but it is the one option where "unit count unchanged" is
  a property to re-prove rather than a structural fact.
- **Verdict (assistant-proposed): priced, NOT recommended.** Same benefit as O-2b, strictly
  more coupling.

### O-2b — THE THICKNESS WORD'S HIGH BYTE (assistant-proposed; **RECOMMENDED**)

The second u32 becomes `(epoch << 24) | quanta`, i.e. **u24 thickness + u8 epoch**.

**The arithmetic that makes it safe, both ends:**

- **Thickness ceiling:** 2²⁴ − 1 = 16,777,215 quanta × 2⁻¹⁰ m = **16,384 m** per unit,
  against a measured max of **65.38 m** — **250× headroom**. (Today's ceiling is 4.19 × 10⁶ m,
  which the struct doc itself calls *"no geology approaches it"*, `:463-466`.)
- **Epoch ceiling:** 0..255 against `DEEP_ITERATIONS` = 200, and every `iterations:` literal
  in the tree ≤ 200. **56 epochs of headroom.**
- **Size and alignment unchanged:** the `const _` pack assertion at `:760-773` holds verbatim.

| | A = 7.36 M | B = 10.95 M |
|---|---:|---:|
| **storage delta** | **+0 B** | **+0 B** |

- **Unit count / merge key: bit-identical BY CONSTRUCTION.** `key_bits()` masks `bits`; the
  epoch is not in `bits`. Nothing about `MERGE_KEY_MASK` changes, so `deposit_moved`'s merge
  condition (`:920-926`) and `overprint_top`'s merge-down (`:993-1000`) are byte-for-byte the
  same decisions on the same inputs. **This is the strongest form of "out of the key": not a
  convention, a structural impossibility.**
- **Merge semantics: FIRST (bottom) epoch, free.** `top.quanta += q` becomes a masked
  read-modify-write of the low 24 bits; the high byte is never touched, so a merged run
  records the epoch it *started*. Per the brief's own construction, one field per unit then
  gives the full interval via stack order: unit *k*'s age interval is
  `[epoch(k), epoch(k+1))`, with the top unit's interval closing at the run end.
- **Hot path:** one OR at construction; one `& 0x00FF_FFFF` on each `thickness_quanta()`
  read; masked read-modify-write in the merge (`:923`), the overprint (`:982`), the
  merge-down (`:998-999`) and `erode`'s decrement (`:1023`); one `debug_assert!` on the
  ceiling at construction. All register-resident integer ops on values already loaded. ⚠ No
  measurement is offered and none is claimed; the *shape* is "a mask on a value already in a
  register", against a path whose per-op cost is dominated by the f64 carry arithmetic
  (`:912-918`) and `Vec::push`.
- **Risk, named:** the mask must be right in **five** places or thicknesses corrupt, and
  `total_quanta()` sums them (`:811-816`), so a missed mask shows up as a mass-closure
  failure — which the finalize invariant already tests (`:804-807`). **The existing gate
  catches this class**, which is the best available answer to "five sites is five chances".
- **What it costs conceptually:** a u32 that meant one thing now means two. Mitigated
  exactly the way L-8 mitigated the first packing: **accessors** (`:476-481` — *"every read
  goes through accessors"*, user-endorsed), so no consumer outside `recorder.rs` learns the
  layout.
- **Verdict: RECOMMENDED (assistant-proposed).** Zero bytes, zero unit-count movement, no
  config coupling, no merge-key surgery, and it leaves the chapter byte exactly as it is —
  so **nothing that reads `chapter()` today changes at all** (24 call sites, measured by
  grep over `crates/`).

### O-3 — derive-don't-store (sidecars), priced honestly and rejected

Two shapes, both worse than O-2b, recorded so the option is genuinely closed.

- **O-3a, a parallel per-cell `Vec<u8>` of epochs, index-aligned with `units`.** Storage:
  A = 7,363,947 B data + 297,025 × 24 B of `Vec` headers = 14,492,547 B = **13.82 MiB**;
  B = **17.24 MiB** (plus capacity slack). *Cheaper than O-1, more expensive than free* — and
  the hot path is the worst of the four: **every** recorder op (append, merge, overprint,
  merge-down, and `erode`'s pop loop) must keep two vectors index-aligned, doubling the
  bounds checks and adding a second cache line to a path that currently touches one.
- **O-3b, a per-epoch watermark sidecar (epoch → stack position).** Dense cost:
  200 × 297,025 × 4 B = 237,620,000 B = **226.6 MiB**. Dead on arrival — but the *correctness*
  objection is the real one and it generalises: **erosion invalidates a watermark, and
  inverting it exactly requires logging the strips.** `erode` pops whole units (`:1021`) and
  truncates partial ones (`:1023`); a watermark recorded at epoch *e* points at a stack index
  that later shrinks, so recovering "which unit was on top at epoch *e*" needs the full pop
  history — **a log of the same order as the units themselves**, at which point you have paid
  more than O-1 to store less than O-2b.
- **Verdict: priced, NOT recommended.** "Derive don't store" is the right instinct and it has
  already been *used correctly elsewhere* — `flux::slot_for_chapter` (`flux.rs:676`,
  docs `:129-133`) derives a stratum slot from the chapter rather than storing it, and its
  own doc explains why deriving is *more honest* there (when erosion strips the chapter's
  unit the derivation returns nothing, where a stored slot would lie). **That argument does
  not transfer**: the slot is derivable because the chapter is stored. Nothing derives the
  epoch, because nothing stores it.

### O-4 — do nothing (R-C / R-C′ as designed) — the control

The correlation design's own statement of the artifact it buys, restated so the comparison is
fair (`2026-08-03-stratigraphic-correlation-design.md:222-231`):

> within a chapter it correlates by **position, not identity** — a storm bed at 40 % of A's
> chapter 5 correlates with whatever sits at 40 % of B's, even when B carries the same
> distinctive bed at 60 %… can be a *doubled* facies transition where matching would have
> found one.

And: *"the record keeps only survivors, so the fraction clock is a **preserved-section**
clock, not true time."*

- **Storage: +0 B. Hot path: +0. Goldens: still. Correlation ships sooner.**
- **What it costs:** the WHEN axis stays at **62.5 Myr** granularity (`grid.rs:224` — 8
  chapters over the ratified history), the funnel stays hole 1 of the explainability audit,
  knowledge.md req 3 stays uncomputable across cells (§ 4), and § 1.2's weakness is bought
  permanently rather than provisionally.
- **Honest note in its favour:** R-C is *correct* — it is the standard construction between
  undated internal surfaces, and geologists live with exactly this error. O-4 is not a
  defect; it is a smaller world.
- **⚠ And it is the right pick if the clock's answer is "not yet":** nothing in O-2b's
  mechanism decays if it lands after the correlation build, because the record is re-derived
  every boot (F7). **Sequencing this before or after S1/S2 is a free choice** — which is
  itself a finding, and it is in § 6.

### O-5 — epoch REPLACES chapter in `bits`; chapter becomes derived (assistant-proposed, priced)

Store the epoch in bits 22–29; make `chapter()` a derived accessor.

- **Storage: +0 B** (or +1.13 MiB if `epochs_per_chapter` is cached per cell rather than
  passed — 297,025 cells × 4 B).
- **It is the most *conceptually* clean option** (F2 says chapter is redundant) and the most
  *mechanically* invasive, in three ways the others avoid:
  1. **The merge key stops being a mask.** Chapter is in `MERGE_KEY_MASK` (`:525`) and merging
     must still refuse to cross a chapter boundary; with the epoch stored, `key_bits()` must
     become `(bits & MASK_WITHOUT) | (chapter_of(epoch) << SHIFT)` — a computed key needing
     `iterations`/`chapters` in scope inside `DeepStrata`, which has no config today.
     **"Unit count unchanged" becomes a claim to prove, not a structure.**
  2. **The non-tectonic path breaks** (F2.2): chapter is pinned at 0 there, so the derivation
     must know `tectonic_history` — config leaking into the record type.
  3. **`chapter()`'s 24 call sites** (measured by grep over `crates/`, incl. 8 probes/examples
     and `flux::slot_for_chapter`) all gain a parameter or lose the accessor.
- It also **encodes a chapterization into storage**, which § 5's partition flag argues
  against.
- **Verdict: priced, NOT recommended** — it buys nothing over O-2b and pays for the insight
  it is built on.

### O-6 — store the RUN LENGTH, not the epoch (assistant-proposed, **rejected on correctness**)

Recorded because the brief's own delta-coding suggestion lands here if taken as a *semantics*
rather than an *encoding*. Store per unit the number of epochs its merged run spanned; absolute
epochs come back by prefix-summing up the stack. Cheaper to bound (a run rarely spans 200).

**It cannot work, and the reason is the point of the whole record.** `erode` pops whole units
(`:1021`) — and the record keeps **survivors only**. Every pop deletes its run length from the
prefix sum, so every unit above it is dated wrong by the removed span, permanently and
invisibly. The record's defining operation is the one that breaks the encoding.

**Generalised rule this yields:** *delta-coding is safe as a **storage encoding** of absolute
epochs (a sidecar stream reconstructed in full), never as the **stored semantics**.* Since
O-2b makes absolute epochs free, the encoding question never arises.

### O-7 — the headroom question, flagged rather than answered

O-2b's clock lives as long as `iterations ≤ 256`. **The arithmetic that decides its lifetime
is worth stating now, because it is asymmetric:** the thickness word can donate **8 bits and
not 16.** A u16/u16 split gives 65,535 quanta = **63.999 m** per unit — and the measured max
unit on the *shipped world today* is **65.38 m**. A 16-bit epoch would overflow on production
immediately. So the escalation path, if a world ever wants > 256 epochs, is **O-1's 12 bytes**
or a **declared coarsening** (`epoch >> shift`, with the shift recorded), never "take another
byte from thickness". ⚠ **Nothing needs deciding today**; recorded so the decision is not
re-derived under pressure.

### The comparison

| | O-1 widen | O-2a chapter-relative | **O-2b high byte** | O-3a sidecar | O-4 nothing | O-5 replace |
|---|---:|---:|---:|---:|---:|---:|
| storage Δ @ 7.36 M | +28.09 MiB | 0 | **0** | +13.82 MiB | 0 | 0 |
| storage Δ @ 10.95 M | +41.77 MiB | 0 | **0** | +17.24 MiB | 0 | 0 |
| unit count identical | yes (by mask) | re-prove | **yes (structural)** | yes | yes | re-prove |
| merge key touched | no | **yes** | **no** | no | no | **yes** |
| hot-path shape | +50 % traffic | mask | **mask** | 2 vectors | — | computed key |
| config coupling | none | **chapter length** | **none** | none | none | **chapters + tec flag** |
| `chapter()` sites moved | 0 | 0 | **0** | 0 | 0 | **24** |
| clock granularity | 1 epoch | 1 epoch | **1 epoch** | 1 epoch | 1 chapter | 1 epoch |

**Recommendation (assistant-proposed): O-2b, with F4's overprint semantics ruled (a) or (b)
before the slice, and the fingerprint extended per § 3.**

---

## 3. Goldens — one branch point, and it is a real choice

F8: `record_fingerprint` hashes by accessor (`tests/providers_common/mod.rs:625-643`), so a
new axis is invisible unless somebody adds `h.byte(u.epoch())`.

- **Branch A — leave the fingerprint alone.** **All 12 golden constants byte-identical**
  (`tests/providers_common/mod.rs:239-497`): 6 `GOLDEN_SURFACE*` + 6 `GOLDEN_RECORD*`
  (`GOLDEN_RECORD`, `_CALIBRATED`, `_UNBOUNDED_CREEP`, `_SCALAR_LOAD`, `_ANONYMOUS_CREEP`,
  `_SINGLE_RECEIVER`). Correlation-design F3's *"every deep-time golden family must be
  STILL"* is satisfied **in full**. **Cost: the new WHEN axis has no tripwire at all** — the
  `flow_cost_probe` shape (an axis the gate structurally cannot see), which this project has
  already been bitten by twice (CLAUDE.md § Gates).
- **Branch B — extend the fingerprint (assistant-proposed; RECOMMENDED).** Exactly the **6
  `GOLDEN_RECORD*` families re-capture, once.** `GOLDEN_SURFACE*` cannot move — terrain never
  reads the record's epoch (`expose` reads outcropping lithology: species/tag, not the
  thickness word's high bits, `runner.rs:765-772`). `GOLDEN_GEOTHERM` / `GOLDEN_FLUX` /
  `GOLDEN_HEAD` cannot move for the same reason. **CONTENTS and SURFACE do not move** —
  expression must not read the epoch (§ 4). This is a **ratified-semantics re-capture** under
  the scratch-pad doctrine (CLAUDE.md § Conventions: *"a hash move produced by ratified
  semantics re-captures the goldens with the why recorded"*) — it owes no byte-identicality
  and no ratification loop on the new bytes, only the recorded why.

⚠ **Reasoned, not measured.** The claim "no terrain hash can move" rests on the epoch being
write-only from the sim's point of view — nothing in the erosion pipeline reads it back. That
is true of the design as specified and would be **falsified by any pass that branches on
epoch**; the build slice owes a one-line assert or a reviewer's grep, not a probe.

⚠ **And the masking is where a real red would come from.** If a `thickness_quanta()` mask is
missed, thicknesses corrupt and `Σ units == H` fails — caught by the existing finalize
invariant (`:804-807`, tested), which is the honest reassurance: **the failure mode is loud
and already gated**, not silent.

---

## 4. What correlation becomes — P-3 and § 1.2, re-read against a real clock

### 4.1 The rule

R-C partitions each stack **by chapter**, then within a chapter by **cumulative thickness
fraction** (`2026-08-03-stratigraphic-correlation-design.md:203-231`). With a per-unit epoch,
the partition becomes an **epoch-interval** partition: unit *k* owns
`[epoch(k), epoch(k+1))` (F3 gives the ordering; the top unit closes at the run end), and two
parents' units correlate **when their epoch intervals overlap**, weighted by overlap.

That is what the anchor asked for: *layers laid down on the same clock are matched by the
clock*, not re-derived.

### 4.2 What survives of R-C — it is demoted, not deleted

**R-C survives as the WITHIN-INTERVAL interpolation rule, which is the one place it was
always defensible.** Two reasons it is still needed:

1. **A unit is a merged run.** The merge key keeps a stable environment's epochs 37..41 in one
   unit (`:920-926`), so an interval of several epochs still has no internal clock. Splitting
   an overlap proportionally inside it is exactly R-C, at 1-epoch granularity instead of
   62.5-Myr granularity.
2. **Coalesced spans** at expression (F5, `geology.rs:776-782`) span wider intervals still.

So the correct reading is: **the chapters were the dated horizons and the fractions filled
between them; now the epochs are the dated horizons and the fractions fill between them.**
Same construction, 25× finer anchors (§ 4.5). Correlation-design § 1.2's geology paragraph
(*"proportional correlation of preserved sections between isochrons is the standard
construction when internal surfaces are undated"*) stands verbatim — the isochrons just got
much closer together.

### 4.3 P-3 (R-C vs R-C′) largely DISSOLVES, and the unconformity flag gets a better job

R-C′ existed to buy **anchors inside a chapter** from the unconformity flag
(`:233-243`). Epoch intervals supply anchors at ≤ 1-epoch granularity for free, so the flag
is no longer needed *as a positional anchor*. **It returns to gap semantics — and for the
first time the gap is a measurable quantity:** unit *k* at epoch 41, unit *k+1* at epoch 88
carrying `unconformity` ⇒ **47 epochs of missing time at this cell**. The record cannot
express that today at any granularity.

⚠ **It does not dissolve completely, and the residue is real.** Epochs alone cannot
distinguish *deposited-then-stripped* from *never deposited* — both read as a gap. The
unconformity flag is precisely the bit that tells them apart (`:781-783, :650-656`,
`:1028-1031`). So **R-C′'s logic migrates from correlation-position to gap-interpretation**:
a flagged contact says the interval was *removed*; an unflagged gap says it was *never laid*.
That is a genuinely richer cross-section and it is a smaller mechanism than R-C′ was.

**⇒ P-3 as posed — "R-C vs R-C′" — becomes largely moot under the clock; the user-owned
question changes shape** (§ 6, P-C).

### 4.4 What is fixed, and what is not

- **FIXED — the position-not-identity artifact.** A storm bed at 40 % of A's chapter 5 no
  longer correlates with whatever sits at 40 % of B's; it correlates with whatever B holds at
  the same epochs. The doubled-facies-transition risk collapses to the residual within-unit
  case (§ 4.2).
- **NOT FIXED — preserved-section is still not true time.** Epochs date what **survives**.
  Two neighbours with different erosion histories still correlate survivor-to-survivor. The
  difference is that they now *know* it: the gaps are visible and quantified (§ 4.3). **The
  clock does not undo erosion; it makes erosion legible.** Strictly better, not complete.
  ⚠ The correlation design's honest weakness paragraph (`:227-231`) should be read as
  *narrowed*, not retired.
- **UNTOUCHED — § 2's mass argument.** F2's identity `Σ_i h(p,i) = Σ_j w_j(p)·H_j` is about
  weights and thicknesses; the partition's *provenance* does not enter it. All five § 2.3
  invariants stand verbatim, and invariant 5 (**seam-free**) gets **stronger**: an epoch
  partition is parent-independent — the interval breakpoints are absolute epoch numbers,
  identical for any chunk that touches those parents, where fractional breakpoints are
  computed from the parents' own thickness sums.
- **UNTOUCHED — § 1.2's rejection of R-A and R-B.** Both stay rejected. Worth recording that
  **the corpus already named what would beat R-B**: its honest-virtue note says index-free
  identity matching is *"how field correlation actually works **when no ash bed dates the
  section**"* (`:199-201`). **This pass is the ash bed.**
- **UNTOUCHED — § 1.3's discontinuity seam.** Correlation still never fails today; the seam
  ships empty with identity default `None`.

### 4.5 The granularity, stated with its caveat

8 chapters over the ratified history at **62.5 Myr per chapter** (`grid.rs:224`); 200 epochs
over 8 chapters ⇒ **1 epoch = 2.5 Myr**. **A 25× finer clock.** ⚠ Neither number is a
ratified real-time mapping — `grid.rs:224` states the chapter length as an
*Earth-orogeny-length* design intent, and the epoch↔years mapping is nowhere ratified. **Use
the ratio (25×), not the Myr figures, in any claim.**

---

## 5. The two threads the clock pays for

### 5.1 The funnel fix (correlation-design I-2, explainability hole 1)

Correlation-design I-2 asks for *"D-2's event-partition carrier (chapter tag on `StrataEvent`
vs. a parallel index)"*. **With an epoch the answer sharpens and gets cheaper:**

- `StrataEvent` gains **`epoch_bottom: u8` + `epoch_top: u8`** — an *interval*, not a chapter
  tag — 2 B on a struct currently ~48 B (`geology.rs:84-119`). An interval is what the
  coalescer forces (below) and what correlation consumes (§ 4.1); a single tag would be wrong
  the moment two units merge.
- **The coalescer must widen, not discard.** `geology.rs:776-782` merges on `prev == member`
  alone; the fix is `e.epoch_top = u.epoch_top` in the merge branch — **one line**, and it
  converts the coalescer from *lossy in time* to **lossless in time** while keeping it lossless
  for expression exactly as its doc claims (`:716-723`).
- ⚠ **It stays lossy in STRUCTURE.** Two beds become one event, so *"were these once
  distinct?"* remains unanswerable — the explainability audit's own last-column entry
  (`:320`). The clock does not fix that and this pass does not claim it does.
- **Expression must not READ the epoch** — only carry it. Any fitness or draw that branched on
  epoch would move CONTENTS/SURFACE and would re-open the *"expression expresses; it stops
  inventing"* ruling (`geology.rs:106-112`). **Stated as a build constraint.**

### 5.2 The explainability WHEN column

The audit's table row for a deep-recorded bed currently reads *"⚠ **in the deep record
only**: `chapter` u8 + stratigraphic order… **Dropped at expression**"* (`:320`). With O-2b +
§ 5.1 it becomes **✅ epoch interval, at the voxel**, and the record's own WHEN goes from a
1-of-8 chapter bucket to a 1-of-200 epoch bucket.

The audit's headline number moves accordingly: a deep-recorded voxel goes from **1 of 5
questions answerable at the voxel to 2 of 5** (what + when). ⚠ **It does not close hole 2**
(the absent query — `dc-api` cannot name the types, `:424-431`) **or hole 3** (the unrecorded
bulk — mean `H` 3.91 m against a 517.4 m column, `:433-448`). It converts hole 1 from
*structurally impossible* to *plumbed*, which is what the audit ranked first as **cheapest and
blocking**.

And it moves one row of the audit's DATA-vs-CODE table (`:452-462`): *bed epoch — data ✅,
code ⚠ one reader that discards it* becomes a genuinely consumed axis.

### 5.3 knowledge.md requirement 3 — the join key

> *"Every natural feature carries process provenance; every process has a signature (geometry
> envelope, material assemblage, tectonic context). 'What explains this?' is a real query;
> **anomaly = no valid provenance match**."* (`docs/design/knowledge.md:25-28`)

**A "match" needs something to match ON.** Two beds in two cells are comparable as evidence of
one process only if you can say they are the same age; a material assemblage that is
*contemporaneous* across a basin is a formation, and the identical assemblage 50 Myr apart is
a coincidence. **Today the finest available join key is the 62.5 Myr chapter bucket** —
coarser than most processes the default pack models, so "no valid provenance match" is not
computable across cells at any useful resolution. **The epoch axis is the join key requirement
3 has been missing**, and it is the reason the anchor's *"pays for two other threads"* is
correct rather than merely convenient.

⚠ **It is necessary, not sufficient.** Requirement 3 also needs the signature libraries, the
query, and the crate-wall decision (explainability audit hole 2). This pass supplies one
prerequisite.

---

## 6. Cadence / Schedule, and the partition

### 6.1 Is "epoch" the right recorded tick? — yes, and RATE's own ruling is the argument

- **The runner's loop counter is the epoch** (`runner.rs:226`), and RATE's base unit is the
  epoch **by ruling**: *"these two numbers are in **epochs**, because the epoch is what the
  runner's loop iterates… the expressible set is strictly larger"* (`cadence.rs:27-43`).
- **Sub-turns are strictly finer and strictly per-pass.** `dt = period / sub_turns`
  (`cadence.rs:20`); a pass at `sub_turns = 5` and a pass at `sub_turns = 1` share **no**
  sub-epoch clock. So a sub-turn index would be **incomparable across passes** — useless as a
  correlation key and actively misleading as a provenance stamp. **The epoch is the finest
  tick the schedule guarantees is common to every pass, which is exactly the property a
  correlation clock needs.**
- **Chapters are the wrong tick for the opposite reason** (too coarse *and* config-derived,
  § 2 O-2a).

### 6.2 Does epoch N mean the same thing in a cadenced world as an uncadenced one?

**For ordering and count: yes.** A `CadenceTable` does not change `cfg.iterations`, does not
change how many times the loop runs, and does not change which epoch index a deposit lands in.
An empty table is today's shipped schedule exactly (`cadence.rs:56-58`), and 200 epochs are 200
comparable ticks under any table.

**For "how much geology happened": no.** Cadence changes how often a pass fires and how much
of its transformation it applies per firing. So epoch N in world A and epoch N in world B are
the same *position in the run* and not the same *amount of process*.

**⇒ Within one world — the only scope correlation needs — the epoch is a schedule-invariant
tick and the answer is clean.** ⚠ Any future **cross-world** claim ("this bed and that bed in
another world are contemporaneous") is **not** supported and should be refused; nothing here
needs one.

### 6.3 Engine or pack? — FLAGGED, not resolved

`dependency-graph.md:79` already rules the shape for P11: *"Spans the partition on purpose —
the **record** is E2 storage, the **identities** are pack content."* Applied here:

- **The FIELD — a tick stamped on the packed unit — is E2 storage, engine.** Same class as
  the species byte and the chapter byte beside it.
- **The CLOCK'S MEANING is authored.** `iterations` and `chapters` are `DeepConfig` fields
  (`field.rs:52`, `grid.rs:226`), cadence is authored data (`CadenceTable`, E3 **BUILT**,
  `dependency-graph.md:49`), and pass ORDER is authored per world (CLAUDE.md read-first 0).
  How long an epoch is, whether chapters exist, and what a tick is *in years* are all
  world/pack calls.
- **The design consequence, and it is the reason this belongs in the ratification list:**
  **the record must store the RAW TICK and never a derived time**, or the engine bakes a
  pack's calendar into E2 storage. That argument is *also* an independent argument against
  O-2a and O-5, both of which encode a **chapterization** — a config-derived, authored
  structure — into the storage layout.
- ⚠ **This is the same shape as the E6 open-vocabulary question and this pass does not
  resolve it** (§ 7, P-D).

---

## 7. NEEDS RATIFICATION

### 7a. User-owned

| # | question | the shape of the trade |
|---|---|---|
| **P-A** | **Restore the clock at all — O-2b vs O-4 (do nothing)** | O-2b is **zero bytes, zero unit-count movement, no config coupling**; it costs a masked thickness word (5 sites, failure mode caught by an existing invariant) and one golden re-capture family (§ 3 branch B). O-4 keeps the 62.5 Myr bucket, leaves explainability hole 1 open and knowledge.md req 3 uncomputable. **This is the pass's headline pick.** |
| **P-B** | **The storage shape if P-A is yes: O-2b (high byte) vs O-1 (12 B)** | O-1 costs **+28.09 MiB @ 7.36 M units / +41.77 MiB @ 10.95 M** and gives back half of L-8's ratified saving for one byte of payload; O-2b costs a 16,384 m per-unit thickness ceiling (250× the measured max) and an epoch ceiling of 256 (`DEEP_ITERATIONS` = 200). **Recommended: O-2b.** A user call because it re-opens a packing decision the user ruled six days ago (L-8). |
| **P-C** | **P-3 IS RE-SHAPED — the question is no longer "R-C vs R-C′"** | Under a clock, R-C′'s anchors are free and its *logic* migrates to **gap interpretation** (§ 4.3). The live question becomes: *does a flagged unconformity's measurable gap get expressed* (a truncation surface the eye can read, sized by the missing epochs) *or only recorded?* ⚠ Correlation-design § 9's P-3 should be **superseded, not answered** — and its author owes it a banner (read-first item 5). |
| **P-D** | **The partition flag (§ 6.3): the record stores the RAW TICK and never a derived time** | Recorded as a proposed constraint, not applied. It is what stops the engine baking a pack's calendar; it is also what rules out O-2a/O-5 on principle rather than on cost. Same class as E6. |
| **P-E** | **Sequencing: before, after, or independent of correlation S1/S2** | ⚠ **This is a genuinely free choice and that is a finding.** Because the record is re-derived every boot (F7) there is no migration and no ordering constraint. Before S1 means correlation is built once against the real clock; after S2 means correlation ships sooner and is then *simplified*. **No recommendation offered — it is a velocity call.** |

### 7b. Integrator-settleable (recorded when done)

| # | item |
|---|---|
| **I-1** | **F4's overprint semantics**: (a) rewrite the epoch alongside the chapter (last-alteration age, matches today's chapter behaviour) or (b) stop rewriting the chapter and derive it (first-deposit age, uniform). Monotonicity survives either way; pick one and write it in the accessor's doc. |
| **I-2** | The `Erosion::set_epoch` plumbing (F1) — **unconditional, driven from the epoch loop, never from `tectonics_pass`**, which is not registered on the legacy path (`runner.rs:729`). |
| **I-3** | § 3's golden branch: extend `record_fingerprint` (branch B, recommended) and re-capture the **6 `GOLDEN_RECORD*`** families once with the why, or leave it and accept an untripwired axis. |
| **I-4** | The `debug_assert!` that converts F3 from an argument into a fact (`epoch` non-decreasing up-stack) — **it discharges correlation-design § 10.4's owed chapter assert at the same time**. |
| **I-5** | `StrataEvent`'s `epoch_bottom`/`epoch_top` + the one-line coalescer widening (§ 5.1), with a test that expression **never branches** on the epoch. |
| **I-6** | Resolve F6 by re-running `member_diversity_probe` at tip (the sub-quantum-carry hypothesis has a one-command falsifier) and **stamp both** journal/0136 and `2026-08-03-stratigraphic-correlation-design.md:98` — plus close `ROADMAP.md:2937-2938`. |
| **I-7** | Stamp targets in the same commits: the correlation design's header (P-3 re-shaped, § 1.2's weakness narrowed, § 10.7's *"true per-epoch bed ages are not in the 8-byte record"* answered), the explainability audit's header (hole 1's WHEN row), `dependency-graph.md`'s P11 row, ROADMAP. |

---

## 8. What this pass could not verify — flagged, not smoothed

1. **No cargo was run.** No cost figure here is measured. The hot-path claims are **shape**
   arguments ("a mask on a register-resident value", "+50 % memory traffic on a 56–84 MiB
   working set"), not numbers. **The one number that could change a verdict** — whether O-1's
   residency delta matters at runtime — is unmeasured, and it is the reason O-1 is priced
   rather than dismissed.
2. **F6's unit-count discrepancy is resolved only by HYPOTHESIS.** The sub-quantum-carry
   mechanism fits the arithmetic (0.0370 → 0.0550 m mean unit at constant mass, against an
   independently-quoted 0.053 m) but was not run. **Both counts are priced everywhere.** I-6.
3. **F3's monotonicity is argued from five writers and seven call sites, not proven.** Same
   epistemic status as correlation-design F1's chapter claim, and the same one-line fix.
4. **§ 3's "no terrain hash can move" is reasoned, not measured** — it rests on the epoch
   being write-only to the sim. Falsified by any pass that branches on epoch; nothing does
   today, and § 5.1 states it as a build constraint rather than assuming it.
5. **The 2.5 Myr/epoch figure is derived from an unratified mapping** (`grid.rs:224`'s
   62.5 Myr chapter). **Use the 25× ratio, not the absolute.**
6. **Chapter-length uniformity is read from `blend_into`'s arithmetic, not measured** — the
   floor/min in `mod.rs:491` means the LAST chapter absorbs any remainder when
   `iterations % chapters ≠ 0`. At production (200/8 = 25) it is exact; at
   `iterations: 30, chapters: 8` it is not. Irrelevant to O-2b (which stores the raw tick) and
   fatal to O-2a (whose field width assumes a uniform chapter).
7. **⚠ CONTESTS discipline check — nothing here contradicts the anchor, and one thing
   *corrects* the answer that was relayed with it, in the anchor's favour.** The relayed
   answer said the per-epoch ages *"were dropped at packing"*. They were not: the recorder was
   **never given the epoch** (`recorder.rs:901-908` — it is not an argument), and the packing
   is where the *unspent bits* are. **The correction makes the fix cheaper, not harder**, and
   it is recorded rather than silently smoothed because the anchor's *"did we throw the time
   away?"* deserves a precise answer: **yes, at the call site, and the bits to undo it were
   never spent.**
8. **This pass does not re-open P-1 or P-2** (both RULED 2026-08-04, banners in the
   correlation design's header). Nothing in § 4 touches the identity question (M-C) or the
   acceptance criterion — the clock changes *which* beds correlate, never *how* their
   identities are expressed.

---

*Author: read-only design-pass agent, 2026-08-04, read at `0ac5b7f`. Read-only except this
file; no cargo invoked. O-2b, O-3's rejection, O-5, O-6, O-7, § 4's demotion-not-deletion
reading, § 5.3's join-key argument and § 6.3's raw-tick constraint are assistant-originated
analysis, marked where they go beyond arithmetic over cited measurements. The anchor is the
user's; this file executes it.*
