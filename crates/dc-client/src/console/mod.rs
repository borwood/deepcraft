//! The in-game dev console: press **T** to drop an overlay that exposes the
//! ENTIRE dc-api command/query surface plus the three client-shell tools, with
//! autocomplete and help.
//!
//! The load-bearing property (docs/API.md principle 5): everything the console
//! knows is GENERATED from `dc_api::schema::registry()`. The command table, the
//! completions, the help text, and the argument→JSON assembly are all built by
//! walking the registry's schemas — there is no per-command code here. A command
//! added to the registry appears in the console, with autocomplete and help,
//! with zero console changes. The client-shell tools (`client_screenshot`,
//! `client_player_pose_get/set`) stay OUTSIDE the registry (they touch the
//! window/renderer, not the world); the console appends them the same way the
//! MCP tool list does (`mcp.rs`), reading their hand-authored schemas.
//!
//! Dispatch is one door: a submitted line is assembled into JSON and handed to
//! [`crate::mcp::bridge_request_for`] — the SAME mapping the in-client MCP
//! server uses — then pushed onto the SAME [`crate::mcp::BridgeRequest`] channel
//! [`crate::authority::drain_bridge`] drains. The console is a third consumer of
//! that one bridge, not a second dispatch path; it stamps nothing itself (the
//! ECS side attaches the dev-session token, exactly as for an MCP call).
//!
//! The [`core`] submodule holds the UI-agnostic logic (parse, expand,
//! complete, help) with no Bevy or render types, exhaustively unit-tested.

pub mod core;

use std::collections::HashSet;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use dc_api::HostWorld;
use dc_api::schema::{self, Completer};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::authority::Authority;
use crate::mcp::{BridgeRequest, bridge_request_for, client_tools};
use core::{Candidate, CommandClass, Completion, ConsoleCommand};

/// How many scrollback lines to retain, and how many to show at once. The cap
/// is a few hundred lines so PageUp/wheel can scroll back through a real
/// session; only [`MAX_VISIBLE_LINES`] render at the current scroll offset.
const SCROLLBACK_CAP: usize = 500;
const MAX_VISIBLE_LINES: usize = 22;
/// Lines a PageUp/PageDown or one wheel notch moves the scrollback window.
const SCROLL_STEP: usize = 8;
/// How many completion candidates to list before eliding the rest.
const MAX_LISTED_CANDIDATES: usize = 24;
/// How many example values to show in the per-arg hint area.
const MAX_HINT_EXAMPLES: usize = 4;

/// The console plugin: owns the console resources and systems. `console_input`
/// is registered in the app's main system chain (it must run before the
/// gameplay input systems so it can swallow keystrokes while open); everything
/// else — the UI, the reply poll, the render — lives here.
pub struct ConsolePlugin {
    /// A sender into the shared MCP bridge channel; console submissions ride it
    /// exactly like MCP tool calls (one door).
    pub bridge: mpsc::UnboundedSender<BridgeRequest>,
}

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleBridge(self.bridge.clone()))
            .insert_resource(ConsoleState::default())
            .insert_resource(ConsoleCommands(build_command_table()))
            .add_systems(Startup, setup_console_ui)
            .add_systems(Update, (console_poll, console_render));
    }
}

/// The full command table: the registry surface plus the client-shell tools,
/// assembled once at startup. Appending the client tools mirrors `mcp.rs:373`.
fn build_command_table() -> Vec<ConsoleCommand> {
    let mut cmds = ConsoleCommand::from_registry();
    for tool in client_tools() {
        let schema = Value::Object((*tool.input_schema).clone());
        let description = tool.description.as_deref().unwrap_or("");
        cmds.push(ConsoleCommand::client(
            tool.name.to_string(),
            description,
            schema,
        ));
    }
    cmds
}

/// The command table, built from the registry + client tools (a resource so the
/// systems share one copy).
#[derive(Resource)]
pub struct ConsoleCommands(pub Vec<ConsoleCommand>);

/// The console's producer end of the shared MCP bridge channel.
#[derive(Resource)]
pub struct ConsoleBridge(pub mpsc::UnboundedSender<BridgeRequest>);

/// A submitted command awaiting its reply (an async receipt / query result /
/// client-tool result on the bridge's oneshot).
struct Pending {
    label: String,
    rx: oneshot::Receiver<Value>,
}

