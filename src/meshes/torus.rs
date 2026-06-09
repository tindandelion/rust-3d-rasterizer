//! Parametric **torus** in **XZ** (major circle), tube around **Y** (vertical hole).
//!
//! Default radii (**`DEFAULT_MAJOR_RADIUS`**, **`DEFAULT_MINOR_RADIUS`**) target a readable
//! silhouette before world scale in export bins.

use std::f32::consts::TAU;

use glam::Vec3;

use crate::geometry::{Facet, Mesh, UnitVec3};

/// Ring center radius (origin → tube center in **XZ**).
const DEFAULT_MAJOR_RADIUS: f32 = 0.7;
/// Tube cross-section radius.
const DEFAULT_MINOR_RADIUS: f32 = 0.3;

/// Indexed torus with **smooth vertex normals** for **Phong** shading.
///
/// **`ring_segments`** and **`tube_segments`** must each be **≥ 3**.
pub fn torus(ring_segments: usize, tube_segments: usize) -> Mesh {
    assert!(ring_segments >= 3, "ring_segments must be at least 3");
    assert!(tube_segments >= 3, "tube_segments must be at least 3");

    let major = DEFAULT_MAJOR_RADIUS;
    let minor = DEFAULT_MINOR_RADIUS;

    let vertex_count = ring_segments * tube_segments;
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut vertex_normals = Vec::with_capacity(vertex_count);

    for i in 0..ring_segments {
        let u = TAU * i as f32 / ring_segments as f32;
        for j in 0..tube_segments {
            let v = TAU * j as f32 / tube_segments as f32;
            let (position, normal) = torus_frame(major, minor, u, v);
            vertices.push(position);
            vertex_normals.push(normal);
        }
    }

    let idx =
        |i: usize, j: usize| -> usize { (i % ring_segments) * tube_segments + (j % tube_segments) };

    let mut facets = Vec::with_capacity(ring_segments * tube_segments * 2);
    for i in 0..ring_segments {
        for j in 0..tube_segments {
            let i0 = idx(i, j);
            let i1 = idx(i + 1, j);
            let i2 = idx(i + 1, j + 1);
            let i3 = idx(i, j + 1);

            facets.push(Facet::with_vertex_normals(
                [i0, i1, i2],
                [vertex_normals[i0], vertex_normals[i1], vertex_normals[i2]],
            ));
            facets.push(Facet::with_vertex_normals(
                [i0, i2, i3],
                [vertex_normals[i0], vertex_normals[i2], vertex_normals[i3]],
            ));
        }
    }

    Mesh::new(vertices, facets)
}

/// Parametric sample: major circle in **XZ**, tube displacement along **Y**.
fn torus_frame(major: f32, minor: f32, u: f32, v: f32) -> (Vec3, UnitVec3) {
    let (cu, su) = (u.cos(), u.sin());
    let (cv, sv) = (v.cos(), v.sin());
    let ring = major + minor * cv;
    let position = Vec3::new(ring * cu, minor * sv, ring * su);
    let normal = Vec3::new(cu * cv, sv, su * cv);
    (position, normal.into())
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::Vec3;

    use crate::geometry::UnitVec3;

    use super::{DEFAULT_MAJOR_RADIUS, DEFAULT_MINOR_RADIUS, torus};

    fn ring_center_at_u(u: f32) -> Vec3 {
        let (cu, su) = (u.cos(), u.sin());
        Vec3::new(DEFAULT_MAJOR_RADIUS * cu, 0.0, DEFAULT_MAJOR_RADIUS * su)
    }

    #[test]
    fn torus_vertex_and_facet_counts() {
        let mesh = torus(12, 8);
        assert_eq!(12 * 8, mesh.vertices().len());
        assert_eq!(12 * 8 * 2, mesh.facets().len());
    }

    #[test]
    fn torus_vertices_lie_on_tube_radius() {
        let mesh = torus(16, 12);
        let ring_segments = 16;
        let tube_segments = 12;

        for i in 0..ring_segments {
            let u = std::f32::consts::TAU * i as f32 / ring_segments as f32;
            let center = ring_center_at_u(u);
            for j in 0..tube_segments {
                let idx = i * tube_segments + j;
                let p = mesh.vertices()[idx];
                assert_relative_eq!(DEFAULT_MINOR_RADIUS, (p - center).length());
            }
        }
    }

    #[test]
    fn torus_vertex_normals_are_unit_length() {
        let mesh = torus(10, 6);
        for facet in mesh.facets() {
            for &normal in facet.vertex_normals() {
                assert_relative_eq!(1.0, Vec3::from(normal).length());
            }
        }
    }

    #[test]
    fn torus_vertex_normals_point_outward_from_ring() {
        let ring_segments = 12;
        let tube_segments = 8;
        let mesh = torus(ring_segments, tube_segments);

        for facet in mesh.facets() {
            let corners = facet.resolve_vertices(mesh.vertices());
            for (corner, &vert_idx) in corners.iter().zip(facet.vert_indices()) {
                let i = vert_idx / tube_segments;
                let u = std::f32::consts::TAU * i as f32 / ring_segments as f32;
                let center = ring_center_at_u(u);
                let corner_idx = facet
                    .vert_indices()
                    .iter()
                    .position(|&v| v == vert_idx)
                    .unwrap();
                let normal: Vec3 = facet.vertex_normals()[corner_idx].into();
                assert!(normal.dot(*corner - center) > 0.0);
            }
        }
    }

    #[test]
    fn torus_facet_winding_aligns_with_vertex_normals() {
        let mesh = torus(12, 8);
        for facet in mesh.facets() {
            let corners = facet.resolve_vertices(mesh.vertices());
            let from_winding = UnitVec3::from_points_ccw(&corners);
            assert!(from_winding.dot(facet.facet_normal()) > 0.95);
        }
    }
}
