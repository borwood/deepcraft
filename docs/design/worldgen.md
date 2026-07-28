# Worldgen — bounded world, lazy detail, real history

Status: design draft 2026-07-18, distilled from discussion. Firm enough to
shape S7; extent/topology intentionally left flexible.

> ## ⏸ THE HISTORY LAYER IS **ON HOLD** — user, 2026-07-28
>
> **History is coming, eventually, for the reasons this document states** (user's
> words; and *"not exhaustive"* — the rationale below is a floor, not a ceiling).
> What is *not* true, and what this banner exists to stop a reader believing, is
> that any of it **exists** or is **in the pipeline today**.
>
> **What happened.** A settlement-history stage did run here for about a week —
> peoples, polities, expansion, conflict, sacked sites, ruin posts. It was
> **fabricated during bring-up by nobody's design**, was never ratified, and was
> **removed 2026-07-28** (journal/0121; the world lost 102 wood voxels and no
> golden moved). The user, 2026-07-26: *"unratified zealous fabrications from the
> early bootstrapping of the project… they WILL be wholesale replaced."*
>
> **The distinction this document must carry, because CLAUDE.md's rule is easy to
> over-apply:** the *ambition* recorded here is **user-originated and survives**;
> the *implementation* was unratified content and is gone. **Recorded ambition is
> not fabrication.** Do not read the removal as a retirement of the design.
>
> **When it returns is gated, and the gate is a user call** — see § *Sequencing*
> at the foot of this document. Nothing here is scheduled, and nothing here may be
> cited as an existing capability.
>
> **Sections affected:** the title's *"real history"* · § *The core decision* ·
> § *Above the region scale* item 4 · § *After year zero* · § *Borders* · the
> history-density half of § *Extent*.

## The core decision

**Bounded in extent; lazy — effectively infinite — in detail, including
downward. Real forward-simulated deep-time history above the region scale
(⏸ ON HOLD 2026-07-28 — see the banner); collapse-on-approach below it;
unbounded wilds at the borders.**

Rationale: terrain *structure* (tectonics, climate, drainage) is a timeless
function and can be generated lazily with full coherence — but *history* is a
process with long-range causal coupling, and the Dwarf-Fortress-quality
histories we want (this ruin burned because that war happened because two
polities claimed one confluence) only come from actually running the process
forward. That requires a finite coarse world. Boundedness also buys closure:
poles, tropics, winds, currents, and tectonics as consequences of a closed
system rather than painted-on features — and a finite historical record
(legends, atlases, genealogies: a knowable world).

> **⚠ WHICH LEG IS CARRYING BOUNDEDNESS RIGHT NOW.** The decision above rests on
> **two independent** arguments, and one of them is currently on hold:
> - **History needs a finite coarse world** — ⏸ on hold with the layer itself.
> - **Closure** — poles, tropics, winds, currents, drainage and tectonics as
>   consequences of a *closed system* rather than painted-on features. **This leg
>   is pure earth science, is live, and is load-bearing today.**
>
> So **bounded-in-extent stands on its own** and is not weakened by the hold. This
> note exists because the alternative — a ratified decision visibly resting on a
> suspended premise, with nothing saying the other leg holds — is precisely the
> shape we log as a justification outliving its premise (spines A-2). It is
> recorded here so the next reader does not have to re-derive it, and so nobody
> "discovers" that the boundedness argument has collapsed. It has not.

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
4. **History** — ⏸ **ON HOLD 2026-07-28; NOT A PIPELINE STAGE TODAY.** *Intended*:
   peoples, polities, trade, wars, migrations run over pre-player millennia,
   outcomes written as **committed facts in a constraint ledger** — the contract
   S2 validated ("worldgen pre-commits facts the live sim must honor").
   **Status:** the pregen pipeline has **three** stages. The fourth was
   unratified bootstrap content and was removed 2026-07-28 (journal/0121). The
   S2 ledger machinery it drove still exists, has **no production caller**, and is
   **kept deliberately as a shape reference** to be re-checked against real
   requirements when this stage is actually designed (`docs/spikes/S2-results.md`;
   spines § 3). *Read this item as a design intention, not as a description.*

The "generating world history…" pause at creation is a genre ritual, not a
cost — but it must be measured (S7) and scale with the extent knob. *(Applies to
the history stage when it returns; the three live stages carry their own measured
costs.)*

## Below the region scale: lazy, unbounded in detail

