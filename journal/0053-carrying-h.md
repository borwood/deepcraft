# 0053 — Carrying `H`: the record *is* the regolith column

> blogworthy: the slice set out to stop a heuristic from inventing soil, and
> discovered instead that the quantity it was going to carry was already in the
> world, decomposed — and that three-quarters of it was being silently deleted
> by a rounding rule. The fix isn't "use the real number", it's "stop throwing
> away the beds too thin to see, and let the surface amalgamate them" — which is
> also, exactly, what soil is.

## The defect as filed

`docs/design/stubs.md` § 3. The deep-time erosion sim keeps a per-cell regolith
thickness `h` alongside bedrock `r`, and spends its entire 200-iteration run
weathering bedrock into `h`, moving `h` downslope and downwind, and dropping it
again. `deeptime/field.rs` then distilled the run to `surf = r + h` and threw
`h` away. So the collapse tier had no idea how much loose material was under
your feet and re-invented it from *present-day rainfall*: 3 / 2 / 1 voxels of
soil by precipitation band, and a clastic veneer budget of `1.0 + precip·2.5`.

The gameplay trace was already written for us. journal/0049, station 1: the user
standing in a deflation basin, unable to judge the wind erosion because the
world painted grass and topsoil over it anyway — *"always going to have
topsoil."* Under a rule whose floor was one voxel, that was structurally true:
**no column in the world could ever be bare.**

## Wrong turn 1 — the premise was wrong

The brief (and journal/0049, and me) said: the sim holds `H ≈ 0` at station 1,
so carrying `H` bares it out. It does not. Station 1's deep cell holds **10.66 m
of `H`**. The 13.06 m in the station's name is `ΔH` — what the wind *took* —
measured by isolating the wind agent against an agents-off run. `tour_map` picks
the deflation station by maximum `ΔH`, and the cell that loses the most is by
construction the cell that had the most to lose: a thick, loose, arid basin
fill. It is a *deflating* basin, not a *deflated* one.

So carrying `H` makes station 1 **deeper**, not barer. That is the ledger's
answer and we take it. Filed as corrections #26, whose lesson is the sibling of
#25: an extremum of a *rate* is not an extremum of a *state*.

Bareness is still real and still caused by erosion history — it is just rare.
91 of 44 265 subaerial deep cells (0.2 %) round to zero loose cover. The barest,
world (101 663, 5 073) with `H = 0.168 m`, now generates **basalt at the surface
with no soil at all**. Before this slice it wore 2 voxels of dirt because it
happens to get moderate rainfall.

## Wrong turn 2 — `H` was already in the world

Carry the plane, read soil depth from it, done. Except the first probe run
produced 8-voxel soil almost everywhere and a cap binding on 21 % of land, which
did not smell like a fix. So I measured `H` against the strata record the world
*already* keeps, expecting the record to be some fraction of the loose column.

```
recorded deposition Σ: mean 4.75 m;  residue (H − Σrecord): mean 0.00 m
residue quantized    : [44265, 0, 0, 0, 0, 0, 0, 0, 0]
```

**`Σ(recorded unit thicknesses) ≡ H`, cell for cell, over the whole world.** The
recorder logs every metre of loose cover the sim lays down, so the strata record
is not a *sample* of the regolith column — it *is* the regolith column,
decomposed into beds. Carrying `H` adds no new thickness information whatsoever.

Which means the old clastic veneer was never "the soil on top of the record". It
was a second, wholly imaginary loose column stacked on the real one.

## The mechanism that mattered: the sieve

If the record already is the column, why does the ground not already show it?
Because `deposit_deep_history` rounds each unit to whole voxels and drops
anything under half a voxel. Measure what survives:

- mean `H` per subaerial cell: **4.75 m** (5.3 voxels)
- mean thickness the record *expresses* as whole voxels: **~1.15 m**

**Three-quarters of the world's loose cover was being deleted by a rounding
rule.** Station 1 records 106 units summing to 10.66 m and expresses 5 voxels of
them. The dune field records 379 units summing to 7.99 m and expresses **zero** —
every single bed there is thinner than 45 cm.

That reframes the veneer's job completely. It is not "recent alluvium under the
year-zero climate". It is **the part of the loose column whole voxels cannot
resolve**, and the honest budget is

```
veneer_vox = round(H / 0.9) − (voxels deposit_deep_history just expressed)
```

clamped at 0 below and 8 above, with the fluvial fan term still added on top.

Amalgamating the unresolvable beds into one surficial body is not a fudge — it
is the physical process. Bioturbation, creep and soil mixing homogenize thin
beds into a surficial mantle; that is *why real soil is not laminated*. And it
makes the fill **mass-conserving against the ledger**: expressed loose voxels =
`round(H / 0.9)` exactly, wherever the cap does not bite. It is also the
geology.md § form-follows-provenance doctrine applied literally — sub-voxel
reality stays expressed by *changing form*, not by being deleted.

## The quantization rule, stated

`H` is metres; voxels are 0.9 m. **Round to nearest whole voxel**, clamp
`0..=8`. Bands: `H < 0.45 m → 0`, `0.45–1.35 → 1`, `1.35–2.25 → 2`, …

- **Round, not floor**, because floor shaves up to a whole voxel off every
  column in the world and would make the map systematically barer than the
  ledger says. Round is the minimum-error whole-voxel quantizer.
- **Zero is reachable, and that is the point.** The old rule's floor of 1 is the
  thing station 1 was complaining about.
- **Whole voxels only.** A 0.3 m grit mantle is not faked as a thin block here.
  Partial-voxel tops are the forms/partials slice and it is blocked on the
  user-owned surface-veneer retirement.

