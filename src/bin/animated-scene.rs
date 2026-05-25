//! **`scene::dodecahedron::unit_dodecahedron`** (same **`[-0.5, 0.5]³`** framing as **`scene::cube::unit_cube`**),
//! diffuse **`CUBE_ALBEDO`**, back-face culled — **two-phase** **`ANIMATED_SCENE_FRAME_COUNT`**-frame clip (**double** the older single‑phase length).
//!
//! 1. **Camera orbit (`… / 2` frames):** **eye** **`(0, 0.3, −1)` → … → `(0, 0.3, −1)`** by **`360°`** around **`+Y`** on **`xz`** radius **`CAMERA_ORBIT_RADIUS`**, **`y = 0.3`** (**`(sin θ, 0.3, −cos θ)`**); scaled dodecahedron **does not tumble** (**`0.75`** uniform scale only).
//! 2. **Model tumble (`… / 2` frames):** **camera** pinned at **`(0, 0.3, −1)`**; **`0.75`** uniform world scale plus **three-axis Euler** tumble (**`R_z R_y R_x`** with a common angle).

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
/// **`20 ms`** ⇒ **50 fps** (`1000 / 20`).
const FRAME_SPACING_MS: i32 = 20;
const CAMERA_ORBIT_RADIUS: f32 = 1.0;
/// **`y`** elevation shared by default **orbit** start/end and **tumble** pin (horizontal circle **`y =`** this).
const CAMERA_EYE_Y: f32 = 0.3;
const CAMERA_DEFAULT_EYE: Vec3 = Vec3::new(0.0, CAMERA_EYE_Y, -CAMERA_ORBIT_RADIUS);

fn half_lap_frames() -> u32 {
    ANIMATED_SCENE_FRAME_COUNT / 2
}

/// **Eye** position on the **`xz`** circle (**`CAMERA_ORBIT_RADIUS`**) framing **`Vec3::ZERO`**, **`+Y`** up.
/// **`angle = 0`** yields **`CAMERA_DEFAULT_EYE`** (**`−Z`** at **`CAMERA_EYE_Y`**); angle increases toward **world +X** (**right‑hand wrap** around **+Y**).
fn camera_eye_orbit(angle: f32) -> Vec3 {
    Vec3::new(
        angle.sin() * CAMERA_ORBIT_RADIUS,
        CAMERA_EYE_Y,
        -angle.cos() * CAMERA_ORBIT_RADIUS,
    )
}

/// Fixed **Euler‑free** scaled mesh for the orbit segment (same **`0.75`** diagonal as tumble phase).
fn model_matrix_scaled_only() -> Mat4 {
    Mat4::from_mat3(Mat3::from_diagonal(Vec3::splat(0.75)))
}

/// **World-fixed** Euler tumble (**`R_z R_y R_x`**) with **`0.75`** uniform world scale;
/// **`frame_index`** in **`0‥lap_frames`**; base **`unit_dodecahedron()`** verts span **`[-0.5, 0.5]³`** axis box.
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
    let half = half_lap_frames();
    assert_eq!(
        ANIMATED_SCENE_FRAME_COUNT,
        half * 2,
        "ANIMATED_SCENE_FRAME_COUNT must be even"
    );

    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear_black();

        let (camera_pos, mesh) = if frame_index < half {
            let n = half.max(1) as f32;
            let angle = (frame_index as f32 / n) * std::f32::consts::TAU;
            let eye = camera_eye_orbit(angle);
            let mesh = shape.transform(model_matrix_scaled_only());
            (eye, mesh)
        } else {
            let local_frame = frame_index - half;
            (
                CAMERA_DEFAULT_EYE,
                shape.transform(model_matrix_euler_sweep(local_frame, half)),
            )
        };

        let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(camera_pos);
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
