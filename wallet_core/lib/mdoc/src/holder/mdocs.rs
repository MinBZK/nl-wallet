use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use itertools::Itertools;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;

use crypto::keys::CredentialEcdsaKey;
use crypto::keys::CredentialKeyType;
use crypto::x509::BorrowingCertificate;
use error_category::ErrorCategory;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use wallet_common::generator::Generator;
use wallet_common::urls::HttpsUri;

use crate::errors::Error;
use crate::identifiers::AttributeIdentifier;
use crate::iso::*;
use crate::unsigned::Entry;
use crate::unsigned::UnsignedMdoc;
use crate::utils::cose::CoseError;
use crate::verifier::ValidityRequirement;

use super::HolderError;

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mdoc {
    /// Mobile Security Object of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub mso: MobileSecurityObject,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Keyfactory::generate(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub private_key_id: String,
    pub issuer_signed: IssuerSigned,
    pub key_type: CredentialKeyType,
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

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
    }

    pub fn doc_type(&self) -> &String {
        &self.mso.doc_type
    }

    pub fn validity_info(&self) -> &ValidityInfo {
        &self.mso.validity_info
    }

    pub fn type_metadata_integrity(&self) -> Result<&Integrity, Error> {
        let integrity = self
            .mso
            .type_metadata_integrity
            .as_ref()
            .ok_or(HolderError::MissingMetadataIntegrity)?;

        Ok(integrity)
    }

    pub fn type_metadata(&self) -> Result<NormalizedTypeMetadata, Error> {
        let documents = self.issuer_signed.type_metadata_documents()?;

        let (metadata, sorted) = documents
            .into_normalized(&self.mso.doc_type)
            .map_err(HolderError::TypeMetadata)?;

        let integrity = self.type_metadata_integrity()?.clone();
        let _verified = sorted.into_verified(integrity).map_err(HolderError::TypeMetadata)?;

        Ok(metadata)
    }

    /// Check that the doc_type, issuer, validity_info, attestation_qualification, namespaces, attribute names and
    /// attribute values of this instance are equal to to the provided unsigned value.
    pub fn compare_unsigned(&self, unsigned: &UnsignedMdoc) -> Result<(), IssuedDocumentMismatchError> {
        if self.mso.doc_type != unsigned.doc_type {
            return Err(IssuedDocumentMismatchError::IssuedDoctypeMismatch(
                unsigned.doc_type.clone(),
                self.mso.doc_type.clone(),
            ));
        }

        match self.mso.issuer_uri.as_ref() {
            None => Err(IssuedDocumentMismatchError::IssuedIssuerMissing),
            Some(issuer_uri) if *issuer_uri != unsigned.issuer_uri => {
                Err(IssuedDocumentMismatchError::IssuedIssuerMismatch(
                    Box::new(unsigned.issuer_uri.clone()),
                    Box::new(issuer_uri.clone()),
                ))
            }
            Some(_) => Ok(()),
        }?;

        if self.mso.validity_info.valid_from != unsigned.valid_from
            || self.mso.validity_info.valid_until != unsigned.valid_until
        {
            return Err(IssuedDocumentMismatchError::IssuedValidityInfoMismatch(
                (unsigned.valid_from.clone(), unsigned.valid_until.clone()),
                (
                    self.mso.validity_info.valid_from.clone(),
                    self.mso.validity_info.valid_until.clone(),
                ),
            ));
        }

        match self.mso.attestation_qualification.as_ref() {
            None => Err(IssuedDocumentMismatchError::IssuedAttestationQualificationMissing),
            Some(attestation_qualification) if *attestation_qualification != unsigned.attestation_qualification => {
                Err(IssuedDocumentMismatchError::IssuedAttestationQualificationMismatch(
                    unsigned.attestation_qualification,
                    *attestation_qualification,
                ))
            }
            Some(_) => Ok(()),
        }?;

        let our_attrs = self.issuer_signed.clone().into_entries_by_namespace();
        let our_attrs = &flatten_attributes(self.doc_type(), &our_attrs);
        let expected_attrs = &flatten_attributes(&unsigned.doc_type, unsigned.attributes.as_ref());

        let missing = map_difference(expected_attrs, our_attrs);
        let unexpected = map_difference(our_attrs, expected_attrs);

        if !missing.is_empty() || !unexpected.is_empty() {
            return Err(IssuedDocumentMismatchError::IssuedAttributesMismatch(
                missing, unexpected,
            ));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum IssuedDocumentMismatchError<T = AttributeIdentifier> {
    #[error("issued doc_type mismatch: expected {0}, found {1}")]
    #[category(pd)]
    IssuedDoctypeMismatch(String, String),
    #[error("issued issuer common name missing")]
    #[category(critical)]
    IssuedIssuerMissing,
    #[error("issued issuer mismatch: expected {0:?}, found {0:?}")]
    #[category(pd)]
    IssuedIssuerMismatch(Box<HttpsUri>, Box<HttpsUri>),
    #[error("issued validity info mismatch: expected {0:?}, found {1:?}")]
    #[category(critical)]
    IssuedValidityInfoMismatch((Tdate, Tdate), (Tdate, Tdate)),
    #[error("issued attributes mismatch: missing {0}, unexpected {1}")]
    #[category(pd)]
    IssuedAttributesMismatch(Vec<T>, Vec<T>),
    #[error("issued attestation qualification missing")]
    #[category(critical)]
    IssuedAttestationQualificationMissing,
    #[error("issued attestation qualification mismatch: expected {0}, found {1}")]
    #[category(critical)]
    IssuedAttestationQualificationMismatch(AttestationQualification, AttestationQualification),
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
