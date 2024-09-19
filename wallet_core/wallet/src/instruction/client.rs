use std::future::Future;
use tokio::sync::{RwLock, RwLockWriteGuard};

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::messages::instructions::{Instruction, InstructionAndResult, InstructionChallengeRequest},
    jwt::EcdsaDecodingKey,
    urls::BaseUrl,
};

use crate::{
    account_provider::AccountProviderClient,
    pin::key::PinKey,
    storage::{InstructionData, RegistrationData, Storage},
};

use super::InstructionError;

pub struct InstructionClient<'a, S, K, A> {
    pin: String,
    storage: &'a RwLock<S>,
    hw_privkey: &'a K,
    account_provider_client: &'a A,
    registration: &'a RegistrationData,
    account_provider_base_url: &'a BaseUrl,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, K, A> InstructionClient<'a, S, K, A>
where
    S: Storage,
    K: PlatformEcdsaKey,
    A: AccountProviderClient,
{
    pub fn new(
        pin: String,
        storage: &'a RwLock<S>,
        hw_privkey: &'a K,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        account_provider_base_url: &'a BaseUrl,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            pin,
            storage,
            hw_privkey,
            account_provider_client,
            registration,
            account_provider_base_url,
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
                    seq_num,
                    self.hw_privkey,
                    self.registration.wallet_certificate.clone(),
                )
            })
            .await?;

        let result = self
            .account_provider_client
            .instruction_challenge(self.account_provider_base_url, challenge_request)
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
            .instruction(self.account_provider_base_url, instruction)
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
    account_provider_base_url: &'a BaseUrl,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, K, A> InstructionClientFactory<'a, S, K, A> {
    pub fn new(
        storage: &'a RwLock<S>,
        hw_privkey: &'a K,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        account_provider_base_url: &'a BaseUrl,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            storage,
            hw_privkey,
            account_provider_client,
            registration,
            account_provider_base_url,
            instruction_result_public_key,
        }
    }

    pub fn create(&self, pin: String) -> InstructionClient<'a, S, K, A> {
        InstructionClient {
            pin,
            storage: self.storage,
            hw_privkey: self.hw_privkey,
            account_provider_client: self.account_provider_client,
            registration: self.registration,
            account_provider_base_url: self.account_provider_base_url,
            instruction_result_public_key: self.instruction_result_public_key,
        }
    }
}
