use p256::{PublicKey, SecretKey};
use rand_core::OsRng;
use url::Url;

use crate::{
    errors::Result,
    holder::HolderError,
    iso::engagement::{
        DeviceEngagement, Engagement, EngagementVersion, OriginInfo, OriginInfoDirection, OriginInfoType,
        ReaderEngagement, SessionTranscript,
    },
    utils::crypto::{SessionKey, SessionKeyUser},
    verifier::SessionType,
};

impl ReaderEngagement {
    /// Get the URL for the HTTPS endpoint of the verifier.
    pub fn verifier_url(&self) -> Result<&Url> {
        let verifier_url = self
            .0
            .connection_methods
            .as_ref()
            .and_then(|methods| methods.first())
            .map(|method| &method.0.connection_options.0.uri)
            .ok_or(HolderError::VerifierUrlMissing)?;

        Ok(verifier_url)
    }

    /// Get the public key of the verifier.
    pub fn verifier_public_key(&self) -> Result<PublicKey> {
        let verifier_public_key = self
            .0
            .security
            .as_ref()
            .ok_or(HolderError::VerifierEphemeralKeyMissing)?
            .try_into()?;

        Ok(verifier_public_key)
    }

    /// Calculate the [`SessionTranscript`], the [`SessionKey`] for the reader
    /// (for decrypting the [`DeviceRequest`]) and the [`SessionKey`] for the
    /// device (for encrypting the [`DeviceResponse`]).
    pub fn transcript_and_keys_for_device_engagement(
        &self,
        session_type: SessionType,
        device_engagement: &DeviceEngagement,
        device_private_key: SecretKey,
    ) -> Result<(SessionTranscript, SessionKey, SessionKey)> {
        let verifier_public_key = self.verifier_public_key()?;

        // Create the session transcript so far based on both engagement payloads.
        let session_transcript = SessionTranscript::new_iso(session_type, self, device_engagement)
            .map_err(|_| HolderError::VerifierEphemeralKeyMissing)?;

        // Derive the session key for both directions from the private and public keys and the session transcript.
        let reader_key = SessionKey::new(
            &device_private_key,
            &verifier_public_key,
            &session_transcript,
            SessionKeyUser::Reader,
        )?;
        let device_key = SessionKey::new(
            &device_private_key,
            &verifier_public_key,
            &session_transcript,
            SessionKeyUser::Device,
        )?;

        Ok((session_transcript, reader_key, device_key))
    }
}

impl DeviceEngagement {
    pub fn new_device_engagement(referrer_url: Url) -> Result<(DeviceEngagement, SecretKey)> {
        let privkey = SecretKey::random(&mut OsRng);

        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: None,
            origin_infos: vec![
                OriginInfo {
                    cat: OriginInfoDirection::Received,
                    typ: OriginInfoType::Website(referrer_url),
                },
                OriginInfo {
                    cat: OriginInfoDirection::Delivered,
                    typ: OriginInfoType::MessageData,
                },
            ],
        };

        Ok((engagement.into(), privkey))
    }
}
