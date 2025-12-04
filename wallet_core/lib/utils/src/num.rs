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
    validate(finite, greater = 0, less_or_equal = 1)
)]
pub struct Ratio(f32);

impl Ratio {
    pub fn of_nonzero_u31(self, size: NonZeroU31) -> NonZeroU31 {
        // Calculating in f64 because of the limited precision of f32 compared to i32
        let mut result = ((i32::from(size) as f64) * (f32::from(self) as f64)).round() as u32;
        if result == 0 {
            result = 1
        }
        // source is a larger NonZeroU31
        NonZeroU31::try_new(result as i32).unwrap()
    }

    pub fn of_duration(self, duration: Duration) -> Duration {
        // Calculating in f64 because of the limited precision of f32 compared to u64
        duration.mul_f64(f32::from(self) as f64)
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
    #[case(0.0001, 1, 1)]
    #[case(1.0, i32::MAX, i32::MAX)]
    fn test_ratio_of_nonzero_u31(#[case] ratio: f32, #[case] size: i32, #[case] expected_result: i32) {
        let ratio = Ratio::try_new(ratio).unwrap();
        assert_eq!(
            ratio.of_nonzero_u31(size.try_into().unwrap()),
            NonZeroU31::try_new(expected_result).unwrap()
        )
    }
}
