use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, utils::sha256};

use crate::{
    basic_sa_ext::Entry,
    iso::*,
    utils::{keys::MdocEcdsaKey, serialization::cbor_serialize},
    verifier::ValidityRequirement,
    Result,
};

use super::{CborHttpClient, HolderError, HttpClient, SessionState};

pub trait Storage {
    fn add<K: MdocEcdsaKey>(&self, creds: impl Iterator<Item = Mdoc<K>>) -> Result<()>;
    fn list<K: MdocEcdsaKey>(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>>;

    // TODO returning all copies of all mdocs is very crude and should be refined.
    fn get<K: MdocEcdsaKey>(&self, doctype: &DocType) -> Option<Vec<MdocCopies<K>>>;
}

pub struct Wallet<C, H = CborHttpClient> {
    pub(crate) storage: C,
    pub(crate) session_state: Option<SessionState<H>>,
}

impl<C: Storage, H: HttpClient> Wallet<C, H> {
    pub fn new(storage: C) -> Self {
        Self {
            storage,
            session_state: None,
        }
    }

    pub fn list_mdocs<K: MdocEcdsaKey>(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.storage.list::<K>()
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
///
/// TODO: support marking an mdoc has having been used, so that it can be avoided in future disclosures,
/// for unlinkability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "K: MdocEcdsaKey")]
pub struct MdocCopies<K: MdocEcdsaKey> {
    pub cred_copies: Vec<Mdoc<K>>,
}

impl<K: MdocEcdsaKey> IntoIterator for MdocCopies<K> {
    type Item = Mdoc<K>;
    type IntoIter = std::vec::IntoIter<Mdoc<K>>;
    fn into_iter(self) -> Self::IntoIter {
        self.cred_copies.into_iter()
    }
}
impl<K: MdocEcdsaKey> From<Vec<Mdoc<K>>> for MdocCopies<K> {
    fn from(creds: Vec<Mdoc<K>>) -> Self {
        Self { cred_copies: creds }
    }
}
impl<K: MdocEcdsaKey> MdocCopies<K> {
    pub fn new() -> Self {
        MdocCopies::<K> { cred_copies: vec![] }
    }
}

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "K: MdocEcdsaKey")]
pub struct Mdoc<K> {
    /// Doctype of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub doc_type: String,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Mdoc::private_key()`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub(crate) private_key_id: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) key_type: PrivateKeyType<K>,
}

/// Represents the type of the private key, in Rust using a generic type that implements [`MdocEcdsaKey`],
/// and when serialized using [`MdocEcdsaKey::KEY_TYPE`]. This serializes to a &'static str associated to the type `K`,
/// to be stored in the database that stores the mdoc.
/// Deserialization fails at runtime if the corresponding string in the serialized value does not equal the `KEY_TYPE`
/// constant from the specified type `K`.
#[derive(Debug, Clone, Default)]
pub struct PrivateKeyType<K>(PhantomData<K>);

impl<K: MdocEcdsaKey> PrivateKeyType<K> {
    pub fn new() -> PrivateKeyType<K> {
        PrivateKeyType(PhantomData)
    }
}

impl<K: MdocEcdsaKey> Serialize for PrivateKeyType<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        K::KEY_TYPE.serialize(serializer)
    }
}

impl<'de, K: MdocEcdsaKey> Deserialize<'de> for PrivateKeyType<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let serialized_key_type = String::deserialize(deserializer)?;
        if serialized_key_type == K::KEY_TYPE {
            Ok(PrivateKeyType(PhantomData))
        } else {
            Err(serde::de::Error::custom(HolderError::PrivateKeyTypeMismatch {
                expected: K::KEY_TYPE.to_string(),
                have: serialized_key_type,
            }))
        }
    }
}

impl<K: MdocEcdsaKey> Mdoc<K> {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new(
        private_key: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Mdoc<K>> {
        let (_, mso) = issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;
        let mdoc = Mdoc {
            doc_type: mso.doc_type,
            private_key_id: private_key,
            issuer_signed,
            key_type: PrivateKeyType::new(),
        };
        Ok(mdoc)
    }

    pub(crate) fn private_key(&self) -> K {
        K::new(&self.private_key_id)
    }

    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    pub fn attributes(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .unwrap_or(&IndexMap::new())
            .iter()
            .map(|(namespace, attrs)| (namespace.clone(), Vec::<Entry>::from(attrs)))
            .collect::<IndexMap<_, _>>()
    }

    /// Hash of the mdoc, acting as an identifier for the mdoc. Takes into account its doctype
    /// and all of its attributes, using [`Self::attributes()`].
    /// Computed schematically as `SHA256(CBOR(doctype, attributes))`.
    ///
    /// Credentials having the exact same attributes
    /// with the exact same values have the same hash, regardless of the randoms of the attributes; the issuer
    /// signature; or the validity of the mdoc.
    pub fn hash(&self) -> Result<Vec<u8>> {
        let digest = sha256(&cbor_serialize(&(&self.doc_type, &self.attributes()))?);
        Ok(digest)
    }
}
