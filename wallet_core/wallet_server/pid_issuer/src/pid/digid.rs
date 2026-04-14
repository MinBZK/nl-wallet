use jsonwebtoken::Algorithm;
use jwk_simple::Key;

use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::tls_pinned_client_builder;
use jwe::algorithm::RsaAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweRsaPrivateKey;
use jwe::error::RsaPrivateJwkError;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::token::TokenRequest;

use crate::pid::userinfo;
use crate::pid::userinfo::UserInfo;
use crate::pid::userinfo::UserInfoError;
use crate::settings::DigidClientSettings;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("RSA private key JWK error: {0}")]
    RsaJwk(#[from] RsaPrivateJwkError),

    #[error("userinfo error: {0}")]
    UserInfo(#[from] UserInfoError),
}

/// An OIDC client for exchanging an access token provided by the user for their BSN at the IdP.
pub struct OpenIdClient {
    decrypter: JweDecrypter,
    client_id: String,
    http_client: HttpJsonClient,
    oidc_identifier: IssuerIdentifier,
}

impl OpenIdClient {
    pub fn try_new(
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        digid_client_settings: DigidClientSettings,
    ) -> Result<Self> {
        let jwe_private_key = JweRsaPrivateKey::try_from_jwk(bsn_privkey, RsaAlgorithm::RsaOaep)?;
        let certs = digid_client_settings
            .trust_anchors
            .into_iter()
            .map(|ta| ta.into_certificate());

        let userinfo_client = OpenIdClient {
            decrypter: JweDecrypter::from_rsa_private_key(&jwe_private_key),
            client_id: client_id.into(),
            http_client: HttpJsonClient::try_new(tls_pinned_client_builder(certs))?,
            oidc_identifier: digid_client_settings.oidc_identifier,
        };

        Ok(userinfo_client)
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String> {
        let userinfo_claims = userinfo::request_userinfo::<UserInfo>(
            &self.http_client,
            &self.oidc_identifier,
            token_request,
            &self.client_id,
            Algorithm::RS256,
            &self.decrypter,
        )
        .await?;

        Ok(userinfo_claims.bsn)
    }
}
