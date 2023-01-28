//! Data structures containing mdoc credentials.
//!
//! The main citizen of this module is [`MobileSecurityObject`], which is the object that is signed by the issuer.
//! This data structure does not directly contain the attributes ([`IssuerSignedItem`]) but instead only their digests,
//! to enable selective disclosure.

use crate::{
    cose::CoseKey,
    crypto::{cbor_digest, random_bytes},
    serialization::TaggedBytes,
};

use anyhow::Result;
use chrono::Utc;
use ciborium::{tag, value::Value};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::fmt::Debug;

pub type NameSpace = String;

pub type Digest = ByteBuf;
pub type DigestID = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigestIDs(pub IndexMap<DigestID, Digest>);
impl TryFrom<&Attributes> for DigestIDs {
    type Error = anyhow::Error;
    fn try_from(val: &Attributes) -> Result<Self> {
        Ok(DigestIDs(
            val.0
                .iter()
                .enumerate()
                .map(|(i, attr)| Ok((i as u32, ByteBuf::from(cbor_digest(attr)?))))
                .collect::<Result<IndexMap<_, _>, anyhow::Error>>()?,
        ))
    }
}

/// Digests of the attributes, grouped per [`NameSpace`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueDigests(pub IndexMap<NameSpace, DigestIDs>);
impl TryFrom<&IssuerNameSpaces> for ValueDigests {
    type Error = anyhow::Error;
    fn try_from(val: &IssuerNameSpaces) -> Result<Self> {
        Ok(ValueDigests(
            val.iter()
                .map(|(namespace, attrs)| Ok((namespace.clone(), DigestIDs::try_from(attrs)?)))
                .collect::<Result<IndexMap<_, _>, anyhow::Error>>()?,
        ))
    }
}

pub type KeyInfo = IndexMap<i32, Value>;

pub type AuthorizedNameSpaces = Vec<NameSpace>;
pub type AuthorizedDataElements = IndexMap<NameSpace, DataElementsArray>;
pub type DataElementsArray = Vec<DataElementIdentifier>;
pub type DataElementIdentifier = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyAuthorizations {
    #[serde(rename = "nameSpaces")]
    pub(crate) name_spaces: Option<AuthorizedNameSpaces>,
    #[serde(rename = "dataElements")]
    pub(crate) data_elements: Option<AuthorizedDataElements>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceKeyInfo {
    #[serde(rename = "deviceKey")]
    pub(crate) device_key: DeviceKey,
    #[serde(rename = "keyAuthorizations")]
    pub(crate) key_authorizations: Option<KeyAuthorizations>,
    #[serde(rename = "KeyInfo")]
    pub(crate) key_info: Option<KeyInfo>,
}

pub type DeviceKey = CoseKey;

/// Data signed by the issuer, containing among others the digests of the attributes ([`ValueDigests`]).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileSecurityObject {
    pub(crate) version: String,
    #[serde(rename = "digestAlgorithm")]
    pub(crate) digest_algorithm: String,
    #[serde(rename = "valueDigests")]
    pub(crate) value_digests: ValueDigests,
    #[serde(rename = "deviceKeyInfo")]
    pub(crate) device_key_info: DeviceKeyInfo,
    #[serde(rename = "docType")]
    pub(crate) doc_type: String,
    #[serde(rename = "validityInfo")]
    pub(crate) validity_info: ValidityInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidityInfo {
    pub(crate) signed: Tdate,
    #[serde(rename = "validFrom")]
    pub(crate) valid_from: Tdate,
    #[serde(rename = "validUntil")]
    pub(crate) valid_until: Tdate,
    #[serde(rename = "expectedUpdate")]
    pub(crate) expected_update: Option<Tdate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tdate(pub tag::Required<String, 0>);
impl From<chrono::DateTime<Utc>> for Tdate {
    fn from(t: chrono::DateTime<Utc>) -> Self {
        Tdate(tag::Required(
            t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ))
    }
}

pub type DocType = String;

/// [`Attributes`], which contains [`IssuerSignedItem`]s, grouped per [`NameSpace`].
pub type IssuerNameSpaces = IndexMap<NameSpace, Attributes>;

/// A `Vec` of [`IssuerSignedItemBytes`], i.e., attributes. In the [`IssuerNameSpaces`] map,
/// this is used as the type of the keys. (This datastructure is itself not named in the spec.)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attributes(pub(crate) Vec<IssuerSignedItemBytes>);
impl From<Vec<IssuerSignedItemBytes>> for Attributes {
    fn from(val: Vec<IssuerSignedItemBytes>) -> Self {
        Attributes(val)
    }
}
impl TryFrom<IndexMap<String, Value>> for Attributes {
    type Error = anyhow::Error;
    fn try_from(val: IndexMap<String, Value>) -> Result<Self> {
        Ok(Attributes(
            val.into_iter()
                .enumerate()
                .map(|(i, (key, val))| Ok(IssuerSignedItem::new(i as u32, key, val)?.into()))
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
        ))
    }
}

pub type IssuerSignedItemBytes = TaggedBytes<IssuerSignedItem>;

/// An attribute.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerSignedItem {
    #[serde(rename = "digestID")]
    pub(crate) digest_id: u32,
    #[serde(with = "serde_bytes")]
    pub(crate) random: Vec<u8>,
    #[serde(rename = "elementIdentifier")]
    pub(crate) element_identifier: String,
    #[serde(rename = "elementValue")]
    pub(crate) element_value: Value,
}

impl IssuerSignedItem {
    pub fn new(
        digest_id: u32,
        element_identifier: String,
        element_value: Value,
    ) -> Result<IssuerSignedItem> {
        let random = random_bytes(32)?;
        Ok(IssuerSignedItem {
            digest_id,
            random,
            element_identifier,
            element_value,
        })
    }
}
