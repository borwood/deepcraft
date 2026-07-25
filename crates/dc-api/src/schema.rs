//! Machine-readable command table (docs/API.md principle 5).
//!
//! ONE declarative site — the `commands! { ... }` table below — generates the
//! three things that used to be hand-synchronized per command: the `ids::`
//! string const, the [`Payload`] enum variant (+ its [`Payload::command_id`]
//! mapping), and the [`registry`] entry. A command that is missing from any one
//! of those places is now a **compile error**, not a completeness-test artifact:
//! you cannot add a `Payload` variant without its registry row (they are the
//! same table row), nor add a row without the variant (the generated
//! `command_id` match would reference a variant that does not exist). See
//! docs/API.md decision 7.
//!
//! Each row registers: id, kind, doc, a hand-rolled JSON schema for its payload,
//! a JSON decoder into the typed [`Payload`], and an optional value-completion
//! source ([`Completer`], decision 6). Consumers are GENERATED from this table —
//! the MCP tool list is `registry()` mapped into rmcp tools, never hand-written —
//! so the surface cannot drift from its consumers.
//!
//! The reflection contract a consumer can rely on per [`CommandSpec`]: `id`,
//! `kind`, `doc`, `capability` (human-readable), `payload_schema` (the wire JSON
//! schema — inline, no `$ref`; property *descriptions* are load-bearing, help
//! and completion parse them), `decode_json` (JSON → typed payload), and
//! `completions` (value-level completion for a parameter, when a source exists).
//!
//! The per-command payload *structs* stay hand-written in [`crate::payload`];
//! only the union, the ids, the id-mapping, and the registry are macro-emitted
//! here (and re-exported from `payload` so `crate::payload::Payload` / `ids`
//! keep their paths).

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::host::HostWorld;

/// Command (mutates at a tick boundary) vs query (reads the last completed
/// tick's state immediately).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CommandKind {
    Command,
    Query,
}

/// One registered command/query. The fields are the machine-readable
/// reflection contract (docs/API.md principle 5): consumers — the MCP tool
/// list, WASM binding glue, editor/console property UIs — are generated from
/// them and never hand-written, so the surface cannot drift.
pub struct CommandSpec {
    pub id: &'static str,
    pub kind: CommandKind,
    pub doc: &'static str,
    /// Human-readable capability requirement, for tool docs.
    pub capability: &'static str,
    /// JSON schema (draft-07-ish subset) for the payload. Inline, no `$ref`;
    /// property descriptions are part of the contract (consumers parse them).
    pub payload_schema: fn() -> Value,
    /// Decode a JSON payload (e.g. MCP tool arguments) into the typed enum.
    pub decode_json: fn(&Value) -> Result<Payload, String>,
    /// Value-level completion source for this command's parameters, or `None`
    /// when no meaningful source exists (docs/API.md decision 6). Parameter
    /// *names*/types already come from `payload_schema`; this completes legal
    /// *values* (`block=<TAB>` → registered block names). A console consuming
    /// the registry offers value completion for free, and it stays free for
    /// future commands. See [`Completer`].
    pub completions: Option<Completer>,
}

