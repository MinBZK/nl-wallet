use std::error::Error;
use std::num::NonZeroU8;

use derive_more::Constructor;
use jwt::nonce::Nonce;
use utils::vec_at_least::VecNonEmpty;

use crate::payload::key_attestation::KeyAttestation;

#[derive(Debug, Constructor)]
pub struct IssuanceKeyResult {
    pub identifiers: VecNonEmpty<String>,
    pub attestation: KeyAttestation,
}

pub trait IssuanceWscd {
    type Error: Error + Send + Sync + 'static;

    /// Construct new keys along with Key Attestations for use during issuance. This allows for the generation of
    /// multiple sets of keys. Each set is meant to be used for a distinct credential within an issuance session,
    /// while the keys within the set are meant for the copies of a single credential.
    ///
    /// The sets of keys require as input the number of keys requested and optionally a nonce to be included in the
    /// generated Key Attestation claims. The output is a list of [`IssuanceKeyResult`] values, which each contain a
    /// [`KeyAttestation`] and a list of key identifiers for the generated keys, whose count is equal to the
    /// number of requested keys.
    async fn perform_issuance(
        &self,
        key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
    ) -> Result<VecNonEmpty<IssuanceKeyResult>, Self::Error>;
}
