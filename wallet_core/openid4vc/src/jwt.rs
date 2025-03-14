//! JWT functionality augmenting that of the `wallet_common` crate:
//!
//! - Conversion functions for JWK (JSON Web Key), a key format to transport (a)symmetric public/private keys such as an
//!   ECDSA public key.
//! - Bulk signing of JWTs.

use std::collections::HashSet;
use std::collections::VecDeque;

use base64::prelude::*;
use base64::DecodeError;
use chrono::DateTime;
use chrono::Utc;
use josekit::JoseError;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use mdoc::server_keys::KeyPair;
use mdoc::utils::x509::BorrowingCertificate;
use mdoc::utils::x509::CertificateError;
use mdoc::utils::x509::CertificateUsage;
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
use wallet_common::keys::EcdsaKey;

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
    #[error("error parsing certificate: {0}")]
    CertificateParsing(#[source] CertificateError),
    #[error("error verifying certificate: {0}")]
    CertificateValidation(#[source] CertificateError),
}

/// Verify the JWS against the provided trust anchors, using the X.509 certificate(s) present in the `x5c` JWT header.
pub fn verify_against_trust_anchors<T: DeserializeOwned, A: ToString>(
    jwt: &Jwt<T>,
    audience: &[A],
    trust_anchors: &[TrustAnchor],
    time: &impl Generator<DateTime<Utc>>,
) -> Result<(T, BorrowingCertificate), JwtX5cError> {
    let header = jsonwebtoken::decode_header(&jwt.0).map_err(JwtError::Validation)?;
    let mut certs = header
        .x5c
        .ok_or(JwtX5cError::MissingCertificates)?
        .into_iter()
        .map(|cert_base64| {
            let cert = CertificateDer::from(
                BASE64_STANDARD
                    .decode(cert_base64)
                    .map_err(JwtX5cError::CertificateBase64)?,
            );
            Ok(cert)
        })
        .collect::<Result<VecDeque<_>, JwtX5cError>>()?;

    // Verify the certificate chain against the trust anchors.
    let leaf_cert =
        BorrowingCertificate::from_certificate_der(certs.pop_front().ok_or(JwtX5cError::MissingCertificates)?)
            .map_err(JwtX5cError::CertificateParsing)?;
    // The `VecDeque` containing the certificates will be contiguous at this point, so the second value is empty.
    let (intermediates, _) = certs.as_slices();
    leaf_cert
        .verify(CertificateUsage::ReaderAuth, intermediates, time, trust_anchors)
        .map_err(JwtX5cError::CertificateValidation)?;

    // The leaf certificate is trusted, we can now use its public key to verify the JWS.
    let pubkey = leaf_cert.public_key();

    let validation_options = {
        let mut validation = Validation::new(Algorithm::ES256);

        validation.required_spec_claims = HashSet::default();
        validation.set_audience(audience);

        validation
    };

    let payload = jwt.parse_and_verify(&pubkey.into(), &validation_options)?;

    Ok((payload, leaf_cert))
}

/// Sign a payload into a JWS, and put the certificate of the provided keypair in the `x5c` JWT header.
/// The resulting JWS can be verified using [`verify_against_trust_anchors()`].
pub async fn sign_with_certificate<T: Serialize, K: EcdsaKey>(
    payload: &T,
    keypair: &KeyPair<K>,
) -> Result<Jwt<T>, JwtError> {
    // The `x5c` header supports certificate chains, but ISO 18013-5 doesn't: it requires that issuer
    // and RP certificates are signed directly by the trust anchor. So we don't support certificate chains
    // here (yet).
    let certs = vec![BASE64_STANDARD.encode(keypair.certificate().as_ref())];

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
    use base64::prelude::*;
    use indexmap::IndexMap;
    use jsonwebtoken::Header;
    use serde_json::json;

    use mdoc::server_keys::generate::Ca;
    use mdoc::utils::x509::CertificateConfiguration;
    use mdoc::utils::x509::CertificateError;
    use mdoc::utils::x509::CertificateUsage;
    use wallet_common::generator::TimeGenerator;
    use wallet_common::jwt::Jwt;
    use wallet_common::jwt::JwtCredentialClaims;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;

    use crate::jwt::sign_with_certificate;
    use crate::jwt::JwtCredential;
    use crate::jwt::JwtX5cError;

    use super::verify_against_trust_anchors;

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = sign_with_certificate(&payload, &keypair).await.unwrap();

        let audience: &[String] = &[];
        let (deserialized, leaf_cert) =
            verify_against_trust_anchors(&jwt, audience, &[ca.to_trust_anchor()], &TimeGenerator).unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(leaf_cert, *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert_intermediates() {
        // Generate a chain of certificates
        let ca = Ca::generate_with_intermediate_count("myca", CertificateConfiguration::default(), 3).unwrap();
        let intermediate1 = ca
            .generate_intermediate(
                "myintermediate1",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate2 = intermediate1
            .generate_intermediate(
                "myintermediate2",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate3 = intermediate2
            .generate_intermediate(
                "myintermediate3",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let keypair = intermediate3.generate_reader_mock(None).unwrap();

        // Construct a JWT with the `x5c` field containing the X.509 certificates
        // of the leaf certificate and the intermediates, in reverse order.
        let payload = json!({"hello": "world"});
        let certs = vec![
            keypair.certificate().as_ref(),
            intermediate3.as_certificate_der().as_ref(),
            intermediate2.as_certificate_der().as_ref(),
            intermediate1.as_certificate_der().as_ref(),
        ]
        .into_iter()
        .map(|der| BASE64_STANDARD.encode(der))
        .collect();

        let jwt = Jwt::sign(
            &payload,
            &Header {
                alg: jsonwebtoken::Algorithm::ES256,
                x5c: Some(certs),
                ..Default::default()
            },
            keypair.private_key(),
        )
        .await
        .unwrap();

        // Verifying this JWT against the CA's trust anchor should succeed.
        let audience: &[String] = &[];
        let (deserialized, leaf_cert) =
            verify_against_trust_anchors(&jwt, audience, &[ca.to_trust_anchor()], &TimeGenerator).unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(leaf_cert, *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_wrong_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = sign_with_certificate(&payload, &keypair).await.unwrap();

        let other_ca = Ca::generate("myca", Default::default()).unwrap();

        let audience: &[String] = &[];
        let err =
            verify_against_trust_anchors(&jwt, audience, &[other_ca.to_trust_anchor()], &TimeGenerator).unwrap_err();
        assert_matches!(
            err,
            JwtX5cError::CertificateValidation(CertificateError::Verification(_))
        );
    }

    #[tokio::test]
    async fn test_jwt_credential() {
        let holder_key_id = "key";
        let holder_keypair = MockRemoteEcdsaKey::new_random(holder_key_id.to_string());
        let issuer_keypair = Ca::generate_issuer_mock_ca()
            .unwrap()
            .generate_issuer_mock(None)
            .unwrap();

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
            issuer_keypair.certificate().public_key(),
        )
        .unwrap();

        assert_eq!(cred.jwt_claims(), claims);
    }
}
