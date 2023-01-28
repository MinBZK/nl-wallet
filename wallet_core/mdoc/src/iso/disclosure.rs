//! Data structures used in disclosure, created by the holder and sent to the verifier.
//!
//! The main citizens of this module are [`DeviceResponse`], which is what the holder sends to the verifier during
//! verification, and [`IssuerSigned`], which contains the entire issuer-signed credential and the disclosed attributes.

use crate::{
    cose::MdocCose,
    iso::credentials::*,
    serialization::{NullCborValue, RequiredValue, TaggedBytes},
};

use ciborium::value::Value;
use coset::{CoseMac0, CoseSign1};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceResponse {
    pub(crate) version: String,
    pub(crate) documents: Option<Vec<Document>>,
    #[serde(rename = "documentErrors")]
    pub(crate) document_errors: Option<Vec<DocumentError>>,
    pub(crate) status: u32,
}

pub type DocumentError = IndexMap<DocType, ErrorCode>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    #[serde(rename = "docType")]
    pub(crate) doc_type: DocType,
    #[serde(rename = "issuerSigned")]
    pub(crate) issuer_signed: IssuerSigned,
    #[serde(rename = "deviceSigned")]
    pub(crate) device_signed: DeviceSigned,
    pub(crate) errors: Option<Errors>,
}

/// The issuer-signed MSO in Cose format, as well as some or all of the attributes
/// (i.e. [`IssuerSignedItem`]s) contained in the credential.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerSigned {
    #[serde(rename = "nameSpaces")]
    pub(crate) name_spaces: Option<IssuerNameSpaces>,
    #[serde(rename = "issuerAuth")]
    pub(crate) issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceSigned {
    #[serde(rename = "nameSpaces")]
    pub(crate) name_spaces: DeviceNameSpacesBytes,
    #[serde(rename = "deviceAuth")]
    pub(crate) device_auth: DeviceAuth,
}

pub type DeviceNameSpacesBytes = TaggedBytes<DeviceNameSpaces>;
pub type DeviceNameSpaces = IndexMap<NameSpace, DeviceSignedItems>;
pub type DeviceSignedItems = IndexMap<DataElementIdentifier, Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DeviceAuth {
    #[serde(rename = "deviceSignature")]
    DeviceSignature(MdocCose<CoseSign1, RequiredValue<NullCborValue>>),
    #[serde(rename = "deviceMac")]
    DeviceMac(MdocCose<CoseMac0, RequiredValue<NullCborValue>>),
}

pub type Errors = IndexMap<NameSpace, ErrorItems>;
pub type ErrorItems = IndexMap<DataElementIdentifier, ErrorCode>;
pub type ErrorCode = i32;
