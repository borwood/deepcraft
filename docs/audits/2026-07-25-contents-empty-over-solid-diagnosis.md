# "The contents record reads EMPTY over SOLID ground" — diagnosis

**Date:** 2026-07-25
**Mode:** DIAGNOSIS ONLY (read-only on `src/`; no fix, no plan). Every claim carries
`file:line`. A final section separates **PROVEN** (static from the code, plus the
headless probe) from **INFERRED / needs-live-probe**.

Observation under diagnosis: ROADMAP **Observed** — *"The contents record reads
EMPTY over SOLID ground"* (journal/0097 § "Wrong turns worth keeping").
Probe: `crates/dc-worldgen/examples/contents_air_over_solid_probe.rs` (new, this
audit). Paths under `crates/`.

---

## 0. TL;DR — the premise dissolves, and what is underneath is worse

**The world is not empty there. The record is not a hole. The *query* is lying,
in two fields at once, and the walk's own `eye_in_solid` was the honest
instrument.**

At journal/0097's own station — world (84 185 m, 9 212 m) = voxel (93 539,
10 236), surface voxel `h = 300` — the probe reproduces the reported band **to
the voxel**:

```
      vy | chunk_y | block     | has_contents | classified | contents
     300 |       9 | dc:peat   | true         | dc:peat    | recorded
     299 |       9 | dc:stone  | true         | dc:air     | EMPTY   <== reported "air"
      …                                                              (12 voxels)
     288 |       9 | dc:stone  | true         | dc:air     | EMPTY   *chunk floor*
     287 |       8 | dc:stone  | false        | dc:stone   | -       (honest)
```

Voxels **288–299** are `Block::Stone` — genuinely, correctly **solid**. That is
why `eye_in_solid` said `true` at 296 and 291: it reads
`HostWorld::block_at` (`dc-client/src/authority.rs:480-487` → `:464-475` →
`dc-api/src/host.rs` `block_at`), the same function `get_contents` reads for its
own `block` field (`host.rs:1269`). **The two instruments never disagreed.** One
voxel-level source of truth answered both, and it said stone.

What said `dc:air` is the query's **`classified`** field, and it said so because
of a chunk-granular reporting bug:

1. The **unrecorded basement** below the deep-time record is `Block::Stone` with
   `MixtureId::EMPTY` — solid rock, no contents record. This is normal,
   documented, and not a defect (`collapse.rs:445-459`, `:591-592`;
   `classify.rs:31-45`).
2. `chunk_contents` returns `Some(grid)` for the **whole 32³ chunk** if *any*
   voxel in it is recorded (`collapse.rs:717-724`, `intern.rs:313-314`).
3. Inside that grid the unrecorded voxels resolve to `VoxelContents::EMPTY`
   (`intern.rs:101`, `:409-418`, `:469-471`).
4. `HostWorld::contents_at` therefore returns `Some(EMPTY)`, not `None` — its
   `?` only fires when the *chunk* had nothing (`host.rs:320-326`).
5. `contents_query_data` then reports **`has_contents: true`** (`host.rs:83`) and
   **`classified = classify(EMPTY) = "dc:air"`** (`host.rs:76-79`;
   `classify.rs:97-102`) — which is precisely the operation `classify.rs:31-45`
   says must never be performed on a voxel that was never given a record.

So the answer to the ROADMAP's integrator caveat is **neither of the two options
it offered**:

> It was **not** genuinely air, and it was **not** `has_contents: false` read as
> air. It was **`has_contents: true`** — a *claimed* record — carrying an
> **empty** composition, whose derived `classified` name is `dc:air`, sitting on
> top of a correctly-solid `block: dc:stone`.

That is the third and worst case: **an agent-facing query returning a value that
looks authoritative when the truth is "no record here"**, while the flag whose
entire job is to say "no record here" says the opposite. And the same physical
voxel one chunk lower (`vy ≤ 287`, chunk_y 8, wholly unrecorded) is reported
**correctly** — `has_contents: false`, `classified: dc:stone`. The walk saw the
transition at **287/288**, which is exactly the chunk-y floor `9 × 32 = 288`.
The "~11 voxels" was 12, and its extent was decided by *a chunk boundary*, not by
anything in the world.

