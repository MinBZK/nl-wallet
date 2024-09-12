use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::{self, Uuid};

use wallet_provider_domain::{
    model::{
        wallet_user::{InstructionChallenge, WalletUserCreate, WalletUserKeys, WalletUserQueryResult},
        wrapped_key::WrappedKey,
    },
    repository::{PersistenceError, TransactionStarter, WalletUserRepository},
};

use crate::{database::Db, transaction, transaction::Transaction, wallet_user, wallet_user_key};

pub struct Repositories(Db);

impl Repositories {
    pub fn new(db: Db) -> Self {
        Self(db)
    }
}

impl TransactionStarter for Repositories {
    type TransactionType = Transaction;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
        transaction::begin_transaction(&self.0).await
    }
}

impl WalletUserRepository for Repositories {
    type TransactionType = Transaction;

    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<(), PersistenceError> {
        wallet_user::create_wallet_user(transaction, user).await
    }

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult, PersistenceError> {
        wallet_user::find_wallet_user_by_wallet_id(transaction, wallet_id).await
    }

    async fn clear_instruction_challenge(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::clear_instruction_challenge(transaction, wallet_id).await
    }

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user::update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number).await
    }

    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
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

    async fn register_unsuccessful_pin_entry(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        is_blocked: bool,
        datetime: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        wallet_user::register_unsuccessful_pin_entry(transaction, wallet_id, is_blocked, datetime).await
    }

    async fn reset_unsuccessful_pin_entries(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user::reset_unsuccessful_pin_entries(transaction, wallet_id).await
    }

    async fn save_keys(
        &self,
        transaction: &Self::TransactionType,
        keys: WalletUserKeys,
    ) -> Result<(), PersistenceError> {
        wallet_user_key::create_keys(transaction, keys).await
    }

    async fn find_keys_by_identifiers(
        &self,
        transaction: &Self::TransactionType,
        wallet_user_id: Uuid,
        key_identifiers: &[String],
    ) -> Result<HashMap<String, WrappedKey>, PersistenceError> {
        wallet_user_key::find_keys_by_identifiers(transaction, wallet_user_id, key_identifiers).await
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use chrono::{DateTime, Utc};
    use mockall;
    use std::collections::HashMap;
    use uuid::Uuid;

    use wallet_provider_domain::{
        model::{
            wallet_user::{InstructionChallenge, WalletUserCreate, WalletUserKeys, WalletUserQueryResult},
            wrapped_key::WrappedKey,
        },
        repository::{MockTransaction, PersistenceError, TransactionStarter, WalletUserRepository},
    };

    mockall::mock! {
        pub TransactionalWalletUserRepository {}

        impl WalletUserRepository for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn create_wallet_user(
                &self,
                transaction: &MockTransaction,
                user: WalletUserCreate,
            ) -> Result<(), PersistenceError>;

            async fn find_wallet_user_by_wallet_id(
                &self,
                _transaction: &MockTransaction,
                wallet_id: &str,
            ) -> Result<WalletUserQueryResult, PersistenceError>;

            async fn register_unsuccessful_pin_entry(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _is_blocked: bool,
                _datetime: DateTime<Utc>,
            ) -> Result<(), PersistenceError>;

            async fn reset_unsuccessful_pin_entries(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
            ) -> Result<(), PersistenceError>;

            async fn clear_instruction_challenge(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_challenge_and_sequence_number(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _challenge: InstructionChallenge,
                _instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn update_instruction_sequence_number(
                &self,
                _transaction: &MockTransaction,
                _wallet_id: &str,
                _instruction_sequence_number: u64,
            ) -> Result<(), PersistenceError>;

            async fn save_keys(
                &self,
                _transaction: &MockTransaction,
                _keys: WalletUserKeys,
            ) -> Result<(), PersistenceError>;

            async fn find_keys_by_identifiers(
                &self,
                _transaction: &MockTransaction,
                wallet_user_id: Uuid,
                key_identifiers: &[String],
            ) -> Result<HashMap<String, WrappedKey>, PersistenceError>;
        }

        impl TransactionStarter for TransactionalWalletUserRepository {
            type TransactionType = MockTransaction;

            async fn begin_transaction(&self) -> Result<MockTransaction, PersistenceError>;
        }
    }
}
