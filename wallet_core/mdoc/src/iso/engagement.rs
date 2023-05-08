//! Data structures used in disclosure for everything that has to be signed with the credential's private key.
//! Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
//! of the session so far.
//!
//! NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
//! [`disclosure`] module (which contains the holder's signature over [`DeviceAuthentication`] defined here).

use ciborium::value::Value;
use fieldnames_derive::FieldNames;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use crate::{
    cose::CoseKey,
    iso::{credentials::*, disclosure::*},
    serialization::{CborIntMap, CborSeq, DeviceAuthenticationString, RequiredValue, TaggedBytes},
};

pub type DeviceAuthentication = CborSeq<DeviceAuthenticationKeyed>;

pub type DeviceAuthenticationBytes = TaggedBytes<DeviceAuthentication>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceAuthenticationKeyed {
    pub device_authentication: RequiredValue<DeviceAuthenticationString>,
    pub session_transcript: SessionTranscript,
    pub doc_type: DocType,
    pub device_name_spaces_bytes: DeviceNameSpacesBytes,
}

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SessionTranscriptKeyed {
    pub device_engagement_bytes: DeviceEngagementBytes,
    pub ereader_key_bytes: EReaderKeyBytes,
    pub handover: Handover,
}

pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;

pub type DeviceEngagementBytes = TaggedBytes<DeviceEngagement>;

#[derive(Debug, Clone)]
pub enum Handover {
    QRHandover,
    NFCHandover(NFCHandover),
}

#[derive(Debug, Clone)]
pub struct NFCHandover {
    pub handover_select_message: ByteBuf,
    pub handover_request_message: Option<ByteBuf>,
}

pub type DeviceEngagement = CborIntMap<DeviceEngagementKeyed>;

// TODO: support remaining fields
#[skip_serializing_none]
#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceEngagementKeyed {
    pub version: String,
    pub security: Security,
    pub device_retrieval_methods: Option<DeviceRetrievalMethods>,
    pub server_retrieval_methods: Option<ServerRetrievalMethods>,
    pub protocol_info: Option<ProtocolInfo>,
}

pub type Security = CborSeq<SecurityKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SecurityKeyed {
    pub cipher_suite_identifier: u64,
    pub e_device_key_bytes: EDeviceKeyBytes,
}

pub type DeviceRetrievalMethods = Vec<DeviceRetrievalMethod>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerRetrievalMethods {
    pub web_api: WebApi,
    pub oidc: Oidc,
}

pub type Oidc = CborSeq<WebSessionInfo>;

pub type WebApi = CborSeq<WebSessionInfo>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct WebSessionInfo {
    pub version: u64,
    pub issuer_url: String,
    pub server_retrieval_token: String,
}

pub type ProtocolInfo = Value;

pub type DeviceRetrievalMethod = CborSeq<DeviceRetrievalMethodKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceRetrievalMethodKeyed {
    pub typ: u64,
    pub version: u64,
    pub retrieval_options: RetrievalOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RetrievalOptions {
    WifiOptions, // TODO
    BleOptions,
    NfcOptions,
}

pub type EReaderKeyBytes = TaggedBytes<EReaderKey>;
pub type EReaderKey = CoseKey;
pub type EDeviceKeyBytes = TaggedBytes<EDeviceKey>;
pub type EDeviceKey = CoseKey;
