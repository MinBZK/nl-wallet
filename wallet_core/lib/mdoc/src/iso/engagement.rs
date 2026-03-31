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
use mdoc_derive::CborIndexedFields;
use utils::vec_at_least::VecNonEmpty;

use crate::errors::Result;
use crate::iso::disclosure::*;
use crate::utils::cose::CoseKey;
use crate::utils::serialization;
use crate::utils::serialization::CborError;
use crate::utils::serialization::CborIndexedFields;
use crate::utils::serialization::CborIntMap;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::DeviceAuthenticationString;
use crate::utils::serialization::OpenID4VPHandoverString;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;
use crate::utils::serialization::cbor_deserialize;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscriptKeyed {
    pub device_engagement_bytes: Option<DeviceEngagementBytes>,
    pub e_reader_key_bytes: Option<EReaderKeyBytes>,
    pub handover: Handover,
}

/// Transcript of the session so far. Used in [`DeviceAuthentication`].
pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;

impl SessionTranscript {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, CborError> {
        cbor_deserialize(bytes)
    }

    pub fn new_qr(e_reader_key: impl Into<CoseKey>, device_engagement: Option<DeviceEngagement>) -> Self {
        let cose_key: CoseKey = e_reader_key.into();
        SessionTranscriptKeyed {
            device_engagement_bytes: device_engagement.map(Into::into),
            e_reader_key_bytes: Some(cose_key.into()),
            handover: Handover::QrHandover,
        }
        .into()
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
            e_reader_key_bytes: None,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Handover {
    QrHandover,
    NfcHandover(CborSeq<NFCHandover>),
    Oid4vpHandover(CborSeq<OID4VPHandover>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFCHandover {
    pub handover_select_message: ByteBuf,
    pub handover_request_message: Option<ByteBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, CborIndexedFields)]
pub struct Engagement {
    /// 0: tstr, ; Version
    pub version: EngagementVersion,
    /// 1: Security,
    pub security: Security,
    /// ? 2: DeviceRetrievalMethods, ; Is absent if NFC is used for device engagement
    pub device_retrieval_methods: Option<VecNonEmpty<DeviceRetrievalMethod>>,
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
    pub e_device_key_bytes: EDeviceKeyBytes,
}

pub type DeviceRetrievalMethod = CborSeq<DeviceRetrievalMethodKeyed>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRetrievalMethodKeyed {
    pub r#type: u64,
    pub version: u64,
    pub retrieval_options: RetrievalOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RetrievalOptions {
    Wifi(CborIntMap<WifiOptions>),
    Ble(CborIntMap<BleOptions>),
    Nfc(CborIntMap<NfcOptions>),
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, CborIndexedFields)]
pub struct WifiOptions {
    pub pass_phrase: Option<String>,
    pub operating_class: Option<u64>,
    pub channel_number: Option<u64>,
    pub supported_bands: Option<ByteBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CborIndexedFields)]
pub struct BleOptions {
    pub peripheral_server_mode: bool,
    pub central_client_mode: bool,
    #[cbor_index = 10]
    pub peripheral_server_uuid: Option<ByteBuf>,
    pub central_client_uuid: Option<ByteBuf>,
    #[cbor_index = 20]
    pub peripheral_server_address: Option<ByteBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CborIndexedFields)]
pub struct NfcOptions {
    pub command_max_len: u64,
    pub response_max_len: u64,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CipherSuiteIdentifier {
    P256 = 1,
}

pub type EDeviceKeyBytes = TaggedBytes<CoseKey>;
pub type EReaderKeyBytes = TaggedBytes<CoseKey>;

#[cfg(any(test, feature = "mock"))]
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

    #[test]
    fn test_deserialize_session_transcript() {
        let example_session_transcript = hex::decode(
            "83d8185874a30063312e30018201d818584ba401022001215820423222782a6b167018f903e1972ec8f42f8e2810efe33f2568cb3e\
             935ed4eec02258202fdeabd892c74ed215ea9d9fbd294a2e8de53d05b2154b4e6d9484ae6a2b7f7a0281830201a300f501f40a5088\
             686e34eba74f3dacdaeadfff9b2cb2d818584ba40102200121582060e3392385041f51403051f2415531cb56dd3f999c71687013aa\
             c6768bc8187e225820e58deb8fdbe907f7dd5368245551a34796f7d2215c440c339bb0f7b67beccdfaf6",
        )
        .unwrap();
        let session_transcript = SessionTranscript::try_from_bytes(&example_session_transcript).unwrap();

        assert_eq!(
            serialization::cbor_serialize(&session_transcript).unwrap(),
            example_session_transcript
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
