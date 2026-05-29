//! Filled triangle in pixel space (y-sorted scanlines, edge interpolation, horizontal spans).

use glam::{UVec2, Vec2};

use super::{FrameBuffer, Rgb, linear_fn::LinearFn};

#[derive(Clone, Copy, Debug)]
pub struct ShadedCorner {
    pub pos: UVec2,
    pub intensity: f32,
}

pub struct ShadedFillTriangle {
    corners: [ShadedCorner; 3],
    color: Rgb,
}

impl ShadedFillTriangle {
    pub fn new(mut corners: [ShadedCorner; 3], color: Rgb) -> Self {
        corners.sort_by_key(|v| v.pos.y);
        Self { corners, color }
    }

    pub fn draw(&self, fb: &mut FrameBuffer) {
        for ((x1, start_intensity), (x2, end_intensity), y) in self.scan_lines() {
            let horz_intensity =
                LinearFn::from_endpoints((x1 as f32, start_intensity), (x2 as f32, end_intensity));
            for x in x1..=x2 {
                let intensity = horz_intensity.get(x as f32);
                fb.set_pixel(x, y, self.color.scale(intensity));
            }
        }
    }

    fn scan_lines(&self) -> impl Iterator<Item = ((u32, f32), (u32, f32), u32)> {
        let y_range = self.corners[0].pos.y..=self.corners[2].pos.y;
        let midpoint = self.corners[1].pos;

        let ac_edge = EdgeWalker::from_corners(self.corners[0], self.corners[2]);
        let ab_edge = EdgeWalker::from_corners(self.corners[0], self.corners[1]);
        let bc_edge = EdgeWalker::from_corners(self.corners[1], self.corners[2]);

        y_range.map(move |y| {
            let current_edge = if y + 1 > midpoint.y {
                &bc_edge
            } else {
                &ab_edge
            };

            let y = y as f32;
            let mut x_start = current_edge.get(y);
            let mut x_end = ac_edge.get(y);

            if x_end < x_start {
                std::mem::swap(&mut x_start, &mut x_end);
            }

            (
                (x_start.0.round() as u32, x_start.1),
                (x_end.0.round() as u32, x_end.1),
                y.round() as u32,
            )
        })
    }
}

struct EdgeWalker {
    start_pt: (Vec2, f32),
    x_interp: LinearFn,
    intensity_interp: LinearFn,
}

impl EdgeWalker {
    pub fn from_corners(a: ShadedCorner, b: ShadedCorner) -> Self {
        {
            let pos_a = a.pos.as_vec2();
            let pos_b = b.pos.as_vec2();

            Self {
                start_pt: (pos_a, a.intensity),
                x_interp: LinearFn::from_endpoints((pos_a.y, pos_a.x), (pos_b.y, pos_b.x)),
                intensity_interp: LinearFn::from_endpoints(
                    (0.0, a.intensity),
                    ((pos_b - pos_a).length(), b.intensity),
                ),
            }
        }
    }

