use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use itertools::Itertools;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use super::mdoc::PartialMdoc;
use crate::DeviceResponseWithPoa;
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

    async fn sign_from_partial_mdocs_inner<K, W, P>(
        partial_mdocs: &VecNonEmpty<PartialMdoc>,
        session_transcript: &SessionTranscript,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<(Vec<DeviceSigned>, Option<P>)>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K, Poa = P>,
        P: WscdPoa,
    {
        // Prepare the credential keys and device auth challenges per mdoc.
        let (keys, challenges) = partial_mdocs
            .iter()
            .map(|partial_mdoc| {
                let credential_key = partial_mdoc.credential_key(wscd)?;
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
        DeviceSigned::new_signatures(keys_and_challenges, wscd, poa_input).await
    }

    pub async fn sign_multiple_from_partial_mdocs<K, W, P>(
        partial_mdocs: VecNonEmpty<PartialMdoc>,
        session_transcript: &SessionTranscript,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<(VecNonEmpty<Self>, Option<P>)>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K, Poa = P>,
        P: WscdPoa,
    {
        let (device_signeds, poa) =
            Self::sign_from_partial_mdocs_inner(&partial_mdocs, session_transcript, wscd, poa_input).await?;

        let device_responses = partial_mdocs
            .into_iter()
            .zip_eq(device_signeds)
            .map(|(partial_mdoc, device_signed)| Self::new(vec_nonempty![Document::new(partial_mdoc, device_signed)]))
            .collect_vec()
            .try_into()
            // This is safe, as the source iterator is non-empty.
            .unwrap_or_else(|_| unreachable!());

        Ok((device_responses, poa))
    }
}

impl<P> DeviceResponseWithPoa<P> {
    pub fn new(documents: VecNonEmpty<Document>, poa: Option<P>) -> Self {
        DeviceResponseWithPoa {
            device_response: DeviceResponse::new(documents),
            poa,
        }
    }

    pub async fn sign_from_partial_mdocs<K, W>(
        partial_mdocs: VecNonEmpty<PartialMdoc>,
        session_transcript: &SessionTranscript,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<Self>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K, Poa = P>,
        P: WscdPoa,
    {
        let (device_signeds, poa) =
            DeviceResponse::sign_from_partial_mdocs_inner(&partial_mdocs, session_transcript, wscd, poa_input).await?;

        let documents = partial_mdocs
            .into_iter()
            .zip_eq(device_signeds)
            .map(|(partial_mdoc, device_signed)| Document::new(partial_mdoc, device_signed))
            .collect_vec()
            .try_into()
            // This is safe, as the source iterator is non-empty.
            .unwrap();

        Ok(DeviceResponseWithPoa::new(documents, poa))
    }
}

#[cfg(test)]
mod tests {
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use crypto::server_keys::generate::Ca;
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;

    use super::super::mdoc::PartialMdoc;
    use crate::DeviceResponseWithPoa;
    use crate::examples::Example;
    use crate::iso::disclosure::DeviceAuth;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::disclosure::DeviceResponseStatus;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;
    use crate::utils::serialization::cbor_deserialize;
    use crate::utils::serialization::cbor_serialize;

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
        let (partial_mdocs, keys): (Vec<_>, Vec<_>) = (0..3)
            .map(|index| {
                let key = MockRemoteEcdsaKey::new(format!("key_{index}"), SigningKey::random(&mut OsRng));
                let mdoc = PartialMdoc::new_mock_with_ca_and_key(&ca, &key);

                (mdoc, key)
            })
            .unzip();
        let wscd = MockRemoteWscd::new(keys);

        // Create a `SessionTranscript`, its contents do not matter.
        let session_transcript = SessionTranscript::new_mock();

        // Sign multiple `DeviceResponse`s that contain all of the attributes from the generated mdocs.
        let (device_responses, _) = DeviceResponse::sign_multiple_from_partial_mdocs(
            partial_mdocs.clone().try_into().unwrap(),
            &session_transcript,
            &wscd,
            (),
        )
        .now_or_never()
        .unwrap()
        .expect("signing DeviceResponse from mdocs should succeed");

        for (document, partial_mdoc) in device_responses
            .into_iter()
            .flat_map(|device_response| device_response.documents.map(|v| v.into_inner()).unwrap_or_default())
            .zip(&partial_mdocs)
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
                    .verify(partial_mdoc.credential_key(&wscd).unwrap().verifying_key())
                    .expect("device authentication in DeviceResponse should be valid");
            } else {
                panic!("device authentication in DeviceResponse should be of signature type");
            }
        }
    }

    #[rstest]
    #[case(None)]
    #[case(Some("this.isa.poa".to_string()))]
    fn test_device_response_with_poa(#[case] poa: Option<String>) {
        let device_response = DeviceResponse::example();
        let device_response_with_poa = DeviceResponseWithPoa {
            device_response: device_response.clone(),
            poa,
        };

        let serialized = cbor_serialize(&device_response_with_poa).unwrap();
        let deserialized: DeviceResponse = cbor_deserialize(serialized.as_slice()).unwrap();
        assert_eq!(device_response, deserialized);
    }
}
