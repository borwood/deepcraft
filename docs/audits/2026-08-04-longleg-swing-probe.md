# The longleg mid-swing knee wobble — a diagnostic probe

**Measured 2026-08-04.** Diagnostic only: no fix, no design, no ruling. Every number
below is printed by `swing_probe::gate::the_longleg_swing_is_measured`
(`cargo test -p dc-client --release --bin dc-client swing_probe -- --nocapture`).

*Line numbers in `body.rs` / `character.rs` are as of `35bf942`. A concurrent slice was
hoisting per-frame work into `BodyAssets` in both files while this ran; the **symbols** are
the durable references, the line numbers may have moved.*

## The observation (user, 2026-08-04 walk, station 2, `--anim-fps 60`)

> *"the longer legged one has a little weirdness in the forward swing of walking leg
> which i dont see in the stout — and it may have to do with IK adjusting the leg
> mid-swing. what it looks like is the leg doing a **mini bend and correction back to
> straight and back to bent** during the swing forward. very hard to see in realtime,
> i would have to look at slow motion to really catch what's going on. it's non
> blocking at any rate."*

Conditions: `dc:body/longleg` and `dc:body/stout`, `speed: 0.3` × `walk_speed_m_s = 4.5`
= **1.35 m/s**, level stone, `--anim-fps 60`.

## The answer in one line

**The knee jumps 5.0200° at cycle phase u = 0.9980 and again 5.0195° at u = 0.5025, plus
four smaller jumps of 1.01–1.12° in between — six per cycle, every one of them at the
instant `character.rs:313`'s `adjust.abs() > 1e-3` guard flips the leg between the gait's
pose and the IK's.** It is not the half-voxel window, not `d_min`, and not quantization.
It is a **1 mm** threshold sitting exactly on the two-bone solver's singular point,
because the derived standing root puts a grounded sole **exactly on** the annulus
boundary.

---

## 1. Where the discontinuity is

`dc:body/longleg`, continuous geometry, 1.35 m/s (phases are `u`, the limb's own cycle
position; `u < 0.5` is stance, `u ≥ 0.5` is swing):

| `u` | transition | knee jump |
|---|---|---|
| **0.5025** | `seat → ik:OK` | **5.0195°** |
| 0.6635 | `ik:OK → seat` | 1.0125° |
| 0.6740 | `seat → ik:OK` | 1.1246° |
| 0.8265 | `ik:OK → seat` | 1.1234° |
| 0.8370 | `seat → ik:OK` | 1.0138° |
| **0.9980** | `ik:OK → seat` | **5.0200°** |

All six are **inside the swing**. The two 5° jumps sit 2.5 ‰ of a cycle from the swing's
two ends (liftoff and touchdown); the four 1.0–1.1° ones are two brief re-entries into
the seated band at u ≈ 0.66 and u ≈ 0.83.

**In metres.** The *sole* barely moves — the rendered sole's whole-stride envelope is
**[−0.91 mm, +0.99 mm]** and the largest adjacent-sample sole step is **0.99 mm**. The
foot is pinned; what visibly moves is the **knee joint**. Across the boundary the hip goes
`22.349° → 19.944°` while the knee goes `−5.165° → −0.145°` (rows `u = 0.9975 → 0.9980`),
so the knee joint translates **2.405° × 0.520 m = 21.8 mm** in one sample while the sole
stays put — the leg visibly snaps from bent to straight with the foot planted. Quantized
to the shipped 60 fps grid the knee step is larger still: **12.2558°** in a single
held-pose change.

**The shape, which is the user's description exactly.** The rendered swing knee track,
one entry per distinct pose, `*` = seated/clip pose (longleg, continuous, 1.35 m/s):

```
0.0* -11.2 -14.9 -17.1 -18.4 -19.1 -19.2 -19.0 -18.4 -17.5 -16.4 -15.1 -13.7 -12.2*
-10.3 -8.5 -6.6 -4.6 -2.6 -0.5 -1.6 -3.7 -5.7 -7.6 -9.5 -11.3 -13.0 -14.5 -15.9
-17.1 -18.0 -18.7 -19.1 -19.2 -18.8 -17.8 -16.1 -13.2 -7.9
```

