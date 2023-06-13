use std::marker::PhantomData;

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use x509_parser::prelude::X509Certificate;

use crate::{basic_sa_ext::Entry, crypto::sha256, iso::*, serialization::cbor_serialize, signer::MdocEcdsaKey, Result};

use super::HolderError;

pub trait CredentialStorage {
    fn add<K: MdocEcdsaKey>(&self, creds: impl Iterator<Item = Credential<K>>) -> Result<()>;
    fn list<K: MdocEcdsaKey>(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>>;

    // TODO returning all copies of all credentials is very crude and should be refined.
    fn get<K: MdocEcdsaKey>(&self, doctype: &DocType) -> Option<Vec<CredentialCopies<K>>>;
}

pub struct Wallet<C> {
    pub(crate) credential_storage: C,
}

impl<C: CredentialStorage> Wallet<C> {
    pub fn new(credential_storage: C) -> Self {
        Self { credential_storage }
    }

    pub fn list_credentials<K: MdocEcdsaKey>(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.credential_storage.list::<K>()
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
///
/// TODO: support marking an mdoc has having been used, so that it can be avoided in future disclosures,
/// for unlinkability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "K: MdocEcdsaKey")]
pub struct CredentialCopies<K: MdocEcdsaKey> {
    pub cred_copies: Vec<Credential<K>>,
}

impl<K: MdocEcdsaKey> IntoIterator for CredentialCopies<K> {
    type Item = Credential<K>;
    type IntoIter = std::vec::IntoIter<Credential<K>>;
    fn into_iter(self) -> Self::IntoIter {
        self.cred_copies.into_iter()
    }
}
impl<K: MdocEcdsaKey> From<Vec<Credential<K>>> for CredentialCopies<K> {
    fn from(creds: Vec<Credential<K>>) -> Self {
        Self { cred_copies: creds }
    }
}
impl<K: MdocEcdsaKey> CredentialCopies<K> {
    pub fn new() -> Self {
        CredentialCopies::<K> { cred_copies: vec![] }
    }
}

/// A full mdoc credential: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "K: MdocEcdsaKey")]
pub struct Credential<K> {
    pub doc_type: String,

    /// Identifier of the credential's private key. Obtain a reference to it with [`Credential::private_key()`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the credential and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // credential after deserialization.
    pub(crate) private_key: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) key_type: PrivateKeyType<K>,
}

/// Represents the type of the private key, in Rust using a generic type that implements [`MdocEcdsaKey`],
/// and when serialized using [`MdocEcdsaKey::KEY_TYPE`]. This serializes to a &'static str associated to the type `K`,
/// to be stored in the database that stores the credential.
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

impl<K: MdocEcdsaKey> Credential<K> {
    pub fn new(private_key: String, issuer_signed: IssuerSigned, ca_cert: &X509Certificate) -> Result<Credential<K>> {
        let (_, mso) = issuer_signed.verify(ca_cert)?;
        Ok(Self::_new(mso.doc_type, private_key, issuer_signed))
    }

    pub(crate) fn _new(doc_type: DocType, private_key: String, issuer_signed: IssuerSigned) -> Credential<K> {
        Credential {
            doc_type,
            private_key,
            issuer_signed,
            key_type: PrivateKeyType::new(),
        }
    }

    pub(crate) fn private_key(&self) -> K {
        K::new(&self.private_key)
    }

    pub fn attributes(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .unwrap()
            .iter()
            .map(|(namespace, attrs)| (namespace.clone(), Vec::<Entry>::from(attrs)))
            .collect::<IndexMap<_, _>>()
    }

    /// Hash of the credential, acting as an identifier for the credential that takes into account its doctype
    /// and all of its attributes. Computed schematically as `SHA256(CBOR(doctype, attributes))`.
    pub fn hash(&self) -> Result<Vec<u8>> {
        let digest = sha256(&cbor_serialize(&(&self.doc_type, &self.attributes()))?);
        Ok(digest)
    }
}
