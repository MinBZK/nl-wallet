use std::future::Future;

use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::messages::instructions::Instruction;
use wallet_common::account::messages::instructions::InstructionAndResult;
use wallet_common::account::messages::instructions::InstructionChallengeRequest;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::jwt::EcdsaDecodingKey;

use crate::account_provider::AccountProviderClient;
use crate::pin::key::PinKey;
use crate::storage::InstructionData;
use crate::storage::RegistrationData;
use crate::storage::Storage;

use super::InstructionError;

pub struct InstructionClient<'a, S, K, A> {
    pin: String,
    storage: &'a RwLock<S>,
    hw_privkey: &'a K,
    account_provider_client: &'a A,
    registration: &'a RegistrationData,
    client_config: &'a TlsPinningConfig,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, K, A> InstructionClient<'a, S, K, A>
where
    S: Storage,
    K: PlatformEcdsaKey,
    A: AccountProviderClient,
{
    /// Creates an [`InstructionClient`].
    /// In most cases this function should not be used directly, as the wallet must try to finalize
    /// a PIN change if it is in progress. [`Wallet::new_instruction_client`] will do this before
    /// returning the [`InstructionClient`] and so is the recommended way to obtain an [`InstructionClient`].
    pub fn new(
        pin: String,
        storage: &'a RwLock<S>,
        hw_privkey: &'a K,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        client_config: &'a TlsPinningConfig,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            pin,
            storage,
            hw_privkey,
            account_provider_client,
            registration,
            client_config,
            instruction_result_public_key,
        }
    }

    async fn with_sequence_number<F, O, R>(
        &self,
        storage: &mut RwLockWriteGuard<'_, S>,
        f: F,
    ) -> Result<R, InstructionError>
    where
        F: FnOnce(u64) -> O,
        O: Future<Output = Result<R, wallet_common::account::errors::Error>>,
    {
        let mut instruction_data = storage.fetch_data::<InstructionData>().await?.unwrap_or_default();
        instruction_data.instruction_sequence_number += 1;

        storage.upsert_data(&instruction_data).await?;

        (f)(instruction_data.instruction_sequence_number)
            .await
            .map_err(InstructionError::Signing)
    }

    async fn instruction_challenge<I>(&self, storage: &mut RwLockWriteGuard<'_, S>) -> Result<Vec<u8>, InstructionError>
    where
        I: InstructionAndResult,
    {
        let challenge_request = self
            .with_sequence_number(storage, |seq_num| {
                InstructionChallengeRequest::new_signed::<I>(
                    self.registration.wallet_id.clone(),
                    seq_num,
                    self.hw_privkey,
                    self.registration.wallet_certificate.clone(),
                )
            })
            .await?;

        let result = self
            .account_provider_client
            .instruction_challenge(self.client_config, challenge_request)
            .await?;

        Ok(result)
    }

    pub async fn send<I>(&self, instruction: I) -> Result<I::Result, InstructionError>
    where
        I: InstructionAndResult + 'static,
    {
        self.construct_and_send(|_| async { Ok(instruction) }).await
    }

    pub async fn construct_and_send<F, Fut, I>(&self, construct: F) -> Result<I::Result, InstructionError>
    where
        F: FnOnce(Vec<u8>) -> Fut,
        Fut: Future<Output = Result<I, InstructionError>>,
        I: InstructionAndResult + 'static,
    {
        let mut storage = self.storage.write().await;

        let challenge = self.instruction_challenge::<I>(&mut storage).await?;

        let pin_key = PinKey::new(&self.pin, &self.registration.pin_salt);

        let instruction = construct(challenge.clone()).await?;

        let instruction = self
            .with_sequence_number(&mut storage, |seq_num| {
                Instruction::new_signed(
                    instruction,
                    challenge,
                    seq_num,
                    self.hw_privkey,
                    &pin_key,
                    self.registration.wallet_certificate.clone(),
                )
            })
            .await?;

        let signed_result = self
            .account_provider_client
            .instruction(self.client_config, instruction)
            .await
            .map_err(InstructionError::from)?;

        let result = signed_result
            .parse_and_verify_with_sub(self.instruction_result_public_key)
            .map_err(InstructionError::InstructionResultValidation)?
            .result;

        Ok(result)
    }
}

pub struct InstructionClientFactory<'a, S, K, A> {
    storage: &'a RwLock<S>,
    hw_privkey: &'a K,
    account_provider_client: &'a A,
    registration: &'a RegistrationData,
    client_config: &'a TlsPinningConfig,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, K, A> InstructionClientFactory<'a, S, K, A> {
    pub fn new(
        storage: &'a RwLock<S>,
        hw_privkey: &'a K,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        client_config: &'a TlsPinningConfig,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            storage,
            hw_privkey,
            account_provider_client,
            registration,
            client_config,
            instruction_result_public_key,
        }
    }

    /// Creates an [`InstructionClient`].
    /// See [`InstructionClient::new`].
    pub fn create(&self, pin: String) -> InstructionClient<'a, S, K, A>
    where
        S: Storage,
        K: PlatformEcdsaKey,
        A: AccountProviderClient,
    {
        InstructionClient::new(
            pin,
            self.storage,
            self.hw_privkey,
            self.account_provider_client,
            self.registration,
            self.client_config,
            self.instruction_result_public_key,
        )
    }
}
