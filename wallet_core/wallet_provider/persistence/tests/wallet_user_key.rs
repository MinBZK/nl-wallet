use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use uuid::Uuid;

use hsm::model::wrapped_key::WrappedKey;
use wallet_provider_domain::model::wallet_user::WalletUserKey;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_persistence::wallet_user_key::create_keys;
use wallet_provider_persistence::wallet_user_key::find_keys_by_identifiers;

pub mod common;

#[tokio::test]
async fn test_create_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let privkey = SigningKey::random(&mut OsRng);
    let key1 = WalletUserKey {
        wallet_user_key_id: Uuid::new_v4(),
        key_identifier: "key1".to_string(),
        key: WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
    };
    let privkey = SigningKey::random(&mut OsRng);
    let key2 = WalletUserKey {
        wallet_user_key_id: Uuid::new_v4(),
        key_identifier: "key2".to_string(),
        key: WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
    };

    let wallet_id = Uuid::new_v4().to_string();

    let wallet_user_id = common::create_wallet_user_with_random_keys(&db, wallet_id.clone()).await;

    create_keys(
        &db,
        WalletUserKeys {
            wallet_user_id,
            keys: vec![key1.clone(), key2.clone()],
        },
    )
    .await
    .unwrap();

    let mut persisted_keys = find_keys_by_identifiers(&db, wallet_user_id, &["key1".to_string(), "key2".to_string()])
        .await
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    persisted_keys.sort_by_key(|(key, _)| key.clone());
    let keys = persisted_keys
        .iter()
        .map(|(_, key)| key.wrapped_private_key())
        .collect::<Vec<_>>();

    let key1 = key1.key.wrapped_private_key();
    let key2 = key2.key.wrapped_private_key();
    assert_eq!(vec![key1, key2], keys);
}
