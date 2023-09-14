use std::future::Future;
use tokio::sync::{Mutex, MutexGuard};

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::{
    messages::{
        auth::WalletCertificate,
        instructions::{
            Instruction, InstructionChallengeRequest, InstructionChallengeRequestMessage, InstructionEndpoint,
        },
    },
    serialization::Base64Bytes,
};

use crate::{
    account_provider::AccountProviderClient,
    pin::key::PinKey,
    storage::{InstructionData, Storage},
    wallet::InstructionError,
    AccountServerConfiguration,
};

pub struct InstructionClient<'a, S, A, K> {
    pin: String,
    pin_salt: &'a Base64Bytes,
    wallet_certificate: &'a WalletCertificate,
    hw_privkey: &'a K,
    account_provider_client: &'a A,
    storage: &'a Mutex<S>,
    account_provider_config: AccountServerConfiguration,
}

impl<'a, S, A, K> InstructionClient<'a, S, A, K>
where
    S: Storage,
    A: AccountProviderClient,
    K: PlatformEcdsaKey,
{
    pub fn new(
        pin: String,
        pin_salt: &'a Base64Bytes,
        wallet_certificate: &'a WalletCertificate,
        hw_privkey: &'a K,
        account_server: &'a A,
        storage: &'a Mutex<S>,
        account_provider_config: AccountServerConfiguration,
    ) -> Self {
        Self {
            pin,
            pin_salt,
            wallet_certificate,
            hw_privkey,
            account_provider_client: account_server,
            storage,
            account_provider_config,
        }
    }

    async fn with_sequence_number<F, O, R>(&self, storage: &MutexGuard<'_, S>, f: F) -> Result<R, InstructionError>
    where
        F: FnOnce(u64) -> O + 'a,
        O: Future<Output = Result<R, wallet_common::errors::Error>> + 'a,
    {
        let mut instruction_data = storage.fetch_data::<InstructionData>().await?.unwrap_or_default();
        instruction_data.instruction_sequence_number += 1;

        // A value of 1 means the default is used (0 for the default incremented by 1) and no instruction_data exists
        // in the database. Therefore, it should be inserted instead of updated.
        if instruction_data.instruction_sequence_number == 1 {
            storage.insert_data(&instruction_data).await?;
        } else {
            storage.update_data(&instruction_data).await?;
        }

        (f)(instruction_data.instruction_sequence_number)
            .await
            .map_err(InstructionError::Signing)
    }

    async fn instruction_challenge(&self, storage: &MutexGuard<'_, S>) -> Result<Vec<u8>, InstructionError> {
        let message = self
            .with_sequence_number(storage, |seq_num| {
                InstructionChallengeRequest::new_signed(seq_num, "wallet", self.hw_privkey)
            })
            .await?;

        let challenge_request = InstructionChallengeRequestMessage {
            message,
            certificate: self.wallet_certificate.clone(),
        };

        let result = self
            .account_provider_client
            .instruction_challenge(&self.account_provider_config.base_url, challenge_request)
            .await?;

        Ok(result)
    }

    pub async fn send<I>(&self, instruction: I) -> Result<I::Result, InstructionError>
    where
        I: InstructionEndpoint + Send + Sync + 'static,
    {
        let storage = self.storage.lock().await;

        let challenge = self.instruction_challenge(&storage).await?;

        let pin_key = PinKey::new(&self.pin, &self.pin_salt.0);

        let instruction = self
            .with_sequence_number(&storage, |seq_num| {
                Instruction::new_signed(
                    instruction,
                    seq_num,
                    self.hw_privkey,
                    &pin_key,
                    &challenge,
                    self.wallet_certificate.clone(),
                )
            })
            .await?;

        let signed_result = self
            .account_provider_client
            .instruction(&self.account_provider_config.base_url, instruction)
            .await
            .map_err(InstructionError::from)?;

        let result = signed_result
            .parse_and_verify(&self.account_provider_config.instruction_result_public_key)
            .map_err(InstructionError::InstructionResultValidation)?
            .result;

        Ok(result)
    }
}
