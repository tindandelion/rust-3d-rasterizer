//! Lossless **animated** WebP: orthographic **wireframe** cube (edge length **0.5** in world space),
//! **three-axis Euler** tumble (**`R_z R_y R_x`** with a common angle) sampled over **`ANIMATED_CUBE_FRAME_COUNT`** frames.

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::cube::Cube;
use thorus_forge::wireframe::draw_edges;
use thorus_forge::{
    ANIMATED_CUBE_FRAME_COUNT, Camera, FrameBuffer, Rgb, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder,
    output_webp_path_from_args,
};

/// Wireframe stroke: **cornflower blue** (reads clearly on black).
const WIRE_RGB: Rgb = Rgb(100, 149, 237);

/// Timestamp step between successive frames (**ms**); last frame duration uses the same spacing at **`finalize`**.
///
/// **`20 ms`** ⇒ **50 fps** (`1000 / 20`). With **`360`** frames, one full tumble lap samples **`t`** from **0** to **τ** (exclusive of **τ** on the last sample step).
const FRAME_SPACING_MS: i32 = 20;

/// **World-fixed** Euler tumble: **`R_z R_y R_x`** with **`α = β = γ = t`**, on a **0.5** uniform-scale unit cube.
fn model_matrix_euler_sweep(frame_index: u32, lap_frames: u32) -> Mat4 {
    let n = lap_frames.max(1) as f32;
    let t = (frame_index as f32 / n) * std::f32::consts::TAU;
    let spin = Mat4::from_rotation_z(t) * Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t);
    let scale = Mat3::from_diagonal(Vec3::splat(0.5));
    spin * Mat4::from_mat3(scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let camera = Camera::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);

    let mut encoder = WebpEncoder::with_frame_spacing(SCENE_WIDTH, SCENE_HEIGHT, FRAME_SPACING_MS)?;

    for frame_index in 0..ANIMATED_CUBE_FRAME_COUNT {
        framebuffer.clear_black();

        let mut mesh = Cube::new();
        mesh.set_transform(model_matrix_euler_sweep(
            frame_index,
            ANIMATED_CUBE_FRAME_COUNT,
        ));
        draw_edges(&mut framebuffer, &camera, &mesh, WIRE_RGB);

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
