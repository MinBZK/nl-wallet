use std::sync::LazyLock;

use http::header::LOCATION;
use http::StatusCode;
use regex::Regex;
use reqwest::redirect::Policy;
use reqwest::Client;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DeserializeFromStr;
use serde_with::NoneAsEmptyString;
use tracing::info;
use tracing::warn;
use url::Url;

use wallet_configuration::wallet_config::DigidConfiguration;
use openid4vc::oidc::HttpOidcClient;
use openid4vc::oidc::OidcClient;
use openid4vc::token::TokenRequest;
use wallet_common::reqwest::JsonReqwestBuilder;
use wallet_common::urls;

use crate::config::UNIVERSAL_LINK_BASE_URL;

use super::DigidSession;
use super::DigidSessionError;

#[derive(Debug, Serialize)]
pub struct RedirectUrlParameters {
    #[serde(rename = "app-app", with = "json_base64")]
    app_app: DigidJsonRequest,
}

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DigidJsonRequest {
    icon: Url,            // URL to the icon of the app, must be within the same domain as the return URL
    return_url: Url,      // universal link of the wallet app
    host: Option<String>, // on production this must be empty
    #[serde(flatten)]
    saml_parameters: SamlRedirectUrlParameters,
}

// As these parameters are constructed by rdo-max, they are opaque to us and passed as is to DigiD
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SamlRedirectUrlParameters {
    #[serde(rename = "SAMLRequest")]
    saml_request: String,
    relay_state: Option<String>,
    sig_alg: String,   // technically optional
    signature: String, // technically optional
}

