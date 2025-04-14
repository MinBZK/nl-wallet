use serde::Deserialize;
use serde::Serialize;

use http_utils::reqwest::JsonReqwestBuilder;
use openid4vc::oidc;
use openid4vc::oidc::alg::rsaes::RsaesJweAlgorithm;
use openid4vc::oidc::alg::rsaes::RsaesJweDecrypter;
use openid4vc::oidc::enc::aescbc_hmac::AescbcHmacJweEncryption;
use openid4vc::oidc::BiscuitError;
use openid4vc::oidc::Empty;
use openid4vc::oidc::JoseError;
use openid4vc::oidc::OidcError;
use openid4vc::oidc::SignatureAlgorithm;
use openid4vc::oidc::JWT;
use openid4vc::token::TokenRequest;

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
pub struct OpenIdClient<C> {
    decrypter_private_key: RsaesJweDecrypter,
    http_config: C,
}

impl<C> OpenIdClient<C>
where
    C: JsonReqwestBuilder,
{
    pub fn new(bsn_privkey: &str, http_config: C) -> Result<Self> {
        let userinfo_client = OpenIdClient {
            decrypter_private_key: OpenIdClient::<C>::decrypter(bsn_privkey)?,
            http_config,
        };
        Ok(userinfo_client)
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String> {
        let access_token = &oidc::request_token(&self.http_config, token_request)
            .await?
            .access_token;

        let userinfo_claims: JWT<UserInfo, Empty> = oidc::request_userinfo(
            &self.http_config,
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
        let metadata = oidc::Config::discover(&self.http_config).await?;
        Ok(metadata)
    }
}
