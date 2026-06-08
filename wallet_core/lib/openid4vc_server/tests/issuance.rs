use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::slice::Iter;
use std::sync::Arc;

use attestation_data::credential_payload::CredentialPayload;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::server::TlsServerConfig;
use openid4vc::credential_offer::CredentialOffer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::credential_offer::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::dpop::DPOP_HEADER_NAME;
use openid4vc::dpop::Dpop;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::test::MOCK_ATTESTATION_TYPES;
use openid4vc::test::MockAuthorizingIssuer;
use openid4vc::test::MockIssuer;
use openid4vc::test::StaticAuthorizationCodeFlow;
use openid4vc::test::mock_issuable_documents;
use openid4vc::test::setup_mock_authorizing_issuer;
use openid4vc::test::setup_mock_issuer;
use openid4vc::token::AuthorizationCode;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceFlow;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredential;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use openid4vc::wallet_issuance::preview::NormalizedCredentialPreview;
use openid4vc_server::issuer::create_authorization_router;
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
use utils::vec_nonempty;
use wscd::mock_remote::MockRemoteWscd;

fn generate_localhost_tls() -> (TlsServerConfig, ReqwestTrustAnchor) {
    let ca = Ca::generate("localhost", Default::default()).unwrap();
    let keypair = ca.generate_tls_mock("localhost").unwrap();

    let tls_config = TlsServerConfig {
        cert: keypair.certificate().as_ref().to_vec(),
        key: keypair.private_key().to_pkcs8_der().unwrap().as_bytes().to_vec(),
    };
    let trust_anchors = ReqwestTrustAnchor::try_from(ca.certificate().as_ref().to_vec()).unwrap();

    (tls_config, trust_anchors)
}

/// Type alias for the [`AuthorizingIssuer`] flavor used in these auth-code-flow tests.
type TestAuthorizingIssuer = MockAuthorizingIssuer<TimeGenerator>;

/// Bundle returned by `start_auth_code_flow_server` so tests can call
/// [`StaticAuthorizationCodeFlow::fake_complete_authorization`] after `/authorize`.
struct AuthCodeFlowServer {
    authorizing_issuer: Arc<TestAuthorizingIssuer>,
    trust_anchors: TrustAnchors,
    issuer_identifier: IssuerIdentifier,
    wia_keypair: KeyPair,
    tls_trust_anchor: ReqwestTrustAnchor,
}

async fn start_auth_code_flow_server(attestation_count: NonZeroUsize, upstream: Url) -> AuthCodeFlowServer {
    let (tls_server_config, tls_trust_anchor) = generate_localhost_tls();

    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();
    let issuer_identifier: IssuerIdentifier = format!("https://localhost:{port}").parse().unwrap();

    let sessions = Arc::new(MemorySessionStore::default());
    let mock_documents = mock_issuable_documents(attestation_count);

    let flow = StaticAuthorizationCodeFlow::new(upstream, mock_documents);
    let (authorizing_issuer, trust_anchors, wia_keypair) = setup_mock_authorizing_issuer(
        issuer_identifier.clone(),
        attestation_count,
        sessions,
        flow,
        vec_nonempty!["https://wallet.example.com/callback".parse().unwrap()],
    );
    let authorizing_issuer = Arc::new(authorizing_issuer);
    let issuer = Arc::clone(authorizing_issuer.issuer());

    // Authorization-code flow mounts the issuance endpoints (including `/token`) alongside the
    // authorization router (PAR + `/authorize`).
    let router = create_issuance_router(issuer).merge(create_authorization_router(Arc::clone(&authorizing_issuer)));

    tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls_server_config.into_rustls_config().unwrap())
            .unwrap()
            .serve(router.into_make_service())
            .await
            .unwrap()
    });

    AuthCodeFlowServer {
        authorizing_issuer,
        trust_anchors,
        issuer_identifier,
        wia_keypair,
        tls_trust_anchor,
    }
}

