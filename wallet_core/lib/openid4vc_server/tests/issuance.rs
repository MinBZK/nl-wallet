use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::slice::Iter;
use std::sync::Arc;

use attestation_data::credential_payload::CredentialPayload;
use crypto::server_keys::generate::Ca;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::ReqwestTrustAnchor;
use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::server::TlsServerConfig;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::test::MOCK_ATTESTATION_TYPES;
use openid4vc::test::MockAttrService;
use openid4vc::test::MockIssuer;
use openid4vc::test::mock_issuable_documents;
use openid4vc::test::setup_mock_issuer;
use openid4vc::token::AuthorizationCode;
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
use rstest::rstest;
use rustls_pki_types::TrustAnchor;
use tokio::net::TcpListener;
use url::Url;
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

async fn start_server(
    attestation_count: NonZeroUsize,
    upstream_oauth_identifier: Option<IssuerIdentifier>,
) -> (
    Arc<MockIssuer>,
    TrustAnchor<'static>,
    IssuerIdentifier,
    SigningKey,
    ReqwestTrustAnchor,
) {
    let (tls_server_config, tls_trust_anchor) = generate_localhost_tls();

    let listener = TcpListener::bind("localhost:0").await.unwrap().into_std().unwrap();
    let port = listener.local_addr().unwrap().port();
    let issuer_identifier: IssuerIdentifier = format!("https://localhost:{port}").parse().unwrap();

    let sessions = Arc::new(MemorySessionStore::default());
    let (issuer, trust_anchor, wia_issuer_privkey) = setup_mock_issuer(
        issuer_identifier.clone(),
        MockAttrService {
            documents: mock_issuable_documents(attestation_count),
        },
        attestation_count,
        sessions,
        upstream_oauth_identifier,
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
        wia_issuer_privkey,
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
            .map(ToString::to_string)
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
                    IssuedCredential::MsoMdoc { mdoc } => {
                        let payload = CredentialPayload::from_mdoc(mdoc, &preview_data.normalized_metadata).unwrap();
                        assert_eq!(payload.previewable_payload, preview_data.content.credential_payload);
                    }
                    IssuedCredential::SdJwt { .. } => {
                        panic!("SdJwt should not be issued");
                    }
                })
        });
}

#[rstest]
#[tokio::test]
async fn authorization_code_flow(
    #[values(NonZeroUsize::MIN, NonZeroUsize::new(2).unwrap())] attestation_count: NonZeroUsize,
) {
    let upstream_oauth_id: IssuerIdentifier = "https://auth.example.com/".parse().unwrap();
    let (_, trust_anchor, issuer_identifier, wia_issuer_privkey, tls_trust_anchor) =
        start_server(attestation_count, Some(upstream_oauth_id)).await;

    let redirect_uri: Url = "https://wallet.example.com/callback".parse().unwrap();
    let discovery = HttpIssuanceDiscovery::new(
        HttpJsonClient::try_new(tls_reqwest_client_builder([tls_trust_anchor.into_certificate()])).unwrap(),
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

    // Auth URL must point to the upstream authorization server endpoint.
    assert!(
        auth_session
            .auth_url()
            .as_str()
            .starts_with("https://auth.example.com/authorize")
    );

    // Extract the state from the auth URL so we can craft a valid redirect.
    let auth_params: HashMap<String, String> = auth_session
        .auth_url()
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let state = auth_params.get("state").unwrap().clone();

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

    let wscd = MockRemoteWscd::new_with_wia_signing_key(wia_issuer_privkey);
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
    let (issuer, trust_anchor, issuer_identifier, wia_issuer_privkey, tls_trust_anchor) =
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
    let wscd = MockRemoteWscd::new_with_wia_signing_key(wia_issuer_privkey);
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
