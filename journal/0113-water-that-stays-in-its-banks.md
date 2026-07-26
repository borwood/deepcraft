# 0113 — Water that stays in its banks

*2026-07-26 · FLOW continuation (b'), the hybrid convergence exponent*

> blogworthy: **lens 4 (respect for earth processes)** and **lens 2 (procgen
> against the backdrop of priors)**. The interesting thing here is not that a
> constant became a function — everybody does that. It is *which* function, and
> that the obvious choice from the literature would have quietly destroyed the
> capability the previous slice had just bought. Two well-cited families of
> spatially varying `p` exist; one of them deletes deltas.

## The defect, stated precisely

journal/0109 shipped multi-flow-direction routing and measured the price in the
same entry:

> **peak drainage area 1,245 → 84 cells.**

The single-receiver world's biggest river drained 1,245 cells; the MFD world's
biggest drained 84. That is a **15× collapse**, and it is not a tuning artefact.
It is the statement that **a trunk river never accumulates**, because a uniform
exponent disperses at *every* cell and the loss compounds down the chain.

0109 named the cause honestly and left it: *"pure MFD is used on hillslopes and
something convergent is used in channels… We have shipped uniform `p` everywhere,
which is the simplest correct thing and is honestly the wrong long-run shape."*

The physical content of "wrong shape" is worth spelling out, because it is not
"the number was too low". A uniform exponent applies **hillslope sheet-flow
behaviour inside channels**. Water on an unchannelised hillslope genuinely
spreads: it has no banks, its width is set by the topography, and a divergent
nose sheds it in a fan. Water in a channel does not spread, even though the
interfluve beside it is downslope, because the channel has cut banks that a
cell-averaged slope field cannot see. One exponent cannot say both things, and
Holmgren's calibrated 4–6 band is precisely the **compromise** a single-exponent
scheme is forced into.

## The choice that mattered, and it is a fork

The literature offers two families, both real, both cited, and they are not
interchangeable here:

1. **`p` as a function of drainage area** (Quinn et al. 1995 is the canonical
   form: dispersive on low-area hillslopes, convergent once upslope area passes a
   threshold), or the harder version — **a single receiver above a channel
   threshold**.
2. **`p` as a function of slope** (Qin et al. 2007's MFD-md: steep ground
   converges, gentle ground disperses).

Area is the obvious pick. It is also, on this project, **the one that destroys
the slice before it.**

journal/0109's acceptance was **simultaneous divergence** — 7,548,646
within-epoch `(cell, epoch)` pairs where a cell's discharge left through two or
more faces at once. That is what makes a delta a delta rather than a channel that
wanders. And distributaries, alluvial-fan tops and braid plains are exactly the
places with the **largest** drainage area on the world. An area-only law makes
them the *most convergent* ground there is, and concurrent distributaries vanish
— you would recover the peak catchment by deleting the delta.

Slope alone is not right either. It concentrates gorges, which is correct, but it
has no way to tell a 1-cell hillslope rill from a trunk river on the same
gradient, so hillslopes stay under-dispersed or trunks stay under-concentrated
depending on where you put the constant.

**What the two families are each reaching for is confinement**, and geomorphology
has a standard index for it that uses both: **Montgomery & Dietrich (1988, 1992)'s
channel-initiation criterion, `χ = A · S²`.** Channel heads on a real landscape
occur where the drainage-area × slope-squared product exceeds a critical value.
Run our three regimes through it:

| regime | `A` | `S` | `χ` | `p` | what it does |
|---|---|---|---|---|---|
| hillslope / interfluve | small | any | low | `p_hill` | sheet flow, spreads |
| trunk river, gorge, incised valley | large | moderate–high | **high** | `p_chan` | stays in its banks |
| **delta top, fan, coastal plain** | large | **≈ 0** | **low** | `p_hill` | **splits — distributaries** |

One expression, three regimes, and the third one falls out for the *physically
correct reason*: **a delta is where a channel loses its confinement.** That is not
a special case bolted on to protect an acceptance number; it is what the criterion
says, and it is why the criterion was the right thing to reach for rather than a
free parameter.

The endpoints then stop being a compromise. `p_hill = 1` is Quinn/Freeman's
dispersive limit — which is what unchannelised overland flow *is*. `p_chan = 16`
is convergent enough that a neighbour at 90 % of the steepest slope keeps 19 % of
its weight and one at 70 % keeps 0.3 %. **Holmgren's 4–6 was never a measurement
of a landscape; it was a measurement of the compromise.**

## Three things the build had to settle

### 1. The circularity, and why the lag is not a cheat

`p` needs `A`. `A` is accumulated **from the weights `p` produces**. That is a
genuine circular dependency, and — this is the part worth recording — **there is
no fixed point to iterate toward.** A second accumulation pass would compute an
area from weights that the new area then invalidates; a third would do it again.
The system does not converge to a self-consistent partition, it just costs more.

So the exponent reads `self.area` as the routing phase finds it: **the previous
epoch's accumulation.** This is the S10 biotic coupling's shape exactly
(`grid.bio_weather` is a lagged plane for the same reason) and it costs nothing —
no extra storage, no extra pass, and `accumulate_area` re-seeds the plane at its
own start so nothing stale leaks into the sums.

At epoch 0 the plane is zero, so `χ = 0` and the first epoch is maximally
dispersive everywhere. That is not a defect to paper over; it is the honest
initial condition and it is also the physical statement — **nothing is channelised
until water has run once.** Over 200 epochs the network self-organises: area
concentrates, `χ` rises, `p` rises, and the concentration reinforces. The feedback
is bounded above by the single-receiver limit, so it tightens rather than runs
away.

### 2. Normalising the slopes, which turned out to be a correctness fix

The partition was `w_k ∝ S_k^p · L_k`. Raising `p_chan` to 16 (or a future 32)
runs `S^p` on gradients this world actually has — and its gentle ones are gentle:
`S ~ 10⁻⁵` per cell width in the flats. `(10⁻⁵)^64` underflows to zero. Every
direction underflows, the partition sums to zero, and **`partition_cell` reports a
draining cell as a sink** — silently disconnecting a piece of the drainage
network, at a threshold that depends on the world's smallest slope.

The fix is to divide every slope by `S_max` before exponentiating. Algebraically
it is a no-op: the renormalisation at the end divides it straight back out. But it
puts every base in `(0, 1]`, the steepest direction's weight is exactly `1`, and
the safe exponent range stops being coupled to the terrain. There is a test that
pins it directly — `a_steep_exponent_on_a_gentle_slope_still_finds_a_receiver`.

### 3. The exponent is rounded to an integer, and that is a stated cost decision

journal/0109 already noted why: `powf` on `8 directions × 297,025 cells × 200
epochs` is half a billion transcendental calls and would dominate the deep run,
where an integer `powi` is a handful of multiplies. A uniform `p` could cache one
integer; a *ramp* cannot. So the ramp rounds.

At a 460 m tier where `p` is a coarse sub-grid dial, the difference between `7`
and `7.3` is false precision and the difference in gen time is not. Uniform mode
does not round, so a probe can still sweep fractional `p`.

## The honest limit: what a 460 m cell is

At 460 m a single cell contains an entire hillslope-and-channel system. Every real
channel-initiation threshold in the literature — a few hectares in a humid
temperate landscape — is *smaller than one cell of this grid*. So this is **not**
"is this cell a channel". It is a **sub-grid parameterisation of how much of the
cell's discharge is confined**, and the thresholds have to be calibrated against
this world's own `χ` distribution rather than lifted from a field study at 10 m.

That is filed as **stubs #26**, and the reason it is a stub rather than a knob is
that `S` here is a rise per **cell width**, so `χ` carries the grid's resolution
inside it: at a coarser extent the same landscape produces smaller `χ` and the
same two constants describe a different fraction of it as channelised. The heirs
are the joint supply+transport calibration (which would give `χ` a physical scale
to derive the threshold *from*) or a dimensionless re-expression of the index.

## The numbers

Seed 1337, `Extent::Medium`, 297,025 deep cells of which **44,264 are land**,
200 epochs, shipped configuration otherwise. Three routings on the same pregen:
**D8** (`mfd: false`), **uniform `p = 4`** (journal/0109's solve, reached by
pinning `p_chan == p_hill`), and **hybrid**.

### Acceptance 1 — peak catchment

| | D8 | uniform `p = 4` | **hybrid** |
|---|---|---|---|
| peak catchment (cells) | 1,255 | **84** | **298** |
| as a share of D8's peak | 100 % | 6.7 % | **23.8 %** |

**84 → 298, a 3.5× recovery.** Not all the way back to D8, and it should not be:
D8 is the sheet-flow-free extreme, and a landscape where nothing ever spreads is
as wrong as one where everything does. (D8's peak reads 1,255 here against
journal/0109's 1,245 — the same quantity on a world that has since gained
material-aware transport and material-aware creep. The uniform-`p` figure, 84, is
identical to the digit.)

### Acceptance 2 — the catchment distribution

The peak is one cell, and one cell is noise. The distribution is the claim:

| land-cell catchment | D8 | uniform `p = 4` | **hybrid** |
|---|---|---|---|
| p50 | 20.0 | 27.8 | 23.0 |
| p90 | 51.0 | 51.7 | **60.2** |
| p99 | 127.0 | 70.0 | **164.4** |
| p99.9 | 298.0 | 78.2 | 213.8 |
| **top-1 % share of all land drainage area** | 0.0849 | 0.0248 | **0.0613** |
| peak / mean catchment | 41.4 | 2.8 | **9.4** |
| land cells with a catchment > 100 | 530 | **0** | **2,270** |

Two rows are worth stopping on.

**`land cells with A > 100` went 0 → 2,270.** Under uniform `p` the shipped world
contained **not one cell** whose drainage exceeded a hundred cells. There was no
trunk network at all — not a weak one, none.

**At p99 the hybrid solve beats D8**: 164.4 against 127.0. That is not a rounding
artefact and it is the most interesting number in the slice. A single-receiver
network is a *tree*: every cell hands its water to exactly one parent, so the
mid-range of the distribution is thin — area either sits in the trunk or it does
not. The hybrid solve disperses on the hillslopes, so a channel collects from a
*fan* of upslope cells instead of a single tributary line, and then keeps what it
collected. The result is a network with **more** moderately large channels than
D8 and a smaller single largest one. That is a better description of a real
drainage basin than either extreme, and it fell out of the law rather than being
aimed at.

### Acceptance 3 — simultaneous divergence must survive, and it does

| | D8 | uniform `p = 4` | **hybrid** |
|---|---|---|---|
| `(cell, epoch)` pairs with ≥2 lateral out-faces | **0** | 7,548,535 | **7,228,964** |
| distinct `(cell, chapter)` pairs | **0** | 395,449 | **398,955** |
| most lateral out-faces in one epoch | 0 | 8 | 8 |
| temporal (avulsion) `(cell, chapter)` pairs | 337,406 | 418,889 | 417,981 |

**95.8 % of the simultaneous divergence survives** — and the `(cell, chapter)`
count is *higher* than uniform `p`'s. Concentrating the trunks cost the deltas
almost nothing, which is the whole argument for `A·S²` over `A` alone, measured
rather than asserted. The D8 column is still a structural zero and always will be.

### Acceptance 4 — mass

| | |
|---|---|
| `Δ(ΣR + ΣH) − uplift − biotic` | **+1.28 × 10⁻⁵ m** on a total mass of 5.63 × 10⁸ m — a relative residual of **2 × 10⁻¹⁴** |
| worst **per-species** split residue over the whole run | **2.18 × 10⁻¹⁶** (relative; one ulp) |

### The calibration, and the trade it hides

The `χ` distribution over 44,204 draining land cells on the shipped world (`S`
dimensionless):

| | p1 | p10 | p50 | p75 | p90 | p99 | max |
|---|---|---|---|---|---|---|---|
| `χ = A·S²` | 6.3e−5 | 2.6e−3 | 2.0e−2 | 4.5e−2 | 1.07e−1 | 3.3e−1 | 5.8e−1 |
| `S` | 4.1e−3 | 1.4e−2 | 3.3e−2 | 4.3e−2 | 4.9e−2 | 5.8e−2 | 6.2e−2 |

At the shipped thresholds that puts **64.3 % of land dispersive, 26.8 % on the
ramp, and 8.9 % channelised.**

`chi_hi` is the knob that decides how much of the world is treated as confined,
and it trades concentration against divergence. So it is reported as a **curve**,
not defended as a point (`chi_lo` held at `chi_hi/4` so only the switch moves):

| `chi_hi` | land channelised | peak | **p99** | **top-1 % share** | simultaneous `(cell,epoch)` |
|---|---|---|---|---|---|
| 3.0e−2 | 35.8 % | **355** | 115.5 | 0.0463 | 5,724,055 |
| 6.0e−2 | 20.2 % | 251 | 150.2 | 0.0572 | 6,677,499 |
| **1.2e−1 (shipped)** | **8.9 %** | 298 | **164.4** | **0.0613** | **7,228,964** |
| 2.4e−1 | 3.2 % | 290 | 155.7 | 0.0588 | 7,491,081 |

**Read the first row before assuming more channelisation is better.** At
`chi_hi = 3e−2` the single largest catchment is the biggest of the four — 355 —
and *both* proper concentration measures are the **worst** of the four (p99
115.5, top-1 % 0.0463), while a fifth of the simultaneous divergence is gone.
Channelising a third of the land does not build a drainage network; it builds
thousands of parallel independent threads that never merge, which raises one
extremum and flattens everything else. The shipped point maximises p99 *and* the
top-1 % share *and* retains the most divergence of any concentrating setting. It
was not chosen to; it is where the curve put it.

### `MFD_MIN_WEIGHT` — stub #22, measured at last

The stub said the floor's bias was *"argued, **not measured**"*. Now it is —
hybrid `p` with the floor at 1 % against the floor **off**:

| | floor 1 % | floor off | delta |
|---|---|---|---|
| peak catchment | 298 | 305 | **−2.1 %** |
| simultaneous `(cell, epoch)` | 7,228,964 | 7,248,627 | **−0.3 %** |
| record entries | 4,107,188 | 4,171,836 | −1.5 % |
| record residency | 63.80 MiB | 64.79 MiB | −1.5 % |

So the floor's effect on the *physics* is real but small — and it points the way
the stub predicted, slightly **less** concentrated rather than more. It buys 1.5 %
of the record for 2 % of the peak catchment. **The floor is now a knob**
(`DeepConfig::mfd_min_weight`, default unchanged at `0.01`) — not in order to move
it, but so that the number above exists at all. A knob is not an heir, and stub
#22 stays open.

### Cost

| | D8 | uniform `p = 4` | hybrid |
|---|---|---|---|
| deep run | 26.87 s | 30.67 s | **30.71 s** |
| record entries | 2,897,904 | 3,735,614 | 4,107,188 |
| record residency | 45.35 MiB | 58.13 MiB | **63.80 MiB** |
| `DeepField` residency | 174.35 MiB | 187.21 MiB | **192.32 MiB** |

**Gen time against uniform `p`: +0.04 s, inside the noise.** The integer-rounded
ramp keeps the partition on `powi`, and the channel switch is *cheaper* than a
partition — one weight, no loop. Nothing here is on a runtime path. Residency
against uniform `p` is **+5.67 MiB of record**, bought by the dispersive
hillslopes: `p_hill = 1` keeps more sub-dominant receivers than `p = 4` did, and
those are entries. That is the honest price of putting the endpoints at the
endpoints.

## The invariants, verified rather than assumed

All four survive, and each was checked by something that could have failed rather
than by reading the diff.

**The traversal licence.** journal/0109's finding was that the priority-flood pop
order is a topological order of the DAG *because every routed edge descends the
potential* — the tree never licensed it, the potential did. Hybrid `p` reweights
edges and, above `chi_hi`, deletes all but one; it never creates one. Both the
ramp and the switch draw only from directions with `drop > 0` on `filled`, and the
switch takes the **steepest** of exactly those. Verified by
`mfd_routing.rs::every_routed_edge_descends_the_free_surface_potential`, which
walks the live final epoch under the shipped law.

**Mass with no residue.** Untouched — deliberately. The residual rule lives in
`accumulate_area` and `transport`; this slice changed neither, only how `mfd_w` is
*filled*. In the channelised case the rule is trivially exact: one weighted
direction, which is therefore also the last, so it takes `q − 0`. Verified by
`mass_is_conserved_with_mfd_on` and by the probe's whole-world ledger
(2 × 10⁻¹⁴ relative).

**Per-species budgets.** Also untouched, and also verified rather than reasoned:
`the_partition_leaves_no_residue`, `no_species_leaks_at_its_own_junction`,
`a_shared_total_leaks_per_species_and_an_own_budget_does_not`, and the probe's
worst per-species split residue over a 200-epoch run — **2.18 × 10⁻¹⁶**, one ulp.

**A-3, the live risk.** *The record may only describe water the solve actually
moved*, and the value written must be the same `share` that was added to the
neighbour, never re-derived from the weights. A variable exponent makes
re-derivation tempting — the weights are right there. It was not done: the
recorder's write sites are byte-for-byte the ones journal/0109 left, and this
slice touched no line between a `share` and its `out_area` / `out_face_load`
store. Checked by grepping those hunks back out of the creep merge intact, and
guarded by `the_partition_leaves_no_residue`, which compares the **record's**
per-face plane against the **solve's** accumulated area and fails the moment the
two arithmetics diverge.

**And one invariant that had to be restored rather than merely preserved.** The
`S/S_max` normalisation changes the uniform path's last ulp, and the first gate
caught what that costs: **three** cross-commit fixed points — the pre-MFD world,
the pre-2b scalar-load world, and the anonymous-creep world — stopped being
byte-reachable. They are not decoration; they are the evidence that each slice
*added* a path rather than perturbing the old one. So the normalisation is applied
**only when the law is non-uniform** (`x / 1.0` is exact, so a uniform law runs
journal/0109's arithmetic bit for bit). The price, stated where a reader will meet
it: a *uniform* law keeps 0109's exponent-range limit. Uniform `p` is a control,
not a shipping mode, and the ramp — where the large exponents actually live — is
normalised.

## What this is for, and what it is not for

**Not for**: denudation, the facies gradient, or the look of the world.
journal/0111 measured this world exporting **0.02 %** of its sediment by rivers,
at a catchment-averaged denudation of 0.0110 m/Myr — 9× slower than the slowest
landscape ever measured on Earth. Hybrid `p` concentrates that 0.02 %. Anyone who
measures denudation before and after this slice will measure a null, and the null
will say nothing about the slice. The joint supply+transport calibration is the
fix for the magnitude, and it is somebody else's.

**For**: the **refinement tier**, which was opened as a sequenced arc the same day
this was built. `flow.md` § 4 says refinement is *"solving a small boundary-value
problem inside a cell, with **face fluxes as Dirichlet conditions**."* Visible
river channels are the first thing to be built on it.

> **A channel cannot be refined out of a flux record that never concentrates.**

That is the whole justification for the slice, and it is a *substrate* argument
rather than an appearance one. Under uniform `p` the record handed refinement a
world whose largest boundary condition was 84 cells of accumulated discharge,
spread across up to eight faces. There is no channel in that. Under hybrid `p` the
record hands it a trunk network with the discharge concentrated on one or two
faces per cell where the flow is confined, and genuinely split where it is not —
which is exactly the Dirichlet data a boundary-value problem needs in order to
have a channel as its solution.

## A correction found on the way

journal/0109 wrote — and flow.md and `DeepConfig` repeated — that *"`p → ∞` is
single-receiver D8, **exactly**"*. It is not, and the falsifier was already in the
same slice: `route_cell` takes the steepest **drop** and the partition takes the
steepest **slope**, and they differ on diagonals by exactly the `√2` flow-path
length that 0109 introduced four lines earlier. `the_partition_follows_slope_not_drop`
pins a case where the two choose different receivers.

Nothing shipped is wrong — the limit the partition converges to is the *better*
one. What is wrong is the word "exactly", and it mattered because that sentence
was the argument that `mfd: false` lives *inside* the new model's family. It does
not: it is a separate pinned byte-identical path, which is why it needs its own
golden. **corrections #58**, struck at all three sites.
