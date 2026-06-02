//! Directional **Lambert**-style diffuse lighting with a simple ambient blend.
//!
//! See [`DiffuseLight`]: direction is **from the surface toward the light** (unit vector stored internally).

use crate::geometry::UnitVec3;

/// Surface shading coefficients: **ambient** floor and **diffuse** scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    ambient_factor: f32,
    diffuse_factor: f32,
    shininess: Option<f32>,
}

impl Material {
    /// **`ambient_factor`** plus **`1.0 − ambient_factor`** diffuse (both clamped in [`new`]).
    pub const fn matte(ambient_factor: f32) -> Self {
        Self::new(ambient_factor, 1.0 - ambient_factor, None)
    }

    pub const fn shiny(ambient_factor: f32, shininess: f32) -> Self {
        Self::new(ambient_factor, 1.0 - ambient_factor, Some(shininess))
    }

    const fn new(ambient_factor: f32, diffuse_factor: f32, shininess: Option<f32>) -> Self {
        Self {
            ambient_factor: ambient_factor.max(0.0),
            diffuse_factor: diffuse_factor.max(0.0),
            shininess,
        }
    }
}

pub struct PhongLightModel {
    toward_light: UnitVec3,
    material: Material,
}

impl PhongLightModel {
    pub fn new(toward_light: UnitVec3, material: Material) -> Self {
        Self {
            toward_light,
            material,
        }
    }

    pub fn calc_intensity(&self, normal: UnitVec3, toward_eye: UnitVec3) -> f32 {
        let diffuse = self.toward_light.dot(normal).max(0.0);
        let specular = self
            .material
            .shininess
            .map(|shininess| {
                let half_vector = self.toward_light.as_vec3() + toward_eye.as_vec3();
                if half_vector.length_squared() > 0.0 {
                    let half_vector: UnitVec3 = half_vector.into();
                    half_vector.dot(normal).max(0.0).powf(shininess)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        self.material.ambient_factor + self.material.diffuse_factor * (diffuse + specular)
    }
}

/// Directional light: **toward-light** direction and surface [`Material`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffuseLight {
    toward_light: UnitVec3,
    material: Material,
}

impl DiffuseLight {
    /// **`toward_light`:** direction **from surface toward the light**.
    pub fn new(toward_light: UnitVec3, material: Material) -> Self {
        Self {
            toward_light,
            material,
        }
    }

    pub fn calc_intensity(&self, normal: UnitVec3) -> f32 {
        let diffuse = self.material.diffuse_factor * self.toward_light.dot(normal).max(0.0);
        self.material.ambient_factor + diffuse
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::UnitVec3;

    use super::{DiffuseLight, Material};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const PURE_DIFFUSE: Material = Material::matte(0.0);
    const FULL_AMBIENT: Material = Material::matte(1.0);
    const HALF_BLEND: Material = Material::matte(0.5);

    #[test]
    fn pure_directional_fully_lit_when_normal_aligns_with_light() {
        let light = DiffuseLight::new(UnitVec3::Z, PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_perpendicular_to_light() {
        let light = DiffuseLight::new(UnitVec3::Z, PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 0.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_faces_away_from_light() {
        let light = DiffuseLight::new(UnitVec3::Z, PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 0.0);
    }

    #[test]
    fn full_ambient_is_one_for_arbitrary_normals() {
        let light = DiffuseLight::new(UnitVec3::Z, FULL_AMBIENT);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 1.0);
    }

    #[test]
    fn half_ambient_blends_directional_term() {
        let light = DiffuseLight::new(UnitVec3::Z, HALF_BLEND);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 0.5);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 0.5);
    }

    #[test]
    fn non_unit_toward_light_is_normalized() {
        let light = DiffuseLight::new(Vec3::new(0.0, 0.0, 3.0).into(), PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
    }

    #[test]
    fn ambient_factor_clamps_below_range_to_zero() {
        assert_eq!(
            Material::new(-0.1, 1.0, None),
            Material::new(0.0, 1.0, None)
        );
    }

    #[test]
    fn diffuse_factor_clamps_below_range_to_zero() {
        assert_eq!(
            Material::new(0.5, -0.1, None),
            Material::new(0.5, 0.0, None)
        );
    }
}

#[cfg(test)]
mod phong_light_model {
    use crate::geometry::UnitVec3;

    use super::{DiffuseLight, Material, PhongLightModel};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const TOWARD_LIGHT: UnitVec3 = UnitVec3::Z;

    const PURE_DIFFUSE: Material = Material::matte(0.0);
    const FULL_AMBIENT: Material = Material::matte(1.0);
    const HALF_BLEND: Material = Material::matte(0.5);

    #[test]
    fn pure_directional_fully_lit_when_normal_aligns_with_light() {
        let toward_light = UnitVec3::Z;
        let light = PhongLightModel::new(toward_light, PURE_DIFFUSE);

        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(normal, toward_eye), 1.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_perpendicular_to_light() {
        let light = PhongLightModel::new(TOWARD_LIGHT, PURE_DIFFUSE);

        let normal = UnitVec3::X;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(normal, toward_eye), 0.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_faces_away_from_light() {
        let light = PhongLightModel::new(TOWARD_LIGHT, PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z, UnitVec3::Z), 0.0);
    }

    #[test]
    fn full_ambient_is_one_when_normal_faces_away_from_light() {
        let normal = UnitVec3::NEG_Z;
        let toward_eye = -normal;

        let light = PhongLightModel::new(TOWARD_LIGHT, FULL_AMBIENT);
        assert_relative_eq!(light.calc_intensity(normal, toward_eye), 1.0);
    }

    #[test]
    fn half_ambient_blends_directional_term() {
        let light = PhongLightModel::new(TOWARD_LIGHT, HALF_BLEND);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X, UnitVec3::Z), 0.5);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z, UnitVec3::Z), 0.5);
    }

    #[test]
    fn matte_matches_diffuse_light_intensity() {
        let phong = PhongLightModel::new(TOWARD_LIGHT, HALF_BLEND);
        let diffuse = DiffuseLight::new(TOWARD_LIGHT, HALF_BLEND);
        let normal = UnitVec3::X;
        let toward_eye = UnitVec3::Y;
        assert_relative_eq!(
            phong.calc_intensity(normal, toward_eye),
            diffuse.calc_intensity(normal)
        );
    }

    #[test]
    fn matte_material_ignores_view_direction() {
        let light = PhongLightModel::new(TOWARD_LIGHT, PURE_DIFFUSE);
        let from_front = light.calc_intensity(UnitVec3::Z, UnitVec3::Z);
        let from_side = light.calc_intensity(UnitVec3::Z, UnitVec3::Y);
        assert_relative_eq!(from_front, from_side);
    }

    #[test]
    fn shiny_material_brightens_tangent_view_over_matte() {
        let matte = PhongLightModel::new(TOWARD_LIGHT, PURE_DIFFUSE);
        let shiny = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 1.0));
        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Y;
        assert_relative_eq!(matte.calc_intensity(normal, toward_eye), 1.0);
        assert_relative_eq!(
            shiny.calc_intensity(normal, toward_eye),
            1.0 + 2.0_f32.sqrt().recip()
        );
    }

