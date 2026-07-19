//! Character-surface conformance (character-MCP milestone): the character
//! primitive lives behind the one door — spawn/controller verbs are ordered,
//! receipted commands; senses are grant-scoped diegetic queries; the whole
//! thing replays deterministically from the seed + command log.

use dc_core::{Block, CHUNK_SIZE_USIZE, Chunk, ChunkPos};

use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, Grant, HostWorld,
    Payload, QueryData, QueryResult, RejectReason, Vec3i, ids, payload, payload::Vec3f,
};

/// Flat synthetic ground: solid stone for chunk-y < 0 (surface at voxel y=0),
/// air above — predictable footing for movement assertions.
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

fn slab_world(seed: u64) -> HostWorld {
    HostWorld::with_generator(seed, Box::new(slab_generator))
}

/// The broad parent/dev posture: may spawn characters and control any.
fn dev_source() -> (ConsumerId, CapabilityToken) {
    (
        ConsumerId::new(ConsumerKind::McpSession, "dev"),
        CapabilityToken::new(vec![
            Grant::EntitySpawn,
            Grant::CharacterControl { character: None },
            Grant::WorldWrite { volume: None },
        ]),
    )
}

/// A character session: attenuated to exactly one character, nothing else.
fn session_source(character: &str) -> (ConsumerId, CapabilityToken) {
    let (_, parent) = dev_source();
    let token = parent
        .attenuate(vec![Grant::CharacterControl {
            character: Some(character.into()),
        }])
        .expect("parent covers any character");
    (
        ConsumerId::new(ConsumerKind::McpSession, format!("character-{character}")),
        token,
    )
}

fn env(source: &(ConsumerId, CapabilityToken), payload: Payload) -> CommandEnvelope {
    CommandEnvelope {
        id: payload.command_id().to_string(),
        source: source.0.clone(),
        grant: source.1.clone(),
        payload,
        target_tick: None,
        txn: None,
    }
}

/// Submit + tick, returning the command's result.
fn run(
    world: &mut HostWorld,
    source: &(ConsumerId, CapabilityToken),
    payload: Payload,
) -> CommandResult {
    match world.submit(env(source, payload)) {
        Err(entry) => entry.receipt.result,
        Ok(ack) => {
            let receipts = world.tick();
            receipts
                .iter()
                .find(|e| e.source == source.0 && e.consumer_seq == ack.consumer_seq)
                .expect("receipt at tick boundary")
                .receipt
                .result
                .clone()
        }
    }
}

fn spawn_scout(world: &mut HostWorld, pos: Vec3f) {
    let dev = dev_source();
    let result = run(
        world,
        &dev,
        Payload::SpawnCharacter(payload::SpawnCharacter {
            name: "scout".into(),
            pos,
        }),
    );
    match result {
        CommandResult::Ok(fx) => assert_eq!(fx.characters_spawned, vec!["scout".to_string()]),
        CommandResult::Rejected(r) => panic!("spawn rejected: {r}"),
    }
}

fn pose(world: &mut HostWorld, source: &(ConsumerId, CapabilityToken), name: &str) -> QueryResult {
    world
        .query(&env(
            source,
            Payload::CharacterPose(payload::CharacterPose {
                character: name.into(),
            }),
        ))
        .result
}

