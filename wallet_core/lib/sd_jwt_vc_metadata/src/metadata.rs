use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
use std::ops::Deref;
use std::sync::LazyLock;

use http::Uri;
use itertools::Itertools;
use jsonschema::Draft;
use jsonschema::ValidationError;
use jsonschema::Validator;
use nutype::nutype;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::MapSkipError;
use ssri::Integrity;

use http_utils::data_uri::DataUri;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;

// The requirements for the svg_id according to the specification are:
// "It MUST consist of only alphanumeric characters and underscores and MUST NOT start with a digit."
static SVG_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][0-9A-Za-z_]*$").unwrap());
static TEMPLATE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{([A-Za-z_][0-9A-Za-z_]*)\}\}").unwrap());

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataError {
    #[error("json schema validation failed: {0}")]
    JsonSchemaValidation(#[from] ValidationError<'static>),

    #[error("could not deserialize JSON schema: {0}")]
    JsonSchema(#[from] serde_json::Error),

    #[error("detected claim path collision: {0}")]
    ClaimPathCollision(String),

    #[error("detected duplicate display metadata language(s): {}", .0.join(", "))]
    DuplicateDisplayLanguages(Vec<String>),

    #[error(
        "detected duplicate claim display metadata language(s) at path {}: {}",
        ClaimMetadata::path_to_string(.0.as_ref()),
        .1.join(", ")
    )]
    DuplicateClaimDisplayLanguages(VecNonEmpty<ClaimPath>, Vec<String>),

    #[error("detected duplicate `svg_id`s: {}", .0.join(", "))]
    DuplicateSvgIds(Vec<String>),

    #[error("found missing `svg_id`s: {}", .0.join(", "))]
    MissingSvgIds(Vec<String>),
}

/// SD-JWT VC type metadata document.
/// See: https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
///
/// Note that within the context of the wallet app we place additional constraints on the contents of this document,
/// most of which stem from practical concerns. These constraints consist of the following:
///
/// * Some optional fields we consider as mandatory. These are marked by the `SpecOptional` type.
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

pub(crate) fn find_missing_svg_ids(display: &[DisplayMetadata], claims: &[ClaimMetadata]) -> Vec<String> {
    let svg_ids = claims
        .iter()
        .filter_map(|claim| claim.svg_id.as_deref())
        .collect::<HashSet<_>>();

    display
        .iter()
        .filter_map(|display| display.summary.as_deref())
        .flat_map(|summary| {
            TEMPLATE_REGEX
                .captures_iter(summary)
                .flat_map(|captures| captures.extract::<1>().1)
        })
        .unique()
        .filter(|id| !svg_ids.contains(id))
        .map(ToString::to_string)
        .collect()
}

#[nutype(
    derive(Debug, Clone, AsRef, PartialEq, Eq, Into, Serialize, Deserialize),
    validate(with = UncheckedTypeMetadata::check_metadata_consistency, error = TypeMetadataError),
)]
pub struct TypeMetadata(UncheckedTypeMetadata);

