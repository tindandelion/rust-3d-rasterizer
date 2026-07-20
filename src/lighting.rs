mod color;
mod light;

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
    pub fn new(emissive: Color, diffuse: Color, specular: Color, shininess: Option<i32>) -> Self {
        Self {
            emissive,
            diffuse,
            specular,
            shininess,
        }
    }

    pub fn shade(&self, lights: &[Light], point: SurfacePoint, toward_eye: UnitVec3) -> Color {
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
        self.emissive + self.diffuse * diffuse_contrib + self.specular * specular_contrib
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::UnitVec3;

    use super::{Light, Material, color::Color};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;
    const BLACK: Color = Color(0.0, 0.0, 0.0);

    mod material_shade {
        use crate::geometry::SurfacePoint;

        use super::*;

        fn shade_linear(
            emissive: Color,
            diffuse: Color,
            specular: Color,
            diffuse_contrib: f32,
            specular_contrib: f32,
        ) -> Color {
            emissive + diffuse * diffuse_contrib + specular * specular_contrib
        }

        fn assert_shade_eq(
            material: &Material,
            lights: &[Light],
            point: SurfacePoint,
            toward_eye: UnitVec3,
            expected: Color,
        ) {
            assert_relative_eq!(expected, material.shade(lights, point, toward_eye));
        }

        #[test]
        fn emissive_only_ignores_light_direction() {
            let emissive = Color(0.04, 0.08, 0.12);
            let material = Material::new(emissive, BLACK, BLACK, None);

            for normal in [UnitVec3::Z, UnitVec3::X, UnitVec3::NEG_Z] {
                let light = Light::directional(TOWARD_LIGHT, 1.0);
                let point = SurfacePoint::new(Vec3::ZERO, normal);
                assert_shade_eq(&material, &[light], point, UnitVec3::Z, emissive);
            }
        }

        #[test]
        fn diffuse_scales_with_light_facing() {
            let emissive = Color(0.02, 0.04, 0.06);
            let diffuse = Color(0.5, 0.55, 0.6);
            let material = Material::new(emissive, diffuse, BLACK, None);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_shade_eq(
                &material,
                &[light],
                point,
                UnitVec3::Z,
                shade_linear(emissive, diffuse, BLACK, 1.0, 0.0),
            );

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::X);
            assert_shade_eq(&material, &[light], point, UnitVec3::Z, emissive);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);
            assert_shade_eq(&material, &[light], point, UnitVec3::Z, emissive);
        }

        #[test]
        fn specular_adds_on_top_of_diffuse_for_shiny_material() {
            let diffuse = Color(0.5, 0.5, 0.5);
            let specular = Color(0.25, 0.25, 0.25);
            let diffuse_only = Material::new(BLACK, diffuse, BLACK, None);
            let shiny = Material::new(BLACK, diffuse, specular, Some(1));
            let toward_eye = UnitVec3::Y;
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            assert_shade_eq(
                &diffuse_only,
                &[light],
                point,
                toward_eye,
                shade_linear(BLACK, diffuse, BLACK, 1.0, 0.0),
            );

            let light = Light::directional(TOWARD_LIGHT, 1.0);
            assert_shade_eq(
                &shiny,
                &[light],
                point,
                toward_eye,
                shade_linear(BLACK, diffuse, specular, 1.0, 2.0_f32.sqrt().recip()),
            );
        }

        #[test]
        fn aligned_view_sums_emissive_diffuse_and_specular() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let emissive = Color(0.1, 0.1, 0.1);
            let diffuse = Color(0.15, 0.15, 0.15);
            let specular = Color(0.2, 0.2, 0.2);
            let material = Material::new(emissive, diffuse, specular, Some(1));

            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            assert_shade_eq(
                &material,
                &[light],
                point,
                UnitVec3::Z,
                shade_linear(emissive, diffuse, specular, 1.0, 1.0),
            );
        }

        #[test]
        fn unlit_shiny_surface_returns_emissive_only() {
            let light = Light::directional(TOWARD_LIGHT, 1.0);
            let emissive = Color(0.5, 0.5, 0.5);
            let material = Material::new(
                emissive,
                Color(0.25, 0.25, 0.25),
                Color(0.125, 0.125, 0.125),
                Some(1),
            );

            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::NEG_Z);
            assert_shade_eq(&material, &[light], point, UnitVec3::Z, emissive);
        }

        #[test]
        fn intensity_scales_diffuse_and_specular_but_not_emissive() {
            let toward_eye = UnitVec3::Z;
            let emissive = Color(0.02, 0.04, 0.06);
            let diffuse = Color(0.15, 0.15, 0.15);
            let specular = Color(0.2, 0.2, 0.2);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let triple = Light::directional(TOWARD_LIGHT, 3.0);
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_shade_eq(
                &material,
                &[unit],
                point,
                toward_eye,
                shade_linear(emissive, diffuse, specular, 1.0, 1.0),
            );
            assert_shade_eq(
                &material,
                &[triple],
                point,
                toward_eye,
                shade_linear(emissive, diffuse, specular, 3.0, 3.0),
            );
        }

        #[test]
        fn sums_diffuse_and_specular_across_lights() {
            let toward_eye = UnitVec3::Z;
            let emissive = Color(0.02, 0.04, 0.06);
            let diffuse = Color(0.15, 0.15, 0.15);
            let specular = Color(0.2, 0.2, 0.2);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let lights = [
                Light::directional(TOWARD_LIGHT, 1.0),
                Light::directional(TOWARD_LIGHT, 1.0),
            ];

            assert_shade_eq(
                &material,
                &lights,
                point,
                toward_eye,
                shade_linear(emissive, diffuse, specular, 2.0, 2.0),
            );
        }

        #[test]
        fn zero_intensity_light_adds_no_contribution() {
            let toward_eye = UnitVec3::Z;
            let emissive = Color(0.02, 0.04, 0.06);
            let diffuse = Color(0.15, 0.15, 0.15);
            let specular = Color(0.2, 0.2, 0.2);
            let material = Material::new(emissive, diffuse, specular, Some(1));
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);
            let unit = Light::directional(TOWARD_LIGHT, 1.0);
            let zero = Light::directional(TOWARD_LIGHT, 0.0);

            assert_shade_eq(
                &material,
                &[unit, zero],
                point,
                toward_eye,
                shade_linear(emissive, diffuse, specular, 1.0, 1.0),
            );
        }

        #[test]
        fn empty_lights_returns_emissive_only() {
            let emissive = Color(0.03, 0.05, 0.07);
            let material = Material::new(
                emissive,
                Color(0.25, 0.25, 0.25),
                Color(0.125, 0.125, 0.125),
                Some(1),
            );
            let point = SurfacePoint::new(Vec3::ZERO, UnitVec3::Z);

            assert_shade_eq(&material, &[], point, UnitVec3::Z, emissive);
        }
    }
}
