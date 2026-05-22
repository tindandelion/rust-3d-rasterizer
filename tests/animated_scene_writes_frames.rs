//! Runs the **`animated-scene`** binary and checks the animated WebP contains the expected frame count.

mod common;

use std::fs;

use common::run_integration_binary;
use thorus_forge::ANIMATED_SCENE_FRAME_COUNT;
use webp_animation::Decoder;

const BIN_NAME: &str = "animated-scene";
const OUTPUT_FILE_NAME: &str = "test-animated-scene.webp";

#[test]
fn animated_scene_output_has_expected_frame_count() {
    let rendered = run_integration_binary(BIN_NAME, OUTPUT_FILE_NAME);

    let bytes = fs::read(rendered.output_path).expect("read output webp");
    let decoder = Decoder::new(&bytes).expect("decode output as WebP animation");

    let frame_count = decoder.into_iter().count();

    assert_eq!(
        frame_count as u32, ANIMATED_SCENE_FRAME_COUNT,
        "decoder must yield one item per authored frame ({ANIMATED_SCENE_FRAME_COUNT}); got {frame_count}",
    );
}
