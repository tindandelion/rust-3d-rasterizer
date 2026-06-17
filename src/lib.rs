//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and **`animated-scene`** timing constants live here (see **`doc/planning/project-spec.md`**).

use std::array;

pub mod framebuffer;
pub mod geometry;
pub mod lighting;
pub mod meshes;
pub mod ortho_camera;
pub mod webp_encoder;

pub use framebuffer::{FrameBuffer, Rgb};
pub use lighting::{DirectionalLight, Material};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use crate::framebuffer::{PhongCorner, PhongShadedTriangle};
use crate::geometry::{Mesh, UnitVec3};

/// Raster width in pixels (golden stills / integration tests must agree).
pub const SCENE_WIDTH: u32 = 800;
/// Raster height in pixels (golden stills / integration tests must agree).
pub const SCENE_HEIGHT: u32 = 600;

/// Frame count for the **`animated-scene`** lossless WebP (integration tests must agree).
pub const ANIMATED_SCENE_FRAME_COUNT: u32 = 360;

/// Milliseconds between successive **`animated-scene`** frame timestamps (**`1000 / 20` ⇒ 50 fps**).
///
/// `libwebp` may merge identical consecutive frames in the mux, but the **animation timeline** (sum
/// of frame durations) still matches **`ANIMATED_SCENE_FRAME_COUNT ×` this value**.
pub const ANIMATED_SCENE_FRAME_SPACING_MS: i32 = 20;

/// Geometry-browser scene background (**`0x444444`**).
pub const SCENE_BACKGROUND: Rgb = Rgb::from_hex(0x444444);

/// Default export-bin surface material.
///
/// Diffuse and emissive match geometry-browser **`MeshPhongMaterial`** (**`0x156289`**, **`0x072534`**).
/// **Specular / shininess are tentative** (**`0x444444`**, **`100`**) until lighting-equation parity
/// with Three.js lands — see **`Lighting parity — Three.js equation alignment (tentative)`** in
/// **`doc/planning/project-breakdown.md`**. Browser defaults: specular **`0x111111`**, **`shininess` 30**.
pub fn default_material() -> Material {
    Material::new(
        Rgb::from_hex(0x072534),
        Rgb::from_hex(0x156289),
        Rgb::from_hex(0x444444),
        Some(100),
    )
}

/// A posed **[`Mesh`]** plus surface **[`Material`]** for filled rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub mesh: Mesh,
    pub material: Material,
}

impl Shape {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self { mesh, material }
    }

    pub fn render(&self, fb: &mut FrameBuffer, camera: &Camera, light: &DirectionalLight) {
        let forward = camera.direction();
        let toward_eye: UnitVec3 = -camera.direction();
        let material = self.material;
        for triangle in self.mesh.visible_triangles(forward) {
            let corners: [PhongCorner; 3] = array::from_fn(|i| PhongCorner {
                point: camera.transform(triangle.corners[i]),
                normal: triangle.normals[i],
            });
            PhongShadedTriangle::new(corners)
                .draw(fb, |normal| material.shade(light, normal, toward_eye));
        }
    }
}
