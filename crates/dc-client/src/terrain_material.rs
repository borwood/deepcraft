//! The LabPBR terrain material (ROADMAP PBR-1, docs/rendering/PIPELINE.md § 5).
//!
//! A custom Forward+ [`Material`]: three `texture_2d_array`s (basecolor /
//! normal+AO / specular) with **layer index = material id**, blended per
//! fragment from the mesher's per-vertex splat data (up to [`SPLAT_N`] material
//! layers + weights, plus a UV). The surface shader (shaders/terrain.wgsl)
//! heightlerps the LabPBR triplets and lights them with one directional sun +
//! hemispherical ambient. Fullbright is a *different* material (unlit vertex
//! color, app.rs) so the walk protocol keeps its pure-color diagnostic.
//!
//! The atlas is assembled at startup from the placeholder LabPBR packs
//! (assets/textures/placeholder-labpbr/<slug>/) — one layer per registry
//! material (slug = [`MaterialId::props`] name), then the block-only packs
//! ([`BLOCK_ONLY_SLUGS`]). A missing/oversized PNG degrades to a flat layer
//! synthesized from the material's registry albedo rather than crashing the
//! client (mirrors the shader-pack "never blank the screen" rule).

use std::path::PathBuf;

use bevy::asset::{RenderAssetUsages, load_internal_asset, uuid_handle};
use bevy::image::{Image, ImageSampler};
use bevy::mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat,
};
use bevy::shader::{Shader, ShaderRef};

use crate::meshing::{ATLAS_LAYER_COUNT, BLOCK_ONLY_SLUGS};
use dc_core::{MATERIAL_COUNT, MaterialId};

/// Embedded surface shader handle (in-tree, compiled into the binary — no
/// runtime asset-dir dependency, same robustness as the shader-pack loader).
const TERRAIN_SHADER: Handle<Shader> = uuid_handle!("7b9c1e2a-3d4f-4a5b-8c6d-0e1f2a3b4c5d");
/// Embedded fullbright surface shader handle (the unlit splat variant).
const FULLBRIGHT_SHADER: Handle<Shader> = uuid_handle!("2f8a6b3c-1d0e-4f9a-8b7c-5d4e3a2b1c0f");

/// Length of the fullbright albedo palette: one flat color per atlas layer.
/// Must equal [`ATLAS_LAYER_COUNT`] and the `array<..>` size in the fullbright
/// shader (kept in lockstep — `palette_len_matches_atlas` guards it).
pub const PALETTE_LEN: usize = MATERIAL_COUNT + BLOCK_ONLY_SLUGS.len();

/// Per-vertex material layer indices (up to 4), `@location(3)` in the shader.
pub const ATTRIBUTE_MAT_LAYERS: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_MatLayers", 0x0DC_0AB1, VertexFormat::Uint32x4);
/// Per-vertex splat weights (match `ATTRIBUTE_MAT_LAYERS`), `@location(4)`.
pub const ATTRIBUTE_MAT_WEIGHTS: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_MatWeights", 0x0DC_0AB2, VertexFormat::Float32x4);

/// Texture resolution of every placeholder pack (visuals.md § Material model;
/// "try 16×16"). One resolution per pack — the loader asserts it.
const TEX_SIZE: u32 = 16;
/// Placeholder pack root, relative to the client's working dir (repo root).
const TEX_ROOT: &str = "assets/textures/placeholder-labpbr";

/// Directional sun + hemispherical ambient, mirrored in the shader's
/// `TerrainLighting`. PBR-2 will drive these from a day/weather system; for now
/// the app sets them once to match the scene sun.
#[derive(Clone, Copy, ShaderType)]
pub struct TerrainLighting {
    /// xyz: normalized direction toward the sun; w unused.
    pub sun_dir: Vec4,
    /// rgb: sun color; a: intensity.
    pub sun_color: Vec4,
    /// rgb: ambient toward the sky (up).
    pub sky_color: Vec4,
    /// rgb: ambient toward the ground (down).
    pub ground_color: Vec4,
}

impl TerrainLighting {
    /// Scene default: a warm sun and a cool sky, tuned to stay under LDR
    /// clip on pale top faces (the walk-3 blowout was near-vertical sun with no
    /// tonemap shoulder — kept moderate here until HDR/tonemap lands in PBR-2).
    pub fn from_sun(sun_dir: Vec3) -> Self {
        Self {
            sun_dir: sun_dir.normalize_or_zero().extend(0.0),
            sun_color: Vec4::new(1.0, 0.96, 0.88, 0.95),
            sky_color: Vec4::new(0.42, 0.50, 0.62, 1.0),
            ground_color: Vec4::new(0.26, 0.24, 0.21, 1.0),
        }
    }
}

