//! RGB framebuffer: `width × height` pixels, three `u8` channels per pixel, row-major.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub struct FrameBuffer {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = width as usize * height as usize;
        Self {
            width,
            height,
            rgb: vec![0u8; pixel_count * 3],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgb) {
        if x >= self.width || y >= self.height {
            return;
        }

        let i = (y as usize * self.width as usize + x as usize) * 3;
        self.rgb[i] = color.0;
        self.rgb[i + 1] = color.1;
        self.rgb[i + 2] = color.2;
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

    const PIXEL_COLOR: Rgb = Rgb(10, 20, 30);

    #[test]
    fn initialize_frame_buffer() {
        let fb = FrameBuffer::new(10, 10);

        assert_eq!(fb.as_ref().len(), 10 * 10 * 3);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn set_pixel_writes_rgb_at_correct_offset() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.set_pixel(1, 1, PIXEL_COLOR);

        #[rustfmt::skip]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, PIXEL_COLOR.0, PIXEL_COLOR.1, PIXEL_COLOR.2, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(fb.as_ref(), expected.as_slice());
    }

    #[test]
    fn set_pixel_out_of_bounds_is_ignored() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set_pixel(100, 0, PIXEL_COLOR);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }
}
