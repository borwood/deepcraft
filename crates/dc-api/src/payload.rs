//! Typed command/query payloads — the data half of "commands are data".
//!
//! Every command in the surface has one payload *struct* here and one
//! [`Payload`] variant. The envelope's string id and the payload variant are
//! redundant on purpose (the id is what routes/logs/schemas key on; the variant
//! is what the type system keys on); [`Payload::command_id`] is the bridge and
//! the host rejects envelopes where the two disagree.
//!
//! The [`Payload`] union, the [`ids`] consts, and `command_id` are NOT written
//! here: they are generated from the one command table in [`crate::schema`]
//! (docs/API.md decision 7 — a missing command is a compile error), then
//! re-exported below so `crate::payload::Payload` and `crate::payload::ids`
//! keep their paths. The per-command structs stay hand-written.

use serde::{Deserialize, Serialize};

pub use crate::schema::{Payload, ids};

/// A world-space voxel coordinate (the 3D lattice is unbounded; i64 like
/// dc-core's world-voxel space).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Vec3i {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Vec3i {
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}

/// An entity position in world meters-space (f64 sim coordinates, see
/// ARCHITECTURE.md § floating origin).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vec3f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3f {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// The voxel containing this position (floor per axis).
    pub fn floor_voxel(self) -> Vec3i {
        Vec3i::new(
            self.x.floor() as i64,
            self.y.floor() as i64,
            self.z.floor() as i64,
        )
    }
}

/// An axis-aligned box of voxels, **inclusive** on both ends
/// (`min == max` is a single voxel).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub min: Vec3i,
    pub max: Vec3i,
}

impl Volume {
    /// Normalizing constructor: per-axis min/max of the two corners.
    pub fn new(a: Vec3i, b: Vec3i) -> Self {
        Self {
            min: Vec3i::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Vec3i::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn point(p: Vec3i) -> Self {
        Self { min: p, max: p }
    }

    pub fn contains(&self, p: Vec3i) -> bool {
        (self.min.x..=self.max.x).contains(&p.x)
            && (self.min.y..=self.max.y).contains(&p.y)
            && (self.min.z..=self.max.z).contains(&p.z)
    }

    pub fn contains_volume(&self, other: &Volume) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    /// Number of voxels in the box. Saturates at u64::MAX for absurd inputs.
    pub fn voxel_count(&self) -> u64 {
        let dx = (self.max.x - self.min.x) as u64 + 1;
        let dy = (self.max.y - self.min.y) as u64 + 1;
        let dz = (self.max.z - self.min.z) as u64 + 1;
        dx.saturating_mul(dy).saturating_mul(dz)
    }
}

// ---------------------------------------------------------------- payloads --

/// `dc:world/set_block` — set one voxel to a named block state.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SetBlock {
    pub pos: Vec3i,
    /// Block name, e.g. `dc:stone`. v0 block names are the hard-coded S1 set;
    /// the data-driven block registry is a later slice.
    pub block: String,
}

/// `dc:world/get_block` — read one voxel.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GetBlock {
    pub pos: Vec3i,
}

/// `dc:world/get_contents` — read one voxel's whole material composition (the
/// full [`dc_core::VoxelContents`], not the single classified block name).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GetContents {
    pub pos: Vec3i,
}

/// `dc:world/fill` — set every voxel in an inclusive box to a named block.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Fill {
    pub min: Vec3i,
    pub max: Vec3i,
    pub block: String,
}

/// `dc:world/scan_region` — read an inclusive box of voxels.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ScanRegion {
    pub min: Vec3i,
    pub max: Vec3i,
}

/// `dc:entity/spawn` — spawn a simple entity of a named kind.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntitySpawn {
    pub kind: String,
    pub pos: Vec3f,
}

/// `dc:entity/query` — list entities, optionally filtered by volume and kind.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntityQuery {
    /// Voxel-space filter (an entity matches if the voxel containing its
    /// position is inside). `None` = anywhere (requires an unbounded
    /// `world.read` grant).
    #[serde(default)]
    pub volume: Option<Volume>,
    #[serde(default)]
    pub kind: Option<String>,
}

/// `dc:registry/define_item` — data-driven item definition.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DefineItem {
    /// Namespaced item name `namespace:path`, e.g. `demo:builder_wand`.
    /// The consumer's `registry.define` grant must own the namespace.
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// `dc:registry/define_content_class` — declare a content class (a
/// contract): its namespaced name and the parameter schema its members must
/// satisfy. The consumer's `registry.define` grant must own the namespace.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DefineContentClass {
    /// Namespaced class name, e.g. `dc:stratum/clastic-fine`.
    pub name: String,
    #[serde(default)]
    pub doc: String,
    /// The class contract: what member definitions must supply.
    pub params: Vec<crate::classes::ParamSpec>,
}

