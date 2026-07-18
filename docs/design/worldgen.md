# Worldgen — bounded world, lazy detail, real history

Status: design draft 2026-07-18, distilled from discussion. Firm enough to
shape S7; extent/topology intentionally left flexible.

## The core decision

**Bounded in extent; lazy — effectively infinite — in detail, including
downward. Real forward-simulated deep-time history above the region scale;
collapse-on-approach below it; unbounded wilds at the borders.**

Rationale: terrain *structure* (tectonics, climate, drainage) is a timeless
function and can be generated lazily with full coherence — but *history* is a
process with long-range causal coupling, and the Dwarf-Fortress-quality
histories we want (this ruin burned because that war happened because two
polities claimed one confluence) only come from actually running the process
forward. That requires a finite coarse world. Boundedness also buys closure:
poles, tropics, winds, currents, and tectonics as consequences of a closed
system rather than painted-on features — and a finite historical record
(legends, atlases, genealogies: a knowable world).

## The level pyramid

Generation runs on a hierarchy (~5–6 levels: planet → plates/climate →
region graph → region → locale → chunk). One rule everywhere:

> A cell's collapsed state = f(its base state, summary vector of its 1-ring
> neighbors' **base** states, the collapsed state of its **parent** cell).

Neighbor influence is one hop per level — no infinite regression — but
influence still travels arbitrarily far by riding the coarse levels (a
mountain chain planned at the 500 km level correctly rain-shadows regions
hundreds of km downwind, where it is one hop). Base states are pure functions
of (seed, coords, parent), so everything below the pregenerated levels is
deterministic, lazy, and coherent.

## Above the region scale: pregenerated, simulated

At world creation (finite, coarse cells ~10–100 km):

1. **Tectonics** — plates on a closed surface; orogeny, rifts, shelves.
2. **Climate/hydrology** — latitude bands, winds, currents, precipitation,
   drainage basins; rivers planned at region-graph scale (reach the sea by
   construction).
3. **Deep-time geology** — strata as *deposited by simulated processes* via
   the materials model (docs/design/materials.md): worldgen strata, gameplay
   middens, and live deposition are one system at different tick rates.
4. **History** — peoples, polities, trade, wars, migrations: dc-sim's coarse
   tier run over pre-player millennia. Outcomes are written as **committed
   facts in the constraint ledger** — the exact contract S2 validated
   ("worldgen pre-commits facts the live sim must honor").

The "generating world history…" pause at creation is a genre ritual, not a
cost — but it must be measured (S7) and scale with the extent knob.

## Below the region scale: lazy, unbounded in detail

Nothing is pregenerated. Regions, locales, chunks collapse on approach via
the pyramid rule, conditioned by the levels above. Consequences:

- The **extreme depth** costs nothing until descended into; deep-earth
  societies are lazily determined under the same rule (and sit in the
  statistical sim tier until observed — ARCHITECTURE.md § chunk shape).
- Voxel/detail generation is never bounded by the world's extent decision.

## After year zero

Pregenerated history provides priors and committed facts; the live
statistical tier *continues* history from there, collapsing under observation
(S2 machinery). One system, one ledger, no seam at world creation.

## Borders: unbounded wilds — DECIDED

The *civilized* world is bounded; its fringes are lazily unbounded hostile
wilderness — polar ice wastes, abyssal ocean, and always downward. No walls:
the edge fades into endless mystery, serving the mood (isolation, sublime
vastness) while the bounded interior keeps closure and history. The wilds use
the lazy machinery with no history layer — there is nothing out there to
record, which is the point.

## Extent: flexible, a player knob — DECIDED

World size is a creation-time parameter, not a constant. Working target for
default: few-hundred-km class (DF "large world" territory — enough latitude
span for real climate gradients, many instances of every landform, dozens of
cultures, dense-enough history that players collide with it). Half-Earth is
possible but likely counterproductive: sparse history reads as empty; dense
history reads as ancient. S7 must measure pregen time vs extent so the knob
gets honest labels.

## Open questions

- Topology — PROVISIONALLY DECIDED by S7 (2026-07-18): **continent-disc in a
  world-ocean.** The chunk lattice is natively planar; a wrap would force a
  seam column through every pyramid level. The disc gives border wilds on
  every compass point (deepening abyssal ocean, polar ice beyond ~84°
  synthesized latitude); climate closure via prescribed latitude/wind fields.
  Trade-off accepted: plates on a bounded grid with the ocean ring as
  closure, not a closed sphere. Revisit only if a strong reason emerges.
- Coarse cell resolution per level; history-sim tick count vs quality.
- How much of the historical record is browsable in-game at v1 (legends UI)
  vs merely present in the ledger.
- Seed + knob → reproducible worlds: how much of the coarse sim must be
  deterministic across releases (savegame compatibility policy).
