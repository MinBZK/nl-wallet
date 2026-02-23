use std::hash::Hash;
use std::marker::PhantomData;

use tracing::info;
use url::Url;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::oidc::HttpOidcClient;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcReqwestClient;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

use super::DigidClient;
use super::DigidError;
use super::DigidSession;

#[derive(Debug)]
pub struct HttpDigidClient<C = TlsPinningConfig, O = HttpOidcClient> {
    _marker: PhantomData<(C, O)>,
}

#[derive(Debug)]
pub struct HttpDigidSession<C = TlsPinningConfig, O = HttpOidcClient> {
    _http_config: PhantomData<C>,
    oidc_client: O,
    auth_url: Url,
}

impl<C, O> HttpDigidClient<C, O> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<C, O> Default for HttpDigidClient<C, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, O> DigidClient<C> for HttpDigidClient<C, O>
where
    C: IntoPinnedReqwestClient + Clone + Hash,
    O: OidcClient,
{
    type Session = HttpDigidSession<C, O>;

    async fn start_session(
        &self,
        digid_config: DigidConfiguration,
        http_config: C,
        redirect_uri: Url,
    ) -> Result<Self::Session, DigidError> {
        let http_client = OidcReqwestClient::try_new(http_config)?;
        let (oidc_client, auth_url) = O::start(&http_client, digid_config.client_id, redirect_uri).await?;

        info!("DigiD auth URL generated");

        let session = HttpDigidSession {
            _http_config: PhantomData,
            oidc_client,
            auth_url,
        };

        Ok(session)
    }
}

impl<C, O> DigidSession<C> for HttpDigidSession<C, O>
where
    C: IntoPinnedReqwestClient + Clone + Hash,
    O: OidcClient,
{
    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    async fn into_token_request(self, _http_config: &C, redirect_uri: Url) -> Result<TokenRequest, DigidError> {
        let token_request = self.oidc_client.into_token_request(&redirect_uri)?;

        Ok(token_request)
    }
}

#[cfg(test)]
mod test {
    use http_utils::tls::insecure::InsecureHttpConfig;
    use openid4vc::oidc::MockOidcClient;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use serial_test::serial;
    use url::Url;
    use wallet_configuration::wallet_config::DigidConfiguration;

    use super::super::DigidClient;
    use super::super::DigidSession;
    use super::HttpDigidClient;
    use super::HttpDigidSession;

    fn default_token_request() -> TokenRequest {
        TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code: "".to_owned().into(),
            },
            code_verifier: Default::default(),
            client_id: Default::default(),
            redirect_uri: Default::default(),
        }
    }

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_start_no_app2app() {
        let oidc_client = MockOidcClient::start_context();
        oidc_client
            .expect()
            .return_once(|_, _, _| Ok((MockOidcClient::default(), Url::parse("https://example.com/").unwrap())));

        let client = HttpDigidClient::<_, MockOidcClient>::new();
        let session = client
            .start_session(
                DigidConfiguration::default(),
                InsecureHttpConfig::new("https://digid.example.com".parse().unwrap()),
                "https://app.example.com".parse().unwrap(),
            )
            .await
            .expect("starting DigiD session should succeed");

        assert_eq!(*session.auth_url(), "https://example.com/".parse().unwrap());
    }

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request_no_app2app() {
        let mut client = MockOidcClient::default();
        client
            .expect_into_token_request()
            .return_once(|_| Ok(default_token_request()));

        let session = HttpDigidSession {
            _http_config: std::marker::PhantomData::<InsecureHttpConfig>,
            oidc_client: client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request = session
            .into_token_request(
                &InsecureHttpConfig::new("https://digid.example.com".parse().unwrap()),
                "https://example.com/deeplink/return-from-digid".parse().unwrap(),
            )
            .await;

        assert!(token_request.is_ok());
    }
}
