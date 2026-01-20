use std::num::NonZeroUsize;
use std::time::Duration;
use std::vec;

use futures::future::join_all;
use futures::future::try_join_all;
use itertools::Either;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use rstest::rstest;
use utils::num::NonZeroU31;
use utils::num::U31;
use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use crypto::server_keys::generate::Ca;
use crypto::utils::random_string;
use hsm::model::mock::MockPkcs11Client;
use hsm::service::HsmError;
use status_lists::config::StatusListConfig;
use status_lists::postgres::PostgresStatusListService;
use status_lists::publish::PublishDir;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_service::StatusListRevocationService;
use token_status_list::status_list_service::StatusListService;
use token_status_list::status_list_token::StatusListToken;
use utils::generator::mock::MockTimeGenerator;
use wallet_provider_domain::model::wallet_user::RevocationReason;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_persistence::PersistenceConnection;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_env;
use wallet_provider_persistence::wallet_user;
use wallet_provider_persistence::wallet_user_wua;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::account_server::mock::user_state;
use wallet_provider_service::revocation::revoke_all_wallets;
use wallet_provider_service::revocation::revoke_wallets;
use wallet_provider_service::wua_issuer::mock::MockWuaIssuer;

async fn setup_state(
    publish_dir: PublishDir,
) -> UserState<Repositories, MockPkcs11Client<HsmError>, MockWuaIssuer, PostgresStatusListService<SigningKey>> {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let db: Db = db_from_env().await.unwrap();

    let key_pair = ca.generate_status_list_mock().unwrap();

    let config = StatusListConfig {
        list_size: NonZeroU31::try_new(100).unwrap(),
        create_threshold: U31::ZERO, // no new lists are needed during test
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,
        base_url: "https://example.com/".parse().unwrap(),
        publish_dir,
        key_pair,
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

async fn register_wallets_to_revoke(
    wuas_per_wallet: Vec<usize>,
    user_state: &UserState<
        Repositories,
        MockPkcs11Client<HsmError>,
        MockWuaIssuer,
        PostgresStatusListService<SigningKey>,
    >,
) -> (Vec<String>, Vec<Vec<(Uuid, StatusClaim)>>) {
    let (wallets, wuas): (Vec<String>, Vec<Vec<(Uuid, StatusClaim)>>) =
        join_all(wuas_per_wallet.into_iter().map(async |wua_count| {
            let tx = user_state.repositories.begin_transaction().await.unwrap();
            let wallet_id = random_string(10);

            // manually create a user and some WUA IDs, bypassing registration logic
            let user_uuid = create_wallet_user_with_random_keys(&tx, wallet_id.clone()).await;

            let mut wuas: Vec<(Uuid, StatusClaim)> = vec![];
            for _ in 0..wua_count {
                let wua_id = Uuid::new_v4();
                let claim = user_state
                    .status_list_service
                    .obtain_status_claims(wua_id, None, NonZeroUsize::MIN)
                    .await
                    .unwrap()
                    .into_iter()
                    .exactly_one() // only one claim per WUA ID
                    .unwrap();
                wallet_user_wua::create(&tx, user_uuid, wua_id).await.unwrap();

                wuas.push((wua_id, claim));
            }

            tx.commit().await.unwrap();
            (wallet_id, wuas)
        }))
        .await
        .into_iter()
        .unzip();

    (wallets, wuas)
}

async fn status_type_for_claim(StatusClaim::StatusList(claim): &StatusClaim, publish_dir: &PublishDir) -> StatusType {
    let external_id = claim.uri.path().split('/').next_back().unwrap();
    let path = publish_dir.jwt_path(external_id);
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
        .single_unpack(
            claim
                .idx
                .try_into()
                .expect("should always work on 32- or higher bit systems"),
        )
}

async fn verify_revocation(
    wallet_ids: impl IntoIterator<Item = &String>,
    expected_revocation_reason: Option<RevocationReason>,
    wua_id_and_claim: impl IntoIterator<Item = &(Uuid, StatusClaim)>,
    publish_dir: &PublishDir,
    user_state: &UserState<
        Repositories,
        MockPkcs11Client<HsmError>,
        MockWuaIssuer,
        PostgresStatusListService<SigningKey>,
    >,
    expected_status_type: StatusType,
) {
    // verify wallet revocation
    join_all(wallet_ids.into_iter().map(async |wallet_id| {
        let tx = user_state.repositories.begin_transaction().await.unwrap();
        let wallet_user = wallet_user::find_wallet_user_by_wallet_id(&tx, wallet_id)
            .await
            .unwrap()
            .unwrap_found();
        assert_eq!(
            wallet_user.state == WalletUserState::Revoked,
            expected_revocation_reason.is_some()
        );
        assert_eq!(
            wallet_user.revocation_registration.is_some(),
            expected_revocation_reason.is_some()
        );
        assert_eq!(
            wallet_user.revocation_registration.map(|r| r.reason),
            expected_revocation_reason
        );
        tx.commit().await.unwrap();
    }))
    .await;

    // verify wua revocation
    join_all(wua_id_and_claim.into_iter().map(async |(wua_id, wua_claim)| {
        let batch = user_state
            .status_list_service
            .get_attestation_batch(*wua_id)
            .await
            .unwrap();
        assert_eq!(expected_status_type == StatusType::Invalid, batch.is_revoked);

        // since the status list is not served in this test, we read it directly from disk
        let status_type = status_type_for_claim(wua_claim, publish_dir).await;
        assert_eq!(status_type, expected_status_type);
    }))
    .await;
}

fn partition_vec_by_indices<T>(iterator: impl IntoIterator<Item = T>, indices_to_revoke: &[usize]) -> (Vec<T>, Vec<T>) {
    iterator.into_iter().enumerate().partition_map(|(i, val)| {
        if indices_to_revoke.contains(&i) {
            Either::Left(val)
        } else {
            Either::Right(val)
        }
    })
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

    let (wallets, wuas) = register_wallets_to_revoke(wuas_per_wallet, &user_state).await;

    // all wuas should not be revoked
    verify_revocation(
        wallets.iter(),
        None,
        wuas.iter().flatten(),
        &publish_dir,
        &user_state,
        StatusType::Valid,
    )
    .await;

    let (wallet_ids_to_revoke, wallet_ids_not_to_revoke) = partition_vec_by_indices(wallets, &indices_to_revoke);
    revoke_wallets(wallet_ids_to_revoke.iter(), &user_state, &MockTimeGenerator::default())
        .await
        .unwrap();
    let (revoked_wua_ids, non_revoked_wua_ids) = partition_vec_by_indices(wuas, &indices_to_revoke);

    let revoked_wua_ids = revoked_wua_ids.into_iter().flatten().collect_vec();
    let non_revoked_wua_ids = non_revoked_wua_ids.into_iter().flatten().collect_vec();

    // check revoked wuas
    verify_revocation(
        wallet_ids_to_revoke.iter(),
        Some(RevocationReason::AdminRequest),
        revoked_wua_ids.iter(),
        &publish_dir,
        &user_state,
        StatusType::Invalid,
    )
    .await;
    verify_revocation(
        wallet_ids_not_to_revoke.iter(),
        None,
        non_revoked_wua_ids.iter(),
        &publish_dir,
        &user_state,
        StatusType::Valid,
    )
    .await;

    // verify idempotency
    revoke_wallets(wallet_ids_to_revoke.iter(), &user_state, &MockTimeGenerator::default())
        .await
        .unwrap();
    verify_revocation(
        wallet_ids_to_revoke.iter(),
        Some(RevocationReason::AdminRequest),
        revoked_wua_ids.iter(),
        &publish_dir,
        &user_state,
        StatusType::Invalid,
    )
    .await;
    verify_revocation(
        wallet_ids_not_to_revoke.iter(),
        None,
        non_revoked_wua_ids.iter(),
        &publish_dir,
        &user_state,
        StatusType::Valid,
    )
    .await;
}

#[tokio::test]
#[rstest]
#[case(vec![1])]
#[case(vec![4, 4, 4])]
#[case(vec![0, 10, 10])]
#[ignore] // TODO this test fails due to the DB already containing token status lists (PVW-5455)
async fn test_revoke_all(#[case] wuas_per_wallet: Vec<usize>) {
    let temp_dir = tempfile::tempdir().unwrap();
    let publish_dir = PublishDir::try_new(temp_dir.path().to_path_buf()).unwrap();
    let user_state = setup_state(publish_dir.clone()).await;

    let (wallet_ids, wuas) = register_wallets_to_revoke(wuas_per_wallet, &user_state).await;
    let wuas: Vec<(Uuid, StatusClaim)> = wuas.into_iter().flatten().collect();

    // all wuas should not be revoked
    verify_revocation(
        wallet_ids.iter(),
        None,
        wuas.iter(),
        &publish_dir,
        &user_state,
        StatusType::Valid,
    )
    .await;

    revoke_all_wallets(&user_state, &MockTimeGenerator::default())
        .await
        .unwrap();

    // all wuas should be revoked
    verify_revocation(
        wallet_ids.iter(),
        Some(RevocationReason::WalletSolutionCompromised),
        wuas.iter(),
        &publish_dir,
        &user_state,
        StatusType::Invalid,
    )
    .await;

    // verify idempotency
    revoke_all_wallets(&user_state, &MockTimeGenerator::default())
        .await
        .unwrap();
    verify_revocation(
        wallet_ids.iter(),
        Some(RevocationReason::WalletSolutionCompromised),
        wuas.iter(),
        &publish_dir,
        &user_state,
        StatusType::Invalid,
    )
    .await;
}