    #[test]
    fn specular_adds_to_diffuse_when_view_aligns_with_light() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 1.0));

        let normal = UnitVec3::Z;
        let toward_eye = UnitVec3::Z;
        assert_relative_eq!(light.calc_intensity(normal, toward_eye), 2.0);
    }

    #[test]
    fn specular_is_weaker_when_view_is_tangent_to_surface() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 1.0));
        let aligned = light.calc_intensity(UnitVec3::Z, UnitVec3::Z);
        let tangent = light.calc_intensity(UnitVec3::Z, UnitVec3::Y);
        assert_relative_eq!(tangent, 1.0 + 2.0_f32.sqrt().recip());
        assert!(tangent < aligned);
    }

    #[test]
    fn half_ambient_scales_specular_on_aligned_view() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.5, 1.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Z), 1.5);
    }

    #[test]
    fn shiny_material_falls_back_to_ambient_on_unlit_surface() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.5, 1.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z, UnitVec3::Z), 0.5);
    }

    #[test]
    fn specular_is_skipped_when_light_and_view_cancel() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 1.0));

        let normal = UnitVec3::Z;
        let toward_eye = -TOWARD_LIGHT;
        assert_relative_eq!(light.calc_intensity(normal, toward_eye), 1.0);
    }

    #[test]
    fn higher_shininess_tightens_specular_highlight() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 2.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Y), 1.5);
    }

    #[test]
    fn high_shininess_approaches_diffuse_only_off_axis() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 128.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Y), 1.0);
    }

    #[test]
    fn zero_shininess_yields_broad_specular_highlight() {
        let light = PhongLightModel::new(TOWARD_LIGHT, Material::shiny(0.0, 0.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Z), 2.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Y), 2.0);
    }

    #[test]
    fn specular_is_zero_when_half_vector_faces_away_from_normal() {
        let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, -1.0).into();
        let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, -1.0).into();
        let light = PhongLightModel::new(toward_light, Material::shiny(0.0, 2.0));
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, toward_eye), 0.0);
    }

    #[test]
    fn oblique_light_and_view_produce_known_intensity() {
        let toward_light: UnitVec3 = Vec3::new(1.0, 0.0, 1.0).into();
        let toward_eye: UnitVec3 = Vec3::new(-1.0, 0.0, 1.0).into();
        let light = PhongLightModel::new(toward_light, Material::shiny(0.0, 1.0));
        assert_relative_eq!(
            light.calc_intensity(UnitVec3::Z, toward_eye),
            1.0 + 2.0_f32.sqrt().recip()
        );
    }

    #[test]
    fn non_unit_toward_light_is_normalized() {
        let light = PhongLightModel::new(Vec3::new(0.0, 0.0, 3.0).into(), PURE_DIFFUSE);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z, UnitVec3::Z), 1.0);
    }
}
