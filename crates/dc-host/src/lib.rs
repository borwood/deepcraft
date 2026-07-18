//! wasmtime plugin host (S5): runs a plugin compiled to wasm32-unknown-unknown
//! and bridges its `dc.call` import onto a [`HostWorld`].
//!
//! Target choice: **wasm32-unknown-unknown**, not WASI. Plugins are pure
//! compute driving the world exclusively through envelopes; they get no
//! clocks, no filesystem, no ambient anything — which is exactly the
//! capability posture the API wants. (WASI would only add surface to audit.)
//!
//! Boundary rules enforced here, before the world ever sees an envelope:
//! - the envelope's `source` must be the plugin's installed identity;
//! - the envelope's claimed grant must be covered by the installed token
//!   ([`dc_api::CapabilityToken::covers_token`] — the attenuation relation).
//!   A plugin can claim less than it was granted (least authority), never
//!   more.

pub mod demo_script;

use dc_api::abi::{self, PluginRequest, PluginResponse};
use dc_api::envelope::{CommandReceipt, CommandResult, RejectReason};
use dc_api::{CapabilityToken, CommandEnvelope, ConsumerId, HostWorld};
use wasmtime::{Caller, Engine, Extern, Linker, Module, Store, TypedFunc};

/// Store payload: the world plus the plugin's installed identity/authority.
pub struct HostState {
    pub world: HostWorld,
    pub consumer: ConsumerId,
    pub installed: CapabilityToken,
    /// Cursor into the world's receipt log for DrainReceipts.
    receipt_cursor: usize,
}

/// A loaded plugin instance bound to a world.
pub struct PluginHost {
    store: Store<HostState>,
    run: TypedFunc<(), ()>,
    tick: TypedFunc<(), ()>,
}

impl PluginHost {
    /// Compile and instantiate a plugin, wiring `dc.call` to the world.
    pub fn load(
        wasm_path: &std::path::Path,
        world: HostWorld,
        consumer: ConsumerId,
        installed: CapabilityToken,
    ) -> wasmtime::Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)?;
        let mut linker: Linker<HostState> = Linker::new(&engine);
        linker.func_wrap(
            "dc",
            "call",
            |mut caller: Caller<'_, HostState>, ptr: u32, len: u32| -> wasmtime::Result<u64> {
                let memory = caller
                    .get_export("memory")
                    .and_then(Extern::into_memory)
                    .ok_or_else(|| wasmtime::Error::msg("plugin exports no memory"))?;
                let mut req_bytes = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut req_bytes)?;
                let response = match abi::decode::<PluginRequest>(&req_bytes) {
                    Ok(request) => handle_request(caller.data_mut(), request),
                    Err(e) => PluginResponse::Error(format!("bad request: {e}")),
                };
                let resp_bytes = abi::encode(&response);
                // Allocate in the guest and write the response there.
                let alloc = caller
                    .get_export("dc_alloc")
                    .and_then(Extern::into_func)
                    .ok_or_else(|| wasmtime::Error::msg("plugin exports no dc_alloc"))?
                    .typed::<u32, u32>(&caller)?;
                let resp_ptr = alloc.call(&mut caller, resp_bytes.len() as u32)?;
                memory.write(&mut caller, resp_ptr as usize, &resp_bytes)?;
                Ok(abi::pack_ptr_len(resp_ptr, resp_bytes.len() as u32))
            },
        )?;
        let mut store = Store::new(
            &engine,
            HostState {
                world,
                consumer,
                installed,
                receipt_cursor: 0,
            },
        );
        let instance = linker.instantiate(&mut store, &module)?;
        let run = instance.get_typed_func::<(), ()>(&mut store, "dc_run")?;
        let tick = instance.get_typed_func::<(), ()>(&mut store, "dc_tick")?;
        Ok(Self { store, run, tick })
    }

    /// Call the plugin's session entry (`dc_run`).
    pub fn run(&mut self) -> wasmtime::Result<()> {
        self.run.call(&mut self.store, ())
    }

    /// Notify the plugin a tick completed (`dc_tick`): it drains receipts,
    /// polls events, and may submit reactions for the next tick.
    pub fn notify_tick(&mut self) -> wasmtime::Result<()> {
        self.tick.call(&mut self.store, ())
    }

    pub fn world(&self) -> &HostWorld {
        &self.store.data().world
    }

    pub fn world_mut(&mut self) -> &mut HostWorld {
        &mut self.store.data_mut().world
    }
}

/// Boundary check + dispatch for one plugin request.
fn handle_request(state: &mut HostState, request: PluginRequest) -> PluginResponse {
    if std::env::var("DC_HOST_TRACE").is_ok() {
        eprintln!("[dc-host] request: {request:?}");
    }
    match request {
        PluginRequest::Submit(env) => match boundary_check(state, &env) {
            Err(reason) => PluginResponse::SubmitRejected(boundary_receipt(state, reason)),
            Ok(()) => match state.world.submit(env) {
                Ok(ack) => PluginResponse::Submitted(ack),
                Err(entry) => PluginResponse::SubmitRejected(entry.receipt),
            },
        },
        PluginRequest::Query(env) => match boundary_check(state, &env) {
            Err(reason) => PluginResponse::Query(dc_api::QueryReceipt {
                tick_observed: state.world.current_tick(),
                result: dc_api::QueryResult::Rejected(reason),
            }),
            Ok(()) => PluginResponse::Query(state.world.query(&env)),
        },
        PluginRequest::DrainReceipts => {
            let log = state.world.receipt_log();
            let mine: Vec<_> = log[state.receipt_cursor.min(log.len())..]
                .iter()
                .filter(|e| e.source == state.consumer)
                .cloned()
                .collect();
            state.receipt_cursor = log.len();
            PluginResponse::Receipts(mine)
        }
    }
}

fn boundary_check(state: &HostState, env: &CommandEnvelope) -> Result<(), RejectReason> {
    if env.source != state.consumer {
        return Err(RejectReason::MissingCapability {
            needed: format!(
                "source identity {:?}/{} (claimed {:?}/{})",
                state.consumer.kind, state.consumer.name, env.source.kind, env.source.name
            ),
        });
    }
    if !state.installed.covers_token(&env.grant) {
        return Err(RejectReason::MissingCapability {
            needed: "claimed grant exceeds installed token".to_string(),
        });
    }
    Ok(())
}

/// A synthetic receipt for boundary rejections: never entered the world, so
/// it carries no world seq (0) and the current completed tick.
fn boundary_receipt(state: &HostState, reason: RejectReason) -> CommandReceipt {
    CommandReceipt {
        seq: 0,
        tick_applied: state.world.current_tick(),
        result: CommandResult::Rejected(reason),
    }
}
