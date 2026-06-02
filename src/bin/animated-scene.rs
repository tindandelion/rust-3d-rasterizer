//! **`shapes::sphere(4)`** (unit-radius octasphere seed, four subdivision passes),
//! **Gouraud** **`PhongLightModel`** (Blinn–Phong) on radial vertex normals, **`SHAPE_BASE_COLOR`**, back-face culled — **two-phase** **`ANIMATED_SCENE_FRAME_COUNT`**-frame clip.
//!
//! 1. **Camera orbit (`… / 2` frames):** **eye** **`(0, 0.2, −1)` → … → `(0, 0.2, −1)`** by **`360°`** around **`+Y`** on **`xz`** radius **`CAMERA_ORBIT_RADIUS`**, **`y = 0.2`** (**`(sin θ, 0.2, −cos θ)`**); **cubic ease‑in‑out** on angle per lap (slow ends, quicker middle); mesh **does not squash** (**`0.75`** uniform scale only).
//! 2. **Y squash (`… / 2` frames):** **camera** pinned at **`(0, 0.2, −1)`**; **`0.75`** on **`x`/`z`**, **`y`** eased **`0.75 → 0.4 → 0.75`** (same cubic pacing as orbit).

use std::path::Path;

use glam::{Mat3, Mat4, Vec3};

use thorus_forge::Material;
use thorus_forge::shapes::sphere;
use thorus_forge::{
    ANIMATED_SCENE_FRAME_COUNT, ANIMATED_SCENE_FRAME_SPACING_MS, Camera, FrameBuffer,
    PhongLightModel, SCENE_HEIGHT, SCENE_WIDTH, WebpEncoder, draw_facets,
    output_webp_path_from_args,
};

const CAMERA_ORBIT_RADIUS: f32 = 1.0;
/// **`y`** elevation shared by default **orbit** start/end and **squash** pin (horizontal circle **`y =`** this).
const CAMERA_EYE_Y: f32 = 0.2;
const CAMERA_DEFAULT_EYE: Vec3 = Vec3::new(0.0, CAMERA_EYE_Y, -CAMERA_ORBIT_RADIUS);
/// Uniform **x**/**z** world scale and the **y** scale at loop endpoints (**squash** phase animates **`y`** down to [`Y_SCALE_MIN`]).
const MESH_SCALE_XZ: f32 = 0.75;
const Y_SCALE_MIN: f32 = 0.4;

fn half_lap_frames() -> u32 {
    ANIMATED_SCENE_FRAME_COUNT / 2
}

/// **Ease‑in‑out (cubic):** slow at the ends, faster in the middle — applied separately to **each** animation half (**orbit**, **squash**).
///
/// **`u`** is linear progress in **`[0, 1]`** (**`frame_index / lap_frames`** in this bin); **`0 → 1`** and **`d/du`** zero at **`u ∈ {0, 1}`** so motion eases without overshoot.
fn ease_in_out_cubic(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    if u < 0.5 {
        4.0 * u * u * u
    } else {
        let v = -2.0 * u + 2.0;
        1.0 - v * v * v / 2.0
    }
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

/// Fixed uniform scaled mesh for the orbit segment (same **`x`/`z`** scale as squash phase).
fn model_matrix_scaled_only() -> Mat4 {
    Mat4::from_mat3(Mat3::from_diagonal(Vec3::splat(MESH_SCALE_XZ)))
}

/// **Non-uniform Y squash:** **`x`/`z`** stay at [`MESH_SCALE_XZ`]; **`y`** eases **`MESH_SCALE_XZ → Y_SCALE_MIN → MESH_SCALE_XZ`**
/// over **`frame_index`** in **`0‥lap_frames`**.
fn model_matrix_y_scale_sweep(frame_index: u32, lap_frames: u32) -> Mat4 {
    let n = lap_frames.max(1) as f32;
    let u = frame_index as f32 / n;
    let t = ease_in_out_cubic(u);
    let blend = (t * std::f32::consts::PI).sin();
    let y_scale = MESH_SCALE_XZ + (Y_SCALE_MIN - MESH_SCALE_XZ) * blend;
    Mat4::from_mat3(Mat3::from_diagonal(Vec3::new(
        MESH_SCALE_XZ,
        y_scale,
        MESH_SCALE_XZ,
    )))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut encoder = WebpEncoder::with_frame_spacing(
        SCENE_WIDTH,
        SCENE_HEIGHT,
        ANIMATED_SCENE_FRAME_SPACING_MS,
    )?;
    let light = PhongLightModel::new(
        glam::Vec3::new(1.0, 0.5, -1.0).into(),
        Material::shiny(0.15, 100.0),
    );

    let shape = sphere(4);
    let half = half_lap_frames();
    assert_eq!(
        ANIMATED_SCENE_FRAME_COUNT,
        half * 2,
        "ANIMATED_SCENE_FRAME_COUNT must be even"
    );

    let frame_production_start = std::time::Instant::now();
    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear_black();

        let (camera_pos, mesh) = if frame_index < half {
            let n = half.max(1) as f32;
            let u = frame_index as f32 / n;
            let angle = ease_in_out_cubic(u) * std::f32::consts::TAU;
            let eye = camera_eye_orbit(angle);
            let mesh = shape.transform(model_matrix_scaled_only());
            (eye, mesh)
        } else {
            let local_frame = frame_index - half;
            (
                CAMERA_DEFAULT_EYE,
                shape.transform(model_matrix_y_scale_sweep(local_frame, half)),
            )
        };

        let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(camera_pos);
        draw_facets(&mut framebuffer, &camera, &mesh, &light);

        encoder.add_frame(&framebuffer)?;
    }
    let frame_production_elapsed = frame_production_start.elapsed();
    let frame_production_secs = frame_production_elapsed.as_secs_f64().max(1e-12);
    let frame_production_fps = ANIMATED_SCENE_FRAME_COUNT as f64 / frame_production_secs;

    encoder.write(Path::new(&out_path))?;

    println!(
        "Wrote {} ({ANIMATED_SCENE_FRAME_COUNT} frames, {}×{}, lossless)",
        out_path.to_string_lossy(),
        SCENE_WIDTH,
        SCENE_HEIGHT,
    );
    println!(
        "Frame production: {:.2} fps ({ANIMATED_SCENE_FRAME_COUNT} frames in {:?})",
        frame_production_fps, frame_production_elapsed,
    );

    Ok(())
}
