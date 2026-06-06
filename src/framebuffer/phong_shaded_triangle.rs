use glam::Vec3;

use crate::geometry::UnitVec3;

use super::interpolator::Interpolator;

struct NormalInterpolator(Interpolator<Vec3>);

impl NormalInterpolator {
    pub fn from_endpoints(a: (f32, UnitVec3), b: (f32, UnitVec3)) -> Self {
        Self(Interpolator::from_endpoints(
            (a.0, Vec3::from(a.1)),
            (b.0, Vec3::from(b.1)),
        ))
    }

    pub fn get(&self, x: f32) -> UnitVec3 {
        self.0.get(x).into()
    }
}

#[cfg(test)]
mod normal_interpolator_tests {
    use super::NormalInterpolator;
    use crate::geometry::UnitVec3;
    use approx::assert_relative_eq;

    #[test]
    fn returns_start_and_end_normals_at_endpoints() {
        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::X), (10.0, UnitVec3::Y));

        assert_relative_eq!(interpolator.get(0.0), UnitVec3::X);
        assert_relative_eq!(interpolator.get(10.0), UnitVec3::Y);
    }

    #[test]
    fn returns_normalized_midpoint_between_endpoints() {
        use glam::Vec3;

        let interpolator =
            NormalInterpolator::from_endpoints((0.0, UnitVec3::X), (2.0, UnitVec3::Y));

        let mid = interpolator.get(1.0);
        let expected = UnitVec3::from(Vec3::new(1.0, 1.0, 0.0));

        assert_relative_eq!(mid, expected);
    }

    #[test]
    fn interpolates_with_non_zero_start_parameter() {
        use glam::Vec3;

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
