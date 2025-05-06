//! Data structures with which a verifier requests attributes from a holder.

use std::borrow::Cow;
use std::fmt::Debug;

use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use attestation_data::identifiers::AttributeIdentifier;
use attestation_data::identifiers::AttributeIdentifierHolder;

use crate::iso::engagement::*;
use crate::iso::mdocs::*;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::ReaderAuthenticationString;
use crate::utils::serialization::RequiredValue;
use crate::utils::serialization::TaggedBytes;

/// Sent by the RP to the holder to request the disclosure of attributes out of one or more mdocs.
/// For each mdoc out of which attributes are requested, a [`DocRequest`] is included.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRequest {
    pub version: DeviceRequestVersion,
    pub doc_requests: Vec<DocRequest>,
    /// This is a custom and optional field. Other implementations should ignore it.
    pub return_url: Option<Url>,
}

impl AttributeIdentifierHolder for DeviceRequest {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.doc_requests
            .iter()
            .flat_map(|doc_request| doc_request.items_request.0.attribute_identifiers())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum DeviceRequestVersion {
    #[default]
    #[serde(rename = "1.0")]
    V1_0,
}

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DeviceRequest`].
/// Includes reader (RP) authentication.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocRequest {
    pub items_request: ItemsRequestBytes,
    pub reader_auth: Option<ReaderAuth>,
}

pub type ReaderAuth = MdocCose<CoseSign1, Value>;
pub type ReaderAuthenticationBytes<'a> = TaggedBytes<ReaderAuthentication<'a>>;
pub type ReaderAuthentication<'a> = CborSeq<ReaderAuthenticationKeyed<'a>>;

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

/// See [`ItemsRequest`].
pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DocRequest`] in a
/// [`DeviceRequest`].
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemsRequest {
    pub doc_type: DocType,
    pub name_spaces: NameSpaces,

    /// Free-form additional information.
    pub request_info: Option<IndexMap<String, Value>>,
}

impl AttributeIdentifierHolder for ItemsRequest {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.name_spaces
            .iter()
            .flat_map(|(namespace, attributes)| {
                attributes.into_iter().map(|(attribute, _)| AttributeIdentifier {
                    credential_type: self.doc_type.to_owned(),
                    namespace: namespace.to_owned(),
                    attribute: attribute.to_owned(),
                })
            })
            .collect()
    }
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

#[cfg(any(test, feature = "examples"))]
mod examples {
    use std::iter;

    use indexmap::IndexMap;

    use crate::examples::EXAMPLE_ATTRIBUTES;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;

    use super::ItemsRequest;

    impl ItemsRequest {
        /// Build an [`ItemsRequest`] from a list of attributes.
        fn from_attributes(
            doc_type: String,
            name_space: String,
            attributes: impl Iterator<Item = impl Into<String>>,
        ) -> Self {
            Self {
                doc_type,
                name_spaces: IndexMap::from_iter([(
                    name_space,
                    attributes.map(|attribute| (attribute.into(), false)).collect(),
                )]),
                request_info: None,
            }
        }

        pub fn new_example() -> Self {
            Self::from_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            )
        }

        pub fn new_example_empty() -> Self {
            Self::from_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                iter::empty::<String>(),
            )
        }
    }
}
