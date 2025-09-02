use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use p256::ecdsa::VerifyingKey;
use uuid::Uuid;

use apple_app_attest::AssertionCounter;
use hsm::model::encrypted::Encrypted;
use hsm::model::wrapped_key::WrappedKey;

use crate::model::wallet_user::InstructionChallenge;
use crate::model::wallet_user::WalletUserCreate;
use crate::model::wallet_user::WalletUserKeys;
use crate::model::wallet_user::WalletUserQueryResult;

use super::errors::PersistenceError;
use super::transaction::Committable;

type Result<T> = std::result::Result<T, PersistenceError>;

pub trait WalletUserRepository {
    type TransactionType: Committable;

    async fn create_wallet_user(&self, transaction: &Self::TransactionType, user: WalletUserCreate) -> Result<Uuid>;

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult>;

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

    async fn find_keys_by_identifiers(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: uuid::Uuid,
        key_identifiers: &[String],
    ) -> Result<HashMap<String, WrappedKey>>;

    async fn change_pin(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        encrypted_pin_pubkey: Encrypted<VerifyingKey>,
    ) -> Result<()>;

    async fn commit_pin_change(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn rollback_pin_change(&self, transaction: &Self::TransactionType, wallet_id: &str) -> Result<()>;

    async fn store_recovery_code(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        recovery_code: String,
    ) -> Result<()>;

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
}

#[cfg(feature = "mock")]
pub mod mock {
    use uuid::Uuid;
    use uuid::uuid;

    use crate::model::wallet_user;

    use super::super::transaction::mock::MockTransaction;
    use super::*;

    pub struct MockWalletUserRepository;

    impl WalletUserRepository for MockWalletUserRepository {
        type TransactionType = MockTransaction;

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
            Ok(WalletUserQueryResult::Found(Box::new(
                wallet_user::mock::wallet_user_1(),
            )))
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

        async fn clear_instruction_challenge(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
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

        async fn find_keys_by_identifiers(
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
        ) -> Result<()> {
            Ok(())
        }

        async fn commit_pin_change(&self, _transaction: &Self::TransactionType, _wallet_id: &str) -> Result<()> {
            Ok(())
        }

        async fn rollback_pin_change(&self, _transaction: &Self::TransactionType, _wallet_id: &str) -> Result<()> {
            Ok(())
        }

        async fn update_apple_assertion_counter(
            &self,
            _transaction: &Self::TransactionType,
            _wallet_id: &str,
            _assertion_counter: AssertionCounter,
        ) -> Result<()> {
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

        async fn has_multiple_active_accounts_by_recovery_code(
            &self,
            _transaction: &Self::TransactionType,
            _recovery_code: &str,
        ) -> crate::repository::wallet_user_repository::Result<bool> {
            Ok(false)
        }
    }
}