/// All console UI/interaction state. The overlay is a pure view of this.
#[derive(Resource, Default)]
pub struct ConsoleState {
    pub open: bool,
    input: String,
    history: Vec<String>,
    /// Index into `history` while browsing with the arrows; `None` = editing a
    /// fresh line.
    history_pos: Option<usize>,
    scrollback: Vec<String>,
    /// Lines scrolled UP from the bottom (0 = pinned to the latest output).
    scroll: usize,
    /// The live hint area: the recognized command's signature, plus a per-arg
    /// help line for the parameter the caret sits on. Recomputed each frame the
    /// console is open (cheap, pure schema walk + one completer read).
    hint: Vec<String>,
    /// Keys physically held while the console is open, tracked from the event
    /// stream so a movement key held across close can be re-pressed (the
    /// `reset_all` swallow otherwise leaves it stuck "released").
    held: HashSet<KeyCode>,
    pending: Vec<Pending>,
}

impl ConsoleState {
    fn push_line(&mut self, line: impl Into<String>) {
        self.scrollback.push(line.into());
        if self.scrollback.len() > SCROLLBACK_CAP {
            let overflow = self.scrollback.len() - SCROLLBACK_CAP;
            self.scrollback.drain(..overflow);
            self.scroll = self.scroll.saturating_sub(overflow);
        }
    }

    fn push_err(&mut self, msg: impl Into<String>) {
        self.push_line(format!("  ! {}", msg.into()));
    }

    /// The largest scroll offset that still shows a full window (so PageUp
    /// stops at the oldest retained line).
    fn max_scroll(&self) -> usize {
        self.scrollback.len().saturating_sub(MAX_VISIBLE_LINES)
    }
}

// ------------------------------------------------------- marker types ----

#[derive(Component)]
struct ConsoleUiRoot;
#[derive(Component)]
struct ConsoleScrollback;
#[derive(Component)]
struct ConsoleHint;
#[derive(Component)]
struct ConsoleInputLine;

// ---------------------------------------------------------- the UI -------

fn setup_console_ui(mut commands: Commands) {
    commands
        .spawn((
            ConsoleUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(45.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.85)),
            // Above the crosshair and any other UI spawned at startup.
            GlobalZIndex(1000),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            panel.spawn((
                ConsoleScrollback,
                Text::new(String::new()),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgba(0.82, 0.90, 0.82, 1.0)),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            // The live hint area: signature + per-arg help for what's being
            // typed. Dimmer/tinted so it reads as guidance, not output.
            panel.spawn((
                ConsoleHint,
                Text::new(String::new()),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgba(0.70, 0.78, 0.95, 0.92)),
            ));
            panel.spawn((
                ConsoleInputLine,
                Text::new("> "),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.65, 0.95, 0.70)),
            ));
        });
}

