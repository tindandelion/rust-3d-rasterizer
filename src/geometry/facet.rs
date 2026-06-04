//! Triangular hull facet (**CCW winding** vertex indices viewed from outside along the outward [`UnitVec3`]).
//!
//! Stored indices reference a parent mesh **`vertices`** (**[`Shape`](crate::geometry::Shape)**, e.g. [`cube`](crate::shapes::cube), [`dodecahedron`](crate::shapes::dodecahedron))—**Facet** stays mesh‑agnostic and **does not** embed positions.

use glam::{Mat3, Mat4, Vec3};

use super::unit_vec3::UnitVec3;

/// One planar triangle: three **CCW** vertex indices (**`verts`**) into a mesh **`vertices`**, plus outward **unit** **[`UnitVec3`]** (**same spatial frame** as **`vertices`**).
///
/// **`verts[k]` ↔ `verts[(k + 1) % 3]`** (**k = 0, 1, 2**) traverses boundary **counter‑clockwise** when looking from outside along **`normal`** toward the facet interior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Facet {
    /// Outward **unit** normal (re-posed via inverse-transpose when the mesh is transformed).
    normal: UnitVec3,
    /// Indices into the parent mesh **`vertices`** (CCW winding as seen against **`normal`**).
    verts: [usize; 3],
    /// Outward unit normals at each vertex.
    vertex_normals: [UnitVec3; 3],
}

impl Facet {
    pub const fn with_facet_normal(verts: [usize; 3], normal: UnitVec3) -> Self {
        Self::with_normals(verts, normal, [normal, normal, normal])
    }

    pub fn with_vertex_normals(verts: [usize; 3], vertex_normals: [UnitVec3; 3]) -> Self {
        let facet_normal = vertex_normals[0] + vertex_normals[1] + vertex_normals[2];
        Self::with_normals(verts, facet_normal.into(), vertex_normals)
    }

    const fn with_normals(
        verts: [usize; 3],
        facet_normal: UnitVec3,
        vertex_normals: [UnitVec3; 3],
    ) -> Self {
        Self {
            normal: facet_normal,
            verts,
            vertex_normals,
        }
    }

    /// Indices into the parent **`vertices`** (winding follows [`Self::edges`] CCW traversal).
    pub fn vert_indices(&self) -> &[usize; 3] {
        &self.verts
    }

    pub fn resolve_vertices(&self, vertices: &[Vec3]) -> [Vec3; 3] {
        [
            vertices[self.verts[0]],
            vertices[self.verts[1]],
            vertices[self.verts[2]],
        ]
    }

    /// Outward unit normal in **`vertices`** space.
    pub fn facet_normal(&self) -> UnitVec3 {
        self.normal
    }

    pub fn vertex_normals(&self) -> &[UnitVec3; 3] {
        &self.vertex_normals
    }

    /// Re-transforms stored normals with **`normal_transform`**; copies **`verts`** unchanged.
    pub(crate) fn transform(&self, normal_transform: NormalTransform) -> Facet {
        Facet {
            verts: self.verts,
            normal: normal_transform.apply(self.normal.into()).into(),
            vertex_normals: self
                .vertex_normals
                .map(|n| normal_transform.apply(n.into()).into()),
        }
    }

    /// Three undirected vertex-index pairs (**`verts[k]` ↔ `verts[(k + 1) % 3]`**) following triangular CCW winding.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let v = self.verts;
        [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])].into_iter()
    }

    /// **`normal` · `view_direction` < 0** for **into‑scene** view (**[`crate::Camera::direction`]**)—mirrors **`cube`** / hull wireframe classification.
    ///
    /// Grazing (**`dot == 0`**) is **not** front-facing.
    pub fn is_front_facing(&self, view_direction: UnitVec3) -> bool {
        view_direction.dot(self.normal) < 0.0
    }
}

/// Linear map for transforming **normals** alongside a **`Mat4`** point transform.
///
/// Built from the model matrix once per pose (e.g. in **`Shape::transform`**), then applied per facet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NormalTransform(Mat3);

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
mod facet_tests {
    use approx::assert_relative_eq;
    use glam::{Mat4, Vec3};
    use std::f32::consts::FRAC_PI_2;

    use super::{Facet, NormalTransform, UnitVec3};

    const VERTS: [usize; 3] = [1, 2, 7];

    #[test]
    fn transform_updates_facet_normal() {
        let original_normal = Vec3::new(0.0, 0.0, 1.0).into();
        let expected_normal: UnitVec3 = Vec3::new(0.0, -1.0, 0.0).into();
        let facet = Facet::with_facet_normal(VERTS, original_normal);

        let normal_transform = NormalTransform::from_model(Mat4::from_rotation_x(FRAC_PI_2));
        let posed = facet.transform(normal_transform);
        assert_relative_eq!(expected_normal, posed.facet_normal());
    }

    #[test]
    fn transform_updates_vertex_normals() {
        let facet =
            Facet::with_normals(VERTS, UnitVec3::Z, [UnitVec3::X, UnitVec3::Y, UnitVec3::Z]);

        let normal_transform = NormalTransform::from_model(Mat4::from_rotation_x(FRAC_PI_2));
        let posed = facet.transform(normal_transform);
        let expected = [UnitVec3::X, UnitVec3::Z, UnitVec3::NEG_Y];
        for (actual, expected) in posed.vertex_normals().iter().zip(expected) {
            assert_relative_eq!(*actual, expected);
        }
    }

    #[test]
    fn edges_walk_triangle_boundary_in_ccw_order() {
        let facet = Facet::with_facet_normal(VERTS, UnitVec3::Z);
        assert_eq!(
            facet.edges().collect::<Vec<_>>(),
            vec![(1, 2), (2, 7), (7, 1)],
        );
    }

    #[test]
    fn is_front_facing_true_for_neg_z_normal_when_view_is_pos_z() {
        let facet = Facet::with_facet_normal(VERTS, UnitVec3::NEG_Z);
        assert!(facet.is_front_facing(UnitVec3::Z));
    }

    #[test]
    fn is_front_facing_false_for_pos_z_normal_when_view_is_pos_z() {
        let facet = Facet::with_facet_normal(VERTS, UnitVec3::Z);
        assert!(!facet.is_front_facing(UnitVec3::Z));
    }

    #[test]
    fn is_front_facing_false_when_grazing() {
        let facet = Facet::with_facet_normal(VERTS, UnitVec3::X);
        assert!(!facet.is_front_facing(UnitVec3::Z));
    }
}

#[cfg(test)]
mod normal_transform_tests {
    use approx::assert_relative_eq;
    use glam::{Mat4, Vec3};
    use std::f32::consts::FRAC_PI_2;

    use super::{NormalTransform, UnitVec3};

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
