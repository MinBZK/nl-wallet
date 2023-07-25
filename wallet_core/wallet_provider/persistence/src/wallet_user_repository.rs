use chrono::{DateTime, Local, Utc};
use p256::{ecdsa::VerifyingKey, pkcs8::DecodePublicKey};
use sea_orm::{
    sea_query::{Expr, IntoIden, SimpleExpr},
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};

use wallet_common::account::serialization::DerVerifyingKey;
use wallet_provider_domain::{
    model::wallet_user::{WalletUser, WalletUserCreate},
    repository::PersistenceError,
};

use crate::{entity::wallet_user, PersistenceConnection};

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn create_wallet_user<S, T>(db: &T, user: WalletUserCreate) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::ActiveModel {
        id: Set(user.id),
        wallet_id: Set(user.wallet_id),
        hw_pubkey_der: Set(user.hw_pubkey_der),
        pin_pubkey_der: Set(user.pin_pubkey_der),
        instruction_sequence_number: Set(0),
        instruction_challenge: Set(None),
        pin_entries: Set(0),
        last_unsuccessful_pin: Set(None),
        is_blocked: Set(false),
    }
    .insert(db.connection())
    .await
    .map(|_| ())
    .map_err(|e| PersistenceError::Execution(e.into()))
}

pub async fn find_wallet_user_by_wallet_id<S, T>(db: &T, wallet_id: &str) -> Result<WalletUser>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let user = wallet_user::Entity::find()
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .one(db.connection())
        .await
        .map_err(|e| PersistenceError::Execution(e.into()))?;

    user.map(|model| {
        Ok(WalletUser {
            id: model.id,
            wallet_id: model.wallet_id.to_string(),
            pin_pubkey: DerVerifyingKey(VerifyingKey::from_public_key_der(&model.pin_pubkey_der).unwrap()),
            hw_pubkey: DerVerifyingKey(VerifyingKey::from_public_key_der(&model.hw_pubkey_der).unwrap()),
            unsuccessful_pin_entries: model.pin_entries.try_into().ok().unwrap_or(u8::MAX),
            last_unsuccessful_pin_entry: model.last_unsuccessful_pin.map(DateTime::<Local>::from),
            instruction_challenge: model.instruction_challenge,
            instruction_sequence_number: u64::try_from(model.instruction_sequence_number).unwrap(),
        })
    })
    .ok_or(PersistenceError::NotFound(format!(
        "wallet_user with wallet_id: {}",
        wallet_id
    )))?
}
pub async fn clear_instruction_challenge<S, T>(db: &T, wallet_id: &str) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let challenge: Option<Vec<u8>> = None;

    update_fields(
        db,
        wallet_id,
        vec![(wallet_user::Column::InstructionChallenge, Expr::value(challenge))],
    )
    .await
}

pub async fn update_instruction_challenge_and_sequence_number<S, T>(
    db: &T,
    wallet_id: &str,
    challenge: Option<Vec<u8>>,
    instruction_sequence_number: u64,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_fields(
        db,
        wallet_id,
        vec![
            (wallet_user::Column::InstructionChallenge, Expr::value(challenge)),
            (
                wallet_user::Column::InstructionSequenceNumber,
                Expr::value(instruction_sequence_number),
            ),
        ],
    )
    .await
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
    datetime: DateTime<Local>,
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
        Some(datetime.into()),
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
