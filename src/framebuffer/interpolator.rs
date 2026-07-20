use std::ops::{Add, Mul, Sub};

pub type AnchorPoint<T> = (f32, T);

pub trait Interpolatable: Sized + Clone + Copy {
    fn calc_coefficients(a: AnchorPoint<Self>, b: AnchorPoint<Self>) -> (Self, Self);
    fn interpolate(x: f32, slope: Self, intercept: Self) -> Self;
}

impl<T> Interpolatable for T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    fn calc_coefficients(a: AnchorPoint<Self>, b: AnchorPoint<Self>) -> (Self, Self) {
        let slope = (b.1 - a.1) * (1.0 / (b.0 - a.0));
        let intercept = a.1 - slope * a.0;
        (slope, intercept)
    }

    fn interpolate(x: f32, slope: Self, intercept: Self) -> Self {
        slope * x + intercept
    }
}

pub struct Interpolator<T> {
    slope: Option<T>,
    intercept: T,
}

impl<T> Interpolator<T>
where
    T: Interpolatable,
{
    pub fn from_endpoints(a: AnchorPoint<T>, b: AnchorPoint<T>) -> Self {
        if a.0 == b.0 {
            return Self {
                slope: None,
                intercept: a.1,
            };
        }

        let (slope, intercept) = T::calc_coefficients(a, b);
        Self {
            slope: Some(slope),
            intercept,
        }
    }

    pub fn get(&self, x: f32) -> T {
        match self.slope {
            Some(slope) => Interpolatable::interpolate(x, slope, self.intercept),
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
