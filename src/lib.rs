//! **Thorus Forge** — software rasterizer building blocks: RGB framebuffer, orthographic screen mapping, lossless WebP encode.

pub mod framebuffer;
pub mod ortho_camera;
pub mod webp_encoder;

pub use framebuffer::{FrameBuffer, Rgb};
pub use ortho_camera::Camera;
pub use webp_encoder::WebpEncoder;
