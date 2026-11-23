use std::collections::HashSet;
use std::fmt::Debug;
use std::num::NonZeroU8;
use std::sync::Arc;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::urls;
use itertools::Itertools;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::CredentialSelection;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceDiscoveryParameters;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::WalletIssuanceError;
use openid4vc::wallet_issuance::authorization::OAuthError;
use p256::ecdsa::VerifyingKey;
use platform_support::attested_key::AttestedKeyHolder;
use tracing::info;
use tracing::instrument;
use update_policy_model::update_policy::VersionState;
use url::Url;
use utils::generator::TimeGenerator;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::IssuanceError;
use super::PersistedPinRecoverySessionData;
use super::Session;
use super::Wallet;
use super::WalletRegistration;
use super::issuance::PidAttestationFormat;
use super::recovery_code::RecoveryCodeError;
use crate::account_provider::AccountProviderClient;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::errors::InstructionError;
use crate::errors::PinKeyError;
use crate::errors::PinValidationError;
use crate::errors::StorageError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionClientParameters;
use crate::instruction::PinRecoveryRemoteEcdsaWscd;
use crate::instruction::PinRecoveryWscd;
use crate::pin::key::Pin;
use crate::pin::key::PinKey;
use crate::pin::key::new_pin_salt;
use crate::repository::Repository;
use crate::storage::PinRecoveryData;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::validate_pin;
use crate::wallet::CheckPreconditionsError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum PinRecoveryError {
    #[error("preconditions failed: {0}")]
    CheckPreconditions(#[source] CheckPreconditionsError),

    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,

    #[error("pin recovery session is not in the correct state")]
    #[category(expected)]
    SessionState,

    #[error("error during PID issuance: {0}")]
    Issuance(#[source] IssuanceError),

    #[error("the new PIN does not adhere to requirements: {0}")]
    #[category(expected)]
    PinValidation(#[source] PinValidationError),

    #[error("error computing PIN public key: {0}")]
    // This error cannot actually occur, as deriving the verifying key from the PIN should never fail.
    #[category(impossible)]
    PinKey(#[source] PinKeyError),

    #[error("storage error: {0}")]
    Storage(#[source] StorageError),

    #[error("failed to disclose recovery code to WP: {0}")]
    DiscloseRecoveryCode(#[source] InstructionError),

    #[error("user denied authentication")]
    #[category(expected)]
    AuthorizationDenied,

    #[error("cannot recover PIN without a PID")]
    #[category(critical)]
    NoPidPresent,

    #[error("recovery code error: {0}")]
    RecoveryCode(#[source] RecoveryCodeError),
}

#[derive(Debug)]
pub enum PinRecoverySession<AS, IS> {
    OAuth { authorization_session: AS },
    Issuance { issuance_session: IS },
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pin_recovery_redirect_uri(&mut self) -> Result<Url, PinRecoveryError> {
        info!("Generating OAuth URL, starting issuer discovery");

        self.check_config_preconditions()
            .map_err(PinRecoveryError::CheckPreconditions)?;

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if a pid is present");
        let config = &self.config_repository.get();

        let has_pid = self
            .has_pid(&config.pid_attributes, PidAttestationFormat::SdJwt)
            .await
            .map_err(PinRecoveryError::Storage)?;

        if !has_pid {
            return Err(PinRecoveryError::NoPidPresent);
        }

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(PinRecoveryError::SessionState);
        }

        // No need to check if a `PinRecoveryData` is already stored: we can always start PIN recovery again.

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| PinRecoveryError::NotRegistered)?;

        info!("Fetching issuer metadata to discover authorization server");
        let credential_kinds = config.pid_attributes.sd_jwt_credential_kinds().collect::<HashSet<_>>();
        let authorization_session = self
            .issuance_discovery
            .start_authorization_code_flow(
                IssuanceDiscoveryParameters::new(
                    &config.pid_credential_offer,
                    CredentialSelection::ByCredentialKind(&credential_kinds),
                    &self.new_remote_wia_client(attested_key.to_owned(), registration_data, config),
                    config.wrpac_trust_anchors(),
                    config.wrprc_trust_anchors(),
                ),
                String::from(NL_WALLET_CLIENT_ID),
                urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner(),
            )
            .await
            .map_err(IssuanceError::IssuanceSession)
            .map_err(PinRecoveryError::Issuance)?;

        info!("PIN recovery OAuth URL generated");
        let auth_url = authorization_session.auth_url().clone();
        self.storage
            .write()
            .await
            .upsert_data(&PersistedPinRecoverySessionData {
                authorization_session: authorization_session.persist(),
            })
            .await
            .map_err(PinRecoveryError::Storage)?;
        self.session.replace(Session::PinRecovery(PinRecoverySession::OAuth {
            authorization_session,
        }));

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pin_recovery(&mut self, redirect_uri: Url) -> Result<(), PinRecoveryError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        self.check_config_preconditions()
            .map_err(PinRecoveryError::CheckPreconditions)?;

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| PinRecoveryError::NotRegistered)?;

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active PIN recovery session");
        if !matches!(
            self.session,
            Some(Session::PinRecovery(PinRecoverySession::OAuth { .. }))
        ) {
            return Err(PinRecoveryError::SessionState);
        }

        self.storage
            .write()
            .await
            .delete_data::<PersistedPinRecoverySessionData<<CID::Authorization as AuthorizationSession>::Persisted>>()
            .await
            .map_err(PinRecoveryError::Storage)?;

        let Some(Session::PinRecovery(PinRecoverySession::OAuth { authorization_session })) = self.session.take()
        else {
            return Err(PinRecoveryError::SessionState);
        };

        let config = self.config_repository.get();

        let issuance_session = authorization_session
            .start_issuance(
                &redirect_uri,
                &self.new_remote_wia_client(Arc::clone(attested_key), registration_data, &config),
            )
            .await
            .map_err(|e| match e {
                WalletIssuanceError::OAuth(OAuthError::Denied) => PinRecoveryError::AuthorizationDenied,
                e => PinRecoveryError::Issuance(IssuanceError::IssuanceSession(e)),
            })?;

        info!("successfully received token from issuer");

        self.session
            .replace(Session::PinRecovery(PinRecoverySession::Issuance { issuance_session }));

        // After this function is done, the user is asked to choose their new PIN.
        // Therefore, regardless of the state of `self.lock` the app is effectively accessible to whoever holds the
        // phone in their hands: all they have to is enter any PIN and then they can access the app.
        // Whether or not the app is locked according to `self.lock` therefore does not matter at this point.
        // Therefore we set it to unlocked, so that the inactivity timer in Flutter will fire if the user is
        // inactive for too long. If they are, and the app is locked again, the PinRecovery session is cleared
        // so that they have to start PIN recovery again.
        self.lock.unlock();

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn complete_pin_recovery(&mut self, new_pin: Pin) -> Result<(), PinRecoveryError> {
        self.complete_pin_recovery_internal(
            |instruction_client, pin_pubkey| PinRecoveryRemoteEcdsaWscd::new(instruction_client, pin_pubkey),
            new_pin,
        )
        .await
    }

    #[instrument(skip_all)]
    async fn complete_pin_recovery_internal<P, F>(
        &mut self,
        pin_recovery_wscd_factory: F,
        new_pin: Pin,
    ) -> Result<(), PinRecoveryError>
    where
        P: PinRecoveryWscd,
        F: FnOnce(
            InstructionClient<S, <AKH as AttestedKeyHolder>::AppleKey, <AKH as AttestedKeyHolder>::GoogleKey, APC>,
            VerifyingKey,
        ) -> P,
    {
        self.check_config_preconditions()
            .map_err(PinRecoveryError::CheckPreconditions)?;

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or(PinRecoveryError::NotRegistered)?;
        let attested_key = Arc::clone(attested_key);

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active issuance session");
        if !matches!(
            self.session,
            Some(Session::PinRecovery(PinRecoverySession::Issuance { .. }))
        ) {
            return Err(PinRecoveryError::SessionState);
        }

        let Some(Session::PinRecovery(PinRecoverySession::Issuance { mut issuance_session })) = self.session.take()
        else {
            return Err(PinRecoveryError::SessionState);
        };

        validate_pin(&new_pin).map_err(PinRecoveryError::PinValidation)?;

        // All sanity checks are done, we proceed with recovery.

        let new_pin_salt = new_pin_salt();
        let pin_pubkey = PinKey {
            pin: &new_pin,
            salt: &new_pin_salt,
        }
        .verifying_key()
        .map_err(PinRecoveryError::PinKey)?;

        let registration_data = RegistrationData {
            pin_salt: new_pin_salt.clone(),
            ..registration_data.clone()
        };

        let config = self.config_repository.get();
        let instruction_client = self
            .new_instruction_client(
                new_pin.clone(),
                Arc::clone(&attested_key),
                InstructionClientParameters::new(
                    registration_data.wallet_id.clone(),
                    registration_data.pin_salt.clone(),
                    registration_data.wallet_certificate.clone(),
                    config.account_server.http_config.clone(),
                    config.account_server.instruction_result_public_keys.clone(),
                ),
            )
            .await
            .map_err(IssuanceError::ChangePin)
            .map_err(PinRecoveryError::Issuance)?;

        let pin_recovery_wscd = pin_recovery_wscd_factory(instruction_client, pin_pubkey);

        // Accept issuance to obtain only the SD-JWT PID. This sends the `StartPinRecovery` instruction to the WP.
        // `accept_issuance()` below is the point of no return. If the app is killed between there and completion,
        // PIN recovery will have to start again from the start.
        self.storage
            .write()
            .await
            .upsert_data(&PinRecoveryData)
            .await
            .map_err(PinRecoveryError::Storage)?;

        // Since all we need is to extract the recovery code, request just a single copy of the SD-JWT PID credential.
        let issuance_result = issuance_session
            .accept_issuance(
                NonZeroU8::MIN,
                config.accept_issuance_anchors(),
                &pin_recovery_wscd,
                &TimeGenerator,
            )
            .await
            .map_err(|error| Self::handle_accept_issuance_error(error, &issuance_session));

        let issuance_result = match issuance_result {
            Err(error @ IssuanceError::Instruction(InstructionError::AccountRevoked(data))) => {
                self.handle_wallet_revocation(data).await;
                Err(PinRecoveryError::Issuance(error))?
            }
            _ => issuance_result.map_err(PinRecoveryError::Issuance)?,
        };

        // Check the recovery code in the received PID against the one in the stored PID, as on a mismatch the Wallet
        // Provider revokes all wallets of both persons.
        let recovery_code_disclosure = self
            .checked_recovery_code_disclosure(&issuance_result, &config.pid_attributes)
            .await
            .map_err(PinRecoveryError::RecoveryCode)?;

        // Store the new wallet certificate and the new salt.

        let new_wallet_certificate = pin_recovery_wscd
            .certificates()
            .into_iter()
            .exactly_one()
            .expect("expected exactly one certificate");

        let registration_data = RegistrationData {
            wallet_certificate: new_wallet_certificate,
            pin_salt: new_pin_salt,
            ..registration_data.clone()
        };
        self.storage
            .write()
            .await
            .upsert_data(&registration_data)
            .await
            .map_err(PinRecoveryError::Storage)?;

        self.registration = WalletRegistration::Registered {
            attested_key: Arc::clone(&attested_key),
            data: registration_data.clone(),
        };

        // Finish PIN recovery by sending the second WP instruction.

        // Use a new instruction client that uses our new WP certificate
        let result = InstructionClient::new(
            new_pin,
            Arc::clone(&self.storage),
            attested_key,
            Arc::clone(&self.account_provider_client),
            Arc::new(InstructionClientParameters::new(
                registration_data.wallet_id.clone(),
                registration_data.pin_salt.clone(),
                registration_data.wallet_certificate.clone(),
                config.account_server.http_config.clone(),
                config.account_server.instruction_result_public_keys.clone(),
            )),
        )
        .send(DiscloseRecoveryCodePinRecovery {
            recovery_code_disclosure: recovery_code_disclosure.into(),
        })
        .await;

        self.check_result_for_wallet_revocation(result)
            .await
            .map_err(PinRecoveryError::DiscloseRecoveryCode)?;

        self.storage
            .write()
            .await
            .delete_data::<PinRecoveryData>()
            .await
            .map_err(PinRecoveryError::Storage)?;

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_pin_recovery(&mut self) -> Result<(), PinRecoveryError> {
        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        // We don't check if the wallet is blocked: PIN recovery is allowed in that case, so cancelling it is too.

        let Some(Session::PinRecovery(_)) = self.session.take() else {
            return Err(PinRecoveryError::SessionState);
        };

        self.storage
            .write()
            .await
            .delete_data::<PersistedPinRecoverySessionData<<CID::Authorization as AuthorizationSession>::Persisted>>()
            .await
            .map_err(PinRecoveryError::Storage)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;
    use std::num::NonZeroU8;
    use std::str::FromStr;
    use std::sync::Arc;

    use attestation_data::attributes::Attribute;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::validity::ValidityWindow;
    use attestation_types::claim_path::ClaimPath;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKindRef;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_FAMILY_NAME;
    use attestation_types::pid_constants::PID_RECOVERY_CODE;
    use jwt::UnverifiedJwt;
    use jwt::nonce::Nonce;
    use mockall::predicate::eq;
    use openid4vc::wallet_issuance::CredentialSelection;
    use openid4vc::wallet_issuance::WalletIssuanceError;
    use openid4vc::wallet_issuance::authorization::OAuthError;
    use openid4vc::wallet_issuance::credential::IssuedCredentialMetadata;
    use openid4vc::wallet_issuance::issuer_registration::IssuerRegistration;
    use openid4vc::wallet_issuance::mock::MockAuthorizationSession;
    use openid4vc::wallet_issuance::mock::MockAuthorizationSessionData;
    use openid4vc::wallet_issuance::mock::MockIssuanceSession;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use sd_jwt::sd_jwt::DecodedSdJwt;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;
    use uuid::Uuid;
    use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::registration::WalletCertificate;
    use wscd::issuance::IssuanceKeyResult;
    use wscd::issuance::IssuanceWscd;

    use super::PinRecoveryError;
    use super::PinRecoverySession;
    use crate::errors::PinValidationError;
    use crate::instruction::PinRecoveryWscd;
    use crate::repository::Repository;
    use crate::storage::ChangePinData;
    use crate::storage::InstructionData;
    use crate::storage::PinRecoveryData;
    use crate::storage::RegistrationData;
    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::storage::StoredAttestationMetadata;
    use crate::storage::WithKeyIdentifier;
    use crate::wallet::PersistedPinRecoverySessionData;
    use crate::wallet::Session;
    use crate::wallet::recovery_code::RecoveryCodeError;
    use crate::wallet::test::AUTH_URL;
    use crate::wallet::test::ISSUER_KEY;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_example_pid_mdoc;
    use crate::wallet::test::create_example_pid_sd_jwt;
    use crate::wallet::test::create_wp_result;
    use crate::wallet::test::decoded_sd_jwt_from_credential_payload;
    use crate::wallet::test::mock_issuance_session;
    use crate::wallet::test::nl_pid_mdoc_credential_metadata_example;

    fn setup_issuer_metadata_mock(wallet: &mut TestWalletMockStorage) {
        wallet
            .issuance_discovery
            .expect_start_authorization_code_flow_sync()
            .withf(|selection| {
                let expected_credential_kinds =
                    HashSet::from([CredentialKindRef::new(Format::SdJwt, PID_ATTESTATION_TYPE)]);

                matches!(
                    selection,
                    CredentialSelection::ByCredentialKind(credential_kinds)
                        if **credential_kinds == expected_credential_kinds
                )
            })
            .return_once(|_| {
                let mut authorization_session = MockAuthorizationSession::new();

                authorization_session
                    .expect_get_auth_url()
                    .return_const(Url::parse(AUTH_URL).unwrap());
                authorization_session
                    .expect_get_state()
                    .return_const("state".to_string());
                Ok(authorization_session)
            });
    }

    #[tokio::test]
    pub async fn create_pin_recovery_redirect_uri() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // PIN recovery requires a PID in the SD-JWT format, so only the SD-JWT PID credential kinds
        // should be requested from storage.
        let expected_credential_kinds = wallet
            .config_repository
            .get()
            .pid_attributes
            .sd_jwt_credential_kinds()
            .map(CredentialKindRef::into_owned)
            .collect::<HashSet<_>>();

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_credential_kinds()
            .once()
            .with(eq(expected_credential_kinds))
            .return_once(|_| Ok(true));
        wallet
            .mut_storage()
            .expect_upsert_data::<PersistedPinRecoverySessionData<MockAuthorizationSessionData>>()
            .return_once(|data| {
                assert_eq!(data.authorization_session.auth_url.as_str(), AUTH_URL);
                assert_eq!(data.authorization_session.state, "state");
                Ok(())
            });

        setup_issuer_metadata_mock(&mut wallet);

        let redirect_uri = wallet.create_pin_recovery_redirect_uri().await.unwrap();
        assert!(redirect_uri.as_str().starts_with(AUTH_URL));
    }

    #[tokio::test]
    pub async fn continue_pin_recovery() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let redirect_uri = Url::parse(AUTH_URL).unwrap();

        // The PID is only checked when completing PIN recovery, so the issuance session is not used yet.
        let mut authorization_session = MockAuthorizationSession::new();
        authorization_session
            .expect_start_issuance_sync()
            .return_once(|| Ok(MockIssuanceSession::new()));

        wallet.session = Some(Session::PinRecovery(PinRecoverySession::OAuth {
            authorization_session,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedPinRecoverySessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        wallet.continue_pin_recovery(redirect_uri).await.unwrap();

        assert_matches!(
            wallet.session,
            Some(Session::PinRecovery(PinRecoverySession::Issuance { .. }))
        );
    }

    #[tokio::test]
    pub async fn complete_pin_recovery() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // complete_pin_recovery() updates the PIN recovery state.
        wallet
            .mut_storage()
            .expect_upsert_data()
            .once()
            .returning(|_: &PinRecoveryData| Ok(()));

        // Constructing an instruction client check if the PIN is being changed.
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .once()
            .returning(|| Ok(None));

        // General expectations for sending instructions.
        wallet
            .mut_storage()
            .expect_upsert_data()
            .once()
            .returning(|registration_data: &RegistrationData| {
                assert_eq!(registration_data.wallet_certificate.serialization(), "a.b.c");
                Ok(())
            });
        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .times(2)
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 42,
                }))
            });
        wallet
            .mut_storage()
            .expect_upsert_data()
            .times(2)
            .returning(|_: &InstructionData| Ok(()));

        // Setup expectations for sending `DiscloseRecoveryCodePinRecovery` instruction
        let account_provider_client = Arc::get_mut(&mut wallet.account_provider_client).unwrap();
        account_provider_client
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));
        account_provider_client.expect_instruction().once().return_once(
            move |_, instruction: Instruction<DiscloseRecoveryCodePinRecovery>| {
                assert_eq!(instruction.certificate.serialization(), "a.b.c"); // The wallet uses the new certificate

                // PIN recovery disclosure has one disclosure, so two tildes, so three parts
                assert_eq!(
                    instruction
                        .instruction
                        .dangerous_parse_unverified()
                        .unwrap()
                        .payload
                        .recovery_code_disclosure
                        .to_string()
                        .split('~')
                        .count(),
                    3
                );

                Ok(create_wp_result(()))
            },
        );

        // Finally, complete_pin_recovery() deletes the PIN recovery data.
        wallet
            .mut_storage()
            .expect_delete_data::<PinRecoveryData>()
            .once()
            .returning(|| Ok(()));

        setup_stored_sd_jwt_pid(&mut wallet);

        // Setup the issuance session
        setup_issuance_session(&mut wallet, true);

        wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn cancel_pin_recovery() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        setup_issuance_session(&mut wallet, false);

        assert_matches!(
            &wallet.session,
            Some(Session::PinRecovery(PinRecoverySession::Issuance { .. }))
        );
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedPinRecoverySessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        wallet.cancel_pin_recovery().await.unwrap();

        // Cancelling clears the session state.
        assert_matches!(wallet.session, None);
    }

    // Failing unit tests for create_pin_recovery_redirect_uri()

    #[tokio::test]
    pub async fn create_pin_recovery_redirect_uri_without_pid() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_credential_kinds()
            .once()
            .return_once(|_| Ok(false));

        let err = wallet.create_pin_recovery_redirect_uri().await.unwrap_err();

        assert_matches!(err, PinRecoveryError::NoPidPresent);
    }

    // Failing unit tests for continue_pin_recovery()

    #[tokio::test]
    async fn continue_pin_recovery_no_pid_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn continue_pin_recovery_has_issuance_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        setup_issuance_session(&mut wallet, false);

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn continue_pin_recovery_user_refused() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let mut authorization_session = MockAuthorizationSession::new();
        authorization_session
            .expect_start_issuance_sync()
            .return_once(|| Err(WalletIssuanceError::OAuth(OAuthError::Denied)));

        wallet.session = Some(Session::PinRecovery(PinRecoverySession::OAuth {
            authorization_session,
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedPinRecoverySessionData<MockAuthorizationSessionData>>()
            .return_once(|| Ok(()));

        // Pass a redirect URI with `error=access_denied` to simulate user denial
        let denied_redirect = Url::parse(&format!("{AUTH_URL}?error=access_denied&state=whatever")).unwrap();
        let err = wallet.continue_pin_recovery(denied_redirect).await.unwrap_err();

        assert_matches!(err, PinRecoveryError::AuthorizationDenied);
    }

    #[tokio::test]
    async fn continue_pin_recovery_delete_persisted_session_error_keeps_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.session = Some(Session::PinRecovery(PinRecoverySession::OAuth {
            authorization_session: MockAuthorizationSession::new(),
        }));
        wallet
            .mut_storage()
            .expect_delete_data::<PersistedPinRecoverySessionData<MockAuthorizationSessionData>>()
            .return_once(|| Err(crate::errors::StorageError::NotOpened));

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::Storage(crate::errors::StorageError::NotOpened));
        assert_matches!(
            wallet.session,
            Some(Session::PinRecovery(PinRecoverySession::OAuth { .. }))
        );
    }

    // Failing unit tests for complete_pin_recovery()

    #[tokio::test]
    async fn complete_pin_recovery_no_issuance_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".into())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn complete_pin_recovery_too_simple_pin() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Setup the issuance session
        setup_issuance_session(&mut wallet, false);

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "111111".into())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::PinValidation(PinValidationError::TooFewUniqueDigits)
        );
    }

    #[tokio::test]
    async fn complete_pin_recovery_received_no_recovery_code() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Have the issued PID contain only the family name, so that it lacks a recovery code.
        let (mut credential_payload, _) = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
        credential_payload
            .contents
            .attributes
            .prune(&[vec_nonempty![ClaimPath::SelectByKey(PID_FAMILY_NAME.to_string())]]);
        setup_rejected_pid(&mut wallet, issued_sd_jwt_pid(credential_payload));

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".into())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::RecoveryCode(RecoveryCodeError::MissingIssuedRecoveryCode)
        );
    }

    #[tokio::test]
    async fn complete_pin_recovery_received_wrong_recovery_code() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Have the issued PID contain a different recovery code than the stored PID.
        let (mut credential_payload, _) = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());
        credential_payload
            .contents
            .attributes
            .overwrite([PID_RECOVERY_CODE], Attribute::Text("wrong recovery code".to_string()));
        setup_rejected_pid(&mut wallet, issued_sd_jwt_pid(credential_payload));

        setup_stored_sd_jwt_pid(&mut wallet);

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".into())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::RecoveryCode(RecoveryCodeError::IncorrectRecoveryCode { .. })
        );
    }

    fn issued_sd_jwt_pid(credential_payload: CredentialPayload) -> DecodedSdJwt {
        decoded_sd_jwt_from_credential_payload(
            credential_payload,
            Some(&NormalizedTypeMetadata::nl_pid_example()),
            &ISSUER_KEY.issuance_key,
        )
    }

    /// Have the storage contain the example SD-JWT PID, whose recovery code is compared against that of the issued PID.
    fn setup_stored_sd_jwt_pid(wallet: &mut TestWalletMockStorage) {
        let pid_credential_kinds = wallet
            .config_repository
            .get()
            .pid_attributes
            .sd_jwt_credential_kinds()
            .map(CredentialKindRef::into_owned)
            .collect::<HashSet<_>>();

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations_by_credential_kinds()
            .with(eq(pid_credential_kinds))
            .once()
            .return_once(|_| {
                Ok(vec![StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    ValidityWindow::new_valid_mock(),
                    WithKeyIdentifier {
                        key_identifier: "key".to_string(),
                        data: StoredAttestation::SdJwt(create_example_pid_sd_jwt().0),
                    },
                    StoredAttestationMetadata::TypeMetadata(NormalizedTypeMetadata::nl_pid_example()),
                    None,
                    IssuerRegistration::new_mock(),
                )])
            });
    }

    /// Set up a PIN recovery session in which `sd_jwt` is issued as the PID, which should be rejected after the
    /// `StartPinRecovery` instruction. The [`PinRecoveryData`] stored before that should be kept, so that PIN recovery
    /// has to be completed, while the new registration data should not be stored and the recovery code should not be
    /// disclosed to the Wallet Provider.
    fn setup_rejected_pid(wallet: &mut TestWalletMockStorage, sd_jwt: DecodedSdJwt) {
        wallet
            .mut_storage()
            .expect_upsert_data()
            .once()
            .returning(|_: &PinRecoveryData| Ok(()));

        // Constructing an instruction client check if the PIN is being changed.
        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .once()
            .returning(|| Ok(None));

        wallet.mut_storage().expect_upsert_data::<RegistrationData>().never();
        wallet.mut_storage().expect_delete_data::<PinRecoveryData>().never();
        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .never();

        setup_issuance_session_with_sd_jwt_pid(wallet, sd_jwt, true);
    }

    fn setup_issuance_session(wallet: &mut TestWalletMockStorage, expect_accept_issuance: bool) {
        setup_issuance_session_with_sd_jwt_pid(wallet, create_example_pid_sd_jwt().0, expect_accept_issuance);
    }

    /// Set up a PIN recovery session in which the PID is issued as an mdoc and as `sd_jwt`, in that order.
    fn setup_issuance_session_with_sd_jwt_pid(
        wallet: &mut TestWalletMockStorage,
        sd_jwt: DecodedSdJwt,
        expect_accept_issuance: bool,
    ) {
        let (mdoc, _metadata) = create_example_pid_mdoc(&SigningKey::generate());

        let (pid_issuer, _) = mock_issuance_session(
            [
                (
                    WithKeyIdentifier {
                        key_identifier: "key_id".to_string(),
                        data: StoredAttestation::MsoMdoc(mdoc),
                    },
                    IssuedCredentialMetadata::CredentialMetadata(nl_pid_mdoc_credential_metadata_example()),
                ),
                (
                    WithKeyIdentifier {
                        key_identifier: "key_id".to_string(),
                        data: StoredAttestation::SdJwt(sd_jwt),
                    },
                    IssuedCredentialMetadata::nl_pid_example(),
                ),
            ],
            expect_accept_issuance.then_some(NonZeroU8::MIN),
        );

        wallet.session = Some(Session::PinRecovery(PinRecoverySession::Issuance {
            issuance_session: pid_issuer,
        }));
    }

    struct MockPinWscd;

    impl IssuanceWscd for MockPinWscd {
        type Error = !;

        async fn perform_issuance(
            &self,
            _key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
        ) -> Result<VecNonEmpty<IssuanceKeyResult>, Self::Error> {
            unimplemented!()
        }
    }

    impl PinRecoveryWscd for MockPinWscd {
        fn certificates(self) -> Vec<WalletCertificate> {
            vec![UnverifiedJwt::from_str("a.b.c").unwrap()]
        }
    }
}
