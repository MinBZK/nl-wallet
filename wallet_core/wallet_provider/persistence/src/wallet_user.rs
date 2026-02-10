use std::collections::HashMap;
use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use p256::pkcs8::EncodePublicKey;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::JoinType;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use sea_orm::Set;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::IntoIden;
use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::Query;
use sea_orm::sea_query::SimpleExpr;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::encrypted::InitializationVector;
use wallet_account::messages::errors::RevocationReason;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::model::wallet_user::AndroidHardwareIdentifiers;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::RevocationRegistration;
use wallet_provider_domain::model::wallet_user::WalletUser;
use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
use wallet_provider_domain::model::wallet_user::WalletUserAttestationCreate;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::denied_recovery_code;
use crate::entity::wallet_user;
use crate::entity::wallet_user_android_attestation;
use crate::entity::wallet_user_apple_attestation;
use crate::entity::wallet_user_instruction_challenge;

type Result<T> = std::result::Result<T, PersistenceError>;

pub async fn list_wallet_ids<S, T>(db: &T) -> Result<Vec<String>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::WalletId)
        .into_tuple()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}

pub async fn list_wallet_user_ids<S, T>(db: &T) -> Result<Vec<Uuid>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::Id)
        .into_tuple()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
}

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
            .map_err(PersistenceError::Execution)?;

            (Some(id), None)
        }
        WalletUserAttestationCreate::Android {
            certificate_chain,
            integrity_verdict_json,
            identifiers,
        } => {
            let id = Uuid::new_v4();

            wallet_user_android_attestation::ActiveModel {
                id: Set(id),
                certificate_chain: Set(certificate_chain),
                integrity_verdict_json: Set(integrity_verdict_json),
                brand: Set(identifiers.brand),
                model: Set(identifiers.model),
                os_version: Set(identifiers.os_version.map(|os_version| {
                    u32::from(os_version)
                        .try_into()
                        .expect("OsVersion u32 should always fit in i32")
                })),
                os_patch_level: Set(identifiers.os_patch_level.map(|os_patch_level| {
                    u32::from(os_patch_level)
                        .try_into()
                        .expect("PatchLevel u32 should always fit in i32")
                })),
            }
            .insert(connection)
            .await
            .map_err(PersistenceError::Execution)?;

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
        revocation_reason: Set(None),
        revocation_date_time: Set(None),
        attestation_date_time: Set(user.attestation_date_time.into()),
        apple_attestation_id: Set(apple_attestation_id),
        android_attestation_id: Set(android_attestation_id),
        revocation_code_hmac: Set(user.revocation_code_hmac),
        recovery_code: Set(None),
    }
    .insert(connection)
    .await
    .map_err(PersistenceError::Execution)?;

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
    android_brand: Option<String>,
    android_device: Option<String>,
    android_os_version: Option<i32>,
    android_os_patch_level: Option<i32>,
    revocation_code_hmac: Vec<u8>,
    revocation_reason: Option<String>,
    revocation_date_time: Option<DateTimeWithTimeZone>,
    recovery_code: Option<String>,
    recovery_code_on_deny_list: bool,
}

