# 0121 — The heir that was never owed

*2026-07-28. Removing the bootstrap history content: `pregen/history.rs`, four
`Pregen` fields, `collapse.rs::ruin_posts`, five draw domains, and the pass's
membership in `vanilla_passes()`. The world lost 102 voxels and not one golden
moved.*

> blogworthy: **lens 1 (AI-native development)** — a stub entry, an heir line,
> and a spines § 3 row are all *records*, and all three were read as
> *obligations*. The corpus was working exactly as designed and every instrument
> in it was arguing for keeping content nobody had ever voted for.
> Also **lens 3 (reflexions in a deepsim codebase)** for the second half: what it
> looks like when the last consumer of a whole ratified subsystem walks out.

---

## The thing being removed, and why it is a removal

For a week the pregen pipeline had four stages, and the fourth was a
civilization. `pregen/history.rs` seeded peoples at habitable sites, ran twelve
epochs of expansion and conflict, decided whether a settlement was sacked by
collapsing a hostile-pressure question through dc-sim's S2 statistical engine,
and committed the outcome as immutable facts in a constraint ledger. A sacked
site's `abandoned` flag survived into `Pregen.sites`, `collapse.rs` turned that
into ten wood posts scattered at hashed angle and radius, and `generate_chunk`
stood them up out of the ground where a player could walk into them.

It was, mechanically, rather good. It was also fabricated during bring-up by
nobody's design. On 2026-07-26 the user said so:

> *"They are unratified zealous fabrications from the early bootstrapping of the
> project and I DO NOT care about them, they WILL be wholesale replaced, they
> should just be removed. We do NOT have any form of evo/socia/civ modeling even
> at the design stage: they are NOTHING."*

The interesting part of this slice is not the deletion. It is **how hard the
corpus had been arguing against it**, and how every one of those arguments was
formally correct.

## Three instruments, all working, all wrong about standing

**`stubs.md` § 1 was a model entry.** The ruin posts were listed, loud, with a
code marker, honest that the posts were a rule-of-thumb rather than a mechanism,
and with a named heir: *the social sim + ecology — dwarf-fortress-class
civilization history; "that's the only way a post gets there."* Every property we
ask of a stub entry, present. And the entry's whole grammar — *what this fakes ·
the expresser that subsumes it · blast radius* — **presumes the thing should be
expressed**. "What expresses this better?" cannot return "nothing; delete it".
The inventory asks the design question and never the standing question, so a
well-formed stub entry is a quiet, permanent argument for its own subject.

**`spines.md` § 3 was two days old and pointing the right way.** A rider had
listed the S7 handoff row on 2026-07-26 and measured it precisely:
`Pregen.{ledger, overlay, n_polities, observe_count}` had *exactly one non-test
reader between them* — `approx_resident_bytes`, which measures how much memory
the ledger costs. The header of § 3 says *"leaving this list is a good event:
record what consumed it and when."* Consumption is the only exit the index knew
about. Read that way, a zero-consumer row is a **backlog item**: find it a
consumer. The row was in fact evidence for the opposite verdict, and it took the
user to read it that way.

**And the ROADMAP had it sequenced as an appearance slice.** Because the posts
render, moving the draw that decides sackings would change what a player sees, so
draw-domain part (a) had been carved out as *user-owned*: needs a walk, needs
screenshots, needs ratification. Two days earlier the integrator had gone one
step further and proposed **counting the ruins** — which, as CLAUDE.md now says,
already concedes that some number would matter.

Three independent controls, none stale, none wrong on its own terms, all
converging on *preserve and schedule*. The rule that broke the tie had to be
imported from outside all three: **if this did not exist, would we build it today,
in this shape?**

## What the traced consumer graph got right, and what it missed

The ROADMAP entry said the graph was already traced, and the brief for this slice
said to verify that before building on it. Re-derived by grep, field by field:

- **Right, and load-bearing.** `Pregen.ledger` had exactly one non-test reader,
  and it was `approx_resident_bytes`. `Pregen.sites` escaped only through
  `abandoned`, into the posts. The history pass really was an unconditional
  `vanilla_passes()` member with no flag and no knob.
- **Right in a way worth restating.** *Three of the four fields had no reader at
  all* — not even the measurer. `approx_resident_bytes` touched `ledger` and
  `sites`; `overlay`, `n_polities` and `observe_count` were written and never
  read by anything outside tests. "Exactly one non-test reader **between them**"
  is true and is a weaker statement than it sounds.
- **Missed: a third test file.** The trace named `s7_pregen.rs` and
  `s7_handoff.rs`. `s7_measurements.rs` also read all four fields, plus
  `approx_resident_bytes`, and printed them as four columns of the world-size
  table. Small, but the trace was quoted as exhaustive.
- **Missed, and it is the biggest thing here.** The scope line said "dc-sim's
  region / agent-step draws". What it did not say is that `pregen/history.rs` was
  the **only production caller of the entire S2 statistical tier**. Deleting it
  leaves `engine::{query, observe, force_fact}`, `Ledger` and `ToyWorld` with
  zero production consumers workspace-wide — a whole ratified subsystem, tested
  by its own suites and reached by nothing. That is not a side effect worth a
  footnote; it is the largest § 3 row in the file, and this slice created it. It
  is recorded rather than acted on: the removal was scoped to the *content*, and
  what to do with the S2 toy implementation is a separate decision.

What dc-worldgen still uses from dc-sim is `statistical::rng` — the draw provider
— which is on every worldgen path and unaffected. The confusion is easy to have
and expensive: the module named `statistical` contains both the subsystem nobody
calls and the one everybody calls.

