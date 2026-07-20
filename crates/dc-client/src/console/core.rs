//! Console core: the UI-agnostic logic of the dev console (docs/API.md
//! principle 5, "Reflectable"). Everything here is a pure function of the
//! command table and a line of text — no Bevy, no rendering, no world access —
//! so it is exhaustively unit-testable and cannot drift from the registry.
//!
//! The command table ([`ConsoleCommand`]) is built once from
//! `dc_api::schema::registry()` plus the three hand-authored client-shell tools
//! (see the parent module). This module never names an individual command: it
//! tokenizes a line, expands flat dotted keys into the nested JSON the command's
//! `payload_schema()` describes, computes completions, and renders help — all by
//! walking that schema. A command added to the registry tomorrow gains
//! autocomplete and help here with zero code changes.

use serde_json::{Map, Value};

/// How a surfaced command reaches the world, for display and (in the parent
/// module) dispatch routing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandClass {
    /// A dc-api command (mutates at a tick boundary).
    Command,
    /// A dc-api query (reads the last completed tick).
    Query,
    /// A client-shell tool (screenshot / pose), outside the registry.
    Client,
}

impl CommandClass {
    pub fn label(self) -> &'static str {
        match self {
            CommandClass::Command => "command",
            CommandClass::Query => "query",
            CommandClass::Client => "client",
        }
    }
}

/// One command surfaced in the console. Built from a registry [`CommandSpec`] or
/// a client-shell tool; the console only ever sees this uniform shape.
#[derive(Clone)]
pub struct ConsoleCommand {
    /// The name typed at the console — the MCP tool name for registry commands
    /// (`world_set_block`), the tool name for client tools (`client_screenshot`).
    pub name: String,
    /// The dc-api command id (`dc:world/set_block`); `None` for client tools.
    pub id: Option<String>,
    /// One-line summary (the spec `doc` / tool description, first line).
    pub summary: String,
    /// Human-readable capability requirement (registry commands only).
    pub capability: Option<String>,
    pub class: CommandClass,
    /// The JSON schema (draft-07-ish object) for this command's arguments.
    pub schema: Value,
}

impl ConsoleCommand {
    /// Build the table from the dc-api schema registry. The parent module
    /// appends the client-shell tools with [`ConsoleCommand::client`].
    pub fn from_registry() -> Vec<ConsoleCommand> {
        dc_api::schema::registry()
            .iter()
            .map(|spec| ConsoleCommand {
                name: dc_api::schema::mcp_tool_name(spec.id),
                id: Some(spec.id.to_string()),
                summary: collapse_ws(spec.doc),
                capability: Some(spec.capability.to_string()),
                class: match spec.kind {
                    dc_api::schema::CommandKind::Command => CommandClass::Command,
                    dc_api::schema::CommandKind::Query => CommandClass::Query,
                },
                schema: (spec.payload_schema)(),
            })
            .collect()
    }

    /// Build a client-shell tool entry (screenshot / pose) from its name,
    /// description, and argument schema.
    pub fn client(name: impl Into<String>, description: &str, schema: Value) -> ConsoleCommand {
        ConsoleCommand {
            name: name.into(),
            id: None,
            summary: collapse_ws(description),
            capability: None,
            class: CommandClass::Client,
            schema,
        }
    }
}

/// Collapse runs of whitespace (the registry docs wrap across source lines) into
/// single spaces so a summary renders on one console line.
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Look a command up by exact name.
pub fn find<'a>(cmds: &'a [ConsoleCommand], name: &str) -> Option<&'a ConsoleCommand> {
    cmds.iter().find(|c| c.name == name)
}

// ------------------------------------------------------------- parsing ----

/// A parsed command line: the command name and its `key=value` arguments (keys
/// are still flat dotted paths; expansion into nested JSON is a later step).
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedLine {
    pub command: String,
    pub args: Vec<(String, String)>,
}

