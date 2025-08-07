use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroUsize;

use derive_more::Constructor;
use derive_more::Debug;
use futures::FutureExt;
use futures::future;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Signer;
use parking_lot::Mutex;
use rand_core::OsRng;

use crypto::CredentialEcdsaKey;
use crypto::CredentialKeyType;
use crypto::EcdsaKey;
use crypto::SecureEcdsaKey;
use crypto::WithIdentifier;
use crypto::p256_der::verifying_key_sha256;
use jwt::Jwt;
use jwt::credential::JwtCredentialClaims;
use jwt::jwk::jwk_from_p256;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteClaims;
use jwt::wte::WteDisclosure;

use crate::Poa;
use crate::factory::mock::MOCK_WALLET_CLIENT_ID;
use crate::keyfactory::IssuanceResult;
use crate::keyfactory::KeyFactory;

#[derive(Debug, thiserror::Error)]
pub enum MockRemoteKeyFactoryError {
    #[error("key generation error")]
    Generating,
    #[error("signing error")]
    Signing,
    #[error("poa error")]
    Poa,
    #[error("ECDSA error: {0}")]
    Ecdsa(#[source] <MockRemoteEcdsaKey as EcdsaKey>::Error),
}

/// To be used in test in place of `RemoteEcdsaKey`, implementing the
/// [`EcdsaKey`], [`SecureEcdsaKey`] and [`WithIdentifier`] traits.
#[derive(Debug, Clone, Constructor)]
pub struct MockRemoteEcdsaKey {
    identifier: String,
    #[debug(skip)]
    pub key: SigningKey,
}

impl PartialEq for MockRemoteEcdsaKey {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for MockRemoteEcdsaKey {}

impl Hash for MockRemoteEcdsaKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl MockRemoteEcdsaKey {
    pub fn new_random(identifier: String) -> Self {
        Self::new(identifier, SigningKey::random(&mut OsRng))
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        self.key.verifying_key()
    }
}

impl EcdsaKey for MockRemoteEcdsaKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let key = self.key.verifying_key();

        Ok(*key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Signer::try_sign(&self.key, msg)
    }
}
impl SecureEcdsaKey for MockRemoteEcdsaKey {}

impl WithIdentifier for MockRemoteEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl CredentialEcdsaKey for MockRemoteEcdsaKey {
    const KEY_TYPE: CredentialKeyType = CredentialKeyType::Mock;
}

/// A type that implements [`KeyFactory`] and can be used in tests. It has the option
/// of returning `MockRemoteKeyFactoryError::Generating` when generating multiple
/// keys and `MockRemoteKeyFactoryError::Signing` when signing multiple, influenced
/// by boolean fields on the type.
#[derive(Debug)]
pub struct MockRemoteKeyFactory {
    signing_keys: Mutex<HashMap<String, SigningKey>>,
    wua_signing_key: Option<SigningKey>,

    pub has_generating_error: bool,
    pub has_multi_key_signing_error: bool,
    pub has_poa_error: bool,
}

impl MockRemoteKeyFactory {
    pub fn new(keys: Vec<MockRemoteEcdsaKey>) -> Self {
        let signing_keys = keys.into_iter().map(|key| (key.identifier, key.key)).collect();

        Self::new_signing_keys(signing_keys)
    }

    fn new_signing_keys(signing_keys: HashMap<String, SigningKey>) -> Self {
        Self {
            signing_keys: Mutex::new(signing_keys),
            wua_signing_key: None,
            has_generating_error: false,
            has_multi_key_signing_error: false,
            has_poa_error: false,
        }
    }

    pub fn new_with_wua_signing_key(wua_signing_key: SigningKey) -> Self {
        Self {
            wua_signing_key: Some(wua_signing_key),
            ..Default::default()
        }
    }

    pub fn add_key(&mut self, key: MockRemoteEcdsaKey) {
        self.signing_keys.get_mut().insert(key.identifier, key.key);
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

impl KeyFactory for MockRemoteKeyFactory {
    type Key = MockRemoteEcdsaKey;
    type Error = MockRemoteKeyFactoryError;

    fn generate_existing<I: Into<String>>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        let identifier = identifier.into();
        let signing_key = self
            .signing_keys
            .lock()
            .get(&identifier)
            .expect("called generate_existing() with unknown identifier")
            .clone();

        // If the provided public key does not match the key fetched
        // using the identifier, this is programmer error.
        assert_eq!(
            signing_key.verifying_key(),
            &public_key,
            "called generate_existing() with incorrect public_key"
        );

        MockRemoteEcdsaKey::new(identifier, signing_key)
    }

    async fn sign_multiple_with_existing_keys(
        &self,
        messages_and_keys: Vec<(Vec<u8>, Vec<&Self::Key>)>,
    ) -> Result<Vec<Vec<Signature>>, Self::Error> {
        if self.has_multi_key_signing_error {
            return Err(MockRemoteKeyFactoryError::Signing);
        }

        let result = future::try_join_all(
            messages_and_keys
                .into_iter()
                .map(|(msg, keys)| async move {
                    let signatures = future::try_join_all(keys.into_iter().map(|key| async {
                        let signature = key.try_sign(&msg).await.map_err(MockRemoteKeyFactoryError::Ecdsa)?;

                        Ok::<_, MockRemoteKeyFactoryError>(signature)
                    }))
                    .await?
                    .into_iter()
                    .collect::<Vec<_>>();

                    Ok::<_, MockRemoteKeyFactoryError>(signatures)
                })
                .collect::<Vec<_>>(),
        )
        .await?;

        Ok(result)
    }

    async fn perform_issuance(
        &self,
        count: NonZeroUsize,
        aud: String,
        nonce: Option<String>,
        include_wua: bool,
    ) -> Result<IssuanceResult, Self::Error> {
        let claims = JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud);

        let mut keys = self.signing_keys.lock();
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

        let wua_key = include_wua.then(|| {
            let key = SigningKey::random(&mut OsRng);
            MockRemoteEcdsaKey::new(verifying_key_sha256(key.verifying_key()), key)
        });
        let wua = include_wua.then(|| {
            // If no WUA signing key is configured, just use the WUA's private key to sign it
            let wua_signing_key = self.wua_signing_key.as_ref().unwrap_or(&wua_key.as_ref().unwrap().key);
            let wua = JwtCredentialClaims::new_signed(
                wua_key.as_ref().unwrap().verifying_key(),
                wua_signing_key,
                MOCK_WALLET_CLIENT_ID.to_string(),
                Some("wte+jwt".to_string()),
                WteClaims::new(),
            )
            .now_or_never()
            .unwrap()
            .unwrap();

            let wua_disclosure = Jwt::sign(&claims, &Header::new(Algorithm::ES256), wua_key.as_ref().unwrap())
                .now_or_never()
                .unwrap()
                .unwrap();

            WteDisclosure::new(wua, wua_disclosure)
        });

        let count_including_wua = if include_wua { count.get() + 1 } else { count.get() };
        let poa = (count_including_wua > 1).then(|| {
            Poa::new(
                attestation_keys
                    .iter()
                    .chain(wua_key.as_ref())
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
            wua,
            poa,
        })
    }
}
