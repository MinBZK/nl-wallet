use std::{collections::HashMap, iter};

use cfg_if::cfg_if;
use futures::future;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use parking_lot::Mutex;

use rand_core::OsRng;
use wallet_common::{
    keys::{software::SoftwareEcdsaKey, EcdsaKey},
    utils,
};

use crate::utils::keys::KeyFactory;

/// The [`SoftwareKeyFactory`] type implements [`KeyFactory`] and has the option
/// of returning [`SoftwareKeyFactoryError::Generating`] when generating multiple
/// keys and [`SoftwareKeyFactoryError::Signing`] when signing multiple.
#[derive(Debug)]
pub struct SoftwareKeyFactory {
    signing_keys: Mutex<HashMap<String, SigningKey>>,
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
    Ecdsa(#[source] <SoftwareEcdsaKey as EcdsaKey>::Error),
}

impl KeyFactory for SoftwareKeyFactory {
    type Key = SoftwareEcdsaKey;
    type Error = SoftwareKeyFactoryError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        if self.has_generating_error {
            return Err(SoftwareKeyFactoryError::Generating);
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
            .map(|(identifer, signing_key)| SoftwareEcdsaKey::new(identifer, signing_key))
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

        SoftwareEcdsaKey::new(identifier, signing_key)
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
                .map_err(SoftwareKeyFactoryError::Ecdsa)?;

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
        if self.has_multi_key_signing_error {
            return Err(SoftwareKeyFactoryError::Signing);
        }

        let result = future::try_join_all(
            messages_and_keys
                .into_iter()
                .map(|(msg, keys)| async move {
                    let signatures = future::try_join_all(keys.into_iter().map(|key| async {
                        let signature = key.try_sign(&msg).await.map_err(SoftwareKeyFactoryError::Ecdsa)?;

                        Ok(signature)
                    }))
                    .await?
                    .into_iter()
                    .collect::<Vec<_>>();

                    Ok(signatures)
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        Ok(result)
    }
}

impl Default for SoftwareKeyFactory {
    fn default() -> Self {
        cfg_if! {
            // Pre-populate the static example key, if the feature is enabled.
            if #[cfg(any(test, feature = "examples"))] {
                use crate::examples::{Examples, EXAMPLE_KEY_IDENTIFIER};

                let keys = HashMap::from([(EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key())]);
            } else {
                let keys = HashMap::default();
            }
        }

        SoftwareKeyFactory {
            signing_keys: keys.into(),
            has_generating_error: false,
            has_multi_key_signing_error: false,
        }
    }
}
