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
    pub e_session_id: SessionId,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.RequestKeyGeneration")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestKeyGenerationMessage {
    pub e_session_id: SessionId,
    pub challenge: ByteBuf,
    pub unsigned_mdocs: Vec<UnsignedMdoc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedMdoc {
    pub doc_type: DocType,
    pub count: u64,
    pub valid_until: Tdate,
    pub attributes: IndexMap<NameSpace, Vec<Entry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.KeyGenerationResponse")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct KeyGenerationResponseMessage {
    pub e_session_id: SessionId,
    pub doc_type_responses: Vec<DocTypeResponses>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocTypeResponses {
    pub doc_type: DocType,
    pub responses: Vec<Response>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub public_key: CoseKey,
    pub signature: MdocCose<CoseSign1, ResponseSignaturePayload>,
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
    pub e_session_id: SessionId,
    #[serde(rename = "mobileIDdocuments")]
    pub mobile_id_documents: Vec<MobileIDDocuments>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileIDDocuments {
    pub doc_type: DocType,
    pub sparse_issuer_signed: Vec<SparseIssuerSigned>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerSigned {
    pub randoms: IndexMap<NameSpace, Vec<ByteBuf>>,
    pub sparse_issuer_auth: SparseIssuerAuth,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerAuth {
    pub version: String,
    pub digest_algorithm: String,
    pub validity_info: ValidityInfo,
    pub issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.rijksoverheid.edi.issuance.DataToIssueResult")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueResultMessage {
    pub e_session_id: SessionId,
    pub result: ByteBuf,
}
