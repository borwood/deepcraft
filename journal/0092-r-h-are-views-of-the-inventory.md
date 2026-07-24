# 0092 — R and H become views of the inventory

**Movement 2a of the R/H unification** (material-behavior.md §13.6, ratified
2026-07-23). The deep cell's working material inventory becomes the **authority**
for surface material; the scalar `R` (bedrock elevation) and `H` (regolith
thickness) planes the erosion loop has always evolved become **derived views** of
it. This is a *representation* change with a hard acceptance instrument: the
production goldens must not move a bit.

> blogworthy (lenses: deepsim-reflexions; procgen-against-priors): the cheapest
> honest way to promote a scalar plane to a derived view is to notice the
> authority was already being maintained — the recorder had been writing the
> reconciled inventory all along, one deposit-fact per epoch.

## The trap: the loop cannot run on the inventory

The obvious reading of "make the inventory the authority" is to have the erosion
passes read and write material through it. That cannot be byte-identical. The
two-plane engine (`erosion.rs`) is a tightly-coupled f64 relaxation — priority
flood, a downstream flux chain, gathered hillslope diffusion — whose result is
pinned to the *bit* by summation order (S9b). Route those metres through a
`(material, form)` multiset and every golden moves, not because the physics
changed but because the additions regroup. The brief anticipated this and named
the ratified shape: **scratch-first reconcile**. The hot per-epoch work stays on
the materialized `R`/`H` planes; plane deltas reconcile into the inventory **as
facts at pass/chapter boundaries**, never per-inner-loop.

## The authority was already there

The reconcile turned out to need no new machinery, because the deposition pass
already *is* it. Every epoch the recorder takes each cell's net ΔH and appends it
to the strata record: `ΔH > 0` is a `deposit` (a `void→Loose` arrival),
`ΔH < 0` an `erode` (a `Loose→void` removal) — the inventory's own edge
primitives — tagged by the `DepTag` measured that epoch, which routes to a
material through the *current* `deep_class`/`litho_of_tag` rule (no new material
rule, no anonymous-dominant stub — the real current routing). journal/0053
already proved the invariant that makes this exact: `Σ unit.thickness == H` at
every point in the run. So the strata record **is** the persistent, reconciled,
ungated (`record: true`) per-cell surface-`Loose` inventory. Recognizing it as
such — rather than building a second structure beside it — is the whole move.

## H derives *positionally* (and it is more honest than the plane)

`H = the Loose above the topmost Structure` — a positional query, not a
whole-column `Loose` sum. `build_working` lays the record's `Loose` cover spans
first and appends the basal bedrock `Structure` seam (STUB #16) last, so scanning
top-to-bottom sums the whole cover and stops at the bedrock contact:
`H_derived = Σ record Loose = grid.h`. The positional rule earns its keep on the
case the scalar plane **cannot represent**: a `Structure` roof over buried
`Loose` — cave fill. The scalar `H` is one number; it would count the buried
loose as surface regolith. The inventory excludes it by position
(`surface_regolith_m` breaks at the first `Structure`). The unification is not a
lossless re-encoding of `H`; it is a *more faithful* one, exactly the gain the
user's cave question predicted.

## R is a datum, not a stock (the finding)

§13.6 writes `R = Σ Structure`. That is the fully-materialized-column ideal, and
the two-plane engine is not there yet: its `R` is a bedrock-top **elevation**
(signed — production `R` runs hundreds of metres below the datum after isostasy),
not a structural thickness. `Σ Structure` over the column is the bedrock seam's
*stock* (a positive quantity), which is a different thing from an elevation. So
`R` derives as the **datum** it has always been: `R = surf − H`, the elevation of
the topmost `Structure` contact, which recovers `grid.r` (= `surf − regolith`) to
the recorder residual. The structural *stock* query
(`derived_structure_stock_m`) exists beside it and reads the seam today; it is
what the fully-materialized column will grow into. This is recorded as a finding,
not forced into a false `Σ Structure == R` that would have moved the plane.

## Byte-identity, measured

The erosion loop, the recorder, and `build_field`'s computation of
`surf`/`regolith`/`strata` are **untouched**, so the golden fingerprints (which
hash exactly those planes and the record) are unmoved by construction —
`the_production_world_still_hashes_to_the_pre_slice_goldens` and
`generated_world_is_byte_identical_to_the_pre_contract_goldens` stay green, as do
`full_agents` / `tectonic_history` / `deep_config_plumbing`. The derived views are
proven against the planes over the *real* production field
(`rh_unification.rs`): `derive_regolith_at` vs the `regolith` plane and
`derive_bedrock_at` vs `surf − regolith`, per cell over the 25 600-cell
production field, **max residual 8.3 × 10⁻⁸ m for both `H` and `R`** — pure f64
round-off between the incrementally-mutated `grid.h` and the record's
`Σ thickness` (they accumulate the same per-epoch deltas from different bases over
200 epochs), 83 nanometres, far below any physical signal. The cave rule is a
named unit test (`buried_loose_below_a_structure_is_excluded_from_surface_h`),
and dc-worldgen's lib suite (87 tests) and the two byte-identity goldens plus
`full_agents` / `tectonic_history` / `deep_config_plumbing` all stayed green.

## Perf shape

Gen-time only — the present VM never runs the deep loop. The reconcile is the
recorder, already paid once per epoch at the boundary (no per-inner-loop
inventory walk). The derived views **materialize on demand** (`build_working`
from the record); no resident per-cell inventory is carried on the `DeepField`,
deliberately — a resident duplicate the collapse does not read would be the
built-but-unconsumed anti-shape (spines A-2). The record already carries the
authority; the views read it when asked. A full-field materialization
(`build_working` from the record for all 25 600 cells, then derive `H`/`R`) is a
small fraction of the agreement suite's 3.4 s (two whole-field passes plus two
production-field rebuilds) — gen-time, free by doctrine. **Resident cost: zero
additional** (no per-cell inventory carried; the record already exists).

## What stays

- **Movement 2b — material-aware transport / sorting** (Hjulström entrainment,
  settling-velocity deposition, grain/density sort). Transport still moves surface
  `Loose` at its current scalar rates; the inventory now gives it a real material
  column to sort *when 2b lands*.
- **Movement 3 — weathering-rate.** The `dt`/rate stays inert (pinned); the
  weather pass is not un-pinned here.

## Shape compliance

North-star: **cell storage is the authority, passes declare their access** —
landed at the deep tier as "the inventory is the authority, `R`/`H` are derived
views the passes read/write through the materialized cache." The crossing
constraint holds: the reconcile edges are plain data (`Portion`/`InvForm`, the
`deposit`/`erode` facts), no closures across the seam. Anti-shapes guarded: the
material stays the **real current `deep_class` rule** (not an anonymous-dominant
stub — the summary-wearing-authority sin), and no resident unconsumed inventory
was carried (A-2).
