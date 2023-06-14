//! An ISO 23220-3 application specific issuance protocol, extending BasicSA.
//!
//! Where sensible, this protocol uses the same names and concepts for the messages and datastructures as BasicSA.
//! This protocol differs from BasicSA in the following aspects.
//! - It supports issuance of multiple copies of an mdoc, that are identical in terms of the present attributes and
//!   their values but having differing (but valid) issuer signatures and attribute randoms. This allows the holder
//!   to not reuse an mdoc after it has been used, preventing the mdoc itself from becoming a stable identifier for the
//!   holder.
//! - It additionally supports issuance of multiple distinct mdocs within a single session.
//! - During the protocol, the issuer informs the holder of the mdocs that it is going to receive in the remainder of
//!   the session. This allows the holder to abort early if they decide they do not want the mdocs. See [`UnsignedMdoc`]
//!   as part of [`RequestKeyGenerationMessage`].
//! - At the end of the session, the issuer does not send complete MSO COSE's to the holder. Instead, for each issued
//!   mdoc it sends "sparse" MSO COSE's, that only contain those values the holder has not already learned earlier in
//!   the session.

use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{
    cose::{CoseKey, MdocCose},
    issuance::SessionId,
    serialization::TaggedBytes,
    DataElementIdentifier, DataElementValue, DocType, MobileSecurityObject, NameSpace, Tdate, ValidityInfo,
};

pub const START_ISSUING_MSG_TYPE: &str = "nl.referencewallet.issuance.StartIssuing";

/// Holder -> issuer. Request the [`RequestKeyGenerationMessage`] from the issuer.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.StartIssuing")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct StartIssuingMessage {
    pub e_session_id: SessionId,
    pub version: u64,
}

/// Issuer -> holder, in reply to [`StartIssuingMessage`]. Contains the mdocs that will be issued in this session
/// (containing all their attributes but not yet an issuer signature), and the challenge that the holder must sign
/// with the private keys whose public keys are to be included in the mdocs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.RequestKeyGeneration")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct RequestKeyGenerationMessage {
    pub e_session_id: SessionId,
    pub challenge: ByteBuf,
    pub unsigned_mdocs: Vec<UnsignedMdoc>,
}

/// A not-yet-signed mdoc, presented by the issuer to the holder during issuance, so that the holder can agree
/// or disagree to receive the signed mdoc in the rest of the protocol.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedMdoc {
    pub doc_type: DocType,
    pub count: u64,
    pub valid_until: Tdate,
    pub attributes: IndexMap<NameSpace, Vec<Entry>>,
}

/// An attribute name and value.
///
/// See also [`super::IssuerSignedItem`], which additionally contains the attribute's `random` and `digestID`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub name: DataElementIdentifier,
    pub value: DataElementValue,
}

pub const KEY_GEN_RESP_MSG_TYPE: &str = "nl.referencewallet.issuance.KeyGenerationResponse";

/// Holder -> issuer, in reply to [`RequestKeyGenerationMessage`]. Contains the responses, i.e., the challenge from the
/// [`RequestKeyGenerationMessage`] signed with the private keys whose public keys are to be included in the mdocs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.KeyGenerationResponse")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct KeyGenerationResponseMessage {
    pub e_session_id: SessionId,
    pub mdoc_responses: Vec<MdocResponses>,
}

/// Responses for a specific mdoc during issuance. Contains one response for each copy of the mdoc that is being issued.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MdocResponses {
    pub doc_type: DocType,
    pub responses: Vec<Response>,
}

/// Response for one copy of one of the mdocs being issued. Includes the public key to be included in the mdoc copy,
/// as well as the signature over the challenge, which should verify against that public key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub public_key: CoseKey,
    pub signature: MdocCose<CoseSign1, ResponseSignaturePayload>,
}

/// To be signed by the holder during issuance using the mdoc private key.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ResponseSignaturePayload {
    challenge: ByteBuf,
}

impl ResponseSignaturePayload {
    pub fn new(challenge: Vec<u8>) -> ResponseSignaturePayload {
        ResponseSignaturePayload {
            challenge: ByteBuf::from(challenge),
        }
    }
}

/// Issuer -> holder, in reply to [`KeyGenerationResponseMessage`]. Contains all data of the signed mdocs being issued
/// (in particular the issuer signatures) that the holder has not yet already learned during the protocol so far.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nl.referencewallet.issuance.DataToIssue")]
#[serde(tag = "messageType")]
#[serde(rename_all = "camelCase")]
pub struct DataToIssueMessage {
    pub e_session_id: SessionId,
    #[serde(rename = "mobileIDdocuments")]
    pub mobile_id_documents: Vec<MobileIDDocuments>,
}

/// All data of the signed mdocs being issued (in particular the issuer signatures) that the holder has not yet already
/// learned during the protocol so far.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileIDDocuments {
    pub doc_type: DocType,
    pub sparse_issuer_signed: Vec<SparseIssuerSigned>,
}

/// All data of a signed mdoc being issued (in particular the issuer signatures). Like an [`super::IssuerSigned`],
/// excluding the data that the holder has already learned during the protocol so far.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerSigned {
    pub randoms: IndexMap<NameSpace, Vec<ByteBuf>>,
    pub sparse_issuer_auth: SparseIssuerAuth,
}

/// Issuer signed data of an mdoc being issued. Like the `issuer_auth` field of an [`super::IssuerSigned`], excluding
/// the data that the holder has already learned during the protocol so far.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SparseIssuerAuth {
    pub version: String,
    pub digest_algorithm: String,
    pub validity_info: ValidityInfo,
    pub issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}
