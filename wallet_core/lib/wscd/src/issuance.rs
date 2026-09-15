use std::error::Error;
use std::num::NonZeroU8;

use derive_more::Constructor;
use jwt::nonce::Nonce;
use utils::vec_at_least::VecNonEmpty;

use crate::payload::jwt_proof::JwtProof;

#[derive(Debug, Constructor)]
pub struct IssuanceKeyResult {
    pub key_identifier: String,
    pub pop: JwtProof,
}

pub trait IssuanceWscd {
    type Error: Error + Send + Sync + 'static;

    /// Construct new keys along with Proofs of Possession for use during issuance. This allows for the generation of
    /// multiple sets of keys. Each set is meant to be used for a distinct credential within an issuance session,
    /// while the keys within the set are meant for the copies of a single credential.
    ///
    /// The sets of keys require as input the number of keys requested and optionally take a nonce to be included in the
    /// generated PoPs. The output is a two-dimensional vector, in which the key identifiers and PoPs are grouped per
    /// set that was present in the inciting request.
    async fn perform_issuance(
        &self,
        aud: String,
        key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
    ) -> Result<VecNonEmpty<VecNonEmpty<IssuanceKeyResult>>, Self::Error>;
}
