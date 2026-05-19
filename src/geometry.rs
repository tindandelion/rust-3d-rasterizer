use std::ops::Deref;

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal3(Vec3);

impl Normal3 {
    pub const Z: Self = Self(Vec3::Z);
}

impl From<Vec3> for Normal3 {
    fn from(value: Vec3) -> Self {
        assert!(
            value.length_squared() > 0.0,
            "Normal3 requires a non-zero Vec3"
        );
        Self(value.normalize())
    }
}

impl From<Normal3> for Vec3 {
    fn from(value: Normal3) -> Self {
        value.0
    }
}

impl Deref for Normal3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
