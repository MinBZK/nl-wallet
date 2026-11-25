use std::collections::HashMap;
use std::num::NonZeroU8;
use std::str::FromStr;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use error_category::ErrorCategory;
use http_utils::reqwest::HttpClient;
use itertools::Either;
use itertools::Itertools;
use jwt::nonce::Nonce;
use oauth::authorization::AuthorizationResponse;
use oauth::authorization::PushedAuthorizationRequest;
use oauth::authorization::PushedAuthorizationResponse;
use oauth::dpop::DPOP_HEADER_NAME;
use oauth::dpop::Dpop;
use oauth::dpop::DpopNonce;
use oauth::errors::AuthorizationErrorCode;
use oauth::errors::RemoteErrorCode;
use oauth::errors::RemoteErrorResponse;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::pkce::PkcePair;
use oauth::pkce::S256PkcePair;
use oauth::token::AuthorizationCode;
use reqwest::Method;
use reqwest::Response;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use wscd::payload::wia::WIA_HEADER_NAME;
use wscd::payload::wia::WIA_POP_HEADER_NAME;
use wscd::payload::wia::WiaDisclosure;
use wscd::wia::WiaClient;

use super::AuthorizationSession;
use super::WalletIssuanceError;
use super::authorization_endpoints::AuthorizationEndpoints;
use super::credential_request_encryption::CredentialRequestEncryptionParameters;
use super::dpop::DpopState;
use super::dpop::nonce_from_headers;
use super::issuance_session::HttpIssuanceSession;
use super::issuance_session::HttpVcMessageClient;
use super::issuer_registration::IssuerRegistration;
use crate::authorization::VciAuthorizationRequest;
use crate::client_auth::ClientAttestationChallengeMechanism;
use crate::client_auth::attestation_challenge_from_headers;
use crate::client_auth::fetch_client_auth_challenge;
use crate::errors::ParErrorCode;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::token::VciTokenRequest;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OAuthError {
    #[error("error encoding authorization request to URL: {0}")]
    AuthRequestUrlEncoding(#[source] serde_qs::Error),

    #[error("error decoding authorization response from URL: {0}")]
    AuthResponseUrlDecoding(#[source] serde_qs::Error),

    #[error("error decoding error response from URL: {0}")]
    ErrorResponseUrlDecoding(#[source] serde_qs::Error),

    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(Box<RemoteErrorResponse<AuthorizationErrorCode>>),

    #[error("invalid state token received in redirect URI")]
    #[category(critical)]
    StateTokenMismatch,

    #[error("no authorization code received in redirect URI")]
    #[category(critical)]
    NoAuthCode,

    #[error("invalid redirect URI received")]
    #[category(critical)]
    RedirectUriMismatch,

    #[error("pushed authorization request rejected: {0:?}")]
    #[category(expected)]
    PushedAuthorizationRequest(Box<RemoteErrorResponse<ParErrorCode>>),

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,
}

/// Outcome of one PAR or token attempt. A `400 use_dpop_nonce` response can carry two independent challenges.
/// Keep them together when consuming the HTTP response so the retry can sign each value into the matching proof.
#[derive(Debug)]
pub enum AuthorizationServerResponse<T> {
    Success(T),
    UseDpopNonce {
        /// `DPoP-Nonce`, used in the DPoP proof's `nonce` claim.
        nonce: DpopNonce,
        /// `OAuth-Client-Attestation-Challenge`, used in the WIA PoP's `challenge` claim.
        attestation_challenge: Option<Nonce>,
    },
}

impl<T> AuthorizationServerResponse<T> {
    pub(super) async fn from_http_error<E: FromStr + PartialEq>(
        response: Response,
        use_dpop_nonce: E,
        http_error: fn(reqwest::Error) -> WalletIssuanceError,
        oauth_error: impl FnOnce(RemoteErrorResponse<E>) -> WalletIssuanceError,
    ) -> Result<Self, WalletIssuanceError> {
        let status = response.status();
        let nonce = nonce_from_headers(response.headers());
        let attestation_challenge = attestation_challenge_from_headers(response.headers());
        // A nonce header on another OAuth error does not authorize a retry.
        let error = response.json::<RemoteErrorResponse<E>>().await.map_err(http_error)?;
        if status == StatusCode::BAD_REQUEST
            && error.error == RemoteErrorCode::Known(use_dpop_nonce)
            && let Some(nonce) = nonce?
        {
            return Ok(Self::UseDpopNonce {
                nonce,
                attestation_challenge: attestation_challenge
                    .map_err(WalletIssuanceError::ClientAttestationChallengeMechanism)?,
            });
        }
        Err(oauth_error(error))
    }
}

/// Proof parameters for a PAR or token request. DPoP uses the request endpoint as its `htu`.
/// The WIA PoP uses the authorization server's issuer identifier as its `aud`.
pub(super) struct AttestedRequest<'a> {
    pub endpoint: &'a Url,
    pub audience: &'a IssuerIdentifier,
    pub challenge: ClientAttestationChallengeMechanism,
}

impl AttestedRequest<'_> {
    /// Send a PAR or token request and retry once on a DPoP nonce challenge.
    ///
    /// The server may consume a WIA PoP or its challenge before asking for a DPoP nonce. Obtain a new WIA and PoP
    /// from the Wallet Provider on each attempt. The caller resends the same request body, and the wallet signs the
    /// new DPoP proof with the original key.
    pub(super) async fn send_with_dpop_retry<T, ChallengeFut, RequestFut>(
        mut self,
        dpop: &mut DpopState,
        wia_client: &impl WiaClient,
        fetch_challenge: impl Fn(Url) -> ChallengeFut,
        send: impl Fn(Dpop, WiaDisclosure) -> RequestFut,
    ) -> Result<T, WalletIssuanceError>
    where
        ChallengeFut: Future<Output = Result<Nonce, WalletIssuanceError>>,
        RequestFut: Future<Output = Result<AuthorizationServerResponse<T>, WalletIssuanceError>>,
    {
        let mut retried = false;
        loop {
            let challenge = match &self.challenge {
                ClientAttestationChallengeMechanism::ChallengeEndpoint(url) => {
                    Some(fetch_challenge(url.clone()).await?)
                }
                ClientAttestationChallengeMechanism::Header(header)
                | ClientAttestationChallengeMechanism::HeaderWithEndpoint { header, .. } => Some(header.clone()),
                ClientAttestationChallengeMechanism::None => None,
            };
            let wia = wia_client
                .issue_wia(self.audience.to_string(), challenge)
                .await
                .map_err(|e| WalletIssuanceError::WiaIssuance(e.into()))?;
            let proof = Dpop::new(
                &dpop.signing_key,
                self.endpoint.clone(),
                &Method::POST,
                None,
                dpop.nonce.clone(),
            )?;

            match send(proof, wia).await? {
                AuthorizationServerResponse::Success(response) => return Ok(response),
                AuthorizationServerResponse::UseDpopNonce {
                    nonce,
                    attestation_challenge,
                } => {
                    if retried {
                        return Err(WalletIssuanceError::DpopNonceRejected);
                    }
                    self.challenge = self.challenge.after_response(attestation_challenge);
                    dpop.nonce = Some(nonce);
                    retried = true;
                }
            }
        }
    }
}

/// The state of an in-progress OAuth authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationSession<P = S256PkcePair> {
    credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    issuer_registration: IssuerRegistration,
    batch_size: NonZeroU8,
    encryption_parameters: Option<CredentialRequestEncryptionParameters>,
    token_endpoint: Url,
    client_attestation_challenge: ClientAttestationChallengeMechanism,
    authorization_server: IssuerIdentifier,
    http_client: HttpClient,

    auth_url: Url,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
    dpop: DpopState,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HttpAuthorizationSessionData {
    credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
    credential_issuer: IssuerIdentifier,
    issuer_endpoints: IssuerEndpoints,
    issuer_registration: IssuerRegistration,
    batch_size: NonZeroU8,
    encryption_parameters: Option<CredentialRequestEncryptionParameters>,
    token_endpoint: Url,
    client_attestation_challenge: ClientAttestationChallengeMechanism,
    authorization_server: IssuerIdentifier,
    auth_url: Url,
    redirect_uri: Url,
    code_verifier: String,
    state: String,
    dpop: DpopState,
}

impl<P: PkcePair> HttpAuthorizationSession<P> {
    /// POST the authorization parameters to the PAR endpoint, then build the authorization URL
    /// using the returned `request_uri`. Returns an error if the provider has no PAR endpoint,
    /// the PAR request is rejected, or the URL cannot be constructed.
    #[expect(clippy::too_many_arguments, reason = "internal constructor")]
    pub(super) async fn create(
        http_client: HttpClient,
        credential_configurations: HashMap<CredentialConfigurationId, CredentialConfiguration>,
        credential_issuer: IssuerIdentifier,
        issuer_endpoints: IssuerEndpoints,
        issuer_registration: IssuerRegistration,
        batch_size: NonZeroU8,
        encryption_parameters: Option<CredentialRequestEncryptionParameters>,
        auth_endpoints: AuthorizationEndpoints,
        client_id: String,
        redirect_uri: Url,
        issuer_state: Option<String>,
        wia_client: &impl WiaClient,
        authorization_server: IssuerIdentifier,
    ) -> Result<Self, WalletIssuanceError> {
        // Include the `scope` values for each Credential Configuration with a supported credential format that was
        // present in the Credential Offer and return an error if any of them do not include a `scope`. Note that we
        // include the `scope` values in favour of using the `authorization_details` field. According to HAIP:
        //
        // "For Grant Type authorization_code, the Issuer MUST include a scope value in order to allow the Wallet to
        // identify the desired Credential Type. The Wallet MUST use that value in the scope Authorization parameter."
        //
        // Source: <https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-4.2>
        let (scope, no_scope_config_ids): (_, Vec<_>) = credential_configurations
            .iter()
            .filter(|(_id, config)| config.format.is_supported())
            .partition_map(|(id, config)| match &config.scope {
                Some(scope) => Either::Left(scope.clone()),
                None => Either::Right(id.clone()),
            });

        if !no_scope_config_ids.is_empty() {
            return Err(WalletIssuanceError::IssuerMetadataNoScope(no_scope_config_ids));
        }

        let pkce_pair = P::generate();
        let state = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));

        let par_request = VciAuthorizationRequest::for_auth_code(
            client_id.clone(),
            redirect_uri.clone(),
            state.clone(),
            issuer_state,
            scope,
            &pkce_pair,
        );

        let mut dpop = DpopState::generate();
        let response = AttestedRequest {
            endpoint: &auth_endpoints.par_endpoint,
            audience: &authorization_server,
            challenge: auth_endpoints.challenge_endpoint.clone().map_or(
                ClientAttestationChallengeMechanism::None,
                ClientAttestationChallengeMechanism::ChallengeEndpoint,
            ),
        }
        .send_with_dpop_retry(
            &mut dpop,
            wia_client,
            async |url| {
                fetch_client_auth_challenge(&http_client, url)
                    .await
                    .map_err(WalletIssuanceError::ClientAttestationChallenge)
            },
            async |proof, wia| {
                let response = http_client
                    .post(auth_endpoints.par_endpoint.clone(), |builder| {
                        builder
                            .form(&par_request)
                            .header(DPOP_HEADER_NAME, proof.to_string())
                            .header(WIA_HEADER_NAME, wia.wia().serialization())
                            .header(WIA_POP_HEADER_NAME, wia.wia_pop().serialization())
                    })
                    .await
                    .map_err(WalletIssuanceError::ParHttp)?;
                if response.status().is_success() {
                    return Ok(AuthorizationServerResponse::Success(response));
                }
                AuthorizationServerResponse::from_http_error(
                    response,
                    ParErrorCode::UseDpopNonce,
                    WalletIssuanceError::ParHttp,
                    |error| OAuthError::PushedAuthorizationRequest(Box::new(error)).into(),
                )
                .await
            },
        )
        .await?;
        // A successful PAR may rotate the nonce for the token request. Otherwise retain the nonce accepted at PAR.
        if let Some(nonce) = nonce_from_headers(response.headers())? {
            dpop.nonce = Some(nonce);
        }

        let client_auth_challenge =
            ClientAttestationChallengeMechanism::try_new_acf(auth_endpoints.challenge_endpoint, &response)
                .map_err(WalletIssuanceError::ClientAttestationChallengeMechanism)?;
        let par_response = response
            .json::<PushedAuthorizationResponse>()
            .await
            .map_err(WalletIssuanceError::ParHttp)?;

        let auth_url = PushedAuthorizationRequest::from_par_response(client_id, &par_response)
            .into_authorization_url(auth_endpoints.authorization_endpoint);

        Ok(Self {
            credential_configurations,
            credential_issuer,
            issuer_endpoints,
            issuer_registration,
            batch_size,
            encryption_parameters,
            token_endpoint: auth_endpoints.token_endpoint,
            client_attestation_challenge: client_auth_challenge,
            authorization_server,
            http_client,
            auth_url,
            redirect_uri,
            pkce_pair,
            state,
            dpop,
        })
    }

    #[cfg(any(test, feature = "test"))]
    pub fn state(&self) -> &str {
        &self.state
    }

    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool {
        received_redirect_uri.as_str().starts_with(self.redirect_uri.as_str())
    }

    fn authorization_code(&self, received_redirect_uri: &Url) -> Result<AuthorizationCode, OAuthError> {
        if !self.matches_received_redirect_uri(received_redirect_uri) {
            return Err(OAuthError::RedirectUriMismatch);
        }

        let auth_response = received_redirect_uri.query().ok_or(OAuthError::NoAuthCode)?;

        // First see if we received an error
        if received_redirect_uri.query_pairs().any(|(key, _)| key == "error") {
            let err_response: RemoteErrorResponse<AuthorizationErrorCode> =
                serde_qs::from_str(auth_response).map_err(OAuthError::ErrorResponseUrlDecoding)?;

            return if err_response.error == RemoteErrorCode::Known(AuthorizationErrorCode::AccessDenied) {
                Err(OAuthError::Denied)
            } else {
                Err(OAuthError::RedirectUriError(Box::new(err_response)))
            };
        }

        let auth_response: AuthorizationResponse =
            serde_qs::from_str(auth_response).map_err(OAuthError::AuthResponseUrlDecoding)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OAuthError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

impl HttpAuthorizationSession {
    pub fn restore(http_client: HttpClient, data: HttpAuthorizationSessionData) -> Self {
        Self {
            credential_configurations: data.credential_configurations,
            credential_issuer: data.credential_issuer,
            issuer_endpoints: data.issuer_endpoints,
            issuer_registration: data.issuer_registration,
            batch_size: data.batch_size,
            encryption_parameters: data.encryption_parameters,
            token_endpoint: data.token_endpoint,
            client_attestation_challenge: data.client_attestation_challenge,
            authorization_server: data.authorization_server,
            http_client,
            auth_url: data.auth_url,
            redirect_uri: data.redirect_uri,
            pkce_pair: S256PkcePair::from_code_verifier(data.code_verifier),
            state: data.state,
            dpop: data.dpop,
        }
    }
}

impl AuthorizationSession for HttpAuthorizationSession {
    type Issuance = HttpIssuanceSession;
    type Persisted = HttpAuthorizationSessionData;

    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    fn state(&self) -> &str {
        &self.state
    }

    fn persist(&self) -> Self::Persisted {
        HttpAuthorizationSessionData {
            credential_configurations: self.credential_configurations.clone(),
            credential_issuer: self.credential_issuer.clone(),
            issuer_endpoints: self.issuer_endpoints.clone(),
            issuer_registration: self.issuer_registration.clone(),
            batch_size: self.batch_size,
            encryption_parameters: self.encryption_parameters.clone(),
            token_endpoint: self.token_endpoint.clone(),
            client_attestation_challenge: self.client_attestation_challenge.clone(),
            authorization_server: self.authorization_server.clone(),
            auth_url: self.auth_url.clone(),
            redirect_uri: self.redirect_uri.clone(),
            code_verifier: self.pkce_pair.code_verifier().to_string(),
            state: self.state.clone(),
            dpop: self.dpop.clone(),
        }
    }

    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        wia_client: &impl WiaClient,
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let authorization_code = self.authorization_code(received_redirect_uri)?;
        let message_client = HttpVcMessageClient::new(self.http_client);

        // Create the Token Request to be sent to the issuer with the minimal amount of information required. This does
        // not include either a `scope` or `authorization_details` field, as we have no need to further restrict the
        // credentials requested at this point.
        let token_request = VciTokenRequest::new_authorization_code(
            authorization_code,
            self.redirect_uri,
            self.pkce_pair.into_code_verifier(),
        );

        HttpIssuanceSession::create(
            message_client,
            self.credential_configurations,
            self.credential_issuer,
            self.issuer_endpoints,
            self.issuer_registration,
            self.batch_size,
            self.encryption_parameters,
            self.token_endpoint,
            self.client_attestation_challenge,
            token_request,
            wia_client,
            &self.authorization_server,
            self.dpop,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;

    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use http::header;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpClient;
    use http_utils::reqwest::default_reqwest_client_builder;
    use http_utils::urls::BaseUrl;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use jwe::algorithm::EcdhAlgorithm;
    use jwe::algorithm::EncryptionAlgorithm;
    use jwe::encryption::JwePublicKey;
    use jwt::nonce::Nonce;
    use oauth::errors::AuthorizationErrorCode;
    use oauth::errors::RemoteErrorCode;
    use oauth::issuer_identifier::IssuerIdentifier;
    use oauth::pkce::MockPkcePair;
    use oauth::pkce::S256PkcePair;
    use p256::SecretKey;
    use p256::elliptic_curve::Generate;
    use rstest::rstest;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;
    use wscd::payload::wia::WIA_CLIENT_CHALLENGE_HEADER_NAME;
    use wscd::wia::mock::MockWiaClient;

    use super::super::AuthorizationSession;
    use super::super::WalletIssuanceError;
    use super::super::authorization_endpoints::AuthorizationEndpoints;
    use super::super::credential_request_encryption::CredentialRequestEncryptionParameters;
    use super::super::dpop::DpopState;
    use super::super::dpop::tests::request_nonce_matches;
    use super::super::dpop::tests::wia_challenge_matches;
    use super::super::issuer_registration::IssuerRegistration;
    use super::super::mock::RecordingWiaClient;
    use super::HttpAuthorizationSession;
    use super::HttpAuthorizationSessionData;
    use super::OAuthError;
    use crate::client_auth::ClientAttestationChallengeMechanism;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::IssuerMetadata;
    use crate::mock::MOCK_WALLET_CLIENT_ID;

    const ISSUER_URL: &str = "https://example.com";
    const TOKEN_ENDPOINT: &str = "/issuance/token";
    const CHALLENGE_ENDPOINT: &str = "/issuance/client_auth_challenge";
    const REDIRECT_URI: &str = "redirect://here";
    const CSRF_TOKEN: &str = "csrf_token";
    const CODE: &str = "code";
    const PAR_REQUEST_URI: &str = "urn:ietf:params:oauth:request_uri:test-12345";

    pub fn url_with_query_pairs(mut url: Url, query_pairs: &[(&str, &str)]) -> Url {
        if !query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();
            query_pairs.iter().for_each(|(name, value)| {
                query.append_pair(name, value);
            });
        }
        url
    }

    fn create_session() -> HttpAuthorizationSession<MockPkcePair> {
        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            ISSUER_URL.parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let mut pkce_pair = MockPkcePair::new();
        pkce_pair.expect_code_challenge().return_const("challenge".to_string());
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        HttpAuthorizationSession {
            credential_configurations: issuer_metadata.credential_configurations_supported,
            credential_issuer: issuer_metadata.credential_issuer,
            issuer_endpoints: issuer_metadata.endpoints,
            issuer_registration: IssuerRegistration::new_mock(),
            batch_size,
            encryption_parameters: None,
            token_endpoint: ISSUER_URL.parse::<BaseUrl>().unwrap().join(TOKEN_ENDPOINT),
            client_attestation_challenge: ClientAttestationChallengeMechanism::ChallengeEndpoint(
                ISSUER_URL.parse::<BaseUrl>().unwrap().join(CHALLENGE_ENDPOINT),
            ),
            authorization_server: ISSUER_URL.parse().unwrap(),
            http_client: HttpClient::try_new(default_reqwest_client_builder()).unwrap(),
            auth_url: ISSUER_URL.parse().unwrap(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            pkce_pair,
            state: CSRF_TOKEN.to_string(),
            dpop: DpopState::generate(),
        }
    }

    #[tokio::test]
    async fn test_authorization_code() {
        let session = create_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("code", CODE), ("state", &state)]);

        let code = session.authorization_code(&received).unwrap();

        assert_eq!(code.as_ref(), CODE);
    }

    #[tokio::test]
    async fn test_user_cancels() {
        let session = create_session();

        let state = session.state.clone();
        let redirect_uri: Url = REDIRECT_URI.parse().unwrap();
        let received = url_with_query_pairs(redirect_uri, &[("error", "access_denied"), ("state", &state)]);

        let error = session.authorization_code(&received).unwrap_err();

        assert_matches!(error, OAuthError::Denied);
    }

    fn authorization_endpoints(base_url: &BaseUrl) -> AuthorizationEndpoints {
        let authorization_endpoint = base_url.join("/authorize");
        let pushed_authorization_request_endpoint = base_url.join("/issuance/par");
        let token_endpoint = base_url.join("/issuance/token");
        let challenge_endpoint = Some(base_url.join("/issuance/client_auth_challenge"));

        AuthorizationEndpoints {
            authorization_endpoint,
            par_endpoint: pushed_authorization_request_endpoint,
            token_endpoint,
            challenge_endpoint,
        }
    }

    #[rstest]
    #[case::without_issuer_state(None)]
    #[case::with_issuer_state(Some("foobar"))]
    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_auth_url(#[case] issuer_state: Option<&str>) {
        let server = MockServer::start_async().await;

        let par_request = server
            .mock_async(|when, then| {
                let when = when
                    .method(POST)
                    .path("/issuance/par")
                    .form_urlencoded_tuple("client_id", MOCK_WALLET_CLIENT_ID)
                    .form_urlencoded_tuple("redirect_uri", REDIRECT_URI)
                    .is_true(|request| request_nonce_matches(request, None));

                match issuer_state {
                    None => {
                        when.form_urlencoded_tuple_missing("issuer_state");
                    }
                    Some(issuer_state) => {
                        when.form_urlencoded_tuple("issuer_state", issuer_state);
                    }
                }

                then.status(201)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(json!({
                        "request_uri": PAR_REQUEST_URI,
                        "expires_in": 60,
                    }));
            })
            .await;

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/client_auth_challenge");
                then.status(200)
                    .json_body(json!({
                        "attestation_challenge": "foobar",
                    }))
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref());
            })
            .await;

        let auth_endpoints = authorization_endpoints(&server.base_url().parse().unwrap());

        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair.expect_code_challenge().return_const("challenge".to_string());
            pkce_pair
        });

        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            server.base_url().parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let session = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            None,
            auth_endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            issuer_state.map(str::to_string),
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
        )
        .await
        .unwrap();

        par_request.assert_calls_async(1).await;
        assert!(session.dpop.nonce.is_none());
        let params: HashMap<_, _> = session.auth_url.query_pairs().collect();

        // Auth URL after PAR carries only client_id + request_uri (RFC 9126 §4)
        assert_eq!(params.get("client_id").map(|v| v.as_ref()), Some(MOCK_WALLET_CLIENT_ID));
        assert_eq!(params.get("request_uri").map(|v| v.as_ref()), Some(PAR_REQUEST_URI));
        assert!(!params.contains_key("code_challenge"));
        assert!(!params.contains_key("state"));
        assert!(!params.contains_key("redirect_uri"));
        assert!(!params.contains_key("issuer_state"));
        assert!(!params.contains_key("scope"));
    }

    /// Which client attestation challenge mechanism(s) a `test_auth_url_client_auth_challenge_mechanism` case
    /// exercises, and what `HttpAuthorizationSession::create()` is expected to do with them.
    enum ExpectedChallengeMechanism {
        None,
        Header,
        ChallengeEndpoint,
        Both,
    }

    #[rstest]
    #[case::none(false, false, ExpectedChallengeMechanism::None)]
    #[case::header(false, true, ExpectedChallengeMechanism::Header)]
    #[case::challenge_endpoint(true, false, ExpectedChallengeMechanism::ChallengeEndpoint)]
    #[case::header_with_endpoint(true, true, ExpectedChallengeMechanism::Both)]
    #[tokio::test]
    #[serial(MockPkcePair)]
    async fn test_auth_url_client_auth_challenge_mechanism(
        #[case] challenge_endpoint_configured: bool,
        #[case] header_present: bool,
        #[case] expected: ExpectedChallengeMechanism,
    ) {
        let server = MockServer::start_async().await;

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/par");
                let then = then
                    .status(201)
                    .header(header::CONTENT_TYPE.as_str(), mime::APPLICATION_JSON.as_ref())
                    .json_body(json!({
                        "request_uri": PAR_REQUEST_URI,
                        "expires_in": 60,
                    }));
                if header_present {
                    then.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, "header-challenge");
                }
            })
            .await;

        let challenge = "foobar";
        if challenge_endpoint_configured {
            server
                .mock_async(|when, then| {
                    when.method(POST).path("/issuance/client_auth_challenge");
                    then.status(200).json_body(json!({
                        "attestation_challenge": challenge,
                    }));
                })
                .await;
        }

        let auth_endpoints = {
            let mut auth_endpoints = authorization_endpoints(&server.base_url().parse().unwrap());
            if !challenge_endpoint_configured {
                auth_endpoints.challenge_endpoint = None;
            }
            auth_endpoints
        };

        let pkce_context = MockPkcePair::generate_context();
        pkce_context.expect().return_once(|| {
            let mut pkce_pair = MockPkcePair::new();
            pkce_pair.expect_code_challenge().return_const("challenge".to_string());
            pkce_pair
        });

        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            server.base_url().parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let wia_client = RecordingWiaClient::default();
        let result = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            None,
            auth_endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            None,
            &wia_client,
            issuer_metadata.credential_issuer,
        )
        .await;

        // The WIA generated for the PAR request should include the challenge fetched from the challenge
        // endpoint if and only if one is configured, regardless of whether a header challenge is also present:
        // the latter is only known after the PAR response has been received.
        let expected_challenge = challenge_endpoint_configured.then(|| Nonce::from(challenge.to_string()));
        assert_eq!(*wia_client.received_challenge.borrow(), Some(expected_challenge));

        let expected_endpoint: Url = server.url(CHALLENGE_ENDPOINT).parse().unwrap();
        match expected {
            ExpectedChallengeMechanism::None => {
                assert_matches!(
                    result.unwrap().client_attestation_challenge,
                    ClientAttestationChallengeMechanism::None
                );
            }
            ExpectedChallengeMechanism::Header => {
                assert_matches!(
                    result.unwrap().client_attestation_challenge,
                    ClientAttestationChallengeMechanism::Header(challenge)
                        if challenge == Nonce::from("header-challenge".to_string())
                );
            }
            ExpectedChallengeMechanism::ChallengeEndpoint => {
                assert_matches!(
                    result.unwrap().client_attestation_challenge,
                    ClientAttestationChallengeMechanism::ChallengeEndpoint(url)
                        if url == expected_endpoint
                );
            }
            ExpectedChallengeMechanism::Both => {
                assert_matches!(
                    result.unwrap().client_attestation_challenge,
                    ClientAttestationChallengeMechanism::HeaderWithEndpoint { header, endpoint }
                        if header.as_ref() == "header-challenge" && endpoint == expected_endpoint
                );
            }
        }
    }

    #[tokio::test]
    async fn test_http_authorization_session_create_issuer_metadata_no_scope_error() {
        let issuer_identifier = "https://example.com".parse::<IssuerIdentifier>().unwrap();
        let config_id = CredentialConfigurationId::from("config_id".to_string());

        let auth_endpoints = authorization_endpoints(issuer_identifier.as_base_url());

        let mut issuer_metadata = IssuerMetadata::new_mock(
            issuer_identifier,
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );

        for config in issuer_metadata.credential_configurations_supported.values_mut() {
            config.scope = None;
        }

        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let error = HttpAuthorizationSession::<MockPkcePair>::create(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            None,
            auth_endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            None,
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
        )
        .await
        .expect_err("creating authorization session should fail");

        assert_matches!(
            error,
            WalletIssuanceError::IssuerMetadataNoScope(config_ids) if config_ids == vec![config_id]
        );
    }

    #[tokio::test]
    async fn test_matches_received_redirect_uri() {
        let session = create_session();

        assert!(session.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(session.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        assert!(!session.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!session.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    #[tokio::test]
    async fn test_redirect_uri_mismatch() {
        let session = create_session();
        let uri = Url::parse("http://not-the-redirect-uri.com").unwrap();

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::RedirectUriMismatch);
    }

    #[tokio::test]
    async fn test_error() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[
                ("error", "invalid_request"),
                ("error_description", "this is the error description"),
            ],
        );

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(
            error,
            OAuthError::RedirectUriError(response)
                if matches!(response.error, RemoteErrorCode::Known(AuthorizationErrorCode::InvalidRequest))
                && response.error_description == Some("this is the error description".to_string())
        );
    }

    #[tokio::test]
    async fn test_state_mismatch() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("code", CODE), ("state", "foobar")],
        );

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::StateTokenMismatch);
    }

    #[tokio::test]
    async fn test_missing_code() {
        let session = create_session();
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[("state", CSRF_TOKEN)]);

        let error = session.authorization_code(&uri).unwrap_err();

        assert_matches!(error, OAuthError::AuthResponseUrlDecoding(err) if err.to_string() == "missing field `code`");
    }

    #[tokio::test]
    async fn test_get_authorization_url() {
        let session = create_session();
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("state", CSRF_TOKEN), ("code", "123")],
        );

        assert_eq!(session.authorization_code(&uri).unwrap().as_ref(), "123");
    }

    #[tokio::test]
    async fn test_persist_and_restore() {
        let config_id: CredentialConfigurationId = "config_id".to_string().into();
        let issuer_metadata = IssuerMetadata::new_mock(
            ISSUER_URL.parse().unwrap(),
            vec![(
                config_id.clone(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let encryption_parameters = CredentialRequestEncryptionParameters::new(
            JwePublicKey::new(
                Some("key".to_string()),
                SecretKey::generate().public_key(),
                EcdhAlgorithm::EcdhEs,
            ),
            EncryptionAlgorithm::A128Gcm,
            None,
        );
        let persisted = HttpAuthorizationSessionData {
            credential_configurations: issuer_metadata.credential_configurations_supported,
            credential_issuer: issuer_metadata.credential_issuer,
            issuer_endpoints: issuer_metadata.endpoints,
            issuer_registration: IssuerRegistration::new_mock(),
            batch_size,
            encryption_parameters: Some(encryption_parameters),
            token_endpoint: ISSUER_URL.parse::<BaseUrl>().unwrap().join(TOKEN_ENDPOINT),
            client_attestation_challenge: ClientAttestationChallengeMechanism::ChallengeEndpoint(
                ISSUER_URL.parse::<BaseUrl>().unwrap().join(CHALLENGE_ENDPOINT),
            ),
            auth_url: ISSUER_URL.parse().unwrap(),
            redirect_uri: REDIRECT_URI.parse().unwrap(),
            code_verifier: "verifier".to_string(),
            state: CSRF_TOKEN.to_string(),
            authorization_server: ISSUER_URL.parse().unwrap(),
            dpop: DpopState::generate(),
        };

        let session = HttpAuthorizationSession {
            credential_configurations: persisted.credential_configurations,
            credential_issuer: persisted.credential_issuer,
            issuer_endpoints: persisted.issuer_endpoints,
            issuer_registration: persisted.issuer_registration.clone(),
            token_endpoint: persisted.token_endpoint,
            batch_size: persisted.batch_size,
            encryption_parameters: persisted.encryption_parameters.clone(),
            client_attestation_challenge: persisted.client_attestation_challenge,
            http_client: HttpClient::try_new(default_reqwest_client_builder()).unwrap(),
            auth_url: persisted.auth_url.clone(),
            redirect_uri: persisted.redirect_uri.clone(),
            pkce_pair: S256PkcePair::from_code_verifier(persisted.code_verifier.clone()),
            state: persisted.state.clone(),
            dpop: persisted.dpop.clone(),
            authorization_server: ISSUER_URL.parse().unwrap(),
        };

        let restored = HttpAuthorizationSession::restore(
            HttpClient::try_new(default_reqwest_client_builder()).unwrap(),
            serde_json::from_value(serde_json::to_value(session.persist()).unwrap()).unwrap(),
        );
        let restored_persisted = restored.persist();

        assert_eq!(
            restored_persisted.issuer_registration.organization(),
            persisted.issuer_registration.organization(),
        );
        assert_eq!(
            restored_persisted
                .issuer_registration
                .registration_certificate()
                .to_vec()
                .unwrap(),
            persisted
                .issuer_registration
                .registration_certificate()
                .to_vec()
                .unwrap(),
        );
        assert_eq!(
            restored_persisted.issuer_registration.access_certificate(),
            persisted.issuer_registration.access_certificate(),
        );
        assert_eq!(restored_persisted.batch_size, persisted.batch_size);
        assert_eq!(
            restored_persisted.encryption_parameters,
            persisted.encryption_parameters
        );
        assert_eq!(restored_persisted.auth_url, persisted.auth_url);
        assert_eq!(restored_persisted.redirect_uri, persisted.redirect_uri);
        assert_eq!(restored_persisted.code_verifier, persisted.code_verifier);
        assert_eq!(restored_persisted.state, persisted.state);
        assert_eq!(
            restored_persisted.dpop.signing_key.verifying_key(),
            persisted.dpop.signing_key.verifying_key()
        );
        assert_eq!(restored_persisted.dpop.nonce, persisted.dpop.nonce);

        let mut without_dpop = serde_json::to_value(restored_persisted).unwrap();
        without_dpop.as_object_mut().unwrap().remove("dpop");
        assert!(serde_json::from_value::<HttpAuthorizationSessionData>(without_dpop).is_err());
    }

    #[rstest]
    #[case("http://example.com", [], "http://example.com")]
    #[case("http://example.com", [("foo", "bar"), ("bleh", "blah")], "http://example.com?foo=bar&bleh=blah")]
    #[case("http://example.com", [("foo", ""), ("foo", "more_foo")], "http://example.com?foo=&foo=more_foo")]
    fn test_url_with_query_pairs<const N: usize>(
        #[case] url: Url,
        #[case] query_pairs: [(&str, &str); N],
        #[case] expected: Url,
    ) {
        let url = url_with_query_pairs(url, &query_pairs);
        assert_eq!(url, expected);
    }

    #[rstest]
    #[case::missing_nonce(400, "use_dpop_nonce", None)]
    #[case::invalid_nonce(400, "use_dpop_nonce", Some("invalid nonce"))]
    #[case::wrong_error(400, "invalid_request", Some("retry_nonce"))]
    #[case::wrong_status(401, "use_dpop_nonce", Some("retry_nonce"))]
    #[tokio::test]
    async fn test_par_does_not_retry_invalid_challenges(
        #[case] status: u16,
        #[case] error_code: &str,
        #[case] nonce: Option<&str>,
    ) {
        use oauth::dpop::DPOP_NONCE_HEADER_NAME;

        let server = MockServer::start_async().await;
        let par = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/par");
                let then = then.status(status);
                let then = match nonce {
                    Some(nonce) => then.header(DPOP_NONCE_HEADER_NAME, nonce),
                    None => then,
                };
                then.json_body(json!({"error": error_code}));
            })
            .await;
        let issuer_metadata = IssuerMetadata::new_mock(
            server.base_url().parse().unwrap(),
            vec![(
                "config_id".to_string().into(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let mut endpoints = authorization_endpoints(&server.base_url().parse().unwrap());
        endpoints.challenge_endpoint = None;
        let error = HttpAuthorizationSession::<S256PkcePair>::create(
            HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap(),
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            None,
            endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            None,
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
        )
        .await
        .unwrap_err();
        if nonce == Some("invalid nonce") {
            assert_matches!(error, WalletIssuanceError::DpopNonce(_));
        } else {
            assert_matches!(
                error,
                WalletIssuanceError::OAuth(OAuthError::PushedAuthorizationRequest(_))
            );
        }
        par.assert_calls_async(1).await;
    }

    #[rstest]
    #[case::repeated_par_challenge(true, false, false)]
    #[case::keep_both_challenges(false, false, false)]
    #[case::rotate_dpop_nonce(false, true, false)]
    #[case::rotate_wia_challenge(false, false, true)]
    #[tokio::test]
    async fn test_par_nonce_retry_and_key_continuity(
        #[case] repeated_challenge: bool,
        #[case] rotate_nonce_on_success: bool,
        #[case] rotate_wia_on_token_retry: bool,
    ) {
        use std::sync::Arc;
        use std::sync::Mutex;

        use crypto::keys::PublicKey;
        use oauth::dpop::DPOP_HEADER_NAME;
        use oauth::dpop::DPOP_NONCE_HEADER_NAME;
        use oauth::dpop::Dpop;
        use reqwest::Method;

        use crate::errors::VciTokenErrorCode;

        let server = MockServer::start_async().await;
        let first_proof = Arc::new(Mutex::new(None::<Dpop>));
        let captured_proof = Arc::clone(&first_proof);
        let par_challenge = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/par").is_true(move |request| {
                    if !request_nonce_matches(request, None) || !wia_challenge_matches(request, None) {
                        return false;
                    }
                    let proof = request
                        .headers()
                        .get(DPOP_HEADER_NAME)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    *captured_proof.lock().unwrap() = Some(proof);
                    true
                });
                then.status(400)
                    .header(DPOP_NONCE_HEADER_NAME, "par_nonce")
                    .header(WIA_CLIENT_CHALLENGE_HEADER_NAME, "par_retry_challenge")
                    .json_body(json!({"error":"use_dpop_nonce"}));
            })
            .await;
        let par_retry = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/par").is_true(|request| {
                    request_nonce_matches(request, Some("par_nonce"))
                        && wia_challenge_matches(request, Some("par_retry_challenge"))
                });
                if repeated_challenge {
                    then.status(400)
                        .header(DPOP_NONCE_HEADER_NAME, "another_nonce")
                        .json_body(json!({"error":"use_dpop_nonce"}));
                } else {
                    let then = then
                        .status(201)
                        .header(WIA_CLIENT_CHALLENGE_HEADER_NAME, "token_challenge");
                    let then = if rotate_nonce_on_success {
                        then.header(DPOP_NONCE_HEADER_NAME, "token_nonce")
                    } else {
                        then
                    };
                    then.json_body(json!({"request_uri":PAR_REQUEST_URI, "expires_in":60}));
                }
            })
            .await;
        let issuer_metadata = IssuerMetadata::new_mock(
            server.base_url().parse().unwrap(),
            vec![(
                "config_id".to_string().into(),
                CredentialKind::new(Format::SdJwt, "test".to_string()),
            )],
        );
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let mut endpoints = authorization_endpoints(&server.base_url().parse().unwrap());
        endpoints.challenge_endpoint = None;
        let session = HttpAuthorizationSession::<S256PkcePair>::create(
            client.clone(),
            issuer_metadata.credential_configurations_supported,
            issuer_metadata.credential_issuer.clone(),
            issuer_metadata.endpoints,
            IssuerRegistration::new_mock(),
            batch_size,
            None,
            endpoints,
            MOCK_WALLET_CLIENT_ID.to_string(),
            REDIRECT_URI.parse().unwrap(),
            None,
            &MockWiaClient::new(),
            issuer_metadata.credential_issuer,
        )
        .await;
        par_challenge.assert_calls_async(1).await;
        par_retry.assert_calls_async(1).await;
        if repeated_challenge {
            assert_matches!(session, Err(WalletIssuanceError::DpopNonceRejected));
            return;
        }
        let session = session.unwrap();
        let session = HttpAuthorizationSession::restore(
            client,
            serde_json::from_value(serde_json::to_value(session.persist()).unwrap()).unwrap(),
        );
        let expected_nonce = if rotate_nonce_on_success {
            "token_nonce"
        } else {
            "par_nonce"
        };
        assert_eq!(session.dpop.nonce.as_ref().unwrap().as_ref(), expected_nonce);
        let key = PublicKey::from(*session.dpop.signing_key.verifying_key());
        assert_eq!(
            first_proof
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .verify(&server.url("/issuance/par").parse().unwrap(), &Method::POST, None,)
                .unwrap(),
            key,
        );
        let token_url: Url = server.url("/issuance/token").parse().unwrap();
        let expected_key = key.clone();
        let expected_url = token_url.clone();
        let token_challenge = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token").is_true(move |request| {
                    let headers = request.headers();
                    let proof: Dpop = headers
                        .get(DPOP_HEADER_NAME)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    wia_challenge_matches(request, Some("token_challenge"))
                        && proof
                            .verify_expecting_key(expected_key.clone(), &expected_url, &Method::POST, None)
                            .is_ok_and(|nonce| nonce.into_inner().is_some_and(|nonce| nonce.as_ref() == expected_nonce))
                });
                let then = if rotate_wia_on_token_retry {
                    then.header(WIA_CLIENT_CHALLENGE_HEADER_NAME, "token_retry_challenge")
                } else {
                    then
                };
                then.status(400)
                    .header(DPOP_NONCE_HEADER_NAME, "token_retry_nonce")
                    .json_body(json!({"error":"use_dpop_nonce"}));
            })
            .await;
        let token_retry = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token").is_true(move |request| {
                    let headers = request.headers();
                    let proof: Dpop = headers
                        .get(DPOP_HEADER_NAME)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let expected_wia_challenge = if rotate_wia_on_token_retry {
                        "token_retry_challenge"
                    } else {
                        "token_challenge"
                    };
                    wia_challenge_matches(request, Some(expected_wia_challenge))
                        && proof
                            .verify_expecting_key(key.clone(), &token_url, &Method::POST, None)
                            .is_ok_and(|nonce| {
                                nonce
                                    .into_inner()
                                    .is_some_and(|nonce| nonce.as_ref() == "token_retry_nonce")
                            })
                });
                // Stop after token processing, without having to set up metadata and credential issuance.
                then.status(400).json_body(json!({"error":"invalid_grant"}));
            })
            .await;
        let redirect = url_with_query_pairs(
            REDIRECT_URI.parse().unwrap(),
            &[("code", CODE), ("state", session.state())],
        );
        let error = session
            .start_issuance(&redirect, &MockWiaClient::new())
            .await
            .unwrap_err();
        assert_matches!(error, WalletIssuanceError::VciTokenRequest(response)
            if response.error == RemoteErrorCode::Known(VciTokenErrorCode::InvalidGrant));
        token_challenge.assert_calls_async(1).await;
        token_retry.assert_calls_async(1).await;
    }
}
