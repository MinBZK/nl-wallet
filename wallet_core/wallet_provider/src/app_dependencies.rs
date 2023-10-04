use std::error::Error;

use chrono::{DateTime, Duration, Local};
use serde::{de::DeserializeOwned, Serialize};
use tracing::info;
use uuid::Uuid;
use wallet_common::account::messages::instructions::{Instruction, InstructionEndpoint, InstructionResultMessage};

use wallet_common::generator::Generator;

use crate::errors::WalletProviderError;
use wallet_provider_persistence::{database::Db, repositories::Repositories};
use wallet_provider_service::{account_server::AccountServer, instructions::HandleInstruction, pin_policy::PinPolicy};

use crate::settings::Settings;

pub struct AppDependencies {
    pub account_server: AccountServer,
    pub pin_policy: PinPolicy,
    pub repositories: Repositories,
}

impl AppDependencies {
    pub async fn new_from_settings(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let account_server = AccountServer::new(
            settings.certificate_private_key.into(),
            settings.instruction_result_private_key.into(),
            settings.pin_hash_salt.0,
            settings.instruction_challenge_timeout_in_ms,
            "account_server".into(),
        )
        .await?;

        let db = Db::new(
            &settings.database.host,
            &settings.database.name,
            settings.database.username.as_deref(),
            settings.database.password.as_deref(),
        )
        .await?;

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
            .handle_instruction(instruction, self, &self.repositories, &self.pin_policy, self)
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
