//! Serde helper for (de)serializing a [`Duration`] as a whole number of days, for use with `#[serde(with = ...)]`.

use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;

pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(duration.as_secs() / (24 * 60 * 60))
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    let days = u64::deserialize(deserializer)?;
    Ok(Duration::from_hours(days * 24))
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Wrapper(#[serde(with = "super")] Duration);

    #[test]
    fn test_serialize_deserialize() {
        let duration = Wrapper(Duration::from_hours(24 * 7));
        let json = serde_json::to_string(&duration).unwrap();
        assert_eq!(json, "7");

        let deserialized: Wrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(duration, deserialized);
    }
}
