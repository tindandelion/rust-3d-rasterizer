use crate::framebuffer::Rgb;
use crate::geometry::UnitVec3;

/// Surface **emissive** and pre-scaled **diffuse** colors for directional shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub emissive: Rgb,
    pub diffuse: Rgb,
    shininess: Option<i32>,
}

impl Material {
    /// **`shininess`** enables specular when **`Some`**.
    pub const fn new(emissive: Rgb, diffuse: Rgb, shininess: Option<i32>) -> Self {
        Self {
            emissive,
            diffuse,
            shininess,
        }
    }

    pub fn shade(
        &self,
        light_model: &BlinnLightModel,
        normal: UnitVec3,
        toward_eye: UnitVec3,
    ) -> Rgb {
        let (diffuse_contrib, specular_contrib) =
            light_model.calc_intensity_contrib(self.shininess, normal, toward_eye);
        let diffuse = self.diffuse * diffuse_contrib;
        let specular = self.diffuse * specular_contrib;
        self.emissive + diffuse + specular
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
        fn specular_is_zero_without_shininess() {
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
        fn emissive_only_ignores_light_direction() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let emissive = Rgb(20, 40, 60);
            let material = Material::new(emissive, Rgb::BLACK, None);

            for normal in [UnitVec3::Z, UnitVec3::X, UnitVec3::NEG_Z] {
                assert_eq!(material.shade(&light, normal, UnitVec3::Z), emissive);
            }
        }

        #[test]
        fn diffuse_scales_with_light_facing() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(100, 110, 120);
            let material = Material::new(emissive, diffuse, None);

            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                emissive + diffuse
            );
            assert_eq!(material.shade(&light, UnitVec3::X, UnitVec3::Z), emissive);
            assert_eq!(material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z), emissive);
        }

        #[test]
        fn specular_adds_on_top_of_diffuse_for_shiny_material() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let diffuse = Rgb(128, 128, 128);
            let diffuse_only = Material::new(Rgb::BLACK, diffuse, None);
            let shiny = Material::new(Rgb::BLACK, diffuse, Some(1));
            let normal = UnitVec3::Z;
            let toward_eye = UnitVec3::Y;

            assert_eq!(diffuse_only.shade(&light, normal, toward_eye), diffuse);
            assert_eq!(
                shiny.shade(&light, normal, toward_eye),
                diffuse + diffuse * 2.0_f32.sqrt().recip()
            );
        }

        #[test]
        fn aligned_view_sums_emissive_diffuse_and_specular() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let emissive = Rgb(30, 30, 30);
            let diffuse = Rgb(40, 40, 40);
            let material = Material::new(emissive, diffuse, Some(1));

            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                emissive + diffuse + diffuse
            );
        }

        #[test]
        fn unlit_shiny_surface_returns_emissive_only() {
            let light = BlinnLightModel::new(TOWARD_LIGHT);
            let emissive = Rgb(128, 128, 128);
            let material = Material::new(emissive, Rgb(64, 64, 64), Some(1));

            assert_eq!(
                material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z),
                emissive
            );
        }
    }
}