/// `dc:registry/define_class_member` — register a member into a content
/// class. `params` is schema-validated against the class contract at define
/// time (the smallest honest opening of the closed `Payload` union: open
/// keys over a closed value vocabulary — see `classes` module docs). The
/// grant must own the *member* name's namespace; the class may live in
/// another namespace.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DefineClassMember {
    /// Namespaced member name, e.g. `dc:geo/mudstone`.
    pub name: String,
    /// The class being implemented, e.g. `dc:stratum/clastic-fine`.
    pub class: String,
    pub params: Vec<crate::classes::ParamEntry>,
}

/// `dc:registry/define_body_plan` — register a body plan (a joint-tree of
/// cuboid segments plus its verb→anim-slot bindings; docs/design/bodies.md).
/// Validated at define time against the already-registered clips: a missing
/// required slot, an unknown verb, or a slot bound to a missing/joint-
/// incompatible clip all reject. The grant must own the plan name's namespace.
/// The payload IS the plan content ([`crate::bodies::BodyPlan`]); the stored
/// def adds provenance.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DefineBodyPlan(pub crate::bodies::BodyPlan);

/// `dc:registry/define_anim_clip` — register an animation clip (keyframed
/// joint rotations + optional root bob; loop flag). Clips are standalone data
/// (they name joints but not a plan), so they are defined before the plan that
/// binds them. The grant must own the clip name's namespace.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DefineAnimClip(pub crate::bodies::AnimClip);

/// `dc:events/subscribe` — open an event subscription with a filter.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventsSubscribe {
    /// Event kinds to receive. Empty = all kinds.
    #[serde(default)]
    pub kinds: Vec<crate::event::EventKind>,
    /// For events with a world position, only deliver those inside this box.
    #[serde(default)]
    pub volume: Option<Volume>,
}

/// `dc:events/poll` — drain queued events from an owned subscription.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventsPoll {
    pub subscription: u64,
    /// Max events to return (default 256).
    #[serde(default)]
    pub max: Option<u32>,
}

/// `dc:character/spawn_character` — create a persistent named character with
/// a body at a position (feet, meters). Dev-grant (`entity.spawn`); the
/// character surface spawns through its session attach flow, never directly.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SpawnCharacter {
    /// Bare slug (`[a-z0-9_-]{1,64}`): becomes a grant scope and a consumer
    /// identity.
    pub name: String,
    pub pos: Vec3f,
}

/// `dc:character/set_move_intent` — set the character's horizontal movement
/// intent: a world-space direction (normalized by the host; zero = stop) and
/// a fraction of full walk speed. Persists until countermanded.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SetMoveIntent {
    pub character: String,
    /// World-space X component of the intended direction.
    pub dx: f64,
    /// World-space Z component of the intended direction.
    pub dz: f64,
    /// Fraction of full walk speed, clamped to [0, 1].
    pub speed: f64,
}

/// `dc:character/set_look` — aim the character's head/eyes.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SetLook {
    pub character: String,
    /// Radians, 0 = -Z.
    pub yaw: f32,
    /// Radians, NEGATIVE looks down; clamped to ±1.55.
    pub pitch: f32,
}

/// `dc:character/jump` — request a jump; fires at the next tick step if the
/// character is on the ground then (dropped otherwise).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Jump {
    pub character: String,
}

/// `dc:character/set_posture` — set the character's discrete collider posture
/// (`standing` | `crouching`). Crouching shrinks the swept-AABB height at the
/// next tick boundary (bodies.md § determinism firewall). A `crouching →
/// standing` change is guarded: it is refused when standing would embed the
/// taller collider in solid terrain (e.g. under a low ceiling), so the body
/// stays crouched rather than clipping upward.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SetPosture {
    pub character: String,
    /// `standing` or `crouching`.
    pub posture: String,
}

/// `dc:character/pose` — the character's own proprioception: pose, ground
/// contact, and eye_in_solid.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CharacterPose {
    pub character: String,
}

/// `dc:character/sense_raycast` — look along the character's own gaze (or a
/// given direction) from its eyes; first solid voxel within range.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SenseRaycast {
    pub character: String,
    /// Direction to look; `None` = the character's current view direction.
    #[serde(default)]
    pub dir: Option<Vec3f>,
    /// Max range in meters, capped at 50.
    #[serde(default)]
    pub max_distance_m: Option<f64>,
}

/// `dc:character/sense_surroundings` — near perception: the block volume in a
/// cube of `radius` voxels (≤ 16) around the character's feet.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SenseSurroundings {
    pub character: String,
    /// Half-extent of the scanned cube in voxels, 0..=16.
    pub radius: u32,
}

/// A simple entity as the reference host stores and reports it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: u64,
    pub kind: String,
    pub pos: Vec3f,
}

