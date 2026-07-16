use base64::prelude::*;
use crypto::PublicKey;
use ecdsa::elliptic_curve::AffinePoint;
use ecdsa::elliptic_curve::CurveArithmetic;
use ecdsa::elliptic_curve::FieldBytes;
use ecdsa::elliptic_curve::FieldBytesSize;
use ecdsa::elliptic_curve::sec1;
use jsonwebtoken::jwk;
pub use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::jwk::EllipticCurve;
pub use jsonwebtoken::jwk::Jwk;
pub use jsonwebtoken::jwk::JwkSet;
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
        PublicKey::P256(key) => jwk_alg_from_ecdsa(key),
        PublicKey::P384(key) => jwk_alg_from_ecdsa(key),
        PublicKey::P521(key) => jwk_alg_from_ecdsa(key),
        PublicKey::RSA2048(key) => Ok(jwk_alg_from_rsa(key.as_ref())),
        PublicKey::RSA4096(key) => Ok(jwk_alg_from_rsa(key.as_ref())),
    }
}

trait EcdsaCurveInfo {
    fn curve() -> jwk::EllipticCurve;
}

impl EcdsaCurveInfo for p256::NistP256 {
    fn curve() -> jwk::EllipticCurve {
        jwk::EllipticCurve::P256
    }
}

impl EcdsaCurveInfo for p384::NistP384 {
    fn curve() -> jwk::EllipticCurve {
        jwk::EllipticCurve::P384
    }
}

impl EcdsaCurveInfo for p521::NistP521 {
    fn curve() -> jwk::EllipticCurve {
        jwk::EllipticCurve::P521
    }
}

fn jwk_alg_from_ecdsa<C: ecdsa::EcdsaCurve + CurveArithmetic + EcdsaCurveInfo>(
    value: &ecdsa::VerifyingKey<C>,
) -> Result<jwk::AlgorithmParameters, JwkConversionError>
where
    AffinePoint<C>: sec1::FromSec1Point<C> + sec1::ToSec1Point<C>,
    FieldBytesSize<C>: sec1::ModulusSize,
{
    let point = value.to_sec1_point(false);
    Ok(jwk::AlgorithmParameters::EllipticCurve(
        jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: C::curve(),
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
        EllipticCurve::P256 => coordinates_to_verifying_key(&x, &y).map(PublicKey::P256),
        EllipticCurve::P384 => coordinates_to_verifying_key(&x, &y).map(PublicKey::P384),
        EllipticCurve::P521 => coordinates_to_verifying_key(&x, &y).map(PublicKey::P521),
        curve => Err(JwkConversionError::UnsupportedJwkEcCurve(curve.to_owned())),
    }
}

fn coordinates_to_verifying_key<C: ecdsa::EcdsaCurve + CurveArithmetic>(
    x: &[u8],
    y: &[u8],
) -> Result<ecdsa::VerifyingKey<C>, JwkConversionError>
where
    AffinePoint<C>: sec1::FromSec1Point<C> + sec1::ToSec1Point<C>,
    FieldBytesSize<C>: sec1::ModulusSize,
{
    let x: &FieldBytes<C> = x.try_into().map_err(JwkConversionError::InvalidCoordinate)?;
    let y: &FieldBytes<C> = y.try_into().map_err(JwkConversionError::InvalidCoordinate)?;
    ecdsa::VerifyingKey::<C>::from_sec1_point(&ecdsa::Sec1Point::<C>::from_affine_coordinates(x, y, false))
        .map_err(JwkConversionError::InvalidEcKey)
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
    use p256::elliptic_curve::Generate;

    use super::*;

    #[test]
    fn jwk_p256_roundtrip() {
        let signing_key = SigningKey::generate();
        let verifying_key = *signing_key.verifying_key();

        let jwk = jwk_from_public_key(&PublicKey::from(verifying_key)).unwrap();
        let public_key = jwk_to_public_key(&jwk).unwrap();

        assert_eq!(public_key, PublicKey::P256(verifying_key));
    }
}
