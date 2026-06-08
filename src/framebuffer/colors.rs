//! sRGB-style **RGB888** triple (**`u8`** per channel).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const BLACK: Self = Self(0, 0, 0);
    pub const WHITE: Self = Self(255, 255, 255);
    pub const BLUE: Self = Self(0, 0, 255);

    /// **Brightness** in **`[0.0, 1.0]`**: arithmetic mean of **`R`**, **`G`**, and **`B`**
    /// (each channel normalized from **`u8`**).
    pub fn brightness(self) -> f32 {
        (self.0 as f32 + self.1 as f32 + self.2 as f32) / (3.0 * 255.0)
    }

    /// Per-channel **multiply** by **`factor`**, rounded to nearest **`u8`**, then
    /// **clamped** to **`[0, 255]`** per channel.
    pub fn scale(self, factor: f32) -> Self {
        Self(
            scale_channel(self.0, factor),
            scale_channel(self.1, factor),
            scale_channel(self.2, factor),
        )
    }
}

fn scale_channel(value: u8, factor: f32) -> u8 {
    (value as f32 * factor).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::Rgb;
    use approx::assert_relative_eq;

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
    fn scale_one_is_identity() {
        assert_eq!(Rgb(10, 20, 200).scale(1.0), Rgb(10, 20, 200));
    }

    #[test]
    fn scale_zero_is_black() {
        assert_eq!(Rgb(10, 20, 200).scale(0.0), Rgb::BLACK);
    }

    #[test]
    fn scale_half_rounds_each_channel() {
        assert_eq!(Rgb(100, 101, 255).scale(0.5), Rgb(50, 51, 128));
    }

    #[test]
    fn scale_negative_factor_clamps_channels_to_zero() {
        assert_eq!(Rgb(200, 0, 0).scale(-1.0), Rgb::BLACK);
    }

    #[test]
    fn scale_above_one_saturates_channels() {
        assert_eq!(Rgb(128, 128, 255).scale(2.0), Rgb::WHITE);
        assert_eq!(Rgb(200, 0, 0).scale(2.0), Rgb(255, 0, 0));
    }
}
