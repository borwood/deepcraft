//! Reference-host conformance: tick quantization, total order, txn atomicity,
//! capability enforcement, event delivery, replay determinism.

use dc_api::{
    CapabilityToken, CommandEnvelope, CommandResult, ConsumerId, ConsumerKind, EventKind,
    GameEvent, Grant, HostWorld, Payload, QueryData, QueryResult, RejectReason, TxnId, Vec3i,
    Volume,
    payload::{
        DefineItem, EntitySpawn, EventsPoll, EventsSubscribe, Fill, GetBlock, ScanRegion, SetBlock,
        Vec3f,
    },
};

fn v(x: i64, y: i64, z: i64) -> Vec3i {
    Vec3i::new(x, y, z)
}

fn all_powers() -> CapabilityToken {
    CapabilityToken::new(vec![
        Grant::WorldRead { volume: None },
        Grant::WorldWrite { volume: None },
        Grant::EntitySpawn,
        Grant::RegistryDefine {
            namespace: "test".into(),
        },
        Grant::EventsSubscribe,
    ])
}

fn envelope(source: &ConsumerId, grant: &CapabilityToken, payload: Payload) -> CommandEnvelope {
    CommandEnvelope {
        id: payload.command_id().to_string(),
        source: source.clone(),
        grant: grant.clone(),
        payload,
        target_tick: None,
        txn: None,
    }
}

fn set_block(
    source: &ConsumerId,
    grant: &CapabilityToken,
    pos: Vec3i,
    block: &str,
) -> CommandEnvelope {
    envelope(
        source,
        grant,
        Payload::SetBlock(SetBlock {
            pos,
            block: block.into(),
        }),
    )
}

fn get_block(
    world: &mut HostWorld,
    source: &ConsumerId,
    grant: &CapabilityToken,
    pos: Vec3i,
) -> String {
    let receipt = world.query(&envelope(
        source,
        grant,
        Payload::GetBlock(GetBlock { pos }),
    ));
    match receipt.result {
        QueryResult::Ok(QueryData::Block { block }) => block,
        other => panic!("get_block failed: {other:?}"),
    }
}

#[test]
fn commands_are_tick_quantized() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    let token = all_powers();
    let pos = v(2, 5, 2);

    let ack = world
        .submit(set_block(&src, &token, pos, "dc:stone"))
        .expect("queued");
    assert_eq!(ack.consumer_seq, 0);
    assert_eq!(ack.scheduled_tick, 1);
    // Not applied yet: the world still reflects tick 0.
    assert_eq!(get_block(&mut world, &src, &token, pos), "dc:air");

    let receipts = world.tick();
    assert_eq!(world.current_tick(), 1);
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].receipt.tick_applied, 1);
    assert!(receipts[0].receipt.result.is_ok());
    assert_eq!(get_block(&mut world, &src, &token, pos), "dc:stone");
}

#[test]
fn target_tick_defers_and_past_targets_reject() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    let token = all_powers();
    let pos = v(0, 5, 0);

    let mut env = set_block(&src, &token, pos, "dc:wood");
    env.target_tick = Some(3);
    world.submit(env).expect("queued");
    world.tick(); // 1
    world.tick(); // 2
    assert_eq!(get_block(&mut world, &src, &token, pos), "dc:air");
    let receipts = world.tick(); // 3
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].receipt.tick_applied, 3);
    assert_eq!(get_block(&mut world, &src, &token, pos), "dc:wood");

    let mut past = set_block(&src, &token, pos, "dc:dirt");
    past.target_tick = Some(2);
    let entry = world.submit(past).expect_err("past tick rejects");
    assert!(matches!(
        entry.receipt.result,
        CommandResult::Rejected(RejectReason::TargetTickInPast {
            target: 2,
            current: 3
        })
    ));
}

