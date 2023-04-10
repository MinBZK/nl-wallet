use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{
    cose::{CoseKey, MdocCose},
    issuance::SessionId,
    serialization::TaggedBytes,
    DocType, MobileSecurityObject, NameSpace, ValidityInfo,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.StartIssuing")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct StartIssuingMessage {
    e_session_id: SessionId,
    version: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.RequestKeyGeneration")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestKeyGenerationMessage {
    e_session_id: SessionId,
    challenge: ByteBuf,
    to_be_issued: UnsignedMdocs,
}

pub type UnsignedMdocs = IndexMap<DocType, UnsignedMdoc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsignedMdoc {
    attributes: IndexMap<NameSpace, Entry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    name: String,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.KeyGenerationResponse")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct KeyGenerationResponseMessage {
    e_session_id: SessionId,
    mdoc_public_keys: MdocPublicKeys,
    responses: Responses,
}

pub type MdocPublicKeys = IndexMap<DocType, Vec<CoseKey>>;
pub type Responses = IndexMap<DocType, Vec<MdocCose<CoseSign1, ByteBuf>>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.DataToIssue")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueMessage {
    e_session_id: SessionId,
    #[serde(rename = "mobileIDdocuments")]
    mobiel_id_documents: MobileIDDocuments,
}

pub type MobileIDDocuments = IndexMap<DocType, IssuanceIssuerSigned>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuanceIssuerSigned {
    randoms: IndexMap<NameSpace, Vec<ByteBuf>>,
    issuance_issuer_auth: IssuanceIssuerAuth,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuanceIssuerAuth {
    version: String,
    digest_algorithm: String,
    validity_info: ValidityInfo,
    issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.rijksoverheid.edi.issuance.DataToIssueResult")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueResultMessage {
    e_session_id: SessionId,
    result: ByteBuf,
}
