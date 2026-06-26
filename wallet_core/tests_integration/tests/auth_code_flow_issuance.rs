use std::collections::HashSet;

use db_test::DbName;
use db_test::DbSetup;
use http_utils::reqwest::default_reqwest_client_builder;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::credential_offer::CredentialOffer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::scope::Scope;
use reqwest::StatusCode;
use reqwest::header;
use reqwest::redirect::Policy;
use serial_test::serial;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use tests_integration::common::*;
use url::Url;
use utils::vec_at_least::VecNonEmptyUnique;
use utils::vec_nonempty;
use wallet::IssuanceStartResult;
use wallet::Pin;

/// The `issuer_state` carried by the auth-code credential offer, identifying the demo usecase.
const ISSUER_STATE: &str = "insurance_acf";

/// Server-side smoke test for the authorization-code-flow demo issuer, without a wallet.
///
/// Boots `acf_demo_issuer` in-process and exercises the Authorization Phase half of the flow:
/// 1. it serves its OpenID4VCI metadata, advertising the configured insurance credential config;
/// 2. a Pushed Authorization Request carrying `issuer_state = "insurance_acf"` is accepted;
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

    // 2. Push an authorization request carrying issuer_state = "insurance_acf".
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
    let pin: Pin = "112233".into();

    let wallet = setup_wallet_env(&db_setup, WalletDeviceVendor::Apple).await;
    let acf = setup_auth_code_env(&db_setup).await;
    let mut wallet = do_wallet_registration(wallet, pin.clone()).await;

    // The demo issuer's QR: a static auth-code offer selecting the insurance usecase via issuer_state.
    let offer_uri = create_auth_code_credential_offer(&acf.public, ISSUER_STATE, "com.example.insurance");

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

    wallet.accept_issuance(pin).await.expect("should accept issuance");

    let attestations = wallet_attestations(&mut wallet).await;
    assert!(
        attestations
            .iter()
            .any(|attestation| attestation.attestation_type == "com.example.insurance"),
        "the issued insurance attestation should be present in the wallet",
    );
}

/// Build the static authorization-code credential offer the demo issuer's QR encodes: a by-value
/// `openid-credential-offer://` URL carrying the configured credential configuration and the
/// `issuer_state` that selects the usecase. Mirrors `demo_issuer`'s `authorization_code_usecase`.
pub fn create_auth_code_credential_offer(
    acf_demo_issuer_url: &IssuerIdentifier,
    issuer_state: &str,
    credential_configuration_id: &str,
) -> Url {
    let credential_configuration_ids: VecNonEmptyUnique<CredentialConfigurationId> =
        vec_nonempty![CredentialConfigurationId::from(credential_configuration_id.to_string())].into();

    let offer = CredentialOffer::new_authorization(
        acf_demo_issuer_url.clone(),
        credential_configuration_ids,
        Some(issuer_state.to_string()),
    );

    CredentialOfferContainer::new_offer(offer).to_credential_offer_url()
}

/// Drive the acf demo issuer's consent flow the way a browser would: follow the wallet's authorize URL
/// to the consent page, submit consent, and return the wallet-facing redirect (carrying the
/// issuer-minted code + the wallet's `state`) that [`Wallet::continue_issuance`] expects. The acf
/// analogue of [`fake_digid_auth`](crate::fake_digid::fake_digid_auth), but without the upstream
/// SAML/DigiD hops.
pub async fn fake_consent_auth(authorization_url: Url) -> Url {
    let client = default_reqwest_client_builder()
        .redirect(Policy::none())
        .build()
        .unwrap();

    // The wallet's authorize URL (PAR already pushed) redirects to the issuer's consent page.
    let authorize_response = client.get(authorization_url).send().await.unwrap();
    let consent_url: Url = authorize_response
        .headers()
        .get(header::LOCATION)
        .expect("authorize should redirect to the consent page")
        .to_str()
        .unwrap()
        .parse()
        .expect("failed to parse consent page url");

    // Submit consent. The handler generates the authorization code, writes the `AuthCodeIssued` session and
    // redirects back to the wallet's redirect_uri with the code and echoed state.
    let consent_response = client.post(consent_url).send().await.unwrap();
    consent_response
        .headers()
        .get(header::LOCATION)
        .expect("consent submission should redirect back to the wallet")
        .to_str()
        .unwrap()
        .parse()
        .expect("failed to parse wallet redirect url")
}