It is a **W**: bend to −19.2°, **back to −0.5° at mid-swing**, bend to −19.2° again,
straighten at touchdown. The gait's own knee over the same swing is a *single* triangle,
**most bent at mid-swing** — `traversal` returns `blend = 1 − |2s − 1|`
(`gait/evaluate.rs:191-197`) and `limb_pose` lerps the straight neutral toward the flexed
`clearance` pose by exactly that (`:250-264`). **The foot IK therefore inverts the knee
profile: the rendered leg is straightest precisely where the gait wants it most bent.**
"A mini bend and correction back to straight and back to bent" is a literal description of
this row.

## 2. Which recorded quantity changes state at that phase

`Sample.decision` — the mirror of `character.rs:309-337`. At every one of the six phases
it flips between `Seated` and `Applied(Ok)`, and **nothing else changes state anywhere in
the cycle**: `Reach` is `Ok` at every applied sample, the window is never left, no limit
is ever hit.

The gate makes this a standing claim rather than an observation.
`the_only_swing_discontinuity_is_the_placement_boundary` measures the largest adjacent
knee step **between samples that decided the same way** and asserts it refines with the
sample grid. Measured: **0.4807° at 1 000 samples → refines** (longleg), 0.5857°
(stout), 0.5129° (biped). The raw statistic does *not* refine — 5.5008° → 5.0019° across
a 4× refinement — which is what makes the boundary a genuine C0 discontinuity rather than
a steep slope.

### The mechanism, with the arithmetic

1. `character.rs:312-313` computes `adjust = ground − foot_y` and applies the IK only
   when `adjust.abs() > 1e-3`. Below 1 mm the **gait's** pose renders; above it, the
   **solver's**.
2. At 1.35 m/s the gait's own sole crosses that 1 mm band **six times a cycle**: its
   whole-stride envelope is **[−13.49 mm, +12.66 mm]** (it dips *below* the ground in
   early and late swing because the root bob drops the hip while the swing leg's
   angle-lerp keeps it near-straight, and rises to the derived Winter clearance at
   mid-swing).
3. The two poses on either side of the guard are **not** near-equal in joint space, even
   though they are within 1 mm in foot space, because the leg is at **full extension**
   there. The measured target distance at the boundary is `d_planar = 1.01896 m` against
   `reach = 1.02000 m` — **1.04 mm inside the annulus**. The two-bone knee for that
   offset is `φ ≈ 2·√(Δ·(l1+l2)/(l1·l2)) = 0.0903 rad = 5.17°`, against a measured
   **5.02°**. `dφ/dd → ∞` as `φ → 0`: the guard sits on the solver's singular point.
4. It sits there **by construction**, not by accident: the posture bake stands every body
   at chain reach (`hip_derived_m == reach_m`, posture-bake audit § 2.2), so a sole on the
   ground is at `d = reach` **exactly**. The probe measures `d_planar / reach max =
   1.00000` on all three plans at walking speed — the leg lives on the boundary.

## 3. Which candidate it is

| candidate | verdict | the number |
|---|---|---|
| **1. half-voxel override window** (`character.rs:313`, 0.45 m) | **RULED OUT — 33×** | the gait's sole never leaves **[−13.49, +12.66] mm**. It would need 450 mm to reach the window's edge. `OutsideWindow` never occurs at any sampled phase. |
| **2. B7 `d_min` sector boundary** (`body.rs:884`) | **RULED OUT — 5.2×** | `min(d_planar)/d_min` = **5.245** (longleg), **1.812** (stout), **4.527** (biped). `BeyondFlexion` is never returned; `refused_by_limits` is 0 at every sample. |
| **3. per-cycle quantization** (journal/0155) | **RULED OUT as the cause; it is an AMPLIFIER** | the discontinuity is fully present in the **continuous** trace (5.0200°), and its **phase does not move with the pose grid** — identical at `--anim-fps 60` (N = 63) and `--anim-fps 12` (N = 13). What quantization does is *enlarge* the visible step: 5.0200° continuous → **12.2558°** at 60 fps → **19.2112°** at 12 fps, because the held pose lands further down the steep stretch. |
| **4. a fourth thing** | **THIS ONE** | the **1 mm seated guard** at `character.rs:313`, sitting on the solver's singular point because the derived root puts a grounded sole exactly on the annulus boundary. Neither the guard nor the root placement is wrong on its own; their composition is what jumps. |

