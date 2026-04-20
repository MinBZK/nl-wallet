use std::sync::Arc;

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
use openid4vc::metadata::well_known::WellKnownError;
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error creating RSA private key from BSN private key: {0}")]
    RsaJwk(#[source] RsaPrivateJwkError),

    #[error("error fetching well-known openid metadata: {0}")]
    WellKnown(#[source] WellKnownError),

    #[error("error fetching userinfo: {0}")]
    UserInfo(#[source] UserInfoError),
}

/// Holds the TLS-pinned HTTP client, upstream issuer identifier, and a lazy
/// cache of the upstream's OIDC discovery metadata. Shared (behind an `Arc`)
/// by everything that needs to talk to the upstream — the authorization
/// endpoint resolver and the userinfo exchange — so `/.well-known/openid-configuration`
/// is fetched at most once per process.
pub struct DigidMetadataCache {
    http_client: HttpJsonClient,
    oidc_identifier: IssuerIdentifier,
    cached: OnceCell<OidcProviderMetadata>,
}

impl DigidMetadataCache {
    pub fn try_new(settings: DigidClientSettings) -> Result<Self, reqwest::Error> {
        let certs = settings.trust_anchors.into_iter().map(|ta| ta.into_certificate());
        let http_client = HttpJsonClient::try_new(tls_pinned_client_builder(certs))?;
        Ok(Self {
            http_client,
            oidc_identifier: settings.oidc_identifier,
            cached: OnceCell::new(),
        })
    }

    pub async fn metadata(&self) -> Result<&OidcProviderMetadata, WellKnownError> {
        self.cached
            .get_or_try_init(|| async {
                well_known::fetch_well_known::<OidcProviderMetadata>(
                    &self.http_client,
                    &self.oidc_identifier,
                    WellKnownPath::OpenidConfiguration,
                )
                .await
            })
            .await
    }

    pub fn http_client(&self) -> &HttpJsonClient {
        &self.http_client
    }
}

/// Implements [`UpstreamAuthorizationEndpointResolver`] by performing OIDC discovery against the configured
/// upstream issuer on the first call. The discovery result is cached in the shared
/// [`DigidMetadataCache`] for the lifetime of the process.
pub struct DigidAuthorizationEndpointResolver {
    cache: Arc<DigidMetadataCache>,
}

impl DigidAuthorizationEndpointResolver {
    pub fn new(cache: Arc<DigidMetadataCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl UpstreamAuthorizationEndpointResolver for DigidAuthorizationEndpointResolver {
    async fn resolve(&self) -> Result<Url, UpstreamResolveError> {
        let metadata = self
            .cache
            .metadata()
            .await
            .map_err(|e| UpstreamResolveError::Discovery(Box::new(e)))?;
        metadata
            .authorization_endpoint
            .clone()
            .ok_or(UpstreamResolveError::NoAuthorizationEndpoint)
    }
}

/// An OIDC client for exchanging an access token provided by the user for their BSN at the IdP.
pub struct OpenIdClient {
    decrypter: JweDecrypter,
    client_id: String,
    cache: Arc<DigidMetadataCache>,
}

impl OpenIdClient {
    pub fn try_new(
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        cache: Arc<DigidMetadataCache>,
    ) -> Result<Self, Error> {
        let jwe_private_key =
            JweRsaPrivateKey::try_from_jwk(bsn_privkey, EXPECTED_JWE_RSA_ALGORITHM).map_err(Error::RsaJwk)?;

        Ok(OpenIdClient {
            decrypter: JweDecrypter::from_rsa_private_key(&jwe_private_key),
            client_id: client_id.into(),
            cache,
        })
    }

    pub async fn bsn(&self, token_request: TokenRequest) -> Result<String, Error> {
        let metadata = self.cache.metadata().await.map_err(Error::WellKnown)?;
        let userinfo_claims = userinfo::request_userinfo::<UserInfo>(
            self.cache.http_client(),
            metadata,
            token_request,
            &self.client_id,
            &self.decrypter,
            (EXPECTED_JWS_ALGORITHM, EXPECTED_JWE_ENC_ALGORITHM),
        )
        .await
        .map_err(Error::UserInfo)?;

        Ok(userinfo_claims.bsn)
    }
}
