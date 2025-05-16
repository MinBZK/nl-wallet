use chrono::DateTime;
use chrono::Timelike;
use chrono::Utc;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::TimestampSeconds;

/// Newtype around `DateTime<Utc>` having only seconds precision.
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Into, Serialize, Deserialize)]
pub struct DateTimeSeconds(#[serde_as(as = "TimestampSeconds<i64>")] DateTime<Utc>);

impl DateTimeSeconds {
    pub fn new(datetime: DateTime<Utc>) -> Self {
        // Ensure only seconds precision
        Self(datetime.with_nanosecond(0).unwrap())
    }
}

impl From<DateTime<Utc>> for DateTimeSeconds {
    fn from(value: DateTime<Utc>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod test {
    use chrono::Timelike;
    use chrono::Utc;

    use super::DateTimeSeconds;

    #[test]
    fn test_constructor() {
        let now = Utc::now();
        let datetime_seconds = DateTimeSeconds::new(now);
        assert_ne!(now, datetime_seconds.into());
        assert_eq!(now.with_nanosecond(0).unwrap(), datetime_seconds.into());
    }

    #[test]
    fn test_serialize_deserialize() {
        let now = Utc::now();
        let now_json = serde_json::to_string(&now.timestamp()).unwrap();

        let deserialized: DateTimeSeconds = serde_json::from_str(&now_json).unwrap();
        assert_eq!(now.with_nanosecond(0).unwrap(), deserialized.into());

        let serialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(&now_json, &serialized);

        let deserialized_from_serialized: DateTimeSeconds = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, deserialized_from_serialized);
    }
}
