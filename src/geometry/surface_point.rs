//! Interpolatable surface attributes (e.g. per-vertex normal) for scanline rasterization.

use std::ops::{Add, Mul, Sub};

use glam::Vec3;

use super::UnitVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePoint {
    position: Vec3,
    normal: Vec3,
}

impl SurfacePoint {
    pub fn new(position: Vec3, normal: UnitVec3) -> Self {
        Self {
            position,
            normal: Vec3::from(normal),
        }
    }

    pub fn normal(&self) -> UnitVec3 {
        self.normal.into()
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}

impl Sub for SurfacePoint {
    type Output = SurfacePoint;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            position: self.position - other.position,
            normal: self.normal - other.normal,
        }
    }
}

impl Mul<f32> for SurfacePoint {
    type Output = SurfacePoint;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            position: self.position * other,
            normal: self.normal * other,
        }
    }
}

impl Add for SurfacePoint {
    type Output = SurfacePoint;

    fn add(self, other: Self) -> Self::Output {
        Self {
            position: self.position + other.position,
            normal: self.normal + other.normal,
        }
    }
}
