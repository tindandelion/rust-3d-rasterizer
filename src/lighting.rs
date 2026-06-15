use crate::framebuffer::Rgb;
use crate::geometry::UnitVec3;

/// Surface color and shading coefficients: **ambient** floor and **diffuse** scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub color: Rgb,
    ambient_factor: f32,
    diffuse_factor: f32,
    shininess: Option<i32>,
}

impl Material {
    /// **`ambient_factor`** plus **`1.0 − ambient_factor`** diffuse (both clamped in [`new`]).
    pub const fn matte(color: Rgb, ambient_factor: f32) -> Self {
        Self::new(color, ambient_factor, 1.0 - ambient_factor, None)
    }

    pub const fn shiny(color: Rgb, ambient_factor: f32, shininess: i32) -> Self {
        Self::new(color, ambient_factor, 1.0 - ambient_factor, Some(shininess))
    }

    const fn new(
        color: Rgb,
        ambient_factor: f32,
        diffuse_factor: f32,
        shininess: Option<i32>,
    ) -> Self {
        Self {
            color,
            ambient_factor: ambient_factor.max(0.0),
            diffuse_factor: diffuse_factor.max(0.0),
            shininess,
        }
    }

    pub fn shade(
        &self,
        light_model: &BlinnLightModel,
        normal: UnitVec3,
        toward_eye: UnitVec3,
    ) -> Rgb {
        let (diffuse, specular) =
            light_model.calc_intensity_contrib(self.shininess, normal, toward_eye);
        let emissive = self.color * self.ambient_factor;
        let diffuse = self.color * (self.diffuse_factor * diffuse);
        let specular = self.color * (self.diffuse_factor * specular);
        emissive + diffuse + specular
    }
}

pub struct BlinnLightModel {
    toward_light: UnitVec3,
}

impl BlinnLightModel {
    pub fn new(toward_light: UnitVec3) -> Self {
        Self { toward_light }
    }

    pub fn calc_intensity_contrib(
        &self,
        shininess: Option<i32>,
        normal: UnitVec3,
        toward_eye: UnitVec3,
    ) -> (f32, f32) {
        let diffuse = self.toward_light.dot(normal).max(0.0);
        let specular = shininess
            .map(|shininess| {
                let half_vector = (self.toward_light + toward_eye).normalize();
                normal.dot(half_vector).max(0.0).powi(shininess)
            })
            .unwrap_or(0.0);
        (diffuse, specular)
    }
}

#[cfg(test)]
mod tests {
    use crate::framebuffer::Rgb;
    use crate::geometry::UnitVec3;

    use super::{BlinnLightModel, Material};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;

    mod calc_intensity_contrib {
        use super::*;

        #[test]
        fn diffuse_fully_lit_when_normal_aligns_with_light() {
            let light = BlinnLightModel::new(UnitVec3::Z);
            let normal = UnitVec3::Z;
            let toward_eye = UnitVec3::Z;
            let (diffuse, specular) = light.calc_intensity_contrib(None, normal, toward_eye);
            assert_relative_eq!(diffuse, 1.0);
            assert_relative_eq!(specular, 0.0);

            let scaled = BlinnLightModel::new(Vec3::new(0.0, 0.0, 3.0).into());
            let (diffuse, specular) = scaled.calc_intensity_contrib(None, normal, toward_eye);
            assert_relative_eq!(diffuse, 1.0);
            assert_relative_eq!(specular, 0.0);
        }

        #[test]
        fn diffuse_zero_when_normal_does_not_face_light() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let toward_eye = UnitVec3::Z;

            let perp = light.calc_intensity_contrib(None, UnitVec3::X, toward_eye);
            assert_relative_eq!(perp.0, 0.0);
            assert_relative_eq!(perp.1, 0.0);

            let away = light.calc_intensity_contrib(None, UnitVec3::NEG_Z, toward_eye);
            assert_relative_eq!(away.0, 0.0);
            assert_relative_eq!(away.1, 0.0);
        }

        #[test]
        fn matte_specular_is_zero_and_view_independent() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let from_front = light.calc_intensity_contrib(None, UnitVec3::Z, UnitVec3::Z);
            let from_side = light.calc_intensity_contrib(None, UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(from_front.0, from_side.0);
            assert_relative_eq!(from_front.1, 0.0);
            assert_relative_eq!(from_side.1, 0.0);
        }

