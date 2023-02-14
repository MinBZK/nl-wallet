use std::marker::PhantomData;

use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// TODO implement keyring and use kid header item for key rollover

#[derive(Debug)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T> From<String> for Jwt<T> {
    fn from(val: String) -> Self {
        Jwt(val, PhantomData)
    }
}

pub trait JwtClaims {
    fn sub() -> String;
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtClaims,
{
    pub fn parse_and_verify(&self, pubkey: &[u8]) -> Result<T> {
        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims.clear(); // we don't use `exp`, don't require it
        validation_options.sub = T::sub().into();

        Ok(jsonwebtoken::decode::<T>(
            &self.0,
            &DecodingKey::from_ec_der(pubkey),
            &validation_options,
        )
        .context("Wallet certificate JWT validation failed")?
        .claims)
    }

    pub fn sign(payload: &T, privkey: &[u8]) -> Result<Jwt<T>> {
        jsonwebtoken::encode(
            &Header {
                alg: Algorithm::ES256,
                kid: "0".to_owned().into(),
                ..Default::default()
            },
            payload,
            &jsonwebtoken::EncodingKey::from_ec_der(privkey),
        )
        .context("JWT signing failed")
        .map(Jwt::from)
    }
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
