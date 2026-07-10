use glam::Vec3;

use crate::geometry::{SurfacePoint, UnitVec3};

enum LightType {
    Directional { toward_light: UnitVec3 },
    Point { position: Vec3 },
}

pub struct Light {
    light_type: LightType,
    intensity: f32,
}

impl Light {
    pub fn directional(toward_light: UnitVec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional { toward_light },
            intensity,
        }
    }

    pub fn point(position: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Point { position },
            intensity,
        }
    }

    pub fn diffuse_contrib(&self, point: SurfacePoint) -> f32 {
        self.intensity * self.toward_light(point).dot(point.normal()).max(0.0)
    }

    pub fn specular_contrib(
        &self,
        shininess: i32,
        point: SurfacePoint,
        toward_eye: UnitVec3,
    ) -> f32 {
        let half_vector = (self.toward_light(point) + toward_eye).normalize();
        self.intensity * point.normal().dot(half_vector).max(0.0).powi(shininess)
    }

    fn toward_light(&self, point: SurfacePoint) -> UnitVec3 {
        match self.light_type {
            LightType::Directional { toward_light } => toward_light,
            LightType::Point { position } => (position - point.position()).into(),
        }
    }
}

#[cfg(test)]
mod directional_light_tests {
    use super::Light;
    use crate::geometry::{SurfacePoint, UnitVec3};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;

    mod diffuse_contrib {
        use super::*;

        #[test]
        fn fully_lit_when_normal_aligns_with_light() {
            let light = Light::directional(UnitVec3::Z, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.diffuse_contrib(point), 1.0);

            let scaled = Light::directional(Vec3::new(0.0, 0.0, 3.0).into(), 1.0);
            assert_relative_eq!(scaled.diffuse_contrib(point), 1.0);
        }

        #[test]
        fn zero_when_normal_does_not_face_light() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt1 = SurfacePoint::new(Vec3::ZERO, UnitVec3::X);
            let pt2 = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);

            assert_relative_eq!(light.diffuse_contrib(pt1), 0.0);
            assert_relative_eq!(light.diffuse_contrib(pt2), 0.0);
        }

        #[test]
        fn oblique_light_produces_known_contrib() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
            let light = Light::directional(toward_light, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.diffuse_contrib(pt), 2.0_f32.sqrt().recip());
        }

        #[test]
        fn intensity_scales_diffuse_contrib() {
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let triple = Light::directional(TOWARD_LIGHT, 3.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(triple.diffuse_contrib(pt), 3.0 * unit.diffuse_contrib(pt));
        }

        #[test]
        fn zero_intensity_suppresses_diffuse_contrib() {
            let light = Light::directional(TOWARD_LIGHT, 0.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.diffuse_contrib(pt), 0.0);
        }
    }

    mod specular_contrib {
        use super::*;

        #[test]
        fn one_when_view_aligns_with_light() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(1, pt, UnitVec3::Z), 1.0);
        }

        #[test]
        fn weaker_when_view_is_tangent() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            let aligned = light.specular_contrib(1, pt, UnitVec3::Z);
            let tangent = light.specular_contrib(1, pt, UnitVec3::Y);
            assert_relative_eq!(tangent, 2.0_f32.sqrt().recip());
            assert!(tangent < aligned);
        }

        #[test]
        fn zero_when_light_and_view_cancel() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.specular_contrib(1, pt, -TOWARD_LIGHT), 0.0);
        }

        #[test]
        fn higher_shininess_tightens_highlight() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let low = light.specular_contrib(1, pt, UnitVec3::Y);
            let high = light.specular_contrib(2, pt, UnitVec3::Y);
            assert_relative_eq!(high, 0.5);
            assert!(high < low);
        }

        #[test]
        fn high_shininess_suppresses_off_axis() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(128, pt, UnitVec3::Y), 0.0);
        }

        #[test]
        fn zero_shininess_yields_broad_highlight() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.specular_contrib(0, pt, UnitVec3::Z), 1.0);
            assert_relative_eq!(light.specular_contrib(0, pt, UnitVec3::Y), 1.0);
        }

        #[test]
        fn zero_when_half_vector_faces_away_from_normal() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, -1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
            let light = Light::directional(toward_light, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.specular_contrib(2, pt, toward_eye), 0.0);
        }

        #[test]
        fn oblique_light_and_view_produce_known_contrib() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
            let light = Light::directional(toward_light, 1.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.specular_contrib(1, pt, toward_eye), 1.0);
        }

        #[test]
        fn intensity_scales_specular_contrib() {
            let toward_eye = UnitVec3::Y;
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let triple = Light::directional(TOWARD_LIGHT, 3.0);
            let pt = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(
                triple.specular_contrib(1, pt, toward_eye),
                3.0 * unit.specular_contrib(1, pt, toward_eye)
            );
        }
    }
}

#[cfg(test)]
mod point_light_tests {
    use super::Light;
    use crate::geometry::{SurfacePoint, UnitVec3};
    use approx::assert_relative_eq;
    use glam::Vec3;

    mod diffuse_contrib {
        use super::*;

        #[test]
        fn fully_lit_when_normal_aligns_with_toward_light() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_relative_eq!(light.diffuse_contrib(point), 1.0);
        }