## The carve-out that expired by having its subject deleted

`engine.rs` carried this, on the region-pressure step draw:

> *"routing it through `Draws::of` … would change the key and re-roll every
> world's history layer: polities, sites, ruins."*

That is A-2, a justification outliving its premise, in an unusually pure form:
the premise did not become *false*, it became **absent**. There is no history
layer to re-roll. Both step draws can now move onto the provider for free — which
discharges draw-domain part (a), and discharges it by removing the thing it was
protecting rather than by converting anything. That conversion is not done here:
with nothing user-visible downstream it stopped being a slice and became
housekeeping for whoever next holds that file.

## The prediction that was wrong, and why the shape of it matters

Both the ROADMAP entry and corrections #64 said the goldens would move, and #64
named them: `contents_contract`, `s7_walk`, `geology`, all of which *"hash
`generate_chunk` blocks and are structurally downstream of the posts"*, plus
`s7_handoff.rs:118` as "the test most likely to break".

**Nothing moved.** Every byte-identity golden in the workspace passed, by name,
against a tree that had just deleted 102 wood voxels from the shipped world.

The mechanism is arithmetic, not architecture. On production-Medium the pass
abandoned 4 of 13 sites; their posts landed in exactly **twelve chunks**, the
nearest of them 727 chunks — about 21 km — off the `cz = 0` line. The samplers
walk fixed, origin-hugging sets: `contents_contract` spans `cz ∈ [−60, 42]`,
`geology` spans `cz ∈ [−20, 24]`, `providers_golden` fingerprints the `DeepField`
and never a chunk at all, and `s7_walk`'s sample hashes are compared against a
*re-generation of the same world in the same run* — they are a determinism check
and could not move for any reason whatsoever. Measured on the pre-removal tree
before anything was touched: **zero wood in the `contents_contract` sample set.**

"Structurally downstream" was true of all of them and told us nothing. Whether a
fingerprint moves is a question about **which chunks the sampler visits**, and it
is answerable in two lines of arithmetic against the sampler's own `for` loop.
The uncomfortable part: corrections #64 is *the entry that names this reflex* —
"a justification assembled by symmetry", found only by tracing what a shipped
world executes — and its closing paragraph commits it again, six lines later, in
the same voice.

There is a second, sharper reason to file this as a correction (#66) rather than
shrug at a happy outcome. **"The goldens will move and that is correct"
pre-authorises a hash change**, and a byte-identity golden exists precisely to
make hash changes expensive. Had a golden moved here for an unrelated reason — a
sibling's stale artifact, a merge folding two changes — the prediction would have
been sitting ready to absorb the movement as expected. A pre-authorised golden
move is indistinguishable from an unexplained one. The honest form is a
prediction *with* a mechanism, which is falsifiable in advance; this one would
have been falsified in advance, for free.

## Accept by outcome, because green is not evidence of absence

A passing gate proves nothing about whether content is gone — every deleted
reference passes. So the "before" was measured **first**, on the untouched tree:
generate the 3×3 chunk neighbourhood at two y slabs around each of the four
abandoned sites and count `Block::Wood`. Result: **102 voxels across 12 chunks**,
with their exact `ChunkPos`es printed. After the removal, those same twelve
chunks regenerate to **0**, and a wider 5×5×2 box around each of the four former
clusters — 200 chunks — is also **0**. The posts did not move; they are absent.

The residency figure is the same story from the other end. The two terms that
left `approx_resident_bytes` were the fact ledger and the site summaries:
**4,896 + 624 = 5,520 bytes of 377,364,589** on production-Medium, 0.0015 %. The
world's whole recorded settlement history — 153 committed facts, 13 sites, 2
polities, 90 statistical collapses — cost about five kilobytes and was read by
the function that measured it.

## What is left, deliberately

- `Block::Wood` stays. Its only worldgen emitter is gone, and its doc comment now
  says so; it survives on the edit palette and the dc-api host-generator path.
  Removing the variant would renumber block ordinals and move every golden in the
  workspace for reasons that have nothing to do with this removal — which would
  destroy exactly the attribution this slice was able to make.
- The five retired draw salts (`SitePos`, `Overlay`, `Expand`, `Sack`, `Ruin`,
  `0x5700_0005`…`0x5700_0009`) leave a **deliberate hole** in the domain list,
  with a comment saying so. A salt is baked into every world ever generated from
  it; re-issuing one of these to a new decision would silently make two unrelated
  decisions share a stream across every saved world and every old journal
  capture. Take the next unused value; never fill a hole.
- dc-sim's `Subject::{Site, Polity}` / `Aspect::{SiteExists, SitePolity,
  SiteEvent, PolityExtent}` / `SiteEventKind` / `Value::{Exists, PolityRef,
  Event, Extent}` vocabulary stays, with no producer, marked in-code as *not a
  schema to build on*. Deleting variants of a `Serialize` enum is wider than this
  removal's scope. It is flagged as a candidate for the same disposal.

## The rule this slice adds to the inventory's own doctrine

`stubs.md` § 1 has been rewritten to say it, because it is where the next person
will look:

> A stub entry is not neutral about its subject's standing — **it asserts it.**
> An heir is a promise that the thing stood in for is wanted. Before writing one,
> ask the standing question, not only the design question.

And § 3 of `spines.md` now records that a row can leave the index **two** ways.
It had only ever recorded the first. Consumption is the good exit; *deletion* is
the other one, and for machinery nobody voted for it is the better one. Reading a
zero-consumer row as "find this a consumer" is the mirror image of A-1: instead
of a stand-in becoming the definition, an **artifact becomes a requirement**
because somebody wrote it down.
