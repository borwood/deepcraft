# 0031 — The console reads the registry, not the commands

The game could already be driven two ways: a human at the mouse (LMB/RMB
edits) and an agent over MCP (port 7777, the whole dc-api surface plus three
client-shell tools). What it lacked was the thing every voxel game grows early
— a text console you drop with a keypress and type commands into. The catch
that makes this entry worth writing is what a deepcraft console must NOT be: a
switch statement of hand-written command handlers. We already decided (API.md
principle 5, "Reflectable") that consumers are *generated* from
`dc_api::schema::registry()` — the MCP tool list, the WASM glue, the editor
property UIs. A console with its own per-command code would be a fourth
consumer that could drift from the other three. So the whole exercise was:
build a console where a command added to the registry tomorrow appears, with
autocomplete and help, with **zero console changes**.

## The shape that falls out of that constraint

Everything the console knows is a `Vec<ConsoleCommand>` built once at startup
from `registry()` (each spec gives a name via `mcp_tool_name`, a doc, a
capability string, a `payload_schema()`, and a kind) plus the three client
tools appended exactly the way `mcp.rs` appends them to the MCP tool list. From
that table, four pure functions do all the work, none of them naming a single
command:

- **parse** — split a line into `command` + `key=value` tokens;
- **assemble** — expand flat dotted keys (`pos.x=10 pos.y=64 block=dc:stone`)
  into the nested JSON the command's schema describes, coercing each value to
  the schema's declared type;
- **complete** — command names by prefix, then (after a known command) that
  command's parameter keys, walked out of its schema as dotted leaf paths with
  type + required-marker + description;
- **help** — `help` lists everything; `help <command>` renders the spec's doc,
  capability, and every parameter from the schema.

These live in `console/core.rs` with no Bevy or render types at all, so they
are exhaustively unit-testable (14 tests: parse, nested-path expansion,
type coercion, unknown-key rejection, command/param/help completion, help
rendering). The Bevy shell (`console/mod.rs`) is a thin, replaceable view: T
opens, Esc closes, Enter submits, Tab completes, up/down browse history,
rendered with built-in `bevy_ui` `Text` — no egui, no new dependency (the
pinned Bevy 0.19 makes third-party UI crates a compatibility gamble we didn't
need to take).

> blogworthy: "the console that has no commands in it" — the entire feature is
> a projection of one registry table through four schema-walking functions, and
> the proof is that the unit tests construct their world from `registry()`
> itself.

## The one-door decision, and why the client tools stay outside it

The tempting shortcut was to have the console call `authority.handle_api_call`
directly — it's ECS-side already, why cross a channel? But there is a subtler
"one door" available. The in-client MCP server maps a tool name + JSON onto a
`BridgeRequest` and pushes it onto a channel `drain_bridge` empties; queries,
commands, screenshots, and pose get/set all resolve through that single
dispatcher. Rather than duplicate any of that mapping, the console became a
**third producer on the same bridge**. `ClientMcpServer::request_for` — which
already knew how to turn `client_screenshot` / `client_player_pose_set` / any
registry tool into the right `BridgeRequest` — was lifted to a free
`bridge_request_for(name, args)` that both the MCP server and the console call.
The console assembles the JSON, hands it to that one function, sends the
request, and polls the oneshot for the reply exactly as an MCP call awaits it:
the SubmitAck is a local echo, the receipt (or query result, or rejection)
lands at the tick boundary. There is no second dispatch path, and the ECS side
still stamps the dev-session token — the console holds no grants of its own.

This is also *why the three client tools stay outside the registry* and the
console still surfaces them: they touch the window and renderer, not the world
(a screenshot is not a tick-boundary command with a capability). They are
hand-authored in `mcp.rs`, and the console appends them to its table the same
way the MCP `list_tools` appends them — reading their hand-authored schemas for
completion and help. The firewall (headless world vs. GPU shell) is preserved:
the console surfaces the client tools without pretending they are dc-api
commands.

To make the console work even with `--no-mcp`, the bridge channel is now always
created; only the server *threads* are gated by the flags. `spawn_servers`
returns the receiver-side `McpBridge` **and** a sender clone for the console.

## Two wrong turns

**The 20-system wall.** Adding `console_input` to the front of the app's main
`.chain()` blew up with a wall of `for<'a,'b,…>` lifetimes: Bevy's tuple trait
impls stop at 20 elements, and the gameplay chain was already exactly 20. The
fix is a nested tuple — `(console_input, (…20 systems…).chain()).chain()` —
which keeps the outer tuple at two elements while preserving the inner order.
`console_input` must run first because of how it swallows input.

**Swallowing input without gating every system.** The console must stop the
player from walking, jumping, breaking blocks, switching scale, or tossing a
cube while you type. Gating each of those systems with a run-condition would
have been a broad, conflict-prone diff into `app.rs` (a sibling worktree is
touching arg parsing). Instead `console_input` runs at the front of the chain
and, while open, calls `ButtonInput::reset_all()` on the keyboard and mouse —
so every gameplay system downstream this frame sees empty input — and releases
the cursor grab so mouse-look (which reads grab state, not the button input we
reset) stops too. Text itself comes from the `KeyboardInput` message stream's
`text` field, which honors layout and shift, so the reset never eats what the
console is typing. One system, one front-of-chain insertion; `app.rs` grew a
plugin line, an import, and the nested-tuple wrapper, nothing more.

## What it looks like in game

Press **T**: a translucent panel fills the bottom of the window with a
scrollback and a `>` prompt; the player freezes. Type `help` and every command
in the registry lists itself. Type `world_get_block ` then Tab and it completes
`pos.` for you; fill in `pos.x=0 pos.y=64 pos.z=0`, Enter, and the block name at
that voxel prints back. `world_set_block pos.x=0 pos.y=64 pos.z=0 block=dc:stone`
prints `submitted` then the receipt when the tick lands, and the block appears.
`client_player_pose_get` prints your pose (with `eye_in_solid`). Esc closes and
you're walking again. Add a command to `registry()` and it is simply *there*
next launch — the console never learns its name.

## Follow-ups (v0 scope notes)

- **Value-level completion is out of scope for v0.** Completion stops at
  parameter keys; it does not suggest block names (`dc:stone`), entity kinds, or
  enum members. The hook is a schema `enum`/value-source lookup at the leaf; a
  follow-up.
- **Array / deeply-structured payloads** (`define_body_plan.segments`,
  `define_anim_clip.keyframes`, `events_subscribe.kinds`) can't be expressed in
  the flat dotted syntax; the assembler accepts a raw JSON literal at those
  leaves as an escape hatch. A structured entry mode is a later slice.
- The look (translucent panel, monospace-ish default font, colors), the **T**
  binding, and the `key=value` / dotted-path syntax are dev-tool defaults that
  ride as-built — flagged for ratification, not decided here.