/// One material and how many of a voxel's eighths it fills within a role.
/// Each occupied slot is one eighth, so `eighths` is the run length of this
/// material in its (sorted) segment.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MaterialCount {
    /// Qualified material slug, e.g. `dc:granite`.
    pub material: String,
    /// Count in eighths (1..=8).
    pub eighths: u8,
}

/// A voxel's full material composition — the whole [`dc_core::VoxelContents`]
/// unpacked for the dev inspector. The three multisets (structure / pore-fill /
/// debris) are the source of truth; the single classified block name is a
/// *summary* derived from them and lives beside this in [`QueryData::Contents`],
/// never inside it (the "a summary is not an authority" doctrine).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ContentsView {
    /// Structure shape reserving capacity: `none` | `quarter` | `slab` | `full`.
    pub shape: String,
    /// Occupied eighths (structure + pore fill + debris).
    pub solid_eighths: u8,
    /// Wholly unoccupied eighths (what a fluid could still enter).
    pub free_eighths: u8,
    /// Open (unfilled) pores in the structure's reserved capacity.
    pub open_pores: u8,
    /// Unreserved eighths still free for debris.
    pub free_debris_eighths: u8,
    /// Structural fill — the load-bearing identity — sorted by material.
    pub structure: Vec<MaterialCount>,
    /// Fine material packed into the structure's pores, sorted by material.
    pub pore_fill: Vec<MaterialCount>,
    /// Loose granular fill of the unreserved volume, sorted by material.
    pub debris: Vec<MaterialCount>,
}

impl ContentsView {
    /// Unpack canonical [`dc_core::VoxelContents`] into the wire view: each
    /// sorted segment run-length-collapses into `(slug, eighths)` pairs.
    pub fn from_contents(c: &dc_core::VoxelContents) -> Self {
        Self {
            shape: shape_slug(c.shape()).to_string(),
            solid_eighths: c.solid_eighths(),
            free_eighths: c.free_eighths(),
            open_pores: c.open_pores(),
            free_debris_eighths: c.free_debris_eighths(),
            structure: group_materials(c.structure()),
            pore_fill: group_materials(c.pore_fill()),
            debris: group_materials(c.debris()),
        }
    }
}

/// Run-length collapse a sorted material segment into `(slug, eighths)` pairs.
fn group_materials(sorted: &[dc_core::MaterialId]) -> Vec<MaterialCount> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let m = sorted[i];
        let mut n: u8 = 1;
        while i + n as usize < sorted.len() && sorted[i + n as usize] == m {
            n += 1;
        }
        out.push(MaterialCount {
            material: m.qualified_name().to_string(),
            eighths: n,
        });
        i += n as usize;
    }
    out
}

/// Wire slug for a structure shape.
fn shape_slug(shape: dc_core::StructureShape) -> &'static str {
    match shape {
        dc_core::StructureShape::None => "none",
        dc_core::StructureShape::Quarter => "quarter",
        dc_core::StructureShape::Slab => "slab",
        dc_core::StructureShape::Full => "full",
    }
}

