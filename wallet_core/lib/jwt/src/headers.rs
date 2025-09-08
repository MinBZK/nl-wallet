use derive_more::Constructor;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;

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
        Header::new(value.alg)
    }
}

impl From<Header> for JwtHeader {
    fn from(value: Header) -> Self {
        JwtHeader { alg: value.alg }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithTyp {
    pub typ: String,

    #[serde(flatten)]
    header: JwtHeader,
}

impl Default for HeaderWithTyp {
    fn default() -> Self {
        Self::new("jwt".to_string())
    }
}

impl HeaderWithTyp {
    pub fn new(typ: String) -> Self {
        Self::new_with_alg(typ, Algorithm::ES256)
    }

    fn new_with_alg(typ: String, alg: Algorithm) -> HeaderWithTyp {
        HeaderWithTyp {
            typ,
            header: JwtHeader { alg },
        }
    }
}

impl From<HeaderWithTyp> for Header {
    fn from(value: HeaderWithTyp) -> Self {
        let mut header: Header = value.header.into();
        header.typ = Some(value.typ);
        header
    }
}

impl TryFrom<Header> for HeaderWithTyp {
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let typ = value.typ.as_ref().ok_or(JwtError::HeaderConversion)?.clone();
        Ok(HeaderWithTyp {
            header: value.into(),
            typ,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithJwkAndTyp {
    pub jwk: Jwk,

    #[serde(flatten)]
    header: HeaderWithTyp,
}

impl HeaderWithJwkAndTyp {
    pub fn new(typ: String, jwk: Jwk) -> Self {
        HeaderWithJwkAndTyp {
            jwk,
            header: HeaderWithTyp::new(typ),
        }
    }

    pub fn typ(&self) -> &str {
        &self.header.typ
    }
}

impl From<HeaderWithJwkAndTyp> for Header {
    fn from(value: HeaderWithJwkAndTyp) -> Self {
        let mut header: Header = value.header.into();
        header.jwk = Some(value.jwk);
        header
    }
}

impl TryFrom<Header> for HeaderWithJwkAndTyp {
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let jwk = value.jwk.as_ref().ok_or(JwtError::HeaderConversion)?.clone();
        Ok(HeaderWithJwkAndTyp {
            header: value.try_into()?,
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
