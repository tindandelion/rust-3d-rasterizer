//! Indexed triangle mesh backed by **`Vec<glam::Vec3>`** + **`Vec<Facet>`**.
//!
//! Construction takes arbitrary **vertex positions** + **facet list** (**CCW**, outward facet normal); **[`Shape::transform`](Shape::transform)** poses like procedural **[`cube`](crate::shapes::cube)** or **[`dodecahedron`](crate::shapes::dodecahedron)**.

use glam::{Mat4, Vec3};

use super::facet::{Facet, NormalTransform};
use super::unit_vec3::UnitVec3;

use crate::{TriMesh, Triangle};

/// Generic mesh: **`Facet::verts`** index into vertex positions from **[`vertices`](Shape::vertices)**.
///
/// Canonical **`TriMesh`** examples: **[`cube`](crate::shapes::cube)**, **[`dodecahedron`](crate::shapes::dodecahedron)**.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    vertices: Vec<Vec3>,
    facets: Vec<Facet>,
}

impl Shape {
    pub fn new(vertices: Vec<Vec3>, facets: Vec<Facet>) -> Self {
        Self { vertices, facets }
    }

    #[inline]
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    #[inline]
    pub fn facets(&self) -> &[Facet] {
        &self.facets
    }

    /// Applies **`Mat4::transform_point3`** per vertex and re-poses stored normals per facet
    /// (inverse-transpose of **`m`**'s upper-left **3×3**, computed once for all facets).
    pub fn transform(&self, m: Mat4) -> Shape {
        let normal_transform = NormalTransform::from_model(m);
        Shape {
            vertices: self
                .vertices
                .iter()
                .copied()
                .map(|v| m.transform_point3(v))
                .collect(),
            facets: self
                .facets
                .iter()
                .map(|f| f.transform(normal_transform))
                .collect(),
        }
    }
}

impl TriMesh for Shape {
    fn visible_facets(&self, view_direction: UnitVec3) -> impl Iterator<Item = Triangle> + '_ {
        self.facets
            .iter()
            .filter(move |f| f.is_front_facing(view_direction))
            .map(|facet| {
                let corners = facet.resolve_vertices(&self.vertices);
                Triangle {
                    corners,
                    normals: *facet.vertex_normals(),
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use approx::assert_relative_eq;

    use super::*;
    use glam::Mat4;

    /// **`z = 0`**, **`[-½, ½]²`** in **XY**. Two triangles, outward **[`UnitVec3::NEG_Z`]** —
    /// visible when **into‑scene** view is **`+Z`** (same rule as **`cube`** fronts vs
    /// [`Camera::direction`](crate::Camera::direction)).
    fn flat_square_xy() -> Shape {
        #[rustfmt::skip]
        let vertices = vec![
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new( 0.5, -0.5, 0.0),
            Vec3::new( 0.5,  0.5, 0.0),
            Vec3::new(-0.5,  0.5, 0.0),
        ];
        // CCW winding viewed from **`−Z`** (outside along **`UnitVec3::NEG_Z`**).
        let facets = vec![
            Facet::with_facet_normal([0, 2, 1], UnitVec3::NEG_Z),
            Facet::with_facet_normal([0, 3, 2], UnitVec3::NEG_Z),
        ];
        Shape::new(vertices, facets)
    }

    #[test]
    fn from_pos_z_both_facets_visible_with_neg_z_normals() {
        let shape = flat_square_xy();
        let visible = shape.visible_facets(UnitVec3::Z).collect::<Vec<_>>();
        assert_eq!(visible.len(), 2);
        assert!(
            visible
                .iter()
                .all(|tri| tri.normals == [UnitVec3::NEG_Z, UnitVec3::NEG_Z, UnitVec3::NEG_Z])
        );
    }

    #[test]
    fn from_neg_z_no_facets_visible() {
        assert_eq!(flat_square_xy().visible_facets(UnitVec3::NEG_Z).count(), 0);
    }

    #[test]
    fn perpendicular_view_is_grazing_neither_triangle_front() {
        let shape = flat_square_xy();
        assert_eq!(shape.visible_facets(UnitVec3::X).count(), 0);
        assert_eq!(shape.visible_facets(UnitVec3::Y).count(), 0);
    }

    #[test]
    fn pi_rotation_y_swaps_which_direction_sees_square() {
        let shape = flat_square_xy();
        let flipped = shape.transform(Mat4::from_rotation_y(PI));

        assert_eq!(flipped.visible_facets(UnitVec3::Z).count(), 0);
        assert_eq!(flipped.visible_facets(UnitVec3::NEG_Z).count(), 2);

        assert_eq!(
            shape.visible_facets(UnitVec3::Z).count(),
            flipped.visible_facets(UnitVec3::NEG_Z).count(),
        );
    }

    #[test]
    fn transform_applies_to_facet_normal() {
        let vertices = [
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let facet_normal = UnitVec3::from_points_ccw(&vertices);
        let shape = Shape::new(
            vertices.to_vec(),
            vec![Facet::with_facet_normal([0, 1, 2], facet_normal)],
        );

        let transformed_shape = shape.transform(Mat4::from_scale(Vec3::new(1.0, 0.5, 1.0)));
        let transformed_facet = transformed_shape.facets()[0];
        let transformed_vertices =
            transformed_facet.resolve_vertices(transformed_shape.vertices());

        let expected_transformed_normal = UnitVec3::from_points_ccw(&transformed_vertices);
        let transformed_facet_normal = transformed_shape.facets()[0].facet_normal();

        assert_relative_eq!(expected_transformed_normal, transformed_facet_normal);
        for (actual, expected) in transformed_facet
            .vertex_normals()
            .iter()
            .zip([expected_transformed_normal; 3])
        {
            assert_relative_eq!(*actual, expected);
        }
    }
}
