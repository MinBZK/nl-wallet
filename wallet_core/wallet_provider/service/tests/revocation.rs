use std::num::NonZeroUsize;
use std::time::Duration;
use std::vec;

use futures::future::join_all;
use futures::future::try_join_all;
use itertools::Itertools;
use rstest::rstest;
use utils::num::NonZeroU31;
use utils::num::U31;
use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use hsm::model::mock::MockPkcs11Client;
use hsm::service::HsmError;
use server_utils::keys::test::private_key_variant;
use status_lists::config::StatusListConfig;
use status_lists::postgres::PostgresStatusListService;
use status_lists::publish::PublishDir;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListService;
use token_status_list::status_list_token::StatusListToken;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_persistence::PersistenceConnection;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_env;
use wallet_provider_persistence::wallet_user_wua;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::account_server::mock::user_state;
use wallet_provider_service::revocation::revoke_all_wallets;
use wallet_provider_service::revocation::revoke_wallets;
use wallet_provider_service::wua_issuer::mock::MockWuaIssuer;

async fn setup_state(
    publish_dir: PublishDir,
) -> UserState<Repositories, MockPkcs11Client<HsmError>, MockWuaIssuer, PostgresStatusListService> {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db: Db = db_from_env().await.unwrap();

    let private_key = private_key_variant(ca.generate_status_list_mock().unwrap()).await;

    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(100).unwrap(),
        create_threshold: U31::ZERO, // no new lists are needed during test
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,
        base_url: "https://example.com/".parse().unwrap(),
        publish_dir,
        key_pair: private_key.clone(),
    };

    let service = PostgresStatusListService::try_new(db.connection().clone(), &random_string(20), config)
        .await
        .unwrap();
    try_join_all(service.initialize_lists().await.unwrap().into_iter())
        .await
        .unwrap();

    user_state(
        Repositories::from(db),
        MockPkcs11Client::default(),
        "wrapping_key_identifier".to_owned(),
        vec![],
        service,
    )
}

async fn status_type_for_claim(StatusClaim::StatusList(claim): &StatusClaim, publish_dir: &PublishDir) -> StatusType {
    let external_id = claim.uri.path().split('/').last().unwrap();
    let path = publish_dir.jwt_path(&external_id);
    tokio::fs::read_to_string(path)
        .await
        .unwrap()
        .parse::<StatusListToken>()
        .unwrap()
        .as_ref()
        .dangerous_parse_unverified()
        .unwrap()
        .1
        .status_list
        .single_unpack(claim.idx as usize)
}

