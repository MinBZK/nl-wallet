//! Data structures used in disclosure, created by the holder and sent to the RP.
//!
//! The main citizens of this module are [`DeviceResponse`], which is what the holder sends to the verifier during
//! verification, and [`IssuerSigned`], which contains the entire issuer-signed mdoc and the disclosed attributes.

use coset::CoseMac0;
use coset::CoseSign1;
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use attestation::identifiers::AttributeIdentifier;

use crate::iso::mdocs::*;
use crate::unsigned::Entry;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::NullCborValue;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;

/// A disclosure of a holder, containing multiple [`Document`]s, containing some or all of their attributes.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceResponse {
    pub version: DeviceResponseVersion,
    pub documents: Option<Vec<Document>>,
    pub document_errors: Option<Vec<DocumentError>>,
    pub status: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DeviceResponseVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

pub type DocumentError = IndexMap<DocType, ErrorCode>;

/// A disclosed mdoc, containing:
/// - the MSO signed by the issuer including the mdoc's public key and the digests of the attributes,
/// - the values and `random` bytes of the disclosed (i.e. included) attributes,
/// - the holder signature (over the session transcript so far, which is not included here; see
///   [`DeviceAuthentication`](super::DeviceAuthentication)), using the private key corresponding to the public key
///   contained in the mdoc; this acts as challenge-response mechanism.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub doc_type: DocType,
    pub issuer_signed: IssuerSigned,
    pub device_signed: DeviceSigned,
    pub errors: Option<Errors>,
}

impl Document {
    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(&self.doc_type)
    }
}

/// The issuer-signed MSO in Cose format, as well as some or all of the attributes including their randoms
/// (i.e. [`IssuerSignedItem`]s) contained in the mdoc. This includes the public key of the MSO,
/// but not the private key (for that, see [`Mdoc`](crate::holder::Mdoc)).
///
/// This data structure is used as part of mdocs (in which case `name_spaces` necessarily contains all attributes
/// of the mdoc), and also as part of a disclosure of the mdoc in the [`Document`] struct (in which some
/// attributes may be absent, i.e., not disclosed).
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssuerSigned {
    pub name_spaces: Option<IssuerNameSpaces>,
    pub issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
}

impl IssuerSigned {
    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    pub fn into_entries_by_namespace(self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.name_spaces
            .map(|name_spaces| {
                name_spaces
                    .into_iter()
                    .map(|(name_space, attributes)| (name_space, attributes.into()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn attribute_identifiers(&self, doc_type: &str) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .as_ref()
                    .iter()
                    .flat_map(|(namespace, attributes)| {
                        attributes
                            .as_ref()
                            .iter()
                            .map(|TaggedBytes(attribute)| AttributeIdentifier {
                                credential_type: doc_type.to_owned(),
                                namespace: namespace.to_owned(),
                                attribute: attribute.element_identifier.to_owned(),
                            })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The holder signature as during disclosure of an mdoc (see [`Document`]) computed with the mdoc private key, as well
/// as any self-asserted attributes.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSigned {
    pub name_spaces: DeviceNameSpacesBytes,
    pub device_auth: DeviceAuth,
}

/// Attributes included in a holder disclosure that have not been signed by the issuer, but only
/// by the holder: self-asserted attributes. See also [`DeviceSigned`] and
/// [`DeviceAuthentication`](super::DeviceAuthentication).
pub type DeviceNameSpaces = IndexMap<NameSpace, DeviceSignedItems>;

/// See [`DeviceNameSpaces`].
pub type DeviceNameSpacesBytes = TaggedBytes<DeviceNameSpaces>;

/// Self-asserted attributes as part of an mdoc disclosure.
pub type DeviceSignedItems = IndexMap<DataElementIdentifier, DataElementValue>;

/// The signature or MAC created by the holder during disclosure of an mdoc, with the private key of the mdoc
/// (whose public key is included in its MSO).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DeviceAuth {
    DeviceSignature(MdocCose<CoseSign1, RequiredValue<NullCborValue>>),
    DeviceMac(MdocCose<CoseMac0, RequiredValue<NullCborValue>>),
}

pub type Errors = IndexMap<NameSpace, ErrorItems>;
pub type ErrorItems = IndexMap<DataElementIdentifier, ErrorCode>;
pub type ErrorCode = i32;

/// Contains an encrypted mdoc disclosure protocol message, and a status code containing an error code or a code
/// that aborts the session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionData {
    pub data: Option<ByteBuf>,
    pub status: Option<SessionStatus>,
}

/// Status codes sent along with encrypted mdoc disclosure protocol messages in [`StatusCode`].
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SessionStatus {
    EncryptionError = 10,
    DecodingError = 11,
    Termination = 20,
}

impl SessionData {
    pub fn new_encryption_error() -> Self {
        SessionData {
            data: None,
            status: Some(SessionStatus::EncryptionError),
        }
    }

    pub fn new_decoding_error() -> Self {
        SessionData {
            data: None,
            status: Some(SessionStatus::DecodingError),
        }
    }

    pub fn new_termination() -> Self {
        SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        }
    }
}
