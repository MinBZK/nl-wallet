use std::str::FromStr;

use attestation_data::attributes::Attribute;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Visitor;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

/// The [ISO/IEC 5218] `sex` codes.
///
/// [ISO/IEC 5218]: https://en.wikipedia.org/wiki/ISO/IEC_5218
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, DeserializeFromStr, SerializeDisplay,
)]
#[strum(serialize_all = "lowercase")]
pub enum IsoSex {
    NotKnown = 0,
    Male = 1,
    Female = 2,
    NotApplicable = 9,
}

/// PID Rulebook (section 2.3) `sex` attribute values.
///
/// [`Self::Iso`] covers the [ISO/IEC 5218] values (see [`IsoSex`]); the remaining variants are Rulebook additions on
/// top of that standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebookSex {
    Iso(IsoSex),
    Other,
    Inter,
    Diverse,
    Open,
}

impl From<RulebookSex> for Attribute {
    fn from(value: RulebookSex) -> Attribute {
        match value {
            RulebookSex::Iso(iso) => Attribute::Number(iso as i64),
            RulebookSex::Other => Attribute::Number(3),
            RulebookSex::Inter => Attribute::Number(4),
            RulebookSex::Diverse => Attribute::Number(5),
            RulebookSex::Open => Attribute::Number(6),
        }
    }
}

impl<'de> Deserialize<'de> for RulebookSex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RulebookVisitor;

        impl<'de> Visitor<'de> for RulebookVisitor {
            type Value = RulebookSex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string with the sex constant")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "other" => Ok(RulebookSex::Other),
                    "inter" => Ok(RulebookSex::Inter),
                    "diverse" => Ok(RulebookSex::Diverse),
                    "open" => Ok(RulebookSex::Open),
                    _ => IsoSex::from_str(v)
                        .map(RulebookSex::Iso)
                        .map_err(|_| E::custom(format!("invalid sex '{v}'"))),
                }
            }
        }

        deserializer.deserialize_str(RulebookVisitor)
    }
}

impl Serialize for RulebookSex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RulebookSex::Iso(iso) => iso.serialize(serializer),
            RulebookSex::Other => "other".serialize(serializer),
            RulebookSex::Inter => "inter".serialize(serializer),
            RulebookSex::Diverse => "diverse".serialize(serializer),
            RulebookSex::Open => "open".serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;
    use serde_json::value::Value;

    use super::IsoSex;
    use super::RulebookSex;

    #[rstest]
    #[case(RulebookSex::Iso(IsoSex::NotKnown), "notknown")]
    #[case(RulebookSex::Other, "other")]
    #[case(RulebookSex::Inter, "inter")]
    #[case(RulebookSex::Diverse, "diverse")]
    #[case(RulebookSex::Open, "open")]
    fn test_serde(#[case] sex: RulebookSex, #[case] expected: &str) {
        let serialized = serde_json::to_value(sex).unwrap();
        assert_eq!(json!(expected), serialized);
        assert_eq!(sex, serde_json::from_value(serialized).unwrap());
    }

    #[test]
    fn test_deserialize_invalid_string() {
        let error = serde_json::from_value::<RulebookSex>(json!("misc")).unwrap_err();

        assert_eq!(error.to_string(), "invalid sex 'misc'");
    }

    #[test]
    fn test_deserialize_invalid_type() {
        let error = serde_json::from_value::<RulebookSex>(Value::Null).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid type: null, expected a string with the sex constant"
        );
    }
}
