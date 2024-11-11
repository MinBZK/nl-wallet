use derive_more::{AsRef, From, Into};
use serde::{Deserialize, Serialize};

/// A newtype around `Vec<u8>` that represent an assertion generated by Apple AppAttest.
/// It is to be treated as opaque bytes until received by the server.
#[derive(Debug, Clone, From, Into, AsRef, Serialize, Deserialize)]
pub struct AppleAssertion(Vec<u8>);

/// Trait for an Apple attested key. Note that [`SecureEcdsaKey`] is not
/// a supertrait, since signing does not produce an ECDSA signature.
pub trait AppleAttestedKey {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generate an Apple assertion using the attested key, which returns the [`AppleAssertion`] newtype.
    async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error>;
}

#[cfg(any(test, feature = "mock_apple_attested_key"))]
pub use mock_apple_attested_key::MockAppleAttestedKey;

#[cfg(any(test, feature = "mock_apple_attested_key"))]
mod mock_apple_attested_key {
    use std::convert::Infallible;

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use apple_app_attest::{AppIdentifier, Assertion};

    use super::{AppleAssertion, AppleAttestedKey};

    pub struct MockAppleAttestedKey {
        pub signing_key: SigningKey,
        pub app_identifier: AppIdentifier,
        pub counter: u32,
    }

    impl MockAppleAttestedKey {
        pub fn new(app_identifier: AppIdentifier) -> Self {
            Self {
                signing_key: SigningKey::random(&mut OsRng),
                app_identifier,
                counter: 1,
            }
        }
    }

    impl AppleAttestedKey for MockAppleAttestedKey {
        type Error = Infallible;

        async fn sign(&self, payload: Vec<u8>) -> Result<AppleAssertion, Self::Error> {
            let assertion = Assertion::new_mock(&self.signing_key, &self.app_identifier, self.counter, &payload);

            let mut bytes = Vec::<u8>::new();
            ciborium::into_writer(&assertion, &mut bytes).unwrap();

            Ok(bytes.into())
        }
    }
}
