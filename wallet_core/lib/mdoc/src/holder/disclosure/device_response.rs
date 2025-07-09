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
    // TODO: Implement test for DeviceResponse::sign_from_mdocs().
}
