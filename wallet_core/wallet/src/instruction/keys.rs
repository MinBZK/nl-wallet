use async_trait::async_trait;
use p256::ecdsa::{signature, signature::Verifier, Signature, VerifyingKey};

use nl_wallet_mdoc::utils::keys::{KeyFactory, MdocEcdsaKey, MdocKeyType};
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{
    account::messages::instructions::{GenerateKey, GenerateKeyResult, Sign, SignResult},
    keys::{EcdsaKey, SecureEcdsaKey, WithIdentifier},
};

use crate::{account_provider::AccountProviderClient, storage::Storage};

use super::{InstructionClient, InstructionError};

#[derive(Debug, thiserror::Error)]
pub enum RemoteEcdsaKeyError {
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    Signature(#[from] signature::Error),
}

pub struct RemoteEcdsaKeyFactory<'a, S, K, A> {
    remote_instruction: &'a InstructionClient<'a, S, K, A>,
}

pub struct RemoteEcdsaKey<'a, S, K, A> {
    identifier: String,
    public_key: VerifyingKey,
    key_factory: &'a RemoteEcdsaKeyFactory<'a, S, K, A>,
}

impl<'a, S, K, A> RemoteEcdsaKeyFactory<'a, S, K, A> {
    pub fn new(remote_instruction: &'a InstructionClient<'a, S, K, A>) -> Self {
        Self { remote_instruction }
    }
}

#[async_trait]
impl<'a, S, K, A> KeyFactory<'a> for RemoteEcdsaKeyFactory<'a, S, K, A>
where
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey + Sync,
    A: AccountProviderClient + Sync,
{
    type Key = RemoteEcdsaKey<'a, S, K, A>;
    type Error = RemoteEcdsaKeyError;

    async fn generate<I: AsRef<str> + Sync>(&'a self, identifiers: &[I]) -> Result<Vec<Self::Key>, Self::Error> {
        let generate_key = GenerateKey {
            identifiers: identifiers.iter().map(|i| i.as_ref().to_owned()).collect(),
        };
        let result: GenerateKeyResult = self.remote_instruction.send(generate_key).await?;

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
}

impl<S, K, A> WithIdentifier for RemoteEcdsaKey<'_, S, K, A> {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[async_trait]
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
        let result: SignResult = self
            .key_factory
            .remote_instruction
            .send(Sign {
                msg: msg.to_vec().into(),
                identifier: self.identifier.clone(),
            })
            .await?;

        self.public_key.verify(msg, &result.signature.0)?;

        Ok(result.signature.0)
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
