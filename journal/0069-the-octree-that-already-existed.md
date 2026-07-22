# 0069 — The octree that already existed

*2026-07-22. A design pass, not a build — and the first design pass to get a
journal entry, for a reason recorded at the end. Companion doc:
`docs/design/octree-substrate.md`, where the decisions live; this is the
narrative of how they were reached.*

> blogworthy: **"we didn't design an octree; we noticed we had one."** The
> engine's most load-bearing structure was already shipped in pieces under
> four names, and the design pass consisted of naming it, giving its node a
> contract, and hunting the contract for holes. Also the ratification-hygiene
> moment: the user catching the integrator ratifying a rejection nobody had
> consciously made.

## The prompt

The user asked what our priority order should be, adding: *"i think octree /
the farfield rework needs solved."* The corpus agreed loudly. Three queued
items — FF2b, the far-field summarization half of the surface-branch fix, the
unwritten S16 — were all filed as waiting on "the octree question," and
water.md had been saying for two days that the shared-representation question
should be asked *before* either consumer got built.

## The finding: the pieces were already on the shelf

The sweep-before-design-pass rule earned its keep again. What the corpus
held:

- **The S3 chunk pyramid is an octree** — 32³ cubes at 3D positions, each
  level-L+1 chunk derived from 8 children. ARCHITECTURE committed to "LOD is
  a 3D lattice, octree-style" on day one.
- **The material half of the reduction was built, tested, and unrendered** —
  `MixtureDownsampleRule`, sitting in spines § 3 as the index's first row:
  64 child eighths → 8 parent eighths, debris-aware, proven to compose with
  the block pyramid. Machinery with no consumer, exactly the shape the
  spines file exists to catch.
- **FF2a left the socket** (`ColumnSpan` stacks, pure mesher), **HostWorld
  already did committed/fluid at L0**, and the DH/Voxy recon had already
  decided the persistence outline.

So D1 wrote itself: **one substrate = the existing pyramid, named**. No new
spatial structure. The pass's real work was the node payload contract.

## The correction worth blogging

Presenting the far-register options, the integrator wrote that Aokana-style
ray-marched SVDAG was "already effectively rejected by the Bevy-mesh-path
decision." The user: *"i did not (consciously) ratify no-aokana. i do not
understand it."*

They were right. The voxy-dh addendum rejected *Voxy's* bespoke
command-generation and vertex-pulling — a submission-machinery decision. It
never adjudicated the mesh-free paradigm. "Effectively rejected" was the
integrator extending a decision's scope past what was ratified, which is the
quiet way a corpus rots: not wrong facts, but real decisions inflated into
neighboring decisions nobody made.

Unpacking Aokana produced the useful clarification: **ray-marched voxels
still look like stepped voxels**, so stepped-all-the-way (D3) is an
*appearance* decision and mesh-vs-march is a *production* decision beneath
it — orthogonal, both now recorded that way. Aokana stands OPEN and
spike-gated on FF2b-minimal's own measurements. And the user's parting
observation — *the near field itself could have been built mesh-free from
the beginning* — went into the doc with the reasons the mesh commitment is
architecture rather than habit (sub-voxel shapes at mesh time, the pack
pillar's forward surface, upstream-maintained GPU path, bounded remesh).

## The gap-hunt: who actually uses this thing?

The user asked what the contract buys, who consumes it, and whether anything
was missing. Enumerating consumers (far renderer, adaptive load volume,
skylight caches, water's three coarse questions, coarse light, the
statistical tier) surfaced the hole:

**v0 only described bottom-up derivation, and most far-field nodes have no
bottom.** At 10 km nearly every node covers terrain that has never been
generated as chunks — and corrections #31 had already taught, in another
costume, that never-generated ground is the *default* case. A node over
ungenerated volume cannot be a reduction; it must be **synthesized top-down**
from the worldgen statistical authority, seeded and deterministic, with a
statistical agreement test where the two directions meet. That rule is also
the legal home the surface-branch fix's summarization half has been waiting
for — the far-field need that once leaked *into* the authority now has an
address *outside* it.

The corollary nearly shipped as a defect: `derive_material_lod_chunk` reads a
missing child as **empty**. Correct in a test scene; wrong over a lazy world,
where missing means "synthesize me." Air and not-yet-derived are different
facts. v0.1 pins the distinction, plus strata independence (light and
distributions are already queued to arrive) and explicit seeding for
synthesis. Ratified same session.

## Why this entry exists

The user, reading the pass: *"one thing missing from our journals … reasoning
like this isn't captured in them. is it captured anywhere?"* It wasn't — the
conclusions were, in DECIDED entries, but the narrative (the correction, the
hole, the road not taken) lived only in conversation. The journal charter had
asked for "the reasoning behind the decision" all along; practice had only
ever journaled builds. Design passes get journal entries now. This is the
first.
