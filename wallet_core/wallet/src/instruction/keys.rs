use std::hash::Hash;
use std::hash::Hasher;
use std::iter;

use itertools::Itertools;
use p256::ecdsa::signature;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use platform_support::attested_key::GoogleAttestedKey;
use wallet_common::account::messages::instructions::ConstructPoa;
use wallet_common::account::messages::instructions::GenerateKey;
use wallet_common::account::messages::instructions::GenerateKeyResult;
use wallet_common::account::messages::instructions::Sign;
use wallet_common::apple::AppleAttestedKey;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::poa::Poa;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::CredentialKeyType;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;
use wallet_common::keys::WithIdentifier;
use wallet_common::utils::random_string;
use wallet_common::vec_at_least::VecAtLeastTwoUnique;

use crate::account_provider::AccountProviderClient;
use crate::storage::Storage;

use super::InstructionClient;
use super::InstructionError;

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

pub struct RemoteEcdsaKeyFactory<'a, S, AK, GK, A> {
    instruction_client: &'a InstructionClient<'a, S, AK, GK, A>,
}

pub struct RemoteEcdsaKey<'a, S, AK, GK, A> {
    identifier: String,
    public_key: VerifyingKey,
    key_factory: &'a RemoteEcdsaKeyFactory<'a, S, AK, GK, A>,
}

impl<S, AK, GK, A> PartialEq for RemoteEcdsaKey<'_, S, AK, GK, A> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<S, AK, GK, A> Eq for RemoteEcdsaKey<'_, S, AK, GK, A> {}

impl<S, AK, GK, A> Hash for RemoteEcdsaKey<'_, S, AK, GK, A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl<'a, S, AK, GK, A> RemoteEcdsaKeyFactory<'a, S, AK, GK, A> {
    pub fn new(instruction_client: &'a InstructionClient<'a, S, AK, GK, A>) -> Self {
        Self { instruction_client }
    }
}

impl<'a, S, AK, GK, A> KeyFactory for &'a RemoteEcdsaKeyFactory<'a, S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    type Key = RemoteEcdsaKey<'a, S, AK, GK, A>;
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

    async fn poa(
        &self,
        keys: VecAtLeastTwoUnique<&Self::Key>,
        aud: String,
        nonce: Option<String>,
    ) -> Result<Poa, Self::Error> {
        let poa = self
            .instruction_client
            .send(ConstructPoa {
                key_identifiers: keys
                    .as_ref()
                    .iter()
                    .map(|key| key.identifier.clone())
                    .collect_vec()
                    .try_into()
                    .unwrap(), // our iterable is a VecAtLeastTwoUnique
                aud,
                nonce,
            })
            .await?
            .poa;

        Ok(poa)
    }
}

impl<S, AK, GK, A> WithIdentifier for RemoteEcdsaKey<'_, S, AK, GK, A> {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl<S, AK, GK, A> EcdsaKey for RemoteEcdsaKey<'_, S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
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

impl<S, AK, GK, A> SecureEcdsaKey for RemoteEcdsaKey<'_, S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
}

impl<S, AK, GK, A> CredentialEcdsaKey for RemoteEcdsaKey<'_, S, AK, GK, A>
where
    S: Storage,
    AK: AppleAttestedKey,
    GK: GoogleAttestedKey,
    A: AccountProviderClient,
{
    const KEY_TYPE: CredentialKeyType = CredentialKeyType::Remote;
}
