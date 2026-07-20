# 0035 — The console had the data; it was hiding it

Console v1 (journal/0032) was, on paper, the thing we wanted: a dev console
that is a pure projection of `dc_api::schema::registry()` through four
schema-walking functions, no per-command code. Every parameter's name, type,
required-ness, and description was *right there* in the schema the console
already walked. Then the user actually drove it, and the field report was one
sentence: **"still unusable unfortunately because i do not know the shape of
args, expected inputs, no help per arg etc. also cannot scroll the command
pane."**

The gap was not data — it was **presentation**. v1 had all the data and showed
almost none of it. Help rendered only when you typed `help <cmd>` — but to type
`help world_set_block` you first had to *guess* that `world_set_block` exists
and that `help` is the way to interrogate it. Completion stopped at parameter
keys, so `block=` sat there with a blinking caret and no clue that `dc:stone`
was the kind of thing it wanted. And a 24-line pane with no scroll meant the
one time the console *did* tell you something long (the `help` overview lists
the whole surface), it scrolled off the top the moment it arrived.

This is a specific, recurring failure mode worth naming: **a reflective system
that reflects into a void.** The registry knew everything; the console
faithfully carried it; and none of it reached the eyes of a first-time user at
the moment they needed it. "Generated from the registry" bought correctness and
zero drift, but a projection is only as useful as the surface it projects
*onto*, and v1's surface was a bare prompt.

> blogworthy: "reflection into a void" — the same schema that makes the console
> driftless also makes it mute unless you deliberately render the reflection
> where the user is looking. Correct-by-construction is not the same as
> legible-by-construction.

## What v2 adds — still all generated, none per-command

The invariant is untouched: no function in `console/core.rs` names a single
command. Everything below is another schema walk or another read of the
`Completer` hook. The four fixes:

**1. A live hint area (the core complaint).** The moment a command name is
recognized, a persistent line under the scrollback shows the whole signature,
generated from `payload_schema()`:

```
world_set_block  pos.x=<int> pos.y=<int> pos.z=<int> block=<string>
```

Required params are bare; optional ones are bracketed (`[kind=<string>]`), so
the shape of a valid command is visible before you type a single argument. When
the caret then sits in a `key=` token, a second hint line describes *that*
parameter — its type, whether it is required, its schema `description`, and (if
the command's `Completer` offers values for it) a few live examples:

```
block : string (required) — block name, e.g. dc:stone   e.g. dc:air, dc:stone, dc:dirt, dc:grass
```

**2. `help <command>` is a real reference card.** One line per argument with
type / required-marker / description, plus a **generated example invocation**
built from schema-appropriate placeholders — ints→`0`, strings→the `e.g.` hint
parsed out of their own description, nested params kept dotted:

```
example: world_set_block  pos.x=0 pos.y=0 pos.z=0 block=dc:stone
```

Because the placeholder logic is schema-driven, a command added tomorrow gets
its example for free.

**3. The `Completer` hook is wired (API decision #6, filed unwired at 0033's
merge).** In value position (`block=<TAB>`), the console now asks the command's
registry completer for legal values: block names, live character names,
postures, event kinds. `Static` completers (a fixed vocabulary) answer from the
schema alone. `World` completers need the live world — and here is the one
genuinely interesting wiring problem.

## Passing the world to completion without letting completion touch the world

A `World` completer's signature is `fn(&HostWorld, param, prefix) -> Vec<String>`.
The console system runs at the front of the frame's system chain, specifically
so it can swallow input *before* the gameplay systems — and several of those
downstream systems (`drain_bridge`, `tick_authority`, `remesh_dirty`) take
`ResMut<Authority>`, which owns the `HostWorld`. The console must read that same
world for completion.

The resolution is that completion is *provably* read-only, at two levels:

- **Type level.** The completer takes `&HostWorld`, and the three world-backed
  sources call only `&self` accessors — `block_names()`, `characters()`,
  `content_classes()`. None of them can `tick()`, `submit()`, or even
  `block_at()` (which needs `&mut self` for lazy chunk generation). The borrow
  the console hands over is shared; the sim cannot advance or mutate through it.
- **Scheduler level.** The console takes `Option<Res<Authority>>` — a shared
  resource borrow. Bevy's scheduler serialises it against every
  `ResMut<Authority>` system, so there is no runtime contention to block on; the
  access is resolved when the schedule is built, never mid-frame. The `Option`
  is the graceful-degrade seam: if the world is somehow absent, `World`
  completers yield nothing rather than stalling — the frame never waits on
  completion. (`Static` completers keep working with no world at all, so
  posture and event-kind completion survive that case.)

So `block=<TAB>` cycles real block names and `character=<TAB>` lists live
characters, and the completion path cannot advance a tick or write a voxel even
by accident.

**4. Scrollback scrolling.** The retained history rose from ~24 to 500 lines and
the render is now a window into it: PageUp/PageDown and the mouse wheel move a
`scroll` offset, a fresh submission or a completion listing snaps back to the
latest, and a `-- N line(s) below --` marker shows when you are scrolled up.

## The held-key close quirk, fixed in passing

v1's input swallow calls `ButtonInput::reset_all()` every frame the console is
open, so downstream gameplay sees empty input. But Bevy re-emits no event for a
key that is *physically still held* — its press event fired once, before the
console opened. So opening the console while walking, then closing it, left the
movement key stuck "released" until you let go and pressed it again: `reset_all`
had cleared the `pressed` bit and nothing re-set it.

The fix tracks the physically-held key set from the raw event stream (which
survives `reset_all`, unlike the `ButtonInput` view), seeded from
`get_pressed()` at open. On the closing frame, after the usual `reset_all`
swallow, it re-`press()`es each still-held key and clears its `just_pressed`
edge — so walking resumes seamlessly without re-firing a one-shot action like
jump or an edit.

## What it looks like in game

Press **T**. Type `world_set_block` and its full signature appears under the
scrollback before you touch an argument. Type ` block=` and the hint line
describes `block` and shows real block names; press Tab and the value completes.
`help world_set_block` prints a card ending in a copy-pasteable example line.
PageUp scrolls back through a long `help` dump instead of losing its top. Close
the console mid-stride and you keep walking.

## Rides-as-built vs. ratification

The hint-area layout, its colours, the bracket-for-optional convention, and the
`key : type (required) — desc` phrasing are dev-tool **appearance** — they ride
as-built and are the user's to ratify or restyle. The mechanisms (schema-driven
signature/example, the read-only completer wiring, the scroll window, the
held-key restore) are load-bearing and are what this entry records.
