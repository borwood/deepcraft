# 0107 — The revision you read, and the writer that supersedes it

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** and **lens 1
> (AI-native development)**. A self-declaring pass system where the declaration
> was *almost* enough — and the shape of the "almost". Plus a spine-audit claim
> that came with the instruction *"verify that claim before trusting it"*, which
> turned out to be exactly the right instruction: half of it was wrong.

## The under-declaration

`dc:field/head` — the groundwater potential field, on by default, the pass that
made artesian columns expressible (journal/0098) — declared this:

```rust
id: "dc:deep/head",
reads: &[Routed],
writes: &[Head],
reads_prev: &[Recorded],
```

And its body did this:

```rust
let ground: Vec<f64> = (0..n).map(|i| ctx.grid.surf_at(i)).collect();
```

`surf_at` is `R + H`. The **terrain**. And `ground` is not a detail inside the
solve — it is three of its boundary conditions at once: the **seepage cap** (an
unconfined water table may not stand above its own land), the **lake datum**
(free water pins the head at the ground plus the ponded depth), and the
**free-surface start value** every relaxed cell begins from.

So the pass reached outside its declaration, and reached for the one plane in the
whole system that is rewritten six times per epoch. *Which* of those six writes
it saw was decided by `passgraph`'s id-lexicographic tie-break: `dc:deep/head`
happens to sort before `dc:deep/transport`, so it read the pre-incision terrain.
Rename it and the head field's values change — and with them the vertical flux
the record keeps, which is what the next tier consumes.

