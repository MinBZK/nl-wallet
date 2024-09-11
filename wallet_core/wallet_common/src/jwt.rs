use std::{marker::PhantomData, str::FromStr, sync::LazyLock};

use base64::prelude::*;
use itertools::Itertools;
use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use p256::ecdsa::VerifyingKey;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use x509_parser::{
    der_parser::{asn1_rs::BitString, Oid},
    prelude::FromDer,
    x509::AlgorithmIdentifier,
};

use error_category::ErrorCategory;

use crate::{
    account::serialization::DerVerifyingKey,
    keys::{EcdsaKey, SecureEcdsaKey},
};

/// JWT type, generic over its contents.
///
/// This wrapper of the `jsonwebtoken` crate echoes the following aspect of `jsonwebtoken`:
/// Validating one of the a standard fields during verification of the JWT using [`Validation`] does NOT automatically
/// result in enforcement that the field is present. For example, if validation of `exp` is turned on then JWTs without
/// an `exp` fields are still accepted (but not JWTs having an `exp` from the past).
///
/// Presence of the field may be enforced using [`Validation::required_spec_claims`] and/or by including it
/// explicitly as a field in the (de)serialized type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T, S: Into<String>> From<S> for Jwt<T> {
    fn from(val: S) -> Self {
        Jwt(val.into(), PhantomData)
    }
}

pub type Result<T, E = JwtError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum JwtError {
    #[error("JSON parsing error: {0}")]
    #[category(pd)]
    JsonParsing(#[from] serde_json::Error),
    #[error("error validating JWT: {0}")]
    #[category(critical)]
    Validation(#[source] jsonwebtoken::errors::Error),
    #[error("error signing JWT: {0}")]
    #[category(critical)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("trust anchor key of unexpected format: {0}")]
    #[category(critical)]
    TrustAnchorKeyFormat(String),
    #[error("failed to parse trust anchor algorithm: {0}")]
    #[category(critical)]
    TrustAnchorAlgorithmParsing(#[source] x509_parser::nom::Err<x509_parser::error::X509Error>),
    #[error("failed to parse trust anchor key: {0}")]
    #[category(critical)]
    TrustAnchorKeyParsing(#[from] x509_parser::nom::Err<x509_parser::der_parser::error::Error>),
    #[error("unexpected amount of parts in JWT credential: expected 3, found {0}")]
    #[category(critical)]
    Parts(usize),
    #[error("failed to decode Base64: {0}")]
    #[category(pd)]
    Base64(#[from] base64::DecodeError),
}

pub trait JwtSubject {
    const SUB: &'static str;
}

/// EcdsaDecodingKey is an ECDSA public key for use with the `jsonwebtoken` crate. It wraps [`DecodingKey`] and aims to
/// solve a confusing aspect of the [`DecodingKey`] API: the functions [`DecodingKey::from_ec_der()`] and
/// [`DecodingKey::from_ec_pem()`] do not really do what their name suggests, and they are not equivalent apart from
/// taking DER and PEM encodings.
///
/// There are two commonly used encodings for ECDSA public keys:
///
/// * SEC1: this encodes the two public key coordinates (i.e. numbers) `x` and `y` that an ECDSA public key consists of
///   as `04 || x || y` where `||` is bitwise concatenation. Note that this encodes just the public key, and it does not
///   include any information on the particular curve that is used, of which the public key is an element. In case of
///   JWTs this is okay, because in that case that information is transmitted elsewhere: in the `alg` field of the JWT
///   header, which in our case is `ES256` - meaning the `secp256r` curve. This encoding is what
///   [`DecodingKey::from_ec_der()`] requires as input - even though it is not in fact DER.
/// * PKIX: this uses DER to encode an identifier for the curve (`secp256r` in our case), as well as the public key
///   coordinates in SEC1 form. This is the encoding that is used in X509 certificates (hence the name). The function
///   [`DecodingKey::from_ec_pem()`] accepts this encoding, in PEM form (although it also accepts SEC1-encoded keys in
///   PEM form).
///
/// This type solves the unclarity by explicitly naming the SEC1 encoding in [`EcdsaDecodingKey::from_sec1()`] that it
/// takes to construct it. From a `VerifyingKey` of the `ecdsa` crate, this encoding may be obtained by calling
/// `public_key.to_encoded_point(false).as_bytes()`.
#[derive(Clone)]
pub struct EcdsaDecodingKey(pub DecodingKey);

impl From<DecodingKey> for EcdsaDecodingKey {
    fn from(value: DecodingKey) -> Self {
        EcdsaDecodingKey(value)
    }
}

impl From<DerVerifyingKey> for EcdsaDecodingKey {
    fn from(value: DerVerifyingKey) -> Self {
        (&value.0).into()
    }
}

impl From<&VerifyingKey> for EcdsaDecodingKey {
    fn from(value: &VerifyingKey) -> Self {
        EcdsaDecodingKey::from_sec1(value.to_encoded_point(false).as_bytes())
    }
}

impl EcdsaDecodingKey {
    pub fn from_sec1(key: &[u8]) -> Self {
        DecodingKey::from_ec_der(key).into()
    }
}

/// The OID of Elliptic Curve public keys.
static OID_EC_PUBKEY: LazyLock<Oid<'static>> = LazyLock::new(|| Oid::from_str("1.2.840.10045.2.1").unwrap());

impl<T> Jwt<T>
where
    T: DeserializeOwned,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<T> {
        let payload = jsonwebtoken::decode::<T>(&self.0, &pubkey.0, validation_options)
            .map_err(JwtError::Validation)?
            .claims;

        Ok(payload)
    }

    /// Verify a JWT against the `subjectPublicKeyInfo` of a trust anchor.
    pub fn verify_against_spki(&self, spki: &[u8]) -> Result<T> {
        let (key_bytes, algorithm) =
            AlgorithmIdentifier::from_der(spki).map_err(JwtError::TrustAnchorAlgorithmParsing)?;
        if algorithm.algorithm != *OID_EC_PUBKEY {
            return Err(JwtError::TrustAnchorKeyFormat(algorithm.oid().to_id_string()));
        }

        let (_, key_bytes) = BitString::from_der(key_bytes)?;
        let key = DecodingKey::from_ec_der(&key_bytes.data); // this is actually SEC1, not DER

        let claims = jsonwebtoken::decode(&self.0, &key, &validations())
            .map_err(JwtError::Validation)?
            .claims;

        Ok(claims)
    }

    pub fn dangerous_parse_unverified(&self) -> Result<(Header, T)> {
        let parts = self.0.split('.').collect_vec();
        if parts.len() != 3 {
            return Err(JwtError::Parts(parts.len()));
        }

        let header: Header = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[0])?)?;
        let body: T = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[1])?)?;

        Ok((header, body))
    }
}

