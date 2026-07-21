//! `HostWorld.chunks` eviction (journal/0051): the chunk store is a bounded
//! LRU cache over *generated-and-untouched* chunks plus a pinned set of
//! *edited* ones.
//!
//! The two invariants under test are the whole safety argument:
//!
//! 1. **Evicting is invisible.** A dropped chunk re-derives byte-identically,
//!    so no query can tell whether a chunk was resident or regenerated.
//! 2. **Edits are never dropped.** An edited chunk carries information the
//!    generator cannot reproduce, so it is pinned regardless of pressure.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos};

use dc_api::{
    CapabilityToken, CommandEnvelope, ConsumerId, ConsumerKind, Grant, HostWorld, Payload, Vec3i,
    Volume, payload,
};

/// A deliberately position-dependent, high-entropy generator: every voxel is
/// decided by a hash of (seed, chunk pos, local index), so a regenerated chunk
/// matching the original byte-for-byte is a real statement, not an artifact of
/// a mostly-air world.
fn noisy_generator(seed: u64) -> impl Fn(ChunkPos) -> Chunk + Send + Sync + 'static {
    move |pos: ChunkPos| {
        let mut chunk = Chunk::new();
        let base = mix(seed ^ mix(pos.x as u64) ^ mix((pos.y as u64) << 21) ^ mix(pos.z as u64));
        for y in 0..CHUNK_SIZE_USIZE {
            for z in 0..CHUNK_SIZE_USIZE {
                for x in 0..CHUNK_SIZE_USIZE {
                    let n = mix(base ^ ((x + z * 32 + y * 1024) as u64));
                    let block = match n % 5 {
                        0 => Block::Air,
                        1 => Block::Stone,
                        2 => Block::Dirt,
                        3 => Block::Granite,
                        _ => Block::Basalt,
                    };
                    if block != Block::Air {
                        chunk.set(x, y, z, block);
                    }
                }
            }
        }
        chunk
    }
}

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
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

fn set_block(world: &mut HostWorld, pos: Vec3i, block: &str) {
    let (source, grant) = writer();
    world
        .submit(CommandEnvelope {
            id: dc_api::ids::WORLD_SET_BLOCK.to_string(),
            source,
            grant,
            payload: Payload::SetBlock(payload::SetBlock {
                pos,
                block: block.into(),
            }),
            target_tick: None,
            txn: None,
        })
        .expect("submit");
    world.tick();
}

/// Walk a long line of fresh chunk positions — the headless equivalent of the
/// teleport storm: every touch is new ground.
fn sweep(world: &mut HostWorld, count: i32, avoid: &[ChunkPos]) {
    for i in 0..count {
        let pos = ChunkPos::new(1000 + i, 7, 1000 - i);
        assert!(!avoid.contains(&pos), "sweep must not touch the pinned set");
        let (mx, my, mz) = pos.min_voxel();
        // Read through the normal voxel door, exactly as a collision or
        // grounding query would.
        world.block_at(Vec3i::new(mx, my, mz));
    }
}

#[test]
fn evicted_chunks_regenerate_byte_identically() {
    // Several seeds and several positions, because "deterministic" is a claim
    // about the whole (seed, pos) space, not about one lucky chunk.
    for seed in [1u64, 7, 424_242] {
        let mut world = HostWorld::with_generator(seed, Box::new(noisy_generator(seed)));
        world.set_chunk_budget(16);

        let watched = [
            ChunkPos::new(0, 0, 0),
            ChunkPos::new(-3, 2, 11),
            ChunkPos::new(57, -4, -19),
        ];
        let before: Vec<Vec<Block>> = watched
            .iter()
            .map(|p| world.chunk(*p).blocks().to_vec())
            .collect();

        // Pressure: 300 fresh chunks against a 16-chunk budget.
        sweep(&mut world, 300, &watched);

        let residency = world.chunk_residency();
        assert!(
            residency.evicted > 0,
            "seed {seed}: sweep should have evicted (residency {residency:?})"
        );
        assert!(
            residency.resident <= residency.budget + residency.edited,
            "seed {seed}: store stayed over budget ({residency:?})"
        );

        for (p, expect) in watched.iter().zip(&before) {
            let after = world.chunk(*p).blocks().to_vec();
            assert_eq!(
                &after, expect,
                "seed {seed}: chunk {p:?} did not regenerate byte-identically"
            );
        }
    }
}

