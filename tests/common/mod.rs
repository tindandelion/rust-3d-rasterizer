//! Shared helpers for integration tests. Each `tests/*.rs` binary imports this via `mod common;`.
#![allow(dead_code)]
// Not every test binary uses every helper; suppress `dead_code` per submodule when building
// separate integration-test crates (`tests/*.rs`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;
use webp_animation::{ColorMode, Decoder};

/// Decoded first frame of a **WebP** still (**RGBA** pixels, same decoder path as
/// [`run_package_binary`] output).
pub struct WebpImage {
    pub dimensions: (u32, u32),
    pub rgba: Vec<u8>,
}

impl WebpImage {
    /// Load and decode a file (single-frame still or first frame only).
    pub fn read(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("read webp {}: {e}", path.display()));
        let (dimensions, rgba) = decode_first_webp_frame(&bytes);
        Self { dimensions, rgba }
    }
}

/// Result of [`run_package_binary`]: absolute path to the **`.webp`** inside a temp directory.
/// Keep this value alive until you finish reading the file; dropping it removes the directory.
pub struct RenderedWebp {
    _temp_dir: TempDir,
    pub output_path: PathBuf,
}

/// Resolves `CARGO_BIN_EXE_<pkg>` for this package’s binary (same layout as `cargo test`).
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

/// Creates a temp workspace, runs the package binary once (**`current_dir`** = that directory,
/// single CLI arg = relative **`.webp`** path), and returns the **absolute** path to the file.
pub fn run_package_binary(output_relative_to_temp: impl AsRef<Path>) -> RenderedWebp {
    let dir = tempfile::tempdir().expect("temp directory");
    let rel = output_relative_to_temp.as_ref();
    let status = Command::new(package_binary_path())
        .current_dir(dir.path())
        .arg(rel)
        .status()
        .expect("spawn binary");
    assert!(status.success(), "binary exited with {status}");
    let path = dir.path().join(rel);
    RenderedWebp {
        _temp_dir: dir,
        output_path: path,
    }
}

/// Joins `relative` to the crate root (`CARGO_MANIFEST_DIR`).
pub fn manifest_path(relative: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// First frame of a WebP still or animation, default **RGBA** (`webp_animation` decoder defaults).
///
/// Using **`ColorMode::Rgb`** decode can fail on **VP8L** stills from our encoder; tests keep the
/// default **RGBA** path.
fn decode_first_webp_frame(webp: &[u8]) -> ((u32, u32), Vec<u8>) {
    let decoder = Decoder::new(webp).expect("decode webp");
    let dims = decoder.dimensions();
    let mut it = decoder.into_iter();
    let frame = it.next().expect("at least one frame in still or animation");
    assert!(
        it.next().is_none(),
        "expected a single-frame still, got multiple frames"
    );
    assert_eq!(frame.dimensions(), dims);
    assert_eq!(frame.color_mode(), ColorMode::Rgba);

    (dims, frame.data().to_vec())
}
