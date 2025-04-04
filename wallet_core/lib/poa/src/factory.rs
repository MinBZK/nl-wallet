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
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use jwt::pop::JwtPopClaims;
    use utils::vec_at_least::VecAtLeastTwoUnique;

    use crate::error::PoaError;
    use crate::poa::Poa;

    use super::PoaFactory;

    pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";

    impl PoaFactory for MockRemoteKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = PoaError;

        async fn poa(
            &self,
            keys: VecAtLeastTwoUnique<&Self::Key>,
            aud: String,
            nonce: Option<String>,
        ) -> Result<Poa, Self::Error> {
            let poa = Poa::new(keys, JwtPopClaims::new(nonce, MOCK_WALLET_CLIENT_ID.to_string(), aud)).await?;

            Ok(poa)
        }
    }
}
