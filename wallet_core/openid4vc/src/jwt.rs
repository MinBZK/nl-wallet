use std::marker::PhantomData;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use p256::ecdsa::VerifyingKey;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::skip_serializing_none;
use wallet_common::{
    account::serialization::DerVerifyingKey,
    errors::Result,
    errors::{SigningError, ValidationError},
    keys::SecureEcdsaKey,
};

// JWT type, using `<T>` and `Phantomdata<T>` in the same way and for the same reason as `SignedDouble<T>`; see the
// comment there.
#[derive(Debug, Clone)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T, S: Into<String>> From<S> for Jwt<T> {
    fn from(val: S) -> Self {
        Jwt(val.into(), PhantomData)
    }
}

/// EcdsaDecodingKey is an ECDSA public key for use with the `jsonwebtoken` crate. It wraps [`DecodingKey`] and aims to
/// solve a confusing aspect of the [`DecodingKey`] API: the functions [`DecodingKey::from_ec_der()`] and
/// [`DecodingKey::from_ec_pem()`] do not really do what their name suggests, and they are not equivalent apart from
/// taking DER and PEM encodings.
///
/// There are two commonly used encodings for ECDSA public keys:
///
/// * SEC1: this encodes the two public key coordinates (i.e. numbers) `x` and `y` that an ECDSA public key consists of
/// as `04 || x || y` where `||` is bitwise concatenation. Note that this encodes just the public key, and it does not
/// include any information on the particular curve that is used, of which the public key is an element. In case of JWTs
/// this is okay, because in that case that information is transmitted elsewhere: in the `alg` field of the JWT header,
/// which in our case is `ES256` - meaning the `secp256r` curve. This encoding is what [`DecodingKey::from_ec_der()`]
/// requires as input - even though it is not in fact DER.
/// * PKIX: this uses DER to encode an identifier for the curve (`secp256r` in our case), as well as the public key
/// coordinates in SEC1 form. This is the encoding that is used in X509 certificates (hence the name). The function
/// [`DecodingKey::from_ec_pem()`] accepts this encoding, in PEM form (although it also accepts SEC1-encoded keys
/// in PEM form).
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
        value.0.into()
    }
}

impl From<&VerifyingKey> for EcdsaDecodingKey {
    fn from(value: &VerifyingKey) -> Self {
        EcdsaDecodingKey::from_sec1(value.to_encoded_point(false).as_bytes())
    }
}

impl From<VerifyingKey> for EcdsaDecodingKey {
    fn from(value: VerifyingKey) -> Self {
        EcdsaDecodingKey::from(&value)
    }
}

impl EcdsaDecodingKey {
    pub fn from_sec1(key: &[u8]) -> Self {
        DecodingKey::from_ec_der(key).into()
    }
}

/// Set of IANA registered claims by the Internet Engineering Task Force (IETF) in
/// [RFC 7519](https://tools.ietf.org/html/rfc7519#section-4.1).
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct StandardJwtClaims {
    #[serde(rename = "iss")]
    pub issuer: Option<String>,
    #[serde(rename = "sub")]
    pub subject: Option<String>,
    #[serde(rename = "aud")]
    pub audience: Option<String>,
    #[serde(rename = "exp")]
    pub expiry: Option<i64>,
    #[serde(rename = "nbf")]
    pub not_before: Option<i64>,
    #[serde(rename = "iat")]
    pub issued_at: Option<i64>,
    #[serde(rename = "jti")]
    pub jwt_id: Option<String>,
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<T> {
        let payload = jsonwebtoken::decode::<T>(&self.0, &pubkey.0, validation_options)
            .map_err(ValidationError::from)?
            .claims;

        Ok(payload)
    }

    /// Get a default header.
    fn default_header() -> Header {
        Header {
            alg: Algorithm::ES256,
            ..Default::default()
        }
    }

    pub async fn sign(payload: &T, header: &Header, privkey: &impl SecureEcdsaKey) -> Result<Jwt<T>> {
        let encoded_header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?);
        let encoded_claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload)?);
        let message = [encoded_header, encoded_claims].join(".");

        let signature = privkey
            .try_sign(message.as_bytes())
            .await
            .map_err(|err| SigningError::Ecdsa(Box::new(err)))?;
        let encoded_signature = URL_SAFE_NO_PAD.encode(signature.to_vec());

        Ok([message, encoded_signature].join(".").into())
    }
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
