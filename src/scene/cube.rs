//! Axis-aligned **unit cube** (edge length **1**, centered at the origin, half-extent **0.5**).
//!
//! A [`Cube`] stores eight **`vertices`** and **twelve** **[`crate::scene::facet::Facet`]** records (**two** CCW wedges per seeded hull quad **`(w,x,y)` **`(w,y,z)`**, see **`[`Default`](#impl-Default-for-Cube)`**).
//!
//! **Facet** **transform** / **front-facing** drive **`Cube::transform`** and **`Cube::visible_facets`** (into-scene view — match **`Camera`** +Z forward).
//!
//! Planning: `doc/planning/project-spec.md`, `doc/planning/project-breakdown.md`.
use glam::{Mat4, Vec3};
use std::array;

use super::facet::Facet;

use crate::geometry::Normal3;

/// One strictly front-filled **triangle** in world space: **`corners`** + outward **facet** **[`Normal3`]**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub corners: [Vec3; 3],
    pub normal: Normal3,
}

/// Axis-aligned box corners plus triangular facet topology ready for posing via [`Cube::transform`].
///
/// **`Default`** seeds **twelve **`Facet`**s** (**two per hull quad**) with **`0…7` vertex indices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cube {
    pub vertices: [Vec3; 8],
    pub faces: [Facet; 12],
}

/// Two **`Facet`**s per planar hull quad (same **`normal`**, **`(w,x,y)`** + **`(w,y,z)`** given CCW verts **`w…z`** seen from outside along **`normal`**).
fn facets_from_quad_ccw_corner(normal: Normal3, verts: [usize; 4]) -> [Facet; 2] {
    let [w, x, y, z] = verts;
    [Facet::new(normal, [w, x, y]), Facet::new(normal, [w, y, z])]
}

impl Cube {
    /// Returns a copy with each corner multiplied by **`m.transform_point3`** and each facet updated via **[`Facet::transform`]**.
    ///
    /// **Composition:** chaining maps **`cube.transform(A).transform(B)`** ⇒ **`vertices` become `|v| ↦ B · (A · v)`** (column-vector **`Mat4`** semantics matching callers).
    pub fn transform(&self, m: Mat4) -> Cube {
        Cube {
            vertices: array::from_fn(|i| m.transform_point3(self.vertices[i])),
            faces: array::from_fn(|i| self.faces[i].transform(m)),
        }
    }

    /// One **`Triangle`** per visible **`Facet`** (world **`corners`** plus that facet’s **`normal`**).
    pub fn visible_facets(&self, view_direction: Normal3) -> impl Iterator<Item = Triangle> + '_ {
        self.faces
            .iter()
            .filter(move |f| f.is_front_facing(view_direction))
            .map(|facet| {
                let v = facet.verts();
                Triangle {
                    corners: array::from_fn(|i| self.vertices[v[i]]),
                    normal: facet.normal(),
                }
            })
    }
}

impl Default for Cube {
    fn default() -> Self {
        let vertices: [Vec3; 8] = [
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ];

        /// Six hull quads (**CCW**) from **`Cube`** historical layout (**one normal + four verts**).
        const QUADS: [(Normal3, [usize; 4]); 6] = [
            (Normal3::NEG_Z, [0, 3, 2, 1]),
            (Normal3::Z, [4, 5, 6, 7]),
            (Normal3::X, [1, 2, 6, 5]),
            (Normal3::NEG_X, [0, 4, 7, 3]),
            (Normal3::Y, [3, 7, 6, 2]),
            (Normal3::NEG_Y, [0, 1, 5, 4]),
        ];

        let faces: [Facet; 12] =
            array::from_fn(|i| facets_from_quad_ccw_corner(QUADS[i / 2].0, QUADS[i / 2].1)[i % 2]);

        Self { vertices, faces }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use super::*;
    use glam::Vec3;

    #[test]
    fn visible_facets_count_from_front() {
        let cube = Cube::default();
        let forward = Normal3::Z;
        assert_eq!(cube.visible_facets(forward).count(), 2);
    }

    #[test]
    fn visible_facets_count_from_arbitrary_direction() {
        let cube = Cube::default();
        let forward = Vec3::new(-1.0, -1.0, -1.0).into();
        assert_eq!(cube.visible_facets(forward).count(), 6);
    }

    #[test]
    fn visible_facets_count_after_transform() {
        let forward = Normal3::Z;
        let transform = Mat4::from_rotation_x(FRAC_PI_4) * Mat4::from_rotation_y(FRAC_PI_4);

        assert_eq!(
            Cube::default()
                .transform(transform)
                .visible_facets(forward)
                .count(),
            6,
        );
    }

    #[test]
    fn looking_at_cube_from_front() {
        let cube = Cube::default();
        let look_along_z_axis = Normal3::Z;

        let visible = cube.visible_facets(look_along_z_axis).collect::<Vec<_>>();
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|tri| tri.normal == Normal3::NEG_Z));
    }
}
