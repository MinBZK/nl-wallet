use itertools::Itertools;

use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;

use crate::errors::Error;
use crate::errors::Result;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::disclosure::DeviceResponseVersion;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

use super::mdoc::DisclosureMdoc;

impl DeviceResponse {
    pub fn new(documents: Vec<Document>) -> Self {
        Self {
            version: DeviceResponseVersion::default(),
            documents: Some(documents),
            document_errors: None,
            status: 0,
        }
    }

    pub async fn sign_from_mdocs<K, W, P>(
        disclosure_mdocs: Vec<DisclosureMdoc>,
        session_transcript: &SessionTranscript,
        wscd: &W,
        poa_input: P::Input,
    ) -> Result<(Self, Option<P>)>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K, Poa = P>,
        P: WscdPoa,
    {
        // Prepare the credential keys and device auth challenges per mdoc.
        let (keys, challenges) = disclosure_mdocs
            .iter()
            .map(|disclosure_mdoc| {
                let credential_key = disclosure_mdoc.credential_key(wscd)?;
                let device_signed_challenge =
                    DeviceAuthenticationKeyed::challenge(&disclosure_mdoc.doc_type, session_transcript)?;

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
        let (device_signeds, poa) = DeviceSigned::new_signatures(keys_and_challenges, wscd, poa_input).await?;

        let documents = disclosure_mdocs
            .into_iter()
            .zip(device_signeds)
            .map(|(disclosure_mdoc, device_signed)| Document::new(disclosure_mdoc, device_signed))
            .collect();

        let device_response = Self::new(documents);

        Ok((device_response, poa))
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use crypto::server_keys::generate::Ca;

    use crate::iso::disclosure::DeviceAuth;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;

    use super::super::mdoc::DisclosureMdoc;

    #[test]
    fn test_device_response_sign_from_mdocs() {
        // Generate and sign some mdocs.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (disclosure_mdocs, keys): (Vec<_>, Vec<_>) = (0..3)
            .map(|index| {
                let key = MockRemoteEcdsaKey::new(format!("key_{index}"), SigningKey::random(&mut OsRng));
                let mdoc = DisclosureMdoc::new_mock_with_ca_and_key(&ca, &key);

                (mdoc, key)
            })
            .unzip();
        let wscd = MockRemoteWscd::new(keys);

        // Create a `SessionTranscript`, its contents do not matter.
        let session_transcript = SessionTranscript::new_mock();

        // Sign a `DeviceResponse` that contains all of the attributes from the generated mdocs.
        let (device_response, _) =
            DeviceResponse::sign_from_mdocs(disclosure_mdocs.clone(), &session_transcript, &wscd, ())
                .now_or_never()
                .unwrap()
                .expect("signing DeviceResponse from mdocs should succeed");

        for (document, disclosure_mdoc) in device_response
            .documents
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .zip(&disclosure_mdocs)
        {
            // For each created `Document`, check the contents against the input mdoc.
            assert_eq!(document.doc_type, disclosure_mdoc.doc_type);
            assert!(document.device_signed.name_spaces.0.is_empty());
            assert_eq!(document.issuer_signed, disclosure_mdoc.issuer_signed);

            // Re-create the device authentication challenge and validate that
            // each document has a valid device authentication signature.
            let device_auth_bytes =
                DeviceAuthenticationKeyed::challenge(&document.doc_type, &session_transcript).unwrap();

            if let DeviceAuth::DeviceSignature(signature) = &document.device_signed.device_auth {
                signature
                    .clone_with_payload(device_auth_bytes)
                    .verify(&(&disclosure_mdoc.device_key).try_into().unwrap())
                    .expect("device authentication in DeviceResponse should be valid");
            } else {
                panic!("device authentication in DeviceResponse should be of signature type");
            }
        }
    }
}