---

## 1. The `has_contents` question, settled

### 1.1 What the flag is documented to mean

Three places promise the same thing, and all three are wrong at voxel
granularity:

- `payload.rs:426-429`: *"Whether a full contents record backs this voxel.
  `false` under the S1 terrain authority, for legacy stubs, or when the world
  carries no contents source."*
- `schema.rs:754-756` (the MCP tool doc an agent actually reads): *"`has_contents`
  is false where no contents record backs the voxel (S1 terrain, legacy stubs)."*
- `CLAUDE.md:139` (§ Agent walks, added by the walk itself): *"check it, since
  contents-free voxels are not the same thing as air."*

### 1.2 What it actually computes

`host.rs:83`:

```rust
has_contents: contents.is_some(),
```

and `contents` comes from `HostWorld::contents_at` (`host.rs:320-326`):

```rust
let source = self.contents_source.as_ref()?;   // None only if NO source installed
let grid = source(cpos)?;                      // None only if the CHUNK is empty
Some(grid.get(lx, ly, lz))                     // else ALWAYS Some — even of EMPTY
```

The only two ways to get `None` are (a) no `ContentsSource` at all (the S1
terrain authority — `authority.rs:326-331` returns `None` for `Terrain`), and
(b) the *chunk* had no recorded voxel whatsoever. **A per-voxel absence of record
is not representable.** So `has_contents` means *"the 32³ chunk containing this
voxel has at least one recorded voxel"* — a **chunk-resolution answer wearing a
voxel-resolution name**.

The probe demonstrates the failure directly: at the station, `vy = 301` and
`vy = 302` are *sky* — `block: dc:air` — and they also report
`has_contents: true`. Recorded rock, unrecorded basement and open air all answer
`true` inside a partly-recorded chunk. Within such a chunk the flag carries **no
information at all**.

### 1.3 Why the walk read it as air

Even had the walk checked the flag, the payload reinforces the wrong reading. For
`VoxelContents::EMPTY`, `ContentsView::from_contents` (`payload.rs:363-374`)
produces `shape: "none"`, `solid_eighths: 0`, `free_eighths: 8`, and three empty
material lists. Beside a `classified: "dc:air"`, that is an unambiguous picture of
an empty voxel. Nothing in the response contradicts it except `block: "dc:stone"`
— one field out of four saying "solid", against three saying "empty".

**Settled:** the answer is `has_contents: true` with an EMPTY composition. Not
"genuinely air"; not "`has_contents: false` read as air". A **claimed record that
is empty**, over solid stone.

---

## 2. The mechanism, end to end

### 2.1 Where solidity comes from

`WorldGenerator::generate_chunk` (`collapse.rs:406-498`), per voxel:

- `vy > h` → `Block::Air` (`collapse.rs:434-436`) — the only source of `Air`.
- `vy == h` → `col.surface[i]` (`:436-437`).
- `!has_record` → legacy soil: `Dirt` above `h - soil`, else `Stone`
  (`:438-444`) — ocean floor and the **border wilds** (stubs.md § Genesis).
- otherwise `fill.plan(depth + 1)`:
  - `None` → **`Block::Stone`** — *"below it is unrecorded basement = stone"*
    (`collapse.rs:457-459`).
  - `Single`/`Mixed` → `classify` of the very contents the material path builds
    (`:460-470`).

**There is no path by which a voxel below `h` is `Block::Air`.** This is the
structural proof that the walk's column was never empty: the ON run showed the
same column with a veneer at `y = 300`, so `h = 300 > 299`, so 288–299 are
`Stone` or recorded material by construction.

### 2.2 Where contents come from

`WorldGenerator::material_ids` (`collapse.rs:521-620`) walks the *same*
`ColumnFill` and writes into `vec![MixtureId::EMPTY; CHUNK_VOLUME]`
(`collapse.rs:525`):

- the surface voxel from `fill.plan(1)`, `continue` if the column has no record
  top span (`:547-549`);
