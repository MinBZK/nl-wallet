use std::future::Future;
use std::sync::Arc;

use derive_more::Constructor;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

use http_utils::tls::pinning::TlsPinningConfig;
use jwt::EcdsaDecodingKey;
use platform_support::attested_key::AppleAttestedKey;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::GoogleAttestedKey;
use wallet_account::messages::instructions::HwSignedInstruction;
use wallet_account::messages::instructions::Instruction;
use wallet_account::messages::instructions::InstructionAndResult;
use wallet_account::messages::instructions::InstructionChallengeRequest;

use crate::account_provider::AccountProviderClient;
use crate::pin::key::PinKey;
use crate::storage::InstructionData;
use crate::storage::RegistrationData;
use crate::storage::Storage;

use super::InstructionError;

pub struct InstructionClient<S, AK, GK, A> {
    pin: String,
    hw_signed_instruction_client: HwSignedInstructionClient<S, AK, GK, A>,
}

pub struct HwSignedInstructionClient<S, AK, GK, A> {
    storage: Arc<RwLock<S>>,
    attested_key: Arc<AttestedKey<AK, GK>>,
    account_provider_client: Arc<A>,
    parameters: Arc<InstructionClientParameters>,
}

#[derive(Constructor)]
pub struct InstructionClientParameters {
    registration: RegistrationData,
    client_config: TlsPinningConfig,
    instruction_result_public_key: EcdsaDecodingKey,
}

// Manually implement clone in order to prevent Clone trait bounds on the generics.
impl<S, AK, GK, A> Clone for InstructionClient<S, AK, GK, A> {
    fn clone(&self) -> Self {
        Self {
            pin: self.pin.clone(),
            hw_signed_instruction_client: self.hw_signed_instruction_client.clone(),
        }
    }
}

// Manually implement clone in order to prevent Clone trait bounds on the generics.
impl<S, AK, GK, A> Clone for HwSignedInstructionClient<S, AK, GK, A> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            attested_key: Arc::clone(&self.attested_key),
            account_provider_client: Arc::clone(&self.account_provider_client),
            parameters: Arc::clone(&self.parameters),
        }
    }
}

impl<S, AK, GK, A> InstructionClient<S, AK, GK, A> {
    /// Creates an [`InstructionClient`].
    /// In most cases this function should not be used directly, as the wallet must try to finalize
    /// a PIN change if it is in progress. [`Wallet::new_instruction_client`] will do this before
    /// returning the [`InstructionClient`] and so is the recommended way to obtain an [`InstructionClient`].
    pub fn new(
        pin: String,
        storage: Arc<RwLock<S>>,
        attested_key: Arc<AttestedKey<AK, GK>>,
        account_provider_client: Arc<A>,
        parameters: Arc<InstructionClientParameters>,
    ) -> Self {
        Self {
            pin,
            hw_signed_instruction_client: HwSignedInstructionClient::new(
                storage,
                attested_key,
                account_provider_client,
                parameters,
            ),
        }
    }
}

impl<S, AK, GK, A> HwSignedInstructionClient<S, AK, GK, A> {
    pub fn new(
        storage: Arc<RwLock<S>>,
        attested_key: Arc<AttestedKey<AK, GK>>,
        account_provider_client: Arc<A>,
        parameters: Arc<InstructionClientParameters>,
    ) -> Self {
        Self {
            storage,
            attested_key,
            account_provider_client,
            parameters,
        }
    }

    async fn with_sequence_number<F, O, R>(storage: &mut RwLockWriteGuard<'_, S>, f: F) -> Result<R, InstructionError>
    where
        S: Storage,
        F: FnOnce(u64) -> O,
        O: Future<Output = Result<R, wallet_account::error::EncodeError>>,
    {
        let mut instruction_data = storage.fetch_data::<InstructionData>().await?.unwrap_or_default();
        instruction_data.instruction_sequence_number += 1;

        storage.upsert_data(&instruction_data).await?;

        (f)(instruction_data.instruction_sequence_number)
            .await
            .map_err(InstructionError::Signing)
    }
}

impl<S, AK, GK, A> HwSignedInstructionClient<S, AK, GK, A>
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
        let registration = &self.parameters.registration;
        let wallet_id = registration.wallet_id.clone();
        let wallet_certificate = registration.wallet_certificate.clone();

