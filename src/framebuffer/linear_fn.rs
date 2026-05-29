pub struct LinearFn(f32, f32);

impl LinearFn {
    pub fn from_endpoints(a: (f32, f32), b: (f32, f32)) -> Self {
        if a.0 == b.0 {
            return Self(f32::NAN, a.1);
        }
        let slope = (b.1 - a.1) / (b.0 - a.0);
        let intercept = a.1 - slope * a.0;
        Self(slope, intercept)
    }

    pub fn get(&self, x: f32) -> f32 {
        if self.0.is_nan() {
            self.1
        } else {
            x * self.0 + self.1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinearFn;
    use approx::assert_relative_eq;

    #[test]
    fn start_and_end_points() {
        let linear_fn = LinearFn::from_endpoints((0.0, 1.0), (10.0, 11.0));

        assert_relative_eq!(linear_fn.get(0.0), 1.0);
        assert_relative_eq!(linear_fn.get(10.0), 11.0);
    }

    #[test]
    fn value_at_intercept() {
        let linear_fn = LinearFn::from_endpoints((2.0, 6.0), (4.0, 9.0));
        assert_relative_eq!(linear_fn.get(0.0), 3.0);
    }

    #[test]
    fn interior_point() {
        let linear_fn = LinearFn::from_endpoints((0.0, 0.0), (4.0, 8.0));
        assert_relative_eq!(linear_fn.get(2.0), 4.0);
    }

    #[test]
    fn decreasing_line() {
        let linear_fn = LinearFn::from_endpoints((0.0, 100.0), (4.0, 0.0));

        assert_relative_eq!(linear_fn.get(0.0), 100.0);
        assert_relative_eq!(linear_fn.get(4.0), 0.0);
        assert_relative_eq!(linear_fn.get(1.0), 75.0);
    }

    #[test]
    fn horizontal_line() {
        let linear_fn = LinearFn::from_endpoints((0.0, 5.0), (10.0, 5.0));

        assert_relative_eq!(linear_fn.get(0.0), 5.0);
        assert_relative_eq!(linear_fn.get(10.0), 5.0);
        assert_relative_eq!(linear_fn.get(3.0), 5.0);
    }

    #[test]
    fn slope_is_infinity() {
        let linear_fn = LinearFn::from_endpoints((3.0, 7.0), (3.0, 12.0));

        assert_relative_eq!(linear_fn.get(3.0), 7.0);
        assert_relative_eq!(linear_fn.get(100.0), 7.0);
    }
}
