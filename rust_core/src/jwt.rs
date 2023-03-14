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

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtClaims,
{
    pub fn parse_and_verify(&self, pubkey: &[u8]) -> Result<T> {
        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims.clear(); // we don't use `exp`, don't require it
        validation_options.sub = T::SUB.to_owned().into();

        Ok(jsonwebtoken::decode::<JwtPayload<T>>(
            &self.0,
            &DecodingKey::from_ec_der(pubkey),
            &validation_options,
        )
        .context("Wallet certificate JWT validation failed")?
        .claims
        .payload)
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
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        String::serialize(&self.0, serializer)
    }
}
impl<'de, T> Deserialize<'de> for Jwt<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer).map(Jwt::from)
    }
}
