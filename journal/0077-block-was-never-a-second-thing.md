# 0077 — block was never a second thing

*2026-07-22, late. A design pass with no code — the decision that block and
material are one namespace, reached by the user rejecting a slice I had queued
as if it were routine. Recorded here because the reasoning is the deliverable;
`materials.md` carries the two DECIDED entries, this carries how we got there.*

> blogworthy (lenses: deepsim-reflexions, AI-native development): **the moment
> the modder's instinct overruled the engine's convenience.** An architecture
> decision had been made — block decoupled from material — that nobody had ever
> put to the person whose whole reason for building this game was heavy
> modding. He found it by being asked to ratify its downstream consequence, and
> his objection dissolved a chokepoint three fixed vocabularies deep.

## The slice that shouldn't have been routine

The last A-7 defect standing was `classify::block_twin`: a match of fifteen
named materials to blocks, with a `_ => Block::Stone` arm that silently
swallowed nine more. By the A-7 diagnostic it was clean-looking work — I
proposed moving the wardrobe from code to a registry field, default-self,
load-time validated. A tidy slice. I brought it for ratification.

The user did not ratify it. He asked why block and material were two things at
all: *"to me, they are the same thing. They should always be coupled."*

## What I had to answer honestly

My first instinct was to defend the split — mixtures force it, a voxel holds
5/8 sandstone and 2/8 silt and 1/8 charcoal, "block" can only be a derived
one-name summary of that mixture, and the fill contract already couples them
in the only direction a mixture allows (truth → name). All true. But the user
kept pulling on the thread, and the thread was longer than my defense.

Because the split is *only* forced as far as "a mixture needs one name." It is
**not** forced that the name lives in a separate, smaller, art-gated,
un-registrable vocabulary. That second part — Block as its own closed enum —
was never a consequence of mixtures. It was inherited convenience: Minecraft
has blocks, the renderer wanted a cheap token, the far field wanted a small
palette. Three consumers, each reasonably wanting a compact identity, and their
sum quietly hardened into an ontology: *materials are physics, blocks are
presentation, and the two are different kinds of thing.*

The user's modding experience saw straight through it. His mithril example did
the work no argument of mine had: a pack adds mithril-the-material, full
property sheet, real erosion — and under the old ontology it works perfectly
and **looks like grey stone forever**, because the block vocabulary is a fixed
enum a pack cannot extend. That is not a summary doing its job. That is a
chokepoint. And it is the *same* chokepoint as the six erosion proxies (which
we killed this morning) and the fixed `Biofacies` record tags (which
entry-species will kill): three fixed vocabularies where a plugin-first engine
must have registrable ones.

## The decision, and why it is coherent rather than reckless

**Block collapses into material + Air.** A voxel's one-name is "which material
dominates this mixture." `classify` keeps its job — a mixture still needs its
one name chosen — but it answers with a *material identity wearing its own
face*, not a member of a parallel enum. The twin field is never built; the
slice I brought is superseded by the thing it was a workaround for.

The cost is real and was accepted with eyes open: art per material (every
material that can dominate a voxel needs a face), and a long-tail migration of
Block's remaining honest consumers — the storage palette, the far-field span,
the ~80 solidity checks (already draining into the fill contract's occupancy
primitives), the mesher's layer pick, the player-facing name. None of that
touches mixtures, partials, strata, or the geology. It is a token migration,
not a world-model change — which is exactly why it can proceed as a slow
background arc rather than a big bang.

Categories survive, and become the registrable thing: *"a category list which
can be registered to is defensible for materials, but the properties of the
materials themselves is what matters, no proxy."* Packs add classes, not only
members.

## The larger rule that fell out of it

Chasing charcoal one more step produced a second DECIDED entry that is bigger
than blocks. The user's picture of an honestly-authored fire: a pass checks for
flammable things in a cell, their density and kind decide the fire's behavior,
wind weights propagation, and *the charcoal outcome comes from the burned
material's own definition* — a `burned → charcoal at X conservation`
transformation axis, inheritable from a category, patchable per material,
deletable. And the load-bearing clause: **the runtime game and the deep-time
sim read the same rule.** One `burned` authority; the bulk deep-time arithmetic
may aggregate and approximate it but must never carry a parallel rule beside
it. That is S-3 — "a summary is derived from the authority, never beside it" —
applied to *processes* instead of data. It is the mithril principle promoted
from properties to transformations: mods add behaviors, and both clocks honor
them.

The standing counterexample is the fires pass we ship today: `soil_pool ×
FIRE_CHAR_FRAC`, a bespoke rule over a vegetation proxy the user explicitly
does **not** avow. Its heir chain is now written down in order — ecology
supplies the flammable inventory, the entry-species record carries what
actually burned, the burned-axis on the organic materials supplies the outcome
— and coal and charcoal ride as-built until those heirs land. *"I don't care
about coal and charcoal right now, they have heirs."*

## What this cost me to learn

I had queued a defensible slice and was ready to ship it. It was defensible and
it was wrong — not buggy, but pointed at preserving a distinction that
shouldn't exist. The tell I missed: I was proposing a *workaround for a
vocabulary limit* (materials can't get their own faces, so let them borrow),
and a workaround for a limit is usually evidence the limit is the actual bug.
The user, who does not read our code, caught it from the shape of the
consequence alone. The architecture decision to split block from material was
never ratified by anyone; it accreted. This is the second time in one session
the corpus (or the convenience) was ahead of nobody in particular and wrong —
and the only defense was a person asking "why is it like this at all."
