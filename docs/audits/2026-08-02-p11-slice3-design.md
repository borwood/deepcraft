# P11 slice 3 — the near-path restructure + the packed `DepUnit` co-rider: design pass

**Arc anchor:** `ROADMAP.md:359-496` § Sequenced *"MEMBERS INTO DEEP HISTORY"*, build item 3
(ruling 5, folded 2026-08-01) + the U5 co-rider (RULED 2026-08-02, user — recorded in
`docs/design/materials.md:242-253`). Graph rows: `docs/dependency-graph.md:79` (P11),
`:51` (E5, MM-3's origin), `:72` (P4, the near-path half-shipped state).

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option is priced
and its trade-offs named; picks marked **user-owned** are the user's. Anything the agent
originated is marked **(assistant-proposed)**. **No cargo was invoked** — every number is
arithmetic over already-measured quantities cited to source, or a hand layout computation
flagged as such.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (verified in code here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / stale input /
needs the user's eye).

> **P-2 RULED 2026-08-02 (user): L-8.** *"I theoretically agree with L-8… if it's cheap
> to replace 8, may as well start at 8."* The packed `DepUnit` is **8 B/unit** (u32
> bitfield + u32 fixed-point thickness; record 116.31 → 58.15 MiB). The ruling's load-
> bearing premise is § 2.2's accessor conversion — every read goes through accessors, so
> a future widening is accessors + one golden re-capture, not ~141 raw sites — and the
> user endorsed that as a repo-wide pattern: *"I'm glad we're converting to accessor
> calls right now and I hope we follow that pattern where possible in this repo."*
> Recorded expectation, not a commitment: an upgrade may come *"one day, who knows how
> soon"* — eco/civ storage (ON HOLD domain) is the named unknown; the ruling-3 hint byte
> stays UNFORECLOSED as exactly such a widening. L-12 was declined, not refuted.
>
> **P-3 + MARCH ORDER 2026-08-02 (user): "Seriously whatever's easiest because FS-A is
> shipping today and this is an in-progress slice. Eyes on the prize."** P-3 = O-2
> (bits named and UNSET — also the easiest). The remaining picks take this audit's
> integrator defaults under that directive, NEEDS-RATIFICATION-flagged where user-owned:
> P-4 measure-M0-first (M-1 iff the mover split ≲1.1×, else M-2 with the number reported
> loudly) · P-5 veneer OUT (stub #31's heir re-pointed honestly; flagged) · P-6/P-7 ride
> the acceptance walk (the walk IS the ratification) · P-8 sequencing as proposed
> (3a cheap-evidence → 3b with the full gate clearing the batch debt). Build dispatched
> same hour; FS-A dispatched behind 3b.
>
> **U4 RULED 2026-08-02 (user, "I agree: 5"):** five φ classes — the loose ladder's
> natural rungs (scree / gravel / sand / silt / clay). Grain reserves **3 bits** in the
> packed u32 (§ 2.2's U4=5 row); widening to 8 later is a semantic re-capture inside the
> same bits, not a layout surgery (§ 3 reading (b)). The user flagged, explicitly *not*
> bearing on this ruling: the **presentation layer for grades is an open question** (how
> does the player know which grade they're looking at — and the broader unwritten one,
> loose vs structure legibility in general); user has thoughts; filed ROADMAP § Observed
> 2026-08-02.

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Read at commit
`665167e`. Anything that later refutes or re-scopes this file gets a banner **here**,
stamped by the author of the correction.

**Read with:** `docs/audits/2026-08-01-members-into-history-design.md` (rulings 5–6 +
§ 3.1/§ 7 R4) · `docs/audits/2026-08-01-p10-grain-axis-design.md` (F4's 8-byte quantum,
§ 2.1's φ arithmetic, U4/U5) · `docs/audits/2026-07-29-member0-coarsefield-design.md`
(§ 2b/§ 4 MM-1…MM-6, § 6 law checks) · `docs/audits/2026-07-29-fluvial-record-terms-priors.md`
header (ruling 3, the hint-byte non-foreclosure) · journal/0129 (the shipped 28.8 m half +
the measured blast radius) · journal/0141 (slice 2's counts) · `docs/design/stubs.md`
#21/#25/#31/#37 · CLAUDE.md § Gates (probe doctrine, batching rule).

---

## 0. The findings that reorganise the slice

**F1 — the field-report mechanism VERIFIES in code, with one citation correction.**
The brief cited `collapse.rs:1459`; that line is the chunk-centre **provenance** lookup.
The record read is `collapse.rs:1483-1487`: `column()` calls
`record_at_voxel(cx*32+16, cz*32+16)` — the same chunk-centre address — and
`field.rs:836-844` resolves it **NEAREST** at the ~460 m deep-cell grid (its own doc
comment says *"FLAGGED sampling choice"*). `run_strata` then runs **once per chunk**
(`collapse.rs:1517`) and the one resulting `StrataRec` skins all 1024 columns
(`ColumnRec` at `:1546-1553`). So: recorded-mix composition is **uniform per chunk**
(journal/0129 measured it directly — 0 of 56 adjacent chunk pairs differ at the U3 pose),
and member-**presence** borders can only fall where the chunk-centre's nearest cell flips —
straight runs along deep-cell edges, staircase-quantized to the 28.8 m chunk pitch.
**The integrator's desk confirmation is correct.** Two honest caveats: (a) the field
report's *specific* line was never grid-alignment-checked in game, so "this mechanism
exists and produces exactly this signature" is proven while "this mechanism is what the
user saw" is inferred — the acceptance walk should do the alignment check at station 1;
(b) ⚠ the field-report entry (`ROADMAP.md:2753-2768`) records **no pose** — corrections
#48's exact gap. The tour map must find its own station.

**F2 — the three NEAREST reads are MASS-COUPLED, and the dither must move them as one.**
`field.rs:806-815` (doc comment on `regolith_at_voxel`): `H` is *exactly* the sum of the
same cell's record unit thicknesses (journal/0053's finalize invariant), the collapse
tier's surficial veneer is the **difference** of the two, and *"that subtraction only
conserves mass if both terms name the same cell. Interpolating one and not the other would
leak or invent loose material at every cell boundary."* `ledger_at_voxel`
(`field.rs:870-887`) is index-parallel to `strata` and is sampled at the same cell for the
weathering fold (`collapse.rs:1488-1497`), whose bedrock slot index is `deep_units.len()`
— a per-record quantity. **Therefore the per-column membership dither picks ONE cell per
column and that cell supplies record + regolith `H` + ledger together.** A dither applied
to the record alone is a Law-3 leak at every boundary column, silently, at every frontier
on the map. This is the member-#0 design pass's own § 6 warning
(*"do NOT fix `regolith_at_voxel` NEAREST→bilinear here"*) arriving as a positive
requirement. It is the single most important structural constraint on the slice.

**F3 — the blast radius MOVED since it was measured: 13 files / ~40 sites → 14 files /
~71 sites.** Re-measured in this worktree (`git grep -c -E "(col|rec|ce|c)\.strata"`,
column-level reads): `collapse.rs` 13 · tests `contents_contract` 1, `deeptime_integration`
1, `distribution_fill` 4, `geology` 6, `organic` 4 · examples `appearance_tour_p11` **9**,
`weathering_profile_probe` 16, `soil_depth_probe` 4, `surface_dither_probe` 3,
`palette_quant_tour` 3, `coal_charcoal_probe` 3, `organic_probe` 2,
`pore_decorrelation_probe` 2. The growth is real, not noise: **`appearance_tour_p11.rs`
(journal/0143, added 2026-08-02, gated) did not exist when the 13-file number was taken**,
and it reads `col.strata` as *the* chunk record in three passes. journal/0129's own lesson
(*"grep the field, not the function"*) has already gone stale once more in three days.
Pattern-based, so ±a few; two more files (`coal_walk_tour`, `contents_air_over_solid_probe`)
call `column_record` but read only `heights` and survive untouched.

**F4 — the packed layout arithmetic closes at 8 B/unit with room to spare, and the hint
byte is the swing vote between 8 and 12.** Hand layout computation (flagged as such, § 2).
Bit budget: `DepTag` 9 bits (env 1 + aridity 1 + energy 2 + biota 3 + eolian 2, from the
enum inhabitant counts at `recorder.rs:229-347`) + unconformity 1 + species 6 (stub #21's
51-material ceiling; 26 today needs 5) + mover 3 (stub #25: `FlowCause`'s 7) + grain 1–3
(U4) + chapter 3–8 = **23–30 bits → one u32 bitfield**, plus u32 fixed-point thickness =
**8 B**. At slice 2's measured **7,622,541 units** (journal/0141; priors header:
3,998,428 → 7,622,541, 1.9064×), the record goes **116.31 → 58.15 MiB (−58.15)** at 8 B,
or **87.23 MiB (−29.08)** at 12 B with a full spare byte for ruling 3's per-unit hint.
Either way the co-rider is a net residency **reduction** while adding two axes — the U5
ruling's premise, confirmed by arithmetic.

**F5 — the "measured grain split factor" gate cannot bind at slice 3, because slice 3's
grain bits cannot split anything.** Until FS-A/P10 writes a grain state that *varies within
an identity*, any grain value is a function of the recorded `MaterialId` (its sheet's φ,
quantized) — a function of a field **already in the merge key** adds zero splits, split
factor exactly 1.0 by construction. **The gate the U5 ruling names actually gates FS-A/P10's
writers**, and slice 3's obligations are: reserve the bits, ship the split-factor
*instrument*, and assert 1.0 (§ 2.4). The measurable-today analogue is the **mover** split
factor (the agent axis CAN vary within the existing key — that is its whole point), and it
must be measured **before** the mover joins the merge key, or corrections #88 repeats
verbatim (§ 2.5).

**F6 — ⚠ USER-ATTENTION (CONTESTS-adjacent, surfaced not reconciled): the near-path dither
carries the same cell-wide-blend semantics the user REJECTED BY EYE at the far tier.**
MM-1's membership dither realizes bilinear weights: the home cell wins **9/16** of its own
cell's positions, so **~44 % of columns everywhere read a neighbouring cell's record** —
not only near a frontier (journal/0125's measured consequence; the law test shipped with
MM-1, journal/0129). At the far tier this exact property produced *"the whole cake is
swirled now"* (user, 2026-07-29) and rides as rejected-interim with the far register as
heir. Ruling 5 (user, 2026-08-01) folded MM-3 into P11 knowing MM-1's shape, so this is
not a contradiction — but the near tier is where the user's *eye* lives, and the walk
verdict should be taken knowing that "fixed" looks like a **gradational, interfingered
contact spread across up to a full ~460 m cell**, not a sharper line in a better place.
If the user rejects that reading too, the alternative (a perimeter-weighted dither) is a
**new mechanism beside MM-1** (anti-shape A-1) and a deviation to ratify loudly — priced
in § 9 as a named exit, not taken.

---

## 1. The restructure (ruling 5 — already ruled; this section is shape and price, not a pick)

### 1.1 What lands (BUILT machinery it composes from, DECIDED shape)

Per the member-#0 design pass § 2b (its reshaped adoption) and ruling 5:

1. Per chunk, enumerate the deep cells the 2×2 bilinear stencil touches — **always 4 in a
   cell interior, up to 9 when the chunk footprint straddles cell edges, ~1 only within
   half a metre of a cell-centre line** (journal/0129's corrected arithmetic; the design
   pass's "typically 1" was the same misreading that cost journal/0125 a failing gate).
2. `run_strata` once per touched cell → `ColumnRec.strata: StrataRec` becomes
   `records: Vec<StrataRec>` (+ a per-column index). The expensive pass stays per-cell;
   only the cheap index is per-voxel-column — the same sentence the `dithered_member`
   hoist already implements one joint over (journal/0129).
3. Per voxel column, `CoarseField::sample_source_cell` (MM-1, **BUILT**, with its 9/16 law
   test) picks which record skins the column, through a **registered draw domain**
   (`draw_domains!` — the salt lesson of journal/0141 § *the salt that was two decisions*;
   do not hand-roll). Source: `Octaves` (the unbiased coherent source, journal/0128) —
   **(assistant-proposed)** for consistency with the near member dither; `Coherent` would
   re-import corrections #39's majority amplification at a new joint.
4. **MM-3's `SubCell` type is written WITH its consumer** — the condition journal/0129 set
   when it withdrew the type. The member-#0 report's judgment calls stand for the
   ratification conversation: the type owns the height field and `ColumnFill` travels
   inside it, *"so slicing record A with fill B is a type error."* Its consumer now exists
   twice over: the restructure itself, and the member-grade record it lands on.
5. **The F2 coupling, as a type-level guarantee (assistant-proposed):** the per-column pick
   returns a *bundle* — `(StrataRec, regolith H, LedgerView)` of the SAME cell — so the
   veneer subtraction and the weathering fold structurally cannot mix parents. This is the
   cheapest place the Law-3 requirement can be made unbreakable rather than remembered.

### 1.2 What re-derives inside `column()`

Today's per-chunk quantities and their dispositions (`collapse.rs:1397-1556`):

| quantity | today | under the restructure |
|---|---|---|
| `deep_units` (record) | chunk-centre nearest cell | **per-column dithered cell** (the slice) |
| `regolith_m` → `soil` | chunk-centre, one value/chunk | **per-column, from the dithered cell** (F2) |
| ledger / `deep_weathering_m` | chunk-centre | **per-column, same cell** (F2; its bedrock slot is `deep_units.len()` of *that* record) |
| `temp_c` / `precip` (climate) | chunk-centre | ⚠ **open scope call D-6** (§ 1.4) |
| `elev_m` (mean height), `flow_energy`, `provenance` | chunk aggregates | unchanged — chunk-scale by design |
| heights / `surface_eighths` | per-column bilinear `surf` | unchanged (elevation is not record-derived) |

### 1.3 Cost — runtime, and runtime is sacred

`column()` is the **chunk-load path** (cached per chunk in `column_cache`; `dc-client`
generates chunks at runtime). The cost prior stands: **~4× `run_strata` per chunk**
(the stencil is always 2×2), up to 9× straddling. What is NOT known is `run_strata`'s
share of chunk generation — unmeasured, and the honest plan is to measure it **before**
the slice, not after:

- **Pre-slice (cheap, one probe extension):** time `run_strata` inside an existing
  chunk-generating probe (`surface_dither_probe` builds columns already) — its share of
  `column()` wall clock decides whether 4× is noise or a fight.
- **The slice's RETURN spec** (CLAUDE.md § runtime perf): measured before/after on
  `contents_contract` — **240 `generate_chunk_with_materials` calls, baseline 102.43 s**
  (journal/0129's own instrument and number), same machine, uncontended.
- **Memory:** `ColumnRec` grows ~4× on its `strata` payload (records for 4 cells instead
  of 1). `ColumnRec` heap size is unmeasured — a probe line (heap bytes per `ColumnRec`,
  before/after) belongs in the RETURN spec because `column_cache` is runtime-resident.
- **Mitigations if 4× bites (priced, none picked):** (i) share `run_strata` output across
  the chunks that touch the same cell via a small cell-keyed cache — legal only for the
  cell-dependent part of `StrataCtx` (the chunk-dependent context — climate, elev,
  flow_energy — currently feeds the same call, so this requires splitting the ctx; real
  work); (ii) accept 4× if the pre-slice measurement says `run_strata` is a small share;
  (iii) lazy per-cell records built on first column touch (saves ~nothing in the interior
  where all 4 are touched — journal/0129: even weight 0.001 is realised somewhere in 1024
  columns).

### 1.4 D-6 — ⚠ the veneer's formation context: in or out (scope call, user-adjacent)

Stub #31 (narrowed 2026-08-01) survives for the **veneer**: `column()` samples
`temp_c`/`precip` once per chunk and the year-zero veneer passes select members from that
frozen sample at a 28.8 m step (`stubs.md:1409-1415`). The stub's own heir line names
*"per-column formation context, naturally part of the near-path record restructure"* —
i.e. **this slice** — and the 2026-08-02 staleness sweep's F4 filed *"veneer heir =
slice 3"* (ROADMAP § Observed OWED-SMALL). Doing it means per-column `climate_at` (1024
bilinear samples per chunk instead of 1 — cheap arithmetic, but on the hot path; measure).
Not doing it means stub #31 stays live **and its heir pointer goes stale a second time**
— the banner then owes a new heir. **In-or-out is a scope call the main session should
put to the user with the measured cost**; this audit only prices both sides. A step in
fitness *thresholds* is a far weaker signal than the retired step in the draw
(stub #31's own text), so "out, with an honest re-pointed heir" is defensible.

### 1.5 The consumer migration (the 14 files)

The dominant idiom is `ColumnFill::build(&col.strata, VOXEL_M)` + `col.strata.events[k]`.
Under the restructure "the chunk's record" stops existing; every consumer must name a
column. **(assistant-proposed)** migration shape: `ColumnRec::record_for(lx, lz) ->
&SubCell` (or the `SubCell` carries fill + events + heights slice), so probe/test sites
convert mechanically to `col.record_for(x, z).strata`. Sites that iterated "the chunk's
events" as a census (e.g. `deeptime_integration.rs:138`) must decide per-column vs
any-column semantics — ~5 judgment sites out of ~71; the rest are renames. **Existence
check (CLAUDE.md § existence is not standing):** all 14 files are production
(`collapse.rs`), gate tests, or measurement instruments — nothing unratified found riding
the path; nothing to challenge for removal.

---

## 2. The packed `DepUnit` co-rider (U5 — RULED 2026-08-02, user; this section prices the layout, which is not yet picked)

### 2.1 The current 16 bytes, verified

`recorder.rs:424-475` (**BUILT**, compile-asserted since slice 1): `tag: DepTag` 5 ×
1-byte enums + `thickness_m: f64` + `unconformity: bool` + `chapter: u8` +
`species: MaterialId` = 16 B, zero padding, align 8. Not serialized (no `Serialize` on
`DepUnit`/`DeepStrata` — re-verified at `recorder.rs:424`; wire discipline is therefore
not in play, and there is **no on-disk world to migrate** — "one migration" = one golden
re-capture + probe caption updates).

### 2.2 The bit budget (hand layout computation — NOT compiler-verified here)

| field | inhabitants | bits | source |
|---|---|---:|---|
| env | 2 | 1 | `recorder.rs:230` |
| aridity | 2 | 1 | `:241` |
| energy | 3 | 2 | `:250` |
| biota | 6 | 3 | `:280` |
| eolian | 3 | 2 | `:337` |
| unconformity | 2 | 1 | `:432` |
| species | 26 now / **51 ceiling** | **6** | stub #21 (`inventory.rs:250-255`) |
| **mover (agent axis, NEW)** | 7 (`FlowCause`) | **3** | stub #25 `:924-926`, narrowed note `:944-948` |
| **grain (NEW)** | U4: 8/5/4/2 | **3/3/2/1** | U4, § 3 |
| chapter | 8 today; config-K | 3 (today) … 8 (keep u8 generality) | `recorder.rs:433-443` |

Totals: **23 bits** (U4=2, chapter@3) … **30 bits** (U4=8, chapter@8). **One u32 bitfield
holds every combination**, with 2–9 spare bits. Every read site becomes an accessor call
(`u.species()`, `u.thickness_m()`); `.thickness_m` alone appears at ≤141 sites across 32
files (pattern count; includes the collapse tier's *distinct* `StrataEvent::thickness_m`
f32, which is NOT touched — the surgery is deep-tier only). Mechanical, wide, mostly
find-and-replace-with-parentheses.

### 2.3 Thickness: fixed-point, and the Law-3 carry

Mean unit thickness on the shipped world: 404,202 m of record / 7,622,541 units ≈
**0.053 m** (both from journal/0141; hand division). Quantum options, both power-of-two
so f64↔fixed conversion is exact on dyadics:

| quantum q | u32 max | mean unit in quanta | note |
|---|---:|---:|---|
| 2⁻¹⁰ m ≈ 0.977 mm | 4,194,304 m | ~54 | no cap concern at any geology |
| 2⁻¹³ m ≈ 0.122 mm | 524,288 m | ~434 | finer; still uncappable in practice |

⚠ The **max** unit thickness is unmeasured (owed: a one-line histogram from any probe that
walks the record — rides the pre-slice measurement run, § 7). u16 thickness (a 6 B unit)
caps at 64 m @2⁻¹⁰ and is **not recommended** without that measurement.

**The remainder carry (the ruling's gate (b)).** The record is *working state* — the
outcrop window reads `thickness_m` back every epoch — so quantization enters the sim loop.
The recorder keeps one sub-quantum f64 remainder per cell (n × 8 B = **2.27 MiB**
gen-transient); every deposit/strip quantizes through it. Invariants this makes derivable
(§ 4.2): |carry| ≤ q per cell at all times (scale-free), and record + carry closes against
the delivered budget **exactly** — the Law-3 bound stops being float-accumulation-shaped
and becomes *quantum × merge count*, which is the ruling's stated condition. Bonus the
materials.md ruling text already names: fixed-point addition is exact and commutative, so
**corrections #89's hazard class (segmentation moving terrain through float accumulation
order) retires for the record's own sums.**

### 2.4 Grain bits at slice 3 — reserved, not modelled (F5)

Two shapes, one recommendation:

- **O-1 — write identity-derived grain** (φ of the recorded material's sheet, quantized to
  the U4 ladder): split factor 1.0 by construction, but the stored value is a pure function
  of `species` — *a constant wearing state's clothes*, and a consumer reading it where the
  sheet is available is the summary-is-not-authority shape.
- **O-2 — bits named and UNSET** (the `Identity::Unrecorded` shape, journal/0101): a loud
  "no grain recorded here" until FS-A/P10 writes real state. **(assistant-proposed:
  recommend O-2.)** Funding = the space exists; the U5 gate then binds exactly where it can
  bind — on FS-A/P10's writers, with the split-factor instrument (a count-model probe:
  units with vs without the grain axis in the key) shipped by this slice and asserting 1.0
  until a writer exists.

Either way the gate assert is the same and scale-free: **enabling the grain axis leaves the
unit count identical** — the #88 lesson as a tripwire rather than a memory.

### 2.5 The mover / agent axis — real day one, with its own split question

`arriving_material` (`erosion/record.rs`, stub #25's narrowed note) already computes the
per-mover sums; writing the winning mover into 3 bits is cheap. The open design fork:

- **M-1 — mover joins the merge key**: honest (a colluvial and an alluvial bed of the same
  rock are two units — stub #25 says they are *genuinely different rocks to a geologist*),
  but it multiplies units by an **unmeasured mover split factor**. Measurable TODAY, before
  the slice: count consecutive same-key deposition runs whose dominant mover differs — one
  probe run under the existing record machinery. **Do not take M-1 without this number
  (the #88 rule, applied by this audit to its own co-rider).**
- **M-2 — mover recorded, NOT in the key** (dominant mover at last merge): zero split, but
  the value is an argmax over a merged history — mushier, and a merge must define a
  combine rule (last-writer? thickness-majority?). Half of stub #25 rather than all of it.

**(assistant-proposed)** sequencing: measure first (§ 7 M0); pick M-1 if the factor is
small (say ≲1.1×, against the −58 MiB the pack banks), else present M-2 to the user with
the number. The pick itself is integrator-settleable *once measured*, user-owned if the
number forces a trade.

### 2.6 The layouts, priced (7,622,541 units; hand arithmetic)

| layout | contents | size/unit | record MiB | Δ vs today (116.31) | hint byte? |
|---|---|---:|---:|---:|---|
| today | f64 + 5×u8 + bool + u8 + u8 | 16 | 116.31 | — | no (would step to 24) |
| naive byte-append (refused by U5) | +1 B | 24 | 174.46 | **+58.15** | the step the ruling forbids |
| **L-8** | u32 bits + u32 thickness | **8** | **58.15** | **−58.15** | only as spare *bits*, U4-dependent (§ 5) |
| L-10 | u32 bits + u32 thickness + u16 (hint+spare) | 10¹ | 72.69 | −43.62 | ¹align 4 → rounds to 12 in practice |
| **L-12** | u32 bits + u32 thickness + hint u8 + 3 spare B | **12** | **87.23** | **−29.08** | **yes, a full byte + 3 spare** |

Arithmetic: 7,622,541 × 8 = 60,980,328 B = 58.15 MiB; × 12 = 91,470,492 B = 87.23 MiB.
All figures inherit the unit count's staleness caveat (it moves with every deep-sim
semantic change; it was re-measured 2026-08-02 and is the freshest count in the corpus).
**L-8 vs L-12 is a user-owned pick** (§ 9): it is 29 MiB against the hint byte and slack
for the next axis nobody has named yet.

---

## 3. U4 — how many φ classes (user-owned, OPEN; presented, not picked)

Re-derived against the packed layout. The P10 audit's physical bound (its § 2.1, cited
arithmetic): a full-width traverse of the 250.7 km world at Sternberg α = 0.015 km⁻¹ buys
**≈ 5.4 φ halvings** — the world can *make* about five classes of downstream change.

| U4 | grain bits | bitfield total (chapter@4) | spare bits in L-8's u32 | hint byte fits in L-8's spare bits? | what it buys / loses |
|---:|---:|---:|---:|---|---|
| **8** | 3 | 27 | 5 | ✗ (share-nibble only, 4 bits) | full Wentworth headroom; generous vs the 5.4-halving bound |
| **5** | 3 | 27 | 5 | ✗ | the F1 ladder's natural 5 rungs (scree/gravel/sand/silt/clay); same bit cost as 8 |
| **4** | 2 | 26 | 6 | ✗ | spans the traverse bound tightly; loses one tail class |
| **2** | 1 | 25 | 7 | ✗ (needs 8) | coarse/fine only — "discards most of what the world can make" (P10 § 2.1) |

Readings the user should have in hand: (a) **at the record, U4 is residency-free across
all four options** — every ladder fits the same u32; the discriminating costs live in
P10's transport planes and settle tables (P10 § 3.2's ×K table), *not* here; (b) 5 and 8
cost identical bits, so the honest fork is 8-vs-4-vs-2, with 5 as a labelling choice
inside 3 bits; (c) under **L-12** the hint byte is unconditionally funded and U4 stops
interacting with ruling 3 entirely — that decoupling is L-12's quiet argument.

---

## 4. The acceptance criterion (item 4 — drafted for ratification)

### 4.1 The criterion, stated

> **At a deep-cell frontier where one cell's near-surface record contains a member (e.g.
> mudstone in the mix) and its neighbour's does not, the presence border must stop being a
> straight chunk-quantized line: both parent records must be realized by voxel columns on
> both sides of the geometric cell edge, with the mix fraction shifting monotonically
> across it — and no chunk straddling the frontier may be single-parent.**

### 4.2 The instrument (gated probe; invariants, never snapshots)

**(assistant-proposed)** Fold into `appearance_tour_p11.rs` (already gated, already builds
seed 1337 Medium — **zero added pregen**; report the added wall clock per probe doctrine).
Small is unusable: its sampled chunks carry **no strata record** (journal/0129's golden
note), so Medium is the smallest extent that exercises the invariant — say so in the
test's doc comment. Asserts, each scale-free because each is a per-column predicate:

1. **Frontier finder (tour-map half):** scan adjacent deep-cell pairs for a material
   present in one top window and absent in the other; **a null is a result** — brief it,
   never fabricate a station (corrections #51).
2. **Interfingering:** in the two chunk-columns of straddling chunks, both parent cells'
   records are realized on both sides of the edge (the near sibling of the shipped
   `far_class_dither_interfingers_across_a_real_deep_cell_frontier`).
3. **The chunk-uniformity tripwire (the field report's mechanism, inverted):** every chunk
   straddling the frontier realizes **≥ 2 distinct parents** among its 1024 columns, with
   a 4σ binomial floor on the minority count **derived from the bilinear stencil weights**
   — a derived bound, not a fitted tolerance.
4. **The MM-1 law at world scale:** realized parent frequency over sampled columns matches
   the bilinear weight within 4σ (the dc-core law test's production echo).
5. **The F2 coupling:** per sampled column, `Σ unit.thickness == H` for the *dithered*
   cell (journal/0053's invariant, now per column) — the Law-3 leak tripwire.

Plus the pack's instruments when 3b lands (§ 2.3–2.5): the size/offset compile-asserts,
|carry| ≤ q, exact closure, unit-count invariance under the grain axis.

### 4.3 The walk (the loop per CLAUDE.md § Agent walks — Claude drives)

Tour-map first: the § 4.2 finder prints the strongest frontier + a ready pose. Launch
`--fullbright` (material question) `+ --edges` if geometry legibility is needed. Station 1
**before** merge if cheap (the alignment check F1 owes the field report: is the live line
on the deep-cell grid pitch?); station 2 after: same frontier, verdict. **Brief the user
honestly on what "fixed" looks like (F6):** a gradational interfingered contact spread
over up to ~460 m, mudstone fraction ramping across the cell edge — not a crisper line.
Record the station pose in Observed beside the asset (corrections #48 — the field report
currently has none). The user's live view is senior to the screenshot read.

---

## 5. Interaction with record-terms ruling 3 (item 5)

Ruling 3 (user, 2026-08-02: **FACE**) puts the composition term on the flux record as a
sparse CSR sidecar — a **different structure entirely** (`FluxRecord`, gen-tier), which
slice 3 does not touch and cannot foreclose. What the ruling explicitly keeps open is the
**packed per-unit hint** — and since `DepUnit.species` *is* the argmax, the hint byte is
the argmax's **share** (how much of the arriving mix the winner was), u8-quantized.
Statement per layout × U4, per the brief:

- **L-12: the hint byte is funded under every U4 option** (a full spare byte + 3 more).
  Cost of carrying it: already inside L-12's −29.08 MiB net.
- **L-8: the hint does NOT fit as a byte under any U4 option** (spare bits are 5–7; a byte
  needs 8). A degraded 4-bit share nibble fits at U4 ≤ 4 — 16 share levels, honest but
  coarse. Upgrading L-8 to carry the full hint later = the same +4 B step as L-12, paid as
  a second golden move.
- **Not foreclosed either way**: the accessor surface (§ 2.2) means adding the byte later
  is a layout bump + re-capture, not a re-plumb. But paid later it costs a second
  migration event — which is exactly what "one layout surgery, one golden move" was ruled
  to avoid. **This is the concrete content of the L-8 vs L-12 user pick.**

---

## 6. Goldens and migration (corrections #89 discipline: name the mechanism per family)

No world is persisted (§ 2.1) — migration is golden re-capture + probe captions only
(captions are part of the diff — CLAUDE.md § Gates).

| merge | families that move | mechanism class | families that must NOT move (assert the stillness) |
|---|---|---|---|
| **3a restructure** | CONTENTS (the Medium triples), the SURFACE family | **semantics** — per-column record membership, ratified by ruling 5 | ALL deep-time fingerprints (`GOLDEN_RECORD`+variants, `GOLDEN_GEOTHERM`, `GOLDEN_FLUX`, `GOLDEN_HEAD`, chapters) — the restructure is collapse-tier only; and `GOLDEN_FAR_SURFACE` — the far path is deliberately unconverted. **A family that moves here is a defect, not a re-capture.** |
| **3b pack** | **all 14 families** — quantized thickness feeds back through the outcrop window into rates, so the whole deep trajectory moves once | **semantics (quantization), ratified by U5** — NOT float order; and note the #89 hazard class *retires* for record sums afterward | none |

Sequencing consequence: 3a-then-3b is **two capture events** (3b's supersedes 3a's for the
collapse families); a single combined merge is one capture but a ~20-file all-at-once diff.
Priced in § 7; the order is a user-vetoable integrator call.

---

## 7. Build sequencing + the gate plan (proposal — integrator sequencing, veto welcome)

**M0 — the pre-slice measurement run (no semantics, one build-slot session):** extend an
existing probe to print (a) the **mover split factor** (§ 2.5), (b) the **max/histogram of
unit thickness** (§ 2.3), (c) **`run_strata`'s share of `column()`** (§ 1.3). Three
numbers, one run; every open pick in §§ 1–2 consumes at least one of them. *(The grain
split factor is NOT in M0 — it is unmeasurable until FS-A writes a varying grain, F5.)*

**M1 — 3a, the restructure** (SubCell + per-column dither + the F2 bundle + `records` +
the 14-file consumer migration + § 4.2 asserts + the walk). Merges on **cheap evidence**
per the 2026-08-02 batching rule: fmt + clippy on dc-worldgen/dc-core + their suites
(which is most of the workspace's cost anyway), **batch debt recorded in the commit
message**. Goldens: CONTENTS + SURFACE re-captured with the why; stillness of the
deep-time families asserted (§ 6).

**M2 — 3b, the pack** (bitfield + fixed-point + carry + agent axis per the M0-informed
M-pick + grain bits per O-pick + hint per the L-pick). **Full workspace gate here** —
clearing M1's recorded debt, closing the arc-chunk before anything ships beyond it. All
14 families re-captured once with per-family whys. Build-slot rules throughout (the hook
owns the mutex; one cargo at a time).

Order argument for 3a-first **(assistant-proposed)**: the user-visible prize (the field
report's border) and the walk attach to 3a; 3b is desk-verified by instruments and needs
no game time. Reversing them delays the only walkable deliverable behind a layout surgery.
Combined-single-merge is legitimate under "one golden move" but couples a 14-file consumer
migration to a 141-site accessor sweep in one diff — named, not recommended.

**Same-commit doc obligations** (read-first rules): stubs #25 banner (agent axis funded /
or M-2's partial), #31 disposition per D-6, spines § 3 (SubCell ships *with* consumers —
keep the row honest), dependency-graph P11/E5/P4 rows, ROADMAP arc item 3, and the
field-report Observed entry resolved with the station pose.

---

## 8. What slice 3 does NOT do

- **No grain MODELLING** — no release spectra, no abrasion, no sorting-by-φ; that is
  FS-A/P10, resuming behind this slice (ROADMAP continuation slot). Slice 3 reserves bits.
- **No far-path work** — `ShareVec<6>`/`surface_class` ride as rejected-interim; heir =
  the far register from refinement budgets. Slice 4 patches minimally; nothing here.
- **No Litho dissolution** — the ~250 class-speaking sites are slice 4 (stub #37's door
  stays open until then).
- **No WINDOW, no mover lineage** (`Fact::Move` chain — stub #25's named partner rides).
- **No composition sidecar build** — ruling 3's CSR sidecar is the recording slice of the
  record-terms arc, not this one; slice 3 only keeps the hint byte fundable (§ 5).
- **No re-opening of the pregen content door** (stub #35 / E7's manifest).

---

## 9. NEEDS RATIFICATION

### 9a. User-owned

| # | question | the shape of the trade |
|---|---|---|
| **P-1** | **U4: 8 / 5 / 4 / 2 φ classes** (§ 3) | free at the record under the pack; 5.4-halving physics bound; 8 and 5 cost identical bits; decides P10's downstream ×K costs, not slice 3's |
| **P-2** | **L-8 vs L-12** (§ 2.6, § 5) | 29.08 MiB against an always-funded hint byte + slack; L-8 + later hint = a second golden move |
| **P-3** | **grain bits: unset (O-2) vs identity-derived (O-1)** (§ 2.4) | recommendation on record: O-2; either way split-factor 1.0 asserted |
| **P-4** | **mover in the merge key (M-1) vs recorded-only (M-2)** (§ 2.5) | gated on M0's measured mover split factor — integrator-settleable if small, user call if it forces a trade |
| **P-5** | **D-6: the veneer's per-column formation context in or out** (§ 1.4) | stub #31's heir names this slice; out = re-point the heir honestly, in = measured per-column climate cost |
| **P-6** | **⚠ F6: accept the cell-wide-blend semantics at the near tier** — the walk verdict is the ratification | the far tier's rejected "swirl" arrives at the near tier by construction; the exit (perimeter weighting) is an A-1 deviation to ratify loudly, not to slip in |
| **P-7** | acceptance criterion § 4.1 + walk plan § 4.3 | drafted here for a yes/no |
| **P-8** | sequencing veto (M0→M1→M2, 3a-first, gate batching per § 7) | drafted as integrator sequencing under the standing batching rule |

### 9b. Integrator-settleable (recorded when done)

| # | item |
|---|---|
| I-1 | M0's three measurements, before any layout constant is chosen |
| I-2 | the dither's registered draw `Domain` + `Octaves` as source (record why, not just what) |
| I-3 | the `SubCell` API surface (record_for / bundle shape) — with MM-3's report judgments carried in |
| I-4 | quantum pick (2⁻¹⁰ vs 2⁻¹³) once M0's max-thickness lands; compile-assert layout + q |
| I-5 | the ~5 judgment sites in the consumer migration (per-column vs any-column semantics), named in the diff |
| I-6 | stamp targets in the same commits: stubs #25/#31/#37 banners, this file's header when refuted, priors header if the hint pick changes its "not foreclosed" line |

---

## 10. What this audit could not verify — flagged, not smoothed

1. **No cargo was run.** All layout arithmetic (§ 2) is hand-derived from field lists and
   enum inhabitant counts; not compiler-verified here. The compile-asserts are I-4's job.
2. **The 7,622,541 unit count** is 2026-08-02 (freshest in the corpus) but moves with any
   deep-sim semantic change; every MiB figure in § 2.6 inherits it.
3. **Max unit thickness is unmeasured** (§ 2.3) — u16 thickness is unsafe to consider
   until M0 measures it.
4. **`run_strata`'s share of chunk generation is unmeasured** — the 4× prior could be
   trivial or dominant; M0 decides which fight this is.
5. **The mover split factor is unmeasured** — M-1 must not be taken without it (#88).
6. **The blast-radius count (~71 sites / 14 files) is pattern-based** — variable-name
   greps miss aliases; treat as a floor with the same generosity journal/0129 earned.
7. **Whether the user's observed border IS this mechanism** is inferred, not walked —
   the mechanism is code-verified (F1), the specific line was never alignment-checked, and
   the field report has no recorded pose. Station 1 of the walk closes this.
8. **`contents_contract`'s 102.43 s baseline** is journal/0129's machine-state; re-baseline
   in M0 rather than trusting a nine-day-old wall clock.

---

*Author: read-only design-pass agent, 2026-08-02. Read-only except this file; no cargo
invoked. §§ 1.1(5), 2.4–2.6, 4.2, 7 and the F-findings' syntheses contain
assistant-originated analysis, marked where they go beyond arithmetic over cited
measurements.*
