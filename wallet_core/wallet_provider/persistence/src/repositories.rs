use async_trait::async_trait;
use chrono::{DateTime, Local};
use wallet_provider_domain::model::wallet_user::{WalletUserCreate, WalletUserQueryResult};

use wallet_provider_domain::repository::{PersistenceError, TransactionStarter, WalletUserRepository};

use crate::{database::Db, transaction, transaction::Transaction, wallet_user_repository};

pub struct Repositories(Db);

impl Repositories {
    pub fn new(db: Db) -> Self {
        Self(db)
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
}
