use std::error::Error;

use chrono::{DateTime, Duration, Local};
use serde::{de::DeserializeOwned, Serialize};
use tracing::info;
use uuid::Uuid;

use wallet_common::{
    account::messages::instructions::{Instruction, InstructionEndpoint, InstructionResultMessage},
    generator::Generator,
    keys::EcdsaKey,
};
use wallet_provider_persistence::{database::Db, repositories::Repositories};
use wallet_provider_service::{
    account_server::AccountServer,
    hsm::Pkcs11Hsm,
    instructions::HandleInstruction,
    keys::{CertificateSigning, InstructionResultSigning, WalletProviderEcdsaKey},
    pin_policy::PinPolicy,
};

use crate::{errors::WalletProviderError, settings::Settings};

pub struct AppDependencies {
    pub account_server: AccountServer,
    pub pin_policy: PinPolicy,
    pub repositories: Repositories,
    pub hsm: Pkcs11Hsm,
    pub certificate_signing_key: CertificateSigning,
    pub instruction_result_signing_key: InstructionResultSigning,
}

impl AppDependencies {
    pub async fn new_from_settings(settings: Settings) -> Result<AppDependencies, Box<dyn Error>> {
        let hsm = Pkcs11Hsm::new(settings.hsm.library_path, settings.hsm.user_pin)?;

        let certificate_signing_key = CertificateSigning(WalletProviderEcdsaKey::new(
            settings.certificate_signing_key_identifier,
            hsm.clone(),
        ));
        let instruction_result_signing_key = InstructionResultSigning(WalletProviderEcdsaKey::new(
            settings.instruction_result_signing_key_identifier,
            hsm.clone(),
        ));

        let account_server = AccountServer::new(
            settings.pin_hash_salt.0,
            settings.instruction_challenge_timeout_in_ms,
            "account_server".into(),
            certificate_signing_key.verifying_key().await?.into(),
        )
        .await?;

        let db = Db::new(settings.database.connection_string()).await?;

        let pin_policy = PinPolicy::new(
            settings.pin_policy.rounds,
            settings.pin_policy.attempts_per_round,
            settings
                .pin_policy
                .timeouts_in_ms
                .into_iter()
                .map(|t| Duration::milliseconds(i64::from(t)))
                .collect(),
        );

        let repositories = Repositories::new(db);

        let dependencies = AppDependencies {
            account_server,
            repositories,
            pin_policy,
            hsm,
            certificate_signing_key,
            instruction_result_signing_key,
        };

        Ok(dependencies)
    }

    pub async fn handle_instruction<I, R>(
        &self,
        instruction: Instruction<I>,
    ) -> Result<InstructionResultMessage<<I as HandleInstruction>::Result>, WalletProviderError>
    where
        I: InstructionEndpoint<Result = R> + HandleInstruction<Result = R>,
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
            )
            .await?;

        info!("Replying with the instruction result");

        Ok(InstructionResultMessage { result })
    }
}

impl Generator<uuid::Uuid> for AppDependencies {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

impl Generator<DateTime<Local>> for AppDependencies {
    fn generate(&self) -> DateTime<Local> {
        Local::now()
    }
}
