use std::fmt::Display;
use std::fmt::Formatter;

use base64::prelude::*;
use derive_more::Into;
use http::Uri;
use jsonschema::ValidationError;
use nutype::nutype;
use serde::de;
use serde::ser;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_with::skip_serializing_none;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug)]
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

/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct TypeMetadata {
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
    pub display: Vec<DisplayMetadata>,

    /// An array of objects containing claim information for the type.
    #[serde(default)]
    pub claims: Vec<ClaimMetadata>,

    /// A JSON Schema document describing the structure of the Verifiable Credential
    #[serde(flatten)]
    pub schema: SchemaOption,
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

impl TypeMetadata {
    pub fn try_as_base64(&self) -> Result<String, TypeMetadataError> {
        let bytes: Vec<u8> = serde_json::to_vec(&self)?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn try_from_base64(encoded: &str) -> Result<Self, TypeMetadataError> {
        let bytes: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(encoded.as_bytes())?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn validate(&self, json_claims: &serde_json::Value) -> Result<(), TypeMetadataError> {
        if let SchemaOption::Embedded { schema } = &self.schema {
            jsonschema::draft202012::validate(schema.as_ref(), json_claims).map_err(ValidationError::to_owned)?;
            Ok(())
        } else {
            Err(TypeMetadataError::UnsupportedSchemaOption(self.schema.clone()))
        }
    }
}

impl TryFrom<Vec<u8>> for TypeMetadata {
    type Error = serde_json::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value)
    }
}

impl TryFrom<&TypeMetadata> for Vec<u8> {
    type Error = serde_json::Error;

    fn try_from(value: &TypeMetadata) -> Result<Self, Self::Error> {
        serde_json::to_vec(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataExtends {
    /// A URI of another type that this type extends.
    #[serde(with = "http_serde::uri")]
    pub extends: Uri,

    /// Validating the integrity of the extends field.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: SpecOptionalImplRequired<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOption {
    Embedded {
        /// An embedded JSON Schema document describing the structure of the Verifiable Credential.
        schema: JsonSchema,
    },
    Remote {
        /// A URL pointing to a JSON Schema document describing the structure of the Verifiable Credential.
        #[serde(with = "http_serde::uri")]
        schema_uri: Uri,
        /// Validating the integrity of the schema_uri field.
        #[serde(rename = "schema_uri#integrity")]
        schema_uri_integrity: SpecOptionalImplRequired<String>,
    },
}

#[nutype(
    validate(with = validate_json_schema, error = TypeMetadataError),
    derive(Debug, Clone, AsRef, Serialize, Deserialize)
)]
pub struct JsonSchema(serde_json::Value);

fn validate_json_schema(schema: &serde_json::Value) -> Result<(), TypeMetadataError> {
    jsonschema::draft202012::meta::validate(schema).map_err(ValidationError::to_owned)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct DisplayMetadata {
    pub lang: String,
    pub name: String,
    pub description: Option<String>,
    pub rendering: Option<RenderingMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[skip_serializing_none]
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

    #[serde(rename = "uri#integrity")]
    pub uri_integrity: SpecOptionalImplRequired<String>,

    pub alt_text: SpecOptionalImplRequired<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ClaimMetadata {
    pub path: VecNonEmpty<ClaimPath>,
    #[serde(default)]
    pub display: Vec<ClaimDisplayMetadata>,
    #[serde(default)]
    pub sd: ClaimSelectiveDisclosureMetadata,
    pub svg_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ClaimPath {
    SelectByKey(String),
    SelectAll,
    SelectByIndex(usize),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimSelectiveDisclosureMetadata {
    Always,
    #[default]
    Allowed,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

#[cfg(any(test, feature = "example_constructors"))]
pub mod mock {
    use serde_json::json;

    use wallet_common::utils::random_string;

    use crate::metadata::ClaimDisplayMetadata;
    use crate::metadata::ClaimMetadata;
    use crate::metadata::ClaimPath;
    use crate::metadata::ClaimSelectiveDisclosureMetadata;
    use crate::metadata::JsonSchema;
    use crate::metadata::SchemaOption;
    use crate::metadata::TypeMetadata;

    impl TypeMetadata {
        pub fn empty_example() -> Self {
            Self {
                vct: random_string(16),
                name: Some(random_string(8)),
                description: None,
                extends: None,
                display: vec![],
                claims: vec![],
                schema: SchemaOption::Embedded {
                    schema: JsonSchema::try_new(json!({})).unwrap(),
                },
            }
        }

        pub fn bsn_only_example() -> Self {
            Self {
                vct: random_string(16),
                name: Some(random_string(8)),
                description: None,
                extends: None,
                display: vec![],
                claims: vec![ClaimMetadata {
                    path: vec![ClaimPath::SelectByKey(String::from("bsn"))].try_into().unwrap(),
                    display: vec![ClaimDisplayMetadata {
                        lang: String::from("en"),
                        label: String::from("BSN"),
                        description: None,
                    }],
                    sd: ClaimSelectiveDisclosureMetadata::Always,
                    svg_id: None,
                }],
                schema: SchemaOption::Embedded {
                    schema: JsonSchema::try_new(json!({})).unwrap(),
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use serde_json::json;

    use crate::metadata::ClaimPath;
    use crate::metadata::MetadataExtendsOption;
    use crate::metadata::ResourceIntegrity;
    use crate::metadata::SchemaOption;
    use crate::metadata::TypeMetadata;

    async fn read_and_parse_metadata(filename: &str) -> TypeMetadata {
        let base_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

        let metadata_file = tokio::fs::read(base_path.join("examples").join(filename))
            .await
            .unwrap();

        serde_json::from_slice(metadata_file.as_slice()).unwrap()
    }

    #[tokio::test]
    async fn test_deserialize() {
        let metadata = read_and_parse_metadata("example-metadata.json").await;

        assert_eq!(
            "https://sd_jwt_vc_metadata.example.com/example_credential",
            metadata.vct
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

        assert_matches!(metadata.extends, Some(MetadataExtendsOption::Identifier { .. }));
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

        assert_matches!(metadata.extends, Some(MetadataExtendsOption::Uri { .. }));
        assert_matches!(metadata.schema, SchemaOption::Remote { .. });
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

    #[tokio::test]
    async fn test_schema_validation_success() {
        let metadata = read_and_parse_metadata("example-metadata.json").await;

        let claims = json!({
          "vct":"https://credentials.example.com/identity_credential",
          "iss":"https://example.com/issuer",
          "nbf":1683000000,
          "exp":1883000000,
          "address":{
            "country":"DE"
          },
          "cnf":{
            "jwk":{
              "kty":"EC",
              "crv":"P-256",
              "x":"TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
              "y":"ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
            }
          }
        });

        assert_eq!(
            vec![
                ClaimPath::SelectByKey(String::from("nationalities")),
                ClaimPath::SelectAll
            ],
            metadata.claims[5].path.clone().into_inner()
        );

        metadata.validate(&claims).unwrap();
    }

    #[tokio::test]
    async fn test_schema_validation_failure() {
        let metadata = read_and_parse_metadata("example-metadata.json").await;

        let claims = json!({
          "address":{
            "country":123
          }
        });

        assert!(metadata.validate(&claims).is_err());
    }

    #[tokio::test]
    async fn test_protect_verify() {
        let metadata = read_and_parse_metadata("example-metadata.json").await;
        let bytes = serde_json::to_vec(&metadata).unwrap();
        let integrity = ResourceIntegrity::from_bytes(&bytes);
        integrity.verify(&bytes).unwrap();
    }
}
