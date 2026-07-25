# 0106 — the helper named `production`

*2026-07-25.*

Two things happened in this slice and they are related by one idea: **a name is a
claim, and an unchecked claim is a defect that outlives the person who wrote it.**
The first is the repair of `production_field()`. The second is the R3 measurement
for the "weathering is one process" arc, which exists precisely so that arc's cost
claim is a *number* before it is a plan.

> blogworthy: **AI-native development** (lens 1) — the guard was written to stop an
> unverified magnitude claim, and it was itself unverified about *which world it
> ran on*. The failure mode is not laziness; it is that a fixture's name was
> accepted as evidence by every subsequent reader, human and agent alike.
> Also **reflexions in a deepsim codebase** (lens 3): the R3 half is what it looks
> like to price a design before committing to it, using only the substrate's public
> API and refusing to build any of it.

---

## Part 1 — `production_field()` built a world nobody ships

### The defect

`crates/dc-worldgen/tests/geotherm.rs` opened with this:

```rust
const SEED: u64 = 0x0B0A_57EE_0059;

fn production_field() -> DeepField {
    let pregen = Pregen::run(WorldParams { seed: SEED, extent: Extent::Small });
    deeptime::build_field(&pregen.grid, SEED)
}
```

`dc-client` boots `BENCH_SEED = 1337` (`dc-client/src/bench.rs:17`) at
`WORLDGEN_EXTENT = Extent::Medium` (`authority.rs:58`). **Neither the seed nor the
extent matched.** The helper was named for an environment it was not.

Resting on it was `the_geotherm_coal_shift_is_plausible_not_degenerate` — an A-3
guard written *specifically* so journal/0093's coal recalibration could not be
believed on an unverified magnitude. It was green. It was also about a different
planet. On the world the player actually walks there are **zero coal units** across
297 025 deep cells, against 27 134 peat units; the hottest coalification candidate
is 15.4 °C against a `COAL_ONSET_C` of 22 °C (corrections #51, measured by
`examples/coal_walk_tour.rs`).

The user's reaction was the right one: *"it's baffling and somewhat hilarious that
it invokes 'production' and lies."*

### Why this is a distinct species of A-3