This is the third instance in three sweeps of a tie-break deciding physics. The
first (`weather_inventory`'s `BioMod`) was fixed by declaring honestly. The second
(`reads_prev` consumed by nothing) needed a new **mechanism** — the
anti-dependency edge, journal/0104. This one needed both, and the interesting part
is *why* declaring honestly was not enough on its own.

## The claim that was half right

The spine-audit's verdict was: *"Declaring `Forced` is free, and it pins it."* The
ROADMAP entry that carried it forward added: **"Verify that claim before trusting
it."**

The first half is true. The revision `head` reads **is** `Forced`, in every cfg
path, and that is determined from the code rather than chosen. Between the pass
that writes `Forced` and the pass that next touches `R`/`H`, the roster runs
`drainage` (`build_surface`/`flood`/`route`/`accumulate_area` — takes `&DeepGrid`),
`frost` (`periglacial`, `&DeepGrid`) and `geotherm` (writes `grid.geotherm` and
nothing else). None of them mutates the terrain. So the ground surface at `head`'s
firing point is exactly the terrain `forcing` left, on both the legacy path (where
`apply_uplift` adds into `R`) and the tectonic path (where `apply_thickening` only
grows `t_crust` and elevation waits for isostasy). One revision, no cfg-selected
slices needed.

It is also the revision the pass *should* read, which is the better reason to pin
it there. `head` takes four arguments off the live solve: `filled`, `routed`,
`area` and `ground`. The first three are snapshots `build_surface` took **from the
`Forced` terrain**. If `ground` came from any later revision, the seepage cap would
describe one landscape while the free-water anchors described another — the water
table would be capped at a surface the lakes and streams no longer sit on. The
four have to come from one moment or the boundary conditions contradict each other.
So there was no tension here between "what the body reads" and "what is physically
right": they are the same answer, and the declaration only had to say it.

**The second half of the claim is false.** Declaring `Forced` does not pin it.

## Why a `reads` edge only pins one side

The deep-time runner's terrain is a *pipeline*, and it is declared as one: each
stage that transforms `R`/`H` publishes its output as a distinct **revision
token** the next stage consumes — `Forced → … → Weathered → Diffused →
Compensated → Windblown → Settled`. That is what forces the topo-sort to reproduce
the hand-written phase order.

But a revision token is, to the graph, an ordinary resource. Reading `Forced`
creates one edge: `forcing → head`. It says *"I come after the pass that made
this"*. It says **nothing at all** about the pass that will overwrite the same
underlying plane next, because that pass writes a *different token* — a different
resource entirely. The graph has no idea `Forced` and `Weathered` are the same
`Vec<f64>` seen at two moments.

For every pass in the erosion pipeline this has never mattered, and that is why it
went unnoticed for so long. Each of them is held on the far side by a **forward
edge into the stages after it**: `drainage` reads `Forced` and is pinned before
`transport` because `transport` reads `Routed`; `frost` reads `Forced` and is
pinned before `weather` because `weather` reads `Frosted`. The pipeline braces
itself.

A **sidecar** has no such brace. `head` writes only its own field; the only pass
that reads it is `flow_record`, which is itself downstream of `transport`. Nothing
downstream of `head` constrains it, so it floated — free to slide anywhere after
`drainage`, with the tie-break as the only thing holding it in place. The head
field is the first pass in this system that reads a terrain revision and hands
nothing back to the terrain, which is exactly why it is the first one exposed.

**The generalisation, which is the part worth keeping:** in a system where one
plane has several revisions per epoch, *declaring the revision you consume pins
you on one side only*. To pin the other you need the anti-dependency — a
`reads_prev` naming the **next** revision of that plane. And it must be the next
one, not the last one: an anti-dependency against the final revision would let you
slide past every writer before it.

## The missing link in the chain

There was a hole in the revision chain, and it was sitting exactly where `head`
needed a writer to be ordered against. `dc:deep/transport` — stream transport plus
bedrock incision — **mutates the terrain**: it lowers `grid.r` by incision, moves
`grid.h` by entrainment and deposition, and dumps suspended load into sinks. It
declared `writes: [Energy, DeltaH]`. Its by-products. Not the terrain.

So the chain read `Forced → (nothing) → Weathered`, and there was no name in the
vocabulary for the moment the ground surface first changes each epoch. A pass
reading `R + H` in that gap could not say *when* it read, because the vocabulary
had no word for it.

`DeepAxis::Incised` is that word. `transport` publishes it; `weather` reads it
(it genuinely does — cover thickness and bedrock, as transport left them); and
`head` lag-reads it. The declaration is now the **pair**:

```rust
id: "dc:deep/head",
reads: &[Routed, Forced],          // after the writer that produced this terrain
writes: &[Head],
reads_prev: &[Recorded, Incised],  // before the writer that supersedes it
```

Neither line alone pins anything. Together they define exactly the window in which
`head`'s four inputs describe one landscape, and the tie-break never gets a say
again. `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename` picks up
the `Incised` side for free (it enumerates the roster's lagged reads and renames
each victim to an id that provably loses the alphabet fight against every writer of
the axis it lags on); the `Forced` side gets its own hostile rename —
`dc:deep/aaa_head`, which would win the tie-break against `dc:deep/forcing` and
still cannot be scheduled ahead of it.

## `dc:deep/climate`, and why the first revision beats three

The same slice closes the smaller thread journal/0104 filed against itself.
`climate::march` reads `grid.surf_at` — the start-of-epoch topography, orographic
precipitation marched over last epoch's final surface — and declared
`reads_prev: &[]`.

The obvious declaration is the *last* terrain revision of the epoch, which is what
the pass actually observes the value of: `Settled` with the agents, `Compensated`
on the bare tectonic path, `Diffused` legacy. Three cfg-selected slices, in the
`WINV_READS_*` mould.

The better declaration is one slice: **`reads_prev: [Forced]`**. An
anti-dependency against the *first* terrain revision of the epoch orders `climate`
before `forcing`, and every later terrain writer — `transport`, `weather`,
`diffuse`, `isostasy`, the agents — is downstream of `forcing`, so one token
covers the whole chain transitively. Lagging against the last revision would pin
strictly *less*: it would leave `climate` free to slide past `forcing` and
`transport`, which is most of what it needs protection from. And it says in the
vocabulary exactly what the code comment already claimed in prose — *"climate
samples the pre-forcing surface"*.

That is the shape for any "I read the plane at the top of the epoch" pass: lag
against the **first** writer, not the last.

## What moved

**Nothing.** Stated plainly, because that is a fine result and the slice was
authorised to produce a different one.

The user's ratification came with the consequence attached — *"it should declare
what it reads and we eat it if it changes the physics; the world is a scratch pad
right now"* — and the brief was explicit that byte-identity here is a regression
detector, not a target, and that choosing a revision to preserve the current output
would be the wrong move. So the revision was picked from the code and from the
physics, and the numbers were run afterwards to find out what happened, not to
decide what to declare.

Every edge the four new declarations introduce was already in the graph's
transitive closure:

| declaration | edge added | already implied by |
|---|---|---|
| `climate.reads_prev += Forced` | `climate → forcing` | `forcing` reads `Climate` |
| `transport.writes += Incised` | `transport → weather` | `weather` reads `DeltaH` |
| `weather.reads += Incised` | (the same edge) | — |
| `head.reads += Forced` | `forcing → head` | `forcing → drainage → head` |
| `head.reads_prev += Incised` | `head → transport` | *nothing — this is the new pin* |

Only the last one is new information, and it points the way the tie-break was
already pointing. Pass order, before and after, all 17:

```
climate · expose · tectonics · forcing · drainage · frost · geotherm · head ·
transport · flow_record · weather · diffuse · isostasy · deposition · eolian ·
wave · biotic
```

Proven directly rather than by appeal to a hash elsewhere:
`the_terrain_revision_declarations_are_schedule_neutral` rebuilds each of five
rosters with the pre-slice declarations restored and asserts the order is
identical.

And the vertical-flux record, the thing that would have moved if the order had —
`examples/head_field_probe.rs`, seed 1337, `Extent::Medium`, production flags, run
before and after on the same machine:

| | journal/0098 baseline | after |
|---|---|---|
| vertical entries | 307,364 | **307,364** |
| DOWN / UP | 306,227 / 1,137 | **306,227 / 1,137** |
| columns carrying vertical flux | 131,586 (44.301 %) | **131,586 (44.301 %)** |
| magnitude mean / p95 / max | 0.501013 / 0.720093 / 10.0 | **identical** |
| artesian columns (max excess) | 60 (2.935 m) | **60 (2.935 m)** |
| confined subaerial columns | 1,044 | **1,044** |

Terrain byte-identity held, as it had to — `head` writes only its own field and
the exchange plane, and `the_production_world_still_hashes_to_the_pre_slice_goldens`
is unmoved. If it had *not* held, that would have meant the head field was not the
pure sidecar it claims to be, and the instruction was to stop and say so. It did
hold.

## Why this is capabilities work, not tidying

The north star's whole bet is a pass system where materials, their behavior, and
the passes over them are authored in **one uniform, self-declaring,
compiler-validated shape**, safe enough to hand to untrusted third parties in a
WASM sandbox. The load-bearing word is *self-declaring*: the runtime's contract
with a pass is that its declaration is the **complete** account of what it touches,
because that declaration is what a sandbox will be built out of. Reads become the
capability grant. A pass that can reach a plane it never named is not a pass with a
documentation defect — it is a pass whose sandbox has a hole in it, and it is
exactly what that architecture outlaws.

Building toward that while leaving a default-on pass reading the terrain out of
band would have been building something that gets outlawed later. It is cheaper to
be honest now, at the cost of one enum variant and five declaration lines, than to
discover at sandbox-time that half the roster's declarations are approximations.

The other half of the lesson is about the audit itself. The finding arrived with a
prescription attached — *"declaring `Forced` is free, and it pins it"* — and it was
half right, in the way that is hardest to catch: the true half was checkable in
thirty seconds and the false half looked like the same sentence. The instruction
that saved it was one line in the ROADMAP: **verify that claim before trusting
it.** A diagnosis is not a fix, and a fix proposed by the same pass that found the
defect deserves the same scrutiny as the defect.
