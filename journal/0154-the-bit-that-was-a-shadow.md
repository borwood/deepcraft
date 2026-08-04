# The bit that was a shadow

*2026-08-04 · the geo session · the deposition-clock slice (O-2b), from question to
merged in one day*

> blogworthy: a user asked one architecture question — "they were laid down on the
> same clock. did we throw the time away?" — and the answer dismantled a design pick,
> found eight bits storing a coarsened shadow of the thing we wanted, and landed a
> zero-byte fix the same day. Lenses: AI-native development (the design pass that
> falsified its own dispatcher's framing; the build agent that wedged twice on a
> mutex and had to be taken over), deepsim architecture (the merge-key mask as a
> structural free lunch).

## The question under P-3

The stratigraphic-correlation design offered R-C vs R-C′ — two flavors of the field
geologist's proportional correlation for undated sections. The user stopped the pick
cold: *"i don't quite understand… they were laid down on the same clock. did we throw
the time away?"*

Yes — but the design pass dispatched to price the fix came back with two findings
sharper than the framing it was handed:

1. **The epoch was never dropped at packing. It was never handed to the recorder.**
   `deposit_moved` took `(tag, d, chapter, species, mover)` — no epoch argument. The
   sim held the clock at every deposit; the recorder never asked. A signature-level
   loss, one level above the bitfield, and cheaper to fix than anyone assumed.
2. **Chapter is a pure function of epoch** — `floor(epoch / chapter_length)`, one
   writer. The record was spending 8 bits on a 3-bit quantity that is itself a lossy
   projection of the clock the correlation design was trying to reconstruct by
   proportional inference. We were simulating ignorance we didn't have.

## The free lunch

The user predicted *"some cleverness we aren't considering yet for storage"*, and the
pass found it: `key_bits()` masks the packed unit's **first word only**. The second
word — thickness — can donate space with the merge semantics untouched **by
structure, not convention**. u32 thickness became u24 + u8 raw epoch: the u24 caps a
single unit at 16,384 m against a measured production maximum of 65.38 m (250×
headroom; a u16 split would overflow today, so 8 bits is exactly what the word can
give). Zero bytes of widening. Unit count bit-identical. Merge keeps the bottom
epoch, and since epochs are monotone up-stack, one epoch per unit yields the full
age-interval structure from stack order alone.

**P-3 resolved by reshaping rather than by answer**: correlation now matches true
epoch intervals; the unconformity flag returns to gap semantics with the gap
measurable in epochs (a unit at epoch 41 under a flagged contact, the next at 88 —
47 epochs stripped); R-C is demoted to within-interval interpolation, the one place
it was always defensible.

## The slice

Recorder signatures thread the epoch; `set_epoch` stamps unconditionally from the
epoch loop (the tectonics pass is absent on the legacy path, so it could not ride
`set_tectonic`); monotonicity debug asserts land at both push sites — epoch AND
chapter, discharging the correlation design's own owed assert. The expression funnel
widens: `StrataEvent` carries `epoch_bottom`/`epoch_top` and the coalescer widens the
span on merge — expression carries, never reads, with a test pinning that. One
deliberate seam ships marked: **epoch is immutable under pedogenic overprint** while
`set_tag_and_chapter` still rewrites chapter in place, so the two can disagree after
an overprint — alteration is not deposition, and the alteration-time axis is the
named heir (stubs #52).

The identity proof ran before any constant moved: the new world hashed through the
**old** fingerprint shape reproduced the prior `GOLDEN_RECORD` bit for bit — 47,399
units on the golden fixture, epochs 0–199 all present — so the record is
byte-identical apart from the new axis. Then the fingerprint gained the epoch byte
and exactly the six `GOLDEN_RECORD*` families re-derived once; every surface arm
held still in the same runs. Full trio on the merged tree: **fmt 0 · clippy 0 ·
1006 passed / 0 failed / 94 suites** (22 tests up on the prior main gate — the new
asserts and pins).

## The lifecycle footnote

The build agent wedged twice waiting for the build slot — first yielding its turn to
a "background watcher" (the exact no-yield-on-watcher failure folded into the
fingerprints the day before, in a new costume), then, resumed with orders to hold its
own place in the retry loop, wedging again inside a monitor that would never trip.
The user diagnosed it from outside: *"it's stuck on nothing"* — and the slot was
indeed free, the lock stamp our own. The integrator stopped the agent, verified its
two commits (complete, with the identity proof already run), and ran the gate from
the worktree directly. The work was good; the waiting was the defect.
