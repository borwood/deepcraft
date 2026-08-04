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

## What was measured

<!-- FILL: numbers -->

## The acceptance criterion was byte-identity, not green tests

<!-- FILL: byte identity -->

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
