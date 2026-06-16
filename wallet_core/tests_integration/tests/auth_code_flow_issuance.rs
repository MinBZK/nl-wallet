use db_test::DbName;
use db_test::DbSetup;
use http_utils::reqwest::default_reqwest_client_builder;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use reqwest::StatusCode;
use reqwest::header;
use reqwest::redirect::Policy;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use tests_integration::common::*;
use url::Url;

/// The `issuer_state` carried by the auth-code credential offer, identifying the demo usecase.
const ISSUER_STATE: &str = "insurance";

/// Preliminary integration test for the authorization-code-flow demo issuer.
///
/// Boots `acf_demo_issuer` in-process and exercises the Authorization Phase half of the flow:
/// 1. it serves its OpenID4VCI metadata, advertising the configured insurance credential config;
/// 2. a Pushed Authorization Request carrying `issuer_state = "insurance"` is accepted;
/// 3. `/authorize` resolves the PAR and redirects the user-agent to the consent page for that usecase.
///
/// TODO: test full flow including the consent page
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_acf_demo_issuer_authorize_redirects_to_consent() {
    let db_setup = DbSetup::create_clean_only([DbName::AcfDemoIssuer]).await;

    let acf = setup_auth_code_env(&db_setup).await;

    let client = default_reqwest_client_builder().build().unwrap();

    // 1. Boot smoke: the issuer serves its OpenID4VCI metadata, advertising the insurance config.
    let metadata = client
        .get(acf.public.as_base_url().join(".well-known/openid-credential-issuer"))
        .send()
        .await
        .unwrap();
    assert!(metadata.status().is_success(), "issuer metadata should be served");
    assert!(
        metadata.text().await.unwrap().contains("com.example.insurance"),
        "metadata should advertise the insurance credential configuration"
    );

    // 2. Push an authorization request carrying issuer_state = "insurance".
    let redirect_uri: Url = ACF_WALLET_REDIRECT_URI.parse().unwrap();
    let par_request = VciAuthorizationRequest::for_auth_code(
        NL_WALLET_CLIENT_ID.to_string(),
        redirect_uri,
        "wallet-state".to_string(),
        Some(ISSUER_STATE.to_string()),
        &S256PkcePair::generate(),
    );

    let par_response = client
        .post(acf.public.as_base_url().join("issuance/par"))
        .form(&par_request)
        .send()
        .await
        .unwrap();
    assert_eq!(par_response.status(), StatusCode::CREATED, "PAR should be accepted");
    let par_response: PushedAuthorizationResponse = par_response.json().await.unwrap();

    // 3. /authorize resolves the PAR and redirects the user-agent to the consent page for the usecase.
    let no_redirect_client = default_reqwest_client_builder()
        .redirect(Policy::none())
        .build()
        .unwrap();
    let authorize_response = no_redirect_client
        .get(acf.public.as_base_url().join("issuance/authorize"))
        .query(&[
            ("request_uri", par_response.request_uri.as_str()),
            ("client_id", NL_WALLET_CLIENT_ID),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(
        authorize_response.status(),
        StatusCode::FOUND,
        "authorize should redirect to the consent page"
    );
    let location = authorize_response
        .headers()
        .get(header::LOCATION)
        .expect("authorize response should carry a Location header")
        .to_str()
        .unwrap();

    let consent_url = acf.public.as_base_url().join("consent");
    assert!(
        location.starts_with(consent_url.as_str()),
        "should redirect to the consent page, got: {location}"
    );
    assert!(
        location.contains(&format!("usecase={ISSUER_STATE}")),
        "consent redirect should carry the usecase, got: {location}"
    );
}
