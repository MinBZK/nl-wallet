use indexmap::IndexMap;
use x509_parser::prelude::X509Certificate;

use crate::{basic_sa_ext::Entry, crypto::sha256, iso::*, serialization::cbor_serialize, Result};

pub trait CredentialStorage {
    fn add(&self, creds: impl Iterator<Item = Credential>) -> Result<()>;
    fn list(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>>;

    // TODO returning all copies of all credentials is very crude and should be refined.
    fn get(&self, doctype: &DocType) -> Option<Vec<CredentialCopies>>;
}

pub struct Wallet<C> {
    pub(crate) credential_storage: C,
}

impl<C: CredentialStorage> Wallet<C> {
    pub fn new(credential_storage: C) -> Self {
        Self { credential_storage }
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
pub struct CredentialCopies {
    pub creds: Vec<Credential>,
}

impl IntoIterator for CredentialCopies {
    type Item = Credential;
    type IntoIter = std::vec::IntoIter<Credential>;
    fn into_iter(self) -> Self::IntoIter {
        self.creds.into_iter()
    }
}
impl From<Vec<Credential>> for CredentialCopies {
    fn from(creds: Vec<Credential>) -> Self {
        Self { creds }
    }
}
impl CredentialCopies {
    pub fn new() -> Self {
        CredentialCopies { creds: vec![] }
    }
}

/// A full mdoc credential: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone)]
pub struct Credential {
    pub(crate) private_key: ecdsa::SigningKey<p256::NistP256>,
    pub(crate) issuer_signed: IssuerSigned,
    pub doc_type: String,
}

impl Credential {
    pub fn new(
        private_key: ecdsa::SigningKey<p256::NistP256>,
        issuer_signed: IssuerSigned,
        ca_cert: &X509Certificate,
    ) -> Result<Credential> {
        let (_, mso) = issuer_signed.verify(ca_cert)?;
        let cred = Credential {
            private_key,
            issuer_signed,
            doc_type: mso.doc_type,
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
