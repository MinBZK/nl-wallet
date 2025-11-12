use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Duration;

use chrono::Utc;
use derive_more::Debug;
use futures::FutureExt;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;

use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd as DisclosureMockRemoteWscd;
use crypto::mock_remote::MockRemoteWscdError;
use crypto::p256_der::verifying_key_sha256;
use crypto::wscd::DisclosureResult;
use crypto::wscd::DisclosureWscd;
use crypto::wscd::WscdPoa;
use jwt::SignedJwt;
use jwt::pop::JwtPopClaims;
use jwt::wua::WuaClaims;
use jwt::wua::WuaDisclosure;

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
    wua_signing_key: Option<SigningKey>,
}

impl MockRemoteWscd {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            disclosure: DisclosureMockRemoteWscd::new_signing_keys(signing_keys),
            wua_signing_key: None,
        }
    }

    pub fn new_with_wua_signing_key(wua_signing_key: SigningKey) -> Self {
        Self {
            wua_signing_key: Some(wua_signing_key),
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
    type Poa = Poa;

    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<String>,
        include_wua: bool,
    ) -> Result<IssuanceResult<Poa>, Self::Error> {
        let claims = JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud);

        let mut keys = self.disclosure.signing_keys.lock();
        let attestation_keys = (0..count.get())
            .map(|_| {
                let key = SigningKey::random(&mut OsRng);
                let identifier = verifying_key_sha256(key.verifying_key());
                keys.insert(identifier.clone(), key.clone());
                MockRemoteEcdsaKey::new(identifier, key)
            })
            .collect_vec();
        drop(keys);

        let pops = attestation_keys
            .iter()
            .map(|attestation_key| {
                SignedJwt::sign_with_jwk(&claims, attestation_key)
                    .now_or_never()
                    .unwrap()
                    .unwrap()
                    .into()
            })
            .collect_vec()
            .try_into()
            .unwrap();

        let wua_and_key = include_wua.then(|| {
            let wua_key = SigningKey::random(&mut OsRng);
            let wua_key = MockRemoteEcdsaKey::new(verifying_key_sha256(wua_key.verifying_key()), wua_key);

            // If no WUA signing key is configured, just use the WUA's private key to sign it
            let wua_signing_key = self.wua_signing_key.as_ref().unwrap_or(&wua_key.key);
            let wua = WuaClaims::into_signed(
                wua_key.verifying_key(),
                wua_signing_key,
                MOCK_WALLET_CLIENT_ID.to_string(),
                Utc::now() + Duration::from_secs(600),
            )
            .now_or_never()
            .unwrap()
            .unwrap()
            .into();

            let wua_disclosure = SignedJwt::sign(&claims, &wua_key).now_or_never().unwrap().unwrap();

            (WuaDisclosure::new(wua, wua_disclosure.into()), wua_key)
        });

        let count_including_wua = if include_wua { count.get() + 1 } else { count.get() };
        let poa = (count_including_wua > 1).then(|| {
            Poa::new(
                attestation_keys
                    .iter()
                    .chain(wua_and_key.as_ref().map(|(_, key)| key))
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                claims,
            )
            .now_or_never()
            .unwrap()
            .unwrap()
        });

        Ok(IssuanceResult {
            key_identifiers: attestation_keys
                .into_iter()
                .map(|key| key.identifier)
                .collect_vec()
                .try_into()
                .unwrap(),
            pops,
            wua: wua_and_key.map(|(wua, _)| wua),
            poa,
        })
    }
}
