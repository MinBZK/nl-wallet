use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use openid4vc::{
    oidc::{
        self,
        alg::rsaes::{RsaesJweAlgorithm, RsaesJweDecrypter},
        enc::aescbc_hmac::AescbcHmacJweEncryption,
        BiscuitError, Empty, JoseError, OidcError, SignatureAlgorithm, JWT,
    },
    token::TokenRequest,
};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Serialize, Deserialize)]
struct UserInfo {
    bsn: String,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OpenID Connect error: {0}")]
    Oidc(#[from] OidcError),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("JOSE error: {0}")]
    JoseKit(#[from] JoseError),
    #[error("JWE error: {0}")]
    Jwe(#[from] BiscuitError),
}

/// An OIDC client for exchanging an access token provided by the user for their BSN at the IdP.
pub struct OpenIdClient {
    issuer_url: Url,
    decrypter_private_key: RsaesJweDecrypter,
}

impl OpenIdClient {
    pub fn new(issuer_url: Url, bsn_privkey: String) -> Result<Self> {
        let userinfo_client = OpenIdClient {
            issuer_url,
            decrypter_private_key: OpenIdClient::decrypter(&bsn_privkey)?,
        };
        Ok(userinfo_client)
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String> {
        let access_token = &oidc::request_token(&http_client(), self.issuer_url.clone(), token_request)
            .await?
            .access_token;

        let userinfo_claims: JWT<UserInfo, Empty> = oidc::request_userinfo(
            &http_client(),
            self.issuer_url.clone(),
            access_token,
            SignatureAlgorithm::RS256,
            Some((&self.decrypter_private_key, &AescbcHmacJweEncryption::A128cbcHs256)),
        )
        .await?;

        let bsn = userinfo_claims.payload()?.private.bsn.clone();

        Ok(bsn)
    }

    fn decrypter(jwk_json: &str) -> Result<RsaesJweDecrypter> {
        let jwk = serde_json::from_str(jwk_json)?;
        let decrypter = RsaesJweAlgorithm::RsaOaep.decrypter_from_jwk(&jwk)?;

        Ok(decrypter)
    }
}

fn http_client() -> reqwest::Client {
    let builder = reqwest::Client::builder();
    #[cfg(feature = "disable_tls_validation")]
    let builder = builder.danger_accept_invalid_certs(true);
    builder
        .timeout(CLIENT_TIMEOUT)
        .build()
        .expect("Could not build reqwest HTTP client")
}
