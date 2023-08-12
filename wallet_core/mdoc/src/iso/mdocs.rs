//! Data structures contained in mdocs.
//!
//! The main citizen of this module is [`MobileSecurityObject`], which is the object that is signed by the issuer.
//! This data structure does not directly contain the attributes ([`IssuerSignedItem`]) but instead only their digests,
//! to enable selective disclosure.

use chrono::Utc;
use ciborium::{tag, value::Value};
use indexmap::IndexMap;
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use wallet_common::utils::random_bytes;

use crate::{
    basic_sa_ext::Entry,
    utils::{cose::CoseKey, crypto::cbor_digest, serialization::TaggedBytes},
    Error, Result,
};

/// Name of a namespace within an mdoc.
pub type NameSpace = String;

/// Digest (hash) of an attribute, computed over a [`IssuerSignedItemBytes`], included in the device-signed part
/// ([`MobileSecurityObject`]) of an mdoc.
pub type Digest = ByteBuf;

/// Incrementing integer identifying attributes within an mdoc.
pub type DigestID = u64;

/// A map containing attribute digests keyed by the attribute ID (an incrementing integer).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigestIDs(pub IndexMap<DigestID, Digest>);
impl TryFrom<&Attributes> for DigestIDs {
    type Error = Error;
    fn try_from(val: &Attributes) -> Result<Self> {
        let ids = DigestIDs(
            val.0
                .iter()
                .enumerate()
                .map(|(i, attr)| Ok((i as u64, ByteBuf::from(cbor_digest(attr)?))))
                .collect::<Result<IndexMap<_, _>>>()?,
        );
        Ok(ids)
    }
}

/// Digests of the attributes, grouped per [`NameSpace`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueDigests(pub IndexMap<NameSpace, DigestIDs>);
impl TryFrom<&IssuerNameSpaces> for ValueDigests {
    type Error = Error;
    fn try_from(val: &IssuerNameSpaces) -> Result<Self> {
        let digests = ValueDigests(
            val.iter()
                .map(|(namespace, attrs)| Ok((namespace.clone(), DigestIDs::try_from(attrs)?)))
                .collect::<Result<IndexMap<_, _>>>()?,
        );
        Ok(digests)
    }
}

/// Free-form information about the device key (see [`DeviceKeyInfo`]).
///
///  ISO 18013-5: "Positive integers are RFU, negative integers may be used for proprietary use".
pub type KeyInfo = IndexMap<i32, Value>;

/// Namespaces under which the holder may include self-asserted attributes, as determined by the [`KeyAuthorizations`]
/// in the mdoc's device key.
pub type AuthorizedNameSpaces = Vec<NameSpace>;
/// Specific attributes grouped by namespace that the holder may include in its self-asserted attributes, as determined
/// by the [`KeyAuthorizations`] in the mdoc's device key.
pub type AuthorizedDataElements = IndexMap<NameSpace, DataElementsArray>;
/// Specific attributes in a namespace that the holder may include in its self-asserted attributes, as determined
/// by the [`KeyAuthorizations`] in the mdoc's device key.
pub type DataElementsArray = Vec<DataElementIdentifier>;

/// Name of an attribute, see [`IssuerSignedItem`].
pub type DataElementIdentifier = String;
/// Value of an attribute, see [`IssuerSignedItem`]. May be any CBOR value.
pub type DataElementValue = Value;

/// Specific attributes that the holder of this mdoc is allowed to self-assert, or whole namespaces under which the
/// holder is allowed to self-assert attributes.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyAuthorizations {
    pub name_spaces: Option<AuthorizedNameSpaces>,
    pub data_elements: Option<AuthorizedDataElements>,
}

/// An mdoc public key ([`DeviceKey`]) along with some information about it, as part of the
/// [`MobileSecurityObject`] of an mdoc.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceKeyInfo {
    pub device_key: DeviceKey,
    pub key_authorizations: Option<KeyAuthorizations>,
    pub key_info: Option<KeyInfo>,
}

impl TryFrom<VerifyingKey> for DeviceKeyInfo {
    type Error = Error;
    fn try_from(value: VerifyingKey) -> Result<Self> {
        let key_info = DeviceKeyInfo {
            device_key: (&value).try_into()?,
            key_authorizations: None,
            key_info: None,
        };
        Ok(key_info)
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

/// Public key of an mdoc, contained in [`DeviceKeyInfo`] which is contained in [`MobileSecurityObject`].
pub type DeviceKey = CoseKey;

/// Data signed by the issuer, containing a.o.
/// - The public key of the mdoc (in [`DeviceKeyInfo`])
/// - the digests of the attributes ([`ValueDigests`]), but not their randoms (for that see the containing struct
///   [`IssuerSigned`](super::IssuerSigned))
/// - When the mdoc was signed by the issuer and when it expires ([`ValidityInfo`]).
///
/// This is signed by the issuer during issuance into a COSE and included in an [`IssuerSigned`](super::IssuerSigned).
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

/// A date-time, serialized as a string value as specified in RFC 3339, e.g. `"2020-10-01T13:30:02Z"`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tdate(pub tag::Required<String, 0>);
impl From<chrono::DateTime<Utc>> for Tdate {
    fn from(t: chrono::DateTime<Utc>) -> Self {
        Tdate(tag::Required(t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)))
    }
}

impl Tdate {
    pub fn now() -> Tdate {
        Utc::now().into()
    }
}

/// Doctype of an mdoc. For example, `"org.iso.18013.5.1.mDL"`. Determines the namespaces and attribute names that the
/// mdoc may or must contain, and the issuer(s) that are authorized to sign it.
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
        let attrs = Attributes(
            val.into_iter()
                .enumerate()
                .map(|(i, (key, val))| Ok(IssuerSignedItem::new(i as u64, key, val)?.into()))
                .collect::<Result<Vec<_>>>()?,
        );
        Ok(attrs)
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

/// See [`IssuerSignedItem`].
pub type IssuerSignedItemBytes = TaggedBytes<IssuerSignedItem>;

/// An attribute, containing
/// - an identifying incrementing number (`digestID`),
/// - random bytes for selective disclosure (see below),
/// - the attribute's name and value.
///
/// This value is kept by the holder, and transmitted to the RP during disclosure, but it is not directly included
/// in the MSO itself; instead its digest (hash) is. This enables selective disclosure for the holder, by witholding
/// this value for an attribute that it wants to hide. The RP then only sees the hash of the attribute in the MSO,
/// which hides the attribute from it because of the `random` bytes.
///
/// See also
/// - [`Entry`], which contains just the name and value of the attribute,
/// - [`Digest`] and [`DigestIDs`]: the digests (hashes) of [`IssuerSignedItem`]s, contained in the MSO.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssuerSignedItem {
    #[serde(rename = "digestID")]
    pub digest_id: u64,
    pub random: ByteBuf,
    pub element_identifier: DataElementIdentifier,
    pub element_value: DataElementValue,
}

impl IssuerSignedItem {
    /// Generate a new `IssuerSignedItem` including a new `random`.
    pub fn new(
        digest_id: u64,
        element_identifier: DataElementIdentifier,
        element_value: DataElementValue,
    ) -> Result<IssuerSignedItem> {
        let random = ByteBuf::from(random_bytes(32));
        let item = IssuerSignedItem {
            digest_id,
            random,
            element_identifier,
            element_value,
        };
        Ok(item)
    }
}
