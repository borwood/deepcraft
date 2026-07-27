# 0118 — Three lines, a test that was guarding nothing, and a blast radius assembled by symmetry

*2026-07-26. Background agent, isolated worktree. The brief was explicitly
housekeeping — "byte-identical housekeeping, three lines" — with one read-only
rider attached at the end, because the user had looked at the ROADMAP entry
justifying the sibling task and found the claim odd.*

> blogworthy — lens 1 (AI-native development) and lens 3 (reflexions in a deepsim
> codebase): **the rider was worth more than the task**, and the task's own
> surprise was that finishing a conversion *dissolved the premise of the test
> that had been guarding it*. Two different shapes of "a thing that was true when
> it was written and quietly stopped being the point."

## The three lines

journal/0105 built the randomness provider and converted 26 hand-rolled salts,
then named four holes. Hole 2 was the smallest: `deeptime/`'s three domains were
*registered* in `draws.rs` — which is what makes the compile-time uniqueness check
total — but their call sites still spelled the number locally. The entry left it
as "a three-line follow-on, left to whoever holds those files."

It really was three lines:

```rust
// grid.rs
- let jitter = (draw_f64(&[cfg.seed, SALT_DT_ROUGH, gx as u64, gy as u64]) * 2.0 - 1.0)
+ let jitter = (Draws::of::<DeepTimeRoughness>(cfg.seed).unit(&[gx as u64, gy as u64]) * 2.0 - 1.0)

// biotic.rs
- let roll = draw_f64(&[seed, SALT_BIO_FIRE,  gx as u64, gy as u64, u64::from(epoch)]);
+ let roll = Draws::of::<BioticFire>(seed).unit(&[gx as u64, gy as u64, u64::from(epoch)]);
- let roll = draw_f64(&[seed, SALT_BIO_FLOOD, gx as u64, gy as u64, u64::from(epoch)]);
+ let roll = Draws::of::<BioticFlood>(seed).unit(&[gx as u64, gy as u64, u64::from(epoch)]);
```

Byte-identical by construction, not by luck: `Draws::bits` folds
`(seed, salt, addr…)` through the same splitmix chain with the same pi-digit IV
that `mix(&[seed, SALT, addr…])` did, and `unit` is `draw_f64`'s
`(bits >> 11) · 2⁻⁵³`. Same numbers, same order, same arithmetic. Every golden
held, unmoved.

## The surprise: completing the conversion made a test vacuous

The gate failed anyway, and on something nobody had predicted:

```
error: constant `SALT_BIO_FIRE` is never used
error: constant `SALT_BIO_FLOOD` is never used
```

Which is obvious in hindsight and was not obvious in foresight. journal/0105 had
paired the registration with an agreement test — deliberately, and citing the
right doctrine:

> *"the 'a summary is not an authority' pattern: the list is the authority, the
> local `const` is the copy, and a test asserts they agree."*

That was exactly right **while a call site still read the copy**. The test's
value came entirely from the copy having a *second* reader: production code. Take
that away and the test compares a constant to itself under another name, and
keeps the constant alive purely so it can be compared. It is not a weakened test;
it is a test whose subject no longer exists.

So the constants and their two assertions were deleted rather than kept alive
under an `#[allow(dead_code)]`. The domains in `draws.rs` are now the only
spelling of `0x5B00_0001` and `0x5B00_0002` — **a copy that does not exist cannot
drift**, which is strictly stronger than any test that checks whether it did.

The generalisation is worth keeping: **an agreement test is scaffolding, and its
retirement is the success condition, not a regression.** When the "summary is not
an authority" pattern is applied to a *duplicate spelling* (as opposed to a
genuinely cheaper derived answer), the goal is a world where the duplicate is
gone and the test with it. Watching one dissolve is the pattern working.

Notably, **the assertion did not catch this — the dead-code lint did.** The test
went on passing, correctly and pointlessly, and would have gone on passing
forever. That is a mild echo of journal/0105's own moral about claims a gate
cannot see.

`SALT_DT_ROUGH` survives, and honestly. journal/0105 counted three call sites;
there are four. `deeptime/refine.rs:155` has a **second** roughness-jitter site
reading the same constant — a file a sibling agent held during this slice, so it
was left alone rather than converted from outside. The constant therefore still
has a real reader and keeps a real assertion. Converting that one line is what
lets both retire. It is now in the ROADMAP rather than in nobody's head.

