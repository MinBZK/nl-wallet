use std::error::Error;

use derive_more::Constructor;
use p256::ecdsa::VerifyingKey;

use crypto::keys::SecureEcdsaKey;
use crypto::p256_der::verifying_key_sha256;
use hsm::keys::HsmEcdsaKey;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use jwt::Jwt;
use jwt::credential::JwtCredentialClaims;
use jwt::error::JwtError;
use jwt::wua::WuaClaims;
use wallet_provider_domain::model::hsm::WalletUserHsm;

pub trait WuaIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wua(&self) -> Result<(WrappedKey, String, Jwt<JwtCredentialClaims<WuaClaims>>), Self::Error>;
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

static WUA_JWT_TYP: &str = "wua+jwt";

impl<H, K> WuaIssuer for HsmWuaIssuer<H, K>
where
    H: WalletUserHsm<Error = HsmError>,
    K: SecureEcdsaKey,
{
    type Error = HsmWuaIssuerError;

    async fn issue_wua(&self) -> Result<(WrappedKey, String, Jwt<JwtCredentialClaims<WuaClaims>>), Self::Error> {
        let wrapped_privkey = self.hsm.generate_wrapped_key(&self.wrapping_key_identifier).await?;
        let pubkey = *wrapped_privkey.public_key();

        let jwt = JwtCredentialClaims::new_signed(
            &pubkey,
            &self.private_key,
            self.iss.clone(),
            Some(WUA_JWT_TYP.to_string()),
            WuaClaims::new(),
        )
        .await?;

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

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::p256_der::verifying_key_sha256;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::Jwt;
    use jwt::credential::JwtCredentialClaims;
    use jwt::wua::WuaClaims;

    use super::WUA_JWT_TYP;
    use super::WuaIssuer;

    pub struct MockWuaIssuer;

    impl WuaIssuer for MockWuaIssuer {
        type Error = Infallible;

        async fn issue_wua(&self) -> Result<(WrappedKey, String, Jwt<JwtCredentialClaims<WuaClaims>>), Self::Error> {
            let privkey = SigningKey::random(&mut OsRng);
            let pubkey = privkey.verifying_key();

            let jwt = JwtCredentialClaims::new_signed(
                pubkey,
                &privkey, // Sign the WUA with its own private key in this test
                "iss".to_string(),
                Some(WUA_JWT_TYP.to_string()),
                WuaClaims::new(),
            )
            .await
            .unwrap();

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
    use chrono::Utc;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use jwt::jwk::jwk_to_p256;

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

        let (wua_privkey, _key_id, wua) = wua_issuer.issue_wua().await.unwrap();

        let wua_claims = wua
            .parse_and_verify(&wua_verifying_key.into(), &jwt::validations())
            .unwrap();

        assert_eq!(
            wua_privkey.public_key(),
            &jwk_to_p256(&wua_claims.confirmation.jwk).unwrap()
        );

        // Check that the fields have the expected contents
        assert_eq!(wua_claims.contents.iss, iss.to_string());
        assert!(wua_claims.contents.attributes.exp > Utc::now());
    }
}
