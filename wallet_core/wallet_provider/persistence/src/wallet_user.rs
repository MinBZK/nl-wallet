use chrono::{DateTime, Utc};
use p256::{
    ecdsa::VerifyingKey,
    pkcs8::{DecodePublicKey, EncodePublicKey},
};
use sea_orm::{
    sea_query::{Expr, IntoIden, OnConflict, Query, SimpleExpr},
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use wallet_common::account::serialization::DerVerifyingKey;
use wallet_provider_domain::{
    model::{
        encrypted::{Encrypted, InitializationVector},
        wallet_user::{InstructionChallenge, WalletUser, WalletUserCreate, WalletUserQueryResult},
    },
    repository::PersistenceError,
};

use crate::{
    entity::{wallet_user, wallet_user_instruction_challenge},
    PersistenceConnection,
};

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn create_wallet_user<S, T>(db: &T, user: WalletUserCreate) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::ActiveModel {
        id: Set(user.id),
        wallet_id: Set(user.wallet_id),
        hw_pubkey_der: Set(user.hw_pubkey.to_public_key_der()?.to_vec()),
        encrypted_pin_pubkey_sec1: Set(user.encrypted_pin_pubkey.data),
        pin_pubkey_iv: Set(user.encrypted_pin_pubkey.iv.0),
        encrypted_previous_pin_pubkey_sec1: Set(None),
        previous_pin_pubkey_iv: Set(None),
        instruction_sequence_number: Set(0),
        pin_entries: Set(0),
        last_unsuccessful_pin: Set(None),
        is_blocked: Set(false),
    }
    .insert(db.connection())
    .await
    .map(|_| ())
    .map_err(|e| PersistenceError::Execution(e.into()))
}

pub async fn find_wallet_user_by_wallet_id<S, T>(db: &T, wallet_id: &str) -> Result<WalletUserQueryResult>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let user_challenge = wallet_user::Entity::find()
        .find_also_related(wallet_user_instruction_challenge::Entity)
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .one(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    Ok(user_challenge
        .map(|(wallet_user, challenge)| {
            if wallet_user.is_blocked {
                WalletUserQueryResult::Blocked
            } else {
                WalletUserQueryResult::Found(Box::new(WalletUser {
                    id: wallet_user.id,
                    wallet_id: wallet_user.wallet_id,
                    encrypted_pin_pubkey: Encrypted::new(
                        wallet_user.encrypted_pin_pubkey_sec1,
                        InitializationVector(wallet_user.pin_pubkey_iv),
                    ),
                    encrypted_previous_pin_pubkey: wallet_user.encrypted_previous_pin_pubkey_sec1.and_then(|sec1| {
                        wallet_user
                            .previous_pin_pubkey_iv
                            .map(|iv| Encrypted::new(sec1, InitializationVector(iv)))
                    }),
                    hw_pubkey: DerVerifyingKey(VerifyingKey::from_public_key_der(&wallet_user.hw_pubkey_der).unwrap()),
                    unsuccessful_pin_entries: wallet_user.pin_entries.try_into().ok().unwrap_or(u8::MAX),
                    last_unsuccessful_pin_entry: wallet_user.last_unsuccessful_pin.map(DateTime::<Utc>::from),
                    instruction_challenge: challenge.map(|c| InstructionChallenge {
                        bytes: c.instruction_challenge,
                        expiration_date_time: DateTime::<Utc>::from(c.expiration_date_time),
                    }),
                    instruction_sequence_number: u64::try_from(wallet_user.instruction_sequence_number).unwrap(),
                }))
            }
        })
        .unwrap_or(WalletUserQueryResult::NotFound))
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
    wallet_user::Entity::update_many()
        .col_expr(wallet_user::Column::PinEntries, pin_entries)
        .col_expr(wallet_user::Column::LastUnsuccessfulPin, Expr::value(datetime))
        .col_expr(wallet_user::Column::IsBlocked, Expr::value(is_blocked))
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
