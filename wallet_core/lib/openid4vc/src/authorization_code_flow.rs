//! The [`AuthorizationCodeFlow`] trait is the issuer-side abstraction over a single
//! OAuth 2.0 authorization-code grant: producing the protocol-level response at `/authorize`
//! (a redirect, or an authorization code) and, later, resolving the authorization code into
//! the documents to issue at `/token`. Any state that needs to survive between the two endpoints
//! is private to the impl, so the `openid4vc` layer stays free of specific flow-related concerns.
//! [`AuthorizingIssuer`](crate::authorizing_issuer::AuthorizingIssuer) is generic over this trait and delegates both
//! endpoints to the configured impl.

use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization::PkceCodeChallenge;
use crate::authorization::VciAuthorizationRequest;
use crate::issuable_document::IssuableDocument;

/// Represents the wallet-side parameters the `openid4vc` layer extracts from a [`VciAuthorizationRequest`] and that an
/// [`AuthorizationCodeFlow`] must retain to complete the authorization later: the wallet's
/// `redirect_uri` and `state` (to build the wallet-facing redirect) and its PKCE `code_challenge`
/// (which the `/token` handler verifies the wallet's `code_verifier` against).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletAuthorizationContext {
    pub redirect_uri: Url,
    pub state: Option<String>,
    pub code_challenge: String,
}

/// Reasons a [`VciAuthorizationRequest`] cannot be accepted into a [`WalletAuthorizationContext`].
#[derive(Debug, thiserror::Error)]
pub enum InvalidAuthorizationRequest {
    #[error("unsupported code_challenge_method: only S256 is supported")]
    UnsupportedCodeChallenge,
}

impl WalletAuthorizationContext {
    /// Build the wallet-side context from an authorization request, rejecting requests we can't
    /// support.
    pub fn try_from_request(request: VciAuthorizationRequest) -> Result<Self, InvalidAuthorizationRequest> {
        let code_challenge = match request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge,
            PkceCodeChallenge::Plain { .. } => return Err(InvalidAuthorizationRequest::UnsupportedCodeChallenge),
        };

        Ok(Self {
            redirect_uri: request.redirect_uri.into_inner(),
            state: request.oauth_request.state,
            code_challenge,
        })
    }
}

/// Defines what the `openid4vc` layer should do in response to `/authorize`, expressed at the protocol level. The
/// `openid4vc_server` HTTP layer turns each variant into the corresponding 302 redirect.
#[derive(Debug, Clone)]
pub enum AuthorizeOutcome {
    /// Send the user-agent to this URL (e.g. an external identity provider). The impl is
    /// responsible for whatever callback / state mechanism eventually turns this round-trip
    /// into an authorization code presentable at `/token`; that mechanism is impl-private and
    /// not modelled by this trait.
    RedirectTo(Url),

    /// Represents the state where the holder is authorized synchronously (no external round-trip) given the issuable
    /// documents and authorization context required to create a new session and redirect the wallet back to its
    /// `redirect_uri` with the code (and echoed `state`).
    Authorized(VecNonEmpty<IssuableDocument>, WalletAuthorizationContext),
}

#[trait_variant::make(Send)]
pub trait AuthorizationCodeFlow {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Called after the `openid4vc` layer has consumed the PAR entry, resolved the original authorization
    /// request and extracted the wallet-side [`WalletAuthorizationContext`]. The implementation
    /// decides how the user authenticates and returns the protocol-level outcome.
    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error>;
}
