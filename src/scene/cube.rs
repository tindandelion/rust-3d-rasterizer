//! Axis-aligned **unit cube** (edge length **1**, centered at the origin, half-extent **0.5**).
//!
//! A [`Cube`] holds eight corner positions plus six [`CubeFace`] records (matching normals and quad indices)—no separate model matrix is stored.
//! [`Cube::transform`] repacks both arrays; [`Cube::visible_edges`] applies **Option A** silhouette filtering (emit a hull edge unless **both** adjacent facets face **away**—compare each stored **`normal`** with [`crate::Camera::direction`], which follows the **`+Z` forward**, **into‑scene** convention used elsewhere in this crate).
//!
//! Planning context for ordering and milestones: `doc/planning/project-spec.md` and `doc/planning/project-breakdown.md`.

mod face;

use glam::{Mat4, Vec3};
use std::{array, collections::HashSet};

pub use face::CubeFace;

/// Undirected segment between two points in the same frame as the parent [`Cube`]'s `vertices` positions (exported wireframe treats posed cubes as world-space endpoints).
///
/// **`0` / `1`**: unordered endpoints (`glam::Vec3` each).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge(pub Vec3, pub Vec3);

/// Axis-aligned boxed corners plus template facet metadata ready for posing via [`Cube::transform`].
///
/// **Defaults:** [`Cube::default`] ⇒ corners equal **`UNIT_VERTS`**, normals/quads seeded from **`UNIT_FACES`** (**identity** posture before posing).
///
/// **Typical exporter path:** raster [`Cube::visible_edges`] through **`crate::wireframe::draw_edges`** with [`crate::Camera`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cube {
    pub vertices: [Vec3; 8],
    pub faces: [CubeFace; 6],
}

impl Cube {
    /// Returns a copy with each corner multiplied by **`m.transform_point3`** and each **`faces`** slot updated via [`CubeFace::transform`] (**corner indices **`verts`** stay **0 … 7** throughout).
    ///
    /// **Composition:** chaining maps **`cube.transform(A).transform(B)`** ⇒ **`vertices` become `|v| ↦ B · (A · v)`** (matching column-vector math / outside-in **`Mat4` multiplication** semantics used by callers).
    pub fn transform(&self, m: Mat4) -> Cube {
        Cube {
            vertices: array::from_fn(|i| m.transform_point3(self.vertices[i])),
            faces: array::from_fn(|i| self.faces[i].transform(m)),
        }
    }

    /// Yields **[`Edge`]** segments under **Option A**: omit an edge only when **both** incident **`faces`** are strictly **back‑facing** vs **`camera_forward_world`** (**`facet_normal · camera_forward_world < 0`** test per stored facet **`normal`).
    ///
    /// Edge↔facet incidence follows **`EDGE_INDICES`** × **`EDGE_FACE_PAIRS`** (private tables kept parallel to **`faces`** row order seeded from **`UNIT_FACES`** documentation).
    ///
    /// Prefer passing [`crate::Camera::direction`] unchanged so [`crate::wireframe::draw_edges`] stays coherent with camera math.
    pub fn visible_edges(&self, view_direction: Vec3) -> impl Iterator<Item = Edge> + '_ {
        let mut seen = HashSet::new();
        for face in self.faces.iter() {
            if face.is_back(view_direction) {
                continue;
            }
            for (a, b) in face.edges() {
                let edge = if a < b { (a, b) } else { (b, a) };
                seen.insert(edge);
            }
        }
        seen.into_iter()
            .map(|(i, j)| Edge(self.vertices[i], self.vertices[j]))
    }
}

impl Default for Cube {
    /// Identity-posture boxed cube
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

        let faces: [CubeFace; 6] = [
            CubeFace {
                normal: Vec3::NEG_Z,
                verts: [0, 3, 2, 1],
            },
            CubeFace {
                normal: Vec3::Z,
                verts: [4, 5, 6, 7],
            },
            CubeFace {
                normal: Vec3::X,
                verts: [1, 2, 6, 5],
            },
            CubeFace {
                normal: Vec3::NEG_X,
                verts: [0, 4, 7, 3],
            },
            CubeFace {
                normal: Vec3::Y,
                verts: [3, 7, 6, 2],
            },
            CubeFace {
                normal: Vec3::NEG_Y,
                verts: [0, 1, 5, 4],
            },
        ];

        Self { vertices, faces }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use super::*;

    #[test]
    fn visible_edges_count_from_front() {
        let cube = Cube::default();
        let forward = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(cube.visible_edges(forward).count(), 12);
    }

    #[test]
    fn visible_edges_count_from_arbitrary_point() {
        let cube = Cube::default();
        let forward = Vec3::new(-1.0, -1.0, -1.0);
        assert_eq!(cube.visible_edges(forward).count(), 9);
    }

    #[test]
    fn visible_edges_count_after_transform() {
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let transform = Mat4::from_rotation_x(FRAC_PI_4) * Mat4::from_rotation_y(FRAC_PI_4);

        assert_eq!(
            Cube::default()
                .transform(transform)
                .visible_edges(forward)
                .count(),
            9,
        );
    }
}
