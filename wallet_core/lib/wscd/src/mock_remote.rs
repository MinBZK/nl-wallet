use std::convert::Infallible;
use std::num::NonZeroU8;
use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use crypto::PublicKey;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use futures::FutureExt;
use jwt::nonce::Nonce;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::Generate;
use url::Url;
use utils::generator::Generator;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::issuance::IssuanceKeyResult;
use crate::issuance::IssuanceWscd;
use crate::payload::key_attestation::KeyAttestation;
use crate::payload::status_claim::TimeBoundedStatusClaim;

pub static MOCK_CERTIFICATION_URL: LazyLock<Url> =
    LazyLock::new(|| "https://wallet-provider.example.com/certification".parse().unwrap());

/// A type that implements the [`IssuanceWscd`] trait and can be used in tests. The private keys it generates are
/// discarded, so they cannot be used for signing afterwards.
#[derive(Debug)]
pub struct IssuanceMockRemoteWscd {
    attestation_key_pair: KeyPair,
}

impl IssuanceMockRemoteWscd {
    pub fn new(wptl_ca: &Ca) -> Self {
        let attestation_key_pair = wptl_ca.generate_key_attestation_mock().unwrap();

        Self { attestation_key_pair }
    }

    pub fn sign_key_attestation<I, G>(&self, verifying_keys: I, nonce: Option<Nonce>, time: &G) -> KeyAttestation
    where
        I: IntoNonEmptyIterator<Item = VerifyingKey>,
        G: Generator<DateTime<Utc>>,
    {
        KeyAttestation::sign(
            verifying_keys
                .into_nonempty_iter()
                .map(PublicKey::from)
                .collect::<VecNonEmpty<_>>()
                .nonempty_iter(),
            MOCK_CERTIFICATION_URL.clone(),
            nonce,
            TimeBoundedStatusClaim::new_mock(),
            &self.attestation_key_pair,
            time,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
    }
}

impl IssuanceWscd for IssuanceMockRemoteWscd {
    type Error = Infallible;

    async fn perform_issuance(
        &self,
        key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
    ) -> Result<VecNonEmpty<IssuanceKeyResult>, Self::Error> {
        let time = MockTimeGenerator::default();

        let key_results = key_counts_and_nonces
            .into_nonempty_iter()
            .map(|(key_count, nonce)| {
                let (verifying_keys, identifiers): (VecNonEmpty<_>, _) =
                    utils::vec_at_least::repeat_n((), key_count.into())
                        .map(|_| {
                            let key = SigningKey::generate();

                            (*key.verifying_key(), random_string(16))
                        })
                        .unzip();

                let key_attestation = self.sign_key_attestation(verifying_keys, nonce, &time);

                IssuanceKeyResult::new(identifiers, key_attestation)
            })
            .collect();

        Ok(key_results)
    }
}
