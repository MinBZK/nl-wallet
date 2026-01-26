use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use semver::Version;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;

use crate::model::wallet_user::InstructionChallenge;
use crate::model::wallet_user::RevocationReason;
use crate::model::wallet_user::TransferSession;
use crate::model::wallet_user::WalletUserCreate;
use crate::model::wallet_user::WalletUserKeys;
use crate::model::wallet_user::WalletUserQueryResult;
use crate::model::wallet_user::WalletUserState;

use super::errors::PersistenceError;
use super::transaction::Committable;

type Result<T> = std::result::Result<T, PersistenceError>;

pub trait WalletUserRepository {
    type TransactionType: Committable;

    async fn list_wallet_user_ids(&self, transaction: &Self::TransactionType) -> Result<Vec<Uuid>>;

    async fn list_wallet_ids(&self, transaction: &Self::TransactionType) -> Result<Vec<String>>;

    async fn create_wallet_user(&self, transaction: &Self::TransactionType, user: WalletUserCreate) -> Result<Uuid>;

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult>;

    async fn find_wallet_user_id_by_wallet_ids(
        &self,
        transaction: &Self::TransactionType,
        wallet_ids: &[String],
    ) -> Result<HashMap<String, Uuid>>;

    async fn clear_instruction_challenge(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        challenge: InstructionChallenge,
        instruction_sequence_number: u64,
    ) -> Result<()>;

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<()>;

    async fn register_unsuccessful_pin_entry(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        is_blocked: bool,
        datetime: DateTime<Utc>,
    ) -> Result<()>;

    async fn reset_unsuccessful_pin_entries(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn save_keys(&self, transaction: &Self::TransactionType, keys: WalletUserKeys) -> Result<()>;

    async fn is_blocked_key(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<Option<bool>>;

    async fn delete_blocked_keys_in_batch(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<()>;

    async fn delete_all_blocked_keys(&self, transaction: &Self::TransactionType, wallet_user_id: Uuid) -> Result<()>;

    async fn unblock_blocked_keys_in_batch(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key: VerifyingKey,
    ) -> Result<()>;

    async fn find_active_keys_by_identifiers(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key_identifiers: &[String],
    ) -> Result<HashMap<String, WrappedKey>>;

    async fn change_pin(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        encrypted_pin_pubkey: Encrypted<VerifyingKey>,
        user_state: WalletUserState,
    ) -> Result<()>;

    async fn commit_pin_change(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn rollback_pin_change(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn store_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        recovery_code: String,
    ) -> Result<()>;

    async fn recover_pin(&self, transaction: &Self::TransactionType, wallet_user_id: Uuid) -> Result<()>;

    async fn has_multiple_active_accounts_by_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        recovery_code: &str,
    ) -> Result<bool>;

    async fn update_apple_assertion_counter(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        assertion_counter: AssertionCounter,
    ) -> Result<()>;

    async fn create_transfer_session(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
        destination_wallet_app_version: Version,
        created: DateTime<Utc>,
    ) -> Result<()>;

    async fn find_transfer_session_by_transfer_session_id(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
    ) -> Result<Option<TransferSession>>;

    async fn find_transfer_session_id_by_destination_wallet_user_id(
        &self,
        transaction: &Self::TransactionType,
        destination_wallet_user_id: Uuid,
    ) -> Result<Option<Uuid>>;

    async fn pair_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
        transfer_session_id: Uuid,
    ) -> Result<()>;

    async fn cancel_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Option<Uuid>,
        destination_wallet_user_id: Uuid,
        error: bool,
    ) -> Result<()>;

    async fn reset_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Option<Uuid>,
        destination_wallet_user_id: Uuid,
    ) -> Result<()>;

