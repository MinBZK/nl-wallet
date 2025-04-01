use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;

use http::Uri;
use itertools::Itertools;
use jsonschema::Draft;
use jsonschema::ValidationError;
use jsonschema::Validator;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::MapSkipError;
use ssri::Integrity;

use wallet_common::spec::SpecOptional;
use wallet_common::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataError {
    #[error("json schema validation failed: {0}")]
    JsonSchemaValidation(#[from] ValidationError<'static>),

    #[error("could not deserialize JSON schema: {0}")]
    JsonSchema(#[from] serde_json::Error),

    #[error("schema option {0:?} is not supported")]
    UnsupportedSchemaOption(SchemaOption),

    #[error("detected claim path collision: {0}")]
    ClaimPathCollision(String),

    #[error("detected duplicate `svg_id`s: {0:?}")]
    DuplicateSvgIds(Vec<String>),

    #[error("found missing `svg_id`s: {0:?}")]
    MissingSvgIds(Vec<String>),
}

/// SD-JWT VC type metadata document.
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
///
/// Note that within the context of the wallet app we place additional constraints on the contents of this document,
/// most of which stem from practical concerns. These constraints consist of the following:
///
/// * Some optional fields we consider as mandatory. These are marked by the `SpecOptionalImplRequired` type.
/// * Attributes contained in arrays are not (yet) supported.
/// * Optional attributes are not yet supported.
/// * Every attribute in the attestation received from the issuer should be covered by the JSON schema, so that its data
///   type is known.
/// * Every attribute in the attestation received from the issuer should have corresponding claim metadata, so that the
///   attribute can be rendered for display to the user.
/// * Claims that cover a group of attributes are not (yet) supported and will not be accepted, as rendering groups of
///   attributes covered by the same display data is not supported by the UI.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncheckedTypeMetadata {
    /// A String or URI that uniquely identifies the type.
    pub vct: String,

    /// A human-readable name for the type, intended for developers reading the JSON document.
    pub name: Option<String>,

    /// A human-readable description for the type, intended for developers reading the JSON document.
    pub description: Option<String>,

    /// Another type that this type extends.
    #[serde(flatten)]
    pub extends: Option<MetadataExtends>,

    /// An array of objects containing display information for the type.
    #[serde(default)]
    pub display: Vec<DisplayMetadata>,

    /// An array of objects containing claim information for the type.
    #[serde(default)]
    pub claims: Vec<ClaimMetadata>,

    /// A JSON Schema document describing the structure of the Verifiable Credential
    #[serde(flatten)]
    pub schema: SchemaOption,
}

#[nutype(
    derive(Debug, Clone, AsRef, PartialEq, Eq, Serialize, Deserialize),
    validate(with = UncheckedTypeMetadata::check_metadata_consistency, error = TypeMetadataError),
)]
pub struct TypeMetadata(UncheckedTypeMetadata);

impl UncheckedTypeMetadata {
    pub fn check_metadata_consistency(unchecked_metadata: &UncheckedTypeMetadata) -> Result<(), TypeMetadataError> {
        unchecked_metadata.detect_path_collisions()?;
        unchecked_metadata.validate_svg_ids()?;

        Ok(())
    }

    fn detect_path_collisions(&self) -> Result<(), TypeMetadataError> {
        let mut paths: HashSet<String> = HashSet::new();

        for claim in &self.claims {
            // Flatten all claim paths by joining them with a '.'
            let flattened_key = claim
                .path
                .iter()
                .filter_map(|path| path.try_key_path())
                .collect::<Vec<_>>()
                .join(".");

            // If the flattened key is already present in the set, this means
            // that two different claims paths lead to the same flattened path.
            if paths.contains(&flattened_key) {
                return Err(TypeMetadataError::ClaimPathCollision(flattened_key));
            }

            paths.insert(flattened_key);
        }

        Ok(())
    }

