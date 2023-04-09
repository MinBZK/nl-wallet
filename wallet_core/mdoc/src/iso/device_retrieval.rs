//! Data structures with which a verifier requests attributes from a holder.

use crate::{
    cose::MdocCose,
    iso::{credentials::*, device_authentication::*},
    serialization::{CborSeq, ReaderAuthenticationString, RequiredValue, TaggedBytes},
};
use fieldnames_derive::FieldNames;

use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRequest {
    pub(crate) version: String,
    pub(crate) doc_requests: Vec<DocRequest>,
}
impl DeviceRequest {
    pub fn new(items_requests: Vec<ItemsRequest>) -> DeviceRequest {
        DeviceRequest {
            version: "1.0".to_string(),
            doc_requests: items_requests
                .into_iter()
                .map(|items_request| DocRequest {
                    items_request: items_request.into(),
                    reader_auth: None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocRequest {
    pub(crate) items_request: ItemsRequestBytes,
    pub(crate) reader_auth: Option<ReaderAuth>,
}

pub type ReaderAuth = MdocCose<CoseSign1, Value>;
pub type ReaderAuthenticationBytes = TaggedBytes<ReaderAuthentication>;
pub type ReaderAuthentication = CborSeq<ReaderAuthenticationKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct ReaderAuthenticationKeyed {
    pub(crate) reader_auth_string: RequiredValue<ReaderAuthenticationString>,
    pub(crate) session_transcript: SessionTranscript,
    pub(crate) items_request_bytes: ItemsRequestBytes,
}

pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemsRequest {
    pub(crate) doc_type: DocType,
    pub(crate) name_spaces: NameSpaces,
    pub(crate) request_info: Option<IndexMap<String, Value>>,
}

pub type NameSpaces = IndexMap<NameSpace, DataElements>;
pub type DataElements = IndexMap<DataElementIdentifier, IndentToRetain>;
pub type IndentToRetain = bool;
