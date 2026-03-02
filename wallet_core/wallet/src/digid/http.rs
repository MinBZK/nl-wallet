use std::hash::Hash;

use tracing::info;
use url::Url;

use http_utils::reqwest::IntoReqwestClient;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcReqwestClient;
use wallet_configuration::wallet_config::DigidConfiguration;

use super::DigidError;
use super::DigidSessionState;

pub async fn start_digid_session<O, C>(
    digid_config: DigidConfiguration,
    http_config: C,
    redirect_uri: Url,
) -> Result<DigidSessionState<O>, DigidError>
where
    C: IntoReqwestClient + Clone + Hash,
    O: OidcClient,
{
    let http_client = OidcReqwestClient::try_new(http_config)?;
    let (oidc_client, auth_url) = O::start(&http_client, digid_config.client_id, redirect_uri).await?;

    info!("DigiD auth URL generated");

    Ok(DigidSessionState { oidc_client, auth_url })
}

#[cfg(test)]
mod test {
    use http_utils::client::TlsPinningConfig;
    use serial_test::serial;
    use url::Url;

    use openid4vc::oidc::MockOidcClient;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use wallet_configuration::wallet_config::DigidConfiguration;

    use crate::digid::DigidSessionState;

    use super::start_digid_session;

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
    async fn test_start_session() {
        let oidc_client = MockOidcClient::start_context();
        oidc_client
            .expect()
            .return_once(|_, _, _| Ok((MockOidcClient::default(), Url::parse("https://example.com/").unwrap())));

        let session: DigidSessionState<MockOidcClient> = start_digid_session(
            DigidConfiguration::default(),
            TlsPinningConfig::try_new("https://digid.example.com".parse().unwrap(), vec![]).unwrap(),
            "https://app.example.com".parse().unwrap(),
        )
        .await
        .expect("starting DigiD session should succeed");

        assert_eq!(session.auth_url, "https://example.com/".parse().unwrap());
    }

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request() {
        let mut oidc_client = MockOidcClient::default();
        oidc_client
            .expect_into_token_request()
            .return_once(|_| Ok(default_token_request()));

        let session = super::DigidSessionState {
            oidc_client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request =
            session.into_token_request(&"https://example.com/deeplink/return-from-digid".parse().unwrap());

        assert!(token_request.is_ok());
    }
}
