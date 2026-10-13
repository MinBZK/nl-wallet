use std::collections::HashMap;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use error_category::ErrorCategory;
use jwt::error::JwtVerifyError;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;
use tracing::info;
use tracing::warn;
use update_policy_model::update_policy::VersionState;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use wallet_account::messages::instructions::RefreshWalletCertificate;
use wallet_account::messages::registration::WalletCertificate;
use wallet_configuration::wallet_config::CertificatePublicKey;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::Wallet;
use super::WalletRegistration;
use crate::account_provider::AccountProviderClient;
use crate::errors::StorageError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionError;
use crate::repository::Repository;
use crate::storage::Storage;

/// Errors that can occur while refreshing the wallet certificate
///
/// Only an `AccountRevoked` error is ever returned, any other error is logged and swallowed, to not fail the action
/// triggering the refresh.
#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub(super) enum RefreshCertificateError {
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[source] InstructionError),
    #[error("could not validate refreshed wallet certificate received from Wallet Provider: {0}")]
    CertificateValidation(#[source] JwtVerifyError),
    #[error(
        "public key in refreshed wallet certificate received from Wallet Provider does not match hardware public key"
    )]
    #[category(critical)]
    PublicKeyMismatch,
    #[error("wallet ID in refreshed wallet certificate received from Wallet Provider does not match current wallet ID")]
    #[category(critical)]
    WalletIdMismatch,
    #[error("could not persist refreshed wallet certificate to database: {0}")]
    CertificateStorage(#[source] StorageError),
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    /// Refresh the wallet certificate if needed. A failure to refresh is swallowed, unless it reveals that the account
    /// has been revoked.
    pub(super) async fn refresh_wallet_certificate_if_needed(
        &mut self,
        remote_instruction: &InstructionClient<S, AKH::AppleKey, AKH::GoogleKey, APC>,
        current_certificate: &WalletCertificate,
        config: &WalletConfiguration,
    ) -> Result<(), RefreshCertificateError>
    where
        UR: Repository<VersionState>,
        S: Storage,
        APC: AccountProviderClient,
    {
        if certificate_needs_refresh(
            current_certificate,
            &config.account_server.certificate_public_keys,
            config.account_server.certificate_refresh_threshold,
            &TimeGenerator,
        ) {
            match self
                .refresh_wallet_certificate(remote_instruction, &config.account_server.certificate_public_keys)
                .await
            {
                Err(error @ RefreshCertificateError::Instruction(InstructionError::AccountRevoked(_))) => {
                    return Err(error);
                }
                Err(error) => {
                    warn!(
                        "Failed to refresh wallet certificate, will retry after the next PIN-signed instruction: \
                         {error}"
                    );
                }
                Ok(()) => {}
            }
        }

        Ok(())
    }

    async fn refresh_wallet_certificate(
        &mut self,
        remote_instruction: &InstructionClient<S, AKH::AppleKey, AKH::GoogleKey, APC>,
        certificate_public_keys: &HashMap<String, CertificatePublicKey>,
    ) -> Result<(), RefreshCertificateError>
    where
        UR: Repository<VersionState>,
        S: Storage,
        APC: AccountProviderClient,
    {
        info!("Refreshing Wallet certificate");

        let (_, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or(RefreshCertificateError::NotRegistered)?;

        // The current certificate is already trusted (it was itself verified upon receipt), so re-verifying it here
        // yields a trustworthy hardware public key and wallet ID to check the refreshed certificate against.
        let (_, current_claims) = registration_data
            .wallet_certificate
            .parse_and_verify_with_sub_by_kid(certificate_public_keys)
            .map_err(RefreshCertificateError::CertificateValidation)?;
        let hw_pubkey = *current_claims.hw_pubkey.as_inner();
        let wallet_id = registration_data.wallet_id.clone();

        let new_certificate = self
            .check_result_for_wallet_revocation(remote_instruction.send(RefreshWalletCertificate).await)
            .await
            .map_err(RefreshCertificateError::Instruction)?;

        // Verify that the refreshed certificate is signed by a key we trust, and that its contents are otherwise
        // identical to the certificate it replaces.
        let (_, new_claims) = new_certificate
            .parse_and_verify_with_sub_by_kid(certificate_public_keys)
            .map_err(RefreshCertificateError::CertificateValidation)?;

        if *new_claims.hw_pubkey.as_inner() != hw_pubkey {
            return Err(RefreshCertificateError::PublicKeyMismatch);
        }

        if new_claims.wallet_id != wallet_id {
            return Err(RefreshCertificateError::WalletIdMismatch);
        }

        let WalletRegistration::Registered { data, .. } = &mut self.registration else {
            return Err(RefreshCertificateError::NotRegistered);
        };
        data.wallet_certificate = new_certificate;

        self.storage
            .write()
            .await
            .upsert_data(data)
            .await
            .map_err(RefreshCertificateError::CertificateStorage)?;

        Ok(())
    }
}

fn certificate_needs_refresh(
    certificate: &WalletCertificate,
    certificate_public_keys: &HashMap<String, CertificatePublicKey>,
    refresh_threshold: Duration,
    time: &impl Generator<DateTime<Utc>>,
) -> bool {
    let Ok((header, claims)) = certificate.dangerous_parse_unverified() else {
        return false;
    };

    let now = time.generate();
    if claims.exp <= now + refresh_threshold {
        return true;
    }

    let Some(newest_kid) = certificate_public_keys
        .iter()
        .filter(|(_, key)| *key.used_from.as_ref() <= now)
        .max_by_key(|(_, key)| -> DateTimeSeconds { key.used_from })
        .map(|(kid, _)| kid)
    else {
        // if we arrive here, it means the list of public keys in the wallet configuration is empty or all keys are in
        // the future, this should never happen and means the configuration is invalid
        tracing::warn!("No valid certificate public key found in the wallet configuration");
        return false;
    };

    &header.kid != newest_kid
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::LazyLock;
    use std::time::Duration;

    use chrono::Utc;
    use crypto::utils::random_bytes;
    use futures::future::FutureExt;
    use jwt::KeyWithKid;
    use jwt::SignedJwt;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use platform_support::attested_key::AttestedKey;
    use rstest::rstest;
    use wallet_account::messages::errors::AccountError;
    use wallet_account::messages::instructions::CheckPin;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::RefreshWalletCertificate;

    use super::super::test::ACCOUNT_SERVER_KEYS;
    use super::super::test::TestWalletInMemoryStorage;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::create_wallet_configuration;
    use super::super::test::create_wp_result;
    use super::super::test::valid_certificate;
    use super::super::test::valid_certificate_claims;
    use super::*;
    use crate::account_provider::AccountProviderResponseError;
    use crate::pin::key::Pin;
    use crate::storage::RegistrationData;

    static PIN: LazyLock<Pin> = LazyLock::new(|| "051097".into());

    #[tokio::test]
    #[rstest]
    async fn test_wallet_lock_unlock_refreshes_certificate(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        // Add a key to the configuration that is newer than the one the stored certificate is signed with, so that
        // unlocking should trigger a RefreshWalletCertificate instruction.
        let mut config = create_wallet_configuration();
        config
            .account_server
            .certificate_public_keys
            .get_mut(ACCOUNT_SERVER_KEYS.certificate_signing_key.kid())
            .unwrap()
            .used_from = (Utc::now() - Duration::from_secs(60)).into();
        config.account_server.certificate_public_keys.insert(
            "newer".to_owned(),
            CertificatePublicKey {
                key: (*SigningKey::generate().verifying_key()).into(),
                used_from: (Utc::now() - Duration::from_secs(1)).into(),
            },
        );

        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked_with_config(vendor, config).await;
        wallet.lock();

        let (attested_key, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        let old_certificate = registration_data.wallet_certificate.clone();
        let wallet_id = registration_data.wallet_id.clone();
        let hw_pubkey = match attested_key.as_ref() {
            AttestedKey::Apple(key) => *key.verifying_key(),
            AttestedKey::Google(key) => *key.verifying_key(),
        };

        let new_certificate = valid_certificate(Some(wallet_id), hw_pubkey);
        let new_certificate_for_closure = new_certificate.clone();

        let challenge = random_bytes(32);
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .times(2)
            .returning(move |_, _| Ok(challenge.clone()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Ok(create_wp_result(())));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<RefreshWalletCertificate>| {
                Ok(create_wp_result(new_certificate_for_closure))
            });

        wallet.unlock(PIN.clone()).await.expect("should unlock wallet");

        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        assert_ne!(registration_data.wallet_certificate, old_certificate);
        assert_eq!(registration_data.wallet_certificate, new_certificate);

        let stored_registration_data = wallet
            .storage
            .read()
            .await
            .fetch_data::<RegistrationData>()
            .await
            .unwrap()
            .expect("registration data should be present in storage");
        assert_eq!(stored_registration_data.wallet_certificate, new_certificate);
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_lock_unlock_succeeds_when_certificate_refresh_failure(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        // Add a key to the configuration that is newer than the one the stored certificate is signed with, so that
        // unlocking should trigger a RefreshWalletCertificate instruction.
        let mut config = create_wallet_configuration();
        config
            .account_server
            .certificate_public_keys
            .get_mut(ACCOUNT_SERVER_KEYS.certificate_signing_key.kid())
            .unwrap()
            .used_from = (Utc::now() - Duration::from_secs(60)).into();
        config.account_server.certificate_public_keys.insert(
            "newer".to_owned(),
            CertificatePublicKey {
                key: (*SigningKey::generate().verifying_key()).into(),
                used_from: (Utc::now() - Duration::from_secs(1)).into(),
            },
        );

        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked_with_config(vendor, config).await;
        wallet.lock();

        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        let old_certificate = registration_data.wallet_certificate.clone();

        let challenge = random_bytes(32);
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .times(2)
            .returning(move |_, _| Ok(challenge.clone()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Ok(create_wp_result(())));

        // The refresh attempt fails with a transient server error, unrelated to the correctness of the PIN.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<RefreshWalletCertificate>| {
                Err(AccountProviderResponseError::Account(AccountError::Unexpected, None).into())
            });

        // Unlocking should still succeed: the user provided a correct PIN, and the failed refresh attempt is
        // simply retried on the next unlock.
        wallet.unlock(PIN.clone()).await.expect("should unlock wallet");

        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        assert_eq!(registration_data.wallet_certificate, old_certificate);
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_lock_unlock_succeeds_when_certificate_refresh_public_key_mismatch(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        // Add a key to the configuration that is newer than the one the stored certificate is signed with, so that
        // unlocking should trigger a RefreshWalletCertificate instruction.
        let mut config = create_wallet_configuration();
        config
            .account_server
            .certificate_public_keys
            .get_mut(ACCOUNT_SERVER_KEYS.certificate_signing_key.kid())
            .unwrap()
            .used_from = (Utc::now() - Duration::from_secs(60)).into();
        config.account_server.certificate_public_keys.insert(
            "newer".to_owned(),
            CertificatePublicKey {
                key: (*SigningKey::generate().verifying_key()).into(),
                used_from: (Utc::now() - Duration::from_secs(1)).into(),
            },
        );

        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked_with_config(vendor, config).await;
        wallet.lock();

        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        let old_certificate = registration_data.wallet_certificate.clone();
        let wallet_id = registration_data.wallet_id.clone();

        // The Wallet Provider returns a certificate that is validly signed and trusted, but bound to a hardware
        // public key that does not match the wallet's own attested key.
        let other_hw_pubkey = *SigningKey::generate().verifying_key();
        let new_certificate = valid_certificate(Some(wallet_id), other_hw_pubkey);
        let new_certificate_for_closure = new_certificate.clone();

        let challenge = random_bytes(32);
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .times(2)
            .returning(move |_, _| Ok(challenge.clone()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<CheckPin>| Ok(create_wp_result(())));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .return_once(move |_, _: Instruction<RefreshWalletCertificate>| {
                Ok(create_wp_result(new_certificate_for_closure))
            });

        // Unlocking should still succeed: the user provided a correct PIN, and the rejected refresh attempt (its
        // public key does not match the wallet's hardware key) is simply retried on the next unlock, rather than
        // blocking the user.
        wallet.unlock(PIN.clone()).await.expect("should unlock wallet");

        let (_, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        assert_eq!(registration_data.wallet_certificate, old_certificate);
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_lock_unlock_does_not_refresh_certificate_without_a_newer_key(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        // The default test configuration only contains the key that the stored certificate is already signed
        // with, so unlocking should not trigger a RefreshWalletCertificate instruction.
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(vendor).await;
        wallet.lock();

        let challenge = crypto::utils::random_bytes(32);
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .times(1)
            .returning(move |_, _| Ok(challenge.clone()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .times(1)
            .return_once(move |_, _: Instruction<CheckPin>| Ok(create_wp_result(())));

        wallet.unlock(PIN.clone()).await.expect("should unlock wallet");
    }

    #[test]
    fn test_certificate_needs_refresh() {
        let hw_pubkey = *SigningKey::generate().verifying_key();
        let certificate = valid_certificate(None, hw_pubkey);
        let current_kid = ACCOUNT_SERVER_KEYS.certificate_signing_key.kid().to_string();
        let current_public_key = CertificatePublicKey {
            key: (*ACCOUNT_SERVER_KEYS.certificate_signing_key.verifying_key()).into(),
            used_from: (Utc::now() - Duration::from_secs(20)).into(),
        };

        // The certificate's kid is the only (and therefore newest) key: no refresh needed.
        assert!(!certificate_needs_refresh(
            &certificate,
            &HashMap::from([(current_kid.clone(), current_public_key.clone())]),
            Duration::from_secs(60),
            &TimeGenerator,
        ));

        // A newer key than the certificate's kid is already in use: refresh needed.
        let newer_public_key = CertificatePublicKey {
            key: (*SigningKey::generate().verifying_key()).into(),
            used_from: (Utc::now() - Duration::from_secs(10)).into(),
        };
        assert!(certificate_needs_refresh(
            &certificate,
            &HashMap::from([
                (current_kid.clone(), current_public_key.clone()),
                ("newer".to_owned(), newer_public_key.clone()),
            ]),
            Duration::from_secs(60),
            &TimeGenerator,
        ));

        // The certificate's kid is still the newest, even though an older key is also present: no refresh needed.
        let older_public_key = CertificatePublicKey {
            key: (*SigningKey::generate().verifying_key()).into(),
            used_from: (Utc::now() - Duration::from_secs(30)).into(),
        };
        assert!(!certificate_needs_refresh(
            &certificate,
            &HashMap::from([
                (current_kid.clone(), current_public_key.clone()),
                ("older".to_owned(), older_public_key)
            ]),
            Duration::from_secs(60),
            &TimeGenerator,
        ));

        // A newer key is present in the configuration, but the wallet provider is not using it yet (its
        // `used_from` lies in the future): no refresh needed.
        let future_public_key = CertificatePublicKey {
            key: (*SigningKey::generate().verifying_key()).into(),
            used_from: (Utc::now() + Duration::from_secs(3600)).into(),
        };
        assert!(!certificate_needs_refresh(
            &certificate,
            &HashMap::from([
                (current_kid.clone(), current_public_key.clone()),
                ("future".to_owned(), future_public_key.clone()),
            ]),
            Duration::from_secs(60),
            &TimeGenerator,
        ));

        // The newer, already-in-use key is still preferred over one that isn't in use yet.
        assert!(certificate_needs_refresh(
            &certificate,
            &HashMap::from([
                (current_kid, current_public_key),
                ("newer".to_owned(), newer_public_key),
                ("future".to_owned(), future_public_key),
            ]),
            Duration::from_secs(60),
            &TimeGenerator,
        ));
    }

    #[test]
    fn test_certificate_needs_refresh_exp() {
        let hw_pubkey = *SigningKey::generate().verifying_key();
        let current_kid = ACCOUNT_SERVER_KEYS.certificate_signing_key.kid().to_string();
        let current_public_key = CertificatePublicKey {
            key: (*ACCOUNT_SERVER_KEYS.certificate_signing_key.verifying_key()).into(),
            used_from: (Utc::now() - Duration::from_secs(20)).into(),
        };
        let certificate_public_keys = HashMap::from([(current_kid, current_public_key)]);

        let mut claims = valid_certificate_claims(None, hw_pubkey);
        claims.exp = Utc::now() + Duration::from_secs(30);
        let certificate = SignedJwt::sign_with_sub_and_kid(claims, &ACCOUNT_SERVER_KEYS.certificate_signing_key)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into();

        // The certificate's kid is up to date, but its `exp` lies within the refresh threshold: refresh needed.
        assert!(certificate_needs_refresh(
            &certificate,
            &certificate_public_keys,
            Duration::from_secs(60),
            &TimeGenerator,
        ));

        // The certificate's kid is up to date and its `exp` lies outside the refresh threshold: no refresh
        // needed.
        assert!(!certificate_needs_refresh(
            &certificate,
            &certificate_public_keys,
            Duration::from_secs(10),
            &TimeGenerator,
        ));
    }
}