#[test]
fn total_order_is_priority_then_consumer_then_seq() {
    let mut world = HostWorld::new(7);
    let token = all_powers();
    let player = ConsumerId::new(ConsumerKind::Player, "alice");
    let plugin = ConsumerId::new(ConsumerKind::Plugin, "builder");
    let sched = ConsumerId::new(ConsumerKind::Scheduler, "cron");
    let pos = v(1, 5, 1);

    // Submitted in "wrong" order: scheduler first, then plugin, then player.
    world
        .submit(set_block(&sched, &token, pos, "dc:dirt"))
        .unwrap();
    world
        .submit(set_block(&plugin, &token, pos, "dc:wood"))
        .unwrap();
    world
        .submit(set_block(&player, &token, pos, "dc:stone"))
        .unwrap();
    let receipts = world.tick();

    // Applied player -> plugin -> scheduler; receipts are in apply order.
    let order: Vec<&ConsumerId> = receipts.iter().map(|r| &r.source).collect();
    assert_eq!(order, vec![&player, &plugin, &sched]);
    let seqs: Vec<u64> = receipts.iter().map(|r| r.receipt.seq).collect();
    assert!(seqs.windows(2).all(|w| w[0] < w[1]), "global seq monotone");
    // Last writer in the total order wins the voxel.
    assert_eq!(get_block(&mut world, &player, &token, pos), "dc:dirt");

    // Effects chain: player saw air->stone, plugin stone->wood, sched wood->dirt.
    let changes: Vec<(String, String)> = receipts
        .iter()
        .map(|r| match &r.receipt.result {
            CommandResult::Ok(fx) => (
                fx.blocks_changed[0].from.clone(),
                fx.blocks_changed[0].to.clone(),
            ),
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        changes,
        vec![
            ("dc:air".into(), "dc:stone".into()),
            ("dc:stone".into(), "dc:wood".into()),
            ("dc:wood".into(), "dc:dirt".into()),
        ]
    );
}

#[test]
fn same_class_consumers_order_by_id_then_seq() {
    let mut world = HostWorld::new(7);
    let token = all_powers();
    let a = ConsumerId::new(ConsumerKind::Plugin, "aaa");
    let b = ConsumerId::new(ConsumerKind::Plugin, "bbb");
    let pos = v(3, 5, 3);
    world.submit(set_block(&b, &token, pos, "dc:wood")).unwrap();
    world
        .submit(set_block(&a, &token, pos, "dc:stone"))
        .unwrap();
    world.tick();
    // "aaa" sorts before "bbb" regardless of submission order.
    assert_eq!(get_block(&mut world, &a, &token, pos), "dc:wood");
}

#[test]
fn deny_by_default_and_volume_scoped_writes() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "scoped");
    let yard = Volume::new(v(0, 0, 0), v(15, 15, 15));
    let scoped = CapabilityToken::new(vec![
        Grant::WorldWrite { volume: Some(yard) },
        Grant::WorldRead { volume: Some(yard) },
    ]);
    let none = CapabilityToken::none();
    let admin = ConsumerId::new(ConsumerKind::Editor, "admin");
    let admin_token = all_powers();

    // Empty token: rejected.
    world
        .submit(set_block(&src, &none, v(1, 1, 1), "dc:stone"))
        .unwrap();
    // Inside the volume: ok.
    world
        .submit(set_block(&src, &scoped, v(1, 1, 1), "dc:stone"))
        .unwrap();
    // Outside the volume: rejected.
    world
        .submit(set_block(&src, &scoped, v(16, 1, 1), "dc:stone"))
        .unwrap();
    // Fill straddling the boundary: rejected atomically (nothing applied).
    world
        .submit(envelope(
            &src,
            &scoped,
            Payload::Fill(Fill {
                min: v(14, 5, 14),
                max: v(17, 5, 17),
                block: "dc:stone".into(),
            }),
        ))
        .unwrap();
    let receipts = world.tick();
    assert!(matches!(
        receipts[0].receipt.result,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
    assert!(receipts[1].receipt.result.is_ok());
    assert!(matches!(
        receipts[2].receipt.result,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
    assert!(matches!(
        receipts[3].receipt.result,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
    assert_eq!(
        get_block(&mut world, &admin, &admin_token, v(1, 1, 1)),
        "dc:stone"
    );
    assert_eq!(
        get_block(&mut world, &admin, &admin_token, v(16, 1, 1)),
        "dc:air"
    );
    // The straddling fill wrote nothing, not even the in-bounds corner.
    assert_eq!(
        get_block(&mut world, &admin, &admin_token, v(14, 5, 14)),
        "dc:air"
    );

    // Reads are scoped too.
    let outside = world.query(&envelope(
        &src,
        &scoped,
        Payload::ScanRegion(ScanRegion {
            min: v(10, 0, 10),
            max: v(17, 3, 17),
        }),
    ));
    assert!(matches!(
        outside.result,
        QueryResult::Rejected(RejectReason::MissingCapability { .. })
    ));
}

#[test]
fn namespace_ownership_is_enforced() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "foo-plugin");
    let token = CapabilityToken::new(vec![Grant::RegistryDefine {
        namespace: "foo".into(),
    }]);

    let define = |name: &str| {
        Payload::DefineItem(DefineItem {
            name: name.into(),
            display_name: "X".into(),
            description: None,
        })
    };
    world
        .submit(envelope(&src, &token, define("foo:hammer")))
        .unwrap();
    world
        .submit(envelope(&src, &token, define("bar:hammer")))
        .unwrap();
    world
        .submit(envelope(&src, &token, define("foo:hammer")))
        .unwrap(); // dup
    world
        .submit(envelope(&src, &token, define("nonamespace")))
        .unwrap();
    let receipts = world.tick();
    assert!(receipts[0].receipt.result.is_ok());
    assert!(matches!(
        receipts[1].receipt.result,
        CommandResult::Rejected(RejectReason::MissingCapability { .. })
    ));
    assert!(matches!(
        receipts[2].receipt.result,
        CommandResult::Rejected(RejectReason::AlreadyDefined { .. })
    ));
    assert!(matches!(
        receipts[3].receipt.result,
        CommandResult::Rejected(RejectReason::PayloadInvalid { .. })
    ));
    assert!(world.item("foo:hammer").is_some());
    assert!(world.item("bar:hammer").is_none());
}

#[test]
fn transactions_are_all_or_nothing() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "txn");
    let token = all_powers();
    let txn = Some(TxnId(1));

    // Good txn: two writes commit together.
    let mut a = set_block(&src, &token, v(0, 5, 0), "dc:stone");
    a.txn = txn;
    let mut b = set_block(&src, &token, v(1, 5, 0), "dc:stone");
    b.txn = txn;
    world.submit(a).unwrap();
    world.submit(b).unwrap();
    let receipts = world.tick();
    assert!(receipts.iter().all(|r| r.receipt.result.is_ok()));

    // Bad txn: a write + an unknown block. Both rejected, first rolled back.
    let txn2 = Some(TxnId(2));
    let mut c = set_block(&src, &token, v(2, 5, 0), "dc:stone");
    c.txn = txn2;
    let mut d = set_block(&src, &token, v(3, 5, 0), "dc:nosuchblock");
    d.txn = txn2;
    // Spawn + define in the same doomed txn to prove those roll back too.
    let mut e = envelope(
        &src,
        &token,
        Payload::EntitySpawn(EntitySpawn {
            kind: "dc:deer".into(),
            pos: Vec3f::new(2.5, 6.0, 0.5),
        }),
    );
    e.txn = txn2;
    let mut f = envelope(
        &src,
        &token,
        Payload::DefineItem(DefineItem {
            name: "test:doomed".into(),
            display_name: "Doomed".into(),
            description: None,
        }),
    );
    f.txn = txn2;
    world.submit(c).unwrap();
    world.submit(e).unwrap();
    world.submit(f).unwrap();
    world.submit(d).unwrap();
    let receipts = world.tick();
    assert_eq!(receipts.len(), 4);
    for r in &receipts {
        assert!(
            matches!(
                r.receipt.result,
                CommandResult::Rejected(RejectReason::TxnAborted { .. })
            ),
            "expected TxnAborted, got {:?}",
            r.receipt.result
        );
    }
    assert_eq!(get_block(&mut world, &src, &token, v(2, 5, 0)), "dc:air");
    assert!(world.entities().is_empty());
    assert!(world.item("test:doomed").is_none());
    // And events from the aborted txn were never delivered (see events test).
}

#[test]
fn events_flow_filter_and_drain() {
    let mut world = HostWorld::new(7);
    let token = all_powers();
    let me = ConsumerId::new(ConsumerKind::Plugin, "watcher");
    let other = ConsumerId::new(ConsumerKind::Editor, "editor");

    // Subscribe to BlockChanged inside a yard.
    let yard = Volume::new(v(0, 0, 0), v(7, 7, 7));
    world
        .submit(envelope(
            &me,
            &token,
            Payload::EventsSubscribe(EventsSubscribe {
                kinds: vec![EventKind::BlockChanged],
                volume: Some(yard),
            }),
        ))
        .unwrap();
    let receipts = world.tick();
    let sub_id = match &receipts[0].receipt.result {
        CommandResult::Ok(fx) => fx.subscriptions_created[0],
        other => panic!("subscribe failed: {other:?}"),
    };

    // In-yard write (delivered), out-of-yard write (filtered), item define
    // (wrong kind, filtered), and a doomed txn write (never delivered).
    world
        .submit(set_block(&other, &token, v(3, 3, 3), "dc:wood"))
        .unwrap();
    world
        .submit(set_block(&other, &token, v(20, 3, 3), "dc:wood"))
        .unwrap();
    world
        .submit(envelope(
            &other,
            &token,
            Payload::DefineItem(DefineItem {
                name: "test:noise".into(),
                display_name: "Noise".into(),
                description: None,
            }),
        ))
        .unwrap();
    let mut doomed = set_block(&other, &token, v(4, 4, 4), "dc:stone");
    doomed.txn = Some(TxnId(9));
    let mut doom = set_block(&other, &token, v(5, 5, 5), "dc:nosuchblock");
    doom.txn = Some(TxnId(9));
    world.submit(doomed).unwrap();
    world.submit(doom).unwrap();
    world.tick();

    let poll = |world: &mut HostWorld, source: &ConsumerId| {
        world.query(&envelope(
            source,
            &token,
            Payload::EventsPoll(EventsPoll {
                subscription: sub_id,
                max: None,
            }),
        ))
    };
    // Foreign consumers cannot poll my subscription.
    let foreign = poll(&mut world, &other);
    assert!(matches!(
        foreign.result,
        QueryResult::Rejected(RejectReason::UnknownSubscription { .. })
    ));
    // I get exactly the one in-yard committed change.
    let mine = poll(&mut world, &me);
    match mine.result {
        QueryResult::Ok(QueryData::Events { events, remaining }) => {
            assert_eq!(remaining, 0);
            assert_eq!(events.len(), 1);
            match &events[0] {
                GameEvent::BlockChanged {
                    pos,
                    to,
                    cause,
                    tick,
                    ..
                } => {
                    assert_eq!(*pos, v(3, 3, 3));
                    assert_eq!(to, "dc:wood");
                    assert_eq!(cause, &other);
                    assert_eq!(*tick, 2);
                }
                other => panic!("unexpected event {other:?}"),
            }
        }
        other => panic!("poll failed: {other:?}"),
    }
    // Drained: a second poll is empty.
    match poll(&mut world, &me).result {
        QueryResult::Ok(QueryData::Events { events, remaining }) => {
            assert!(events.is_empty());
            assert_eq!(remaining, 0);
        }
        other => panic!("poll failed: {other:?}"),
    }
}

#[test]
fn queries_read_last_completed_tick_and_never_mutate() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::McpSession, "dev");
    let token = all_powers();
    world
        .submit(set_block(&src, &token, v(0, 10, 0), "dc:stone"))
        .unwrap();
    // Query before the tick: still air, tick_observed = 0.
    let q = world.query(&envelope(
        &src,
        &token,
        Payload::GetBlock(GetBlock { pos: v(0, 10, 0) }),
    ));
    assert_eq!(q.tick_observed, 0);
    assert!(matches!(
        q.result,
        QueryResult::Ok(QueryData::Block { ref block }) if block == "dc:air"
    ));
    world.tick();
    let q = world.query(&envelope(
        &src,
        &token,
        Payload::GetBlock(GetBlock { pos: v(0, 10, 0) }),
    ));
    assert_eq!(q.tick_observed, 1);
    assert!(matches!(
        q.result,
        QueryResult::Ok(QueryData::Block { ref block }) if block == "dc:stone"
    ));
    // Submitting a query through the command path is rejected.
    let entry = world
        .submit(envelope(
            &src,
            &token,
            Payload::GetBlock(GetBlock { pos: v(0, 10, 0) }),
        ))
        .expect_err("queries do not queue");
    assert!(matches!(
        entry.receipt.result,
        CommandResult::Rejected(RejectReason::NotACommand { .. })
    ));
}