/// A value-completion source for a command's parameters (docs/API.md
/// decision 6). Given a `param_path` — the payload field to complete, a dotted
/// path for future nested fields but today a top-level key like `"block"` — and
/// the partial `prefix` typed so far, it returns candidate values. An unknown
/// `param_path` yields an empty vec (a command's completer owns only the params
/// with a real source).
///
/// Two shapes so a world-independent source (a fixed enum) is not forced to
/// take a world it never reads — the split the ratified decision points at:
pub enum Completer {
    /// World-independent candidates (a fixed vocabulary, e.g. event kinds or
    /// posture names). Evaluable from the schema alone, with no world borrow —
    /// a console can offer these before it even holds the world.
    Static(fn(param_path: &str, prefix: &str) -> Vec<String>),
    /// Candidates drawn from live world/registry content (registered block
    /// names, spawned character names, defined content classes). Takes a
    /// read-only borrow of the host world — the authority layer already holds
    /// one for the in-client console. dc-api owns [`HostWorld`], so this stays
    /// in-crate and wasm-safe.
    World(fn(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String>),
}

fn decode<T: DeserializeOwned>(v: &Value, wrap: fn(T) -> Payload) -> Result<Payload, String> {
    serde_json::from_value::<T>(v.clone())
        .map(wrap)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------- completion sources --
//
// Real value sources for the obvious cases; commands with no meaningful source
// carry `None`. No invented completions.

/// `block` params (`set_block`, `fill`): the block names this world resolves.
/// World-backed on purpose — today a fixed table, but the data-driven block
/// registry (host.rs `block_from_name`) will make it live world content and
/// this source will not change (placeholder state is not the design target).
fn complete_block(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String> {
    if param_path != "block" {
        return Vec::new();
    }
    world
        .block_names()
        .iter()
        .filter(|n| n.starts_with(prefix))
        .map(|n| (*n).to_string())
        .collect()
}

/// `character` params (the controller verbs and senses): the names of
/// characters currently alive in the world — a genuinely dynamic source.
fn complete_character(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String> {
    if param_path != "character" {
        return Vec::new();
    }
    world
        .characters()
        .map(|c| c.name.clone())
        .filter(|n| n.starts_with(prefix))
        .collect()
}

/// `class` param (`define_class_member`): the content classes registered so
/// far — a member joins an existing class, so its name completes from the
/// registry.
fn complete_class(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String> {
    if param_path != "class" {
        return Vec::new();
    }
    world
        .content_classes()
        .map(|c| c.name.clone())
        .filter(|n| n.starts_with(prefix))
        .collect()
}

/// `set_posture` completes a world-backed `character` and a static `posture`
/// (the one command with a source on two params) — so it is a `World` completer
/// that answers the static param without touching the world.
fn complete_set_posture(world: &HostWorld, param_path: &str, prefix: &str) -> Vec<String> {
    match param_path {
        "character" => complete_character(world, param_path, prefix),
        "posture" => filter_static(&["standing", "crouching"], prefix),
        _ => Vec::new(),
    }
}

/// `kinds` param (`events/subscribe`): the fixed event-kind vocabulary — a
/// purely world-independent source (the `Static` half of the split).
fn complete_event_kinds(param_path: &str, prefix: &str) -> Vec<String> {
    if param_path != "kinds" {
        return Vec::new();
    }
    filter_static(&["BlockChanged", "EntitySpawned", "ItemDefined"], prefix)
}

/// Prefix-filter a fixed candidate list into owned strings.
fn filter_static(candidates: &[&str], prefix: &str) -> Vec<String> {
    candidates
        .iter()
        .filter(|c| c.starts_with(prefix))
        .map(|c| (*c).to_string())
        .collect()
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

/// Schema for a `Vec<ParamSpec>` (serde externally tagged kinds). The JSON
/// shape mirrors serde exactly; deep validation happens at define time.
fn s_param_specs(doc: &str) -> Value {
    json!({
        "type": "array",
        "description": format!(
            "{doc}. Each item: {{name, required, kind}} where kind is \"Text\", \
             \"MaterialName\", {{\"Number\":{{\"min\",\"max\"}}}}, \
             {{\"Range\":{{\"min\",\"max\"}}}}, or {{\"Choice\":{{\"options\":[...]}}}}."
        ),
        "items": {
            "type": "object",
            "properties": {
                "name": s_str("parameter name"),
                "kind": {"description": "parameter kind (serde externally tagged)"},
                "required": {"type": "boolean", "description": "must members supply it?"},
            },
            "required": ["name", "kind", "required"],
        },
    })
}

/// Schema for a `Vec<ParamEntry>`: `(name, value)` pairs over the closed
/// value vocabulary.
fn s_param_entries(doc: &str) -> Value {
    json!({
        "type": "array",
        "description": format!(
            "{doc}. Each item: {{name, value}} where value is \
             {{\"Number\": f64}}, {{\"Range\": [lo, hi]}}, or {{\"Text\": string}}."
        ),
        "items": {
            "type": "object",
            "properties": {
                "name": s_str("parameter name from the class contract"),
                "value": {"description": "parameter value (serde externally tagged)"},
            },
            "required": ["name", "value"],
        },
    })
}

/// Schema for a body plan's `segments`/`slots` (body-plan staircase). Loose
/// object arrays that mirror serde exactly; the joint-tree and verb→slot
/// contract are deeply validated at define time.
fn s_body_plan(doc: &str) -> Value {
    json!({
        "type": "object",
        "description": format!(
            "{doc}. Fields: name (namespaced, e.g. dc:body/biped), doc, \
             segments (each {{name, parent|null, pivot_m:[3], size_m:[3], \
             offset_m:[3], tint:[3]}}), slots (each {{verb, clip}} — verb one \
             of idle/walk/jump; idle+walk required)."
        ),
        "properties": {
            "name": s_str("namespaced plan name"),
            "doc": s_str("human-readable description"),
            "segments": {"type": "array", "description": "joint-tree cuboid segments"},
            "slots": {"type": "array", "description": "verb -> clip bindings"},
        },
        "required": ["name", "segments", "slots"],
    })
}

/// Schema for an anim clip payload (body-plan staircase). Loose; keyframe
/// timing and finiteness are validated at define time.
fn s_anim_clip(doc: &str) -> Value {
    json!({
        "type": "object",
        "description": format!(
            "{doc}. Fields: name (namespaced, e.g. dc:anim/biped_walk), doc, \
             duration_s (>0), loops (bool), keyframes (each {{t, root_bob_m, \
             rotations:[{{segment, euler:[3]}}]}}, t strictly ascending in \
             [0,duration_s])."
        ),
        "properties": {
            "name": s_str("namespaced clip name"),
            "doc": s_str("human-readable description"),
            "duration_s": s_num("clip length in seconds (>0)"),
            "loops": {"type": "boolean", "description": "does the clip loop?"},
            "keyframes": {"type": "array", "description": "keyed joint rotations over time"},
        },
        "required": ["name", "duration_s", "loops", "keyframes"],
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

/// The one declarative command table. Each row is `Variant: Kind { ... }` and
/// generates, together, the `ids::` const, the `Payload` variant + its
/// `command_id` mapping, and the `registry()` entry — so the three cannot
/// drift. Row order is the **`Payload` wire order** (postcard encodes a variant
/// by its declaration index; appended commands stay at the end); `registry()`
/// therefore returns this same order. The wrapped payload struct for `Variant`
/// is `crate::payload::Variant`.
macro_rules! commands {
    ($(
        $variant:ident : $kind:ident {
            id: $const:ident = $id:literal,
            doc: $doc:expr,
            cap: $cap:expr,
            schema: $schema:expr,
            complete: $comp:expr,
        }
    )*) => {
        /// Command ids. Convention (docs/API.md, decision 1):
        /// `dc:domain/verb_noun`.
        pub mod ids {
            $( pub const $const: &str = $id; )*
        }

        /// The typed union of every payload in the v0 slice. Externally tagged
        /// serde (JSON: `{"SetBlock": {...}}`), enum-indexed in postcard — so
        /// the variant order here is wire identity and new commands append at
        /// the end of the table. Generated from the command table together with
        /// the ids and the registry (schema.rs), so it cannot drift from them.
        #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
        pub enum Payload {
            $( $variant(crate::payload::$variant), )*
        }

        impl Payload {
            /// The command id this payload belongs to. The host rejects
            /// envelopes whose `id` field disagrees. Generated from the same
            /// table as the registry, so every variant has an id and vice
            /// versa.
            pub fn command_id(&self) -> &'static str {
                match self {
                    $( Payload::$variant(_) => ids::$const, )*
                }
            }
        }

        /// The v0 command/query registry — generated from the table above with
        /// the ids and the `Payload` union, so a missing command is a compile
        /// error, not a test failure.
        pub fn registry() -> &'static [CommandSpec] {
            static REGISTRY: &[CommandSpec] = &[
                $( CommandSpec {
                    id: ids::$const,
                    kind: CommandKind::$kind,
                    doc: $doc,
                    capability: $cap,
                    payload_schema: $schema,
                    decode_json: |v| decode(v, Payload::$variant),
                    completions: $comp,
                }, )*
            ];
            REGISTRY
        }
    };
}

commands! {
    SetBlock: Command {
        id: WORLD_SET_BLOCK = "dc:world/set_block",
        doc: "Set one voxel to a named block state.",
        cap: "world.write covering pos",
        schema: || s_obj(
            "set_block payload",
            &[
                ("pos", s_vec3i("voxel to set"), true),
                ("block", s_str("block name, e.g. dc:stone"), true),
            ],
        ),
        complete: Some(Completer::World(complete_block)),
    }
    GetBlock: Query {
        id: WORLD_GET_BLOCK = "dc:world/get_block",
        doc: "Read one voxel's block state.",
        cap: "world.read covering pos",
        schema: || s_obj("get_block payload", &[("pos", s_vec3i("voxel to read"), true)]),
        complete: None,
    }
    Fill: Command {
        id: WORLD_FILL = "dc:world/fill",
        doc: "Fill an inclusive box of voxels with a named block state.",
        cap: "world.write covering the box",
        schema: || s_obj(
            "fill payload",
            &[
                ("min", s_vec3i("inclusive minimum corner"), true),
                ("max", s_vec3i("inclusive maximum corner"), true),
                ("block", s_str("block name, e.g. dc:stone"), true),
            ],
        ),
        complete: Some(Completer::World(complete_block)),
    }
    ScanRegion: Query {
        id: WORLD_SCAN_REGION = "dc:world/scan_region",
        doc: "Read an inclusive box of voxels as palette + indices \
              (x-fastest, then z, then y).",
        cap: "world.read covering the box",
        schema: || s_obj(
            "scan_region payload",
            &[
                ("min", s_vec3i("inclusive minimum corner"), true),
                ("max", s_vec3i("inclusive maximum corner"), true),
            ],
        ),
        complete: None,
    }
    EntitySpawn: Command {
        id: ENTITY_SPAWN = "dc:entity/spawn",
        doc: "Spawn a simple entity of a named kind at a position.",
        cap: "entity.spawn",
        schema: || s_obj(
            "spawn payload",
            &[
                ("kind", s_str("entity kind, e.g. dc:deer"), true),
                ("pos", s_vec3f("spawn position in meters"), true),
            ],
        ),
        complete: None,
    }
    EntityQuery: Query {
        id: ENTITY_QUERY = "dc:entity/query",
        doc: "List entities, optionally filtered by volume and kind.",
        cap: "world.read covering the volume (unbounded read if no volume)",
        schema: || s_obj(
            "entity query payload",
            &[
                ("volume", s_volume("only entities inside this box"), false),
                ("kind", s_str("only entities of this kind"), false),
            ],
        ),
        complete: None,
    }
    DefineItem: Command {
        id: REGISTRY_DEFINE_ITEM = "dc:registry/define_item",
        doc: "Define a data-driven item. The name's namespace must be owned \
              by the consumer's registry.define grant.",
        cap: "registry.define(namespace of name)",
        schema: || s_obj(
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
        ),
        complete: None,
    }
    EventsSubscribe: Command {
        id: EVENTS_SUBSCRIBE = "dc:events/subscribe",
        doc: "Open an event subscription. Returns the subscription id in \
              the receipt's effects.",
        cap: "events.subscribe",
        schema: || s_obj(
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
        ),
        complete: Some(Completer::Static(complete_event_kinds)),
    }
    EventsPoll: Query {
        id: EVENTS_POLL = "dc:events/poll",
        doc: "Drain queued events from a subscription you own.",
        cap: "events.subscribe (and subscription ownership)",
        schema: || s_obj(
            "poll payload",
            &[
                (
                    "subscription",
                    s_int("subscription id from subscribe"),
                    true,
                ),
                ("max", s_int("max events to return (default 256)"), false),
            ],
        ),
        complete: None,
    }
    SpawnCharacter: Command {
        id: CHARACTER_SPAWN = "dc:character/spawn_character",
        doc: "Spawn a persistent named character with a physical body at a \
              position (feet, world meters). It exists in the world, falls \
              and collides, and is driven by whoever holds control of it.",
        cap: "entity.spawn (dev grant; character sessions spawn via \
              their attach flow)",
        schema: || s_obj(
            "spawn_character payload",
            &[
                (
                    "name",
                    s_str("bare slug name ([a-z0-9_-], max 64), e.g. scout"),
                    true,
                ),
                ("pos", s_vec3f("feet position in world meters"), true),
            ],
        ),
        complete: None,
    }
    SetMoveIntent: Command {
        id: CHARACTER_SET_MOVE_INTENT = "dc:character/set_move_intent",
        doc: "Set a character's horizontal movement intent: world-space \
              direction (dx, dz — normalized; zero = stop) and a fraction \
              of full walk speed. Persists until countermanded; the body \
              integrates with collision and gravity every tick.",
        cap: "character.control(character)",
        schema: || s_obj(
            "set_move_intent payload",
            &[
                ("character", s_str("character name"), true),
                ("dx", s_num("world-space X direction component"), true),
                ("dz", s_num("world-space Z direction component"), true),
                ("speed", s_num("fraction of full walk speed, 0..1"), true),
            ],
        ),
        complete: Some(Completer::World(complete_character)),
    }
    SetLook: Command {
        id: CHARACTER_SET_LOOK = "dc:character/set_look",
        doc: "Aim a character's gaze: yaw and pitch in radians \
              (yaw 0 = -Z; NEGATIVE pitch looks down, clamped to ±1.55).",
        cap: "character.control(character)",
        schema: || s_obj(
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
        ),
        complete: Some(Completer::World(complete_character)),
    }
    Jump: Command {
        id: CHARACTER_JUMP = "dc:character/jump",
        doc: "Request a jump; fires at the next tick if the character is \
              on the ground then (dropped otherwise).",
        cap: "character.control(character)",
        schema: || s_obj(
            "jump payload",
            &[("character", s_str("character name"), true)],
        ),
        complete: Some(Completer::World(complete_character)),
    }
    CharacterPose: Query {
        id: CHARACTER_POSE = "dc:character/pose",
        doc: "The character's own proprioception: feet position in both \
              meters (`pos`) and world voxels (`pos_voxel`, the coordinate \
              `get_block` takes), velocity, yaw/pitch, on_ground, posture \
              (`standing`|`crouching`), and eye_in_solid (true = its eyes \
              are buried; senses from here see the inside of terrain).",
        cap: "character.control(character)",
        schema: || s_obj(
            "pose payload",
            &[("character", s_str("character name"), true)],
        ),
        complete: Some(Completer::World(complete_character)),
    }
    SenseRaycast: Query {
        id: CHARACTER_SENSE_RAYCAST = "dc:character/sense_raycast",
        doc: "Cast the character's gaze from its eyes along its look \
              direction (or a given direction): first solid voxel with \
              block name, entry face, and distance. Max range 50 m.",
        cap: "character.control(character)",
        schema: || s_obj(
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
        ),
        complete: Some(Completer::World(complete_character)),
    }
    SenseSurroundings: Query {
        id: CHARACTER_SENSE_SURROUNDINGS = "dc:character/sense_surroundings",
        doc: "The character's near perception: scan the block volume in a \
              cube of `radius` voxels (max 16) around its feet, as \
              palette + indices (x-fastest, then z, then y).",
        cap: "character.control(character)",
        schema: || s_obj(
            "sense_surroundings payload",
            &[
                ("character", s_str("character name"), true),
                ("radius", s_int("cube half-extent in voxels, 0..=16"), true),
            ],
        ),
        complete: Some(Completer::World(complete_character)),
    }
    DefineContentClass: Command {
        id: REGISTRY_DEFINE_CONTENT_CLASS = "dc:registry/define_content_class",
        doc: "Declare a content class (a contract): its namespaced name \
              and the parameter schema members must satisfy. Member \
              defines are validated against it at define time.",
        cap: "registry.define(namespace of name)",
        schema: || s_obj(
            "define_content_class payload",
            &[
                (
                    "name",
                    s_str("namespaced class name, e.g. dc:stratum/clastic-fine"),
                    true,
                ),
                ("doc", s_str("human-readable description"), false),
                (
                    "params",
                    s_param_specs("the class contract: parameters members must supply"),
                    true,
                ),
            ],
        ),
        complete: None,
    }
    DefineClassMember: Command {
        id: REGISTRY_DEFINE_CLASS_MEMBER = "dc:registry/define_class_member",
        doc: "Register a member into a content class. `params` must \
              satisfy the class contract (schema-validated at define \
              time). The grant must own the member name's namespace; the \
              class may live in another namespace.",
        cap: "registry.define(namespace of name)",
        schema: || s_obj(
            "define_class_member payload",
            &[
                (
                    "name",
                    s_str("namespaced member name, e.g. dc:geo/mudstone"),
                    true,
                ),
                (
                    "class",
                    s_str("class being implemented, e.g. dc:stratum/clastic-fine"),
                    true,
                ),
                (
                    "params",
                    s_param_entries(
                        "member parameters, validated against the class contract",
                    ),
                    true,
                ),
            ],
        ),
        complete: Some(Completer::World(complete_class)),
    }
    DefineBodyPlan: Command {
        id: REGISTRY_DEFINE_BODY_PLAN = "dc:registry/define_body_plan",
        doc: "Define a body plan: a joint-tree of cuboid segments plus its \
              verb->anim-slot bindings. FAILS at define time if a required \
              verb slot (idle/walk) is unfilled, a verb is unknown, or a \
              slot binds a missing or joint-incompatible clip. Define its \
              clips first.",
        cap: "registry.define(namespace of name)",
        schema: || s_body_plan("define_body_plan payload"),
        complete: None,
    }
    DefineAnimClip: Command {
        id: REGISTRY_DEFINE_ANIM_CLIP = "dc:registry/define_anim_clip",
        doc: "Define an animation clip: keyframed joint rotations plus an \
              optional root bob, and a loop flag. Standalone data — define \
              clips before the body plan that binds them. Validated at \
              define time (finite positive duration, strictly-ascending \
              in-range keyframe times).",
        cap: "registry.define(namespace of name)",
        schema: || s_anim_clip("define_anim_clip payload"),
        complete: None,
    }
    SetPosture: Command {
        id: CHARACTER_SET_POSTURE = "dc:character/set_posture",
        doc: "Set a character's discrete collider posture: `standing` or \
              `crouching`. Crouching shrinks the swept-AABB height at the \
              next tick (an honest hunker that clears low gaps); the \
              cosmetic bend is client-side. Standing up is refused when the \
              taller collider would embed in solid (e.g. under a low \
              ceiling), leaving the body crouched.",
        cap: "character.control(character)",
        schema: || s_obj(
            "set_posture payload",
            &[
                ("character", s_str("character name"), true),
                (
                    "posture",
                    s_str("posture: \"standing\" or \"crouching\""),
                    true,
                ),
            ],
        ),
        complete: Some(Completer::World(complete_set_posture)),
    }
    GetContents: Query {
        id: WORLD_GET_CONTENTS = "dc:world/get_contents",
        doc: "Read one voxel's WHOLE material composition — the full contents, \
              not the single classified block name `get_block` returns: the \
              structure / pore-fill / debris material multisets (slug + \
              count-in-eighths), the structure shape and occupancy (solid / \
              free / open-pore eighths), plus the classified winner block for \
              reference. `has_contents` is a PER-VOXEL fact: false where the \
              world has no composition record for this voxel (the unrecorded \
              basement below the deep-time record, legacy soil, ocean floor, \
              the border wilds, S1 terrain) — and then `classified` echoes the \
              stored `block` rather than naming a mixture that does not exist. \
              A solid block with `has_contents: false` is NOT air: it is rock \
              whose composition the world cannot state.",
        cap: "world.read covering pos",
        schema: || s_obj("get_contents payload", &[("pos", s_vec3i("voxel to read"), true)]),
        complete: None,
    }
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
    use crate::capability::Grant;
    use crate::envelope::{ConsumerId, ConsumerKind};
    use crate::payload::{self, Vec3f, Vec3i, Volume};

    /// Every Payload variant round-trips through its registry decoder. With the
    /// macro, "the registry covers every variant" is now guaranteed at compile
    /// time (variant and row are one table entry); this test keeps the per-
    /// command *decode* coverage and, via the length check, guards the sample
    /// list itself against rot.
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
                pos: Vec3f::new(0.5, 1.0, 0.5),
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
                pos: Vec3f::new(1.0, 2.0, 3.0),
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
            Payload::SetPosture(payload::SetPosture {
                character: "scout".into(),
                posture: "crouching".into(),
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
            Payload::DefineContentClass(payload::DefineContentClass {
                name: "demo:stratum/test".into(),
                doc: "a test class".into(),
                params: vec![crate::classes::ParamSpec {
                    name: "abundance".into(),
                    kind: crate::classes::ParamKind::Number {
                        min: 0.0,
                        max: 10.0,
                    },
                    required: true,
                }],
            }),
            Payload::DefineClassMember(payload::DefineClassMember {
                name: "demo:member/test".into(),
                class: "demo:stratum/test".into(),
                params: vec![crate::classes::ParamEntry {
                    name: "abundance".into(),
                    value: crate::classes::ParamValue::Number(1.0),
                }],
            }),
            Payload::DefineAnimClip(payload::DefineAnimClip(
                crate::bodies::biped_clips()[0].clone(),
            )),
            Payload::DefineBodyPlan(payload::DefineBodyPlan(crate::bodies::biped_plan())),
            Payload::GetContents(payload::GetContents {
                pos: Vec3i::new(1, 2, 3),
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

    /// The macro's guarantee, pinned at the table level: every registry row's
    /// id maps 1:1 through the MCP name mapping and its schema is an object.
    /// This is the invariant that used to rest on the completeness test's
    /// length equality; it is now structural (variant and row are one entry).
    #[test]
    fn macro_table_is_internally_consistent() {
        for spec in registry() {
            assert_eq!((spec.payload_schema)()["type"], "object", "{}", spec.id);
            assert_eq!(
                id_for_mcp_tool(&mcp_tool_name(spec.id)),
                Some(spec.id),
                "{} name mapping",
                spec.id
            );
        }
        // The Payload/registry pairing: a looked-up spec reports its own id.
        assert_eq!(
            spec(ids::WORLD_SET_BLOCK).map(|s| s.id),
            Some(ids::WORLD_SET_BLOCK)
        );
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

    // -------------------------------------------------- completions (dec. 6) --

    /// Spawn a character into a fresh world through the one door, so the
    /// world-backed completer has live state to read.
    fn world_with_character(name: &str) -> HostWorld {
        let mut world = HostWorld::new(7);
        let source = ConsumerId::new(ConsumerKind::McpSession, "test");
        let grant = crate::capability::CapabilityToken::new(vec![Grant::EntitySpawn]);
        let env = crate::envelope::CommandEnvelope {
            id: ids::CHARACTER_SPAWN.to_string(),
            source,
            grant,
            payload: Payload::SpawnCharacter(payload::SpawnCharacter {
                name: name.to_string(),
                pos: Vec3f::new(0.0, 40.0, 0.0),
            }),
            target_tick: None,
            txn: None,
        };
        assert!(world.submit(env).is_ok(), "spawn accepted");
        world.tick();
        world
    }

    #[test]
    fn block_completion_offers_known_names_filtered() {
        let world = HostWorld::new(0);
        let Some(Completer::World(f)) = spec(ids::WORLD_SET_BLOCK).unwrap().completions else {
            panic!("set_block must carry a world completer for `block`");
        };
        let got = f(&world, "block", "dc:s");
        assert!(got.contains(&"dc:stone".to_string()));
        assert!(got.contains(&"dc:sandstone".to_string()));
        assert!(!got.contains(&"dc:dirt".to_string()), "prefix filtered");
        // Unknown param path → nothing (the completer owns only `block`).
        assert!(f(&world, "pos", "").is_empty());
        // fill shares the same source.
        assert!(matches!(
            spec(ids::WORLD_FILL).unwrap().completions,
            Some(Completer::World(_))
        ));
    }

    #[test]
    fn event_kinds_completion_is_static_and_filtered() {
        let Some(Completer::Static(f)) = spec(ids::EVENTS_SUBSCRIBE).unwrap().completions else {
            panic!("events/subscribe must carry a static completer for `kinds`");
        };
        assert_eq!(f("kinds", "Entity"), vec!["EntitySpawned".to_string()]);
        assert_eq!(
            f("kinds", ""),
            vec![
                "BlockChanged".to_string(),
                "EntitySpawned".to_string(),
                "ItemDefined".to_string()
            ]
        );
        assert!(f("volume", "").is_empty(), "unknown param path → empty");
    }

    #[test]
    fn character_completion_reads_live_world() {
        let world = world_with_character("scout");
        let Some(Completer::World(f)) = spec(ids::CHARACTER_SET_MOVE_INTENT).unwrap().completions
        else {
            panic!("set_move_intent must carry a world completer for `character`");
        };
        assert_eq!(f(&world, "character", ""), vec!["scout".to_string()]);
        assert!(f(&world, "character", "z").is_empty(), "prefix filtered");
        // Empty world → no candidates (genuinely dynamic, not a fixed list).
        assert!(f(&HostWorld::new(0), "character", "").is_empty());
    }

    #[test]
    fn set_posture_completes_both_character_and_static_posture() {
        let world = world_with_character("scout");
        let Some(Completer::World(f)) = spec(ids::CHARACTER_SET_POSTURE).unwrap().completions
        else {
            panic!("set_posture carries a world completer (character + posture)");
        };
        assert_eq!(f(&world, "character", ""), vec!["scout".to_string()]);
        assert_eq!(f(&world, "posture", "cr"), vec!["crouching".to_string()]);
        assert!(f(&world, "nonesuch", "").is_empty());
    }

    #[test]
    fn class_completion_reads_registered_classes() {
        let world = HostWorld::new(0);
        // No classes registered yet → empty, but the hook is present.
        let Some(Completer::World(f)) =
            spec(ids::REGISTRY_DEFINE_CLASS_MEMBER).unwrap().completions
        else {
            panic!("define_class_member must carry a world completer for `class`");
        };
        assert!(f(&world, "class", "").is_empty());
    }

    /// The presence/absence of a completion source per command matches intent:
    /// value sources exist only where there is a real one.
    #[test]
    fn completion_hook_presence_matches_intent() {
        let has_source = [
            ids::WORLD_SET_BLOCK,
            ids::WORLD_FILL,
            ids::EVENTS_SUBSCRIBE,
            ids::CHARACTER_SET_MOVE_INTENT,
            ids::CHARACTER_SET_LOOK,
            ids::CHARACTER_JUMP,
            ids::CHARACTER_POSE,
            ids::CHARACTER_SENSE_RAYCAST,
            ids::CHARACTER_SENSE_SURROUNDINGS,
            ids::CHARACTER_SET_POSTURE,
            ids::REGISTRY_DEFINE_CLASS_MEMBER,
        ];
        let no_source = [
            ids::WORLD_GET_BLOCK,
            ids::WORLD_GET_CONTENTS,
            ids::WORLD_SCAN_REGION,
            ids::ENTITY_SPAWN,
            ids::ENTITY_QUERY,
            ids::REGISTRY_DEFINE_ITEM,
            ids::EVENTS_POLL,
            ids::CHARACTER_SPAWN,
            ids::REGISTRY_DEFINE_CONTENT_CLASS,
            ids::REGISTRY_DEFINE_BODY_PLAN,
            ids::REGISTRY_DEFINE_ANIM_CLIP,
        ];
        for id in has_source {
            assert!(
                spec(id).unwrap().completions.is_some(),
                "{id} should carry a completion source"
            );
        }
        for id in no_source {
            assert!(
                spec(id).unwrap().completions.is_none(),
                "{id} should have no completion source"
            );
        }
        // The two lists partition the whole registry — no command forgotten.
        assert_eq!(has_source.len() + no_source.len(), registry().len());
    }
}