## The rider, which was the valuable half

The ROADMAP's reason for keeping part (a) — the two `dc-sim/engine.rs` draws — as
a separate, user-owned slice:

> *converting them changes the key and **re-rolls every world's history layer**:
> polities, sites, ruins. That is a real appearance change.*

The user's objection was that there **are** no polities, sites or ruins in the
world, so there would be no appearance change. The instruction was to
investigate read-only and report, not to act.

Both halves of that exchange turned out to be partly wrong, which is the most
interesting outcome available and the reason this is written down.

**The claim's load-bearing half is true, and there are ruins.** The chain is
short and has no flag on it anywhere:

```
engine.rs:331          region-step draw → collapsed pressure
history.rs:221         Pressure(2) opens the sack roll
history.rs:237         site_state[s].abandoned = Some(epoch)
history.rs:280-293     → SiteSummary { …, abandoned }
pregen/mod.rs:302      → Pregen.sites
collapse.rs:1588       ruin_posts: `if !s.abandoned { continue; }`
collapse.rs:483-488    chunk.set(…, Block::Wood)      ← inside generate_chunk
meshing.rs:214         Block::Wood → [0.44, 0.33, 0.17, 1.0]
```

The history pass is an unconditional member of `vanilla_passes()`
(`pipeline.rs:359-366`) — no CLI flag, no config knob, nothing behind a feature.
So there are wooden posts standing up out of the ground around abandoned sites in
every world the client builds, and re-addressing that draw relocates them. This is
the *opposite* of the pattern we have hit twice this week (corrections #51's zero
coal, journal/0105's own zero pore riders): those were claims about a world behind
a flag. **This one ships.**

**But the word "two" is false, and it halves the slice.** The agent-step draw
(`engine.rs:355`) re-rolls nothing at all. The pregen overlay is constructed with
an empty agent roster — `history.rs:82` and `:84-88` both pass `vec![]` as
`ToyWorld::with_graph`'s third parameter, `agent_home` — so `num_agents()` is
zero and the loop at `engine.rs:342-363` never executes outside dc-sim's own test
suite. Converting that draw is byte-identical for every world.

**And "polities" is false.** The polity count is fixed at epoch 0 from slot count
(`history.rs:134-149`); no draw ever founds, merges or destroys one. `PolityExtent`
facts do move, but they are written into `Pregen.ledger` — and nothing in
production reads the ledger. `Pregen.ledger`, `.overlay`, `.n_polities` and
`.observe_count` have exactly one non-test reader between them, and it is
`approx_resident_bytes`. A whole clause of the blast radius is a spines § 3
"built, and nothing calls it" cluster wearing an appearance claim's clothes.

### How the sentence got written

Not by anyone lying, and not by a justification outliving its constraint — the
usual A-2 shape. This one was **assembled by symmetry**. Two draws sat side by
side in one loop, addressed identically as `[seed, k, SALT, …]`, feeding a
subsystem called "history"; the sentence generalised over both and then reached
for the three nouns that subsystem's vocabulary offers. Nobody traced what a
shipped world executes.

The give-away had been sitting in a doc comment the whole time. `world.rs:132`:

> *"Agents are optional — an empty `agent_home` gives a pressure-field-only
> world."*

The parameter is documented, the caller passes `vec![]` twice on adjacent lines,
and the conclusion still went the other way. That is the same failure as
journal/0105's pore-rider comment — a claim about code written from the code's
*shape* rather than its *behaviour* — and it is worth naming as a distinct
species next to A-2, because the remedy is different. An expired justification is
found by re-checking its premise. A justification assembled by symmetry has no
premise to re-check; it is found only by tracing.

**The re-scoping is not this slice's to make.** Filed as corrections #64 and
recorded against the ROADMAP entry, with the goldens that would actually move
named (`contents_contract`, `s7_walk`, `geology` block hashes — and
`s7_handoff.rs:118`, which pins a *seed-specific* sack and is the likeliest
break). `GOLDEN_SURFACE` / `GOLDEN_RECORD` are not downstream at all; they hash
the deep-time field.

## What it cost

One background trace, no world builds, no game time. The task it rode along with
was three lines. That ratio is the argument for attaching riders to housekeeping:
the agent was already in the neighbourhood, and the question that turned out to
matter was one the user asked in passing because a sentence read oddly.
