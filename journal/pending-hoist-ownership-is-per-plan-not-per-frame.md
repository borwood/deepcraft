# Ownership is a property of the plan, and we were asking it sixty times a second

*(pending — SLUG-ONLY, no ordinal: a parallel user session shares this checkout and
ordinal collisions have happened repeatedly. The integrator numbers it.)*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — the smallest possible
> example of a defect class this project keeps re-finding: work whose *granularity*
> is wrong. Not slow code, not a bad algorithm. Correct code, run at the wrong rate.
> Also **lens 1 (AI-native development)**: the slice that introduced it measured it,
> named its own heir in a doc comment, and left the number where the next session
> would trip over it. That is the whole mechanism by which this got fixed at all.

## The defect, in the shape it was left

`journal/0155` shipped per-cycle stop-motion quantization and, honestly, shipped a
**22 % regression** with it: 3312 ns per body per frame before, 4036 ns after. It
recorded both numbers, the two shapes it had already rejected on measurement, and —
in the test's own doc comment — the heir:

> **The named heir for the rest is hoisting the ownership tables into `character.rs`'s
> per-plan `BodyAssets`** — ownership is constant per (plan, clip set) and only
> `gait_step` moves with speed, so the whole pre-pass is per-plan work being redone
> per body per frame.

The user's reaction on reading it was sharper than the note: *"we're doing per-instance
redundant math instead of it being owned per-plan? we can't make that mistake."*

Concretely, `pose_for` opened every frame, for every body, by:

1. walking the gait's limbs to build a `bearing` name list and a `gait_names` list;
2. for every clip, walking **every keyframe and every rotation in it** to discover
   which segments that clip animates — into a fresh `Vec<&str>` per clip, deduped
   with linear `contains` scans.

None of that can change between frames. A clip's keyframes are fixed when it is
defined. Which limbs bear is fixed when the plan is baked. Four heap allocations and
a nested scan, per body, per frame, to recompute a constant.

## Why the fix is not "cache it" but "ask it at the right boundary"

The rule was already ratified — seam-first practice #4, *granularity follows the hot
loop*: **a provider must never be called inside a hot loop to answer a question that
does not change inside that loop.** The corollary the note attaches is the one that
predicts this exact failure: *left implicit, the tenth conversion lands in the
innermost loop.*

There is a version of this fix that is a bug: bolt a cache onto `pose_for`, key it on
something, invalidate it on something else. That is the shape the brief forbade by
name — *"not a new cache with unclear invalidation"* — and it is the shape that makes
a perf win into a correctness liability.

What we did instead has no invalidation rule at all, because it has no cache. The
answer is a **field of the struct that owns its inputs**. `character.rs`'s `BodyAssets`
already holds the plan, the idle clip, the baked gait and the leg rigs, all by value,
all built once per plan in one constructor. `PoseOwnership` is built in that same
constructor, from those same clones, three lines below them. It cannot describe a
different clip set than the one it lives beside, because there is no second copy of
the inputs to drift from — the copy *is* the input (S-3).

The one honest staleness statement: the table goes stale if a plan or a bound clip is
**redefined in the registry under the same name**. That is not new and not ours —
`BodyAssets` already cached the `BodyPlan` and the `AnimClip` as clones and is never
invalidated on redefinition, so a redefined clip changes nothing about an
already-rendering body today, table or no table. The table inherits exactly that
staleness and adds none. Whatever verb eventually invalidates a plan's assets rebuilds
this with them, because it is a *field of the thing being rebuilt*.

## The seam that keeps the two halves honest

`pose_for` now takes the table. That means a caller could hand it a table built from a
different clip set — a silent wrong-pose bug, the worst kind. Two things close it:

- `BodyAssets::layers()` is the **only** place that names the layer set. Both the table
  build and the per-frame `pose_for` call read the clip slice from there, so there is
  no way to build from one set and pose with another.
- `pose_for` opens with a `debug_assert` that the table's clip names match the slice it
  was handed. It costs nothing in release and it fails loudly in every test.

That is the reason `ClipOwnership` stores a name it otherwise has no use for.

## What was measured — and the measurement that had to be thrown away

The obvious experiment is: build the pre-hoist sources, build the post-hoist sources,
run the same bench in both. We did that. **It is not trustworthy, and the reason is
worth more than the number was.**

Interleaved, alternating order, 30 runs a side, the shipped path went from a minimum of
**4320 ns** to **3606 ns** (−16.5 %), median 5434 → 4798 (−11.7 %). But the bench also
prints a **control** — `sample_clip` on the parked walk fixture, a function neither slice
touched — and *the control moved too*: 1725 → 1500 ns minimum, −13 %. Identical source,
different number. Either the compiler laid it out differently in the two binaries, or it
inherits allocator state from the loop that runs before it (which is exactly what the
hoist changed). Both are plausible; neither is attributable. **A before/after across two
binaries measures the binaries, not the change.**

