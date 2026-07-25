# 0103 — The gate that could not see its own instruments, and a mass claim that turned out to be a measurement

*2026-07-25 · background agent, worktree · two jobs that turned out to be the same job*

> blogworthy: **AI-native development** — a green gate is a claim about the code
> it actually ran, and "664 tests passed" was true while a broken instrument sat
> in the same repo printing wrong numbers into the docs. Also **reflexions in a
> deepsim codebase**: the difference between an *estimator* and a *quantizer*,
> and why measuring the wrong one produces a confident wrong diagnosis.

## Job 1 — the gate could not see an instrument fail

`cargo test --workspace` **builds** examples. It never **runs** them.

That is documented Cargo behaviour and it is easy to nod at. What it means for
this project is not: our measurement instruments — the residency probes, the
acceptance probes, the tour maps — all live in `examples/`, because they need to
build a production world and print a report a human reads. Several of them make
hard claims. One of them, `dc-client/examples/identify_census.rs`, carried a
literal `assert_eq!` **inside `main`**, which no gate on this machine could ever
reach.

The proof landed the same day: `flow_cost_probe` was broken by the FLOW slice-1
merge — its itemised residency baseline lost track of the flux record, off by
42.6 MB — and sat green through a full 664-test workspace gate. Not because the
gate was lenient. Because the gate never executed it.

The rule that follows is short: **an example that can fail belongs in the gate.**
The interesting part is *how*, because the obvious answers are both bad. Moving
the probe's logic into the library pollutes a shipping crate with measurement
code that exists for a report. Duplicating the logic into `tests/` gives you two
copies that drift, which is this project's characteristic failure (spines § A-1)
wearing a lab coat.

The answer was already in Cargo, unused:

```toml
[[example]]
name = "weathering_profile_probe"
test = true
```

With `test = true` Cargo builds the example **twice** — once normally, so
`cargo run --example` still executes `main` and prints the whole report, and once
with the libtest harness, so its `#[test]` functions run under `cargo test`. One
file. One set of measurement functions. Two consumers, neither of which is a copy
of the other. The example prints; the `mod gate` asserts; they call the same code.

### Sizing the test rather than the report

The design constraint was real: several of these probes build seed 1337 at
`Extent::Medium` and take 20–90 seconds. A gate that grows by five minutes will
get worked around, and a worked-around gate is worse than an honest gap.

So every converted probe runs its **test** at `Extent::Small` and says, in the
test's own doc comment, **why that is legitimate**:

- `identify(pos)` never reporting air over solid ground is a **per-voxel
  predicate over a per-voxel fact**. Grid width buys samples, not semantics.
- The flux record holding a junction a receiver tree cannot represent is a
  statement about the **primitive**. A tree's divergence count is identically
  zero at every scale, so any non-zero count falsifies "this is still a tree".
- The weathering front's mass identity is **one column's arithmetic**, and a
  column does not know how wide the grid is.

The *production numbers* — 307,364 vertical crossings, 702 → 0 phantom voxels,
the MiB — remain the example's job at `Extent::Medium`, where they belong. The
gate checks the claim; the report carries the magnitude. That distinction is the
whole trick, and it is the reason the conversion cost what it did rather than
five minutes.

### A caption is a published claim, and the gate cannot check it either

While converting `flux_record_probe` the integrator caught a live specimen.
The probe printed:

> `vertical (slot<->slot) : 307364  <- structurally present, honestly EMPTY (no infiltration term in this solve; heirs: the head field + the free/bound edge)`

The named heir **had landed** — `dc:field/head`, journal/0098, on by default — so
the probe was printing a non-zero count directly beside the word EMPTY, and had
been for a day. Nothing failed. Nothing could: no gate runs an example, and no
test asserts on a `println!`. This is the same blindness, one layer up, and it is
worth naming because the caption is the part that gets **quoted into documents**.
When a slice fills a hole that a probe narrates, the caption is part of the diff.

## Job 2 — is the front's voxel-tier mass error noise or bias?

### The question, and why 21 columns could not answer it

journal/0099 shipped the weathering **profile**: the same mass the deep ledger
committed, graded down a front instead of emplaced as one slab. Record-tier
conservation is exact and asserted. At the voxel tier the slice reported
**+3.7 % over 21 columns** and called it quantization noise under journal/0055's
unbiased-estimator doctrine.

The integrator's review did not accept it, and was right not to. An unbiased
estimator has `E[expressed] = owed`; a population mean should trend to **zero**,
not to +3.7 %. Worse, the **median column was +16 %** — and a median is not moved
by variance. That is the signature of a **floor effect**: a band thinner than one
eighth cannot express as less than one eighth without vanishing entirely, so
every thin front rounds the only direction it can.

### The first thing that had to change was the measurement

