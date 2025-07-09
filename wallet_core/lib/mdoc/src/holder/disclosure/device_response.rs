use crypto::factory::KeyFactory;
use crypto::CredentialEcdsaKey;
use itertools::Itertools;

use crate::errors::Error;
use crate::errors::Result;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::disclosure::DeviceResponseVersion;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

use super::super::Mdoc;

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
}

#[cfg(test)]
mod tests {

    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use crypto::server_keys::generate::Ca;

    use crate::holder::Mdoc;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;
    use crate::DeviceAuth;

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
}