#[test]
fn scan_region_palette_is_first_appearance_order() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::McpSession, "dev");
    let token = all_powers();
    world
        .submit(set_block(&src, &token, v(0, 20, 0), "dc:wood"))
        .unwrap();
    world
        .submit(set_block(&src, &token, v(1, 20, 0), "dc:grass"))
        .unwrap();
    world.tick();
    let q = world.query(&envelope(
        &src,
        &token,
        Payload::ScanRegion(ScanRegion {
            min: v(0, 20, 0),
            max: v(2, 20, 0),
        }),
    ));
    match q.result {
        QueryResult::Ok(QueryData::Region {
            palette, indices, ..
        }) => {
            assert_eq!(palette, vec!["dc:wood", "dc:grass", "dc:air"]);
            assert_eq!(indices, vec![0, 1, 2]);
        }
        other => panic!("scan failed: {other:?}"),
    }
}

/// Determinism: same seed + same command log => identical world state and
/// identical receipts (docs/API.md principle 3: seed + command log = replay).
#[test]
fn replay_determinism() {
    let script = |world: &mut HostWorld| {
        let plugin = ConsumerId::new(ConsumerKind::Plugin, "p");
        let player = ConsumerId::new(ConsumerKind::Player, "alice");
        let token = all_powers();
        world
            .submit(envelope(
                &plugin,
                &token,
                Payload::Fill(Fill {
                    min: v(0, 1, 0),
                    max: v(4, 3, 4),
                    block: "dc:stone".into(),
                }),
            ))
            .unwrap();
        world
            .submit(set_block(&player, &token, v(2, 4, 2), "dc:wood"))
            .unwrap();
        world.tick();
        world
            .submit(envelope(
                &plugin,
                &token,
                Payload::EntitySpawn(EntitySpawn {
                    kind: "dc:deer".into(),
                    pos: Vec3f::new(2.5, 5.0, 2.5),
                }),
            ))
            .unwrap();
        world
            .submit(set_block(&plugin, &token, v(2, 4, 2), "dc:grass"))
            .unwrap();
        world.tick();
    };
    let region = Volume::new(v(-16, -8, -16), v(31, 15, 31));

    let mut w1 = HostWorld::new(42);
    let mut w2 = HostWorld::new(42);
    script(&mut w1);
    script(&mut w2);
    assert_eq!(w1.region_hash(region), w2.region_hash(region));
    assert_eq!(
        w1.receipt_log(),
        w2.receipt_log(),
        "receipts replay identically"
    );
    assert_eq!(w1.entities(), w2.entities());

    // A different seed changes the generated terrain (and thus the hash).
    let mut w3 = HostWorld::new(43);
    script(&mut w3);
    assert_ne!(w1.region_hash(region), w3.region_hash(region));
}

