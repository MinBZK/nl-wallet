use std::error::Error;
use std::sync::Arc;

use tracing::info;
use tracing::instrument;
use tracing::warn;

use crypto::keys::EcdsaKey;
use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::tls::pinning::TlsPinningConfig;
use jwt::error::JwtError;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::attested_key::KeyWithAttestation;
use platform_support::attested_key::hardware::AttestedKeyError;
use platform_support::attested_key::hardware::HardwareAttestedKeyError;
use update_policy_model::update_policy::VersionState;
use utils::vec_at_least::VecAtLeastNError;
use wallet_account::messages::registration::Registration;
use wallet_account::signed::ChallengeResponse;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::account_provider::AccountProviderError;
use crate::digid::DigidClient;
use crate::errors::UpdatePolicyError;
use crate::pin::key::PinKey;
use crate::pin::key::{self as pin_key};
use crate::pin::validation::PinValidationError;
use crate::pin::validation::validate_pin;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::KeyData;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::storage::StorageError;

use super::Wallet;
use super::WalletRegistration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletRegistrationError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is already registered")]
    #[category(expected)]
    AlreadyRegistered,
    #[error("PIN provided for registration does not adhere to requirements: {0}")]
    InvalidPin(#[source] PinValidationError),
    #[error("could not request registration challenge from Wallet Provider: {0}")]
    ChallengeRequest(#[source] AccountProviderError),
    #[error("could not generate attested key: {0}")]
    #[category(pd)]
    KeyGeneration(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not perform key and/or app attestation: {0}")]
    #[category(pd)]
    Attestation(#[source] Box<dyn Error + Send + Sync>),
    #[error("certificate chain for Android key attestation has too few entries: {0}")]
    #[category(critical)]
    AndroidCertificateChain(#[source] VecAtLeastNError),
    #[category(pd)]
    #[error("could not get attested public key: {0}")]
    AttestedPublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not sign registration message: {0}")]
    Signing(#[source] wallet_account::error::EncodeError),
    #[error("could not request registration from Wallet Provider: {0}")]
    RegistrationRequest(#[source] AccountProviderError),
    #[error("could not validate registration certificate received from Wallet Provider: {0}")]
    CertificateValidation(#[source] JwtError),
    #[error("public key in registration certificate received from Wallet Provider does not match hardware public key")]
    #[category(expected)]
    PublicKeyMismatch,
    #[error("could not store registration state in database: {0}")]
    StoreRegistrationState(#[source] StorageError),
    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
}

impl WalletRegistrationError {
    pub fn is_attestation_not_supported(&self) -> bool {
        match self {
            Self::KeyGeneration(error) | Self::Attestation(error) => {
                matches!(
                    error.downcast_ref::<HardwareAttestedKeyError>(),
                    Some(HardwareAttestedKeyError::Platform(
                        AttestedKeyError::AttestationNotSupported
                    ))
                )
            }
            _ => false,
        }
    }
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    pub fn has_registration(&self) -> bool {
        self.registration.is_registered()
    }

    async fn set_registration_key_identifier(&mut self, key_identifier: String) -> Result<(), StorageError>
    where
        S: Storage,
    {
        let mut storage = self.storage.write().await;
        storage.open_if_needed().await?;

        let key_data = KeyData {
            identifier: key_identifier.clone(),
        };
        storage.insert_data(&key_data).await?;

        self.registration = WalletRegistration::KeyIdentifierGenerated(key_identifier);

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn register(&mut self, pin: &str) -> Result<(), WalletRegistrationError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
    {
        let config = &self.config_repository.get();

        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(WalletRegistrationError::VersionBlocked);
        }

        info!("Checking if already registered");
        // Registration is only allowed if we do not currently have a registration on record.
        if self.has_registration() {
            return Err(WalletRegistrationError::AlreadyRegistered);
        }

        info!("Validating PIN");

        // Make sure the PIN adheres to the requirements.
        // TODO: do not keep PIN in memory while request is in flight (PVW-1290)
        validate_pin(pin).map_err(WalletRegistrationError::InvalidPin)?;

        info!("Requesting challenge from account server");

        // Retrieve a challenge from the account server
        let challenge = self
            .account_provider_client
            .registration_challenge(&config.account_server.http_config)
            .await
            .map_err(WalletRegistrationError::ChallengeRequest)?;

        info!("Challenge received from account server, generating attested key");

        // If a reusable key identifier was already generated, use that instead of generating a new one.
        let key_identifier = match &self.registration {
            WalletRegistration::Unregistered => self
                .key_holder
                .generate()
                .await
                .map_err(|error| WalletRegistrationError::KeyGeneration(Box::new(error)))?,
            WalletRegistration::KeyIdentifierGenerated(key_identifier) => key_identifier.clone(),
            // This variant is not possible, as checked by self.has_registration() above.
            WalletRegistration::Registered { .. } => unreachable!(),
        };

        info!("Performing key and app attestation");

        let key_with_attestation_result = self
            .key_holder
            .attest(
                key_identifier.clone(),
                crypto::utils::sha256(&challenge),
                config.google_cloud_project_number,
            )
            .await;

        let key_with_attestation = match key_with_attestation_result {
            Ok(key_with_attestation) => key_with_attestation,
            Err(error) => {
                // If the error indicates attestation is retryable and we did not do do so already,
                // store the key identifier for later re-use, logging any potential errors.
                if error.retryable
                    && matches!(self.registration, WalletRegistration::Unregistered)
                    && let Err(storage_error) = self.set_registration_key_identifier(key_identifier.clone()).await
                {
                    warn!("Could not store attested key identifier: {0}", storage_error);
                }

                return Err(WalletRegistrationError::Attestation(Box::new(error.error)));
            }
        };

        // If the attestation was successful, we should probably also store the key identifier
        // for potential re-use, i.e. if registration fails. This would prevent creating too
        // many keys on the device, as the Apple documentation recommends. However, performing
        // attestation again using the same key identifier results in an error, which may be
        // caused by the fact that we use the key to sign the registration message immediately
        // after performing attestation.
        //
        // The fact that a succesfully attested key cannot be re-used also means we should
        // clean up any lingering key identifier that is stored at this point.

        if matches!(self.registration, WalletRegistration::KeyIdentifierGenerated(_)) {
            self.storage
                .write()
                .await
                .delete_data::<KeyData>()
                .await
                .map_err(WalletRegistrationError::StoreRegistrationState)?;
            self.registration = WalletRegistration::Unregistered;
        }

        info!("Key and app attestation successful, signing and sending registration to account server");

        // Create a registration message and double sign it with the challenge.
        // Generate a new PIN salt and derive the private key from the provided PIN.
        let pin_salt = pin_key::new_pin_salt();
        let pin_key = PinKey { pin, salt: &pin_salt };

        // Sign the registration message based on the attestation type.
        let (registration_message, attested_key) = match key_with_attestation {
            KeyWithAttestation::Apple { key, attestation_data } => {
                ChallengeResponse::<Registration>::new_apple(&key, attestation_data, &pin_key, challenge)
                    .await
                    .map(|message| (message, AttestedKey::Apple(key)))
            }
            // TODO: Remove into() from app_attestation_token.into() once we merge wallet_provider stuff.
            KeyWithAttestation::Google {
                key,
                certificate_chain,
                app_attestation_token,
            } => ChallengeResponse::<Registration>::new_google(
                &key,
                certificate_chain
                    .try_into()
                    .map_err(WalletRegistrationError::AndroidCertificateChain)?,
                app_attestation_token,
                &pin_key,
                challenge,
            )
            .await
            .map(|message| (message, AttestedKey::Google(key))),
        }
        .map_err(WalletRegistrationError::Signing)?;

        // Send the registration message to the account server and receive the wallet certificate in response.
        let wallet_certificate = self
            .account_provider_client
            .register(&config.account_server.http_config, registration_message)
            .await
            .map_err(WalletRegistrationError::RegistrationRequest)?;

        info!("Certificate received from account server, verifying contents");

        // Double check that the public key returned in the wallet certificate matches that of our hardware key.
        // Note that this public key is only available on Android, on iOS all we have is opaque attestation data.
        let cert_claims = wallet_certificate
            .parse_and_verify_with_sub(&config.account_server.certificate_public_key.as_inner().into())
            .map_err(WalletRegistrationError::CertificateValidation)?;

        if let AttestedKey::Google(key) = &attested_key {
            let attested_pub_key = key
                .verifying_key()
                .await
                .map_err(|error| WalletRegistrationError::AttestedPublicKey(Box::new(error)))?;

            if cert_claims.hw_pubkey.as_inner() != &attested_pub_key {
                return Err(WalletRegistrationError::PublicKeyMismatch);
            }
        }

        info!("Storing received registration");

        let mut storage = self.storage.write().await;
        storage
            .open_if_needed()
            .await
            .map_err(WalletRegistrationError::StoreRegistrationState)?;

        // Save the registration data in storage.
        let data = RegistrationData {
            attested_key_identifier: key_identifier,
            wallet_id: cert_claims.wallet_id,
            pin_salt,
            wallet_certificate,
        };
        storage
            .insert_data(&data)
            .await
            .map_err(WalletRegistrationError::StoreRegistrationState)?;

        // Keep the registration data in memory.
        self.registration = WalletRegistration::Registered {
            attested_key: Arc::new(attested_key),
            data,
        };

        // Unlock the wallet after successful registration
        self.lock.unlock();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use http::StatusCode;
    use p256::ecdsa::SigningKey;
    use parking_lot::Mutex;
    use rand_core::OsRng;
    use rstest::rstest;

    use apple_app_attest::AssertionCounter;
    use apple_app_attest::VerifiedAttestation;
    use crypto::x509::BorrowingCertificate;
    use jwt::Jwt;
    use platform_support::attested_key::mock::KeyHolderErrorScenario;
    use platform_support::attested_key::mock::KeyHolderType;
    use wallet_account::messages::registration::RegistrationAttestation;
    use wallet_account::messages::registration::WalletCertificate;
    use wallet_account::signed::SequenceNumberComparison;

    use crate::account_provider::AccountProviderResponseError;
    use crate::wallet::test::WalletWithDefaultStorage;
    use crate::wallet::test::WalletWithStorage;
    use crate::wallet::test::valid_certificate;
    use crate::wallet::test::valid_certificate_claims;

    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::*;

    const PIN: &str = "051097";

    async fn test_register_success(wallet: &mut WalletWithStorage) {
        // The wallet should report that it is currently unregistered and locked.
        assert!(!wallet.has_registration());
        assert!(wallet.is_locked());

        // Have the account server respond with a random
        // challenge when the wallet sends a request for it.
        let challenge = crypto::utils::random_bytes(32);
        let challenge_response = challenge.clone();

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_registration_challenge()
            .return_once(|_| Ok(challenge_response));

        // Have the account server respond with a valid
        // certificate when the wallet sends a request for it.
        let challenge_expected = challenge.clone();
        let holder_data = match &wallet.key_holder.holder_type {
            KeyHolderType::Apple {
                ca,
                environment,
                app_identifier,
            } => Some((ca.trust_anchor().to_owned(), *environment, app_identifier.clone())),
            KeyHolderType::Google { .. } => None,
        };

        // Set up a mutex for the mock callback to write the generated wallet certificate to.
        let generated_certificate: Arc<Mutex<Option<WalletCertificate>>> = Arc::new(Mutex::new(None));
        let generated_certificate_clone = Arc::clone(&generated_certificate);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_register()
            .return_once(move |_, registration_signed| {
                let registration = registration_signed
                    .dangerous_parse_unverified()
                    .expect("registration message should parse");

                assert_eq!(registration.challenge, challenge_expected);

                let attested_public_key = match (registration.payload.attestation, holder_data) {
                    (
                        RegistrationAttestation::Apple { data: attestation_data },
                        Some((trust_anchor, environment, app_identifier)),
                    ) => {
                        // Verify the mock attestation in order to get the public key.
                        let (_, attested_public_key) = VerifiedAttestation::parse_and_verify(
                            &attestation_data,
                            &[trust_anchor],
                            &crypto::utils::sha256(&registration.challenge),
                            &app_identifier,
                            environment,
                        )
                        .expect("registration message Apple attestation should verify");

                        // Verify the registration message, both counters start at 0.
                        registration_signed
                            .parse_and_verify_apple(
                                &registration.challenge,
                                SequenceNumberComparison::EqualTo(0),
                                &attested_public_key,
                                &app_identifier,
                                AssertionCounter::default(),
                                registration.payload.pin_pubkey.as_inner(),
                            )
                            .expect("registration message should verify");

                        attested_public_key
                    }
                    (RegistrationAttestation::Google { certificate_chain, .. }, None) => {
                        // For now, just extract the verifying key from the first X.509 certificate in the chain.
                        let leaf_certificate = BorrowingCertificate::from_der(certificate_chain.into_first())
                            .expect("leaf should be a valid X.509 certificate");

                        *leaf_certificate.public_key()
                    }
                    _ => {
                        panic!("registration message should contain attestation for the correct platform");
                    }
                };

                // Generate a valid certificate and wallet id based on on the public key.
                let certificate = valid_certificate(None, attested_public_key);
                generated_certificate_clone.lock().replace(certificate.clone());

                Ok(certificate)
            });

        // Register the wallet with a valid PIN.
        wallet.register(PIN).await.expect("Could not register wallet");

        // The wallet should now report that it is registered and unlocked.
        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());

        // A re-usable key identifier should not be stored in the database.
        let storage = wallet.storage.read().await;
        assert!(storage.fetch_data::<KeyData>().await.unwrap().is_none());

        // The registration should be stored in the database.
        let stored_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .unwrap()
            .expect("Registration data not present in storage");
        assert_eq!(
            stored_registration.wallet_certificate.0,
            generated_certificate.lock().as_ref().unwrap().0
        );
        assert!(
            wallet
                .key_holder
                .is_attested(&stored_registration.attested_key_identifier)
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_success(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithStorage::new_unregistered(vendor).await;

        test_register_success(&mut wallet).await;
    }

    async fn add_key_identifier_to_wallet(wallet: &mut WalletWithStorage) -> String {
        let key_identifier = wallet.key_holder.generate().await.unwrap();
        wallet
            .storage
            .write()
            .await
            .insert_data(&KeyData {
                identifier: key_identifier.clone(),
            })
            .await
            .unwrap();
        wallet.registration = WalletRegistration::KeyIdentifierGenerated(key_identifier.clone());

        key_identifier
    }

    #[tokio::test]
    async fn test_wallet_register_success_apple_key_identifier() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        // Set up a key identifier to re-use, both in storage and in the Wallet internal state.
        let key_identifier = add_key_identifier_to_wallet(&mut wallet).await;

        test_register_success(&mut wallet).await;

        // The same key identifier we generated should now be attested.
        assert!(wallet.key_holder.is_attested(&key_identifier));
    }

    #[tokio::test]
    async fn test_wallet_register_error_already_registered() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::AlreadyRegistered);
        assert!(wallet.has_registration());
    }

    #[tokio::test]
    async fn test_wallet_register_error_invalid_pin() {
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Try to register with an insecure PIN.
        let error = wallet
            .register("123456")
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::InvalidPin(_));
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
    }

    #[tokio::test]
    async fn test_wallet_register_error_challenge_request() {
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Have the account server respond to the challenge request with a 500 error.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_registration_challenge()
            .return_once(|_| Err(AccountProviderResponseError::Status(StatusCode::INTERNAL_SERVER_ERROR).into()));

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::ChallengeRequest(_));
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
    }

    async fn unregistered_wallet_with_registration_challenge(vendor: WalletDeviceVendor) -> WalletWithStorage {
        let mut wallet = WalletWithStorage::new_unregistered(vendor).await;

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_registration_challenge()
            .return_once(|_| Ok(crypto::utils::random_bytes(32)));

        wallet
    }

    async fn test_wallet_register_error_key_holder(
        vendor: WalletDeviceVendor,
        error_scenario: KeyHolderErrorScenario,
    ) -> WalletRegistrationError {
        let mut wallet = unregistered_wallet_with_registration_challenge(vendor).await;

        // Have the key holder fail in some way.
        wallet.key_holder.error_scenario = error_scenario;

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
        assert!(
            wallet
                .storage
                .read()
                .await
                .fetch_data::<RegistrationData>()
                .await
                .unwrap()
                .is_none()
        );

        error
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_error_key_generation(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        assert_matches!(
            test_wallet_register_error_key_holder(vendor, KeyHolderErrorScenario::GenerateError).await,
            WalletRegistrationError::KeyGeneration(_)
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_error_attestation_unretryable(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        assert_matches!(
            test_wallet_register_error_key_holder(vendor, KeyHolderErrorScenario::UnretryableAttestationError).await,
            WalletRegistrationError::Attestation(_)
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_error_signing(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        assert_matches!(
            test_wallet_register_error_key_holder(vendor, KeyHolderErrorScenario::SigningError).await,
            WalletRegistrationError::Signing(_)
        );
    }

    #[tokio::test]
    async fn test_wallet_register_error_attestation_retryable() {
        let mut wallet = unregistered_wallet_with_registration_challenge(WalletDeviceVendor::Apple).await;

        // Have the hardware key signing fail.
        wallet.key_holder.error_scenario = KeyHolderErrorScenario::RetryableAttestationError;

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::Attestation(_));

        // The storage should contain a key identifier for re-use, but not a registration.
        assert!(!wallet.has_registration());
        let key_data: KeyData = wallet
            .storage
            .read()
            .await
            .fetch_data()
            .await
            .unwrap()
            .expect("KeyData data not present in storage");
        assert_matches!(
            wallet.registration,
            WalletRegistration::KeyIdentifierGenerated(identifier) if identifier == key_data.identifier
        );
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_error_registration_request(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        let mut wallet = unregistered_wallet_with_registration_challenge(vendor).await;

        // Have the account server respond to the registration request with a 401 error.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_register()
            .return_once(|_, _| Err(AccountProviderResponseError::Status(StatusCode::UNAUTHORIZED).into()));

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::RegistrationRequest(_));
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
        assert!(
            wallet
                .storage
                .read()
                .await
                .fetch_data::<RegistrationData>()
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_wallet_register_error_registration_request_key_identifier() {
        let mut wallet = unregistered_wallet_with_registration_challenge(WalletDeviceVendor::Apple).await;

        // Set up a key identifier to re-use, both in storage and in the Wallet internal state.
        // This key identifier is no longer re-usable after attestation, so if registration gets as far
        // as sending the registration message to the Wallet Provider, the key identifier should be removed.
        let key_identifier = add_key_identifier_to_wallet(&mut wallet).await;

        // Have the account server respond to the registration request with a 401 error.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_register()
            .return_once(|_, _| Err(AccountProviderResponseError::Status(StatusCode::UNAUTHORIZED).into()));

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        // The Wallet should not be registered, but the the key referenced
        // by the identifier should be attested and is further unusable.
        assert_matches!(error, WalletRegistrationError::RegistrationRequest(_));
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
        assert!(
            wallet
                .storage
                .read()
                .await
                .fetch_data::<RegistrationData>()
                .await
                .unwrap()
                .is_none()
        );
        assert!(wallet.key_holder.is_attested(&key_identifier));
    }

    #[tokio::test]
    #[rstest]
    async fn test_wallet_register_error_certificate_validation(
        #[values(WalletDeviceVendor::Apple, WalletDeviceVendor::Google)] vendor: WalletDeviceVendor,
    ) {
        let mut wallet = unregistered_wallet_with_registration_challenge(vendor).await;

        // Have the account server sign the wallet certificate with
        // a key to which the certificate public key does not belong.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_register()
            .return_once(|_, _| {
                let other_account_server_key = SigningKey::random(&mut OsRng);
                let random_pubkey = *SigningKey::random(&mut OsRng).verifying_key();

                let certificate = Jwt::sign_with_sub(
                    &valid_certificate_claims(None, random_pubkey),
                    &other_account_server_key,
                )
                .now_or_never()
                .unwrap()
                .unwrap();

                Ok(certificate)
            });

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::CertificateValidation(_));
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
        assert!(
            wallet
                .storage
                .read()
                .await
                .fetch_data::<RegistrationData>()
                .await
                .unwrap()
                .is_none()
        );
    }

    fn expect_register_with_random_pubkey<S>(wallet: &mut WalletWithDefaultStorage<S>) {
        // Have the account server respond with a certificate that contains
        // a public key that does not belong to the wallet's attested key.
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_register()
            .return_once(|_, _| {
                let random_pubkey = *SigningKey::random(&mut OsRng).verifying_key();
                let certificate = valid_certificate(None, random_pubkey);

                Ok(certificate)
            });
    }

    #[tokio::test]
    async fn test_wallet_register_error_public_key_mismatch() {
        let mut wallet = unregistered_wallet_with_registration_challenge(WalletDeviceVendor::Google).await;

        expect_register_with_random_pubkey(&mut wallet);

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::PublicKeyMismatch);
        assert_matches!(wallet.registration, WalletRegistration::Unregistered);
        assert!(
            wallet
                .storage
                .read()
                .await
                .fetch_data::<RegistrationData>()
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_wallet_register_error_store_certificate() {
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_registration_challenge()
            .return_once(|_| Ok(crypto::utils::random_bytes(32)));

        expect_register_with_random_pubkey(&mut wallet);

        wallet.mut_storage().expect_open_if_needed().return_once(|| Ok(()));

        // Have the database return an error
        // when inserting the wallet certificate.
        wallet
            .mut_storage()
            .expect_insert_data::<RegistrationData>()
            .returning(|_| Err(StorageError::AlreadyOpened));

        let error = wallet
            .register(PIN)
            .await
            .expect_err("Wallet registration should have resulted in error");

        assert_matches!(error, WalletRegistrationError::StoreRegistrationState(_));
        assert!(!wallet.has_registration());

        wallet.mut_storage().checkpoint();
    }
}
