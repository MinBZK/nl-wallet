use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZero;

use indexmap::IndexMap;
use indexmap::IndexSet;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;

use attestation_types::attribute_paths::AttestationAttributePaths;
use attestation_types::attribute_paths::AttestationAttributePathsError;
use dcql::ClaimPath;
use dcql::CredentialQueryFormat;
use mdoc::identifiers::AttributeIdentifier;
use mdoc::identifiers::AttributeIdentifierHolder;
use mdoc::verifier::ItemsRequests;
use mdoc::DeviceResponse;
use mdoc::ItemsRequest;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("attributes mismatch: {0:?}")]
    MissingAttributes(Vec<AttributeIdentifier>),
    #[error("expected an mdoc")]
    ExpectedMdoc,
}

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, AsRef, TryFrom, Into, IntoIterator, Serialize, Deserialize),
    validate(predicate = |items| !items.is_empty()),
)]
pub struct NormalizedCredentialRequests(Vec<NormalizedCredentialRequest>);

impl NormalizedCredentialRequests {
    pub fn match_against_response(&self, device_response: &DeviceResponse) -> Result<(), ResponseError> {
        let not_found = self
            .as_ref()
            .iter()
            .map(|request| {
                let CredentialQueryFormat::MsoMdoc { ref doctype_value } = request.format else {
                    return Err(ResponseError::ExpectedMdoc);
                };

                let not_found = device_response
                    .documents
                    .as_ref()
                    .and_then(|docs| docs.iter().find(|doc| doc.doc_type == *doctype_value))
                    .map_or_else(
                        // If the entire document is missing then all requested attributes are missing
                        || request.attribute_identifiers().into_iter().collect(),
                        |doc| request.match_against_issuer_signed(doc),
                    );
                Ok(not_found)
            })
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(ResponseError::MissingAttributes(not_found))
        }
    }

    pub fn try_into_attribute_paths(self) -> Result<AttestationAttributePaths, AttestationAttributePathsError> {
        let paths = self
            .into_iter()
            .fold(HashMap::<_, HashSet<_>>::new(), |mut paths, request| {
                // For an mdoc items request, simply make a path of length 2,
                // consisting of the name space and element identifier.
                let CredentialQueryFormat::MsoMdoc { doctype_value } = request.format else {
                    panic!("sd-jwt is not yet supported")
                };

                let attributes: HashSet<VecNonEmpty<String>> = request
                    .claims
                    .into_iter()
                    .map(|claim| {
                        let attrs = claim.path.into_iter().map(|attr| format!("{attr}")).collect::<Vec<_>>();
                        VecNonEmpty::try_from(attrs).expect("source was also non empty")
                    })
                    .collect();

                // In case a doc type occurs multiple times, merge the paths.
                paths.entry(doctype_value).or_default().extend(attributes);

                paths
            });

        AttestationAttributePaths::try_new(paths)
    }
}

/// Request for a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCredentialRequest {
    pub format: CredentialQueryFormat,
    pub claims: Vec<AttributeRequest>,
}

/// Request for a single attribute with the given [path].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRequest {
    pub path: VecNonEmpty<ClaimPath>,
    pub intent_to_retain: bool,
}

impl AttributeIdentifierHolder for NormalizedCredentialRequest {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        let CredentialQueryFormat::MsoMdoc { doctype_value } = &self.format else {
            panic!("SdJwt not supported yet");
        };
        self.claims
            .iter()
            .map(|claim| claim.to_attribute_identifier(doctype_value).unwrap())
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MdocCredentialRequestError {
    #[error("unexpected amount of claim paths: {0}")]
    UnexpectedClaimsPathAmount(NonZero<usize>),
    #[error("unexpected claim path type, expected key string")]
    UnexpectedClaimsPathType,
}

impl AttributeRequest {
    pub fn to_namespace_and_attribute(&self) -> Result<(&str, &str), MdocCredentialRequestError> {
        if self.path.len() != 2.try_into().unwrap() {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathAmount(self.path.len()));
        }
        let ClaimPath::SelectByKey(namespace) = &self.path[0] else {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathType);
        };
        let ClaimPath::SelectByKey(attribute) = &self.path[1] else {
            return Err(MdocCredentialRequestError::UnexpectedClaimsPathType);
        };
        Ok((namespace, attribute))
    }

    pub fn to_attribute_identifier(&self, doc_type: &str) -> Result<AttributeIdentifier, MdocCredentialRequestError> {
        let (namespace, attribute) = self.to_namespace_and_attribute()?;

        let identifier = AttributeIdentifier {
            credential_type: doc_type.to_owned(),
            namespace: namespace.to_owned(),
            attribute: attribute.to_owned(),
        };

        Ok(identifier)
    }
}

impl TryFrom<NormalizedCredentialRequest> for ItemsRequest {
    type Error = MdocCredentialRequestError;

