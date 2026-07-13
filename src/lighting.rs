mod color;
mod light;

use crate::framebuffer::Rgb;
use crate::geometry::{SurfacePoint, UnitVec3};

pub use color::Color;
pub use light::DistanceFalloff;
pub use light::Light;

/// Surface **emissive**, pre-scaled **diffuse**, and **specular** colors for Phong shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    emissive: Color,
    diffuse: Color,
    specular: Color,
    shininess: Option<i32>,
}

impl Material {
    /// **`shininess`** enables specular when **`Some`**.
    pub fn new(emissive: Rgb, diffuse: Rgb, specular: Rgb, shininess: Option<i32>) -> Self {
        Self {
            emissive: Color::from(emissive),
            diffuse: Color::from(diffuse),
            specular: Color::from(specular),

            shininess,
        }
    }

    pub fn shade(&self, lights: &[Light], point: SurfacePoint, toward_eye: UnitVec3) -> Rgb {
        let diffuse_contrib: f32 = lights
            .iter()
            .map(|light| light.diffuse_contrib(point))
            .sum();
        let specular_contrib: f32 = self.shininess.map_or(0.0, |shininess| {
            lights
                .iter()
                .map(|light| light.specular_contrib(shininess, point, toward_eye))
                .sum()
        });
        let shaded_color =
            self.emissive + self.diffuse * diffuse_contrib + self.specular * specular_contrib;
        Rgb::from(shaded_color)
    }
}

#[cfg(test)]
mod tests {
    use crate::framebuffer::Rgb;
    use crate::geometry::UnitVec3;

    use super::{Light, Material, color::Color};
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;

    mod material_shade {
        use crate::geometry::SurfacePoint;

        use super::*;

        fn shade_linear(
            emissive: Rgb,
            diffuse: Rgb,
            specular: Rgb,
            diffuse_contrib: f32,
            specular_contrib: f32,
        ) -> Rgb {
            let emissive = Color::from(emissive);
            let diffuse = Color::from(diffuse);
            let specular = Color::from(specular);
            Rgb::from(emissive + diffuse * diffuse_contrib + specular * specular_contrib)
        }

        #[test]
        fn emissive_only_ignores_light_direction() {
            let emissive = Rgb(20, 40, 60);
            let material = Material::new(emissive, Rgb::BLACK, Rgb::BLACK, None);

            for normal in [UnitVec3::Z, UnitVec3::X, UnitVec3::NEG_Z] {
                let light = Light::directional(TOWARD_LIGHT, 1.0);
                let point = SurfacePoint::new(Vec3::ZERO, normal);
                assert_eq!(material.shade(&[light], point, UnitVec3::Z), emissive);
            }
        }

        #[test]
        fn diffuse_scales_with_light_facing() {
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(100, 110, 120);
            let material = Material::new(emissive, diffuse, Rgb::BLACK, None);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_eq!(
                material.shade(&[light], point, UnitVec3::Z),
                shade_linear(emissive, diffuse, Rgb::BLACK, 1.0, 0.0)
            );

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::X);
            assert_eq!(material.shade(&[light], point, UnitVec3::Z), emissive);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);
            assert_eq!(material.shade(&[light], point, UnitVec3::Z), emissive);
        }

        #[test]
        fn specular_adds_on_top_of_diffuse_for_shiny_material() {
            let diffuse = Rgb(128, 128, 128);
            let specular = Rgb(64, 64, 64);
            let diffuse_only = Material::new(Rgb::BLACK, diffuse, Rgb::BLACK, None);
            let shiny = Material::new(Rgb::BLACK, diffuse, specular, Some(1));
            let toward_eye = UnitVec3::Y;
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            assert_eq!(
                diffuse_only.shade(&[light], point, toward_eye),
                shade_linear(Rgb::BLACK, diffuse, Rgb::BLACK, 1.0, 0.0)
            );

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            assert_eq!(
                shiny.shade(&[light], point, toward_eye),
                shade_linear(Rgb::BLACK, diffuse, specular, 1.0, 2.0_f32.sqrt().recip(),)
            );
        }

        #[test]
        fn aligned_view_sums_emissive_diffuse_and_specular() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let emissive = Rgb(30, 30, 30);
            let diffuse = Rgb(40, 40, 40);
            let specular = Rgb(50, 50, 50);
            let material = Material::new(emissive, diffuse, specular, Some(1));

            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_eq!(
                material.shade(&[light], point, UnitVec3::Z),
                shade_linear(emissive, diffuse, specular, 1.0, 1.0)
            );
        }

        #[test]
        fn unlit_shiny_surface_returns_emissive_only() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let emissive = Rgb(128, 128, 128);
            let material = Material::new(emissive, Rgb(64, 64, 64), Rgb(32, 32, 32), Some(1));

            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);
            assert_eq!(material.shade(&[light], point, UnitVec3::Z), emissive);
        }

        #[test]
        fn intensity_scales_diffuse_and_specular_but_not_emissive() {
            let toward_eye = UnitVec3::Z;
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(40, 40, 40);
            let specular = Rgb(50, 50, 50);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let triple = Light::directional(TOWARD_LIGHT, 3.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_eq!(
                material.shade(&[unit], point, toward_eye),
                shade_linear(emissive, diffuse, specular, 1.0, 1.0)
            );
            assert_eq!(
                material.shade(&[triple], point, toward_eye),
                shade_linear(emissive, diffuse, specular, 3.0, 3.0)
            );
        }

        #[test]
        fn sums_diffuse_and_specular_across_lights() {
            let toward_eye = UnitVec3::Z;
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(40, 40, 40);
            let specular = Rgb(50, 50, 50);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let lights = [
                Light::directional(TOWARD_LIGHT, 1.0),
                Light::directional(TOWARD_LIGHT, 1.0),
            ];

            assert_eq!(
                material.shade(&lights, point, toward_eye),
                shade_linear(emissive, diffuse, specular, 2.0, 2.0)
            );
        }

        #[test]
        fn zero_intensity_light_adds_no_contribution() {
            let toward_eye = UnitVec3::Z;
            let emissive = Rgb(10, 20, 30);
            let diffuse = Rgb(40, 40, 40);
            let specular = Rgb(50, 50, 50);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let zero = Light::directional(TOWARD_LIGHT, 0.0);

            assert_eq!(
                material.shade(&[unit, zero], point, toward_eye),
                shade_linear(emissive, diffuse, specular, 1.0, 1.0)
            );
        }

        #[test]
        fn empty_lights_returns_emissive_only() {
            let emissive = Rgb(15, 25, 35);
            let material = Material::new(emissive, Rgb(64, 64, 64), Rgb(32, 32, 32), Some(1));
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_eq!(material.shade(&[], point, UnitVec3::Z), emissive);
        }
    }
}
