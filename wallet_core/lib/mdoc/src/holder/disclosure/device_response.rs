use itertools::Itertools;

use crypto::CredentialEcdsaKey;
use crypto::factory::KeyFactory;
use dcql::CredentialQueryFormat;
use dcql::normalized::NormalizedCredentialRequest;

use crate::errors::Error;
use crate::errors::Result;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::disclosure::DeviceResponseVersion;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

use super::super::Mdoc;
use super::IssuerSignedMatchingError;
use super::ResponseMatchingError;

impl DeviceResponse {
    pub fn new(documents: Vec<Document>) -> Self {
        Self {
            version: DeviceResponseVersion::default(),
            documents: Some(documents),
            document_errors: None,
            status: 0,
        }
    }

    pub async fn sign_from_mdocs<K, KF>(
        mdocs: Vec<Mdoc>,
        session_transcript: &SessionTranscript,
        key_factory: &KF,
    ) -> Result<(Self, Vec<K>)>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
    {
        // Prepare the credential keys and device auth challenges per mdoc.
        let (keys, challenges) = mdocs
            .iter()
            .map(|mdoc| {
                let credential_key = mdoc.credential_key(key_factory)?;
                let device_signed_challenge =
                    DeviceAuthenticationKeyed::challenge(&mdoc.mso.doc_type, session_transcript)?;

                Ok((credential_key, device_signed_challenge))
            })
            .process_results::<_, _, Error, (Vec<_>, Vec<_>)>(|iter| iter.unzip())?;

        let keys_and_challenges = keys
            .into_iter()
            .zip(&challenges)
            .map(|(key, challenge)| (key, challenge.as_slice()))
            .collect();

        // Create all of the DeviceSigned values in bulk using the keys
        // and challenges, then use these to create the Document values.
        let (device_signeds, keys) = DeviceSigned::new_signatures(keys_and_challenges, key_factory).await?;

        let documents = mdocs
            .into_iter()
            .zip(device_signeds)
            .map(|(mdoc, device_signed)| Document::new(mdoc, device_signed))
            .collect();

        let device_response = Self::new(documents);

        Ok((device_response, keys))
    }

    pub fn matches_request(
        &self,
        credential_request: &NormalizedCredentialRequest,
    ) -> Result<(), ResponseMatchingError> {
        self.matches_requests(std::slice::from_ref(credential_request))
    }

