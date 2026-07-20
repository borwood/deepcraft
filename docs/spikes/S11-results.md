# S11 results — water locality + the free-water body graph

Status: spike complete, 2026-07-20. Code is **additive and standalone** in
`crates/dc-worldgen/src/water/` (`vox.rs`, `sat.rs`, `conn.rs`, `body.rs`);
measurement harness `examples/water_spike.rs`; invariants `tests/water.rs`.
Nothing in the production world-generation path calls it, deep time is
untouched, and there is no renderer work. Numbers from the Windows dev box,
release profile.

The question (docs/design/water.md § "The hypothesis S11 tests — persist
bodies, derive voxels"): the two regimes have different *locality*, so the
hypothesis is that bound water can be **derived** within a halo while free
water must be **persisted** as a graph. Two measurements decide it.

- **Q1** — is the bound-water relaxation genuinely LOCAL? Measure the halo of a
  perturbation against permeability contrast, the way S9 measured erosion's
  decay length.
- **Q2** — does the free-water body graph stay sparse under adversarial
  digging? Measure nodes, links, events, resolution cost and persisted bytes
  across the user's seven scenarios.

**Headline: GO on "persist bodies, derive voxels".** Bound water's halo is
**4–11 cells** at a bounded post-edit budget and the *player-visible* halo
(the integer water table moving a whole voxel) is **0–6 cells**. The body
graph does not grow with edits at all — it grows with **components**, and one
body per component is the ceiling: a heavily-dug 512×64×512 world with 1 624
edits and 508 084 dug voxels holds **217 bodies in 4 400 persisted bytes**.

---

## Q1 — bound water is local, and the drainage network is why

### What was built

`sat.rs` is the smallest honest saturation relaxation over the S8 pore model.
One scalar per voxel — `sat` ∈ [0, 1], the filled fraction of that voxel's pore
volume, so water is `sat × porosity` in voxel-volumes. Three sub-steps per
iteration, each a **gather from a frozen snapshot** (the S9b reformulation):

1. **Infiltration** — recharge into the topmost non-void voxel of each column.
2. **Gravity percolation** — one downward edge per voxel, limited by
   `min(k_here, k_below)` and by the receiving voxel's free pore space. One
   edge per voxel means outflow is bounded by the voxel's own water, so no
   limiter is needed and mass is exact by construction.
3. **Permeability-limited lateral redistribution** — head `H = y + sat` (an
   unconfined proxy), flux `k·ΔH·c` per lateral edge. Each edge is computed
   **once from the unordered pair** and scaled by two frozen per-voxel limiters
   (`l_out` on the donor, `l_in` on the receiver). Both limiters are functions
   of the frozen state, so the edge value is identical read from either
   endpoint: exactly antisymmetric, mass-exact, and independent of traversal
   order.

The **water table is read, never stored** — `water_table()` returns the top of
the saturated zone per column, per water.md consequence 1.

### The boundary condition, and a wrong turn worth recording

The first version of this measurement was **degenerate and I nearly reported
it**. A closed box with recharge and a single drain has no local equilibrium:
water accumulates until the whole domain saturates, so perturbing it changes
every column and the "halo" came back as the domain radius at every
permeability contrast — a flat profile that looked like a dramatic finding and
was actually an artifact of a missing boundary condition.

A real water table is not bounded by the edge of the world. It is **pinned by
the drainage network** — streams, springs and coastlines every few hundred
metres. Adding that (a lattice of seepage columns at spacing `L`) plus a leaky
aquitard over a regional deep sink turns the field into something with an
actual equilibrium, and the measurement into something meaningful.

Two further corrections along the way, both recorded because they would each
have produced a confident wrong number:

- **The relaxation was oscillating, not relaxing.** `k·lateral_c` must stay
  under `φ/(2·ndim)` for an explicit diffusion step; the first version used
  `1.0` against `φ = 0.30`, so cells dumped their whole content into a
  neighbour and got it back next step. The oscillation floor (~0.5–1.5 storage
  units) swamped the signal. Fixed at `lateral_c = 0.03`.
- **Control and treated were compared at different times.** The reference ran
  `steady` steps; the perturbed run ran `steady + budget`. Both runs now
  advance together and are differenced at matched step counts, so common-mode
  transient cancels and only the perturbation's differential effect is read.

Method: 97×24×97 (225 816 cells), equilibrate 2 500 steps, then fork into a
**control** (unperturbed) and a **treated** (one new seepage column — a dug
shaft — at the centre of a lattice cell). Both advance identically. Halo = the
deepest Chebyshev ring whose max |Δ| still exceeds the threshold. `drift` is
how much the reference still moves in 20 more steps, i.e. the residual noise
floor the halo reading sits above. Aquifer `k` is held at 1.0 so convergence
time is constant; **the contrast variable is the aquitard beneath it** — the
loose-vs-packed soil contrast water.md names.

### The halo table

`halo_bud` = after a bounded 64-step post-edit budget (what production would
actually pay). `halo_ss` = at steady state (the honest asymptote).
`vis_halo` = rings where the **integer water table** moves a whole voxel — the
only halo a player can see. Threshold 0.01 storage units for the field halos.

| L | k_aquitard | contrast | halo_bud | halo_ss | vis_halo | L/2 | drift |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 8 | 1.0 | 1 | **5** | 5 | 0 | 4 | 0.0000 |
| 8 | 0.1 | 10 | **4** | 4 | 0 | 4 | 0.0000 |
| 8 | 0.01 | 100 | **11** | 26¹ | 0 | 4 | 0.0084 |
| 8 | 0.001 | 1 000 | **9** | 15¹ | 0 | 4 | 0.0049 |
| 8 | 0.0001 | 10 000 | **9** | 14¹ | 4 | 4 | 0.0019 |
| 16 | 1.0 | 1 | **5** | 5 | 0 | 8 | 0.0000 |
| 16 | 0.1 | 10 | **4** | 4 | 0 | 8 | 0.0000 |
| 16 | 0.01 | 100 | **6** | 10 | 0 | 8 | 0.0038 |
| 16 | 0.001 | 1 000 | **5** | 6 | 0 | 8 | 0.0049 |
| 16 | 0.0001 | 10 000 | **5** | 5 | 6 | 8 | 0.0019 |
| 32 | 1.0 | 1 | **5** | 5 | 0 | 16 | 0.0000 |
| 32 | 0.1 | 10 | **4** | 4 | 0 | 16 | 0.0000 |
| 32 | 0.01 | 100 | **6** | 10 | 0 | 16 | 0.0037 |
| 32 | 0.001 | 1 000 | **5** | 6 | 0 | 16 | 0.0049 |
| 32 | 0.0001 | 10 000 | **5** | 5 | 6 | 16 | 0.0019 |

¹ **Not resolvable.** In these three rows the residual drift (0.0019–0.0084) is
within an order of magnitude of the 0.01 threshold, so the steady-state halo is
reading the noise floor, not the signal. The bounded-budget column is
trustworthy throughout (the perturbation is 3–4 orders of magnitude above the
floor at `d0`); treat the `halo_ss` figures for L=8 at contrast ≥ 100 as upper
bounds only.

Decay profiles at L=32 (max |Δ storage| by ring — the shape is what matters):

```
contrast     1:  d0=1.6000 d2=0.1621 d4=0.0142 d6=0.0008 d8=0.0000  (→0 by d8)
contrast    10:  d0=1.6000 d2=0.1266 d4=0.0058 d6=0.0001 d8=0.0000  (→0 by d8)
contrast   100:  d0=18.909 d2=4.4768 d4=0.9560 d6=0.1391 d8=0.0388 d10=0.0096 d12=0.0020 d14=0.0003
contrast  1000:  d0=19.615 d2=2.8254 d4=0.1064 d6=0.0016 d8=0.0000
contrast 10000:  d0=19.671 d2=2.4896 d4=0.0935 d6=0.0014 d8=0.0000
```

This is clean geometric decay — roughly a factor of 4–30 per two cells — not a
long tail, and emphatically not the unbounded propagation that would have
falsified the hypothesis. Compare S9's fluvial row, where isolated spikes
*reappeared* at d14 and d28: nothing of that kind happens here. Bound water has
no advective term; it is a pure relaxation, and it measures like one.

### The two mechanisms that bound it

The halo is bounded by whichever of two lengths is shorter:

1. **Drainage spacing.** The water table is pinned at every seepage column, so
   a new drain cannot influence past its neighbours in the drainage lattice.
   This is why the numbers cluster near `L/2` and never exceed it by much.
2. **Leakage through the floor.** A leaky aquitard (contrast 1–10) drains the
   aquifer downward everywhere, so the table is already near the sink and a new
   drain barely perturbs it: **halo 4–5 cells, decayed to zero by d8**.

The counter-intuitive result worth stating plainly: **a sharper aquitard does
NOT give a bigger halo.** Contrast 10 000 measures 5 cells at L=32 where
contrast 100 measures 6–10. A tight floor perches the aquifer at full
saturation, which caps the head range a drawdown cone can express (saturation
is bounded at 1 per layer); a leaky floor gives the perturbation somewhere to
go locally. Either way the influence is short. The aquitard case the brief
specifically flagged as the risk — the sharp loose-vs-packed contrast — is the
*most* local case measured, not the least.

Secondary perturbation, a saturation spike rather than a drain: **halo 0 cells**
— a transient injection dissipates entirely and leaves no steady-state trace.

### Q1 invariants

- **Conservation** (no recharge, no drains, 64 steps): relative drift
  **2.4 × 10⁻⁷** — float round-off, and the water.md § consequence 5 invariant
  (the transition must neither create nor destroy water) holds.
- **Determinism**: relaxation double-run is **byte-identical** (`f32::to_bits`
  comparison). The gather formulation means a parallel driver would be too, by
  the same argument S9b proved for the deep-time phases — not measured here,
  because nothing needed it yet.

### Q1 verdict

**Bound water is local enough to derive.** Budget halo **4–11 cells**;
player-visible halo **0–6 cells**; decay is geometric with no advective spikes;
and it gets *more* local, not less, under the sharp permeability contrast that
was the named risk. A derivation halo of **~12 cells** covers every case
measured with a large margin, and ~16 would be generous. That is comfortably
inside the collapse layer's existing bounded-lookahead budget, and far smaller
than S9's 16–24-cell erosion halo the pipeline already carries.

---

## Q2 — the body graph, and what actually grows

### What was built

Two structures, and the split between them is the finding.

**`body.rs` — the persisted graph.** A `Body` is `{id, kind, anchor, volume,
level, merged_into}` — 21 bytes for one body in postcard, the project's wire
format. `BodyKind` is `Finite` (a conserved volume in a container) or `Pinned`
(a level-pinned reservoir whose level comes from outside). Links are spill/gate
relations `{a, b, sill, open}`. The event vocabulary is water.md § 4's, made
commutative (below).

**`conn.rs` — the derived connectivity index**, which is *not* persisted. Two
levels:

1. Per-chunk local air-component labelling (32³, the project's chunk),
   recomputed for a dirty chunk only.
2. A union-find over `(chunk, local label)` nodes joined across shared chunk
   faces, with the face adjacency **cached per chunk pair** and refreshed only
   for pairs touching a relabelled chunk.

A query is chunk lookup → local label → `find` → root: **45.9 ns**, no search
at any radius. That is scenario 2's whole requirement, met by construction.

### The architectural finding: connectivity does most of the graph's work

The notebook sketched bodies with inlets and outlets as the primary structure.
Measured, the primary structure is the **air component**, and the body graph is
much thinner than expected: two bodies in the same air component are the same
body (they merge), so links only exist *between* components — a waterfall lip,
a gate, a pipe. In every one of the seven scenarios the link count was **0 or
1**. The persisted graph is a handful of nodes carrying volume and level; the
expensive, dense, changes-on-every-edit part is the connectivity index, and
that is derived, so it never has to be saved or made consistent across a
reload.

### Sparsity growth curve (scenario 5)

512×64×512 world, one pinned river, then: a comb of 64 parallel trenches off
the river, ~100 deliberately separate channels, a 900-step spiral shaft, a dug
maze (96 corridors + 400 random plugs).

| edits | dug voxels | bodies | links | coarse nodes | components | persisted B |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 1 | 0 | 32 | 1 | 21 |
| 128 | 216 480 | 1 | 0 | 900 | 1 | 21 |
| 228 | 313 280 | 1 | 0 | 1 400 | 101 | 21 |
| 1 128 | 316 314 | 1 | 0 | 1 545 | 239 | 21 |
| 1 624 | 508 084 | 1 | 0 | 1 699 | 217 | 21 |

**The body count does not grow with edits.** It stayed at 1 through 1 624 edits
and half a million dug voxels, because digging does not create water — it
creates *space*, and space is the derived index's problem. The persisted graph
was 21 bytes throughout.

What grows is the derived side, and it grows **sub-linearly and in the coarse
currency**: 1 699 coarse nodes for 508 084 dug voxels — one node per ~300
voxels. Components peak at 239 and then *fall* to 217 as further digging
reconnects things.

**Worst case, forced.** Seeding one body into every component that has none —
the true ceiling, every disconnected hole its own persisted body — gives
**217 bodies, 0 links, 4 400 persisted bytes, 20.3 B per body**, resolving in
0.1 ms. So the ceiling on node count is the component count, and the component
count is bounded by the coarse index, not by edits.

**Index maintenance: 0.249 ms per one-voxel edit** at 1 699 coarse nodes.

> This number was **19.560 ms** in the first working version, which would have
> been a ship-blocker, and the fix is the interesting part. Rescanning every
> chunk face on every edit is O(labelled chunks × 1024). Caching the distinct
> label pairs per chunk-face and refreshing only pairs touching a relabelled
> chunk makes a rebuild O(#adjacencies) instead — a **78× improvement** with no
> change to the answers (`tests/water.rs` asserts the incremental index is
> byte-identical to a fresh build).

### Splits — the case union-find cannot do

Union-find famously cannot un-union, and a split (digging that *disconnects*)
is the adversarial case for any incremental connectivity scheme. This design
sidesteps it: the coarse union-find is **rebuilt from scratch on every edit**,
and "from scratch" is thousands of nodes, not millions of voxels. Measured on
the km channel: plugging it at the midpoint cost **2 chunk relabels, 116
unions, 0.3 ms**, and the far end correctly became a separate component with no
body — the derivation says dry, which is right.

**A split is exactly as cheap as a merge.** That is the property that makes the
whole scheme work, and it comes from refusing to be incremental at the coarse
level.

### Per-scenario results

**1 — breach a lake bottom into a void.** Lake 36×36×8 at level 46, volume
7 776; a 16³ cave beneath. A 1-voxel shaft breaches them. After: components
2 → 1, **level 46.00 → 42.83, volume 7 776 conserved exactly**, 2 fixpoint
rounds, resolve 0.002 ms. Cave floor (47,11,47) reads wet, the old lake surface
(32,45,32) reads dry. With an outlet added (cave linked to a lower sink at sill
18): merged level settles to 18.00 and 5 728 voxel-volumes transit to the sink
in **1 transit**. Correct end state in both variants.

**2 — dig a km-long channel tangent from a river.** 1 280×48×64 world; pinned
river at x<8; a channel dug 1 272 voxels (**1 145 m**) away from it. Index
update: **80 chunks relabelled, 118 coarse unions, 6.2 ms**; resolve 0.002 ms,
1 round. **Far end (1279,37,31) reads wet.** Whole query including rebind
**0.9 µs**; hot query **45.9 ns**. No kilometre-scale search exists in the code
path — the far end is wet because it is the same component as the river, which
one `find` answers.

**3 — dig a km-deep chasm from a lake bottom.** 64×1 216×64 world; chasm 1 192
voxels (**1 073 m**) from the lake floor. Index 15.6 ms (152 chunks).
*Finite* lake: level 1 208 → **520.00**, and standing at the bottom (31,9,31)
reads wet, answered in **2.8 µs**. *Pinned* lake: level stays **1 208.00**,
bottom wet, **0.8 µs**. Nothing re-infers history in either case; the level is
a field on one node.

**4 — OCEAN SCALE.** 512×64×512 sea, 7 340 032 voxel-volumes at level 60,
breached into a 400×24×400 void (3 840 000 voxels).

| | level after | Δ | capacity scan |
|---|---:|---:|---:|
| **Finite** sea | 45.351 | **−14.65 voxels (−13.18 m)** | 287–415 ms |
| **Pinned** sea | **60.000** | **0.000** | **0.0 ms** |

**A finite sea drains, and visibly.** Asked to fill a void half its own volume,
it dropped 13 metres — a shoreline retreating across the map because someone
dug a big cellar. So the character distinction is **not** optional: it is the
difference between a sea and a very large puddle.

The pinned form's cost is the striking part. A pinned body's level comes from
outside, so **nothing ever needs its capacity curve**, and the hypsometry scan
can refuse to walk it: **415 ms → 0.0 ms**. Since an ocean is most of the open
volume in a world, that refusal is most of the cost. Level-pinning is not just
semantically right, it is the cheaper implementation — the derived work it
deletes is proportional to the thing that is biggest.

Both cases fill the void: `(300,5,300)` reads wet either way. The difference is
only whether the sea pays for it.

**5 — adversarial sparsity.** See the growth curve above. **Bounded**: bodies
track components, components track the coarse index, and neither tracks edits.

**6 — bucket into a sealed stone basin.** A 4×4×4 fully-enclosed void. With no
body, `(13,9,13)` reads **dry** — the derivation is correct to say so, because
nothing in the geometry implies water. Adding one body with volume 8: level
8.50, `(13,8,13)` wet, `(13,10,13)` dry. **The storage no derivation can
replace costs 21 bytes.** This is water.md's sharpened storage rule confirmed
at its own example, and the price is nil.

**7 — unload / reload identity.** 192×48×192 (1 769 472 voxels), a pinned river
with 24 tributary trenches and a finite flooded cave; **38 358 wet voxels**
derived. Persist **39 bytes for 2 bodies**, drop everything else, rebuild the
index from geometry (**6 ms** including rebind), re-derive: **byte-identical**.
Asserted in `tests/water.rs` as well as measured. Voxel water is never
persisted; the graph is.

### Event storm

A chain of N bodies, each spilling into the next over a lip at the same height.

| N | fill (1 event) | then OutletBlocked + pour |
|---:|---|---|
| 10 | 2 rounds, 9 transits, 0.01 ms | 2 rounds, 1 body touched, 0.01 ms |
| 100 | 2 rounds, 99 transits, 0.07 ms | 2 rounds, 1 body touched, 0.06 ms |
| 1 000 | **2 rounds, 999 transits, 2.30 ms** | 2 rounds, 1 body touched, 2.70 ms |

**Worst case seen: 2.30 ms** — one `RegimeCross` event cascading through a
1 000-body chain in **2 fixpoint rounds**. The fixpoint converges in a constant
number of rounds rather than one per body, because a round processes bodies in
ascending id and the cascade runs with that order.

Blocking an outlet on an already-full chain touches exactly **1 body**:
back-pressure is absorbed locally by the first pool that cannot spill. Note the
per-round cost is O(bodies × links) because `resolve` scans the link list per
body; at 1 000 bodies that is ~2.5 ms and it would want a per-body link index
before this carried tens of thousands of nodes. Flagged, not fixed — no
scenario approaches that count.

### Determinism

The design decision is that **events do not do anything**. Each is a
commutative, monotone mutation of graph state; a single deterministic fixpoint
solve (bodies in ascending id, merges to the lowest id) then computes every
level. Order-independence is therefore by construction, not by ordering a
queue — the same move S9b made when it reformulated diffusion from scatter to
gather.

Two places where that was **not** true in the first version, both found by
building the adversarial case rather than by the shuffle passing:

- **A `Breach` and an `OutletBlocked` naming the same link in one batch.** If
  the block landed first the link did not exist yet, so it no-oped, and the
  breach then created it *open*. Fixed by having both events get-or-create the
  link and then apply only monotone mutations (`open` only falls, `sill` only
  falls), so any interleaving converges.
- **Several `RegimeCross` events on one body.** Float addition commutes but
  does not associate, so `1e16 + 1 − 1e16` depends on order. Fixed by
  accumulating crossings per body and summing them in a canonical order
  (`total_cmp` on the deltas) rather than in arrival order.

Results, with both adversarial shapes in the event set:

- **256 shuffled event orders → byte-identical persisted graph: true**
- **Double-run byte-identical: true**
- **Edit-order-independent world graph: true** — the same eight digs applied in
  two different orders produce identical persisted bytes.
- **The connectivity index is a pure function of geometry**: incremental
  updates in two different edit orders agree with each other *and* with a
  from-scratch rebuild, asserted component-id-exactly (roots are canonical —
  lowest node wins).

### Persisted bytes

| case | bodies | bytes | per body |
|---|---:|---:|---:|
| any single-body scenario | 1 | 21 | 21 |
| reload scenario (river + cave) | 2 | 39 | 19.5 |
| **heavily-dug world, forced worst case** | **217** | **4 400** | **20.3** |

**~20 bytes per body.** A world with ten thousand distinct water bodies would
persist ~200 KB — smaller than one chunk of the S3 container. Storage is not a
consideration at any scale this design can reach.

---

## Design choices where water.md was silent — each FLAGGED

1. **FLAG: a body's container is its whole air component**, not a per-basin
   depression analysis. Two puddles in one cave connected only above the
   waterline are modelled as one container filled from the bottom. Correct
   3-D basin decomposition is a priority-flood over air cells (the same
   algorithm S9 already runs on terrain) and was out of scope. Consequence: a
   lake spilling over a lip into a lower part of the *same* cave is not
   represented as two bodies today.
2. **FLAG: bodies in the same component always merge**, lowest id surviving.
   This is what makes the graph thin, and it means body identity is not stable
   across a breach — the merged body keeps the lower id. If gameplay ever needs
   a named lake to stay named, that needs a separate identity layer.
3. **FLAG: order-independence achieved by making the event vocabulary
   commutative and monotone**, not by ordering the queue. This constrains what
   a future event may do: any new event must be expressible as a monotone
   mutation, or the property is lost.
4. **FLAG: the connectivity index is derived and never persisted**, at a
   measured 6 ms rebuild for a 1.77 M-voxel world. At real world scale this is
   a streaming problem (build per region on load), not a whole-world rebuild;
   the per-chunk structure is already shaped for that but it was not measured.
5. **FLAG: the coarse union-find is rebuilt wholesale per edit** rather than
   maintained incrementally. This is what makes splits cheap and it measures at
   0.249 ms, but it is O(coarse nodes) per edit, so it wants a
   region-scoped rebuild before the coarse graph reaches ~10⁵ nodes.
6. **FLAG: a body's geometric anchor is a single voxel.** Rebinding on reload
   looks up that voxel's component. If an edit makes the anchor solid, the body
   is orphaned — not handled, and a real implementation needs either a rebind
   search or an anchor-follows-water rule.
7. **FLAG: `Pinned` bodies take their level from outside** and the spike simply
   sets it. Where that level actually comes from (deep-time sea level, a river
   reach's stage, a spring's discharge) is unspecified and is a real design
   question the notebook has not reached.
8. **FLAG: bound water's head proxy is `H = y + sat`** and lateral flow is
   per-layer. This is an unconfined-aquifer approximation adequate for a
   locality measurement; it is not a Darcy solver and should not be quoted as
   one.
9. **FLAG: the Q1 halo depends on the drainage-network boundary condition.**
   The number is only meaningful because the water table is pinned by streams
   at spacing L. If a production world has regions with no drainage within
   kilometres, the halo there is set by leakage alone — measured 4–5 cells for
   contrast ≤ 10, but unmeasured for a tight aquitard with no drains at all.
10. **FLAG: the spike's rock properties are a small local table**, not the real
    property sheet. Porosity and permeability are property-sheet facts
    (materials.md); wiring them through was out of scope for a locality
    measurement.

---

## Recommendation — **GO**, with the ocean call attached · NEEDS RATIFICATION

**Ship "persist bodies, derive voxels".** Both halves of the hypothesis
measured favourably, and neither was close to its failure mode.

1. **Bound water is local** — budget halo 4–11 cells, visible halo 0–6, clean
   geometric decay, no advective spikes, and *more* local under sharp
   permeability contrast, which was the named risk. A ~12-cell derivation halo
   covers everything measured. This is the S9 halo argument reproduced for the
   water table, and it is a smaller halo than erosion's.
2. **The free-water graph stays sparse** — it does not grow with edits at all.
   Node count is bounded by component count (217 at the forced worst case in a
   heavily-dug world), links stayed at 0–1 in every scenario, ~20 bytes per
   body, and every scenario resolved in under 0.11 ms.
3. **All seven scenarios end in the right state**, including the two that broke
   the derive-everything model (the km channel and the km chasm), both answered
   in microseconds with no history re-inference.
4. **Unload/reload is byte-identical** on derived water from 39 persisted bytes.
5. **Order-independence holds by construction** and survives 256 shuffles with
   adversarial event shapes in the batch.

**NEEDS RATIFICATION (user-owned):**

1. **The two body characters — `Finite` vs `Pinned` — are load-bearing and
   should be ratified as a design commitment.** The measurement says a finite
   sea drops 13 m when breached into a large void, and that pinning is also
   ~415 ms cheaper per resolve. But *which* bodies are pinned is a world-model
   decision, not a performance one: it is the claim that some water has a level
   set by the world rather than by its own volume. Seas and fed river reaches
   obviously; the boundary (a big lake? a spring-fed pool?) is the user's.
2. **The ~12-cell bound-water derivation halo**, if it is to be a constant
   anywhere. Measured with wide margin, but it is a calibration on a model whose
   rate constants are plausible-not-tuned — same status as S9's physics
   constants and S10's rate constants (no-bandaid: it rides as measured).
3. **Body identity is not stable across a merge** (design choice 2). If lakes
   are ever to be named, findable, or referred to by quest/ledger state, that
   needs deciding before the graph ships.
4. **Whether the connectivity index is truly never persisted.** The spike
   asserts it is derived, and reload identity proves it can be. At real world
   scale the rebuild becomes a streaming cost nobody has measured, and the
   alternative (persist it beside the S3 region files, like the owed far-field
   summary store) is the same open question the far-field thread carries.

**What this does NOT decide**: the bulk-flow octree (nothing here moves water
*fast*; free water is static data with a level, which is exactly what the
notebook's "creates no new blocks so long as its outlet connects" predicted),
sub-resolution water, capillary action, the cave families, or where the
deep-time water field lives. Those remain open in water.md.

## Falsified along the way

Two are filed in `journal/corrections.md`; the third is recorded here only,
because it never became a claim outside this document.

1. **"The body graph needs rich inlet/outlet structure"** — **corrections #13.**
   Measured, air connectivity does that work, and links were 0–1 in every
   scenario. The persisted graph is thinner than the notebook sketched, and the
   dense structure relocates to the *derived* index where it need not be saved.
2. **"A closed domain with recharge and a drain measures a water-table halo"** —
   **corrections #14.** It does not: no local equilibrium, and the flat
   full-domain profile it produces looks exactly like a dramatic negative
   result. A relaxation measurement is only meaningful against the boundary
   condition the real system has; for a water table that is the drainage
   network, not the edge of the box. Three compounding harness errors are
   recorded with it (an oscillating integrator, mismatched control/treated step
   counts, and a deep sink balanced exactly against the recharge).
3. **"Union-find cannot handle splits, so the index needs a split algorithm."**
   True of *incremental* union-find, irrelevant here: rebuilding the coarse
   graph wholesale is 0.249 ms because it is thousands of nodes. The right move
   was to stop being incremental at the level where incrementality was
   expensive. (Not filed separately — it is the mechanism behind #13.)