    pub fn get(&self, y: f32) -> (f32, f32) {
        let x = self.x_interp.get(y);
        let current_distance = (Vec2::new(x, y) - self.start_pt.0).length();
        let intensity = self.intensity_interp.get(current_distance);
        (x, intensity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameBuffer;
    use crate::framebuffer::{assert_ascii_art_eq, to_ascii_art};

    mod draw_triangle_shapes {
        use super::*;

        /// Degenerate input: two corners coincide—horizontal segment. All cross terms vanish for bbox samples, so pixels along that span are treated as inside.
        #[test]
        fn fill_degenerate_triangle_is_line_segment() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(pts([(2, 3), (8, 3), (7, 3)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ███████ ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Degenerate input: two corners coincide—horizontal segment. All cross terms vanish for bbox samples, so pixels along that span are treated as inside.
        #[test]
        fn fill_degenerate_triangle_two_corners_coincide() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(pts([(2, 3), (7, 3), (7, 3)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ██████  ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Degenerate input: all corners coincide. Every cross term is zero, so winding never flips—all bbox samples qualify.
        #[test]
        fn fill_degenerate_all_corners_same_point() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(pts([(4, 2), (4, 2), (4, 2)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "          ",
                "    █     ",
                "          ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        #[test]
        fn fill_axis_aligned_shapes_isosceles() {
            let mut fb = FrameBuffer::new(10, 5);
            // Apex at top; CCW cyclic order for consistent half-plane winding.
            ShadedFillTriangle::new(pts([(4, 1), (6, 3), (2, 3)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ███    ",
                "  █████   ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        #[test]
        fn fill_same_triangle_under_rotated_vertex_order() {
            let corners_ccw = [(4, 1), (6, 3), (2, 3)];

            let expected = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ███    ",
                "  █████   ",
                "          ",
            ]);

            for start in 0..3 {
                let mut fb = FrameBuffer::new(10, 5);
                ShadedFillTriangle::new(
                    pts([
                        corners_ccw[start],
                        corners_ccw[(start + 1) % 3],
                        corners_ccw[(start + 2) % 3],
                    ]),
                    Rgb::WHITE,
                )
                .draw(&mut fb);
                assert_ascii_art_eq(
                    &fb.to_ascii_art(),
                    &expected,
                    &format!("order start {start}"),
                );
            }
        }

        #[test]
        fn draw_right_triangle() {
            let mut fb = FrameBuffer::new(10, 5);
            // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
            ShadedFillTriangle::new(pts([(2, 1), (4, 1), (2, 3)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "  ███     ",
                "  ██      ",
                "  █       ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        #[test]
        fn fill_clips_when_triangle_extends_past_buffer() {
            let mut fb = FrameBuffer::new(10, 5);
            // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
            ShadedFillTriangle::new(pts([(2, 1), (14, 1), (2, 3)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "  ████████",
                "  ███████ ",
                "  █       ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        #[test]
        fn fill_slanted_triangle() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(pts([(2, 3), (7, 3), (8, 1)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "        █ ",
                "     ████ ",
                "  ██████  ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// One edge is axis-aligned **vertical** (`x` constant)—exercises bbox + half-planes on a non-horizontal base.
        #[test]
        fn fill_triangle_with_vertical_edge() {
            let mut fb = FrameBuffer::new(10, 5);
            // Vertical segment (2,1)–(2,4); apex (6, 2). Consistent CCW half-plane winding.
            ShadedFillTriangle::new(pts([(2, 1), (2, 4), (6, 2)]), Rgb::WHITE).draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "  █       ",
                "  █████   ",
                "  ███     ",
                "  █       ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        fn pts(corners: [(u32, u32); 3]) -> [ShadedCorner; 3] {
            std::array::from_fn(|i| ShadedCorner {
                pos: UVec2::new(corners[i].0, corners[i].1),
                intensity: 1.0,
            })
        }
    }

    mod shading_triangles {
        use super::*;

        /// Uniform corner intensity scales the base color across the whole fill.
        #[test]
        fn uniform_intensity_scales_color_on_horizontal_segment() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(
                pts([((2, 3), 0.5), ((8, 3), 0.5), ((7, 3), 0.5)]),
                Rgb::WHITE,
            )
            .draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ▓▓▓▓▓▓▓ ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Intensity varies along a horizontal degenerate fill (Gouraud: lerp along the span).
        #[test]
        fn horizontal_span_interpolates_intensity_from_corners() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(
                pts([((2, 3), 1.0), ((8, 3), 0.1), ((7, 3), 1.0 / 6.0)]),
                Rgb::WHITE,
            )
            .draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ██▓▓▒▒░ ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Bright apex, dark base: each scanline gets edge-interpolated intensities at `y`, then span fill.
        #[test]
        fn isosceles_interpolates_intensity_along_edges_per_scanline() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(
                pts([((4, 1), 1.0), ((6, 3), 0.1), ((2, 3), 0.1)]),
                Rgb::WHITE,
            )
            .draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ▓▓▓    ",
                "  ░░░░░   ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Right triangle clipped at buffer width; intensity still interpolates on drawable spans.
        #[test]
        fn fill_clips_when_triangle_extends_past_buffer() {
            let mut fb = FrameBuffer::new(10, 5);
            // Right-angle corner at (2,1); hypotenuse runs toward (14,1) so only x ∈ [2,9] is drawable.
            ShadedFillTriangle::new(
                pts([((2, 1), 1.0), ((14, 1), 0.1), ((2, 3), 0.1)]),
                Rgb::WHITE,
            )
            .draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "  ████▓▓▓▒",
                "  ▓▒▒▒▒░░ ",
                "  ░       ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        /// Slanted triangle: intensity varies along both edges and within each span.
        #[test]
        fn slanted_triangle_interpolates_intensity_in_x_and_y() {
            let mut fb = FrameBuffer::new(10, 5);
            ShadedFillTriangle::new(
                pts([((2, 3), 0.1), ((7, 3), 1.0), ((8, 1), 0.5)]),
                Rgb::WHITE,
            )
            .draw(&mut fb);

            let expected = to_ascii_art(&[
                "          ",
                "        ▓ ",
                "     ▒▒▓▓ ",
                "  ░▒▒▓██  ",
                "          ",
            ]);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        fn pts(corners: [((u32, u32), f32); 3]) -> [ShadedCorner; 3] {
            std::array::from_fn(|i| ShadedCorner {
                pos: UVec2::new(corners[i].0.0, corners[i].0.1),
                intensity: corners[i].1,
            })
        }
    }
}
