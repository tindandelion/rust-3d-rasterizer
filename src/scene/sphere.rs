use glam::Vec3;

use crate::{geometry::Normal3, scene::facet::Facet};

use super::shape::Shape;

const VERTICES: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, -1.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, -1.0, 0.0],
];

const FACETS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [2, 1, 3],
    [1, 4, 3],
    [4, 1, 0],
    [0, 2, 5],
    [5, 2, 3],
    [5, 3, 4],
    [4, 0, 5],
];

pub fn unit_sphere() -> Shape {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for v in VERTICES {
        vertices.push(Vec3::new(v[0], v[1], v[2]));
    }

    for f in FACETS {
        let corners = [vertices[f[0]], vertices[f[1]], vertices[f[2]]];
        let normal = Normal3::from_vertices_ccw(&corners);
        let facet = Facet::new(normal, f);
        faces.push(facet);
    }

    Shape::new(vertices, faces)
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::geometry::Normal3;

    use super::unit_sphere;

    #[test]
    fn initial_unit_sphere_vertices_and_facets() {
        let sphere = unit_sphere();

        assert_eq!(6, sphere.vertices().len());
        assert_eq!(8, sphere.faces().len());
    }

    #[test]
    fn initial_unit_sphere_normals() {
        let expected_normals: Vec<Normal3> = vec![
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
        ]
        .iter()
        .map(|&v| v.into())
        .collect();

        let sphere = unit_sphere();
        let facets = sphere.faces();

        for (i, facet) in facets.iter().enumerate() {
            assert_eq!(expected_normals[i], facet.normal());
        }
    }
}
