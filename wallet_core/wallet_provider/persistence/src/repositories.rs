use std::collections::HashMap;
use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::From;
use p256::ecdsa::VerifyingKey;
use semver::Version;
use uuid;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;
use measure::measure;
use wallet_account::messages::errors::RevocationReason;
use wallet_account::messages::transfer::TransferSessionState;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::model::wallet_user::InstructionChallenge;
use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::model::wallet_user::TransferSession;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::model::wallet_user::WalletUserCreate;
use wallet_provider_domain::model::wallet_user::WalletUserIsRevoked;
use wallet_provider_domain::model::wallet_user::WalletUserKeys;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::database::Db;
use crate::recovery_code;
use crate::transaction;
use crate::transaction::Transaction;
use crate::wallet_transfer;
use crate::wallet_user;
use crate::wallet_user_key;
use crate::wallet_user_wua;

#[derive(From, AsRef)]
pub struct Repositories(Db);

impl TransactionStarter for Repositories {
    type TransactionType = Transaction;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
        transaction::begin_transaction(&self.0).await
    }
}

impl WalletUserRepository for Repositories {
    type TransactionType = Transaction;

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn list_wallet_user_ids(&self, transaction: &Self::TransactionType) -> Result<Vec<Uuid>, PersistenceError> {
        wallet_user::list_wallet_user_ids(transaction).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn list_wallets(
        &self,
        transaction: &Self::TransactionType,
    ) -> Result<Vec<WalletUserIsRevoked>, PersistenceError> {
        wallet_user::list_wallets(transaction).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<Uuid, PersistenceError> {
        wallet_user::create_wallet_user(transaction, user).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
    ) -> Result<WalletUserQueryResult, PersistenceError> {
        wallet_user::find_wallet_user_by_wallet_id(transaction, wallet_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_wallet_user_id_by_wallet_ids(
        &self,
        transaction: &Self::TransactionType,
        wallet_ids: &HashSet<WalletId>,
    ) -> Result<HashMap<WalletId, Uuid>, PersistenceError> {
        wallet_user::find_wallet_user_id_by_wallet_ids(transaction, wallet_ids.iter().map(AsRef::as_ref)).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_wallet_user_id_by_revocation_code(
        &self,
        transaction: &Self::TransactionType,
        revocation_code_hmac: &[u8],
    ) -> Result<QueryResult<Uuid>, PersistenceError> {
        wallet_user::find_wallet_user_id_by_revocation_code(transaction, revocation_code_hmac).await
    }

    async fn find_wallet_user_ids_by_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: &RecoveryCode,
    ) -> Result<Vec<Uuid>, PersistenceError> {
        wallet_user::find_wallet_user_ids_by_recovery_code(transaction, recovery_code).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn clear_instruction_challenge(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
    ) -> Result<(), PersistenceError> {
        wallet_user::clear_instruction_challenge(transaction, wallet_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        challenge: InstructionChallenge,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_challenge_and_sequence_number(
            transaction,
            wallet_id,
            challenge,
            instruction_sequence_number,
        )
        .await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn register_unsuccessful_pin_entry(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        is_blocked: bool,
        datetime: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        wallet_user::register_unsuccessful_pin_entry(transaction, wallet_id, is_blocked, datetime).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn reset_unsuccessful_pin_entries(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
    ) -> Result<(), PersistenceError> {
        wallet_user::reset_unsuccessful_pin_entries(transaction, wallet_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn save_keys(
        &self,
        transaction: &Self::TransactionType,
        keys: WalletUserKeys,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::persist_keys(transaction, keys).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn is_blocked_key(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<Option<bool>, PersistenceError> {
        wallet_user_key::is_blocked_key(transaction, wallet_user_id, key).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn delete_blocked_keys_in_batch(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::delete_blocked_keys_in_same_batch(transaction, wallet_user_id, key).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn delete_all_blocked_keys(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::delete_all_blocked_keys(transaction, wallet_user_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn unblock_blocked_keys_in_batch(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::unblock_blocked_keys_in_same_batch(transaction, wallet_user_id, key).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_active_keys_by_identifiers(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key_identifiers: &[String],
    ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
        wallet_user_key::find_active_keys_by_identifiers(transaction, wallet_user_id, key_identifiers).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn change_pin(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        new_encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        user_state: WalletUserState,
    ) -> Result<(), PersistenceError> {
        wallet_user::change_pin(transaction, wallet_id, new_encrypted_pin_pubkey, user_state).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn commit_pin_change(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
    ) -> Result<(), PersistenceError> {
        wallet_user::commit_pin_change(transaction, wallet_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn rollback_pin_change(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
    ) -> Result<(), PersistenceError> {
        wallet_user::rollback_pin_change(transaction, wallet_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn store_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        recovery_code: RecoveryCode,
    ) -> Result<(), PersistenceError> {
        wallet_user::store_recovery_code(transaction, wallet_id, recovery_code).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn recover_pin(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user::transition_wallet_user_state(
            transaction,
            wallet_user_id,
            WalletUserState::RecoveringPin,
            WalletUserState::Active,
        )
        .await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn has_multiple_active_accounts_by_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: &RecoveryCode,
    ) -> Result<bool, PersistenceError> {
        wallet_user::has_multiple_active_accounts_by_recovery_code(transaction, recovery_code).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn update_apple_assertion_counter(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &WalletId,
        assertion_counter: AssertionCounter,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_apple_assertion_counter(transaction, wallet_id, assertion_counter).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn create_transfer_session(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
        destination_wallet_app_version: Version,
        created: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        wallet_transfer::create_transfer_session(
            transaction,
            destination_wallet_user_id,
            transfer_session_id,
            destination_wallet_app_version,
            created,
        )
        .await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_transfer_session_by_transfer_session_id(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
    ) -> Result<Option<TransferSession>, PersistenceError> {
        wallet_transfer::find_transfer_session_by_transfer_session_id(transaction, transfer_session_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn find_transfer_session_id_by_destination_wallet_user_id(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
    ) -> Result<Option<Uuid>, PersistenceError> {
        wallet_transfer::find_transfer_session_id_by_destination_wallet_user_id(transaction, destination_wallet_user_id)
            .await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn pair_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_transfer::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Paired).await?;
        wallet_transfer::set_transfer_source(transaction, transfer_session_id, source_wallet_user_id).await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            source_wallet_user_id,
            WalletUserState::Active,
            WalletUserState::Transferring,
        )
        .await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            destination_wallet_user_id,
            WalletUserState::Active,
            WalletUserState::Transferring,
        )
        .await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn cancel_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Option<Uuid>,
        destination_wallet_user_id: Uuid,
        error: bool,
    ) -> Result<(), PersistenceError> {
        if let Some(wallet_user_id) = source_wallet_user_id {
            wallet_user::reset_wallet_user_state(transaction, wallet_user_id).await?;
        }
        wallet_user::reset_wallet_user_state(transaction, destination_wallet_user_id).await?;
        wallet_transfer::update_transfer_state(
            transaction,
            transfer_session_id,
            if error {
                TransferSessionState::Error
            } else {
                TransferSessionState::Canceled
            },
        )
        .await?;
        wallet_transfer::set_wallet_transfer_data(transaction, transfer_session_id, None).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn reset_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Option<Uuid>,
        destination_wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        if let Some(wallet_user_id) = source_wallet_user_id {
            wallet_user::reset_wallet_user_state(transaction, wallet_user_id).await?;
        }
        wallet_user::reset_wallet_user_state(transaction, destination_wallet_user_id).await?;
        wallet_transfer::set_wallet_transfer_data(transaction, transfer_session_id, None).await?;
        wallet_transfer::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Created).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn confirm_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_transfer::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Confirmed).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn store_wallet_transfer_data(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        encrypted_wallet_data: String,
    ) -> Result<(), PersistenceError> {
        wallet_transfer::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Uploaded)
            .await?;
        wallet_transfer::set_wallet_transfer_data(transaction, transfer_session_id, Some(encrypted_wallet_data)).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn complete_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::move_keys(transaction, source_wallet_user_id, destination_wallet_user_id).await?;
        wallet_transfer::set_wallet_transfer_data(transaction, transfer_session_id, None).await?;
        wallet_transfer::update_transfer_state(transaction, transfer_session_id, TransferSessionState::Success).await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            source_wallet_user_id,
            WalletUserState::Transferring,
            WalletUserState::Transferred,
        )
        .await?;
        wallet_user::transition_wallet_user_state(
            transaction,
            destination_wallet_user_id,
            WalletUserState::Transferring,
            WalletUserState::Active,
        )
        .await?;

        Ok(())
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn store_wua_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        wua_id: Uuid,
    ) -> Result<(), PersistenceError> {
        wallet_user_wua::create(transaction, wallet_user_id, wua_id).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn list_wua_ids(&self, transaction: &Self::TransactionType) -> Result<Vec<Uuid>, PersistenceError> {
        wallet_user_wua::list_wua_ids(transaction).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn revoke_wallet_users(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_ids: Vec<Uuid>,
        revocation_reason: RevocationReason,
        revocation_date_time: DateTime<Utc>,
    ) -> Result<Vec<Uuid>, PersistenceError> {
        wallet_user_key::delete_all_keys(transaction, wallet_user_ids.clone()).await?;
        wallet_user::revoke_wallets(
            transaction,
            wallet_user_ids.clone(),
            revocation_reason,
            revocation_date_time,
        )
        .await?;
        wallet_user_wua::find_wua_ids_for_wallet_users(transaction, wallet_user_ids).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn deny_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: RecoveryCode,
    ) -> Result<(), PersistenceError> {
        recovery_code::insert(transaction, recovery_code).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn recovery_code_is_denied(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: RecoveryCode,
    ) -> Result<bool, PersistenceError> {
        recovery_code::is_denied(transaction, recovery_code).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn list_denied_recovery_codes(
        &self,
        transaction: &Self::TransactionType,
    ) -> Result<Vec<RecoveryCode>, PersistenceError> {
        recovery_code::list(transaction).await
    }

    #[measure(name = "nlwallet_db_operations", "service" => "database")]
    async fn allow_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: &RecoveryCode,
    ) -> Result<bool, PersistenceError> {
        recovery_code::set_allowed(transaction, recovery_code).await
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::time::Duration;

    use chrono::DateTime;
    use chrono::Utc;
    use mockall;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;
    use semver::Version;
    use uuid::Uuid;
    use uuid::uuid;

    use apple_app_attest::AssertionCounter;
    use hsm::model::encrypted::Encrypted;
    use hsm::model::wrapped_key::WrappedKey;
    use wallet_account::messages::errors::RevocationReason;
    use wallet_provider_domain::model::QueryResult;
    use wallet_provider_domain::model::wallet_user::AndroidHardwareIdentifiers;
    use wallet_provider_domain::model::wallet_user::InstructionChallenge;
    use wallet_provider_domain::model::wallet_user::RecoveryCode;
    use wallet_provider_domain::model::wallet_user::RevocationRegistration;
    use wallet_provider_domain::model::wallet_user::TransferSession;
    use wallet_provider_domain::model::wallet_user::WalletId;
    use wallet_provider_domain::model::wallet_user::WalletUser;
    use wallet_provider_domain::model::wallet_user::WalletUserAttestation;
    use wallet_provider_domain::model::wallet_user::WalletUserCreate;
    use wallet_provider_domain::model::wallet_user::WalletUserIsRevoked;
    use wallet_provider_domain::model::wallet_user::WalletUserKeys;
    use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
    use wallet_provider_domain::model::wallet_user::WalletUserState;
    use wallet_provider_domain::model::wallet_user::mock::wallet_user_1;
    use wallet_provider_domain::model::wallet_user::mock::wallet_user_with_id;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_domain::repository::MockTransactionStarter;
    use wallet_provider_domain::repository::PersistenceError;
    use wallet_provider_domain::repository::TransactionStarter;
    use wallet_provider_domain::repository::WalletUserRepository;

    mockall::mock! {
        pub TransactionalWalletUserRepository {}

        impl WalletUserRepository for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn list_wallets(
                &self,
                transaction: &MockTransaction,
            ) -> Result<Vec<WalletUserIsRevoked>, PersistenceError>;

            async fn list_wallet_user_ids(
                &self,
                transaction: &MockTransaction,
            ) -> Result<Vec<Uuid>, PersistenceError>;

            async fn create_wallet_user(
                &self,
                transaction: &MockTransaction,
                user: WalletUserCreate,
            ) -> Result<Uuid, PersistenceError>;

            async fn find_wallet_user_by_wallet_id(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
            ) -> Result<WalletUserQueryResult, PersistenceError>;

            async fn find_wallet_user_id_by_wallet_ids(
                &self,
                transaction: &MockTransaction,
                wallet_ids: &HashSet<WalletId> ,
            ) -> Result<HashMap<WalletId, Uuid>, PersistenceError>;

            async fn find_wallet_user_id_by_revocation_code(
                &self,
                transaction: &MockTransaction,
                revocation_code_hmac: &[u8],
            ) -> Result<QueryResult<Uuid>, PersistenceError>;

            async fn find_wallet_user_ids_by_recovery_code(
                &self,
                transaction: &MockTransaction,
                recovery_code: &RecoveryCode,
            ) -> Result<Vec<Uuid>, PersistenceError>;

            async fn register_unsuccessful_pin_entry(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                is_blocked: bool,
                datetime: DateTime<Utc>,
            ) -> Result<(), PersistenceError>;

            async fn reset_unsuccessful_pin_entries(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
            ) -> Result<(), PersistenceError>;

            async fn clear_instruction_challenge(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_challenge_and_sequence_number(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                challenge: InstructionChallenge,
                instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_sequence_number(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn save_keys(
                &self,
                transaction: &MockTransaction,
                keys: WalletUserKeys,
            ) -> Result<(), PersistenceError>;

            async fn is_blocked_key(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key: VerifyingKey,
            ) -> Result<Option<bool>, PersistenceError>;

            async fn unblock_blocked_keys_in_batch(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key: VerifyingKey,
            ) -> Result<(), PersistenceError>;

            async fn delete_blocked_keys_in_batch(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key: VerifyingKey,
            ) -> Result<(), PersistenceError>;

            async fn delete_all_blocked_keys(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn find_active_keys_by_identifiers(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key_identifiers: &[String],
            ) -> Result<HashMap<String, WrappedKey>, PersistenceError>;

            async fn change_pin(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                encrypted_pin_pubkey: Encrypted<VerifyingKey>,
                user_state: WalletUserState,
            ) -> Result<(), PersistenceError>;

            async fn commit_pin_change(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId
            ) -> Result<(), PersistenceError>;

            async fn rollback_pin_change(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId
            ) -> Result<(), PersistenceError>;

            async fn update_apple_assertion_counter(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                assertion_counter: AssertionCounter,
            ) -> Result<(), PersistenceError>;

            async fn store_recovery_code(
                &self,
                transaction: &MockTransaction,
                wallet_id: &WalletId,
                recovery_code: RecoveryCode,
            ) -> Result<(), PersistenceError>;

            async fn recover_pin(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn has_multiple_active_accounts_by_recovery_code(
                &self,
                transaction: &MockTransaction,
                recovery_code: &RecoveryCode,
            ) -> Result<bool, PersistenceError>;

            async fn create_transfer_session(
                &self,
                transaction: &MockTransaction,
                destination_wallet_user_id: Uuid,
                transfer_session_id: Uuid,
                destination_wallet_app_version: Version,
                created: DateTime<Utc>,
            ) -> Result<(), PersistenceError>;

            async fn find_transfer_session_by_transfer_session_id(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
            ) -> Result<Option<TransferSession>, PersistenceError>;

            async fn find_transfer_session_id_by_destination_wallet_user_id(
                &self,
                transaction: &MockTransaction,
                destination_wallet_user_id: Uuid,
            ) -> Result<Option<Uuid>, PersistenceError>;

            async fn pair_wallet_transfer(&self,
                transaction: &MockTransaction,
                source_wallet_user_id: Uuid,
                destination_wallet_user_id: Uuid,
                transfer_session_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn cancel_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                source_wallet_user_id: Option<Uuid>,
                destination_wallet_user_id: Uuid,
                error: bool,
            ) -> Result<(), PersistenceError>;

            async fn reset_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                source_wallet_user_id: Option<Uuid>,
                destination_wallet_user_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn confirm_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn store_wallet_transfer_data(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                encrypted_wallet_data: String,
            ) -> Result<(), PersistenceError>;

            async fn complete_wallet_transfer(
                &self,
                transaction: &MockTransaction,
                transfer_session_id: Uuid,
                source_wallet_user_id: Uuid,
                destination_wallet_user_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn store_wua_id(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Uuid,
                wua_id: Uuid,
            ) -> Result<(), PersistenceError>;

            async fn list_wua_ids(
                &self,
                transaction: &MockTransaction,
            ) -> Result<Vec<Uuid>, PersistenceError>;

            async fn revoke_wallet_users(
                &self,
                transaction: &MockTransaction,
                wallet_user_id: Vec<Uuid>,
                revocation_reason: RevocationReason,
                revocation_date_time: DateTime<Utc>
            ) -> Result<Vec<Uuid>, PersistenceError>;

            async fn deny_recovery_code(
                &self,
                transaction: &MockTransaction,
                recovery_code: RecoveryCode,
            ) -> Result<(), PersistenceError>;

            async fn recovery_code_is_denied(
                &self,
                transaction: &MockTransaction,
                recovery_code: RecoveryCode,
            ) -> Result<bool, PersistenceError>;

            async fn list_denied_recovery_codes(&self, transaction: &MockTransaction) -> Result<Vec<RecoveryCode>, PersistenceError>;

            async fn allow_recovery_code(
                &self,
                transaction: &MockTransaction,
                recovery_code: &RecoveryCode,
            ) -> Result<bool, PersistenceError>;
        }

        impl TransactionStarter for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn begin_transaction(&self) -> Result<MockTransaction, PersistenceError>;
        }
    }

    #[derive(Clone)]
    pub struct WalletUserTestRepo {
        pub hw_pubkey: VerifyingKey,
        pub encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        pub previous_encrypted_pin_pubkey: Option<Encrypted<VerifyingKey>>,
        pub challenge: Option<Vec<u8>>,
        pub instruction_sequence_number: u64,
        pub apple_assertion_counter: Option<AssertionCounter>,
        pub state: WalletUserState,
        pub revocation_code_hmac: Vec<u8>,
        pub revocation_registration: Option<RevocationRegistration>,
    }

    impl WalletUserRepository for WalletUserTestRepo {
        type TransactionType = MockTransaction;

        async fn list_wallet_user_ids(
            &self,
            _transaction: &Self::TransactionType,
        ) -> Result<Vec<Uuid>, PersistenceError> {
            Ok(vec![
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                uuid!("a123f36e-ffbd-402f-b6f3-418cf4c49e09"),
            ])
        }

        async fn list_wallets(
            &self,
            _transaction: &Self::TransactionType,
        ) -> Result<Vec<WalletUserIsRevoked>, PersistenceError> {
            Ok(vec![
                wallet_user_1().into(),
                wallet_user_with_id("wallet-456".to_owned().into()).into(),
            ])
        }

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<Uuid, PersistenceError> {
            Ok(uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"))
        }

        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            wallet_id: &WalletId,
        ) -> Result<WalletUserQueryResult, PersistenceError> {
            Ok(QueryResult::Found(Box::new(WalletUser {
                id: uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                wallet_id: wallet_id.to_owned(),
                hw_pubkey: self.hw_pubkey,
                encrypted_pin_pubkey: self.encrypted_pin_pubkey.clone(),
                encrypted_previous_pin_pubkey: self.previous_encrypted_pin_pubkey.clone(),
                unsuccessful_pin_entries: 0,
                last_unsuccessful_pin_entry: None,
                instruction_challenge: self.challenge.clone().map(|c| InstructionChallenge {
                    bytes: c,
                    expiration_date_time: Utc::now() + Duration::from_millis(15000),
                }),
                instruction_sequence_number: self.instruction_sequence_number,
                attestation: match self.apple_assertion_counter {
                    Some(assertion_counter) => WalletUserAttestation::Apple { assertion_counter },
                    None => WalletUserAttestation::Android {
                        identifiers: AndroidHardwareIdentifiers::default(),
                    },
                },
                state: self.state,
                revocation_code_hmac: self.revocation_code_hmac.clone(),
                revocation_registration: self.revocation_registration,
                recovery_code: None,
                recovery_code_is_denied: false,
            })))
        }

        async fn find_wallet_user_id_by_wallet_ids(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &HashSet<WalletId>,
        ) -> Result<HashMap<WalletId, Uuid>, PersistenceError> {
            Ok([(
                WalletId::from("wallet-123".to_owned()),
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
            )]
            .into())
        }

        async fn find_wallet_user_id_by_revocation_code(
            &self,
            _transaction: &Self::TransactionType,
            _revocation_code_hmac: &[u8],
        ) -> Result<QueryResult<Uuid>, PersistenceError> {
            Ok(QueryResult::Found(uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08").into()))
        }

        async fn find_wallet_user_ids_by_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &RecoveryCode,
        ) -> Result<Vec<Uuid>, PersistenceError> {
            Ok([uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08")].into())
        }

        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn update_instruction_challenge_and_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
            _challenge: InstructionChallenge,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn update_instruction_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
            _instruction_sequence_number: u64,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
            _is_blocked: bool,
            _datetime: DateTime<Utc>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn reset_unsuccessful_pin_entries(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn save_keys(
            &self,
            _transaction: &Self::TransactionType,
            _keys: WalletUserKeys,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn is_blocked_key(
            &self,
            _transaction: &MockTransaction,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<Option<bool>, PersistenceError> {
            Ok(Some(true))
        }

        async fn delete_blocked_keys_in_batch(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn delete_all_blocked_keys(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn unblock_blocked_keys_in_batch(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn find_active_keys_by_identifiers(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            key_identifiers: &[String],
        ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
            Ok(key_identifiers
                .iter()
                .map(|id| {
                    let privkey = SigningKey::random(&mut OsRng);

                    (
                        id.clone(),
                        WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
                    )
                })
                .collect())
        }

        async fn change_pin(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
            _encrypted_pin_pubkey: Encrypted<VerifyingKey>,
            _user_state: WalletUserState,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn commit_pin_change(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn rollback_pin_change(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn store_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &WalletId,
            _recovery_code: RecoveryCode,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn recover_pin(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn has_multiple_active_accounts_by_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &RecoveryCode,
        ) -> Result<bool, PersistenceError> {
            Ok(false)
        }

        async fn update_apple_assertion_counter(
            &self,
            _transaction: &MockTransaction,
            _wallet_id: &WalletId,
            _assertion_counter: AssertionCounter,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn create_transfer_session(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
            _destination_wallet_app_version: Version,
            _created: DateTime<Utc>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn find_transfer_session_by_transfer_session_id(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
        ) -> Result<Option<TransferSession>, PersistenceError> {
            Ok(None)
        }

        async fn find_transfer_session_id_by_destination_wallet_user_id(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
        ) -> Result<Option<Uuid>, PersistenceError> {
            Ok(None)
        }

        async fn pair_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn cancel_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Option<Uuid>,
            _destination_wallet_user_id: Uuid,
            _error: bool,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn reset_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Option<Uuid>,
            _destination_wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn confirm_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn store_wallet_transfer_data(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _encrypted_wallet_data: String,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn complete_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn store_wua_id(
            &self,
            _transaction: &Self::TransactionType,
            _wua_id: Uuid,
            _wallet_user_id: Uuid,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn list_wua_ids(&self, _transaction: &Self::TransactionType) -> Result<Vec<Uuid>, PersistenceError> {
            Ok(vec![
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                uuid!("a123f36e-ffbd-402f-b6f3-418cf4c49e09"),
            ])
        }

        async fn revoke_wallet_users(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Vec<Uuid>,
            _revocation_reason: RevocationReason,
            _revocation_date_time: DateTime<Utc>,
        ) -> Result<Vec<Uuid>, PersistenceError> {
            Ok(vec![
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                uuid!("a123f36e-ffbd-402f-b6f3-418cf4c49e09"),
            ])
        }

        async fn deny_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: RecoveryCode,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn recovery_code_is_denied(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: RecoveryCode,
        ) -> Result<bool, PersistenceError> {
            Ok(false)
        }

        async fn list_denied_recovery_codes(
            &self,
            _transaction: &Self::TransactionType,
        ) -> Result<Vec<RecoveryCode>, PersistenceError> {
            Ok(vec![])
        }

        async fn allow_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &RecoveryCode,
        ) -> Result<bool, PersistenceError> {
            Ok(false)
        }
    }

    impl TransactionStarter for WalletUserTestRepo {
        type TransactionType = <MockTransactionStarter as TransactionStarter>::TransactionType;

        async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
            MockTransactionStarter.begin_transaction().await
        }
    }
}
