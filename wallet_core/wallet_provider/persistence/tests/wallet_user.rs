use assert_matches::assert_matches;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::EncodePublicKey;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use crypto::utils::random_string;
use hsm::model::encrypted::Encrypted;
use utils::generator::Generator;
use wallet_provider_domain::EpochGenerator;
use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::entity::wallet_user;
use wallet_provider_persistence::transaction;
use wallet_provider_persistence::wallet_user::clear_instruction_challenge;
use wallet_provider_persistence::wallet_user::commit_pin_change;
use wallet_provider_persistence::wallet_user::find_wallet_user_by_wallet_id;
use wallet_provider_persistence::wallet_user::has_multiple_active_accounts_by_recovery_code;
use wallet_provider_persistence::wallet_user::register_unsuccessful_pin_entry;
use wallet_provider_persistence::wallet_user::reset_wallet_user_state;
use wallet_provider_persistence::wallet_user::rollback_pin_change;
use wallet_provider_persistence::wallet_user::store_recovery_code;
use wallet_provider_persistence::wallet_user::transition_wallet_user_state;
use wallet_provider_persistence::wallet_user::update_apple_assertion_counter;

use crate::common::encrypted_pin_key;

pub mod common;

#[tokio::test]
async fn test_create_wallet_user() {
    let (_db, _wallet_user_id, wallet_id, wallet_user) = common::create_test_user().await;
    assert_eq!(wallet_id, wallet_user.wallet_id);
}

#[tokio::test]
async fn test_find_wallet_user_by_wallet_id() {
    let (db, wallet_user_id, wallet_id, wallet_user_model) = common::create_test_user().await;

    // A wallet user that does not have an instruction challenge associated with it should be found.
    let wallet_user_result = find_wallet_user_by_wallet_id(&db, &wallet_id)
        .await
        .expect("finding wallet user by wallet id should succeed");

    assert_matches!(wallet_user_result, WalletUserQueryResult::Found(_));

    let WalletUserQueryResult::Found(wallet_user) = wallet_user_result else {
        panic!();
    };

    assert_eq!(wallet_user.id, wallet_user_id);
    assert_eq!(wallet_user.wallet_id, wallet_id);
    assert_eq!(
        wallet_user.hw_pubkey.to_public_key_der().unwrap().as_bytes(),
        &wallet_user_model.hw_pubkey_der
    );
    assert!(wallet_user.instruction_challenge.is_none());
    assert_matches!(wallet_user.attestation, WalletUserAttestation::Apple { .. });

    // After generating a random instruction challenge, this should be found together with the user.
    common::create_instruction_challenge_with_random_data(&db, &wallet_id).await;

    let wallet_user_result = find_wallet_user_by_wallet_id(&db, &wallet_id)
        .await
        .expect("finding wallet user by wallet id should succeed");

    assert_matches!(wallet_user_result, WalletUserQueryResult::Found(_));

    let WalletUserQueryResult::Found(wallet_user) = wallet_user_result else {
        panic!();
    };

    assert!(wallet_user.instruction_challenge.is_some());
}

#[tokio::test]
async fn test_create_wallet_user_transaction_commit() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let transaction = transaction::begin_transaction(&db)
        .await
        .expect("Could not begin transaction");

    let wallet_id = random_string(32);

    let wallet_user_id = common::create_wallet_user_with_random_keys(&transaction, wallet_id.clone()).await;

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
    let wallet_id = random_string(32);

    let wallet_user_id = {
        let transaction = transaction::begin_transaction(&db)
            .await
            .expect("Could not begin transaction");

        common::create_wallet_user_with_random_keys(&transaction, wallet_id).await
    };

    let maybe_wallet_user = common::find_wallet_user(&db, wallet_user_id).await;

    assert!(maybe_wallet_user.is_none());
}

