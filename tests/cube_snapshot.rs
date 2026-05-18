//! Integration: render output must match the committed decode of `snapshots/cube/scene.webp`
//! (default **RGBA** frame buffer from **`webp_animation::Decoder`**, opaque alpha).
//!
//! Regenerate the snapshot after intentional visual changes:
//! `cargo run --quiet --bin still-cube -- snapshots/cube/scene.webp` (from the crate root).

mod common;

use common::{WebpImage, manifest_path, run_integration_binary};
use thorus_forge::{SCENE_HEIGHT, SCENE_WIDTH};

const OUTPUT_FILE_NAME: &str = "cube-golden-test.webp";
const SNAPSHOT_REL_PATH: &str = "snapshots/cube/scene.webp";
/// Hyphenated Cargo bin target (**`cargo run --bin still-cube`**).
const STILL_CUBE_BIN: &str = "still-cube";

#[test]
fn cube_still_matches_snapshot_webp() {
    let snapshot_file = manifest_path(SNAPSHOT_REL_PATH);
    assert!(
        snapshot_file.is_file(),
        "missing golden file: {} — run `cargo run --quiet -- {}` from the repo root and commit",
        snapshot_file.display(),
        SNAPSHOT_REL_PATH
    );

    let expected = WebpImage::read(&snapshot_file);
    assert_eq!(
        expected.dimensions,
        (SCENE_WIDTH, SCENE_HEIGHT),
        "golden snapshot canvas must match crate-root SCENE_WIDTH × SCENE_HEIGHT"
    );

    let render_result = run_integration_binary(STILL_CUBE_BIN, OUTPUT_FILE_NAME);

    let actual = WebpImage::read(render_result.output_path);

    assert_eq!(
        actual.dimensions, expected.dimensions,
        "rendered dimensions must match snapshot canvas"
    );
    assert_eq!(
        actual.rgba.len(),
        expected.rgba.len(),
        "pixel buffer length mismatch"
    );

    compare_webp_images(expected, actual);
}

fn compare_webp_images(expected: WebpImage, actual: WebpImage) {
    if actual.rgba != expected.rgba {
        let mismatch = actual
            .rgba
            .iter()
            .zip(expected.rgba.iter())
            .position(|(a, b)| a != b)
            .expect("buffers unequal but no diff position");
        panic!(
            "decoded frame differs from snapshot at byte index {mismatch} \
             (rendered {}, expected {})",
            actual.rgba[mismatch], expected.rgba[mismatch]
        );
    }
}
