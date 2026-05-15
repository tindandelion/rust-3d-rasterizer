//! Lossless WebP still: wireframe **cube** with edge length **0.5** in world space.

use std::env;
use std::ffi::OsString;
use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::cube::{Cube, Edge};
use thorus_forge::{Camera, FrameBuffer, Rgb, WebpEncoder};

const SCENE_WIDTH: u32 = 800;
const SCENE_HEIGHT: u32 = 600;

const DEFAULT_OUT_PATH: &str = "scene.webp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mut mesh = Cube::new();
    mesh.set_transform(cube_transform());
    draw_wireframe(&mut framebuffer, &camera, &mesh, Rgb::WHITE);

    let mut encoder = WebpEncoder::new(SCENE_WIDTH, SCENE_HEIGHT)?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({SCENE_WIDTH}×{SCENE_HEIGHT}, lossless)",
        out_path.to_string_lossy()
    );

    Ok(())
}

/// Project each edge through **`camera`** and rasterize with the framebuffer's **DDA** line routine.
fn draw_wireframe(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    for Edge(a, b) in cube.edges() {
        fb.draw_line(camera.transform(a), camera.transform(b), color);
    }
}

/// Uniform scale to world-space edge **0.5** on the **`Cube`** model, then **π/4** about **Y** and **X**.
///
/// Applied as **R · S** to column positions (`0.5` matches the orthographic cube snapshot).
/// `Camera::transform` uses **xy** only (**z** dropped); this tilt keeps orthographic **xy** from stacking faces.
fn cube_transform() -> Mat4 {
    let tilt = std::f32::consts::FRAC_PI_4;
    let rot = Mat3::from_rotation_x(tilt) * Mat3::from_rotation_y(tilt);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    Mat4::from_mat3(rot * scale)
}

fn output_file_name() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}
