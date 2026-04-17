//! Data structures used in disclosure, created by the holder and sent to the RP.
//!
//! The main citizens of this module are [`DeviceResponse`], which is what the holder sends to the verifier during
//! verification, and [`IssuerSigned`], which contains the entire issuer-signed mdoc and the disclosed attributes.
use std::fmt::Debug;

use coset::CoseMac0;
use coset::CoseSign1;
use indexmap::IndexMap;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::skip_serializing_none;

use utils::vec_at_least::VecNonEmpty;

use crate::iso::mdocs::*;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::NullCborValue;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;

/// A disclosure of a holder, containing multiple [`Document`]s, containing some or all of their attributes.
///
/// ```cddl
/// DeviceResponse = {
///     "version" : tstr,
///     ? "documents" : [+Document],
///     ? "documentErrors": [+DocumentError],
///     "status" : uint
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(PartialEq))]
pub struct DeviceResponse {
    /// Version of the DeviceResponse structure
    pub version: DeviceResponseVersion,

    /// Returned documents
    pub documents: Option<VecNonEmpty<Document>>,

    /// For unreturned documents, optional error codes
    pub document_errors: Option<VecNonEmpty<DocumentError>>,

    /// Status code
    pub status: DeviceResponseStatus,
}

/// A [`DeviceResponse`] with a POA attached.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceResponseWithPoa<P> {
    #[serde(flatten)]
    pub device_response: DeviceResponse,

    /// Optional because it is not in the spec.
    pub poa: Option<P>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u64)]
pub enum DeviceResponseStatus {
    Ok = 0,
    GeneralError = 10,
    CborDecodingError = 11,
    InvalidRequest = 12,
}

/// Version of the [`DeviceResponse`] structure
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceResponseVersion {
    #[default]
    #[serde(rename = "1.0")]
    V1_0,
}

/// Error codes for unreturned documents
///
/// ```cddl
/// DocumentError = {
///     DocType => ErrorCode
/// }
/// ```
pub type DocumentError = IndexMap<DocType, ErrorCode>;

/// A disclosed mdoc, containing:
/// - the MSO signed by the issuer including the mdoc's public key and the digests of the attributes,
/// - the values and `random` bytes of the disclosed (i.e. included) attributes,
/// - the holder signature (over the session transcript so far, which is not included here; see
///   [`DeviceAuthentication`](super::DeviceAuthentication)), using the private key corresponding to the public key
///   contained in the mdoc; this acts as challenge-response mechanism.
///
/// ```cddl
/// Document = {
///     "docType" : DocType,
///     "issuerSigned" : IssuerSigned,
///     "deviceSigned" : DeviceSigned,
///     ? "errors" : Errors
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(PartialEq))]
pub struct Document {
    /// Document type returned
    pub doc_type: DocType,
    /// Returned data elements signed by the issuer
    pub issuer_signed: IssuerSigned,
    /// Returned data elements signed by the mdoc
    pub device_signed: DeviceSigned,
    pub errors: Option<Errors>,
}

/// The issuer-signed MSO in Cose format, as well as some or all of the attributes including their randoms
/// (i.e. [`IssuerSignedItem`]s) contained in the mdoc. This includes the public key of the MSO,
/// but not the private key (for that, see [`Mdoc`](crate::holder::Mdoc)).
///
/// This data structure is used as part of mdocs (in which case `name_spaces` necessarily contains all attributes
/// of the mdoc), and also as part of a disclosure of the mdoc in the [`Document`] struct (in which some
/// attributes may be absent, i.e., not disclosed).
///
/// ```cddl
/// IssuerSigned = {
///     ? "nameSpaces" : IssuerNameSpaces,
///     "issuerAuth" : IssuerAuth
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuerSigned {
    /// Returned data elements
    pub name_spaces: Option<IssuerNameSpaces>,

    /// Contains the mobile security object (MSO) for issuer data authentication
    pub issuer_auth: IssuerAuth,
}

