//! Filled triangle in pixel space (half-plane / perp-dot inside test per pixel, bbox scan).

use glam::{UVec2, Vec2};

use super::{FrameBuffer, Rgb};

/// Three vertices in cyclic order (either winding). **Flat** fill: one RGB for the whole triangle.
///
/// Corners are supplied as integer pixel positions ([`UVec2`]) and stored internally as [`Vec2`] so edge
/// vectors and the 2D cross test use ordinary signed subtraction ([`Vec2::perp_dot`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FillTriangle {
    corners: [Vec2; 3],
    /// Inclusive axis-aligned scan bounds in pixels: minimum corner, then maximum corner.
    bounding_rect: [UVec2; 2],
    pub color: Rgb,
}

impl FillTriangle {
    pub fn new(corners: [UVec2; 3], color: Rgb) -> Self {
        let bounding_rect = calculate_bounding_rect(&corners);
        let corners = std::array::from_fn(|i| corners[i].as_vec2());
        Self {
            corners,
            bounding_rect,
            color,
        }
    }

    /// Rasterizes interior **inclusive** of edges. Pixels outside the buffer are skipped via [`FrameBuffer::set_pixel`].
    pub fn draw(self, fb: &mut FrameBuffer) {
        let top_left = self.bounding_rect[0];
        let bottom_right = self.bounding_rect[1];

        for y in top_left.y..=bottom_right.y {
            for x in top_left.x..=bottom_right.x {
                let p = Vec2::new(x as f32, y as f32);
                if self.is_point_inside(p) {
                    fb.set_pixel(x, y, self.color);
                }
            }
        }
    }

    fn is_point_inside(&self, p: Vec2) -> bool {
        let mut winding: i8 = 0;
        for i in 0..3 {
            let a = self.corners[i];
            let b = self.corners[(i + 1) % 3];
            let cross = (b - a).perp_dot(p - a);
            if cross == 0.0 {
                continue;
            }
            let w = if cross > 0.0 { 1 } else { -1 };
            match winding {
                0 => winding = w,
                x if x == w => {}
                _ => return false,
            }
        }
        true
    }
}

/// Inclusive pixel-space AABB: **`[0]`** = min corner, **`[1]`** = max corner.
fn calculate_bounding_rect(corners: &[UVec2; 3]) -> [UVec2; 2] {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    for v in corners {
        min_x = min_x.min(v.x);
        min_y = min_y.min(v.y);
        max_x = max_x.max(v.x);
        max_y = max_y.max(v.y);
    }

    [UVec2::new(min_x, min_y), UVec2::new(max_x, max_y)]
}

#[cfg(test)]
mod tests {
    use glam::UVec2;

    use super::super::{FrameBuffer, Rgb};
    use super::FillTriangle;

    fn pts(corners: [(u32, u32); 3]) -> [UVec2; 3] {
        std::array::from_fn(|i| UVec2::new(corners[i].0, corners[i].1))
    }

    /// Degenerate input: all corners coincide. Every cross term is zero, so winding never flips—all bbox samples qualify.
    #[test]
    fn fill_degenerate_all_corners_same_point() {
        let mut fb = FrameBuffer::new(10, 5);
        FillTriangle::new(pts([(4, 2), (4, 2), (4, 2)]), Rgb::WHITE).draw(&mut fb);

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
        FillTriangle::new(pts([(4, 1), (6, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

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
            FillTriangle::new(
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
    fn fill_clips_when_triangle_extends_past_buffer() {
        let mut fb = FrameBuffer::new(10, 5);
        // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
        FillTriangle::new(pts([(2, 1), (14, 1), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  ++++++++",
            "  +++++++ ",
            "  +       ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn fill_slanted_triangle() {
        let mut fb = FrameBuffer::new(10, 5);
        FillTriangle::new(pts([(2, 3), (7, 3), (8, 1)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "        + ",
            "     +++  ",
            "  ++++++  ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    /// One edge is axis-aligned **vertical** (`x` constant)—exercises bbox + half-planes on a non-horizontal base.
    #[test]
    fn fill_triangle_with_vertical_edge() {
        let mut fb = FrameBuffer::new(10, 5);
        // Vertical segment (2,1)–(2,4); apex (6, 2). Consistent CCW half-plane winding.
        FillTriangle::new(pts([(2, 1), (2, 4), (6, 2)]), Rgb::WHITE).draw(&mut fb);

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

    /// Degenerate input: two corners coincide—horizontal segment. All cross terms vanish for bbox samples, so pixels along that span are treated as inside.
    #[test]
    fn fill_degenerate_triangle_is_line_segment() {
        let mut fb = FrameBuffer::new(10, 5);
        FillTriangle::new(pts([(2, 3), (7, 3), (7, 3)]), Rgb::WHITE).draw(&mut fb);

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
}
