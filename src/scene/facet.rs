//! Triangular hull facet (**CCW winding** vertex indices viewed from outside along the outward [`Normal3`]).
//!
//! Stored indices reference a parent mesh **`vertices`** (**[`Cube`](crate::scene::cube::Cube)** / **[`Dodecahedron`](crate::scene::dodecahedron::Dodecahedron)** carry **`vertices`** + **`Facet`**s)—**Facet** stays mesh‑agnostic and **does not** embed positions.

use glam::Mat4;

use crate::geometry::Normal3;

/// One planar triangle: three **CCW** vertex indices (**`verts`**) into a mesh **`vertices`**, plus outward **unit** **[`Normal3`]** (**same spatial frame** as **`vertices`**).
///
/// **`verts[k]` ↔ `verts[(k + 1) % 3]`** (**k = 0, 1, 2**) traverses boundary **counter‑clockwise** when looking from outside along **`normal`** toward the facet interior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Facet {
    /// Outward **unit** normal (**[`Facet::transform`]** keeps **`normal`** coherent with **`vertices`** after each matrix—same caveat as **`Cube`** **`transform`** **for non‑uniform scales**).
    normal: Normal3,
    /// Indices into the parent mesh **`vertices`** (CCW winding as seen against **`normal`**).
    verts: [usize; 3],
}

impl Facet {
    pub const fn new(normal: Normal3, verts: [usize; 3]) -> Self {
        Self { normal, verts }
    }

    /// Indices into the parent **`vertices`** (winding follows [`Self::edges`] CCW traversal).
    pub fn verts(&self) -> &[usize; 3] {
        &self.verts
    }

    /// Outward unit normal in **`vertices`** space.
    pub fn normal(&self) -> Normal3 {
        self.normal
    }

    /// Re-transforms **`normal`** with **`m.transform_vector3`**, normalized; copies **`verts` unchanged**.
    pub fn transform(&self, m: Mat4) -> Facet {
        Facet {
            normal: m.transform_vector3(self.normal.into()).into(),
            verts: self.verts,
        }
    }

    /// Three undirected vertex-index pairs (**`verts[k]` ↔ `verts[(k + 1) % 3]`**) following triangular CCW winding.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let v = self.verts;
        [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])].into_iter()
    }

    /// **`normal` · `view_direction` < 0** for **into‑scene** view (**[`crate::Camera::direction`]**)—mirrors **`Cube`** wireframe classification.
    ///
    /// Grazing (**`dot == 0`**) is **not** front-facing.
    pub fn is_front_facing(&self, view_direction: Normal3) -> bool {
        view_direction.dot(self.normal) < 0.0
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::{Mat4, Vec3};
    use std::f32::consts::FRAC_PI_2;

    use crate::geometry::Normal3;

    use super::Facet;

    const VERTS: [usize; 3] = [1, 2, 7];

    // Tracer bullets modeled on **`scene::cube`** historical quad-face classifier tests.

    #[test]
    fn transform_updates_normal() {
        let original_normal = Vec3::new(0.0, 0.0, 1.0).into();
        let expected_normal: Normal3 = Vec3::new(0.0, -1.0, 0.0).into();
        let facet = Facet::new(original_normal, VERTS);

        let m = Mat4::from_rotation_x(FRAC_PI_2);
        let posed = facet.transform(m);
        assert_relative_eq!(expected_normal, posed.normal());
    }

    #[test]
    fn edges_walk_triangle_boundary_in_ccw_order() {
        let facet = Facet::new(Normal3::Z, VERTS);
        assert_eq!(
            facet.edges().collect::<Vec<_>>(),
            vec![(1, 2), (2, 7), (7, 1)],
        );
    }

    #[test]
    fn is_front_facing_true_for_neg_z_normal_when_view_is_pos_z() {
        let facet = Facet::new(Normal3::NEG_Z, VERTS);
        assert!(facet.is_front_facing(Normal3::Z));
    }

    #[test]
    fn is_front_facing_false_for_pos_z_normal_when_view_is_pos_z() {
        let facet = Facet::new(Normal3::Z, VERTS);
        assert!(!facet.is_front_facing(Normal3::Z));
    }

    #[test]
    fn is_front_facing_false_when_grazing() {
        let facet = Facet::new(Normal3::X, VERTS);
        assert!(!facet.is_front_facing(Normal3::Z));
    }
}
