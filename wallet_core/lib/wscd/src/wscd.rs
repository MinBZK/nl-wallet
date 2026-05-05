use std::error::Error;
use std::num::NonZeroUsize;

use derive_more::Constructor;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::nonce::Nonce;
use jwt::pop::JwtPopClaims;
use jwt::wia::WiaDisclosure;
use utils::vec_at_least::VecNonEmpty;

pub trait IssuanceWscd {
    type Error: Error + Send + Sync + 'static;

    /// Construct new keys along with PoPs, and optionally a WIA, for use during issuance.
    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<Nonce>,
        include_wia: bool,
    ) -> Result<IssuanceResult, Self::Error>;
}

#[derive(Debug, Constructor)]
pub struct IssuanceResult {
    pub key_identifiers: VecNonEmpty<String>,
    pub pops: VecNonEmpty<UnverifiedJwt<JwtPopClaims, HeaderWithJwk>>,
    pub wia: Option<WiaDisclosure>,
}
