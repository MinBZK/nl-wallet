use std::error::Error;
use std::hash::Hash;

use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::vec_at_least::VecAtLeastTwoUnique;

use crate::Poa;

pub trait PoaFactory {
    type Key: CredentialEcdsaKey + Eq + Hash;
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
    use wallet_common::jwt::JwtPopClaims;
    use wallet_common::jwt::NL_WALLET_CLIENT_ID;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;
    use wallet_common::keys::mock_remote::MockRemoteKeyFactory;
    use wallet_common::vec_at_least::VecAtLeastTwoUnique;

    use crate::Poa;
    use crate::PoaError;

    use super::PoaFactory;

    impl PoaFactory for MockRemoteKeyFactory {
        type Key = MockRemoteEcdsaKey;
        type Error = PoaError;

        async fn poa(
            &self,
            keys: VecAtLeastTwoUnique<&Self::Key>,
            aud: String,
            nonce: Option<String>,
        ) -> Result<Poa, Self::Error> {
            let poa = Poa::generate(keys, JwtPopClaims::new(nonce, NL_WALLET_CLIENT_ID.to_string(), aud)).await?;

            Ok(poa)
        }
    }
}
