# 0060 — three sockets, and a borrowed binary

*2026-07-22 — the first provider seams: `outcrop_at`, `wave_energy`, `parent_p`.*

## The problem, as it was handed over

Two decisions written a day apart set this up. The first,
**"a summary is not an authority"** (ARCHITECTURE.md, DECIDED 2026-07-21), named
a failure class and found four live instances in one audit: a cheap answer,
written because the real authority did not exist yet, quietly hardening into the
definition of the thing it stood in for. The line that stuck was *"a stand-in
becomes the definition unless something stops it"* — and the uncomfortable part
was the diagnosis of why `stubs.md` had caught none of the four: **a leaked
requirement looks like working code that passes tests.** A stub announces
itself. A leaked requirement does not.

The second, **"the content set is frozen at world creation"** (DECIDED
2026-07-22), said the answer to "who supplies this" is part of a world's
identity, not a runtime lookup.

So the task was not "write an extension system". It was: find the places where
the sim asks a question that an unbuilt system will one day answer, and make the
constant sitting there *visibly a default in a socket with a labelled owner*
rather than the rule. Three conversions, chosen to teach the shape.

## What a provider slot is

`deeptime::providers::Providers` — three plain `fn` pointers, resolved once at
world build, carried in `DeepConfig`, default-identity:

```rust
pub struct Providers {
    pub outcrop_at: fn(Option<&DepUnit>) -> Litho,
    pub wave_energy: fn(WaveCell) -> f64,
    pub parent_p: fn(ParentCell) -> f64,
}
```

The discipline is copied wholesale from `pipeline::PassBody` — deterministic,
no captured state, no closures, no trait objects, `Copy`, trivially registrable
as data. Each slot's doc comment carries three things: the **question**, the
**heir** (the unbuilt system expected to answer it), and the **identity value**
(the constant it holds meanwhile). `Providers::default()` is the identity set,
and the acceptance test is that a default-provider world is bit-for-bit the
pre-seam world.

There is deliberately no registry, no loader, no declaration/validation pass.
Three seams is not enough to design a general mechanism *from* — only enough to
design one *for*, which is the same mistake one level up.

### 1. `outcrop_at` — the seam its own author had already named

`lithology.rs` said, in prose, of `exposed_litho`: *"This is the one function
structural deformation will change… every other part of this module carries over
unaltered."* That is a correct and well-earned sentence, and it is also exactly
what corrections #29 warns about — **prose cannot fail a build**. Converting it
cost nothing conceptually: it was already a function call at four sites (fluvial
incision + creep, periglacial frost, eolian deflation, littoral attack), so the
seam replaces a direct call with an indirect one and changes nothing else. The
identity *is* `exposed_litho`, re-exported under the slot's name so that the
identity is a thing rather than a description of a thing.

### 2. `wave_energy` — the one that was not on any list

The littoral agent cut every shore cell at one global constant. A lee shore
inside a 40 km inland sea and a west-facing ocean coast at 45° S got identical
attack. Wave height is set by **fetch** and **wind**, and this engine has both
within reach — the eolian agent already reads a zonal wind field, and the S11
body graph will know how much open water lies upwind. It was the model's most
conspicuous "one number where a field belongs", and it had **no `stubs.md`
entry**, which is precisely the defect the doctrine's own line ("an unlisted stub
is a defect in this inventory") describes. It has one now (#13).

