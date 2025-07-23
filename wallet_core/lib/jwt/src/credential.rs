use jsonwebtoken::Header;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crypto::keys::EcdsaKey;

use crate::Jwt;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_from_p256;

/// Claims of a JWT credential: the body of the JWT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialClaims<T> {
    #[serde(rename = "cnf")]
    pub confirmation: JwtCredentialConfirmation,

    #[serde(flatten)]
    pub contents: JwtCredentialContents<T>,
}

impl<T> JwtCredentialClaims<T>
where
    T: Serialize,
{
    pub fn new(pubkey: &VerifyingKey, iss: String, attributes: T) -> Result<Self, JwkConversionError> {
        let claims = Self {
            confirmation: JwtCredentialConfirmation {
                jwk: jwk_from_p256(pubkey)?,
            },
            contents: JwtCredentialContents { iss, attributes },
        };

        Ok(claims)
    }

    pub async fn new_signed(
        holder_pubkey: &VerifyingKey,
        issuer_privkey: &impl EcdsaKey,
        iss: String,
        typ: Option<String>,
        attributes: T,
    ) -> Result<Jwt<JwtCredentialClaims<T>>, JwtError> {
        let jwt = Jwt::<JwtCredentialClaims<T>>::sign(
            &JwtCredentialClaims::<T>::new(holder_pubkey, iss, attributes)?,
            &Header {
                typ: typ.or(Some("jwt".to_string())),
                ..Header::new(jsonwebtoken::Algorithm::ES256)
            },
            issuer_privkey,
        )
        .await?;

        Ok(jwt)
    }
}

/// Contents of a `JwtCredential`, containing everything of the [`JwtCredentialClaims`] except the holder public
/// key (`Cnf`): the attributes and metadata of the credential.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialContents<T> {
    pub iss: String,

    #[serde(flatten)]
    pub attributes: T,
}

/// Contains the holder public key of a `JwtCredential`.
/// ("Cnf" stands for "confirmation", see <https://datatracker.ietf.org/doc/html/rfc7800>.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialConfirmation {
    pub jwk: Jwk,
}
