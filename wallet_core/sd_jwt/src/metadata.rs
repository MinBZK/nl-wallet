use http::Uri;
use serde::Deserialize;
use serde::Serialize;

/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMetadata {
    /// A String or URI that uniquely identifies the type.
    pub vct: String,

    /// A human-readable name for the type, intended for developers reading the JSON document.
    pub name: Option<String>,

    /// A human-readable description for the type, intended for developers reading the JSON document.
    pub description: Option<String>,

    /// Another type that this type extends.
    #[serde(flatten)]
    pub extends: Option<MetadataExtends>,

    /// Validating the integrity of the extends field.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: Option<String>,

    /// An array of objects containing display information for the type.
    pub display: Vec<DisplayMetadata>,

    /// An array of objects containing claim information for the type.
    pub claims: Vec<ClaimMetadata>,

    /// A JSON Schema document describing the structure of the Verifiable Credential
    #[serde(flatten)]
    pub schema: SchemaOption,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataExtends {
    /// A URI of another type that this type extends.
    #[serde(with = "http_serde::uri")]
    pub extends: Uri,

    /// Validating the integrity of the extends field.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOption {
    Embedded {
        /// An embedded JSON Schema document describing the structure of the Verifiable Credential.
        schema: serde_json::Value,
    },
    Remote {
        /// A URL pointing to a JSON Schema document describing the structure of the Verifiable Credential.
        /// schema_uri MUST NOT be used if schema is present.
        #[serde(with = "http_serde::uri")]
        schema_uri: Uri,
        /// Validating the integrity of the schema_uri field.
        #[serde(rename = "schema_uri#integrity")]
        schema_uri_integrity: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayMetadata {
    pub lang: String,
    pub name: String,
    pub description: Option<String>,
    pub rendering: Option<RenderingMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    pub uri_integrity: String,

    pub alt_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
pub struct ClaimDisplayMetadata {
    pub lang: String,
    pub label: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::metadata::SchemaOption;
    use crate::metadata::TypeMetadata;
    use serde_json::json;
    use std::env;
    use std::path::PathBuf;

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

    #[tokio::test]
    async fn test_schema_validation() {
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

        match &metadata.schema {
            SchemaOption::Embedded { schema } => {
                assert!(jsonschema::draft202012::is_valid(schema, &claims))
            }
            SchemaOption::Remote { .. } => {
                panic!("Remote schema option is not supported")
            }
        }
    }
}
