//! Filled triangle with per-pixel Phong shading: interpolate normals, evaluate lighting per fragment.

use glam::{Vec2, Vec3};

use crate::{
    framebuffer::{FbPoint, interpolator::Interpolator},
    geometry::UnitVec3,
};

use super::{FrameBuffer, Rgb};

#[derive(Clone, Copy, Debug)]
pub struct PhongCorner {
    pub point: FbPoint,
    pub normal: UnitVec3,
}

pub struct PhongShadedTriangle {
    corners: [PhongCorner; 3],
    color: Rgb,
}

impl PhongShadedTriangle {
    pub fn new(mut corners: [PhongCorner; 3], color: Rgb) -> Self {
        corners.sort_by_key(|v| v.point.y);
        Self { corners, color }
    }

    pub fn draw(&self, fb: &mut FrameBuffer, intensity_fn: impl Fn(UnitVec3) -> f32) {
        for ((x1, z1, start_normal), (x2, z2, end_normal), y) in self.scan_lines() {
            let horz_normal = NormalInterpolator::from_endpoints(
                (x1 as f32, start_normal),
                (x2 as f32, end_normal),
            );
            let z_interp = Interpolator::from_endpoints((x1 as f32, z1), (x2 as f32, z2));
            for x in x1..=x2 {
                let depth = z_interp.get(x as f32);
                let normal = horz_normal.get(x as f32);
                let intensity = intensity_fn(normal);
                fb.write_pixel(FbPoint::new(x, y, depth), self.color.scale(intensity));
            }
        }
    }

    fn scan_lines(
        &self,
    ) -> impl Iterator<Item = ((u32, f32, UnitVec3), (u32, f32, UnitVec3), u32)> {
        let y_range = self.corners[0].point.y..=self.corners[2].point.y;
        let midpoint = self.corners[1].point;

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
            let mut pt_start = current_edge.get(y);
            let mut pt_end = ac_edge.get(y);

            if pt_end.0 < pt_start.0 {
                std::mem::swap(&mut pt_start, &mut pt_end);
            }

            (
                (pt_start.0.round() as u32, pt_start.1, pt_start.2),
                (pt_end.0.round() as u32, pt_end.1, pt_end.2),
                y.round() as u32,
            )
        })
    }
}

struct NormalInterpolator(Interpolator<Vec3>);

impl NormalInterpolator {
    fn from_endpoints(a: (f32, UnitVec3), b: (f32, UnitVec3)) -> Self {
        Self(Interpolator::from_endpoints(
            (a.0, Vec3::from(a.1)),
            (b.0, Vec3::from(b.1)),
        ))
    }

    fn get(&self, x: f32) -> UnitVec3 {
        self.0.get(x).into()
    }
}

struct EdgeWalker {
    start_pt: Vec2,
    x_interp: Interpolator<f32>,
    depth_interp: Interpolator<f32>,
    normal_interp: NormalInterpolator,
}

impl EdgeWalker {
    fn from_corners(a: PhongCorner, b: PhongCorner) -> Self {
        let pt_a = Vec2::new(a.point.x as f32, a.point.y as f32);
        let pt_b = Vec2::new(b.point.x as f32, b.point.y as f32);
        let edge_length = (pt_b - pt_a).length();

        Self {
            start_pt: pt_a,
            normal_interp: NormalInterpolator::from_endpoints(
                (0.0, a.normal),
                (edge_length, b.normal),
            ),
            x_interp: Interpolator::from_endpoints((pt_a.y, pt_a.x), (pt_b.y, pt_b.x)),
            depth_interp: Interpolator::from_endpoints(
                (pt_a.y, a.point.depth),
                (pt_b.y, b.point.depth),
            ),
        }
    }

    fn get(&self, y: f32) -> (f32, f32, UnitVec3) {
        let x = self.x_interp.get(y);
        let depth = self.depth_interp.get(y);

        let current_distance = (Vec2::new(x, y) - self.start_pt).length();
        let normal = self.normal_interp.get(current_distance);
        (x, depth, normal)
    }
}

#[cfg(test)]
mod normal_interpolator_tests {
    use super::NormalInterpolator;
    use crate::geometry::UnitVec3;
    use approx::assert_relative_eq;
    use glam::Vec3;

