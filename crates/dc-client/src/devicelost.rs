//! Loud degradation when the GPU device is lost.
//!
//! A lost device is a permanent possibility, not a symptom of one bug: driver
//! faults, a Windows TDR, a GPU reset, an unplugged eGPU, another process
//! exhausting VRAM. journal/0051 removed the *cause* of the losses we had seen
//! (host-RAM exhaustion); this module is about what happens when one arrives
//! anyway.
//!
//! The failure it replaces (ROADMAP Observed, journal/0050): wgpu reports
//! `DeviceLost`, some renderer worker panics on an `unwrap`, that panic poisons
//! a `Mutex`, and every later system that touches the same mutex panics on
//! `PoisonError` — a cluster of panics all naming the wrong thing, with the one
//! true cause buried a hundred lines up. Worse, the panics happen on worker
//! threads, so the process can still wind down and report **exit 0**: two of
//! the recorded crashes did exactly that, which is why the ROADMAP carries the
//! warning that exit codes lie about GPU crashes.
//!
//! Four properties, in the order they matter:
//!
//! 1. **The root cause is stated once, unmistakably.** A boxed banner at
//!    detection, and — because a log tail is what a future agent actually
//!    reads — a second terminal summary printed as the last thing before the
//!    process exits ([`finish`]).
//! 2. **No misleading exit 0.** [`finish`] maps a device loss to
//!    [`EXIT_DEVICE_LOST`] and any observed panic to [`EXIT_PANICKED`]. A panic
//!    on a *worker* thread is included, which the Rust default cannot do.
//! 3. **No poison cascade.** [`lock_forgiving`] replaces the `expect` at every
//!    client mutex seam, so one panic cannot convert every later lock into a
//!    second panic reporting the wrong thing.
//! 4. **A controlled shutdown, not a crash.** The device loss arrives through
//!    Bevy's [`RenderErrorHandler`] seam; we log, flag, and ask the app to
//!    exit, which unwinds the window and the MCP threads normally.
//!
//! ## What the API actually allows
//!
//! wgpu 29 exposes `Device::set_device_lost_callback` and
//! `Device::on_uncaptured_error`, but **Bevy 0.19 already owns both**
//! (`bevy_render::error_handler::DeviceErrorHandler::new`), and registering our
//! own would *replace* Bevy's rather than chain with it. What Bevy leaves us is
//! the policy seam: the [`RenderErrorHandler`] resource, a plain fn pointer
//! called with the classified error and both worlds. That is the hook this
//! module rests on. Note also that a `DeviceLost` never reaches
//! `on_uncaptured_error` at all — wgpu's core backend explicitly drops it there
//! (`ErrorType::DeviceLost => return, // will be surfaced via callback`), so the
//! device-lost callback is the only path, and Bevy holds it.
//!
//! **Device recovery is out of scope and stays out.** Bevy 0.19 does have a
//! `RenderErrorPolicy::Recover(RenderCreation)` arm, but taking it means every
//! GPU resource in the render world is recreated from scratch — and our
//! renderer holds a great deal of hand-rolled state that assumes it was created
//! once: the shader-pack post stage, the edge pass, the terrain material's
//! assembled atlases, and thousands of streamed chunk/far-tile meshes. Nothing
//! about that has ever been exercised against a re-created device. Claiming
//! recovery we cannot test would be worse than an honest shutdown, so the
//! policy is [`RenderErrorPolicy::StopRendering`] plus an exit.

use std::panic::PanicHookInfo;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, Once, OnceLock};

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy};

/// Exit code when the session ended because the GPU device was lost.
/// Distinct from a panic so a script can tell "the GPU died" from "our code is
/// wrong" without parsing the log.
pub const EXIT_DEVICE_LOST: u8 = 70;
/// Exit code when the session ended with a panic anywhere (including on a
/// worker thread, where Rust's own `101` never reaches the process).
pub const EXIT_PANICKED: u8 = 101;
/// Exit code for any other unsuccessful `AppExit`.
pub const EXIT_RENDER_ERROR: u8 = 71;

/// Panics seen before we call it a cascade and force a shutdown. One panic is a
/// bug to surface; three in a session is the failure mode this module exists to
/// stop, and continuing only buries the first one further.
const CASCADE_THRESHOLD: u32 = 3;

