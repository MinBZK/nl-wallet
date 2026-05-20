//! Authorization Phase wrapper around the Issuance Phase [`Issuer`].
//!
//! [`AuthorizingIssuer`] owns the PAR store, the wallet ↔ upstream PKCE bridge store, and the
//! upstream authorization adapter. It handles the Pushed Authorization Request and authorize
//! endpoints, performs the wallet ↔ upstream PKCE bridge consumption at `/token`, and then
//! delegates token processing to the inner [`Issuer`]. Deployments that only do the
//! pre-authorized-code grant (no upstream OIDC server) use the bare [`Issuer`] directly and never
//! construct an [`AuthorizingIssuer`].

use std::sync::Arc;

use crypto::EcdsaKey;
use url::Url;

use crate::authorization::PkceCodeChallenge;
use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::dpop::Dpop;
use crate::issuer::AttributeService;
use crate::issuer::AuthorizeError;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::issuer::ParError;
use crate::issuer::TokenRequestError;
use crate::issuer::UpstreamAuthorizationAdapter;
use crate::par;
use crate::par::PAR_TTL;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::server_state::SessionStore;
use crate::store::Store;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;

/// Authorization Phase wrapper around an Issuance Phase [`Issuer`].
pub struct AuthorizingIssuer<A, K, L, S, N, PAS, PKS, UAA> {
    issuer: Arc<Issuer<A, K, L, S, N>>,
    par_store: Arc<PAS>,
    pkce_flow_store: Arc<PKS>,
    upstream_authorization_adapter: UAA,
}

impl<A, K, L, S, N, PAS, PKS, UAA> AuthorizingIssuer<A, K, L, S, N, PAS, PKS, UAA> {
    pub fn new(
        issuer: Arc<Issuer<A, K, L, S, N>>,
        par_store: Arc<PAS>,
        pkce_flow_store: Arc<PKS>,
        upstream_authorization_adapter: UAA,
    ) -> Self {
        Self {
            issuer,
            par_store,
            pkce_flow_store,
            upstream_authorization_adapter,
        }
    }

    pub fn inner(&self) -> &Arc<Issuer<A, K, L, S, N>> {
        &self.issuer
    }
}

impl<A, K, L, S, N, PAS, PKS, UAA> AuthorizingIssuer<A, K, L, S, N, PAS, PKS, UAA>
where
    PAS: Store<String, VciAuthorizationRequest>,
{
    pub async fn process_pushed_authorization_request(
        &self,
        request: VciAuthorizationRequest,
    ) -> Result<PushedAuthorizationResponse, ParError> {
        if !self
            .issuer
            .accepted_wallet_client_ids()
            .any(|id| id == request.oauth_request.client_id.as_str())
        {
            return Err(ParError::InvalidClient(request.oauth_request.client_id));
        }

        let request_uri = par::generate_request_uri();

        self.par_store
            .store(request_uri.clone(), request)
            .await
            .map_err(|error| ParError::Store(Box::new(error)))?;

        Ok(PushedAuthorizationResponse {
            request_uri,
            expires_in: PAR_TTL,
        })
    }
}

impl<A, K, L, S, N, PAS, PKS, UAA> AuthorizingIssuer<A, K, L, S, N, PAS, PKS, UAA>
where
    PAS: Store<String, VciAuthorizationRequest>,
    PKS: Store<String, String>,
    UAA: UpstreamAuthorizationAdapter,
{
    /// Consume the PAR, swap the wallet's PKCE challenge for an upstream one (storing the upstream
    /// verifier under the wallet's challenge for the matching `/token` call), dispatch via the
    /// configured [`UpstreamAuthorizationAdapter`], and return the URL the wallet should be
    /// redirected to.
    pub async fn process_authorize(&self, request_uri: &str, client_id: &str) -> Result<Url, AuthorizeError> {
        if !self.issuer.accepted_wallet_client_ids().any(|id| id == client_id) {
            return Err(AuthorizeError::InvalidClient(client_id.to_string()));
        }

        let mut authorization_request = self
            .par_store
            .consume(request_uri)
            .await
            .map_err(|error| AuthorizeError::ParStore(Box::new(error)))?
            .ok_or_else(|| AuthorizeError::UnknownRequestUri(request_uri.to_string()))?;

        // Bridge PKCE: generate a new PKCE pair for the upstream server, substitute the wallet's challenge with the
        // upstream challenge, and store the upstream verifier keyed by the wallet's challenge for the matching
        // /token call.
        {
            let wallet_code_challenge = match &authorization_request.code_challenge {
                PkceCodeChallenge::S256 { code_challenge } => code_challenge.clone(),
                PkceCodeChallenge::Plain { .. } => return Err(AuthorizeError::UnsupportedCodeChallenge),
            };

            let upstream_pkce = S256PkcePair::generate();
            authorization_request.code_challenge = PkceCodeChallenge::S256 {
                code_challenge: upstream_pkce.code_challenge().to_string(),
            };

            self.pkce_flow_store
                .store(wallet_code_challenge, upstream_pkce.into_code_verifier())
                .await
                .map_err(|error| AuthorizeError::PkceStore(Box::new(error)))?;
        }

        let (authorization_endpoint, authorization_request) = self
            .upstream_authorization_adapter
            .adapt(authorization_request)
            .await
            .map_err(AuthorizeError::UpstreamResolve)?;

        let query_string = serde_urlencoded::to_string(&authorization_request).map_err(AuthorizeError::Encode)?;

        let mut redirect_url = authorization_endpoint;
        redirect_url.set_query(Some(&query_string));

        Ok(redirect_url)
    }
}

impl<A, K, L, S, N, PAS, PKS, UAA> AuthorizingIssuer<A, K, L, S, N, PAS, PKS, UAA>
where
    A: AttributeService,
    K: EcdsaKey,
    S: SessionStore<IssuanceData>,
    PKS: Store<String, String>,
{
    /// Process a token request, performing the wallet ↔ upstream PKCE bridge consumption when the
    /// grant type is `authorization_code`, then delegating to the inner [`Issuer`].
    pub async fn process_token_request(
        &self,
        mut token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponse, String), TokenRequestError> {
        // The wallet ↔ issuer PKCE check (RFC 7636) is generic and stays here: consuming the bridge
        // entry keyed by the wallet's code challenge *is* that verification. The stored value is the
        // upstream code verifier; relay it onward via `code_verifier` (the field the upstream token
        // exchange uses), keeping the bridge an upstream-OIDC detail the inner issuer and the
        // attribute service stay out of.
        if let TokenRequestGrantType::AuthorizationCode { .. } = &token_request.grant_type {
            let wallet_code_verifier = token_request
                .code_verifier
                .as_ref()
                .ok_or(TokenRequestError::MissingCodeVerifier)?;
            let wallet_code_challenge = S256PkcePair::challenge_for(wallet_code_verifier);

            let upstream_code_verifier = self
                .pkce_flow_store
                .consume(&wallet_code_challenge)
                .await
                .map_err(|error| TokenRequestError::PkceStore(Box::new(error)))?
                .ok_or(TokenRequestError::PkceVerificationFailed)?;

            token_request.code_verifier = Some(upstream_code_verifier);
        }

        self.issuer.process_token_request(token_request, dpop).await
    }
}
