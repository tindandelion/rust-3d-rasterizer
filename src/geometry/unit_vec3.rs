use std::ops::{Add, Deref, Neg};

use approx::{AbsDiffEq, RelativeEq};
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitVec3(Vec3);

impl UnitVec3 {
    pub const X: Self = Self(Vec3::X);
    pub const NEG_X: Self = Self(Vec3::NEG_X);
    pub const Y: Self = Self(Vec3::Y);
    pub const NEG_Y: Self = Self(Vec3::NEG_Y);
    pub const Z: Self = Self(Vec3::Z);
    pub const NEG_Z: Self = Self(Vec3::NEG_Z);

    /// Outward-facing unit normal for triangle corners **`vertices[0]` → `vertices[1]` → `vertices[2]`**
    /// in the same winding sense as **[`crate::geometry::Facet`]**: CCW viewed from outside along the normal,
    /// two edge vectors anchored at **`vertices[0]`**, cross **\((v_2 - v_0) \times (v_1 - v_0)\)**.
    pub fn from_points_ccw(vertices: &[Vec3; 3]) -> Self {
        let vec1 = vertices[1] - vertices[0];
        let vec2 = vertices[2] - vertices[0];
        vec2.cross(vec1).into()
    }

    pub fn dot(&self, other: impl Into<Vec3>) -> f32 {
        self.0.dot(other.into())
    }

    pub fn as_vec3(&self) -> &Vec3 {
        &self.0
    }
}

impl From<Vec3> for UnitVec3 {
    // TODO: Probably, we should move to using TryFrom<Vec3> for UnitVec3.
    fn from(value: Vec3) -> Self {
        assert!(
            value.length_squared() > 0.0,
            "UnitVec3 requires a non-zero Vec3"
        );
        Self(value.normalize())
    }
}

impl From<UnitVec3> for Vec3 {
    fn from(value: UnitVec3) -> Self {
        value.0
    }
}

impl Deref for UnitVec3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Neg for UnitVec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for UnitVec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Self::Output {
        self.0 + other.0
    }
}

impl AbsDiffEq for UnitVec3 {
    type Epsilon = <Vec3 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        <Vec3 as AbsDiffEq>::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        <Vec3 as AbsDiffEq>::abs_diff_eq(&self.0, &other.0, epsilon)
    }
}

impl RelativeEq for UnitVec3 {
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
    use super::UnitVec3;
    use approx::assert_relative_eq;
    use glam::Vec3;

    #[test]
    fn neg_flips_underlying_vec3() {
        let n = UnitVec3::from(Vec3::new(1.0, 2.0, 8.0));
        assert_relative_eq!(Vec3::from(-n), -Vec3::from(n));
    }

    #[test]
    fn neg_preserves_unit_length() {
        let n = UnitVec3::from(Vec3::new(1.0, 2.0, 8.0));
        assert_relative_eq!((-n).length(), 1.0);
    }

    #[test]
    fn relative_eq_delegates_to_inner_vec3() {
        let v = Vec3::new(1.0, 2.0, 8.0);
        let a = UnitVec3::from(v);
        let b = UnitVec3::from(v + Vec3::new(1e-8, 0.0, 0.0));
        assert_relative_eq!(a, b);
    }

    // Same corner order as `shapes::sphere` facet indices [4, 1, 0] (+ octant).
    #[test]
    fn from_vertices_ccw_octahedron_positive_octant() {
        let corners = [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ];
        let n = UnitVec3::from_points_ccw(&corners);
        assert_relative_eq!(n, UnitVec3::from(Vec3::ONE));
        assert_relative_eq!(n.length_squared(), 1.0);
    }

    #[test]
    fn from_vertices_ccw_swapping_last_two_corners_flips_normal() {
        let corners = [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ];
        let swapped = [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let a = UnitVec3::from_points_ccw(&corners);
        let b = UnitVec3::from_points_ccw(&swapped);
        assert_relative_eq!(a, -b);
    }

    #[test]
    #[should_panic(expected = "UnitVec3 requires a non-zero Vec3")]
    fn from_vertices_ccw_collinear_corners_panics() {
        let corners = [Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)];
        let _ = UnitVec3::from_points_ccw(&corners);
    }
}
