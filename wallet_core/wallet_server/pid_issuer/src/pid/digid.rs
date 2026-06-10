use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::tls_pinned_client_builder;
use indexmap::IndexSet;
use jsonwebtoken::Algorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::algorithm::RsaAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::decryption::JweRsaPrivateKey;
use jwe::error::RsaPrivateJwkError;
use jwk_simple::Key;
use jwt::nonce::Nonce;
use openid4vc::authorization::OidcAuthorizationRequest;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
use openid4vc::metadata::well_known;
use openid4vc::metadata::well_known::WellKnownError;
use openid4vc::metadata::well_known::WellKnownPath;
use openid4vc::pkce::S256PkcePair;
use openid4vc::token::AuthorizationCode;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
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

    #[error("upstream metadata has no authorization_endpoint")]
    NoUpstreamAuthorizationEndpoint,

    #[error("error fetching userinfo: {0}")]
    UserInfo(#[source] UserInfoError),

    #[error("encoding authorization request as query string failed: {0}")]
    Encode(#[source] serde_qs::Error),
}

/// Holds the TLS-pinned HTTP client, upstream issuer identifier, and a lazy cache of the upstream's OIDC discovery
/// metadata.
pub struct DigidMetadataCache {
    http_client: HttpJsonClient,
    oidc_identifier: IssuerIdentifier,
    // TODO (PVW-5936): implement cache expiration
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

/// Abstraction over the DigiD operations the pid_issuer's authorization-code flow needs: resolving
/// the upstream authorization endpoint and exchanging an authorization code for the user's BSN.
#[trait_variant::make(Send)]
pub trait DigidClient {
    /// Resolve the upstream provider's authorization endpoint from its discovery metadata.
    async fn authorization_request(
        &self,
        client_id: String,
        redirect_uri: Url,
        state: String,
        pkce_pair: &S256PkcePair,
    ) -> Result<Url, Error>;

    /// Exchange an upstream authorization `code` (with its PKCE `code_verifier` and the
    /// `redirect_uri` used at `/authorize`) for the user's BSN via the upstream `/userinfo` endpoint.
    async fn bsn(&self, code: AuthorizationCode, code_verifier: String, redirect_uri: Url) -> Result<String, Error>;
}

/// HTTP-backed [`DigidClient`] that exchanges an access token provided by the user for their BSN at the IdP.
pub struct HttpDigidClient {
    decrypter: JweDecrypter,
    client_id: String,
    cache: DigidMetadataCache,
}

impl HttpDigidClient {
    pub fn try_new(bsn_privkey: &Key, client_id: impl Into<String>, cache: DigidMetadataCache) -> Result<Self, Error> {
        let jwe_private_key =
            JweRsaPrivateKey::try_from_jwk(bsn_privkey, EXPECTED_JWE_RSA_ALGORITHM).map_err(Error::RsaJwk)?;

        Ok(HttpDigidClient {
            decrypter: JweDecrypter::from_rsa_private_key(&jwe_private_key),
            client_id: client_id.into(),
            cache,
        })
    }

    async fn authorization_endpoint(&self) -> Result<Url, Error> {
        let metadata = self.cache.metadata().await.map_err(Error::WellKnown)?;

        let authorization_endpoint = metadata
            .as_ref()
            .authorization_endpoint
            .clone()
            .ok_or(Error::NoUpstreamAuthorizationEndpoint)?;

        Ok(authorization_endpoint)
    }
}

impl DigidClient for HttpDigidClient {
    async fn authorization_request(
        &self,
        client_id: String,
        redirect_uri: Url,
        state: String,
        pkce_pair: &S256PkcePair,
    ) -> Result<Url, Error> {
        // Create a new upstream authorization request
        let mut vci_request = VciAuthorizationRequest::for_auth_code(client_id, redirect_uri, state, None, pkce_pair);
        vci_request.scope = Some(IndexSet::from_iter([String::from("openid")]));

        let oidc_request = OidcAuthorizationRequest {
            vci_request,
            nonce: Some(Nonce::new_random()),
        };

        let query_string = serde_qs::to_string(&oidc_request).map_err(Error::Encode)?;
        let mut redirect_url = self.authorization_endpoint().await?;
        redirect_url.set_query(Some(&query_string));

        Ok(redirect_url)
    }

    async fn bsn(&self, code: AuthorizationCode, code_verifier: String, redirect_uri: Url) -> Result<String, Error> {
        let metadata = self.cache.metadata().await.map_err(Error::WellKnown)?;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::AuthorizationCode { code },
            client_id: Some(self.client_id.clone()),
            code_verifier: Some(code_verifier),
            redirect_uri: Some(redirect_uri),
        };

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use http_utils::reqwest::HttpJsonClient;
    use http_utils::reqwest::default_reqwest_client_builder;
    use jwk_simple::Key;
    use openid4vc::issuer_identifier::IssuerIdentifier;
    use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
    use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
    use openid4vc::pkce::PkcePair;
    use openid4vc::pkce::S256PkcePair;
    use tokio::sync::OnceCell;
    use url::Url;

    use super::DigidClient;
    use super::DigidMetadataCache;
    use super::HttpDigidClient;

    const CLIENT_ID: &str = "issuer-client-id";
    const REDIRECT_URI: &str = "https://issuer.example.com/digid/callback";
    const ISSUER_STATE: &str = "issuer-state";
    const AUTHORIZATION_ENDPOINT: &str = "https://issuer.example.com/authorize";

    impl Default for DigidMetadataCache {
        fn default() -> Self {
            Self::new_preloaded()
        }
    }

    impl DigidMetadataCache {
        fn new_preloaded() -> Self {
            let issuer_identifier = IssuerIdentifier::try_new(String::from("https://issuer.example.com/")).unwrap();
            Self {
                http_client: HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap(),
                oidc_identifier: issuer_identifier.clone(),
                cached: OnceCell::new_with(Some(OidcProviderMetadata::new(AuthorizationServerMetadata::new_mock(
                    issuer_identifier,
                )))),
            }
        }
    }

    /// A valid RSA private JWK, only needed to satisfy `HttpDigidClient::try_new`'s `JweDecrypter`
    /// construction; the decrypter is unused by `authorization_request`.
    fn fixed_test_jwe_key() -> Key {
        let json = r#"{
          "kty": "RSA",
          "kid": "cc34c0a0-bd5a-4a3c-a50d-a2a7db7643df",
          "alg": "RSA-OAEP",
          "n": "pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGaut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByyFKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk9_B_f1lVrmOKuHjTNHq48w",
          "e": "AQAB",
          "d": "ksDmucdMJXkFGZxiomNHnroOZxe8AmDLDGO1vhs-POa5PZM7mtUPonxwjVmthmpbZzla-kg55OFfO7YcXhg-Hm2OWTKwm73_rLh3JavaHjvBqsVKuorX3V3RYkSro6HyYIzFJ1Ek7sLxbjDRcDOj4ievSX0oN9l-JZhaDYlPlci5uJsoqro_YrE0PRRWVhtGynd-_aWgQv1YzkfZuMD-hJtDi1Im2humOWxA4eZrFs9eG-whXcOvaSwO4sSGbS99ecQZHM2TcdXeAs1PvjVgQ_dKnZlGN3lTWoWfQP55Z7Tgt8Nf1q4ZAKd-NlMe-7iqCFfsnFwXjSiaOa2CRGZn-Q",
          "p": "4A5nU4ahEww7B65yuzmGeCUUi8ikWzv1C81pSyUKvKzu8CX41hp9J6oRaLGesKImYiuVQK47FhZ--wwfpRwHvSxtNU9qXb8ewo-BvadyO1eVrIk4tNV543QlSe7pQAoJGkxCia5rfznAE3InKF4JvIlchyqs0RQ8wx7lULqwnn0",
          "q": "ven83GM6SfrmO-TBHbjTk6JhP_3CMsIvmSdo4KrbQNvp4vHO3w1_0zJ3URkmkYGhz2tgPlfd7v1l2I6QkIh4Bumdj6FyFZEBpxjE4MpfdNVcNINvVj87cLyTRmIcaGxmfylY7QErP8GFA-k4UoH_eQmGKGK44TRzYj5hZYGWIC8",
          "dp": "lmmU_AG5SGxBhJqb8wxfNXDPJjf__i92BgJT2Vp4pskBbr5PGoyV0HbfUQVMnw977RONEurkR6O6gxZUeCclGt4kQlGZ-m0_XSWx13v9t9DIbheAtgVJ2mQyVDvK4m7aRYlEceFh0PsX8vYDS5o1txgPwb3oXkPTtrmbAGMUBpE",
          "dq": "mxRTU3QDyR2EnCv0Nl0TCF90oliJGAHR9HJmBe__EjuCBbwHfcT8OG3hWOv8vpzokQPRl5cQt3NckzX3fs6xlJN4Ai2Hh2zduKFVQ2p-AF2p6Yfahscjtq-GY9cB85NxLy2IXCC0PF--Sq9LOrTE9QV988SJy_yUrAjcZ5MmECk",
          "qi": "ldHXIrEmMZVaNwGzDF9WG8sHj2mOZmQpw9yrjLK9hAsmsNr5LTyqWAqJIYZSwPTYWhY4nu2O0EY9G9uYiqewXfCKw_UngrJt8Xwfq1Zruz0YY869zPN4GiE9-9rzdZB33RBw8kIOquY3MK74FMwCihYx_LiU2YTHkaoJ3ncvtvg"
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[tokio::test]
    async fn authorization_request_builds_upstream_redirect_url() {
        let client = HttpDigidClient::try_new(&fixed_test_jwe_key(), CLIENT_ID, DigidMetadataCache::default()).unwrap();
        let pkce_pair = S256PkcePair::generate();

        let redirect_url = client
            .authorization_request(
                CLIENT_ID.to_string(),
                REDIRECT_URI.parse::<Url>().unwrap(),
                ISSUER_STATE.to_string(),
                &pkce_pair,
            )
            .await
            .unwrap();

        // The redirect targets the upstream authorization endpoint from the (preloaded) metadata.
        assert!(redirect_url.as_str().starts_with(AUTHORIZATION_ENDPOINT));

        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert_eq!(params.get("client_id").map(String::as_str), Some(CLIENT_ID));
        assert_eq!(params.get("redirect_uri").map(String::as_str), Some(REDIRECT_URI));
        assert_eq!(params.get("state").map(String::as_str), Some(ISSUER_STATE));
        assert_eq!(params.get("scope").map(String::as_str), Some("openid"));
        assert!(params.contains_key("nonce"));
        assert_eq!(params.get("code_challenge_method").map(String::as_str), Some("S256"));
        assert_eq!(
            params.get("code_challenge").map(String::as_str),
            Some(pkce_pair.code_challenge())
        );
    }
}
