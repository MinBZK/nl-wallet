use std::sync::LazyLock;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::x509::CertificateUsage;
use derive_more::Constructor;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use utils::generator::TimeGenerator;

use crate::DEFAULT_VALIDATIONS;
use crate::JwtTyp;
use crate::UnverifiedJwt;
use crate::confirmation::ConfirmationClaim;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::error::JwtX5cError;
use crate::headers::HeaderWithX5c;
use crate::nonce::Nonce;
use crate::pop::JwtPopClaims;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaClaims {
    pub cnf: ConfirmationClaim,

    // Standard JWT fields, without `iss`; that is derived from the `x5c` certs
    pub sub: String,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub nbf: DateTime<Utc>,

    #[serde(flatten)]
    pub wallet_info: WiaWalletInfo,

    pub client_status: ClientStatus,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaWalletInfo {
    pub wallet_name: String,
    pub wallet_version: String,
    pub wallet_solution_certification_information: String,
    #[serde(default)]
    pub wallet_link: Option<String>, // TODO URL
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientStatus {
    // Revocation status of the Wallet Instance that presented the WIA.
    pub status: StatusClaim,

    // The duration for which the WP will track revocation status in the `status` URL.
    // (Distinct in terms of semantics as well as value from the top level WIA `exp` claim, which is max 24h.)
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

impl WiaClaims {
    pub fn new(
        holder_pubkey: &VerifyingKey,
        sub: String,
        exp: DateTime<Utc>,
        wallet_info: WiaWalletInfo,
        client_status: ClientStatus,
    ) -> Result<Self, JwtError> {
        Ok(Self {
            cnf: ConfirmationClaim::from_verifying_key(holder_pubkey)?,
            sub,
            exp,
            iat: Utc::now(),
            nbf: Utc::now(),
            wallet_info,
            client_status,
        })
    }
}

pub const WIA_JWT_TYP: &str = "oauth-client-attestation+jwt";

impl JwtTyp for WiaClaims {
    const TYP: &'static str = WIA_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WiaDisclosure(UnverifiedJwt<WiaClaims, HeaderWithX5c>, UnverifiedJwt<JwtPopClaims>);

#[cfg(feature = "test")]
impl WiaDisclosure {
    pub fn wia(&self) -> &UnverifiedJwt<WiaClaims, HeaderWithX5c> {
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
    #[error("JWT with certificate error: {0}")]
    JwtX5c(#[from] JwtX5cError),
}

impl WiaDisclosure {
    pub fn verify(
        &self,
        trust_anchors: &[BorrowingTrustAnchor],
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
    ) -> Result<(VerifyingKey, Nonce), WiaError> {
        let (_, verified_wia_claims) = self.0.parse_and_verify_against_trust_anchors(
            trust_anchors,
            &TimeGenerator,
            CertificateUsage::Wia,
            &WIA_JWT_VALIDATIONS,
        )?;
        let wia_pubkey = verified_wia_claims.cnf.verifying_key()?;
        tracing::debug!("WIA status claim: {:?}", verified_wia_claims.client_status.status);

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

    // Enforce validity of exp and nbf. (Presence is already enforced by the presence of the fields.)
    validations.validate_exp = true;
    validations.validate_nbf = true;

    validations
});
