use platform_support::preferred;

use crate::{
    account_server::RemoteAccountServerClient,
    config::{AccountServerConfiguration, CONFIGURATION},
    storage::DatabaseStorage,
    wallet::WalletInitError,
};

pub type Wallet = crate::wallet::Wallet<
    RemoteAccountServerClient<'static, AccountServerConfiguration>,
    DatabaseStorage,
    preferred::PlatformEcdsaKey,
>;

pub async fn init_wallet() -> Result<Wallet, WalletInitError> {
    Wallet::new(
        RemoteAccountServerClient::new(&CONFIGURATION.account_server),
        CONFIGURATION.account_server.public_key.clone(),
        DatabaseStorage::default(),
    )
    .await
}
