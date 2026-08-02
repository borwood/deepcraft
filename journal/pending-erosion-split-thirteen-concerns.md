# The erosion file, re-housed — and what 4,712 lines turned out to be

*Pure re-housing. No behaviour change, no renames, no arithmetic moved, no
"while I'm here" cleanups. The only thing worth writing down is **the cut**, and
what the file turned out to contain once something looked at it structurally.*

> blogworthy: **lens 1 (AI-native development)** — a 4,700-line file is a *unit
> of context*, and the honest way to find its seams is not to read it but to ask
> the symbol graph where the cheap cuts are. **lens 3 (reflexions in a deepsim
> codebase)** — the seams a big simulation file actually has are rarely the ones
> its narrative advertises.

## Why now

`deeptime/erosion.rs` stood at **4,712 lines against the 700-line source
threshold** — 6.7×, the largest single file in the tree. The file-size hook has
been firing on it for a week. The user's ruling on 2026-08-02 was short: *"we
split it now."* Source files split **by concern, on ordinary module boundaries**
(corrections #85 scoped the liveness doctrine to docs only), so this is an
ordinary Rust module directory and nothing more.

## The cut, and why it is thirteen files and not three

The previous builder proposed three concerns — the transport chain, the
diffusion/creep kernel, and the recorder phase. All three are real. They are also
**three of eleven**, and three files of ~1,500 lines each would have left every
one of them over threshold. So the question was never *are those concerns real*
but *what else is in there*.

The answer, by line count, once the file was mapped item by item:

| concern | lines | what it is |
|---|---|---|
| the `Erosion` state + epoch orchestration | 460 | the struct, `new`, `step`, the parallel/sea-level/tectonic knobs |
| the transport chain | 605 | entrainment, capacity, the competence ceiling, deposition, the species split |
| the MFD partition kernel | 243 | the D8 receiver rule, the weight partition, the geometry constants |
| the hybrid-`p` law | 164 | `MfdParams` — the *authored* convergence exponent |
| the routing phase | 252 | receiver/weight planes, drainage accumulation, processing order |
| priority-flood | 204 | the min-heap fill + the S9b parallel probe |
| the creep operator kernel | 441 | the S-10 gather functions, the bound, the explicit step, the audit |
| the creep pass | 210 | the sub-cycling driver and its instrumentation |
| weathering | 313 | exposure, the conversion phase, the frost agent |
| the strata recorder | 211 | facies tagging + the record phase |
| the surface agents | 275 | wind and wave — mass-neutral, self-recording |
| the vertical drivers | 112 | uplift, thickening, isostasy, exhumation |
| the denudation ledger | 247 | every metre that leaves the domain, and the flux-record exports |

Eleven concerns, thirteen files (two concerns each split once more, below). Every
file lands between 129 and 634 lines.

## What made it cheap: the struct stays in `mod.rs`

`Erosion` is one struct with 68 inherent methods and ~60 private fields, and
almost every method touches the fields directly. Rust allows an inherent `impl`
to be **split across files inside a crate**, and — the load-bearing detail — a
private item in a module is visible to that module *and every descendant*. So:

- `struct Erosion` and its private fields stay in `erosion/mod.rs`;
- each concern file carries its own `impl Erosion { … }` with only its methods;
- **the shared grid helpers needed no visibility change whatsoever** —
  `in_grid`, `coords_of`, `is_border`, `NEIGH8`, `NEIGH4`, `SPECIES`,
  `PAR_MIN_CELLS` and `Erosion::par()` all live in `mod.rs` and are reached by
  every child as private items.

Only items crossing **sideways** between siblings needed plumbing, and there were
exactly **22** of them, all `pub(super)`: `Item`, `MFD_DIRS`, `MFD_DIST`,
`REFERENCE_KT`, `ENERGY_LOW_MED`, `ENERGY_MED_HIGH`, `split_by_shares`,
`route_cell`, `partition_cell`, the six creep kernels, and five private methods
(`tally_sink`, `tally_creep_to_sea`, `mfd_neighbour`, `diffuse_step`,
`audit_creep_species`).

## What the file turned out to contain

Three things worth recording, all of which came from a symbol-usage matrix run
over candidate partitions rather than from reading:

1. **The creep code is two concerns with a perfectly clean cut, and nobody knew
   it.** The sub-cycling driver `diffuse` reads `eff_diff` and
   `CREEP_MAX_EDGE_COEFF` and **never touches a gather kernel**. The inner
   `diffuse_step` calls all four gather kernels and **never touches `eff_diff`**.
   The two halves share nothing but the struct. That is the *pass* and the
   *operator* — exactly the seam spines § S-10 says is there, showing up
   independently in the dependency graph. (E4 still owns the extraction; this
   move only puts the kernel in a file whose doc comment says what it is.)

2. **The MFD block is a kernel and an authored law**, coupled by one constant.
   `MfdParams` — the hybrid-`p` exponent, journal/0113, its own stub #26 — reads
   nothing from the partition but `MFD_MIN_WEIGHT`, its default floor. Content
   authoring on one side, arithmetic on the other.

3. **The denudation ledger is a standalone 247-line concern** that had been
   scattered across the file: `TransportLedger`, its two tally sites, and the
   flux-record export accessors, filed under three different neighbours.

## Where "copy-exact" bit

Every line was relocated by script from explicit ranges and then verified by a
second script that reconstructs the original from the new files — **4,542 source
lines across 88 blocks, matched verbatim and contiguously**. The tests moved with
their concern (`mfd_tests` into `mfd.rs`, `creep_tests` into `creep_kernel.rs`,
`hillslope_operator_tests` into `creep.rs`), which kept their `use super::*;`
lines correct without touching them, and avoided the 4-space de-indent that a
separate `tests.rs` file would have forced.

Exactly **five** `super::` paths in 4,712 lines needed retargeting, because
`super::` used to mean `deeptime` and now means `erosion` inside a child: two
code paths in `isostasy` and three doc links. rustfmt then re-wrapped **two**
signatures — both because my own edits (`pub(super) fn diffuse_step`,
`super::super::tectonics::CrustKind`) pushed them past 100 columns. That is the
entire non-mechanical diff.

The gate is a before/after `cargo test -p dc-worldgen --release --no-fail-fast`
compared **by test name and status**, not by exit code: several golden families
are expected red on main awaiting P11 slice 2's capture, and that red set is part
of the baseline. `--no-fail-fast` matters — the default stops at the first failing
binary, which on a red baseline hides most of the suite from the comparison.
