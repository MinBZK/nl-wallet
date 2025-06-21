use crypto::factory::KeyFactory;
use crypto::CredentialEcdsaKey;
use itertools::Itertools;

use crate::errors::Error;
use crate::errors::Result;
use crate::iso::disclosure::DeviceSigned;
use crate::iso::disclosure::Document;
use crate::iso::engagement::DeviceAuthenticationKeyed;
use crate::iso::engagement::SessionTranscript;

use super::super::Mdoc;

impl Document {
    pub fn new(mdoc: Mdoc, device_signed: DeviceSigned) -> Self {
        Document {
            doc_type: mdoc.mso.doc_type,
            issuer_signed: mdoc.issuer_signed,
            device_signed,
            errors: None,
        }
    }

    pub async fn sign_documents_from_mdocs<K, KF>(
        mdocs: Vec<Mdoc>,
        session_transcript: &SessionTranscript,
        key_factory: &KF,
    ) -> Result<(Vec<Self>, Vec<K>)>
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
            .map(|(mdoc, device_signed)| Self::new(mdoc, device_signed))
            .collect();

        Ok((documents, keys))
    }
}
