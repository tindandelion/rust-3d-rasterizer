//! Convex quad facet type for [`super::Cube`] hull topology.

use glam::{Mat4, Vec3};

/// One convex hull facet of [`super::Cube`] (paired with **`vertices`** positions in **one** spatial frame—normals rotate with **[`super::Cube::transform`]**, indices stay unchanged).
///
/// **`verts`** follow winding consistent with seeded hull data in the parent module (`EDGE_INDICES` connectivity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeFace {
    /// Outward **unit normal** for this hull quad ([`CubeFace::transform`] / [`super::Cube::transform`] keep it coherent with **`vertices`** after each matrix).
    pub normal: Vec3,
    /// Indices into **`vertices`** for this quad (see parent module’s default **`Cube`** layout).
    pub verts: [usize; 4],
}

impl CubeFace {
    /// Re-transforms **`normal`** with **`m.transform_vector3`**, normalized; copies **`verts` unchanged**.
    ///
    /// **Note:** rotations + uniform scaling match analytic normals here; arbitrary **non-uniform** scales should eventually use transpose-inverse of **3×3** (explicitly deferred in this codebase).
    pub fn transform(&self, m: Mat4) -> CubeFace {
        CubeFace {
            normal: m.transform_vector3(self.normal).normalize(),
            verts: self.verts,
        }
    }

    /// Four undirected vertex-index pairs (**`verts[k]` ↔ `verts[(k + 1) % 4]`**) following quad winding.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let v = self.verts;
        [(v[0], v[1]), (v[1], v[2]), (v[2], v[3]), (v[3], v[0])].into_iter()
    }

    /// Returns **true** when **`normal`** · **`view_direction`** is **< 0** (facet purely back‑facing versus “into‑scene **`view_direction`**”; matches **[`crate::Camera::direction`]** payloads used by **`crate::wireframe`**).
    ///
    /// Grazing normals (**`dot == 0`**), including edge-on cues, classify as **not** back-facing.
    pub(super) fn is_back(&self, view_direction: Vec3) -> bool {
        self.normal.dot(view_direction) < 0.0
    }
}
