use std::error::Error;

use async_trait::async_trait;
use p256::ecdsa::VerifyingKey;
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

    async fn generate_new<I: AsRef<str> + Sync>(&'a self, identifiers: &[I]) -> Result<Vec<Self::Key>, Self::Error>;
    fn generate_existing<I: AsRef<str> + Sync>(&'a self, identifier: &I, public_key: VerifyingKey) -> Self::Key;
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
