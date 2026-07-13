use glam::Vec3;

use crate::geometry::{SurfacePoint, UnitVec3};

pub struct Light {
    light_type: LightType,
    intensity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DistanceFalloff {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

enum LightType {
    Directional {
        toward_light: UnitVec3,
    },
    Point {
        position: Vec3,
        falloff: DistanceFalloff,
    },
}

impl DistanceFalloff {
    pub const NONE: Self = Self {
        constant: 1.0,
        linear: 0.0,
        quadratic: 0.0,
    };

    fn calculate(&self, distance: f32) -> f32 {
        (self.constant + self.linear * distance + self.quadratic * distance * distance).recip()
    }
}

impl LightType {
    // A helper method to calculate the unit vector toward the light and the falloff factor
    // for a given point position
    fn factors(&self, point_pos: Vec3) -> (UnitVec3, f32) {
        match self {
            LightType::Directional { toward_light } => (*toward_light, 1.0),
            LightType::Point { position, falloff } => {
                let toward_light = position - point_pos;
                let falloff = falloff.calculate(toward_light.length());
                (toward_light.into(), falloff)
            }
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

    pub const fn point(position: Vec3, intensity: f32, falloff: DistanceFalloff) -> Self {
        Self {
            light_type: LightType::Point { position, falloff },
            intensity,
        }
    }

    pub fn diffuse_contrib(&self, point: SurfacePoint) -> f32 {
        let (toward_light, falloff) = self.light_type.factors(point.position());
        self.intensity * falloff * toward_light.dot(point.normal()).max(0.0)
    }

    pub fn specular_contrib(
        &self,
        shininess: i32,
        point: SurfacePoint,
        toward_eye: UnitVec3,
    ) -> f32 {
        let (toward_light, falloff) = self.light_type.factors(point.position());

        let half_vector = (toward_light + toward_eye).normalize();
        self.intensity * falloff * point.normal().dot(half_vector).max(0.0).powi(shininess)
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
    const POINT_LIGHT: Light = Light::point(LIGHT_POS, 2.0, DistanceFalloff::NONE);

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
            let point_light = Light::point(LIGHT_POS, 0.0, DistanceFalloff::NONE);

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
        fn diffuse_contrib_changes_with_distance() {
            let falloff = DistanceFalloff {
                constant: 1.0,
                linear: 0.0,
                quadratic: 1.0,
            };
            let light_with_falloff = Light::point(Vec3::new(0.0, 1.0, 0.0), 1.0, falloff);

            let unit_distance = SurfacePoint::new(Vec3::ZERO, UnitVec3::Y);
            let further_away = SurfacePoint::new(Vec3::new(0.0, -1.0, 0.0), UnitVec3::Y);

            assert_relative_eq!(0.5, light_with_falloff.diffuse_contrib(unit_distance));
            assert_relative_eq!(0.2, light_with_falloff.diffuse_contrib(further_away));
        }

        #[test]
        fn specular_contrib_changes_with_distance() {
            let falloff = DistanceFalloff {
                constant: 1.0,
                linear: 0.0,
                quadratic: 1.0,
            };
            let light_with_falloff = Light::point(Vec3::new(0.0, 1.0, 0.0), 1.0, falloff);

            let unit_distance = SurfacePoint::new(Vec3::ZERO, UnitVec3::Y);
            let further_away = SurfacePoint::new(Vec3::new(0.0, -1.0, 0.0), UnitVec3::Y);

            let toward_eye = UnitVec3::Y;
            assert_relative_eq!(
                0.5,
                light_with_falloff.specular_contrib(1, unit_distance, toward_eye)
            );
            assert_relative_eq!(
                0.2,
                light_with_falloff.specular_contrib(1, further_away, toward_eye)
            );
        }
    }
}
