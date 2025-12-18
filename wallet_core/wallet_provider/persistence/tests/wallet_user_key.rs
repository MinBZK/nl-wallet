use std::collections::HashSet;
use std::slice::from_ref;

use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use uuid::Uuid;

use hsm::model::wrapped_key::WrappedKey;
use wallet_provider_domain::model::wallet_user::WalletUserKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_persistence::wallet_user_key::delete_blocked_keys;
use wallet_provider_persistence::wallet_user_key::find_active_keys_by_identifiers;
use wallet_provider_persistence::wallet_user_key::is_blocked_key;
use wallet_provider_persistence::wallet_user_key::move_keys;
use wallet_provider_persistence::wallet_user_key::persist_keys;
use wallet_provider_persistence::wallet_user_key::unblock_blocked_keys;

pub mod common;

fn test_wallet_user_key() -> WalletUserKey {
    let privkey = SigningKey::random(&mut OsRng);
    let key = WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key());
    WalletUserKey {
        wallet_user_key_id: Uuid::new_v4(),
        key,
        is_blocked: false,
    }
}

#[tokio::test]
async fn test_create_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let key1 = WalletUserKey {
        is_blocked: false,
        ..test_wallet_user_key()
    };
    let key2 = WalletUserKey {
        is_blocked: false,
        ..test_wallet_user_key()
    };

    let wallet_id = Uuid::new_v4().to_string();

    let wallet_user_id = common::create_wallet_user_with_random_keys(&db, wallet_id.clone()).await;

    persist_keys(
        &db,
        WalletUserKeys {
            wallet_user_id,
            keys: vec![key1.clone(), key2.clone()],
        },
    )
    .await
    .unwrap();

    let mut persisted_keys = find_active_keys_by_identifiers(
        &db,
        wallet_user_id,
        &[key1.sha256_fingerprint(), key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<Vec<_>>();
    persisted_keys.sort_by_key(|(key, _)| key.clone());
    let keys = persisted_keys
        .iter()
        .map(|(_, key)| key.wrapped_private_key())
        .collect::<HashSet<_>>();

    let key1 = key1.key.wrapped_private_key();
    let key2 = key2.key.wrapped_private_key();
    assert_eq!(HashSet::from_iter([key1, key2]), keys);
}

#[tokio::test]
async fn test_move_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let source_key1 = WalletUserKey {
        is_blocked: false,
        ..test_wallet_user_key()
    };
    let source_key2 = WalletUserKey {
        is_blocked: false,
        ..test_wallet_user_key()
    };
    let destination_key1 = WalletUserKey {
        is_blocked: false,
        ..test_wallet_user_key()
    };

    let source_wallet_id = Uuid::new_v4();
    let source_wallet_user_id = common::create_wallet_user_with_random_keys(&db, source_wallet_id.to_string()).await;

    let destination_wallet_id = Uuid::new_v4();
    let destination_wallet_user_id =
        common::create_wallet_user_with_random_keys(&db, destination_wallet_id.to_string()).await;

    // Create example keys in source and destination wallets

    persist_keys(
        &db,
        WalletUserKeys {
            wallet_user_id: source_wallet_user_id,
            keys: vec![source_key1.clone(), source_key2.clone()],
        },
    )
    .await
    .unwrap();

    persist_keys(
        &db,
        WalletUserKeys {
            wallet_user_id: destination_wallet_user_id,
            keys: vec![destination_key1.clone()],
        },
    )
    .await
    .unwrap();

    // Verify the keys are persisted correctly

    let persisted_source_keys = find_active_keys_by_identifiers(
        &db,
        source_wallet_user_id,
        &[source_key1.sha256_fingerprint(), source_key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_keys()
    .collect::<HashSet<_>>();

    assert_eq!(
        HashSet::from([source_key1.sha256_fingerprint(), source_key2.sha256_fingerprint()]),
        persisted_source_keys
    );

    let persisted_destination_keys = find_active_keys_by_identifiers(
        &db,
        destination_wallet_user_id,
        from_ref(&destination_key1.sha256_fingerprint()),
    )
    .await
    .unwrap()
    .into_keys()
    .collect::<HashSet<_>>();

    assert_eq!(
        HashSet::from([destination_key1.sha256_fingerprint()]),
        persisted_destination_keys
    );

    // Move the keys

    move_keys(&db, source_wallet_user_id, destination_wallet_user_id)
        .await
        .unwrap();

    // Verify that the keys are moved correctly

    let persisted_source_keys = find_active_keys_by_identifiers(
        &db,
        source_wallet_user_id,
        &[source_key1.sha256_fingerprint(), source_key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<HashSet<_>>();
    assert!(persisted_source_keys.is_empty());

    let persisted_destination_keys = find_active_keys_by_identifiers(
        &db,
        destination_wallet_user_id,
        &[source_key1.sha256_fingerprint(), source_key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_keys()
    .collect::<HashSet<_>>();

    assert_eq!(
        HashSet::from([source_key1.sha256_fingerprint(), source_key2.sha256_fingerprint()]),
        persisted_destination_keys
    );
}

#[tokio::test]
async fn test_create_blocked_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let key1 = WalletUserKey {
        is_blocked: true,
        ..test_wallet_user_key()
    };
    let key2 = WalletUserKey {
        is_blocked: true,
        ..test_wallet_user_key()
    };

    let wallet_id = Uuid::new_v4().to_string();

    let wallet_user_id = common::create_wallet_user_with_random_keys(&db, wallet_id.clone()).await;

    persist_keys(
        &db,
        WalletUserKeys {
            wallet_user_id,
            keys: vec![key1.clone(), key2.clone()],
        },
    )
    .await
    .unwrap();

    // Blocked keys should not be retrieved by `find_active_keys_by_identifiers`
    let active_keys = find_active_keys_by_identifiers(
        &db,
        wallet_user_id,
        &[key1.sha256_fingerprint(), key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<Vec<_>>();
    assert!(active_keys.is_empty());

    // Check whether both keys are blocked
    for key in [&key1, &key2] {
        assert!(
            is_blocked_key(&db, wallet_user_id, *key.key.public_key())
                .await
                .unwrap()
                .unwrap()
        );
    }

    // Unblock keys
    unblock_blocked_keys(&db, wallet_user_id).await.unwrap();

    // Keys should be active now
    let active_keys = find_active_keys_by_identifiers(
        &db,
        wallet_user_id,
        &[key1.sha256_fingerprint(), key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<Vec<_>>();
    assert_eq!(active_keys.len(), 2);
}

#[tokio::test]
async fn test_delete_blocked_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let key1 = WalletUserKey {
        is_blocked: true,
        ..test_wallet_user_key()
    };
    let key2 = WalletUserKey {
        is_blocked: true,
        ..test_wallet_user_key()
    };

    let wallet_id = Uuid::new_v4().to_string();

    let wallet_user_id = common::create_wallet_user_with_random_keys(&db, wallet_id.clone()).await;

    persist_keys(
        &db,
        WalletUserKeys {
            wallet_user_id,
            keys: vec![key1.clone(), key2.clone()],
        },
    )
    .await
    .unwrap();

    // Blocked keys should not be retrieved by `find_active_keys_by_identifiers`
    let active_keys = find_active_keys_by_identifiers(
        &db,
        wallet_user_id,
        &[key1.sha256_fingerprint(), key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<Vec<_>>();
    assert!(active_keys.is_empty());

    // Delete the blocked keys
    delete_blocked_keys(&db, wallet_user_id).await.unwrap();

    // Keys should no longer be found
    for key in [&key1, &key2] {
        assert!(
            is_blocked_key(&db, wallet_user_id, *key.key.public_key())
                .await
                .unwrap()
                .is_none()
        );
    }

    // Try to unblock keys
    unblock_blocked_keys(&db, wallet_user_id).await.unwrap();

    // Keys should not be found
    let active_keys = find_active_keys_by_identifiers(
        &db,
        wallet_user_id,
        &[key1.sha256_fingerprint(), key2.sha256_fingerprint()],
    )
    .await
    .unwrap()
    .into_iter()
    .collect::<Vec<_>>();
    assert!(active_keys.is_empty());
}
