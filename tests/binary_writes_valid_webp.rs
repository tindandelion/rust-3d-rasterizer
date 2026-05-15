//! Runs the `still-cube` binary and checks that output WebP decodes correctly.

mod common;

use std::fs;

use webp_animation::Decoder;

use common::run_package_binary;

const OUTPUT_FILE_NAME: &str = "test-scene.webp";

#[test]
fn output_file_is_valid_webp() {
    let rendered = run_package_binary(OUTPUT_FILE_NAME);
    let bytes = fs::read(rendered.output_path).expect("read output webp");

    Decoder::new(&bytes).expect("output file is a valid WebP file");
}
