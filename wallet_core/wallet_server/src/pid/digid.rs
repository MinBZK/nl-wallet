use reqwest::Certificate;
use serde::{Deserialize, Serialize};

use openid4vc::{
    oidc::{
        self,
        alg::rsaes::{RsaesJweAlgorithm, RsaesJweDecrypter},
        enc::aescbc_hmac::AescbcHmacJweEncryption,
        BiscuitError, Empty, JoseError, OidcError, SignatureAlgorithm, JWT,
    },
    token::TokenRequest,
};
use wallet_common::{reqwest::trusted_reqwest_client_builder, urls::BaseUrl};

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
    issuer_url: BaseUrl,
    decrypter_private_key: RsaesJweDecrypter,
    trust_anchors: Vec<Certificate>,
}

impl OpenIdClient {
    pub fn new(issuer_url: BaseUrl, bsn_privkey: &str, trust_anchors: Vec<Certificate>) -> Result<Self> {
        let userinfo_client = OpenIdClient {
            issuer_url,
            decrypter_private_key: OpenIdClient::decrypter(bsn_privkey)?,
            trust_anchors,
        };
        Ok(userinfo_client)
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String> {
        let http_client = trusted_reqwest_client_builder(self.trust_anchors.clone())
            .build()
            .expect("Could not build reqwest HTTP client");

        let access_token = &oidc::request_token(&http_client, &self.issuer_url, token_request)
            .await?
            .access_token;

        let userinfo_claims: JWT<UserInfo, Empty> = oidc::request_userinfo(
            &http_client,
            &self.issuer_url,
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

    pub async fn discover_metadata(&self) -> Result<oidc::Config> {
        let http_client = trusted_reqwest_client_builder(self.trust_anchors.clone())
            .build()
            .expect("Could not build reqwest HTTP client");

        let metadata = oidc::Config::discover(&http_client, &self.issuer_url).await?;
        Ok(metadata)
    }
}