/// The terrain surface material. One instance is shared by every lit chunk
/// (near-field, far field, legacy S1); the per-voxel material selection lives
/// entirely in the mesh's splat attributes.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub lighting: TerrainLighting,
    #[texture(1, dimension = "2d_array")]
    #[sampler(4)]
    pub basecolor: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    pub normal: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    pub specular: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        TERRAIN_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        TERRAIN_SHADER.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_MAT_LAYERS.at_shader_location(3),
            ATTRIBUTE_MAT_WEIGHTS.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

/// Flat per-layer albedo palette for the fullbright variant: `albedo[layer]` is
/// the registry albedo of the material occupying that atlas layer (block-only
/// layers carry the block's side color). The fullbright shader reads this — NOT
/// the LabPBR basecolor textures — because "albedo-only is better for AI
/// viewers" (visuals.md § PBR-1 walk-14 ratifications).
#[derive(Clone, Copy, ShaderType)]
pub struct TerrainPalette {
    pub albedo: [Vec4; PALETTE_LEN],
}

/// The **fullbright** terrain material (ROADMAP PBR-1 walk-14 ratification): an
/// unlit splat variant for the walk protocol's screenshot auditability. Uniform
/// and block faces render one flat albedo (the mesh's vertex color); *mixed*
/// faces reproduce the old world-anchored 4×4-cell mixture speckle shader-side
/// (journal/0010), computed from the same splat attributes + a world-anchored
/// hash — zero mosaic geometry, zero lighting. It shares the chunk mesh with the
/// lit [`TerrainMaterial`]; the streamer picks one by the `--fullbright` flag.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FullbrightTerrainMaterial {
    #[uniform(0)]
    pub palette: TerrainPalette,
}

impl Material for FullbrightTerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        FULLBRIGHT_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        FULLBRIGHT_SHADER.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_MAT_LAYERS.at_shader_location(3),
            ATTRIBUTE_MAT_WEIGHTS.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

/// The flat side color of a block-only pack (grass/dirt/stone/wood), matching
/// the mesher's `face_color` side tones — the palette entry for a block-only
/// atlas layer. (Block-only layers never appear on a *mixed* face, so these
/// entries are only a defensive fill; the speckle only ever reads material
/// layers.)
fn block_only_albedo(slug: &str) -> Vec4 {
    match slug {
        "grass" => Vec4::new(0.38, 0.45, 0.22, 1.0),
        "dirt" => Vec4::new(0.42, 0.30, 0.19, 1.0),
        "stone" => Vec4::new(0.52, 0.52, 0.54, 1.0),
        "wood" => Vec4::new(0.44, 0.33, 0.17, 1.0),
        _ => Vec4::new(0.5, 0.5, 0.5, 1.0),
    }
}

/// Build the fullbright material's flat-albedo palette from the registry.
pub fn build_fullbright_material() -> FullbrightTerrainMaterial {
    let mut albedo = [Vec4::new(0.5, 0.5, 0.5, 1.0); PALETTE_LEN];
    for m in MaterialId::all() {
        let a = m.props().albedo;
        albedo[m.raw() as usize] = Vec4::new(a[0], a[1], a[2], 1.0);
    }
    for (i, slug) in BLOCK_ONLY_SLUGS.iter().enumerate() {
        albedo[MATERIAL_COUNT + i] = block_only_albedo(slug);
    }
    FullbrightTerrainMaterial {
        palette: TerrainPalette { albedo },
    }
}

/// Installs the embedded shaders and both material renderers (lit + fullbright).
pub struct TerrainMaterialPlugin;

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TERRAIN_SHADER,
            "shaders/terrain.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            FULLBRIGHT_SHADER,
            "shaders/terrain_fullbright.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
        app.add_plugins(MaterialPlugin::<FullbrightTerrainMaterial>::default());
    }
}

/// Atlas layer slugs in layer order: every registry material (slug = its
/// registry name), then the block-only packs.
fn layer_slugs() -> Vec<String> {
    let mut slugs: Vec<String> = MaterialId::all()
        .map(|m| m.props().name.to_string())
        .collect();
    slugs.extend(BLOCK_ONLY_SLUGS.iter().map(|s| (*s).to_string()));
    slugs
}

/// One 16×16 RGBA layer of a channel for `slug`, or a flat fallback synthesized
/// from the material's registry albedo when the PNG is missing/malformed.
fn load_layer(slug: &str, channel: Channel) -> Vec<u8> {
    let path: PathBuf = [TEX_ROOT, slug, channel.file()].iter().collect();
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(img) = image::load_from_memory(&bytes)
    {
        let rgba = img.to_rgba8();
        if rgba.width() == TEX_SIZE && rgba.height() == TEX_SIZE {
            return rgba.into_raw();
        }
        warn!(
            "terrain atlas: {} is not {TEX_SIZE}×{TEX_SIZE}; using fallback",
            path.display()
        );
    } else {
        warn!(
            "terrain atlas: {} missing; using fallback layer",
            path.display()
        );
    }
    channel.fallback(slug)
}

