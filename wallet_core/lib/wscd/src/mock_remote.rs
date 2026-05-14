use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Duration;

use attestation_types::status_claim::StatusClaim;
use chrono::Utc;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd as DisclosureMockRemoteWscd;
use crypto::mock_remote::MockRemoteWscdError;
use crypto::p256_der::verifying_key_sha256;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use derive_more::Debug;
use futures::FutureExt;
use itertools::Itertools;
use jwt::SignedJwt;
use jwt::nonce::Nonce;
use jwt::pop::JwtPopClaims;
use jwt::wia::ClientStatus;
use jwt::wia::WiaClaims;
use jwt::wia::WiaDisclosure;
use jwt::wia::WiaWalletInfo;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::Poa;
use crate::wscd::IssuanceResult;
use crate::wscd::IssuanceWscd;

pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";

/// A type that implements [`Wscd`] and can be used in tests. It has the option
/// of returning `MockRemoteWscdError::Generating` when generating multiple
/// keys and `MockRemoteWscdError::Signing` when signing multiple, influenced
/// by boolean fields on the type.
#[derive(Debug)]
pub struct MockRemoteWscd {
    pub disclosure: DisclosureMockRemoteWscd,
    wia_signing_key: Option<SigningKey>,
}

impl MockRemoteWscd {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            disclosure: DisclosureMockRemoteWscd::new_signing_keys(signing_keys),
            wia_signing_key: None,
        }
    }

    pub fn new_with_wia_signing_key(wia_signing_key: SigningKey) -> Self {
        Self {
            wia_signing_key: Some(wia_signing_key),
            ..Default::default()
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

impl AsRef<DisclosureMockRemoteWscd> for MockRemoteWscd {
    fn as_ref(&self) -> &DisclosureMockRemoteWscd {
        &self.disclosure
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
                    JwtPopClaims::new(poa_input.nonce, MOCK_WALLET_CLIENT_ID.to_string(), poa_input.aud),
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
        count: NonZeroUsize,
        aud: String,
        nonce: Option<Nonce>,
    ) -> Result<IssuanceResult, Self::Error> {
        let claims = JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud);

        let mut keys = self.disclosure.signing_keys.lock();
        let attestation_keys: VecNonEmpty<_> = (0..count.get())
            .map(|_| {
                let key = SigningKey::random(&mut OsRng);
                let identifier = verifying_key_sha256(key.verifying_key());
                keys.insert(identifier.clone(), key.clone());
                MockRemoteEcdsaKey::new(identifier, key)
            })
            .collect_vec()
            .try_into()
            .unwrap(); // `count` is non-zero, so the unwrap is safe.
        drop(keys);

        let pops = attestation_keys
            .nonempty_iter()
            .map(|attestation_key| {
                SignedJwt::sign_with_jwk(&claims, attestation_key)
                    .now_or_never()
                    .unwrap()
                    .unwrap()
                    .into()
            })
            .collect();

        Ok(IssuanceResult {
            key_identifiers: attestation_keys
                .into_nonempty_iter()
                .map(|key| key.identifier)
                .collect(),
            pops,
        })
    }

    async fn issue_wia(&self, aud: String, nonce: Option<Nonce>) -> Result<WiaDisclosure, Self::Error> {
        let wia_key = SigningKey::random(&mut OsRng);
        let wia_key = MockRemoteEcdsaKey::new(verifying_key_sha256(wia_key.verifying_key()), wia_key);

        // If no WIA signing key is configured, just use the WIA's private key to sign it
        let wia_signing_key = self.wia_signing_key.as_ref().unwrap_or(&wia_key.key);

        let exp = Utc::now() + Duration::from_secs(600);
        let wallet_info = WiaWalletInfo {
            wallet_name: "Mock Wallet".to_string(),
            wallet_link: None,
            wallet_version: "1.0.0".to_string(),
            wallet_solution_certification_information: "info".to_string(),
        };

        let wia = SignedJwt::sign(
            &WiaClaims::new(
                wia_key.verifying_key(),
                MOCK_WALLET_CLIENT_ID.to_string(),
                exp,
                wallet_info,
                ClientStatus {
                    status: StatusClaim::new_mock(),
                    exp,
                },
            )
            .unwrap(),
            wia_signing_key,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
        .into();

        let wia_disclosure = SignedJwt::sign(
            &JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud),
            &wia_key,
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        Ok(WiaDisclosure::new(wia, wia_disclosure.into()))
    }
}
