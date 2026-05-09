//! Runs the `rust-3d-rasterizer` binary and checks that output WebP decodes correctly.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;
use webp_animation::Decoder;

const OUTPUT_FILE_NAME: &str = "test-scene.webp";

fn package_binary_path() -> PathBuf {
    let pkg_name = std::env::var("CARGO_PKG_NAME")
        .expect("CARGO_PKG_NAME is set when integration tests run via `cargo test`");

    let key = format!("CARGO_BIN_EXE_{pkg_name}");
    std::env::var_os(&key)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            panic!(
                "`{key}` must be set when integration tests run via `cargo test` \
                 (Cargo supplies the path to the package binary)"
            )
        })
}

#[test]
fn output_file_is_valid_webp() {
    let dir = tempdir().expect("temp directory");
    let status = Command::new(package_binary_path())
        .current_dir(dir.path())
        .arg(OUTPUT_FILE_NAME)
        .status()
        .expect("spawn binary");

    assert!(status.success(), "binary exited with {status}");

    let webp_path = dir.path().join(OUTPUT_FILE_NAME);
    let bytes = fs::read(&webp_path).expect("read output webp");

    Decoder::new(&bytes).expect("output file is a valid WebP file");
}
