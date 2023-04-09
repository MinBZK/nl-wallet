//! Data structures used in disclosure for everything that has to be signed with the credential's private key.
//! Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
//! of the session so far.
//!
//! NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
//! [`disclosure`] module (which contains the holder's signature over [`DeviceAuthentication`] defined here).

use crate::{
    cose::CoseKey,
    iso::{credentials::*, disclosure::*},
    serialization::{CborIntMap, CborSeq, DeviceAuthenticationString, RequiredValue, TaggedBytes},
};

use ciborium::value::Value;
use fieldnames_derive::FieldNames;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::fmt::Debug;

pub type DeviceAuthentication = CborSeq<DeviceAuthenticationKeyed>;

pub type DeviceAuthenticationBytes = TaggedBytes<DeviceAuthentication>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceAuthenticationKeyed {
    pub(crate) device_authentication: RequiredValue<DeviceAuthenticationString>,
    pub(crate) session_transcript: SessionTranscript,
    pub(crate) doc_type: DocType,
    pub(crate) device_name_spaces_bytes: DeviceNameSpacesBytes,
}

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SessionTranscriptKeyed {
    pub(crate) device_engagement_bytes: DeviceEngagementBytes,
    pub(crate) ereader_key_bytes: EReaderKeyBytes,
    pub(crate) handover: Handover,
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
    pub(crate) handover_select_message: ByteBuf,
    pub(crate) handover_request_message: Option<ByteBuf>,
}

pub type DeviceEngagement = CborIntMap<DeviceEngagementKeyed>;

// TODO: support remaining fields
#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceEngagementKeyed {
    pub(crate) version: String,
    pub(crate) security: Security,
    pub(crate) device_retrieval_methods: Option<DeviceRetrievalMethods>,
    pub(crate) server_retrieval_methods: Option<ServerRetrievalMethods>,
    pub(crate) protocol_info: Option<ProtocolInfo>,
}

pub type Security = CborSeq<SecurityKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SecurityKeyed {
    pub(crate) cipher_suite_identifier: u32,
    pub(crate) e_device_key_bytes: EDeviceKeyBytes,
}

pub type DeviceRetrievalMethods = Vec<DeviceRetrievalMethod>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerRetrievalMethods {
    pub(crate) web_api: WebApi,
    pub(crate) oidc: Oidc,
}

pub type Oidc = CborSeq<WebSessionInfo>;

pub type WebApi = CborSeq<WebSessionInfo>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct WebSessionInfo {
    pub(crate) version: u32,
    pub(crate) issuer_url: String,
    pub(crate) server_retrieval_token: String,
}

pub type ProtocolInfo = Value;

pub type DeviceRetrievalMethod = CborSeq<DeviceRetrievalMethodKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceRetrievalMethodKeyed {
    pub(crate) typ: u32,
    pub(crate) version: u32,
    pub(crate) retrieval_options: RetrievalOptions,
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
