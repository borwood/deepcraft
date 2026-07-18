//! Typed command/query payloads — the data half of "commands are data".
//!
//! Every command in the surface has one payload struct and one [`Payload`]
//! variant. The envelope's string id and the payload variant are redundant on
//! purpose (the id is what routes/logs/schemas key on; the variant is what the
//! type system keys on); [`Payload::command_id`] is the bridge and the host
//! rejects envelopes where the two disagree.

use serde::{Deserialize, Serialize};

/// Command ids. Convention (docs/API.md, decision 1): `dc:domain/verb_noun`.
pub mod ids {
    pub const WORLD_SET_BLOCK: &str = "dc:world/set_block";
    pub const WORLD_GET_BLOCK: &str = "dc:world/get_block";
    pub const WORLD_FILL: &str = "dc:world/fill";
    pub const WORLD_SCAN_REGION: &str = "dc:world/scan_region";
    pub const ENTITY_SPAWN: &str = "dc:entity/spawn";
    pub const ENTITY_QUERY: &str = "dc:entity/query";
    pub const REGISTRY_DEFINE_ITEM: &str = "dc:registry/define_item";
    pub const EVENTS_SUBSCRIBE: &str = "dc:events/subscribe";
    pub const EVENTS_POLL: &str = "dc:events/poll";
}

/// A world-space voxel coordinate (the 3D lattice is unbounded; i64 like
/// dc-core's world-voxel space).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
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

/// The typed union of every payload in the v0 slice. Externally tagged serde
/// (JSON: `{"SetBlock": {...}}`), enum-indexed in postcard.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Payload {
    SetBlock(SetBlock),
    GetBlock(GetBlock),
    Fill(Fill),
    ScanRegion(ScanRegion),
    EntitySpawn(EntitySpawn),
    EntityQuery(EntityQuery),
    DefineItem(DefineItem),
    EventsSubscribe(EventsSubscribe),
    EventsPoll(EventsPoll),
}

impl Payload {
    /// The command id this payload belongs to. The host rejects envelopes
    /// whose `id` field disagrees.
    pub fn command_id(&self) -> &'static str {
        match self {
            Payload::SetBlock(_) => ids::WORLD_SET_BLOCK,
            Payload::GetBlock(_) => ids::WORLD_GET_BLOCK,
            Payload::Fill(_) => ids::WORLD_FILL,
            Payload::ScanRegion(_) => ids::WORLD_SCAN_REGION,
            Payload::EntitySpawn(_) => ids::ENTITY_SPAWN,
            Payload::EntityQuery(_) => ids::ENTITY_QUERY,
            Payload::DefineItem(_) => ids::REGISTRY_DEFINE_ITEM,
            Payload::EventsSubscribe(_) => ids::EVENTS_SUBSCRIBE,
            Payload::EventsPoll(_) => ids::EVENTS_POLL,
        }
    }
}

/// A simple entity as the reference host stores and reports it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: u64,
    pub kind: String,
    pub pos: Vec3f,
}

/// Typed result data for queries, mirrored to JSON at the MCP boundary.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum QueryData {
    Block {
        block: String,
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
            ids::WORLD_FILL,
            ids::WORLD_SCAN_REGION,
            ids::ENTITY_SPAWN,
            ids::ENTITY_QUERY,
            ids::REGISTRY_DEFINE_ITEM,
            ids::EVENTS_SUBSCRIBE,
            ids::EVENTS_POLL,
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