## The wilds

Beyond the pregen grid there is no deep-time run, so `regolith_at_voxel` returns
`None` and both consumers keep the year-zero precipitation rule they always had —
unchanged, and now explicitly *scoped* to the one place stubs.md § Genesis says
inventing a number is legitimate, because there is no recorded cause to consult.
No panic, no garbage read, no silent zero.

## A registration note that cost nothing because it was written down

`regolith_at_voxel` samples **nearest cell**, not bilinear — deliberately,
because the veneer is a *difference* against `record_at_voxel`, which is
necessarily nearest (a variable-length unit sequence cannot be interpolated).
Interpolating one term and not the other would leak or invent loose material at
every deep-cell boundary. Both step at ~460 m together, so no new seam appears.
Registration itself is the shared `deep_coords` convention — integer `wp/2`
centring, journal/0043's half-cell fix — which I did not have to rediscover
because it is documented at the one place that owns it.

## The goldens moved, and they had to

`tests/contents_contract.rs` says: *if a golden ever moves, the world moved,
and that is a bug until a journal entry says otherwise.* This entry is that
authorization.

| seed / extent | blocks | materials | table |
|---|---|---|---|
| 0x0D5EED572026 medium | `2028…3142` → `5863…4D62` | `9EF6…70C2` → `9B67…BDBC` | unchanged |
| 0x539 medium | `F584…B00A` → `54CD…852A` | `51DC…8EC9` → `89F9…B38C` | unchanged |
| 0xC11A7E2026 small | `641A…0F8C` → `024F…39CC` | unchanged | unchanged |

The **mixture-table** hashes did not move at all, and the small world's material
hash did not either: the change is one of thickness and extent, not of which
materials exist or how they intern. The `block == classify(contents)` invariant
test passed unchanged throughout — no second opinion about a voxel was
introduced.

One existing test had to change its claim. `tests/geology.rs` asserted *every*
land column deposits a clastic stratum, which was unfalsifiable while the veneer
had a floor of one voxel. It now asserts predominance and prints the bare count,
because "a land column can be bare" is exactly the fact this slice buys.

## What a player will see

Numbers at named ground, `seed 1337 / Medium` — the world every walk so far has
stood in. `rec` is the voxels the strata record expresses on its own; the veneer
is what the surficial pass adds; the loose column is their sum, i.e. the whole
diggable pile above bedrock. River-free columns, so the fluvial term is 0 in
both rules.

| site | `H` | rec | veneer old → new | loose column old → new |
|---|---|---|---|---|
| 1 deflation basin (5993, 14732) | 10.66 m | 5 | 1 → **7** | 6 vox / 5.4 m → **12 vox / 10.8 m** |
| 2 dune field (107183, 9672) | 7.99 m | 0 | 2 → **8** | 2 vox / 1.8 m → **8 vox / 7.2 m** |
| 3 loess margin (82346, 24391) | 80.49 m | 29 | 2 → **8** | 31 vox / 27.9 m → **37 vox / 33.3 m** |
| 4 periglacial summit (−4586, −3206) | 15.36 m | 5 | 2 → **8** | 7 vox / 6.3 m → **13 vox / 11.7 m** |
| 5 wave coast (95224, 22091) | 8.76 m | 4 | 2 → **6** | 6 vox / 5.4 m → **10 vox / 9.0 m** |
| 7 barest land (101663, 5073) | 0.17 m | 0 | 2 → **0** | 2 vox / 1.8 m → **0 — basalt at grade** |

Mass conservation reads off sites 1 (`round(H/0.9)` = 12, expressed 12), 5
(10, 10) and 7 (0, 0). Sites 2, 3 and 4 are the 8-voxel cap biting: the ledger
says 9, 89 and 17 voxels and the world expresses 8, 37 and 13. See § the cap.

Whole-world: soil depth stops correlating with rainfall and starts correlating
with erosion history. 8 % of land now carries no amalgamated veneer at all (only
its recorded strata), 0.2 % is bare rock at the surface, and the median column
is deeper than it was — because the sim genuinely deposited more loose material
than a rainfall lookup was ever going to guess.

Concretely, walking: a dune field and an arid basin are no longer one voxel of
dirt over stone; you dig 8–12 voxels of loose fill, which is what a basin fill
*is*. Two arid columns with identical rainfall can now differ by 8 voxels of
diggable depth because one sits in a sink and the other on a scoured shoulder.
Cut faces near the loess margin show 37 voxels of section before basement. And
for the first time there are places where the shovel hits rock immediately.

## The cap, and the defect this uncovered

The veneer is still clamped at 8 voxels, which truncates 14.3 % of land — those
columns lose ledger mass. Uncapping is worse: the loess margin would grow a
54 m single-member homogenized band. The real fix is neither: it is
**amalgamating adjacent sub-voxel units inside the record into composite units**,
which preserves stratigraphy instead of flattening it, and it belongs to the
record, not the veneer. Filed as stubs.md § 12 with that heir named. The number
to remember when it is picked up: **the 0.9 m sieve is currently eating ~75 % of
the recorded sediment pile.**

## Cost

The `H` plane is one `f64` per deep cell: 297 025 cells = **2 376 200 B
(2.27 MB)** at Medium, against a 147.80 MB `DeepField` — **+1.5 %**. Ritual wall
time is unchanged; the plane already existed in the run, it was being dropped at
the door. Given ROADMAP Observed's scrutiny of the +92.76 MB eolian record entry,
worth saying plainly: this is not that.

Probe: `cargo run --release -p dc-worldgen --example soil_depth_probe`.
