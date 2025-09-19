use std::result::Result;

use chrono::DateTime;
use chrono::Utc;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;

use crypto::keys::CredentialEcdsaKey;
use crypto::x509::BorrowingCertificate;
use utils::generator::Generator;

use crate::errors::Error;
use crate::iso::*;
use crate::utils::cose::CoseError;
use crate::verifier::ValidityRequirement;

use super::HolderError;

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mdoc {
    /// Mobile Security Object of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    mso: MobileSecurityObject,

    /// Identifier of the mdoc's private key. Obtain a reference to it with
    /// [`DisclosureWscd::new_key(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    private_key_id: String,
    issuer_signed: IssuerSigned,
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
        };
        Ok(mdoc)
    }

    pub fn doc_type(&self) -> &str {
        &self.mso.doc_type
    }

    pub fn issuer_signed(&self) -> &IssuerSigned {
        &self.issuer_signed
    }

    pub fn into_issuer_signed(self) -> IssuerSigned {
        self.issuer_signed
    }

    pub fn into_components(self) -> (MobileSecurityObject, String, IssuerSigned) {
        let Self {
            mso,
            private_key_id,
            issuer_signed,
        } = self;

        (mso, private_key_id, issuer_signed)
    }

    pub fn issuer_certificate(&self) -> Result<BorrowingCertificate, CoseError> {
        self.issuer_signed.issuer_auth.signing_cert()
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
    use crate::IssuerSigned;
    use crate::MobileSecurityObject;
    use crate::iso::mdocs::IssuerSignedItemBytes;

    use super::Mdoc;

    impl Mdoc {
        pub fn new_unverified(mso: MobileSecurityObject, private_key_id: String, issuer_signed: IssuerSigned) -> Self {
            Self {
                mso,
                private_key_id,
                issuer_signed,
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

#[cfg(any(test, feature = "mock_example_constructors"))]
pub mod mock {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::examples::EXAMPLE_KEY_IDENTIFIER;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;

    use crate::examples::IsoCertTimeGenerator;
    use crate::iso::disclosure::DeviceResponse;
    use crate::test::data::pid_example;

    use super::Mdoc;

    impl Mdoc {
        /// Out of the example data structures in the standard, assemble an mdoc.
        /// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the
        /// originating mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some
        /// [`IssuerSignedItem`] instances, i.e. attributes.
        /// This is because the other attributes are actually nowhere present in the standard so it is impossible to
        /// construct the example mdoc with all attributes present.
        ///
        /// Using tests should not rely on all attributes being present.
        pub async fn new_example_resigned(ca: &Ca) -> Self {
            let issuer_signed = DeviceResponse::example_resigned(ca)
                .await
                .documents
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .issuer_signed;

            Mdoc::new::<MockRemoteEcdsaKey>(
                EXAMPLE_KEY_IDENTIFIER.to_string(),
                issuer_signed,
                &IsoCertTimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap()
        }

        pub async fn new_mock() -> Self {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            Self::new_mock_with_ca(&ca).await
        }

        pub async fn new_mock_with_doctype(doc_type: &str) -> Self {
            let mut mdoc = Self::new_mock().await;
            mdoc.mso.doc_type = String::from(doc_type);
            mdoc
        }

        pub async fn new_mock_with_key(key: &MockRemoteEcdsaKey) -> Self {
            let ca = Ca::generate_issuer_mock_ca().unwrap();
            Self::new_mock_with_ca_and_key(&ca, key).await
        }

        pub async fn new_mock_with_ca(ca: &Ca) -> Self {
            let key = MockRemoteEcdsaKey::new("identifier".to_owned(), SigningKey::random(&mut OsRng));
            Self::new_mock_with_ca_and_key(ca, &key).await
        }

        pub async fn new_mock_with_ca_and_key(ca: &Ca, device_key: &MockRemoteEcdsaKey) -> Self {
            let test_document = pid_example().into_first().unwrap();
            test_document.sign(ca, device_key).await
        }
    }
}
