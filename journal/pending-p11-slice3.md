# The record stopped being a chunk's opinion, and then it packed its bags

*P11 slice 3 — the near-path restructure (ruling 5) + the packed `DepUnit`
co-rider (U5, P-2 = L-8). Design pass:
`docs/audits/2026-08-02-p11-slice3-design.md`; built 2026-08-02/03 in the
worktree the main session harvests.*

**The M0 numbers, measured before any layout constant was chosen (audit § 7):**
mover split factor **1.0308×** (226,534 would-be splits over 7,363,947 units,
seed 1337 Medium) — under the ≲1.1× bar, so **P-4 resolved M-1: the mover is in
the merge key**; max unit thickness **65.38 m** — the 2⁻¹⁰ m u32 quantum stands
with five orders of headroom, and the audit's refusal to consider u16 unmeasured
was vindicated the cheap way (u16 @ 2⁻¹⁰ caps at 64 m — it would have
overflowed on the shipped world); `run_strata` is **0.2 % of `column()`**
(~7 µs per call over a 200-chunk timing), so the restructure's ~4× multiplier
on it is noise and the audit's mitigation (ii) — accept — is taken with a
number rather than a shrug.*

---

## Two changes that had to be one merge, and why that is a deviation worth naming

The plan was three merge-units: M0 (measurements), 3a (the restructure), 3b
(the pack), with 3a's goldens captured alone so a deep-time fingerprint moving
under a collapse-tier-only change would scream. That plan died of a fact about
the diff, not of a choice: the pack's accessor sweep (`.thickness_m` →
`.thickness_m()`, ~141 sites) and the restructure's consumer migration
(`col.strata` → `col.record_for(x, z)`, ~71 sites) run through the **same
fourteen files**, and the recorder that carries M0's mover instrumentation is
the same file the pack rewrites. A compiling intermediate state would have
meant hand-reconstructing a 16-byte recorder against already-converted
consumers. The design audit § 7 priced this exact fork and called the combined
merge *"legitimate under 'one golden move', not recommended"* — we took the
legitimate branch and this paragraph is the loud version of that call. What was
lost is the isolated 3a stillness tripwire; what was kept is the invariant it
guarded, asserted structurally instead (the restructure touches no deep-sim
state, and the pack's golden move is ratified quantization semantics).

## The restructure: run the expensive thing where its answer changes

Since 3e-1 a chunk's whole strata composition was **one point sample** — the
deep record of the cell nearest the chunk centre, NEAREST at the ~460 m grid.
The 2026-08-02 field report saw the consequence from ground level: member
presence borders that are straight, axis-aligned, and quantized to the 28.8 m
chunk pitch, because a border could only fall where the chunk-centre's nearest
cell flips (F1 verified the mechanism in code).

Now `column()`:

1. draws, per voxel column, **which deep cell's record skins it** — MM-1's
   `CoarseField::sample_source_cell` over a `CoarseField<u32>` of the deep
   grid's own cell indices, through the newly registered
   `NearRecordMembership` domain on the `Octaves` source (`Coherent` would
   amplify the home cell — corrections #39 — and the amplified party would be
   exactly the straight border this dither exists to kill);
2. runs `run_strata` **once per touched cell** — always 4 in a cell interior,
   up to 9 straddling edges, the arithmetic journal/0129 corrected — and wraps
   each product in the new `SubCell` (strata + its own `ColumnFill` + the same
   cell's soil), MM-3's type shipping *with* its consumer exactly as
   journal/0129's withdrawal condition demanded;
3. skins each column's surface voxel from **its own** SubCell's top span.

**F2 is the load-bearing constraint and it became a type.** The three NEAREST
reads — record, regolith `H`, ledger bedrock slot — are mass-coupled: `H` is
exactly the record's thickness sum, the veneer is their difference, and the
ledger's bedrock slot index is `record.units.len()`. Dither one without the
others and Law 3 leaks at every frontier column on the map, silently.
`DeepField::cell_bundle` answers all three for ONE cell in one call, the
bundle's ledger is private so its slot can only be indexed by its own record's
length, and the collapse tier reads nothing else.

The acceptance instrument (audit § 4.2) rides in `appearance_tour_p11`'s gate
— **with one correction to the audit's draft, made loudly.** The draft asked
for per-chunk 4σ *binomial* floors derived from the stencil weights; a
binomial bound assumes per-column independent draws, and the audit's own
source pick (I-2: `Octaves`, coherent on purpose — a rock body is not
per-voxel speckle) makes nearby columns share low-frequency draw values.
journal/0129 taught exactly this at the MM-1 law test: *"the source has to be
white for that test to mean anything."* So the shipped instrument asserts
**structural impossibilities of the retired read** instead: a realized-parent
transition INSIDE a chunk, off the 28.8 m lattice (the old read's parent was
constant per chunk); and realization of the far parent MORE than 16 voxels
past the geometric edge (the old read's maximum reach, half a straddling
chunk). The frequency-vs-weight comparison is printed as a report, never
asserted — a fitted band would be the tolerance anti-pattern. The F2 bundle
equality per sampled column stays an assert, with a derived bound
(f64 residual + half a thickness quantum).

What deliberately did NOT come along: the veneer's per-column formation
context (D-6/P-5 — stub #31's heir, re-pointed honestly in its banner rather
than left stale a second time), the far path, and anything grain-modelling.

## The pack: 16 bytes of unit becomes 8, while gaining two axes

`DepUnit` is now a u32 bitfield (env 1 · aridity 1 · energy 2 · biota 3 ·
eolian 2 · unconformity 1 · species 6 · **mover 3** · **grain 3** · chapter 8
= 30 bits) plus a u32 **fixed-point thickness** at 2⁻¹⁰ m. Every read site is
an accessor call now — the P-2 ruling's premise, and the reason a future
widening (the ruling-3 hint byte, eco/civ-era axes) is an accessor plus one
golden re-capture instead of another 141-site sweep.

The thickness quantum drags quantization into the sim loop (the outcrop window
reads thickness back every epoch), so every deposit and strip goes through a
per-cell f64 **carry**: `|carry| ≤ q/2` after every op, and record + carry
closes against the delivered budget to a bound that is *derivable* — quantum ×
op count — instead of float-accumulation-shaped. Fixed-point sums are exact
and commutative, so corrections #89's hazard class (terrain moved by summation
order) retires for the record's own sums. The finalize invariant
`Σ units == H` became `Σ units + carry == H`, and the five tests that pin it
now say so.

**The mover axis is real on day one** (stub #25 discharged): the transport
recorder writes the dominant arriving mover (fluvial vs hillslope gravity,
derived from the same argmax that names the rock), wind writes `Eolian`, waves
write `Marine`, in-place events write `MOVER_NONE`. Colluvium and alluvium
stopped being separable only by signature — they are separate units now,
because the measured 1.0308× split said the key could afford them. The M-2
machinery (a thickness-weighted Boyer–Moore majority combine) was written,
measured against, and deleted in the same session: the instrument existed to
pick a branch, it picked M-1, and keeping the loser would have been a second
mechanism beside the one that won.

**The grain axis is named and UNSET** (P-3 = O-2): three bits reserved for
U4's five φ classes, `GRAIN_UNSET` loud in every unit, `grain()` /
`set_grain()` waiting for FS-A's release-spectrum writers. The bits sit in the
merge key on purpose — inert while uniform, which is precisely the #88
tripwire: a gate test asserts the unit count is identical to a grain-blind
count model, and the day a writer splits the record, that split arrives
measured or not at all.

## Goldens

All CONTENTS/SURFACE movement in 3a's half is ratified per-column-record
semantics (ruling 5); the pack's movement is ratified quantization semantics
(U5) that feeds back through the outcrop window into rates, so **every family
moves once** — re-captured with the why per family in the test comments,
scratch-world doctrine, no byte-ratification loop. The Small contents row's
long-standing structural stillness (no strata record in its sampled chunks)
finally ends where the pack's quantization reaches the terrain everywhere.

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — a mass-coupling
> constraint (F2) turned into a type so it cannot be remembered wrong; and a
> record that got *cheaper* by getting *more honest* (8 B/unit while adding the
> mover and grain axes). Also **lens 1** — a three-commit plan collapsing into
> one merge because two surgeries shared fourteen files, and the discipline of
> saying so loudly instead of pretending the intermediate state existed.