/// Tokenize and parse a line. Returns `None` for a blank line. Arguments are
/// whitespace-separated `key=value` tokens; a token without `=` is reported as a
/// bad argument. Values contain no spaces in v0 (block names, numbers, and
/// coordinates never need them).
pub fn parse_line(line: &str) -> Option<Result<ParsedLine, String>> {
    let mut tokens = line.split_whitespace();
    let command = tokens.next()?.to_string();
    let mut args = Vec::new();
    for tok in tokens {
        match tok.split_once('=') {
            Some((k, v)) if !k.is_empty() => args.push((k.to_string(), v.to_string())),
            _ => {
                return Some(Err(format!(
                    "bad argument `{tok}`: expected key=value (e.g. pos.x=10)"
                )));
            }
        }
    }
    Some(Ok(ParsedLine { command, args }))
}

// ----------------------------------------------------- schema walking ----

/// One argument leaf discovered by walking a command's schema: its flat dotted
/// path, JSON type name, whether it is required, and its description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamInfo {
    pub path: String,
    pub type_name: String,
    pub required: bool,
    pub description: String,
}

/// Every settable leaf of a command's schema, as flat dotted paths (`pos.x`,
/// `block`, …). Object subtrees recurse; arrays and schemaless nodes are leaves
/// (the flat syntax passes them a raw JSON literal — see [`assemble`]).
pub fn param_paths(cmd: &ConsoleCommand) -> Vec<ParamInfo> {
    let mut out = Vec::new();
    walk_params(&cmd.schema, "", true, &mut out);
    out
}

fn walk_params(schema: &Value, prefix: &str, ancestors_required: bool, out: &mut Vec<ParamInfo>) {
    let props = schema.get("properties").and_then(Value::as_object);
    let Some(props) = props else {
        return;
    };
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    for (name, node) in props {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        let req = ancestors_required && required.contains(&name.as_str());
        let is_object = node.get("type").and_then(Value::as_str) == Some("object")
            && node.get("properties").is_some();
        if is_object {
            walk_params(node, &path, req, out);
        } else {
            out.push(ParamInfo {
                path,
                type_name: type_name_of(node),
                required: req,
                description: node
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }
}

fn type_name_of(node: &Value) -> String {
    node.get("type")
        .and_then(Value::as_str)
        .unwrap_or("any")
        .to_string()
}

/// Resolve the schema node reached by a dotted path from a root schema, together
/// with whether every step stayed inside a known `properties` map. Returns
/// `Ok(node)` for a known path, `Err(msg)` when a segment names an unknown
/// property of a closed object.
fn resolve_node<'a>(root: &'a Value, path: &str, cmd: &str) -> Result<Option<&'a Value>, String> {
    let mut node = root;
    for (i, seg) in path.split('.').enumerate() {
        let props = node.get("properties").and_then(Value::as_object);
        match props.and_then(|p| p.get(seg)) {
            Some(child) => node = child,
            None => {
                // Unknown segment. If this object is open (additionalProperties
                // not false, or it has no declared properties at all — a loose
                // node like a body-plan segment array), accept it as a raw
                // passthrough. Otherwise it is a genuine unknown parameter.
                let closed = node.get("additionalProperties") == Some(&Value::Bool(false));
                let has_props = props.is_some();
                if has_props && closed {
                    let so_far: Vec<&str> = path.split('.').take(i + 1).collect();
                    return Err(format!(
                        "unknown parameter `{}` for `{cmd}` (try `help {cmd}`)",
                        so_far.join(".")
                    ));
                }
                return Ok(None);
            }
        }
    }
    Ok(Some(node))
}

// ------------------------------------------------------ arg assembly ----

/// Expand flat `key=value` arguments into the nested JSON object the command's
/// schema describes, coercing each value to the schema's type. Unknown keys of a
/// closed object are rejected here; type errors are left for the host's
/// `decode_json` so the authoritative message reaches the console.
pub fn assemble(cmd: &ConsoleCommand, args: &[(String, String)]) -> Result<Value, String> {
    let mut root = Map::new();
    for (key, raw) in args {
        let node = resolve_node(&cmd.schema, key, &cmd.name)?;
        let value = coerce(node, raw);
        insert_path(&mut root, key, value)?;
    }
    Ok(Value::Object(root))
}

/// Coerce a raw string to JSON per the resolved schema node's `type`. Numbers,
/// integers and booleans that parse become JSON scalars; a value that does not
/// parse is left as a string so the host's decoder emits the definitive type
/// error. Array / object / unknown leaves accept a raw JSON literal (so the flat
/// syntax can still reach `segments=[...]`), falling back to a string.
fn coerce(node: Option<&Value>, raw: &str) -> Value {
    let ty = node.and_then(|n| n.get("type")).and_then(Value::as_str);
    match ty {
        Some("integer") => raw
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or_else(|_| Value::String(raw.to_string())),
        Some("number") => raw
            .parse::<f64>()
            .map(Value::from)
            .unwrap_or_else(|_| Value::String(raw.to_string())),
        Some("boolean") => raw
            .parse::<bool>()
            .map(Value::from)
            .unwrap_or_else(|_| Value::String(raw.to_string())),
        Some("string") => Value::String(raw.to_string()),
        // Arrays, loose objects, or unknown: allow a raw JSON literal escape
        // hatch, else treat as a plain string.
        _ => serde_json::from_str::<Value>(raw).unwrap_or_else(|_| Value::String(raw.to_string())),
    }
}

/// Insert `value` at a dotted `path` into `root`, creating intermediate objects.
/// Errors if a path segment collides with a non-object already placed there.
fn insert_path(root: &mut Map<String, Value>, path: &str, value: Value) -> Result<(), String> {
    let segments: Vec<&str> = path.split('.').collect();
    let mut cursor = root;
    for seg in &segments[..segments.len() - 1] {
        let entry = cursor
            .entry((*seg).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        cursor = entry.as_object_mut().ok_or_else(|| {
            format!("argument `{path}` conflicts with an earlier value at `{seg}`")
        })?;
    }
    cursor.insert(segments[segments.len() - 1].to_string(), value);
    Ok(())
}

// -------------------------------------------------------- completion ----

/// One completion candidate: the text that replaces the current token, and a
/// richer display string (for the multi-candidate listing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    pub display: String,
}

/// The result of completing the token under the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Byte offset in the input where the current token begins (what to replace).
    pub token_start: usize,
    /// The longest common prefix of all candidate `text`s (what to auto-fill).
    pub common: String,
    pub candidates: Vec<Candidate>,
}

