//! The [`AuthorizationCodeFlow`] trait is the issuer-side abstraction over a single
//! OAuth 2.0 authorization-code grant: producing the protocol-level response at `/authorize`
//! (a redirect, or an authorization code) and, later, resolving the authorization code into
//! the documents to issue at `/token`. Any state that needs to survive between the two endpoints
//! is private to the impl, so the `openid4vc` layer stays free of specific flow-related concerns.
//! [`AuthorizingIssuer`](crate::authorizing_issuer::AuthorizingIssuer) is generic over this trait and delegates both
//! endpoints to the configured impl.

use url::Url;

use crate::authorization::VciAuthorizationRequest;
use crate::token::AuthorizationCode;

/// Defines what the framework should do in response to `/authorize`, expressed at the protocol level. The
/// `openid4vc_server` HTTP layer turns each variant into the corresponding 302 redirect.
pub enum AuthorizeOutcome {
    /// Send the user-agent to this URL (e.g. an external identity provider). The impl is
    /// responsible for whatever callback / state mechanism eventually turns this round-trip
    /// into an authorization code presentable at `/token`; that mechanism is impl-private and
    /// not modelled by this trait.
    RedirectTo(Url),

    /// The impl produced an authorization code with no external round-trip. The framework
    /// redirects the wallet back to the original `redirect_uri` with this code (and the
    /// echoed `state`).
    IssuedCode(AuthorizationCode),
}

#[trait_variant::make(Send)]
pub trait AuthorizationCodeFlow {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Called after the framework has consumed the PAR entry and resolved the original
    /// authorization request. The implementation decides how the user authenticates and
    /// returns the protocol-level outcome. Anything the impl needs after this point -- an
    /// external callback, an issuer-generated code, the issuance session itself -- is the impl's
    /// responsibility and is not modelled by this trait.
    async fn authorize(&self, request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error>;
}
