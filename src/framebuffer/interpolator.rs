use std::ops::{Add, Div, Mul, Sub};

pub struct Interpolator<T> {
    slope: Option<T>,
    intercept: T,
}

pub type ScalarInterpolator = Interpolator<f32>;

impl<T> Interpolator<T>
where
    T: Copy + Add<T, Output = T> + Sub<T, Output = T> + Mul<f32, Output = T> + Div<f32, Output = T>,
{
    pub fn from_endpoints(a: (f32, T), b: (f32, T)) -> Self {
        if a.0 == b.0 {
            return Self {
                slope: None,
                intercept: a.1,
            };
        }
        let slope = (b.1 - a.1) / (b.0 - a.0);
        let intercept = a.1 - slope * a.0;
        Self {
            slope: Some(slope),
            intercept,
        }
    }

    pub fn get(&self, x: f32) -> T {
        match self.slope {
            Some(slope) => slope * x + self.intercept,
            None => self.intercept,
        }
    }
}

#[cfg(test)]
mod interpolator_tests {
    use super::Interpolator;
    use approx::assert_relative_eq;

    #[test]
    fn start_and_end_points() {
        let interpolator = Interpolator::from_endpoints((0.0, 1.0), (10.0, 11.0));

        assert_relative_eq!(interpolator.get(0.0), 1.0);
        assert_relative_eq!(interpolator.get(10.0), 11.0);
    }

    #[test]
    fn value_at_intercept() {
        let interpolator = Interpolator::from_endpoints((2.0, 6.0), (4.0, 9.0));
        assert_relative_eq!(interpolator.get(0.0), 3.0);
    }

    #[test]
    fn interior_point() {
        let interpolator = Interpolator::from_endpoints((0.0, 0.0), (4.0, 8.0));
        assert_relative_eq!(interpolator.get(2.0), 4.0);
    }

    #[test]
    fn decreasing_line() {
        let interpolator = Interpolator::from_endpoints((0.0, 100.0), (4.0, 0.0));

        assert_relative_eq!(interpolator.get(0.0), 100.0);
        assert_relative_eq!(interpolator.get(4.0), 0.0);
        assert_relative_eq!(interpolator.get(1.0), 75.0);
    }

    #[test]
    fn horizontal_line() {
        let interpolator = Interpolator::from_endpoints((0.0, 5.0), (10.0, 5.0));

        assert_relative_eq!(interpolator.get(0.0), 5.0);
        assert_relative_eq!(interpolator.get(10.0), 5.0);
        assert_relative_eq!(interpolator.get(3.0), 5.0);
    }

    #[test]
    fn slope_is_infinity() {
        let interpolator = Interpolator::from_endpoints((3.0, 7.0), (3.0, 12.0));

        assert_relative_eq!(interpolator.get(3.0), 7.0);
        assert_relative_eq!(interpolator.get(100.0), 7.0);
    }
}
