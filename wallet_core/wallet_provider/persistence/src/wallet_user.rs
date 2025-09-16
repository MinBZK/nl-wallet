use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::JoinType;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::IntoIden;
use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::Query;
use sea_orm::sea_query::SimpleExpr;
use semver::Version;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::encrypted::InitializationVector;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::TransferSession;
use wallet_provider_domain::model::wallet_user::TransferSessionState;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
use wallet_provider_domain::model::wallet_user::WalletUserAttestationCreate;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::wallet_transfer;
use crate::entity::wallet_user;
use crate::entity::wallet_user_android_attestation;
use crate::entity::wallet_user_apple_attestation;
use crate::entity::wallet_user_instruction_challenge;

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn create_wallet_user<S, T>(db: &T, user: WalletUserCreate) -> Result<Uuid>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let user_id = Uuid::new_v4();
    let connection = db.connection();

    let (apple_attestation_id, android_attestation_id) = match user.attestation {
        WalletUserAttestationCreate::Apple {
            data,
            assertion_counter,
        } => {
            let id = Uuid::new_v4();

            wallet_user_apple_attestation::ActiveModel {
                id: Set(id),
                assertion_counter: Set((*assertion_counter).into()),
                attestation_data: Set(data),
            }
            .insert(connection)
            .await
            .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

            (Some(id), None)
        }
        WalletUserAttestationCreate::Android {
            certificate_chain,
            integrity_verdict_json,
        } => {
            let id = Uuid::new_v4();

            wallet_user_android_attestation::ActiveModel {
                id: Set(id),
                certificate_chain: Set(certificate_chain),
                integrity_verdict_json: Set(integrity_verdict_json),
            }
            .insert(connection)
            .await
            .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

            (None, Some(id))
        }
    };

    wallet_user::ActiveModel {
        id: Set(user_id),
        wallet_id: Set(user.wallet_id),
        hw_pubkey_der: Set(user.hw_pubkey.to_public_key_der()?.to_vec()),
        encrypted_pin_pubkey_sec1: Set(user.encrypted_pin_pubkey.data),
        pin_pubkey_iv: Set(user.encrypted_pin_pubkey.iv.0),
        encrypted_previous_pin_pubkey_sec1: Set(None),
        previous_pin_pubkey_iv: Set(None),
        instruction_sequence_number: Set(0),
        pin_entries: Set(0),
        last_unsuccessful_pin: Set(None),
        state: Set(WalletUserState::Active.to_string()),
        attestation_date_time: Set(user.attestation_date_time.into()),
        apple_attestation_id: Set(apple_attestation_id),
        android_attestation_id: Set(android_attestation_id),
        recovery_code: Set(None),
    }
    .insert(connection)
    .await
    .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(user_id)
}

#[derive(FromQueryResult)]
struct WalletUserJoinedModel {
    state: String,
    id: Uuid,
    wallet_id: String,
    hw_pubkey_der: Vec<u8>,
    encrypted_pin_pubkey_sec1: Vec<u8>,
    pin_pubkey_iv: Vec<u8>,
    encrypted_previous_pin_pubkey_sec1: Option<Vec<u8>>,
    previous_pin_pubkey_iv: Option<Vec<u8>>,
    pin_entries: i16,
    last_unsuccessful_pin: Option<DateTimeWithTimeZone>,
    instruction_challenge: Option<Vec<u8>>,
    instruction_challenge_expiration_date_time: Option<DateTimeWithTimeZone>,
    instruction_sequence_number: i32,
    apple_assertion_counter: Option<i64>,
    recovery_code: Option<String>,
}

pub fn transfer_session_from_model(
    model: wallet_transfer::Model,
    destination_wallet_recovery_code: String,
) -> TransferSession {
    TransferSession {
        id: model.id,
        destination_wallet_user_id: model.destination_wallet_user_id,
        transfer_session_id: model.transfer_session_id,
        destination_wallet_app_version: Version::parse(&model.destination_wallet_app_version).unwrap(),
        destination_wallet_recovery_code,
        state: model
            .state
            .parse()
            .expect("parsing the wallet transfer state from the database should always succeed"),
        encrypted_wallet_data: model.encrypted_wallet_data,
    }
}

