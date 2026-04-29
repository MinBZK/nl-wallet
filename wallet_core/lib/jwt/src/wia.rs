use std::sync::LazyLock;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use derive_more::Constructor;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::DEFAULT_VALIDATIONS;
use crate::EcdsaDecodingKey;
use crate::JwtTyp;
use crate::UnverifiedJwt;
use crate::confirmation::ConfirmationClaim;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::nonce::Nonce;
use crate::pop::JwtPopClaims;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaClaims {
    pub cnf: ConfirmationClaim,

    pub iss: String,

    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,

    pub status: StatusClaim,
}

impl WiaClaims {
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

pub const WIA_JWT_TYP: &str = "wia+jwt";

impl JwtTyp for WiaClaims {
    const TYP: &'static str = WIA_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WiaDisclosure(UnverifiedJwt<WiaClaims>, UnverifiedJwt<JwtPopClaims>);

#[cfg(feature = "test")]
impl WiaDisclosure {
    pub fn wia(&self) -> &UnverifiedJwt<WiaClaims> {
        &self.0
    }

    pub fn wia_pop(&self) -> &UnverifiedJwt<JwtPopClaims> {
        &self.1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WiaError {
    #[error("nonce is missing from WIA payload")]
    MissingNonce,
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
}

impl WiaDisclosure {
    pub fn verify(
        &self,
        issuer_public_key: &EcdsaDecodingKey,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
    ) -> Result<(VerifyingKey, Nonce), WiaError> {
        let (_, verified_wia_claims) = self.0.parse_and_verify(issuer_public_key, &WIA_JWT_VALIDATIONS)?;
        let wia_pubkey = verified_wia_claims.cnf.verifying_key()?;
        tracing::debug!("WIA status claim: {:?}", verified_wia_claims.status);

        let mut validations = DEFAULT_VALIDATIONS.to_owned();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let (_, wia_disclosure_claims) = self.1.parse_and_verify(&(&wia_pubkey).into(), &validations)?;

        let nonce = wia_disclosure_claims.nonce.ok_or(WiaError::MissingNonce)?;

        Ok((wia_pubkey, nonce))
    }
}

// Returns the JWS validations for WIA verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WIA issuer and the caller.
pub static WIA_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = DEFAULT_VALIDATIONS.to_owned();
    validations.leeway = 0;

    // Enforce presence and validity of exp.
    validations.set_required_spec_claims(&["exp"]);
    validations.validate_exp = true;

    validations
});
