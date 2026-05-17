//! Axis-aligned **unit cube** (edge length **1**, centered at the origin): eight vertices, twelve edges.

use glam::{Mat4, Vec3};

/// One undirected segment as two **already-transformed** world-space endpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge(pub Vec3, pub Vec3);

/// One square face of the axis-aligned unit cube: outward normal in model space and quad corner indices into [`UNIT_VERTS`].
///
/// Corner order matches a consistent winding so each quad’s edges match [`EDGE_INDICES`] hull connectivity.
///
/// [`CubeFace::transform`] only affects **`normal`**; **`verts`** stay model-space indices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeFace {
    /// Unit outward normal in **model** space for [`FACES`]; after [`CubeFace::transform`], **`normal`** is **world** space.
    pub normal: Vec3,
    /// Quad vertex indices in winding order (always **model** space).
    pub verts: [usize; 4],
}

impl CubeFace {
    /// Maps **`normal`** through **`model_to_world`** via [`Mat4::transform_vector3`], then renormalizes.
    ///
    /// **`verts`** are copied unchanged (still indices into [`UNIT_VERTS`]).
    ///
    /// For **uniform** scale and rotation this matches correct plane normals; **non-uniform** scale needs the
    /// inverse-transpose of the upper **3×3** (deferred until required).
    pub fn transform(&self, model_to_world: Mat4) -> CubeFace {
        CubeFace {
            normal: model_to_world.transform_vector3(self.normal).normalize(),
            verts: self.verts,
        }
    }

    /// **`true`** if **`self.normal`** faces away from view direction **`camera_forward_world`** (**into** the scene).
    ///
    /// **`normal`** must be a **world-space** unit outward normal (e.g. from [`CubeFace::transform`] on a [`FACES`] entry).
    ///
    /// Back-facing means **`n · camera_forward_world < 0`**. **`dot == 0`** (edge-on) is **not** back-facing.
    pub fn is_back(&self, camera_forward_world: Vec3) -> bool {
        self.normal.dot(camera_forward_world) < 0.0
    }
}

/// Corners of the axis-aligned unit cube, edge length **1**, half-extent **0.5**.
/// Ordering matches [`EDGE_INDICES`] hull connectivity and [`FACES`] quads.
const UNIT_VERTS: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, -0.5),
    Vec3::new(0.5, -0.5, -0.5),
    Vec3::new(0.5, 0.5, -0.5),
    Vec3::new(-0.5, 0.5, -0.5),
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(0.5, -0.5, 0.5),
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(-0.5, 0.5, 0.5),
];

/// The six faces of the unit cube ([`UNIT_VERTS`]), axis-aligned in model space.
///
/// Face index order matches [`EDGE_FACE_PAIRS`].
pub const FACES: [CubeFace; 6] = [
    // −Z (back)
    CubeFace {
        normal: Vec3::NEG_Z,
        verts: [0, 3, 2, 1],
    },
    // +Z (front)
    CubeFace {
        normal: Vec3::Z,
        verts: [4, 5, 6, 7],
    },
    // +X (right)
    CubeFace {
        normal: Vec3::X,
        verts: [1, 2, 6, 5],
    },
    // −X (left)
    CubeFace {
        normal: Vec3::NEG_X,
        verts: [0, 4, 7, 3],
    },
    // +Y (top)
    CubeFace {
        normal: Vec3::Y,
        verts: [3, 7, 6, 2],
    },
    // −Y (bottom)
    CubeFace {
        normal: Vec3::NEG_Y,
        verts: [0, 1, 5, 4],
    },
];

/// Hull edges as vertex index pairs; order matches [`EDGE_FACE_PAIRS`].
const EDGE_INDICES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

/// For each hull edge, the two incident face indices into [`FACES`].
const EDGE_FACE_PAIRS: [(usize, usize); 12] = [
    (5, 0),
    (0, 2),
    (0, 4),
    (0, 3),
    (5, 1),
    (2, 1),
    (4, 1),
    (3, 1),
    (5, 3),
    (5, 2),
    (4, 2),
    (4, 3),
];

/// Unit cube (**[`UNIT_VERTS`]**) plus a **`Mat4`** model → world-style transform (`set_transform`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cube {
    transform: Mat4,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            transform: Mat4::IDENTITY,
        }
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }

    /// Hull edges that survive **Option A** back-face filtering: draw iff **not** both adjacent faces are back-facing ([`FACES`], [`EDGE_FACE_PAIRS`]).
    ///
    /// Pass **`camera.direction()`** from [`crate::Camera`] so classification matches the raster camera (see [`crate::wireframe::draw_edges`]).
    pub fn visible_edges(&self, camera_forward_world: Vec3) -> impl Iterator<Item = Edge> + '_ {
        let mut face_back = [false; 6];
        for (i, face) in FACES.iter().enumerate() {
            face_back[i] = face.transform(self.transform).is_back(camera_forward_world);
        }

        EDGE_INDICES
            .iter()
            .copied()
            .zip(EDGE_FACE_PAIRS.iter().copied())
            .filter_map(move |((i, j), (fa, fb))| {
                if face_back[fa] && face_back[fb] {
                    return None;
                }
                Some(Edge(
                    self.transform.transform_point3(UNIT_VERTS[i]),
                    self.transform.transform_point3(UNIT_VERTS[j]),
                ))
            })
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_edges_count_from_front() {
        let cube = Cube::new();
        let forward = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(cube.visible_edges(forward).count(), 12);
    }

    #[test]
    fn visible_edges_count_from_arbitrary_point() {
        let cube = Cube::new();
        let forward = Vec3::new(-1.0, -1.0, -1.0);
        assert_eq!(cube.visible_edges(forward).count(), 9);
    }
}