    fn validate_svg_ids(&self) -> Result<(), TypeMetadataError> {
        // `svg_id` MUST be unique within the type metadata.
        let svg_ids = self
            .claims
            .iter()
            .filter_map(|claim| claim.svg_id.as_ref())
            .collect_vec();
        match svg_ids.iter().duplicates().collect_vec() {
            dups if !dups.is_empty() => Err(TypeMetadataError::DuplicateSvgIds(
                dups.into_iter().copied().cloned().collect_vec(),
            )),
            _ => Ok(()),
        }?;

        // If the svg_id is not present in the claim metadata, the consuming application SHOULD reject the
        // SVG template.
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        match self
            .display
            .iter()
            .filter_map(|display| display.summary.as_ref())
            .flat_map(|summary| re.captures_iter(summary).flat_map(|s| s.extract::<1>().1))
            .unique()
            .filter(|id| !svg_ids.contains(&&id.to_string()))
            .collect_vec()
        {
            missing_ids if !missing_ids.is_empty() => Err(TypeMetadataError::MissingSvgIds(
                missing_ids.iter().map(|s| s.to_string()).collect_vec(),
            )),
            _ => Ok(()),
        }?;

        Ok(())
    }
}

impl TypeMetadata {
    pub fn validate(&self, json_claims: &serde_json::Value) -> Result<(), TypeMetadataError> {
        if let SchemaOption::Embedded { schema } = &self.as_ref().schema {
            schema
                .validator
                .validate(json_claims)
                .map_err(ValidationError::to_owned)?;
            Ok(())
        } else {
            Err(TypeMetadataError::UnsupportedSchemaOption(self.as_ref().schema.clone()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataExtends {
    /// A string or URI of another type that this type extends.
    pub extends: String,

    /// Validating the integrity of the extends field.
    /// Note that this is optional in the specification, but we consider this mandatory:
    /// * If the metadata this type extends is fetched from an external URI, the integrity digest guarantees that its
    ///   contents match what is expected by the issuer.
    /// * If the metadata is included with issuance, e.g. in an unprotected header, a chain of integrity digests that
    ///   starts from a digest included in a signed section of the attestation acts as a de facto signature, protecting
    ///   against tampering. In SD-JWT the `vct#integrity` claim would contain this first digest.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: SpecOptional<Integrity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOption {
    Embedded {
        /// An embedded JSON Schema document describing the structure of the Verifiable Credential.
        schema: Box<JsonSchema>,
    },
    Remote {
        /// A URL pointing to a JSON Schema document describing the structure of the Verifiable Credential.
        #[serde(with = "http_serde::uri")]
        schema_uri: Uri,
        /// Validating the integrity of the schema_uri field.
        /// Note that although this is optional in the specification, we consider validation using a digest mandatory
        /// if the schema is to be fetched from an external URI, in order to check that this matches the
        /// contents as intended by the issuer.
        #[serde(rename = "schema_uri#integrity")]
        schema_uri_integrity: SpecOptional<Integrity>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Value", into = "serde_json::Value")]
pub struct JsonSchema {
    raw_schema: serde_json::Value,

    // When deserializing, the JSON schema properties are parsed so the metadata describing the attributes can be used
    // when converting to wallet internal types.
    properties: JsonSchemaProperties,

    // The validator is instantiated once and (meta)validated upon deserialization.
    validator: Validator,
}

impl JsonSchema {
    fn try_new(raw_schema: serde_json::Value) -> Result<JsonSchema, TypeMetadataError> {
        let properties: JsonSchemaProperties = serde_json::from_value(raw_schema.clone())?;
        let validator = Self::build_validator(&raw_schema)?;

        Ok(Self {
            raw_schema,
            properties,
            validator,
        })
    }

    // Building the validator for the 202012 draft also validates the JSON Schema itself.
    fn build_validator(raw_schema: &serde_json::Value) -> Result<Validator, ValidationError<'static>> {
        jsonschema::options()
            .should_validate_formats(true)
            .with_draft(Draft::Draft202012)
            .build(raw_schema)
            .map_err(ValidationError::to_owned)
    }

    pub fn into_properties(self) -> JsonSchemaProperties {
        self.properties
    }
}

impl From<JsonSchema> for serde_json::Value {
    fn from(value: JsonSchema) -> Self {
        value.raw_schema
    }
}

impl TryFrom<serde_json::Value> for JsonSchema {
    type Error = TypeMetadataError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl Clone for JsonSchema {
    fn clone(&self) -> Self {
        Self {
            raw_schema: self.raw_schema.clone(),
            properties: self.properties.clone(),
            // Unwrap is safe here, since having a valid validator is a prerequisite for constructing a JsonSchema
            validator: Self::build_validator(&self.raw_schema).unwrap(),
        }
    }
}

impl PartialEq for JsonSchema {
    fn eq(&self, other: &Self) -> bool {
        self.raw_schema == other.raw_schema
    }
}

impl Eq for JsonSchema {}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaProperties {
    // A HashMap is used here since there are no uses that rely on the order of the properties
    #[serde_as(as = "MapSkipError<_, _>")]
    pub properties: HashMap<String, JsonSchemaProperty>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaProperty {
    #[serde(rename = "type")]
    pub r#type: JsonSchemaPropertyType,

    pub format: Option<JsonSchemaPropertyFormat>,

    // A HashMap is used here since there are no uses that rely on the order of the properties
    #[serde_as(as = "Option<MapSkipError<_, _>>")]
    pub properties: Option<HashMap<String, JsonSchemaProperty>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaPropertyType {
    String,
    Number,
    Integer,
    Object,
    Array,
    Boolean,
    Null,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaPropertyFormat {
    Date,
    #[serde(other)]
    Other,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayMetadata {
    ///  A language tag as defined in Section 2 of [RFC5646].
    pub lang: String,

    /// A human-readable name for the type, intended for end users.
    pub name: String,

    /// A human-readable description for the type, intended for end users.
    pub description: Option<String>,

    /// A templated summary for the type, intended to be rendered to the end user.
    pub summary: Option<String>,

    /// An object containing rendering information for the type
    pub rendering: Option<RenderingMetadata>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderingMetadata {
    Simple {
        logo: Option<LogoMetadata>,
        background_color: Option<String>,
        text_color: Option<String>,
    },
    SvgTemplates,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoMetadata {
    #[serde(with = "http_serde::uri")]
    pub uri: Uri,

    /// Note that although this is optional in the specification, we consider validation using a digest mandatory if
    /// the logo is to be fetched from an external URI, in order to check that this matches the image as intended
    /// by the issuer.
    #[serde(rename = "uri#integrity")]
    pub uri_integrity: SpecOptional<Integrity>,

    /// Note that although this is optional in the specification, it is mandatory within the context of the wallet app
    /// because of accessibility requirements.
    pub alt_text: SpecOptional<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMetadata {
    /// A list indicating the claim or claims that are being addressed, as described below.
    pub path: VecNonEmpty<ClaimPath>,

    /// A list of objects containing display information for the claim.  The array contains an object for each
    /// language that is supported by the type.
    #[serde(default)]
    pub display: Vec<ClaimDisplayMetadata>,

    /// Indicates whether the claim is selectively disclosable
    #[serde(default)]
    pub sd: ClaimSelectiveDisclosureMetadata,

    /// A string defining the ID of the claim for reference in the SVG template.
    pub svg_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimPath {
    SelectByKey(String),
    SelectAll,
    SelectByIndex(usize),
}

impl ClaimPath {
    pub fn try_key_path(&self) -> Option<&str> {
        match self {
            ClaimPath::SelectByKey(key) => Some(key.as_str()),
            _ => None,
        }
    }

    pub fn try_into_key_path(self) -> Option<String> {
        match self {
            ClaimPath::SelectByKey(key) => Some(key),
            _ => None,
        }
    }
}

impl Display for ClaimPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimPath::SelectByKey(key) => write!(f, "{}", key),
            ClaimPath::SelectAll => f.write_str("*"),
            ClaimPath::SelectByIndex(index) => write!(f, "{}", index),
        }
    }
}

impl Display for ClaimMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.path.iter().fold(String::new(), |mut output, p| {
                let _ = write!(output, "[{p}]");
                output
            })
        )
    }
}

/// An indication whether the claim is selectively disclosable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimSelectiveDisclosureMetadata {
    /// The Issuer MUST make the claim selectively disclosable.
    Always,

    /// The Issuer MAY make the claim selectively disclosable.
    #[default]
    Allowed,

    /// The Issuer MUST NOT make the claim selectively disclosable.
    Never,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

#[cfg(any(test, feature = "example_constructors"))]
mod example_constructors {
    use std::collections::HashMap;

    use serde_json::json;

    use crypto::utils::random_string;

    use crate::examples::ADDRESS_METADATA_BYTES;
    use crate::examples::CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES;
    use crate::examples::EXAMPLE_EXTENSION_METADATA_BYTES;
    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::PID_METADATA_BYTES;
    use crate::examples::SD_JWT_VC_SPEC_METADATA_BYTES;

    use super::ClaimDisplayMetadata;
    use super::ClaimMetadata;
    use super::ClaimPath;
    use super::ClaimSelectiveDisclosureMetadata;
    use super::JsonSchema;
    use super::JsonSchemaProperties;
    use super::JsonSchemaProperty;
    use super::JsonSchemaPropertyFormat;
    use super::JsonSchemaPropertyType;
    use super::SchemaOption;
    use super::TypeMetadata;
    use super::UncheckedTypeMetadata;

    impl UncheckedTypeMetadata {
        pub fn empty_example() -> Self {
            Self {
                vct: random_string(16),
                name: Some(random_string(8)),
                description: None,
                extends: None,
                display: vec![],
                claims: vec![],
                schema: SchemaOption::Embedded {
                    schema: Box::new(JsonSchema::try_new(json!({"properties": {}})).unwrap()),
                },
            }
        }
    }

    impl TypeMetadata {
        pub fn empty_example() -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata::empty_example()).unwrap()
        }

        pub fn empty_example_with_attestation_type(attestation_type: &str) -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata {
                vct: String::from(attestation_type),
                ..UncheckedTypeMetadata::empty_example()
            })
            .unwrap()
        }

        pub fn example_with_claim_name(
            attestation_type: &str,
            name: &str,
            r#type: JsonSchemaPropertyType,
            format: Option<JsonSchemaPropertyFormat>,
        ) -> Self {
            Self::example_with_claim_names(attestation_type, &[(name, r#type, format)])
        }

        pub fn example_with_claim_names(
            attestation_type: &str,
            names: &[(&str, JsonSchemaPropertyType, Option<JsonSchemaPropertyFormat>)],
        ) -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata {
                vct: String::from(attestation_type),
                claims: names
                    .iter()
                    .map(|(name, _, _)| ClaimMetadata {
                        path: vec![ClaimPath::SelectByKey(String::from(*name))].try_into().unwrap(),
                        display: vec![ClaimDisplayMetadata {
                            lang: String::from("en"),
                            label: name.to_uppercase(),
                            description: None,
                        }],
                        sd: ClaimSelectiveDisclosureMetadata::Always,
                        svg_id: None,
                    })
                    .collect(),
                schema: SchemaOption::Embedded {
                    schema: Box::new(JsonSchema::example_with_claim_names(names)),
                },
                ..UncheckedTypeMetadata::empty_example()
            })
            .unwrap()
        }

        pub fn example() -> Self {
            serde_json::from_slice(EXAMPLE_METADATA_BYTES).unwrap()
        }

        pub fn example_extension() -> Self {
            serde_json::from_slice(EXAMPLE_EXTENSION_METADATA_BYTES).unwrap()
        }

        pub fn pid_example() -> Self {
            serde_json::from_slice(PID_METADATA_BYTES).unwrap()
        }

        pub fn address_example() -> Self {
            serde_json::from_slice(ADDRESS_METADATA_BYTES).unwrap()
        }

        pub fn sd_jwt_vc_spec_example() -> Self {
            serde_json::from_slice(SD_JWT_VC_SPEC_METADATA_BYTES).unwrap()
        }

        pub fn credential_payload_sd_jwt_metadata() -> Self {
            serde_json::from_slice(CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES).unwrap()
        }
    }

    impl JsonSchema {
        fn example_with_claim_names(
            names: &[(&str, JsonSchemaPropertyType, Option<JsonSchemaPropertyFormat>)],
        ) -> Self {
            let properties = JsonSchemaProperties {
                properties: HashMap::from_iter(names.iter().map(|(name, prop_type, prop_format)| {
                    (
                        String::from(*name),
                        JsonSchemaProperty {
                            r#type: *prop_type,
                            format: *prop_format,
                            properties: None,
                        },
                    )
                })),
            };

            let raw_schema = serde_json::to_value(&properties).unwrap();
            let validator = JsonSchema::build_validator(&raw_schema).unwrap();

            Self {
                raw_schema,
                properties,
                validator,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use jsonschema::error::ValidationErrorKind;
    use jsonschema::ValidationError;
    use rstest::rstest;
    use serde_json::json;

    use super::ClaimPath;
    use super::MetadataExtends;
    use super::SchemaOption;
    use super::TypeMetadata;
    use super::TypeMetadataError;
    use super::UncheckedTypeMetadata;

    #[test]
    fn test_deserialize() {
        let metadata = TypeMetadata::example();
        assert_eq!(
            "https://sd_jwt_vc_metadata.example.com/example_credential",
            metadata.as_ref().vct
        );
    }

    #[test]
    fn test_extends() {
        let metadata = serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "extends": "https://sd_jwt_vc_metadata.example.com/other_schema",
            "extends#integrity": "sha256-LmXfh-9cLlJNXN-TsMk-PmKjZ5t0WRL5ca_xGgX3c1V",
            "display": [],
            "schema_uri": "https://sd_jwt_vc_metadata.example.com/",
            "schema_uri#integrity": "sha256-9cLlJNXN-TsMk-PmKjZ5t0WRL5ca_xGgX3c1VLmXfh-WRL5",
        }))
        .unwrap();

        assert_matches!(metadata.as_ref().extends, Some(MetadataExtends { .. }));
        assert_matches!(metadata.as_ref().schema, SchemaOption::Remote { .. });
    }

    #[test]
    fn test_embedded_schema_validation() {
        assert!(serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "extends": "https://sd_jwt_vc_metadata.example.com/other_schema",
            "extends#integrity": "sha256-LmXfh-9cLlJNXN-TsMk-PmKjZ5t0WRL5ca_xGgX3c1V",
            "display": [],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "flobject",
                "properties": {
                    "vct": {
                        "type": "string"
                    }
                }
            }
        }))
        .is_err());
    }

    #[test]
    fn test_schema_validation_success() {
        let metadata = TypeMetadata::example();

        let claims = json!({
          "vct": "https://credentials.example.com/identity_credential",
          "iss": "https://example.com/issuer",
          "nbf": 1683000000,
          "iat": 1683000000,
          "exp": 1883000000,
          "attestation_qualification": "EAA",
          "place_of_birth": {
            "locality": "DE"
          }
        });

        assert_eq!(
            vec![
                ClaimPath::SelectByKey(String::from("place_of_birth")),
                ClaimPath::SelectByKey(String::from("country")),
                ClaimPath::SelectByKey(String::from("area_code")),
            ],
            metadata.as_ref().claims[3].path.clone().into_inner()
        );

        metadata.validate(&claims).unwrap();
    }

    #[test]
    fn test_schema_validation_failure() {
        let metadata = TypeMetadata::example();

        let claims = json!({
          "vct": "https://credentials.example.com/identity_credential",
          "iss": "https://example.com/issuer",
          "iat": 1683000000,
          "attestation_qualification": "EAA",
          "financial": {
            "has_job": "yes"
          }
        });

        assert!(metadata.validate(&claims).is_err());
    }

    #[rstest]
    #[case("2004-12-25")]
    #[case("2024-02-29")]
    fn test_schema_validation_date_format_happy(#[case] date_str: &str) {
        let metadata = TypeMetadata::example();

        let claims = json!({
            "vct": "https://credentials.example.com/identity_credential",
            "iss": "https://example.com/issuer",
            "iat": 1683000000,
            "attestation_qualification": "EAA",
            "birth_date": date_str,
        });
        metadata.validate(&claims).unwrap();
    }

    #[rstest]
    #[case("not_a_date")]
    #[case("2025-02-29")]
    #[case("01-01-2000")]
    fn test_schema_validation_date_format_error(#[case] date_str: &str) {
        let metadata = TypeMetadata::example();

        let claims = json!({
            "vct": "https://credentials.example.com/identity_credential",
            "iss": "https://example.com/issuer",
            "iat": 1683000000,
            "attestation_qualification": "EAA",
            "birth_date": date_str,
        });

        assert_matches!(
            metadata.validate(&claims),
            Err(TypeMetadataError::JsonSchemaValidation(ValidationError {
                instance,
                kind: ValidationErrorKind::Format { format },
                instance_path,
                ..
            })) if instance.to_string() == format!("\"{}\"", date_str)
                    && format == "date" && instance_path.to_string() == "/birth_date"
        );
    }

    #[rstest]
    #[case(vec![vec!["a.b"], vec!["a", "b"]], "a.b")]
    #[case(vec![vec!["x.y.z"], vec!["x", "y.z"]], "x.y.z")]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y.z"]], "x.y.z")]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y", "z"]], "x.y.z")]
    #[case(vec![vec!["x", "y.z"], vec!["x.y", "z"]], "x.y.z")]
    fn test_claim_path_collision(#[case] claims: Vec<Vec<&str>>, #[case] expected_path: &str) {
        let result = serde_json::from_value::<UncheckedTypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "claims": claims.into_iter().map(|claim| HashMap::from([("path", claim)])).collect::<Vec<_>>(),
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        }))
        .unwrap()
        .detect_path_collisions();

