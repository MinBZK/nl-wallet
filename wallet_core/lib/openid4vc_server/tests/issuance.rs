use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::slice::Iter;
use std::sync::Arc;

use attestation_data::credential_payload::CredentialPayload;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::BorrowingTrustAnchor;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::server::TlsServerConfig;
use indexmap::IndexSet;
use openid4vc::authorization::OidcAuthorizationRequest;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::dpop::DPOP_HEADER_NAME;
use openid4vc::dpop::Dpop;
use openid4vc::issuer::UpstreamAuthorizationAdapter;
use openid4vc::issuer::UpstreamResolveError;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::par::PAR_TTL;
use openid4vc::pkce::PKCE_FLOW_TTL;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::store::MemoryStore;
use openid4vc::test::MOCK_ATTESTATION_TYPES;
use openid4vc::test::MockAttrService;
use openid4vc::test::MockIssuer;
use openid4vc::test::mock_issuable_documents;
use openid4vc::test::setup_mock_issuer;
use openid4vc::token::AuthorizationCode;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredential;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use openid4vc::wallet_issuance::preview::NormalizedCredentialPreview;
use openid4vc_server::issuer::create_issuance_router;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePrivateKey;
use rand_core::OsRng;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::redirect::Policy;
use rstest::rstest;
use tokio::net::TcpListener;
use url::Url;
use utils::generator::TimeGenerator;
use wscd::mock_remote::MockRemoteWscd;

const MOCK_UPSTREAM_CLIENT_ID: &str = "mock_upstream_client_id";

struct StaticAuthorizationAdapter {
    url: Url,
    client_id: String,
    scopes: IndexSet<String>,
}

impl StaticAuthorizationAdapter {
    fn new(url: Url) -> Self {
        Self {
            url,
            client_id: MOCK_UPSTREAM_CLIENT_ID.to_string(),
            scopes: IndexSet::from_iter([String::from("openid")]),
        }
    }
}

impl UpstreamAuthorizationAdapter for StaticAuthorizationAdapter {
    async fn adapt(
        &self,
        mut request: VciAuthorizationRequest,
    ) -> Result<(Url, OidcAuthorizationRequest), UpstreamResolveError> {
        request.oauth_request.client_id = self.client_id.clone();
        request.scope = Some(self.scopes.clone());
        Ok((
            self.url.clone(),
            OidcAuthorizationRequest {
                vci_request: request,
                nonce: None,
            },
        ))
    }
}

fn generate_localhost_tls() -> (TlsServerConfig, ReqwestTrustAnchor) {
    let ca = Ca::generate("localhost", Default::default()).unwrap();
    let keypair = ca.generate_tls_mock("localhost").unwrap();

    let tls_config = TlsServerConfig {
        cert: keypair.certificate().as_ref().to_vec(),
        key: keypair.private_key().to_pkcs8_der().unwrap().as_bytes().to_vec(),
    };
    let trust_anchor = ReqwestTrustAnchor::try_from(ca.certificate().as_ref().to_vec()).unwrap();

    (tls_config, trust_anchor)
}

async fn start_server(
    attestation_count: NonZeroUsize,
    upstream_authorization_endpoint: Option<Url>,
) -> (
    Arc<
        MockIssuer<
            TimeGenerator,
            MemoryStore<String, VciAuthorizationRequest>,
            MemoryStore<String, String>,
            StaticAuthorizationAdapter,
        >,
    >,
    BorrowingTrustAnchor,
    IssuerIdentifier,
    KeyPair,
    ReqwestTrustAnchor,
) {
    let (tls_server_config, tls_trust_anchor) = generate_localhost_tls();

    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();
    let issuer_identifier: IssuerIdentifier = format!("https://localhost:{port}").parse().unwrap();

    let sessions = Arc::new(MemorySessionStore::default());
    let par_store = Arc::new(MemoryStore::new(PAR_TTL));
    let pkce_store = Arc::new(MemoryStore::new(PKCE_FLOW_TTL));

    let adapter = upstream_authorization_endpoint.map(StaticAuthorizationAdapter::new);
    let (issuer, trust_anchor, wia_issuer_keypair) = setup_mock_issuer(
        issuer_identifier.clone(),
        MockAttrService {
            documents: mock_issuable_documents(attestation_count),
        },
        attestation_count,
        sessions,
        par_store,
        pkce_store,
        adapter,
    );
    let issuer = Arc::new(issuer);

    let router = create_issuance_router(Arc::clone(&issuer));
    tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls_server_config.into_rustls_config().unwrap())
            .unwrap()
            .serve(router.into_make_service())
            .await
            .unwrap()
    });

    (
        issuer,
        trust_anchor,
        issuer_identifier,
        wia_issuer_keypair,
        tls_trust_anchor,
    )
}