// ------------------------------------------------------------- get_contents --
//
// The look-at-voxel inspector's load-bearing query. It exposes the whole
// `VoxelContents` (structure / pore-fill / debris multisets, shape, occupancy)
// where `get_block` returns only the single classified name — the authority
// beside the summary (S-3 in reverse). Contents reach the host through the
// optional `set_contents_source` seam, parallel to the block generator.

/// A contents source that plants one known mixed voxel at world (1,2,3) — a
/// granite slab with an olivine pore inclusion and two eighths of loose sand —
/// and leaves the rest of the chunk empty.
fn planted_contents(cpos: dc_core::ChunkPos) -> Option<dc_core::ContentsGrid> {
    if cpos != dc_core::ChunkPos::new(0, 0, 0) {
        return None;
    }
    let mut dense = vec![dc_core::VoxelContents::EMPTY; dc_core::CHUNK_VOLUME];
    let mixed = dc_core::VoxelContents::new(
        dc_core::StructureShape::Slab,
        &[dc_core::MaterialId::GRANITE, dc_core::MaterialId::GRANITE],
        &[dc_core::MaterialId::OLIVINE],
        &[dc_core::MaterialId::SAND, dc_core::MaterialId::SAND],
    )
    .unwrap();
    dense[dc_core::Chunk::index(1, 2, 3)] = mixed;
    Some(dc_core::ContentsGrid::from_dense(&dense))
}

