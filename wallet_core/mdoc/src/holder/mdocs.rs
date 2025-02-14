use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use itertools::Itertools;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use sd_jwt::metadata::TypeMetadata;
use wallet_common::generator::Generator;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::CredentialKeyType;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::identifiers::AttributeIdentifier;
use crate::iso::*;
use crate::unsigned::Entry;
use crate::unsigned::UnsignedMdoc;
use crate::utils::cose::CoseError;
use crate::utils::x509::BorrowingCertificate;
use crate::verifier::ValidityRequirement;

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mdoc {
    /// Mobile Security Object of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub(crate) mso: MobileSecurityObject,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Keyfactory::generate(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub(crate) private_key_id: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) key_type: CredentialKeyType,
}

impl Mdoc {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> crate::Result<Mdoc> {
        let (_, mso) = issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;
        let mdoc = Mdoc {
            mso,
            private_key_id,
            issuer_signed,
            key_type: K::KEY_TYPE,
        };
        Ok(mdoc)
    }

    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    pub fn attributes(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed.to_entries_by_namespace()
    }

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    pub fn doc_type(&self) -> &String {
        &self.mso.doc_type
    }

    pub fn validity_info(&self) -> &ValidityInfo {
        &self.mso.validity_info
    }

    pub fn type_metadata(&self) -> crate::Result<VecNonEmpty<TypeMetadata>> {
        let (metadata, _) = self.issuer_signed.type_metadata()?.verify_and_destructure()?;
        Ok(metadata)
    }

    /// Check that the namespaces, attribute names and attribute values of this instance are equal to to the
    /// provided unsigned value.
    pub fn compare_unsigned(&self, unsigned: &UnsignedMdoc) -> Result<(), IssuedAttributesMismatch> {
        let our_attrs = self.attributes();
        let our_attrs = &flatten_attributes(self.doc_type(), &our_attrs);
        let expected_attrs = &flatten_attributes(&unsigned.doc_type, unsigned.attributes.as_ref());

        let missing = map_difference(expected_attrs, our_attrs);
        let unexpected = map_difference(our_attrs, expected_attrs);

        if !missing.is_empty() || !unexpected.is_empty() {
            return Err(IssuedAttributesMismatch { missing, unexpected });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[error("missing attributes: {missing:?}; unexpected attributes: {unexpected:?}")]
#[category(pd)]
pub struct IssuedAttributesMismatch<T = AttributeIdentifier> {
    pub missing: Vec<T>,
    pub unexpected: Vec<T>,
}

pub fn map_difference<K, T>(left: &IndexMap<K, T>, right: &IndexMap<K, T>) -> Vec<K>
where
    K: Clone + std::hash::Hash + Eq,
    T: PartialEq,
{
    left.iter()
        .filter_map(|(id, value)| (!right.contains_key(id) || right[id] != *value).then_some(id.clone()))
        .collect_vec()
}

fn flatten_attributes<'a>(
    doctype: &'a DocType,
    attrs: &'a IndexMap<NameSpace, Vec<Entry>>,
) -> IndexMap<AttributeIdentifier, &'a ciborium::Value> {
    attrs
        .iter()
        .flat_map(|(namespace, entries)| {
            entries.iter().map(|entry| {
                (
                    AttributeIdentifier {
                        credential_type: doctype.clone(),
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
