use std::{collections::HashMap, iter};

use futures::{executor, future};
use p256::ecdsa::{Signature, VerifyingKey};

use parking_lot::Mutex;
use wallet_common::{
    keys::{software::SoftwareEcdsaKey, EcdsaKey, WithIdentifier},
    utils,
};

use crate::utils::keys::KeyFactory;

/// The [`SoftwareKeyFactory`] type implements [`KeyFactory`] and has the option
/// of returning [`SoftwareKeyFactoryError::Generating`] when generating keys, as well as generating
/// [`FactorySoftwareEcdsaKey`] that return [`SoftwareKeyFactoryError::Signing`] when signing.
#[derive(Debug)]
pub struct SoftwareKeyFactory {
    software_keys: Mutex<HashMap<String, SoftwareEcdsaKey>>,
    pub has_generating_error: bool,
    pub has_multi_key_signing_error: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("SoftwareKeyFactoryError")]
pub enum SoftwareKeyFactoryError {
    #[error("key generation error")]
    Generating,
    #[error("signing error")]
    Signing,
    #[error("ECDSA error: {0}")]
    ECDSA(#[source] <SoftwareEcdsaKey as EcdsaKey>::Error),
}

impl KeyFactory for SoftwareKeyFactory {
    type Key = SoftwareEcdsaKey;
    type Error = SoftwareKeyFactoryError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        if self.has_generating_error {
            return Err(SoftwareKeyFactoryError::Generating);
        }

        let keys = iter::repeat_with(|| SoftwareEcdsaKey::new_random(utils::random_string(32)))
            .take(count as usize)
            .collect::<Vec<_>>();

        self.software_keys
            .lock()
            .extend(keys.iter().map(|key| (key.identifier().to_string(), key.clone())));

        Ok(keys)
    }

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        let key = self
            .software_keys
            .lock()
            .get(&identifier.into())
            .expect("called generate_existing() with unknown identifier")
            .clone();

        // If the provided public key does not match the key fetched
        // using the identifier, this is programmer error.
        assert_eq!(
            executor::block_on(key.verifying_key()).unwrap(),
            public_key,
            "called generate_existing() with incorrect public_key"
        );

        key
    }

    async fn sign_with_new_keys(
        &self,
        msg: Vec<u8>,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;

        if self.has_multi_key_signing_error {
            return Err(SoftwareKeyFactoryError::Signing);
        }

        let signatures_by_identifier = future::try_join_all(keys.into_iter().map(|key| async {
            let signature = key
                .try_sign(msg.as_slice())
                .await
                .map_err(SoftwareKeyFactoryError::ECDSA)?;

            Ok((key, signature))
        }))
        .await?
        .into_iter()
        .collect();

        Ok(signatures_by_identifier)
    }

    async fn sign_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<Self::Key>)>,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        if self.has_multi_key_signing_error {
            return Err(SoftwareKeyFactoryError::Signing);
        }

        let result = future::try_join_all(
            messages_and_keys
                .into_iter()
                .map(|(msg, keys)| async move {
                    let signatures_by_identifier: Vec<(Self::Key, Signature)> =
                        future::try_join_all(keys.into_iter().map(|key| async {
                            let signature = key.try_sign(&msg).await.map_err(SoftwareKeyFactoryError::ECDSA)?;

                            Ok((key, signature))
                        }))
                        .await?
                        .into_iter()
                        .collect();

                    Ok(signatures_by_identifier)
                })
                .collect::<Vec<_>>(),
        )
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        Ok(result)
    }
}

impl Default for SoftwareKeyFactory {
    fn default() -> Self {
        // Pre-populate the static example key, if the feature is enabled.
        #[cfg(any(test, feature = "examples"))]
        let keys = {
            use crate::examples::{Examples, EXAMPLE_KEY_IDENTIFIER};

            HashMap::from([(
                EXAMPLE_KEY_IDENTIFIER.to_string(),
                SoftwareEcdsaKey::new(EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key()),
            )])
        };

        #[cfg(not(any(test, feature = "examples")))]
        let keys = HashMap::default();

        SoftwareKeyFactory {
            software_keys: keys.into(),
            has_generating_error: false,
            has_multi_key_signing_error: false,
        }
    }
}
