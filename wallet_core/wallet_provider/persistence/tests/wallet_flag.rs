use db_test::DbName;
use db_test::DbSetup;

use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::model::wallet_flag::WalletFlag::SolutionRevoked;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_persistence::wallet_flag::get_wallet_flag;
use wallet_provider_persistence::wallet_flag::list_wallet_flags;
use wallet_provider_persistence::wallet_flag::set_wallet_flag;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_flag() {
    let db_setup = DbSetup::create_clean_only([DbName::WalletProvider]).await;
    let db = db_from_setup(&db_setup).await;

    // Empty
    assert_list_flag(&db, []).await;
    assert_get_flag(&db, SolutionRevoked, false).await;

    // Set flag
    set_wallet_flag(&db, SolutionRevoked, true)
        .await
        .expect("should set flag");
    assert_list_flag(&db, [(SolutionRevoked, true)]).await;
    assert_get_flag(&db, SolutionRevoked, true).await;

    // Clear flag
    set_wallet_flag(&db, SolutionRevoked, false)
        .await
        .expect("should set flag");
    assert_list_flag(&db, [(SolutionRevoked, false)]).await;
    assert_get_flag(&db, SolutionRevoked, false).await;
}

async fn assert_list_flag(db: &Db, expected: impl IntoIterator<Item = (WalletFlag, bool)>) {
    let result = list_wallet_flags(db).await.expect("should list flags");
    assert_eq!(result, expected.into_iter().collect::<Vec<_>>());
}

async fn assert_get_flag(db: &Db, flag: WalletFlag, expected: bool) {
    let result = get_wallet_flag(db, flag).await.expect("should get flag");
    assert_eq!(result, expected);
}
