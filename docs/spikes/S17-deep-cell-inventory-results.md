# S17 — Deep-cell material inventory (spike results)

> **Keystone extension — 2026-07-24 (merge-candidate).** After the commit-semantics
> plea below was **ratified** (`material-behavior.md` § "Commit semantics — DECIDED
> 2026-07-24"), the spike was rebased onto merged main (**`1a16e67`** — A1's
> block↔material collapse) and extended into the ratified shape: a **transformation-
> fact ledger**, a **diff-and-append `commit_chapter`**, and a **provenance read**.
> See § 7 (appended) for the keystone results; §§ 1–6 are the original spike and
> stand except where § 7 supersedes the commit mechanism.

**Status: spike complete (agent A2, 2026-07-23).** De-risks
`docs/design/material-behavior.md` §1 — the deep cell's mutable **working
material inventory** + the **chapter-commit** — and closes three of the §10 open
questions. This is a *prototype + measurement*, not a merge-ready milestone: the
substrate, its identity default, the chapter-commit and the collapse/quantize
demonstration exist and are gated; **no real behavior** (no weathering, no
dissolution) was built.

- Branch: `worktree-agent-a440caba50b9545b5`, branched from main **`606f17a`**.
- Code: `crates/dc-worldgen/src/deeptime/inventory.rs` (substrate),
  `crates/dc-worldgen/tests/s17_deep_cell_inventory.rs` (falsifiers +
  measurement).
- Spines ridden: **S-1** (bounded derivation — the inventory is a bounded local
  span-list built from the record), **S-2** (committed facts vs fluid — the
  record stays the temporal authority, the working inventory is the mutable
  fluid state, *store only what derivation cannot predict*), **S-9** (derive +
  facts — the present still derives from record + inventory).

---

## 1. Byte-identity verdict — **PASS**

**The identity-default collapse is byte-identical to pre-spike main.**

Two independent grounds:

1. **By construction.** The spike touches nothing on the gen/collapse path. The
   only edits are a **new** module (`inventory.rs`), two additive lines in
   `deeptime/mod.rs` (`pub mod inventory;` + a re-export), a new test file, and
   this doc. `run_cells` / `build_field` / `collapse.rs` are unmodified, so the
   collapsed world is bit-for-bit what `606f17a` produced.

2. **Demonstrated on live data, non-circularly.** The whole point of an
   identity default (the deep-sim-flags pattern) is that the seam is provably
   free *when empty*. The test
   `identity_default_commit_is_byte_identical_over_a_whole_field` builds a real
   production `DeepField` (all flags on) on a Small world, and for **every one of
   its 25,600 cells** does `record → build_identity(PerStratum) → commit_chapter
   → record'` and asserts `record' == record` (`DeepStrata` derives
   `PartialEq`). All 25,600 pass.

**Golden provenance (the anti-circularity point).** The comparison target is the
`DeepStrata` that unmodified `build_field` emits at `606f17a` — the pre-spike
authority itself, not a snapshot captured *from* the inventory. The inventory is
never asserted to reproduce its own output; it is asserted to reproduce the
**record**, which existed before the spike. (This is exactly the trap
corrections warns about with circular goldens, avoided by anchoring on the
prior authority.)

Why per-stratum is exact: at `Granularity::PerStratum` a span ↔ record unit is
1:1 and the thickness is carried by value, so the commit reproduces each
`DepUnit` (tag, chapter, unconformity, thickness) with no float re-accumulation.
The private `stripped` flag and the `strips` counter are never touched (the
commit only rewrites the public `units`), so the whole struct compares equal.

---

## 2. Granularity recommendation — **per-stratum**

Measured on the Small production field (seed `0x0D5EED572026`), 25,600 cells:

