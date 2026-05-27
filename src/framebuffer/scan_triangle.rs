use glam::{UVec2, Vec2};

use super::{FrameBuffer, Rgb};

pub struct ScanTriangle {
    corners: [UVec2; 3],

    color: Rgb,
}

impl ScanTriangle {
    pub fn new(mut corners: [UVec2; 3], color: Rgb) -> Self {
        corners.sort_by_key(|v| v.y);
        Self { corners, color }
    }

    fn scan_lines(&self) -> impl Iterator<Item = (u32, u32, u32)> {
        let y_range = self.corners[0].y..=self.corners[2].y;
        let [a, b, c] = self.corners.map(|v| v.as_vec2());

        let ac_walker = EdgeWalker::new(a, c);
        let ab_walker = EdgeWalker::new(a, b);
        let bc_walker = EdgeWalker::new(b, c);

        let product = (b - a).perp_dot(c - a);
        let is_left = product < 0.0;

        y_range.map(move |y| {
            let y = y as f32;
            let mut x_end = ac_walker.get(y);
            let mut x_start = if y + 1.0 > b.y {
                bc_walker.get(y)
            } else {
                ab_walker.get(y)
            };

            if !is_left {
                std::mem::swap(&mut x_start, &mut x_end);
            }

            (
                x_start.round() as u32,
                x_end.round() as u32,
                y.round() as u32,
            )
        })
    }

    pub fn draw(&self, fb: &mut FrameBuffer) {
        for (x_min, x_max, y) in self.scan_lines() {
            fb.draw_horz_line(x_min, x_max, y, self.color);
        }
    }
}

impl FrameBuffer {
    fn draw_horz_line(&mut self, x_min: u32, x_max: u32, y: u32, color: Rgb) {
        for x in x_min..=x_max {
            self.set_pixel(x, y, color);
        }
    }
}

struct EdgeWalker {
    slope: f32,
    intercept: f32,
    default_x: f32,
}

impl EdgeWalker {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        let slope = if b.y == a.y {
            f32::NAN
        } else {
            (b.x - a.x) / (b.y - a.y)
        };
        let intercept = a.x - a.y * slope;
        Self {
            slope,
            intercept,
            default_x: a.x,
        }
    }

    pub fn get(&self, y: f32) -> f32 {
        if self.slope.is_nan() {
            self.default_x
        } else {
            y * self.slope + self.intercept
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FrameBuffer;

    use super::*;

    /// Degenerate input: two corners coincide—horizontal segment. All cross terms vanish for bbox samples, so pixels along that span are treated as inside.
    #[test]
    fn fill_degenerate_triangle_is_line_segment() {
        let mut fb = FrameBuffer::new(10, 5);
        ScanTriangle::new(pts([(2, 3), (8, 3), (7, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
            let expected = concat!(
                "          ",
                "          ",
                "          ",
                "  +++++++ ",
                "          ",
            );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    /// Degenerate input: two corners coincide—horizontal segment. All cross terms vanish for bbox samples, so pixels along that span are treated as inside.
    #[test]
    fn fill_degenerate_triangle_two_corners_coincide() {
        let mut fb = FrameBuffer::new(10, 5);
        ScanTriangle::new(pts([(2, 3), (7, 3), (7, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
            let expected = concat!(
                "          ",
                "          ",
                "          ",
                "  ++++++  ",
                "          ",
            );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    /// Degenerate input: all corners coincide. Every cross term is zero, so winding never flips—all bbox samples qualify.
    #[test]
    fn fill_degenerate_all_corners_same_point() {
        let mut fb = FrameBuffer::new(10, 5);
        ScanTriangle::new(pts([(4, 2), (4, 2), (4, 2)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
            let expected = concat!(
                "          ",
                "          ",
                "    +     ",
                "          ",
                "          ",
            );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn fill_axis_aligned_shapes_isosceles() {
        let mut fb = FrameBuffer::new(10, 5);
        // Apex at top; CCW cyclic order for consistent half-plane winding.
        ScanTriangle::new(pts([(4, 1), (6, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "    +     ",
            "   +++    ",
            "  +++++   ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn fill_same_triangle_under_rotated_vertex_order() {
        let corners_ccw = [(4, 1), (6, 3), (2, 3)];

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "    +     ",
            "   +++    ",
            "  +++++   ",
            "          ",
        );

        for start in 0..3 {
            let mut fb = FrameBuffer::new(10, 5);
            ScanTriangle::new(
                pts([
                    corners_ccw[start],
                    corners_ccw[(start + 1) % 3],
                    corners_ccw[(start + 2) % 3],
                ]),
                Rgb::WHITE,
            )
            .draw(&mut fb);
            assert_eq!(expected, fb.to_ascii_art(), "order start {}", start);
        }
    }

    #[test]
    fn draw_right_triangle() {
        let mut fb = FrameBuffer::new(10, 5);
        // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
        ScanTriangle::new(pts([(2, 1), (4, 1), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  +++     ",
            "  ++      ",
            "  +       ",
            "          ",
        );
        assert_eq!(expected, fb.to_ascii_art());
    }

    #[test]
    fn fill_clips_when_triangle_extends_past_buffer() {
        let mut fb = FrameBuffer::new(10, 5);
        // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
        ScanTriangle::new(pts([(2, 1), (14, 1), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  ++++++++",
            "  +++++++ ",
            "  +       ",
            "          ",
        );
        assert_eq!(expected, fb.to_ascii_art());
    }

    #[test]
    fn fill_slanted_triangle() {
        let mut fb = FrameBuffer::new(10, 5);
        ScanTriangle::new(pts([(2, 3), (7, 3), (8, 1)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "        + ",
            "     ++++ ",
            "  ++++++  ",
            "          ",
        );
        assert_eq!(expected, fb.to_ascii_art());
    }

    /// One edge is axis-aligned **vertical** (`x` constant)—exercises bbox + half-planes on a non-horizontal base.
    #[test]
    fn fill_triangle_with_vertical_edge() {
        let mut fb = FrameBuffer::new(10, 5);
        // Vertical segment (2,1)–(2,4); apex (6, 2). Consistent CCW half-plane winding.
        ScanTriangle::new(pts([(2, 1), (2, 4), (6, 2)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
            let expected = concat!(
                "          ",
                "  +       ",
                "  +++++   ",
                "  +++     ",
                "  +       ",
            );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    fn pts(corners: [(u32, u32); 3]) -> [UVec2; 3] {
        std::array::from_fn(|i| UVec2::new(corners[i].0, corners[i].1))
    }
}