/// The three LabPBR channels; each knows its filename and its flat fallback.
#[derive(Clone, Copy)]
enum Channel {
    Basecolor,
    Normal,
    Specular,
}

impl Channel {
    fn file(self) -> &'static str {
        match self {
            Channel::Basecolor => "basecolor.png",
            Channel::Normal => "normal.png",
            Channel::Specular => "specular.png",
        }
    }

    /// Is this channel color data (sRGB) or linear packed data?
    fn srgb(self) -> bool {
        matches!(self, Channel::Basecolor)
    }

    /// Flat 16×16 fallback bytes. Basecolor uses the registry albedo (matching
    /// how the generator authored the real PNGs); normal is flat-up with full
    /// AO and mid height; specular is a dull dielectric with no emission.
    fn fallback(self, slug: &str) -> Vec<u8> {
        let px: [u8; 4] = match self {
            Channel::Basecolor => {
                let a = MaterialId::all()
                    .find(|m| m.props().name == slug)
                    .map_or([0.5, 0.5, 0.5], |m| m.props().albedo);
                [
                    (a[0] * 255.0) as u8,
                    (a[1] * 255.0) as u8,
                    (a[2] * 255.0) as u8,
                    255,
                ]
            }
            Channel::Normal => [128, 128, 255, 128],
            Channel::Specular => [100, 10, 32, 255],
        };
        px.iter()
            .copied()
            .cycle()
            .take((TEX_SIZE * TEX_SIZE * 4) as usize)
            .collect()
    }
}

/// Build one `texture_2d_array` from stacked per-layer bytes.
fn make_array(images: &mut Assets<Image>, channel: Channel, slugs: &[String]) -> Handle<Image> {
    let layers = slugs.len() as u32;
    let mut data = Vec::with_capacity((TEX_SIZE * TEX_SIZE * 4) as usize * slugs.len());
    for slug in slugs {
        data.extend_from_slice(&load_layer(slug, channel));
    }
    let format = if channel.srgb() {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Rgba8Unorm
    };
    // A vertically stacked 2D image, reinterpreted as an array (Bevy helper).
    let mut image = Image::new(
        Extent3d {
            width: TEX_SIZE,
            height: TEX_SIZE * layers,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    image
        .reinterpret_stacked_2d_as_array(layers)
        .expect("stacked atlas height is layers × TEX_SIZE by construction");
    // Pixel-game look: nearest filtering, no mip blur across the 16×16 texels.
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

/// Assemble the LabPBR atlases and return a ready terrain material for `sun_dir`
/// (direction toward the sun, world space).
pub fn build_terrain_material(images: &mut Assets<Image>, sun_dir: Vec3) -> TerrainMaterial {
    let slugs = layer_slugs();
    debug_assert_eq!(slugs.len() as u32, ATLAS_LAYER_COUNT);
    TerrainMaterial {
        lighting: TerrainLighting::from_sun(sun_dir),
        basecolor: make_array(images, Channel::Basecolor, &slugs),
        normal: make_array(images, Channel::Normal, &slugs),
        specular: make_array(images, Channel::Specular, &slugs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_slugs_cover_every_atlas_layer_in_order() {
        let slugs = layer_slugs();
        assert_eq!(slugs.len() as u32, ATLAS_LAYER_COUNT);
        // Material layers come first, in id order.
        for m in MaterialId::all() {
            assert_eq!(slugs[m.raw() as usize], m.props().name);
        }
        // Block-only packs follow.
        for (i, s) in BLOCK_ONLY_SLUGS.iter().enumerate() {
            assert_eq!(&slugs[MaterialId::all().count() + i], s);
        }
    }

    #[test]
    fn palette_len_matches_atlas() {
        // The fullbright palette has one flat color per atlas layer, and the
        // shader hardcodes this size in its `array<vec4<f32>, N>` — keep them
        // in lockstep with this guard.
        assert_eq!(PALETTE_LEN as u32, ATLAS_LAYER_COUNT);
        assert_eq!(PALETTE_LEN, 26);
    }

    #[test]
    fn fullbright_palette_is_registry_albedo() {
        let m = build_fullbright_material();
        for mat in MaterialId::all() {
            let a = mat.props().albedo;
            let p = m.palette.albedo[mat.raw() as usize];
            assert_eq!([p.x, p.y, p.z], a, "layer {} albedo", mat.raw());
        }
    }

    #[test]
    fn fallback_layers_are_the_right_size() {
        for ch in [Channel::Basecolor, Channel::Normal, Channel::Specular] {
            assert_eq!(
                ch.fallback("sandstone").len(),
                (TEX_SIZE * TEX_SIZE * 4) as usize
            );
        }
    }
}
