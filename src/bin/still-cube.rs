//! Lossless WebP still: wireframe **cube** with edge length **0.5** in world space.

use std::env;
use std::ffi::OsString;
use std::path::Path;

use glam::{Mat3, Vec3};

use thorus_forge::{Camera, FrameBuffer, Rgb, WebpEncoder};

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;

/// Half of the cube edge length (`0.5 / 2`) in world coordinates.
const CUBE_HALF_EXTENT: f32 = 0.25;

const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Axis-aligned cube, centered at the origin, edge length `2 * CUBE_HALF_EXTENT`.
const CUBE_VERTS: [Vec3; 8] = [
    Vec3::new(-CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT),
    Vec3::new(CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT),
    Vec3::new(CUBE_HALF_EXTENT, CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT),
    Vec3::new(-CUBE_HALF_EXTENT, CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT),
    Vec3::new(-CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT, CUBE_HALF_EXTENT),
    Vec3::new(CUBE_HALF_EXTENT, -CUBE_HALF_EXTENT, CUBE_HALF_EXTENT),
    Vec3::new(CUBE_HALF_EXTENT, CUBE_HALF_EXTENT, CUBE_HALF_EXTENT),
    Vec3::new(-CUBE_HALF_EXTENT, CUBE_HALF_EXTENT, CUBE_HALF_EXTENT),
];

/// Vertex index pairs for the twelve undirected edges.
const CUBE_EDGES: [(usize, usize); 12] = [
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    draw_cube_wireframe(&mut framebuffer, &camera, Rgb::WHITE);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}

fn draw_cube_wireframe(fb: &mut FrameBuffer, camera: &Camera, color: Rgb) {
    // `Camera::transform` uses **xy** only (`z` dropped). Tilt **π/4** about **Y** then **X** so depth shows up on
    // screen instead of faces stacking in projection.
    let tilt = std::f32::consts::FRAC_PI_4;
    let rot = Mat3::from_rotation_x(tilt) * Mat3::from_rotation_y(tilt);
    let verts: [Vec3; 8] = CUBE_VERTS.map(|v| rot * v);

    for &(i, j) in &CUBE_EDGES {
        let a = camera.transform(verts[i]);
        let b = camera.transform(verts[j]);
        fb.draw_line(a, b, color);
    }
}

fn output_file_name() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}