One thing had to be preserved carefully: `wave_erosion <= 0.0` short-circuits
the whole agent *before* any provider is consulted. That early return is the
byte-identity control `tests/full_agents.rs` already leans on ("flag on, all
three rate knobs zero, must equal the uncoupled run"), and it must stay a
property of the *configured* rate, not of whatever a provider returns per cell.
The seam sits inside the gate, not around it.

### 3. `parent_p` — the one that is not per-cell-per-epoch

Stub 8 said every cell starts with the same rock-phosphorus pool: granite and
basalt pretended equal. It is in this slice for a specific reason — **it is
pass-level, not value-level.** Parent material does not change over the run, so
calling a provider inside the epoch loop would be paying `n × 200` calls for `n`
distinct answers. `BioticSim::new` materializes a plane once; the loop reads it
by index.

Converting it turned up a second-order defect I would not have found by reading:
the constant was used **twice**. Once to seed the pool, and once as the *cap*
that rejuvenation restores a stripped surface toward. Only fixing the first would
have produced a world where a hypothetical phosphorus-poor parent material still
rejuvenated to the phosphorus-rich maximum the moment erosion touched it — a
brand-new leaked requirement, introduced by the very slice meant to stop them.
Both consumers read the plane. **When you socket a constant, grep the constant,
not the call site.**

## The wrong turn: a binary from another commit

The byte-identity requirement was explicit that self-comparison is circular. So
the fingerprints — FNV-1a over every plane the world keeps, and over every tag
axis, thickness bit, unconformity flag and chapter in the record — were captured
*first*, against untouched `main`, before a line of the slice existed. Then
`git worktree add ../preslice 2434f37` and the same test file run against a
genuine pre-slice checkout, which reproduced them independently:

```
surface fingerprint = 0x7B8968FD90E04062
record  fingerprint = 0xA53BD77F769D7FF4
```

Post-slice, with default providers, both land exactly.

And then, while timing the ritual in both worktrees alternately, the harness
reported:

```
running 0 tests
test result: ok. 0 passed; ... 3 filtered out
```

Three tests. My post-slice suite has seven. Cargo had served the **pre-slice
worktree's `providers.exe`** as fresh, because both worktrees share one
`CARGO_TARGET_DIR` — corrections #27, live, in the middle of the one measurement
the whole slice is judged on. It announced itself only because the test *count*
was wrong; had I filtered on `test result: ok` I would have recorded a timing
for the wrong binary and never known. `cargo clean -p dc-worldgen --release`,
re-run, seven tests, six passed one ignored.

> blogworthy: the byte-identity proof that nearly measured the wrong binary —
> why "verify by name and count, never by `test result: ok`" is not pedantry.

## Cost

Ritual = the production deep-time run at `Extent::Medium` (545², 200 epochs,
biotic + erodibility + tectonic history + full agent roster), timed inside the
test, machine otherwise idle, both sides warm:

| | runs | mean |
|---|---|---|
| pre-slice (`2434f37`, its own worktree) | 16.112 / 15.880 / 15.779 / 15.944 | **15.93 s** |
| post-slice (after `cargo clean -p dc-worldgen`) | 15.991 / 16.040 / 15.899 | **15.98 s** |

**+0.3 %**, against a within-set spread of the same order. Under the 2 % bar and
honestly indistinguishable from noise. That is the expected result rather than a
lucky one: `outcrop_at` replaced a call with a call, `wave_energy` is consulted
only in the thin freeboard band, and `parent_p` is not in the loop at all.

An early reading claimed the slice made the sim *6 % faster*, which is not a
thing a pointer indirection does. That number came from comparing a baseline
measured immediately after a heavy compile against a later idle machine. Both
sides had to be re-measured in the same session, alternating, before the delta
meant anything.

## What the shape taught me

**The value-level / pass-level split held up, and it is the important part.**
Not because two granularities are needed, but because writing `parent_p` forced
the rule to be stated: *a provider must never be called inside a hot loop to
answer a question that does not change inside that loop.* Left implicit, the next
ten conversions would each be a per-cell `fn` call by default, and the tenth one
would be the one in the innermost loop. The rule is now in the module docs with a
worked example beside it.

Three things I would change before converting the next ten:

1. **Payload structs will not survive their heirs.** `WaveCell` carries
   `{ index, gx, gy, base_rate }` because that is all the call site honestly has.
   But the named heir needs *fetch* — a quantity computed from the body graph,
   which is not available at a per-cell call site at all. So the heir will not
   plug into this signature; it will arrive as a **materialized plane**, i.e. it
   will want to be pass-level, and `wave_energy` will convert from value-level to
   pass-level when its heir lands. The lesson is not "the payload was wrong", it
   is that **granularity is a property of the heir, not of the call site**, and
   the call site is what I designed against. Next time: ask what the heir needs
   to read *before* choosing the granularity.
2. **`is_identity()` is the wrong shape for ten slots.** It works for three named
   fields. At ten it wants to be "which slots are non-identity", because that is
   the sentence a world's save file needs to record under the frozen-content-set
   rule. I did not build that; it should be designed when the save-side
   validation is, not before.
3. **No override channel yet, on purpose.** `DeepOverrides` was not extended.
   Nothing can currently *select* a provider, and a selection channel with no
   selectors is the same stand-in-becomes-definition shape in miniature. The
   resolution point is documented (`production_config_with`) so the socket for
   the socket is named without being built.

## Found and deliberately not fixed

- **`coal_rank` promotes on thickness where its own docstring says depth.**
  `biotic.rs::finalize` calls `promote_coal(COAL_MIN_M)`, i.e. buried peat
  becomes coal when the *seam* is thick enough, not when it is *buried* deep
  enough. Burial diagenesis is a function of overburden. Flagged in the seam
  inventory, confirmed still true, untouched — it is a behaviour change, and this
  slice changes no behaviour.
- **`P_FRESH` rejuvenation has no per-cell counterpart.** The *rate* fresh
  phosphorus returns at is still a global constant, even now that the *pool* it
  returns toward is a plane. That is a fourth seam, not a defect in this one.

---

*Gates green. Byte-identity proven against `2434f37` by two independent routes
(pre-capture, and a checked-out pre-slice worktree). `stubs.md` amended: entry 8
gains a slot, entry 13 is new, the layer-cake sibling gap gains a socket.*
