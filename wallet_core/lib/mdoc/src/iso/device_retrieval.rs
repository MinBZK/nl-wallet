//! Data structures with which a verifier requests attributes from a holder.

use std::borrow::Cow;
use std::fmt::Debug;

use attestation_types::claim_path::ClaimPath;
use ciborium::de::Error as CiboriumError;
use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::iso::engagement::*;
use crate::iso::mdocs::*;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::CborError;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::ReaderAuthenticationString;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;
use crate::utils::serialization::cbor_deserialize;

/// Sent by the RP to the holder to request the disclosure of attributes out of one or more mdocs.
/// For each mdoc out of which attributes are requested, a [`DocRequest`] is included.
///
/// ```cddl
/// DeviceRequest = {
///     "version" : tstr,
///     "docRequests" : [+ DocRequest],
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRequest {
    /// Version of DeviceRequest structure
    pub version: DeviceRequestVersion,

    /// Requested documents
    pub doc_requests: VecNonEmpty<DocRequest>,

    /// This is a custom and optional field. Other implementations should ignore it.
    pub return_url: Option<Url>,
}

#[derive(thiserror::Error, Debug)]
pub enum DeviceRequestParseError {
    #[error("malformed CBOR: {0}")]
    MalformedCbor(#[source] CborError),
    #[error("invalid DeviceRequest structure: {0}")]
    InvalidStructure(#[source] CborError),
}

/// Version of [`DeviceRequest`] structure
///
/// In the currently implemented version of the spec its value shall be "1.0". If other versions are specified in the
/// future, the major version of a [`DeviceRequest`] structure shall not be higher than the major version of the device
/// engagement structure communicated by the mdoc in the same transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceRequestVersion {
    #[default]
    #[serde(rename = "1.0")]
    V1_0,
}

/// Contains an array of all requested documentes, as part of a [`DeviceRequest`].
///
/// ```cddl
/// DocRequest = {
///     "itemsRequest" : ItemsRequestBytes,
///     ? "readerAuth" : ReaderAuth
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocRequest {
    pub items_request: ItemsRequestBytes,

    /// mdoc reader authentication
    pub reader_auth: Option<ReaderAuth>,
}

/// The signature over the [`ReaderAuthentication`] structure.
///
/// ```cddl
/// ReaderAuth = COSE_Sign1
/// ```
pub type ReaderAuth = MdocCose<CoseSign1, Value>;

/// The data that the mdoc reader authenticates.
///
/// The mdoc reader shall generate this structure and calculate the signature. In order to verify the data, the mdoc
/// shall generate the structure as well and validate the signature.
///
/// ```cddl
/// ReaderAuthentication = [
///     "ReaderAuthentication",
///     SessionTranscript,
///     ItemsRequestBytes
/// ]
/// ```
pub type ReaderAuthentication<'a> = CborSeq<ReaderAuthenticationKeyed<'a>>;

/// See [`ReaderAuthentication`].
///
/// ```cddl
/// ReaderAuthenticationBytes = #6.24(bstr .cbor ReaderAuthentication)
/// ```
pub type ReaderAuthenticationBytes<'a> = TaggedBytes<ReaderAuthentication<'a>>;

/// See [`ReaderAuthentication`].
// In production code, this struct is never deserialized.
#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
pub struct ReaderAuthenticationKeyed<'a> {
    pub reader_auth_string: RequiredValue<ReaderAuthenticationString>,
    pub session_transcript: Cow<'a, SessionTranscript>,
    pub items_request_bytes: Cow<'a, ItemsRequestBytes>,
}

impl<'a> ReaderAuthenticationKeyed<'a> {
    pub fn new(session_transcript: &'a SessionTranscript, items_request_bytes: &'a ItemsRequestBytes) -> Self {
        ReaderAuthenticationKeyed {
            reader_auth_string: Default::default(),
            session_transcript: Cow::Borrowed(session_transcript),
            items_request_bytes: Cow::Borrowed(items_request_bytes),
        }
    }
}

/// Contains the [`ItemsRequest`] structure as a tagged CBOR bytestring.
///
/// ```cddl
/// ItemsRequestBytes = #6.24(bstr .cbor ItemsRequest)
/// ```
pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DocRequest`] in a
/// [`DeviceRequest`].
///
/// ```cddl
/// ItemsRequest = {
///     "docType" : DocType,
///     "nameSpaces" : NameSpaces,
///     ? "requestInfo" : {* tstr => any}
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsRequest {
    /// Document type requested
    pub doc_type: DocType,
    pub name_spaces: NameSpaces,

    /// Additional information
    pub request_info: Option<IndexMap<String, Value>>,
}

impl ItemsRequest {
    pub fn claims(&self) -> impl Iterator<Item = VecNonEmpty<ClaimPath>> {
        self.name_spaces.as_ref().iter().flat_map(|(namespace, identifiers)| {
            identifiers
                .into_iter()
                .map(|(identifier, _)| {
                    vec_nonempty![
                        ClaimPath::SelectByKey(namespace.clone()),
                        ClaimPath::SelectByKey(identifier.clone()),
                    ]
                })
                .collect_vec()
        })
    }

