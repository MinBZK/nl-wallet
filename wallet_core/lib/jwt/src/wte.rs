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
use crate::Jwt;
use crate::VerifiedJwt;
use crate::credential::JwtCredentialClaims;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_to_p256;
use crate::pop::JwtPopClaims;
use crate::validations;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WteClaims {
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

pub static WTE_EXPIRY: Duration = Duration::minutes(5);

impl WteClaims {
    pub fn new() -> Self {
        Self {
            exp: Utc::now() + WTE_EXPIRY,
        }
    }
}

impl Default for WteClaims {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WteDisclosure(pub Jwt<JwtCredentialClaims<WteClaims>>, pub Jwt<JwtPopClaims>);

#[derive(Debug, thiserror::Error)]
pub enum WteError {
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
}

impl WteDisclosure {
    pub fn verify(
        self,
        issuer_public_key: &EcdsaDecodingKey,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
        expected_nonce: &str,
    ) -> Result<(VerifiedJwt<JwtCredentialClaims<WteClaims>>, VerifyingKey), WteError> {
        let verified_jwt = VerifiedJwt::try_new(self.0, issuer_public_key, &WTE_JWT_VALIDATIONS)?;
        let wte_pubkey = jwk_to_p256(&verified_jwt.payload().confirmation.jwk)?;

        let mut validations = validations();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let wte_disclosure_claims = self.1.parse_and_verify(&(&wte_pubkey).into(), &validations)?;

        if wte_disclosure_claims.nonce.as_deref() != Some(expected_nonce) {
            return Err(WteError::IncorrectNonce);
        }

        Ok((verified_jwt, wte_pubkey))
    }
}

// Returns the JWS validations for WTE verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WTE issuer and the caller.
pub static WTE_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = validations();
    validations.leeway = 0;

    // Enforce presence and validity of exp.
    validations.set_required_spec_claims(&["exp"]);
    validations.validate_exp = true;

    validations
});
