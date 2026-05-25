//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and export defaults live here (**`still-cube`**, **`animated-scene`** default output — see **`doc/planning/project-spec.md`**).

use std::array;
use std::env;
use std::ffi::OsString;

pub mod framebuffer;
pub mod geometry;
pub mod lighting;
pub mod ortho_camera;
pub mod scene;
pub mod webp_encoder;

pub use framebuffer::{FillTriangle, FrameBuffer, Rgb};
pub use lighting::DiffuseLight;
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use crate::geometry::Normal3;

/// Raster width in pixels (golden stills / integration tests must agree).
pub const SCENE_WIDTH: u32 = 800;
/// Raster height in pixels (golden stills / integration tests must agree).
pub const SCENE_HEIGHT: u32 = 600;

pub const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Frame count for the **`animated-scene`** lossless WebP (integration tests must agree).
pub const ANIMATED_SCENE_FRAME_COUNT: u32 = 720;

/// Milliseconds between successive **`animated-scene`** frame timestamps (**`1000 / 20` ⇒ 50 fps**).
///
/// `libwebp` may merge identical consecutive frames in the mux, but the **animation timeline** (sum
/// of frame durations) still matches **`ANIMATED_SCENE_FRAME_COUNT ×` this value**.
pub const ANIMATED_SCENE_FRAME_SPACING_MS: i32 = 20;

/// Single **`Rgb`** **albedo** for filled cube rendering ([`draw_faces`]) — saturated blue (**`#346ED2`**) tuned for diffuse shading.
pub const CUBE_ALBEDO: Rgb = Rgb(52, 110, 210);

/// Output **`.webp`** path for export binaries: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
pub fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

type Vertex = glam::Vec3;

/// One strictly front-filled **triangle** in world space: **`corners`** + outward **facet** **[`Normal3`]**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub corners: [Vertex; 3],
    pub normal: Normal3,
}

pub trait TriMesh {
    fn visible_facets(&self, view_direction: Normal3) -> impl Iterator<Item = Triangle> + '_;
}

/// Filled mesh: **[`FillTriangle::draw`](framebuffer::FillTriangle::draw)** per [`Triangle`] from **[`TriMesh::visible_facets`]**, with **[`DiffuseLight::calc_intensity`]** on each facet normal.
///
/// **[`DiffuseLight::calc_intensity`]** consumes **`triangle.normal`**; shaded color **`CUBE_ALBEDO`** · intensity.
pub fn draw_faces(
    fb: &mut FrameBuffer,
    camera: &Camera,
    mesh: &impl TriMesh,
    light: &DiffuseLight,
) {
    let forward = camera.direction();
    for triangle in mesh.visible_facets(forward) {
        let intensity = light.calc_intensity(triangle.normal);
        let color = CUBE_ALBEDO.scale(intensity);
        let corners: [glam::UVec2; 3] = array::from_fn(|i| camera.transform(triangle.corners[i]));
        FillTriangle::new(corners, color).draw(fb);
    }
}
