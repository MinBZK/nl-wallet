use std::collections::HashMap;
use std::iter;

use derive_more::Debug;
use futures::future;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use parking_lot::Mutex;
use rand_core::OsRng;

use crate::jwt::JwtPopClaims;
use crate::jwt::NL_WALLET_CLIENT_ID;
use crate::utils;

use super::factory::KeyFactory;
use super::poa::Poa;
use super::poa::PoaError;
use super::poa::VecAtLeastTwo;
use super::CredentialEcdsaKey;
use super::CredentialKeyType;
use super::EcdsaKey;
use super::SecureEcdsaKey;
use super::WithIdentifier;

#[derive(Debug, thiserror::Error)]
pub enum MockRemoteKeyFactoryError {
    #[error("key generation error")]
    Generating,
    #[error("signing error")]
    Signing,
    #[error("ECDSA error: {0}")]
    Ecdsa(#[source] <MockRemoteEcdsaKey as EcdsaKey>::Error),
    #[error("PoA error: {0}")]
    Poa(#[from] PoaError),
}

/// To be used in test in place of `RemoteEcdsaKey`, implementing the
/// [`EcdsaKey`], [`SecureEcdsaKey`] and [`WithIdentifier`] traits.
#[derive(Debug, Clone)]
pub struct MockRemoteEcdsaKey {
    identifier: String,
    #[debug(skip)]
    key: SigningKey,
}

impl MockRemoteEcdsaKey {
    pub fn new(identifier: String, key: SigningKey) -> Self {
        Self { identifier, key }
    }

    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, SigningKey::random(&mut OsRng))
    }
}

impl EcdsaKey for MockRemoteEcdsaKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let key = self.key.verifying_key();

        Ok(*key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Signer::try_sign(&self.key, msg)
    }
}
impl SecureEcdsaKey for MockRemoteEcdsaKey {}

impl WithIdentifier for MockRemoteEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl CredentialEcdsaKey for MockRemoteEcdsaKey {
    const KEY_TYPE: CredentialKeyType = CredentialKeyType::Mock;
}

/// A type that implements [`KeyFactory`] and can be used in tests. It has the option
/// of returning `MockRemoteKeyFactoryError::Generating` when generating multiple
/// keys and `MockRemoteKeyFactoryError::Signing` when signing multiple, influenced
/// by boolean fields on the type.
#[derive(Debug)]
pub struct MockRemoteKeyFactory {
    signing_keys: Mutex<HashMap<String, SigningKey>>,
    pub has_generating_error: bool,
    pub has_multi_key_signing_error: bool,
}

impl MockRemoteKeyFactory {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            signing_keys: Mutex::new(signing_keys),
            has_generating_error: false,
            has_multi_key_signing_error: false,
        }
    }
}

impl Default for MockRemoteKeyFactory {
    fn default() -> Self {
        let keys = HashMap::from([
            #[cfg(feature = "examples")]
            {
                use super::examples::Examples;
                use super::examples::EXAMPLE_KEY_IDENTIFIER;

                (EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key())
            },
        ]);

        Self::new_signing_keys(keys)
    }
}

impl KeyFactory for MockRemoteKeyFactory {
    type Key = MockRemoteEcdsaKey;
    type Error = MockRemoteKeyFactoryError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        if self.has_generating_error {
            return Err(MockRemoteKeyFactoryError::Generating);
        }

        let identifiers_and_signing_keys =
            iter::repeat_with(|| (utils::random_string(32), SigningKey::random(&mut OsRng)))
                .take(count as usize)
                .collect::<Vec<_>>();

        self.signing_keys.lock().extend(
            identifiers_and_signing_keys
                .iter()
                .map(|(identifer, signing_key)| (identifer.clone(), signing_key.clone())),
        );

        let keys = identifiers_and_signing_keys
            .into_iter()
            .map(|(identifer, signing_key)| MockRemoteEcdsaKey::new(identifer, signing_key))
            .collect();

        Ok(keys)
    }

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        let identifier = identifier.into();
        let signing_key = self
            .signing_keys
            .lock()
            .get(&identifier)
            .expect("called generate_existing() with unknown identifier")
            .clone();

        // If the provided public key does not match the key fetched
        // using the identifier, this is programmer error.
        assert_eq!(
            signing_key.verifying_key(),
            &public_key,
            "called generate_existing() with incorrect public_key"
        );

        MockRemoteEcdsaKey::new(identifier, signing_key)
    }

    async fn sign_with_new_keys(
        &self,
        msg: Vec<u8>,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;

        if self.has_multi_key_signing_error {
            return Err(MockRemoteKeyFactoryError::Signing);
        }

        let signatures_by_identifier = future::try_join_all(keys.into_iter().map(|key| async {
            let signature = key
                .try_sign(msg.as_slice())
                .await
                .map_err(MockRemoteKeyFactoryError::Ecdsa)?;

            Ok::<_, MockRemoteKeyFactoryError>((key, signature))
        }))
        .await?
        .into_iter()
        .collect();

        Ok(signatures_by_identifier)
    }

    async fn sign_multiple_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error> {
        if self.has_multi_key_signing_error {
            return Err(MockRemoteKeyFactoryError::Signing);
        }

        let result = future::try_join_all(
            messages_and_keys
                .into_iter()
                .map(|(msg, keys)| async move {
                    let signatures = future::try_join_all(keys.into_iter().map(|key| async {
                        let signature = key.try_sign(&msg).await.map_err(MockRemoteKeyFactoryError::Ecdsa)?;

                        Ok::<_, MockRemoteKeyFactoryError>(signature)
                    }))
                    .await?
                    .into_iter()
                    .collect::<Vec<_>>();

                    Ok::<_, MockRemoteKeyFactoryError>(signatures)
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        Ok(result)
    }

    async fn poa(
        &self,
        keys: VecAtLeastTwo<&Self::Key>,
        aud: String,
        nonce: Option<String>,
    ) -> Result<Poa, Self::Error> {
        let poa = Poa::new(keys, JwtPopClaims::new(nonce, NL_WALLET_CLIENT_ID.to_string(), aud)).await?;

        Ok(poa)
    }
}