- buried voxels only `if fill.depth_count() > 1` (`:573`), and inside that loop
  `fill.plan(depth+1) == None` → **`continue`** (`:591-592`) — *"unrecorded
  basement"*, leaving `MixtureId::EMPTY`.

So block and contents agree perfectly about the *world*: unrecorded basement is
`Stone` with no mixture. The two paths are deliberately kept in lockstep
(`collapse.rs:410-423`, `:445-456`).

### 2.3 Where the empty becomes a claimed record

`chunk_contents` (`collapse.rs:717-724`):

```rust
let mc = MaterialChunk::from_dense(&dense);
if mc.is_all_empty() { return None; }        // whole-chunk, not per-voxel
mc.resolve_contents(&self.materials)
```

`MaterialChunk::is_all_empty` is `self.palette == [MixtureId::EMPTY]`
(`intern.rs:313-314`) — **true only if every voxel in the chunk is empty**. One
recorded voxel flips the whole chunk to `Some(grid)`.

`resolve_contents` (`intern.rs:409-418`) maps each palette id through the region
`MixtureTable`, whose **entry 0 is always `VoxelContents::EMPTY`**
(`intern.rs:56`, `:101`, asserted `:501`). So `MixtureId::EMPTY` resolves
successfully to `VoxelContents::EMPTY`, and `ContentsGrid::get`
(`intern.rs:469-471`) hands it back as a perfectly ordinary value.

The client installs exactly this as the query's source
(`dc-client/src/authority.rs:221-228`):

```rust
world.set_contents_source(Box::new(move |pos| {
    crate::devicelost::lock_forgiving(&contents_gen).chunk_contents(pos)
}));
```

### 2.4 Where it becomes `dc:air`

`contents_query_data` (`host.rs:75-88`):

```rust
let classified = match contents {
    Some(c) => block_name(dc_core::classify(c)),   // classify(EMPTY) == Air
    None => block_name(block),
};
… has_contents: contents.is_some(),
```

`classify(VoxelContents::EMPTY) == Block::Air` (`classify.rs:97-102`, test
`:111`). And `classify.rs:31-45` — the **absent-contents rule** — states the
prohibition in advance, naming this exact voxel:

> *"A voxel whose contents are `VoxelContents::EMPTY` is **unclassified, not
> classified as Air**. `classify` answers `Block::Air` there because a voxel with
> nothing in it is air as far as contents go — but the generator does not apply
> `classify` to voxels it never gave a contents record: … the legacy soil band and
> **the unrecorded basement below the deep-time record**, ocean floor, the border
> wilds …"*

The generator honours that rule. **`dc-api` does not** — because by the time the
answer reaches `host.rs:76`, the distinction between "a record that is empty" and
"no record" has already been destroyed by `contents_at`'s chunk-granular `?`.

### 2.5 The one-line summary

> A per-voxel question is answered from a per-chunk presence test, and the
> resulting `Option::None` — the only channel that could carry "no record here" —
> is spent on a whole-chunk condition. Everything downstream then treats
> `Some(EMPTY)` as data.

This is **spines A-2/S-1 territory in the small**: `Option<VoxelContents>` is the
right shape for the answer, and the wrong shape for the *source* it is derived
from. It is also, precisely, *"a summary is not an authority"* (ARCHITECTURE,
DECIDED 2026-07-21) at a tier nobody had audited: `has_contents` is a **summary of
the chunk** standing in as an **authority about the voxel**. Applying the doctrine's
own test — *"if this consumer disappeared tomorrow, would this code still exist in
this shape?"* — `contents_at`'s chunk-wide `?` exists only because
`chunk_contents` was built for the **mesher**, which wants a whole grid or
nothing. The query inherited the mesher's shape and its meaning with it.

---

## 3. Is it the bare-cell fallback? — refuted, but with a caveat

**Verdict: distinct. The walk's instinct was right, though not for the reason it
gave.**

