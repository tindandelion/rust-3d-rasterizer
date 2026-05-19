use std::ops::{Deref, Neg};

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal3(Vec3);

impl Normal3 {
    pub const Z: Self = Self(Vec3::Z);
}

impl From<Vec3> for Normal3 {
    fn from(value: Vec3) -> Self {
        assert!(
            value.length_squared() > 0.0,
            "Normal3 requires a non-zero Vec3"
        );
        Self(value.normalize())
    }
}

impl From<Normal3> for Vec3 {
    fn from(value: Normal3) -> Self {
        value.0
    }
}

impl Deref for Normal3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Neg for Normal3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Normal3;
    use approx::assert_relative_eq;
    use glam::Vec3;

    #[test]
    fn neg_flips_underlying_vec3() {
        let n = Normal3::from(Vec3::new(1.0, 2.0, 8.0));
        assert_relative_eq!(Vec3::from(-n), -Vec3::from(n));
    }

    #[test]
    fn neg_preserves_unit_length() {
        let n = Normal3::from(Vec3::new(1.0, 2.0, 8.0));
        assert_relative_eq!((-n).length(), 1.0);
    }

    #[test]
    fn neg_z_axis_constant() {
        assert_eq!(Vec3::from(-Normal3::Z), Vec3::NEG_Z);
    }
}
