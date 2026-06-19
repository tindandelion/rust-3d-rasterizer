//! Linear RGB for Phong shading (light-energy space; encode to sRGB via [`Rgb::from_linear`](crate::framebuffer::Rgb::from_linear)).

use std::ops::{Add, Mul};

/// Linear RGB in **`[0, 1]`** per channel (light-energy space); may exceed **`1.0`** during shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Color(f32, f32, f32);

impl Color {
    pub(super) fn from_linear(linear: (f32, f32, f32)) -> Self {
        Self(linear.0, linear.1, linear.2)
    }

    pub(super) fn to_linear(self) -> (f32, f32, f32) {
        (self.0, self.1, self.2)
    }
}

impl Add for Color {
    type Output = Self;

    /// Per-channel **add** in linear light space (no clamp until sRGB encode).
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    /// Per-channel **multiply** by **`factor`** in linear light space.
    fn mul(self, factor: f32) -> Self::Output {
        Self(self.0 * factor, self.1 * factor, self.2 * factor)
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, color: Color) -> Self::Output {
        color * self
    }
}

#[cfg(test)]
mod tests {
    use super::Color;
    use crate::framebuffer::Rgb;
    use approx::assert_relative_eq;

    #[test]
    fn add_sums_linear_channels() {
        let a = Color::from_linear(Rgb(10, 20, 30).to_linear());
        let b = Color::from_linear(Rgb(1, 2, 3).to_linear());
        let sum = a + b;
        assert_relative_eq!(sum.0, a.0 + b.0);
        assert_relative_eq!(sum.1, a.1 + b.1);
        assert_relative_eq!(sum.2, a.2 + b.2);
    }

    #[test]
    fn mul_scales_each_channel() {
        let color = Color::from_linear(Rgb(0, 98, 0).to_linear());
        assert_eq!(Rgb::from_linear((color * 3.0).to_linear()), Rgb(0, 163, 0));
        assert_eq!(
            Rgb::from_linear((0.5 * color).to_linear()),
            Rgb::from_linear(Color::from_linear((0.0, color.1 * 0.5, 0.0)).to_linear())
        );
    }

    #[test]
    fn scalar_mul_matches_color_mul() {
        let color = Color::from_linear(Rgb(40, 80, 120).to_linear());
        assert_eq!(0.25 * color, color * 0.25);
    }
}
