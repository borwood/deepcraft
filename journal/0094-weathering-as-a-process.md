# 0094 — Weathering as a PROCESS, not a snapshot

*(Movement 3 of the weathering arc. Discharges stub #17. Rides the north-star
pass-runner; the material-transformation half of material-behavior.md §5.)*

> blogworthy: **reflexions in a deepsim codebase** (lens 3) — the difference
> between running a rate *once over the end-state* and running it *inside the
> loop that made the end-state*, and why a keystone can be fully built, fully
> tested, fully consumed, and still model **nothing**. Also lens 2 (procgen
> against priors): a saprolite front is an integral over time; you cannot fake
> the integral with its integrand evaluated at `t = end`.

## The miss this redeems

S18 (journal/0089) built the machinery for inventory weathering — the working
inventory, the fact ledger, the cause-carrying `Structure→Loose` edge, the
collapse fold that lays a basal saprolite band — and wired it end to end. Every
piece was real and tested. Then it **ran the whole thing once**, after the
deep-time run, over the finished record, with the final-state fields frozen
(`field.rs::build_ledgers`). The product was **0.04 m** at production scale —
sub-voxel, zero eighths, invisible (corrections #46/#47, stub #17).

The diagnosis in stub #17 was blunt: *this stub IS the behavior, not a
constant.* Weathering is a **continuous process**. The height tier already knows
this — its `dc:deep/weather` pass runs bedrock→regolith every epoch and
accumulates metres of `H` over the run. The inventory weathering was bolted onto
the *end* instead of living where weathering runs. A snapshot of a process is
not the process, and the magnitude gap (0.04 m vs metres) is exactly the missing
time integral.

Movement 3 moves it inside the loop.

## The move: a one-shot becomes a declared cellular pass

The deep-time loop is, since journal/0090, a runner of self-declaring passes.
Weathering becomes one more: **`dc:deep/weather_inventory`**, a cellular pass
gated behind `cfg.weather_inventory` (absent when off — pass presence *is* the
old `if`, exactly like every other gated pass, so the flag-off world stays
byte-identical). It:

- fires **every epoch** (`period = 1`), after the erosion pipeline settles the
  terrain — its declared `reads` are the contemporaneous settled terrain
  revision (`Settled` on the production agent roster), `Frosted`, and `Exposed`,
  plus **`reads_prev BioMod`** (the biotic multiplier is loop-carried, exactly
  as `weather`/`diffuse` declare it);
- weathers each **subaerial** cell's bedrock seam once per firing on **that
  epoch's live inputs** — the contemporaneous regolith `H` (the `cover_taper`
  shielding), the frost plane, the biotic multiplier — moving a share
  `Structure→Loose` and appending one cause-carrying fact per agent;
- **accumulates** those facts across the whole run into per-cell ledgers threaded
  through `DeepStepCtx`, initialized empty before the loop and handed to the
  `DeepField` at the end (replacing `build_ledgers`, now deleted).

The subaerial gate got *more* honest in the move. The post-hoc version had to
read the **record** ("did this cell ever deposit a subaerial unit?") because
after the loop the only sea stand it knew was the final one — and the final
`surf` sits ≈ −460 m below the datum after isostasy, so a `surf > 0` gate would
have weathered nothing. In the loop we have the **contemporaneous** sea stand
(`ctx.sea_level`) every epoch, so the gate is the real thing: `r + h >
sea_level` *this epoch*. A cell weathers exactly when it stands above the water
that epoch — the same authority the height-tier weathering used all along.

## The design crux: keying facts to a growing record

Here is the subtle part, and it is the heart of the slice.

