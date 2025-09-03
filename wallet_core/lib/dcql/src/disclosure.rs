use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Either;
use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use utils::vec_at_least::VecNonEmpty;

use crate::CredentialFormat;
use crate::CredentialQueryIdentifier;
use crate::normalized::NormalizedCredentialRequests;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum CredentialValidationError {
    #[error("multiple credentials received for identifier(s): {}", .0.iter().join(", "))]
    MultipleCredentials(HashSet<CredentialQueryIdentifier>),
    #[error("missing request identifier(s) in received credential(s): {}", .0.iter().join(", "))]
    MissingIdentifiers(HashSet<CredentialQueryIdentifier>),
    #[error("received unexpected identifier(s): {}", .0.iter().join(", "))]
    UnexpectedIdentifiers(HashSet<CredentialQueryIdentifier>),
    #[error("received incorrect format for identifier(s): {}", .0.iter().map(|(id, (expected, received))| {
        format!("({id}): expected \"{expected}\", received \"{received}\"")
    }).join(" / "))]
    FormatMismatch(HashMap<CredentialQueryIdentifier, (CredentialFormat, CredentialFormat)>),
    #[error("received incorrect credential type for identifier(s): {}", .0.iter().map(|(id, (expected, received))| {
        format!(
            "({}): expected {}, received \"{}\"",
            id,
            expected.iter().map(|expected| format!("\"{expected}\"")).join(" or "),
            received
        )
    }).join(" / "))]
    CredentialTypeMismatch(HashMap<CredentialQueryIdentifier, (Vec<String>, String)>),
    #[error("requested attributes are missing for identifier(s): {}", .0.iter().map(|(id, paths)| {
        format!("({}): {}", id, paths.iter().map(|path| {
            format!("[{}]", path.iter().join(", "))
        }).join(", "))
    }).join(" / "))]
    MissingAttributes(HashMap<CredentialQueryIdentifier, HashSet<VecNonEmpty<ClaimPath>>>),
}