`docs/spines.md` § A-3 collects tests that are green for a reason unrelated to what
they assert, and its existing instances are all *mechanical*: a stale artifact
served as fresh (corrections #27), a function pointer folded by the optimizer
(#32), an example that `cargo test` builds and never runs (journal/0103). Those
share a shape — the code did not run.

This one is nastier, because **the code ran and it passed.** The assertion was
sound, the arithmetic was right, the world was real. The only thing wrong was
*which* world, and that fact was carried entirely by an identifier. Nothing in the
type system, the harness, or the gate can see the difference between a `DeepField`
of seed 1337 and a `DeepField` of seed `0x0B0A57EE0059`. The only instrument that
could have caught it is a reader asking "which world did this run on?" — and the
helper's name was engineered, accidentally, to stop anyone asking.

That is why the repair is not "change the seed and move on". It is a shape:

> **A helper named for an environment must BE that environment.** Every future
> claim routed through it inherits its lie, and a claim's blast radius is the
> lifetime of the fixture, not the lifetime of the test.

Recorded as a new A-3 instance in `docs/spines.md`.

### The repair, and the part that had to stay red-adjacent

Making the name true was the easy half:

```rust
const PRODUCTION_SEED: u64 = 1337;                 // dc-client/src/bench.rs
const PRODUCTION_EXTENT: Extent = Extent::Medium;  // dc-client/src/authority.rs
```

memoized in a `LazyLock` because a Medium deep run is tens of seconds and more than
one test reads it.

The interesting half is what happens to the guard. On seed 1337 there is **no
coal**, so the guard's central assertion — `geotherm_coal > 0` — is *false on the
world we ship*. There were three tempting wrong moves and all three were refused:

1. **Lower `COAL_ONSET_C`** until coal appears. Explicitly forbidden: coal is a
   blessed placeholder, and the user's standing instruction is that *"after biology
   we will recalibrate, but not by tweaking coal onset temp — our default onset temp
   will match real earth numbers."* Tuning a constant so a test goes green is the
   purest form of the thing this whole journal exists to avoid.
2. **Delete the guard.** It is the only thing standing between us and stub #14's
   all-to-coal degeneracy.
3. **Weaken it into vacuity** — `assert!(coal >= 0)` and a shrug. That is a guard
   that has stopped being evidence while continuing to look like evidence, which is
   worse than no guard.

The fourth option is the one that was taken: **the guard was asserting two different
claims at once, and they belong on two different worlds.**

- The claim *"coalification is governed by the temperature field"* is a claim about
  the **shipped** world. It does not need coal to exist. It needs the mechanism to
  be wired, fed, and sole.
- The claim *"coal follows the warm crust"* is a claim about the **mechanism**. It
  needs a world with coal in it, and it should say so in its own name.

So:

**`the_geotherm_rule_governs_coalification_on_the_production_world`** — seed 1337,
Medium. Asserts that the `temperature` field is populated; that the world grows
coalification candidates at all (26 845 of them, so a null is a fact about
temperature and not about an empty record); that it does not degenerate; and — the
load-bearing one — that **every** candidate's coal state agrees, unit for unit,
with `T(surface, gradient, mid-slab depth) ≥ COAL_ONSET_C`.

That last assertion is what "responds to gradient" looks like when you cannot
observe the response. It re-derives the rule from the exported `temperature` plane
exactly as `DeepStrata::promote_coal` derives it and demands zero disagreements
across all 26 845 candidates. Move the gradient field and the coal set must move
with it, because *nothing else decides it* — a second promotion path, a stale
threshold, or a rank rule quietly reading burial depth again would each break it.
It is bit-exact, and it holds. Notably, it would still be a real assertion on a
world *with* coal, which is the test of whether a claim is honest or merely
survivable.

It also prints the onset sensitivity curve on every run:

```
candidate T (C): p05=3.4 p25=4.4 p50=6.1 p75=8.6 p90=11.8 p95=13.1 max=15.4
coal fraction vs trial onset:  4C:87%  8C:31%  12C:9%  16C:0%  20C:0%  22C:0%
production headroom: hottest candidate 15.4 C vs onset 22 C (-6.6 C); coal 0 / 26845
```

That curve is corrections #51 lesson 3 made permanent: *a threshold calibrated on
one world needs its sensitivity reported, not just its value.* `COAL_ONSET_C` sits
on a cliff, the curve was computable at calibration time, and printing it costs
nothing.

**`coal_follows_the_warm_crust_on_the_warm_reference_world`** — `0x0D5EED572026`,
Medium, built by a helper called `warm_reference_field()` whose doc comment's first
job is to say *this is not production*. Asserts coal exists (2 379 units of 31 800
candidates), is not degenerate, and that coal units sit on hotter crust than the
peat that stayed peat: **42.8 °C/km against 31.7**. That is the relocation
journal/0093 claimed, measured, on a world named for what it is.

A non-production fixture is fine. A non-production fixture *called* production is
not.

The old `medium_onset_for_thick_coal` diagnostic already ran on this world; it is
now `warm_reference_onset_for_thick_coal`, same world, honest name.

### The cost, stated

The suite went from two Small deep runs to two Medium ones: **63 s**, up roughly
55 s. That is the price of the guard being about the world we ship, and it is worth
paying — this is exactly the case CLAUDE.md's "size the test, not the report" rule
carves out, because the claim *is* about production scale and seed. (`COAL_ONSET_C`'s
own doc comment still calls the warm reference world "the production Medium world";
that sentence is now flagged in the diagnostic's doc comment rather than silently
inherited.)

### The audit for siblings

Grepping the test corpus for `production*` / `prod*` helpers that build a
non-shipped world turned up:

| helper | world | verdict |
| --- | --- | --- |
| `tests/geotherm.rs::production_field` | `0x0B0A57EE0059` / Small | **the defect — fixed** |
| `tests/providers_common/mod.rs::production_field` / `production_pregen` | `0x0B0A57EE0059` / Small | misnamed, **claim is sound** |
| `tests/rh_unification.rs::production_field` | `0x0B0A57EE0059` / Small | misnamed, **claim is sound** |
| `tests/s18_first_behavior_weathering.rs::production_scale_saprolite_band_…` | **1337 / Medium** | honest — the shape to copy |
| `examples/weathering_profile_probe.rs::production_world(extent)` | 1337, extent a parameter | honest |
| `deeptime::field::production_config` · `water::coarse::production()` | — | names a **config**, not a world |

The two `providers_common` / `rh_unification` cases deserve a note, because the
answer is *not* "rename everything that says production". What they assert — that a
refactor is byte-identical to its goldens, and that a derived view equals the scalar
plane it replaces — is **genuinely seed-independent**. Any fixed cheap world proves
it. Re-seeding them to 1337/Medium would move the goldens and add tens of seconds to
the gate to prove nothing new. The defect there is purely the *name*, and the fix is
`golden_*`, which ripples into `providers_golden.rs` and comments in
`flux_record.rs` / `head_field.rs` — three files three live siblings are editing
today. So they got a loud doc comment naming the world, naming corrections #51, and
saying **"must not be used to accept a magnitude, a count, or any claim about what a
player will find"**, and the rename is sequenced.

The distinction that matters: **a fixture's world only has to be production when the
claim is about the world.** Byte-identity is a claim about code. Coal is a claim
about the world.

---

## Part 2 — R3: what would per-depth weathering cost?

### The question

ROADMAP Sequenced carries *"WEATHERING IS ONE PROCESS — SAPROLITE IS A STATE ALONG
IT, NOT A SLICE"*, a user leaning strong enough that it enters sequence the moment
its requisites are met. R1 (the `head` potential field, journal/0098) and R2 (real
vertical flux) are met. R3 is the last one and it is not an argument, it is a
measurement:

> per-depth × per-cell × per-epoch over 297 k cells with multi-slot columns is far
> larger than today's per-cell scalar, and the deep run is already 35–85 s.

Today `dc:deep/weather_inventory` fires once per subaerial cell per epoch and
weathers exactly one thing: the materialized bedrock seam. Its per-cell cost is
**O(1) in the record's depth** — `weather_bedrock_epoch` builds its working
inventory over an *empty* record, which is journal/0094's span-index crux paying an
unintended dividend. Under the arc the same firing becomes a rate evaluated at
**every stratum slot**, i.e. O(slots).

`examples/perdepth_weathering_cost_probe.rs` prices that, and it **builds no part of
the change**. The per-depth "firing" in it is a cost model living in `examples/`,
where it cannot be mistaken for a pass, written entirely against the substrate's
existing public API (`build_working` / `InvCtx::move_form` / `commit_chapter`).

### The causally-triangular insight, and whether it applies

It does, and it is worth more than expected.

A stratum slot deposited in chapter `c` **cannot weather before `c`** — it does not
exist yet. The record is appended bottom-up and never re-ordered, so `DepUnit`'s
`chapter` tag makes the slots-present-at-chapter-`c` set an exact **prefix** of the
final record. The per-depth pass at epoch `e` therefore sees a triangle, not a
rectangle, and the probe counts the triangle exactly rather than assuming the
rectangle.

### The numbers

Production world, seed 1337, `Extent::Medium`, 297 025 deep cells, 200 epochs,
8 tectonic chapters:

```
--- 1. the tuple counts ---
  loop visits (cell x epoch, gate included)              59405000
  TODAY  (cell, epoch) firings                             344410 .. 8610250 (lo/hi)
  PER-DEPTH (cell, slot, chapter), triangular            20487597
  PER-DEPTH (cell, slot, chapter), flat                  33415801
  => MULTIPLIER  triangular 59.49x   flat 97.02x   (the causal triangle saves 38.7%)

--- 2. per-invocation wall clock (240600 firings each) ---
  today (one bedrock span)                552 ns
  per-depth, triangular                  3066 ns   (5.6x)
  per-depth, flat                        5150 ns   (9.3x)

--- 3. gen time ---
  deep run, pass ON                      25.7 s
  deep run, pass OFF                     21.2 s
  => the pass itself                      4.5 s  (17.6% of the run)
  => per-depth (triangular)              25.2 s  (+20.7 s on the deep run, 25.7 s -> 46.4 s)
  => per-depth (flat, no triangle)       42.4 s  (+37.8 s)

--- 4. residency (the LedgerField sidecar) ---
  today      facts    1033189  slot rows      72006  =    17.45 MiB  (measured 17.45 MiB, EXACT)
  per-depth  facts   61460352  slot rows    4516541  =   973.40 MiB  (+955.95 MiB, 55.8x)
```

### Reading them

**The multiplier is 59.5×, and it is not the cost multiplier.** That gap is the
finding. A per-depth pass would visit **59.5 slots per firing on average** — the
weathering cells' columns are deep, and the 25 epochs per chapter cancel out of the
ratio exactly, so that number is not an artifact of chapter resolution. But the
*wall clock* only goes up **5.6×**. Today's firing carries a large fixed cost — it
re-derives a working inventory from `base + facts` and drains a commit log even
though the inventory has exactly one span — and the marginal cost of a slot is
small beside it. **A 59× increase in work is a 5.6× increase in time**, because the
work being multiplied is the cheap part.

That is the sort of thing that is only knowable by measuring. The R3 requisite was
written expecting "far larger", and on the tuple axis it is; on the axis that
actually matters for the gen clock it is an order of magnitude milder.

**The causal triangle is worth 38.7 % of the tuples and 40 % of the time.** A slot
deposited in chapter `c` is invisible to every firing before `c`, and honouring that
is the difference between +20.7 s and +37.8 s. It is also free: the record already
carries `DepUnit::chapter`, and a per-depth pass walking the live record at epoch
`e` gets the triangle automatically — you have to work to *lose* it. Worth stating
explicitly in the arc's design so nobody implements the rectangle.

**Gen time is not the blocker.** The deep run goes 25.7 s → **46.4 s**. Under this
project's standing doctrine — *gen time is not a constraint, ready-made worlds are
the sanctioned answer; runtime is sacred* — that is a shrug. This probe expected to
be the reason R3 gated the arc. It is not.

**Residency is the blocker, and it is a big one.** The `LedgerField` sidecar goes
**17.45 MiB → 973 MiB**, a 55.8× jump, and it is *resident*: it ships inside the
`DeepField`. Nearly a gigabyte of weathering facts for a Medium world is not a cost
to absorb; it is a design constraint the arc has to answer before it is built.

The itemisation says exactly where it goes. Today the pass commits **3.0 facts per
(cell, chapter) firing** — one per agent (chemical, biotic, frost), each 16 bytes,
merged within the chapter by `commit_chapter`. Per-depth, that becomes 3 facts per
**(cell, slot, chapter)**, and 61.5 M facts is simply 20.5 M visited slots × 3.
So the levers are visible and they are axes, not micro-optimisations:

- **drop the chapter axis** for weathering facts (merge a slot's whole history into
  one fact per agent): ÷ ~4.8 (the mean chapters-fired per weathering cell) →
  ≈ 200 MiB;
- **drop the agent axis** in the persisted form (keep the sum; agents are a
  gen-time decomposition, and `Σ share_a = rate` by construction): a further ÷ 3 →
  ≈ 68 MiB;
- **or persist a per-slot scalar rather than facts at all**, which is what the
  collapse consumer actually reads (`weathering_product_m`) — and which is the same
  "a summary must be derived from the authority, never become it" question the
  project already has a doctrine for.

None of those is a decision this probe gets to make. What it can say is: **R3's
answer is that per-depth weathering is affordable in time and expensive in space,
and the space cost is concentrated in the fact ledger's chapter × agent axes** —
which is a much more tractable problem than "the deep run doubles".

An honest residual on the residency figure: it assumes every visited slot ends up
carrying facts. Slots that weather to below `EPS` commit nothing, so the real
number is somewhat lower — but slots are visited *because* they are exposed to
reactant, so the discount is not the order of magnitude that would change the
conclusion.

One incidental confirmation: **only 72 006 of 297 025 cells (24.2 %) ever weather**,
which is journal/0102's 75.8 %-never-weather figure re-measured from the other
direction, a year of slices later. The CSR layout it motivated is still earning its
keep, and it is the reason today's sidecar is 17 MiB instead of 31.

### What the probe does not model, deliberately

The arc's own constraint is that *one process must not mean one pass*: reactant
**transport is a FIELD**, the **edge is CELLULAR**. The probe prices the **cellular**
half only — which is the correct half, because it is the one that multiplies by
depth. A transport field is per-cell-per-epoch, i.e. today's cost class; it changes
the constant, not the exponent. That split is why the projection below is a
projection of the expensive half and not an underestimate of the whole.

The probe also does not model the *value* of the drivers (biotic and frost
multipliers are gen-time planes, not exported on the field, and are held at their
1.0 identity): the shape of the work is what costs, not the numbers flowing through
it.

One honest residual: units eroded away before the end of the run are gone from the
final record, so the reconstructed per-chapter depth is a **lower bound** on what a
firing actually saw. On a world whose columns are net-depositional this is small;
it is stated rather than papered over.

### The gate

Per journal/0103 the probe carries `[[example]] test = true`, and its gate runs at
`Extent::Small` with the pass-OFF baseline skipped. Every assertion in it is
**scale-free by construction** — relations between counts derived from one record,
not magnitudes:

- the residency **itemisation reconstructs the record's own measured
  `footprint_bytes()` byte-for-byte** (`finalize_ledgers` exact-sizes every array, so
  there is no capacity slack to hide an error in). This is the assertion that fires
  if `Fact`, `SlotRun` or the CSR layout moves under the probe — the `flow_cost_probe`
  failure mode, which happened twice;
- the per-depth visit count is `≥` today's (a firing cannot see fewer slots than
  the one bedrock span);
- the triangular count is **strictly below** the flat one — the causal triangle is a
  measured saving, not a rhetorical one;
- both timing arms did non-zero work, so the cost model cannot silently become a
  no-op.

No assertion is pinned to a MiB figure or a nanosecond count. Those are what `main`
is for.

**Measured gate cost: +8.8 s** (`gate::the_perdepth_projection_is_self_consistent`),
against the 35.0 s the seven existing converted probes add.

---

## The gate, by name

```
cargo fmt --all --check                                              clean
cargo clippy -p dc-worldgen --all-targets --release -- -D warnings   clean
cargo clean -p dc-worldgen --release   (then, so the run is honest)

tests/geotherm.rs                                       3 passed, 1 ignored (55.8 s)
  the_temperature_field_is_populated_and_varies
  the_geotherm_rule_governs_coalification_on_the_production_world
  coal_follows_the_warm_crust_on_the_warm_reference_world
  warm_reference_onset_for_thick_coal                   (ignored, diagnostic)
examples/perdepth_weathering_cost_probe.rs              1 passed (8.8 s)
  gate::the_perdepth_projection_is_self_consistent
tests/providers_golden.rs                               2 passed (6.9 s)
tests/rh_unification.rs                                 2 passed (3.6 s)
```

The `cargo clean -p dc-worldgen --release` is not decoration — three siblings share
this `CARGO_TARGET_DIR` today, and corrections #27's false green is exactly the
failure this journal entry is otherwise about, one layer down.