/// Find a user by its `wallet_id` and return it, if it exists.
/// Note that this function will also return blocked users.
pub async fn find_wallet_user_by_wallet_id<S, T>(db: &T, wallet_id: &str) -> Result<WalletUserQueryResult>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let Some((user_model, transfer_model)) = wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::State)
        .column(wallet_user::Column::Id)
        .column(wallet_user::Column::WalletId)
        .column(wallet_user::Column::HwPubkeyDer)
        .column(wallet_user::Column::EncryptedPinPubkeySec1)
        .column(wallet_user::Column::PinPubkeyIv)
        .column(wallet_user::Column::EncryptedPreviousPinPubkeySec1)
        .column(wallet_user::Column::PreviousPinPubkeyIv)
        .column(wallet_user::Column::PinEntries)
        .column(wallet_user::Column::LastUnsuccessfulPin)
        .column(wallet_user::Column::RecoveryCode)
        .column(wallet_user_instruction_challenge::Column::InstructionChallenge)
        .column_as(
            wallet_user_instruction_challenge::Column::ExpirationDateTime,
            "instruction_challenge_expiration_date_time",
        )
        .column(wallet_user::Column::InstructionSequenceNumber)
        .column_as(
            wallet_user_apple_attestation::Column::AssertionCounter,
            "apple_assertion_counter",
        )
        .join(
            JoinType::LeftJoin,
            wallet_user::Relation::WalletUserInstructionChallenge.def(),
        )
        .join(
            JoinType::LeftJoin,
            wallet_user::Relation::WalletUserAppleAttestation.def(),
        )
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .find_also_related(wallet_transfer::Entity)
        .into_model::<WalletUserJoinedModel, wallet_transfer::Model>()
        .one(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?
    else {
        return Ok(WalletUserQueryResult::NotFound);
    };

    let state: WalletUserState = user_model
        .state
        .parse()
        .expect("parsing the wallet user state from the database should always succeed");

    let encrypted_pin_pubkey = Encrypted::new(
        user_model.encrypted_pin_pubkey_sec1,
        InitializationVector(user_model.pin_pubkey_iv),
    );
    let encrypted_previous_pin_pubkey = match (
        user_model.encrypted_previous_pin_pubkey_sec1,
        user_model.previous_pin_pubkey_iv,
    ) {
        (Some(sec1), Some(iv)) => Some(Encrypted::new(sec1, InitializationVector(iv))),
        _ => None,
    };
    let instruction_challenge = match (
        user_model.instruction_challenge,
        user_model.instruction_challenge_expiration_date_time,
    ) {
        (Some(instruction_challenge), Some(expiration_date_time)) => Some(InstructionChallenge {
            bytes: instruction_challenge,
            expiration_date_time: DateTime::<Utc>::from(expiration_date_time),
        }),
        _ => None,
    };
    let attestation = match user_model.apple_assertion_counter {
        Some(counter) => WalletUserAttestation::Apple {
            assertion_counter: AssertionCounter::from(u32::try_from(counter).unwrap()),
        },
        // If the JOIN results in an assertion counter of NULL, we can safely assume that this
        // user has registered using an Android attestation instead. This is enforced by the
        // CHECK statement on the table.
        None => WalletUserAttestation::Android,
    };
    let wallet_user = WalletUser {
        id: user_model.id,
        wallet_id: user_model.wallet_id,
        encrypted_pin_pubkey,
        encrypted_previous_pin_pubkey,
        hw_pubkey: VerifyingKey::from_public_key_der(&user_model.hw_pubkey_der).unwrap(),
        unsuccessful_pin_entries: user_model.pin_entries.try_into().ok().unwrap_or(u8::MAX),
        last_unsuccessful_pin_entry: user_model.last_unsuccessful_pin.map(DateTime::<Utc>::from),
        instruction_challenge,
        instruction_sequence_number: u64::try_from(user_model.instruction_sequence_number).unwrap(),
        attestation,
        state,
        recovery_code: user_model.recovery_code.clone(),
        transfer_session: transfer_model.and_then(|transfer| {
            user_model
                .recovery_code
                .map(|recovery_code| transfer_session_from_model(transfer, recovery_code))
        }),
    };

    Ok(WalletUserQueryResult::Found(Box::new(wallet_user)))
}