    fn try_from(source: NormalizedCredentialRequest) -> Result<Self, Self::Error> {
        let CredentialQueryFormat::MsoMdoc { doctype_value } = &source.format else {
            panic!("SdJwt not supported yet");
        };

        let name_spaces = source
            .claims
            .into_iter()
            .map(|req| {
                let (ns, attr) = req.to_namespace_and_attribute().unwrap(); // TODO: error handling
                (ns.to_owned(), attr.to_owned(), req.intent_to_retain)
            })
            .fold(IndexMap::new(), |mut acc, (ns, attr, intent_to_retain)| {
                let entry: &mut IndexMap<_, _> = acc.entry(ns).or_default();
                entry.insert(attr, intent_to_retain);
                acc
            });

        let items_request = ItemsRequest {
            doc_type: doctype_value.clone(),
            name_spaces,
            request_info: None,
        };
        Ok(items_request)
    }
}

impl From<ItemsRequest> for NormalizedCredentialRequest {
    fn from(source: ItemsRequest) -> Self {
        let doctype_value = source.doc_type;

        let format = CredentialQueryFormat::MsoMdoc { doctype_value };

        // unwrap below is safe because claims path is not empty
        let claims = source
            .name_spaces
            .into_iter()
            .flat_map(|(namespace, attrs)| {
                attrs
                    .into_iter()
                    .map(move |(attribute, intent_to_retain)| AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(namespace.clone()),
                            ClaimPath::SelectByKey(attribute.clone()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain,
                    })
            })
            .collect();

        NormalizedCredentialRequest { format, claims }
    }
}

impl From<ItemsRequests> for NormalizedCredentialRequests {
    fn from(source: ItemsRequests) -> Self {
        source
            .0
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use dcql::ClaimPath;
    use dcql::CredentialQueryFormat;
    use mdoc::examples::EXAMPLE_ATTR_NAME;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use mdoc::test::data::ADDR;
    use mdoc::test::data::ADDR_NS;
    use mdoc::test::data::PID;
    use mdoc::test::TestDocument;
    use mdoc::test::TestDocuments;

    use super::AttributeRequest;
    use super::NormalizedCredentialRequest;
    use super::NormalizedCredentialRequests;

    impl From<TestDocument> for NormalizedCredentialRequest {
        fn from(source: TestDocument) -> Self {
            let format = CredentialQueryFormat::MsoMdoc {
                doctype_value: source.doc_type,
            };

            // unwrap below is safe because claims path is not empty
            let claims = source
                .namespaces
                .into_iter()
                .flat_map(|(namespace, attrs)| {
                    attrs.into_iter().map(move |entry| AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(namespace.clone()),
                            ClaimPath::SelectByKey(entry.name),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    })
                })
                .collect();

            NormalizedCredentialRequest { format, claims }
        }
    }

    // impl TryFrom<TestDocuments> for NormalizedCredentialRequests {
    //     type Error = NormalizedCredentialRequestsError;

    //     fn try_from(source: TestDocuments) -> Result<Self, Self::Error> {
    //         NormalizedCredentialRequests::try_new(source.0.into_iter().map(Into::into).collect())
    //     }
    // }

    impl From<TestDocuments> for NormalizedCredentialRequests {
        fn from(source: TestDocuments) -> Self {
            NormalizedCredentialRequests::try_new(source.0.into_iter().map(Into::into).collect()).unwrap()
        }
    }

    impl NormalizedCredentialRequest {
        pub fn new_example() -> Self {
            // unwrap below is safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![AttributeRequest {
                    path: vec![
                        ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                        ClaimPath::SelectByKey(EXAMPLE_ATTR_NAME.to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    intent_to_retain: true,
                }],
            }
        }

        pub fn pid_full_name() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: PID.to_string(),
                },
                claims: vec![
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(PID.to_string()),
                            ClaimPath::SelectByKey("family_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(PID.to_string()),
                            ClaimPath::SelectByKey("given_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    },
                ],
            }
        }

        pub fn addr_street() -> Self {
            // unwraps below are safe because claims path is not empty
            Self {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: ADDR.to_string(),
                },
                claims: vec![AttributeRequest {
                    path: vec![
                        ClaimPath::SelectByKey(ADDR_NS.to_string()),
                        ClaimPath::SelectByKey("street_address".to_string()),
                    ]
                    .try_into()
                    .unwrap(),
                    intent_to_retain: true,
                }],
            }
        }
    }

    impl NormalizedCredentialRequests {
        pub fn example() -> Self {
            vec![NormalizedCredentialRequest {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: EXAMPLE_DOC_TYPE.to_string(),
                },
                claims: vec![
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("family_name".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("issue_date".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("expiry_date".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("document_number".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("portrait".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                    AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey("driving_privileges".to_string()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: false,
                    },
                ],
            }]
            .try_into()
            .unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use mdoc::test::data::addr_street;
    use mdoc::test::data::pid_full_name;
    use mdoc::test::TestDocuments;
    use mdoc::ItemsRequest;

    use super::NormalizedCredentialRequest;

    #[rstest]
    #[case(NormalizedCredentialRequest::pid_full_name(), pid_full_name())]
    #[case(NormalizedCredentialRequest::addr_street(), addr_street())]
    fn try_from_credential_request_for_items_request(
        #[case] input: NormalizedCredentialRequest,
        #[case] expected: TestDocuments,
    ) {
        let actual: ItemsRequest = input.try_into().unwrap();

        assert_eq!(actual, expected.into_first().unwrap().into());
    }
}
