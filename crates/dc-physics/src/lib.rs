//! Dynamic-tier physics: rapier3d islands around dynamic bodies (S6).
//!
//! Division of labor (docs/ARCHITECTURE.md § Physics): characters and simple
//! movers stay on dc-core's swept-AABB path — game feel is ours, not a physics
//! engine's. This crate owns the *dynamic tier*: dropped items, detachable
//! voxel props, (later) ragdolls. The voxel world is never a global collider
//! set; static colliders exist only inside **bubbles** around dynamic bodies
//! and are dropped the moment no bubble needs them (see [`bubble`]).
//!
//! Why direct `rapier3d` and not `bevy_rapier`: the parts a Bevy plugin layer
//! would own — the stepping loop, collider lifecycle, transform sync — are
//! exactly the parts this design has to own. Our colliders are transient
//! bubble tiles keyed to voxel data, our stepping is a fixed-timestep
//! accumulator inside our own schedule, our positions are f64 world meters
//! with a floating origin, and everything below dc-client must stay headless.
//! bevy_rapier's value (ECS component sync, Bevy-managed scheduling) is
//! exactly the coupling we would immediately fight. dc-client only ever sees
//! poses coming out of [`PhysicsWorld`].
//!
//! Determinism: rapier is built with `enhanced-determinism` (libm-forced
//! transcendentals, no SIMD paths), and every internal collection this crate
//! iterates while mutating the rapier sets is ordered (`BTreeMap`/`BTreeSet`),
//! so identical command sequences produce bit-identical trajectories — tested
//! in `tests/s6_physics.rs`, measured cost in docs/spikes/S6-results.md.
//!
//! Units: the public surface speaks f64 **meters** (world space) plus world
//! voxel coordinates for solidity, exactly like [`dc_core::collision`].
//! Voxel solidity always arrives through a caller-provided [`dc_core::VoxelQuery`]
//! closure — this crate never learns what a chunk is.

pub mod bubble;
pub mod merge;
pub mod prop;
pub mod world;

pub use bubble::BubbleConfig;
pub use merge::{VoxelBox, merge_boxes};
pub use world::{BodyPose, PhysicsConfig, PhysicsWorld, PropError, ReattachError, StepStats};

// The one rapier type callers hold on to. Everything else stays internal.
pub use rapier3d::dynamics::RigidBodyHandle;
