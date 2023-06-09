use std::marker::PhantomData;

use indexmap::IndexMap;
use x509_parser::prelude::X509Certificate;

use crate::{basic_sa_ext::Entry, crypto::sha256, iso::*, serialization::cbor_serialize, Result};

pub trait CredentialStorage<K> {
    fn add(&self, creds: impl Iterator<Item = Credential<K>>) -> Result<()>;
    fn list(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>>;

    // TODO returning all copies of all credentials is very crude and should be refined.
    fn get(&self, doctype: &DocType) -> Option<Vec<CredentialCopies<K>>>;
}

pub struct Wallet<K, C> {
    pub(crate) credential_storage: C,
    phantom_data: PhantomData<K>,
}

impl<K, C: CredentialStorage<K>> Wallet<K, C> {
    pub fn new(credential_storage: C) -> Self {
        Self {
            credential_storage,
            phantom_data: PhantomData,
        }
    }

    pub fn list_credentials(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.credential_storage.list()
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
///
/// TODO: support marking an mdoc has having been used, so that it can be avoided in future disclosures,
/// for unlinkability.
#[derive(Debug, Clone, Default)]
pub struct CredentialCopies<K> {
    pub creds: Vec<Credential<K>>,
}

impl<K> IntoIterator for CredentialCopies<K> {
    type Item = Credential<K>;
    type IntoIter = std::vec::IntoIter<Credential<K>>;
    fn into_iter(self) -> Self::IntoIter {
        self.creds.into_iter()
    }
}
impl<K> From<Vec<Credential<K>>> for CredentialCopies<K> {
    fn from(creds: Vec<Credential<K>>) -> Self {
        Self { creds }
    }
}
impl<K> CredentialCopies<K> {
    pub fn new() -> Self {
        CredentialCopies { creds: vec![] }
    }
}

/// A full mdoc credential: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone)]
pub struct Credential<K> {
    pub(crate) private_key: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub doc_type: String,
    pub(crate) phantom_data: PhantomData<K>,
}

impl<K> Credential<K> {
    pub fn new(private_key: String, issuer_signed: IssuerSigned, ca_cert: &X509Certificate) -> Result<Credential<K>> {
        let (_, mso) = issuer_signed.verify(ca_cert)?;
        let cred = Credential {
            private_key,
            issuer_signed,
            doc_type: mso.doc_type,
            phantom_data: PhantomData,
        };
        Ok(cred)
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