    #[test]
    fn returns_start_and_end_normals_at_endpoints() {
        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::X), (10.0, UnitVec3::Y));

        assert_relative_eq!(interpolator.get(0.0), UnitVec3::X);
        assert_relative_eq!(interpolator.get(10.0), UnitVec3::Y);
    }

    #[test]
    fn returns_normalized_midpoint_between_endpoints() {
        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::X), (2.0, UnitVec3::Y));

        let mid = interpolator.get(1.0);
        let expected = UnitVec3::from(Vec3::new(1.0, 1.0, 0.0));

        assert_relative_eq!(mid, expected);
    }

    #[test]
    fn interpolates_with_non_zero_start_parameter() {
        let interpolator =
            NormalInterpolator::from_endpoints((2.0, UnitVec3::X), (6.0, UnitVec3::Y));

        let mid = interpolator.get(4.0);
        let expected = UnitVec3::from(Vec3::new(1.0, 1.0, 0.0));

        assert_relative_eq!(mid, expected);
    }

    #[test]
    fn returns_start_normal_when_parameter_span_is_zero() {
        let interpolator =
            NormalInterpolator::from_endpoints((3.0, UnitVec3::X), (3.0, UnitVec3::Y));

        assert_relative_eq!(interpolator.get(3.0), UnitVec3::X);
        assert_relative_eq!(interpolator.get(100.0), UnitVec3::X);
    }

    #[test]
    fn returns_same_normal_when_endpoints_share_normal() {
        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::Z), (8.0, UnitVec3::Z));

        assert_relative_eq!(interpolator.get(4.0), UnitVec3::Z);
    }

    #[test]
    #[should_panic(expected = "UnitVec3 requires a non-zero Vec3")]
    fn panics_when_blend_cancels_out() {
        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::X), (2.0, UnitVec3::NEG_X));

        let _ = interpolator.get(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameBuffer;
    use crate::framebuffer::test_helpers::assert_ascii_art_eq;

    mod draw_triangle_shapes {
        use crate::framebuffer::test_helpers::to_ascii_art;

        use super::*;

        const FULL_BRIGHTNESS: fn(UnitVec3) -> f32 = |_| 1.0;

        #[test]
        fn fill_degenerate_triangle_is_line_segment() {
            let corners = pts([(2, 3), (8, 3), (7, 3)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ███████ ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_degenerate_triangle_two_corners_coincide() {
            let corners = pts([(2, 3), (7, 3), (7, 3)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ██████  ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_degenerate_all_corners_same_point() {
            let corners = pts([(4, 2), (4, 2), (4, 2)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "          ",
                "    █     ",
                "          ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_axis_aligned_shapes_isosceles() {
            let corners = pts([(4, 1), (6, 3), (2, 3)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ███    ",
                "  █████   ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_same_triangle_under_rotated_vertex_order() {
            let corners_ccw = [(4, 1), (6, 3), (2, 3)];
            let expected_result = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ███    ",
                "  █████   ",
                "          ",
            ]);

            for start in 0..3 {
                let corners = pts([
                    corners_ccw[start],
                    corners_ccw[(start + 1) % 3],
                    corners_ccw[(start + 2) % 3],
                ]);
                let mut fb = FrameBuffer::new(10, 5);
                PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
                assert_ascii_art_eq(
                    &fb.to_ascii_art(),
                    &expected_result,
                    &format!("order start {start}"),
                );
            }
        }

        #[test]
        fn draw_right_triangle() {
            let corners = pts([(2, 1), (4, 1), (2, 3)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "  ███     ",
                "  ██      ",
                "  █       ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_clips_when_triangle_extends_past_buffer() {
            let corners = pts([(2, 1), (14, 1), (2, 3)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "  ████████",
                "  ███████ ",
                "  █       ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_slanted_triangle() {
            let corners = pts([(2, 3), (7, 3), (8, 1)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "        █ ",
                "     ████ ",
                "  ██████  ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_triangle_with_vertical_edge() {
            let corners = pts([(2, 1), (2, 4), (6, 2)]);
            let expected_result = to_ascii_art(&[
                "          ",
                "  █       ",
                "  █████   ",
                "  ███     ",
                "  █       ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, FULL_BRIGHTNESS);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        fn pts(corners: [(u32, u32); 3]) -> [PhongCorner; 3] {
            std::array::from_fn(|i| PhongCorner {
                point: FbPoint::new(corners[i].0, corners[i].1, 0.0),
                normal: UnitVec3::Z,
            })
        }
    }

    mod shading_triangles {
        use crate::framebuffer::test_helpers::to_ascii_art;

        use super::*;

        const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;
        const DIFFUSE_INTENSITY: fn(UnitVec3) -> f32 = |normal| TOWARD_LIGHT.dot(normal).max(0.0);

        #[test]
        fn uniform_normal_scales_color_on_horizontal_segment() {
            let corners = pts([
                ((2, 3), UnitVec3::Z),
                ((8, 3), UnitVec3::Z),
                ((7, 3), UnitVec3::Z),
            ]);
            let expected_result = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "  ███████ ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, DIFFUSE_INTENSITY);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        /// Horizontal span: Phong interpolates normals, then shades—differs from Gouraud intensity lerp.
        #[test]
        fn horizontal_span_interpolates_normals_before_shading() {
            let corners = pts([
                ((2, 3), UnitVec3::X),
                ((8, 3), UnitVec3::Z),
                ((7, 3), UnitVec3::from(Vec3::new(1.0, 0.0, 1.0))),
            ]);
            let expected_result = to_ascii_art(&[
                "          ",
                "          ",
                "          ",
                "   ░▒▓███ ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, DIFFUSE_INTENSITY);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn isosceles_interpolates_normals_along_edges_per_scanline() {
            let corners = pts([
                ((4, 1), UnitVec3::Z),
                ((6, 3), UnitVec3::X),
                ((2, 3), UnitVec3::X),
            ]);
            let expected_result = to_ascii_art(&[
                "          ",
                "    █     ",
                "   ▓▓▓    ",
                "          ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, DIFFUSE_INTENSITY);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn fill_clips_when_triangle_extends_past_buffer() {
            let corners = pts([
                ((2, 1), UnitVec3::Z),
                ((14, 1), UnitVec3::X),
                ((2, 3), UnitVec3::X),
            ]);
            let expected_result = to_ascii_art(&[
                "          ",
                "  ██████▓▓",
                "  ▓▓▓▒▒░  ",
                "          ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, DIFFUSE_INTENSITY);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        #[test]
        fn slanted_triangle_interpolates_normals_in_x_and_y() {
            let corners = pts([
                ((2, 3), UnitVec3::X),
                ((7, 3), UnitVec3::Z),
                ((8, 1), UnitVec3::from(Vec3::new(0.0, 1.0, 1.0))),
            ]);
            let expected_result = to_ascii_art(&[
                "          ",
                "        ▓ ",
                "     ▓▓██ ",
                "   ░▓███  ",
                "          ",
            ]);
            let mut fb = FrameBuffer::new(10, 5);
            PhongShadedTriangle::new(corners, Rgb::WHITE).draw(&mut fb, DIFFUSE_INTENSITY);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected_result, "");
        }

        fn pts(corners: [((u32, u32), UnitVec3); 3]) -> [PhongCorner; 3] {
            std::array::from_fn(|i| PhongCorner {
                point: FbPoint::new(corners[i].0.0, corners[i].0.1, 0.0),
                normal: corners[i].1,
            })
        }
    }

    mod occlusion_tests {
        use crate::framebuffer::test_helpers::{assert_ascii_art_eq, to_ascii_art};

        use super::*;

        #[test]
        fn nearer_dimmed_triangle_occludes_further_bright_triangle() {
            let expected = to_ascii_art(&[
                "█                  ▒",
                "███              ▒▒▒",
                "█████           ▒▒▒▒",
                "███████       ▒▒▒▒▒▒",
                "████████    ▒▒▒▒▒▒▒▒",
                "██████████▒▒▒▒▒▒▒▒▒▒",
                "█████████▒▒▒▒▒▒▒▒▒▒▒",
                "███████▒▒▒▒▒▒▒▒▒▒▒▒▒",
                "█████████▒▒▒▒▒▒▒▒▒▒▒",
                "██████████▒▒▒▒▒▒▒▒▒▒",
                "████████    ▒▒▒▒▒▒▒▒",
                "███████       ▒▒▒▒▒▒",
                "█████           ▒▒▒▒",
                "███              ▒▒▒",
                "█                  ▒",
            ]);
            let mut fb = FrameBuffer::new(20, 15);
            PhongShadedTriangle::new(
                [corner(13, 7, 1.0), corner(0, 0, 1.0), corner(0, 14, 1.0)],
                Rgb::WHITE,
            )
            .draw(&mut fb, |_| 1.0);
            PhongShadedTriangle::new(
                [corner(7, 7, 0.0), corner(19, 0, 0.0), corner(19, 14, 0.0)],
                Rgb::WHITE,
            )
            .draw(&mut fb, |_| 0.25);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        #[test]
        fn dim_triangle_pierces_bright_triangle() {
            let expected = to_ascii_art(&[
                "█                  ▒",
                "██                ▒▒",
                "████             ▒▒▒",
                "█████           ▒▒▒▒",
                "██████         ▒▒▒▒▒",
                "████████      ▒▒▒▒▒▒",
                "█████████    ▒▒▒▒▒▒▒",
                "███████████ ▒▒▒▒▒▒▒▒",
                "████████████▒▒▒▒▒▒▒▒",
                "█████████████▒▒▒▒▒▒▒",
                "█████████▒█████▒▒▒▒▒",
                "████████▒▒██████▒▒▒▒",
                "███████▒▒▒███████▒▒▒",
                "██████▒▒▒▒█████████▒",
                "█████▒▒▒▒▒██████████",
            ]);
            let mut fb = FrameBuffer::new(20, 15);
            PhongShadedTriangle::new(
                [corner(0, 0, 1.0), corner(19, 14, 1.0), corner(0, 14, 1.0)],
                Rgb::WHITE,
            )
            .draw(&mut fb, |_| 1.0);
            PhongShadedTriangle::new(
                [corner(19, 0, 2.0), corner(0, 19, 0.0), corner(19, 19, 2.0)],
                Rgb::WHITE,
            )
            .draw(&mut fb, |_| 0.25);
            assert_ascii_art_eq(&fb.to_ascii_art(), &expected, "");
        }

        fn corner(x: u32, y: u32, depth: f32) -> PhongCorner {
            PhongCorner {
                point: FbPoint::new(x, y, depth),
                normal: UnitVec3::Z,
            }
        }
    }
}
