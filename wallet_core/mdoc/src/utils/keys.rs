use std::error::Error;

use async_trait::async_trait;
use p256::ecdsa::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

use wallet_common::keys::{SecureEcdsaKey, WithIdentifier};

/// Contract for ECDSA private keys suitable for mdoc attestations.
/// Should be sufficiently secured e.g. through a HSM, or Android's TEE/StrongBox or Apple's SE.
pub trait MdocEcdsaKey: SecureEcdsaKey + WithIdentifier {
    const KEY_TYPE: MdocKeyType;

    // from WithIdentifier: identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MdocKeyType {
    Software,
    Remote,
}

#[async_trait]
pub trait KeyFactory<'a> {
    type Key: MdocEcdsaKey + 'a;
    type Error: Error + Send + Sync + 'static;

    async fn generate_new(&'a self) -> Result<Self::Key, Self::Error> {
        self.generate_new_multiple(1).await.map(|mut keys| keys.remove(1))
    }

    async fn generate_new_multiple(&'a self, count: u64) -> Result<Vec<Self::Key>, Self::Error>;
    fn generate_existing<I: Into<String> + Send>(&'a self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    async fn sign_with_new_keys<T: Into<Vec<u8>> + Send>(
        &'a self,
        msg: T,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error>;
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use crate::utils::keys::MdocKeyType;
    use wallet_common::keys::software::SoftwareEcdsaKey;

    use super::MdocEcdsaKey;

    impl MdocEcdsaKey for SoftwareEcdsaKey {
        const KEY_TYPE: MdocKeyType = MdocKeyType::Software;
    }
}
