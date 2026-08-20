use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
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
