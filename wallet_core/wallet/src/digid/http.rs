use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::LazyLock;

use http::StatusCode;
use http::header::LOCATION;
use reqwest::Response;
use reqwest::redirect::Policy;
use tracing::info;
use tracing::warn;
use url::Url;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::issuance_base_uri;
use openid4vc::oidc::HttpOidcClient;
use openid4vc::oidc::OidcClient;
use openid4vc::oidc::OidcReqwestClient;
use openid4vc::token::TokenRequest;
use wallet_configuration::wallet_config::DigidConfiguration;

use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::reqwest::CachedReqwestClient;

use super::DigidClient;
use super::DigidError;
use super::DigidSession;
use super::app2app::DigidJsonRequest;
use super::app2app::ReturnUrlParameters;
use super::app2app::format_app2app_query;

fn build_app2app_http_client<C>(http_config: C) -> Result<PinnedReqwestClient, reqwest::Error>
where
    C: IntoPinnedReqwestClient,
{
    http_config.try_into_custom_client(|client_builder| client_builder.redirect(Policy::none()))
}

fn extract_location_header(res: &Response) -> Result<Url, DigidError> {
    if res.status() != StatusCode::TEMPORARY_REDIRECT {
        return Err(DigidError::ExpectedRedirect(res.status()));
    }

    info!("Received redirect, extracting URL from location header");

    let url = res
        .headers()
        .get(LOCATION)
        .ok_or(DigidError::MissingLocation)?
        .to_str()?
        .parse()?;

    Ok(url)
}

#[derive(Debug)]
pub struct HttpDigidClient<C = TlsPinningConfig, O = HttpOidcClient> {
    http_client: Arc<CachedReqwestClient<C>>,
    oidc_client_type: PhantomData<O>,
}

#[derive(Debug)]
enum DigidSessionType<C> {
    Web,
    App2App(Arc<CachedReqwestClient<C>>),
}

#[derive(Debug)]
pub struct HttpDigidSession<C = TlsPinningConfig, O = HttpOidcClient> {
    session_type: DigidSessionType<C>,
    oidc_client: O,
    auth_url: Url,
}