| metric | value |
|---|---|
| record units total | 19,935 |
| mean units / cell | 0.78 (max 9) |
| mean H | 0.007 m (max 2.31 m) |
| `sizeof(Portion)` | 16 B |
| `sizeof(InvSpan)` | 32 B |
| `sizeof(WorkingInventory)` | 48 B |
| **per-stratum footprint (this field)** | **2.089 MiB (85.5 B/cell)** |
| per-voxel (0.9 m) footprint (this field) | 2.089 MiB (identical here) |
| current `DeepField` resident | 3.265 MiB |

Extrapolated by bytes-per-cell to the production grids (per-stratum):

| grid | cells | per-stratum working inventory |
|---|---|---|
| Medium (~226 km) | ~241 k | **~19.7 MiB** |
| cap (`DEEP_MAX_WIDTH²` = 550²) | ~302 k | **~24.7 MiB** |

**The two granularities coincide on this world** because the cover is thin —
mean H (0.007 m) is far below one collapse voxel (0.9 m), so almost every unit
is a single sub-voxel slice and per-voxel produces the *same* span count as
per-stratum. The divergence only appears in **thick** columns, and there it is
severe: the illustrative H = 20 m / 4-unit column measures

- per-stratum: **4 spans / 192 B**
- per-voxel (0.9 m): **24 spans / 1408 B** — **7.33× heavier.**

**Recommendation: per-stratum.** It is never more expensive than per-voxel (equal
in thin cover, up to ~7× cheaper in thick cover), it commits *exactly* (per-voxel
re-accumulates thickness under float addition at the re-merge — a second strike,
not just memory), and it follows the record's own tag boundaries so the
span↔unit provenance the chapter-commit needs is 1:1 and free. Per-voxel buys
nothing the collapse-time quantize (§4 below) doesn't already provide: the
eighth-resolution view is *derived at collapse*, so paying for it *in storage* at
depth is the summary-becomes-authority anti-pattern one tier down.

**The tradeoff, stated not optimised (per the brief).** ~20–25 MiB at production
scale is real but is **one quarter** the class of the eolian record's +92.76 MB,
and — the load-bearing mitigation — **it need not be resident at all.** Because
the record is the committed authority and `build_identity` reconstructs the
working inventory from it, the inventory is a **per-chapter transient**: at each
chapter start `working = build_identity(record)`, behaviors mutate it, the
chapter boundary commits deltas back, and it is dropped. Held whole-grid during a
chapter it costs the ~25 MiB above; streamed cell-by-cell (or tile-by-tile) its
resident cost is a rounding error. Either way it is **not** a permanent addition
to `DeepField` the way the eolian facies axis was. This is the S-2 dividend:
the fluid state is derivable, so we are free to *not* store it.

---

## 3. The concrete shape prototyped

Types (all in `deeptime/inventory.rs`):

```
FracM = f64                       // finer-than-eighth quantity, METRES (the deep unit)

enum InvForm { Structure, Loose, PoreFill, Fluid }   // Void = complement, not stored (§2)

struct Portion { material: MaterialId, form: InvForm, quantity_m: FracM }   // the (Material,Form) fraction §4/§6

struct InvSpan {                  // a VoxelContents-shaped span over a depth interval
    tag: DepTag, chapter: u8, unconformity: bool,   // provenance for the commit
    portions: Vec<Portion>,                          // the multiset behaviors RMW
}

enum Granularity { PerStratum, PerVoxel { voxel_m } }

struct WorkingInventory { spans: Vec<InvSpan>, basement: MaterialId, granularity }
```

**Where it hangs off the deep cell.** One `WorkingInventory` per deep cell,
derived from that cell's `DeepStrata` (the per-cell record already on
`DeepGrid`/`DeepField`). Spans are bottom-up, mirroring `DeepStrata.units`;
below the spans is the basement (`R`, derived at real collapse from the tectonic
province — here the reference `GRANITE`, and deliberately **outside** the commit
because the record only ever held the `H` column).

**How the delta commits into the record** (`commit_chapter`, the
deeptime-as-compiler step §1/§8): reconstruct `units` from the spans,
run-length-merging consecutive spans that share `tag`+`chapter` and don't open an
unconformity — *the recorder's own merge rule* (`DeepStrata::deposit`), so
per-voxel subdivisions re-coalesce and a real behavior's changed quantities
become committed history. Identity ⇒ rebuilt units == input units.

