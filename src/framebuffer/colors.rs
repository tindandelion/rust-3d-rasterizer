//! sRGB-style **RGB888** triple (**`u8`** per channel).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const BLACK: Self = Self(0, 0, 0);
    pub const WHITE: Self = Self(255, 255, 255);
    pub const BLUE: Self = Self(0, 0, 255);
    pub const RED: Self = Self(255, 0, 0);

    /// **sRGB** **`0xRRGGBB`** (e.g. geometry-browser **`0x156289`**) → **`Rgb`** channels.
    pub const fn from_hex(hex: u32) -> Self {
        Self(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    /// **Brightness** in **`[0.0, 1.0]`**: arithmetic mean of **`R`**, **`G`**, and **`B`**
    /// (each channel normalized from **`u8`**).
    pub fn brightness(self) -> f32 {
        (self.0 as f32 + self.1 as f32 + self.2 as f32) / (3.0 * 255.0)
    }

    /// Decodes **IEC 61966-2-1** sRGB channels to linear light-energy **`[0, 1]`** per channel.
    pub fn to_linear(self) -> (f32, f32, f32) {
        (
            srgb_channel_to_linear(self.0),
            srgb_channel_to_linear(self.1),
            srgb_channel_to_linear(self.2),
        )
    }

    /// Encodes linear light-energy channels to sRGB **`u8`** values, clamping each channel to **`[0, 255]`**.
    pub fn from_linear((r, g, b): (f32, f32, f32)) -> Self {
        Self(
            linear_channel_to_srgb(r),
            linear_channel_to_srgb(g),
            linear_channel_to_srgb(b),
        )
    }
}

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

#[cfg(test)]
mod tests {
    use super::Rgb;
    use approx::assert_relative_eq;

    #[test]
    fn from_hex_decodes_rgb_channels() {
        assert_eq!(Rgb::from_hex(0x156289), Rgb(21, 98, 137));
        assert_eq!(Rgb::from_hex(0x072534), Rgb(7, 37, 52));
        assert_eq!(Rgb::from_hex(0x111111), Rgb(17, 17, 17));
        assert_eq!(Rgb::from_hex(0x444444), Rgb(68, 68, 68));
    }

    #[test]
    fn from_hex_extremes_match_named_colors() {
        assert_eq!(Rgb::from_hex(0x000000), Rgb::BLACK);
        assert_eq!(Rgb::from_hex(0xFFFFFF), Rgb::WHITE);
        assert_eq!(Rgb::from_hex(0xFF0000), Rgb::RED);
        assert_eq!(Rgb::from_hex(0x0000FF), Rgb::BLUE);
    }

    #[test]
    fn brightness_black_is_zero() {
        assert_relative_eq!(Rgb::BLACK.brightness(), 0.0);
    }

    #[test]
    fn brightness_white_is_one() {
        assert_relative_eq!(Rgb::WHITE.brightness(), 1.0);
    }

    #[test]
    fn brightness_is_channel_average() {
        assert_relative_eq!(Rgb(255, 0, 0).brightness(), 1.0 / 3.0);
        assert_relative_eq!(Rgb(0, 255, 0).brightness(), 1.0 / 3.0);
        assert_relative_eq!(Rgb(0, 0, 255).brightness(), 1.0 / 3.0);
        assert_relative_eq!(Rgb(60, 90, 120).brightness(), 90.0 / 255.0);
    }

    #[test]
    fn black_and_white_linear_round_trip_is_identity() {
        assert_eq!(Rgb::from_linear(Rgb::BLACK.to_linear()), Rgb::BLACK);
        assert_eq!(Rgb::from_linear(Rgb::WHITE.to_linear()), Rgb::WHITE);
    }

    #[test]
    fn decodes_srgb_green_channel() {
        let (_, g, _) = Rgb(0, 98, 0).to_linear();
        assert_relative_eq!(g, 0.122_138_8, max_relative = 1e-5);
    }

    #[test]
    fn tripled_linear_green_stays_below_byte_space_clip() {
        let (_, g, _) = Rgb(0, 98, 0).to_linear();
        let linear_path = Rgb::from_linear((0.0, g * 3.0, 0.0));
        let byte_space_clip = Rgb(0, 255, 0);

        assert_eq!(linear_path, Rgb(0, 163, 0));
        assert_eq!(byte_space_clip, Rgb(0, 255, 0));
        assert!(linear_path.1 < byte_space_clip.1);
    }

    #[test]
    fn round_trips_geometry_browser_palette() {
        let palette = Rgb::from_hex(0x156289);
        assert_eq!(Rgb::from_linear(palette.to_linear()), palette);
    }

    #[test]
    fn round_trips_every_byte_value_per_channel() {
        for channel in 0..=255_u8 {
            let rgb = Rgb(channel, channel, channel);
            assert_eq!(
                Rgb::from_linear(rgb.to_linear()),
                rgb,
                "round-trip failed for channel {channel}"
            );
        }
    }

    #[test]
    fn linear_above_one_clamps_to_white() {
        assert_eq!(Rgb::from_linear((2.0, 2.0, 2.0)), Rgb::WHITE);
    }

    #[test]
    fn negative_linear_clamps_to_black() {
        assert_eq!(Rgb::from_linear((-1.0, -0.5, 0.0)), Rgb::BLACK);
    }

    #[test]
    fn low_srgb_segment_uses_linear_slope() {
        let (r, _, _) = Rgb(1, 0, 0).to_linear();
        assert_relative_eq!(r, (1.0 / 255.0) / 12.92);
    }
}