The bare-cell fallback (ROADMAP:4519-4547, `examples/record_hole_probe.rs`) is a
**generation** finding: at deep cell (488, 278) the record carries `H 0.25 m ·
7 units`, under half a voxel, so the column *falls off the record path entirely*
and gets year-zero fallback paint (`dc:dirt`, twinning `LOAM`, in no class) over
unrecorded basement. Its census is `0.2 % of land (91 of 44 265 cells)` expressing
zero voxels, and its two named defects are `regolith_at_voxel` NEAREST-vs-bilinear
and the sub-voxel record falling into paint.

This observation is a **query-surface** finding at a column with a **perfectly
ordinary, healthy record** — the walk's own station carried a 6.09 m weathering
band in the ON run. Nothing about the world is wrong at 288–299. Different layer
(dc-api vs dc-worldgen), different cause (an `Option` collapsed at chunk
granularity vs a record too thin to express), different fix surface.

The walk's stated distinction — *"that one is paint over stone; this is the record
reading empty over solid"* — should **not** be inherited verbatim, because its
second half is the false premise. The correct distinction is: *that one is a real
thin-record generation artifact; this one is a reporting artifact over a normal
record.*

**Caveat worth the integrator's attention.** The probe did surface one genuinely
adjacent fact at the same station: in the **flag-OFF control**, the top 65 voxels
of that column contain **exactly one** recorded voxel (`dc:peat` at `y = 300`) and
64 unrecorded ones. The record expresses the veneer and nothing below it — the
same *thinness* family as the bare-cell finding, one notch less extreme (one voxel
rather than zero). That is a real observation about the world; it is **not** what
the "empty over solid" report was about, and it should be filed separately rather
than folded in.

---

## 4. Position-dependent, or global?

**Global — and the "position" that decides it is a chunk boundary, not a place in
the world.**

Probe census: 169 columns on a 4 096-voxel (≈ 3.7 km) lattice around the station,
each column's top 65 voxels, flag-OFF:

| | count | share of solid |
|---|---|---|
| solid voxels examined | 10 985 | 100 % |
| **phantom air** (solid · `has_contents: true` · `classified: dc:air`) | **702** | **6.4 %** |
| honest no-record (solid · `has_contents: false`) | 4 211 | 38.3 % |
| genuinely recorded | 6 072 | 55.3 % |
| columns with ≥1 phantom-air voxel | **39 / 169** | 23 % |

Per-column spread (probe output): 3 to 32 phantom-air voxels in the top 65,
depending only on how far the record's bottom sits above the nearest chunk floor.
So:

- It is **not** a property of the walk's region, a marine cell, the border wilds,
  or a record hole. It appears wherever a chunk contains *both* recorded and
  unrecorded voxels — which is every chunk the record's bottom passes through, and
  every chunk holding a thin-record column.
- Its **vertical extent is set by the chunk lattice**: from the record's bottom
  down to the chunk floor, then it stops and the honest report resumes. The
  reported band always ends on a multiple of 32.
- It applies to the **worldgen authority only** (key 2). Under the S1 terrain
  authority no contents source is installed (`authority.rs:326-331`), so
  `has_contents` is uniformly `false` and honest.

---

## 5. Blast radius — who is misled