#[test]
fn spawned_character_falls_walks_and_jumps_with_real_collision() {
    let mut world = slab_world(7);
    spawn_scout(&mut world, Vec3f::new(0.3, 2.0, 0.3));
    let session = session_source("scout");

    // Gravity settles the body onto the slab surface (voxel y=0 => 0 m).
    for _ in 0..60 {
        world.tick();
    }
    let c = world.character("scout").expect("exists").clone();
    assert!(c.on_ground, "settled onto the slab");
    assert!(
        c.pos_m.y.abs() < 1e-9,
        "feet at the surface, got {}",
        c.pos_m.y
    );

    // A move intent through the session's own grant walks the body.
    let result = run(
        &mut world,
        &session,
        Payload::SetMoveIntent(payload::SetMoveIntent {
            character: "scout".into(),
            dx: 1.0,
            dz: 0.0,
            speed: 1.0,
        }),
    );
    assert!(result.is_ok(), "own-character control allowed: {result:?}");
    for _ in 0..20 {
        world.tick();
    }
    let walked = world.character("scout").unwrap().pos_m.x - 0.3;
    // 21 ticks of walking at 4.5 m/s * 1/20 s (the intent tick moved too).
    assert!(walked > 4.0, "walked {walked} m east");

    // Stop, then jump: airborne next tick, back on the ground soon after.
    run(
        &mut world,
        &session,
        Payload::SetMoveIntent(payload::SetMoveIntent {
            character: "scout".into(),
            dx: 0.0,
            dz: 0.0,
            speed: 0.0,
        }),
    );
    let result = run(
        &mut world,
        &session,
        Payload::Jump(payload::Jump {
            character: "scout".into(),
        }),
    );
    assert!(result.is_ok());
    let mut apex: f64 = 0.0;
    for _ in 0..40 {
        world.tick();
        apex = apex.max(world.character("scout").unwrap().pos_m.y);
    }
    assert!(apex > 0.5, "jump apex {apex} m");
    assert!(world.character("scout").unwrap().on_ground, "landed");
}

