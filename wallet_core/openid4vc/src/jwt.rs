//! JWT functionality augmenting that of the `wallet_common` crate:
//!
//! - Conversion functions for JWK (JSON Web Key), a key format to transport (a)symmetric public/private keys such as an
//!   ECDSA public key.
//! - Bulk signing of JWTs.

use std::collections::HashSet;

use base64::prelude::*;
use base64::DecodeError;
use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use josekit::JoseError;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use nl_wallet_mdoc::holder::TrustAnchor;
use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::x509::Certificate;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::CertificateUsage;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::generator::Generator;
use wallet_common::jwt::jwk_to_p256;
use wallet_common::jwt::validations;
use wallet_common::jwt::JwkConversionError;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtCredentialClaims;
use wallet_common::jwt::JwtError;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::CredentialKeyType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtCredential<T> {
    pub(crate) private_key_id: String,
    pub(crate) key_type: CredentialKeyType,

    pub jwt: Jwt<JwtCredentialClaims<T>>,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum JwtCredentialError {
    #[error("failed to decode JWT body: {0}")]
    JoseDecoding(#[from] JoseError),
    #[error("unknown issuer: {0}")]
    #[category(critical)]
    UnknownIssuer(String),
    #[error("failed to parse trust anchor name: {0}")]
    #[category(critical)]
    TrustAnchorNameParsing(#[source] x509_parser::nom::Err<x509_parser::error::X509Error>),
    #[error("failed to verify JWT: {0}")]
    JwtVerification(#[from] jsonwebtoken::errors::Error),
    #[error("JWT error: {0}")]
    #[category(defer)]
    Jwt(#[from] JwtError),
}

impl<T> JwtCredential<T>
where
    T: DeserializeOwned,
{
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        jwt: Jwt<JwtCredentialClaims<T>>,
        pubkey: &VerifyingKey,
    ) -> Result<(Self, JwtCredentialClaims<T>), JwtCredentialError> {
        let claims = jwt.parse_and_verify(&pubkey.into(), &validations())?;

        let cred = Self {
            private_key_id,
            key_type: K::KEY_TYPE,
            jwt,
        };

        Ok((cred, claims))
    }

    #[cfg(any(feature = "test", test))]
    pub fn new_unverified<K: CredentialEcdsaKey>(private_key_id: String, jwt: Jwt<JwtCredentialClaims<T>>) -> Self {
        Self {
            private_key_id,
            key_type: K::KEY_TYPE,
            jwt,
        }
    }

    pub fn jwt_claims(&self) -> JwtCredentialClaims<T> {
        // Unwrapping is safe here because this was checked in new()
        let (_, contents) = self.jwt.dangerous_parse_unverified().unwrap();
        contents
    }

    pub(crate) fn private_key<K>(&self, key_factory: &impl KeyFactory<Key = K>) -> Result<K, JwkConversionError> {
        Ok(key_factory.generate_existing(&self.private_key_id, jwk_to_p256(&self.jwt_claims().confirmation.jwk)?))
    }
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
    use indexmap::IndexMap;
    use serde_json::json;

    use wallet_common::generator::TimeGenerator;
    use wallet_common::jwt::JwtCredentialClaims;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;

    use nl_wallet_mdoc::server_keys::KeyPair;
    use nl_wallet_mdoc::utils::x509::CertificateError;

    use crate::jwt::sign_with_certificate;
    use crate::jwt::JwtCredential;
    use crate::jwt::JwtX5cError;

    use super::verify_against_trust_anchors;

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

    #[tokio::test]
    async fn test_jwt_credential() {
        let holder_key_id = "key";
        let holder_keypair = MockRemoteEcdsaKey::new_random(holder_key_id.to_string());
        let issuer_keypair = KeyPair::generate_issuer_mock_ca().unwrap();

        // Produce a JWT with `JwtCredentialClaims` in it
        let jwt = JwtCredentialClaims::new_signed(
            holder_keypair.verifying_key(),
            issuer_keypair.private_key(),
            issuer_keypair
                .certificate()
                .common_names()
                .unwrap()
                .first()
                .unwrap()
                .to_string(),
            None,
            IndexMap::<String, serde_json::Value>::default(),
        )
        .await
        .unwrap();

        let (cred, claims) = JwtCredential::new::<MockRemoteEcdsaKey>(
            holder_key_id.to_string(),
            jwt,
            &issuer_keypair.certificate().public_key().unwrap(),
        )
        .unwrap();

        assert_eq!(cred.jwt_claims(), claims);
    }
}
