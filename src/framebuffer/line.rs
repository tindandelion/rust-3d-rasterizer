//! 2D line primitive (**DDA** rasterization via [`FrameBuffer::set_pixel`](super::FrameBuffer::set_pixel)).

use glam::UVec2;

use super::{FrameBuffer, Rgb};

/// Endpoint-inclusive segment in pixel space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub pt1: UVec2,
    pub pt2: UVec2,
    pub color: Rgb,
}

impl Line {
    pub fn new(pt1: UVec2, pt2: UVec2, color: Rgb) -> Self {
        Self { pt1, pt2, color }
    }

    /// **DDA-style:** parameter `t` in `[0, 1]` with `steps = max(|Δx|, |Δy|)` and floating drift
    /// along the segment; samples are rounded to integer pixels. Pixels outside the buffer are
    /// skipped (`set_pixel` guards).
    pub fn draw(self, fb: &mut FrameBuffer) {
        let x0 = self.pt1.x as f64;
        let y0 = self.pt1.y as f64;
        let dx = self.pt2.x as f64 - x0;
        let dy = self.pt2.y as f64 - y0;

        let dx_i = self.pt2.x as i64 - self.pt1.x as i64;
        let dy_i = self.pt2.y as i64 - self.pt1.y as i64;
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
                fb.set_pixel(px as u32, py as u32, self.color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::UVec2;

    use crate::framebuffer::test_helpers::{assert_ascii_art_eq, to_ascii_art};

    use super::super::{FrameBuffer, Rgb};
    use super::Line;

    #[test]
    fn draw_horizontal_line() {
        let mut fb = FrameBuffer::new(10, 5);
        Line::new(UVec2::new(1, 3), UVec2::new(8, 3), Rgb::WHITE).draw(&mut fb);

        let expected = to_ascii_art(&[
            "          ",
            "          ",
            "          ",
            " ████████ ",
            "          ",
        ]);
        assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
    }

    #[test]
    fn draw_vertical_line() {
        let mut fb = FrameBuffer::new(10, 5);
        Line::new(UVec2::new(3, 1), UVec2::new(3, 3), Rgb::WHITE).draw(&mut fb);

        let expected = to_ascii_art(&[
            "          ",
            "   █      ",
            "   █      ",
            "   █      ",
            "          ",
        ]);
        assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
    }

    #[test]
    fn draw_diagonal_line_slope_one() {
        let mut fb = FrameBuffer::new(10, 5);
        Line::new(UVec2::new(1, 0), UVec2::new(4, 3), Rgb::WHITE).draw(&mut fb);

        let expected = to_ascii_art(&[
            " █        ",
            "  █       ",
            "   █      ",
            "    █     ",
            "          ",
        ]);
        assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
    }

    #[test]
    fn draw_diagonal_line_reverse() {
        let mut fb = FrameBuffer::new(10, 5);
        Line::new(UVec2::new(4, 3), UVec2::new(1, 0), Rgb::WHITE).draw(&mut fb);

        let expected = to_ascii_art(&[
            " █        ",
            "  █       ",
            "   █      ",
            "    █     ",
            "          ",
        ]);
        assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
    }

    #[test]
    fn draw_line_clips_diagonal_when_end_lies_outside_buffer() {
        let mut fb = FrameBuffer::new(10, 5);
        // Start inside the buffer; end is beyond both width and height.
        Line::new(UVec2::new(3, 1), UVec2::new(12, 10), Rgb::WHITE).draw(&mut fb);

        let expected = to_ascii_art(&[
            "          ",
            "   █      ",
            "    █     ",
            "     █    ",
            "      █   ",
        ]);
        assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
    }
}