/// The byte offset where the final whitespace-delimited token begins, and that
/// token. A trailing space means the current token is empty and starts at the
/// end (the user is starting a fresh argument).
fn current_token(input: &str) -> (usize, &str) {
    match input.rfind(char::is_whitespace) {
        Some(i) => (i + 1, &input[i + 1..]),
        None => (0, input),
    }
}

/// Compute completions for the token under the cursor at the end of `input`.
/// Command position → command names by prefix; after a command → its schema's
/// param keys by prefix; after `help` → command names.
pub fn complete(cmds: &[ConsoleCommand], input: &str) -> Completion {
    let (start, token) = current_token(input);
    let leading = &input[..start];
    let first = leading.split_whitespace().next();

    let candidates = match first {
        // Command position (nothing but the token so far).
        None => command_candidates(cmds, token),
        // `help <cmd>`: complete a command name as the argument.
        Some("help") if leading.split_whitespace().count() == 1 => command_candidates(cmds, token),
        // After a known command: complete param keys (only before any `=`).
        Some(cmd_name) => match find(cmds, cmd_name) {
            Some(cmd) if !token.contains('=') => param_candidates(cmd, token),
            _ => Vec::new(),
        },
    };

    let common = longest_common_prefix(candidates.iter().map(|c| c.text.as_str()));
    Completion {
        token_start: start,
        common,
        candidates,
    }
}

fn command_candidates(cmds: &[ConsoleCommand], prefix: &str) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = cmds
        .iter()
        .filter(|c| c.name.starts_with(prefix))
        .map(|c| Candidate {
            text: c.name.clone(),
            display: format!("{}  [{}]  {}", c.name, c.class.label(), c.summary),
        })
        .collect();
    out.sort_by(|a, b| a.text.cmp(&b.text));
    out
}

