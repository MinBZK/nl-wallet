use std::collections::HashMap;
use std::num::NonZeroU8;
use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use crypto::PublicKey;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd as DisclosureMockRemoteWscd;
use crypto::mock_remote::MockRemoteWscdError;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use derive_more::AsRef;
use futures::FutureExt;
use itertools::Itertools;
use jwt::nonce::Nonce;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use url::Url;
use utils::generator::Generator;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::issuance::IssuanceKeyResult;
use crate::issuance::IssuanceWscd;
use crate::mock::MOCK_WALLET_CLIENT_ID;
use crate::payload::jwt_proof::JwtProofClaims;
use crate::payload::key_attestation::KeyAttestation;
use crate::payload::poa::Poa;

pub static MOCK_CERTIFICATION_URL: LazyLock<Url> =
    LazyLock::new(|| "https://wallet-provider.example.com/certification".parse().unwrap());

/// A type that implements both the [`DisclosureWscd`] and [`IssuanceWscd`] traits and can be used in tests. It has the
/// option of returning `MockRemoteWscdError::Generating` when generating multiple keys and
/// `MockRemoteWscdError::Signing` when signing multiple, influenced by boolean fields on the type.
#[derive(Debug, AsRef)]
pub struct MockRemoteWscd {
    #[as_ref]
    pub disclosure: DisclosureMockRemoteWscd,
    attestation_key_pair: KeyPair,
}

impl MockRemoteWscd {
    pub fn new() -> Self {
        Self::new_with_keys(Vec::new())
    }

    pub fn new_with_keys(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let wptl_ca = Ca::generate_wptl_mock_ca().unwrap();
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_with_with_wptl_ca_and_keys(&wptl_ca, signing_keys)
    }

    pub fn new_with_wptl_ca(wptl_ca: &Ca) -> Self {
        Self::new_with_with_wptl_ca_and_keys(wptl_ca, HashMap::new())
    }

    fn new_with_with_wptl_ca_and_keys(wptl_ca: &Ca, signing_keys: HashMap<String, SigningKey>) -> Self {
        let attestation_key_pair = wptl_ca.generate_key_attestation_mock().unwrap();

        Self {
            disclosure: DisclosureMockRemoteWscd::new_signing_keys(signing_keys),
            attestation_key_pair,
        }
    }

    #[cfg(feature = "examples")]
    pub fn new_example() -> Self {
        use crypto::examples::EXAMPLE_KEY_IDENTIFIER;
        use crypto::examples::Examples;

        let wptl_ca = Ca::generate_wptl_mock_ca().unwrap();
        let keys = HashMap::from([(EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key())]);

        Self::new_with_with_wptl_ca_and_keys(&wptl_ca, keys)
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
            &self.attestation_key_pair,
            time,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
    }
}

impl Default for MockRemoteWscd {
    fn default() -> Self {
        Self::new()
    }
}

impl DisclosureWscd for MockRemoteWscd {
    type Key = MockRemoteEcdsaKey;
    type Error = MockRemoteWscdError;
    type Poa = Poa;

    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        self.disclosure.new_key(identifier, public_key)
    }

    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        poa_input: <Self::Poa as WscdPoa>::Input,
    ) -> Result<DisclosureResult<Self::Poa>, Self::Error> {
        let keys = messages_and_keys
            .iter()
            .flat_map(|(_, keys)| keys.clone())
            .collect_vec();

        let poa = if keys.len() < 2 {
            None
        } else {
            Some(
                Poa::new(
                    keys.try_into().unwrap(),
                    JwtProofClaims::new(
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        poa_input.aud,
                        poa_input.nonce,
                        &MockTimeGenerator::default(),
                    ),
                )
                .await
                .map_err(|_| MockRemoteWscdError::Poa)?,
            )
        };

        let DisclosureResult { signatures, .. } = self.disclosure.sign(messages_and_keys, ()).await?;

        Ok(DisclosureResult { signatures, poa })
    }
}

impl IssuanceWscd for MockRemoteWscd {
    type Error = MockRemoteWscdError;

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
                            let remote_key = self.disclosure.create_random_key();

                            (*remote_key.key.verifying_key(), remote_key.identifier)
                        })
                        .unzip();

                let key_attestation = self.sign_key_attestation(verifying_keys, nonce, &time);

                IssuanceKeyResult::new(identifiers, key_attestation)
            })
            .collect();

        Ok(key_results)
    }
}
