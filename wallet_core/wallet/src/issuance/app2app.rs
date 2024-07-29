use std::sync::LazyLock;

use http::{header::LOCATION, StatusCode};
use regex::Regex;
use reqwest::{redirect::Policy, Client, Response};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::{info, warn};
use url::Url;

use openid4vc::{
    oidc::{HttpOidcClient, OidcClient},
    token::TokenRequest,
};
use wallet_common::{
    config::wallet_config::{BaseUrl, PidIssuanceConfiguration, WalletConfiguration},
    reqwest::trusted_reqwest_client_builder,
};

use super::{DigidSession, DigidSessionError};
use crate::config::UNIVERSAL_LINK_BASE_URL;

#[derive(Debug, Serialize)]
pub struct RedirectUrlParameters {
    #[serde(rename = "app-app", with = "json_base64")]
    app_app: DigidJsonRequest,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde_as]
pub struct DigidJsonRequest {
    icon: Url,            // URL to the icon of the app, must be within the same domain as the return URL
    return_url: Url,      // universal link of the wallet app
    host: Option<String>, // on production this must be empty
    #[serde(flatten)]
    saml_parameters: SamlRedirectUrlParameters,
}

// As these parameters are constructed by rdo-max, they are opaque to us and passed as is to DigiD
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde_as]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde_as]
pub struct DigidJsonResponse {
    #[serde(flatten)]
    saml_parameters: SamlReturnUrlParameters,
    #[serde_as(as = "NoneAsEmptyString")]
    error_message: Option<String>,
}

// As these parameters are constructed by DigiD, they are opaque to us and passed as is to rdo-max
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde_as]
pub struct SamlReturnUrlParameters {
    #[serde(rename = "SAMLart")]
    saml_art: String,
    #[serde_as(as = "NoneAsEmptyString")]
    relay_state: Option<String>,
}

struct App2AppSession {
    http_client: Client,
    digid_url: BaseUrl,
}

pub struct HttpDigidSession<OIC = HttpOidcClient> {
    oidc_client: OIC,
    app2app_session: Option<App2AppSession>,
}