impl<C, O> HttpDigidClient<C, O> {
    pub fn new() -> Self {
        Self {
            http_client: Arc::new(CachedReqwestClient::new()),
            oidc_client_type: PhantomData,
        }
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
        // the icon must be within the same domain as the return URL, but the path is stripped off the UL
        const LOGO_PATH: &str = "/.well-known/logo.png";
        static ICON_URL: LazyLock<Url> = LazyLock::new(|| UNIVERSAL_LINK_BASE_URL.as_ref().join(LOGO_PATH).unwrap());

        // Use a different HTTP client for the app2app requests than the one we use for OidcReqwestClient.
        let app2app_config_http_client = digid_config
            .app2app
            .map(|app2app_config| {
                let http_client = self
                    .http_client
                    .get_or_try_init(&http_config, build_app2app_http_client)?;

                Ok::<_, reqwest::Error>((app2app_config, http_client))
            })
            .transpose()?;

        // Note that http_config contains the TLS pinning configuration for both the OIDC discovery quest and any
        // subsequent requests, including potential app2app requests. This means that any TLS certificate presented to
        // the client should be issued under the same (set of) CA(s), even though the requests could in theory end up
        // connecting to different hosts. In practice however, this will most likely be the same host.
        let http_client = OidcReqwestClient::try_new(http_config)?;
        let (oidc_client, mut auth_url) = O::start(&http_client, digid_config.client_id, redirect_uri).await?;

        let (session_type, auth_url) = match app2app_config_http_client {
            Some((app2app_config, http_client)) => {
                info!("Constructing DigiD universal link from Auth URL");

                // This enforces DigiD, even if others are supported by rdo-max.
                auth_url.query_pairs_mut().append_pair("login_hint", "digid");

                info!("Sending get request to Auth URL, expecting an HTTP Redirect");
                let response = http_client
                    .send_get(ReqwestClientUrl::Absolute(auth_url))
                    .await?
                    .error_for_status()?;
                let location = extract_location_header(&response)?;

                let saml_parameters =
                    serde_urlencoded::from_str(location.query().ok_or(DigidError::MissingLocationQuery)?)?;

                info!("Constructing DigiD universal link from redirect parameters");
                let json_request = DigidJsonRequest {
                    icon: ICON_URL.to_owned(),
                    return_url: issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner(),
                    host: app2app_config.host().map(|h| h.to_string()),
                    saml_parameters,
                };

                let app2app_query = format_app2app_query(json_request)?;
                let mut app2app_url = app2app_config.universal_link().clone();
                app2app_url.set_query(Some(&app2app_query));

                info!("DigiD app2app URL generated");

                (DigidSessionType::App2App(Arc::clone(&self.http_client)), app2app_url)
            }
            _ => (DigidSessionType::Web, auth_url),
        };

        let session = HttpDigidSession {
            session_type,
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

    async fn into_token_request(self, http_config: &C, redirect_uri: Url) -> Result<TokenRequest, DigidError> {
        let location = match self.session_type {
            DigidSessionType::Web => redirect_uri,
            DigidSessionType::App2App(http_client) => {
                info!("Constructing redirect_uri from JsonResponse");

                let ReturnUrlParameters { app_app } =
                    serde_urlencoded::from_str(redirect_uri.query().ok_or(DigidError::MissingLocationQuery)?)?;

                if let Some(error) = app_app.error_message {
                    warn!("Error message in JsonResponse: {error}");
                    return Err(DigidError::App2AppError(error));
                }

                // pass saml artifact and relay_state to rdo-max, exchange for redirect_uri
                info!("Sending SAML artifact and Relay State to acs URL, expecting a redirect");
                let response = http_client
                    .get_or_try_init(http_config, build_app2app_http_client)?
                    // this route is standard for rdo-max
                    .send_custom_get(ReqwestClientUrl::Relative("acs"), |request_builder| {
                        request_builder.query(&app_app.saml_parameters)
                    })
                    .await?
                    .error_for_status()?;

                extract_location_header(&response)?
            }
        };

        let token_request = self.oidc_client.into_token_request(&location)?;

        Ok(token_request)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use http::StatusCode;
    use http::header::LOCATION;
    use rstest::rstest;
    use serde::de::Error;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::http::HeaderValue;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::matchers::query_param;

    use http_utils::tls::insecure::InsecureHttpConfig;
    use http_utils::urls::BaseUrl;
    use openid4vc::oidc::MockOidcClient;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenRequestGrantType;
    use wallet_configuration::digid::DigidApp2AppConfiguration;
    use wallet_configuration::wallet_config::DigidConfiguration;

    use crate::reqwest::CachedReqwestClient;

    use super::super::DigidClient;
    use super::super::DigidError;
    use super::super::DigidSession;
    use super::super::app2app::App2AppErrorMessage;
    use super::super::test::base64;
    use super::DigidSessionType;
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

    #[rstest]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes(
            "https://preprod.example.com/saml/idp/request_authentication?SAMLRequest=rZNBj9owEIXv%2Byui3CHBBEIsiETZqkW\
             iLAK2h15Wjj1ZLCV2ak9Yfn7tBLRUqjj1FCWeee%2Bb58ncsrpq6LLFk9rD7xYsPgXBpa6Upd3RImyNoppZaaliNViKnB6WPzaUDGPaGI\
             2a6yr8q%2BlxD7MWDEqtfNP6eRG%2BbL9uXr6tt2%2BTEckyMZ2lMSmztGAJGYNIYz4aT%2BNxTAhLszguZqlv%2FAnGOo1F6CT9exDsj\
             D5LAWbrHBfhASU%2FoVTvwXp1fO2srG1hrSwyha4tJskgngzi2THOKJnSJP3lq55dAlIx7LRPiI2lUdQYcJOK0VDIdymGqor8mJEUTWT6\
             0N6YSxCUM2W30XbXbL5IJRzH41CKvsjS78fjbrB0%2BZSMo5dZ3uJaaWXbGswBzFlyeN1vPgHRIQzPWpsCoBIfrKoAPWY5ZiOeiHLME5K\
             whLNimjE2I7NJOSUTVkbdQAOulQKO2kSM2z7NJaKRRYvQ2zq2q%2B9aCbgswlGYd3VznwTtsjX5%2F8aZR%2FfqnV9v2NDrsoLoVtdBIl\
             wwWOm6YUZaf3mOWdZtHeY94n3dqnJLuIcyf7innHJf5z7v3ONDG%2BGv1IGBOBqmbKMNXgn%2FKd7RRg9w86fb8f3%2Fl%2F8B&RelayS\
             tate=eyJzdGF0ZSI6ICI5Y2Q3ZTcwMWEyMzJmYWZjMjUzNWM4ODE2MmU4Yjg2MzA4YTRmM2U3MmRjMDc2MmNjNjk2YjQ1YmVlYTdlOTIz\
             IiwgImNsaWVudF9pZCI6ICI1Zjg2Yzg3Yi1jMzViLTQ3M2ItYmZmOS00YTE5NzY3ZWQzZjciLCAicmVkaXJlY3RfdXJpIjogImh0dHBzO\
             i8vYXBwLnRlc3Qudm9vcmJlZWxkd2FsbGV0Lm5sL2RlZXBsaW5rL2F1dGhlbnRpY2F0aW9uIn0%3D&Signature=P7y2OyXiYm7oMGC7d\
             4m7K8gDZFfCWWFwNrDusQfh928T7kHGZob%2F3unQnvwu36EGxc805y8t87riyUgT9sgmWRm9D4u3Rr6eZ5tCcbvR5ERQZvS%2BlNOOMc\
             %2FuHlSv7L3ToXy5uh0tnUFX64L8qXLnPo9MgBBz%2FhOjSY72Pw2u%2BRTU3gmmij4M5%2FqFL4f5J%2FHupeYc3lID6ABLT4QpMinng\
             LQO8F82KqbfJXeLNqQ5pmGUB3nvDfsVmxsNGm1ZMI2ifQqtyDUvGGwXxkpHi7EL%2FdSripWBZ%2FHhJbnPzJkC5KqUxUEi1%2FZ8o4Mi\
             wLUHhS3khiMKKxjW6w2tIvpWi0yXDISQJvIfrSVAx24OXqHW3nti41gmndWbviLRH7E7GIB2b9V9qTRMKFv9RCN7oTT%2FsE1MKNNkJ2F\
             A2ePUu4lGYvZHDDf5gpcWr1y5ZiQull1x2nWlvGYkvBCHaWukThax39RC8AOZwixg3UvznMSGisElkDB6V7fqfdMbPXhDIb6%2BdXsAVs\
             G84iJ9qBXmdawO2OgusSMI1i7O8jZyZXM68qg%2FYP8ZJJdwQuQESlOjhZxyUNsTytif9%2FaqCjvb7FC%2B7L16kjPLACCk%2BOLft8J\
             RAw1%2B15MqdNBdg5aSlptdfFEwhI7REyJt8pv8HtlqPi11qGw1vZ%2BIH409nUsfAiXjkgk%3D&SigAlg=http%3A%2F%2Fwww.w3.or\
             g%2F2001%2F04%2Fxmldsig-more%23rsa-sha256".as_bytes()
        ).ok(),
        None,
        None,
        Ok(("https://app-preprod.example.com/app?app-app=".to_string() + &base64(json!({
            "Icon":"walletdebuginteraction://wallet.edi.rijksoverheid.nl/.well-known/logo.png",
            "ReturnUrl": "walletdebuginteraction://wallet.edi.rijksoverheid.nl/return-from-digid",
            "Host": "preprod.example.com",
            "SAMLRequest":
                "rZNBj9owEIXv+yui3CHBBEIsiETZqkWiLAK2h15Wjj1ZLCV2ak9Yfn7tBLRUqjj1FCWeee+b58ncsrpq6LLFk9rD7xYsPgXBpa6Up\
                 d3RImyNoppZaaliNViKnB6WPzaUDGPaGI2a6yr8q+lxD7MWDEqtfNP6eRG+bL9uXr6tt2+TEckyMZ2lMSmztGAJGYNIYz4aT+NxTA\
                 hLszguZqlv/AnGOo1F6CT9exDsjD5LAWbrHBfhASU/oVTvwXp1fO2srG1hrSwyha4tJskgngzi2THOKJnSJP3lq55dAlIx7LRPiI2\
                 lUdQYcJOK0VDIdymGqor8mJEUTWT60N6YSxCUM2W30XbXbL5IJRzH41CKvsjS78fjbrB0+ZSMo5dZ3uJaaWXbGswBzFlyeN1vPgHR\
                 IQzPWpsCoBIfrKoAPWY5ZiOeiHLME5KwhLNimjE2I7NJOSUTVkbdQAOulQKO2kSM2z7NJaKRRYvQ2zq2q+9aCbgswlGYd3VznwTts\
                 jX5/8aZR/fqnV9v2NDrsoLoVtdBIlwwWOm6YUZaf3mOWdZtHeY94n3dqnJLuIcyf7innHJf5z7v3ONDG+Gv1IGBOBqmbKMNXgn/Kd\
                 7RRg9w86fb8f3/l/8B",
            "RelayState":
                "eyJzdGF0ZSI6ICI5Y2Q3ZTcwMWEyMzJmYWZjMjUzNWM4ODE2MmU4Yjg2MzA4YTRmM2U3MmRjMDc2MmNjNjk2YjQ1YmVlYTdlOTIzI\
                iwgImNsaWVudF9pZCI6ICI1Zjg2Yzg3Yi1jMzViLTQ3M2ItYmZmOS00YTE5NzY3ZWQzZjciLCAicmVkaXJlY3RfdXJpIjogImh0dHB\
                zOi8vYXBwLnRlc3Qudm9vcmJlZWxkd2FsbGV0Lm5sL2RlZXBsaW5rL2F1dGhlbnRpY2F0aW9uIn0=",
            "SigAlg": "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "Signature":
                "P7y2OyXiYm7oMGC7d4m7K8gDZFfCWWFwNrDusQfh928T7kHGZob/3unQnvwu36EGxc805y8t87riyUgT9sgmWRm9D4u3Rr6eZ5tCc\
                bvR5ERQZvS+lNOOMc/uHlSv7L3ToXy5uh0tnUFX64L8qXLnPo9MgBBz/hOjSY72Pw2u+RTU3gmmij4M5/qFL4f5J/HupeYc3lID6AB\
                LT4QpMinngLQO8F82KqbfJXeLNqQ5pmGUB3nvDfsVmxsNGm1ZMI2ifQqtyDUvGGwXxkpHi7EL/dSripWBZ/HhJbnPzJkC5KqUxUEi1\
                /Z8o4MiwLUHhS3khiMKKxjW6w2tIvpWi0yXDISQJvIfrSVAx24OXqHW3nti41gmndWbviLRH7E7GIB2b9V9qTRMKFv9RCN7oTT/sE1\
                MKNNkJ2FA2ePUu4lGYvZHDDf5gpcWr1y5ZiQull1x2nWlvGYkvBCHaWukThax39RC8AOZwixg3UvznMSGisElkDB6V7fqfdMbPXhDI\
                b6+dXsAVsG84iJ9qBXmdawO2OgusSMI1i7O8jZyZXM68qg/YP8ZJJdwQuQESlOjhZxyUNsTytif9/aqCjvb7FC+7L16kjPLACCk+OL\
                ft8JRAw1+15MqdNBdg5aSlptdfFEwhI7REyJt8pv8HtlqPi11qGw1vZ+IH409nUsfAiXjkgk="
        }))).parse().unwrap()),
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes(
            "https://preprod.example.com/saml/idp/request_authentication?SAMLRequest=rZNBj9owEIXv%2Byui3CHBBEIsiETZqkW\
             iLAK2h15Wjj1ZLCV2ak9Yfn7tBLRUqjj1FCWeee%2Bb58ncsrpq6LLFk9rD7xYsPgXBpa6Upd3RImyNoppZaaliNViKnB6WPzaUDGPaGI\
             2a6yr8q%2BlxD7MWDEqtfNP6eRG%2BbL9uXr6tt2%2BTEckyMZ2lMSmztGAJGYNIYz4aT%2BNxTAhLszguZqlv%2FAnGOo1F6CT9exDsj\
             D5LAWbrHBfhASU%2FoVTvwXp1fO2srG1hrSwyha4tJskgngzi2THOKJnSJP3lq55dAlIx7LRPiI2lUdQYcJOK0VDIdymGqor8mJEUTWT6\
             0N6YSxCUM2W30XbXbL5IJRzH41CKvsjS78fjbrB0%2BZSMo5dZ3uJaaWXbGswBzFlyeN1vPgHRIQzPWpsCoBIfrKoAPWY5ZiOeiHLME5K\
             whLNimjE2I7NJOSUTVkbdQAOulQKO2kSM2z7NJaKRRYvQ2zq2q%2B9aCbgswlGYd3VznwTtsjX5%2F8aZR%2FfqnV9v2NDrsoLoVtdBIl\
             wwWOm6YUZaf3mOWdZtHeY94n3dqnJLuIcyf7innHJf5z7v3ONDG%2BGv1IGBOBqmbKMNXgn%2FKd7RRg9w86fb8f3%2Fl%2F8B&RelayS\
             tate=eyJzdGF0ZSI6ICI5Y2Q3ZTcwMWEyMzJmYWZjMjUzNWM4ODE2MmU4Yjg2MzA4YTRmM2U3MmRjMDc2MmNjNjk2YjQ1YmVlYTdlOTIz\
             IiwgImNsaWVudF9pZCI6ICI1Zjg2Yzg3Yi1jMzViLTQ3M2ItYmZmOS00YTE5NzY3ZWQzZjciLCAicmVkaXJlY3RfdXJpIjogImh0dHBzO\
             i8vYXBwLnRlc3Qudm9vcmJlZWxkd2FsbGV0Lm5sL2RlZXBsaW5rL2F1dGhlbnRpY2F0aW9uIn0%3D&Signature=P7y2OyXiYm7oMGC7d\
             4m7K8gDZFfCWWFwNrDusQfh928T7kHGZob%2F3unQnvwu36EGxc805y8t87riyUgT9sgmWRm9D4u3Rr6eZ5tCcbvR5ERQZvS%2BlNOOMc\
             %2FuHlSv7L3ToXy5uh0tnUFX64L8qXLnPo9MgBBz%2FhOjSY72Pw2u%2BRTU3gmmij4M5%2FqFL4f5J%2FHupeYc3lID6ABLT4QpMinng\
             LQO8F82KqbfJXeLNqQ5pmGUB3nvDfsVmxsNGm1ZMI2ifQqtyDUvGGwXxkpHi7EL%2FdSripWBZ%2FHhJbnPzJkC5KqUxUEi1%2FZ8o4Mi\
             wLUHhS3khiMKKxjW6w2tIvpWi0yXDISQJvIfrSVAx24OXqHW3nti41gmndWbviLRH7E7GIB2b9V9qTRMKFv9RCN7oTT%2FsE1MKNNkJ2F\
             A2ePUu4lGYvZHDDf5gpcWr1y5ZiQull1x2nWlvGYkvBCHaWukThax39RC8AOZwixg3UvznMSGisElkDB6V7fqfdMbPXhDIb6%2BdXsAVs\
             G84iJ9qBXmdawO2OgusSMI1i7O8jZyZXM68qg%2FYP8ZJJdwQuQESlOjhZxyUNsTytif9%2FaqCjvb7FC%2B7L16kjPLACCk%2BOLft8J\
             RAw1%2B15MqdNBdg5aSlptdfFEwhI7REyJt8pv8HtlqPi11qGw1vZ%2BIH409nUsfAiXjkgk%3D&SigAlg=http%3A%2F%2Fwww.w3.or\
             g%2F2001%2F04%2Fxmldsig-more%23rsa-sha256".as_bytes()
        ).ok(),
        None,
        Some("file://etc/passwd".parse().unwrap()),
        Err(DigidError::Http(
            reqwest::Client::new().get("file://etc/passwd?login_hint=digid").send().await.unwrap_err()
        ))
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        None,
        Some(OidcError::NoAuthCode),
        None,
        Err(DigidError::Oidc(OidcError::NoAuthCode))
    )]
    #[case(StatusCode::OK, None, None, None, Err(DigidError::ExpectedRedirect(StatusCode::OK)))]
    #[case(StatusCode::TEMPORARY_REDIRECT, None, None, None, Err(DigidError::MissingLocation))]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("not_a_url".as_bytes()).ok(),
        None,
        None,
        Err(DigidError::NotAUrl(url::ParseError::RelativeUrlWithoutBase))
    )]
    // this case is impossible to test without using unsafe, also the error cannot be created because of private fields
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        Some(unsafe { HeaderValue::from_maybe_shared_unchecked("🦀") }),
        None,
        None,
        Err(DigidError::HeaderNotAStr(
            unsafe { http::header::HeaderValue::from_maybe_shared_unchecked("🦀") }.to_str().unwrap_err()
        ))
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication".as_bytes()).ok(),
        None,
        None,
        Err(DigidError::MissingLocationQuery)
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication?hello".as_bytes()).ok(),
        None,
        None,
        Err(DigidError::UrlDeserialize(serde_urlencoded::de::Error::missing_field("SAMLRequest")))
    )]
    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_start_app2app(
        #[case] status: StatusCode,
        #[case] location: Option<HeaderValue>,
        #[case] oidc_error: Option<OidcError>,
        #[case] auth_url: Option<Url>,
        #[case] expected: Result<Url, DigidError>,
    ) {
        let server = MockServer::start().await;
        let base_url = auth_url.unwrap_or(server.uri().parse().unwrap());

        let digid_config = DigidConfiguration {
            client_id: Default::default(),
            app2app: Some(DigidApp2AppConfiguration::Preprod {
                host: "preprod.example.com".to_owned(),
                universal_link: "https://app-preprod.example.com/app".parse().unwrap(),
            }),
        };

        let mut template = ResponseTemplate::new(status);
        if let Some(location) = location {
            template = template.insert_header(LOCATION, location);
        }

        Mock::given(method("GET"))
            .and(query_param("login_hint", "digid"))
            .respond_with(template)
            .mount(&server)
            .await;

        let client = MockOidcClient::start_context();
        let auth_url = base_url.clone();
        client.expect().return_once(move |_, _, _| {
            if let Some(err) = oidc_error {
                return Err(err);
            }

            Ok((MockOidcClient::default(), auth_url))
        });

        let client = HttpDigidClient::<_, MockOidcClient>::new();
        let session_result = client
            .start_session(
                digid_config,
                InsecureHttpConfig::new("https://digid.example.com".parse().unwrap()),
                "https://app.example.com".parse().unwrap(),
            )
            .await;

        match (session_result.map(|session| session.auth_url().clone()), expected) {
            (Ok(o), Ok(k)) => assert_eq!(o, k),
            // unfortunately some of the errors don't implement PartialEq
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Err(e), Ok(o)) => panic!("assertion `left == right` failed\n left: {e:?}\nright: {o:?}"),
            (Ok(o), Err(e)) => panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}"),
        };
    }

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request_no_app2app() {
        let mut client = MockOidcClient::default();
        client
            .expect_into_token_request()
            .return_once(|_| Ok(default_token_request()));

        // same as a call to `HttpDigidSession::start`
        let session = HttpDigidSession {
            session_type: DigidSessionType::Web,
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

    // Don't use base64(json!()) here in the cases here as these tests test the base64 itself
    #[rstest]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiQUFRQUFNMks0c3dQOXdXWk1wU01hd3pteXBcd\
         TAwMkI3NUtFVXZFUGNmWHFIUVZFSG9WWENXNmdsbjdrcDNaZz0iLCJSZWxheVN0YXRlIjoiZXlKemRHRjBaU0k2SUNJNVkyUTNaVGN3TVdFe\
         U16Sm1ZV1pqTWpVek5XTTRPREUyTW1VNFlqZzJNekE0WVRSbU0yVTNNbVJqTURjMk1tTmpOamsyWWpRMVltVmxZVGRsT1RJeklpd2dJbU5zY\
         VdWdWRGOXBaQ0k2SUNJMVpqZzJZemczWWkxak16VmlMVFEzTTJJdFltWm1PUzAwWVRFNU56WTNaV1F6WmpjaUxDQWljbVZrYVhKbFkzUmZkW\
         EpwSWpvZ0ltaDBkSEJ6T2k4dllYQndMblJsYzNRdWRtOXZjbUpsWld4a2QyRnNiR1YwTG01c0wyUmxaWEJzYVc1ckwyRjFkR2hsYm5ScFkyR\
         jBhVzl1SW4wPSIsIkVycm9yTWVzc2FnZSI6bnVsbH0=".parse().unwrap(),
        None,
        Ok(()) // without encoded `=`
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiQUFRQUFNMks0c3dQOXdXWk1wU01hd3pteXBcd\
         TAwMkI3NUtFVXZFUGNmWHFIUVZFSG9WWENXNmdsbjdrcDNaZz0iLCJSZWxheVN0YXRlIjoiZXlKemRHRjBaU0k2SUNJNVkyUTNaVGN3TVdFe\
         U16Sm1ZV1pqTWpVek5XTTRPREUyTW1VNFlqZzJNekE0WVRSbU0yVTNNbVJqTURjMk1tTmpOamsyWWpRMVltVmxZVGRsT1RJeklpd2dJbU5zY\
         VdWdWRGOXBaQ0k2SUNJMVpqZzJZemczWWkxak16VmlMVFEzTTJJdFltWm1PUzAwWVRFNU56WTNaV1F6WmpjaUxDQWljbVZrYVhKbFkzUmZkW\
         EpwSWpvZ0ltaDBkSEJ6T2k4dllYQndMblJsYzNRdWRtOXZjbUpsWld4a2QyRnNiR1YwTG01c0wyUmxaWEJzYVc1ckwyRjFkR2hsYm5ScFkyR\
         jBhVzl1SW4wPSIsIkVycm9yTWVzc2FnZSI6bnVsbH0%3D".parse().unwrap(),
        None,
        Ok(()) // with encoded `=`
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiLyIsIlJlbGF5U3RhdGUiOiIvIiwiRXJyb3JNZ\
         XNzYWdlIjpudWxsfQ==".parse().unwrap(),
        None,
        Ok(()) // without two encoded `=`
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiLyIsIlJlbGF5U3RhdGUiOiIvIiwiRXJyb3JNZ\
         XNzYWdlIjpudWxsfQ%3D%3D".parse().unwrap(),
        None,
        Ok(()) // with two encoded `=`
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiIiwiUmVsYXlTdGF0ZSI6Ii8iLCJFcnJvck1lc\
         3NhZ2UiOm51bGx9".parse().unwrap(),
        None,
        Ok(()) // no padding
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiIiwiUmVsYXlTdGF0ZSI6Ii8iLCJFcnJvck1lc\
         3NhZ2UiOm51bGx9".parse().unwrap(),
        Some(OidcError::StateTokenMismatch),
        Err(DigidError::Oidc(OidcError::StateTokenMismatch))
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjpudWxsLCJSZWxheVN0YXRlIjpudWxsLCJFcnJvc\
         k1lc3NhZ2UiOiJ0aW1lb3V0In0=".parse().unwrap(),
        None,
        Err(DigidError::App2AppError(App2AppErrorMessage::Timeout))
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid".parse().unwrap(),
        None,
        Err(DigidError::MissingLocationQuery)
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?hello".parse().unwrap(),
        None,
        Err(DigidError::UrlDeserialize(serde_urlencoded::de::Error::missing_field("app-app")))
    )]
    // case DigidError::Http is much harder to trigger due to our implementation of BaseUrl and covered by
    // test_start_app2app cases DigidError::ExpectedRedirect, DigidError::MissingLocation,
    // DigidError::NotAUrl and DigidError::HeaderNotAStr in extract_location_header are covered by
    // test_start_app2app
    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request_app2app(
        #[case] redirect_uri: Url,
        #[case] oidc_error: Option<OidcError>,
        #[case] expected: Result<(), DigidError>,
    ) {
        let server = MockServer::start().await;
        let base_url: BaseUrl = server.uri().parse().unwrap();

        Mock::given(method("GET"))
            .and(path("/acs"))
            .respond_with(
                ResponseTemplate::new(StatusCode::TEMPORARY_REDIRECT).insert_header(LOCATION, "https://example.com"),
            )
            .mount(&server)
            .await;

        let mut client = MockOidcClient::default();

        client.expect_into_token_request().return_once(move |_| {
            if let Some(err) = oidc_error {
                return Err(err);
            }

            Ok(default_token_request())
        });

        // same as a call to `HttpDigidSession::start`
        let session = HttpDigidSession {
            session_type: DigidSessionType::App2App(Arc::new(CachedReqwestClient::new())),
            oidc_client: client,
            auth_url: "https://example.com/".parse().unwrap(),
        };

        let token_request = session
            .into_token_request(&InsecureHttpConfig::new(base_url), redirect_uri)
            .await;

        match (token_request, expected) {
            // unfortunately some of the errors don't implement PartialEq
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Ok(o), Err(e)) => panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}"),
            (tr, Ok(())) => {
                tr.unwrap();
            }
        };
    }
}