#[test]
fn character_session_grant_is_a_cage() {
    let mut world = slab_world(7);
    spawn_scout(&mut world, Vec3f::new(0.3, 0.0, 0.3));
    // A second character the session must NOT reach.
    let dev = dev_source();
    let result = run(
        &mut world,
        &dev,
        Payload::SpawnCharacter(payload::SpawnCharacter {
            name: "other".into(),
            pos: Vec3f::new(5.0, 0.0, 5.0),
        }),
    );
    assert!(result.is_ok());
    let session = session_source("scout");

    // 1. Cannot set blocks.
    let result = run(
        &mut world,
        &session,
        Payload::SetBlock(payload::SetBlock {
            pos: Vec3i::new(0, 0, 0),
            block: "dc:stone".into(),
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{result:?}"
    );

    // 2. Cannot spawn entities (or characters).
    let result = run(
        &mut world,
        &session,
        Payload::EntitySpawn(payload::EntitySpawn {
            kind: "dc:deer".into(),
            pos: Vec3f::new(0.0, 1.0, 0.0),
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{result:?}"
    );
    let result = run(
        &mut world,
        &session,
        Payload::SpawnCharacter(payload::SpawnCharacter {
            name: "minion".into(),
            pos: Vec3f::new(0.0, 1.0, 0.0),
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{result:?}"
    );

    // 3. Cannot define items.
    let result = run(
        &mut world,
        &session,
        Payload::DefineItem(payload::DefineItem {
            name: "dev:contraband".into(),
            display_name: "Contraband".into(),
            description: None,
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{result:?}"
    );

    // 4. Cannot control or sense a different character.
    let result = run(
        &mut world,
        &session,
        Payload::SetMoveIntent(payload::SetMoveIntent {
            character: "other".into(),
            dx: 1.0,
            dz: 0.0,
            speed: 1.0,
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{result:?}"
    );
    assert!(
        matches!(
            pose(&mut world, &session, "other"),
            QueryResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "foreign pose must be denied"
    );

    // 5. No world reads either — the diegetic senses are the only eyes.
    let scan = world.query(&env(
        &session,
        Payload::ScanRegion(payload::ScanRegion {
            min: Vec3i::new(0, 0, 0),
            max: Vec3i::new(1, 1, 1),
        }),
    ));
    assert!(
        matches!(
            scan.result,
            QueryResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "{:?}",
        scan.result
    );

    // ...while its own body remains fully accessible.
    assert!(matches!(
        pose(&mut world, &session, "scout"),
        QueryResult::Ok(QueryData::CharacterPose { .. })
    ));

    // And the dev/broad token can still do all of it.
    assert!(matches!(
        pose(&mut world, &dev, "other"),
        QueryResult::Ok(QueryData::CharacterPose { .. })
    ));
    let result = run(
        &mut world,
        &dev,
        Payload::SetBlock(payload::SetBlock {
            pos: Vec3i::new(9, 3, 9),
            block: "dc:stone".into(),
        }),
    );
    assert!(result.is_ok());
}

#[test]
fn senses_are_embodied_and_range_capped() {
    let mut world = slab_world(7);
    spawn_scout(&mut world, Vec3f::new(0.3, 0.0, 0.3));
    let dev = dev_source();
    let session = session_source("scout");

    // Default gaze (yaw 0 = -Z) with nothing ahead over a flat slab: a miss,
    // even at the (capped) max range.
    let miss = world.query(&env(
        &session,
        Payload::SenseRaycast(payload::SenseRaycast {
            character: "scout".into(),
            dir: None,
            max_distance_m: Some(500.0), // clamped to 50
        }),
    ));
    match miss.result {
        QueryResult::Ok(QueryData::CharacterRaycast { hit, .. }) => assert!(!hit),
        other => panic!("{other:?}"),
    }

    // A stone block at voxel (0, 2, -5) sits dead ahead of the eyes
    // (eye height 1.62 m / 0.6 m per voxel => voxel y=2).
    let result = run(
        &mut world,
        &dev,
        Payload::SetBlock(payload::SetBlock {
            pos: Vec3i::new(0, 2, -5),
            block: "dc:stone".into(),
        }),
    );
    assert!(result.is_ok());
    let hit = world.query(&env(
        &session,
        Payload::SenseRaycast(payload::SenseRaycast {
            character: "scout".into(),
            dir: None,
            max_distance_m: None,
        }),
    ));
    match hit.result {
        QueryResult::Ok(QueryData::CharacterRaycast {
            hit: true,
            voxel: Some(v),
            block: Some(block),
            normal: Some(n),
            distance_m: Some(d),
        }) => {
            assert_eq!((v.x, v.y, v.z), (0, 2, -5));
            assert_eq!(block, "dc:stone");
            assert_eq!((n.x, n.y, n.z), (0, 0, 1), "entered through the +Z face");
            // Eye z = 0.3 m; block entry face at voxel z=-4 => -2.4 m.
            assert!((d - 2.7).abs() < 1e-6, "distance {d} m");
        }
        other => panic!("{other:?}"),
    }
    // A shorter leash makes the same wall invisible.
    let leashed = world.query(&env(
        &session,
        Payload::SenseRaycast(payload::SenseRaycast {
            character: "scout".into(),
            dir: None,
            max_distance_m: Some(2.0),
        }),
    ));
    match leashed.result {
        QueryResult::Ok(QueryData::CharacterRaycast { hit, .. }) => assert!(!hit),
        other => panic!("{other:?}"),
    }

    // Surroundings: radius 1 around the feet voxel (0, 0, 0) — 27 voxels,
    // slab below, air at and above the feet.
    let near = world.query(&env(
        &session,
        Payload::SenseSurroundings(payload::SenseSurroundings {
            character: "scout".into(),
            radius: 1,
        }),
    ));
    match near.result {
        QueryResult::Ok(QueryData::Region {
            min,
            max,
            palette,
            indices,
        }) => {
            assert_eq!((min.x, min.y, min.z), (-1, -1, -1));
            assert_eq!((max.x, max.y, max.z), (1, 1, 1));
            assert_eq!(indices.len(), 27);
            // Scan order is y-major: the first 9 are the stone slab.
            assert_eq!(palette[0], "dc:stone");
            assert!(indices[..9].iter().all(|&i| i == 0));
            assert!(palette.contains(&"dc:air".to_string()));
        }
        other => panic!("{other:?}"),
    }
    // Beyond the sense radius: refused, not served.
    let far = world.query(&env(
        &session,
        Payload::SenseSurroundings(payload::SenseSurroundings {
            character: "scout".into(),
            radius: 17,
        }),
    ));
    assert!(
        matches!(
            far.result,
            QueryResult::Rejected(RejectReason::PayloadInvalid { .. })
        ),
        "{:?}",
        far.result
    );

    // Proprioception knows when the eyes are buried: entomb the character.
    let result = run(
        &mut world,
        &dev,
        Payload::Fill(payload::Fill {
            min: Vec3i::new(-2, 0, -2),
            max: Vec3i::new(2, 4, 2),
            block: "dc:stone".into(),
        }),
    );
    assert!(result.is_ok());
    match pose(&mut world, &session, "scout") {
        QueryResult::Ok(QueryData::CharacterPose { eye_in_solid, .. }) => {
            assert!(eye_in_solid, "buried eyes must report eye_in_solid");
        }
        other => panic!("{other:?}"),
    }
}

/// The determinism payoff (docs/API.md principle 3), embodied: the same
/// scripted movement session against the same seed lands on the bit-identical
/// final pose; a different seed's terrain produces a different trajectory.
#[test]
fn scripted_movement_session_replays_to_identical_pose() {
    let final_pose = |seed: u64| -> dc_api::CharacterState {
        // The built-in hill-field terrain (seed-dependent footing).
        let mut world = HostWorld::new(seed);
        spawn_scout(&mut world, Vec3f::new(0.3, 2.0, 0.3));
        let session = session_source("scout");
        let intent = |dx: f64, dz: f64, speed: f64| {
            Payload::SetMoveIntent(payload::SetMoveIntent {
                character: "scout".into(),
                dx,
                dz,
                speed,
            })
        };
        // A little choreography: walk east, turn, look around, jump, walk on.
        run(&mut world, &session, intent(1.0, 0.0, 1.0));
        for _ in 0..30 {
            world.tick();
        }
        run(&mut world, &session, intent(0.0, 1.0, 0.6));
        run(
            &mut world,
            &session,
            Payload::SetLook(payload::SetLook {
                character: "scout".into(),
                yaw: 1.2,
                pitch: -0.4,
            }),
        );
        for _ in 0..20 {
            world.tick();
        }
        run(
            &mut world,
            &session,
            Payload::Jump(payload::Jump {
                character: "scout".into(),
            }),
        );
        for _ in 0..40 {
            world.tick();
        }
        world.character("scout").expect("still exists").clone()
    };

    let a = final_pose(1337);
    let b = final_pose(1337);
    assert_eq!(a, b, "same seed + same command log = identical final pose");
    // Bit-level, not just approximate:
    assert_eq!(a.pos_m.x.to_bits(), b.pos_m.x.to_bits());
    assert_eq!(a.pos_m.y.to_bits(), b.pos_m.y.to_bits());
    assert_eq!(a.pos_m.z.to_bits(), b.pos_m.z.to_bits());

    // Another seed's ground is elsewhere: the trajectory diverges.
    let c = final_pose(1338);
    assert_ne!(
        (
            a.pos_m.x.to_bits(),
            a.pos_m.y.to_bits(),
            a.pos_m.z.to_bits()
        ),
        (
            c.pos_m.x.to_bits(),
            c.pos_m.y.to_bits(),
            c.pos_m.z.to_bits()
        ),
        "different seed diverges"
    );
}

#[test]
fn spawn_validates_names_and_uniqueness() {
    let mut world = slab_world(7);
    let dev = dev_source();
    for bad in ["", "Bad Name", "a/b", &"x".repeat(65)] {
        let result = run(
            &mut world,
            &dev,
            Payload::SpawnCharacter(payload::SpawnCharacter {
                name: bad.into(),
                pos: Vec3f::new(0.0, 0.0, 0.0),
            }),
        );
        assert!(
            matches!(
                result,
                CommandResult::Rejected(RejectReason::PayloadInvalid { .. })
            ),
            "{bad:?}: {result:?}"
        );
    }
    spawn_scout(&mut world, Vec3f::new(0.0, 0.0, 0.0));
    let result = run(
        &mut world,
        &dev,
        Payload::SpawnCharacter(payload::SpawnCharacter {
            name: "scout".into(),
            pos: Vec3f::new(1.0, 0.0, 1.0),
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::AlreadyDefined { .. })
        ),
        "{result:?}"
    );
    // Verbs against a character that does not exist name the real problem.
    let result = run(
        &mut world,
        &dev,
        Payload::Jump(payload::Jump {
            character: "ghost".into(),
        }),
    );
    assert!(
        matches!(
            result,
            CommandResult::Rejected(RejectReason::UnknownCharacter { .. })
        ),
        "{result:?}"
    );
    // (The session form is indistinguishable: a session for `ghost` holds
    // character.control(ghost), which fails on UnknownCharacter, revealing
    // nothing about other characters.)
    assert_eq!(ids::CHARACTER_JUMP, "dc:character/jump");
}

/// Parametric crouch is sim state (bodies.md § determinism firewall): a scripted
/// session that crouches, walks, and stands replays to a bit-identical final
/// state — posture included — over the same seed.
#[test]
fn posture_transitions_replay_identically() {
    let final_state = |seed: u64| -> dc_api::CharacterState {
        let mut world = slab_world(seed);
        spawn_scout(&mut world, Vec3f::new(0.3, 2.0, 0.3));
        let session = session_source("scout");
        for _ in 0..40 {
            world.tick(); // settle onto the slab
        }
        let posture = |p: &str| {
            Payload::SetPosture(payload::SetPosture {
                character: "scout".into(),
                posture: p.into(),
            })
        };
        run(&mut world, &session, posture("crouching"));
        run(
            &mut world,
            &session,
            Payload::SetMoveIntent(payload::SetMoveIntent {
                character: "scout".into(),
                dx: 1.0,
                dz: 0.0,
                speed: 1.0,
            }),
        );
        for _ in 0..20 {
            world.tick();
        }
        run(&mut world, &session, posture("standing"));
        for _ in 0..20 {
            world.tick();
        }
        world.character("scout").expect("still exists").clone()
    };
    let a = final_state(7);
    let b = final_state(7);
    assert_eq!(a, b, "same seed + posture script = identical final state");
    assert_eq!(a.pos_m.x.to_bits(), b.pos_m.x.to_bits());
    assert_eq!(a.pos_m.y.to_bits(), b.pos_m.y.to_bits());
    assert_eq!(a.posture, b.posture);
}

/// The crouch collider is real, and standing back up is guarded: a low ceiling
/// that a crouched body clears blocks the stand-up, and clearing it lets the
/// body rise. The sim half of bodies.md's crouch example.
#[test]
fn stand_up_is_blocked_under_a_low_ceiling() {
    let mut world = slab_world(7);
    let dev = dev_source();
    spawn_scout(&mut world, Vec3f::new(0.3, 2.0, 0.3));
    let session = session_source("scout");
    for _ in 0..40 {
        world.tick();
    }
    assert!(world.character("scout").unwrap().on_ground, "settled");

    let posture = |p: &str| {
        Payload::SetPosture(payload::SetPosture {
            character: "scout".into(),
            posture: p.into(),
        })
    };
    // Crouch is always allowed.
    assert!(run(&mut world, &session, posture("crouching")).is_ok());
    assert_eq!(
        world.character("scout").unwrap().posture,
        dc_api::Posture::Crouching
    );

    // A low ceiling over the footprint at voxel y=2 (1.2–1.8 m): a crouched body
    // (~1.08 m) clears it, a standing one (1.8 m) does not.
    let ceiling = |block: &str| {
        Payload::Fill(payload::Fill {
            min: Vec3i::new(-1, 2, -1),
            max: Vec3i::new(1, 2, 1),
            block: block.into(),
        })
    };
    assert!(run(&mut world, &dev, ceiling("dc:stone")).is_ok());
    for _ in 0..5 {
        world.tick();
    }
    // The crouched body still fits and stays put.
    assert_eq!(
        world.character("scout").unwrap().posture,
        dc_api::Posture::Crouching
    );

    // Standing up is refused — the taller collider would embed in the ceiling.
    let r = run(&mut world, &session, posture("standing"));
    assert!(
        matches!(
            r,
            CommandResult::Rejected(RejectReason::PostureBlocked { .. })
        ),
        "stand-up under a low ceiling must be blocked: {r:?}"
    );
    assert_eq!(
        world.character("scout").unwrap().posture,
        dc_api::Posture::Crouching,
        "stays crouched after a blocked stand-up"
    );

    // Clear the ceiling; now standing succeeds.
    assert!(run(&mut world, &dev, ceiling("dc:air")).is_ok());
    let r = run(&mut world, &session, posture("standing"));
    assert!(r.is_ok(), "with the ceiling gone, standing succeeds: {r:?}");
    assert_eq!(
        world.character("scout").unwrap().posture,
        dc_api::Posture::Standing
    );
}