        let challenge_request = Self::with_sequence_number(storage, |seq_num| async move {
            match self.attested_key.as_ref() {
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
            .instruction_challenge(&self.parameters.client_config, challenge_request)
            .await?;

        Ok(result)
    }

    pub async fn send<I>(&self, instruction: I) -> Result<I::Result, InstructionError>
    where
        I: InstructionAndResult + 'static,
    {
        let mut storage = self.storage.write().await;

        let challenge = self.instruction_challenge::<I>(&mut storage).await?;

        let wallet_certificate = self.parameters.registration.wallet_certificate.clone();

        let instruction =
            HwSignedInstructionClient::<S, AK, GK, A>::with_sequence_number(&mut storage, |seq_num| async move {
                match self.attested_key.as_ref() {
                    AttestedKey::Apple(key) => {
                        HwSignedInstruction::new_apple(instruction, challenge, seq_num, key, wallet_certificate).await
                    }
                    AttestedKey::Google(key) => {
                        HwSignedInstruction::new_google(instruction, challenge, seq_num, key, wallet_certificate).await
                    }
                }
            })
            .await?;

        let signed_result = self
            .account_provider_client
            .hw_signed_instruction(&self.parameters.client_config, instruction)
            .await
            .map_err(InstructionError::from)?;

        let result = signed_result
            .parse_and_verify_with_sub(&self.parameters.instruction_result_public_key)
            .map_err(InstructionError::InstructionResultValidation)?
            .1
            .result;

        Ok(result)
    }
}

impl<S, AK, GK, A> InstructionClient<S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
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
        let mut storage = self.hw_signed_instruction_client.storage.write().await;

        let challenge = self
            .hw_signed_instruction_client
            .instruction_challenge::<I>(&mut storage)
            .await?;

        let pin_key = PinKey {
            pin: &self.pin,
            salt: &self.hw_signed_instruction_client.parameters.registration.pin_salt,
        };

        let instruction = construct(challenge.clone()).await?;

        let wallet_certificate = self
            .hw_signed_instruction_client
            .parameters
            .registration
            .wallet_certificate
            .clone();

        let instruction =
            HwSignedInstructionClient::<S, AK, GK, A>::with_sequence_number(&mut storage, |seq_num| async move {
                match self.hw_signed_instruction_client.attested_key.as_ref() {
                    AttestedKey::Apple(key) => {
                        Instruction::new_apple(instruction, challenge, seq_num, key, &pin_key, wallet_certificate).await
                    }
                    AttestedKey::Google(key) => {
                        Instruction::new_google(instruction, challenge, seq_num, key, &pin_key, wallet_certificate)
                            .await
                    }
                }
            })
            .await?;

        let signed_result = self
            .hw_signed_instruction_client
            .account_provider_client
            .instruction(&self.hw_signed_instruction_client.parameters.client_config, instruction)
            .await
            .map_err(InstructionError::from)?;

        let result = signed_result
            .parse_and_verify_with_sub(
                &self
                    .hw_signed_instruction_client
                    .parameters
                    .instruction_result_public_key,
            )
            .map_err(InstructionError::InstructionResultValidation)?
            .1
            .result;

        Ok(result)
    }
}

pub struct InstructionClientFactory<S, AK, GK, A> {
    storage: Arc<RwLock<S>>,
    attested_key: Arc<AttestedKey<AK, GK>>,
    account_provider_client: Arc<A>,
    parameters: Arc<InstructionClientParameters>,
}

impl<S, AK, GK, A> InstructionClientFactory<S, AK, GK, A> {
    pub fn new(
        storage: Arc<RwLock<S>>,
        attested_key: Arc<AttestedKey<AK, GK>>,
        account_provider_client: Arc<A>,
        parameters: InstructionClientParameters,
    ) -> Self {
        Self {
            storage,
            attested_key,
            account_provider_client,
            parameters: Arc::new(parameters),
        }
    }

    /// Creates an [`InstructionClient`].
    /// See [`InstructionClient::new`].
    pub fn create(&self, pin: String) -> InstructionClient<S, AK, GK, A> {
        InstructionClient {
            pin,
            hw_signed_instruction_client: HwSignedInstructionClient::<S, AK, GK, A>::new(
                Arc::clone(&self.storage),
                Arc::clone(&self.attested_key),
                Arc::clone(&self.account_provider_client),
                Arc::clone(&self.parameters),
            ),
        }
    }
}
