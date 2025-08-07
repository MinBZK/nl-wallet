use std::error::Error;
use std::num::NonZeroUsize;

use derive_more::Constructor;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crypto::CredentialEcdsaKey;
use jwt::Jwt;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteDisclosure;
use utils::vec_at_least::VecNonEmpty;

pub trait KeyFactory {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;
    type PoaInput;
    type Poa;

    /// Instantiate a new reference to a key in this WSCD.
    ///
    /// NOTE: this does not generate the key in the WSCD if it does not already exist.
    /// For generating keys, use [`KeyFactory::perform_issuance()`].
    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    /// Sign the given inputs with the given keys, also returning a PoA when more than one key is used.
    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        poa_input: Self::PoaInput,
    ) -> Result<DisclosureResult<Self::Poa>, Self::Error>;

    /// Construct new keys along with PoPs and PoA, and optionally a WUA, for use during issuance.
    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<String>,
        include_wua: bool,
    ) -> Result<IssuanceResult<Self::Poa>, Self::Error>;
}

#[derive(Debug, Constructor)]
pub struct DisclosureResult<P> {
    pub signatures: Vec<Vec<Signature>>,
    pub poa: Option<P>,
}

#[derive(Debug, Constructor)]
pub struct IssuanceResult<P> {
    pub key_identifiers: VecNonEmpty<String>,
    pub pops: VecNonEmpty<Jwt<JwtPopClaims>>,
    pub poa: Option<P>,
    pub wua: Option<WteDisclosure>,
}

#[derive(Debug, Constructor)]
pub struct JwtPoaInput {
    pub nonce: Option<String>,
    pub aud: String,
}
