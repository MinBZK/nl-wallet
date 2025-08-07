use std::error::Error;
use std::num::NonZeroUsize;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crypto::CredentialEcdsaKey;
use jwt::Jwt;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteDisclosure;
use utils::vec_at_least::VecNonEmpty;

use crate::Poa;

pub trait KeyFactory {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key;

    async fn sign_multiple_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error>;

    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<String>,
        include_wua: bool,
    ) -> Result<IssuanceResult, Self::Error>;
}

#[derive(Debug)]
pub struct IssuanceResult {
    pub key_identifiers: VecNonEmpty<String>,
    pub pops: VecNonEmpty<Jwt<JwtPopClaims>>,
    pub wua: Option<WteDisclosure>,
    pub poa: Option<Poa>,
}
