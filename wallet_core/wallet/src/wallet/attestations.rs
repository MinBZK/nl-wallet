use tracing::info;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::IntoCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use crypto::x509::BorrowingCertificateExtension;
use crypto::x509::CertificateError;
use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use mdoc::utils::cose::CoseError;
use platform_support::attested_key::AttestedKeyHolder;

use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::attestation::AttestationPresentation;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::StoredAttestationCopy;
use crate::storage::StoredAttestationFormat;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AttestationsError {
    #[error("could not fetch documents from database storage: {0}")]
    Storage(#[from] StorageError),

    #[error("could not extract Mdl extension from X.509 certificate: {0}")]
    Certificate(#[from] CertificateError),

    #[error("could not interpret X.509 certificate: {0}")]
    Cose(#[from] CoseError),

    #[error("X.509 certificate does not contain IssuerRegistration")]
    #[category(critical)]
    MissingIssuerRegistration,

    #[error("Sd-JWT does not contain an issuer certificate")]
    #[category(critical)]
    MissingIssuerCertificate,

    #[error("could not extract type metadata from mdoc: {0}")]
    #[category(defer)]
    Metadata(#[source] mdoc::Error),

    #[error("could not convert SD-JWT to a CredentialPayload: {0}")]
    #[category(defer)]
    CredentialPayloadConversion(#[from] SdJwtCredentialPayloadError),

    #[error("error converting credential payload to attestation: {0}")]
    #[category(defer)]
    Attestation(#[from] AttestationError),
}

pub type AttestationsCallback = Box<dyn FnMut(Vec<AttestationPresentation>) + Send + Sync>;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    S: Storage,
    AKH: AttestedKeyHolder,
{
    pub(super) async fn emit_attestations(&mut self) -> Result<(), AttestationsError> {
        info!("Emit attestations from storage");

        let storage = self.storage.read().await;

        let attestations = storage
            .fetch_unique_attestations()
            .await?
            .into_iter()
            .map(
                |StoredAttestationCopy {
                     attestation_id,
                     attestation,
                     normalized_metadata,
                     ..
                 }| {
                    match attestation {
                        StoredAttestationFormat::MsoMdoc { mdoc } => {
                            let issuer_certificate = mdoc.issuer_certificate()?;
                            let issuer_registration = IssuerRegistration::from_certificate(&issuer_certificate)?
                                .ok_or(AttestationsError::MissingIssuerRegistration)?;

                            let attestation = AttestationPresentation::create_for_issuance(
                                AttestationIdentity::Fixed {
                                    id: attestation_id.to_string(),
                                },
                                normalized_metadata,
                                issuer_registration.organization,
                                mdoc.issuer_signed.into_entries_by_namespace(),
                            )?;

                            Ok(attestation)
                        }
                        StoredAttestationFormat::SdJwt { sd_jwt } => {
                            let issuer_certificate = sd_jwt
                                .issuer_certificate()
                                .ok_or(AttestationsError::MissingIssuerCertificate)?;
                            let issuer_registration = IssuerRegistration::from_certificate(issuer_certificate)?
                                .ok_or(AttestationsError::MissingIssuerRegistration)?;

                            let payload = sd_jwt.into_credential_payload(&normalized_metadata)?;
                            let attestation = AttestationPresentation::create_from_attributes(
                                AttestationIdentity::Fixed {
                                    id: attestation_id.to_string(),
                                },
                                normalized_metadata,
                                issuer_registration.organization,
                                payload.previewable_payload.attributes,
                            )?;

                            Ok(attestation)
                        }
                    }
                },
            )
            .collect::<Result<Vec<_>, AttestationsError>>()?;

        if let Some(ref mut callback) = self.attestations_callback {
            callback(attestations);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_attestations_callback(
        &mut self,
        callback: AttestationsCallback,
    ) -> Result<Option<AttestationsCallback>, AttestationsError> {
        let previous_callback = self.attestations_callback.replace(callback);

        if self.registration.is_registered() {
            self.emit_attestations().await?;
        }

        Ok(previous_callback)
    }

    pub fn clear_attestations_callback(&mut self) -> Option<AttestationsCallback> {
        self.attestations_callback.take()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;

    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;
    use openid4vc::issuance_session::CredentialWithMetadata;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::issuance_session::IssuedCredentialCopies;
    use sd_jwt::sd_jwt::SdJwt;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    // Tests both setting and clearing the attestations callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_attestations_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);

        // Confirm that the callback was not called.
        {
            let attestations = attestations.lock();

            assert!(attestations.is_empty());
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    // Tests both setting and clearing the documents callback on a registered `Wallet`.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();

        let mdoc = test::create_example_pid_mdoc();
        let sd_jwt = SdJwt::example_pid_sd_jwt(&issuance_keypair);

        let attestation_type = sd_jwt
            .claims()
            .properties
            .get("vct")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        wallet.storage.write().await.issued_credential_copies.insert(
            attestation_type.clone(),
            vec![
                CredentialWithMetadata::new(
                    IssuedCredentialCopies::new_or_panic(
                        vec![IssuedCredential::SdJwt(Box::new(sd_jwt))].try_into().unwrap(),
                    ),
                    attestation_type.clone(),
                    VerifiedTypeMetadataDocuments::nl_pid_example(),
                ),
                CredentialWithMetadata::new(
                    IssuedCredentialCopies::new_or_panic(
                        vec![IssuedCredential::MsoMdoc(Box::new(mdoc))].try_into().unwrap(),
                    ),
                    attestation_type.clone(),
                    VerifiedTypeMetadataDocuments::nl_pid_example(),
                ),
            ],
        );

        // Register mock document_callback
        let attestations = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("Failed to set mock attestations callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);

        // Confirm that we received a single `Document` on the callback.
        {
            let attestations = attestations.lock();

            let attestation = attestations
                .first()
                .expect("Attestations callback should have been called")
                .first()
                .expect("Attestations callback should have been provided an Mdoc");
            assert_eq!(attestation.attestation_type, attestation_type);
        }

        // Clear the documents callback on the `Wallet.`
        wallet.clear_attestations_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 1);
    }

    // Tests that setting the documents callback on a registered `Wallet`, with invalid issuer certificate raises
    // a `MissingIssuerRegistration` error.
    #[tokio::test]
    async fn test_wallet_set_clear_documents_callback_registered_no_issuer_registration() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // The database contains a single `Mdoc`, without Issuer registration.
        let mdoc = test::create_example_pid_mdoc_unauthenticated();
        let credential = IssuedCredential::MsoMdoc(Box::new(mdoc.clone()));

        wallet.storage.write().await.issued_credential_copies.insert(
            mdoc.doc_type().clone(),
            vec![CredentialWithMetadata::new(
                IssuedCredentialCopies::new_or_panic(vec![credential].try_into().unwrap()),
                mdoc.doc_type().clone(),
                VerifiedTypeMetadataDocuments::nl_pid_example(),
            )],
        );

        // Register mock attestation_callback
        let (attestations, error) = test::setup_mock_attestations_callback(&mut wallet)
            .await
            .map(|_| ())
            .expect_err("Expected error");

        assert_matches!(error, AttestationsError::MissingIssuerRegistration);

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&attestations), 2);
    }

    #[tokio::test]
    async fn test_wallet_set_attestations_callback_error() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the database return an error on query.
        wallet.storage.write().await.has_query_error = true;

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_attestations_callback(Box::new(|_| {}))
            .await
            .map(|_| ())
            .expect_err("Setting attestations callback should have resulted in an error");

        assert_matches!(error, AttestationsError::Storage(_));
    }
}
