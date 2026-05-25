//! **`scene::dodecahedron::unit_dodecahedron`** (same **`[-0.5, 0.5]³`** framing as **`scene::cube::unit_cube`**), diffuse **`CUBE_ALBEDO`**, back-face culled.
//!
//! **`0.5`** uniform scale in world space, **three-axis Euler** tumble (**`R_z R_y R_x`** with a common angle) over **`ANIMATED_SCENE_FRAME_COUNT`** frames.

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::scene::dodecahedron::unit_dodecahedron;
use thorus_forge::{
    ANIMATED_SCENE_FRAME_COUNT, Camera, FrameBuffer, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder,
    output_webp_path_from_args,
};
use thorus_forge::{DiffuseLight, draw_faces};

/// Timestamp step between successive frames (**ms**); last frame duration uses the same spacing at **`finalize`**.
///
/// **`20 ms`** ⇒ **50 fps** (`1000 / 20`). With **`360`** frames, one full tumble lap samples **`t`** from **0** to **τ** (exclusive of **τ** on the last sample step).
const FRAME_SPACING_MS: i32 = 20;
const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, -1.0);

/// **World-fixed** Euler tumble (**`R_z R_y R_x`**) with **`0.75`** uniform world scale; base **`unit_dodecahedron()`** verts already span **`[-0.5, 0.5]³`** axis box.
fn model_matrix_euler_sweep(frame_index: u32, lap_frames: u32) -> Mat4 {
    let n = lap_frames.max(1) as f32;
    let t = (frame_index as f32 / n) * std::f32::consts::TAU;
    let spin = Mat4::from_rotation_z(t) * Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t);
    let scale = Mat3::from_diagonal(Vec3::splat(0.75));
    spin * Mat4::from_mat3(scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut encoder = WebpEncoder::with_frame_spacing(SCENE_WIDTH, SCENE_HEIGHT, FRAME_SPACING_MS)?;
    let light = DiffuseLight::new(glam::Vec3::new(1.0, 1.0, -1.0).into(), 0.25);

    let shape = unit_dodecahedron();
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);

    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear_black();

        let mesh = shape.transform(model_matrix_euler_sweep(
            frame_index,
            ANIMATED_SCENE_FRAME_COUNT,
        ));
        draw_faces(&mut framebuffer, &camera, &mesh, &light);

        encoder.add_frame(&framebuffer)?;
    }

    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({ANIMATED_SCENE_FRAME_COUNT} frames, {}×{}, lossless)",
        out_path.to_string_lossy(),
        SCENE_WIDTH,
        SCENE_HEIGHT,
    );

    Ok(())
}