static DEVICE_LOST: AtomicBool = AtomicBool::new(false);
static PANICS: AtomicU32 = AtomicU32::new(0);
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static BANNER: Once = Once::new();
static REASON: OnceLock<String> = OnceLock::new();
static HOOK: Once = Once::new();

/// Has the render device been lost (or a loss been simulated) this session?
pub fn device_lost() -> bool {
    DEVICE_LOST.load(Ordering::Relaxed)
}

/// Bevy run condition: the inverse of [`device_lost`]. Gating the gameplay
/// chain on this shrinks the window in which systems keep streaming chunks and
/// building meshes for a device that no longer exists — which is precisely the
/// work that generates the downstream noise.
pub fn renderer_healthy() -> bool {
    !device_lost()
}

/// How many panics this process has observed, across all threads.
pub fn panic_count() -> u32 {
    PANICS.load(Ordering::Relaxed)
}

/// Record a lost device and state it once, unmistakably.
///
/// Safe to call from any thread and any number of times; only the first call
/// prints and only the first reason is kept, because the *first* one is the
/// root cause and everything after it is consequence.
pub fn report_device_lost(reason: &str) {
    DEVICE_LOST.store(true, Ordering::Relaxed);
    let _ = REASON.set(reason.to_string());
    BANNER.call_once(|| {
        error!(
            "\n\
             ╔══════════════════════════════════════════════════════════════════╗\n\
             ║  GPU DEVICE LOST — this is the ROOT CAUSE of everything below.   ║\n\
             ╚══════════════════════════════════════════════════════════════════╝\n\
             reason: {reason}\n\
             The render device this process was using no longer exists (driver \n\
             fault, GPU reset/TDR, or the adapter went away). Nothing the game \n\
             can draw will work again on this device.\n\
             Any panic, PoisonError, buffer-map failure or missing-pipeline \n\
             message printed AFTER this line is downstream noise, not a second \n\
             bug. deepcraft is shutting down and will exit {EXIT_DEVICE_LOST}."
        );
    });
}

/// Lock a mutex, tolerating poison.
///
/// The `expect("…mutex")` this replaces is what turned one panic into a cluster
/// of panics: a thread that panics while holding a lock poisons it, and every
/// later `expect` on that lock panics too, each reporting the mutex rather than
/// the original fault.
///
/// Recovering the guard is sound at the seams we use it on because none of them
/// guard an invariant that spans a lock acquisition. The
/// `Arc<Mutex<WorldGenerator>>` — the notable one — guards pure memoization
/// caches: a `HashMap` insert either happened or it didn't, every entry is a
/// deterministic function of its key, and any entry can be recomputed. A
/// half-updated *entry* is not representable. The MCP bridge's receiver is a
/// queue; the worst a poisoned recovery costs is a dropped request.
pub fn lock_forgiving<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poison| {
        // Deliberately quiet after the first: the point is not to add to the
        // noise, and `report_device_lost` / the panic hook have already said
        // what actually went wrong.
        poison.into_inner()
    })
}

/// Install the process-wide panic hook, chaining the existing one so the normal
/// message and backtrace still print.
///
/// It does three things the default hook cannot:
/// - counts panics on **every** thread, so the exit code can reflect a panic
///   that happened off the main thread (the exit-0 defect);
/// - labels a panic that arrives after a device loss as downstream, so a reader
///   scanning the tail is not misled into debugging the wrong thing;
/// - requests a controlled shutdown once a cascade is evident.
pub fn install_panic_hook() {
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info: &PanicHookInfo<'_>| {
            let n = PANICS.fetch_add(1, Ordering::Relaxed) + 1;
            if device_lost() {
                eprintln!(
                    "[dc-client] panic #{n} AFTER the GPU device was lost — downstream of the \
                     DEVICE LOST banner above, not a separate bug."
                );
            } else if n > 1 {
                eprintln!(
                    "[dc-client] panic #{n} this session — if an earlier panic poisoned state, \
                     this one may be reporting a consequence, not a cause."
                );
            }
            previous(info);
            if device_lost() || n >= CASCADE_THRESHOLD {
                request_shutdown();
            }
        }));
    });
}

