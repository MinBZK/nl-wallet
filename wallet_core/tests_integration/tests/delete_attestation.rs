use attestation_data::issuable_document::IssuableDocument;
use db_test::DbSetup;
use dcql::CredentialFormat;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use serial_test::serial;
use tests_integration::common::*;
use wallet::AttestationIdentity;
use wallet_provider_persistence::entity::wallet_user;
use wallet_provider_persistence::entity::wallet_user_key;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn test_delete_attestation_ok() {
    let db_setup = DbSetup::create_clean().await;
    let pin = "112233";

    // Use only a single degree document so there is exactly one non-PID attestation to delete.
    let (issuance_settings, _, trust_anchor, tls_config) = issuance_server_settings(db_setup.issuance_server_url());
    let (wp_settings, wp_root_ca) = wallet_provider_settings(db_setup.wallet_provider_url(), db_setup.audit_log_url());

    // Open a connection to the WP database so we can inspect its key table later.
    let wp_db_connection = new_connection(wp_settings.database.url.clone()).await.unwrap();

    let (mut wallet, _, issuance_urls) = setup_wallet_and_env(
        &db_setup,
        WalletDeviceVendor::Apple,
        update_policy_server_settings(),
        (wp_settings, wp_root_ca),
        pid_issuer_settings(db_setup.pid_issuer_url()),
        (
            issuance_settings,
            vec![IssuableDocument::new_mock_degree("MSc".to_string())],
            trust_anchor,
            tls_config,
        ),
    )
    .await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Find the wallet user account in the WP database so we can inspect its keys.
    let wallet_user_id = {
        let wallet_ids = get_all_wallet_ids(&wp_db_connection).await;
        let wallet_id = wallet_ids.first().expect("should have one registered wallet");
        wallet_user::Entity::find()
            .filter(wallet_user::Column::WalletId.eq(wallet_id.as_str()))
            .one(&wp_db_connection)
            .await
            .unwrap()
            .expect("wallet_user record should exist")
            .id
    };

    // Count all private keys stored in the WP for this wallet user now that it only has a PID.
    let keys_before_degree_issuance = wallet_user_key::Entity::find()
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .count(&wp_db_connection)
        .await
        .unwrap();

    // Since we can't delete the PID we have to issue another attestation to the wallet first.
    do_degree_issuance(
        &mut wallet,
        pin.to_owned(),
        &issuance_urls.issuance_server.public,
        CredentialFormat::MsoMdoc,
    )
    .await;

    // Find the degree attestation in the wallet and extract its ID.
    let attestation_id = {
        let attestations = wallet_attestations(&mut wallet).await;
        let degree_attestation = attestations
            .iter()
            .find(|a| a.attestation_type == "com.example.degree")
            .expect("degree attestation should be present after issuance");

        let AttestationIdentity::Fixed { id: attestation_id } = degree_attestation.identity else {
            panic!("degree attestation should have a Fixed identity");
        };

        attestation_id
    };

    // Delete the degree attestation.
    wallet
        .delete_attestation(pin.to_string(), attestation_id.to_string())
        .await
        .expect("delete_attestation should succeed");

    // The degree's private keys should have been removed from the WP database.
    let keys_after_degree_deletion = wallet_user_key::Entity::find()
        .filter(wallet_user_key::Column::WalletUserId.eq(wallet_user_id))
        .count(&wp_db_connection)
        .await
        .unwrap();
    assert_eq!(keys_before_degree_issuance, keys_after_degree_deletion);

    // The degree attestation should no longer be present in the wallet.
    assert!(
        !wallet_attestations(&mut wallet)
            .await
            .iter()
            .any(|a| a.attestation_type == "com.example.degree"),
        "degree attestation should no longer be present in the wallet after deletion"
    );
}
