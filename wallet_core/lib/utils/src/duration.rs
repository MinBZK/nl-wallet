use std::num::NonZeroU64;
use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
use serde::ser;
use serde_with::DeserializeAs;
use serde_with::SerializeAs;

pub type DurationHours = DurationMultipleSeconds<{ 60 * 60 }>;
pub type DurationDays = DurationMultipleSeconds<{ 24 * 60 * 60 }>;

/// Helper type for deserializing and serializing long non-zero durations from configurations. To be used in conjunction
/// with the `serde_with` crate. See the unit test below for an example of such use.
///
/// Note that it is an error to serialize a [`Duration`] that is not an exact multiple of `MULTIPLIER`.
pub struct DurationMultipleSeconds<const MULTIPLIER: u64>;

impl<const MULTIPLIER: u64> SerializeAs<Duration> for DurationMultipleSeconds<MULTIPLIER> {
    fn serialize_as<S>(source: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Have serialization using a type with a MULTIPLIER of 0 fail to compile, as this will result in a panic
        // because of division by 0. Note that we do not apply this to deserialization as it will simply result in a 0
        // duration and not panic.
        const { assert!(MULTIPLIER > 0, "MULTIPLIER must be non-zero") };

        let seconds = source.as_secs();
        let (increments, remainder) = (seconds / MULTIPLIER, seconds % MULTIPLIER);

        if source.subsec_nanos() != 0 || remainder > 0 {
            return Err(ser::Error::custom(format!(
                "{seconds} seconds is not a multiple of {MULTIPLIER}",
            )));
        }

        let increments =
            NonZeroU64::new(increments).ok_or_else(|| ser::Error::custom("duration should be non-zero length"))?;

        increments.serialize(serializer)
    }
}

impl<'de, const MULTIPLIER: u64> DeserializeAs<'de, Duration> for DurationMultipleSeconds<MULTIPLIER> {
    fn deserialize_as<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let increments = NonZeroU64::deserialize(deserializer)?;
        let seconds = increments.get().checked_mul(MULTIPLIER).ok_or_else(|| {
            de::Error::custom(format!(
                "multiplying {increments} by {MULTIPLIER} seconds would cause u64 overflow"
            ))
        })?;

        Ok(Duration::from_secs(seconds))
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use serde::Deserialize;
    use serde::Serialize;
    use serde_with::serde_as;

    use super::DurationDays;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct Wrapper(#[serde_as(as = "DurationDays")] Duration);

    #[test]
    fn test_serialize_deserialize() {
        let Wrapper(duration) = serde_json::from_str("7").expect("deserializing wrapper with duration should succeed");
        assert_eq!(duration, Duration::from_hours(7 * 24));

        let json = serde_json::to_string(&Wrapper(duration)).expect("serializing wrapper with duration should succeed");
        assert_eq!(json, "7");
    }

    #[test]
    fn test_deserialize_error_no_multiple() {
        let _error = serde_json::to_string(&Wrapper(Duration::from_hours(25)))
            .expect_err("serializing wrapper with duration should fail");

        let _error = serde_json::to_string(&Wrapper(Duration::from_hours(25) + Duration::from_nanos(42)))
            .expect_err("serializing wrapper with duration should fail");
    }

    #[test]
    fn test_deserialize_error_zero_duration() {
        let _error = serde_json::to_string(&Wrapper(Duration::default()))
            .expect_err("serializing wrapper with duration should fail");
    }

    #[test]
    fn test_deserialize_error_u64_overflow() {
        let _error = serde_json::from_str::<Wrapper>(&u64::MAX.to_string())
            .expect_err("deserializing wrapper with duration should fail");
    }
}
