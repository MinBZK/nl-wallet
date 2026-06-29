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
use crate::issuable_document::CredentialKind;
use crate::issuable_document::IssuableDocument;
use crate::issuer::AuthRequestValues;

/// Represents the wallet-side parameters the `openid4vc` layer extracts from a [`VciAuthorizationRequest`] and that an
/// [`AuthorizationCodeFlow`] must retain to complete the authorization later: the wallet's
/// `redirect_uri` and `state` (to build the wallet-facing redirect), the `scope` values and its PKCE `code_challenge`
/// (which the `/token` handler verifies the wallet's `code_verifier` against) and the `issuer_state`
/// the wallet echoes back from the credential offer (which a flow may use to identify the context set up during
/// previous process steps).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletAuthorizationContext {
    pub state: Option<String>,
    pub issuer_state: Option<String>,

    // Represents those values present in the inciting Authorization Request that an implementor of
    // [`AuthorizationCodeFlow`] will need to pass to `AuthorizingIssuer::complete_authorization()`. These values are
    // then retained in the `AuthCodeIssued` statue of the `Issuer`.
    #[serde(flatten)]
    pub request_values: AuthRequestValues,
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
            state: request.oauth_request.state,
            issuer_state: request.issuer_state,
            request_values: AuthRequestValues {
                client_id: request.oauth_request.client_id,
                redirect_uri: request.redirect_uri.into_inner(),
                code_challenge,
                scope: request.scope,
            },
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

    /// Called after the `openid4vc` layer has consumed the PAR entry, resolved the original authorization request and
    /// extracted the wallet-side [`WalletAuthorizationContext`]. The implementation decides how the user authenticates
    /// and returns the protocol-level outcome. Part of the contract of this method is that any `IssuableDocument` that
    /// is returned (directly or indirectly) adheres to the requested combinations of format and attestation type.
    async fn authorize(
        &self,
        context: WalletAuthorizationContext,
        credential_kinds: VecNonEmpty<CredentialKind>,
    ) -> Result<AuthorizeOutcome, Self::Error>;

    /// Removes any expired state this flow owns.
    ///
    /// Called periodically by the authorization-phase cleanup task. Defaults to a no-op for flows that have no
    /// expiring state of their own.
    fn cleanup(&self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }
}
