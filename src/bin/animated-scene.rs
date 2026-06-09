//! Two **`meshes::sphere(4)`** instances at **`(±0.5, 0, 0)`** (**left radius 0.4**, **right 0.3**), distinct colors,
//! **Phong** **`BlinnLightModel`**, back-face culled — **`ANIMATED_SCENE_FRAME_COUNT`**-frame lossless WebP.
//!
//! **Camera orbit:** **eye** on **`xz`** radius **`CAMERA_ORBIT_RADIUS`**, **`y = CAMERA_EYE_Y`**, one eased
//! **`360°`** lap around **`Vec3::ZERO`** over the full clip (**`ease_in_out_cubic`** on frame index).

use std::env;
use std::ffi::OsString;
use std::path::Path;

use glam::{Mat4, Quat, Vec3};

use thorus_forge::Material;
use thorus_forge::geometry::Mesh;
use thorus_forge::meshes::sphere;
use thorus_forge::{
    ANIMATED_SCENE_FRAME_COUNT, ANIMATED_SCENE_FRAME_SPACING_MS, BlinnLightModel, Camera,
    FrameBuffer, Rgb, SCENE_HEIGHT, SCENE_WIDTH, Shape, WebpEncoder,
};

const CAMERA_ORBIT_RADIUS: f32 = 1.0;
const CAMERA_EYE_Y: f32 = 0.2;

const LEFT_SPHERE_RADIUS: f32 = 0.4;
const RIGHT_SPHERE_RADIUS: f32 = 0.3;
const SPHERE_SPLITS: usize = 4;

const LEFT_SPHERE_CENTER: Vec3 = Vec3::new(-0.5, 0.0, 0.0);
const RIGHT_SPHERE_CENTER: Vec3 = Vec3::new(0.5, 0.0, 0.0);

const LEFT_SPHERE_COLOR: Rgb = Rgb(52, 110, 210);
const RIGHT_SPHERE_COLOR: Rgb = Rgb(210, 90, 52);

const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Output **`.webp`** path: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

/// **Ease‑in‑out (cubic):** slow at the ends, faster in the middle.
///
/// **`u`** is linear progress in **`[0, 1]`**; **`0 → 1`** and **`d/du`** zero at **`u ∈ {0, 1}`**.
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
/// **`angle = 0`** yields **`(0, CAMERA_EYE_Y, −CAMERA_ORBIT_RADIUS)`**; angle increases toward **world +X**.
fn camera_eye_orbit(angle: f32) -> Vec3 {
    Vec3::new(
        angle.sin() * CAMERA_ORBIT_RADIUS,
        CAMERA_EYE_Y,
        -angle.cos() * CAMERA_ORBIT_RADIUS,
    )
}

fn sphere_at(center: Vec3, radius: f32) -> Mesh {
    let pose = Mat4::from_scale_rotation_translation(Vec3::splat(radius), Quat::IDENTITY, center);
    sphere(SPHERE_SPLITS).transform(pose)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut encoder = WebpEncoder::with_frame_spacing(
        SCENE_WIDTH,
        SCENE_HEIGHT,
        ANIMATED_SCENE_FRAME_SPACING_MS,
    )?;
    let light = BlinnLightModel::new(
        glam::Vec3::new(1.0, 0.5, -1.0).into(),
        Material::shiny(0.15, 100.0),
    );

    let shapes = [
        Shape::new(
            sphere_at(LEFT_SPHERE_CENTER, LEFT_SPHERE_RADIUS),
            LEFT_SPHERE_COLOR,
        ),
        Shape::new(
            sphere_at(RIGHT_SPHERE_CENTER, RIGHT_SPHERE_RADIUS),
            RIGHT_SPHERE_COLOR,
        ),
    ];

    let frame_production_start = std::time::Instant::now();
    let lap_frames = ANIMATED_SCENE_FRAME_COUNT.max(1) as f32;
    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear();

        let u = frame_index as f32 / lap_frames;
        let angle = ease_in_out_cubic(u) * std::f32::consts::TAU;
        let camera =
            Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(camera_eye_orbit(angle));

        for shape in &shapes {
            shape.render(&mut framebuffer, &camera, &light);
        }

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
