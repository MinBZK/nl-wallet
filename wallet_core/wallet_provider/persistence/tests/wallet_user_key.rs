use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use uuid::Uuid;

use wallet_provider_domain::model::wallet_user::WalletUserKeysCreate;
use wallet_provider_persistence::wallet_user_key::{create_keys, find_keys_by_identifiers};

pub mod common;

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
async fn test_create_keys() {
    let db = common::db_from_env().await.expect("Could not connect to database");

    let key1 = (Uuid::new_v4(), "key1".to_string(), SigningKey::random(&mut OsRng));
    let key2 = (Uuid::new_v4(), "key2".to_string(), SigningKey::random(&mut OsRng));

    let wallet_user_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4().to_string();

    common::create_wallet_user_with_random_keys(&db, wallet_user_id, wallet_id.clone()).await;

    create_keys(
        &db,
        WalletUserKeysCreate {
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
    let keys = persisted_keys.into_iter().map(|(_, key)| key).collect::<Vec<_>>();

    assert_eq!(vec![key1.2, key2.2], keys);
}
