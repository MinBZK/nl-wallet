use std::error::Error;
use std::num::NonZeroU8;

use derive_more::Constructor;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::nonce::Nonce;
use jwt::pop::JwtPopClaims;
use jwt::wia::WiaDisclosure;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, Constructor)]
pub struct IssuanceKeyresult {
    pub key_identifier: String,
    pub pop: UnverifiedJwt<JwtPopClaims, HeaderWithJwk>,
}

pub trait IssuanceWscd {
    type Error: Error + Send + Sync + 'static;

    /// Construct new keys along with Proofs of Possession for use during issuance. This allows for the generation of
    /// multiple sets of keys. Each set is meant to be used for a distinct credential within an issuance session,
    /// while the keys within the set are meant for the copies of a single credential.
    ///
    /// The sets of keys require as input the number of keys requested and optionally take a nonce to be included in the
    /// generated PoPs. The output is a two-dimensional vector, in which the key identifiers and PoPs are grouped per
    /// set.
    async fn perform_issuance(
        &self,
        aud: String,
        key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
    ) -> Result<VecNonEmpty<VecNonEmpty<IssuanceKeyresult>>, Self::Error>;
}

pub trait WiaClient {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wia(&self, aud: String, nonce: Option<Nonce>) -> Result<WiaDisclosure, Self::Error>;
}
