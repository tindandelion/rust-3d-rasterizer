//! Directional **Lambert**-style diffuse lighting with a simple ambient blend.
//!
//! See [`DiffuseLight`]: direction is **from the surface toward the light** (unit vector stored internally).

use crate::geometry::UnitVec3;

/// Surface shading coefficients: **ambient** floor and **diffuse** scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    ambient_factor: f32,
    diffuse_factor: f32,
}

impl Material {
    /// **`ambient_factor`** plus **`1.0 − ambient_factor`** diffuse (both clamped in [`new`]).
    pub const fn matte(ambient_factor: f32) -> Self {
        Self::new(ambient_factor, 1.0 - ambient_factor)
    }

    pub const fn new(ambient_factor: f32, diffuse_factor: f32) -> Self {
        Self {
            ambient_factor: ambient_factor.max(0.0),
            diffuse_factor: diffuse_factor.max(0.0),
        }
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
        let diffuse = self.toward_light.dot(normal).max(0.0);
        self.material.ambient_factor + self.material.diffuse_factor * diffuse
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::UnitVec3;

    use super::{DiffuseLight, Material};
    use approx::assert_relative_eq;
    use glam::Vec3;

    const PURE_DIFFUSE: Material = Material::new(0.0, 1.0);
    const FULL_AMBIENT: Material = Material::new(1.0, 0.0);
    const HALF_BLEND: Material = Material::new(0.5, 0.5);

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
    fn matte_matches_complementary_new() {
        let matte = DiffuseLight::new(UnitVec3::Z, Material::matte(0.2));
        let split = DiffuseLight::new(UnitVec3::Z, Material::new(0.2, 0.8));
        for normal in [UnitVec3::Z, UnitVec3::X, UnitVec3::NEG_Z] {
            assert_relative_eq!(
                matte.calc_intensity(normal),
                split.calc_intensity(normal),
            );
        }
    }

    #[test]
    fn ambient_factor_clamps_below_range_to_zero() {
        assert_eq!(Material::new(-0.1, 1.0), Material::new(0.0, 1.0));
    }

    #[test]
    fn diffuse_factor_clamps_below_range_to_zero() {
        assert_eq!(Material::new(0.5, -0.1), Material::new(0.5, 0.0));
    }
}