fn param_candidates(cmd: &ConsoleCommand, prefix: &str) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = param_paths(cmd)
        .into_iter()
        .filter(|p| p.path.starts_with(prefix))
        .map(|p| {
            let req = if p.required { " (required)" } else { "" };
            Candidate {
                // Completing a key leaves the cursor ready for its value.
                text: format!("{}=", p.path),
                display: format!("{}  {}{}  {}", p.path, p.type_name, req, p.description),
            }
        })
        .collect();
    out.sort_by(|a, b| a.text.cmp(&b.text));
    out
}

fn longest_common_prefix<'a>(mut iter: impl Iterator<Item = &'a str>) -> String {
    let Some(first) = iter.next() else {
        return String::new();
    };
    let mut prefix = first.to_string();
    for s in iter {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return prefix;
            }
        }
    }
    prefix
}

// -------------------------------------------------------------- help ----

/// The overview: every surfaced command, one per line, name + class + summary.
pub fn help_overview(cmds: &[ConsoleCommand]) -> Vec<String> {
    let width = cmds.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let mut lines = vec![format!(
        "{} commands. `help <command>` for parameters. \
         Syntax: <command> key=value (dotted keys, e.g. pos.x=10).",
        cmds.len()
    )];
    let mut sorted: Vec<&ConsoleCommand> = cmds.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    for c in sorted {
        lines.push(format!(
            "  {:width$}  [{:<7}]  {}",
            c.name,
            c.class.label(),
            c.summary,
            width = width
        ));
    }
    lines
}

/// The detail for one command: id/class/capability, the doc, and every
/// parameter with type, required-ness, and description.
pub fn help_command(cmd: &ConsoleCommand) -> Vec<String> {
    let mut lines = Vec::new();
    let id = cmd
        .id
        .clone()
        .unwrap_or_else(|| "(client tool)".to_string());
    lines.push(format!("{}  [{}]  (id: {id})", cmd.name, cmd.class.label()));
    if let Some(cap) = &cmd.capability {
        lines.push(format!("  requires: {cap}"));
    }
    lines.push(format!("  {}", cmd.summary));
    let params = param_paths(cmd);
    if params.is_empty() {
        lines.push("  parameters: (none)".to_string());
        return lines;
    }
    lines.push("  parameters:".to_string());
    let width = params.iter().map(|p| p.path.len()).max().unwrap_or(0);
    for p in &params {
        let req = if p.required { "required" } else { "optional" };
        lines.push(format!(
            "    {:width$}  {:<8} {:<8}  {}",
            p.path,
            p.type_name,
            req,
            p.description,
            width = width
        ));
    }
    // A schema-driven example invocation, so a reader sees a valid line to copy,
    // not just a parameter list. Description-based placeholders (`e.g. dc:stone`)
    // are deterministic; the Bevy shell can enrich with live completer values.
    lines.push(String::new());
    lines.push(format!("  example: {}", example_invocation(cmd, |_| None)));
    lines
}

// ------------------------------------------------ signatures & hints ----

/// A short display type for a schema `type` name: `integer`→`int`,
/// `number`→`num`, `boolean`→`bool`, `string`→`string`, everything else
/// (arrays, loose objects, unknown) →`json` (the raw-JSON escape hatch leaf).
pub fn short_type(type_name: &str) -> &'static str {
    match type_name {
        "integer" => "int",
        "number" => "num",
        "boolean" => "bool",
        "string" => "string",
        _ => "json",
    }
}

/// The one-line signature of a command, generated from its schema: every leaf
/// parameter as `key=<type>`, required params bare and optional ones bracketed
/// (`[key=<type>]`). This is the persistent "shape of args" the console shows
/// the moment a command name is recognized, so a first-time user sees the
/// whole form without prior knowledge. Pure function of the schema.
pub fn signature(cmd: &ConsoleCommand) -> String {
    let params = param_paths(cmd);
    if params.is_empty() {
        return format!("{}  (no arguments)", cmd.name);
    }
    let parts: Vec<String> = params
        .iter()
        .map(|p| {
            let body = format!("{}=<{}>", p.path, short_type(&p.type_name));
            if p.required {
                body
            } else {
                format!("[{body}]")
            }
        })
        .collect();
    format!("{}  {}", cmd.name, parts.join(" "))
}

