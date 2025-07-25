use itertools::Itertools;

use crypto::CredentialEcdsaKey;
use dcql::CredentialQueryFormat;
use dcql::normalized::NormalizedCredentialRequest;
use utils::vec_at_least::VecNonEmpty;
use wscd::keyfactory::KeyFactory;

use crate::errors::Error;
use crate::errors::Result;
use crate::identifiers::AttributeIdentifierHolder;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::disclosure::DeviceResponseVersion;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

use super::super::Mdoc;
use super::ResponseValidationError;

impl DeviceResponse {
    pub fn new(documents: Vec<Document>) -> Self {
        Self {
            version: DeviceResponseVersion::default(),
            documents: Some(documents),
            document_errors: None,
            status: 0,
        }
    }

    pub async fn sign_from_mdocs<K, KF, P, PI>(
        mdocs: Vec<Mdoc>,
        session_transcript: &SessionTranscript,
        key_factory: &KF,
        poa_input: PI,
    ) -> Result<(Self, Vec<K>, Option<P>)>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K, Poa = P, PoaInput = PI>,
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
        let (device_signeds, keys, poa) =
            DeviceSigned::new_signatures(keys_and_challenges, key_factory, poa_input).await?;

        let documents = mdocs
            .into_iter()
            .zip(device_signeds)
            .map(|(mdoc, device_signed)| Document::new(mdoc, device_signed))
            .collect();

        let device_response = Self::new(documents);

        Ok((device_response, keys, poa))
    }

    pub fn match_against_request(
        &self,
        credential_requests: &VecNonEmpty<NormalizedCredentialRequest>,
    ) -> Result<(), ResponseValidationError> {
        let not_found = credential_requests
            .as_ref()
            .iter()
            .map(|request| {
                let CredentialQueryFormat::MsoMdoc { ref doctype_value } = request.format else {
                    return Err(ResponseValidationError::ExpectedMdoc);
                };

                let not_found = self
                    .documents
                    .as_ref()
                    .and_then(|docs| docs.iter().find(|doc| doc.doc_type == *doctype_value))
                    .map_or_else(
                        // If the entire document is missing then all requested attributes are missing
                        || Ok(request.mdoc_attribute_identifiers()?.into_iter().collect()),
                        |doc| request.match_against_issuer_signed(doc),
                    )?;
                Ok(not_found)
            })
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(ResponseValidationError::MissingAttributes(not_found))
        }
    }
}

#[cfg(test)]
mod tests {

    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::server_keys::generate::Ca;
    use wscd::keyfactory::JwtPoaInput;
    use wscd::mock_remote::MockRemoteEcdsaKey;
    use wscd::mock_remote::MockRemoteKeyFactory;

    use crate::DeviceAuth;
    use crate::holder::Mdoc;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::engagement::DeviceAuthenticationKeyed;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::ClonePayload;

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
        let (device_response, _keys, _) = DeviceResponse::sign_from_mdocs(
            mdocs.clone(),
            &session_transcript,
            &key_factory,
            JwtPoaInput::new(Some("nonce".to_string()), "aud".to_string()),
        )
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