/// Find a user by its `wallet_id` and return it, if it exists.
/// Note that this function will also return blocked users.
pub async fn find_wallet_user_by_wallet_id<S, T>(db: &T, wallet_id: &str) -> Result<WalletUserQueryResult>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let exists_query = Query::select()
        .column(denied_recovery_code::Column::Id)
        .from(denied_recovery_code::Entity)
        .and_where(
            Expr::col((denied_recovery_code::Entity, denied_recovery_code::Column::RecoveryCode))
                .eq(Expr::col((wallet_user::Entity, wallet_user::Column::RecoveryCode))),
        )
        .take();

    let Some(model) = wallet_user::Entity::find()
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
        .column(wallet_user::Column::RevocationCodeHmac)
        .column(wallet_user::Column::RevocationReason)
        .column(wallet_user::Column::RevocationDateTime)
        .column(wallet_user::Column::RecoveryCode)
        .expr_as(Expr::exists(exists_query), "recovery_code_on_deny_list")
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
        .column_as(wallet_user_android_attestation::Column::Brand, "android_brand")
        .column_as(wallet_user_android_attestation::Column::Model, "android_device")
        .column_as(wallet_user_android_attestation::Column::OsVersion, "android_os_version")
        .column_as(
            wallet_user_android_attestation::Column::OsPatchLevel,
            "android_os_patch_level",
        )
        .join(
            JoinType::LeftJoin,
            wallet_user::Relation::WalletUserInstructionChallenge.def(),
        )
        .join(
            JoinType::LeftJoin,
            wallet_user::Relation::WalletUserAppleAttestation.def(),
        )
        .join(
            JoinType::LeftJoin,
            wallet_user::Relation::WalletUserAndroidAttestation.def(),
        )
        .filter(wallet_user::Column::WalletId.eq(wallet_id))
        .into_model::<WalletUserJoinedModel>()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
    else {
        return Ok(QueryResult::NotFound);
    };

    let state: WalletUserState = model
        .state
        .parse()
        .expect("parsing the wallet user state from the database should always succeed");

    let encrypted_pin_pubkey = Encrypted::new(
        model.encrypted_pin_pubkey_sec1,
        InitializationVector(model.pin_pubkey_iv),
    );
    let encrypted_previous_pin_pubkey = match (model.encrypted_previous_pin_pubkey_sec1, model.previous_pin_pubkey_iv) {
        (Some(sec1), Some(iv)) => Some(Encrypted::new(sec1, InitializationVector(iv))),
        _ => None,
    };
    let instruction_challenge = match (
        model.instruction_challenge,
        model.instruction_challenge_expiration_date_time,
    ) {
        (Some(instruction_challenge), Some(expiration_date_time)) => Some(InstructionChallenge {
            bytes: instruction_challenge,
            expiration_date_time: DateTime::<Utc>::from(expiration_date_time),
        }),
        _ => None,
    };

    let attestation = match model.apple_assertion_counter {
        Some(counter) => WalletUserAttestation::Apple {
            assertion_counter: AssertionCounter::from(
                u32::try_from(counter).expect("assertion_counter should never be negative"),
            ),
        },
        // If the JOIN results in an assertion counter of NULL, we can safely assume that this
        // user has registered using an Android attestation instead. This is enforced by the
        // CHECK statement on the table.
        None => {
            let os_version = model.android_os_version.map(|os_version| {
                u32::try_from(os_version)
                    .expect("os_version should never be negative")
                    .try_into()
                    .expect("os_version should parse correctly")
            });
            let os_patch_level = model.android_os_patch_level.map(|os_patch_level| {
                u32::try_from(os_patch_level)
                    .expect("os_patch_level should never be negative")
                    .try_into()
                    .expect("os_patch_level should parse correctly")
            });

            WalletUserAttestation::Android {
                identifiers: AndroidHardwareIdentifiers {
                    brand: model.android_brand,
                    model: model.android_device,
                    os_version,
                    os_patch_level,
                },
            }
        }
    };

    let revocation_registration = match (model.revocation_reason, model.revocation_date_time) {
        (Some(reason), Some(date_time)) => Some(RevocationRegistration {
            reason: reason.parse().unwrap(),
            date_time: date_time.into(),
        }),
        (None, None) => None,
        _ => panic!("every reason should have a registered datetime"),
    };

    let wallet_user = WalletUser {
        id: model.id,
        wallet_id: model.wallet_id,
        encrypted_pin_pubkey,
        encrypted_previous_pin_pubkey,
        hw_pubkey: VerifyingKey::from_public_key_der(&model.hw_pubkey_der).unwrap(),
        unsuccessful_pin_entries: model.pin_entries.try_into().ok().unwrap_or(u8::MAX),
        last_unsuccessful_pin_entry: model.last_unsuccessful_pin.map(DateTime::<Utc>::from),
        instruction_challenge,
        instruction_sequence_number: u64::try_from(model.instruction_sequence_number).unwrap(),
        attestation,
        state,
        revocation_code_hmac: model.revocation_code_hmac,
        revocation_registration,
        recovery_code: model.recovery_code.clone(),
        recovery_code_on_deny_list: model.recovery_code_on_deny_list,
    };

    Ok(QueryResult::Found(Box::new(wallet_user)))
}

pub async fn find_wallet_user_id_by_wallet_ids<S, T>(
    db: &T,
    wallet_ids: &HashSet<String>,
) -> Result<HashMap<String, Uuid>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    Ok(wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::Id)
        .column(wallet_user::Column::WalletId)
        .filter(wallet_user::Column::WalletId.is_in(wallet_ids))
        .into_tuple::<(Uuid, String)>()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .into_iter()
        .map(|(wallet_user_id, wallet_id)| (wallet_id, wallet_user_id))
        .collect())
}

pub async fn find_wallet_user_id_by_revocation_code<S, T>(
    db: &T,
    revocation_code_hmac: &[u8],
) -> Result<QueryResult<Uuid>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::Id)
        .filter(wallet_user::Column::RevocationCodeHmac.eq(revocation_code_hmac))
        .into_tuple::<Uuid>()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result {
        Some(wallet_user_id) => Ok(QueryResult::Found(Box::new(wallet_user_id))),
        None => Ok(QueryResult::NotFound),
    }
}

