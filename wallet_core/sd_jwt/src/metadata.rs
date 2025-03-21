use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;

use base64::prelude::*;
use derive_more::Into;
use http::Uri;
use jsonschema::Draft;
use jsonschema::ValidationError;
use jsonschema::Validator;
use nutype::nutype;
use serde::de;
use serde::ser;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::MapSkipError;

use wallet_common::utils::sha256;
use wallet_common::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataError {
    #[error("json schema validation failed {0}")]
    JsonSchemaValidation(#[from] ValidationError<'static>),

    #[error("serialization failed {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("decoding failed {0}")]
    Decoding(#[from] base64::DecodeError),

    #[error("resource integrity check failed, expected: {expected:?}, actual: {actual:?}")]
    ResourceIntegrity {
        expected: ResourceIntegrity,
        actual: ResourceIntegrity,
    },

    #[error("schema option {0:?} is not supported")]
    UnsupportedSchemaOption(SchemaOption),

    #[error("detected claim path collision")]
    ClaimPathCollision,
}

/// Communicates that a type is optional in the specification it is derived from but implemented as mandatory due to
/// various reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecOptionalImplRequired<T>(pub T);

impl<T> From<T> for SpecOptionalImplRequired<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsRef<T> for SpecOptionalImplRequired<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

pub const COSE_METADATA_HEADER_LABEL: &str = "vctm";
pub const COSE_METADATA_INTEGRITY_HEADER_LABEL: &str = "type_metadata_integrity";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeMetadataChain {
    metadata: VecNonEmpty<EncodedTypeMetadata>,
    root_integrity: ResourceIntegrity,
}

impl TypeMetadataChain {
    pub fn create(
        root: TypeMetadata,
        mut extended_metadata: Vec<TypeMetadata>,
    ) -> Result<TypeMetadataChain, TypeMetadataError> {
        let root_bytes: Vec<u8> = (&root).try_into()?;
        let root_integrity = ResourceIntegrity::from_bytes(&root_bytes);
        extended_metadata.push(root);
        let metadata = VecNonEmpty::try_from(
            extended_metadata
                .into_iter()
                .map(EncodedTypeMetadata)
                .collect::<Vec<_>>(),
        )
        .unwrap(); // unwrap is safe here because there is always at least one item (root)
        let result = Self {
            metadata,
            root_integrity,
        };
        Ok(result)
    }

    fn into_destructured(self) -> (VecNonEmpty<TypeMetadata>, ResourceIntegrity) {
        (
            // Unwrapping is safe since we're mapping from a `VecNonEmpty` to a `VecNonEmpty`
            VecNonEmpty::try_from(self.metadata.into_inner().into_iter().map(|m| m.0).collect::<Vec<_>>()).unwrap(),
            self.root_integrity,
        )
    }

    pub fn verify_and_destructure(self) -> Result<(VecNonEmpty<TypeMetadata>, ResourceIntegrity), TypeMetadataError> {
        let bytes: Vec<u8> = (&self.metadata.first().0).try_into()?; // TODO: verify chain in PVW-3824
        self.root_integrity.verify(&bytes)?;
        Ok(self.into_destructured())
    }

    pub fn verify(&self) -> Result<TypeMetadata, TypeMetadataError> {
        let root = self.metadata.first().0.clone();
        let bytes: Vec<u8> = (&root).try_into()?; // TODO: verify chain in PVW-3824
        self.root_integrity.verify(&bytes)?;
        Ok(root)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedTypeMetadata(TypeMetadata);

impl Serialize for EncodedTypeMetadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = self.0.try_as_base64().map_err(ser::Error::custom)?;
        encoded.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EncodedTypeMetadata {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let metadata: TypeMetadata =
            TypeMetadata::try_from_base64(&String::deserialize(deserializer)?).map_err(de::Error::custom)?;
        Ok(Self(metadata))
    }
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
    pub extends: Option<MetadataExtendsOption>,

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
        unchecked_metadata.detect_path_collisions()
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

            // If inserting the flattened key in the set returns false, it means it is already in the set and there
            // is a collision.
            if !paths.insert(flattened_key) {
                return Err(TypeMetadataError::ClaimPathCollision);
            }
        }

        Ok(())
    }
}

impl TypeMetadata {
    pub fn try_as_base64(&self) -> Result<String, TypeMetadataError> {
        let bytes: Vec<u8> = serde_json::to_vec(&self.as_ref())?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn try_from_base64(encoded: &str) -> Result<Self, TypeMetadataError> {
        let bytes: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(encoded.as_bytes())?;
        Self::try_new(serde_json::from_slice::<UncheckedTypeMetadata>(&bytes)?)
    }

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

impl TryFrom<&TypeMetadata> for Vec<u8> {
    type Error = serde_json::Error;

    fn try_from(value: &TypeMetadata) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Into, Serialize, Deserialize)]
pub struct ResourceIntegrity(String);

impl ResourceIntegrity {
    const ALG_PREFIX: &'static str = "sha256";

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let sig = sha256(bytes);
        let integrity = format!("{}-{}", Self::ALG_PREFIX, BASE64_STANDARD.encode(sig));
        ResourceIntegrity(integrity)
    }

    pub fn verify(&self, bytes: &[u8]) -> Result<(), TypeMetadataError> {
        let integrity = Self::from_bytes(bytes);
        if self != &integrity {
            return Err(TypeMetadataError::ResourceIntegrity {
                expected: self.clone(),
                actual: integrity,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataExtendsOption {
    Uri {
        #[serde(flatten)]
        extends: MetadataExtends,
    },
    Identifier {
        extends: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataExtends {
    /// A URI of another type that this type extends.
    #[serde(with = "http_serde::uri")]
    pub extends: Uri,

    /// Validating the integrity of the extends field.
    /// Note that this is optional in the specification, but we consider this mandatory:
    /// * If the metadata this type extends is fetched from an external URI, the integrity digest guarantees that its
    ///   contents match what is expected by the issuer.
    /// * If the metadata is included with issuance, e.g. in an unprotected header, a chain of integrity digests that
    ///   starts from a digest included in a signed section of the attestation acts as a de facto signature, protecting
    ///   against tampering. In SD-JWT the `vct#integrity` claim would contain this first digest.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: SpecOptionalImplRequired<String>,
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
        schema_uri_integrity: SpecOptionalImplRequired<String>,
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
    pub lang: String,
    pub name: String,
    pub description: Option<String>,
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
    pub uri_integrity: SpecOptionalImplRequired<String>,

    /// Note that although this is optional in the specification, it is mandatory within the context of the wallet app
    /// because of accessibility requirements.
    pub alt_text: SpecOptionalImplRequired<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMetadata {
    pub path: VecNonEmpty<ClaimPath>,

    #[serde(default)]
    pub display: Vec<ClaimDisplayMetadata>,

    #[serde(default)]
    pub sd: ClaimSelectiveDisclosureMetadata,

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimSelectiveDisclosureMetadata {
    Always,
    #[default]
    Allowed,
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
pub mod mock {
    use std::collections::HashMap;

    use serde_json::json;

    use wallet_common::utils::random_string;

    use crate::metadata::ClaimDisplayMetadata;
    use crate::metadata::ClaimMetadata;
    use crate::metadata::ClaimPath;
    use crate::metadata::ClaimSelectiveDisclosureMetadata;
    use crate::metadata::JsonSchema;
    use crate::metadata::JsonSchemaProperties;
    use crate::metadata::JsonSchemaProperty;
    use crate::metadata::JsonSchemaPropertyFormat;
    use crate::metadata::JsonSchemaPropertyType;
    use crate::metadata::SchemaOption;
    use crate::metadata::TypeMetadata;
    use crate::metadata::UncheckedTypeMetadata;

    const ADDRESS_METADATA_BYTES: &[u8] = include_bytes!("../examples/address-metadata.json");
    const EXAMPLE_METADATA_BYTES: &[u8] = include_bytes!("../examples/example-metadata.json");
    const PID_METADATA_BYTES: &[u8] = include_bytes!("../examples/pid-metadata.json");

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

        pub fn address_example() -> Self {
            serde_json::from_slice(ADDRESS_METADATA_BYTES).unwrap()
        }

        pub fn example() -> Self {
            serde_json::from_slice(EXAMPLE_METADATA_BYTES).unwrap()
        }

        pub fn pid_example() -> Self {
            serde_json::from_slice(PID_METADATA_BYTES).unwrap()
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
    use rstest::*;
    use serde_json::json;

    use crate::metadata::ClaimPath;
    use crate::metadata::MetadataExtendsOption;
    use crate::metadata::ResourceIntegrity;
    use crate::metadata::SchemaOption;
    use crate::metadata::TypeMetadata;
    use crate::metadata::TypeMetadataError;
    use crate::metadata::UncheckedTypeMetadata;

    #[test]
    fn test_deserialize() {
        let metadata = TypeMetadata::example();
        assert_eq!(
            "https://sd_jwt_vc_metadata.example.com/example_credential",
            metadata.as_ref().vct
        );
    }

    #[test]
    fn test_extends_with_identifier() {
        let metadata = serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "extends": "random_string",
            "display": [],
            "schema_uri": "https://sd_jwt_vc_metadata.example.com/",
            "schema_uri#integrity": "abc123",
        }))
        .unwrap();

        assert_matches!(
            metadata.as_ref().extends,
            Some(MetadataExtendsOption::Identifier { .. })
        );
    }

    #[test]
    fn test_with_uri() {
        let metadata = serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "extends": "https://sd_jwt_vc_metadata.example.com/other_schema",
            "extends#integrity": "abc123",
            "display": [],
            "schema_uri": "https://sd_jwt_vc_metadata.example.com/",
            "schema_uri#integrity": "abc123",
        }))
        .unwrap();

        assert_matches!(metadata.as_ref().extends, Some(MetadataExtendsOption::Uri { .. }));
        assert_matches!(metadata.as_ref().schema, SchemaOption::Remote { .. });
    }

    #[test]
    fn test_embedded_schema_validation() {
        assert!(serde_json::from_value::<TypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "extends": "https://sd_jwt_vc_metadata.example.com/other_schema",
            "extends#integrity": "abc123",
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
          "vct":"https://credentials.example.com/identity_credential",
          "iss":"https://example.com/issuer",
          "nbf":1683000000,
          "iat":1683000000,
          "exp":1883000000,
          "place_of_birth":{
            "locality":"DE"
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
          "vct":"https://credentials.example.com/identity_credential",
          "iss":"https://example.com/issuer",
          "iat":1683000000,
          "address":{
            "country":123
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
            "vct":"https://credentials.example.com/identity_credential",
            "iss":"https://example.com/issuer",
            "iat":1683000000,
            "birth_date":date_str,
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
            "vct":"https://credentials.example.com/identity_credential",
            "iss":"https://example.com/issuer",
            "iat":1683000000,
            "birth_date":date_str,
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

    #[test]
    fn test_protect_verify() {
        let metadata = TypeMetadata::example();
        let bytes = serde_json::to_vec(&metadata).unwrap();
        let integrity = ResourceIntegrity::from_bytes(&bytes);
        integrity.verify(&bytes).unwrap();
    }

    #[test]
    fn test_claim_path_collision() {
        let result = serde_json::from_value::<UncheckedTypeMetadata>(json!({
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
        .unwrap()
        .detect_path_collisions();

        assert_matches!(result, Err(TypeMetadataError::ClaimPathCollision));
    }

    #[rstest]
    #[case(vec![vec!["a.b"], vec!["a", "b"]])]
    #[case(vec![vec!["x.y.z"], vec!["x", "y.z"]])]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y.z"]])]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y", "z"]])]
    #[case(vec![vec!["x", "y.z"], vec!["x.y", "z"]])]
    fn test_claim_path_collisions(#[case] claims: Vec<Vec<&str>>) {
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

        assert_matches!(result, Err(TypeMetadataError::ClaimPathCollision));
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
}
