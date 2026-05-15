use std::fs;
use std::path::Path;

use webp_animation::{ColorMode, Encoder, EncoderOptions, EncodingConfig, EncodingType};

pub struct WebpEncoder {
    encoder: Encoder,
    next_timestamp_ms: i32,
    frame_spacing_ms: i32,
}

impl WebpEncoder {
    pub fn new(width: u32, height: u32) -> Result<Self, webp_animation::Error> {
        Self::with_frame_spacing(width, height, 1)
    }

    /// Lossless encoder with **`frame_spacing_ms`** between successive frame timestamps
    /// (WebP requires strictly increasing timestamps; see **`webp_animation::Encoder::add_frame`**).
    ///
    /// **`frame_spacing_ms`** must be **`>= 1`**.
    pub fn with_frame_spacing(
        width: u32,
        height: u32,
        frame_spacing_ms: i32,
    ) -> Result<Self, webp_animation::Error> {
        assert!(
            frame_spacing_ms >= 1,
            "frame_spacing_ms must be >= 1 for strictly increasing timestamps"
        );

        let encoder = Encoder::new_with_options(
            (width, height),
            EncoderOptions {
                encoding_config: Some(EncodingConfig {
                    encoding_type: EncodingType::Lossless,
                    ..Default::default()
                }),
                color_mode: ColorMode::Rgb,
                ..Default::default()
            },
        )?;

        Ok(Self {
            encoder,
            next_timestamp_ms: 0,
            frame_spacing_ms,
        })
    }

    pub fn add_frame(&mut self, rgb: impl AsRef<[u8]>) -> Result<(), webp_animation::Error> {
        let ts = self.next_timestamp_ms;
        self.encoder.add_frame(rgb.as_ref(), ts)?;
        self.next_timestamp_ms = ts.saturating_add(self.frame_spacing_ms);
        Ok(())
    }

    pub fn write(self, out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let webp = self.encoder.finalize(self.next_timestamp_ms)?;
        fs::write(out_path, &webp)?;
        Ok(())
    }
}
