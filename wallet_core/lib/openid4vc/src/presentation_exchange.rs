//! An implementation of a subset of
//! [Presentation Exchange v2.0.0](https://identity.foundation/presentation-exchange/spec/v2.0.0),
//! implementing only the fields used by the OpenID4VP profile from ISO 18013-7.
//! Other fields are left out of the various structs and enums for now, and some fields that are optional per
//! Presentation Exchange that are always used by the ISO 18013-7 profile are mandatory here.
use std::sync::LazyLock;

use indexmap::IndexSet;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

use attestation_data::request::AttributeRequest;
use attestation_data::request::NormalizedCredentialRequest;
use attestation_data::request::NormalizedCredentialRequests;
use crypto::utils::random_string;
use dcql::ClaimPath;
use dcql::CredentialQueryFormat;
use error_category::ErrorCategory;
use mdoc::Document;

use crate::openid4vp::FormatAlg;
use crate::openid4vp::VpFormat;
use crate::Format;

/// As specified in https://identity.foundation/presentation-exchange/spec/v2.0.0/#presentation-definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationDefinition {
    pub id: String,
    pub input_descriptors: Vec<InputDescriptor>,
}

/// As specified in https://identity.foundation/presentation-exchange/spec/v2.0.0/#input-descriptor-object.
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
pub enum PdConversionError {
    #[error("too many paths")]
    #[category(critical)]
    TooManyPaths,
    #[error("unsupported JsonPath expression")]
    #[category(critical)]
    UnsupportedJsonPathExpression,
    #[error("signature algorithms not supported")]
    #[category(critical)]
    UnsupportedAlgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub path: Vec<String>,
    pub intent_to_retain: bool,
}

/// Per ISO 18013.7, the field paths in a Presentation Definition must all be a JSONPath expression of the form
/// "$['namespace']['attribute_name']".
///
/// See also https://identity.foundation/presentation-exchange/spec/v2.0.0/#jsonpath-syntax-definition
static FIELD_PATH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\$\[['"]([^'"]*)['"]\]\[['"]([^'"]*)['"]\]$"#).unwrap());

impl Field {
    pub(crate) fn parse_paths(&self) -> Result<(String, String), PdConversionError> {
        if self.path.len() != 1 {
            return Err(PdConversionError::TooManyPaths);
        }
        let path = &self.path[0];

        let captures = FIELD_PATH_REGEX
            .captures(path)
            .ok_or(PdConversionError::UnsupportedJsonPathExpression)?;

        // `captures` will always have three elements due to how the regex is constructed.
        Ok((captures[1].to_string(), captures[2].to_string()))
    }
}

impl From<&NormalizedCredentialRequests> for PresentationDefinition {
    fn from(requested_creds: &NormalizedCredentialRequests) -> Self {
        PresentationDefinition {
            id: random_string(16),
            input_descriptors: requested_creds
                .as_ref()
                .iter()
                .map(|request| {
                    let CredentialQueryFormat::MsoMdoc { doctype_value } = &request.format else {
                        panic!("SdJwt not supported yet");
                    };
                    InputDescriptor {
                        id: doctype_value.clone(),
                        format: VpFormat::MsoMdoc {
                            alg: IndexSet::from([FormatAlg::ES256]),
                        },
                        constraints: Constraints {
                            limit_disclosure: LimitDisclosure::Required,
                            fields: request
                                .claims
                                .iter()
                                .map(|attr_req| {
                                    let (namespace, attr) = attr_req.to_namespace_and_attribute().unwrap(); // TODO: error handling, TryFrom?
                                    Field {
                                        path: vec![format!("$['{}']['{}']", namespace, attr)],
                                        intent_to_retain: attr_req.intent_to_retain,
                                    }
                                })
                                .collect(),
                        },
                    }
                })
                .collect(),
        }
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

impl TryFrom<&PresentationDefinition> for NormalizedCredentialRequests {
    type Error = PdConversionError;

    fn try_from(pd: &PresentationDefinition) -> Result<Self, Self::Error> {
        let items_requests = pd
            .input_descriptors
            .iter()
            .map(|input_descriptor| {
                let VpFormat::MsoMdoc { alg } = &input_descriptor.format;
                if !alg.contains(&FormatAlg::ES256) {
                    return Err(PdConversionError::UnsupportedAlgs);
                }

                let claims = input_descriptor
                    .constraints
                    .fields
                    .iter()
                    .map(|field| {
                        let (namespace, attr) = field.parse_paths()?;
                        // The unwrap below is safe because we know the vec is non-empty.
                        Ok(AttributeRequest {
                            path: vec![ClaimPath::SelectByKey(namespace), ClaimPath::SelectByKey(attr)]
                                .try_into()
                                .unwrap(),
                            intent_to_retain: field.intent_to_retain,
                        })
                    })
                    .collect::<Result<_, _>>()?;

                Ok(NormalizedCredentialRequest {
                    format: CredentialQueryFormat::MsoMdoc {
                        doctype_value: input_descriptor.id.clone(),
                    },
                    claims,
                })
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        Ok(items_requests.try_into().unwrap()) // TODO: Error Handling
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

    use mdoc::examples::example_items_requests;
    use mdoc::verifier::ItemsRequests;

    use super::FormatAlg;
    use super::LimitDisclosure;
    use super::PresentationDefinition;
    use super::VpFormat;
    use super::FIELD_PATH_REGEX;

    #[rstest]
    #[case("$['namespace']['attribute_name']", true)]
    #[case("$['namespace']", false)]
    #[case("$['namespace']['attribute_name']['extra']", false)]
    #[case("$['namespace\'']['attribute_name']", false)] // We don't support escaped quotes ...
    #[case("$['namespace']['\"attribute_name']", false)] // ... in namespace or attribute names.
    #[case(r#"$["namespace"]["attribute_name"]"#, true)] // We also support double quotes ...
    #[case(r#"$['namespace']["attribute_name"]"#, true)] // ... and even mixes between the two
    #[case(r#"$['namespace']['attribute_name"]"#, true)] // (although not required by ISO 18013-7).
    fn field_path_regex(#[case] path: &str, #[case] should_match: bool) {
        assert_eq!(FIELD_PATH_REGEX.is_match(path), should_match);
    }

    #[test]
    fn convert_pd_itemsrequests() {
        let items_requests: ItemsRequests = example_items_requests();
        let pd: PresentationDefinition = (&items_requests).into();
        let converted: ItemsRequests = (&pd).try_into().unwrap();

        assert_eq!(items_requests, converted);
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