#[allow(
    clippy::type_complexity,
    reason = "three disjoint Text queries need mutually-exclusive marker filters"
)]
fn console_render(
    state: Res<ConsoleState>,
    mut root: Query<&mut Visibility, With<ConsoleUiRoot>>,
    mut scrollback: Query<
        &mut Text,
        (
            With<ConsoleScrollback>,
            Without<ConsoleHint>,
            Without<ConsoleInputLine>,
        ),
    >,
    mut hint: Query<
        &mut Text,
        (
            With<ConsoleHint>,
            Without<ConsoleScrollback>,
            Without<ConsoleInputLine>,
        ),
    >,
    mut input: Query<
        &mut Text,
        (
            With<ConsoleInputLine>,
            Without<ConsoleScrollback>,
            Without<ConsoleHint>,
        ),
    >,
) {
    if let Ok(mut vis) = root.single_mut() {
        *vis = if state.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !state.open {
        return;
    }
    if let Ok(mut text) = scrollback.single_mut() {
        // Render the window ending `scroll` lines above the latest.
        let total = state.scrollback.len();
        let scroll = state.scroll.min(state.max_scroll());
        let end = total - scroll;
        let start = end.saturating_sub(MAX_VISIBLE_LINES);
        let mut body = state.scrollback[start..end].join("\n");
        if scroll > 0 {
            body.push_str(&format!(
                "\n  -- {scroll} line(s) below (PageDown / wheel down for latest) --"
            ));
        }
        text.0 = body;
    }
    if let Ok(mut text) = hint.single_mut() {
        text.0 = state.hint.join("\n");
    }
    if let Ok(mut text) = input.single_mut() {
        // A trailing block stands in for a caret.
        text.0 = format!("> {}\u{2588}", state.input);
    }
}

// ------------------------------------------------------- reply polling ---

/// Drain resolved replies (receipts, query results, client-tool results) into
/// the scrollback. Async: a query answers within a frame, a command answers at
/// the next host tick boundary (exactly like the MCP path).
fn console_poll(mut state: ResMut<ConsoleState>) {
    let mut resolved: Vec<(usize, String)> = Vec::new();
    for i in 0..state.pending.len() {
        match state.pending[i].rx.try_recv() {
            Ok(value) => {
                let line = format_reply(&state.pending[i].label, &value);
                resolved.push((i, line));
            }
            Err(oneshot::error::TryRecvError::Empty) => {}
            Err(oneshot::error::TryRecvError::Closed) => {
                let line = format!(
                    "  {} — dropped (world reset or shutdown)",
                    state.pending[i].label
                );
                resolved.push((i, line));
            }
        }
    }
    // Remove in descending index order so earlier indices stay valid.
    for (i, _) in resolved.iter().rev() {
        state.pending.remove(*i);
    }
    for (_, line) in resolved {
        state.push_line(line);
    }
}

/// Render one reply value as a scrollback line, surfacing errors and rejections
/// plainly and echoing everything else as compact JSON.
fn format_reply(label: &str, value: &Value) -> String {
    if let Some(err) = value.get("error").and_then(Value::as_str) {
        return format!("  = {label}: error: {err}");
    }
    if let Some(reason) = value.pointer("/result/Rejected") {
        let reason = reason
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| reason.to_string());
        return format!("  = {label}: rejected: {reason}");
    }
    let body = truncate(&value.to_string(), 400);
    format!("  = {label}: {body}")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

// ---------------------------------------------------------- input --------

/// The keyboard front door. When closed, watches for **T** to open. When open,
/// consumes every keystroke — text into the line, Enter submits, Esc closes,
/// Tab completes, arrows browse history — and then RESETS the keyboard/mouse
/// input so the gameplay systems downstream in the chain (movement, edits,
/// scale switch, cube toss) see nothing while the player is typing.
///
/// Registered at the FRONT of the app's system chain so the reset lands before
/// any gameplay input system runs this frame.
#[expect(
    clippy::too_many_arguments,
    reason = "bevy system: each parameter is a distinct resource / event reader"
)]
pub fn console_input(
    mut reader: MessageReader<KeyboardInput>,
    mut wheel: MessageReader<MouseWheel>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut state: ResMut<ConsoleState>,
    cmds: Res<ConsoleCommands>,
    bridge: Res<ConsoleBridge>,
    // Read-only, for value-level completion + per-arg example hints. Optional so
    // a missing/contended world degrades to no suggestions, never a stall; the
    // completer touches only `&self` accessors, so it cannot mutate the sim.
    authority: Option<Res<Authority>>,
    mut cursor: Single<&mut CursorOptions>,
) {
    // Drain this frame's key events regardless (so a stale T press cannot leak
    // into the line the frame after opening).
    let events: Vec<KeyboardInput> = reader.read().cloned().collect();

    if !state.open {
        let opened = events
            .iter()
            .any(|e| e.state == ButtonState::Pressed && !e.repeat && e.key_code == KeyCode::KeyT);
        if opened {
            state.open = true;
            state.history_pos = None;
            state.scroll = 0;
            // Snapshot the keys physically held at open time; the event stream
            // keeps this current while open, so movement keys held across close
            // can be re-pressed (see below).
            state.held = keys.get_pressed().copied().collect();
            // Release the cursor so mouse-look (which reads grab state, not the
            // button input we reset) stops while the console is up.
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        }
        return;
    }

    let world = authority.as_deref().map(|a| &a.world);

    for e in &events {
        // Track physically-held keys from the raw event stream (survives the
        // reset_all swallow below, which the ButtonInput view does not).
        match e.state {
            ButtonState::Pressed => {
                state.held.insert(e.key_code);
            }
            ButtonState::Released => {
                state.held.remove(&e.key_code);
            }
        }
        if e.state != ButtonState::Pressed {
            continue;
        }
        match e.key_code {
            KeyCode::Escape => state.open = false,
            KeyCode::Enter | KeyCode::NumpadEnter => submit(&mut state, &cmds.0, &bridge.0),
            KeyCode::Backspace => {
                state.input.pop();
            }
            KeyCode::Tab => complete_input(&mut state, &cmds.0, world),
            KeyCode::ArrowUp => history_prev(&mut state),
            KeyCode::ArrowDown => history_next(&mut state),
            KeyCode::PageUp => {
                let max = state.max_scroll();
                state.scroll = (state.scroll + SCROLL_STEP).min(max);
            }
            KeyCode::PageDown => {
                state.scroll = state.scroll.saturating_sub(SCROLL_STEP);
            }
            _ => {
                if let Some(text) = &e.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            state.input.push(ch);
                        }
                    }
                }
            }
        }
        if !state.open {
            break;
        }
    }

    // Mouse wheel over the console scrolls its scrollback (the console owns focus
    // while open; no hit-test needed).
    for ev in wheel.read() {
        if ev.y > 0.0 {
            let max = state.max_scroll();
            state.scroll = (state.scroll + SCROLL_STEP).min(max);
        } else if ev.y < 0.0 {
            state.scroll = state.scroll.saturating_sub(SCROLL_STEP);
        }
    }

    // Swallow all input from the gameplay systems that run after us this frame
    // (including the close-frame's own Esc/keystrokes).
    keys.reset_all();
    mouse.reset_all();
    if state.open {
        recompute_hint(&mut state, &cmds.0, world);
    } else {
        // Closing this frame: reset_all just cleared the pressed state of every
        // physically-held key, and Bevy re-emits no event for a key that is
        // still down — so a movement key held across close would stay stuck
        // "released" until re-pressed (the journal/0032 quirk). Re-press what is
        // still held and clear its just_pressed edge, so walking resumes
        // seamlessly without re-firing a one-shot action (jump/edit).
        let held: Vec<KeyCode> = state.held.iter().copied().collect();
        for k in held {
            keys.press(k);
            keys.clear_just_pressed(k);
        }
        state.held.clear();
        state.hint.clear();
    }
}

