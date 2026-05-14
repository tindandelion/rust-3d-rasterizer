//! Integration: render output must match the committed decode of `snapshots/cube/scene.webp`
//! (default **RGBA** frame buffer from **`webp_animation::Decoder`**, opaque alpha).
//!
//! Regenerate the snapshot after intentional visual changes:
//! `cargo run --quiet -- snapshots/cube/scene.webp` (from the crate root).

mod common;

use common::{WebpImage, manifest_path, run_package_binary};

const OUTPUT_FILE_NAME: &str = "cube-golden-test.webp";
const SNAPSHOT_REL_PATH: &str = "snapshots/cube/scene.webp";

#[test]
#[ignore]
fn cube_still_matches_snapshot_webp() {
    let snapshot_file = manifest_path(SNAPSHOT_REL_PATH);
    assert!(
        snapshot_file.is_file(),
        "missing golden file: {} — run `cargo run --quiet -- {}` from the repo root and commit",
        snapshot_file.display(),
        SNAPSHOT_REL_PATH
    );

    let render_result = run_package_binary(OUTPUT_FILE_NAME);

    let expected = WebpImage::read(&snapshot_file);
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
