# 0090 — the runner names what the loop already did

> STATUS: STUB / IN PROGRESS (Movement 1 of deep-time-loop → declared-passes).
> Byte-identical re-housing of the deep-time epoch loop onto a self-declaring
> pass-runner. This file is committed early per the resource rules; the
> narrative below is filled as the work lands.

## The task

Build a deep-time **pass-runner** and re-house the existing per-epoch erosion
loop (`deeptime::mod::run_cells`) onto it as **self-declaring passes**,
byte-identical. Zero behavior change. The production goldens are the acceptance
instrument.

## Plan

1. Extract the topo-sort/validation mechanism out of `pipeline.rs` into a
   generic `passgraph` kernel (shared, not re-invented beside it — A-4).
2. Build `deeptime/runner.rs`: passes declare `{reads, writes, period}`, the
   runner topo-sorts by reads/writes and drives the epoch loop, firing each
   pass at its cadence and handing it `dt` (pinned this movement).
3. Convert the loop's four phases to declared passes: `climate`, `tectonics`,
   `erosion` (composite), `biotic`.
4. The biology↔erosion one-epoch lag becomes a **declared loop-carried edge**;
   the runner rejects it as a within-epoch cycle.
5. `climate`'s `remarch_interval` becomes a low-rate (coarse-cadence) pass.

## Results

(to be filled)
