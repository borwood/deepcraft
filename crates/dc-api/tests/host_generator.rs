//! The pluggable base-terrain generator (client-through-dc-api milestone):
//! a `HostWorld` built with `with_generator` serves the closure's terrain,
//! edits layer on top of it, and the default `new` path is untouched.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos};

use dc_api::{
    CapabilityToken, CommandEnvelope, ConsumerId, ConsumerKind, Grant, HostWorld, Payload, Vec3i,
    Volume, payload,
};

/// A recognizable synthetic terrain: solid stone slab for chunk-y < 0, air
/// above — nothing like the built-in hill-field.
fn slab_generator(pos: ChunkPos) -> Chunk {
    let mut chunk = Chunk::new();
    if pos.y < 0 {
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    chunk.set(x, y, z, Block::Stone);
                }
            }
        }
    }
    chunk
}

fn writer() -> (ConsumerId, CapabilityToken) {
    (
        ConsumerId::new(ConsumerKind::Player, "test"),
        CapabilityToken::new(vec![
            Grant::WorldWrite { volume: None },
            Grant::WorldRead { volume: None },
        ]),
    )
}

#[test]
fn generator_terrain_is_served_and_edits_layer_on_top() {
    let mut world = HostWorld::with_generator(7, Box::new(slab_generator));

    // The closure's terrain, not the hill-field.
    assert_eq!(world.block_at(Vec3i::new(5, -1, 5)), Block::Stone);
    assert_eq!(world.block_at(Vec3i::new(5, 0, 5)), Block::Air);
    // The built-in generator would have grass in [-3, -1]; the slab has none.
    assert_eq!(world.block_at(Vec3i::new(0, -2, 0)), Block::Stone);

    // An edit goes through the command door and lands on the slab terrain.
    let (source, grant) = writer();
    world
        .submit(CommandEnvelope {
            id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
            source,
            grant,
            payload: Payload::SetBlock(payload::SetBlock {
                pos: Vec3i::new(5, 0, 5),
                block: "dc:wood".into(),
            }),
            target_tick: None,
            txn: None,
        })
        .expect("submit");
    world.tick();
    assert_eq!(world.block_at(Vec3i::new(5, 0, 5)), Block::Wood);

    // The chunk accessor sees terrain + edit together (the cache-fill path).
    let chunk = world.chunk(ChunkPos::from_world_voxel(5, 0, 5));
    assert_eq!(chunk.get(5, 0, 5), Block::Wood);
    assert_eq!(chunk.get(6, 0, 5), Block::Air);

    // Determinism: a fresh world with the same generator + edit hashes equal.
    let vol = Volume::new(Vec3i::new(-4, -4, -4), Vec3i::new(8, 8, 8));
    let hash = world.region_hash(vol);
    let mut replay = HostWorld::with_generator(7, Box::new(slab_generator));
    let (source, grant) = writer();
    replay
        .submit(CommandEnvelope {
            id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
            source,
            grant,
            payload: Payload::SetBlock(payload::SetBlock {
                pos: Vec3i::new(5, 0, 5),
                block: "dc:wood".into(),
            }),
            target_tick: None,
            txn: None,
        })
        .expect("submit");
    replay.tick();
    assert_eq!(hash, replay.region_hash(vol));

    // And it differs from the built-in terrain (same seed, no generator).
    let mut builtin = HostWorld::new(7);
    assert_ne!(hash, builtin.region_hash(vol));
}
