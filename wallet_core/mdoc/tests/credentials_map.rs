use dashmap::DashMap;

use indexmap::IndexMap;
use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    holder::{Credential, CredentialCopies, CredentialStorage},
    DocType, Error, NameSpace,
};

/// An implementation of [`CredentialStorage`] using maps, structured as follows::
/// - mdocs with different doctypes, through the map over `DocType`,
/// - multiple mdocs having the same doctype but distinct attributes, through the map over `Vec<u8>` which is computed
///   with [`Credential::hash()`] (see its rustdoc for details),
/// - multiple mdocs having the same doctype and the same attributes, through the `CredentialCopies` data structure.
#[derive(Debug, Clone, Default)]
pub struct CredentialsMap(pub(crate) DashMap<DocType, DashMap<Vec<u8>, CredentialCopies>>);

impl<const N: usize> TryFrom<[Credential; N]> for CredentialsMap {
    type Error = Error;

    fn try_from(value: [Credential; N]) -> Result<Self, Self::Error> {
        let creds = CredentialsMap(DashMap::new());
        creds.add(value.into_iter())?;
        Ok(creds)
    }
}

impl CredentialsMap {
    pub fn new() -> CredentialsMap {
        CredentialsMap(DashMap::new())
    }
}

impl CredentialStorage for CredentialsMap {
    fn add(&self, creds: impl Iterator<Item = Credential>) -> Result<(), Error> {
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

    fn get(&self, doctype: &DocType) -> Option<Vec<CredentialCopies>> {
        self.0
            .get(doctype)
            .map(|v| v.value().iter().map(|entry| entry.value().clone()).collect())
    }

    fn list(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.0
            .iter()
            .map(|allcreds| {
                (
                    allcreds.key().clone(),
                    allcreds
                        .iter()
                        .map(|doctype_creds| doctype_creds.creds.first().unwrap().attributes())
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }
}
