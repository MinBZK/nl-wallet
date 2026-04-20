use async_trait::async_trait;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::tls_pinned_client_builder;
use jsonwebtoken::Algorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::algorithm::RsaAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweRsaPrivateKey;
use jwe::error::RsaPrivateJwkError;
use jwk_simple::Key;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
use openid4vc::metadata::well_known;
use openid4vc::metadata::well_known::WellKnownPath;
use openid4vc::token::TokenRequest;
use openid4vc_server::issuer::UpstreamAuthorizationEndpointResolver;
use openid4vc_server::issuer::UpstreamResolveError;
use tokio::sync::OnceCell;
use url::Url;

use crate::pid::userinfo;
use crate::pid::userinfo::UserInfo;
use crate::pid::userinfo::UserInfoError;
use crate::settings::DigidClientSettings;

const EXPECTED_JWE_RSA_ALGORITHM: RsaAlgorithm = RsaAlgorithm::RsaOaep;
const EXPECTED_JWE_ENC_ALGORITHM: EncryptionAlgorithm = EncryptionAlgorithm::A128CbcHs256;
const EXPECTED_JWS_ALGORITHM: Algorithm = Algorithm::RS256;

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

/// Implements [`UpstreamAuthorizationEndpointResolver`] by performing OIDC discovery against the configured
/// upstream issuer on the first call and caching the result for the lifetime of the process.
pub struct DigidAuthorizationEndpointResolver {
    http_client: HttpJsonClient,
    oidc_identifier: IssuerIdentifier,
    cached_endpoint: OnceCell<Url>,
}

impl DigidAuthorizationEndpointResolver {
    pub fn try_new(settings: DigidClientSettings) -> std::result::Result<Self, reqwest::Error> {
        let certs = settings.trust_anchors.into_iter().map(|ta| ta.into_certificate());
        let http_client = HttpJsonClient::try_new(tls_pinned_client_builder(certs))?;
        Ok(Self {
            http_client,
            oidc_identifier: settings.oidc_identifier,
            cached_endpoint: OnceCell::new(),
        })
    }
}

#[async_trait]
impl UpstreamAuthorizationEndpointResolver for DigidAuthorizationEndpointResolver {
    async fn resolve(&self) -> std::result::Result<Url, UpstreamResolveError> {
        self.cached_endpoint
            .get_or_try_init(|| async {
                let metadata: OidcProviderMetadata = well_known::fetch_well_known(
                    &self.http_client,
                    &self.oidc_identifier,
                    WellKnownPath::OpenidConfiguration,
                )
                .await
                .map_err(|e| UpstreamResolveError::Discovery(Box::new(e)))?;

                metadata
                    .authorization_endpoint
                    .ok_or(UpstreamResolveError::NoAuthorizationEndpoint)
            })
            .await
            .cloned()
    }
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
        let jwe_private_key = JweRsaPrivateKey::try_from_jwk(bsn_privkey, EXPECTED_JWE_RSA_ALGORITHM)?;
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
            &self.decrypter,
            (EXPECTED_JWS_ALGORITHM, EXPECTED_JWE_ENC_ALGORITHM),
        )
        .await?;

        Ok(userinfo_claims.bsn)
    }
}
