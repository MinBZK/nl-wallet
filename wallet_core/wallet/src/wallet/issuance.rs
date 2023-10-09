use p256::ecdsa::signature;
use tracing::{info, instrument, warn};
use url::Url;

use platform_support::hw_keystore::PlatformEcdsaKey;

use crate::{
    account_provider::AccountProviderClient,
    config::ConfigurationRepository,
    digid::{DigidError, DigidSession},
    document::{Document, DocumentMdocError},
    instruction::{InstructionClient, InstructionError, RemoteEcdsaKeyError, RemoteEcdsaKeyFactory},
    pid_issuer::{PidIssuerClient, PidIssuerError},
    storage::{Storage, StorageError},
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum PidIssuanceError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidError),
    #[error("no DigiD session was found")]
    NoSession,
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[source] PidIssuerError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    Signature(#[from] signature::Error),
    #[error("could not interpret mdoc attributes: {0}")]
    Document(#[from] DocumentMdocError),
    #[error("could not access mdocs database: {0}")]
    Database(#[from] StorageError),
}

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    C: ConfigurationRepository,
    D: DigidSession,
    P: PidIssuerClient,
{
    #[instrument(skip_all)]
    pub async fn create_pid_issuance_auth_url(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if already registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        if self.digid_session.is_some() {
            warn!("DigiD auth url is requested for PID issuance while another Digid session is present, overwriting");
        }

        let pid_issuance_config = &self.config_repository.config().pid_issuance;

        let session = D::start(
            pid_issuance_config.digid_url.clone(),
            pid_issuance_config.digid_client_id.to_string(),
            pid_issuance_config.digid_redirect_uri.clone(),
        )
        .await
        .map_err(PidIssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");

        let auth_url = session.auth_url();
        self.digid_session.replace(session);

        Ok(auth_url)
    }

    pub fn cancel_pid_issuance(&mut self) {
        if self.digid_session.is_none() {
            warn!("PID issuance was cancelled, but no DigiD session is currently present");

            return;
        }

        info!("PID issuance cancelled, removing DigiD session");

        self.digid_session.take();
    }

    #[instrument(skip_all)]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: &Url) -> Result<Vec<Document>, PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        info!("Checking if already registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        // Try to take ownership of any active `DigidSession`.
        let session = self.digid_session.take().ok_or(PidIssuanceError::NoSession)?;

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
            .map(Document::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        documents.sort_by_key(Document::priority);

        Ok(documents)
    }

    #[instrument(skip_all)]
    pub async fn reject_pid_issuance(&mut self) -> Result<(), PidIssuanceError> {
        info!("Checking if already registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Rejecting any PID held in memory");
        self.pid_issuer.reject_pid().await.map_err(PidIssuanceError::PidIssuer)
    }

    #[instrument(skip_all)]
    pub async fn accept_pid_issuance(&mut self, pin: String) -> Result<(), PidIssuanceError>
    where
        S: Storage + Send + Sync,
        K: PlatformEcdsaKey + Sync,
        A: AccountProviderClient + Sync,
    {
        info!("Accepting PID issuance");

        info!("Checking if already registered");
        let registration_data = self
            .registration
            .as_ref()
            .ok_or_else(|| PidIssuanceError::NotRegistered)?;

        let config = self.config_repository.config();

        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &self.hw_privkey,
            &self.account_provider_client,
            registration_data,
            &config.account_server.base_url,
            &config.account_server.instruction_result_public_key,
        );
        let remote_key_factory = RemoteEcdsaKeyFactory::new(&remote_instruction);

        info!("Accepting PID by signing mdoc using Wallet Provider");

        let mdocs = self
            .pid_issuer
            .accept_pid(&config.mdoc_trust_anchors(), &remote_key_factory)
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaKeyFactory` we provide above.
                    PidIssuerError::MdocError(nl_wallet_mdoc::Error::KeyGeneration(error)) => {
                        match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                            RemoteEcdsaKeyError::Instruction(error) => PidIssuanceError::Instruction(error),
                            RemoteEcdsaKeyError::Signature(error) => PidIssuanceError::Signature(error),
                        }
                    }
                    _ => PidIssuanceError::PidIssuer(error),
                }
            })?;

        info!("PID accepted, storing mdoc in database");

        self.storage.get_mut().insert_mdocs(mdocs).await?;
        self.emit_documents().await?;

        Ok(())
    }
}
