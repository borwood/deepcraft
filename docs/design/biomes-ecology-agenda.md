# Biomes & ecology — the conversation agenda (NOT decisions)

Status: assembled 2026-07-19 by Claude as the briefing for two design
conversations that have never happened. Nothing here is decided beyond the
one-line ratified seeds, each cited to where it lives. The rest is agenda:
what those seeds imply, and the questions only the user can answer.

## Where the seeds actually live (all of it)

- **ROADMAP Sequenced 4**: "Biomes-as-diagnosis design (consumers of
  climate/substrate/disturbance axes; registry-defined)."
- **ROADMAP Sequenced 5**: "Ecology design (succession as
  derived-from-disturbance state; populations as statistical-tier
  distributions)."
- **geology.md § backbone** (ratified 2026-07-18): "biomes-as-diagnosis
  consume the same axes; ecology niches are classes" — biomes and ecology
  are further instances of the roles-as-contracts pattern.
- **materials.md ~line 61**: population/disturbance state lives at the
  statistical tier (S2 ledger), "never simulated per-voxel."
- **S7-results.md ~lines 215–218**: the settlement overlay wants a biome
  transition model; biome-ish boundaries sampled per chunk-column are
  blocky (the same quantization family as corrections #6).
- That is *everything*. There is no design doc; these two conversations
  are the missing artifact.

## What "biomes-as-diagnosis" means (unpacking the seed)

A biome is **not a generated thing** — it is a *reading* of ground truth
that already exists: given the axes (climate from pregen/deep-time,
substrate from the strata record, disturbance from the ledger), a biome
def is a registry-defined **classifier**: "call it steppe where precip in
[a,b] ∧ substrate is loose-fine ∧ disturbance is low." Consequences worth
noticing before the conversation:
- Nothing new is simulated; biomes cost nothing at generation and can be
  added/redefined by packs without touching worlds (they re-diagnose).
- Two biome packs can disagree about the same terrain — legal, like two
  field guides. The *default* pack is just the first diagnosis.
- The S9 deep-time work upgraded the available axes enormously (real
  paleoclimate, real substrate history) — a diagnosis can now reference
  *history*, not just present state ("post-glacial outwash plain").

## What the ecology seed implies

- **Succession as derived-from-disturbance state**: a place's vegetation
  stage is a function of time-since-last-disturbance (fire, flood,
  clearing — ledger facts) and substrate, not a hand-placed decoration.
  The S2 statistical tier already stores exactly this kind of
  "distribution + last-event" state.
- **Populations as statistical-tier distributions**: no per-creature
  simulation at world scale; a region carries population distributions
  that collapse to individuals on observation (the S2 observe/collapse
  machinery — the same physics of attention the whole sim runs on).
- **Niches are classes**: species register into niche classes with
  fitness over the same context axes (the geology selection machinery,
  fifth instance of the pattern).

## The questions only the user can answer (the actual agenda)

1. **What is a biome *for*, in this game?** Diagnosis feeding rendering
   moods and spawn tables (light touch)? Or a first-class gameplay
   surface (biome-specific mechanics, discoveries, map legibility)?
2. **Player-facing legibility**: do players see named biomes (UI, map),
   or only infer them diegetically (the knowledge system's "this looks
   like steppe" thoughts)? The knowledge/prospecting doctrine suggests
   diegetic — but naming is also how players share a world.
3. **Ecology's simulation depth v1**: statistical distributions +
   succession stages only (flora as terrain-feature output)? Or does v1
   already want fauna presence (which drags in the NPC-intelligence and
   body-plan tracks)?
4. **Disturbance vocabulary**: which events count (fire, flood, logging,
   mining, battle?) and which are v1 — this defines what the ledger must
   record going forward, so it has schema consequences *now*.
5. **Transition rendering**: biome boundaries as gradients (dither/blend,
   like the 3d member contacts) or as readable ecotones with their own
   character? (Reality-first says ecotones are real communities, not
   blends — the doctrine may answer this one itself.)

## Suggested shape (Claude's proposal, for reaction only)

One conversation, two halves, same pattern both times: ratify the
diagnosis/derivation principles → pick the v1 axis set and vocabulary →
sequence a design doc + implementation milestone per half. The machinery
cost is low (both ride existing registries and the statistical tier); the
decisions are almost entirely about game feel and scope, which is why
they waited for you.
