use db_test::DbName;
use db_test::DbSetup;

use wallet_provider_domain::model::wallet_flag::WalletFlag::SolutionRevoked;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_persistence::wallet_flag::list_wallet_flags;
use wallet_provider_persistence::wallet_flag::set_wallet_flag;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_flag() {
    let db_setup = DbSetup::create_clean_only([DbName::WalletProvider]).await;
    let db = db_from_setup(&db_setup).await;

    // List flags
    let result = list_wallet_flags(&db).await.expect("should list flags");
    assert_eq!(result, vec![]);

    // Set flag
    set_wallet_flag(&db, SolutionRevoked, true)
        .await
        .expect("should set flag");
    let result = list_wallet_flags(&db).await.expect("should list flags");
    assert_eq!(result, vec![(SolutionRevoked, true)]);

    // Clear flag
    set_wallet_flag(&db, SolutionRevoked, false)
        .await
        .expect("should set flag");
    let result = list_wallet_flags(&db).await.expect("should list flags");
    assert_eq!(result, vec![(SolutionRevoked, false)]);
}