#[tokio::test]
async fn test_insert_instruction_challenge_on_conflict() {
    let (db, wallet_user_id, wallet_id, _wallet_user) = common::create_test_user().await;

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for the first time, we should only find that one afterwards
    common::create_instruction_challenge_with_random_data(&db, &wallet_id).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let og_id = challenges[0].id;
    common::create_instruction_challenge_with_random_data(&db, &wallet_id).await;

    // insert another instruction challenge, this should update the bytes and expiration date in the first one
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // as the challenge should be updated, its ID stays the same
    assert_eq!(challenges[0].id, og_id);

    clear_instruction_challenge(&db, &wallet_id)
        .await
        .expect("Could not clear instruction challenges");

    // after clearing it, we should not find any challenges anymore
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 0);

    common::create_instruction_challenge_with_random_data(&db, &wallet_id).await;

    // insert an instruction challenge for the second time, we should only find that one afterwards
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id);
    // this time a new challenge is injected, so the ID should be changed
    assert_ne!(challenges[0].id, og_id);

    // create a second wallet
    let wallet_id2 = random_string(32);
    let wallet_user_id2 = common::create_wallet_user_with_random_keys(&db, wallet_id2.clone()).await;

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id2).await;
    assert!(challenges.is_empty());

    // insert an instruction challenge for our second wallet, we should only find one per wallet
    common::create_instruction_challenge_with_random_data(&db, &wallet_id2).await;
    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id2).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id2.clone());

    let challenges = common::find_instruction_challenges_by_wallet_id(&db, &wallet_id).await;
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].wallet_user_id, wallet_user_id.clone());
}

