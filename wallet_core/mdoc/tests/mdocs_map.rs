use dashmap::DashMap;

use indexmap::IndexMap;
use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    holder::{Mdoc, MdocCopies, Storage},
    utils::{
        serialization::{cbor_deserialize, cbor_serialize},
        signer::{MdocEcdsaKey, SoftwareEcdsaKey},
    },
    DocType, Error, NameSpace,
};

/// An implementation of [`Storage`] using maps, structured as follows::
/// - mdocs with different doctypes, through the map over `DocType`,
/// - multiple mdocs having the same doctype but distinct attributes, through the map over `Vec<u8>` which is computed
///   with [`Mdoc::hash()`] (see its rustdoc for details),
/// - multiple mdocs having the same doctype and the same attributes, through the `MdocCopies` data structure.
#[derive(Debug, Clone, Default)]
pub struct MdocsMap(pub(crate) DashMap<DocType, DashMap<Vec<u8>, MdocCopies<SoftwareEcdsaKey>>>);

impl<const N: usize> TryFrom<[Mdoc<SoftwareEcdsaKey>; N]> for MdocsMap {
    type Error = Error;

    fn try_from(value: [Mdoc<SoftwareEcdsaKey>; N]) -> Result<Self, Self::Error> {
        let creds = MdocsMap(DashMap::new());
        creds.add(value.into_iter())?;
        Ok(creds)
    }
}

impl MdocsMap {
    pub fn new() -> MdocsMap {
        MdocsMap(DashMap::new())
    }
}

// `impl Storage for CredentialsMap` below requires its method to be generic over K,
// but in these tests we want to deal only with `SoftwareEcdsaKey`. This is some ugly trickery to cast between the two,
// which works because in fact the `Mdoc` type only stores the identifier string of its key.
fn to_software_key<K: MdocEcdsaKey>(cred: Mdoc<K>) -> Mdoc<SoftwareEcdsaKey> {
    cbor_deserialize::<Mdoc<SoftwareEcdsaKey>, _>(cbor_serialize(&cred).unwrap().as_slice()).unwrap()
}
fn from_software_key<K: MdocEcdsaKey>(cred: Mdoc<SoftwareEcdsaKey>) -> Mdoc<K> {
    cbor_deserialize::<Mdoc<K>, _>(cbor_serialize(&cred).unwrap().as_slice()).unwrap()
}

impl Storage for MdocsMap {
    fn add<K: MdocEcdsaKey>(&self, creds: impl Iterator<Item = Mdoc<K>>) -> Result<(), Error> {
        for cred in creds.into_iter() {
            self.0
                .entry(cred.doc_type.clone())
                .or_insert(DashMap::new())
                .entry(cred.hash()?)
                .or_insert(MdocCopies::new())
                .cred_copies
                .push(to_software_key(cred));
        }

        Ok(())
    }

    fn get<K: MdocEcdsaKey>(&self, doctype: &DocType) -> Option<Vec<MdocCopies<K>>> {
        self.0.get(doctype).map(|v| {
            v.value()
                .iter()
                .map(|entry| {
                    entry
                        .value()
                        .clone()
                        .into_iter()
                        .map(|cred| from_software_key(cred))
                        .collect::<Vec<_>>()
                        .into()
                })
                .collect()
        })
    }

    fn list<K>(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.0
            .iter()
            .map(|allcreds| {
                (
                    allcreds.key().clone(),
                    allcreds
                        .iter()
                        .map(|doctype_creds| doctype_creds.cred_copies.first().unwrap().attributes())
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }
}
