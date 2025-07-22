use std::error::Error;

use crypto::keys::CredentialEcdsaKey;
use utils::vec_at_least::VecAtLeastTwoUnique;

use crate::poa::Poa;

pub trait PoaFactory {
    type Key: CredentialEcdsaKey;
    type Error: Error + Send + Sync + 'static;

    /// Construct a Proof of Association, with which the key factory asserts that all provided keys
    /// are managed by this one key factory.
    async fn poa(
        &self,
        keys: VecAtLeastTwoUnique<&Self::Key>,
        aud: String,
        nonce: Option<String>,
    ) -> Result<Poa, Self::Error>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use jwt::pop::JwtPopClaims;
    use utils::vec_at_least::VecAtLeastTwoUnique;

    use crate::mock_remote::MockRemoteEcdsaKey;
    use crate::mock_remote::MockRemoteKeyFactory;
    use crate::mock_remote::MockRemoteKeyFactoryError;
    use crate::poa::Poa;

    use super::PoaFactory;

    pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";

    impl PoaFactory for MockRemoteKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = MockRemoteKeyFactoryError;

        async fn poa(
            &self,
            keys: VecAtLeastTwoUnique<&Self::Key>,
            aud: String,
            nonce: Option<String>,
        ) -> Result<Poa, Self::Error> {
            if self.has_poa_error {
                return Err(MockRemoteKeyFactoryError::Poa);
            }

            let poa = Poa::new(keys, JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud))
                .await
                .unwrap();

            Ok(poa)
        }
    }
}
