//! Data structures used in disclosure for everything that has to be signed with the mdoc's private key.
//! Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
//! of the session so far.
//!
//! NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
//! [`disclosure`](super::disclosure) module (which contains the holder's signature over [`DeviceAuthentication`]
//! defined here).
use std::borrow::Cow;
use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::skip_serializing_none;

use crypto::utils::sha256;
use http_utils::urls::BaseUrl;

use crate::errors::Result;
use crate::iso::disclosure::*;
use crate::utils::cose::CoseKey;
use crate::utils::serialization;
use crate::utils::serialization::cbor_serialize;
use crate::utils::serialization::CborIntMap;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::DeviceAuthenticationString;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;

/// The data structure that the holder signs with the mdoc private key when disclosing attributes out of that mdoc.
/// Contains a.o. transcript of the session so far, acting as the challenge in a challenge-response mechanism,
/// and the "device-signed items" ([`DeviceNameSpaces`]): attributes that are signed only by the device, since they
/// are part of this data structure, but not by the issuer (i.e., self asserted attributes).
///
/// This data structure is computed by the holder and the RP during a session, and then signed and verified
/// respectively. It is not otherwise included in other data structures.
pub type DeviceAuthentication<'a> = CborSeq<DeviceAuthenticationKeyed<'a>>;

/// See [`DeviceAuthentication`].
pub type DeviceAuthenticationBytes<'a> = TaggedBytes<DeviceAuthentication<'a>>;

/// See [`DeviceAuthentication`].
// In production code, this struct is never deserialized.
#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
pub struct DeviceAuthenticationKeyed<'a> {
    pub device_authentication: RequiredValue<DeviceAuthenticationString>,
    pub session_transcript: Cow<'a, SessionTranscript>,
    pub doc_type: Cow<'a, str>,
    pub device_name_spaces_bytes: DeviceNameSpacesBytes,
}

impl<'a> DeviceAuthenticationKeyed<'a> {
    pub fn new(doc_type: &'a str, session_transcript: &'a SessionTranscript) -> Self {
        DeviceAuthenticationKeyed {
            device_authentication: Default::default(),
            session_transcript: Cow::Borrowed(session_transcript),
            doc_type: Cow::Borrowed(doc_type),
            device_name_spaces_bytes: Default::default(),
        }
    }

    pub fn challenge(doc_type: &'a str, session_transcript: &'a SessionTranscript) -> Result<Vec<u8>> {
        let device_auth = Self::new(doc_type, session_transcript);
        let challenge = serialization::cbor_serialize(&TaggedBytes(CborSeq(device_auth)))?;

        Ok(challenge)
    }
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct SessionTranscriptKeyed {
    pub device_engagement_bytes: Option<DeviceEngagementBytes>,
    pub ereader_key_bytes: Option<ESenderKeyBytes>,
    pub handover: Handover,
}

/// Transcript of the session so far. Used in [`DeviceAuthentication`].
pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;

#[derive(Debug, thiserror::Error)]
pub enum SessionTranscriptError {
    #[error("reader engagement is missing security information")]
    MissingReaderEngagementSecurity,
}

impl SessionTranscript {
    pub fn new_qr(
        reader_engagement: &ReaderEngagement,
        device_engagement: &DeviceEngagement,
    ) -> Result<Self, SessionTranscriptError> {
        let reader_security = reader_engagement
            .0
            .security
            .as_ref()
            .ok_or(SessionTranscriptError::MissingReaderEngagementSecurity)?;

        let transcript = SessionTranscriptKeyed {
            device_engagement_bytes: Some(device_engagement.clone().into()),
            handover: Handover::QrHandover,
            ereader_key_bytes: Some(reader_security.0.e_sender_key_bytes.clone()),
        }
        .into();

        Ok(transcript)
    }

    pub fn new_oid4vp(response_uri: &BaseUrl, client_id: &str, nonce: String, mdoc_nonce: &str) -> Self {
        let handover = OID4VPHandover {
            client_id_hash: ByteBuf::from(sha256(&cbor_serialize(&[client_id, mdoc_nonce]).unwrap())),
            response_uri_hash: ByteBuf::from(sha256(
                &cbor_serialize(&[&response_uri.to_string(), mdoc_nonce]).unwrap(),
            )),
            nonce,
        };

        SessionTranscriptKeyed {
            device_engagement_bytes: None,
            ereader_key_bytes: None,
            handover: Handover::Oid4vpHandover(handover.into()),
        }
        .into()
    }
}

pub type DeviceEngagementBytes = TaggedBytes<DeviceEngagement>;

/// Bytes/transcript of the first RP message with which the wallet and RP first established contact.
/// Differs per communication channel.
/// Through the [`SessionTranscript`], this is part of the [`DeviceAuthentication`] so it is signed
/// with each mdoc private key. This message is never sent but instead independently computed by
/// the wallet and RP. If both sides do not agree on this message then mdoc verification fails.
///
/// Serde's `untagged` enum representation ignores the enum variant name, and serializes instead
/// the contained data of the enum variant. It is unfortunately not able to deserialize the `SchemeHandoverBytes`
/// variant, so there is a custom deserializer in `serialization.rs`.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Handover {
    QrHandover,
    NfcHandover(CborSeq<NFCHandover>),
    Oid4vpHandover(CborSeq<OID4VPHandover>),
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct OID4VPHandover {
    /// Must be `SHA256(CBOR_encode([client_id, mdoc_nonce]))`
    pub client_id_hash: ByteBuf,
    /// Must be `SHA256(CBOR_encode([response_uri, mdoc_nonce]))`
    pub response_uri_hash: ByteBuf,
    pub nonce: String,
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct NFCHandover {
    pub handover_select_message: ByteBuf,
    pub handover_request_message: Option<ByteBuf>,
}

/// Describes available methods for the RP to connect to the holder.
pub type DeviceEngagement = CborIntMap<Engagement>;

/// Describes available methods for the holder to connect to the RP.
pub type ReaderEngagement = CborIntMap<Engagement>;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Engagement {
    pub version: EngagementVersion,
    pub security: Option<Security>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EngagementVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

pub type Security = CborSeq<SecurityKeyed>;

/// The ephemeral public key used for establishing an E2E encrypted protocol channel.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityKeyed {
    pub cipher_suite_identifier: CipherSuiteIdentifier,
    pub e_sender_key_bytes: ESenderKeyBytes,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum CipherSuiteIdentifier {
    P256 = 1,
}

pub type ESenderKeyBytes = TaggedBytes<CoseKey>;

#[cfg(any(test, feature = "mock"))]
mod test {
    use super::SessionTranscript;

    impl SessionTranscript {
        pub fn new_mock() -> Self {
            Self::new_oid4vp(
                &"https://example.com".parse().unwrap(),
                "client_id",
                "nonce_1234".to_string(),
                "mdoc_nonce_1234",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::examples::Example;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::utils::serialization;
    use crate::utils::serialization::TaggedBytes;

    use super::*;

    #[test]
    fn test_device_authentication_keyed_new() {
        let TaggedBytes(CborSeq(example_device_auth)) = DeviceAuthenticationBytes::example();
        let session_transcript = example_device_auth.session_transcript.into_owned();
        let device_auth = DeviceAuthenticationKeyed::new(EXAMPLE_DOC_TYPE, &session_transcript);

        assert_eq!(
            serialization::cbor_serialize(&TaggedBytes(CborSeq(device_auth))).unwrap(),
            DeviceAuthenticationBytes::example_bts()
        );
    }
}