/// Typed result data for queries, mirrored to JSON at the MCP boundary.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum QueryData {
    Block {
        block: String,
    },
    /// A voxel's full material composition (`dc:world/get_contents`). The three
    /// multisets in `contents` are the authority; `block` is the stored
    /// (edit-aware) classified name and `classified` is what the contents
    /// themselves classify to — equal for an unedited recorded voxel, and
    /// diverging is the signal that the voxel was edited (an edit writes a
    /// block, not contents) or that no record backs it.
    Contents {
        /// Authoritative stored block name (edit-aware).
        block: String,
        /// The block name `classify` derives from `contents`; equals `block`
        /// for an unedited recorded voxel. When `has_contents` is false this
        /// echoes `block`.
        classified: String,
        /// Whether a full contents record backs this voxel. `false` under the
        /// S1 terrain authority, for legacy stubs, or when the world carries no
        /// contents source — then only `block` is meaningful and `contents` is
        /// the empty composition.
        has_contents: bool,
        contents: ContentsView,
    },
    /// Scan result: palette (block names, order of first appearance while
    /// scanning x-fastest, then z, then y — same order as chunk layout) plus
    /// one palette index per voxel in that scan order.
    Region {
        min: Vec3i,
        max: Vec3i,
        palette: Vec<String>,
        indices: Vec<u32>,
    },
    Entities {
        entities: Vec<EntityInfo>,
    },
    /// Drained events plus how many remain queued after this poll.
    Events {
        events: Vec<crate::event::GameEvent>,
        remaining: u64,
    },
    /// A character's own pose and proprioception (`dc:character/pose`).
    CharacterPose {
        name: String,
        /// Feet position, meters.
        pos: Vec3f,
        vel: Vec3f,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
        /// True when the eye voxel is solid — screenshots/senses from here
        /// are inside terrain.
        eye_in_solid: bool,
        /// Feet **voxel** coordinate at the active scale — the same world-voxel
        /// language `dc:world/get_block`/`scan_region` speak, so a driver can
        /// cross-check its meters pose against block queries with no mental unit
        /// conversion (corrections #10). Appended field: postcard is positional,
        /// so this stays last; `serde(default)` decodes pre-echo streams to the
        /// origin.
        #[serde(default)]
        pos_voxel: Vec3i,
        /// Discrete collider posture as its wire string (`standing`|`crouching`)
        /// — the vocabulary `dc:character/set_posture` accepts, so a driver reads
        /// its own posture back (walk-11 loose end). Appended field; `serde(default)`
        /// decodes pre-posture streams to an empty string.
        #[serde(default)]
        posture: String,
    },
    /// First solid voxel along a character's gaze (`sense_raycast`).
    /// All fields are `None` on a miss.
    CharacterRaycast {
        hit: bool,
        voxel: Option<Vec3i>,
        block: Option<String>,
        /// Entry-face normal; (0,0,0) when the eye started inside a solid.
        normal: Option<Vec3i>,
        distance_m: Option<f64>,
        /// The hit voxel's full material composition, when a contents record
        /// backs it (the same authority `dc:world/get_contents` returns). `None`
        /// on a miss, or when the world carries no contents source (S1 terrain,
        /// legacy stubs). Appended field; `serde(default)` decodes pre-contents
        /// streams to `None` (postcard is positional — this stays last).
        #[serde(default)]
        contents: Option<ContentsView>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_normalizes_and_contains() {
        let v = Volume::new(Vec3i::new(4, -1, 9), Vec3i::new(0, 5, 3));
        assert_eq!(v.min, Vec3i::new(0, -1, 3));
        assert_eq!(v.max, Vec3i::new(4, 5, 9));
        assert!(v.contains(Vec3i::new(0, -1, 3)));
        assert!(v.contains(Vec3i::new(4, 5, 9)));
        assert!(!v.contains(Vec3i::new(5, 0, 4)));
        assert_eq!(Volume::point(Vec3i::new(1, 1, 1)).voxel_count(), 1);
        assert_eq!(v.voxel_count(), 5 * 7 * 7);
    }

    #[test]
    fn volume_containment_is_inclusive() {
        let outer = Volume::new(Vec3i::new(0, 0, 0), Vec3i::new(10, 10, 10));
        let edge = Volume::new(Vec3i::new(0, 0, 0), Vec3i::new(10, 10, 10));
        let inner = Volume::new(Vec3i::new(1, 1, 1), Vec3i::new(9, 9, 9));
        let poking = Volume::new(Vec3i::new(1, 1, 1), Vec3i::new(11, 9, 9));
        assert!(outer.contains_volume(&edge));
        assert!(outer.contains_volume(&inner));
        assert!(!outer.contains_volume(&poking));
        assert!(!inner.contains_volume(&outer));
    }

    #[test]
    fn floor_voxel_handles_negatives() {
        assert_eq!(
            Vec3f::new(-0.5, 2.0, -3.9).floor_voxel(),
            Vec3i::new(-1, 2, -4)
        );
    }

    #[test]
    fn payload_ids_match_convention() {
        // Every id follows dc:domain/verb_noun.
        for id in [
            ids::WORLD_SET_BLOCK,
            ids::WORLD_GET_BLOCK,
            ids::WORLD_GET_CONTENTS,
            ids::WORLD_FILL,
            ids::WORLD_SCAN_REGION,
            ids::ENTITY_SPAWN,
            ids::ENTITY_QUERY,
            ids::REGISTRY_DEFINE_ITEM,
            ids::REGISTRY_DEFINE_CONTENT_CLASS,
            ids::REGISTRY_DEFINE_CLASS_MEMBER,
            ids::REGISTRY_DEFINE_BODY_PLAN,
            ids::REGISTRY_DEFINE_ANIM_CLIP,
            ids::EVENTS_SUBSCRIBE,
            ids::EVENTS_POLL,
            ids::CHARACTER_SPAWN,
            ids::CHARACTER_SET_MOVE_INTENT,
            ids::CHARACTER_SET_LOOK,
            ids::CHARACTER_SET_POSTURE,
            ids::CHARACTER_JUMP,
            ids::CHARACTER_POSE,
            ids::CHARACTER_SENSE_RAYCAST,
            ids::CHARACTER_SENSE_SURROUNDINGS,
        ] {
            let rest = id.strip_prefix("dc:").expect("dc: namespace");
            let (domain, verb) = rest.split_once('/').expect("domain/verb");
            assert!(!domain.is_empty() && !verb.is_empty());
            assert!(
                domain
                    .chars()
                    .chain(verb.chars())
                    .all(|c| c.is_ascii_lowercase() || c == '_')
            );
        }
    }
}
