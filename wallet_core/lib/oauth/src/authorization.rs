use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use chrono::Duration;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::StringWithSeparator;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use utils::spec::SpecForbidden;

/// The shared [OAuth 2.0 RFC 6749](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.1) fields that any
/// authorization request must carry. Generic over the `response_type` value, since supported response types vary
/// per profile (e.g. plain OAuth `code`, OpenID4VP's `vp_token`, SIOPv2's `id_token`).
///
/// Flow-specific request types embed this with `#[serde(flatten)]` and add their own fields.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizationRequestBase<T>
where
    T: Display + FromStr + Eq + Hash + Clone + Debug,
    T::Err: Display,
{
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, T>")]
    pub response_type: HashSet<T>,

    pub client_id: String,
    pub state: Option<String>,

    // Should not be present for PAR and openid4vp.
    #[serde(default, skip_serializing, rename = "request_uri")]
    _request_uri: SpecForbidden,
}

impl<T> AuthorizationRequestBase<T>
where
    T: Display + FromStr + Eq + Hash + Clone + Debug,
    T::Err: Display,
{
    pub fn new(response_type: HashSet<T>, client_id: String, state: Option<String>) -> Self {
        Self {
            response_type,
            client_id,
            state,
            _request_uri: SpecForbidden,
        }
    }
}

/// The OAuth 2.0 Authorization Response, which is URL-encoded and provided as query parameters added to the
/// `redirect_uri` that was passed in the Authorization Request.
///
/// See: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

/// Represents the response from the `/par` endpoint containing a `request_uri` that can be used to retrieve the
/// pushed authorization request later at the `/authorize` endpoint, as defined by
/// [RFC 9126](https://www.rfc-editor.org/rfc/rfc9126.html#section-2.2).
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub expires_in: Duration,
}

/// Represents the parameters that are passed in the query string of the `/authorize` endpoint where the
/// `request_uri` refers to a pushed authorization request sent earlier, as defined by
/// [RFC 9126](https://www.rfc-editor.org/rfc/rfc9126.html#section-4).
#[derive(Serialize, Deserialize, Debug)]
pub struct PushedAuthorizationRequest {
    pub client_id: String,
    pub request_uri: String,
}
