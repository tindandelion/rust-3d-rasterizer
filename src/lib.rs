//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and **`still-cube`** default output live here (see **`doc/planning/project-spec.md`**).

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
pub mod ortho_camera;
pub mod scene;
pub mod webp_encoder;

pub use framebuffer::{FillQuad, FrameBuffer, Line, Rgb};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use scene::cube::{Cube, Edge, Quad};

/// Flat **`Rgb`** tint for **[`Cube::faces`] slot** **`i`** after **`[`Cube::default`]`**, in order:
///
/// **`0`** **`−Z`**, **`1`** **`+Z`**, **`2`** **`+X`**, **`3`** **`−X`**, **`4`** **`+Y`**, **`5`** **`−Y`**
/// (outward normals in **`[`scene::cube::Cube::default`]`**; [`Cube::transform`] keeps slots, only **`normal`** /
/// **`vertices`** move — paired with **`slot`** keys from [`Cube::visible_faces`]).
///
/// [Cube::faces]: scene::cube::Cube::faces
pub const CUBE_FACE_PALETTE: [Rgb; 6] = [
    Rgb(0, 0, 255),     // −Z — blue
    Rgb(255, 0, 0),     // +Z — red
    Rgb(237, 190, 77),  // +X — amber
    Rgb(155, 102, 210), // −X — iris
    Rgb(78, 198, 128),  // +Y — jade
    Rgb(234, 128, 196), // −Y — orchid
];

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

/// Fills [`Cube::visible_faces`] through **`camera`** (**[`FillQuad`]** per **strictly front‑facing** facet).
///
/// Each surviving facet is filled with **`CUBE_FACE_PALETTE[slot]`**, where **`slot`** (**`faces`** index **`0 … 5`**) matches [`Cube::visible_faces`].
pub fn draw_faces(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube) {
    let forward = camera.direction();
    for (face_idx, Quad(a, b, c, d)) in cube.visible_faces(forward) {
        let color = CUBE_FACE_PALETTE[face_idx];
        let corners = [
            camera.transform(a),
            camera.transform(b),
            camera.transform(c),
            camera.transform(d),
        ];
        FillQuad::new(corners, color).draw(fb);
    }
}
