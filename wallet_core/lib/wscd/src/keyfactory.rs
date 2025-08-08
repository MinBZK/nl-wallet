use std::num::NonZeroUsize;

use crypto::wscd::DisclosureKeyFactory;
use derive_more::Constructor;

use jwt::Jwt;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteDisclosure;
use utils::vec_at_least::VecNonEmpty;

use crate::Poa;

pub trait KeyFactory: DisclosureKeyFactory<Poa = Poa> {
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
