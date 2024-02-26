use std::result::Result;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use webpki::TrustAnchor;

use wallet_common::generator::Generator;

use crate::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    identifiers::AttributeIdentifier,
    iso::*,
    utils::{
        cose::CoseError,
        keys::{MdocEcdsaKey, MdocKeyType},
        x509::Certificate,
    },
    verifier::ValidityRequirement,
};

use super::{CborHttpClient, HttpClient, IssuanceSessionState};

pub struct Wallet<H = CborHttpClient> {
    pub(crate) session_state: Option<IssuanceSessionState>,
    pub(crate) client: H,
}

impl<H: HttpClient> Wallet<H> {
    pub fn new(client: H) -> Self {
        Self {
            session_state: None,
            client,
        }
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MdocCopies {
    pub cred_copies: Vec<Mdoc>,
}

impl IntoIterator for MdocCopies {
    type Item = Mdoc;
    type IntoIter = std::vec::IntoIter<Mdoc>;
    fn into_iter(self) -> Self::IntoIter {
        self.cred_copies.into_iter()
    }
}
impl From<Vec<Mdoc>> for MdocCopies {
    fn from(creds: Vec<Mdoc>) -> Self {
        Self { cred_copies: creds }
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
            .unwrap_or(&IndexMap::new())
            .iter()
            .map(|(namespace, attrs)| (namespace.clone(), Vec::<Entry>::from(attrs)))
            .collect::<IndexMap<_, _>>()
    }

    pub fn issuer_certificate(&self) -> Result<Certificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    /// Check that the namespaces, attribute names and attribute values of this instance are equal to to the
    /// provided unsigned value.
    pub fn compare_unsigned(&self, unsigned: &UnsignedMdoc) -> Result<(), IssuedAttributesMismatch> {
        let our_attrs = self.attributes();
        let our_attrs = flatten_map(&self.doc_type, &our_attrs);
        let expected_attrs = flatten_map(&unsigned.doc_type, &unsigned.attributes);

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

#[derive(Debug, thiserror::Error)]
#[error("missing attributes: {missing:?}; unexpected attributes: {unexpected:?}")]
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
