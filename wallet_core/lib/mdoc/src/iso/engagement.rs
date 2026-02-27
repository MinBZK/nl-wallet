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
use serde_bytes::Bytes;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::skip_serializing_none;

use crypto::utils::sha256;
use http_utils::urls::BaseUrl;

use crate::errors::Result;
use crate::iso::disclosure::*;
use crate::utils::cose::CoseKey;
use crate::utils::serialization;
use crate::utils::serialization::CborIntMap;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::DeviceAuthenticationString;
use crate::utils::serialization::OpenID4VPHandoverString;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;
use crate::utils::serialization::cbor_serialize;

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
#[derive(Debug, Clone, Serialize)]
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

    pub fn new_oid4vp(client_id: &str, nonce: &str, jwk_thumbprint: Option<&[u8]>, response_uri: &BaseUrl) -> Self {
        let info = OID4VPHandoverInfo {
            client_id: Cow::Borrowed(client_id),
            nonce: Cow::Borrowed(nonce),
            jwk_thumbprint: jwk_thumbprint.map(|jwk| Cow::Borrowed(jwk.into())),
            response_uri: Cow::Borrowed(response_uri.as_ref().as_str()),
        };
        let handover = OID4VPHandover::new(&info);

        let keyed = SessionTranscriptKeyed {
            device_engagement_bytes: None,
            ereader_key_bytes: None,
            handover: Handover::Oid4vpHandover(CborSeq(handover)),
        };

        CborSeq(keyed)
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
#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Handover {
    QrHandover,
    NfcHandover(CborSeq<NFCHandover>),
    Oid4vpHandover(CborSeq<OID4VPHandover>),
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct NFCHandover {
    pub handover_select_message: ByteBuf,
    pub handover_request_message: Option<ByteBuf>,
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct OID4VPHandover {
    pub identifier: RequiredValue<OpenID4VPHandoverString>,
    pub info_hash: ByteBuf,
}

impl OID4VPHandover {
    fn new(info: &OID4VPHandoverInfo) -> Self {
        let info_hash =
            sha256(&cbor_serialize(&CborSeq(&info)).expect("OID4VPHandoverInfo should serialize to CBOR")).into();

        Self {
            identifier: Default::default(),
            info_hash,
        }
    }
}

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct OID4VPHandoverInfo<'a> {
    pub client_id: Cow<'a, str>,
    pub nonce: Cow<'a, str>,
    pub jwk_thumbprint: Option<Cow<'a, Bytes>>,
    pub response_uri: Cow<'a, str>,
}

/// Describes available methods for the RP to connect to the holder.
pub type DeviceEngagement = CborIntMap<Engagement>;

/// Describes available methods for the holder to connect to the RP.
pub type ReaderEngagement = CborIntMap<Engagement>;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engagement {
    pub version: EngagementVersion,
    pub security: Option<Security>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EngagementVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

pub type Security = CborSeq<SecurityKeyed>;

/// The ephemeral public key used for establishing an E2E encrypted protocol channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityKeyed {
    pub cipher_suite_identifier: CipherSuiteIdentifier,
    pub e_sender_key_bytes: ESenderKeyBytes,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CipherSuiteIdentifier {
    P256 = 1,
}

pub type ESenderKeyBytes = TaggedBytes<CoseKey>;

#[cfg(test)]
mod test {
    use super::SessionTranscript;

    impl SessionTranscript {
        pub fn new_mock() -> Self {
            Self::new_oid4vp(
                "client_id",
                "mdoc_nonce_1234",
                None,
                &"https://example.com".parse().unwrap(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::Example;
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

    mod openid4vp {
        use std::sync::LazyLock;

        use base64::prelude::*;
        use hex_literal::hex;
        use jsonwebtoken::jwk::Jwk;
        use jsonwebtoken::jwk::ThumbprintHash;
        use serde_json::json;

        use super::*;

        const EXAMPLE_CLIENT_ID: &str = "x509_san_dns:example.com";
        const EXAMPLE_NONCE: &str = "exc7gBkxjx1rdc9udRrveKvSsJIq80avlXeLHhGwqtA";
        const EXAMPLE_RESPONSE_URI: &str = "https://example.com/response";

        static EXAMPLE_JWK_THUMBPRINT: LazyLock<Vec<u8>> = LazyLock::new(|| {
            // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#appendix-B.2.6.1-7
            let json = json!({
                "kty": "EC",
                "crv": "P-256",
                "x": "DxiH5Q4Yx3UrukE2lWCErq8N8bqC9CHLLrAwLz5BmE0",
                "y": "XtLM4-3h5o3HUH0MHVJV0kyq0iBlrBwlh8qEDMZ4-Pc",
                "use": "enc",
                "alg": "ECDH-ES",
                "kid": "1"
            });

            let jwk = serde_json::from_value::<Jwk>(json).unwrap();
            BASE64_URL_SAFE_NO_PAD
                .decode(jwk.thumbprint(ThumbprintHash::SHA256))
                .unwrap()
        });

        // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#appendix-B.2.6.1-13
        const EXAMPLE_SESSION_TRANSCRIPT_BYTES: [u8; 56] = hex!(
            "83f6f682714f70656e494434565048616e646f7665725820048bc053c00442af9b8e\
             ed494cefdd9d95240d254b046b11b68013722aad38ac"
        );

        #[test]
        fn test_openid4vp_handover_info_serialization() {
            let info = OID4VPHandoverInfo {
                client_id: Cow::Borrowed(EXAMPLE_CLIENT_ID),
                nonce: Cow::Borrowed(EXAMPLE_NONCE),
                jwk_thumbprint: Some(Cow::Borrowed(EXAMPLE_JWK_THUMBPRINT.as_slice().into())),
                response_uri: Cow::Borrowed(EXAMPLE_RESPONSE_URI),
            };

            let bytes = cbor_serialize(&CborSeq(info)).expect("OID4VPHandoverInfo should serialize successfully");

            assert_eq!(bytes, CborSeq::<OID4VPHandoverInfo>::example_bts());
        }

        #[test]
        fn test_session_transcript_new_oid4vp() {
            let session_transcript = SessionTranscript::new_oid4vp(
                EXAMPLE_CLIENT_ID,
                EXAMPLE_NONCE,
                Some(EXAMPLE_JWK_THUMBPRINT.as_slice()),
                &EXAMPLE_RESPONSE_URI.parse().unwrap(),
            );

            let bytes = cbor_serialize(&session_transcript).expect("SessionTranscript should serialize successfully");

            assert_eq!(bytes, EXAMPLE_SESSION_TRANSCRIPT_BYTES);
        }
    }
}
