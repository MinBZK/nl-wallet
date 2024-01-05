use std::iter;

use p256::ecdsa::{signature, signature::Verifier, Signature, VerifyingKey};

use nl_wallet_mdoc::utils::keys::{KeyFactory, MdocEcdsaKey, MdocKeyType};
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
    S: Storage + Send + Sync,
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

    async fn sign_with_new_keys<T: Into<Vec<u8>>>(
        &self,
        msg: T,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;

        let result = self
            .instruction_client
            .send(Sign {
                msg_with_identifiers: (
                    msg.into().into(),
                    keys.iter().map(|key| key.identifier.clone()).collect(),
                ),
            })
            .await?;

        keys.into_iter()
            .map(|key| {
                result
                    .signatures_by_identifier
                    .get(key.identifier())
                    .ok_or(RemoteEcdsaKeyError::KeyNotFound(key.identifier().to_string()))
                    .map(|der_signature| (key, der_signature.0))
            })
            .collect()
    }
}

impl<S, K, A> WithIdentifier for RemoteEcdsaKey<'_, S, K, A> {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl<S, K, A> EcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Send + Sync,
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
                msg_with_identifiers: (msg.to_vec().into(), vec![self.identifier.clone()]),
            })
            .await?;

        let signature = result
            .signatures_by_identifier
            .get(&self.identifier)
            .ok_or(RemoteEcdsaKeyError::KeyNotFound(self.identifier.clone()))?;

        self.public_key.verify(msg, &signature.0)?;

        Ok(signature.0)
    }
}

impl<S, K, A> SecureEcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
}

impl<S, K, A> MdocEcdsaKey for RemoteEcdsaKey<'_, S, K, A>
where
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
    const KEY_TYPE: MdocKeyType = MdocKeyType::Remote;
}
