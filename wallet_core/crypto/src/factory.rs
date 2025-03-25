use std::error::Error;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::keys::CredentialEcdsaKey;

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