    pub fn into_doctype_and_claims(self) -> (DocType, impl Iterator<Item = VecNonEmpty<ClaimPath>>) {
        (
            self.doc_type,
            self.name_spaces.into_iter().flat_map(|(namespace, identifiers)| {
                identifiers
                    .into_iter()
                    .map(|(identifier, _)| {
                        vec_nonempty![
                            ClaimPath::SelectByKey(namespace.clone()),
                            ClaimPath::SelectByKey(identifier),
                        ]
                    })
                    .collect_vec()
            }),
        )
    }
}

impl DeviceRequest {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, DeviceRequestParseError> {
        let value: Value = cbor_deserialize(bytes).map_err(DeviceRequestParseError::MalformedCbor)?;

        value.deserialized().map_err(|error| {
            DeviceRequestParseError::InvalidStructure(CborError::Deserialization(CiboriumError::Semantic(
                None,
                error.to_string(),
            )))
        })
    }

    pub fn from_doc_requests(doc_requests: VecNonEmpty<DocRequest>) -> Self {
        DeviceRequest {
            doc_requests,
            version: DeviceRequestVersion::V1_0,
            return_url: None,
        }
    }

    pub fn from_items_requests(items_requests: impl IntoNonEmptyIterator<Item = ItemsRequest>) -> Self {
        Self::from_doc_requests(
            items_requests
                .into_nonempty_iter()
                .map(|items_request| DocRequest {
                    items_request: items_request.into(),
                    reader_auth: None,
                })
                .collect(),
        )
    }
}

/// Requested data elements for each NameSpace
///
/// The attribute names that the RP wishes disclosed, grouped per namespace, as part of a [`ItemsRequest`].
///
/// ```cddl
/// NameSpaces = {
///     + NameSpace => DataElements
/// }
/// ```
#[nutype(
    derive(Debug, Clone,  TryFrom, PartialEq, Eq, AsRef, IntoIterator, Serialize, Deserialize),
    validate(predicate = |elems| !elems.is_empty()),
)]
pub struct NameSpaces(IndexMap<NameSpace, DataElements>);

/// Requested data element identifiers with intent to retain values
///
/// The attribute names that the RP wishes disclosed within a particular namespace, as part of a [`ItemsRequest`],
/// along with a boolean with which the RP can claim its intention to (not) retain the attribute value after receiving
/// and verifying it.
///
/// ```cddl
/// DataElements = {
///     + DataElementIdentifier => IntentToRetain
/// }
/// ```
#[nutype(
    derive(Debug, Clone, TryFrom, PartialEq, Eq, AsRef, IntoIterator, Serialize, Deserialize),
    validate(predicate = |elems| !elems.is_empty()),
)]
pub struct DataElements(IndexMap<DataElementIdentifier, IntentToRetain>);

///  Claimed intention of the RP to (not) retain the attribute value after receiving and verifying it, as part of
/// [`DataElements`] within a [`ItemsRequest`].
///
/// ```cddl
/// IntentToRetain = bool
/// ```
pub type IntentToRetain = bool;

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::DeviceRequest;
    use super::DeviceRequestParseError;
    use crate::examples::Example;
    use crate::utils::serialization::cbor_serialize;

    #[test]
    fn test_device_request_try_from_bytes_reports_malformed_cbor() {
        assert_matches!(
            DeviceRequest::try_from_bytes(&[0xff]),
            Err(DeviceRequestParseError::MalformedCbor(_))
        );
    }

    #[test]
    fn test_device_request_try_from_bytes_reports_truncated_cbor_encoding() {
        assert_matches!(
            DeviceRequest::try_from_bytes(&[0x81]),
            Err(DeviceRequestParseError::MalformedCbor(_))
        );
    }

    #[test]
    fn test_device_request_try_from_bytes_reports_invalid_structure() {
        let bytes = cbor_serialize(&42u8).unwrap();

        assert_matches!(
            DeviceRequest::try_from_bytes(&bytes),
            Err(DeviceRequestParseError::InvalidStructure(_))
        );
    }

    #[test]
    fn test_device_request_try_from_bytes_parses_valid_request() {
        let device_request = DeviceRequest::example();
        let bytes = cbor_serialize(&device_request).unwrap();

        assert_eq!(DeviceRequest::try_from_bytes(&bytes).unwrap(), device_request);
    }
}

#[cfg(test)]
mod examples {
    use indexmap::IndexMap;
    use utils::vec_at_least::NonEmptyIterator;

    use super::ItemsRequest;
    use crate::examples::EXAMPLE_ATTRIBUTES;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;

    impl ItemsRequest {
        /// Build an [`ItemsRequest`] from a list of attributes.
        fn from_attributes(
            doc_type: String,
            name_space: String,
            attributes: impl NonEmptyIterator<Item = impl Into<String>>,
        ) -> Self {
            Self {
                doc_type,
                name_spaces: IndexMap::from_iter([(
                    name_space,
                    attributes
                        .into_iter()
                        .map(|attribute| (attribute.into(), false))
                        .collect::<IndexMap<_, _>>()
                        .try_into()
                        .unwrap(),
                )])
                .try_into()
                .unwrap(),
                request_info: None,
            }
        }

        pub fn new_example() -> Self {
            Self::from_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.nonempty_iter().copied(),
            )
        }
    }
}
