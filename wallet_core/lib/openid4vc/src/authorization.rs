use chrono::Duration;
use indexmap::IndexSet;
use jwt::nonce::Nonce;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::DurationSeconds;
use serde_with::SerializeDisplay;
use serde_with::StringWithSeparator;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-authorization-request>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1>.
/// When sent using [PAR (Pushed Authorization Requests)](https://datatracker.ietf.org/doc/html/rfc9126),
/// it is usually sent URL-encoded in the request body to the /par endpoint.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationRequest {
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, ResponseType>")]
    pub response_type: IndexSet<ResponseType>,

    pub client_id: String,
    pub redirect_uri: Option<Url>,
    pub state: Option<String>,
    pub authorization_details: Option<Vec<AuthorizationDetails>>,

    #[serde(flatten)]
    pub code_challenge: Option<PkceCodeChallenge>,

    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, String>>")]
    pub scope: Option<IndexSet<String>>,

    pub nonce: Option<Nonce>,
    pub response_mode: Option<ResponseMode>,
}

/// Represents the response from the /par endpoint containing a `request_uri` that can be used to retrieve the pushed
/// `AuthorizationRequest` later at the /authorize endpoint. Note: this is not a response to the
/// `PushedAuthorizationRequest` defined below.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub expires_in: Duration,
}

/// Represents the parameters that are passed in the query string of the /authorize endpoint where the `request_uri`
/// refers to a pushed `AuthorizationRequest` sent earlier.
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationRequest {
    pub client_id: String,
    pub request_uri: String,
}

/// Defined in https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html#ResponseModes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    Query,
    #[default]
    Fragment,

    // The following two are defined in OpenID4VP
    DirectPost,
    #[serde(rename = "direct_post.jwt")]
    DirectPostJwt,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "code_challenge_method")]
pub enum PkceCodeChallenge {
    S256 {
        code_challenge: String,
    },
    #[serde(rename = "plain")]
    Plain {
        code_challenge: String,
    },
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum ResponseType {
    /// OAuth
    #[default]
    Code,

    /// OpenID4VP
    VpToken,

    /// SIOPv2 (not supported (yet))
    IdToken,
}

impl From<ResponseType> for IndexSet<ResponseType> {
    fn from(value: ResponseType) -> Self {
        IndexSet::from([value])
    }
}

/// Format-specific data for the [`AuthorizationDetails`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum AuthorizationDetailsFormatData {
    MsoMdoc { doctype: String },
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationDetails {
    #[serde(rename = "type")]
    typ: AuthorizationDetailsType,
    credential_identifiers: Option<Vec<String>>,
    #[serde(flatten)]
    format_data: AuthorizationDetailsFormatData,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationDetailsType {
    #[default]
    OpenidCredential,
}

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-successful-authorization-re>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.2>.
/// Contains the token that may be exchanged for an access token.
/// URL-encoded and provided as query parameters added to the `redirect_uri` that was passed in the
/// [`AuthorizationRequest`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;
    use jwt::nonce::Nonce;
    use serde_json::json;
    use serde_urlencoded;
    use url::Url;

    use crate::authorization::AuthorizationDetails;
    use crate::authorization::AuthorizationDetailsFormatData;
    use crate::authorization::AuthorizationRequest;
    use crate::authorization::PkceCodeChallenge;
    use crate::authorization::ResponseMode;
    use crate::authorization::ResponseType;

    #[test]
    fn authorization_request_serialization_roundtrip() {
        let mut response_type = IndexSet::new();
        response_type.insert(ResponseType::Code);

        let mut scope = IndexSet::new();
        scope.insert("openid".to_string());
        scope.insert("profile".to_string());

        let nonce = Nonce::new_random();

        let request = AuthorizationRequest {
            response_type,
            client_id: "client-123".to_string(),
            redirect_uri: Some(Url::parse("https://example.com/callback").unwrap()),
            state: Some("state-abc".to_string()),
            authorization_details: None,
            code_challenge: Some(PkceCodeChallenge::S256 {
                code_challenge: "challenge-xyz".to_string(),
            }),
            scope: Some(scope),
            nonce: Some(nonce.clone()),
            response_mode: Some(ResponseMode::Fragment),
        };

        let encoded = serde_urlencoded::to_string(&request).unwrap();
        let decoded: AuthorizationRequest = serde_urlencoded::from_str(&encoded).unwrap();

        assert_eq!(decoded.client_id, "client-123");
        assert_eq!(decoded.state.as_deref(), Some("state-abc"));
        assert_eq!(decoded.nonce, Some(nonce));
        assert_eq!(
            decoded.scope.unwrap().iter().cloned().collect::<Vec<_>>(),
            vec!["openid", "profile"]
        );
    }

    #[test]
    fn authorization_details_serialization() {
        assert_eq!(
            serde_json::to_string(&AuthorizationDetails {
                typ: crate::authorization::AuthorizationDetailsType::OpenidCredential,
                credential_identifiers: None,
                format_data: AuthorizationDetailsFormatData::MsoMdoc {
                    doctype: "example_doctype".to_string()
                }
            })
            .unwrap(),
            json!({"type": "openid_credential","format": "mso_mdoc","doctype": "example_doctype"}).to_string(),
        );
    }
}
