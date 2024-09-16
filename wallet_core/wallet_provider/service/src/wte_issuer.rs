use std::error::Error;

use chrono::{TimeDelta, Utc};
use indexmap::IndexMap;
use jsonwebtoken::Header;

use wallet_common::jwt::{
    jwk_from_p256, JwkConversionError, Jwt, JwtCredentialClaims, JwtCredentialCnf, JwtCredentialContents,
};
use wallet_provider_domain::model::{hsm::WalletUserHsm, wrapped_key::WrappedKey};

use crate::{hsm::HsmError, keys::WalletProviderEcdsaKey};

pub trait WteIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims>), Self::Error>;
}

pub struct HsmWteIssuer<H> {
    private_key: WalletProviderEcdsaKey,
    iss: String,
    hsm: H,
}

impl<H> HsmWteIssuer<H> {
    pub fn new(private_key: WalletProviderEcdsaKey, iss: String, hsm: H) -> Self {
        Self { private_key, iss, hsm }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HsmWteIssuerError {
    #[error("HSM error: {0}")]
    Hsm(#[from] HsmError),
    #[error("key conversion error: {0}")]
    KeyConversion(#[from] JwkConversionError),
}

static WTE_JWT_TYP: &str = "wte+jwt";

impl<H: WalletUserHsm<Error = HsmError>> WteIssuer for HsmWteIssuer<H> {
    type Error = HsmWteIssuerError;

    async fn issue_wte(&self) -> Result<(WrappedKey, Jwt<JwtCredentialClaims>), Self::Error> {
        let (pubkey, wrapped_privkey) = self.hsm.generate_wrapped_key().await?;
        let jwk = jwk_from_p256(&pubkey)?;

        let jwt = Jwt::<JwtCredentialClaims>::sign(
            &JwtCredentialClaims {
                cnf: JwtCredentialCnf { jwk },
                contents: JwtCredentialContents {
                    iss: self.iss.clone(),
                    attributes: IndexMap::from([(
                        "exp".to_string(),
                        Utc::now()
                            .checked_add_signed(TimeDelta::minutes(5))
                            .unwrap() // Adding 5 minutes won't overflow
                            .timestamp()
                            .into(),
                    )]),
                },
            },
            &Header {
                typ: Some(WTE_JWT_TYP.to_string()),
                ..Header::new(jsonwebtoken::Algorithm::ES256)
            },
            &self.private_key,
        )
        .await
        .unwrap();

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