async fn start_pre_authorized_code_flow_server(
    attestation_count: NonZeroUsize,
) -> (
    Arc<MockIssuer<TimeGenerator>>,
    TrustAnchors,
    IssuerIdentifier,
    KeyPair,
    ReqwestTrustAnchor,
) {
    let (tls_server_config, tls_trust_anchor) = generate_localhost_tls();

    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();
    let issuer_identifier: IssuerIdentifier = format!("https://localhost:{port}").parse().unwrap();

    let sessions = Arc::new(MemorySessionStore::default());

    let (issuer, trust_anchors, wia_keypair) =
        setup_mock_issuer(issuer_identifier.clone(), attestation_count, sessions);
    let issuer = Arc::new(issuer);

    // Pre-authorized-code flow mounts the issuance endpoints standalone.
    let router = create_issuance_router(Arc::clone(&issuer));

    tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls_server_config.into_rustls_config().unwrap())
            .unwrap()
            .serve(router.into_make_service())
            .await
            .unwrap()
    });

    (issuer, trust_anchors, issuer_identifier, wia_keypair, tls_trust_anchor)
}

fn make_credential_offer_url(
    issuer_identifier: IssuerIdentifier,
    attestation_count: NonZeroUsize,
    pre_authorized_code: Option<AuthorizationCode>,
) -> Url {
    let config_ids = MOCK_ATTESTATION_TYPES[..attestation_count.get()]
        .iter()
        .map(|attestation_type| attestation_type.to_string().into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let credential_offer = match pre_authorized_code {
        None => CredentialOffer::new_authorization(issuer_identifier, config_ids, None),
        Some(pre_authorized_code) => {
            CredentialOffer::new_pre_authorized(issuer_identifier, config_ids, pre_authorized_code)
        }
    };
    let offer_container = CredentialOfferContainer::new_offer(credential_offer);
    let query = serde_urlencoded::to_string(&offer_container).unwrap();

    let mut url = Url::parse(&format!("{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://")).unwrap();
    url.set_query(Some(&query));

    url
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
    let AuthCodeFlowServer {
        authorizing_issuer,
        trust_anchors,
        issuer_identifier,
        wia_keypair,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(attestation_count, upstream_oauth_id).await;

    let credential_offer_url = make_credential_offer_url(issuer_identifier.clone(), attestation_count, None);
    let redirect_uri = Url::parse("https://wallet.example.com/callback").unwrap();

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor
            .clone()
            .into_certificate()]))
        .unwrap(),
    );

    // Start authorization code flow — fetches metadata and creates an auth session.
    let flow = discovery
        .start(
            &credential_offer_url,
            MOCK_WALLET_CLIENT_ID.to_string(),
            redirect_uri.clone(),
            &trust_anchors,
        )
        .await
        .unwrap();

    let IssuanceFlow::AuthorizationCode {
        authorization_session: auth_session,
    } = flow
    else {
        panic!("should have received Authorization Code flow");
    };

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

    // Simulate the user agent following the auth_url. The /authorize handler captures the
    // wallet's PKCE challenge + redirect_uri + state into the AF's single-slot cell, then
    // returns a 302 to the upstream URL. We don't actually follow the upstream redirect — we
    // synthesize the upstream callback below by calling `fake_complete_authorization`.
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

    // Plant the `AuthCodeIssued` session that the wallet's `/token` call will load. This stands in
    // for what the real upstream callback handler does (BSN → BRP → issuables →
    // complete_authorization).
    let (issuer_code, _captured) = authorizing_issuer
        .flow()
        .fake_complete_authorization(&authorizing_issuer)
        .await;

    let mut received_redirect = redirect_uri;
    received_redirect.set_query(Some(&format!("code={}&state={state}", issuer_code.as_ref())));

    let mut session = auth_session
        .start_issuance(&received_redirect, &trust_anchors)
        .await
        .unwrap();

    assert_eq!(session.normalized_credential_preview().len(), attestation_count.get());

    let wscd = MockRemoteWscd::new_with_wia_keypair(wia_keypair);
    let issued_creds = session.accept_issuance(&trust_anchors, &wscd, true).await.unwrap();

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
    let (issuer, trust_anchors, issuer_identifier, wia_keypair, tls_trust_anchor) =
        start_pre_authorized_code_flow_server(attestation_count).await;

    let documents = mock_issuable_documents(attestation_count);
    let session_token = issuer.new_preauthorized_session(documents).await.unwrap();

    let credential_offer_url =
        make_credential_offer_url(issuer_identifier, attestation_count, Some(session_token.into()));

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])).unwrap(),
    );

    let flow = discovery
        .start(
            &credential_offer_url,
            MOCK_WALLET_CLIENT_ID.to_string(),
            "https://wallet.example.com/callback".parse().unwrap(),
            &trust_anchors,
        )
        .await
        .unwrap();

    let IssuanceFlow::PreAuthorizedCode {
        issuance_session: mut session,
    } = flow
    else {
        panic!("should have received Pre-Authorized Code flow");
    };

    let copy_count = 4;
    let wscd = MockRemoteWscd::new_with_wia_keypair(wia_keypair);
    let issued_creds = session.accept_issuance(&trust_anchors, &wscd, true).await.unwrap();

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
    let (issuer, trust_anchors, issuer_identifier, _, tls_trust_anchor) =
        start_pre_authorized_code_flow_server(attestation_count).await;

    let documents = mock_issuable_documents(attestation_count);
    let session_token = issuer.new_preauthorized_session(documents).await.unwrap();

    let offer_url = make_credential_offer_url(issuer_identifier, attestation_count, Some(session_token.into()));

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])).unwrap(),
    );

    let flow = discovery
        .start(
            &offer_url,
            MOCK_WALLET_CLIENT_ID.to_string(),
            "https://wallet.example.com/callback".parse().unwrap(),
            &trust_anchors,
        )
        .await
        .unwrap();

    let IssuanceFlow::PreAuthorizedCode {
        issuance_session: session,
    } = flow
    else {
        panic!("should have received Pre-Authorized Code flow");
    };

    session.reject_issuance().await.unwrap();
}

