//! Linear RGB for Phong shading (**IEC 61966-2-1** sRGB ↔ linear conversion).

use crate::framebuffer::Rgb;
use std::ops::{Add, Mul};

/// Linear RGB in **`[0, 1]`** per channel (light-energy space); may exceed **`1.0`** during shading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Color(f32, f32, f32);

const SRGB_LINEAR_BREAK: f32 = 0.04045;
const SRGB_LINEAR_SCALE: f32 = 12.92;
const SRGB_ENCODE_BREAK: f32 = 0.0031308;
const SRGB_GAMMA: f32 = 2.4;

fn srgb_channel_to_linear(channel: u8) -> f32 {
    let s = channel as f32 / 255.0;
    if s <= SRGB_LINEAR_BREAK {
        s / SRGB_LINEAR_SCALE
    } else {
        ((s + 0.055) / 1.055).powf(SRGB_GAMMA)
    }
}

fn linear_channel_to_srgb(channel: f32) -> u8 {
    let s = if channel <= SRGB_ENCODE_BREAK {
        channel * SRGB_LINEAR_SCALE
    } else {
        1.055 * channel.powf(1.0 / SRGB_GAMMA) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Self(
            srgb_channel_to_linear(rgb.0),
            srgb_channel_to_linear(rgb.1),
            srgb_channel_to_linear(rgb.2),
        )
    }
}

impl From<Color> for Rgb {
    fn from(color: Color) -> Self {
        Self(
            linear_channel_to_srgb(color.0),
            linear_channel_to_srgb(color.1),
            linear_channel_to_srgb(color.2),
        )
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
    fn black_and_white_are_identity() {
        assert_eq!(Rgb::from(Color::from(Rgb::BLACK)), Rgb::BLACK);
        assert_eq!(Rgb::from(Color::from(Rgb::WHITE)), Rgb::WHITE);
    }

    #[test]
    fn decodes_srgb_green_channel() {
        let linear: Color = Rgb(0, 98, 0).into();
        assert_relative_eq!(linear.1, 0.122_138_8, max_relative = 1e-5);
    }

    #[test]
    fn tripled_linear_green_stays_below_byte_space_clip() {
        let base: Color = Rgb(0, 98, 0).into();
        let linear_path = Rgb::from(Color(0.0, base.1 * 3.0, 0.0));
        let byte_space_clip = Rgb(0, 255, 0);

        assert_eq!(linear_path, Rgb(0, 163, 0));
        assert_eq!(byte_space_clip, Rgb(0, 255, 0));
        assert!(linear_path.1 < byte_space_clip.1);
    }

    #[test]
    fn round_trips_geometry_browser_palette() {
        let palette = Rgb::from_hex(0x156289);
        assert_eq!(Rgb::from(Color::from(palette)), palette);
    }

    #[test]
    fn round_trips_every_byte_value_per_channel() {
        for channel in 0..=255_u8 {
            let rgb = Rgb(channel, channel, channel);
            assert_eq!(
                Rgb::from(Color::from(rgb)),
                rgb,
                "round-trip failed for channel {channel}"
            );
        }
    }

    #[test]
    fn linear_above_one_clamps_to_white() {
        assert_eq!(Rgb::from(Color(2.0, 2.0, 2.0)), Rgb::WHITE);
    }

    #[test]
    fn negative_linear_clamps_to_black() {
        assert_eq!(Rgb::from(Color(-1.0, -0.5, 0.0)), Rgb::BLACK);
    }

    #[test]
    fn low_srgb_segment_uses_linear_slope() {
        let linear: Color = Rgb(1, 0, 0).into();
        assert_relative_eq!(linear.0, (1.0 / 255.0) / 12.92);
    }

    #[test]
    fn add_sums_linear_channels() {
        let a: Color = Rgb(10, 20, 30).into();
        let b: Color = Rgb(1, 2, 3).into();
        let sum = a + b;
        assert_relative_eq!(sum.0, a.0 + b.0);
        assert_relative_eq!(sum.1, a.1 + b.1);
        assert_relative_eq!(sum.2, a.2 + b.2);
    }

    #[test]
    fn mul_scales_each_channel() {
        let color: Color = Rgb(0, 98, 0).into();
        assert_eq!(Rgb::from(color * 3.0), Rgb(0, 163, 0));
        assert_eq!(
            Rgb::from(0.5 * color),
            Rgb::from(Color(0.0, color.1 * 0.5, 0.0))
        );
    }

    #[test]
    fn scalar_mul_matches_color_mul() {
        let color: Color = Rgb(40, 80, 120).into();
        assert_eq!(0.25 * color, color * 0.25);
    }
}
