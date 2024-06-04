//! An implementation of a subset of
//! [Presentation Exchange v2.0.0](https://identity.foundation/presentation-exchange/spec/v2.0.0),
//! implementing only the fields used by the OpenID4VP profile from ISO 18013-7.
//! Other fields are left out of the various structs and enums for now, and some fields that are optional per
//! Presentation Exchange that are always used by the ISO 18013-7 profile are mandatory here.

use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::{verifier::ItemsRequests, ItemsRequest};
use wallet_common::utils::random_string;

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
    pub format: RequestedFormat,
    pub constraints: Constraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedFormat {
    MsoMdoc { alg: IndexSet<FormatAlg> },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatAlg {
    #[default]
    ES256,
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
    /// attestation containing them does not support selective disclosure of attributes.
    Preferred,
}

#[derive(Debug, thiserror::Error)]
pub enum PdConversionError {
    #[error("too many paths")]
    TooManyPaths,
    #[error("unsupported JsonPath expression")]
    UnsupportedJsonPathExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub path: Vec<String>,
    pub intent_to_retain: bool,
}

impl Field {
    fn parse_paths(&self) -> Result<(String, String), PdConversionError> {
        if self.path.len() != 1 {
            return Err(PdConversionError::TooManyPaths);
        }
        let path = &self.path[0];

        /// Per ISO 18013.7, the path must be a JSONPath expression of the form: "$['namespace']['attribute_name']"
        /// See also https://identity.foundation/presentation-exchange/spec/v2.0.0/#jsonpath-syntax-definition
        static FIELD_PATH_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"^\$\[['"](.*?)['"]\]\[['"](.*?)['"]\]$"#).unwrap());

        let captures = FIELD_PATH_REGEX
            .captures(path)
            .ok_or(PdConversionError::UnsupportedJsonPathExpression)?;

        // `captures` will always have three elements due to how the regex is constructed.
        Ok((captures[1].to_string(), captures[2].to_string()))
    }
}

impl From<&ItemsRequests> for PresentationDefinition {
    fn from(items_requests: &ItemsRequests) -> Self {
        PresentationDefinition {
            id: random_string(16),
            input_descriptors: items_requests
                .0
                .iter()
                .map(|items_request| InputDescriptor {
                    id: items_request.doc_type.clone(),
                    format: RequestedFormat::MsoMdoc {
                        alg: IndexSet::from([FormatAlg::ES256]),
                    },
                    constraints: Constraints {
                        limit_disclosure: LimitDisclosure::Required,
                        fields: items_request
                            .name_spaces
                            .iter()
                            .flat_map(|(namespace, attrs)| {
                                attrs.iter().map(|(attr, intent_to_retain)| Field {
                                    path: vec![format!("$['{}']['{}']", namespace.as_str(), attr.as_str())],
                                    intent_to_retain: *intent_to_retain,
                                })
                            })
                            .collect(),
                    },
                })
                .collect(),
        }
    }
}

impl TryFrom<&PresentationDefinition> for ItemsRequests {
    type Error = PdConversionError;

    fn try_from(pd: &PresentationDefinition) -> Result<Self, Self::Error> {
        let items_requests = pd
            .input_descriptors
            .iter()
            .map(|input_descriptor| {
                let mut name_spaces: IndexMap<String, IndexMap<String, bool>> = IndexMap::new();
                for field in &input_descriptor.constraints.fields {
                    let (namespace, attr) = field.parse_paths()?;
                    name_spaces
                        .entry(namespace)
                        .or_default()
                        .insert(attr, field.intent_to_retain);
                }

                Ok(ItemsRequest {
                    doc_type: input_descriptor.id.clone(),
                    request_info: None,
                    name_spaces,
                })
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        Ok(items_requests.into())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDescriptorMappingObject {
    /// Matches the `id` property of the Input Descriptor in the Presentation Definition that this Presentation
    /// Submission is related to.
    pub id: String,
    pub format: Format,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::json;

    use nl_wallet_mdoc::{examples::Examples, verifier::ItemsRequests};

    use crate::presentation_exchange::{FormatAlg, LimitDisclosure, RequestedFormat};

    use super::PresentationDefinition;

    #[test]
    fn convert_pd_itemsrequests() {
        let items_requests: ItemsRequests = Examples::items_requests();
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
            RequestedFormat::MsoMdoc { alg } if alg.len() == 1 && matches!(alg[0], FormatAlg::ES256)
        );
        assert_matches!(input_descriptor.constraints.limit_disclosure, LimitDisclosure::Required);

        let field = &input_descriptor.constraints.fields[0];
        assert_eq!(field.path[0], "$['org.iso.18013.5.1']['family_name']");
        assert!(!field.intent_to_retain);
    }
}
