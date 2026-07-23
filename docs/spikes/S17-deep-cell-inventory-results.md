# S17 — Deep-cell material inventory (spike results)

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
