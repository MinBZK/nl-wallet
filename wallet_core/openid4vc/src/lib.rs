#![allow(dead_code)] // TODO

use base64::prelude::*;
use jsonwebtoken::jwk::{self, EllipticCurve, Jwk};
use p256::{
    ecdsa::{signature, VerifyingKey},
    EncodedPoint,
};
use serde::{Deserialize, Serialize};

pub mod authorization;
pub mod credential;
pub mod token;

pub mod dpop;
pub mod jwt;
pub mod pkce;

pub mod issuance_client;

pub const NL_WALLET_CLIENT_ID: &str = "https://example.com";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    MsoMdoc,
}

// TODO implement once we support Wallet (Instance) Attestation / Wallet Trust Anchor,
// and include it as well as the ClientAssertion itself in the `AuthorizationRequest` and `TokenRequest`
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub enum ClientAssertionType {
//     #[serde(rename = "urn:ietf:params:oauth:client-assertion-type:jwt-client-attestation")]
//     JwtClientAttestation,
// }

// TODO
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported JWT algorithm: expected {expected}, found {found}")]
    UnsupportedJwtAlgorithm { expected: String, found: String },
    #[error("unsupported JWK EC curve: expected P256, found {found:?}")]
    UnsupportedJwkEcCurve { found: EllipticCurve },
    #[error("unsupported JWK algorithm")]
    UnsupportedJwkAlgorithm,
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("failed to construct verifying key: {0}")]
    VerifyingKeyConstruction(#[from] signature::Error),
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
    #[error("incorrect nonce")]
    IncorrectNonce,
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn jwk_from_p256(value: &VerifyingKey) -> Result<Jwk> {
    let jwk = Jwk {
        common: Default::default(),
        algorithm: jwk::AlgorithmParameters::EllipticCurve(jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: jwk::EllipticCurve::P256,
            x: BASE64_URL_SAFE_NO_PAD.encode(value.to_encoded_point(false).x().unwrap()),
            y: BASE64_URL_SAFE_NO_PAD.encode(value.to_encoded_point(false).y().unwrap()),
        }),
    };
    Ok(jwk)
}

pub fn jwk_to_p256(value: &Jwk) -> Result<VerifyingKey> {
    let ec_params = match value.algorithm {
        jwk::AlgorithmParameters::EllipticCurve(ref params) => Ok(params),
        _ => Err(Error::UnsupportedJwkAlgorithm),
    }?;
    if !matches!(ec_params.curve, EllipticCurve::P256) {
        return Err(Error::UnsupportedJwkEcCurve {
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
