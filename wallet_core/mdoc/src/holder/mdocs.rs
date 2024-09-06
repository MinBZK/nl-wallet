use std::result::Result;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use itertools::Itertools;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use webpki::TrustAnchor;

use error_category::ErrorCategory;
use wallet_common::generator::Generator;

use crate::{
    identifiers::AttributeIdentifier,
    iso::*,
    unsigned::{Entry, UnsignedMdoc},
    utils::{
        cose::CoseError,
        keys::{MdocEcdsaKey, MdocKeyType},
        x509::Certificate,
    },
    verifier::ValidityRequirement,
};

/// Stores multiple copies of credentials that have identical attributes.
#[nutype(
    validate(predicate = |copies| !copies.is_empty()),
    derive(Debug, Clone, AsRef, TryFrom, Serialize, Deserialize, PartialEq)
)]
pub struct CredentialCopies<T>(Vec<T>);

pub type MdocCopies = CredentialCopies<Mdoc>;

impl<T> IntoIterator for CredentialCopies<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<T> CredentialCopies<T> {
    pub fn first(&self) -> &T {
        self.as_ref().first().unwrap()
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mdoc {
    /// Doctype of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub doc_type: String,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Keyfactory::generate(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub(crate) private_key_id: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) key_type: MdocKeyType,
}

impl Mdoc {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new<K: MdocEcdsaKey>(
        private_key_id: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> crate::Result<Mdoc> {
        let (_, mso) = issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;
        let mdoc = Mdoc {
            doc_type: mso.doc_type,
            private_key_id,
            issuer_signed,
            key_type: K::KEY_TYPE,
        };
        Ok(mdoc)
    }

    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    pub fn attributes(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .as_ref()
                    .iter()
                    .map(|(name_space, attrs)| (name_space.clone(), attrs.into()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn issuer_certificate(&self) -> Result<Certificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    /// Check that the namespaces, attribute names and attribute values of this instance are equal to to the
    /// provided unsigned value.
    pub fn compare_unsigned(&self, unsigned: &UnsignedMdoc) -> Result<(), IssuedAttributesMismatch> {
        let our_attrs = self.attributes();
        let our_attrs = flatten_map(&self.doc_type, &our_attrs);
        let expected_attrs = flatten_map(&unsigned.doc_type, unsigned.attributes.as_ref());

        let missing = expected_attrs
            .iter()
            .filter_map(|(id, expected)| {
                if !our_attrs.contains_key(id) || our_attrs[id] != *expected {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect_vec();

        let unexpected = our_attrs
            .iter()
            .filter_map(|(id, received)| {
                if !expected_attrs.contains_key(id) || expected_attrs[id] != *received {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect_vec();

        if !missing.is_empty() || !unexpected.is_empty() {
            return Err(IssuedAttributesMismatch { missing, unexpected });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[error("missing attributes: {missing:?}; unexpected attributes: {unexpected:?}")]
#[category(pd)]
pub struct IssuedAttributesMismatch {
    pub missing: Vec<AttributeIdentifier>,
    pub unexpected: Vec<AttributeIdentifier>,
}

fn flatten_map<'a>(
    doctype: &'a DocType,
    attrs: &'a IndexMap<NameSpace, Vec<Entry>>,
) -> IndexMap<AttributeIdentifier, &'a ciborium::Value> {
    attrs
        .iter()
        .flat_map(|(namespace, entries)| {
            entries.iter().map(|entry| {
                (
                    AttributeIdentifier {
                        doc_type: doctype.clone(),
                        namespace: namespace.clone(),
                        attribute: entry.name.clone(),
                    },
                    &entry.value,
                )
            })
        })
        .collect()
}

#[cfg(any(test, feature = "test"))]
mod test {
    use crate::IssuerSignedItemBytes;

    use super::Mdoc;

    impl Mdoc {
        pub fn modify_attributes<F>(&mut self, name_space: &str, modify_func: F)
        where
            F: FnOnce(&mut Vec<IssuerSignedItemBytes>),
        {
            let name_spaces = self.issuer_signed.name_spaces.as_mut().unwrap();
            name_spaces.modify_attributes(name_space, modify_func);
        }
    }
}