impl<OIC> DigidSession for HttpDigidSession<OIC>
where
    OIC: OidcClient,
{
    async fn start(config: PidIssuanceConfiguration, redirect_uri: Url) -> Result<(Self, Url), DigidSessionError>
    where
        Self: Sized,
    {
        // the icon must be within the same domain as the return URL, but the path is stripped off the UL
        const LOGO_PATH: &str = "/.well-known/logo.png";
        static ICON_URL: LazyLock<Url> = LazyLock::new(|| UNIVERSAL_LINK_BASE_URL.as_ref().join(LOGO_PATH).unwrap());

        let trust_anchors = config.digid_trust_anchors();
        let (oidc_client, mut auth_url) = OIC::start(
            trust_anchors.clone(),
            config.digid_url.clone(),
            config.digid_client_id,
            redirect_uri,
        )
        .await?;

        let (app2app_config, auth_url) = match config.digid_app2app {
            Some(digid_app2app) => {
                info!("Constructing DigiD universal link from Auth URL");

                // this enforces DigiD, even if others are supported by rdo-max
                auth_url.query_pairs_mut().append_pair("login_hint", "digid");

                let http_client = trusted_reqwest_client_builder(trust_anchors)
                    .redirect(Policy::none())
                    .build()?;

                info!("Sending get request to Auth URL, expecting an HTTP Redirect");
                let res = http_client.get(auth_url).send().await?.error_for_status()?;
                let location = Self::extract_location_header(&res)?;

                let saml_parameters =
                    serde_urlencoded::from_str(location.query().ok_or(DigidSessionError::MissingLocationQuery)?)?;

                info!("Constructing DigiD universal link from redirect parameters");
                let json_request = DigidJsonRequest {
                    icon: ICON_URL.to_owned(),
                    return_url: WalletConfiguration::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner(),
                    host: digid_app2app.host().map(|h| h.to_string()),
                    saml_parameters,
                };

                let app2app_url = Self::format_app2app_url(digid_app2app.universal_link().to_owned(), json_request)?;

                info!("DigiD app2app URL generated");
                (
                    Some(App2AppSession {
                        http_client,
                        digid_url: config.digid_url,
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

                // this route is standard for rdo-max
                let mut acs_url = config.digid_url.join("acs");
                acs_url.set_query(Some(serde_urlencoded::to_string(app_app.saml_parameters)?.as_str()));

                // pass saml artifact and relay_state to rdo-max, exchange for redirect_uri
                info!("Sending SAML artifact and Relay State to acs URL, expecting a redirect");
                let res = config.http_client.get(acs_url).send().await?.error_for_status()?;
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
    use serde::{
        de::{self, DeserializeOwned},
        ser, Deserializer, Serialize, Serializer,
    };
    use serde_with::{
        base64::{Base64, Standard},
        formats::Padded,
        DeserializeAs, SerializeAs,
    };

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
    use http::StatusCode;
    use rstest::rstest;
    use serde::de::Error;
    use serial_test::serial;
    use wiremock::{
        http::HeaderValue,
        matchers::{method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use openid4vc::{
        oidc::{MockOidcClient, OidcError},
        token::TokenRequestGrantType,
    };
    use wallet_common::config::digid::DigidApp2AppConfiguration;

    use super::*;

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_start_no_app2app() {
        let client = MockOidcClient::start_context();
        client
            .expect()
            .return_once(|_, _, _, _| Ok((MockOidcClient::default(), Url::parse("https://example.com/").unwrap())));

        let config = PidIssuanceConfiguration {
            pid_issuer_url: "https://example.com".parse().unwrap(),
            digid_url: "https://digid.example.com".parse().unwrap(),
            digid_client_id: Default::default(),
            digid_trust_anchors: Default::default(),
            digid_app2app: Default::default(),
        };

        let session = HttpDigidSession::<MockOidcClient>::start(config, "https://app.example.com".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(session.1, "https://example.com/".parse().unwrap());
    }

    #[rstest]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication?SAMLRequest=rZNBj9owEIXv%2Byui3CHBBEIsiETZqkWiLAK2h15Wjj1ZLCV2ak9Yfn7tBLRUqjj1FCWeee%2Bb58ncsrpq6LLFk9rD7xYsPgXBpa6Upd3RImyNoppZaaliNViKnB6WPzaUDGPaGI2a6yr8q%2BlxD7MWDEqtfNP6eRG%2BbL9uXr6tt2%2BTEckyMZ2lMSmztGAJGYNIYz4aT%2BNxTAhLszguZqlv%2FAnGOo1F6CT9exDsjD5LAWbrHBfhASU%2FoVTvwXp1fO2srG1hrSwyha4tJskgngzi2THOKJnSJP3lq55dAlIx7LRPiI2lUdQYcJOK0VDIdymGqor8mJEUTWT60N6YSxCUM2W30XbXbL5IJRzH41CKvsjS78fjbrB0%2BZSMo5dZ3uJaaWXbGswBzFlyeN1vPgHRIQzPWpsCoBIfrKoAPWY5ZiOeiHLME5KwhLNimjE2I7NJOSUTVkbdQAOulQKO2kSM2z7NJaKRRYvQ2zq2q%2B9aCbgswlGYd3VznwTtsjX5%2F8aZR%2FfqnV9v2NDrsoLoVtdBIlwwWOm6YUZaf3mOWdZtHeY94n3dqnJLuIcyf7innHJf5z7v3ONDG%2BGv1IGBOBqmbKMNXgn%2FKd7RRg9w86fb8f3%2Fl%2F8B&RelayState=eyJzdGF0ZSI6ICI5Y2Q3ZTcwMWEyMzJmYWZjMjUzNWM4ODE2MmU4Yjg2MzA4YTRmM2U3MmRjMDc2MmNjNjk2YjQ1YmVlYTdlOTIzIiwgImNsaWVudF9pZCI6ICI1Zjg2Yzg3Yi1jMzViLTQ3M2ItYmZmOS00YTE5NzY3ZWQzZjciLCAicmVkaXJlY3RfdXJpIjogImh0dHBzOi8vYXBwLnRlc3Qudm9vcmJlZWxkd2FsbGV0Lm5sL2RlZXBsaW5rL2F1dGhlbnRpY2F0aW9uIn0%3D&Signature=P7y2OyXiYm7oMGC7d4m7K8gDZFfCWWFwNrDusQfh928T7kHGZob%2F3unQnvwu36EGxc805y8t87riyUgT9sgmWRm9D4u3Rr6eZ5tCcbvR5ERQZvS%2BlNOOMc%2FuHlSv7L3ToXy5uh0tnUFX64L8qXLnPo9MgBBz%2FhOjSY72Pw2u%2BRTU3gmmij4M5%2FqFL4f5J%2FHupeYc3lID6ABLT4QpMinngLQO8F82KqbfJXeLNqQ5pmGUB3nvDfsVmxsNGm1ZMI2ifQqtyDUvGGwXxkpHi7EL%2FdSripWBZ%2FHhJbnPzJkC5KqUxUEi1%2FZ8o4MiwLUHhS3khiMKKxjW6w2tIvpWi0yXDISQJvIfrSVAx24OXqHW3nti41gmndWbviLRH7E7GIB2b9V9qTRMKFv9RCN7oTT%2FsE1MKNNkJ2FA2ePUu4lGYvZHDDf5gpcWr1y5ZiQull1x2nWlvGYkvBCHaWukThax39RC8AOZwixg3UvznMSGisElkDB6V7fqfdMbPXhDIb6%2BdXsAVsG84iJ9qBXmdawO2OgusSMI1i7O8jZyZXM68qg%2FYP8ZJJdwQuQESlOjhZxyUNsTytif9%2FaqCjvb7FC%2B7L16kjPLACCk%2BOLft8JRAw1%2B15MqdNBdg5aSlptdfFEwhI7REyJt8pv8HtlqPi11qGw1vZ%2BIH409nUsfAiXjkgk%3D&SigAlg=http%3A%2F%2Fwww.w3.org%2F2001%2F04%2Fxmldsig-more%23rsa-sha256".as_bytes()).ok(),
        None,
        None,
        Ok("https://app-preprod.example.com/app?app-app=eyJJY29uIjoid2FsbGV0ZGVidWdpbnRlcmFjdGlvbjovL3dhbGxldC5lZGkucmlqa3NvdmVyaGVpZC5ubC8ud2VsbC1rbm93bi9sb2dvLnBuZyIsIlJldHVyblVybCI6IndhbGxldGRlYnVnaW50ZXJhY3Rpb246Ly93YWxsZXQuZWRpLnJpamtzb3ZlcmhlaWQubmwvcmV0dXJuLWZyb20tZGlnaWQiLCJIb3N0IjoicHJlcHJvZC5leGFtcGxlLmNvbSIsIlNBTUxSZXF1ZXN0IjoiclpOQmo5b3dFSVh2K3l1aTNDSEJCRUlzaUVUWnFrV2lMQUsyaDE1V2pqMVpMQ1YyYWs5WWZuN3RCTFJVcWpqMUZDV2VlZStiNThuY3NycHE2TExGazlyRDd4WXNQZ1hCcGE2VXBkM1JJbXlOb3BwWmFhbGlOVmlLbkI2V1B6YVVER1BhR0kyYTZ5cjhxK2x4RDdNV0RFcXRmTlA2ZVJHK2JMOXVYcjZ0dDIrVEVja3lNWjJsTVNtenRHQUpHWU5JWXo0YVQrTnhUQWhMc3pndVpxbHYvQW5HT28xRjZDVDlleERzakQ1TEFXYnJIQmZoQVNVL29WVHZ3WHAxZk8yc3JHMWhyU3d5aGE0dEpza2duZ3ppMlRIT0tKblNKUDNscTU1ZEFsSXg3TFJQaUkybFVkUVljSk9LMFZESWR5bUdxb3I4bUpFVVRXVDYwTjZZU3hDVU0yVzMwWGJYYkw1SUpSekg0MUNLdnNqUzc4ZmpickIwK1pTTW81ZFozdUphYVdYYkdzd0J6Rmx5ZU4xdlBnSFJJUXpQV3BzQ29CSWZyS29BUFdZNVppT2VpSExNRTVLd2hMTmltakUySTdOSk9TVVRWa2JkUUFPdWxRS08ya1NNMno3TkphS1JSWXZRMnpxMnErOWFDYmdzd2xHWWQzVnpud1R0c2pYNS84YVpSL2ZxblY5djJORHJzb0xvVnRkQklsd3dXT202WVVaYWYzbU9XZFp0SGVZOTRuM2RxbkpMdUljeWY3aW5uSEpmNXo3djNPTkRHK0d2MUlHQk9CcW1iS01OWGduL0tkN1JSZzl3ODZmYjhmMy9sLzhCIiwiUmVsYXlTdGF0ZSI6ImV5SnpkR0YwWlNJNklDSTVZMlEzWlRjd01XRXlNekptWVdaak1qVXpOV000T0RFMk1tVTRZamcyTXpBNFlUUm1NMlUzTW1Sak1EYzJNbU5qTmprMllqUTFZbVZsWVRkbE9USXpJaXdnSW1Oc2FXVnVkRjlwWkNJNklDSTFaamcyWXpnM1lpMWpNelZpTFRRM00ySXRZbVptT1MwMFlURTVOelkzWldRelpqY2lMQ0FpY21Wa2FYSmxZM1JmZFhKcElqb2dJbWgwZEhCek9pOHZZWEJ3TG5SbGMzUXVkbTl2Y21KbFpXeGtkMkZzYkdWMExtNXNMMlJsWlhCc2FXNXJMMkYxZEdobGJuUnBZMkYwYVc5dUluMD0iLCJTaWdBbGciOiJodHRwOi8vd3d3LnczLm9yZy8yMDAxLzA0L3htbGRzaWctbW9yZSNyc2Etc2hhMjU2IiwiU2lnbmF0dXJlIjoiUDd5Mk95WGlZbTdvTUdDN2Q0bTdLOGdEWkZmQ1dXRndOckR1c1FmaDkyOFQ3a0hHWm9iLzN1blFudnd1MzZFR3hjODA1eTh0ODdyaXlVZ1Q5c2dtV1JtOUQ0dTNScjZlWjV0Q2NidlI1RVJRWnZTK2xOT09NYy91SGxTdjdMM1RvWHk1dWgwdG5VRlg2NEw4cVhMblBvOU1nQkJ6L2hPalNZNzJQdzJ1K1JUVTNnbW1pajRNNS9xRkw0ZjVKL0h1cGVZYzNsSUQ2QUJMVDRRcE1pbm5nTFFPOEY4MktxYmZKWGVMTnFRNXBtR1VCM252RGZzVm14c05HbTFaTUkyaWZRcXR5RFV2R0d3WHhrcEhpN0VML2RTcmlwV0JaL0hoSmJuUHpKa0M1S3FVeFVFaTEvWjhvNE1pd0xVSGhTM2toaU1LS3hqVzZ3MnRJdnBXaTB5WERJU1FKdklmclNWQXgyNE9YcUhXM250aTQxZ21uZFdidmlMUkg3RTdHSUIyYjlWOXFUUk1LRnY5UkNON29UVC9zRTFNS05Oa0oyRkEyZVBVdTRsR1l2WkhERGY1Z3BjV3IxeTVaaVF1bGwxeDJuV2x2R1lrdkJDSGFXdWtUaGF4MzlSQzhBT1p3aXhnM1V2em5NU0dpc0Vsa0RCNlY3ZnFmZE1iUFhoREliNitkWHNBVnNHODRpSjlxQlhtZGF3TzJPZ3VzU01JMWk3TzhqWnlaWE02OHFnL1lQOFpKSmR3UXVRRVNsT2poWnh5VU5zVHl0aWY5L2FxQ2p2YjdGQys3TDE2a2pQTEFDQ2srT0xmdDhKUkF3MSsxNU1xZE5CZGc1YVNscHRkZkZFd2hJN1JFeUp0OHB2OEh0bHFQaTExcUd3MXZaK0lINDA5blVzZkFpWGprZ2s9In0=".parse().unwrap()),
    )]
    #[case(
        StatusCode::TEMPORARY_REDIRECT,
        HeaderValue::from_bytes("https://preprod.example.com/saml/idp/request_authentication?SAMLRequest=rZNBj9owEIXv%2Byui3CHBBEIsiETZqkWiLAK2h15Wjj1ZLCV2ak9Yfn7tBLRUqjj1FCWeee%2Bb58ncsrpq6LLFk9rD7xYsPgXBpa6Upd3RImyNoppZaaliNViKnB6WPzaUDGPaGI2a6yr8q%2BlxD7MWDEqtfNP6eRG%2BbL9uXr6tt2%2BTEckyMZ2lMSmztGAJGYNIYz4aT%2BNxTAhLszguZqlv%2FAnGOo1F6CT9exDsjD5LAWbrHBfhASU%2FoVTvwXp1fO2srG1hrSwyha4tJskgngzi2THOKJnSJP3lq55dAlIx7LRPiI2lUdQYcJOK0VDIdymGqor8mJEUTWT60N6YSxCUM2W30XbXbL5IJRzH41CKvsjS78fjbrB0%2BZSMo5dZ3uJaaWXbGswBzFlyeN1vPgHRIQzPWpsCoBIfrKoAPWY5ZiOeiHLME5KwhLNimjE2I7NJOSUTVkbdQAOulQKO2kSM2z7NJaKRRYvQ2zq2q%2B9aCbgswlGYd3VznwTtsjX5%2F8aZR%2FfqnV9v2NDrsoLoVtdBIlwwWOm6YUZaf3mOWdZtHeY94n3dqnJLuIcyf7innHJf5z7v3ONDG%2BGv1IGBOBqmbKMNXgn%2FKd7RRg9w86fb8f3%2Fl%2F8B&RelayState=eyJzdGF0ZSI6ICI5Y2Q3ZTcwMWEyMzJmYWZjMjUzNWM4ODE2MmU4Yjg2MzA4YTRmM2U3MmRjMDc2MmNjNjk2YjQ1YmVlYTdlOTIzIiwgImNsaWVudF9pZCI6ICI1Zjg2Yzg3Yi1jMzViLTQ3M2ItYmZmOS00YTE5NzY3ZWQzZjciLCAicmVkaXJlY3RfdXJpIjogImh0dHBzOi8vYXBwLnRlc3Qudm9vcmJlZWxkd2FsbGV0Lm5sL2RlZXBsaW5rL2F1dGhlbnRpY2F0aW9uIn0%3D&Signature=P7y2OyXiYm7oMGC7d4m7K8gDZFfCWWFwNrDusQfh928T7kHGZob%2F3unQnvwu36EGxc805y8t87riyUgT9sgmWRm9D4u3Rr6eZ5tCcbvR5ERQZvS%2BlNOOMc%2FuHlSv7L3ToXy5uh0tnUFX64L8qXLnPo9MgBBz%2FhOjSY72Pw2u%2BRTU3gmmij4M5%2FqFL4f5J%2FHupeYc3lID6ABLT4QpMinngLQO8F82KqbfJXeLNqQ5pmGUB3nvDfsVmxsNGm1ZMI2ifQqtyDUvGGwXxkpHi7EL%2FdSripWBZ%2FHhJbnPzJkC5KqUxUEi1%2FZ8o4MiwLUHhS3khiMKKxjW6w2tIvpWi0yXDISQJvIfrSVAx24OXqHW3nti41gmndWbviLRH7E7GIB2b9V9qTRMKFv9RCN7oTT%2FsE1MKNNkJ2FA2ePUu4lGYvZHDDf5gpcWr1y5ZiQull1x2nWlvGYkvBCHaWukThax39RC8AOZwixg3UvznMSGisElkDB6V7fqfdMbPXhDIb6%2BdXsAVsG84iJ9qBXmdawO2OgusSMI1i7O8jZyZXM68qg%2FYP8ZJJdwQuQESlOjhZxyUNsTytif9%2FaqCjvb7FC%2B7L16kjPLACCk%2BOLft8JRAw1%2B15MqdNBdg5aSlptdfFEwhI7REyJt8pv8HtlqPi11qGw1vZ%2BIH409nUsfAiXjkgk%3D&SigAlg=http%3A%2F%2Fwww.w3.org%2F2001%2F04%2Fxmldsig-more%23rsa-sha256".as_bytes()).ok(),
        None,
        Some("file://etc/passwd".parse().unwrap()),
        Err(DigidSessionError::Http(reqwest::Client::new().get("file://etc/passwd?login_hint=digid").send().await.unwrap_err()))
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
        Err(DigidSessionError::HeaderNotAStr(unsafe { http::header::HeaderValue::from_maybe_shared_unchecked("🦀") }.to_str().unwrap_err()))
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

        let config = PidIssuanceConfiguration {
            pid_issuer_url: "https://example.com".parse().unwrap(),
            digid_url: "https://digid.example.com".parse().unwrap(),
            digid_client_id: Default::default(),
            digid_trust_anchors: Default::default(),
            digid_app2app: Some(DigidApp2AppConfiguration::Preprod {
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
        client.expect().return_once(move |_, _, _, _| {
            if let Some(err) = oidc_error {
                return Err(err);
            }

            Ok((MockOidcClient::default(), base_url.clone()))
        });

        let session =
            HttpDigidSession::<MockOidcClient>::start(config, "https://app.example.com".parse().unwrap()).await;

        match (session.map(|(_, x)| x), expected) {
            (Ok(o), Ok(k)) => assert_eq!(o, k),
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()), // unfortunately some of the errors don't implement PartialEq
            (Err(e), Ok(o)) => panic!("assertion `left == right` failed\n left: {e:?}\nright: {o:?}"),
            (Ok(o), Err(e)) => panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}"),
        };
    }

    #[tokio::test]
    #[serial(MockOidcClient)]
    async fn test_into_token_request_no_app2app() {
        let mut client = MockOidcClient::default();
        client.expect_into_token_request().return_once(|_| {
            Ok(TokenRequest {
                grant_type: TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: "".to_owned().into(),
                },
                code_verifier: Default::default(),
                client_id: Default::default(),
                redirect_uri: Default::default(),
            })
        });

        // same as a call to `HttpDigidSession::start`
        let session = HttpDigidSession::<MockOidcClient> {
            oidc_client: client,
            app2app_session: None,
        };

        let token_request = session
            .into_token_request(
                "https://app.example.com/deeplink/return-from-digid"
                    .parse()
                    .unwrap(),
            )
            .await;

        assert!(token_request.is_ok());
    }

    #[rstest]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiQUFRQUFNMks0c3dQOXdXWk1wU01hd3pteXBcdTAwMkI3NUtFVXZFUGNmWHFIUVZFSG9WWENXNmdsbjdrcDNaZz0iLCJSZWxheVN0YXRlIjoiZXlKemRHRjBaU0k2SUNJNVkyUTNaVGN3TVdFeU16Sm1ZV1pqTWpVek5XTTRPREUyTW1VNFlqZzJNekE0WVRSbU0yVTNNbVJqTURjMk1tTmpOamsyWWpRMVltVmxZVGRsT1RJeklpd2dJbU5zYVdWdWRGOXBaQ0k2SUNJMVpqZzJZemczWWkxak16VmlMVFEzTTJJdFltWm1PUzAwWVRFNU56WTNaV1F6WmpjaUxDQWljbVZrYVhKbFkzUmZkWEpwSWpvZ0ltaDBkSEJ6T2k4dllYQndMblJsYzNRdWRtOXZjbUpsWld4a2QyRnNiR1YwTG01c0wyUmxaWEJzYVc1ckwyRjFkR2hsYm5ScFkyRjBhVzl1SW4wPSIsIkVycm9yTWVzc2FnZSI6bnVsbH0=".parse().unwrap(),
        None,
        Ok(()) // without encoded `=`
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiQUFRQUFNMks0c3dQOXdXWk1wU01hd3pteXBcdTAwMkI3NUtFVXZFUGNmWHFIUVZFSG9WWENXNmdsbjdrcDNaZz0iLCJSZWxheVN0YXRlIjoiZXlKemRHRjBaU0k2SUNJNVkyUTNaVGN3TVdFeU16Sm1ZV1pqTWpVek5XTTRPREUyTW1VNFlqZzJNekE0WVRSbU0yVTNNbVJqTURjMk1tTmpOamsyWWpRMVltVmxZVGRsT1RJeklpd2dJbU5zYVdWdWRGOXBaQ0k2SUNJMVpqZzJZemczWWkxak16VmlMVFEzTTJJdFltWm1PUzAwWVRFNU56WTNaV1F6WmpjaUxDQWljbVZrYVhKbFkzUmZkWEpwSWpvZ0ltaDBkSEJ6T2k4dllYQndMblJsYzNRdWRtOXZjbUpsWld4a2QyRnNiR1YwTG01c0wyUmxaWEJzYVc1ckwyRjFkR2hsYm5ScFkyRjBhVzl1SW4wPSIsIkVycm9yTWVzc2FnZSI6bnVsbH0%3D".parse().unwrap(),
        None,
        Ok(()) // with encoded `=`
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiLyIsIlJlbGF5U3RhdGUiOiIvIiwiRXJyb3JNZXNzYWdlIjpudWxsfQ==".parse().unwrap(),
        None,
        Ok(()) // without two encoded `=`
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiLyIsIlJlbGF5U3RhdGUiOiIvIiwiRXJyb3JNZXNzYWdlIjpudWxsfQ%3D%3D".parse().unwrap(),
        None,
        Ok(()) // with two encoded `=`
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiIiwiUmVsYXlTdGF0ZSI6Ii8iLCJFcnJvck1lc3NhZ2UiOm51bGx9".parse().unwrap(),
        None,
        Ok(()) // no padding
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiIiwiUmVsYXlTdGF0ZSI6Ii8iLCJFcnJvck1lc3NhZ2UiOm51bGx9".parse().unwrap(),
        Some(OidcError::StateTokenMismatch),
        Err(DigidSessionError::Oidc(OidcError::StateTokenMismatch))
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?app-app=eyJTQU1MYXJ0IjoiIiwiUmVsYXlTdGF0ZSI6IiIsIkVycm9yTWVzc2FnZSI6InRpbWVvdXQifQ==".parse().unwrap(),
        None,
        Err(DigidSessionError::App2AppError("timeout".to_owned()))
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid".parse().unwrap(),
        None,
        Err(DigidSessionError::MissingLocationQuery)
    )]
    #[case(
        "https://app.example.com/deeplink/return-from-digid?hello".parse().unwrap(),
        None,
        Err(DigidSessionError::UrlDeserialize(serde_urlencoded::de::Error::missing_field("app-app")))
    )]
    // case DigidSessionError::Http is much harder to trigger due to our implementation of BaseUrl and covered by test_start_app2app
    // cases DigidSessionError::ExpectedRedirect, DigidSessionError::MissingLocation, DigidSessionError::NotAUrl and DigidSessionError::HeaderNotAStr in extract_location_header are covered by test_start_app2app
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

            Ok(TokenRequest {
                grant_type: TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: "".to_owned().into(),
                },
                code_verifier: Default::default(),
                client_id: Default::default(),
                redirect_uri: Default::default(),
            })
        });

        // same as a call to `HttpDigidSession::start`
        let session = HttpDigidSession::<MockOidcClient> {
            oidc_client: client,
            app2app_session: Some(App2AppSession {
                http_client: trusted_reqwest_client_builder(vec![])
                    .redirect(Policy::none())
                    .build()
                    .unwrap(),
                digid_url: base_url,
            }),
        };

        let token_request = session.into_token_request(redirect_uri).await;

        match (token_request, expected) {
            (Err(e), Err(r)) => assert_eq!(e.to_string(), r.to_string()), // unfortunately some of the errors don't implement PartialEq
            (Ok(o), Err(e)) => panic!("assertion `left == right` failed\n left: {o:?}\nright: {e:?}"),
            (tr, Ok(())) => {
                tr.unwrap();
            }
        };
    }

    #[rstest]
    #[case(Some("example.com/no_padding".to_owned()), "https://app.example.com/app?app-app=eyJJY29uIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9sb2dvLnBuZyIsIlJldHVyblVybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vcmV0dXJuIiwiSG9zdCI6ImV4YW1wbGUuY29tL25vX3BhZGRpbmciLCJTQU1MUmVxdWVzdCI6IiIsIlJlbGF5U3RhdGUiOm51bGwsIlNpZ0FsZyI6IiIsIlNpZ25hdHVyZSI6IiJ9")]
    #[case(Some("example.com/padding__".to_owned()), "https://app.example.com/app?app-app=eyJJY29uIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9sb2dvLnBuZyIsIlJldHVyblVybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vcmV0dXJuIiwiSG9zdCI6ImV4YW1wbGUuY29tL3BhZGRpbmdfXyIsIlNBTUxSZXF1ZXN0IjoiIiwiUmVsYXlTdGF0ZSI6bnVsbCwiU2lnQWxnIjoiIiwiU2lnbmF0dXJlIjoiIn0=")]
    #[case(Some("example.com/more___padding".to_owned()), "https://app.example.com/app?app-app=eyJJY29uIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9sb2dvLnBuZyIsIlJldHVyblVybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vcmV0dXJuIiwiSG9zdCI6ImV4YW1wbGUuY29tL21vcmVfX19wYWRkaW5nIiwiU0FNTFJlcXVlc3QiOiIiLCJSZWxheVN0YXRlIjpudWxsLCJTaWdBbGciOiIiLCJTaWduYXR1cmUiOiIifQ==")]
    #[case(None, "https://app.example.com/app?app-app=eyJJY29uIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9sb2dvLnBuZyIsIlJldHVyblVybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vcmV0dXJuIiwiSG9zdCI6bnVsbCwiU0FNTFJlcXVlc3QiOiIiLCJSZWxheVN0YXRlIjpudWxsLCJTaWdBbGciOiIiLCJTaWduYXR1cmUiOiIifQ==")]
    fn test_format_app2app_url(#[case] host: Option<String>, #[case] expected: Url) {
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
}
