use std::error::Error;

use p256::ecdsa::VerifyingKey;

use hsm::keys::HsmEcdsaKey;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtCredentialClaims;
use wallet_common::jwt::JwtError;
use wallet_common::keys::SecureEcdsaKey;
use wallet_common::wte::WteClaims;
use wallet_provider_domain::model::hsm::WalletUserHsm;

pub trait WteIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims<WteClaims>>), Self::Error>;
    async fn public_key(&self) -> Result<VerifyingKey, Self::Error>;
}

pub struct HsmWteIssuer<H, K = HsmEcdsaKey> {
    private_key: K,
    iss: String,
    hsm: H,
}

impl<H, K> HsmWteIssuer<H, K> {
    pub fn new(private_key: K, iss: String, hsm: H) -> Self {
        Self { private_key, iss, hsm }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HsmWteIssuerError {
    #[error("HSM error: {0}")]
    Hsm(#[from] HsmError),
    #[error("JWT error: {0}")]
    KeyConversion(#[from] JwtError),
    #[error("public key error: {0}")]
    PublicKeyError(Box<dyn Error + Send + Sync + 'static>),
}

static WTE_JWT_TYP: &str = "wte+jwt";

impl<H, K> WteIssuer for HsmWteIssuer<H, K>
where
    H: WalletUserHsm<Error = HsmError>,
    K: SecureEcdsaKey,
{
    type Error = HsmWteIssuerError;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims<WteClaims>>), Self::Error> {
        let (pubkey, wrapped_privkey) = self.hsm.generate_wrapped_key().await?;

        let jwt = JwtCredentialClaims::new_signed(
            &pubkey,
            &self.private_key,
            self.iss.clone(),
            Some(WTE_JWT_TYP.to_string()),
            WteClaims::new(),
        )
        .await?;

        Ok((wrapped_privkey, jwt))
    }

    async fn public_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.private_key
            .verifying_key()
            .await
            .map_err(|e| HsmWteIssuerError::PublicKeyError(Box::new(e)))
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use hsm::model::wrapped_key::WrappedKey;
    use wallet_common::jwt::Jwt;
    use wallet_common::jwt::JwtCredentialClaims;
    use wallet_common::wte::WteClaims;

    use super::WteIssuer;

    pub struct MockWteIssuer;

    impl WteIssuer for MockWteIssuer {
        type Error = Infallible;

        async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims<WteClaims>>), Self::Error> {
            let privkey = SigningKey::random(&mut OsRng);
            Ok((
                WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
                "a.b.c".into(),
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

    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use wallet_common::jwt;
    use wallet_common::jwt::jwk_to_p256;

    use super::HsmWteIssuer;
    use super::WteIssuer;

    #[tokio::test]
    async fn it_works() {
        let hsm = MockPkcs11Client::<HsmError>::default();
        let wte_signing_key = SigningKey::random(&mut OsRng);
        let wte_verifying_key = wte_signing_key.verifying_key();
        let iss = "iss";

        let wte_issuer = HsmWteIssuer {
            private_key: wte_signing_key.clone(),
            iss: iss.to_string(),
            hsm,
        };

        let (wte_privkey, wte) = wte_issuer.issue_wte().await.unwrap();

        let wte_claims = wte
            .parse_and_verify(&wte_verifying_key.into(), &jwt::validations())
            .unwrap();

        assert_eq!(
            wte_privkey.public_key(),
            &jwk_to_p256(&wte_claims.confirmation.jwk).unwrap()
        );

        // Check that the fields have the expected contents
        assert_eq!(wte_claims.contents.iss, iss.to_string());
        assert!(wte_claims.contents.attributes.exp > Utc::now());
    }
}
