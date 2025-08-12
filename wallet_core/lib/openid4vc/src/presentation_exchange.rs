//! An implementation of a subset of
//! [Presentation Exchange v2.0.0](https://identity.foundation/presentation-exchange/spec/v2.0.0),
//! implementing only the fields used by the OpenID4VP profile from ISO 18013-7.
//! Other fields are left out of the various structs and enums for now, and some fields that are optional per
//! Presentation Exchange that are always used by the ISO 18013-7 profile are mandatory here.
use std::num::NonZeroUsize;
use std::sync::LazyLock;

use indexmap::IndexSet;
use itertools::Itertools;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

use attestation_types::claim_path::ClaimPath;
use crypto::utils::random_string;
use dcql::CredentialQueryFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use error_category::ErrorCategory;
use mdoc::Document;
use utils::vec_at_least::VecNonEmpty;

use crate::Format;
use crate::openid4vp::FormatAlg;
use crate::openid4vp::VpFormat;

/// As specified in <https://identity.foundation/presentation-exchange/spec/v2.0.0/#presentation-definition>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationDefinition {
    pub id: String,
    pub input_descriptors: Vec<InputDescriptor>,
}

/// As specified in <https://identity.foundation/presentation-exchange/spec/v2.0.0/#input-descriptor-object>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDescriptor {
    pub id: String,
    pub format: VpFormat,
    pub constraints: Constraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub fields: Vec<Field>,
    pub limit_disclosure: LimitDisclosure,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitDisclosure {
    /// The wallet must disclose only those attributes requested by the RP, and no more.
    #[default]
    Required,

    /// The wallet may disclose more attributes to the RP than the ones it requested, for example if the
    /// credential containing them does not support selective disclosure of attributes.
    Preferred,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum PdConversionError {
    #[error("too many paths: expected 1, found {0}")]
    #[category(critical)]
    TooManyPaths(NonZeroUsize),
    #[error("unsupported JsonPath expression: {0}")]
    #[category(critical)]
    UnsupportedJsonPathExpression(String),
    #[error("signature algorithms not supported")]
    #[category(critical)]
    UnsupportedAlgs,
    #[error("too many VCT values in credential request: expected 1, found {0}")]
    #[category(critical)]
    UnsupportedSdJwt(NonZeroUsize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub path: VecNonEmpty<String>,
    pub intent_to_retain: bool,
}

/// Per ISO 18013.7, the field paths in a Presentation Definition must all be a JSONPath expression of the form
/// "$['namespace']['attribute_name']". To be able to support SD-JWTs with nested attributes, we also allow JSONPath
/// expressions with more components, such as "$['path']['to']['attribute']".
///
/// See also <https://identity.foundation/presentation-exchange/spec/v2.0.0/#jsonpath-syntax-definition>.
const FIELD_PATH_ELEMENT_REGEX_STRING: &str = r#"\[['"]([^'"]+)['"]\]"#;
static FIELD_PATH_ELEMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(FIELD_PATH_ELEMENT_REGEX_STRING).unwrap());
static FIELD_PATH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r#"^\$(?:{FIELD_PATH_ELEMENT_REGEX_STRING})+$"#)).unwrap());

impl Field {
    pub(crate) fn parse_paths(&self) -> Result<VecNonEmpty<String>, PdConversionError> {
        let path = self
            .path
            .iter()
            .exactly_one()
            .map_err(|_| PdConversionError::TooManyPaths(self.path.len()))?;

        if !FIELD_PATH_REGEX.is_match(path) {
            return Err(PdConversionError::UnsupportedJsonPathExpression(path.to_string()));
        }

        let segments = FIELD_PATH_ELEMENT_REGEX
            .captures_iter(path)
            .map(|captures| {
                let (_, [segment]) = captures.extract();
                segment.to_string()
            })
            .collect_vec()
            .try_into()
            .unwrap(); // the regex guarantees at least one segment

        Ok(segments)
    }
}

// TODO: Remove in PVW-4419
impl TryFrom<&VecNonEmpty<NormalizedCredentialRequest>> for PresentationDefinition {
    type Error = PdConversionError;

    fn try_from(requested_creds: &VecNonEmpty<NormalizedCredentialRequest>) -> Result<Self, Self::Error> {
        let pd = PresentationDefinition {
            id: random_string(16),
            input_descriptors: requested_creds
                .as_ref()
                .iter()
                .map(|request| {
                    let (id, format) = match &request.format {
                        CredentialQueryFormat::MsoMdoc { doctype_value } => (
                            doctype_value.to_owned(),
                            VpFormat::MsoMdoc {
                                alg: IndexSet::from([FormatAlg::ES256]),
                            },
                        ),
                        CredentialQueryFormat::SdJwt { vct_values } => (
                            vct_values
                                .iter()
                                .exactly_one()
                                .map_err(|_| PdConversionError::UnsupportedSdJwt(vct_values.len()))?
                                .to_owned(),
                            VpFormat::SdJwt {
                                alg: IndexSet::from([FormatAlg::ES256]),
                            },
                        ),
                    };
                    let id = InputDescriptor {
                        id,
                        format,
                        constraints: Constraints {
                            limit_disclosure: LimitDisclosure::Required,
                            fields: request
                                .claims
                                .iter()
                                .map(|attr| Field {
                                    path: vec![format!("$['{}']", attr.path.iter().join("']['"))]
                                        .try_into()
                                        .unwrap(),
                                    intent_to_retain: attr.intent_to_retain,
                                })
                                .collect(),
                        },
                    };

                    Ok(id)
                })
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(pd)
    }
}

/// As specified in https://identity.foundation/presentation-exchange/spec/v2.0.0/#presentation-submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationSubmission {
    pub id: String,
    /// Must be the id value of a valid presentation definition
    pub definition_id: String,
    pub descriptor_map: Vec<InputDescriptorMappingObject>,
}

impl TryFrom<&PresentationDefinition> for VecNonEmpty<NormalizedCredentialRequest> {
    type Error = PdConversionError;

    fn try_from(pd: &PresentationDefinition) -> Result<Self, Self::Error> {
        let credential_requests = pd
            .input_descriptors
            .iter()
            .enumerate()
            .map(|(index, input_descriptor)| {
                let alg = match &input_descriptor.format {
                    VpFormat::MsoMdoc { alg } => alg,
                    VpFormat::SdJwt { alg } => alg,
                };
                if !alg.contains(&FormatAlg::ES256) {
                    return Err(PdConversionError::UnsupportedAlgs);
                }

                let claims = input_descriptor
                    .constraints
                    .fields
                    .iter()
                    .map(|field| {
                        let attrs = field.parse_paths()?;
                        // The unwrap below is safe because we know the vec is non-empty.
                        Ok(AttributeRequest {
                            path: attrs
                                .into_iter()
                                .map(ClaimPath::SelectByKey)
                                .collect_vec()
                                .try_into()
                                .unwrap(),
                            intent_to_retain: field.intent_to_retain,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    // TODO: This is temporary and will be removed when we switch over to using DCQL.
                    .unwrap();

                Ok(NormalizedCredentialRequest {
                    // TODO: This is temporary and will be removed when we switch over to using DCQL.
                    id: format!("mdoc_{index}").try_into().unwrap(),
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: input_descriptor.id.clone(),
                    },
                    claims,
                })
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        Ok(credential_requests.try_into().unwrap()) // TODO: Error Handling
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDescriptorMappingObject {
    /// Matches the `id` property of the Input Descriptor in the Presentation Definition that this Presentation
    /// Submission is related to.
    pub id: String,
    pub format: Format,
    pub path: String,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(critical)]
pub enum PsError {
    #[error("unexpected amount of Presentation Submission descriptors: expected {expected}, found {found}")]
    UnexpectedDescriptorCount { expected: usize, found: usize },
    #[error("received unexpected Presentation Submission ID: expected '{expected}', found '{found}'")]
    UnexpectedSubmissionId { expected: String, found: String },
    #[error("received unexpected path in Presentation Submission Input Descriptor: expected '$', found '{0}'")]
    UnexpectedInputDescriptorPath(String),
    #[error("received unexpected Presentation Submission Input Descriptor ID: expected '{expected}', found '{found}'")]
    UnexpectedInputDescriptorId { expected: String, found: String },
}

impl PresentationSubmission {
    pub fn verify(
        &self,
        documents: &[Document],
        presentation_definition: &PresentationDefinition,
    ) -> Result<(), PsError> {
        if self.definition_id != presentation_definition.id {
            return Err(PsError::UnexpectedSubmissionId {
                expected: presentation_definition.id.clone(),
                found: self.definition_id.clone(),
            });
        }

        if self.descriptor_map.len() != documents.len() {
            return Err(PsError::UnexpectedDescriptorCount {
                expected: documents.len(),
                found: self.descriptor_map.len(),
            });
        }

        for (doc, input_descriptor) in documents.iter().zip(&self.descriptor_map) {
            if input_descriptor.path != "$" {
                return Err(PsError::UnexpectedInputDescriptorPath(
                    input_descriptor.path.to_string(),
                ));
            }
            if input_descriptor.id != doc.doc_type {
                return Err(PsError::UnexpectedInputDescriptorId {
                    expected: doc.doc_type.clone(),
                    found: input_descriptor.id.clone(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use rstest::rstest;
    use serde_json::json;

    use dcql::normalized;
    use dcql::normalized::NormalizedCredentialRequest;
    use utils::vec_at_least::VecNonEmpty;

    use crate::presentation_exchange::Field;
    use crate::presentation_exchange::PdConversionError;

    use super::FIELD_PATH_REGEX;
    use super::FormatAlg;
    use super::LimitDisclosure;
    use super::PresentationDefinition;
    use super::VpFormat;

    #[rstest]
    #[case("$['namespace']['attribute_name']", true)]
    #[case("$['namespace']", true)] // This is needed for SD-JWT
    #[case("$['namespace']['attribute_name']['extra']", true)] // This is needed for SD-JWT nested attributes
    #[case("$['namespace\'']['attribute_name']", false)] // We don't support escaped quotes ...
    #[case("$['namespace']['\"attribute_name']", false)] // ... in namespace or attribute names.
    #[case("$['']", false)]
    #[case(r#"$["namespace"]["attribute_name"]"#, true)] // We also support double quotes ...
    #[case(r#"$['namespace']["attribute_name"]"#, true)] // ... and even mixes between the two
    #[case(r#"$['namespace']['attribute_name"]"#, true)] // (although not required by ISO 18013-7).
    fn field_path_regex(#[case] path: &str, #[case] should_match: bool) {
        assert_eq!(FIELD_PATH_REGEX.is_match(path), should_match);
    }

    #[rstest]
    #[case(vec!["$['namespace']['attribute_name']".to_string()].try_into().unwrap(), Ok(vec!["namespace".to_string(), "attribute_name".to_string()].try_into().unwrap()))]
    #[case(vec!["$['namespace']".to_string()].try_into().unwrap(), Ok(vec!["namespace".to_string()].try_into().unwrap()))]
    #[case(vec!["$['namespace']['attribute_name']['extra']".to_string()].try_into().unwrap(), Ok(vec!["namespace".to_string(), "attribute_name".to_string(), "extra".to_string()].try_into().unwrap()))]
    #[case(vec!["too".to_string(), "many".to_string(), "paths".to_string()].try_into().unwrap(), Err(PdConversionError::TooManyPaths(3.try_into().unwrap())))]
    #[case(vec!["$['']".to_string()].try_into().unwrap(), Err(PdConversionError::UnsupportedJsonPathExpression("$['']".to_string())))]
    fn field_parse_paths(
        #[case] path: VecNonEmpty<String>,
        #[case] expected: Result<VecNonEmpty<String>, PdConversionError>,
    ) {
        let field = Field {
            path,
            intent_to_retain: false,
        };

        assert_eq!(field.parse_paths(), expected);
    }

    #[test]
    fn convert_pd_credential_requests() {
        let orginal: VecNonEmpty<NormalizedCredentialRequest> = normalized::mock::example();
        let pd: PresentationDefinition = (&orginal).try_into().unwrap();
        let converted: VecNonEmpty<NormalizedCredentialRequest> = (&pd).try_into().unwrap();

        assert_eq!(orginal.len(), converted.len());
        assert_eq!(orginal.first().format, converted.first().format);
        assert_eq!(orginal.first().claims, converted.first().claims);
    }

    #[test]
    fn deserialize_example_presentation_definition() {
        let example_json = json!(
            {
                "id": "mDL-sample-req",
                "input_descriptors": [
                    {
                        "format": {
                            "mso_mdoc": {
                                "alg": [ "ES256" ]
                            }
                        },
                        "id": "org.iso.18013.5.1.mDL",
                        "constraints": {
                            "limit_disclosure": "required",
                            "fields": [
                                { "path": ["$['org.iso.18013.5.1']['family_name']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['birth_date']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['document_number']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['driving_privileges']"], "intent_to_retain": false }
                            ]
                        }
                    }
                ]
            }
        );

        let pd: PresentationDefinition = serde_json::from_value(example_json).unwrap();

        assert_eq!(pd.id, "mDL-sample-req".to_string());

        let input_descriptor = &pd.input_descriptors[0];
        assert_eq!(input_descriptor.id, "org.iso.18013.5.1.mDL");
        assert_matches!(
            &input_descriptor.format,
            VpFormat::MsoMdoc { alg } if alg.len() == 1 && matches!(alg[0], FormatAlg::ES256)
        );
        assert_matches!(input_descriptor.constraints.limit_disclosure, LimitDisclosure::Required);

        let field = &input_descriptor.constraints.fields[0];
        assert_eq!(field.path[0], "$['org.iso.18013.5.1']['family_name']");
        assert!(!field.intent_to_retain);
    }
}