    pub fn matches_requests(
        &self,
        credential_requests: &[NormalizedCredentialRequest],
    ) -> Result<(), ResponseMatchingError> {
        let documents = self.documents.as_deref().unwrap_or_default();

        if documents.len() != credential_requests.len() {
            return Err(ResponseMatchingError::AttestationCountMismatch {
                expected: credential_requests.len(),
                found: documents.len(),
            });
        }

        let missing_attributes = credential_requests
            .iter()
            .zip_eq(documents)
            .map(|(request, document)| {
                let CredentialQueryFormat::MsoMdoc { doctype_value } = &request.format else {
                    return Err(ResponseMatchingError::FormatNotMdoc);
                };

                if document.doc_type != *doctype_value {
                    return Err(ResponseMatchingError::DocTypeMismatch {
                        expected: doctype_value.clone(),
                        found: document.doc_type.clone(),
                    });
                };

                Ok((request, document))
            })
            .process_results(|iter| {
                iter.flat_map(|(request, document)| {
                    match document
                        .issuer_signed
                        .matches_requested_attributes(request.claim_paths())
                    {
                        Ok(()) => None,
                        Err(IssuerSignedMatchingError::MissingAttributes(missing_attributes)) => {
                            Some((document.doc_type.clone(), missing_attributes))
                        }
                    }
                })
                .collect_vec()
            })?;

        if !missing_attributes.is_empty() {
            return Err(ResponseMatchingError::MissingAttributes(missing_attributes));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use futures::FutureExt;
    use itertools::Itertools;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;

    use attestation_types::claim_path::ClaimPath;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use crypto::server_keys::generate::Ca;
    use dcql::normalized;
    use dcql::normalized::NormalizedCredentialRequest;
    use utils::vec_at_least::VecNonEmpty;

    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;
    use crate::examples::Example;
    use crate::holder::Mdoc;
    use crate::iso::disclosure::DeviceAuth;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;

    use super::super::ResponseMatchingError;

    #[test]
    fn test_device_response_sign_from_mdocs() {
        // Generate and sign some mdocs.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (mdocs, keys): (Vec<_>, Vec<_>) = (0..3)
            .map(|index| {
                let key = MockRemoteEcdsaKey::new(format!("key_{index}"), SigningKey::random(&mut OsRng));
                let mdoc = Mdoc::new_mock_with_ca_and_key(&ca, &key).now_or_never().unwrap();

                (mdoc, key)
            })
            .unzip();
        let key_factory = MockRemoteKeyFactory::new(keys);

        // Create a `SessionTranscript`, its contents do not matter.
        let session_transcript = SessionTranscript::new_mock();

        // Sign a `DeviceResponse` that contains the attributes from the generated mdocs.
        let (device_response, _keys) =
            DeviceResponse::sign_from_mdocs(mdocs.clone(), &session_transcript, &key_factory)
                .now_or_never()
                .unwrap()
                .expect("signing DeviceResponse from mdocs should succeed");

        for (document, mdoc) in device_response.documents.as_deref().unwrap_or(&[]).iter().zip(&mdocs) {
            // For each created `Document`, check the contents against the input mdoc.
            assert_eq!(document.doc_type, mdoc.mso.doc_type);
            assert!(document.device_signed.name_spaces.0.is_empty());
            assert_eq!(document.issuer_signed, mdoc.issuer_signed);

            // Re-create the device authentication challenge and validate that
            // each document has a valid device authentication signature.
            let device_auth_bytes =
                DeviceAuthenticationKeyed::challenge(&document.doc_type, &session_transcript).unwrap();

            if let DeviceAuth::DeviceSignature(signature) = &document.device_signed.device_auth {
                signature
                    .clone_with_payload(device_auth_bytes)
                    .verify(&(&mdoc.mso.device_key_info.device_key).try_into().unwrap())
                    .expect("device authentication in DeviceResponse should be valid");
            } else {
                panic!("device authentication in DeviceResponse should be of signature type");
            }
        }
    }

    fn full_example_credential_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
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

    fn empty_device_response() -> DeviceResponse {
        DeviceResponse {
            version: Default::default(),
            documents: None,
            document_errors: None,
            status: 0,
        }
    }

    fn double_full_example_credential_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        vec![
            full_example_credential_request().into_first(),
            full_example_credential_request().into_first(),
        ]
        .try_into()
        .unwrap()
    }

    fn wrong_doc_type_example_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[("wrong_doc_type", &[&[EXAMPLE_NAMESPACE, "family_name"]])])
    }

    fn wrong_name_space_example_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(EXAMPLE_DOC_TYPE, &[&["wrong_name_space", "family_name"]])])
    }

    fn wrong_attributes_example_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_mdoc_from_slices(&[(
            EXAMPLE_DOC_TYPE,
            &[
                &[EXAMPLE_NAMESPACE, "family_name"],
                &[EXAMPLE_NAMESPACE, "favourite_colour"],
                &[EXAMPLE_NAMESPACE, "average_airspeed"],
            ],
        )])
    }

    fn sd_jwt_example_request() -> VecNonEmpty<NormalizedCredentialRequest> {
        normalized::mock::mock_sd_jwt_from_slices(&[(&[EXAMPLE_DOC_TYPE], &[&[EXAMPLE_NAMESPACE, "family_name"]])])
    }

    fn missing_attributes(attributes: &[(&str, &[&[&str]])]) -> Vec<(String, HashSet<VecNonEmpty<ClaimPath>>)> {
        attributes
            .iter()
            .copied()
            .map(|(doc_type, attributes)| {
                let attributes = attributes
                    .iter()
                    .copied()
                    .map(|path| {
                        path.iter()
                            .copied()
                            .map(|path_element| ClaimPath::SelectByKey(path_element.to_string()))
                            .collect_vec()
                            .try_into()
                            .unwrap()
                    })
                    .collect();

                (doc_type.to_string(), attributes)
            })
            .collect()
    }

    #[rstest]
    #[case(DeviceResponse::example(), full_example_credential_request(), Ok(()))]
    #[case(
        empty_device_response(),
        full_example_credential_request(),
        Err(ResponseMatchingError::AttestationCountMismatch {
            expected: 1,
            found: 0,
        }),
    )]
    #[case(
        DeviceResponse::example(),
        double_full_example_credential_request(),
        Err(ResponseMatchingError::AttestationCountMismatch {
            expected: 2,
            found: 1,
        }),
    )]
    #[case(
        DeviceResponse::example(),
        sd_jwt_example_request(),
        Err(ResponseMatchingError::FormatNotMdoc)
    )]
    #[case(
        DeviceResponse::example(),
        wrong_doc_type_example_request(),
        Err(ResponseMatchingError::DocTypeMismatch {
            expected: "wrong_doc_type".to_string(),
            found: EXAMPLE_DOC_TYPE.to_string()
        }),
    )]
    #[case(
        DeviceResponse::example(),
        wrong_name_space_example_request(),
        Err(ResponseMatchingError::MissingAttributes(missing_attributes(
            &[(EXAMPLE_DOC_TYPE, &[&["wrong_name_space", "family_name"]])]
        ))),
    )]
    #[case(
        DeviceResponse::example(),
        wrong_attributes_example_request(),
        Err(ResponseMatchingError::MissingAttributes(missing_attributes(
            &[(
                EXAMPLE_DOC_TYPE,
                &[
                    &[EXAMPLE_NAMESPACE, "average_airspeed"],
                    &[EXAMPLE_NAMESPACE, "favourite_colour"],
                ]
            )]
        ))),
    )]
    fn test_device_response_matches_requests(
        #[case] device_response: DeviceResponse,
        #[case] requests: VecNonEmpty<NormalizedCredentialRequest>,
        #[case] expected_result: Result<(), ResponseMatchingError>,
    ) {
        let result = device_response.matches_requests(requests.as_ref());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_device_response_matches_request() {
        DeviceResponse::example()
            .matches_request(full_example_credential_request().first())
            .expect("credential request should match device response");
    }
}