Nothing is pregenerated. Regions, locales, chunks collapse on approach via
the pyramid rule, conditioned by the levels above. Consequences:

- The **extreme depth** costs nothing until descended into; deep-earth
  societies are lazily determined under the same rule (and sit in the
  statistical sim tier until observed — ARCHITECTURE.md § chunk shape).
- Voxel/detail generation is never bounded by the world's extent decision.

## After year zero — ⏸ ON HOLD with the history layer

Pregenerated history provides priors and committed facts; the live
statistical tier *continues* history from there, collapsing under observation
(S2 machinery). One system, one ledger, no seam at world creation.

*Status 2026-07-28: this describes the intended handoff and no part of it runs.
There is no pregenerated history to hand off and no live statistical tier
consuming one. The no-seam property is the requirement worth preserving — it is
the reason the two clocks were ever meant to share a ledger — and it is the first
thing to re-derive when the layer is designed for real.*

## Borders: unbounded wilds — DECIDED

The *civilized* world is bounded; its fringes are lazily unbounded hostile
wilderness — polar ice wastes, abyssal ocean, and always downward. No walls:
the edge fades into endless mystery, serving the mood (isolation, sublime
vastness) while the bounded interior keeps closure and history. The wilds use
the lazy machinery with no history layer — there is nothing out there to
record, which is the point.

*⏸ The interior/wilds contrast is currently **half-expressed**: with history on
hold, neither region has a history layer, so the distinction that survives today
is closure and pregeneration, not record-keeping. The DECIDED status of unbounded
wilds is unaffected — it was never contingent on history.*

## Extent: flexible, a player knob — DECIDED

World size is a creation-time parameter, not a constant. Working target for
default: few-hundred-km class (DF "large world" territory — enough latitude
span for real climate gradients, many instances of every landform, dozens of
cultures, dense-enough history that players collide with it). Half-Earth is
possible but likely counterproductive: sparse history reads as empty; dense
history reads as ancient. S7 must measure pregen time vs extent so the knob
gets honest labels.

> **⏸ Two of this knob's four sizing criteria are on hold.** *Latitude span for
> climate gradients* and *many instances of every landform* are earth-science
> criteria and are live — they are what actually sizes the world today. *Dozens of
> cultures* and *density of history* are on hold with the layer, **and so is the
> "sparse reads empty / dense reads ancient" trade-off**, which is entirely a
> statement about history density. **Consequence worth stating plainly: the extent
> default is currently being chosen on half its intended criteria**, so expect it
> to be revisited when history lands rather than treating today's value as settled
> against the full argument.

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

## Sequencing — when the history layer may be taken up (DECIDED 2026-07-28, user)

The hold is **not** open-ended, and it is **not** schedulable either. The user's
ruling, in their terms:

> *"We have a lot of engine work and non-bio default-modpack work to do prior to
> considering ecology, then social concepts. All non-bio earth science will have to
> be in the ratified SDK-plugin shape and we will fully respect the plugin-agnostic
> engine shape… the criteria to consider bio/soc is not hard defined yet other
> than, much work and reflection will be done before the **USER** decides it is
> time."*

**The ordering is firm:** engine + non-bio earth science, in the ratified
plugin shape → **ecology** → **social concepts** (of which history is one).
Ecology precedes anything social; nothing social is considered before it.

**The engine shape that must hold first** (user, same ruling — and note it
partially answers the open boundary question in
[`north-star.md`](north-star.md) § *The core/plugin boundary*):
- **Engine owns**: the **primitives** — field kernels and **refinement kernels** —
  and the **runner**.
- **Plugins declare all content**: fields · field passes and cell passes · **pass
  order and rate** · materials · **refinement** · and more.
- **⚠ That list is explicitly NOT exhaustive**, by the user's own statement. Do not
  cite it as a closed enumeration — it is the shape, not the inventory. *(This is
  the one control the corpus is measured as lacking: no enumeration here is checked
  for completeness, so treat an absent item as unlisted rather than as excluded.)*

**Why there is no checkable gate, and why that is deliberate.** Each of those
sections *"has its own ongoing design conversations / acceptance criteria, and flux
in one may ripple to others."* **"Sufficiently complete to begin thinking about
bio/soc" is a USER CALL** — it is not a test, not a checklist, and not something an
agent may declare met by tracing code. *Existence is not standing, and neither is
progress: no amount of shipped earth science entitles anyone to open the social
thread.* Ask; do not infer.
