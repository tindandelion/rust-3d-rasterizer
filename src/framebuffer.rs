//! RGB framebuffer: `width × height` pixels, three `u8` channels per pixel, row-major.

mod colors;
mod half_space_fill_triangle;
mod line;
mod scanline_fill_triangle;

pub use colors::Rgb;
pub use half_space_fill_triangle::HalfSpaceFillTriangle;
pub use line::Line;
pub use scanline_fill_triangle::ScanlineFillTriangle;

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn clear_black(&mut self) {
        self.rgb.fill(0);
    }

    fn pixel_offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some((y as usize * self.width as usize + x as usize) * 3)
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgb) {
        let Some(i) = self.pixel_offset(x, y) else {
            return;
        };

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
impl FrameBuffer {
    const ASCII_SHADES: [char; 4] = ['░', '▒', '▓', '█'];

    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> Rgb {
        let Some(i) = self.pixel_offset(x, y) else {
            return Rgb::BLACK;
        };
        Rgb(self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
    }

    /// Row-major text: row 0, then row 1, … with no separators. **`' '`** matches
    /// [`Rgb::BLACK`]; other pixels use block-element shades by [`Rgb::brightness`]
    /// (**`░` `▒` `▓` `█`**, light → dark).
    pub(crate) fn to_ascii_art(&self) -> String {
        let mut out = String::with_capacity((self.height * self.width) as usize);

        for y in 0..self.height {
            for x in 0..self.width {
                out.push(Self::ascii_shade_for_rgb(self.get_pixel(x, y)));
            }
        }

        out
    }

    fn ascii_shade_for_rgb(color: Rgb) -> char {
        let brightness = color.brightness();
        if brightness == 0.0 {
            return ' ';
        }
        let level = (brightness * 4.0).ceil() as usize;
        Self::ASCII_SHADES[level.min(4) - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_frame_buffer() {
        let fb = FrameBuffer::new(10, 5);

        assert_eq!(fb.as_ref().len(), 10 * 5 * 3);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn set_pixel_writes_rgb_at_correct_offset() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.set_pixel(1, 1, Rgb::WHITE);

        assert_eq!(fb.get_pixel(1, 1), Rgb::WHITE);
        assert_eq!(fb.get_pixel(0, 0), Rgb::BLACK);

        #[rustfmt::skip]
        let expected = [
            0, 0, 0,            0,            0,            0, 0, 0, 0,
            0, 0, 0, Rgb::WHITE.0, Rgb::WHITE.1, Rgb::WHITE.2, 0, 0, 0,
            0, 0, 0,            0,            0,            0, 0, 0, 0,
        ];
        assert_eq!(fb.as_ref(), expected.as_slice());
    }

    #[test]
    fn set_pixel_out_of_bounds_is_ignored() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set_pixel(100, 0, Rgb::WHITE);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn to_ascii_art_maps_black_to_space_and_white_to_full_block() {
        let mut fb = FrameBuffer::new(2, 1);
        fb.set_pixel(0, 0, Rgb::BLACK);
        fb.set_pixel(1, 0, Rgb::WHITE);
        assert_eq!(fb.to_ascii_art(), " █");
    }

    #[test]
    fn ascii_shade_for_rgb_buckets_brightness() {
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb::BLACK), ' ');
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb(32, 32, 32)), '░');
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb(64, 64, 64)), '▒');
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb(128, 128, 128)), '▓');
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb(200, 200, 200)), '█');
        assert_eq!(FrameBuffer::ascii_shade_for_rgb(Rgb::WHITE), '█');
    }
}