        #[test]
        fn zero_when_normal_does_not_face_light() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let facing_x = SurfacePoint::new(Vec3::ZERO, UnitVec3::X);
            let facing_away = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);

            assert_relative_eq!(light.diffuse_contrib(facing_x), 0.0);
            assert_relative_eq!(light.diffuse_contrib(facing_away), 0.0);
        }

        #[test]
        fn oblique_geometry_produces_known_contrib() {
            let light = Light::point(Vec3::new(2.0, 0.0, 2.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.diffuse_contrib(point), 2.0_f32.sqrt().recip());
        }

        #[test]
        fn surface_position_shifts_toward_light() {
            let light = Light::point(Vec3::new(2.0, 0.0, 0.0), 1.0);
            let offset = SurfacePoint::new(Vec3::new(1.0, 0.0, 0.0), UnitVec3::X);

            assert_relative_eq!(light.diffuse_contrib(offset), 1.0);
        }

        #[test]
        fn matches_directional_when_displacement_equals_direction() {
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let point_light = Light::point(Vec3::new(0.0, 0.0, 3.0), 1.0);
            let directional = Light::directional(UnitVec3::Z, 1.0);

            assert_relative_eq!(
                point_light.diffuse_contrib(point),
                directional.diffuse_contrib(point)
            );
        }

        #[test]
        fn intensity_scales_diffuse_contrib() {
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let unit = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let triple = Light::point(Vec3::new(0.0, 0.0, 1.0), 3.0);

            assert_relative_eq!(
                triple.diffuse_contrib(point),
                3.0 * unit.diffuse_contrib(point)
            );
        }

        #[test]
        fn zero_intensity_suppresses_diffuse_contrib() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 0.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.diffuse_contrib(point), 0.0);
        }

        #[test]
        fn surface_position_changes_diffuse_contrib() {
            let light = Light::point(Vec3::new(0.0, 0.0, 2.0), 1.0);
            let centered = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let shifted = SurfacePoint::new(Vec3::new(1.0, 0.0, 0.0), UnitVec3::Z);

            assert_relative_eq!(light.diffuse_contrib(centered), 1.0);
            assert_relative_eq!(light.diffuse_contrib(shifted), 2.0 / 5.0_f32.sqrt());
        }
    }

    mod specular_contrib {
        use super::*;

        #[test]
        fn one_when_view_aligns_with_toward_light() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(1, point, UnitVec3::Z), 1.0);
        }

        #[test]
        fn weaker_when_view_is_tangent() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            let aligned = light.specular_contrib(1, point, UnitVec3::Z);
            let tangent = light.specular_contrib(1, point, UnitVec3::Y);
            assert_relative_eq!(tangent, 2.0_f32.sqrt().recip());
            assert!(tangent < aligned);
        }

        #[test]
        fn zero_when_light_and_view_cancel() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(1, point, UnitVec3::NEG_Z), 0.0);
        }

        #[test]
        fn higher_shininess_tightens_highlight() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let low = light.specular_contrib(1, point, UnitVec3::Y);
            let high = light.specular_contrib(2, point, UnitVec3::Y);

            assert_relative_eq!(high, 0.5);
            assert!(high < low);
        }

        #[test]
        fn high_shininess_suppresses_off_axis() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(128, point, UnitVec3::Y), 0.0);
        }

        #[test]
        fn zero_shininess_yields_broad_highlight() {
            let light = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(0, point, UnitVec3::Z), 1.0);
            assert_relative_eq!(light.specular_contrib(0, point, UnitVec3::Y), 1.0);
        }

        #[test]
        fn zero_when_half_vector_faces_away_from_normal() {
            let light = Light::point(Vec3::new(1.0, 0.0, -1.0), 1.0);
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(2, point, toward_eye), 0.0);
        }

        #[test]
        fn oblique_light_and_view_produce_known_contrib() {
            let light = Light::point(Vec3::new(2.0, 0.0, 2.0), 1.0);
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_relative_eq!(light.specular_contrib(1, point, toward_eye), 1.0);
        }

        #[test]
        fn intensity_scales_specular_contrib() {
            let toward_eye = UnitVec3::Y;
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let unit = Light::point(Vec3::new(0.0, 0.0, 1.0), 1.0);
            let triple = Light::point(Vec3::new(0.0, 0.0, 1.0), 3.0);

            assert_relative_eq!(
                triple.specular_contrib(1, point, toward_eye),
                3.0 * unit.specular_contrib(1, point, toward_eye)
            );
        }

        #[test]
        fn surface_position_changes_specular_contrib() {
            let light = Light::point(Vec3::new(0.0, 0.0, 2.0), 1.0);
            let centered = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let shifted = SurfacePoint::new(Vec3::new(1.0, 0.0, 0.0), UnitVec3::Z);
            let toward_eye = UnitVec3::Y;

            let centered_contrib = light.specular_contrib(1, centered, toward_eye);
            let shifted_contrib = light.specular_contrib(1, shifted, toward_eye);

            assert_relative_eq!(centered_contrib, 2.0_f32.sqrt().recip());
            assert!(shifted_contrib < centered_contrib);
        }
    }

    mod toward_light {
        use super::*;

        #[test]
        #[should_panic(expected = "UnitVec3 requires a non-zero Vec3")]
        fn panics_when_light_coincides_with_surface() {
            let light = Light::point(Vec3::new(1.0, 2.0, 3.0), 1.0);
            let point = SurfacePoint::new(Vec3::new(1.0, 2.0, 3.0), UnitVec3::Z);

            let _ = light.diffuse_contrib(point);
        }
    }
}