/// Contains the mobile security object (MSO) for issuer data authentication
///
/// ```cddl
/// IssuerAuth = COSE_Sign1
/// ```
pub type IssuerAuth = MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>;

impl IssuerSigned {
    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    /// Note that this should only be called after the [`IssuerSigned`] has been validated.
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
}

/// The holder signature as during disclosure of an mdoc (see [`Document`]) computed with the mdoc private key, as well
/// as any self-asserted attributes.
///
/// ```cddl
/// DeviceSigned = {
///     "nameSpaces" : DeviceNameSpacesBytes,
///     "deviceAuth" : DeviceAuth
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(PartialEq))]
pub struct DeviceSigned {
    /// Returned data elements
    pub name_spaces: DeviceNameSpacesBytes,

    /// Contains the device authentication for mdoc authentication
    pub device_auth: DeviceAuth,
}

/// Returned data elements for each namespace
///
/// Attributes included in a holder disclosure that have not been signed by the issuer, but only
/// by the holder: self-asserted attributes. See also [`DeviceSigned`] and
/// [`DeviceAuthentication`](super::DeviceAuthentication).
///
/// ```cddl
/// DeviceNameSpaces = {
///     * NameSpace => DeviceSignedItems
/// }
/// ```
pub type DeviceNameSpaces = IndexMap<NameSpace, DeviceSignedItems>;

/// See [`DeviceNameSpaces`].
///
/// ```cddl
/// DeviceNameSpacesBytes = #6.24(bstr .cbor DeviceNameSpaces)
/// ```
pub type DeviceNameSpacesBytes = TaggedBytes<DeviceNameSpaces>;

/// Returned data element identifier and value. Self-asserted attributes as part of an mdoc disclosure.
///
/// ```cddl
/// DeviceSignedItems = {
///     + DataElementIdentifier => DataElementValue
/// }
/// ```
#[nutype(
    derive(Debug, Clone, PartialEq, Serialize, Deserialize),
    validate(predicate = |items| !items.is_empty()),
)]
pub struct DeviceSignedItems(IndexMap<DataElementIdentifier, DataElementValue>);

/// The signature or MAC created by the holder during disclosure of an mdoc, with the private key of the mdoc
/// (whose public key is included in its MSO).
///
/// Either signature or MAC for mdoc authentication
/// ```cddl
/// DeviceAuth = {
///     "deviceSignature" : DeviceSignature //
///     "deviceMac" : DeviceMac
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(PartialEq))]
pub enum DeviceAuth {
    DeviceSignature(MdocCose<CoseSign1, RequiredValue<NullCborValue>>),
    DeviceMac(MdocCose<CoseMac0, RequiredValue<NullCborValue>>),
}

/// Error codes for each namespace
///
/// ```cddl
/// Errors = {
///     + NameSpace => ErrorItems
/// }
/// ```
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize),
    validate(predicate = |errors| !errors.is_empty()),
)]
pub struct Errors(IndexMap<NameSpace, ErrorItems>);

/// Error code per data element
///
/// ```cddl
/// ErrorItems = {
///     + DataElementIdentifier => ErrorCode
/// }
/// ```
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize),
    validate(predicate = |items| !items.is_empty()),
)]
pub struct ErrorItems(IndexMap<DataElementIdentifier, ErrorCode>);

/// Error code
///
/// ```cddl
/// ErrorCode = int
/// ```
pub type ErrorCode = i32;

#[cfg(test)]
mod tests {
    use super::DeviceResponseStatus;
    use crate::utils::serialization::cbor_deserialize;
    use crate::utils::serialization::cbor_serialize;

    #[test]
    fn test_device_response_status_serializes_to_wire_values() {
        let encoded = cbor_serialize(&DeviceResponseStatus::InvalidRequest).unwrap();
        let decoded: u64 = cbor_deserialize(encoded.as_slice()).unwrap();

        assert_eq!(decoded, 12);
    }
}