**The capability (`InvCtx`, §7).** Granularity-agnostic by construction: it
indexes by span (`span_count`, `fraction(span, material, form)`) and exposes one
RMW primitive — `move_form(span, material, from, to, qty)`, a **§3
form-transition edge**, mass-neutral within the span, clamped so it cannot mint
material. This is the whole `read → write` of §6; a real weathering behavior is
`move_form(.., Structure, Loose, rate·dt)` and nothing else. No behavior runs in
the spike, but the primitive is present and tested, so the substrate demonstrably
carries the §3 edge graph and the §4 (Material,Form)-fraction RMW. The `ctx` is
identical whether spans are per-stratum or per-voxel — the shape is **not**
accidentally deeptime-only.

**Material provenance.** A span's identity material is
`litho_of_tag(tag).reference_material()` — the **same** routing the collapse
tier's `deep_class` uses (asserted mirror in `tests/erodibility.rs`). The
inventory builds the rock the world already builds, not a parallel guess.

---

## 4. §10 open questions — closed vs remaining

| §10 question | verdict |
|---|---|
| **Byte-identity of the empty seam** | **CLOSED** — §1: byte-identical, whole-field proof, non-circular golden. |
| **Span granularity** (per-voxel vs per-stratum) | **CLOSED → per-stratum** — §2, measured. |
| **Fractional representation** (sub-eighth at depth, quantise at collapse) | **CLOSED** — spans hold metres (e.g. a 0.37 m portion survives with no rounding); `quantize_to_eighths` / `collapse_top_voxel` are the *one* step eighths appear (0.37 m / 0.9 m → 3 eighths). Confirmed the quantise-at-collapse step is exactly where eighths are minted. |
| **Fluid as a stored role vs derived** | **PARTIALLY CLOSED (as specified)** — the substrate *accommodates* `InvForm::Fluid` (a legal `move_form` target) but the identity default populates none and the spike builds no water. Whether the working inventory ever *stores* a fluid role stays tied to the water model (S-2: bound water is derived in a halo), as §10 says. Confirmed the substrate need not build water to accommodate it. |
| Coupling timescale vs cadence | **OUT OF SCOPE** (a pass-scheduling measurement, not a substrate question). |
| Entity substrate / actor-stepping | **OUT OF SCOPE** (deferred second substrate). |
| Dissolution carbonate-gate | **OUT OF SCOPE** (no behavior built). |

### Pleas — where the spec was underspecified (per north-star § Compliance, not a silent divergence)

1. **The record is tagged by depositional *environment*, not material; the
   working inventory is `(material, form)`. The mapping is one-way.**
   `litho_of_tag` is many-to-one (e.g. `Subsea/Mineral` and `Subaerial/Low/
   Mineral` both → `ClasticFine`), so a working inventory cannot reconstruct the
   `DepTag` from its material. §1 says "chapter boundaries commit the working
   inventory's deltas back into the record" but does not say the span must
   **retain the source `DepTag`** to do so faithfully. This spike carries
   `tag/chapter/unconformity` on `InvSpan` as commit provenance; the spec should
   name that a span's environment-tag is preserved *alongside* its material, or
   state how a behavior that changes the material (e.g. diagenesis mudstone→slate)
   is expected to re-tag. **This is the single sharpest under-specification.**

2. **A behavior that changes a span's *material* (not just its form) has no
   defined commit semantics.** The §3 edge graph is form→form (material
   conserved). Genuine material change (dissolution removing carbonate,
   cementation adding a precipitate) alters the `(material, …)` key. The commit
   here handles form/quantity deltas; material-changing behaviors would need the
   record's tag vocabulary to grow, or an explicit "material overrides the tag's
   `deep_class` routing" rule. Out of scope to build, in scope to flag.

