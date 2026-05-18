//! Axis-aligned **unit cube** (edge length **1**, centered at the origin, half-extent **0.5**).
//!
//! A [`Cube`] holds eight corner positions plus six [`CubeFace`] records (matching normals and quad indices)—no separate model matrix is stored.
//! [`Cube::transform`] repacks both arrays; [`Cube::visible_edges`] applies **Option A** silhouette filtering (emit a hull edge unless **both** adjacent facets face **away**). [`Cube::visible_faces`] yields each **`(faces` slot `0 … 5`, [`Quad`] corners)** hull facet that is **not** strictly back‑facing (same **`view_direction`** / **normal · view** rule as [`CubeFace::is_back`]).
//! Classify faceting using each stored **`normal`** against the **into‑scene** view vector—typically [`crate::Camera::direction`] (**`+Z` forward**, left‑handed scene convention used elsewhere in this crate).
//!
//! Planning context for ordering and milestones: `doc/planning/project-spec.md` and `doc/planning/project-breakdown.md`.

mod face;

use glam::{Mat4, Vec3};
use std::{array, collections::HashSet};

use face::CubeFace;

/// Undirected segment between two points in the same frame as the parent [`Cube`]'s `vertices` positions (exported wireframe treats posed cubes as world-space endpoints).
///
/// **`0` / `1`**: unordered endpoints (`glam::Vec3` each).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge(pub Vec3, pub Vec3);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad(pub Vec3, pub Vec3, pub Vec3, pub Vec3);

/// Axis-aligned box corners plus template facet metadata ready for posing via [`Cube::transform`].
///
/// **Defaults:** [`Cube::default`] seeds the eight **`±0.5`** corners and six outward‑normal quads (**identity** posture before posing).
///
/// **Typical exporter path:** raster [`Cube::visible_edges`] through **[`crate::draw_edges`]**, or fill from [`Cube::visible_faces`] (**`.map(|(_, q)| q)`** when you only need [`Quad`] corners), with [`crate::Camera`].
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

    /// Yields **[`Edge`]** segments under **Option A**: omit an edge only when **both** incident faces are strictly **back‑facing** vs **`view_direction`** (**`facet_normal · view_direction < 0`** per [`CubeFace`], using each stored facet **`normal`).
    ///
    /// **Implementation:** take every face from [`Self::iter_visible_faces`], append its quad boundary pairs from [`CubeFace::edges`], dedupe undirected hull edges with canonical **`(min(i,j), max(i,j))`** keys, then pair **`vertices`** endpoints.
    ///
    /// Prefer passing [`crate::Camera::direction`] unchanged so [`crate::draw_edges`] stays coherent with camera math.
    pub fn visible_edges(&self, view_direction: Vec3) -> impl Iterator<Item = Edge> + '_ {
        let mut seen = HashSet::new();
        for face in self.iter_visible_faces(view_direction) {
            for (a, b) in face.edges() {
                let edge = if a < b { (a, b) } else { (b, a) };
                seen.insert(edge);
            }
        }
        seen.into_iter()
            .map(|(i, j)| Edge(self.vertices[i], self.vertices[j]))
    }

    /// Yields **`(slot, quad)`** for each visible hull facet. **`slot`** is **`Self::faces`** index **`0 … 5`** (order fixed by [`Cube::default`], unchanged by [`Cube::transform`]); **`quad`** is four **[`Vec3`]** corners in facet winding (**same back‑face test** as [`Self::visible_edges`]). [`crate::CUBE_FACE_PALETTE`] / [`crate::draw_faces`] use **`slot`** for flat tints.
    ///
    /// Prefer passing [`crate::Camera::direction`] unchanged alongside [`Self::visible_edges`] for consistent BF culling.
    pub fn visible_faces(&self, view_direction: Vec3) -> impl Iterator<Item = (usize, Quad)> + '_ {
        self.faces
            .iter()
            .enumerate()
            .filter(move |(_, face)| !face.is_back(view_direction))
            .map(|(idx, face)| {
                let v = face.verts();
                (
                    idx,
                    Quad(
                        self.vertices[v[0]],
                        self.vertices[v[1]],
                        self.vertices[v[2]],
                        self.vertices[v[3]],
                    ),
                )
            })
    }

    fn iter_visible_faces(&self, view_direction: Vec3) -> impl Iterator<Item = &CubeFace> + '_ {
        self.faces
            .iter()
            .filter(move |face| !face.is_back(view_direction))
    }
}

impl Default for Cube {
    /// Identity-pose unit cube (see struct docs).
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
            CubeFace::new(Vec3::NEG_Z, [0, 3, 2, 1]),
            CubeFace::new(Vec3::Z, [4, 5, 6, 7]),
            CubeFace::new(Vec3::X, [1, 2, 6, 5]),
            CubeFace::new(Vec3::NEG_X, [0, 4, 7, 3]),
            CubeFace::new(Vec3::Y, [3, 7, 6, 2]),
            CubeFace::new(Vec3::NEG_Y, [0, 1, 5, 4]),
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

    #[test]
    fn visible_faces_count_from_front() {
        let cube = Cube::default();
        let forward = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(cube.visible_faces(forward).count(), 5);
    }

    #[test]
    fn visible_faces_count_from_arbitrary_direction() {
        let cube = Cube::default();
        let forward = Vec3::new(-1.0, -1.0, -1.0);
        assert_eq!(cube.visible_faces(forward).count(), 3);
    }

    #[test]
    fn visible_faces_count_after_transform() {
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let transform = Mat4::from_rotation_x(FRAC_PI_4) * Mat4::from_rotation_y(FRAC_PI_4);

        assert_eq!(
            Cube::default()
                .transform(transform)
                .visible_faces(forward)
                .count(),
            3,
        );
    }

    #[test]
    fn visible_faces_corners_match_default_cube_layout() {
        let cube = Cube::default();
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let faces: Vec<Quad> = cube.visible_faces(forward).map(|(_, q)| q).collect();
        assert!(faces.iter().any(|f| {
            f.0 == cube.vertices[0]
                && f.1 == cube.vertices[3]
                && f.2 == cube.vertices[2]
                && f.3 == cube.vertices[1]
        }));
    }
}
