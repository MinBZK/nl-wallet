use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{
    cose::{CoseKey, MdocCose},
    issuance::SessionId,
    serialization::TaggedBytes,
    DocType, MobileSecurityObject, NameSpace, Tdate, ValidityInfo,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.StartIssuing")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct StartIssuingMessage {
    pub(crate) e_session_id: SessionId,
    pub(crate) version: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.RequestKeyGeneration")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestKeyGenerationMessage {
    pub(crate) e_session_id: SessionId,
    pub(crate) challenge: ByteBuf,
    pub(crate) unsigned_mdocs: UnsignedMdocs,
}

pub type UnsignedMdocs = IndexMap<DocType, UnsignedMdoc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedMdoc {
    pub(crate) count: u64,
    pub(crate) valid_until: Tdate,
    pub(crate) attributes: IndexMap<NameSpace, Vec<Entry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub(crate) name: String,
    pub(crate) value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.KeyGenerationResponse")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct KeyGenerationResponseMessage {
    pub(crate) e_session_id: SessionId,
    pub(crate) responses: Responses,
}

pub type MdocPublicKeys = IndexMap<DocType, Vec<CoseKey>>;
pub type Responses = IndexMap<DocType, Vec<Response>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub(crate) public_key: CoseKey,
    pub(crate) signature: MdocCose<CoseSign1, ResponseSignaturePayload>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ResponseSignaturePayload {
    challenge: Vec<u8>,
}

impl ResponseSignaturePayload {
    pub fn new(challenge: Vec<u8>) -> ResponseSignaturePayload {
        ResponseSignaturePayload { challenge }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.DataToIssue")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueMessage {
    pub(crate) e_session_id: SessionId,
    #[serde(rename = "mobileIDdocuments")]
    pub(crate) mobile_id_documents: MobileIDDocuments,
}

pub type MobileIDDocuments = IndexMap<DocType, SparseIssuerSigned>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerSigned {
    pub(crate) randoms: IndexMap<NameSpace, Vec<ByteBuf>>,
    pub(crate) sparse_issuer_auth: SparseIssuerAuth,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerAuth {
    pub(crate) version: String,
    pub(crate) digest_algorithm: String,
    pub(crate) validity_info: ValidityInfo,
    pub(crate) issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.rijksoverheid.edi.issuance.DataToIssueResult")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueResultMessage {
    pub(crate) e_session_id: SessionId,
    pub(crate) result: ByteBuf,
}
