//! Runs the **`animated-scene`** binary and checks the animated WebP spans the full authored
//! timeline in milliseconds (see [`thorus_forge::ANIMATED_SCENE_FRAME_SPACING_MS`]).
//!
//! `libwebp` may store fewer physical frames when consecutive raster outputs are identical, but
//! cumulative frame durations must still match **`ANIMATED_SCENE_FRAME_COUNT ×` spacing**.

mod common;

use std::fs;

use common::run_integration_binary;
use thorus_forge::{ANIMATED_SCENE_FRAME_COUNT, ANIMATED_SCENE_FRAME_SPACING_MS};
use webp_animation::Decoder;

const BIN_NAME: &str = "animated-scene";
const OUTPUT_FILE_NAME: &str = "test-animated-scene.webp";

#[test]
#[ignore]
fn animated_scene_output_has_expected_clip_duration_ms() {
    let rendered = run_integration_binary(BIN_NAME, OUTPUT_FILE_NAME);

    let bytes = fs::read(rendered.output_path).expect("read output webp");
    let decoder = Decoder::new(&bytes).expect("decode output as WebP animation");

    let expected_ms = ANIMATED_SCENE_FRAME_COUNT as i32 * ANIMATED_SCENE_FRAME_SPACING_MS;
    let last_ts = decoder
        .into_iter()
        .last()
        .expect("animated webp has at least one frame")
        .timestamp();

    assert_eq!(
        last_ts, expected_ms,
        "decoder timeline end should match authored duration ({expected_ms} ms); got {last_ts} ms",
    );
}
