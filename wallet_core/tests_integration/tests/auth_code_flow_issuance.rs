use std::collections::HashSet;

use db_test::DbName;
use db_test::DbSetup;
use http_utils::reqwest::default_reqwest_client_builder;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::scope::Scope;
use reqwest::StatusCode;
use reqwest::header;
use reqwest::redirect::Policy;
use serial_test::serial;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use tests_integration::common::*;
use wallet::IssuanceStartResult;

/// The `issuer_state` carried by the auth-code credential offer, identifying the demo usecase.
const ISSUER_STATE: &str = "insurance";

/// Server-side smoke test for the authorization-code-flow demo issuer, without a wallet.
///
/// Boots `acf_demo_issuer` in-process and exercises the Authorization Phase half of the flow:
/// 1. it serves its OpenID4VCI metadata, advertising the configured insurance credential config;
/// 2. a Pushed Authorization Request carrying `issuer_state = "insurance"` is accepted;
/// 3. `/authorize` resolves the PAR and redirects the user-agent to the consent page for that usecase.
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
    let par_request = VciAuthorizationRequest::for_auth_code(
        NL_WALLET_CLIENT_ID.to_string(),
        wallet_issuance_redirect_uri(),
        "wallet-state".to_string(),
        Some(ISSUER_STATE.to_string()),
        HashSet::from(["com.example.insurance".parse::<Scope>().unwrap()]),
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

/// Full wallet driven issuance for the authorization-code flow demo issuer.
///
/// Registers a wallet, then issues the `com.example.insurance` attestation end-to-end from the static
/// auth-code credential offer: the wallet pushes the PAR and yields the authorize URL,
/// [`fake_consent_auth`] drives the consent page, and the wallet exchanges the resulting code for
/// previews and accepts issuance. The PKCE round-trip (challenge at PAR, verifier at `/token`) and the
/// `issuer_state` -> usecase selection are exercised for real.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_acf_demo_issuer_wallet_issuance() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let wallet = setup_wallet_env(&db_setup, WalletDeviceVendor::Apple).await;
    let acf = setup_auth_code_env(&db_setup).await;
    let mut wallet = do_wallet_registration(wallet, pin).await;

    // The demo issuer's QR: a static auth-code offer selecting the insurance usecase via issuer_state.
    let offer_uri = create_auth_code_credential_offer(&acf.public, ISSUER_STATE);

    // Starting issuance pushes the PAR and returns the authorize URL for the user-agent to open.
    let IssuanceStartResult::AuthorizationUrl(authorization_url) = wallet
        .start_issuance_from_offer(offer_uri)
        .await
        .expect("should start issuance from offer")
    else {
        panic!("expected an authorization URL for the auth-code flow");
    };

    // Drive the consent page, then hand the wallet the resulting redirect to exchange for previews.
    let redirect_uri = fake_consent_auth(authorization_url).await;
    let previews = wallet
        .continue_issuance(redirect_uri)
        .await
        .expect("should continue issuance after consent");

    assert_eq!(previews.len(), 1);
    assert_eq!(previews.first().unwrap().attestation_type, "com.example.insurance");

    wallet
        .accept_issuance(pin.to_owned())
        .await
        .expect("should accept issuance");

    let attestations = wallet_attestations(&mut wallet).await;
    assert!(
        attestations
            .iter()
            .any(|attestation| attestation.attestation_type == "com.example.insurance"),
        "the issued insurance attestation should be present in the wallet",
    );
}
