use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use p256::ecdsa::SigningKey;

use wallet_provider_domain::{
    model::wallet_user::{WalletUserCreate, WalletUserQueryResult},
    repository::{PersistenceError, TransactionStarter, WalletUserRepository},
};

use crate::{database::Db, transaction, transaction::Transaction, wallet_user_repository};

pub struct Repositories(Db, pub Mutex<HashMap<String, SigningKey>>);

impl Repositories {
    pub fn new(db: Db) -> Self {
        Self(db, Mutex::new(HashMap::new()))
    }
}

#[async_trait]
impl TransactionStarter for Repositories {
    type TransactionType = Transaction;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
        transaction::begin_transaction(&self.0).await
    }
}

#[async_trait]
impl WalletUserRepository for Repositories {
    type TransactionType = Transaction;

    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::create_wallet_user(transaction, user).await
    }

    async fn find_wallet_user_by_wallet_id(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<WalletUserQueryResult, PersistenceError> {
        wallet_user_repository::find_wallet_user_by_wallet_id(transaction, wallet_id).await
    }

    async fn clear_instruction_challenge(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::clear_instruction_challenge(transaction, wallet_id).await
    }

    async fn update_instruction_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::update_instruction_sequence_number(transaction, wallet_id, instruction_sequence_number)
            .await
    }

    async fn update_instruction_challenge_and_sequence_number(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
        challenge: Option<Vec<u8>>,
        instruction_sequence_number: u64,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::update_instruction_challenge_and_sequence_number(
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
        datetime: DateTime<Local>,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::register_unsuccessful_pin_entry(transaction, wallet_id, is_blocked, datetime).await
    }

    async fn reset_unsuccessful_pin_entries(
        &self,
        transaction: &Self::TransactionType,
        wallet_id: &str,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::reset_unsuccessful_pin_entries(transaction, wallet_id).await
    }

    async fn save_key(
        &self,
        _transaction: &Self::TransactionType,
        _wallet_id: &str,
        keys: &[(String, SigningKey)],
    ) -> Result<(), PersistenceError> {
        let mut data = self.1.lock().unwrap();
        for (key_identifier, private_key) in keys {
            if !data.contains_key(key_identifier) {
                data.insert(key_identifier.to_string(), private_key.clone());
            }
        }
        Ok(())
    }

    async fn get_key(
        &self,
        _transaction: &Self::TransactionType,
        _wallet_id: &str,
        key_identifier: &str,
    ) -> Result<Option<SigningKey>, PersistenceError> {
        Ok(self.1.lock().unwrap().get(key_identifier).cloned())
    }

    async fn get_keys<T: AsRef<str> + Sync>(
        &self,
        _transaction: &Self::TransactionType,
        _wallet_id: &str,
        key_identifiers: &[T],
    ) -> Result<Vec<Option<SigningKey>>, PersistenceError> {
        let data = self.1.lock().unwrap();

        let existing_keys = key_identifiers
            .iter()
            .map(|key_identifier| data.get(key_identifier.as_ref()).cloned())
            .collect();

        Ok(existing_keys)
    }
}
