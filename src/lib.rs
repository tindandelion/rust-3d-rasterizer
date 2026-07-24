//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.
//!
//! Shared raster canvas size and **`animated-scene`** timing constants live here (see **`doc/planning/project-spec.md`**).

use std::array;

use glam::Vec3;

pub mod framebuffer;
pub mod geometry;
pub mod lighting;
pub mod meshes;
pub mod ortho_camera;
pub mod shaders;
pub mod webp_encoder;

pub use framebuffer::{FrameBuffer, Rgb};
pub use lighting::{Light, Material};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;

use crate::framebuffer::{Interpolatable, ShadedCorner, ShadedTriangle};
use crate::geometry::{Mesh, UnitVec3};
use crate::lighting::{Color, DistanceFalloff};
use crate::shaders::{GouraudShader, PhongShader};

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
/// Export-bin **`MeshPhongMaterial`** palette: emissive **`0x072534`**, diffuse **`0x156289`**,
/// specular **`0x444444`**, **`shininess` 30**.
pub fn default_material() -> Material {
    Material::from_rgb(
        Rgb::from_hex(0x072534),
        Rgb::from_hex(0x156289),
        Rgb::from_hex(0x444444),
        Some(30),
    )
}

/// Default export-bin lights — geometry-browser positions adapted to **LHS** (see **`project-breakdown.md`**).
///
/// One **`Light::directional`** toward **`(0, 2, 0)`** at **`intensity` 0.5**, plus two **`Light::point`**
/// at **`(1, 2, −1)`** and **`(-1, −2, 1)`** at **`intensity` 3.0** each with **`DistanceFalloff`**
/// **`{ constant: 0.5, linear: 0.0, quadratic: 1.0 }`** (see **`project-breakdown.md`** **Phase 2 reference palette**).
pub fn default_lights() -> [Light; 3] {
    let light_falloff = DistanceFalloff {
        constant: 0.5,
        linear: 0.0,
        quadratic: 1.0,
    };
    [
        Light::directional(Vec3::new(0.0, 2.0, 0.0).into(), 1.0),
        Light::point(Vec3::new(1.0, 2.0, -1.0), 6.0, light_falloff),
        Light::point(Vec3::new(-1.0, -2.0, 1.0), 6.0, light_falloff),
    ]
}

impl Material {
    pub fn from_rgb(emissive: Rgb, diffuse: Rgb, specular: Rgb, shininess: Option<i32>) -> Self {
        Self::new(
            Color::from(emissive),
            Color::from(diffuse),
            Color::from(specular),
            shininess,
        )
    }
}

trait Shader {
    type VertexData: Interpolatable;

    fn shade_vertex(&self, position: Vec3, normal: UnitVec3) -> Self::VertexData;
    fn shade_pixel(&self, data: Self::VertexData) -> Color;
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

    pub fn render_gouraud(&self, fb: &mut FrameBuffer, camera: &Camera, lights: &[Light]) {
        let shader = GouraudShader {
            material: &self.material,
            lights,
            toward_eye: -camera.direction(),
        };
        self._render(fb, camera, &shader);
    }

    pub fn render_phong(&self, fb: &mut FrameBuffer, camera: &Camera, lights: &[Light]) {
        let shader = PhongShader {
            material: &self.material,
            lights,
            toward_eye: -camera.direction(),
        };
        self._render(fb, camera, &shader);
    }

    fn _render<S: Shader>(&self, fb: &mut FrameBuffer, camera: &Camera, shader: &S) {
        let forward = camera.direction();
        for triangle in self.mesh.visible_triangles(forward) {
            let corners = array::from_fn(|i| ShadedCorner {
                pixel: camera.transform(triangle.corners[i]),
                value: shader.shade_vertex(triangle.corners[i], triangle.normals[i]),
            });
            ShadedTriangle::new(corners)
                .draw(fb, |surface_pt| Rgb::from(shader.shade_pixel(surface_pt)));
        }
    }
}
