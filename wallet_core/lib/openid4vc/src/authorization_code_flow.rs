//! The [`AuthorizationCodeFlow`] trait is the issuer-side abstraction over a single
//! OAuth 2.0 authorization-code grant: producing the protocol-level response at `/authorize`
//! (a redirect, or an authorization code) and, later, resolving the authorization code into
//! the documents to issue at `/token`. Any state that needs to survive between the two endpoints
//! is private to the impl, so the `openid4vc` layer stays free of specific flow-related concerns.
//! [`AuthorizingIssuer`](crate::authorizing_issuer::AuthorizingIssuer) is generic over this trait and delegates both
//! endpoints to the configured impl.
//!
//! The two endpoints are separated by a user-agent round-trip whose duration the issuer does not control, so several
//! independent timeouts (the PAR entry, this flow's own bridged state, and any upstream identity-provider session) can
//! each expire mid-flow.

use std::collections::HashSet;

use attestation_types::credential_kind::CredentialKind;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization::PkceCodeChallenge;
use crate::authorization::VciAuthorizationRequest;
use crate::credential_configurations::CredentialConfigurations;
use crate::errors::AuthorizationErrorCode;
use crate::errors::ErrorWithCode;
use crate::issuable_document::IssuableDocument;
use crate::issuer::AuthRequestValues;
use crate::scope::Scope;

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
    pub credential_kinds: HashSet<CredentialKind>,

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

    #[error("none of the scopes requested reference a known credential configuration: {}", .0.iter().join(" "))]
    NoValidScope(HashSet<Scope>),
}

impl WalletAuthorizationContext {
    /// Build the wallet-side context from an authorization request, rejecting requests we can't
    /// support.
    pub(crate) fn try_from_request<K, L>(
        request: VciAuthorizationRequest,
        credential_configs: &CredentialConfigurations<K, L>,
    ) -> Result<Self, InvalidAuthorizationRequest> {
        let code_challenge = match request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge,
            PkceCodeChallenge::Plain { .. } => return Err(InvalidAuthorizationRequest::UnsupportedCodeChallenge),
        };

        // OpenID4VCI states: "Credential Issuers MUST ignore unknown scope values in a request."
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-5.1.2>
        //
        // Look up the credential configurations based on the scope values, ignoring any unknown scope value. Do not
        // continue if none of the scopes match Credential Configurations. This could happen if no scope values are
        // provided. The Credential Configurations are then used to pass their format and attestation types to the
        // type that implements `AuthorizationCodeFlow`, so that it can return the relevant `IssuableDocument`s.
        //
        // The scope is part of `WalletAuthorizationContext` in order to store this in the session state in the next
        // step. Once there, it is used to compare against any scope that is requested as part of the Token Request.
        let credential_kinds = request
            .scope
            .iter()
            .flat_map(|scope| credential_configs.get_by_scope(scope))
            .map(|(_id, config)| config.credential_kind.clone())
            .collect::<HashSet<_>>();

        if credential_kinds.is_empty() {
            return Err(InvalidAuthorizationRequest::NoValidScope(request.scope));
        }

        Ok(Self {
            state: request.oauth_request.state,
            issuer_state: request.issuer_state,
            credential_kinds,
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
    Authorized(VecNonEmpty<IssuableDocument>, Box<WalletAuthorizationContext>),
}

#[trait_variant::make(Send)]
pub trait AuthorizationCodeFlow {
    type Error: ErrorWithCode<ErrorCode = AuthorizationErrorCode> + Send + Sync + 'static;

    /// Called after the `openid4vc` layer has consumed the PAR entry, resolved the original authorization request and
    /// extracted the wallet-side [`WalletAuthorizationContext`]. The implementation decides how the user authenticates
    /// and returns the protocol-level outcome. Part of the contract of this method is that any `IssuableDocument` that
    /// is returned (directly or indirectly) adheres to the requested combinations of format and attestation type.
    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error>;

    /// Removes any expired state this flow owns.
    ///
    /// Called periodically by the authorization-phase cleanup task. Defaults to a no-op for flows that have no
    /// expiring state of their own.
    fn cleanup(&self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;
    use std::sync::Arc;

    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;

    use super::InvalidAuthorizationRequest;
    use super::WalletAuthorizationContext;
    use crate::authorization::PkceCodeChallenge;
    use crate::authorization::VciAuthorizationRequest;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::pkce::PkcePair;
    use crate::pkce::S256PkcePair;
    use crate::scope::Scope;
    use crate::server_state::MemorySessionStore;
    use crate::test::MOCK_ATTESTATION_TYPES;
    use crate::test::MockIssuer;
    use crate::test::setup_mock_issuer;

    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_STATE: &str = "wallet-state";
    // Match the credential config id / scope configured in the mock issuer.
    const WALLET_SCOPE: &str = "com.example.pid_dc+sd-jwt";

    /// Builds a mock issuer (whose credential configurations cover `MOCK_ATTESTATION_TYPES`) and
    /// returns it so tests can pass its [`CredentialConfigurations`] to `try_from_request`.
    fn mock_issuer() -> Arc<MockIssuer> {
        let issuer_identifier = IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap();
        let sessions = Arc::new(MemorySessionStore::default());
        let (issuer, _, _) = setup_mock_issuer(
            issuer_identifier,
            MOCK_ATTESTATION_TYPES.len().try_into().unwrap(),
            sessions,
        );
        Arc::new(issuer)
    }

    fn vci_request(scope: HashSet<Scope>) -> VciAuthorizationRequest {
        VciAuthorizationRequest::for_auth_code(
            "client-id".to_string(),
            WALLET_REDIRECT_URI.parse().unwrap(),
            WALLET_STATE.to_string(),
            None,
            scope,
            &S256PkcePair::generate(),
        )
    }

    #[tokio::test]
    async fn try_from_request_rejects_invalid_scope() {
        // None of the requested scopes reference a known credential configuration.
        let issuer = mock_issuer();
        let scope = HashSet::from(["scope1".parse().unwrap(), "scope2".parse().unwrap()]);
        let request = vci_request(scope.clone());

        let error = WalletAuthorizationContext::try_from_request(request, issuer.credential_configs()).unwrap_err();

        assert_matches!(error, InvalidAuthorizationRequest::NoValidScope(rejected) if rejected == scope);
    }

    #[tokio::test]
    async fn try_from_request_rejects_unsupported_code_challenge() {
        // A `plain` code_challenge_method is not supported.
        let issuer = mock_issuer();
        let mut request = vci_request(HashSet::from([WALLET_SCOPE.parse().unwrap()]));
        request.code_challenge = PkceCodeChallenge::Plain {
            code_challenge: "plain-challenge".to_string(),
        };

        let error = WalletAuthorizationContext::try_from_request(request, issuer.credential_configs()).unwrap_err();

        assert_matches!(error, InvalidAuthorizationRequest::UnsupportedCodeChallenge);
    }

    #[tokio::test]
    async fn try_from_request_derives_credential_kinds_from_known_scope() {
        // The known scope resolves to the SD-JWT credential configuration for the first attestation type.
        let issuer = mock_issuer();
        let request = vci_request(HashSet::from([WALLET_SCOPE.parse().unwrap()]));

        let context = WalletAuthorizationContext::try_from_request(request, issuer.credential_configs()).unwrap();

        assert_eq!(
            context.credential_kinds,
            HashSet::from_iter([CredentialKind::new(
                Format::SdJwt,
                String::from(MOCK_ATTESTATION_TYPES[0])
            )])
        );
    }
}