/// The parameter key of the `key=value` token the caret currently sits in (the
/// caret is the end of `input`). `None` in command position or while a bare key
/// is still being typed (no `=` yet). Drives the per-arg hint area.
pub fn active_arg_key(input: &str) -> Option<&str> {
    let (_, token) = current_token(input);
    let (key, _) = token.split_once('=')?;
    (!key.is_empty()).then_some(key)
}

/// The [`ParamInfo`] for a command's dotted `key`, or `None` if the command has
/// no such leaf parameter.
pub fn describe_param(cmd: &ConsoleCommand, key: &str) -> Option<ParamInfo> {
    param_paths(cmd).into_iter().find(|p| p.path == key)
}

/// The per-arg help line for the parameter the caret sits on: name, type,
/// required-ness, description, and up to a few example values when a command's
/// completer supplied them (passed in — the world lives in the Bevy shell).
pub fn arg_help_line(param: &ParamInfo, examples: &[String]) -> String {
    let req = if param.required {
        "required"
    } else {
        "optional"
    };
    let mut line = format!("{} : {} ({req})", param.path, short_type(&param.type_name));
    if !param.description.is_empty() {
        line.push_str(" — ");
        line.push_str(&param.description);
    }
    if !examples.is_empty() {
        line.push_str("   e.g. ");
        line.push_str(&examples.join(", "));
    }
    line
}

