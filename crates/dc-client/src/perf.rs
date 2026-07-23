//! Runtime perf-observability — the opener of the "perf window" (ROADMAP).
//!
//! The project measures gen-time rigorously but had ZERO runtime span profiling.
//! This module is that instrument: `tracing` spans on the hot paths, an
//! aggregating `tracing_subscriber::Layer` that computes **self-time** (exclusive
//! of child spans) per span name, and a queryable in-memory authority
//! ([`PerfAggregate`], exposed as the [`PerfHandle`] resource).
//!
//! **S-3 (a summary derived from the authority, never beside it), applied to
//! timing data.** The aggregate is the authority. The `docs/audits/` markdown
//! dump ([`write_baseline`]) is the FIRST consumer. The ratified in-game
//! perf/debug overlay (docs/design/ideas.md § "The perf/debug overlay is
//! player-facing") is the named heir: it reads the SAME `PerfHandle` resource
//! live (`handle.0.lock().ranked()`), never a re-instrumented file dumper. Do
//! not build the overlay here; leave it this clean seam.
//!
//! **Zero cost when the `perf` feature is off.** [`perf_span!`] expands to
//! nothing, no dependency on this module's `enabled` items is compiled, and
//! `bevy/trace` (bevy's own system/render spans) is not enabled. Everything
//! below `enabled` is `#[cfg(feature = "perf")]`.

/// Open a self-timing span with a STABLE literal name. **Bind the result** to a
/// scope-lived guard: `let _perf = perf_span!("x");`. The guard holds the span
/// entered until it drops at end of scope (RAII).
///
/// Under `--features perf` it expands to a real `bevy::log::info_span!(..)
/// .entered()` guard; otherwise it expands to a zero-sized [`PerfGuard`] — so
/// there is no span-creation cost in normal play, and the binding is a real
/// value in both builds (never `()`, which would trip `clippy::let_unit_value`;
/// never a stray statement, which would trip `redundant_semicolons`).
///
/// Put each sub-phase in its own block (`let x = { let _perf = perf_span!("x");
/// work() };`) so its guard drops before the next phase enters — that is what
/// keeps sibling phases as separate children rather than one nested chain, so the
/// self-time layer attributes each phase to itself.
#[cfg(feature = "perf")]
#[macro_export]
macro_rules! perf_span {
    ($name:literal) => {
        ::bevy::log::info_span!($name).entered()
    };
}

#[cfg(not(feature = "perf"))]
#[macro_export]
macro_rules! perf_span {
    ($name:literal) => {
        $crate::perf::PerfGuard
    };
}

/// Zero-sized no-op guard for the non-perf build (see [`perf_span!`]).
#[cfg(not(feature = "perf"))]
pub struct PerfGuard;

#[cfg(feature = "perf")]
pub use enabled::*;

#[cfg(feature = "perf")]
mod enabled {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use bevy::app::AppExit;
    use bevy::log::BoxedLayer;
    use bevy::log::tracing::Subscriber;
    use bevy::log::tracing::span::Id;
    use bevy::log::tracing_subscriber::layer::{Context, Layer};
    use bevy::log::tracing_subscriber::registry::LookupSpan;
    use bevy::prelude::*;

    use crate::player::Player;

    /// Cumulative timing for one span name over a capture.
    #[derive(Default)]
    pub struct SpanStat {
        /// Exclusive (self) time, nanoseconds, summed over every exit.
        pub self_nanos: u64,
        /// Number of times the span was entered.
        pub calls: u64,
    }

    /// One ranked row (a plain view over [`SpanStat`], for consumers).
    pub struct Row {
        pub name: &'static str,
        pub self_nanos: u64,
        pub calls: u64,
    }

    /// The authority: per-span cumulative self-time + call count, queryable at
    /// any time. The docs dump reads it; the future overlay reads the SAME
    /// structure live through [`PerfHandle`].
    #[derive(Default)]
    pub struct PerfAggregate {
        spans: HashMap<&'static str, SpanStat>,
        /// Sum of top-level (stack-empty) span durations — a reference "captured
        /// wall" figure. The ranking's `%` column is against total *self*-time,
        /// which sums to 100 regardless of parallelism.
        total_wall_nanos: u64,
    }

    impl PerfAggregate {
        /// Record one span exit: `self_nanos` accrues, `calls += 1`.
        pub fn record(&mut self, name: &'static str, self_nanos: u64) {
            let e = self.spans.entry(name).or_default();
            e.self_nanos += self_nanos;
            e.calls += 1;
        }

