use derive_more::AsRef;
use derive_more::From;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// Communicates that a type is optional in the specification it is derived from but implemented as mandatory due to
/// various reasons.
#[derive(Debug, Clone, From, AsRef, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecOptional<T>(T);

impl<T> SpecOptional<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// A marker field for keys whose presence is forbidden by the specification a type is derived from, and fails to
/// (de)serialize.
///
/// The field must be marked with `#[serde(default)]` (so absence yields [`Default::default()`] without invoking the
/// deserializer) and `#[serde(skip_serializing)]` (so the field is never emitted). Since the field is never read,
/// prefix its name with an underscore to silence the `dead_code` lint and use `#[serde(rename = "...")]` to map back to
/// the spec name.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use serde::Serialize;
/// use utils::spec::SpecForbidden;
///
/// #[derive(Deserialize, Serialize)]
/// struct Message {
///     #[serde(default, skip_serializing, rename = "forbidden")]
///     _forbidden: SpecForbidden,
///     name: String,
/// }
///
/// // Absence is accepted.
/// let _: Message = serde_json::from_str(r#"{"name":"foo"}"#).unwrap();
///
/// // Presence is rejected.
/// assert!(serde_json::from_str::<Message>(r#"{"name":"foo","forbidden":"x"}"#).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct SpecForbidden;

impl<'de> Deserialize<'de> for SpecForbidden {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("field MUST NOT be present"))
    }
}

impl Serialize for SpecForbidden {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("field MUST NOT be present"))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;

    use super::SpecForbidden;

    #[derive(Debug, Deserialize, Serialize)]
    struct Wrapper {
        #[serde(default, skip_serializing, rename = "forbidden")]
        _forbidden: SpecForbidden,
        keep: String,
    }

    #[test]
    fn deserializes_when_field_absent() {
        let w: Wrapper = serde_json::from_str(r#"{"keep":"value"}"#).unwrap();
        assert_eq!(w.keep, "value");
    }

    #[test]
    fn fails_to_deserialize_when_field_present() {
        let err = serde_json::from_str::<Wrapper>(r#"{"keep":"value","forbidden":"anything"}"#).unwrap_err();
        assert!(err.to_string().contains("MUST NOT be present"));
    }

    #[test]
    fn fails_to_deserialize_when_field_present_with_null() {
        let err = serde_json::from_str::<Wrapper>(r#"{"keep":"value","forbidden":null}"#).unwrap_err();
        assert!(err.to_string().contains("MUST NOT be present"));
    }

    #[test]
    fn serializes_with_field_skipped() {
        let w = Wrapper {
            _forbidden: SpecForbidden,
            keep: "value".to_string(),
        };
        assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"keep":"value"}"#);
    }

    #[test]
    fn fails_to_serialize_when_field_emitted() {
        #[derive(Serialize)]
        struct EmitForbidden {
            forbidden: SpecForbidden,
        }

        let err = serde_json::to_string(&EmitForbidden {
            forbidden: SpecForbidden,
        })
        .unwrap_err();
        assert!(err.to_string().contains("MUST NOT be present"));
    }
}
