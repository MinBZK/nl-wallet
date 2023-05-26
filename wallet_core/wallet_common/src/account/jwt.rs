use std::marker::PhantomData;

use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::errors::{Result, SigningError, ValidationError};

// TODO implement keyring and use kid header item for key rollover

// JWT type, using `<T>` and `Phantomdata<T>` in the same way and for the same reason as `SignedDouble<T>`; see the
// comment there.
#[derive(Debug, Clone)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T, S: Into<String>> From<S> for Jwt<T> {
    fn from(val: S) -> Self {
        Jwt(val.into(), PhantomData)
    }
}

pub trait JwtClaims {
    const SUB: &'static str;
}

/// EcdsaDecodingKey is an ECDSA public key for use with the `jsonwebtoken` crate. It wraps [`DecodingKey`] and aims to solve a confusing aspect of the [`DecodingKey`] API: the functions [`DecodingKey::from_ec_der()`] and [`DecodingKey::from_ec_pem()`] do not really do what their name suggests, and they are not equivalent apart from taking DER and PEM encodings.
///
/// There are two commonly used encodings for ECDSA public keys:
///
/// * SEC1: this encodes the two public key coordinates (i.e. numbers) `x` and `y` that an ECDSA public key consists of as `04 || x || y` where `||` is bitwise concatenation. Note that this encodes just the public key, and it does not include any information on the particular curve that is used, of which the public key is an element. In case of JWTs this is okay, because in that case that information is transmitted elsewhere: in the `alg` field of the JWT header, which in our case is `ES256` - meaning the `secp256r` curve. This encoding is what [`DecodingKey::from_ec_der()`] requires as input - even though it is not in fact DER.
/// * PKIX: this uses DER to encode an identifier for the curve (`secp256r` in our case), as well as the public key coordinates in SEC1 form. This is the encoding that is used in X509 certificates (hence the name). The function [`DecodingKey::from_ec_pem()`] accepts this encoding, in PEM form (although it also accepts SEC1-encoded keys in PEM form).
///
/// This type solves the unclarity by explicitly naming the SEC1 encoding in [`EcdsaDecodingKey::from_sec1()`] that it takes to construct it. From a `VerifyingKey` of the `ecdsa` crate, this encoding may be obtained by calling `public_key.to_encoded_point(false).as_bytes()`.
#[derive(Clone)]
pub struct EcdsaDecodingKey(DecodingKey);
impl From<DecodingKey> for EcdsaDecodingKey {
    fn from(value: DecodingKey) -> Self {
        EcdsaDecodingKey(value)
    }
}
impl EcdsaDecodingKey {
    pub fn from_sec1(key: &[u8]) -> Self {
        DecodingKey::from_ec_der(key).into()
    }
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtClaims,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: &EcdsaDecodingKey) -> Result<T> {
        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims.clear(); // we don't use `exp`, don't require it
        validation_options.sub = T::SUB.to_owned().into();

        let payload = jsonwebtoken::decode::<JwtPayload<T>>(&self.0, &pubkey.0, &validation_options)
            .map_err(ValidationError::from)?
            .claims
            .payload;
        Ok(payload)
    }

    pub fn sign(payload: &T, privkey: &[u8]) -> Result<Jwt<T>> {
        let message = jsonwebtoken::encode(
            &Header {
                alg: Algorithm::ES256,
                kid: "0".to_owned().into(),
                ..Default::default()
            },
            &JwtPayload {
                payload,
                sub: T::SUB.to_owned(),
            },
            &jsonwebtoken::EncodingKey::from_ec_der(privkey),
        )
        .map_err(SigningError::from)?;

        Ok(message.into())
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