So the number that means something is measured **inside one binary**, in the same test:
the identical loop, run twice, the only difference being whether the ownership table is
rebuilt inside it. That is precisely the defect, with layout, allocator and CPU state
held constant.

| | ns per body per frame |
|---|---|
| hoisted (shipped) | **5013** |
| the same loop, table rebuilt per call | **6825** |
| **removed by the hoist** | **1812 (−26.6 %)** |
| the pre-pass on its own | 1069 |

*One honesty note on that table:* the rebuilt-per-call figure uses the **new** table type,
which owns its names (`Box<str>`, one allocation each) where the old inline code borrowed
them (`Vec<&str>`, four allocations total). So 1812 ns is an **upper bound** on what the
old pre-pass cost, not an exact restatement of it. The cross-binary minimum (−714 ns) is a
lower bound with a different flaw. The true figure is between them, and both say the same
thing about direction and order of magnitude.

**Against the 3312 ns pre-slice baseline the brief asked about: I cannot settle it, and I
will not pretend otherwise.** The machine tonight was running two other sessions' cargo
gates for the entire measurement window; the same bench spread 4320–6726 ns on the
*pre-hoist* binary depending on when it ran. 3606 and 3312 are 8 % apart, and the noise
floor is three times that. What can be said without hedging: **the hoist removes more than
the quantization slice added** (−26.6 % measured in-binary against a +22 % regression), and
**it removes it from the frame loop entirely rather than making it cheaper**.

## The acceptance criterion was byte-identity, not green tests

The tests were green before this slice and they would have been green after almost any
version of it, including a wrong one. A re-housing has exactly one acceptance criterion
worth the name: **the poses must be bit-identical**. So the evidence is a dump, not an
assertion — every composed pose's three Euler components as raw `f64::to_bits`, over a
sweep of three plans × four target rates × five speeds × four layer sets × both the
derived-gait body and the identity fallback, twenty-four frames each. Pre-hoist sources
checked out into the same worktree, same harness, same command; the two logs diffed.

**61 919 pose lines, identical SHA-256, zero differing lines.** Not "the tests pass" — the
same bits.

The harness is deliberately not left in the tree. A committed digest of today's poses is
the snapshot § Gates forbids by name: it would fail the day a colleague legitimately
improves the gait, which is worse than the defect it guards. What *is* left is the
property test — the gait owns every segment on every limb it derives, a clip owns
everything it names except a bearing chain, and the table is a pure function of its two
inputs. That stays true of a plan nobody has written yet.

## A false RED, and how it was caught: a test that does not exist

The workspace gate came back **1032 passed, 1 failed**, with the failure in
`dc-worldgen`'s `creep_operator_probe` — a crate this slice does not touch — plus
`-p dc-worldgen --doc` failing to compile on `unresolved import
dc_core::field::ExplicitPlan`. Neither is plausible from a `dc-client` re-housing, so
the first instinct is "flaky, ignore it." That instinct is what corrections #27 exists
to punish.

The check that settled it took one grep. **The failing test's name does not exist
anywhere in this checkout.** `creep_operator_probe.rs` here declares exactly two gate
tests; the binary the gate ran declared **five**, including
`the_implicit_scheme_never_flips_the_grid_scale_mode`. Likewise, the string
`dc_core::field::ExplicitPlan` appears in no file here — while `ExplicitPlan` itself is
a perfectly public `dc-core` type, so the import *should* resolve, and the doctest that
failed to write it is simply not one of ours. Re-running both targets alone recompiled
`dc-worldgen` **from this worktree** and returned `2 passed / 0 failed` and `0 doctests
/ ok`.

So: **another session's `cargo test --workspace --release` was live on the shared
`CARGO_TARGET_DIR` for the whole run, and its artifacts were served to my gate.** That
is corrections #27's mechanism running in the opposite direction — not the false GREEN
where your code never built, but the false RED where *someone else's* did. The build-slot
hook is supposed to make this impossible; I watched two cargo processes coexist for most
of the run, so it did not.

The generalisable bit is the discriminator, and it is better than "did it pass?": **count
the tests and read their names.** A suite that runs five tests where the source declares
two is not a failing suite, it is somebody else's suite. *"Did it run?" is a separate
question from "did it pass?" — and "whose code ran?" is a third one.*

## What is deliberately not here

- **The `body.rs` module extraction.** The file is over 3400 lines and the size hook
  fires on every edit. `journal/0155` already proposed the split; it is a separate
  slice with its own gate, and doing it inside a perf slice would make the
  byte-identity diff unreadable — which is the one piece of evidence this slice is
  actually made of.
- **The pack-declared per-body max target fps** (`stubs.md` #53). Still B3's.
- **Baking the client's `AnimRate` into the table.** Every rate-derived quantity in the
  loop is three floating-point operations. Folding a *setting* into a per-plan table
  would buy a few nanoseconds and cost a second invalidation rule — which is the trade
  this slice exists to refuse.
