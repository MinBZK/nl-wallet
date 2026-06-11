use db_test::DbSetup;
use openid4vc::credential_offer::CredentialOffer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::issuable_document::IssuableDocument;
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_nonempty;
use wallet::IssuanceStartResult;

const DEGREE_CREDENTIAL_CONFIG_ID: &str = "com.example.degree_sd_jwt";

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_pre_authorized_code_issuance() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, issuance_data) = setup_wallet_and_default_env(&db_setup, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;

    // Create a pre-authorized issuance session on the issuance server.
    let documents = vec_nonempty![IssuableDocument::new_mock_degree("BSc".to_string())];
    let session_token = issuance_data
        .issuance_server_issuer
        .new_session(documents)
        .await
        .expect("should create pre-authorized session");

    // Build the URL including the credential offer.
    let config_ids = vec_nonempty![DEGREE_CREDENTIAL_CONFIG_ID.to_string().into()];
    let credential_offer = CredentialOffer::new_pre_authorized(
        issuance_data.issuance_server.public.clone(),
        config_ids,
        session_token.into(),
    );
    let offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();

    // The wallet should resolve the offer and return credential previews immediately.
    let IssuanceStartResult::Previews(previews) = wallet
        .start_issuance_from_offer(offer_url)
        .await
        .expect("should start issuance from offer")
    else {
        panic!("expected Previews, got authorization redirect");
    };

    assert_eq!(previews.len(), 1);

    // Accept the issuance with the wallet PIN.
    wallet
        .accept_issuance(pin.to_owned())
        .await
        .expect("should accept issuance");

    // Check that every preview attestation is present in the wallet database after issuance.
    let attestations = wallet_attestations(&mut wallet).await;

    for preview in &previews {
        assert!(
            attestations
                .iter()
                .any(|a| a.attestation_type == preview.attestation_type && a.attributes == preview.attributes),
            "attestation for '{}' not found in wallet",
            preview.attestation_type,
        );
    }
}
