//! Dev look-at inspector: an on-screen readout of the **full material contents**
//! of the voxel under the crosshair — the whole [`dc_core::VoxelContents`]
//! (structure / pore-fill / debris multisets, shape, occupancy), not the single
//! classified block name the world stores.
//!
//! It is the human-at-the-keyboard twin of dc-api's `dc:world/get_contents`
//! query: both unpack the same [`ContentsView`] from the same
//! `Authority::world` contents source (the render authority, a pure function of
//! pos). This one reads the looked-at voxel live and paints it to the screen;
//! the query serves the MCP/agent walker. Off by default — toggle with **F3**.
//!
//! Why this exists at all: `get_block` answers ONE name (`classify(contents)`),
//! a summary that hides the mixed composition the world now carries. The
//! palette-quantization station (ROADMAP Observed) needs to tell whether the
//! full contents are smoothly shared across a chunk seam or genuinely different
//! — a question the classified name cannot see. This is that instrument.

use bevy::prelude::*;
use dc_api::Identity;
use dc_api::payload::{ContentsView, MaterialCount};

use crate::authority::Authority;
use crate::edit::CrosshairTarget;

/// Key that toggles the look-at contents HUD.
const TOGGLE_KEY: KeyCode = KeyCode::F3;

/// Is the look-at contents HUD showing? Off by default (a dev instrument).
#[derive(Resource, Default)]
pub struct ContentsHud {
    pub enabled: bool,
}

/// Marks the HUD's text node.
#[derive(Component)]
pub struct ContentsHudText;

/// The HUD's per-gaze cache: the looked-at voxel and its formatted readout, so
/// a steady gaze does not re-resolve the chunk contents grid every frame.
type ReadoutCache = Option<((i64, i64, i64), String)>;

/// Spawn the (initially empty) HUD text node, pinned top-left over a dim panel.
pub fn setup_contents_hud(mut commands: Commands) {
    commands.spawn((
        Text::new(String::new()),
        TextFont::from_font_size(15.0),
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(6.0)),
            max_width: Val::Px(460.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ContentsHudText,
    ));
}

/// F3 flips the HUD on/off.
pub fn toggle_contents_hud(keys: Res<ButtonInput<KeyCode>>, mut hud: ResMut<ContentsHud>) {
    if keys.just_pressed(TOGGLE_KEY) {
        hud.enabled = !hud.enabled;
    }
}

/// Repaint the HUD from the crosshair target's full contents. Cheap by
/// construction: the composition is a pure function of the voxel, so we resolve
/// (and intern a whole chunk grid for) a voxel only when the looked-at voxel
/// *changes* — a cache miss, not every frame. The block name is re-read each
/// tick so a live edit under the crosshair shows immediately.
pub fn update_contents_hud(
    hud: Res<ContentsHud>,
    target: Res<CrosshairTarget>,
    mut authority: ResMut<Authority>,
    mut text_q: Query<&mut Text, With<ContentsHudText>>,
    mut cache: Local<ReadoutCache>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    if !hud.enabled {
        if !text.0.is_empty() {
            text.0.clear();
        }
        *cache = None;
        return;
    }
    let readout = match target.0 {
        None => {
            *cache = None;
            "look-at contents (F3)\n(nothing in reach)".to_string()
        }
        Some(hit) => {
            let key = hit.voxel;
            if let Some((cached_key, cached)) = cache.as_ref()
                && *cached_key == key
            {
                cached.clone()
            } else {
                let pos = dc_api::Vec3i::new(key.0, key.1, key.2);
                let block = dc_api::block_name(authority.world.block_at(pos));
                // The honest per-voxel answer, not the chunk-granular grid
                // read: `identify` is what makes the "(no contents record
                // here)" branch below REACHABLE for the unrecorded basement
                // (corrections #49 — it used to print `classified: dc:air`
                // over solid stone instead).
                let identity = authority.world.identify(pos);
                let s = format_readout(pos, block, &identity);
                *cache = Some((key, s.clone()));
                s
            }
        }
    };
    if text.0 != readout {
        text.0 = readout;
    }
}

/// Format the readout: header + block/classified names + shape/occupancy + one
/// line per non-empty role (structure / pore-fill / debris).
fn format_readout(pos: dc_api::Vec3i, block: &str, identity: &Identity) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(s, "look-at contents (F3)  @ {},{},{}", pos.x, pos.y, pos.z);
    let Some(c) = identity.mixture() else {
        let _ = write!(s, "block: {block}\n(no contents record here)");
        return s;
    };
    let classified = dc_api::block_name(dc_core::classify(c));
    let view = ContentsView::from_contents(c);
    let _ = writeln!(s, "block: {block}   classified: {classified}");
    let _ = writeln!(
        s,
        "shape: {}   solid {}/8   free {}   open-pores {}",
        view.shape, view.solid_eighths, view.free_eighths, view.open_pores
    );
    write_segment(&mut s, "structure", &view.structure);
    write_segment(&mut s, "pore-fill", &view.pore_fill);
    write_segment(&mut s, "debris", &view.debris);
    while s.ends_with('\n') {
        s.pop();
    }
    s
}

fn write_segment(s: &mut String, label: &str, seg: &[MaterialCount]) {
    use std::fmt::Write as _;
    if seg.is_empty() {
        return;
    }
    let _ = write!(s, "{label}: ");
    for (i, m) in seg.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ", ");
        }
        let _ = write!(s, "{} x{}", m.material, m.eighths);
    }
    let _ = writeln!(s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_core::{MaterialId, StructureShape, VoxelContents};

    /// The corrections #49 repair, at the HUD: an **unrecorded** voxel (solid
    /// stone under a thin record, sharing its chunk with recorded voxels) must
    /// reach the "(no contents record here)" branch — which was *unreachable*
    /// while the HUD read `contents_at` directly, because the chunk-granular
    /// grid handed it an ordinary empty composition and it printed
    /// `classified: dc:air` over the stone instead.
    #[test]
    fn unrecorded_reaches_the_no_record_branch() {
        let s = format_readout(
            dc_api::Vec3i::new(93_539, 296, 10_236),
            "dc:stone",
            &Identity::Unrecorded,
        );
        assert!(s.contains("(no contents record here)"), "{s}");
        assert!(!s.contains("dc:air"), "must not name air over stone: {s}");
        assert!(!s.contains("classified:"), "classify never runs here: {s}");
    }

    /// Do not overcorrect: a genuinely empty voxel (air) is a *record* saying
    /// nothing is here, and still reads as air.
    #[test]
    fn genuine_air_still_reads_as_air_in_the_hud() {
        let s = format_readout(
            dc_api::Vec3i::new(0, 400, 0),
            "dc:air",
            &Identity::Mixture(VoxelContents::EMPTY),
        );
        assert!(s.contains("classified: dc:air"), "{s}");
        assert!(!s.contains("no contents record"), "{s}");
    }

    /// A recorded mixture still prints its segments unchanged.
    #[test]
    fn a_recorded_mixture_still_prints_its_materials() {
        let c = VoxelContents::new(StructureShape::Full, &[MaterialId::GRANITE; 8], &[], &[])
            .unwrap();
        let s = format_readout(
            dc_api::Vec3i::new(1, 2, 3),
            "dc:granite",
            &Identity::Mixture(c),
        );
        assert!(s.contains("structure: dc:granite x8"), "{s}");
        assert!(s.contains("classified: dc:granite"), "{s}");
    }
}
