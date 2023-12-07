//! Data structures used in disclosure for everything that has to be signed with the mdoc's private key.
//! Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
//! of the session so far.
//!
//! NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
//! [`disclosure`](super::disclosure) module (which contains the holder's signature over [`DeviceAuthentication`]
//! defined here).

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
use url::Url;

use crate::{
    iso::{disclosure::*, mdocs::*},
    utils::{
        cose::CoseKey,
        serialization::{CborIntMap, CborSeq, DeviceAuthenticationString, RequiredValue, TaggedBytes},
    },
    verifier::SessionType,
};

/// The data structure that the holder signs with the mdoc private key when disclosing attributes out of that mdoc.
/// Contains a.o. transcript of the session so far, acting as the challenge in a challenge-response mechanism,
/// and the "device-signed items" ([`DeviceNameSpaces`]): attributes that are signed only by the device, since they
/// are part of this data structure, but not by the issuer (i.e., self asserted attributes).
///
/// This data structure is computed by the holder and the RP during a session, and then signed and verified
/// respectively. It is not otherwise included in other data structures.
pub type DeviceAuthentication = CborSeq<DeviceAuthenticationKeyed>;

impl DeviceAuthentication {
    /// Re-construct a [`DeviceAuthentication`] from a [`SessionTranscript`] and [`DocType`].
    pub fn from_session_transcript(session_transcript: SessionTranscript, doc_type: DocType) -> Self {
        DeviceAuthenticationKeyed {
            device_authentication: Default::default(),
            session_transcript,
            doc_type,
            device_name_spaces_bytes: Default::default(),
        }
        .into()
    }
}

/// See [`DeviceAuthentication`].
pub type DeviceAuthenticationBytes = TaggedBytes<DeviceAuthentication>;

/// See [`DeviceAuthentication`].
// In production code, this struct is never deserialized.
#[cfg_attr(feature = "examples", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
pub struct DeviceAuthenticationKeyed {
    pub device_authentication: RequiredValue<DeviceAuthenticationString>,
    pub session_transcript: SessionTranscript,
    pub doc_type: DocType,
    pub device_name_spaces_bytes: DeviceNameSpacesBytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionTranscriptKeyed {
    pub device_engagement_bytes: DeviceEngagementBytes,
    pub ereader_key_bytes: ESenderKeyBytes,
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
    pub fn new(
        session_type: SessionType,
        reader_engagement: &ReaderEngagement,
        device_engagement: &DeviceEngagement,
    ) -> Result<Self, SessionTranscriptError> {
        let reader_security = reader_engagement
            .0
            .security
            .as_ref()
            .ok_or(SessionTranscriptError::MissingReaderEngagementSecurity)?;

        let transcript = SessionTranscriptKeyed {
            device_engagement_bytes: device_engagement.clone().into(),
            handover: match session_type {
                SessionType::SameDevice => Handover::SchemeHandoverBytes(TaggedBytes(reader_engagement.clone())),
                SessionType::CrossDevice => Handover::QRHandover,
            },
            ereader_key_bytes: reader_security.0.e_sender_key_bytes.clone(),
        }
        .into();

        Ok(transcript)
    }
}

pub type DeviceEngagementBytes = TaggedBytes<DeviceEngagement>;

#[derive(Debug, Clone)]
pub enum Handover {
    QRHandover,
    NFCHandover(NFCHandover),
    SchemeHandoverBytes(TaggedBytes<ReaderEngagement>),
}

#[derive(Debug, Clone)]
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
    pub connection_methods: Option<ConnectionMethods>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub origin_infos: Vec<OriginInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EngagementVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

/// Describes the kind and direction of the previously received protocol message.
/// Part of the [`DeviceAuthenticationBytes`] which are signed with the mdoc private key during disclosure.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OriginInfo {
    pub cat: OriginInfoDirection,
    #[serde(flatten)]
    pub typ: OriginInfoType,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum OriginInfoDirection {
    Delivered = 0,
    Received = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginInfoType {
    Website(Url),
    OnDeviceQRCode,
    MessageData,
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

/// Describes the available connection methods. Called DeviceRetrievalMethods in ISO 18013-5
pub type ConnectionMethods = Vec<ConnectionMethod>;

/// Describes an available connection method. Called DeviceRetrievalMethod in ISO 18013-5
pub type ConnectionMethod = CborSeq<ConnectionMethodKeyed>;

/// Describes an available connection method.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionMethodKeyed {
    pub typ: ConnectionMethodType,
    pub version: ConnectionMethodVersion,
    pub connection_options: CborSeq<RestApiOptionsKeyed>,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum ConnectionMethodType {
    RestApi = 4,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum ConnectionMethodVersion {
    RestApi = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestApiOptionsKeyed {
    pub uri: Url,
}

pub type ESenderKeyBytes = TaggedBytes<CoseKey>;

#[cfg(test)]
mod tests {
    use crate::{
        examples::Example,
        utils::serialization::{self, TaggedBytes},
    };

    use super::*;

    #[test]
    fn test_device_authentication_bytes_from_session_transcript() {
        let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
        let device_authentication =
            DeviceAuthentication::from_session_transcript(session_transcript, "org.iso.18013.5.1.mDL".to_string());

        assert_eq!(
            serialization::cbor_serialize(&TaggedBytes(device_authentication)).unwrap(),
            DeviceAuthenticationBytes::example_bts()
        );
    }
}
