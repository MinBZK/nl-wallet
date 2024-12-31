use std::future::Future;

use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_common::account::messages::instructions::Instruction;
use wallet_common::account::messages::instructions::InstructionAndResult;
use wallet_common::account::messages::instructions::InstructionChallengeRequest;
use wallet_common::apple::AppleAttestedKey;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::jwt::EcdsaDecodingKey;

use crate::account_provider::AccountProviderClient;
use crate::pin::key::PinKey;
use crate::storage::InstructionData;
use crate::storage::RegistrationData;
use crate::storage::Storage;

use super::InstructionError;

pub struct InstructionClient<'a, S, AK, GK, A> {
    pin: String,
    storage: &'a RwLock<S>,
    attested_key: &'a AttestedKey<AK, GK>,
    account_provider_client: &'a A,
    registration: &'a RegistrationData,
    client_config: &'a TlsPinningConfig,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, AK, GK, A> InstructionClient<'a, S, AK, GK, A> {
    /// Creates an [`InstructionClient`].
    /// In most cases this function should not be used directly, as the wallet must try to finalize
    /// a PIN change if it is in progress. [`Wallet::new_instruction_client`] will do this before
    /// returning the [`InstructionClient`] and so is the recommended way to obtain an [`InstructionClient`].
    pub fn new(
        pin: String,
        storage: &'a RwLock<S>,
        attested_key: &'a AttestedKey<AK, GK>,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        client_config: &'a TlsPinningConfig,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            pin,
            storage,
            attested_key,
            account_provider_client,
            registration,
            client_config,
            instruction_result_public_key,
        }
    }

    async fn with_sequence_number<F, O, R>(storage: &mut RwLockWriteGuard<'_, S>, f: F) -> Result<R, InstructionError>
    where
        S: Storage,
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
}

impl<'a, S, AK, GK, A> InstructionClient<'a, S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    async fn instruction_challenge<I>(&self, storage: &mut RwLockWriteGuard<'_, S>) -> Result<Vec<u8>, InstructionError>
    where
        I: InstructionAndResult,
    {
        let wallet_id = self.registration.wallet_id.clone();
        let wallet_certificate = self.registration.wallet_certificate.clone();

        let challenge_request = Self::with_sequence_number(storage, |seq_num| async move {
            match self.attested_key {
                AttestedKey::Apple(key) => {
                    InstructionChallengeRequest::new_apple::<I>(wallet_id, seq_num, key, wallet_certificate).await
                }
                AttestedKey::Google(key) => {
                    InstructionChallengeRequest::new_google::<I>(wallet_id, seq_num, key, wallet_certificate).await
                }
            }
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

        let wallet_certificate = self.registration.wallet_certificate.clone();

        let instruction = Self::with_sequence_number(&mut storage, |seq_num| async move {
            match self.attested_key {
                AttestedKey::Apple(key) => {
                    Instruction::new_apple(instruction, challenge, seq_num, key, &pin_key, wallet_certificate).await
                }
                AttestedKey::Google(key) => {
                    Instruction::new_google(instruction, challenge, seq_num, key, &pin_key, wallet_certificate).await
                }
            }
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

pub struct InstructionClientFactory<'a, S, AK, GK, A> {
    storage: &'a RwLock<S>,
    attested_key: &'a AttestedKey<AK, GK>,
    account_provider_client: &'a A,
    registration: &'a RegistrationData,
    client_config: &'a TlsPinningConfig,
    instruction_result_public_key: &'a EcdsaDecodingKey,
}

impl<'a, S, AK, GK, A> InstructionClientFactory<'a, S, AK, GK, A> {
    pub fn new(
        storage: &'a RwLock<S>,
        attested_key: &'a AttestedKey<AK, GK>,
        account_provider_client: &'a A,
        registration: &'a RegistrationData,
        client_config: &'a TlsPinningConfig,
        instruction_result_public_key: &'a EcdsaDecodingKey,
    ) -> Self {
        Self {
            storage,
            attested_key,
            account_provider_client,
            registration,
            client_config,
            instruction_result_public_key,
        }
    }

    /// Creates an [`InstructionClient`].
    /// See [`InstructionClient::new`].
    pub fn create(&self, pin: String) -> InstructionClient<'a, S, AK, GK, A> {
        InstructionClient::new(
            pin,
            self.storage,
            self.attested_key,
            self.account_provider_client,
            self.registration,
            self.client_config,
            self.instruction_result_public_key,
        )
    }
}
