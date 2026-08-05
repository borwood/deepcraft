# 0159 — the turn that was in the wrong layer

*2026-08-05 · bodies thread · slice one of DIRECTION IS AN AXIS, NOT A RATE*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — the deleted constant was
> not wrong about its number; it was wrong about *which layer it lived in*, and the
> tell was that nobody could say what would have made 0.22 s right. Also **lens 1
> (AI-native development)**: the slice's most valuable output was a user sentence
> spoken mid-edit, and the process question is whether the code can catch it in time.

## The constant

`TRUNK_TURN_WINDOW_S: f64 = 0.22` sat in `dc-client/src/body.rs`, one line, with a
doc comment describing exactly what it did: how quickly a body's trunk yaw chases the
direction it is travelling. It had been there since the facing split, it worked, and
in the game it read as bodies rounding their corners instead of pivoting on a dime.

It went today, and the ruling that killed it was the user's, made yesterday:
*"snap-turning is good enough for minecraft. i don't care that it's ugly just like i
don't care that we don't have grass yet: **the point is honesty and not entrenching
and enshrining and obscuring a thousand little proxies**."*

That sentence is the whole entry, really. The rest is what it cost to act on and what
turned up on the way.

## Three arguments, and only the first is the user's

The ROADMAP entry that carried this slice named three reasons, in a deliberate order.

**One: a world-global turn rate forecloses content.** This is the argument that
matters, and it is the one the user reached for. A golem *should* pivot instantly. A
loaded cart *should* need to come about. With 0.22 s compiled into the renderer,
neither is authorable — every body in every pack turns at the same rate, and a pack
that wants otherwise has no verb to say so. The engine had taken a position on a
question it has no business holding a position on. `dependency-graph.md` § 0a states
this as the surviving constraint after the additivity bar was withdrawn: *a primitive
must not foreclose content.*

**Two: it is an opinion sitting in the engine** — the mirror of § 0b's
opinion-vs-absence test. Two well-made packs would answer "how fast does a body come
about" differently, which makes it an opinion, which makes it the pack's.

**Three, and this is the one I did not expect to find: it was in the wrong layer
entirely.** `bodies.md`'s three-layer split already puts volition in the
**controller**. Turning is volition. It was implemented in the **renderer**. Nobody
decided that; it happened because the trunk yaw was decoration when it was written,
and decoration lives client-side.

## The measurement that made the smoothing dishonest

Here is the part that reframed the slice for me.

I went looking for what the 0.22 s was smoothing, expecting to find a turn rate in
the sim that the client was easing toward. There isn't one. `dc-api/src/character.rs`
assigns horizontal velocity **straight from intent, with no inertia** — the comment
says so in as many words, *"no inertia, like the player"* — and `facing_yaw` is a
**bare assignment** from the travel direction a few lines later.

**The movement already snapped. Only the render was smoothed.**

So the 0.22 s was not modelling a body's rotational inertia. It was *hiding* the fact
that the body had none. And the lag the user noticed at running speed — *"you can
imagine it's still turning even though the body… has covered some ground"* — was the
render telling the truth about the sim, which is exactly backwards from how a
smoothing constant is supposed to fail. A proxy that makes the world look better while
making it read *less* honestly is the thing the user's sentence was aimed at.

## Where the seam does NOT go

The obvious move, once you delete a constant, is to leave a seam where it was: keep
`steer()` as an identity passthrough, doc-comment it with its heir, done. Every
instinct the corpus has trained says *write the seam, not the value*.

That would have been wrong, and the reason is worth writing down because I nearly did
it.

The heir to a turn rate is **not** a better turn rate. The arc's continuation slot
already said where the replacement lives: on the **movement**, not on the trunk yaw —
*an animal cannot reverse its velocity instantly, and that is a fact about the body*.
A seam at `steer()` would have been planted at the site of the old constant rather
than the site of the new mechanism, which is the S18 failure the corpus already names:
*place the seam where the full thing is supposed to live.*

So `steer()`, the `AnimState.trunk_yaw` field, and the `AnimState::facing()`
constructor all went with the constant, and the marker went into
`step_character` — the line where intent becomes velocity. That line is now annotated
as a stand-in (`stubs.md` #55) with the layer that will own it named above it.

## What the user said while I was editing

Mid-slice, two messages arrived that were worth more than the delete.

The first defined the heir, and I have quoted it verbatim into the stub because
paraphrasing it would lose the both-ends structure:

> *"whatever we do we are not foreclosing the ability for a pack to define bodies that
> simply snap to the pointed direction, pivot immediately, etc: nor foreclosing the
> ability for a pack to have bodies that preserve momentum, inertia, most slow and
> turn, etc. This implies a layer of pack-owned logic which may be opinionated or
> bare, etc, and which movement intent passes through."*

Note what that does to today's behaviour: **bare passthrough is not the degenerate
case to be replaced, it is one of the two ends that must stay expressible.** The delete
did not create a hole to be filled; it revealed that the engine's current behaviour is
a legitimate point in a space the pack gets to choose from.

The second was shorter and lands much harder:

> *"let's also be clear about the player: the player will control a body, there will be
> no separate player concept except that the body is driven by user instead of script
> or mcp etc. thus the current player has an heir."*

The character primitive's own module doc cites `dc-client/src/player.rs` as *"the
reference for these dynamics."* The reference is a stand-in. Filed as `stubs.md` #56
in the same commit, per *defer = write it now* — and it points straight at
corrections #100, where the observer's stop channel and the driver's are different
mechanisms **because** the player and the character are different things. Collapse
those and that gap closes too.

## What the gate saw

`fmt` clean, `clippy --workspace --all-targets --release -D warnings` exit 0,
`dc-api` + `dc-client` tests **328 passed / 0 failed**, exit 0. Both crates verified
`Compiling` from main's own path. Verified by name, not by `ok`: the new test
`the_client_applies_the_sims_facing_unmodified` present and passing, the deleted
`the_trunk_chases_the_sims_target_and_holds_when_it_stops_moving` absent.

One test was replaced rather than removed, and the replacement is a stronger claim
than its predecessor. The old one asserted **convergence** — the trunk approaches the
sim's target and never drifts off — because an asymptotic chase never lands exactly.
The new one asserts **exact equality** at `resolve_orientation`, the seam the renderer
now feeds `facing_yaw` into. If a turn rate ever creeps back into the client, it shows
up there as a residual.

Workspace gate deferred to the arc's pause — batch debt recorded in the commit
message.

## What it looks like

Bodies snap-turn. It is uglier and it is honest, and the user blessed it in advance
under the announce-then-go protocol, which is what made this slice a delete rather
than a conversation.

**Also worth stating plainly, because a summary that names one half gets read as the
whole: the render is now *more* jittery-looking and the sim is unchanged.** This slice
removed a proxy; it did not add a mechanism. The body still has no rotational inertia,
still reverses direction in one tick, and still costs nothing to turn. Everything that
was dishonest about the movement is still dishonest — it is just no longer wearing a
smoothing curve that made it look considered.
