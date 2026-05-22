//! Indexed triangle mesh backed by **`Vec<glam::Vec3>`** + **`Vec<Facet>`**.
//!
//! Construction takes arbitrary **vertex positions** + **facet list** (**CCW**, outward **[`Facet::normal`](crate::scene::facet::Facet::normal)**); **[`Shape::transform`](Shape::transform)** poses like **`Cube`** / **`Dodecahedron`** (**[`Facet::transform`](crate::scene::facet::Facet::transform)** per face).

use glam::{Mat4, Vec3};
use std::array;

use super::facet::Facet;

use crate::{TriMesh, Triangle, geometry::Normal3};

/// Generic mesh: **`vertices[k]`** positions, **`Facet::verts`** index into **`vertices`**.
///
/// Caller must ensure indices are valid; logic matches **[`crate::scene::cube::Cube`]**.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Facet>,
}

impl Shape {
    pub fn new(vertices: Vec<Vec3>, faces: Vec<Facet>) -> Self {
        Self { vertices, faces }
    }

    /// Applies **`Mat4::transform_point3`** per vertex and **[`Facet::transform`]** per facet (composition matches **`Cube`**).
    pub fn transform(&self, m: Mat4) -> Shape {
        Shape {
            vertices: self
                .vertices
                .iter()
                .copied()
                .map(|v| m.transform_point3(v))
                .collect(),
            faces: self.faces.iter().map(|f| f.transform(m)).collect(),
        }
    }
}

impl TriMesh for Shape {
    fn visible_facets(&self, view_direction: Normal3) -> impl Iterator<Item = Triangle> + '_ {
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

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;
    use glam::Mat4;

    /// **`z = 0`**, **`[-½, ½]²`** in **XY**. Two triangles, outward **[`Normal3::NEG_Z`]** —
    /// visible when **into‑scene** view is **`+Z`** (same rule as **`Cube`** fronts vs
    /// [`Camera::direction`](crate::Camera::direction)).
    fn flat_square_xy() -> Shape {
        #[rustfmt::skip]
        let vertices = vec![
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new( 0.5, -0.5, 0.0),
            Vec3::new( 0.5,  0.5, 0.0),
            Vec3::new(-0.5,  0.5, 0.0),
        ];
        // CCW winding viewed from **`−Z`** (outside along **`Normal3::NEG_Z`**).
        let faces = vec![
            Facet::new(Normal3::NEG_Z, [0, 2, 1]),
            Facet::new(Normal3::NEG_Z, [0, 3, 2]),
        ];
        Shape::new(vertices, faces)
    }

    #[test]
    fn from_pos_z_both_faces_visible_with_neg_z_normals() {
        let shape = flat_square_xy();
        let visible = shape.visible_facets(Normal3::Z).collect::<Vec<_>>();
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|tri| tri.normal == Normal3::NEG_Z));
    }

    #[test]
    fn from_neg_z_no_faces_visible() {
        assert_eq!(flat_square_xy().visible_facets(Normal3::NEG_Z).count(), 0);
    }

    #[test]
    fn perpendicular_view_is_grazing_neither_triangle_front() {
        let shape = flat_square_xy();
        assert_eq!(shape.visible_facets(Normal3::X).count(), 0);
        assert_eq!(shape.visible_facets(Normal3::Y).count(), 0);
    }

    #[test]
    fn pi_rotation_y_swaps_which_direction_sees_square() {
        let shape = flat_square_xy();
        let flipped = shape.transform(Mat4::from_rotation_y(PI));

        assert_eq!(flipped.visible_facets(Normal3::Z).count(), 0);
        assert_eq!(flipped.visible_facets(Normal3::NEG_Z).count(), 2);

        assert_eq!(
            shape.visible_facets(Normal3::Z).count(),
            flipped.visible_facets(Normal3::NEG_Z).count(),
        );
    }
}