        assert_matches!(result, Err(TypeMetadataError::ClaimPathCollision(path)) if path == expected_path);
    }

    #[test]
    fn should_detect_claim_path_collision_for_deserializing_typemetadata() {
        let result = serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "claims": [
                { "path": ["address.street"] },
                { "path": ["address", "street"] },
            ],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        }))
        .expect_err("Should fail deserializing type metadata because of path collision");

        assert!(result.to_string().contains("detected claim path collision"));
    }

    #[rstest]
    #[case(json!([
        { "path": vec!["address"] },
    ]), Ok(()))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
    ]), Ok(()))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"] },
    ]), Ok(()))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"], "svg_id": "address_city" },
    ]), Ok(()))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address_street" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"] },
    ]), Err(TypeMetadataError::DuplicateSvgIds(vec!["address_street".to_owned()])))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"], "svg_id": "address_street" },
    ]), Err(TypeMetadataError::DuplicateSvgIds(vec!["address_street".to_owned()])))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"], "svg_id": "address" },
        { "path": vec!["address", "number"], "svg_id": "address_street" },
    ]), Err(TypeMetadataError::DuplicateSvgIds(vec!["address".to_owned(), "address_street".to_owned()])))]
    #[case(json!([
        { "path": vec!["address"], "svg_id": "address" },
        { "path": vec!["address", "street"], "svg_id": "address_street" },
        { "path": vec!["address", "city"], "svg_id": "address_street" },
        { "path": vec!["address", "number"], "svg_id": "address" },
    ]), Err(TypeMetadataError::DuplicateSvgIds(vec!["address_street".to_owned(), "address".to_owned()])))]
    fn test_claim_svg_ids(#[case] claims: serde_json::Value, #[case] expected: Result<(), TypeMetadataError>) {
        let mut metadata = serde_json::from_value::<UncheckedTypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "claims": [],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        }))
        .unwrap();
        metadata.claims = serde_json::from_value(claims).unwrap();

        let result = metadata.validate_svg_ids();
        match (result, expected) {
            (Ok(()), Ok(())) => {}
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Err(e), Ok(())) => {
                panic!("assertion failed\n left: {e:?}\nright: ()")
            }
            (Ok(()), Err(e)) => {
                panic!("assertion failed\n left: ()\nright: {e:?}")
            }
        };
    }

    #[rstest]
    #[case("{{address}}", Ok(()))]
    #[case("{{address_street}}", Ok(()))]
    #[case("{{address_street}} {{address_city}}", Ok(()))]
    #[case(
        "{{address_street}} {{address_number}}",
        Err(TypeMetadataError::MissingSvgIds(vec!["address_number".to_owned()]))
    )]
    #[case(
        "{{address_country}} {{address_number}}",
        Err(TypeMetadataError::MissingSvgIds(vec!["address_country".to_owned(), "address_number".to_owned()])))]
    #[case("{{address_number}} {{address_country}} {{address_number}}", Err(TypeMetadataError::MissingSvgIds(vec![
        "address_number".to_owned(),
        "address_country".to_owned()
    ])))]
    fn should_detect_missing_svg_ids(#[case] summary: &str, #[case] expected: Result<(), TypeMetadataError>) {
        let metadata = serde_json::from_value::<UncheckedTypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "display": [{
                    "lang": "en",
                    "name": "Example Credential",
                    "summary": summary,
                }
            ],
            "claims": [
                { "path": vec!["address"], "svg_id": "address" },
                { "path": vec!["address", "street"], "svg_id": "address_street" },
                { "path": vec!["address", "city"], "svg_id": "address_city" },
                { "path": vec!["address", "number"] },
            ],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        }))
        .unwrap();
        let result = metadata.validate_svg_ids();
        match (result, expected) {
            (Ok(()), Ok(())) => {}
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Err(e), Ok(())) => {
                panic!("assertion failed\n left: {e:?}\nright: ()")
            }
            (Ok(()), Err(e)) => {
                panic!("assertion failed\n left: ()\nright: {e:?}")
            }
        };
    }
}