impl UncheckedTypeMetadata {
    pub fn check_metadata_consistency(unchecked_metadata: &UncheckedTypeMetadata) -> Result<(), TypeMetadataError> {
        unchecked_metadata.detect_path_collisions()?;
        unchecked_metadata.detect_duplicate_languages()?;
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

    fn detect_duplicate_languages(&self) -> Result<(), TypeMetadataError> {
        let duplicates = self
            .display
            .iter()
            .duplicates_by(|display| display.lang.as_str())
            .map(|display| display.lang.clone())
            .collect_vec();

        if !duplicates.is_empty() {
            return Err(TypeMetadataError::DuplicateDisplayLanguages(duplicates));
        }

        for claim in &self.claims {
            let duplicates = claim.find_duplicate_languages();

            if !duplicates.is_empty() {
                return Err(TypeMetadataError::DuplicateClaimDisplayLanguages(
                    claim.path.clone(),
                    duplicates,
                ));
            }
        }

        Ok(())
    }

    fn validate_svg_ids(&self) -> Result<(), TypeMetadataError> {
        // `svg_id` MUST be unique within the type metadata.
        let duplicate_svg_ids = self
            .claims
            .iter()
            .filter_map(|claim| claim.svg_id.as_deref())
            .duplicates()
            .map(ToString::to_string)
            .collect_vec();

        if !duplicate_svg_ids.is_empty() {
            return Err(TypeMetadataError::DuplicateSvgIds(duplicate_svg_ids));
        }

        // If the svg_id is not present in the claim metadata, the consuming application SHOULD reject the SVG template.
        let missing_svg_ids = find_missing_svg_ids(&self.display, &self.claims);
        if !missing_svg_ids.is_empty() {
            return Err(TypeMetadataError::MissingSvgIds(missing_svg_ids));
        }

        Ok(())
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

    pub(crate) fn validate(&self, attestation_json: &serde_json::Value) -> Result<(), ValidationError<'static>> {
        self.validator
            .validate(attestation_json)
            .map_err(ValidationError::to_owned)
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
#[serde(untagged)]
pub enum UriMetadata {
    Embedded {
        uri: DataUri,
    },
    Remote {
        #[serde(with = "http_serde::uri")]
        uri: Uri,

        /// Note that although this is optional in the specification, we consider validation using a digest mandatory
        /// if the logo is to be fetched from an external URI, in order to check that this matches the image as
        /// intended by the issuer.
        #[serde(rename = "uri#integrity")]
        uri_integrity: SpecOptional<Integrity>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoMetadata {
    #[serde(flatten)]
    pub uri_metadata: UriMetadata,

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
    pub svg_id: Option<SvgId>,
}

impl ClaimMetadata {
    pub(crate) fn path_to_string(path: &[ClaimPath]) -> String {
        path.iter().fold(String::new(), |mut output, path| {
            let _ = write!(output, "[{path}]");
            output
        })
    }

    fn find_duplicate_languages(&self) -> Vec<String> {
        self.display
            .iter()
            .duplicates_by(|display| &display.lang)
            .map(|display| display.lang.clone())
            .collect()
    }
}

impl Display for ClaimMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::path_to_string(self.path.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// An indication whether the claim is selectively disclosable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[nutype(
    derive(Debug, Clone, AsRef, PartialEq, Eq, Into, Serialize, Deserialize),
    validate(regex = SVG_ID_REGEX),
)]
pub struct SvgId(String);

impl Deref for SvgId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[cfg(any(test, feature = "example_constructors"))]
mod example_constructors {
    use std::collections::HashMap;

    use serde_json::json;

    use crypto::utils::random_string;

    use crate::examples::ADDRESS_METADATA_BYTES;
    use crate::examples::CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES;
    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::EXAMPLE_V2_METADATA_BYTES;
    use crate::examples::EXAMPLE_V3_METADATA_BYTES;
    use crate::examples::PID_METADATA_BYTES;
    use crate::examples::SD_JWT_VC_SPEC_METADATA_BYTES;
    use crate::examples::SIMPLE_EMBEDDED_BYTES;
    use crate::examples::SIMPLE_REMOTE_BYTES;

    use super::ClaimDisplayMetadata;
    use super::ClaimMetadata;
    use super::ClaimPath;
    use super::ClaimSelectiveDisclosureMetadata;
    use super::DisplayMetadata;
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
            let name = random_string(8);

            Self {
                vct: random_string(16),
                name: Some(name.clone()),
                description: None,
                extends: None,
                display: vec![DisplayMetadata {
                    lang: "en".to_string(),
                    name,
                    description: None,
                    summary: None,
                    rendering: None,
                }],
                claims: vec![],
                schema: SchemaOption::Embedded {
                    schema: Box::new(JsonSchema::try_new(json!({"properties": {}})).unwrap()),
                },
            }
        }

        pub fn empty_example_with_attestation_type(attestation_type: &str) -> Self {
            Self {
                vct: String::from(attestation_type),
                ..UncheckedTypeMetadata::empty_example()
            }
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
            Self {
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
            }
        }

        pub fn example() -> Self {
            serde_json::from_slice(EXAMPLE_METADATA_BYTES).unwrap()
        }

        pub fn example_v2() -> Self {
            serde_json::from_slice(EXAMPLE_V2_METADATA_BYTES).unwrap()
        }

        pub fn example_v3() -> Self {
            serde_json::from_slice(EXAMPLE_V3_METADATA_BYTES).unwrap()
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

        pub fn simple_embedded_example() -> Self {
            serde_json::from_slice(SIMPLE_EMBEDDED_BYTES).unwrap()
        }

        pub fn simple_remote_example() -> Self {
            serde_json::from_slice(SIMPLE_REMOTE_BYTES).unwrap()
        }
    }

    impl TypeMetadata {
        pub fn empty_example() -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata::empty_example()).unwrap()
        }

        pub fn empty_example_with_attestation_type(attestation_type: &str) -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata::empty_example_with_attestation_type(
                attestation_type,
            ))
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
            TypeMetadata::try_new(UncheckedTypeMetadata::example_with_claim_names(attestation_type, names)).unwrap()
        }

