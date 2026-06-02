//! Directional **Lambert**-style diffuse lighting with a simple ambient blend.
//!
//! See [`DiffuseLight`]: direction is **from the surface toward the light** (unit vector stored internally).

use crate::geometry::UnitVec3;

/// Directional light model: **toward-light** direction plus an **ambient** fraction of directional contrast.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffuseLight {
    toward_light: UnitVec3,
    ambient_factor: f32,
    diffuse_factor: f32,
}

impl DiffuseLight {
    /// **`toward_light`:** direction **from surface toward the light**.
    pub fn new(toward_light: UnitVec3, ambient_factor: f32, diffuse_factor: f32) -> Self {
        let ambient_factor = ambient_factor.max(0.0);
        let diffuse_factor = diffuse_factor.max(0.0);
        Self {
            toward_light,
            ambient_factor,
            diffuse_factor,
        }
    }

    pub fn calc_intensity(&self, normal: UnitVec3) -> f32 {
        let diffuse = self.toward_light.dot(normal).max(0.0);
        self.ambient_factor + self.diffuse_factor * diffuse
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::UnitVec3;

    use super::DiffuseLight;
    use approx::assert_relative_eq;
    use glam::Vec3;

    #[test]
    fn pure_directional_fully_lit_when_normal_aligns_with_light() {
        let light = DiffuseLight::new(UnitVec3::Z, 0.0, 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_perpendicular_to_light() {
        let light = DiffuseLight::new(UnitVec3::Z, 0.0, 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 0.0);
    }

    #[test]
    fn pure_directional_zero_when_normal_faces_away_from_light() {
        let light = DiffuseLight::new(UnitVec3::Z, 0.0, 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 0.0);
    }

    #[test]
    fn full_ambient_is_one_for_arbitrary_normals() {
        let light = DiffuseLight::new(UnitVec3::Z, 1.0, 0.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 1.0);
    }

    #[test]
    fn half_ambient_blends_directional_term() {
        let light = DiffuseLight::new(UnitVec3::Z, 0.5, 0.5);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::X), 0.5);
        assert_relative_eq!(light.calc_intensity(UnitVec3::NEG_Z), 0.5);
    }

    #[test]
    fn non_unit_toward_light_is_normalized() {
        let light = DiffuseLight::new(Vec3::new(0.0, 0.0, 3.0).into(), 0.0, 1.0);
        assert_relative_eq!(light.calc_intensity(UnitVec3::Z), 1.0);
    }

    #[test]
    fn ambient_factor_clamps_below_range_to_zero() {
        assert_eq!(
            DiffuseLight::new(UnitVec3::Z, -0.1, 1.0),
            DiffuseLight::new(UnitVec3::Z, 0.0, 1.0),
        );
    }
}
