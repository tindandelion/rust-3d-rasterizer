//! Lossless **animated** WebP: orthographic **wireframe** cube (edge length **0.5** in world space)
//! rotating every frame — template copied from **`still-cube`** (duplication tolerated for now).

use std::env;
use std::ffi::OsString;
use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::cube::{Cube, Edge};
use thorus_forge::{
    ANIMATED_CUBE_FRAME_COUNT, Camera, DEFAULT_OUT_PATH, FrameBuffer, Rgb, SCENE_HEIGHT,
    SCENE_WIDTH, WebpEncoder,
};

/// Timestamp step between successive frames (**ms**); last frame duration uses the same spacing at **`finalize`**.
///
/// **`20 ms`** ⇒ **50 fps** (`1000 / 20`). With **`360`** frames, one full **`R_y`** lap is **\(360^\circ\)** at **\(1^\circ\)** per sample.
const FRAME_SPACING_MS: i32 = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_file_name();

    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mut encoder = WebpEncoder::with_frame_spacing(SCENE_WIDTH, SCENE_HEIGHT, FRAME_SPACING_MS)?;

    for frame_index in 0..ANIMATED_CUBE_FRAME_COUNT {
        framebuffer.clear_black();

        let mut mesh = Cube::new();
        mesh.set_transform(cube_transform_frame(frame_index));
        draw_wireframe(&mut framebuffer, &camera, &mesh, Rgb::WHITE);

        encoder.add_frame(&framebuffer)?;
    }

    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({ANIMATED_CUBE_FRAME_COUNT} frames, {}×{}, lossless)",
        out_path.to_string_lossy(),
        SCENE_WIDTH,
        SCENE_HEIGHT,
    );

    Ok(())
}

/// Project each edge through **`camera`** and rasterize with the framebuffer's **DDA** line routine.
fn draw_wireframe(fb: &mut FrameBuffer, camera: &Camera, cube: &Cube, color: Rgb) {
    for Edge(a, b) in cube.edges() {
        fb.draw_line(camera.transform(a), camera.transform(b), color);
    }
}

/// Static **π/4** tilt (same readability trick as **`still-cube`**), plus one full **\(2π\)** spin about **world +Y**.
///
/// **Composition (column vectors):** **`R_y(t) · R_tilt · S`** with **`t = 2π · frame_index / N`**
/// (**`frame_index ∈ [0, N)`**). **`R_tilt = R_x(π/4) · R_y(π/4)`** matches the still frame’s fixed pose when **`t = 0`**.
fn cube_transform_frame(frame_index: u32) -> Mat4 {
    let tilt = std::f32::consts::FRAC_PI_4;
    let base = Mat3::from_rotation_x(tilt) * Mat3::from_rotation_y(tilt);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));

    let n = ANIMATED_CUBE_FRAME_COUNT.max(1) as f32;
    let t = (frame_index as f32 / n) * std::f32::consts::TAU;

    let spin_y = Mat3::from_rotation_y(t);

    Mat4::from_mat3(spin_y * base * scale)
}

fn output_file_name() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}
