use std::error::Error;

use derive_more::Constructor;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::CredentialEcdsaKey;

pub trait DisclosureKeyFactory {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;
    type Poa: KeyFactoryPoa;

    /// Instantiate a new reference to a key in this WSCD.
    ///
    /// NOTE: this does not generate the key in the WSCD if it does not already exist.
    /// For generating keys, use `KeyFactory::perform_issuance()`.
    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    /// Sign the given inputs with the given keys, also returning a PoA when more than one key is used.
    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        poa_input: <Self::Poa as KeyFactoryPoa>::Input,
    ) -> Result<DisclosureResult<Self::Poa>, Self::Error>;
}

pub trait KeyFactoryPoa {
    type Input;
}

#[derive(Debug, Constructor)]
pub struct DisclosureResult<P> {
    pub signatures: Vec<Vec<Signature>>,
    pub poa: Option<P>,
}
