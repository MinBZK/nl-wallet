use http::Uri;
use jsonschema::ValidationError;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataError {
    #[error("json schema validation failed {0}")]
    JsonSchemaValidation(#[from] ValidationError<'static>),
}

/// Communicates that a type is optional in the specification it is derived from but implemented as mandatory due to
/// various reasons.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecOptionalImplRequired<T>(pub T);

/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataExtends {
    /// A URI of another type that this type extends.
    #[serde(with = "http_serde::uri")]
    pub extends: Uri,

    /// Validating the integrity of the extends field.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: SpecOptionalImplRequired<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[nutype(validate(with = validate_json_schema, error = TypeMetadataError), derive(Debug, Clone, Serialize, Deserialize))]
pub struct JsonSchema(serde_json::Value);

fn validate_json_schema(schema: &serde_json::Value) -> Result<(), TypeMetadataError> {
    jsonschema::draft202012::meta::validate(schema).map_err(ValidationError::to_owned)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct DisplayMetadata {
    pub lang: String,
    pub name: String,
    pub description: Option<String>,
    pub rendering: Option<RenderingMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoMetadata {
    #[serde(with = "http_serde::uri")]
    pub uri: Uri,

    #[serde(rename = "uri#integrity")]
    pub uri_integrity: SpecOptionalImplRequired<String>,

    pub alt_text: SpecOptionalImplRequired<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ClaimMetadata {
    pub path: Vec<serde_json::Value>,
    pub display: Vec<ClaimDisplayMetadata>,
    #[serde(default)]
    pub sd: ClaimSelectiveDisclosureMetadata,
    pub svg_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimSelectiveDisclosureMetadata {
    Always,
    #[default]
    Allowed,
    Never,
}

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use serde_json::json;

    use crate::metadata::MetadataExtendsOption;
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
          "iat":1683000000,
          "exp":1883000000,
          "sub":"6c5c0a49-b589-431d-bae7-219122a9ec2c",
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

        match metadata.schema {
            SchemaOption::Embedded { schema } => {
                assert!(jsonschema::draft202012::is_valid(&schema.into_inner(), &claims))
            }
            SchemaOption::Remote { .. } => {
                panic!("Remote schema option is not supported")
            }
        }
    }

    #[tokio::test]
    async fn test_schema_validation_failure() {
        let metadata = read_and_parse_metadata("example-metadata.json").await;

        let claims = json!({
          "address":{
            "country":123
          }
        });

        match metadata.schema {
            SchemaOption::Embedded { schema } => {
                assert!(jsonschema::draft202012::validate(&schema.into_inner(), &claims).is_err())
            }
            SchemaOption::Remote { .. } => {
                panic!("Remote schema option is not supported")
            }
        }
    }
}