| Consumer | Path | Misled? |
|---|---|---|
| **`world_get_contents`** (agent walks) | `host.rs:1268-1272` → `:75-88` | **YES** — `has_contents: true` + `classified: dc:air` over solid stone. The reported defect. |
| **F3 look-at HUD** (human at the keyboard) | `inspector.rs:125-133` | **YES, identically.** The `let Some(c) = contents else { "(no contents record here)" }` escape at `:125-126` is exactly the right message and is **never reached** for these voxels; the HUD instead prints `block: dc:stone  classified: dc:air / shape: none  solid 0/8  free 8` with no material lines. |
| **`character_sense_raycast`** | `host.rs:1424-1426` | **YES, mildly.** Returns `Some(<empty ContentsView>)` where `payload.rs:485-491` documents `None` for "no contents record". A sensing character reads a hit on solid rock whose composition is nothing. |
| **The mesher** | `meshing.rs:295-306`, `:266-268` | **NO.** `cover_frac` returns `1.0` for any non-`Block::Material` block before contents are consulted, and the `Some(c) if !c.is_empty()` arm guards the rest. `NeighborFill::fill` (`authority.rs:1102-1122`) gates on `block_uses_contents` first. Unrecorded basement is `Block::Stone`, so the render never asks. **This is why nobody saw it for months: the only consumer that reads contents at scale is structurally immune, and the two that are misled are the two *diagnostic* surfaces.** |
| **Far field / LOD** | `farfield.rs:134-152` | **NO.** Explicitly guards: the span's block is `classify` *"where the reduced contents are non-empty"*, falling back otherwise. |
| **Physics / collision / `eye_in_solid`** | `authority.rs:464-487` | **NO.** Reads `block_at` only. It was right the whole time. |
| **Break / place** | `apply_block_changes` (`authority.rs:856-878`), edits write blocks | **NO** today — edits never consult contents. Becomes exposed the moment a break yields *items* derived from `get_contents`. |
| **`identify(pos)`** (Sequenced, not yet built) | — | **Would inherit it verbatim** if built over `contents_at` as-is. This is the arc's first concrete requirement: the tier flag must be able to say **"unrecorded"** as a first-class answer, distinct from both "air" and "recorded". |

**Same family as "block is a summary"?** Yes — it is the *inverse* instance of the
same doctrine. journal/0097's other finding was a summary (`Block`) being read as
an authority about materials. This is a summary (`has_contents`, a chunk fact)
being read as an authority about one voxel, *and* a derived name (`classified`)
being computed from a value that carries no information. Both are the same failure
of the same rule, at the same query surface, found on the same walk.

---

## 6. Is it a defect at all?

**Yes — but not the defect that was filed, and not in the layer it was filed
against.**

- **The world is correct.** Solid stone under a thin record is exactly what
  `collapse.rs:457-459` intends, and `classify.rs:31-45` names the unrecorded
  basement in advance as a legitimate absent-contents case. There is nothing to
  fix in dc-worldgen. **The ROADMAP Observed entry as written — "the record reads
  empty and the world is solid", "~11 voxels", "Movement 3 did not cause it" — has
  a true clause (M3 is innocent) wrapped around a false premise.**
- **The report is wrong, twice.** `has_contents: true` contradicts its own
  documented contract in `payload.rs:426-429` and `schema.rs:754-756`, and
  `classified: dc:air` is `classify` applied to a voxel the core module explicitly
  forbids applying it to. Both are defects in `dc-api`, one line each
  (`host.rs:83`, `host.rs:76-79`), rooted in one design choice
  (`host.rs:320-326`).
- **It is therefore a "correct-but-badly-surfaced no record here" that has been
  surfaced so badly it reads as a world defect** — which is the strongest possible
  argument for the `identify(pos)` arc, made in the field, at cost: a walk
  reported a phantom world bug, the ROADMAP filed it as Observed, and an agent
  was dispatched to diagnose a world that was never broken.

The honest one-line verdict: **not a record hole; a record *reporting* hole.**

---

## 7. PROVEN vs INFERRED

### PROVEN (static from the code, and reproduced headless)

- No voxel below a column's height `h` can be `Block::Air`: the only `Air` branch
  is `vy > h`. `collapse.rs:434-436`; unrecorded basement is `Block::Stone`,
  `collapse.rs:457-459`. Therefore the walk's 288–299 was never empty world.
- `eye_in_solid` and `get_contents`'s `block` field read the **same**
  `HostWorld::block_at`. `authority.rs:480-487` → `:464-475`; `host.rs:1269`.
  They cannot disagree; there was no instrument contradiction.
- `contents_at` returns `None` only when no source is installed or the **whole
  chunk** is empty; otherwise always `Some`, including `Some(EMPTY)`.
  `host.rs:320-326`; `collapse.rs:717-724`; `intern.rs:313-314`.
- `MixtureId::EMPTY` resolves to `VoxelContents::EMPTY` through the region table
  (entry 0 is always EMPTY), so an empty voxel inside a non-empty chunk survives
  `resolve_contents` as a value. `intern.rs:56`, `:101`, `:409-418`, `:469-471`.
