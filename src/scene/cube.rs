//! Axis-aligned unit-scale cube at the origin: eight vertices, twelve edges, wireframe draw.

use glam::{Mat3, Vec3};

use crate::{Camera, FrameBuffer, Rgb};

/// Half of the cube edge length (`0.5 / 2`) in world coordinates.
const HALF_EXTENT: f32 = 0.25;

const VERTS: [Vec3; 8] = [
    Vec3::new(-HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
    Vec3::new(HALF_EXTENT, -HALF_EXTENT, -HALF_EXTENT),
    Vec3::new(HALF_EXTENT, HALF_EXTENT, -HALF_EXTENT),
    Vec3::new(-HALF_EXTENT, HALF_EXTENT, -HALF_EXTENT),
    Vec3::new(-HALF_EXTENT, -HALF_EXTENT, HALF_EXTENT),
    Vec3::new(HALF_EXTENT, -HALF_EXTENT, HALF_EXTENT),
    Vec3::new(HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
    Vec3::new(-HALF_EXTENT, HALF_EXTENT, HALF_EXTENT),
];

/// Vertex index pairs for the twelve undirected edges.
const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

/// Wireframe of the stock cube: fixed **π/4** tilt about **Y** then **X** so **xy** projection shows depth.
///
/// `Camera::transform` uses **xy** only (**z** dropped); without this tilt, faces stack in ortho **xy**.
pub fn draw_wireframe(fb: &mut FrameBuffer, camera: &Camera, color: Rgb) {
    let tilt = std::f32::consts::FRAC_PI_4;
    let rot = Mat3::from_rotation_x(tilt) * Mat3::from_rotation_y(tilt);
    let verts: [Vec3; 8] = VERTS.map(|v| rot * v);

    for &(i, j) in &EDGES {
        let a = camera.transform(verts[i]);
        let b = camera.transform(verts[j]);
        fb.draw_line(a, b, color);
    }
}
