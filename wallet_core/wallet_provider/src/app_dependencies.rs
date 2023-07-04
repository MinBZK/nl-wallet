use std::error::Error;

use async_trait::async_trait;
use uuid::Uuid;

use wallet_common::utils::random_bytes;
use wallet_provider_domain::{
    generator::Generator,
    model::wallet_user::WalletUserCreate,
    repository::{PersistenceError, TransactionStarter, WalletUserRepository},
};
use wallet_provider_persistence::{
    database::Db,
    transaction::{self, Transaction},
    wallet_user_repository,
};
use wallet_provider_service::account_server::AccountServer;

use crate::settings::Settings;

pub struct AppDependencies {
    pub account_server: AccountServer,
    db: Db,
}

impl AppDependencies {
    pub async fn new_from_settings(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let account_server = AccountServer::new(
            settings.signing_private_key.0,
            random_bytes(32),
            "account_server".into(),
        )?;
        let db = Db::new(
            &settings.database.host,
            &settings.database.name,
            settings.database.username.as_deref(),
            settings.database.password.as_deref(),
        )
        .await?;

        let dependencies = AppDependencies { account_server, db };

        Ok(dependencies)
    }
}

impl Generator<uuid::Uuid> for AppDependencies {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[async_trait]
impl TransactionStarter for AppDependencies {
    type TransactionType = Transaction;

    async fn begin_transaction(&self) -> Result<Self::TransactionType, PersistenceError> {
        transaction::begin_transaction(&self.db).await
    }
}

#[async_trait]
impl WalletUserRepository for AppDependencies {
    type TransactionType = Transaction;

    async fn create_wallet_user(
        &self,
        transaction: &Self::TransactionType,
        user: WalletUserCreate,
    ) -> Result<(), PersistenceError> {
        wallet_user_repository::create_wallet_user(transaction, user).await
    }
}
