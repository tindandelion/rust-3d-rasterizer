//! Axis-aligned **unit cube** (edge length **1**, **`[-½, ½]³`**) built as **`[Shape](crate::scene::shape::Shape)`**.
//!
//! Use **[`unit_cube`]** plus **[`Shape::transform`](crate::scene::shape::Shape::transform)** for posing
//! (**`Facet::transform`** / **`TriMesh::visible_facets`** — same **`Camera`** +**Z**‑forward semantics as rest of **`scene`**).
//!
//! Planning: `doc/planning/project-spec.md`, `doc/planning/project-breakdown.md`.

use glam::Vec3;

use super::facet::Facet;
use super::shape::Shape;

use crate::geometry::Normal3;

/// Two **`Facet`**s per planar hull quad (same **`normal`**, **`(w,x,y)` + `(w,y,z)`** given CCW verts **`w…z`** seen from outside along **`normal`**).
const fn facets_from_quad_ccw_corner(normal: Normal3, verts: [usize; 4]) -> [Facet; 2] {
    let [w, x, y, z] = verts;
    [Facet::new(normal, [w, x, y]), Facet::new(normal, [w, y, z])]
}

const UNIT_CUBE_VERTICES: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, -0.5),
    Vec3::new(0.5, -0.5, -0.5),
    Vec3::new(0.5, 0.5, -0.5),
    Vec3::new(-0.5, 0.5, -0.5),
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(0.5, -0.5, 0.5),
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(-0.5, 0.5, 0.5),
];

// Six hull quads (**CCW**) from historical **`cube`** vertex layout (**one normal + four corners**).
const UNIT_CUBE_QUADS: [(Normal3, [usize; 4]); 6] = [
    (Normal3::NEG_Z, [0, 3, 2, 1]),
    (Normal3::Z, [4, 5, 6, 7]),
    (Normal3::X, [1, 2, 6, 5]),
    (Normal3::NEG_X, [0, 4, 7, 3]),
    (Normal3::Y, [3, 7, 6, 2]),
    (Normal3::NEG_Y, [0, 1, 5, 4]),
];

/// Canonical axis-aligned **`[-½, ½]³`** mesh (**eight verts**, twelve wedge **`Facet`**s (**`(w,x,y)` **`(w,y,z)`** per planar quad)).
#[must_use]
pub fn unit_cube() -> Shape {
    let mut faces = Vec::with_capacity(12);
    for &(normal, corners) in &UNIT_CUBE_QUADS {
        let [a, b] = facets_from_quad_ccw_corner(normal, corners);
        faces.push(a);
        faces.push(b);
    }
    Shape::new(UNIT_CUBE_VERTICES.into_iter().collect(), faces)
}

#[cfg(test)]
mod tests {
    use glam::{Mat4, Vec3};

    use super::unit_cube;
    use crate::{TriMesh, geometry::Normal3};
    use std::f32::consts::FRAC_PI_4;

    #[test]
    fn unit_cube_corner_and_face_counts() {
        let mesh = unit_cube();
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 12);
        assert!(
            mesh.vertices
                .iter()
                .all(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5 && p.z.abs() <= 0.5),
        );
    }

    #[test]
    fn visible_facets_count_from_front() {
        let mesh = unit_cube();
        let forward = Normal3::Z;
        assert_eq!(mesh.visible_facets(forward).count(), 2);
    }

    #[test]
    fn visible_facets_count_from_arbitrary_direction() {
        let mesh = unit_cube();
        let forward = Vec3::new(-1.0, -1.0, -1.0).into();
        assert_eq!(mesh.visible_facets(forward).count(), 6);
    }

    #[test]
    fn visible_facets_count_after_transform() {
        let forward = Normal3::Z;
        let transform = Mat4::from_rotation_x(FRAC_PI_4) * Mat4::from_rotation_y(FRAC_PI_4);

        assert_eq!(
            unit_cube()
                .transform(transform)
                .visible_facets(forward)
                .count(),
            6,
        );
    }

    #[test]
    fn looking_at_cube_from_front() {
        let mesh = unit_cube();
        let visible = mesh.visible_facets(Normal3::Z).collect::<Vec<_>>();
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|tri| tri.normal == Normal3::NEG_Z));
    }
}