3. **"Stack of `VoxelContents`-shaped spans" is aspirational, not literal, at
   depth.** `VoxelContents` is eighth-quantised (`u8` slot counts) and present-
   tier; the deep tier is fractional metres. The prototype's `InvSpan` is
   *`VoxelContents`-shaped* (three-role multiset, structure/pore/loose) but
   **cannot be** a `VoxelContents` until the collapse quantise. The spec's "one
   shape at different resolutions" holds as a *correspondence*, not a shared
   type — worth stating so an implementer does not try to store `VoxelContents`
   at depth.

---

## 5. Gate results

All on branch `worktree-agent-a440caba50b9545b5`, `--release`, at `606f17a`:

- `cargo fmt --all --check` — **PASS** (exit 0).
- `cargo clippy --workspace --all-targets --release -- -D warnings` — **PASS**
  (exit 0). `cargo clean -p dc-worldgen --release` first; verified
  `Checking dc-worldgen … \agent-a440caba50b9545b5\crates\dc-worldgen` in the log
  (my worktree path, not a sibling's).
- `cargo test --workspace --release` — **PASS**. `cargo clean -p dc-worldgen
  --release` first; verified `Compiling dc-worldgen …
  \agent-a440caba50b9545b5\crates\dc-worldgen`.

New tests by name (10 total):

`inventory.rs` unit tests (7): `identity_default_per_stratum_round_trips_byte_identically`,
`per_stratum_span_material_matches_the_collapse_tier_routing`,
`per_voxel_round_trips_within_floating_point_and_preserves_tags`,
`move_form_is_a_mass_neutral_read_modify_write_edge`,
`substrate_accommodates_fluid_without_the_identity_default_building_it`,
`eighths_appear_only_at_the_quantize_step`, `footprint_and_sizes_are_reported`.

`tests/s17_deep_cell_inventory.rs` (2):
`identity_default_commit_is_byte_identical_over_a_whole_field` (25,600 cells),
`memory_measurement_per_stratum_vs_per_voxel`.

(The 7th "test" in the file count is the harness's `footprint_and_sizes` size-pin.)

**Sibling builds live during the gate:** A1 (`dc-core/materials/**`, `dc-client/**`)
was running concurrently on the shared `CARGO_TARGET_DIR`. Poisoning guard
followed: waited for `Get-Process cargo,rustc,dc-client` empty before each
cargo call, `cargo clean -p dc-worldgen --release` before both gate builds, and
verified the `Compiling`/`Checking dc-worldgen` line named **this** worktree's
path. No sibling `dc-worldgen` artifact could have been served.

## 6. Branch

`worktree-agent-a440caba50b9545b5` (from main `606f17a`).

---

## 7. Keystone extension — the transformation-fact ledger (2026-07-24)

Rebased onto merged main **`1a16e67`** (A1's block↔material collapse merged
cleanly — `inventory.rs` is a new file, disjoint from `collapse.rs`; no
conflicts). Built the ratified commit-semantics on top of the proven identity
seam. Still **no real behavior** — only the fact-ledger, the diff-and-append
commit, the provenance read, and their tests.

### 7.1 The fact/unit types and where they hang

The ratified model: **a unit = immutable depositional base (`DepTag` + thickness)
+ an appended list of transformation facts; current composition = `derive(tag)`
then fold the facts** (S-9 per unit). Types (`deeptime/inventory.rs`):

```
enum Fact {                              // the persistent compiled artifact
    InPlace { chapter: u8,
              from: (MaterialId, InvForm),
              to:   (MaterialId, InvForm),
              fraction_m: FracM },
    // (reserved) Move { chapter, from_addr, to_addr, portion }  <- see 7.5
}
struct FactLedger { facts: Vec<Vec<Fact>> }   // facts[i] = facts on units[i]
```

`InvForm` gained a **`Void`** variant — *edge endpoint only, never a stored
portion* — so dissolution (`… → Void`) and deposition (`Void → …`) ride the same
`Fact` shape as material change and form-only change. One `Fact` carries all three
§3 process classes; `sizeof(Fact) ≤ 24 B`, `sizeof(InvSpan) = 32 B`.

**Where it hangs (reported design choice — a plea, not a silent divergence).**
The ledger is a **sidecar** `Vec<Vec<Fact>>` keyed by unit index, *parallel to*
`DeepStrata.units`, **not** a `facts` field grown onto `DepUnit`. `DepUnit` is
`Copy` and read across the just-merged `collapse.rs` / `erosion.rs` / `biotic.rs`;
growing a `Vec` onto it un-`Copy`s it and churns the exact files A1 merged this
cycle. The sidecar keeps this spike's write-set disjoint (only `inventory.rs`) and
the base byte-identical. **The eventual home is a `RecordedUnit { base, facts }`
on `DeepStrata`** — the integration step (see § 7.6 plea 1). The sidecar's one
fragility: if the live erosion loop pops/merges `units` *between* build and commit,
the indices drift; today facts are produced and consumed within one chapter-cycle
so this does not bite, but the `RecordedUnit` home is what makes facts pop *with*
their unit under erosion.

### 7.2 The diff-and-append `commit_chapter`

`build_working(strata, ledger)` re-derives the chapter's working inventory from
`base + facts` (S-2: the working inventory is transient compiler scratch) and
snapshots that as the **baseline**. A behavior mutates portions through
`InvCtx::apply_edge(span, (from_mat,from_form), (to_mat,to_form), qty)` — the §3
edge, mass-conserving except at a `Void` side. `commit_chapter(inv, &mut ledger,
chapter)` **diffs** each span's post-behavior portions against its baseline: net
per-`(material, form)` losses are **sources**, gains are **sinks**; sources pair to
sinks in canonical order → `InPlace` facts appended to the span's unit. Leftover
source ⇒ `→ Void` (dissolution); leftover sink ⇒ `Void →` (deposition).

- **Empty delta ⇒ no facts ⇒ record byte-identical** (the identity default, on the
  fact path).
- **The record's `units` are never written** — only the ledger grows (the ratified
  "base immutable, append facts"). Depositional arrival stays `deposit_deep_history`
  (untouched).

The diff is **exact when the behavior's net effect is a set of edges with distinct
endpoints** (the single-edge case the tests exercise). The greedy source→sink
pairing is a deterministic *simplification* of the general minimal-move assignment
for a multi-source/multi-sink chapter — flagged in § 7.6 plea 2.

### 7.3 Byte-identity (identity default) + the non-identity agreement test

Both over a real production `DeepField` (25,600 cells, seed `0x0D5EED572026`),
built by **unmodified** deep-time code on merged main:

- **`identity_default_appends_no_facts_over_a_whole_field`** — build from
  `base + empty-ledger`, run **no** behavior, `commit_chapter` → `ledger.is_empty()`
  and `compose_unit(base, []) == derive_base(base)` for **every** cell. The record
  base is the pre-spike authority (never rewritten), so byte-identity holds; and
  `build_field`/`run_cells`/`collapse.rs` are untouched, so the collapsed world is
  byte-identical to merged main regardless.
- **`non_identity_agreement_on_a_real_cell`** — on a real cell's bottom unit, apply
  a known material-change edge (`base_mat/Loose → CLAY/Loose`, half the mass);
  `commit_chapter(chapter=5)` appends **one** fact whose `(chapter, from, to,
  fraction)` match; re-deriving `base + facts` shows CLAY holding the moved mass;
  the `UnitProvenance` read returns the fact and the same composition.

Plus 8 unit falsifiers in `inventory.rs`, incl. `dissolution_edge_commits_a_to_void_fact`
(a `→ Void` fact shrinks the re-derived column) and
`non_identity_a_known_edge_commits_a_fact_that_re_derives`.

### 7.4 The provenance read shape

```
UnitProvenance::of(&strata, &ledger, i) -> Option<{ base: &DepUnit, facts: &[Fact] }>
    .compose() -> Vec<Portion>   // derive(base) then fold facts (current composition)
    .facts()   -> &[Fact]        // the lineage, chapter-ordered
```

"Started as X, chapter-Z weathering did Y" is exactly `base + facts`. **Per-voxel
provenance falls out as a read** over the per-unit ledger (DECIDED), which is why
the ledger is per-unit (per-stratum) and needs no per-voxel storage.

### 7.5 Move-fact room (forward-note, not built)

`Fact` is an `enum` precisely so a `Move { chapter, from_addr, to_addr, portion }`
sibling lands without disturbing `InPlace` — the representational room the DECIDED
required ("a move is itself a fact and travels with the material; provenance
addresses the portion's lineage"). Transport in deeptime and pickup/deposit in the
present will append a `Move` and relocate the portion's address. **Not built.**

### 7.6 Where the DECIDED was underspecified (pleas — filed, not diverged)

1. **The fact ledger's storage home.** DECIDED says "a unit *gains* an appended
   fact list" but not the storage. I used a unit-indexed sidecar (§ 7.1) to avoid
   un-`Copy`ing `DepUnit` right after A1's collapse merge. The faithful long-term
   home is `RecordedUnit { base: DepUnit, facts: Vec<Fact> }` on `DeepStrata`, so
   facts travel with their unit through erosion pop/merge. This is an integration
   decision (touches shared `recorder.rs` + `collapse.rs`), deliberately left for
   the integrating session.
2. **Multi-source/multi-sink factoring.** "The deltas ARE the facts" is exact for a
   single edge but a chapter with several coincident edges has no unique
   factorization of the net delta into `(from → to)` moves. I used deterministic
   greedy pairing; the honest fix is for the ctx/behavior to **emit the edges it
   ran** (each `apply_edge` is already a fact-shaped event) rather than reconstruct
   them from a multiset diff — i.e. record moves at apply-time, so the commit is a
   *fold of logged edges*, not a diff. That reconciles with "diff vs chapter-start"
   because a logged-edge fold *is* the diff, without the assignment ambiguity.
   Recommend the DECIDED name apply-time edge logging as the fact source.
3. **Dissolution's `to` material.** A `→ Void` fact keeps the source material as its
   `to.0` label (convention: "this carbonate dissolved"), since void has no
   material. Reasonable, but the DECIDED does not say; worth pinning.

### 7.7 Gate results (keystone, on merged main `1a16e67`)

- `cargo fmt --all --check` — **PASS**.
- `cargo clippy --workspace --all-targets --release -- -D warnings` — **PASS**
  (`cargo clean -p dc-worldgen --release` first; `Checking dc-worldgen …
  \agent-a440caba50b9545b5\crates\dc-worldgen` verified in the log).
- `cargo test --workspace --release` — **PASS** (`cargo clean -p dc-worldgen`
  first; `Compiling dc-worldgen …` from this worktree verified).

New/updated tests by name — `inventory.rs` unit (8):
`identity_default_commit_appends_no_facts_and_leaves_the_record`,
`base_composition_matches_the_collapse_tier_routing`,
`non_identity_a_known_edge_commits_a_fact_that_re_derives`,
`dissolution_edge_commits_a_to_void_fact`,
`move_form_is_a_mass_neutral_read_modify_write_edge`,
`substrate_accommodates_fluid_without_building_it`,
`eighths_appear_only_at_the_quantize_step`, `sizes_are_pinned`.
`tests/s17_deep_cell_inventory.rs` (3):
`identity_default_appends_no_facts_over_a_whole_field`,
`non_identity_agreement_on_a_real_cell`,
`memory_measurement_per_stratum_vs_per_voxel`.

**Sibling builds live during the gate:** a sibling was building `dc-api`/`dc-client`
on the shared `CARGO_TARGET_DIR`; the same poisoning guard as § 5 (wait for empty
`cargo,rustc,dc-client`, `cargo clean -p dc-worldgen` before each gate build, verify
the worktree path on the `Compiling`/`Checking dc-worldgen` line).
