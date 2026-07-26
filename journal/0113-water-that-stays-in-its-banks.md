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

PLACEHOLDER-NUMBERS

## The invariants, verified rather than assumed

PLACEHOLDER-INVARIANTS

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
