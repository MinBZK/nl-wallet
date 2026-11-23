use attestation_data::attributes::Attribute;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

/// The [ISO/IEC 5218] `sex` codes.
///
/// [ISO/IEC 5218]: https://en.wikipedia.org/wiki/ISO/IEC_5218
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, DeserializeFromStr)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RulebookSex {
    Other,
    Inter,
    Diverse,
    Open,
    #[serde(untagged)]
    Iso(IsoSex),
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

impl From<IsoSex> for RulebookSex {
    fn from(value: IsoSex) -> RulebookSex {
        RulebookSex::Iso(value)
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
    fn test_serde(#[case] sex: RulebookSex, #[case] serialized: &str) {
        assert_eq!(sex, serde_json::from_value(Value::String(serialized.into())).unwrap());
    }

    #[test]
    fn test_deserialize_invalid_string() {
        let error = serde_json::from_value::<RulebookSex>(json!("misc")).unwrap_err();

        assert_eq!(
            error.to_string(),
            "data did not match any variant of untagged enum RulebookSex",
        );
    }

    #[test]
    fn test_deserialize_invalid_type() {
        let error = serde_json::from_value::<RulebookSex>(Value::Null).unwrap_err();

        assert_eq!(
            error.to_string(),
            "data did not match any variant of untagged enum RulebookSex",
        );
    }
}
