//! Platonic **dodecahedron** (**20** verts × **three.js** **`DodecahedronGeometry`** detail **0**) as **[`crate::geometry::Shape`]**.
//!
//! Golden‑ratio coordinates scaled **`× (0.5 /  φ)`** — **`max |x|, |y|, |z| = 0.5`**, same bounding box axis as **[`cube`](crate::shapes::cube)**.
//!
//! We start from Wikipedia **`three.js`** golden‑ratio Cartesian coordinates (max coordinate **φ**), then **`0.5 /  φ`** so **`max |x|, |y|, |z| = 0.5`**. Hull edge length **`1 /  φ² ≈ 0.382`**, triangulation chords **`1 /  φ ≈ 0.618`**.
//!
//! [`Facet::transform`](crate::geometry::Facet::transform) follows **`cube`**’s **non‑uniform scale** caveat on stored normals.

use glam::Vec3;

use crate::geometry::{Facet, Shape, UnitVec3};

/// Indices for **every** planar triangle (**12** pentagons × 3 wedges) lifted from **`three.js`** **`DodecahedronGeometry`** (detail **0**).
const THREE_JS_DETAIL0_TRIANGLES: [[usize; 3]; 36] = [
    [3, 11, 7],
    [3, 7, 15],
    [3, 15, 13],
    [7, 19, 17],
    [7, 17, 6],
    [7, 6, 15],
    [17, 4, 8],
    [17, 8, 10],
    [17, 10, 6],
    [8, 0, 16],
    [8, 16, 2],
    [8, 2, 10],
    [0, 12, 1],
    [0, 1, 18],
    [0, 18, 16],
    [6, 10, 2],
    [6, 2, 13],
    [6, 13, 15],
    [2, 16, 18],
    [2, 18, 3],
    [2, 3, 13],
    [18, 1, 9],
    [18, 9, 11],
    [18, 11, 3],
    [4, 14, 12],
    [4, 12, 0],
    [4, 0, 8],
    [11, 9, 5],
    [11, 5, 19],
    [11, 19, 7],
    [19, 5, 14],
    [19, 14, 4],
    [19, 4, 17],
    [1, 12, 14],
    [1, 14, 5],
    [1, 5, 9],
];

/// Canonical **scaled Platonic vertex table**: every coordinate **`∈ [− 0.5 ,  0.5 ]`** (**same framing as **`cube`**).
fn platonic_scaled_vertices_array() -> [Vec3; 20] {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let r = phi - 1.0;

    #[rustfmt::skip]
        let verts = [
            Vec3::new(-1., -1., -1.),
            Vec3::new(-1., -1.,  1.),
            Vec3::new(-1.,  1., -1.),
            Vec3::new(-1.,  1.,  1.),
            Vec3::new( 1., -1., -1.),
            Vec3::new( 1., -1.,  1.),
            Vec3::new( 1.,  1., -1.),
            Vec3::new( 1.,  1.,  1.),
            Vec3::new( 0., -r, -phi),
            Vec3::new( 0., -r,  phi),
            Vec3::new( 0.,  r, -phi),
            Vec3::new( 0.,  r,  phi),
            Vec3::new(-r, -phi,  0.),
            Vec3::new(-r,  phi,  0.),
            Vec3::new( r, -phi,  0.),
            Vec3::new( r,  phi,  0.),
            Vec3::new(-phi,  0., -r),
            Vec3::new( phi,  0., -r),
            Vec3::new(-phi,  0.,  r),
            Vec3::new( phi,  0.,  r),
        ];
    let scale = 0.5 / phi;
    verts.map(|v| v * scale)
}

/// Default **scaled Platonic dodecahedron** as **`Shape`**: **20** verts, **36** wedge **`Facet`**s (**`three.js`** detail **0** tri list).
///
/// Pose with **`Shape::transform`** ( **`Mat4`** per-corner + **`Facet::transform`** per facet).
pub fn dodecahedron() -> Shape {
    let vertices_arr = platonic_scaled_vertices_array();
    let facets: Vec<Facet> = THREE_JS_DETAIL0_TRIANGLES
        .into_iter()
        .map(|tri| facet_from_corners(&vertices_arr, tri))
        .collect();

    Shape::new(vertices_arr.into_iter().collect(), facets)
}

/// Outward **CCW** facet (left‑handed view along **`UnitVec3`**) matching [`Facet::is_front_facing`].
fn facet_from_corners(vertices: &[Vec3; 20], [i, j, k]: [usize; 3]) -> Facet {
    let a = vertices[i];
    let b = vertices[j];
    let c = vertices[k];
    let e1 = b - a;
    let e2 = c - a;
    let mut n = e1.cross(e2);
    let centroid_tri = (a + b + c) * (1.0 / 3.0);
    if n.dot(centroid_tri) < 0.0 {
        n = -n;
        Facet::with_facet_normal([i, k, j], UnitVec3::from(n))
    } else {
        Facet::with_facet_normal([i, j, k], UnitVec3::from(n))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use approx::assert_relative_eq;

    use crate::{TriMesh, geometry::UnitVec3};

    use super::*;

    #[test]
    fn vertices_match_cube_axis_half_extent() {
        let mesh = dodecahedron();
        let half = 0.5_f32;
        for p in mesh.vertices() {
            assert!(
                p.x.abs() <= half + 1e-4 && p.y.abs() <= half + 1e-4 && p.z.abs() <= half + 1e-4,
                "vertices must stay inside [-0.5, 0.5]^3 like cube ({p:?})",
            );
        }
        let max_coord = mesh.vertices().iter().fold(0_f32, |acc, p| {
            acc.max(p.x.abs().max(p.y.abs()).max(p.z.abs()))
        });
        assert_relative_eq!(max_coord, half, epsilon = 1e-4);
    }

    #[test]
    fn convex_hull_shortest_edge_is_one_over_phi_squared() {
        let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
        let expected_shortest = 1.0 / (phi * phi);
        let d = dodecahedron();
        let mut seen: BTreeMap<(usize, usize), f32> = BTreeMap::new();
        for [i, j, k] in THREE_JS_DETAIL0_TRIANGLES {
            for (u, v) in [(i, j), (j, k), (k, i)] {
                let key = if u < v { (u, v) } else { (v, u) };
                seen.entry(key)
                    .or_insert_with(|| d.vertices()[u].distance(d.vertices()[v]));
            }
        }
        let shortest = seen.values().copied().fold(f32::INFINITY, f32::min);
        assert_relative_eq!(shortest, expected_shortest, epsilon = 1e-4);
    }

    #[test]
    fn visible_facets_count_from_pos_z() {
        let d = dodecahedron();
        assert_eq!(d.visible_facets(UnitVec3::Z).count(), 13);
    }
}