fn make_credential_offer_url(
    issuer_identifier: IssuerIdentifier,
    session_token: SessionToken,
    attestation_count: NonZeroUsize,
) -> Url {
    let auth_code: AuthorizationCode = session_token.into();
    let credential_offer = CredentialOffer {
        credential_issuer: issuer_identifier,
        credential_configuration_ids: MOCK_ATTESTATION_TYPES[..attestation_count.get()]
            .iter()
            .map(|attestation_type| attestation_type.to_string().into())
            .collect(),
        grants: Some(Grants::PreAuthorizedCode {
            pre_authorized_code: GrantPreAuthorizedCode::new(auth_code),
        }),
    };
    let container = CredentialOfferContainer { credential_offer };
    let query = serde_urlencoded::to_string(&container).unwrap();
    format!("openid-credential-offer://?{query}").parse().unwrap()
}

fn verify_issued_credentials(
    issued_creds: Vec<CredentialWithMetadata>,
    normalized_credential_previews: Iter<NormalizedCredentialPreview>,
    expected_attestations: usize,
    expected_copies: usize,
) {
    assert_eq!(issued_creds.len(), expected_attestations);
    assert_eq!(
        issued_creds.first().unwrap().copies.as_ref().len().get(),
        expected_copies
    );

    issued_creds
        .into_iter()
        .zip(normalized_credential_previews)
        .for_each(|(credential, preview_data)| {
            credential
                .copies
                .into_inner()
                .into_iter()
                .for_each(|issued_credential| match issued_credential {
                    IssuedCredential::MsoMdoc { .. } => {
                        panic!("mdoc should not be issued");
                    }
                    IssuedCredential::SdJwt { sd_jwt, .. } => {
                        let payload =
                            CredentialPayload::from_sd_jwt(sd_jwt, &preview_data.normalized_metadata).unwrap();
                        assert_eq!(payload.previewable_payload, preview_data.content.credential_payload);
                    }
                })
        });
}

#[rstest]
#[tokio::test]
async fn authorization_code_flow(
    #[values(NonZeroUsize::MIN, NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let upstream_oauth_id: Url = "https://auth.example.com/".parse().unwrap();
    let (_, trust_anchor, issuer_identifier, wia_keypair, tls_trust_anchor) =
        start_server(attestation_count, Some(upstream_oauth_id)).await;

    let redirect_uri: Url = "https://wallet.example.com/callback".parse().unwrap();
    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor
            .clone()
            .into_certificate()]))
        .unwrap(),
    );

    // Start authorization code flow — fetches metadata and creates an auth session.
    let auth_session = discovery
        .start_authorization_code_flow(
            &issuer_identifier,
            MOCK_WALLET_CLIENT_ID.to_string(),
            redirect_uri.clone(),
        )
        .await
        .unwrap();

    // Auth URL must point to the authorize endpoint and carry PAR params (RFC 9126).
    assert!(auth_session.auth_url().as_str().starts_with(&format!(
        "{}issuance/authorize",
        issuer_identifier.as_base_url().as_ref().as_str()
    )));
    let auth_params: HashMap<String, String> = auth_session
        .auth_url()
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert!(auth_params.contains_key("request_uri"));
    assert!(!auth_params.contains_key("state"));

    // State is inside the PAR-stored request; read it from the session to craft the redirect.
    let state = auth_session.state().to_owned();

    // Simulate the user agent following the auth_url. The /authorize handler bridges PKCE
    // (substituting the wallet's challenge with one it holds the verifier for) and stores
    // that bridge entry, so /token can consume it later. We don't follow the upstream redirect.
    let browser_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();
    let authorize_response = browser_client
        .get(auth_session.auth_url().clone())
        .send()
        .await
        .unwrap();
    assert_eq!(authorize_response.status(), StatusCode::FOUND);

    // Simulate the authorization server redirecting back with a code and the matching state.
    // The issuer will treat any unknown code as a new (authorization-code-flow) session and
    // fetch documents from the attribute service, so any string works as the code here.
    let mut received_redirect = redirect_uri;
    received_redirect.set_query(Some(&format!("code=fake_auth_code&state={state}")));

    let trust_anchors = &[trust_anchor];
    let mut session = auth_session
        .start_issuance(&received_redirect, trust_anchors)
        .await
        .unwrap();

    assert_eq!(session.normalized_credential_preview().len(), attestation_count.get());

    let wscd = MockRemoteWscd::new_with_wia_keypair(wia_keypair);
    let issued_creds = session.accept_issuance(trust_anchors, &wscd, true).await.unwrap();

    let copy_count = 4;
    verify_issued_credentials(
        issued_creds,
        session.normalized_credential_preview().iter(),
        attestation_count.get(),
        copy_count,
    );
}