The probe's population check compared, per column, "product metres the record
owes" against "product eighths counted in the voxels". Both look like the same
quantity. They are not, and two separate problems hide in the gap.

**Problem one: there are two quantizers, not one, and only one of them is an
estimator.**

1. **The fill geometry.** `ColumnFill::build` slices the record into whole voxels.
   The record's bottom lands mid-voxel; that last voxel is claimed
   round-to-nearest and its shares are then *renormalized over the covered part*,
   so a claimed bottom voxel expresses a **full** voxel of a band the record only
   partly covers. This is not an estimator with variance to hide behind — it is a
   rounding. And because the weathering front is emplaced at the very bottom of
   the record, **the whole of that error lands on the front, every time, and on
   nothing else.**
2. **The draw.** Inside each voxel the eighths are allocated stochastically. *This*
   is the estimator journal/0055's doctrine is about.

Lumping them into one percentage means a defect in either is invisible if the
other is loud. The probe now reports `owed → plan → expressed` as two separate
stages.

**Problem two, and this is the one that produced the wrong diagnosis: counting
materials in a finished voxel does not tell you which event put them there.**

The weathering product is `CLASS_CLASTIC_FINE` — mudstone, in this world. So is
much of the sediment pile sitting directly **on top of** the front. At the top
contact the two share a `Mixed` voxel, and a census that counts mudstone eighths
attributes the overlying bed's mudstone to the front.

The scale of this is not marginal. Over 247 production columns, **222 of them**
contain at least one front voxel whose plan holds a non-front event made of the
product's own material. The probe now detects that per voxel and excludes those
voxels from **both** sides of the comparison, so the remaining figure compares
like with like.

### The verdict: **NOISE**, and the bias was in the instrument

Production world, seed 1337, `Extent::Medium`, `weather_inventory` ON. **247
columns** sampled across the whole band distribution (journal/0099 had 21), 2 077
front voxels.

```
                                   aggregate      per column
                                                mean    median      p5     p95
naive  record -> voxels             +6.76 %   +12.59 %  +6.01 %  -13.80 %  +65.56 %
stage 1  record -> fill geometry    -0.02 %    +0.03 %  +0.00 %   -1.22 %   +0.86 %
stage 2  the draw, attributable     -0.60 %    -0.10 %  -0.00 %  -30.00 %  +28.72 %
         (Mixed-plan voxels only)   -0.84 %
```

Stratified by front magnitude, in eighths of a voxel of owed product — the axis a
floor effect must live on, because the floor *is* one eighth:

```
owed product        N    naive mean   stage-1 mean   stage-2 mean   stage-2 median
  1 – 2 eighths     2      -43.57 %        +0.00 %       -43.57 %         +12.86 %
  2 – 4 eighths    13      +76.14 %        +2.04 %        +9.08 %         +20.56 %
  4 – 8 eighths    23      +30.64 %        -0.59 %        +2.88 %         -12.31 %
  8 – 24 eighths   91      +10.79 %        -0.03 %        -0.07 %          +0.27 %
    >= 24 eighths 118       +4.41 %        -0.02 %        -0.74 %          -0.43 %
```

Read the **naive** column and the floor-effect story is perfect: +76 % on the
thinnest fronts decaying monotonically to +4 % on the thickest, exactly what a
one-sided quantum would produce. Read **stage 2** — the same columns, with the
overlying bed's mudstone no longer counted as the front's — and the trend is
gone. What is left is a distribution centred on zero (mean −0.10 %, median
−0.00 %) with a symmetric spread (p5 −30 %, p95 +29 %) that **widens** as the
front thins, which is exactly what an unbiased estimator over a quantum of one
eighth does: a front owing two eighths cannot be wrong by less than ±50 %, and it
is wrong in **both** directions.

The naive trend was not a floor effect. It was **the contact voxel's share of the
front shrinking as the front grows**. A thin front is mostly contact; a thick one
is mostly interior. The contaminated top voxel therefore contributed a large
fraction of a thin front's count and a small fraction of a thick one's, and that
produced a clean monotone curve in a quantity that had nothing to do with
quantization.

**So: noise, not bias.** journal/0055's unbiased-estimator doctrine is **not
falsified** — it is confirmed, and the arithmetic below shows why it had to be.
What is falsified is journal/0099's *number*: the front's voxel tier does not run
+3.7 % hot; over the 89 % of front voxels whose product can be attributed it runs
**−0.6 %**, and the residual is the estimator's variance, not its mean. Recorded
in corrections.md.

Two honest limits on that:

