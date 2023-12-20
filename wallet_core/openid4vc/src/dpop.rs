//! Implements the DPoP HTTP header as in https://datatracker.ietf.org/doc/html/rfc9449.
//!
//! Like PKCE, DPoP allows a HTTP server to authenticate a HTTP client as the same client of some previous request.
//! However, being based on asymmetric signature schemes it can be used across more than two HTTP requests,
//! unlike PKCE, and additionally it can also sign other things such as (access) tokens.
//!
//! A DPoP is a JWT which (1) includes the public key with which it can be verified, and (2) signs the URL and HTTP
//! method of the HTTP request being done. In a second HTTP request, the HTTP server can authenticate the request
//! by matching the public key against that of the DPoP JWT from some earlier request, and verifying the signature.
//!
//! HTTP endpoints that require some token (e.g. an (access) token as in OpenID/OAuth) may additionally require this
//! token to be signed in a DPoP JWT, to prevent replay attacks. If so, then:
//! - the token itself must be sent as a HTTP header as follows:
//!   ```text
//!   Authorization: DPoP $token
//!   ```
//! - The DPoP JWT must include the `ath` field in its body, which must be equal to the URL-no-pad base64 encoding of
//!   the SHA256 of the token.
//!
//! Example DPoP JWT header and body:
//! ```json
//! {
//!   "typ": "dpop+jwt",
//!   "alg": "ES256",
//!   "jwk": {
//!     "kty": "EC",
//!     "crv": "P-256"
//!     "x": "l8tFrhx-34tV3hRICRDY9zCkDlpBhF42UQUfWVAWBFs",
//!     "y": "9VE4jf_Ok_o64zbTTlcuNJajHmt6v9TDVrU0CdvGRDA",
//!   }
//! }
//! .
//! {
//!   "jti": "-BwC3ESc6acc2lTc",
//!   "htm": "POST",
//!   "htu": "https://server.example.com/token",
//!   "iat": 1562262616
//! }
//! ```

use jsonwebtoken::{Algorithm, Header, TokenData, Validation};
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use url::Url;
use wallet_common::{
    jwt::{EcdsaDecodingKey, Jwt},
    keys::SecureEcdsaKey,
    utils::random_string,
};

use crate::{jwk_from_p256, jwk_to_p256, Error, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DpopPayload {
    #[serde(rename = "htu")]
    http_url: Url,
    #[serde(rename = "htm")]
    http_method: String,
    #[serde(rename = "ath")]
    access_token_hash: Option<String>,
    jti: String,
    iat: u64,
}

struct Dpop(Jwt<DpopPayload>);

const OPENID4VCI_DPOP_JWT_TYPE: &str = "dpop+jwt";

impl Dpop {
    pub async fn new(
        private_key: &impl SecureEcdsaKey,
        url: Url,
        method: String,
        access_token_hash: Option<String>,
    ) -> Result<Self> {
        let header = Header {
            typ: Some(OPENID4VCI_DPOP_JWT_TYPE.to_string()),
            alg: Algorithm::ES256,
            jwk: Some(jwk_from_p256(
                &private_key
                    .verifying_key()
                    .await
                    .map_err(|e| Error::VerifyingKeyFromPrivateKey(e.into()))?,
            )?),
            ..Default::default()
        };

        let payload = DpopPayload {
            jti: random_string(32),
            iat: jsonwebtoken::get_current_timestamp(),
            http_method: method,
            http_url: url,
            access_token_hash,
        };

        let jwt = Jwt::sign(&payload, &header, private_key).await?;
        Ok(Self(jwt))
    }

    fn verify_signature(&self, verifying_key: &VerifyingKey) -> Result<TokenData<DpopPayload>> {
        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims = Default::default();
        let token_data = jsonwebtoken::decode::<DpopPayload>(
            &self.0 .0,
            &EcdsaDecodingKey::from(verifying_key).0,
            &validation_options,
        )?;
        Ok(token_data)
    }

    fn verify_data(
        &self,
        token_data: &TokenData<DpopPayload>,
        url: Url,
        method: String,
        access_token: Option<String>,
    ) -> Result<()> {
        if token_data.header.typ != Some(OPENID4VCI_DPOP_JWT_TYPE.to_string()) {
            return Err(Error::UnsupportedJwtAlgorithm {
                expected: OPENID4VCI_DPOP_JWT_TYPE.to_string(),
                found: token_data.header.typ.clone().unwrap_or_default(),
            });
        }
        if token_data.claims.http_method != method {
            return Err(Error::IncorrectDpopMethod);
        }
        if token_data.claims.http_url != url {
            return Err(Error::IncorrectDpopUrl);
        }
        if token_data.claims.access_token_hash != access_token {
            return Err(Error::IncorrectDpopAccessTokenHash);
        }
        Ok(())
    }

    /// Verify the DPoP JWT against the public key inside its header, returning that public key.
    /// This should only be called in the first HTTP request of a protocol. In later requests,
    /// [`Dpop::verify_expecting_key()`] should be used with the public key that this method returns.
    pub async fn verify(&self, url: Url, method: String, access_token: Option<String>) -> Result<VerifyingKey> {
        // Grab the public key from the JWT header
        let header = jsonwebtoken::decode_header(&self.0 .0)?;
        let verifying_key = jwk_to_p256(&header.jwk.ok_or(Error::MissingJwk)?)?;

        let token_data = self.verify_signature(&verifying_key)?;
        self.verify_data(&token_data, url, method, access_token)?;

        Ok(verifying_key)
    }

    /// Verify the DPoP JWT against the specified public key obtained previously from [`Dpop::verify()`].
    pub async fn verify_expecting_key(
        &self,
        expected_verifying_key: &VerifyingKey,
        url: Url,
        method: String,
        access_token: Option<String>,
    ) -> Result<()> {
        let token_data = self.verify_signature(expected_verifying_key)?;
        self.verify_data(&token_data, url, method, access_token)?;

        // Compare the specified key against the one in the JWT header
        let contained_key = jwk_to_p256(&token_data.header.jwk.ok_or(Error::MissingJwk)?)?;
        if contained_key != *expected_verifying_key {
            return Err(Error::IncorrectJwkPublicKey);
        }

        Ok(())
    }
}