fn history_prev(state: &mut ConsoleState) {
    if state.history.is_empty() {
        return;
    }
    let pos = match state.history_pos {
        None => state.history.len() - 1,
        Some(0) => 0,
        Some(p) => p - 1,
    };
    state.history_pos = Some(pos);
    state.input = state.history[pos].clone();
}

fn history_next(state: &mut ConsoleState) {
    match state.history_pos {
        Some(p) if p + 1 < state.history.len() => {
            state.history_pos = Some(p + 1);
            state.input = state.history[p + 1].clone();
        }
        Some(_) => {
            state.history_pos = None;
            state.input.clear();
        }
        None => {}
    }
}

fn complete_input(state: &mut ConsoleState, cmds: &[ConsoleCommand], world: Option<&HostWorld>) {
    // In value position (`key=<partial>`), ask the command's registry completer
    // for legal values (block names, live character names, postures, …).
    // Everywhere else — command names, param keys — it is the pure schema walk.
    let comp = if let Some((vstart, key, prefix)) = core::value_context(&state.input) {
        let head = state.input.split_whitespace().next().unwrap_or("");
        match core::find(cmds, head) {
            Some(cmd) => {
                let values = completer_values(cmd, world, key, prefix);
                core::value_completion(vstart, prefix, &values)
            }
            None => core::complete(cmds, &state.input),
        }
    } else {
        core::complete(cmds, &state.input)
    };
    apply_completion(state, comp);
}

/// Apply a completion to the input line: a lone candidate replaces the token;
/// several fill the common prefix (when it extends the token) and list into the
/// scrollback with their type/description display.
fn apply_completion(state: &mut ConsoleState, comp: Completion) {
    if comp.candidates.is_empty() {
        return;
    }
    if comp.candidates.len() == 1 {
        state.input.truncate(comp.token_start);
        state.input.push_str(&comp.candidates[0].text);
        return;
    }
    let current_len = state.input.len() - comp.token_start;
    if comp.common.len() > current_len {
        state.input.truncate(comp.token_start);
        state.input.push_str(&comp.common);
    }
    // Listing pushes output; pin to the bottom so the candidates are visible.
    state.scroll = 0;
    state.push_line("  candidates:");
    for Candidate { display, .. } in comp.candidates.iter().take(MAX_LISTED_CANDIDATES) {
        state.push_line(format!("    {display}"));
    }
    if comp.candidates.len() > MAX_LISTED_CANDIDATES {
        state.push_line(format!(
            "    … {} more",
            comp.candidates.len() - MAX_LISTED_CANDIDATES
        ));
    }
}