        pub fn example() -> Self {
            Self::try_new(UncheckedTypeMetadata::example()).unwrap()
        }

        pub fn example_v2() -> Self {
            Self::try_new(UncheckedTypeMetadata::example_v2()).unwrap()
        }

        pub fn example_v3() -> Self {
            Self::try_new(UncheckedTypeMetadata::example_v3()).unwrap()
        }

        pub fn pid_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::pid_example()).unwrap()
        }

        pub fn address_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::address_example()).unwrap()
        }

        pub fn simple_embedded_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::simple_embedded_example()).unwrap()
        }

        pub fn simple_remote_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::simple_remote_example()).unwrap()
        }
    }

    impl JsonSchema {
        pub fn example_with_claim_names(
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

    use http_utils::data_uri::DataUri;
    use utils::vec_at_least::VecNonEmpty;

    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::RED_DOT_BYTES;

    use super::*;

    #[test]
    fn test_deserialize() {
        let metadata = TypeMetadata::example();
        assert_eq!(
            "https://sd_jwt_vc_metadata.example.com/example_credential",
            metadata.as_ref().vct
        );
    }

    #[test]
    fn test_deserialize_with_simple_rendering_and_embedded_logo() {
        assert_eq!(
            Some(RenderingMetadata::Simple {
                logo: Some(LogoMetadata {
                    uri_metadata: UriMetadata::Embedded {
                        uri: DataUri {
                            mime_type: "image/png".to_string(),
                            data: RED_DOT_BYTES.to_vec()
                        }
                    },
                    alt_text: "An example PNG logo".to_string().into(),
                }),
                background_color: Some("#FF8000".to_string()),
                text_color: Some("#0080FF".to_string()),
            }),
            TypeMetadata::simple_embedded_example().as_ref().display[0].rendering
        );
    }

    #[test]
    fn test_deserialize_with_simple_rendering_and_remote_logo() {
        assert_eq!(
            Some(RenderingMetadata::Simple {
                logo: Some(LogoMetadata {
                    uri_metadata: UriMetadata::Remote {
                        uri: Uri::from_static("https://simple.example.com/red-dot.png"),
                        uri_integrity: Integrity::from(RED_DOT_BYTES).into(),
                    },
                    alt_text: "An example PNG logo".to_string().into(),
                }),
                background_color: Some("#FF8000".to_string()),
                text_color: Some("#0080FF".to_string()),
            }),
            TypeMetadata::simple_remote_example().as_ref().display[0].rendering
        );
    }

    #[rstest]
    #[case("foo_bar", true)]
    #[case("foo_bar123", true)]
    #[case("a1", true)]
    #[case("x", true)]
    #[case("_", true)]
    #[case("0", false)]
    #[case("1identifier", false)]
    #[case(" identifier", false)]
    #[case("identifier ", false)]
    #[case("foo bar", false)]
    #[case("日本語", false)]
    #[case("🇳🇱", false)]
    fn test_deserialize_svg_id_error(#[case] svg_id: &str, #[case] should_succeed: bool) {
        let json = String::from_utf8(EXAMPLE_METADATA_BYTES.to_vec())
            .unwrap()
            .replace("\"identifier\"", &format!("\"{svg_id}\""));

        let result = serde_json::from_str::<TypeMetadata>(&json);

        if should_succeed {
            let _ = result.expect("SD-JWT type metadata with correct svg_id should deserialize");
        } else {
            let error = result.expect_err("SD-JWT type metadata with incorrect svg_id should fail to deserialize");

            assert!(error.to_string().contains("SvgId"));
        }
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

        let json_schema = match &metadata.as_ref().schema {
            SchemaOption::Embedded { schema } => schema.as_ref(),
            _ => unreachable!(),
        };

        json_schema.validate(&claims).expect("JSON schema should validate");
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

        let json_schema = match &metadata.as_ref().schema {
            SchemaOption::Embedded { schema } => schema.as_ref(),
            _ => unreachable!(),
        };

        let _error = json_schema
            .validate(&claims)
            .expect_err("JSON schema should fail validation");
    }

    #[rstest]
    #[case("2004-12-25")]
    #[case("2024-02-29")]
    fn test_schema_validation_date_format_happy(#[case] date_str: &str) {
        let metadata = TypeMetadata::example_v3();

        let claims = json!({
            "vct": "https://credentials.example.com/identity_credential",
            "iss": "https://example.com/issuer",
            "iat": 1683000000,
            "attestation_qualification": "EAA",
            "birth_date": date_str,
        });

        let json_schema = match &metadata.as_ref().schema {
            SchemaOption::Embedded { schema } => schema.as_ref(),
            _ => unreachable!(),
        };

        json_schema.validate(&claims).expect("JSON schema should validate");
    }

    #[rstest]
    #[case("not_a_date")]
    #[case("2025-02-29")]
    #[case("01-01-2000")]
    fn test_schema_validation_date_format_error(#[case] date_str: &str) {
        let metadata = TypeMetadata::example_v3();

        let claims = json!({
            "vct": "https://credentials.example.com/identity_credential",
            "iss": "https://example.com/issuer",
            "iat": 1683000000,
            "attestation_qualification": "EAA",
            "birth_date": date_str,
        });

        let json_schema = match &metadata.as_ref().schema {
            SchemaOption::Embedded { schema } => schema.as_ref(),
            _ => unreachable!(),
        };

        let error = json_schema
            .validate(&claims)
            .expect_err("JSON schema should fail validation");

        assert_matches!(
            error,
            ValidationError {
                instance,
                kind: ValidationErrorKind::Format { format },
                instance_path,
                ..
            } if instance.to_string() == format!("\"{}\"", date_str)
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

    fn duplicate_display_language_metadata_json() -> serde_json::Value {
        json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "display": [
                { "lang": "en", "name": "Name" },
                { "lang": "en", "name": "Other name" }
            ],
            "claims": [],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        })
    }

    #[test]
    fn test_error_duplicate_display_languages() {
        let error = serde_json::from_value::<UncheckedTypeMetadata>(duplicate_display_language_metadata_json())
            .unwrap()
            .detect_duplicate_languages()
            .expect_err("duplicate display metadata languages should result in an error");

        assert_matches!(error, TypeMetadataError::DuplicateDisplayLanguages(duplicates) if duplicates == vec!["en"]);
    }

    #[test]
    fn test_deserialize_type_metadata_error_duplicate_display_languages() {
        let error = serde_json::from_value::<TypeMetadata>(duplicate_display_language_metadata_json())
            .expect_err("deserializing duplicate display metadata languages should result in an error");

        assert!(error
            .to_string()
            .contains("detected duplicate display metadata language(s)"));
    }

    fn duplicate_claim_display_language_metadata_json() -> serde_json::Value {
        json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "claims": [
                {
                    "path": ["address.street"],
                    "display": [
                        { "lang": "en", "label": "Street" },
                        { "lang": "en", "label": "Street name" }
                    ],
                },
            ],
            "schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {}
            }
        })
    }

    #[test]
    fn test_error_duplicate_claim_display_languages() {
        let error = serde_json::from_value::<UncheckedTypeMetadata>(duplicate_claim_display_language_metadata_json())
            .unwrap()
            .detect_duplicate_languages()
            .expect_err("duplicate claim display metadata languages should result in an error");

        let expected_path = VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("address.street".to_string())]).unwrap();
        assert_matches!(
            error,
            TypeMetadataError::DuplicateClaimDisplayLanguages(path, duplicates)
                if path == expected_path && duplicates == vec!["en"]
        );
    }

    #[test]
    fn test_deserialize_type_metadata_error_duplicate_claim_display_languages() {
        let error = serde_json::from_value::<TypeMetadata>(duplicate_claim_display_language_metadata_json())
            .expect_err("deserializing duplicate claim display metadata languages should result in an error");

        assert!(error
            .to_string()
            .contains("detected duplicate claim display metadata language(s)"));
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
    fn test_validate_svg_ids_duplicates(
        #[case] claims: serde_json::Value,
        #[case] expected: Result<(), TypeMetadataError>,
    ) {
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
    fn test_validate_svg_ids_missing(#[case] summary: &str, #[case] expected: Result<(), TypeMetadataError>) {
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
