use base64::prelude::*;
use crypto::PublicKey;
use jsonwebtoken::jwk;
pub use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::jwk::EllipticCurve;
pub use jsonwebtoken::jwk::Jwk;
pub use jsonwebtoken::jwk::JwkSet;
use p256::ecdsa::VerifyingKey;
use rsa::BigUint;
use rsa::traits::PublicKeyParts;

use crate::error::JwkConversionError;

/// Converts a [`PublicKey`] to a JWK.
pub fn jwk_from_public_key(value: &PublicKey) -> Result<Jwk, JwkConversionError> {
    Ok(Jwk {
        common: Default::default(),
        algorithm: jwk_alg_from_public_key(value)?,
    })
}

pub fn jwk_alg_from_public_key(value: &PublicKey) -> Result<jwk::AlgorithmParameters, JwkConversionError> {
    match value {
        PublicKey::P256(key) => jwk_alg_from_p256(key),
        PublicKey::P384(key) => jwk_alg_from_p384(key),
        PublicKey::P521(_) => todo!(),
        PublicKey::RSA2048(key) => Ok(jwk_alg_from_rsa(key.as_ref())),
        PublicKey::RSA4096(key) => Ok(jwk_alg_from_rsa(key.as_ref())),
    }
}

/// Builds `jsonwebtoken::jwk::AlgorithmParameters` for an EC P-256 public key.
pub fn jwk_alg_from_p256(value: &VerifyingKey) -> Result<jwk::AlgorithmParameters, JwkConversionError> {
    let point = value.to_encoded_point(false);
    Ok(jwk::AlgorithmParameters::EllipticCurve(
        jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: jwk::EllipticCurve::P256,
            x: BASE64_URL_SAFE_NO_PAD.encode(point.x().ok_or(JwkConversionError::MissingCoordinate)?),
            y: BASE64_URL_SAFE_NO_PAD.encode(point.y().ok_or(JwkConversionError::MissingCoordinate)?),
        },
    ))
}

fn jwk_alg_from_p384(value: &p384::ecdsa::VerifyingKey) -> Result<jwk::AlgorithmParameters, JwkConversionError> {
    let point = value.to_encoded_point(false);
    Ok(jwk::AlgorithmParameters::EllipticCurve(
        jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: jwk::EllipticCurve::P384,
            x: BASE64_URL_SAFE_NO_PAD.encode(point.x().ok_or(JwkConversionError::MissingCoordinate)?),
            y: BASE64_URL_SAFE_NO_PAD.encode(point.y().ok_or(JwkConversionError::MissingCoordinate)?),
        },
    ))
}

fn jwk_alg_from_rsa(value: &rsa::RsaPublicKey) -> jwk::AlgorithmParameters {
    jwk::AlgorithmParameters::RSA(jwk::RSAKeyParameters {
        key_type: jwk::RSAKeyType::RSA,
        n: BASE64_URL_SAFE_NO_PAD.encode(value.n().to_bytes_be()),
        e: BASE64_URL_SAFE_NO_PAD.encode(value.e().to_bytes_be()),
    })
}

/// Converts a JWK into a [`PublicKey`].
pub fn jwk_to_public_key(value: &Jwk) -> Result<PublicKey, JwkConversionError> {
    match &value.algorithm {
        AlgorithmParameters::EllipticCurve(params) => ec_jwk_to_public_key(params),
        AlgorithmParameters::RSA(params) => rsa_jwk_to_public_key(params),
        alg => Err(JwkConversionError::UnsupportedJwkAlgorithm(alg.to_owned())),
    }
}

fn ec_jwk_to_public_key(params: &jwk::EllipticCurveKeyParameters) -> Result<PublicKey, JwkConversionError> {
    let x = base64url_decode(&params.x)?;
    let y = base64url_decode(&params.y)?;

    match &params.curve {
        EllipticCurve::P256 => p256::ecdsa::VerifyingKey::from_encoded_point(
            &p256::EncodedPoint::from_affine_coordinates(x.as_slice().into(), y.as_slice().into(), false),
        )
        .map(PublicKey::P256)
        .map_err(JwkConversionError::InvalidEcKey),

        EllipticCurve::P384 => p384::ecdsa::VerifyingKey::from_encoded_point(
            &p384::EncodedPoint::from_affine_coordinates(x.as_slice().into(), y.as_slice().into(), false),
        )
        .map(PublicKey::P384)
        .map_err(JwkConversionError::InvalidEcKey),

        EllipticCurve::P521 => todo!(),

        curve => Err(JwkConversionError::UnsupportedJwkEcCurve(curve.to_owned())),
    }
}

fn rsa_jwk_to_public_key(params: &jwk::RSAKeyParameters) -> Result<PublicKey, JwkConversionError> {
    let n = base64url_decode(&params.n)?;
    let e = base64url_decode(&params.e)?;

    rsa::RsaPublicKey::new(BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e))
        .map_err(JwkConversionError::InvalidRsaKey)?
        .try_into()
        .map_err(JwkConversionError::UnsupportedJwkRsaKeySize)
}

fn base64url_decode(s: &str) -> Result<Vec<u8>, JwkConversionError> {
    BASE64_URL_SAFE_NO_PAD
        .decode(s)
        .map_err(JwkConversionError::Base64Error)
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[test]
    fn jwk_p256_roundtrip() {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = *signing_key.verifying_key();

        let jwk = jwk_from_public_key(&PublicKey::from(verifying_key)).unwrap();
        let public_key = jwk_to_public_key(&jwk).unwrap();

        assert_eq!(public_key, PublicKey::P256(verifying_key));
    }
}
