use std::time::Duration;

use nutype::nutype;

#[nutype(
    derive(Debug, Clone, Copy, PartialEq, Eq, TryFrom, Into, Deserialize),
    validate(greater = 0)
)]
pub struct NonZeroU31(i32);

impl NonZeroU31 {
    pub fn as_usize(self) -> usize {
        self.into_inner() as usize
    }
}

#[nutype(
    derive(Debug, Clone, Copy, PartialEq, Eq, TryFrom, Into, Deserialize),
    validate(finite, greater_or_equal = 0, less_or_equal = 1)
)]
pub struct Ratio(f64);

impl Ratio {
    pub fn of_nonzero_u31(self, size: NonZeroU31) -> u32 {
        ((i32::from(size) as f64) * self.into_inner()).round() as u32
    }

    pub fn of_duration(self, duration: Duration) -> Duration {
        duration.mul_f64(self.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0.1, 99, 10)]
    #[case(0.1, 100, 10)]
    #[case(0.1, 101, 10)]
    #[case(0.0001, 1, 0)]
    #[case(1.0, i32::MAX, i32::MAX)]
    fn test_ratio_of_nonzero_u31(#[case] ratio: f64, #[case] size: i32, #[case] expected_result: i32) {
        let ratio = Ratio::try_new(ratio).unwrap();
        assert_eq!(ratio.of_nonzero_u31(size.try_into().unwrap()), expected_result as u32)
    }
}