        #[test]
        fn specular_when_view_aligns_with_light() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let (diffuse, specular) =
                light.calc_intensity_contrib(Some(1), UnitVec3::Z, UnitVec3::Z);
            assert_relative_eq!(diffuse, 1.0);
            assert_relative_eq!(specular, 1.0);
        }

        #[test]
        fn specular_weaker_when_view_is_tangent() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let aligned = light.calc_intensity_contrib(Some(1), UnitVec3::Z, UnitVec3::Z);
            let tangent = light.calc_intensity_contrib(Some(1), UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(tangent.1, 2.0_f32.sqrt().recip());
            assert!(tangent.1 < aligned.1);
        }

        #[test]
        fn specular_skipped_when_light_and_view_cancel() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let (diffuse, specular) =
                light.calc_intensity_contrib(Some(1), UnitVec3::Z, -TOWARD_LIGHT);
            assert_relative_eq!(diffuse, 1.0);
            assert_relative_eq!(specular, 0.0);
        }

        #[test]
        fn higher_shininess_tightens_specular_highlight() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let low = light.calc_intensity_contrib(Some(1), UnitVec3::Z, UnitVec3::Y);
            let high = light.calc_intensity_contrib(Some(2), UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(high.1, 0.5);
            assert!(high.1 < low.1);
        }

        #[test]
        fn high_shininess_suppresses_off_axis_specular() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let (diffuse, specular) =
                light.calc_intensity_contrib(Some(128), UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(diffuse, 1.0);
            assert_relative_eq!(specular, 0.0);
        }

        #[test]
        fn zero_shininess_yields_broad_specular_highlight() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let aligned = light.calc_intensity_contrib(Some(0), UnitVec3::Z, UnitVec3::Z);
            let tangent = light.calc_intensity_contrib(Some(0), UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(aligned.0, 1.0);
            assert_relative_eq!(aligned.1, 1.0);
            assert_relative_eq!(tangent.0, 1.0);
            assert_relative_eq!(tangent.1, 1.0);
        }

        #[test]
        fn specular_zero_when_half_vector_faces_away_from_normal() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, -1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
            let light = BlinnLightModel::new(toward_light);
            let (diffuse, specular) =
                light.calc_intensity_contrib(Some(2), UnitVec3::Z, toward_eye);
            assert_relative_eq!(diffuse, 0.0);
            assert_relative_eq!(specular, 0.0);
        }

        #[test]
        fn oblique_light_and_view_produce_known_contrib() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
            let light = BlinnLightModel::new(toward_light);
            let (diffuse, specular) =
                light.calc_intensity_contrib(Some(1), UnitVec3::Z, toward_eye);
            assert_relative_eq!(diffuse, 2.0_f32.sqrt().recip());
            assert_relative_eq!(specular, 1.0);
        }
    }

    mod material_shade {
        use super::*;

        #[test]
        fn full_ambient_ignores_directional_contrib() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let material = Material::matte(Rgb::WHITE, 1.0);
            let normal = UnitVec3::NEG_Z;
            assert_eq!(material.shade(&light, normal, -normal), Rgb::WHITE);
        }

        #[test]
        fn half_ambient_blends_directional_contrib() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let material = Material::matte(Rgb::WHITE, 0.5);
            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                Rgb::WHITE * 1.0
            );
            assert_eq!(
                material.shade(&light, UnitVec3::X, UnitVec3::Z),
                Rgb::WHITE * 0.5
            );
            assert_eq!(
                material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z),
                Rgb::WHITE * 0.5
            );
        }

        #[test]
        fn shiny_brightens_tangent_view_over_matte() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let normal = UnitVec3::Z;
            let toward_eye = UnitVec3::Y;
            let base = Rgb(128, 128, 128);
            let matte = Material::matte(base, 0.0);
            let shiny = Material::shiny(base, 0.0, 1);
            assert_eq!(matte.shade(&light, normal, toward_eye), base);
            assert_eq!(
                shiny.shade(&light, normal, toward_eye),
                base * (1.0 + 2.0_f32.sqrt().recip())
            );
        }

        #[test]
        fn half_ambient_scales_specular_on_aligned_view() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let material = Material::shiny(Rgb::WHITE, 0.5, 1);
            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                Rgb::WHITE * 1.5
            );
        }

        #[test]
        fn shiny_falls_back_to_ambient_on_unlit_surface() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let material = Material::shiny(Rgb::WHITE, 0.5, 1);
            assert_eq!(
                material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z),
                Rgb::WHITE * 0.5
            );
        }
    }
}