pub async fn find_wallet_user_ids_by_recovery_code<S, T>(db: &T, recovery_code: &str) -> Result<Vec<Uuid>>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .select_only()
        .column(wallet_user::Column::Id)
        .filter(wallet_user::Column::RecoveryCode.eq(recovery_code))
        .into_tuple::<Uuid>()
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)
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
        .map_err(PersistenceError::Execution)?;

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
        // this only occurs if the number of selected values do not match the number of columns
        .expect("number of columns should match number of values")
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
        .map_err(PersistenceError::Execution)?;

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

pub async fn change_pin<S, T>(
    db: &T,
    wallet_id: &str,
    new_encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    user_state: WalletUserState,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let mut fields = vec![
        (
            wallet_user::Column::EncryptedPinPubkeySec1,
            Expr::value(new_encrypted_pin_pubkey.data),
        ),
        (
            wallet_user::Column::PinPubkeyIv,
            Expr::value(new_encrypted_pin_pubkey.iv.0),
        ),
        (wallet_user::Column::State, Expr::value(user_state.to_string())),
    ];

    if !matches!(user_state, WalletUserState::RecoveringPin) {
        // During and after PIN recovery, the user's previous PIN is never needed anymore.
        fields.push((
            wallet_user::Column::EncryptedPreviousPinPubkeySec1,
            Expr::col(wallet_user::Column::EncryptedPinPubkeySec1).into(),
        ));
        fields.push((
            wallet_user::Column::PreviousPinPubkeyIv,
            Expr::col(wallet_user::Column::PinPubkeyIv).into(),
        ));
    }

    update_fields(db, wallet_id, fields).await
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
        .map_err(PersistenceError::Execution)
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
        .map_err(PersistenceError::Execution)
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
        .map_err(PersistenceError::Execution)
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
        .map_err(PersistenceError::Execution)
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
        .map_err(PersistenceError::Execution)?;

    Ok(())
}

pub async fn transition_wallet_user_state<S, T>(
    db: &T,
    wallet_user_id: Uuid,
    from_state: WalletUserState,
    to_state: WalletUserState,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    // revocation is irreversible
    assert!(
        from_state != WalletUserState::Revoked,
        "Wallet user is in a revoked state, cannot transition to: '{}'",
        to_state
    );

    let result = wallet_user::Entity::update_many()
        .col_expr(wallet_user::Column::State, Expr::value(to_state.to_string()))
        .filter(
            wallet_user::Column::Id
                .eq(wallet_user_id)
                .and(wallet_user::Column::State.eq(from_state.to_string())),
        )
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_user`s with the same `wallet_id`"),
    }
}

pub async fn reset_wallet_user_state<S, T>(db: &T, wallet_user_id: Uuid) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let result = wallet_user::Entity::update_many()
        .col_expr(
            wallet_user::Column::State,
            Expr::value(WalletUserState::Active.to_string()),
        )
        .filter(
            wallet_user::Column::Id
                .eq(wallet_user_id)
                // revocation is irreversible
                .and(wallet_user::Column::State.ne(WalletUserState::Revoked.to_string())),
        )
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    match result.rows_affected {
        0 => Err(PersistenceError::NoRowsUpdated),
        1 => Ok(()),
        _ => panic!("multiple `wallet_user`s with the same `wallet_id`"),
    }
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
        .map_err(PersistenceError::Execution)?;

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
                .and(wallet_user::Column::State.is_not_in([
                    WalletUserState::Transferred.to_string(),
                    WalletUserState::Revoked.to_string(),
                ])),
        )
        .count(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(count > 1)
}

pub async fn revoke_wallets<S, T>(
    db: &T,
    wallet_user_ids: Vec<Uuid>,
    revocation_reason: RevocationReason,
    revocation_date_time: DateTime<Utc>,
) -> Result<()>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::update_many()
        .col_expr(
            wallet_user::Column::State,
            Expr::value(WalletUserState::Revoked.to_string()),
        )
        .col_expr(
            wallet_user::Column::RevocationReason,
            Expr::value(revocation_reason.to_string()),
        )
        .col_expr(
            wallet_user::Column::RevocationDateTime,
            Expr::value(revocation_date_time),
        )
        .filter(
            wallet_user::Column::Id
                .is_in(wallet_user_ids)
                .and(wallet_user::Column::State.ne(WalletUserState::Revoked.to_string()))
                .and(wallet_user::Column::RevocationReason.is_null())
                .and(wallet_user::Column::RevocationDateTime.is_null()),
        )
        .exec(db.connection())
        .await
        .map_err(PersistenceError::Execution)?;

    Ok(())
}
