use std::iter;

use futures::{executor, future};
use p256::ecdsa::{Signature, VerifyingKey};

use wallet_common::{
    keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier, EcdsaKey, SecureEcdsaKey, WithIdentifier},
    utils,
};

use crate::utils::keys::{KeyFactory, MdocEcdsaKey, MdocKeyType};

/// The [`FactorySoftwareEcdsaKey`] type wraps [`SoftwareEcdsaKey`] and has
/// the possibility of returning [`SoftwareKeyFactoryError::Signing`] when signing.
pub struct FactorySoftwareEcdsaKey {
    key: SoftwareEcdsaKey,
    has_signing_error: bool,
}

impl MdocEcdsaKey for FactorySoftwareEcdsaKey {
    const KEY_TYPE: MdocKeyType = MdocKeyType::Software;
}
impl SecureEcdsaKey for FactorySoftwareEcdsaKey {}
impl EcdsaKey for FactorySoftwareEcdsaKey {
    type Error = SoftwareKeyFactoryError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let verifying_key = self.key.verifying_key().await.unwrap();

        Ok(verifying_key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        if self.has_signing_error {
            return Err(SoftwareKeyFactoryError::Signing);
        }

        let signature = self.key.try_sign(msg).await.unwrap();

        Ok(signature)
    }
}
impl WithIdentifier for FactorySoftwareEcdsaKey {
    fn identifier(&self) -> &str {
        self.key.identifier()
    }
}

/// The [`SoftwareKeyFactory`] type implements [`KeyFactory`] and has the option
/// of returning [`SoftwareKeyFactoryError::Generating`] when generating keys, as well as generating
/// [`FactorySoftwareEcdsaKey`] that return [`SoftwareKeyFactoryError::Signing`] when signing.
#[derive(Debug, Default)]
pub struct SoftwareKeyFactory {
    pub has_generating_error: bool,
    pub has_key_signing_error: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("SoftwareKeyFactoryError")]
pub enum SoftwareKeyFactoryError {
    #[error("error generating keys")]
    Generating,
    #[error("signing error")]
    Signing,
}

impl SoftwareKeyFactory {
    fn new_key(&self, identifier: &str) -> FactorySoftwareEcdsaKey {
        FactorySoftwareEcdsaKey {
            key: SoftwareEcdsaKey::new(identifier),
            has_signing_error: self.has_key_signing_error,
        }
    }
}

impl KeyFactory for SoftwareKeyFactory {
    type Key = FactorySoftwareEcdsaKey;
    type Error = SoftwareKeyFactoryError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        if self.has_generating_error {
            return Err(SoftwareKeyFactoryError::Generating);
        }

        let keys = iter::repeat_with(|| self.new_key(&utils::random_string(32)))
            .take(count as usize)
            .collect();

        Ok(keys)
    }

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        let key = self.new_key(&identifier.into());

        // If the provided public key does not match the key fetched
        // using the identifier, this is programmer error.
        assert_eq!(executor::block_on(key.verifying_key()).unwrap(), public_key);

        key
    }

    async fn sign_with_new_keys(
        &self,
        msg: Vec<u8>,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;

        let signatures_by_identifier = future::try_join_all(keys.into_iter().map(|key| async {
            let signature = SoftwareEcdsaKey::new(key.identifier())
                .try_sign(msg.as_slice())
                .await
                .map_err(|_| SoftwareKeyFactoryError::Signing)?;

            Ok((key, signature))
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
        let result = future::try_join_all(
            messages_and_keys
                .into_iter()
                .map(|(msg, keys)| async move {
                    let signatures = future::try_join_all(keys.into_iter().map(|key| async {
                        let signature = key.try_sign(&msg).await?;
                        Ok(signature)
                    }))
                    .await?
                    .into_iter()
                    .collect::<Vec<_>>();

                    Ok(signatures)
                })
                .collect::<Vec<_>>(),
        )
        .await?
        .into_iter()
        .collect::<Vec<_>>();

        Ok(result)
    }
}
