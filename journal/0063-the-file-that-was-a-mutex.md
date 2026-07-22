# 0063 — the file that was a mutex

*2026-07-22 — splitting `providers.rs` into a module directory, and the
identity check that had been quietly lying about itself.*

## The problem was not the file, it was the concurrency

`providers.rs` was 440 lines and perfectly readable. Nothing about it was
badly written. What was wrong with it only became visible from outside the
file: journal/0060 converted three seams, journal/0061 converted one more, and
**both slices had to edit the same four regions of the same file** — the
payload structs, the identity functions, the `Providers` fields, the `Default`
impl. Run those two slices as concurrent agents instead of sequentially, and
they conflict on every one.

That matters more than it sounds, because there are **34 seams in the
inventory** and four are converted. The remaining thirty are almost entirely
independent of one another: `depth_to_water` and `parent_p` share no code, no
consumer and no heir. There is no reason two people cannot convert two seams at
the same time — except that the file said no.

So this slice changes no behaviour at all. It exists to make a file stop being
a mutex.

## The layout, and the grouping that is not decoration

```
deeptime/providers/
    mod.rs              the Providers struct, Default, Slot
    outcrop_at.rs       identity + tests
    wave_energy.rs      payload + identity + tests
    parent_p.rs         payload + identity + tests
    depth_to_water.rs   payload + identity + accessor + tests
```

A new slot adds a **new file**, and a new file cannot conflict with anything.
That is most of the win, for free.

The rest of the win is in `mod.rs`, which a new slot still has to touch — the
field, the `Default` line, the `Slot` variant. Those are grouped by **owing
system** with comment banners: hydrology, ecology, materials, structural — the
same four buckets the 34-seam inventory counts in (ecology 9, hydrology 6,
materials 6, social 5). Two agents converting an ecology seam and a materials
seam now insert at two different points in every one of those lists, and git
merges them without a human.

The grouping key is deliberately **who will answer, not who asks**. `parent_p`
is read by the biotic layer and filed under *materials*, because
parent-material petrology is the heir. That is not a filing quibble: a seam's
whole content is an obligation on an unbuilt system, so grouping by consumer
would sort the sockets by the least informative half of the relationship. Read
down the file and you get a **map of who owes what**.

`ecology` currently has a banner and no slots under it. Leaving an empty group
in place looks like clutter and is the opposite: it is the insertion point, and
it is worth more than the four lines it costs, because the next ecology
conversion does not have to decide where to put anything.

The tests split the same way — `providers_<slot>.rs` per slot, with the
fingerprint helpers in `tests/providers_common/mod.rs` and, alone in its own
file, `providers_golden.rs`. **No seam conversion has any reason to open the
file holding the goldens.** A constant that nobody is ever editing near is a
constant that cannot get "updated to match".

## The bug the flat file was hiding

The refactor was supposed to be inert. It broke two tests immediately:

```
the default set reported non-identity slots: [OutcropAt]
```

`Providers::default().is_identity()` — false. On a default set. The one thing
that function exists to say.

`is_identity` compares `fn` pointers by address, because there is no other way
to compare function pointers. The old docstring worried about exactly one
failure mode: identical-code-folding could give two *different* functions the
*same* address, reporting a custom provider as the identity — "the harmless
direction". The dangerous direction was assumed impossible.

It is not impossible. `exposed_litho` is `#[inline]`, and an `#[inline]`
function may be instantiated in **several codegen units**, each instance with
its own address. `identity_outcrop_at` was a `pub use` of it. So two
`Providers::default()` values built in two different codegen units could hold
two different addresses **for the same function**, and compare unequal. One
function, two addresses, and an identity check that says a pristine world is
not pristine.

The other three identities never had the problem because they are plain,
non-`#[inline]`, crate-local functions — codegened once, one address each. The
fix is to make `outcrop_at`'s identity the same shape: a two-line wrapper in
`providers/outcrop_at.rs` that calls `exposed_litho`. It costs nothing, because
**a provider is always called through a pointer and is therefore never inlined
at the call site anyway**; the wrapper body inlines `exposed_litho` and the
generated code is identical.

The rule this earns, now written on the wrapper: **a slot's identity must be a
plain, non-inline function defined in the slot's own module.** Never a `pub use`
of somebody else's function, whose inlining attributes are not ours to control
and can change under us without a diff in this file.