The bedrock `Structure` seam (stub #16) is the **last span** of a cell's working
inventory, at index `strata.units.len()`. The fact ledger is a `Vec<Vec<Fact>>`
keyed by span/unit index, and the bedrock's facts live at that last slot. Fine
for a one-shot: the record is finished, `units.len()` is fixed.

But **inside the loop the record grows**. The deposition pass appends a unit to
most cells every epoch, so `strata.units.len()` — and therefore the bedrock
seam's *numeric* index — **shifts upward across epochs**. If I keyed the
accumulating bedrock facts to `units.len()`, epoch 40's facts would land at slot
40, epoch 80's at slot 80, and each would later be *reinterpreted* as the facts
of whatever record unit now occupies that index. `N` epochs of accumulation
would scatter across `N` different, wrong slots and compose to garbage.

The fix is to key the accumulator to a **stable bedrock sentinel** that does not
move as the record grows. The accumulator is a **bedrock-only ledger**: built
against an *empty* record (`FactLedger::empty_with_bedrock(&DeepStrata::default())`),
so the bedrock seam sits at slot **0** — and slot 0 of an empty record is
invariant, whatever the real record does. Every firing re-derives the seam from
`base + facts` at that fixed slot, moves more `Structure→Loose`, and commits back
to slot 0. The record's growth never enters.

Only at **loop end** do we re-key: `finalize_ledgers` maps each accumulator's
slot-0 facts onto slot `strata.units.len()` of a record-sized ledger — the exact
slot the collapse consumer reads (`weathering_product_m(deep_units.len())`,
`ledger_at_voxel`). The band composes to precisely the value it had in the
accumulator; the shifting index is confined to a single deterministic remap after
the process is done. `bedrock_facts_key_stably_as_the_record_grows` tests it
directly: accumulate against the sentinel, finalize onto a record grown to seven
units, and assert the band survives the re-key bit-for-bit and lands at the
right slot (not slot 0).

## One fact per agent per firing — and keeping that bounded

The S18 §1 invariant — each agent commits **its own** fact, never a blended one —
is preserved: a single firing yields exactly three facts (chemical, biotic,
frost). But naively accumulated, that is three facts *per epoch*, hundreds per
cell over the run — a fat ledger and a slow collapse-time fold. `commit_chapter`
only coalesces *consecutive* identical edges, and successive firings interleave
the three agents (chem, biotic, frost, chem, …), so nothing merges on its own.
So after each firing the accumulator is coalesced by `(chapter, cause, edge)`:
one fact per agent **per tectonic chapter**, bounded to `3 × chapters`. The band
(Σ fractions) is invariant to the merge — coalescing changes only the fact count,
never the composed `Loose`. Provenance stays per-agent, at chapter granularity.

## `dt` goes live

journal/0090 threaded `dt` (the pass's phase length) through the runner but left
it **inert** — "Movement 3 makes it live." This is that. Each agent's share now
scales by `dt`: `share_a = cover_taper × driver_a × susceptibility_{m,a} × dt`.
At `period = 1`, `dt = 1.0` and a firing is one epoch, so the production number
is unchanged by the multiply — but the rate is now a real per-pass knob: a
coarser cadence would weather proportionally more per firing, `share ∝ dt`,
principled rather than a magic per-epoch constant. `dt_scales_the_share_linearly`
pins it (dt = 2 moves exactly twice dt = 1).

## Two authorities, held

Per material-behavior.md §11 / journal/0089, the **material** weathering (this
pass, `Structure→Loose` on the inventory ledger) and the **height** weathering
(the existing scalar-`R/H` `dc:deep/weather` pass) are **two different
authorities**, and unifying them is a reserved later arc. Movement 3 honours the
split exactly:

- the pass **reads** the contemporaneous `H` (the regolith cover that shields
  the bedrock — the `cover_taper`), so it "runs every epoch riding `H`";
- it **writes only the ledger sidecar** — its output axis `DeepAxis::Saprolite`
  is a pure write that **no in-epoch pass reads** (mirroring how the geotherm
  plants a field and runs no edges). It never touches `R`/`H`.

Both "rides `H`" and "never touches `R`/`H`" are satisfied by that read/write
split. The proof is structural: `on_flag_is_purely_additive_record_and_surface_untouched`
asserts the erosion `strata`/`surf`/`regolith` are byte-identical flag-on vs
flag-off — the pass perturbs the height sim not at all. The height-tier
`dc:deep/weather` pass was not touched.

**No deviation to raise.** Implementing to the split was clean and nothing
fought it; I did not become convinced it was wrong.

## The number

At production scale — the S18 tour's world (seed 1337, `Extent::Medium`, the
production flag set + `--weather-inventory`, 0.9 m voxels, 200 epochs) — the
accumulated saprolite band at the argmax cell (deep cell 150859) is **6.094 m
= 6.77 voxels (54 eighths)**. **≥ 1 voxel: PASS.** That is the S18 0.04 m miss
redeemed: **~152× the one-shot**, because the integral over the cell's subaerial
epochs — thin-cover early epochs weathering hard, thick-cover late epochs
weathering less — is exactly what the snapshot threw away.
`production_scale_saprolite_band_reaches_at_least_one_voxel` is the A-3 guard:
the band is proven at production scale, in-slice, not on a hand-fed magnitude.

## What this is NOT

It is not the R/H unification (reserved), not a new material axis (the
biotic/frost susceptibility reuse stays the annotated seam from S18), not a
change to the sum-agent rate model (agents still SUM). And the loose product
still inherits **stub #16**'s stand-in bedrock identity — one flat granite
basement, expressed as a fine-clastic saprolite — so #16 is **not** retired here;
the *material* a rind is made of still waits on the genesis/emplacement heir. The
band is a walk-gated appearance flip (default off, byte-identical) — the ≥1-voxel
result is the user's to bless from a screenshot.
