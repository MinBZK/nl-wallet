use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use itertools::Itertools;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use super::mdoc::PartialMdoc;
use crate::errors::Error;
use crate::errors::Result;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::disclosure::DeviceResponseStatus;
use crate::iso::disclosure::DeviceResponseVersion;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

impl DeviceResponse {
    pub fn new(documents: VecNonEmpty<Document>) -> Self {
        Self {
            version: DeviceResponseVersion::default(),
            documents: Some(documents),
            document_errors: None,
            status: DeviceResponseStatus::Ok,
        }
    }

    pub fn error(status: DeviceResponseStatus) -> Self {
        Self {
            version: DeviceResponseVersion::default(),
            documents: None,
            document_errors: None,
            status,
        }
    }

    async fn sign_documents<K, W>(
        partial_mdocs_and_key_ids: VecNonEmpty<(PartialMdoc, String)>,
        session_transcript: &SessionTranscript,
        wscd: &W,
    ) -> Result<VecNonEmpty<Document>>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        // Prepare the credential keys and device auth challenges per mdoc.
        let (keys, challenges) = partial_mdocs_and_key_ids
            .iter()
            .map(|(partial_mdoc, key_identifier)| {
                let credential_key = wscd.new_key(key_identifier, partial_mdoc.public_key()?);
                let device_signed_challenge =
                    DeviceAuthenticationKeyed::challenge(&partial_mdoc.doc_type, session_transcript)?;

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
        let device_signeds = DeviceSigned::new_signatures(keys_and_challenges, wscd).await?;

        let documents = partial_mdocs_and_key_ids
            .into_iter()
            .zip_eq(device_signeds)
            .map(|((partial_mdoc, _), device_signed)| Document::new(partial_mdoc, device_signed))
            .collect_vec()
            .try_into()
            // This is safe, as the source iterator is non-empty.
            .unwrap();

        Ok(documents)
    }

    pub async fn sign_multiple_from_partial_mdocs<K, W>(
        partial_mdocs: VecNonEmpty<(PartialMdoc, String)>,
        session_transcript: &SessionTranscript,
        wscd: &W,
    ) -> Result<VecNonEmpty<Self>>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        let device_responses = Self::sign_documents(partial_mdocs, session_transcript, wscd)
            .await?
            .into_nonempty_iter()
            .map(|document| Self::new(vec_nonempty![document]))
            .collect();

        Ok(device_responses)
    }

    pub async fn sign_from_partial_mdocs<K, W>(
        partial_mdocs: VecNonEmpty<(PartialMdoc, String)>,
        session_transcript: &SessionTranscript,
        wscd: &W,
    ) -> Result<Self>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        let device_response = Self::new(Self::sign_documents(partial_mdocs, session_transcript, wscd).await?);

        Ok(device_response)
    }
}

#[cfg(test)]
mod tests {
    use crypto::mock_remote::DisclosureMockRemoteWscd;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;

    use super::super::mdoc::PartialMdoc;
    use crate::iso::disclosure::DeviceAuth;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::disclosure::DeviceResponseStatus;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;

    #[test]
    fn test_error_device_response_constructor() {
        let device_response = DeviceResponse::error(DeviceResponseStatus::GeneralError);

        assert!(device_response.documents.is_none());
        assert!(device_response.document_errors.is_none());
        assert_eq!(device_response.status, DeviceResponseStatus::GeneralError);
    }

    #[test]
    fn test_device_response_sign_from_mdocs() {
        // Generate and sign some mdocs.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (partial_mdocs_and_key_ids, keys): (Vec<_>, Vec<_>) = (0..3)
            .map(|index| {
                let key = MockRemoteEcdsaKey::new(format!("key_{index}"), SigningKey::generate());
                let mdoc = PartialMdoc::new_mock_with_ca_and_key(&ca, &key);

                ((mdoc, key.identifier.clone()), key)
            })
            .unzip();
        let wscd = DisclosureMockRemoteWscd::new(keys);

        // Create a `SessionTranscript`, its contents do not matter.
        let session_transcript = SessionTranscript::new_mock();

        // Sign multiple `DeviceResponse`s that contain all of the attributes from the generated mdocs.
        let device_responses = DeviceResponse::sign_multiple_from_partial_mdocs(
            partial_mdocs_and_key_ids.clone().try_into().unwrap(),
            &session_transcript,
            &wscd,
        )
        .now_or_never()
        .unwrap()
        .expect("signing DeviceResponse from mdocs should succeed");

        for (document, (partial_mdoc, _)) in device_responses
            .into_iter()
            .flat_map(|device_response| device_response.documents.map(|v| v.into_inner()).unwrap_or_default())
            .zip(&partial_mdocs_and_key_ids)
        {
            // For each created `Document`, check the contents against the input mdoc.
            assert_eq!(document.doc_type, partial_mdoc.doc_type);
            assert!(document.device_signed.name_spaces.0.is_empty());
            assert_eq!(document.issuer_signed, partial_mdoc.issuer_signed);

            // Re-create the device authentication challenge and validate that
            // each document has a valid device authentication signature.
            let device_auth_bytes =
                DeviceAuthenticationKeyed::challenge(&document.doc_type, &session_transcript).unwrap();

            if let DeviceAuth::DeviceSignature(signature) = &document.device_signed.device_auth {
                signature
                    .clone_with_payload(device_auth_bytes)
                    .verify(&partial_mdoc.public_key().unwrap())
                    .expect("device authentication in DeviceResponse should be valid");
            } else {
                panic!("device authentication in DeviceResponse should be of signature type");
            }
        }
    }
}
