use josekit::JoseError;
use josekit::jwe::alg::rsaes::RsaesJweAlgorithm;
use josekit::jwe::alg::rsaes::RsaesJweDecrypter;
use josekit::jwe::enc::aescbc_hmac::AescbcHmacJweEncryption;
use jsonwebtoken::Algorithm;

use http_utils::client::TlsPinningConfig;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::tls_pinned_client_builder;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::token::TokenRequest;

use crate::pid::userinfo;
use crate::pid::userinfo::UserInfo;
use crate::pid::userinfo::UserInfoError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("JOSE error: {0}")]
    JoseKit(#[from] JoseError),

    #[error("userinfo error: {0}")]
    UserInfo(#[from] UserInfoError),
}

/// An OIDC client for exchanging an access token provided by the user for their BSN at the IdP.
pub struct OpenIdClient {
    decrypter_private_key: RsaesJweDecrypter,
    client_id: String,
    http_client: HttpJsonClient,
    authorization_server: IssuerIdentifier,
}

impl OpenIdClient {
    pub fn try_new(bsn_privkey: &str, client_id: impl Into<String>, http_config: &TlsPinningConfig) -> Result<Self> {
        let authorization_server: IssuerIdentifier = http_config
            .base_url()
            .as_ref()
            .as_str()
            .trim_end_matches('/')
            .parse()
            .expect("TlsPinningConfig base URL should be a valid IssuerIdentifier");
        let certs = http_config
            .trust_anchors()
            .iter()
            .map(|ta| ta.clone().into_certificate());
        let userinfo_client = OpenIdClient {
            decrypter_private_key: Self::decrypter(bsn_privkey)?,
            client_id: client_id.into(),
            http_client: HttpJsonClient::try_new(tls_pinned_client_builder(certs))?,
            authorization_server,
        };

        Ok(userinfo_client)
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String> {
        let userinfo_claims = userinfo::request_userinfo::<UserInfo>(
            &self.http_client,
            &self.authorization_server,
            token_request,
            &self.client_id,
            Algorithm::RS256,
            Some((&self.decrypter_private_key, &AescbcHmacJweEncryption::A128cbcHs256)),
        )
        .await?;

        Ok(userinfo_claims.bsn)
    }

    fn decrypter(jwk_json: &str) -> Result<RsaesJweDecrypter> {
        let jwk = serde_json::from_str(jwk_json)?;
        let decrypter = RsaesJweAlgorithm::RsaOaep.decrypter_from_jwk(&jwk)?;

        Ok(decrypter)
    }
}
