use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::model::{
    wallet_user::{InstructionChallenge, WalletUserCreate, WalletUserKeys, WalletUserQueryResult},
    wrapped_key::WrappedKey,
};

use super::{errors::PersistenceError, transaction::Committable};

type Result<T> = std::result::Result<T, PersistenceError>;

pub trait WalletUserRepository {
    type TransactionType: Committable;

    async fn create_wallet_user(&self, transaction: &Self::TransactionType, user: WalletUserCreate) -> Result<()>;

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
}

#[cfg(feature = "mock")]
pub mod mock {
    use uuid::Uuid;

    use crate::model::wallet_user;

    use super::{super::transaction::mock::MockTransaction, *};

    pub struct MockWalletUserRepository;

    impl WalletUserRepository for MockWalletUserRepository {
        type TransactionType = MockTransaction;

        async fn create_wallet_user(
            &self,
            _transaction: &Self::TransactionType,
            _user: WalletUserCreate,
        ) -> Result<()> {
            Ok(())
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
    }
}
