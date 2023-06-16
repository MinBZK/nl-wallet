//! Data structures with which a verifier requests attributes from a holder.

use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use crate::{
    iso::{credentials::*, engagement::*},
    utils::{
        cose::MdocCose,
        serialization::{CborSeq, ReaderAuthenticationString, RequiredValue, TaggedBytes},
    },
};
use fieldnames_derive::FieldNames;

/// Sent by the RP to the holder to request the disclosure of attributes out of one or more mdocs.
/// For each mdoc out of which attributes are requested, a [`DocRequest`] is included.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRequest {
    pub version: String,
    pub doc_requests: Vec<DocRequest>,
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

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DeviceRequest`].
/// Includes reader (RP) authentication.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocRequest {
    pub items_request: ItemsRequestBytes,
    pub reader_auth: Option<ReaderAuth>,
}

pub type ReaderAuth = MdocCose<CoseSign1, Value>;
pub type ReaderAuthenticationBytes = TaggedBytes<ReaderAuthentication>;
pub type ReaderAuthentication = CborSeq<ReaderAuthenticationKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct ReaderAuthenticationKeyed {
    pub reader_auth_string: RequiredValue<ReaderAuthenticationString>,
    pub session_transcript: SessionTranscript,
    pub items_request_bytes: ItemsRequestBytes,
}

/// See [`ItemsRequest`].
pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DocRequest`] in a
/// [`DeviceRequest`].
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemsRequest {
    pub doc_type: DocType,
    pub name_spaces: NameSpaces,

    /// Free-form additional information.
    pub request_info: Option<IndexMap<String, Value>>,
}

/// The attribute names that the RP wishes disclosed, grouped per namespace, as part of a [`ItemsRequest`].
pub type NameSpaces = IndexMap<NameSpace, DataElements>;

/// The attribute names that the RP wishes disclosed within a particular namespace, as part of a [`ItemsRequest`],
/// along with a boolean with which the RP can claim its intention to (not) retain the attribute value after receiving
/// and verifying it.
pub type DataElements = IndexMap<DataElementIdentifier, IndentToRetain>;

///  Claimed intention of the RP to (not) retain the attribute value after receiving and verifying it, as part of
/// [`DataElements`] within a [`ItemsRequest`].
pub type IndentToRetain = bool;
