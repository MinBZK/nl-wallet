use http::Uri;
use serde::Deserialize;
use serde::Serialize;

/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#name-type-metadata-format
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMetadata {
    /// A URI that uniquely identifies the type.
    /// This URI MUST be dereferenceable to a JSON document that describes the type.
    pub vct: String,

    /// A human-readable name for the type, intended for developers reading the JSON document.
    pub name: Option<String>,

    /// A human-readable description for the type, intended for developers reading the JSON document.
    pub description: Option<String>,

    /// A URI of another type that this type extends.
    pub extends: Option<String>,

    /// Validating the integrity of the extends field.
    #[serde(rename = "extends#integrity")]
    pub extends_integrity: Option<String>,

    pub display: Vec<DisplayMetadata>,

    pub claims: Vec<ClaimMetadata>,

    /// An embedded JSON Schema document describing the structure of the Verifiable Credential.
    pub schema: serde_json::Value,
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
    pub path: Vec<String>,
    pub display: Vec<ClaimDisplayMetadata>,
    // #[serde(default = "ClaimSelectiveDisclosureMetadata::default")]
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
    use crate::metadata::TypeMetadata;
    use serde_json::json;
    use std::env;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_deserialize() {
        let base_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

        let metadata_file = tokio::fs::read(base_path.join("examples").join("example-metadata.json"))
            .await
            .unwrap();

        let metadata: TypeMetadata = serde_json::from_slice(metadata_file.as_slice()).unwrap();

        assert_eq!(
            "https://sd_jwt_vc_metadata.example.com/example_credential",
            metadata.vct
        );
    }

    #[tokio::test]
    async fn test_schema_validation() {
        let base_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

        let metadata_file = tokio::fs::read(base_path.join("examples").join("example-metadata.json"))
            .await
            .unwrap();

        let metadata: TypeMetadata = serde_json::from_slice(metadata_file.as_slice()).unwrap();

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

        assert!(jsonschema::draft202012::is_valid(&metadata.schema, &claims))
    }
}
