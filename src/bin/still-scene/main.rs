mod kitty_terminal;
use std::path::Path;
use std::thread;

use glam::Vec3;

use thorus_forge::BlinnLightModel;
use thorus_forge::Material;
use thorus_forge::Rgb;
use thorus_forge::meshes::torus;
use thorus_forge::{Camera, FrameBuffer, Shape, WebpEncoder};

use crate::kitty_terminal::KittyTerminal;

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);
const LIGHT_DIRECTION: Vec3 = Vec3::new(-10.0, 10.0, -10.0);
const OUT_PATH: &str = "still-scene.webp";
const TORUS_COLOR: Rgb = Rgb(52, 110, 210);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = KittyTerminal::new();
    let (width, height) = terminal.pixel_dimensions()?;
    let mut framebuffer = FrameBuffer::new(width, height);
    let camera = Camera::for_viewport(width, height).move_to(CAMERA_POS);
    let light = BlinnLightModel::new(LIGHT_DIRECTION.into(), Material::shiny(0.15, 100));

    let torus = Shape::new(torus(48, 32), TORUS_COLOR);
    println!("Rendering torus...");
    torus.render(&mut framebuffer, &camera, &light);

    let fb_clone = framebuffer.clone();
    let handle = thread::spawn(move || save_webp(&fb_clone).unwrap());
    display_in_terminal(&mut terminal, &framebuffer)?;
    handle.join().unwrap();

    Ok(())
}

fn save_webp(framebuffer: &FrameBuffer) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = WebpEncoder::new(framebuffer.width(), framebuffer.height())?;
    encoder.add_frame(&framebuffer)?;
    encoder.write(Path::new(OUT_PATH))?;
    println!(
        "Wrote {} ({}×{}, lossless)",
        OUT_PATH,
        framebuffer.width(),
        framebuffer.height()
    );
    Ok(())
}

fn display_in_terminal(
    terminal: &mut KittyTerminal,
    framebuffer: &FrameBuffer,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.enter()?;
    terminal.display_rgb(&framebuffer, framebuffer.width(), framebuffer.height())?;
    terminal.wait_for_key()?;
    terminal.leave()?;
    Ok(())
}
