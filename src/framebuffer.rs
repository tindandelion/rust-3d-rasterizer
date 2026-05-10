//! RGB framebuffer: `width × height` pixels, three `u8` channels per pixel, row-major.

pub struct FrameBuffer {
    rgb: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = width as usize * height as usize;
        Self {
            rgb: vec![0u8; pixel_count * 3],
        }
    }
}

impl AsRef<[u8]> for FrameBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.rgb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_frame_buffer() {
        let fb = FrameBuffer::new(10, 10);

        assert_eq!(fb.as_ref().len(), 10 * 10 * 3);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }
}
