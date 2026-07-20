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

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::mcp::{BridgeRequest, bridge_request_for, client_tools};
use core::{Candidate, CommandClass, ConsoleCommand};

/// How many scrollback lines to retain, and how many to show at once.
const SCROLLBACK_CAP: usize = 240;
const MAX_VISIBLE_LINES: usize = 24;
/// How many completion candidates to list before eliding the rest.
const MAX_LISTED_CANDIDATES: usize = 24;

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
    pending: Vec<Pending>,
}

impl ConsoleState {
    fn push_line(&mut self, line: impl Into<String>) {
        self.scrollback.push(line.into());
        if self.scrollback.len() > SCROLLBACK_CAP {
            let overflow = self.scrollback.len() - SCROLLBACK_CAP;
            self.scrollback.drain(..overflow);
        }
    }

    fn push_err(&mut self, msg: impl Into<String>) {
        self.push_line(format!("  ! {}", msg.into()));
    }
}

// ------------------------------------------------------- marker types ----

#[derive(Component)]
struct ConsoleUiRoot;
#[derive(Component)]
struct ConsoleScrollback;
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
            panel.spawn((
                ConsoleInputLine,
                Text::new("> "),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.65, 0.95, 0.70)),
            ));
        });
}

fn console_render(
    state: Res<ConsoleState>,
    mut root: Query<&mut Visibility, With<ConsoleUiRoot>>,
    mut scrollback: Query<&mut Text, With<ConsoleScrollback>>,
    mut input: Query<&mut Text, (With<ConsoleInputLine>, Without<ConsoleScrollback>)>,
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
        let start = state.scrollback.len().saturating_sub(MAX_VISIBLE_LINES);
        text.0 = state.scrollback[start..].join("\n");
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
pub fn console_input(
    mut reader: MessageReader<KeyboardInput>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut state: ResMut<ConsoleState>,
    cmds: Res<ConsoleCommands>,
    bridge: Res<ConsoleBridge>,
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
            // Release the cursor so mouse-look (which reads grab state, not the
            // button input we reset) stops while the console is up.
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        }
        return;
    }

    for e in &events {
        if e.state != ButtonState::Pressed {
            continue;
        }
        match e.key_code {
            KeyCode::Escape => state.open = false,
            KeyCode::Enter | KeyCode::NumpadEnter => submit(&mut state, &cmds.0, &bridge.0),
            KeyCode::Backspace => {
                state.input.pop();
            }
            KeyCode::Tab => complete_input(&mut state, &cmds.0),
            KeyCode::ArrowUp => history_prev(&mut state),
            KeyCode::ArrowDown => history_next(&mut state),
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

    // Swallow all input from the gameplay systems that run after us this frame.
    keys.reset_all();
    mouse.reset_all();
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

fn complete_input(state: &mut ConsoleState, cmds: &[ConsoleCommand]) {
    let comp = core::complete(cmds, &state.input);
    if comp.candidates.is_empty() {
        return;
    }
    if comp.candidates.len() == 1 {
        state.input.truncate(comp.token_start);
        state.input.push_str(&comp.candidates[0].text);
        return;
    }
    // Multiple: fill in the common prefix (if it extends the current token),
    // then list the candidates.
    let current_len = state.input.len() - comp.token_start;
    if comp.common.len() > current_len {
        state.input.truncate(comp.token_start);
        state.input.push_str(&comp.common);
    }
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
