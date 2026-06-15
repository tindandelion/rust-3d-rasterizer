use crate::framebuffer::Rgb;
use crate::geometry::UnitVec3;

/// Surface **emissive**, pre-scaled **diffuse**, and **specular** colors for Phong shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub emissive: Rgb,
    pub diffuse: Rgb,
    pub specular: Rgb,
    shininess: Option<i32>,
}

impl Material {
    /// **`shininess`** enables specular when **`Some`**.
    pub const fn new(emissive: Rgb, diffuse: Rgb, specular: Rgb, shininess: Option<i32>) -> Self {
        Self {
            emissive,
            diffuse,
            specular,
            shininess,
        }
    }

    pub fn shade(&self, light: &DirectionalLight, normal: UnitVec3, toward_eye: UnitVec3) -> Rgb {
        let specular_contrib = self.shininess.map_or(0.0, |shininess| {
            light.specular_contrib(shininess, normal, toward_eye)
        });
        self.emissive
            + self.diffuse * light.diffuse_contrib(normal)
            + self.specular * specular_contrib
    }
}

pub struct DirectionalLight {
    toward_light: UnitVec3,
}

impl DirectionalLight {
    pub fn new(toward_light: UnitVec3) -> Self {
        Self { toward_light }
    }

    pub fn diffuse_contrib(&self, normal: UnitVec3) -> f32 {
        self.toward_light.dot(normal).max(0.0)
    }

    pub fn specular_contrib(&self, shininess: i32, normal: UnitVec3, toward_eye: UnitVec3) -> f32 {
        let half_vector = (self.toward_light + toward_eye).normalize();
        normal.dot(half_vector).max(0.0).powi(shininess)
    }
}

#[cfg(test)]
mod tests {
    use crate::framebuffer::Rgb;
    use crate::geometry::UnitVec3;

    use super::{DirectionalLight, Material};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;

    mod diffuse_contrib {
        use super::*;

        #[test]
        fn fully_lit_when_normal_aligns_with_light() {
            let light = DirectionalLight::new(UnitVec3::Z);
            let normal = UnitVec3::Z;
            assert_relative_eq!(light.diffuse_contrib(normal), 1.0);

            let scaled = DirectionalLight::new(Vec3::new(0.0, 0.0, 3.0).into());
            assert_relative_eq!(scaled.diffuse_contrib(normal), 1.0);
        }

        #[test]
        fn zero_when_normal_does_not_face_light() {
            let light = DirectionalLight::new(TOWARD_LIGHT);

            assert_relative_eq!(light.diffuse_contrib(UnitVec3::X), 0.0);
            assert_relative_eq!(light.diffuse_contrib(UnitVec3::NEG_Z), 0.0);
        }

        #[test]
        fn oblique_light_produces_known_contrib() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
            let light = DirectionalLight::new(toward_light);
            assert_relative_eq!(light.diffuse_contrib(UnitVec3::Z), 2.0_f32.sqrt().recip());
        }
    }

    mod specular_contrib {
        use super::*;

        #[test]
        fn one_when_view_aligns_with_light() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            assert_relative_eq!(light.specular_contrib(1, UnitVec3::Z, UnitVec3::Z), 1.0);
        }

        #[test]
        fn weaker_when_view_is_tangent() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let aligned = light.specular_contrib(1, UnitVec3::Z, UnitVec3::Z);
            let tangent = light.specular_contrib(1, UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(tangent, 2.0_f32.sqrt().recip());
            assert!(tangent < aligned);
        }

        #[test]
        fn zero_when_light_and_view_cancel() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            assert_relative_eq!(light.specular_contrib(1, UnitVec3::Z, -TOWARD_LIGHT), 0.0);
        }

        #[test]
        fn higher_shininess_tightens_highlight() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let low = light.specular_contrib(1, UnitVec3::Z, UnitVec3::Y);
            let high = light.specular_contrib(2, UnitVec3::Z, UnitVec3::Y);
            assert_relative_eq!(high, 0.5);
            assert!(high < low);
        }

        #[test]
        fn high_shininess_suppresses_off_axis() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            assert_relative_eq!(light.specular_contrib(128, UnitVec3::Z, UnitVec3::Y), 0.0);
        }

        #[test]
        fn zero_shininess_yields_broad_highlight() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            assert_relative_eq!(light.specular_contrib(0, UnitVec3::Z, UnitVec3::Z), 1.0);
            assert_relative_eq!(light.specular_contrib(0, UnitVec3::Z, UnitVec3::Y), 1.0);
        }

        #[test]
        fn zero_when_half_vector_faces_away_from_normal() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, -1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
            let light = DirectionalLight::new(toward_light);
            assert_relative_eq!(light.specular_contrib(2, UnitVec3::Z, toward_eye), 0.0);
        }

        #[test]
        fn oblique_light_and_view_produce_known_contrib() {
            let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
            let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
            let light = DirectionalLight::new(toward_light);
            assert_relative_eq!(light.specular_contrib(1, UnitVec3::Z, toward_eye), 1.0);
        }
    }

    mod material_shade {
        use super::*;

        #[test]
        fn emissive_only_ignores_light_direction() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let emissive = Rgb(20, 40, 60);
            let material = Material::new(emissive, Rgb::BLACK, Rgb::BLACK, None);

            for normal in [UnitVec3::Z, UnitVec3::X, UnitVec3::NEG_Z] {
                assert_eq!(material.shade(&light, normal, UnitVec3::Z), emissive);
            }
        }

        #[test]
        fn diffuse_scales_with_light_facing() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(100, 110, 120);
            let material = Material::new(emissive, diffuse, Rgb::BLACK, None);

            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                emissive + diffuse
            );
            assert_eq!(material.shade(&light, UnitVec3::X, UnitVec3::Z), emissive);
            assert_eq!(
                material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z),
                emissive
            );
        }

        #[test]
        fn specular_adds_on_top_of_diffuse_for_shiny_material() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let diffuse = Rgb(128, 128, 128);
            let specular = Rgb(64, 64, 64);
            let diffuse_only = Material::new(Rgb::BLACK, diffuse, Rgb::BLACK, None);
            let shiny = Material::new(Rgb::BLACK, diffuse, specular, Some(1));
            let normal = UnitVec3::Z;
            let toward_eye = UnitVec3::Y;

            assert_eq!(diffuse_only.shade(&light, normal, toward_eye), diffuse);
            assert_eq!(
                shiny.shade(&light, normal, toward_eye),
                diffuse + specular * 2.0_f32.sqrt().recip()
            );
        }

        #[test]
        fn aligned_view_sums_emissive_diffuse_and_specular() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let emissive = Rgb(30, 30, 30);
            let diffuse = Rgb(40, 40, 40);
            let specular = Rgb(50, 50, 50);
            let material = Material::new(emissive, diffuse, specular, Some(1));

            assert_eq!(
                material.shade(&light, UnitVec3::Z, UnitVec3::Z),
                emissive + diffuse + specular
            );
        }

        #[test]
        fn unlit_shiny_surface_returns_emissive_only() {
            let light = DirectionalLight::new(TOWARD_LIGHT);
            let emissive = Rgb(128, 128, 128);
            let material = Material::new(emissive, Rgb(64, 64, 64), Rgb(32, 32, 32), Some(1));

            assert_eq!(
                material.shade(&light, UnitVec3::NEG_Z, UnitVec3::Z),
                emissive
            );
        }
    }
}