#[derive(Debug, Deserialize)]
pub struct ReturnUrlParameters {
    #[serde(rename = "app-app", with = "json_base64")]
    app_app: DigidJsonResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum App2AppErrorMessage {
    #[strum(to_string = "_by_user")]
    ByUser,
    NotActivated,
    Timeout,
    IconMissing,
    #[strum(default)]
    Other(String),
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DigidJsonResponse {
    #[serde(flatten)]
    saml_parameters: SamlReturnUrlParameters,
    #[serde_as(as = "NoneAsEmptyString")]
    error_message: Option<App2AppErrorMessage>,
}

// As these parameters are constructed by DigiD, they are opaque to us and passed as is to rdo-max
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SamlReturnUrlParameters {
    #[serde(rename = "SAMLart")]
    #[serde_as(as = "NoneAsEmptyString")]
    saml_art: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    relay_state: Option<String>,
}

struct App2AppSession {
    http_client: Client,
    acs_request: reqwest::RequestBuilder,
}

pub struct HttpDigidSession<OIC = HttpOidcClient> {
    oidc_client: OIC,
    app2app_session: Option<App2AppSession>,
}

impl<OIC> DigidSession for HttpDigidSession<OIC>
where
    OIC: OidcClient,
{
    async fn start<C>(
        digid_config: DigidConfiguration,
        http_config: &C,
        redirect_uri: Url,
    ) -> Result<(Self, Url), DigidSessionError>
    where
        Self: Sized,
        C: JsonReqwestBuilder + 'static,
    {
        // the icon must be within the same domain as the return URL, but the path is stripped off the UL
        const LOGO_PATH: &str = "/.well-known/logo.png";
        static ICON_URL: LazyLock<Url> = LazyLock::new(|| UNIVERSAL_LINK_BASE_URL.as_ref().join(LOGO_PATH).unwrap());

        let (oidc_client, mut auth_url) = OIC::start(http_config, digid_config.client_id, redirect_uri).await?;

        let (app2app_config, auth_url) = match digid_config.app2app {
            Some(digid_app2app) => {
                info!("Constructing DigiD universal link from Auth URL");

                // this enforces DigiD, even if others are supported by rdo-max
                auth_url.query_pairs_mut().append_pair("login_hint", "digid");

                let http_client = http_config.builder().redirect(Policy::none()).build()?;

                info!("Sending get request to Auth URL, expecting an HTTP Redirect");
                let res = http_client.get(auth_url).send().await?.error_for_status()?;
                let location = Self::extract_location_header(&res)?;

                let saml_parameters =
                    serde_urlencoded::from_str(location.query().ok_or(DigidSessionError::MissingLocationQuery)?)?;

                info!("Constructing DigiD universal link from redirect parameters");
                let json_request = DigidJsonRequest {
                    icon: ICON_URL.to_owned(),
                    return_url: urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner(),
                    host: digid_app2app.host().map(|h| h.to_string()),
                    saml_parameters,
                };

                let app2app_url = Self::format_app2app_url(digid_app2app.universal_link().to_owned(), json_request)?;

                // this route is standard for rdo-max
                let acs_request = http_config.get_with_client(&http_client, "acs");

                info!("DigiD app2app URL generated");
                (
                    Some(App2AppSession {
                        http_client,
                        acs_request,
                    }),
                    app2app_url,
                )
            }
            _ => (None, auth_url),
        };

        let session = (
            Self {
                oidc_client,
                app2app_session: app2app_config,
            },
            auth_url,
        );
        Ok(session)
    }

    async fn into_token_request(self, received_redirect_uri: Url) -> Result<TokenRequest, DigidSessionError> {
        let location = match self.app2app_session {
            None => received_redirect_uri,
            Some(config) => {
                info!("Constructing redirect_uri from JsonResponse");

                let ReturnUrlParameters { app_app } = serde_urlencoded::from_str(
                    received_redirect_uri
                        .query()
                        .ok_or(DigidSessionError::MissingLocationQuery)?,
                )?;

                if let Some(error) = app_app.error_message {
                    warn!("Error message in JsonResponse: {error}");
                    return Err(DigidSessionError::App2AppError(error));
                }

                let acs_request = config.acs_request.query(&app_app.saml_parameters).build()?;

                // pass saml artifact and relay_state to rdo-max, exchange for redirect_uri
                info!("Sending SAML artifact and Relay State to acs URL, expecting a redirect");
                let res = config.http_client.execute(acs_request).await?.error_for_status()?;

                Self::extract_location_header(&res)?
            }
        };

        let token_request = self.oidc_client.into_token_request(&location)?;
        Ok(token_request)
    }
}

impl<OIC> HttpDigidSession<OIC> {
    fn extract_location_header(res: &Response) -> Result<Url, DigidSessionError> {
        if res.status() != StatusCode::TEMPORARY_REDIRECT {
            return Err(DigidSessionError::ExpectedRedirect(res.status()));
        }

        info!("Received redirect, extracting URL from location header");
        let url = res
            .headers()
            .get(LOCATION)
            .ok_or(DigidSessionError::MissingLocation)?
            .to_str()?
            .parse()?;

        Ok(url)
    }

    fn format_app2app_url(mut app2app_url: Url, json_request: DigidJsonRequest) -> Result<Url, DigidSessionError> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%3[dD]").unwrap());
        app2app_url.set_query(Some(
            RE.replace_all(
                &serde_urlencoded::to_string(RedirectUrlParameters { app_app: json_request })?,
                "=",
            )
            .as_ref(), // DigiD fails to properly decode the URL parameter
        ));

        Ok(app2app_url)
    }
}

mod json_base64 {
    use serde::de;
    use serde::de::DeserializeOwned;
    use serde::ser;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use serde_with::base64::Base64;
    use serde_with::base64::Standard;
    use serde_with::formats::Padded;
    use serde_with::DeserializeAs;
    use serde_with::SerializeAs;