/// Value candidates for a command's parameter, from the registry completion
/// hook (docs/API.md decision 6). READ-ONLY by construction: `Static` sources
/// need no world; `World` sources take a shared `&HostWorld` and call only
/// `&self` accessors (`block_names`, `characters`, `content_classes`), so
/// completion can never tick, submit, or otherwise mutate the sim. Client-shell
/// tools (no registry id) and commands without a source yield nothing; a
/// missing world degrades a `World` source to nothing rather than stalling.
fn completer_values(
    cmd: &ConsoleCommand,
    world: Option<&HostWorld>,
    param: &str,
    prefix: &str,
) -> Vec<String> {
    let Some(id) = &cmd.id else {
        return Vec::new();
    };
    let Some(spec) = schema::spec(id) else {
        return Vec::new();
    };
    match &spec.completions {
        None => Vec::new(),
        Some(Completer::Static(f)) => f(param, prefix),
        Some(Completer::World(f)) => match world {
            Some(w) => f(w, param, prefix),
            None => Vec::new(),
        },
    }
}

/// Recompute the live hint area from the current line: the recognized command's
/// signature, plus a per-arg help line (type, required, description, example
/// values) for the parameter the caret sits on.
fn recompute_hint(state: &mut ConsoleState, cmds: &[ConsoleCommand], world: Option<&HostWorld>) {
    state.hint.clear();
    let head = state
        .input
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    if head.is_empty() {
        return;
    }
    if head == "help" {
        state
            .hint
            .push("help <command>  —  full reference card (params + example)".to_string());
        return;
    }
    let Some(cmd) = core::find(cmds, &head) else {
        return;
    };
    let active = core::active_arg_key(&state.input).map(str::to_string);
    state.hint.push(core::signature(cmd));
    let detail = active.and_then(|key| {
        core::describe_param(cmd, &key).map(|param| {
            let examples: Vec<String> = completer_values(cmd, world, &key, "")
                .into_iter()
                .take(MAX_HINT_EXAMPLES)
                .collect();
            core::arg_help_line(&param, &examples)
        })
    });
    if let Some(line) = detail {
        state.hint.push(line);
    }
}

/// Parse, assemble, and dispatch the current input line through the shared
/// bridge. `help` / `help <command>` render locally; everything else routes to
/// the one door.
fn submit(
    state: &mut ConsoleState,
    cmds: &[ConsoleCommand],
    bridge: &mpsc::UnboundedSender<BridgeRequest>,
) {
    let line = state.input.trim().to_string();
    state.input.clear();
    state.history_pos = None;
    // A fresh submission pins the view to the latest output.
    state.scroll = 0;
    if line.is_empty() {
        return;
    }
    state.push_line(format!("> {line}"));
    state.history.push(line.clone());

    // `help` takes a bare command name, not key=value — handle it before the
    // key=value parser (which would reject a bare token).
    let mut words = line.split_whitespace();
    let head = words.next().unwrap_or_default();
    if head == "help" {
        match words.next() {
            None => {
                for l in core::help_overview(cmds) {
                    state.push_line(l);
                }
            }
            Some(name) => match core::find(cmds, name) {
                Some(cmd) => {
                    for l in core::help_command(cmd) {
                        state.push_line(l);
                    }
                }
                None => state.push_err(format!("unknown command `{name}` (try `help`)")),
            },
        }
        return;
    }

    let parsed = match core::parse_line(&line) {
        None => return,
        Some(Ok(p)) => p,
        Some(Err(e)) => {
            state.push_err(e);
            return;
        }
    };
    let Some(cmd) = core::find(cmds, &parsed.command) else {
        state.push_err(format!("unknown command `{}` (try `help`)", parsed.command));
        return;
    };
    let args = match core::assemble(cmd, &parsed.args) {
        Ok(v) => v,
        Err(e) => {
            state.push_err(e);
            return;
        }
    };
    // The one door: same mapping the MCP server uses, same channel drain_bridge
    // drains. We only reach here for commands in our table, so `Ok(None)`
    // (truly unknown) should not happen — report it if it ever does.
    match bridge_request_for(&cmd.name, &args) {
        Ok(Some((request, rx))) => {
            if cmd.class == CommandClass::Command {
                state.push_line(format!("  submitted {} (awaiting receipt)", cmd.name));
            }
            if bridge.send(request).is_err() {
                state.push_err("console bridge closed (client shutting down)");
                return;
            }
            state.pending.push(Pending {
                label: cmd.name.clone(),
                rx,
            });
        }
        Ok(None) => state.push_err(format!("`{}` is not dispatchable", cmd.name)),
        Err(e) => state.push_err(e),
    }
}
