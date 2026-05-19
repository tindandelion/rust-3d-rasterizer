use std::ops::{Deref, Neg};

use approx::{AbsDiffEq, RelativeEq};
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal3(Vec3);

impl Normal3 {
    pub const X: Self = Self(Vec3::X);
    pub const NEG_X: Self = Self(Vec3::NEG_X);
    pub const Y: Self = Self(Vec3::Y);
    pub const NEG_Y: Self = Self(Vec3::NEG_Y);
    pub const Z: Self = Self(Vec3::Z);
    pub const NEG_Z: Self = Self(Vec3::NEG_Z);

    pub fn dot(&self, other: Self) -> f32 {
        self.0.dot(other.0)
    }
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

impl AbsDiffEq for Normal3 {
    type Epsilon = <Vec3 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        <Vec3 as AbsDiffEq>::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        <Vec3 as AbsDiffEq>::abs_diff_eq(&self.0, &other.0, epsilon)
    }
}

impl RelativeEq for Normal3 {
    fn default_max_relative() -> Self::Epsilon {
        <Vec3 as RelativeEq>::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        <Vec3 as RelativeEq>::relative_eq(&self.0, &other.0, epsilon, max_relative)
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

    #[test]
    fn relative_eq_delegates_to_inner_vec3() {
        let v = Vec3::new(1.0, 2.0, 8.0);
        let a = Normal3::from(v);
        let b = Normal3::from(v + Vec3::new(1e-8, 0.0, 0.0));
        assert_relative_eq!(a, b);
    }
}
