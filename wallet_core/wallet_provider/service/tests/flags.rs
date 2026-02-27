use std::time::Duration;

use db_test::DbSetup;
use status_lists::postgres::RevokeAll;
use wallet_provider_domain::model::wallet_flag::WalletFlag::SolutionRevoked;
use wallet_provider_domain::repository::WalletFlagRepository;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::clear_flags_dropper;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_service::flags::WalletFlags;
use wallet_provider_service::flags::WalletRepoFlags;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_flags_on_startup() {
    let db_setup = DbSetup::create().await;
    let db: Db = db_from_setup(&db_setup).await;
    let _clear_flags = clear_flags_dropper(&db_setup);

    let repo: Repositories = db.into();
    repo.set_flag(SolutionRevoked).await.expect("could not set flag");

    // Should fetch flags directly
    let flags = WalletRepoFlags::try_new(repo, Duration::from_secs(100)).await.unwrap();
    assert!(flags.solution_is_revoked());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_flags_when_set() {
    let db_setup = DbSetup::create().await;
    let db: Db = db_from_setup(&db_setup).await;
    let _clear_flags = clear_flags_dropper(&db_setup);

    let repo: Repositories = db.into();
    let flags = WalletRepoFlags::try_new(repo, Duration::from_secs(100)).await.unwrap();
    assert!(!flags.solution_is_revoked());

    // Set flags
    flags.set_solution_revoked().await.expect("could not solution revoked");
    assert!(flags.solution_is_revoked());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_flags_on_refresh() {
    let db_setup = DbSetup::create().await;
    let db: Db = db_from_setup(&db_setup).await;
    let _clear_flags = clear_flags_dropper(&db_setup);

    let repo: Repositories = db.into();
    let flags = WalletRepoFlags::try_new(repo.clone(), Duration::from_millis(100))
        .await
        .unwrap();
    assert!(!flags.solution_is_revoked());
    assert!(!flags.is_revoked_all().await.expect("could not get is revoked all"));

    // Using repo, since set_solution_revoked will refresh automatically
    repo.set_flag(SolutionRevoked).await.expect("could not set flag");
    assert!(flags.is_revoked_all().await.expect("could not get is revoked all"));

    let refresh_handle = flags.start_refresh_job();
    let result = tokio::time::timeout(Duration::from_secs(1), async {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            if flags.solution_is_revoked() {
                break;
            }
        }
    })
    .await;
    refresh_handle.abort();

    result.expect("solution not revoked");
}