/// Ask the app to exit at the next frame boundary. Called from panic hooks and
/// worker threads, where writing an ECS message directly is not possible.
fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

/// The plugin: installs the render-error policy, the shutdown pump, and the
/// diagnostic trigger.
pub struct DeviceLostPlugin;

impl Plugin for DeviceLostPlugin {
    fn build(&self, app: &mut App) {
        install_panic_hook();
        // Overrides Bevy's default handler (which logs a one-line `error!` and
        // exits) with one that names the cause in full and flags the rest of
        // the client.
        app.insert_resource(RenderErrorHandler(handle_render_error));
        app.add_systems(Update, (pump_shutdown, simulate_device_lost));
    }
}

/// Our [`RenderErrorHandler`]. Bevy calls this from `pre_extract` with the
/// classified error and both worlds; the return value is the policy.
fn handle_render_error(
    error: &RenderError,
    main_world: &mut World,
    _render_world: &mut World,
) -> RenderErrorPolicy {
    match error.ty {
        ErrorType::DeviceLost => report_device_lost(&error.description),
        other => {
            // Not a device loss, but still fatal to rendering. Say so once in
            // the same shape, so the tail is readable either way.
            error!(
                "RENDER ERROR ({other:?}) — shutting down. {}",
                error.description
            );
        }
    }
    main_world.write_message(AppExit::error());
    RenderErrorPolicy::StopRendering
}

/// Turns a shutdown requested from a non-ECS context (a panic on any thread,
/// the diagnostic trigger) into a real `AppExit`, so the window, the MCP
/// threads and the render device tear down the normal way instead of the
/// process lingering with dead workers.
fn pump_shutdown(mut exit: MessageWriter<AppExit>) {
    if SHUTDOWN_REQUESTED.swap(false, Ordering::Relaxed) {
        exit.write(AppExit::error());
    }
}

/// `DC_SIMULATE_DEVICE_LOST=<seconds>`: after that many seconds of app time,
/// inject a synthetic device loss through the same [`report_device_lost`] seam
/// the real handler uses. Zero cost when unset (one `Local<Option<f32>>` read
/// per frame after the first).
///
/// This is the only way to regression-test this path — a real `DeviceLost`
/// cannot be induced on demand. Be honest about its reach: it exercises the
/// banner, the flag, the gameplay gate, the shutdown pump and the exit code —
/// **everything downstream of the classification**. What it does not exercise
/// is Bevy's own `DeviceErrorHandler::poll` → `RenderState::Errored` →
/// `RenderErrorHandler` link, which is Bevy's code, not ours.
///
/// ```text
/// DC_SIMULATE_DEVICE_LOST=8 cargo run --release -p dc-client
/// ```
fn simulate_device_lost(time: Res<Time>, mut after: Local<Option<f32>>) {
    if after.is_none() {
        *after = Some(
            std::env::var("DC_SIMULATE_DEVICE_LOST")
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(f32::INFINITY),
        );
    }
    let Some(after) = *after else { return };
    if time.elapsed_secs() >= after && !device_lost() {
        report_device_lost(
            "SIMULATED by DC_SIMULATE_DEVICE_LOST (diagnostic trigger — the real device is fine)",
        );
        request_shutdown();
    }
}

/// Print the terminal diagnosis and map the session to a process exit code.
///
/// This runs after `App::run` returns, so it is the **last** thing in the log —
/// which is the part a future agent (or a `tail`) actually sees. The banner at
/// detection can be scrolled away by a thousand lines of cascade; this cannot.
pub fn finish(exit: &AppExit) -> ExitCode {
    let code = exit_code_for(exit);
    match code {
        0 => {}
        EXIT_DEVICE_LOST => eprintln!(
            "\n[dc-client] SESSION ENDED ABNORMALLY\n  \
             root cause : GPU DEVICE LOST — {}\n  \
             panics     : {}\n  \
             exit code  : {EXIT_DEVICE_LOST}",
            REASON.get().map(String::as_str).unwrap_or("(no reason)"),
            match panic_count() {
                0 => "none".to_string(),
                n => format!("{n} — ALL downstream of the device loss, not separate bugs"),
            },
        ),
        EXIT_PANICKED => eprintln!(
            "\n[dc-client] SESSION ENDED ABNORMALLY\n  \
             root cause : {} panic(s) — see the FIRST one above; the device was NOT lost\n  \
             exit code  : {EXIT_PANICKED}",
            panic_count(),
        ),
        other => eprintln!(
            "\n[dc-client] SESSION ENDED ABNORMALLY\n  \
             root cause : the renderer reported a fatal error (see the RENDER ERROR line above)\n  \
             exit code  : {other}"
        ),
    }
    ExitCode::from(code)
}