    async fn confirm_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
    ) -> Result<()>;

    async fn store_wallet_transfer_data(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        encrypted_wallet_data: String,
    ) -> Result<()>;

    async fn complete_wallet_transfer(
        &self,
        transaction: &Self::TransactionType,
        transfer_session_id: Uuid,
        source_wallet_user_id: Uuid,
        destination_wallet_user_id: Uuid,
    ) -> Result<()>;

    async fn store_wua_id(&self, transaction: &Self::TransactionType, wallet_user_id: Uuid, wua_id: Uuid)
    -> Result<()>;

    async fn list_wua_ids(&self, transaction: &Self::TransactionType) -> Result<Vec<Uuid>>;

    async fn revoke_wallet_users(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Vec<Uuid>,
        revocation_reason: RevocationReason,
        revocation_date_time: DateTime<Utc>,
    ) -> Result<Vec<Uuid>>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use uuid::Uuid;
    use uuid::uuid;

    use crate::model::QueryResult;
    use crate::model::wallet_user;
    use crate::model::wallet_user::WalletUserQueryResult;

    use super::super::transaction::mock::MockTransaction;
    use super::*;

    pub struct WalletUserRepositoryStub;

    impl WalletUserRepository for WalletUserRepositoryStub {
        type TransactionType = MockTransaction;

        async fn list_wallet_ids(&self, _transaction: &Self::TransactionType) -> Result<Vec<String>> {
            Ok(vec!["wallet-123".to_string(), "wallet-456".to_string()])
        }

        async fn list_wallet_user_ids(&self, _transaction: &Self::TransactionType) -> Result<Vec<Uuid>> {
            Ok(vec![
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                uuid!("a123f36e-ffbd-402f-b6f3-418cf4c49e09"),
            ])
        }

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<Uuid> {
            Ok(uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"))
        }

        async fn find_wallet_user_by_wallet_id(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<WalletUserQueryResult> {
            Ok(QueryResult::Found(Box::new(wallet_user::mock::wallet_user_1())))
        }

        async fn find_wallet_user_id_by_wallet_ids(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_ids: &[String],
        ) -> Result<HashMap<String, Uuid>> {
            Ok([("wallet-123".to_owned(), uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"))].into())
        }

        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_instruction_challenge_and_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _challenge: InstructionChallenge,
            _instruction_sequence_number: u64,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_instruction_sequence_number(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _instruction_sequence_number: u64,
        ) -> Result<()> {
            Ok(())
        }

        async fn register_unsuccessful_pin_entry(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _is_blocked: bool,
            _datetime: DateTime<Utc>,
        ) -> Result<()> {
            Ok(())
        }

        async fn reset_unsuccessful_pin_entries(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
        ) -> Result<()> {
            Ok(())
        }

        async fn save_keys(&self, _transaction: &Self::TransactionType, _keys: WalletUserKeys) -> Result<()> {
            Ok(())
        }

        async fn is_blocked_key(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<Option<bool>> {
            Ok(Some(true))
        }

        async fn unblock_blocked_keys_in_batch(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<()> {
            Ok(())
        }

        async fn delete_blocked_keys_in_batch(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key: VerifyingKey,
        ) -> Result<()> {
            Ok(())
        }

        async fn delete_all_blocked_keys(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn find_active_keys_by_identifiers(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _key_identifiers: &[String],
        ) -> Result<HashMap<String, WrappedKey>> {
            Ok(HashMap::new())
        }

        async fn change_pin(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _encrypted_pin_pubkey: Encrypted<VerifyingKey>,
            _user_state: WalletUserState,
        ) -> Result<()> {
            Ok(())
        }

        async fn commit_pin_change(&self, _transaction: &Self::TransactionType, _wallet_id: &str) -> Result<()> {
            Ok(())
        }

        async fn rollback_pin_change(&self, _transaction: &Self::TransactionType, _wallet_id: &str) -> Result<()> {
            Ok(())
        }

        async fn store_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _recovery_code: String,
        ) -> Result<()> {
            Ok(())
        }

        async fn recover_pin(&self, _transaction: &Self::TransactionType, _wallet_user_id: Uuid) -> Result<()> {
            Ok(())
        }

        async fn has_multiple_active_accounts_by_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &str,
        ) -> Result<bool> {
            Ok(false)
        }

        async fn update_apple_assertion_counter(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _assertion_counter: AssertionCounter,
        ) -> Result<()> {
            Ok(())
        }

        async fn create_transfer_session(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
            _destination_wallet_app_version: Version,
            _created: DateTime<Utc>,
        ) -> Result<()> {
            Ok(())
        }

        async fn find_transfer_session_by_transfer_session_id(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
        ) -> Result<Option<TransferSession>> {
            Ok(None)
        }

        async fn find_transfer_session_id_by_destination_wallet_user_id(
            &self,
            _transaction: &Self::TransactionType,
            _destination_wallet_user_id: Uuid,
        ) -> Result<Option<Uuid>> {
            Ok(None)
        }

        async fn pair_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
            _transfer_session_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn cancel_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Option<Uuid>,
            _destination_wallet_user_id: Uuid,
            _error: bool,
        ) -> Result<()> {
            Ok(())
        }

        async fn reset_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Option<Uuid>,
            _destination_wallet_user_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn confirm_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn store_wallet_transfer_data(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _encrypted_wallet_data: String,
        ) -> Result<()> {
            Ok(())
        }

        async fn complete_wallet_transfer(
            &self,
            _transaction: &Self::TransactionType,
            _transfer_session_id: Uuid,
            _source_wallet_user_id: Uuid,
            _destination_wallet_user_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn store_wua_id(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_user_id: Uuid,
            _wua_id: Uuid,
        ) -> Result<()> {
            Ok(())
        }

        async fn list_wua_ids(&self, _transaction: &Self::TransactionType) -> Result<Vec<Uuid>> {
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
        ) -> Result<Vec<Uuid>> {
            Ok(vec![
                uuid!("d944f36e-ffbd-402f-b6f3-418cf4c49e08"),
                uuid!("a123f36e-ffbd-402f-b6f3-418cf4c49e09"),
            ])
        }
    }
}
