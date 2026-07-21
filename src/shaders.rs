use glam::Vec3;

use crate::{
    Light, Material, Shader,
    geometry::{SurfacePoint, UnitVec3},
    lighting::Color,
};

pub struct PhongShader<'a> {
    pub material: &'a Material,
    pub lights: &'a [Light],
    pub toward_eye: UnitVec3,
}

impl<'a> Shader for PhongShader<'a> {
    type VertexData = SurfacePoint;

    fn shade_vertex(&self, position: Vec3, normal: UnitVec3) -> Self::VertexData {
        SurfacePoint::new(position, normal)
    }

    fn shade_pixel(&self, surface_point: SurfacePoint) -> Color {
        self.material
            .shade(self.lights, surface_point, self.toward_eye)
    }
}

pub struct GouraudShader<'a> {
    pub material: &'a Material,
    pub lights: &'a [Light],
    pub toward_eye: UnitVec3,
}

impl<'a> Shader for GouraudShader<'a> {
    type VertexData = Color;

    fn shade_vertex(&self, position: Vec3, normal: UnitVec3) -> Self::VertexData {
        let surface_point = SurfacePoint::new(position, normal);
        self.material
            .shade(self.lights, surface_point, self.toward_eye)
    }

    fn shade_pixel(&self, color: Color) -> Color {
        color
    }
}
