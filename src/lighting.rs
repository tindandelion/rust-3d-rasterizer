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
}

pub struct BlinnLightModel {
    toward_light: UnitVec3,
}

impl BlinnLightModel {
    pub fn new(toward_light: UnitVec3) -> Self {
        Self { toward_light }
    }

    pub fn calc_intensity(
        &self,
        material: Material,
        normal: UnitVec3,
        toward_eye: UnitVec3,
    ) -> f32 {
        let diffuse = self.toward_light.dot(normal).max(0.0);
        let specular = material
            .shininess
            .map(|shininess| {
                let half_vector = (self.toward_light + toward_eye).normalize();
                normal.dot(half_vector).max(0.0).powi(shininess)
            })
            .unwrap_or(0.0);
        material.ambient_factor + material.diffuse_factor * (diffuse + specular)
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

    const PURE_DIFFUSE: Material = Material::matte(Rgb::WHITE, 0.0);
    const FULL_AMBIENT: Material = Material::matte(Rgb::WHITE, 1.0);
    const HALF_BLEND: Material = Material::matte(Rgb::WHITE, 0.5);

    #[test]
    fn pure_directional_fully_lit_when_normal_aligns_with_light() {
        let toward_light = UnitVec3::Z;
        let light = BlinnLightModel::new(toward_light);

        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(PURE_DIFFUSE, normal, toward_eye), 1.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_perpendicular_to_light() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);

        let normal = UnitVec3::X;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(PURE_DIFFUSE, normal, toward_eye), 0.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_faces_away_from_light() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        assert_relative_eq!(light.calc_intensity(PURE_DIFFUSE, UnitVec3::NEG_Z, UnitVec3::Z), 0.0);
    }

    #[test]
    fn full_ambient_is_one_when_normal_faces_away_from_light() {
        let normal = UnitVec3::NEG_Z;
        let toward_eye = -normal;

        let light = BlinnLightModel::new(TOWARD_LIGHT);
        assert_relative_eq!(light.calc_intensity(FULL_AMBIENT, normal, toward_eye), 1.0);
    }

    #[test]
    fn half_ambient_blends_directional_term() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        assert_relative_eq!(light.calc_intensity(HALF_BLEND, UnitVec3::Z, UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(HALF_BLEND, UnitVec3::X, UnitVec3::Z), 0.5);
        assert_relative_eq!(light.calc_intensity(HALF_BLEND, UnitVec3::NEG_Z, UnitVec3::Z), 0.5);
    }

    #[test]
    fn matte_material_ignores_view_direction() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let from_front = light.calc_intensity(PURE_DIFFUSE, UnitVec3::Z, UnitVec3::Z);
        let from_side = light.calc_intensity(PURE_DIFFUSE, UnitVec3::Z, UnitVec3::Y);
        assert_relative_eq!(from_front, from_side);
    }

    #[test]
    fn shiny_material_brightens_tangent_view_over_matte() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 1);

        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Y;
        assert_relative_eq!(light.calc_intensity(PURE_DIFFUSE, normal, toward_eye), 1.0);
        assert_relative_eq!(
            light.calc_intensity(shiny, normal, toward_eye),
            1.0 + 2.0_f32.sqrt().recip()
        );
    }

    #[test]
    fn specular_adds_to_diffuse_when_view_aligns_with_light() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 1);

        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(shiny, normal, toward_eye), 2.0);
    }

    #[test]
    fn specular_is_weaker_when_view_is_tangent_to_surface() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 1);
        let aligned = light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Z);
        let tangent = light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Y);
        assert_relative_eq!(tangent, 1.0 + 2.0_f32.sqrt().recip());
        assert!(tangent < aligned);
    }

    #[test]
    fn half_ambient_scales_specular_on_aligned_view() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.5, 1);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Z), 1.5);
    }

    #[test]
    fn shiny_material_falls_back_to_ambient_on_unlit_surface() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.5, 1);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::NEG_Z, UnitVec3::Z), 0.5);
    }

    #[test]
    fn specular_is_skipped_when_light_and_view_cancel() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 1);

        let normal = UnitVec3::Z;
        let toward_eye = -TOWARD_LIGHT;
        assert_relative_eq!(light.calc_intensity(shiny, normal, toward_eye), 1.0);
    }

    #[test]
    fn higher_shininess_tightens_specular_highlight() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 2);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Y), 1.5);
    }

    #[test]
    fn high_shininess_approaches_diffuse_only_off_axis() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 128);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Y), 1.0);
    }

    #[test]
    fn zero_shininess_yields_broad_specular_highlight() {
        let light = BlinnLightModel::new(TOWARD_LIGHT);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 0);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Z), 2.0);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, UnitVec3::Y), 2.0);
    }

    #[test]
    fn specular_is_zero_when_half_vector_faces_away_from_normal() {
        let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, -1.0).into();
        let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
        let light = BlinnLightModel::new(toward_light);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 2);
        assert_relative_eq!(light.calc_intensity(shiny, UnitVec3::Z, toward_eye), 0.0);
    }

    #[test]
    fn oblique_light_and_view_produce_known_intensity() {
        let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
        let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
        let light = BlinnLightModel::new(toward_light);
        let shiny = Material::shiny(Rgb::WHITE, 0.0, 1);
        assert_relative_eq!(
            light.calc_intensity(shiny, UnitVec3::Z, toward_eye),
            1.0 + 2.0_f32.sqrt().recip()
        );
    }

    #[test]
    fn non_unit_toward_light_is_normalized() {
        let light = BlinnLightModel::new(Vec3::new(0.0, 0.0, 3.0).into());
        assert_relative_eq!(light.calc_intensity(PURE_DIFFUSE, UnitVec3::Z, UnitVec3::Z), 1.0);
    }
}
