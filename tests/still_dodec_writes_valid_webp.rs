//! Runs the **`still-dodec`** binary and checks that output WebP decodes correctly.

mod common;

use std::fs;

use webp_animation::Decoder;

use common::run_integration_binary;

const INTEGRATION_TEST_BIN: &str = "still-dodec";
const OUTPUT_FILE_NAME: &str = "test-still-dodec.webp";

#[test]
fn output_file_is_valid_webp() {
    let rendered = run_integration_binary(INTEGRATION_TEST_BIN, OUTPUT_FILE_NAME);
    let bytes = fs::read(rendered.output_path).expect("read output webp");

    Decoder::new(&bytes).expect("output file is a valid WebP file");
}
