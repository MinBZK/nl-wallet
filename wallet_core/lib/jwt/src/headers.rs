use derive_more::Constructor;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;

use crate::DEFAULT_HEADER;
use crate::error::JwtError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct JwtHeader {
    pub alg: Algorithm,
}

impl Default for JwtHeader {
    fn default() -> Self {
        JwtHeader { alg: Algorithm::ES256 }
    }
}

impl From<JwtHeader> for Header {
    fn from(value: JwtHeader) -> Self {
        Header {
            alg: value.alg,
            ..DEFAULT_HEADER.to_owned()
        }
    }
}

impl From<Header> for JwtHeader {
    fn from(value: Header) -> Self {
        JwtHeader { alg: value.alg }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithJwk {
    pub jwk: Jwk,

    #[serde(flatten)]
    header: JwtHeader,
}

impl HeaderWithJwk {
    pub fn new(jwk: Jwk) -> Self {
        HeaderWithJwk {
            jwk,
            header: JwtHeader::default(),
        }
    }
}

impl From<HeaderWithJwk> for Header {
    fn from(value: HeaderWithJwk) -> Self {
        let mut header: Header = value.header.into();
        header.jwk = Some(value.jwk);
        header
    }
}

impl TryFrom<Header> for HeaderWithJwk {
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let jwk = value.jwk.as_ref().ok_or(JwtError::HeaderConversion)?.clone();
        Ok(HeaderWithJwk {
            header: value.into(),
            jwk,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithX5c {
    pub x5c: Vec<String>,

    #[serde(flatten)]
    header: JwtHeader,
}

impl HeaderWithX5c {
    pub fn new(x5c: Vec<String>) -> Self {
        HeaderWithX5c {
            x5c,
            header: JwtHeader { alg: Algorithm::ES256 },
        }
    }
}

impl From<HeaderWithX5c> for Header {
    fn from(value: HeaderWithX5c) -> Self {
        let mut header: Header = value.header.into();
        header.x5c = Some(value.x5c);
        header
    }
}

impl TryFrom<Header> for HeaderWithX5c {
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let x5c = value.x5c.as_ref().ok_or(JwtError::HeaderConversion)?.clone();
        Ok(HeaderWithX5c {
            header: value.into(),
            x5c,
        })
    }
}
