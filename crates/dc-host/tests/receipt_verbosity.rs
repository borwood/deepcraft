//! Measurement for docs/API.md open question 4 (how much should a receipt
//! echo?): serialized sizes of every envelope and receipt in the demo
//! session, in postcard (the WASM boundary encoding) and JSON (the MCP
//! boundary encoding), with full effect lists vs count-only summaries.
//!
//! Run with `-- --nocapture` to read the table; numbers are recorded in
//! docs/spikes/S5-results.md.

use dc_api::envelope::{CommandResult, EffectsSummary, RejectReason, Tick};
use dc_api::{CommandEnvelope, ConsumerId, ConsumerKind, HostWorld, Payload, Vec3i, Volume};
use dc_host::demo_script;
use serde::Serialize;

/// A receipt shape that echoes only counts — the candidate alternative.
#[derive(Serialize)]
struct SummaryReceipt {
    seq: u64,
    tick_applied: Tick,
    result: SummaryResult,
}

#[derive(Serialize)]
enum SummaryResult {
    Ok(EffectsSummary),
    #[allow(dead_code)]
    Rejected(RejectReason),
}

fn postcard_len<T: Serialize>(value: &T) -> usize {
    postcard::to_allocvec(value).expect("postcard").len()
}

fn json_len<T: Serialize>(value: &T) -> usize {
    serde_json::to_vec(value).expect("json").len()
}

#[test]
fn measure_envelope_and_receipt_bytes_for_the_demo_session() {
    let mut world = HostWorld::new(99);
    let source = ConsumerId::new(ConsumerKind::Plugin, "measure");
    let envelopes: Vec<CommandEnvelope> = demo_script::demo_payloads()
        .into_iter()
        .map(|payload| CommandEnvelope {
            id: payload.command_id().to_string(),
            source: source.clone(),
            grant: demo_script::demo_token(),
            payload,
            target_tick: None,
            txn: None,
        })
        .collect();
    for env in &envelopes {
        world.submit(env.clone()).expect("submit");
    }
    let receipts = world.tick();
    assert_eq!(receipts.len(), envelopes.len());

    println!();
    println!(
        "{:<28} {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8}",
        "command", "env pc", "env json", "rcpt pc", "rcpt js", "sum pc", "sum js"
    );
    let (mut te_pc, mut te_js, mut tr_pc, mut tr_js, mut ts_pc, mut ts_js) = (0, 0, 0, 0, 0, 0);
    for (env, entry) in envelopes.iter().zip(&receipts) {
        let summary = SummaryReceipt {
            seq: entry.receipt.seq,
            tick_applied: entry.receipt.tick_applied,
            result: match &entry.receipt.result {
                CommandResult::Ok(fx) => SummaryResult::Ok(fx.summary()),
                CommandResult::Rejected(r) => SummaryResult::Rejected(r.clone()),
            },
        };
        let row = (
            postcard_len(env),
            json_len(env),
            postcard_len(&entry.receipt),
            json_len(&entry.receipt),
            postcard_len(&summary),
            json_len(&summary),
        );
        println!(
            "{:<28} {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8}",
            entry.command_id, row.0, row.1, row.2, row.3, row.4, row.5
        );
        te_pc += row.0;
        te_js += row.1;
        tr_pc += row.2;
        tr_js += row.3;
        ts_pc += row.4;
        ts_js += row.5;
    }
    println!(
        "{:<28} {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8}",
        format!("TOTAL ({} commands)", envelopes.len()),
        te_pc,
        te_js,
        tr_pc,
        tr_js,
        ts_pc,
        ts_js
    );

    // And the read-back query the session ends with: scan the build yard.
    let scan = CommandEnvelope {
        id: dc_api::ids::WORLD_SCAN_REGION.to_string(),
        source: source.clone(),
        grant: demo_script::demo_token(),
        payload: Payload::ScanRegion(dc_api::payload::ScanRegion {
            min: Vec3i::new(0, 0, 0),
            max: Vec3i::new(10, 15, 10),
        }),
        target_tick: None,
        txn: None,
    };
    let receipt = world.query(&scan);
    println!(
        "{:<28} {:>8} {:>8} | {:>8} {:>8} |  (query: {} voxels)",
        "dc:world/scan_region",
        postcard_len(&scan),
        json_len(&scan),
        postcard_len(&receipt),
        json_len(&receipt),
        Volume::new(Vec3i::new(0, 0, 0), Vec3i::new(10, 15, 10)).voxel_count(),
    );

    // Keep the receipts honest: full-effects receipts must dominate summary
    // receipts for the bulk fill (that is the entire point of measuring).
    let fill_entry = receipts
        .iter()
        .find(|e| e.command_id == dc_api::ids::WORLD_FILL)
        .expect("fill receipt");
    if let CommandResult::Ok(fx) = &fill_entry.receipt.result {
        assert!(fx.blocks_changed.len() > 10);
    } else {
        panic!("fill rejected");
    }
    assert!(postcard_len(&fill_entry.receipt) > 100);
}