    pub fn serialize<S: Serializer, T: Serialize>(input: T, serializer: S) -> Result<S::Ok, S::Error> {
        Base64::<Standard, Padded>::serialize_as(&serde_json::to_vec(&input).map_err(ser::Error::custom)?, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: DeserializeOwned>(deserializer: D) -> Result<T, D::Error> {
        let x: Vec<u8> = Base64::<Standard, Padded>::deserialize_as(deserializer)?;
        serde_json::from_slice(&x).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use base64::prelude::*;
    use http::StatusCode;
    use rstest::rstest;
    use serde::de::Error;
    use serde_json::json;
    use serial_test::serial;
    use wiremock::http::HeaderValue;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::matchers::query_param;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    use wallet_configuration::digid::DigidApp2AppConfiguration;
    use wallet_configuration::wallet_config::DigidConfiguration;
    use openid4vc::oidc::MockOidcClient;
    use openid4vc::oidc::OidcError;
    use openid4vc::token::TokenRequestGrantType;
    use wallet_common::http::test::HttpConfig;
    use wallet_common::reqwest::default_reqwest_client_builder;
    use wallet_common::urls::BaseUrl;

    use super::*;

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
        let client = MockOidcClient::start_context();
        client.expect().return_once(|_: &HttpConfig, _, _| {
            Ok((MockOidcClient::default(), Url::parse("https://example.com/").unwrap()))
        });

        let session = HttpDigidSession::<MockOidcClient>::start(
            DigidConfiguration::default(),
            &HttpConfig {
                base_url: "https://digid.example.com".parse().unwrap(),
            },
            "https://app.example.com".parse().unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(session.1, "https://example.com/".parse().unwrap());
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
        Err(DigidSessionError::Http(
            reqwest::Client::new().get("file://etc/passwd?login_hint=digid").send().await.unwrap_err()
        ))
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        None,
        Some(OidcError::NoAuthCode),
        None,
        Err(DigidSessionError::Oidc(OidcError::NoAuthCode))
    )]
    #[case(
        StatusCode::OK,
        None,
        None,
        None,
        Err(DigidSessionError::ExpectedRedirect(StatusCode::OK))
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        None,
        None,
        None,
        Err(DigidSessionError::MissingLocation)
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("not_a_url".as_bytes()).ok(),
        None,
        None,
        Err(DigidSessionError::NotAUrl(url::ParseError::RelativeUrlWithoutBase))
    )]
    // this case is impossible to test without using unsafe, also the error cannot be created because of private fields
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        Some(unsafe { HeaderValue::from_maybe_shared_unchecked("🦀") }),
        None,
        None,
        Err(DigidSessionError::HeaderNotAStr(
            unsafe { http::header::HeaderValue::from_maybe_shared_unchecked("🦀") }.to_str().unwrap_err()
        ))
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication".as_bytes()).ok(),
        None,
        None,
        Err(DigidSessionError::MissingLocationQuery)
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication?hello".as_bytes()).ok(),
        None,
        None,
        Err(DigidSessionError::UrlDeserialize(serde_urlencoded::de::Error::missing_field("SAMLRequest")))
    )]
    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_start_app2app(
        #[case] status: StatusCode,
        #[case] location: Option<HeaderValue>,
        #[case] oidc_error: Option<OidcError>,
        #[case] auth_url: Option<Url>,
        #[case] expected: Result<Url, DigidSessionError>,
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
        client.expect().return_once(move |_: &HttpConfig, _, _| {
            if let Some(err) = oidc_error {
                return Err(err);
            }

            Ok((MockOidcClient::default(), auth_url))
        });

        let session = HttpDigidSession::<MockOidcClient>::start(
            digid_config,
            &HttpConfig {
                base_url: "https://digid.example.com".parse().unwrap(),
            },
            "https://app.example.com".parse().unwrap(),
        )
        .await;

        match (session.map(|(_, x)| x), expected) {
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
        let session = HttpDigidSession::<MockOidcClient> {
            oidc_client: client,
            app2app_session: None,
        };

        let token_request = session
            .into_token_request("https://example.com/deeplink/return-from-digid".parse().unwrap())
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
        Err(DigidSessionError::Oidc(OidcError::StateTokenMismatch))
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjpudWxsLCJSZWxheVN0YXRlIjpudWxsLCJFcnJvc\
         k1lc3NhZ2UiOiJ0aW1lb3V0In0=".parse().unwrap(),
        None,
        Err(DigidSessionError::App2AppError(App2AppErrorMessage::Timeout))
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid".parse().unwrap(),
        None,
        Err(DigidSessionError::MissingLocationQuery)
    )]
    #[case(
        "https://example.com/deeplink/return-from-digid?hello".parse().unwrap(),
        None,
        Err(DigidSessionError::UrlDeserialize(serde_urlencoded::de::Error::missing_field("app-app")))
    )]
    // case DigidSessionError::Http is much harder to trigger due to our implementation of BaseUrl and covered by
    // test_start_app2app cases DigidSessionError::ExpectedRedirect, DigidSessionError::MissingLocation,
    // DigidSessionError::NotAUrl and DigidSessionError::HeaderNotAStr in extract_location_header are covered by
    // test_start_app2app
    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request_app2app(
        #[case] redirect_uri: Url,
        #[case] oidc_error: Option<OidcError>,
        #[case] expected: Result<(), DigidSessionError>,
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

        let http_client = default_reqwest_client_builder()
            .redirect(Policy::none())
            .build()
            .unwrap();
        let acs_request = http_client.get(base_url.join("acs"));

        // same as a call to `HttpDigidSession::start`
        let session = HttpDigidSession::<MockOidcClient> {
            oidc_client: client,
            app2app_session: Some(App2AppSession {
                http_client,
                acs_request,
            }),
        };

        let token_request = session.into_token_request(redirect_uri).await;

        match (token_request, expected) {
            // unfortunately some of the errors don't implement PartialEq
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()),
            (Ok(o), Err(e)) => panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}"),
            (tr, Ok(())) => {
                tr.unwrap();
            }
        };
    }

    #[rstest]
    #[case(
        Some("example.com/no_padding".to_owned()),
        "https://app.example.com/app?app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": "example.com/no_padding",
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "", "Signature": ""
        }))
    )]
    #[case(
        Some("example.com/padding__".to_owned()),
        "https://app.example.com/app?app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": "example.com/padding__",
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "",
            "Signature": ""
        }))
    )]
    #[case(
        Some("example.com/more___padding".to_owned()),
        "https://app.example.com/app?app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": "example.com/more___padding",
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "",
            "Signature": ""
        }))
    )]
    #[case(
        None,
        "https://app.example.com/app?app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": null,
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "",
            "Signature": ""
        }))
    )]
    fn test_format_app2app_url(#[case] host: Option<String>, #[case] expected: String) {
        let expected = expected.parse().unwrap();
        let url = HttpDigidSession::<HttpOidcClient>::format_app2app_url(
            "https://app.example.com/app".parse().unwrap(),
            DigidJsonRequest {
                icon: "https://example.com/logo.png".parse().unwrap(),
                return_url: "https://example.com/return".parse().unwrap(),
                host,
                saml_parameters: SamlRedirectUrlParameters {
                    saml_request: Default::default(),
                    relay_state: Default::default(),
                    sig_alg: Default::default(),
                    signature: Default::default(),
                },
            },
        )
        .unwrap();

        assert_eq!(url, expected);
    }

    #[rstest]
    #[case(
        base64(json!({"SAMLart": null, "RelayState": null, "ErrorMessage": "_by_user"})),
        Some(App2AppErrorMessage::ByUser)
    )]
    #[case(
        base64(json!({"SAMLart": null, "RelayState": null, "ErrorMessage": "timeout"})),
        Some(App2AppErrorMessage::Timeout)
    )]
    #[case(
        base64(json!({"SAMLart": null, "RelayState": null, "ErrorMessage": "not_activated"})),
        Some(App2AppErrorMessage::NotActivated)
    )]
    #[case(
        base64(json!({"SAMLart": null, "RelayState": null, "ErrorMessage": "icon_missing"})),
        Some(App2AppErrorMessage::IconMissing)
    )]
    #[case(
        base64(json!({"SAMLart": null, "RelayState": null, "ErrorMessage": "Hello, World!"})),
        Some(App2AppErrorMessage::Other("Hello, World!".to_owned()))
    )]
    #[case(base64(json!({"SAMLart": "", "RelayState": "/", "ErrorMessage": null})), None)]
    fn test_digid_app2app_error_message(#[case] input: String, #[case] expected: Option<App2AppErrorMessage>) {
        let res: ReturnUrlParameters = serde_urlencoded::from_str(&format!("app-app={}", input)).unwrap();
        assert_eq!(res.app_app.error_message, expected);
    }

    fn base64<T: Serialize>(input: T) -> String {
        BASE64_URL_SAFE.encode(serde_json::to_string(&input).unwrap())
    }
}
