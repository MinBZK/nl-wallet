use std::assert_matches;
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
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::credential_offer::CredentialOffer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::dpop::DPOP_HEADER_NAME;
use openid4vc::dpop::Dpop;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::test::AlwaysAuthorizingFlow;
use openid4vc::test::MOCK_ATTESTATION_TYPES;
use openid4vc::test::MOCK_ATTRS;
use openid4vc::test::MockAuthorizingIssuer;
use openid4vc::test::MockIssuer;
use openid4vc::test::mock_issuable_document_with_attrs;
use openid4vc::test::mock_issuable_documents;
use openid4vc::test::mock_type_metadata;
use openid4vc::test::mock_type_metadata_with_required_attr;
use openid4vc::test::setup_mock_authorizing_issuer_from_type_metadata;
use openid4vc::test::setup_mock_issuer;
use openid4vc::token::AuthorizationCode;
use openid4vc::token::CredentialPreview;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceFlow;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::WalletIssuanceError;
use openid4vc::wallet_issuance::credential::CredentialWithMetadata;
use openid4vc::wallet_issuance::credential::IssuedCredential;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use openid4vc::wallet_issuance::issuance_session::HttpIssuanceSession;
use openid4vc::wallet_issuance::issuance_session::IssuanceTypeMetadata;
use openid4vc_server::issuer::create_authorization_router;
use openid4vc_server::issuer::create_issuance_router;
use p256::ecdsa::SigningKey;
use p256::pkcs8::EncodePrivateKey;
use rand_core::OsRng;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::redirect::Policy;
use rstest::rstest;
use sd_jwt_vc_metadata::TypeMetadata;
use tokio::net::TcpListener;
use url::Url;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wscd::mock_remote::MockRemoteWscd;

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

/// Bundle returned by `start_auth_code_flow_server`. The `authorizing_issuer` is exposed so tests
/// can plant `AuthCodeIssued` sessions directly via [`MockAuthorizingIssuer::complete_authorization`].
struct AuthCodeFlowServer {
    authorizing_issuer: Arc<MockAuthorizingIssuer>,
    trust_anchors: TrustAnchors,
    issuer_identifier: IssuerIdentifier,
    wia_keypair: KeyPair,
    tls_trust_anchor: ReqwestTrustAnchor,
}

/// Bundle returned by `start_pre_authorized_code_flow_server`.
struct PreAuthCodeFlowServer {
    issuer: Arc<MockIssuer<TimeGenerator>>,
    trust_anchors: TrustAnchors,
    issuer_identifier: IssuerIdentifier,
    wia_keypair: KeyPair,
    tls_trust_anchor: ReqwestTrustAnchor,
}

async fn start_auth_code_flow_server(attestation_count: NonZeroUsize) -> AuthCodeFlowServer {
    let type_metadata = MOCK_ATTESTATION_TYPES[..attestation_count.get()]
        .iter()
        .map(|attestation_type| mock_type_metadata(attestation_type))
        .collect();

    start_auth_code_flow_server_with(type_metadata, mock_issuable_documents(attestation_count)).await
}

