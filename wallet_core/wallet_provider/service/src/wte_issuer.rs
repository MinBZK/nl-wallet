use std::error::Error;

use chrono::{TimeDelta, Utc};
use indexmap::IndexMap;

use wallet_common::{
    jwt::{Jwt, JwtCredentialClaims, JwtError},
    keys::SecureEcdsaKey,
};
use wallet_provider_domain::model::{hsm::WalletUserHsm, wrapped_key::WrappedKey};

use crate::{hsm::HsmError, keys::WalletProviderEcdsaKey};

pub trait WteIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims>), Self::Error>;
}

pub struct HsmWteIssuer<H, K = WalletProviderEcdsaKey> {
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
}

static WTE_JWT_TYP: &str = "wte+jwt";

impl<H, K> WteIssuer for HsmWteIssuer<H, K>
where
    H: WalletUserHsm<Error = HsmError>,
    K: SecureEcdsaKey,
{
    type Error = HsmWteIssuerError;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims>), Self::Error> {
        let (pubkey, wrapped_privkey) = self.hsm.generate_wrapped_key().await?;

        let jwt = JwtCredentialClaims::new_signed(
            &pubkey,
            &self.private_key,
            self.iss.clone(),
            Some(WTE_JWT_TYP.to_string()),
            IndexMap::from([(
                "exp".to_string(),
                Utc::now()
                    .checked_add_signed(TimeDelta::minutes(5))
                    .unwrap() // Adding 5 minutes won't overflow
                    .timestamp()
                    .into(),
            )]),
        )
        .await?;

        Ok((wrapped_privkey, jwt))
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use wallet_common::jwt::{Jwt, JwtCredentialClaims};
    use wallet_provider_domain::model::wrapped_key::WrappedKey;

    use super::WteIssuer;

    pub struct MockWteIssuer;

    impl WteIssuer for MockWteIssuer {
        type Error = Infallible;

        async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims>), Self::Error> {
            Ok((WrappedKey::new(b"mock_key_bytes".to_vec()), "a.b.c".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use p256::ecdsa::signature::Verifier;

    use wallet_common::{
        jwt::{self, jwk_to_p256},
        keys::{software::SoftwareEcdsaKey, EcdsaKey},
    };
    use wallet_provider_domain::model::hsm::{mock::MockPkcs11Client, WalletUserHsm};

    use crate::hsm::HsmError;

    use super::{HsmWteIssuer, WteIssuer};

    #[tokio::test]
    async fn it_works() {
        let hsm = MockPkcs11Client::<HsmError>::default();
        let wte_signing_key = SoftwareEcdsaKey::new_random("wte_signing_key".to_string());
        let wte_verifying_key = wte_signing_key.verifying_key().await.unwrap();
        let iss = "iss";

        let wte_issuer = HsmWteIssuer {
            private_key: wte_signing_key.clone(),
            iss: iss.to_string(),
            hsm,
        };

        let (wte_privkey, wte) = wte_issuer.issue_wte().await.unwrap();

        let wte_claims = wte
            .parse_and_verify(&(&wte_verifying_key).into(), &jwt::validations())
            .unwrap();

        // We want to check that the public key of `wte_privkey` equals the public key in the `wte_claims`,
        // but the `hsm` API does not allow us to do that. So instead we check that we can sign things
        // with it that validate against the public key in the `wte_claims`.
        let sig = wte_issuer
            .hsm
            .sign_wrapped(wte_privkey, Arc::new(b"".to_vec()))
            .await
            .unwrap();
        jwk_to_p256(&wte_claims.cnf.jwk).unwrap().verify(b"", &sig).unwrap();

        // Check that the fields have the expected contents
        assert_eq!(wte_claims.contents.iss, iss.to_string());
        assert!(wte_claims.contents.attributes["exp"].as_i64().unwrap() > Utc::now().timestamp());
    }
}
