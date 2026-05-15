//! Axis-aligned **unit cube** (edge length **1**, centered at the origin): eight vertices, twelve edges.

use glam::{Mat4, Vec3};

/// One undirected segment as two **already-transformed** model-space endpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge(pub Vec3, pub Vec3);

/// Unit cube (**[`UNIT_VERTS`]**) plus a **`Mat4`** model → world-style transform (`set_transform`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cube {
    transform: Mat4,
}

/// Corners of the axis-aligned unit cube, edge length **1**, half-extent **0.5**.
/// Ordering matches connectivity used by [`Cube::edges`].
pub const UNIT_VERTS: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, -0.5),
    Vec3::new(0.5, -0.5, -0.5),
    Vec3::new(0.5, 0.5, -0.5),
    Vec3::new(-0.5, 0.5, -0.5),
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(0.5, -0.5, 0.5),
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(-0.5, 0.5, 0.5),
];

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

impl Cube {
    pub fn new() -> Self {
        Self {
            transform: Mat4::IDENTITY,
        }
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }

    /// The twelve cube edges **after** applying this cube's transform to **`UNIT_VERTS`**.
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        EDGE_INDICES.iter().copied().map(move |(i, j)| {
            Edge(
                self.transform.transform_point3(UNIT_VERTS[i]),
                self.transform.transform_point3(UNIT_VERTS[j]),
            )
        })
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}
