# 0033 — Moving registry self-consistency from the test suite to the compiler

Two ratified dc-api improvements landed together (API.md decisions 6 and 7).
Both are about the same thing from opposite ends: the command registry is
supposed to be the single source of truth the whole surface is *generated*
from (principle 5), but two seams still let it drift, and one of those seams
was only caught by a test.

## The four hand-synchronized places

Adding a command to dc-api meant editing four sites that had to agree:

1. the `ids::WORLD_SET_BLOCK` string const (in `payload.rs`),
2. the `Payload::SetBlock(SetBlock)` enum variant (in `payload.rs`),
3. the `CommandSpec { id: ids::WORLD_SET_BLOCK, .. }` row (in `schema.rs`),
4. a sample in `registry_covers_every_payload_variant`, whose whole job was to
   assert `samples.len() == registry().len()` and round-trip each.

Forget #3 and everything still *compiled*; the completeness test failed at
`cargo test` time with a length mismatch. That is exactly the class of bug the
"reflectable, generated, cannot drift" principle exists to forbid, and here it
was being enforced by discipline plus a test artifact. Decision 7 says: make it
a compile error instead.

## One table, and the wire-order constraint that shaped it

The fix is a single `macro_rules! commands { ... }` table in `schema.rs`. Each
row is `Variant: Kind { id: CONST = "dc:...", doc, cap, schema, complete }` and
expands, together, to the `ids::` const, the `Payload` variant, its
`command_id` arm, and the `registry()` entry. You cannot add a variant without
its row (same row) nor a row without the variant (the generated `command_id`
match would name a variant that does not exist). The old length-equality test is
now a tautology carried only to guard the *sample list* against rot; the real
guarantee moved to `rustc`.

The one genuinely forced consequence is worth recording, because it looks like a
violation of "change nothing" until you see why it is unavoidable. Postcard
encodes an enum variant by its **declaration index**, so the `Payload` order is
wire identity and can never be reordered. But `registry()` had historically been
kept in a *different*, human-grouped order (all the `world/*` together, then
`registry/*`, ...). A single ordered table cannot emit two different orders. The
enum order is non-negotiable, so the table is in wire order and `registry()` now
returns that same order. This changes the *order* of the generated MCP tool list
— a cosmetic change no consumer depends on (clients look up tools by name; no
test asserts registry order) — while the observable per-command contract (every
`payload_schema()` JSON, every id, every decoder) is byte-for-byte identical.
The alternative — keeping the enum hand-written and generating only ids +
registry + `command_id` — would also make omissions a compile error (via the
exhaustive `command_id` match), but it leaves the enum as a second edit site and
does not honour the decision's literal "one declarative site... emits the
Payload variant." So: generate the enum, accept the cosmetic reorder, document
it here.

> blogworthy: "the registry that cannot be wrong" — how a 12-line `macro_rules!`
> table turns a whole category of drift from a test failure into a compile
> error, and the postcard-wire-order constraint that decides which of two
> orderings the single source of truth is allowed to have.

### Where the code lives now

The macro forced a small relocation. `registry()`, `CommandSpec`,
`mcp_tool_name`, and friends must keep their `dc_api::schema::` paths (consumers
import them there), so the whole table stays in `schema.rs` — which means the
`Payload` enum and `ids` are now *defined* there and re-exported from
`payload.rs` (`pub use crate::schema::{Payload, ids};`). Every existing path —
`crate::payload::Payload`, `dc_api::ids::WORLD_SET_BLOCK`, `dc_api::Payload` —
still resolves through the re-export, so no consumer edit rides along. The
per-command payload *structs* stay hand-written in `payload.rs`; the macro only
owns the union, the ids, the id-mapping, and the registry.

## Decision 6: the completions hook, and the static/world split

Value-level completion (`block=<TAB>` → registered block names) had no source in
the registry. Parameter *names* and types come free from `payload_schema()`; the
legal *values* are dynamic world/registry content the schema cannot know. So
`CommandSpec` grew an additive `completions: Option<Completer>`.

The signature question was the interesting part. The obvious shape is
`fn(&HostWorld, param_path, prefix) -> Vec<String>`, and dc-api owns `HostWorld`
so that stays in-crate and wasm-safe. But some sources are genuinely
world-independent — a fixed enum like event kinds or posture names — and forcing
them to take a world they never read is a lie in the type. So `Completer` is a
two-variant enum:

```rust
pub enum Completer {
    Static(fn(param_path: &str, prefix: &str) -> Vec<String>),
    World(fn(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String>),
}
```

`param_path` is the payload field to complete (a dotted path for future nested
fields; today a top-level key), `prefix` is what has been typed. A completer owns
only the params with a real source and returns an empty vec for anything else —
which is also the honest answer for an unknown path.

Real sources, no invented ones:

- **`block`** (`set_block`, `fill`) → `World`, reads `HostWorld::block_names()`.
  This is world-backed *on purpose* even though today it returns a fixed table:
  the block registry is a filed later slice, and when it lands `block_names()`
  reports registered blocks and the completer does not change. (Placeholder
  state is not the design target.)
- **`character`** (the six controller/sense verbs) → `World`, reads live
  `HostWorld::characters()` — a genuinely dynamic source, the one that makes the
  `World` variant meaningfully world-dependent today.
- **`class`** (`define_class_member`) → `World`, reads registered content
  classes — a member joins an existing class, so its name completes from the
  registry.
- **`posture`** (`set_posture`) → part of a `World` completer that also handles
  `character`; the one command with a source on two params.
- **`kinds`** (`events/subscribe`) → `Static`, the fixed event-kind vocabulary —
  the purely world-independent case the split exists for.

Everything else carries `None` (no `kind` entity registry exists, a spawn `name`
is a *new* name, a scan region is coordinates). A unit test pins the presence/
absence per command and asserts the two lists partition the whole registry, so a
future command is not silently forgotten by the completion layer either.

No consumer is wired to any of this — the console/MCP integration is a follow-up
owned elsewhere. dc-api only.

## Proof the schemas didn't move

The byte-identity constraint (consumers parse property *descriptions* for help)
is checked, not asserted by hand: `dc-mcp-dev`'s session test already compares
each tool's `input_schema` against `(spec.payload_schema)()` for every command
over a real MCP session, and it stays green. The `s_*` schema builders were
copied into the macro table verbatim; no schemars, no `$ref`, same inline shape.
