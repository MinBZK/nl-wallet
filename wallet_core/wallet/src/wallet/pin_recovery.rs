use std::sync::Arc;

use itertools::Itertools;
use p256::ecdsa::VerifyingKey;
use tracing::info;
use tracing::instrument;
use url::Url;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::reqwest::client_builder_accept_json;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::HttpVcMessageClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::oidc::OidcError;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_account::NL_WALLET_CLIENT_ID;
use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
use wallet_configuration::wallet_config::PidAttributesConfigurationError;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::digid::DigidClient;
use crate::digid::DigidError;
use crate::digid::DigidSession;
use crate::errors::InstructionError;
use crate::errors::PinKeyError;
use crate::errors::PinValidationError;
use crate::errors::StorageError;
use crate::instruction::InstructionClient;
use crate::instruction::InstructionClientParameters;
use crate::instruction::PinRecoveryRemoteEcdsaWscd;
use crate::instruction::PinRecoveryWscd;
use crate::pin::key::PinKey;
use crate::pin::key::new_pin_salt;
use crate::repository::Repository;
use crate::storage::PinRecoveryData;
use crate::storage::RegistrationData;
use crate::storage::Storage;
use crate::validate_pin;
use crate::wallet::recovery_code::RecoveryCodeError;

use super::IssuanceError;
use super::Session;
use super::Wallet;
use super::WalletRegistration;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum PinRecoveryError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,

    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,

    #[error("pin recovery session is not in the correct state")]
    #[category(expected)]
    SessionState,

    #[error("error during PID issuance: {0}")]
    Issuance(#[from] IssuanceError),

    #[error("the new PIN does not adhere to requirements: {0}")]
    #[category(expected)]
    PinValidation(#[from] PinValidationError),

    #[error("error computing PIN public key: {0}")]
    #[category(unexpected)]
    PinKey(#[from] PinKeyError),

    #[error("storage error: {0}")]
    #[category(unexpected)]
    Storage(#[from] StorageError),

    #[error("failed to disclose recovery code to WP: {0}")]
    DiscloseRecoveryCode(#[source] InstructionError),

    #[error("not permitted: already committed to PIN recovery")]
    #[category(unexpected)]
    CommittedToPinRecovery,

    #[error("user denied DigiD authentication")]
    #[category(expected)]
    DeniedDigiD,

    #[error("cannot recover PIN without a PID")]
    #[category(critical)]
    NoPidPresent,

    #[error("recovery code error: {0}")]
    RecoveryCode(#[from] RecoveryCodeError),

    #[error("PID configuration error: {0}")]
    PidAttributesConfiguration(#[from] PidAttributesConfigurationError),
}

impl From<DigidError> for PinRecoveryError {
    fn from(error: DigidError) -> Self {
        if matches!(error, DigidError::Oidc(OidcError::Denied)) {
            PinRecoveryError::DeniedDigiD
        } else {
            PinRecoveryError::Issuance(IssuanceError::DigidSessionFinish(error))
        }
    }
}

#[derive(Debug)]
pub(super) enum PinRecoverySession<DS, IS> {
    Digid(DS),
    Issuance {
        pid_attestation_type: String,
        issuance_session: IS,
    },
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pin_recovery_redirect_uri(&mut self) -> Result<Url, PinRecoveryError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PinRecoveryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        info!("Checking if a pid is present");
        let config = &self.config_repository.get();
        let pid_config = config.pid_attributes.clone();
        let pid_attestation_types = pid_config.sd_jwt.keys().map(String::clone).collect_vec();
        let has_pid = self
            .storage
            .read()
            .await
            .has_any_attestations_with_types(pid_attestation_types.as_slice())
            .await?;
        if !has_pid {
            return Err(PinRecoveryError::NoPidPresent);
        }

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active session");
        if self.session.is_some() {
            return Err(PinRecoveryError::SessionState);
        }

        // No need to check if a `PinRecoveryData` is already stored: we can always start PIN recovery again.

        let session = self
            .digid_client
            .start_session(
                config.pid_issuance.digid.clone(),
                config.pid_issuance.digid_http_config.clone(),
                urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().to_owned(),
            )
            .await
            .map_err(IssuanceError::DigidSessionStart)?;

        info!("PIN recovery DigiD auth URL generated");
        let auth_url = session.auth_url().clone();
        self.session.replace(Session::PinRecovery {
            pid_config,
            session: PinRecoverySession::Digid(session),
        });

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pin_recovery(&mut self, redirect_uri: Url) -> Result<(), PinRecoveryError> {
        info!("Received redirect URI, processing URI and retrieving access token");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PinRecoveryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(PinRecoveryError::NotRegistered);
        }

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active DigiD issuance session");
        if !matches!(
            self.session,
            Some(Session::PinRecovery {
                session: PinRecoverySession::Digid(..),
                ..
            })
        ) {
            return Err(PinRecoveryError::SessionState);
        }

        let Some(Session::PinRecovery {
            session: PinRecoverySession::Digid(session),
            pid_config,
        }) = self.session.take()
        else {
            unreachable!("session contained no PinRecoveryDigid"); // we just checked this above
        };

        // Fetch issuance previews
        let config = self.config_repository.get();
        let token_request = session
            .into_token_request(&config.pid_issuance.digid_http_config, redirect_uri)
            .await?;
        let http_client = client_builder_accept_json(default_reqwest_client_builder())
            .build()
            .expect("Could not build reqwest HTTP client");
        let issuance_session = IS::start_issuance(
            HttpVcMessageClient::new(NL_WALLET_CLIENT_ID.to_string(), http_client),
            config.pid_issuance.pid_issuer_url.clone(),
            token_request,
            &config.issuer_trust_anchors(),
        )
        .await
        .map_err(IssuanceError::from)?;

        info!("successfully received token and previews from issuer");

        // Check the recovery code in the received PID against the one in the stored PID, as otherwise
        // the WP will reject our PIN recovery instructions.
        let pid_preview = Self::pid_preview(issuance_session.normalized_credential_preview(), &pid_config)?;
        self.compare_recovery_code_against_stored(pid_preview, &pid_config)
            .await?;

        self.session.replace(Session::PinRecovery {
            pid_config,
            session: PinRecoverySession::Issuance {
                pid_attestation_type: pid_preview.content.credential_payload.attestation_type.clone(),
                issuance_session,
            },
        });

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
    pub async fn complete_pin_recovery(&mut self, new_pin: String) -> Result<(), PinRecoveryError> {
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
        new_pin: String,
    ) -> Result<(), PinRecoveryError>
    where
        P: PinRecoveryWscd,
        F: FnOnce(
            InstructionClient<S, <AKH as AttestedKeyHolder>::AppleKey, <AKH as AttestedKeyHolder>::GoogleKey, APC>,
            VerifyingKey,
        ) -> P,
    {
        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(PinRecoveryError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or(PinRecoveryError::NotRegistered)?;

        // Don't check if wallet is locked since PIN recovery is allowed in that case

        info!("Checking if there is an active issuance session");
        if !matches!(
            self.session,
            Some(Session::PinRecovery {
                session: PinRecoverySession::Issuance { .. },
                ..
            })
        ) {
            return Err(PinRecoveryError::SessionState);
        }

        let Some(Session::PinRecovery {
            pid_config,
            session:
                PinRecoverySession::Issuance {
                    pid_attestation_type: offered_pid,
                    issuance_session,
                },
        }) = &self.session.take()
        else {
            unreachable!("session contained no PIN recovery issuance session"); // we just checked this above
        };

        validate_pin(&new_pin)?;

        // All sanity checks are done, we proceed with recovery.

        let new_pin_salt = new_pin_salt();
        let pin_pubkey = PinKey {
            pin: &new_pin,
            salt: &new_pin_salt,
        }
        .verifying_key()?;

        let registration_data = RegistrationData {
            pin_salt: new_pin_salt.clone(),
            ..registration_data.clone()
        };

        let config = self.config_repository.get();
        let instruction_client = self
            .new_instruction_client(
                new_pin.clone(),
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                config.account_server.instruction_result_public_key.as_inner().into(),
            )
            .await
            .map_err(IssuanceError::from)?;

        let pin_recovery_wscd = pin_recovery_wscd_factory(instruction_client, pin_pubkey);

        // Accept issuance to obtain the PID. This sends the `StartPinRecovery` instruction to the WP.
        // `accept_issuance()` below is the point of no return. If the app is killed between there and completion,
        // PIN recovery will have to start again from the start.
        self.storage.write().await.upsert_data(&PinRecoveryData).await?;

        let issuance_result = issuance_session
            .accept_issuance(&config.issuer_trust_anchors(), &pin_recovery_wscd, true)
            .await
            .map_err(|error| Self::handle_accept_issuance_error(error, issuance_session))?;

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
        self.storage.write().await.upsert_data(&registration_data).await?;

        let attested_key = Arc::clone(attested_key);
        self.registration = WalletRegistration::Registered {
            attested_key: Arc::clone(&attested_key),
            data: registration_data.clone(),
        };

        // Get an SD-JWT copy out of the PID we just received.
        let attestation = issuance_result
            .into_iter()
            .find(|attestation| attestation.attestation_type == *offered_pid)
            .expect("no PID received"); // accept_issuance() already checks this against the previews

        let pid_attestation_type = attestation.attestation_type;
        let pid = attestation
            .copies
            .into_inner()
            .into_iter()
            .find_map(|copy| match copy {
                IssuedCredential::MsoMdoc { .. } => None,
                IssuedCredential::SdJwt { sd_jwt, .. } => Some(sd_jwt),
            })
            .expect("no SD-JWT PID received"); // accept_issuance() already checks this against the previews

        let recovery_code_disclosure = pid
            .into_presentation_builder()
            .disclose(&pid_config.recovery_code_path(&pid_attestation_type)?)
            .unwrap() // accept_issuance() already checks against the previews that the PID has a recovery code
            .finish()
            .into();

        // Finish PIN recovery by sending the second WP instruction.

        // Use a new instruction client that uses our new WP certificate
        InstructionClient::new(
            new_pin,
            Arc::clone(&self.storage),
            attested_key,
            Arc::clone(&self.account_provider_client),
            Arc::new(InstructionClientParameters::new(
                registration_data,
                config.account_server.http_config.clone(),
                config.account_server.instruction_result_public_key.as_inner().into(),
            )),
        )
        .send(DiscloseRecoveryCodePinRecovery {
            recovery_code_disclosure,
        })
        .await
        .map_err(PinRecoveryError::DiscloseRecoveryCode)?;

        self.storage.write().await.delete_data::<PinRecoveryData>().await?;

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

        self.session = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::num::NonZeroUsize;
    use std::str::FromStr;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;
    use uuid::Uuid;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::validity::ValidityWindow;
    use attestation_types::claim_path::ClaimPath;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use attestation_types::pid_constants::PID_RECOVERY_CODE;
    use jwt::UnverifiedJwt;
    use openid4vc::Format;
    use openid4vc::issuance_session::IssuedCredential;
    use openid4vc::mock::MockIssuanceSession;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wallet_account::messages::instructions::DiscloseRecoveryCodePinRecovery;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::registration::WalletCertificateClaims;
    use wscd::Poa;
    use wscd::wscd::IssuanceWscd;

    use crate::digid::DigidError;
    use crate::digid::MockDigidSession;
    use crate::errors::PinValidationError;
    use crate::instruction::PinRecoveryWscd;
    use crate::repository::Repository;
    use crate::storage::ChangePinData;
    use crate::storage::InstructionData;
    use crate::storage::PinRecoveryData;
    use crate::storage::RegistrationData;
    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::wallet::PinRecoverySession;
    use crate::wallet::Session;
    use crate::wallet::recovery_code::RecoveryCodeError;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::create_example_pid_preview_data;
    use crate::wallet::test::create_example_pid_sd_jwt;
    use crate::wallet::test::create_wp_result;
    use crate::wallet::test::mock_issuance_session;

    use super::PinRecoveryError;

    const AUTH_URL: &str = "http://example.com/auth";

    #[rstest]
    #[tokio::test]
    pub async fn create_pin_recovery_redirect_uri(
        #[values(None, Some(PinRecoveryData))] pin_recovery_data: Option<PinRecoveryData>,
    ) {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // create_pin_recovery_redirect_uri() is allowed regardless of whether the wallet
        // was already recovering the PIN.
        wallet
            .mut_storage()
            .expect_fetch_data()
            .return_once(|| Ok(pin_recovery_data));

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .once()
            .return_once(|_| Ok(true));

        wallet
            .digid_client
            .expect_start_session()
            .once()
            .return_once(|_digid_config, _http_config, _redirect_uri| {
                let mut session = MockDigidSession::new();

                session
                    .expect_auth_url()
                    .once()
                    .return_const(Url::parse(AUTH_URL).unwrap());

                Ok(session)
            });

        let redirect_uri = wallet.create_pin_recovery_redirect_uri().await.unwrap();
        assert_eq!(&redirect_uri.to_string(), AUTH_URL);
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    pub async fn continue_pin_recovery() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let mut session = MockDigidSession::new();
        session
            .expect_into_token_request()
            .once()
            .return_once(|_http_config, _redirect_uri| {
                let token_request = TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                };

                Ok(token_request)
            });
        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(session),
        });

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut client = MockIssuanceSession::new();

            client
                .expect_normalized_credential_previews()
                .once()
                .return_const(vec![create_example_pid_preview_data(
                    &MockTimeGenerator::default(),
                    Format::SdJwt,
                )]);

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            Ok(client)
        });

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations_by_types_and_format()
            .once()
            .returning(|_, _| {
                Ok(vec![StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    ValidityWindow::new_valid_mock(),
                    StoredAttestation::SdJwt {
                        key_identifier: "key".to_string(),
                        sd_jwt: create_example_pid_sd_jwt().0,
                    },
                    NormalizedTypeMetadata::nl_pid_example(),
                    None,
                )])
            });

        wallet.continue_pin_recovery(AUTH_URL.parse().unwrap()).await.unwrap();
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

        // Setup the issuance session
        setup_issuance_session(&mut wallet);

        wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".to_string())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn cancel_pin_recovery() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        setup_issuance_session(&mut wallet);

        assert_matches!(
            &wallet.session,
            Some(Session::PinRecovery {
                session: PinRecoverySession::Issuance { .. },
                ..
            })
        );

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
            .expect_has_any_attestations_with_types()
            .once()
            .return_once(|_| Ok(false));

        let err = wallet.create_pin_recovery_redirect_uri().await.unwrap_err();

        assert_matches!(err, PinRecoveryError::NoPidPresent);
    }

    // Failing unit tests for continue_pid_recovery()

    #[tokio::test]
    async fn continue_pid_recovery_no_digid_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn continue_pid_recovery_has_issuance_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        setup_issuance_session(&mut wallet);

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn continue_pid_recovery_user_refused() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let mut digid_session = MockDigidSession::new();
        digid_session
            .expect_into_token_request()
            .once()
            .returning(|_, _| Err(DigidError::Oidc(OidcError::Denied)));

        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(digid_session),
        });

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::DeniedDigiD);
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    pub async fn continue_pin_recovery_received_no_recovery_code() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let mut session = MockDigidSession::new();
        session
            .expect_into_token_request()
            .once()
            .return_once(|_http_config, _redirect_uri| {
                let token_request = TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                };

                Ok(token_request)
            });
        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(session),
        });

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().once().return_once(|| {
            let mut client = MockIssuanceSession::new();

            // Remove the recovery code attribute from the preview
            let mut preview = create_example_pid_preview_data(&MockTimeGenerator::default(), Format::SdJwt);
            preview
                .content
                .credential_payload
                .attributes
                .prune(&[vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())]]);

            client
                .expect_normalized_credential_previews()
                .once()
                .return_const(vec![preview]);

            client.expect_issuer().return_const(IssuerRegistration::new_mock());

            Ok(client)
        });

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::RecoveryCode(RecoveryCodeError::MissingRecoveryCode)
        );
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    pub async fn continue_pin_recovery_received_wrong_recovery_code() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let mut session = MockDigidSession::new();
        session
            .expect_into_token_request()
            .once()
            .return_once(|_http_config, _redirect_uri| {
                let token_request = TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                };

                Ok(token_request)
            });
        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(session),
        });

        // Set up the `MockIssuanceSession` to return one `CredentialPreviewState`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().once().return_once(move || {
            let mut client = MockIssuanceSession::new();

            // Change the recovery code attribute from the preview
            let mut preview = create_example_pid_preview_data(&MockTimeGenerator::default(), Format::SdJwt);

            let attributes = &mut preview.content.credential_payload.attributes;
            attributes.prune(&[vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())]]);
            attributes
                .insert(
                    &vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())],
                    Attribute::Single(AttributeValue::Text("wrong recovery code".to_string())),
                )
                .unwrap();

            client
                .expect_normalized_credential_previews()
                .once()
                .return_const(vec![preview]);

            Ok(client)
        });

        wallet
            .mut_storage()
            .expect_fetch_unique_attestations_by_types_and_format()
            .once()
            .returning(|_, _| {
                Ok(vec![StoredAttestationCopy::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    ValidityWindow::new_valid_mock(),
                    StoredAttestation::SdJwt {
                        key_identifier: "key".to_string(),
                        sd_jwt: create_example_pid_sd_jwt().0,
                    },
                    NormalizedTypeMetadata::nl_pid_example(),
                    None,
                )])
            });

        let err = wallet
            .continue_pin_recovery(AUTH_URL.parse().unwrap())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::RecoveryCode(RecoveryCodeError::IncorrectRecoveryCode { .. })
        );
    }

    // Failing unit tests for complete_pid_recovery()

    #[tokio::test]
    async fn complete_pid_recovery_no_issuance_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".to_string())
            .await
            .unwrap_err();

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn complete_pid_recovery_has_digid_session() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "112233".to_string())
            .await
            .unwrap_err();

        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Digid(MockDigidSession::new()),
        });

        assert_matches!(err, PinRecoveryError::SessionState);
    }

    #[tokio::test]
    async fn complete_pid_recovery_too_simple_pin() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Setup the issuance session
        setup_issuance_session(&mut wallet);

        let err = wallet
            .complete_pin_recovery_internal(|_, _| MockPinWscd, "111111".to_string())
            .await
            .unwrap_err();

        assert_matches!(
            err,
            PinRecoveryError::PinValidation(PinValidationError::TooFewUniqueDigits)
        );
    }

    fn setup_issuance_session(wallet: &mut TestWalletMockStorage) {
        let (sd_jwt, _metadata) = create_example_pid_sd_jwt();
        let (pid_issuer, _) = mock_issuance_session(
            IssuedCredential::SdJwt {
                key_identifier: "key_id".to_string(),
                sd_jwt: sd_jwt.clone(),
            },
            PID_ATTESTATION_TYPE.to_string(),
            VerifiedTypeMetadataDocuments::nl_pid_example(),
        );

        wallet.session = Some(Session::PinRecovery {
            pid_config: wallet.config_repository.get().pid_attributes.clone(),
            session: PinRecoverySession::Issuance {
                pid_attestation_type: PID_ATTESTATION_TYPE.to_string(),
                issuance_session: pid_issuer,
            },
        });
    }

    struct MockPinWscd;

    impl IssuanceWscd for MockPinWscd {
        type Error = Infallible;
        type Poa = Poa;

        async fn perform_issuance(
            &self,
            _count: NonZeroUsize,
            _aud: String,
            _nonce: Option<String>,
            _include_wua: bool,
        ) -> Result<wscd::wscd::IssuanceResult<Self::Poa>, Self::Error> {
            unimplemented!()
        }
    }

    impl PinRecoveryWscd for MockPinWscd {
        fn certificates(self) -> Vec<UnverifiedJwt<WalletCertificateClaims>> {
            vec![UnverifiedJwt::from_str("a.b.c").unwrap()]
        }
    }
}
