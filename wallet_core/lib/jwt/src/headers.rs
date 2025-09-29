use std::borrow::Cow;

use base64::prelude::*;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::WithVerifyingKey;
use crypto::x509::BorrowingCertificate;
use utils::vec_at_least::VecNonEmpty;

use crate::JwtTyp;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_from_p256;
use crate::jwk::jwk_to_p256;

pub(crate) const DEFAULT_JWT_TYP: &str = "jwt";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithTyp {
    pub alg: Algorithm,
    pub typ: Cow<'static, str>,
}

impl HeaderWithTyp {
    pub(crate) fn new<T: JwtTyp>() -> Self {
        HeaderWithTyp {
            alg: Algorithm::ES256,
            typ: Cow::Borrowed(T::TYP),
        }
    }
}

impl Default for HeaderWithTyp {
    fn default() -> Self {
        HeaderWithTyp {
            alg: Algorithm::ES256,
            typ: Cow::Borrowed(DEFAULT_JWT_TYP),
        }
    }
}

impl From<HeaderWithTyp> for Header {
    fn from(value: HeaderWithTyp) -> Self {
        Header {
            alg: value.alg,
            typ: Some(value.typ.into_owned()),
            ..Default::default()
        }
    }
}

impl TryFrom<Header> for HeaderWithTyp {
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let typ = value.typ.ok_or(JwtError::MissingTyp)?;
        Ok(HeaderWithTyp {
            alg: value.alg,
            typ: Cow::Owned(typ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithJwk<H = HeaderWithTyp> {
    #[serde(flatten)]
    header: H,

    pub jwk: Jwk,
}

impl HeaderWithJwk {
    pub async fn try_from_verifying_key(key: &impl WithVerifyingKey) -> Result<Self, JwkConversionError> {
        let jwk = jwk_from_p256(
            &key.verifying_key()
                .await
                .map_err(|e| JwkConversionError::VerifyingKeyFromPrivateKey(e.into()))?,
        )?;
        Ok(HeaderWithJwk {
            header: HeaderWithTyp::default(),
            jwk,
        })
    }
}

impl<H: Into<Header>> From<HeaderWithJwk<H>> for Header {
    fn from(value: HeaderWithJwk<H>) -> Self {
        let mut header: Header = value.header.into();
        header.jwk = Some(value.jwk);
        header
    }
}

impl<H> HeaderWithJwk<H> {
    pub fn inner(&self) -> &H {
        &self.header
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey, JwkConversionError> {
        jwk_to_p256(&self.jwk)
    }
}

impl<H, E> TryFrom<Header> for HeaderWithJwk<H>
where
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let jwk = value.jwk.as_ref().ok_or(JwtError::MissingJwk)?.clone();
        Ok(HeaderWithJwk {
            header: value.try_into().map_err(|e| JwtError::HeaderConversion(Box::new(e)))?,
            jwk,
        })
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderWithX5c<H = HeaderWithTyp> {
    #[serde(flatten)]
    header: H,

    #[serde_as(as = "Vec<Base64>")]
    pub x5c: VecNonEmpty<BorrowingCertificate>,
}

impl HeaderWithX5c {
    pub fn from_certs(x5c: VecNonEmpty<BorrowingCertificate>) -> HeaderWithX5c {
        HeaderWithX5c {
            header: HeaderWithTyp::default(),
            x5c,
        }
    }
}

impl<H> HeaderWithX5c<H> {
    pub fn inner(&self) -> &H {
        &self.header
    }
}

impl<H: Into<Header>> From<HeaderWithX5c<H>> for Header {
    fn from(value: HeaderWithX5c<H>) -> Self {
        let mut header: Header = value.header.into();
        header.x5c = Some(
            value
                .x5c
                .iter()
                .map(|cert| BASE64_STANDARD.encode(cert.as_ref()))
                .collect(),
        );
        header
    }
}

impl<H, E> TryFrom<Header> for HeaderWithX5c<H>
where
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = JwtError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let x5c = value
            .x5c
            .as_ref()
            .ok_or(JwtError::MissingX5c)?
            .iter()
            .map(|encoded_cert| {
                BASE64_STANDARD
                    .decode(encoded_cert)
                    .map_err(|e| JwtError::HeaderConversion(Box::new(e)))
                    .and_then(|bytes| {
                        BorrowingCertificate::from_der(bytes).map_err(|e| JwtError::HeaderConversion(Box::new(e)))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|e| JwtError::HeaderConversion(Box::new(e)))?;

        Ok(HeaderWithX5c {
            header: value.try_into().map_err(|e| JwtError::HeaderConversion(Box::new(e)))?,
            x5c,
        })
    }
}
