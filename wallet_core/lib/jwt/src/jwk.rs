use base64::prelude::*;
use jsonwebtoken::jwk;
pub use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::jwk::EllipticCurve;
pub use jsonwebtoken::jwk::Jwk;
use p256::EncodedPoint;
use p256::ecdsa::VerifyingKey;

use crate::error::JwkConversionError;

/// Converts a P-256 `VerifyingKey` to a JWK with EC P-256 parameters.
pub fn jwk_from_p256(value: &VerifyingKey) -> Result<Jwk, JwkConversionError> {
    let jwk = Jwk {
        common: Default::default(),
        algorithm: jwk_alg_from_p256(value)?,
    };

    Ok(jwk)
}

/// Builds `jsonwebtoken::jwk::AlgorithmParameters` for an EC P-256 public key.
pub fn jwk_alg_from_p256(value: &VerifyingKey) -> Result<jwk::AlgorithmParameters, JwkConversionError> {
    let point = value.to_encoded_point(false);
    let alg = jwk::AlgorithmParameters::EllipticCurve(jwk::EllipticCurveKeyParameters {
        key_type: jwk::EllipticCurveKeyType::EC,
        curve: jwk::EllipticCurve::P256,
        x: BASE64_URL_SAFE_NO_PAD.encode(point.x().ok_or(JwkConversionError::MissingCoordinate)?),
        y: BASE64_URL_SAFE_NO_PAD.encode(point.y().ok_or(JwkConversionError::MissingCoordinate)?),
    });

    Ok(alg)
}

/// Converts a JWK into a P-256 `VerifyingKey`.
///
/// Returns an error if the JWK is not an EC key or the curve is not P-256.
pub fn jwk_to_p256(value: &Jwk) -> Result<VerifyingKey, JwkConversionError> {
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
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[test]
    fn jwk_p256_key_conversion() {
        let private_key = SigningKey::random(&mut OsRng);
        let verifying_key = private_key.verifying_key();
        let jwk = jwk_from_p256(verifying_key).unwrap();
        let converted = jwk_to_p256(&jwk).unwrap();

        assert_eq!(*verifying_key, converted);
    }
}
