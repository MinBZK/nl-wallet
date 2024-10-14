use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::{formats::SpaceSeparator, serde_as, skip_serializing_none, StringWithSeparator};
use url::Url;

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-authorization-request>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1>.
/// When sent using [PAR (Pushed Authorization Requests)](https://datatracker.ietf.org/doc/html/rfc9126),
/// it is usually sent URL-encoded in the request body to POST /op/par.
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

    /// https://datatracker.ietf.org/doc/html/rfc9126. MUST NOT be sent in a PAR.
    /// This is a `String` and not a `Url`, because despite its name it need not be an actual URL;
    /// its contents is completely up to the server and to be considered opaque.
    pub request_uri: Option<String>,

    #[serde(flatten)]
    pub code_challenge: Option<PkceCodeChallenge>,

    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, String>>")]
    pub scope: Option<IndexSet<String>>,

    pub nonce: Option<String>,
    pub response_mode: Option<ResponseMode>,
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
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumString, strum::Display,
)]
#[serde(rename_all = "snake_case")]
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
    use serde_json::json;

    use crate::authorization::{AuthorizationDetails, AuthorizationDetailsFormatData};

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
