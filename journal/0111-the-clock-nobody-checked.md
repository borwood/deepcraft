# 0111 — The clock nobody checked

*2026-07-26 · the denudation measurement — one number, one published answer*

> blogworthy: **lens 4 (respect for earth processes)** first, then **lens 2
> (procgen against the backdrop of priors)**. This is the entry where the project
> takes a quantity it has been simulating for a month and holds it up against the
> literature for the first time — and finds it is **~500× below the median
> measured erosion rate on Earth, and 9× below the slowest surface ever measured
> anywhere.** The interesting part is not that a number was wrong. It is *which*
> number, *why nobody could have noticed*, and the fact that the world's shape
> turned out to be right while its rate was off by three orders of magnitude.

## The question, and why it could be answered at all

journal/0110 shipped a null. Movement 2b made the fluvial load material-aware,
correctly, and it changed nothing, because the probe it was required to carry
measured **fluvial transport at 0.109 % of this world's sediment routing** —
hillslope creep moves 918× more, and no cell on the world can carry sand
(corrections #55).

That left a fork the project could not settle by argument:

- **the honest craton.** In-place weathering plus creep dominating is exactly
  what a low-relief cratonic interior looks like. Rivers moving a thousandth of
  the sediment would then be *correct*, the facies acceptance test simply would
  not apply, and nothing is broken.
- **the broken energy budget.** Almost nothing is eroding at all, and the fix is
  structural.

The user's framing was the whole brief: *"if cratonic is honest, it's honest."*

**Denudation rate is the discriminator**, and it is one of the very few
quantities in this project with a **real published external anchor**. Stable
cratons, passive margins and active orogens occupy well-separated, well-measured
bands. A landscape inside a band is a landscape. A landscape below all of them is
not a slow landscape — it is a stopped clock.

## Getting the definition right is most of the work

"Denudation rate" has several defensible readings and **they do not give the same
number**, so the probe names the one it leads with and reports the others beside
it.

The headline is **catchment-averaged denudation**: mass **exported from the
subaerial land system**, per unit *land* area, per unit time. That is the reading
a gauging station and a cosmogenic `10Be` catchment average both measure. And it
is the strict one:

> Material that merely redistributes within the landscape is **not** denudation.
> Weathered in place: no. Crept one cell downslope: no. **Crossed the shoreline:
> yes.**

That distinction is the entire measurement. The existing `TransportLedger` from
0110 already carried `deposited_at_sink_m` — but a sink in this engine is *either*
a submerged cell *or* a subaerial domain-border cell, and the second is not export
at all, it is sediment piling against the edge of a simulated box. Counting the
two together would have been the exact defect this repo keeps catching: a missing
row in an itemisation. So the ledger grew a **split**, and the gate asserts the
two halves re-sum to the total the old field already held.

Then the honest question: *what else crosses the shoreline?* Four things, and the
ledger now counts all four separately — fluvial yield to the sea, **regolith crept
across the shore**, wave-quarried rock sent offshore, and dust settling on water.

**Two things decide whether any of it is trustworthy, and both had to be derived
rather than assumed.**

### 1. The time calibration, because every number is linear in it

`earth-processes.md` § 3e-2 decision 5, ratified 2026-07-19, sets the
**Phanerozoic register**: *"the recorded span calibrates to ~500 Myr."*
`DeepConfig::chapters` agrees from a completely different direction: *"K=8 gives
Earth-orogeny-length chapters (62.5 Myr)"* — and 8 × 62.5 = 500. Two independent
statements, one answer. The run is 200 epochs, so **1 epoch = 2.5 Myr**.

Confidence is high *on the intent* — and the intent is all there is. This is a
**stipulated** register, never a fitted one. § 3e's own owed list still reads
"calibrate iteration↔Myr against a real orogen." **No physical rate in this
engine has ever been checked against this clock.** That sentence turned out to be
the finding, so the probe also prints every rate per *epoch*, which is
calibration-free.

### 2. Land area, not world area

14.9 % of cells are subaerial. Dividing by the whole grid would have understated
the rate 6.7×. And because the sea stand cycles ±35 m, the land count itself moves
±4.1 % over the run; that is printed as the denominator's own error bar.

## The number

**Seed 1337, `Extent::Medium`, 297,025 cells at 460 m, 200 epochs, 44,264 land
cells (9,366 km²).**

| definition | m/Myr |
|---|---|
| **D1 catchment-averaged denudation (export / land area / time)** | **0.0110** |
| D2 mean surface lowering | **−0.4084** (the land is *building*) |
| D3 bedrock erosion (incision + weathering-front descent) | 0.0112 |
| D4 rock uplift | 0.4095 |

Over the whole 500 Myr the land system exported **5.48 m** of average thickness.

Where it went:

| export channel | metres | share |
|---|---|---|
| regolith crept across the shoreline | 232,746 | **96.0 %** |
| wave-quarried, sent offshore | 8,849 | 3.7 % |
| dust settled on the sea | 743 | 0.3 % |
| **fluvial yield to the sea** | **48.6** | **0.02 %** |

Rivers deliver **one five-thousandth** of this world's sediment to the sea. 0110
measured them at 0.109 % of *routing*; at the shoreline they are 0.02 % of
*yield*. The transport pass is even less of this landscape than the last entry
found.

### The two instruments that agree

D1 is a **boundary-flux** accounting and it inherits a real uncertainty: the
land/sea budget closes to within 218,528 m, which is 90 % of the export term
itself, because ±35 m of sea-level cycling shuffles cells between land and sea and
the shoreline is not a clean control surface.

D3 is a **per-cell rock-removal plane** (`grid.exhum`) with no shoreline in it at
all. It agrees with D1 **to 2.4 %**.

Two instruments that share no arithmetic and can fail in unrelated ways land on
the same number. That is what licenses reporting it.

It also says something: **98 % of every metre of bedrock this world detaches
leaves the land system.** Nothing is piling up. At the shipped calibration the
landscape is **supply-limited** — the weathering constant *is* the denudation
rate.

## Against the literature

All in m/Myr (= mm/kyr = µm/yr).

| band | range | this world |
|---|---|---|
| **floor — McMurdo Dry Valleys / hyperarid Atacama** | 0.1 – 1 | **9× slower** |
| stable craton / shield bedrock | 1 – 10 | 91× slower |
| global outcrop median (`10Be`, n = 1599) | 5.4 – 12 | 493× slower |
| Phanerozoic global continental mean | 16 – 62 | 1,461× slower |
| passive-margin upland (Appalachians) | 27 – 40 | 2,465× slower |
| active orogen (Taiwan, Himalaya, S. Alps) | 3,000 – 12,000 | 273,926× slower |

Sources: Portenga & Bierman 2011 *GSA Today* (the global `10Be` compilation —
median **5.4**, mean 12, max ~140); Bierman & Caffee 2002 *GSA Bull* (Australian
shield inselbergs, 0.3–5.7) and 2001 *Am. J. Sci.* (Namib bedrock 1–5);
Veselovskiy et al. 2019 *Tectonics* (Fennoscandian AFT 1–2.5); Matmon, Bierman et
al. 2003 *Geology* (Great Smokies 27 ± 4); Wilkinson & McElroy 2007 *GSA Bull*
(Phanerozoic mean 16 from preserved sediment volumes, 62 from modern natural
yield); Dadson et al. 2003 *Nature* (Taiwan 3,000–6,000); Herman et al. 2013
*Nature* (Himalaya 7,000–12,000); Koppes & Montgomery 2009 *Nat. Geosci.*
(>10,000 local); Morgan et al. 2010 *JGR-ES* (McMurdo 0.1–4, Arena Valley ~0.19)
and Ritter et al. 2023 *JGR-ES* (Atacama near-stasis, `21Ne` exposure ages of
9–37 Myr).

**Honest uncertainty in the comparison, and it cuts one way.** Cosmogenic rates
integrate 10³–10⁵ yr and so record what a craton is doing *right now*, in a quiet
phase. The right comparator for a 500 Myr run is **thermochronology**, which
integrates 10⁶–10⁸ yr — and thermochronology says cratons are *punctuated*, not
uniformly slow: Kola sheds 3–5 km over the Phanerozoic, the Pilbara sheds
multiple km in discrete Paleozoic pulses (Morón et al. 2020 *Tectonics*), and the
South African plateau sheds ≥4.5 km in just 130 Myr (Brown et al. 2002). The
geologically plausible time-averaged rate for a *quiet* craton over a full
Phanerozoic span is **~10–20 m/Myr**, stripping **5–10 km**.

This world strips **5.48 m**. About **a thousandth**.

The uncertainty in the literature (Wilkinson & McElroy's 16 vs their own 62;
Willenbring & von Blanckenburg 2010 arguing rates have been stable against
Herman-style acceleration claims) spans a factor of ~4. The gap being measured
here is a factor of ~10³. The disagreement in the literature cannot reach it.

## The distribution — and the sentence that settles the fork

A craton-like *average* can hide active margins, so the probe reports the whole
distribution of per-cell bedrock erosion across 44,264 land cells:

| | m/Myr |
|---|---|
| min | 0.0000 |
| p25 | 0.0062 |
| **median** | **0.0104** |
| p75 | 0.0141 |
| p90 | 0.0181 |
| p99 | 0.0468 |
| p99.9 | 0.0934 |
| **max** | **0.1341** |

The most active 10 % of land does 25.8 % of the erosion (a uniform surface would
give 10 %), max/median is 12.9×. So the world is **not** flat-dead — it has real
erosional structure, and that structure is worth keeping.

But:

> **The single most active cell on the entire world, at 0.134 m/Myr, is still
> slower than bare Antarctic bedrock in the McMurdo Dry Valleys.**

There are no active margins. There is a quiet interior and some slightly less
dead ground. The whole distribution is compressed inside a band no instrument on
Earth would call erosion.

**Denudation / uplift = 0.027.** In topographic steady state that ratio is 1.
Here erosion removes 2.7 % of what uplift adds, the mean land surface *rose* 204 m
over the run, and the landscape's shape is 97 % tectonic. Erosion has essentially
no authority over the topography of this world.

## The verdict, and why the honest answer needed one more experiment

The evidence above says **the energy budget is broken, not cratonic.** A craton
erodes; this does not.

But "broken" has two very different repairs, and the difference between them is
the difference between a calibration and an architecture. So the probe sweeps the
**shipped** `erosion_budget` override — which scales `weathering`, `k_transport`
and `k_bedrock` together — and measures the world's *response*. A rate that is
**linear** in a knob is that knob. A rate that **saturates** is being held by
something the knob cannot reach.

| scenario | D1 (m/Myr) | D3 | D1/D3 | D1/D4 | vs production |
|---|---|---|---|---|---|
| **PRODUCTION (1×)** | 0.0110 | 0.0112 | 0.98 | 0.027 | 1.0× |
| budget 10× | 0.0140 | 0.0281 | 0.50 | 0.034 | 1.3× |
| budget 100× | 0.0149 | 0.0416 | 0.36 | 0.036 | **1.4×** |
| creep 10× only | 0.0181 | 0.0112 | 1.63 | 0.044 | **1.7×** |
| budget 10× + creep 10× | 0.0801 | 0.0455 | 1.76 | 0.184 | 7.3× |
| **budget 100× + creep 10×** | **1.4474** | 0.2976 | 4.86 | 3.056 | **132×** |

**Nothing in production was changed by this. These are hypothetical configs built
inside the probe to measure a response.** That distinction is the whole reason the
sweep is legitimate rather than tuning: it reports what the world *does*, and the
one thing it must never do is pick a value because the output looked right.

### What the table says

**Neither lever pays alone.** A hundredfold erosion budget buys 1.4×. A tenfold
creep buys 1.7×. **Together they buy 132× — 59× more than their separate gains
multiplied.**

The mechanism is legible in the `D1/D3` column. At production the ratio is 0.98:
the land sheds everything it detaches, and the weathering constant *is* the rate.
Turn the budget up alone and the ratio collapses to 0.36 — the land is now making
regolith it cannot move, and the **cover taper** `exp(−H/H*)` with `H* = 3 m`
shuts weathering off from underneath. The extra regolith shields the rock that
made it.

**That taper is the structural cap.** It is also, for what it is worth, *correct
physics* — a thick soil really does slow its own bedrock weathering front. The
defect is not the taper. The defect is that the only knob the project has for
"more erosion" cannot reach the process that does 96 % of the eroding.

`erosion_budget` scales `weathering`, `k_transport` and `k_bedrock`. It does
**not** scale `diffusion`. Its own doc comment calls it "**the** TERRAIN (erosion)
amplitude" and says it lets "the total amount of material erosion" move. Measured:
at 100× it moves the total by 1.4×. **A knob that cannot move the thing it is
named after** — corrections #56, stubs #24.

## The answer to the fork, stated plainly

**The energy budget is broken.** Not ambiguously, not marginally:

- the land average is **9× below the slowest landscape ever measured on Earth**;
- **so is the single most active cell on the world**, so it is not a quiet
  interior with live margins;
- 500 Myr of denudation strips **5.48 m** where a real craton strips 5–10 km;
- denudation is **2.7 %** of uplift, so the landscape has never approached
  topographic steady state;
- and the world is one calibrated order of magnitude in *two coupled terms* away
  from the bottom of the real craton band, which is where a landscape like this
  one ought to sit.

**And the shape is right.** This is the part worth being careful about, because
the temptation on reading the numbers above is to conclude the model is wrong.
It is not. A weathering-limited landscape whose sediment is routed by hillslope
diffusion, whose rivers are minor, whose regolith armours its own weathering
front, and whose erosion concentrates mildly on steep ground — that is a
*textbook* low-relief craton. Every qualitative statement journal/0110 made about
this world is confirmed. Only the **rate** is wrong, and it is wrong because the
rates were chosen to build visible relief in 200 iterations and then a clock was
stipulated over the top of them without anyone multiplying the two together.

Movement 2b's null is therefore **honest about the mechanism and dishonest about
the cause**. The facies gradient did not express because this world's rivers do
nothing — true. But its rivers do nothing partly because *nothing* does anything,
and that was never a fact about rivers.

## What this entry is really about

It is the third instance in three days of the same failure and the first one that
was caught by asking an *external* question.

journal/0109 mistook a claim about **order** for a claim about **substance**
(twice). corrections #55 mistook a claim about **naming** for a claim about
**magnitude**. This one is a claim about **units** — *the recorded span is 500 Myr*
— sitting beside a set of physical constants in metres-per-iteration, with nobody
ever having divided one by the other. Three disguises, one root: **a quantity
believed because it was written down, never because it was measured.**

The defence that worked here is different from the previous two and worth naming.
The others were caught by internal instruments — a ledger, a competence ceiling.
This one could not have been: every internal check the engine has says the
simulation is consistent, and it *is* consistent. Mass closes, the goldens hold,
the passes are pure. **A closed system cannot detect its own scale error.** It
took an anchor from outside the corpus — published erosion rates, a floor set by
the Antarctic Dry Valleys — to see it at all.

That is the argument for reaching for real-world numbers whenever one exists.
Not for fidelity's sake. Because they are the only errors an otherwise perfect
internal audit is structurally blind to.

## What it cost

The instrumentation is behind `DeepConfig::denudation_ledger`, **off in
production**: zero counters, no branch, and no shoreline-creep sweep, so the
shipped run is byte- *and* cost-identical. The load-bearing gate test asserts the
bedrock and regolith planes are **bit-identical** with the flag on — which is what
makes a number taken with it on a number about the shipped world. Three gate
tests, **12.5 s** added. No golden moved.