1. **10.7 % of front voxels (222 of 2 077) are excluded** from the attributable
   figure, and they are not a random 10.7 % — they are systematically the *top
   contact* voxels. Nothing in a finished voxel says which event put a mudstone
   eighth there, so that share is not directly measurable by this instrument at
   all. The inference rests on stage 1 covering **100 %** of voxels and coming
   out flat, plus the code-level argument that the draw is unbiased by
   construction.
2. **The +16 % median that opened the question was a 21-column median.** At 247
   columns the naive median is already +6.0 %. The number was moving with `N` the
   whole time, which is itself the small-sample signature the review suspected.

### The generalisation, and it matters for flow.md § 3

**Voxel contents carry no provenance.** A finished voxel is a multiset of
materials; it does not record which recorded event contributed which eighth. So
*any* conservation audit at the voxel tier that works by counting materials is
measuring "how much of material M is here", not "how much of material M did
source S put here" — and those differ by exactly however much of M the
neighbours are made of.

flow.md § 3's mass budget is a conservation audit at the voxel tier. It can
safely build on this front — the expression is unbiased — **provided it compares
against the fill plan rather than against a material census.** The plan knows
which event owns which fraction of which voxel; the voxel does not. Writing that
down is worth more than the percentage.

## What the code actually says about the rounding

The brief asked for this from the code, not from the numbers, and it is worth
having on the record because it is the part that will still be true after the
constants move.

**`fill::allocate` / `allocate_to` is systematic sampling — a Cranley–Patterson
rotation — and it is exactly unbiased.** Each material's fractional remainder is
laid end to end on a line and one uniform offset `uq` marks the integer
crossings:

```rust
let extra = ((c_next + uq) >> FRAC_BITS) - ((c + uq) >> FRAC_BITS);
```

For a remainder `r`, `P(extra = 1) = r / ONE` exactly, so `E[n_i] = w_i / ONE`.
The crossings of a segment of integer total length number exactly that integer,
so the eighths sum to eight as an identity rather than a fixup. **There is no
minimum share and nothing rounds up at the floor** — a material with a tiny
remainder gets its eighth with probability equal to that remainder, which is the
entire point of choosing this over deterministic flooring.

**`allocate_partial` adds a rescale, and the rescale is a deterministic
cumulative floor** — `c = floor(cum · target / TOTAL)`, monotone-clamped — before
handing off to the same `allocate_to`. Because it floors the *cumulative* rather
than each share, a share's error is the difference of two floors along a monotone
run: the errors cancel down the run instead of accumulating, and the residual is
under one ulp of a 20-bit fixed-point eighth. Not a floor effect.

**`pore_rider_share` is stochastic-proportional and unbiased too.**

```rust
let uq = (u * 4096.0) as u64 & 7;
let n = (u64::from(cnt) * u64::from(k8.min(8)) + uq) / 8;
```

Writing `C = cnt · k8 = 8q + r`, the sum of `floor((C + uq) / 8)` over
`uq ∈ {0..7}` is `8q + r`, so the mean is exactly `C / 8`. The `.min(cnt)` clamp
can only bite at `k8 = 8`, and the weathering profile caps at 7 — saprolite is
*defined* by retained parent fabric — so it is inert on this path.

**The one genuine floor in the expression path is elsewhere and is inert here.**
`contents_for_event` does `let k = eighths.clamp(1, 7)` — a recorded 0-eighth
accessory would express as 1/8. `WEATHERING_PROFILE` is `[7,5,4,3,2,1,1,1]`, so
no front band ever reaches the clamp. It is a floor, it is real, and it is not
this front's.

**One thing the code says that is worth a second look, and is not a mass
problem.** `pore_rider_share`'s comment says it takes "a **low digit** of the
voxel's own fill draw, not its high bits: `allocate_partial` consumes the high
end". `(u * 4096.0) as u64 & 7` is bits 10–12 of the fraction; `allocate_to`'s
`uq` is the top **20**. They overlap, so the two offsets are not independent —
`cnt` and the rider's `uq` are correlated. Each estimator is still marginally
unbiased, and the measurement below is the empirical answer for the joint case;
but the comment claims a disjointness the arithmetic does not have. Filed rather
than fixed: changing it moves every contact voxel in the world, and that is the
user's call.

## Shape compliance

Job 1 guards **A-3** head on: a gate that structurally cannot see a failure is
green for a reason unrelated to correctness, and this one had already produced a
false negative in the wild. It also rides the doctrine that *a thing which can
fail should be run by the gate* — and the conversion deliberately did **not**
build a parallel test harness beside the probes (A-1); it used the target flag
Cargo already had.

Job 2 guards **A-1 and A-5**: *"mass is conserved"* must not enter the corpus as
a stronger claim than the evidence supports, and an assumed cause must not be
reported as a diagnosis. The first pass at this thread nearly did both — the
floor-effect hypothesis was plausible, the numbers appeared to support it, and it
was the measurement that was wrong.