/// Extract an `e.g. X` example token from a parameter description, if present
/// (the registry docs carry them: `"block name, e.g. dc:stone"`). Used to seed
/// a string placeholder in a generated example when no live completer value
/// exists.
fn hint_from_description(desc: &str) -> Option<String> {
    let idx = desc.find("e.g. ")? + "e.g. ".len();
    let token = desc[idx..].split([',', ';', ')', ' ', '\n']).next()?.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// A schema-appropriate placeholder value for one parameter: ints/nums→`0`,
/// bools→`false`, arrays/objects→`[]`, strings→an `e.g.` hint from the
/// description if present, else `text`.
fn placeholder_value(p: &ParamInfo) -> String {
    match p.type_name.as_str() {
        "integer" | "number" => "0".to_string(),
        "boolean" => "false".to_string(),
        "string" => hint_from_description(&p.description).unwrap_or_else(|| "text".to_string()),
        _ => "[]".to_string(),
    }
}

/// A copy-pasteable example invocation covering every REQUIRED parameter with
/// schema-appropriate placeholders (nested params stay dotted: `pos.x=0`).
/// `example_for(path)` may override a placeholder with a live value (e.g. a real
/// block name from the completer); returning `None` falls back to the schema
/// placeholder. Pure — the impurity (asking the world) lives in the closure.
pub fn example_invocation(
    cmd: &ConsoleCommand,
    example_for: impl Fn(&str) -> Option<String>,
) -> String {
    let parts: Vec<String> = param_paths(cmd)
        .into_iter()
        .filter(|p| p.required)
        .map(|p| {
            let v = example_for(&p.path).unwrap_or_else(|| placeholder_value(&p));
            format!("{}={}", p.path, v)
        })
        .collect();
    if parts.is_empty() {
        cmd.name.clone()
    } else {
        format!("{}  {}", cmd.name, parts.join(" "))
    }
}

// ------------------------------------------------- value completion ----

/// When the caret sits in the value half of a `key=value` token, the byte
/// offset where the value begins, the key, and the partial value typed so far.
/// `None` in command or bare-key position. The console asks the command's
/// [`dc_api::schema::Completer`] for candidate values for `key` and feeds them
/// to [`value_completion`].
pub fn value_context(input: &str) -> Option<(usize, &str, &str)> {
    let (start, token) = current_token(input);
    let eq = token.find('=')?;
    let key = &token[..eq];
    if key.is_empty() {
        return None;
    }
    let value = &token[eq + 1..];
    Some((start + eq + 1, key, value))
}

/// Build a [`Completion`] over `values` (the candidate set a completer returned
/// for this parameter), replacing only the partial value after `=` and filtering
/// by its prefix. `value_start` is the byte offset [`value_context`] reported.
pub fn value_completion(value_start: usize, prefix: &str, values: &[String]) -> Completion {
    let mut candidates: Vec<Candidate> = values
        .iter()
        .filter(|v| v.starts_with(prefix))
        .map(|v| Candidate {
            text: v.clone(),
            display: v.clone(),
        })
        .collect();
    candidates.sort_by(|a, b| a.text.cmp(&b.text));
    candidates.dedup_by(|a, b| a.text == b.text);
    let common = longest_common_prefix(candidates.iter().map(|c| c.text.as_str()));
    Completion {
        token_start: value_start,
        common,
        candidates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn table() -> Vec<ConsoleCommand> {
        let mut cmds = ConsoleCommand::from_registry();
        // A synthetic client tool mirroring client_player_pose_set's shape.
        cmds.push(ConsoleCommand::client(
            "client_player_pose_set",
            "Teleport and/or aim the player.",
            json!({
                "type": "object",
                "properties": {
                    "pos": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number", "description": "meters x"},
                            "y": {"type": "number", "description": "meters y"},
                            "z": {"type": "number", "description": "meters z"},
                        },
                        "required": ["x", "y", "z"],
                        "additionalProperties": false,
                    },
                    "surface": {"type": "boolean", "description": "clamp to surface"},
                },
                "required": [],
                "additionalProperties": false,
            }),
        ));
        cmds
    }

    #[test]
    fn registry_is_fully_surfaced() {
        let cmds = ConsoleCommand::from_registry();
        assert_eq!(cmds.len(), dc_api::schema::registry().len());
        // Every registry id maps to its MCP tool name and is present.
        assert!(find(&cmds, "world_set_block").is_some());
        assert!(find(&cmds, "world_get_block").is_some());
        assert!(find(&cmds, "character_pose").is_some());
    }

    #[test]
    fn parse_splits_command_and_kv() {
        let p = parse_line("world_set_block pos.x=10 pos.y=64 block=dc:stone")
            .unwrap()
            .unwrap();
        assert_eq!(p.command, "world_set_block");
        assert_eq!(
            p.args,
            vec![
                ("pos.x".to_string(), "10".to_string()),
                ("pos.y".to_string(), "64".to_string()),
                ("block".to_string(), "dc:stone".to_string()),
            ]
        );
    }

    #[test]
    fn parse_blank_is_none_and_bad_arg_errors() {
        assert!(parse_line("   ").is_none());
        let err = parse_line("world_set_block pos.x").unwrap().unwrap_err();
        assert!(err.contains("key=value"), "{err}");
    }

    #[test]
    fn assemble_expands_dotted_paths_and_coerces() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        let v = assemble(
            cmd,
            &[
                ("pos.x".into(), "10".into()),
                ("pos.y".into(), "64".into()),
                ("pos.z".into(), "-3".into()),
                ("block".into(), "dc:stone".into()),
            ],
        )
        .unwrap();
        assert_eq!(
            v,
            json!({"pos": {"x": 10, "y": 64, "z": -3}, "block": "dc:stone"})
        );
        // The integer coordinate is a JSON number, not a string.
        assert!(v["pos"]["x"].is_i64());
    }

    #[test]
    fn assemble_rejects_unknown_key_of_closed_object() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        let err = assemble(cmd, &[("pos.w".into(), "1".into())]).unwrap_err();
        assert!(err.contains("unknown parameter"), "{err}");
        let err = assemble(cmd, &[("bogus".into(), "1".into())]).unwrap_err();
        assert!(err.contains("unknown parameter"), "{err}");
    }

    #[test]
    fn assemble_client_tool_bool_and_nested() {
        let cmds = table();
        let cmd = find(&cmds, "client_player_pose_set").unwrap();
        let v = assemble(
            cmd,
            &[
                ("pos.x".into(), "1.5".into()),
                ("pos.y".into(), "64".into()),
                ("pos.z".into(), "2".into()),
                ("surface".into(), "true".into()),
            ],
        )
        .unwrap();
        assert_eq!(v["pos"]["x"], json!(1.5));
        assert_eq!(v["surface"], json!(true));
    }

    #[test]
    fn param_paths_flattens_nested_and_marks_required() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        let paths = param_paths(cmd);
        let names: Vec<&str> = paths.iter().map(|p| p.path.as_str()).collect();
        assert!(names.contains(&"pos.x"));
        assert!(names.contains(&"pos.y"));
        assert!(names.contains(&"pos.z"));
        assert!(names.contains(&"block"));
        let x = paths.iter().find(|p| p.path == "pos.x").unwrap();
        assert_eq!(x.type_name, "integer");
        assert!(x.required, "pos.x is required");
        assert_eq!(x.description, "voxel x");
    }

    #[test]
    fn complete_command_names_by_prefix() {
        let cmds = table();
        let c = complete(&cmds, "world_");
        assert!(c.candidates.iter().all(|x| x.text.starts_with("world_")));
        assert!(c.candidates.iter().any(|x| x.text == "world_set_block"));
        // Common prefix of all world_* commands starts with the typed prefix.
        assert!(c.common.starts_with("world_"));
    }

    #[test]
    fn complete_param_keys_after_command() {
        let cmds = table();
        let c = complete(&cmds, "world_set_block po");
        let texts: Vec<&str> = c.candidates.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["pos.x=", "pos.y=", "pos.z="]);
        assert_eq!(c.common, "pos.");
        // token_start points at the "po" token, not the command.
        assert_eq!(&"world_set_block po"[c.token_start..], "po");
    }

    #[test]
    fn complete_help_argument_is_command_names() {
        let cmds = table();
        let c = complete(&cmds, "help character_");
        assert!(
            c.candidates
                .iter()
                .all(|x| x.text.starts_with("character_"))
        );
    }

    #[test]
    fn no_value_completion_after_equals() {
        // Value-level completion is out of scope for v0.
        let cmds = table();
        let c = complete(&cmds, "world_set_block block=");
        assert!(c.candidates.is_empty());
    }

    #[test]
    fn help_overview_lists_every_command() {
        let cmds = table();
        let lines = help_overview(&cmds);
        // header + one line per command.
        assert_eq!(lines.len(), cmds.len() + 1);
        assert!(lines.iter().any(|l| l.contains("world_set_block")));
    }

    #[test]
    fn help_command_renders_params_types_and_requiredness() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        let lines = help_command(cmd);
        let joined = lines.join("\n");
        assert!(joined.contains("dc:world/set_block"));
        assert!(joined.contains("world.write"));
        assert!(joined.contains("pos.x"));
        assert!(joined.contains("integer"));
        assert!(joined.contains("required"));
        assert!(joined.contains("block name"));
    }

    #[test]
    fn help_command_handles_no_params() {
        // character_jump takes only `character` — but a truly empty one:
        // events_poll has params; use a synthetic empty client tool.
        let empty = ConsoleCommand::client(
            "client_player_pose_get",
            "Read the player pose.",
            json!({"type": "object", "properties": {}, "required": []}),
        );
        let lines = help_command(&empty);
        assert!(lines.iter().any(|l| l.contains("(none)")));
    }

    #[test]
    fn signature_marks_required_and_brackets_optional() {
        let cmds = table();
        // set_block: all params required (schema key order is alphabetical —
        // serde_json Map is a BTreeMap — so assert by token, not full string).
        let sig = signature(find(&cmds, "world_set_block").unwrap());
        assert!(sig.starts_with("world_set_block  "), "{sig}");
        for tok in [
            "pos.x=<int>",
            "pos.y=<int>",
            "pos.z=<int>",
            "block=<string>",
        ] {
            assert!(sig.contains(tok), "{sig} missing {tok}");
        }
        // Required params are bare (not bracketed).
        assert!(!sig.contains('['), "{sig} should have no optionals");
        // entity_query: both params optional → bracketed.
        let sig = signature(find(&cmds, "entity_query").unwrap());
        assert!(sig.contains("[kind=<string>]"), "{sig}");
        assert!(sig.contains("[volume.min.x=<int>]"), "{sig}");
    }

    #[test]
    fn active_arg_key_tracks_the_value_token() {
        // Command position / bare key → no active arg.
        assert_eq!(active_arg_key("world_set_block po"), None);
        assert_eq!(active_arg_key("world_set_block"), None);
        // Inside a key=value token → that key (empty value counts).
        assert_eq!(active_arg_key("world_set_block block="), Some("block"));
        assert_eq!(active_arg_key("world_set_block pos.x=1"), Some("pos.x"));
        // Only the token under the caret matters.
        assert_eq!(
            active_arg_key("world_set_block pos.x=1 block=dc:s"),
            Some("block")
        );
    }

    #[test]
    fn arg_help_line_carries_type_required_desc_and_examples() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        let param = describe_param(cmd, "block").unwrap();
        let line = arg_help_line(&param, &["dc:stone".into(), "dc:dirt".into()]);
        assert!(line.contains("block : string (required)"), "{line}");
        assert!(line.contains("block name"), "{line}");
        assert!(line.contains("e.g. dc:stone, dc:dirt"), "{line}");
        // No examples → no joined example tail (`dc:dirt` appears only in the
        // examples list, never in this param's own description).
        assert!(!arg_help_line(&param, &[]).contains("dc:dirt"));
    }

    #[test]
    fn example_invocation_uses_placeholders_and_description_hints() {
        let cmds = table();
        let cmd = find(&cmds, "world_set_block").unwrap();
        // Schema-only (no live values): ints→0, string block→its e.g. hint.
        let ex = example_invocation(cmd, |_| None);
        assert!(ex.starts_with("world_set_block  "), "{ex}");
        for tok in ["pos.x=0", "pos.y=0", "pos.z=0", "block=dc:stone"] {
            assert!(ex.contains(tok), "{ex} missing {tok}");
        }
        // A live completer value overrides the placeholder.
        let ex = example_invocation(cmd, |path| {
            (path == "block").then(|| "dc:granite".to_string())
        });
        assert!(ex.contains("block=dc:granite"), "{ex}");
    }

    #[test]
    fn value_context_finds_the_value_half() {
        assert_eq!(value_context("world_set_block po"), None);
        let (start, key, prefix) = value_context("world_set_block block=dc:s").unwrap();
        assert_eq!(key, "block");
        assert_eq!(prefix, "dc:s");
        assert_eq!(&"world_set_block block=dc:s"[start..], "dc:s");
        // Empty value: prefix is empty, start points just past '='.
        let (start, key, prefix) = value_context("world_set_block block=").unwrap();
        assert_eq!((key, prefix), ("block", ""));
        assert_eq!(&"world_set_block block="[start..], "");
    }

    #[test]
    fn value_completion_replaces_only_the_value_and_finds_common_prefix() {
        let input = "world_set_block block=dc:s";
        let (start, _key, prefix) = value_context(input).unwrap();
        let values = vec![
            "dc:stone".to_string(),
            "dc:sandstone".to_string(),
            "dc:dirt".to_string(),
        ];
        let comp = value_completion(start, prefix, &values);
        let texts: Vec<&str> = comp.candidates.iter().map(|c| c.text.as_str()).collect();
        // Prefix-filtered to the dc:s* names, sorted.
        assert_eq!(texts, vec!["dc:sandstone", "dc:stone"]);
        assert_eq!(comp.common, "dc:s");
        // Replacing from token_start keeps `block=` and swaps the value.
        let mut line = input.to_string();
        line.truncate(comp.token_start);
        line.push_str(&comp.candidates[1].text);
        assert_eq!(line, "world_set_block block=dc:stone");
    }

    /// Property-style: every command in the registry renders a signature, an
    /// example, and a per-arg help line for each of its params without
    /// panicking — the "generated from the registry" guarantee, now covering
    /// presentation as well as data.
    #[test]
    fn every_registry_command_renders_signature_example_and_arg_help() {
        for cmd in ConsoleCommand::from_registry() {
            let sig = signature(&cmd);
            assert!(sig.starts_with(&cmd.name), "signature: {sig}");
            let ex = example_invocation(&cmd, |_| None);
            assert!(ex.starts_with(&cmd.name), "example: {ex}");
            for p in param_paths(&cmd) {
                let line = arg_help_line(&p, &[]);
                assert!(line.starts_with(&p.path), "arg help: {line}");
                // help_command must also fold this param in without panic.
            }
            let help = help_command(&cmd);
            assert!(help.iter().any(|l| l.contains("example:")), "{}", cmd.name);
        }
    }
}
