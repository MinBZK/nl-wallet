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
//! ```text
//! Authorization: DPoP $token
//! ```
//! - The DPoP JWT must include the `ath` field in its body, which must be equal to the URL-safe-no-pad base64 encoding
//!   of the SHA256 of the token.
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

use std::sync::LazyLock;

use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use crypto::keys::PublicKey;
use crypto::utils::random_string;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::FromStr;
use error_category::ErrorCategory;
use futures::FutureExt;
use jwt::Algorithm;
use jwt::JwtTyp;
use jwt::JwtValidation;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtSignError;
use jwt::error::JwtVerifyError;
use jwt::headers::HeaderWithJwk;
use p256::ecdsa::SigningKey;
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

use crate::scope::Scope;
use crate::scope::ScopeInvalid;
use crate::token::AccessToken;

pub const DPOP_HEADER_NAME: &str = "DPoP";
pub const DPOP_NONCE_HEADER_NAME: &str = "DPoP-Nonce";

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DpopError {
    #[error("unsupported JWT: {0}")]
    #[category(critical)]
    InvalidJwt(#[source] JwtVerifyError),

    #[error("incorrect DPoP JWT HTTP method")]
    #[category(critical)]
    IncorrectMethod,

    #[error("incorrect DPoP JWT url")]
    #[category(critical)]
    IncorrectUrl,

    #[error("incorrect DPoP JWT nonce")]
    #[category(critical)]
    IncorrectNonce,

    #[error("incorrect DPoP JWT access token hash")]
    #[category(critical)]
    IncorrectAccessTokenHash,

    #[error("missing JWK")]
    #[category(critical)]
    MissingJwk,

    #[error("incorrect JWK public key")]
    #[category(critical)]
    IncorrectJwkPublicKey,

    #[error("JWT sign error: {0}")]
    JwtSign(#[source] JwtSignError),
}

pub type Result<T, E = DpopError> = std::result::Result<T, E>;

pub type DpopNonceInvalid = ScopeInvalid;

/// The allowed characters for the DPoP nonce are exactly the same as a OAuth 2.0 scope value.
///
/// Source: <https://datatracker.ietf.org/doc/html/rfc9449#name-nonce-syntax>
#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, FromStr, Serialize, Deserialize)]
#[as_ref(str)]
pub struct DpopNonce(Scope);

impl DpopNonce {
    pub fn new_random() -> Self {
        Self(Scope::try_new(random_string(32)).expect("a random alphanumeric string should be a valid DPoP nonce"))
    }
}

/// The optional nonce from a verified DPoP proof, pending validation against the nonce store.
#[must_use = "the DPoP nonce must be checked against and consumed from the nonce store"]
#[derive(Debug)]
pub struct DpopProofNonce(Option<DpopNonce>);

impl DpopProofNonce {
    pub fn into_inner(self) -> Option<DpopNonce> {
        self.0
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DpopPayload {
    #[serde(rename = "htu")]
    http_url: Url,
    #[serde(rename = "htm")]
    http_method: String,
    #[serde(rename = "ath")]
    #[serde_as(as = "Option<Base64<UrlSafe, Unpadded>>")]
    access_token_hash: Option<Vec<u8>>,
    nonce: Option<DpopNonce>,
    jti: String,
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,
}

pub const DPOP_JWT_TYPE: &str = "dpop+jwt";

impl JwtTyp for DpopPayload {
    const TYP: &'static str = DPOP_JWT_TYPE;
}

#[derive(Clone, AsRef, FromStr, Display)]
pub struct Dpop(UnverifiedJwt<DpopPayload, HeaderWithJwk>);

static DPOP_VALIDATION: LazyLock<JwtValidation> =
    LazyLock::new(|| JwtValidation::default_with_algorithms([Algorithm::ES256]));

impl Dpop {
    pub fn new(
        signing_key: &SigningKey,
        url: Url,
        method: &Method,
        access_token: Option<&AccessToken>,
        nonce: Option<DpopNonce>,
    ) -> Result<Self> {
        let payload = DpopPayload {
            jti: random_string(32),
            iat: Utc::now(),
            http_method: method.to_string(),
            http_url: url,
            nonce,
            access_token_hash: access_token.map(AccessToken::sha256),
        };
        let jwt = SignedJwt::sign_with_jwk(&payload, signing_key)
            // This is safe, as the `EcdsaKey::try_sign()` implementation on `SigningKey` will return immediately.
            .now_or_never()
            .unwrap()
            .map_err(DpopError::JwtSign)?;

        Ok(Self(jwt.into()))
    }

    fn verify_data(
        verified_dpop: &DpopPayload,
        url: &Url,
        method: &Method,
        access_token: Option<&AccessToken>,
    ) -> Result<()> {
        if verified_dpop.http_method != method.to_string() {
            return Err(DpopError::IncorrectMethod);
        }
        if verified_dpop.http_url != *url {
            return Err(DpopError::IncorrectUrl);
        }
        if verified_dpop.access_token_hash != access_token.map(AccessToken::sha256) {
            return Err(DpopError::IncorrectAccessTokenHash);
        }

        Ok(())
    }

    /// Verify the DPoP JWT against the public key inside its header, returning that public key.
    /// This should only be called in the first HTTP request of a protocol, where an unsolicited nonce is ignored.
    /// In later requests, use [`Dpop::verify_expecting_key()`] with the public key that this method returns.
    pub fn verify(self, url: &Url, method: &Method, access_token: Option<&AccessToken>) -> Result<PublicKey> {
        let Self(jwt) = self;
        let (_header, payload, public_key) = jwt
            .parse_and_verify_with_jwk(DPOP_VALIDATION.to_owned())
            .map_err(DpopError::InvalidJwt)?;
        Self::verify_data(&payload, url, method, access_token)?;

        Ok(public_key)
    }

    /// Verify the DPoP JWT against the specified public key obtained previously from [`Dpop::verify()`], returning
    /// the nonce from the verified payload. The caller must validate and consume this nonce to prevent replay:
    /// this method does not track `jti` values or validate `iat`.
    pub fn verify_expecting_key(
        self,
        expected_public_key: PublicKey,
        url: &Url,
        method: &Method,
        access_token: Option<&AccessToken>,
    ) -> Result<DpopProofNonce> {
        let validation = DPOP_VALIDATION.to_owned().into_validation(&expected_public_key);

        let (_, payload) = self
            .0
            .parse_and_verify_with_expected_jwk(expected_public_key, validation)
            .map_err(DpopError::InvalidJwt)?;
        Self::verify_data(&payload, url, method, access_token)?;

        Ok(DpopProofNonce(payload.nonce))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use base64::prelude::*;
    use crypto::keys::PublicKey;
    use jwt::Header;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use reqwest::Method;
    use rstest::rstest;
    use serde::de::DeserializeOwned;
    use url::Url;

    use super::DPOP_JWT_TYPE;
    use super::Dpop;
    use super::DpopError;
    use super::DpopNonce;
    use super::DpopPayload;
    use crate::token::AccessToken;

    #[rstest]
    #[case(None, Some("123".to_string().into()))]
    #[case(Some("123".to_string().into()), None)]
    #[case(Some("123".to_string().into()), Some("456".to_string().into()))]
    #[tokio::test]
    async fn dpop(
        #[case] access_token: Option<AccessToken>,
        #[case] wrong_access_token: Option<AccessToken>,
        #[values(None, Some(DpopNonce::new_random()))] nonce: Option<DpopNonce>,
    ) {
        let signing_key = SigningKey::generate();
        let url: Url = "https://example.com/path".parse().unwrap();
        let method = Method::POST;

        let dpop = Dpop::new(&signing_key, url.clone(), &method, access_token.as_ref(), nonce.clone()).unwrap();

        // Check the `typ` of the Header
        let header: Header = part(0, dpop.0.serialization());
        assert_eq!(header.typ, Some(DPOP_JWT_TYPE.to_string()));

        // Examine some fields in the claims
        let claims: DpopPayload = part(1, dpop.0.serialization());
        assert_eq!(claims.access_token_hash, access_token.as_ref().map(AccessToken::sha256));
        assert_eq!(claims.http_url, url);
        assert_eq!(claims.http_method, method.to_string());

        // Verifying it against incorrect parameters doesn't work
        dpop.clone()
            .verify(&url, &method, wrong_access_token.as_ref())
            .unwrap_err();
        dpop.clone()
            .verify(&url, &Method::PATCH, access_token.as_ref())
            .unwrap_err();
        dpop.clone()
            .verify(
                &"https://incorrect_url/".parse().unwrap(),
                &method,
                access_token.as_ref(),
            )
            .unwrap_err();

        // We can verify the DPoP
        let pubkey = dpop.clone().verify(&url, &method, access_token.as_ref()).unwrap();
        assert_matches!(pubkey, PublicKey::ESP256(verifying_key) if verifying_key == *signing_key.verifying_key());
        assert_eq!(
            dpop.verify_expecting_key(pubkey, &url, &method, access_token.as_ref())
                .unwrap()
                .into_inner(),
            nonce
        );
    }

    #[test]
    fn verify_expecting_key_returns_verified_nonce() {
        let signing_key = SigningKey::generate();
        let public_key = PublicKey::from(*signing_key.verifying_key());
        let url: Url = "https://example.com/credential".parse().unwrap();
        let access_token = AccessToken::from("access_token".to_string());
        let nonce = DpopNonce::new_random();
        let dpop = Dpop::new(
            &signing_key,
            url.clone(),
            &Method::POST,
            Some(&access_token),
            Some(nonce.clone()),
        )
        .unwrap();

        // A nonce must not bypass any of the proof's authentication or request-binding checks.
        let wrong_key = PublicKey::from(*SigningKey::generate().verifying_key());
        assert_matches!(
            dpop.clone()
                .verify_expecting_key(wrong_key, &url, &Method::POST, Some(&access_token)),
            Err(DpopError::InvalidJwt(_))
        );
        assert_matches!(
            dpop.clone()
                .verify_expecting_key(public_key.clone(), &url, &Method::GET, Some(&access_token)),
            Err(DpopError::IncorrectMethod)
        );
        assert_matches!(
            dpop.clone().verify_expecting_key(
                public_key.clone(),
                &"https://example.com/other".parse().unwrap(),
                &Method::POST,
                Some(&access_token),
            ),
            Err(DpopError::IncorrectUrl)
        );
        assert_matches!(
            dpop.clone()
                .verify_expecting_key(public_key.clone(), &url, &Method::POST, None),
            Err(DpopError::IncorrectAccessTokenHash)
        );

        assert_eq!(
            dpop.verify_expecting_key(public_key, &url, &Method::POST, Some(&access_token))
                .unwrap()
                .into_inner(),
            Some(nonce)
        );
    }

    /// Decode and deserialize the specified part of the JWT.
    fn part<T: DeserializeOwned>(i: u8, jwt: &str) -> T {
        let bts = BASE64_URL_SAFE_NO_PAD
            .decode(jwt.split('.').take((i + 1).into()).last().unwrap())
            .unwrap();
        serde_json::from_slice(&bts).unwrap()
    }
}
