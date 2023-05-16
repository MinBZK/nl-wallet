use dashmap::DashMap;
use indexmap::IndexMap;
use x509_parser::prelude::X509Certificate;

use crate::{basic_sa_ext::Entry, crypto::sha256, iso::*, serialization::cbor_serialize, Error, Result};

/// Mdoc credentials of the holder. This data structure supports storing:
/// - mdocs with different doctypes, through the map over `DocType`,
/// - multiple mdocs having the same doctype but distinct attributes, through the map over `Vec<u8>` which is computed
///   with [`Credential::hash()`] (see its rustdoc for details),
/// - multiple mdocs having the same doctype and the same attributes, through the `CredentialCopies` data structure.
#[derive(Debug, Clone, Default)]
pub struct Credentials(pub(crate) DashMap<DocType, DashMap<Vec<u8>, CredentialCopies>>);

impl<const N: usize> TryFrom<[Credential; N]> for Credentials {
    type Error = Error;

    fn try_from(value: [Credential; N]) -> Result<Self> {
        let creds = Credentials(DashMap::new());
        creds.add(value.into_iter())?;
        Ok(creds)
    }
}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials(DashMap::new())
    }

    pub fn add(&self, creds: impl Iterator<Item = Credential>) -> Result<()> {
        for cred in creds.into_iter() {
            self.0
                .entry(cred.doc_type.clone())
                .or_insert(DashMap::new())
                .entry(cred.hash()?)
                .or_insert(CredentialCopies::new())
                .creds
                .push(cred);
        }

        Ok(())
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
///
/// TODO: support marking an mdoc has having been used, so that it can be avoided in future disclosures,
/// for unlinkability.
#[derive(Debug, Clone)]
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
    fn new() -> Self {
        CredentialCopies { creds: vec![] }
    }
}

/// A full mdoc credential: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone)]
pub struct Credential {
    pub(crate) private_key: ecdsa::SigningKey<p256::NistP256>,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) doc_type: String,
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

    /// Hash of the credential, acting as an identifier for the credential that takes into account its doctype
    /// and all of its attributes. Computed schematically as `SHA256(CBOR(doctype, attributes))`.
    fn hash(&self) -> Result<Vec<u8>> {
        let digest = sha256(&cbor_serialize(&(
            &self.doc_type,
            &self
                .issuer_signed
                .name_spaces
                .as_ref()
                .unwrap()
                .iter()
                .map(|(namespace, attrs)| (namespace.clone(), Vec::<Entry>::from(attrs)))
                .collect::<IndexMap<_, _>>(),
        ))?);
        Ok(digest)
    }
}
