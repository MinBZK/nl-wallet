use std::error::Error;

use p256::ecdsa::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::keys::{SecureEcdsaKey, WithIdentifier};

/// Contract for ECDSA private keys suitable for credentials.
/// Should be sufficiently secured e.g. through a HSM, or Android's TEE/StrongBox or Apple's SE.
pub trait CredentialEcdsaKey: SecureEcdsaKey + WithIdentifier {
    const KEY_TYPE: CredentialKeyType;

    // from WithIdentifier: identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CredentialKeyType {
    #[cfg(any(test, feature = "software_keys"))]
    Software,
    Remote,
}

pub trait KeyFactory {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;

    async fn generate_new(&self) -> Result<Self::Key, Self::Error> {
        self.generate_new_multiple(1).await.map(|mut keys| keys.pop().unwrap())
    }

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error>;
    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    async fn sign_with_new_keys(
        &self,
        msg: Vec<u8>,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error>;

    async fn sign_multiple_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error>;
}

#[cfg(any(test, feature = "software_keys"))]
mod software {
    use crate::keys::software::SoftwareEcdsaKey;

    use super::{CredentialKeyType, CredentialEcdsaKey};

    impl CredentialEcdsaKey for SoftwareEcdsaKey {
        const KEY_TYPE: CredentialKeyType = CredentialKeyType::Software;
    }
}
