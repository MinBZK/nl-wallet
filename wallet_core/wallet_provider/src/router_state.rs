use std::error::Error;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

use wallet_common::account::messages::instructions::Instruction;
use wallet_common::account::messages::instructions::InstructionAndResult;
use wallet_common::account::messages::instructions::InstructionResultMessage;
use wallet_common::generator::Generator;
use wallet_common::keys::EcdsaKey;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_service::account_server::AccountServer;
use wallet_provider_service::hsm::Pkcs11Hsm;
use wallet_provider_service::instructions::HandleInstruction;
use wallet_provider_service::instructions::ValidateInstruction;
use wallet_provider_service::keys::InstructionResultSigning;
use wallet_provider_service::keys::WalletCertificateSigning;
use wallet_provider_service::keys::WalletProviderEcdsaKey;
use wallet_provider_service::pin_policy::PinPolicy;
use wallet_provider_service::wte_issuer::HsmWteIssuer;

use crate::errors::WalletProviderError;
use crate::settings::Settings;

pub struct RouterState {
    pub account_server: AccountServer,
    pub pin_policy: PinPolicy,
    pub repositories: Repositories,
    pub hsm: Pkcs11Hsm,
    pub certificate_signing_key: WalletCertificateSigning,
    pub instruction_result_signing_key: InstructionResultSigning,
    pub wte_issuer: HsmWteIssuer<Pkcs11Hsm>,
}

impl RouterState {
    pub async fn new_from_settings(settings: Settings) -> Result<RouterState, Box<dyn Error>> {
        let hsm = Pkcs11Hsm::new(
            settings.hsm.library_path,
            settings.hsm.user_pin,
            settings.hsm.max_sessions,
            settings.hsm.max_session_lifetime,
            settings.attestation_wrapping_key_identifier,
        )?;

        let certificate_signing_key = WalletCertificateSigning(WalletProviderEcdsaKey::new(
            settings.certificate_signing_key_identifier,
            hsm.clone(),
        ));
        let instruction_result_signing_key = InstructionResultSigning(WalletProviderEcdsaKey::new(
            settings.instruction_result_signing_key_identifier,
            hsm.clone(),
        ));

        let certificate_signing_pubkey = certificate_signing_key.verifying_key().await?;

        let account_server = AccountServer::new(
            settings.instruction_challenge_timeout,
            "account_server".into(),
            (&certificate_signing_pubkey).into(),
            settings.pin_pubkey_encryption_key_identifier,
            settings.pin_public_disclosure_protection_key_identifier,
        )?;

        let db = Db::new(
            settings.database.connection_string(),
            settings.database.connection_options,
        )
        .await?;

        let pin_policy = PinPolicy::new(
            settings.pin_policy.rounds,
            settings.pin_policy.attempts_per_round,
            settings
                .pin_policy
                .timeouts
                .into_iter()
                .map(Duration::from_std)
                .collect::<Result<_, _>>()?,
        );

        let repositories = Repositories::new(db);
        let wte_issuer = HsmWteIssuer::new(
            WalletProviderEcdsaKey::new(settings.wte_signing_key_identifier, hsm.clone()),
            settings.wte_issuer_identifier,
            hsm.clone(),
        );

        let state = RouterState {
            account_server,
            repositories,
            pin_policy,
            hsm,
            certificate_signing_key,
            instruction_result_signing_key,
            wte_issuer,
        };

        Ok(state)
    }

    pub async fn handle_instruction<I, R>(
        &self,
        instruction: Instruction<I>,
    ) -> Result<InstructionResultMessage<<I as HandleInstruction>::Result>, WalletProviderError>
    where
        I: InstructionAndResult<Result = R> + HandleInstruction<Result = R> + ValidateInstruction,
        R: Serialize + DeserializeOwned,
    {
        let result = self
            .account_server
            .handle_instruction(
                instruction,
                &self.instruction_result_signing_key,
                self,
                &self.repositories,
                &self.pin_policy,
                &self.hsm,
                &self.wte_issuer,
            )
            .await?;

        info!("Replying with the instruction result");

        Ok(InstructionResultMessage { result })
    }
}

impl Generator<uuid::Uuid> for RouterState {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

impl Generator<DateTime<Utc>> for RouterState {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
