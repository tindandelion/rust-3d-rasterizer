mod kitty_terminal;
use std::path::Path;
use std::thread;

use glam::Vec3;

use thorus_forge::DirectionalLight;
use thorus_forge::Material;
use thorus_forge::Rgb;
use thorus_forge::meshes::torus;
use thorus_forge::{Camera, FrameBuffer, Shape, WebpEncoder};

use crate::kitty_terminal::KittyTerminal;

const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.5, -1.0);
const LIGHT_DIRECTION: Vec3 = Vec3::new(1.0, 0.5, -1.0);
const OUT_PATH: &str = "still-scene.webp";
// Geometry-browser MeshPhongMaterial: color 0x156289, emissive 0x072534,
// specular 0x111111, shininess 30.
const TORUS_MATERIAL: Material =
    Material::new(Rgb(7, 37, 52), Rgb(21, 98, 137), Rgb(17, 17, 17), Some(30));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = KittyTerminal::new();
    let (width, height) = terminal.pixel_dimensions()?;
    let framebuffer = render_scene(width, height);

    let fb_clone = framebuffer.clone();
    let handle = thread::spawn(move || save_webp(&fb_clone).unwrap());
    display_in_terminal(&mut terminal, &framebuffer)?;
    handle.join().unwrap();

    Ok(())
}

fn render_scene(width: u32, height: u32) -> FrameBuffer {
    let mut framebuffer = FrameBuffer::new(width, height);
    let camera = Camera::for_viewport(width, height).move_to(CAMERA_POS);
    let light = DirectionalLight::new(LIGHT_DIRECTION.into());

    let torus = Shape::new(torus(48, 32), TORUS_MATERIAL);
    torus.render(&mut framebuffer, &camera, &light);

    framebuffer
}

fn save_webp(framebuffer: &FrameBuffer) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = WebpEncoder::new(framebuffer.width(), framebuffer.height())?;
    encoder.add_frame(framebuffer)?;
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
    terminal.display_rgb(framebuffer, framebuffer.width(), framebuffer.height())?;
    terminal.wait_for_key()?;
    terminal.leave()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::render_scene;
    use webp_animation::{ColorMode, Decoder};

    const EXPECTED_WEBP: &str = "test-data/still-scene.webp";

    #[test]
    fn test_render_scene() {
        let expected_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_WEBP);
        let (width, height, expected_rgb) = decode_webp_rgb(&expected_path);

        let actual = render_scene(width, height);
        let actual_rgb = actual.as_ref();
        let l2_distance = rgb_l2_distance(actual_rgb, &expected_rgb);
        assert!(
            actual_rgb == expected_rgb.as_slice(),
            "rendered scene does not match {} (L2 distance: {l2_distance})",
            expected_path.display()
        );
    }

    /// Regenerate **`test-data/still-scene.webp`** after intentional render changes:
    /// `cargo test --bin still-scene refresh_still_scene_golden_webp -- --ignored`
    #[test]
    #[ignore = "manual golden refresh"]
    fn refresh_still_scene_golden_webp() {
        use thorus_forge::WebpEncoder;

        let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_WEBP);
        let (width, height, _) = decode_webp_rgb(&golden_path);
        let framebuffer = render_scene(width, height);
        let mut encoder = WebpEncoder::new(width, height).expect("webp encoder");
        encoder
            .add_frame(&framebuffer)
            .expect("encode still-scene frame");
        encoder.write(&golden_path).expect("write golden webp");
    }

    fn rgb_l2_distance(actual: &[u8], expected: &[u8]) -> f64 {
        assert_eq!(actual.len(), expected.len(), "RGB buffers differ in length");
        actual
            .iter()
            .zip(expected)
            .map(|(a, e)| {
                let delta = i32::from(*a) - i32::from(*e);
                f64::from(delta * delta)
            })
            .sum::<f64>()
            .sqrt()
    }

    fn decode_webp_rgb(path: &Path) -> (u32, u32, Vec<u8>) {
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("read webp {}: {e}", path.display()));
        let decoder = Decoder::new(&bytes).expect("decode webp");
        let dims = decoder.dimensions();
        let mut frames = decoder.into_iter();
        let frame = frames.next().expect("at least one frame in still");
        assert!(
            frames.next().is_none(),
            "expected a single-frame still, got multiple frames"
        );
        assert_eq!(frame.dimensions(), dims);
        assert_eq!(frame.color_mode(), ColorMode::Rgba);

        let rgba = frame.data();
        let rgb: Vec<u8> = rgba
            .chunks_exact(4)
            .flat_map(|pixel| pixel[..3].iter().copied())
            .collect();

        (dims.0, dims.1, rgb)
    }
}
