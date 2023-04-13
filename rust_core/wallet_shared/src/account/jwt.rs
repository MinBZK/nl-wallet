use std::marker::PhantomData;

use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// TODO implement keyring and use kid header item for key rollover

// JWT type, using `<T>` and `Phantomdata<T>` in the same way and for the same reason as `SignedDouble<T>`; see the
// comment there.
#[derive(Debug)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T, S: Into<String>> From<S> for Jwt<T> {
    fn from(val: S) -> Self {
        Jwt(val.into(), PhantomData)
    }
}

pub trait JwtClaims {
    const SUB: &'static str;
}

pub struct EcdsaDecodingKey(DecodingKey);
impl From<DecodingKey> for EcdsaDecodingKey {
    fn from(value: DecodingKey) -> Self {
        EcdsaDecodingKey(value)
    }
}

impl EcdsaDecodingKey {
    pub fn from_pkix(key: &[u8]) -> Result<Self> {
        // `from_ec_der()` accepts exclusively a bare SEC1 encoded key (which is in fact a custom encoding and not DER at all).
        // But `from_ec_pem()` also accepts ASN.1 DER-encoded PKIX keys.
        Ok(DecodingKey::from_ec_pem(der_to_pem(key, "PUBLIC KEY")?.as_bytes())
            .map_err(anyhow::Error::msg)?
            .into())
    }

    pub fn from_sec1(key: &[u8]) -> Result<Self> {
        Ok(DecodingKey::from_ec_der(key).into())
    }
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtClaims,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: EcdsaDecodingKey) -> Result<T> {
        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims.clear(); // we don't use `exp`, don't require it
        validation_options.sub = T::SUB.to_owned().into();

        let payload = jsonwebtoken::decode::<JwtPayload<T>>(&self.0, &pubkey.0, &validation_options)
            .context("Wallet certificate JWT validation failed")?
            .claims
            .payload;
        Ok(payload)
    }

    pub fn sign(payload: &T, privkey: &[u8]) -> Result<Jwt<T>> {
        jsonwebtoken::encode(
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
        .context("JWT signing failed")
        .map(Jwt::from)
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

fn der_to_pem(bts: &[u8], label: &str) -> Result<String> {
    use der::pem::{encode, encoded_len, LineEnding};

    let expected_len = encoded_len(label, LineEnding::LF, bts).map_err(anyhow::Error::msg)?;
    let mut buf = vec![0u8; expected_len];
    let pem = encode(label, LineEnding::LF, bts, &mut buf).map_err(anyhow::Error::msg)?;
    Ok(pem.to_owned())
}
