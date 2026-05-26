use std::collections::HashMap;

use glam::Vec3;

use crate::{geometry::Normal3, scene::facet::Facet};

use super::shape::Shape;

struct OctoSplitter {
    vertices: Vec<Vec3>,
    facets: Vec<Facet>,
    midpoint_cache: HashMap<(usize, usize), usize>,
}

impl OctoSplitter {
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

    pub fn new() -> Self {
        let vertices: Vec<Vec3> = Self::VERTICES
            .iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]))
            .collect();

        let facets: Vec<Facet> = Self::FACETS
            .iter()
            .map(|f| {
                let corners = [vertices[f[0]], vertices[f[1]], vertices[f[2]]];
                let normal = Normal3::from_vertices_ccw(&corners);
                Facet::new(normal, *f)
            })
            .collect();

        Self {
            vertices,
            facets,
            midpoint_cache: HashMap::new(),
        }
    }

    pub fn split_facets(&mut self) {
        let facets = std::mem::take(&mut self.facets);
        self.midpoint_cache.clear();

        for facet in facets.iter() {
            let splitted = self.split_facet(facet);
            self.facets.extend(splitted);
        }
    }

    pub fn split_facet(&mut self, facet: &Facet) -> [Facet; 4] {
        let [i_a, i_b, i_c] = facet.verts();
        let [a, b, c] = facet.retrieve_vertices(&self.vertices);
        let (i_m_ab, m_ab) = self.split_edge(*i_a, *i_b);
        let (i_m_ac, m_ac) = self.split_edge(*i_a, *i_c);
        let (i_m_bc, m_bc) = self.split_edge(*i_b, *i_c);

        let facet_1 = Facet::new(
            Normal3::from_vertices_ccw(&[a, m_ab, m_ac]),
            [*i_a, i_m_ab, i_m_ac],
        );

        let facet_2 = Facet::new(
            Normal3::from_vertices_ccw(&[b, m_bc, m_ab]),
            [*i_b, i_m_bc, i_m_ab],
        );

        let facet_3 = Facet::new(
            Normal3::from_vertices_ccw(&[c, m_ac, m_bc]),
            [*i_c, i_m_ac, i_m_bc],
        );

        let facet_4 = Facet::new(
            Normal3::from_vertices_ccw(&[m_ab, m_bc, m_ac]),
            [i_m_ab, i_m_bc, i_m_ac],
        );
        [facet_1, facet_2, facet_3, facet_4]
    }

    pub fn build(self) -> Shape {
        Shape::new(self.vertices, self.facets)
    }

    fn split_edge(&mut self, idx_a: usize, idx_b: usize) -> (usize, Vec3) {
        let cache_key = if idx_a < idx_b {
            (idx_a, idx_b)
        } else {
            (idx_b, idx_a)
        };
        if let Some(&idx_m) = self.midpoint_cache.get(&cache_key) {
            return (idx_m, self.vertices[idx_m]);
        }

        let a = self.vertices[idx_a];
        let b = self.vertices[idx_b];
        let new_vertex = (a + b).normalize();
        let vertex_idx = self.vertices.len();

        self.vertices.push(new_vertex);
        self.midpoint_cache.insert(cache_key, vertex_idx);
        (vertex_idx, new_vertex)
    }
}

pub fn unit_sphere(splits: usize) -> Shape {
    let mut octo_splitter = OctoSplitter::new();
    for _ in 0..splits {
        octo_splitter.split_facets();
    }
    octo_splitter.build()
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::geometry::Normal3;

    use super::unit_sphere;

    #[test]
    fn initial_unit_sphere_vertices_and_facets() {
        let sphere = unit_sphere(0);

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

        let sphere = unit_sphere(0);
        let facets = sphere.faces();

        for (i, facet) in facets.iter().enumerate() {
            assert_eq!(expected_normals[i], facet.normal());
        }
    }

    #[test]
    fn unit_sphere_with_one_split() {
        let sphere = unit_sphere(1);
        assert_eq!(32, sphere.faces().len());
        assert_eq!(18, sphere.vertices().len());
    }

    #[test]
    fn unit_sphere_with_two_splits() {
        let sphere = unit_sphere(2);
        assert_eq!(128, sphere.faces().len());
        assert_eq!(66, sphere.vertices().len());
    }
}
