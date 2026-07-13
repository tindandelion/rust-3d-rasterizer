use glam::Vec3;

use crate::geometry::{SurfacePoint, UnitVec3};

pub struct Light {
    light_type: LightType,
    intensity: f32,
}

enum LightType {
    Directional { toward_light: UnitVec3 },
    Point { position: Vec3 },
}

impl LightType {
    fn toward_light(&self, point_pos: Vec3) -> UnitVec3 {
        match self {
            LightType::Directional { toward_light } => *toward_light,
            LightType::Point { position } => (position - point_pos).into(),
        }
    }
}

impl Light {
    pub const fn directional(toward_light: UnitVec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional { toward_light },
            intensity,
        }
    }

    pub const fn point(position: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Point { position },
            intensity,
        }
    }

    pub fn diffuse_contrib(&self, point: SurfacePoint) -> f32 {
        self.intensity
            * self
                .light_type
                .toward_light(point.position())
                .dot(point.normal())
                .max(0.0)
    }

    pub fn specular_contrib(
        &self,
        shininess: i32,
        point: SurfacePoint,
        toward_eye: UnitVec3,
    ) -> f32 {
        let half_vector = (self.light_type.toward_light(point.position()) + toward_eye).normalize();
        self.intensity * point.normal().dot(half_vector).max(0.0).powi(shininess)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // Common tests for both directional and point lights, their contribution to the illumination to the fixed surface point
    // Test setup:
    // Surface point is fixed at (0, 0, 0)
    // Light is located above:
    // - Directional light: toward_light = (0, 1, 0)
    // - Point light: position = (0, 1, 0)

    const SURFACE_POS: Vec3 = Vec3::ZERO;
    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Y;
    const LIGHT_POS: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    const DIRECTIONAL_LIGHT: Light = Light::directional(TOWARD_LIGHT, 3.0);
    const POINT_LIGHT: Light = Light::point(LIGHT_POS, 2.0);

    mod diffuse_contrib {
        use super::*;

        #[test]
        fn max_illumination_when_normal_aligns_with_toward_light() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);
            assert_relative_eq!(3.0, DIRECTIONAL_LIGHT.diffuse_contrib(surface_point));
            assert_relative_eq!(2.0, POINT_LIGHT.diffuse_contrib(surface_point));
        }

        #[test]
        fn zero_intensity_suppresses_diffuse_contrib() {
            let directional_light = Light::directional(TOWARD_LIGHT, 0.0);
            let point_light = Light::point(LIGHT_POS, 0.0);

            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);
            assert_relative_eq!(0.0, directional_light.diffuse_contrib(surface_point));
            assert_relative_eq!(0.0, point_light.diffuse_contrib(surface_point));
        }

        #[test]
        fn zero_intensity_when_normal_points_away_from_light() {
            let surface_point = SurfacePoint::new(SURFACE_POS, -TOWARD_LIGHT);
            assert_relative_eq!(0.0, DIRECTIONAL_LIGHT.diffuse_contrib(surface_point));
            assert_relative_eq!(0.0, POINT_LIGHT.diffuse_contrib(surface_point));
        }

        #[test]
        fn zero_intensity_when_normal_is_orthogonal_to_light() {
            let normal = UnitVec3::X;
            let surface_point = SurfacePoint::new(SURFACE_POS, normal);

            assert_relative_eq!(0.0, DIRECTIONAL_LIGHT.diffuse_contrib(surface_point));
            assert_relative_eq!(0.0, POINT_LIGHT.diffuse_contrib(surface_point));
        }

        #[test]
        fn weaker_intensity_when_normal_is_oblique_to_light() {
            let normal = UnitVec3::from(Vec3::new(1.0, 1.0, 0.0));
            let surface_point = SurfacePoint::new(SURFACE_POS, normal);

            let contrib = 2.0_f32.sqrt().recip();
            assert_relative_eq!(
                3.0 * contrib,
                DIRECTIONAL_LIGHT.diffuse_contrib(surface_point)
            );
            assert_relative_eq!(2.0 * contrib, POINT_LIGHT.diffuse_contrib(surface_point));
        }
    }

    mod specular_contrib {
        use super::*;

        const SHININESS: i32 = 1;

        #[test]
        fn full_intensity_when_view_direction_aligns_with_toward_light() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);

            let toward_eye = TOWARD_LIGHT;
            assert_relative_eq!(
                3.0,
                DIRECTIONAL_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
            assert_relative_eq!(
                2.0,
                POINT_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
        }

        #[test]
        fn blinn_phong_non_zero_when_view_direction_grazes_the_surface() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);

            let toward_eye = UnitVec3::from(Vec3::new(1.0, 0.0, 0.0));
            assert_relative_eq!(
                3.0 * 2.0_f32.sqrt().recip(),
                DIRECTIONAL_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
            assert_relative_eq!(
                2.0 * 2.0_f32.sqrt().recip(),
                POINT_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
        }

        #[test]
        fn blinn_phong_non_zero_when_view_direction_faces_away_from_light() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);

            let toward_eye = UnitVec3::from(Vec3::new(1.0, -0.1, 0.0));
            assert!(DIRECTIONAL_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye) > 0.0);
            assert!(POINT_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye) > 0.0);
        }

        #[test]
        fn blinn_phong_zero_when_view_direction_is_strictly_away_from_light() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);

            let toward_eye = -TOWARD_LIGHT;
            assert_relative_eq!(
                0.0,
                DIRECTIONAL_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
            assert_relative_eq!(
                0.0,
                POINT_LIGHT.specular_contrib(SHININESS, surface_point, toward_eye)
            );
        }

        #[test]
        fn high_shininess_tightens_highlight() {
            let surface_point = SurfacePoint::new(SURFACE_POS, TOWARD_LIGHT);

            let toward_eye = UnitVec3::from(Vec3::new(1.0, 0.0, 0.0));
            let high_shininess = 128;
            assert_relative_eq!(
                0.0,
                DIRECTIONAL_LIGHT.specular_contrib(high_shininess, surface_point, toward_eye)
            );
            assert_relative_eq!(
                0.0,
                POINT_LIGHT.specular_contrib(high_shininess, surface_point, toward_eye)
            );
        }
    }

    mod point_light_tests {
        use super::*;

        #[test]
        fn diffuse_contrib_changes_with_surface_position() {
            let centered = SurfacePoint::new(Vec3::ZERO, UnitVec3::Y);
            let shifted = SurfacePoint::new(Vec3::new(1.0, 0.0, 0.0), UnitVec3::Y);

            assert_relative_eq!(2.0, POINT_LIGHT.diffuse_contrib(centered));
            assert_relative_eq!(2.0_f32.sqrt(), POINT_LIGHT.diffuse_contrib(shifted));
        }

        #[test]
        fn specular_contrib_changes_with_surface_position() {
            let centered = SurfacePoint::new(Vec3::ZERO, UnitVec3::Y);
            let shifted = SurfacePoint::new(Vec3::new(1.0, 0.0, 0.0), UnitVec3::Y);

            let toward_eye = UnitVec3::from(Vec3::new(1.0, 1.0, 0.0));
            assert_relative_eq!(
                1.847759,
                POINT_LIGHT.specular_contrib(1, centered, toward_eye)
            );
            assert_relative_eq!(2.0, POINT_LIGHT.specular_contrib(1, shifted, toward_eye));
        }

        #[test]
        #[should_panic(expected = "UnitVec3 requires a non-zero Vec3")]
        fn panics_when_light_coincides_with_surface() {
            let light = Light::point(Vec3::new(1.0, 2.0, 3.0), 1.0);
            let point = SurfacePoint::new(Vec3::new(1.0, 2.0, 3.0), UnitVec3::Z);

            let _ = light.diffuse_contrib(point);
        }
    }
}
