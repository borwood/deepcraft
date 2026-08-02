# The plan learns to say what its parts are

*2026-08-01 · B0, the body-plan structure slice · dc-api, dc-client*

> blogworthy: five separate rulings collapsing into one two-type mechanism —
> and a validator whose job flipped from "is this on my list" to "did you
> supply what you claimed". Lens 3 (the right primitives, nothing bespoke),
> with a lens-1 thread: the defeat-check sweep that found three more consumers
> of the same seam *before* the design was frozen, in files nobody had open.

## The shape of the problem

After the 2026-08-01 design conversation, the bodies arc held five ratified
rulings that read like a list: an open action vocabulary, functional parts
declared not derived, address + role as two identifiers, balance scoped to
standing bodies, support as a per-segment capability activated per mode.

The design pass (`docs/audits/2026-08-01-body-plan-structure-design.md`) had
proposed five new types to carry them — `ContactDef`, `LimbDef`,
`LimbPurpose`, `ModeDef`, `Locomotion`. Working the rulings back through the
proposal collapsed it: **a contact is a role with an anchor that a mode names
as bearing; a limb's purpose is which modes name it; a support kind is the
dimensionality of the active contact set.** Two types survived — `RoleDef`
and `ModeDef` — plus the rename `slots` → `actions`. The five rulings were
one mechanism wearing five descriptions.

## What the engine stopped owning

`KNOWN_VERBS = ["idle", "walk", "jump"]` and `REQUIRED_VERBS = ["idle",
"walk"]` are gone. They were the loudest finding of the design pass's
five-body table: a bird cannot bind `fly`, a tree cannot walk, and both were
hard define-time rejections — from an engine that never reasoned about verbs
at all. Verbs were clip-slot keys; the client picks idle/walk by speed. A
closed set with no machine justification (anti-shape A-7, one level up from
the usual material-naming instance), colliding with north-star Deviation 2.

The replacement check is smaller and stronger: **you supplied what you
claimed.** Every action a plan declares must bind a registered,
joint-compatible clip; every role a mode bears on must exist on some
segment. Nothing is mandatory. The two acceptance tests are the two bodies
the old contract rejected: `a_plan_with_no_actions_is_legal` (the tree) and
`any_action_name_is_legal_when_backed` (the bird's `fly`).

What the *driver* does with a plan that lacks `walk` is the driver's affair —
the client's locomotion driver asks for it by name and reports the plan as
unbuildable rather than inventing a pose. That line (engine checks claims,
consumers own vocabularies) is the whole ruling in one sentence.

## What the plans started saying

The biped, stout and longleg each declare: `sole` on both lower legs —
anchored at the **bottom face** of the bone's box, not its centroid, in
segment-local coordinates — `look` on the neck, `face` on the head, and one
mode, `stand`, bearing `[sole]`. Only what has a consumer today; the open
vocabulary means the scorpion's `stinger` costs nothing when it comes.

Three name-couplings died to the declarations:

- **`leg_rigs`' string surgery** (`starts_with("leg_")`,
  `replace("_upper","_lower")`) — the one true naming coupling, flagged in
  its own doc comment since journal/0130. The rig is now derived by walking
  parents from each declared sole to the first branch point; the chain is
  derived, the contact is declared. A plan gets foot placement in any names
  it likes.
- **`character.rs`'s `== "neck"` and `== "head"`** — the two lookups the
  design pass showed fail *silently* on a generated body. They are now role
  queries resolved once per plan, **unique-or-loud**: zero matches is
  feature-off (what a silent miss used to mean, now visible), many names the
  contenders and disables the feature. Never a silent first-match.
- **`longleg_plan`'s own mutation loop**, which found the bones to lengthen
  by the same prefix. It now finds them by declaration — the sole-bearing
  segments and their parents — and *re-derives the anchor* after moving the
  bone, so the declaration cannot go stale against the geometry it sits on.

## The mirror, and a minus sign

The defect that opened this whole arc was two independent lists of magic
numbers whose symmetry was a coincidence of hand-typing. The design pass
verified the right-side rows are exactly the left with lateral coordinates
negated; the authoring functions now say so — `mirrored(&arm_l_upper,
"arm_r_upper", ...)` — with the byte-identity acceptance test pinned to the
old literals.

One wrinkle earns its sentence: negating an authored `0.0` yields `-0.0`,
which is `==` equal and **bit** different — and a hash or a postcard encoding
sees bits. `mirror_coord` normalises the zero, and the test asserts
`is_sign_positive()` on every mirrored zero, so the helper provably
reproduces the hand-typed bytes rather than a float-equal impostor.

## What this slice deliberately did not do

- **`collide` is not a field** (user ruling at ratification): its only
  consumer is B4's derived collider sets, and the bird's folded-vs-spread
  wing suggests participation may be *mode-scoped* — a static bool today
  would pre-answer B4's design question. Left open for the machinery.
- **Clip binding stays address-exact** — stubs.md § 34. Roles landed for
  systems; animations bind to roles at the gait bake, before the firewall
  moves. The `JointRot.segment` doc comment now carries the marker.
- **No golden moves, no behaviour change** — bodies sit behind the
  determinism firewall; every new field has an identity default; the wire is
  a recompile because nothing persists a `BodyPlan`. The three shipped plans
  produce byte-identical rigs (the sole anchors reproduce the retired
  `|offset.y| + size.y/2` arithmetic exactly, straight-down anchors taking a
  fast path so no `sqrt` noise enters shipped numbers).
- **The hover is untouched** — corrections #80's space-layering defect is
  upstream of everything visual and rides until its own slice. No walk has
  run since the quantizer came out.

## What it unblocks

The resting-posture bake (B2 member #0) now reads **declared contacts** —
the solver takes soles as input data instead of deriving feet from three
bipeds' naming habits, which was the N=1 trap (corrections #78) that made
`structure → posture → gait` the ruled order. And a bird, snake, alligator,
centaur or tree now *defines* — the declaration layer no longer prejudges
what a body is allowed to be, which is the property the evolution arc was
waiting on.
