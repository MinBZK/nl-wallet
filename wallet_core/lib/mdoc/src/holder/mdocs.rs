use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;

use crypto::keys::CredentialEcdsaKey;
use crypto::keys::CredentialKeyType;
use crypto::x509::BorrowingCertificate;
use utils::generator::Generator;

use crate::errors::Error;
use crate::identifiers::AttributeIdentifier;
use crate::iso::*;
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
    key_type: CredentialKeyType,
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

    pub fn issuer_signed_attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.issuer_signed.attribute_identifiers(self.doc_type())
    }

    pub fn doc_type(&self) -> &String {
        &self.mso.doc_type
    }

    pub fn type_metadata_integrity(&self) -> Result<&Integrity, Error> {
        let integrity = self
            .mso
            .type_metadata_integrity
            .as_ref()
            .ok_or(HolderError::MissingMetadataIntegrity)?;

        Ok(integrity)
    }
}

#[cfg(any(test, feature = "test"))]
mod test {
    use crypto::CredentialKeyType;

    use crate::iso::mdocs::IssuerSignedItemBytes;
    use crate::IssuerSigned;
    use crate::MobileSecurityObject;

    use super::Mdoc;

    impl Mdoc {
        pub fn new_unverified(
            mso: MobileSecurityObject,
            private_key_id: String,
            issuer_signed: IssuerSigned,
            key_type: CredentialKeyType,
        ) -> Self {
            Self {
                mso,
                private_key_id,
                issuer_signed,
                key_type,
            }
        }

        pub fn modify_attributes<F>(&mut self, name_space: &str, modify_func: F)
        where
            F: FnOnce(&mut Vec<IssuerSignedItemBytes>),
        {
            let name_spaces = self.issuer_signed.name_spaces.as_mut().unwrap();
            name_spaces.modify_attributes(name_space, modify_func);
        }
    }
}
