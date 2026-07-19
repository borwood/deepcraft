# Knowledge — design seed

Status: seeded 2026-07-19 from the "balrog bore" scenario analysis (ratified).
The information architecture's requirements, derived by walking one fictional
find end-to-end. Sister sketches in ideas.md (§ Knowledge is physical, § The
historied player, § Procedural languages).

## The scenario (the load test)

Deep history: a mythic entity descends violently through the earth, melting a
bore past all natural depth, and lies dormant, fed whispers by servants. Play
era: a strip-mining player intersects the bore, earnestly cannot interpret
it; a companion geologist and lore-keeper cannot name the cause either — but
they know precisely *why it is anomalous*, and one half-remembers a song.

## Requirements the scenario forces

1. **Mythic actors + rare-event grammar** in the history sim: singular
   entities with catastrophe-scale event spaces, committing facts like any
   actor (epoch, path, manner).
2. **Events emit worldgen features; the ledger is spatially indexed.** A
   fact carries generative parameters (path, radius, thermal profile) that
   lazy chunk collapse consumes — so collapse must query facts *by volume*,
   not only by subject.
3. **Computable anomaly.** Every natural feature carries process provenance;
   every process has a signature (geometry envelope, material assemblage,
   tectonic context). "What explains this?" is a real query; anomaly = no
   valid provenance match. Freakiness is physics, never a quest flag.
4. **Perception ≠ interpretation.** Finding surfaces only what senses give;
   identification is a separate, knowledge-gated operation. Thoughts respect
   ignorance ("…this stone is wrong somehow"). No free identification, no
   lore toasts.
5. **Typed knowledge**: *episodic* (ledger-derived memories), *schematic*
   (expertise — process-signature libraries, classifiers), *legendary*
   (lossy transforms of old facts). Mythologization is a modeled process —
   compression + distortion over transmission generations — that keeps a
   traceable link to the source fact (so legend↔evidence matches are
   detectable and later verifiable).
6. **Structured expertise output.** A consultation runs the explanation
   query against the expert's own library and returns structure (partial
   matches, violated constraints); dialogue — scripted or LLM — is generated
   *from* that structure. Grounded, never canned.
7. **Associative retrieval by content descriptors** (deep+fire+old+mountain
   → the Song), shared machinery with dreams.
8. **Bidirectional propagation with stakes.** Player-generated facts enter
   the same whisper networks NPCs use; distant dormant actors condition on
   propagated facts (fame machinery = spy network, opposite directions).
   Deep-time checkpointing (S2 condition #1) is load-bearing for
   centuries-dormant actors.

Already covered elsewhere: diegetic observation rules and observe/inspect
(API.md), claims-above-ledger (S2), statistical dormancy + collapse-on-
approach (S2/S7).

## The theses

- **Quests are knowledge gradients.** nothing → anomaly → legend-fragment →
  truth is an investigation no one authored; the gradient of who-knows-what
  *is* the content.
- **Information is never conjured for the player's convenience.** The world
  computes what is knowable, by whom, through which channel; the game
  surfaces exactly that and no more.
- (With ideas.md § Procedural languages:) **language is the wire format** —
  utterances serialize fact-fragments; distortion, rumor, and myth are
  mechanical re-encodings; comprehension is a rendering setting.