- `has_contents` is `contents.is_some()` — a **chunk-resolution** fact published
  under a voxel-resolution name that three docs describe otherwise.
  `host.rs:83`; `payload.rs:426-429`; `schema.rs:754-756`.
- `classified` is `classify(EMPTY) = Block::Air`, the operation `classify.rs:31-45`
  forbids for unrecorded voxels. `host.rs:76-79`; `classify.rs:97-102`, `:111`.
- The F3 HUD has the identical bug and its correct "(no contents record here)"
  branch is unreachable for these voxels. `inspector.rs:125-133`.
- The mesher and the far field are **not** misled (block gate / non-empty guard).
  `meshing.rs:266-268`, `:295-306`; `authority.rs:1102-1122`; `farfield.rs:134-152`.
- **Reproduced headless at the walk's own station**, seed 1337 / Medium /
  flag-OFF: voxel (93 539, 10 236), `h = 300`; `has_contents: true` +
  `classified: dc:air` + `block: dc:stone` for **288–299**; honest
  `has_contents: false` + `dc:stone` from **287** down. The transition is the
  chunk-y floor at `9 × 32 = 288`. Matches journal/0097's reported range exactly.
  `examples/contents_air_over_solid_probe.rs`.
- **Global, not regional**: 702 of 10 985 solid voxels (6.4 %) over 169 columns on
  a 3.7 km lattice; 39 of 169 columns show at least one; per-column count 3–32.
  Same probe.
- Under the S1 terrain authority no contents source exists, so the flag is
  uniformly `false` and honest there. `authority.rs:326-331`.

### INFERRED / needs-live-probe

- **Which field the walk actually read.** The probe proves the response contains
  `block: "dc:stone"` and `classified: "dc:air"` simultaneously; journal/0097
  records only *"`world_get_contents` returned `dc:air`"*. That the walk read
  `classified` (or the empty `ContentsView`, whose `shape: "none" / solid 0/8 /
  free 8` reads as air) is the strongly-indicated reading, not a proven one. It
  does not change the diagnosis — no other field can produce `dc:air` there — but
  a live `world_get_contents` at (93 539, 296, 10 236) would put the whole
  response on the record and close it. **Cheap, and worth doing on the next walk.**
- **Whether the ON run's band bottom also produces a phantom-air band below
  itself.** Structurally it must (the record still ends somewhere above a chunk
  floor), but the probe only measured flag-OFF. A `--weather-inventory` ON census
  would confirm the band merely *moves down*, and would explain why the walk saw
  the ON run "fill that same range" — the record grew into the reported gap.
- **Whether any not-yet-written consumer is exposed.** Break/place and
  `identify(pos)` are called out on their designs, not on shipped code.

---

## 8. Recommended corrections (for the integrator — this agent wrote no
non-audit files)

### 8.1 `journal/corrections.md` — ready to paste

> **#NN — "The contents record reads empty over solid ground" was a query bug,
> not a world bug** (2026-07-25, journal/0097 → this audit).
> *Claim:* `world_get_contents` returning `dc:air` for voxels 288–299 while
> `eye_in_solid` said `true` meant ~11 voxels where the record was empty and the
> world was solid — a record hole, adjacent to the bare-cell fallback.
> *Falsified.* The world was solid and correct there: unrecorded basement is
> `Block::Stone` by construction (`collapse.rs:457-459`), and no voxel below a
> column's height can be `Block::Air` at all (`collapse.rs:434-436`).
> `eye_in_solid` and `get_contents`'s own `block` field read the **same**
> `HostWorld::block_at` (`authority.rs:480-487`, `host.rs:1269`) and both said
> stone. What said `dc:air` was the derived **`classified`** field.
> *Mechanism.* `HostWorld::contents_at` (`host.rs:320-326`) answers a per-voxel
> question from a per-chunk presence test: `chunk_contents` returns `Some(grid)`
> if **any** voxel in the 32³ chunk is recorded (`collapse.rs:717-724`,
> `intern.rs:313-314`), and inside that grid an unrecorded voxel resolves to a
> perfectly ordinary `VoxelContents::EMPTY` (`intern.rs:101`, `:409-418`). So
> `has_contents` reports **`true`** (`host.rs:83`) — contradicting its own doc
> (`payload.rs:426-429`, `schema.rs:754-756`) — and `classified` reports
> **`dc:air`**, which is `classify` applied to an unrecorded voxel, the one thing
> `classify.rs:31-45` says never to do. The band's extent (288–299) was set by the
> chunk floor at `9 × 32 = 288`, not by anything in the world.
> *Scope:* 6.4 % of solid voxels in the top 65 of a column, 39 of 169 sampled
> columns — global, worldgen authority only. The F3 HUD (`inspector.rs:125-133`)
> and `character_sense_raycast` (`host.rs:1424-1426`) carry the identical bug;
> the mesher and far field are immune (`meshing.rs:295-306`,
> `farfield.rs:134-152`), which is why only the diagnostic surfaces ever showed
> it.
> *Lesson:* `Option::None` was the only channel that could say "no record here",
> and it had already been spent on a whole-chunk condition inherited from the
> mesher's needs. **A summary is not an authority** — including when the summary
> is a `bool` named after the thing it is not measuring.

