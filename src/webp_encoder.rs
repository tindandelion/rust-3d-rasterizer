use std::fs;
use std::path::Path;

use webp_animation::{ColorMode, Encoder, EncoderOptions, EncodingConfig, EncodingType, Error};

pub struct WebpEncoder {
    encoder: Encoder,
    next_timestamp_ms: i32,
}

impl WebpEncoder {
    pub fn new(width: u32, height: u32) -> Result<Self, Error> {
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
        })
    }

    pub fn add_frame(&mut self, rgb: impl AsRef<[u8]>) -> Result<(), Error> {
        let ts = self.next_timestamp_ms;
        self.encoder.add_frame(rgb.as_ref(), ts)?;
        self.next_timestamp_ms = ts.saturating_add(1);
        Ok(())
    }

    pub fn write(self, out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let webp = self.encoder.finalize(self.next_timestamp_ms)?;
        fs::write(out_path, &webp)?;
        Ok(())
    }
}
