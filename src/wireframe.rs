//! Wireframe **Cube** helpers used by **`still-cube`** and **`animated-cube`** bins.

use glam::{Mat3, Mat4, Vec3};

use crate::scene::cube::{Cube, Edge};
use crate::{Camera, FrameBuffer, Rgb};

const TILT: f32 = std::f32::consts::FRAC_PI_4;

/// Rasterize every [`Cube::edges`] segment through **`camera`** (**DDA** in **[`FrameBuffer::draw_line`]**).
pub fn draw_edges(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    for Edge(a, b) in cube.edges() {
        fb.draw_line(camera.transform(a), camera.transform(b), color);
    }
}

/// Fixed pose for the orthographic still (**`still-cube`** snapshot): **π/4** tilt on **X** then **Y**, **0.5** uniform scale.
pub fn model_matrix_still() -> Mat4 {
    let base = Mat3::from_rotation_x(TILT) * Mat3::from_rotation_y(TILT);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(base * scale)
}
