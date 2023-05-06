//! Data structures containing mdoc credentials.
//!
//! The main citizen of this module is [`MobileSecurityObject`], which is the object that is signed by the issuer.
//! This data structure does not directly contain the attributes ([`IssuerSignedItem`]) but instead only their digests,
//! to enable selective disclosure.

use chrono::Utc;
use ciborium::{tag, value::Value};
use ecdsa::VerifyingKey;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use crate::{
    basic_sa_ext::Entry,
    cose::CoseKey,
    crypto::{cbor_digest, random_bytes},
    serialization::TaggedBytes,
    Error, Result,
};

pub type NameSpace = String;

pub type Digest = ByteBuf;
pub type DigestID = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigestIDs(pub IndexMap<DigestID, Digest>);
impl TryFrom<&Attributes> for DigestIDs {
    type Error = Error;
    fn try_from(val: &Attributes) -> Result<Self> {
        Ok(DigestIDs(
            val.0
                .iter()
                .enumerate()
                .map(|(i, attr)| Ok((i as u64, ByteBuf::from(cbor_digest(attr)?))))
                .collect::<Result<IndexMap<_, _>>>()?,
        ))
    }
}

/// Digests of the attributes, grouped per [`NameSpace`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueDigests(pub IndexMap<NameSpace, DigestIDs>);
impl TryFrom<&IssuerNameSpaces> for ValueDigests {
    type Error = Error;
    fn try_from(val: &IssuerNameSpaces) -> Result<Self> {
        Ok(ValueDigests(
            val.iter()
                .map(|(namespace, attrs)| Ok((namespace.clone(), DigestIDs::try_from(attrs)?)))
                .collect::<Result<IndexMap<_, _>>>()?,
        ))
    }
}

pub type KeyInfo = IndexMap<i32, Value>;

pub type AuthorizedNameSpaces = Vec<NameSpace>;
pub type AuthorizedDataElements = IndexMap<NameSpace, DataElementsArray>;
pub type DataElementsArray = Vec<DataElementIdentifier>;
pub type DataElementIdentifier = String;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyAuthorizations {
    pub name_spaces: Option<AuthorizedNameSpaces>,
    pub data_elements: Option<AuthorizedDataElements>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceKeyInfo {
    pub device_key: DeviceKey,
    pub key_authorizations: Option<KeyAuthorizations>,
    pub key_info: Option<KeyInfo>,
}

impl TryFrom<VerifyingKey<p256::NistP256>> for DeviceKeyInfo {
    type Error = Error;
    fn try_from(value: VerifyingKey<p256::NistP256>) -> Result<Self> {
        Ok(DeviceKeyInfo {
            device_key: (&value).try_into()?,
            key_authorizations: None,
            key_info: None,
        })
    }
}
impl From<CoseKey> for DeviceKeyInfo {
    fn from(value: CoseKey) -> Self {
        DeviceKeyInfo {
            device_key: value,
            key_authorizations: None,
            key_info: None,
        }
    }
}

pub type DeviceKey = CoseKey;

/// Data signed by the issuer, containing among others the digests of the attributes ([`ValueDigests`]).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MobileSecurityObject {
    pub version: String,
    pub digest_algorithm: String,
    pub value_digests: ValueDigests,
    pub device_key_info: DeviceKeyInfo,
    pub doc_type: String,
    pub validity_info: ValidityInfo,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidityInfo {
    pub signed: Tdate,
    pub valid_from: Tdate,
    pub valid_until: Tdate,
    pub expected_update: Option<Tdate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tdate(pub tag::Required<String, 0>);
impl From<chrono::DateTime<Utc>> for Tdate {
    fn from(t: chrono::DateTime<Utc>) -> Self {
        Tdate(tag::Required(t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)))
    }
}

pub type DocType = String;

/// [`Attributes`], which contains [`IssuerSignedItem`]s, grouped per [`NameSpace`].
pub type IssuerNameSpaces = IndexMap<NameSpace, Attributes>;

/// A `Vec` of [`IssuerSignedItemBytes`], i.e., attributes. In the [`IssuerNameSpaces`] map,
/// this is used as the type of the keys. (This datastructure is itself not named in the spec.)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attributes(pub Vec<IssuerSignedItemBytes>);
impl From<Vec<IssuerSignedItemBytes>> for Attributes {
    fn from(val: Vec<IssuerSignedItemBytes>) -> Self {
        Attributes(val)
    }
}
impl TryFrom<IndexMap<String, Value>> for Attributes {
    type Error = Error;
    fn try_from(val: IndexMap<String, Value>) -> Result<Self> {
        Ok(Attributes(
            val.into_iter()
                .enumerate()
                .map(|(i, (key, val))| Ok(IssuerSignedItem::new(i as u64, key, val)?.into()))
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}
impl TryFrom<Vec<Entry>> for Attributes {
    type Error = Error;
    fn try_from(attrs: Vec<Entry>) -> std::result::Result<Self, Self::Error> {
        Attributes::try_from(
            attrs
                .iter()
                .map(|entry| (entry.name.clone(), entry.value.clone()))
                .collect::<IndexMap<String, Value>>(),
        )
    }
}
impl From<&Attributes> for Vec<Entry> {
    fn from(attrs: &Attributes) -> Self {
        attrs
            .0
            .iter()
            .map(|issuer_signed| Entry {
                name: issuer_signed.0.element_identifier.clone(),
                value: issuer_signed.0.element_value.clone(),
            })
            .collect()
    }
}

pub type IssuerSignedItemBytes = TaggedBytes<IssuerSignedItem>;

/// An attribute.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssuerSignedItem {
    #[serde(rename = "digestID")]
    pub digest_id: u64,
    pub random: ByteBuf,
    pub element_identifier: String,
    pub element_value: Value,
}

impl IssuerSignedItem {
    pub fn new(digest_id: u64, element_identifier: String, element_value: Value) -> Result<IssuerSignedItem> {
        let random = ByteBuf::from(random_bytes(32));
        Ok(IssuerSignedItem {
            digest_id,
            random,
            element_identifier,
            element_value,
        })
    }
}