#[rstest]
#[tokio::test]
async fn pre_authorized_code_flow(
    #[values(NonZeroUsize::MIN, NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let (issuer, trust_anchor, issuer_identifier, wia_keypair, tls_trust_anchor) =
        start_server(attestation_count, None).await;

    let documents = mock_issuable_documents(attestation_count);
    let session_token = issuer.new_session(documents).await.unwrap();

    let credential_offer_url = make_credential_offer_url(issuer_identifier, session_token, attestation_count);

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])).unwrap(),
    );

    let trust_anchors = &[trust_anchor];
    let mut session = discovery
        .start_pre_authorized_code_flow(&credential_offer_url, MOCK_WALLET_CLIENT_ID.to_string(), trust_anchors)
        .await
        .unwrap();

    let copy_count = 4;
    let wscd = MockRemoteWscd::new_with_wia_keypair(wia_keypair);
    let issued_creds = session.accept_issuance(trust_anchors, &wscd, true).await.unwrap();

    verify_issued_credentials(
        issued_creds,
        session.normalized_credential_preview().iter(),
        attestation_count.get(),
        copy_count,
    );
}

#[tokio::test]
async fn reject_issuance() {
    let attestation_count = NonZeroUsize::MIN;
    let (issuer, trust_anchor, issuer_identifier, _, tls_trust_anchor) = start_server(attestation_count, None).await;

    let documents = mock_issuable_documents(attestation_count);
    let session_token = issuer.new_session(documents).await.unwrap();

    let offer_url = make_credential_offer_url(issuer_identifier, session_token, attestation_count);

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])).unwrap(),
    );

    let trust_anchors = &[trust_anchor];
    let session = discovery
        .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), trust_anchors)
        .await
        .unwrap();

    session.reject_issuance().await.unwrap();
}

