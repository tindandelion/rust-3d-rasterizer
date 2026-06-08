//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and export defaults live here (**`still-scene`**, **`animated-scene`** default output — see **`doc/planning/project-spec.md`**).

use std::array;
use std::env;
use std::ffi::OsString;

pub mod framebuffer;
pub mod geometry;
pub mod lighting;
pub mod meshes;
pub mod ortho_camera;
pub mod webp_encoder;

pub use framebuffer::{FrameBuffer, Rgb};
pub use lighting::{BlinnLightModel, Material};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use crate::framebuffer::{PhongCorner, PhongShadedTriangle};
use crate::geometry::{Mesh, UnitVec3};

/// Raster width in pixels (golden stills / integration tests must agree).
pub const SCENE_WIDTH: u32 = 800;
/// Raster height in pixels (golden stills / integration tests must agree).
pub const SCENE_HEIGHT: u32 = 600;

pub const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Frame count for the **`animated-scene`** lossless WebP (integration tests must agree).
pub const ANIMATED_SCENE_FRAME_COUNT: u32 = 360;

/// Milliseconds between successive **`animated-scene`** frame timestamps (**`1000 / 20` ⇒ 50 fps**).
///
/// `libwebp` may merge identical consecutive frames in the mux, but the **animation timeline** (sum
/// of frame durations) still matches **`ANIMATED_SCENE_FRAME_COUNT ×` this value**.
pub const ANIMATED_SCENE_FRAME_SPACING_MS: i32 = 20;

/// Shared **`Rgb`** **base color** for filled mesh rendering ([`render_mesh`]) — saturated blue (**`#346ED2`**) tuned for diffuse shading.
pub const SHAPE_BASE_COLOR: Rgb = Rgb(52, 110, 210);

/// Output **`.webp`** path for export binaries: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
pub fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

/// Filled mesh: **[`PhongShadedTriangle::draw`](framebuffer::PhongShadedTriangle::draw)** per [`Triangle`] from **[`Mesh::visible_triangles`]**.
///
/// **[`BlinnLightModel::calc_intensity`]** runs **per pixel** on the interpolated normal with a constant **toward-eye** (orthographic **`Camera::direction`**); **`PhongShadedTriangle`** interpolates **`UnitVec3`** normals across the triangle and scales **`SHAPE_BASE_COLOR`** per fragment (**Phong**).
pub fn render_mesh(
    mesh: &Mesh,
    fb: &mut FrameBuffer,
    camera: &Camera,
    light_model: &BlinnLightModel,
) {
    let forward = camera.direction();
    let toward_eye: UnitVec3 = -camera.direction();
    for triangle in mesh.visible_triangles(forward) {
        let corners: [PhongCorner; 3] = array::from_fn(|i| PhongCorner {
            pos: camera.transform(triangle.corners[i]),
            normal: triangle.normals[i],
        });
        PhongShadedTriangle::new(corners, SHAPE_BASE_COLOR)
            .draw(fb, |normal| light_model.calc_intensity(normal, toward_eye));
    }
}
