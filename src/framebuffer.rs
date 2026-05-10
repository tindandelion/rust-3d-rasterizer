//! RGB framebuffer: `width × height` pixels, three `u8` channels per pixel, row-major.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point(pub u32, pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    #[allow(dead_code)]
    pub const BLACK: Self = Self(0, 0, 0);
    pub const WHITE: Self = Self(255, 255, 255);
}

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

    /// Endpoint-inclusive segment (`pt1`–`pt2`). **DDA-style:** parameter `t` in `[0, 1]` with
    /// `steps = max(|Δx|, |Δy|)` and floating drift along the segment; samples are rounded to
    /// integer pixels. Pixels outside the buffer are skipped (`set_pixel` guards).
    pub fn draw_line(&mut self, pt1: Point, pt2: Point, color: Rgb) {
        let x0 = pt1.0 as f64;
        let y0 = pt1.1 as f64;
        let dx = pt2.0 as f64 - x0;
        let dy = pt2.1 as f64 - y0;

        let dx_i = pt2.0 as i64 - pt1.0 as i64;
        let dy_i = pt2.1 as i64 - pt1.1 as i64;
        let nx = dx_i.unsigned_abs();
        let ny = dy_i.unsigned_abs();
        let steps = nx.max(ny);

        for i in 0..=steps {
            let t = if steps == 0 {
                0.0
            } else {
                i as f64 / steps as f64
            };
            let px = (x0 + dx * t).round();
            let py = (y0 + dy * t).round();

            if px >= 0.0 && py >= 0.0 {
                self.set_pixel(px as u32, py as u32, color);
            }
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
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, Rgb::WHITE.0, Rgb::WHITE.1, Rgb::WHITE.2, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(fb.as_ref(), expected.as_slice());
    }

    #[test]
    fn get_pixel_out_of_bounds_returns_black() {
        let fb = FrameBuffer::new(2, 2);
        assert_eq!(fb.get_pixel(9, 0), Rgb::BLACK);
        assert_eq!(fb.get_pixel(0, 9), Rgb::BLACK);
    }

    #[test]
    fn set_pixel_out_of_bounds_is_ignored() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set_pixel(100, 0, Rgb::WHITE);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn draw_horizontal_line() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.draw_line(Point(1, 3), Point(8, 3), Rgb::WHITE);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "          ",
            "          ",
            " ++++++++ ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn draw_vertical_line() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.draw_line(Point(3, 1), Point(3, 3), Rgb::WHITE);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "   +      ",
            "   +      ",
            "   +      ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn draw_diagonal_line_slope_one() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.draw_line(Point(1, 0), Point(4, 3), Rgb::WHITE);

        #[rustfmt::skip]
        let expected = concat!(
            " +        ",
            "  +       ",
            "   +      ",
            "    +     ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn draw_diagonal_line_reverse() {
        let mut fb = FrameBuffer::new(10, 5);
        fb.draw_line(Point(4, 3), Point(1, 0), Rgb::WHITE);

        #[rustfmt::skip]
        let expected = concat!(
            " +        ",
            "  +       ",
            "   +      ",
            "    +     ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn draw_line_clips_diagonal_when_end_lies_outside_buffer() {
        let mut fb = FrameBuffer::new(10, 5);
        // Start inside the buffer; end is beyond both width and height.
        fb.draw_line(Point(3, 1), Point(12, 10), Rgb::WHITE);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "   +      ",
            "    +     ",
            "     +    ",
            "      +   ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    impl FrameBuffer {
        pub fn get_pixel(&self, x: u32, y: u32) -> Rgb {
            let Some(i) = self.pixel_offset(x, y) else {
                return Rgb::BLACK;
            };
            Rgb(self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
        }

        /// Row-major text: row 0, then row 1, … with no separators. `' '` matches `Rgb::BLACK`, `'+'` any other color.
        pub fn to_ascii_art(&self) -> String {
            let mut out = String::with_capacity((self.height * self.width) as usize);

            for y in 0..self.height {
                for x in 0..self.width {
                    let ch = if self.get_pixel(x, y) == Rgb::BLACK {
                        ' '
                    } else {
                        '+'
                    };
                    out.push(ch);
                }
            }

            out
        }
    }
}
