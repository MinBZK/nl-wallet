use indexmap::IndexMap;

use crate::{
    basic_sa_ext::Entry,
    holder::{Mdoc, MdocCopies, MdocRetriever},
    DocType, Error, NameSpace,
};

/// An implementation of [`Storage`] using maps, structured as follows::
/// - mdocs with different doctypes, through the map over `DocType`,
/// - multiple mdocs having the same doctype but distinct attributes, through the map over `Vec<u8>` which is computed
///   with [`Mdoc::hash()`] (see its rustdoc for details),
/// - multiple mdocs having the same doctype and the same attributes, through the `MdocCopies` data structure.
#[derive(Debug, Clone, Default)]
pub struct MdocsMap(pub(crate) IndexMap<DocType, IndexMap<Vec<u8>, MdocCopies>>);

impl<const N: usize> TryFrom<[Mdoc; N]> for MdocsMap {
    type Error = Error;

    fn try_from(value: [Mdoc; N]) -> Result<Self, Self::Error> {
        let mut creds = MdocsMap::new();
        creds.add(value.into_iter())?;
        Ok(creds)
    }
}

impl TryFrom<Vec<Mdoc>> for MdocsMap {
    type Error = Error;

    fn try_from(value: Vec<Mdoc>) -> Result<Self, Self::Error> {
        let mut creds = MdocsMap::new();
        creds.add(value.into_iter())?;
        Ok(creds)
    }
}

impl MdocsMap {
    pub fn new() -> MdocsMap {
        MdocsMap(IndexMap::new())
    }

    pub fn add(&mut self, creds: impl Iterator<Item = Mdoc>) -> Result<(), Error> {
        for cred in creds.into_iter() {
            self.0
                .entry(cred.doc_type.clone())
                .or_insert(IndexMap::new())
                .entry(cred.hash()?)
                .or_insert(MdocCopies::new())
                .cred_copies
                .push(cred);
        }

        Ok(())
    }

    pub fn list(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.0
            .iter()
            .map(|(key, allcreds)| {
                (
                    key.clone(),
                    allcreds
                        .iter()
                        .map(|(_key, doctype_creds)| doctype_creds.cred_copies.first().unwrap().attributes())
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }
}

#[cfg(feature = "mock")]
impl MdocRetriever for MdocsMap {
    fn get(&self, doctype: &DocType) -> Option<Vec<MdocCopies>> {
        self.0.get(doctype).map(|v| {
            v.iter()
                .map(|(_key, entry)| entry.cred_copies.to_vec().into())
                .collect()
        })
    }
}