#[tokio::test]
async fn par_rejects_unknown_client_id() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, Some(upstream_endpoint)).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let par_url = format!("{}issuance/par", issuer_identifier.as_base_url().as_ref().as_str());
    let response = http_client
        .post(&par_url)
        .form(&[
            ("response_type", "code"),
            ("client_id", "unknown_client_id"),
            ("code_challenge", "some-challenge"),
            ("code_challenge_method", "S256"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_client"), "unexpected body: {body}");
}

#[tokio::test]
async fn authorize_rewrites_client_id_for_upstream() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) =
        start_server(NonZeroUsize::MIN, Some(upstream_endpoint.clone())).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();

    let base = issuer_identifier.as_base_url().as_ref().as_str().to_string();

    // Step 1: PAR with the wallet client_id and an S256 PKCE challenge (required by /authorize).
    let wallet_code_challenge = "wallet-c1-challenge-value";
    let par_resp: PushedAuthorizationResponse = http_client
        .post(format!("{base}issuance/par"))
        .form(&[
            ("response_type", "code"),
            ("client_id", MOCK_WALLET_CLIENT_ID),
            ("code_challenge", wallet_code_challenge),
            ("code_challenge_method", "S256"),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Step 2: Authorize — should redirect to upstream with the upstream client_id.
    let mut authorize_url: Url = format!("{base}issuance/authorize").parse().unwrap();
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", MOCK_WALLET_CLIENT_ID)
        .append_pair("request_uri", &par_resp.request_uri);
    let response = http_client.get(authorize_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers().get("location").unwrap().to_str().unwrap();
    let location_url: Url = location.parse().unwrap();

    let params: HashMap<String, String> = location_url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // The forwarded request must carry the upstream (DigiD) client_id, not the wallet client_id.
    assert_eq!(params.get("client_id").unwrap(), MOCK_UPSTREAM_CLIENT_ID);
    // The wallet's code_challenge must NOT be forwarded — handler substitutes its own (v2,c2).
    assert_eq!(params.get("code_challenge_method").unwrap(), "S256");
    assert_ne!(params.get("code_challenge").unwrap(), wallet_code_challenge);
    assert_ne!(params.get("client_id").unwrap(), MOCK_WALLET_CLIENT_ID);
    assert!(location.starts_with(upstream_endpoint.as_str()));
}

#[tokio::test]
async fn par_rejects_missing_code_challenge() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, Some(upstream_endpoint)).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let base = issuer_identifier.as_base_url().as_ref().as_str().to_string();

    // `code_challenge` is mandatory on `VciAuthorizationRequest`; absence makes the form fail to
    // deserialize at the /par boundary (HTTP 422), rather than slipping through to /authorize.
    let response = http_client
        .post(format!("{base}issuance/par"))
        .form(&[("response_type", "code"), ("client_id", MOCK_WALLET_CLIENT_ID)])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn authorize_rejects_plain_code_challenge_method() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, Some(upstream_endpoint)).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();

    let base = issuer_identifier.as_base_url().as_ref().as_str().to_string();

    // PAR with `code_challenge_method=plain` — the handler bridges PKCE only when S256 is used.
    let par_resp: PushedAuthorizationResponse = http_client
        .post(format!("{base}issuance/par"))
        .form(&[
            ("response_type", "code"),
            ("client_id", MOCK_WALLET_CLIENT_ID),
            ("code_challenge", "wallet-c1-challenge-value"),
            ("code_challenge_method", "plain"),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut authorize_url: Url = format!("{base}issuance/authorize").parse().unwrap();
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", MOCK_WALLET_CLIENT_ID)
        .append_pair("request_uri", &par_resp.request_uri);
    let response = http_client.get(authorize_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_request"), "unexpected body: {body}");
    assert!(body.contains("S256"), "unexpected body: {body}");
}

#[tokio::test]
async fn authorize_rejects_unknown_request_uri() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, Some(upstream_endpoint)).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();

    let mut authorize_url: Url = format!(
        "{}issuance/authorize",
        issuer_identifier.as_base_url().as_ref().as_str()
    )
    .parse()
    .unwrap();
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", MOCK_WALLET_CLIENT_ID)
        .append_pair("request_uri", "urn:ietf:params:oauth:request_uri:not-stored");
    let response = http_client.get(authorize_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_request"), "unexpected body: {body}");
    assert!(body.contains("request_uri not found"), "unexpected body: {body}");
}

#[tokio::test]
async fn authorize_rejects_unknown_client_id() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, Some(upstream_endpoint)).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();

    let mut authorize_url: Url = format!(
        "{}issuance/authorize",
        issuer_identifier.as_base_url().as_ref().as_str()
    )
    .parse()
    .unwrap();
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", "definitely-not-the-wallet")
        .append_pair("request_uri", "urn:ietf:params:oauth:request_uri:doesnt-matter");
    let response = http_client.get(authorize_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_client"), "unexpected body: {body}");
}

/// Builds a parseable DPoP header for `POST {token_url}`. The PKCE check in the `/token`
/// handler runs before DPoP semantic verification, so the value just has to deserialize —
/// the URL/method don't need to match what the handler will later try to verify against.
fn dpop_header_for(token_url: &Url) -> String {
    let signing_key = SigningKey::random(&mut OsRng);
    Dpop::new(&signing_key, token_url.clone(), &Method::POST, None, None)
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn token_rejects_missing_code_verifier() {
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, None).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let token_url: Url = format!("{}issuance/token", issuer_identifier.as_base_url().as_ref().as_str())
        .parse()
        .unwrap();

    // AuthorizationCode grant without a code_verifier — the handler must reject before
    // ever consulting `PkceFlowStore` or `process_token_request`.
    let token_request = TokenRequest {
        grant_type: TokenRequestGrantType::AuthorizationCode {
            code: "any-code".to_string().into(),
        },
        code_verifier: None,
        client_id: None,
        redirect_uri: None,
    };

    let response = http_client
        .post(token_url.clone())
        .header(DPOP_HEADER_NAME, dpop_header_for(&token_url))
        .form(&token_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_grant"), "unexpected body: {body}");
    assert!(body.contains("missing code_verifier"), "unexpected body: {body}");
}

#[tokio::test]
async fn token_rejects_unknown_code_verifier() {
    let (_, _, issuer_identifier, _, tls_trust_anchor) = start_server(NonZeroUsize::MIN, None).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let token_url: Url = format!("{}issuance/token", issuer_identifier.as_base_url().as_ref().as_str())
        .parse()
        .unwrap();

    // AuthorizationCode grant with a `code_verifier` whose SHA256 hash is not in `PkceFlowStore`
    // (since we skipped /authorize, the store is empty). The handler must return invalid_grant.
    let token_request = TokenRequest {
        grant_type: TokenRequestGrantType::AuthorizationCode {
            code: "any-code".to_string().into(),
        },
        code_verifier: Some("a-verifier-no-one-stored".to_string()),
        client_id: None,
        redirect_uri: None,
    };

    let response = http_client
        .post(token_url.clone())
        .header(DPOP_HEADER_NAME, dpop_header_for(&token_url))
        .form(&token_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_grant"), "unexpected body: {body}");
    assert!(body.contains("PKCE verification failed"), "unexpected body: {body}");
}
