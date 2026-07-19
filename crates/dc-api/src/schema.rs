//! Machine-readable schema registry (docs/API.md principle 5).
//!
//! Every command/query registers: id, kind, doc, a hand-rolled JSON schema for
//! its payload, and a JSON decoder into the typed [`Payload`]. Consumers are
//! GENERATED from this table — the MCP tool list is `registry()` mapped into
//! rmcp tools, never hand-written — so the surface cannot drift from its
//! consumers.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::payload::{Payload, ids};

/// Command (mutates at a tick boundary) vs query (reads the last completed
/// tick's state immediately).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CommandKind {
    Command,
    Query,
}

/// One registered command/query.
pub struct CommandSpec {
    pub id: &'static str,
    pub kind: CommandKind,
    pub doc: &'static str,
    /// Human-readable capability requirement, for tool docs.
    pub capability: &'static str,
    /// JSON schema (draft-07-ish subset) for the payload.
    pub payload_schema: fn() -> Value,
    /// Decode a JSON payload (e.g. MCP tool arguments) into the typed enum.
    pub decode_json: fn(&Value) -> Result<Payload, String>,
}

fn decode<T: DeserializeOwned>(v: &Value, wrap: fn(T) -> Payload) -> Result<Payload, String> {
    serde_json::from_value::<T>(v.clone())
        .map(wrap)
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------- schema builders --

fn s_int(doc: &str) -> Value {
    json!({"type": "integer", "description": doc})
}

fn s_num(doc: &str) -> Value {
    json!({"type": "number", "description": doc})
}

fn s_str(doc: &str) -> Value {
    json!({"type": "string", "description": doc})
}

fn s_vec3i(doc: &str) -> Value {
    json!({
        "type": "object",
        "description": doc,
        "properties": {"x": s_int("voxel x"), "y": s_int("voxel y"), "z": s_int("voxel z")},
        "required": ["x", "y", "z"],
        "additionalProperties": false,
    })
}

fn s_vec3f(doc: &str) -> Value {
    json!({
        "type": "object",
        "description": doc,
        "properties": {"x": s_num("meters x"), "y": s_num("meters y"), "z": s_num("meters z")},
        "required": ["x", "y", "z"],
        "additionalProperties": false,
    })
}

fn s_volume(doc: &str) -> Value {
    json!({
        "type": "object",
        "description": doc,
        "properties": {
            "min": s_vec3i("inclusive minimum corner"),
            "max": s_vec3i("inclusive maximum corner"),
        },
        "required": ["min", "max"],
        "additionalProperties": false,
    })
}

/// `props`: (name, schema, required).
fn s_obj(doc: &str, props: &[(&str, Value, bool)]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for (name, schema, req) in props {
        properties.insert((*name).to_string(), schema.clone());
        if *req {
            required.push(Value::String((*name).to_string()));
        }
    }
    json!({
        "type": "object",
        "description": doc,
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

// ------------------------------------------------------------- the table --

/// The v0 slice, in a stable order.
pub fn registry() -> &'static [CommandSpec] {
    static REGISTRY: &[CommandSpec] = &[
        CommandSpec {
            id: ids::WORLD_SET_BLOCK,
            kind: CommandKind::Command,
            doc: "Set one voxel to a named block state.",
            capability: "world.write covering pos",
            payload_schema: || {
                s_obj(
                    "set_block payload",
                    &[
                        ("pos", s_vec3i("voxel to set"), true),
                        ("block", s_str("block name, e.g. dc:stone"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SetBlock),
        },
        CommandSpec {
            id: ids::WORLD_GET_BLOCK,
            kind: CommandKind::Query,
            doc: "Read one voxel's block state.",
            capability: "world.read covering pos",
            payload_schema: || {
                s_obj(
                    "get_block payload",
                    &[("pos", s_vec3i("voxel to read"), true)],
                )
            },
            decode_json: |v| decode(v, Payload::GetBlock),
        },
        CommandSpec {
            id: ids::WORLD_FILL,
            kind: CommandKind::Command,
            doc: "Fill an inclusive box of voxels with a named block state.",
            capability: "world.write covering the box",
            payload_schema: || {
                s_obj(
                    "fill payload",
                    &[
                        ("min", s_vec3i("inclusive minimum corner"), true),
                        ("max", s_vec3i("inclusive maximum corner"), true),
                        ("block", s_str("block name, e.g. dc:stone"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::Fill),
        },
        CommandSpec {
            id: ids::WORLD_SCAN_REGION,
            kind: CommandKind::Query,
            doc: "Read an inclusive box of voxels as palette + indices \
                  (x-fastest, then z, then y).",
            capability: "world.read covering the box",
            payload_schema: || {
                s_obj(
                    "scan_region payload",
                    &[
                        ("min", s_vec3i("inclusive minimum corner"), true),
                        ("max", s_vec3i("inclusive maximum corner"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::ScanRegion),
        },
        CommandSpec {
            id: ids::ENTITY_SPAWN,
            kind: CommandKind::Command,
            doc: "Spawn a simple entity of a named kind at a position.",
            capability: "entity.spawn",
            payload_schema: || {
                s_obj(
                    "spawn payload",
                    &[
                        ("kind", s_str("entity kind, e.g. dc:deer"), true),
                        ("pos", s_vec3f("spawn position in meters"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::EntitySpawn),
        },
        CommandSpec {
            id: ids::ENTITY_QUERY,
            kind: CommandKind::Query,
            doc: "List entities, optionally filtered by volume and kind.",
            capability: "world.read covering the volume (unbounded read if no volume)",
            payload_schema: || {
                s_obj(
                    "entity query payload",
                    &[
                        ("volume", s_volume("only entities inside this box"), false),
                        ("kind", s_str("only entities of this kind"), false),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::EntityQuery),
        },
        CommandSpec {
            id: ids::REGISTRY_DEFINE_ITEM,
            kind: CommandKind::Command,
            doc: "Define a data-driven item. The name's namespace must be owned \
                  by the consumer's registry.define grant.",
            capability: "registry.define(namespace of name)",
            payload_schema: || {
                s_obj(
                    "define_item payload",
                    &[
                        (
                            "name",
                            s_str("namespaced item name, e.g. demo:builder_wand"),
                            true,
                        ),
                        ("display_name", s_str("human-readable name"), true),
                        ("description", s_str("optional flavor/doc text"), false),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::DefineItem),
        },
        CommandSpec {
            id: ids::EVENTS_SUBSCRIBE,
            kind: CommandKind::Command,
            doc: "Open an event subscription. Returns the subscription id in \
                  the receipt's effects.",
            capability: "events.subscribe",
            payload_schema: || {
                s_obj(
                    "subscribe payload",
                    &[
                        (
                            "kinds",
                            json!({
                                "type": "array",
                                "description": "event kinds to receive (empty = all)",
                                "items": {"type": "string", "enum": ["BlockChanged", "EntitySpawned", "ItemDefined"]},
                            }),
                            false,
                        ),
                        (
                            "volume",
                            s_volume("only positioned events inside this box"),
                            false,
                        ),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::EventsSubscribe),
        },
        CommandSpec {
            id: ids::EVENTS_POLL,
            kind: CommandKind::Query,
            doc: "Drain queued events from a subscription you own.",
            capability: "events.subscribe (and subscription ownership)",
            payload_schema: || {
                s_obj(
                    "poll payload",
                    &[
                        (
                            "subscription",
                            s_int("subscription id from subscribe"),
                            true,
                        ),
                        ("max", s_int("max events to return (default 256)"), false),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::EventsPoll),
        },
        CommandSpec {
            id: ids::CHARACTER_SPAWN,
            kind: CommandKind::Command,
            doc: "Spawn a persistent named character with a physical body at a \
                  position (feet, world meters). It exists in the world, falls \
                  and collides, and is driven by whoever holds control of it.",
            capability: "entity.spawn (dev grant; character sessions spawn via \
                         their attach flow)",
            payload_schema: || {
                s_obj(
                    "spawn_character payload",
                    &[
                        (
                            "name",
                            s_str("bare slug name ([a-z0-9_-], max 64), e.g. scout"),
                            true,
                        ),
                        ("pos", s_vec3f("feet position in world meters"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SpawnCharacter),
        },
        CommandSpec {
            id: ids::CHARACTER_SET_MOVE_INTENT,
            kind: CommandKind::Command,
            doc: "Set a character's horizontal movement intent: world-space \
                  direction (dx, dz — normalized; zero = stop) and a fraction \
                  of full walk speed. Persists until countermanded; the body \
                  integrates with collision and gravity every tick.",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "set_move_intent payload",
                    &[
                        ("character", s_str("character name"), true),
                        ("dx", s_num("world-space X direction component"), true),
                        ("dz", s_num("world-space Z direction component"), true),
                        ("speed", s_num("fraction of full walk speed, 0..1"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SetMoveIntent),
        },
        CommandSpec {
            id: ids::CHARACTER_SET_LOOK,
            kind: CommandKind::Command,
            doc: "Aim a character's gaze: yaw and pitch in radians \
                  (yaw 0 = -Z; NEGATIVE pitch looks down, clamped to ±1.55).",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "set_look payload",
                    &[
                        ("character", s_str("character name"), true),
                        ("yaw", s_num("radians, 0 = -Z"), true),
                        (
                            "pitch",
                            s_num("radians, NEGATIVE looks down, clamped to ±1.55"),
                            true,
                        ),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SetLook),
        },
        CommandSpec {
            id: ids::CHARACTER_JUMP,
            kind: CommandKind::Command,
            doc: "Request a jump; fires at the next tick if the character is \
                  on the ground then (dropped otherwise).",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "jump payload",
                    &[("character", s_str("character name"), true)],
                )
            },
            decode_json: |v| decode(v, Payload::Jump),
        },
        CommandSpec {
            id: ids::CHARACTER_POSE,
            kind: CommandKind::Query,
            doc: "The character's own proprioception: feet position (meters), \
                  velocity, yaw/pitch, on_ground, and eye_in_solid (true = its \
                  eyes are buried; senses from here see the inside of terrain).",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "pose payload",
                    &[("character", s_str("character name"), true)],
                )
            },
            decode_json: |v| decode(v, Payload::CharacterPose),
        },
        CommandSpec {
            id: ids::CHARACTER_SENSE_RAYCAST,
            kind: CommandKind::Query,
            doc: "Cast the character's gaze from its eyes along its look \
                  direction (or a given direction): first solid voxel with \
                  block name, entry face, and distance. Max range 50 m.",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "sense_raycast payload",
                    &[
                        ("character", s_str("character name"), true),
                        (
                            "dir",
                            s_vec3f("direction override (default: current gaze)"),
                            false,
                        ),
                        (
                            "max_distance_m",
                            s_num("max range in meters, capped at 50"),
                            false,
                        ),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SenseRaycast),
        },
        CommandSpec {
            id: ids::CHARACTER_SENSE_SURROUNDINGS,
            kind: CommandKind::Query,
            doc: "The character's near perception: scan the block volume in a \
                  cube of `radius` voxels (max 16) around its feet, as \
                  palette + indices (x-fastest, then z, then y).",
            capability: "character.control(character)",
            payload_schema: || {
                s_obj(
                    "sense_surroundings payload",
                    &[
                        ("character", s_str("character name"), true),
                        ("radius", s_int("cube half-extent in voxels, 0..=16"), true),
                    ],
                )
            },
            decode_json: |v| decode(v, Payload::SenseSurroundings),
        },
    ];
    REGISTRY
}

/// Look a spec up by command id.
pub fn spec(id: &str) -> Option<&'static CommandSpec> {
    registry().iter().find(|s| s.id == id)
}

/// MCP tool name for a command id: MCP tool names must match
/// `[a-zA-Z0-9_-]+`, so `dc:world/set_block` becomes `world_set_block`
/// (the `dc:` namespace is implied by the server).
pub fn mcp_tool_name(id: &str) -> String {
    id.strip_prefix("dc:").unwrap_or(id).replace('/', "_")
}

/// Reverse mapping from an MCP tool name to the command id.
pub fn id_for_mcp_tool(name: &str) -> Option<&'static str> {
    registry()
        .iter()
        .map(|s| s.id)
        .find(|id| mcp_tool_name(id) == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::{self, Vec3i, Volume};

    /// Every Payload variant must be constructible from the registry's JSON
    /// decoder — i.e. the registry is complete.
    #[test]
    fn registry_covers_every_payload_variant() {
        let samples: Vec<Payload> = vec![
            Payload::SetBlock(payload::SetBlock {
                pos: Vec3i::new(1, 2, 3),
                block: "dc:stone".into(),
            }),
            Payload::GetBlock(payload::GetBlock {
                pos: Vec3i::new(1, 2, 3),
            }),
            Payload::Fill(payload::Fill {
                min: Vec3i::new(0, 0, 0),
                max: Vec3i::new(1, 1, 1),
                block: "dc:dirt".into(),
            }),
            Payload::ScanRegion(payload::ScanRegion {
                min: Vec3i::new(0, 0, 0),
                max: Vec3i::new(1, 1, 1),
            }),
            Payload::EntitySpawn(payload::EntitySpawn {
                kind: "dc:deer".into(),
                pos: crate::payload::Vec3f::new(0.5, 1.0, 0.5),
            }),
            Payload::EntityQuery(payload::EntityQuery {
                volume: Some(Volume::new(Vec3i::new(0, 0, 0), Vec3i::new(4, 4, 4))),
                kind: None,
            }),
            Payload::DefineItem(payload::DefineItem {
                name: "demo:thing".into(),
                display_name: "Thing".into(),
                description: None,
            }),
            Payload::EventsSubscribe(payload::EventsSubscribe {
                kinds: vec![crate::event::EventKind::BlockChanged],
                volume: None,
            }),
            Payload::EventsPoll(payload::EventsPoll {
                subscription: 1,
                max: None,
            }),
            Payload::SpawnCharacter(payload::SpawnCharacter {
                name: "scout".into(),
                pos: crate::payload::Vec3f::new(1.0, 2.0, 3.0),
            }),
            Payload::SetMoveIntent(payload::SetMoveIntent {
                character: "scout".into(),
                dx: 1.0,
                dz: -0.5,
                speed: 0.8,
            }),
            Payload::SetLook(payload::SetLook {
                character: "scout".into(),
                yaw: 0.5,
                pitch: -0.2,
            }),
            Payload::Jump(payload::Jump {
                character: "scout".into(),
            }),
            Payload::CharacterPose(payload::CharacterPose {
                character: "scout".into(),
            }),
            Payload::SenseRaycast(payload::SenseRaycast {
                character: "scout".into(),
                dir: None,
                max_distance_m: Some(20.0),
            }),
            Payload::SenseSurroundings(payload::SenseSurroundings {
                character: "scout".into(),
                radius: 8,
            }),
        ];
        assert_eq!(
            samples.len(),
            registry().len(),
            "sample list and registry must cover the same surface"
        );
        for p in samples {
            let id = p.command_id();
            let spec = spec(id).unwrap_or_else(|| panic!("{id} missing from registry"));
            // Round-trip the payload through JSON via the spec's decoder.
            // Payload is externally tagged; the tool boundary carries the
            // *inner* payload object, so extract it.
            let outer = serde_json::to_value(&p).expect("serialize");
            let inner = outer
                .as_object()
                .and_then(|m| m.values().next())
                .expect("externally tagged");
            let decoded = (spec.decode_json)(inner).expect("decode");
            assert_eq!(decoded, p, "{id} decode round-trip");
            // Schema must be an object schema.
            let schema = (spec.payload_schema)();
            assert_eq!(schema["type"], "object", "{id} schema");
        }
    }

    #[test]
    fn ids_are_unique_and_mcp_names_are_valid_and_unique() {
        let mut ids: Vec<&str> = registry().iter().map(|s| s.id).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "duplicate command ids");

        let mut names: Vec<String> = registry().iter().map(|s| mcp_tool_name(s.id)).collect();
        for name in &names {
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "invalid MCP tool name {name}"
            );
            assert_eq!(
                id_for_mcp_tool(name).map(mcp_tool_name).as_deref(),
                Some(name.as_str())
            );
        }
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate MCP tool names");
    }

    #[test]
    fn decode_rejects_malformed_payloads() {
        let spec = spec(ids::WORLD_SET_BLOCK).unwrap();
        assert!((spec.decode_json)(&json!({"pos": {"x": 0, "y": 0}})).is_err());
        assert!((spec.decode_json)(&json!("nonsense")).is_err());
    }
}
