use std::error::Error;
use std::num::NonZeroUsize;

use crypto::wscd::WscdPoa;
use derive_more::Constructor;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::nonce::Nonce;
use jwt::pop::JwtPopClaims;
use jwt::wua::WuaDisclosure;
use utils::vec_at_least::VecNonEmpty;

pub trait IssuanceWscd {
    type Error: Error + Send + Sync + 'static;
    type Poa: WscdPoa;

    /// Construct new keys along with PoPs and PoA, and optionally a WUA, for use during issuance.
    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<Nonce>,
        include_wua: bool,
    ) -> Result<IssuanceResult<Self::Poa>, Self::Error>;
}

#[derive(Debug, Constructor)]
pub struct IssuanceResult<P> {
    pub key_identifiers: VecNonEmpty<String>,
    pub pops: VecNonEmpty<UnverifiedJwt<JwtPopClaims, HeaderWithJwk>>,
    pub poa: Option<P>,
    pub wua: Option<WuaDisclosure>,
}

#[derive(Debug, Constructor)]
pub struct JwtPoaInput {
    pub nonce: Option<Nonce>,
    pub aud: String,
}
