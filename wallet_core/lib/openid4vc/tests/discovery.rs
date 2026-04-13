use std::collections::HashMap;
use std::num::NonZeroU8;

use indexmap::IndexMap;
use rustls_pki_types::TrustAnchor;
use serde_json::json;
use url::Url;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::x509::generate::mock::generate_pid_issuer_mock_with_registration;
use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
use crypto::server_keys::generate::Ca;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::default_reqwest_client_builder;
use openid4vc::Format;
use openid4vc::credential::CredentialOffer;
use openid4vc::credential::CredentialOfferContainer;
use openid4vc::credential::GrantPreAuthorizedCode;
use openid4vc::credential::Grants;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::preview::CredentialPreviewResponse;
use openid4vc::token::CredentialPreview;
use openid4vc::token::CredentialPreviewContent;
use openid4vc::token::TokenResponse;
use openid4vc::token::TokenType;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;

/// Starts a wiremock server that serves the well-known metadata endpoints, a token endpoint,
/// and a credential preview endpoint. Returns the server, issuer identifier, and trust anchor.
async fn start_wiremock_issuer(
    authorization_endpoint: Option<&str>,
) -> (MockServer, IssuerIdentifier, TrustAnchor<'static>) {
    let server = MockServer::start().await;
    let server_url = server.uri();
    let issuer_identifier: IssuerIdentifier = format!("{server_url}/").parse().unwrap();

    // Create CA and issuer certificate for the credential preview.
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuance_keypair = generate_pid_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
    let trust_anchor = ca.to_trust_anchor().to_owned();

    // Create type metadata for the credential preview.
    let (_, _, type_metadata_documents) =
        TypeMetadataDocuments::from_single_example(TypeMetadata::example_with_claim_name(
            PID_ATTESTATION_TYPE,
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        ));

    let credential_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

    let preview = CredentialPreview {
        content: CredentialPreviewContent {
            copies_per_format: IndexMap::from([(Format::MsoMdoc, NonZeroU8::new(4).unwrap())]),
            credential_payload,
            issuer_certificate: issuance_keypair.certificate().clone(),
        },
        type_metadata: type_metadata_documents,
    };

    let preview_response = CredentialPreviewResponse {
        credential_previews: vec_nonempty![preview],
    };

    let token_response = TokenResponse {
        access_token: "mock_access_token".to_string().into(),
        token_type: TokenType::DPoP,
        refresh_token: None,
        scope: None,
        expires_in: None,
        authorization_details: None,
    };

    // Construct issuer metadata JSON.
    let issuer_metadata_json = json!({
        "credential_issuer": issuer_identifier.to_string(),
        "credential_endpoint": format!("{server_url}/issuance/credential"),
        "batch_credential_endpoint": format!("{server_url}/issuance/batch_credential"),
        "nonce_endpoint": format!("{server_url}/issuance/nonce"),
        "credential_preview_endpoint": format!("{server_url}/issuance/credential_preview"),
        "credential_configurations_supported": {
            PID_ATTESTATION_TYPE: {
                "format": "mso_mdoc",
                "doctype": PID_ATTESTATION_TYPE,
                "proof_types_supported": {
                    "jwt": { "proof_signing_alg_values_supported": ["ES256"] }
                },
            }
        },
    });

    // Construct OAuth metadata JSON.
    let mut oauth_metadata_json = json!({
        "issuer": issuer_identifier.to_string(),
        "token_endpoint": format!("{server_url}/issuance/token"),
        "response_types_supported": ["code"],
        "subject_types_supported": [],
        "id_token_signing_alg_values_supported": [],
    });
    if let Some(auth_endpoint) = authorization_endpoint {
        oauth_metadata_json["authorization_endpoint"] = json!(auth_endpoint);
    }

    Mock::given(method("GET"))
        .and(path("/.well-known/openid-credential-issuer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issuer_metadata_json))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/.well-known/oauth-authorization-server"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&oauth_metadata_json))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/issuance/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&token_response)
                .insert_header("DPoP-Nonce", "mock_dpop_nonce"),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/issuance/credential_preview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&preview_response))
        .mount(&server)
        .await;

    (server, issuer_identifier, trust_anchor)
}

#[tokio::test]
async fn pre_authorized_code_flow() {
    let (_server, issuer_identifier, trust_anchor) = start_wiremock_issuer(None).await;

    // Construct a credential offer URL with a fake pre-authorized code.
    let credential_offer = CredentialOffer {
        credential_issuer: issuer_identifier,
        credential_configuration_ids: vec![PID_ATTESTATION_TYPE.to_string()],
        grants: Some(Grants::PreAuthorizedCode {
            pre_authorized_code: GrantPreAuthorizedCode::new("fake_pre_auth_code".to_string().into()),
        }),
    };
    let container = CredentialOfferContainer { credential_offer };
    let query = serde_urlencoded::to_string(&container).unwrap();
    let offer_url: Url = format!("openid-credential-offer://?{query}").parse().unwrap();

    let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap());

    let session = discovery
        .start_pre_authorized_code_flow(&offer_url, MOCK_WALLET_CLIENT_ID.to_string(), &[trust_anchor])
        .await
        .unwrap();

    assert_eq!(session.normalized_credential_preview().len(), 1);
    assert_eq!(
        session.normalized_credential_preview()[0]
            .content
            .credential_payload
            .attestation_type,
        PID_ATTESTATION_TYPE
    );
}

#[tokio::test]
async fn authorization_code_flow() {
    let authorization_endpoint = "https://auth.example.com/authorize";
    let (_server, issuer_identifier, trust_anchor) = start_wiremock_issuer(Some(authorization_endpoint)).await;

    let redirect_uri: Url = "https://wallet.example.com/callback".parse().unwrap();

    let discovery = HttpIssuanceDiscovery::new(HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap());

    // Start the authorization code flow — fetches metadata and creates an auth session.
    let auth_session = discovery
        .start_authorization_code_flow(
            &issuer_identifier,
            MOCK_WALLET_CLIENT_ID.to_string(),
            redirect_uri.clone(),
        )
        .await
        .unwrap();

    // Verify the auth URL points to the expected authorization endpoint.
    assert!(auth_session.auth_url().as_str().starts_with(authorization_endpoint));

    // Extract the state from the auth URL to simulate the redirect.
    let auth_params: HashMap<String, String> = auth_session
        .auth_url()
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let state = auth_params.get("state").unwrap();

    // Simulate the authorization server redirecting back with a code and state.
    let mut received_redirect_uri = redirect_uri;
    received_redirect_uri.set_query(Some(&format!("code=fake_auth_code&state={state}")));

    // Complete the flow — exchanges the code for a token and fetches credential previews.
    let session = auth_session
        .start_issuance(&received_redirect_uri, &[trust_anchor])
        .await
        .unwrap();

    assert_eq!(session.normalized_credential_preview().len(), 1);
    assert_eq!(
        session.normalized_credential_preview()[0]
            .content
            .credential_payload
            .attestation_type,
        PID_ATTESTATION_TYPE
    );
}