fn get_contents(world: &mut HostWorld, token: &CapabilityToken, pos: Vec3i) -> QueryData {
    let src = ConsumerId::new(ConsumerKind::Plugin, "inspector");
    let receipt = world.query(&envelope(
        &src,
        token,
        Payload::GetContents(dc_api::payload::GetContents { pos }),
    ));
    match receipt.result {
        QueryResult::Ok(data) => data,
        other => panic!("get_contents failed: {other:?}"),
    }
}

#[test]
fn get_contents_unpacks_the_full_composition() {
    let mut world = HostWorld::new(7);
    world.set_contents_source(Box::new(planted_contents));
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    let token = all_powers();
    let pos = v(1, 2, 3);
    // Make the stored block agree with the contents (as real worldgen would).
    world
        .submit(set_block(&src, &token, pos, "dc:granite"))
        .unwrap();
    world.tick();

    match get_contents(&mut world, &token, pos) {
        QueryData::Contents {
            block,
            classified,
            has_contents,
            contents,
        } => {
            assert!(has_contents);
            assert_eq!(block, "dc:granite");
            assert_eq!(classified, "dc:granite");
            assert_eq!(contents.shape, "slab");
            // 2 structure + 1 pore + 2 debris = 5 occupied eighths.
            assert_eq!(contents.solid_eighths, 5);
            assert_eq!(contents.free_eighths, 3);
            assert_eq!(contents.open_pores, 1); // slab cap 4, 2 struct + 1 pore
            assert_eq!(contents.free_debris_eighths, 2); // 8 - 4 reserved - 2 debris
            assert_eq!(contents.structure.len(), 1);
            assert_eq!(contents.structure[0].material, "dc:granite");
            assert_eq!(contents.structure[0].eighths, 2);
            assert_eq!(contents.pore_fill[0].material, "dc:olivine");
            assert_eq!(contents.pore_fill[0].eighths, 1);
            assert_eq!(contents.debris[0].material, "dc:sand");
            assert_eq!(contents.debris[0].eighths, 2);
        }
        other => panic!("expected Contents, got {other:?}"),
    }
}

