//! JWT functionality augmenting that of the `wallet_common` crate:
//!
//! - Conversion functions for JWK (JSON Web Key), a key format to transport (a)symmetric public/private keys such as an
//!   ECDSA public key.
//! - Bulk signing of JWTs.

use std::collections::HashSet;

use base64::{prelude::*, DecodeError};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use jsonwebtoken::{
    jwk::{self, EllipticCurve, Jwk},
    Algorithm, Header, Validation,
};
use p256::{
    ecdsa::{signature, VerifyingKey},
    EncodedPoint,
};
use serde::{de::DeserializeOwned, Serialize};

use error_category::ErrorCategory;
use nl_wallet_mdoc::{
    holder::TrustAnchor,
    server_keys::KeyPair,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        x509::{Certificate, CertificateError, CertificateUsage},
    },
};
use wallet_common::{
    account::serialization::DerVerifyingKey,
    generator::Generator,
    jwt::{Jwt, JwtError},
    keys::EcdsaKey,
};

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum JwkConversionError {
    #[error("unsupported JWK EC curve: expected P256, found {found:?}")]
    #[category(critical)]
    UnsupportedJwkEcCurve { found: EllipticCurve },
    #[error("unsupported JWK algorithm")]
    #[category(critical)]
    UnsupportedJwkAlgorithm,
    #[error("base64 decoding failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("failed to construct verifying key: {0}")]
    VerifyingKeyConstruction(#[from] signature::Error),
    #[error("missing coordinate in conversion to P256 public key")]
    #[category(critical)]
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

    // Have the WP sign our messages.
    let signatures = key_factory
        .sign_multiple_with_existing_keys(
            messages
                .iter()
                .map(|msg| msg.clone().into_bytes())
                .zip(keys.iter().map(|key| vec![key]))
                .collect_vec(),
        )
        .await
        .map_err(|err| JwtError::Signing(Box::new(err)))?;

    // For each received key-signature pair, we use the key to lookup the appropriate message
    // from the map constructed above and create the JWT.
    let jwts = signatures
        .into_iter()
        .zip(keys)
        .zip(messages)
        .map(|((sigs, key), msg)| {
            // The WP will respond only with the keys we fed it above, so we can unwrap
            let jwt = [msg, BASE64_URL_SAFE_NO_PAD.encode(sigs.first().unwrap().to_vec())]
                .join(".")
                .into();
            (key, jwt)
        })
        .collect();

    Ok(jwts)
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum JwtX5cError {
    #[error("error validating JWT: {0}")]
    Jwt(#[from] JwtError),
    #[error("missing X.509 certificate(s) in JWT header to validate JWT against")]
    #[category(critical)]
    MissingCertificates,
    #[error("error base64-decoding certificate: {0}")]
    #[category(critical)]
    CertificateBase64(#[source] DecodeError),
    #[error("error verifying certificate: {0}")]
    CertificateValidation(#[source] CertificateError),
    #[error("error parsing public key from certificate: {0}")]
    CertificatePublicKey(#[source] CertificateError),
}

/// Verify the JWS against the provided trust anchors, using the X.509 certificate(s) present in the `x5c` JWT header.
pub fn verify_against_trust_anchors<T: DeserializeOwned, A: ToString>(
    jwt: &Jwt<T>,
    audience: &[A],
    trust_anchors: &[TrustAnchor],
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(T, Certificate), JwtX5cError> {
    let header = jsonwebtoken::decode_header(&jwt.0).map_err(JwtError::Validation)?;
    let mut certs = header
        .x5c
        .ok_or(JwtX5cError::MissingCertificates)?
        .into_iter()
        .map(|cert_base64| {
            let cert: Certificate = BASE64_STANDARD
                .decode(cert_base64)
                .map_err(JwtX5cError::CertificateBase64)?
                .into();
            Ok(cert)
        })
        .collect::<Result<Vec<_>, JwtX5cError>>()?;

    // Verify the certificate chain against the trust anchors.
    let leaf_cert = certs.pop().ok_or(JwtX5cError::MissingCertificates)?;
    let intermediate_certs = certs.iter().map(|cert| cert.as_bytes()).collect_vec();
    leaf_cert
        .verify(CertificateUsage::ReaderAuth, &intermediate_certs, time, trust_anchors)
        .map_err(JwtX5cError::CertificateValidation)?;

    // The leaf certificate is trusted, we can now use its public key to verify the JWS.
    let pubkey = leaf_cert.public_key().map_err(JwtX5cError::CertificatePublicKey)?;

    let validation_options = {
        let mut validation = Validation::new(Algorithm::ES256);

        validation.required_spec_claims = HashSet::default();
        validation.set_audience(audience);

        validation
    };

    let payload = jwt.parse_and_verify(&DerVerifyingKey(pubkey).into(), &validation_options)?;

    Ok((payload, leaf_cert))
}

/// Sign a payload into a JWS, and put the certificate of the provided keypair in the `x5c` JWT header.
/// The resulting JWS can be verified using [`verify_against_trust_anchors()`].
pub async fn sign_with_certificate<T: Serialize>(payload: &T, keypair: &KeyPair) -> Result<Jwt<T>, JwtError> {
    // The `x5c` header supports certificate chains, but ISO 18013-5 doesn't: it requires that issuer
    // and RP certificates are signed directly by the trust anchor. So we don't support certificate chains
    // here (yet).
    let certs = vec![BASE64_STANDARD.encode(keypair.certificate().as_bytes())];

    let jwt = Jwt::sign(
        payload,
        &Header {
            alg: jsonwebtoken::Algorithm::ES256,
            x5c: Some(certs),
            ..Default::default()
        },
        keypair.private_key(),
    )
    .await?;

    Ok(jwt)
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use futures::StreamExt;
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use nl_wallet_mdoc::{
        server_keys::KeyPair,
        software_key_factory::SoftwareKeyFactory,
        utils::{
            keys::{KeyFactory, MdocEcdsaKey},
            x509::CertificateError,
        },
    };
    use wallet_common::{
        generator::TimeGenerator,
        jwt::{validations, EcdsaDecodingKey},
    };

    use crate::jwt::{sign_with_certificate, JwtX5cError};

    use super::{jwk_from_p256, jwk_to_p256, verify_against_trust_anchors};

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = sign_with_certificate(&payload, &keypair).await.unwrap();

        let audience: &[String] = &[];
        let (deserialized, leaf_cert) =
            verify_against_trust_anchors(&jwt, audience, &[ca.certificate().try_into().unwrap()], &TimeGenerator)
                .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(leaf_cert, *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_wrong_cert() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = sign_with_certificate(&payload, &keypair).await.unwrap();

        let other_ca = KeyPair::generate_ca("myca", Default::default()).unwrap();

        let audience: &[String] = &[];
        let err = verify_against_trust_anchors(
            &jwt,
            audience,
            &[other_ca.certificate().try_into().unwrap()],
            &TimeGenerator,
        )
        .unwrap_err();
        assert_matches!(
            err,
            JwtX5cError::CertificateValidation(CertificateError::Verification(_))
        );
    }

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
        bulk_jwt_sign(&SoftwareKeyFactory::default()).await;
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
