use std::collections::HashMap;
use std::num::NonZeroUsize;

use derive_more::Debug;
use futures::FutureExt;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;

use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteKeyFactory as DisclosureMockRemoteKeyFactory;
use crypto::mock_remote::MockRemoteKeyFactoryError;
use crypto::p256_der::verifying_key_sha256;
use crypto::wscd::DisclosureKeyFactory;
use crypto::wscd::DisclosureResult;
use crypto::wscd::KeyFactoryPoa;
use jwt::Jwt;
use jwt::credential::JwtCredentialClaims;
use jwt::jwk::jwk_from_p256;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteClaims;
use jwt::wte::WteDisclosure;

use crate::MOCK_WALLET_CLIENT_ID;
use crate::Poa;
use crate::keyfactory::IssuanceResult;
use crate::keyfactory::KeyFactory;

/// A type that implements [`KeyFactory`] and can be used in tests. It has the option
/// of returning `MockRemoteKeyFactoryError::Generating` when generating multiple
/// keys and `MockRemoteKeyFactoryError::Signing` when signing multiple, influenced
/// by boolean fields on the type.
#[derive(Debug)]
pub struct MockRemoteKeyFactory {
    pub disclosure: DisclosureMockRemoteKeyFactory,
    wua_signing_key: Option<SigningKey>,
}

impl MockRemoteKeyFactory {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            disclosure: DisclosureMockRemoteKeyFactory::new_signing_keys(signing_keys),
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
}

impl Default for MockRemoteKeyFactory {
    fn default() -> Self {
        Self::new_signing_keys(HashMap::new())
    }
}

impl DisclosureKeyFactory for MockRemoteKeyFactory {
    type Key = MockRemoteEcdsaKey;
    type Error = MockRemoteKeyFactoryError;
    type Poa = Poa;

    fn new_key<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        self.disclosure.new_key(identifier, public_key)
    }

    async fn sign(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
        poa_input: <Self::Poa as KeyFactoryPoa>::Input,
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
                .map_err(|_| MockRemoteKeyFactoryError::Poa)?,
            )
        };

        let DisclosureResult { signatures, .. } = self.disclosure.sign(messages_and_keys, ()).await?;

        Ok(DisclosureResult { signatures, poa })
    }
}

impl KeyFactory for MockRemoteKeyFactory {
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
                let header = Header {
                    typ: Some("openid4vci-proof+jwt".to_string()),
                    alg: Algorithm::ES256,
                    jwk: Some(jwk_from_p256(attestation_key.verifying_key()).unwrap()),
                    ..Default::default()
                };

                Jwt::sign(&claims, &header, attestation_key)
                    .now_or_never()
                    .unwrap()
                    .unwrap()
            })
            .collect_vec()
            .try_into()
            .unwrap();

        let wua_and_key = include_wua.then(|| {
            let wua_key = SigningKey::random(&mut OsRng);
            let wua_key = MockRemoteEcdsaKey::new(verifying_key_sha256(wua_key.verifying_key()), wua_key);

            // If no WUA signing key is configured, just use the WUA's private key to sign it
            let wua_signing_key = self.wua_signing_key.as_ref().unwrap_or(&wua_key.key);
            let wua = JwtCredentialClaims::new_signed(
                wua_key.verifying_key(),
                wua_signing_key,
                MOCK_WALLET_CLIENT_ID.to_string(),
                Some("wte+jwt".to_string()),
                WteClaims::new(),
            )
            .now_or_never()
            .unwrap()
            .unwrap();

            let wua_disclosure = Jwt::sign(&claims, &Header::new(Algorithm::ES256), &wua_key)
                .now_or_never()
                .unwrap()
                .unwrap();

            (WteDisclosure::new(wua, wua_disclosure), wua_key)
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
