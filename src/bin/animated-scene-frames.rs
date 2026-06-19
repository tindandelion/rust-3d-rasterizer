//! **`meshes::torus(48, 32)`** at the origin, **Phong** multi-light, back-face culled —
//! **`ANIMATED_SCENE_FRAME_COUNT`** PNG frames.
//!
//! **Fixed camera** (same eye as **`still-scene`**). **Model:** world-fixed **`R_z R_y R_x`** tumble with
//! **`α = β = γ = t`**, **`t`** sweeping **`0 … τ`** over the clip (**seamless loop**).

use std::env;
use std::f32::consts::TAU;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use glam::{Mat4, Vec3};
use png::{BitDepth, ColorType, Encoder};

use thorus_forge::meshes::torus;
use thorus_forge::{
    ANIMATED_SCENE_FRAME_COUNT, Camera, FrameBuffer, SCENE_BACKGROUND, SCENE_HEIGHT, SCENE_WIDTH,
    Shape, default_lights, default_material,
};

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);

const TORUS_RING_SEGMENTS: usize = 48;
const TORUS_TUBE_SEGMENTS: usize = 32;
const TORUS_SCALE: f32 = 0.8;

const DEFAULT_OUT_DIR: &str = "target/animated-scene";

struct PngFrameWriter {
    out_dir: PathBuf,
    width: u32,
    height: u32,
}

impl PngFrameWriter {
    fn new(
        out_dir: impl Into<PathBuf>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            out_dir: out_dir.into(),
            width,
            height,
        })
    }

    fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.out_dir.exists() {
            fs::remove_dir_all(&self.out_dir)?;
        }
        fs::create_dir_all(&self.out_dir)?;
        Ok(())
    }

    fn write_frame(&self, frame_index: u32, rgb: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.out_dir.join(format!("{frame_index:06}.png"));
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = Encoder::new(writer, self.width, self.height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgb)?;
        Ok(())
    }
}

/// Output directory: first **argv** argument if set, else [`DEFAULT_OUT_DIR`].
fn output_dir_from_args() -> PathBuf {
    env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| DEFAULT_OUT_DIR.into())
}

/// Uniform scale plus world-fixed **`R_z R_y R_x`** at angle **`t`** (radians).
fn model_matrix_tumble(t: f32) -> Mat4 {
    let rotation = Mat4::from_rotation_z(t) * Mat4::from_rotation_y(t) * Mat4::from_rotation_x(t);
    Mat4::from_scale(Vec3::splat(TORUS_SCALE)) * rotation
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let writer = PngFrameWriter::new(output_dir_from_args(), SCENE_WIDTH, SCENE_HEIGHT)?;
    writer.clear()?;

    let mut framebuffer = FrameBuffer::new(SCENE_WIDTH, SCENE_HEIGHT);
    let camera = Camera::for_viewport(SCENE_WIDTH, SCENE_HEIGHT).move_to(CAMERA_POS);
    let lights = default_lights();

    let base_mesh = torus(TORUS_RING_SEGMENTS, TORUS_TUBE_SEGMENTS);

    println!(
        "Mesh: {} vertices, {} facets",
        base_mesh.vertices().len(),
        base_mesh.facets().len(),
    );

    let frame_production_start = std::time::Instant::now();
    let lap_frames = ANIMATED_SCENE_FRAME_COUNT.max(1) as f32;
    for frame_index in 0..ANIMATED_SCENE_FRAME_COUNT {
        framebuffer.clear(SCENE_BACKGROUND);

        let t = frame_index as f32 / lap_frames * TAU;
        let torus = Shape::new(
            base_mesh.transform(model_matrix_tumble(t)),
            default_material(),
        );
        torus.render(&mut framebuffer, &camera, &lights);

        writer.write_frame(frame_index, framebuffer.as_ref())?;
    }
    let frame_production_elapsed = frame_production_start.elapsed();
    let frame_production_secs = frame_production_elapsed.as_secs_f64().max(1e-12);
    let frame_production_fps = ANIMATED_SCENE_FRAME_COUNT as f64 / frame_production_secs;

    println!(
        "Wrote {ANIMATED_SCENE_FRAME_COUNT} PNG frames to {} ({}×{})",
        writer.out_dir.display(),
        writer.width,
        writer.height,
    );
    println!(
        "Frame production: {:.2} fps ({ANIMATED_SCENE_FRAME_COUNT} frames in {:?})",
        frame_production_fps, frame_production_elapsed,
    );

    Ok(())
}
