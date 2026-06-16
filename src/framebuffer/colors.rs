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
}
