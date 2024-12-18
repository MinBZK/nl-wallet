use p256::ecdsa::VerifyingKey;
use uuid::Uuid;

use wallet_common::generator::Generator;
use wallet_common::utils::random_string;
use wallet_provider_domain::model::encrypted::Encrypted;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::EpochGenerator;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::entity::wallet_user;
use wallet_provider_persistence::transaction;
use wallet_provider_persistence::wallet_user::clear_instruction_challenge;
use wallet_provider_persistence::wallet_user::commit_pin_change;
use wallet_provider_persistence::wallet_user::register_unsuccessful_pin_entry;
use wallet_provider_persistence::wallet_user::rollback_pin_change;

use crate::common::encrypted_pin_key;

pub mod common;

async fn create_test_user() -> (Db, Uuid, String, wallet_user::Model) {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    common::create_wallet_user_with_random_keys(&db, wallet_user_id, wallet_id.clone()).await;

    let user = common::find_wallet_user(&db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    (db, wallet_user_id, wallet_id, user)
}

#[tokio::test]
async fn test_create_wallet_user() {
    let (_db, _wallet_user_id, wallet_id, wallet_user) = create_test_user().await;
    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[tokio::test]
async fn test_create_wallet_user_transaction_commit() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let transaction = transaction::begin_transaction(&db)
        .await
        .expect("Could not begin transaction");

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id.clone()).await;

    let maybe_wallet_user = common::find_wallet_user(&db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());

    transaction
        .commit()
        .await
        .expect("Could not commit wallet user transaction");

    let wallet_user = common::find_wallet_user(&db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[tokio::test]
async fn test_create_wallet_user_transaction_rollback() {
    let db = common::db_from_env().await.expect("Could not connect to database");
    let wallet_user_id = Uuid::new_v4();
    let wallet_id = random_string(32);

    {
        let transaction = transaction::begin_transaction(&db)
            .await
            .expect("Could not begin transaction");

        common::create_wallet_user_with_random_keys(&transaction, wallet_user_id, wallet_id).await;
    }

    let maybe_wallet_user = common::find_wallet_user(&db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}

#[tokio::test]
async fn test_insert_instruction_challenge_on_conflict() {
    let (db, wallet_user_id, wallet_id, _wallet_user) = create_test_user().await;

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for the first time, we should only find that one afterwards
    common::create_instruction_challenge_with_random_data(&db, wallet_id.clone()).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let og_id = challenges[0].id;
    common::create_instruction_challenge_with_random_data(&db, wallet_id.clone()).await;

    // insert another instruction challenge, this should update the bytes and expiration date in the first one
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // as the challenge should be updated, its ID stays the same
    assert_eq!(challenges[0].id, og_id);

    clear_instruction_challenge(&db, &wallet_id)
        .await
        .expect("Could not clear instruction challenges");

    // after clearing it, we should not find any challenges anymore
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 0);

    common::create_instruction_challenge_with_random_data(&db, wallet_id.clone()).await;

    // insert an instruction challenge for the second time, we should only find that one afterwards
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // this time a new challenge is injected, so the ID should be changed
    assert_ne!(challenges[0].id, og_id);

    // create a second wallet
    let wallet_user_id2 = Uuid::new_v4();
    let wallet_id2 = random_string(32);
    common::create_wallet_user_with_random_keys(&db, wallet_user_id2, wallet_id2.clone()).await;

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id2.clone()).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for our second wallet, we should only find one per wallet
    common::create_instruction_challenge_with_random_data(&db, wallet_id2.clone()).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id2.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id2.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, wallet_id.clone()).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());
}

#[tokio::test]
async fn test_register_unsuccessful_pin_entry() {
    let (db, wallet_user_id, wallet_id, before) = create_test_user().await;
    assert!(before.last_unsuccessful_pin.is_none());

    register_unsuccessful_pin_entry(&db, &wallet_id, false, EpochGenerator.generate())
        .await
        .expect("Could register unsuccessful pin entry");

    let after = common::find_wallet_user(&db, wallet_user_id).await.unwrap();

    assert_eq!(before.pin_entries + 1, after.pin_entries);
    assert_eq!(EpochGenerator.generate(), after.last_unsuccessful_pin.unwrap());
}

async fn do_change_pin() -> (
    Db,
    Uuid,
    String,
    Encrypted<VerifyingKey>,
    wallet_user::Model,
    wallet_user::Model,
) {
    let (db, wallet_user_id, wallet_id, before) = create_test_user().await;

    let new_pin = encrypted_pin_key("new_pin_1").await;

    wallet_provider_persistence::wallet_user::change_pin(&db, &wallet_id, new_pin.clone())
        .await
        .expect("Could register unsuccessful pin entry");

    let after = common::find_wallet_user(&db, wallet_user_id).await.unwrap();

    assert_eq!(
        after.encrypted_previous_pin_pubkey_sec1.clone().unwrap(),
        before.encrypted_pin_pubkey_sec1,
    );
    assert_eq!(after.previous_pin_pubkey_iv.clone().unwrap(), before.pin_pubkey_iv);

    assert_eq!(after.encrypted_pin_pubkey_sec1, new_pin.clone().data);
    assert_eq!(after.pin_pubkey_iv, new_pin.clone().iv.0);

    (db, wallet_user_id, wallet_id, new_pin, before, after)
}

#[tokio::test]
async fn test_change_pin_and_commit() {
    let (db, wallet_user_id, wallet_id, new_pin, _before_pin_change, _after_pin_change) = do_change_pin().await;

    commit_pin_change(&db, wallet_id.as_str()).await.unwrap();

    let after_commit = common::find_wallet_user(&db, wallet_user_id).await.unwrap();

    assert!(after_commit.encrypted_previous_pin_pubkey_sec1.is_none());
    assert!(after_commit.previous_pin_pubkey_iv.is_none());
    assert_eq!(after_commit.encrypted_pin_pubkey_sec1, new_pin.clone().data);
    assert_eq!(after_commit.pin_pubkey_iv, new_pin.iv.0);
}

#[tokio::test]
async fn test_rollback_pin() {
    let (db, wallet_user_id, wallet_id, _new_pin, before_pin_change, _after_pin_change) = do_change_pin().await;

    rollback_pin_change(&db, wallet_id.as_str()).await.unwrap();

    let after_rollback = common::find_wallet_user(&db, wallet_user_id).await.unwrap();

    assert!(after_rollback.encrypted_previous_pin_pubkey_sec1.is_none());
    assert!(after_rollback.previous_pin_pubkey_iv.is_none());
    assert_eq!(
        after_rollback.encrypted_pin_pubkey_sec1,
        before_pin_change.encrypted_pin_pubkey_sec1
    );
    assert_eq!(after_rollback.pin_pubkey_iv, before_pin_change.pin_pubkey_iv);
}
