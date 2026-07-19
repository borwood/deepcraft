// The demo build script — the ONE command sequence used by the parity proof.
//
// This file is a module of dc-host AND textually `include!`d by
// plugins/demo-builder, so the native path, the WASM path, and the MCP path
// all execute byte-identical payload sequences. Only `dc_api` paths may be
// used here; no crate-local imports, and plain `//` comments only (inner doc
// comments are not legal in an `include!` expansion).

/// The demo consumer's build yard (inclusive voxel volume).
pub fn build_volume() -> dc_api::Volume {
    dc_api::Volume::new(dc_api::Vec3i::new(0, 0, 0), dc_api::Vec3i::new(10, 15, 10))
}

/// The demo plugin's manifest token: write/read its yard, define items in
/// `demo:*`, spawn entities, subscribe to events. This is what the host
/// installs; the plugin claims per-command attenuations of it.
pub fn demo_token() -> dc_api::CapabilityToken {
    dc_api::CapabilityToken::new(vec![
        dc_api::Grant::WorldRead {
            volume: Some(build_volume()),
        },
        dc_api::Grant::WorldWrite {
            volume: Some(build_volume()),
        },
        dc_api::Grant::EntitySpawn,
        dc_api::Grant::RegistryDefine {
            namespace: "demo".into(),
        },
        dc_api::Grant::EventsSubscribe,
    ])
}

/// The demo session: subscribe to block changes in the yard, define an item,
/// build a hut (stone floor, wood walls, a doorway), spawn a deer inside.
pub fn demo_payloads() -> Vec<dc_api::Payload> {
    use dc_api::payload::{
        DefineItem, EntitySpawn, EventsSubscribe, Fill, Payload, SetBlock, Vec3f, Vec3i,
    };
    let v = Vec3i::new;
    let fill = |min: Vec3i, max: Vec3i, block: &str| {
        Payload::Fill(Fill {
            min,
            max,
            block: block.into(),
        })
    };
    vec![
        Payload::EventsSubscribe(EventsSubscribe {
            kinds: vec![dc_api::EventKind::BlockChanged],
            volume: Some(build_volume()),
        }),
        Payload::DefineItem(DefineItem {
            name: "demo:builder_wand".into(),
            display_name: "Builder's Wand".into(),
            description: Some("Proof that a plugin can author content.".into()),
        }),
        // Floor.
        fill(v(2, 1, 2), v(6, 1, 6), "dc:stone"),
        // Walls.
        fill(v(2, 2, 2), v(6, 4, 2), "dc:wood"),
        fill(v(2, 2, 6), v(6, 4, 6), "dc:wood"),
        fill(v(2, 2, 3), v(2, 4, 5), "dc:wood"),
        fill(v(6, 2, 3), v(6, 4, 5), "dc:wood"),
        // Doorway.
        Payload::SetBlock(SetBlock {
            pos: v(4, 2, 2),
            block: "dc:air".into(),
        }),
        Payload::SetBlock(SetBlock {
            pos: v(4, 3, 2),
            block: "dc:air".into(),
        }),
        // A resident.
        Payload::EntitySpawn(EntitySpawn {
            kind: "dc:deer".into(),
            pos: Vec3f::new(4.5, 2.0, 4.5),
        }),
    ]
}

/// The minimal (attenuated) grant set a well-behaved consumer claims for one
/// payload — the plugin uses this so each envelope carries least authority.
pub fn minimal_grants_for(payload: &dc_api::Payload) -> Vec<dc_api::Grant> {
    use dc_api::Payload as P;
    match payload {
        P::SetBlock(_) | P::Fill(_) => vec![dc_api::Grant::WorldWrite {
            volume: Some(build_volume()),
        }],
        P::GetBlock(_) | P::ScanRegion(_) | P::EntityQuery(_) => vec![dc_api::Grant::WorldRead {
            volume: Some(build_volume()),
        }],
        P::EntitySpawn(_) => vec![dc_api::Grant::EntitySpawn],
        P::DefineItem(item) => vec![dc_api::Grant::RegistryDefine {
            namespace: item.name.split(':').next().unwrap_or("").into(),
        }],
        P::EventsSubscribe(_) | P::EventsPoll(_) => vec![dc_api::Grant::EventsSubscribe],
        // The demo consumer never touches characters; these arms exist only
        // because the payload union grew (character-MCP milestone) and this
        // match is deliberately exhaustive (drift protection).
        P::SpawnCharacter(_) => vec![dc_api::Grant::EntitySpawn],
        P::SetMoveIntent(dc_api::payload::SetMoveIntent { character, .. })
        | P::SetLook(dc_api::payload::SetLook { character, .. })
        | P::Jump(dc_api::payload::Jump { character })
        | P::CharacterPose(dc_api::payload::CharacterPose { character })
        | P::SenseRaycast(dc_api::payload::SenseRaycast { character, .. })
        | P::SenseSurroundings(dc_api::payload::SenseSurroundings { character, .. }) => {
            vec![dc_api::Grant::CharacterControl {
                character: Some(character.clone()),
            }]
        }
    }
}
