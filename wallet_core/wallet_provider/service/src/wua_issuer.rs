use std::error::Error;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use p256::ecdsa::VerifyingKey;

use crypto::keys::SecureEcdsaKey;
use crypto::p256_der::verifying_key_sha256;
use hsm::keys::HsmEcdsaKey;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;
use jwt::wua::WuaClaims;
use wallet_provider_domain::model::hsm::WalletUserHsm;

// used as the identifier for a WUA specific token status list
pub const WUA_ATTESTATION_TYPE_IDENTIFIER: &str = "wua";

pub trait WuaIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wua(
        &self,
        exp: DateTime<Utc>,
    ) -> Result<(WrappedKey, String, UnverifiedJwt<WuaClaims>), Self::Error>;
    async fn public_key(&self) -> Result<VerifyingKey, Self::Error>;
}

#[derive(Constructor)]
pub struct HsmWuaIssuer<H, K = HsmEcdsaKey> {
    private_key: K,
    iss: String,
    hsm: H,
    wrapping_key_identifier: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HsmWuaIssuerError {
    #[error("HSM error: {0}")]
    Hsm(#[from] HsmError),
    #[error("JWT error: {0}")]
    KeyConversion(#[from] JwtError),
    #[error("public key error: {0}")]
    PublicKeyError(Box<dyn Error + Send + Sync + 'static>),
}

impl<H, K> WuaIssuer for HsmWuaIssuer<H, K>
where
    H: WalletUserHsm<Error = HsmError>,
    K: SecureEcdsaKey,
{
    type Error = HsmWuaIssuerError;

    async fn issue_wua(
        &self,
        exp: DateTime<Utc>, // TODO status_claim: ... (PVW-4574)
    ) -> Result<(WrappedKey, String, UnverifiedJwt<WuaClaims>), Self::Error> {
        let wrapped_privkey = self.hsm.generate_wrapped_key(&self.wrapping_key_identifier).await?;
        let pubkey = *wrapped_privkey.public_key();

        // TODO add `status_claim` to WuaClaims (PVW-4574)
        let jwt = WuaClaims::into_signed(&pubkey, &self.private_key, self.iss.clone(), exp)
            .await?
            .into();

        Ok((wrapped_privkey, verifying_key_sha256(&pubkey), jwt))
    }

    async fn public_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.private_key
            .verifying_key()
            .await
            .map_err(|e| HsmWuaIssuerError::PublicKeyError(Box::new(e)))
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use chrono::DateTime;
    use chrono::Utc;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::p256_der::verifying_key_sha256;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::UnverifiedJwt;
    use jwt::wua::WuaClaims;

    use super::WuaIssuer;

    pub struct MockWuaIssuer;

    impl WuaIssuer for MockWuaIssuer {
        type Error = Infallible;

        async fn issue_wua(
            &self,
            exp: DateTime<Utc>,
        ) -> Result<(WrappedKey, String, UnverifiedJwt<WuaClaims>), Self::Error> {
            let privkey = SigningKey::random(&mut OsRng);
            let pubkey = privkey.verifying_key();

            let jwt = WuaClaims::into_signed(
                pubkey,
                &privkey, // Sign the WUA with its own private key in this test
                "iss".to_string(),
                exp,
            )
            .await
            .unwrap()
            .into();

            Ok((
                WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
                verifying_key_sha256(privkey.verifying_key()),
                jwt,
            ))
        }

        async fn public_key(&self) -> Result<p256::ecdsa::VerifyingKey, Self::Error> {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use jwt::DEFAULT_VALIDATIONS;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;

    use super::HsmWuaIssuer;
    use super::WuaIssuer;

    #[tokio::test]
    async fn it_works() {
        let hsm = MockPkcs11Client::<HsmError>::default();
        let wua_signing_key = SigningKey::random(&mut OsRng);
        let wua_verifying_key = wua_signing_key.verifying_key();
        let iss = "iss";
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wua_issuer = HsmWuaIssuer {
            private_key: wua_signing_key.clone(),
            iss: iss.to_string(),
            hsm,
            wrapping_key_identifier: wrapping_key_identifier.to_string(),
        };

        let (wua_privkey, _key_id, wua) = wua_issuer
            .issue_wua(Utc::now() + Duration::from_secs(600))
            .await
            .unwrap();

        let (_, wua_claims) = wua
            .parse_and_verify(&wua_verifying_key.into(), &DEFAULT_VALIDATIONS)
            .unwrap();

        assert_eq!(
            wua_privkey.public_key(),
            &wua_claims.confirmation.verifying_key().unwrap()
        );

        // Check that the fields have the expected contents
        assert_eq!(wua_claims.iss, iss.to_string());
        assert!(wua_claims.exp > Utc::now());
    }
}
