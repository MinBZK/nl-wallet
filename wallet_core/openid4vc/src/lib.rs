use base64::prelude::*;
use credential::CredentialErrorType;
use jsonwebtoken::jwk::{self, EllipticCurve, Jwk};
use nl_wallet_mdoc::utils::serialization::CborError;
use p256::{
    ecdsa::{signature, VerifyingKey},
    EncodedPoint,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use token::TokenErrorType;
use url::Url;
use wallet_common::jwt::JwtError;

pub mod authorization;
pub mod credential;
pub mod token;

pub mod dpop;

pub mod issuance_client;
pub mod issuer;

pub const NL_WALLET_CLIENT_ID: &str = "https://example.com";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    MsoMdoc,

    // Other formats we don't currently support; we include them here so we can give the appropriate error message
    // when they might be requested by the wallet (as opposed to a deserialization error).
    // The OpenID4VCI and OpenID4VP specs aim to be general and do not provide an exhaustive list; the formats below
    // are found as examples in the specs.
    LdpVc,
    JwtVc,
    JwtVcJson,
    AcVc, // Anonymous Credentials i.e. Idemix
}

// TODO implement once we support Wallet (Instance) Attestation / Wallet Trust Anchor,
// and include it as well as the ClientAssertion itself in the `AuthorizationRequest` and `TokenRequest`
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub enum ClientAssertionType {
//     #[serde(rename = "urn:ietf:params:oauth:client-assertion-type:jwt-client-attestation")]
//     JwtClientAttestation,
// }

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported JWT algorithm: expected {expected}, found {found}")]
    UnsupportedJwtAlgorithm { expected: String, found: String },
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("JWT signing failed: {0}")]
    JwtSigningFailed(#[from] wallet_common::errors::Error),
    #[error("JWT decoding failed: {0}")]
    JwtDecodingFailed(#[from] jsonwebtoken::errors::Error),
    #[error("incorrect DPoP JWT HTTP method")]
    IncorrectDpopMethod,
    #[error("incorrect DPoP JWT url")]
    IncorrectDpopUrl,
    #[error("incorrect DPoP JWT access token hash")]
    IncorrectDpopAccessTokenHash,
    #[error("missing JWK")]
    MissingJwk,
    #[error("incorrect JWK public key")]
    IncorrectJwkPublicKey,
    #[error(transparent)]
    JwkConversion(#[from] JwkConversionError),
    #[error(transparent)]
    Jwt(#[from] JwtError),
    #[error("URL encoding failed: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
    #[error("http request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("missing c_nonce")]
    MissingNonce,
    #[error("JSON (de)serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CBOR (de)serialization error: {0}")]
    Cbor(#[from] CborError),
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("not all expected attributes were issued")]
    ExpectedAttributesMissing,
    #[error("mdoc verification failed: {0}")]
    MdocVerification(#[source] nl_wallet_mdoc::Error),
    #[error("error requesting access token: {0:?}")]
    TokenRequest(ErrorResponse<TokenErrorType>),
    #[error("error requesting credentials: {0:?}")]
    CredentialRequest(ErrorResponse<CredentialErrorType>),
    #[error("generating attestation private keys failed: {0}")]
    PrivateKeyGeneration(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("missing issuance session state")]
    MissingIssuanceSessionState,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[skip_serializing_none]
pub struct ErrorResponse<T> {
    pub error: T,
    pub error_description: Option<String>,
    pub error_uri: Option<Url>,
}

pub trait ErrorStatusCode {
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug, thiserror::Error)]
pub enum JwkConversionError {
    #[error("unsupported JWK EC curve: expected P256, found {found:?}")]
    UnsupportedJwkEcCurve { found: EllipticCurve },
    #[error("unsupported JWK algorithm")]
    UnsupportedJwkAlgorithm,
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("failed to construct verifying key: {0}")]
    VerifyingKeyConstruction(#[from] signature::Error),
    #[error("missing coordinate in conversion to P256 public key")]
    MissingCoordinate,
}

pub fn jwk_from_p256(value: &VerifyingKey) -> std::result::Result<Jwk, JwkConversionError> {
    let jwk = Jwk {
        common: Default::default(),
        algorithm: jwk::AlgorithmParameters::EllipticCurve(jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: jwk::EllipticCurve::P256,
            x: BASE64_URL_SAFE_NO_PAD.encode(
                value
                    .to_encoded_point(false)
                    .x()
                    .ok_or(JwkConversionError::MissingCoordinate)?,
            ),
            y: BASE64_URL_SAFE_NO_PAD.encode(
                value
                    .to_encoded_point(false)
                    .y()
                    .ok_or(JwkConversionError::MissingCoordinate)?,
            ),
        }),
    };
    Ok(jwk)
}

pub fn jwk_to_p256(value: &Jwk) -> std::result::Result<VerifyingKey, JwkConversionError> {
    let ec_params = match value.algorithm {
        jwk::AlgorithmParameters::EllipticCurve(ref params) => Ok(params),
        _ => Err(JwkConversionError::UnsupportedJwkAlgorithm),
    }?;
    if !matches!(ec_params.curve, EllipticCurve::P256) {
        return Err(JwkConversionError::UnsupportedJwkEcCurve {
            found: ec_params.curve.clone(),
        });
    }

    let key = VerifyingKey::from_encoded_point(&EncodedPoint::from_affine_coordinates(
        BASE64_URL_SAFE_NO_PAD.decode(&ec_params.x)?.as_slice().into(),
        BASE64_URL_SAFE_NO_PAD.decode(&ec_params.y)?.as_slice().into(),
        false,
    ))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

    use crate::{jwk_from_p256, jwk_to_p256};

    #[test]
    fn jwk_p256_key_conversion() {
        let private_key = SigningKey::random(&mut OsRng);
        let verifying_key = private_key.verifying_key();
        let jwk = jwk_from_p256(verifying_key).unwrap();
        let converted = jwk_to_p256(&jwk).unwrap();

        assert_eq!(*verifying_key, converted);
    }
}
