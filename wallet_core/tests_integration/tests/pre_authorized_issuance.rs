use db_test::DbSetup;
use http_utils::reqwest::default_reqwest_client_builder;
use openid4vc::issuable_document::IssuableDocument;
use pacf_issuance_server::offer::OfferRequest;
use pacf_issuance_server::offer::OfferResponse;
use serial_test::serial;
use tests_integration::common::*;
use utils::vec_nonempty;
use wallet::IssuanceStartResult;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_pre_authorized_code_issuance() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    let (mut wallet, _, issuance_data) = setup_wallet_and_default_env(&db_setup, WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;

    // Create a pre-authorized issuance session on the issuance server.
    let documents = vec_nonempty![IssuableDocument::new_mock_loyalty()];

    let offer_response = default_reqwest_client_builder()
        .build()
        .unwrap()
        .post(issuance_data.pacf_issuance_server.internal.join("offer"))
        .json(&OfferRequest { documents })
        .send()
        .await
        .unwrap()
        .json::<OfferResponse>()
        .await
        .unwrap();

    // The wallet should resolve the offer and return credential previews immediately.
    let IssuanceStartResult::Previews(previews) = wallet
        .start_issuance_from_offer(offer_response.credential_offer_url)
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

    let a = attestations.first().unwrap();
    assert_eq!(a.attestation_type, "com.example.jum.bonuskaart".to_string());
}