#[test]
fn region_hash_is_independent_of_the_budget() {
    // The end-to-end statement: a world that can hold everything and a world
    // thrashing at a 4-chunk budget answer every query identically.
    let vol = Volume::new(Vec3i::new(-40, -4, -40), Vec3i::new(24, 4, 24));

    let mut roomy = HostWorld::with_generator(99, Box::new(noisy_generator(99)));
    roomy.set_chunk_budget(100_000);
    let mut cramped = HostWorld::with_generator(99, Box::new(noisy_generator(99)));
    cramped.set_chunk_budget(4);

    assert_eq!(roomy.region_hash(vol), cramped.region_hash(vol));
    assert!(cramped.chunk_residency().evicted > 0, "budget 4 must evict");
    assert_eq!(roomy.chunk_residency().evicted, 0, "budget 100k must not");
}

#[test]
fn an_edited_chunk_survives_eviction_pressure() {
    let mut world = HostWorld::with_generator(3, Box::new(noisy_generator(3)));
    world.set_chunk_budget(8);

    let edit_at = Vec3i::new(5, 1, 5);
    let edited_chunk = ChunkPos::from_world_voxel(edit_at.x, edit_at.y, edit_at.z);
    set_block(&mut world, edit_at, "dc:wood");
    assert_eq!(world.block_at(edit_at), Block::Wood);
    assert_eq!(world.chunk_residency().edited, 1);

    // 400 fresh chunks against an 8-chunk budget: everything droppable goes.
    sweep(&mut world, 400, &[edited_chunk]);

    let residency = world.chunk_residency();
    assert_eq!(residency.edited, 1, "the edited chunk was unpinned");
    assert!(
        residency.evicted >= 300,
        "pressure did not evict: {residency:?}"
    );
    // The edit is still there — and it is the *edit*, not regenerated terrain.
    assert_eq!(world.block_at(edit_at), Block::Wood);
    assert_eq!(world.chunk(edited_chunk).get(5, 1, 5), Block::Wood);
}

#[test]
fn edited_chunks_accumulate_beyond_the_budget() {
    // Documented, deliberate behaviour (journal/0051): edits are unbounded
    // until the save layer can spill them. This test exists so that the day
    // someone makes edited chunks evictable, it fails loudly.
    let mut world = HostWorld::with_generator(11, Box::new(noisy_generator(11)));
    world.set_chunk_budget(4);

    for i in 0..40 {
        set_block(&mut world, Vec3i::new(i * 32, 1, 0), "dc:wood");
    }
    let residency = world.chunk_residency();
    assert_eq!(residency.edited, 40);
    assert!(residency.resident >= 40);
    for i in 0..40 {
        assert_eq!(world.block_at(Vec3i::new(i * 32, 1, 0)), Block::Wood);
    }
}

#[test]
fn a_no_op_write_does_not_pin_a_chunk() {
    // Writing the value that is already there changes nothing the generator
    // cannot reproduce, so it must not cost a permanent chunk of RAM.
    let mut world = HostWorld::with_generator(5, Box::new(noisy_generator(5)));
    let pos = Vec3i::new(0, 0, 0);
    let existing = world.block_at(pos);
    let name = match existing {
        Block::Air => "dc:air",
        Block::Stone => "dc:stone",
        Block::Dirt => "dc:dirt",
        Block::Granite => "dc:granite",
        _ => "dc:basalt",
    };
    set_block(&mut world, pos, name);
    assert_eq!(world.chunk_residency().edited, 0);
}

#[test]
fn the_builtin_world_evicts_too() {
    // The default (generator-less) world is the S5 conformance reference; the
    // policy must hold there as well.
    let mut world = HostWorld::new(17);
    world.set_chunk_budget(8);
    let watched = ChunkPos::new(0, -1, 0);
    let before = world.chunk(watched).blocks().to_vec();
    sweep(&mut world, 200, &[watched]);
    assert!(world.chunk_residency().evicted > 0);
    assert_eq!(world.chunk(watched).blocks().to_vec(), before);
}
