use std::sync::Arc;

use crypto::keys::EcdsaKey;
use crypto::kid::VerifyingKeyWithKid;
use jwt::KeyWithKid;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Signer;
use p256::elliptic_curve::Generate;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::ConfigurationError;
use crate::config::data::default_wallet_config;
use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

pub struct ConfigSigningKey {
    pub key: p256::ecdsa::SigningKey,
    pub kid: &'static str,
}

impl ConfigSigningKey {
    pub fn new_with_kid(kid: &'static str) -> Self {
        let key = p256::ecdsa::SigningKey::generate();
        Self { key, kid }
    }

    pub fn to_config_key_with_kid(&self) -> VerifyingKeyWithKid {
        VerifyingKeyWithKid {
            kid: self.kid.try_into().unwrap(),
            public_key: (*self.key.verifying_key()).into(),
        }
    }
}

impl EcdsaKey for ConfigSigningKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.key.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Signer::try_sign(&self.key, msg)
    }
}

impl KeyWithKid for ConfigSigningKey {
    fn kid(&self) -> &str {
        self.kid
    }
}

pub struct LocalConfigurationRepository {
    config: Arc<WalletConfiguration>,
}

impl LocalConfigurationRepository {
    pub fn new(config: WalletConfiguration) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Default for LocalConfigurationRepository {
    fn default() -> Self {
        Self::new(default_wallet_config())
    }
}

impl Repository<Arc<WalletConfiguration>> for LocalConfigurationRepository {
    fn get(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config)
    }
}

impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for LocalConfigurationRepository
where
    B: Send + Sync,
{
    type Error = ConfigurationError;

    async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
        Ok(RepositoryUpdateState::Updated {
            from: self.get(),
            to: self.get(),
        })
    }
}

impl ObservableRepository<Arc<WalletConfiguration>> for LocalConfigurationRepository {
    fn register_callback_on_update(
        &self,
        _callback: RepositoryCallback<Arc<WalletConfiguration>>,
    ) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        None
    }

    fn clear_callback(&self) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        None
    }
}
