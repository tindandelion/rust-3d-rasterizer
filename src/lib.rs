//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and **`still-cube`** default output live here (see **`doc/planning/project-spec.md`**).

use std::array;
use std::env;
use std::ffi::OsString;

/// Raster width in pixels (golden stills / integration tests must agree).
pub const SCENE_WIDTH: u32 = 800;
/// Raster height in pixels (golden stills / integration tests must agree).
pub const SCENE_HEIGHT: u32 = 600;

pub const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Frame count for the **`animated-cube`** lossless WebP (integration tests must agree).
pub const ANIMATED_CUBE_FRAME_COUNT: u32 = 360;

pub mod framebuffer;
pub mod geometry;
pub mod lighting;
pub mod ortho_camera;
pub mod scene;
pub mod webp_encoder;

pub use framebuffer::{FillTriangle, FrameBuffer, Line, Rgb};
pub use lighting::DiffuseLight;
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use scene::cube::{Cube, Edge};

/// Single **`Rgb`** **albedo** for filled cube rendering ([`draw_faces`]) — saturated blue (**`#346ED2`**) tuned for diffuse shading.
pub const CUBE_ALBEDO: Rgb = Rgb(52, 110, 210);

/// Output **`.webp`** path for export binaries: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
pub fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

/// Rasterize [`scene::cube::Cube::visible_edges`] through **`camera`** (**DDA** in [`Line::draw`]).
///
/// View direction comes from [`Camera::direction`] (same axis used for **front‑facing** classification as [`Cube::visible_faces`]).
pub fn draw_edges(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    let forward = camera.direction();
    for Edge(a, b) in cube.visible_edges(forward) {
        Line::new(camera.transform(a), camera.transform(b), color).draw(fb);
    }
}

/// Filled **`Cube`**: **`FillTriangle::draw`** per **`Triangle`** yielded by **`[`scene::cube::Cube::visible_faces`]** (**[`crate::scene::facet::Facet::is_front_facing`]**, same classifier as **`[`scene::cube::Cube::visible_edges`]**).
///
/// **[`DiffuseLight::calc_intensity`]** consumes **`triangle.normal`**; shaded color **`CUBE_ALBEDO`** · intensity.
pub fn draw_faces(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, light: &DiffuseLight) {
    let forward = camera.direction();
    for triangle in cube.visible_faces(forward) {
        let intensity = light.calc_intensity(triangle.normal);
        let color = CUBE_ALBEDO.scale(intensity);
        let corners: [glam::UVec2; 3] = array::from_fn(|i| camera.transform(triangle.corners[i]));
        FillTriangle::new(corners, color).draw(fb);
    }
}
