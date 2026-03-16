use db_test::DbSetup;
use db_test::connection_from_url;
use issuer_common::nonce_store::ProofNonceStore;
use openid4vc::nonce::store::test::test_nonce_store;
use server_utils::store::StoreConnection;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_proof_nonce_store() {
    let db_setup = DbSetup::create().await;
    let store_connection = StoreConnection::Postgres(connection_from_url(db_setup.issuer_common_url()).await);
    let store = ProofNonceStore::new(store_connection);

    test_nonce_store(store).await
}