#[test]
fn get_contents_without_a_source_is_block_only() {
    let mut world = HostWorld::new(7);
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    let token = all_powers();
    let pos = v(5, 5, 5);
    world
        .submit(set_block(&src, &token, pos, "dc:stone"))
        .unwrap();
    world.tick();

    match get_contents(&mut world, &token, pos) {
        QueryData::Contents {
            block,
            classified,
            has_contents,
            contents,
        } => {
            assert!(!has_contents, "no source installed");
            assert_eq!(block, "dc:stone");
            assert_eq!(classified, "dc:stone", "classified echoes the stored block");
            assert_eq!(contents, dc_api::payload::ContentsView::default());
        }
        other => panic!("expected Contents, got {other:?}"),
    }
}

#[test]
fn get_contents_surfaces_edit_divergence() {
    // An edit writes a block, never contents (the source is a pure function of
    // pos, blind to edits). Breaking the granite to air must show block=air but
    // the recorded contents still granite — the divergence made legible.
    let mut world = HostWorld::new(7);
    world.set_contents_source(Box::new(planted_contents));
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    let token = all_powers();
    let pos = v(1, 2, 3);
    world
        .submit(set_block(&src, &token, pos, "dc:air"))
        .unwrap();
    world.tick();

    match get_contents(&mut world, &token, pos) {
        QueryData::Contents {
            block,
            classified,
            has_contents,
            ..
        } => {
            assert!(has_contents);
            assert_eq!(block, "dc:air", "the edit is authoritative");
            assert_eq!(classified, "dc:granite", "contents still record granite");
        }
        other => panic!("expected Contents, got {other:?}"),
    }
}

#[test]
fn get_contents_requires_world_read() {
    let mut world = HostWorld::new(7);
    world.set_contents_source(Box::new(planted_contents));
    let src = ConsumerId::new(ConsumerKind::Plugin, "p");
    // A token with no world.read grant.
    let token = CapabilityToken::new(vec![Grant::EntitySpawn]);
    let receipt = world.query(&envelope(
        &src,
        &token,
        Payload::GetContents(dc_api::payload::GetContents { pos: v(1, 2, 3) }),
    ));
    assert!(
        matches!(
            receipt.result,
            QueryResult::Rejected(RejectReason::MissingCapability { .. })
        ),
        "get_contents must be gated by world.read: {:?}",
        receipt.result
    );
}
