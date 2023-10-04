use serial_test::serial;
use uuid::Uuid;

use wallet_common::{generator::Generator, utils::random_string};
use wallet_provider_domain::{repository::Committable, EpochGenerator};
use wallet_provider_persistence::{
    transaction,
    wallet_user::{clear_instruction_challenge, register_unsuccessful_pin_entry},
};

pub mod common;

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    common::create_wallet_user_with_random_keys(db, wallet_user_id, wallet_id.clone()).await;

    let wallet_user = common::find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_commit() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let transaction = transaction::begin_transaction(db)
        .await
        .expect("Could not begin transaction");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id.clone()).await;

    let maybe_wallet_user = common::find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());

    transaction
        .commit()
        .await
        .expect("Could not commit wallet user transaction");

    let wallet_user = common::find_wallet_user(db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_create_wallet_user_transaction_rollback() {
    let db = common::db_from_env().await.expect("Could not connect to database");
    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    {
        let transaction = transaction::begin_transaction(db)
            .await
            .expect("Could not begin transaction");

        common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id).await;
    }

    let maybe_wallet_user = common::find_wallet_user(db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_insert_instruction_challenge_on_conflict() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);
    common::create_wallet_user_with_random_keys(db, wallet_user_id, wallet_id.clone()).await;

    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for the first time, we should only find that one afterwards
    common::create_instruction_challenge_with_random_data(db, wallet_id.clone()).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let og_id = challenges[0].id;
    common::create_instruction_challenge_with_random_data(db, wallet_id.clone()).await;

    // insert another instruction challenge, this should update the bytes and expiration date in the first one
    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // as the challenge should be updated, its ID stays the same
    assert_eq!(challenges[0].id, og_id);

    clear_instruction_challenge(db, &wallet_id)
        .await
        .expect("Could not clear instruction challenges");

    // after clearing it, we should not find any challenges anymore
    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 0);

    common::create_instruction_challenge_with_random_data(db, wallet_id.clone()).await;

    // insert an instruction challenge for the second time, we should only find that one afterwards
    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // this time a new challenge is injected, so the ID should be changed
    assert_ne!(challenges[0].id, og_id);

    // create a second wallet
    let wallet_user_id2 = Uuid::new_v4();
    let wallet_id2 = random_string(32);
    common::create_wallet_user_with_random_keys(db, wallet_user_id2, wallet_id2.clone()).await;

    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id2.clone()).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for our second wallet, we should only find one per wallet
    common::create_instruction_challenge_with_random_data(db, wallet_id2.clone()).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id2.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id2.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());
}

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
#[serial]
async fn test_register_unsuccessful_pin_entry() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    common::create_wallet_user_with_random_keys(db, wallet_user_id, wallet_id.clone()).await;

    let before = common::find_wallet_user(db, wallet_user_id).await.unwrap();
    assert!(before.last_unsuccessful_pin.is_none());

    register_unsuccessful_pin_entry(db, &wallet_id, false, EpochGenerator.generate())
        .await
        .expect("Could register unsuccessful pin entry");

    let after = common::find_wallet_user(db, wallet_user_id).await.unwrap();

    assert_eq!(before.pin_entries + 1, after.pin_entries);
    assert_eq!(EpochGenerator.generate(), after.last_unsuccessful_pin.unwrap());
}
