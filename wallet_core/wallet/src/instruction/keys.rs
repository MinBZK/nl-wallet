use std::iter;

use p256::ecdsa::{signature, signature::Verifier, Signature, VerifyingKey};

use nl_wallet_mdoc::utils::keys::{CredentialKeyType, KeyFactory, MdocEcdsaKey};
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::messages::instructions::{GenerateKey, GenerateKeyResult, Sign},
    keys::{EcdsaKey, SecureEcdsaKey, WithIdentifier},
    utils::random_string,
};

use crate::{account_provider::AccountProviderClient, storage::Storage};

use super::{InstructionClient, InstructionError};

#[derive(Debug, thiserror::Error)]
pub enum RemoteEcdsaKeyError {
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    Signature(#[from] signature::Error),
    #[error("no signature received from Wallet Provider")]
    MissingSignature,
    #[error("key '{0}' not found in Wallet Provider")]
    KeyNotFound(String),
}

pub struct RemoteEcdsaKeyFactory<'a, S, K, A> {
    instruction_client: &'a InstructionClient<'a, S, K, A>,
}

pub struct RemoteEcdsaKey<'a, S, K, A> {
    identifier: String,
    public_key: VerifyingKey,
    key_factory: &'a RemoteEcdsaKeyFactory<'a, S, K, A>,
}

impl<'a, S, K, A> RemoteEcdsaKeyFactory<'a, S, K, A> {
    pub fn new(instruction_client: &'a InstructionClient<'a, S, K, A>) -> Self {
        Self { instruction_client }
    }
}

impl<'a, S, K, A> KeyFactory for &'a RemoteEcdsaKeyFactory<'a, S, K, A>
where
    S: Storage + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
    type Key = RemoteEcdsaKey<'a, S, K, A>;
    type Error = RemoteEcdsaKeyError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        let identifiers = iter::repeat_with(|| random_string(32)).take(count as usize).collect();
        let result: GenerateKeyResult = self.instruction_client.send(GenerateKey { identifiers }).await?;

        let keys = result
            .public_keys
            .into_iter()
            .map(|(identifier, public_key)| RemoteEcdsaKey {
                identifier,
                public_key: public_key.0,
                key_factory: self,
            })
            .collect();

        Ok(keys)
    }

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        RemoteEcdsaKey {
            identifier: identifier.into(),
            public_key,
            key_factory: self,
        }
    }

    async fn sign_with_new_keys(
        &self,
        msg: Vec<u8>,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;

        let signatures = self
            .sign_multiple_with_existing_keys(vec![(msg, keys.iter().collect())])
            .await?;

        let result = keys
            .into_iter()
            .zip(
                signatures
                    .first()
                    .cloned()
                    .ok_or_else(|| RemoteEcdsaKeyError::MissingSignature)?,
            )
            .collect::<Vec<_>>();

        Ok(result)
    }

    async fn sign_multiple_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error> {
        let sign_result = self
            .instruction_client
            .send(Sign {
                messages_with_identifiers: messages_and_keys
                    .into_iter()
                    .map(|(message, keys)| {
                        let identifiers = keys.into_iter().map(|key| key.identifier.clone()).collect();
                        (message, identifiers)
                    })
                    .collect(),
            })
            .await?;

        let signatures = sign_result
            .signatures
            .into_iter()
            .map(|signatures| signatures.into_iter().map(|signature| signature.0).collect())
            .collect();

        Ok(signatures)
    }
}

impl<S, K, A> WithIdentifier for RemoteEcdsaKey<'_, S, K, A> {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl<S, K, A> EcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
    type Error = RemoteEcdsaKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(self.public_key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let result = self
            .key_factory
            .instruction_client
            .send(Sign {
                messages_with_identifiers: vec![(msg.to_vec(), vec![self.identifier.clone()])],
            })
            .await?;

        let signature = result
            .signatures
            .first()
            .and_then(|r| r.first())
            .ok_or(RemoteEcdsaKeyError::KeyNotFound(self.identifier.clone()))?;

        self.public_key.verify(msg, &signature.0)?;

        Ok(signature.0)
    }
}

impl<S, K, A> SecureEcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
}

impl<S, K, A> MdocEcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
    const KEY_TYPE: CredentialKeyType = CredentialKeyType::Remote;
}
