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
pub mod shapes;
pub mod webp_encoder;

pub use framebuffer::{FrameBuffer, Rgb};
pub use lighting::{BlinnShadingModel, Material};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use crate::framebuffer::ShadedCorner;
use crate::framebuffer::ShadedFillTriangle;
use crate::geometry::UnitVec3;

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

/// Shared **`Rgb`** **base color** for filled mesh rendering ([`draw_facets`]) — saturated blue (**`#346ED2`**) tuned for diffuse shading.
pub const SHAPE_BASE_COLOR: Rgb = Rgb(52, 110, 210);

/// Output **`.webp`** path for export binaries: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
pub fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

type Vertex = glam::Vec3;

/// One strictly front-filled **triangle** in world space: **`corners`** plus per-vertex **[`UnitVec3`]** normals for shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub corners: [Vertex; 3],
    pub normals: [UnitVec3; 3],
}

pub trait TriMesh {
    fn visible_facets(&self, view_direction: UnitVec3) -> impl Iterator<Item = Triangle> + '_;
}

/// Filled mesh: **[`ShadedFillTriangle::draw`](framebuffer::ShadedFillTriangle::draw)** per [`Triangle`] from **[`TriMesh::visible_facets`]**.
///
/// **[`BlinnShadingModel::calc_intensity`]** runs at each corner on **`triangle.normals[i]`** with per-vertex **toward-eye**; **`ShadedFillTriangle`** interpolates intensity across the triangle and scales **`SHAPE_BASE_COLOR`** per pixel (**Gouraud**). **Cube** / **dodecahedron** duplicate the facet normal at all three corners, so shading stays **faceted**.
pub fn draw_facets(
    fb: &mut FrameBuffer,
    camera: &Camera,
    mesh: &impl TriMesh,
    shading_model: &BlinnShadingModel,
) {
    let forward = camera.direction();
    let toward_eye: UnitVec3 = -camera.direction();
    for triangle in mesh.visible_facets(forward) {
        let shaded_corners: [ShadedCorner; 3] = array::from_fn(|i| {
            let vertex = triangle.corners[i];
            let vertex_normal = triangle.normals[i];

            let intensity = shading_model.calc_intensity(vertex_normal, toward_eye);
            ShadedCorner {
                pos: camera.transform(vertex),
                intensity,
            }
        });
        ShadedFillTriangle::new(shaded_corners, SHAPE_BASE_COLOR).draw(fb);
    }
}