#[tokio::test]
async fn par_rejects_unknown_client_id() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let par_url = format!("{}issuance/par", issuer_identifier.as_base_url().as_ref().as_str());
    let response = http_client
        .post(&par_url)
        .form(&[
            ("response_type", "code"),
            ("client_id", "unknown_client_id"),
            ("redirect_uri", "https://wallet.example.com/callback"),
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
async fn par_rejects_missing_code_challenge() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

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
async fn authorize_rejects_unknown_request_uri() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

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
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

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

/// Plant an `AuthCodeIssued` session against the given authorizing_issuer using a freshly generated
/// PKCE pair, returning the (issuer-generated code, code_verifier) the caller can use to drive
/// `/token`. The wallet redirect URL the framework builds is discarded — these tests don't
/// follow the wallet redirect, they call `/token` directly.
async fn plant_authorized_session(authorizing_issuer: &TestAuthorizingIssuer) -> (AuthorizationCode, String) {
    let pair = S256PkcePair::generate();
    let challenge = pair.code_challenge().to_string();
    let verifier = pair.into_code_verifier();
    let documents = mock_issuable_documents(NonZeroUsize::MIN);
    let redirect_url = authorizing_issuer
        .complete_authorization(
            documents,
            challenge,
            "https://wallet.example.com/callback".parse().unwrap(),
            None,
        )
        .await
        .unwrap();

    // Extract the authorization code from the redirect URL.
    let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
    let authorization_code: AuthorizationCode = params.get("code").unwrap().clone().into();

    (authorization_code, verifier)
}

#[tokio::test]
async fn token_rejects_missing_code_verifier() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let AuthCodeFlowServer {
        authorizing_issuer,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

    let (code, _verifier) = plant_authorized_session(&authorizing_issuer).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let token_url: Url = format!("{}issuance/token", issuer_identifier.as_base_url().as_ref().as_str())
        .parse()
        .unwrap();

    let token_request = TokenRequest {
        grant_type: TokenRequestGrantType::AuthorizationCode { code },
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
}

#[tokio::test]
async fn token_rejects_unknown_code_verifier() {
    let upstream_endpoint: Url = "https://auth.example.com/oauth2/authorize".parse().unwrap();
    let AuthCodeFlowServer {
        authorizing_issuer,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN, upstream_endpoint).await;

    let (code, _verifier) = plant_authorized_session(&authorizing_issuer).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let token_url: Url = format!("{}issuance/token", issuer_identifier.as_base_url().as_ref().as_str())
        .parse()
        .unwrap();

    let token_request = TokenRequest {
        grant_type: TokenRequestGrantType::AuthorizationCode { code },
        code_verifier: Some("a-verifier-the-issuer-does-not-have".to_string()),
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
}