        /// Reset — used to drop the warm-up/load window so a capture measures
        /// only the scenario.
        pub fn clear(&mut self) {
            self.spans.clear();
            self.total_wall_nanos = 0;
        }

        pub fn total_wall_nanos(&self) -> u64 {
            self.total_wall_nanos
        }

        pub fn total_self_nanos(&self) -> u64 {
            self.spans.values().map(|s| s.self_nanos).sum()
        }

        /// Rows sorted worst-first by self-time. The ranked metric is self-time,
        /// not wall-time-with-children.
        pub fn ranked(&self) -> Vec<Row> {
            let mut v: Vec<Row> = self
                .spans
                .iter()
                .map(|(name, s)| Row {
                    name,
                    self_nanos: s.self_nanos,
                    calls: s.calls,
                })
                .collect();
            v.sort_unstable_by(|a, b| b.self_nanos.cmp(&a.self_nanos).then(a.name.cmp(b.name)));
            v
        }
    }

    /// The queryable authority as a Bevy resource. The overlay heir reads this.
    #[derive(Resource, Clone)]
    pub struct PerfHandle(pub Arc<Mutex<PerfAggregate>>);

    // Per-thread span stack for exclusive self-time. Bevy runs each system on a
    // single thread, and the spans within a system nest on that thread, so a
    // thread-local stack attributes correctly across the multi-threaded executor.
    thread_local! {
        static STACK: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) };
    }

    struct Frame {
        name: &'static str,
        start: Instant,
        /// Total (inclusive) time of child spans that have already exited.
        child_nanos: u64,
    }

    /// The aggregating layer. Self-time = a span's inclusive elapsed minus the
    /// inclusive time of its children (which each reported their elapsed up to
    /// the parent frame on exit).
    struct PerfLayer {
        agg: Arc<Mutex<PerfAggregate>>,
    }

    impl<S> Layer<S> for PerfLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
            let name = ctx.span(id).map_or("?", |s| s.name());
            STACK.with(|st| {
                st.borrow_mut().push(Frame {
                    name,
                    start: Instant::now(),
                    child_nanos: 0,
                });
            });
        }

        fn on_exit(&self, _id: &Id, _ctx: Context<'_, S>) {
            STACK.with(|st| {
                let mut st = st.borrow_mut();
                let Some(frame) = st.pop() else { return };
                let elapsed = frame.start.elapsed().as_nanos() as u64;
                let self_nanos = elapsed.saturating_sub(frame.child_nanos);
                if let Some(parent) = st.last_mut() {
                    parent.child_nanos += elapsed;
                }
                let top_level = st.is_empty();
                drop(st);
                let mut agg = self.agg.lock().unwrap();
                agg.record(frame.name, self_nanos);
                if top_level {
                    agg.total_wall_nanos += elapsed;
                }
            });
        }
    }

    /// Bevy `LogPlugin::custom_layer` entry point: build the aggregate, insert it
    /// as the [`PerfHandle`] resource (so the game — and the future overlay — can
    /// query it), and return the layer to add to bevy's subscriber.
    pub fn log_layer(app: &mut App) -> Option<BoxedLayer> {
        let agg = Arc::new(Mutex::new(PerfAggregate::default()));
        app.insert_resource(PerfHandle(agg.clone()));
        Some(Box::new(PerfLayer { agg }))
    }

    /// `--perf-drop <seconds>` config: the scenario duration.
    #[derive(Resource)]
    pub struct PerfDrop {
        pub secs: f64,
    }

    /// Progress state for the scripted drop (a `Local`).
    #[derive(Default)]
    pub struct DropState {
        elapsed: f64,
        since_step: f64,
        started: bool,
        done: bool,
    }

    const DROP_WARMUP_S: f64 = 2.0;
    const DROP_STEP_INTERVAL_S: f64 = 0.5;
    const DROP_STEP_M: f64 = 30.0;

    /// The deterministic vertical-drop capture: warm up, reset the aggregate,
    /// then teleport the player straight DOWN in fixed steps so the adaptive load
    /// volume continuously streams fresh below-ground chunks (the "time slows to a
    /// crawl on a vertical drop" symptom, walk 0071). No gravity, no physics — a
    /// pure teleport-step so the scenario is reproducible. When the window
    /// elapses, write the ranked artifact and exit honestly.
    pub fn perf_drop_driver(
        time: Res<Time>,
        drop: Res<PerfDrop>,
        handle: Res<PerfHandle>,
        mut player: ResMut<Player>,
        mut state: Local<DropState>,
        mut exit: MessageWriter<AppExit>,
    ) {
        if state.done {
            return;
        }
        let dt = f64::from(time.delta_secs());
        state.elapsed += dt;
        if state.elapsed < DROP_WARMUP_S {
            return;
        }
        if !state.started {
            state.started = true;
            // Drop the warm-up/load window: the capture measures only the drop.
            handle.0.lock().unwrap().clear();
            info!(
                "perf: warm-up done, capturing {:.1}s of vertical drop (-{} m every {}s)",
                drop.secs, DROP_STEP_M, DROP_STEP_INTERVAL_S
            );
        }
        state.since_step += dt;
        if state.since_step >= DROP_STEP_INTERVAL_S {
            state.since_step -= DROP_STEP_INTERVAL_S;
            // Fly mode is the boot default, so a direct teleport is not fought by
            // collision — the player sinks and the streamer chases it downward.
            player.pos_m.y -= DROP_STEP_M;
        }
        if state.elapsed - DROP_WARMUP_S >= drop.secs {
            state.done = true;
            let agg = handle.0.lock().unwrap();
            match write_baseline(&agg, drop.secs) {
                Ok(path) => info!("perf: wrote baseline to {path}"),
                Err(e) => error!("perf: failed to write baseline: {e}"),
            }
            exit.write(AppExit::Success);
        }
    }

    const ARTIFACT_PATH: &str = "docs/audits/2026-07-23-perf-baseline-vertical-drop.md";

    /// The docs/audits consumer: render the aggregate as a ranked self-time
    /// table (worst first) and rewrite the artifact. Returns the path written.
    pub fn write_baseline(agg: &PerfAggregate, scenario_secs: f64) -> std::io::Result<String> {
        use std::fmt::Write as _;

        let rows = agg.ranked();
        let total_self = agg.total_self_nanos().max(1);
        let ns_ms = |n: u64| n as f64 / 1_000_000.0;

        let mut table = String::new();
        writeln!(
            table,
            "| # | span | self-time (ms) | calls | mean (µs) | % of captured |"
        )
        .ok();
        writeln!(
            table,
            "|---|------|----------------|-------|-----------|---------------|"
        )
        .ok();
        for (i, r) in rows.iter().enumerate() {
            let mean_us = if r.calls > 0 {
                (r.self_nanos as f64 / r.calls as f64) / 1000.0
            } else {
                0.0
            };
            let pct = 100.0 * r.self_nanos as f64 / total_self as f64;
            writeln!(
                table,
                "| {} | `{}` | {:.3} | {} | {:.2} | {:.1}% |",
                i + 1,
                r.name,
                ns_ms(r.self_nanos),
                r.calls,
                mean_us,
                pct,
            )
            .ok();
        }

        let doc = format!(
            "# Perf baseline — the vertical drop (2026-07-23)\n\
\n\
Captured by `cargo run --release -p dc-client --features perf -- --perf-drop {secs:.0}`.\n\
The player warms up ~{warm:.0}s (world load), the aggregate is reset, then the player\n\
teleport-steps -{step} m every {interval}s for {secs:.0}s so a curtain of fresh\n\
below-ground chunks streams every frame (the walk-0071 vertical-drop symptom).\n\
\n\
## The instrument\n\
\n\
`tracing` spans on the hot paths (feature-gated behind `perf`; zero cost when off).\n\
A custom `tracing_subscriber::Layer` (installed via bevy's `LogPlugin::custom_layer`)\n\
computes **self-time** (exclusive of child spans) per span name into an\n\
`Arc<Mutex<PerfAggregate>>` that is also the Bevy `PerfHandle` resource. This file\n\
is the first consumer of that authority; the in-game perf/debug overlay is the heir\n\
that reads the same resource live (S-3 — journal/0080). `bevy/trace` is on, so\n\
bevy's own system/render spans rank beside ours.\n\
\n\
Self-time, not wall-time-with-children, is the ranked metric: a parent span shows\n\
only the time not attributed to a child, so killers surface without double-counting.\n\
The `%` column is over total captured self-time (sums to ~100).\n\
\n\
## Ranked self-time (worst first)\n\
\n\
{table}\n\
Captured window: {secs:.1} s. Total captured self-time: {total_ms:.3} ms \
(reference top-level wall: {wall_ms:.3} ms).\n",
            secs = scenario_secs,
            warm = DROP_WARMUP_S,
            step = DROP_STEP_M,
            interval = DROP_STEP_INTERVAL_S,
            table = table,
            total_ms = ns_ms(agg.total_self_nanos()),
            wall_ms = ns_ms(agg.total_wall_nanos()),
        );

        std::fs::write(ARTIFACT_PATH, doc)?;
        Ok(ARTIFACT_PATH.to_string())
    }
}