/// This should be implemented on a credential that a verifier receives from the holder.
pub trait DisclosedCredential {
    fn format(&self) -> CredentialFormat;
    fn credential_type(&self) -> &str;
    fn missing_claim_paths<'a, 'b>(
        &'a self,
        request_claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> HashSet<VecNonEmpty<ClaimPath>>;
}

impl NormalizedCredentialRequests {
    /// Match keyed credentials received from the holder against a set of normalized DQCL requests.
    pub fn is_satisfied_by_disclosed_credentials(
        &self,
        disclosed_credentials: &HashMap<CredentialQueryIdentifier, VecNonEmpty<impl DisclosedCredential>>,
    ) -> Result<(), CredentialValidationError> {
        // Credential queries that allow for multiple responses are not supported, so make the `HashMap` resolve to a
        // single credential. If at least one of the values has more than one credential, this consitutes an error.
        let (mut single_credentials, multiple_credential_ids): (HashMap<_, _>, HashSet<_>) =
            disclosed_credentials.iter().partition_map(|(id, credentials)| {
                if let Ok(credential) = credentials.iter().exactly_one() {
                    Either::Left((id, credential))
                } else {
                    Either::Right(id.clone())
                }
            });

        if !multiple_credential_ids.is_empty() {
            return Err(CredentialValidationError::MultipleCredentials(multiple_credential_ids));
        }

        // Combine the queries and credentials into a single `HashMap`. If a query identifier is not found in the
        // credential response, this consitutes an error, as optional credentials are not supported.
        let (requests_and_credentials, missing_ids): (HashMap<_, _>, HashSet<_>) =
            self.as_ref().iter().partition_map(|request| {
                if let Some(credential) = single_credentials.remove(&request.id) {
                    Either::Left((&request.id, (request, credential)))
                } else {
                    Either::Right(request.id.clone())
                }
            });

        if !missing_ids.is_empty() {
            return Err(CredentialValidationError::MissingIdentifiers(missing_ids));
        }

        // If the response contained a query identifier that was not part of the query, this is also an error.
        if !single_credentials.is_empty() {
            let unexpected_ids = single_credentials.into_keys().cloned().collect();

            return Err(CredentialValidationError::UnexpectedIdentifiers(unexpected_ids));
        }

        // Each received credential should be of the requested format.
        let format_mismatches = requests_and_credentials
            .iter()
            .filter_map(|(id, (request, credential))| {
                let expected_format = request.format_request.format();
                let received_format = credential.format();

                (received_format != expected_format).then(|| ((*id).clone(), (expected_format, received_format)))
            })
            .collect::<HashMap<_, _>>();

        if !format_mismatches.is_empty() {
            return Err(CredentialValidationError::FormatMismatch(format_mismatches));
        }

        // Each received credential should be of (one of) the requested credential type(s) for that query.
        let credential_type_mismatches = requests_and_credentials
            .iter()
            .filter_map(|(id, (request, credential))| {
                let credential_type = credential.credential_type();

                (!request.format_request.credential_types().contains(credential_type)).then(|| {
                    (
                        (*id).clone(),
                        (
                            request
                                .format_request
                                .credential_types()
                                .map(str::to_string)
                                .collect_vec(),
                            credential_type.to_string(),
                        ),
                    )
                })
            })
            .collect::<HashMap<_, _>>();

        if !credential_type_mismatches.is_empty() {
            return Err(CredentialValidationError::CredentialTypeMismatch(
                credential_type_mismatches,
            ));
        }

        // Finally, each received credential should contain all of the requested attributes,
        // as optional attributes are not supported.
        let missing_attribute_credentials = requests_and_credentials
            .into_iter()
            .filter_map(|(id, (request, credential))| {
                let missing_attributes = credential.missing_claim_paths(request.format_request.claim_paths());

                (!missing_attributes.is_empty()).then(|| (id.clone(), missing_attributes))
            })
            .collect::<HashMap<_, _>>();

        if !missing_attribute_credentials.is_empty() {
            return Err(CredentialValidationError::MissingAttributes(
                missing_attribute_credentials,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use rstest::rstest;

    use attestation_types::claim_path::ClaimPath;
    use mdoc::examples::EXAMPLE_ATTRIBUTES;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use crate::CredentialFormat;
    use crate::CredentialQueryIdentifier;
    use crate::normalized::NormalizedCredentialRequests;

    use super::CredentialValidationError;
    use super::DisclosedCredential;

    /// A very simple type that implements [`MockDisclosedCredential`] for testing.
    struct MockDisclosedCredential {
        format: CredentialFormat,
        credential_type: String,
        claim_paths: HashSet<VecNonEmpty<ClaimPath>>,
    }

    impl MockDisclosedCredential {
        pub fn example() -> Self {
            Self {
                format: CredentialFormat::MsoMdoc,
                credential_type: EXAMPLE_DOC_TYPE.to_string(),
                claim_paths: EXAMPLE_ATTRIBUTES
                    .iter()
                    .map(|attribute| {
                        vec_nonempty![
                            ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                            ClaimPath::SelectByKey(attribute.to_string())
                        ]
                    })
                    .collect(),
            }
        }
    }

    impl DisclosedCredential for MockDisclosedCredential {
        fn format(&self) -> CredentialFormat {
            self.format
        }

        fn credential_type(&self) -> &str {
            &self.credential_type
        }

        fn missing_claim_paths<'a, 'b>(
            &'a self,
            request_claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
        ) -> HashSet<VecNonEmpty<ClaimPath>> {
            request_claim_paths
                .into_iter()
                .collect::<HashSet<_>>()
                .difference(&self.claim_paths.iter().collect())
                .copied()
                .cloned()
                .collect()
        }
    }

    fn example_mdoc_single_credential_requests() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            EXAMPLE_DOC_TYPE,
            &[
                &[EXAMPLE_NAMESPACE, "family_name"],
                &[EXAMPLE_NAMESPACE, "issue_date"],
                &[EXAMPLE_NAMESPACE, "expiry_date"],
                &[EXAMPLE_NAMESPACE, "document_number"],
                &[EXAMPLE_NAMESPACE, "portrait"],
                &[EXAMPLE_NAMESPACE, "driving_privileges"],
            ],
        )])
    }