### 8.2 ROADMAP **Observed** — replacement wording

Replace the entry at `ROADMAP.md:3033-3049` (including its INTEGRATOR CAVEAT
block, now answered) with:

> - **DIAGNOSED 2026-07-25 — `world_get_contents` reports `dc:air` and
>   `has_contents: true` over solid, correctly-unrecorded rock** (walk observation
>   journal/0097; diagnosis
>   `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`, probe
>   `dc-worldgen/examples/contents_air_over_solid_probe.rs`). **The original
>   premise is falsified** (corrections #NN): the world was never empty there, and
>   `eye_in_solid` was the honest instrument — it and the query's own `block` field
>   read the same `block_at` and both said `dc:stone`. The mechanism is a
>   query-surface defect in dc-api: `contents_at` answers a per-voxel question from
>   a per-chunk presence test (`host.rs:320-326`), so an **unrecorded basement**
>   voxel sharing a chunk with any recorded voxel comes back as
>   `Some(VoxelContents::EMPTY)` — reported as `has_contents: true`
>   (`host.rs:83`, contradicting `payload.rs:426-429` / `schema.rs:754-756`) with
>   `classified: dc:air` (`host.rs:76-79`, the operation `classify.rs:31-45`
>   forbids). Reproduced to the voxel at journal/0097's own station: phantom band
>   288–299, honest from 287 down, the transition being the chunk floor `9×32`.
>   **Global**: 702/10 985 solid voxels (6.4 %) over 169 columns; 39/169 columns
>   affected; worldgen authority only. **Blast radius:** the F3 HUD
>   (`inspector.rs:125-133`) and `character_sense_raycast` (`host.rs:1424-1426`)
>   carry it identically; the mesher (`meshing.rs:295-306`) and far field
>   (`farfield.rs:134-152`) are immune. **NOT the bare-cell fallback** (that one is
>   a real thin-record *generation* artifact; this is a *reporting* artifact over a
>   healthy record) — the two Observed entries stay separate. **Owed:** the fix is
>   the `identify(pos)` arc's first concrete requirement — a tier flag that can say
>   **"unrecorded"** as a first-class answer, distinct from both "air" and
>   "recorded". Not fixed here (diagnosis-only agent).

### 8.3 Two smaller items the integrator may want to file

1. **New Observed (or a line on the bare-cell entry):** at journal/0097's station,
   the flag-OFF record expresses **one** voxel (`dc:peat` at `y = 300`) over 64
   unrecorded — the *thin-record* family again, one notch less extreme than the
   bare-cell zero. Measured by the same probe. Genuinely about the world, unlike
   the entry above.
2. **`docs/design/stubs.md`:** the unrecorded basement is listed under § Genesis /
   surviving fallbacks as a legitimate absence of record (`stubs.md:65-70`). That
   remains correct. What is missing is the note that **no query surface can
   currently express that absence at voxel resolution** — worth a line, since the
   stubs doctrine is that an unlisted loose end is the defect.