#[tokio::test]
#[rstest]
#[case(vec![1], vec![0])]
#[case(vec![4, 4, 4], vec![2])]
#[case(vec![4, 4, 4], vec![0, 1, 2])]
#[case(vec![0, 10, 10], vec![0])]
#[case(vec![0, 10, 10], vec![1])]
async fn test_revoke_wallet(#[case] wuas_per_wallet: Vec<usize>, #[case] indices_to_revoke: Vec<usize>) {
    let temp_dir = tempfile::tempdir().unwrap();
    let publish_dir = PublishDir::try_new(temp_dir.path().to_path_buf()).unwrap();
    let user_state = setup_state(publish_dir.clone()).await;

    let (wallets, wuas): (Vec<String>, Vec<Vec<(Uuid, StatusClaim)>>) =
        join_all(wuas_per_wallet.into_iter().map(async |wua_count| {
            let tx = user_state.repositories.begin_transaction().await.unwrap();
            let wallet_user_id = random_string(10);

            // manually create a user and some WUA IDs, bypassing registration logic
            let user_uuid = create_wallet_user_with_random_keys(&tx, wallet_user_id.clone()).await;

            let mut wuas: Vec<(Uuid, StatusClaim)> = vec![];
            for _ in 0..wua_count {
                let wua_id = Uuid::new_v4();
                let claim = user_state
                    .status_list_service
                    .obtain_status_claims(wua_id, None, NonZeroUsize::MIN)
                    .await
                    .unwrap()
                    .into_first(); // only one claim per WUA ID
                wallet_user_wua::create(&tx, user_uuid, wua_id).await.unwrap();

                wuas.push((wua_id, claim));
            }

            tx.commit().await.unwrap();
            (wallet_user_id, wuas)
        }))
        .await
        .into_iter()
        .unzip();

    // all wuas should not be revoked
    join_all(wuas.iter().flatten().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(!batch.is_revoked);
    }))
    .await;

    let wallet_ids_to_revoke = indices_to_revoke
        .iter()
        .filter_map(|i| wallets.get(*i))
        .cloned()
        .collect_vec();
    revoke_wallets(wallet_ids_to_revoke.clone(), &user_state).await.unwrap();

    let revoked_wua_ids = wuas
        .iter()
        .enumerate()
        .filter_map(|(i, val)| indices_to_revoke.contains(&i).then_some(val.clone()))
        .collect_vec();
    let non_revoked_wua_ids = wuas
        .iter()
        .enumerate()
        .filter_map(|(i, val)| (!indices_to_revoke.contains(&i)).then_some(val.clone()))
        .collect_vec();

    // check revoked wuas
    join_all(revoked_wua_ids.iter().flatten().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(batch.is_revoked);
    }))
    .await;
    join_all(non_revoked_wua_ids.iter().flatten().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(!batch.is_revoked);
    }))
    .await;

    // verify idempotency
    revoke_wallets(wallet_ids_to_revoke, &user_state).await.unwrap();
    join_all(revoked_wua_ids.iter().flatten().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(batch.is_revoked);
    }))
    .await;
    join_all(non_revoked_wua_ids.iter().flatten().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(!batch.is_revoked);
    }))
    .await;

    join_all(revoked_wua_ids.iter().flatten().map(async |(_, wua_claim)| {
        // since the status list is not served in this test, we read it directly from disk
        let status_type = status_type_for_claim(wua_claim, &publish_dir).await;

        assert_eq!(status_type, StatusType::Invalid);
    }))
    .await;
    join_all(non_revoked_wua_ids.iter().flatten().map(async |(_, wua_claim)| {
        // since the status list is not served in this test, we read it directly from disk
        let status_type = status_type_for_claim(wua_claim, &publish_dir).await;

        assert_eq!(status_type, StatusType::Valid);
    }))
    .await;
}

#[tokio::test]
#[rstest]
#[case(vec![1])]
#[case(vec![4, 4, 4])]
#[case(vec![0, 10, 10])]
#[ignore] // TODO this test fails due to the DB already containing token status lists
async fn test_revoke_all(#[case] wuas_per_wallet: Vec<usize>) {
    let temp_dir = tempfile::tempdir().unwrap();
    let publish_dir = PublishDir::try_new(temp_dir.path().to_path_buf()).unwrap();
    let user_state = setup_state(publish_dir.clone()).await;

    let wuas = join_all(wuas_per_wallet.into_iter().map(async |wua_count| {
        let tx = user_state.repositories.begin_transaction().await.unwrap();
        let wallet_user_id = random_string(10);

        // manually create a user and some WUA IDs, bypassing registration logic
        let user_uuid = create_wallet_user_with_random_keys(&tx, wallet_user_id.clone()).await;

        let mut wua_uuids: Vec<(Uuid, StatusClaim)> = vec![];
        for _ in 0..wua_count {
            let wua_id = Uuid::new_v4();
            let claim = user_state
                .status_list_service
                .obtain_status_claims(wua_id, None, NonZeroUsize::MIN)
                .await
                .unwrap()
                .into_first();
            wallet_user_wua::create(&tx, user_uuid, wua_id).await.unwrap();

            wua_uuids.push((wua_id, claim));
        }

        tx.commit().await.unwrap();
        wua_uuids
    }))
    .await
    .into_iter()
    .flatten()
    .collect_vec();

    // all wuas should not be revoked
    join_all(wuas.iter().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(!batch.is_revoked);
    }))
    .await;

    revoke_all_wallets(&user_state).await.unwrap();

    // all wuas should be revoked
    join_all(wuas.iter().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(batch.is_revoked);
    }))
    .await;

    // verify idempotency
    revoke_all_wallets(&user_state).await.unwrap();
    join_all(wuas.iter().map(async |(wua_id, _)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();

        assert!(batch.is_revoked);
    }))
    .await;

    join_all(wuas.iter().map(async |(_, wua_claim)| {
        // since the status list is not served in this test, we read it directly from disk
        let status_type = status_type_for_claim(wua_claim, &publish_dir).await;

        assert_eq!(status_type, StatusType::Invalid);
    }))
    .await;
}
