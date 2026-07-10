//! Linear RGB for Phong shading (light-energy space; encode to sRGB via [`Rgb::from_linear`](crate::framebuffer::Rgb::from_linear)).

use std::ops::{Add, Mul};

use approx::{AbsDiffEq, RelativeEq};

/// Linear RGB in **`[0, 1]`** per channel (light-energy space); may exceed **`1.0`** during shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub f32, pub f32, pub f32);

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

impl AbsDiffEq for Color {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
            && self.1.abs_diff_eq(&other.1, epsilon)
            && self.2.abs_diff_eq(&other.2, epsilon)
    }
}

impl RelativeEq for Color {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.0.relative_eq(&other.0, epsilon, max_relative)
            && self.1.relative_eq(&other.1, epsilon, max_relative)
            && self.2.relative_eq(&other.2, epsilon, max_relative)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;
    use approx::assert_relative_eq;

    #[test]
    fn add_sums_linear_channels() {
        let a = Color(0.1, 0.2, 0.3);
        let b = Color(0.4, 0.5, 0.6);

        let sum = a + b;
        assert_relative_eq!(Color(0.5, 0.7, 0.9), sum);
    }

    #[test]
    fn mul_scales_each_channel() {
        let color = Color(0.1, 0.2, 0.3);

        let scaled = color * 3.0;

        assert_relative_eq!(Color(0.3, 0.6, 0.9), scaled);
    }

    #[test]
    fn scalar_mul_matches_color_mul() {
        let color = Color(0.1, 0.2, 0.3);
        assert_eq!(0.25 * color, color * 0.25);
    }
}
