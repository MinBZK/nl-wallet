use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

use crypto::utils::random_duration;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

#[derive(Debug, Clone)]
/// Structure to help in determining if something needs to be refreshed and how long to wait
///
/// This structures provides sane defaults. Ensure that the refresh duration is larger than the minimum delay.
pub struct RefreshControl<G = TimeGenerator>
where
    G: Generator<DateTime<Utc>>,
{
    /// Minimum refresh delay to prevent hammering and on failures
    minimum_delay: Duration,
    /// Maximum refresh delay to be robust against clock drift or time changes.
    maximum_delay: Duration,
    /// Random delay variance to break step when running for multiple instances
    delay_variance: Duration,
    /// Threshold to refresh before deadline is hit
    threshold: Duration,
    /// Generator for current time used in calculations
    time_generator: G,
}

impl<G> Default for RefreshControl<G>
where
    G: Generator<DateTime<Utc>> + Default,
{
    fn default() -> Self {
        Self {
            minimum_delay: Duration::from_secs(30),
            maximum_delay: Duration::from_secs(3600),
            delay_variance: Duration::from_secs(300),
            threshold: Duration::from_secs(600),
            time_generator: G::default(),
        }
    }
}

impl RefreshControl {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            ..Self::default()
        }
    }
}

impl<G> RefreshControl<G>
where
    G: Generator<DateTime<Utc>>,
{
    pub fn should_refresh(&self, exp: DateTime<Utc>) -> bool {
        exp - self.threshold - self.delay_variance < self.time_generator.generate()
    }

    pub fn next_refresh_delay(&self, expiries: impl IntoIterator<Item = DateTime<Utc>>) -> Duration {
        expiries
            .into_iter()
            .min()
            .map(|exp| {
                let delta = exp - self.time_generator.generate();
                let delay = delta.to_std().unwrap_or_default();

                if delay < self.minimum_delay {
                    // This can only happen if the refresh cannot update, or it updates
                    // to a point in time that is not large enough. See `check_refresh`.
                    tracing::warn!("Adjusting next refresh of {} to minimum", delta);
                    return self.minimum_delay;
                }

                // Ceil to be robust against clock drift and time changes
                let delay = std::cmp::min(self.maximum_delay, delay.saturating_sub(self.threshold));
                // Randomize to break step when running for multiple instances
                let delay = delay.saturating_sub(random_duration(self.delay_variance));
                // Floor to prevent hammering refresh
                std::cmp::max(self.minimum_delay, delay)
            })
            .unwrap_or(self.minimum_delay)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use utils::generator::mock::MockTimeGenerator;

    use super::*;

    fn assert_duration_with_variance(duration: Duration, expected: Duration, variance: Duration) {
        assert!(
            duration <= expected,
            "duration ({duration:?}) larger than expected ({expected:?})"
        );
        let minimum_expect = expected.saturating_sub(variance);
        assert!(
            duration >= minimum_expect,
            "duration ({duration:?}) less than expected minus variance ({:?})",
            minimum_expect
        );
    }

    #[rstest]
    #[case::refresh_should_happen_immediately_but_is_delayed(0, 30, 0, true)]
    #[case::refresh_should_happen_within_minimum_delay(30, 30, 0, true)]
    #[case::refresh_at_exactly_minimum_delay_plus_threshold(330, 30, 300, true)]
    #[case::refresh_at_exactly_minimum_delay_plus_threshold_plus_variance_delay(930, 330, 300, true)]
    #[case::refresh_at_exactly_maximum_delay(3600, 3000, 300, true)]
    #[case::refresh_at_larger_than_maximum_delay(7200, 3600, 300, false)]
    fn refresh_delay(
        #[case] before_timestamp: i64,
        #[case] expected_delay_sec: u64,
        #[case] maximum_variance_sec: u64,
        #[case] should_refresh: bool,
    ) {
        let control = RefreshControl {
            time_generator: MockTimeGenerator::epoch(),
            ..RefreshControl::default()
        };
        let before_instant = DateTime::<Utc>::from_timestamp(before_timestamp, 0).unwrap();
        let delay = control.next_refresh_delay(std::iter::once(before_instant));
        assert_duration_with_variance(
            delay,
            Duration::from_secs(expected_delay_sec),
            Duration::from_secs(maximum_variance_sec),
        );

        // Test if we advance time to the delay returned, the control also says it should refresh
        let control = RefreshControl {
            time_generator: MockTimeGenerator::new(DateTime::default() + delay),
            ..RefreshControl::default()
        };
        assert_eq!(control.should_refresh(before_instant), should_refresh);
    }
}
