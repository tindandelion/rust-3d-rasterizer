//! sRGB-style **RGB888** triple (**`u8`** per channel).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const BLACK: Self = Self(0, 0, 0);
    pub const WHITE: Self = Self(255, 255, 255);

    /// Per-channel **multiply** by **`factor`**, rounded to nearest **`u8`**.
    ///
    /// **`factor`** is **clamped** to **`[0.0, 1.0]`** before scaling.
    pub fn scale(self, factor: f32) -> Self {
        let f = factor.clamp(0.0, 1.0);
        Self(
            scale_channel(self.0, f),
            scale_channel(self.1, f),
            scale_channel(self.2, f),
        )
    }
}

fn scale_channel(value: u8, factor: f32) -> u8 {
    (value as f32 * factor).round() as u8
}

#[cfg(test)]
mod tests {
    use super::Rgb;

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
    fn scale_clamps_factor_below_range() {
        assert_eq!(Rgb(200, 0, 0).scale(-1.0), Rgb::BLACK);
    }

    #[test]
    fn scale_clamps_factor_above_range() {
        assert_eq!(Rgb(200, 0, 0).scale(2.0), Rgb(200, 0, 0));
    }
}
