use std::error::Error;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::CredentialEcdsaKey;

pub trait DisclosureWscd {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;

    /// Instantiate a new reference to a key in this WSCD.
    ///
    /// NOTE: this does not generate the key in the WSCD if it does not already exist.
    /// For generating keys, use `Wscd::perform_issuance()`.
    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    /// Sign the given inputs with the given keys.
    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error>;
}
