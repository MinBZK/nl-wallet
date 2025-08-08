use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use derive_more::Constructor;
use derive_more::Debug;
use futures::future;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Signer;
use parking_lot::Mutex;
use rand_core::OsRng;

use crate::CredentialEcdsaKey;
use crate::CredentialKeyType;
use crate::EcdsaKey;
use crate::SecureEcdsaKey;
use crate::WithIdentifier;
use crate::wscd::DisclosureKeyFactory;
use crate::wscd::DisclosureResult;
use crate::wscd::KeyFactoryPoa;

/// To be used in test in place of `RemoteEcdsaKey`, implementing the
/// [`EcdsaKey`], [`SecureEcdsaKey`] and [`WithIdentifier`] traits.
#[derive(Debug, Clone, Constructor)]
pub struct MockRemoteEcdsaKey {
    pub identifier: String,
    #[debug(skip)]
    pub key: SigningKey,
}

impl PartialEq for MockRemoteEcdsaKey {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for MockRemoteEcdsaKey {}

impl Hash for MockRemoteEcdsaKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl MockRemoteEcdsaKey {
    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, SigningKey::random(&mut OsRng))
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        self.key.verifying_key()
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

#[derive(Debug, thiserror::Error)]
pub enum MockRemoteKeyFactoryError {
    #[error("key generation error")]
    Generating,
    #[error("signing error")]
    Signing,
    #[error("poa error")]
    Poa,
    #[error("ECDSA error: {0}")]
    Ecdsa(#[source] <MockRemoteEcdsaKey as EcdsaKey>::Error),
}

/// A type that implements [`KeyFactory`] and can be used in tests. It has the option
/// of returning `MockRemoteKeyFactoryError::Generating` when generating multiple
/// keys and `MockRemoteKeyFactoryError::Signing` when signing multiple, influenced
/// by boolean fields on the type.
#[derive(Debug)]
pub struct MockRemoteKeyFactory {
    pub signing_keys: Mutex<HashMap<String, SigningKey>>,

    pub has_multi_key_signing_error: bool,
}

impl MockRemoteKeyFactory {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    pub fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            signing_keys: Mutex::new(signing_keys),
            has_multi_key_signing_error: false,
        }
    }

    #[cfg(feature = "examples")]
    pub fn new_example() -> Self {
        use crate::examples::EXAMPLE_KEY_IDENTIFIER;
        use crate::examples::Examples;

        let keys = HashMap::from([(EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key())]);
        Self::new_signing_keys(keys)
    }
}

impl Default for MockRemoteKeyFactory {
    fn default() -> Self {
        Self::new_signing_keys(HashMap::new())
    }
}

pub struct MockPoa;

impl KeyFactoryPoa for MockPoa {
    type Input = ();
}

impl DisclosureKeyFactory for MockRemoteKeyFactory {
    type Key = MockRemoteEcdsaKey;
    type Error = MockRemoteKeyFactoryError;
    type Poa = MockPoa;

    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
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

    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        _poa_input: <Self::Poa as KeyFactoryPoa>::Input,
    ) -> Result<DisclosureResult<Self::Poa>, Self::Error> {
        if self.has_multi_key_signing_error {
            return Err(MockRemoteKeyFactoryError::Signing);
        }

        let signatures = future::try_join_all(
            messages_and_keys
                .iter()
                .map(|(msg, keys)| async move {
                    let signatures = future::try_join_all(keys.iter().map(|key| async { key.try_sign(msg).await }))
                        .await
                        .map_err(MockRemoteKeyFactoryError::Ecdsa)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    Ok::<_, Self::Error>(signatures)
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        let keys_count = messages_and_keys.into_iter().flat_map(|(_, keys)| keys).count();
        let poa = if keys_count < 2 { None } else { Some(MockPoa) };

        Ok(DisclosureResult { signatures, poa })
    }
}
