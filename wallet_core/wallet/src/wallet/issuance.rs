use p256::ecdsa::signature;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::{
    server_keys::KeysError,
    utils::{cose::CoseError, issuer_auth::IssuerRegistration, x509::MdocCertificateExtension},
};
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::config::wallet_config::ISSUANCE_REDIRECT_URI;

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    digid::{DigidError, DigidSession},
    document::{Document, DocumentMdocError},
    instruction::{InstructionClient, InstructionError, RemoteEcdsaKeyError, RemoteEcdsaKeyFactory},
    pid_issuer::{PidIssuerClient, PidIssuerError},
    storage::{Storage, StorageError, WalletEvent},
};

use super::{documents::DocumentsError, Wallet};

#[derive(Debug, thiserror::Error)]
pub enum PidIssuanceError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is locked")]
    Locked,
    #[error("issuance session is not in the correct state")]
    SessionState,
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidError),
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[source] PidIssuerError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    Signature(#[from] signature::Error),
    #[error("could not interpret mdoc attributes: {0}")]
    MdocDocument(#[from] DocumentMdocError),
    #[error("could not insert mdocs in database: {0}")]
    MdocStorage(#[source] StorageError),
    #[error("could not store history in database: {0}")]
    HistoryStorage(#[source] StorageError),
    #[error("key '{0}' not found in Wallet Provider")]
    KeyNotFound(String),
    #[error("invalid issuer certificate: {0}")]
    InvalidIssuerCertificate(#[source] CoseError),
    #[error("issuer not authenticated")]
    MissingIssuerRegistration,
    #[error("could not read documents from storage: {0}")]
    Document(#[source] DocumentsError),
}

// TODO: Remove this once issuer certificate can be known early in the issuance protocol
pub fn rvig_registration() -> IssuerRegistration {
    serde_json::from_str(r#"
        {
          "organization": {
            "displayName": {
              "nl": "Rijksdienst voor Identiteitsgegevens"
            },
            "legalName": {
              "nl": "RvIG"
            },
            "description": {
                "nl": "Opvragen van PID (Person Identification Data)"
            },
            "webUrl": "https://www.rvig.nl",
            "city": {
              "nl": "'s-Gravenhage"
            },
            "category": {
              "nl": "Overheid"
            },
            "logo": {
              "mimeType": "image/png",
              "imageData": "iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAR6SURBVHgB7ZtNbFRVHMXPffUDByNtnCIktlMXMqELWtNCIoYiAaUmIAQ0uiGRhcHIRraYiBBjTFyIGyPBBBMw0UQabUlEAlGmESPSMGWB0g2dYiKlQ2ZMYCofncv/3M59vGlo6DwYCL3vlzTzvnJnzrn//7kvnXkKExjJXXtRe94arbEW0E2YFqi0Ujo9VqzZPrdODZadsRtnc7p2plfcprV+D9MYT6mdl4re9mfqVJ77xgCKj6niLzLjrXAClS5obxlN8LjLmXdHPNGtjxvNYsW/Od3kqbGzcBBP62VejTe2DY4yBqwVE5RDpV+OUmqNGs5f13AYD44TGQDHiQyA40QGwHEiA+A4kQFwnMgAOE5kABzHeQMeQgj2d6fQ1dNrtr/Z/T4eZKIWwD1g5GLe/E3lukLhyl0bbyqEaoGpsl/a5OfDx3G58L/Zr39yFta92oGOxQv8ayh473eH0Jce8K+Lx2v981vefQ1NDU+Z7YNHjqOru9e/bmZsBlauWIT1q5cgLFUz4Ms9B9B7rN9sx+OzMCpCRy7+h117enB59ApeWb7QnNv1dTdOnBxAQkSulGNZuSZ17BRiIq699Vn/uzua2SXZQyic0Agey2bz2LRxNcJQFQMowIp/Z+MqLFncYsTvk5mmWH7opc8vELF5s0+2bH7dVAgpiLATUhHzk03GGJpixXO2WUWEx2gM349VNT+ZQKVUJQP6Tp4xr23PJY14QnGdyxeZbQq8IOIvBVrDiifx0vZIdrzPj/7WX7qu1hdPOqX8bTUMnhtGGKpiQLYUUImnZ5cdT5R6mYyKePY2BbA6fjrypwm202cy6Osfr4r6UhbwPInHnygbj20Siz1qtgslMyulqiE4MdGzJSEWCtj01irJgQPY9+0h82dhtQTDcny8q5O+R6xUCZVSFQMaGuZISV5A6vdTWC8la2cpFShl26+nB4ZMmAVDrDnZKG1wcyVgpfC2K3PuvDHRtghXBbsitEu7heGODWCqB9nwxstYKjPXK8HEsty6YzfaWpPI/DOMv6S8SeeKhf71fenxvOA5K+yo9D7Lv1lM4jFWAu88KfajT/f65jH8zHhSLcEMqYRQ3w0Gb4UnsvOTzWaGbUIHYb+vkxRneN0cS5a3ntQtx6Kojz9421RQRkLusy++94PRQvEb3nwJYQllAD9MZpLUbZfZtiXP8BoaOm8SmjMaPGfH2brjKynfef4KQVg5zAXTGpIRHS+0+Oe4wnA8mslQDbP0BQnVAnzjYKJPhl3e2ibpT5sJscdmlAlhsNEoGlAfuCskHKstZL/fiqquArcj0TjHvLKXWS02yf+WPKD4sDc3lXBfDbDL3MHDf/gBydJubJiN5nmJspueahH9QAKOExkAx4kMgONEBsBxIgPgOJEBcJzIADhOZAAcJzJA/icyCEdRUGlP6+KPcBWlxQDgBzgKH6X15tY9/KvW+nM4BjXzOWLz+4NcTtdedfnR2TrZeEQOuFAJ1GjFc19NvMA8SotrH0LVtEyfilCDDHvmHVs+eOYGfAbtBPsp8XkAAAAASUVORK5CYII="
            },
            "countryCode": "nl",
            "kvk": "27373207",
            "privacyPolicyUrl": "https://www.rvig.nl/privacy"
          }
        }"#).unwrap()
}

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: ConfigurationRepository,
    DGS: DigidSession,
    PIC: PidIssuerClient,
    S: Storage,
{
    #[instrument(skip_all)]
    pub async fn create_pid_issuance_auth_url(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is a DigidSession or PidIssuerClient has session");
        if self.digid_session.is_some() || self.pid_issuer.has_session() {
            return Err(PidIssuanceError::SessionState);
        }

        let pid_issuance_config = &self.config_repository.config().pid_issuance;
        let session = DGS::start(
            pid_issuance_config.digid_url.clone(),
            pid_issuance_config.digid_client_id.to_string(),
            ISSUANCE_REDIRECT_URI.to_owned(),
        )
        .await
        .map_err(PidIssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");

        let auth_url = session.auth_url();
        self.digid_session.replace(session);

        Ok(auth_url)
    }

    pub fn cancel_pid_issuance(&mut self) -> Result<(), PidIssuanceError> {
        info!("PID issuance cancelled");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        if self.digid_session.is_none() {
            return Err(PidIssuanceError::SessionState);
        }

        info!("Removing DigiD session");

        self.digid_session.take();

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: &Url) -> Result<Vec<Document>, PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        // Try to take ownership of any active `DigidSession`.
        let session = self.digid_session.take().ok_or(PidIssuanceError::SessionState)?;

        let access_token = session
            .get_access_token(redirect_uri)
            .await
            .map_err(PidIssuanceError::DigidSessionFinish)?;

        info!("DigiD access token retrieved, starting actual PID issuance");

        let config = self.config_repository.config();

        let unsigned_mdocs = self
            .pid_issuer
            .start_retrieve_pid(&config.pid_issuance.pid_issuer_url, &access_token)
            .await
            .map_err(PidIssuanceError::PidIssuer)?;

        info!("PID received successfully from issuer, returning preview documents");

        let mut documents = unsigned_mdocs
            .into_iter()
            .map(|mdoc| Document::from_unsigned_mdoc(mdoc, rvig_registration())) // TODO: obtain IssuerRegistration via some Issuer Authentication mechanism
            .collect::<Result<Vec<_>, _>>()?;

        documents.sort_by_key(Document::priority);

        Ok(documents)
    }

    #[instrument(skip_all)]
    pub async fn reject_pid_issuance(&mut self) -> Result<(), PidIssuanceError> {
        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if PidIssuerClient has session");
        if !self.pid_issuer.has_session() {
            return Err(PidIssuanceError::SessionState);
        }

        info!("Rejecting any PID held in memory");
        self.pid_issuer.reject_pid().await.map_err(PidIssuanceError::PidIssuer)
    }

    #[instrument(skip_all)]
    pub async fn accept_pid_issuance(&mut self, pin: String) -> Result<(), PidIssuanceError>
    where
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient,
    {
        info!("Accepting PID issuance");

        info!("Checking if registered");
        let registration_data = self
            .registration
            .as_ref()
            .ok_or_else(|| PidIssuanceError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if PidIssuerClient has session");
        if !self.pid_issuer.has_session() {
            return Err(PidIssuanceError::SessionState);
        }

        let config = self.config_repository.config();

        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();

        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &self.hw_privkey,
            &self.account_provider_client,
            registration_data,
            &config.account_server.base_url,
            &instruction_result_public_key,
        );
        let remote_key_factory = RemoteEcdsaKeyFactory::new(&remote_instruction);

        info!("Accepting PID by signing mdoc using Wallet Provider");

        let mdocs = self
            .pid_issuer
            .accept_pid(&config.mdoc_trust_anchors(), &&remote_key_factory)
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaKeyFactory` we provide above.
                    PidIssuerError::MdocError(nl_wallet_mdoc::Error::KeysError(KeysError::KeyGeneration(error))) => {
                        match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                            RemoteEcdsaKeyError::Instruction(error) => PidIssuanceError::Instruction(error),
                            RemoteEcdsaKeyError::Signature(error) => PidIssuanceError::Signature(error),
                            RemoteEcdsaKeyError::KeyNotFound(identifier) => PidIssuanceError::KeyNotFound(identifier),
                        }
                    }
                    _ => PidIssuanceError::PidIssuer(error),
                }
            })?;

        // TODO: Wipe the wallet when receiving a PIN timeout from the WP or when it is blocked.

        // Prepare events before storing mdocs, to avoid cloning mdocs
        let event = {
            // Extract first copy from cred_copies
            let mdocs = mdocs
                .iter()
                .flat_map(|mdoc| mdoc.cred_copies.first())
                .cloned()
                .collect::<Vec<_>>();

            // Validate all issuer_certificates
            for mdoc in mdocs.iter() {
                let certificate = mdoc
                    .issuer_certificate()
                    .map_err(PidIssuanceError::InvalidIssuerCertificate)?;

                // Verify that the certificate contains IssuerRegistration
                if matches!(IssuerRegistration::from_certificate(&certificate), Err(_) | Ok(None)) {
                    return Err(PidIssuanceError::MissingIssuerRegistration);
                }
            }
            WalletEvent::new_issuance(mdocs.try_into().map_err(PidIssuanceError::InvalidIssuerCertificate)?)
        };

        info!("PID accepted, storing mdoc in database");
        self.storage
            .get_mut()
            .insert_mdocs(mdocs)
            .await
            .map_err(PidIssuanceError::MdocStorage)?;

        self.store_history_event(event)
            .await
            .map_err(PidIssuanceError::HistoryStorage)?;

        self.emit_documents().await.map_err(PidIssuanceError::Document)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use assert_matches::assert_matches;
    use chrono::{Days, Utc};
    use mockall::predicate::*;
    use serial_test::serial;
    use url::Url;

    use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, holder::HolderError, issuer_shared::IssuanceError, Tdate};

    use crate::{
        digid::{MockDigidSession, OpenIdError},
        document::{self, DocumentPersistence},
        wallet::test,
    };

    use super::{super::test::WalletWithMocks, *};

    #[tokio::test]
    #[serial]
    async fn test_create_pid_issuance_auth_url() {
        const AUTH_URL: &str = "http://example.com/auth";

        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        assert!(wallet.digid_session.is_none());

        // Set up `DigidSession` to have `start()` and `auth_url()` called on it.
        let session_start_context = MockDigidSession::start_context();
        session_start_context.expect().returning(|_, _, _| {
            let mut session = MockDigidSession::default();

            session.expect_auth_url().return_const(Url::parse(AUTH_URL).unwrap());

            Ok(session)
        });

        // Have the `Wallet` generate a DigiD authentication URL and test it.
        let auth_url = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect("Could not generate PID issuance auth URL");

        assert_eq!(auth_url.as_str(), AUTH_URL);
        assert!(wallet.digid_session.is_some());
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Creating a DigiD authentication URL on
        // a locked wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Creating a DigiD authentication URL on an
        // unregistered wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an active `DigidSession`.
        wallet.digid_session = MockDigidSession::default().into();

        // Creating a DigiD authentication URL on a `Wallet` that
        // has an active `DigidSession` should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);

        // Prepare another wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the `PidIssuerClient` report that it has an active session.
        wallet.pid_issuer.has_session = true;

        // Creating a DigiD authentication URL on a `Wallet` that has
        // an active `PidIssuerClient` session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_pid_issuance_auth_url_error_digid_session_start() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up `DigidSession` to have `start()` return an error.
        let session_start_context = MockDigidSession::start_context();
        session_start_context
            .expect()
            .return_once(|_, _, _| Err(OpenIdError::from(openid::error::Error::CannotBeABase).into()));

        // The error should be forwarded when attempting to create a DigiD authentication URL.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::DigidSessionStart(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up an active `DigidSession`.
        wallet.digid_session = MockDigidSession::default().into();

        assert!(wallet.digid_session.is_some());

        // Cancelling PID issuance should clear this session.
        wallet.cancel_pid_issuance().expect("Could not cancel PID issuance");

        assert!(wallet.digid_session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Cancelling PID issuance on a locked wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Cancelling PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Cancelling PID issuance on a wallet with no
        // active DigiD session should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    const REDIRECT_URI: &str = "redirect://here";
    const ACCESS_TOKEN: &str = "the_access_code";

    #[tokio::test]
    async fn test_continue_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a `DigidSession` that returns an access token.
        wallet.digid_session = {
            let mut session = MockDigidSession::default();

            session
                .expect_get_access_token()
                .with(eq(Url::parse(REDIRECT_URI).unwrap()))
                .return_once(|_| Ok(ACCESS_TOKEN.to_string()));

            session
        }
        .into();

        // Set up the `PidIssuerClient` to return one `UnsignedMdoc`.
        wallet.pid_issuer.unsigned_mdocs = vec![document::create_full_unsigned_pid_mdoc()];

        // Continuing PID issuance should result in one preview `Document`.
        let documents = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(documents.len(), 1);
        assert_matches!(documents[0].persistence, DocumentPersistence::InMemory);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Continuing PID issuance on a locked wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Continuing PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_session_state() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Continuing PID issuance on a wallet with no active `DigidSession` should result in an error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_digid_session_finish() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a `DigidSession` that returns an error when requesting an access token.
        wallet.digid_session = {
            let mut session = MockDigidSession::default();

            session
                .expect_get_access_token()
                .with(eq(Url::parse(REDIRECT_URI).unwrap()))
                .return_once(|_| Err(OpenIdError::from(openid::error::Error::MissingOpenidScope).into()));

            session
        }
        .into();

        // Continuing PID issuance on a wallet should forward this error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::DigidSessionFinish(_));
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_pid_issuer() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a `DigidSession` that returns an access token.
        wallet.digid_session = {
            let mut session = MockDigidSession::default();

            session
                .expect_get_access_token()
                .with(eq(Url::parse(REDIRECT_URI).unwrap()))
                .return_once(|_| Ok(ACCESS_TOKEN.to_string()));

            session
        }
        .into();

        // Set up the `PidIssuerClient` to return an error.
        wallet.pid_issuer.next_error =
            PidIssuerError::from(nl_wallet_mdoc::Error::from(IssuanceError::MissingSessionId)).into();

        // Continuing PID issuance on a wallet should forward this error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_document() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a `DigidSession` that returns an access token.
        wallet.digid_session = {
            let mut session = MockDigidSession::default();

            session
                .expect_get_access_token()
                .with(eq(Url::parse(REDIRECT_URI).unwrap()))
                .return_once(|_| Ok(ACCESS_TOKEN.to_string()));

            session
        }
        .into();

        // Set up the `PidIssuerClient` to return an `UnsignedMdoc` with an unknown doctype.
        wallet.pid_issuer.unsigned_mdocs = vec![UnsignedMdoc {
            doc_type: "foobar".to_string(),
            valid_from: Tdate::now(),
            valid_until: (Utc::now() + Days::new(365)).into(),
            attributes: Default::default(),
            copy_count: 1,
        }];

        // Continuing PID issuance when receiving an unknown mdoc should result in an error.
        let error = wallet
            .continue_pid_issuance(&Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::MdocDocument(_));
    }

    #[tokio::test]
    async fn test_reject_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up the `PidIssuerClient` to report having a session
        wallet.pid_issuer.has_session = true;

        // Cancelling PID issuance should not fail.
        wallet
            .reject_pid_issuance()
            .await
            .expect("Could not reject PID issuance");
    }

    #[tokio::test]
    async fn test_reject_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Rejecting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .reject_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_reject_pid_issuance_error_not_registered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Rejecting PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .reject_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_reject_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Rejecting PID issuance on a `Wallet` that has a
        // `PidIssuerClient` with no session should return an error.
        let error = wallet
            .reject_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_reject_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up the `PidIssuerClient` to report having a session, then return an error.
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.next_error =
            PidIssuerError::from(nl_wallet_mdoc::Error::from(IssuanceError::MissingSessionId)).into();

        // Rejecting PID issuance on a wallet should forward this error.
        let error = wallet
            .reject_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    const PIN: &str = "051097";

    #[tokio::test]
    async fn test_accept_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Wrap a `Vec<Document>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let documents = Arc::new(Mutex::new(Vec::<Vec<Document>>::with_capacity(2)));
        let callback_documents = Arc::clone(&documents);

        // Set the documents callback on the `Wallet`, which should
        // immediately be called with an empty `Vec`.
        wallet
            .set_documents_callback(move |documents| callback_documents.lock().unwrap().push(documents.clone()))
            .await
            .expect("Could not set documents callback");

        // Have the `PidIssuerClient` accept the PID with a single
        // instance of `MdocCopies`, which contains a single valid `Mdoc`.
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.mdoc_copies = vec![vec![test::create_full_pid_mdoc().await].into()];

        // Accept the PID issuance with the PIN.
        wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect("Could not accept PID issuance");

        // Test that one successful issuance event is logged
        let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_matches!(events.first().unwrap(), WalletEvent::Issuance { .. });

        // Test which `Document` instances we have received through the callback.
        let documents = documents.lock().unwrap();

        // The first entry should be empty, because there are no mdocs in the database.
        assert_eq!(documents.len(), 2);
        assert!(documents[0].is_empty());

        // The second entry should contain a single document with the PID.
        assert_eq!(documents[1].len(), 1);
        let document = &documents[1][0];
        assert_matches!(document.persistence, DocumentPersistence::Stored(_));
        assert_eq!(document.doc_type, "com.example.pid");
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_missing_issuer_registration() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the `PidIssuerClient` accept the PID with a single instance of `MdocCopies`, which contains a single
        // valid `Mdoc`, but signed with a Certificate that is missing IssuerRegistration
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.mdoc_copies = vec![vec![test::create_full_pid_mdoc_unauthenticated().await].into()];

        // Accept the PID issuance with the PIN.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MissingIssuerRegistration);

        // No issuance event is logged
        let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Accepting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Accepting PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Accepting PID issuance on a `Wallet` with a `PidIssuerClient`
        // that has no session should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    async fn test_accept_pid_issuance_error_remote_key(key_error: RemoteEcdsaKeyError) -> PidIssuanceError {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the `PidIssuerClient` return a particular `RemoteEcdsaKeyError`.
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.next_error = PidIssuerError::MdocError(nl_wallet_mdoc::Error::KeysError(
            KeysError::KeyGeneration(Box::new(key_error)),
        ))
        .into();

        // Accepting PID issuance should result in an error.
        wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error")
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_instruction() {
        let error =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(InstructionError::Blocked)).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Instruction(_));
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_signature() {
        let error =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(signature::Error::default())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Signature(_));
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_key_not_found() {
        let error =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::KeyNotFound("not found".to_string())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::KeyNotFound(_));
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the `PidIssuerClient` return an error.
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.next_error =
            PidIssuerError::MdocError(nl_wallet_mdoc::Error::from(HolderError::ReaderAuthMissing)).into();

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_storage() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the `PidIssuerClient` report a a session
        // and have the database return an error on query.
        wallet.pid_issuer.has_session = true;
        wallet.pid_issuer.mdoc_copies = vec![vec![test::create_full_pid_mdoc().await].into()];
        wallet.storage.get_mut().has_query_error = true;

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MdocStorage(_));
    }
}
