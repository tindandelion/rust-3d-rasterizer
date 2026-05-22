//! Platonic **dodecahedron**: golden‑ratio **`three.js`** coordinates scaled **`× (0.5 /  φ)`** — **`max |x|, |y|, |z| = 0.5`**, matching **`[−0.5, 0.5]³`** (**same as **[`Cube::default`](crate::scene::cube::Cube)** with edge length 1).
//!
//! Twenty **`glam::Vec3`** **vertices**, **36** **[`crate::scene::facet::Facet`]** wedges (**three** planar triangles per pentagonal hull face—the **`three.js`** `DodecahedronGeometry` detail‑0 triangle list, numbering unchanged).
//!
//! We start from **Wikipedia / `three.js`** golden‑ratio Cartesian coordinates (max coordinate **φ**), then apply **uniform `0.5 /  φ`**, so **`max |x|, |y|, |z| = 0.5`**. **Platonic** hull edge length is **`1 /  φ² ≈ 0.382 `**; triangulation chords are **`1 /  φ ≈ 0.618 `**.
//!
//! [`Facet::transform`](crate::scene::facet::Facet::transform) follows the **`Cube`** **non‑uniform scale** caveat on stored normals.

use glam::{Mat4, Vec3};
use std::array;

use super::facet::Facet;
use crate::{TriMesh, Triangle, geometry::Normal3};

/// Indices for **every** planar triangle (**12** pentagons × 3 wedges) lifted from **`three.js`** **`DodecahedronGeometry`** (detail 0).
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

/// **Platonic dodecahedron** implementing [`TriMesh`]—feeds **`draw_faces`** ([`Triangle`]).
///
/// [`Default`] calls [`Self::unit_vertices`] plus the bundled **`three.js`** `DodecahedronGeometry` wedge indices (**detail 0**).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dodecahedron {
    pub vertices: [Vec3; 20],
    pub faces: [Facet; 36],
}

impl Dodecahedron {
    /// Applies **`m.transform_point3`** per vertex plus [`Facet::transform`](crate::scene::facet::Facet::transform) per wedge (composition matches **`Cube`**).
    pub fn transform(&self, m: Mat4) -> Dodecahedron {
        Dodecahedron {
            vertices: array::from_fn(|i| m.transform_point3(self.vertices[i])),
            faces: array::from_fn(|i| self.faces[i].transform(m)),
        }
    }

    /// Golden‑ratio **`three.js`** vertex table (**Wikipedia**) scaled by **`0.5 /  φ`** so **every coordinate lies in [− 0.5 ,  0.5 ]** (**same bounding box radius as **`Cube`** half‑extent** along **x / y / z**).
    pub fn unit_vertices() -> [Vec3; 20] {
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
}

impl TriMesh for Dodecahedron {
    fn visible_facets(&self, view_direction: Normal3) -> impl Iterator<Item = Triangle> + '_ {
        self.faces
            .iter()
            .filter(move |f| f.is_front_facing(view_direction))
            .map(|facet| {
                let idx = facet.verts();
                Triangle {
                    corners: array::from_fn(|k| self.vertices[idx[k]]),
                    normal: facet.normal(),
                }
            })
    }
}

impl Default for Dodecahedron {
    fn default() -> Self {
        let vertices = Dodecahedron::unit_vertices();
        let faces =
            array::from_fn(|fi| facet_from_corners(&vertices, THREE_JS_DETAIL0_TRIANGLES[fi]));
        Self { vertices, faces }
    }
}

/// Outward **CCW** facet (left‑handed view along **`Normal3`**) matching [`Facet::is_front_facing`].
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
        Facet::new(Normal3::from(n), [i, k, j])
    } else {
        Facet::new(Normal3::from(n), [i, j, k])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use approx::assert_relative_eq;

    use super::*;
    use crate::geometry::Normal3;

    #[test]
    fn vertices_match_cube_axis_half_extent() {
        let half = 0.5_f32;
        let v = Dodecahedron::unit_vertices();
        for p in v {
            assert!(
                p.x.abs() <= half + 1e-4 && p.y.abs() <= half + 1e-4 && p.z.abs() <= half + 1e-4,
                "vertices must stay inside [-0.5, 0.5]^3 like Cube::default ({p:?})",
            );
        }
        let max_coord = v.iter().fold(0_f32, |acc, p| {
            acc.max(p.x.abs().max(p.y.abs()).max(p.z.abs()))
        });
        assert_relative_eq!(max_coord, half, epsilon = 1e-4);
    }

    #[test]
    fn convex_hull_shortest_edge_is_one_over_phi_squared() {
        let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
        let expected_shortest = 1.0 / (phi * phi);
        let d = Dodecahedron::default();
        let mut seen: BTreeMap<(usize, usize), f32> = BTreeMap::new();
        for [i, j, k] in THREE_JS_DETAIL0_TRIANGLES {
            for (u, v) in [(i, j), (j, k), (k, i)] {
                let key = if u < v { (u, v) } else { (v, u) };
                seen.entry(key)
                    .or_insert_with(|| d.vertices[u].distance(d.vertices[v]));
            }
        }
        let shortest = seen.values().copied().fold(f32::INFINITY, f32::min);
        assert_relative_eq!(shortest, expected_shortest, epsilon = 1e-4);
    }

    #[test]
    fn visible_facets_count_from_pos_z() {
        let d = Dodecahedron::default();
        assert_eq!(d.visible_facets(Normal3::Z).count(), 13);
    }
}