pub async fn clear_instruction_challenge<S, T>(db: &T, wallet_id: &str) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let stmt = Query::delete()
        .from_table(wallet_user_instruction_challenge::Entity)
        .and_where(
            wallet_user_instruction_challenge::Column::WalletUserId.in_subquery(
                Query::select()
                    .column(wallet_user::Column::Id)
                    .from(wallet_user::Entity)
                    .and_where(Expr::col(wallet_user::Column::WalletId).eq(wallet_id))
                    .to_owned(),
            ),
        )
        .to_owned();

    let conn = db.connection();
    let builder = conn.get_database_backend();
    conn.execute(builder.build(&stmt))
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn update_instruction_challenge_and_sequence_number<S, T>(
    db: &T,
    wallet_id: &str,
    instruction_challenge: InstructionChallenge,
    instruction_sequence_number: u64,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_instruction_sequence_number(db, wallet_id, instruction_sequence_number).await?;

    // insert a new instruction challenge, or update if one already exists with this wallet.id
    let stmt = Query::insert()
        .into_table(wallet_user_instruction_challenge::Entity)
        .columns([
            wallet_user_instruction_challenge::Column::Id,
            wallet_user_instruction_challenge::Column::WalletUserId,
            wallet_user_instruction_challenge::Column::InstructionChallenge,
            wallet_user_instruction_challenge::Column::ExpirationDateTime,
        ])
        .select_from(
            Query::select()
                .expr(Expr::value(Uuid::new_v4()))
                .column(wallet_user::Column::Id)
                .expr(Expr::value(instruction_challenge.bytes))
                .expr(Expr::value(instruction_challenge.expiration_date_time))
                .from(wallet_user::Entity)
                .and_where(Expr::col(wallet_user::Column::WalletId).eq(wallet_id))
                .to_owned(),
        )
        .map_err(|e| PersistenceError::Execution(e.into()))?
        .on_conflict(
            OnConflict::column(wallet_user_instruction_challenge::Column::WalletUserId)
                .update_columns([
                    wallet_user_instruction_challenge::Column::InstructionChallenge,
                    wallet_user_instruction_challenge::Column::ExpirationDateTime,
                ])
                .to_owned(),
        )
        .to_owned();

    let conn = db.connection();
    let builder = conn.get_database_backend();
    conn.execute(builder.build(&stmt))
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(())
}

pub async fn update_instruction_sequence_number<S, T>(
    db: &T,
    wallet_id: &str,
    instruction_sequence_number: u64,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_fields(
        db,
        wallet_id,
        vec![(
            wallet_user::Column::InstructionSequenceNumber,
            Expr::value(instruction_sequence_number),
        )],
    )
    .await
}

pub async fn register_unsuccessful_pin_entry<S, T>(
    db: &T,
    wallet_id: &str,
    is_blocked: bool,
    datetime: DateTime<Utc>,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_pin_entries(
        db,
        wallet_id,
        // make sure the pin_entries column doesn't overflow
        Expr::cust_with_exprs(
            "least($1, $2)",
            vec![Expr::col(wallet_user::Column::PinEntries).add(1), Expr::value(u8::MAX)],
        ),
        Some(datetime),
        is_blocked,
    )
    .await
}

pub async fn reset_unsuccessful_pin_entries<S, T>(db: &T, wallet_id: &str) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let datetime: Option<DateTime<Utc>> = None;
    update_pin_entries(db, wallet_id, Expr::value(0), datetime, false).await
}

pub async fn change_pin<S, T>(db: &T, wallet_id: &str, new_encrypted_pin_pubkey: Encrypted<VerifyingKey>) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_fields(
        db,
        wallet_id,
        vec![
            (
                wallet_user::Column::EncryptedPinPubkeySec1,
                Expr::value(new_encrypted_pin_pubkey.data),
            ),
            (
                wallet_user::Column::PinPubkeyIv,
                Expr::value(new_encrypted_pin_pubkey.iv.0),
            ),
            (
                wallet_user::Column::EncryptedPreviousPinPubkeySec1,
                Expr::col(wallet_user::Column::EncryptedPinPubkeySec1).into(),
            ),
            (
                wallet_user::Column::PreviousPinPubkeyIv,
                Expr::col(wallet_user::Column::PinPubkeyIv).into(),
            ),
        ],
    )
    .await
}

pub async fn commit_pin_change<S, T>(db: &T, wallet_id: &str) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::update_many()
        .col_expr(wallet_user::Column::EncryptedPreviousPinPubkeySec1, Expr::cust("null"))
        .col_expr(wallet_user::Column::PreviousPinPubkeyIv, Expr::cust("null"))
        .filter(
            wallet_user::Column::WalletId
                .eq(wallet_id)
                .and(wallet_user::Column::EncryptedPreviousPinPubkeySec1.is_not_null())
                .and(wallet_user::Column::PreviousPinPubkeyIv.is_not_null()),
        )
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

pub async fn rollback_pin_change<S, T>(db: &T, wallet_id: &str) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::update_many()
        .col_expr(
            wallet_user::Column::EncryptedPinPubkeySec1,
            Expr::col(wallet_user::Column::EncryptedPreviousPinPubkeySec1).into(),
        )
        .col_expr(
            wallet_user::Column::PinPubkeyIv,
            Expr::col(wallet_user::Column::PreviousPinPubkeyIv).into(),
        )
        .col_expr(wallet_user::Column::EncryptedPreviousPinPubkeySec1, Expr::cust("null"))
        .col_expr(wallet_user::Column::PreviousPinPubkeyIv, Expr::cust("null"))
        .filter(
            wallet_user::Column::WalletId
                .eq(wallet_id)
                .and(wallet_user::Column::EncryptedPreviousPinPubkeySec1.is_not_null())
                .and(wallet_user::Column::PreviousPinPubkeyIv.is_not_null()),
        )
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

