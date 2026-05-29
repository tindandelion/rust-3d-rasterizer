use std::collections::HashMap;

use glam::Vec3;

use crate::geometry::{Facet, Shape, UnitVec3};

pub fn sphere(splits: usize) -> Shape {
    let mut splitter = OctaSplitter::new();
    for _ in 0..splits {
        splitter.split_facets();
    }
    splitter.build()
}

struct OctaSplitter {
    vertices: Vec<Vec3>,
    facets: Vec<Facet>,
    midpoint_cache: HashMap<(usize, usize), usize>,
}

impl OctaSplitter {
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
                let normal = UnitVec3::from_points_ccw(&corners);
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
        let &[i_a, i_b, i_c] = facet.vert_indices();
        let [a, b, c] = facet.resolve_vertices(&self.vertices);

        let (i_m_ab, m_ab) = self.split_edge(i_a, i_b);
        let (i_m_ac, m_ac) = self.split_edge(i_a, i_c);
        let (i_m_bc, m_bc) = self.split_edge(i_b, i_c);

        let facet_1 = Facet::new(
            UnitVec3::from_points_ccw(&[a, m_ab, m_ac]),
            [i_a, i_m_ab, i_m_ac],
        );
        let facet_2 = Facet::new(
            UnitVec3::from_points_ccw(&[b, m_bc, m_ab]),
            [i_b, i_m_bc, i_m_ab],
        );
        let facet_3 = Facet::new(
            UnitVec3::from_points_ccw(&[c, m_ac, m_bc]),
            [i_c, i_m_ac, i_m_bc],
        );
        let facet_4 = Facet::new(
            UnitVec3::from_points_ccw(&[m_ab, m_bc, m_ac]),
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

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::Vec3;

    use crate::geometry::UnitVec3;

    use super::sphere;

    #[test]
    fn initial_sphere_vertices_and_facets() {
        let mesh = sphere(0);

        assert_eq!(6, mesh.vertices().len());
        assert_eq!(8, mesh.facets().len());
    }

    #[test]
    fn initial_sphere_normals() {
        let expected_normals: [UnitVec3; 8] = [
            Vec3::new(1.0, 1.0, -1.0).into(),
            Vec3::new(-1.0, 1.0, -1.0).into(),
            Vec3::new(-1.0, 1.0, 1.0).into(),
            Vec3::new(1.0, 1.0, 1.0).into(),
            Vec3::new(1.0, -1.0, -1.0).into(),
            Vec3::new(-1.0, -1.0, -1.0).into(),
            Vec3::new(-1.0, -1.0, 1.0).into(),
            Vec3::new(1.0, -1.0, 1.0).into(),
        ];

        let mesh = sphere(0);
        let facets = mesh.facets();

        for (i, facet) in facets.iter().enumerate() {
            assert_eq!(expected_normals[i], facet.facet_normal());
        }
    }

    #[test]
    fn sphere_with_one_split() {
        let mesh = sphere(1);
        assert_eq!(32, mesh.facets().len());
        assert_eq!(18, mesh.vertices().len());
    }

    #[test]
    fn sphere_with_two_splits() {
        let mesh = sphere(2);
        assert_eq!(128, mesh.facets().len());
        assert_eq!(66, mesh.vertices().len());
    }

    #[test]
    fn sphere_vertices_lie_on_unit_sphere() {
        for splits in 0..=3 {
            let mesh = sphere(splits);
            for p in mesh.vertices() {
                assert_relative_eq!(1.0, p.length());
            }
        }
    }
}
