//! Events — the push half of the surface. Delivered into per-subscription
//! queues at tick apply time; consumers drain with `dc:events/poll`.

use serde::{Deserialize, Serialize};

use crate::envelope::{ConsumerId, Tick};
use crate::payload::{Vec3f, Vec3i};

/// Event kinds a subscription can filter on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EventKind {
    BlockChanged,
    EntitySpawned,
    ItemDefined,
}

/// A delivered event. Every event carries the tick it happened on and the
/// consumer whose command caused it — reacting-without-feedback-loops requires
/// being able to filter out your own writes.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    BlockChanged {
        pos: Vec3i,
        from: String,
        to: String,
        cause: ConsumerId,
        tick: Tick,
    },
    EntitySpawned {
        id: u64,
        kind: String,
        pos: Vec3f,
        cause: ConsumerId,
        tick: Tick,
    },
    ItemDefined {
        name: String,
        cause: ConsumerId,
        tick: Tick,
    },
}

impl GameEvent {
    pub fn kind(&self) -> EventKind {
        match self {
            GameEvent::BlockChanged { .. } => EventKind::BlockChanged,
            GameEvent::EntitySpawned { .. } => EventKind::EntitySpawned,
            GameEvent::ItemDefined { .. } => EventKind::ItemDefined,
        }
    }

    /// The world position this event is "at", for volume-filtered
    /// subscriptions. Events with no position (registry) never match a
    /// volume-filtered subscription's volume test — they are delivered only
    /// by kind filter when the subscription has no volume.
    pub fn pos_hint(&self) -> Option<Vec3i> {
        match self {
            GameEvent::BlockChanged { pos, .. } => Some(*pos),
            GameEvent::EntitySpawned { pos, .. } => Some(pos.floor_voxel()),
            GameEvent::ItemDefined { .. } => None,
        }
    }

    pub fn cause(&self) -> &ConsumerId {
        match self {
            GameEvent::BlockChanged { cause, .. }
            | GameEvent::EntitySpawned { cause, .. }
            | GameEvent::ItemDefined { cause, .. } => cause,
        }
    }
}
