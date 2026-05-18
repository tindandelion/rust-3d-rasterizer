//! Filled **convex** quad in pixel space (half-plane / cross-product inside test per pixel, bbox scan).

use glam::{UVec2, Vec2};

use super::{FrameBuffer, Rgb};

/// Four **convex** corners in cyclic order (either winding). **Flat** fill: one RGB for the whole quad.
///
/// Corners are supplied as integer pixel positions ([`UVec2`]) and stored internally as [`Vec2`] so edge
/// vectors and the 2D cross test use ordinary signed subtraction ([`Vec2::perp_dot`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FillQuad {
    corners: [Vec2; 4],
    /// Inclusive axis-aligned scan bounds in pixels: minimum corner, then maximum corner.
    bounding_rect: [UVec2; 2],
    pub color: Rgb,
}

impl FillQuad {
    pub fn new(corners: [UVec2; 4], color: Rgb) -> Self {
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

    /// Strictly convex quad (no three collinear vertices in a degenerate way that splits winding).
    fn is_point_inside(&self, p: Vec2) -> bool {
        let mut winding: i8 = 0;
        for i in 0..4 {
            let a = self.corners[i];
            let b = self.corners[(i + 1) % 4];
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
fn calculate_bounding_rect(corners: &[UVec2; 4]) -> [UVec2; 2] {
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
    use super::FillQuad;

    fn pts(corners: [(u32, u32); 4]) -> [UVec2; 4] {
        std::array::from_fn(|i| UVec2::new(corners[i].0, corners[i].1))
    }

    #[test]
    fn fill_axis_aligned_rectangle() {
        let mut fb = FrameBuffer::new(10, 5);
        // Cyclic corners: bottom-left → bottom-right → top-right → top-left.
        FillQuad::new(pts([(2, 1), (7, 1), (7, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  ++++++  ",
            "  ++++++  ",
            "  ++++++  ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn fill_same_quad_under_rotated_vertex_order() {
        let corners_ccw = [(2, 1), (7, 1), (7, 3), (2, 3)];

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  ++++++  ",
            "  ++++++  ",
            "  ++++++  ",
            "          ",
        );

        for start in 0..4 {
            let mut fb = FrameBuffer::new(10, 5);
            FillQuad::new(
                pts([
                    corners_ccw[start],
                    corners_ccw[(start + 1) % 4],
                    corners_ccw[(start + 2) % 4],
                    corners_ccw[(start + 3) % 4],
                ]),
                Rgb::WHITE,
            )
            .draw(&mut fb);
            let art = fb.to_ascii_art();
            assert_eq!(expected, art, "order start {}", start);
        }
    }

    #[test]
    fn fill_clips_when_quad_extends_past_buffer() {
        let mut fb = FrameBuffer::new(10, 5);
        // Axis-aligned rectangle mostly to the right of the buffer; only x ∈ [2,9] is drawable.
        FillQuad::new(pts([(2, 1), (14, 1), (14, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            "  ++++++++",
            "  ++++++++",
            "  ++++++++",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    #[test]
    fn fill_convex_slanted_quad() {
        let mut fb = FrameBuffer::new(10, 5);
        FillQuad::new(pts([(2, 3), (7, 3), (8, 1), (1, 1)]), Rgb::WHITE).draw(&mut fb);

        #[rustfmt::skip]
        let expected = concat!(
            "          ",
            " ++++++++ ",
            "  ++++++  ",
            "  ++++++  ",
            "          ",
        );
        assert_eq!(fb.to_ascii_art(), expected);
    }

    /// Degenerate input: all corners coincide. The half-plane test treats every sample as inside (all cross terms are zero).
    #[test]
    fn fill_degenerate_all_corners_same_point() {
        let mut fb = FrameBuffer::new(10, 5);
        FillQuad::new(pts([(4, 2), (4, 2), (4, 2), (4, 2)]), Rgb::WHITE).draw(&mut fb);

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

    /// Degenerate input: four corners collapse to a horizontal segment (duplicate endpoints). Bounding box is a 1-pixel-tall stripe.
    #[test]
    fn fill_degenerate_quad_is_line_segment() {
        let mut fb = FrameBuffer::new(10, 5);
        FillQuad::new(pts([(2, 3), (7, 3), (7, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

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
