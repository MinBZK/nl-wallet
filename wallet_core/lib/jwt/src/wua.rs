use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use derive_more::Constructor;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use attestation_types::status_claim::StatusClaim;

use crate::DEFAULT_VALIDATIONS;
use crate::EcdsaDecodingKey;
use crate::JwtTyp;
use crate::UnverifiedJwt;
use crate::confirmation::ConfirmationClaim;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::pop::JwtPopClaims;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WuaClaims {
    pub cnf: ConfirmationClaim,

    pub iss: String,

    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,

    pub status: StatusClaim,
}

impl WuaClaims {
    pub fn new(
        holder_pubkey: &VerifyingKey,
        iss: String,
        exp: DateTime<Utc>,
        status: StatusClaim,
    ) -> Result<Self, JwtError> {
        Ok(Self {
            cnf: ConfirmationClaim::from_verifying_key(holder_pubkey)?,
            iss,
            exp,
            status,
        })
    }
}

pub const WUA_JWT_TYP: &str = "wua+jwt";

impl JwtTyp for WuaClaims {
    const TYP: &'static str = WUA_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WuaDisclosure(UnverifiedJwt<WuaClaims>, UnverifiedJwt<JwtPopClaims>);

#[cfg(feature = "test")]
impl WuaDisclosure {
    pub fn wua(&self) -> &UnverifiedJwt<WuaClaims> {
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
        &self,
        issuer_public_key: &EcdsaDecodingKey,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
        expected_nonce: &str,
    ) -> Result<VerifyingKey, WuaError> {
        let (_, verified_wua_claims) = self.0.parse_and_verify(issuer_public_key, &WUA_JWT_VALIDATIONS)?;
        let wua_pubkey = verified_wua_claims.cnf.verifying_key()?;
        tracing::debug!("WUA status claim: {:?}", verified_wua_claims.status);

        let mut validations = DEFAULT_VALIDATIONS.to_owned();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let (_, wua_disclosure_claims) = self.1.parse_and_verify(&(&wua_pubkey).into(), &validations)?;

        if wua_disclosure_claims.nonce.as_deref() != Some(expected_nonce) {
            return Err(WuaError::IncorrectNonce);
        }

        Ok(wua_pubkey)
    }
}

// Returns the JWS validations for WUA verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WUA issuer and the caller.
pub static WUA_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = DEFAULT_VALIDATIONS.to_owned();
    validations.leeway = 0;

    // Enforce presence and validity of exp.
    validations.set_required_spec_claims(&["exp"]);
    validations.validate_exp = true;

    validations
});
