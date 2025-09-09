use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crypto::keys::EcdsaKey;

use crate::JwtTyp;
use crate::VerifiedJwt;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_from_p256;
use crate::wua::WUA_JWT_TYP;

/// Claims of a JWT credential: the body of the JWT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialClaims<T> {
    #[serde(rename = "cnf")]
    pub confirmation: JwtCredentialConfirmation,

    #[serde(flatten)]
    pub contents: JwtCredentialContents<T>,
}

impl<T> JwtTyp for JwtCredentialClaims<T> {
    const TYP: &'static str = WUA_JWT_TYP;
}

impl<T> JwtCredentialClaims<T>
where
    T: Serialize + std::fmt::Debug,
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
        attributes: T,
    ) -> Result<VerifiedJwt<JwtCredentialClaims<T>>, JwtError> {
        let jwt = VerifiedJwt::sign_with_typ(
            JwtCredentialClaims::new(holder_pubkey, iss, attributes)?,
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
