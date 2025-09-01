use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Either;
use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use mdoc::DeviceResponse;
use mdoc::Document;
use mdoc::holder::disclosure::MissingAttributesError;
use utils::vec_at_least::VecNonEmpty;

use crate::CredentialFormat;
use crate::CredentialQueryIdentifier;
use crate::normalized::NormalizedCredentialRequest;
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

/// Ephemeral type that wraps references to format specific types the verifier received from the holder.
#[derive(Debug, Clone)]
pub enum DisclosedCredential<'a> {
    MsoMdoc(&'a Document),
    // TODO (PVW-4139): Support SD-JWT.
}

impl<'a> DisclosedCredential<'a> {
    pub fn new_from_device_response(device_response: &'a DeviceResponse) -> impl Iterator<Item = Self> {
        device_response
            .documents
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(Self::MsoMdoc)
    }

    fn format(&self) -> CredentialFormat {
        match self {
            Self::MsoMdoc(_) => CredentialFormat::MsoMdoc,
        }
    }

    fn credential_type(&self) -> &str {
        match self {
            Self::MsoMdoc(document) => &document.doc_type,
        }
    }

    fn missing_attributes_for_request(
        &self,
        credential_request: &NormalizedCredentialRequest,
    ) -> HashSet<VecNonEmpty<ClaimPath>> {
        match self {
            Self::MsoMdoc(document) => {
                match document
                    .issuer_signed
                    .matches_requested_attributes(credential_request.format_request.claim_paths())
                {
                    Ok(()) => HashSet::new(),
                    Err(MissingAttributesError(missing_attributes)) => missing_attributes,
                }
            }
        }
    }
}

impl NormalizedCredentialRequests {
    /// Match keyed credentials received from the holder against a set of normalized DQCL requests.
    pub fn is_satisfied_by_disclosed_credentials(
        &self,
        disclosed_credentials: &HashMap<&CredentialQueryIdentifier, VecNonEmpty<DisclosedCredential>>,
    ) -> Result<(), CredentialValidationError> {
        // Credential queries that allow for multiple responses are not supported, so make the `HashMap` resolve to a
        // single credential. If at least one of the values has more than one credential, this consitutes an error.
        let (mut single_credentials, multiple_credential_ids): (HashMap<_, _>, HashSet<_>) =
            disclosed_credentials.iter().partition_map(|(id, credentials)| {
                if let Ok(credential) = credentials.iter().exactly_one() {
                    Either::Left((*id, credential))
                } else {
                    Either::Right((*id).clone())
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
                let missing_attributes = credential.missing_attributes_for_request(request);

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

    use itertools::Itertools;
    use rstest::rstest;

    use attestation_types::claim_path::ClaimPath;
    use mdoc::DeviceResponse;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use mdoc::examples::Example;
    use utils::vec_nonempty;

    use crate::CredentialFormat;
    use crate::CredentialQueryIdentifier;
    use crate::normalized::NormalizedCredentialRequests;

    use super::CredentialValidationError;
    use super::DisclosedCredential;

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
        HashMap::from([("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()])]),
        Ok(()),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()]),
            ("mdoc_1".try_into().unwrap(), vec![DeviceResponse::example()]),
        ]),
        Ok(()),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()]),
            ("mdoc_1".try_into().unwrap(), vec![DeviceResponse::example(), DeviceResponse::example()]),
        ]),
        Err(CredentialValidationError::MultipleCredentials(HashSet::from(["mdoc_1".try_into().unwrap()]))),
    )]
    #[case(
        example_mdoc_double_credential_requests(),
        HashMap::from([("mdoc_1".try_into().unwrap(), vec![DeviceResponse::example()])]),
        Err(CredentialValidationError::MissingIdentifiers(HashSet::from(["mdoc_0".try_into().unwrap()]))),
    )]
    #[case(
        example_mdoc_single_credential_requests(),
        HashMap::from([
            ("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()]),
            ("mdoc_1".try_into().unwrap(), vec![DeviceResponse::example()]),
        ]),
        Err(CredentialValidationError::UnexpectedIdentifiers(HashSet::from(["mdoc_1".try_into().unwrap()]))),
    )]
    #[case(
        example_sd_jwt_single_credential_requests(),
        HashMap::from([("sd_jwt_0".try_into().unwrap(), vec![DeviceResponse::example()])]),
        Err(CredentialValidationError::FormatMismatch(
            HashMap::from([("sd_jwt_0".try_into().unwrap(),
            (CredentialFormat::SdJwt, CredentialFormat::MsoMdoc),
        )]))),
    )]
    #[case(
        wrong_credential_type_mdoc_requests(),
        HashMap::from([("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()])]),
        Err(CredentialValidationError::CredentialTypeMismatch(
            HashMap::from([("mdoc_0".try_into().unwrap(),
            (vec!["wrong_credential_type".to_string()], EXAMPLE_DOC_TYPE.to_string()),
        )]))),
    )]
    #[case(
        wrong_attributes_mdoc_requests(),
        HashMap::from([("mdoc_0".try_into().unwrap(), vec![DeviceResponse::example()])]),
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
        #[case] device_responses: HashMap<CredentialQueryIdentifier, Vec<DeviceResponse>>,
        #[case] expected_result: Result<(), CredentialValidationError>,
    ) {
        let result = device_responses
            .iter()
            .map(|(id, device_responses)| {
                let disclosed_credentials = device_responses
                    .iter()
                    .flat_map(DisclosedCredential::new_from_device_response)
                    .collect_vec()
                    .try_into()
                    .unwrap();

                Ok((id, disclosed_credentials))
            })
            .try_collect()
            .and_then(|disclosed_credentials| requests.is_satisfied_by_disclosed_credentials(&disclosed_credentials));

        assert_eq!(result, expected_result);
    }
}