/// The exit code this session earns. `0` only when nothing went wrong: a device
/// loss or a panic on **any** thread outranks whatever `AppExit` the event loop
/// managed to produce, because the recorded crashes produced a clean-looking
/// `AppExit` while the render threads were already dead.
fn exit_code_for(exit: &AppExit) -> u8 {
    if device_lost() {
        return EXIT_DEVICE_LOST;
    }
    if panic_count() > 0 {
        return EXIT_PANICKED;
    }
    match exit {
        AppExit::Success => 0,
        AppExit::Error(_) => EXIT_RENDER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// The poison cascade, in miniature: a thread panics while holding the
    /// generator-shaped mutex, and the next acquisition must still hand back a
    /// usable guard rather than panicking about a mutex.
    #[test]
    fn a_poisoned_lock_still_hands_back_its_value() {
        let m = Arc::new(Mutex::new(vec![1u32, 2, 3]));
        let poisoner = m.clone();
        // Silence the default hook for the deliberate panic below.
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let joined = std::thread::spawn(move || {
            let mut guard = poisoner.lock().expect("fresh mutex");
            guard.push(4);
            panic!("the panic that poisons the lock");
        })
        .join();
        std::panic::set_hook(hook);
        assert!(joined.is_err(), "the poisoning thread should have panicked");

        assert!(m.lock().is_err(), "the mutex should now be poisoned");
        // The `expect` this replaces would panic here. The forgiving lock does
        // not, and the value it returns is the real one, mutation included.
        let guard = lock_forgiving(&m);
        assert_eq!(*guard, vec![1, 2, 3, 4]);
    }

    /// A forgiving lock is an ordinary lock when nothing has gone wrong.
    #[test]
    fn a_healthy_lock_is_unaffected() {
        let m = Mutex::new(7u8);
        assert_eq!(*lock_forgiving(&m), 7);
        *lock_forgiving(&m) = 9;
        assert_eq!(*lock_forgiving(&m), 9);
    }

    /// The exit-code contract, which is the part a script and a future agent
    /// depend on. Exercised through the classification, not around it.
    ///
    /// These three cases are asserted in one test because the state they read
    /// is process-global: `cargo test` runs tests in parallel threads, so
    /// splitting them would let one test's `report_device_lost` leak into
    /// another's assertion. The healthy case must therefore be checked first.
    #[test]
    fn the_exit_code_never_lies_about_a_lost_device() {
        // Healthy: a clean exit stays 0. (Must run before anything below sets
        // the process-global flag.)
        assert_eq!(
            exit_code_for(&AppExit::Success),
            0,
            "a clean session must still exit 0"
        );
        assert_eq!(
            exit_code_for(&AppExit::error()),
            EXIT_RENDER_ERROR,
            "a fatal render error is not a device loss, and is not 0 either"
        );

        // A device loss reported anywhere makes even an AppExit::Success —
        // which is what a worker-thread cascade produced in the recorded
        // crashes — exit non-zero.
        assert!(!device_lost());
        report_device_lost("test: synthetic loss");
        assert!(device_lost());
        assert!(!renderer_healthy());
        assert_eq!(
            exit_code_for(&AppExit::Success),
            EXIT_DEVICE_LOST,
            "a lost device must never report exit 0"
        );
        // And the reason is kept, once, for the terminal summary.
        assert_eq!(
            REASON.get().map(String::as_str),
            Some("test: synthetic loss")
        );
        report_device_lost("test: a later, downstream reason");
        assert_eq!(
            REASON.get().map(String::as_str),
            Some("test: synthetic loss"),
            "the FIRST reason is the root cause and must not be overwritten"
        );
    }
}