/// Like [`start_auth_code_flow_server`], but with custom type metadata and custom documents to authorize.
async fn start_auth_code_flow_server_with(
    type_metadata: Vec<TypeMetadata>,
    documents: VecNonEmpty<IssuableDocument>,
) -> AuthCodeFlowServer {
    let (tls_server_config, tls_trust_anchor) = generate_localhost_tls();

    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();
    let issuer_identifier: IssuerIdentifier = format!("https://localhost:{port}").parse().unwrap();

    let sessions = Arc::new(MemorySessionStore::default());

    let flow = AlwaysAuthorizingFlow::new(documents);
    let (authorizing_issuer, trust_anchors, wia_keypair) = setup_mock_authorizing_issuer_from_type_metadata(
        issuer_identifier.clone(),
        type_metadata,
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

async fn start_pre_authorized_code_flow_server(attestation_count: NonZeroUsize) -> PreAuthCodeFlowServer {
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

    PreAuthCodeFlowServer {
        issuer,
        trust_anchors,
        issuer_identifier,
        wia_keypair,
        tls_trust_anchor,
    }
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

    CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url()
}

fn verify_issued_credentials(
    issued_creds: Vec<CredentialWithMetadata>,
    credential_previews: Iter<CredentialPreview>,
    type_metadata: &HashMap<CredentialConfigurationId, IssuanceTypeMetadata>,
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
        .zip(credential_previews)
        .for_each(|(credential, preview_data)| {
            let normalized_metadata = &type_metadata
                .get(&preview_data.config_id)
                .expect("credential type metadata is missing")
                .normalized_metadata;
            credential
                .copies
                .into_inner()
                .into_iter()
                .for_each(|issued_credential| match issued_credential {
                    IssuedCredential::MsoMdoc { .. } => {
                        panic!("mdoc should not be issued");
                    }
                    IssuedCredential::SdJwt { sd_jwt, .. } => {
                        let payload = CredentialPayload::from_sd_jwt(sd_jwt, normalized_metadata).unwrap();
                        assert_eq!(payload.previewable_payload, preview_data.credential_payload);
                    }
                })
        });
}

/// Simulate wallet issuance, by going through discovery, the `/authorize` redirect (playing the user-agent)
/// and the wallet redirect, returning the issuance session holding the credential previews.
async fn start_issuance_session(server: &AuthCodeFlowServer, attestation_count: NonZeroUsize) -> HttpIssuanceSession {
    let credential_offer_url = make_credential_offer_url(server.issuer_identifier.clone(), attestation_count, None);
    let redirect_uri = Url::parse("https://wallet.example.com/callback").unwrap();

    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([server
            .tls_trust_anchor
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
            &server.trust_anchors,
        )
        .await
        .unwrap();
    let IssuanceFlow::AuthorizationCode {
        authorization_session: auth_session,
    } = flow
    else {
        panic!("should have received Authorization Code flow");
    };

    // Auth URL must point to the authorize endpoint and carry PAR params.
    assert!(auth_session.auth_url().as_str().starts_with(&format!(
        "{}issuance/authorize",
        server.issuer_identifier.as_base_url().as_ref().as_str()
    )));
    let auth_params: HashMap<String, String> = auth_session
        .auth_url()
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert!(auth_params.contains_key("request_uri"));
    assert!(!auth_params.contains_key("state"));

    // State is inside the PAR-stored request; read it from the session to check the redirect.
    let state = auth_session.state().to_owned();

    // Simulate the user agent following the auth_url. `AlwaysAuthorizingFlow` authorizes
    // synchronously, so the /authorize handler writes the `AuthCodeIssued` session and 302s the
    // user-agent straight back to the wallet's redirect_uri with the issuer-generated code and the
    // echoed state.
    let browser_client = tls_reqwest_client_builder([server.tls_trust_anchor.clone().into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();
    let authorize_response = browser_client
        .get(auth_session.auth_url().clone())
        .send()
        .await
        .unwrap();
    assert_eq!(authorize_response.status(), StatusCode::FOUND);

    let received_redirect: Url = authorize_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("authorize response should carry a Location header")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(received_redirect.as_str().starts_with(redirect_uri.as_str()));
    let redirect_params: HashMap<_, _> = received_redirect.query_pairs().into_owned().collect();
    assert_eq!(redirect_params.get("state"), Some(&state));

    auth_session
        .start_issuance(&received_redirect, &server.trust_anchors)
        .await
        .unwrap()
}

#[rstest]
#[tokio::test]
async fn authorization_code_flow(
    #[values(NonZeroUsize::MIN, NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let server = start_auth_code_flow_server(attestation_count).await;
    let mut session = start_issuance_session(&server, attestation_count).await;

    assert_eq!(session.credential_previews().len(), attestation_count.get());

    let wscd = MockRemoteWscd::new_with_wia_keypair(server.wia_keypair);
    let issued_creds = session
        .accept_issuance(&server.trust_anchors, &wscd, true)
        .await
        .unwrap();

    let copy_count = 4;
    verify_issued_credentials(
        issued_creds,
        session.credential_previews().iter(),
        session.type_metadata(),
        attestation_count.get(),
        copy_count,
    );
}

/// The issuer accepts documents omitting attributes the type-metadata schema does not require.
#[tokio::test]
async fn ltc1_issuance_allows_missing_optional_attribute() {
    let (optional_attr, _) = MOCK_ATTRS[0];
    let (required_attr, _) = MOCK_ATTRS[1];

    let type_metadata = mock_type_metadata_with_required_attr(MOCK_ATTESTATION_TYPES[0], required_attr);
    // The document carries only the required attribute; the optional one is absent.
    let documents = vec_nonempty![mock_issuable_document_with_attrs(
        MOCK_ATTESTATION_TYPES[0],
        &[MOCK_ATTRS[1]]
    )];
    let server = start_auth_code_flow_server_with(vec![type_metadata], documents).await;

    let mut session = start_issuance_session(&server, NonZeroUsize::MIN).await;

    let previews = session.credential_previews();
    assert_eq!(previews.len(), 1);
    let attributes = previews[0].credential_payload.attributes.as_ref();
    assert!(attributes.get(required_attr).is_some());
    assert!(attributes.get(optional_attr).is_none());

    let wscd = MockRemoteWscd::new_with_wia_keypair(server.wia_keypair);
    let issued_creds = session
        .accept_issuance(&server.trust_anchors, &wscd, true)
        .await
        .expect("issuance of a document missing only an optional attribute should succeed");

    verify_issued_credentials(
        issued_creds,
        session.credential_previews().iter(),
        session.type_metadata(),
        1,
        4,
    );
}

/// Issuing a document that lacks an attribute the type-metadata schema marks as required fails at
/// credential issuance, surfacing to the wallet as a `CredentialRequest` error response carrying
/// the schema violation.
#[tokio::test]
async fn ltc2_issuance_rejects_missing_required_attribute() {
    let (required_attr, _) = MOCK_ATTRS[1];

    let type_metadata = mock_type_metadata_with_required_attr(MOCK_ATTESTATION_TYPES[0], required_attr);
    // The document carries only the optional attribute; the required one is missing.
    let documents = vec_nonempty![mock_issuable_document_with_attrs(
        MOCK_ATTESTATION_TYPES[0],
        &[MOCK_ATTRS[0]]
    )];
    let server = start_auth_code_flow_server_with(vec![type_metadata], documents).await;

    let mut session = start_issuance_session(&server, NonZeroUsize::MIN).await;

    let wscd = MockRemoteWscd::new_with_wia_keypair(server.wia_keypair);
    let error = session
        .accept_issuance(&server.trust_anchors, &wscd, true)
        .await
        .expect_err("issuance of a document missing a required attribute should fail");

    assert_matches!(
        error,
        WalletIssuanceError::CredentialRequest(response) if matches!(
            &response.error_description,
            Some(description) if description.contains(&format!("\"{required_attr}\" is a required property"))
        )
    );
}

#[rstest]
#[tokio::test]
async fn pre_authorized_code_flow(
    #[values(NonZeroUsize::MIN, NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let PreAuthCodeFlowServer {
        issuer,
        trust_anchors,
        issuer_identifier,
        wia_keypair,
        tls_trust_anchor,
    } = start_pre_authorized_code_flow_server(attestation_count).await;

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
        session.credential_previews().iter(),
        session.type_metadata(),
        attestation_count.get(),
        copy_count,
    );
}

#[tokio::test]
async fn reject_issuance() {
    let attestation_count = NonZeroUsize::MIN;
    let PreAuthCodeFlowServer {
        issuer,
        trust_anchors,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_pre_authorized_code_flow_server(attestation_count).await;

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
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

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
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

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
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

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
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

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

#[tokio::test]
async fn authorize_rejects_unsupported_code_challenge_method() {
    let AuthCodeFlowServer {
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .redirect(Policy::none())
        .build()
        .unwrap();

    let base = issuer_identifier.as_base_url().as_ref().as_str().to_string();

    // PAR doesn't validate the code_challenge_method, so a `plain` request is stored and yields a
    // request_uri; the rejection is the `openid4vc` layer's job at /authorize, uniformly for every flow.
    let par_response = http_client
        .post(format!("{base}issuance/par"))
        .form(&[
            ("response_type", "code"),
            ("client_id", MOCK_WALLET_CLIENT_ID),
            ("redirect_uri", "https://wallet.example.com/callback"),
            ("code_challenge", "plain-challenge"),
            ("code_challenge_method", "plain"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(par_response.status(), StatusCode::CREATED);
    let request_uri = par_response
        .json::<PushedAuthorizationResponse>()
        .await
        .unwrap()
        .request_uri;

    let mut authorize_url: Url = format!("{base}issuance/authorize").parse().unwrap();
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", MOCK_WALLET_CLIENT_ID)
        .append_pair("request_uri", &request_uri);
    let response = http_client.get(authorize_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.text().await.unwrap();
    assert!(body.contains("invalid_request"), "unexpected body: {body}");
    assert!(body.contains("code_challenge_method"), "unexpected body: {body}");
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

/// Plant an `AuthCodeIssued` session against the given authorizing_issuer, returning the
/// issuer-generated code the caller can use to drive `/token`.
async fn plant_authorized_session(authorizing_issuer: &MockAuthorizingIssuer) -> AuthorizationCode {
    let documents = mock_issuable_documents(NonZeroUsize::MIN);
    let redirect_url = authorizing_issuer
        .complete_authorization(
            documents,
            WalletAuthorizationContext {
                redirect_uri: "https://wallet.example.com/callback".parse().unwrap(),
                state: None,
                code_challenge: "irrelevant-challenge".to_string(),
            },
        )
        .await
        .unwrap();

    // Extract the authorization code from the redirect URL.
    let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
    params.get("code").unwrap().clone().into()
}

#[tokio::test]
async fn token_rejects_missing_code_verifier() {
    let AuthCodeFlowServer {
        authorizing_issuer,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

    let code = plant_authorized_session(&authorizing_issuer).await;

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
    let AuthCodeFlowServer {
        authorizing_issuer,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

    let code = plant_authorized_session(&authorizing_issuer).await;

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

#[tokio::test]
async fn token_rejects_grant_type_mismatch() {
    let AuthCodeFlowServer {
        authorizing_issuer,
        issuer_identifier,
        tls_trust_anchor,
        ..
    } = start_auth_code_flow_server(NonZeroUsize::MIN).await;

    // Plant an authorization-code session, then try to redeem its code using the pre-authorized-code
    // grant. The code is placed in the `pre-authorized_code` field so the session is still found, and
    // the grant-type mismatch is what the handler must reject.
    let code = plant_authorized_session(&authorizing_issuer).await;

    let http_client = tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])
        .build()
        .unwrap();

    let token_url: Url = format!("{}issuance/token", issuer_identifier.as_base_url().as_ref().as_str())
        .parse()
        .unwrap();

    let token_request = TokenRequest {
        grant_type: TokenRequestGrantType::PreAuthorizedCode {
            pre_authorized_code: code,
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
    assert!(body.contains("unsupported_grant_type"), "unexpected body: {body}");
}
