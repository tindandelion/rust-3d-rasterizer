//! Convex quad facet type for [`super::Cube`] hull topology.

use glam::{Mat4, Vec3};

/// One convex hull facet of [`super::Cube`] (paired with **`vertices`** positions in **one** spatial frame—normals rotate with **[`super::Cube::transform`]**, indices stay unchanged).
///
/// **`verts`** winding matches the six quads seeded in **[`super::Cube::default`]** (outward **unit `normal`** per face).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeFace {
    /// Outward **unit normal** for this hull quad ([`CubeFace::transform`] / [`super::Cube::transform`] keep it coherent with **`vertices`** after each matrix).
    normal: Vec3,
    /// Indices into **`vertices`** for this quad (see parent module’s default **`Cube`** layout).
    verts: [usize; 4],
}

impl CubeFace {
    pub fn new(normal: Vec3, verts: [usize; 4]) -> Self {
        Self { normal, verts }
    }

    /// Indices into the parent [`super::Cube`]'s **`vertices`** for this quad (winding matches [`Self::edges`]).
    pub fn verts(&self) -> &[usize; 4] {
        &self.verts
    }

    /// Outward **unit** normal in the same spatial frame as the parent [`super::Cube`]'s **`vertices`**.
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    /// Re-transforms **`normal`** with **`m.transform_vector3`**, normalized; copies **`verts` unchanged**.
    ///
    /// **Note:** rotations + uniform scaling match analytic normals here; arbitrary **non-uniform** scales should eventually use transpose-inverse of **3×3** (explicitly deferred in this codebase).
    pub fn transform(&self, m: Mat4) -> CubeFace {
        CubeFace {
            normal: m.transform_vector3(self.normal).normalize(),
            verts: self.verts,
        }
    }

    /// Four undirected vertex-index pairs (**`verts[k]` ↔ `verts[(k + 1) % 4]`**) following quad winding.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let v = self.verts;
        [(v[0], v[1]), (v[1], v[2]), (v[2], v[3]), (v[3], v[0])].into_iter()
    }

    /// Returns **true** when **`normal`** · **`view_direction`** is **< 0** — outward **`normal`** points
    /// **against** into‑scene **`view_direction`** (strictly toward the default camera), matching
    /// **[`crate::Camera::direction`]** for [`crate::draw_edges`] and [`crate::draw_faces`].
    ///
    /// Grazing faceting (**`dot == 0`**) is **excluded** (not strictly front-facing).
    pub fn is_front_facing(&self, view_direction: Vec3) -> bool {
        self.normal.dot(view_direction) < 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::CubeFace;
    use approx::assert_relative_eq;
    use glam::{Mat4, Vec3};
    use std::f32::consts::FRAC_PI_2;

    const VERTS: [usize; 4] = [1, 2, 3, 4];

    #[test]
    fn transform_updates_normal() {
        let original_normal = Vec3::new(0.0, 0.0, 1.0);
        let expected_normal = Vec3::new(0.0, -1.0, 0.0);
        let face = CubeFace::new(original_normal, VERTS);

        let m = Mat4::from_rotation_x(FRAC_PI_2);
        let transformed_face = face.transform(m);
        assert_relative_eq!(expected_normal, transformed_face.normal);
    }

    #[test]
    fn edges_walk_quad_boundary_in_order() {
        let face = CubeFace::new(Vec3::Z, VERTS);
        assert_eq!(
            face.edges().collect::<Vec<_>>(),
            vec![(1, 2), (2, 3), (3, 4), (4, 1)],
        );
    }

    #[test]
    fn is_front_facing_true_for_neg_z_cap_when_view_is_pos_z() {
        let face = CubeFace::new(Vec3::NEG_Z, VERTS);
        assert!(face.is_front_facing(Vec3::Z));
    }

    #[test]
    fn is_front_facing_false_for_pos_z_cap_when_view_is_pos_z() {
        let face = CubeFace::new(Vec3::Z, VERTS);
        assert!(!face.is_front_facing(Vec3::Z));
    }

    #[test]
    fn is_front_facing_false_when_grazing() {
        let face = CubeFace::new(Vec3::X, VERTS);
        assert!(!face.is_front_facing(Vec3::Z));
    }
}