    fn example_mdoc_double_credential_requests() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[
            (
                EXAMPLE_DOC_TYPE,
                &[
                    &[EXAMPLE_NAMESPACE, "driving_privileges"],
                    &[EXAMPLE_NAMESPACE, "document_number"],
                ],
            ),
            (
                EXAMPLE_DOC_TYPE,
                &[&[EXAMPLE_NAMESPACE, "family_name"], &[EXAMPLE_NAMESPACE, "portrait"]],
            ),
        ])
    }

    fn wrong_credential_type_mdoc_requests() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            "wrong_credential_type",
            &[&[EXAMPLE_NAMESPACE, "family_name"]],
        )])
    }

    fn wrong_attributes_mdoc_requests() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_mdoc_from_slices(&[(
            EXAMPLE_DOC_TYPE,
            &[
                &[EXAMPLE_NAMESPACE, "family_name"],
                &[EXAMPLE_NAMESPACE, "favourite_colour"],
                &[EXAMPLE_NAMESPACE, "average_airspeed"],
            ],
        )])
    }

    fn example_sd_jwt_single_credential_requests() -> NormalizedCredentialRequests {
        NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[(&[EXAMPLE_DOC_TYPE], &[&["family_name"]])])
    }

    #[rstest]
    #[case(
        example_mdoc_single_credential_requests(),
        HashMap::from([("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()])]),
        Ok(()),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()]),
            ("mdoc_1".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()]),
        ]),
        Ok(()),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()]),
            ("mdoc_1".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example(), MockDisclosedCredential::example()]),
        ]),
        Err(CredentialValidationError::MultipleCredentials(HashSet::from(["mdoc_1".try_into().unwrap()]))),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([("mdoc_1".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()])]),
        Err(CredentialValidationError::MissingIdentifiers(HashSet::from(["mdoc_0".try_into().unwrap()]))),
    )]
    #[case(
        example_mdoc_single_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()]),
            ("mdoc_1".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()]),
        ]),
        Err(CredentialValidationError::UnexpectedIdentifiers(HashSet::from(["mdoc_1".try_into().unwrap()]))),
    )]
    #[case(
        example_sd_jwt_single_credential_requests(),
        HashMap::from([("sd_jwt_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()])]),
        Err(CredentialValidationError::FormatMismatch(
            HashMap::from([("sd_jwt_0".try_into().unwrap(),
            (CredentialFormat::SdJwt, CredentialFormat::MsoMdoc),
        )]))),
    )]
    #[case(
        wrong_credential_type_mdoc_requests(),
        HashMap::from([("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()])]),
        Err(CredentialValidationError::CredentialTypeMismatch(
            HashMap::from([("mdoc_0".try_into().unwrap(),
            (vec!["wrong_credential_type".to_string()], EXAMPLE_DOC_TYPE.to_string()),
        )]))),
    )]
    #[case(
        wrong_attributes_mdoc_requests(),
        HashMap::from([("mdoc_0".try_into().unwrap(), vec_nonempty![MockDisclosedCredential::example()])]),
        Err(CredentialValidationError::MissingAttributes(
            HashMap::from([("mdoc_0".try_into().unwrap(),
            HashSet::from([
                vec_nonempty![
                    ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                    ClaimPath::SelectByKey("favourite_colour".to_string()),
                ],
                vec_nonempty![
                    ClaimPath::SelectByKey(EXAMPLE_NAMESPACE.to_string()),
                    ClaimPath::SelectByKey("average_airspeed".to_string()),
                ],
            ]),
        )]))),
    )]
    fn test_normalized_credential_requests_is_satisfied_by_disclosed_credentials(
        #[case] requests: NormalizedCredentialRequests,
        #[case] disclosed_credentials: HashMap<CredentialQueryIdentifier, VecNonEmpty<MockDisclosedCredential>>,
        #[case] expected_result: Result<(), CredentialValidationError>,
    ) {
        let result = requests.is_satisfied_by_disclosed_credentials(&disclosed_credentials);

        assert_eq!(result, expected_result);
    }
}
