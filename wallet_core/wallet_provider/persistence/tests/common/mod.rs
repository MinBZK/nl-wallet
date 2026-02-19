use chrono::DateTime;
use chrono::Utc;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::Query;
use uuid::Uuid;

use crypto::utils::random_bytes;
use crypto::utils::random_string;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_persistence::PersistenceConnection;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::entity::wallet_user;
use wallet_provider_persistence::entity::wallet_user_instruction_challenge;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_persistence::test::db_from_env;
use wallet_provider_persistence::wallet_user::update_instruction_challenge_and_sequence_number;

pub async fn find_wallet_user<S, T>(db: &T, id: Uuid) -> Option<wallet_user::Model>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .filter(wallet_user::Column::Id.eq(id))
        .one(db.connection())
        .await
        .expect("Could not fetch wallet user")
}

pub async fn create_instruction_challenge_with_random_data<S, T>(db: &T, wallet_id: &WalletId)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_instruction_challenge_and_sequence_number(
        db,
        wallet_id,
        InstructionChallenge {
            expiration_date_time: Utc::now(), // irrelevant for these tests
            bytes: random_bytes(32),
        },
        0, // irrelevant for these tests
    )
    .await
    .expect("Could not create wallet user");
}

#[derive(FromQueryResult)]
pub struct InstructionChallengeResult {
    pub id: Uuid,
    pub wallet_user_id: Uuid,
    pub instruction_challenge: Vec<u8>,
    pub expiration_date_time: DateTime<Utc>,
}

pub async fn find_instruction_challenges_by_wallet_id<S, T>(
    db: &T,
    wallet_id: &WalletId,
) -> Vec<InstructionChallengeResult>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let stmt = Query::select()
        .columns([
            wallet_user_instruction_challenge::Column::Id,
            wallet_user_instruction_challenge::Column::WalletUserId,
            wallet_user_instruction_challenge::Column::InstructionChallenge,
            wallet_user_instruction_challenge::Column::ExpirationDateTime,
        ])
        .from(wallet_user_instruction_challenge::Entity)
        .and_where(
            wallet_user_instruction_challenge::Column::WalletUserId.in_subquery(
                Query::select()
                    .column(wallet_user::Column::Id)
                    .from(wallet_user::Entity)
                    .and_where(Expr::col(wallet_user::Column::WalletId).eq(wallet_id.as_ref()))
                    .to_owned(),
            ),
        )
        .to_owned();

    let conn = db.connection();
    let builder = conn.get_database_backend();

    InstructionChallengeResult::find_by_statement(builder.build(&stmt))
        .all(conn)
        .await
        .expect("Could not fetch instruction challenges")
}

pub async fn create_test_user(vendor: WalletDeviceVendor) -> (Db, Uuid, WalletId, wallet_user::Model) {
    let db = db_from_env().await.expect("Could not connect to database");

    let wallet_id: WalletId = random_string(32).into();

    let wallet_user_id = create_wallet_user_with_random_keys(&db, vendor, wallet_id.clone()).await;

    let user = find_wallet_user(&db, wallet_user_id)
        .await
        .expect("Wallet user not found");

    (db, wallet_user_id, wallet_id, user)
}
