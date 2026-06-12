//! **`meshes::torus(48, 32)`** at the origin, **Phong** **`BlinnLightModel`**, back-face culled —
//! **`ANIMATED_SCENE_FRAME_COUNT`**-frame lossless WebP.
//!
//! **Fixed camera** (same eye as **`still-scene`**). **Model:** world-fixed **`R_z R_y R_x`** tumble with
//! **`α = β = γ = t`**, **`t`** sweeping **`0 … τ`** over the clip (**seamless loop**).

use std::f32::consts::TAU;

use glam::{Mat4, Vec3};

use thorus_forge::Material;
use thorus_forge::geometry::Mesh;
use thorus_forge::meshes::torus;
use thorus_forge::{ANIMATED_SCENE_FRAME_COUNT, BlinnLightModel, Camera, FrameBuffer, Rgb, Shape};

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);

const TORUS_RING_SEGMENTS: usize = 24;
const TORUS_TUBE_SEGMENTS: usize = 16;
const TORUS_SCALE: f32 = 0.8;

const TORUS_COLOR: Rgb = Rgb(52, 110, 210);

/// Raster width in pixels (golden stills / integration tests must agree).
pub const SCENE_WIDTH: u32 = 800;
/// Raster height in pixels (golden stills / integration tests must agree).
pub const SCENE_HEIGHT: u32 = 600;

/// Uniform scale plus world-fixed **`R_z R_y R_x`** at angle **`t`** (radians).
fn model_matrix_tumble(t: f32) -> Mat4 {
    let rotation = Mat4::from_rotation_z(t) * Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t);
    Mat4::from_scale(Vec3::splat(TORUS_SCALE)) * rotation
}

pub const MODEL_DETAILS_FACTORS: [usize; 5] = [1, 2, 4, 8, 16];
pub const SCENE_DIMS: [(u32, u32); 5] = [
    (800, 600),
    (1280, 720),
    (1920, 1080),
    (3840, 2160),
    (7680, 4320),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n--- Rendering torus with different model details ---");
    for factor in MODEL_DETAILS_FACTORS {
        let mesh = torus(TORUS_RING_SEGMENTS * factor, TORUS_TUBE_SEGMENTS * factor);
        let fps = run_render(&mesh, SCENE_WIDTH, SCENE_HEIGHT);
        println!(
            "Verts: {}, Facets: {}, FPS: {:.2}",
            mesh.vertices().len(),
            mesh.facets().len(),
            fps
        );
    }

    println!("\n\n--- Rendering different resolutions ---");
    let base_mesh = torus(TORUS_RING_SEGMENTS, TORUS_TUBE_SEGMENTS);
    println!(
        "Mesh: {} vertices, {} facets",
        base_mesh.vertices().len(),
        base_mesh.facets().len(),
    );

    for (width, height) in SCENE_DIMS {
        let fps = run_render(&base_mesh, width, height);
        println!(
            "Dimensions: {}x{} ({} px), FPS :{:.2}",
            width,
            height,
            width * height,
            fps
        );
    }

    Ok(())
}

fn run_render(model: &Mesh, scene_width: u32, scene_height: u32) -> f64 {
    let mut framebuffer = FrameBuffer::new(scene_width, scene_height);
    let camera = Camera::for_viewport(scene_width, scene_height).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(
        glam::Vec3::new(1.0, 0.5, -1.0).into(),
        Material::shiny(0.15, 100.0),
    );

    let frame_production_start = std::time::Instant::now();
    let lap_frames = ANIMATED_SCENE_FRAME_COUNT.max(1) as f32;
    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear();

        let t = frame_index as f32 / lap_frames * TAU;
        let torus = Shape::new(model.transform(model_matrix_tumble(t)), TORUS_COLOR);
        torus.render(&mut framebuffer, &camera, &light);
    }
    let frame_production_elapsed = frame_production_start.elapsed();
    let frame_production_secs = frame_production_elapsed.as_secs_f64().max(1e-12);
    let frame_production_fps = ANIMATED_SCENE_FRAME_COUNT as f64 / frame_production_secs;

    return frame_production_fps;
}
