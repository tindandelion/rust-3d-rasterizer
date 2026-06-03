//! Inverse-transpose of a model matrix's linear **3×3** — maps unit normals under **`Mat4`** posing.

use glam::{Mat3, Mat4, Vec3};

/// Linear map for transforming **normals** alongside a **`Mat4`** point transform.
///
/// Construct with **[`NormalTransform::from_model`]** once per pose (e.g. in
/// **[`Shape::transform`](crate::geometry::Shape::transform)**), then pass to
/// **[`Facet::transform`](crate::geometry::Facet::transform)**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalTransform(Mat3);

impl NormalTransform {
    /// Inverse-transpose of **`model`**'s upper-left **3×3** (translation ignored).
    pub fn from_model(model: Mat4) -> Self {
        let linear = Mat3::from_mat4(model);
        Self(linear.inverse().transpose())
    }

    #[inline]
    pub(crate) fn apply(self, v: Vec3) -> Vec3 {
        self.0.mul_vec3(v)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::{Mat4, Vec3};
    use std::f32::consts::FRAC_PI_2;

    use super::NormalTransform;
    use crate::geometry::UnitVec3;

    #[test]
    fn from_model_rotation_matches_transform_vector3() {
        let m = Mat4::from_rotation_x(FRAC_PI_2);
        let n: UnitVec3 = Vec3::Z.into();
        let expected: UnitVec3 = m.transform_vector3(n.into()).into();
        let actual: UnitVec3 = NormalTransform::from_model(m).apply(n.into()).into();
        assert_relative_eq!(expected, actual);
    }

    #[test]
    fn from_model_non_uniform_scale_inverts_axis_scale() {
        let m = Mat4::from_scale(Vec3::new(1.0, 0.5, 1.0));
        let n = Vec3::new(1.0, 1.0, 1.0);
        let posed = NormalTransform::from_model(m).apply(n);
        assert_relative_eq!(posed, Vec3::new(1.0, 2.0, 1.0));
    }
}
