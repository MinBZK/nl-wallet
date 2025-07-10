use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DeserializeFromStr;
use serde_with::NoneAsEmptyString;
use url::Url;

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

#[derive(Debug, Serialize)]
pub(super) struct RedirectUrlParameters {
    #[serde(rename = "app-app", with = "json_base64")]
    pub app_app: DigidJsonRequest,
}

#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct DigidJsonRequest {
    // URL to the icon of the app, must be within the same domain as the return URL
    pub icon: Url,
    // universal link of the wallet app
    pub return_url: Url,
    // on production this must be empty
    pub host: Option<String>,
    #[serde(flatten)]
    pub saml_parameters: SamlRedirectUrlParameters,
}

// As these parameters are constructed by rdo-max, they are opaque to us and passed as is to DigiD
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct SamlRedirectUrlParameters {
    #[serde(rename = "SAMLRequest")]
    pub saml_request: String,
    pub relay_state: Option<String>,
    // technically optional
    pub sig_alg: String,
    // technically optional
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ReturnUrlParameters {
    #[serde(rename = "app-app", with = "json_base64")]
    pub app_app: DigidJsonResponse,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct DigidJsonResponse {
    #[serde(flatten)]
    pub saml_parameters: SamlReturnUrlParameters,
    #[serde_as(as = "NoneAsEmptyString")]
    pub error_message: Option<App2AppErrorMessage>,
}

// As these parameters are constructed by DigiD, they are opaque to us and passed as is to rdo-max
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct SamlReturnUrlParameters {
    #[serde(rename = "SAMLart")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub saml_art: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub relay_state: Option<String>,
}

pub(super) fn format_app2app_query(json_request: DigidJsonRequest) -> Result<String, serde_urlencoded::ser::Error> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%3[dD]").unwrap());

    let query = serde_urlencoded::to_string(RedirectUrlParameters { app_app: json_request })?;

    // DigiD fails to properly decode the URL parameter
    let modified_query = match RE.replace_all(&query, "=") {
        Cow::Borrowed(_) => query,
        Cow::Owned(modified_query) => modified_query,
    };

    Ok(modified_query)
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
    use rstest::rstest;
    use serde_json::json;

    use super::super::test::base64;
    use super::format_app2app_query;
    use super::App2AppErrorMessage;
    use super::DigidJsonRequest;
    use super::ReturnUrlParameters;
    use super::SamlRedirectUrlParameters;

    #[rstest]
    #[case(
        Some("example.com/no_padding".to_string()),
        "app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": "example.com/no_padding",
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "", "Signature": ""
        }))
    )]
    #[case(
        Some("example.com/padding__".to_string()),
        "app-app=".to_string() + &base64(json!({
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
        Some("example.com/more___padding".to_string()),
        "app-app=".to_string() + &base64(json!({
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
        "app-app=".to_string() + &base64(json!({
            "Icon": "https://example.com/logo.png",
            "ReturnUrl": "https://example.com/return",
            "Host": null,
            "SAMLRequest": "",
            "RelayState": null,
            "SigAlg": "",
            "Signature": ""
        }))
    )]
    fn test_format_app2app_query(#[case] host: Option<String>, #[case] expected: String) {
        let query = format_app2app_query(DigidJsonRequest {
            icon: "https://example.com/logo.png".parse().unwrap(),
            return_url: "https://example.com/return".parse().unwrap(),
            host,
            saml_parameters: SamlRedirectUrlParameters {
                saml_request: Default::default(),
                relay_state: Default::default(),
                sig_alg: Default::default(),
                signature: Default::default(),
            },
        })
        .unwrap();

        assert_eq!(query, expected);
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
        Some(App2AppErrorMessage::Other("Hello, World!".to_string()))
    )]
    #[case(base64(json!({"SAMLart": "", "RelayState": "/", "ErrorMessage": null})), None)]
    fn test_digid_app2app_error_message(#[case] input: String, #[case] expected: Option<App2AppErrorMessage>) {
        let res: ReturnUrlParameters = serde_urlencoded::from_str(&format!("app-app={input}")).unwrap();
        assert_eq!(res.app_app.error_message, expected);
    }
}
