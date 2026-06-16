use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::LazyLock;

use attestation_types::claim_path::ClaimPath;
use attestation_types::data_uri::DataUri;
use attestation_types::image::Image;
use itertools::Itertools;
use nutype::nutype;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use ssri::Integrity;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;

// The requirements for the svg_id according to the specification are:
// "It MUST consist of only alphanumeric characters and underscores and MUST NOT start with a digit."
static SVG_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][0-9A-Za-z_]*$").unwrap());
static TEMPLATE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{([A-Za-z_][0-9A-Za-z_]*)}}").unwrap());

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataError {
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

    #[error("internal attributes found in claim: {}", .0.join(", "))]
    InternalAttributeInClaim(Vec<String>),
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
/// * Every attribute in the attestation received from the issuer should have corresponding claim metadata, so that the
///   attribute can be rendered for display to the user.
/// * Claims that cover a group of attributes are not (yet) supported and will not be accepted, as rendering groups of
///   attributes covered by the same display data is not supported by the UI.
/// * Claims cannot contain paths for internal attributes.
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

const INTERNAL_ATTRIBUTES: &[&str] = &[
    "vct",
    "vct#integrity",
    "cnf",
    "iss",
    "nbf",
    "exp",
    "iat",
    "sub",
    "status",
    "attestation_qualification",
];

impl UncheckedTypeMetadata {
    pub fn check_metadata_consistency(unchecked_metadata: &UncheckedTypeMetadata) -> Result<(), TypeMetadataError> {
        unchecked_metadata.check_internal_attributes()?;
        unchecked_metadata.detect_path_collisions()?;
        unchecked_metadata.detect_duplicate_languages()?;
        unchecked_metadata.validate_svg_ids()?;

        Ok(())
    }

    fn check_internal_attributes(&self) -> Result<(), TypeMetadataError> {
        let internal_claims = self
            .claims
            .iter()
            .filter_map(|claim| match claim.path.first() {
                ClaimPath::SelectByKey(key) if INTERNAL_ATTRIBUTES.contains(&key.as_str()) => Some(key.clone()),
                _ => None,
            })
            .collect::<Vec<String>>();
        if !internal_claims.is_empty() {
            return Err(TypeMetadataError::InternalAttributeInClaim(internal_claims));
        }
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
            .duplicates_by(|display| display.locale.as_str())
            .map(|display| display.locale.clone())
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

#[skip_serializing_none]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayMetadata {
    ///  A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable name for the type, intended for end users.
    pub name: String,

    /// A human-readable description for the type, intended for end users.
    pub description: Option<String>,

    /// A templated summary for the type, intended to be rendered to the end user.
    pub summary: Option<String>,

    /// An object containing rendering information for the type
    #[debug(skip)]
    pub rendering: Option<RenderingMetadata>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderingMetadata {
    Simple {
        /// An object containing information about the logo to be displayed for the type.
        logo: Option<LogoMetadata>,

        /// An object containing information about the background image to be displayed for the type.
        background_image: Option<BackgroundImageMetadata>,

        /// An RGB color value for the background of the credential.
        background_color: Option<String>,

        /// An RGB color value for the text of the credential.
        text_color: Option<String>,
    },
    SvgTemplates,
}

#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoMetadata {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,

    /// Note that although this is optional in the specification, it is mandatory within the context of the wallet app
    /// because of accessibility requirements.
    pub alt_text: SpecOptional<String>,
}

#[serde_as]
#[derive(derive_more::Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundImageMetadata {
    /// Explicitly reject non-embedded images and unsupported mime types
    #[debug(skip)]
    #[serde(rename = "uri")]
    #[serde_as(as = "TryFromInto<DataUri>")]
    pub image: Image,
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

    /// The mandatory property is a boolean indicating that, if set to true, the claim MUST be included in the
    /// credential by the Issuer. If the value is false or omitted, the claim is considered optional for the Issuer to
    /// include.
    #[serde(default)]
    pub mandatory: bool,

    /// A string defining the ID of the claim for reference in the SVG template.
    pub svg_id: Option<SvgId>,
}

impl ClaimMetadata {
    pub(crate) fn path_to_string(path: &[ClaimPath]) -> String {
        format!("[{}]", path.iter().join(", "))
    }

    fn find_duplicate_languages(&self) -> Vec<String> {
        self.display
            .iter()
            .duplicates_by(|display| &display.locale)
            .map(|display| display.locale.clone())
            .collect()
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
    /// A language tag as defined in Section 2 of [RFC5646](https://www.rfc-editor.org/info/rfc5646).
    pub locale: String,

    /// A human-readable label for the claim, intended for end users.
    pub label: String,

    /// A human-readable description for the claim, intended for end users.
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

    use attestation_types::claim_path::ClaimPath;
    use crypto::utils::random_string;
    use utils::vec_nonempty;

    use super::ClaimDisplayMetadata;
    use super::ClaimMetadata;
    use super::ClaimSelectiveDisclosureMetadata;
    use super::DisplayMetadata;
    use super::TypeMetadata;
    use super::UncheckedTypeMetadata;
    use crate::examples::CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES;
    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::PID_METADATA_BYTES;

    impl UncheckedTypeMetadata {
        pub fn empty_example() -> Self {
            let name = random_string(8);

            Self {
                vct: random_string(16),
                name: Some(name.clone()),
                description: None,
                extends: None,
                display: vec![DisplayMetadata {
                    locale: "en".to_string(),
                    name,
                    description: None,
                    summary: None,
                    rendering: None,
                }],
                claims: vec![],
            }
        }

        pub fn empty_example_with_attestation_type(attestation_type: &str) -> Self {
            Self {
                vct: String::from(attestation_type),
                ..UncheckedTypeMetadata::empty_example()
            }
        }

        pub fn example_with_claim_name(attestation_type: &str, name: &str) -> Self {
            Self::example_with_claim_names(attestation_type, &[name])
        }

        pub fn example_with_claim_names(attestation_type: &str, names: &[&str]) -> Self {
            Self {
                vct: String::from(attestation_type),
                claims: names
                    .iter()
                    .map(|name| ClaimMetadata {
                        path: vec_nonempty![ClaimPath::SelectByKey(String::from(*name))],
                        display: vec![ClaimDisplayMetadata {
                            locale: String::from("en"),
                            label: name.to_uppercase(),
                            description: None,
                        }],
                        sd: ClaimSelectiveDisclosureMetadata::Always,
                        mandatory: false,
                        svg_id: None,
                    })
                    .collect(),
                ..UncheckedTypeMetadata::empty_example()
            }
        }

        pub(crate) fn example() -> Self {
            serde_json::from_slice(EXAMPLE_METADATA_BYTES).unwrap()
        }

        pub fn pid_example() -> Self {
            serde_json::from_slice(PID_METADATA_BYTES).unwrap()
        }

        pub fn credential_payload_sd_jwt_metadata() -> Self {
            serde_json::from_slice(CREDENTIAL_PAYLOAD_SD_JWT_SPEC_METADATA_BYTES).unwrap()
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

        pub fn example_with_claim_name(attestation_type: &str, name: &str) -> Self {
            Self::example_with_claim_names(attestation_type, &[name])
        }

        pub fn example_with_claim_names(attestation_type: &str, names: &[&str]) -> Self {
            TypeMetadata::try_new(UncheckedTypeMetadata::example_with_claim_names(attestation_type, names)).unwrap()
        }

        pub fn pid_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::pid_example()).unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::str::FromStr;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::data_uri::DataUri;
    use attestation_types::image::ImageError;
    use rstest::rstest;
    use serde_json::json;
    use utils::vec_nonempty;

    use super::*;
    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::test::EXAMPLE_V2_METADATA_BYTES;
    use crate::examples::test::EXAMPLE_V3_METADATA_BYTES;
    use crate::examples::test::RED_DOT_BYTES;
    use crate::examples::test::SIMPLE_EMBEDDED_METADATA_BYTES;
    use crate::examples::test::SIMPLE_REMOTE_BACKGROUND_METADATA_BYTES;
    use crate::examples::test::SIMPLE_REMOTE_METADATA_BYTES;
    use crate::examples::test::VCT_EXAMPLE_CREDENTIAL;

    impl UncheckedTypeMetadata {
        pub(crate) fn example_v2() -> Self {
            serde_json::from_slice(EXAMPLE_V2_METADATA_BYTES).unwrap()
        }

        pub(crate) fn example_v3() -> Self {
            serde_json::from_slice(EXAMPLE_V3_METADATA_BYTES).unwrap()
        }

        pub(crate) fn simple_embedded_example() -> Self {
            serde_json::from_slice(SIMPLE_EMBEDDED_METADATA_BYTES).unwrap()
        }

        pub(crate) fn simple_remote_example() -> serde_json::Error {
            // Explicitly unsupported at the moment, hence the error return
            serde_json::from_slice::<Self>(SIMPLE_REMOTE_METADATA_BYTES).unwrap_err()
        }

        pub(crate) fn simple_remote_background_example() -> serde_json::Error {
            // Explicitly unsupported at the moment, hence the error return
            serde_json::from_slice::<Self>(SIMPLE_REMOTE_BACKGROUND_METADATA_BYTES).unwrap_err()
        }
    }

    impl TypeMetadata {
        pub(crate) fn example() -> Self {
            Self::try_new(UncheckedTypeMetadata::example()).unwrap()
        }

        pub(crate) fn example_v2() -> Self {
            Self::try_new(UncheckedTypeMetadata::example_v2()).unwrap()
        }

        pub(crate) fn example_v3() -> Self {
            Self::try_new(UncheckedTypeMetadata::example_v3()).unwrap()
        }

        pub(crate) fn simple_embedded_example() -> Self {
            Self::try_new(UncheckedTypeMetadata::simple_embedded_example()).unwrap()
        }
    }

    #[test]
    fn test_path_to_string() {
        let result = ClaimMetadata::path_to_string(&[
            ClaimPath::SelectByKey("key".to_string()),
            ClaimPath::SelectByIndex(3),
            ClaimPath::SelectAll,
        ]);
        assert_eq!(result, String::from("[key, 3, null]"));
    }

    #[test]
    fn test_deserialize() {
        let metadata = TypeMetadata::example();
        assert_eq!(VCT_EXAMPLE_CREDENTIAL, metadata.as_ref().vct);
    }

    #[test]
    fn test_deserialize_with_simple_rendering_and_embedded_logo() {
        assert_eq!(
            Some(RenderingMetadata::Simple {
                logo: Some(LogoMetadata {
                    image: Image::Png(RED_DOT_BYTES.to_vec()),
                    alt_text: "An example PNG logo".to_string().into(),
                }),
                background_image: Some(BackgroundImageMetadata {
                    image: Image::Png(RED_DOT_BYTES.to_vec())
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
            "data-url error: not a valid data url at line 14 column 59",
            UncheckedTypeMetadata::simple_remote_example().to_string(),
        );
    }

    #[test]
    fn test_deserialize_with_simple_rendering_and_remote_background() {
        assert_eq!(
            "data-url error: not a valid data url at line 18 column 59",
            UncheckedTypeMetadata::simple_remote_background_example().to_string(),
        );
    }

    #[rstest]
    #[case("data:image/png;base64,q80=")]
    #[case("data:image/jpeg;base64,yv4=")]
    #[case("data:image/svg+xml;utf8,<svg></svg>")]
    fn test_try_from_into_image(#[case] uri: &str) {
        let uri = DataUri::from_str(uri).unwrap();
        let image: Image = Image::try_from(uri.clone()).unwrap();
        assert_eq!(uri, image.into());
    }

    #[test]
    fn test_image_uri_unsupported_mime_type() {
        let uri = DataUri::from_str("data:image/webp;base64,q7o=").unwrap();
        let error = Image::try_from(uri).expect_err("should return error");
        assert_matches!(error, ImageError::UnsupportedMimeType(mime_type) if mime_type == "image/webp");
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
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "extends": "https://sd_jwt_vc_metadata.example.com/other_schema",
            "extends#integrity": "sha256-LmXfh-9cLlJNXN-TsMk-PmKjZ5t0WRL5ca_xGgX3c1V",
            "display": [],
        }))
        .unwrap();

        assert_matches!(metadata.as_ref().extends, Some(MetadataExtends { .. }));
    }

    #[rstest]
    #[case(vec![vec!["a.b"], vec!["a", "b"]], "a.b")]
    #[case(vec![vec!["x.y.z"], vec!["x", "y.z"]], "x.y.z")]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y.z"]], "x.y.z")]
    #[case(vec![vec!["x.y", "z"], vec!["x", "y", "z"]], "x.y.z")]
    #[case(vec![vec!["x", "y.z"], vec!["x.y", "z"]], "x.y.z")]
    fn test_claim_path_collision(#[case] claims: Vec<Vec<&str>>, #[case] expected_path: &str) {
        let result = serde_json::from_value::<UncheckedTypeMetadata>(json!({
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "claims": claims.into_iter().map(|claim| HashMap::from([("path", claim)])).collect::<Vec<_>>()
        }))
        .unwrap()
        .detect_path_collisions();

        assert_matches!(result, Err(TypeMetadataError::ClaimPathCollision(path)) if path == expected_path);
    }

    #[test]
    fn should_detect_claim_path_collision_for_deserializing_typemetadata() {
        let result = serde_json::from_value::<TypeMetadata>(json!({
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "claims": [
                { "path": ["address.street"] },
                { "path": ["address", "street"] },
            ]
        }))
        .expect_err("Should fail deserializing type metadata because of path collision");

        assert!(result.to_string().contains("detected claim path collision"));
    }

    fn duplicate_display_language_metadata_json() -> serde_json::Value {
        json!({
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "display": [
                { "locale": "en", "name": "Name" },
                { "locale": "en", "name": "Other name" }
            ],
            "claims": []
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

        assert!(
            error
                .to_string()
                .contains("detected duplicate display metadata language(s)")
        );
    }

    fn duplicate_claim_display_language_metadata_json() -> serde_json::Value {
        json!({
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "claims": [
                {
                    "path": ["address.street"],
                    "display": [
                        { "locale": "en", "label": "Street" },
                        { "locale": "en", "label": "Street name" }
                    ],
                },
            ]
        })
    }

    #[test]
    fn test_error_duplicate_claim_display_languages() {
        let error = serde_json::from_value::<UncheckedTypeMetadata>(duplicate_claim_display_language_metadata_json())
            .unwrap()
            .detect_duplicate_languages()
            .expect_err("duplicate claim display metadata languages should result in an error");

        let expected_path = vec_nonempty![ClaimPath::SelectByKey("address.street".to_string())];
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

        assert!(
            error
                .to_string()
                .contains("detected duplicate claim display metadata language(s)")
        );
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
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "claims": []
        }))
        .unwrap();
        metadata.claims = serde_json::from_value(claims).unwrap();

        assert_type_metadata_error(metadata.validate_svg_ids(), expected);
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
            "vct": VCT_EXAMPLE_CREDENTIAL,
            "display": [{
                    "locale": "en",
                    "name": "Example Credential",
                    "summary": summary,
                }
            ],
            "claims": [
                { "path": vec!["address"], "svg_id": "address" },
                { "path": vec!["address", "street"], "svg_id": "address_street" },
                { "path": vec!["address", "city"], "svg_id": "address_city" },
                { "path": vec!["address", "number"] },
            ]
        }))
        .unwrap();
        assert_type_metadata_error(metadata.validate_svg_ids(), expected);
    }

    #[rstest]
    #[case(json![["nbf"]], Err(TypeMetadataError::InternalAttributeInClaim(vec!["nbf".to_string()])))]
    #[case(json![["nested", "nbf"]], Ok(()))]
    fn test_internal_attributes_in_claim(
        #[case] path: serde_json::Value,
        #[case] expected: Result<(), TypeMetadataError>,
    ) {
        let metadata = serde_json::from_value::<UncheckedTypeMetadata>(json!({
            "vct": "https://sd_jwt_vc_metadata.example.com/example_credential",
            "claims": [{
                "path": path,
                "display": []
            }]
        }))
        .unwrap();

        assert_type_metadata_error(metadata.check_internal_attributes(), expected);
    }

    fn assert_type_metadata_error(result: Result<(), TypeMetadataError>, expected: Result<(), TypeMetadataError>) {
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
