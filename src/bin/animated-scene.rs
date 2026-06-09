//! **`meshes::torus(48, 32)`** at the origin, **Phong** **`BlinnLightModel`**, back-face culled —
//! **`ANIMATED_SCENE_FRAME_COUNT`**-frame lossless WebP.
//!
//! **Fixed camera** (same eye as **`still-scene`**). **Model:** world-fixed **`R_z R_y R_x`** tumble with
//! **`α = β = γ = t`**, **`t`** sweeping **`0 … τ`** over the clip (**seamless loop**).

use std::env;
use std::ffi::OsString;
use std::f32::consts::TAU;
use std::path::Path;

use glam::{Mat4, Vec3};

use thorus_forge::Material;
use thorus_forge::meshes::torus;
use thorus_forge::{
    ANIMATED_SCENE_FRAME_COUNT, ANIMATED_SCENE_FRAME_SPACING_MS, BlinnLightModel, Camera,
    FrameBuffer, Rgb, SCENE_HEIGHT, SCENE_WIDTH, Shape, WebpEncoder,
};

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);

const TORUS_RING_SEGMENTS: usize = 48;
const TORUS_TUBE_SEGMENTS: usize = 32;
const TORUS_SCALE: f32 = 0.8;

const TORUS_COLOR: Rgb = Rgb(52, 110, 210);

const DEFAULT_OUT_PATH: &str = "scene.webp";

/// Output **`.webp`** path: first **argv** argument if set, else [`DEFAULT_OUT_PATH`].
fn output_webp_path_from_args() -> OsString {
    env::args_os()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_OUT_PATH.into())
}

/// Uniform scale plus world-fixed **`R_z R_y R_x`** at angle **`t`** (radians).
fn model_matrix_tumble(t: f32) -> Mat4 {
    let rotation =
        Mat4::from_rotation_z(t) * Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t);
    Mat4::from_scale(Vec3::splat(TORUS_SCALE)) * rotation
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = output_webp_path_from_args();

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut encoder = WebpEncoder::with_frame_spacing(
        SCENE_WIDTH,
        SCENE_HEIGHT,
        ANIMATED_SCENE_FRAME_SPACING_MS,
    )?;
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(
        glam::Vec3::new(1.0, 0.5, -1.0).into(),
        Material::shiny(0.15, 100.0),
    );

    let base_mesh = torus(TORUS_RING_SEGMENTS, TORUS_TUBE_SEGMENTS);

    println!(
        "Mesh: {} vertices, {} facets",
        base_mesh.vertices().len(),
        base_mesh.facets().len(),
    );

    let frame_production_start = std::time::Instant::now();
    let lap_frames = ANIMATED_SCENE_FRAME_COUNT.max(1) as f32;
    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear();

        let t = frame_index as f32 / lap_frames * TAU;
        let torus = Shape::new(base_mesh.transform(model_matrix_tumble(t)), TORUS_COLOR);
        torus.render(&mut framebuffer, &camera, &light);

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