#[tokio::test]
async fn test_register_unsuccessful_pin_entry() {
    let (db, wallet_user_id, wallet_id, before) = common::create_test_user().await;
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
    let (db, wallet_user_id, wallet_id, before) = common::create_test_user().await;

    let new_pin = encrypted_pin_key("new_pin_1").await;

    wallet_provider_persistence::wallet_user::change_pin(&db, &wallet_id, new_pin.clone(), WalletUserState::Active)
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

#[tokio::test]
async fn test_update_apple_assertion_counter() {
    let (db, _wallet_user_id, wallet_id, _wallet_user_model) = common::create_test_user().await;
    let (_, _, other_wallet_id, _) = common::create_test_user().await;

    // Each of the two users created above should start out with an assertion counter of 0.
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap();
    let other_wallet_user = find_wallet_user_by_wallet_id(&db, &other_wallet_id).await.unwrap();

    match (wallet_user, other_wallet_user) {
        (WalletUserQueryResult::Found(wallet_user), WalletUserQueryResult::Found(other_wallet_user)) => {
            assert_matches!(
                wallet_user.attestation,
                WalletUserAttestation::Apple { assertion_counter } if *assertion_counter == 0
            );
            assert_matches!(
                other_wallet_user.attestation,
                WalletUserAttestation::Apple { assertion_counter } if *assertion_counter == 0
            );
        }
        _ => panic!(),
    };

    // After updating the assertion counter for the first user it
    // should be changed, while the other one should remain at 0.
    update_apple_assertion_counter(&db, &wallet_id, AssertionCounter::from(1337))
        .await
        .expect("updating apple assertion could succeed");

    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap();
    let other_wallet_user = find_wallet_user_by_wallet_id(&db, &other_wallet_id).await.unwrap();

    match (wallet_user, other_wallet_user) {
        (WalletUserQueryResult::Found(wallet_user), WalletUserQueryResult::Found(other_wallet_user)) => {
            assert_matches!(
                wallet_user.attestation,
                WalletUserAttestation::Apple { assertion_counter } if *assertion_counter == 1337
            );
            assert_matches!(
                other_wallet_user.attestation,
                WalletUserAttestation::Apple { assertion_counter } if *assertion_counter == 0
            );
        }
        _ => panic!(),
    };
}

#[tokio::test]
async fn test_transition_wallet_user_state() {
    let (db, wallet_user_id, wallet_id, user) = common::create_test_user().await;

    assert_eq!(user.state, WalletUserState::Active.to_string());

    transition_wallet_user_state(
        &db,
        wallet_user_id,
        WalletUserState::Active,
        WalletUserState::RecoveringPin,
    )
    .await
    .unwrap();

    let WalletUserQueryResult::Found(user) = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap() else {
        panic!("Could not find wallet user");
    };

    assert_eq!(user.state, WalletUserState::RecoveringPin);

    // Does not transition state when coming from different state
    let err = transition_wallet_user_state(
        &db,
        wallet_user_id,
        WalletUserState::Transferred,
        WalletUserState::Active,
    )
    .await
    .expect_err("Could not transition state");
    assert_matches!(err, PersistenceError::NoRowsUpdated);
}

#[tokio::test]
async fn test_reset_wallet_user_state() {
    let (db, wallet_user_id, wallet_id, user) = common::create_test_user().await;

    assert_eq!(user.state, WalletUserState::Active.to_string());

    transition_wallet_user_state(
        &db,
        wallet_user_id,
        WalletUserState::Active,
        WalletUserState::Transferring,
    )
    .await
    .unwrap();

    reset_wallet_user_state(&db, wallet_user_id).await.unwrap();

    let WalletUserQueryResult::Found(user) = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap() else {
        panic!("Could not find wallet user");
    };

    assert_eq!(user.state, WalletUserState::Active);

    // Resetting should be idempotent
    reset_wallet_user_state(&db, wallet_user_id).await.unwrap();

    let WalletUserQueryResult::Found(user) = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap() else {
        panic!("Could not find wallet user");
    };

    assert_eq!(user.state, WalletUserState::Active);
}

#[tokio::test]
async fn test_store_recovery_code() {
    let (db, _, wallet_id, _) = common::create_test_user().await;
    let (_, _, other_wallet_id, _) = common::create_test_user().await;

    // Each of the two users created above should start out with a recovery code that is null.
    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap();
    let other_wallet_user = find_wallet_user_by_wallet_id(&db, &other_wallet_id).await.unwrap();

    assert_matches!(
        wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.is_none()
    );
    assert_matches!(
        other_wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.is_none()
    );

    let recovery_code = "885ed8a2-f07a-4f77-a8df-2e166f5ebd36".to_string();
    // After updating the recovery_code for the first user it should be changed, while the other one should remain null
    store_recovery_code(&db, &other_wallet_id, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");

    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap();
    let other_wallet_user = find_wallet_user_by_wallet_id(&db, &other_wallet_id).await.unwrap();

    assert_matches!(
        wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.is_none()
    );
    assert_matches!(
        other_wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.as_ref().is_some_and(|rc| rc == &recovery_code)
    );

    // After updating the recovery_code for the second user it should be changed as well
    store_recovery_code(&db, &wallet_id, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");

    let wallet_user = find_wallet_user_by_wallet_id(&db, &wallet_id).await.unwrap();
    let other_wallet_user = find_wallet_user_by_wallet_id(&db, &other_wallet_id).await.unwrap();

    assert_matches!(
        wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.as_ref().is_some_and(|rc| rc == &recovery_code)
    );
    assert_matches!(
        other_wallet_user, WalletUserQueryResult::Found(wallet_user) if wallet_user.recovery_code.as_ref().is_some_and(|rc| rc == &recovery_code)
    );

    // Updating the recovery_code for the first user again should give an error
    store_recovery_code(&db, &wallet_id, recovery_code.clone())
        .await
        .expect_err("storing the recovery code twice should error");
    store_recovery_code(&db, &other_wallet_id, recovery_code)
        .await
        .expect_err("storing the recovery code twice should error");
}

#[tokio::test]
async fn test_has_multiple_accounts() {
    // Prepare three wallet users
    let (db, _, wallet_id1, _) = common::create_test_user().await;
    let (_, _, wallet_id2, _) = common::create_test_user().await;
    let (_, _, wallet_id3, _) = common::create_test_user().await;

    let recovery_code = Uuid::new_v4().to_string();

    // There is only one wallet user having the recovery_code
    store_recovery_code(&db, &wallet_id1, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");
    assert!(
        !has_multiple_active_accounts_by_recovery_code(&db, &recovery_code)
            .await
            .unwrap()
    );

    // There are two wallet users having the recovery_code
    store_recovery_code(&db, &wallet_id2, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");
    assert!(
        has_multiple_active_accounts_by_recovery_code(&db, &recovery_code)
            .await
            .unwrap()
    );

    // There are trhee wallet users having the recovery_code
    store_recovery_code(&db, &wallet_id3, recovery_code.clone())
        .await
        .expect("storing the recovery code should succeed");
    assert!(
        has_multiple_active_accounts_by_recovery_code(&db, &recovery_code)
            .await
            .unwrap()
    );

    // After blocking one of the wallet users, there are two wallet users having the recovery_code
    register_unsuccessful_pin_entry(&db, &wallet_id3, true, Utc::now())
        .await
        .unwrap();
    assert!(
        has_multiple_active_accounts_by_recovery_code(&db, &recovery_code)
            .await
            .unwrap()
    );

    // After blocking another of the wallet users, there is only one wallet user having the recovery_code
    register_unsuccessful_pin_entry(&db, &wallet_id2, true, Utc::now())
        .await
        .unwrap();
    assert!(
        !has_multiple_active_accounts_by_recovery_code(&db, &recovery_code)
            .await
            .unwrap()
    );
}
