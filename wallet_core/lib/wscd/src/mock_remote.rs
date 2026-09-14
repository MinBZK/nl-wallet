use std::collections::HashMap;
use std::num::NonZeroU8;

use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd as DisclosureMockRemoteWscd;
use crypto::mock_remote::MockRemoteWscdError;
use crypto::p256_der::verifying_key_sha256;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use derive_more::AsRef;
use futures::FutureExt;
use itertools::Itertools;
use jwt::SignedJwt;
use jwt::nonce::Nonce;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::Generate;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::issuance::IssuanceKeyresult;
use crate::issuance::IssuanceWscd;
use crate::mock::MOCK_WALLET_CLIENT_ID;
use crate::payload::jwt_proof::JwtProofClaims;
use crate::payload::poa::Poa;

/// A type that implements both the [`DisclosureWscd`] and [`IssuanceWscd`] traits and can be used in tests. It has the
/// option of returning `MockRemoteWscdError::Generating` when generating multiple keys and
/// `MockRemoteWscdError::Signing` when signing multiple, influenced by boolean fields on the type.
#[derive(Debug, AsRef)]
pub struct MockRemoteWscd {
    pub disclosure: DisclosureMockRemoteWscd,
}

impl MockRemoteWscd {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            disclosure: DisclosureMockRemoteWscd::new_signing_keys(signing_keys),
        }
    }

    #[cfg(feature = "examples")]
    pub fn new_example() -> Self {
        use crypto::examples::EXAMPLE_KEY_IDENTIFIER;
        use crypto::examples::Examples;

        let keys = HashMap::from([(EXAMPLE_KEY_IDENTIFIER.to_string(), Examples::static_device_key())]);
        Self::new_signing_keys(keys)
    }

    pub fn create_random_key(&self) -> MockRemoteEcdsaKey {
        self.disclosure.create_random_key()
    }
}

impl Default for MockRemoteWscd {
    fn default() -> Self {
        Self::new_signing_keys(HashMap::new())
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
        aud: String,
        key_counts_and_nonces: VecNonEmpty<(NonZeroU8, Option<Nonce>)>,
    ) -> Result<VecNonEmpty<VecNonEmpty<IssuanceKeyresult>>, Self::Error> {
        let time = MockTimeGenerator::default();
        let mut signing_keys = self.disclosure.signing_keys.lock();

        let nonce_count = key_counts_and_nonces.len();
        let key_results = key_counts_and_nonces
            .into_nonempty_iter()
            .zip(utils::vec_at_least::repeat_n(aud, nonce_count))
            .map(|((key_count, nonce), aud)| {
                let claims = JwtProofClaims::new(MOCK_WALLET_CLIENT_ID.to_string(), aud, nonce, &time);

                utils::vec_at_least::repeat_n((), key_count.into())
                    .map(|_| {
                        let key = SigningKey::generate();
                        let identifier = verifying_key_sha256(key.verifying_key());

                        let pop = SignedJwt::sign_with_jwk(&claims, &key)
                            .now_or_never()
                            .unwrap()
                            .unwrap()
                            .into();

                        signing_keys.insert(identifier.clone(), key);

                        IssuanceKeyresult::new(identifier, pop)
                    })
                    .collect()
            })
            .collect();

        Ok(key_results)
    }
}
