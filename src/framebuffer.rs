//! RGB framebuffer: `width × height` pixels, three `u8` channels per pixel, row-major.

mod colors;
mod interpolator;
mod phong_shaded_triangle;

pub use colors::Rgb;

pub use phong_shaded_triangle::{PhongCorner, PhongShadedTriangle};

/// Screen pixel location with **view-space depth** for depth testing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FbPoint {
    pub x: u32,
    pub y: u32,
    pub depth: f32,
}

impl FbPoint {
    pub const fn new(x: u32, y: u32, depth: f32) -> Self {
        Self { x, y, depth }
    }
}

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    depth: Vec<f32>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = width as usize * height as usize;
        Self {
            width,
            height,
            rgb: vec![0u8; pixel_count * 3],
            depth: vec![f32::INFINITY; pixel_count],
        }
    }

    pub fn clear(&mut self) {
        self.rgb.fill(0);
        self.depth.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgb) {
        let Some(i) = self.pixel_offset(x, y) else {
            return;
        };

        self.rgb[i] = color.0;
        self.rgb[i + 1] = color.1;
        self.rgb[i + 2] = color.2;
    }

    /// Writes **color** at **point** when **point.depth** is nearer than the stored depth.
    pub fn write_pixel(&mut self, point: FbPoint, color: Rgb) {
        let Some(i) = self.pixel_index(point.x, point.y) else {
            return;
        };

        if point.depth >= self.depth[i] {
            return;
        }

        self.depth[i] = point.depth;
        let rgb = i * 3;
        self.rgb[rgb] = color.0;
        self.rgb[rgb + 1] = color.1;
        self.rgb[rgb + 2] = color.2;
    }

    fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(y as usize * self.width as usize + x as usize)
        }
    }

    fn pixel_offset(&self, x: u32, y: u32) -> Option<usize> {
        self.pixel_index(x, y).map(|i| i * 3)
    }
}

impl AsRef<[u8]> for FrameBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.rgb
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    impl FrameBuffer {
        const ASCII_SHADES: [char; 4] = ['░', '▒', '▓', '█'];

        pub(crate) fn get_pixel(&self, x: u32, y: u32) -> Rgb {
            let Some(i) = self.pixel_offset(x, y) else {
                return Rgb::BLACK;
            };
            Rgb(self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
        }

        /// Multi-line text: one row per line. **`' '`** matches [`Rgb::BLACK`]; other pixels
        /// use block-element shades by [`Rgb::brightness`] (**`░` `▒` `▓` `█`**, light → dark).
        pub(crate) fn to_ascii_art(&self) -> String {
            let mut out = String::with_capacity(
                (self.height * self.width + self.height.saturating_sub(1)) as usize,
            );

            for y in 0..self.height {
                if y > 0 {
                    out.push('\n');
                }
                for x in 0..self.width {
                    out.push(Self::ascii_shade_for_rgb(self.get_pixel(x, y)));
                }
            }

            out
        }

        pub(crate) fn ascii_shade_for_rgb(color: Rgb) -> char {
            let brightness = color.brightness();
            if brightness == 0.0 {
                return ' ';
            }
            let level = (brightness * 4.0).ceil() as usize;
            Self::ASCII_SHADES[level.min(4) - 1]
        }
    }

    /// Builds expected ASCII art for tests from one string per framebuffer row.
    pub(crate) fn to_ascii_art(rows: &[&'static str]) -> String {
        rows.join("\n")
    }

    /// Asserts framebuffer ASCII art matches, with readable multi-line output on failure.
    pub(crate) fn assert_ascii_art_eq(actual: &str, expected: &str, context: &str) {
        assert!(
            actual == expected,
            "{prefix}FrameBuffer content does not match the expected\n\nExpected:\n{expected_boxed}\n\nActual:\n{actual_boxed}",
            prefix = if context.is_empty() {
                String::new()
            } else {
                format!("{context}\n\n")
            },
            expected_boxed = border_ascii_art(expected),
            actual_boxed = border_ascii_art(actual),
        );
    }

    /// Wraps multi-line ASCII art in a box for readable test failure output.
    pub(crate) fn border_ascii_art(art: &str) -> String {
        let lines: Vec<&str> = art.lines().collect();
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let horizontal = "─".repeat(width);

        let mut out = String::with_capacity((width + 2) * (lines.len() + 2) + 8);
        out.push('┌');
        out.push_str(&horizontal);
        out.push_str("┐\n");

        for line in lines {
            out.push('│');
            out.push_str(line);
            out.push_str(&" ".repeat(width - line.chars().count()));
            out.push_str("│\n");
        }

        out.push('└');
        out.push_str(&horizontal);
        out.push('┘');
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::framebuffer::test_helpers::{assert_ascii_art_eq, to_ascii_art};

    use super::*;

    #[test]
    fn clear_resets_depth_so_farther_write_succeeds() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.write_pixel(FbPoint::new(1, 1, 0.3), Rgb::WHITE);
        fb.write_pixel(FbPoint::new(1, 1, 0.8), Rgb(255, 0, 0));
        assert_eq!(fb.get_pixel(1, 1), Rgb::WHITE);

        fb.clear();
        fb.write_pixel(FbPoint::new(1, 1, 0.8), Rgb(255, 0, 0));
        assert_eq!(fb.get_pixel(1, 1), Rgb(255, 0, 0));
    }

    #[test]
    fn write_pixel_out_of_bounds_is_ignored() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.write_pixel(FbPoint::new(100, 0, 0.0), Rgb::WHITE);
        assert!(fb.as_ref().iter().all(|&b| b == 0));
    }

    #[test]
    fn write_pixel_accepts_nearer_depth() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.write_pixel(FbPoint::new(1, 1, 0.8), Rgb::WHITE);
        fb.write_pixel(FbPoint::new(1, 1, 0.3), Rgb(255, 0, 0));
        assert_eq!(fb.get_pixel(1, 1), Rgb(255, 0, 0));
    }

    #[test]
    fn write_pixel_rejects_farther_depth() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.write_pixel(FbPoint::new(1, 1, 0.3), Rgb::WHITE);
        fb.write_pixel(FbPoint::new(1, 1, 0.8), Rgb(255, 0, 0));
        assert_eq!(fb.get_pixel(1, 1), Rgb::WHITE);
    }

    #[test]
    fn write_pixel_stores_color_on_cleared_depth() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.write_pixel(FbPoint::new(1, 1, 0.5), Rgb::WHITE);
        assert_eq!(fb.get_pixel(1, 1), Rgb::WHITE);
    }

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
    fn border_ascii_art_draws_box_around_rows() {
        let boxed = test_helpers::border_ascii_art(" █\n██");
        assert_eq!(
            boxed,
            "┌──┐\n\
             │ █│\n\
             │██│\n\
             └──┘"
        );
    }

    #[test]
    fn to_ascii_art_maps_black_to_space_and_white_to_full_block() {
        let mut fb = FrameBuffer::new(2, 1);
        fb.set_pixel(0, 0, Rgb::BLACK);
        fb.set_pixel(1, 0, Rgb::WHITE);
        assert_ascii_art_eq(&fb.to_ascii_art(), &to_ascii_art(&[" █"]), "");
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
