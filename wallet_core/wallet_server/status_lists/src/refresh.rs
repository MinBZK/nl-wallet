use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

use crypto::utils::random_duration;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

#[derive(Debug, Clone, Copy)]
/// Structure to help in determining if something needs to be refreshed and how long to wait
///
/// This structures provides sane defaults, only the threshold is configurable.
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
    /// # Panics
    /// The constructor asserts if the threshold is greater than or equal to double the variance
    pub fn new(threshold: Duration) -> Self {
        let control = RefreshControl::default();
        // This can never happen with the settings that are configured, since threshold is set in hours.
        // Note that also the minimum delay is checked to be less than threshold via this assertion.
        assert!(
            threshold >= control.delay_variance * 2,
            "Threshold {:?} must be greater than or equal to double the variance: {:?}",
            threshold,
            control.delay_variance * 2
        );
        Self { threshold, ..control }
    }
}

impl<G> RefreshControl<G>
where
    G: Generator<DateTime<Utc>>,
{
    pub fn should_refresh(&self, expiry: DateTime<Utc>) -> bool {
        expiry - self.threshold < self.time_generator.generate()
    }

    pub fn next_refresh_delay(&self, expiries: impl IntoIterator<Item = DateTime<Utc>>) -> Duration {
        expiries
            .into_iter()
            .min()
            .map(|exp| {
                let delta_to_expiry = exp - self.time_generator.generate();
                let delay_to_expiry = delta_to_expiry.to_std().unwrap_or_default();

                // Special check to see if we are not too late
                if delay_to_expiry < self.minimum_delay {
                    // This can only happen if the refresh cannot update, or it updates
                    // to a point in time that is not large enough.
                    tracing::warn!("Adjusting next refresh of {} to minimum", delta_to_expiry);
                    return self.minimum_delay;
                }

                // Maximize delay to be robust against clock drift and time changes
                let mut delay = std::cmp::min(self.maximum_delay, delay_to_expiry.saturating_sub(self.threshold));

                // Randomize delay to break step when running for multiple instances.
                // Constructor restricts threshold to be larger than double the variance.
                // This prevents the `delay` to be larger than the `delay_to_expiry`.
                delay += random_duration(self.delay_variance);

                // Minimize delay to prevent hammering refresh
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
            duration >= expected,
            "duration ({duration:?}) less than expected ({expected:?})"
        );
        let maximum_expect = expected + variance;
        assert!(
            duration <= maximum_expect,
            "duration ({duration:?}) greater than expected plus variance ({:?})",
            maximum_expect
        );
    }

    #[rstest]
    #[case::refresh_should_happen_immediately_but_is_delayed(0, 30, 0, true)]
    #[case::refresh_should_happen_within_minimum_delay(20, 30, 0, true)]
    #[case::refresh_at_exactly_threshold(600, 30, 270, true)]
    #[case::refresh_at_exactly_maximum_delay(3600, 3000, 300, true)]
    #[case::refresh_at_greater_than_maximum_delay(7200, 3600, 300, false)]
    fn refresh_delay(
        #[case] expiry_timestamp: i64,
        #[case] expected_delay_sec: u64,
        #[case] maximum_variance_sec: u64,
        #[case] should_refresh: bool,
    ) {
        let control = RefreshControl {
            time_generator: MockTimeGenerator::epoch(),
            ..RefreshControl::default()
        };
        let expiry_date_time = DateTime::<Utc>::from_timestamp(expiry_timestamp, 0).unwrap();
        let delay = control.next_refresh_delay(std::iter::once(expiry_date_time));
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
        assert_eq!(control.should_refresh(expiry_date_time), should_refresh);
    }
}