`swing_gain` (stubs #43) is **not** the cause but it is in the loop: below a normal walk it
fades the mid-swing lift, which is part of why the sole sits close enough to the ground to
cross the 1 mm band six times. The clearance blend itself is C0-continuous — its kink at
mid-swing is C1 and shows as a slope change, not a step, which is why the within-decision
statistic refines cleanly.

## 4. Why the stout "does not show it" — the comparative does NOT reproduce

**In every angular measure the stout's artifact is LARGER than the longleg's.**

| | `longleg` | `stout` | `biped` |
|---|---|---|---|
| reach (l1 + l2) | 0.520 + 0.500 = **1.020** | 0.225 + 0.215 = **0.440** | 0.450 + 0.430 = **0.880** |
| cycle at 1.35 m/s | **1.0426 s** | **0.5788 s** | 0.9402 s |
| `N` at 60 fps | 63 | 35 | 56 |
| continuous max knee jump | **5.0200°** | **7.3098°** | 5.3573° |
| rendered max knee jump @ 60 fps | 12.2558° | **20.7422°** | 13.7435° |
| rendered max knee jump @ 12 fps | 19.2112° | **25.8985°** | 20.2125° |
| decision changes / cycle (continuous) | 6 | 6 | 6 |
| `min d_planar / d_min` | 5.245 | 1.812 | 4.527 |
| gait sole envelope | [−13.49, +12.66] mm | [−10.96, +6.60] mm | [−13.29, +11.93] mm |
| rendered sole envelope | [−0.91, +0.99] mm | [−0.96, +0.99] mm | [−0.94, +0.98] mm |

The stout's swing knee track has the identical W:
`0.0* -15.0 -20.0 -23.0 -24.8 -25.7 -25.9 … -0.6 … -25.9 -25.3 -24.0 -21.6 -17.7 -10.5`.

**So the honest finding is: the artifact is on all three bodies, and the geometry gives no
reason for the longleg to show it and the stout not to.** What the numbers *do* say is why
it is **legible** on the longleg and not on the stout:

- **Duration.** The longleg's swing lasts **0.5213 s**, the stout's **0.2894 s** — 1.80×.
  The bend→straight→bend traverse occupies `u ∈ [0.63, 0.87]` on both, which is **0.25 s**
  of wall clock on the longleg and **0.14 s** on the stout.
- **Resolution.** At 60 fps the longleg's swing is drawn in **31–32 distinct held poses**,
  the stout's in **17–18**. The longleg's wobble is *rendered*; the stout's is closer to
  being swallowed by its own staircase.
- **Screen size.** The moving bones are 2.31× / 2.33× longer (`l1` 0.520 vs 0.225, `l2`
  0.500 vs 0.215), so the same angular error is 2.3× more displacement.

⚠ **That last paragraph is an INFERENCE about visibility, not a measurement.** The probe
can say the geometry does not distinguish the two bodies; it cannot say what a human eye
resolves. **I cannot determine from these numbers why the user saw it on one and not the
other** — a slow-motion capture of both bodies is what would settle it, and the user
already named that as the missing instrument.

## 5. How the phase moves with speed — and what that discriminates

`dc:body/longleg`, continuous, 60 fps:

| speed fraction | v (m/s) | Fr | jump phases (`u`) | max knee jump | gait sole envelope |
|---|---|---|---|---|---|
| 0.15 | 0.675 | 0.04553 | 0.5090, 0.6430, 0.6740, 0.8265, 0.8575, **0.9915** | **4.7580°** | [−3.85, +5.52] mm |
| 0.30 | 1.350 | 0.18214 | 0.5025, 0.6635, 0.6740, 0.8265, 0.8370, **0.9980** | **5.0200°** | [−13.49, +12.66] mm |
| 0.50 | 2.250 | 0.50594 | 0.2275, 0.2730 (**stance**, 0.1605° each); 0.7480, 0.7525 (swing, **0.0000°**) | **0.1605°** | [0.00, +118.98] mm |

**The phase does not move with speed** across the walk band — the six crossings sit within
0.03 of a cycle of each other at half and full walking speed. That is the strongest single
discrimination available: an aliasing/beat artifact (candidate 3) would slide its phase as
the cycle length changed (1.3757 s → 1.0426 s here, a 24 % change), and it does not. A
*geometric* boundary crossing stays put, and it does.

**At 0.50 the artifact vanishes — by 31×, and the mechanism is named.** `Fr = 0.50594`
crosses `transition_fr`, so `root_height_ratio_at` returns `None` and the root stops
bobbing (`body.rs:1131`'s documented one step in an otherwise continuous ladder,
`stubs.md` #39). With the hip no longer dropping, the gait's sole never goes below ground
at all — envelope **[0, +118.98] mm** — the leg is far from full extension all swing, and
the same 1 mm guard now costs **0.1605°** instead of 5.02°. Those two 0.1605° crossings
are in **stance**; the two that remain in the swing are `ik:ex ↔ ik:OK` at `u = 0.7480`
and `0.7525` and carry a knee jump of **0.0000°**.

*(Above the transition the swing shows a different artifact, out of scope here and
recorded so nobody re-derives it: the rendered swing knee is **one constant pose, −0.2°**
— dead straight — because every swing sample returns `BeyondExtension` (`d_planar / reach
max = 1.10439`) and the solver clamps. The foot then hovers at up to **+96.42 mm**.)*

## 6. What this measurement says about the corpus

**Nothing measured here falsifies a recorded claim.** Two things it *sharpens*, offered to
the integrator rather than asserted:

1. **The derived foot clearance does not reach the screen at walking speed.** The gait
   derives a mid-swing lift from Winter (1992) — `CLEARANCE_RATIO = 0.015` of reach, i.e.
   15.3 mm for the longleg — and its own sole reaches **+12.66 mm** at mid-swing (the rest
   is `swing_gain`'s below-normal-walk fade, `stubs.md` #43). The **rendered** sole
   envelope is **[−0.91 mm, +0.99 mm]**. The foot IK pins the sole to the ground for the
   entire swing: **92 % of the lift the gait computed is cancelled before it is drawn**
   (94 % against the published 15.3 mm). This is not a
   contradiction of anything written (no document claims the clearance survives IK), but
   `posture-gait.md` § 3.2's derivation and `stubs.md` #43's blast radius (*"every body's
   foot height below a normal walk"*) are both written as if the number reaches the
   viewer, and at 1.35 m/s it does not.
2. **B7 § 5.3's predicted `d_min` table is confirmed a second time, from a third
   direction** — longleg **0.19175** (predicted 0.19199), stout **0.23664** (0.23660),
   biped **0.19131** (0.19143), matching the audit's own banner. No banner is owed.

## 7. The instrument

`crates/dc-client/src/swing_probe.rs`, a `#[cfg(test)]` module wired at
`crates/dc-client/src/main.rs:67`.

**Why not `examples/ … test = true`** (CLAUDE.md § Gates' pattern): `dc-client` is a
**binary** crate with no lib target, so an example cannot reach `body.rs`'s solver at all.
This is the same *"prints from the test under `--nocapture`"* answer `body::retarget_report`
already gives, and it costs the shipped binary nothing.

It drives the production path — `pose_for`, `root_offset_m`, `fk_foot_local`,
`solve_leg_ik`, `leg_rigs`, `derived_gait` — and mirrors exactly two pieces that need a
bevy world: `character.rs::ground_top_m` (level ground has a closed form) and
`character.rs`'s window/limit decision. Both mirrors are named in the module doc; they are
the same honest cost `retarget_report` pays.

Three tests, **0.14 s total added to the gate**:

- `the_longleg_swing_is_measured` — the report above, printed under `--nocapture`.
  Asserts nothing a magnitude could move.
- `the_only_swing_discontinuity_is_the_placement_boundary` — **the scale-free invariant.**
  The largest adjacent knee step between samples that decided the *same* way must divide
  by ≈ 4 when the phase grid is refined 4×. Scale-free because it is a statement about a
  per-phase function of one body's own geometry: no world is built, and it holds at any
  sample count, speed or plan. A *second* discontinuity — a pole flip, a `d_min` refusal,
  a clearance seam — lands here.
- `the_placement_decision_is_a_function_of_phase` — the same phase must decide the same
  way at 500 and 1 000 samples, or every number above is suspect.

**One instrument bug, recorded because it produced a silent null on the first run:** the
swing-track printer seeded its "has the value changed" comparison with `f64::NAN`, and
`(x − NaN).abs() > eps` is **false**, so the row printed as *"0 distinct poses"* — an
empty result that looked like a legitimate absence. Fixed to an `Option`.
