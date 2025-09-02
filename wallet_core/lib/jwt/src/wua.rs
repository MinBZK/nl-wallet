use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use chrono::serde::ts_seconds;
use derive_more::Constructor;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::EcdsaDecodingKey;
use crate::UnverifiedJwt;
use crate::VerifiedJwt;
use crate::credential::JwtCredentialClaims;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_to_p256;
use crate::pop::JwtPopClaims;
use crate::validations;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WuaClaims {
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

pub static WUA_EXPIRY: Duration = Duration::minutes(5);

impl WuaClaims {
    pub fn new() -> Self {
        Self {
            exp: Utc::now() + WUA_EXPIRY,
        }
    }
}

impl Default for WuaClaims {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WuaDisclosure(UnverifiedJwt<JwtCredentialClaims<WuaClaims>>, UnverifiedJwt<JwtPopClaims>);

#[cfg(feature = "test")]
impl WuaDisclosure {
    pub fn wua(&self) -> &UnverifiedJwt<JwtCredentialClaims<WuaClaims>> {
        &self.0
    }

    pub fn wua_pop(&self) -> &UnverifiedJwt<JwtPopClaims> {
        &self.1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WuaError {
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
}

impl WuaDisclosure {
    pub fn verify(
        self,
        issuer_public_key: &EcdsaDecodingKey,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
        expected_nonce: &str,
    ) -> Result<(VerifiedJwt<JwtCredentialClaims<WuaClaims>>, VerifyingKey), WuaError> {
        let verified_jwt = VerifiedJwt::try_new(self.0, issuer_public_key, &WUA_JWT_VALIDATIONS)?;
        let wua_pubkey = jwk_to_p256(&verified_jwt.payload().confirmation.jwk)?;

        let mut validations = validations();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let wua_disclosure_claims = self.1.parse_and_verify(&(&wua_pubkey).into(), &validations)?;

        if wua_disclosure_claims.nonce.as_deref() != Some(expected_nonce) {
            return Err(WuaError::IncorrectNonce);
        }

        Ok((verified_jwt, wua_pubkey))
    }
}

// Returns the JWS validations for WUA verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WUA issuer and the caller.
pub static WUA_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = validations();
    validations.leeway = 0;

    // Enforce presence and validity of exp.
    validations.set_required_spec_claims(&["exp"]);
    validations.validate_exp = true;

    validations
});
