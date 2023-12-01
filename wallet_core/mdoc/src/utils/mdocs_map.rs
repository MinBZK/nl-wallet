use indexmap::IndexMap;
use wallet_common::utils::sha256;

use crate::{
    basic_sa_ext::Entry,
    holder::{Mdoc, MdocCopies},
    DocType, Error, NameSpace,
};

use crate::utils::serialization::cbor_serialize;

/// An implementation of [`Storage`] using maps, structured as follows::
/// - mdocs with different doctypes, through the map over `DocType`,
/// - multiple mdocs having the same doctype but distinct attributes, through the map over `Vec<u8>` which is computed
///   with [`Mdoc::hash()`] (see its rustdoc for details),
/// - multiple mdocs having the same doctype and the same attributes, through the `MdocCopies` data structure.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MdocsMap(pub IndexMap<DocType, IndexMap<Vec<u8>, MdocCopies>>);

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
        for cred in creds {
            self.0
                .entry(cred.doc_type.clone())
                .or_default()
                .entry(cred.hash()?)
                .or_default()
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

impl Mdoc {
    /// Hash of the mdoc, acting as an identifier for the mdoc. Takes into account its doctype
    /// and all of its attributes, using [`Self::attributes()`].
    /// Computed schematically as `SHA256(CBOR(doctype, attributes))`.
    ///
    /// Credentials having the exact same attributes
    /// with the exact same values have the same hash, regardless of the randoms of the attributes; the issuer
    /// signature; or the validity of the mdoc.
    pub fn hash(&self) -> crate::Result<Vec<u8>> {
        let digest = sha256(&cbor_serialize(&(&self.doc_type, &self.attributes()))?);
        Ok(digest)
    }
}