async fn update_pin_entries<S, T>(
    db: &T,
    wallet_id: &str,
    pin_entries: SimpleExpr,
    datetime: Option<DateTime<Utc>>,
    is_blocked: bool,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let mut query = wallet_user::Entity::update_many();
    if is_blocked {
        query = query.col_expr(
            wallet_user::Column::State,
            Expr::value(WalletUserState::Blocked.to_string()),
        );
    }

    query
        .col_expr(wallet_user::Column::PinEntries, pin_entries)
        .col_expr(wallet_user::Column::LastUnsuccessfulPin, Expr::value(datetime))
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

async fn update_fields<S, T, C>(db: &T, wallet_id: &str, col_values: Vec<(C, SimpleExpr)>) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
    C: IntoIden,
{
    col_values
        .into_iter()
        .fold(wallet_user::Entity::update_many(), |stmt, col_value| {
            stmt.col_expr(col_value.0, col_value.1)
        })
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .exec(db.connection())
        .await
        .map(|_| ())
        .map_err(|e| PersistenceError::Execution(e.into()))
}

pub async fn update_apple_assertion_counter<S, T>(
    db: &T,
    wallet_id: &str,
    assertion_counter: AssertionCounter,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user_apple_attestation::Entity::update_many()
        .col_expr(
            wallet_user_apple_attestation::Column::AssertionCounter,
            Expr::value(i64::from(*assertion_counter)),
        )
        .filter(
            wallet_user_apple_attestation::Column::Id.in_subquery(
                Query::select()
                    .column(wallet_user::Column::AppleAttestationId)
                    .from(wallet_user::Entity)
                    .and_where(Expr::col(wallet_user::Column::WalletId).eq(wallet_id))
                    .to_owned(),
            ),
        )
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(())
}

pub async fn store_recovery_code<S, T>(db: &T, wallet_id: &str, recovery_code: String) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_user::Entity::update_many()
        .col_expr(wallet_user::Column::RecoveryCode, Expr::value(recovery_code))
        .filter(
            wallet_user::Column::WalletId
                .eq(wallet_id)
                .and(wallet_user::Column::RecoveryCode.is_null()),
        )
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_user`s with the same `wallet_id`"),
    }
}

pub async fn has_multiple_active_accounts_by_recovery_code<S, T>(db: &T, recovery_code: &str) -> Result<bool>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let count: u64 = wallet_user::Entity::find()
        .filter(
            wallet_user::Column::RecoveryCode
                .eq(recovery_code)
                .and(wallet_user::Column::State.ne(WalletUserState::Blocked.to_string())),
        )
        .count(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(count > 1)
}

pub async fn create_transfer_session<S, T>(
    db: &T,
    destination_wallet_user_id: Uuid,
    transfer_session_id: Uuid,
    destination_wallet_app_version: Version,
    created: DateTime<Utc>,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_transfer::ActiveModel {
        id: Set(Uuid::new_v4()),
        destination_wallet_user_id: Set(destination_wallet_user_id),
        transfer_session_id: Set(transfer_session_id),
        destination_wallet_app_version: Set(destination_wallet_app_version.to_string()),
        state: Set(TransferSessionState::Created.to_string()),
        created: Set(created.into()),
        encrypted_wallet_data: Set(None),
    }
    .insert(db.connection())
    .await
    .map_err(|e| PersistenceError::Execution(Box::new(e)))?;

    Ok(())
}

pub async fn find_transfer_session_by_transfer_session_id<S, T>(
    db: &T,
    transfer_session_id: Uuid,
) -> Result<Option<TransferSession>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::find()
        .filter(wallet_transfer::Column::TransferSessionId.eq(transfer_session_id))
        .find_also_related(wallet_user::Entity)
        .into_model::<wallet_transfer::Model, wallet_user::Model>()
        .one(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    let transfer_session = result.and_then(|(transfer_model, user_model)| {
        user_model.and_then(|user_model| {
            user_model
                .recovery_code
                .map(|recovery_code| transfer_session_from_model(transfer_model, recovery_code))
        })
    });

    Ok(transfer_session)
}

pub async fn update_transfer_state<S, T>(
    db: &T,
    transer_session_id: Uuid,
    transfer_session_state: TransferSessionState,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::update_many()
        .col_expr(
            wallet_transfer::Column::State,
            Expr::value(transfer_session_state.to_string()),
        )
        .filter(wallet_transfer::Column::TransferSessionId.eq(transer_session_id))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_transfer`s with the same `transfer_session_id`"),
    }
}

pub async fn clear_wallet_transfer_data<S, T>(db: &T, transer_session_id: Uuid) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_transfer::Entity::update_many()
        .col_expr(wallet_transfer::Column::EncryptedWalletData, Expr::cust("null"))
        .filter(wallet_transfer::Column::TransferSessionId.eq(transer_session_id))
        .exec(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_transfer`s with the same `transfer_session_id`"),
    }
}