impl<T> Jwt<T>
where
    T: Serialize,
{
    pub async fn sign(payload: &T, header: &Header, privkey: &impl EcdsaKey) -> Result<Jwt<T>> {
        let encoded_header = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?);
        let encoded_claims = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload)?);
        let message = [encoded_header, encoded_claims].join(".");

        let signature = privkey
            .try_sign(message.as_bytes())
            .await
            .map_err(|err| JwtError::Signing(Box::new(err)))?;
        let encoded_signature = BASE64_URL_SAFE_NO_PAD.encode(signature.to_vec());

        Ok([message, encoded_signature].join(".").into())
    }
}

pub fn validations() -> Validation {
    let mut validation_options = Validation::new(Algorithm::ES256);

    validation_options.required_spec_claims.clear(); // we generally don't use `exp`, don't require it
    validation_options.leeway = 60;

    validation_options
}

pub fn header() -> Header {
    Header {
        alg: Algorithm::ES256,
        ..Default::default()
    }
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtSubject,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify_with_sub(&self, pubkey: &EcdsaDecodingKey) -> Result<T> {
        let mut validation_options = validations();
        validation_options.required_spec_claims.insert("sub".to_string());
        self.parse_and_verify(pubkey, &validation_options)
    }

    pub async fn sign_with_sub(payload: &T, privkey: &impl SecureEcdsaKey) -> Result<Jwt<T>> {
        let header = &Header {
            alg: Algorithm::ES256,
            kid: "0".to_owned().into(),
            ..Default::default()
        };
        let claims = &JwtPayload {
            payload,
            sub: T::SUB.to_owned(),
        };

        let jwt = Jwt::sign(claims, header, privkey).await?.0;
        Ok(jwt.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtPayload<T> {
    #[serde(flatten)]
    payload: T,
    sub: String,
}

impl<T> Serialize for Jwt<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        String::serialize(&self.0, serializer)
    }
}
impl<'de, T> Deserialize<'de> for Jwt<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer).map(Jwt::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ToyMessage {
        number: u8,
        string: String,
    }

    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                number: 42,
                string: "Hello, world!".to_string(),
            }
        }
    }

    impl JwtSubject for ToyMessage {
        const SUB: &'static str = "toy_message";
    }

    #[tokio::test]
    async fn test_sign_and_verify_with_sub() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let jwt = Jwt::sign_with_sub(&t, &private_key).await.unwrap();

        // the JWT has a `sub` with the expected value
        let jwt_message: HashMap<String, serde_json::Value> = part(1, &jwt.0);
        assert_eq!(
            *jwt_message.get("sub").unwrap(),
            serde_json::Value::String(ToyMessage::SUB.to_string())
        );

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify_with_sub(&private_key.verifying_key().into())
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sign_and_verify() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let header = header();
        let jwt = Jwt::sign(&t, &header, &private_key).await.unwrap();

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &validations())
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sub_required() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        // create a new JWT without a `sub`
        let header = header();
        let jwt = Jwt::sign(&t, &header, &private_key).await.unwrap();
        let jwt_message: HashMap<String, serde_json::Value> = part(1, &jwt.0);
        assert!(!jwt_message.contains_key("sub"));

        // verification fails because `sub` is required
        jwt.parse_and_verify_with_sub(&private_key.verifying_key().into())
            .unwrap_err();

        // we can parse and verify the JWT if we don't require the `sub` field to be present
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &validations())
            .unwrap();

        assert_eq!(t, parsed);
    }

    /// Decode and deserialize the specified part of the JWT.
    fn part<T: DeserializeOwned>(i: u8, jwt: &str) -> T {
        let bts = BASE64_URL_SAFE_NO_PAD
            .decode(jwt.split('.').take((i + 1) as usize).last().unwrap())
            .unwrap();
        serde_json::from_slice(&bts).unwrap()
    }
}