The part worth sitting with is *when* this surfaced. The bug was latent on
`main`: same comparison, same `#[inline]`, same `pub use`. It passed because,
in the flatter code, the optimizer happened to see both sides of the comparison
in one view and fold them to `true`. **The test was passing for a reason that
had nothing to do with the property it was asserting.** Splitting the file
changed the inlining, the coincidence stopped holding, and a genuine defect
fell out of a pure code-motion refactor. Byte-identity is what proves the
refactor changed no *values*; it cannot prove the refactor changed no
*coincidences*, and this project's whole habit of running the gate on the code
it actually built is why that showed up as a red test and not as a save file
that mis-records its own provenance in six months.

> blogworthy: a refactor with zero behaviour change that broke two tests — the
> latent bug that was being masked by an optimization coincidence, and why
> `fn`-pointer identity is a worse foundation than it looks.

## `is_identity` grew a vocabulary

journal/0060's second parting note said `is_identity()` is the wrong shape at
ten slots: the useful sentence is *which* slots are non-identity, because that
is what a world's manifest records under the frozen-content-set rule
(ARCHITECTURE.md, DECIDED 2026-07-22). A world that loads with a generational
provider missing must **hard refuse**, and it cannot refuse against a bool.

```rust
pub enum Slot { WaveEnergy, DepthToWater, ParentP, OutcropAt }
impl Slot {
    pub const ALL: &'static [Slot];
    pub fn name(self) -> &'static str;   // the field name, verbatim
}
impl Providers {
    pub fn non_identity_slots(&self) -> Vec<Slot>;
    pub fn is_identity(&self) -> bool { self.non_identity_slots().is_empty() }
}
```

`name()` returns the Rust identifier — `"wave_energy"`, not "wave energy" —
so a token recorded in a manifest and a `grep` of the source agree. `is_identity`
survives as a one-line convenience over the report, because most callers are
assertions that only want the yes/no, and deleting it would have churned call
sites for nothing.

This is still not the registry. There is no loader, no declaration pass, no
selection channel; `Slot::ALL` is a hand-written list and adding a slot is a
compile error at four places in `mod.rs`, which is the point. The enum is the
**alphabet** the save side will eventually need, built now because the shape of
the answer was already known and building it later would have meant touching
every conversion again.

## What a conversion has to touch now

The number this slice exists to reduce, before and after:

| | before | after |
|---|---|---|
| source files edited | 1 (`providers.rs`, in 4 places) | 2 — a **new** `providers/<slot>.rs`, plus `mod.rs` in 4 grouped places |
| test files edited | 1 (`tests/providers.rs`) | 1 — a **new** `tests/providers_<slot>.rs` |
| files with a conflict surface | 2 | **1** (`providers/mod.rs`) |
| files holding the goldens that a conversion opens | 1 | **0** |

Two concurrent conversions now collide only inside `providers/mod.rs`, and only
if they belong to the same owing system. The 34-seam inventory says the four
groups are 9 / 6 / 6 / 5, so most pairs of concurrent conversions land in
different groups and merge clean.

## Could any of this have changed a value?

No, and the argument is stronger than the golden test on its own.

Every change is code motion: the four identity functions, the two payload
structs and the accessor moved between files with their bodies untouched — the
same expressions in the same order with the same `f64 → f32` cast points. The
one non-motion change is `identity_outcrop_at`, which went from a re-export of
`exposed_litho` to a wrapper that calls `exposed_litho` — the same function
computing the same value for every input, and `exposed_litho`'s body is not
touched. `is_identity`'s answer is not read by any generation code; it is
consulted only by tests and (eventually) the manifest, so even if the
address-comparison fix had changed its answer for some caller, no plane and no
recorded unit could move.

The goldens confirm it: `surface 0x7B8968FD90E04062` / `record
0xA53BD77F769D7FF4`, from pre-slice `main` (`2434f37`), pass unchanged — and so
does the agreement test that runs the same production world down the
materialized-plane path.

## Found and deliberately not fixed

- **`grid.rs`'s docstring still points at `tests/providers.rs`**, which no
  longer exists. Not in this slice's write-set (a sibling agent is working
  nearby); one line, `crates/dc-worldgen/src/deeptime/grid.rs:230`.
- **`Slot::ALL` is hand-maintained.** Adding a field to `Providers` without
  adding a variant compiles fine and the new slot is silently never reported.
  `providers_set.rs` pins the full list as a string so the omission fails a
  test rather than passing silently, which is the cheap half of the guarantee;
  the expensive half (a derive, or the registry) is still correctly deferred.

---

*Gates green on the code that was actually built (`cargo clean -p dc-worldgen
--release` before both the clippy and test gates). Journal number 0063 taken at
commit time; two sibling agents were writing entries in the same session.*
