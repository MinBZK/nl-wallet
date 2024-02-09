//! JWT functionality augmenting that of the `wallet_common` crate:
//!
//! - Conversion functions for JWK (JSON Web Key), a key format to transport (a)symmetric public/private keys
//!   such as an ECDSA public key.
//! - Bulk signing of JWTs.

use std::collections::HashMap;

use base64::prelude::*;
use itertools::Itertools;
use jsonwebtoken::{
    jwk::{self, EllipticCurve, Jwk},
    Algorithm, Header,
};
use nl_wallet_mdoc::utils::keys::{KeyFactory, MdocEcdsaKey};
use p256::{
    ecdsa::{signature, VerifyingKey},
    EncodedPoint,
};
use serde::Serialize;
use wallet_common::{
    jwt::{Jwt, JwtError},
    keys::EcdsaKey,
};

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
    #[error("failed to get public key: {0}")]
    VerifyingKeyFromPrivateKey(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub fn jwk_from_p256(value: &VerifyingKey) -> Result<Jwk, JwkConversionError> {
    let point = value.to_encoded_point(false);
    let jwk = Jwk {
        common: Default::default(),
        algorithm: jwk::AlgorithmParameters::EllipticCurve(jwk::EllipticCurveKeyParameters {
            key_type: jwk::EllipticCurveKeyType::EC,
            curve: jwk::EllipticCurve::P256,
            x: BASE64_URL_SAFE_NO_PAD.encode(point.x().ok_or(JwkConversionError::MissingCoordinate)?),
            y: BASE64_URL_SAFE_NO_PAD.encode(point.y().ok_or(JwkConversionError::MissingCoordinate)?),
        }),
    };
    Ok(jwk)
}

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

pub async fn jwk_jwt_header(typ: &str, key: &impl EcdsaKey) -> Result<Header, JwkConversionError> {
    let header = Header {
        typ: Some(typ.to_string()),
        alg: Algorithm::ES256,
        jwk: Some(jwk_from_p256(
            &key.verifying_key()
                .await
                .map_err(|e| JwkConversionError::VerifyingKeyFromPrivateKey(e.into()))?,
        )?),
        ..Default::default()
    };
    Ok(header)
}

/// Bulk-sign the keys and JWT payloads into JWTs.
pub async fn sign_jwts<T: Serialize, K: MdocEcdsaKey>(
    keys_and_messages: Vec<(K, (T, jsonwebtoken::Header))>,
    key_factory: &impl KeyFactory<Key = K>,
) -> Result<Vec<(K, Jwt<T>)>, JwtError> {
    let (keys, to_sign): (Vec<_>, Vec<_>) = keys_and_messages.into_iter().unzip();

    // Construct a Vec containing the strings to be signed with the private keys, i.e. schematically "header.body"
    let messages = to_sign
        .iter()
        .map(|(message, header)| {
            Ok([
                BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?),
                BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(message)?),
            ]
            .join("."))
        })
        .collect::<Result<Vec<_>, JwtError>>()?;

    // Associate the messages to the keys with which they are to be signed, for below
    let keys_messages_map: HashMap<_, _> = keys
        .iter()
        .zip(&messages)
        .map(|(key, msg)| (key.identifier().to_string(), msg.clone()))
        .collect();

    // Have the WP sign our messages. It returns key-signature pairs in a random order.
    let keys_and_sigs = key_factory
        .sign_with_existing_keys(
            messages
                .into_iter()
                .map(|msg| msg.into_bytes())
                .zip(keys.into_iter().map(|key| vec![key]))
                .collect_vec(),
        )
        .await
        .map_err(|err| JwtError::Signing(Box::new(err)))?;

    // For each received key-signature pair, we use the key to lookup the appropriate message
    // from the map constructed above and create the JWT.
    let jwts = keys_and_sigs
        .into_iter()
        .map(|(key, sig)| {
            // The WP will respond only with the keys we fed it above, so we can unwrap
            let msg = keys_messages_map.get(&key.identifier().to_string()).unwrap().clone();
            let jwt = [msg, BASE64_URL_SAFE_NO_PAD.encode(sig.to_vec())].join(".").into();
            (key, jwt)
        })
        .collect();

    Ok(jwts)
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
    use serde::{Deserialize, Serialize};

    use nl_wallet_mdoc::{
        mock::SoftwareKeyFactory,
        utils::keys::{KeyFactory, MdocEcdsaKey},
    };
    use wallet_common::jwt::{validations, EcdsaDecodingKey};

    use super::{jwk_from_p256, jwk_to_p256};

    #[test]
    fn jwk_p256_key_conversion() {
        let private_key = SigningKey::random(&mut OsRng);
        let verifying_key = private_key.verifying_key();
        let jwk = jwk_from_p256(verifying_key).unwrap();
        let converted = jwk_to_p256(&jwk).unwrap();

        assert_eq!(*verifying_key, converted);
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ToyMessage {
        count: usize,
    }

    #[tokio::test]
    async fn test_sign_jwts() {
        bulk_jwt_sign(&SoftwareKeyFactory::default()).await
    }

    fn json_header() -> jsonwebtoken::Header {
        jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::ES256,
            ..Default::default()
        }
    }

    pub async fn bulk_jwt_sign<K: MdocEcdsaKey>(key_factory: &impl KeyFactory<Key = K>) {
        // Generate keys to sign with and messages to sign
        let keys = key_factory.generate_new_multiple(4).await.unwrap();
        let keys_and_messages = keys
            .into_iter()
            .enumerate()
            .map(|(count, key)| (key, (ToyMessage { count }, json_header())))
            .collect();

        let jwts = super::sign_jwts(keys_and_messages, key_factory).await.unwrap();

        // Verify JWTs
        futures::stream::iter(jwts) // convert to stream which supports async for_each closures
            .for_each(|(key, jwt)| async move {
                jwt.parse_and_verify(
                    &EcdsaDecodingKey::from(&key.verifying_key().await.unwrap()),
                    &validations(),
                )
                .unwrap();
            })
            .await;
    }
}
