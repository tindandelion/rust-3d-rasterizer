//! Axis-aligned **unit cube** (edge length **1**, **`[-½, ½]³`**) built as **[`Shape`](crate::geometry::Shape)**.
//!
//! Use **[`cube`]** plus **[`Shape::transform`](crate::geometry::Shape::transform)** for posing

use glam::Vec3;

use crate::geometry::{Facet, Shape, UnitVec3};

/// Two **`Facet`**s per planar hull quad (same **`normal`**, **`(w,x,y)` + `(w,y,z)`** given CCW verts **`w…z`** seen from outside along **`normal`**).
const fn facets_from_quad_ccw_corner(normal: UnitVec3, verts: [usize; 4]) -> [Facet; 2] {
    let [w, x, y, z] = verts;
    [
        Facet::with_facet_normal([w, x, y], normal),
        Facet::with_facet_normal([w, y, z], normal),
    ]
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
const UNIT_CUBE_QUADS: [(UnitVec3, [usize; 4]); 6] = [
    (UnitVec3::NEG_Z, [0, 3, 2, 1]),
    (UnitVec3::Z, [4, 5, 6, 7]),
    (UnitVec3::X, [1, 2, 6, 5]),
    (UnitVec3::NEG_X, [0, 4, 7, 3]),
    (UnitVec3::Y, [3, 7, 6, 2]),
    (UnitVec3::NEG_Y, [0, 1, 5, 4]),
];

/// Canonical axis-aligned **`[-½, ½]³`** mesh (**eight verts**, twelve wedge **`Facet`**s (**`(w,x,y)` **`(w,y,z)`** per planar quad)).
#[must_use]
pub fn cube() -> Shape {
    let mut facets = Vec::with_capacity(12);
    for &(normal, corners) in &UNIT_CUBE_QUADS {
        let [a, b] = facets_from_quad_ccw_corner(normal, corners);
        facets.push(a);
        facets.push(b);
    }
    Shape::new(UNIT_CUBE_VERTICES.into_iter().collect(), facets)
}

#[cfg(test)]
mod tests {
    use glam::{Mat4, Vec3};

    use super::cube;
    use crate::geometry::UnitVec3;
    use std::f32::consts::FRAC_PI_4;

    #[test]
    fn cube_corner_and_facet_counts() {
        let mesh = cube();
        assert_eq!(mesh.vertices().len(), 8);
        assert_eq!(mesh.facets().len(), 12);
        assert!(
            mesh.vertices()
                .iter()
                .all(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5 && p.z.abs() <= 0.5),
        );
    }

    #[test]
    fn visible_triangles_count_from_front() {
        let mesh = cube();
        let forward = UnitVec3::Z;
        assert_eq!(mesh.visible_triangles(forward).count(), 2);
    }

    #[test]
    fn visible_triangles_count_from_arbitrary_direction() {
        let mesh = cube();
        let forward = Vec3::new(-1.0, -1.0, -1.0).into();
        assert_eq!(mesh.visible_triangles(forward).count(), 6);
    }

    #[test]
    fn visible_triangles_count_after_transform() {
        let forward = UnitVec3::Z;
        let transform = Mat4::from_rotation_x(FRAC_PI_4) * Mat4::from_rotation_y(FRAC_PI_4);

        assert_eq!(
            cube()
                .transform(transform)
                .visible_triangles(forward)
                .count(),
            6,
        );
    }

    #[test]
    fn looking_at_cube_from_front() {
        let mesh = cube();
        let visible = mesh.visible_triangles(UnitVec3::Z).collect::<Vec<_>>();
        assert_eq!(visible.len(), 2);
        assert!(
            visible
                .iter()
                .all(|tri| tri.normals == [UnitVec3::NEG_Z, UnitVec3::NEG_Z, UnitVec3::NEG_Z])
        );
    }
}
